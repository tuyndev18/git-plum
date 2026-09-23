---
phase: 05-hunk-staging
plan: 02
subsystem: hunk-staging
tags: [WORK-03, WORK-04, WORK-05, git-apply, recount, byte-tho]
requires:
  - "05-01 (tach_hunk_tho / dung_ban_va_mot_hunk — nguồn bản vá con)"
  - "04-02 (chay_lenh_ghi_co_thu_lai_phan_loai — đường thử lại index.lock, DÙNG LẠI)"
  - "04-02 (lay_trang_thai — kiểu trả về RepoStatus mới)"
provides:
  - "commands::hunk::stage_hunk / unstage_hunk (Tauri command, đã đăng ký)"
  - "commands::hunk::stage_mot_khoi / unstage_mot_khoi (hàm thuần, test gọi được)"
  - "commands::hunk::kiem_blob_hash — WORK-05"
  - "commands::hunk::CO_APPLY — một nguồn sự thật cho tập cờ git apply"
  - "GitError::FileChanged, mã `file_changed`"
affects:
  - "05-03+ (giao diện chọn khối gọi hai command này)"
tech-stack:
  added: []
key-files:
  created:
    - "src-tauri/src/commands/hunk.rs (633 dòng)"
    - "src-tauri/tests/hunk_commands.rs (792 dòng)"
  modified:
    - "src-tauri/src/error.rs (+FileChanged, +3 test)"
    - "src-tauri/src/commands/mod.rs (khai module)"
    - "src-tauri/src/lib.rs (generate_handler!)"
decisions:
  - "Test G (hash rút gọn) được viết thay vì ghi 'M11 chấp nhận được' — và M11 ĐỎ"
  - "Chạy test tích hợp qua CARGO_TARGET_DIR riêng để không phải kill git-plum.exe"
metrics:
  duration: "~1h"
  completed: "2026-09-23"
---

# 05-02 — `stage_hunk` / `unstage_hunk` qua `git apply --cached --recount`

Nối tầng byte thô của 05-01 vào `git apply --cached`, có `--recount`, có kiểm blob hash
trước khi áp, và có `GitError::FileChanged` cho ca tệp đã đổi. Toàn bộ kiểm được **không
cần mở ứng dụng** — đúng ràng buộc "tầng Rust tự đứng được" của CONTEXT.md mục 0.1.

## Số đo — tôi tự chạy, không lấy từ suy luận

| Phép đo | Trước | Sau | Lệnh |
|---|---:|---:|---|
| `cargo test --lib` | 321 | **331** (+10) | `cd src-tauri && cargo test --lib` |
| `cargo test --test hunk_commands` | — | **7 passed, 0 failed** | `CARGO_TARGET_DIR=<scratch> cargo test --test hunk_commands` |
| `cargo clippy --lib` | sạch | **sạch** | `cargo clippy --lib` |
| `cargo test --lib --tests` | 425 (Phase 4) | ✅ **454 passed, 1 ignored** | orchestrator đo sau, xem dưới |

`--lib --tests` **chưa đo được**. Bản **debug** `git-plum.exe` (PID 16124, CommandLine
`"target\debug\git-plum.exe"`) đang chạy từ `tauri dev` của một phiên khác, nên cargo
không relink được binary:

```text
error: failed to remove file `...\target\debug\git-plum.exe`
Caused by: Access is denied. (os error 5)
```

**Không kill** — tiến trình không phải của phase này, và cổng dogfood của Phase 4 cần
ứng dụng chạy được. Ghi "chưa đo", **không** suy ra tổng.

