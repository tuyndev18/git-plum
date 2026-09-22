---
phase: 04-commit-loop
plan: 02
subsystem: worktree-commands
tags: [WORK-01, WORK-02, ipc, zustand, component, pathspec-gate, R8]
requires:
  - "04-01 (RepoStatus, STATUS_ARGS, parse_status, status-cases fixture)"
provides:
  - "get_status / stage_files / unstage_files (lệnh ghi trả RepoStatus MỚI)"
  - "GitError::IndexLocked + mã lỗi index_locked ở cả hai phía"
  - "get_worktree_diff (KHÔNG cache, có chủ ý)"
  - "statusStore.byRepo + applyExternal (điểm vào cho watcher 04-04)"
  - "ChangeList: ba nhóm tệp bấm được, độc lập với trình xem diff"
affects:
  - "04-03 (vòng commit đọc statusStore), 04-04 (watcher gọi applyExternal), 04-05 (hàng WIP)"
tech-stack:
  added: []
  patterns:
    - "lệnh ghi trả trạng thái mới trực tiếp — ràng buộc ghim bằng KIỂU trả về"
    - "nhận biết index.lock bằng stderr của git, KHÔNG bằng phép kiểm tệp tồn tại"
    - "cổng đọc mã nguồn phải xoá khoảng trắng trước khi tìm (rustfmt ngắt dòng)"
key-files:
  created:
    - src-tauri/src/commands/worktree.rs
    - src-tauri/tests/worktree_commands.rs
    - src-tauri/tests/worktree_diff.rs
    - src/stores/statusStore.ts
    - src/stores/statusStore.test.ts
    - src/lib/ipc.status.test.ts
    - src/components/worktree/ChangeList.tsx
    - src/components/worktree/ChangeList.test.tsx
  modified:
    - src-tauri/src/commands/diff.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/error.rs
    - src-tauri/src/git/parsers/status.rs
    - src/lib/ipc.ts
    - scripts/fixtures/make-status-fixtures.sh
    - src/styles/app.css
decisions:
  - "index.lock nhận biết bằng stderr, không bằng fs::exists — kiểm tệp là một cuộc đua"
  - "unstage dùng `git restore --staged`, không `git reset HEAD` — reset hỏng ở repo chưa có commit"
  - "cổng đọc mã nguồn xoá khoảng trắng trước khi tìm — rustfmt ngắt `runner\\n.read(`"
  - "fixture status-cases thiếu hẳn nhóm Unstaged; đã sửa và ghim bằng phép khẳng định ba nhóm"
metrics:
  duration: ~4h
  completed: 2026-09-22
---

# Phase 4 Plan 02: get_status / stage_files / unstage_files + ChangeList Summary

Slice dọc đầu tiên dùng được của Phase 4 — React → IPC → Rust → `git` — cộng **hai phát
hiện**: repo mẫu `status-cases` của wave 1 **thiếu hẳn một trong ba nhóm** mà WORK-01
định nghĩa, và một cổng đọc mã nguồn của chính plan này **đỏ dù mã đúng** vì `rustfmt`
ngắt dòng giữa người nhận và tên phương thức.

## Số test đo được — lệnh nào cho số nào

| Phạm vi | Trước | Sau | Lệnh đã chạy |
|---|---|---|---|
| `cargo test --lib` | **242** | **258** (+16) | `cd src-tauri && cargo test --lib` |
| `cargo test --lib --tests` | **321 passed, 1 ignored** | ⚠️ xem dưới | `cd src-tauri && cargo test --lib --tests` |
| Test tích hợp mới (chạy trực tiếp binary) | — | **22** (14 + 8) | `./target/debug/deps/worktree_commands-*.exe` |
| Frontend | **441** | **494** | `npx vitest run --reporter=json` |
| `npx tsc --noEmit` | sạch | sạch | — |
| `npm run build` | sạch | sạch | — |

