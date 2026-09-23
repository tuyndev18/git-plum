---
task: 260923-kzl-remake-giao-dien-theo-design-system-curs
plan: 01
status: complete
subsystem: ui
tags: [css, design-system, theme, canvas-render]

# Dependency graph
requires:
  - phase: 02-history-graph
    provides: canvas graph renderer (geometry.ts, canvasRenderer.ts) and app.css layout invariants (app.css.test.ts)
provides:
  - Theme sáng cream (Cursor) mặc định, theme tối ấm dưới prefers-color-scheme dark
  - Toàn bộ token --color-*/--warn/--ok có giá trị thật, không còn chỉ là fallback
  - readSelectionRing() trên canvas đồ thị, theo đúng pattern readNodeFill
affects: [ui, graph-render, diff-viewer, worktree, hunk-table]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Token CSS :root/media hai nhánh (sáng mặc định, tối dưới media query) — đảo hoàn toàn so với Phase 1"
    - "readSelectionRing() nhân bản pattern readNodeFill (đọc CSS var lúc chạy, fallback tĩnh phòng thủ try/catch)"

key-files:
  created: []
  modified:
    - src/styles/app.css
    - src/styles/app.css.test.ts
    - src/lib/graph-render/geometry.ts
    - src/lib/graph-render/canvasRenderer.ts
    - src/lib/graph-render/canvasRenderer.test.ts

key-decisions:
  - "--bg-inset dùng surface-strong (#e6e5e0) thay vì canvas-soft (#fafaf7) — canvas-soft quá sát --bg (#f7f7f4) để tách được vùng inset trên nền lớn"
  - "--row-highlight dùng color-mix(--text, --accent) thay vì hex cố định — tự nhất quán theo theme, đảo hướng alpha giữa hai theme (tối đi trên nền sáng, sáng lên trên nền tối)"
  - ".ref-badge--tag dùng --warn thay vì thêm --color-tag mồ côi — cùng hue vàng ấm, không phình bảng token cho một component"
  - ".branch { border-radius: 10px } giữ nguyên — ngữ nghĩa không rõ ràng đủ để xếp vào thang 4/6/8/12/9999, theo đúng chỉ dẫn plan 'ưu tiên KHÔNG đổi nếu không chắc'"

requirements-completed: []

duration: ~35min
completed: 2026-09-23
---

# Quick Task 260923-kzl: Remake giao diện theo design system Cursor Summary

