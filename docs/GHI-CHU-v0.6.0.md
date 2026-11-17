# Markdown Tabs v0.6.0 — Ghi chú phát hành

Hai việc chính: giao diện mới và rà soát lại xuất file (hình, công thức, bảng).

## 1. Giao diện mới
- **Ribbon một hàng nút biểu tượng nhỏ**, không có tên nút. Rê chuột vào nút để xem tooltip (tên chức năng và phím tắt). Nút có màu theo nhóm lệnh. Ribbon vẫn thu gọn được bằng cách bấm lại vào tab đang chọn.
- **Thanh trên cùng**: logo, 7 tab Ribbon, nút chế độ xem (Soạn thảo / Xem trước / Chia đôi, chỉ còn biểu tượng khi cửa sổ hẹp) và **nút Sáng/Tối** ngay trên thanh này.
- **4 màu chủ đạo** (xanh dương, lục ngọc, cam san hô, tím) chọn trong **Trợ giúp → Màu chủ đạo**. Màu lưu theo phiên làm việc.
- Tab tài liệu có biểu tượng tệp, dấu chấm cam khi chưa lưu. Thanh trạng thái thêm nhãn "Đã lưu / Chưa lưu".
- Chế độ tối được sửa lại: trước đây trên máy đặt giao diện Windows sáng, ô xem trước và khung soạn thảo vẫn dùng màu sáng dù đã bấm Tối.
- Biểu tượng tự vẽ bằng nét vector (không cần font ngoài).

## 2. Xuất file — những gì đã sửa
Đã mở thử bản DOCX trong Microsoft Word thật (qua COM) và bản HTML trong Edge để đối chiếu với tệp thử `mau-thu/Test-Xuat-file_v0.6.md`.

| Mục | Trước | Nay |
|---|---|---|
| Ảnh có dấu cách / tiếng Việt trong đường dẫn (`%20`, `<đường dẫn>`) | Không tìm thấy ảnh | Hiển thị và xuất đủ |
| Ảnh WebP, BMP | Không xuất vào Word | Tự đổi sang PNG |
| Ảnh nhúng sẵn `data:image/...;base64` | Chỉ ghi chú chữ | Hiển thị và xuất đủ |
| Ảnh từ mạng (http/https) | Chỉ ghi chú chữ; xem trước không hiện | Xem trước tải nền; Word nhúng thật (cần Internet khi xuất) |
| Thẻ HTML `<img>`, `<br>` trong Markdown | Hiện mã thô | Hiện ảnh thật và xuống dòng |
| Ảnh trong HTML / PDF | Đường dẫn tương đối, mất ảnh khi chép tệp đi hoặc in PDF từ thư mục tạm | Ảnh cục bộ nhúng thẳng vào tệp |
| Ảnh quá to trong Word | Có thể tràn trang hoặc tràn ô bảng | Tự thu theo bề rộng trang / cột và chiều cao trang |
| Bảng: căn trái / giữa / phải | Mất khi xuất Word; không hiện trong xem trước | Giữ trong Word, HTML và xem trước |
| Bảng: độ rộng cột | Chia đều | Chia theo nội dung, hàng không bị cắt ngang trang, tiêu đề lặp lại |
| `<br>` trong ô bảng | Hiện chữ `<br>` | Xuống dòng thật |
| Công thức đứng một mình trong ô / đoạn | Word tự căn giữa | Giữ đúng căn lề |
| Ma trận, `cases` trong Word | Dồn thành một hàng | Đúng dạng ma trận nhiều hàng, nhiều cột |
| `aligned` / `align` / `\\` trong Word | Các dòng nối liền nhau | Mỗi dòng một hàng của mảng phương trình |
| PDF / In | Ảnh cục bộ không hiện | Ảnh nhúng sẵn; thêm quy tắc không cắt hàng bảng, ảnh, công thức giữa trang |

## 3. Giới hạn đã biết
- **Word, mảng phương trình `aligned`:** các dòng đã xuống hàng đúng nhưng dấu `=` giữa các dòng chưa thẳng cột (Word căn giữa từng dòng). Nội dung đúng, chưa đẹp tuyệt đối. Trong HTML và xem trước vẫn thẳng cột.
- Ảnh SVG chưa xuất vào Word (ghi chú chữ thay thế); trình duyệt vẫn hiển thị khi xuất HTML nếu ảnh là tệp cục bộ.
- Ảnh từ mạng cần Internet lúc xuất Word; ảnh lỗi hoặc quá 12 giây thì ghi chú chữ.
- Trong xem trước, ô bảng chứa công thức hoặc ảnh chưa căn giữa/phải theo cột (xuất file thì đúng).
- PDF vẫn đi qua hộp thoại In của trình duyệt (chọn "Lưu thành PDF"). Đã tạo thử bằng Edge nhưng chưa xem trực quan từng trang.
- Giao diện đã chụp và xem trên một máy Windows (Windows 11, màn hình rộng). Chưa thử ở các tỷ lệ hiển thị 125%/150% và trên máy khác.

## 4. Kiểm thử
`cargo test --release` trong `src-rust/`: 30 bài, đạt cả 30. Bài thử mới: ảnh (đường dẫn, data URI, HTML), bảng căn lề, ma trận và `aligned` trong Word, biểu tượng.
Tệp thử xuất file: `mau-thu/Test-Xuat-file_v0.6.md` (hình, bảng, công thức). Mở bằng Markdown Tabs rồi xuất từng định dạng để đối chiếu.

## 5. Cài đặt
Giải nén gói `MarkdownTabs-Rust_v0.6.0.zip`, chạy `Cai-dat-MarkdownTabs-Rust_v0.5.bat` (bộ cài giữ tên cũ, cài đè bản mới, giữ liên kết `.md` và phiên làm việc).
Tự biên dịch: `Bien-dich-Windows.bat`, hoặc GitHub Actions (`.github/workflows/build.yml`).
