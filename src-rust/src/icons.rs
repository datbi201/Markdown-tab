//! Bieu tuong ve bang net vector (khong can font hay thu vien ngoai).
//! Du lieu la cac path SVG khung 24x24 (xem `icons_data.rs`, sinh boi tools/gen_icons.py).

use crate::icons_data::ICON_DATA;
use eframe::egui::{self, epaint::PathShape, Color32, Pos2, Rect, Shape, Stroke};
use std::collections::HashMap;
use std::sync::OnceLock;

type Poly = (Vec<[f32; 2]>, bool); // (diem, khep kin)

enum Tok {
    Cmd(char),
    Num(f32),
}

fn tokenize(d: &str) -> Vec<Tok> {
    let b: Vec<char> = d.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if c.is_ascii_alphabetic() {
            out.push(Tok::Cmd(c));
            i += 1;
        } else if c == '-' || c == '.' || c.is_ascii_digit() {
            let s = i;
            if b[i] == '-' {
                i += 1;
            }
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            if i < b.len() && b[i] == '.' {
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let t: String = b[s..i].iter().collect();
            out.push(Tok::Num(t.parse::<f32>().unwrap_or(0.0)));
        } else {
            i += 1;
        }
    }
    out
}

fn arc(out: &mut Vec<[f32; 2]>, p1: [f32; 2], mut rx: f32, mut ry: f32, rot: f32, fa: bool, fs: bool, p2: [f32; 2]) {
    if rx.abs() < 1e-6 || ry.abs() < 1e-6 || (p1[0] == p2[0] && p1[1] == p2[1]) {
        out.push(p2);
        return;
    }
    let phi = rot.to_radians();
    let (s, c) = phi.sin_cos();
    let dx = (p1[0] - p2[0]) / 2.0;
    let dy = (p1[1] - p2[1]) / 2.0;
    let x1p = c * dx + s * dy;
    let y1p = -s * dx + c * dy;
    rx = rx.abs();
    ry = ry.abs();
    let lam = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lam > 1.0 {
        let k = lam.sqrt();
        rx *= k;
        ry *= k;
    }
    let num = rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p;
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let mut co = (num / den).max(0.0).sqrt();
    if fa == fs {
        co = -co;
    }
    let cxp = co * rx * y1p / ry;
    let cyp = -co * ry * x1p / rx;
    let cx = c * cxp - s * cyp + (p1[0] + p2[0]) / 2.0;
    let cy = s * cxp + c * cyp + (p1[1] + p2[1]) / 2.0;
    let ang = |ux: f32, uy: f32, vx: f32, vy: f32| (ux * vy - uy * vx).atan2(ux * vx + uy * vy);
    let th1 = ang(1.0, 0.0, (x1p - cxp) / rx, (y1p - cyp) / ry);
    let mut dth = ang((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry);
    if !fs && dth > 0.0 {
        dth -= std::f32::consts::TAU;
    } else if fs && dth < 0.0 {
        dth += std::f32::consts::TAU;
    }
    let n = ((dth.abs() / (std::f32::consts::PI / 14.0)).ceil() as usize).max(2);
    for k in 1..=n {
        let t = th1 + dth * k as f32 / n as f32;
        let (st, ct) = t.sin_cos();
        out.push([cx + c * rx * ct - s * ry * st, cy + s * rx * ct + c * ry * st]);
    }
}

fn parse_path(d: &str) -> Vec<Poly> {
    let toks = tokenize(d);
    let mut i = 0;
    let mut polys: Vec<Poly> = Vec::new();
    let mut pts: Vec<[f32; 2]> = Vec::new();
    let mut cur = [0.0f32, 0.0];
    let mut start = [0.0f32, 0.0];
    let mut cmd = ' ';
    let mut ctrl: Option<[f32; 2]> = None;
    macro_rules! num {
        () => {{
            match toks.get(i) {
                Some(Tok::Num(n)) => {
                    i += 1;
                    *n
                }
                _ => break,
            }
        }};
    }
    while i < toks.len() {
        if let Tok::Cmd(c) = toks[i] {
            cmd = c;
            i += 1;
            if c == 'Z' || c == 'z' {
                if !pts.is_empty() {
                    polys.push((std::mem::take(&mut pts), true));
                }
                cur = start;
                continue;
            }
        }
        let rel = cmd.is_ascii_lowercase();
        let (ox, oy) = if rel { (cur[0], cur[1]) } else { (0.0, 0.0) };
        let last_ctrl = ctrl;
        let mut new_ctrl: Option<[f32; 2]> = None;
        match cmd.to_ascii_uppercase() {
            'M' => {
                let x = num!();
                let y = num!();
                if !pts.is_empty() {
                    polys.push((std::mem::take(&mut pts), false));
                }
                cur = [ox + x, oy + y];
                start = cur;
                pts.push(cur);
                cmd = if rel { 'l' } else { 'L' };
            }
            'L' => {
                let x = num!();
                let y = num!();
                if pts.is_empty() {
                    pts.push(cur);
                }
                cur = [ox + x, oy + y];
                pts.push(cur);
            }
            'H' => {
                let x = num!();
                if pts.is_empty() {
                    pts.push(cur);
                }
                cur = [ox + x, cur[1]];
                pts.push(cur);
            }
            'V' => {
                let y = num!();
                if pts.is_empty() {
                    pts.push(cur);
                }
                cur = [cur[0], oy + y];
                pts.push(cur);
            }
            'C' | 'S' => {
                let smooth = cmd.eq_ignore_ascii_case(&'s');
                let (x1, y1) = if smooth { (0.0, 0.0) } else { (num!(), num!()) };
                let (x2, y2, x, y) = (num!(), num!(), num!(), num!());
                if pts.is_empty() {
                    pts.push(cur);
                }
                let p0 = cur;
                // 'S': diem dieu khien dau la phan chieu cua diem dieu khien cuoi cua doan cong truoc
                let p1 = if smooth {
                    match last_ctrl {
                        Some(c) => [2.0 * cur[0] - c[0], 2.0 * cur[1] - c[1]],
                        None => cur,
                    }
                } else {
                    [ox + x1, oy + y1]
                };
                let (p2, p3) = ([ox + x2, oy + y2], [ox + x, oy + y]);
                new_ctrl = Some(p2);
                for k in 1..=12 {
                    let t = k as f32 / 12.0;
                    let u = 1.0 - t;
                    pts.push([
                        u * u * u * p0[0] + 3.0 * u * u * t * p1[0] + 3.0 * u * t * t * p2[0] + t * t * t * p3[0],
                        u * u * u * p0[1] + 3.0 * u * u * t * p1[1] + 3.0 * u * t * t * p2[1] + t * t * t * p3[1],
                    ]);
                }
                cur = p3;
            }
            'Q' => {
                let (x1, y1, x, y) = (num!(), num!(), num!(), num!());
                if pts.is_empty() {
                    pts.push(cur);
                }
                let p0 = cur;
                let (p1, p2) = ([ox + x1, oy + y1], [ox + x, oy + y]);
                for k in 1..=10 {
                    let t = k as f32 / 10.0;
                    let u = 1.0 - t;
                    pts.push([u * u * p0[0] + 2.0 * u * t * p1[0] + t * t * p2[0], u * u * p0[1] + 2.0 * u * t * p1[1] + t * t * p2[1]]);
                }
                cur = p2;
            }
            'A' => {
                let (rx, ry, rot, fa, fs, x, y) = (num!(), num!(), num!(), num!(), num!(), num!(), num!());
                if pts.is_empty() {
                    pts.push(cur);
                }
                let p2 = [ox + x, oy + y];
                arc(&mut pts, cur, rx, ry, rot, fa != 0.0, fs != 0.0, p2);
                cur = p2;
            }
            _ => {
                i += 1;
            }
        }
        ctrl = new_ctrl;
    }
    if !pts.is_empty() {
        polys.push((pts, false));
    }
    polys
}

fn cache() -> &'static HashMap<&'static str, Vec<Poly>> {
    static C: OnceLock<HashMap<&'static str, Vec<Poly>>> = OnceLock::new();
    C.get_or_init(|| ICON_DATA.iter().map(|(n, d)| (*n, parse_path(d))).collect())
}

/// Ve bieu tuong `name` vao hinh chu nhat `rect` (vuong), mau `color`.
pub fn paint(painter: &egui::Painter, rect: Rect, name: &str, color: Color32) {
    let Some(polys) = cache().get(name) else { return };
    let scale = rect.width().min(rect.height()) / 24.0;
    let origin = rect.center() - egui::vec2(12.0 * scale, 12.0 * scale);
    let stroke = Stroke::new((1.75 * scale).max(1.2), color);
    for (pts, closed) in polys {
        let points: Vec<Pos2> = pts.iter().map(|p| origin + egui::vec2(p[0] * scale, p[1] * scale)).collect();
        if points.len() < 2 {
            continue;
        }
        if *closed {
            painter.add(Shape::Path(PathShape::closed_line(points, stroke)));
        } else {
            painter.add(Shape::Path(PathShape::line(points, stroke)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_parses_inside_frame() {
        for (name, d) in ICON_DATA {
            let polys = parse_path(d);
            assert!(!polys.is_empty(), "{name}");
            for (pts, _) in &polys {
                for p in pts {
                    assert!(p[0] > -1.0 && p[0] < 25.0 && p[1] > -1.0 && p[1] < 25.0, "{name}: diem ngoai khung {p:?}");
                    assert!(p[0].is_finite() && p[1].is_finite(), "{name}");
                }
            }
        }
    }
}
