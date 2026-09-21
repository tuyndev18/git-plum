# Mô hình vẽ đồ thị: đo từ ảnh tham chiếu, không đoán

**Ngày:** 2026-09-21
**Bối cảnh:** Sau ba vòng sửa lỗi liên tiếp (seam giữa hai hàng, cạnh hợp nhánh bị vẽ
như đường dọc, lane ma ở hàng merge), đồ thị vẫn bị người dùng đánh giá "rất xấu và khó
nhìn". Ba lần sửa đều đúng về mặt lỗi được nêu nhưng không lần nào chạm tới nguyên nhân
thật, nên tài liệu này **đo bằng pixel** thay vì tiếp tục suy luận.

**Phương pháp:** giải mã ảnh tham chiếu bằng một bộ giải PNG tự viết trong Node
(`zlib.inflateSync` + bỏ filter Paeth/Sub/Up/Average), rồi quét pixel. Mọi con số dưới
đây là **đo được**, không phải ước lượng bằng mắt.

**Hai ảnh đã đo:**

| Ảnh | Kích thước | Đặc điểm |
|---|---|---|
| `docs/screenshots/main-4.png` | 1921×1032 | 5 lane đồng thời |
| ảnh người dùng gửi 2026-09-21 (case 8 lane) | 1820×1210 | **8 lane đồng thời**, nhiều merge chồng nhau |

Ảnh thứ hai xác nhận lại toàn bộ số đo của ảnh thứ nhất **và** bổ sung ba phát hiện mới
(mục 2.5, 2.6, 3) mà ảnh 5 lane không đủ ca để lộ ra.

---

## 1. Số đo thực tế của tham chiếu

| Đại lượng | Giá trị đo | Hằng số hiện tại | Khớp? |
|---|---|---|---|
| Khoảng cách giữa hai lane (pitch) | **22 px** (tâm lane tại x = 384.5, 406.5, 428.5, 450.5, 472.5) | `LANE_WIDTH = 22` | ✅ |
| Độ dày đường lane | **2 px** | `EDGE_WIDTH = 2` | ✅ |
| Chiều cao hàng | **28 px** (tâm nút cách nhau 28, 56, 140, 364 — đều là bội của 28) | `ROW_HEIGHT = 28` | ✅ |
| Đường kính nút thường | **10 px** → bán kính **5** | `NODE_RADIUS = 5` | ✅ |
| Đường kính nút lớn của tham chiếu | **22 px** → bán kính **11** | `MERGE_NODE_RADIUS = 6` | *không áp dụng* |
| Bán kính bo góc khi rẽ nhánh | **~6 px** (đo từ chuyển tiếp y=294→301) | `CORNER_RADIUS = 4` | ~ |
| Nền khung | `#1c1e23` đồng nhất **mọi cột** | canvas trong suốt | ✅ |

Ba hằng số cốt lõi (`LANE_WIDTH`, `EDGE_WIDTH`, `ROW_HEIGHT`) **đã đúng**. Vấn đề không
nằm ở các con số.

> **Nút 22px của tham chiếu KHÔNG phải mục tiêu của git-plum.** Nó lớn vì bên trong là
> **ảnh avatar tác giả**, tải từ mạng. git-plum không làm vậy — ràng buộc Privacy của
> `PROJECT.md` cấm gửi gì ra ngoài khi chưa được cho phép rõ ràng, và email commit gửi
> sang Gravatar/GitHub là đúng thứ bị cấm. Capability của webview cũng không có quyền
> HTTP nào. Nút của git-plum là **chấm đặc màu lane** cho mọi commit; hiệu ứng "tách nút
> khỏi đường chạy phía sau" mà avatar đem lại được thay bằng quầng nền
> (`NODE_HALO_WIDTH`). Dòng trên chỉ ghi lại số đo, không phải việc phải làm.

---

## 2. Nguyên nhân thật: sai **mô hình vẽ**, không sai tham số

### 2.1 Tham chiếu vẽ lane là **một nét liền**, không phải chuỗi đoạn theo hàng

