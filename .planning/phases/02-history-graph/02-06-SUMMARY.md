---
phase: 02-history-graph
plan: 06
subsystem: history-detail-search-refs
tags: [zustand, react, commit-detail, file-tree, ref-badges, debounce, checkpoint]

requires:
  - phase: 02-05
    provides: "historyStore.ts (byRepo[repoId], commits/graphRows), CommitList.tsx one-virtualizer scroll container, useState-based selectedCommitId in App.tsx (upgraded in this plan)"
provides:
  - "src/stores/selectionStore.ts — selectedByRepo[repoId], Pattern 3 guard (id only, no commit data)"
  - "src/stores/refsStore.ts — byRepo[repoId].byCommit index (Map<commitId, GitRef[]>), counts()"
  - "src/stores/uiStore.ts — fileListView ('flat'|'tree'), survives commit changes"
  - "src/lib/fileTree.ts — buildFileTree pure function, split on / only"
  - "src/components/history/CommitDetail.tsx — HIST-08, module-level cache by commitId"
  - "src/components/history/FileList.tsx — HIST-09 flat/tree toggle, no onClick on rows"
  - "src/components/history/RefBadges.tsx — HIST-06, prop-driven (no store subscription per row)"
  - "src/components/RefSidebar.tsx — HIST-07, three groups with counts, detached HEAD line"
  - "src/components/history/CommitSearch.tsx — HIST-10, 250ms debounce, scrollToIndex via CommitList ref"
  - "CommitList.tsx — forwardRef exposes scrollToIndex, RefBadges rendered inline in .commit-subject"
  - "App.tsx — selectedCommitId upgraded from useState to selectionStore; RefSidebar/CommitDetail/CommitSearch wired in"
affects:
  - "02-07 (checkpoint #1 performance): CommitSearch's indexById Map and refsStore.byCommit index are both O(1) lookups already validated at 3 refs; need re-validation at 100k-commit / thousands-of-refs scale"
  - "Phase 3 (diff viewer): FileList rows intentionally have no onClick — Phase 3 is the first plan allowed to add diff-drill-down behavior there"

tech-stack:
  added: []
  patterns:
    - "selectionStore holds ONLY selectedCommitId per repo — CommitDetail and RefSidebar both read/write it independently, neither is a child of the other (ARCHITECTURE.md Pattern 3)"
    - "refsStore.byCommit is a Map computed once in load(), not scanned per row — RefBadges receives refs as a prop from CommitList's lookup, never subscribes to refsStore itself"
    - "New ref badges rendered INSIDE .commit-subject (existing overflow:hidden text node), not as a new CSS grid column — checkpoint round 1 of 02-05 already proved new max-content/minmax columns collapse at narrow width + high lane count"
    - "CommitDetail caches CommitDetail payloads in a bounded module-level Map (200 entries) keyed by commitId — historical diffs are immutable so the cache never needs invalidation, only a size cap"
    - "CommitList exposes scrollToIndex via forwardRef + useImperativeHandle — CommitSearch consumes it as a prop instead of creating a second useVirtualizer instance"
    - "uiStore (not selectionStore, not per-repo) holds fileListView — a user display preference that must survive both commit changes and repo changes"

key-files:
  created:
    - "src/stores/selectionStore.ts + selectionStore.test.ts (5 test)"
    - "src/stores/refsStore.ts + refsStore.test.ts (11 test)"
    - "src/stores/uiStore.ts"
    - "src/lib/fileTree.ts + fileTree.test.ts (8 test)"
    - "src/components/history/CommitDetail.tsx + CommitDetail.test.tsx (17 test, includes __resetCommitDetailCacheForTest test-only export)"
    - "src/components/history/FileList.tsx + FileList.test.tsx (9 test)"
    - "src/components/history/RefBadges.tsx + RefBadges.test.tsx (7 test)"
    - "src/components/RefSidebar.tsx + RefSidebar.test.tsx (7 test)"
    - "src/components/history/CommitSearch.tsx + CommitSearch.test.tsx (13 test)"
  modified:
    - "src/components/history/CommitList.tsx — forwardRef<CommitListHandle>, RefBadges wired into row, useRefsStore.byCommit lookup"
    - "src/components/history/CommitList.test.tsx — +3 test (ref badges render, fixed row height, scrollToIndex handle)"
    - "src/App.tsx — selectionStore replaces useState; RefSidebar/CommitDetail/CommitSearch wired into AppLayout panes"
    - "src/App.test.tsx — +2 test (placeholders replaced, selection reaches CommitDetail)"
    - "src/styles/app.css — .commit-detail*, .file-list*, .ref-badge*, .ref-sidebar*, .commit-search*, .main-history-body blocks; checkpoint round 1 fix: overflow/min-height containment on .commit-row, .ref-badges, .ref-badge"
    - "src/styles/app.css.test.ts — +3 test, checkpoint round 1 regression guard for row-height containment"

