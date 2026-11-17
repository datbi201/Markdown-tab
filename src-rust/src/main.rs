// =====================================================================
//  Markdown Tabs — ban Rust native v0.1
//  * EXE native doc lap: khoi dong nhanh, icon rieng (khong phai icon Python)
//  * Single-instance: mo file .md thu hai tu Explorer -> mo TAB trong cua
//    so dang chay va nang cua so do len (truyen qua TCP loopback 127.0.0.1)
//  * Da tab, soan thao + xem truoc truc tiep, muc luc (TOC), sang/toi,
//    tim kiem, xuat HTML, khoi phuc phien, tu tai lai khi file doi tren dia
//  Gioi han v0.1 (so voi ban Python): chua co chuyen doi docx/pdf/...,
//  chua xuat PDF, anh trong preview hien o giu cho (chua tai anh).
// =====================================================================
#![cfg_attr(windows, windows_subsystem = "windows")]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant, SystemTime};

mod docx;
mod features;
mod fixer;
mod vn_tables;
mod math;

use eframe::egui;
use egui::text::{CCursor, CCursorRange, LayoutJob};
use egui::{Color32, FontData, FontDefinitions, FontFamily, FontId, RichText, TextFormat};
use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

// ------------------------------------------------------------------ consts
const APP_TITLE: &str = "Markdown Tabs";
const VERSION: &str = "0.5.0-rust";
const IPC_PORT: u16 = 47655;
const IPC_MAGIC: &str = "MDTABS/1";
const MD_EXTS: &[&str] = &["md", "markdown", "mdown", "mkd", "txt"];
static EGCTX: OnceLock<egui::Context> = OnceLock::new();

// ------------------------------------------------------- single instance
/// Neu da co instance khac dang chay: gui danh sach file sang cho no va
/// tra ve true (instance nay thoat ngay). Khong co thi tra ve false.
fn try_forward_to_running_instance(paths: &[PathBuf]) -> bool {
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], IPC_PORT));
    let mut stream = match TcpStream::connect_timeout(&addr, Duration::from_millis(350)) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
    let mut msg = String::from(IPC_MAGIC);
    msg.push('\n');
    for p in paths {
        msg.push_str(&p.to_string_lossy());
        msg.push('\n');
    }
    if stream.write_all(msg.as_bytes()).is_err() {
        return false;
    }
    let mut ack = [0u8; 2];
    match stream.read_exact(&mut ack) {
        Ok(()) => &ack == b"OK",
        Err(_) => false,
    }
}

enum IpcMsg {
    Open(Vec<PathBuf>),
}

/// Mo server lang nghe (instance chinh). Tra ve Receiver neu bind duoc cong.
fn start_ipc_server() -> Option<Receiver<IpcMsg>> {
    let listener = TcpListener::bind(("127.0.0.1", IPC_PORT)).ok()?;
    let (tx, rx): (Sender<IpcMsg>, Receiver<IpcMsg>) = channel();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let mut stream = match conn {
                Ok(s) => s,
                Err(_) => continue,
            };
            let _ = stream.set_read_timeout(Some(Duration::from_millis(800)));
            let cloned = match stream.try_clone() {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut reader = BufReader::new(cloned);
            let mut first = String::new();
            if reader.read_line(&mut first).is_err() || first.trim() != IPC_MAGIC {
                continue;
            }
            let _ = stream.write_all(b"OK");
            let mut paths = Vec::new();
            let mut line = String::new();
            while reader.read_line(&mut line).is_ok() {
                let t = line.trim_end_matches(['\r', '\n']);
                if t.is_empty() {
                    break;
                }
                paths.push(PathBuf::from(t));
                line = String::new();
            }
            let _ = tx.send(IpcMsg::Open(paths));
            if let Some(ctx) = EGCTX.get() {
                ctx.request_repaint();
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
    });
    Some(rx)
}

