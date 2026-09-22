---
phase: 03-diff-viewer
plan: 05
subsystem: diff-viewer
tags: [rust, frontend, diff-05, file-history, exit-gate]
requires:
  - "src-tauri/src/git/parsers/log.rs LOG_FORMAT (%x1f là tiền lệ đúng cho git log)"
  - "src-tauri/src/commands/diff.rs PATHSPEC_SAU_DAU_GACH + repo_cua + DEFAULT_TIMEOUT"
  - "src-tauri/src/commands/history.rs parse_name_status (khuôn R/C hai đường dẫn)"
  - "src/lib/commands.ts registerCommands/runCommand/unregisterCommand (PLAT-04)"
  - "src/stores/diffStore.ts (03-04), src/stores/selectionStore.ts"
  - "target/fixtures/diff-cases repo mẫu có rename R077"
provides:
  - "src-tauri/src/git/parsers/file_history.rs — parse_file_history + FILE_HISTORY_FORMAT"
  - "src-tauri/src/commands/diff.rs — lay_lich_su_tep + get_file_history + MAX_FILE_HISTORY"
  - "src-tauri/src/domain/diff.rs — FileVersion + FileHistory"
  - "src/lib/ipc.ts — FileVersion/FileHistory + ipc.getFileHistory"
  - "src/components/diff/FileHistory.tsx — danh sách phiên bản bấm được"
  - "src/stores/diffStore.ts — historyOpen + historyCommitOverride"
  - "docs/08-phase3-dogfood.md — khung ghi chép cổng thoát"
affects:
  - "Phase 4 (thư mục làm việc) — cache lịch sử tệp chỉ khả thi khi có watcher"
  - "Phase 4 — lệnh riêng cho merge commit, nếu cổng thoát cho thấy nó đáng"
tech-stack:
  added: []
  patterns:
    - "git log -z phân tách bằng `\\0\\n`, KHÔNG `\\0` trơn — phải tách theo CẢ `\\x1f` VÀ `\\0`"
    - "`--follow` + pathspec: git ĐÃ lọc --name-status, nên phép lọc theo path là mã chết"
    - "Cổng đọc nguồn phải bỏ chú thích — kể cả chú thích do chính mutation sinh ra"
    - "Đo trước khi thêm cờ: `--diff-merges=first-parent` phá `--max-count` (104 → 5715)"
decisions:
  - "KHÔNG cài phép lọc theo path mà plan đòi — đo được là git đã lọc sẵn"
  - "Bấm phiên bản dùng historyCommitOverride, KHÔNG ghi vào selectionStore"
  - "Merge commit không vào lịch sử tệp: NHẬN giới hạn, ghim bằng test, không dùng cờ tệ hơn"
  - "Danh sách phẳng không ảo hoá — chặn 200 ở tầng git nên DOM ≤ 200 hàng"
metrics:
  duration: "~75 phút"
  completed: "2026-09-22"
  tasks_completed: "2/3 (Task 3 là EXIT GATE, ĐANG CHỜ chủ dự án)"
  tests_added: "50 Rust (25 mới) + 38 frontend"
  tests_total: "284 Rust + 2 doctest + 1 ignored; 435 frontend"
---

# Phase 3 Plan 05: Lịch sử một tệp + cổng thoát Summary

DIFF-05 xong: `git log --follow` cho một tệp, lần theo được đổi tên và **nói ra** chỗ
đổi tên, chặn 200 phiên bản, không cache. Cộng **hai chỗ plan sai** (một đo được ngược
hẳn), **một cổng xanh sai** phải sửa, và **một giới hạn đã biết** được nhận thay vì
lấp bằng một cờ tệ hơn.

**⏸️ Task 3 (EXIT GATE dogfood) CHƯA CHẠY** — chờ chủ dự án. Bản release đã dựng:
**2026-09-22 11:38:59 +0700**, commit `0845b24`.

---

## Bằng chứng byte thật — dấu phân tách `\0\n`

Đo lại bằng `od -c` trên máy này (git 2.54.0.windows.1, 2026-09-22), **không** chép từ
plan. Lệnh: `git log --follow --max-count=3 --format='%H%x1f%an%x1f%at%x1f%s'
--name-status -z -- docs/09-phase3-diff-decision.md`:

```
0000140       s   k   i   p   p   e   d   ,       p   a   t   h       A
0000160       c   h   o   s   e   n       w   i   t   h   o   u   t
0000200       e   v   i   d   e   n   c   e  \0  \n   M  \0   d   o   c   s
                                              ^^^^^^^^
                                              NUL RỒI NEWLINE
0000220   /   0   9   -   p   h   a   s   e   3   -   d   i   f   f   -
```

