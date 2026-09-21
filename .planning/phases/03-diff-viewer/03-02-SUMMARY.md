---
phase: 03-diff-viewer
plan: 02
subsystem: diff-viewer
tags: [backend, parser, cache, diff-06, ipc-contract]
requires:
  - "src-tauri/src/git/exec.rs (GitCommand, --no-ext-diff tập trung từ 03-01)"
  - "src-tauri/src/git/runner.rs (GitRunner, CommandLog)"
  - "src-tauri/src/commands/history.rs (mẫu repo_cua, EMPTY_TREE, parse_name_status)"
  - "src-tauri/src/cache/repo_cache.rs (mẫu cache khoá theo RepoId)"
provides:
  - "domain::diff — FileDiff, DiffKind (5 dạng), Hunk, DiffLine, LineKind"
  - "git::parsers::patch::parse_patch(&[u8]) -> PatchParse — bộ phân tích unified diff viết tay"
  - "cache::DiffCache — LRU 200 mục khoá (repo_id, sha, path), không bao giờ vô hiệu hoá theo thời gian"
  - "commands::get_file_diff — cổng DIFF-06 chạy TRƯỚC git diff"
  - "scripts/fixtures/make-diff-fixtures.sh — 15 hình dạng tệp, 26 commit, in bảng SHA"
  - "testing::require_diff_fixture — bỏ qua ồn ào khi thiếu fixture"
  - "ipc.getFileDiff + kiểu TS discriminated union"
affects:
  - "03-03 (diff mức từ) — ba fixture empty-line/one-side/mixed đã sinh và đã kiểm hình dạng"
  - "03-04 (giao diện) — dựng UI từ FileDiff; ngưỡng 5 MB hiện trong thông báo"
  - "Phase 5 (staging theo khối) — dùng lại parse_patch ở phía Rust"
tech-stack:
  added: []
  patterns:
    - "Cổng an ninh neo vào QUYẾT ĐỊNH (vị trí return), không vào phép ĐO"
    - "Cổng đọc mã nguồn phải cắt theo từng lệnh git, không kiểm cả thân hàm như một khối"
    - "serde rename_all phải lặp lại trên từng biến thể enum mang dữ liệu"
key-files:
  created:
    - "src-tauri/src/domain/diff.rs"
    - "src-tauri/src/git/parsers/patch.rs"
    - "src-tauri/src/cache/diff_cache.rs"
    - "src-tauri/src/commands/diff.rs"
    - "src-tauri/tests/diff_commands.rs"
    - "scripts/fixtures/make-diff-fixtures.sh"
    - "src/lib/ipc.diff.test.ts"
  modified:
    - "src-tauri/src/domain/mod.rs"
    - "src-tauri/src/git/parsers/mod.rs"
    - "src-tauri/src/cache/mod.rs"
    - "src-tauri/src/commands/mod.rs"
    - "src-tauri/src/state/mod.rs"
    - "src-tauri/src/testing/mod.rs"
    - "src-tauri/src/lib.rs"
    - "src/lib/ipc.ts"
decisions:
  - "Ngưỡng DIFF-06 là 5 MB mỗi phía, dùng max(cũ, mới) — ngưỡng của TRÌNH XEM, không của máy"
  - "Nhận biết LFS bằng nội dung blob, không check-attr — đã đo lại và xác nhận"
  - "Khoá cache là (repo_id, sha, path) dạng struct; -w về sau PHẢI vào khoá, chế độ hiển thị thì không"
  - "🔴 --name-status chạy KHÔNG pathspec — plan sai, xem mục Phát hiện đo được"
metrics:
  duration: "~85 phút"
  completed: "2026-09-22"
  tasks_completed: "3/3"
  tests_added: "53 Rust + 7 frontend"
  tests_total: "216 Rust (+1 ignored), 233 frontend"
---

# Phase 3 Plan 02: Backend trình xem diff Summary

Hợp đồng dữ liệu diff, bộ phân tích unified diff viết tay, cache LRU khoá
`(repo_id, sha, path)`, và `get_file_diff` với cổng DIFF-06 chạy **trước** `git diff` —
cộng ba phát hiện mà đầu ra git thật khác điều plan dự đoán.

---