// ---------------------------------------------------------------- helpers
fn is_md_path(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(|e| MD_EXTS.contains(&e.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Enc {
    Utf8,
    Utf8Bom,
    Cp1258,
}
impl Enc {
    fn label(self) -> &'static str {
        match self {
            Enc::Utf8 => "UTF-8",
            Enc::Utf8Bom => "UTF-8 BOM",
            Enc::Cp1258 => "Windows-1258",
        }
    }
}

fn decode_bytes(raw: &[u8]) -> (String, Enc) {
    if let Some(rest) = raw.strip_prefix(b"\xef\xbb\xbf") {
        return (String::from_utf8_lossy(rest).into_owned(), Enc::Utf8Bom);
    }
    match std::str::from_utf8(raw) {
        Ok(s) => (s.to_string(), Enc::Utf8),
        Err(_) => {
            let (cow, _, _) = encoding_rs::WINDOWS_1258.decode(raw);
            (cow.into_owned(), Enc::Cp1258)
        }
    }
}

fn encode_text(text: &str, enc: Enc) -> Vec<u8> {
    match enc {
        Enc::Utf8 => text.as_bytes().to_vec(),
        Enc::Utf8Bom => {
            let mut v = b"\xef\xbb\xbf".to_vec();
            v.extend_from_slice(text.as_bytes());
            v
        }
        Enc::Cp1258 => encoding_rs::WINDOWS_1258.encode(text).0.into_owned(),
    }
}

fn config_dir() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        return PathBuf::from(appdata).join("MarkdownTabs");
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("MarkdownTabs");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config").join("MarkdownTabs")
}

fn hash_text(s: &str) -> u64 {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}

// ------------------------------------------------------------ markdown AST
#[derive(Clone, Debug)]
struct Span {
    text: String,
    strong: bool,
    em: bool,
    code: bool,
    strike: bool,
    link: Option<String>,
    image: Option<String>,
    /// Some(display) = cong thuc LaTeX (text la ma nguon); display = true -> cong thuc rieng dong
    math: Option<bool>,
}

impl Span {
    fn plain(text: String) -> Self {
        Span { text, strong: false, em: false, code: false, strike: false, link: None, image: None, math: None }
    }
}

#[derive(Clone, Debug)]
enum Block {
    Paragraph(Vec<Span>),
    Heading(u8, Vec<Span>, usize), // level, spans, char-offset trong nguon
    Code(String, String),         // ngon ngu, noi dung
    List { ordered: bool, items: Vec<Vec<Block>> },
    Quote(Vec<Block>),
    Table { head: Vec<Vec<Span>>, rows: Vec<Vec<Vec<Span>>> },
    Rule,
    Html(String),
}

#[derive(Clone)]
struct Style {
    strong: bool,
    em: bool,
    strike: bool,
    link: Option<String>,
    image: Option<String>,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum FrameKind {
    Root,
    Quote,
    List,
    Item,
    Table,
}

struct Frame {
    kind: FrameKind,
    blocks: Vec<Block>,
    list_ordered: bool,
    list_items: Vec<Vec<Block>>,
    tbl_head: Vec<Vec<Span>>,
    tbl_rows: Vec<Vec<Vec<Span>>>,
    tbl_row: Vec<Vec<Span>>,
}

impl Frame {
    fn new(kind: FrameKind) -> Self {
        Frame {
            kind,
            blocks: Vec::new(),
            list_ordered: false,
            list_items: Vec::new(),
            tbl_head: Vec::new(),
            tbl_rows: Vec::new(),
            tbl_row: Vec::new(),
        }
    }
}

struct MdParser {
    frames: Vec<Frame>,
    spans: Vec<Span>,
    style: Style,
    style_stack: Vec<Style>,
    pending: Option<Pending>,
    in_code: bool,
    code_info: String,
    code_text: String,
    in_table_head: bool,
    in_cell: bool,
}

#[derive(Clone)]
enum Pending {
    Paragraph,
    Heading(u8, usize),
    Footnote(String),
}

impl MdParser {
    fn new() -> Self {
        MdParser {
            frames: vec![Frame::new(FrameKind::Root)],
            spans: Vec::new(),
            style: Style { strong: false, em: false, strike: false, link: None, image: None },
            style_stack: Vec::new(),
            pending: None,
            in_code: false,
            code_info: String::new(),
            code_text: String::new(),
            in_table_head: false,
            in_cell: false,
        }
    }

    fn top(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("frame stack")
    }

    fn push_span(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        let st = &self.style;
        self.spans.push(Span {
            text,
            strong: st.strong,
            em: st.em,
            code: false,
            strike: st.strike,
            link: st.link.clone(),
            image: st.image.clone(),
            math: None,
        });
    }

    fn flush_pending(&mut self) {
        let spans = std::mem::take(&mut self.spans);
        let block = match self.pending.take() {
            Some(Pending::Paragraph) => Block::Paragraph(spans),
            Some(Pending::Heading(lvl, off)) => Block::Heading(lvl, spans, off),
            Some(Pending::Footnote(name)) => {
                let mut s = vec![Span::plain(format!("[{name}]: "))];
                s.extend(spans);
                Block::Paragraph(s)
            }
            None => {
                if spans.is_empty() {
                    return;
                }
                Block::Paragraph(spans)
            }
        };
        self.top().blocks.push(block);
    }

    fn run(mut self, text: &str) -> Vec<Block> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_TASKLISTS);
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_FOOTNOTES);
        opts.insert(Options::ENABLE_MATH);
        let parser = Parser::new_ext(text, opts).into_offset_iter();
        for (event, range) in parser {
            match event {
                Event::Start(tag) => self.start_tag(tag, range.start, text),
                Event::End(end) => self.end_tag(end),
                Event::Text(t) => {
                    if self.in_code {
                        self.code_text.push_str(&t);
                    } else {
                        self.push_span(t.to_string());
                    }
                }
                Event::Code(t) => {
                    let st = &self.style;
                    self.spans.push(Span {
                        text: t.to_string(),
                        strong: st.strong,
                        em: st.em,
                        code: true,
                        strike: false,
                        link: st.link.clone(),
                        image: None,
                        math: None,
                    });
                }
                Event::InlineMath(t) => {
                    let mut sp = Span::plain(t.to_string());
                    sp.math = Some(false);
                    self.spans.push(sp);
                }
                Event::DisplayMath(t) => {
                    let mut sp = Span::plain(t.to_string());
                    sp.math = Some(true);
                    self.spans.push(sp);
                }
                Event::SoftBreak => self.push_span(" ".into()),
                Event::HardBreak => self.push_span("\n".into()),
                Event::TaskListMarker(done) => {
                    self.push_span(if done { "☑ ".into() } else { "☐ ".into() })
                }
                Event::Html(t) | Event::InlineHtml(t) => {
                    if self.in_code {
                        self.code_text.push_str(&t);
                    } else {
                        let mut s = Span::plain(t.to_string());
                        s.code = true;
                        self.spans.push(s);
                    }
                }
                Event::FootnoteReference(name) => {
                    self.push_span(format!("[^{name}]"))
                }
                Event::Rule => {
                    self.flush_pending();
                    self.top().blocks.push(Block::Rule)
                }
                _ => {}
            }
        }
        self.flush_pending();
        let mut root = self.frames.pop().expect("root frame");
        std::mem::take(&mut root.blocks)
    }

    fn start_tag(&mut self, tag: Tag, offset: usize, src: &str) {
        match tag {
            Tag::Paragraph => self.pending = Some(Pending::Paragraph),
            Tag::Heading { level, .. } => {
                let lvl = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                // offset theo byte -> doi sang char index de dung cho editor
                let char_off = src[..offset.min(src.len())].chars().count();
                self.pending = Some(Pending::Heading(lvl, char_off));
            }
            Tag::CodeBlock(info) => {
                self.flush_pending();
                self.in_code = true;
                self.code_text.clear();
                self.code_info = match info {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                };
            }
            Tag::List(start) => {
                self.flush_pending();
                let mut f = Frame::new(FrameKind::List);
                f.list_ordered = start.is_some();
                self.frames.push(f);
            }
            Tag::Item => self.frames.push(Frame::new(FrameKind::Item)),
            Tag::BlockQuote(_) => {
                self.flush_pending();
                self.frames.push(Frame::new(FrameKind::Quote))
            }
            Tag::Table(_) => {
                self.flush_pending();
                self.frames.push(Frame::new(FrameKind::Table))
            }
            Tag::TableHead => {
                self.in_table_head = true;
                if let Some(f) = self.frames.last_mut() {
                    f.tbl_row = Vec::new();
                }
            }
            Tag::TableRow => {
                if let Some(f) = self.frames.last_mut() {
                    f.tbl_row = Vec::new();
                }
            }
            Tag::TableCell => {
                self.in_cell = true;
                self.spans = Vec::new();
            }
            Tag::Emphasis => {
                self.style_stack.push(self.style.clone());
                self.style.em = true;
            }
            Tag::Strong => {
                self.style_stack.push(self.style.clone());
                self.style.strong = true;
            }
            Tag::Strikethrough => {
                self.style_stack.push(self.style.clone());
                self.style.strike = true;
            }
            Tag::Link { dest_url, .. } => {
                self.style_stack.push(self.style.clone());
                self.style.link = Some(dest_url.to_string());
            }
            Tag::Image { dest_url, .. } => {
                self.style_stack.push(self.style.clone());
                self.style.image = Some(dest_url.to_string());
            }
            Tag::FootnoteDefinition(name) => {
                self.pending = Some(Pending::Footnote(name.to_string()))
            }
            Tag::HtmlBlock => {
                self.flush_pending();
                self.in_code = true;
                self.code_text.clear();
                self.code_info = "html".into();
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, end: TagEnd) {
        match end {
            TagEnd::Paragraph | TagEnd::FootnoteDefinition => self.flush_pending(),
            TagEnd::Heading(_) => self.flush_pending(),
            TagEnd::CodeBlock => {
                self.in_code = false;
                let text = std::mem::take(&mut self.code_text);
                let info = std::mem::take(&mut self.code_info);
                self.top().blocks.push(Block::Code(info, text));
            }
            TagEnd::HtmlBlock => {
                self.in_code = false;
                let text = std::mem::take(&mut self.code_text);
                self.top().blocks.push(Block::Html(text));
            }
            TagEnd::Item => {
                self.flush_pending();
                if let Some(mut f) = self.frames.pop() {
                    if f.kind == FrameKind::Item {
                        let blocks = std::mem::take(&mut f.blocks);
                        self.top().list_items.push(blocks);
                    }
                }
            }
            TagEnd::List(_) => {
                if let Some(mut f) = self.frames.pop() {
                    let items = std::mem::take(&mut f.list_items);
                    let ordered = f.list_ordered;
                    self.top().blocks.push(Block::List { ordered, items });
                }
            }
            TagEnd::BlockQuote(_) => {
                self.flush_pending();
                if let Some(mut f) = self.frames.pop() {
                    let blocks = std::mem::take(&mut f.blocks);
                    self.top().blocks.push(Block::Quote(blocks));
                }
            }
            TagEnd::Table => {
                if let Some(mut f) = self.frames.pop() {
                    let head = std::mem::take(&mut f.tbl_head);
                    let rows = std::mem::take(&mut f.tbl_rows);
                    self.top().blocks.push(Block::Table { head, rows });
                }
            }
            TagEnd::TableHead => {
                self.in_table_head = false;
                if let Some(f) = self.frames.last_mut() {
                    f.tbl_head = std::mem::take(&mut f.tbl_row);
                }
            }
            TagEnd::TableRow => {
                if let Some(f) = self.frames.last_mut() {
                    let row = std::mem::take(&mut f.tbl_row);
                    f.tbl_rows.push(row);
                }
            }
            TagEnd::TableCell => {
                self.in_cell = false;
                let cell = std::mem::take(&mut self.spans);
                if let Some(f) = self.frames.last_mut() {
                    f.tbl_row.push(cell);
                }
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link
            | TagEnd::Image => {
                if let Some(prev) = self.style_stack.pop() {
                    self.style = prev;
                }
            }
            _ => {}
        }
    }
}

fn parse_markdown(text: &str) -> Vec<Block> {
    MdParser::new().run(text)
}

#[derive(Clone, Debug)]
struct TocEntry {
    level: u8,
    title: String,
    char_offset: usize,
}

fn toc_of(blocks: &[Block], out: &mut Vec<TocEntry>) {
    for b in blocks {
        match b {
            Block::Heading(lvl, spans, off) => {
                let title: String = spans.iter().map(|s| s.text.as_str()).collect();
                out.push(TocEntry { level: *lvl, title, char_offset: *off });
            }
            Block::Quote(inner) => toc_of(inner, out),
            Block::List { items, .. } => {
                for it in items {
                    toc_of(it, out);
                }
            }
            _ => {}
        }
    }
}

// ------------------------------------------------------- preview renderer
enum PreviewAction {
    OpenUrl(String),
    OpenFile(PathBuf),
    Anchor(String),
}

fn link_color(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(88, 166, 255)
    } else {
        Color32::from_rgb(9, 105, 218)
    }
}

fn spans_to_job(spans: &[Span], size: f32, bold_all: bool, ui: &egui::Ui, break_anywhere: bool) -> LayoutJob {
    let mut job = LayoutJob::default();
    let base = ui.visuals().text_color();
    for s in spans {
        let mut fmt = TextFormat {
            font_id: if s.code {
                FontId::monospace(size - 1.5)
            } else {
                FontId::proportional(size)
            },
            color: if s.code {
                Color32::from_rgb(207, 110, 32)
            } else {
                base
            },
            italics: s.em,
            ..Default::default()
        };
        if s.strong || bold_all {
            fmt.color = base; // egui khong co bold that trong 1 font; dung mau dam + font bold family
            fmt.font_id = FontId::new(
                size,
                egui::FontFamily::Name("InterSemiBold".into()),
            );
            if s.code {
                fmt.font_id = FontId::monospace(size - 1.5);
            }
        }
        if s.strike {
            fmt.strikethrough = egui::Stroke::new(1.0_f32, base);
        }
        if s.code {
            fmt.background = ui.visuals().faint_bg_color;
        }
        job.append(&s.text, 0.0, fmt);
    }
    job.wrap.max_width = f32::INFINITY; // Label tu xu ly wrap
    job.wrap.break_anywhere = break_anywhere; // chi bat trong o bang: tu qua dai (SHA, URL) tu be dong thay vi tran cot
    job
}

fn spans_have_link(spans: &[Span]) -> bool {
    spans.iter().any(|s| s.link.is_some() || s.image.is_some() || s.math.is_some())
}

/// Ve mot cong thuc (inline hoac display) vao `ui`; tra ve response.
fn show_math(ui: &mut egui::Ui, src: &str, display: bool, size: f32) -> egui::Response {
    let color = ui.visuals().text_color();
    let msize = if display { size * 1.18 } else { size * 1.04 };
    let mb = math::layout(ui.ctx(), src.trim(), display, msize, color);
    let h = mb.asc + mb.desc;
    // can dong theo duong co so cua chu: tam hop cong thuc cach co so ~0.36 co chu
    let (mut pad_top, mut pad_bot) = (0.0f32, 0.0f32);
    if !display {
        let c = 0.36 * size;
        let cb = (mb.asc - mb.desc) / 2.0;
        if cb < c {
            pad_top = 2.0 * (c - cb);
        } else {
            pad_bot = 2.0 * (cb - c);
        }
    }
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(mb.w + 2.0, h + 2.0 + pad_top + pad_bot), egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect.expand(4.0));
        mb.paint(&painter, rect.min + egui::vec2(1.0, 1.0 + pad_top));
        if mb.has_error {
            painter.line_segment(
                [rect.left_bottom() - egui::vec2(0.0, pad_bot), rect.right_bottom() - egui::vec2(0.0, pad_bot)],
                egui::Stroke::new(1.0, Color32::from_rgb(220, 70, 70)),
            );
        }
    }
    if mb.has_error {
        resp.on_hover_text("Công thức có lệnh LaTeX chưa hỗ trợ (đã hiển thị nguyên văn lệnh đó). Dùng Công cụ ▸ Kiểm tra công thức.")
    } else {
        resp
    }
}

/// Ve mot khoi spans: neu co link/anh thi ve tung span (link bam duoc),
/// khong thi gop 1 LayoutJob cho dep.
type ImgCache = std::collections::HashMap<PathBuf, Option<egui::TextureHandle>>;

fn load_image_cached(cache: &mut ImgCache, ctx: &egui::Context, path: &Path) -> Option<egui::TextureHandle> {
    if let Some(v) = cache.get(path) {
        return v.clone();
    }
    let result = (|| -> Option<egui::TextureHandle> {
        let img = image::open(path).ok()?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
        Some(ctx.load_texture(
            format!("mdimg-{}", path.display()),
            ci,
            egui::TextureOptions::LINEAR,
        ))
    })();
    cache.insert(path.to_path_buf(), result.clone());
    result
}

fn render_spans(
    ui: &mut egui::Ui,
    spans: &[Span],
    size: f32,
    bold_all: bool,
    doc_dir: &Path,
    actions: &mut Vec<PreviewAction>,
    cache: &mut ImgCache,
    in_table: bool,
    zs: f32,
) {
    if !spans_have_link(spans) {
        ui.label(spans_to_job(spans, size, bold_all, ui, in_table));
        return;
    }
    // doan chi gom mot cong thuc display -> can giua, rieng dong
    if let [only] = spans {
        if let Some(true) = only.math {
            ui.vertical_centered(|ui| {
                show_math(ui, &only.text, true, size);
            });
            return;
        }
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for s in spans {
            if let Some(disp) = s.math {
                show_math(ui, &s.text, disp && false, size);
                continue;
            }
            if let Some(dest) = &s.image {
                let alt = if s.text.trim().is_empty() { "ảnh".to_string() } else { s.text.clone() };
                let is_http = dest.starts_with("http://") || dest.starts_with("https://");
                let local = if is_http {
                    None
                } else {
                    let p = PathBuf::from(dest.split('#').next().unwrap_or(dest));
                    let full = if p.is_absolute() { p } else { doc_dir.join(p) };
                    if full.exists() { Some(full) } else { None }
                };
                let tex = local.as_ref().and_then(|full| load_image_cached(cache, ui.ctx(), full));
                if let Some(tex) = tex {
                    let sz = tex.size_vec2();
                    let scale = (ui.available_width().min(860.0) / sz.x).min(420.0 * zs / sz.y).min(zs.max(0.2));
                    ui.image((tex.id(), sz * scale));
                } else {
                    ui.label(
                        RichText::new(format!("[Ảnh: {alt} → {dest}]"))
                            .color(ui.visuals().weak_text_color())
                            .italics(),
                    );
                }
                continue;
            }
            if let Some(dest) = &s.link {
                let label = ui.add(
                    egui::Label::new(
                        RichText::new(&s.text).color(link_color(ui)).underline(),
                    )
                    .sense(egui::Sense::click()),
                );
                if label.clicked() {
                    if let Some(anchor) = dest.strip_prefix('#') {
                        actions.push(PreviewAction::Anchor(anchor.to_string()));
                    } else if dest.starts_with("http://") || dest.starts_with("https://")
                        || dest.starts_with("mailto:")
                    {
                        actions.push(PreviewAction::OpenUrl(dest.clone()));
                    } else {
                        let p = PathBuf::from(dest.split('#').next().unwrap_or(dest));
                        let full = if p.is_absolute() { p } else { doc_dir.join(p) };
                        if is_md_path(&full) {
                            actions.push(PreviewAction::OpenFile(full));
                        } else {
                            actions.push(PreviewAction::OpenUrl(full.to_string_lossy().into_owned()));
                        }
                    }
                }
            } else {
                ui.label(spans_to_job(std::slice::from_ref(s), size, bold_all, ui, in_table));
            }
        }
    });
}

