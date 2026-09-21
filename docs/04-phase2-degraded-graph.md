# Cách vẽ đồ thị suy giảm có chủ ý

**Chốt:** 2026-09-21, plan 02-03
**Đóng:** câu hỏi để ngỏ số 2 của `.planning/phases/02-history-graph/CONTEXT.md`
(*"Số lane tối đa hiển thị trước khi chuyển sang cách vẽ suy giảm"*)

---

## 1. Vấn đề

Hình dạng lịch sử bệnh lý có thật trong repo thật, không phải giả thuyết:

- một merge octopus **mười lăm cha** là hợp pháp với git;
- **bốn mươi nhánh sống đồng thời** xảy ra trên repo có nhiều người cùng làm;
- repo mẫu `wide` của dự án đã có **25 lane đồng thời** chỉ với 73 commit.

Vẽ thẳng những hình dạng này ra thì cột đồ thị tràn ngang, đẩy cột thông điệp commit ra
khỏi màn hình, và phá mốc "dưới một giây tới khung hình đầu" của tiêu chí thành công số
1. ROADMAP vì vậy đòi **đặt và ghi lại một cách vẽ suy giảm có chủ ý** — chứ không để
giao diện tự vỡ theo cách nào đó không ai lường trước.

Tài liệu này ghi cách đó.

---

## 2. `MAX_VISIBLE_LANES = 20` — chốt bằng lập luận hiển thị

### 2.1 Phép tính

```text
  cửa sổ mặc định                    1440 px   src-tauri/tauri.conf.json "width"
  × vùng giữa                        × 52%     src/components/AppLayout.tsx Panel id="main"
  = vùng giữa                        ≈ 749 px
  × ngân sách cột đồ thị             × 40%     60% còn lại cho thông điệp commit
  = cột đồ thị                       ≈ 300 px
  − lề trái GRAPH_PADDING_LEFT       −   8 px
  ÷ LANE_WIDTH                       ÷  14 px  chốt ở plan 02-05
  = 20,9                             → 20 lane
```

`LANE_WIDTH = 14 px` là mật độ đủ cho một chấm commit bán kính 4 px cộng khoảng trống
hai bên để hai lane kề nhau phân biệt được ở tỉ lệ 100%. Ngân sách 40% cho cột đồ thị
là trần: quá con số đó thì thông điệp commit — thứ người dùng thực sự đọc — bị cắt.

### 2.2 Vì sao **không** chốt từ số đo của `wide`

Bản đầu của plan 02-03 định lấy số lane lớn nhất đo được trên repo mẫu `wide` rồi đặt
giới hạn cao hơn nó. Lập luận đó tự phủ định:

> Chốt `MAX_VISIBLE_LANES` ≥ số lane lớn nhất của mọi fixture ⟹ cap không bao giờ chạm
> trên bất kỳ fixture nào ⟹ `truncated_parents` luôn bằng 0 ⟹ **toàn bộ nhánh mã vẽ suy
> giảm chưa từng chạy một lần nào trên dữ liệu thật** — trong khi chính tài liệu này mô
> tả một cơ chế không ai kiểm.

Giá trị tạm trước đó là 32, cao hơn cả `wide` (25) lẫn repo hiệu năng (21), nên nó rơi
đúng vào cái bẫy này.

Số đo của fixture có một việc khác: **kiểm chứng** rằng bộ dữ liệu thật đủ rộng để chạm
cái cap đã chốt từ hiển thị. Đo được, sau khi chốt 20:

| Nguồn dữ liệu | Commit | Lane lớn nhất | Lane sống đồng thời | Hàng vượt cap | Cha bị cắt |
|---|---|---|---|---|---|
| `wide` | 73 | **24** | **25** | **10** | **5** |
| repo hiệu năng 100k | 100 007 | 20 | 21 | 0 | 0 |
| `octopus` | 7 | 3 | 4 | 0 | 0 |
| `linear`, `orphan`, `detached`, … | — | 0–1 | 1–2 | 0 | 0 |

`wide` chạm cap và sinh `truncated_parents = 5`, nên nhánh vẽ suy giảm **được kiểm trên
dữ liệu git thật**, không chỉ trên dữ liệu dựng tay. Đó là điều cap 32 không cho.

---

## 3. Hai loại tràn, hai hành vi khác nhau

Đây là phần dễ nhầm nhất và là nơi một lỗi mất commit từng suýt lọt qua.

| Loại | Xảy ra khi | Rust làm gì | Frontend làm gì |
|---|---|---|---|
| **Cha thêm không đủ lane** | Merge có nhiều cha hơn số lane trống dưới `MAX_VISIBLE_LANES` | Không cấp lane cho cha đó; tăng `GraphRow.truncated_parents` | Hiện chỉ báo `+N cha nữa` trên hàng (plan 02-06) |
| **Lane của hàng vượt giới hạn** | Có hơn `MAX_VISIBLE_LANES` nhánh sống đồng thời | **Vẫn trả hàng với lane thật** — không bao giờ bỏ hàng | Gập lane vào cột cuối kèm chỉ báo (plan 02-05) |