## Chữ ký cuối và hình dạng JSON (03-04 dựng giao diện từ đây)

```rust
// Tauri command
get_file_diff(repo_id: String, commit_id: String, path: String) -> Result<FileDiff>

// Hàm logic mà test tích hợp gọi (không cần webview)
pub async fn lay_diff_tep(
    state: &AppState, repo: Arc<RepoHandle>, commit_id: &str, path: &str,
) -> Result<Arc<FileDiff>>
```

Phía JS: `ipc.getFileDiff(repoId, commitId, path)` → `invoke('get_file_diff', { repoId, commitId, path })`.

**Hình dạng JSON thật của cả năm dạng** (đã ghim bằng test ở `domain/diff.rs`):

```jsonc
// 1. text
{ "path": "a.txt", "oldPath": null, "status": "M",
  "kind": { "kind": "text", "hunks": [ /* Hunk[] */ ], "truncated": false } }

// 2. binary — KHÔNG có trường nội dung nào, chặn ở tầng KIỂU (T-03-13)
{ "path": "b.png", "oldPath": null, "status": "M",
  "kind": { "kind": "binary", "oldSize": 4096, "newSize": 8192 } }

// 3. tooLarge — hiện cả hai số để người dùng biết vượt bao nhiêu
{ "path": "large.txt", "oldPath": null, "status": "M",
  "kind": { "kind": "tooLarge", "size": 5999916, "limit": 5242880 } }

// 4. lfsPointer — size là số TRONG con trỏ, không phải cỡ tệp con trỏ (~130 byte)
{ "path": "pointer.bin", "oldPath": null, "status": "M",
  "kind": { "kind": "lfsPointer", "oid": "<64 hex>", "size": 1048576 } }

// 5. unchanged — commit chỉ đổi mode tệp
{ "path": "mode-only.txt", "oldPath": null, "status": "M",
  "kind": { "kind": "unchanged" } }
```

`Hunk` và `DiffLine`:

```jsonc
{ "oldStart": 3, "oldCount": 3, "newStart": 3, "newCount": 4,
  "heading": "fn main()",
  "lines": [
    { "kind": "context", "content": "a", "oldLine": 3, "newLine": 3,    "noNewlineAtEof": false },
    { "kind": "removed", "content": "b", "oldLine": 4, "newLine": null, "noNewlineAtEof": false },
    { "kind": "added",   "content": "B", "oldLine": null, "newLine": 4, "noNewlineAtEof": false }
  ] }
```

⚠️ `DiffKind` **lồng dưới khoá `kind`** của `FileDiff`, và tự nó có khoá `kind` phân
biệt dạng — nên đường truy cập là `fileDiff.kind.kind`. Có test ghim để 03-04 không đoán.

---

## Ngưỡng 5 MB — con số mà 03-04 phải hiển thị

`MAX_DIFF_BLOB_BYTES = 5 * 1024 * 1024` = **5.242.880 byte**, áp cho **mỗi phía** bằng
`max(cũ, mới)`.

Lý do (ghi lại để 03-04 viết đúng thông báo, và để người sau không "nới cho thoáng"):

- Ngưỡng phải là mức mà **trình xem** còn dùng được, không phải mức máy còn chịu nổi.
  5 MB văn bản là ~100 nghìn dòng; CodeMirror ảo hoá được, nhưng **tính** diff hay dựng
  decoration trên 100 nghìn dòng đã ngoài mọi ngân sách của checkpoint #3.
- 5 MB cũng là ngưỡng GitHub từ chối hiện diff — mốc đã kiểm nghiệm trên rất nhiều người dùng.
- `max(cũ, mới)` chứ không phải chỉ phía mới: một tệp 10 MB **bị xoá** có phía mới 0 byte
  và vẫn phải bị chặn.

Thông báo nên nêu **cả hai** số ("tệp 5,7 MB, vượt ngưỡng 5,0 MB"), không chỉ "quá lớn".

---

## 🔴 Phát hiện đo được — chỗ git thật khác điều plan dự đoán

Đây là loại phát hiện mà plan gọi là giá trị nhất (`%x1f` và `%(*objectname)` của Phase 2).
Có **ba** ca, một trong đó là lỗi thật của plan.

### 1. 🔴 Plan SAI: `--name-status` **không được** mang pathspec

