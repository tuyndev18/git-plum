/**
 * Hàng WIP (WORK-11) — **nguồn duy nhất** của mọi phép ±1 và mọi độ lệch chỉ số.
 *
 * Module **thuần**: không React, không DOM, không store. Cùng tinh thần
 * `geometry.ts`, vốn đã là "nguồn duy nhất của `ROW_HEIGHT`".
 *
 * # 🔴 Vì sao tệp này tồn tại
 *
 * `CommitList` có **đúng một** lời gọi `useVirtualizer`, và `GraphCanvas` vẽ từ
 * **cùng** mảng `virtualItems`. Đó là bất biến cốt lõi của Phase 2, và nó tồn tại vì
 * lớp lỗi "cột đồ thị lệch cột văn bản **đúng một hàng**" đã xảy ra **hai lần**.
 *
 * Hàng WIP va thẳng vào bất biến đó: nó là một hàng thứ 0 không phải commit. Mọi
 * cách cài đặt đều phải trả lời "chỉ số nào ánh xạ sang `commits[?]`", và câu trả
 * lời sai một đơn vị ở **một** chỗ tiêu thụ là đủ để hai cột lệch nhau.
 *
 * # Quyết định: **cách B** — hàng WIP nằm NGOÀI virtualizer hoàn toàn
 *
 * **Cách A — `count: total + 1`, mọi chỗ đọc `commits[index - 1]` — bị TỪ CHỐI.**
 * Nó đặt một phép `-1` vào **mọi** chỗ tiêu thụ `virtualItems`: `renderRows`,
 * `commits[v.index]`, `graphRows[v.index]`, `ensureRange(first.index, last.index+1)`,
 * `indexById`, `scrollToIndex`, `scrollToCommit`. Đó là **bảy** chỗ, và ROADMAP nói
 * mọi phép cộng/trừ chỉ số phải nằm ở **một** chỗ. Bảy chỗ là bảy dịp để lệch.
 *
 * **Cách B — đã chọn.** `count: total` **không đổi**. Hàng WIP là một phần tử **anh
 * em** (sibling) của vùng cuộn, ghim ở đầu danh sách, cao đúng `ROW_HEIGHT`. Không
 * chỉ số nào đổi. Phép "+1" duy nhất tồn tại là **độ lệch pixel** ([`contentOffset`])
 * áp cho vùng nội dung bên dưới — **một** phép cộng, ở **một** tệp, và cả cột đồ thị
 * lẫn cột văn bản đều đọc nó qua **cùng** hàm [`commitRowY`].
 *
 * ROADMAP đòi hàng WIP "không được đếm vào `total` của virtualizer theo cách làm lệch
 * ánh xạ `commits[index]`". Cách B không đếm nó vào `total` **một cách nào cả** — nên
 * ánh xạ `commits[index]` **không thể** lệch, về mặt **cấu trúc**, chứ không phải nhờ
 * sửa cho thẳng. Đó cùng là lập luận `CommitList.tsx` dùng cho "một virtualizer".
 *
 * **Hệ quả phải chấp nhận, ghi rõ:** hàng WIP **không cuộn đi** cùng danh sách. Với
 * WORK-11 đó là hành vi **đúng** — nó là chỗ vào vùng soạn commit, nên luôn bấm được.
 * Cần mắt người xác nhận ở checkpoint 04-05.
 *
 * # Bất biến có cổng ghim
 *
 * Tệp này là tệp **duy nhất** được phép chứa [`WIP_ROW_COUNT`] / [`WIP_ROW_HEIGHT`]
 * trong một biểu thức số học. Cổng ở `wipRow.test.ts` đọc nguồn của `graph-render/`
 * và `components/history/` (đã **bỏ chú thích và chuỗi** trước khi tìm) và đỏ nếu
 * tệp khác vi phạm.
 */

import { laneX, MAX_VISIBLE_LANES, ROW_HEIGHT } from '@/lib/graph-render/geometry'
import type { RepoStatus, WipCounts } from '@/lib/ipc'

