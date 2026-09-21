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

/** Bề rộng mỗi lane, tính bằng px — chốt cùng phép tính ở `docs/04-phase2-degraded-graph.md`. */
export const LANE_WIDTH = 14

/** Lề trái của cột đồ thị trước lane 0. */
export const GRAPH_PADDING_LEFT = 8

/** Bán kính nút tròn đánh dấu một commit. */
export const NODE_RADIUS = 4

/**
 * Số lane tối đa được **cấp mới**, chốt ở `docs/04-phase2-degraded-graph.md`
 * mục 2.1 bằng phép tính hiển thị (cửa sổ 1440px × 52% × 40% cột đồ thị).
 *
 * Đây là hằng số ràng buộc *hiển thị* của frontend, khớp với `MAX_VISIBLE_LANES`
 * ở `src-tauri/src/graph/types.rs`. Backend dùng nó để **không cấp** lane cha
 * vượt cap; frontend dùng nó để **gập** lane hàng (có thể vượt cap, xem
 * `docs/04-phase2-degraded-graph.md` mục 3) vào cột cuối khi vẽ.
 */
export const MAX_VISIBLE_LANES = 20

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