> ✅ **Đo bổ sung 2026-09-23, orchestrator.** Sau khi ứng dụng đóng và tôi dọn ba tiến
> trình `cargo` bị kẹt:
>
> ```text
> cargo test --lib --tests
> cargo test: 454 passed, 1 ignored (18 suites, 40.19s)
> ```
>
> Từ 425 (cuối Phase 4) lên **454** — cộng đúng phần wave 1 và wave 2 thêm vào. **0 đỏ.**
>
> Ba tiến trình kẹt đã dọn: 62 phút / 45 phút / 10 phút, tất cả ở **0,15–0,33 giây CPU**
> — xếp hàng trên `target/debug/.cargo-lock`, không biên dịch gì. Đây là ca thứ ba
> trong ngày và nó xác nhận quy tắc ở `CONTEXT.md` §5: **phân biệt bằng CPU, không bằng
> thời gian trôi**, và **luôn in `CommandLine`** — một trong các tiến trình tôi suýt
> giết nhầm là `tauri dev` của chủ dự án.

Lần thử thứ hai (sau khi commit) còn gặp **nguyên nhân thứ hai**: lệnh chạy quá 10 phút
mà stdout vẫn 0 byte. Phân biệt "bị chặn" với "đang làm" bằng **CPU, không bằng thời
gian trôi** (CONTEXT.md mục 5):

```text
ProcessId Name      CPUs
    58932 cargo.exe  0.2
    65396 rustc.exe  1.0
    16804 cargo.exe  0.1
```

0,1–1 giây CPU sau nhiều phút = **bị chặn**, đang xếp hàng trên
`target/debug/.cargo-lock` sau các phiên Claude khác (3–5 phiên dùng chung một `target/`).
Không phải lỗi của thay đổi này. `--lib --tests` vẫn **chưa đo**.

### Cách chạy được test tích hợp mà không kill app

`cargo test --test hunk_commands` vẫn dựng bin target theo mặc định, nên nó chạm tệp bị
khoá. Đặt `CARGO_TARGET_DIR` sang thư mục scratchpad tránh hoàn toàn đường đó: cargo
dựng vào cây khác, không đụng `target/debug/git-plum.exe`. Giá phải trả là một lần biên
dịch lại đầy đủ (1 phút 57), nhưng nó **không** đòi giết tiến trình của ai.

## Bảng đột biến — chạy thật, dán đầu ra đỏ

Hoàn nguyên **bằng `cp` từ bản sao trong scratchpad**, không bao giờ `git checkout --`
(CONTEXT.md 4.1). Sau khi xong: `diff` với bản sao cho **rỗng** trên cả hai tệp, và
`cargo test --lib` về lại 331.

| # | Đột biến | Đỏ mong đợi | **Đo được** |
|---|---|---|---|
| M6 | Bỏ `--recount` khỏi `CO_APPLY` | Test B bước 3 | ✅ **3 đỏ** — Test B, cộng 2 cổng `--lib` |
| M7 | Bỏ bước `--check`, áp thẳng | Test D — index bẩn | ⚠️ **1 đỏ, KHÔNG phải Test D** — xem dưới |
| M8 | `kiem_blob_hash` luôn `Ok(())` | Test D | ✅ **2 đỏ** — Test D + Test G |
| M9 | `dung_ban_va_mot_hunk(i)` → `(0)` | Test A | ✅ **4 đỏ** — A, C, E, F |
| M10 | Ghép `stderr` vào bản vá | Test A hoặc C | ⚠️ **1 đỏ, chỉ ở cổng nguồn** — xem dưới |
| M11 | So hash bằng `starts_with` | (dự đoán: không đỏ) | ✅ **1 đỏ** — Test G. **Dự đoán của plan SAI** |
| M12 | Thêm `--whitespace=fix` vào `CO_APPLY` | cổng Task 3 | ✅ **1 đỏ** |
| M13 | Thêm `--3way` **chỉ trong chú thích** | 🔴 **không được đỏ** | ✅ **0 đỏ** — phép lọc chú thích hoạt động |

### M6 — và bằng chứng fixture `--recount` phân biệt được

Chạy đột biến cho đỏ ở **tiền đề**, đúng thiết kế (không tìm thấy ⇒ đỏ):

