---
task: 260923-lhl-reskin-giao-dien-theo-design-system-line
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
    - "Theme mặc định khi mở app là TỐI kiểu Linear (--bg #08090a), không cần theme tối hệ điều hành"
    - "Theme sáng Linear-app chỉ hiện khi hệ điều hành ở light mode, qua @media (prefers-color-scheme: light) — nhánh media giờ là 'light', đảo lại so với bản Cursor (260923-kzl) nơi media là 'dark'"
    - "Lavender (--accent #5e6ad2) chỉ xuất hiện ở nút primary, focus ring, trạng thái chọn chính, link — .brand và hover nút thường/phụ KHÔNG chuyển sang lavender"
    - "Focus ring toàn cục dùng outline 2px color-mix(--accent-dim 50%, transparent), không còn outline đặc --accent"
    - "Không còn hex cứng nào của theme Cursor (#f54e00, #ff6a1f, #d04200, #f7f7f4, #26251e, v.v.) sót lại trong app.css hoặc geometry.ts"
    - "Nút commit trên canvas đồ thị có lòng nút và vòng chọn nhìn thấy được trên nền tối Linear mới (không còn giá trị fallback cream #f7f7f4 của bản trước)"
    - "Toàn bộ describe block hiện có trong app.css.test.ts vẫn xanh sau khi đổi bảng màu/font-feature"
  artifacts:
    - path: "src/styles/app.css"
      provides: "Token :root (tối Linear mặc định) + nhánh @media (prefers-color-scheme: light) (sáng Linear-app) + font-feature-settings/tabular-nums + focus-visible toàn cục"
    - path: "src/lib/graph-render/geometry.ts"
      provides: "NODE_FILL/SELECTION_RING fallback khớp nền tối Linear mới (#08090a-family)"
    - path: "src/lib/graph-render/canvasRenderer.ts"
      provides: "readNodeFill/readSelectionRing không đổi cấu trúc, chỉ ăn theo token mới qua CSS var — xác nhận không cần sửa logic, chỉ verify"
  key_links:
    - from: "src/lib/graph-render/canvasRenderer.ts"
      to: "src/styles/app.css"
      via: "getComputedStyle(host).getPropertyValue('--graph-node-fill' | '--graph-selection-ring')"
      pattern: "graph-node-fill|graph-selection-ring"
    - from: "src/styles/app.css :root"
      to: "src/styles/app.css.test.ts"
      via: "REF_COL_WIDTH / MIN_SPLIT_WIDTH pinned var reads (không đổi)"
      pattern: "--ref-col-width|--min-split-width"
---

<objective>
Reskin toàn bộ `src/styles/app.css` (3013 dòng) từ theme Cursor (cream + cam #f54e00, dựng ở
quick task 260923-kzl) sang design system Linear (`skills/design/Linear/SKILL.md`), theo các
quyết định đã khoá của orchestrator: theme **TỐI** kiểu Linear làm mặc định (đảo lại quyết định
D-01 của 260923-kzl, nơi cream sáng là mặc định), nhánh `@media (prefers-color-scheme: light)`
giữ bảng sáng Linear-app, accent lavender `#5e6ad2` dùng tiết kiệm, bo góc theo thang Linear
(4/6/8/12/9999), không gradient/không drop-shadow, font vẫn Inter Variable + JetBrains Mono
Variable (đã bundle) nhưng bật `font-feature-settings`/`tabular-nums` để tăng độ dễ đọc.

Purpose: Đưa giao diện git-plum từ bản sắc Cursor sang bản sắc Linear mà chủ dự án chọn ở quick
task này, không phá bất kỳ bất biến bố cục nào đã ghim trong `app.css.test.ts` (grid/flex/REF_COL_WIDTH/
MIN_SPLIT_WIDTH — các cổng này kiểm cấu trúc CSS, không kiểm giá trị hex, nên không bị ảnh hưởng
bởi việc đổi token màu).

