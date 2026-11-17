//! Xuat DOCX (OOXML) theo dung schema ISO/IEC 29500 de Microsoft Word mo duoc.
//!
//! Cac loi gay "file bi hong" o ban cu da duoc sua:
//!  * `<w:numPr>` dung `<w:numId>` (truoc day ghi nham `<w:num>`)
//!  * thu tu phan tu con cua pPr / rPr / tcPr / tblPr dung schema
//!  * loai bo ky tu dieu khien khong hop le trong XML 1.0
//!  * Target cua hyperlink duoc ma hoa URI (khoang trang, tieng Viet...)
//!  * co day du docProps, settings, [Content_Types]
//!  * cong thuc LaTeX -> OMML (Equation cua Word, sua duoc)

use crate::math;
use crate::media;
use crate::{Align, Block, Span};
use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};

const NS_W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
const NS_R: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";
const NS_M: &str = "http://schemas.openxmlformats.org/officeDocument/2006/math";
const REL_HYPERLINK: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink";
const REL_IMAGE: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image";

/// Thoat ky tu XML, bo ky tu khong hop le (control chars, U+FFFE/FFFF, surrogate khong the xuat hien trong &str).
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\t' | '\n' | '\r' => out.push(c),
            c if (c as u32) < 0x20 => {}
            '\u{FFFE}' | '\u{FFFF}' => {}
            c if (0x7F..=0x9F).contains(&(c as u32)) && c != '\u{85}' => {}
            _ => out.push(c),
        }
    }
    out
}

/// Ma hoa phan tram cho URI dich (giu nguyen ky tu hop le & ky tu da ma hoa).
pub fn encode_uri(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        let keep = c.is_ascii_alphanumeric()
            || matches!(c, b'-' | b'.' | b'_' | b'~' | b':' | b'/' | b'?' | b'#' | b'[' | b']' | b'@' | b'!' | b'$' | b'&' | b'\'' | b'(' | b')' | b'*' | b'+' | b',' | b';' | b'=');
        let pct = c == b'%' && i + 2 < b.len() + 0 && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit();
        if keep || pct {
            out.push(c as char);
        } else {
            out.push_str(&format!("%{:02X}", c));
        }
        i += 1;
    }
    out
}

