---
phase: 03-diff-viewer
plan: 04
subsystem: diff-viewer
tags: [frontend, codemirror, diff-01, diff-02, diff-03, diff-04, diff-06, layout]
requires:
  - "src/lib/ipc.ts getFileDiff + FileDiff/DiffKind/Hunk/DiffLine/Span (03-02, 03-03)"
  - "src/lib/commands.ts registerCommands/runCommand/unregisterCommand (PLAT-04)"
  - "src/stores/selectionStore.ts (khuôn byRepo), src/stores/uiStore.ts (khuôn không-theo-repo)"
  - "src/lib/perf.ts measureFirstPaint (nhãn tĩnh, T-02-24)"
  - "target/fixtures/diff-cases (03-02) cho bước 10 của checkpoint"
provides:
  - "src/stores/diffStore.ts — selectedFileByRepo + viewMode + showWhitespace"
  - "src/lib/diff-render/langLoader.ts — loadLanguage(path), bảng tra import() động"
  - "src/lib/diff-render/decorations.ts — hunksToDoc/buildDecorations/spanToUtf16/nextHunkLine"
  - "src/lib/diff-render/types.ts — interface DiffRenderer (tiền lệ GraphRenderer)"
  - "src/lib/diff-render/codemirrorRenderer.ts — đường A, tệp DUY NHẤT nhập @codemirror/merge"
  - "src/components/diff/DiffViewer.tsx + DiffToolbar.tsx (bốn lệnh + năm thông báo DIFF-06)"
  - "scripts/check-lang-chunks.mjs — cổng bundle, thay cổng tên-chunk mà Vite 8 làm bất khả"
affects:
  - "03-05 (exit gate) — nếu checkpoint 11 bước bị hoãn, 03-05 là chỗ đầu tiên phát hiện lỗi hiển thị"
  - "Phase 5 (staging theo khối) — dùng lại decorations.ts; MergeView đang tắt revertControls"
tech-stack:
  added:
    - "@codemirror/language 6.12.4, @codemirror/commands 6.11.1"
    - "@codemirror/lang-javascript 6.2.5, -rust 6.0.2, -json 6.0.2, -css 6.3.1, -markdown 6.5.2, -html 6.4.12"
  patterns:
    - "Nhãn hiển thị đọc từ DỮ LIỆU đã về, không từ TRẠNG THÁI store — nhãn đọc store CHE mất lỗi đua"
    - "Cổng bundle không neo vào TÊN chunk (bundler đặt) mà vào NỘI DUNG chunk entry"
    - "Khoá object literal sống qua rút gọn; tên định danh và tên nhập thì không"
    - "await Promise.resolve() KHÔNG flush render của React 19 — phải act()"
decisions:
  - "Bố cục: DiffViewer chiếm vùng `main`, KHÔNG thêm panel thứ tư — AppLayout.tsx không đổi một dòng"
  - "Đường A (@codemirror/merge) sau interface DiffRenderer; hợp nhất KHÔNG để CM tính lại diff"
  - "MIN_SPLIT_WIDTH = 720, đo bằng ResizeObserver trên panel, có test ghim hai phía"
  - "🔴 Cổng bundle của plan BẤT KHẢ trên Vite 8 — đã thay bằng cổng nội dung, xem mục Phát hiện đo được"
metrics:
  duration: "~150 phút"
  completed: "2026-09-22"
  tasks_completed: "2/3 (Task 3 là checkpoint, ĐANG CHỜ chủ dự án)"
  tests_added: "129 frontend"
  tests_total: "365 frontend (mốc 236), 257 Rust + 1 ignored (không đổi)"
---

# Phase 3 Plan 04: Trình xem diff Summary

Trình xem diff thật: tô màu cú pháp nạp lười theo phần mở rộng, hai chế độ hiển thị,
nhảy khối có nói-ra-khi-hết, bật tắt khoảng trắng không mất vị trí cuộn, word-level từ
`spans` của git, và năm dạng thông báo DIFF-06 — tất cả nằm sau `interface DiffRenderer`
để việc đổi đường A→B là sửa **một** tệp. Cộng **ba** cổng xanh sai phải sửa, trong đó
một cổng của plan là **bất khả** trên Vite 8.

**⏸️ Task 3 (checkpoint 11 bước) CHƯA CHẠY** — chờ chủ dự án. Bản release đã dựng xong.

---

## Quyết định A/B, và hệ quả của việc checkpoint #3 bị bỏ qua

**Đường A (`@codemirror/merge`), chọn KHÔNG CÓ SỐ ĐO.** Cơ sở ghi nguyên văn từ
`docs/09-phase3-diff-decision.md` mục 7 (commit `6910732`): checkpoint #3 bị chủ dự án bỏ
qua ngày 2026-09-22, và đường A được chọn **vì nó là mặc định khi thiếu bằng chứng**, không
vì đã chứng minh đủ nhanh.

Mục 8 của tài liệu đó nêu điều kiện kèm theo, và plan này **thi hành nó như ràng buộc tuyệt
đối**:

> *"03-04 phải giữ trình xem sau một interface để việc đổi đường là sửa một tệp, không phải
> sửa cả giao diện."*

Cách thi hành, đúng khuôn `interface GraphRenderer` của Phase 2:

| Tệp | Vai trò |
|---|---|
| `lib/diff-render/types.ts` | `interface DiffRenderer` + `DiffRendererOptions`. Không mã chạy nào của CodeMirror |
| `lib/diff-render/codemirrorRenderer.ts` | **Đường A. Tệp DUY NHẤT trong dự án nhập `@codemirror/merge`** |
| `components/diff/DiffViewer.tsx` | Tầng giao diện. Nhập **đúng một** hàm dựng, **không** nhập gì từ `@codemirror` |

Đổi sang đường B = viết `hunkRenderer.ts` cạnh `codemirrorRenderer.ts` rồi đổi **một** dòng
`import` trong `DiffViewer.tsx`.

**Cổng giữ lời hứa đó:** `lib/diff-render/interface-boundary.test.ts`, tương đương cổng
`grep getContext GraphCanvas.tsx` → 0 của Phase 2 nhưng ở dạng test thật (plan cấm cổng
`grep ... == 0` trên tệp có doc comment tiếng Việt dày — và `DiffViewer.tsx` **có** nhắc
`@codemirror/merge` trong comment giải thích quyết định A/B, nên một cổng grep sẽ khớp nhầm
chính đoạn văn đó). Đã kiểm là **có thể đỏ**: rò một `import { MergeView }` vào
`DiffViewer.tsx` → **3 test đỏ**.

Test đó còn ghim một điều mà danh sách cho-phép một mình không ghim được: hai tệp trong danh
sách (`langLoader.ts`, `types.ts`) được thêm với lý do *"chỉ nhập KIỂU"*, và có test riêng
khẳng định mọi dòng nhập của chúng là `import type` — nếu không, lý do trong danh sách chỉ là
một câu văn không ai kiểm.

**Một điều làm giảm rủi ro, đo được:** kể cả ở đường A, chế độ **hợp nhất** **không** để
CodeMirror tính lại diff — `git diff` đã tính rồi và `decorations.ts` dựng `DecorationSet` từ
đầu ra đó. Chỉ chế độ **hai cột** dùng `MergeView`. Nên nguy cơ hiệu năng của checkpoint #3
áp cho **một** chế độ, và chế độ đó **không** phải mặc định (`viewMode = 'unified'`). Nếu
`MergeView` chậm trên tệp lớn, đường thoát của người dùng là chuyển về hợp nhất — một suy
giảm có thật, không phải treo giao diện.

---

## Quyết định bố cục — BƯỚC 1 CỦA CHECKPOINT, cần chủ dự án xác nhận

`<action>` Task 2 cho hai lựa chọn và đòi ghi lý do. **Đã chọn (a):** khi có tệp đang chọn,
`DiffViewer` chiếm vùng `main` và đồ thị commit nhường chỗ.

**Vì sao không chọn (b)** — thêm một `Panel` thứ tư, vốn plan gọi là an toàn hơn về hồi quy:

- Chế độ **hai cột** cần hai cột nội dung **cộng** hai cột số dòng trong **một** panel. Thêm
  panel thứ tư chia bề rộng cửa sổ thành **bốn** phần, và `<layout_constraints>` mục 4 gọi
  hẹp là *"ca hẹp tệ nhất của chế độ hai cột"*. Cách (b) làm ca tệ nhất tệ hơn.
- Cách (a) cho diff đúng vùng rộng nhất (`main`, `defaultSize="52%"`) và — điều plan có thể
  chưa lường — **cũng không đổi một dòng nào** trong `AppLayout.tsx`, vì nó chỉ đổi thứ được
  render *bên trong* vùng `main`. Nên lợi thế hồi quy mà plan gán cho (b) thật ra cả hai cách
  đều có.

**Đánh đổi đang nhận:** đồ thị bị che khi đang xem diff. Nút "Đóng diff" đưa nó về, và
`FileList` ở vùng `detail` vẫn hiện nên người dùng không mất ngữ cảnh commit.

🔴 Đây là **thay đổi bố cục** và Phase 2 cho thấy bố cục là chỗ hay sai nhất — nên nó là
**bước 1** của checkpoint để chủ dự án xác nhận hoặc yêu cầu đổi sang (b).

---

## 🔴 Phát hiện đo được — chỗ plan sai, và chỗ công cụ khác điều plan dự đoán

### 1. 🔴 Plan SAI: cổng bundle theo TÊN chunk là bất khả trên Vite 8

`<done>` Task 2 viết: *"khẳng định ≥2 tệp js mà tên chứa `lang` hoặc `javascript`/`rust`"*.

Đo thật trên Vite 8.3.0: Vite đặt tên chunk theo **tên tệp entry của gói**, và mọi gói
`@codemirror/lang-*` có entry là `dist/index.js`. Nên **năm** chunk lang đều tên
`dist-<băm>.js`, và **không tệp nào** chứa `lang`, `javascript` hay `rust` trong tên:

```
dist/assets/dist-CxyXM1QH.js    92.88 kB   ← lang-javascript
dist/assets/dist-SFc2S7QS.js    83.68 kB   ← lang-rust
dist/assets/dist-BIoOWjef.js    28.37 kB
dist/assets/dist-NCLBF_bC.js    26.79 kB
dist/assets/dist-1ugLc69A.js     1.99 kB
dist/assets/index-DhGy75Pd.js  619.79 kB   ← entry
```

Plan nói rõ phải làm gì trong ca này: *"không đổi cổng thành một phép kiểm dễ hơn mà vô
nghĩa."* Nên cổng (`scripts/check-lang-chunks.mjs`) kiểm **đúng bất biến mà phase cần**, chỉ
bằng phép đo khác — neo vào **nội dung** thay vì **tên**:

1. Chunk **entry** không chứa bảng parser nào. *Đây là điều thật sự quan trọng*: Core Value là
   thời gian khởi động, và nó bị hại đúng khi mã lang nằm trong chunk entry.
2. Có **≥4** chunk không-entry mang bảng parser — loại riêng ca "điều kiện 1 xanh chỉ vì
   không ai nhập gói lang-* cả".

**Cổng đã kiểm là có thể đỏ**, không chỉ là xanh: đổi `langLoader` sang `import` tĩnh rồi
`vite build` →

```
LỖI: chunk entry index-CvHFk8kW.js chứa `stateData:,nodeNames:,tokenPrec:` — mã bộ phân tích ngôn ngữ.
Gói lang-* đã vào bundle khởi động — nạp lười bị phá, và thời gian khởi động
là Core Value của dự án.
```

### 2. Chọn dấu hiệu cho cổng bundle — BA lần thử, hai lần đầu đo sai đại lượng

Ghi cả ba vì đây là bài học, không phải chi tiết:

| Lần | Dấu hiệu | Kết quả | Nguyên nhân |
|---|---|---|---|
| 1 | `javascriptLanguage`, `rustLanguage`, … | **2/5** — sai | Bộ rút gọn **đổi tên** định danh |
| 2 | `LRParser.deserialize(` | **0/5** — sai | Cũng bị rút gọn: chunk thật có `z=l.deserialize(`. Phép probe ban đầu dùng `OR` với `deserialize(` nên nó đo một chuỗi **khác** chuỗi mà cổng dùng |
| 3 | `stateData:` + `nodeNames:` + `tokenPrec:` | **5/5** ✅ | **Khoá object literal** không bị rút gọn (đổi chúng phá chính `LRParser.deserialize`) |

Cố ý **không** dùng từ chung như `rust`/`json`: hai từ đó có trong chunk entry vì lý do khác
(khoá phần mở rộng trong bảng tra của `langLoader.ts`), và cổng dùng chúng sẽ **đỏ vì lý do
sai** — tệ hơn cổng luôn xanh, vì nó dạy người sau bỏ qua cổng.

### 3. 🔴 Lỗ kế hoạch: DIFF-05 chưa được cài ở wave nào

Phát hiện khi lập `VERIFICATION.md`, **không** thuộc phạm vi plan này nhưng phải ghi ra.

DIFF-05 (*"mở lịch sử thay đổi của riêng một tệp và lần theo được các phiên bản của nó"*)
thuộc Phase 3 theo ROADMAP (`Requirements: DIFF-01..DIFF-06`) và là **tiêu chí thành công số
3** của phase. Nhưng không wave nào nhận nó:

| Wave | DIFF-05? |
|---|---|
| 03-01 spike A/B | không |
| 03-02 backend diff | không |
| 03-03 word-level | không |
| 03-04 (plan này) | không — `requirements: [DIFF-01, DIFF-02, DIFF-03, DIFF-04, DIFF-06]` |
| 03-05 exit gate | chưa lập kế hoạch chi tiết |

Không có `git log --follow -- <path>` ở phía Rust, không có component nào ở phía TS. Tiêu chí
thành công số 3 của phase **không thể đạt** với mã hiện có.

`03-05` phải xử lý — hoặc phải có một quyết định **tường minh** hoãn DIFF-05, như ROADMAP đã
làm với minimap/Blame. Ghi ở đây và trong `STATE.md` để nó không lọt qua exit gate trong im
lặng.

### 4. `await Promise.resolve()` KHÔNG flush render của React 19

Ba `await Promise.resolve()` liên tiếp **không** đủ để React 19 flush một `setState` gọi từ
callback của promise. Hệ quả: test chống đua xanh **kể cả ở cài đặt đã bị đột biến**, vì DOM
chưa kịp mang giá trị cũ lúc `expect` chạy. Đây là lớp lỗi "đo sai **thời điểm**", họ hàng với
bài học `measureFirstPaint` của Phase 2 (đo tới `useEffect` đầu tiên thay vì tới lúc có dữ
liệu). Bản sửa: `act(async () => { await new Promise((r) => setTimeout(r, 0)) })`.

---

## Kiểm mutation — 13/13 đã chạy, **ba** cổng xanh sai phải sửa

Cột cuối là output đỏ thật, không diễn giải.

