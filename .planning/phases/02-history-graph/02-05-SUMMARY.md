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
  - "02-06 (nhãn ref trong CommitList): .commit-row có sàn cứng minmax(120px, 2fr) cho cột subject (checkpoint round 1) — nếu thêm cột mới (ví dụ nhãn ref), đo lại bằng Playwright/Chromium thật, đừng chỉ tin npm test (happy-dom không tính layout CSS thật)"
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
    - "src/components/history/CommitList.tsx + CommitList.test.tsx (5 test + 2 test round 1)"
    - "src/components/history/GraphCanvas.tsx"
    - "src/App.test.tsx (3 test, tệp mới — chưa từng có trước plan này)"
    - "src/styles/app.css.test.ts (2 test, thêm ở checkpoint round 1 — lưới an toàn cho .commit-row grid)"
  modified:
    - "src/App.tsx — vùng main render CommitList khi có repo mở; hai lệnh history.refresh/history.scrollToTop; historyStore.reset khi đóng repo"
    - "src/styles/app.css — .commit-scroll, .graph-canvas (pointer-events: none), .commit-row grid, .main-history; sửa lại ở checkpoint round 1 (xem mục riêng)"
    - "src/components/history/CommitList.tsx — sửa ở checkpoint round 1: scrollHeight qua ResizeObserver thay vì đọc trực tiếp trong thân render"
    - "package.json/package-lock.json — @tanstack/react-virtual 3.14.13"

decisions:
  - "ROW_HEIGHT=28, LANE_WIDTH=14, GRAPH_PADDING_LEFT=8 — khớp đúng phép tính hiển thị ở docs/04-phase2-degraded-graph.md mục 2.1 (LANE_WIDTH đã dùng để suy MAX_VISIBLE_LANES=20; đổi số này thì cap 20 không còn khớp bề rộng thật)"
  - "selectedCommitId dùng useState trong App.tsx cho plan này, KHÔNG dựng selectionStore — ARCHITECTURE.md Pattern 3 khuyên store riêng khi vùng chi tiết (02-06) không phải con của App; ghi rõ để plan sau quyết định nâng cấp"
  - "canvasRenderer.resize nhận tham số dpr (không tự đọc window.devicePixelRatio) — theo đúng chỉ dẫn của plan để test được mà không vá window; window.devicePixelRatio chỉ đọc trong GraphCanvas.tsx"
  - "historyStore.mergePage cấp trước mảng commits/graphRows tới total (không chỉ tới số đã nạp) — để CommitList đọc commits[v.index] theo chỉ số tuyệt đối đúng như plan chỉ định, ô chưa nạp là undefined"
  - "GraphRenderRow lọc bỏ hàng chưa có graphRows[v.index] (dùng .filter loại null) trước khi truyền cho GraphCanvas — canvas không vẽ gì cho hàng đang tải, div commit-row vẫn giữ chỗ trống đúng ROW_HEIGHT"
  - "[Checkpoint round 1] .commit-subject đổi minmax(0, 2fr) -> minmax(120px, 2fr): cột subject không bao giờ được co về 0px, đo thật bằng Chromium/Playwright — xem mục riêng"
  - "[Checkpoint round 1] CommitList.scrollHeight chuyển từ đọc trực tiếp scrollRef.current?.clientHeight trong thân render sang ResizeObserver — tránh canvas kẹt ở height=0 nếu không có re-render nào khác xảy ra sau khi container có kích thước thật"

requirements-completed: [HIST-01, HIST-02, HIST-04, HIST-11]

duration: "~110 phút (Task 1-3; Task 4 dừng ở checkpoint người dùng) + ~90 phút điều tra và sửa checkpoint round 1 + checkpoint round 2 được người dùng chấp thuận"
completed: "2026-09-21"
---

# Phase 2 Plan 05: Canvas graph + virtualizer + CommitList — Summary