fn render_blocks(
    ui: &mut egui::Ui,
    blocks: &[Block],
    doc_dir: &Path,
    actions: &mut Vec<PreviewAction>,
    id_seed: &mut u64,
    cache: &mut ImgCache,
    zs: f32,
    in_list: bool,
) {
    for block in blocks {
        match block {
            Block::Paragraph(spans) => {
                render_spans(ui, spans, 15.0 * zs, false, doc_dir, actions, cache, false, zs);
                ui.add_space(if in_list { 1.0 } else { 8.0 });
            }
            Block::Heading(level, spans, off) => {
                let size = match level {
                    1 => 29.0,
                    2 => 23.5,
                    3 => 19.5,
                    4 => 17.5,
                    5 => 16.0,
                    _ => 15.0,
                };
                ui.add_space(if *level <= 2 { 14.0 } else { 9.0 });
                let top_y = ui.cursor().min.y;
                let tgt: Option<usize> = ui.ctx().data(|d| d.get_temp(egui::Id::new("preview-scroll-target")));
                if tgt == Some(*off) {
                    ui.scroll_to_rect(
                        egui::Rect::from_min_size(egui::pos2(ui.min_rect().left(), top_y - 8.0), egui::vec2(10.0, 30.0)),
                        Some(egui::Align::TOP),
                    );
                    ui.ctx().data_mut(|d| d.remove::<usize>(egui::Id::new("preview-scroll-target")));
                }
                render_spans(ui, spans, size * zs, true, doc_dir, actions, cache, false, zs);
                if *level <= 2 {
                    ui.separator();
                }
                ui.add_space(4.0);
            }
            Block::Code(info, text) => {
                egui::Frame::default()
                    .fill(Color32::from_rgb(13, 17, 23))
                    .inner_margin(egui::Margin::same(12.0))
                    .rounding(egui::Rounding::same(8.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        if !info.is_empty() {
                            ui.label(
                                RichText::new(info)
                                    .size(11.0)
                                    .color(Color32::from_rgb(139, 148, 158)),
                            );
                        }
                        // dong code dai: tu xuong dong (be o bat ky dau) thay vi tran ra ngoai
                        let mut job = LayoutJob::single_section(
                            text.trim_end_matches('\n').to_string(),
                            TextFormat {
                                font_id: FontId::monospace(13.0 * zs),
                                color: Color32::from_rgb(230, 237, 243),
                                ..Default::default()
                            },
                        );
                        job.wrap.break_anywhere = true;
                        ui.add(egui::Label::new(job).wrap_mode(egui::TextWrapMode::Wrap));
                    });
                ui.add_space(8.0);
            }
            Block::List { ordered, items } => {
                for (i, item) in items.iter().enumerate() {
                    let is_task = matches!(
                        item.first(),
                        Some(Block::Paragraph(sp)) if sp.first().map_or(false, |x| x.text.starts_with('\u{2611}') || x.text.starts_with('\u{2610}'))
                    );
                    let marker = if is_task {
                        String::new()
                    } else if *ordered {
                        format!("{}.", i + 1)
                    } else {
                        "\u{2022}".to_string()
                    };
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing.x = 4.0;
                        // cot ky hieu rong co dinh, canh phai: so 1. .. 10. thang hang
                        let mw = (if is_task { 2.0 } else if *ordered { 28.0 } else { 18.0 }) * zs;
                        ui.allocate_ui_with_layout(
                            egui::vec2(mw, 18.0 * zs),
                            egui::Layout::right_to_left(egui::Align::Min),
                            |ui| {
                                ui.label(
                                    RichText::new(marker)
                                        .size(15.0 * zs)
                                        .color(ui.visuals().text_color()),
                                );
                            },
                        );
                        ui.vertical(|ui| {
                            render_blocks(ui, item, doc_dir, actions, id_seed, cache, zs, true);
                        });
                    });
                }
                ui.add_space(4.0);
            }
            Block::Quote(inner) => {
                let out = egui::Frame::default()
                    .inner_margin(egui::Margin { left: 16.0, right: 6.0, top: 2.0, bottom: 0.0 })
                    .show(ui, |ui| {
                        render_blocks(ui, inner, doc_dir, actions, id_seed, cache, zs, false);
                    });
                let r = out.response.rect;
                // vach trai cao bang toan bo khoi trich dan (khong co dinh 18px)
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(r.left_top() + egui::vec2(2.0, 2.0), egui::vec2(3.5, (r.height() - 4.0).max(6.0))),
                    2.0,
                    Color32::from_gray(150),
                );
            }
            Block::Table { head, rows } => {
                *id_seed += 1;
                let gid = *id_seed;
                let ncols = head
                    .len()
                    .max(rows.iter().map(|r| r.len()).max().unwrap_or(1))
                    .max(1);
                // chieu rong toi da moi o: chia theo be rong cot noi dung, khong qua 360, khong be hon 90
                let col_w = ((ui.available_width() - 16.0 - 14.0 * (ncols as f32 - 1.0)) / ncols as f32 - 2.0)
                    .clamp(48.0, 360.0);
                egui::Frame::default()
                    .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                    .rounding(egui::Rounding::same(6.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        // trong Grid mac dinh khong xuong dong -> ep Wrap de o dai (SHA, URL) tu be dong
                        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                        egui::Grid::new(egui::Id::new(("tbl", gid)))
                            .striped(true)
                            .spacing([14.0, 6.0])
                            .show(ui, |ui| {
                                if !head.is_empty() {
                                    for cell in head {
                                        ui.scope(|ui| {
                                            ui.set_max_width(col_w);
                                            render_spans(ui, cell, 14.0 * zs, true, doc_dir, actions, cache, true, zs);
                                        });
                                    }
                                    ui.end_row();
                                }
                                for row in rows {
                                    for cell in row {
                                        ui.scope(|ui| {
                                            ui.set_max_width(col_w);
                                            render_spans(ui, cell, 14.0 * zs, false, doc_dir, actions, cache, true, zs);
                                        });
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                ui.add_space(6.0);
            }
            Block::Rule => {
                ui.separator();
                ui.add_space(4.0);
            }
            Block::Html(text) => {
                ui.label(
                    RichText::new(text.trim()).font(FontId::monospace(11.0))
                        .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(4.0);
            }
        }
    }
}

// ----------------------------------------------------- editor highlighting
fn editor_layouter(ui: &egui::Ui, text: &str, wrap_width: f32) -> Arc<egui::Galley> {
    let dark = ui.visuals().dark_mode;
    let normal = ui.visuals().text_color();
    let heading_c = if dark { Color32::from_rgb(121, 192, 255) } else { Color32::from_rgb(5, 80, 174) };
    let code_c = if dark { Color32::from_rgb(165, 214, 255) } else { Color32::from_rgb(160, 80, 0) };
    let mut job = LayoutJob::default();
    job.wrap.max_width = f32::INFINITY; // khong xuong dong: 1 dong logic = 1 dong hien thi (cho cot so dong khop)
    let fsize = ui
        .ctx()
        .data(|d| d.get_temp::<f32>(egui::Id::new("md-editor-font")))
        .unwrap_or(13.5);
    let font = FontId::monospace(fsize);
    let mut in_fence = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let is_fence = trimmed.starts_with("```");
        let color = if in_fence || is_fence {
            code_c
        } else if trimmed.starts_with('#') {
            heading_c
        } else if trimmed.starts_with('>') {
            ui.visuals().weak_text_color()
        } else {
            normal
        };
        job.append(line, 0.0, TextFormat { font_id: font.clone(), color, ..Default::default() });
        if is_fence {
            in_fence = !in_fence;
        }
    }
    if text.is_empty() {
        job.append("", 0.0, TextFormat { font_id: font, color: normal, ..Default::default() });
    }
    ui.fonts(|f| f.layout_job(job))
}

/// Vach ngan doc co chieu cao co dinh (ui.separator trong hang tu xuong dong
/// bi keo cao bat thuong).
fn vsep(ui: &mut egui::Ui) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 20.0), egui::Sense::hover());
    ui.painter().line_segment(
        [rect.center_top(), rect.center_bottom()],
        ui.visuals().widgets.noninteractive.bg_stroke,
    );
}

// ---------------------------------------------------------------- document
struct Doc {
    id: u64,
    path: Option<PathBuf>,
    text: String,
    saved_text: String,
    enc: Enc,
    mtime: Option<SystemTime>,
    ast: Vec<Block>,
    toc: Vec<TocEntry>,
    ast_hash: u64,
    cursor_char: usize,
    sel_start: usize,
    sel_end: usize,
}

impl Doc {
    fn dirty(&self) -> bool {
        self.text != self.saved_text
    }
    fn title(&self) -> String {
        let base = match &self.path {
            Some(p) => p
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
                .unwrap_or_else(|| "tai-lieu.md".into()),
            None => "Tài liệu mới".into(),
        };
        if self.dirty() {
            format!("\u{25CF} {base}")
        } else {
            base
        }
    }
    fn refresh_ast(&mut self) {
        let h = hash_text(&self.text);
        if h != self.ast_hash {
            self.ast_hash = h;
            self.ast = parse_markdown(&self.text);
            self.toc = Vec::new();
            toc_of(&self.ast, &mut self.toc);
        }
    }
    fn load_from(path: &Path) -> std::io::Result<Doc> {
        let raw = std::fs::read(path)?;
        let (text, enc) = decode_bytes(&raw);
        let mtime = std::fs::metadata(path).and_then(|m| m.modified()).ok();
        Ok(Doc {
            id: 0,
            path: Some(path.to_path_buf()),
            saved_text: text.clone(),
            text,
            enc,
            mtime,
            ast: Vec::new(),
            toc: Vec::new(),
            ast_hash: 0,
            cursor_char: 0,
            sel_start: 0,
            sel_end: 0,
        })
    }
}

// ------------------------------------------------------------------ session
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Session {
    files: Vec<String>,
    dark: bool,
    restore: bool,
    #[serde(default = "default_zoom")]
    zoom: f32,
    #[serde(default)]
    recent: Vec<String>,
    #[serde(default)]
    autosave: bool,
}

fn default_zoom() -> f32 {
    1.0
}

fn session_path() -> PathBuf {
    config_dir().join("session.json")
}

fn load_session() -> Session {
    std::fs::read_to_string(session_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Session { files: vec![], dark: false, restore: true, zoom: 1.0, recent: vec![], autosave: false })
}

fn save_session(s: &Session) {
    let dir = config_dir();
    let _ = std::fs::create_dir_all(&dir);
    if let Ok(txt) = serde_json::to_string_pretty(s) {
        let _ = std::fs::write(session_path(), txt);
    }
}

// ---------------------------------------------------------------------- app
#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Edit,
    Preview,
    Split,
}

enum CloseReq {
    Tab(usize),
    Exit,
}

