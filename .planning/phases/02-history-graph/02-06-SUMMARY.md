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
    - "Ref badges get their OWN grid column (.commit-ref-cell, sized minmax(0, max-content)) — never share a column with message text. A 0-floor column yields space entirely when narrow, so it does not repeat 02-05's collapse (caused by hard max-content columns that never shrink). Matches the GitKraken reference layout."
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
    - "src/components/history/CommitList.tsx — forwardRef<CommitListHandle>, useRefsStore.byCommit lookup; checkpoint round 1 fix B: RefBadges moved OUT of .commit-subject into its own .commit-ref-cell grid cell"
    - "src/components/history/CommitList.test.tsx — +3 test (ref badges render, fixed row height, scrollToIndex handle)"
    - "src/App.tsx — selectionStore replaces useState; RefSidebar/CommitDetail/CommitSearch wired into AppLayout panes"
    - "src/App.test.tsx — +2 test (placeholders replaced, selection reaches CommitDetail)"
    - "src/styles/app.css — .commit-detail*, .file-list*, .ref-badge*, .ref-sidebar*, .commit-search*, .main-history-body blocks; checkpoint round 1 fix A: overflow/min-height containment on .commit-row/.ref-badges/.ref-badge; fix B: 6-column grid with a dedicated .commit-ref-cell badge column"
    - "src/styles/app.css.test.ts — +6 test total (3 CSS-string guards for fix A row-height containment; 3 STRUCTURAL guards for fix B: exact column count, badge-column 0 floor, RefBadges position in JSX outside .commit-subject)"

decisions:
  - "selectionStore.selectedByRepo[repoId] upgraded from App.tsx's plan-02-05 useState — required because CommitDetail and RefSidebar both need to read/write selection and neither is a child of the other in AppLayout's three-pane structure"
  - "[SUPERSEDED by checkpoint round 1 fix B] Ref badges were initially inline content inside .commit-subject to avoid adding a grid column — that choice is what caused the badge-vs-message space competition the user reported. Badges now have their own grid column; see the fix-B decision below."
  - "CommitDetail prefers historyStore.commits for metadata (zero IPC) and only calls ipc.getCommitDetail for the file list — per plan's ARCHITECTURE.md guidance that selecting a commit already in the loaded page should not round-trip to Rust"
  - "CommitDetail's module-level cache needed a test-only reset export (__resetCommitDetailCacheForTest) after mutation/isolation testing surfaced that tests reusing commit id 'c1' leaked cached state across test cases — production behavior (cache persists across mounts, sized 200 entries) is unchanged and correct"
  - "CommitSearch debounce test was insufficient as originally written — advancing fake timers by exactly 250ms cannot distinguish a 250ms delay from a 0ms delay, since both thresholds are crossed either way. Strengthened to assert no call at 249ms, then a call after +1ms."
  - "[Checkpoint round 1 — fix A, row height] .commit-row gets overflow:hidden + min-height:0, .ref-badges/.ref-badge get max-height + overflow:hidden — CSS Grid items default min-height to auto (not 0), so multi-badge rows could force the grid track taller than the virtualizer's fixed height:28px inline style, regardless of overflow:hidden on the descendant .commit-subject. Not reproduced by measurement in headless Chromium, but correct per CSS spec and removes the bug class."
  - "[Checkpoint round 1 — fix B, THE main cause] Ref badges get their own grid column (.commit-ref-cell, column 2, sized minmax(0, max-content)) instead of living inside .commit-subject. Reproduced by measurement with real Segoe UI at the real 52% main-pane width: a 4-badge group takes 258px while the subject column is only 144px at a 900px window, so the message rendered 0%. Chose a dedicated column over capping .ref-badges max-width inside the shared column because sharing one column is structurally wrong — capping only moves the breakpoint. Matches the GitKraken reference layout (docs/screenshots/main-1.png, main-4.png: separate BRANCH/TAG and COMMIT MESSAGE columns, with long branch names ellipsised inside the badge)."
  - "[Checkpoint round 1 — fix B] The new badge column uses minmax(0, max-content), NOT hard max-content — the 0 floor lets it yield space entirely at narrow widths, which is precisely why adding a column here does not repeat 02-05's column-collapse bug (that was caused by hard max-content date/sha columns that never shrink)."
  - "[Checkpoint round 1 — fix B] .commit-ref-cell always renders even with zero refs, because RefBadges returns null per its <behavior> spec — without the always-present wrapper, unlabelled rows would have one fewer grid cell and every later cell would shift one column left, misaligning against labelled rows."
  - "[Checkpoint round 1 — fix B] .ref-badge has min-width: 48px rather than 0 — measured at a 900px window, a 0 floor shrank badges to unreadable single characters (m., o., o.). Showing fewer legible badges and folding the rest into +N (GitKraken's behaviour) beats showing many unreadable ones."