Và `037` (byte 0x1f) thấy rõ ở đầu bản ghi — xác nhận `%x1f` **dùng được** với
`git log`, khác `for-each-ref` vốn cần `%1f` (02-04-SUMMARY).

Plan gọi đây là mutation then chốt và **đúng**: một bộ phân tích chỉ tách `\0` đọc
`status` thành `"\nM"`, phép so `status == "M"` trượt **không một tiếng nào**, và danh
sách phiên bản về rỗng.

### 🔴 Và một bẫy thứ hai mà plan không nêu: phải tách theo CẢ HAI byte

Bản cài đầu của tôi tách **chỉ** theo `\0` rồi cắt `\n` — tức làm đúng điều plan mô
tả. Nó **sai**, và sai theo một cách mà plan không lường: bốn trường của
`--format=` ngăn nhau bằng `\x1f`, không bằng `\0`. Nên cả header thành **một** trường
(`sha\x1ftác giả\x1f…`), không khớp `la_sha`, và **mọi** bản ghi bị tính là méo.

Con số đo được: `skipped_records = 9` trên một buffer **ba commit hoàn toàn hợp lệ**.
13 test đơn vị đỏ cùng lúc, tất cả với `left: 0`.

Bản đúng tách theo `b == 0 || b == UNIT_SEP` rồi mới cắt `\n`/`\r`. Hai bước, hai lý
do khác nhau, và ghi rõ cả hai trong doc comment.

---

## 🔴 Chỗ plan SAI — và cả hai đều đo được

### 1. `--follow` KHÔNG in mọi tệp của commit — git đã lọc sẵn

`<behavior>` Task 1 viết: *"**Commit đụng nhiều tệp**: `--follow` vẫn in mọi tệp của
commit đó. Chỉ giữ bản ghi khớp path đang theo, bỏ phần còn lại. Không lọc thì lịch sử
một tệp hiện cả tệp khác."* Và `<verification>` đặt mutation #4 là *"bỏ lọc theo path
đang theo → ≥1 đỏ"*.

**Đo thật thì tiền đề đó sai.** Repo dựng riêng: `target.txt` và `other.txt`, một
commit sửa **cả hai**:

```
$ git log --follow --format='%H%x1f%at%x1f%s' --name-status -z -- target.txt | od -c
  o   n   e       c   o   m   m   i   t       t   o   u   c   h   e   s
  B   O   T   H       f   i   l   e   s  \0  \n   M  \0   t   a   r   g   e   t   .   t   x   t  \0
                                                         ^^^^^^^^^^^^^^^^^^^^^^^^ chỉ target.txt
```

```
$ git log --follow ... -- target.txt | tr '\0' '\n' | grep -c "other.txt"
0
```

`git log` với một pathspec **đã lọc** `--name-status` xuống đúng path đó. Phép lọc mà
plan đòi sẽ là **mã không bao giờ chạy**.

**Và nó còn tệ hơn "vô dụng".** Để lọc "theo path đang theo", mã phải biết path đó là
gì — nhưng path **đổi** ở mỗi lần đổi tên. Một phép lọc `v.path == path_được_hỏi` sẽ
loại bỏ đúng các phiên bản **trước** lần đổi tên, tức xoá sạch thứ mà `--follow` vừa
được thêm vào để mua. Cài đúng theo plan ở đây sẽ tạo ra chính lỗi mà plan gọi là "hỏng
im lặng, tệ hơn chậm".

**Đã làm thay:** giữ **bản ghi tệp đầu tiên** của mỗi commit, cộng một test tích hợp
ghim rằng git lọc sẵn. Test đó là cổng có giá trị thật: nếu một phiên bản git về sau
đổi hành vi, nó đỏ và ta biết phải thêm phép lọc — chứ không phát hiện qua người dùng.

