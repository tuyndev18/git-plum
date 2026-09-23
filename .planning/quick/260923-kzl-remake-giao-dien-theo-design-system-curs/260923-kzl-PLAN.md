---
task: 260923-kzl-remake-giao-dien-theo-design-system-curs
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/styles/app.css
  - src/styles/app.css.test.ts
  - src/lib/graph-render/geometry.ts
  - src/lib/graph-render/canvasRenderer.ts
  - src/lib/graph-render/canvasRenderer.test.ts
autonomous: true
requirements: []
must_haves:
  truths:
    - "Theme sáng cream (Cursor) là mặc định khi mở app — không cần theme tối hệ điều hành"
    - "Theme tối ấm (đảo mực/giấy kiểu pricing-tier-featured) hiện khi hệ điều hành ở dark mode"
    - "Mọi biến --color-* dùng trong hunk/worktree/trash UI có giá trị định nghĩa ở :root, không chỉ dựa vào fallback hex cứng"
    - "Nút commit trên canvas đồ thị có lòng nút và vòng chọn nhìn thấy được trên nền cream (không còn #e6edf3 vô hình)"
    - "Không còn box-shadow đổ bóng nào ngoài các box-shadow inset dùng làm chỉ báo hairline đã có"
    - "Toàn bộ 34 describe block hiện có trong app.css.test.ts vẫn xanh sau khi đổi bảng màu/bo góc"
  artifacts:
    - path: "src/styles/app.css"
      provides: "Token :root/media đầy đủ theo CONTEXT.md + toàn bộ component rules dùng token thay vì hex cứng"
    - path: "src/lib/graph-render/geometry.ts"
      provides: "SELECTION_RING và NODE_FILL fallback phù hợp theme cream"
    - path: "src/lib/graph-render/canvasRenderer.ts"
      provides: "readSelectionRing() đọc --graph-selection-ring theo đúng pattern readNodeFill, dùng trong draw()"
  key_links:
    - from: "src/lib/graph-render/canvasRenderer.ts"
      to: "src/styles/app.css"
      via: "getComputedStyle(host).getPropertyValue('--graph-selection-ring')"
      pattern: "graph-selection-ring"
    - from: "src/styles/app.css :root"
      to: "src/styles/app.css.test.ts"
      via: "REF_COL_WIDTH / MIN_SPLIT_WIDTH pinned var reads"
      pattern: "--ref-col-width|--min-split-width"
---

<objective>
Reskin toàn bộ `src/styles/app.css` (2944 dòng, một stylesheet duy nhất) theo design system
Cursor ghi ở `skills/design/Cursor/SKILL.md`, đúng các quyết định đã khoá trong
`260923-kzl-CONTEXT.md`: theme sáng cream làm mặc định, theme tối ấm dưới media query, font
stack Inter/JetBrains Mono, bo góc theo thang 4/6/8/12/pill, bỏ đổ bóng (giữ inset hairline),
và thay mọi màu hex cứng bằng token CSS. Đây thuần là việc **nhìn** — không đổi bố cục, DOM,
logic, hay các hằng số đã ghim bằng test (`ROW_HEIGHT`, `--ref-col-width`, `--min-split-width`).

