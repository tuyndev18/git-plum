# Nhận diện thương hiệu git-plum

Tài liệu này ghi lại quyết định thiết kế của logo git-plum (D-01, D-02) — đọc
trước khi thêm logo vào một chỗ mới trong app, hoặc trước khi sửa hình học/màu
của logo hiện có.

## 1. Khái niệm

Logo là **quả mận + nhánh git**: thân quả mận bầu dục bè ngang, có một **rãnh
dọc chạy mặt trước** (quả mận thật luôn có rãnh này, chạy dọc mặt quả — nó
KHÔNG phải vết khuyết cắt vào đường bao), và từ đỉnh quả vươn lên một thân
nhánh git thẳng đứng mang nút commit gốc, rồi rẽ một nhánh sang phải mang nút
commit con.

Ý đồ: logo phải đọc được là **VỪA quả mận VỪA đồ thị commit** cùng lúc, không
phải một quả mận với một đường kẻ gắn thêm cho có. Đường nhánh mọc ra từ đúng
điểm cuống tự nhiên của quả mận rồi mới rẽ, để cảm giác nó "mọc" ra như một
cái cuống thật, không phải một cái que cắm cạnh.

Hình học gốc: `src/assets/logo-mark.svg`, `viewBox="0 0 64 64"`. Thân mận là
bầu dục 34×30, tâm (32,37).

### Hai ràng buộc hình học đã trả giá để học

Hai bản nháp đầu đều hỏng theo cách chỉ nhìn ảnh render mới thấy, sửa hình học
phải giữ lại cả hai:

1. **Nhánh git phải BẤT ĐỐI XỨNG** — một thân chính thẳng đứng, một nhánh rẽ
   ra *một* bên. Bản nháp cho hai cuống toả đều sang hai bên (và bản sau cho
   hai nhánh cùng vươn chéo lên) đều đọc ra **râu côn trùng**, không ra đồ
   thị: mắt người gộp hai nét đối xứng thành một cặp. Đồ thị commit thật cũng
   luôn có một lane chính + nhánh rẽ ra một bên.
2. **Nhánh rẽ phải đi CHÉO LÊN, không nằm ngang.** Bản nháp cho nhánh này gần
   nằm ngang (y 19 → 16) và nó đọc ra một sợi dây có hạt ở đầu.

Ngoài ra, rãnh quả phải vẽ *đè lên mặt* thân (một nét stroke màu
`--accent-dim`), không được cắt lõm vào đường bao thân: bản nháp đầu khoét
đường bao ở góc trên phải và trông đúng như một lỗi render.

## 2. Tên hiển thị

Tên hiển thị luôn là **`git-plum`** — chữ thường toàn bộ, có gạch nối. KHÔNG
bao giờ viết `GitPlum`, `Git-Plum`, hay rút gọn thành `Plum`/`plum` một mình.

Tên này xuất hiện nguyên văn ở:

- Tiêu đề cửa sổ / thanh công cụ (`src/App.tsx`, `.brand`)
- `package.json` → trường `name`
- `src-tauri/tauri.conf.json` → trường `productName`
- Mọi tài liệu thương hiệu, kể cả tệp này

## 3. Bảng màu

Logo **tái dùng token màu đã có** trong `src/styles/app.css` (`:root`),
**không phải** một bảng màu riêng cho thương hiệu. Hai token dùng cho logo:

| Vai trò | Token CSS | Hex — theme tối (mặc định) | Hex — theme sáng |
|---|---|---|---|
| Thân quả mận | `--accent` | `#9d7cd8` | `#6b46a8` |
| Nhánh git + nút commit | `--success` | `#86b384` | `#4a7c47` |

`--accent` là màu định danh chính của `.brand` trong thanh công cụ hiện tại —
dùng lại cho thân mận để logo và chữ `git-plum` cùng một họ màu. `--success`
tương phản rõ với tím của thân mận và gợi liên tưởng "cành lá" cho nhánh git.