Plan Task 3 bước 6 ngầm định lệnh `--name-status` chạy kèm `-- <path>`. Đo thật trên
git 2.54.0.windows.1, commit vừa-đổi-tên-vừa-sửa (`renamed.txt` → `renamed-new.txt`, 77% giống):

```
$ git diff --name-status -z --find-renames A B -- renamed-new.txt
A\0renamed-new.txt\0                            ← "tệp MỚI"

$ git diff --name-status -z --find-renames A B
R077\0renamed.txt\0renamed-new.txt\0            ← đổi tên, 77%
```

**Nguyên nhân:** git phát hiện đổi tên bằng cách ghép **tập** tệp bị xoá với **tập** tệp
được thêm. Pathspec lọc `renamed.txt` ra khỏi tập đầu vào **trước khi** phép ghép chạy,
nên git chỉ còn thấy một tệp xuất hiện từ hư không.

**Hệ quả nghiêm trọng hơn chữ trạng thái sai** — cùng lý do đó, lệnh sinh bản vá cũng hỏng:

```
$ git diff --unified=3 --find-renames A B -- renamed-new.txt
diff --git a/renamed-new.txt b/renamed-new.txt
new file mode 100644
--- /dev/null
+++ b/renamed-new.txt
@@ -0,0 +1,8 @@
+alpha
+BETA DA SUA          ← TOÀN BỘ 8 dòng là dòng thêm

$ git diff --unified=3 --find-renames A B -- renamed.txt renamed-new.txt
diff --git a/renamed.txt b/renamed-new.txt
similarity index 77%
rename from renamed.txt
rename to renamed-new.txt
@@ -1,5 +1,5 @@      ← hunk THẬT: 1 dòng thêm, 1 dòng xoá
```

Người dùng bấm vào một tệp đổi tên sẽ thấy **cả tệp sáng xanh** thay vì thấy một dòng đã sửa.

**Đã sửa assertion của plan:** `--name-status` chạy **không** pathspec rồi tự tìm bản ghi
khớp (`chon_ban_ghi_khop`); lệnh diff thật truyền **cả hai** đường dẫn khi có `old_path`.
Có hai test ghim: một test đơn vị trên byte thật (`chon_dung_ban_ghi_trong_name_status`,
gồm ca bản ghi `R` nằm giữa để bắt lỗi lệch-một-nấc), và một test hồi quy đọc mã nguồn
(`lenh_name_status_khong_mang_pathspec`). Test tích hợp đếm `(1 thêm, 1 xoá)` chứ không
chỉ kiểm `status` — con số `(8, 0)` là dấu hiệu lỗi đã quay lại.

### 2. `serde(rename_all)` ở cấp enum **không** chạm tên trường bên trong biến thể

`#[serde(tag = "kind", rename_all = "camelCase")]` đặt ở cấp enum chỉ đổi tên **biến thể**
(`LfsPointer` → `lfsPointer`). Trường bên trong giữ nguyên `snake_case`:

```
left:  ["kind", "new_size", "old_size"]     ← thực tế
right: ["kind", "newSize",  "oldSize"]      ← điều ipc.ts đọc
```

Không lỗi biên dịch ở bên nào; giao diện chỉ nhận `undefined` và hiện kích thước trống —
đúng lớp lỗi mà CONTEXT.md gọi là "lệch một bên không gây lỗi biên dịch ở cả hai phía".

Bị bắt vì test `binary_khong_mang_byte_noi_dung` liệt kê **toàn bộ** khoá JSON rồi so bằng,
chứ không chỉ kiểm khoá mình mong có mặt. **Sửa:** lặp `#[serde(rename_all = "camelCase")]`
trên từng biến thể mang dữ liệu.

### 3. Ba `<measured_facts>` của plan được kiểm chứng lại — đều đúng

Chạy trên fixture mới, gọi `/mingw64/bin/git` trực tiếp (rtk chèn khối `--- Changes ---`
vào stdout của `git diff`):

| Sự thật | Kết quả |
|---|---|
| `--numstat -z` cho tệp nhị phân | `-  \t  -  \t  b i n a r y . p n g  \0` ✅ |
| `cat-file --batch-check` cho cỡ blob | `63a0a21b... blob 5999916` ✅ (biết cỡ mà không đọc byte nào) |
| `check-attr -z filter` trên con trỏ LFS thật, repo không `.gitattributes` | `pointer.bin\0filter\0unspecified\0` ✅ |

