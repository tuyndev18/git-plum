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
 *
 * **Tăng 14 → 22px ngày 2026-09-23** để nút commit là avatar 18px có chữ cái
 * như GitKraken (ảnh người dùng gửi). Ở 14px avatar chỉ còn 8px, chữ cái không
 * vẽ được và nút đọc như một chấm màu mờ — người dùng báo "rất khó nhìn". Cap
 * 20 giữ nguyên; đổi lại cột đồ thị rộng hơn khi repo thật sự có nhiều lane
 * (`graphWidth` vẫn co theo số lane quan sát được).
 */
export const LANE_WIDTH = 22

/** Lề trái của cột đồ thị trước lane 0. */
export const GRAPH_PADDING_LEFT = 12

/**
 * Bán kính nút commit.
 *
 * Nút ngồi **trên** đường lane chứ không lấp kín ô — nhỏ so với chiều cao hàng,
 * to so với độ dày đường.
 *
 * # Ràng buộc cứng: nút + quầng KHÔNG được rộng hơn một lane
 *
 * ```text
 *   nút thường  r=9 + NODE_HALO_WIDTH 2  → đường kính 22px, lane 22px → khít
 *   nút merge   r=5 + NODE_HALO_WIDTH 2  → đường kính 14px, lane 22px → thoáng
 * ```
 *
 * Vượt con số này thì quầng nền của một nút **xoá mất đường lane bên cạnh** —
 * quầng tô bằng màu nền, nên nó không chỉ chồng lên mà cắt hẳn. Ở mật độ cao,
 * hai nhánh song song sát nhau sẽ có một nhánh đứt quãng tại mỗi hàng nhánh
 * kia có commit. Đây là lý do `NODE_RADIUS` giảm 5 → 4 và `MERGE_NODE_RADIUS`
 * 6 → 5 khi `LANE_WIDTH` xuống 22 → 14px: hai bộ số phải đi cùng nhau. Ngày
 * 2026-09-23 lane quay lại 22px nên nút thường lên r=9 (avatar như GitKraken).
 *
 * Theo chiều dọc: đường kính 18px trong ô cao `ROW_HEIGHT` 28px còn 10px trống
 * trên dưới, đủ để đoạn lane giữa hai hàng vẫn thấy.
 */
export const NODE_RADIUS = 9

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
 * Bán kính nút của merge commit — **chấm đặc nhỏ** màu lane, không avatar,
 * đúng cách GitKraken phân biệt merge (ảnh tham chiếu 2026-09-23): merge không
 * mang nội dung tác giả đáng xem, chấm nhỏ để mắt lướt qua nó.
 */
export const MERGE_NODE_RADIUS = 5

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
 * để lòng nút khớp nền thật ở **cả hai theme**.
 *
 * Giá trị đổi sang tối Linear (`#08090a`) cùng lúc với reskin theo design
 * system Linear (260923-lhl) — theme TỐI Linear giờ là MẶC ĐỊNH (`--bg` ở
 * `:root`, đảo lại quyết định D-01 của 260923-kzl nơi cream sáng là mặc định),
 * nên giá trị dự phòng phải khớp nền mặc định mới, đúng ý nghĩa doc comment
 * trên không đổi: "trùng nền của hàng phía sau nút".
 */
export const NODE_FILL = '#08090a'

/** Tên biến CSS giữ màu tô lòng nút. Khai ở `app.css`, đọc lúc chạy. */
export const NODE_FILL_VAR = '--graph-node-fill'

/**
 * Màu vòng tròn đánh dấu hàng đang được chọn — trung tính, không theo màu lane.
 *
 * Đây chỉ là **giá trị dự phòng** cho môi trường không có DOM (test), cùng vai
 * trò với `NODE_FILL` ở trên. Lúc chạy thật, `canvasRenderer` đọc
 * `--graph-selection-ring` qua `readSelectionRing()`. Giá trị đổi từ
 * `#46443b` (border-strong tối của bản Cursor, chọn để đọc được trên nền
 * cream) sang một tông SÁNG hơn cùng lúc với reskin Linear (260923-lhl):
 * `#34343a` (`--border-strong` mới của theme tối Linear) có tương phản THẤP
 * trên `#08090a` vì cả hai đều tối — `#5a5c66` (khớp `--graph-selection-ring`
 * mới trong app.css) thực sự đọc được trên nền tối Linear, đúng mục đích ban
 * đầu của hằng số này ("vòng chọn nhìn thấy được").
 */
export const SELECTION_RING = '#5a5c66'

/** Tên biến CSS giữ màu vòng chọn. Khai ở `app.css`, đọc lúc chạy. */
export const SELECTION_RING_VAR = '--graph-selection-ring'

/**
 * Độ dày đường lane.
 *
 * **Giảm 2 → 1,5px cùng lúc với `LANE_WIDTH` 22 → 14px** (2026-09-22). Hai con
 * số phải đi cùng nhau: đường 2px trong lane 14px chiếm 14% bề rộng lane, so
 * với 9% ở lane 22px — đường dày trong lane hẹp làm khoảng trắng giữa hai lane
 * cạnh nhau co lại và đồ thị đọc như một khối sọc đặc.
 *
 * Tỉ lệ nhắm tới là **đường mảnh, nút to** — đọc từ ảnh tham chiếu người dùng
 * gửi: đường lane gần như sợi chỉ, còn nút commit nổi hẳn lên. Tương phản đó
 * là thứ cho mắt bám vào nút trước rồi mới lần theo đường, thay vì phải tách
 * nút khỏi một đường cùng độ dày.
 *
 * Không xuống 1px: WebView2 vẽ đường 1px ở `devicePixelRatio` lẻ (125% scale,
 * mặc định trên nhiều máy Windows) bị nhoè sang hai pixel và mất màu.
 */
