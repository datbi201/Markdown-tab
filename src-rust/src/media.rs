//! Anh va noi dung nhung: tim anh (tep cuc bo, data:, http/https), chuan hoa dinh dang
//! cho Word/HTML, tai anh tu mang (xem truoc va khi xuat), doi khoi HTML sang doan van.

use crate::{Block, Span};
use std::collections::HashMap;
use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

const MAX_REMOTE_BYTES: u64 = 25 * 1024 * 1024;

// ------------------------------------------------------------------ ma hoa
pub fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() + 0 && b[i + 1].is_ascii_hexdigit() && b[i + 2].is_ascii_hexdigit() {
            let h = |c: u8| (c as char).to_digit(16).unwrap_or(0) as u8;
            out.push(h(b[i + 1]) * 16 + h(b[i + 2]));
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len() * 4 / 3 + 4);
    for ch in data.chunks(3) {
        let n = (ch[0] as u32) << 16 | (*ch.get(1).unwrap_or(&0) as u32) << 8 | *ch.get(2).unwrap_or(&0) as u32;
        out.push(B64[(n >> 18) as usize & 63] as char);
        out.push(B64[(n >> 12) as usize & 63] as char);
        out.push(if ch.len() > 1 { B64[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if ch.len() > 2 { B64[n as usize & 63] as char } else { '=' });
    }
    out
}

pub fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0u32);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        acc = acc << 6 | v as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

// ----------------------------------------------------------------- nguon anh
pub enum Source {
    Data(Vec<u8>),
    Local(PathBuf),
    Remote(String),
}

pub fn is_remote(dest: &str) -> bool {
    let d = dest.trim();
    d.starts_with("http://") || d.starts_with("https://")
}

/// Xac dinh anh nam o dau. `Local` chi tra ve khi tep thuc su ton tai.
pub fn classify(dest: &str, doc_dir: &Path) -> Option<Source> {
    let d = dest.trim().trim_start_matches('<').trim_end_matches('>').trim();
    if d.is_empty() {
        return None;
    }
    if let Some(rest) = d.strip_prefix("data:") {
        let (head, body) = rest.split_once(',')?;
        if !head.contains("base64") {
            return None;
        }
        return base64_decode(body).map(Source::Data);
    }
    if is_remote(d) {
        return Some(Source::Remote(d.to_string()));
    }
    let mut path_s = d.to_string();
    if let Some(rest) = path_s.strip_prefix("file://") {
        let r = percent_decode(rest);
        let r = r.trim_start_matches('/');
        // file:///C:/x -> C:/x ; file:///home/x -> /home/x (khong ap dung tren Windows)
        path_s = if r.len() > 1 && r.as_bytes()[1] == b':' { r.to_string() } else { format!("/{r}") };
    }
    let candidates = {
        let no_frag = path_s.split('#').next().unwrap_or(&path_s).to_string();
        let no_query = no_frag.split('?').next().unwrap_or(&no_frag).to_string();
        let mut v = vec![path_s.clone(), no_frag.clone(), no_query.clone()];
        v.push(percent_decode(&no_frag));
        v.push(percent_decode(&no_query));
        v
    };
    for c in candidates {
        let p = PathBuf::from(&c);
        let full = if p.is_absolute() { p } else { doc_dir.join(p) };
        if full.is_file() {
            return Some(Source::Local(full));
        }
    }
    None
}

// -------------------------------------------------------------- chuan hoa anh
pub struct Img {
    pub bytes: Vec<u8>,
    /// "png" | "jpg" | "gif"
    pub ext: &'static str,
    pub w: u32,
    pub h: u32,
}

/// PNG/JPEG/GIF giu nguyen; dinh dang khac (WebP, BMP, TIFF...) doi sang PNG de Word doc duoc.
pub fn normalize(bytes: Vec<u8>) -> Option<Img> {
    let fmt = image::guess_format(&bytes).ok()?;
    let keep = match fmt {
        image::ImageFormat::Png => Some("png"),
        image::ImageFormat::Jpeg => Some("jpg"),
        image::ImageFormat::Gif => Some("gif"),
        _ => None,
    };
    if let Some(ext) = keep {
        let (w, h) = image::ImageReader::new(Cursor::new(&bytes)).with_guessed_format().ok()?.into_dimensions().ok()?;
        if w == 0 || h == 0 {
            return None;
        }
        return Some(Img { bytes, ext, w, h });
    }
    let dynimg = image::load_from_memory(&bytes).ok()?;
    let (w, h) = (dynimg.width(), dynimg.height());
    let mut out = Cursor::new(Vec::new());
    dynimg.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(Img { bytes: out.into_inner(), ext: "png", w, h })
}