🔴 **Con số `--lib --tests` trước đây là 321, không phải 324.** Wave 1 **suy ra** 324
("82 test integration + 242 lib") và ghi rõ rằng đó là lập luận chứ không phải phép đo.
Tôi đo được **321 passed, 1 ignored** ở đầu plan này, lúc `git-plum.exe` tạm thời được
thả. Suy luận của wave 1 lệch 3.

### ⚠️ `--lib --tests` SAU plan này: CHƯA ĐO ĐƯỢC

Hai nguyên nhân độc lập, cả hai đều **không** phải lỗi test:

1. `git-plum.exe` chạy lại nhiều lần trong suốt phiên (đúng trap đã biết). Đã thử lại
   theo vòng có giãn cách; có lúc thoát được, có lúc không.
2. 🔴 **Đĩa C: đầy 100 % (0 byte trống)** ở cuối phiên, `target/` một mình 16 GB:
   ```text
   error: failed to write `...\.fingerprint\git-plum-...\test-bin-lanedist.json`
   Caused by: There is not enough space on the disk. (os error 112)
   ```
   Sau đó có người dọn và còn 5,8 GB, nhưng lúc đó cargo phải dựng lại từ đầu và lần
   chạy vượt quá thời gian của phiên này.

**Vì vậy tôi ghi số `--lib` là ĐÃ ĐO và `--lib --tests` là CHƯA ĐO.** Không suy ra tổng
rồi trình bày như đã đo — đó đúng là điều wave 1 bị nhắc và tôi vừa chứng minh là sai 3
đơn vị.

**22 test tích hợp mới thì ĐÃ ĐO**, bằng cách chạy thẳng binary test (`cargo` chỉ bị
chặn ở bước **relink binary chính**, phần biên dịch test target vẫn xong). Mỗi lần chạy
đều **đối chiếu dấu thời gian** binary với dấu thời gian mọi tệp nguồn trước — bài học
mục 3.7 của CONTEXT.md (chủ dự án từng kiểm một exe cũ hơn bản sửa 32 phút). Có **một
lần** phép đối chiếu đó cứu tôi khỏi một kết luận sai: xem "M5 suýt bị ghi nhầm" bên dưới.

## Bảng đột biến — M1–M11, đầu ra thật đã dán

Quy trình: sao lưu bản sạch → áp đột biến → **khẳng định tệp đã thật sự đổi** (nếu không
đổi thì harness báo LỖI, **không** ghi "0 test đỏ" như một kết quả) → chạy → ghi tên test
đỏ → phục hồi từ bản sao.

