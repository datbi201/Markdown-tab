//! Sua loi trinh bay / font chu / cong thuc trong van ban Markdown.
//!
//! Cac nhom loi thuong gap khi dan tu Word, MathType, PDF, web:
//!  * chu tieng Viet bi vo (UTF-8 doc thanh Windows-1252: "Viá»‡t"), font cu TCVN3 / VNI
//!  * ky tu font Symbol cua MathType (vung rieng U+F0xx) hien thanh o vuong / ky tu la
//!  * dau tieng Viet dang to hop (NFD), khoang trang khong ngat, ky tu vo hinh
//!  * danh sach "•", tieu de thieu dau cach, bang thieu dong phan cach
//!  * cong thuc: \( \) \[ \], khoang trang thua trong $ x $, dong LaTeX tran khong co $, $ tien te
//!    bi hieu nham la cong thuc, lenh LaTeX sai / ngoac khong can bang

use crate::math;
use crate::vn_tables::VN_TABLE;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Legacy {
    None,
    Tcvn3,
    Vni,
}

#[derive(Clone, Debug)]
pub struct Opts {
    pub mojibake: bool,
    pub legacy: Legacy,
    pub nfc: bool,
    pub symbol_font: bool,
    pub invisible: bool,
    pub spaces: bool,
    pub trim: bool,
    pub punct_space: bool,
    pub quotes: bool,
    pub bullets: bool,
    pub heading_space: bool,
    pub table_fix: bool,
    pub math_delims: bool,
    pub math_trim: bool,
    pub bare_latex: bool,
    pub currency: bool,
    pub unicode_scripts: bool,
}

