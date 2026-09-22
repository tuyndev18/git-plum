---
phase: 04-commit-loop
plan: 03
part: rust-only
subsystem: commit-commands
tags: [WORK-08, WORK-09, hooks, no-verify, amend, cleanup-whitespace]
requires:
  - "04-02 (chay_lenh_ghi_co_thu_lai, lay_trang_thai, repo_cua, GitError::IndexLocked)"
  - "04-01 (RepoStatus, BranchInfo.ahead là Option, parse_status)"
provides:
  - "create_commit(repoId, message, noVerify) -> RepoStatus"
  - "amend_commit(repoId, message, noVerify) -> AmendResult { status, wasPushed }"
  - "GitError::{EmptyCommitMessage, NothingToCommit, HookRejected} + ba mã lỗi"
  - "chay_lenh_ghi_co_thu_lai_phan_loai (đường thử lại dùng chung, phân loại tiêm vào)"
  - "repo mẫu hook-reject / hook-msg-reject + require_hook_fixture"
affects:
  - "04-03 phần frontend (commitStore gọi hai IPC này), 04-05 (CommitBox)"
tech-stack:
  added: []
  patterns:
    - "phân loại lỗi đọc CẢ hai luồng — git chia thông báo giữa stdout và stderr"
    - "cờ cảnh báo đo TRƯỚC thao tác ghi, không sau"
    - "fixture phải làm dữ liệu CÓ HẠI, không chỉ đúng hình dạng"
key-files:
  created:
    - src-tauri/src/commands/commit.rs
    - src-tauri/tests/commit_commands.rs
  modified:
    - src-tauri/src/error.rs
    - src-tauri/src/commands/worktree.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/testing/mod.rs
    - scripts/fixtures/make-status-fixtures.sh
decisions:
  - "was_pushed đo TRƯỚC amend — plan nói 'cùng lời gọi status' nhưng amend làm ahead nhảy 0->1"
  - "tách chay_lenh_ghi_co_thu_lai_phan_loai thay vì sao chép đường thử lại"
  - "test --cleanup đặt commit.cleanup=strip để dữ liệu có hại; mặc định không phân biệt được"
  - "M3 không có test hành vi nào phân biệt được — cổng đọc mã nguồn là cổng DUY NHẤT"
metrics:
  duration: ~4h
  completed: 2026-09-22
---

# Phase 4 Plan 03 (phần Rust): create_commit / amend_commit Summary

> 🔴 **Đây là NỬA RUST của plan 04-03.** Nửa frontend (`commitDraft.ts`,
> `commitStore.ts`, `CommitBox.tsx`, `ipc.ts`, `app.css`) do một agent khác chạy
> **song song** và ghi vào `04-03-SUMMARY.md`. Task 2 và Task 3 của plan là thuần
> frontend — tôi **bỏ qua** chúng, xem mục "Task đã bỏ qua".

Vòng commit phía Rust, cộng **ba phát hiện đo được** mà mỗi cái đổi cách viết mã so
với plan: git chia thông báo lỗi giữa hai luồng theo cách ngược trực giác, `was_pushed`
đo sau amend thì luôn sai, và cờ `--cleanup=whitespace` **không** phân biệt được gì
trên một repo cấu hình mặc định.

## Hợp đồng IPC đã cài — để đối chiếu với mock của agent frontend

```ts
// Tham số qua IPC là camelCase (serde rename_all = "camelCase" ở lớp command).
invoke('create_commit', { repoId: string, message: string, noVerify: boolean })
  -> RepoStatus                       // { branch, entries, hasConflicts }

invoke('amend_commit',  { repoId: string, message: string, noVerify: boolean })
  -> AmendResult                      // { status: RepoStatus, wasPushed: boolean }
```

**Khớp đúng chữ ký plan mô tả.** Một điểm plan không nói mà tôi phải chọn:
`amend_commit` **cũng** nhận `noVerify` (plan chỉ viết `amend_commit(...)`). Có test
ghim rằng amend chạy hook mặc định — một đường amend tự thêm `--no-verify` sẽ lọt qua
mọi test của `create_commit`.

**Ba mã lỗi mới** cho `GitErrorCode` phía TypeScript:

| mã | khi nào | thông báo nói gì |
|---|---|---|
| `empty_commit_message` | thông điệp rỗng/chỉ khoảng trắng | chặn **trước** khi chạy git |
| `nothing_to_commit` | index rỗng | *"Chọn tệp để stage trước"* |
| `hook_rejected` | hook từ chối, hoặc nguyên nhân chưa biết | **nguyên văn** đầu ra hook |

## Số test đo được — lệnh nào cho số nào

| Phạm vi | Trước (đo đầu phiên) | Sau | Lệnh |
|---|---|---|---|
| `cargo test --lib` | **264** | **277** (+13) | `cd src-tauri && cargo test --lib` |
| `cargo test --lib --tests` | ⚠️ **không đo được** | **332 passed, 1 failed** | `cd src-tauri && cargo test --lib --tests` |
| Test tích hợp mới | — | **15** | `cargo test --test commit_commands` |
| `cargo clippy --all-targets` | 1 warning (`diff.rs`) | **2 warning, KHÔNG cái nào của tôi** | — |

🔴 **Baseline 264, KHÔNG phải 242.** Brief nói "Rust là 242 sau wave 1, đo lại vì wave 2
chưa đo". Đo được **264** — wave 2 thêm 22 test lib mà SUMMARY của nó ghi là 258. Lệch 6
so với con số nó ghi; tôi không truy lý do, chỉ ghi con số tôi **đo**.

### ⚠️ `--lib --tests` TRƯỚC: không đo được, và lý do là MỚI

Hai lần chạy đầu phiên đều chết, **không** phải vì test:

```text
lần 1: rustc crash — handle_alloc_error / failed to mmap file
       (hết RAM lúc biên dịch: ba phiên cargo chạy đồng thời)
lần 2: error: linking with `link.exe` failed: exit code 1318 / 1140
       crate `serde_core`/`anyhow`/`muda` required in rlib format, but was not found
       (artifact hỏng do lần crash trước để lại)
```

Đây là một trap **khác** với hai trap đã ghi trong CONTEXT.md 3.6 (exe đang chạy, đĩa
đầy): lần này là **hết RAM lúc biên dịch**, và nó để lại `target/` ở trạng thái hỏng
làm lần chạy kế tiếp thất bại với một thông báo **hoàn toàn khác** (linker). Ai gặp
thông báo linker mà đi tìm lỗi phụ thuộc sẽ tìm nhầm chỗ.

`git-plum.exe` **không** chạy suốt phiên này (`tasklist` xác nhận), nên `--lib --tests`
đo được ở cuối phiên: **332 passed, 1 failed**.

### 🔴 Test đỏ duy nhất KHÔNG phải của tôi

`snapshot_wide` (`tests/graph_fixtures.rs`) đỏ vì `src-tauri/src/graph/types.rs` mang
thay đổi **chưa commit** của session khác — đúng thay đổi `MAX_VISIBLE_LANES` 13 → 20 mà
CONTEXT.md mục 2.6 ghi lại. Snapshot `graph_fixtures__wide.snap` vẫn là bản của cap 13:

```text
37 │-r36 lane=0 ... trunc=1
38 │-r37 lane=13 color=0 ...      <- lane 13+ giờ không còn bị truncate
```

Xác nhận bằng `git status --porcelain -- src-tauri/src/graph/types.rs` → ` M`. **Không
đụng vào nó**; người sở hữu thay đổi cap phải chạy `cargo insta accept` cùng phần của họ.
Tôi đã xoá tệp `.snap.new` mà lần chạy test sinh ra để không để rác lại trong cây.

## 🔴 Bằng chứng hook THẬT SỰ CHẠY — đo, không giả định

Script fixture **tự khẳng định tiền đề của nó**: chạy `git commit` thật rồi đòi ba
điều. Thiếu bất kỳ điều nào → `exit 1`, không phải in cảnh báo rồi thoát 0.

```text
[7] repo mẫu có hook từ chối
    hook-reject: hook 'pre-commit' CHẠY, chặn commit, index giữ nguyên. ĐẠT.
      đầu ra hook: HOOK-PRE-COMMIT-REJECTED (stdout) HOOK-PRE-COMMIT-REJECTED (stderr)
    hook-msg-reject: hook 'commit-msg' CHẠY, chặn commit, index giữ nguyên. ĐẠT.
      đầu ra hook: HOOK-COMMIT-MSG-REJECTED (stdout) HOOK-COMMIT-MSG-REJECTED (stderr)
```