Mutation tương ứng (#4, đã chuyển thành "bỏ cổng giữ-bản-ghi-đầu") vẫn cho **1 đỏ**.

### 2. Merge commit không chỉ thiếu khối tệp — nó KHÔNG XUẤT HIỆN

`<behavior>` viết: *"`git log --follow` mặc định **không** in `--name-status` cho merge.
Commit đó xuất hiện với danh sách tệp rỗng → **bỏ nó**."*

Nửa đầu đúng, nửa sau sai. Đo trên repo dựng riêng có **xung đột thật đã giải quyết**:

```
$ git log --follow --format='%H %s' -- f.txt
970f2772... main
51343e35... side
a7895ffe... init
```

Merge `59165ca` — commit đã **thật sự giải quyết xung đột trong chính `f.txt`** —
không có mặt. Không phải "xuất hiện với danh sách rỗng"; nó bị loại khỏi cả danh sách
commit.

Nhánh "bỏ commit không có bản ghi tệp" **vẫn được cài** vì nó là cổng chặn một hàng
trống nếu ai đó thêm `-m`/`--diff-merges` về sau, và có test đơn vị riêng. Nhưng nó
gần như không bao giờ chạy trên đầu ra git thật, và doc comment nói đúng điều đó.

Xem mục "Giới hạn đã biết" dưới đây cho vì sao tôi **không** sửa nó bằng một cờ.

---

## 🔴 Một cổng XANH SAI — và nó khớp chú thích do chính mutation sinh ra

Mutation #5 (xoá `.arg("--")` khỏi `lay_lich_su_tep`) cho **0 test đỏ** ở lần chạy đầu
trong cổng mới của tôi.

**Nguyên nhân.** Cổng tìm `.arg("--")` trên thân hàm **thô**. Dòng bị xoá được thay
bằng chú thích:

```rust
// MUTATION 5: .arg("--") removed
```

— và cổng khớp **chuỗi trong chú thích đó**. Đếm được:

```
$ awk '/pub async fn lay_lich_su_tep/,/^}/' diff.rs | grep -n 'arg("--"'
22:                // MUTATION 5: .arg("--") removed
```

Đây là lần thứ **năm** dự án gặp lớp lỗi này (bốn lần trước ở `03-04-SUMMARY.md`), và
quy tắc đã ghi ở đó áp đúng:

> **Mọi cổng đọc nguồn phải bỏ chú thích trước khi tìm.**

**Điều làm ca này khác bốn ca trước, và đáng ghi riêng:** chú thích gây nhiễu **không
có sẵn trong mã** — nó do chính **phép kiểm mutation** sinh ra. Nên một cổng đọc nguồn
thô có thể sống qua mọi lần review và chỉ hỏng **đúng lúc người ta đang kiểm nó**. Một
cổng tự vô hiệu hoá theo cách này là vô hình với cả review lẫn mutation testing nếu
mutation được viết bằng chú thích — và viết mutation bằng chú thích là cách tự nhiên
nhất.

**Bản sửa:** cổng tìm trên `bo_chu_thich(&than)`, cộng một khẳng định **tiền đề**
(`sach.contains("GitCommand::new")`) để phép lọc quá tay không làm cổng luôn xanh.

Sau khi sửa: **1 đỏ** trong cổng của tôi.

**Một điều làm giảm nhẹ, đáng ghi:** cổng **đã có** của dự án
(`moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path`, viết ở 03-02) **bắt được**
mutation này ngay từ lần chạy đầu — nó cắt thân hàm theo từng `GitCommand::new` và
kiểm trong phạm vi từng lệnh. Nên hệ thống cổng của dự án đúng; chỉ cổng **mới** của
tôi sai. Cổng cũ cũng buộc tôi cập nhật hai con số (4 → 5 mốc pathspec, 3 → 4 lệnh
mang pathspec) — nó phát hiện lệnh mới của tôi chưa được kiểm, đúng như thông điệp lỗi
của nó nói.

---

## ⚠️ Giới hạn đã biết: merge commit — và vì sao cờ trông-như-bản-sửa thì TỆ HƠN

`--diff-merges=first-parent` khôi phục được merge commit, sạch, đúng một bản ghi:

```
$ git log --follow --diff-merges=first-parent --format='%H %s' --name-status -z -- f.txt
59165ca... merge with real resolution|M|f.txt|970f277... main|M|f.txt|...
```

Trông như bản sửa. **Đo trên repo thật thì nó phá hai chặn trên.** `dau-tri-toan-hoc`
(1140 commit, **39 merge**), cùng `--max-count=200` ở cả hai cột:

| tệp | không cờ | `--diff-merges=first-parent` |
|---|---|---|
| `.planning/STATE.md` | **104** bản ghi | **5715** |
| `.planning/ROADMAP.md` | **118** bản ghi | **4655** |
| `src/pages/match/MatchInGamePage.tsx` | **63** bản ghi | **3176** |

55×. Nguyên nhân: cờ đó làm git in **nội dung bản vá** ra cùng luồng, nên `--max-count`
không còn chặn số dòng và bộ phân tích nhận hàng nghìn dòng không phải bản ghi tệp
(kiểm được: `--format='%H'` một mình mà vẫn thấy `new file mode 100644` trong đầu ra).

Hệ quả: nó phá **cả hai** chặn của threat model — `--max-count` (T-03-33, DoS) và 200
hàng DOM (T-03-34). Tức nó đổi **một khoảng thiếu nhìn thấy được** thành **hai lỗi im
lặng**.

**Quyết định: nhận giới hạn, ghim bằng test, ghi vào tài liệu người dùng đọc.** Có test
(`merge_commit_khong_vao_danh_sach_phien_ban_gioi_han_da_biet`) dựng repo có xung đột
thật, khẳng định ba commit thường có mặt và merge thì không, **cộng một khẳng định tiền
đề** rằng repo mẫu thật sự có đúng một merge — thiếu nó thì test xanh vì repo dựng
thất bại chứ không vì hành vi đúng.

Test đó **đã kiểm là đỏ được**: thêm `--diff-merges=first-parent` →

```
assertion `left == right` failed: ba commit thường phải có mặt, đọc được 4:
["merge with real resolution", "main edits", "side edits", "init"]
  left: 4
 right: 3
```

Và thông điệp lỗi của nó nói thẳng rằng **đỏ không tự động nghĩa là tốt hơn** — đọc
phép đo 104 → 5715 trước khi coi đó là một cải thiện. Một test ghim giới hạn mà không
nói điều đó sẽ bị người sau "sửa" thành lỗi.

Cách đúng nếu cổng thoát cho thấy nó đáng: một lệnh **riêng** cho merge (`git show -m`
trên đúng một commit), không phải một cờ thêm vào lệnh lịch sử tệp.

---

## Kiểm mutation — 9 đột biến, tất cả đỏ; một cổng phải sửa trước

Cột cuối là output đỏ thật, trích nguyên.

| # | Đột biến | Kết quả |
|---|---|---|
| 1 | Bỏ cắt `\n` (phân tích như thể phân tách chỉ là `\0`) | ✅ **7 đỏ** (5 đơn vị + 2 tích hợp) — **then chốt** |
| 2 | Bỏ `--follow` | ✅ **2 đỏ** (test đổi tên trên git thật + cổng đọc nguồn) |
| 3 | Đọc bản ghi `R` như một đường dẫn | ✅ **3 đỏ**, và đỏ ở bản ghi **sau** nó |
| 4 | Bỏ cổng giữ-bản-ghi-đầu *(thay cho "bỏ lọc theo path" — plan sai, xem trên)* | ✅ **1 đỏ** |
| 5 | Xoá `.arg("--")` | ⚠️ **0 đỏ trong cổng mới** → sửa cổng → ✅ **1 đỏ** (cổng cũ của dự án: đỏ ngay) |
| 6 | Bấm phiên bản ghi vào `selectionStore` | ✅ **3 đỏ** |
| 7 | Bỏ cổng chống đua | ✅ **1 đỏ** |
| 8 | Bỏ `unregisterCommand` ở cleanup | ✅ **1 đỏ** (mount hai lần) |
| 9 | Thêm `--diff-merges=first-parent` *(ngoài plan — kiểm test giới hạn có đỏ được)* | ✅ **1 đỏ** |
| + | CSS: bỏ `min-height`, thêm `minmax(0,`, bỏ `min-width` | ✅ **đúng test đó đỏ, cả ba** |

### Output đỏ thật (trích nguyên)

**#1 — mutation then chốt, đơn vị:**
```
status phải là "M"; "\nM" nghĩa là dấu phân tách \0\n chưa được cắt
  left: "\nM"
 right: "M"

status mang \n: "\nM"
```

**#1 — và trên GIT THẬT, không phải buffer tự dựng:**
```
🔴 status méo nghĩa là dấu phân tách \0\n chưa được cắt: "\nM"
🔴 không có --follow thì lịch sử ĐỨT ở chỗ đổi tên và chỉ còn 1 phiên bản.
```
Bản ghi thứ hai đáng chú ý: bỏ bước cắt `\n` làm **cả** test `--follow` đỏ, vì bản ghi
`R077` cũng không đọc được. Hai cổng độc lập cùng quan sát một lỗi.

**#2 — trên git thật, repo mẫu có `R077`:**
```
🔴 không có --follow thì lịch sử ĐỨT ở chỗ đổi tên và chỉ còn 1 phiên bản.
Phải có ≥2: bản ghi R và bản ghi A của tên cũ phía trước nó
```

**#3 — đỏ ở bản ghi SAU bản ghi `R`, đúng như plan đòi:**
```
assertion `left == right` failed: phải đọc đúng ba phiên bản
  left: 1
 right: 3

assertion `left == right` failed
  left: None
 right: Some("goc.txt")
```

**#4 — tệp khác rò vào lịch sử một tệp:**
```
assertion `left == right` failed
  left: "khac.txt"
 right: "a.txt"
```

**#5 sau khi sửa cổng:**
```
🔴 thiếu .arg("--") — một path trùng tên nhánh sẽ bị git hiểu là revision (T-03-32)
```

**#6 — ba đỏ, gồm đúng cổng đã thiết kế:**
```
FAIL > 🔴 đặt historyCommitOverride, KHÔNG đổi selectedCommitId trên đồ thị
AssertionError: expected null to be 'dddddddddddddddddddddddddddddddddddddddd'
```

**#7 — cổng chống đua quan sát được ngay lần đầu** (khác 03-04, nơi nó cần hai lần sửa):
```
expect(panel.textContent).toContain('của tệp MỚI')
```

**#8:**
```
expect(getCommand('diff.toggleFileHistory')).toBeUndefined()
```

---

## Cổng verification — cả 9 cổng của bảng `<verification>`

| # | Cổng | Kết quả |
|---|---|---|
| 1 | `cargo test file_history` + bỏ cắt `\n` | ✅ **7 đỏ** — then chốt |
| 2 | `cargo test file_history` + bỏ `--follow` | ✅ 2 đỏ |
| 3 | `cargo test file_history` + đọc `R` một đường dẫn | ✅ 3 đỏ, ở bản ghi **sau** |
| 4 | `cargo test file_history` + bỏ lọc theo path | ⚠️ **cổng của plan BẤT KHẢ** (git đã lọc sẵn) — đã thay bằng "bỏ cổng giữ-bản-ghi-đầu", ✅ 1 đỏ |
| 5 | `cargo test file_history` + xoá `.arg("--")` | ⚠️ 0 đỏ → sửa cổng → ✅ 1 đỏ |
| 6 | `npm test FileHistory` + ghi vào `selectionStore` | ✅ 3 đỏ |
| 7 | `npm test FileHistory` + bỏ cổng chống đua | ✅ 1 đỏ |
| 8 | `npm test FileHistory` + bỏ `unregisterCommand` | ✅ 1 đỏ |
| 9 | `npm test app.css` + vi phạm `.file-history` | ✅ đúng test đó đỏ, **cả ba** điều |

**Cổng grep KHÔNG dùng** (plan nêu ba, đã tôn trọng cả ba):
- `grep -c '\-\-follow' file_history.rs` → thay bằng **test tệp đổi tên trên git thật**,
  vốn đỏ khi và chỉ khi `--follow` mất. Doc comment của tệp dẫn `--follow` nhiều lần.
- `grep -c 'max-count'` → thay bằng test đọc **thân hàm đã bỏ chú thích**
  (`than_ham` + `bo_chu_thich`), cộng khẳng định tiền đề về độ dài.
- Mọi `grep -c ... == 0` trên tệp có doc comment tiếng Việt → mọi cổng đọc nguồn trong
  wave này lọc chú thích trước. Ca #5 chứng minh vì sao (xem "Một cổng xanh sai").
- `get_file_history` trong `generate_handler!` kiểm bằng **test phân tích khối**
  `generate_handler![...]` trên nguồn đã bỏ chú thích, cộng khẳng định tiền đề rằng
  khối chứa `get_file_diff` — không `grep -c`.

### Cổng hạ tầng

| Cổng | Kết quả |
|---|---|
| `cargo test --lib --tests` | ✅ **284 passed, 1 ignored** (mốc 259 → +25) |
| `cargo test --doc` | ✅ 2 passed |
| `cargo clippy --all-targets` | ✅ 0 issue mới (1 warning **có trước**, `diff.rs:1470`, đã kiểm bằng `git stash`) |
| `npm run typecheck` | ✅ sạch |
| `npm test` | ✅ **435 passed**, 34 tệp (mốc 397 → +38) |
| `npm run build` | ✅ built in 314ms |
| `node scripts/check-lang-chunks.mjs` | ✅ entry sạch, 5 chunk ngôn ngữ tách riêng |
| `npx tauri build --no-bundle` | ✅ Finished in 1m 59s |

**Không hồi quy.** Rust 259 → 284; frontend 397 → 435.

**Lưu ý về con số mốc.** Prompt nêu "261 test Rust + 1 ignored, 396 test frontend".
Đo thật lúc bắt đầu: Rust **259** `--lib --tests` **cộng 2 doctest** = 261 (khớp, chỉ
là cách đếm); frontend **397**, không 396 — chênh một test đến từ công việc **không
thuộc plan này** đã có trong cây làm việc lúc tôi bắt đầu (xem "Cây làm việc" dưới).

---

## 🔴 Cây làm việc có công việc KHÔNG thuộc plan này

`git status` lúc bắt đầu session báo "clean", nhưng khi tôi chuẩn bị commit thì có bốn
tệp đã sửa mà **tôi không đụng tới**:

```
 M src/App.tsx                            (+22 -1)
 M src/components/RefSidebar.tsx          (+38 -9)
 M src/components/history/CommitList.tsx  (+38 -2)
 M src/styles/app.css                     (hai hunk đầu)
```

Nội dung là việc của **Phase 2** (bấm nhánh/tag ở thanh bên thì cuộn đồ thị tới commit
đó — `handleSelectRef`, `scrollToCommit`). Không liên quan DIFF-05.

**Xử lý:** **không** commit chúng, và **không** `git stash` (một lệnh có thể làm mất
việc của người khác — tôi đã thử và hệ thống quyền từ chối đúng lúc). Thay vào đó stage
từng tệp một, và với `app.css` — vốn có **cả** hunk của họ **lẫn** hunk của tôi — tôi
tách diff và `git apply --cached` **chỉ** hunk của mình:

```
$ git diff --cached --stat -- src/styles/app.css
 src/styles/app.css | 118 +++++++++++++++++++  ← đúng phần tôi thêm
```

Bốn tệp đó **vẫn nguyên trong cây làm việc**, chưa commit. Chủ dự án cần quyết định:
đó là việc đang làm dở, hay cần một commit riêng.

Đây cũng là lý do mốc frontend là 397 chứ không 396: hai test của `App.test.tsx` thuộc
công việc đó đã có trong cây.

---

## Quyết định thiết kế

### Bấm phiên bản = `historyCommitOverride`, KHÔNG ghi `selectionStore`

Người dùng đang đứng ở một chỗ trong lịch sử và bấm qua từng phiên bản để xem tệp đổi
thế nào. Nếu mỗi lần bấm cũng đổi commit đang chọn trên **đồ thị**, họ **mất chỗ** —
hàng đang sáng nhảy đi, và khi đóng panel thì không còn đường về.

Hai việc độc lập, và giữ chúng độc lập là cả điểm. `DiffViewer` đọc
`historyCommitOverride ?? commitTrenDoThi`; đóng panel xoá ghi đè nên viewer về đúng
commit trên đồ thị. Có băng thông báo nói rõ "Commit đang chọn trên đồ thị **không
đổi**" — nếu không, người dùng thấy diff đổi mà không hiểu mình đang xem commit nào.

### Hai lớp phòng thủ cho ca đổi tệp

`selectFile`/`clearFile`/`commitChanged` **đóng** panel và xoá ghi đè (lớp 1) — danh
sách phiên bản đang hiện thuộc tệp cũ. Cổng chống đua `requestIdRef` trong
`FileHistory` là lớp 2, cho ca người dùng mở lại ngay. Test chống đua đi qua đúng
đường đó (đổi tệp **rồi** `openHistory()`), nên nó kiểm lớp 2 chứ không chỉ lớp 1.

### Danh sách phẳng, không ảo hoá

Chặn 200 ở tầng git nghĩa là DOM không bao giờ vượt 200 hàng (T-03-34). `useVirtualizer`
thêm một lớp đo chiều cao, một `ref` cuộn, và lớp lỗi "hàng lệch so với vùng cuộn" mà
Phase 2 trả hai vòng checkpoint để sửa. `CommitList` cần nó vì 100 nghìn hàng; đây thì
không. Ghi lý do trong comment để người sau không thêm theo quán tính.

### `DiffToolbar` nhận `repoId` **optional**

Đúng tiền lệ `FileList` (03-04 deviation #4): 21 test `DiffToolbar` hiện có dựng
component không có prop này, và một prop bắt buộc sẽ phá chúng vì lý do không liên
quan. Không truyền → nút mờ, và `runCommand` vẫn không làm gì nhờ `enabled` của lệnh.

Nút **không** tự kiểm "có tệp đang chọn không" trong `onClick` — `enabled` của `Command`
là chỗ đúng, vì bảng lệnh gõ nhanh (v2) đọc trường đó. `disabled` trên nút chỉ để người
dùng **thấy**, không phải để thực thi.

---

## Deviations from Plan

### 1. [Rule 1 - Bug] Phép lọc theo path mà plan đòi là mã chết, và cài nó sẽ tạo lỗi

Xem "Chỗ plan SAI" #1. Đo được là git đã lọc sẵn theo pathspec; và một phép lọc theo
"path đang theo" sẽ xoá đúng các phiên bản trước lần đổi tên. Đã thay bằng "giữ bản ghi
đầu tiên" cộng một test tích hợp ghim hành vi lọc của git. **Commit:** `506619e`

### 2. [Rule 1 - Bug] Cổng `--` của tôi khớp chú thích do mutation sinh ra

Xem "Một cổng XANH SAI". Sửa ở **cổng**, không ở mã sản phẩm — mã sản phẩm đúng, và
cổng đã có của dự án bắt được mutation ngay. **Commit:** `fc4c851`

### 3. [Rule 2 - Correctness] Ghim giới hạn merge commit bằng test, ngoài plan

Plan không nêu ca merge nào ngoài "commit xuất hiện với danh sách rỗng", vốn đo được là
sai. Đã thêm một test dựng repo có xung đột thật, cộng phép đo 104 → 5715 chứng minh cờ
trông-như-bản-sửa thì tệ hơn. Không thêm cờ. **Commit:** `fc4c851`

### 4. [Rule 3 - Blocking] Cập nhật hai con số của cổng pathspec đã có

`moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path` (03-02) đỏ khi tôi thêm lệnh
git thứ tư mang pathspec — **đúng như thông điệp lỗi của nó nói**: *"số mốc khác 4
nghĩa là một lệnh mất mốc hoặc có lệnh mới chưa được kiểm"*. Cập nhật 4 → 5 mốc và
3 → 4 lệnh, kèm tên lệnh mới trong thông điệp. Cổng vẫn kiểm `--` cho **từng** lệnh,
gồm lệnh mới. **Commit:** `506619e`

### 5. Thêm test ngoài plan

Plan đòi "≥13 test Rust (≥4 tích hợp)" và "≥11 test frontend". Thực tế:

| Tệp | Số test |
|---|---|
| `git/parsers/file_history.rs` (đơn vị) | 15 |
| `tests/file_history_commands.rs` (tích hợp, **10 chạy git thật**) | 10 |
| `components/diff/FileHistory.test.tsx` | 25 |
| `lib/ipc.fileHistory.test.ts` | 6 |
| `styles/app.css.test.ts` (thêm) | 7 |

Rust +25, frontend +38. Phần vượt chủ yếu là ca bản ghi méo (`sha_khong_40_hex`,
`header_cut_o_cuoi_buffer`), ca `C` sao chép, và ca "đúng `max` mà hết buffer thì
**không** truncated" — vốn khác ca bị cắt và dễ bị gộp thành một.

---

## Known Stubs

Không có stub. Hai điều **có chủ ý** giới hạn, ghi rõ để không ai nhầm là stub:

1. **Merge commit không vào lịch sử tệp.** Giới hạn đã đo, có test ghim, có lý do tường
   minh vì sao cờ khắc phục thì tệ hơn. Xem mục riêng ở trên.
2. **Không cache lịch sử tệp.** Phụ thuộc HEAD; cache cần vô hiệu hoá mà watcher chỉ
   tới ở Phase 4. Ghi trong doc comment để người sau không "tối ưu" nó vào một lỗi —
   hệ quả cụ thể của cache sai: người dùng commit rồi mở lại lịch sử tệp và **không
   thấy commit của chính mình**.

---

## Threat Flags

Không có bề mặt an ninh mới ngoài `<threat_model>` của plan. Bảy threat T-03-32..38 đều
có đường mã và ít nhất một test:

| Threat | Cổng |
|---|---|
| T-03-32 (`path` bị đọc thành revision) | `.arg("--")` trước `PATHSPEC_SAU_DAU_GACH`; mutation #5 (sau khi sửa cổng) + cổng đã có của 03-02 |
| T-03-33 (`--follow` không chặn) | `--max-count=200` qua `MAX_FILE_HISTORY` + `DEFAULT_TIMEOUT` 30 s; test đọc thân hàm đã bỏ chú thích. Và phép đo 104 → 5715 là bằng chứng chặn này **thật sự** đang chặn |
| T-03-34 (50 nghìn hàng DOM) | chặn 200 ở tầng git; danh sách phẳng không ảo hoá là **hệ quả** của chặn đó |
| T-03-35 (`repo_id` bịa) | `repo_cua()` → `state.get_repo()`, chỉ repo đã mở |
| T-03-36 (path vào log) | `tracing::warn!` ghi `repo.id` và **số**, không ghi path |
| T-03-37 (danh sách tệp trước hiện cho tệp sau) | `requestIdRef` + `selectFile` đóng panel (hai lớp); mutation #7 → 1 đỏ |
| T-03-38 (thiếu commit mà không ai biết) | `skipped_records` + `warn!`; `truncated` hiện **trên giao diện** ("200 phiên bản gần nhất"), có test |

---

## ⏸️ Những gì vẫn CHƯA KIỂM CHỨNG — ghi thẳng, không lấp bằng suy luận

happy-dom không tính layout CSS. Nên 435 test xanh **không** nói được:

| Chưa kiểm | Vì sao test không kiểm được |
|---|---|
| Bốn cột của một hàng phiên bản có **thẳng hàng** | không có layout |
| Tiêu đề commit dài bị **cắt kèm ellipsis** hay **biến mất** | không có grid/flex track sizing |
| Panel lịch sử có **đẩy diff ra khỏi khung** không | không tính chiều cao |
| Bấm một phiên bản → người dùng **thấy** diff đổi mà hàng trên đồ thị không nhảy | test khẳng định giá trị store, không khẳng định thị giác |
| Danh sách 200 hàng có **cuộn mượt** | không có cuộn thật |

Cổng cấp hai cho bố cục là `app.css.test.ts` (7 test đọc nguồn CSS, **đã kiểm là đỏ
được** ở cả ba điều). Nó là điều kiện **cần**, không **đủ** — đúng như mọi cổng CSS
trong dự án này.

**Và hai điều thừa hưởng, chưa đổi:**
- **Chế độ hai cột vẫn không có test tự động nào chạy qua nó** (`ResizeObserver` không
  có trong happy-dom). Wave 4 tìm ra **năm** lỗi chỉ tồn tại ở chế độ đó, tất cả do
  người dùng mở app thật.
- **Hiệu năng `MergeView` trên tệp lớn thật vẫn chưa có số.** Wave 4 đã thu hẹp bằng đo
  Chromium (200 nghìn dòng = 229ms, dưới ngân sách 1 giây của Core Value), nhưng đó là
  dòng sinh tổng hợp, không phải `yarn.lock` 630 KB thật mà checkpoint #3 đặt ra.

---

## Self-Check: PASSED

Tệp đã kiểm tồn tại, số dòng đọc bằng `wc -l`:

| Tệp | Dòng | Mốc plan |
|---|---|---|
| `src-tauri/src/git/parsers/file_history.rs` | 571 | ≥130 ✅ |
| `src/components/diff/FileHistory.tsx` | 219 | ≥90 ✅ |
| `docs/08-phase3-dogfood.md` | 201 | chứa "Exit gate" ✅ |
| `src-tauri/tests/file_history_commands.rs` | 496 | — |
| `src/components/diff/FileHistory.test.tsx` | 398 | — |

Chuỗi commit RED→GREEN đầy đủ trong `git log`:

```
0845b24 docs(03-05): complete the file history plan, phase 3 not closed
d1b59cc docs(03-05): add the phase 3 exit gate dogfood record
fc4c851 docs(03-05): pin the merge-commit gap and why first-parent is not the fix
807d942 feat(03-05): add the file history panel with rename labels
1fd3e4b test(03-05): add failing tests for the file history panel        ← RED
506619e feat(03-05): implement get_file_history with git log --follow
f8d11fd test(03-05): add failing tests for per-file history              ← RED
```

`key_links` của frontmatter đã kiểm bằng phép khớp mẫu thật:

| from → to | pattern | Kết quả |
|---|---|---|
| `FileHistory.tsx` → `ipc.ts` | `getFileHistory` | ✅ dòng 98: `.getFileHistory(repoId, selectedFile)` — prettier tách `ipc` sang dòng trước, nên chuỗi `ipc.getFileHistory` **không** khớp liền; đã kiểm bằng `grep getFileHistory` chứ không bằng chuỗi có tiền tố |
| `FileHistory.tsx` → `diffStore.ts` | bấm phiên bản đặt commit | ✅ `selectHistoryVersion(v.commitId)` — **không** `runCommand('diff.` như plan dự đoán; xem dưới |

**Một sai lệch nhỏ với `key_links` của plan.** Plan đòi pattern `runCommand\('diff\.`
cho đường `FileHistory → diffStore`. Cài đặt dùng `selectHistoryVersion` gọi thẳng
store cho việc **chọn phiên bản**, và `runCommand('diff.toggleFileHistory')` cho việc
**mở/đóng panel**. Lý do: PLAT-04 nói mọi **thao tác người dùng gọi được** phải có tên
trong sổ đăng ký để bảng lệnh gõ nhanh (v2) thấy được. "Mở/đóng lịch sử tệp" là một
thao tác như vậy — nó có id, có `enabled`, và có test bấm nút + spy. "Chọn phiên bản
thứ 7 trong danh sách" thì **không**: nó mang một tham số động (`commitId`) mà sổ đăng
ký không mô hình hoá được, và đăng ký 200 lệnh cho 200 hàng là vô nghĩa. Cùng khuôn
`FileList` đã dùng: `runCommand('diff.selectFile')` cho thao tác có tên, không cho mỗi
hàng một lệnh.
