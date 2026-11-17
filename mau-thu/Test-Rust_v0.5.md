# File thử — Markdown Tabs v0.5.0

Nếu anh đang đọc file này ở chế độ **Xem trước** ngay khi mở thì mục 2 (mặc định Xem trước) đã đúng. Mục lục bên trái và thanh tab **File / Định dạng / Chèn / Xem / Xuất / Công cụ / Trợ giúp** là thanh ribbon mới.

## Thử 6 mục đã làm

1. **Ribbon:** bấm lần lượt từng tab ở trên đầu cửa sổ — mỗi tab chứa nhóm lệnh riêng.
2. **Mặc định Xem trước:** mở file bất kỳ → thấy bản xem trước, không phải khung soạn thảo. Đổi qua *Xem → Chia đôi / Soạn thảo* hoặc Ctrl+1/2/3.
3. **Sửa lỗi trình bày & font:** mở `Test-Sua-loi_v0.5.md`, `Test-Ma-cu-TCVN3_v0.5.md`, `Test-Ma-cu-VNI_v0.5.md` rồi *Công cụ → Sửa lỗi trình bày & font…* (Ctrl+Shift+F).
4. **Xuất DOCX:** *Xuất → Word (.docx)*, mở bằng Word — công thức bên dưới trở thành phương trình Word (OMML) sửa được.
5. **File → Gần đây:** mở vài file rồi vào tab File, danh sách file gần đây nằm ngay trong tab; có nút xoá danh sách.
6. **Chức năng bổ sung:** thống kê, căn bảng, chèn mục lục, tự lưu, mở lại tab đã đóng, toàn màn hình, in / lưu PDF, xuất HTML/TXT, bảng ký hiệu, mẫu văn bản, phím tắt (F1).

## Công thức (render thật, không cần mạng)

Phương trình bậc hai $ax^2+bx+c=0$ có nghiệm:

$$x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}$$

Tổng $\sum_{i=1}^{n} i = \frac{n(n+1)}{2}$ và tích phân $\int_0^1 x^2\,dx = \frac13$.

$$\begin{pmatrix} a & b \\ c & d \end{pmatrix}\begin{pmatrix} x \\ y \end{pmatrix} = \begin{cases} 1 & x>0 \\ 0 & \text{ngược lại}\end{cases}$$

$$\lim_{x\to 0}\frac{\sin x}{x}=1,\qquad \vec{F}=m\vec{a},\qquad \alpha+\beta\le\gamma$$

Ứng suất vòng (Barlow): $\sigma_h = \dfrac{P\,D}{2\,t}$, tỉ lệ $\sigma_h/\mathrm{SMYS}\le 0{,}72$.

## Nội dung mẫu để xuất thử

Đoạn văn có **chữ đậm**, *chữ nghiêng*, ~~gạch ngang~~ và `code`, cùng [một liên kết web](https://example.com) và [nhảy tới mục công thức](#công-thức-render-thật-không-cần-mạng).

- Mục chấm một
  - Mục lồng cấp hai
- Mục chấm hai

1. Bước một
2. Bước hai

- [x] Việc đã xong
- [ ] Việc chưa xong

> Trích dẫn để thử khi xuất Word.

```
let x = 1; // khối code
```

| Hạng mục | Giá trị | Ghi chú |
|---|---:|---|
| Áp suất | 98,4 barg | tại đầu giếng |
| SHA-256 | 4dc06552177bf94b2aa1 | checksum dài |

![Ảnh thử](Anh-test_v0.5.png)
