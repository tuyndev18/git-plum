/**
 * Hình học thuần cho cột đồ thị — không DOM, không React.
 *
 * Đây là **nguồn duy nhất** của `ROW_HEIGHT`: `CommitList.tsx` (Task 2) đọc
 * hằng này cho `estimateSize` của `useVirtualizer`, và `rowY()` phải sinh đúng
 * con số mà virtualizer báo lại qua `virtualItem.start` cho cùng chỉ số hàng.
 * Có hai nguồn tính toạ độ Y là cách chắc chắn nhất để đồ thị lệch hàng —
 * xem test "bất biến thẳng hàng" ở `geometry.test.ts`.
 */

import type { GraphRow } from '@/lib/ipc'

/** Chiều cao cố định của một hàng commit, tính bằng px. */
export const ROW_HEIGHT = 28

/**
 * Bề rộng mỗi lane, tính bằng px.
 *
 * Đo từ ảnh tham chiếu (`docs/screenshots/`): nút commit ở lane 0 và nhánh con
 * ở lane 1 cách nhau khoảng 22px. Con số 14px trước đây quá chật — nút chồng
 * gần nhau và đường rẽ gần như trùng hướng đường dọc bên cạnh.
 *
 * Đổi số này **đổi cả cap lane**: xem `MAX_VISIBLE_LANES` bên dưới và
 * `docs/04-phase2-degraded-graph.md` mục 2.1.
 */
export const LANE_WIDTH = 22

/** Lề trái của cột đồ thị trước lane 0. */
export const GRAPH_PADDING_LEFT = 12

/**
 * Bán kính nút commit.
 *
 * Tham chiếu vẽ nút là vòng tròn có viền, **nhỏ so với chiều cao hàng**: nút
 * ngồi trên đường lane chứ không lấp kín ô. Bán kính 5px + viền 2px cho đường
 * kính ngoài 12px trong ô cao `ROW_HEIGHT` = 28px — còn 8px khoảng trống trên
 * và dưới nút, nên đoạn lane nối giữa hai hàng vẫn nhìn thấy rõ.
 *
 * Bản trước dùng 7px + viền 2.5px (đường kính ngoài 17px): nút của hai hàng kề
 * nhau chỉ còn cách nhau 11px, đoạn lane giữa chúng gần như biến mất và đồ thị
 * trông như một chuỗi vòng tròn rời rạc thay vì một cái cây.
 */
export const NODE_RADIUS = 5

/** Độ dày viền nút commit — chỉ dùng cho nút nét đứt của hàng `terminates`. */
export const NODE_STROKE_WIDTH = 2

/**
 * Bề rộng quầng nền quanh chấm commit, px.
 *
 * Quầng tô bằng màu nền khung trước khi tô chấm, nên nó **cắt** mọi đường lane
 * và cạnh merge chạy sát phía sau nút. Không có quầng thì ở mật độ cao một
 * chấm màu lane nằm đè lên một đường cùng màu lane là không phân biệt được —
 * đúng hiện tượng "khó nhìn" ở ảnh 13 lane.
 *
 * Tham chiếu đạt hiệu quả này bằng ảnh avatar 22px che hẳn vùng quanh nút;
 * git-plum không gọi mạng nên không có avatar, quầng nền là cách tương đương
 * rẻ nhất.
 */
export const NODE_HALO_WIDTH = 2

/**
 * Bán kính nút của merge commit — lớn hơn nút thường một chút để merge nổi bật
 * khi lần theo lịch sử, đúng cách tham chiếu phân biệt hai loại.
 */
export const MERGE_NODE_RADIUS = 6