| # | Đột biến | Test ĐỎ (tên thật) | Số đỏ |
|---|---|---|---|
| **M1** | `stage_files` trả `()` thay vì `RepoStatus` | đỏ **lúc biên dịch**: 8 × `error[E0308]: mismatched types`, `could not compile (lib)` + `(lib test)` | biên dịch |
| **M2** | `statusStore.stage` gọi `refresh` sau khi stage | `đường stage KHÔNG gọi getStatus`, `stage ghi RepoStatus từ giá trị trả về của stageFiles`, `stage thất bại giữ nguyên status cũ và ghi lỗi`, `stage không cần refresh trước đó`, `tệp chuyển nhóm ngay từ giá trị trả về, không gọi lại getStatus` | **5** |
| **M3** | Bỏ `.arg("--")` khỏi lệnh `git add` | `moi_lenh_ghi_co_pathspec_deu_co_dau_gach_ngang_truoc_path` | **1** |
| **M4** | Bỏ chặn `paths` rỗng ở `stage_duong_dan` | `paths_rong_khong_sinh_lenh_git_add_nao` | **1** |
| **M5** | Bỏ `write_lock` (đường ghi dùng `runner.read`) | `duong_doc_dung_read_duong_ghi_dung_write` (lib), `stage_giu_write_lock_plat_03`, `unstage_giu_write_lock_plat_03` (tích hợp) | **3** |
| **M6** | Đường xử lý lock **xoá** `index.lock` | `khong_co_duong_nao_xoa_tep_lock` (lib), `index_lock_ton_tai_thi_that_bai_va_lock_van_con` (tích hợp) | **2** |
| **M7** | Nút stage bỏ `stopPropagation` | `bấm nút stage KHÔNG làm đổi tệp đang chọn` | **1** |
| **M8** | `ChangeList` hiện tiêu đề cho nhóm rỗng | `nhóm rỗng không hiện tiêu đề` | **1** |
| **M9** | 🔴 `ChangeList` vô hiệu nút stage khi diff lỗi | `getWorktreeDiff ném lỗi → nút stage VẪN bấm được và vẫn gọi stageFiles`, `không nút stage/unstage nào bị vô hiệu, kể cả khi diff hỏng`, `ChangeList không tự gọi getWorktreeDiff` | **3** |
| **M10** | `get_worktree_diff` ghi kết quả vào `DiffCache` | `duong_thu_muc_lam_viec_khong_cham_cache` (lib), `diff_thu_muc_lam_viec_khong_vao_cache`, `sua_tep_roi_goi_lai_cho_noi_dung_moi` (tích hợp) | **3** |
| **M11** | Đổi tên tham số IPC `repoId` → `repo_id` | `getStatus gọi get_status với { repoId }`, `mọi hàm dùng khoá camelCase repoId, không dùng repo_id` | **2** |
| M12 | Nới cổng pathspec `diff.rs` từ `== 5` thành `>= 5` | **không phải đột biến mã** — xem mục "Con số cổng pathspec" | — |

### M9 — đột biến quan trọng nhất của plan, đầu ra thật

M9 cài đúng thứ R8 cấm: cho `HangTep` tự nạp `getWorktreeDiff` rồi `disabled` nút khi
nó hỏng.

```text
× getWorktreeDiff ném lỗi → nút stage VẪN bấm được và vẫn gọi stageFiles 46ms
× không nút stage/unstage nào bị vô hiệu, kể cả khi diff hỏng 14ms
× ChangeList không tự gọi getWorktreeDiff 18ms
     Tests  3 failed | 17 passed (20)
```

Ba cổng đo **ba thứ khác nhau**, có chủ ý: một cổng đo nút đang chọn còn bấm được,
một cổng quét **mọi** nút (một cài đặt chỉ vô hiệu nút "của tệp đang xem" lọt qua cổng
đầu), và một cổng đo **cấu trúc** — `ChangeList` không được tự gọi `getWorktreeDiff`,
vì nếu nó tự nạp thì một lỗi render của diff nằm trong **cùng** cây con với nút stage.

### M10 — đầu ra chứng minh dữ liệu cũ là dữ liệu cũ thật

```text
thread 'sua_tep_roi_goi_lai_cho_noi_dung_moi' panicked at tests\worktree_diff.rs:187:5:
🔴 gọi lại sau khi sửa tệp phải cho nội dung MỚI. Đọc được: ["dong mot", "dong hai", "DA SUA", "dong ba"]
```

Tệp đã được ghi đè bằng `SUA LAN HAI HOAN TOAN KHAC`, nhưng với cache bật thì lời gọi
thứ hai trả lại `DA SUA` — nội dung của lần trước. Đây là ca CONTEXT.md 2.2 mô tả, đo
được chứ không suy luận.

### M6 — đầu ra chứng minh tệp lock bị xoá thật

```text
thread 'index_lock_ton_tai_thi_that_bai_va_lock_van_con' panicked at tests\worktree_commands.rs:317:5:
🔴 `index.lock` phải VẪN CÒN sau khi thất bại. Xoá nó là phá repo của người dùng đang
chạy `git rebase` ở terminal (CONTEXT.md 2.3, R3).
```

### M1 — đỏ lúc biên dịch, đúng như plan cho phép

```text
error[E0308]: mismatched types      (× 8)
error: could not compile `git-plum` (lib) due to 4 previous errors
error: could not compile `git-plum` (lib test) due to 4 previous errors
```

