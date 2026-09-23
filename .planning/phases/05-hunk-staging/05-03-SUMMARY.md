---
phase: 05-hunk-staging
plan: 03
subsystem: trash-safety
tags: [WORK-06, WORK-07, stash-create, update-ref, byte-tho]
requires:
  - "05-01 (dung_ban_va_mot_hunk / tach_hunk_tho — nguồn bản vá con cho huy_hunk)"
  - "05-02 (kiem_blob_hash — hợp đồng WORK-05 dùng lại cho huy_hunk)"
  - "04-02 (chay_lenh_ghi_co_thu_lai_phan_loai — đường thử lại index.lock, DÙNG LẠI)"
  - "04-02 (lay_trang_thai — nguồn StatusGroup để phân nhánh chưa-theo-dõi)"
provides:
  - "domain::trash::{MucThungRac, BienNhan, DongTu, dong_tu_cho, TIEN_TO_REF}"
  - "commands::trash::luu_truoc_khi_huy — bằng chứng kiểu, chỉ hàm này dựng BienNhan"
  - "commands::trash::{huy_tep, huy_hunk, danh_sach_thung_rac, khoi_phuc}"
  - "Tauri command: discard_files, discard_hunk, list_trash, restore_trash (đã đăng ký)"
  - "CO_HUY_HUNK — một nguồn sự thật cho tập cờ git apply của đường huỷ"
affects:
  - "05-04+ (giao diện danh sách 'Vừa huỷ gần đây' gọi list_trash/restore_trash)"
tech-stack:
  added: []
key-files:
  created:
    - "src-tauri/src/domain/trash.rs (kiểu miền + 7 test)"
    - "src-tauri/src/commands/trash.rs (lệnh git + 12 cổng đọc nguồn)"
    - "src-tauri/tests/trash_commands.rs (8 test tích hợp trên git thật)"
  modified:
    - "src-tauri/src/domain/mod.rs (khai module)"
    - "src-tauri/src/commands/mod.rs (khai module)"
    - "src-tauri/src/lib.rs (generate_handler! — 4 command)"
decisions:
  - "Phân nhánh chưa-theo-dõi theo TỪNG đường dẫn, không theo sha của stash create — ca hỗn hợp là mất dữ liệu im lặng"
  - "`git checkout HEAD --` chứ không `git checkout --` — bản sau khôi phục từ INDEX nên không huỷ được thay đổi đã stage"
  - "`khoi_phuc` gỡ index về HEAD sau checkout — nếu không, khôi phục stage hộ người dùng"
  - "M14 được thay bằng M14c (lưu sha vào tệp thường) vì M14 nguyên bản là đột biến vô hiệu hoá cả đường khôi phục"
  - "Cổng M18 đòi bước đọc lại phải TỚI ĐƯỢC, không chỉ có mặt — cổng cũ xanh trên mã chết"
metrics:
  duration: "~2h"
  completed: "2026-09-23"
---

# 05-03 — An toàn khi huỷ: trash ref namespace + khôi phục

Mọi đường huỷ đi qua một lần lưu mà **kiểu** bắt buộc, object lưu ra **sống qua
`git gc --prune=now`**, và ca `git stash create` im lặng bỏ qua — tệp chưa theo dõi —
được lưu bằng đường riêng. Toàn bộ kiểm được **không cần mở ứng dụng**.

## Số đo — tôi tự chạy, lệnh đầy đủ

| Phép đo | Trước | Sau | Lệnh |
|---|---:|---:|---|
| `cargo test --lib` | 331 | **350** (+19) | `cd src-tauri && cargo test --lib` |
| `cargo test --lib --tests` | 454 | **481** (+27), 1 ignored | `cd src-tauri && cargo test --lib --tests` |
| `cargo test --test trash_commands` | — | **8 passed, 0 failed** | `cargo test --test trash_commands` |
| `cargo test --test hunk_commands` (wave 2) | 7 | **7** — không hồi quy | `cargo test --test hunk_commands` |
| `cargo clippy --lib` | sạch | **sạch** | `cargo clippy --lib` |

Con số `--lib --tests` được cộng từ **19 suite** bằng cách đọc tệp thô, không suy ra:

```text
TOTAL passed = 481
suites: 19
any failed? []
ignored: 1
```