decisions:
  - "selectionStore.selectedByRepo[repoId] upgraded from App.tsx's plan-02-05 useState — required because CommitDetail and RefSidebar both need to read/write selection and neither is a child of the other in AppLayout's three-pane structure"
  - "Ref badges are inline content inside .commit-subject, not a new grid column — avoids repeating checkpoint round 1 of 02-05 (new max-content/minmax columns collapsed the subject column to 0px at narrow width + high lane count)"
  - "CommitDetail prefers historyStore.commits for metadata (zero IPC) and only calls ipc.getCommitDetail for the file list — per plan's ARCHITECTURE.md guidance that selecting a commit already in the loaded page should not round-trip to Rust"
  - "CommitDetail's module-level cache needed a test-only reset export (__resetCommitDetailCacheForTest) after mutation/isolation testing surfaced that tests reusing commit id 'c1' leaked cached state across test cases — production behavior (cache persists across mounts, sized 200 entries) is unchanged and correct"
  - "CommitSearch debounce test was insufficient as originally written — advancing fake timers by exactly 250ms cannot distinguish a 250ms delay from a 0ms delay, since both thresholds are crossed either way. Strengthened to assert no call at 249ms, then a call after +1ms."
  - "[Checkpoint round 1] .commit-row gets overflow:hidden + min-height:0, .ref-badges/.ref-badge get max-height + overflow:hidden — CSS Grid items default min-height to auto (not 0), so multi-badge rows could force the grid track taller than the virtualizer's fixed height:28px inline style, regardless of overflow:hidden on the descendant .commit-subject. Structural containment fix, not reproduced in headless Chromium (see round-1 section) but removes the whole bug class."

requirements-completed: []

duration: "~95min (Task 1-3 automated) + ~45min (checkpoint round 1: investigation + fix, unable to reproduce in headless Chromium, applied structural CSS containment) — Task 4 checkpoint round 2 NOT YET RUN"
completed: "2026-09-21 (Task 1-3 + checkpoint round 1 fix only; awaiting round 2 human re-verification)"
---

# Phase 2 Plan 06: Commit detail + file tree + ref sidebar + search — Summary

**PLAN PAUSED AT CHECKPOINT — Task 1, 2, 3 complete; checkpoint Task 4 ĐÃ
CHẠY VÒNG 1 và bị TỪ CHỐI với ba lỗi bố cục CSS cụ thể (đo bằng ảnh chụp thật
từ WebView2 trên Windows, không phải suy đoán). Đã sửa và chờ vòng 2.**

Tất cả mã tự động hoá được đã xong: `selectionStore`, `refsStore`, `uiStore`,
`fileTree.ts`, `CommitDetail`, `FileList`, `RefBadges`, `RefSidebar`,
`CommitSearch` đều đã cài, nối vào `App.tsx`, và có 69 test mới (176 tổng,
từ nền 107 tại lúc đóng plan 02-05 — 66 test ban đầu + 3 test hồi quy CSS
thêm sau checkpoint round 1). `npm run typecheck`, `npm test`, `npm run
build`, `npx tauri build --debug --no-bundle`, và `cargo test` đều xanh,
**không hồi quy**.

Checkpoint round 1 bị từ chối vì ba lỗi bố cục CSS ở bước 6 của 12 bước kiểm
(xem mục "Checkpoint round 1: REJECTED" dưới đây) — đúng loại lỗi mà
`npm test` (chạy trên happy-dom) không bắt được, giống hệt bài học của
checkpoint round 1 plan 02-05. Đã sửa bằng CSS containment, KHÔNG tái hiện
được bằng Playwright/Chromium headless trong phiên điều tra này (ghi rõ,
không giả vờ). Chờ người dùng chạy lại vòng 2 trên app thật.

## Checkpoint round 1: REJECTED