struct App {
    docs: Vec<Doc>,
    active: usize,
    next_id: u64,
    dark: bool,
    view: ViewMode,
    show_toc: bool,
    find_open: bool,
    find_text: String,
    replace_text: String,
    zoom: f32,
    prev_layout: (u8, bool, egui::Rect),
    img_cache: std::collections::HashMap<PathBuf, Option<egui::TextureHandle>>,
    find_goto: Option<(u64, usize, usize)>, // doc_id, from_char, to_char
    toc_goto: Option<(u64, usize)>,         // doc_id, char_offset
    status: String,
    ipc_rx: Option<Receiver<IpcMsg>>,
    close_req: Option<CloseReq>,
    allow_exit: bool,
    last_mtime_check: Instant,
    untitled_no: usize,
    // ---- v0.5: ribbon, tep gan day, hop thoai, tu dong luu ----
    ribbon: features::RibbonTab,
    ribbon_open: bool,
    recent: Vec<String>,
    closed: Vec<PathBuf>,
    dialog: Option<features::Dialog>,
    fx_opts: fixer::Opts,
    fx_changes: Vec<fixer::Change>,
    fx_dirty: bool,
    fx_undo: Option<(u64, String)>,
    fx_hint: String,
    fm_result: Option<(usize, Vec<fixer::Issue>)>,
    autosave: bool,
    last_autosave: Instant,
    fullscreen: bool,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, initial_files: Vec<PathBuf>, session: &Session) -> Self {
        let _ = EGCTX.set(cc.egui_ctx.clone());
        install_fonts(&cc.egui_ctx);
        // zoom chi ap dung cho vung noi dung (editor + preview), khong dung set_zoom_factor toan app
        let mut app = App {
            docs: Vec::new(),
            active: 0,
            next_id: 1,
            dark: session.dark,
            view: ViewMode::Preview, // mac dinh: Xem truoc
            show_toc: true,
            find_open: false,
            find_text: String::new(),
            replace_text: String::new(),
            zoom: if session.zoom > 0.2 && session.zoom < 4.0 { session.zoom } else { 1.0 },
            prev_layout: (9, true, egui::Rect::NOTHING),
            img_cache: std::collections::HashMap::new(),
            find_goto: None,
            toc_goto: None,
            status: String::new(),
            ipc_rx: None,
            close_req: None,
            allow_exit: false,
            last_mtime_check: Instant::now(),
            untitled_no: 0,
            ribbon: features::RibbonTab::File,
            ribbon_open: true,
            recent: session.recent.clone(),
            closed: Vec::new(),
            dialog: None,
            fx_opts: fixer::Opts::recommended(),
            fx_changes: Vec::new(),
            fx_dirty: true,
            fx_undo: None,
            fx_hint: String::new(),
            fm_result: None,
            autosave: session.autosave,
            last_autosave: Instant::now(),
            fullscreen: false,
        };
        apply_theme(&cc.egui_ctx, session.dark);
        let mut files = initial_files;
        if files.is_empty() && session.restore {
            files = session.files.iter().map(PathBuf::from).filter(|p| p.exists()).collect();
        }
        for f in files {
            app.open_path(&f);
        }
        if app.docs.is_empty() {
            app.new_doc();
        }
        // (che do kiem thu giao dien) MDT_TAB=0..6, MDT_DIALOG=fixer|formulas|stats|shortcuts|syntax|about, MDT_VIEW=edit|split
        if let Ok(t) = std::env::var("MDT_TAB") {
            if let Ok(n) = t.parse::<usize>() {
                if let Some((tab, _)) = features::RibbonTab::ALL.get(n) {
                    app.ribbon = *tab;
                }
            }
        }
        if let Ok(a) = std::env::var("MDT_ANCHOR") {
            app.goto_anchor(&cc.egui_ctx, &a);
        }
        match std::env::var("MDT_VIEW").as_deref() {
            Ok("edit") => app.view = ViewMode::Edit,
            Ok("split") => app.view = ViewMode::Split,
            _ => {}
        }
        match std::env::var("MDT_DIALOG").as_deref() {
            Ok("fixer") => app.open_fixer(),
            Ok("formulas") => app.run_formula_check(),
            Ok("stats") => app.dialog = Some(features::Dialog::Stats),
            Ok("shortcuts") => app.dialog = Some(features::Dialog::Shortcuts),
            Ok("syntax") => app.dialog = Some(features::Dialog::Syntax),
            Ok("about") => app.dialog = Some(features::Dialog::About),
            _ => {}
        }
        app
    }

    fn new_doc(&mut self) {
        self.untitled_no += 1;
        let id = self.next_id;
        self.next_id += 1;
        self.docs.push(Doc {
            id,
            path: None,
            text: String::new(),
            saved_text: String::new(),
            enc: Enc::Utf8,
            mtime: None,
            ast: Vec::new(),
            toc: Vec::new(),
            ast_hash: u64::MAX,
            cursor_char: 0,
            sel_start: 0,
            sel_end: 0,
        });
        self.active = self.docs.len() - 1;
    }

    fn open_path(&mut self, path: &Path) {
        if path.is_dir() {
            let mut entries: Vec<PathBuf> = Vec::new();
            collect_md_files(path, &mut entries, 0);
            entries.sort();
            for e in entries {
                self.open_path(&e);
            }
            return;
        }
        if !path.exists() {
            self.status = format!("Không tìm thấy file: {}", path.display());
            return;
        }
        if let Some(idx) = self.docs.iter().position(|d| {
            d.path.as_deref().map(|p| same_path(p, path)).unwrap_or(false)
        }) {
            self.active = idx;
            self.push_recent(path);
            return;
        }
        match Doc::load_from(path) {
            Ok(mut doc) => {
                doc.id = self.next_id;
                self.next_id += 1;
                doc.refresh_ast();
                self.status = format!("Đã mở: {}", path.display());
                self.docs.push(doc);
                self.active = self.docs.len() - 1;
                self.push_recent(path);
            }
            Err(e) => {
                self.status = format!("Không đọc được {}: {e}", path.display());
            }
        }
    }

    fn save_doc(&mut self, idx: usize, force_dialog: bool) -> bool {
        if idx >= self.docs.len() {
            return false;
        }
        if self.docs[idx].path.is_none() || force_dialog {
            let start = self.docs[idx]
                .path
                .clone()
                .unwrap_or_else(|| PathBuf::from("tai-lieu.md"));
            let picked = rfd::FileDialog::new()
                .set_title("Lưu thành…")
                .set_file_name(start.file_name().unwrap_or_default().to_string_lossy())
                .add_filter("Markdown", &["md", "markdown"])
                .save_file();
            match picked {
                Some(p) => self.docs[idx].path = Some(p),
                None => return false,
            }
        }
        let doc = &mut self.docs[idx];
        let path = doc.path.clone().unwrap();
        let bytes = encode_text(&doc.text, doc.enc);
        match std::fs::write(&path, bytes) {
            Ok(()) => {
                doc.saved_text = doc.text.clone();
                doc.mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
                self.status = format!("Đã lưu: {}", path.display());
                self.push_recent(&path);
                true
            }
            Err(e) => {
                self.status = format!("Lỗi khi lưu {}: {e}", path.display());
                false
            }
        }
    }

    fn fmt(&mut self, kind: FmtKind) {
        if self.view == ViewMode::Preview {
            self.view = ViewMode::Split;
        }
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let id = doc.id;
        let (a, b) = fmt_apply_doc(doc, kind);
        if let Some(d) = self.docs.get_mut(self.active) {
            d.sel_start = a;
            d.sel_end = b;
            d.cursor_char = b;
        }
        self.find_goto = Some((id, a, b));
    }

    fn set_zoom(&mut self, _ctx: &egui::Context, z: f32) {
        // Zoom noi dung: chi editor + preview doi co chu; thanh cong cu, tab,
        // muc luc, thanh trang thai giu nguyen kich thuoc.
        let z = ((z.clamp(0.6, 2.0)) * 10.0).round() / 10.0;
        self.zoom = z;
        self.persist_session();
    }

    fn replace_all(&mut self) {
        if self.find_text.is_empty() {
            return;
        }
        let needle = self.find_text.clone();
        let repl = self.replace_text.clone();
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let mut count = 0usize;
        let mut from = 0usize;
        while let Some(pos) = find_ci_from(&doc.text, &needle, from) {
            replace_char_range(&mut doc.text, pos, pos + needle.chars().count(), &repl);
            from = pos + repl.chars().count();
            count += 1;
            if count > 100_000 {
                break;
            }
        }
        self.status = format!("Đã thay thế {count} chỗ");
    }

    fn replace_next(&mut self, ctx: &egui::Context) {
        if self.find_text.is_empty() {
            return;
        }
        let needle = self.find_text.clone();
        let repl = self.replace_text.clone();
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let (sa, sb) = sel_text(doc);
        let cur_sel = slice_chars(&doc.text, sa, sb).to_lowercase();
        if sa < sb && cur_sel == needle.to_lowercase() {
            replace_char_range(&mut doc.text, sa, sb, &repl);
            doc.cursor_char = sa + repl.chars().count();
        } else if let Some(pos) = find_ci_from(&doc.text, &needle, doc.cursor_char) {
            replace_char_range(&mut doc.text, pos, pos + needle.chars().count(), &repl);
            doc.cursor_char = pos + repl.chars().count();
        } else {
            self.status = format!("Không tìm thấy: {}", self.find_text);
            return;
        }
        self.do_find_next(ctx);
    }

    fn persist_session(&self) {
        let files = self
            .docs
            .iter()
            .filter_map(|d| d.path.as_ref().map(|p| p.to_string_lossy().into_owned()))
            .collect();
        save_session(&Session { files, dark: self.dark, restore: true, zoom: self.zoom, recent: self.recent.clone(), autosave: self.autosave });
    }
}

// ------------------------------------------------------------------ theme
pub const ACCENT: Color32 = Color32::from_rgb(21, 66, 115);
pub const ACCENT_BRIGHT: Color32 = Color32::from_rgb(31, 111, 235);
pub const AMBER: Color32 = Color32::from_rgb(232, 147, 12);

pub fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

/// Mau nen "chrome" (toolbar / tabbar / status / TOC) theo theme hien tai.
pub fn chrome_fill(dark: bool) -> Color32 {
    if dark {
        rgb(22, 27, 34)
    } else {
        rgb(248, 249, 251)
    }
}

/// Mau nen dai phia sau cac tab (khac voi nen noi dung).
pub fn tabstrip_fill(dark: bool) -> Color32 {
    if dark {
        rgb(13, 16, 20)
    } else {
        rgb(233, 236, 240)
    }
}

/// Mau nen vung noi dung (khop panel_fill cua theme) - dung de lap day panel
/// tranh vung "trong suot" lo nen den cua he dieu hanh.
pub fn content_fill(dark: bool) -> Color32 {
    if dark {
        rgb(16, 20, 26)
    } else {
        Color32::WHITE
    }
}

pub fn border_color(dark: bool) -> Color32 {
    if dark {
        rgb(42, 49, 60)
    } else {
        rgb(216, 220, 227)
    }
}