## 🔴 M5 suýt bị ghi nhầm là "0 test đỏ" — và thứ đã cứu nó

Lần chạy M5 đầu tiên, hai test PLAT-03 tích hợp cho **xanh**:

```text
test result: ok. 14 passed; 0 failed
```

Nếu tôi dừng ở đó thì bảng đột biến sẽ ghi "M5: chỉ 1 test đỏ ở lib, hai test tích hợp
KHÔNG phân biệt được" — và đó là một kết luận **sai**. Chạy riêng hai test đó lại cho:

```text
test stage_giu_write_lock_plat_03 ... FAILED
test unstage_giu_write_lock_plat_03 ... FAILED
```

**Nguyên nhân:** lần chạy "xanh" dùng **binary test cũ**. `cargo` dựng lại được lib
(mutation đã vào) nhưng bước relink binary chính bị `git-plum.exe` chặn, nên nó dùng
lại file `.exe` test từ lần dựng **trước** mutation. Tức tôi đã đo một bản dựng cũ hơn
bản sửa — **đúng** lỗi mục 3.7 của CONTEXT.md, chỉ khác là lần này nó rơi vào một
mutation chứ không vào một bản giao cho người dùng.

**Quy tắc rút ra, và tôi đã áp cho mọi phép đo sau đó:** trước khi tin một kết quả test
tích hợp trong môi trường này, **đối chiếu dấu thời gian của binary với dấu thời gian
của mọi tệp nguồn**. Một binary cũ hơn nguồn là một phép đo vô giá trị, và nó **trông
giống hệt** một phép đo hợp lệ.

## Con số cổng pathspec — cũ → mới (M12 bằng lời)

Cổng `moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path` của `src-tauri/src/commands/diff.rs`
đã được **cập nhật**, **không** nới lỏng:

| Phép khẳng định | Cũ (hết Phase 3) | Mới (04-02) | Vì sao đổi |
|---|---|---|---|
| `so_moc` (số mốc `PATHSPEC_SAU_DAU_GACH`) | `== 5` | `== 7` | `lay_diff_thu_muc_lam_viec` mang **hai** mốc: lệnh `--numstat` của cổng nhị phân và lệnh diff thật |
| `so_lenh_co_pathspec` (số lệnh git mang pathspec) | `== 4` | `== 6` | cùng hai lệnh trên |

Cả hai vẫn là **phép so bằng tuyệt đối**, không phải `>=`. Thông điệp thất bại của cả hai
giờ mang **lịch sử con số** (`5 → 7`, `4 → 6`) cộng câu giải thích vì sao không được nới —
để người rà plan sau đọc được ý định mà không phải tra lại lịch sử git.

**M12 không phải đột biến mã.** Nới `== 5` thành `>= 5` **không** làm test nào đỏ; đó là
cả điểm — nó là ca duy nhất trong bảng mà bảng không tự kiểm được, và nó được kiểm bằng
mắt người rà plan. Ghi lại ở đây theo đúng yêu cầu của plan: **các con số đã được cập
nhật, không nới lỏng.**

Cộng thêm một cổng pathspec **mới, độc lập** cho `worktree.rs`
(`moi_lenh_ghi_co_pathspec_deu_co_dau_gach_ngang_truoc_path`, `so_moc == 3`,
`so_lenh_co_pathspec == 2`), vì cổng của `diff.rs` đọc `include_str!("diff.rs")` và
**không** phủ được lệnh nằm ở tệp khác. M3 chứng minh nó phân biệt được.

## Ba lần chạy suite frontend — và một lần đỏ không quy được trách nhiệm

Plan đòi ≥ 3 lần liên tiếp, `rm -f .vitest/json/output.json` trước **mỗi** lần. Tôi chạy
**15** lần:

| Lần | Tổng | Đỏ |
|---|---|---|
| 1 (ngay sau khi viết xong) | 492 | 0 |
| 2, 3, 4 | 492 | 0 |
| 5 (sau bảng đột biến) | **494** | **2** |
| 6 → 15 | 494 | 0 |
| Chỉ 3 tệp test của plan này, 5 lần | 51 | 0 |

🔴 **Lần 5 có 2 test đỏ và tôi KHÔNG xác định được chúng là test nào.** Lần chạy đó dùng
reporter không in tên test đỏ, và 12 lần chạy sau đó không tái hiện được.

**Điều tôi biết:** tổng số test **đổi từ 492 sang 494 ở đúng lần đó** — tức một session
khác đang sửa tệp test của nó ngay lúc suite chạy. **Điều tôi không biết:** hai test đỏ
đó có phải của tôi không.

**Bằng chứng gián tiếp rằng chúng không phải của tôi:** 51 test của plan này chạy riêng
5 lần đều xanh, và chạy trong suite đầy đủ 14 lần đều xanh. Nhưng đó là bằng chứng gián
tiếp, **không phải phép loại trừ**. Ghi là **quan sát được một lần, chưa quy được trách
nhiệm** — không ghi là "suite ổn định".

## Deviations from Plan

### 1. [Rule 1 — Bug] 🔴 Repo mẫu `status-cases` thiếu hẳn nhóm `Unstaged`

**Tìm thấy ở:** Task 1, khi `get_status_tren_fixture_cho_ca_ba_nhom` chạy lần đầu.

Fixture của wave 1 `git add` sau **mỗi** lần sửa, nên **mọi** bản ghi theo dõi đều là
`R.` hoặc `M.` — nhóm `Unstaged`, một trong **ba** nhóm mà WORK-01 định nghĩa, **hoàn
toàn vắng mặt**:

```text
2 R. ... a_new.txt      <- Staged
1 M. ... m_one.txt      <- Staged
1 M. ... n_two.txt      <- Staged
? b_café.txt            <- Untracked
? z with space.txt      <- Untracked
? z_untracked.txt       <- Untracked
```

Test đơn vị của 04-01 không thấy vì chúng chạy trên **byte tự dựng**, nơi ca `.M` có test
riêng. Khoảng trống chỉ lộ ra khi một lệnh `git status` **thật** chạy trên fixture và có
ai đó đòi đủ ba nhóm.

**Đây là họ hàng của lỗi #7** mà CONTEXT.md vừa ghi: fixture có **hình dạng đúng cho câu
hỏi cũ** (một bản ghi dạng `2` đứng trước ≥ 2 bản ghi khác — và nó vẫn đúng) nhưng
**không phủ câu hỏi mới**.

**Sửa:** thêm `p_mm.txt` (XY = `MM`) — nó cấp nhóm đang thiếu **và** ca "một tệp sinh
HAI phần tử" mà `wip_counts` phải đếm là một.

**Vì sao không sửa bằng cách bỏ một `git add` ở mục 4:** hai bản ghi đó (`m_one.txt`,
`n_two.txt`) là **chính** hai bản ghi đứng sau bản ghi dạng `2`, và chúng là **dữ liệu
chịu lực** của ca M1 mà wave 1 vừa phải sửa một lần vì nó không phân biệt được. Đụng vào
chúng là đụng vào cổng đắt nhất của phase.

**Phép khẳng định ba nhóm đã được chứng minh phân biệt được**, không phải được chỉ định:
tôi chạy nó trên bản ghi của fixture **cũ** trước khi sinh lại, và nó báo đúng
`NO UNSTAGED RECORD -> gate would FAIL`.

Commit `604f922`.

### 2. [Rule 1 — Bug] Test wave 1 viết cứng `modified == 3`, vỡ khi fixture thêm tệp

`wip_counts_tren_repo_mau_that` đỏ sau khi sửa fixture, dù `wip_counts` **hoàn toàn
đúng**. Chú thích của chính test nói ý định là *"đếm được từ chính `entries` để test
không vỡ khi script thêm một tệp"* — phép khẳng định thứ nhất làm vậy, phép thứ hai thì
viết cứng `3`.

