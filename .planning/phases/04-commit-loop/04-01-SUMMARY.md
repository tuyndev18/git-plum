---
phase: 04-commit-loop
plan: 01
subsystem: git-status
tags: [parser, domain, porcelain-v2, fixtures, WORK-01, WORK-11]
requires: []
provides:
  - "RepoStatus / StatusEntry / BranchInfo / WipCounts / StatusGroup"
  - "STATUS_ARGS (nguồn duy nhất của đối số lệnh trạng thái)"
  - "parse_status (byte -> RepoStatus, hàm thuần)"
  - "repo mẫu status-cases + require_status_fixture()"
affects:
  - "04-02 (command trạng thái), 04-03 (stage/unstage), 04-04 (watcher), 04-05 (hàng WIP)"
tech-stack:
  added: []
  patterns:
    - "quét tuần tự, dạng bản ghi quyết định số đoạn NUL tiêu thụ (KHÔNG split(0))"
    - "tách trường theo khoảng trắng CÓ GIỚI HẠN số lần (đường dẫn là trường cuối)"
    - "nhận dạng bản ghi bằng byte đầu + khoảng trắng theo sau"
key-files:
  created:
    - src-tauri/src/domain/status.rs
    - src-tauri/src/git/parsers/status.rs
    - scripts/fixtures/make-status-fixtures.sh
  modified:
    - src-tauri/src/domain/mod.rs
    - src-tauri/src/git/parsers/mod.rs
    - src-tauri/src/testing/mod.rs
decisions:
  - "wip_counts() là hàm dẫn xuất trên RepoStatus, không phải trường — ràng buộc \"không lệnh git thứ hai để đếm\" đúng theo KIỂU"
  - "ahead/behind là Option; # branch.ab VẮNG MẶT khi không upstream, None != Some(0)"
  - "status-cases KHÔNG vào FIXTURE_NAMES — script riêng, nếu gộp thì require_fixture in lệnh sai"
  - "tên tệp không UTF-8 dựng được trong tree nhưng KHÔNG trên NTFS — cổng thật là test byte-literal"
metrics:
  duration: ~2.5h
  completed: 2026-09-22
---

# Phase 4 Plan 01: RepoStatus + parse_status Summary

Hợp đồng dữ liệu trạng thái thư mục làm việc cộng bộ phân tích
`git status --porcelain=v2 --branch -z --untracked-files=all` theo byte, quét tuần tự
với dạng bản ghi quyết định số đoạn NUL tiêu thụ — cộng một phát hiện rằng ca kiểm
quan trọng nhất của plan **không phân biệt được** đột biến quan trọng nhất, và nó đã
được sửa.

## Byte thật đã đo (không đọc tài liệu)

Đo lại độc lập trên git 2.54.0.windows.1, `od -c`, repo dựng tại chỗ. Ba khẳng định
của plan **đều đúng**:

```text
0000260   c   1   9   c   5   2   f   3   4       R   1   0   0       a
0000300   _   n   e   w   .   t   x   t  \0   a   _   o   l   d   .   t
0000320   x   t  \0   1       M   .       N   .   .   .       1   0   0
```

1. **Bản ghi dạng `2` chiếm HAI đoạn NUL trong MỘT bản ghi logic**, đường dẫn **mới
   trước, cũ sau**.
2. **Trường ngăn nhau bằng KHOẢNG TRẮNG**, không phải `\x1f`. Điểm số `R100` là trường
   ngay trước đường dẫn mới → dạng `2` có **9** trường trước path, dạng `1` có 8,
   dạng `u` có **10**.
3. **`# branch.ab` VẮNG MẶT** khi không có upstream — git không in `+0 -0`:
   - repo mới `git init`: chỉ `# branch.oid`, `# branch.head`
   - repo git-plum: thêm `# branch.upstream origin/main`, `# branch.ab +27 -0`
   - HEAD tách rời: `# branch.head (detached)`

## Bằng chứng fixture PHÂN BIỆT ĐƯỢC (không chỉ được chỉ định)

`bash scripts/fixtures/make-status-fixtures.sh`, đầu ra `tr '\0' '\n'`:

```text
1:# branch.oid 31b31da1e0a130b4f360ad453d9d70f440e9abfd
2:# branch.head main
3:2 R. N... 100644 100644 100644 72943a16… 72943a16… R100 a_new.txt
4:a_old.txt
5:1 M. N... 100644 100644 100644 b6802534… 8781b9bf… m_one.txt
6:1 M. N... 100644 100644 100644 b6802534… 8781b9bf… n_two.txt
7:? b_café.txt
8:? z with space.txt
9:? z_untracked.txt

Bản ghi dạng '2' ở dòng 3; sau nó còn 5 bản ghi (cần >= 2). ĐẠT.
```