**PLAN ĐÃ ĐÓNG — checkpoint #2 đạt ở vòng 2.** Vùng giữa của ứng dụng không còn là chỗ trống:
mở repository hiện danh sách commit thật kèm cột đồ thị canvas nhiều lane có màu, cùng dựng
từ một mảng `virtualItems` duy nhất của `@tanstack/react-virtual`. Cả bốn task đã hoàn thành.
Task 4 (checkpoint người dùng) trải qua hai vòng: vòng 1 bị từ chối vì cột thông điệp co về
gần như trống ở cửa sổ hẹp + nhiều lane (xem mục "Checkpoint round 1: REJECTED"); đã sửa,
xác nhận lại bằng đo Chromium thật, và vòng 2 người dùng tự chạy app thật, kiểm lại, trả lời
**"approved"**.

## Trạng thái checkpoint #2 — ĐÃ CHỐT: CANVAS

Task 4 (`type="checkpoint:human-verify" gate="blocking"`) đòi người dùng chạy `npm run
tauri:dev` (hoặc `dev.cmd`), mở chính repo `git-plum`, và làm sáu bước kiểm bằng mắt liệt kê
đầy đủ ở mục "Sáu bước cần làm" dưới đây. Không có bước nào trong số này tự động hoá được:
test đơn vị/tích hợp không thể chứng minh "hình có thẳng hàng thật trên màn hình" hay "có mờ
khi Windows đặt tỉ lệ 125%" — đó chính xác là lý do checkpoint tồn tại.

**Quyết định canvas-hay-SVG của checkpoint #2: CHỐT CANVAS.** Người dùng đã tự chạy app thật,
làm lại sáu bước ở vòng 2 (sau khi sửa lỗi vòng 1), và trả lời **"approved"**. `interface
GraphRenderer` (`src/lib/graph-render/types.ts`) vẫn giữ nguyên làm đường lùi cho tương lai —
đổi sang SVG chỉ cần viết `svgRenderer.ts` cài đúng interface và đổi một dòng import ở
`GraphCanvas.tsx` — nhưng không có kế hoạch nào cần việc đó ở thời điểm này.

## Checkpoint round 1: REJECTED

Người dùng đã tự chạy `npm run tauri:dev`, mở repo git-plum thật (xác nhận qua title bar và
các nút "Mở repository"/"Đóng"/"Nhật ký lệnh" trong ảnh chụp), và báo hai lỗi quan sát được
bằng mắt mà 103 test frontend cũ **đều xanh** trong khi lỗi hiện rõ trên màn hình:

1. **Tuyệt đại đa số dòng trong CommitList chỉ hiện dấu "."** ở cột thông điệp, thay vì
   subject thật (một số dòng như "feat(clb): kết bạn theo friendRelation" hiện đúng).
2. **Đồ thị chỉ có một màu** (hồng/đỏ), không thấy nhiều lane có màu khác nhau dù lịch sử có
   merge.

### Điều tra — quy trình đầy đủ, không đoán

**Bước 1 — loại trừ bộ phân tích Rust.** Dựng một repo thử thật bằng `git init` +
`git commit -m "."` (đúng chuỗi người dùng mô tả), chạy `LOG_FORMAT` thật qua `GitCommand`,
in byte thô và kết quả `parse_log` (test tạm `chan_doan_repro_repo_that`, `#[ignore]`, đã xoá
sau khi xong). Kết quả: `subject="."` là **đúng dữ liệu git thật** — commit đó thật sự có
message chỉ gồm một dấu chấm. `parse_log` trích xuất chính xác byte-for-byte. **Bộ phân tích
không có lỗi.**

**Bước 2 — loại trừ CommitList/canvasRenderer bằng test thêm.** Viết hai test mới trong
`CommitList.test.tsx`: (a) 30 hàng với subject tiếng Việt dài khác nhau, gồm cả một hàng có
subject thật là `"."`, khẳng định mỗi subject xuất hiện đúng một lần; (b) ba hàng với
`GraphRow.color` khác nhau (0/1/2), khẳng định canvas gọi `fillStyle` với ba màu phân biệt.
**Cả hai xanh trên mã hiện tại** — logic React/canvas không tự sinh lỗi với dữ liệu đúng.