/**
 * Số hàng mà hàng WIP chiếm: **một**.
 *
 * Là hằng số có tên chứ không phải chữ `1` rải rác, để chỗ nào phụ thuộc vào "hàng
 * WIP là một hàng" đều tìm được bằng một lần grep.
 */
export const WIP_ROW_COUNT = 1

/**
 * Chiều cao hàng WIP, px.
 *
 * 🔴 **Dẫn xuất từ `ROW_HEIGHT`, không viết `28`.** Một hằng số 28 thứ hai là đúng
 * lớp lỗi mà Phase 2 đã phải thêm test ghim hai phía cho `MAX_VISIBLE_LANES` và
 * `REF_COL_WIDTH`: hai nguồn cho một con số, và chúng lệch nhau được trong im lặng.
 * Lệch ở đây nghĩa là cột đồ thị lệch cột văn bản — lớp lỗi đã xảy ra hai lần.
 */
export const WIP_ROW_HEIGHT = ROW_HEIGHT

/**
 * Độ lệch pixel áp cho vùng nội dung bên dưới hàng WIP.
 *
 * 🔴 Đây là **phép cộng duy nhất** của cả tính năng. Không tệp nào khác được viết
 * `+ WIP_ROW_HEIGHT`; có cổng ghim.
 */
export function contentOffset(hasWip: boolean): number {
  return hasWip ? WIP_ROW_HEIGHT : 0
}

/**
 * Toạ độ Y (px) của một hàng commit trong khung nhìn.
 *
 * 🔴 **Một** hàm mà **cả** cột đồ thị **và** cột văn bản dùng. Với cùng đầu vào, hai
 * bên luôn cho cùng số — đó là toàn bộ nội dung của bất biến "một nguồn toạ độ Y".
 * Hai công thức là hai dịp để lệch một hàng.
 *
 * `hasWip = false` cho **đúng** giá trị hiện tại của `CommitList.tsx`
 * (`v.start - scrollTop`), nên bật tính năng lên không đổi hành vi khi không có thay
 * đổi chưa commit. Có test hồi quy ghim điều đó.
 *
 * @param virtualItemStart `virtualItem.start` do virtualizer báo về.
 * @param scrollTop Vị trí cuộn hiện tại của vùng cuộn.
 * @param hasWip Có hàng WIP đang hiện hay không.
 */
export function commitRowY(
  virtualItemStart: number,
  scrollTop: number,
  hasWip: boolean,
): number {
  return virtualItemStart - scrollTop + contentOffset(hasWip)
}

/**
 * Chỉ số commit ứng với một chỉ số của virtualizer — **đồng nhất**.
 *
 * 🔴 **Đồng nhất có chủ ý, không phải mã chết.** Ba lý do nó tồn tại:
 *
 * 1. Nó là chỗ **neo** để một cổng (và một người đọc) chứng minh rằng không có phép
 *    trừ nào trên đường chỉ số — một hàm trả `x => x` là bằng chứng đọc được, còn
 *    "không có phép trừ ở đâu cả" thì không đọc được.
 * 2. Nó là chỗ **duy nhất** phải sửa nếu sau này ai đó buộc phải đổi sang cách A.
 * 3. Nó làm chỗ tiêu thụ `virtualItems` **khai báo** ý định: `commits[commitIndexFor(
 *    v.index)]` nói rõ rằng phép ánh xạ đã được nghĩ tới, khác với `commits[v.index]`
 *    nói rằng chưa ai nghĩ tới.
 */
export function commitIndexFor(virtualIndex: number): number {
  return virtualIndex
}

/**
 * Số hàng mà virtualizer phải đếm.
 *
 * 🔴 Trả `total` trong **cả hai** ca. Hàng WIP **không** vào `total` — nguyên văn
 * ROADMAP, và là toàn bộ điểm của cách B. Trả `total + 1` ở đây là quay về cách A
 * bằng cửa sau, và nó sẽ làm `commits[index]` lệch ở bảy chỗ.
 *
 * Tham số `hasWip` được nhận **và cố ý bỏ qua**: nó buộc chỗ gọi phải viết ra rằng
 * câu hỏi "có hàng WIP không" đã được tính đến, và làm test ghim được **cả hai** ca.
 */
