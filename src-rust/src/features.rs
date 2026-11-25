//! Giao dien ribbon (File / Dinh dang / Chen / Xem / Xuat / Cong cu / Tro giup),
//! tep gan day, hop thoai sua loi, kiem tra cong thuc, thong ke, tro giup...

use super::*;
use crate::{docx, fixer, icons, math};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RibbonTab {
    File,
    Format,
    Insert,
    View,
    Export,
    Tools,
    Help,
}

impl RibbonTab {
    pub const ALL: [(RibbonTab, &'static str); 7] = [
        (RibbonTab::File, "File"),
        (RibbonTab::Format, "Định dạng"),
        (RibbonTab::Insert, "Chèn"),
        (RibbonTab::View, "Xem"),
        (RibbonTab::Export, "Xuất"),
        (RibbonTab::Tools, "Công cụ"),
        (RibbonTab::Help, "Trợ giúp"),
    ];
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Dialog {
    Fixer,
    Formulas,
    Stats,
    Shortcuts,
    Syntax,
    About,
}

#[derive(Clone, Copy)]
enum CaseMode {
    Upper,
    Lower,
    Title,
}

const MAX_RECENT: usize = 15;

fn short_name(p: &str, max: usize) -> String {
    let name = Path::new(p).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_else(|| p.to_string());
    if name.chars().count() > max {
        format!("{}…", name.chars().take(max - 1).collect::<String>())
    } else {
        name
    }
}

// ---------------------------------------------------------------- nut bieu tuong
/// Nut bieu tuong 32x32: chi co hinh, ten hien trong tooltip khi re chuot.
/// `icon` bat dau bang "text:" thi ve chu thay vi hinh (vi du "text:Aa", "text:100%").
pub fn ibtn_w(ui: &mut egui::Ui, icon: &str, tip: &str, hue: f32, enabled: bool, selected: bool, width: f32) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let sense = if enabled { egui::Sense::click() } else { egui::Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(width, 32.0), sense);
    if ui.is_rect_visible(rect) {
        let ac = accent(dark);
        let hovered = resp.hovered() && enabled;
        let bg = if selected {
            accent_soft(dark)
        } else if hovered {
            mix(chrome_fill(dark), ac, if dark { 0.20 } else { 0.11 })
        } else {
            Color32::TRANSPARENT
        };
        let painter = ui.painter();
        if bg != Color32::TRANSPARENT {
            painter.rect_filled(rect, egui::Rounding::same(8.0), bg);
        }
        if selected {
            painter.rect_stroke(rect, egui::Rounding::same(8.0), egui::Stroke::new(1.0, mix(bg, ac, 0.55)));
        }
        let mut col = hue_color(hue, dark);
        if !enabled {
            col = col.gamma_multiply(0.35);
        }
        if let Some(t) = icon.strip_prefix("text:") {
            painter.text(rect.center(), egui::Align2::CENTER_CENTER, t, FontId::new(13.0, egui::FontFamily::Name("InterSemiBold".into())), col);
        } else {
            icons::paint(painter, egui::Rect::from_center_size(rect.center(), egui::vec2(19.0, 19.0)), icon, col);
        }
    }
    resp.on_hover_text(tip)
}

pub fn ibtn(ui: &mut egui::Ui, icon: &str, tip: &str, hue: f32, enabled: bool, selected: bool) -> egui::Response {
    ibtn_w(ui, icon, tip, hue, enabled, selected, 32.0)
}

fn icon_menu_impl(ui: &mut egui::Ui, icon: &str, tip: &str, hue: f32, close: egui::PopupCloseBehavior, add: impl FnOnce(&mut egui::Ui)) {
    let id = ui.make_persistent_id(("icmenu", tip));
    let open = ui.memory(|m| m.is_popup_open(id));
    let r = ibtn_w(ui, icon, tip, hue, true, open, 38.0);
    let chev_rect = egui::Rect::from_center_size(egui::pos2(r.rect.right() - 6.5, r.rect.center().y + 1.0), egui::vec2(8.0, 8.0));
    icons::paint(ui.painter(), chev_rect, "chev", ui.visuals().weak_text_color());
    if r.clicked() {
        ui.memory_mut(|m| m.toggle_popup(id));
    }
    egui::popup_below_widget(ui, id, &r, close, |ui| {
        ui.set_min_width(180.0);
        add(ui)
    });
}

/// Nut bieu tuong mo danh sach lua chon; bam vao muc la dong lai.
pub fn icon_menu(ui: &mut egui::Ui, icon: &str, tip: &str, hue: f32, add: impl FnOnce(&mut egui::Ui)) {
    icon_menu_impl(ui, icon, tip, hue, egui::PopupCloseBehavior::CloseOnClick, add)
}

/// Nhu `icon_menu` nhung giu mo cho den khi bam ra ngoai (bang ky hieu).
pub fn icon_menu_keep(ui: &mut egui::Ui, icon: &str, tip: &str, hue: f32, add: impl FnOnce(&mut egui::Ui)) {
    icon_menu_impl(ui, icon, tip, hue, egui::PopupCloseBehavior::CloseOnClickOutside, add)
}

/// Nut trong thanh chon che do xem: hinh + chu (chu bo khi cua so hep).
fn seg_btn(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool, tip: &str) -> (bool, egui::Rect) {
    let dark = ui.visuals().dark_mode;
    let galley = ui.painter().layout_no_wrap(label.to_string(), FontId::proportional(13.5), Color32::PLACEHOLDER);
    let text_w = if label.is_empty() { 0.0 } else { galley.size().x + 6.0 };
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(10.0 + 16.0 + text_w + 10.0, 28.0), egui::Sense::click());
    let ac = accent(dark);
    let fg = if selected { if dark { rgb(10, 14, 22) } else { Color32::WHITE } } else { ui.visuals().text_color() };
    let p = ui.painter();
    if selected {
        p.rect_filled(rect, egui::Rounding::same(8.0), ac);
    } else if resp.hovered() {
        p.rect_filled(rect, egui::Rounding::same(8.0), accent_soft(dark));
    }
    icons::paint(p, egui::Rect::from_center_size(egui::pos2(rect.left() + 10.0 + 8.0, rect.center().y), egui::vec2(16.0, 16.0)), icon, fg);
    if !label.is_empty() {
        p.galley(egui::pos2(rect.left() + 10.0 + 16.0 + 6.0, rect.center().y - galley.size().y / 2.0), galley, fg);
    }
    (resp.on_hover_text(tip).clicked(), rect)
}

pub fn slugify(title: &str) -> String {
    let mut out = String::new();
    for c in title.trim().chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if c.is_whitespace() || c == '-' {
            out.push('-');
        }
    }
    out
}

fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() + 0 && i + 2 <= b.len() - 1 + 0 {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn ast_to_text(blocks: &[Block], out: &mut String, depth: usize) {
    let spans_txt = |sp: &[Span]| -> String { sp.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join("") };
    for b in blocks {
        match b {
            Block::Paragraph(sp) => {
                out.push_str(&spans_txt(sp));
                out.push_str("\n\n");
            }
            Block::Heading(_, sp, _) => {
                out.push_str(&spans_txt(sp));
                out.push_str("\n\n");
            }
            Block::Code(_, t) => {
                out.push_str(t.trim_end_matches('\n'));
                out.push_str("\n\n");
            }
            Block::List { ordered, items } => {
                for (i, it) in items.iter().enumerate() {
                    let mut inner = String::new();
                    ast_to_text(it, &mut inner, depth + 1);
                    let inner = inner.trim_end();
                    let mark = if *ordered { format!("{}. ", i + 1) } else { "- ".to_string() };
                    out.push_str(&"  ".repeat(depth));
                    out.push_str(&mark);
                    out.push_str(&inner.replace('\n', &format!("\n{}", "  ".repeat(depth + 1))));
                    out.push('\n');
                }
                out.push('\n');
            }
            Block::Quote(inner) => {
                let mut t = String::new();
                ast_to_text(inner, &mut t, depth);
                for l in t.trim_end().lines() {
                    out.push_str("| ");
                    out.push_str(l);
                    out.push('\n');
                }
                out.push('\n');
            }
            Block::Table { head, rows, .. } => {
                if !head.is_empty() {
                    out.push_str(&head.iter().map(|c| spans_txt(c)).collect::<Vec<_>>().join("\t"));
                    out.push('\n');
                }
                for r in rows {
                    out.push_str(&r.iter().map(|c| spans_txt(c)).collect::<Vec<_>>().join("\t"));
                    out.push('\n');
                }
                out.push('\n');
            }
            Block::Rule => out.push_str("----------------------------------------\n\n"),
            Block::Html(t) => {
                let txt = spans_txt(&crate::media::html_to_spans(t));
                if !txt.trim().is_empty() {
                    out.push_str(txt.trim());
                    out.push_str("\n\n");
                }
            }
        }
    }
}

fn count_blocks(blocks: &[Block], st: &mut Stats) {
    let mut spans = |sp: &[Span], st: &mut Stats| {
        for s in sp {
            if s.link.is_some() {
                st.links += 1;
            }
            if s.image.is_some() {
                st.images += 1;
            }
        }
    };
    for b in blocks {
        match b {
            Block::Paragraph(sp) => {
                st.paragraphs += 1;
                spans(sp, st);
            }
            Block::Heading(l, sp, _) => {
                st.headings[(*l as usize).clamp(1, 6) - 1] += 1;
                spans(sp, st);
            }
            Block::Code(..) => st.code_blocks += 1,
            Block::List { items, .. } => {
                for it in items {
                    st.list_items += 1;
                    count_blocks(it, st);
                }
            }
            Block::Quote(inner) => {
                st.quotes += 1;
                count_blocks(inner, st);
            }
            Block::Table { head, rows, .. } => {
                st.tables += 1;
                for c in head {
                    spans(c, st);
                }
                for r in rows {
                    for c in r {
                        spans(c, st);
                    }
                }
            }
            _ => {}
        }
    }
}

#[derive(Default)]
struct Stats {
    paragraphs: usize,
    headings: [usize; 6],
    code_blocks: usize,
    list_items: usize,
    quotes: usize,
    tables: usize,
    links: usize,
    images: usize,
}

fn align_table_lines(lines: &[String]) -> Vec<String> {
    let split = |l: &str| -> Vec<String> {
        let t = l.trim();
        let t = t.strip_prefix('|').unwrap_or(t);
        let t = t.strip_suffix('|').unwrap_or(t);
        // tach theo | nhung khong tach \|
        let mut cells = Vec::new();
        let mut cur = String::new();
        let cs: Vec<char> = t.chars().collect();
        let mut i = 0;
        while i < cs.len() {
            if cs[i] == '\\' && i + 1 < cs.len() && cs[i + 1] == '|' {
                cur.push_str("\\|");
                i += 2;
                continue;
            }
            if cs[i] == '|' {
                cells.push(cur.trim().to_string());
                cur.clear();
            } else {
                cur.push(cs[i]);
            }
            i += 1;
        }
        cells.push(cur.trim().to_string());
        cells
    };
    let is_sep = |c: &str| -> bool { !c.is_empty() && c.chars().all(|x| x == '-' || x == ':') && c.contains('-') };
    let rows: Vec<Vec<String>> = lines.iter().map(|l| split(l)).collect();
    let ncols = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let mut aligns = vec!['l'; ncols];
    let mut sep_row: Option<usize> = None;
    for (ri, r) in rows.iter().enumerate() {
        if !r.is_empty() && r.iter().all(|c| is_sep(c)) {
            sep_row = Some(ri);
            for (ci, c) in r.iter().enumerate() {
                aligns[ci] = match (c.starts_with(':'), c.ends_with(':')) {
                    (true, true) => 'c',
                    (false, true) => 'r',
                    _ => 'l',
                };
            }
            break;
        }
    }
    let width = |s: &str| -> usize {
        s.chars()
            .map(|c| if ('\u{1100}'..='\u{115F}').contains(&c) || ('\u{2E80}'..='\u{A4CF}').contains(&c) || ('\u{AC00}'..='\u{D7A3}').contains(&c) || ('\u{FF00}'..='\u{FF60}').contains(&c) { 2 } else { 1 })
            .sum()
    };
    let mut w = vec![3usize; ncols];
    for (ri, r) in rows.iter().enumerate() {
        if Some(ri) == sep_row {
            continue;
        }
        for (ci, c) in r.iter().enumerate() {
            w[ci] = w[ci].max(width(c));
        }
    }
    rows.iter()
        .enumerate()
        .map(|(ri, r)| {
            let mut s = String::from("|");
            for ci in 0..ncols {
                let c = r.get(ci).cloned().unwrap_or_default();
                if Some(ri) == sep_row {
                    let body = match aligns[ci] {
                        'c' => format!(":{}:", "-".repeat(w[ci].saturating_sub(2))),
                        'r' => format!("{}:", "-".repeat(w[ci].saturating_sub(1))),
                        _ => "-".repeat(w[ci]),
                    };
                    s.push_str(&format!(" {body} |"));
                } else {
                    let pad = w[ci].saturating_sub(width(&c));
                    let cell = match aligns[ci] {
                        'r' => format!("{}{}", " ".repeat(pad), c),
                        'c' => format!("{}{}{}", " ".repeat(pad / 2), c, " ".repeat(pad - pad / 2)),
                        _ => format!("{}{}", c, " ".repeat(pad)),
                    };
                    s.push_str(&format!(" {cell} |"));
                }
            }
            s
        })
        .collect()
}

const MATH_TEMPLATES: &[(&str, &str)] = &[
    ("Phân số", "$\\frac{⟦a⟧}{b}$"),
    ("Căn bậc hai", "$\\sqrt{⟦x⟧}$"),
    ("Căn bậc n", "$\\sqrt[n]{⟦x⟧}$"),
    ("Luỹ thừa  xⁿ", "$x^{⟦n⟧}$"),
    ("Chỉ số dưới  xᵢ", "$x_{⟦i⟧}$"),
    ("Vectơ", "$\\vec{⟦v⟧}$"),
    ("Đạo hàm", "$\\frac{d⟦y⟧}{dx}$"),
    ("Đạo hàm riêng", "$\\frac{\\partial ⟦f⟧}{\\partial x}$"),
    ("Tổng  Σ", "\n$$\\sum_{i=1}^{n} ⟦a_i⟧$$\n"),
    ("Tích phân  ∫", "\n$$\\int_{a}^{b} ⟦f(x)⟧\\,dx$$\n"),
    ("Giới hạn", "\n$$\\lim_{x \\to 0} ⟦f(x)⟧$$\n"),
    ("Ma trận 2×2", "\n$$\\begin{pmatrix} ⟦a⟧ & b \\\\ c & d \\end{pmatrix}$$\n"),
    ("Hệ phương trình", "\n$$\\begin{cases} ⟦x + y = 1⟧ \\\\ x - y = 0 \\end{cases}$$\n"),
    ("Nghiệm PT bậc hai", "\n$$x = \\frac{-b \\pm \\sqrt{b^2 - 4ac}}{2a}$$\n"),
    ("Ngoặc co giãn", "$\\left( ⟦x⟧ \\right)$"),
    ("Nhiều dòng (aligned)", "\n$$\\begin{aligned} ⟦a⟧ &= b + c \\\\ &= d \\end{aligned}$$\n"),
];

const SYMBOLS: &[(&str, &[&str])] = &[
    ("Hy Lạp", &["α", "β", "γ", "δ", "ε", "ζ", "η", "θ", "λ", "μ", "π", "ρ", "σ", "τ", "φ", "ω", "Γ", "Δ", "Θ", "Λ", "Σ", "Φ", "Ψ", "Ω"]),
    ("Toán tử", &["±", "×", "÷", "≤", "≥", "≠", "≈", "≡", "∞", "∑", "∏", "∫", "√", "∂", "∇", "∈", "∉", "⊂", "∪", "∩", "→", "⇒", "⇔", "°", "²", "³", "½", "¼", "‰", "µ", "Ω", "…"]),
];

impl App {
    // ---------------------------------------------------------------- tep gan day
    pub fn push_recent(&mut self, path: &Path) {
        let s = path.to_string_lossy().into_owned();
        self.recent.retain(|x| x != &s);
        self.recent.insert(0, s);
        self.recent.truncate(MAX_RECENT);
        self.persist_session();
    }

    fn open_recent(&mut self, p: &str) {
        let path = PathBuf::from(p);
        if !path.exists() {
            self.status = format!("Tệp không còn tồn tại: {p}");
            self.recent.retain(|x| x != p);
            self.persist_session();
            return;
        }
        self.open_path(&path);
    }

    fn reopen_closed(&mut self) {
        if let Some(p) = self.closed.pop() {
            self.open_path(&p);
        } else {
            self.status = "Không có tab nào đã đóng để mở lại.".into();
        }
    }

    pub fn tick_autosave(&mut self) {
        if !self.autosave || self.last_autosave.elapsed() < Duration::from_secs(30) {
            return;
        }
        self.last_autosave = Instant::now();
        let idxs: Vec<usize> = (0..self.docs.len()).filter(|&i| self.docs[i].dirty() && self.docs[i].path.is_some()).collect();
        let mut n = 0;
        for i in idxs {
            if self.save_doc(i, false) {
                n += 1;
            }
        }
        if n > 0 {
            self.status = format!("Đã tự động lưu {n} tệp");
        }
    }

    pub fn handle_extra_keys(&mut self, ctx: &egui::Context) {
        let (reopen, f11, fix, f1, print, esc) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::T) && i.modifiers.ctrl && i.modifiers.shift,
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::F) && i.modifiers.ctrl && i.modifiers.shift,
                i.key_pressed(egui::Key::F1),
                i.key_pressed(egui::Key::P) && i.modifiers.ctrl,
                i.key_pressed(egui::Key::Escape),
            )
        });
        if reopen {
            self.reopen_closed();
        }
        if f11 {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
        }
        if fix {
            self.open_fixer();
        }
        if f1 {
            self.dialog = Some(Dialog::Shortcuts);
        }
        if print {
            self.export_pdf(ctx);
        }
        if esc && self.dialog.is_some() {
            self.dialog = None;
        }
    }

    // ---------------------------------------------------------------- chen van ban
    fn ensure_editable(&mut self) {
        if self.view == ViewMode::Preview {
            self.view = ViewMode::Split;
        }
    }

    /// Chen doan van ban tai con tro. `⟦…⟧` danh dau vung se duoc chon sau khi chen.
    pub fn insert_snippet(&mut self, snippet: &str) {
        self.ensure_editable();
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let id = doc.id;
        let (a, b) = sel_text(doc);
        let selected = if a < b { Some(slice_chars(&doc.text, a, b)) } else { None };
        let mut s = snippet.to_string();
        let (mut sa, mut sb) = (None, None);
        // vung chon hien tai thay the placeholder
        if let Some(sel) = &selected {
            if let (Some(p), Some(q)) = (s.find('⟦'), s.find('⟧')) {
                s = format!("{}{}{}", &s[..p], sel, &s[q + '⟧'.len_utf8()..]);
            }
        } else if let (Some(p), Some(q)) = (s.find('⟦'), s.find('⟧')) {
            let start = s[..p].chars().count();
            let len = s[p + '⟦'.len_utf8()..q].chars().count();
            sa = Some(start);
            sb = Some(start + len);
            s = format!("{}{}{}", &s[..p], &s[p + '⟦'.len_utf8()..q], &s[q + '⟧'.len_utf8()..]);
        }
        let n = s.chars().count();
        replace_char_range(&mut doc.text, a, b, &s);
        let (ca, cb) = match (sa, sb) {
            (Some(x), Some(y)) => (a + x, a + y),
            _ => (a + n, a + n),
        };
        doc.sel_start = ca;
        doc.sel_end = cb;
        doc.cursor_char = cb;
        self.find_goto = Some((id, ca, cb));
    }

    fn wrap_selection(&mut self, before: &str, after: &str, placeholder: &str) {
        self.ensure_editable();
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let id = doc.id;
        let (a, b) = sel_text(doc);
        let (ca, cb);
        if a < b {
            let sel = slice_chars(&doc.text, a, b);
            let s = format!("{before}{sel}{after}");
            let n = s.chars().count();
            replace_char_range(&mut doc.text, a, b, &s);
            ca = a + n;
            cb = a + n;
        } else {
            let s = format!("{before}{placeholder}{after}");
            replace_char_range(&mut doc.text, a, a, &s);
            ca = a + before.chars().count();
            cb = ca + placeholder.chars().count();
        }
        doc.sel_start = ca;
        doc.sel_end = cb;
        doc.cursor_char = cb;
        self.find_goto = Some((id, ca, cb));
    }

    fn insert_image_from_disk(&mut self) {
        let Some(p) = rfd::FileDialog::new().set_title("Chọn ảnh để chèn").add_filter("Ảnh", &["png", "jpg", "jpeg", "gif", "bmp", "webp"]).pick_file() else {
            return;
        };
        let doc_dir = self.docs.get(self.active).and_then(|d| d.path.as_ref()).and_then(|x| x.parent().map(|y| y.to_path_buf()));
        let rel = match &doc_dir {
            Some(d) => p.strip_prefix(d).map(|r| r.to_path_buf()).unwrap_or_else(|_| p.clone()),
            None => p.clone(),
        };
        let mut path_txt = rel.to_string_lossy().replace('\\', "/");
        if path_txt.contains(' ') {
            path_txt = path_txt.replace(' ', "%20");
        }
        let alt = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        self.insert_snippet(&format!("\n![{alt}]({path_txt})\n"));
    }

    fn insert_toc(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        if doc.toc.is_empty() {
            self.status = "Tài liệu chưa có tiêu đề để tạo mục lục.".into();
            return;
        }
        let min = doc.toc.iter().map(|e| e.level).min().unwrap_or(1);
        let mut s = String::from("\n## Mục lục\n\n");
        for e in &doc.toc {
            s.push_str(&"  ".repeat((e.level - min) as usize));
            s.push_str(&format!("- [{}](#{})\n", e.title, slugify(&e.title)));
        }
        s.push('\n');
        self.insert_snippet(&s);
    }

    fn format_table(&mut self) {
        self.ensure_editable();
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let id = doc.id;
        let lines: Vec<String> = doc.text.split('\n').map(|l| l.to_string()).collect();
        let before: String = doc.text.chars().take(doc.cursor_char).collect();
        let cur_line = before.matches('\n').count();
        let is_t = |l: &str| {
            let t = l.trim();
            t.starts_with('|') && t.matches('|').count() >= 2
        };
        if cur_line >= lines.len() || !is_t(&lines[cur_line]) {
            self.status = "Đặt con trỏ vào một dòng của bảng rồi bấm lại.".into();
            return;
        }
        let mut a = cur_line;
        while a > 0 && is_t(&lines[a - 1]) {
            a -= 1;
        }
        let mut b = cur_line;
        while b + 1 < lines.len() && is_t(&lines[b + 1]) {
            b += 1;
        }
        let aligned = align_table_lines(&lines[a..=b]);
        let mut all = lines.clone();
        all.splice(a..=b, aligned);
        let new_text = all.join("\n");
        if new_text == doc.text {
            self.status = "Bảng đã được căn chỉnh sẵn.".into();
            return;
        }
        doc.text = new_text;
        let off: usize = lines[..a].iter().map(|l| l.chars().count() + 1).sum();
        doc.cursor_char = off;
        doc.sel_start = off;
        doc.sel_end = off;
        self.find_goto = Some((id, off, off));
        self.status = format!("Đã căn chỉnh bảng ({} dòng).", b - a + 1);
    }

    fn change_case(&mut self, mode: CaseMode) {
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let (a, b) = sel_text(doc);
        if a >= b {
            self.status = "Hãy bôi chọn đoạn văn bản cần đổi chữ.".into();
            return;
        }
        self.ensure_editable();
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let id = doc.id;
        let sel = slice_chars(&doc.text, a, b);
        let out = match mode {
            CaseMode::Upper => sel.to_uppercase(),
            CaseMode::Lower => sel.to_lowercase(),
            CaseMode::Title => {
                let mut r = String::new();
                let mut up = true;
                for c in sel.chars() {
                    if c.is_whitespace() {
                        up = true;
                        r.push(c);
                    } else if up {
                        r.extend(c.to_uppercase());
                        up = false;
                    } else {
                        r.extend(c.to_lowercase());
                    }
                }
                r
            }
        };
        let n = out.chars().count();
        replace_char_range(&mut doc.text, a, b, &out);
        doc.sel_start = a;
        doc.sel_end = a + n;
        doc.cursor_char = a + n;
        self.find_goto = Some((id, a, a + n));
    }

    pub fn goto_anchor(&mut self, ctx: &egui::Context, anchor: &str) {
        let want = slugify(&percent_decode(anchor));
        let Some(doc) = self.docs.get(self.active) else { return };
        if let Some(e) = doc.toc.iter().find(|e| slugify(&e.title) == want) {
            let off = e.char_offset;
            let id = doc.id;
            ctx.data_mut(|d| d.insert_temp(egui::Id::new("preview-scroll-target"), off));
            self.toc_goto = Some((id, off));
            ctx.request_repaint();
        } else {
            self.status = format!("Không tìm thấy mục: #{anchor}");
        }
    }

    pub fn goto_line(&mut self, line1: usize) {
        if matches!(self.view, ViewMode::Preview) {
            self.view = ViewMode::Split;
        }
        let Some(doc) = self.docs.get(self.active) else { return };
        let id = doc.id;
        let mut off = 0usize;
        for (i, l) in doc.text.split('\n').enumerate() {
            if i + 1 >= line1 {
                let len = l.chars().count();
                self.find_goto = Some((id, off, off + len));
                self.toc_goto = None;
                return;
            }
            off += l.chars().count() + 1;
        }
    }

    // ---------------------------------------------------------------- xuat
    pub fn build_html(&self, auto_print: bool) -> Option<String> {
        let doc = self.docs.get(self.active)?;
        let title = doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "Tài liệu".into());
        let doc_dir = doc.path.as_ref().and_then(|p| p.parent().map(|x| x.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
        Some(html_from_markdown(&doc.text, &title, auto_print, &doc_dir))
    }

    pub fn export_html(&mut self) {
        let Some(html) = self.build_html(false) else { return };
        let Some(doc) = self.docs.get(self.active) else { return };
        let default_name = doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| format!("{}.html", s.to_string_lossy())).unwrap_or_else(|| "tai-lieu.html".into());
        if let Some(p) = rfd::FileDialog::new().set_title("Xuất HTML").set_file_name(default_name).add_filter("HTML", &["html", "htm"]).save_file() {
            match std::fs::write(&p, html) {
                Ok(()) => self.status = format!("Đã xuất HTML: {}", p.display()),
                Err(e) => self.status = format!("Lỗi xuất HTML: {e}"),
            }
        }
    }

    fn export_txt(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        let mut t = String::new();
        ast_to_text(&doc.ast, &mut t, 0);
        let default_name = doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| format!("{}.txt", s.to_string_lossy())).unwrap_or_else(|| "tai-lieu.txt".into());
        if let Some(p) = rfd::FileDialog::new().set_title("Xuất văn bản thuần").set_file_name(default_name).add_filter("Văn bản", &["txt"]).save_file() {
            match std::fs::write(&p, t.trim_end().to_string() + "\n") {
                Ok(()) => self.status = format!("Đã xuất TXT: {}", p.display()),
                Err(e) => self.status = format!("Lỗi xuất TXT: {e}"),
            }
        }
    }

    pub fn export_md(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        let default_name = doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| format!("{}.md", s.to_string_lossy())).unwrap_or_else(|| "tai-lieu.md".into());
        if let Some(p) = rfd::FileDialog::new()
            .set_title("Xuất bản sao Markdown (.md)")
            .set_file_name(default_name)
            .add_filter("Markdown", &["md", "markdown"])
            .save_file()
        {
            match std::fs::write(&p, &doc.text) {
                Ok(()) => self.status = format!("Đã xuất MD: {}", p.display()),
                Err(e) => self.status = format!("Lỗi xuất MD: {e}"),
            }
        }
    }

    /// In / lưu PDF: mở bản HTML trong trình duyệt mặc định, hộp thoại In tự bật (chọn "Lưu thành PDF").
    fn export_pdf(&mut self, ctx: &egui::Context) {
        let Some(html) = self.build_html(true) else { return };
        let dir = std::env::temp_dir().join("MarkdownTabs");
        let _ = std::fs::create_dir_all(&dir);
        let file = dir.join("in-pdf.html");
        match std::fs::write(&file, html) {
            Ok(()) => {
                let p = file.to_string_lossy().replace('\\', "/");
                let url = format!("file:///{}", docx::encode_uri(p.trim_start_matches('/')));
                ctx.open_url(egui::OpenUrl { url, new_tab: true });
                self.status = "Đã mở trình duyệt — chọn \"Lưu thành PDF\" trong hộp thoại In.".into();
            }
            Err(e) => self.status = format!("Không tạo được bản in: {e}"),
        }
    }

    fn copy_html(&mut self, ctx: &egui::Context) {
        if let Some(html) = self.build_html(false) {
            ctx.copy_text(html);
            self.status = "Đã sao chép HTML vào clipboard.".into();
        }
    }

    pub fn export_docx(&mut self) {
        let Some(doc) = self.docs.get(self.active) else { return };
        let doc_dir = doc.path.as_ref().and_then(|p| p.parent().map(|x| x.to_path_buf())).unwrap_or_else(|| PathBuf::from("."));
        let stem = doc.path.as_ref().and_then(|p| p.file_stem()).map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "Tài liệu".into());
        let Some(bytes) = docx::build(&doc.ast, &doc_dir, &stem, VERSION) else {
            self.status = "Lỗi khi tạo file DOCX.".into();
            return;
        };
        if let Some(path) = rfd::FileDialog::new().set_title("Xuất file DOCX").add_filter("Word", &["docx"]).set_file_name(format!("{stem}.docx")).save_file() {
            match std::fs::write(&path, &bytes) {
                Ok(()) => self.status = format!("Đã xuất DOCX: {}", path.display()),
                Err(e) => self.status = format!("Lỗi ghi DOCX: {e}"),
            }
        }
    }

    // ---------------------------------------------------------------- sua loi
    pub fn open_fixer(&mut self) {
        if let Some(doc) = self.docs.get(self.active) {
            let (tc, vn, uni) = fixer::detect_legacy(&doc.text);
            self.fx_hint = String::new();
            if uni < 3 && tc >= 4 && tc >= vn {
                self.fx_opts.legacy = fixer::Legacy::Tcvn3;
                self.fx_hint = format!("Phát hiện {tc} ký tự giống font cũ TCVN3 (.VnTime) — đã chọn chuyển sang Unicode.");
            } else if uni < 3 && vn >= 4 {
                self.fx_opts.legacy = fixer::Legacy::Vni;
                self.fx_hint = format!("Phát hiện {vn} cặp ký tự giống font cũ VNI — đã chọn chuyển sang Unicode.");
            } else {
                self.fx_opts.legacy = fixer::Legacy::None;
            }
        }
        self.fx_dirty = true;
        self.dialog = Some(Dialog::Fixer);
    }

    fn apply_fixes(&mut self, o: fixer::Opts) {
        let Some(doc) = self.docs.get_mut(self.active) else { return };
        let (new_text, changes) = fixer::fix(&doc.text, &o);
        if new_text == doc.text {
            self.status = "Không có gì cần sửa.".into();
            return;
        }
        let total: usize = changes.iter().map(|c| c.count).sum();
        self.fx_undo = Some((doc.id, doc.text.clone()));
        doc.text = new_text;
        self.status = format!("Đã sửa {} nhóm lỗi ({} chỗ). Có thể bấm \"Hoàn tác lần sửa\".", changes.len(), total);
        self.fx_dirty = true;
    }

    fn quick_fix(&mut self) {
        let mut o = fixer::Opts::recommended();
        if let Some(doc) = self.docs.get(self.active) {
            let (tc, vn, uni) = fixer::detect_legacy(&doc.text);
            if uni < 3 && tc >= 4 && tc >= vn {
                o.legacy = fixer::Legacy::Tcvn3;
            } else if uni < 3 && vn >= 4 {
                o.legacy = fixer::Legacy::Vni;
            }
        }
        self.apply_fixes(o);
    }

    fn undo_fix(&mut self) {
        if let Some((id, old)) = self.fx_undo.take() {
            if let Some(doc) = self.docs.iter_mut().find(|d| d.id == id) {
                doc.text = old;
                self.status = "Đã hoàn tác lần sửa lỗi gần nhất.".into();
                self.fx_dirty = true;
            }
        }
    }

    pub fn run_formula_check(&mut self) {
        if let Some(doc) = self.docs.get(self.active) {
            self.fm_result = Some(fixer::check_formulas(&doc.text));
        }
        self.dialog = Some(Dialog::Formulas);
    }

    // ---------------------------------------------------------------- UI: nut ribbon
    fn open_dialog_files(&mut self) {
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

    fn request_exit(&mut self, ctx: &egui::Context) {
        if self.docs.iter().any(|d| d.dirty()) {
            self.close_req = Some(CloseReq::Exit);
        } else {
            self.allow_exit = true;
            self.persist_session();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    pub fn ui_ribbon(&mut self, ctx: &egui::Context) {
        let dark = self.dark;
        let ac = accent(dark);
        let wide = ctx.screen_rect().width();
        // ---- hang 1: logo + tab ribbon + che do xem + sang/toi ----
        egui::TopBottomPanel::top("ribbon-tabs")
            .frame(egui::Frame::default().fill(chrome_fill(dark)).inner_margin(egui::Margin { left: 12.0, right: 12.0, top: 7.0, bottom: 0.0 }))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(4.0, 0.0);
                    ui.spacing_mut().button_padding = egui::vec2(11.0, 4.0);
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, egui::Rounding::same(9.0), ac);
                    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, "M", FontId::new(16.0, egui::FontFamily::Name("InterSemiBold".into())), if dark { rgb(10, 14, 22) } else { Color32::WHITE });
                    if wide > 900.0 {
                        ui.add_space(4.0);
                        ui.label(RichText::new("Markdown Tabs").strong().size(15.0).color(ui.visuals().text_color())).on_hover_text(format!("Markdown Tabs v{VERSION}"));
                    }
                    ui.add_space(10.0);
                    for (tab, label) in RibbonTab::ALL {
                        let sel = self.ribbon == tab && self.ribbon_open;
                        let txt = RichText::new(label).size(14.0).color(if sel { ac } else { ui.visuals().text_color() });
                        let txt = if sel { txt.strong() } else { txt };
                        let r = ui.add(egui::Button::new(txt).frame(false).min_size(egui::vec2(0.0, 30.0)));
                        if sel {
                            let y = r.rect.bottom();
                            ui.painter().rect_filled(egui::Rect::from_min_max(egui::pos2(r.rect.left() + 6.0, y - 3.0), egui::pos2(r.rect.right() - 6.0, y)), egui::Rounding { nw: 3.0, ne: 3.0, sw: 0.0, se: 0.0 }, ac);
                        }
                        if r.clicked() {
                            if self.ribbon == tab {
                                self.ribbon_open = !self.ribbon_open;
                            } else {
                                self.ribbon = tab;
                                self.ribbon_open = true;
                            }
                        }
                        r.on_hover_text("Bấm lần nữa để thu gọn / mở rộng thanh công cụ");
                        ui.add_space(6.0);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // nut Tap trung (Zen)
                        let ztip = if self.zen_mode { "Thoát chế độ Tập trung (Zen)" } else { "Chế độ Tập trung / Zen (ẩn thanh công cụ & mục lục)" };
                        if ibtn(ui, "maximize", ztip, 185.0, true, self.zen_mode).clicked() {
                            self.zen_mode = !self.zen_mode;
                        }
                        // nut sang / toi
                        let (icon, tip) = if dark { ("sun", "Chuyển sang giao diện sáng") } else { ("moon", "Chuyển sang giao diện tối") };
                        if ibtn(ui, icon, tip, if dark { 45.0 } else { 255.0 }, true, false).clicked() {
                            self.dark = !self.dark;
                            apply_theme(ctx, self.dark);
                            self.persist_session();
                        }
                        // chon 4 mau chu dao ngay tren thanh dinh
                        let mut picked_accent: Option<usize> = None;
                        icon_menu(ui, "palette", "Đổi màu chủ đạo", 265.0, |ui| {
                            ui.label(RichText::new("Màu chủ đạo").small().color(ui.visuals().weak_text_color()));
                            for (i, (name, light_c, bright_c)) in ACCENTS.iter().enumerate() {
                                let c = if dark { *bright_c } else { *light_c };
                                let mark = if accent_index() == i { "✓ " } else { "   " };
                                if ui.button(RichText::new(format!("{mark}{name}")).color(c)).clicked() {
                                    picked_accent = Some(i);
                                }
                            }
                        });
                        if let Some(i) = picked_accent {
                            set_accent_index(i);
                            apply_theme(ctx, self.dark);
                            self.persist_session();
                        }
                        // bang lenh nhanh Ctrl+Shift+P
                        if ibtn(ui, "search", "Bảng lệnh nhanh (Ctrl+Shift+P)", 215.0, true, self.cmd_open).clicked() {
                            self.cmd_open = !self.cmd_open;
                            self.cmd_query.clear();
                        }
                        ui.add_space(4.0);
                        // che do xem (phan doan): them theo thu tu phai -> trai, nen nen duoc ve sau cung
                        let compact = wide < 1080.0;
                        let bg_idx = ui.painter().add(egui::Shape::Noop);
                        let mut union = egui::Rect::NOTHING;
                        for (mode, icon, label, tip) in [
                            (ViewMode::Split, "columns", "Chia đôi", "Soạn thảo và xem trước cạnh nhau (Ctrl+3)"),
                            (ViewMode::Preview, "eye", "Xem trước", "Chỉ xem trước (Ctrl+2)"),
                            (ViewMode::Edit, "edit", "Soạn thảo", "Chỉ soạn thảo (Ctrl+1)"),
                        ] {
                            let (clicked, rect) = seg_btn(ui, icon, if compact { "" } else { label }, self.view == mode, tip);
                            union = union.union(rect);
                            if clicked {
                                self.view = mode;
                            }
                        }
                        ui.painter().set(bg_idx, egui::Shape::rect_filled(union.expand(3.0), egui::Rounding::same(11.0), if dark { rgb(13, 16, 24) } else { rgb(228, 235, 246) }));
                    });
                });
            });

        // ---- hang 2: nut bieu tuong cua tab (nho gon, khong ten, tooltip khi re chuot) ----
        if !self.ribbon_open || self.zen_mode {
            return;
        }
        egui::TopBottomPanel::top("ribbon-body")
            .frame(egui::Frame::default().fill(chrome_fill(dark)).stroke(egui::Stroke::new(1.0, border_color(dark))).inner_margin(egui::Margin::symmetric(12.0, 4.0)))
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                    match self.ribbon {
                        RibbonTab::File => self.ribbon_file(ui, ctx),
                        RibbonTab::Format => self.ribbon_format(ui),
                        RibbonTab::Insert => self.ribbon_insert(ui),
                        RibbonTab::View => self.ribbon_view(ui, ctx),
                        RibbonTab::Export => self.ribbon_export(ui, ctx),
                        RibbonTab::Tools => self.ribbon_tools(ui),
                        RibbonTab::Help => self.ribbon_help(ui, ctx),
                    }
                });
            });
    }

    fn ribbon_file(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ibtn(ui, "file-plus", "Tài liệu mới (Ctrl+N)", 215.0, true, false).clicked() {
            self.new_doc();
        }
        if ibtn(ui, "folder-open", "Mở file… (Ctrl+O, chọn được nhiều file)", 215.0, true, false).clicked() {
            self.open_dialog_files();
        }
        if ibtn(ui, "folder", "Mở thư mục… (mọi file Markdown trong thư mục)", 215.0, true, false).clicked() {
            if let Some(d) = rfd::FileDialog::new().set_title("Chọn thư mục").pick_folder() {
                self.open_path(&d);
            }
        }
        if ibtn(ui, "undo", "Mở lại tab đã đóng (Ctrl+Shift+T)", 28.0, !self.closed.is_empty(), false).clicked() {
            self.reopen_closed();
        }
        // ---- Tep gan day ----
        let recent = self.recent.clone();
        let mut open: Option<String> = None;
        let mut clear = false;
        icon_menu(ui, "clock", &format!("Tệp gần đây ({})", recent.len()), 185.0, |ui| {
            ui.set_min_width(300.0);
            if recent.is_empty() {
                ui.label("Chưa có tệp nào được mở gần đây.");
            }
            for p in &recent {
                let exists = Path::new(p).exists();
                let name = short_name(p, 40);
                let dir = Path::new(p).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
                let r = ui.add(egui::Button::new(RichText::new(name).color(if exists { ui.visuals().text_color() } else { ui.visuals().weak_text_color() })).frame(false));
                if r.on_hover_text(format!("{dir}{}", if exists { "" } else { "\n(tệp không còn tồn tại)" })).clicked() {
                    open = Some(p.clone());
                }
            }
            if !recent.is_empty() {
                ui.separator();
                if ui.button("Xoá danh sách gần đây").clicked() {
                    clear = true;
                }
            }
        });
        if clear {
            self.recent.clear();
            self.persist_session();
        }
        if let Some(p) = open {
            self.open_recent(&p);
        }
        vsep(ui);
        // Cum nut Xuat nhanh (Word, PDF, MD, HTML, Print) dat ngay canh Tep mo gan day
        if ibtn(ui, "word", "Xuất Word (.docx) — có ảnh, bảng, công thức OMML", 215.0, true, false).clicked() {
            self.export_docx();
        }
        if ibtn(ui, "pdf", "Xuất PDF (.pdf)", 350.0, true, false).clicked() {
            self.export_pdf(ctx);
        }
        if ibtn(ui, "md", "Xuất Markdown (.md)", 265.0, true, false).clicked() {
            self.export_md();
        }
        if ibtn(ui, "globe", "Xuất HTML (.html) độc lập, nhúng ảnh", 185.0, true, false).clicked() {
            self.export_html();
        }
        if ibtn(ui, "printer", "In tài liệu (Ctrl+P)", 28.0, true, false).clicked() {
            self.export_pdf(ctx);
        }
        vsep(ui);
        if ibtn(ui, "save", "Lưu (Ctrl+S)", 150.0, true, false).clicked() {
            self.save_doc(self.active, false);
        }
        if ibtn(ui, "save-plus", "Lưu thành… (Ctrl+Shift+S)", 150.0, true, false).clicked() {
            self.save_doc(self.active, true);
        }
        if ibtn(ui, "layers", "Lưu tất cả tab đang có thay đổi", 150.0, true, false).clicked() {
            let idxs: Vec<usize> = (0..self.docs.len()).filter(|&i| self.docs[i].dirty()).collect();
            let n = idxs.len();
            for i in idxs {
                self.save_doc(i, false);
            }
            self.status = format!("Đã lưu {n} tab.");
        }
        vsep(ui);
        if ibtn(ui, "x", "Đóng tab (Ctrl+W)", 350.0, true, false).clicked() {
            let idx = self.active;
            if self.docs.get(idx).map(|d| d.dirty()).unwrap_or(false) {
                self.close_req = Some(CloseReq::Tab(idx));
            } else {
                self.close_tab_now(idx);
            }
        }
        if ibtn(ui, "power", "Thoát ứng dụng", 350.0, true, false).clicked() {
            self.request_exit(ctx);
        }
    }

    fn ribbon_format(&mut self, ui: &mut egui::Ui) {
        let mut kind: Option<FmtKind> = None;
        if ibtn(ui, "bold", "Đậm (Ctrl+B)", 215.0, true, false).clicked() { kind = Some(FmtKind::Bold); }
        if ibtn(ui, "italic", "Nghiêng (Ctrl+I)", 215.0, true, false).clicked() { kind = Some(FmtKind::Italic); }
        if ibtn(ui, "strike", "Gạch ngang chữ", 215.0, true, false).clicked() { kind = Some(FmtKind::Strike); }
        if ibtn(ui, "code", "Mã trong dòng", 215.0, true, false).clicked() { kind = Some(FmtKind::Code); }
        let mut case: Option<CaseMode> = None;
        icon_menu(ui, "text:Aa", "Đổi kiểu chữ cho đoạn đang bôi chọn", 215.0, |ui| {
            if ui.button("CHỮ HOA").clicked() { case = Some(CaseMode::Upper); }
            if ui.button("chữ thường").clicked() { case = Some(CaseMode::Lower); }
            if ui.button("Chữ Hoa Đầu Từ").clicked() { case = Some(CaseMode::Title); }
        });
        vsep(ui);
        icon_menu(ui, "heading", "Tiêu đề", 265.0, |ui| {
            if ui.button("Tiêu đề 1").clicked() { kind = Some(FmtKind::H1); }
            if ui.button("Tiêu đề 2").clicked() { kind = Some(FmtKind::H2); }
            if ui.button("Tiêu đề 3").clicked() { kind = Some(FmtKind::H3); }
        });
        if ibtn(ui, "list", "Danh sách chấm", 265.0, true, false).clicked() { kind = Some(FmtKind::Bullet); }
        if ibtn(ui, "list-num", "Danh sách đánh số", 265.0, true, false).clicked() { kind = Some(FmtKind::Numbered); }
        if ibtn(ui, "check", "Danh sách việc cần làm (task list)", 265.0, true, false).clicked() { kind = Some(FmtKind::Task); }
        if ibtn(ui, "quote", "Trích dẫn", 265.0, true, false).clicked() { kind = Some(FmtKind::Quote); }
        if let Some(m) = case {
            self.change_case(m);
        }
        if let Some(k) = kind {
            self.ensure_editable();
            self.fmt(k);
        }
    }

    fn ribbon_insert(&mut self, ui: &mut egui::Ui) {
        if ibtn(ui, "link", "Chèn liên kết (Ctrl+K)", 185.0, true, false).clicked() {
            self.ensure_editable();
            self.fmt(FmtKind::Link);
        }
        if ibtn(ui, "image", "Chèn cú pháp ảnh ![mô tả](đường-dẫn)", 185.0, true, false).clicked() {
            self.ensure_editable();
            self.fmt(FmtKind::Image);
        }
        if ibtn(ui, "image-plus", "Chọn ảnh từ máy và chèn đường dẫn", 185.0, true, false).clicked() {
            self.insert_image_from_disk();
        }
        if ibtn(ui, "table", "Chèn bảng mẫu", 185.0, true, false).clicked() {
            self.ensure_editable();
            self.fmt(FmtKind::Table);
        }
        if ibtn(ui, "codeblock", "Chèn khối mã", 185.0, true, false).clicked() {
            self.ensure_editable();
            self.fmt(FmtKind::Fence);
        }
        if ibtn(ui, "minus", "Đường kẻ ngang", 185.0, true, false).clicked() {
            self.ensure_editable();
            self.fmt(FmtKind::Hr);
        }
        vsep(ui);
        if ibtn(ui, "sigma", "Công thức trong dòng $…$ (bọc đoạn đang chọn)", 28.0, true, false).clicked() {
            self.wrap_selection("$", "$", "x^2");
        }
        if ibtn(ui, "sigma-block", "Công thức riêng dòng $$…$$", 28.0, true, false).clicked() {
            self.wrap_selection("\n$$\n", "\n$$\n", "E = mc^2");
        }
        let mut snippet: Option<String> = None;
        icon_menu(ui, "grid", "Mẫu công thức", 28.0, |ui| {
            ui.set_min_width(220.0);
            for (name, tpl) in MATH_TEMPLATES {
                if ui.button(*name).clicked() {
                    snippet = Some((*tpl).to_string());
                }
            }
        });
        icon_menu_keep(ui, "omega", "Bảng ký hiệu toán học", 28.0, |ui| {
            ui.set_max_width(300.0);
            for (grp, items) in SYMBOLS {
                ui.label(RichText::new(*grp).small().color(ui.visuals().weak_text_color()));
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(2.0, 2.0);
                    for s in *items {
                        if ui.add(egui::Button::new(RichText::new(*s).size(15.0)).min_size(egui::vec2(28.0, 26.0))).clicked() {
                            snippet = Some((*s).to_string());
                        }
                    }
                });
            }
        });
        if let Some(s) = snippet {
            self.insert_snippet(&s);
        }
        vsep(ui);
        if ibtn(ui, "toc", "Chèn mục lục tự động từ các tiêu đề", 330.0, true, false).clicked() {
            self.insert_toc();
        }
        if ibtn(ui, "asterisk", "Chèn chú thích cuối trang [^1]", 330.0, true, false).clicked() {
            self.insert_snippet("[^⟦1⟧]\n\n[^1]: Nội dung chú thích.\n");
        }
        if ibtn(ui, "calendar", "Chèn ngày hôm nay", 330.0, true, false).clicked() {
            let d = docx_today();
            self.insert_snippet(&d);
        }
    }

    fn ribbon_view(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ibtn(ui, "sidebar", "Hiện / ẩn mục lục bên trái", 215.0, true, self.show_toc).clicked() {
            self.show_toc = !self.show_toc;
        }
        vsep(ui);
        if ibtn(ui, "zoomout", "Thu nhỏ (Ctrl+-)", 150.0, true, false).clicked() {
            let z = self.zoom - 0.1;
            self.set_zoom(ctx, z);
        }
        if ibtn_w(ui, &format!("text:{:.0}%", self.zoom * 100.0), "Đặt lại 100% (Ctrl+0)", 150.0, true, false, 50.0).clicked() {
            self.set_zoom(ctx, 1.0);
        }
        if ibtn(ui, "zoomin", "Phóng to (Ctrl++)", 150.0, true, false).clicked() {
            let z = self.zoom + 0.1;
            self.set_zoom(ctx, z);
        }
        vsep(ui);
        let tip = if self.fullscreen { "Thoát toàn màn hình (F11)" } else { "Toàn màn hình (F11)" };
        if ibtn(ui, "maximize", tip, 28.0, true, self.fullscreen).clicked() {
            self.fullscreen = !self.fullscreen;
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.fullscreen));
        }
    }

    fn ribbon_export(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ibtn(ui, "word", "Xuất Word (.docx): có ảnh, bảng, công thức Equation sửa được trong Word", 215.0, true, false).clicked() {
            self.export_docx();
        }
        if ibtn(ui, "globe", "Xuất HTML độc lập: ảnh nhúng sẵn, công thức MathML", 185.0, true, false).clicked() {
            self.export_html();
        }
        if ibtn(ui, "printer", "PDF / In (Ctrl+P): mở trình duyệt, chọn \"Lưu thành PDF\"", 28.0, true, false).clicked() {
            self.export_pdf(ctx);
        }
        if ibtn(ui, "txt", "Xuất văn bản thuần (.txt), bỏ ký hiệu Markdown", 265.0, true, false).clicked() {
            self.export_txt();
        }
        vsep(ui);
        if ibtn(ui, "codeblock", "Sao chép mã HTML vào clipboard", 150.0, true, false).clicked() {
            self.copy_html(ctx);
        }
        if ibtn(ui, "copy", "Sao chép toàn bộ Markdown nguồn", 150.0, true, false).clicked() {
            if let Some(d) = self.docs.get(self.active) {
                ctx.copy_text(d.text.clone());
                self.status = "Đã sao chép toàn bộ nội dung.".into();
            }
        }
    }

    fn ribbon_tools(&mut self, ui: &mut egui::Ui) {
        if ibtn(ui, "wand", "Sửa lỗi trình bày & font… (Ctrl+Shift+F): chữ vỡ mã, font cũ TCVN3/VNI, ký tự MathType, dấu công thức, bảng, danh sách", 330.0, true, false).clicked() {
            self.open_fixer();
        }
        if ibtn(ui, "bolt", "Sửa nhanh: áp dụng ngay bộ sửa lỗi khuyến nghị", 45.0, true, false).clicked() {
            self.quick_fix();
        }
        if ibtn(ui, "undo", "Hoàn tác lần sửa lỗi gần nhất", 28.0, self.fx_undo.is_some(), false).clicked() {
            self.undo_fix();
        }
        vsep(ui);
        if ibtn(ui, "check-circle", "Kiểm tra công thức: lệnh sai, ngoặc thiếu, dấu $ lẻ", 150.0, true, false).clicked() {
            self.run_formula_check();
        }
        if ibtn(ui, "align", "Căn chỉnh bảng tại vị trí con trỏ", 215.0, true, false).clicked() {
            self.format_table();
        }
        if ibtn(ui, "chart", "Thống kê: từ, ký tự, tiêu đề, bảng, công thức…", 265.0, true, false).clicked() {
            self.dialog = Some(Dialog::Stats);
        }
        vsep(ui);
        if ibtn(ui, "search", "Tìm / Thay thế (Ctrl+F)", 185.0, true, self.find_open).clicked() {
            self.find_open = !self.find_open;
        }
        if ibtn(ui, "refresh", if self.autosave { "Tự động lưu mỗi 30 giây: đang BẬT" } else { "Tự động lưu mỗi 30 giây: đang tắt (bấm để bật)" }, 150.0, true, self.autosave).clicked() {
            self.autosave = !self.autosave;
            self.persist_session();
        }
    }

    fn ribbon_help(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if ibtn(ui, "keyboard", "Phím tắt (F1)", 215.0, true, false).clicked() {
            self.dialog = Some(Dialog::Shortcuts);
        }
        if ibtn(ui, "book", "Cú pháp Markdown và lệnh LaTeX được hỗ trợ", 265.0, true, false).clicked() {
            self.dialog = Some(Dialog::Syntax);
        }
        if ibtn(ui, "info", "Giới thiệu", 185.0, true, false).clicked() {
            self.dialog = Some(Dialog::About);
        }
        vsep(ui);
        ui.label(RichText::new("Màu chủ đạo").size(12.5).color(ui.visuals().weak_text_color()));
        for (i, (name, light, bright)) in ACCENTS.iter().enumerate() {
            let col = if self.dark { *bright } else { *light };
            let (rect, resp) = ui.allocate_exact_size(egui::vec2(26.0, 28.0), egui::Sense::click());
            let c = egui::Rect::from_center_size(rect.center(), egui::vec2(20.0, 20.0));
            ui.painter().circle_filled(c.center(), 9.0, col);
            if accent_index() == i {
                ui.painter().circle_stroke(c.center(), 12.0, egui::Stroke::new(2.0, ui.visuals().text_color()));
            } else if resp.hovered() {
                ui.painter().circle_stroke(c.center(), 12.0, egui::Stroke::new(1.5, ui.visuals().weak_text_color()));
            }
            if resp.on_hover_text(format!("Màu chủ đạo: {name}")).clicked() {
                set_accent_index(i);
                apply_theme(ctx, self.dark);
                self.persist_session();
            }
        }
    }

    // ---------------------------------------------------------------- thanh tab tai lieu
    pub fn ui_tabbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tabbar")
            .frame(egui::Frame::default().fill(tabstrip_fill(self.dark)).inner_margin(egui::Margin::symmetric(8.0, 5.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::horizontal().id_salt("tabs-scroll").auto_shrink([false, true]).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        let mut to_close: Option<usize> = None;
                        let mut to_select: Option<usize> = None;
                        for i in 0..self.docs.len() {
                            let doc = &self.docs[i];
                            let base = doc.path.as_ref().and_then(|p| p.file_name().map(|f| f.to_string_lossy().into_owned())).unwrap_or_else(|| "Tài liệu mới".into());
                            let dirty = doc.dirty();
                            let selected = i == self.active;
                            let full_name = doc.path.as_ref().map(|p| p.display().to_string());
                            let base = if base.chars().count() > 28 { format!("{}…", base.chars().take(27).collect::<String>()) } else { base };
                            let frame = egui::Frame::default()
                                .fill(if selected { ui.visuals().panel_fill } else { Color32::TRANSPARENT })
                                .rounding(egui::Rounding { nw: 9.0, ne: 9.0, sw: 0.0, se: 0.0 })
                                .inner_margin(egui::Margin::symmetric(10.0, 4.0));
                            let tab_out = frame.show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 6.0;
                                    let (ir, _) = ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::hover());
                                    let hues = [215.0, 150.0, 28.0, 265.0, 330.0, 185.0];
                                    icons::paint(ui.painter(), ir, "file", hue_color(hues[i % hues.len()], self.dark));
                                    if dirty {
                                        let (dot, _) = ui.allocate_exact_size(egui::vec2(8.0, 8.0), egui::Sense::hover());
                                        ui.painter().circle_filled(dot.center(), 3.5, AMBER);
                                    }
                                    let mut txt = RichText::new(&base).size(13.5);
                                    if selected {
                                        txt = txt.strong().color(ui.visuals().text_color());
                                    }
                                    let lab = ui.add(egui::Label::new(txt).sense(egui::Sense::click()));
                                    let lab = match &full_name {
                                        Some(f) => lab.on_hover_text(f),
                                        None => lab,
                                    };
                                    if lab.clicked() {
                                        to_select = Some(i);
                                    }
                                    if ui.add(egui::Button::new(RichText::new("×").size(14.0)).frame(false).min_size(egui::vec2(20.0, 20.0))).on_hover_text("Đóng tab (Ctrl+W)").clicked() {
                                        to_close = Some(i);
                                    }
                                });
                            });
                            let tab_rect = tab_out.response.rect;
                            let tab_sense = ui.interact(tab_rect, ui.make_persistent_id(("tab-hit", i)), egui::Sense::click());
                            if tab_sense.middle_clicked() {
                                to_close = Some(i);
                            }
                            let mut close_others = false;
                            let mut copy_path: Option<String> = None;
                            tab_sense.context_menu(|ui| {
                                if ui.button("Đóng tab này").clicked() {
                                    to_close = Some(i);
                                    ui.close_menu();
                                }
                                if ui.button("Đóng các tab khác").clicked() {
                                    close_others = true;
                                    ui.close_menu();
                                }
                                if let Some(fp) = &full_name {
                                    if ui.button("Sao chép đường dẫn").clicked() {
                                        copy_path = Some(fp.clone());
                                        ui.close_menu();
                                    }
                                }
                            });
                            if let Some(fp) = copy_path {
                                ctx.copy_text(fp);
                                self.status = "Đã sao chép đường dẫn tệp.".into();
                            }
                            if close_others {
                                let keep_id = self.docs[i].id;
                                self.docs.retain(|d| d.id == keep_id || d.dirty());
                                self.active = self.docs.iter().position(|d| d.id == keep_id).unwrap_or(0);
                                break;
                            }
                            if selected {
                                ui.painter().line_segment(
                                    [tab_rect.left_top() + egui::vec2(6.0, 1.0), tab_rect.right_top() + egui::vec2(-6.0, 1.0)],
                                    egui::Stroke::new(2.5, accent(self.dark)),
                                );
                            }
                        }
                        if ibtn_w(ui, "plus", "Tab mới (Ctrl+N)", 215.0, true, false, 28.0).clicked() {
                            self.new_doc();
                        }
                        if let Some(i) = to_select {
                            self.active = i;
                        }
                        if let Some(i) = to_close {
                            if self.docs.get(i).map(|d| d.dirty()).unwrap_or(false) {
                                self.close_req = Some(CloseReq::Tab(i));
                            } else {
                                self.close_tab_now(i);
                            }
                        }
                    });
                });
            });
    }

    // ---------------------------------------------------------------- trang bat dau
    pub fn ui_start_page(&mut self, ui: &mut egui::Ui) {
        let recent = self.recent.clone();
        let mut open: Option<String> = None;
        let mut act: u8 = 0;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(36.0);
                ui.label(RichText::new("Markdown Tabs").size(30.0).strong().color(ui.visuals().text_color()));
                ui.label(RichText::new("Xem, soạn thảo và xuất Markdown — có công thức toán, xuất Word chuẩn.").color(ui.visuals().weak_text_color()));
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().button_padding = egui::vec2(14.0, 8.0);
                    let w = 3.0 * 120.0 + 16.0;
                    ui.add_space(((ui.available_width() - w) / 2.0).max(0.0));
                    if ui.button("Mở file…").clicked() {
                        act = 1;
                    }
                    if ui.button("Mở thư mục…").clicked() {
                        act = 2;
                    }
                    if ui.button("Viết tài liệu mới").clicked() {
                        act = 3;
                    }
                });
                ui.add_space(22.0);
                ui.label(RichText::new("TỆP GẦN ĐÂY").size(11.0).color(ui.visuals().weak_text_color()));
                ui.add_space(4.0);
                if recent.is_empty() {
                    ui.label(RichText::new("Chưa có tệp nào. Kéo thả file .md vào cửa sổ để mở.").color(ui.visuals().weak_text_color()));
                }
                for p in recent.iter().take(10) {
                    let exists = Path::new(p).exists();
                    let dir = Path::new(p).parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
                    ui.horizontal(|ui| {
                        ui.add_space(((ui.available_width() - 460.0) / 2.0).max(0.0));
                        let r = ui.add(egui::Button::new(RichText::new(short_name(p, 34)).size(14.5).color(if exists { link_color(ui) } else { ui.visuals().weak_text_color() })).frame(false));
                        if r.on_hover_text(p).clicked() {
                            open = Some(p.clone());
                        }
                        ui.add(egui::Label::new(RichText::new(dir).small().color(ui.visuals().weak_text_color())).truncate());
                    });
                }
            });
        });
        match act {
            1 => self.open_dialog_files(),
            2 => {
                if let Some(d) = rfd::FileDialog::new().set_title("Chọn thư mục").pick_folder() {
                    self.open_path(&d);
                }
            }
            3 => self.view = ViewMode::Split,
            _ => {}
        }
        if let Some(p) = open {
            self.open_recent(&p);
        }
    }

    // ---------------------------------------------------------------- hop thoai
    pub fn ui_dialogs(&mut self, ctx: &egui::Context) {
        let Some(d) = self.dialog else { return };
        let mut open = true;
        match d {
            Dialog::Fixer => self.dlg_fixer(ctx, &mut open),
            Dialog::Formulas => self.dlg_formulas(ctx, &mut open),
            Dialog::Stats => self.dlg_stats(ctx, &mut open),
            Dialog::Shortcuts => self.dlg_text(ctx, &mut open, "Phím tắt", SHORTCUTS),
            Dialog::Syntax => self.dlg_text(ctx, &mut open, "Cú pháp Markdown & công thức được hỗ trợ", SYNTAX),
            Dialog::About => self.dlg_about(ctx, &mut open),
        }
        if !open && self.dialog == Some(d) {
            self.dialog = None;
        }
    }

    fn dlg_fixer(&mut self, ctx: &egui::Context, open: &mut bool) {
        let screen = ctx.screen_rect();
        let mut apply = false;
        let mut undo = false;
        let mut close = false;
        let mut changed = false;
        let mut opts = self.fx_opts.clone();
        egui::Window::new("Sửa lỗi trình bày, font chữ & công thức")
            .open(open)
            .collapsible(false)
            .resizable(true)
            .default_width((screen.width() * 0.8).clamp(360.0, 640.0))
            .default_height((screen.height() * 0.8).clamp(300.0, 620.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                if !self.fx_hint.is_empty() {
                    ui.label(RichText::new(&self.fx_hint).color(AMBER));
                    ui.add_space(4.0);
                }
                egui::ScrollArea::vertical().auto_shrink([false, false]).max_height((ui.available_height() - 190.0).max(120.0)).show(ui, |ui| {
                    fn cb(ui: &mut egui::Ui, changed: &mut bool, v: &mut bool, label: &str, tip: &str) {
                        if ui.checkbox(v, label).on_hover_text(tip).changed() {
                            *changed = true;
                        }
                    }
                    ui.label(RichText::new("Font chữ & mã ký tự").strong().color(ui.visuals().text_color()));
                    cb(ui, &mut changed, &mut opts.mojibake, "Sửa chữ bị vỡ mã (vd. \"Viá»‡t\" → \"Việt\")", "UTF-8 bị đọc nhầm thành Windows-1252");
                    cb(ui, &mut changed, &mut opts.symbol_font, "Ký tự font Symbol/MathType (ô vuông, ký tự lạ) → Hy Lạp, ≤, ≥, ∑…", "Vùng riêng U+F0xx khi dán từ Word/MathType");
                    cb(ui, &mut changed, &mut opts.nfc, "Gộp dấu tiếng Việt dạng tổ hợp (NFC)", "Dấu bị tách rời khỏi chữ cái");
                    cb(ui, &mut changed, &mut opts.invisible, "Xoá ký tự vô hình / điều khiển", "Zero-width, BOM, điều khiển…");
                    cb(ui, &mut changed, &mut opts.spaces, "Khoảng trắng không ngắt, Tab, CRLF → chuẩn", "NBSP, khoảng trắng Unicode → dấu cách");
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Font tiếng Việt cũ:");
                        let before = opts.legacy;
                        ui.selectable_value(&mut opts.legacy, fixer::Legacy::None, "Không");
                        ui.selectable_value(&mut opts.legacy, fixer::Legacy::Tcvn3, "TCVN3 (.VnTime)");
                        ui.selectable_value(&mut opts.legacy, fixer::Legacy::Vni, "VNI");
                        if before != opts.legacy {
                            changed = true;
                        }
                    });
                    ui.add_space(6.0);
                    ui.label(RichText::new("Công thức").strong().color(ui.visuals().text_color()));
                    cb(ui, &mut changed, &mut opts.math_delims, "\\( … \\) và \\[ … \\] → $ … $ và $$ … $$", "Cú pháp LaTeX kiểu cũ");
                    cb(ui, &mut changed, &mut opts.math_trim, "Bỏ khoảng trắng thừa trong $ x $", "Nhiều trình xem không nhận $ x $");
                    cb(ui, &mut changed, &mut opts.bare_latex, "Dòng LaTeX trần (không có $) → khối $$ … $$", "Vd. dòng chỉ có \\frac{a}{b} = …");
                    cb(ui, &mut changed, &mut opts.currency, "Dấu $ tiền tệ ($5, $10) → \\$ (tránh bị hiểu là công thức)", "");
                    cb(ui, &mut changed, &mut opts.unicode_scripts, "Chỉ số Unicode (x², H₂O) → công thức $x^{2}$", "Mặc định tắt");
                    ui.add_space(6.0);
                    ui.label(RichText::new("Trình bày").strong().color(ui.visuals().text_color()));
                    cb(ui, &mut changed, &mut opts.trim, "Xoá khoảng trắng cuối dòng, dòng trống thừa", "");
                    cb(ui, &mut changed, &mut opts.heading_space, "Tiêu đề thiếu dấu cách sau # (#Tiêu đề → # Tiêu đề)", "");
                    cb(ui, &mut changed, &mut opts.bullets, "Ký hiệu danh sách •, ●, ▪, – → \"-\"", "Thường gặp khi dán từ Word/PDF");
                    cb(ui, &mut changed, &mut opts.table_fix, "Bảng: thêm dòng trống phía trước và dòng phân cách |---|", "Bảng thiếu sẽ không hiển thị");
                    cb(ui, &mut changed, &mut opts.punct_space, "Bỏ dấu cách thừa trước , ; ! ?", "Mặc định tắt");
                    cb(ui, &mut changed, &mut opts.quotes, "Dấu nháy cong “ ” ‘ ’ → nháy thẳng", "Mặc định tắt");
                });
                ui.separator();
                ui.label(RichText::new("Dự kiến sẽ sửa").strong().color(ui.visuals().text_color()));
                egui::ScrollArea::vertical().id_salt("fx-summary").max_height(96.0).auto_shrink([false, true]).show(ui, |ui| {
                    if self.fx_changes.is_empty() {
                        ui.label(RichText::new("Không phát hiện lỗi nào với các tuỳ chọn đã chọn.").color(ui.visuals().weak_text_color()));
                    }
                    for c in &self.fx_changes {
                        ui.label(format!("• {} — {} chỗ", c.label, c.count));
                    }
                });
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui.add_enabled(!self.fx_changes.is_empty(), egui::Button::new(RichText::new("Áp dụng cho tài liệu").strong().color(Color32::WHITE)).fill(accent(self.dark))).clicked() {
                        apply = true;
                    }
                    if ui.add_enabled(self.fx_undo.is_some(), egui::Button::new("Hoàn tác lần sửa")).clicked() {
                        undo = true;
                    }
                    if ui.button("Chọn khuyến nghị").clicked() {
                        let legacy = opts.legacy;
                        opts = fixer::Opts::recommended();
                        opts.legacy = legacy;
                        changed = true;
                    }
                    if ui.button("Đóng").clicked() {
                        close = true;
                    }
                });
            });
        self.fx_opts = opts;
        if changed {
            self.fx_dirty = true;
        }
        if self.fx_dirty {
            if let Some(doc) = self.docs.get(self.active) {
                self.fx_changes = fixer::fix(&doc.text, &self.fx_opts).1;
            }
            self.fx_dirty = false;
        }
        if apply {
            let o = self.fx_opts.clone();
            self.apply_fixes(o);
            if let Some(doc) = self.docs.get(self.active) {
                self.fx_changes = fixer::fix(&doc.text, &self.fx_opts).1;
            }
        }
        if undo {
            self.undo_fix();
        }
        if close {
            *open = false;
        }
    }

    fn dlg_formulas(&mut self, ctx: &egui::Context, open: &mut bool) {
        let screen = ctx.screen_rect();
        let mut goto: Option<usize> = None;
        let mut rerun = false;
        let mut to_fixer = false;
        let result = self.fm_result.clone();
        egui::Window::new("Kiểm tra công thức")
            .open(open)
            .collapsible(false)
            .resizable(true)
            .default_width((screen.width() * 0.7).clamp(340.0, 560.0))
            .default_height((screen.height() * 0.6).clamp(240.0, 480.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                match &result {
                    None => {
                        ui.label("Chưa quét.");
                    }
                    Some((total, issues)) => {
                        if issues.is_empty() {
                            ui.label(RichText::new(format!("Đã quét {total} công thức — không phát hiện lỗi.")).color(Color32::from_rgb(40, 160, 90)));
                        } else {
                            ui.label(RichText::new(format!("Đã quét {total} công thức — {} vấn đề. Bấm vào dòng để nhảy tới vị trí.", issues.len())).color(AMBER));
                        }
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical().auto_shrink([false, false]).max_height(ui.available_height() - 40.0).show(ui, |ui| {
                            for it in issues {
                                let r = ui.add(egui::Label::new(RichText::new(format!("Dòng {}: {}", it.line, it.msg)).strong().color(ui.visuals().text_color())).sense(egui::Sense::click()));
                                if r.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                    goto = Some(it.line);
                                }
                                ui.label(RichText::new(format!("    {}", it.snippet)).font(FontId::monospace(12.0)).color(ui.visuals().weak_text_color()));
                                ui.add_space(3.0);
                            }
                        });
                    }
                }
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    if ui.button("Quét lại").clicked() {
                        rerun = true;
                    }
                    if ui.button("Sửa lỗi tự động…").clicked() {
                        to_fixer = true;
                    }
                });
            });
        if let Some(l) = goto {
            self.goto_line(l);
        }
        if rerun {
            self.run_formula_check();
        }
        if to_fixer {
            self.open_fixer();
        }
    }

    fn dlg_stats(&mut self, ctx: &egui::Context, open: &mut bool) {
        let Some(doc) = self.docs.get(self.active) else { return };
        let mut st = Stats::default();
        count_blocks(&doc.ast, &mut st);
        let words = doc.text.split_whitespace().count();
        let chars = doc.text.chars().count();
        let chars_ns = doc.text.chars().filter(|c| !c.is_whitespace()).count();
        let lines = doc.text.split('\n').count();
        let (formulas, _) = fixer::check_formulas(&doc.text);
        let mins = (words as f32 / 200.0).ceil() as usize;
        egui::Window::new("Thống kê tài liệu")
            .open(open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("stats-grid").num_columns(2).spacing([24.0, 5.0]).show(ui, |ui| {
                    let mut row = |a: &str, b: String| {
                        ui.label(a);
                        ui.label(RichText::new(b).strong().color(ui.visuals().text_color()));
                        ui.end_row();
                    };
                    row("Số từ", words.to_string());
                    row("Số ký tự (cả dấu cách)", chars.to_string());
                    row("Số ký tự (không dấu cách)", chars_ns.to_string());
                    row("Số dòng", lines.to_string());
                    row("Đoạn văn", st.paragraphs.to_string());
                    row("Tiêu đề (H1…H6)", st.headings.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" / "));
                    row("Mục danh sách", st.list_items.to_string());
                    row("Bảng", st.tables.to_string());
                    row("Khối code", st.code_blocks.to_string());
                    row("Trích dẫn", st.quotes.to_string());
                    row("Liên kết", st.links.to_string());
                    row("Ảnh", st.images.to_string());
                    row("Công thức", formulas.to_string());
                    row("Thời gian đọc (ước tính)", format!("{} phút", mins.max(1)));
                });
            });
    }

    fn dlg_text(&mut self, ctx: &egui::Context, open: &mut bool, title: &str, body: &str) {
        let screen = ctx.screen_rect();
        egui::Window::new(title)
            .open(open)
            .collapsible(false)
            .resizable(true)
            .default_width((screen.width() * 0.7).clamp(320.0, 560.0))
            .default_height((screen.height() * 0.7).clamp(240.0, 560.0))
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for line in body.lines() {
                        if let Some(h) = line.strip_prefix("# ") {
                            ui.add_space(6.0);
                            ui.label(RichText::new(h).strong().size(15.0).color(ui.visuals().text_color()));
                        } else if line.is_empty() {
                            ui.add_space(3.0);
                        } else {
                            ui.label(RichText::new(line).font(FontId::monospace(12.5)));
                        }
                    }
                });
            });
    }

    fn dlg_about(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new("Giới thiệu")
            .open(open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label(RichText::new("Markdown Tabs").size(20.0).strong().color(ui.visuals().text_color()));
                ui.label(format!("Phiên bản {VERSION}"));
                ui.add_space(6.0);
                ui.label("Trình xem & soạn thảo Markdown đa tab (Rust + egui).");
                ui.label("Công thức LaTeX, xuất Word (.docx) chuẩn OOXML, sửa lỗi font/MathType.");
                ui.add_space(6.0);
                ui.label(RichText::new("Bảng mã TCVN3/VNI: dữ liệu từ thư viện vietnamese-conversion (MIT).").small().color(ui.visuals().weak_text_color()));
            });
    }

    pub fn ui_command_palette(&mut self, ctx: &egui::Context) {
        if !self.cmd_open {
            return;
        }
        let mut open = self.cmd_open;
        let mut run_id: Option<&'static str> = None;
        egui::Window::new("Bảng lệnh nhanh (Ctrl+Shift+P)")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(440.0)
            .anchor(egui::Align2::CENTER_TOP, [0.0, 72.0])
            .show(ctx, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.cmd_query)
                        .hint_text("Gõ lệnh (vd: xuat word, pdf, md, html, in, sang, toi, zen, wrap)…")
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(6.0);
                let q = self.cmd_query.trim().to_lowercase();
                let items: &[(&'static str, &'static str, &'static str)] = &[
                    ("word", "Xuất Word (.docx)", "xuat word docx"),
                    ("pdf", "Xuất PDF (.pdf)", "xuat pdf"),
                    ("md", "Xuất Markdown (.md)", "xuat md markdown"),
                    ("html", "Xuất HTML (.html) độc lập", "xuat html"),
                    ("print", "In tài liệu (Ctrl+P)", "in print"),
                    ("fixer", "Sửa lỗi trình bày & font… (Ctrl+Shift+F)", "sua loi font tcvn3 vni mathtype"),
                    ("theme", "Chuyển giao diện Sáng / Tối", "sang toi dark light theme"),
                    ("zen", "Bật / Tắt chế độ Tập trung (Zen)", "zen tap trung toan man hinh"),
                    ("wrap", "Bật / Tắt xuống dòng mềm (Soft Wrap)", "wrap xuong dong mem"),
                    ("preview", "Chế độ xem: Xem trước (Ctrl+2)", "xem truoc preview"),
                    ("split", "Chế độ xem: Chia đôi (Ctrl+3)", "chia doi split"),
                    ("edit", "Chế độ xem: Soạn thảo (Ctrl+1)", "soan thao edit"),
                    ("toc", "Hiện / Ẩn thanh Mục lục", "muc luc toc sidebar"),
                    ("find", "Tìm & Thay thế (Ctrl+F)", "tim thay the find replace"),
                ];
                egui::ScrollArea::vertical().max_height(260.0).show(ui, |ui| {
                    for (id, label, keys) in items {
                        if !q.is_empty() && !label.to_lowercase().contains(&q) && !keys.contains(&q) {
                            continue;
                        }
                        if ui.selectable_label(false, *label).clicked() {
                            run_id = Some(*id);
                        }
                    }
                });
            });
        self.cmd_open = open;
        if let Some(id) = run_id {
            self.cmd_open = false;
            match id {
                "word" => self.export_docx(),
                "pdf" | "print" => self.export_pdf(ctx),
                "md" => self.export_md(),
                "html" => self.export_html(),
                "fixer" => self.open_fixer(),
                "theme" => {
                    self.dark = !self.dark;
                    apply_theme(ctx, self.dark);
                    self.persist_session();
                }
                "zen" => self.zen_mode = !self.zen_mode,
                "wrap" => {
                    self.word_wrap = !self.word_wrap;
                    self.persist_session();
                }
                "preview" => self.view = ViewMode::Preview,
                "split" => self.view = ViewMode::Split,
                "edit" => self.view = ViewMode::Edit,
                "toc" => self.show_toc = !self.show_toc,
                "find" => self.find_open = !self.find_open,
                _ => {}
            }
        }
    }
}


