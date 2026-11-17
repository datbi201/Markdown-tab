# Markdown Tabs v0.5.0 — Ghi chú phát hành (bản Rust native)

Bản này làm đủ 6 yêu cầu anh nêu. Exe Windows (có icon, thông tin phiên bản 0.5.0.0) nằm trong gói; mã nguồn đầy đủ trong `src-rust/`.

## 1. Thanh Ribbon theo tab
Thanh trên cùng có 7 tab, mỗi tab chứa nhóm lệnh riêng; giao diện cũ không còn dồn nút ra ngoài:

| Tab | Nội dung |
|---|---|
| **File** | Mới, Mở, Mở thư mục, **Gần đây**, Mở lại tab đã đóng, Lưu, Lưu thành, Lưu tất cả, Đóng tab, Thoát |
| **Định dạng** | Đậm, nghiêng, gạch ngang, code, tiêu đề, danh sách, trích dẫn… |
| **Chèn** | Liên kết, ảnh (URL / từ máy), bảng, khối code, đường kẻ, công thức dòng / khối, mục lục, chú thích, ngày hôm nay, bảng ký hiệu, mẫu văn bản |
| **Xem** | Soạn thảo / Xem trước / Chia đôi, zoom, giao diện sáng/tối, toàn màn hình, mục lục |
| **Xuất** | Word (.docx), HTML, PDF / In, Văn bản (.txt), Sao chép HTML, Sao chép Markdown |
| **Công cụ** | Sửa lỗi trình bày & font, Sửa nhanh, Hoàn tác, Kiểm tra công thức, Căn chỉnh bảng, Thống kê, Tìm / Thay thế, Tự động lưu |
| **Trợ giúp** | Phím tắt (F1), Cú pháp Markdown & công thức, Giới thiệu |

## 2. Mặc định "Xem trước"
Mở app hoặc mở file là vào thẳng chế độ Xem trước. Đổi bằng nút Soạn thảo / Xem trước / Chia đôi hoặc Ctrl+1 / 2 / 3.

## 3. Sửa lỗi trình bày, font chữ, công thức (Công cụ → Sửa lỗi trình bày & font…, Ctrl+Shift+F)
Hộp thoại hiện **danh sách thay đổi dự kiến** trước khi áp dụng, có **Hoàn tác lần sửa** và nút **Chọn khuyến nghị**. Khối code và code nội dòng không bị đụng tới. Các việc xử lý được:
- **Chữ vỡ mã** (UTF-8 bị đọc nhầm Windows-1252, ví dụ `Viá»‡t` → `Việt`), sửa theo từng từ nên dòng lẫn chữ đúng/sai vẫn xử lý được.
- **Ký tự font Symbol/MathType** (vùng riêng U+F0xx khi dán từ Word/MathType: ô vuông, ký tự lạ) → α, β, ≤, ≥, ±, ∑, ∫…
- **Font tiếng Việt cũ TCVN3 (.VnTime/ABC) và VNI → Unicode** (bảng 134 ký tự). App tự gợi ý khi thấy tài liệu chủ yếu là mã cũ; anh cũng có thể chọn tay.
- Gộp dấu tổ hợp (NFC), xoá ký tự vô hình (zero-width, BOM), NBSP/Tab/CRLF → chuẩn, khoảng trắng thừa cuối dòng.
- **Công thức:** `\( … \)` và `\[ … \]` → `$ … $` và `$$ … $$`; bỏ khoảng trắng thừa trong `$ x $`; dòng LaTeX trần (không có `$`) → khối `$$`; dấu `$` tiền tệ (`$5`, `$10`) được thoát `\$` để không bị hiểu nhầm là công thức; tuỳ chọn chuyển chỉ số Unicode (x², H₂O) → công thức.
- **Bảng và danh sách:** thêm dòng trống / dòng phân cách `|---|` còn thiếu; ký hiệu danh sách `•`, `●`, `–` → `-`; tiêu đề thiếu dấu cách sau `#`.
- **Công cụ → Kiểm tra công thức…** quét và liệt kê công thức sai (lệnh lạ, thiếu ngoặc, `$` lẻ) kèm số dòng; bấm vào dòng để nhảy tới đó.

**Công thức hiển thị thật** (không cần mạng): phân số, căn, chỉ số trên/dưới, tổng/tích phân/giới hạn, ma trận, hệ phương trình (`cases`), `aligned`, chữ Hy Lạp, mũ/gạch trên, vectơ, hàm `sin/cos/log/lim`… Công thức sai lệnh hiện dạng mã lỗi rõ ràng thay vì làm hỏng trang.