export function virtualizerCount(total: number, _hasWip: boolean): number {
  return total
}

/**
 * Số đếm hàng WIP suy từ [`RepoStatus.entries`] — đối ứng TS của
 * `RepoStatus::wip_counts` phía Rust.
 *
 * # 🔴 Vì sao phải viết lại ở đây thay vì đọc một trường IPC
 *
 * `wip_counts` phía Rust là một **method dẫn xuất**, có chủ ý: doc comment của
 * nó nói rằng một *trường* `wip_counts` có thể bị ghi từ bất kỳ đâu, kể cả từ
 * một chỗ đã đếm sai. Hệ quả là nó **không** được `serde` sinh ra, nên
 * `RepoStatus` trên dây IPC không mang nó — đã kiểm bằng `interface RepoStatus`
 * của `src/lib/ipc.ts`, chỉ có `branch`/`entries`/`hasConflicts`.
 *
 * Nên phía TS phải tự dẫn xuất từ **cùng** một `entries`. Đó là hai bản cài
 * của một quy tắc, và hai bản cài lệch nhau được trong im lặng — vì vậy quy
 * tắc được chép **nguyên văn** dưới đây và có test ghim cả hai ca đặc biệt mà
 * bản Rust ghim (`MM` đếm một; `A.` là tệp mới, không phải tệp sửa).
 *
 * # Quy tắc (nguyên văn từ `domain/status.rs`)
 *
 * * Đếm theo **đường dẫn duy nhất**, không theo số phần tử: một tệp `MM` sinh
 *   **hai** phần tử (một mỗi nhóm) nhưng người dùng chỉ sửa **một** tệp.
 * * `added` — nhóm `untracked`, **hoặc** XY chứa `A` (đã stage thêm mới). Một
 *   tệp đã `git add` xong vẫn là tệp mới với mắt người dùng.
 * * `modified` — mọi tệp theo dõi còn lại. Tệp đã đếm `added` **không** đếm lại.
 */
export function wipCountsFromStatus(status: RepoStatus | undefined | null): WipCounts {
  if (!status) return { modified: 0, added: 0 }

  const daThem = new Set<string>()
  const daSua = new Set<string>()

  for (const e of status.entries) {
    if (e.group === 'untracked' || e.xy.includes('A')) daThem.add(e.path)
    else daSua.add(e.path)
  }

  // Một tệp vừa stage thêm mới (`A.`) vừa sửa tiếp ở worktree (`.M`) sinh hai
  // phần tử rơi vào hai tập. Với mắt người dùng đó là **một tệp mới**, nên
  // `added` thắng — giống hệt vòng lặp tương ứng phía Rust.
  for (const p of daThem) daSua.delete(p)

  return { modified: daSua.size, added: daThem.size }
}

/**
 * Có hiện hàng WIP hay không.
 *
 * Tiêu chí 7 của ROADMAP: **commit xong thì hàng đó biến mất.** `{0, 0}` nghĩa là
 * không còn gì chưa commit, nên không có "công việc đang làm" để hiện.
 *
 * `undefined` (chưa nạp `RepoStatus` lần nào) cũng là `false`: hiện một hàng WIP với
 * số đếm chưa biết là hiện một con số bịa.
 */
export function hasWipRow(counts: WipCounts | undefined | null): boolean {
  if (!counts) return false
  return counts.modified > 0 || counts.added > 0
}

/** Đầu vào của [`wipEdge`]. */
export interface WipEdgeInput {
  /**
   * Lane **THẬT** của HEAD, **chưa clamp**.
   *
   * 🔴 `null`/`undefined` khi HEAD không nằm trong phần lịch sử đã nạp. Đó là ca
   * thật, không phải phòng thủ thừa: HEAD ở một nhánh cũ hơn số hàng đã nạp thì
   * không có hàng nào mang sha của nó.
   */
  headLane: number | null | undefined
  /**
   * Lane của **hàng 0**, chỉ để ghi lại trong test rằng nó **không** được dùng.
   *
   * 🔴 Hàng 0 và HEAD là **hai thứ khác nhau**, và nhầm chúng là gốc của cả lỗi này —
   * xem doc comment của [`wipEdge`]. Trường này cố ý **không** tham gia phép tính.
   */
  rowZeroLane?: number
}

