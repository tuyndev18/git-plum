---
phase: 02-history-graph
plan: 05
subsystem: commit-list-canvas-graph
tags: [canvas, virtualizer, tanstack-react-virtual, zustand, devicePixelRatio, checkpoint]

requires:
  - phase: 02-04
    provides: "ipc.getCommitPage/GraphRow/CommitPage đã ghim ở src/lib/ipc.ts; PAGE_SIZE backend là 1000-per-page dữ liệu, không phải chỉ báo IPC gộp lô"
provides:
  - "src/lib/graph-render/{types,geometry,canvasRenderer}.ts — GraphRenderer interface, hằng số hình học, bản cài canvas"
  - "src/stores/historyStore.ts — byRepo[repoId], PAGE_SIZE=1000, ensureRange chống bão IPC"
  - "src/components/history/{CommitList,GraphCanvas}.tsx — một scroll container, một virtualizer"
  - "App.tsx nối CommitList vào vùng main — mở repo thấy lịch sử thật"
affects:
  - "02-06 (giao diện chi tiết): cần nâng selectedCommitId từ useState lên selectionStore nếu vùng chi tiết không phải con của App.tsx"
  - "02-07 (checkpoint #1 hiệu năng): PAGE_SIZE=1000, ROW_HEIGHT=28 là input cho đo FPS cuộn"

tech-stack:
  added:
    - "@tanstack/react-virtual@3.14.13 (ghim chính xác, không dùng ^)"
  patterns:
    - "Một mảng virtualItems duy nhất cấp toạ độ Y cho cả đồ thị và văn bản — GraphCanvas và cột commit-row đọc từ CÙNG v.start, không có phép tính song song"
    - "historyStore giữ mảng sparse cấp trước tới total; ô chưa nạp là undefined, đọc bằng commits[v.index] === undefined"
    - "ensureRange đánh dấu dải 'đang bay' TRƯỚC await để chống gọi trùng IPC khi cuộn nhanh"
    - "createCanvasRenderer nhận tham số thứ hai (createCanvasElement) chỉ dùng ở test — sản xuất luôn gọi với một tham số, khớp GraphRendererFactory"
    - "happy-dom đo viewport bằng offsetWidth/offsetHeight (virtual-core getRect), không phải getBoundingClientRect — test phải vá đúng hai thuộc tính đó"

key-files:
  created:
    - "src/lib/graph-render/types.ts — GraphRenderer, GraphRenderRow, GraphRendererFactory (điểm nối checkpoint #2)"
    - "src/lib/graph-render/geometry.ts — ROW_HEIGHT=28, LANE_WIDTH=14, GRAPH_PADDING_LEFT=8, MAX_VISIBLE_LANES=20, LANE_COLORS (7 màu tự thiết kế)"
    - "src/lib/graph-render/canvasRenderer.ts — createCanvasRenderer, chỉ vẽ, không bắt sự kiện"
    - "src/lib/graph-render/geometry.test.ts (11 test), canvasRenderer.test.ts (8 test)"
    - "src/stores/historyStore.ts + historyStore.test.ts (8 test)"
    - "src/components/history/CommitList.tsx + CommitList.test.tsx (5 test)"
    - "src/components/history/GraphCanvas.tsx"
    - "src/App.test.tsx (3 test, tệp mới — chưa từng có trước plan này)"
  modified:
    - "src/App.tsx — vùng main render CommitList khi có repo mở; hai lệnh history.refresh/history.scrollToTop; historyStore.reset khi đóng repo"
    - "src/styles/app.css — .commit-scroll, .graph-canvas (pointer-events: none), .commit-row grid, .main-history"
    - "package.json/package-lock.json — @tanstack/react-virtual 3.14.13"