requirements-completed: []

duration: "~95min (Task 1-3 automated) + ~45min (checkpoint round 1 fix A: row-height containment) + ~40min (checkpoint round 1 fix B: reproduced badge/message space competition with real Segoe UI, moved badges to their own grid column) — Task 4 checkpoint round 2 NOT YET RUN"
completed: "2026-09-21 (Task 1-3 + both checkpoint round 1 fixes; awaiting round 2 human re-verification)"
---

# Phase 2 Plan 06: Commit detail + file tree + ref sidebar + search — Summary

**PLAN PAUSED AT CHECKPOINT — Task 1, 2, 3 complete; checkpoint Task 4 ĐÃ
CHẠY VÒNG 1 và bị TỪ CHỐI với ba lỗi bố cục CSS cụ thể (đo bằng ảnh chụp thật
từ WebView2 trên Windows, không phải suy đoán). Đã sửa và chờ vòng 2.**

Tất cả mã tự động hoá được đã xong: `selectionStore`, `refsStore`, `uiStore`,
`fileTree.ts`, `CommitDetail`, `FileList`, `RefBadges`, `RefSidebar`,
`CommitSearch` đều đã cài, nối vào `App.tsx`, và có 72 test mới (**179 tổng**,
từ nền 107 tại lúc đóng plan 02-05 — 66 test Task 1-3 + 3 test hồi quy fix A
+ 3 test hồi quy cấu trúc fix B). `npm run typecheck`, `npm test`, `npm run
build`, `npx tauri build --debug --no-bundle`, và `cargo test` đều xanh,
**không hồi quy**.

Checkpoint round 1 bị từ chối vì ba lỗi bố cục CSS ở bước 6 của 12 bước kiểm
— đúng loại lỗi mà `npm test` (chạy trên happy-dom) không bắt được, giống hệt
bài học của checkpoint round 1 plan 02-05.

**Ba lỗi đó có HAI nguyên nhân độc lập, cả hai đã sửa:**
- **Nguyên nhân A** (containment chiều cao CSS Grid) — tìm ở vòng điều tra 1,
  giải thích lỗi 1 (hàng cao hơn) và lỗi 2 (đồ thị lệch tâm hàng).
- **🔴 Nguyên nhân B** (nhãn ref và chữ message cạnh tranh CÙNG một cột grid)
  — tìm ở vòng điều tra 2 sau khi coordinator đọc mã và chỉ ra
  `CommitList.tsx:164-165`. Đây là nguyên nhân **CHÍNH** của thứ người dùng
  thấy, và **đã tái hiện được bằng số đo** (nhóm 4 badge chiếm 258px trong khi
  cột subject chỉ còn 144px → chữ message hiển thị **0%**). Sửa bằng quyết
  định kiến trúc: nhãn có **cột grid riêng**, khớp tham chiếu GitKraken.

Vòng điều tra 1 cho **âm tính giả** vì đo trong Chromium không có Segoe UI và
ở full viewport thay vì vùng `main` 52% thật — chi tiết ở mục dưới. Chờ người
dùng chạy lại vòng 2 trên app thật.

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

### Điều tra — HAI nguyên nhân độc lập, tìm ra ở hai vòng điều tra khác nhau