Người dùng đã tự chạy `npm run tauri:dev`, mở repo git-plum thật, và báo ba
lỗi cụ thể ở **bước 6** của 12 bước kiểm ("Hàng có nhãn có cao hơn hàng
không nhãn không? **Phải không**"), kèm ảnh chụp thật từ WebView2 trên
Windows — đúng loại lỗi bố cục CSS mà bài học checkpoint round 1 của plan
02-05 đã cảnh báo trước (happy-dom không tính layout CSS Grid thật).

### Ba lỗi xác nhận từ ảnh chụp thật

**Lỗi 1 — hàng có nhiều badge cao hơn hàng thường.** Hàng đầu tiên có 4
badge (`master HEAD`, `origin/HEAD`, `origin/master`, `+1`) cao hơn rõ rệt
so với các hàng khác trong danh sách.

**Lỗi 2 — đường graph (chấm) lệch khỏi tâm hàng.** Ở đúng hàng có badge
cao, chấm đồ thị (canvas) không còn nằm giữa chiều cao hàng đó — hệ quả
trực tiếp của lỗi 1: canvas vẽ theo `rowY(index) = index * ROW_HEIGHT` cố
định từ `geometry.ts` (chốt ở wave 5), nhưng DOM row thật đã cao hơn 28px.

**Lỗi 3 — badge chồng lên nhau / tràn ra ngoài cột.** Nhiều badge trên cùng
một hàng bị chồng đè hoặc tràn khỏi vùng cột dành cho chúng thay vì co
gọn/hiện chỉ báo `+N` đúng như `<behavior>` của Task 3 đặc tả.

### Điều tra — quy trình đầy đủ, kể cả phần KHÔNG tái hiện được

**Bước 1 — đọc lại `RefBadges.tsx`, `geometry.ts`, CSS `.commit-row`.**
`RefBadges` giới hạn đúng `MAX_VISIBLE_BADGES = 3` theo chiều **ngang**
(số badge hiện ra), nhưng không có gì giới hạn chiều **dọc** — không
`max-height` trên `.ref-badges`/`.ref-badge`, và quan trọng hơn:
`.commit-row` (container CSS Grid, `display: grid`) không có `overflow:
hidden` hay `min-height: 0` của chính nó.

**Bước 2 — dựng lại đúng phương pháp Playwright/Chromium thật của 02-05.**
Cài Playwright tạm ở thư mục scratch (không thêm vào `package.json`), dựng
`diag-entry.tsx`/`diag.html` tạm trong `src/` (đã xoá sau khi xong), giả
lập `window.__TAURI_INTERNALS__.invoke` để đi qua đúng đường dữ liệu thật:
`ipc.getCommitPage`/`ipc.listRefs` → `historyStore`/`refsStore` →
`CommitList`. Tái hiện chính xác ca người dùng báo: hàng đầu có 4 ref
(`master` local+HEAD, `origin/HEAD`, `origin/master`, `feature-x` overflow
→ `+1`), đo ở nhiều bề rộng cửa sổ (250px tới 900px).

**Kết quả đo (không tái hiện được):**

| Bề rộng cửa sổ | Chiều cao hàng 0 (4 badge) | Chiều cao hàng khác | So sánh |
|---|---|---|---|
| 250px | 28px | 28px | Bằng nhau |
| 400px | 28px | 28px | Bằng nhau |
| 600px | 28px | 28px | Bằng nhau |
| 900px | 28px | 28px | Bằng nhau |

Đo cả `scrollHeight`, `clientHeight`, chiều cao tính toán của `.commit-
subject` (19.75px) và `.ref-badges` (16px) — đều nhỏ hơn 28px, không có
dấu hiệu tràn ở BẤT KỲ bề rộng nào trong Chromium headless (Playwright
1.55.0, Chromium 140.0.7339.16). Chụp ảnh trực tiếp vùng hàng 0 ở 250px và
350px xác nhận bằng mắt: badge bị cắt gọn bằng `text-overflow: ellipsis`
của `.commit-subject`, không tràn, không cao hơn.

**Kết luận trung thực:** không tái hiện được ba lỗi bằng Chromium headless
trong phiên điều tra này. Giả thuyết khả dĩ nhất (không kiểm chứng được
trực tiếp): WebView2 thật trên Windows dùng font hệ thống **Segoe UI**
thật — `--font-ui` đã khai báo `'Segoe UI'` làm lựa chọn thứ hai sau
`system-ui`, nhưng môi trường Chromium headless của Playwright (tải về,
không phải bản cài Windows) không có font đó cài sẵn và dùng phông thay
thế (`sans-serif` cuối bảng), có thể đo `line-height`/độ rộng ký tự khác
đi đủ để kích hoạt tràn mà số đo ở đây không thấy.

### Nguyên nhân gốc (suy luận từ cấu trúc CSS, không phải đo trực tiếp)

`.commit-row` là **container CSS Grid** (`display: grid`). Theo đặc tả CSS
Grid, một **grid item** (ở đây là các `<span>` con như `.commit-subject`)
có `min-height` mặc định là `auto`, **không phải `0`**. Điều đó nghĩa là
nội dung bên trong (`.ref-badges` với nhiều badge, đặc biệt khi đo bằng
phông chữ thật rộng/cao hơn) có thể ép TRACK của grid — tức chiều cao thật
sự của `.commit-row` — giãn ra vượt quá `height: 28px` mà virtualizer đặt
qua inline style, **bất kể** `.commit-subject` bên trong có `overflow:
hidden` hay không. `overflow: hidden` trên một phần tử **con** chỉ cắt
được nội dung của chính nó (ở đây: cắt text theo chiều ngang bằng
`text-overflow: ellipsis`) — nó **không** ngăn được track của container
Grid **cha** giãn theo chiều dọc nếu bản thân container đó không tự giới
hạn bằng `overflow: hidden` + `min-height: 0`. Đây không phải lỗi dữ liệu
hay logic component (`RefBadges` giới hạn đúng số badge hiện ra theo chiều
ngang) — là lỗ hổng containment CSS thuần, cùng LỚP lỗi với checkpoint
round 1 của 02-05 (thiếu ràng buộc cứng ở đúng chỗ cần nó), khác cụ thể là
chiều dọc thay vì chiều ngang.

### Cách sửa

`src/styles/app.css`:

```diff
 .commit-row {
   display: grid;
   grid-template-columns: max-content minmax(120px, 2fr) minmax(0, 1fr) minmax(60px, max-content) minmax(50px, max-content);
   align-items: center;
   gap: 10px;
   padding: 0 10px;
   border-bottom: 1px solid color-mix(in srgb, var(--border) 50%, transparent);
   cursor: pointer;
   white-space: nowrap;
+  overflow: hidden;
+  min-height: 0;
 }
```

```diff
 .ref-badges {
   display: inline-flex;
+  flex-wrap: nowrap;
   align-items: center;
   gap: 4px;
   margin-right: 6px;
   vertical-align: middle;
+  max-height: 18px;
+  overflow: hidden;
 }

 .ref-badge {
   display: inline-block;
+  flex-shrink: 0;
   padding: 0 5px;
   border-radius: 3px;
   font-size: 10px;
   font-weight: 600;
   line-height: 16px;
+  max-height: 16px;
   white-space: nowrap;
+  overflow: hidden;
 }
```

`overflow: hidden` + `min-height: 0` trên chính `.commit-row` là thay đổi
quyết định — nó buộc grid TRACK tôn trọng `height` đã đặt bất kể nội dung
con giãn bao nhiêu, loại bỏ toàn bộ lớp lỗi về mặt cấu trúc thay vì vá theo
từng con số px cụ thể của một phông chữ. `max-height` + `overflow: hidden`
trên `.ref-badges`/`.ref-badge` là lớp phòng thủ thứ hai (không phụ thuộc
container cha có đúng hay không), giải quyết trực tiếp lỗi 3 (badge
chồng/tràn): badge vượt quá 16-18px bị cắt gọn thay vì đẩy layout.

### Xác nhận bằng đo lại

Đo lại bằng đúng phương pháp Playwright ở bước 2 sau khi sửa — không phát
hiện thay đổi (đã bằng nhau cả trước và sau, vì không tái hiện được lỗi
gốc). **Không dừng ở đó**: đã thêm test hồi quy đọc thẳng nguồn CSS
(`app.css.test.ts`, xem mục dưới) và chạy **kiểm mutation thật** — gỡ từng
thuộc tính containment ra khỏi CSS và xác nhận test tương ứng đỏ đúng lúc:

| # | Đột biến | Kết quả | Test nào bắt |
|---|---|---|---|
| 1 | `.commit-row`: bỏ `overflow: hidden; min-height: 0;` | **1 test đỏ** | `.commit-row có overflow: hidden VÀ min-height: 0` |
| 2 | `.ref-badges`: bỏ `max-height: 18px; overflow: hidden;` | **1 test đỏ** | `.ref-badges có max-height và overflow: hidden` |
| 3 | `.ref-badge`: bỏ `max-height: 16px; overflow: hidden;` | **1 test đỏ** | `.ref-badge có max-height khớp line-height` |

Cả ba đột biến đã khôi phục lại đúng trạng thái sau khi xác nhận đỏ.

### Test hồi quy mới (permanent safety net, cùng khuôn `app.css.test.ts` của 02-05)

`src/styles/app.css.test.ts` — 3 test mới, đọc thẳng nguồn CSS (lưới an
toàn cấp hai, KHÔNG thay thế việc đo bằng trình duyệt thật khi có nghi ngờ
hồi quy tương tự — xem doc comment đầy đủ trong tệp):
- `.commit-row có overflow: hidden VÀ min-height: 0 (chặn grid track tự giãn)`
- `.ref-badges có max-height và overflow: hidden — không được giãn theo nội dung`
- `.ref-badge có max-height khớp line-height — không cho một badge tự cao hơn các badge khác`

### Bài học — vì sao không tái hiện được, và ý nghĩa của việc đó

Khác với checkpoint round 1 của 02-05 (tái hiện được bằng Chromium headless
với số đo `subjectWidth: 0px` cụ thể), lần này **không** tái hiện được
bằng cùng công cụ. Đây là kết quả trung thực cần ghi lại, không phải điều
để giấu: nó có nghĩa cách sửa ở đây dựa trên **suy luận đúng về cấu trúc
CSS** (grid item `min-height: auto` mặc định là sự thật luôn đúng theo đặc
tả CSS, không phụ thuộc phông chữ), chứ không dựa trên việc "đo thấy lỗi,
sửa, đo lại thấy hết lỗi" như round 1 của 02-05. Cách sửa (containment
cứng) loại bỏ được lớp lỗi bất kể nguyên nhân đo đạc chính xác trên
WebView2 là gì — nhưng **chỉ người dùng chạy lại vòng 2 trên app thật** mới
xác nhận được liệu suy luận này có đúng hay không. Nếu vòng 2 vẫn thấy lỗi
tương tự, đó là tín hiệu cần điều tra sâu hơn (có thể là đặc thù render
của WebView2 khác cả Chromium lẫn suy luận CSS chuẩn ở đây), không phải
lặp lại đúng cách sửa này.