/**
 * Màu tô tâm nút commit — phải trùng **nền của hàng phía sau nút**, không phải
 * màu lane.
 *
 * Tâm nút được tô nền trước rồi mới vẽ viền, nên đường lane chạy phía sau bị
 * cắt đúng trong lòng nút. Đó là cách tham chiếu làm nút "ngồi trên" đường thay
 * vì bị đường xuyên qua giữa.
 *
 * Đây chỉ là **giá trị dự phòng** cho môi trường không có DOM (test). Lúc chạy
 * thật, `canvasRenderer` đọc `--graph-node-fill` từ `getComputedStyle` của host
 * để lòng nút khớp nền thật ở **cả hai theme** — hằng số cứng `#161b22` của bản
 * trước là một đốm đen giữa nền trắng khi người dùng ở theme sáng, và không
 * khớp cả `--bg` (`#16161a`) lẫn dải xen kẽ của `.commit-row` ở theme tối.
 */
export const NODE_FILL = '#16161a'

/** Tên biến CSS giữ màu tô lòng nút. Khai ở `app.css`, đọc lúc chạy. */
export const NODE_FILL_VAR = '--graph-node-fill'

/** Màu vòng tròn đánh dấu hàng đang được chọn — trung tính, không theo màu lane. */
export const SELECTION_RING = '#e6edf3'

/**
 * Độ dày đường lane. Tham chiếu vẽ lane đủ đậm để phân biệt màu ở tỉ lệ 100%;
 * đường 1px mặc định bị mảnh và nhoè khi nhiều lane cạnh nhau.
 */
export const EDGE_WIDTH = 2

/**
 * Bề rộng cột nhãn nhánh/tag, px — **cố định**, không theo nội dung.
 *
 * Tham chiếu (`docs/screenshots/`) có cột `BRANCH / TAG` rộng cố định ở ngoài
 * cùng bên trái, nhãn dài bị cắt bằng ellipsis chứ không nới cột.
 *
 * Vì sao cố định chứ không `max-content`: cột đồ thị nằm ngay sau nó, và canvas
 * phải biết dịch sang phải bao nhiêu px để vẽ đúng chỗ. Cột co theo nội dung
 * nghĩa là canvas phải đo DOM mỗi lần nhãn đổi — thêm một nguồn số liệu có thể
 * lệch, đúng lớp lỗi đã gây ra hai vòng checkpoint thất bại. Cố định thì cả
 * CSS và canvas đọc cùng một hằng số này.
 *
 * PHẢI khớp `--ref-col-width` trong `app.css`; có test ghim hai phía.
 */
export const REF_COL_WIDTH = 132

/**
 * Số lane tối đa được **cấp mới**, chốt bằng phép tính hiển thị — xem
 * `docs/04-phase2-degraded-graph.md` mục 2.1.
 *
 * ```text
 *   cửa sổ mặc định            1440 px
 *   × vùng giữa                × 52%     AppLayout Panel id="main"
 *   × ngân sách cột đồ thị     × 40%     60% còn lại cho thông điệp commit
 *   = cột đồ thị              ≈ 300 px
 *   − GRAPH_PADDING_LEFT       −  12 px
 *   ÷ LANE_WIDTH               ÷  22 px
 *   = 13,1                    →  13 lane
 * ```
 *
 * Giảm từ 20 xuống 13 khi `LANE_WIDTH` tăng 14→22px cho khớp ảnh tham chiếu.
 * Đây là đánh đổi có chủ ý: lane thoáng hơn, đọc dễ hơn, nhưng số nhánh vẽ
 * được đồng thời ít hơn — phần vượt cap hiện bằng chỉ báo `+N` (HIST-03).
 *
 * **PHẢI khớp `MAX_VISIBLE_LANES` ở `src-tauri/src/graph/types.rs`.** Lệch hai
 * phía là lỗi im lặng: backend cấp lane 19 mà frontend chỉ vẽ tới 13 thì hai
 * nhánh khác nhau bị vẽ đè lên cùng một cột. Có test hai phía ghim con số này.
 */
export const MAX_VISIBLE_LANES = 13