### Nguyên văn thông báo hook mà test đọc được

Đo trực tiếp bằng `git commit` trên repo mẫu, hai luồng tách riêng:

```text
$ git commit -m "probe"          # trong target/fixtures/status-cases/hook-reject
EXIT=1
STDOUT:
                                  <- RỖNG
STDERR:
HOOK-PRE-COMMIT-REJECTED (stdout)
HOOK-PRE-COMMIT-REJECTED (stderr)

$ git commit --no-verify -m "probe noverify"   # CÙNG repo
EXIT=0
```

**Hook in ra stdout của nó vẫn tới ta qua stderr của git.** Đây là phát hiện số 1 dưới đây.

## Ba phát hiện đo được — mỗi cái đổi mã so với plan

### 1. 🔴 git chia thông báo giữa hai luồng theo cách ngược trực giác

Đo trên git 2.54.0.windows.1, `rtk proxy` để lấy đầu ra thô:

| nguyên nhân | exit | stdout | stderr |
|---|---:|---|---|
| hook `pre-commit` từ chối | 1 | **rỗng** | đầu ra hook, **cả hai luồng của hook** |
| không có gì để commit | 1 | `"nothing to commit, working tree clean"` | **rỗng** |
| thông điệp rỗng | 1 | **rỗng** | `"Aborting commit due to empty commit message."` |

Hai hệ quả, và cả hai đều là bẫy:

* Một bộ phân loại chỉ đọc **stderr** thấy chuỗi **rỗng** cho ca "không có gì để
  commit" và rơi vào nhánh sai — người dùng nhận một thông báo vô nghĩa cho ca **thường
  gặp nhất** của cả vòng commit.
* Một bộ phân loại chỉ đọc **stdout** nuốt **toàn bộ** thông báo của hook, im lặng.

`ca_hai_luong()` ghép cả hai trước khi so khớp. Mã thoát **không** phân biệt được gì —
mọi ca đều thoát 1.

### 2. 🔴 `was_pushed` đo SAU amend thì luôn SAI — plan sai ở chỗ này

Plan viết: *"`was_pushed` suy từ `BranchInfo` của **cùng** lời gọi status"* và
`<action>` đặt nó sau khi amend chạy. Tôi cài đúng như vậy, rồi test đỏ:

```text
thread 'amend_tren_commit_da_push_canh_bao_nhung_van_chay' panicked at
tests\commit_commands.rs:608:5:
🔴 cảnh báo phải BẬT: upstream có và ahead == Some(0) nghĩa là commit vừa sửa
đã nằm trên remote, tức ta vừa viết lại thứ người khác có thể đã thấy
```

**Nguyên nhân:** amend **viết lại** commit gần nhất, nên ngay sau nó nhánh cục bộ
**phân kỳ** khỏi upstream — `ahead` nhảy từ `Some(0)` lên `Some(1)`, và `da_push` trả
`false` **đúng ở ca duy nhất phải cảnh báo**. Cảnh báo sẽ **không bao giờ hiện**.

Câu hỏi mà cảnh báo trả lời là *"commit tôi **sắp** viết lại có nằm trên remote
không?"* — một câu hỏi về trạng thái **trước** thao tác. Đo sau là đo nhầm câu hỏi.

**Sửa:** đọc status **trước** amend để tính cờ, vẫn đọc lại **sau** để trả trạng thái
mới. Ràng buộc "không lệnh git thứ hai chỉ để đếm" vẫn được tôn trọng: cả hai lời gọi
status là lời gọi ta vốn đã phải chạy.

**Đây là lỗi không thể tìm ra bằng đọc mã.** Nó chỉ lộ ra khi có một upstream thật, và
test phải `git init --bare` + `push -u` để dựng ca đó.

### 3. 🔴 `--cleanup=whitespace` KHÔNG phân biệt được gì ở cấu hình mặc định

`docs/02-phase4-commit-notes.md` bàn giao ràng buộc này và đề nghị kiểm bằng *"commit
một thông điệp có dòng bắt đầu bằng `#`, xác nhận nó còn"*. **Phép kiểm đó vô dụng.**
Đo bốn ca:

```text
-F -, không cờ, cấu hình mặc định      -> dòng `#123` CÒN   (mặc định của -F đã là whitespace)
-F -, KHÔNG cờ, commit.cleanup=strip   -> dòng `#789` MẤT   🔴
-F -, CÓ cờ,    commit.cleanup=strip   -> dòng `#999` CÒN
-m,   không cờ, cấu hình mặc định      -> dòng `#456` CÒN
```

Khi thông điệp đến từ **tệp/stdin**, mặc định của git **đã là** `whitespace`. Cờ không
đổi gì — **cho tới khi** người dùng có `commit.cleanup=strip` trong cấu hình.

Nghĩa là một test "dòng `#` còn nguyên" chạy trên repo mặc định **xanh ở cả hai bên đột
biến**: đúng lỗi #7 của CONTEXT.md 3.1 (hình dạng đúng, **dữ liệu vô hại**). Test của
tôi vì vậy **đặt `commit.cleanup=strip`** vào repo mẫu để dữ liệu trở nên **có hại**.

Đỏ sau khi bỏ cờ, và đầu ra chứng minh dòng bị **xoá thật**:

```text
🔴 dòng bắt đầu bằng `#` phải CÒN NGUYÊN. Repo này có `commit.cleanup=strip`, nên
thiếu cờ `--cleanup=whitespace` thì git XOÁ dòng đó — im lặng...
Đọc được: "dong subject\n\n"
```

## Bảng đột biến — đầu ra thật đã dán

Harness **khẳng định tiền đề**: so `md5sum` trước/sau; tệp không đổi → báo LỖI HARNESS,
**không** ghi "0 test đỏ" như một kết quả. Mọi lần chạy đều phục hồi từ bản sạch và
`md5sum` xác nhận đã về đúng bản gốc.

| # | Đột biến | Test ĐỎ (tên thật) | Số đỏ |
|---|---|---|---|
| **M1** | `if no_verify` → `if true` | `chi_mot_duong_them_no_verify` (nguồn) | **1** |
| **M1b** | `let no_verify = true;` (lọt cổng nguồn) | `cung_repo_no_verify_false_bi_chan_true_thi_qua`, `hook_commit_msg_tu_choi_cung_giu_nguyen_van`, `amend_cung_chay_hook_mac_dinh` | **3** |
| **M2** | `let no_verify = false;` (bỏ hẳn nhánh) | ba test như M1b | **3** |
| **M3** | `-F -` → `-m <message>` | `thong_diep_di_qua_stdin_khong_qua_dau_m` (nguồn) | **1** |
| **M3b** | M3, chỉ chạy test **hành vi** | 🔴 **KHÔNG CÓ** — xem dưới | **0** |
| **M4** | bỏ phép chặn thông điệp rỗng ở chỗ gọi | `thong_diep_rong_khong_sinh_lenh_git_nao` | **1** |
| **M5** | lỗi hook trả `"commit that bai"`, bỏ nguyên văn | `phan_loai_doc_ca_stdout_lan_stderr` (lib) + ba test hook (tích hợp) | **4** |
| **M9** | `if was_pushed { return Err }` | `khong_co_duong_nao_chan_amend_vi_da_push` (nguồn) | **1** |
| **M9b** | `match was_pushed { true => Err }` (lọt cổng nguồn) | `amend_tren_commit_da_push_canh_bao_nhung_van_chay` | **1** |
| **M10** | bỏ `--amend` | `amend_doi_sha_nhung_khong_tang_so_commit`, `amend_khong_upstream_thi_was_pushed_false`, `amend_tren_commit_da_push_...`, `amend_cung_chay_hook_mac_dinh` | **4** |
| **M11** | `was_pushed` true khi không upstream | `was_pushed_phan_biet_khong_upstream_voi_da_dong_bo` (lib) | **1** |
| **M11b** | biến thể lọt cổng lib | `amend_khong_upstream_thi_was_pushed_false`, `amend_doi_sha_nhung_khong_tang_so_commit` | **2** |
| **M12** | `runner.write` → `runner.read` | `duong_doc_dung_read_duong_ghi_dung_write` (lib), `create_commit_giu_write_lock_plat_03` (tích hợp) | **2** |
| **MC** | `CLEANUP_MODE` → `--cleanup=default` | `cleanup_whitespace_ghim_cho_moi_duong_commit` (nguồn) | **1** |
| **MC2** | bỏ hẳn `.arg(CLEANUP_MODE)` | `cleanup_whitespace_giu_dong_hash_ke_ca_khi_cau_hinh_doi_nguoc` | **1** |
| **MS** | `ca_hai_luong` bỏ stdout | `ca_hai_luong_giu_du_ca_hai`, `phan_loai_doc_ca_stdout_lan_stderr` (lib), `khong_co_gi_de_commit_cho_loi_doc_hieu_duoc` (tích hợp) | **3** |
| **ML** | đường lock **xoá** `index.lock` | `khong_co_duong_nao_xoa_tep_lock` | **1** |

**M6, M7, M8, M13, M14 là đột biến FRONTEND** — không áp được ở nửa này.

### 🔴 M3 cho 0 test hành vi đỏ — plan sai về cơ chế

Plan nói M3 làm *"test thông điệp nhiều dòng giữ xuống dòng"* đỏ. Áp M3 thật rồi chạy
cả bộ tích hợp: **`15 passed; 0 failed`**.

Lý lẽ của plan ("argv trên Windows là một chuỗi nên xuống dòng không sống sót") đúng cho
một **shell**, nhưng `std::process::Command` của Rust **không** đi qua shell — nó dựng
dòng lệnh bằng phép trích dẫn đúng của Windows. Đo trực tiếp:

```text
git commit --cleanup=whitespace -m $'subject\n\nbody one\nbody two'
  -> %B đọc lại: "subject\n\nbody one\nbody two\n"   (NGUYÊN VẸN)