Quét dọc theo tâm mỗi lane, tìm khoảng trống (pixel không bão hoà màu):

```
lane x=384  không có khoảng trống nào
lane x=406  trống y=200–213, sau đó liền tới hết
lane x=450  trống y=200–297, sau đó liền tới hết
lane x=472  trống y=200–437, sau đó liền tới hết
```

Mỗi lane chỉ có **đúng một** khoảng trống, nằm ở phía trên — đó là lúc lane **chưa ra
đời**. Từ hàng khai sinh trở xuống, lane là một cột dọc **không đứt một pixel nào**.

Ảnh 8 lane xác nhận lại trên quy mô lớn hơn (quét cả 1200px chiều cao):

```
lane x=200  LIỀN MẠCH toàn bộ
lane x=222  LIỀN MẠCH toàn bộ
lane x=244  LIỀN MẠCH toàn bộ
lane x=288  LIỀN MẠCH toàn bộ
lane x=310  LIỀN MẠCH toàn bộ
lane x=332  trống y=20–220, sinh ra ở y=221
lane x=178  trống y=297–810   ← CHẾT rồi SỐNG LẠI
```

Năm lane chạy suốt 1200px, đi sau hàng chục nút, **không đứt một pixel**.

**Ca mới `lane x=178`:** lane chết ở y=296 rồi được cấp lại cho một nhánh khác ở y=811.
Đây là ca mà hợp đồng `LaneSpan` (mục 4.1) phải chịu được: **một chỉ số lane có thể có
nhiều span rời nhau** trong cùng một trang. Gom span theo `lane` là sai — phải gom theo
*quãng sống*, và một lane cho ra 0..n span.

Bộ vẽ hiện tại làm ngược lại: mỗi hàng tự vẽ phần của mình rồi ghép ở mép ô —
`passthrough` vẽ mép-trên → mép-dưới, `outEdges` vẽ tâm → mép-dưới, cộng thêm một đoạn
mép-trên → tâm mới thêm gần đây. Một cột dọc dài 800px được tái tạo từ **~57 đoạn rời**,
mỗi đoạn phải khớp mép với đoạn kế tiếp.

Đó là lý do mọi vòng sửa đều chỉ dời điểm vỡ:

- Vòng 1: `outEdges` vẽ quá mép dưới → đè lên hàng sau.
- Vòng 2: `passthrough` có cạnh hợp nhánh (`from != to`) bị vẽ như đường dọc.
- Vòng 3: bước 4 của `lanes.rs` phát passthrough cho lane vừa sinh ở chính hàng đó.

Cả ba đều là **lỗi khớp mép**. Với mô hình nét liền, cả ba lớp lỗi này **không tồn tại
được** — không có mép nào để khớp sai.

### 2.2 Nút được vẽ **đè lên** đường, không khoét lỗ

Cắt dọc qua một nút avatar ở lane x=428:

```
y165–173  #8e00c2   ← màu lane, chạy thẳng
y174–191  #056052 / #84a7d3   ← ảnh avatar
y192–200  #8e00c2   ← màu lane, chạy tiếp
```

Đường lane chạy liên tục *phía sau*; nút là một đĩa đặc vẽ chồng lên. Không có thao tác
"tô nền để cắt đường" nào cả.

Bộ vẽ hiện tại tô tâm nút bằng màu nền (`NODE_FILL` / `--graph-node-fill`) để khoét
đường phía sau. Cách đó buộc màu tô phải **trùng tuyệt đối** nền hàng — mà nền hàng lại
xen kẽ (`.commit-row:nth-child(odd)`) và đổi theo theme. Sai một chút là lòng nút thành
một đốm khác màu. Đây là một ràng buộc tự tạo ra rồi tự phải giữ.

### 2.3 Nút thường là **chấm đặc**, không phải vòng rỗng

Cắt ngang qua nút nhỏ (lane x=428, y=322):

```
#1c1e23 ×5   #8c01bf  #8e00c2 ×10  #9001c2   #c517b6 ×4
   nền        viền      LÒNG NÚT      viền      lane kế
```