Đầu ra libtest **thô** (qua `rtk proxy`, vì `rtk` lọc stdout của `cargo test`):

```text
running 8 tests
test khoi_phuc_tu_choi_ref_ngoai_khong_gian_thung_rac ... ok
test tep_sach_khong_duoc_cap_bien_nhan ... ok
test byte_khong_utf8_giu_nguyen_qua_vong_khoi_phuc ... ok
test vong_huy_khoi_phuc_tep_chua_theo_doi ... ok
test ca_hon_hop_tep_chua_theo_doi_van_duoc_luu ... ok
test vong_huy_khoi_phuc_tep_da_theo_doi ... ok
test huy_mot_khoi_giu_nguyen_hai_khoi_kia ... ok
test object_thung_rac_song_qua_git_gc_prune_now ... ok
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## 🔴 Bằng chứng object sống qua `git gc --prune=now`

Đây là yêu cầu trung tâm của WORK-07. Chạy thật, hai vế cạnh nhau, trong repo `tempfile`:

```text
=== SIDE A: CÓ update-ref (đúng thứ git-plum làm) ===
worktree after discard : [one/two/three/]          ← đã huỷ
cat-file -e after gc   : exit=0  (0 = ALIVE)       ← object SỐNG
git show ref:a.txt     : [one/CHANGED/three/]      ← khôi phục được

