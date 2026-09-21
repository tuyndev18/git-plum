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
 * Tham chiếu vẽ nút là vòng tròn có **viền dày** đường kính ~16px, không phải
 * chấm đặc nhỏ. Bán kính 7px + viền 2.5px cho đường kính ngoài ~17px, khớp
 * tham chiếu và đủ lớn để phân biệt commit thường với merge.
 */
export const NODE_RADIUS = 7

/** Độ dày viền nút commit. */
export const NODE_STROKE_WIDTH = 2.5

/**
 * Bán kính nút của merge commit — lớn hơn nút thường một chút để merge nổi bật
 * khi lần theo lịch sử, đúng cách tham chiếu phân biệt hai loại.
 */
export const MERGE_NODE_RADIUS = 8

/**
 * Màu tô tâm nút commit — phải trùng **nền của cột đồ thị**, không phải màu
 * lane.
 *
 * Tâm nút được tô nền trước rồi mới vẽ viền, nên đường lane chạy phía sau bị
 * cắt đúng trong lòng nút. Đó là cách tham chiếu làm nút "ngồi trên" đường thay
 * vì bị đường xuyên qua giữa. Nếu đổi nền `.graph-canvas` trong `app.css` thì
 * phải đổi cả con số này, nếu không lòng nút sẽ hiện thành một đốm khác màu.
 */
export const NODE_FILL = '#161b22'

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
 * Bảng màu lane, tự thiết kế — không sao chép GitKraken. Đủ tương phản trên
 * cả nền tối và nền sáng của `app.css`. Độ dài bảng không cần khớp
 * `LANE_COLORS` bên Rust (7 màu): `GraphRow.color` đã là `lane % 7` từ backend,
 * và `colorFor` chỉ việc gập chỉ số đó (hoặc bất kỳ số nào lớn hơn) vào bảng
 * của chính nó bằng modulo — hai bên độc lập về số lượng màu.
 */
export const LANE_COLORS: readonly string[] = [
  '#e06c75', // đỏ san hô
  '#61afef', // xanh dương
  '#98c379', // xanh lá
  '#e5c07b', // vàng nghệ
  '#c678dd', // tím
  '#56b6c2', // xanh ngọc
  '#d19a66', // cam đất
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