Script **tự khẳng định tiền đề của mình**: không tìm thấy bản ghi dạng `2`, hoặc sau
nó có < 2 bản ghi → script **thoát 1**, không phải in cảnh báo rồi thoát 0.

## 🔴 Phát hiện: đột biến M1 cho 0 test đỏ ở bản đầu — cổng không thể fail thứ BẢY

Plan viết: *"Nếu M1 cho 0 test đỏ thì plan này chưa xong."* Đúng là nó đã cho 0.

**Nguyên nhân đo được.** Đột biến M1 (bản ghi dạng `2` tiêu thụ một đoạn NUL thay vì
hai) làm đoạn đường dẫn cũ rò ra ở vòng lặp sau. Nhưng với `a_old.txt`:

- `la_dang(b"a_old.txt", …)` trượt **mọi** dạng (không có khoảng trắng sau ký tự đầu);
- nó rơi vào nhánh `_ => {}` và **bị bỏ trong im lặng**;
- bộ phân tích vẫn trả **đúng 3** phần tử với **đúng** đường dẫn.

Fixture được *chỉ định* là phân biệt được, không được *chứng minh* — đúng lỗi cổng #4
của CONTEXT.md mục 3.1.

**Phép khẳng định `old_path` cũng không bắt được M1**, và đây là chỗ tôi suy luận sai
một lượt: M1 chỉ bỏ bước **tiêu thụ**; `duong_dan_cu` vẫn được đọc đúng qua
`cat_doan(con_lai)`, nên `old_path` vẫn đúng. Thử `old name.txt` (có khoảng trắng
nhưng token đầu không phải ký tự dạng) cũng **không** đủ — vẫn bị bỏ im lặng.

**Thứ thật sự phân biệt:** đoạn rò ra phải **trông giống một bản ghi** để thành một
phần tử rác đếm được. Đường dẫn cũ `? cu.txt` làm được (git cho phép mọi byte trừ NUL
và `/` trong tên tệp, nên đây là dữ liệu hợp lệ). Sau khi sửa, M1 cho **3** phần tử
thay vì 2 và **hai** test đỏ, gồm cả ca chịu lực.

**Điều này cũng nói về mã, không chỉ về test:** phép lệch nấc chỉ **im lặng** khi
đường dẫn cũ tình cờ vô hại. Đường dẫn cũ có khoảng trắng, hoặc bắt đầu bằng ký tự
dạng, sinh một bản ghi sai thật sự.

Sửa ở commit `c0c4264`.

## Bảng đột biến — cả 10 dòng, đầu ra thật đã dán

Quy trình: áp đột biến → `cargo test --lib` (qua `rtk proxy` để có đầu ra thô) → ghi
tên test đỏ → phục hồi từ bản sao sạch. Harness **khẳng định tiền đề**: nếu tệp không
khác bản sạch thì báo lỗi, không ghi "0 test đỏ" như một kết quả.

| # | Đột biến | Test ĐỎ (tên thật) | Số đỏ |
|---|---|---|---|
| **M1** | dạng `2` tiêu thụ **một** đoạn NUL thay vì hai | `hai_ban_ghi_sau_dang_2_khong_lech_nac`, `duong_dan_cu_trong_giong_ban_ghi_thi_lech_nac_sinh_phan_tu_rac` | **2** |
| M2 | đảo thứ tự hai đường dẫn của dạng `2` | `dang_2_duong_dan_moi_truoc_cu_sau`, `duong_dan_doi_ten_co_khoang_trang_ca_hai_dau`, `hai_ban_ghi_dang_2_lien_tiep_roi_mot_dang_1`, `hai_ban_ghi_sau_dang_2_khong_lech_nac`, `duong_dan_cu_trong_giong_ban_ghi…`, `doc_duoc_dau_ra_that_tu_repo_mau` | 6 |
| M3 | tách trường khoảng trắng **không giới hạn** | `duong_dan_co_khoang_trang_khong_bi_cat`, `duong_dan_doi_ten_co_khoang_trang_ca_hai_dau`, `branch_ab_doc_duoc_ahead_va_behind`, `branch_head_detached_thi_none`, `branch_oid_initial_thi_none`, `dong_branch_la_bi_bo_qua_khong_hong`, `doc_duoc_dau_ra_that_tu_repo_mau` | 7 |
| M4 | nhận dạng bằng byte đầu, bỏ kiểm khoảng trắng | `doan_bat_dau_bang_2_nhung_khong_phai_dang_2_khong_an_ban_ghi_sau` | 1 |
| M5 | bỏ `--untracked-files=all` khỏi `STATUS_ARGS` | `status_args_co_du_bon_co_theo_ten` | 1 |
| M6 | bỏ `--branch` khỏi `STATUS_ARGS` | `status_args_co_du_bon_co_theo_ten`, `doc_duoc_dau_ra_that_tu_repo_mau` | 2 |
| M7 | `ahead`/`behind` trả `Some(0)` khi thiếu `# branch.ab` | `khong_co_branch_ab_thi_none_khong_phai_some_0`, `dau_vao_rong_khong_panic`, `doc_duoc_dau_ra_that_tu_repo_mau` | 3 |
| M8 | `wip_counts` đếm **entry** thay vì đường dẫn duy nhất | `wip_counts_tep_mm_dem_la_mot_khong_phai_hai`, `wip_counts_ba_tep_sua_mot_tep_moi`, `wip_counts_tep_am_dem_mot_lan_va_la_tep_moi`, `xy_mm_sinh_hai_phan_tu_nhung_wip_counts_dem_mot` | 4 |
| M9 | cho dạng `!` (ignored) vào `entries` | `ban_ghi_dang_ignored_bi_bo_qua`, `dang_ignored_xen_giua_khong_lam_lech_nac` | 2 |
| M10 | `has_invalid_utf8` luôn `false` | `duong_dan_byte_latin1_khong_mat_ban_ghi`, `duong_dan_cu_byte_latin1_cung_dat_co` | 2 |