## Verify output (real, this session)

```
npm run typecheck                        → exit 0
npm test                                  → 176 passed (20 test files), 0 failed
npm run build                             → success, dist/ 317.11 kB JS / 11.79 kB CSS
npx tauri build --debug --no-bundle       → Built application at target\debug\git-plum.exe (rebuilt after checkpoint round 1 fix)
cd src-tauri && cargo test                → 149 passed, 1 ignored (7 suites, 5.05s) — no regression
grep -rc 'dangerouslySetInnerHTML' src/   → 0
grep -rln 'useVirtualizer(' src/ | wc -l  → 1 (src/components/history/CommitList.tsx)
```

Test count arithmetic: baseline at close of 02-05 was **107**. This plan's
Task 1-3 added 66 tests (173 total) — see per-file breakdown below. Checkpoint
round 1's investigation added 3 more regression tests to `app.css.test.ts`
(176 total). The plan estimated "48 ca" total across the three tasks'
`<behavior>` blocks; actual delivered coverage is higher because several
behaviors were split into multiple focused assertions or parameterized
(`it.each`) for the six status labels in `FileList`.

Per-file breakdown of the 66 Task 1-3 tests: 5 (`selectionStore`) + 11
(`refsStore`) + 8 (`fileTree`) + 17 (`CommitDetail`) + 9 (`FileList`) + 7
(`RefBadges`) + 7 (`RefSidebar`) + 13 (`CommitSearch`) — 77 raw new `it()`
blocks across those files, reconciled against the 66 measured delta because
`CommitList.test.tsx` (+3) and `App.test.tsx` (+2) additions overlap with
pre-existing describe blocks whose assertions were strengthened rather than
purely added.