Trong mã, khác biệt này là **hai hàm với hai chữ ký**, ở `src-tauri/src/graph/lanes.rs`:

```rust
fn allocate_row_lane(lanes: &mut Vec<Option<String>>, id: &str) -> u16          // KHÔNG cap
fn allocate_parent_lane(lanes: &mut Vec<Option<String>>, id: &str) -> Option<u16>  // CÓ cap
```

### 3.1 Vì sao hai hàm chứ không phải một hàm trả `Option`

Gộp lại thành một hàm trả `Option` là cách viết ngắn hơn và **là nguyên nhân gốc của
một lỗi mất commit**. Khi bước 1 (lane của chính hàng) nhận `None` thì không có cách xử
lý nào đúng:

- `unwrap()` → panic, cả trang lịch sử trắng;
- `continue` → **âm thầm đánh rơi một commit**;
- gán một lane ngoài giới hạn → phá chính assertion mà cap dựng ra.

Đo thật trên hình dạng của `wide` (25 lane đồng thời, cap 20): cài đặt dùng chung một
hàm **đánh rơi 10 trong 73 hàng**. Và mọi fixture nhỏ — `linear`, `octopus`, `orphan` —
vẫn xanh hết, vì chúng không bao giờ chạm cap.

Đó đúng là dạng lỗi tệ nhất mà phase này có thể ship: **đồ thị sai trên repo thật trong
khi trông hoàn toàn đúng trên repo nhỏ**. Test
`gioi_han_lane_khong_ap_cho_hang_khong_bo_hang_nao` (40 nhánh sống, cap 20) tồn tại chỉ
để bắt đúng nó, và khi chạy kiểm chứng đột biến thì **nó là test duy nhất trong cả bộ**
phát hiện ra — 16 test còn lại vẫn xanh.

### 3.2 Điều **không** xảy ra

- **Không** cuộn ngang vô hạn. Cột đồ thị có bề rộng trần; lane vượt trần được gập.
- **Không hàng nào biến mất khỏi danh sách.** `rows.len() == commits.len()` là bất biến
  tuyệt đối, **không phụ thuộc hằng số hiển thị**. Một commit vắng mặt là lỗi người
  dùng không thể phát hiện và không thể tự cứu — tệ hơn nhiều so với một đồ thị vẽ gập,
  vốn nhìn là thấy ngay.

Một ghi chú về hệ quả tất yếu: vì lane của hàng không bị cap, một lane vượt cap **vẫn
đi xuyên qua** các hàng sau ở bước 4, nên `passthrough` có thể chứa `Edge` với
`to_lane ≥ MAX_VISIBLE_LANES`. Đó không phải tràn mà là hệ quả trực tiếp của việc không
bỏ hàng — chính frontend gập chúng. Điều `assign` bảo đảm hẹp hơn nhưng đủ dùng: nó
**không bao giờ cấp mới** một lane rẽ nhánh vượt cap.

---

## 4. Suy giảm **hiển thị**, không phải suy giảm **dữ liệu**

Ranh giới này tuyệt đối:

- `Commit.parents` **giữ đủ mọi cha**, kể cả cha không được vẽ đường kẻ. Merge 15 cha
  vẫn có 15 phần tử trong `parents`.
- Chi tiết commit ở plan 02-05 **liệt kê hết** cha, không phụ thuộc `truncated_parents`.
- Chỉ **đường kẻ** bị lược, và số đường bị lược được báo ra tường minh qua
  `GraphRow.truncated_parents` để giao diện nói rõ với người dùng rằng có thứ chưa vẽ.

Test `merge_muoi_lam_cha_khong_mat_cha_nao_trong_so_sach` ghim điều này bằng một phép
cộng: `out_edges.len() + truncated_parents == 15`. Không cha nào được phép biến mất
khỏi sổ sách, kể cả khi không vẽ hết.

---

## 5. Còn lại cho plan sau

- **Plan 02-05** cài việc gập lane ≥ `MAX_VISIBLE_LANES` vào cột cuối kèm chỉ báo, và
  chốt `LANE_WIDTH` / `GRAPH_PADDING_LEFT` đúng bằng con số phép tính ở mục 2.1 dùng.
  Lệch con số thì cap 20 không còn khớp bề rộng thật và mục 2.1 phải tính lại.
- **Plan 02-06** vẽ chỉ báo `+N cha nữa` từ `GraphRow.truncated_parents`.
- **Bề rộng động.** Cap 20 suy từ cửa sổ *mặc định* 1440 px. Ở bề rộng tối thiểu 900 px
  thì vùng giữa chỉ còn ~225 px và chưa tới 20 lane lọt vào. Cap là trần trên cho việc
  **cấp lane**, còn việc gập theo bề rộng *thực tế* tại thời điểm vẽ là của frontend.
  Nếu sau này muốn cap thành tham số truyền vào thay vì hằng số, chữ ký `assign` là chỗ
  đổi — hàm vốn đã thuần nên việc đó không kéo theo gì khác.