Đầu ra thô từng dòng, ví dụ M1 sau khi sửa:

```text
=== M1: dang 2 tieu thu MOT doan NUL thay vi hai (exit=101) ===
-- test ĐỎ:
   test git::parsers::status::tests::duong_dan_cu_trong_giong_ban_ghi_thi_lech_nac_sinh_phan_tu_rac ... FAILED
   test git::parsers::status::tests::hai_ban_ghi_sau_dang_2_khong_lech_nac ... FAILED
-- tổng:
   test result: FAILED. 240 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
```

## Số test trước / sau

| Phạm vi | Trước | Sau |
|---|---|---|
| `cargo test --lib` | 202 | **242** (+40) |
| `cargo test --lib --tests` | 284 (+1 ignored) | **chưa đo được — xem dưới** |
| `npx tsc --noEmit` | sạch | sạch (plan này không đụng TS) |

Phân bố 40 test mới: 10 ở `domain::status`, 26 ở `git::parsers::status` (24 byte-literal
+ 2 chạy trên repo mẫu thật), 1 ở `testing`, cộng 3 test cũ được giữ nguyên tên.

### ⚠️ `--lib --tests` KHÔNG đo được — nêu là CHƯA KIỂM, không phải suy ra

`cargo test --lib --tests` thất bại **không phải vì test**:

```text
error: failed to remove file `…\src-tauri\target\debug\git-plum.exe`
Caused by: Access is denied. (os error 5)
```

`tasklist` cho thấy `git-plum.exe` PID 32472 **đang chạy** — một session khác đang mở
ứng dụng. Cargo relink mọi binary kể cả khi chỉ yêu cầu test target, nên mọi biến thể
(`--test <tên>`, liệt kê tường minh cả 10 target) đều chặn ở cùng chỗ. Đã thử lại
**28 lần** trong khoảng 13 phút với giãn cách 20–40 s; vẫn bị giữ. **Không** kill tiến
trình đó: nó thuộc session khác.

`CARGO_TARGET_DIR` riêng cũng không đi được — bản dựng 292 crate từ đầu chết vì
`The paging file is too small` (os error 1455).

**Suy ra hợp lý là 324** (82 test integration không bị đụng + 242 lib), nhưng tôi
**không** ghi con số đó là đã đo. Cách kiểm khi exe được thả:
`cd src-tauri && cargo test --lib --tests`.

Lý do tin rằng integration không bị ảnh hưởng: plan này chỉ **thêm** module mới
(`domain::status`, `git::parsers::status`) cộng một hàm mới trong `testing`; không
sửa một dòng nào của mã mà 10 tệp trong `tests/` gọi tới. Nhưng đó là lập luận, không
phải phép đo.

## Clippy

`cargo clippy --all-targets`: đúng **1** warning, là cái đã có ở
`src/commands/diff.rs:1470` (`assertions_on_constants`) — **không** phải của plan này.
`cargo clippy --lib`: **0** warning.

## Deviations from Plan

### 1. `status-cases` KHÔNG đăng ký vào `FIXTURE_NAMES` (khác chỉ dẫn của plan)

Plan nói *"Đăng ký `"status-cases"` vào `FIXTURE_NAMES`"*. **Không làm**, và plan tự
nói ra lý do ở ngay câu sau: *"nếu `status-cases` do script riêng sinh ra thì
`FIXTURES_COMMAND` không dựng nó — ghi rõ lệnh đúng trong thông báo bỏ qua."*