/// Markdown -> HTML day du (cong thuc = MathML, in duoc).
pub fn html_from_markdown(text: &str, title: &str, auto_print: bool, doc_dir: &Path) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_TASKLISTS);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_MATH);
    let parser = Parser::new_ext(text, opts).map(|ev| match ev {
        Event::InlineMath(t) => Event::Html(math::to_mathml(&t, false).into()),
        Event::DisplayMath(t) => Event::Html(math::to_mathml(&t, true).into()),
        // anh cuc bo -> data URI de tep HTML / ban in PDF mang di dau cung hien anh
        Event::Start(Tag::Image { link_type, dest_url, title, id }) => {
            let nd = crate::media::embed_data_uri(&dest_url, doc_dir).map(pulldown_cmark::CowStr::from).unwrap_or(dest_url);
            Event::Start(Tag::Image { link_type, dest_url: nd, title, id })
        }
        Event::Html(h) => Event::Html(crate::media::embed_html_img_src(&h, doc_dir).into()),
        Event::InlineHtml(h) => Event::InlineHtml(crate::media::embed_html_img_src(&h, doc_dir).into()),
        e => e,
    });
    let mut body = String::new();
    pulldown_cmark::html::push_html(&mut body, parser);
    // bang cuon ngang trong khung rieng khi qua rong (xem tren man hinh)
    let body = body.replace("<table", "<div class=\"tw\"><table").replace("</table>", "</table></div>");
    let script = if auto_print { "<script>window.addEventListener('load',function(){setTimeout(function(){window.print()},500)})</script>" } else { "" };
    let css = "body{font-family:'Segoe UI',Arial,sans-serif;max-width:900px;margin:0 auto;padding:24px;line-height:1.65;color:#1f2328}\
h1,h2{border-bottom:1px solid #d8dee4;padding-bottom:.3em}\
.tw{overflow-x:auto;margin:0 0 1em}\
table{border-collapse:collapse;min-width:50%}\
th,td{border:1px solid #c5ccd6;padding:5px 12px;vertical-align:top}\
th{background:#e9f0fb;font-weight:600}\
tbody tr:nth-child(even) td{background:#f7f9fc}\
code{background:#f0f0f0;padding:1px 4px;border-radius:4px}\
pre{background:#f6f8fa;padding:12px;overflow-x:auto;border-radius:6px}\
pre code{background:none;padding:0}\
blockquote{border-left:4px solid #e8930c;margin-left:0;padding-left:14px;color:#57606a}\
img{max-width:100%;height:auto}\
math{font-size:1.15em}math[display=block]{margin:.9em 0}\
@media print{body{max-width:none;padding:0}pre{white-space:pre-wrap}\
*{-webkit-print-color-adjust:exact;print-color-adjust:exact}\
tr,img,pre,blockquote,math[display=block]{break-inside:avoid}thead{display:table-header-group}h1,h2,h3{break-after:avoid}.tw{overflow:visible}}";
    format!(
        "<!DOCTYPE html><html lang='vi'><head><meta charset='utf-8'><meta name='viewport' content='width=device-width,initial-scale=1'><title>{}</title><style>{css}</style></head><body>{body}{script}</body></html>",
        docx::xml_escape(title)
    )
}