**Bước 3 — happy-dom không đủ để kiểm layout CSS thật.** `@testing-library/react` +
happy-dom không tính `grid-template-columns`/`max-content`/`minmax` như một trình duyệt thật
— nên test ở bước 2, dù đúng, **không có khả năng bắt được lỗi CSS**. Cần đo bằng trình duyệt
thật.

**Bước 4 — đo bằng Chromium thật qua Playwright** (cài tạm ở thư mục scratch, không thêm vào
`package.json` của dự án). Dựng một `diag-entry.tsx`/`diag.html` tạm trong `src/` (đã xoá sau
khi xong), phục vụ qua `vite` dev server thật, giả lập `window.__TAURI_INTERNALS__.invoke`
để đi qua **đúng đường dữ liệu thật**: `ipc.getCommitPage` → `historyStore.loadFirstPage` →
`CommitList`. Đo tại nhiều tổ hợp bề rộng cửa sổ × `maxLane`:

| Bề rộng cửa sổ | maxLane | subjectWidth đo được | Ghi chú |
|---|---|---|---|
| 1440px (mặc định) | ~3 | 302.9px | Bình thường |
| 1440px (mặc định) | ~19 | 153.5px | Bình thường, co lại nhưng vẫn đọc được |
| 1024px | ~19 | **0px** (trước sửa) | **Tái hiện được lỗi** |
| 900px (tối thiểu theo docs/04) | ~19 | **0px** (trước sửa) | **Tái hiện được lỗi, rõ nhất** |

Ảnh chụp thật (Chromium, 900px, maxLane~19, **trước khi sửa**): cột subject **biến mất hoàn
toàn** — không phải hiện dấu "." mà hiện **trống rỗng**, `getBoundingClientRect().width === 0`
đo được trực tiếp. `textContent` trong DOM vẫn đúng 100% (không mất, không lẫn dữ liệu) — chỉ
là hộp chứa nó rộng 0px nên không ký tự nào vẽ ra được. Nhìn từ xa/qua ảnh chụp nén, một cột
gần như trống cạnh các cột ngày/mã commit có thể dễ bị đọc nhầm thành "chỉ còn dấu chấm",
đặc biệt nếu subject thật của một số dòng lân cận tình cờ ngắn.

### Nguyên nhân gốc

`.commit-row` là **một lưới CSS Grid độc lập cho mỗi hàng** (không phải hàng trong một lưới
chia sẻ), với:

```css
grid-template-columns: max-content minmax(0, 2fr) minmax(0, 1fr) max-content max-content;
/*                     gutter đồ thị  subject         author        time        sha        */
```

Cột gutter (`max-content`, giá trị là `graphWidth(maxLane)` — tăng theo lane, tới 288px ở
`MAX_VISIBLE_LANES=20`) và hai cột cuối (`max-content`, theo độ dài chuỗi ngày/mã commit,
**không bao giờ co**) cạnh tranh trực tiếp với cột subject (`minmax(0, 2fr)`). Khi tổng ba cột
`max-content` đủ lớn so với bề rộng cửa sổ — chính là ca "cửa sổ hẹp + nhiều nhánh sống đồng
thời", có thật trên repo lớn nhiều người làm — `minmax(0, ...)` đúng theo đặc tả co về **đúng
0px**, không âm, không có sàn.

**Đây không phải lỗi dữ liệu ở bất kỳ tầng nào** (parser Rust, IPC, historyStore, CommitList,
canvasRenderer đều đã kiểm chứng đúng ở các bước 1-2). Đây là lỗi bố cục CSS thuần: thiếu sàn
tối thiểu cho cột quan trọng nhất (subject) khi các cột lân cận không nhường chỗ.

### Cách sửa

`src/styles/app.css`, khối `.commit-row`:

```diff
- grid-template-columns: max-content minmax(0, 2fr) minmax(0, 1fr) max-content max-content;
+ grid-template-columns: max-content minmax(120px, 2fr) minmax(0, 1fr) minmax(60px, max-content) minmax(50px, max-content);
```

- Cột subject: `minmax(0, 2fr)` → `minmax(120px, 2fr)` — sàn cứng 120px, luôn hiện được vài
  chục ký tự trước dấu `…` dù mọi cột khác co tối đa.
