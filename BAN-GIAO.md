# Bàn giao Markdown Tabs

Cập nhật lần cuối: 2026-11-26.

## Hiện trạng
- **Bản mới nhất: v0.7.2 (Rust native — Bản bàn giao hoàn thiện)**, làm ngày 2026-11-26.
- Đã rà soát toàn bộ mã nguồn (`main.rs`, `features.rs`, `icons_data.rs`) và khắc phục triệt để 10 nhóm lỗi lặt vặt & khác biệt so với bản demo:
  1. Bọc khung trang giấy Xem trước trong layout dọc (`ui.vertical`) — hết hoàn toàn lỗi xếp ngang.
  2. Khung đỉnh đúng 2 tầng như demo: Hàng 1 gộp `[M]` + Tab tài liệu + `+` + Tìm lệnh + `Soạn thảo / Xem trước / Chia đôi` + Màu chủ đạo + Sáng/Tối + Zen; Hàng 2 là thanh công cụ đơn sắc gọn gàng với 5 nút **Xuất Word / PDF / MD / HTML / In** đặt cạnh **Tệp mở gần đây**.
  3. Tách riêng vùng bấm chuyển tab khỏi nút đóng `×` — bấm bất kỳ đâu trên tab để chuyển, chuột giữa đóng tab, chuột phải mở menu tab không bị cấn nút `×`.
  4. Căn lại vị trí mũi tên `▾` trên các nút menu để không đè lên nét biểu tượng.
  5. Mở rộng không gian dự trữ cho cụm nút phải ở tỷ lệ DPI 125%/150% tránh cắt nút.
  6. Bổ sung đường kẻ ngăn tiêu đề bảng và tắt thanh cuộn ngang khung Soạn thảo khi bật Soft Wrap.
  7. Đưa thanh Tìm/Thay thế lên phía trên, bổ sung đếm kết quả (`x/y`), nút `↑ Trước` / `↓ Tiếp`, tự chuyển sang Chia đôi khi nhảy tới từ khóa, và thêm phím tắt `Ctrl+P` (In/PDF).
  8. Tab **Thư mục** ở thanh bên trái hiển thị cả tệp **Đang mở** lẫn toàn bộ tệp `.md` **Trong thư mục** hiện tại để mở nhanh.
  9. Đóng khung trang bắt đầu (Start Page) theo đúng ngôn ngữ trang giấy bo góc.
  10. Cập nhật `prev_layout` theo `zen_mode` và `ribbon_open` để đổi bố cục không bị nháy/sót khung hình, đồng thời xử lý các cảnh báo biên dịch thừa.
- Chi tiết phát hành: `docs/GHI-CHU-v0.7.2.md`. Bản demo tham chiếu: `docs/demo-v0.7.0.html`.

## Việc đang chờ
1. Chạy kịch bản `patches\apply_build_and_commit_v0.7.2.ps1` trên máy Windows để chạy `cargo test --release`, biên dịch `02_LOCAL_RUNTIME\MarkdownTabs.exe`, đóng gói zip và push lên `https://github.com/datbi201/Markdown-tab.git`.
2. Mở thử `MarkdownTabs.exe` ở cả chế độ Sáng/Tối, Xem trước/Chia đôi và kiểm thử xuất DOCX/HTML với `mau-thu/Test-Xuat-file_v0.6.md`.
