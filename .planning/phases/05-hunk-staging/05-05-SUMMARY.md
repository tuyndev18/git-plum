---
phase: 05-hunk-staging
plan: 05
subsystem: hunk-staging-ui
tags: [WORK-03, WORK-04, WORK-05, WORK-06, WORK-07, duong-vao-thu-hai, od-c, vong-khoa-chet]
requires:
  - "05-02 (stage_hunk / unstage_hunk)"
  - "05-03 (discard_hunk / discard_files / list_trash / restore_trash)"
  - "05-04 (hunkStore, HunkBar, onLamMoi)"
  - "04-02 (statusStore.applyExternal)"
provides:
  - "scripts/verify-work04-bytes.sh — cổng MÁY cho tiêu chí thành công 2"
  - "components/worktree/HunkTable — bảng khối, KHÔNG qua CodeMirror/Phase 3"
  - "components/worktree/TrashList — Vừa huỷ gần đây, WORK-07"
  - "components/worktree/WorktreePane — ĐƯỜNG VÀO, không qua component Phase 4"
  - "App.tsx: công tắc `Thay đổi` ở thanh công cụ"
affects:
  - "phase sau: gộp hai danh sách tệp khi Phase 4 có bằng chứng mắt người"
tech-stack:
  added: []
key-files:
  created:
    - "scripts/verify-work04-bytes.sh (439 dòng)"
    - "src/components/worktree/HunkTable.tsx + .test.tsx (14 test)"
    - "src/components/worktree/TrashList.tsx + .test.tsx (9 test)"
    - "src/components/worktree/WorktreePane.tsx + .test.tsx (7 test)"
    - "docs/10-phase5-dogfood.md (CHƯA CHẠY)"
    - ".planning/phases/05-hunk-staging/VERIFICATION.md"
  modified:
    - "src/App.tsx (+~70 dòng: công tắc + nhánh render)"
    - "src/styles/app.css (+~260 dòng, kèm khối CHƯA KIỂM)"
decisions:
  - "Plan SAI ở ba chỗ: DiffViewer là diff của COMMIT không phải worktree; nút trong DiffToolbar dựng lại vòng khoá NĂM điều kiện; getWorktreeDiff chưa có chỗ gọi nào"
  - "Phép kiểm (0) của cổng byte KHÔNG có trong plan, và MG2b chứng minh thiếu nó thì cổng là XANH GIẢ"
  - "M28 bản đầu VÔ HIỆU: happy-dom không lan `fieldset disabled` — đo bằng test thăm dò"
  - "luc của MucThungRac lấy lại được TỪ refName, không cần chỗ lưu phụ"
metrics:
  duration: "~2h"
  completed: "2026-09-23"
---

# 05-05 — Đường vào thứ hai, cổng byte bằng máy, và hai cổng người kiểm CHƯA CHẠY

Wave 4 đóng được *"`HunkBar` **có thể** sống không cần Phase 4"* và ghi thẳng rằng
*"người dùng **có** đường tới nó"* thì **chưa** — kèm cảnh báo rằng nếu đường vào duy
nhất là "mở diff từ `ChangeList`" thì *"toàn bộ công của wave này **không mua được
gì**"*. Wave này đóng mệnh đề hai, và trên đường làm việc đó đã **đo được rằng ba điều
plan viết là sai**.

---

## Số đo — lệnh đầy đủ, tôi tự chạy

| Phép đo | Trước | Sau | Lệnh |
|---|---:|---:|---|
| Frontend | **703 passed, 0 failed** | **731 passed, 0 failed** (+28) | `npx vitest run --reporter=json --outputFile=.vitest/json/final05.json` |
| Bộ test | 242 | **257** | cùng lệnh |
| `npx tsc --noEmit` | sạch | **sạch** (`exit=0`) | `npx tsc --noEmit` |
| `cargo test --lib` | 350 | **350 passed, 0 failed** | `cargo test --lib` |
| `cargo test --lib --tests` | 481 passed, 1 ignored | **481 passed, 1 ignored** | `cargo test --lib --tests` |
| `npm run build` | 645 kB | **✓ 403ms**, 656,66 kB | `npm run build` |
| `npm run tauri build` | — | **✓ 2m 47s** + msi | `npm run tauri build` |
| Bản release | 2026-09-22 21:18:40 / `9fef159` | **2026-09-23 12:04:50 / `ee71d3b`** | `ls -l --time-style=full-iso …` |