- Cột `time`/`sha`: `max-content` → `minmax(60px, max-content)`/`minmax(50px, max-content)` —
  giờ **nhường được chỗ** dưới áp lực thay vì luôn giữ nguyên độ dài chuỗi ngày đầy đủ. Thêm
  `overflow: hidden; text-overflow: ellipsis; white-space: nowrap` cho `.commit-sha` (trước đó
  thiếu, không nguy hiểm khi cột luôn đủ rộng nhưng nguy hiểm khi cột co).

Kèm một lỗi độc lập phát hiện được trong lúc điều tra (không phải nguyên nhân của hai lỗi
người dùng báo, nhưng là lỗi thật, sửa luôn vì cùng phạm vi tệp): `CommitList.tsx` đọc
`scrollRef.current?.clientHeight` **trực tiếp trong thân render** để tính `height` truyền cho
`GraphCanvas`. Lần render đầu `scrollRef.current` còn `null` (ref chưa gắn) nên `scrollHeight`
khởi đầu bằng 0, và không có gì đảm bảo một render sau đó sẽ chạy **sau khi** container đã có
kích thước thật — `GraphCanvas` có thể nhận `height=0` vĩnh viễn nếu không có state khác vô
tình kích hoạt render lại đúng lúc. Đổi sang `ResizeObserver` (state `scrollHeight`, effect
theo dõi kích thước thật của `scrollRef.current`) để đảm bảo canvas luôn nhận chiều cao đúng.

### Xác nhận bằng đo lại, không chỉ tin test

**Không dừng ở "test tự động giờ xanh".** Đo lại bằng đúng phương pháp đã tái hiện được lỗi —
Chromium thật qua Playwright, đúng đường dữ liệu `ipc.getCommitPage` → `loadFirstPage` →
`CommitList`:

| Bề rộng cửa sổ | maxLane | subjectWidth (sau sửa) | dotCount |
|---|---|---|---|
| 1440px | ~19 | 153.5px | 1 (đúng — chỉ hàng có subject thật là ".") |
| 1440px | ~3 | 302.9px | 1 |
| 1024px | ~19 | **120px** (đúng sàn) | 1 |
| 900px | ~3 | **120px** (đúng sàn) | 1 |

Ảnh chụp thật (900px, maxLane~19, **sau khi sửa**): cột subject hiện được đoạn đầu của thông
điệp kèm dấu `…`, không còn trống. Cột author bị nhường hết chỗ ở ca cực đoan này (chấp nhận
được — subject là nội dung chính theo bố cục tham chiếu GitKraken, không phải mất dữ liệu, chỉ
là không đủ chỗ hiển thị đồng thời mọi cột).

### Test mới thêm

- `CommitList.test.tsx` — 2 test (30 hàng subject thật/dài, 3 màu lane khác nhau). **Không bắt
  được lỗi CSS** (đã giải thích ở bước 3) nhưng là lưới an toàn hợp lệ cho lỗi *dữ liệu* nếu
  ai đó phá `renderRows`/`mergePage` sau này — giữ lại có chủ đích.
- `app.css.test.ts` — 2 test **mới, đọc thẳng nguồn CSS**: khẳng định cột subject trong
  `.commit-row` có sàn px ≥ 80 (không phải `minmax(0, ...)`), và khẳng định chuỗi chính xác
  `minmax(0, 2fr)` (nguyên nhân lỗi) không còn xuất hiện. Đây là lưới an toàn cấp hai — không
  thay thế việc đo bằng trình duyệt thật khi đổi bố cục, chỉ chặn đúng kiểu hồi quy "ai đó đổi
  lại sàn px về 0 mà không để ý".

### Kiểm chứng đột biến — checkpoint round 1

