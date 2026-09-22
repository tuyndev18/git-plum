---
phase: 04-commit-loop
plan: 03
subsystem: commit-loop-frontend
tags: [WORK-08, WORK-09, zustand, plugin-store, debounce, M14, partial-plan]
status: frontend-only
requires:
  - "04-02 (statusStore.byRepo, ChangeList, RepoStatus, GitErrorCode)"
provides:
  - "commitDraft.ts — nháp bền vững khoá theo repoId, ghi trì hoãn, repoId đóng gói lúc LÊN LỊCH"
  - "commitStore.ts — draftByRepo, commit/amend, no-verify mặc định TẮT"
  - "CommitBox.tsx — vùng soạn + no-verify + amend cảnh-báo-không-chặn"
  - "ipc.createCommit / ipc.amendCommit + 3 mã lỗi mới (chỉ phía TypeScript)"
blocked-on:
  - "🔴 Nửa RUST chưa làm: create_commit / amend_commit chưa tồn tại. Xem mục 'Phần KHÔNG làm'."
affects:
  - "04-04 (watcher), 04-05 (gắn CommitBox vào App.tsx + checkpoint mắt người)"
tech-stack:
  added: []
  patterns:
    - "repoId đóng gói lúc LÊN LỊCH, không đọc lại lúc ghi — áp cho MỌI phép ghi trì hoãn"
    - "đồng hồ giả (fake timers) cho mọi test có trì hoãn — tất định thay vì đua thời gian thật"
    - "store giả CÓ TRẠNG THÁI THẬT (Map) thay vì mockResolvedValue trơn"
key-files:
  created:
    - src/lib/commitDraft.ts
    - src/lib/commitDraft.test.ts
    - src/stores/commitStore.ts
    - src/stores/commitStore.test.ts
    - src/components/worktree/CommitBox.tsx
    - src/components/worktree/CommitBox.test.tsx
  modified:
    - src/lib/ipc.ts
    - src/styles/app.css
decisions:
  - "CSS chèn TRƯỚC header `/* --- Thanh cuộn ---`, không phải cuối tệp — biên sạch với nội dung đã commit"
  - "KHÔNG giả lập @/lib/commitDraft trong commitStore.test — đi qua module thật để đọc GIÁ TRỊ thật"
  - "dùng fireEvent + act thay vì @testing-library/user-event — gói đó KHÔNG có trong dự án"
  - "app.css KHÔNG commit được: mang 3 dòng xoá của session khác, git không stage một phần tệp"
metrics:
  duration: ~2h
  completed: 2026-09-22
---

# Phase 4 Plan 03 (NỬA FRONTEND): vòng commit — Summary

> 🔴 **Plan này CHƯA XONG.** Chỉ nửa frontend được thực thi. **Task 1 (toàn bộ phần
> Rust) KHÔNG được làm** và một session khác đang chạy nó song song. Ứng dụng
> **chưa tạo được một commit thật** sau plan này — `ipc.createCommit` gọi xuống một
> command Tauri **chưa tồn tại**. Xem mục "Phần KHÔNG làm" trước khi đọc tiếp.

## Số đo — lệnh nào cho số nào

| Phạm vi | Trước | Sau | Lệnh đã chạy |
|---|---|---|---|
| Frontend, tổng | **508** | **587** (+79) | `npx vitest run --reporter=json` |
| — của plan này | — | **79** | `commitDraft` 19 + `commitStore` 33 + `CommitBox` 27 |
| — của session khác | 508 | 508 | không đổi |
| `npx tsc --noEmit` | sạch | **sạch** | `npx tsc --noEmit` → `No errors found` |
| `npm run build` | sạch | **sạch** | `npm run build` → `✓ built in 555ms` |
| Rust | — | **CHƯA ĐO** | không chạy cargo, có chủ ý — xem dưới |

**Phép cộng khớp chính xác:** 508 + 79 = 587. Con số 508 là baseline mà điều phối viên
báo giữa chừng (đã dời từ 494 trong brief ban đầu vì session khác thêm test). Tôi **đo
lại** thay vì tin con số cũ, và phép tách theo tên tệp xác nhận 508 test của người khác
**không đổi một cái nào** — tức tôi không phá gì của họ.