Purpose: Đưa giao diện git-plum từ theme tối tím (#16161a/#9d7cd8, dựng ở Phase 1) sang bản sắc
thị giác Cursor mà chủ dự án đã chọn, mà không phá bất kỳ bất biến bố cục nào trong số 34
describe block của `app.css.test.ts` (các block này là lưới an toàn cấp hai chặn hồi quy CSS
grid/flex từ hai vòng checkpoint thất bại của Phase 2 và Phase 3).

Output: `src/styles/app.css` viết lại phần token + toàn bộ quy tắc màu/bo góc/bóng, cộng
`SELECTION_RING`/`NODE_FILL` fallback và một hàm đọc biến CSS mới trong lớp vẽ canvas đồ thị.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@.planning/quick/260923-kzl-remake-giao-dien-theo-design-system-curs/260923-kzl-CONTEXT.md
@skills/design/Cursor/SKILL.md
@src/styles/app.css.test.ts

<interfaces>
<!-- Điểm neo hiện có mà Task 3 phải nối vào, KHÔNG đoán lại từ đầu. -->

Từ `src/lib/graph-render/geometry.ts` (KHÔNG đổi ROW_HEIGHT, LANE_WIDTH, REF_COL_WIDTH,
MAX_VISIBLE_LANES, LANE_COLORS — chỉ đổi hai hằng màu dưới đây):
```typescript
export const NODE_FILL = '#16161a'
export const NODE_FILL_VAR = '--graph-node-fill'
export const SELECTION_RING = '#e6edf3'
```

Từ `src/lib/graph-render/canvasRenderer.ts` — pattern `readNodeFill` cần lặp lại cho vòng chọn:
```typescript
function readNodeFill(host: HTMLElement): string {
  if (typeof window === 'undefined' || typeof window.getComputedStyle !== 'function') {
    return NODE_FILL
  }
  try {
    const value = window.getComputedStyle(host).getPropertyValue(NODE_FILL_VAR).trim()
    return value === '' ? NODE_FILL : value
  } catch {
    return NODE_FILL
  }
}
```
`readNodeFill(host)` được gọi một lần trong `draw()` rồi truyền xuống `drawRow`/`drawWipEdge`.
Vòng chọn vẽ ở cuối `drawRow`:
```typescript
if (row.commitId === selectedCommitId) {
  ctx.beginPath()
  ctx.arc(nodeX, yCenter, radius + 3, 0, Math.PI * 2)
  ctx.strokeStyle = SELECTION_RING
  ctx.lineWidth = 2
  ctx.stroke()
  ctx.lineWidth = 1
}
```

Test ghim hai phía đã tồn tại (đừng phá, dùng làm khuôn cho biến mới):
- `describe('REF_COL_WIDTH phải khớp giữa geometry.ts và app.css')` ở app.css.test.ts:259
- `describe('MIN_SPLIT_WIDTH phải khớp giữa DiffViewer.tsx và app.css')` ở app.css.test.ts:506
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Token :root/media — theme cream mặc định, theme tối ấm dưới media query, font, biến --color-* còn thiếu</name>
  <files>src/styles/app.css</files>
  <action>
    Viết lại khối `:root { ... }` (dòng 8-80) và khối `@media (prefers-color-scheme: light)`
    (dòng 82-108) của app.css theo đúng bảng token trong CONTEXT.md mục "Theme":

    - `:root` (mặc định — SÁNG cream, đảo ngược so với hiện tại nơi tối là mặc định):
      `--bg: #f7f7f4` (canvas), `--bg-raised: #ffffff` (surface-card), `--bg-inset: #fafaf7`
      hoặc `#e6e5e0` (canvas-soft/surface-strong — chọn giá trị nào giữ vùng inset tách biệt
      khỏi --bg theo đúng lời dặn CONTEXT.md, ghi lý do bằng comment ngắn), `--border: #e6e5e0`
      (hairline), `--border-strong: #cfcdc4` (hairline-strong), `--text: #26251e` (ink),
      `--text-dim: #5a5852` (body), `--text-faint: #807d72` (muted). Thêm `--text-faint-soft:
      #a09c92` (muted-soft) nếu cần cho disabled text — chỉ thêm nếu có chỗ dùng thật, không
      thêm biến mồ côi.
      `--accent: #f54e00` (Cursor Orange), `--accent-dim: #d04200` (active). `--danger: #cf2d56`,
      `--success: #1f8a65`. Thêm `--warn: #c08532` (vàng ấm, theo CONTEXT.md).
      `--row-highlight`: đổi từ teal (#12a0b9) sang cam/ink ấm — dùng `color-mix(in srgb,
      var(--accent) NN%, var(--ink hoặc --bg))` hoặc một hex cam trầm cố định; giữ pattern
      "lift sáng lên" hiện có KHÔNG áp dụng nữa vì nền giờ đã sáng — hàng hover/selected trên
      nền cream cần phủ TỐI đi để đọc được (ngược cực với logic dark hiện tại). Đo bằng mắt là
      việc của checkpoint sau, ở đây chọn giá trị hợp lý và ghi rõ trong comment rằng đây là
      Claude's Discretion theo CONTEXT.md.
      `--graph-node-fill: var(--bg)` giá trị tương đương `#f7f7f4` (theo comment hiện có: phải
      trùng nền hàng phía sau nút).
      Thêm `--graph-selection-ring` mới — biến CSS mà Task 3 sẽ đọc từ canvas, giá trị mặc định
      màu tối vừa đủ tương phản trên cream (ví dụ `var(--border-strong)` hoặc `var(--text-dim)`
      — không dùng `--accent` vì vòng chọn phải trung tính, không theo màu lane, đúng ý nghĩa
      hiện tại của SELECTION_RING).
      Giữ nguyên `--ref-col-width: 132px` và `--min-split-width: 720px` — KHÔNG đổi (ghim bởi
      test).
      Thêm các biến `--color-*` đang được dùng với fallback hex nhưng CHƯA khai ở `:root`
      (tìm bằng `var(--color-` trong toàn file): `--color-danger`, `--color-danger-bg`,
      `--color-bg-subtle`, `--color-row-hover`, `--color-row-selected`, `--color-selected-bg`,
      `--color-accent`, `--color-warn`, `--color-notice-bg`, `--color-border`,
      `--color-subtle-bg`, `--color-added`, `--color-removed`, `--warn`, `--ok` — trỏ mỗi biến
      về đúng token tương ứng đã định nghĩa ở trên (`--color-danger: var(--danger)`,
      `--color-accent: var(--accent)`, `--color-added: var(--success)`,
      `--color-removed: var(--danger)`, `--ok: var(--success)`, v.v. — suy luận ngữ nghĩa từ
      tên biến và nơi nó được dùng, đọc lại các dòng liệt kê trong Task 2 nếu cần ngữ cảnh).

    - `@media (prefers-color-scheme: dark) { :root:not([data-theme='light']) { ... } }`
      (đảo nhánh so với hiện tại — media query giờ là DARK, không phải LIGHT): nền
      `--bg: #1b1a16`, `--bg-raised: #23221d` (raised), `--bg-inset: #151410` (inset),
      `--border: #34322b` (hairline), `--border-strong: #46443b` (hairline strong),
      `--text: #f0efe9`, `--text-dim: #b3b0a5` (dim), `--text-faint: #807d72` (faint).
      `--accent: #ff6a1f` (Cursor Orange sáng cho nền tối). `--danger`/`--success`/`--warn` có
      thể giữ nguyên hoặc làm ấm nhẹ — Claude's Discretion, ghi comment. `--row-highlight` một
      biến thể sáng-lên phù hợp nền tối (giữ logic "lift" cũ ở đây vì nền lại tối).
      `--graph-node-fill: var(--bg)`. `--graph-selection-ring` một giá trị sáng tương phản trên
      nền tối (`var(--border-strong)` hoặc tương đương). Tất cả `--color-*` mượn lại giá trị
      dark tương ứng của `--danger`/`--accent`/`--success`/`--warn`/`--border` như ở nhánh sáng.

    - Font: `--font-ui: 'Inter', system-ui, -apple-system, 'Segoe UI', 'Helvetica Neue', Arial,
      sans-serif` (KHÔNG thêm @font-face hay import mạng — chỉ khai trong stack, đúng
      CONTEXT.md "không tải font từ mạng"). `--font-mono: 'JetBrains Mono', 'Cascadia Code',
      Consolas, monospace`.

    Giữ nguyên toàn bộ doc comment giải thích lý do kỹ thuật hiện có (vì sao
    `--graph-node-fill` đọc lúc chạy, vì sao `--row-highlight` cần alpha thấp ở
    `.commit-row` background-image, vì sao `--ref-col-width`/`--min-split-width` cố định) —
    chỉ SỬA giá trị màu, không xoá lý do kỹ thuật. Thêm một dòng comment mới ngắn gọn ở đầu
    khối `:root` ghi rõ: theme sáng cream là mặc định (D-01 CONTEXT.md), nhánh media query giờ
    là dark (đảo so với bản Phase 1 nơi dark là mặc định).
  </action>
  <verify>
    <automated>cd D:\MyCompanyProjects\git-plum && npx vitest run src/styles/app.css.test.ts</automated>
  </verify>
  <done>
    :root chứa đủ mọi biến --color-*/--warn/--ok liệt kê trong CONTEXT.md trỏ về token thật (không
    còn biến nào chỉ tồn tại dưới dạng fallback trong var()); --bg mặc định là #f7f7f4 (hoặc giá
    trị cream tương đương); media query prefers-color-scheme là 'dark' (không phải 'light'); font
    stack có Inter và JetBrains Mono; --ref-col-width và --min-split-width không đổi giá trị;
    `npx vitest run src/styles/app.css.test.ts` xanh toàn bộ 34 describe block.
  </done>