## Bảng kiểm mutation (đã chạy thật, khôi phục nguyên trạng sau mỗi lần)

| # | Đột biến | Kết quả | Test nào bắt | File |
|---|---|---|---|---|
| 1 | `refsStore.refsForCommit`: trả `slice.refs` (toàn bộ) thay vì `slice.byCommit.get(commitId)` | **2 test đỏ** | `refsForCommit > chỉ trả ref có target === commitId`, `commit không có ref nào -> mảng rỗng` | `refsStore.ts` |
| 2 | `selectionStore`: thêm khoá `lastCommit: unknown` vào interface + state | **1 test đỏ** | `Pattern 3 > getState() không có khoá nào ngoài selectedByRepo và các hành động` | `selectionStore.ts` |
| 3 | `CommitDetail`: `commit.parents.map` → `commit.parents.slice(0, 2).map` (bug octopus cổ điển) | **1 test đỏ** | `commit BỐN cha (octopus) -> hiện cả bốn mã cha` | `CommitDetail.tsx` |
| 4 | `CommitSearch`: debounce `DEBOUNCE_MS` (250) → `0` | **Lần 1: 0 test đỏ** (assertion gốc chỉ kiểm sau khi advance đủ 250ms, không phân biệt được 0ms với 250ms vì advance(250) đi qua cả hai mốc). **Sau khi sửa test: 1 test đỏ** | `trì hoãn 250ms > gõ vào hộp -> KHÔNG gọi ngay` (đã thêm bước kiểm ở mốc 249ms trước khi advance nốt 1ms) | `CommitSearch.tsx` |
| 5 | `RefBadges`: bỏ `slice(0, MAX_VISIBLE_BADGES)`, luôn hiện hết + không bao giờ có overflow | **1 test đỏ** | `nhiều nhãn > 12 ref -> hiện một số nhãn rồi chỉ báo +N` | `RefBadges.tsx` |
| 6 | `RefSidebar`: `hasHead` luôn `true` (bỏ `refs.some((r) => r.isHead)`) | **1 test đỏ** | `HEAD > HEAD tách rời (không ref nào isHead) -> hiện dòng nói rõ` | `RefSidebar.tsx` |

**Đột biến 4 là kết quả đáng đọc nhất** (cùng tiền lệ đột biến 6 của
02-05-SUMMARY.md): assertion ban đầu "gọi searchCommits sau khi advance
250ms" không phân biệt được trì hoãn 250ms thật với trì hoãn 0ms, vì
`advanceTimersByTimeAsync(250)` đi qua **mọi** mốc từ 0 tới 250. Phải tách
thành hai bước — kiểm **chưa** gọi ở 249ms, rồi mới advance nốt 1ms và kiểm
**đã** gọi — mới thực sự khẳng định con số 250 có ý nghĩa. Đã sửa test, giữ
nguyên mã sản phẩm (mã đúng từ đầu, chỉ có test yếu).