export const EDGE_WIDTH = 1.5

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
 *   × ngân sách cột đồ thị     × 62%     khi repo thật sự có đủ 20 lane
 *   = cột đồ thị              ≈ 464 px
 *   − GRAPH_PADDING_LEFT       −  12 px
 *   ÷ LANE_WIDTH               ÷  22 px
 *   = 20,5                    →  20 lane
 * ```
 *
 * Ngân sách 40% → 62% khi lane 14 → 22px (2026-09-23, avatar như GitKraken).
 * Chỉ repo bệnh lý mới chạm trần này; repo thường vài lane thì `graphWidth`
 * vẫn hẹp.
 *
 * **Đổi 13 → 20 ngày 2026-09-22, có số đo.** Cap 13 làm **19,99%** hàng của
 * repo perf 100 007 commit bị gập vào cột cuối — tới tám lane vẽ chung một cột,
 * không chỉ báo gì. Cap 20 cho **0,71%**. Xem doc comment dài ở
 * `src-tauri/src/graph/types.rs` cho cả hai ca đo được và lý do chốt.
 *
 * Phần vượt cap vẫn hiện bằng chỉ báo `+N` (HIST-03).
 *
 * **PHẢI khớp `MAX_VISIBLE_LANES` ở `src-tauri/src/graph/types.rs`.** Lệch hai
 * phía là lỗi im lặng: backend cấp lane 19 mà frontend chỉ vẽ tới 13 thì hai
 * nhánh khác nhau bị vẽ đè lên cùng một cột. Có test hai phía ghim con số này.
 */
export const MAX_VISIBLE_LANES = 20

/**
 * Bảng màu lane — **sinh bằng công thức giãn đều hue**, không chọn tay.
 *
 * # Vì sao bỏ bảng đo-từ-ảnh-tham-chiếu
 *
 * Bảng trước có 13 màu: bảy đo pixel từ `docs/screenshots/`, sáu bù thêm vào
 * các khoảng hue tham chiếu bỏ trống. Nó chết theo cap: cap lên 20 thì cần 20
 * màu, và "bù thêm bảy màu nữa vào chỗ trống" là cách chọn màu không có đáy —
 * mỗi màu thêm vào làm khoảng trống còn lại hẹp đi, nên bảy màu cuối chắc chắn
 * là bảy màu khó phân biệt nhất.
 *
 * Người dùng đã báo chính điều đó ở 13 màu ("màu lane khó phân biệt"), nên nhân
 * đôi cách làm cũ là nhân đôi vấn đề. Công thức dưới đây cho khoảng cách hue
 * **đều nhau theo định nghĩa**, không phụ thuộc mắt ai chọn.
 *
 * Mất gì: bảng không còn khớp GitKraken nữa. Đó là đánh đổi có ý thức — khớp
 * tham chiếu về màu có giá trị khi số lane ít; ở 20 lane cùng lúc, phân biệt
 * được nhau quan trọng hơn giống một công cụ khác.
 *
 * Bảng **phải có đúng `MAX_VISIBLE_LANES` màu** và khớp `LANE_COLORS` bên Rust
 * (`src-tauri/src/graph/types.rs`), vì `GraphRow.color` đến từ backend đã là
 * `lane % LANE_COLORS`. Thiếu màu thì `colorFor` gập lần nữa và hai lane vẽ
 * được cùng lúc lại trùng màu — đúng lỗi vừa sửa. Có test ghim hai phía.
 */
export const LANE_COLORS: readonly string[] = [
  // Sinh bằng công thức, KHÔNG chọn tay từng màu — xem doc comment trên.
  //
  //   hue  = ((i * 9) % 20) * 18 + 9      bước nhảy 9, nguyên tố cùng nhau với 20
  //   sat  = 74%                          cố định
  //   light = i chẵn ? 60% : 50%          lệch sáng xen kẽ
  //
  // Bước nhảy 9 làm **lane liền kề cách nhau đúng 162° hue** — gần đối xứng
  // trên vòng màu, tức khoảng cách lớn nhất có thể giữ đều được. Đi tuần tự
  // (18° mỗi bước) sẽ cho lane 0 và lane 1 gần như cùng màu, mà lane cạnh nhau
  // mới chính là cặp người dùng phải phân biệt.
  //
  // Lệch sáng xen kẽ là lớp tách thứ hai: hai màu rơi vào cùng vùng hue vẫn
  // khác nhau về độ sáng. Quan trọng cho người mù màu, vốn không đọc được
  // khoảng cách hue nhưng đọc được độ sáng.
  '#e4644e', // lane 0:  9°  L60
  '#21dec2', // lane 1:  171° L50
  '#e44e91', // lane 2:  333° L60
  '#21de50', // lane 3:  135° L50
  '#dd4ee4', // lane 4:  297° L60
  '#63de21', // lane 5:  99°  L50
  '#824ee4', // lane 6:  261° L60
  '#d4de21', // lane 7:  63°  L50
  '#4e73e4', // lane 8:  225° L60
  '#de7621', // lane 9:  27°  L50
  '#4ecee4', // lane 10: 189° L60
  '#de213d', // lane 11: 351° L50
  '#4ee4a1', // lane 12: 153° L60
  '#de21af', // lane 13: 315° L50
  '#55e44e', // lane 14: 117° L60
  '#9c21de', // lane 15: 279° L50
  '#b0e44e', // lane 16: 81°  L60
  '#2b21de', // lane 17: 243° L50
  '#e4bf4e', // lane 18: 45°  L60
  '#2189de', // lane 19: 207° L50
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