Mục 3 xác nhận quyết định câu 3 của CONTEXT.md: `check-attr` **không** trả lời được câu
đang hỏi, nên nhận biết LFS bằng nội dung blob.

Đầu ra `--word-diff=porcelain` của `mixed.txt` cũng khớp **từng byte** điều plan dán
(kiểm bằng `cat -A`, thấy dấu cách trần ở dòng rỗng):

```
@@ -1,4 +1,4 @@$
 a$
~$
 $          ← dòng rỗng: một dấu cách rồi hết dòng
~$
-DELETED$
~$
 keep$
~$
+ADDED$
~$
```

---

## Kiểm mutation — 9/9 đã chạy, **ba** cổng ban đầu vô dụng phải sửa

Bảng đầy đủ. Cột cuối là output đỏ thật, không diễn giải.

| # | Đột biến | Kết quả | Test đỏ |
|---|---|---|---|
| 1 | `unwrap_or(1)` → `unwrap_or(0)` cho `@@ -1 +1 @@` | ✅ 1 đỏ | `so_dem_khuyet_la_mot_khong_phai_khong` |
| 2 | bỏ tăng `new_line` ở dòng `Added` | ✅ 2 đỏ | `so_dong_dung_cho_ca_ba_loai` **(đúng test số dòng)** |
| 3 | bỏ bước cắt `\r` | ✅ 1 đỏ | `crlf_khong_lot_vao_noi_dung` |
| 4 | `DiffKey` struct → chuỗi nối `format!` | ✅ 1 đỏ | `ranh_gioi_truong_khong_bi_nhap_nhang` |
| 5 | `MAX_CACHED_DIFFS` 200 → 1 | ✅ 6 đỏ | gồm `chan_tren_dung_hai_tram` **(đúng cổng giá trị hằng)** |
| 6 | dời cổng kích thước xuống sau `git diff` | ⚠️ **cổng đơn vị XANH lần đầu** → sửa → ✅ đỏ | `too_large_khong_chay_git_diff_nao_tren_tep_do` (tích hợp) + `cong_kich_thuoc_nam_truoc_moi_lenh_diff` (sau khi sửa) |
| 7 | bỏ bước tra cache | ✅ 1 đỏ | `goi_lan_hai_khong_sinh_tien_trinh_git_nao` |
| 8 | xoá `.arg("--")` khỏi lệnh diff thật | ⚠️ **0 đỏ lần đầu** → sửa cổng → ✅ 1 đỏ | `moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path` |
| 9 | `LfsPointer.size` = cỡ blob con trỏ | ✅ 1 đỏ | `lfs_pointer_doc_oid_va_size_tu_noi_dung_con_tro` |

### Output đỏ thật (trích nguyên)

**#1**
```
panicked at src\git\parsers\patch.rs:382:9:
assertion `left == right` failed: số đếm khuyết ở phía cũ phải là 1
  left: 0
 right: 1
test result: FAILED. 13 passed; 1 failed
```

**#2** — đúng test nói về số dòng cột mới, không phải một test tình cờ đỏ:
```
panicked at src\git\parsers\patch.rs:342:9:
assertion `left == right` failed: dòng `added` thứ hai phải tăng bộ đếm cột mới
  left: (None, Some(4))
 right: (None, Some(5))
test result: FAILED. 12 passed; 2 failed
```

**#3**
```
panicked at src\git\parsers\patch.rs:520:13:
nội dung dòng không được kết thúc bằng `\r` — nó là ký tự vô hình làm lệch mọi phép
so nội dung. Nhận: "a\r"
test result: FAILED. 13 passed; 1 failed
```

**#4** — ca biên giới trường `("abc","d")` vs `("ab","cd")`:
```
panicked at src\cache\diff_cache.rs:245:9:
assertion `left == right` failed: hai cặp khác nhau phải là hai mục; nối chuỗi làm
chúng trùng khoá
  left: 1
 right: 2
test result: FAILED. 8 passed; 1 failed
```