impl Opts {
    /// Bo loi an toan, nen bat san.
    pub fn recommended() -> Self {
        Opts {
            mojibake: true,
            legacy: Legacy::None,
            nfc: true,
            symbol_font: true,
            invisible: true,
            spaces: true,
            trim: true,
            punct_space: false,
            quotes: false,
            bullets: true,
            heading_space: true,
            table_fix: true,
            math_delims: true,
            math_trim: true,
            bare_latex: true,
            currency: true,
            unicode_scripts: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Change {
    pub label: &'static str,
    pub count: usize,
}

#[derive(Clone, Debug)]
pub struct Issue {
    pub line: usize, // 1-based
    pub snippet: String,
    pub msg: String,
}

// -------------------------------------------------------------- bang ma
fn symbol_map(c: char) -> Option<char> {
    // MathType / Word dung font Symbol: ky tu 0xF0xx = 0xxx cua font Symbol
    let b = (c as u32).checked_sub(0xF000)?;
    if !(0x20..=0xFF).contains(&b) {
        return None;
    }
    Some(match b as u8 {
        0x20 => ' ',
        0x22 => '∀',
        0x24 => '∃',
        0x27 => '∋',
        0x2A => '∗',
        0x2D => '−',
        0x40 => '≅',
        0x41 => 'Α', 0x42 => 'Β', 0x43 => 'Χ', 0x44 => 'Δ', 0x45 => 'Ε', 0x46 => 'Φ', 0x47 => 'Γ',
        0x48 => 'Η', 0x49 => 'Ι', 0x4A => 'ϑ', 0x4B => 'Κ', 0x4C => 'Λ', 0x4D => 'Μ', 0x4E => 'Ν',
        0x4F => 'Ο', 0x50 => 'Π', 0x51 => 'Θ', 0x52 => 'Ρ', 0x53 => 'Σ', 0x54 => 'Τ', 0x55 => 'Υ',
        0x56 => 'ς', 0x57 => 'Ω', 0x58 => 'Ξ', 0x59 => 'Ψ', 0x5A => 'Ζ',
        0x5C => '∴',
        0x5E => '⊥',
        0x61 => 'α', 0x62 => 'β', 0x63 => 'χ', 0x64 => 'δ', 0x65 => 'ε', 0x66 => 'φ', 0x67 => 'γ',
        0x68 => 'η', 0x69 => 'ι', 0x6A => 'ϕ', 0x6B => 'κ', 0x6C => 'λ', 0x6D => 'μ', 0x6E => 'ν',
        0x6F => 'ο', 0x70 => 'π', 0x71 => 'θ', 0x72 => 'ρ', 0x73 => 'σ', 0x74 => 'τ', 0x75 => 'υ',
        0x76 => 'ϖ', 0x77 => 'ω', 0x78 => 'ξ', 0x79 => 'ψ', 0x7A => 'ζ',
        0xA2 => '′',
        0xA3 => '≤',
        0xA5 => '∞',
        0xAB => '↔',
        0xAC => '←',
        0xAE => '→',
        0xB0 => '°',
        0xB1 => '±',
        0xB2 => '″',
        0xB3 => '≥',
        0xB4 => '×',
        0xB5 => '∝',
        0xB6 => '∂',
        0xB7 => '•',
        0xB8 => '÷',
        0xB9 => '≠',
        0xBA => '≡',
        0xBB => '≈',
        0xBC => '…',
        0xC4 => '⊗',
        0xC5 => '⊕',
        0xC6 => '∅',
        0xC7 => '∩',
        0xC8 => '∪',
        0xC9 => '⊃',
        0xCA => '⊇',
        0xCC => '⊂',
        0xCD => '⊆',
        0xCE => '∈',
        0xCF => '∉',
        0xD0 => '∠',
        0xD1 => '∇',
        0xD5 => '∏',
        0xD6 => '√',
        0xD7 => '⋅',
        0xD8 => '¬',
        0xD9 => '∧',
        0xDA => '∨',
        0xDB => '⇔',
        0xDC => '⇐',
        0xDE => '⇒',
        0xE5 => '∑',
        0xF2 => '∫',
        _ => return None,
    })
}

/// Mot "tu" (khong chua khoang trang) bi vo ma: UTF-8 bi doc nhu Windows-1252.
fn demojibake_token(tok: &str) -> Option<String> {
    if tok.is_ascii() {
        return None;
    }
    // dau hieu: ky tu dan dau (Ã Â Ä Å á â ã ...) di kem ky tu tiep noi
    if !tok.chars().any(|c| matches!(c, 'Ã' | 'Â' | 'Ä' | 'Å' | 'Æ' | 'á' | 'â' | 'ã' | 'à' | 'ä' | 'å')) {
        return None;
    }
    let (bytes, _, unmappable) = encoding_rs::WINDOWS_1252.encode(tok);
    if unmappable {
        return None;
    }
    let fixed = String::from_utf8(bytes.into_owned()).ok()?;
    if fixed == tok || fixed.is_empty() {
        return None;
    }
    // chi nhan neu ket qua it ky tu "la" hon
    let weird = |s: &str| s.chars().filter(|c| matches!(c, 'Ã' | 'Â' | 'Ä' | 'Å' | 'Æ' | 'Ç') || ('\u{80}'..='\u{BF}').contains(c) || matches!(c, '»' | '‡' | '€' | '‚' | '„' | '…' | '†' | 'ˆ' | '‰' | 'Š' | '‹' | 'Œ' | 'Ž' | '‘' | '’' | '“' | '”' | '•' | '–' | '—' | '˜' | '™' | 'š' | '›' | 'œ' | 'ž' | 'Ÿ')).count();
    if weird(&fixed) < weird(tok) {
        Some(fixed)
    } else {
        None
    }
}

/// Sua cac "tu" bi vo ma trong mot dong ("Viá»‡t" -> "Việt"), giu nguyen phan con lai.
fn demojibake_line(line: &str) -> Option<String> {
    if line.is_ascii() {
        return None;
    }
    let mut out = String::with_capacity(line.len());
    let mut changed = false;
    let mut tok = String::new();
    let flush = |tok: &mut String, out: &mut String, changed: &mut bool| {
        if tok.is_empty() {
            return;
        }
        match demojibake_token(tok) {
            Some(f) => {
                out.push_str(&f);
                *changed = true;
            }
            None => out.push_str(tok),
        }
        tok.clear();
    };
    for c in line.chars() {
        if c.is_whitespace() {
            flush(&mut tok, &mut out, &mut changed);
            out.push(c);
        } else {
            tok.push(c);
        }
    }
    flush(&mut tok, &mut out, &mut changed);
    if changed {
        Some(out)
    } else {
        None
    }
}

/// Chuyen tieng Viet font cu sang Unicode (greedy: cap 2 ky tu truoc, roi 1 ky tu).
pub fn legacy_to_unicode(text: &str, which: Legacy) -> (String, usize) {
    use std::collections::HashMap;
    let mut map: HashMap<&str, char> = HashMap::new();
    for (u, vni, tcvn) in VN_TABLE {
        let key = if which == Legacy::Vni { *vni } else { *tcvn };
        map.insert(key, *u);
    }
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let mut n = 0;
    let mut buf = String::new();
    while i < chars.len() {
        if i + 1 < chars.len() {
            buf.clear();
            buf.push(chars[i]);
            buf.push(chars[i + 1]);
            if let Some(u) = map.get(buf.as_str()) {
                out.push(*u);
                i += 2;
                n += 1;
                continue;
            }
        }
        buf.clear();
        buf.push(chars[i]);
        if !chars[i].is_ascii() {
            if let Some(u) = map.get(buf.as_str()) {
                out.push(*u);
                i += 1;
                n += 1;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    (out, n)
}

/// Uoc luong van ban co phai tieng Viet font cu khong: (diem_tcvn3, diem_vni, so_ky_tu_viet_unicode)
pub fn detect_legacy(text: &str) -> (usize, usize, usize) {
    let mut tcvn = 0;
    let mut vni = 0;
    let mut uni = 0;
    let tcvn_set: Vec<char> = VN_TABLE.iter().filter(|(_, _, t)| t.chars().count() == 1).filter_map(|(_, _, t)| t.chars().next()).filter(|c| !c.is_ascii()).collect();
    let vni_pairs: Vec<&str> = VN_TABLE.iter().map(|(_, v, _)| *v).filter(|v| v.chars().count() == 2).collect();
    let vi: Vec<char> = VN_TABLE.iter().map(|(u, _, _)| *u).collect();
    let cs: Vec<char> = text.chars().collect();
    for (i, c) in cs.iter().enumerate() {
        if vi.contains(c) {
            uni += 1;
        }
        if tcvn_set.contains(c) {
            tcvn += 1;
        }
        if i + 1 < cs.len() {
            let pair: String = [*c, cs[i + 1]].iter().collect();
            if vni_pairs.iter().any(|p| *p == pair) {
                vni += 1;
            }
        }
    }
    (tcvn, vni, uni)
}

// ------------------------------------------------------ tien ich theo dong
/// Chia dong thanh cac doan (noi_dung, duoc_bao_ve): `code`, $cong thuc$.
fn split_protected(line: &str) -> Vec<(String, bool)> {
    let cs: Vec<char> = line.chars().collect();
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut cur = String::new();
    let mut i = 0;
    let flush = |cur: &mut String, out: &mut Vec<(String, bool)>| {
        if !cur.is_empty() {
            out.push((std::mem::take(cur), false));
        }
    };
    while i < cs.len() {
        let c = cs[i];
        if c == '\\' && i + 1 < cs.len() {
            cur.push(c);
            cur.push(cs[i + 1]);
            i += 2;
            continue;
        }
        if c == '`' {
            let mut n = 0;
            while i + n < cs.len() && cs[i + n] == '`' {
                n += 1;
            }
            // tim chuoi dong tuong ung
            let mut j = i + n;
            let mut found = None;
            while j + n <= cs.len() {
                if cs[j..j + n].iter().all(|x| *x == '`') && (j + n == cs.len() || cs[j + n] != '`') {
                    found = Some(j + n);
                    break;
                }
                j += 1;
            }
            if let Some(end) = found {
                flush(&mut cur, &mut out);
                out.push((cs[i..end].iter().collect(), true));
                i = end;
                continue;
            }
            cur.extend(std::iter::repeat('`').take(n));
            i += n;
            continue;
        }
        if c == '$' {
            let dbl = i + 1 < cs.len() && cs[i + 1] == '$';
            let n = if dbl { 2 } else { 1 };
            let open_space = cs.get(i + n).map_or(true, |x| x.is_whitespace());
            let mut j = i + n;
            let mut found = None;
            while j < cs.len() {
                if cs[j] == '\\' {
                    j += 2;
                    continue;
                }
                if cs[j] == '$' && (!dbl || (j + 1 < cs.len() && cs[j + 1] == '$')) {
                    let before_space = cs[j - 1].is_whitespace();
                    let after_digit = cs.get(j + n).map_or(false, |x| x.is_ascii_digit());
                    // dang chuan: $x$ (khong khoang trang sat dau $, khong ngay truoc chu so)
                    // dang them khoang trang: $ x $ (ca hai phia deu co khoang trang)
                    let ok = if open_space { before_space } else { !before_space && (dbl || !after_digit) };
                    if ok {
                        found = Some(j + n);
                        break;
                    }
                    if !open_space && !dbl {
                        // gap $ khong hop le lam dau dong -> bo qua, thu tiep
                    }
                }
                j += 1;
            }
            if let Some(end) = found {
                if end > i + 2 * n {
                    flush(&mut cur, &mut out);
                    out.push((cs[i..end].iter().collect(), true));
                    i = end;
                    continue;
                }
            }
        }
        cur.push(c);
        i += 1;
    }
    flush(&mut cur, &mut out);
    out
}

fn is_fence(line: &str) -> Option<char> {
    let t = line.trim_start();
    if t.starts_with("```") {
        Some('`')
    } else if t.starts_with("~~~") {
        Some('~')
    } else {
        None
    }
}

fn is_table_line(l: &str) -> bool {
    let t = l.trim();
    t.starts_with('|') && t.matches('|').count() >= 2
}

fn is_table_sep(l: &str) -> bool {
    let t = l.trim();
    t.contains('-') && t.chars().all(|c| matches!(c, '|' | '-' | ':' | ' ' | '\t')) && t.contains('|')
}

fn table_cols(l: &str) -> usize {
    let t = l.trim().trim_matches('|');
    t.split('|').count()
}

const LATEX_HINTS: &[&str] = &[
    "\\frac", "\\sqrt", "\\sum", "\\int", "\\prod", "\\lim", "\\alpha", "\\beta", "\\gamma", "\\delta", "\\theta", "\\lambda",
    "\\mu", "\\pi", "\\sigma", "\\omega", "\\Delta", "\\partial", "\\infty", "\\cdot", "\\times", "\\leq", "\\geq", "\\le ",
    "\\ge ", "\\neq", "\\approx", "\\left", "\\right", "\\begin{", "\\vec", "\\hat", "\\bar", "\\mathrm", "\\text{", "\\rightarrow",
    "\\to ", "\\pm", "\\log", "\\ln", "\\sin", "\\cos", "\\tan", "\\nabla", "\\rho", "\\varepsilon", "\\epsilon", "\\tau", "\\phi",
];

fn looks_like_bare_latex(l: &str) -> bool {
    let t = l.trim();
    if t.is_empty() || t.contains('$') || t.starts_with('#') || t.starts_with('>') || t.starts_with('|') || t.starts_with("- ") || t.starts_with("* ") {
        return false;
    }
    if !LATEX_HINTS.iter().any(|h| t.contains(h)) {
        return false;
    }
    // so "tu van" (>= 4 chu cai lien tiep, khong thuoc lenh/ten ham) phai it
    let mut words = 0;
    let cs: Vec<char> = t.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == '\\' {
            i += 1;
            while i < cs.len() && cs[i].is_ascii_alphabetic() {
                i += 1;
            }
            continue;
        }
        if cs[i] == '{' {
            // noi dung \text{...} / \mathrm{...} khong tinh
        }
        if cs[i].is_alphabetic() {
            let st = i;
            while i < cs.len() && cs[i].is_alphabetic() {
                i += 1;
            }
            if i - st >= 4 {
                words += 1;
            }
            continue;
        }
        i += 1;
    }
    words <= 1
}

fn collapse_ws_ascii(s: &str) -> String {
    s.to_string()
}

// ------------------------------------------------------------------- fix chinh
pub fn fix(text: &str, o: &Opts) -> (String, Vec<Change>) {
    let mut ch: Vec<Change> = Vec::new();
    let mut add = |label: &'static str, count: usize, ch: &mut Vec<Change>| {
        if count > 0 {
            ch.push(Change { label, count });
        }
    };
    let mut t = text.to_string();

    // ---- giai doan A: toan van ban
    if o.spaces {
        let n = t.matches('\r').count();
        if n > 0 {
            t = t.replace("\r\n", "\n").replace('\r', "\n");
            add("Chuẩn hoá xuống dòng (CRLF → LF)", n, &mut ch);
        }
        let n = t.matches('\u{2028}').count() + t.matches('\u{2029}').count();
        if n > 0 {
            t = t.replace('\u{2028}', "\n").replace('\u{2029}', "\n\n");
            add("Dấu ngắt dòng/đoạn Unicode", n, &mut ch);
        }
    }
    if o.mojibake {
        let mut n = 0;
        let mut lines: Vec<String> = Vec::new();
        for l in t.split('\n') {
            match demojibake_line(l) {
                Some(f) => {
                    n += 1;
                    lines.push(f)
                }
                None => lines.push(l.to_string()),
            }
        }
        if n > 0 {
            t = lines.join("\n");
            add("Sửa chữ bị vỡ mã (UTF-8 đọc nhầm Windows-1252)", n, &mut ch);
        }
    }
    if o.legacy != Legacy::None {
        let (r, n) = legacy_to_unicode(&t, o.legacy);
        if n > 0 {
            t = r;
            add(
                if o.legacy == Legacy::Tcvn3 { "Chuyển font cũ TCVN3 (.VnTime) → Unicode" } else { "Chuyển font cũ VNI → Unicode" },
                n,
                &mut ch,
            );
        }
    }
    if o.symbol_font {
        let mut n = 0;
        let r: String = t
            .chars()
            .map(|c| match symbol_map(c) {
                Some(m) => {
                    n += 1;
                    m
                }
                None => c,
            })
            .collect();
        if n > 0 {
            t = r;
            add("Ký tự font Symbol/MathType (vùng riêng U+F0xx) → Unicode", n, &mut ch);
        }
    }
    if o.nfc {
        let r: String = t.nfc().collect();
        if r != t {
            let n = t.chars().count().saturating_sub(r.chars().count());
            t = r;
            add("Gộp dấu tiếng Việt dạng tổ hợp (NFC)", n.max(1), &mut ch);
        }
    }
    if o.invisible {
        let mut n = 0;
        let r: String = t
            .chars()
            .filter(|c| {
                let bad = matches!(*c, '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{200E}' | '\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{FFFC}' | '\u{FFFD}')
                    || (((*c as u32) < 0x20) && !matches!(*c, '\n' | '\t'));
                if bad {
                    n += 1;
                }
                !bad
            })
            .collect();
        if n > 0 {
            t = r;
            add("Ký tự vô hình / điều khiển", n, &mut ch);
        }
    }
    if o.spaces {
        let mut n = 0;
        let r: String = t
            .chars()
            .map(|c| {
                if matches!(c, '\u{00A0}' | '\u{1680}' | '\u{2000}'..='\u{200A}' | '\u{202F}' | '\u{205F}' | '\u{3000}') {
                    n += 1;
                    ' '
                } else {
                    c
                }
            })
            .collect();
        if n > 0 {
            t = r;
            add("Khoảng trắng không ngắt / đặc biệt → dấu cách", n, &mut ch);
        }
    }

    // ---- giai doan B: theo dong
    let src_lines: Vec<&str> = t.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(src_lines.len());
    let mut fence: Option<char> = None;
    let mut in_disp = false; // trong khoi \[ ... \] da doi thanh $$
    let (mut c_tab, mut c_trim, mut c_head, mut c_bullet, mut c_punct, mut c_quote, mut c_delim, mut c_mtrim, mut c_bare, mut c_cur, mut c_script, mut c_tbl, mut c_blank) =
        (0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    let mut blank_run = 0usize;
    for (idx, raw) in src_lines.iter().enumerate() {
        let mut line = raw.to_string();
        if let Some(f) = is_fence(&line) {
            match fence {
                None => fence = Some(f),
                Some(g) if g == f => fence = None,
                _ => {}
            }
            // dong rao: chi cat khoang trang cuoi
            out.push(line.trim_end().to_string());
            blank_run = 0;
            continue;
        }
        if fence.is_some() {
            out.push(line);
            blank_run = 0;
            continue;
        }
        // tab
        if o.spaces && line.contains('\t') {
            let n = line.matches('\t').count();
            line = line.replace('\t', "    ");
            c_tab += n;
        }
        // khoi \[ ... \]
        if o.math_delims {
            let tr_owned = line.trim().to_string();
            let tr: &str = &tr_owned;
            if tr.starts_with("\\[") && tr.ends_with("\\]") && tr.len() >= 4 {
                let inner = &tr[2..tr.len() - 2];
                line = format!("$${}$$", inner.trim());
                c_delim += 1;
            } else if tr == "\\[" {
                line = "$$".into();
                in_disp = true;
                c_delim += 1;
            } else if tr == "\\]" && in_disp {
                line = "$$".into();
                in_disp = false;
                c_delim += 1;
            } else if tr.starts_with("\\[") {
                line = format!("$${}", tr[2..].trim_start());
                in_disp = !tr.contains("\\]");
                c_delim += 1;
                if tr.contains("\\]") {
                    line = line.replace("\\]", "$$");
                }
            } else if in_disp && tr.ends_with("\\]") {
                line = format!("{}$$", tr[..tr.len() - 2].trim_end());
                in_disp = false;
                c_delim += 1;
            }
        }
        // trong khoi $$ nhieu dong: khong ap dung sua van ban thuong
        let tr_now = line.trim().to_string();
        if in_disp || (tr_now == "$$") {
            out.push(line.trim_end().to_string());
            blank_run = 0;
            continue;
        }

        // cong thuc dong tran
        if o.bare_latex && looks_like_bare_latex(&line) {
            let inner = line.trim();
            line = format!("$${}$$", inner);
            c_bare += 1;
        }

        // xu ly theo doan, bao ve `code` va $cong thuc$
        let segs = split_protected(&line);
        let mut rebuilt = String::new();
        for (s, protected) in segs {
            if protected {
                if s.starts_with('$') {
                    let dbl = s.starts_with("$$");
                    let n = if dbl { 2 } else { 1 };
                    let inner = &s[n..s.len() - n];
                    let mut new_inner = inner.to_string();
                    if o.math_trim && inner != inner.trim() {
                        new_inner = inner.trim().to_string();
                        c_mtrim += 1;
                    }
                    if o.unicode_scripts {
                        // khong dung trong cong thuc
                    }
                    let d = "$".repeat(n);
                    rebuilt.push_str(&format!("{d}{new_inner}{d}"));
                } else {
                    rebuilt.push_str(&s);
                }
                continue;
            }
            let mut seg = s;
            if o.math_delims && (seg.contains("\\(") || seg.contains("\\)")) {
                let n = seg.matches("\\(").count();
                seg = seg.replace("\\(", "$").replace("\\)", "$");
                c_delim += n;
                // moi cong thuc moi co the can trim
                if o.math_trim {
                    let re = split_protected(&seg);
                    seg = re
                        .into_iter()
                        .map(|(x, p)| if p && x.starts_with('$') && !x.starts_with("$$") { format!("${}$", x[1..x.len() - 1].trim()) } else { x })
                        .collect();
                }
            }
            if o.currency {
                // $5, $1,200.50 ... khong co $ dong -> thoat de khong bi hieu la cong thuc
                let cs: Vec<char> = seg.chars().collect();
                let mut r = String::new();
                let mut i = 0;
                while i < cs.len() {
                    if cs[i] == '$' && i + 1 < cs.len() && cs[i + 1].is_ascii_digit() && (i == 0 || !cs[i - 1].is_alphanumeric() && cs[i - 1] != '\\') {
                        let mut j = i + 1;
                        while j < cs.len() && (cs[j].is_ascii_digit() || ((cs[j] == '.' || cs[j] == ',') && j + 1 < cs.len() && cs[j + 1].is_ascii_digit())) {
                            j += 1;
                        }
                        let after = cs.get(j).copied();
                        let ok = match after {
                            None => true,
                            Some(a) => a.is_whitespace() || matches!(a, '.' | ',' | ';' | ':' | ')' | '!' | '?' | '/' | '-'),
                        };
                        if ok {
                            r.push_str("\\$");
                            c_cur += 1;
                            i += 1;
                            continue;
                        }
                    }
                    r.push(cs[i]);
                    i += 1;
                }
                seg = r;
            }
            if o.unicode_scripts {
                let cs: Vec<char> = seg.chars().collect();
                let sup = |c: char| match c {
                    '⁰' => Some('0'), '¹' => Some('1'), '²' => Some('2'), '³' => Some('3'), '⁴' => Some('4'),
                    '⁵' => Some('5'), '⁶' => Some('6'), '⁷' => Some('7'), '⁸' => Some('8'), '⁹' => Some('9'),
                    '⁺' => Some('+'), '⁻' => Some('-'), 'ⁿ' => Some('n'), _ => None,
                };
                let sub = |c: char| match c {
                    '₀' => Some('0'), '₁' => Some('1'), '₂' => Some('2'), '₃' => Some('3'), '₄' => Some('4'),
                    '₅' => Some('5'), '₆' => Some('6'), '₇' => Some('7'), '₈' => Some('8'), '₉' => Some('9'),
                    _ => None,
                };
                let mut r = String::new();
                let mut i = 0;
                while i < cs.len() {
                    let c = cs[i];
                    if (c.is_alphanumeric() || c == ')') && i + 1 < cs.len() && (sup(cs[i + 1]).is_some() || sub(cs[i + 1]).is_some()) {
                        // gom het mot day chi so
                        let mut j = i + 1;
                        let mut s_up = String::new();
                        let mut s_dn = String::new();
                        while j < cs.len() {
                            if let Some(x) = sup(cs[j]) {
                                s_up.push(x);
                            } else if let Some(x) = sub(cs[j]) {
                                s_dn.push(x);
                            } else {
                                break;
                            }
                            j += 1;
                        }
                        let mut f = format!("${}", c);
                        if !s_dn.is_empty() {
                            f.push_str(&format!("_{{{s_dn}}}"));
                        }
                        if !s_up.is_empty() {
                            f.push_str(&format!("^{{{s_up}}}"));
                        }
                        f.push('$');
                        r.push_str(&f);
                        c_script += 1;
                        i = j;
                        continue;
                    }
                    r.push(c);
                    i += 1;
                }
                seg = r;
            }
            if o.punct_space {
                // bo dau cach truoc , ; . ! ?  (khi theo sau la dau cach hoac het dong), khong dong vao URL/so
                let cs: Vec<char> = seg.chars().collect();
                let mut r = String::new();
                let mut i = 0;
                while i < cs.len() {
                    if cs[i] == ' ' {
                        let mut j = i;
                        while j < cs.len() && cs[j] == ' ' {
                            j += 1;
                        }
                        if j < cs.len() && matches!(cs[j], ',' | ';' | '!' | '?') && (j + 1 >= cs.len() || cs[j + 1] == ' ') && i > 0 && cs[i - 1].is_alphanumeric() {
                            c_punct += 1;
                            i = j;
                            continue;
                        }
                    }
                    r.push(cs[i]);
                    i += 1;
                }
                seg = r;
            }
            if o.quotes {
                let before = seg.clone();
                seg = seg.replace(['\u{201C}', '\u{201D}', '\u{201E}'], "\"").replace(['\u{2018}', '\u{2019}', '\u{201A}'], "'");
                if seg != before {
                    c_quote += 1;
                }
            }
            rebuilt.push_str(&collapse_ws_ascii(&seg));
        }
        line = rebuilt;

        // tieu de thieu dau cach: "#Tieu de" -> "# Tieu de"
        if o.heading_space {
            let hs = line.chars().take_while(|c| *c == '#').count();
            if (1..=6).contains(&hs) {
                let rest: String = line.chars().skip(hs).collect();
                if let Some(f) = rest.chars().next() {
                    if f != ' ' && f != '#' && f != '!' && f != '[' && (f.is_alphanumeric() || f == '*' || f == '_') {
                        line = format!("{} {}", "#".repeat(hs), rest);
                        c_head += 1;
                    }
                }
            }
        }
        // danh sach: "• a", "● a", "▪ a", "○ a", "– a" -> "- a"
        if o.bullets {
            let ind: String = line.chars().take_while(|c| *c == ' ').collect();
            let rest = &line[ind.len()..];
            let mut it = rest.chars();
            if let Some(f) = it.next() {
                if matches!(f, '•' | '●' | '▪' | '○' | '◦' | '■' | '□' | '▫' | '·' | '‣' | '⁃' | '–' | '—' | '➢' | '➤' | '✓' | '✔' | '➔')
                    && it.next().map_or(false, |s| s == ' ')
                {
                    let body: String = rest.chars().skip(2).collect();
                    // "– " co the la gach ngang trong cau: chi doi khi dong bat dau bang no
                    line = format!("{ind}- {}", body.trim_start());
                    c_bullet += 1;
                }
            }
        }
        // khoang trang cuoi dong
        if o.trim {
            let trimmed = line.trim_end_matches(' ');
            let spaces = line.len() - trimmed.len();
            if spaces > 0 {
                if spaces >= 2 && !trimmed.is_empty() {
                    if spaces != 2 {
                        line = format!("{trimmed}  ");
                        c_trim += 1;
                    }
                } else {
                    line = trimmed.to_string();
                    c_trim += 1;
                }
            }
        }
        // dong trong lien tiep
        if line.trim().is_empty() {
            blank_run += 1;
            if o.trim && blank_run > 1 {
                c_blank += 1;
                continue;
            }
            out.push(String::new());
            continue;
        }
        blank_run = 0;

        // bang: can dong trong phia truoc
        if o.table_fix && is_table_line(&line) {
            if let Some(prev) = out.last() {
                if !prev.trim().is_empty() && !is_table_line(prev) {
                    out.push(String::new());
                    c_tbl += 1;
                }
            }
            // thieu dong phan cach |---|---|: chen sau dong dau tien cua bang
            let prev_is_table = out.last().map_or(false, |p| is_table_line(p));
            if !prev_is_table {
                let next = src_lines.get(idx + 1);
                if let Some(n) = next {
                    if is_table_line(n) && !is_table_sep(n) && !is_table_sep(&line) {
                        out.push(line.clone());
                        let cols = table_cols(&line).max(1);
                        out.push(format!("|{}", "---|".repeat(cols)));
                        c_tbl += 1;
                        continue;
                    }
                }
            }
        }
        out.push(line);
    }
    let mut res = out.join("\n");
    if o.trim {
        let t2 = res.trim_end_matches('\n');
        let fin = format!("{t2}\n");
        if fin != res {
            // thay doi cuoi file chi tinh khi that su khac (khong bao cao neu chi them dau xuong dong)
            if !res.ends_with('\n') || res.ends_with("\n\n") {
                c_blank += 1;
            }
            res = fin;
        }
    }
    add("Ký tự Tab → 4 dấu cách", c_tab, &mut ch);
    add("Khoảng trắng thừa cuối dòng", c_trim, &mut ch);
    add("Dòng trống thừa", c_blank, &mut ch);
    add("Tiêu đề thiếu dấu cách sau #", c_head, &mut ch);
    add("Ký hiệu danh sách (•, ●, –…) → \"-\"", c_bullet, &mut ch);
    add("Dấu cách thừa trước dấu câu", c_punct, &mut ch);
    add("Dấu nháy cong → nháy thẳng", c_quote, &mut ch);
    add("Bảng: thêm dòng trống / dòng phân cách |---|", c_tbl, &mut ch);
    add("Dấu công thức \\( \\) \\[ \\] → $ $$", c_delim, &mut ch);
    add("Khoảng trắng thừa trong $ … $", c_mtrim, &mut ch);
    add("Dòng LaTeX trần → khối $$ … $$", c_bare, &mut ch);
    add("Dấu $ tiền tệ được thoát (\\$)", c_cur, &mut ch);
    add("Chỉ số trên/dưới Unicode → công thức", c_script, &mut ch);
    (res, ch)
}

// ------------------------------------------------------------- kiem tra cong thuc
fn brace_check(src: &str) -> Option<String> {
    let mut depth: i32 = 0;
    let cs: Vec<char> = src.chars().collect();
    let mut i = 0;
    while i < cs.len() {
        match cs[i] {
            '\\' => i += 1,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth < 0 {
                    return Some("thừa dấu } đóng".into());
                }
            }
            _ => {}
        }
        i += 1;
    }
    if depth > 0 {
        Some(format!("thiếu {depth} dấu }} đóng"))
    } else {
        None
    }
}

fn check_one(src: &str, out: &mut Vec<String>) {
    let s = src.trim();
    if s.is_empty() {
        out.push("công thức rỗng".into());
        return;
    }
    if let Some(m) = brace_check(s) {
        out.push(m);
    }
    let l = s.matches("\\left").count();
    let r = s.matches("\\right").count();
    if l != r {
        out.push(format!("\\left ({l}) và \\right ({r}) không khớp"));
    }
    let b = s.matches("\\begin{").count();
    let e = s.matches("\\end{").count();
    if b != e {
        out.push(format!("\\begin ({b}) và \\end ({e}) không khớp"));
    }
    if s.ends_with('^') || s.ends_with('_') {
        out.push("kết thúc bằng ^ hoặc _ mà không có chỉ số".into());
    }
    let p = math::parse(s);
    let mut seen: Vec<String> = Vec::new();
    for e in p.errors {
        if !seen.contains(&e) {
            seen.push(e);
        }
    }
    if !seen.is_empty() {
        out.push(format!("lệnh chưa hỗ trợ/sai: {}", seen.join(", ")));
    }
}

/// Quet toan bo tai lieu, bao cao cong thuc co van de.
pub fn check_formulas(text: &str) -> (usize, Vec<Issue>) {
    let mut issues = Vec::new();
    let mut total = 0usize;
    let mut fence: Option<char> = None;
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    let snip = |s: &str| -> String {
        let t: String = s.trim().chars().take(70).collect();
        t
    };
    while i < lines.len() {
        let line = lines[i];
        if let Some(f) = is_fence(line) {
            match fence {
                None => fence = Some(f),
                Some(g) if g == f => fence = None,
                _ => {}
            }
            i += 1;
            continue;
        }
        if fence.is_some() {
            i += 1;
            continue;
        }
        let tr = line.trim();
        // khoi $$ nhieu dong
        if tr.starts_with("$$") && !(tr.len() > 3 && tr[2..].contains("$$")) {
            let start = i;
            let mut body = tr[2..].to_string();
            let mut j = i + 1;
            let mut closed = false;
            while j < lines.len() {
                let lt = lines[j];
                if let Some(p) = lt.find("$$") {
                    body.push('\n');
                    body.push_str(&lt[..p]);
                    closed = true;
                    break;
                }
                body.push('\n');
                body.push_str(lt);
                j += 1;
                if j - start > 200 {
                    break;
                }
            }
            total += 1;
            let mut msgs = Vec::new();
            if !closed {
                msgs.push("khối $$ không có $$ đóng".to_string());
            } else {
                check_one(&body, &mut msgs);
            }
            for m in msgs {
                issues.push(Issue { line: start + 1, snippet: snip(&body.replace('\n', " ")), msg: m });
            }
            i = if closed { j + 1 } else { start + 1 };
            continue;
        }
        let segs = split_protected(line);
        let mut dollars = 0;
        for (s, p) in &segs {
            if *p && s.starts_with('$') {
                let n = if s.starts_with("$$") { 2 } else { 1 };
                let inner = &s[n..s.len() - n];
                total += 1;
                let mut msgs = Vec::new();
                check_one(inner, &mut msgs);
                for m in msgs {
                    issues.push(Issue { line: i + 1, snippet: snip(inner), msg: m });
                }
            } else if !*p {
                // dau $ le (khong thoat)
                let cs: Vec<char> = s.chars().collect();
                let mut k = 0;
                while k < cs.len() {
                    if cs[k] == '\\' {
                        k += 2;
                        continue;
                    }
                    if cs[k] == '$' {
                        dollars += 1;
                    }
                    k += 1;
                }
            }
        }
        if dollars > 0 {
            issues.push(Issue { line: i + 1, snippet: snip(line), msg: "dấu $ lẻ (không có $ đóng) — nếu là tiền tệ hãy viết \\$".into() });
        }
        if tr.contains("\\(") || tr.contains("\\[") {
            issues.push(Issue { line: i + 1, snippet: snip(line), msg: "dùng \\( \\) / \\[ \\] — nên đổi sang $ … $ / $$ … $$ (nút Sửa lỗi tự đổi)".into() });
        }
        i += 1;
    }
    (total, issues)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mojibake_fixed() {
        let bad = "Viá»‡t Nam";
        assert_eq!(demojibake_line(bad).unwrap(), "Việt Nam");
    }

    #[test]
    fn tcvn3_converted() {
        let (r, n) = legacy_to_unicode("ViÖt Nam", Legacy::Tcvn3);
        assert_eq!(r, "Việt Nam");
        assert!(n >= 1);
    }

    #[test]
    fn symbol_mapped() {
        let s = format!("{}{}", '\u{F061}', '\u{F0A3}');
        let (r, _) = fix(&s, &Opts::recommended());
        assert_eq!(r.trim(), "α≤");
    }

    #[test]
    fn math_delims_and_trim() {
        let (r, _) = fix("Cho \\( x^2 \\) và $ y $ nhé\n\\[\n a+b \n\\]\n", &Opts::recommended());
        assert!(r.contains("$x^2$"), "{r}");
        assert!(r.contains("$y$"), "{r}");
        assert!(r.contains("$$\n a+b\n$$"), "{r}");
    }

    #[test]
    fn bare_latex_wrapped() {
        let (r, _) = fix("\\frac{a}{b} = \\sqrt{c}\n", &Opts::recommended());
        assert!(r.starts_with("$$\\frac{a}{b} = \\sqrt{c}$$"), "{r}");
    }

    #[test]
    fn currency_escaped() {
        let (r, _) = fix("Giá $5 và $10.\n", &Opts::recommended());
        assert!(r.contains("\\$5") && r.contains("\\$10"), "{r}");
    }

    #[test]
    fn table_fixed() {
        let (r, _) = fix("Bảng:\n| a | b |\n| 1 | 2 |\n", &Opts::recommended());
        assert!(r.contains("\n\n| a | b |\n|---|---|\n| 1 | 2 |"), "{r}");
    }

    #[test]
    fn code_untouched() {
        let src = "```\n• a  \n\\( x \\)\n```\n";
        let (r, _) = fix(src, &Opts::recommended());
        assert_eq!(r, src);
    }

    #[test]
    fn check_reports() {
        let (n, iss) = check_formulas("a $\\foo$ b\n\n$$\n\\frac{a}{b\n$$\n\ngiá $5\n");
        assert!(n >= 2);
        assert!(iss.iter().any(|i| i.msg.contains("\\foo")));
        assert!(iss.iter().any(|i| i.msg.contains("thiếu")));
        assert!(iss.iter().any(|i| i.msg.contains("$ lẻ")));
    }

    /// Chay bo sua loi tren file mau (dung de kiem tra bang mat): MDT_FIX_IN -> MDT_FIX_OUT
    #[test]
    fn fix_sample_file() {
        let (Ok(i), Ok(o)) = (std::env::var("MDT_FIX_IN"), std::env::var("MDT_FIX_OUT")) else { return };
        let t = std::fs::read_to_string(i).unwrap();
        let (r, ch) = fix(&t, &Opts::recommended());
        for c in &ch {
            eprintln!("{} x{}", c.label, c.count);
        }
        std::fs::write(o, r).unwrap();
    }
}