=== SIDE B: y hệt nhưng KHÔNG update-ref (đột biến M14) ===
cat-file -e BEFORE gc  : exit=0  (0 = alive, looks fine)   ← trông vẫn ổn
cat-file -e AFTER  gc  : exit=1  (1 = GONE - data lost)    ← ĐÃ MẤT
```

Vế B là lý do Test D tồn tại: object **đọc được ngay sau khi tạo**, nên mọi test nhìn
trước lúc gc đều xanh. Chỉ `git gc` — thứ chạy **tự động**, người dùng không gõ gì —
mới phơi ra khác biệt.

## Bảng đột biến — chạy thật, dán đầu ra đỏ

Hoàn nguyên bằng `cp` từ bản sao. **Không bao giờ** `git checkout --` trên tệp chưa
commit. Sau mỗi lần: `git status --porcelain` trên ba tệp trả **rỗng**.

| # | Đột biến | Đỏ mong đợi | **Đo được** |
|---|---|---|---|
| M14 | Bỏ `update-ref`, chỉ giữ `stash create` | Test D (A xanh) | ⚠️ **6 đỏ, A CŨNG đỏ** — đột biến quá rộng, xem dưới |
| M14c | Lưu sha vào **tệp thường** thay vì ref | Test D | ✅ **Test D đỏ, A/C/E XANH** — đây mới là phép đo plan muốn |
| M15 | Bỏ nhánh `hash-object -w` | Test B | ✅ **3 đỏ** — B, E, F |
| M16 | Bỏ `--reverse` khỏi `CO_HUY_HUNK` | Test C | ✅ **2 đỏ** — Test C + cổng nguồn |
| M17 | `khoi_phuc` qua `String::from_utf8_lossy` | Test E | ✅ **2 đỏ** — Test E + cổng nguồn |
| M18 | Bỏ `rev-parse --verify` | (dự đoán: không đỏ) | ⚠️ **0 đỏ lúc đầu → ĐỎ sau khi viết cổng**, xem dưới |
| M19 | `dong_tu_cho` luôn `HuyBo` | Test 1 | ✅ **2 đỏ** |
| M20 | `dong_tu_cho` luôn `Xoa` | Test 1 | ✅ **2 đỏ** — test phân biệt được **cả hai chiều** |
| M21 | Cờ cấm **chỉ trong chú thích** (đối chứng) | 🔴 **không được đỏ** | ✅ **0 đỏ** — phép lọc chú thích hoạt động |

### M14 — plan đoán sai, và lý do đáng ghi

Plan ghi: *"M14 → Test D đỏ, **Test A vẫn xanh** — đó là lý do D tồn tại."*

**Đo được: Test A cũng đỏ**, cùng 5 test khác. Nguyên nhân là `khoi_phuc` của tôi tra
object **qua tên ref**, nên bỏ `update-ref` phá luôn cả đường khôi phục — không chỉ phá
tính tới-được. Đột biến **quá rộng để biểu diễn đúng khuyết tật** đang hỏi: nó không
tách được "tạo rồi" khỏi "tới được".

Đúng bài học wave 2 rút ra ở M7 — *đột biến không biểu diễn được lỗi thì không nói gì về
cổng* — tôi không dừng ở "M14 đỏ, xong". Tôi dựng **M14c**: giữ nguyên mọi thứ, chỉ lưu
ánh xạ `tên → sha` vào một **tệp thường** dưới `.git/` thay vì một ref (đúng lỗi thật
một lập trình viên sẽ viết: *"tôi có lưu sha mà"*). Đường khôi phục vẫn chạy, mọi phép
đọc trước gc vẫn đúng, **chỉ** tính tới-được lúc gc là mất:

```text
test byte_khong_utf8_giu_nguyen_qua_vong_khoi_phuc ... ok      ← Test E XANH
test vong_huy_khoi_phuc_tep_da_theo_doi ... ok                 ← Test A XANH
test huy_mot_khoi_giu_nguyen_hai_khoi_kia ... ok               ← Test C XANH
test object_thung_rac_song_qua_git_gc_prune_now ... FAILED     ← chỉ Test D ĐỎ
```

Và để chắc Test D đỏ ở **chính khẳng định sau gc** chứ không ở tiền đề, tôi nới tạm tiền
đề rồi chạy lại:

```text
thread 'object_thung_rac_song_qua_git_gc_prune_now' panicked at tests\trash_commands.rs:460:5:
🔴 object `054a4ec15679d4b805d6a53edbcdfdd4c4de0086` ĐÃ BỊ DỌN bởi `git gc --prune=now`.
Nó được tạo nhưng không có ref nào trỏ tới — tức nội dung người dùng vừa huỷ đã MẤT
VĨNH VIỄN.
```

Dòng 460 là khẳng định `cat-file -e` **sau** gc. **Test D không thừa, và nó đo đúng thứ
nó nói nó đo.**

### M18 — plan đoán đúng là "không đỏ", nhưng kết luận đúng KHÔNG phải "chấp nhận được"

Plan dự đoán M18 không đỏ ở đường hạnh phúc. **Đúng: 0 đỏ, 350 test xanh.**

Plan cũng chỉ rõ phải làm gì tiếp: *kiểm xem có test nào **hỏi về nó** chưa (lỗi #9); nếu
rỗng, viết test thay vì ghi "chấp nhận được"*. Tôi kiểm:

```text
$ grep -rn "rev-parse" tests/trash_commands.rs
435:    let (ma_rp, _, _) = git_thu(p, &["rev-parse", "--verify", &ref_name]);
```

Một khớp duy nhất, và nó hỏi *"ref có tồn tại không"* — **không** hỏi *"bước đọc lại có
bảo vệ biên nhận không"*. Thuộc tính này **chưa có test nào hỏi**.

Trước khi viết cổng, tôi đo xem đột biến có biểu diễn được thành mất dữ liệu không:

```text
name=[refs/git-plum-trash/bad..name] update-ref_exit=128 verify_exit=128
name=[refs/git-plum-trash/bad name]  update-ref_exit=128 verify_exit=128
name=[refs/git-plum-trash/ok1]       update-ref_exit=0   verify_exit=0
```

`update-ref` thoát **128** với tên xấu, và đường thử lại đã biến 128 thành `Err`. Tôi
**không dựng được** ca `update-ref` thoát 0 mà ref vắng mặt — nên M18 không biểu diễn
được thành mất dữ liệu qua đường hạnh phúc. Ghi rõ như vậy, không tuyên bố hơn.

Nhưng cổng **vẫn thiếu**, và khi viết nó tôi phát hiện cổng đầu của mình cũng hỏng: nó
tìm chuỗi `"rev-parse"` trong thân đã lọc, mà M18 đặt `if true { return Ok(()); }`
**trước** đoạn đó — chuỗi vẫn còn, chỉ thành **mã chết**, nên cổng xanh. Đây là lỗi
#1/#5 ở dạng khó thấy hơn: cổng khớp **mã thật**, không khớp chú thích, mà vẫn không đo
được thứ nó định đo. Cổng mới đòi bước đọc lại phải **tới được**:

```text
test commands::trash::tests::duong_luu_doc_lai_ref_truoc_khi_cap_bien_nhan ... FAILED
🔴 có một `return Ok(())` đứng TRƯỚC bước đọc lại trong `ghi_ref`, tức bước đó là mã
chết. Biên nhận sẽ được cấp mà ref chưa bao giờ được xác nhận — và biên nhận là thứ DUY
NHẤT đường huỷ tin để ghi đè tệp người dùng.
```

**M18: 0 đỏ → ĐỎ.** Commit `b637614`.

### M21 — đối chứng cho chính phép lọc

Thêm `--whitespace=fix --3way --reject --unidiff-zero -C1 pub fn new pub fn moi
return Ok(())` vào **hai** dạng chú thích (`//` và `///`) trong thân không-test:

```text
test result: ok. 350 passed; 0 failed
```

**0 đỏ, đúng như đòi hỏi.** Nếu nó đỏ thì mọi kết luận âm tính của M16/M18 vô giá trị.
Tệp này đặc biệt cần đối chứng: doc comment của nó liệt kê **tên từng cờ bị cấm** và
nguyên văn `pub fn new` để giải thích vì sao **không** có hàm đó.

## Ba điều plan nói mà hoá ra sai — đều đo được, đều đã sửa mã

### 1. `stash create` trả rỗng **không phải** phép kiểm đủ cho ca chưa-theo-dõi

Plan: *"Rỗng → ca tệp chưa theo dõi. Rơi sang `hash-object -w`."*

Đo được, ca **hỗn hợp** phá phép kiểm đó:

```text
repo có CẢ tệp đã theo dõi bẩn LẪN tệp chưa theo dõi:
  git stash create        → 356fea2965dc...      ← sha KHÁC rỗng
  git ls-tree -r 356fea29 → 100644 blob ... a.txt  (CHỈ a.txt)
  git show 356fea29:u.txt → fatal: path 'u.txt' exists on disk, but not in '356fea29'
```

Một cài đặt đọc đúng plan sẽ thấy sha khác rỗng, đi nhánh stash, ghi ref, trả biên
nhận — rồi **xoá tệp chưa theo dõi mà không có bản lưu nào**. Không lỗi, không thông
báo. Đó đúng là dữ liệu WORK-07 tồn tại để giữ.

Sửa: phân nhóm **theo từng đường dẫn** qua `la_chua_theo_doi` (đọc `StatusGroup`), không
theo giá trị trả về của một lời gọi stash. **Test F** ghim ca này, và nó khẳng định cả
hai tiền đề (sha khác rỗng **và** cây không chứa `u.txt`) **bên trong test**.

### 2. `git checkout -- <path>` **không** huỷ được thay đổi đã stage

Plan: *"tệp đã theo dõi → `git checkout -- <path>`"*. Đo được:

```text
thay đổi ĐÃ STAGE, rồi `git checkout -- a.txt`:
  status : [M  a.txt]            ← vẫn trong index
  nội dung: one/STAGED/three     ← KHÔNG huỷ gì cả
cùng ca, `git checkout HEAD -- a.txt`:
  status : []                    ← sạch
```

`git checkout -- <path>` khôi phục từ **index**, và với thay đổi đã stage thì index
*chính là* thứ cần bỏ — nên lệnh đó là thao tác rỗng: người dùng bấm "Huỷ bỏ" và không
có gì bị huỷ. Sửa thành `git checkout HEAD -- <path>`.

### 3. `git checkout <ref> -- <path>` ghi vào **cả index** — khôi phục stage hộ người dùng

Plan: *"ngược lại → `git checkout <ref> -- <paths>`"*. Test C bắt được ngay lần chạy đầu
(0 hunk sau khôi phục thay vì 3). Đo được:

```text
sau `git checkout <ref> -- a.txt`:
  status --porcelain : [M  a.txt]    ← ĐÃ STAGE
  git diff -- a.txt  : 0 hunk        ← worktree khớp index
  git diff HEAD      : 1 hunk        ← nội dung ĐÚNG, chỉ nằm sai tầng
```