**#5** — kiểm **đúng test nào** đỏ, không chỉ đếm. Sáu test đỏ, và cổng đúng có trong đó:
```
cache::diff_cache::tests::chan_tren_dung_hai_tram          ← cổng giá trị hằng
cache::diff_cache::tests::invalidate_repo_chi_xoa_repo_do
cache::diff_cache::tests::khoa_phan_biet_ca_sha_lan_path
cache::diff_cache::tests::la_lru_that_khong_phai_fifo
cache::diff_cache::tests::ranh_gioi_truong_khong_bi_nhap_nhang
state::tests::closing_repo_frees_its_diff_cache
```

**#6** — mutation quan trọng nhất của plan. Test tích hợp đỏ kèm **bằng chứng lệnh đã chạy**:
```
panicked at tests\diff_commands.rs:146:5:
KHÔNG lệnh `git diff` nào được chạy trên tệp vượt ngưỡng — cổng phải chặn TRƯỚC,
không lọc SAU. Lệnh đã chạy: [
    "git -c core.quotepath=false diff --unified=3 --find-renames eebf3e3a... 65062dff... -- large.txt",
    "git -c core.quotepath=false diff --numstat -z --find-renames eebf3e3a... 65062dff... -- large.txt",
]
test result: FAILED. 13 passed; 1 failed
```

**#7**
```
panicked at tests\diff_commands.rs:406:5:
assertion `left == right` failed: lần gọi thứ hai cho cùng (sha, path) phải thêm
ĐÚNG 0 entry vào nhật ký. Thêm 6 entry nghĩa là cache không được tra, và tiêu chí
thành công số 5 của phase trượt.
  left: 12
 right: 6
test result: FAILED. 13 passed; 1 failed
```

**#9** — 132 byte (tệp con trỏ) so với 1.048.576 byte (tệp thật):
```
panicked at tests\diff_commands.rs:185:5:
assertion `left == right` failed: size phải là số TRONG con trỏ (kích thước tệp thật),
KHÔNG phải kích thước của tệp con trỏ (~130 byte)
  left: 132
 right: 1048576
test result: FAILED. 13 passed; 1 failed
```

---

## Hai cổng vô dụng phải sửa — chi tiết, vì chúng là bài học

Plan đòi ghi cả hai lượt "như 02-06 đã ghi cho `DEBOUNCE_MS` và `.arg("--")`".

### Mutation #6: cổng đơn vị neo vào phép **ĐO**, không vào **QUYẾT ĐỊNH**

Cổng đầu tiên (`cong_kich_thuoc_nam_truoc_moi_lenh_diff`) so vị trí của
`kich_thuoc_hai_phia` với vị trí lệnh `diff`. Đột biến dời **khối `if ... return TooLarge`**
xuống sau `git diff` nhưng **để nguyên lời gọi đo ở chỗ cũ** — nên phép so vị trí vẫn đúng
và cổng **xanh** với một cài đặt đã thành "lọc sau". Chỉ test tích hợp đọc `CommandLog` bắt được.

**Sửa:** neo vào `DiffKind::TooLarge` (vị trí `return`). Thứ quyết định "chặn trước hay
lọc sau" là vị trí của `return`, không phải vị trí của phép đo. Chạy lại mutation → đỏ:

```
panicked at src\commands\diff.rs:870:9:
nhánh trả `TooLarge` phải `return` TRƯỚC mọi lệnh `diff`. Chặn-trước và lọc-sau trả
về CÙNG một giá trị, nên chỉ vị trí của `return` này phân biệt được chúng (T-03-10)
```

### Mutation #8: cổng đo đại lượng **toàn cục** cho một bất biến **cục bộ**

Cổng đầu tiên tìm vị trí `--` **đầu tiên** trong thân hàm rồi đòi mọi mốc pathspec nằm sau
nó. Xoá `.arg("--")` khỏi **lệnh diff thật** cho **0 test đỏ**: `--` của lệnh `--numstat`
phía trên vẫn còn và nằm trước mọi mốc, nên phép so vị trí vẫn đúng.

Đây đúng là lỗi HIST-10 của Phase 2 lặp lại ở dạng tinh vi hơn — cổng đọc đúng tệp, đúng
hàm, đúng ý định, nhưng đo một đại lượng toàn cục cho một bất biến cục bộ.