decisions:
  - "ROW_HEIGHT=28, LANE_WIDTH=14, GRAPH_PADDING_LEFT=8 — khớp đúng phép tính hiển thị ở docs/04-phase2-degraded-graph.md mục 2.1 (LANE_WIDTH đã dùng để suy MAX_VISIBLE_LANES=20; đổi số này thì cap 20 không còn khớp bề rộng thật)"
  - "selectedCommitId dùng useState trong App.tsx cho plan này, KHÔNG dựng selectionStore — ARCHITECTURE.md Pattern 3 khuyên store riêng khi vùng chi tiết (02-06) không phải con của App; ghi rõ để plan sau quyết định nâng cấp"
  - "canvasRenderer.resize nhận tham số dpr (không tự đọc window.devicePixelRatio) — theo đúng chỉ dẫn của plan để test được mà không vá window; window.devicePixelRatio chỉ đọc trong GraphCanvas.tsx"
  - "historyStore.mergePage cấp trước mảng commits/graphRows tới total (không chỉ tới số đã nạp) — để CommitList đọc commits[v.index] theo chỉ số tuyệt đối đúng như plan chỉ định, ô chưa nạp là undefined"
  - "GraphRenderRow lọc bỏ hàng chưa có graphRows[v.index] (dùng .filter loại null) trước khi truyền cho GraphCanvas — canvas không vẽ gì cho hàng đang tải, div commit-row vẫn giữ chỗ trống đúng ROW_HEIGHT"

requirements-completed: []

duration: "~110 phút (Task 1-3; Task 4 dừng ở checkpoint người dùng)"
completed: "2026-09-21"
---

# Phase 2 Plan 05: Canvas graph + virtualizer + CommitList — Summary

Vùng giữa của ứng dụng không còn là chỗ trống: mở repository giờ hiện danh sách commit thật
kèm cột đồ thị canvas nhiều lane có màu, cùng dựng từ một mảng `virtualItems` duy nhất của
`@tanstack/react-virtual`. **Task 1-3 đã hoàn thành và tự động hoá đầy đủ. Task 4 — checkpoint
người dùng kiểm bằng mắt sáu bước (thẳng hàng lúc cuộn nhanh, độ nét HiDPI, bấm chọn qua
`div`, hình dạng bệnh lý) — CHƯA thực hiện. Đây là quyết định kiến trúc lâu dài (chốt canvas
hay phải viết lại bằng SVG) mà chỉ người dùng thật quan sát ứng dụng đang chạy mới trả lời
được; agent không có cách tương tác với ứng dụng đang chạy để tự kiểm bước này.**

## Trạng thái checkpoint #2 — CHƯA CHỐT

Task 4 (`type="checkpoint:human-verify" gate="blocking"`) đòi người dùng chạy `npm run
tauri:dev` (hoặc `dev.cmd`), mở chính repo `git-plum`, và làm sáu bước kiểm bằng mắt liệt kê
đầy đủ ở mục "Sáu bước cần làm" dưới đây. Không có bước nào trong số này tự động hoá được:
test đơn vị/tích hợp không thể chứng minh "hình có thẳng hàng thật trên màn hình" hay "có mờ
khi Windows đặt tỉ lệ 125%" — đó chính xác là lý do checkpoint tồn tại.

**Quyết định canvas-hay-SVG của checkpoint #2 vẫn để ngỏ.** `interface GraphRenderer` đã
dựng sẵn đường lùi (xem `src/lib/graph-render/types.ts`), nhưng việc chốt "canvas đạt" hay
"cần viết `svgRenderer.ts`" là của người dùng sau khi làm sáu bước.

## Đã tự động hoá xong (Task 1-3)

### Task 1 — Hình học thuần + interface bộ vẽ + bản cài canvas

- `src/lib/graph-render/geometry.ts`: `ROW_HEIGHT=28`, `LANE_WIDTH=14`,
  `GRAPH_PADDING_LEFT=8`, `NODE_RADIUS=4`, `MAX_VISIBLE_LANES=20` (khớp
  `docs/04-phase2-degraded-graph.md` mục 2.1), bảng `LANE_COLORS` bảy màu tự thiết kế.
  `rowY`/`laneX`/`graphWidth`/`colorFor` đều là hàm thuần.
- `src/lib/graph-render/types.ts`: `GraphRenderer`, `GraphRenderRow`, `GraphRendererFactory`
  — điểm nối checkpoint #2, có doc comment ghi rõ ranh giới.