Nội dung khôi phục **đúng**, nhưng người dùng nhận nó ở trạng thái **đã stage** — khác
trạng thái trước khi họ bấm huỷ, và lần commit kế tiếp sẽ lặng lẽ mang nó theo. Sửa
bằng `git reset --quiet HEAD -- <paths>` sau bước checkout (đo được: không đụng thư mục
làm việc). Test C nay khẳng định `status` bắt đầu bằng `" M"`, nên hồi quy này không
quay lại trong im lặng.

Đây là ca đáng chú ý: **phép đếm hunk một mình nói dối** — nó trả 0 vì index bẩn, không
vì nội dung sai. Nếu tôi "sửa" test bằng cách đổi sang `git diff HEAD` thì test xanh và
**lỗi UX vẫn còn**.

## Cổng đã giao

**12 cổng đọc nguồn** (mọi cổng **bỏ chú thích** và **cắt `mod tests`** trước khi tìm;
mọi cổng **khẳng định tiền đề** — không tìm thấy ⇒ **đỏ**):

| Cổng | Bảo vệ |
|---|---|
| `phep_loc_chu_thich_hoat_dong` ×2 | **chính phép lọc** — hỏng hai hướng đều tạo cổng hỏng, không hướng nào gây lỗi biên dịch |
| `bien_nhan_khong_co_ham_dung_cong_khai` | T-05-09 — `BienNhan` không dựng được ngoài crate |
| `ref_nam_duoi_dung_khong_gian_rieng` | không `refs/stash`, không `refs/heads`, có `/` cuối |
| `duong_huy_khong_bao_gio_khop_mo` | 5 cờ khớp mờ vắng mặt |
| `co_huy_hunk_mang_dung_tap_co` | `--reverse` + `--recount` có, `--cached` **không** |
| `duong_huy_khong_bao_gio_giai_ma_ban_va` | cắt đúng thân `huy_hunk` rồi mới tìm |
| `khoi_phuc_ghi_byte_tho` | M17 — ghi `Vec<u8>` thẳng ra đĩa |
| `khoi_phuc_kiem_tien_to_ref` | T-05-12 |
| `duong_luu_doc_lai_ref_truoc_khi_cap_bien_nhan` | M18 — và bước đó phải **tới được** |
| `duong_luu_khong_ghep_stderr_vao_sha` | cảnh báo clean filter không lọt vào sha |
| `phan_nhanh_chua_theo_doi_theo_tung_duong_dan` | ca hỗn hợp |
| `moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path` | `--` trước **từng** pathspec |

`GitError` giữ nguyên **14** variant — tôi **không** thêm variant nào (dùng lại
`CommandFailed` / `ParseFailed` / `Io` / `FileChanged`), nên tripwire `==` 14 không phải
đụng tới.

## Requirement

| Req | Trạng thái |
|---|---|
| **WORK-06** | Huỷ theo **tệp** (đã/chưa theo dõi) và theo **khối** chạy đúng trên git thật — Test A, B, C. Tầng Rust **đạt**; đường vào giao diện chưa có |
| **WORK-07** | Lưu trước mỗi lần huỷ, object **tới được** (sống qua `gc --prune=now` — Test D), khôi phục được từ danh sách — Test A, B, C, E. Ràng buộc "không đường nào bỏ qua" cài bằng **kiểu** |

Cả hai ở mức **"tầng Rust đạt, đo bằng git thật"**. Không requirement nào ghi "đạt hoàn
toàn": tiêu chí 4 của phase có phần **chỉ người kiểm được** (danh sách "Vừa huỷ gần đây"
có đọc hiểu được không, người dùng có tìm lại đúng thứ họ vừa huỷ không), và CONTEXT.md
mục 0.3 cấm ghi "Đạt" cho tiêu chí hình ảnh dựa trên test tự động.

## Chưa kiểm — ghi rõ

- **Mọi khẳng định về giao diện.** Plan này không render component nào, theo thiết kế.
  Danh sách "Vừa huỷ gần đây" **chưa có giao diện**; `list_trash` trả dữ liệu nhưng chưa
  ai nhìn nó trên màn hình.