/// Doc anh (cuc bo / data: / da tai san) thanh `Img` cho Word.
pub fn load_for_export(dest: &str, doc_dir: &Path, remote: &HashMap<String, Vec<u8>>) -> Option<Img> {
    let bytes = match classify(dest, doc_dir)? {
        Source::Data(b) => b,
        Source::Local(p) => std::fs::read(p).ok()?,
        Source::Remote(u) => remote.get(&u)?.clone(),
    };
    normalize(bytes)
}

/// Doi anh cuc bo sang data URI de HTML mang di dau cung hien (tra None neu khong phai tep cuc bo).
pub fn embed_data_uri(dest: &str, doc_dir: &Path) -> Option<String> {
    let Source::Local(p) = classify(dest, doc_dir)? else { return None };
    let bytes = std::fs::read(&p).ok()?;
    let ext = p.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "svg" => "image/svg+xml",
        _ => return None,
    };
    Some(format!("data:{mime};base64,{}", base64_encode(&bytes)))
}

/// Trong doan HTML tho (the <img>), doi `src` tro toi tep cuc bo thanh data URI.
pub fn embed_html_img_src(html: &str, doc_dir: &Path) -> String {
    if !html.to_ascii_lowercase().contains("<img") {
        return html.to_string();
    }
    let low = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while let Some(p) = low[i..].find("<img") {
        let start = i + p;
        let Some(len) = html[start..].find('>') else { break };
        let tag = &html[start..start + len + 1];
        out.push_str(&html[i..start]);
        match attr(tag, "src").and_then(|s| embed_data_uri(&s, doc_dir).map(|d| (s, d))) {
            Some((orig, data)) => out.push_str(&tag.replacen(&orig, &data, 1)),
            None => out.push_str(tag),
        }
        i = start + len + 1;
    }
    out.push_str(&html[i..]);
    out
}

// ------------------------------------------------------------- tai anh tu mang
fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new().timeout(Duration::from_secs(12)).user_agent("MarkdownTabs").build()
}