- `src/lib/graph-render/canvasRenderer.ts`: `createCanvasRenderer` — chỉ vẽ, không bắt sự
  kiện, không suy lane. Nhận tham số tuỳ chọn thứ hai (`createCanvasElement`) chỉ để tiêm
  ngữ cảnh 2D giả ở test.
- 19 test (11 geometry + 8 canvasRenderer), cả 12 ca `<behavior>` của plan có test riêng.

### Task 2 — historyStore + CommitList

- `src/stores/historyStore.ts`: `byRepo[repoId]`, `PAGE_SIZE=1000`. `loadFirstPage` cấp
  trước mảng `commits`/`graphRows` tới `total`; ô chưa nạp là `undefined`. `ensureRange`
  đánh dấu dải "đang bay" **trước** `await` để chống gọi trùng IPC (T-02-17). Cả hai hàm
  nuốt lỗi vào `slice.error`, không ném ra ngoài.
- `src/components/history/CommitList.tsx`: đúng một lời gọi hook ảo hoá của
  `@tanstack/react-virtual`. Đồ thị (`GraphCanvas`) và cột văn bản đọc từ cùng
  `virtualizer.getVirtualItems()`, khớp theo `v.start`/`v.index`. Hàng chưa có dữ liệu giữ
  `ROW_HEIGHT` cố định, không co lại.
- `src/components/history/GraphCanvas.tsx`: bọc mỏng, chỉ import `GraphRenderer`/
  `createCanvasRenderer`, không gọi canvas API trực tiếp. `pointer-events: none` qua CSS.
- 13 test (8 `historyStore.test.ts` + 5 `CommitList.test.tsx`).

### Task 3 — Nối vào App.tsx

- Mở repository → `useEffect` theo `activeRepoId` gọi `loadFirstPage` → `CommitList` hiện
  lịch sử thật. Đóng repository → `historyStore.reset(id)` cùng lúc với `closeRepository`.
- Hai lệnh mới qua sổ đăng ký (PLAT-04): `history.refresh`, `history.scrollToTop`.
- Banner lỗi hiện có (`error`) mở rộng hiển thị thêm `historyStore` error của repo đang mở —
  không thêm cơ chế lỗi mới.
- `src/App.test.tsx` — **tệp mới, trước plan này chưa tồn tại.** 3 test: empty-state khi
  chưa mở repo, `CommitList` hiện văn bản commit thật khi có repo, và một test khẳng định
  chính đường nối `activeRepoId` → `ipc.getCommitPage` chạy (không chỉ là hành vi render của
  `CommitList` — xem bảng mutation dưới đây, mutation đầu tiên không bị bắt bởi hai test đầu).

## Xác minh — đầu ra thật

```
npm run typecheck                        → exit 0
npm test                                  → 103 đỗ (11 tệp)   [nền 79 (68 phase1 + ipc.history 11 phase2... thực ra nền là 68+11=79 trước plan này], thêm 24 trong plan này qua Task 1-3]
npm run build                             → thành công (dist/ 305KB JS, 6.6KB CSS)
npx tauri build --debug --no-bundle       → Built application at target\debug\git-plum.exe
```

Số đếm test chính xác theo từng bước: baseline trước plan (từ `02-04-SUMMARY.md`, mục
`npm_test`) là **68** test frontend. Plan này thêm: 11 (`geometry.test.ts`) + 8
(`canvasRenderer.test.ts`) + 8 (`historyStore.test.ts`) + 5 (`CommitList.test.tsx`) + 3
(`App.test.tsx`, tệp mới) = **35**. `68 + 35 = 103`, khớp đúng số `npm test` báo ra.

### Cổng grep của `<verification>` — kết quả thật, có hai cổng plan viết sai

