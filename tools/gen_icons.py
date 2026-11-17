#!/usr/bin/env python3
"""Sinh src-rust/src/icons_data.rs tu bo bieu tuong SVG (khung 24x24, net ve).

Moi bieu tuong la chuoi cac duong path SVG; hinh tron va chu nhat bo goc duoc doi ra path.
Chay: python tools/gen_icons.py   (tu thu muc goc cua kho)
"""
import re
from pathlib import Path

ICONS = {
    "file-plus": '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5M12 11v6M9 14h6"/>',
    "folder-open": '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v1"/><path d="M3 10h18l-1.7 8a2 2 0 0 1-2 1.6H6.7a2 2 0 0 1-2-1.6z"/>',
    "folder": '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
    "clock": '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
    "save": '<path d="M5 4h11l3 3v13H5z"/><path d="M8 4v5h7V4M8 20v-6h8v6"/>',
    "save-plus": '<path d="M5 4h11l3 3v13H5z"/><path d="M12 10v6M9 13h6"/>',
    "layers": '<path d="M12 3l9 5-9 5-9-5z"/><path d="M3 13l9 5 9-5"/>',
    "x": '<path d="M6 6l12 12M18 6L6 18"/>',
    "plus": '<path d="M12 5v14M5 12h14"/>',
    "bold": '<path d="M7 5h6a3.5 3.5 0 0 1 0 7H7zM7 12h7a3.5 3.5 0 0 1 0 7H7z"/>',
    "italic": '<path d="M10 5h8M6 19h8M15 5l-5 14"/>',
    "strike": '<path d="M4 12h16M16 7c0-1.7-1.8-3-4-3s-4 1.3-4 3 1.6 2.4 4 3M8 17c0 1.7 1.8 3 4 3s4-1.3 4-3-1.6-2.4-4-3"/>',
    "code": '<path d="M9 8l-5 4 5 4M15 8l5 4-5 4"/>',
    "heading": '<path d="M6 4v16M18 4v16M6 12h12"/>',
    "list": '<path d="M9 6h11M9 12h11M9 18h11"/><circle cx="4.5" cy="6" r="1"/><circle cx="4.5" cy="12" r="1"/><circle cx="4.5" cy="18" r="1"/>',
    "list-num": '<path d="M10 6h10M10 12h10M10 18h10M4 5l1.6-1v5M4 14h3l-3 3h3"/>',
    "check": '<rect x="4" y="4" width="16" height="16" rx="4"/><path d="M8.5 12l2.5 2.5 5-5"/>',
    "quote": '<path d="M4 18v-6a5 5 0 0 1 5-5M13 18v-6a5 5 0 0 1 5-5"/>',
    "link": '<path d="M10 14a4 4 0 0 0 5.6 0l3-3a4 4 0 0 0-5.6-5.6l-1 1M14 10a4 4 0 0 0-5.6 0l-3 3a4 4 0 0 0 5.6 5.6l1-1"/>',
    "image": '<rect x="3" y="4" width="18" height="16" rx="3"/><circle cx="9" cy="10" r="1.6"/><path d="M4 18l5-5 4 4 3-3 4 4"/>',
    "image-plus": '<path d="M13 4H6a3 3 0 0 0-3 3v10a3 3 0 0 0 3 3h12a3 3 0 0 0 3-3v-5"/><circle cx="8.5" cy="9.5" r="1.5"/><path d="M4 18l5-5 4 4 2-2 5 5M18 3v6M15 6h6"/>',
    "table": '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M3 10h18M3 15h18M9 4v16"/>',
    "codeblock": '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M10 9l-2 3 2 3M14 9l2 3-2 3"/>',
    "minus": '<path d="M4 12h16"/>',
    "sigma": '<path d="M18 5H6l6 7-6 7h12"/>',
    "sigma-block": '<path d="M16 5H8l4 5-4 5h8M4 20h16"/>',
    "omega": '<path d="M5 19h4v-2.3a7 7 0 1 1 6 0V19h4"/>',
    "toc": '<path d="M4 6h16M7 12h13M10 18h10"/>',
    "asterisk": '<path d="M12 4v16M5 8l14 8M19 8L5 16"/>',
    "calendar": '<rect x="4" y="5" width="16" height="15" rx="3"/><path d="M4 10h16M9 3v4M15 3v4"/>',
    "columns": '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M12 4v16"/>',
    "sidebar": '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M9 4v16M5.5 8h.01M5.5 11h.01"/>',
    "edit": '<path d="M4 20l1-4L16 5l3 3L8 19z"/>',
    "eye": '<path d="M2 12s4-7 10-7 10 7 10 7-4 7-10 7S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
    "zoomin": '<circle cx="10.5" cy="10.5" r="6.5"/><path d="M15.5 15.5L20 20M10.5 8v5M8 10.5h5"/>',
    "zoomout": '<circle cx="10.5" cy="10.5" r="6.5"/><path d="M15.5 15.5L20 20M8 10.5h5"/>',
    "search": '<circle cx="11" cy="11" r="6.5"/><path d="M16 16l4.5 4.5"/>',
    "moon": '<path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z"/>',
    "sun": '<circle cx="12" cy="12" r="4"/><path d="M12 3v2M12 19v2M3 12h2M19 12h2M5.6 5.6L7 7M17 17l1.4 1.4M5.6 18.4L7 17M17 7l1.4-1.4"/>',
    "maximize": '<path d="M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5"/>',
    "word": '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5M8 12l1.2 5 1.8-4 1.8 4 1.2-5"/>',
    "globe": '<circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18"/>',
    "printer": '<path d="M7 9V4h10v5M7 17H5a2 2 0 0 1-2-2v-4a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-2"/><rect x="7" y="14" width="10" height="6" rx="1"/>',
    "txt": '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5M8 13h8M8 17h6"/>',
    "copy": '<rect x="9" y="9" width="11" height="11" rx="2.5"/><path d="M5 15V6a2 2 0 0 1 2-2h8"/>',
    "wand": '<path d="M5 19L15 9M14 3.5l1 2.2 2.2 1-2.2 1-1 2.2-1-2.2-2.2-1 2.2-1zM19 13l.7 1.3 1.3.7-1.3.7-.7 1.3-.7-1.3-1.3-.7 1.3-.7z"/>',
    "bolt": '<path d="M13 3L5 14h6l-1 7 8-11h-6z"/>',
    "undo": '<path d="M9 7L4 12l5 5"/><path d="M4 12h9a5 5 0 0 1 0 8h-3"/>',
    "check-circle": '<circle cx="12" cy="12" r="9"/><path d="M8 12.5l2.7 2.7L16 9.5"/>',
    "align": '<path d="M4 6h16M4 10h10M4 14h16M4 18h10"/>',
    "chart": '<path d="M5 20V10M12 20V4M19 20v-7"/>',
    "refresh": '<path d="M20 12a8 8 0 1 1-2.5-5.8M20 4v4h-4"/>',
    "keyboard": '<rect x="3" y="6" width="18" height="12" rx="3"/><path d="M7 10h.01M11 10h.01M15 10h.01M7 14h10"/>',
    "book": '<path d="M5 4h10a4 4 0 0 1 4 4v12H9a4 4 0 0 1-4-4z"/><path d="M5 16a4 4 0 0 1 4-4h10"/>',
    "info": '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 8h.01"/>',
    "chev": '<path d="M6 9l6 6 6-6"/>',
    "palette": '<path d="M12 3a9 9 0 1 0 0 18c1.2 0 1.8-.8 1.5-1.8-.3-1 .3-2 1.5-2H17a4 4 0 0 0 4-4c0-5-4-10-9-10z"/><circle cx="8" cy="11" r="1"/><circle cx="12" cy="7.5" r="1"/><circle cx="16" cy="11" r="1"/>',
    "grid": '<rect x="4" y="4" width="7" height="7" rx="2"/><rect x="13" y="4" width="7" height="7" rx="2"/><rect x="4" y="13" width="7" height="7" rx="2"/><rect x="13" y="13" width="7" height="7" rx="2"/>',
    "file": '<path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/><path d="M14 3v5h5"/>',
    "power": '<path d="M12 3v8M6.5 6.5a8 8 0 1 0 11 0"/>',
}