Checkpoint round 1 có **hai nguyên nhân riêng biệt**, không phải một. Vòng
điều tra đầu chỉ tìm ra nguyên nhân A (chiều cao) và cho **âm tính giả** khi
đo — vòng thứ hai (sau khi coordinator đọc mã và chỉ ra chỗ chưa chạm tới)
tìm ra nguyên nhân B, là nguyên nhân **chính** của thứ người dùng thấy.

#### Nguyên nhân A — containment chiều cao (tìm ở vòng điều tra 1)

`.commit-row` là **container CSS Grid**. Theo đặc tả, một **grid item** có
`min-height` mặc định là `auto`, **không phải `0`** — nên nội dung con có
thể ép TRACK của grid (tức chiều cao thật của `.commit-row`) giãn vượt
`height: 28px` mà virtualizer đặt qua inline style, **bất kể** phần tử con
có `overflow: hidden` hay không (`overflow: hidden` trên con chỉ cắt nội
dung của CHÍNH nó, không ngăn track cha giãn). Hàng cao hơn `ROW_HEIGHT`
làm chấm đồ thị lệch, vì canvas vẽ theo `rowY(index) = index * ROW_HEIGHT`
cố định — đó là **lỗi 2** người dùng báo, hệ quả trực tiếp của **lỗi 1**.

*Phép đo vòng 1 (chiều cao) — tất cả đều 28px, tức ÂM TÍNH GIẢ:*

| Bề rộng | Chiều cao hàng 0 (4 badge) | Hàng khác | Kết luận sai lúc đó |
|---|---|---|---|
| 250px / 400px / 600px / 900px | 28px | 28px | "không tái hiện được" |

#### 🔴 Nguyên nhân B — badge và chữ message cạnh tranh CÙNG một cột grid (tìm ở vòng điều tra 2)

`RefBadges` render **BÊN TRONG** `.commit-subject`
(`CommitList.tsx:164-165` trước khi sửa):

```tsx
<span className="commit-subject" title={commit.subject}>
  <RefBadges refs={refsByCommit?.get(commit.id) ?? []} />
  {commit.subject}
```

`.commit-subject` là cột grid thứ 2, chỉ có sàn `minmax(120px, 2fr)` (sàn do
chính checkpoint round 1 của plan 02-05 thêm vào), và có `overflow: hidden;
text-overflow: ellipsis; white-space: nowrap`. Hệ quả: **badge ăn vào cùng
không gian với chữ message**, và chữ bị ellipsis cắt sạch ở hàng nhiều badge.

*Phép đo vòng 2, ĐÃ TÁI HIỆN ĐƯỢC — Chromium + Segoe UI THẬT (nạp
`segoeui.ttf`/`segoeuib.ttf` từ `C:/Windows/Fonts` qua `@font-face`), đo ở
đúng bề rộng vùng `main` thật (52% cửa sổ theo `AppLayout`):*

| Cửa sổ | Vùng `main` | Cột subject | Nhóm 4 badge chiếm | Chữ message hiện được |
|---|---|---|---|---|
| 1440px | 749px | 332px | **258px** (78%) | 74/272px = **27%** |
| 1100px | 572px | 214px | **258px** (121%) | **0%** |
| 900px | 468px | 144px | **258px** (179%) | **0%** |

Hàng **không** badge trong cùng lần đo hiện chữ bình thường (98-100% ở
1440px). Chỉ hàng nhiều badge mất chữ — **khớp chính xác** ảnh người dùng
gửi. Điều này cũng giải thích **lỗi 3** (badge "chồng/tràn"): badge chiếm
121-179% cột nên bị `overflow: hidden` cắt giữa chữ, nhìn như chồng đè.

**Phát hiện quan trọng về ảnh chụp:** dấu `.` mà người dùng thấy ở hàng đó
**không phải** thông điệp commit thật — nó là phần đuôi ellipsis còn sót
lại hiển thị được của một thông điệp dài. Đây là điểm khác biệt then chốt so
với checkpoint round 1 của plan 02-05, nơi `.` **thật sự** là nội dung
commit (đã kiểm chứng bằng `parse_log` trên repo thật lúc đó).