`.vitest/json/output.json` được `rm -f` trước **mỗi** lần chạy, và harness khẳng định
tệp đã biến mất trước khi chạy tiếp (trap 3.6 của CONTEXT.md).

### Ổn định — chạy nhiều lần

| Phạm vi | Số lần | Kết quả |
|---|---|---|
| `commitDraft` + `commitStore` (có trì hoãn) | **5/5** | 52 passed, 0 đỏ, **mọi lần** |
| Toàn bộ suite frontend | **3/3** | 587 passed, 0 đỏ, **mọi lần** |

Không lần nào dao động. Mọi test có trì hoãn dùng **đồng hồ giả**, nên đây là tính tất
định thật chứ không phải may mắn — lỗi #6 của CONTEXT.md 3.1 đỏ 1/5 lần vì nó đua với
thời gian thật.

## 🔴 Phần KHÔNG làm — đọc trước khi lập kế hoạch tiếp

| Task | Trạng thái | Vì sao |
|---|---|---|
| **Task 1** — `create_commit` / `amend_commit`, `error.rs`, `lib.rs`, fixture hook | ❌ **KHÔNG LÀM** | Thuần Rust. Ngoài phạm vi được giao; một session khác chạy song song |
| Task 2 — nháp bền vững | ✅ xong | |
| Task 3 — `CommitBox` | ✅ xong (trừ CSS không commit được) | |

**Hệ quả cụ thể, không phải cảnh báo chung chung:**

1. **`ipc.createCommit` / `ipc.amendCommit` gọi vào khoảng không.** Chúng `invoke`
   `create_commit` / `amend_commit`, và hai command đó **chưa được đăng ký** trong
   `generate_handler!`. Bấm nút commit trong ứng dụng thật lúc này sẽ ném lỗi Tauri.
2. **Ba mã lỗi mới (`empty_message`, `nothing_to_commit`, `hook_rejected`) mới chỉ có
   ở phía TypeScript.** `GitError` bên Rust chưa có variant tương ứng. Phía Rust phải
   dùng **đúng ba chuỗi mã này** khi serialize, nếu không `commitStore.bocLoi` không
   nhận ra lỗi hook và đầu ra hook sẽ mất đường tới `<pre>` — tức đột biến M5 tự xảy
   ra trên sản phẩm mà không test nào đỏ, vì test của tôi giả lập lớp IPC.
3. **Nguyên văn hook chưa được kiểm trên hook THẬT.** Plan đòi đo trên repo mẫu có hook
   thật trước khi viết phép phân loại lỗi. Tôi **không** làm được điều đó (fixture là
   tệp Rust/bash ngoài phạm vi), nên chuỗi `HOOK-PRE-COMMIT-REJECTED` trong test của
   tôi là **quy ước tôi đặt ra**, không phải quan sát. Người làm Task 1 phải in đúng
   chuỗi đó **hoặc** báo lại để tôi sửa test.

**Ràng buộc giao diện mà phía Rust phải tôn trọng** (test của tôi ghim, nên lệch là đỏ):

```ts
createCommit(repoId: string, message: string, noVerify: boolean) -> RepoStatus
amendCommit(repoId: string, message: string, noVerify: boolean) -> AmendResult
//   AmendResult = { status: RepoStatus, wasPushed: boolean }
```

Tham số đi qua `invoke` dạng **camelCase** (`{ repoId, message, noVerify }`) — quy ước
đã ghim từ 04-02 (đột biến M11 của wave đó).

## Bảng đột biến — đầu ra ĐỎ thật, đã dán

Quy trình: sao lưu → áp → **khẳng định tệp đã thật sự đổi** (không đổi ⇒ *lỗi harness*,
**không** ghi "0 đỏ" như một kết quả) → chạy → ghi tên test đỏ → phục hồi.