def num_list(s):
    return [float(x) for x in re.findall(r"-?\d*\.?\d+", s)]


def circle(cx, cy, r):
    return f"M{cx - r} {cy}a{r} {r} 0 1 0 {2 * r} 0a{r} {r} 0 1 0 {-2 * r} 0z"


def rect(x, y, w, h, rx):
    rx = min(rx, w / 2, h / 2)
    if rx <= 0:
        return f"M{x} {y}h{w}v{h}h{-w}z"
    return (f"M{x + rx} {y}h{w - 2 * rx}a{rx} {rx} 0 0 1 {rx} {rx}v{h - 2 * rx}a{rx} {rx} 0 0 1 {-rx} {rx}"
            f"h{-(w - 2 * rx)}a{rx} {rx} 0 0 1 {-rx} {-rx}v{-(h - 2 * rx)}a{rx} {rx} 0 0 1 {rx} {-rx}z")


def to_path(frag):
    out = []
    for m in re.finditer(r"<(path|circle|rect)\b([^>]*)/>", frag):
        tag, attrs = m.group(1), dict(re.findall(r'(\w+)="([^"]*)"', m.group(2)))
        if tag == "path":
            out.append(attrs["d"])
        elif tag == "circle":
            out.append(circle(float(attrs["cx"]), float(attrs["cy"]), float(attrs["r"])))
        else:
            out.append(rect(float(attrs["x"]), float(attrs["y"]), float(attrs["width"]), float(attrs["height"]), float(attrs.get("rx", 0))))
    return " ".join(out)


def main():
    root = Path(__file__).resolve().parent.parent
    lines = ["// TU DONG SINH boi tools/gen_icons.py - khong sua tay.", "pub const ICON_DATA: &[(&str, &str)] = &["]
    for name, frag in ICONS.items():
        lines.append(f'    ("{name}", "{to_path(frag)}"),')
    lines.append("];")
    dst = root / "src-rust" / "src" / "icons_data.rs"
    dst.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Da ghi {dst} ({len(ICONS)} bieu tuong)")


if __name__ == "__main__":
    main()