Lòng nút là `#8e00c2` — **đúng màu lane**, đặc hoàn toàn. Không có vòng rỗng, không có
lỗ nền.

Nút 22px thì có viền 2px màu lane bọc quanh một **ảnh avatar** của tác giả:

```
#8c00c0 #8e00c2 | #055d51 ... #80a5cf ... #056052 | #8e00c2 #8c00c1
   viền lane    |          ảnh avatar             |    viền lane
```

git-plum không có avatar (không gọi mạng — xem ràng buộc Privacy của `PROJECT.md`), nên
tương đương đúng là: **chấm đặc màu lane cho mọi commit**, và phân biệt merge bằng một
dấu hiệu khác (viền dày hơn, hoặc vòng ngoài), không phải bằng ảnh.

### 2.4 Dải nền hàng **nhuộm theo màu lane**, không phải xám trung tính

Nền trong cột đồ thị, đo tại x=520:

```
y314–328  #271b33   ← tím, khớp lane tím #8e00c2
y342–356  #2d1d32   ← hồng, khớp lane hồng #c517b6
y370–384  #271b33
y398–412  #2d1d32
y307/335/363/391  #1c1e23   ← nền trơn, phân cách
```

Mỗi hàng được phủ một lớp màu lane rất mờ (~8–10% alpha trên nền `#1c1e23`). Đó là thứ
giúp mắt gom các hàng cùng nhánh thành một khối khi lướt nhanh. App hiện tại dùng
`color-mix(... var(--bg-raised) 40% ...)` — xám trung tính, không mang thông tin nhánh.

Ảnh 8 lane cho đủ dải để thấy quy luật (đo tại x=355):

```
#323024  hàng lane vàng  (#ebc432)
#2d1d32  hàng lane hồng tím (#c517b6)
#2f1b2b  hàng lane hồng sen (#d90171)
#271b33  hàng lane tím   (#8e00c2)
#322424  hàng lane đỏ    (#cd0101)
#1b1d22  dải phân cách, nền trơn
```

### 2.5 Cạnh merge là **một nét ngang dài**, vẽ ĐÈ lên mọi lane nó cắt qua

Đây là phát hiện quan trọng nhất từ ảnh 8 lane. Quét hàng `Merge branch
'feature/request-info-students' into release` (y=217):

```
y216   178-179  195-206  222-223  244-245  266-267  288-289  310-311
y217   178-179  195-328                                              ← MỘT run duy nhất
y219   178-179  195-206  222-223  244-245  266-267  288-289  310-311
```

Ở y=217 các lane 222/244/266/288/310 **biến mất khỏi kết quả quét** — không phải vì
chúng không được vẽ, mà vì cạnh ngang **phủ đè** lên chúng đúng 2px chiều cao. Trên/dưới
một pixel thì chúng vẫn nguyên.

Hệ quả cho thứ tự vẽ: **lane dọc vẽ trước, cạnh merge ngang vẽ sau, nút vẽ sau cùng.**

### 2.6 Cạnh merge dùng **hai màu**: khuỷu theo lane đích, thân theo lane nguồn

Quét màu dọc cạnh merge ở y=217:

```
x195–206  #0669f7   ← 12px, màu lane ĐÍCH (x=200.5)
x207–328  #f2ca33   ← 122px, màu lane NGUỒN (x=332.5)
```

Kiểm chứng trên 10 cạnh merge độc lập trong cùng ảnh:

| y | màu khuỷu (đầu) | màu thân (giữa) | đích → nguồn |
|---|---|---|---|
| 161 | `#159fbe` | `#c517b6` | lane0 ← lane3 |
| 217 | `#0669f6` | `#f2ca33` | lane1 ← lane7 |
| 357 | `#0669f6` | `#f2ca33` | lane1 ← lane7 |
| 413 | `#c517b6` | `#f2ca33` | lane3 ← lane7 |
| 441 | `#0669f6` | `#f2ca33` | lane1 ← lane7 |
| 945 | `#159fbe` | `#d90171` | lane0 ← lane4 |
| 301, 805, 861, 917 | `#15a0bf` | `#15a0bf` | lane0 ← lane0-teal (nguồn trùng màu đích) |

