# Markdown Tabs

TrÃ¬nh xem vÃ  soáº¡n tháº£o Markdown nhiá»u tab **native trÃªn Windows**, viáº¿t báº±ng Rust (`eframe`/`egui`). á»¨ng dá»¥ng Ä‘Æ°á»£c tá»‘i Æ°u cho tÃ i liá»‡u ká»¹ thuáº­t: hiá»ƒn thá»‹ cÃ´ng thá»©c toÃ¡n LaTeX hoÃ n toÃ n offline, sá»­a lá»—i font tiáº¿ng Viá»‡t cÅ© (TCVN3, VNI, Symbol/MathType) vÃ  xuáº¥t Word (`.docx`) chuáº©n phÆ°Æ¡ng trÃ¬nh OMML.

> **PhiÃªn báº£n hiá»‡n táº¡i:** `v0.7.3` Â· **Ná»n táº£ng:** Windows 64-bit (`x86_64-pc-windows-msvc`) Â· **Báº£n quyá»n:** MIT

## TÃ­nh nÄƒng ná»•i báº­t

- **Giao diá»‡n 2 táº§ng hiá»‡n Ä‘áº¡i (SÃ¡ng & Tá»‘i):** Thanh Ä‘á»‰nh há»£p nháº¥t Tab tÃ i liá»‡u, chuyá»ƒn nhanh 3 cháº¿ Ä‘á»™ xem (**Soáº¡n tháº£o** / **Xem trÆ°á»›c** / **Chia Ä‘Ã´i**), Ä‘á»•i **4 mÃ u chá»§ Ä‘áº¡o** (Xanh dÆ°Æ¡ng, Lá»¥c ngá»c, Cam san hÃ´, TÃ­m), nÃºt SÃ¡ng/Tá»‘i vÃ  cháº¿ Ä‘á»™ **Táº­p trung (Zen)**.
- **Máº·c Ä‘á»‹nh Xem trÆ°á»›c dáº¡ng trang giáº¥y:** Ná»™i dung cÄƒn giá»¯a trÃªn ná»n 4 táº§ng dá»‹u máº¯t, há»— trá»£ **chÃº thÃ­ch áº£nh**, **báº£ng cÄƒn lá»**, **khá»‘i mÃ£ cÃ³ nÃºt Sao chÃ©p** vÃ  **há»‡ cÃ´ng thá»©c LaTeX** (phÃ¢n sá»‘, cÄƒn, tÃ­ch phÃ¢n, ma tráº­n, `cases`, `aligned`â€¦) render trá»±c tiáº¿p khÃ´ng cáº§n Internet.
- **Xuáº¥t tÃ i liá»‡u Ä‘a Ä‘á»‹nh dáº¡ng (1 báº¥m):** Cá»¥m 5 nÃºt **Word (`.docx`)**, **PDF**, **Markdown (`.md`)**, **HTML** Ä‘á»™c láº­p (nhÃºng sáºµn áº£nh & MathML) vÃ  **In (`Ctrl+P`)** Ä‘áº·t ngay trÃªn thanh cÃ´ng cá»¥. File `.docx` xuáº¥t ra Ä‘áº¡t chuáº©n OOXML, phÆ°Æ¡ng trÃ¬nh chuyá»ƒn thÃ nh **Word Equation (OMML)** chá»‰nh sá»­a trá»±c tiáº¿p Ä‘Æ°á»£c trong Microsoft Word.
- **Sá»­a lá»—i trÃ¬nh bÃ y & font tiáº¿ng Viá»‡t cÅ© (`Ctrl+Shift+F`):** Tá»± Ä‘á»™ng khÃ´i phá»¥c chá»¯ UTF-8 vá»¡ mÃ£, chuyá»ƒn **TCVN3 (`.VnTime`)** vÃ  **VNI** sang Unicode chuáº©n, Ä‘á»•i kÃ½ tá»± **Symbol/MathType** (`U+F0xx`) vá» kÃ½ hiá»‡u toÃ¡n há»c chuáº©n, chuáº©n hÃ³a dáº¥u cÃ´ng thá»©c `$...$` / `$$...$$`, báº£ng vÃ  danh sÃ¡ch dÃ¡n tá»« Word/PDF.
- **Soáº¡n tháº£o & Äiá»u hÆ°á»›ng thÃ´ng minh:**
  - **Thanh bÃªn 2 tab:** Chuyá»ƒn nhanh giá»¯a **Má»¥c lá»¥c** (cÃ³ Ã´ lá»c tiÃªu Ä‘á») vÃ  **ThÆ° má»¥c** (liá»‡t kÃª tá»‡p Ä‘ang má»Ÿ & toÃ n bá»™ tá»‡p `.md` cÃ¹ng thÆ° má»¥c), kÃ©o chá»‰nh Ä‘á»™ rá»™ng linh hoáº¡t.
  - **Cuá»™n Ä‘á»“ng bá»™ 2 chiá»u** giá»¯a Soáº¡n tháº£o vÃ  Xem trÆ°á»›c á»Ÿ cháº¿ Ä‘á»™ Chia Ä‘Ã´i, kÃ¨m nÃºt báº­t/táº¯t **Soft Wrap** (tá»± xuá»‘ng dÃ²ng má»m) vÃ  cá»¥m **Zoom (`âˆ’` / `100%` / `+` + `Ctrl+Cuá»™n chuá»™t`)** ngay trÃªn thanh tráº¡ng thÃ¡i.
  - **Báº£ng lá»‡nh nhanh (`Ctrl+Shift+P`)** há»— trá»£ gÃµ tiáº¿ng Viá»‡t khÃ´ng dáº¥u, cÃ¹ng thanh **TÃ¬m & Thay tháº¿** cÃ³ bá»™ Ä‘áº¿m káº¿t quáº£ (`x/y`).

