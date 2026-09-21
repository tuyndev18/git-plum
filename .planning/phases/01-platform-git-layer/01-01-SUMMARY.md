---
phase: 01-platform-git-layer
plan: 01
subsystem: infra
tags: [rust, tokio, git-cli, process-spawning, tempfile]

# Dependency graph
requires:
  - phase: 00-khởi tạo (commit `1002fa5`)
    provides: "`GitCommand` và `apply_env_hardening` trong `src-tauri/src/git/exec.rs`"
provides:
  - "`GIT_CONFIG_PARAMETERS` ghim đủ ba khoá: `log.showSignature`, `diff.noprefix`, `format.coverLetter`"
  - "Hằng `PINNED_GIT_CONFIG` làm nguồn sự thật duy nhất cho chuỗi cấu hình ghim"
  - "Bốn test chạy git thật, đọc ngược từng giá trị từ tiến trình con"
  - "Ghi chú bàn giao `--cleanup=whitespace` cho Phase 4"
affects: [phase-02-đọc-dữ-liệu-git, phase-03-diff, phase-04-commit, phase-05-áp-bản-vá]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Test đọc ngược cấu hình từ tiến trình con thay vì khẳng định trên hằng số trong mã"
    - "Hằng phạm vi module làm nguồn sự thật chung cho mã chạy và test"

key-files:
  created:
    - docs/02-phase4-commit-notes.md
  modified:
    - src-tauri/src/git/exec.rs

key-decisions:
  - "`diff.external` giữ nguyên cách ghim bằng `GIT_EXTERNAL_DIFF=\"\"`, không thêm vào `PINNED_GIT_CONFIG` — biến môi trường thắng cấu hình nên mạnh hơn"
  - "Test 4 so khớp tên khoá đã hạ chữ thường vì `git config --get-regexp` chuẩn hoá phần tên khoá"
  - "`--cleanup=whitespace` bàn giao bằng tài liệu thay vì dựng sẵn đường ống commit, giữ đúng ranh giới phase"

patterns-established:
  - "Ghim cấu hình phải có test đọc ngược: sai cú pháp `GIT_CONFIG_PARAMETERS` làm git bỏ qua toàn bộ chuỗi trong im lặng"
  - "Ràng buộc không thi hành được trong phase hiện tại thì ghi thành ghi chú bàn giao, kèm `TODO(Phase N)` trỏ tới từ mã"

requirements-completed: [PLAT-02]

# Metrics
duration: 18min
completed: 2026-09-21
---

# Phase 1 Plan 01: Hoàn thiện ghim cấu hình git Summary

**`GIT_CONFIG_PARAMETERS` ghim đủ ba khoá cấu hình qua hằng `PINNED_GIT_CONFIG`, kèm bốn test chạy git thật đọc ngược giá trị từ tiến trình con, và ghi chú bàn giao `--cleanup=whitespace` cho Phase 4.**

## Performance

- **Duration:** ~18 phút
- **Tasks:** 2/2
- **Files modified:** 2 (1 sửa, 1 tạo mới)
- **Test:** 19 → 23 (thêm 4)

## Accomplishments

- Đóng khoảng cách **G1**: `diff.noprefix=false` và `format.coverLetter=false` nay
  cùng nằm trong `GIT_CONFIG_PARAMETERS` với `log.showSignature=false`.
- Chuỗi cấu hình gom vào hằng `PINNED_GIT_CONFIG` ở phạm vi module, để test và mã chạy
  tham chiếu cùng một nguồn sự thật thay vì chép lại chuỗi.
- Bốn test mới sinh tiến trình git thật và đọc ngược từng giá trị bằng
  `git config --get`, cộng một lần liệt kê `--get-regexp` chứng minh git phân tích được
  cả chuỗi chứ không bỏ qua vì sai cú pháp nháy đơn.