Con số frontend đọc **sau** khi lệnh xong, từ `--outputFile` **riêng** mỗi lần chạy
(`base0505` / `red1` / `g1` / `red2` / `g2` / `mut-*` / `final05`), `rm -f` trước mỗi
lần. `tsc` gọi **riêng, không qua ống** — `$?` sau pipeline đọc mã lệnh **cuối**.

Rust **không đổi vì wave này không sửa một dòng `src-tauri/**` nào.** `--lib --tests`
chạy được vì `tasklist` xác nhận **không** có `git-plum.exe` nào đang chạy.

> ⚠️ **Cảnh báo `npm run build` CÓ TRƯỚC.** `index-*.js` vượt 500 kB: wave 4 đo 645,53
> kB, hôm nay 656,66 kB. Phần tôi thêm ~11 kB. **Không sửa.**

---

## 🔴 Ba điều plan nói mà ĐO ĐƯỢC là sai — và cả ba cùng một nguyên nhân

Plan giả định ứng dụng **đã có** một màn hình xem diff thư mục làm việc. Nó không có.

### 1. *"Nối `HunkBar` vào `DiffViewer`: một `HunkBar` cho mỗi `kindText.hunks`"*

`DiffViewer.tsx:187` nạp bằng:

```ts
ipc.getFileDiff(repoId, selectedCommitId, selectedFile)
```

Đó là diff của một **commit đã có trong lịch sử**, và component `return` sớm khi thiếu
`selectedCommitId`. Gắn nút *"Đưa khối vào vùng chờ"* lên đó là gắn một thao tác **ghi**
lên một thứ **không có gì để ghi**: khối của một commit cũ đã nằm trong lịch sử rồi.

Làm đúng chữ của plan sẽ cho một giao diện **trông như** hoạt động và **không stage được
gì** — đúng lớp lỗi mà cả phase này tồn tại để chống.

### 2. *"Bật/tắt bằng một nút trong `DiffToolbar`"*

Đếm điều kiện để nút đó tới được người dùng, cho một tệp **thư mục làm việc**:

1. `DiffToolbar` render trong `DiffViewer`;
2. `DiffViewer` chỉ render khi `selectedFile` có (`App.tsx`);
3. `DiffViewer` `return` sớm khi thiếu `selectedCommitId`;
4. đường duy nhất đặt `selectedFile` cho worktree là bấm hàng `ChangeList`;
5. `ChangeList` chỉ mount khi vùng soạn commit đã mở, và vùng soạn chỉ mở khi bấm hàng WIP.

**Năm** điều kiện — nhiều hơn vòng khoá chết bốn điều kiện của Phase 4 một bậc, và điều
kiện (3) làm nó **không bao giờ** mở được cho diff thư mục làm việc.

Công tắc thật nằm ở **thanh công cụ**: **một** điều kiện, "có repo đang mở".

### 3. Nguyên nhân chung: `getWorktreeDiff` chưa từng có chỗ gọi

Grep toàn bộ `src/**` trừ tệp test (bằng **công cụ Grep**):

```text
src\lib\ipc.ts:727                     khai báo
src\components\worktree\HunkBar.tsx:30 nhắc trong chú thích
(không có chỗ gọi nào)
```

`ChangeList` chỉ ghi lựa chọn vào `diffStore`, và nó có test ghim rằng nó **không** tự
gọi `getWorktreeDiff`. Tức trước wave này ứng dụng **không có màn hình nào xem diff thư
mục làm việc** — dù Phase 4 đã có cả lệnh Rust lẫn hàm bọc IPC cho nó.

**Hệ quả cho cách đọc kết quả wave này:** bảng khối **không phải** "đường dự phòng cho
một đường chính đã chạy". Nó là đường **duy nhất**, và nó độc lập Phase 3 + Phase 4 theo
thiết kế. Ràng buộc hệ quả 2 được thoả không phải nhờ khéo léo mà nhờ **không có đường
nào khác để mà đi qua**.

---

## 🔴 Ba câu hỏi vòng khoá chết — trả lời cho KẾT QUẢ ĐÃ NỐI DÂY

Orchestrator đòi trả lời cho **kết quả đã nối**, không chỉ cho component. Đây là câu trả
lời cho `WorktreePane` + `HunkTable` như chúng đang chạy trên bản release `ee71d3b`.

### 1. Cái gì mount nó?

**`App.tsx`, khi có repo đang mở và người dùng bấm nút `Thay đổi` ở thanh công cụ.**

Chuỗi điều kiện đầy đủ, không rút gọn:

```text
activeRepo !== null  →  nút `Thay đổi` render
người dùng bấm       →  xemThayDoi = true
                     →  <WorktreePane repoId={…} />
```