## áº¢nh chá»¥p giao diá»‡n (`v0.7.3`)

*Tham chiáº¿u báº£n thiáº¿t káº¿ tÆ°Æ¡ng tÃ¡c táº¡i [`docs/demo-v0.7.0.html`](docs/demo-v0.7.0.html).*

| Cháº¿ Ä‘á»™ Xem trÆ°á»›c dáº¡ng trang giáº¥y | Cháº¿ Ä‘á»™ Chia Ä‘Ã´i & Máº«u cÃ´ng thá»©c | Xuáº¥t PDF / In trá»±c tiáº¿p (`Ctrl+P`) |
|---|---|---|
| ![Xem trÆ°á»›c v0.7.3](docs/0.7.3-3.png) | ![Chia Ä‘Ã´i v0.7.3](docs/0.7.3-2.png) | ![In & Xuáº¥t PDF v0.7.3](docs/0.7.3-1.png) |

## CÃ i Ä‘áº·t & Sá»­ dá»¥ng nhanh

### CÃ¡ch 1: DÃ¹ng báº£n dá»±ng sáºµn (khuyÃªn dÃ¹ng)
1. Táº£i gÃ³i phÃ¡t hÃ nh `MarkdownTabs-Rust_v0.7.3.zip` (hoáº·c cháº¡y trá»±c tiáº¿p `MarkdownTabs.exe` trong thÆ° má»¥c `02_LOCAL_RUNTIME`).
2. Giáº£i nÃ©n vÃ  nháº¥p Ä‘Ãºp **`Cai-dat-MarkdownTabs.bat`** (khÃ´ng cáº§n quyá»n Administrator, khÃ´ng cáº§n cÃ i Python hay Rust) Ä‘á»ƒ gáº¯n liÃªn káº¿t má»Ÿ máº·c Ä‘á»‹nh file `.md`.
3. Má»Ÿ tá»‡p máº«u [`mau-thu/Test-Xuat-file_v0.6.md`](mau-thu/Test-Xuat-file_v0.6.md) Ä‘á»ƒ kiá»ƒm thá»­ hiá»ƒn thá»‹ hÃ¬nh áº£nh, báº£ng, cÃ´ng thá»©c vÃ  xuáº¥t `.docx` / `.html` / `.pdf`.

### CÃ¡ch 2: Tá»± biÃªn dá»‹ch tá»« mÃ£ nguá»“n
YÃªu cáº§u: **Rust** (toolchain `stable-msvc`) vÃ  **Visual Studio Build Tools** trÃªn Windows.