#[cfg(test)]
mod html_tests {
    use super::*;

    #[test]
    fn html_has_mathml() {
        let h = html_from_markdown("Cho $x^2$ và\n\n$$\\frac{a}{b}$$\n", "T", false, Path::new("."));
        assert!(h.contains("<math"), "{h}");
        assert!(h.contains("<mfrac>"), "{h}");
    }

    /// MDT_HTML_IN -> MDT_HTML_OUT de kiem tra bang trinh duyet that
    #[test]
    fn html_sample_file() {
        let (Ok(i), Ok(o)) = (std::env::var("MDT_HTML_IN"), std::env::var("MDT_HTML_OUT")) else { return };
        let t = std::fs::read_to_string(&i).unwrap();
        let dir = Path::new(&i).parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
        std::fs::write(o, html_from_markdown(&t, "Mau", false, &dir)).unwrap();
    }

    #[test]
    fn html_embeds_local_image_and_table_align() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
        let md = "![Logo](icon-256.png)\n\n| a | b |\n|:--|--:|\n| 1 | 2 |\n";
        let h = html_from_markdown(md, "T", false, &dir);
        assert!(h.contains("src=\"data:image/png;base64,"), "anh phai duoc nhung: {}", &h[..h.len().min(400)]);
        assert!(h.contains("text-align: right"), "can le cot phai duoc giu");
        assert!(h.contains("<div class=\"tw\"><table"));
    }
}