/**
 * Cạnh của hàng WIP, dạng union — **không bao giờ** `null` im lặng.
 *
 * Ba biến thể ứng với ba tình huống có thật, và mỗi biến thể mang đúng dữ liệu mà
 * giao diện cần để nói thật với người dùng.
 */
export type WipEdge =
  /** Vẽ được: HEAD nằm trong tầm cột đồ thị. */
  | { kind: 'normal'; lane: number; x: number }
  /**
   * HEAD ở lane ≥ `MAX_VISIBLE_LANES`. **Suy giảm có chủ ý** — cùng tinh thần
   * `truncated_parents` / chỉ báo `+N` mà ROADMAP Phase 2 đã chốt. Mang `realLane`
   * để giao diện hiện được chỉ báo thay vì im lặng.
   */
  | { kind: 'clamped'; realLane: number }
  /** Không biết HEAD ở đâu. Không đoán. */
  | { kind: 'unknownHead' }

/**
 * Cạnh nối hàng WIP xuống **HEAD**.
 *
 * # 🔴 Hàng WIP nối xuống HEAD. Nó KHÔNG nối xuống hàng 0.
 *
 * `git log --all --topo-order` xếp commit mới nhất theo topo của **MỌI** ref ở hàng
 * 0. Nếu người dùng đang ở một nhánh **không** phải nhánh mới nhất, HEAD nằm **giữa**
 * danh sách và lane của nó là bất kỳ. Đã tái hiện có số 2026-09-22 bằng
 * `cargo run --release --bin lanedist`:
 *
 * ```text
 * hang 0: lane 0 (commit moi nhat theo topo cua MOI ref)
 * HEAD that su: hang 19, lane 19 -> canh hang WIP BI CLAMP, se ve sai cot
 * ```
 *
 * # 🔴 Vì sao không gọi thẳng `laneX(headLane)`
 *
 * `laneX` clamp **vô điều kiện** (`Math.min(lane, MAX_VISIBLE_LANES - 1)`) và không
 * có đường cho người gọi chọn khác. Với hàng **commit** phép clamp đó đúng: nhiều
 * lane đè một cột là suy giảm **nhìn-là-thấy** (cột chen chúc). Với cạnh WIP nó
 * **sai**, vì nó tạo ra một phép nối **cụ thể và sai** — người dùng thấy hàng WIP nối
 * vào một commit **khác**. Đó là một đồ thị vẽ sai một cách **tự tin**.
 *
 * Nên phép so sánh ở đây nằm ở miền **lane**, **trước** khi quy đổi pixel:
 * `laneX(CAP + 6) === laneX(CAP - 1)`, nên so ở miền pixel **không phân biệt được**
 * hai ca — và đó chính là lỗi.
 *
 * **Không sửa `laneX`.** Sửa nó là đổi hành vi vẽ của cả Phase 2.
 */
export function wipEdge(input: WipEdgeInput): WipEdge {
  const { headLane } = input

  // Không biết HEAD ở đâu → nói là không biết. Vẽ vào lane 0 là đoán, và đoán sai
  // một cách tự tin trông giống hệt một đáp án đúng.
  if (headLane === null || headLane === undefined) return { kind: 'unknownHead' }
  if (!Number.isInteger(headLane) || headLane < 0) return { kind: 'unknownHead' }

  // 🔴 So ở miền LANE, không ở miền pixel. Xem doc comment trên.
  if (headLane >= MAX_VISIBLE_LANES) return { kind: 'clamped', realLane: headLane }

  return { kind: 'normal', lane: headLane, x: laneX(headLane) }
}
