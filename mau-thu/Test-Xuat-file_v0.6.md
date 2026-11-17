# Kiểm tra xuất file: hình, công thức, bảng

Tệp này dùng để kiểm tra xuất **Word (.docx)**, **HTML** và **PDF**. Mở bằng Markdown Tabs rồi xuất từng định dạng, đối chiếu với bản xem trước.

## 1. Hình ảnh

Ảnh PNG cục bộ: ![Ảnh PNG](Anh-test_v0.5.png)

Đường dẫn có dấu cách mã hoá %20: ![Ảnh dấu cách](anh%20thu%201.png)

Đường dẫn trong dấu ngoặc nhọn: ![Ảnh ngoặc nhọn](<anh thu 1.png>)

Ảnh JPEG: ![Ảnh JPEG](anh-thu-2.jpg) và ảnh GIF: ![Ảnh GIF](anh-thu-4.gif)

Ảnh nhúng sẵn (data URI, chấm đỏ 1x1): ![Chấm đỏ](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAUAAAAFCAYAAACNbyblAAAAHElEQVQI12P4//8/w38GIAXDIBKE0DHxgljNBAAO9TXL0Y4OHwAAAABJRU5ErkJggg==)

Thẻ HTML có ảnh:

<p align="center"><img src="Anh-test_v0.5.png" alt="Ảnh từ thẻ HTML" width="120"><br>Chú thích dưới ảnh (thẻ HTML)</p>

Ảnh từ mạng (cần Internet): ![Logo Rust](https://www.rust-lang.org/logos/rust-logo-128x128.png)

Ảnh không tồn tại: ![Ảnh hỏng](khong-co-anh.png)

## 2. Bảng

Bảng căn lề trái, giữa, phải, có công thức, ảnh và xuống dòng:

| Hạng mục | Giá trị | Đơn vị | Ghi chú |
|:---------|:-------:|-------:|---------|
| Đường kính ngoài | 323,9 | mm | $D$ |
| Chiều dày | 10,3 | mm | Danh định<br>Theo API 5L |
| Áp suất an toàn | $\frac{2St}{D}F$ | MPa | **Đậm**, *nghiêng*, `mã` |
| Ảnh trong ô | ![nhỏ](Anh-test_v0.5.png) | - | Ô có ảnh |
| Đường dẫn rất dài | https://example.com/mot-duong-dan-rat-dai/khong-co-dau-cach/de-kiem-tra-xuong-dong | - | Liên kết [tại đây](https://example.com) |

Bảng nhiều cột:

| Cột 1 | Cột 2 | Cột 3 | Cột 4 | Cột 5 | Cột 6 | Cột 7 |
|---|---|---|---|---|---|---|
| a | b | c | d | e | f | g |
| 1 | 2 | 3 | 4 | 5 | 6 | 7 |

Bảng thiếu ô:

| A | B | C |
|---|---|---|
| 1 | 2 |
| 3 |

## 3. Công thức

Trong dòng: $x^2 + y^2 = z^2$, $\alpha \le \beta$, $\sqrt[3]{x_i}$, $\mathbb{R}$, $\vec{v}\cdot\hat{n}$, $a_{ij}^{2}$.

Phân số và căn:

$$x = \frac{-b \pm \sqrt{b^2-4ac}}{2a}$$

Tổng, tích phân, giới hạn:

$$\sum_{i=1}^{n} i = \frac{n(n+1)}{2}, \qquad \int_0^1 x^2\,dx = \frac13, \qquad \lim_{x\to 0}\frac{\sin x}{x}=1$$

Ma trận và hệ phương trình:

$$\begin{pmatrix} a & b \\ c & d \end{pmatrix}\begin{pmatrix} x \\ y \end{pmatrix}=\begin{bmatrix} ax+by \\ cx+dy \end{bmatrix}$$

$$f(x)=\begin{cases} x^2 & x\ge 0 \\ -x & x<0 \end{cases}$$

Căn lề nhiều dòng:

$$\begin{aligned} P_f &= \frac{2(S+69)t}{D}\left[\frac{1-0{,}85\,d/t}{1-0{,}85\,d/(tM)}\right] \\ RSF &= \frac{P_f}{P_0} \end{aligned}$$

Ngoặc co giãn, văn bản, kiểu chữ:

$$\left( \frac{1}{2} \right), \quad \left[ \frac{a}{b} \right], \quad \mathbf{F}=m\mathbf{a}, \quad T_{rem}=\frac{t_{meas}-t_{min}}{CR}\approx 11\ \text{năm}$$

Dấu trên đầu và dưới chân: $\overline{AB}$, $\underline{x}$, $\dot{x}$, $\tilde{y}$, $\overbrace{a+b}^{n}$, $\xrightarrow{f}$

## 4. Danh sách, trích dẫn, mã

- Mục một
  - Mục con có $x^2$
- Mục hai

1. Bước một
2. Bước hai

- [x] Đã xong
- [ ] Chưa xong

> Trích dẫn có **đậm** và công thức $E=mc^2$.

```python
def p_safe(s, t, d, f):
    return 2 * s * t / d * f
```

Hết.