</task>

<task type="auto">
  <name>Task 2: Sweep toàn bộ component rules — hex cứng sang token, bo góc theo thang Cursor, bỏ đổ bóng</name>
  <files>src/styles/app.css</files>
  <action>
    Sau khi Task 1 xong (token đầy đủ), quét toàn bộ 2944 dòng app.css và thay:

    Hex cứng ngoài :root — thay bằng var(...) tương ứng, theo danh sách đã xác nhận bằng grep:
    - `.ref-badge--tag` (dòng ~932): `#d9a441` → dùng một biến tag mới hợp lý (ví dụ
      `color-mix(in srgb, var(--warn) 25%, var(--bg-inset))` cho nền, `var(--warn)` cho chữ) —
      hoặc giữ một hằng số tag riêng nếu --warn không phù hợp ngữ nghĩa "tag"; ghi rõ lựa chọn.
    - `.spike-harness { border-top: 2px dashed #8a6d3b }`, `.spike-results th/td { border: 1px
      solid #555 }`, `.spike-copy/.spike-word-html/.spike-stage { background: #222; border:
      1px solid #444 }` (dòng 1359-1430): đây là spike harness TẠM THỜI có comment "Xoá cùng
      lúc với SpikeHarness" — thay hex bằng var(--border)/var(--border-strong)/var(--bg-inset)
      tương ứng, KHÔNG xoá khối (ngoài phạm vi quick task này).
    - `.spike-added { background: rgba(40, 140, 60, 0.28) }`, `.spike-removed { background:
      rgba(170, 50, 50, 0.28) }` → `color-mix(in srgb, var(--success) 28%, transparent)` và
      `color-mix(in srgb, var(--danger) 28%, transparent)`.
    - `.commit-warning { border-left: 3px solid #d9a441; background: rgba(217, 164, 65, 0.12) }`
      (dòng ~2069) → `var(--warn)` và `color-mix(in srgb, var(--warn) 12%, transparent)`.
    - `.commit-hook-error` và `.commit-error` (dòng ~2089, ~2098): `#e05252` / `rgba(224, 82,
      82, 0.1)` → `var(--danger)` / `color-mix(in srgb, var(--danger) 10%, transparent)`.
    - `.commit-box { border-top: 1px solid var(--border, #333) }` (dòng 2023) → bỏ fallback
      `#333` cứng vì `--border` giờ LUÔN được định nghĩa ở :root (không còn trường hợp thiếu
      biến): `border-top: 1px solid var(--border)`.
    - `.avatar { color: #fff }` (dòng 1073, chữ trắng trên avatar màu) — GIỮ NGUYÊN, đây là màu
      cố ý theo comment (đọc được trên mọi màu avatar trong bảng `MAU_NEN`), KHÔNG phải theme
      token; không đổi.
    - Tất cả các `var(--color-*, <hex fallback>)` liệt kê ở Task 1 (25 chỗ, dòng 2382-2920):
      SAU khi Task 1 đã khai các biến này ở :root, các fallback hex trong `var(--color-danger,
      #d9534f)` trở nên chết (không bao giờ dùng tới) nhưng vô hại — GIỮ NGUYÊN cú pháp
      `var(--x, fallback)` không cần xoá fallback (an toàn phòng thủ, ngoài phạm vi bắt buộc
      của quick task này); chỉ đảm bảo biến chính đã có giá trị thật ở :root.

    Bo góc — chuẩn hoá về thang Cursor 4/6/8/12/pill theo CONTEXT.md "nút/input 8px,
    thẻ/panel/popover 12px, tag inline 4px, badge/pill 9999px, hàng gọn 6px":
    - `button { border-radius: 5px }` (dòng 181) → `8px` (nút chuẩn).
    - `.recent-repos ul { border-radius: 6px }` (dòng 328) → giữ `6px` (đúng "hàng gọn").
    - `.ref-badge { border-radius: 3px }` (dòng 905) → `4px` (tag inline).
    - `.commit-detail-body { border-radius: 6px }` (dòng 1008) → giữ `6px`.
    - `.ref-sidebar-notice { border-radius: 6px }` (dòng 1229) → giữ `6px`.
    - `.ref-sidebar-link { border-radius: 4px }` (dòng 1258) → giữ `4px` (đã đúng thang, đây là
      link nhỏ dạng tag).
    - `.ref-sidebar-head-flag { border-radius: 3px }` (dòng 1284) → `4px`.
    - `.commit-search-input { border-radius: 5px }` (dòng 1325) → `8px` (input chuẩn).
    - `.file-list-truncated { border-radius: 6px }` (dòng 1143) → giữ `6px`.
    - `.diff-notice { border-radius: 6px }` (dòng 1849) → giữ `6px`.
    - `.diff-word-changed { border-radius: 2px }` (dòng 1841) → `4px` (tag/inline nhỏ nhất hợp
      lệ trong thang) — nếu 2px quá tinh vi để nhận thấy khác biệt, chấp nhận 4px.
    - `.icon-button { border-radius: 6px }` (dòng 2291) → giữ `6px` (hàng gọn — nút icon nhỏ).
    - `.avatar { border-radius: 50% }` — GIỮ NGUYÊN (hình tròn, ngoài thang bo góc chữ nhật).
    - `.branch { border-radius: 10px }` (dòng 170), badge pill `9999px` không có trong file
      hiện tại cho `.branch` — kiểm tra ngữ nghĩa: nếu `.branch` là badge dạng pill (nội dung
      ngắn, bo tròn hết cỡ) đổi sang `9999px`; nếu không rõ, giữ nguyên vì đây là chi tiết thị
      giác nhỏ ngoài danh sách bắt buộc của CONTEXT.md — ưu tiên KHÔNG đổi nếu không chắc.
    - `::-webkit-scrollbar-thumb { border-radius: 6px }` (dòng 2198) → giữ `6px`.
    - Không đổi bất kỳ `border-radius: 0` chủ ý (`.recent-repo-item`, `.file-version-row`,
      v.v. — các nơi cố ý phẳng vì lý do bố cục, đọc comment tại chỗ trước khi đổi, mặc định
      KHÔNG đổi các giá trị `0`).

    Đổ bóng: xác nhận (đã grep) KHÔNG có `box-shadow` dạng offset/blur nào trong file — chỉ có
    hai `box-shadow: inset ...` (`.wip-row` dòng 2515, `.worktree-file-selected` dòng 2755,
    `.hunk-table-row-selected` dòng 2827) đều là chỉ báo viền 1px/3px, đúng loại CONTEXT.md nói
    GIỮ ("box-shadow inset dùng làm viền/chỉ báo, không phải elevation"). KHÔNG cần sửa gì cho
    mục đổ bóng — xác nhận lại bằng grep cuối task, không thêm box-shadow non-inset nào mới.

    Typography — nhãn section uppercase 11px/600/0.88px theo CONTEXT.md: các quy tắc
    `text-transform: uppercase` hiện có (`.pane h2`, `.recent-repos h3`,
    `.commit-header`, `.ref-sidebar-group h3`, `.change-group-heading`, `.trash-heading`) đã
    dùng `font-size: 10px` hoặc `11px` với `letter-spacing: 0.04em`-`0.06em` — chuẩn hoá
    `letter-spacing` về `0.08em` (~0.88px ở 11px, theo caption-uppercase của SKILL.md) cho các
    quy tắc đang ở 11px; giữ `.commit-header` ở 10px nguyên trạng nếu đổi kích thước ảnh hưởng
    bố cục đã ghim (không đổi font-size, chỉ letter-spacing nếu an toàn). KHÔNG đổi bất kỳ
    font-weight nào đang là 700 sang display context (chỉ nhãn nhỏ, không phải display heading
    — dự án không có headline lớn kiểu marketing).
  </action>
  <verify>
    <automated>cd D:\MyCompanyProjects\git-plum && grep -nE "#[0-9a-fA-F]{3,8}" src/styles/app.css | grep -v "^\s*\*" | grep -vE "avatar \{|color: #fff" | wc -l</automated>
  </verify>
  <done>
    Không còn hex cứng ngoài comment/doc-block và ngoại lệ `.avatar { color: #fff }` (kiểm bằng
    lệnh verify — số dòng khớp còn lại chỉ là comment hoặc avatar); mọi border-radius nằm trong
    tập {4px, 6px, 8px, 12px, 9999px, 50%, 0} theo đúng ngữ nghĩa component; không có box-shadow
    non-inset nào được thêm; `npx vitest run src/styles/app.css.test.ts` vẫn xanh toàn bộ.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: Canvas đọc --graph-selection-ring theo pattern readNodeFill, cập nhật NODE_FILL fallback, cập nhật test hồi quy CSS</name>
  <files>src/lib/graph-render/geometry.ts, src/lib/graph-render/canvasRenderer.ts, src/lib/graph-render/canvasRenderer.test.ts, src/styles/app.css.test.ts</files>
  <behavior>
    - Test 1 (mới, canvasRenderer.test.ts): với một host giả có `getComputedStyle` trả về
      `--graph-selection-ring: #123456`, `draw()` vẽ vòng chọn dùng `strokeStyle: '#123456'`
      (không còn hằng số `SELECTION_RING` cứng khi biến CSS có giá trị).
    - Test 2 (mới, canvasRenderer.test.ts): khi `getComputedStyle` trả rỗng hoặc ném lỗi (không
      có DOM/host giả không phải Element thật), `draw()` rơi về fallback `SELECTION_RING` từ
      geometry.ts — cùng hành vi phòng thủ như `readNodeFill`.
    - Test 3 (app.css.test.ts, mở rộng khối test hiện có gần `--graph-node-fill`): `:root` khai
      `--graph-selection-ring` với một giá trị không rỗng.
  </behavior>
  <action>
    Trong `src/lib/graph-render/geometry.ts`: đổi `NODE_FILL = '#16161a'` thành giá trị cream
    (`'#f7f7f4'`, khớp `--bg` mặc định mới của Task 1 — comment hiện có đã giải thích đây chỉ là
    "giá trị dự phòng cho môi trường không có DOM", KHÔNG đổi ý nghĩa, chỉ đổi hằng số theo theme
    mặc định mới). Đổi `SELECTION_RING = '#e6edf3'` thành một giá trị vẫn trung tính nhưng ĐỌC
    ĐƯỢC trên nền cream (ví dụ `'#46443b'` hoặc tương đương border-strong tối vừa đủ tương phản
    trên `#f7f7f4` — không dùng trắng/near-white vì vô hình trên cream, đúng vấn đề CONTEXT.md
    nêu). Thêm hằng `SELECTION_RING_VAR = '--graph-selection-ring'` ngay dưới `SELECTION_RING`,
    theo đúng khuôn `NODE_FILL_VAR` đã có phía trên.

    Trong `src/lib/graph-render/canvasRenderer.ts`: thêm hàm `readSelectionRing(host:
    HTMLElement): string` ngay sau `readNodeFill`, sao chép chính xác cấu trúc try/catch của
    `readNodeFill` nhưng đọc `SELECTION_RING_VAR` và fallback `SELECTION_RING`. Import
    `SELECTION_RING_VAR` từ `./geometry`. Trong `draw()`, gọi `readSelectionRing(host)` một lần
    cùng chỗ với `readNodeFill(host)` (một lần cho cả lượt vẽ, không gọi lại mỗi hàng — đúng lý
    do hiệu năng đã ghi trong comment của `readNodeFill`), truyền kết quả xuống `drawRow` như
    một tham số mới (`selectionRing: string`) thay vì `drawRow` tự đọc hằng số `SELECTION_RING`
    trực tiếp. Trong `drawRow`, đổi `ctx.strokeStyle = SELECTION_RING` (khối `if (row.commitId
    === selectedCommitId)`) thành `ctx.strokeStyle = selectionRing`. `drawWipEdge` tiếp tục dùng
    `SELECTION_RING` hằng số tĩnh cho màu nét đứt WIP (không đổi — WIP edge không phải vòng chọn,
    đọc lại comment hiện có trước khi sửa nhầm; nếu xét thấy WIP cũng cần theo theme, mở rộng
    tương tự nhưng CHỈ nếu không phá test hiện có của WIP).

    Trong `src/styles/app.css`: thêm biến `--graph-selection-ring` vào cả khối `:root` (giá trị
    sáng — dark border-strong readable trên cream) và khối dark media query (giá trị sáng hơn
    cho nền tối) — nếu Task 1 đã làm việc này thì Task 3 chỉ xác nhận, không lặp lại.

    Trong `src/styles/app.css.test.ts`: thêm một `describe` mới (theo khuôn các describe hiện có
    dùng `css.match(/--tên-biến:\s*.../)`), ví dụ ngay sau khối `REF_COL_WIDTH phải khớp...`,
    khẳng định `--graph-selection-ring` tồn tại trong `:root` với giá trị không rỗng (không cần
    so khớp với hằng số TS nào — đây là biến chỉ CSS đọc, TS chỉ đọc giá trị lúc chạy qua
    `getComputedStyle`, không có hằng số TS tương ứng để so khớp kiểu `REF_COL_WIDTH`).

    Trong `src/lib/graph-render/canvasRenderer.test.ts`: đọc file hiện có trước để nắm cấu trúc
    test host giả (`DrawingContext2D` giả, `host` giả). Thêm hai test theo `<behavior>` ở trên,
    đặt cạnh test hiện có của `readNodeFill`/vòng chọn nếu đã có, hoặc tạo `describe('vòng chọn
    đọc --graph-selection-ring')` mới cuối file.
  </action>
  <verify>
    <automated>cd D:\MyCompanyProjects\git-plum && npx vitest run src/lib/graph-render/canvasRenderer.test.ts src/lib/graph-render/geometry.test.ts src/styles/app.css.test.ts && npx tsc --noEmit</automated>
  </verify>
  <done>
    `readSelectionRing` tồn tại trong canvasRenderer.ts với cùng try/catch phòng thủ như
    `readNodeFill`; `draw()` gọi nó một lần và truyền xuống `drawRow`; `SELECTION_RING`/
    `NODE_FILL` trong geometry.ts có giá trị đọc được trên nền cream; `--graph-selection-ring`
    khai ở cả hai nhánh theme trong app.css; app.css.test.ts có cổng khẳng định biến tồn tại;
    hai test mới trong canvasRenderer.test.ts đỏ trước khi sửa (đọc hằng số cũ), xanh sau khi
    sửa; `npx vitest run` (ba file trên) và `npx tsc --noEmit` đều xanh.
  </done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