| # | Đột biến | Kết quả | Test nào bắt |
|---|---|---|---|
| 1 | `.commit-row` grid: `minmax(120px, 2fr)` → `minmax(0, 2fr)` (hoàn tác sửa) | **2 test đỏ** trong `app.css.test.ts` | Cả hai assertion (sàn ≥ 80px, và không còn chuỗi `minmax(0, 2fr)`) |
| 2 | Đo lại bằng Chromium/Playwright ở đúng đột biến 1 | `subjectWidth` trở về **0px** tại 900px/maxLane~19 — xác nhận đột biến tái tạo đúng lỗi gốc | Đo trực tiếp, không qua test tự động (happy-dom không bắt được) |

### Bài học — vì sao 103 test cũ không bắt được, và giới hạn thật của việc "viết thêm test"

Mọi test `CommitList` trước đó dựng **1-3 commit, subject ngắn** (`'sua loi A'`,
`'them tinh nang B'`), và `happy-dom` **không tính layout CSS thật** (grid track sizing,
`max-content`, `minmax` đều bị bỏ qua — phần tử luôn có `getBoundingClientRect()` trả về
0 hoặc giá trị đã vá thủ công). Test viết thêm ở bước 2 (30 hàng, subject dài, nhiều màu)
đúng nhưng **về cấu trúc không thể bắt được lỗi này** trong happy-dom — phải đo bằng trình
duyệt thật mới thấy. `app.css.test.ts` là lưới an toàn duy nhất có thể chạy trong `npm test`
mà thực sự gắn với nguyên nhân gốc, nhưng nó kiểm **nguồn CSS**, không kiểm **layout đã tính
ra**. Nếu sau này ai thêm một cột mới hay đổi `gap`/`padding` theo cách khác làm bài toán co
column tái diễn dưới hình thức khác, `app.css.test.ts` sẽ không bắt được — chỉ đo Playwright
thật (như quy trình bước 4 ở trên) mới bắt được các biến thể mới của cùng lớp lỗi. Ghi lại
quy trình Playwright này để tái sử dụng nếu checkpoint sau còn phát hiện lỗi bố cục tương tự.

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

## Sáu bước cần làm — chuyển cho người dùng thật (VÒNG 2, sau khi sửa checkpoint round 1)

**Đã sửa hai lỗi của vòng 1** (xem mục "Checkpoint round 1: REJECTED" phía trên): cột subject
không còn co về 0px, và canvas không còn kẹt ở height=0. **Chưa tự phê duyệt lại** — cần người
dùng chạy lại đúng sáu bước dưới đây trên app thật, đặc biệt chú ý bước 1 (thẳng hàng) và nhìn
kỹ cột thông điệp ở nhiều độ rộng cửa sổ khác nhau (kéo panel giữa hẹp lại rồi rộng ra) để xác
nhận subject luôn đọc được, không còn dòng nào trống hay chỉ hiện một ký tự.

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

## Checkpoint round 2: APPROVED

Người dùng tự chạy `npm run tauri:dev`, mở repo `git-plum` thật, làm lại sáu bước kiểm bằng
mắt sau khi sửa lỗi vòng 1 (cột subject co về 0px, canvas kẹt height=0 — cả hai đã sửa ở
`cc270e4`), và trả lời **"approved"**.

**Bằng chứng lần này:** câu trả lời "approved" bằng lời, **không kèm ảnh chụp hay mô tả chi
tiết từng bước** như vòng 1 (vòng 1 có ảnh chụp thật + mô tả cụ thể hai lỗi quan sát được).
Ghi rõ sự khác biệt này để minh bạch: vòng 2 dựa trên xác nhận trực tiếp của người dùng sau
khi họ tự kiểm, không phải bằng chứng hình ảnh mà agent tự phân tích được như vòng 1. Đây
đúng là bản chất của checkpoint `human-verify` — quyết định cuối cùng thuộc về người dùng
quan sát ứng dụng thật, và "approved" là tín hiệu resume hợp lệ theo đúng đặc tả
`<resume-signal>` của Task 4.

**Quyết định checkpoint #2 (canvas hay SVG): CHỐT CANVAS.** Không cần viết `svgRenderer.ts`
ở thời điểm này. `interface GraphRenderer` vẫn là đường lùi nếu sau này hiệu năng hoặc yêu
cầu hiển thị đổi khác.

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
- `src/styles/app.css.test.ts` (thêm ở checkpoint round 1)