| # | Đột biến | Đỏ | Test đỏ (tên thật) |
|---|---|---:|---|
| **M1** | Thêm `--no-verify` **vô điều kiện** | **4** | `🔴 noVerify=false được truyền TƯỜNG MINH…`, `🔴 tắt → createCommit nhận noVerify=false`, +2 |
| **M2** | Bỏ hẳn nhánh `--no-verify` | **2** | `🔴 noVerify=true thì cờ được truyền xuống…`, `🔴 bật → createCommit nhận noVerify=true…` |
| **M3** | `-m <message>` làm biến dạng thông điệp | **2** | `🔴 M3 — chuỗi đi xuống IPC giống HỆT chuỗi người dùng gõ, từng ký tự` + 1 |
| **M4** | Bỏ phép chặn thông điệp rỗng | **3** | `🔴 thông điệp rỗng → lỗi, và KHÔNG gọi IPC`, `KHÔNG tự sửa thông điệp…`, +1 |
| **M5** | Lỗi hook trả thông báo **chung** | **2** | `🔴 lỗi mang NGUYÊN VĂN đầu ra hook…`, `hiện NGUYÊN VĂN đầu ra hook trên màn hình` |
| **M6** | 🔴 Xoá nháp **cả khi thất bại** | **4** | `hook từ chối → nháp còn nguyên trong bộ nhớ`, `…trên ĐĨA cũng còn nguyên`, `amend thất bại → nháp còn nguyên`, `🔴 textarea VẪN GIỮ thông điệp` |
| **M7** | Một khoá dùng chung, bỏ `repoId` | **6** | `khoá chứa repoId…`, `nháp của A không rò sang B…`, `xoá nháp của đúng repo đó`, +3 |
| **M8** | Bỏ phép **xả** nháp khi đổi repo | **1** | `🔴 switchRepo XẢ nháp treo trước khi đổi — không mất ký tự cuối (M8)` |
| **M9** | 🔴 Vô hiệu nút amend khi `wasPushed` | **1** | `🔴 cảnh báo hiện rồi, nút VẪN BẤM ĐƯỢC và amend VẪN CHẠY lần nữa` |
| **M13** | Nút không vô hiệu lúc đang commit | **3** | `lời gọi thứ hai bị bỏ…`, `nút VÔ HIỆU trong lúc đang commit`, `bấm hai lần → IPC chỉ MỘT lần` |
| **M14** | 🔴 `saveDraft` đọc `repoId` **lúc ghi** | **3** | xem mục riêng bên dưới |
| M10, M11, M12 | — | — | **thuần Rust, KHÔNG chạy được** (xem "Phần KHÔNG làm") |

### M6 — bảo vệ công sức người dùng

```
x hook từ chối → nháp còn nguyên trong bộ nhớ
    AssertionError: expected '' to be 'feat: một thông điệp dài mà người dùn…'
x hook từ chối → nháp trên ĐĨA cũng còn nguyên
    AssertionError: expected undefined to be 'công sức người dùng'
x 🔴 hook từ chối → textarea VẪN GIỮ thông điệp người dùng vừa gõ
    AssertionError: expected '' to be 'feat: thông điệp dài mà người dùng vừ…'
```

Ba cổng đo **ba tầng khác nhau** có chủ ý — bộ nhớ, đĩa, và DOM. Một cài đặt xoá nháp ở
bộ nhớ mà quên đĩa (hoặc ngược lại) vẫn là mất dữ liệu ở một trong hai đường người dùng đi.

### M9 — "cảnh báo, KHÔNG chặn"

```
x 🔴 cảnh báo hiện rồi, nút VẪN BẤM ĐƯỢC và amend VẪN CHẠY lần nữa
    AssertionError: expected true to be false
```

Phép khẳng định then chốt **không** phải "cảnh báo có tồn tại" — mà là `amendCommit`
được gọi **lần thứ hai sau khi** cảnh báo đã hiện trên màn hình. Một cổng chỉ kiểm sự
tồn tại của cảnh báo sẽ **xanh** dưới đột biến này.

### M5 — nguyên văn hook

```
x 🔴 lỗi mang NGUYÊN VĂN đầu ra hook, không phải thông báo chung của ứng dụng (M5)
    AssertionError: expected 'Commit thất bại.' to contain 'HOOK-PRE-COMMIT-REJECTED'
```

Cổng đọc **cả** dòng nhận diện **và** một dòng thân (`src/b.ts:4`) — một cài đặt giữ
dòng đầu mà cắt phần còn lại vẫn qua được cổng chỉ kiểm dòng đầu.

## 🔴 M14 — và một chỗ plan nói SAI, tôi đã đo

M14 **đỏ 3 test**, đúng như plan đòi:

```
x nháp KHÁC RỖNG của B không bị ghi đè khi ghi treo của A nổ sau lúc đổi repo
    AssertionError: expected 'chữ của repo A' to be 'CÔNG VIỆC RIÊNG CỦA REPO B — không đư…'
x và chữ của A vẫn về đúng khoá của A
    AssertionError: expected '' to be 'chữ của repo A'
x phép ghi mang đúng KHOÁ của repo đã lên lịch, không phải khoá của repo hiện hành
    AssertionError: expected "vi.fn()" to be called with arguments: [ 'draft:repo-a', 'chữ của repo A' ]
```

Đột biến mô phỏng **đúng cái bẫy** plan mô tả: chữ ký `saveDraft(repoId, text)` **giữ
nguyên tham số**, chỉ có phép ghi thật đọc một biến "repo hiện hành" — mã đọc như thể
nó an toàn.

### ⚠️ Plan nói fixture rỗng sẽ cho XANH. Tôi đo: **nó vẫn đỏ.**

Plan (và CONTEXT.md 3.1 bảng lớp lỗi, hàng #3) ghi rằng nháp B **rỗng** sẽ làm test
xanh dưới M14 — "đè lên ô rỗng vẫn trông như B rỗng". Tôi **kiểm chứng độc lập** thay
vì tin, đúng quy tắc "phải chạy đột biến và **thấy** đỏ":

| Biến thể | Kết quả dưới M14 |
|---|---|
| Fixture của tôi (B **khác rỗng**) | **3 đỏ** |
| Fixture **vô hại** (B **rỗng**), phép khẳng định giữ nguyên | **3 đỏ** — vẫn phân biệt được |
| Fixture rỗng + phép khẳng định **tự nhiên** (`expect(loadDraft(B)).toBe('')`) | **1 đỏ** — vẫn phân biệt được |

**Vì sao plan đoán sai ở đây:** lập luận của plan đúng cho một cài đặt mà nháp chỉ sống
trên đĩa. Nhưng `loadDraft` của tôi **ưu tiên phép ghi đang treo** (`if (pending &&
pending.repoId === repoId) return pending.text`) — có ở đó để người dùng không đọc lại
một giá trị cũ hơn thứ họ vừa gõ. Hệ quả phụ: chữ của A trở nên **quan sát được ngay**
khi đọc B, nên lệch khoá để lại dấu vết dù B rỗng.

**Tôi vẫn giữ fixture B khác rỗng**, vì hai lý do độc lập với việc nó có bắt buộc hay
không: thông báo đỏ **nêu tên dữ liệu bị phá** (`CÔNG VIỆC RIÊNG CỦA REPO B`) thay vì
chỉ nói "mong `''`", và nó vẫn đúng nếu ai đó sau này bỏ tối ưu `pending` kia đi.

**Ghi rõ vì đây là một khẳng định trong tài liệu dự án vừa được đo là sai trong ca này:**
lớp lỗi "hình dạng đúng, dữ liệu vô hại" là thật và đã cắn ba lần, nhưng **ca M14 cụ thể
không thuộc lớp đó** với cài đặt hiện tại. Đừng chép hàng #3 của bảng CONTEXT.md 3.1
sang tài liệu khác như một sự thật đã kiểm.

## Deviations from Plan

### 1. [Rule 3 — Blocking] `@testing-library/user-event` KHÔNG có trong dự án

`CommitBox.test.tsx` bản đầu dùng `userEvent.setup()`. Suite **không nạp được**:

```
Error: Failed to resolve import "@testing-library/user-event" — Does the file exist?
 ❯ src/components/worktree/CommitBox.test.tsx (0 test)
```

🔴 **Và JSON report cho `total 0 passed 0 failed 0`** — tức một suite chết **trông
giống hệt** một suite chưa có test. Nếu tôi chỉ đọc `numFailedTests > 0` làm điều kiện
đỏ như `<verify>` của plan viết, cổng này sẽ **XANH trên một suite không chạy gì**.
Đây đúng khuôn cổng #3 (đường dẫn sai → không tìm thấy gì → xanh), chỉ khác lối vào.

**Sửa:** chuyển sang `fireEvent` + `act` (khuôn `ChangeList.test.tsx` đã dùng
`.click()` trực tiếp). **Không** cài thêm gói: thêm dependency vào `package.json` ở một
cây đang có ba session sửa là chồng việc, và ca dùng ở đây không cần nó.

**Phòng thủ đã thêm vào harness:** mọi lần chạy đột biến khẳng định `numTotalTests >=
79`; tổng tụt nghĩa là suite không nạp được, và harness in `⚠ HARNESS` thay vì báo "0 đỏ".

### 2. [Rule 1 — Bug] Matcher `jest-dom` không tồn tại — và nó **đỏ**, không xanh

`toBeInTheDocument`, `toBeDisabled`, `toHaveValue`, `toBeChecked` đều **không** có
(`src/test/setup.ts` chỉ gọi `cleanup()`, không import `@testing-library/jest-dom`).

```
Error: Invalid Chai property: toBeInTheDocument
```

Chuyển sang đọc thuộc tính DOM thật (`hasAttribute('disabled')`,
`(el as HTMLTextAreaElement).value`, `toBeTruthy()` / `toBeNull()`) — khuôn
`ChangeList.test.tsx`. Lớp lỗi này **tự báo**, khác lỗi #1 ở trên.

### 3. [Rule 1 — Bug] Một test của tôi đỏ vì `setState` ngoài `act`, **không** phải lỗi component

Test M9 (`cảnh báo hiện rồi, nút VẪN BẤM ĐƯỢC`) đỏ ở phép khẳng định nút không bị vô hiệu.

**Tôi không sửa test cho hết đỏ trước khi biết nguyên nhân** — đó là cách một cổng thật
bị vứt đi. Chèn chẩn đoán in trạng thái ngay tại chỗ đỏ:

```
CHAN-DOAN textarea     = ""
CHAN-DOAN store draft  = "sửa tiếp lần hai"
CHAN-DOAN isCommitting = false
CHAN-DOAN ly do        = Hãy viết thông điệp commit trước.
```

Store **đã có** chuỗi mới nhưng textarea còn rỗng ⇒ React **chưa render lại**, và nút
đang vô hiệu đúng theo nháp **cũ** (rỗng sau khi amend thành công). Tức component
**đúng**; phép đo sai vì `useCommitStore.setState` gọi ngoài `act` chưa flush.

**Sửa:** bọc `act`. Đây là cùng lớp lỗi với khiếm khuyết B của mutation #8 ở plan 03-04
(đo **trước** khi React flush), và nó đáng ghi lại vì lần này nó khiến một cổng đúng
**trông như** một lỗi sản phẩm.

### 4. [Rule 3 — Blocking] `src/styles/app.css` KHÔNG commit được — giống hệt wave 02

CSS đã chèn **trước** header `/* --- Thanh cuộn ---` theo chỉ đạo giữa chừng của điều
phối viên (không phải cuối tệp như brief đầu). Phép chèn được **kiểm bằng máy**: sau khi
ghi, tôi khẳng định nội dung cũ **giống hệt từng byte** khi gỡ khối của mình ra
(`67 059 → 70 161 byte, existing content byte-identical`).

Nhưng tệp **không commit được**: nó mang **3 dòng xoá** của session `git-plum-2a`
(`.cm-merge-a .cm-changedText` …), và git **không stage được một phần tệp**. Đã xác nhận
bằng `git diff --numstat`: `app.css` = 583 thêm / 3 xoá, trong khi phần của tôi là ~125
dòng và **0 xoá**.

**Hệ quả cho người sau:** khối CSS `.commit-box` / `.commit-message` /
`.commit-hook-error` nằm ở **dòng 2007** của `src/styles/app.css` trong cây làm việc và
**chưa commit**. Nó cộng dồn với khối `.change-list` của wave 02 cũng chưa commit.

### 5. `ipc.ts` — chỉ THÊM, 0 dòng xoá

`git diff --numstat` → `110  0  src/lib/ipc.ts`. Không luật cũ nào bị sửa, nên không
đụng phần `GitErrorCode` mà 04-02 vừa thêm.

## Điều KHÔNG kiểm chứng được — "có mã, chưa kiểm" cho checkpoint 04-05

happy-dom không tính CSS layout và không có cuộn thật (CONTEXT.md 3.4). Danh sách dưới
đây **có mã và có lý do**, nhưng **chưa ai nhìn thấy** trên Chromium:

| # | Tiêu chí bố cục | Trạng thái |
|---|---|---|
| 1 | Textarea đủ cao để soạn thông điệp nhiều dòng, kéo cao được | **có mã, chưa kiểm** (`min-height: 64px`, `resize: vertical`) |
| 2 | Khối `<pre>` lỗi hook **không đẩy nút commit ra khỏi khung** | **có mã, chưa kiểm** (`max-height: 160px` + `overflow-y: auto`) |
| 3 | Đầu ra hook nhiều dòng đọc được, thụt lề giữ nguyên, không tràn ngang | **có mã, chưa kiểm** (`white-space: pre-wrap`, `overflow-wrap: anywhere`) |
| 4 | Lý do vô hiệu không đè lên nút khi câu dài | **có mã, chưa kiểm** — phụ thuộc `min-width: 0` trên flex item |
| 5 | Vùng soạn không bị danh sách tệp dài ép co về 0 | **có mã, chưa kiểm** (`flex: 0 0 auto`) |
| 6 | Cảnh báo amend thấy được mà không che nút | **có mã, chưa kiểm** |
| 7 | Khối CSS mới không xung đột với luật của `git-plum-2a` **và** của wave 02 | **chưa kiểm** — ba khối chưa từng ở cùng một bản dựng đã commit |

**Và một mục nặng hơn bố cục:** toàn bộ vòng commit **chưa chạy thật một lần nào**, vì
nửa Rust chưa có. Tiêu chí thành công của plan ("gõ thông điệp → bấm commit → commit tồn
tại") là **chưa kiểm được**, không phải "đạt".

## Known Stubs

| Mục | Nơi | Ai nối |
|---|---|---|
| `ipc.createCommit` / `ipc.amendCommit` gọi command **chưa tồn tại** | `src/lib/ipc.ts` | **Task 1 (Rust), chưa làm** |
| 3 mã lỗi mới chưa có variant `GitError` bên Rust | `src/lib/ipc.ts` | **Task 1 (Rust), chưa làm** |
| `CommitBox` **chưa gắn vào `App.tsx`** | — | 04-05 (`App.tsx` đang trong tay session khác) |
| Khối CSS `.commit-*` chưa commit | `src/styles/app.css:2007` | chủ dự án / `git-plum-2a` |

`CommitBox` **chưa xuất hiện trên màn hình ứng dụng**. Plan này cố ý không gắn nó vào
bố cục (`<in_flight_files>`), nên không tiêu chí nào của nó kiểm được bằng mắt ở wave này.

## Threat Flags

| Flag | File | Mô tả |
|---|---|---|
| threat_flag: data-loss | `src/lib/commitDraft.ts` | Phép ghi trì hoãn khoá theo `repoId`. Bất biến chịu lực: `repoId` đóng gói **lúc lên lịch**. Giảm nhẹ đã cài + M14 ghim. **Chưa có** giới hạn kích thước nháp — một thông điệp rất dài ghi thẳng vào tệp store mỗi 300 ms; chưa có test cho ca đó |
| threat_flag: write-surface | `src/stores/commitStore.ts` | Đường gọi tới **bề mặt ghi git** mới (`create_commit`). `noVerify` đi từ một ô tick trong webview xuống thẳng `git commit --no-verify` — tức webview tắt được hook của repo. Đúng thiết kế (ràng buộc 2.4: công tắc tường minh), nhưng đáng nêu tên: phía Rust **không được** để đường nào khác thêm cờ đó |

## Self-Check: PASSED

Tệp đã tạo:
- `src/lib/commitDraft.ts` — FOUND
- `src/lib/commitDraft.test.ts` — FOUND
- `src/stores/commitStore.ts` — FOUND
- `src/stores/commitStore.test.ts` — FOUND
- `src/components/worktree/CommitBox.tsx` — FOUND
- `src/components/worktree/CommitBox.test.tsx` — FOUND

Commit (kiểm bằng `git rev-list`/`git show --stat`, **không** dùng `git log`):
- `496daa0` feat(04-03): persistent commit draft keyed by repoId… — FOUND, 5 tệp
- `f3ec22d` feat(04-03): CommitBox — soạn, no-verify, amend… — FOUND, 2 tệp

Không tệp nào của session khác nằm trong hai commit trên; `git show --stat` liệt kê
đúng 7 tệp, tất cả thuộc phạm vi được giao. `app.css` **cố ý** còn trong cây làm việc.