pub fn fetch_one(url: &str) -> Option<Vec<u8>> {
    let resp = agent().get(url).call().ok()?;
    let mut buf = Vec::new();
    resp.into_reader().take(MAX_REMOTE_BYTES).read_to_end(&mut buf).ok()?;
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

/// Tai song song moi anh http(s) cua tai lieu de nhung vao Word.
pub fn fetch_many(urls: &[String]) -> HashMap<String, Vec<u8>> {
    let mut out = HashMap::new();
    let urls: Vec<&String> = urls.iter().take(80).collect();
    std::thread::scope(|s| {
        let handles: Vec<_> = urls.iter().map(|u| (u, s.spawn(move || fetch_one(u)))).collect();
        for (u, h) in handles {
            if let Ok(Some(b)) = h.join() {
                out.insert((*u).clone(), b);
            }
        }
    });
    out
}

#[derive(Clone)]
pub enum RemoteState {
    Loading,
    Ready(Arc<Vec<u8>>),
    Failed,
}

static REMOTE: OnceLock<Mutex<HashMap<String, RemoteState>>> = OnceLock::new();

/// Trang thai anh tu mang cho xem truoc; lan dau se mo luong nen de tai.
pub fn remote_state(url: &str, repaint: Option<eframe::egui::Context>) -> RemoteState {
    let m = REMOTE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut g = m.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(s) = g.get(url) {
        return s.clone();
    }
    g.insert(url.to_string(), RemoteState::Loading);
    drop(g);
    let u = url.to_string();
    std::thread::spawn(move || {
        let st = match fetch_one(&u) {
            Some(b) => RemoteState::Ready(Arc::new(b)),
            None => RemoteState::Failed,
        };
        if let Some(m) = REMOTE.get() {
            m.lock().unwrap_or_else(|e| e.into_inner()).insert(u, st);
        }
        if let Some(c) = repaint {
            c.request_repaint();
        }
    });
    RemoteState::Loading
}

// ---------------------------------------------------- thu thap anh tu tai lieu
fn spans_images(spans: &[Span], out: &mut Vec<String>) {
    for s in spans {
        if let Some(d) = &s.image {
            if is_remote(d) && !out.contains(d) {
                out.push(d.trim().to_string());
            }
        }
    }
}

pub fn collect_remote_images(blocks: &[Block], out: &mut Vec<String>) {
    for b in blocks {
        match b {
            Block::Paragraph(sp) | Block::Heading(_, sp, _) => spans_images(sp, out),
            Block::List { items, .. } => items.iter().for_each(|it| collect_remote_images(it, out)),
            Block::Quote(inner) => collect_remote_images(inner, out),
            Block::Table { head, rows, .. } => {
                head.iter().for_each(|c| spans_images(c, out));
                rows.iter().for_each(|r| r.iter().for_each(|c| spans_images(c, out)));
            }
            Block::Html(t) => spans_images(&html_to_spans(t), out),
            _ => {}
        }
    }
}

// ----------------------------------------------------------- HTML -> doan van
fn decode_entities(s: &str) -> String {
    s.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let low = tag.to_ascii_lowercase();
    let pat = format!("{name}=");
    let mut from = 0;
    while let Some(pos) = low[from..].find(&pat) {
        let at = from + pos;
        // ky tu dung truoc phai la khoang trang (tranh bat nham data-src=...)
        if at > 0 && !low.as_bytes()[at - 1].is_ascii_whitespace() {
            from = at + pat.len();
            continue;
        }
        let rest = &tag[at + pat.len()..];
        let q = rest.chars().next()?;
        if q == '"' || q == '\'' {
            let end = rest[1..].find(q)?;
            return Some(decode_entities(&rest[1..1 + end]));
        }
        let end = rest.find(|c: char| c.is_whitespace() || c == '>' || c == '/').unwrap_or(rest.len());
        return Some(decode_entities(&rest[..end]));
    }
    None
}

/// Khoi HTML trong Markdown -> cac doan van: giu <br>, <img>, bo cac the con lai.
pub fn html_to_spans(html: &str) -> Vec<Span> {
    let mut spans: Vec<Span> = Vec::new();
    let mut text = String::new();
    let flush = |text: &mut String, spans: &mut Vec<Span>| {
        if !text.is_empty() {
            let t = decode_entities(&text.replace(['\r', '\n'], " "));
            let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
            if !t.is_empty() {
                let lead = text.starts_with(char::is_whitespace) && !spans.is_empty();
                spans.push(Span::plain(if lead { format!(" {t}") } else { t }));
            }
            text.clear();
        }
    };
    let mut i = 0;
    let b = html.as_bytes();
    let mut skip_until: Option<&str> = None;
    while i < b.len() {
        if b[i] == b'<' {
            if html[i..].starts_with("<!--") {
                i = html[i..].find("-->").map(|p| i + p + 3).unwrap_or(b.len());
                continue;
            }
            if let Some(end) = html[i..].find('>') {
                let tag = &html[i + 1..i + end];
                let name: String = tag.trim_start_matches('/').chars().take_while(|c| c.is_ascii_alphanumeric()).collect::<String>().to_ascii_lowercase();
                i += end + 1;
                if let Some(s) = skip_until {
                    if tag.trim_start_matches('/').to_ascii_lowercase().starts_with(s) && tag.starts_with('/') {
                        skip_until = None;
                    }
                    continue;
                }
                match name.as_str() {
                    "script" | "style" if !tag.starts_with('/') => skip_until = Some(if name == "script" { "script" } else { "style" }),
                    "br" => {
                        flush(&mut text, &mut spans);
                        spans.push(Span::plain("\n".into()));
                    }
                    "img" => {
                        flush(&mut text, &mut spans);
                        if let Some(src) = attr(tag, "src") {
                            let mut sp = Span::plain(attr(tag, "alt").unwrap_or_default());
                            sp.image = Some(src);
                            spans.push(sp);
                        }
                    }
                    "p" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" if tag.starts_with('/') || name == "li" => {
                        flush(&mut text, &mut spans);
                        spans.push(Span::plain("\n".into()));
                    }
                    _ => {}
                }
                continue;
            }
        }
        let ch_len = html[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        if skip_until.is_none() {
            text.push_str(&html[i..i + ch_len]);
        }
        i += ch_len;
    }
    flush(&mut text, &mut spans);
    // bo xuong dong thua o dau/cuoi
    while spans.first().map_or(false, |s| s.text == "\n") {
        spans.remove(0);
    }
    while spans.last().map_or(false, |s| s.text == "\n") {
        spans.pop();
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_roundtrip() {
        for s in ["", "a", "ab", "abc", "Xin chào, thế giới!"] {
            let e = base64_encode(s.as_bytes());
            if !s.is_empty() {
                assert_eq!(base64_decode(&e).unwrap(), s.as_bytes());
            }
        }
        assert_eq!(base64_encode(b"Man"), "TWFu");
    }

    #[test]
    fn percent_and_paths() {
        assert_eq!(percent_decode("ph%E1%BA%A7n%20m%E1%BB%9Di.png"), "phần mời.png");
        assert!(matches!(classify("https://x/y.png", Path::new(".")), Some(Source::Remote(_))));
        assert!(classify("khong-co-file.png", Path::new(".")).is_none());
    }

    #[test]
    fn html_img_and_br() {
        let sp = html_to_spans("<p align=\"center\"><img src=\"a b.png\" alt=\"Logo\" width=\"100\"><br>Chú thích &amp; <b>đậm</b></p>");
        assert_eq!(sp[0].image.as_deref(), Some("a b.png"));
        assert_eq!(sp[0].text, "Logo");
        assert!(sp.iter().any(|s| s.text == "\n"));
        assert!(sp.iter().any(|s| s.text.contains("Chú thích & đậm")), "{:?}", sp.iter().map(|s| &s.text).collect::<Vec<_>>());
    }

    #[test]
    fn normalize_png_and_bad() {
        assert!(normalize(vec![1, 2, 3]).is_none());
    }
}