fn apply_theme(ctx: &egui::Context, dark: bool) {
    use egui::{Stroke, Style, TextStyle};
    let mut style = Style::default();
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    if dark {
        v.window_fill = rgb(13, 16, 20);
        v.panel_fill = rgb(16, 20, 26);
        v.faint_bg_color = rgb(27, 33, 42);
        v.extreme_bg_color = rgb(10, 13, 17);
        v.code_bg_color = rgb(13, 17, 23);
        v.selection.bg_fill = rgb(38, 79, 130);
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);
        v.hyperlink_color = rgb(88, 166, 255);
        let border = rgb(42, 49, 60);
        v.widgets.noninteractive.bg_fill = rgb(16, 20, 26);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, rgb(200, 209, 217));
        v.widgets.inactive.bg_fill = rgb(78, 90, 108); // cung la mau tay nam thanh cuon
        v.widgets.inactive.weak_bg_fill = rgb(22, 27, 34);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, rgb(230, 237, 243));
        v.widgets.hovered.bg_fill = rgb(110, 124, 146);
        v.widgets.hovered.weak_bg_fill = rgb(30, 37, 47);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, rgb(60, 70, 85));
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.active.bg_fill = ACCENT_BRIGHT;
        v.widgets.active.weak_bg_fill = ACCENT_BRIGHT;
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_BRIGHT);
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.open.bg_fill = rgb(27, 33, 42);
        v.widgets.open.bg_stroke = Stroke::new(1.0, border);
    } else {
        v.window_fill = rgb(233, 236, 240);
        v.panel_fill = Color32::WHITE;
        v.faint_bg_color = rgb(243, 245, 247);
        v.extreme_bg_color = Color32::WHITE;
        v.code_bg_color = rgb(13, 17, 23);
        v.selection.bg_fill = rgb(176, 214, 255);
        v.selection.stroke = Stroke::new(1.0, rgb(10, 40, 90));
        v.hyperlink_color = rgb(9, 105, 218);
        let border = rgb(216, 220, 227);
        v.widgets.noninteractive.bg_fill = Color32::WHITE;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, border);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, rgb(31, 35, 40));
        v.widgets.inactive.bg_fill = rgb(196, 204, 216); // cung la mau tay nam thanh cuon
        v.widgets.inactive.weak_bg_fill = rgb(248, 249, 251);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, border);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, rgb(31, 35, 40));
        v.widgets.hovered.bg_fill = rgb(150, 162, 182);
        v.widgets.hovered.weak_bg_fill = rgb(243, 245, 248);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, rgb(190, 198, 210));
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, rgb(15, 18, 22));
        v.widgets.active.bg_fill = ACCENT;
        v.widgets.active.weak_bg_fill = ACCENT;
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.open.bg_fill = rgb(243, 245, 247);
        v.widgets.open.bg_stroke = Stroke::new(1.0, border);
    }
    let rounding8 = egui::Rounding::same(8.0);
    v.widgets.noninteractive.rounding = rounding8;
    v.widgets.inactive.rounding = rounding8;
    v.widgets.hovered.rounding = rounding8;
    v.widgets.active.rounding = rounding8;
    v.widgets.open.rounding = rounding8;
    v.window_rounding = egui::Rounding::same(12.0);
    v.menu_rounding = egui::Rounding::same(10.0);
    v.striped = true;
    v.slider_trailing_fill = true;
    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(9.0, 6.0);
    style.spacing.button_padding = egui::vec2(11.0, 5.5);
    style.spacing.window_margin = egui::Margin::same(10.0);
    // Thanh cuon luon hien, chiem cho rieng (khong de len noi dung)
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style.spacing.scroll.bar_width = 10.0;
    style.spacing.scroll.dormant_background_opacity = 0.25;
    style.spacing.scroll.dormant_handle_opacity = 0.55;
    style.spacing.scroll.active_background_opacity = 0.4;
    style.spacing.scroll.active_handle_opacity = 0.7;
    style.spacing.interact_size.y = 28.0;
    style.text_styles.insert(TextStyle::Body, FontId::proportional(14.5));
    style.text_styles.insert(TextStyle::Button, FontId::proportional(14.0));
    style.text_styles.insert(TextStyle::Monospace, FontId::monospace(13.5));
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(19.0, egui::FontFamily::Name("InterSemiBold".into())),
    );
    style.text_styles.insert(TextStyle::Small, FontId::proportional(11.5));
    ctx.set_style(style);
}

fn same_path(a: &Path, b: &Path) -> bool {
    let ca = a.canonicalize().unwrap_or_else(|_| a.to_path_buf());
    let cb = b.canonicalize().unwrap_or_else(|_| b.to_path_buf());
    ca == cb
}

fn collect_md_files(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 6 {
        return;
    }
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect_md_files(&p, out, depth + 1);
            } else if is_md_path(&p) {
                out.push(p);
            }
        }
    }
}

fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "dejavu".to_owned(),
        FontData::from_static(include_bytes!("../assets/DejaVuSans.ttf")),
    );
    fonts.font_data.insert(
        "dejavu-bold".to_owned(),
        FontData::from_static(include_bytes!("../assets/DejaVuSans-Bold.ttf")),
    );
    fonts.font_data.insert(
        "inter".to_owned(),
        FontData::from_static(include_bytes!("../assets/Inter-Regular.ttf")),
    );
    fonts.font_data.insert(
        "inter-semibold".to_owned(),
        FontData::from_static(include_bytes!("../assets/Inter-SemiBold.ttf")),
    );
    fonts.font_data.insert(
        "inter-bold".to_owned(),
        FontData::from_static(include_bytes!("../assets/Inter-Bold.ttf")),
    );
    fonts.font_data.insert(
        "jetbrains-mono".to_owned(),
        FontData::from_static(include_bytes!("../assets/JetBrainsMono-Regular.ttf")),
    );
    fonts.font_data.insert(
        "dejavu-mono".to_owned(),
        FontData::from_static(include_bytes!("../assets/DejaVuSansMono.ttf")),
    );
    {
        let fam = fonts.families.entry(FontFamily::Proportional).or_default();
        fam.insert(0, "inter".to_owned());
        fam.insert(1, "dejavu".to_owned());
    }
    {
        let fam = fonts.families.entry(FontFamily::Monospace).or_default();
        fam.insert(0, "jetbrains-mono".to_owned());
        fam.insert(1, "dejavu-mono".to_owned());
    }
    fonts.families.insert(
        FontFamily::Name("InterSemiBold".into()),
        vec!["inter-semibold".to_owned(), "inter-bold".to_owned(), "dejavu-bold".to_owned()],
    );
    fonts.families.insert(
        FontFamily::Name("DejaVuBold".into()),
        vec!["inter-bold".to_owned(), "dejavu-bold".to_owned()],
    );
    ctx.set_fonts(fonts);
}

// ------------------------------------------------------------------ dinh dang
fn byte_of_char(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(s.len())
}

fn replace_char_range(text: &mut String, a: usize, b: usize, with: &str) {
    let ba = byte_of_char(text, a);
    let bb = byte_of_char(text, b);
    text.replace_range(ba..bb, with);
}

fn sel_text(doc: &Doc) -> (usize, usize) {
    let n = doc.text.chars().count();
    let a = doc.sel_start.min(n);
    let b = doc.sel_end.min(n);
    if a <= b { (a, b) } else { (b, a) }
}

fn slice_chars(text: &str, a: usize, b: usize) -> String {
    text.chars().skip(a).take(b.saturating_sub(a)).collect()
}

/// Boc (hoac go boc neu da boc) vung chon bang cap dau before/after.
fn fmt_wrap(doc: &mut Doc, before: &str, after: &str) -> (usize, usize) {
    let (a, b) = sel_text(doc);
    let bl = before.chars().count();
    let al = after.chars().count();
    if b > a {
        if a >= bl
            && slice_chars(&doc.text, a - bl, a) == before
            && slice_chars(&doc.text, b, b + al) == after
        {
            replace_char_range(&mut doc.text, b, b + al, "");
            replace_char_range(&mut doc.text, a - bl, a, "");
            return (a - bl, b - bl);
        }
        let inner = slice_chars(&doc.text, a, b);
        if inner.starts_with(before) && inner.ends_with(after) && inner.chars().count() >= bl + al {
            replace_char_range(&mut doc.text, b - al, b, "");
            replace_char_range(&mut doc.text, a, a + bl, "");
            return (a, b - bl - al);
        }
        replace_char_range(&mut doc.text, b, b, after);
        replace_char_range(&mut doc.text, a, a, before);
        (a + bl, b + bl)
    } else {
        replace_char_range(&mut doc.text, a, a, &format!("{before}{after}"));
        (a + bl, a + bl)
    }
}

#[derive(Clone, Copy)]
enum LineMode {
    Heading(u8),
    Quote,
    Bullet,
    Numbered,
    Task,
}

fn strip_heading_prefix(line: &str) -> String {
    let s = line.trim_start_matches('#');
    let s = s.strip_prefix(' ').unwrap_or(s);
    s.to_string()
}

/// Ap dung kieu dong (tieu de / trich dan / danh sach) cho cac dong trong vung chon.
fn fmt_lines(doc: &mut Doc, mode: LineMode) -> (usize, usize) {
    let (a, b) = sel_text(doc);
    let mut lines: Vec<String> = doc.text.split('\n').map(|s| s.to_string()).collect();
    // vi tri bat dau (char) cua tung dong
    let mut starts: Vec<usize> = Vec::with_capacity(lines.len());
    let mut off = 0usize;
    for ln in &lines {
        starts.push(off);
        off += ln.chars().count() + 1;
    }
    let mut li_a = 0usize;
    for (i, &st) in starts.iter().enumerate() {
        if st <= a {
            li_a = i;
        }
    }
    let mut li_b = li_a;
    for (i, &st) in starts.iter().enumerate() {
        if st < b || (b == a && st <= b) {
            li_b = i.max(li_a);
        }
    }
    if b > a {
        // neu vung chon ket thuc dung dau dong thi khong tinh dong do
        if let Some(&st) = starts.get(li_b) {
            if st == b && li_b > li_a {
                li_b -= 1;
            }
        }
    }
    let all_have = |pred: &dyn Fn(&str) -> bool| -> bool {
        lines[li_a..=li_b].iter().filter(|l| !l.trim().is_empty()).all(|l| pred(l))
    };
    match mode {
        LineMode::Heading(n) => {
            let hashes = "#".repeat(n as usize);
            for ln in &mut lines[li_a..=li_b] {
                let body = strip_heading_prefix(ln);
                *ln = format!("{hashes} {body}");
            }
        }
        LineMode::Quote => {
            if all_have(&|l: &str| l.starts_with("> ")) {
                for ln in &mut lines[li_a..=li_b] {
                    if let Some(s) = ln.strip_prefix("> ") {
                        *ln = s.to_string();
                    }
                }
            } else {
                for ln in &mut lines[li_a..=li_b] {
                    if !ln.trim().is_empty() {
                        *ln = format!("> {ln}");
                    }
                }
            }
        }
        LineMode::Bullet => {
            if all_have(&|l: &str| l.starts_with("- ")) {
                for ln in &mut lines[li_a..=li_b] {
                    if let Some(s) = ln.strip_prefix("- ") {
                        *ln = s.to_string();
                    }
                }
            } else {
                for ln in &mut lines[li_a..=li_b] {
                    if !ln.trim().is_empty() && !ln.starts_with("- ") {
                        *ln = format!("- {ln}");
                    }
                }
            }
        }
        LineMode::Task => {
            let is_task = |l: &str| l.starts_with("- [ ] ") || l.starts_with("- [x] ");
            if all_have(&is_task) {
                for ln in &mut lines[li_a..=li_b] {
                    if let Some(s) = ln.strip_prefix("- [ ] ").or_else(|| ln.strip_prefix("- [x] ")) {
                        *ln = s.to_string();
                    }
                }
            } else {
                for ln in &mut lines[li_a..=li_b] {
                    if !ln.trim().is_empty() && !is_task(ln) {
                        *ln = format!("- [ ] {ln}");
                    }
                }
            }
        }
        LineMode::Numbered => {
            let num_prefix = |l: &str| -> Option<usize> {
                let digits: usize = l.chars().take_while(|c| c.is_ascii_digit()).count();
                if digits > 0 && l.chars().nth(digits) == Some('.') && l.chars().nth(digits + 1) == Some(' ') {
                    Some(digits + 2)
                } else {
                    None
                }
            };
            if all_have(&|l: &str| num_prefix(l).is_some()) {
                for ln in &mut lines[li_a..=li_b] {
                    if let Some(k) = num_prefix(ln) {
                        *ln = ln.chars().skip(k).collect();
                    }
                }
            } else {
                let mut k = 0usize;
                for ln in &mut lines[li_a..=li_b] {
                    if !ln.trim().is_empty() {
                        k += 1;
                        if num_prefix(ln).is_none() {
                            *ln = format!("{k}. {ln}");
                        }
                    }
                }
            }
        }
    }
    doc.text = lines.join("\n");
    // vung chon moi: tu dau dong dau toi cuoi dong cuoi
    let mut starts2: Vec<usize> = Vec::with_capacity(lines.len());
    let mut off2 = 0usize;
    for ln in &lines {
        starts2.push(off2);
        off2 += ln.chars().count() + 1;
    }
    let na = starts2[li_a];
    let nb = starts2[li_b] + lines[li_b].chars().count();
    (na, nb)
}