```text
🔴 tiền đề sai: `CO_APPLY` không có `--recount`, test này không kiểm gì.
   CO_APPLY = ["apply", "--cached"]
```

Nhưng tiền đề đỏ **trước** bước 3, nên một mình nó **chưa chứng minh fixture phân biệt
được** — nó chỉ chứng minh hằng số bị đổi. Nên tôi tắt tạm assert tiền đề và chạy lại
với M6 còn nguyên, để xem **chính bước 3** nói gì:

```text
thread 'bo_recount_lam_ban_va_cat_than_that_bai' panicked at tests\hunk_commands.rs:337:5:
assertion `left == right` failed: 🔴 VỚI `--recount`, bản vá cắt-thân phải áp được...
  git nói: error: corrupt patch at cat_than.patch:12
  left: 128
 right: 0
```

**Fixture phân biệt được, đo trong chính test.** Cộng phép đo độc lập tôi chạy trên git
2.54.0.windows.1 trước khi viết mã (repo 26 dòng `a`..`z`, sửa dòng 2/13/25, `-U3`):

```text
hunk 2 NGUYÊN VĂN (@@ + 8 dòng thân):
  git apply --check --cached --recount h2.patch  → exit 0
  git apply --check --cached           h2.patch  → exit 0      ← KHÔNG phân biệt

hunk 2 CẮT THÂN (@@ + 6 dòng thân, header vẫn khai 7):
  git apply --check --cached --recount h2t.patch → exit 0
  git apply --check --cached           h2t.patch → exit 128
                              error: corrupt patch at h2t.patch:12
```

Khớp MEASURED.md mục 1–2 chính xác. Lần dựng fixture **đầu tiên** của tôi lệch một dòng
(`slice(i, i+8)` thay vì `i+9`) và cho kết quả trái ngược — hunk "nguyên văn" thành đỏ.
Tôi không ghi con số đó xuống mà đi tìm nguyên nhân; bài học đúng của mục 4.2 là **đo
rồi nhìn kỹ**, vì một fixture lệch một dòng vẫn cho ra số. Test B vì thế mang thêm một
assert rằng bản vá cắt-thân **khác** bản nguyên văn, để phép cắt trượt không bao giờ âm
thầm biến fixture thành bản không-phân-biệt-được.

### M7 — plan đoán sai chỗ đỏ, và lý do đáng ghi

Plan ghi *"M7 → Test D, index bị bẩn"*. **Đo được: Test D vẫn XANH**, cả 7 test tích hợp
xanh; chỉ cổng nguồn `co_buoc_check_truoc_khi_ap_that` đỏ.

Nguyên nhân là thứ tự các bước: `kiem_blob_hash` đứng **trước**, nên ở Test D lời gọi
trả `FileChanged` và **không bao giờ tới** bước apply — bỏ `--check` không quan sát được
qua đường đó. **Đột biến không biểu diễn được lỗi thì không nói gì về cổng**
(CONTEXT.md 4.1), nên tôi không dừng ở "M7 sống sót".

Kiểm chứng bằng cách chạy **M7 + M8 cùng lúc**, tức mở đường cho lời gọi đi tới apply:

```text
test tep_doi_sau_khi_ve_diff_cho_file_changed_va_index_van_sach ... FAILED
test hash_rut_gon_khong_duoc_coi_la_khop ... FAILED
test result: FAILED. 5 passed; 2 failed
```

Vậy Test D **có** đo được đường apply khi đường đó tới được; M7 một mình chỉ không tới.
Phép kiểm hành vi cho `--check` nằm ở M8, và phép kiểm cấu trúc nằm ở cổng nguồn.

### M10 — đỏ ở cổng nguồn, KHÔNG đỏ ở hành vi, và đó là giới hạn thật

```text
test commands::hunk::tests::stderr_khong_bao_gio_vao_ban_va ... FAILED
```