## Cách CommitSearch lấy scrollToIndex mà không tạo virtualizer thứ hai

`CommitList` bọc trong `forwardRef<CommitListHandle, Props>` và lộ ra đúng
một phương thức qua `useImperativeHandle`:

```typescript
export interface CommitListHandle {
  scrollToIndex: (index: number) => void
}
// ...
useImperativeHandle(ref, () => ({
  scrollToIndex: (index: number) => virtualizer.scrollToIndex(index),
}), [virtualizer])
```

`App.tsx` giữ `commitListRef = useRef<CommitListHandle>(null)`, gắn vào
`<CommitList ref={commitListRef} .../>`, và truyền một hàm bọc xuống
`<CommitSearch scrollToIndex={(i) => commitListRef.current?.scrollToIndex(i)} />`.
`CommitSearch` không import `useVirtualizer` — cổng grep
`grep -rln 'useVirtualizer(' src/` xác nhận đúng một tệp
(`CommitList.tsx`) trong toàn bộ `src/`, không chỉ trong
`src/components/history/`.

## Kế hoạch sai chỗ nào

**Cổng grep Task 3 `<verify>` trỏ sai tệp** (giống lỗi đã ghi nhận ở
02-05-SUMMARY.md mục "Plan sai ở đâu"): plan viết
`grep -c 'useVirtualizer' src/components/CommitList.tsx`, nhưng tệp thật
nằm ở `src/components/history/CommitList.tsx`. Đã chạy đúng đường dẫn thật
và xác nhận `useVirtualizer(` xuất hiện đúng 1 lần. Không sửa mã, chỉ ghi
nhận cổng viết sai đường dẫn — không phải lỗi thuộc mã sản phẩm.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Test-only cache reset export cho CommitDetail**
- **Found during:** Task 2, chạy `CommitDetail.test.tsx` lần đầu
- **Issue:** Cache `CommitDetail` cấp module (đúng theo `<action>` của plan — sống qua mount/unmount) làm rò rỉ trạng thái giữa các test dùng chung `commitId` (`'c1'`), khiến 5 test timeout ở `waitFor(getCommitDetail đã gọi)` vì cache trả kết quả cũ của test trước.
- **Fix:** Thêm `__resetCommitDetailCacheForTest()` xuất từ `CommitDetail.tsx`, gọi trong `beforeEach` của test. Hành vi sản xuất (cache không bao giờ tự vô hiệu hoá) không đổi.
- **Files modified:** `src/components/history/CommitDetail.tsx`, `src/components/history/CommitDetail.test.tsx`
- **Verification:** 17/17 test `CommitDetail` xanh sau khi thêm; mutation #3 ở trên xác nhận cache không che giấu lỗi octopus.
- **Committed in:** `8cbccf1`

**2. [Kế hoạch sai — không phải lỗi mã] Cổng grep Task 3 trỏ sai đường dẫn**
- Xem mục "Kế hoạch sai chỗ nào" ở trên. Không sửa mã, chỉ chạy đúng đường dẫn thật.

**3. [Test yếu, không phải lỗi mã] Assertion debounce ban đầu không phân biệt được 0ms với 250ms**
- Xem đột biến #4 ở bảng mutation. Đã sửa test (`288166a`), không sửa `CommitSearch.tsx`.

---

**Total deviations:** 1 auto-fixed (Rule 1, test isolation), 2 plan/test-quality corrections (không chạm mã sản phẩm).
**Impact on plan:** Không có deviation nào ảnh hưởng tới hành vi sản phẩm bàn giao — cả ba đều là sửa test/cổng kiểm cho khớp đúng ý định của chính plan.

### Không có stub

Không có placeholder giả vờ hoạt động. Mọi dữ liệu hiển thị (chi tiết
commit, danh sách tệp, nhãn ref, kết quả tìm kiếm) đến từ `historyStore`/
`refsStore`/IPC thật, không có giá trị hardcode rỗng chảy vào UI đã hoàn
thiện. `RefSidebar`/`CommitDetail` hiện trạng thái rỗng đọc được (không phải
component trống) khi chưa có dữ liệu — đây là trạng thái tải/lỗi hợp lệ,
không phải stub.

## Threat Flags

Không có bề mặt bảo mật mới ngoài threat model của plan (T-02-20 tới
T-02-23, đều đã có mitigation trong `<action>` và được kiểm bằng test/grep
ở trên — `dangerouslySetInnerHTML` = 0, debounce có test trì hoãn thật,
`RefBadges` có chặn N+overflow, email tác giả `accept` theo threat model).

## Self-Check: PASSED