**Sửa:** cắt thân hàm theo `GitCommand::new` (mỗi lần xuất hiện là một lệnh mới) rồi kiểm
`--` trong phạm vi **của chính lệnh đó**. Cộng một khẳng định `so_lenh_co_pathspec == 2`
để phép cắt lệch cũng bị bắt. Chạy lại mutation → đỏ:

```
lệnh git thứ 3 mang pathspec nhưng KHÔNG có `--` ngăn cách revision với nó (T-03-07).
Thiếu `--`, một `path` trùng tên nhánh (`main`, `HEAD`) được git đọc thành REVISION,
và `path` đến từ webview nên nó là chuỗi tuỳ ý.
test result: FAILED. 12 passed; 1 failed
```

### Một cổng thứ ba, do clippy bắt

`assert!(MAX_CACHED_DIFFS > MAX_CACHED_HISTORIES)` — **cả hai vế là hằng số nên phép so
được tính lúc biên dịch** và assertion không quan sát được gì lúc chạy
(`clippy::assertions_on_constants`). Cùng loại "cổng tự vô hiệu hoá", chỉ ở dạng Rust chứ
không phải grep. Đã bỏ, thay bằng `cache_ton_trong_dung_gia_tri_hang_chan_tren` — nhồi gấp
đôi chặn trên rồi đọc sức chứa **thật**, nên một cài đặt truyền nhầm hằng khác vào
`LruCache::new` vẫn bị bắt.

---

## Cổng verification

| Cổng | Kết quả |
|---|---|
| `cargo test` | ✅ **216 passed**, 0 failed, 1 ignored (mốc 163) |
| `cargo clippy --all-targets` | ✅ 0 warning, 0 error |
| `npm run typecheck` | ✅ sạch |
| `npm test` | ✅ **233 passed**, 24 tệp (mốc 226) |
| `npm run tauri:build` | ✅ dựng được, MSI xong (`git-plum_0.1.0_x64_en-US.msi`) |
| `make-diff-fixtures.sh` chạy hai lần | ✅ cả hai lần thành công, in bảng 26 SHA |

Không hồi quy: 163 → 216 Rust (+53), 226 → 233 frontend (+7).

**Cổng grep KHÔNG dùng** (plan nêu rõ, đã tôn trọng):
- `grep -c 'get_file_diff' lib.rs` → thay bằng test **phân tích khối** `generate_handler!`:
  cắt phần trong ngoặc, bỏ chú thích, tách theo dấu phẩy, so khớp từng **mục**.
- `grep -c 'MAX_CACHED_DIFFS'` → thay bằng test đọc **giá trị** hằng, cộng test đọc **sức
  chứa thật**.
- Cổng chặn TTL trong `DiffCache` lọc chú thích trước khi tìm, và có một khẳng định **tiền
  đề** (`ma.contains("pub fn invalidate_repo")`) để phép lọc ăn mất mã không làm cổng luôn xanh.

Một ghi chú đo đạc: `rtk` cũng làm hỏng `grep -c 'arg("--")'` trong lúc chạy mutation (trả
0 trong khi mã có hai chỗ) vì nó phân giải lại pattern. Khi cần đếm chính xác, đọc bằng
`sed -n` hoặc để test Rust làm việc đó.

---

## Repo mẫu `target/fixtures/diff-cases` — 26 commit, 15 hình dạng

Script in bảng `<nhãn> <sha> <ghi chú>` và ghi ra `shas.txt`; test tra theo **nhãn** chứ
không theo thứ tự, nên chèn thêm commit ở giữa về sau không làm test trỏ nhầm.