Một test vỡ vì **dữ liệu fixture đổi** trong khi thứ nó kiểm vẫn đúng dạy người đọc sửa
con số cho hết đỏ. Lần sau một lỗi thật cũng sẽ được "sửa" như vậy.

**Sửa:** suy `modified` từ `entries` — và **kiểm chứng rằng làm vậy không làm cổng yếu
đi**: áp lại đột biến M8 của wave 1 (`wip_counts` đếm phần tử thay vì đường dẫn duy nhất)
và test này **vẫn đỏ** (4 đỏ tổng). Thêm một phép khẳng định **tiền đề** rằng fixture
thật sự có tệp xuất hiện ở hai nhóm — thiếu nó thì hai con số khớp nhau một cách vô
nghĩa trên một fixture không có ca đó.

Commit `c354158`.

### 3. [Rule 3 — Blocking] Cổng đọc mã nguồn đỏ dù mã đúng — `rustfmt` ngắt dòng

`duong_doc_dung_read_duong_ghi_dung_write` tìm chuỗi `"runner.read"` và **đỏ**, vì
`rustfmt` viết:

```rust
let ra = runner
    .read(
```

— chuỗi `runner.read` **không tồn tại** trong tệp.

**Đây là một cổng không-thể-fail đi theo hướng ngược lại** với sáu cổng trước của dự án:
thay vì xanh khi đáng đỏ, nó đỏ khi đáng xanh. Nhưng nó cùng một gốc — **đo cách trình
bày thay vì đo bất biến**.

**Phép khẳng định tiền đề đã bắt được nó** ("tiền đề: lay_trang_thai phải gọi runner")
thay vì để cổng xanh. Đó đúng là công dụng mà CONTEXT.md 3.1 đòi ở quy tắc "cổng phải
khẳng định tiền đề của mình", và đây là lần đầu quy tắc đó trả công trong dự án này.

**Cách sai để sửa:** đổi mã nguồn cho vừa cổng (viết một dòng, `#[rustfmt::skip]`). Làm
vậy là để một cổng quyết định cách định dạng mã, và lần `cargo fmt` sau sẽ phá lại.
**Cách đã dùng:** xoá **hết khoảng trắng** trước khi tìm (`khong_khoang_trang`), và tìm
`"runner.read("` kèm dấu mở ngoặc.

### 4. [Rule 3 — Blocking] `src/styles/app.css` KHÔNG nằm trong commit nào của plan này

CSS đã được **thêm ở cuối** tệp trong cây làm việc, đúng như `<in_flight_files>` yêu cầu.
Nhưng nó **không commit được**: tệp mang thay đổi chưa commit của session `git-plum-2a`
(sửa `.cm-merge-a .cm-changedText`, `.cm-merge-b .cm-changedText`,
`.cm-deletedChunk .cm-deletedText`), và **git không stage được một phần tệp**.

Tôi đã `git add src/styles/app.css` một lần, thấy `372 insertions, 3 deletions` trong khi
khối của tôi chỉ ~120 dòng, và `git restore --staged` ngay. **Không commit nào của plan
này chứa công việc của session khác** — đã xác nhận bằng `git status --porcelain=v2`.

**Hệ quả cho người sau:** khối CSS `.change-list` / `.change-row` / `.change-stage-button`
nằm ở cuối `src/styles/app.css` trong cây làm việc và **chưa được commit**. Chủ dự án
hoặc session `git-plum-2a` cần commit nó cùng phần của họ.

### 5. `wip_counts` / `WipCounts` KHÔNG xuất hiện trong JSON gửi sang TypeScript

Plan liệt kê `WipCounts` trong danh sách kiểu phải thêm vào `ipc.ts`. Đã thêm **kiểu**,
nhưng cần nói rõ: `RepoStatus::wip_counts()` bên Rust là một **phương thức**, không phải
một trường, nên serde **không** serialize nó. `RepoStatus` qua IPC có đúng ba khoá:
`branch`, `entries`, `hasConflicts`.