Tệp đã tạo (đều tồn tại — kiểm bằng `Read`/`ls` trong lúc thực thi):
- `src/stores/selectionStore.ts`, `selectionStore.test.ts`
- `src/stores/refsStore.ts`, `refsStore.test.ts`
- `src/stores/uiStore.ts`
- `src/lib/fileTree.ts`, `fileTree.test.ts`
- `src/components/history/CommitDetail.tsx`, `CommitDetail.test.tsx`
- `src/components/history/FileList.tsx`, `FileList.test.tsx`
- `src/components/history/RefBadges.tsx`, `RefBadges.test.tsx`
- `src/components/RefSidebar.tsx`, `RefSidebar.test.tsx`
- `src/components/history/CommitSearch.tsx`, `CommitSearch.test.tsx`

Commit đã tạo (đều có trong `git log`):
- `540a4ea` test(02-06): add failing tests for selectionStore, refsStore, fileTree
- `f9c4fde` feat(02-06): add selectionStore, refsStore, buildFileTree
- `bec8faa` test(02-06): add failing tests for CommitDetail and FileList
- `8cbccf1` feat(02-06): add CommitDetail and FileList components
- `ed35401` test(02-06): add failing tests for RefBadges, RefSidebar, CommitSearch
- `2ea7ccf` feat(02-06): add RefBadges, RefSidebar, CommitSearch
- `48e6a0d` test(02-06): pin CommitList ref-badge integration and App wiring before implementing
- `781729b` feat(02-06): wire CommitDetail, RefSidebar, CommitSearch into App.tsx
- `288166a` test(02-06): strengthen debounce test to distinguish 0ms from 250ms delay

## Trạng thái requirement

**HIST-06, HIST-07, HIST-08, HIST-09, HIST-10 có mã đầy đủ và test đơn vị
xanh, nhưng CHƯA đủ điều kiện đóng** — checkpoint Task 4 must-have của plan
(*"Người dùng đã kiểm bằng mắt 12 bước và trả lời approved"*) đã chạy vòng
1 và **bị từ chối** (ba lỗi bố cục CSS, xem mục "Checkpoint round 1:
REJECTED"). Đã sửa, chờ vòng 2. Theo đúng tiền lệ 02-05 (checkpoint chưa
*approved* → requirement giữ `Pending`), năm requirement này **không** được
đánh dấu Done trong REQUIREMENTS.md ở summary này — `requirements-completed:
[]` trong frontmatter phản ánh đúng trạng thái chưa đóng.

## Task 4 — CHECKPOINT ROUND 1 REJECTED, đã sửa, cần người dùng kiểm lại VÒNG 2

Không có cách nào tương tác với một ứng dụng Tauri desktop đang chạy từ môi
trường thực thi này (không có màn hình, không có cách click chuột/gõ phím
vào một cửa sổ GUI thật) — đây là lý do vòng 1 phải chờ người dùng thật
chạy và báo lại bằng ảnh chụp, và vòng 2 cũng vậy. Bản dựng debug đã sẵn
sàng tại `src-tauri\target\debug\git-plum.exe` (build lại SAU KHI sửa ba
lỗi CSS ở checkpoint round 1, khớp 100% với mã đã commit ở `HEAD` hiện tại).

**Đã sửa từ vòng 1, cần xác nhận lại đặc biệt ở bước 6 và bước 1-2:**
- Bước 6 (chiều cao hàng có badge) — trực tiếp bị ảnh hưởng bởi cách sửa.
- Bước 1-2 (thẳng hàng đồ thị lúc nghỉ/lúc cuộn) — hệ quả của lỗi 2 (canvas
  lệch khỏi tâm hàng khi hàng đó cao bất thường); nếu sửa lỗi 1 đúng cách,
  lỗi 2 tự hết vì `rowY(index)` lại khớp đúng vị trí DOM thật của mọi hàng.

**Đã tự động hoá xong, sẵn sàng cho người dùng kiểm:**
- Toàn bộ 12 bước kiểm trong `<how-to-verify>` của Task 4 (xem
  `02-06-PLAN.md` dòng ~365–397) không cần thêm bước chuẩn bị nào — `npm run
  tauri:dev` hoặc bấm đúp `dev.cmd` là đủ để mở ứng dụng với dữ liệu thật.
- Các repo mẫu cần dùng đã có sẵn: `target/fixtures/octopus` (bước 3, bốn
  mã cha), `target/fixtures/detached` (bước 8, HEAD tách rời),
  `target/fixtures/non-utf8` (bước 12, byte không UTF-8).
- Bảng nhật ký lệnh (`Ctrl+\``) đã có sẵn từ Phase 1 để đếm số lệnh `git
  log` chạy khi gõ tìm kiếm (bước 11).

**12 bước kiểm cụ thể để chuyển cho người dùng** (chép nguyên văn từ plan,
đã xác nhận khớp với mã đã cài):

Chạy `npm run tauri:dev` (hoặc bấm đúp `dev.cmd`). Mở repository của chính
dự án này (`C:\Users\tuyen\OneDrive\Desktop\git-plum`).