```
1. npm run typecheck / npm test           → PASS (0 / 103 đỗ)
2. npm run build / tauri build --debug    → PASS
3. grep -rc 'useVirtualizer' src/components/history/  → tổng THẬT = 2, không phải 1
4. grep -rc 'dangerouslySetInnerHTML' src/            → 0  ✓ (khớp plan)
5. grep -c 'getContext' GraphCanvas.tsx               → 0  ✓ (khớp plan)
6. grep -c 'devicePixelRatio' canvasRenderer.ts       → 0, không phải ≥1
7. Checkpoint Task 4                                  → CHƯA làm, xem mục trên
```

## Plan sai ở đâu — hai cổng grep không thể xanh theo đúng nghĩa plan viết

### Cổng #3: `grep -rc 'useVirtualizer' src/components/history/` không thể bằng 1

Bất kỳ tệp nào **vừa import vừa gọi** `useVirtualizer` — tức là bất kỳ cài đặt đúng nào —
đều khớp trên **ít nhất hai dòng**: dòng `import { useVirtualizer } from '@tanstack/react-virtual'`
và dòng gọi `useVirtualizer({...})`. Đã kiểm bằng một tệp tối thiểu tuyệt đối (chỉ hai dòng,
không comment):

```tsx
import { useVirtualizer } from '@tanstack/react-virtual'
export function X() { const v = useVirtualizer({}); return v }
```

`grep -c 'useVirtualizer'` trên tệp này trả **2**, không phải 1 — nên cổng plan viết là
**không thể thoả được về mặt cấu trúc**, không liên quan gì đến chất lượng cài đặt.

**Cổng đã sửa và đã kiểm mutation:** đếm **điểm gọi thật** bằng `grep -c 'useVirtualizer('`
(tên hàm theo sát bởi dấu mở ngoặc — loại dòng `import` và mọi bình luận). Kết quả trên mã
hiện tại: tổng **1** (đúng một điểm gọi, trong `CommitList.tsx`). Kiểm mutation: thêm một
lời gọi `useVirtualizer` thứ hai (`const w = useVirtualizer({})`) vào một tệp thử → cổng đã
sửa trả **2** (đỏ đúng lúc cần đỏ). Cổng cũ của plan không phân biệt được hai trường hợp này
vì nó luôn dính ở 2 bất kể đúng sai.

### Cổng #6: `grep -c 'devicePixelRatio' canvasRenderer.ts` sai tệp

Plan tự mâu thuẫn: `<action>` của Task 1 chỉ dẫn rõ *"Đọc `dpr` từ `window.devicePixelRatio`
**ở phía gọi**, truyền vào — để test được mà không phải vá `window`"* — nghĩa là tham số của
`canvasRenderer.resize` phải tên là `dpr`, và chuỗi `window.devicePixelRatio` chỉ nên xuất
hiện ở `GraphCanvas.tsx` (phía gọi). Cài đúng theo chỉ dẫn đó thì chuỗi `devicePixelRatio`
**không** xuất hiện trong `canvasRenderer.ts` — chỉ tham số `dpr` xuất hiện.

Đo thật: `grep -c 'devicePixelRatio' src/lib/graph-render/canvasRenderer.ts` → **0**.
Chuỗi đó thật ra nằm ở ba chỗ khác: `GraphCanvas.tsx` (đọc `window.devicePixelRatio`),
`types.ts` (doc comment của `resize`), và `canvasRenderer.test.ts` (tên ca test). Hành vi
**đã được kiểm đầy đủ** — test `devicePixelRatio 2: canvas.width gấp đôi...` khẳng định
`canvas.width`/`height` nhân đôi và `scale(2,2)` gọi đúng một lần, cộng một lần kiểm mutation
thật (xem bảng dưới) — chỉ là cổng grep của plan trỏ vào tệp không mang chuỗi đó theo đúng
thiết kế mà chính plan yêu cầu.

**Không sửa lại tên tham số thành `devicePixelRatio` để né cổng grep** — làm vậy vi phạm
ngược lại chỉ dẫn "để test được mà không phải vá `window`" mà `<action>` của Task 1 đặt ra.
Ghi nhận cổng #6 là lỗi của plan, không sửa mã.

## Kiểm chứng đột biến — đã chạy thật, khôi phục nguyên trạng sau mỗi lần