| Nhãn | Hình dạng |
|---|---|
| `root` | commit gốc (ca `EMPTY_TREE`) |
| `text_simple` | sửa thường |
| `binary_add` / `binary_mod` | tệp nhị phân có byte `\x00` |
| `large_add` / `large_mod` | **5.999.916 byte** (> 5 MB), sửa đúng một dòng |
| `lfs` | con trỏ LFS đúng spec, **không** `.gitattributes` (có chủ ý) |
| `crlf_add` / `crlf_mod` | nội dung `\r\n` |
| `noeol_add` / `noeol_mod` | không dòng trống cuối |
| `rename_add` / `rename_mod` | vừa đổi tên vừa sửa, `R077` |
| `nonutf8_add` / `nonutf8_mod` | tên tệp chứa byte `0xFF` |
| `deleted` | tệp bị xoá |
| `emptyline_*` | 🔵 **wave 3** — dòng trống trước dòng sửa |
| `dup_*` | 🔵 **wave 3** — hai dòng nội dung trùng nhau trong một hunk |
| `oneside_*` | 🔵 **wave 3** — dòng chỉ có ở một phía |
| `mixed_*` | 🔴 **wave 3** — dòng rỗng **cộng** dòng một-phía |
| `modeonly_*` | chỉ đổi mode (ca `Unchanged`, phụ thuộc nền tảng) |

`large.txt` sinh bằng `yes | head` dưới `target/` (đã trong `.gitignore`) — không commit
tệp 6 MB vào repo git-plum.

---

## Ghi chú cho wave 3 (03-03, diff mức từ)

1. **Ba fixture đã sẵn sàng và đã được kiểm hình dạng**, không chỉ kiểm tồn tại. Test
   `fixture_cho_wave_3_doc_duoc_va_dung_hinh_dang` khẳng định qua chính `get_file_diff`:
   `empty-line.txt` cho một dòng ngữ cảnh **rỗng**; `one-side.txt` có **cả** dòng thêm lẫn
   dòng xoá; `mixed.txt` có **cả ba** (ngữ cảnh rỗng + removed + added); `dup-lines.txt`
   còn dòng `TRUNG` nguyên vẹn trong hunk.

2. **`mixed.txt` là fixture của mutation #6 của 03-03.** Đầu ra `--word-diff=porcelain`
   thật của nó đã dán vào chú thích script (mục 14) — đã kiểm khớp từng byte, không cần đo lại.
   `one-side.txt` **một mình không đủ** cho mutation đó; ghi cảnh báo ngay trong script.

3. **Mọi lệnh diff đã có `--no-ext-diff`** từ bản sửa 03-01, chèn tập trung trong
   `GitCommand::run()`. 03-03 dùng `--word-diff=porcelain` (lệnh con `diff`) nên tự động
   có. Đừng thêm lần nữa.

4. **`parse_patch` đã có sẵn và `pub`.** Nếu 03-03 cần đọc `--word-diff=porcelain` thì đó
   là một **định dạng khác** (mỗi từ một dòng, `~` đánh dấu hết dòng nguồn) — cần bộ phân
   tích riêng, không tái dùng `parse_patch`.

5. **Bài học cổng, dùng ngay được ở 03-03:** cổng đọc mã nguồn phải neo vào **quyết định**
   (vị trí `return`, vị trí `.arg`) chứ không vào phép **đo**, và phải kiểm trong phạm vi
   **từng lệnh** chứ không trên cả thân hàm. Hai mutation của plan này xanh lần đầu vì đúng
   hai lỗi đó.

6. **`spike_blob_pair` và `SpikeHarness` vẫn còn** — plan 03-02 không có nhiệm vụ xoá chúng
   và checkpoint #3 chưa chạy. Chúng phải còn để đo được.

---

## Deviations from Plan

### 1. [Rule 1 - Bug] `--name-status` mang pathspec làm mất phát hiện đổi tên

Xem mục "Phát hiện đo được" #1. Assertion cứng của plan sai; đã sửa assertion, ghi lý do
và số đo vào doc comment của `trang_thai_va_ten_cu`, và thêm hai test ghim.
**Commit:** `a52a018`

### 2. [Rule 2 - Correctness] `serde(rename_all)` không lan xuống trường của biến thể

Xem mục "Phát hiện đo được" #2. Không thuộc plan vì plan giả định một dòng ở cấp enum là
đủ. **Commit:** `5d51f65`

### 3. [Rule 1 - Bug] Hai cổng mutation vô dụng, một assertion tính lúc biên dịch

Xem mục "Hai cổng vô dụng phải sửa". **Commit:** `a52a018` (hai cổng), `f5727d3` (assertion).

### 4. Thêm test ngoài plan