Output: `src/styles/app.css` viết lại toàn bộ token `:root`/media + mọi hex cứng còn sót của theme
Cursor trong component rules sang token Linear, cộng `NODE_FILL`/`SELECTION_RING` fallback mới
trong `geometry.ts` khớp nền tối Linear.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@skills/design/Linear/SKILL.md
@src/styles/app.css.test.ts

<interfaces>
<!-- Điểm neo hiện có mà Task 3 phải giữ nguyên cấu trúc, chỉ đổi giá trị hex fallback. -->

Từ `src/lib/graph-render/geometry.ts` (KHÔNG đổi ROW_HEIGHT, LANE_WIDTH, REF_COL_WIDTH,
MAX_VISIBLE_LANES, LANE_COLORS, NODE_RADIUS, MERGE_NODE_RADIUS, EDGE_WIDTH — chỉ đổi hai hằng
màu dưới đây, giữ nguyên tên và vai trò):
```typescript
export const NODE_FILL = '#f7f7f4'      // -> đổi sang khớp --bg tối Linear mới, ví dụ '#08090a'
export const NODE_FILL_VAR = '--graph-node-fill'
export const SELECTION_RING = '#46443b' // -> đổi sang giá trị trung tính đọc được trên nền tối mới
export const SELECTION_RING_VAR = '--graph-selection-ring'
```

Từ `src/lib/graph-render/canvasRenderer.ts` — `readNodeFill`/`readSelectionRing` đã tồn tại từ
quick task trước, ĐỌC CSS VAR LÚC CHẠY nên KHÔNG cần sửa logic, chỉ cần app.css cung cấp giá trị
mới đúng token:
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
`readSelectionRing` lặp lại đúng cấu trúc này với `SELECTION_RING_VAR`/`SELECTION_RING`. Cả hai
được gọi một lần trong `draw()` rồi truyền xuống `drawRow`.

Test ghim cấu trúc đã tồn tại (đừng phá, KHÔNG kiểm giá trị hex cụ thể):
- `describe('REF_COL_WIDTH phải khớp giữa geometry.ts và app.css')` ở app.css.test.ts:259
- `describe('--graph-selection-ring tồn tại trong :root với giá trị không rỗng')` ở app.css.test.ts:295
- `describe('MIN_SPLIT_WIDTH phải khớp giữa DiffViewer.tsx và app.css')` ở app.css.test.ts:530
- `describe('thanh cuộn')` ở app.css.test.ts:1004 — đã kiểm ngón/rãnh cuộn dùng CSS var, KHÔNG hex
  cứng — không cần sửa, chỉ xác nhận vẫn xanh sau khi đổi token.

Vị trí hex cứng theo theme Cursor cần thay (đã grep xác nhận, chỉ ở hai tệp này):
- `src/styles/app.css` dòng 16-37 (`:root`) và 130-146 (khối
  `@media (prefers-color-scheme: dark) { :root:not([data-theme='light']) {...} }` — khối này sẽ
  ĐỔI TÊN media condition thành `light` và `:not([data-theme='dark'])`, xem Task 1).