`FIXTURE_NAMES` là danh sách repo mà `FIXTURES_COMMAND` (`make-fixtures.sh`) dựng
được, và `require_fixture` in **chính** lệnh đó khi thiếu. Đưa `status-cases` vào đó
làm người mới clone thấy test bỏ qua, chạy đúng lệnh được bảo, và test **vẫn** bỏ qua.
Nó cũng làm test `danh_sach_fixture_du_chin_va_khong_trung` đỏ.

Thay vào đó theo tiền lệ `DIFF_FIXTURES_DIR` của Phase 3: `STATUS_FIXTURES_COMMAND` +
`STATUS_FIXTURES_DIR` + `require_status_fixture()`, cộng test
`status_cases_khong_nam_trong_danh_sach_cua_make_fixtures` ghim phân biệt đó lại.

### 2. Tên tệp không phải UTF-8 KHÔNG dựng được trên máy này — giới hạn nền tảng

Plan yêu cầu fixture có *"một tệp tên byte Latin-1"*. Blob vào tree **đúng**
(`git ls-tree` in `"b_caf\351.txt"`), nhưng khi ghi ra thư mục làm việc, **NTFS lưu tên
tệp dạng UTF-16 và không đựng được một byte `\351` đơn lẻ**; git đọc ngược thành UTF-8
`\303\251`. Xác nhận bằng `od -c` trên đầu ra status:
`?   b   _   c   a   f 303 251   .   t   x   t  \0`.

Đây là giới hạn hệ thống tệp, không phải lỗi script — trên ext4/APFS thì dựng được.
Nên **cổng non-UTF-8 thật sự của plan là test byte-literal** (`? caf\xe9.txt\0`), vốn
chạy giống nhau trên mọi nền tảng, và đột biến M10 chứng minh nó phân biệt được.
Script ghi rõ lý do để ai đọc dòng "KHÔNG" sau này không tưởng fixture hỏng.

### 3. [Rule 1 - Bug] Đếm byte `data <n>` của fast-import bằng tay → sai

Lần chạy đầu của script chết với `fatal: unsupported command:  100644 :1
"b_caf\351.txt"`. Đúng lỗi mà `make-fixtures.sh` đã cảnh báo bằng chữ in hoa
(*"TUYỆT ĐỐI KHÔNG viết số n bằng tay"*) và tôi vẫn mắc. Sửa bằng cách dùng lại khuôn
`emit_data` đo qua `wc -c`.

### 4. [Rule 3 - Blocking] Không truyền đường dẫn byte thô làm pathspec qua shell

`git checkout rawname -- "b_caf\351.txt"` thất bại: `pathspec 'b_café.txt' did not
match` — shell chuyển `\351` thành UTF-8 **trước** khi git thấy. Đi vòng bằng
`read-tree` + `checkout-index -a` (không cần ai gõ tên), rồi `read-tree HEAD` để index
sạch lại.

## Điều đã kiểm về môi trường (khớp lời nhắc của chủ dự án)

- **`cargo test` bị rtk lọc** — nhưng nó **ghi dòng tóm tắt vào tệp**
  (`cargo test: 284 passed, 1 ignored (13 suites, 6.22s)`), nên `> file` vẫn đọc được
  tổng số. Muốn **tên test** từng dòng thì phải `rtk proxy cargo test …`.
- `git log` không dùng ở đâu trong plan này.
- Không đụng bất kỳ tệp nào của hai session khác. `src-tauri/Cargo.toml` hiện `M`
  trong `git status` là `default-run` của session khác — **không** stage, không sửa.
  Mọi commit stage theo đường dẫn tường minh.

## Known Stubs

Không có. Plan này không có UI và không có đường dữ liệu nào bị cắm cứng.

## Threat Flags

Không có bề mặt mới. `parse_status` là hàm thuần trên `&[u8]`, không I/O, không spawn
tiến trình. `STATUS_ARGS` là hằng; plan 04-02 sẽ là nơi nó đi qua `git/exec.rs` và là
nơi cổng thứ tự cờ `--` của 03-02 cần được cập nhật (CONTEXT.md mục 3.5).

## Self-Check: PASSED

Tệp đã tạo:
- `src-tauri/src/domain/status.rs` — FOUND
- `src-tauri/src/git/parsers/status.rs` — FOUND
- `scripts/fixtures/make-status-fixtures.sh` — FOUND

Commit (kiểm bằng `git rev-parse`/`git cat-file`, **không** dùng `git log`):
- `034557d` feat(04-01): add RepoStatus… — FOUND
- `05384ef` feat(04-01): parse porcelain=v2… — FOUND
- `c0c4264` test(04-01): make the step-shift test actually discriminate… — FOUND