| # | Đột biến | Kết quả | Test nào bắt |
|---|---|---|---|
| 1 | `rowY`: `index * ROW_HEIGHT` → `+ 1` | **4 test đỏ** (3 ca hằng số + bất biến thẳng hàng) | `geometry.test.ts` — "bất biến thẳng hàng" là ca chính |
| 2 | `graphWidth`: bỏ `Math.min(..., MAX_VISIBLE_LANES)` | **1 test đỏ** — "1422 to be 302" | `graphWidth > có chặn trên tại MAX_VISIBLE_LANES` |
| 3 | `canvasRenderer.resize`: bỏ `* dpr` khỏi `canvas.width` | **1 test đỏ** — "400 to be 800" | `devicePixelRatio 2: canvas.width gấp đôi…` |
| 4 | `historyStore.ensureRange`: bỏ điều kiện `isCovered` | **1 test đỏ** — gọi 3 lần thay vì 1 | `không gọi lại getCommitPage cho cùng dải đã nạp` |
| 5 | `CommitList`: bỏ guard `if (!row) return null` trong `renderRows` | **1 test đỏ** — `TypeError: Cannot read properties of undefined (reading 'lane')` | `total lớn hơn số commit đã nạp vẫn render, không lỗi` |
| 6 | `App.tsx`: bỏ dòng `loadFirstPage(activeRepoId)` trong effect | **Lần đầu: 0 test đỏ** (hai test cũ bơm sẵn dữ liệu vào store, không kiểm đường nối thật). **Sau khi thêm test thứ ba: 1 test đỏ** | `activeRepoId đổi thì tự gọi ipc.getCommitPage để nạp trang đầu` (test mới thêm để bắt đúng lỗ hổng này) |

**Đột biến 6 là kết quả đáng đọc nhất:** hai test `App.test.tsx` ban đầu chỉ kiểm "component
render đúng khi store đã có dữ liệu" — không kiểm đường nối `App` → `loadFirstPage` →
`ipc.getCommitPage` có thật sự chạy hay không. Gỡ dòng gọi đó, cả hai test vẫn xanh. Đã thêm
test thứ ba dùng `vi.waitFor` khẳng định `ipc.getCommitPage` được gọi sau khi `activeRepoId`
đổi — đúng loại lỗi mà cổng grep hay test render-thuần không bắt được, chỉ test hành vi thật
mới bắt.

## Sáu bước cần làm — chuyển cho người dùng thật

Chạy `npm run tauri:dev` (hoặc `dev.cmd`), mở repository của chính dự án này
(`C:\Users\tuyen\OneDrive\Desktop\git-plum`), rồi:

1. **Thẳng hàng lúc nghỉ.** Mỗi nút tròn nằm đúng giữa chiều cao hàng commit của nó không?
   Nhìn ở đỉnh danh sách, giữa, và đáy.
2. **Thẳng hàng lúc cuộn nhanh.** Kéo thanh cuộn thật nhanh lên xuống liên tục khoảng 10
   giây. Có thời điểm nào nút tròn lệch khỏi hàng, dù chỉ một hàng? Lệch dù một lần là
   **trượt**.
3. **Cuộn bằng con lăn và bằng bàn phím** (PageDown, Ctrl+End). Vẫn thẳng hàng?
4. **Độ nét.** Đường kẻ và nút tròn có nét không, hay hơi mờ/nhoè? Nếu có màn HiDPI hoặc đặt
   tỉ lệ Windows 125%/150%, kiểm ở tỉ lệ đó. Đổi tỉ lệ màn hình trong lúc ứng dụng đang mở và
   xem đồ thị có tự nét lại.
5. **Bấm chọn.** Bấm vào một hàng: hàng được chọn có sáng lên không? Bấm **đúng lên đường kẻ
   đồ thị** — vẫn chọn được hàng đó chứ? (Canvas có `pointer-events: none`; nếu bấm vào đồ
   thị không chọn được hàng thì ràng buộc số 5 bị vi phạm.)
