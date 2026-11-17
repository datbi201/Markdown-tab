# Quy tắc dự án Markdown Tabs (nguồn duy nhất, đồng bộ qua git)

## 1. Dự án là gì
Trình xem và soạn thảo Markdown nhiều tab, viết bằng Rust (eframe/egui), chạy native trên Windows. Người dùng là kỹ sư ăn mòn và toàn vẹn đường ống, không phải lập trình viên: báo cáo bằng kết quả và giao diện, không dán mã thô vào trả lời.

## 2. Cấu trúc 3 tầng (thư mục cha `Markdown Tabs`)
- `01_REPO_GITHUB` (thư mục này): mã nguồn `src-rust/`, quy trình CI `.github/workflows/`, bộ cài `Cai-dat-*` / `Go-cai-dat-*`, tài liệu `docs/`, tệp thử `mau-thu/`. Có `.git`.
- `02_LOCAL_RUNTIME`: bản chạy thử trên máy (`MarkdownTabs.exe`, bộ cài). Không đưa vào git.
- `03_ONEDRIVE_BACKUP`: gói zip phát hành (`releases/`) và mã nguồn các bản cũ (`legacy_versions/`). Không đưa vào git.

## 3. Quy tắc lưu bản
- Kho chỉ giữ MỘT bản mới nhất. Khi lên bản mới: copy bản cũ sang `..\03_ONEDRIVE_BACKUP\legacy_versions\<tên bản>`, kiểm đủ số tệp rồi mới gỡ khỏi kho. Lịch sử cũ vẫn còn trong git.
- Mỗi bản phát hành đóng zip vào `..\03_ONEDRIVE_BACKUP\releases\`, exe chép sang `..\02_LOCAL_RUNTIME\`.
- Số phiên bản nằm ở `src-rust/Cargo.toml` và hằng `VERSION` trong `src-rust/src/main.rs`; đổi cả hai cùng lúc.

## 4. Build và kiểm thử
- Máy này không có Rust. Biên dịch Windows bằng GitHub Actions (`.github/workflows/build.yml`) hoặc `Bien-dich-Windows.bat` trên máy có Rust.
- Kiểm thử: `cargo test --release` trong `src-rust/` (22 bài). Bản nào chưa chạy trên Windows thật phải ghi rõ "Chưa thử trên Windows thật".
- Mỗi lỗi sửa kèm một kiểm thử tái hiện khi làm được.

## 5. Commit và đồng bộ
- Sau mỗi bản mới, sửa lỗi hoặc quyết định của người dùng: cập nhật `BAN-GIAO.md` (và `LICHSU-REV.md` nếu có bản mới) trong cùng lượt, rồi commit.
- Trước khi commit: `git pull --rebase origin main` (khi đã có remote). Có xung đột thì dừng và báo người dùng.
- Dòng cuối commit: `Co-Authored-By` đúng model đang chạy. Không add `target/`, `*.exe`, `*.zip`.
- Email tác giả: `343996192+datbi201@users.noreply.github.com` (đã đặt trong git config của kho). Push bị từ chối vì email thì báo người dùng, không tự đổi email.
- Không dùng chung thư mục `.git` giữa các máy qua Drive/OneDrive; mỗi máy clone riêng.