Bảy test tích hợp **vẫn xanh**. Trước khi kết luận "cổng đủ", tôi đo xem đột biến có
**biểu diễn được** trong fixture không — `git diff` trên repo `autocrlf=true`, tệp CRLF,
bắt hai luồng **tách riêng**:

```text
stderr bytes: 0
stderr content: []
```

`git diff` **không in gì ra stderr** ở các fixture này, nên ghép `stderr` vào bản vá là
ghép một chuỗi rỗng — một đột biến **vô hiệu** ở tầng hành vi. Ghi rõ: **M10 chỉ được
cổng đọc-nguồn bắt.** Một fixture ép được clean filter kêu (cần một `.gitattributes`
filter tự viết, hoặc một tệp mà git cảnh báo lúc diff) sẽ phủ được tầng hành vi; nó
**chưa có** và tôi không tuyên bố là có.

### M11 — dự đoán của plan sai, và lỗi #9 là lý do tôi viết Test G

Plan dự đoán *"không đỏ — hash đủ dài thì tiền tố hiếm khi khớp"*, và bảo: nếu 0 đỏ thì
**kiểm xem có test nào hỏi về điều này chưa** (lỗi #9) rồi viết test dựng hash cắt ngắn,
thay vì ghi "chấp nhận được". Tôi viết Test G ngay từ đầu vì không có test nào hỏi.

Kết quả đo: **M11 ĐỎ.**

```text
🔴 hash RÚT GỌN phải bị từ chối. `starts_with` sẽ chấp nhận nó; `==` từ chối.
Một phép so tiền tố biến 'tệp đã đổi' thành 'tệp không đổi' đúng lúc nó nguy
hiểm nhất — trên repo lớn, một tiền tố khớp với một blob KHÁC
```

Đỏ **vì** Test G tồn tại. Không có nó, M11 sẽ là một ca lỗi #9 hoàn hảo: mã sai, 331
test xanh, và không ai biết vì không ai từng hỏi.

### M13 — đột biến kiểm chính cổng

Thêm `--3way` và `--reject` vào **hai** dạng chú thích (`//` và `///`) ngay trên
`CO_APPLY`, trong thân không-test:

```text
test result: ok. 331 passed; 0 failed
```

**0 đỏ, đúng như đòi hỏi.** Nếu nó đỏ thì cổng đang khớp văn xuôi và mọi kết luận khác
từ M12 vô giá trị. Tệp này đặc biệt nguy hiểm ở điểm đó: doc comment của nó **liệt kê
tên từng cờ bị cấm** để giải thích vì sao không dùng.

## Cổng đã giao

Bảy cổng đọc nguồn trong `hunk.rs` (mọi cổng **bỏ chú thích** và **cắt `mod tests`**
trước khi tìm, và mọi cổng **khẳng định tiền đề** — không tìm thấy ⇒ **đỏ**):

| Cổng | Bảo vệ |
|---|---|
| `phep_loc_chu_thich_hoat_dong` | **chính phép lọc** — không có nó, lọc hỏng hai hướng đều tạo cổng hỏng, không hướng nào gây lỗi biên dịch |
| `duong_apply_khong_bao_gio_khop_mo` | 5 cờ khớp mờ vắng mặt **+ `--recount` có mặt** (cổng phủ định một mình xanh trên tệp rỗng lệnh) |
| `co_buoc_check_truoc_khi_ap_that` | `--check` có, và đứng **trước** lần áp thật |
| `duong_apply_khong_bao_gio_giai_ma_ban_va` | 4 dạng giải mã chuỗi vắng mặt trên đường bản vá **+ `stdin_bytes` có mặt** |
| `stderr_khong_bao_gio_vao_ban_va` | `ra_diff.stdout` có, `ra_diff.stderr` không |
| `moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path` | `--` trước pathspec, kiểm **từng lệnh** chứ không toàn cục |
| `co_apply_mang_recount_va_cached` | `--recount`, `--cached`, và `apply` đứng đầu argv |

Ba test trong `error.rs`: mã riêng, **toàn bộ khoá JSON so bằng** (không phải
`contains_key` — đó là cách lỗi `rename_all` Phase 3 bị bắt), và thông điệp chứa nguyên
văn `"hãy làm mới"`.

`moi_ma_loi_khac_nhau_doi_mot` cập nhật 13 → **14**. Con số tuyệt đối giữ nguyên dạng
`==`, không nới thành `>=`: phép so bằng chính là thứ bắt được một variant thêm vào mà
quên khai mã lỗi.

## Requirement

| Req | Trạng thái |
|---|---|
| **WORK-03** | Stage/unstage **một khối** chạy đúng trên git thật — Test A, F. Tầng Rust **đạt**; đường vào giao diện chưa có (wave sau) |
| **WORK-04** | CRLF (29 `\r` không đổi), thiếu dòng cuối, byte `0xE9` — giữ nguyên trong **cả** worktree lẫn blob đã stage. Đo trên `Vec<u8>`, `autocrlf=true` tường minh — Test C |
| **WORK-05** | Tệp đã đổi → `file_changed` **và index sạch** — Test D. Cộng ca hash rút gọn — Test G |

Ba requirement ở mức **"tầng Rust đạt, đo bằng git thật"**. Không requirement nào được
ghi "đạt hoàn toàn": tiêu chí thành công 1/3/4 của phase có phần **chỉ người kiểm được**
(người dùng có chọn đúng khối mình muốn không, thông báo có đọc hiểu được không), và
CONTEXT.md mục 0.3 cấm ghi "Đạt" cho tiêu chí hình ảnh dựa trên test tự động.

## Chưa kiểm — ghi rõ

- **`cargo test --lib --tests`** — bị khoá bởi `git-plum.exe` đang chạy. **Chưa đo**,
  không suy ra tổng.
- **M10 ở tầng hành vi** — đột biến vô hiệu trên fixture hiện có (`git diff` cho stderr
  rỗng, đã đo). Chỉ cổng đọc-nguồn phủ được nó.
- **Ca `index.lock` thật cho đường apply.** Đường thử lại được **dùng lại** từ 04-02
  (có test ghim ở đó), nhưng chưa có test nào ép một `index.lock` trong lúc
  `stage_hunk` chạy. Đường mã là chung nên rủi ro thấp, nhưng nó **chưa được đo ở đây**.
- **Tệp vừa đổi tên vừa sửa** — ROADMAP liệt kê trong ca ngoài đường hạnh phúc; plan
  này không yêu cầu và tôi không thêm. Chưa có test.
- **Mọi khẳng định về giao diện.** Plan này không render component nào, theo thiết kế.
- **Bảng đột biến của 05-01 vẫn chưa đầy đủ** — tôi không mở rộng nó; tôi chỉ **không
  đổi hành vi** của `patch_build.rs`, nên rủi ro ở đó giữ nguyên như 05-01-SUMMARY mô tả.

## Điều plan nói mà hoá ra sai

1. **M7 → Test D.** Sai. `kiem_blob_hash` đứng trước nên Test D không tới được bước
   apply; M7 chỉ đỏ ở cổng nguồn. Chứng minh bằng M7+M8 (2 test tích hợp đỏ).
2. **M11 "dự đoán: không đỏ".** Sai — **đỏ**, vì Test G hỏi đúng về thuộc tính đó.
3. **M10 → "Test A hoặc C".** Sai. `git diff` cho stderr rỗng ở các fixture này, nên
   đột biến không biểu diễn được ở tầng hành vi.

## Self-Check: PASSED

Tệp có thật trên đĩa: `src-tauri/src/commands/hunk.rs`,
`src-tauri/tests/hunk_commands.rs`, `.planning/phases/05-hunk-staging/05-02-SUMMARY.md`.
Commit có thật trong `git log --all`: `0a7fa04`, `bf4405c`.