| # | Đột biến | Kết quả | Test đỏ |
|---|---|---|---|
| 1 | `langLoader`: `import()` động → `import` tĩnh | ✅ **2 đỏ** | `KHÔNG có import ... @codemirror/lang-*`, `thân bảng tra chứa import() động` |
| 2 | `viewMode` → `viewModeByRepo` | ✅ **5 đỏ** | gồm `đổi chế độ ở repo A → repo B thấy CÙNG chế độ` |
| 3 | bỏ reset `selectedFile` khi đổi commit | ✅ **2 đỏ** | `commitChanged đặt selectedFile về null` |
| 4 | `lfsPointer` hiện cỡ tệp con trỏ thay `kind.size` | ✅ **1 đỏ** | `con số là kind.size, KHÔNG phải cỡ tệp con trỏ` |
| 5 | **bỏ chuyển byte → UTF-16** | ✅ **4 đỏ** | **mutation then chốt**, xem dưới |
| 6 | `nextHunkLine` nhảy vòng thay vì `null` | ✅ **2 đỏ** | `ở khối CUỐI → trả NULL`, `diff MỘT khối → null NGAY` |
| 7 | bỏ `view.destroy()` ở cleanup | ✅ **2 đỏ** | `dựng 6 view nhưng chỉ destroy 0` |
| 8 | bỏ cổng chống đua | ⚠️⚠️ **0 đỏ HAI LẦN** → sửa **hai** khiếm khuyết → ✅ 1 đỏ | xem "Ba cổng xanh sai" |
| 9 | `Compartment` → dựng lại `EditorView` | ✅ **1 đỏ** | `CÙNG MỘT instance EditorView trước và sau` |
| 10a | xoá `overflow: hidden` khỏi `.diff-viewer` | ✅ **1 đỏ** | `.diff-viewer có overflow: hidden VÀ min-height: 0` |
| 10b | thêm `minmax(0, 1fr)` vào `.diff-header` | ✅ **1 đỏ** | `không có chuỗi minmax(0, trong quy tắc .diff-*` |
| 10c | `.diff-word-changed` → `var(--bg-inset)` đục | ✅ **1 đỏ** | `dùng nền có alpha` |
| 11 | `langLoader` tĩnh → `vite build` → cổng bundle | ✅ **đỏ** | `chunk entry chứa stateData:,nodeNames:,tokenPrec:` |
| 12 | `--min-split-width` 720 → 640, TS giữ 720 | ✅ **1 đỏ** | `--min-split-width bằng MIN_SPLIT_WIDTH` |
| + | rò `import { MergeView }` vào `DiffViewer.tsx` | ✅ **3 đỏ** | cổng biên giới interface (ngoài plan) |

### Output đỏ thật (trích nguyên)

**#5 — mutation then chốt. Bốn test đỏ, và ca ASCII XANH đúng như dự đoán:**
```
--- RED: 🔴 tiếng Việt thật: byte 12 ≠ UTF-16 index 9
AssertionError: expected null not to be null

--- RED: 🔴 emoji ngoài BMP: 4 byte nhưng 2 đơn vị UTF-16
TypeError: Cannot read properties of null (reading 'from')

--- RED: 🔴 lineMeta mang spans đã chuyển sang UTF-16, không phải byte thô
AssertionError: expected [ { from: 9, to: 11 } ] to deeply equal [ { from: 7, to: 9 } ]
```
Ca `ASCII thuần` **vẫn xanh** — plan tự nêu điều kiện này và nó đúng: hai hệ đếm bằng nhau
trên ASCII nên ca đó không chứng minh được gì. Nó vẫn ở trong bộ test, có comment nói rõ nó
**không phải** cổng, để người sau không tưởng nó là.