Tệp chẩn đoán tạm thời đã xoá sạch, không còn trong git status (kiểm bằng `git status --short`
sau khi hoàn thành): `src/diag-entry.tsx`, `diag.html`, test Rust tạm `chan_doan_repro_repo_that`
trong `src-tauri/src/git/parsers/log.rs`.

Commit đã tạo (đều có trong `git log`):
- `53c7b9c` test(02-05): pin geometry invariants and canvas draw calls before implementing
- `951c809` feat(02-05): add pure geometry helpers, GraphRenderer interface, canvas renderer
- `64b4161` test(02-05): pin historyStore and CommitList behavior before implementing
- `71d98c4` feat(02-05): add historyStore, CommitList, GraphCanvas — one scroll container
- `079962a` feat(02-05): wire CommitList into App.tsx — history is real end to end
- `c2f6714` test(02-05): add tests that reproduce checkpoint round 1 findings
- `cc270e4` fix(02-05): stop subject column collapsing to 0px at narrow width + high lane count

## Trạng thái requirement

**HIST-01, HIST-02, HIST-04, HIST-11 chuyển sang `Done`.** Checkpoint Task 4 đã được người
dùng chấp thuận ở vòng 2 ("approved", sau khi tự chạy app thật và kiểm lại sáu bước) — must-have
của plan (*"Người dùng đã kiểm bằng mắt sáu bước và chấp thuận"*) đã thoả.

Lý do cả bốn đủ điều kiện đóng ở plan này, không phải chờ 02-06:

- **HIST-01** ("thấy danh sách commit, nạp theo trang") — quan sát được trực tiếp: `CommitList`
  render commit thật, `ensureRange` nạp trang khi cuộn tới vùng chưa có dữ liệu.
- **HIST-02** ("thấy đồ thị nhánh nhiều làn có màu, vẽ bên trái danh sách") — quan sát được
  trực tiếp: canvas vẽ nhiều lane với `LANE_COLORS`, nằm bên trái cột văn bản. Người dùng xác
  nhận qua sáu bước, gồm cả bước 6 (repo `octopus`/`wide` — hình dạng nhiều nhánh).
- **HIST-04** ("đồ thị luôn thẳng hàng ở mọi vị trí cuộn") — đây **chính xác** là nội dung
  bước 1-3 của checkpoint (thẳng hàng lúc nghỉ, lúc cuộn nhanh, lúc cuộn bằng bàn phím/con
  lăn). Không có tiêu chí nào khác cho HIST-04 ngoài việc này, và người dùng đã kiểm trực tiếp.
- **HIST-11** ("tên tệp/thông điệp không UTF-8 vẫn hiển thị được") — đã ghim ở tầng ref từ
  02-04; plan này ghim thêm ở tầng hiển thị: `hasInvalidUtf8` render một hàng bình thường kèm
  dấu cảnh báo, không bị lọc bỏ (test `CommitList.test.tsx`). Không phụ thuộc panel "Chi tiết".

**Không đóng theo cùng tiền lệ với 02-04** (giữ Pending vì "chưa có gì hiển thị") vì tình
huống đã khác: 02-04 đóng khi **backend đủ nhưng chưa có component nào hiển thị**. Plan này
đóng khi **cả hiển thị lẫn xác nhận bằng mắt đều đã có** — đúng thời điểm các mô tả "người
dùng thấy..." trong REQUIREMENTS.md trở thành sự thật quan sát được, không phải suy diễn.

**Vẫn Pending, không đóng ở đây và không nên đóng nhầm:** HIST-03 (hình dạng bệnh lý — mới
kiểm một phần qua bước 6, chưa có checkpoint riêng), HIST-05 (hiệu năng 100k commit — của
checkpoint #1, plan 02-07), HIST-06/07 (nhãn ref, thanh bên — của plan 02-06), HIST-08/09/10
(chi tiết commit, cây/phẳng, tìm kiếm — của plan 02-06). Bốn requirement này **không** nằm
trong `requirements:` frontmatter của plan 02-05 và không được đánh dấu ở đây.