#[derive(Clone, Copy, PartialEq)]
enum FmtKind {
    Bold,
    Italic,
    Strike,
    Code,
    H1,
    H2,
    H3,
    Quote,
    Bullet,
    Numbered,
    Task,
    Link,
    Image,
    Table,
    Fence,
    Hr,
}

/// Ap dung mot thao tac dinh dang len tai lieu; tra ve vung chon moi.
fn fmt_apply_doc(doc: &mut Doc, kind: FmtKind) -> (usize, usize) {
    match kind {
        FmtKind::Bold => fmt_wrap(doc, "**", "**"),
        FmtKind::Italic => fmt_wrap(doc, "*", "*"),
        FmtKind::Strike => fmt_wrap(doc, "~~", "~~"),
        FmtKind::Code => fmt_wrap(doc, "`", "`"),
        FmtKind::H1 => fmt_lines(doc, LineMode::Heading(1)),
        FmtKind::H2 => fmt_lines(doc, LineMode::Heading(2)),
        FmtKind::H3 => fmt_lines(doc, LineMode::Heading(3)),
        FmtKind::Quote => fmt_lines(doc, LineMode::Quote),
        FmtKind::Bullet => fmt_lines(doc, LineMode::Bullet),
        FmtKind::Numbered => fmt_lines(doc, LineMode::Numbered),
        FmtKind::Task => fmt_lines(doc, LineMode::Task),
        FmtKind::Link => {
            let (a, b) = sel_text(doc);
            if a == b {
                replace_char_range(&mut doc.text, a, a, "[](https://)");
                (a + 1, a + 1)
            } else {
                let sel = slice_chars(&doc.text, a, b);
                replace_char_range(&mut doc.text, a, b, &format!("[{sel}](https://)"));
                (b + 3, b + 11)
            }
        }
        FmtKind::Image => {
            let (a, b) = sel_text(doc);
            if a == b {
                replace_char_range(&mut doc.text, a, a, "![](duong-dan)");
                (a + 4, a + 13)
            } else {
                let sel = slice_chars(&doc.text, a, b);
                replace_char_range(&mut doc.text, a, b, &format!("![{sel}](duong-dan)"));
                (b + 3, b + 12)
            }
        }
        FmtKind::Table => {
            let (a, b) = sel_text(doc);
            let tpl = "\n| Cột 1 | Cột 2 | Cột 3 |\n| --- | --- | --- |\n|  |  |  |\n";
            replace_char_range(&mut doc.text, a, b, tpl);
            (a + 3, a + 8)
        }
        FmtKind::Fence => {
            let (a, b) = sel_text(doc);
            if a == b {
                replace_char_range(&mut doc.text, a, a, "```\n\n```");
                (a + 4, a + 4)
            } else {
                let sel = slice_chars(&doc.text, a, b);
                replace_char_range(&mut doc.text, a, b, &format!("```\n{sel}\n```"));
                (a + 4, b + 4)
            }
        }
        FmtKind::Hr => {
            let (a, b) = sel_text(doc);
            replace_char_range(&mut doc.text, a, b, "\n\n---\n\n");
            let na = a + 5;
            (na, na)
        }
    }
}

#[derive(Clone, Copy)]
enum MenuAct {
    Cut,
    Copy,
    Paste,
    SelectAll,
    Fmt(FmtKind),
}

/// Tim kiem khong phan biet hoa thuong (theo ky tu), tra ve vi tri char bat dau.
fn find_ci_from(text: &str, needle: &str, from_char: usize) -> Option<usize> {
    let hay: Vec<char> = text.chars().map(|c| c.to_lowercase().next().unwrap_or(c)).collect();
    let nee: Vec<char> = needle.chars().map(|c| c.to_lowercase().next().unwrap_or(c)).collect();
    if nee.is_empty() || hay.len() < nee.len() || from_char > hay.len() - nee.len() {
        return None;
    }
    (from_char..=hay.len() - nee.len()).find(|&i| hay[i..i + nee.len()] == nee[..])
}

// ------------------------------------------------------------------ UI impl
impl App {
    fn active_doc(&mut self) -> Option<&mut Doc> {
        self.docs.get_mut(self.active)
    }

    fn goto_char(&mut self, ctx: &egui::Context, doc_id: u64, from: usize, to: usize, text: &str) {
        let id = egui::Id::new(("md-editor", doc_id));
        let mut state = egui::TextEdit::load_state(ctx, id).unwrap_or_default();
        state
            .cursor
            .set_char_range(Some(CCursorRange::two(CCursor::new(from), CCursor::new(to))));
        state.store(ctx, id);
        let _ = text;
    }

    fn do_find_next(&mut self, ctx: &egui::Context) {
        if self.find_text.is_empty() {
            return;
        }
        let Some(doc) = self.docs.get(self.active) else { return };
        let doc_id = doc.id;
        let chars: Vec<char> = doc.text.chars().collect();
        let needle: Vec<char> = self.find_text.chars().collect();
        if needle.is_empty() || chars.len() < needle.len() {
            return;
        }
        let lower = |c: &char| c.to_lowercase().next().unwrap_or(*c);
        let hay: Vec<char> = chars.iter().map(|c| lower(c)).collect();
        let nee: Vec<char> = needle.iter().map(|c| lower(c)).collect();
        let start = (doc.cursor_char + 1).min(hay.len());
        let find_from = |from: usize| -> Option<usize> {
            if hay.len() < nee.len() || from > hay.len() - nee.len() {
                return None;
            }
            (from..=hay.len() - nee.len()).find(|&i| hay[i..i + nee.len()] == nee[..])
        };
        let found = find_from(start).or_else(|| find_from(0));
        match found {
            Some(pos) => {
                let text = doc.text.clone();
                self.goto_char(ctx, doc_id, pos, pos + nee.len(), &text);
                if let Some(d) = self.docs.get_mut(self.active) {
                    d.cursor_char = pos;
                }
            }
            None => {
                self.status = format!("Không tìm thấy: {}", self.find_text);
            }
        }
    }

    fn check_external_changes(&mut self) {
        if self.last_mtime_check.elapsed() < Duration::from_millis(1200) {
            return;
        }
        self.last_mtime_check = Instant::now();
        for i in 0..self.docs.len() {
            let (path, old_mtime) = match &self.docs[i].path {
                Some(p) => (p.clone(), self.docs[i].mtime),
                None => continue,
            };
            let new_mtime = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
            if new_mtime != old_mtime && new_mtime.is_some() {
                if self.docs[i].dirty() {
                    self.status = format!(
                        "Cảnh báo: {} vừa thay đổi trên đĩa (bản trong app chưa lưu)",
                        path.display()
                    );
                    self.docs[i].mtime = new_mtime;
                } else if let Ok(mut fresh) = Doc::load_from(&path) {
                    fresh.id = self.docs[i].id;
                    fresh.refresh_ast();
                    self.docs[i] = fresh;
                    self.status = format!("Đã tải lại từ đĩa: {}", path.display());
                }
            }
        }
    }