6. **Mở một repo mẫu có hình dạng lạ.** Mở `target/fixtures/octopus` rồi `target/fixtures/wide`.
   Merge bốn cha có bốn đường rẽ không? Repo `wide` có hiện chỉ báo `+N` chứ không tràn ngang
   vô hạn?

Ghi lại: **canvas hay SVG** — quyết định của checkpoint #2. Nếu canvas đạt cả sáu bước thì
chốt canvas; interface (`src/lib/graph-render/types.ts`) vẫn ở đó làm đường lùi nếu sau này
cần đổi.

Trả lời "approved" nếu cả sáu bước đạt. Nếu không, nêu **bước số mấy** và hiện tượng thấy
được (lệch bao nhiêu hàng ở tốc độ cuộn nào / mờ ở tỉ lệ màn hình nào / bấm chỗ nào không ăn).

## Deviations from Plan

### Auto-fixed Issues

Không có sửa lỗi nào thuộc Rule 1/2/3 trong mã sản phẩm — Task 1-3 khớp đúng đặc tả của
plan. Hai điều chỉnh dưới đây là sửa **assertion của plan**, không phải sửa mã (tiền lệ wave
3 của 02-03):

**1. [Plan sai] Cổng grep `useVirtualizer` đếm cả dòng import, không thể bằng 1**

Xem mục "Plan sai ở đâu" phía trên. Cổng thay bằng đếm điểm gọi (`useVirtualizer(`), đã kiểm
mutation cả hai chiều (đúng → 1, sai → 2).

**2. [Plan sai] Cổng grep `devicePixelRatio` trỏ vào tệp không mang chuỗi đó theo đúng thiết kế của chính plan**

Xem mục "Plan sai ở đâu" phía trên. Hành vi đã kiểm đầy đủ bằng test + mutation; không sửa
tên tham số để né cổng vì làm vậy vi phạm chỉ dẫn testability của `<action>` Task 1.

### Không có stub

Không có placeholder giả vờ hoạt động. Mọi dữ liệu hiển thị (subject, tác giả, thời gian, mã
commit, đồ thị) đến từ `ipc.getCommitPage` thật, không có giá trị hardcode rỗng chảy vào UI.

## Threat Flags

Không có bề mặt bảo mật mới ngoài threat model của plan. T-02-16 (dangerouslySetInnerHTML),
T-02-17 (ensureRange dedup), T-02-18 (graphWidth cap) đều có test khẳng định trực tiếp, liệt
kê ở bảng mutation trên.

## Self-Check: PASSED

Tệp đã tạo (đều tồn tại — kiểm bằng `Read`/`ls` trong lúc thực thi):
- `src/lib/graph-render/types.ts`
- `src/lib/graph-render/geometry.ts`
- `src/lib/graph-render/canvasRenderer.ts`
- `src/lib/graph-render/geometry.test.ts`
- `src/lib/graph-render/canvasRenderer.test.ts`
- `src/stores/historyStore.ts`
- `src/stores/historyStore.test.ts`
- `src/components/history/CommitList.tsx`
- `src/components/history/CommitList.test.tsx`
- `src/components/history/GraphCanvas.tsx`
- `src/App.test.tsx`

Commit đã tạo (đều có trong `git log`):
- `53c7b9c` test(02-05): pin geometry invariants and canvas draw calls before implementing
- `951c809` feat(02-05): add pure geometry helpers, GraphRenderer interface, canvas renderer
- `64b4161` test(02-05): pin historyStore and CommitList behavior before implementing
- `71d98c4` feat(02-05): add historyStore, CommitList, GraphCanvas — one scroll container
- `079962a` feat(02-05): wire CommitList into App.tsx — history is real end to end

## Trạng thái requirement

**HIST-01, HIST-02, HIST-04, HIST-11 giữ `Pending`** cho tới khi checkpoint Task 4 được
người dùng chấp thuận — must-have của plan (*"Người dùng đã kiểm bằng mắt sáu bước và chấp
thuận"*) chưa thoả. Đừng đánh dấu complete trước khi có "approved" thật.