struct Ctx {
    rels: Vec<(String, &'static str, String)>, // (id, type, target)
    media: Vec<(String, Vec<u8>)>,
    nums: Vec<bool>, // moi list = 1 numId; true = danh so
    next_rel: usize,
    next_docpr: usize,
    doc_dir: PathBuf,
    /// anh http(s) da tai san (url -> byte)
    remote: HashMap<String, Vec<u8>>,
    /// be rong toi da cua anh hien tai (EMU): 16.5 cm trong van ban, hep hon trong o bang
    max_img_emu: f64,
}

const PAGE_IMG_EMU: f64 = 5_940_000.0;
const MAX_IMG_H_EMU: f64 = 8_000_000.0; // ~22 cm

impl Ctx {
    fn add_rel(&mut self, ty: &'static str, target: &str) -> String {
        self.next_rel += 1;
        let id = format!("rId{}", self.next_rel + 10);
        self.rels.push((id.clone(), ty, target.to_string()));
        id
    }
}

fn run_props(bold: bool, italic: bool, strike: bool, code: bool, color: Option<&str>, underline: bool) -> String {
    // THU TU theo schema: rFonts, b, i, strike, color, sz, u, shd
    let mut rpr = String::new();
    if code {
        rpr.push_str("<w:rFonts w:ascii=\"Consolas\" w:hAnsi=\"Consolas\" w:cs=\"Consolas\" w:eastAsia=\"Consolas\"/>");
    }
    if bold {
        rpr.push_str("<w:b/><w:bCs/>");
    }
    if italic {
        rpr.push_str("<w:i/><w:iCs/>");
    }
    if strike {
        rpr.push_str("<w:strike/>");
    }
    if let Some(c) = color {
        rpr.push_str(&format!("<w:color w:val=\"{c}\"/>"));
    }
    if code {
        rpr.push_str("<w:sz w:val=\"20\"/><w:szCs w:val=\"20\"/>");
    }
    if underline {
        rpr.push_str("<w:u w:val=\"single\"/>");
    }
    if code {
        rpr.push_str("<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F2F2F2\"/>");
    }
    if rpr.is_empty() {
        String::new()
    } else {
        format!("<w:rPr>{rpr}</w:rPr>")
    }
}

fn text_runs(text: &str, rpr: &str) -> String {
    let mut out = String::new();
    for (i, part) in text.split('\n').enumerate() {
        if i > 0 {
            out.push_str(&format!("<w:r>{rpr}<w:br/></w:r>"));
        }
        if part.is_empty() {
            continue;
        }
        out.push_str(&format!("<w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r>", xml_escape(part)));
    }
    out
}

fn image_run(alt: &str, dest: &str, ctx: &mut Ctx) -> String {
    let note = |s: String| text_runs(&s, &run_props(false, true, false, false, Some("7F7F7F"), false));
    // PNG / JPEG / GIF giu nguyen; WebP, BMP... doi sang PNG; nguon: tep cuc bo, data:, http(s) da tai
    let Some(img) = media::load_for_export(dest, &ctx.doc_dir, &ctx.remote) else {
        let shown = if dest.starts_with("data:") { "data:…" } else { dest };
        return note(format!("[Ảnh không nhúng được: {alt} - {shown}]"));
    };
    let (w, h) = (img.w, img.h);
    let media_name = format!("image{}.{}", ctx.media.len() + 1, img.ext);
    ctx.media.push((media_name.clone(), img.bytes));
    let rid = ctx.add_rel(REL_IMAGE, &format!("media/{media_name}"));
    let (wf, hf) = (w as f64 * 9525.0, h as f64 * 9525.0);
    let scale = (ctx.max_img_emu / wf).min(MAX_IMG_H_EMU / hf).min(1.0);
    let (cx, cy) = (((wf * scale) as i64).max(1), ((hf * scale) as i64).max(1));
    ctx.next_docpr += 1;
    let did = ctx.next_docpr;
    let name = xml_escape(&media_name);
    let descr = xml_escape(alt);
    format!(
        "<w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:effectExtent l=\"0\" t=\"0\" r=\"0\" b=\"0\"/><wp:docPr id=\"{did}\" name=\"{name}\" descr=\"{descr}\"/><wp:cNvGraphicFramePr><a:graphicFrameLocks noChangeAspect=\"1\"/></wp:cNvGraphicFramePr><a:graphic><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic><pic:nvPicPr><pic:cNvPr id=\"{did}\" name=\"{name}\"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r>"
    )
}

fn spans_xml(spans: &[Span], ctx: &mut Ctx, bold_all: bool) -> String {
    let mut out = String::new();
    for s in spans {
        if let Some(display) = s.math {
            let _ = display;
            out.push_str(&math::to_omml(s.text.trim(), false));
            continue;
        }
        if let Some(dest) = &s.image {
            out.push_str(&image_run(&s.text, dest, ctx));
            continue;
        }
        let bold = s.strong || bold_all;
        if let Some(dest) = &s.link {
            if dest.starts_with("http://") || dest.starts_with("https://") || dest.starts_with("mailto:") {
                let rid = ctx.add_rel(REL_HYPERLINK, &encode_uri(dest));
                let rpr = run_props(bold, s.em, s.strike, s.code, Some("0563C1"), true);
                out.push_str(&format!("<w:hyperlink r:id=\"{rid}\" w:history=\"1\">{}</w:hyperlink>", text_runs(&s.text, &rpr)));
                continue;
            }
        }
        out.push_str(&text_runs(&s.text, &run_props(bold, s.em, s.strike, s.code, None, false)));
    }
    out
}

/// Doan chi co 1 cong thuc display -> `<m:oMathPara>` can giua.
fn display_only(spans: &[Span]) -> Option<&Span> {
    match spans {
        [only] if only.math == Some(true) => Some(only),
        _ => None,
    }
}

/// Doan (hoac o bang) chi co dung mot cong thuc (bo qua khoang trang): Word se tu can giua neu de tran.
fn sole_math(spans: &[Span]) -> Option<&Span> {
    let mut it = spans.iter().filter(|s| !(s.math.is_none() && s.image.is_none() && s.text.trim().is_empty()));
    match (it.next(), it.next()) {
        (Some(m), None) if m.math.is_some() => Some(m),
        _ => None,
    }
}

fn para(spans: &[Span], ctx: &mut Ctx, ppr: &str) -> String {
    if let Some(m) = display_only(spans) {
        return format!(
            "<w:p><w:pPr><w:spacing w:before=\"60\" w:after=\"120\"/></w:pPr>{}</w:p>",
            math::to_omml(m.text.trim(), true)
        );
    }
    let ppr = if ppr.is_empty() { String::new() } else { format!("<w:pPr>{ppr}</w:pPr>") };
    if let Some(m) = sole_math(spans) {
        // cong thuc trong dong nhung nam mot minh -> giu can le trai, khong tu nhay ra giua
        return format!("<w:p>{ppr}{}</w:p>", math::to_omml_aligned(m.text.trim(), "left"));
    }
    format!("<w:p>{ppr}{}</w:p>", spans_xml(spans, ctx, false))
}

/// Do dai hien thi uoc tinh cua mot o (de chia be rong cot theo noi dung).
fn cell_len(sp: &[Span]) -> usize {
    let mut n = 0;
    for s in sp {
        n += if s.image.is_some() { 24 } else { s.text.split('\n').map(|l| l.chars().count()).max().unwrap_or(0) };
    }
    n
}

/// Chia `total` twips cho cac cot theo do dai noi dung (4..=44 ky tu), moi cot khong be hon ~1.2 cm.
fn col_widths(head: &[Vec<Span>], rows: &[Vec<Vec<Span>>], cols: usize, total: i64) -> Vec<i64> {
    let weights: Vec<f64> = (0..cols)
        .map(|c| {
            let mut m = head.get(c).map_or(0, |s| cell_len(s));
            for r in rows {
                m = m.max(r.get(c).map_or(0, |s| cell_len(s)));
            }
            m.clamp(9, 40) as f64
        })
        .collect();
    let total_f = total as f64;
    let min_w = (700.0f64).min(total_f / cols as f64);
    let mut fixed = vec![false; cols];
    let mut w = vec![0.0f64; cols];
    for _ in 0..=cols {
        let free_weight: f64 = (0..cols).filter(|&i| !fixed[i]).map(|i| weights[i]).sum();
        let free_total = total_f - (0..cols).filter(|&i| fixed[i]).map(|i| w[i]).sum::<f64>();
        let mut changed = false;
        for i in 0..cols {
            if fixed[i] {
                continue;
            }
            w[i] = free_total * weights[i] / free_weight.max(1.0);
            if w[i] < min_w {
                w[i] = min_w;
                fixed[i] = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    let mut out: Vec<i64> = w.iter().map(|x| x.round() as i64).collect();
    let diff = total - out.iter().sum::<i64>();
    if let Some(m) = out.iter_mut().max() {
        *m += diff;
    }
    out
}

fn blocks_xml(blocks: &[Block], ctx: &mut Ctx, out: &mut String, level: usize, in_cell: bool) {
    for b in blocks {
        match b {
            Block::Paragraph(spans) => out.push_str(&para(spans, ctx, "<w:spacing w:after=\"120\"/>")),
            Block::Heading(lvl, spans, _) => {
                out.push_str(&para(spans, ctx, &format!("<w:pStyle w:val=\"Heading{}\"/>", (*lvl).clamp(1, 6))))
            }
            Block::Code(_, text) => {
                let lines: Vec<&str> = text.trim_end_matches('\n').split('\n').collect();
                let rpr = run_props(false, false, false, true, None, false);
                for (i, line) in lines.iter().enumerate() {
                    // moi dong 1 doan -> nen xam lien mach, khong bi tran
                    let first = i == 0;
                    let last = i + 1 == lines.len();
                    let sp = format!(
                        "<w:spacing w:before=\"{}\" w:after=\"{}\" w:line=\"240\" w:lineRule=\"auto\"/>",
                        if first { 80 } else { 0 },
                        if last { 120 } else { 0 }
                    );
                    let body = if line.is_empty() {
                        String::new()
                    } else {
                        format!("<w:r>{rpr}<w:t xml:space=\"preserve\">{}</w:t></w:r>", xml_escape(&line.replace('\t', "    ")))
                    };
                    out.push_str(&format!(
                        "<w:p><w:pPr><w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"F6F8FA\"/>{sp}<w:ind w:left=\"120\" w:right=\"120\"/></w:pPr>{body}</w:p>"
                    ));
                }
            }
            Block::List { ordered, items } => {
                ctx.nums.push(*ordered);
                let num_id = ctx.nums.len();
                let ilvl = level.min(8);
                for item in items {
                    let mut first = true;
                    for blk in item {
                        if first {
                            first = false;
                            if let Block::Paragraph(spans) = blk {
                                let is_task = spans.first().map_or(false, |x| x.text.starts_with('\u{2611}') || x.text.starts_with('\u{2610}'));
                                let ppr = if is_task {
                                    format!("<w:spacing w:after=\"40\"/><w:ind w:left=\"{}\" w:hanging=\"360\"/>", 720 + 360 * ilvl)
                                } else {
                                    format!("<w:numPr><w:ilvl w:val=\"{ilvl}\"/><w:numId w:val=\"{num_id}\"/></w:numPr><w:spacing w:after=\"40\"/>")
                                };
                                out.push_str(&para(spans, ctx, &ppr));
                                continue;
                            }
                        }
                        match blk {
                            Block::List { .. } => blocks_xml(std::slice::from_ref(blk), ctx, out, level + 1, in_cell),
                            other => blocks_xml(std::slice::from_ref(other), ctx, out, level, in_cell),
                        }
                    }
                }
            }
            Block::Quote(inner) => {
                for blk in inner {
                    if let Block::Paragraph(spans) = blk {
                        // THU TU: pBdr, spacing, ind
                        out.push_str(&para(
                            spans,
                            ctx,
                            "<w:pBdr><w:left w:val=\"single\" w:sz=\"18\" w:space=\"8\" w:color=\"E8930C\"/></w:pBdr><w:spacing w:after=\"120\"/><w:ind w:left=\"360\"/>",
                        ));
                    } else {
                        blocks_xml(std::slice::from_ref(blk), ctx, out, level, in_cell);
                    }
                }
            }
            Block::Table { head, rows, aligns } => {
                let cols = head.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0)).max(1);
                // trong o danh sach/trich dan khong co indent rieng cho bang, nen dung toan bo be rong trang (9638 twips)
                let widths = col_widths(head, rows, cols, 9638);
                let bd = |n: &str| format!("<w:{n} w:val=\"single\" w:sz=\"4\" w:space=\"0\" w:color=\"BFBFBF\"/>");
                let mut tbl = format!(
                    "<w:tbl><w:tblPr><w:tblW w:w=\"{}\" w:type=\"dxa\"/><w:tblBorders>{}{}{}{}{}{}</w:tblBorders><w:tblLayout w:type=\"fixed\"/><w:tblCellMar><w:left w:w=\"100\" w:type=\"dxa\"/><w:right w:w=\"100\" w:type=\"dxa\"/></w:tblCellMar></w:tblPr>",
                    widths.iter().sum::<i64>(), bd("top"), bd("left"), bd("bottom"), bd("right"), bd("insideH"), bd("insideV")
                );
                tbl.push_str("<w:tblGrid>");
                for w in &widths {
                    tbl.push_str(&format!("<w:gridCol w:w=\"{w}\"/>"));
                }
                tbl.push_str("</w:tblGrid>");
                let jc = |c: usize| match aligns.get(c) {
                    Some(Align::Center) => "<w:jc w:val=\"center\"/>",
                    Some(Align::Right) => "<w:jc w:val=\"right\"/>",
                    _ => "",
                };
                let mut emit_row = |cells: &Vec<Vec<Span>>, header: bool, ctx: &mut Ctx, tbl: &mut String| {
                    tbl.push_str("<w:tr><w:trPr><w:cantSplit/>");
                    if header {
                        tbl.push_str("<w:tblHeader/>");
                    }
                    tbl.push_str("</w:trPr>");
                    for c in 0..cols {
                        let cell = cells.get(c);
                        let wcol = widths[c];
                        // anh trong o khong duoc rong hon cot (tru le trai/phai 100 twips moi ben)
                        ctx.max_img_emu = (((wcol - 200).max(400)) as f64 * 635.0).min(PAGE_IMG_EMU);
                        let shd = if header { "<w:shd w:val=\"clear\" w:color=\"auto\" w:fill=\"D9E2F3\"/>" } else { "" };
                        let inner = match cell {
                            Some(sp) if sole_math(sp).is_some() => {
                                let m = sole_math(sp).unwrap();
                                let j = match aligns.get(c) {
                                    Some(Align::Center) => "center",
                                    Some(Align::Right) => "right",
                                    _ => "left",
                                };
                                format!("<w:p><w:pPr><w:spacing w:before=\"20\" w:after=\"20\"/></w:pPr>{}</w:p>", math::to_omml_aligned(m.text.trim(), j))
                            }
                            Some(sp) => format!("<w:p><w:pPr><w:spacing w:before=\"20\" w:after=\"20\"/>{}</w:pPr>{}</w:p>", jc(c), spans_xml(sp, ctx, header)),
                            None => "<w:p/>".to_string(),
                        };
                        // schema tcPr: tcW, ..., shd, ..., vAlign
                        tbl.push_str(&format!("<w:tc><w:tcPr><w:tcW w:w=\"{wcol}\" w:type=\"dxa\"/>{shd}<w:vAlign w:val=\"center\"/></w:tcPr>{inner}</w:tc>"));
                    }
                    ctx.max_img_emu = PAGE_IMG_EMU;
                    tbl.push_str("</w:tr>");
                };
                if !head.is_empty() {
                    emit_row(head, true, ctx, &mut tbl);
                }
                for row in rows {
                    emit_row(row, false, ctx, &mut tbl);
                }
                tbl.push_str("</w:tbl>");
                out.push_str(&tbl);
                out.push_str("<w:p><w:pPr><w:spacing w:after=\"60\"/></w:pPr></w:p>");
            }
            Block::Rule => {
                out.push_str("<w:p><w:pPr><w:pBdr><w:bottom w:val=\"single\" w:sz=\"6\" w:space=\"1\" w:color=\"BFBFBF\"/></w:pBdr></w:pPr></w:p>");
            }
            Block::Html(text) => {
                // <img> -> anh that, <br> -> xuong dong, cac the khac bo di (chi giu chu)
                let spans = media::html_to_spans(text);
                if !spans.is_empty() {
                    out.push_str(&para(&spans, ctx, "<w:spacing w:after=\"120\"/>"));
                }
            }
        }
    }
}

fn styles_xml() -> String {
    let mut s = format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:styles xmlns:w=\"{NS_W}\"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\" w:eastAsia=\"Calibri\" w:cs=\"Calibri\"/><w:sz w:val=\"22\"/><w:szCs w:val=\"22\"/><w:lang w:val=\"vi-VN\" w:eastAsia=\"en-US\" w:bidi=\"ar-SA\"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after=\"120\" w:line=\"264\" w:lineRule=\"auto\"/></w:pPr></w:pPrDefault></w:docDefaults>");
    s.push_str("<w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/><w:qFormat/></w:style>");
    s.push_str("<w:style w:type=\"character\" w:default=\"1\" w:styleId=\"DefaultParagraphFont\"><w:name w:val=\"Default Paragraph Font\"/><w:uiPriority w:val=\"1\"/><w:semiHidden/><w:unhideWhenUsed/></w:style>");
    s.push_str("<w:style w:type=\"table\" w:default=\"1\" w:styleId=\"TableNormal\"><w:name w:val=\"Normal Table\"/><w:uiPriority w:val=\"99\"/><w:semiHidden/><w:unhideWhenUsed/><w:tblPr><w:tblInd w:w=\"0\" w:type=\"dxa\"/><w:tblCellMar><w:top w:w=\"0\" w:type=\"dxa\"/><w:left w:w=\"108\" w:type=\"dxa\"/><w:bottom w:w=\"0\" w:type=\"dxa\"/><w:right w:w=\"108\" w:type=\"dxa\"/></w:tblCellMar></w:tblPr></w:style>");
    s.push_str("<w:style w:type=\"numbering\" w:default=\"1\" w:styleId=\"NoList\"><w:name w:val=\"No List\"/><w:uiPriority w:val=\"99\"/><w:semiHidden/><w:unhideWhenUsed/></w:style>");
    let heads = [(1, 40, "154273"), (2, 32, "154273"), (3, 28, "24292F"), (4, 24, "24292F"), (5, 22, "24292F"), (6, 22, "24292F")];
    for (lvl, sz, color) in heads {
        s.push_str(&format!(
            "<w:style w:type=\"paragraph\" w:styleId=\"Heading{lvl}\"><w:name w:val=\"heading {lvl}\"/><w:basedOn w:val=\"Normal\"/><w:next w:val=\"Normal\"/><w:uiPriority w:val=\"9\"/><w:qFormat/><w:pPr><w:keepNext/><w:keepLines/><w:spacing w:before=\"{}\" w:after=\"120\"/><w:outlineLvl w:val=\"{}\"/></w:pPr><w:rPr><w:b/><w:bCs/><w:color w:val=\"{color}\"/><w:sz w:val=\"{sz}\"/><w:szCs w:val=\"{sz}\"/></w:rPr></w:style>",
            if lvl == 1 { 480 } else { 320 },
            lvl - 1
        ));
    }
    s.push_str("</w:styles>");
    s
}

fn numbering_xml(nums: &[bool]) -> String {
    let mut s = format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:numbering xmlns:w=\"{NS_W}\">");
    let bullets = ["\u{2022}", "o", "\u{25AA}"];
    // abstractNum 0 = gach dau dong, 1 = danh so; 9 cap moi loai
    for (id, ordered) in [(0, false), (1, true)] {
        s.push_str(&format!("<w:abstractNum w:abstractNumId=\"{id}\"><w:multiLevelType w:val=\"hybridMultilevel\"/>"));
        for lvl in 0..9 {
            let left = 720 + 360 * lvl;
            if ordered {
                let (fmt, text) = match lvl % 3 {
                    0 => ("decimal", format!("%{}.", lvl + 1)),
                    1 => ("lowerLetter", format!("%{}.", lvl + 1)),
                    _ => ("lowerRoman", format!("%{}.", lvl + 1)),
                };
                s.push_str(&format!(
                    "<w:lvl w:ilvl=\"{lvl}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"{fmt}\"/><w:lvlText w:val=\"{text}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{left}\" w:hanging=\"360\"/></w:pPr></w:lvl>"
                ));
            } else {
                let ch = bullets[lvl % 3];
                let font = if lvl % 3 == 1 { "Courier New" } else { "Arial" };
                s.push_str(&format!(
                    "<w:lvl w:ilvl=\"{lvl}\"><w:start w:val=\"1\"/><w:numFmt w:val=\"bullet\"/><w:lvlText w:val=\"{ch}\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"{left}\" w:hanging=\"360\"/></w:pPr><w:rPr><w:rFonts w:ascii=\"{font}\" w:hAnsi=\"{font}\" w:cs=\"{font}\" w:hint=\"default\"/></w:rPr></w:lvl>"
                ));
            }
        }
        s.push_str("</w:abstractNum>");
    }
    // numId luon >= 1; neu khong co list nao van hop le (numbering khong can w:num)
    for (i, ordered) in nums.iter().enumerate() {
        s.push_str(&format!(
            "<w:num w:numId=\"{}\"><w:abstractNumId w:val=\"{}\"/></w:num>",
            i + 1,
            if *ordered { 1 } else { 0 }
        ));
    }
    s.push_str("</w:numbering>");
    s
}