**Hai** mắt xích, và mắt xích thứ nhất chỉ đòi "có repo". So với Phase 4: ở đó
`ChangeList` cần vùng soạn đã mở, vùng soạn cần một cú bấm vào hàng WIP, hàng WIP cần
`statusStore` đã có dữ liệu, và dữ liệu đó chỉ `ChangeList` nạp — **bốn** mắt xích khép
thành vòng.

Nút `Thay đổi` có **chữ**, không icon, ngược quy ước thanh công cụ. Cố ý: nó là đường vào
**duy nhất** tới cả staging theo khối lẫn danh sách "Vừa huỷ gần đây", và tiêu chí thành
công 4 hỏi đúng câu *"bạn tìm thấy nó mà không phải hỏi nó ở đâu?"*.

Có cổng: `vòng-1` — render, **không bấm gì**, `statusStore` rỗng, danh sách tệp phải hiện.

### 2. Cái đó có phụ thuộc một thứ mà chính nó mới tạo ra không?

**Không.**

`WorktreePane` **tự gọi** `ipc.getStatus` trong `useEffect` lúc mount và render danh sách
**từ kết quả đó**. Lời gọi nạp đứng **trước** mọi điều kiện hiển thị.

Vòng của Phase 4 có đúng hình dạng **ngược lại**: `ChangeList` vừa là chỗ **duy nhất**
gọi `refresh`, **vừa** chỉ mount khi kết quả của `refresh` đã có.

🔴 Và nó gọi `getStatus` **thẳng** rồi ghi qua `applyExternal`, **không** qua
`statusStore.refresh`. Không phải sở thích: `refresh` là API `ChangeList` sở hữu và gộp
lời gọi theo `repoId` (`dangBay`). Đi qua nó nghĩa là hai đường vào **chia một cơ chế
gộp**, và một lỗi ở cơ chế đó dừng **cả hai** — tức hai đường vào chỉ còn là một, và
ràng buộc hệ quả 2 mất ý nghĩa dù mã trông vẫn tách bạch.

Có cổng: `vòng-2` — `getStatus` phải được gọi trên một store **rỗng**.

### 3. Có đường nào tới nó KHÔNG đi qua một component Phase 4 không?

**Có, và đó là đường DUY NHẤT.** Đây là câu wave 4 trả lời "❌ chưa".

| | wave 4 | wave 5 |
|---|---|---|
| `HunkBar` **có thể** chạy không cần Phase 4 | ✅ M25 + M25b | ✅ giữ nguyên |
| Người dùng **có** đường tới nó | ❌ **chưa có đường nào** | ✅ **M28b đỏ 6 test** |

`WorktreePane` không nhập `ChangeList`, không nhập `CommitBox`, không đọc `commitStore`,
không liên quan hàng WIP. Nó cũng không nhập `DiffViewer` hay module Phase 3 nào.

Cổng `vòng-3` đo **đường đi thật**, không chỉ sự vắng mặt: mount → bấm tệp → bảng khối
hiện → bấm nút khối → `stage_hunk` gọi với đúng `path`, và ở **mỗi** bước khẳng định
`change-list` / `commit-box` / `wip-row` **vắng mặt**. Khẳng định vắng mặt một mình chưa
đủ (một cài đặt nhập `ChangeList` rồi ẩn bằng CSS vẫn qua) — đo đường đi mới là phép đo.

🔴 **`vòng-3` đỏ dưới M28b.** Đó là điều đáng giá nhất của wave này: mệnh đề hai nay
được **ghim bằng test**, không chỉ được hứa.

### 🔴 Phần KHÔNG thoải mái của câu trả lời

Ba câu trên đóng được ràng buộc **chống-Phase-4**. Chúng **không** đóng được vòng
commit, và tôi ghi đúng như nó là:

> **Vùng soạn commit (`CommitBox`) KHÔNG nằm trong màn hình `Thay đổi`.**