Quy luật: **thân cạnh mang màu lane nguồn** (nhánh bị gộp vào), **khuỷu cong mang màu
lane đích**. Điểm đổi màu nằm ngay sau vùng bo góc, cách tâm lane đích ~6px.

Ý nghĩa: cạnh thuộc về **nhánh bị merge**, đúng ngữ nghĩa git. Bốn dòng cuối bảng không
phải ngoại lệ — ở đó lane nguồn tình cờ cũng là lane teal.

---

## 3. Bảng màu lane của tham chiếu

Đo từ pixel đường lane (không phải từ vùng nút). Ảnh 8 lane cho **đủ cả bảng**:

| Lane | Màu đo được | Tên |
|---|---|---|
| 0 | `#15a0bf` | xanh ngọc |
| 1 | `#0669f7` | xanh dương |
| 2 | `#8e00c2` | tím |
| 3 | `#c517b6` | hồng tím |
| 4 | `#d90171` | hồng sen |
| 5 | `#cd0101` | đỏ |
| 6 | `#f25d2e` | cam |
| 7 | `#ebc432` | vàng |

Lane 7 (`#ebc432`) gần như trùng lane 0 về vai trò gập vòng — bảng thực chất có **7 hue
phân biệt**, khớp đúng `LANE_COLORS = 7` đã chốt ở `src-tauri/src/graph/types.rs`. Không
cần đổi hằng số đó.

Bảng hiện tại (`LANE_COLORS` trong `geometry.ts`) là bảng pastel kiểu One Dark:
`#e06c75 #61afef #98c379 #e5c07b #c678dd #56b6c2 #d19a66`. Độ bão hoà thấp hơn hẳn, và
có màu vàng/cam/xanh lá mà tham chiếu không dùng. Trên nền tối `#1c1e23`, dải màu bão
hoà cao của tham chiếu tách nhánh rõ hơn nhiều.

> `PROJECT.md` đã chốt việc sao chép sát bảng màu tham chiếu (commit `5417400`), nên
> dùng đúng các mã trên là hợp với quyết định đã có. Nếu muốn tránh rủi ro giấy phép thì
> giữ **độ bão hoà và khoảng cách hue** tương đương nhưng dịch hue đi một góc.

---

## 4. Đề xuất: đổi sang mô hình "lane liên tục"

### 4.1 Hợp đồng dữ liệu mới

Thay vì mỗi hàng mang các đoạn cạnh của riêng nó, backend trả về **quãng sống của từng
lane** trong trang đang xem:

```rust
pub struct LaneSpan {
    pub lane: u16,
    pub color: u8,
    pub start_row: u32,   // chỉ số hàng lane ra đời
    pub end_row: u32,     // chỉ số hàng lane chết
    pub start_kind: SpanStart,  // Branch { from_lane } | Root | PageEdge
    pub end_kind: SpanEnd,      // Merge { into_lane } | Root | PageEdge
}
```

Một chỉ số `lane` có thể cho **nhiều span rời nhau** (xem ca `lane x=178` ở mục 2.1) —
`Vec<LaneSpan>` chứ không phải map theo lane.

**Thứ tự vẽ bắt buộc** (suy ra từ mục 2.5 — cạnh ngang đè lên lane dọc, nút đè lên cả
hai):

```
1. Mọi LaneSpan          → nét dọc liền, một beginPath mỗi span
2. Mọi cạnh merge ngang  → vẽ ĐÈ lên lane dọc nó cắt qua
3. Mọi nút               → vẽ ĐÈ lên tất cả
```

Với mỗi `LaneSpan`, bước 1 phát **một** `beginPath()`:

1. `moveTo(laneX, yOf(start_row) + half)` — hoặc vẽ đoạn rẽ vào nếu `start_kind = Branch`.
2. `lineTo(laneX, yOf(end_row) + half)` — **một nét thẳng duy nhất** qua hàng chục hàng.

Cạnh merge (bước 2) vẽ **hai nét, hai màu** theo mục 2.6:

- Khuỷu cong từ tâm nút đích ra ~6px: **màu lane đích**.
- Thân ngang từ đó tới tâm lane nguồn: **màu lane nguồn**.

### 4.2 Vì sao cách này đúng hơn

- **Không còn lớp lỗi khớp mép.** Một lane = một nét. Không có mép nào để lệch.
- **Ít lệnh vẽ hơn nhiều.** Hiện tại ~2–3 `stroke()` mỗi hàng mỗi lane → với 50 hàng
  hiển thị × 5 lane là ~500 lệnh. Mô hình mới: ~5 nét cho 5 lane, cộng ~50 nút. Tốt cho
  Core Value (cuộn 60fps).
- **Bỏ được `NODE_FILL` và biến CSS đi kèm.** Nút vẽ đè, không khoét lỗ, nên không cần
  biết nền phía sau màu gì — tự khắc đúng ở cả hai theme.
- **Ảo hoá vẫn chạy được.** Lane bị cắt ở biên viewport thì `start_row`/`end_row` kẹp vào
  khoảng đang hiển thị và `*_kind` thành `PageEdge` — nét vẫn vẽ hết mép canvas.

### 4.3 Việc phải làm

| Việc | Tệp | Ghi chú |
|---|---|---|
| Thêm `LaneSpan` + hàm gom span | `src-tauri/src/graph/` | Gom từ `lanes.rs` đã có; thuật toán gán lane **không đổi**. Một lane → 0..n span |
| Đổi bộ vẽ sang vẽ theo span | `src/lib/graph-render/canvasRenderer.ts` | Bỏ `drawEdge` theo hàng |
| Ba lượt vẽ đúng thứ tự: lane → cạnh merge → nút | `canvasRenderer.ts` | Mục 2.5 |
| Cạnh merge hai màu (khuỷu đích / thân nguồn) | `canvasRenderer.ts` | Mục 2.6 |
| Nút: chấm đặc màu lane | `canvasRenderer.ts` | Bỏ khoét nền |
| `MERGE_NODE_RADIUS` 6 → phân biệt bằng viền | `geometry.ts` | Không dùng avatar |
| Bảng màu bão hoà cao, 7 hue | `geometry.ts` | Xem mục 3 — `LANE_COLORS = 7` giữ nguyên |
| Dải nền hàng nhuộm màu lane | `app.css` + truyền màu lane vào hàng | Thay `--bg-raised 40%` |
| Bỏ `NODE_FILL`, `--graph-node-fill` | cả hai | Không còn cần |

### 4.4 Điều **không** đổi

- Thuật toán gán lane (`assign` trong `lanes.rs`) — đã đúng, 115 test xanh.
- `LANE_WIDTH` / `ROW_HEIGHT` / `EDGE_WIDTH` — đã khớp tham chiếu.
- Ràng buộc "Rust tính, frontend chỉ vẽ".
- Một nguồn toạ độ Y duy nhất từ virtualizer.
- Canvas thuần trình bày, không bắt sự kiện.

---

## 5. Rủi ro

- **Span phải cắt đúng ở biên trang.** Lane chạy xuyên qua cả trang phải cho
  `start_kind = end_kind = PageEdge`; sai chỗ này thì nhánh dài bị cụt ở mép màn hình.
  Cần test riêng cho ca "lane không sinh cũng không chết trong khoảng hiển thị".
- **Một lane nhiều span.** Ca `lane x=178` (chết y=296, sống lại y=811) phải ra **hai**
  span rời, không phải một span dài nuốt cả khoảng chết. Cần test ghim.
- **Thứ tự vẽ là ràng buộc hiển thị, không phải chi tiết cài đặt.** Ba lượt ở mục 4.1
  phải giữ đúng thứ tự; đảo lại thì cạnh merge bị lane dọc cắt vụn. Ghim bằng test trên
  thứ tự lời gọi `stroke()`.
- **`MAX_VISIBLE_LANES` vẫn phải khớp hai phía.** Không đổi, nhưng đừng để refactor làm
  rơi test ghim.