```

Ràng buộc `-F -` **vẫn đúng**, nhưng lý do thật là ca **byte không phải UTF-8**:
`GitCommand::arg` gọi `to_string_lossy()`, nên byte hỏng thành `U+FFFD` **vĩnh viễn**.
Ca đó **không kiểm được qua API công khai** vì `create_commit` nhận `String` (đã là
UTF-8 hợp lệ **theo kiểu**).

**Nên cổng đọc mã nguồn là cổng DUY NHẤT cho M3**, và tôi đã ghi điều đó vào doc comment
của chính test — một cổng được ghi là "lớp thứ hai" trong khi nó là lớp **duy nhất** sẽ
bị ai đó xoá đi vì tưởng có cổng khác đỡ.

### 🔴 M9: cổng đọc mã nguồn KHÔNG đủ, và tôi đo được điều đó

`if was_pushed { return Err }` bị cổng nguồn bắt. Nhưng bản `match`:

```rust
match was_pushed { true => return Err(...), false => {} }
```

**lọt hoàn toàn** — `277 passed, 0 failed` ở lib. Chỉ test hành vi bắt được:

```text
🔴 amend trên commit ĐÃ PUSH phải VẪN CHẠY. Một `Err` ở đây là phép CHẶN, và ROADMAP
nói nguyên văn 'cảnh báo, KHÔNG chặn' (WORK-09, đột biến M9): NoRepositoryOpen
```

Hai lớp cổng, mỗi lớp bắt thứ lớp kia bỏ lọt. Đây là lý do tôi giữ **cả hai** cho M9 và
M11 thay vì chọn một.

## Deviations from Plan

### 1. [Rule 1 — Bug] `was_pushed` đo sai thời điểm — xem phát hiện 2

Plan chỉ định đo **sau** amend. Cài đúng vậy → cảnh báo không bao giờ hiện. Sửa: đo
**trước**. **Agent frontend cần biết:** `wasPushed` vẫn về đúng như hợp đồng, không đổi
gì phía TypeScript.

### 2. [Rule 3 — Blocking] `chay_lenh_ghi_co_thu_lai` gán một nguyên nhân cho mọi exit khác 0

Hàm của 04-02 tự trả `GitError::CommandFailed` cho **mọi** thất bại không phải lock, nên
`create_commit` **không bao giờ** thấy được đầu ra hook để phân loại. Hai test đỏ ngay
lần chạy đầu (`command_failed` thay vì `hook_rejected` / `nothing_to_commit`).

Đây đúng khuôn lỗi **KB-4b** của `open_repository` mà CONTEXT.md mục 0 cảnh báo — chỉ
lần này nó nằm trong một hàm dùng chung viết ở wave trước.

**Sửa:** tách `chay_lenh_ghi_co_thu_lai_phan_loai` nhận phép phân loại **tiêm vào**.
Đường thử lại `index.lock` **dùng lại nguyên vẹn**, đúng lời plan ("dùng cùng hàm, không
viết lại"); chỉ phép phân loại là riêng. `stage`/`unstage` giữ nguyên hành vi qua một
wrapper mỏng — 14 test của `worktree_commands` vẫn xanh.

Nhân tiện sửa một thứ hai: phép nhận biết lock giờ đọc **cả hai luồng**, không chỉ
stderr.

### 3. [Rule 1 — Bug] Repo mẫu hook bị chính phép đo của tôi làm bẩn

Tôi chạy `git commit --no-verify` **bằng tay** trên repo gốc để đo hành vi hook. Lệnh đó
**tiêu thụ** tệp đã stage của fixture, và mọi lần chạy test sau đều thấy repo sạch rồi
đỏ ở *"tệp vẫn stage"* — **vì một lý do không liên quan tới thứ nó kiểm**.

Lần sửa đầu cũng sai: tôi ghi lại đúng chuỗi `"de commit\n"` mà fixture đã commit, nên
`git add` là lệnh **không-làm-gì** và index vẫn sạch. Phải ghi nội dung **khác** HEAD.

**Sửa:** bản sao của mỗi test **tự dựng** tệp đã stage (nội dung khác HEAD), cộng một
phép khẳng định tiền đề `git diff --cached` trước khi trả về. Giờ test độc lập với lịch
sử của fixture kể cả khi ai đó đã commit vào repo gốc.

**Bài học tổng quát hơn fixture này:** một test **dựa vào trạng thái tích luỹ** của
fixture chỉ đúng cho tới lần đầu ai đó chạm vào repo gốc — và nó hỏng ở **lần chạy thứ
hai**, tức lọt qua được một lần đo.

### 4. Cập nhật cổng đếm variant lỗi: 10 → 13, KHÔNG nới lỏng

`moi_ma_loi_khac_nhau_doi_mot` khẳng định `so_luong == 10`. Thêm ba variant làm nó đỏ —
**đó là cổng hoạt động đúng**. Cập nhật thành `13` và ghi **lịch sử con số** cùng lý do
vào thông điệp thất bại, theo đúng tiền lệ 04-02 với cổng pathspec. Vẫn là phép so
**bằng tuyệt đối**, không phải `>=`.

### 5. Cổng pathspec của 03-02 KHÔNG cần cập nhật

Brief cảnh báo cổng `moi_lenh_co_pathspec...` sẽ đỏ khi thêm lệnh git. **Nó không đỏ**,
và đó là đúng: cổng của `diff.rs` đọc `include_str!("diff.rs")`, cổng của `worktree.rs`
đọc tệp của nó — **cả hai đều không nhìn thấy** `commit.rs`. Lệnh commit của tôi cũng
**không mang pathspec** (nó commit cả index).

Thay vì để đó, tôi thêm cổng `lenh_commit_khong_mang_pathspec` ghim điều này: ai thêm
pathspec vào `git commit` sau này sẽ đỏ và được nhắc mang theo `--` cùng mốc
`PATHSPEC_SAU_DAU_GACH` theo khuôn `worktree.rs`.

### 6. `amend_commit` nhận thêm tham số `noVerify`

Plan viết `amend_commit(...)` không nói rõ. Cho nó `noVerify` để amend cũng tôn trọng
công tắc — và thêm test ghim rằng amend chạy hook **mặc định**, vì một đường amend tự
thêm `--no-verify` lọt qua **mọi** test của `create_commit`. **Agent frontend cần biết:**
gọi `amendCommit` phải truyền `noVerify`.

## Task đã bỏ qua — thuần frontend

| Task | Nội dung | Ai làm |
|---|---|---|
| **Task 2** | `commitDraft.ts`, `commitStore.ts`, nháp bền vững, M6/M7/M8/M14 | agent frontend |
| **Task 3** | `CommitBox.tsx`, `app.css`, M9 phía giao diện, M13 | agent frontend |

Tôi **không đụng** `src/lib/ipc.ts` — ba mã lỗi mới (`empty_commit_message`,
`nothing_to_commit`, `hook_rejected`) cần được thêm vào `GitErrorCode` ở đó, và đó là
tệp của agent frontend. **Nếu họ không thêm, `describeError` sẽ không nhận ra ba mã này.**

## Điều KHÔNG kiểm chứng được

| # | Mục | Trạng thái |
|---|---|---|
| 1 | Vòng commit đầu-cuối qua giao diện thật | **chưa kiểm** — chờ 04-05 gắn `CommitBox` vào `App.tsx` |
| 2 | Hook `pre-commit` chạy hàng chục giây (linter thật) có làm giao diện treo không | **chưa đo** — `DEFAULT_TIMEOUT` là 30 s, một hook chậm hơn sẽ bị cắt và người dùng thấy `Timeout`. Chưa có ca thật |
| 3 | Thông điệp chứa byte không phải UTF-8 | **không kiểm được qua API hiện tại** — `create_commit` nhận `String`. Đường `-F -` đúng theo cấu trúc, nhưng ca đó chưa có test |
| 4 | Hành vi trên macOS/Linux | **chưa đo** — hook fixture dùng `chmod +x`, đúng trên POSIX, nhưng chưa ai chạy ở đó |
| 5 | `--lib --tests` trên máy không tranh cargo | **đo được lần này** (332), nhưng hai lần đầu phiên thất bại vì hết RAM |

## Known Stubs

Không có stub. Hai command nối dây đầy đủ từ `generate_handler!` xuống `git` CLI.

`AmendResult.was_pushed` **chưa có người tiêu thụ** — nó chờ `CommitBox` của agent
frontend.

## Threat Flags

| Flag | File | Mô tả |
|---|---|---|
| threat_flag: write-surface | `src-tauri/src/commands/commit.rs` | **Bề mặt ghi thứ hai của ứng dụng, và là bề mặt đầu tiên sinh ra object git vĩnh viễn.** `message` là chuỗi **tuỳ ý từ webview** đi thẳng vào stdin của `git commit`. Giảm nhẹ đã cài: đi qua `-F -` nên nó **không bao giờ** vào argv (không có đường nào để nó trở thành cờ — có test cho thông điệp bắt đầu bằng `--amend`), `repo_id` phải là repo đã mở qua `open_repository`, `write_lock` giữ suốt thời gian chạy, `--cleanup=whitespace` ghim để cấu hình người dùng không nuốt nội dung. **Chưa có giới hạn ĐỘ DÀI thông điệp** — một chuỗi 100 MB từ webview sẽ được ghi vào stdin và thành một object git; git chấp nhận, và ca đó chưa có test. |
| threat_flag: hook-execution | `src-tauri/src/commands/commit.rs` | `git commit` **thực thi mã tuỳ ý** từ `.git/hooks/*` — đó là yêu cầu của ROADMAP, không phải sơ suất. Nhưng nghĩa là mở một repo lạ rồi bấm commit **chạy script của người viết repo đó** với quyền của người dùng. `--no-verify` là lối thoát duy nhất và nó do người dùng bật. Không có sandbox, và không thể có với ràng buộc "dùng git CLI". Nêu tên để ai đọc sau không tưởng đây là chuyện bỏ sót. |

## Self-Check: PASSED

Tệp đã tạo:
- `src-tauri/src/commands/commit.rs` — FOUND
- `src-tauri/tests/commit_commands.rs` — FOUND

Commit (kiểm bằng `git rev-list --format`, **không** dùng `git log`):
- `5e44858` test(04-03): hook fixtures that prove the hook actually runs — FOUND
- `92681c1` feat(04-03): create_commit / amend_commit — hooks by default — FOUND
- `553e9ff` feat(04-03): register create_commit / amend_commit — FOUND

Không tệp nào của session khác bị commit — xác nhận bằng `git diff --cached | grep -iE
"avatar|md-5|default-run"` trước **mỗi** commit (rỗng cả ba lần). Hai tệp dùng chung
(`lib.rs`, `commands/mod.rs`) được dựng nội dung staged thẳng từ bản ở HEAD cộng đúng
bốn dòng của tôi, rồi **trả lại nguyên trạng** cho cây làm việc ngay sau commit.