**#1 — 18/20 test hành vi VẪN XANH ở cài đặt đã phá ràng buộc bundle hoàn toàn:**
```
AssertionError: nhập tĩnh gói lang-* kéo cả sáu gói vào bundle khởi động. Dòng vi phạm:
import { javascript } from '@codemirror/lang-javascript'
import { rust } from '@codemirror/lang-rust'
...
AssertionError: thân LANG_LOADERS phải có một `import(` cho mỗi khoá (9 khoá), đếm được 0
```

**#7 — con số thật, không suy ra từ DOM:**
```
AssertionError: dựng 6 view nhưng chỉ destroy 0 — mỗi lần chọn tệp rò một EditorView,
và người dùng bấm qua hàng chục tệp trong một phiên: expected 6 to be less than or equal to 1
```

**#2 — năm test đỏ, gồm đúng cổng đã thiết kế:**
```
--- RED: 🔴 đổi chế độ ở repo A → repo B thấy CÙNG chế độ (không theo repo)
AssertionError: expected 'unified' to be 'split'
```

**#12 — cổng hằng số hai phía:**
```
AssertionError: CSS có --min-split-width: 640px nhưng DiffViewer.tsx có MIN_SPLIT_WIDTH = 720.
JS quyết định "hai cột hay hợp nhất" theo con số TS, còn CSS đặt sàn cột theo con số CSS —
lệch nhau thì có một dải bề rộng mà JS hiện hai cột trong khi CSS đã co cột xuống dưới mức
đọc được.
```

**#8 sau khi sửa:**
```
AssertionError: nội dung đang hiện phải thuộc tệp ĐANG CHỌN (b.ts). Phản hồi của tệp không
còn được chọn phải bị BỎ, không ghi đè — không có cổng này thì giao diện hiện diff của tệp
khác mà KHÔNG lỗi nào (T-03-31).: expected 'a.ts' to be 'b.ts'
```

---

## Ba cổng xanh sai — chi tiết, vì chúng là bài học

Wave 2 và 3 có **năm** mutation xanh lần đầu. Plan yêu cầu ghi cả hai lượt. Wave này có
**ba**, và một trong đó xanh sai **hai lần liên tiếp vì hai nguyên nhân độc lập**.

### Mutation #8, khiếm khuyết A: nhãn đọc TRẠNG THÁI thay vì DỮ LIỆU

Test chống đua đọc `data-testid="diff-path"`. Nhưng phần tử đó render từ **`selectedFile`** —
tức từ `diffStore`, không từ dữ liệu đã về:

```tsx
{diff?.oldPath ? `${diff.oldPath} → ${selectedFile}` : selectedFile}
```

Nên nó hiện `b.ts` **bất kể** phản hồi nào đã ghi đè `diff`. Cổng không quan sát gì, và
mutation cho **0 đỏ**.

Điều làm khiếm khuyết này đáng ghi: nó **không chỉ** là lỗi test. Một nhãn đọc `selectedFile`
**che** đúng cái lỗi mà cổng chống đua tồn tại để chặn — người dùng thật sẽ thấy tiêu đề
`b.ts` trên nội dung của `a.ts` và không có cách nào biết. Nên bản sửa nằm ở **mã sản phẩm**,
không ở test: nhãn đọc `diff.path`, cộng một `diff-content-path` là bằng chứng "nội dung đang
hiện thuộc tệp nào".

### Mutation #8, khiếm khuyết B: đo TRƯỚC khi React flush

Sau khi sửa A, mutation **vẫn 0 đỏ**. Nguyên nhân thứ hai, độc lập: ba
`await Promise.resolve()` không flush render của React 19 (xem "Phát hiện đo được" **#4**).
Test xanh vì đo quá sớm. Sửa bằng `act()`.

Hai khiếm khuyết này minh hoạ đúng điều wave 3 đã ghi: *khi một mutation cho 0 đỏ, kiểm cả
cổng lẫn dữ liệu* — ở đây là cổng đọc sai **đại lượng**, rồi đọc đúng đại lượng nhưng sai
**thời điểm**.

### Cổng bundle: đo TÊN chunk (bundler đặt) thay vì NỘI DUNG

Xem "Phát hiện đo được" #1 và #2. Cổng của plan neo vào một thứ mà **bundler** quyết định và
dự án không kiểm soát; cổng đúng neo vào nội dung chunk entry, thứ mà lập trình viên kiểm
soát qua việc dùng `import()` hay `import`.

### Một cổng thứ tư, tự bắt trong lúc viết

Test `chỉ được có ĐÚNG một khoá viewMode` ban đầu lọc **mọi** khoá của store, nên nó đỏ ở
cài đặt **đúng** — `setViewMode` và `toggleViewMode` cũng khớp `/viewMode/i`. Một cổng đỏ vì
lý do sai. Sửa: lọc bỏ khoá có giá trị là **hàm**, chỉ giữ khoá **trạng thái**, cộng một
khẳng định `khoá kết thúc bằng ByRepo === ['selectedFileByRepo']`.

---

## Cổng verification — cả 12 cổng của plan, cộng hai cổng thêm

| # | Cổng | Kết quả |
|---|---|---|
| 1 | `npm test langLoader` + mutation import tĩnh | ✅ 2 đỏ |
| 2 | `npm test diffStore` + `viewModeByRepo` | ✅ 5 đỏ |
| 3 | `npm test diffStore` + bỏ reset | ✅ 2 đỏ |
| 4 | `npm test DiffToolbar` + `lfsPointer` sai số | ✅ 1 đỏ |
| 5 | `npm test decorations` + bỏ byte→UTF-16 | ✅ **4 đỏ** (then chốt) |
| 6 | `npm test decorations` + nhảy vòng | ✅ 2 đỏ |
| 7 | `npm test DiffViewer` + bỏ `view.destroy()` | ✅ 2 đỏ — **kiểm được**, không phải "không kiểm được" |
| 8 | `npm test DiffViewer` + bỏ cổng chống đua | ⚠️⚠️ 0 đỏ ×2 → sửa → ✅ 1 đỏ |
| 9 | `npm test DiffViewer` + `Compartment` → dựng lại | ✅ 1 đỏ |
| 10 | `npm test app.css` + vi phạm từng điều (ba test) | ✅ đúng test đó đỏ, cả ba |
| 11 | `npm run build` + cổng chunk | ✅ **đã thay cổng** (plan bất khả), đã kiểm đỏ |
| 12 | `npm test app.css` + lệch `MIN_SPLIT_WIDTH` một phía | ✅ 1 đỏ — **áp dụng được**, hằng số sống hai phía thật |
| +13 | biên giới interface (`@codemirror/merge` một tệp) | ✅ 3 đỏ khi rò — **ngoài plan** |

**Cổng 12 ÁP DỤNG ĐƯỢC, không phải "không áp dụng".** `<done>` Task 2 cho phép ghi "không cần
test ghim" nếu cài đặt cuối không cần con số đó trong CSS. Cài đặt này **cần**:
`.diff-split { min-width: var(--min-split-width) }` là lớp phòng thủ thứ hai cho ca JS và CSS
lệch nhau. Nên test ghim tồn tại, và nó đọc `app.css` bằng `readFileSync` (**không** `?raw` —
`app.css.test.ts` đã xác nhận `?raw` trả chuỗi rỗng dưới vitest).

### Cổng hạ tầng

| Cổng | Kết quả |
|---|---|
| `npm run typecheck` | ✅ sạch |
| `npm test` | ✅ **365 passed**, 30 tệp (mốc 236) |
| `npm run build` | ✅ xanh |
| `node scripts/check-lang-chunks.mjs` | ✅ entry sạch, 5 chunk ngôn ngữ tách riêng |
| `cargo test` | ✅ **257 passed, 1 ignored** — **không đổi**, plan này không sửa Rust |
| `cargo clippy --all-targets` | ✅ 0 issue |
| `npm run tauri:build` | ✅ MSI xong (`git-plum_0.1.0_x64_en-US.msi`) |

Không hồi quy: 236 → 365 frontend (+129); 257 → 257 Rust.

**Cổng grep KHÔNG dùng** (plan nêu rõ bốn cổng, đã tôn trọng cả bốn):
- `grep -c 'runCommand' FileList.tsx` → thay bằng render + `fireEvent.click` + thay `run` của
  lệnh đã đăng ký bằng spy.
- `grep -c 'Decoration' decorations.ts` → thay bằng `buildDecorations` trả `lineCount`/
  `markCount` và test khẳng định **số deco thật** cho một `lineMeta` đã biết.
- `grep 'codemirror' package.json` → `node -e` đọc JSON, tra khoá.
- Cổng `== 0` trên tệp có comment tiếng Việt → mọi cổng đọc mã nguồn trong wave này **lọc
  chú thích trước**, và có một khẳng định **tiền đề** (`ma.contains('LANG_LOADERS')`,
  `tepNguon.length > 20`) để phép lọc ăn mất mã không làm cổng luôn xanh.

---

## Chỗ bàn giao nguy hiểm nhất: byte vs UTF-16 — đã xử lý

`Span` từ wave 3 mang chỉ số **byte**; CodeMirror đánh chỉ số theo **UTF-16 code unit**.

Phép chuyển nằm ở `spanToUtf16` trong `decorations.ts`, gọi **một lần** ngay lúc dữ liệu vào
tầng vẽ (`hunksToDoc`) chứ không muộn hơn — chuyển trong `buildDecorations` nghĩa là
`lineMeta` mang byte và một người gọi khác sẽ dùng thẳng con số đó.

Kiểu trả về là `Utf16Span { from, to }`, **khác tên trường** với `Span { start, end }`. Đây
không phải thẩm mỹ: nó làm việc truyền lẫn hai hệ thành lỗi **biên dịch** thay vì lỗi hiển
thị im lặng.

Số đo cụ thể đã ghim bằng test:

| Nội dung | Vị trí byte | Vị trí UTF-16 |
|---|---|---|
| `'xéy dỏng TEST'` → `TEST` | 12 | **9** |
| `'x🙂y CHANGED'` → `CHANGED` | 7 | **5** |

`TextEncoder`/`TextDecoder`, **không** `Buffer` — mã sản phẩm là browser-only.

**`spans` rỗng là ca bình thường**, đúng như wave 3 ghi: bốn đường dẫn hợp lệ tới nó, và tô
cả dòng là suy giảm đúng. Có test riêng (`spans rỗng là ca bình thường`, `dòng KHÔNG span:
chỉ có line-deco`).

**Dữ liệu backend lỗi → bỏ span, KHÔNG ném (T-03-24).** CodeMirror ném khi range vượt biên
document, và một ngoại lệ ở tầng này làm **trắng cả panel** — người dùng mất luôn phần diff
đúng vì một span sai. Bốn ca bỏ: `start >= end`, `end > byteLength`, offset âm, và
`from >= to` sau khi chuyển (phòng thủ cuối cho ca offset không nằm trên biên ký tự — phía
Rust bảo đảm là có, nhưng bảo đảm của người khác không phải phép kiểm của mình).

---

## `MIN_SPLIT_WIDTH` = 720, đo bằng `ResizeObserver` trên panel

**Không** `window.innerWidth`: panel kéo được, nên bề rộng cửa sổ không nói gì về bề rộng
panel — người dùng có thể kéo panel diff hẹp lại trong một cửa sổ rộng 2560px. Đúng lớp lỗi
"đo đại lượng toàn cục cho một bất biến cục bộ" mà mutation #8 của 03-02 đã gặp ở phía Rust.

Con số sống ở **hai** nơi và có test ghim (cổng 12): `DiffViewer.tsx` và
`--min-split-width` trong `app.css`.

Dưới 720px → hiện **hợp nhất** kèm một dòng nói vì sao (`data-testid="diff-forced-unified"`).
Đây là **suy giảm có chủ ý**, cùng khuôn `MAX_VISIBLE_LANES` + chỉ báo `+N cha nữa` của
Phase 2 — im lặng hiện hai cột 80px mỗi cột tệ hơn.

`width === 0` (`ResizeObserver` chưa chạy lần đầu) → buộc hợp nhất. Đoán "rộng" sẽ nháy một
khung hai cột 0px rồi sửa lại.

---

## ⏸️ Những gì vẫn CHƯA KIỂM CHỨNG — ghi thẳng, không lấp bằng suy luận

**happy-dom không tính layout CSS và không có cuộn thật.** Ba lỗi hiển thị của Phase 2 qua
hết 212 test tự động. Nên 365 test xanh **không** nói được bốn điều sau, và chúng chờ
checkpoint:

| Chưa kiểm | Vì sao test không kiểm được |
|---|---|
| Chữ có **hiển thị** hay bị co mất | happy-dom không tính grid track sizing |
| Hai cột có **thẳng hàng** | không có layout |
| Khoảng trắng có **hiện ra** trên màn hình | không render glyph |
| Word-level có tô **đúng chỗ** về pixel | không có layout |
| Vị trí cuộn có **giữ nguyên** khi bật cờ | không có cuộn thật. Test chỉ ghim "cùng một instance `EditorView`" — điều kiện **cần**, không **đủ** |
| Chế độ **hai cột** nói chung | `ResizeObserver` không có trong happy-dom → `paneWidth` giữ 0 → mọi test chạy ở nhánh hợp nhất |

**Chưa đo bằng Playwright.** Plan cho phép hai lựa chọn — đo bằng Chromium thật, hoặc ghi rõ
là chưa kiểm chứng. Chọn lựa chọn thứ hai và ghi rõ: **không có số đo Playwright nào trong
wave này.** Lý do: checkpoint 11 bước đo trên **WebView2 thật** (bản release), và 02-06 đã ghi
rằng phông thay thế của Chromium headless đo khác Segoe UI thật — nên Playwright ở đây sẽ cho
một con số thứ ba không ai dùng. Nếu checkpoint bị hoãn thì Playwright là bước tiếp theo, và
quy trình của 02-05 (cài ở thư mục **scratch**, **không** thêm vào `package.json`) vẫn dùng
được.

**Và điều lớn nhất:** thời gian mở diff của `MergeView` trên tệp lớn thật vẫn **chưa có số**.
Checkpoint #3 bị bỏ qua; wave này không đo lại và **không chép số từ đâu**. `SpikeHarness` và
`spike_blob_pair` vẫn còn — mục 8 của `docs/09` nói đừng xoá tới khi wave 5 đóng.

---

## Deviations from Plan

### 1. [Rule 1 - Bug] Cổng bundle của plan bất khả trên Vite 8

Xem "Phát hiện đo được" #1. Đã thay bằng cổng nội dung và kiểm là có thể đỏ.
**Commit:** `44f0a7b`

### 2. [Rule 1 - Bug] Nhãn đường dẫn đọc trạng thái store, che mất lỗi đua

Xem "Ba cổng xanh sai", khiếm khuyết A. Sửa ở **mã sản phẩm**, không ở test — nhãn đọc
`selectedFile` là một lỗi thật mà người dùng gặp được, không chỉ một cổng yếu.
**Commit:** `b84f460`

### 3. [Rule 2 - Correctness] Thêm cổng biên giới interface, ngoài plan

Plan đòi giữ trình xem sau interface nhưng không đòi cổng nào cho việc đó. Prompt của wave
nêu rõ: *"Có cổng kiểm việc này thì tốt."* Đã thêm
`lib/diff-render/interface-boundary.test.ts` (7 test) vì một interface không ai kiểm sẽ bị rò
qua trong vài plan — Phase 2 có cổng tương đương cho `GraphRenderer`. **Commit:** `b84f460`

### 4. [Rule 2 - Correctness] `FileList` nhận `repoId` optional

Bảy test `FileList` hiện có gọi component không có `repoId`. Prop bắt buộc sẽ phá cả bảy —
ngoài phạm vi task. Prop optional, và có test cho ca đó (`không truyền repoId → hàng vẫn
render, chỉ là bấm không làm gì`). **Commit:** `da12f22`

### 5. Thêm test ngoài plan

Plan đòi "ít nhất 16 + 20 = 36 test mới". Thực tế **+129** (236 → 365), đếm bằng reporter
JSON chứ không ước lượng:

| Tệp test | Số test |
|---|---|
| `decorations.test.ts` | 33 |
| `DiffToolbar.test.tsx` | 21 |
| `app.css.test.ts` | 21 (17 cũ + **4 mới**) |
| `langLoader.test.ts` | 20 |
| `DiffViewer.test.tsx` | 18 |
| `diffStore.test.ts` | 16 |
| `FileList.diff.test.tsx` | 8 |
| `interface-boundary.test.ts` | 7 |

Phần vượt chủ yếu là ca biên của `extensionOf` (bốn lớp đường dẫn: không dấu chấm, dot-file,
dấu chấm trong thư mục, nhiều dấu chấm) và ca dữ liệu lỗi của `spanToUtf16`.

---

## Known Stubs

Không có stub chặn mục tiêu của plan. Hai điều **có chủ ý** giới hạn, ghi rõ để không ai nhầm
là stub:

1. **`MergeView` tắt `revertControls`.** Nút merge/gộp khối là việc của Phase 5 (staging theo
   khối), không của trình **xem**. Đây là phạm vi, không phải việc chưa xong.
2. **`DiffToolbar` đăng ký `diff.nextHunk`/`diff.prevHunk` kể cả khi chưa mở tệp**, và lệnh
   không làm gì trong ca đó. Cố ý: đăng ký có điều kiện làm `runCommand` **ném** khi người
   dùng gõ phím tắt lúc chưa mở tệp, và một ngoại lệ cho thao tác vô hại là tệ hơn.

Và một điểm chưa kiểm chứng, **không** phải stub: xem mục "Những gì vẫn CHƯA KIỂM CHỨNG".

---

## Threat Flags

Không có bề mặt an ninh mới ngoài `<threat_model>` của plan. Tám threat từ T-03-24 đến T-03-31
đều có đường mã và ít nhất một test:

| Threat | Cổng |
|---|---|
| T-03-24 (span vượt biên → panel trắng) | `spanToUtf16` trả `null` cho bốn ca lỗi; 3 test, gồm ca ba span trong đó hai lỗi |
| T-03-25 (byte dùng thẳng làm UTF-16) | `spanToUtf16` + mutation #5, **4 đỏ** trên nội dung tiếng Việt và emoji |
| T-03-26 (`path` nội suy vào `import()`) | bảng tra **cố định**; test khẳng định không `import(\`@codemirror/lang-${ext}\`)` và không nối chuỗi |
| T-03-27 (tệp sát 5 MB làm đơ webview) | **accept** theo plan. Bước 11 của checkpoint là phép kiểm bằng người |
| T-03-28 (rò `EditorView`) | `renderer.destroy()` ở cleanup + mutation #7, đếm `created - destroyed` thật |
| T-03-29 (rò nội dung nhị phân/LFS) | bốn dạng không-`text` **không** dựng `EditorView`; 4 test khẳng định `created === 0` |
| T-03-30 (đường dẫn vào console) | `measureFirstPaint('diff-first-paint')` — nhãn **hằng chuỗi**, không nội suy |
| T-03-31 (hiện diff của tệp trước) | cổng `requestIdRef` + mutation #8. **Hai** khiếm khuyết cổng đã sửa trước khi nó quan sát được |

---

## Self-Check: PASSED

Tệp đã kiểm tồn tại, số dòng đọc bằng `wc -l`:

| Tệp | Dòng | Mốc plan |
|---|---|---|
| `src/lib/diff-render/langLoader.ts` | 128 | ≥60 ✅ |
| `src/lib/diff-render/decorations.ts` | 264 | ≥120 ✅ |
| `src/components/diff/DiffViewer.tsx` | 355 | ≥150 ✅ |
| `src/stores/diffStore.ts` | 80 | ≥50 ✅ |
| `src/lib/diff-render/types.ts` | 96 | — |
| `src/lib/diff-render/codemirrorRenderer.ts` | 202 | — |
| `src/lib/diff-render/theme.ts` | 75 | — |
| `src/components/diff/DiffToolbar.tsx` | 208 | — |
| `scripts/check-lang-chunks.mjs` | 123 | — |

Chuỗi commit RED→GREEN đầy đủ trong `git log`:

```
b84f460 fix(03-04): make the race gate observable and pin the renderer boundary
44f0a7b feat(03-04): implement the diff viewer behind a renderer interface
20f3d34 test(03-04): add failing tests for decorations, DiffViewer and diff layout   ← RED
da12f22 feat(03-04): implement diff store, lazy lang loader and DIFF-06 notices
8b3a7f4 test(03-04): add failing tests for diff store, lang loader and DIFF-06 notices ← RED
```

`key_links` của frontmatter đã kiểm bằng phép khớp mẫu thật:

| from → to | pattern | Kết quả |
|---|---|---|
| `FileList.tsx` → `diffStore.ts` | `runCommand\('diff\.` | ✅ `runCommand('diff.selectFile')` |
| `DiffViewer.tsx` → `ipc.ts` | `getFileDiff` | ✅ `ipc.getFileDiff(repoId, selectedCommitId, selectedFile)` |
| `decorations.ts` → `domain/diff.rs` | `Decoration\.(line\|mark)` | ✅ cả hai, dựng từ `Hunk`/`DiffLine`/`Span` |