## 4. Xuất DOCX — nguyên nhân lỗi và cách sửa
**Gốc lỗi của v0.4:** phần đánh số danh sách ghi thẻ `<w:num>` thay vì `<w:numId>`, thứ tự phần tử trong định dạng đoạn sai chuẩn; Word khắt khe nên báo file hỏng, không mở được (LibreOffice thì du di nên mở được).
**Đã làm lại toàn bộ bộ xuất:** viết đúng thứ tự phần tử theo chuẩn ISO 29500, loại ký tự không hợp lệ XML, mã hoá đúng liên kết có dấu tiếng Việt. File sinh ra được kiểm bằng bộ kiểm tra chuẩn XSD của Office (kể cả phương trình): **PASSED**.
Nội dung xuất: Heading 1–6, đoạn văn đậm/nghiêng/gạch/code, danh sách chấm/số/việc, trích dẫn, khối code, bảng có hàng tiêu đề và căn lề, liên kết bấm được, ảnh cục bộ nhúng thật, và **công thức thành phương trình Word (OMML)** sửa được trong Word.

## 5. File → Gần đây
Tab File hiển thị ngay các file mở gần đây (bấm là mở), kèm danh sách "Tất cả" và nút **Xoá danh sách gần đây**. Danh sách được lưu qua các lần chạy. Màn hình khởi động (khi chưa mở file nào) cũng có danh sách này.

## 6. Chức năng bổ sung (nghiên cứu thêm)
Thống kê tài liệu (từ, ký tự, dòng, thời gian đọc) · Căn chỉnh bảng Markdown · Chèn mục lục · Tự động lưu mỗi 30 giây · Mở lại tab đã đóng (Ctrl+Shift+T) · Toàn màn hình (F11) · In / Lưu PDF (Ctrl+P) · Xuất HTML (kèm MathML) và TXT · Sao chép HTML/Markdown · Bảng ký hiệu · Mẫu văn bản · Liên kết nội bộ `#tiêu-đề` nhảy tới mục · Trang khởi động · Bảng phím tắt (F1) và cú pháp (Trợ giúp).

## Giới hạn đã biết
- Trên **LibreOffice**, phương trình có dấu toán tử đứng đầu một hàng (ví dụ `\overset{a}{=}`, ô căn chỉnh bắt đầu bằng `=`) hiện thêm ký tự "¿"; **Word hiển thị bình thường**. Chưa xử lý.
- Chuyển font cũ TCVN3/VNI áp dụng cho cả tài liệu; file lẫn chữ Unicode và mã cũ cùng lúc nên chuyển riêng từng đoạn (bôi đen rồi dùng Sửa nhanh).
- Bộ công thức tự viết hỗ trợ tập con LaTeX thường dùng; lệnh hiếm sẽ báo sai ở Kiểm tra công thức.
- Ảnh từ web (http) khi xuất DOCX được ghi chú dạng chữ, không tải về nhúng.
- Bản Windows được cross-compile trên Linux; phần giao diện đã kiểm trên Linux cùng mã nguồn, **chờ anh nghiệm thu trên máy thật**.

## Đề xuất bước tiếp
Xuất PDF trực tiếp không qua hộp thoại in · Danh sách lồng nhiều cấp trong DOCX · Đánh số công thức (1), (2)… và tham chiếu chéo · Nhập .docx → Markdown ngay trong bản Rust.

## Cài đặt / nâng cấp
1. Giải nén toàn bộ gói zip.
2. Nhấp đúp **`Cai-dat-MarkdownTabs-Rust_v0.5.bat`** (per-user, không cần admin). Cài đè tại `%LOCALAPPDATA%\Programs\Markdown Tabs`, giữ nguyên shortcut, liên kết `.md`, phiên làm việc.
3. Thử bằng các file `Test-Rust_v0.5.md`, `Test-Sua-loi_v0.5.md`, `Test-Ma-cu-TCVN3_v0.5.md`, `Test-Ma-cu-VNI_v0.5.md`.

Gỡ cài: Settings → Apps → *Markdown Tabs (Rust native)* → Uninstall, hoặc `Go-cai-dat-MarkdownTabs-Rust_v0.5.bat`.
Tự biên dịch trên Windows: chạy `Bien-dich-Windows.bat`. Kiểm thử: `cargo test --release` trong `src-rust/` (22 test).