|----------|-------------|
| Không có | Thay đổi thuần CSS/hằng số màu tĩnh phía client, không xử lý input người dùng, không I/O mới, không thay đổi bề mặt IPC. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
|-----------|----------|-----------|-------------|-----------------|
| T-260923-01 | I (Information Disclosure) | Không áp dụng | accept | Không có dữ liệu nhạy cảm liên quan — đây là quick task đổi màu/bo góc tĩnh, không chạm dữ liệu repo hay AI key. |
</threat_model>

<verification>
Sau khi cả ba task hoàn tất, chạy toàn bộ cổng:

1. `npx vitest run` (toàn bộ suite frontend) — phải xanh, đặc biệt `app.css.test.ts` (34 describe
   block hiện có + describe mới của Task 3) và `canvasRenderer.test.ts`.
2. `npx tsc --noEmit` — phải xanh, không lỗi kiểu từ đổi tên/thêm hằng số trong geometry.ts.
3. Grep xác nhận không còn hex cứng ngoài comment/ngoại lệ avatar (lệnh verify của Task 2).
4. Đọc lại `:root` và khối `@media (prefers-color-scheme: dark)` — xác nhận mọi biến
   `--color-*`/`--warn`/`--ok` liệt kê trong CONTEXT.md đều có giá trị thật, không còn biến nào
   chỉ tồn tại dưới dạng fallback trong `var(--x, hex)`.