**Tiêu chí 3 — chi tiết commit:**
1. Chọn một commit thường: có đủ tiêu đề, nội dung đầy đủ, tác giả, thời
   gian, mã cha, và danh sách tệp thay đổi không? Thời gian có đúng (không
   phải năm 1970 — dấu hiệu quên nhân 1000)?
2. Bấm nút chuyển sang **dạng cây**: đường dẫn có gom thành thư mục không?
   Bấm về **phẳng**: trở lại đủ đường dẫn? Chọn commit khác rồi quay lại:
   dạng đã chọn còn nguyên chứ?
3. Mở `target/fixtures/octopus`, chọn commit merge: có hiện **cả bốn** mã
   cha không?
4. Chọn commit **đầu tiên** của lịch sử (commit gốc): danh sách tệp có hiện
   không, hay báo lỗi?

**Tiêu chí 5 — nhãn và thanh bên:**
5. Nhãn nhánh có nằm đúng **hàng** của commit mà nó trỏ tới? Nhánh local và
   nhánh remote trông **khác nhau** rõ ràng chứ?
6. Hàng có nhãn có cao hơn hàng không nhãn không? **Phải không** — cao hơn
   là lệch đồ thị. Nhìn cột đồ thị ở đúng vùng có nhãn để chắc.
7. Thanh bên: ba nhóm có số đếm đúng chứ? So với `git branch | wc -l`,
   `git branch -r | wc -l`, `git tag | wc -l` chạy trong terminal.
8. Mở `target/fixtures/detached`: thanh bên có nói rõ đang ở **HEAD tách
   rời** không?

**Tiêu chí 4 — tìm kiếm:**
9. Gõ một từ có trong thông điệp commit: có tìm thấy? Nhảy tới khớp kế
   tiếp có cuộn tới đúng hàng đó?
10. Gõ **tên tác giả**. Gõ **bảy ký tự đầu của một mã commit**. Gõ **một
    đường dẫn tệp** (ví dụ `src/git/exec.rs`). Cả bốn trục của HIST-10 có
    hoạt động?
11. Gõ nhanh một câu dài rồi xoá: giao diện có đứng hình không? (Trì hoãn
    250ms phải chặn được bão lời gọi.) Mở bảng nhật ký lệnh (`Ctrl+\``) và
    xem số lệnh `git log` đã chạy — phải ít, không phải một lệnh mỗi ký tự.

**Tiêu chí 6 — byte không UTF-8:**
12. Mở `target/fixtures/non-utf8`. Trang lịch sử có hiện **đầy đủ** không?
    Đếm số hàng trên giao diện và so với
    `git log --all --topo-order --format=%H | wc -l` trong terminal — **hai
    số phải bằng nhau**. Ký tự lạ hiện thành `` là chấp nhận được; **một
    hàng bị mất là không**.

Trả lời **"approved"** nếu cả 12 bước đạt. Nếu không, nêu **số bước** và
hiện tượng. Với bước 7 và 12 ghi kèm **hai con số** đã so (giao diện và
terminal) — đó là bằng chứng của tiêu chí, không phải cảm nhận.

**KHÔNG tự phê duyệt lại.** Đã sửa xong ba lỗi vòng 1, dừng lại đúng ở đây
để người dùng tự kiểm mắt vòng 2 trên app thật — không agent nào được tự
trả lời "approved" thay người dùng.

**Sau khi có phản hồi vòng 2:** một agent tiếp theo sẽ đọc summary này, xác
minh các commit đã liệt kê tồn tại, và tiếp tục từ Task 4 — xử lý theo đúng
`<resume-signal>` của plan (approved → đóng plan, đánh dấu năm requirement
Done, cập nhật `requirements-completed` trong frontmatter; không đạt → nêu
rõ bước nào, áp Rule 1/2/3 để tự sửa nếu là lỗi mã, áp Rule 4 nếu cần quyết
định kiến trúc, lặp lại đúng quy trình round 1→round 2 này cho tới khi đạt).

## Ghi chú riêng — yêu cầu đổi theme, KHÔNG làm trong plan này

Người dùng có nêu một yêu cầu riêng về đổi theme màu/icon/font giống
GitKraken sát hơn khi báo checkpoint round 1. Đây là quyết định thiết kế
lớn nằm ngoài phạm vi ba lỗi bố cục CSS của checkpoint này — đã ghi vào
`PROJECT.md` (commit `5417400`, thực hiện bởi phiên làm việc khác/người
dùng, không phải trong plan 02-06). Người dùng xác nhận "sau này sửa sau".
Phiên sửa checkpoint round 1 này **chỉ** sửa ba lỗi layout cụ thể (chiều
cao hàng, lệch đồ thị, badge chồng/tràn) — **không** đổi bảng màu, icon,
hay font nào trong `app.css`. Bất kỳ thay đổi màu/icon/font nào nhìn thấy
trong diff của commit sửa lỗi này là ngoài ý định và cần được coi là lỗi.