- `src/lib/graph-render/geometry.ts` dòng 95 (`NODE_FILL`) và dòng 110 (`SELECTION_RING`).
- `src/components/Logo.tsx` dùng `var(--accent)`/`var(--accent-dim)`/`var(--success)` trực tiếp
  trong SVG — KHÔNG có hex cứng, tự động theo theme mới, KHÔNG cần sửa file này.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Token :root/media — tối Linear mặc định, sáng Linear-app dưới prefers-color-scheme light, font-feature-settings, focus ring toàn cục</name>
  <files>src/styles/app.css</files>
  <action>
    Viết lại khối `:root { ... }` (dòng 14-124) và khối media query tối hiện tại (dòng 126-175)
    của app.css theo bảng token Linear đã khoá bởi orchestrator:

    **`:root` (mặc định — TỐI Linear, đảo lại so với hiện tại nơi cream sáng là mặc định):**
    - `--bg: #08090a` (gần canvas Linear #010102 nhưng dễ tách lớp trong app dày đặc — guideline
      cấm #000 thuần).
    - `--bg-raised: #0f1011` (surface-1).
    - `--bg-inset`: chọn `#050506` hoặc `#141516` tuỳ vai trò — đọc lại cách `--bg-inset` được
      dùng trong toàn file (ô nhập, khối mã nguồn, `.sidebar`/`.detail` phải lùi MỘT BẬC so với
      `--bg`, không phải sâu hơn hai bậc). Ghi lý do chọn bằng comment ngắn, theo đúng tiền lệ
      comment `--bg-inset` hiện có (dòng 18-22) giải thích lựa chọn surface-strong so với
      canvas-soft.
    - Thêm `--surface-2: #141516` và `--surface-3: #191a1b` nếu cần cho hover/menu — CHỈ thêm nếu
      có chỗ dùng thật sau khi rà lại component rules ở Task 2 (không thêm biến mồ côi).
    - `--border: #23252a` (hairline), `--border-strong: #34343a` (hairline-strong).
    - `--text: #f7f8f8` (ink), `--text-dim: #d0d6e0`, `--text-faint: #8a8f98` — kiểm contrast
      ≥4.5:1 của cả hai trên `--bg-raised` và `--bg-inset` đã chọn ở trên (tính nhẩm hoặc dùng
      công thức luminance; nếu `--text-faint` không đạt trên nền inset đã chọn, hạ sáng
      `--bg-inset` xuống phương án còn lại, không hạ chuẩn contrast).
    - `--accent: #5e6ad2`, `--accent-dim: #5e69d1` (focus/press), thêm `--accent-hover: #828fff`.
    - `--success: #27a644`; `--danger`/`--warn` chọn bản trầm hợp nền tối (ví dụ `--danger:
      #eb5757`, `--warn: #f2c94c`) — CHỈ dùng ngữ nghĩa (lỗi/cảnh báo), không dùng làm accent
      thay lavender ở đâu khác.
    - `--row-highlight`: đổi từ cam/ink (`color-mix(var(--text), var(--accent))`) sang một
      `color-mix` trung tính hợp nền tối Linear — KHÔNG dùng lavender ở đây (D-02 khoá: lavender
      chỉ ở nút primary/focus/chọn chính/link, không phải hover hàng thường). Dùng
      `color-mix(in srgb, var(--text) NN%, transparent)` thuần, bỏ vế `var(--accent)`. Với
      `.commit-row.selected` (trạng thái CHỌN CHÍNH — được phép lavender theo D-02), cân nhắc
      giữ một biến `--row-selected-tint` riêng dùng `var(--accent)` nếu cần phân biệt hover vs
      selected rõ hơn — Claude's Discretion, ghi rõ trong comment.
    - `--graph-node-fill: var(--bg)` (giữ nguyên pattern, giá trị theo `--bg` tối mới).
    - `--graph-selection-ring`: giữ trung tính (không lavender — vòng chọn hàng trên canvas không
      phải "trạng thái chọn chính" theo nghĩa UI chọn — giữ đúng ý nghĩa hiện tại của
      `SELECTION_RING`), dùng `var(--border-strong)` hoặc tương đương sáng vừa đủ tương phản trên
      nền tối mới.
    - Giữ nguyên `--ref-col-width: 132px` và `--min-split-width: 720px` — KHÔNG đổi (ghim bởi
      test).
    - Bo góc: KHÔNG cần khai token CSS mới cho border-radius (file dùng giá trị px trực tiếp) —
      Task 2 sẽ chuẩn hoá theo thang Linear {4, 6, 8, 12, 9999}.
    - Font: giữ nguyên `--font-ui`/`--font-mono` hiện có (Inter Variable + JetBrains Mono
      Variable, đã bundle qua `@fontsource-variable/*`, KHÔNG đổi vì đã đúng quyết định khoá).
      Thêm vào rule `body` (dòng ~188-196): `font-feature-settings: 'cv11', 'ss01', 'ss03';` và
      giữ `letter-spacing` ở khoảng `-0.05px` đến `0` (kiểm rule `body` hiện tại chưa có
      `letter-spacing` — thêm `letter-spacing: -0.05px`). Với các phần tử hiển thị SHA/số/ngày
      (`.commit-time`, `.exit`, `.duration`, `dd` trong `.repo-summary`, mọi nơi dùng
      `var(--font-mono)` cho số) — rà bằng Task 2, thêm `font-variant-numeric: tabular-nums`.
      Với heading/uppercase labels hiện có (`.pane h2`, `.recent-repos h3`, v.v. — cùng danh sách
      Task 2 của quick task trước) đảm bảo `font-weight: 600` và giữ `letter-spacing` âm nhẹ nếu
      ngữ cảnh là tiêu đề (không áp dụng cho uppercase labels nhỏ vốn đã dùng letter-spacing
      dương — đó là nhãn phân loại kiểu eyebrow, giữ nguyên).

    **`@media (prefers-color-scheme: light) { :root:not([data-theme='dark']) { ... } }`**
    (đổi TÊN media condition từ `dark`/`:not([data-theme='light'])` — đảo ngược lại D-01 của
    260923-kzl, ghi rõ trong comment đầu file: "theme tối là mặc định theo Linear guideline
    'Don't ship a light-mode'; nhánh sáng Linear-app chỉ hiện qua media query, đảo lại quyết định
    D-01 của quick task 260923-kzl"):
    - `--bg: #fcfcfd`, `--bg-raised: #ffffff` hoặc tương đương, `--bg-inset: #f4f5f8`.
    - `--border: #e1e3e8` (hairline sáng Linear-app), `--border-strong` một bậc đậm hơn hợp lý
      (Claude's Discretion, ghi comment).
    - `--text: #1b1c1f` (ink), `--text-dim: #3c4149`, `--text-faint: #6b6f76`.
    - `--accent: #5e6ad2` (giữ nguyên — cùng lavender, không đổi theo theme).
    - `--danger`/`--warn`/`--success` giữ nguyên giá trị hoặc làm đậm nhẹ cho nền sáng — Claude's
      Discretion, ghi comment ngắn nếu đổi.
    - `--graph-node-fill: var(--bg)`, `--graph-selection-ring` một giá trị tối vừa đủ tương phản
      trên nền sáng (khác giá trị nhánh tối).
    - Toàn bộ `--color-*`/`--warn`/`--ok` (14 biến đã có từ quick task trước) mượn lại theo đúng
      token tương ứng của nhánh sáng, cùng cấu trúc như nhánh `:root` mặc định.

    Giữ nguyên toàn bộ doc comment giải thích lý do kỹ thuật hiện có (canvas đọc biến lúc chạy,
    `--ref-col-width`/`--min-split-width` cố định, v.v.) — chỉ SỬA giá trị màu và tên nhánh media,
    không xoá lý do kỹ thuật.

    **Focus ring toàn cục (D-05):** thêm một rule mới `:focus-visible { outline: 2px solid
    color-mix(in srgb, var(--accent-dim) 50%, transparent); outline-offset: 1px; }` — đặt gần khối
    comment "Viền focus PHẢI giữ" hiện có (gần `.icon-button:focus-visible` dòng ~2404). Sau khi
    thêm rule toàn cục, xét bỏ `.icon-button:focus-visible` cụ thể nếu rule toàn cục đã đủ (so
    sánh: rule cũ dùng `var(--accent)` đặc, rule mới dùng `color-mix` 50% — nếu icon-button cần
    outline đặc hơn vì icon nhỏ khó thấy, giữ rule riêng đè lên rule toàn cục, ghi rõ lý do trong
    comment).
  </action>
  <verify>
    <automated>cd D:\MyCompanyProjects\git-plum && npx vitest run src/styles/app.css.test.ts</automated>
  </verify>
  <done>
    `:root` có `--bg: #08090a` (hoặc giá trị tối Linear tương đương) làm mặc định; media query là
    `prefers-color-scheme: light` (không phải `dark`); `--accent: #5e6ad2` ở cả hai nhánh;
    `--ref-col-width`/`--min-split-width` không đổi giá trị; `body` có `font-feature-settings`
    chứa `cv11`; có rule `:focus-visible` toàn cục dùng `color-mix(... var(--accent-dim) 50%,
    ...)`; `npx vitest run src/styles/app.css.test.ts` xanh toàn bộ describe block hiện có.
  </done>
</task>

<task type="auto">
  <name>Task 2: Sweep component rules — hex Cursor còn sót sang token Linear, bo góc theo thang Linear, .brand/hover giữ lavender tiết kiệm</name>
  <files>src/styles/app.css</files>
  <action>
    Sau khi Task 1 xong (token Linear đầy đủ), quét toàn bộ app.css tìm hex cứng CÒN SÓT của theme
    Cursor mà Task 1 chưa chạm (component rules ngoài `:root`/media, ví dụ tham chiếu trong doc
    comment ở dòng 58, 70, 785, 995, 2205-2207 nói về giá trị cũ `#161b22`/`#1b2b32`/`#1c1e23`/
    `#16161a`/`#c1c1c1`/`#f0f0f0` — đây là DOC COMMENT mô tả lịch sử/tham chiếu ảnh chụp màn hình,
    KHÔNG phải giá trị CSS đang dùng; giữ nguyên các comment lịch sử này, KHÔNG xoá, vì chúng giải
    thích lý do kỹ thuật của các quyết định trước — chỉ cần xác nhận không có rule CSS thực nào
    còn tham chiếu hex Cursor).

    Xác nhận và xử lý các vị trí sau (đã grep xác nhận là toàn bộ hex ngoài `:root`/media hiện có
    trong file, tất cả đều đã ở dạng `var(--color-x, #hex-fallback)` — KHÔNG phải hex trần):
    - Dòng 2452, 2683, 2696, 2751, 2968: `var(--color-danger, #d9534f)` — fallback hex CHẾT (biến
      chính luôn có giá trị thật từ Task 1), giữ nguyên cú pháp `var(--x, fallback)`, không cần
      xoá fallback.
    - Dòng 2616: `var(--warn, #d29922)`, dòng 2620: `var(--ok, #3fb950)` — tương tự, giữ nguyên.
    - Dòng 2825, 2897: `box-shadow: inset 3px 0 0 var(--color-accent, #5a8dee)` — đây là chỉ báo
      viền hairline (inset), KHÔNG phải elevation — giữ nguyên theo đúng nguyên tắc Linear "no
      drop shadow trên nền tối, trừ khi dùng hairline thay shadow"; đây chính là trường hợp đó.
    - Dòng 2839, 2873: `var(--color-warn, #d9a05b)` — giữ nguyên.
    - Dòng 2939, 2943: `var(--color-added, #4a9c5a)` / `var(--color-removed, #c65a5a)` — giữ
      nguyên.
    - Dòng 2479: `var(--color-bg-subtle, #1c1c1c)` — giữ nguyên.

    KHÔNG có hex TRẦN (không qua `var()`) nào của theme Cursor sót lại ngoài `:root`/media sau khi
    grep xác nhận — nếu quét thấy phát sinh thêm (ví dụ trong `.spike-*` harness tạm thời, `.ref-
    badge--tag`, `.commit-warning` mà quick task 260923-kzl đã sweep sang token ở lần trước),
    XÁC NHẬN chúng đã dùng `var(--warn)`/`var(--danger)`/`color-mix(...)` (không phải hex trần) —
    không cần sửa gì thêm, các token này tự động ăn theo giá trị Linear mới của Task 1.

    **`.brand` (dòng ~224-228) và button hover (dòng ~254-257) — D-02 khoá, lavender tiết kiệm:**
    - `.brand { color: var(--accent) }` → đổi sang `var(--text)` (hoặc giữ trung tính khác) — KHÔNG
      dùng lavender đậm cho brand mark theo chữ trong text, vì D-02 nói "`.brand` không đổi sang
      lavender đậm". Đọc lại ngữ cảnh dùng thực tế của `.brand` trước khi đổi (nếu `.brand` là
      logo/wordmark hiển thị bên cạnh `<Logo/>` component đã tự dùng `var(--accent)` trong SVG,
      giữ nhất quán bằng cách để `.brand` text dùng `var(--text)` weight 600, không cần trùng màu
      với icon).
    - `button:hover:not(:disabled) { border-color: var(--accent-dim); color: var(--accent) }`
      (dòng 254-257) → đổi thành nâng surface + border-strong theo D-02 ("hover nút thường không
      đổi sang lavender đậm — hover nút phụ = nâng surface + border-strong, không đổi màu chữ
      sang accent"): `background: var(--bg-raised)` (hoặc `--surface-2` nếu thêm ở Task 1),
      `border-color: var(--border-strong)`, bỏ `color: var(--accent)` khỏi hover thường. Nút
      PRIMARY (nếu có class riêng như `.button-primary`/tương đương — kiểm tra file có class nào
      đánh dấu nút chính không, nếu không có class riêng thì toàn bộ `button` hiện tại đều là
      "nút thường/phụ" theo nghĩa D-02, giữ hover trung tính cho tất cả).

    **Bo góc — chuẩn hoá về thang Linear {4, 6, 8, 12, 9999}px** (thang này trùng gần hết thang
    Cursor đã áp dụng ở quick task trước, chỉ khác ở chỗ Linear dùng 12px cho card/panel thay vì
    dải rộng hơn) — rà lại toàn bộ `border-radius` hiện có trong file (grep `border-radius:`):
    hầu hết đã ở {4, 6, 8, 50%, 0} từ lần sweep Cursor trước, KHÔNG cần đổi lại các giá trị này vì
    chúng đã khớp thang Linear. Riêng kiểm các panel/card lớn (nếu có `border-radius: 12px` hoặc
    lớn hơn ở đâu đó cho khối panel/card — grep xác nhận trước khi sửa) khớp đúng 12px theo Linear
    `{rounded.lg}`. `.branch { border-radius: 10px }` (dòng 237) — giữ nguyên theo đúng tiền lệ
    "ưu tiên KHÔNG đổi nếu không chắc" của quick task trước (ngoài phạm vi bắt buộc).

    Với các phần tử hiển thị SHA/số/ngày còn thiếu `font-variant-numeric: tabular-nums` sau rà soát
    Task 1 (ví dụ `.commit-time`, `.exit`, `.duration`, `dd` trong `.repo-summary` nếu hiện số) —
    thêm thuộc tính này. Xác nhận KHÔNG có gradient (`linear-gradient`/`radial-gradient`) và KHÔNG
    có `box-shadow` non-inset nào trong file (grep xác nhận cuối task) — Linear cấm cả hai trên
    nền tối.
  </action>
  <verify>
    <automated>cd D:\MyCompanyProjects\git-plum && grep -nE "#(f54e00|ff6a1f|d04200|f7f7f4|26251e|e6e5e0|cfcdc4|4a4843|65625a|1b1a16|23221d|151410|34322b|46443b|f0efe9|c4c1b6|9d9a8f|c08532)" src/styles/app.css | grep -v '^\s*\*' | wc -l</automated>
  </verify>
  <done>
    Lệnh verify trả về `0` (không còn hex cứng nào của theme Cursor cũ ngoài comment lịch sử đã
    lọc bằng `grep -v '^\s*\*'`); `.brand` không dùng `var(--accent)` cho màu chữ; hover của
    `button` mặc định không còn `color: var(--accent)`; mọi `border-radius` trong file nằm trong
    tập {4px, 6px, 8px, 12px, 9999px, 50%, 0} hoặc ngoại lệ `.branch` đã ghi nhận; không có
    `box-shadow` non-inset hoặc gradient nào; `npx vitest run src/styles/app.css.test.ts` vẫn
    xanh toàn bộ.
  </done>
</task>

<task type="auto" tdd="true">
  <name>Task 3: geometry.ts NODE_FILL/SELECTION_RING fallback khớp nền tối Linear, mở rộng test hồi quy CSS/canvas</name>
  <files>src/lib/graph-render/geometry.ts, src/lib/graph-render/canvasRenderer.test.ts, src/styles/app.css.test.ts</files>
  <behavior>
    - Test 1 (mở rộng, app.css.test.ts): `:root` khai `--bg` với giá trị tối (không phải cream
      `#f7f7f4`) — assert giá trị hex khớp pattern `#0[0-9a-f]{5}` hoặc kiểm cụ thể bằng regex đọc
      `--bg:\s*(#[0-9a-fA-F]{6})` rồi assert giá trị đó KHÁC `#f7f7f4` (giá trị Cursor cũ) — chống
      hồi quy "quên đảo theme mặc định".
    - Test 2 (mở rộng, app.css.test.ts): media query trong file phải là
      `prefers-color-scheme: light` — assert `css` KHÔNG còn chứa
      `@media (prefers-color-scheme: dark)` (chuỗi cũ của bản Cursor) và CÓ chứa
      `@media (prefers-color-scheme: light)`.
    - Test 3 (mở rộng, canvasRenderer.test.ts nếu chưa có sẵn cổng tương đương): xác nhận
      `readNodeFill`/`readSelectionRing` vẫn hoạt động đúng cấu trúc try/catch hiện có khi
      `getComputedStyle` trả về giá trị Linear mới (ví dụ `#08090a`) — test hồi quy đơn giản
      không cần viết lại logic, chỉ xác nhận giá trị mới truyền qua đúng, không bị hard-code đè
      bởi fallback cũ.
  </behavior>
  <action>
    Trong `src/lib/graph-render/geometry.ts`: đổi `NODE_FILL = '#f7f7f4'` (giá trị cream của bản
    Cursor) thành giá trị khớp `--bg` tối Linear mặc định mới của Task 1 (ví dụ `'#08090a'`) — giữ
    nguyên toàn bộ doc comment giải thích ý nghĩa "giá trị dự phòng cho môi trường không có DOM,
    phải trùng nền hàng phía sau nút", chỉ cập nhật câu cuối cùng nói về "đổi sang cream cùng lúc
    với reskin Cursor" thành "đổi sang tối Linear cùng lúc với reskin Linear (260923-lhl)". Đổi
    `SELECTION_RING = '#46443b'` (giá trị border-strong tối của bản Cursor, vốn đã được chọn để
    đọc được trên nền cream) thành một giá trị khớp `--border-strong` MỚI của theme tối Linear
    (ví dụ giữ nguyên `'#34343a'` nếu đó là giá trị `--border-strong` mới từ Task 1, hoặc điều
    chỉnh sáng hơn một chút nếu `#34343a` không đủ tương phản trên `#08090a` — kiểm bằng mắt số:
    `#34343a` trên `#08090a` là tương phản thấp vì cả hai đều tối; cân nhắc dùng giá trị sáng hơn
    như `#5a5c66` hoặc `var(--text-faint)`-tương đương để vòng chọn thực sự nhìn thấy được trên
    nền tối — đây là mục đích chính của must_have "vòng chọn nhìn thấy được trên nền tối Linear
    mới", không phải chỉ đổi số cho khớp tên biến).

    Trong `src/styles/app.css.test.ts`: thêm hai `describe` mới theo Test 1/Test 2 ở `<behavior>`,
    đặt gần khối `describe('--graph-selection-ring tồn tại trong :root...')` hiện có (dòng ~295).
    Dùng đúng khuôn `css.match(/.../)` như các describe khác trong file.

    Trong `src/lib/graph-render/canvasRenderer.test.ts`: đọc file hiện có trước để xác nhận cấu
    trúc test host giả đã kiểm `readNodeFill`/`readSelectionRing` với giá trị CSS var tuỳ ý (test
    này đã tồn tại từ quick task 260923-kzl, dùng giá trị ví dụ như `#123456` không phụ thuộc theme
    cụ thể) — nếu test hiện có đã đủ tổng quát (không hard-code giá trị cream/Cursor), KHÔNG cần
    sửa gì, chỉ xác nhận vẫn xanh. Nếu phát hiện test nào hard-code giá trị hex cũ của theme Cursor
    làm expected value, cập nhật theo giá trị mới từ Task 1/geometry.ts.
  </action>
  <verify>
    <automated>cd D:\MyCompanyProjects\git-plum && npx vitest run src/lib/graph-render/canvasRenderer.test.ts src/lib/graph-render/geometry.test.ts src/styles/app.css.test.ts && npx tsc --noEmit</automated>
  </verify>
  <done>
    `NODE_FILL`/`SELECTION_RING` trong geometry.ts có giá trị khớp theme tối Linear mặc định mới,
    không còn giá trị cream `#f7f7f4` hay `#46443b` cũ của bản Cursor; `SELECTION_RING` đọc được
    (đủ tương phản) trên nền `--bg` tối mới; app.css.test.ts có hai cổng mới khẳng định `--bg` tối
    và media query là `light`; `npx vitest run` (ba file trên) và `npx tsc --noEmit` đều xanh.
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
| T-260923-02 | I (Information Disclosure) | Không áp dụng | accept | Không có dữ liệu nhạy cảm liên quan — đây là quick task đổi màu/bo góc/font-feature tĩnh, không chạm dữ liệu repo hay AI key. |
</threat_model>

<verification>
Sau khi cả ba task hoàn tất, chạy toàn bộ cổng:

1. `npx vitest run` (toàn bộ suite frontend) — phải xanh, đặc biệt `app.css.test.ts` (describe
   block hiện có + hai describe mới của Task 3) và `canvasRenderer.test.ts`.
2. `npx tsc --noEmit` — phải xanh, không lỗi kiểu từ đổi hằng số trong geometry.ts.
3. Grep xác nhận không còn hex cứng theo theme Cursor ngoài comment lịch sử (lệnh verify của
   Task 2, phải trả về `0`).
4. Đọc lại `:root` và khối `@media (prefers-color-scheme: light)` — xác nhận `--bg` mặc định là
   tối Linear (`#08090a`-family), `--accent` là `#5e6ad2` ở cả hai nhánh, mọi biến `--color-*`/
   `--warn`/`--ok` liệt kê trong CONTEXT.md đều có giá trị thật.
5. Xác nhận `.brand` và hover `button` mặc định không dùng lavender đậm (D-02) — đọc lại rule sau
   khi Task 2 sửa.
6. Không chạm `src-tauri/*`, `.planning/phases/*`, hay bất kỳ tệp WIP nào đã có từ trước (kiểm bằng
   `git status` cuối phiên — các tệp đó phải giữ nguyên trạng thái uncommitted đã có).
</verification>

<success_criteria>
- `npx vitest run` toàn bộ suite xanh.
- `npx tsc --noEmit` xanh.
- Theme TỐI Linear (`--bg: #08090a` hoặc tương đương) là giá trị mặc định trong `:root`; theme
  sáng Linear-app nằm dưới `@media (prefers-color-scheme: light)`.
- `--accent: #5e6ad2` chỉ xuất hiện ở nút primary/focus ring/trạng thái chọn chính/link — `.brand`
  và hover nút thường/phụ dùng token trung tính (nâng surface + border-strong).
- Focus ring toàn cục dùng `outline: 2px solid color-mix(in srgb, var(--accent-dim) 50%,
  transparent)`.
- Không còn màu hex cứng nào của theme Cursor (`#f54e00`, `#ff6a1f`, `#d04200`, `#f7f7f4`,
  `#26251e`, v.v.) sót lại trong `app.css` hoặc `geometry.ts`.
- Bo góc trong toàn bộ file nằm trong thang {4, 6, 8, 12, 9999}px hoặc ngoại lệ chủ ý (`50%`, `0`,
  `.branch` 10px) đã ghi trong action Task 2.
- Không có `box-shadow` đổ bóng non-inset hoặc gradient nào.
- Canvas đồ thị (`NODE_FILL`/`SELECTION_RING` trong geometry.ts, đọc qua `readNodeFill`/
  `readSelectionRing`) hiển thị được trên nền tối Linear mới.
</success_criteria>

<output>
After completion, create `.planning/quick/260923-lhl-reskin-giao-dien-theo-design-system-line/260923-lhl-SUMMARY.md`
</output>