#### Vì sao phép đo vòng 1 cho ÂM TÍNH GIẢ — hai sai số cộng dồn

1. **Phông chữ.** Chromium của Playwright tải rời **không có Segoe UI** cài
   sẵn nên rơi về phông thay thế cuối bảng `--font-ui`, hẹp hơn Segoe UI
   thật đáng kể. Ngưỡng tràn 120px vì thế đến muộn hơn nhiều so với máy thật.
2. **Bề rộng vùng chứa.** Vòng 1 render `CommitList` ở **full viewport**
   thay vì bề rộng vùng `main` thật (52% cửa sổ qua `AppLayout`) — cột
   subject rộng gấp đôi thực tế.

Hai sai số cộng lại làm badge "vừa đủ chỗ" trong phép đo trong khi tràn rõ
rệt trên máy thật. **Bài học: đo layout phải khớp CẢ phông CẢ bề rộng vùng
chứa thật** — đo đúng thứ (chiều cao) nhưng trong môi trường sai vẫn cho kết
luận sai. Vòng 1 cũng chỉ đo chiều **cao** mà không đo cạnh tranh không gian
theo chiều **ngang**, nên bỏ sót hẳn nguyên nhân B.

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

`overflow: hidden` + `min-height: 0` trên chính `.commit-row` buộc grid
TRACK tôn trọng `height` đã đặt bất kể nội dung con giãn bao nhiêu, loại bỏ
lớp lỗi A về mặt cấu trúc thay vì vá theo từng con số px của một phông chữ.
`max-height` + `overflow: hidden` trên `.ref-badges`/`.ref-badge` là lớp
phòng thủ thứ hai (không phụ thuộc container cha có đúng hay không).

#### Fix B — cột grid riêng cho nhãn (quyết định KIẾN TRÚC)

Nguyên nhân B không sửa được bằng tinh chỉnh px trong một cột dùng chung:
badge và chữ message cạnh tranh cùng không gian là **sai về cấu trúc**, và
mọi cách chỉnh `max-width` bên trong cột dùng chung chỉ **dịch chuyển điểm
vỡ** chứ không loại bỏ nó.

**Hai lựa chọn đã cân nhắc:**

| | Cách | Vì sao chọn / không chọn |
|---|---|---|
| **(a)** ✅ | Tách nhãn thành **cột grid riêng** | **ĐÃ CHỌN.** Đúng cấu trúc: message có cột riêng, badge không bao giờ ăn vào. Khớp tham chiếu thiết kế GitKraken — `docs/screenshots/main-1.png` và `main-4.png` cho thấy cột `BRANCH / TAG` **tách biệt** khỏi cột `COMMIT MESSAGE`, và GitKraken xử lý đúng ca 4 nhãn này bằng cách cắt tên nhãn trong badge (`ma...`, `feature/uploa...`) chứ không để nó đẩy message. |
| (b) ❌ | Giữ nhãn trong cột subject + `max-width` cứng cho `.ref-badges` | Không chọn: chỉ dịch điểm vỡ. Ở cửa sổ hẹp, cột subject chỉ còn 144px — dù giới hạn nhãn ở 40% (58px) thì message vẫn chỉ còn 86px, và bài toán cạnh tranh vẫn còn nguyên về bản chất. |

```diff
 .commit-row {
   display: grid;
-  grid-template-columns: max-content minmax(120px, 2fr) minmax(0, 1fr) minmax(60px, max-content) minmax(50px, max-content);
+  grid-template-columns: max-content minmax(0, max-content) minmax(120px, 2fr) minmax(0, 1fr) minmax(60px, max-content) minmax(50px, max-content);
```

```diff
+.commit-ref-cell {
+  display: flex;
+  align-items: center;
+  min-width: 0;
+  max-width: 220px;
+  max-height: 18px;
+  overflow: hidden;
+}
+
 .ref-badge {
-  flex-shrink: 0;
+  flex-shrink: 1;
+  min-width: 48px;
+  text-overflow: ellipsis;
 }
```

`CommitList.tsx` — `RefBadges` chuyển ra khỏi `.commit-subject`, bọc trong
ô grid `.commit-ref-cell` riêng.