- **Kiểm chứng đột biến đã chạy thật:** xoá `'diff.noprefix=false'` khỏi hằng số làm
  đúng hai test đỏ (`pins_diff_noprefix` và `all_pinned_keys_reach_the_child_process`),
  hai test còn lại vẫn xanh — chứng minh test bám thật vào hằng số chứ không luôn đỗ.
- `--cleanup=whitespace` được ghi lại thành `docs/02-phase4-commit-notes.md`, có
  `TODO(Phase 4)` trong `exec.rs` trỏ tới.

## Task Commits

1. **Task 1 (RED): test đọc ngược cấu hình ghim** — `9928194` (test)
2. **Task 1 (GREEN): ghim đủ cấu hình git** — `3d3e11e` (feat)
3. **Task 2: ghi chú bàn giao cho Phase 4** — `e1dc4f8` (docs)

Task 1 theo TDD nên có hai commit (test → feat). Không cần bước refactor: mã sau
GREEN đã sạch với `clippy -D warnings`.

## Files Created/Modified

- `src-tauri/src/git/exec.rs` — thêm hằng `PINNED_GIT_CONFIG` kèm tài liệu từng khoá
  chống lỗi gì; `apply_env_hardening` dùng hằng này; bình luận nêu rõ `diff.external`
  ghim qua `GIT_EXTERNAL_DIFF=""` chứ không bị bỏ sót; `TODO(Phase 4)` trỏ tới ghi chú;
  bốn test mới kèm hàm trợ giúp `read_back`.
- `docs/02-phase4-commit-notes.md` — ghi chú bàn giao: ràng buộc, lý do
  (`--cleanup=default` xoá dòng bắt đầu bằng `#`), nơi thi hành, cách kiểm chứng,
  tham chiếu ngược.

## Decisions Made

- **Không** thêm `diff.external` vào `PINNED_GIT_CONFIG`. Nó đã được ghim bằng
  `GIT_EXTERNAL_DIFF=""` tại `exec.rs`, mà biến môi trường thắng cấu hình nên cách đó
  mạnh hơn. Thay vào đó bổ sung bình luận ở cả hai chỗ để lần soát sau không kết luận
  nhầm là bỏ sót.

  > 🔴 **ĐÍNH CHÍNH 2026-09-22 (plan 03-01, commit `f5c4c17`).** Quyết định này **sai**.
  > `GIT_EXTERNAL_DIFF=""` không vô hiệu hoá trình diff ngoài — git spawn chương trình tên
  > rỗng và chết (`error: cannot spawn : No such file or directory`). Mọi lệnh git sinh bản
  > vá thoát 128 với stdout **rỗng, im lặng**. Ghim `diff.external=` rỗng qua config cũng
  > cùng lỗi. Cơ chế đúng: cờ `--no-ext-diff`, thắng cả config lẫn env, chèn tập trung
  > trong `GitCommand::run()`. Chi tiết và bài học ở `VERIFICATION.md` mục 6.1.
- Test 4 so khớp tên khoá dạng chữ thường (`log.showsignature`, `format.coverletter`).
  Xem mục Issues Encountered.
- Giữ nguyên `SSH_ASKPASS`, `GIT_PAGER`, `PAGER` theo Claude's Discretion trong CONTEXT.md.
- Không cần `git init` trong thư mục tạm: đã kiểm chứng `git config --get` đọc được giá trị
  từ `GIT_CONFIG_PARAMETERS` cả khi đứng ngoài repository (git 2.54.0.windows.1).

## Deviations from Plan

None — plan executed exactly as written. Không có tình huống nào phải áp dụng
deviation rule.

## Issues Encountered

**1. `git config --get-regexp` hạ chữ thường tên khoá**

Kế hoạch mô tả Test 4 là "đầu ra chứa cả ba khoá". Chạy thử trước khi viết test cho thấy
git chuẩn hoá phần tên khoá về chữ thường: đầu ra là `log.showsignature`,
`format.coverletter`, không phải dạng camelCase như lúc đặt. So khớp nguyên dạng camelCase
sẽ làm test đỏ oan. Đã xử lý bằng `to_lowercase()` trên đầu ra rồi so khớp dạng chữ thường,
kèm bình luận giải thích để người sau không "sửa" lại thành camelCase.
Lưu ý `git config --get` thì vẫn nhận camelCase vì tra cứu không phân biệt hoa thường.