/**
 * Bảng màu lane — **đo từ ảnh tham chiếu**, xem `docs/06-graph-render-model.md`
 * mục 3.
 *
 * Bảng cũ là dải pastel kiểu One Dark (`#e06c75 #61afef #98c379 …`). Đo pixel
 * đường lane của tham chiếu cho thấy nó dùng dải **bão hoà cao** hẳn, và
 * không dùng vàng/xanh lá — hai màu khó tách khỏi nhau và khỏi cam trên nền
 * tối `#16161a` khi đường chỉ dày 2px. Ở mật độ 13 lane cùng lúc (ảnh người
 * dùng gửi), chênh lệch này là khác biệt giữa "lần theo được một nhánh" và
 * "một mớ sọc".
 *
 * Bảng **phải có đúng `MAX_VISIBLE_LANES` màu** và khớp `LANE_COLORS` bên Rust
 * (`src-tauri/src/graph/types.rs`), vì `GraphRow.color` đến từ backend đã là
 * `lane % LANE_COLORS`. Thiếu màu thì `colorFor` gập lần nữa và hai lane vẽ
 * được cùng lúc lại trùng màu — đúng lỗi vừa sửa. Có test ghim hai phía.
 */
export const LANE_COLORS: readonly string[] = [
  // Bảy màu đo trực tiếp từ ảnh tham chiếu.
  '#15a0bf', // xanh ngọc
  '#0669f7', // xanh dương
  '#8e00c2', // tím
  '#c517b6', // hồng tím
  '#d90171', // hồng sen
  '#cd0101', // đỏ
  '#f25d2e', // cam
  // Sáu màu bù, chèn vào các khoảng hue mà tham chiếu bỏ trống, để đủ
  // MAX_VISIBLE_LANES màu phân biệt. Giữ cùng mức bão hoà với bảy màu trên,
  // nếu không lane 7..12 sẽ trông "nhạt hơn" và đọc như một lớp thứ cấp.
  '#e8a002', // hổ phách
  '#7cb342', // xanh lá
  '#00897b', // xanh lục lam
  '#5c6bc0', // chàm
  '#a1887f', // nâu
  '#ec407a', // hồng đào
]

/** Toạ độ Y (px) của hàng thứ `index`. Nguồn duy nhất — xem doc comment đầu tệp. */
export function rowY(index: number): number {
  return index * ROW_HEIGHT
}

/** Toạ độ X (px) tâm của một lane, đã gập theo `graphWidth` nếu `lane` vượt cap. */
export function laneX(lane: number): number {
  const clamped = Math.min(lane, MAX_VISIBLE_LANES - 1)
  return GRAPH_PADDING_LEFT + clamped * LANE_WIDTH
}

/**
 * Bề rộng cột đồ thị (px) cho một `maxLane` quan sát được. Tăng đơn điệu theo
 * `maxLane` nhưng bị chặn trên tại `MAX_VISIBLE_LANES` — chống tràn ngang vô
 * hạn trên repo có hình dạng bệnh lý (T-02-18).
 */
export function graphWidth(maxLane: number): number {
  const visibleLanes = Math.min(Math.max(maxLane, 0) + 1, MAX_VISIBLE_LANES)
  return GRAPH_PADDING_LEFT + visibleLanes * LANE_WIDTH
}

/** Màu của lane thứ `i`, gập vào bảng bằng modulo — không bao giờ `undefined`. */
export function colorFor(i: number): string {
  const color = LANE_COLORS[((i % LANE_COLORS.length) + LANE_COLORS.length) % LANE_COLORS.length]
  // Bảng màu không rỗng (khẳng định ở test); phép modulo trên luôn nằm trong
  // [0, length). Non-null assertion an toàn vì bất biến đó.
  return color as string
}

/**
 * Hàng nào đang chưa có dữ liệu (đang nạp trang). Dùng bởi `CommitList` để
 * quyết định gọi `ensureRange` — đặt ở đây vì nó là hình học/chỉ số thuần,
 * không phải logic store.
 */
export function isRowLoaded(row: GraphRow | undefined): boolean {
  return row !== undefined
}