5. Không chạm `src/components/RefSidebar.tsx`, `RefSidebar.test.tsx`, `src-tauri/*`,
   `.planning/phases/*` — xác nhận bằng `git status` cuối phiên, các tệp này phải giữ nguyên
   trạng thái uncommitted đã có từ trước, không có thêm thay đổi của quick task này chồng lên.
</verification>

<success_criteria>
- `npx vitest run` toàn bộ suite xanh.
- `npx tsc --noEmit` xanh.
- Theme sáng cream (#f7f7f4 hoặc tương đương) là giá trị mặc định trong `:root`, theme tối ấm
  nằm dưới `@media (prefers-color-scheme: dark)`.
- Không còn màu hex cứng lạc trong component rules ngoài các ngoại lệ đã ghi nhận
  (`.avatar { color: #fff }`, spike harness tạm thời dùng token thay vì hex nhưng KHÔNG bị xoá).
- Canvas đồ thị đọc `--graph-selection-ring` từ CSS theo đúng pattern `readNodeFill` đã có, có
  fallback an toàn.
- Bo góc trong toàn bộ file nằm trong thang {4, 6, 8, 12, 9999}px hoặc các ngoại lệ chủ ý (50%,
  0) đã ghi trong action Task 2.
- Không có `box-shadow` đổ bóng non-inset nào được thêm mới.
</success_criteria>

<output>
After completion, create `.planning/quick/260923-kzl-remake-giao-dien-theo-design-system-curs/260923-kzl-SUMMARY.md`
</output>