```powershell
cd src-rust
cargo test --release
cargo build --release
```

*(Náº¿u á»• `C:` gáº§n Ä‘áº§y, Ä‘áº·t biáº¿n mÃ´i trÆ°á»ng `$env:CARGO_TARGET_DIR = "D:\mdtabs-target"` trÆ°á»›c khi cháº¡y `cargo`.)*

Tá»‡p thá»±c thi sau khi biÃªn dá»‹ch náº±m táº¡i `target\release\MarkdownTabs.exe`. Náº¿u chá»‰nh sá»­a bá»™ biá»ƒu tÆ°á»£ng trong `tools\gen_icons.py`, cháº¡y láº¡i:

```powershell
python tools/gen_icons.py
```

## Báº£ng phÃ­m táº¯t

| PhÃ­m táº¯t | Chá»©c nÄƒng |
|---|---|
| `Ctrl+N` / `Ctrl+O` | Táº¡o tÃ i liá»‡u má»›i / Má»Ÿ file (chá»n Ä‘Æ°á»£c nhiá»u file) |
| `Ctrl+S` / `Ctrl+Shift+S` | LÆ°u / LÆ°u thÃ nhâ€¦ |
| `Ctrl+W` / `Ctrl+Shift+T` | ÄÃ³ng tab hiá»‡n táº¡i / Má»Ÿ láº¡i tab vá»«a Ä‘Ã³ng |
| `Ctrl+1` / `Ctrl+2` / `Ctrl+3` | Chuyá»ƒn cháº¿ Ä‘á»™: Soáº¡n tháº£o / Xem trÆ°á»›c / Chia Ä‘Ã´i |
| `Ctrl+Shift+P` | Má»Ÿ Báº£ng lá»‡nh nhanh (há»— trá»£ gÃµ khÃ´ng dáº¥u) |
| `Ctrl+F` / `Ctrl+H` | TÃ¬m kiáº¿m / Thay tháº¿ (`Enter`: káº¿t quáº£ tiáº¿p, `Esc`: Ä‘Ã³ng) |
| `Ctrl+Shift+F` | Má»Ÿ há»™p thoáº¡i Sá»­a lá»—i trÃ¬nh bÃ y, font chá»¯ & cÃ´ng thá»©c |
| `Ctrl+B` / `Ctrl+I` / `Ctrl+K` | Äá»‹nh dáº¡ng **Äáº­m** / *NghiÃªng* / ChÃ¨n liÃªn káº¿t |
| `Ctrl++` / `Ctrl+-` / `Ctrl+0` | PhÃ³ng to / Thu nhá» / Äáº·t láº¡i 100% (há»— trá»£ `Ctrl + Cuá»™n chuá»™t`) |
| `Ctrl+P` / `F11` / `F1` | In hoáº·c LÆ°u PDF / ToÃ n mÃ n hÃ¬nh / Báº£ng phÃ­m táº¯t |

## Cáº¥u trÃºc kho mÃ£ nguá»“n

```text
01_REPO_GITHUB/
â”œâ”€â”€ src-rust/              # MÃ£ nguá»“n Rust (main.rs, features.rs, math.rs, docx.rs, fixer.rs, icons.rs...)
â”œâ”€â”€ tools/gen_icons.py     # CÃ´ng cá»¥ sinh bá»™ biá»ƒu tÆ°á»£ng SVG vector -> src-rust/src/icons_data.rs
â”œâ”€â”€ docs/                  # Ghi chÃº phÃ¡t hÃ nh (GHI-CHU-v0.7.3.md), báº£n demo & áº£nh chá»¥p v0.7.3
â”œâ”€â”€ mau-thu/               # CÃ¡c tá»‡p Markdown máº«u kiá»ƒm thá»­ cÃ´ng thá»©c, báº£ng, áº£nh & font cÅ©
â”œâ”€â”€ BAN-GIAO.md            # Hiá»‡n tráº¡ng phÃ¡t triá»ƒn & ghi chÃº mÃ´i trÆ°á»ng build
â””â”€â”€ LICHSU-REV.md          # Lá»‹ch sá»­ cÃ¡c phiÃªn báº£n tá»« v0.1 Ä‘áº¿n v0.7.3
```