Đó là thiết kế **đúng** của 04-01 (ràng buộc "không lệnh git thứ hai để đếm" được bảo đảm
bằng *kiểu*), nhưng nó nghĩa là plan 04-05 phải tính `WipCounts` **ở phía TypeScript** từ
`entries`, hoặc thêm một trường dẫn xuất khi serialize. **Kiểu `WipCounts` hiện chưa có
người dùng** — nó chờ 04-05.

## Những gì plan nói mà tôi thấy khác

1. **`cargo test --lib --tests` của 04-01 là 321, không phải 324.** Wave 1 suy ra 324 và
   ghi rõ đó là suy luận. Phép đo thật lệch 3.
2. **Frontend baseline là 441/164 suite, không phải 435/34 tệp.** 435 là số lúc CONTEXT.md
   được viết; 34 là số **tệp** test, còn 164 là số **suite**. Cuối plan: 494.
3. **Cổng pathspec của `diff.rs` cắt từ `lay_diff_tep` tới `#[cfg(test)]`**, tức nó phủ cả
   `lay_lich_su_tep` — không chỉ một hàm. Nên đặt hàm mới ở đâu trong vùng đó cũng làm
   cả hai con số đổi, đúng như plan dự đoán.

## Điều KHÔNG kiểm chứng được — "có mã, chưa kiểm" cho checkpoint 04-05

happy-dom không tính CSS layout và không có cuộn thật (CONTEXT.md 3.4). Danh sách dưới
đây **có mã và có lý do**, nhưng **chưa có ai nhìn thấy** trên Chromium:

| # | Tiêu chí bố cục | Trạng thái |
|---|---|---|
| 1 | Ba nhóm tệp không tràn khỏi khung, cuộn được khi nhiều tệp | **có mã, chưa kiểm** (`overflow-y: auto`, `min-height: 0`) |
| 2 | Tiêu đề nhóm dính (`position: sticky`) khi cuộn | **có mã, chưa kiểm** |
| 3 | Tên tệp dài bị cắt bằng `…` mà **không đẩy nút stage ra ngoài khung** | **có mã, chưa kiểm** — phụ thuộc `min-width: 0` trên flex item, đúng loại luật mà happy-dom không tính |
| 4 | Chiều cao hàng đọc được, không chen chúc | **có mã, chưa kiểm** |
| 5 | Nút stage/unstage đủ lớn để bấm trúng bằng chuột | **có mã, chưa kiểm** |
| 6 | Khối CSS mới không xung đột với luật của `git-plum-2a` trong cùng tệp | **chưa kiểm** — hai khối chưa từng ở cùng một bản dựng đã commit |

Riêng mục 6 đáng chú ý: khối CSS của tôi và khối của session kia **chưa bao giờ được
kiểm cùng nhau**, và cả hai đều chưa commit.

## Điều đã kiểm về môi trường

- **`cargo test --lib` chạy được suốt phiên**; `--lib --tests` bị chặn bởi hai nguyên
  nhân khác nhau ở hai thời điểm (exe đang chạy, rồi đĩa đầy).
- 🔴 **Đĩa C: đầy 100 %** lúc cuối phiên, `target/` 16 GB. Sau khi ai đó dọn còn 5,8 GB.
  Đây là một trap **mới**, chưa có trong CONTEXT.md — đáng ghi vào mục 3.6.
- **Binary test có thể CŨ hơn nguồn** khi relink bị chặn, và một lần chạy trên binary cũ
  **trông giống hệt** một phép đo hợp lệ. Luôn đối chiếu dấu thời gian.
- **`rtk` lọc đầu ra `cargo test` và cả `git diff`/`grep`.** Dùng `rtk proxy` khi cần
  tên từng test hay nội dung diff; `> tệp` cho dòng tổng.
