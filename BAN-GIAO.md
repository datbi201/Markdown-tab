# Bàn giao Markdown Tabs

Cập nhật lần cuối: 2026-11-26.

## Hiện trạng
- **Bản mới nhất: v0.7.0 (Rust native)**, làm ngày 2026-11-26. Nâng cấp toàn diện giao diện theo bản demo đã duyệt (`docs/demo-v0.7.0.html`): mặc định chế độ **Xem trước** với trang giấy bo góc trên nền 4 tầng sáng/tối; cụm 5 nút xuất nhanh **Word / PDF / MD / HTML / In** đặt ngay cạnh **Tệp mở gần đây**; bật mặc định **Soft Wrap** (xuống dòng mềm) kèm nút bật/tắt trên thanh trạng thái; sửa lỗi thanh trạng thái 3 vùng để luôn hiện **● Đã lưu / Chưa lưu**; bổ sung **Bảng lệnh nhanh (`Ctrl+Shift+P`)**, **Chế độ Tập trung (Zen)**, **ô lọc Mục lục**, **nút Sao chép** trên khối mã, đổi 4 màu chủ đạo ngay trên thanh đỉnh, chuột giữa đóng tab + menu chuột phải trên tab, và kéo chỉnh độ rộng mượt giữa Mục lục / Soạn thảo / Xem trước.
- Chi tiết phát hành: `docs/GHI-CHU-v0.7.0.md`.
- **Chưa kiểm thử thực tế ở DPI 125%/150%**: cần nghiệm thu trên máy Windows sau khi chạy `patches\apply_build_and_commit_v0.7.0.ps1`.
- Giữ nguyên toàn bộ lõi parser, bộ sửa lỗi TCVN3/VNI/MathType và bộ xuất DOCX/OMML của v0.6.0.

## Việc đang chờ
1. **Chạy kịch bản `patches\apply_build_and_commit_v0.7.0.ps1`** trên máy Windows để tự động sao lưu v0.6.0, áp mã v0.7.0, chạy `cargo test --release`, biên dịch `MarkdownTabs.exe` và commit/push lên kho `https://github.com/datbi201/Markdown-tab.git`.
2. **Nghiệm thu thực tế** giao diện v0.7.0 ở tỷ lệ hiển thị 100%, 125%, 150% và kiểm tra xuất DOCX/HTML với `mau-thu/Test-Xuat-file_v0.6.md`.

## Môi trường build trên máy Windows
- Rust cài bằng rustup (toolchain stable-msvc); Visual Studio Build Tools có sẵn.
- Ổ C: gần đầy. Build với `CARGO_TARGET_DIR=D:\mdtabs-target` (ngoài Google Drive), cargo nằm ở `%USERPROFILE%\.cargo\bin`:
  `cd src-rust` rồi `cargo test --release` và `cargo build --release`.
- Kiểm thử giao diện có biến môi trường: `MDT_TAB` (0..6), `MDT_VIEW` (edit/split), `MDT_DARK` (1/0), `MDT_ACCENT` (0..3), `MDT_ANCHOR`, `MDT_DIALOG`.
- Tạo lại bộ biểu tượng: `python tools/gen_icons.py`.

## Quyết định của người dùng
- 2026-11-26: chốt bản demo giao diện v0.7.0 (`docs/demo-v0.7.0.html`): mặc định mở ở chế độ **Xem trước**, đặt cụm 5 nút **Xuất Word / PDF / MD / HTML / In** (icon đơn giản, tính biểu tượng cao) cạnh nút **Tệp mở gần đây**, hỗ trợ kéo chuột đổi kích thước Mục lục / Soạn thảo / Xem trước, và triển khai qua gói bản vá kèm kịch bản build/commit tự động.