**2. `cargo fmt --check` đỏ sau khi thêm test**

`assert!` một dòng trong test cuối vượt quá giới hạn chiều rộng của rustfmt. Đã chạy
`cargo fmt --all`, rustfmt tự tách thành nhiều dòng. Kiểm lại sạch.

**3. Nhiễu từ công cụ shell, không phải lỗi sản phẩm**

Khối `<verify>` của Task 2 nối nhiều `grep -q` bằng `&&` và trả về mã 1 dù mọi điều kiện
đều đúng. Nguyên nhân: lớp proxy `rtk` bọc `grep` trả mã thoát khác 0 ngay cả khi có khớp.
Chạy lại từng điều kiện riêng lẻ (tất cả exit 0) và chạy lại cả chuỗi bằng `/usr/bin/grep`
đều đỗ. Đây là nhiễu của môi trường shell, không phải khiếm khuyết của tệp tài liệu hay
`exec.rs`. Ghi lại để người sau gặp lại không đi tìm lỗi sai chỗ.

Ngoài ra `cargo` không nằm sẵn trên PATH của shell này; phải thêm `$HOME/.cargo/bin` trước
khi chạy. Không ảnh hưởng tới mã nguồn.

## Verification

Chạy thật, kết quả nguyên văn:

```
cargo test                            → 23 passed (3 suites), exit 0
cargo test git::exec                  → 7 passed, 16 filtered out
cargo clippy --all-targets -D warnings → No issues found, exit 0
cargo fmt --all --check               → exit 0
```

Số test tăng từ 19 lên 23, đúng mức kế hoạch đòi ("ít nhất 23").
`git::exec` có 7 test (3 cũ + 4 mới), đúng mức `<done>` của Task 1 đòi.

Kiểm chứng đột biến (bằng chứng test không luôn đỗ):

```
xoá 'diff.noprefix=false' khỏi PINNED_GIT_CONFIG
  → FAILED. 5 passed; 2 failed
  → pins_diff_noprefix: "git không thấy khoá diff.noprefix" (left: 1, right: 0)
  → all_pinned_keys_reach_the_child_process: "thiếu diff.noprefix trong đầu ra"
khôi phục → 7 passed
```

## User Setup Required

None — không cần cấu hình dịch vụ ngoài.

## Next Phase Readiness

- PLAT-02 đã trọn: cả bốn ràng buộc cấu hình của ROADMAP đều có chỗ thi hành hoặc
  chỗ bàn giao rõ ràng. Phase 2 và 3 viết bộ phân tích được trên nền đầu ra git đã
  ổn định, không phụ thuộc cấu hình của máy người dùng.
- Phase 4 phải đọc `docs/02-phase4-commit-notes.md` trước khi dựng lệnh commit đầu tiên.
- Khoảng cách còn lại của Phase 1 không thuộc kế hoạch này: G2 (PLAT-07 danh sách repo
  gần đây), G3 và G8 (PLAT-09 bố cục), G6 (locale), G7 (cửa sổ console bản release).
  G4 (test giao diện) do plan 01-02 xử lý song song.

---
*Phase: 01-platform-git-layer*
*Completed: 2026-09-21*

## Self-Check: PASSED

- Tệp khai báo trong summary đều tồn tại trên đĩa: `src-tauri/src/git/exec.rs`,
  `docs/02-phase4-commit-notes.md`, `.planning/phases/01-platform-git-layer/01-01-SUMMARY.md`.
- Ba commit `9928194`, `3d3e11e`, `e1dc4f8` đều có trong `git log`.
- Không commit nào xoá tệp đang được theo dõi.