- **Không dùng `git log`** ở bất kỳ đâu trong plan này; dùng `git rev-list --format`.
- **Không đụng tệp nào của session khác.** Mọi commit stage theo đường dẫn tường minh;
  không `git add -A`, không `git add .`.

## Known Stubs

Không có stub cắm cứng. Hai mục chưa nối dây, **có chủ ý**, và cả hai là điểm vào đã khai
sẵn cho wave sau:

| Mục | Nơi | Ai nối |
|---|---|---|
| `statusStore.applyExternal` | `src/stores/statusStore.ts` | 04-04 (watcher) |
| `ChangeList` chưa được gắn vào `App.tsx` | — | 04-03/04-05 (vùng soạn commit) |
| Kiểu `WipCounts` chưa có người dùng | `src/lib/ipc.ts` | 04-05 (hàng WIP) |

`ChangeList` **chưa xuất hiện trên màn hình ứng dụng**. Plan này không yêu cầu gắn nó vào
bố cục, và `App.tsx` đang có thay đổi chưa commit của session khác nên không được sửa.
Nghĩa là **tiêu chí thành công "người dùng mở repo và thấy ba nhóm tệp" chưa kiểm được
bằng mắt** — nó chờ wave sau gắn component vào cây.

## Threat Flags

| Flag | File | Mô tả |
|---|---|---|
| threat_flag: write-surface | `src-tauri/src/commands/worktree.rs` | **Bề mặt GHI đầu tiên của ứng dụng.** Trước plan này mọi command đều chỉ đọc. `stage_files`/`unstage_files` nhận `Vec<String>` **từ webview** và đưa thẳng làm pathspec. Giảm nhẹ đã cài: `--` trước mọi pathspec (có cổng + test cho `-rf` và `main`), chặn mảng rỗng (`git add` không pathspec stage cả cây), `repo_id` phải là repo **đã mở** qua `open_repository` nên không mở được thư mục tuỳ ý, và `write_lock` giữ suốt thời gian chạy. **Chưa có** giới hạn số phần tử trong `paths` — một mảng 100k phần tử từ webview sẽ dựng một dòng lệnh rất dài; git sẽ từ chối, nhưng ca đó chưa có test. |
| threat_flag: fs-read | `src-tauri/src/commands/diff.rs` | `lay_diff_thu_muc_lam_viec` gọi `tokio::fs::metadata(repo.path.join(path))` — lần đầu một `path` từ webview được nối vào đường dẫn hệ tệp **ngoài** git. `..` trong `path` sẽ trỏ ra ngoài repo. Hậu quả hiện tại giới hạn ở việc **đọc kích thước** (kết quả chỉ dùng để so ngưỡng, không trả về), và lệnh git sau đó vẫn bị `--` chặn, nhưng đây là bề mặt mới đáng nêu tên. |

## Self-Check: PASSED

Tệp đã tạo:
- `src-tauri/src/commands/worktree.rs` — FOUND
- `src-tauri/tests/worktree_commands.rs` — FOUND
- `src-tauri/tests/worktree_diff.rs` — FOUND
- `src/stores/statusStore.ts` — FOUND
- `src/stores/statusStore.test.ts` — FOUND
- `src/lib/ipc.status.test.ts` — FOUND
- `src/components/worktree/ChangeList.tsx` — FOUND
- `src/components/worktree/ChangeList.test.tsx` — FOUND

Commit (kiểm bằng `git rev-list --format`, **không** dùng `git log`):
- `00c206f` test(04-02): add failing tests… — FOUND
- `8ef5374` feat(04-02): add get_status/stage_files/unstage_files… — FOUND
- `b892cfd` feat(04-02): add the worktree diff path… — FOUND
- `fbfb52a` feat(04-02): add statusStore and the three-group ChangeList — FOUND
- `604f922` fix(04-02): the status fixture had no unstaged group at all — FOUND
- `c354158` test(04-02): derive wip_counts fixture assertion… — FOUND
