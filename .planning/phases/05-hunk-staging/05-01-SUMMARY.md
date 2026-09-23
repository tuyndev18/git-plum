---
phase: 05-hunk-staging
plan: 01
subsystem: patch-build
tags: [WORK-03, WORK-04, byte-tho, fixture]
requires:
  - "04-02 (get_worktree_diff — nguồn bản vá mà module này tách)"
provides:
  - "git::patch_build::tach_hunk_tho — tách bản vá thành khối trên &[u8]"
  - "git::patch_build::dung_ban_va_mot_hunk / dung_ban_va_nhieu_hunk"
  - "repo mẫu hunk-cases (scripts/fixtures/make-hunk-fixtures.sh)"
  - "testing::require_hunk_fixture"
affects:
  - "05-02 (stage_hunk/unstage_hunk dùng các hàm dựng bản vá này)"
tech-stack:
  added: []
---

# 05-01 — Tách bản vá theo khối trên byte thô

> ⚠️ **SUMMARY này do orchestrator viết, không phải executor.** Agent thực thi **chết
> giữa chừng** vì lỗi mạng (`API Error: ENOTFOUND`) **sau khi** đã commit mã
> (`359dbce`) nhưng **trước khi** viết SUMMARY. Nên tài liệu này chỉ chứa thứ tôi
> **tự đo lại được**, không chứa bảng đột biến đầy đủ của executor — bảng đó đã mất
> cùng phiên làm việc của nó.
>
> **Hệ quả cần biết:** 05-01 có mã và có test, nhưng **không có bản ghi đầy đủ về việc
> executor đã chạy những đột biến nào**. Ai làm wave 2 nên coi bảng đột biến của 05-01
> là **chưa đầy đủ**, không phải là đã xong.

## Đã giao gì

| Tệp | Dòng | Nội dung |
|---|---:|---|
| `src-tauri/src/git/patch_build.rs` | 705 | tách khối, dựng bản vá con |
| `scripts/fixtures/make-hunk-fixtures.sh` | 442 | repo mẫu `hunk-cases` |
| `src-tauri/src/testing/mod.rs` | +97 | `require_hunk_fixture` |
| `src-tauri/src/git/mod.rs` | +1 | khai báo module |

API công khai:

```rust
pub fn tach_hunk_tho(patch: &[u8]) -> Result<PatchTho>
pub fn dung_ban_va_mot_hunk(p: &PatchTho, chi_so: usize) -> Option<Vec<u8>>
pub fn dung_ban_va_nhieu_hunk(p: &PatchTho, chi_so: &[usize]) -> Result<Vec<u8>>
```

`&[u8]` vào, `Vec<u8>` ra — **không có `String` ở bất kỳ điểm nào trên đường bản vá**,
đúng ràng buộc cứng của ROADMAP.

## Số đo — do tôi chạy lại, không lấy từ báo cáo

| Phép đo | Trước | Sau | Lệnh |
|---|---:|---:|---|
| `cargo test --lib` | 309 | **321** (+12) | `cd src-tauri && cargo test --lib` |
| `cargo clippy --lib` | 0 | **0** | `cargo clippy --lib` |
| `cargo test --lib --tests` | 425 | 🔴 **CHƯA ĐO** | bị chặn — xem dưới |

`--lib --tests` **chưa đo được**: bản **debug** `git-plum.exe` đang chạy (`tauri dev`
của một phiên khác), nên cargo không relink được binary:

```text
error: failed to remove file `...\target\debug\git-plum.exe`
Caused by: Access is denied. (os error 5)
```

**Không kill** — tiến trình không phải của phase này, và cổng dogfood của Phase 4 cần
ứng dụng chạy được. Ghi "chưa đo", **không** suy ra tổng.

## Cổng tôi tự kiểm chứng

Cổng quan trọng nhất của wave này là **"đường bản vá không bao giờ giải mã chuỗi"**
(`duong_ban_va_khong_bao_gio_giai_ma`, `patch_build.rs:667`). Đó là cổng bảo vệ WORK-04:
`String::from_utf8_lossy` đổi mỗi byte không hợp lệ thành U+FFFD — **ba** byte — và git
sẽ ghi ba byte đó vào tệp thật của người dùng.

**Cổng này đúng hình dạng đã yêu cầu:**

- Lọc chú thích **và** cắt `mod tests` trước khi tìm.
- **Khẳng định tiền đề trước**: nếu không thấy `pub fn tach_hunk_tho` trong thân đã lọc
  thì **đỏ**, kèm thông điệp nói thẳng *"cổng này đang tìm trong một chuỗi rỗng… và sẽ
  XANH với MỌI mã"*. Đây là lỗi #1/#3/#5 của `CONTEXT.md` §4.1 bị chặn ngay trong test.
- Có một test **riêng** kiểm phép lọc thật sự cắt được `mod tests` — không có nó thì
  phép lọc hỏng theo hai hướng đều tạo ra cổng hỏng.

**Đo đột biến (tôi tự chạy):** thêm `let _mutation = String::from_utf8_lossy(patch);`
vào đầu `tach_hunk_tho`:

```text
thread 'git::patch_build::tests::duong_ban_va_khong_bao_gio_giai_ma' panicked at
  src\git\patch_build.rs:667:13
test result: FAILED. 320 passed; 1 failed
```

Hoàn nguyên → `321 passed`, cây sạch (`git status --porcelain` rỗng trên tệp đó).

## Chưa kiểm — ghi rõ

- **Bảng đột biến đầy đủ của 05-01.** Executor chết trước khi báo cáo. Tôi chỉ kiểm
  chứng **một** cổng (cổng không-giải-mã). Các đột biến khác mà plan yêu cầu —
  đặc biệt quanh `dung_ban_va_nhieu_hunk` và ranh giới khối — **chưa có bằng chứng đã
  chạy**.
- **`cargo test --lib --tests`** — bị khoá bởi app đang chạy.
- **Hành vi trên repo thật.** `hunk-cases` là repo mẫu dựng bằng script; chưa ai chạy
  `tach_hunk_tho` trên một bản vá sinh từ repo thật có CRLF + thiếu dòng cuối + byte
  không UTF-8 cùng lúc. Đó là việc của 05-02 và của cổng `od -c` ở 05-05.
- **Không có requirement nào của Phase 5 đạt.** WORK-03 và WORK-04 mới ở mức "có mã,
  chưa kiểm" — bản vá dựng ra chưa bao giờ được `git apply` thật (đó là wave 2).

## Việc cho wave 2

`05-02-PLAN.md` dùng ba hàm trên để dựng bản vá rồi `git apply --cached --recount`.
Hai điều đã đo được, đừng đo lại:

1. **Fixture `--recount` phải là thân-bị-cắt-header-cũ**, không phải khối-2-nguyên-văn.
   Số đếm `@@` là **theo từng khối**, không cộng dồn, nên chép nguyên header khối 2 vẫn
   đúng dù bỏ bao nhiêu khối khác — `CONTEXT.md` §2.2 có số đo đầy đủ.
2. **stderr lẫn vào stdout** khi bắt bằng `$(...)`: `git stash create` in
   `warning: LF will be replaced by CRLF` cùng luồng với sha. Đây đúng là lý do ROADMAP
   đòi tách hai luồng, và nó áp cho `git apply` y hệt.