Người dùng chọn khối, đưa vào vùng chờ — rồi phải bấm `Đồ thị`, tìm hàng WIP, bấm nó để
mở vùng soạn, và commit ở đó. Tức **tiêu chí thành công 1** (*"commit chứa đúng khối
đó"*) đi qua **đúng** đoạn mã Phase 4 mà chưa ai bấm thử, và **exit gate** của Task 4
cũng vậy.

Đưa `CommitBox` vào `WorktreePane` sẽ dựng lại chính phụ thuộc Phase 4 mà ba câu trên
vừa cắt. Đó là một đánh đổi **thật** và tôi **không** tự quyết: nó được ghi vào
`docs/10-phase5-dogfood.md` như một khoảng trống tường minh ở bước 4 và ở exit gate, kèm
câu *"nếu bước này không làm được từ trong ứng dụng, đó là một kết quả CÓ GIÁ TRỊ — ghi
lại đúng như vậy"*.

---

## Bảng đột biến — chạy thật, dán đầu ra đỏ

Hoàn nguyên bằng `cp` từ bản sao dựng bằng `git show <sha>:<path> > file`, **không bao
giờ** `git checkout --`. Sau mỗi lần: `diff` với bản sao → `IDENTICAL`, và
`git status --porcelain` trên ba tệp in dạng **thô** → rỗng.

| # | Đột biến | Đỏ mong đợi | **Đo được** |
|---|---|---|---|
| MG1 | cổng byte: `--cached` → `--index` | (0) | ✅ **ĐỎ** |
| MG2 | cổng byte: bản vá cả ba khối | (0) | ⚠️ đỏ nhưng **VÔ HIỆU** |
| MG2b | cổng byte: `git add` cả tệp | (0) | ✅ **ĐỎ**, và **chỉ** (0) đỏ |
| M26 | `TrashList` bỏ sắp xếp | Test 3 | ✅ **1 đỏ** |
| M27 | khôi phục luôn dùng `daSap[0]` | Test 2 | ✅ **1 đỏ** |
| M28 | bảng khối đọc store Phase 3, `fieldset disabled` | Test 4 | ⚠️ **0 đỏ — VÔ HIỆU** |
| **M28b** | bảng khối đọc store Phase 3, không render `HunkBar` | Test 4 | ✅ **6 đỏ** |
| M29 | `HunkBar` nhận `0` thay vì `index` | nối dây | ✅ **1 đỏ** |
| M30 | bỏ cổng chống đua của `HunkTable` | *(không dự đoán)* | ⚠️ **0 đỏ → ĐỎ sau khi viết cổng** |
| M31 | `WorktreePane` truyền `untracked={false}` cứng | *(không dự đoán)* | ✅ **1 đỏ** |

### 🔴 MG2b — phép đo đắt nhất của Task 1, và phép kiểm nó bảo vệ KHÔNG có trong plan

Plan đòi **ba** phép kiểm byte. Tôi thêm phép kiểm **(0)**: *chỉ khối giữa vào vùng chờ*.
MG2b chứng minh vì sao:

```text
  [git apply --cached --recount exit = 0]
  [vùng chờ: '+MOD 16' = 1 (cần 1)   '+MOD 03'/'+MOD 29' = 2 (cần 0)]
  [\r  trước = 29   sau = 29]
  🔴 (0) SAI khối vào vùng chờ: +MOD 16 = 1, hai khối kia = 2
  ✅ (1) CRLF giữ nguyên:      \r 29 → 29
  ✅ (2) vẫn thiếu dòng cuối:  byte cuối là \n? KHONG → KHONG
  ✅ (3) byte 0xE9 giữ nguyên: CO → CO
```

Đột biến là `git add -- muctieu.txt` — **stage cả tệp**. Nó giữ nguyên **hoàn hảo** mọi
byte trong thư mục làm việc (nhờ `--cached`), nên **cả ba** phép kiểm plan yêu cầu đều
xanh trên một cài đặt **không hề staging một phần**.

**Thiếu (0), cổng WORK-04 là một cổng XANH GIẢ** — đúng lớp lỗi "hình dạng đúng, dữ liệu
vô hại" (`CONTEXT.md` 4.2, đã xảy ra ba lần qua hai phase).

MG2 được ghi lại **nguyên trạng** dù vô hiệu: nó đỏ vì `patch does not apply` (lỗi cú
pháp bản vá), không vì "stage cả tệp". Lần thứ **ba** dự án phải thu hẹp một đột biến —
M14→M14c (wave 3), M25→M25b (wave 4), MG2→MG2b (wave này).

### 🔴 M28 VÔ HIỆU — và cách phát hiện

M28 bản đầu bọc `HunkBar` trong `<fieldset disabled>`. **0 đỏ trên 27 test.**

Trước khi kết luận bất cứ điều gì, tôi viết một test **thăm dò** cho chính giả định của
đột biến:

```text
[total=2 failed=2]
  FAILED : button.disabled trong fieldset disabled === true?
  FAILED : click bị chặn?
```

**happy-dom KHÔNG lan `disabled` từ `fieldset` xuống `button`** — cả thuộc tính lẫn việc
chặn click. Đột biến đó **không biểu diễn được khuyết tật nào**, nên 0 đỏ của nó nói về
**happy-dom**, không về cổng.

M28b (`{tepPhase3 !== null && <HunkBar …>}`) — đúng hình dạng vòng khoá chết:

```text
[total=27 failed=6 suitesFailed=7]
  ĐỎ: 🔴 M28: render MỘT MÌNH, statusStore rỗng, không component Phase 3/4 nào
      → nút khối vẫn bấm được và vẫn gọi IPC
  ĐỎ: 🔴 M29: mỗi khối truyền chỉ số THẬT của nó xuống stage_hunk
  ĐỎ: bấm "Làm mới" trên băng file_changed nạp lại diff THẬT (lời gọi thứ hai)
  ĐỎ: 🔴 vòng-3: chọn tệp → chọn khối → stage_hunk, KHÔNG có ChangeList/CommitBox/
      hàng WIP nào trên đường
  ĐỎ: chọn tệp CHƯA THEO DÕI → nút huỷ dùng động từ "Xoá", không "Huỷ bỏ"
  ĐỎ: chọn tệp ĐÃ THEO DÕI → nút huỷ dùng động từ "Huỷ bỏ", không "Xoá"
```

### 🔴 M30 — lỗi #9 tái hiện sống, wave thứ TƯ liên tiếp

M30 (gỡ **cả hai** phép kiểm `id !== lanNap.current`) cho **0 đỏ trên 27 test**.

Grep toàn bộ `src/**/*.test.*` bằng **công cụ Grep** cho `lanNap|requestId|chống đua`:

```text
DiffViewer.test.tsx:167   cổng chống đua (T-03-31)
FileHistory.test.tsx:252  cổng chống đua khi đổi tệp (T-03-37)
CommitBox.test.tsx:84     cảnh báo về test chống đua đo quá sớm
HunkTable                 — KHÔNG KHỚP NÀO
```

**Rỗng nghĩa là cổng thiếu, không phải mã đúng.** Viết test → chạy lại:

```text
[total=28 failed=1]
  ĐỎ: 🔴 M30: phản hồi của lần nạp CŨ về SAU phải bị BỎ, không ghi đè bảng đang hiện
AssertionError: bảng phải giữ kết quả của lần nạp MỚI NHẤT […]
  expected [ …(1) ] to have a length of 2 but got 1
```

**M30: 0 đỏ → ĐỎ.** Cùng khuôn M11 (wave 2), M18 (wave 3), M26 (wave 4) — **bốn wave
liên tiếp**, và **ba wave liên tiếp** ca đó **không có** trong bảng đột biến của plan.

**Vì sao đáng có cổng:** trên `DiffViewer` một phản hồi về sai thứ tự làm người dùng
**đọc** nhầm tệp. Ở đây nó làm họ **ghi** nhầm tệp — `path` xuống `stage_hunk` là prop
**hiện tại**, chỉ số khối trên màn hình thuộc phản hồi **cũ**. Với `--recount` bản vá
**có thể áp thành công** vào sai tệp. Rủi ro R1 ở dạng tệ nhất: không lỗi, không cảnh
báo, tệp hỏng.

### M31 — cổng bắc qua HAI tầng

Wave 4 có M21+M22 ghim `HunkBar` hiển thị đúng động từ **khi được truyền** `untracked`
đúng. Nhưng chỗ **quyết định** giá trị đó là `WorktreePane`, và một `untracked={false}`
cứng ở đó làm **cả hai** cổng của wave 4 vô dụng — mà **không test nào của wave 4 thấy**.
Đó là lỗi #9 nhìn từ một tầng lên. M31 đỏ đóng nó.

---

## Điều TÔI làm khác plan, và vì sao

### 1. `WorktreePane` — một component plan không nhắc tới

Plan chỉ nói "bảng khối + nút toggle". Nhưng một bảng khối cần biết **tệp nào**, và
đường duy nhất có sẵn để chọn tệp là `ChangeList` — tức đúng phụ thuộc phải cắt. Nên
`WorktreePane` mang **danh sách tệp riêng của Phase 5**.

🔴 **Cái giá là hai danh sách tệp trong ứng dụng.** Đó là nợ **có chủ ý**, không phải
sơ suất: khi Phase 4 đóng được hai cổng của nó và `ChangeList` có bằng chứng từ mắt
người, hai danh sách nên gộp. Gộp **trước** khi có bằng chứng đó là đánh đổi sai chiều.

### 2. `luc` của `MucThungRac` — lấy lại được, không cần chỗ lưu phụ

Wave 3/4 ghi rằng `paths`/`luc`/`nhan` đều mất vì `for-each-ref`. Đọc lại
`commands/trash.rs`: tên ref là `refs/git-plum-trash/{epoch_ms}-{n}` — **mốc mili giây
nằm ngay trong tên**, và `for-each-ref` **có** trả tên ref.

Nên `luc` **không bao giờ mất**; nó chỉ chưa được đọc ra. `mocTuRefName()` đọc nó, và có
test khẳng định `data-ms` đúng **và** chuỗi hiện ra không chứa `1970` — tức không đọc
`luc: 0`.

`paths`/`nhan` thì **thật sự mất** — vẫn là nợ.

### 3. `TrashList` **nói ra** khoảng trống thay vì hiện dòng trắng

Danh sách hiện *"Bản lưu này chưa ghi lại tên tệp — danh sách dựng từ tên ref"* kèm 8 ký
tự đầu `objectId`.

Ba dòng trắng trơn là một danh sách người dùng **không dám bấm**: nút ở đó **ghi đè nội
dung tệp của họ**, và họ không biết mục nào chứa gì. Có cổng cho **cả hai** chiều — mục
không có `nhan` phải hiện câu giải thích, mục **có** `nhan` phải hiện `nhan`. Thiếu
chiều hai, một cài đặt bỏ qua `nhan` hoàn toàn sẽ qua, và khoảng trống không đóng được
dù backend đã sửa.

### 4. `onLamMoi` của wave 4 cuối cùng được truyền

Wave 4 thêm prop này và ghi *"chưa được truyền ở đâu cả"*. `HunkTable` truyền
`onLamMoi={() => void nap()}`, và có test khẳng định bấm nó sinh lời gọi
`getWorktreeDiff` **thứ hai** — không có test đó, một `() => {}` qua được mọi test khác
và nút lại nói dối đúng như bản đầu của wave 4.

### 5. Cổng byte dùng `rtk proxy git`, không `git`

Xem mục "bẫy môi trường".

---

## 🔴 Chưa kiểm và nợ — ghi rõ, không làm mượt

### Hai cổng người kiểm: **CHƯA CHẠY**

| Cổng | Trạng thái |
|---|---|
| Task 3 — mười bước trên release | 🔴 **CHƯA CHẠY** |
| Task 4 — exit gate, một commit thật | 🔴 **CHƯA CHẠY** |

Chủ dự án chọn "dừng và báo". Executor **không** phê duyệt, **không** ghi "Đạt" cho bất
kỳ bước nào. Bản dựng đã sẵn: `2026-09-23 12:04:50`, commit `ee71d3b`.

### Nợ mới do wave này tạo ra

- 🔴 **`blobHash` luôn là `''`.** `getWorktreeDiff` không trả mã băm blob, nên
  `HunkTable` truyền chuỗi rỗng. Hệ quả: WORK-05 **mất lớp cảnh báo sớm** (giao diện
  không phát hiện lệch trước khi gọi). Lớp **bảo vệ** không mất — Rust vẫn tự đọc lại
  hash và `git apply --check` vẫn từ chối — nhưng WORK-05 còn **một** lớp thay vì hai.
  Lấp bằng cách thêm một trường vào `FileDiff` của đường worktree.
- 🔴 **Hai danh sách tệp.** Xem mục "điều tôi làm khác plan" #1.
- 🔴 **Vòng commit chưa gặp staging theo khối.** Xem "phần không thoải mái" ở trên.

### Nợ kế thừa, **chưa** lấp ở wave này

- **`paths` và `nhan` của `MucThungRac` vẫn rỗng.** Cần chỗ lưu phụ ở Rust.
- **T-05-11 chưa cài** — `khoi_phuc` ghi đè thẳng, không tự lưu trước. `TrashList`
  **không** tự vá bằng một `confirm()`: T-05-13 nói rõ mọi đường huỷ phải tự lưu ở
  **Rust**, nơi `BienNhan` không dựng được ngoài crate. Một hộp thoại ở giao diện tạo
  cảm giác an toàn cho một đường **vẫn** mất dữ liệu khi gọi từ chỗ khác.
- **`nhanDongTu` vẫn là nguồn sự thật thứ HAI** song song `DongTu::nhan()` của Rust,
  không test nào đối chiếu. **Không lấp.**
- **Không có test tích hợp thật nào trên đường IPC.** Mọi lời gọi vẫn là `vi.fn()`.
  Tôi **không** đóng được khoảng trống này — nó cần chạy Tauri thật. Hợp đồng tham số
  vẫn được đối chiếu **bằng mắt** với `commands/hunk.rs:322` và `commands/trash.rs`.
  Wave này **thêm** một bề mặt vào khoảng trống đó: `getWorktreeDiff(repoId, path,
  staged)`.
- **`dangChon`/`chon` nay CÓ người dùng** (`HunkTable` tô sáng khối đang chọn) — wave 4
  lo nó thành mã chết; nó không. **`boChon` cũng có** (`WorktreePane`).

### Mọi khẳng định về bố cục: **có mã, chưa kiểm**

**Không một test nào** trong 731 test quan sát được: khối đang chọn nhìn ra khác khối
không chọn · thanh nút không đè nội dung · hai cột không đè nhau ở panel hẹp ·
`TrashList` không tràn/cắt chữ · bảng khối cuộn được · băng "hãy làm mới" không che dòng
dưới. Chúng nằm trong `app.css` kèm chú thích nói đúng như vậy.

---

## 🔴 Đánh giá lại lịch — ROADMAP dặn làm ở cuối phase này

ROADMAP: *"Đây là phase nhiều khả năng vượt kế hoạch nhất. Nghiên cứu xếp staging theo
khối ở mức HIGH và khuyến nghị ngân sách gấp 2–3 lần ước lượng ngây thơ. Đánh giá lại
lịch ở cuối phase này trước khi cam kết phạm vi Phase 6."*

**Phase này tốn ĐÚNG như dự phòng HIGH dự liệu, nhưng tiền đi vào chỗ KHÁC chỗ dự đoán.**

Nghiên cứu lo về **tầng Rust**: dựng bản vá con, `--recount`, CRLF, byte không UTF-8.
Tầng đó chạy **trơn tru** — wave 1–3 xong trong ngân sách, và WORK-04 hôm nay đóng được
bằng **máy** ở lần chạy thứ tư của một script viết trong một giờ.

Tiền thật sự đi vào **ba** chỗ không ai dự toán:

1. **Nợ kiểm chứng của Phase 3 và Phase 4.** Mỗi wave của Phase 5 phải tự dựng lớp
   phòng thủ chống một tầng dưới chưa ai xác nhận. Wave 4 dựng `HunkBar` không phụ
   thuộc Phase 4; wave 5 phải dựng **cả một danh sách tệp** vì lý do đó.
2. **Plan sai ở chỗ nối dây**, và sai vì nó tin một màn hình đã tồn tại. Ba điều ở mục
   trên là hệ quả của **một** giả định chưa kiểm.
3. **Cổng giả.** Bốn wave, bốn ca lỗi #9, cộng ba lần phải thu hẹp một đột biến vô hiệu.
   Việc *kiểm cổng* tốn ngang việc *viết mã*.

**Điều đó đổi gì cho Phase 6:**

- 🔴 **Ngân sách 2–3× nên áp cho mọi phase xây trên tầng chưa có bằng chứng mắt người,
  không riêng phase "khó".** Bốn phase liên tiếp đóng với nợ kiểm chứng, và chi phí đó
  là **kép**: mỗi phase sau vừa trả nợ cũ vừa tạo nợ mới.
- 🔴 **Chi phí rẻ nhất có thể trả ngay là đóng hai cổng của Phase 4 và hai cổng của
  Phase 5 — bốn cổng, một bản release, một buổi.** Rẻ hơn hẳn việc Phase 6 lại dựng
  thêm một đường vào thứ hai cho một tầng nữa.
- **Không cam kết phạm vi Phase 6 trước khi bốn cổng đó có kết quả.** Nếu dogfood cho
  thấy `CommitBox` hỏng, việc đầu tiên của Phase 6 là sửa nó, không phải tính năng mới.

---

## 🔴 Nợ kiểm chứng của Phase 4 — nêu rõ, KHÔNG tự quyết định

`CONTEXT.md` mục 7: Phase 4 còn **hai cổng đỏ** và `docs/09-phase4-dogfood.md` **chưa tồn
tại**. Đóng Phase 5 mà Phase 4 chưa đóng nghĩa là **bốn phase liên tiếp** đóng với nợ
kiểm chứng.

Hệ quả **cụ thể**, không trừu tượng: bước 4 và exit gate của Phase 5 **đều** đi qua
`CommitBox` — mã Phase 4 chưa ai bấm thử. Nếu chúng hỏng, hai cổng của Phase 5 sẽ đỏ vì
khuyết tật **của Phase 4**, và phải ghi đúng như vậy chứ không tính vào Phase 5.

Đây là câu hỏi cho **chủ dự án**.

---

## Bẫy môi trường gặp phải — bốn cái, ba cái mới

1. 🔴 **`rtk` TÓM TẮT `git diff`, không chỉ làm hỏng `grep`.** `grep -c '^@@'` trên đầu
   ra của nó trả **1** trong khi bản vá thật có **3** khối — vì nó thay bản vá bằng một
   bản tóm lược cho người đọc (`m.txt | 6 +++---` / `--- Changes ---`). Một script dựng
   bản vá con từ đó dựng ra **rác**, im lặng. Mọi lệnh git trong
   `verify-work04-bytes.sh` đi qua **`rtk proxy`**. *(Mới — nặng hơn cảnh báo của
   `CONTEXT.md` mục 5.)*
2. 🔴 **`node` là chương trình Windows, không hiểu `/tmp`.** `node -e … /tmp/x` mở
   `C:\tmp\x` và thất bại với `ENOENT` trỏ vào một đường dẫn không ai viết ra. Mọi
   đường dẫn đưa cho `node` đi qua `cygpath -w`. *(Mới.)*
3. 🔴 **`"$(printf '\r\n')"` truyền chuỗi RỖNG** — thay thế lệnh của shell cắt mọi ký
   tự xuống dòng ở cuối, và `\r\n` toàn là ký tự xuống dòng. Tệp "CRLF" ra đời với **0**
   byte `\r`. **Thứ bắt được là lượt ĐỐI CHỨNG** (hai lượt cùng `\r=0` → "không phân
   biệt được", exit 1). Cổng đỏ đúng chỗ, trước khi ai kịp tin một kết quả sai. *(Mới —
   và là bằng chứng sống rằng lượt đối chứng không thừa.)*
4. **`git apply` cần bản vá kết thúc bằng `\n`.** `join("\n")` trên các khối đã tách làm
   mất ký tự đó → `error: corrupt patch at <tệp>:13`, trỏ vào một chỗ nghe như lỗi nội
   dung. Đúng lớp `CONTEXT.md` 4.3.
5. **`grep -c` trả `0` thoát `1`** — làm một lệnh nối `&&` trông như thất bại. Gặp lại,
   như wave 4 ghi.
6. **Đĩa 98%, 5,0 GB trống** — thấp hơn con số 9,7 GB tôi được báo. `target/debug`
   **vẫn còn 6,0 GB** (không bị xoá như tôi được báo), nên `cargo test` **không** phải
   rebuild toàn bộ. `tauri build` xong trong 2m 47s, không lỗi `1455`.
7. **Phối hợp nhiều phiên:** `git status --porcelain` in **thô** trước và sau toàn bộ
   lượt làm việc cho đúng tập 8 mục của phiên khác, **không tệp nào bị tôi chạm**. Mọi
   `git add` theo **đường dẫn tường minh**; `git diff --cached --name-only` in thô và
   **nhìn** trước mỗi commit.

---

## Commit

| SHA | Nội dung |
|---|---|
| `ffb13c0` | test(05-05): cổng MÁY cho tiêu chí thành công 2 — `od -c`, hai lượt |
| `418dbef` | test(05-05): test đỏ cho `TrashList` và `HunkTable` (RED) |
| `73ef217` | feat(05-05): `WorktreePane` — đường vào thứ hai (GREEN) |
| `ee71d3b` | test(05-05): cổng thiếu cho cổng chống đua — lỗi #9 |

Cặp RED→GREEN đầy đủ. **0 tệp bị xoá** trong toàn dải commit.

🔴 **Lỗi #8 tái hiện SỐNG hai lần** ở hai lượt RED của wave này (wave 4 gặp hai lần):

```text
[total=0 failed=0 suitesFailed=2]
Failed to resolve import "@/components/worktree/HunkTable"
```

`total=0 failed=0`. Một cổng chỉ kiểm `numFailedTests !== 0` gọi đây là **XANH**.

## Self-Check: PASSED

Tệp có thật trên đĩa: `scripts/verify-work04-bytes.sh`,
`src/components/worktree/{HunkTable,TrashList,WorktreePane}.tsx` + `.test.tsx`,
`docs/10-phase5-dogfood.md`, `.planning/phases/05-hunk-staging/VERIFICATION.md`,
`.planning/phases/05-hunk-staging/05-05-SUMMARY.md`.

Commit có thật trong `git rev-list`: `ffb13c0`, `418dbef`, `73ef217`, `ee71d3b`.

`git diff --diff-filter=D --name-only e3ee7e7..HEAD` → **rỗng** (0 tệp bị xoá).

`git status --porcelain` in **thô**: đúng 6 tệp của phiên khác + 2 tệp chưa theo dõi của
phiên khác, **không tệp nào bị wave này chạm**.