Plan đòi "ít nhất 12/7/11 test". Thực tế: 14 test parser + 5 test hợp đồng dữ liệu,
9 test cache + 1 test state, 13 test đơn vị command + 14 test tích hợp, 7 test frontend.
Phần vượt chủ yếu là các cổng phải thêm sau khi mutation cho 0 test đỏ.

---

## Known Stubs

Không có. Cả bốn dạng `DiffKind` mà DIFF-06 đòi đều được sinh từ dữ liệu thật và có test
tích hợp chạy git thật; dạng thứ năm (`Unchanged`) có đường mã và test đơn vị nhưng fixture
của nó (`modeonly_chmod`) **phụ thuộc nền tảng** — trên Windows `core.filemode` thường là
`false` nên commit đó có thể rỗng. Test không dán cứng giả định về nó; đây là hạn chế của
fixture, không phải stub trong mã.

Một điểm **chưa kiểm chứng**, ghi rõ để không ai nhầm: plan này không sửa tệp `.tsx` nào
nên **không có gì để nhìn bằng mắt**. Thời gian mở diff trên tệp lớn thật thuộc checkpoint
#3 (03-01) và **chưa có số**; plan này không đo lại và không chép số từ đâu.

---

## Threat Flags

Không có bề mặt an ninh mới ngoài `<threat_model>` của plan. Mười threat từ T-03-07 đến
T-03-16 đều có đường mã và ít nhất một test:

| Threat | Cổng |
|---|---|
| T-03-07 (`path` thành revision) | `--` trước pathspec ở **từng** lệnh, kiểm theo phạm vi lệnh |
| T-03-08 (`commit_id` thành cờ) | đi qua `rev-parse` trước |
| T-03-09 (`repo_id` bịa) | `repo_cua()` → `get_repo()` |
| T-03-10 (nạp tệp lớn vào RAM) | cổng `cat-file --batch-check` **trước** mọi lần đọc, mutation #6 |
| T-03-11 (patch hàng triệu hunk) | `MAX_HUNKS`, `MAX_DIFF_LINES`, cờ `truncated` |
| T-03-12 (cache phình) | `MAX_CACHED_DIFFS`, `invalidate_repo` khi `close_repo` |
| T-03-13 (rò nội dung nhị phân) | `DiffKind::Binary` không có trường nội dung **ở cả hai phía**, cộng test độ dài JSON < 1 KB |
| T-03-14 (đường dẫn vào log) | `tracing::warn!` ghi `repo.id` và **số**, không ghi path |
| T-03-15 (stderr thành dòng diff) | `parse_patch` chỉ nhận `stdout` |
| T-03-16 (diff sai không truy được) | `skipped_lines > 0` → `tracing::warn!` |

Ghi nhận một điểm **giảm** rủi ro ngoài dự kiến: cổng LFS chỉ đọc blob khi nó `< 1 KB`, nên
đường "đọc nội dung tệp" duy nhất trong command này bị chặn ở 1 KB chứ không ở 5 MB.

---

## Self-Check: PASSED

Tệp đã kiểm tồn tại, số dòng đọc bằng `wc -l` chứ không ước lượng:

| Tệp | Dòng | Mốc plan |
|---|---|---|
| `src-tauri/src/domain/diff.rs` | 271 | — (có `pub enum DiffKind`) |
| `src-tauri/src/git/parsers/patch.rs` | 625 | ≥150 ✅ |
| `src-tauri/src/cache/diff_cache.rs` | 446 | — (có `MAX_CACHED_DIFFS`) |
| `src-tauri/src/commands/diff.rs` | 939 | — (có `get_file_diff`) |
| `src-tauri/tests/diff_commands.rs` | 586 | — |
| `scripts/fixtures/make-diff-fixtures.sh` | 392 | ≥60 ✅ |
| `src/lib/ipc.diff.test.ts` | 154 | — |

Commit đã kiểm có trong `git log`, chuỗi RED→GREEN đầy đủ:

```
a52a018 feat(03-02): add get_file_diff command with DIFF-06 gates before git diff
f5727d3 feat(03-02): implement LRU diff cache and diff fixture repo
dc09df5 test(03-02): add failing tests for the LRU diff cache          ← RED
5d51f65 feat(03-02): implement unified diff parser and diff data contract
7b41a9b test(03-02): add failing tests for diff contract and unified diff parser  ← RED
```