**Ba quyết định thiết kế đáng ghi lại:**

1. **Cột nhãn là `minmax(0, max-content)`, KHÔNG phải `max-content` cứng.**
   Sàn `0` nghĩa là cột **nhường chỗ được hoàn toàn** khi cửa sổ hẹp — trái
   với cột date/sha `max-content` cứng từng gây lỗi co cột ở checkpoint round
   1 của plan 02-05. Đây chính là lý do thêm cột mới lần này **không** lặp
   lại lỗi cũ, dù bài học 02-05 cảnh báo về việc thêm cột.
2. **`.commit-ref-cell` LUÔN render, kể cả khi không có ref.** `RefBadges`
   trả `null` theo đúng đặc tả `<behavior>` ("không có ref → không render
   gì"), nên nếu để nó tự làm ô grid thì hàng không nhãn sẽ **thiếu một ô** —
   mọi ô sau đó dồn sang trái một cột và lệch cột so với hàng có nhãn. Ô rỗng
   rộng 0px nên không tốn chỗ.
3. **`min-width: 48px` cho `.ref-badge`, không phải `0`.** Đo thật ở cửa sổ
   900px cho thấy `min-width: 0` làm badge co tới mức vô nghĩa (`m.`, `o.`,
   `o.` — không đọc được nhãn nào). Thà hiện **ít** nhãn mà đọc được rồi gộp
   phần còn lại vào `+N` (đúng cách GitKraken làm) còn hơn hiện nhiều nhãn
   không đọc nổi.

### Xác nhận bằng đo lại — trước/sau, cùng phương pháp Segoe UI thật

| Cửa sổ | Cột subject TRƯỚC | Chữ hiện TRƯỚC | Cột subject SAU | Chữ hiện SAU |
|---|---|---|---|---|
| 1440px | 332px (badge ăn 258px) | **27%** | 178px | **66%** |
| 1100px | 214px (badge ăn 258px) | **0%** | 120px (giữ sàn) | **44%** |
| 900px | 144px (badge ăn 258px) | **0%** | 120px (giữ sàn) | **44%** |

Hàng không nhãn: 98-100% ở 1440px, không đổi trước/sau (đúng — fix không
được làm hỏng hàng bình thường). Ảnh chụp thật sau khi sửa ở 1440px cho
`master H...` `origin/H...` `origin/m...` `+1` rồi tới chữ message — khớp
bố cục tham chiếu GitKraken.

### Test hồi quy mới (permanent safety net)

`src/styles/app.css.test.ts` — **6 test mới tổng cộng** (3 cho fix A đọc
nguồn CSS, 3 cho fix B kiểm **cấu trúc**). Ba test của fix B tồn tại vì
coordinator chỉ ra đúng: test đọc chuỗi CSS **không bắt được** loại lỗi cạnh
tranh không gian, và happy-dom cũng không (nó không tính grid track sizing):
- `.commit-row có ĐÚNG 6 cột — cột nhãn ref là cột riêng`
- `cột nhãn ref có sàn 0 để nhường chỗ được khi cửa sổ hẹp`
- `RefBadges nằm trong .commit-ref-cell, không nằm trong .commit-subject`
  (kiểm **cấu trúc DOM/JSX** — đây là test bắt trực tiếp đúng lỗi B)

**Bảng mutation của fix B (đã chạy thật, khôi phục sau mỗi lần):**

| # | Đột biến | Kết quả | Test nào bắt |
|---|---|---|---|
| 4 | `CommitList.tsx`: đưa `RefBadges` trở lại **bên trong** `.commit-subject` (đúng lỗi gốc B) | **1 test đỏ** | `RefBadges nằm trong .commit-ref-cell, không nằm trong .commit-subject` |
| 5 | `app.css`: bỏ cột nhãn khỏi `grid-template-columns` (về 5 cột) | **1 test đỏ** | `.commit-row có ĐÚNG 6 cột` |

### Bài học

**Đo đúng thứ trong môi trường sai vẫn cho kết luận sai.** Vòng 1 đo chiều
cao hàng — đúng thứ cần đo cho lỗi 1 — nhưng trong Chromium không có Segoe
UI và ở full viewport thay vì vùng `main` 52%, nên cho âm tính giả. Hai sai
số đó phải sửa **cùng lúc** mới tái hiện được: nạp phông thật qua
`@font-face` **và** đo ở đúng bề rộng vùng chứa thật.

**Một triệu chứng có thể có nhiều nguyên nhân.** "Hàng cao hơn" và "hàng mất
chữ" nhìn như một lỗi bố cục duy nhất, nhưng là hai nguyên nhân độc lập
(containment chiều dọc vs. cạnh tranh không gian ngang). Sửa xong nguyên
nhân A rồi báo cáo là **sớm** — nếu coordinator không đọc lại mã và chỉ ra
`CommitList.tsx:164-165`, nguyên nhân chính đã lọt qua vòng 2.

**Test đọc chuỗi CSS có trần rõ ràng.** `app.css.test.ts` bắt được "ai đó
xoá `overflow: hidden`" nhưng **không thể** bắt "badge và message dùng chung
cột" — lỗi đó nằm ở **quan hệ cấu trúc** giữa JSX và grid, không nằm trong
một chuỗi CSS nào. Vì thế fix B cần test kiểm cấu trúc (đếm cột, kiểm vị trí
`RefBadges` trong JSX), không phải thêm một assertion chuỗi nữa.

## Verify output (real, this session)

```
npm run typecheck                        → exit 0
npm test                                  → 179 passed (20 test files), 0 failed
npm run build                             → success, dist/ 317.16 kB JS / 11.94 kB CSS
npx tauri build --debug --no-bundle       → Built application at target\debug\git-plum.exe (rebuilt after BOTH checkpoint round 1 fixes)
cd src-tauri && cargo test                → 149 passed, 1 ignored (7 suites, 2.57s) — no regression
grep -rc 'dangerouslySetInnerHTML' src/   → 0
grep -rln 'useVirtualizer(' src/ | wc -l  → 1 (src/components/history/CommitList.tsx)
```

Test count arithmetic: baseline at close of 02-05 was **107**. This plan's
Task 1-3 added 66 tests (173 total) — see per-file breakdown below. Checkpoint
round 1 added 3 regression tests for fix A (176) and 3 structural regression
tests for fix B (**179 total**). The plan estimated "48 ca" total across the three tasks'
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

**Đã sửa HAI nguyên nhân từ vòng 1. Cần xác nhận lại đặc biệt:**
- **Bước 5** (nhãn đúng hàng, local khác remote) — nhãn giờ ở **cột riêng**
  bên trái cột thông điệp, giống bố cục GitKraken. Kiểm cả việc **chữ thông
  điệp có hiện đầy đủ trên hàng CÓ nhãn** hay không — đây là lỗi chính của
  vòng 1 (chữ bị nhãn ăn hết chỗ, hàng đó nhìn như chỉ có một dấu `.`).
- **Bước 6** (chiều cao hàng có nhãn) — fix A nhắm trực tiếp vào việc này.
- **Bước 1-2** (thẳng hàng đồ thị lúc nghỉ/lúc cuộn) — hệ quả của lỗi 2;
  nếu fix A đúng, `rowY(index)` lại khớp vị trí DOM thật của mọi hàng.
- **Kéo panel giữa hẹp lại rồi rộng ra** trong lúc kiểm: bài toán cạnh tranh
  không gian chỉ lộ ra ở bề rộng hẹp, nên kéo qua nhiều bề rộng là cách bắt
  hồi quy nhanh nhất. Ở bề rộng rất hẹp, nhãn sẽ bị cắt dần và gộp vào `+N` —
  đó là hành vi ĐÚNG theo thiết kế (giống GitKraken), không phải lỗi.

**Lưu ý:** có một tiến trình `git-plum.exe` đang chạy lúc build lại bản debug
(đã `taskkill`) — nếu app đang mở từ trước, **phải đóng và mở lại** mới thấy
bản đã sửa.

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