fn iso_now() -> String {
    // UTC, khong can chrono
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    // civil from days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, (rem % 3600) / 60, rem % 60)
}

/// Dong goi DOCX tu AST. `title` dua vao docProps.
pub fn build(blocks: &[Block], doc_dir: &Path, title: &str, version: &str) -> Option<Vec<u8>> {
    // anh http(s): tai truoc (song song, toi da 12 giay moi anh) de nhung that vao Word
    let mut urls = Vec::new();
    media::collect_remote_images(blocks, &mut urls);
    let remote = if urls.is_empty() { HashMap::new() } else { media::fetch_many(&urls) };
    let mut ctx = Ctx { rels: Vec::new(), media: Vec::new(), nums: Vec::new(), next_rel: 0, next_docpr: 0, doc_dir: doc_dir.to_path_buf(), remote, max_img_emu: PAGE_IMG_EMU };
    let mut body = String::new();
    blocks_xml(blocks, &mut ctx, &mut body, 0, false);
    if body.is_empty() {
        body.push_str("<w:p/>");
    }
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:document xmlns:w=\"{NS_W}\" xmlns:r=\"{NS_R}\" xmlns:m=\"{NS_M}\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\" xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><w:body>{body}<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/><w:pgMar w:top=\"1134\" w:right=\"1134\" w:bottom=\"1134\" w:left=\"1134\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>"
    );
    let numbering = numbering_xml(&ctx.nums);
    let settings = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:settings xmlns:w=\"{NS_W}\" xmlns:m=\"{NS_M}\"><w:zoom w:percent=\"100\"/><w:defaultTabStop w:val=\"720\"/><w:characterSpacingControl w:val=\"doNotCompress\"/><w:compat><w:compatSetting w:name=\"compatibilityMode\" w:uri=\"http://schemas.microsoft.com/office/word\" w:val=\"15\"/></w:compat><m:mathPr><m:mathFont m:val=\"Cambria Math\"/><m:brkBin m:val=\"before\"/><m:brkBinSub m:val=\"--\"/><m:smallFrac m:val=\"0\"/><m:dispDef/><m:lMargin m:val=\"0\"/><m:rMargin m:val=\"0\"/><m:defJc m:val=\"centerGroup\"/><m:wrapIndent m:val=\"1440\"/><m:intLim m:val=\"subSup\"/><m:naryLim m:val=\"undOvr\"/></m:mathPr><w:themeFontLang w:val=\"vi-VN\"/></w:settings>"
    );
    let mut rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"{NS_R}/styles\" Target=\"styles.xml\"/><Relationship Id=\"rId2\" Type=\"{NS_R}/numbering\" Target=\"numbering.xml\"/><Relationship Id=\"rId3\" Type=\"{NS_R}/settings\" Target=\"settings.xml\"/>"
    );
    for (id, ty, target) in &ctx.rels {
        rels.push_str(&format!(
            "<Relationship Id=\"{id}\" Type=\"{ty}\" Target=\"{}\"{}/>",
            xml_escape(target),
            if *ty == REL_HYPERLINK { " TargetMode=\"External\"" } else { "" }
        ));
    }
    rels.push_str("</Relationships>");
    let root_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/><Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/><Relationship Id=\"rId3\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/extended-properties\" Target=\"docProps/app.xml\"/></Relationships>";
    let content_types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Default Extension=\"jpg\" ContentType=\"image/jpeg\"/><Default Extension=\"gif\" ContentType=\"image/gif\"/><Default Extension=\"bmp\" ContentType=\"image/bmp\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/><Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/><Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/><Override PartName=\"/word/settings.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.settings+xml\"/><Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/><Override PartName=\"/docProps/app.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.extended-properties+xml\"/></Types>";
    let now = iso_now();
    let core = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" xmlns:dcmitype=\"http://purl.org/dc/dcmitype/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"><dc:title>{}</dc:title><dc:creator>Markdown Tabs</dc:creator><cp:lastModifiedBy>Markdown Tabs</cp:lastModifiedBy><dcterms:created xsi:type=\"dcterms:W3CDTF\">{now}</dcterms:created><dcterms:modified xsi:type=\"dcterms:W3CDTF\">{now}</dcterms:modified></cp:coreProperties>",
        xml_escape(title)
    );
    let app = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Properties xmlns=\"http://schemas.openxmlformats.org/officeDocument/2006/extended-properties\"><Application>Markdown Tabs</Application><AppVersion>{}</AppVersion></Properties>",
        xml_escape(version.split(|c: char| !(c.is_ascii_digit() || c == '.')).next().unwrap_or("1.0"))
    );
    // [Content_Types].xml phai dung dau tien trong zip
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    let mut zw = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let mut put = |zw: &mut zip::ZipWriter<std::io::Cursor<Vec<u8>>>, name: &str, data: &[u8]| -> Option<()> {
        zw.start_file(name, opts).ok()?;
        zw.write_all(data).ok()?;
        Some(())
    };
    put(&mut zw, "[Content_Types].xml", content_types.as_bytes())?;
    put(&mut zw, "_rels/.rels", root_rels.as_bytes())?;
    put(&mut zw, "docProps/core.xml", core.as_bytes())?;
    put(&mut zw, "docProps/app.xml", app.as_bytes())?;
    put(&mut zw, "word/document.xml", document.as_bytes())?;
    put(&mut zw, "word/styles.xml", styles_xml().as_bytes())?;
    put(&mut zw, "word/numbering.xml", numbering.as_bytes())?;
    put(&mut zw, "word/settings.xml", settings.as_bytes())?;
    put(&mut zw, "word/_rels/document.xml.rels", rels.as_bytes())?;
    for (name, data) in &ctx.media {
        put(&mut zw, &format!("word/media/{name}"), data)?;
    }
    Some(zw.finish().ok()?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_strips_invalid() {
        assert_eq!(xml_escape("a\u{0}b\u{1}c\u{B}d&<"), "abcd&amp;&lt;");
    }

    #[test]
    fn uri_encoding() {
        assert_eq!(encode_uri("https://x.vn/tiếng việt?a=1&b=2"), "https://x.vn/ti%E1%BA%BFng%20vi%E1%BB%87t?a=1&b=2");
        assert_eq!(encode_uri("https://a/%20b"), "https://a/%20b");
    }

    #[test]
    fn build_sample_to_tmp() {
        let md = std::fs::read_to_string(std::env::var("MDT_SAMPLE").unwrap_or_else(|_| "../Test-Rust_v0.4.md".into())).unwrap_or_else(|_| "# T\n\n- a\n  - b\n\n$$x^2$$".into());
        let blocks = crate::parse_markdown(&md);
        let dir = std::env::var("MDT_SAMPLE").ok().and_then(|p| Path::new(&p).parent().map(|d| d.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
        let bytes = build(&blocks, &dir, "Test", "0.5.0").unwrap();
        let out = std::env::var("MDT_OUT").unwrap_or_else(|_| std::env::temp_dir().join("mdt-test-sample.docx").to_string_lossy().into_owned());
        std::fs::write(out, bytes).unwrap();
    }
}