    fn close_tab_now(&mut self, idx: usize) {
        if idx < self.docs.len() {
            if let Some(p) = self.docs[idx].path.clone() {
                self.closed.retain(|x| x != &p);
                self.closed.push(p);
                if self.closed.len() > 20 {
                    self.closed.remove(0);
                }
            }
            self.docs.remove(idx);
            if self.docs.is_empty() {
                self.new_doc();
            } else if self.active >= self.docs.len() {
                self.active = self.docs.len() - 1;
            } else if idx < self.active {
                self.active -= 1;
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // ---- IPC: file mo tu instance khac / Explorer ----
        let mut ipc_paths: Vec<PathBuf> = Vec::new();
        if let Some(rx) = &self.ipc_rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    IpcMsg::Open(paths) => ipc_paths.extend(paths),
                }
            }
        }
        if !ipc_paths.is_empty() || EGCTX.get().is_some() && false {
            // (focus da duoc gui kem repaint tu thread IPC)
        }
        if !ipc_paths.is_empty() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            for p in &ipc_paths {
                let p = p.clone();
                self.open_path(&p);
            }
        }

        // ---- dong cua so ----
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested && !self.allow_exit {
            if self.docs.iter().any(|d| d.dirty()) {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.close_req = Some(CloseReq::Exit);
            } else {
                self.persist_session();
            }
        }

        // ---- keo tha file vao ----
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        for p in dropped {
            self.open_path(&p);
        }

        // Khi bo cuc thay doi (doi che do xem, an/hien muc luc, doi kich thuoc
        // cua so) -> ep ve lai ngay vai khung hinh, tranh vung den con sot lai
        // tren Windows (khu vuc panel cu khong con duoc to nen nua).
        {
            let vc: u8 = match self.view { ViewMode::Edit => 0, ViewMode::Preview => 1, ViewMode::Split => 2 };
            let cur = (vc, self.show_toc, ctx.input(|i| i.screen_rect()));
            let changed = cur != self.prev_layout;
            if changed {
                self.prev_layout = cur;
                ctx.request_repaint();
                ctx.request_repaint_after(std::time::Duration::from_millis(60));
            }
        }

        self.check_external_changes();
        self.tick_autosave();
        self.handle_extra_keys(ctx);
        if let Some(doc) = self.active_doc() {
            doc.refresh_ast();
        }

        // ---- phim tat ----
        let input = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::N) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::O) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::S) && i.modifiers.ctrl && !i.modifiers.shift,
                i.key_pressed(egui::Key::S) && i.modifiers.ctrl && i.modifiers.shift,
                i.key_pressed(egui::Key::W) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::F) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Num1) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Num2) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Num3) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Enter) && self.find_open,
                i.key_pressed(egui::Key::B) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::I) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::K) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::H) && i.modifiers.ctrl,
                (i.key_pressed(egui::Key::Equals) || i.key_pressed(egui::Key::Plus)) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Minus) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Num0) && i.modifiers.ctrl,
            )
        });
        let (k_new, k_open, k_save, k_saveas, k_close, k_find, k_v1, k_v2, k_v3, k_enter,
             k_bold, k_italic, k_link, k_replace, k_zin, k_zout, k_z100) = input;
        if k_new {
            self.new_doc();
        }
        if k_open {
            if let Some(files) = rfd::FileDialog::new()
                .set_title("Mở file (chọn được nhiều file)")
                .add_filter("Markdown", &["md", "markdown", "mdown", "mkd", "txt"])
                .pick_files()
            {
                for f in files {
                    self.open_path(&f);
                }
            }
        }
        if k_save {
            self.save_doc(self.active, false);
        }
        if k_saveas {
            self.save_doc(self.active, true);
        }
        if k_close {
            let idx = self.active;
            if self.docs.get(idx).map(|d| d.dirty()).unwrap_or(false) {
                self.close_req = Some(CloseReq::Tab(idx));
            } else {
                self.close_tab_now(idx);
            }
        }
        if k_find {
            self.find_open = true;
        }
        if k_v1 {
            self.view = ViewMode::Edit;
        }
        if k_v2 {
            self.view = ViewMode::Preview;
        }
        if k_v3 {
            self.view = ViewMode::Split;
        }
        if k_enter && self.find_open && !self.find_text.is_empty() {
            self.do_find_next(ctx);
        }
        if k_bold {
            self.fmt(FmtKind::Bold);
        }
        if k_italic {
            self.fmt(FmtKind::Italic);
        }
        if k_link {
            self.fmt(FmtKind::Link);
        }
        if k_replace {
            self.find_open = true;
        }
        if k_zin {
            let z = self.zoom + 0.1;
            self.set_zoom(ctx, z);
        }
        if k_zout {
            let z = self.zoom - 0.1;
            self.set_zoom(ctx, z);
        }
        if k_z100 {
            self.set_zoom(ctx, 1.0);
        }

        // ---- ribbon (File / Dinh dang / Chen / Xem / Xuat / Cong cu / Tro giup) + thanh tab ----
        self.ui_ribbon(ctx);
        self.ui_tabbar(ctx);

        // ---- thanh trang thai ----
        egui::TopBottomPanel::bottom("statusbar")
            .frame(
                egui::Frame::default()
                    .fill(chrome_fill(self.dark))
                    .inner_margin(egui::Margin::symmetric(10.0, 4.0)),
            )
            .show(ctx, |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if let Some(doc) = self.docs.get(self.active) {
                    let path_txt = doc
                        .path
                        .as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "(chưa lưu vào file)".into());
                    let words = doc.text.split_whitespace().count();
                    let chars = doc.text.chars().count();
                    let before: String = doc.text.chars().take(doc.cursor_char).collect();
                    let line = before.matches('\n').count() + 1;
                    let col = before.rsplit('\n').next().map(|s| s.chars().count()).unwrap_or(0) + 1;
                    ui.label(format!("{} | Dòng {}, Cột {} | {} từ · {} ký tự",
                                     doc.enc.label(), line, col, words, chars));
                    if !self.status.is_empty() {
                        vsep(ui);
                        ui.add(egui::Label::new(RichText::new(&self.status).color(ui.visuals().text_color())).truncate());
                        vsep(ui);
                    }
                    ui.add(egui::Label::new(path_txt).truncate());
                } else if !self.status.is_empty() {
                    ui.label(&self.status);
                }
            });
        });

        // ---- thanh tim kiem ----
        if self.find_open {
            egui::TopBottomPanel::bottom("findbar")
                .frame(
                    egui::Frame::default()
                        .fill(chrome_fill(self.dark))
                        .inner_margin(egui::Margin::symmetric(10.0, 5.0)),
                )
                .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Tìm:");
                    let resp = ui.add(egui::TextEdit::singleline(&mut self.find_text).desired_width(220.0));
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.do_find_next(ctx);
                    }
                    if ui.button("Tiếp").clicked() {
                        self.do_find_next(ctx);
                    }
                    if ui.button("Đóng").clicked() {
                        self.find_open = false;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Thay:");
                    ui.add(egui::TextEdit::singleline(&mut self.replace_text).desired_width(220.0));
                    if ui.button("Thay thế").clicked() {
                        self.replace_next(ctx);
                    }
                    if ui.button("Thay tất cả").clicked() {
                        self.replace_all();
                    }
                });
            });
        }

        // ---- muc luc ----
        if self.show_toc {
            let sw = ctx.screen_rect().width();
            egui::SidePanel::left("toc")
                .resizable(true)
                .default_width((sw * 0.25).clamp(130.0, 235.0))
                .width_range(110.0..=(sw * 0.4).clamp(130.0, 420.0))
                .frame(
                    egui::Frame::default()
                        .fill(chrome_fill(self.dark))
                        .inner_margin(egui::Margin::same(10.0)),
                )
                .show(ctx, |ui| {
                    ui.label(
                        RichText::new("MỤC LỤC")
                            .font(FontId::new(11.0, egui::FontFamily::Name("InterSemiBold".into())))
                            .color(ui.visuals().weak_text_color()),
                    );
                    ui.add_space(6.0);
                    let mut goto: Option<usize> = None;
                    if let Some(doc) = self.docs.get(self.active) {
                        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                            if doc.toc.is_empty() {
                                ui.label(RichText::new("(Chưa có tiêu đề)").color(ui.visuals().weak_text_color()));
                            }
                            for e in &doc.toc {
                                let indent = (e.level.saturating_sub(1)) as f32 * 14.0;
                                ui.horizontal(|ui| {
                                    ui.add_space(indent);
                                    let mut txt = RichText::new(&e.title).size(13.5);
                                    if e.level == 1 {
                                        txt = txt.font(FontId::new(13.5, egui::FontFamily::Name("InterSemiBold".into())));
                                    }
                                    txt = txt.color(ui.visuals().text_color());
                                    // cat bot tieu de dai thay vi tran ra ngoai panel
                                    let r = ui.add(egui::Label::new(txt).truncate().sense(egui::Sense::click()));
                                    let r = r.on_hover_text(&e.title).on_hover_cursor(egui::CursorIcon::PointingHand);
                                    if r.hovered() {
                                        ui.painter().rect_stroke(
                                            r.rect.expand(2.0),
                                            4.0,
                                            egui::Stroke::new(1.0, ui.visuals().widgets.hovered.bg_stroke.color),
                                        );
                                    }
                                    if r.clicked() {
                                        goto = Some(e.char_offset);
                                    }
                                });
                            }
                        });
                    }
                    if let Some(off) = goto {
                        if let Some(doc) = self.docs.get(self.active) {
                            let (doc_id, text) = (doc.id, doc.text.clone());
                            self.goto_char(ctx, doc_id, off, off, &text);
                            self.toc_goto = Some((doc_id, off));
                            ctx.data_mut(|d| d.insert_temp(egui::Id::new("preview-scroll-target"), off));
                            ctx.request_repaint();
                        }
                    }
                });
        }

        // ---- vung chinh: editor / preview ----
        let mut preview_actions: Vec<PreviewAction> = Vec::new();
        let show_editor = matches!(self.view, ViewMode::Edit | ViewMode::Split);
        let show_preview = matches!(self.view, ViewMode::Preview | ViewMode::Split);

        let editor_ui = |app: &mut App, ui: &mut egui::Ui| {
            let Some(active_id) = app.docs.get(app.active).map(|d| d.id) else { return };
            // Co chu de layouter biet co chu hien tai (zoom noi dung)
            ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("md-editor-font"), 13.5 * app.zoom));
            // Lenh nhay (TOC / tim kiem) cho tab dang mo, neu co
            let mut pending_range: Option<(usize, usize)> = None;
            if let Some((id, from, to)) = app.find_goto {
                if id == active_id {
                    pending_range = Some((from, to));
                    app.find_goto = None;
                }
            }
            if let Some((id, off)) = app.toc_goto {
                if id == active_id {
                    pending_range = Some((off, off));
                    app.toc_goto = None;
                }
            }
            let mut scroll_y: Option<f32> = None;
            if let Some((from, to)) = pending_range {
                if let Some(doc) = app.docs.get(app.active) {
                    let text = doc.text.clone();
                    let line_no = text.chars().take(from).filter(|&c| c == '\n').count();
                    scroll_y = Some((line_no as f32 * 18.0 * app.zoom - 120.0).max(0.0));
                    app.goto_char(ui.ctx(), active_id, from, to, &text);
                }
            }
            let ez = app.zoom;
            let mut menu_goto: Option<(usize, usize)> = None;
            {
                let Some(doc) = app.docs.get_mut(app.active) else { return };
                let saved_sel = (doc.sel_start, doc.sel_end);
                let total_chars = doc.text.chars().count();
                let line_count = doc.text.matches('\n').count() + 1;
                let cur_line = doc
                    .text
                    .chars()
                    .take(doc.cursor_char.min(total_chars))
                    .filter(|&c| c == '\n')
                    .count()
                    + 1;
                let view_h = ui.available_height().max(60.0);
                let view_w = ui.available_width();
                let mono = FontId::monospace(13.5 * ez);
                let probe = ui.painter().layout_no_wrap("0".to_string(), mono.clone(), Color32::WHITE);
                let row_h = probe.size().y.max(1.0);
                let digits = line_count.to_string().len().max(2);
                let gw = probe.size().x * digits as f32 + 16.0;
                let mut scroll = egui::ScrollArea::both()
                    .id_salt(("editor-scroll", active_id))
                    .max_height(view_h)
                    .auto_shrink([false, false]);
                if let Some(y) = scroll_y {
                    scroll = scroll.scroll_offset(egui::Vec2::new(0.0, y));
                }
                let mut gutter: Option<(Arc<egui::Galley>, egui::Pos2)> = None;
                ui.allocate_ui_with_layout(
                    egui::vec2(view_w, view_h),
                    egui::Layout::left_to_right(egui::Align::TOP),
                    |ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    // cot so dong: dung yen ben trai, KHONG cuon ngang theo noi dung
                    let (gre, _) = ui.allocate_exact_size(egui::vec2(gw, view_h), egui::Sense::hover());
                    ui.painter().rect_filled(gre, egui::Rounding::same(6.0), ui.visuals().faint_bg_color);
                    scroll.show(ui, |ui| {
                        // ---- khung soan thao ----
                        let sec_click = ui.input(|i| i.pointer.secondary_clicked());
                        let mut layouter_fn = editor_layouter;
                        // lap day chieu cao khung nhin: bam vao vung trong phia duoi van dat con tro
                        let rows = line_count.max((view_h / row_h).ceil() as usize + 1);
                        let te = egui::TextEdit::multiline(&mut doc.text)
                            .id(egui::Id::new(("md-editor", active_id)))
                            .font(FontId::monospace(13.5 * ez))
                            .layouter(&mut layouter_fn)
                            .frame(false)
                            .margin(egui::Margin::symmetric(6.0, 4.0))
                            .desired_rows(rows)
                            .desired_width((view_w - gw - 30.0).max(120.0))
                            .code_editor();
                        let out = te.show(ui);
                        if let Some(range) = out.cursor_range {
                            let a = range.primary.ccursor.index;
                            let b = range.secondary.ccursor.index;
                            doc.cursor_char = a;
                            doc.sel_start = a.min(b);
                            doc.sel_end = a.max(b);
                        }
                        // egui tu xoa vung chon khi bam chuot phai (de mo menu) ->
                        // khoi phuc lai vung chon truoc do: vung chon hien lai sau
                        // menu (qua find_goto) va cac muc menu tac dung dung vung ay
                        if sec_click && saved_sel.0 < saved_sel.1 {
                            let (a, b) = saved_sel;
                            doc.sel_start = a;
                            doc.sel_end = b;
                            doc.cursor_char = b;
                            menu_goto = Some((a, b));
                        }
                        // ---- menu chuot phai ----
                        let mut act: Option<MenuAct> = None;
                        out.response.context_menu(|ui| {
                            ui.set_min_width(190.0);
                            ui.with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            if ui.button("Cắt").clicked() { act = Some(MenuAct::Cut); ui.close_menu(); }
                            if ui.button("Sao chép").clicked() { act = Some(MenuAct::Copy); ui.close_menu(); }
                            if ui.button("Dán").clicked() { act = Some(MenuAct::Paste); ui.close_menu(); }
                            if ui.button("Chọn tất cả").clicked() { act = Some(MenuAct::SelectAll); ui.close_menu(); }
                            ui.separator();
                            if ui.button("Đậm").clicked() { act = Some(MenuAct::Fmt(FmtKind::Bold)); ui.close_menu(); }
                            if ui.button("Nghiêng").clicked() { act = Some(MenuAct::Fmt(FmtKind::Italic)); ui.close_menu(); }
                            if ui.button("Gạch ngang chữ").clicked() { act = Some(MenuAct::Fmt(FmtKind::Strike)); ui.close_menu(); }
                            if ui.button("Code trong dòng").clicked() { act = Some(MenuAct::Fmt(FmtKind::Code)); ui.close_menu(); }
                            if ui.button("Chèn liên kết").clicked() { act = Some(MenuAct::Fmt(FmtKind::Link)); ui.close_menu(); }
                            });
                        });
                        if let Some(act) = act {
                            match act {
                                MenuAct::Copy => {
                                    let (a, b) = sel_text(doc);
                                    if a < b {
                                        ui.ctx().copy_text(slice_chars(&doc.text, a, b));
                                    }
                                }
                                MenuAct::Cut => {
                                    let (a, b) = sel_text(doc);
                                    if a < b {
                                        ui.ctx().copy_text(slice_chars(&doc.text, a, b));
                                        replace_char_range(&mut doc.text, a, b, "");
                                        doc.sel_start = a;
                                        doc.sel_end = a;
                                        doc.cursor_char = a;
                                        menu_goto = Some((a, a));
                                    }
                                }
                                MenuAct::Paste => {
                                    if let Ok(mut cb) = arboard::Clipboard::new() {
                                        if let Ok(txt) = cb.get_text() {
                                            let (a, b) = sel_text(doc);
                                            let n = txt.chars().count();
                                            replace_char_range(&mut doc.text, a, b, &txt);
                                            doc.sel_start = a + n;
                                            doc.sel_end = a + n;
                                            doc.cursor_char = a + n;
                                            menu_goto = Some((a + n, a + n));
                                        }
                                    }
                                }
                                MenuAct::SelectAll => {
                                    let n = doc.text.chars().count();
                                    doc.sel_start = 0;
                                    doc.sel_end = n;
                                    doc.cursor_char = n;
                                    menu_goto = Some((0, n));
                                }
                                MenuAct::Fmt(kind) => {
                                    let (a, b) = fmt_apply_doc(doc, kind);
                                    doc.sel_start = a;
                                    doc.sel_end = b;
                                    doc.cursor_char = b;
                                    menu_goto = Some((a, b));
                                }
                            }
                        }
                        gutter = Some((out.galley.clone(), out.galley_pos));
                    });
                    // ---- ve so dong theo dung vi tri tung dong cua khung soan thao ----
                    if let Some((galley, gpos)) = gutter.take() {
                        let painter = ui.painter_at(gre);
                        let mut line_no = 1usize;
                        let mut prev_nl = true;
                        for row in galley.rows.iter() {
                            if !prev_nl {
                                // dong hien thi nay la phan noi tiep (khong xay ra khi tat wrap)
                                prev_nl = row.ends_with_newline;
                                continue;
                            }
                            let y = gpos.y + row.rect.center().y;
                            if y >= gre.top() - row_h && y <= gre.bottom() + row_h {
                                let col = if line_no == cur_line {
                                    ui.visuals().text_color()
                                } else {
                                    ui.visuals().weak_text_color()
                                };
                                painter.text(
                                    egui::pos2(gre.right() - 8.0, y),
                                    egui::Align2::RIGHT_CENTER,
                                    format!("{}", line_no),
                                    mono.clone(),
                                    col,
                                );
                            }
                            line_no += 1;
                            prev_nl = row.ends_with_newline;
                        }
                    }
                    });
            }
            if let Some((a, b)) = menu_goto {
                app.find_goto = Some((active_id, a, b));
            }
        };

        if show_editor && show_preview {
            let avail_w = ctx.available_rect().width();
            egui::SidePanel::left("editor-panel")
                .resizable(true)
                .default_width((avail_w * 0.5).clamp(240.0, 560.0))
                .width_range(200.0..=(avail_w - 220.0).max(200.0))
                .frame(
                    egui::Frame::default()
                        .fill(content_fill(self.dark))
                        .inner_margin(egui::Margin::same(8.0))
                        .stroke(egui::Stroke::new(1.0, border_color(self.dark))),
                )
                .show(ctx, |ui| {
                    editor_ui(self, ui);
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if show_editor && !show_preview {
                editor_ui(self, ui);
            } else if show_preview {
                let info = self.docs.get(self.active).map(|doc| {
                    (
                        doc.path
                            .as_ref()
                            .and_then(|p| p.parent().map(|x| x.to_path_buf()))
                            .unwrap_or_else(|| PathBuf::from(".")),
                        doc.ast.clone(),
                        doc.path.is_none() && doc.text.trim().is_empty(),
                    )
                });
                if let Some((doc_dir, ast, empty_start)) = info {
                    if empty_start {
                        self.ui_start_page(ui);
                        return;
                    }
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            // Cot noi dung o giua, rong toi da 880px nhu cac app doc hien dai
                            let avail = ui.available_width() - 4.0;
                            let w = (avail - 24.0).clamp(160.0, 880.0).min(avail.max(160.0));
                            let m = ((avail - w) / 2.0).max(0.0);
                            ui.add_space(10.0);
                            ui.horizontal_top(|ui| {
                                ui.add_space(m);
                                ui.vertical(|ui| {
                                    ui.set_width(w);
                                    let mut seed = 1u64;
                                    render_blocks(ui, &ast, &doc_dir, &mut preview_actions, &mut seed, &mut self.img_cache, self.zoom, false);
                                    ui.add_space(24.0);
                                });
                            });
                        });
                }
            }
        });

        for act in preview_actions {
            match act {
                PreviewAction::OpenUrl(u) => {
                    ctx.open_url(egui::OpenUrl { url: u, new_tab: true });
                }
                PreviewAction::OpenFile(p) => self.open_path(&p),
                PreviewAction::Anchor(a) => self.goto_anchor(ctx, &a),
            }
        }

        // ---- hop thoai (sua loi, kiem tra cong thuc, thong ke, tro giup) ----
        self.ui_dialogs(ctx);

        // ---- hop thoai xac nhan dong ----
        if let Some(req) = &self.close_req {
            let is_exit = matches!(req, CloseReq::Exit);
            let mut action: Option<&str> = None;
            egui::Window::new("Chưa lưu thay đổi")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(if is_exit {
                        "Có tài liệu chưa lưu. Bạn muốn làm gì?"
                    } else {
                        "Tài liệu này chưa lưu. Bạn muốn làm gì?"
                    });
                    ui.horizontal(|ui| {
                        if ui.button("Lưu").clicked() {
                            action = Some("save");
                        }
                        if ui.button(if is_exit { "Thoát không lưu" } else { "Đóng không lưu" }).clicked() {
                            action = Some("discard");
                        }
                        if ui.button("Hủy").clicked() {
                            action = Some("cancel");
                        }
                    });
                });
            match action {
                Some("save") => {
                    let req = self.close_req.take().unwrap();
                    match req {
                        CloseReq::Tab(i) => {
                            if self.save_doc(i, false) {
                                self.close_tab_now(i);
                            }
                        }
                        CloseReq::Exit => {
                            let dirty: Vec<usize> = (0..self.docs.len())
                                .filter(|&i| self.docs[i].dirty())
                                .collect();
                            let mut ok = true;
                            for i in dirty {
                                if !self.save_doc(i, false) {
                                    ok = false;
                                    break;
                                }
                            }
                            if ok {
                                self.allow_exit = true;
                                self.persist_session();
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }
                }
                Some("discard") => {
                    let req = self.close_req.take().unwrap();
                    match req {
                        CloseReq::Tab(i) => self.close_tab_now(i),
                        CloseReq::Exit => {
                            self.allow_exit = true;
                            self.persist_session();
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
                Some("cancel") => {
                    self.close_req = None;
                }
                _ => {}
            }
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.persist_session();
    }
}


// ---------------------------------------------------------------------- main
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let files: Vec<PathBuf> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .collect();

    // Single-instance: da co cua so dang chay -> day file sang do roi thoat
    if try_forward_to_running_instance(&files) {
        return;
    }
    let ipc_rx = start_ipc_server();

    let session = load_session();
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon-256.png"))
        .expect("icon png hop le");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_TITLE)
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([720.0, 480.0])
            .with_icon(Arc::new(icon)),
        ..Default::default()
    };
    let result = eframe::run_native(
        APP_TITLE,
        options,
        Box::new(move |cc| {
            let mut app = App::new(cc, files.clone(), &session);
            app.ipc_rx = ipc_rx;
            Ok(Box::new(app))
        }),
    );
    if let Err(e) = result {
        eprintln!("Markdown Tabs lỗi khởi động: {e}");
    }
}