fn docx_today() -> String {
    // ngay UTC dang dd/mm/yyyy (khong them phu thuoc chrono)
    let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let z = secs.div_euclid(86400) + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{d:02}/{m:02}/{y}")
}

const SHORTCUTS: &str = "# Tệp
Ctrl+N          Tài liệu mới
Ctrl+O          Mở file
Ctrl+S          Lưu
Ctrl+Shift+S    Lưu thành
Ctrl+W          Đóng tab
Ctrl+Shift+T    Mở lại tab đã đóng
Ctrl+P          In / Lưu PDF
# Soạn thảo
Ctrl+B          Đậm
Ctrl+I          Nghiêng
Ctrl+K          Chèn liên kết
Ctrl+F          Tìm kiếm
Ctrl+H          Thay thế
Ctrl+Shift+F    Sửa lỗi trình bày / font / công thức
# Xem
Ctrl+1          Chỉ soạn thảo
Ctrl+2          Chỉ xem trước
Ctrl+3          Chia đôi
Ctrl++ / Ctrl+- Phóng to / thu nhỏ
Ctrl+0          Đặt lại 100%
F11             Toàn màn hình
F1              Bảng phím tắt này
Esc             Đóng hộp thoại";

const SYNTAX: &str = "# Markdown
# Tiêu đề, **đậm**, *nghiêng*, ~~gạch~~, `code`
- danh sách, 1. danh sách số, - [x] việc đã xong
> trích dẫn, --- đường kẻ, | bảng | với | dòng phân cách |
[liên kết](url), ![ảnh](duong-dan.png), [^1] chú thích
# Công thức
Trong dòng: $x^2 + y^2 = z^2$     Riêng dòng: $$ ... $$
\\frac{a}{b}  \\sqrt{x}  \\sqrt[n]{x}  x^{2}  x_{i}  x_i^2
\\sum_{i=1}^{n}  \\prod  \\int_a^b  \\lim_{x \\to 0}  \\oint  \\iint
\\left( ... \\right)  \\left[ ... \\right|  \\left\\{ ... \\right.
\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}   pmatrix bmatrix vmatrix
\\begin{cases} ... \\end{cases}   \\begin{aligned} ... \\end{aligned}
\\vec{v} \\hat{x} \\bar{x} \\dot{x} \\ddot{x} \\tilde{x} \\overline{AB} \\underline{x}
\\overbrace{...}^{n}  \\underbrace{...}_{n}  \\overset{a}{=}  \\xrightarrow{f}
\\mathbb{R}  \\mathbf{x}  \\mathrm{d}  \\text{văn bản}  \\operatorname{tên}
\\alpha \\beta \\gamma \\delta \\theta \\lambda \\mu \\pi \\sigma \\omega \\Delta \\Sigma \\Omega
\\le \\ge \\ne \\approx \\equiv \\propto \\in \\notin \\subset \\cup \\cap
\\to \\Rightarrow \\Leftrightarrow \\mapsto \\infty \\partial \\nabla \\pm \\times \\cdot \\div
\\sin \\cos \\tan \\log \\ln \\exp \\lim \\max \\min \\det
Khoảng cách: \\, \\: \\; \\quad \\qquad
# Xuất Word
Công thức được xuất thành Equation (OMML) của Word — bấm vào để sửa trong Word.";


#[cfg(test)]
mod helper_tests {
    use super::*;

    #[test]
    fn slug_and_decode() {
        assert_eq!(slugify("Phần 6: Kết luận!"), "phần-6-kết-luận");
        assert_eq!(percent_decode("ph%E1%BA%A7n-6"), "phần-6");
    }

    #[test]
    fn table_alignment() {
        let l: Vec<String> = ["| a | bbb |", "|---|:-:|", "| long cell | x |"].iter().map(|s| s.to_string()).collect();
        let r = align_table_lines(&l);
        assert_eq!(r[0], "| a         | bbb |");
        assert_eq!(r[1], "| --------- | :-: |");
        assert_eq!(r[2], "| long cell |  x  |");
        // chay lai khong doi
        assert_eq!(align_table_lines(&r), r);
    }

    #[test]
    fn plain_text_export() {
        let ast = crate::parse_markdown("# T\n\nMột đoạn **đậm** và $x^2$.\n\n- a\n- b\n");
        let mut out = String::new();
        ast_to_text(&ast, &mut out, 0);
        assert!(out.contains("T\n\nMột đoạn đậm và x^2."), "{out}");
        assert!(out.contains("- a\n- b"), "{out}");
    }
}