- **`danh_sach_thung_rac` trả `paths` rỗng, `luc: 0`, `nhan: ""`.** `for-each-ref` không
  giữ được ba trường đó — chúng chỉ có lúc `luu_truoc_khi_huy` chạy. Wave giao diện sẽ
  cần một chỗ lưu phụ (ghi chú ref, hay một tệp chỉ mục) để hiện nhãn và thời điểm. Test
  F chỉ dùng `la_blob`/`ref_name`/`object_id` nên nó **không** phủ khoảng trống này.
  🔴 Đây là việc còn lại **có thật**, không phải chi tiết làm đẹp: không có nó, danh sách
  hiện ra không có tên tệp lẫn thời điểm.
- **T-05-11 (`restore_trash` ghi đè tệp đang sửa) — chưa cài.** Plan xếp `mitigate` bằng
  cách gọi `luu_truoc_khi_huy` trước khi khôi phục. Tôi **không** cài: `khoi_phuc` hiện
  ghi đè thẳng. Ghi là **nợ**, không phải đã xong.
- **T-05-10 (trash ref tích luỹ vô hạn)** — `accept` theo plan. Ref cũ không bao giờ bị
  dọn; `GIOI_HAN_DANH_SACH` chỉ chặn phần **hiển thị**. Dọn định kỳ để v2.
- **Ca `index.lock` thật cho đường huỷ.** Dùng lại đường thử lại của 04-02 (có test ghim
  ở đó), nhưng chưa có test nào ép một `index.lock` trong lúc `discard_files` chạy.
- **Tệp nhị phân qua `huy_hunk`.** `dung_ban_va_mot_hunk` trả `None` → `ParseFailed`,
  cùng đường với 05-02, nhưng **chưa có test** ở wave này.
- **Tệp vừa đổi tên vừa sửa** — ROADMAP liệt kê trong ca ngoài đường hạnh phúc; plan này
  không yêu cầu và tôi không thêm.
- **`huy_tep` trên thư mục chưa theo dõi.** `remove_file` không xoá được thư mục; ca đó
  chưa có test và chưa được xử lý tường minh.

## Bẫy môi trường gặp phải

- **Đĩa chạm 100% (0 GB free)** giữa lượt chạy cuối, hỏng build với
  `LINK : fatal error LNK1318: Unexpected PDB error; LIMIT (12)`. Theo đúng cảnh báo của
  orchestrator, tôi **đọc đầu ra trước khi gọi tên nguyên nhân** — bảng tiến trình trông
  y hệt ca chờ khoá. Giải phóng bằng cách xoá cây `CARGO_TARGET_DIR` phụ (4,2 GB) và
  `target/debug/incremental` (1,7 GB): 0 → **5,83 GB**. Cả hai là **cache thuần**.
- **`CARGO_TARGET_DIR` phụ dùng khi `git-plum.exe` (PID 17856, `tauri dev` của phiên
  khác) khoá binary** — **không kill**, cổng dogfood Phase 4 cần nó. Xoá cây phụ ngay
  khi có số. Cuối lượt app đã tự thoát nên phép đo cuối chạy trong `target/` chính.
- **Scratchpad bị một phiên Claude khác xoá giữa chừng**, mang theo bản sao pristine của
  tôi. Dựng lại từ **commit** (`git show b2a3f93:<path> > file`) chứ không `git checkout --`.
  Từ đó `revert.sh` tự kiểm "còn dấu đột biến nào không" và báo đỏ nếu còn.
- **`rtk` lọc stdout của `cargo test`** kể cả khi `> file` — mọi con số trong tài liệu
  này đọc qua `rtk proxy cargo test`, tức libtest thô.
- **`rtk` làm hỏng `grep`**: `grep -n "pub fn ..." src/git/exec.rs` trả *"11 matches in 6
  files"* cho một tệp. Dùng công cụ Grep khi kết quả quan trọng (đã dùng để xác nhận cả
  4 command có trong `generate_handler!`).
- **`$?` sau một đường ống đọc mã thoát của lệnh CUỐI**, không phải của git — một lần
  suýt ghi "update-ref thoát 0" cho ca thật ra thoát 128. Đo lại bằng cách tách lệnh.

## Self-Check: PASSED

Tệp có thật trên đĩa: `src-tauri/src/domain/trash.rs`,
`src-tauri/src/commands/trash.rs`, `src-tauri/tests/trash_commands.rs`,
`.planning/phases/05-hunk-staging/05-03-SUMMARY.md`.

Commit có thật trong `git log --all`: `b2a3f93`, `b637614`.