#[cfg(test)]
mod docx_tests {
    use super::*;

    /// Tao file mau day du cac thanh phan -> /tmp/mdt-test.docx (de kiem bang validator schema ISO 29500)
    #[test]
    fn build_docx_sample() {
        let md = "# Tiêu đề lớn\n\nĐoạn có **đậm**, *nghiêng*, ~~gạch~~ và `code` cùng [liên kết](https://example.com/tiếng việt?a=1&b=2) và \u{1}ký tự lạ\u{B}.\n\n## Mục nhỏ\n\n- việc một\n  - con a\n    - cháu\n  - con b\n- việc hai\n\n1. bước một\n   1. bước nhỏ\n2. bước hai\n\n- [x] đã xong\n- [ ] chưa xong\n\n> Trích dẫn quan trọng\n\n```rust\nlet x = 1;\n\n\tlet y = 2;\n```\n\n| Cột A | Cột B | C |\n|---|---|---|\n| 1 | $x^2$ | |\n| 3 | **4** |\n\n![Logo](icon-256.png)\n\nCông thức dòng $\\frac{a}{b}+\\sqrt[3]{x_i}-\\sum_{i=1}^{n} i$ rồi:\n\n$$x = \\frac{-b \\pm \\sqrt{b^2-4ac}}{2a}$$\n\n$$\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix} = \\int_0^1 \\lim_{x\\to 0} \\vec{F}\\,dx \\left( \\frac{1}{2} \\right)$$\n\n---\n\n<div>html</div>\n";
        let ast = parse_markdown(md);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let bytes = docx::build(&ast, &dir, "Mau", "0.5.0").expect("build docx");
        assert!(bytes.len() > 3000, "docx qua nho: {}", bytes.len());
        let out = std::env::var("MDT_OUT").unwrap_or_else(|_| "/tmp/mdt-test.docx".into());
        std::fs::write(out, &bytes).unwrap();
        let mut za = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let names: Vec<String> = za.file_names().map(|s| s.to_string()).collect();
        for need in ["[Content_Types].xml", "word/document.xml", "word/numbering.xml", "word/settings.xml", "docProps/core.xml", "word/media/image1.png"] {
            assert!(names.iter().any(|n| n == need), "thieu {need}: {names:?}");
        }
        use std::io::Read as _;
        let mut doc_xml = String::new();
        za.by_name("word/document.xml").unwrap().read_to_string(&mut doc_xml).unwrap();
        assert!(doc_xml.contains("<w:numId "), "numPr phai dung numId");
        assert!(!doc_xml.contains("<w:num "), "khong duoc con <w:num>");
        assert!(doc_xml.contains("<m:oMathPara>"));
        assert!(!doc_xml.contains('\u{1}'));
    }
}