**Reskin toàn bộ `app.css` sang theme sáng cream Cursor (Orange #f54e00, canvas #f7f7f4) làm mặc định, theme tối ấm đảo nhánh dưới media query, cộng canvas đồ thị đọc `--graph-selection-ring` theo đúng pattern `readNodeFill` đã có.**

## Performance

- **Duration:** ~35 min
- **Tasks:** 3/3 hoàn tất
- **Files modified:** 5

## Accomplishments

- Theme sáng cream (Cursor) là mặc định trong `:root`; theme tối ấm (đảo mực/giấy kiểu `pricing-tier-featured`) nằm dưới `@media (prefers-color-scheme: dark)` — đảo hoàn toàn nhánh so với bản Phase 1 (trước đó tối là mặc định, sáng nằm dưới `light`).
- Toàn bộ 14 biến `--color-*`/`--warn`/`--ok` từng chỉ tồn tại dưới dạng fallback trong `var(--color-x, #hex)` rải rác ở hơn 25 chỗ nay có giá trị THẬT ở cả hai nhánh `:root`, trỏ về đúng token ngữ nghĩa (`--color-danger: var(--danger)`, v.v.).
- Quét sạch hex cứng ngoài `:root`: spike harness tạm thời, `.ref-badge--tag`, `.commit-warning/-hook-error/-error`, `.commit-box` — tất cả chuyển sang `var(...)` hoặc `color-mix(...)`.
- Bo góc chuẩn hoá về thang Cursor {4, 6, 8, 12, 9999, 50%, 0}: `button` 5→8px, `.ref-badge`/`.ref-sidebar-head-flag` 3→4px, `.commit-search-input` 5→8px, `.diff-word-changed` 2→4px.
- Canvas đồ thị có `readSelectionRing(host)` — cùng cấu trúc try/catch phòng thủ với `readNodeFill` — đọc `--graph-selection-ring` lúc chạy, `draw()` gọi một lần cho cả lượt vẽ rồi truyền xuống `drawRow`. `SELECTION_RING` fallback đổi từ `#e6edf3` (gần-trắng, vô hình trên cream) sang `#46443b` (border-strong tối).
- `NODE_FILL` fallback đổi từ `#16161a` sang `#f7f7f4` để khớp `--bg` mặc định mới.

## Task Commits

Mỗi task được commit nguyên tử:

1. **Task 1: Token :root/media** - `43b666e` (feat) — theme cream mặc định, dark đảo nhánh, `--color-*` đầy đủ
2. **Task 2: Sweep hex/bo góc/box-shadow** - `abbf4e5` (refactor)
3. **Task 3: Canvas readSelectionRing** - hai commit TDD:
   - RED: `e569011` (test) — cổng đỏ cho `--graph-selection-ring` (canvasRenderer.test.ts + app.css.test.ts)
   - GREEN: `3cbb8d2` (feat) — implement `readSelectionRing`, cập nhật fallback

_Không có commit metadata riêng (SUMMARY.md không được commit theo constraint của quick task này)._

## Files Created/Modified

- `src/styles/app.css` — token :root/media viết lại hoàn toàn, sweep hex/bo góc/typography trong toàn bộ component rules
- `src/styles/app.css.test.ts` — thêm describe khẳng định `--graph-selection-ring` tồn tại với giá trị không rỗng
- `src/lib/graph-render/geometry.ts` — `NODE_FILL`/`SELECTION_RING` đổi giá trị fallback, thêm `SELECTION_RING_VAR`
- `src/lib/graph-render/canvasRenderer.ts` — thêm `readSelectionRing()`, `draw()`/`drawRow` truyền tham số mới
- `src/lib/graph-render/canvasRenderer.test.ts` — hai test mới (đọc CSS var thật qua DOM thật, fallback qua host giả)

## Decisions Made

- **`--bg-inset` dùng `surface-strong` (#e6e5e0), không phải `canvas-soft` (#fafaf7).** CONTEXT.md cho phép chọn giữa hai giá trị "miễn giữ vùng inset tách biệt khỏi `--bg`". `canvas-soft` (#fafaf7) chỉ cách `--bg` (#f7f7f4) vài đơn vị RGB — không tách được trên vùng lớn (ô nhập, khối mã, `.commit-detail-body`). `surface-strong` cho độ tương phản rõ ràng hơn nhiều mà vẫn trong bảng Cursor chính thức.
- **`--row-highlight` dùng `color-mix(in srgb, var(--text) NN%, var(--accent) NN%)`** thay vì một hex cố định — Claude's Discretion theo CONTEXT.md. Giữ đúng ràng buộc "lift trên nền tối / darken trên nền sáng" của bản gốc bằng cách đảo tỉ lệ `--text` giữa hai nhánh theme (55% trên sáng để tối đi, 25% trên tối để vẫn đủ sáng lên).
- **`.ref-badge--tag` dùng `--warn`** thay vì thêm biến `--color-tag` riêng — tránh phình bảng token cho đúng một component khi `--warn` (vàng ấm Cursor) đã đủ ngữ nghĩa gần với "tag".
- **`.branch { border-radius: 10px }` giữ nguyên** — plan yêu cầu "ưu tiên KHÔNG đổi nếu không chắc" khi ngữ nghĩa component (pill hay không) không rõ ràng từ CSS một mình.
- **`.trash-heading`** nằm trong danh sách "candidate letter-spacing" của Task 2 nhưng hiện không có thuộc tính `letter-spacing` nào để chuẩn hoá (chỉ `text-transform: uppercase` + `opacity`) — không thêm thuộc tính mới vì đó là thay đổi ngoài phạm vi "chuẩn hoá letter-spacing hiện có" mà Task 2 mô tả.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Sửa cách tiêm canvas giả vào host DOM thật trong test mới**
- **Found during:** Task 3, khi viết test `readSelectionRing` đọc CSS var thật
- **Issue:** Test ban đầu tiêm một object canvas giả (`{ width, height, style, getContext }`, không phải `Node` thật) vào `realHost.appendChild(canvas)` — `happy-dom` ném `TypeError` vì `appendChild` yêu cầu một `Node` DOM thật khi host là phần tử DOM thật (khác `fakeHost` — nơi cả `host` lẫn `canvas` đều là object giả nên không đụng DOM thật).
- **Fix:** Đổi sang `document.createElement('canvas')` thật, chỉ ghi đè `getContext` bằng hàm giả trả về `ctx` giả — canvas vẫn là `Node` thật để `appendChild` không ném lỗi, còn bối cảnh vẽ vẫn bị theo dõi được như các test khác trong file.
- **Files modified:** `src/lib/graph-render/canvasRenderer.test.ts`
- **Verification:** `npx vitest run src/lib/graph-render/canvasRenderer.test.ts` — 19/19 xanh
- **Committed in:** `e569011` (Task 3 RED commit — sửa trước khi tách RED/GREEN nên không có commit đỏ riêng cho lỗi cấu trúc test này)

---

**Total deviations:** 1 auto-fixed (1 bug trong chính test mới viết, không phải trong mã sản phẩm)
**Impact on plan:** Không ảnh hưởng phạm vi — lỗi nằm trong cách dựng fixture test, không phải logic `readSelectionRing`/`drawRow`. Sửa xong hai test đúng hành vi `<behavior>` yêu cầu: đọc biến CSS thật khi có, rơi về fallback khi host là object giả không phải Element thật.

## Issues Encountered

- `npx vitest run` chạy qua RTK proxy in ra `[RTK:PASSTHROUGH] vitest parser: All parsing tiers failed` (RTK không parse được output của vitest) nhưng vẫn ghi JSON report đầy đủ ở `.vitest/json/output.json` — đọc trực tiếp JSON đó để xác nhận kết quả test thay vì output text bị RTK nuốt mất. Không phải lỗi của công việc này, không sửa (ngoài phạm vi quick task, thuộc cấu hình RTK của người dùng).

## User Setup Required

None - không cần cấu hình dịch vụ ngoài nào.

## Next Phase Readiness

- `app.css` đã ở theme Cursor hoàn chỉnh, sẵn sàng cho việc kiểm bằng mắt (chưa có checkpoint nhìn nào chạy trong quick task này — đây thuần là thay đổi mã, đúng phạm vi "execute" không có checkpoint).
- Toàn bộ 34 describe block cũ của `app.css.test.ts` (giờ 35 với describe mới của Task 3) và 19 test của `canvasRenderer.test.ts` (17 cũ + 2 mới) đều xanh — không phá bất biến bố cục nào đã ghim từ hai vòng checkpoint thất bại trước của Phase 2/3.
- `npx vitest run` toàn bộ suite: **734/734 xanh**, 0 lỗi. `npx tsc --noEmit`: **0 lỗi**.
- Bốn tệp WIP của người dùng (`RefSidebar.tsx`, `RefSidebar.test.tsx`, `diff.rs`, `.planning/*`) hoàn toàn không đụng tới — `git status` xác nhận chúng vẫn ở đúng trạng thái uncommitted ban đầu.

---
*Task: 260923-kzl-remake-giao-dien-theo-design-system-curs*
*Completed: 2026-09-23*

## Self-Check: PASSED

Tất cả 5 tệp mã nguồn đã sửa và tệp SUMMARY.md này đều tồn tại trên đĩa; cả bốn
commit hash (`43b666e`, `abbf4e5`, `e569011`, `3cbb8d2`) đều tìm thấy trong
`git log --oneline --all`.