Trong `src/components/Logo.tsx` (dùng trong app), hai màu này là `var(--accent)`
/`var(--success)` — **tự đổi theo `prefers-color-scheme`** vì `app.css` đã có
biến thể của cả hai token trong khối `@media (prefers-color-scheme: light)`.
Trong `src/assets/logo-icon.svg` (input cho icon OS), hai màu này là **hex
tĩnh** — icon OS được `tauri icon` rasterize một lần, không chạy qua CSS
runtime của app nên không có ngữ cảnh theme để `var()` resolve.

## 4. Hai biến thể tệp

| Tệp | Nền | Dùng ở đâu |
|---|---|---|
| `src/assets/logo-mark.svg` | Trong suốt | `<Logo/>` trong app — logo ngồi trên `--bg` (nền app đang có), thêm nền riêng sẽ tạo khung hình thừa không khớp UI |
| `src/assets/logo-icon.svg` | `<rect>` tile đặc bo góc, hex `#16161a` | Input **duy nhất** cho `npx tauri icon` — icon OS cần trọng lượng thị giác khi đặt cạnh icon app khác có nền đặc trên desktop/taskbar (giống VS Code, Slack) |

Cả hai tệp vẽ **cùng hình học** quả mận + nhánh (chép markup, không `<use>` —
SVG dùng làm input rasterize của `tauri icon` không đáng tin cậy chạy
`<use xlink:href>` liên tệp), chỉ khác ở có/không có `rect` nền và ở
`var(--token)` (mark) so với hex tĩnh (icon).

**Tái sinh icon khi logo đổi:** sau khi sửa `logo-icon.svg`, chạy từ thư mục
gốc dự án:

```bash
npx tauri icon src/assets/logo-icon.svg
```

Lệnh này ghi đè toàn bộ `src-tauri/icons/*` (PNG các cỡ, `.ico`, `.icns`, bộ
`Square*Logo`/`StoreLogo` cho Windows Store tile). Không cần sửa
`bundle.icon` trong `tauri.conf.json` — danh sách bốn đường dẫn đã đúng tên
tệp mà `tauri icon` sinh ra.

## 5. Quy tắc dùng

**ĐÚNG:**

- Dùng `<Logo/>` (từ `src/components/Logo.tsx`) cho mọi chỗ trong app cần
  hiện logo — component đã tự lo theming qua biến CSS.
- Giữ nguyên màu thân mận (`--accent`) và nhánh (`--success`) — đây là hai
  token định danh của thương hiệu, không tự ý đổi sang màu khác cho một
  ngữ cảnh cụ thể (vd. logo trắng trơn trên nền tối).
- Khi cần icon OS mới (sau khi sửa logo), luôn tái sinh qua `npx tauri icon`
  từ `logo-icon.svg` — không tự tay export PNG rời rạc từng cỡ.

**SAI:**

- Không dùng `<img src="...">` để nhúng `logo-mark.svg` trong app — SVG nạp
  qua `<img>` là tài nguyên tĩnh biệt lập, không thừa hưởng biến CSS của
  trang cha, nên `var(--accent)`/`var(--success)` bên trong sẽ không
  resolve và logo mất màu.
- Không thêm `aria-label` trùng nghĩa lên `<Logo/>` — nó đã `aria-hidden`
  và chỉ là trang trí cạnh chữ `git-plum` trong `.toolbar-left`, chữ đó đã
  là nguồn văn bản cho trình đọc màn hình. Thêm nhãn sẽ đọc hai lần.
- Không tạo bản logo màu khác riêng cho từng theme thủ công (vd. một
  `logo-mark-light.svg` tách biệt) — biến CSS `var(--accent)`/`var(--success)`
  đã tự đổi theo `prefers-color-scheme`, thêm tệp song song chỉ tạo hai
  nguồn dữ liệu có thể lệch nhau.
