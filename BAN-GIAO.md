# Bàn giao Markdown Tabs

Cập nhật lần cuối: 2026-11-17.

## Hiện trạng
- **Bản mới nhất: v0.6.0 (Rust native)**, làm ngày 2026-11-17. Giao diện mới (Ribbon biểu tượng nhỏ có tooltip, nút Sáng/Tối trên thanh trên cùng, 4 màu chủ đạo trong Trợ giúp) và rà soát lại xuất file (hình, bảng, công thức). Chi tiết và giới hạn: `docs/GHI-CHU-v0.6.0.md`.
- Đã biên dịch và chạy `cargo test --release` ngay trên máy này (Rust đã cài bằng rustup): 30/30 đạt. Bản DOCX tạo từ `mau-thu/Test-Xuat-file_v0.6.md` đã mở trong Word thật qua COM (3 bảng, 9 ảnh, 22 công thức) và đối chiếu bằng ảnh chụp trang; HTML đối chiếu trong Edge. Giao diện đã chụp ở chế độ sáng, tối, chia đôi, hai màu chủ đạo.
- **Chưa nghiệm thu bởi người dùng** trên máy thật. Chưa thử ở tỷ lệ hiển thị 125%/150%. PDF mới tạo thử bằng Edge, chưa xem từng trang.
- Gói chạy thử: `..\02_LOCAL_RUNTIME\MarkdownTabs.exe` (v0.6.0) và `..\03_ONEDRIVE_BACKUP\releases\MarkdownTabs-Rust_v0.6.0.zip`.
- Đã tổ chức lại thư mục theo 3 tầng ngày 2026-11-17. Bản cũ nằm ở `..\03_ONEDRIVE_BACKUP`.

## Việc đang chờ
1. **Người dùng thử v0.6.0** (chạy exe, hoặc cài bằng `Cai-dat-MarkdownTabs-Rust_v0.5.bat` trong gói zip) rồi báo lại về giao diện và xuất file.
2. **Kho GitHub chưa có remote.** Cần người dùng đồng ý tạo kho `datbi201/Markdown-Tabs` rồi mới push. Máy này chưa có `gh`.
3. Giới hạn đã biết của xuất file (xem ghi chú v0.6.0): Word chưa thẳng cột dấu `=` trong `aligned`; ảnh SVG chưa xuất vào Word; ô bảng chứa công thức/ảnh chưa căn giữa/phải trong xem trước.
4. Đề xuất cho bản sau: xuất PDF trực tiếp không qua hộp thoại in, danh sách lồng nhiều cấp trong DOCX, đánh số công thức và tham chiếu chéo, nhập .docx sang Markdown, đổi tên bộ cài sang v0.6.

## Môi trường build trên máy này
- Rust cài bằng rustup (2026-11-17), toolchain stable-msvc; Visual Studio Build Tools 18 có sẵn.
- Ổ C: gần đầy. Build với `CARGO_TARGET_DIR=D:\mdtabs-target` (ngoài Google Drive), cargo nằm ở `%USERPROFILE%\.cargo\bin`:
  `cd src-rust` rồi `cargo test --release` và `cargo build --release`.
- Tệp `src-rust\.cargo\config.toml` chỉ áp dụng cho đích mingw (build chéo trên Linux), không ảnh hưởng build MSVC.
- Kiểm thử giao diện có biến môi trường: `MDT_TAB` (0..6), `MDT_VIEW` (edit/split), `MDT_DARK` (1/0), `MDT_ACCENT` (0..3), `MDT_ANCHOR`, `MDT_DIALOG`.
- Tạo lại bộ biểu tượng: `python tools/gen_icons.py`.

## Quyết định của người dùng
- 2026-11-17: tổ chức dự án và cách commit giống các dự án khác trong `My Projects` (3 tầng, `CLAUDE.md`, `BAN-GIAO.md`, `LICHSU-REV.md`).
- 2026-11-17: thiết kế lại toàn diện giao diện, đẹp, hiện đại, tiện dụng, màu tươi sáng; xem demo trước khi làm (demo: https://claude.ai/artifact/6mTGrZnbFQqKbDJHvVLEbr, riêng tư).
- 2026-11-17 (trả lời demo): giữ cả 4 màu chủ đạo, chọn trong Trợ giúp; Ribbon nhiều nút nhỏ, không có tên, có tooltip khi rê chuột; nút Sáng/Tối để ở thanh tiêu đề; kiểm tra lại xuất file từ Markdown (hình, công thức, bảng phải đúng nội dung và hiển thị).
- 2026-11-17: cho phép tải và cài Rust (rustup) trên máy này để biên dịch và chạy kiểm thử.
