---
phase: 02-history-graph
plan: 04
subsystem: refs-parser-cache-history-commands
tags: [HIST-01, HIST-02, HIST-06, HIST-07, HIST-10, HIST-11, cache, tauri-command, ipc, serde]
requires:
  - "02-01 — chín repo mẫu ở target/fixtures/ và repo hiệu năng 100k commit"
  - "02-02 — domain::{Commit, Ref, RefKind}, LOG_FORMAT, LOG_ARGS, parse_log"
  - "02-03 — graph::lanes::assign, MAX_VISIBLE_LANES = 20"
provides:
  - "git::parsers::refs::{parse_refs, REFS_FORMAT, REFS_ARGS}"
  - "cache::{RepoCache, RepoHistory, MAX_CACHED_HISTORIES}"
  - "AppState::cache — cache khoá theo RepoId, giải phóng khi close_repo"
  - "commands::history::{get_commit_page, list_refs, get_commit_detail, search_commits}"
  - "commands::history::{CommitPage, CommitDetail, FileChange}"
  - "src/lib/ipc.ts — Commit, Edge, GraphRow, CommitPage, RefKind, GitRef, FileChange, CommitDetail + bốn hàm gọi"
affects:
  - "02-05 (bộ vẽ): đọc hình dạng GraphRow từ ipc.ts; lane có thể tới 24, phải gập"
  - "02-06 (giao diện): đọc CommitPage/GitRef/CommitDetail từ đây; getCommitPage là nguồn dữ liệu"
  - "02-07 (checkpoint #1): chạy tests/history_budget.rs --release để so ngân sách"
  - "Phase 3 (diff): FileChange.path là đầu vào; cache diff dùng crate lru, KHÔNG dùng RepoCache"
tech-stack:
  added: []
  patterns:
    - "for-each-ref dùng escape %1f, KHÔNG %x1f — hai lệnh git hai ngôn ngữ định dạng"
    - "%(*objectname) bắt buộc cho tag có chú thích, nếu không nhãn không neo được vào hàng"
    - "RepoCache trả Arc, không clone Vec<Commit>"
    - "gán lane chạy trên TOÀN BỘ lịch sử; phân trang chỉ là slice lúc trả về"
    - "R/C của --name-status chiếm HAI đường dẫn; đọc sai làm lệch mọi bản ghi sau"
    - "cổng grep đếm chuỗi thay bằng test phân tích khối generate_handler!"
key-files:
  created:
    - "src-tauri/src/git/parsers/refs.rs (520 dòng, 19 test đơn vị)"
    - "src-tauri/src/cache/mod.rs"
    - "src-tauri/src/cache/repo_cache.rs (415 dòng, 10 test đơn vị)"
    - "src-tauri/src/commands/history.rs (650 dòng, 10 test đơn vị)"
    - "src-tauri/tests/refs_fixtures.rs (5 test tích hợp)"
    - "src-tauri/tests/history_commands.rs (14 test tích hợp)"
    - "src-tauri/tests/history_budget.rs (1 test #[ignore], đo ngân sách)"
    - "src/lib/ipc.history.test.ts (11 test)"
  modified:
    - "src-tauri/src/git/parsers/mod.rs — khai báo refs"
    - "src-tauri/src/cache/... — module mới, khai báo ở lib.rs"
    - "src-tauri/src/state/mod.rs — AppState.cache; close_repo giải phóng cache"
    - "src-tauri/src/commands/mod.rs — khai báo history"
    - "src-tauri/src/lib.rs — pub mod cache; bốn command vào generate_handler!"
    - "src/lib/ipc.ts — tám kiểu + bốn hàm gọi"
decisions:
  - "REFS_FORMAT dùng %1f: for-each-ref KHÔNG diễn giải %x1f của git log (đo thật)"
  - "Thêm %(*objectname): %(objectname) của tag có chú thích là mã đối tượng tag, không phải mã commit"
  - "HISTORY_TIMEOUT = 120s cho git log --all; không mượn NETWORK_TIMEOUT vì đây là lệnh cục bộ"
  - "get_commit_detail dùng git diff với cây rỗng cho commit gốc, KHÔNG git show — một đường phân tích duy nhất"
  - "Merge commit diff với parents[0]"
  - "MAX_CACHED_HISTORIES = 2 suy từ ~67MB/lịch sử so với hạn 150MB"
  - "close_repo gọi cache.invalidate — không làm thì repo đã đóng giữ 67MB"
  - "KHÔNG thêm variant GitError nào; GitErrorCode giữ nguyên chín giá trị"
  - "Thay hai cổng grep của plan bằng test thật; cổng cũ xanh khi command chưa đăng ký"
  - "HIST-01/02/06/07/10/11 giữ Pending: backend đủ nhưng chưa có gì hiển thị"
metrics:
  duration: "~85 phút"
  completed: "2026-09-21"
  tasks: 3
  commits: 5
  cargo_test: "149 đỗ + 1 ignored (90 → 149, thêm 59)"
  npm_test: "68 đỗ (57 → 68, thêm 11)"
  hot_path_release: "793.9 ms / 1000 ms; phần plan này thêm 0.903 ms / ~168 ms"
---

# Phase 2 Plan 04: Bộ phân tích ref, cache theo RepoId, và các command lịch sử — Summary

Bề mặt backend của phase đã đủ: `parse_refs`, cache khoá theo `RepoId` trả `Arc`, bốn
Tauri command chỉ đọc, và kiểu TypeScript khớp từng tên khoá. Phần plan này thêm vào
đường nóng là **0,9ms trong ngân sách ~168ms** — và trên đường đi, việc chạy git thật
thay vì đọc tài liệu đã phát hiện **hai lỗi định dạng** mà bốn tài liệu kế hoạch cùng mắc.

## Hai lỗi của kế hoạch, cả hai đều hỏng trong im lặng

### 🔴 Lỗi 1: `for-each-ref` dùng `%1f`, không phải `%x1f`

`CONTEXT.md`, `<context>` của plan 02-04 và `docs/01-research-competitors.md` mục 5.2
đều ghi:

```bash
git for-each-ref --format="%(refname)%x1f%(objectname)%x1f..."
```

**`%x1f` là escape của `git log` pretty-format, không phải của `for-each-ref`.** Đo thật
trên git 2.54.0.windows.1:

```
$ git log -1 --format='%H%x1f%P' | od -c
  0 f 9 6 f e … 3 e 037 8 6 5 1 …                  ← 037 = 0x1f, ĐÚNG

$ git for-each-ref --format='%(refname)%x1f%(objectname)' | od -c
  r e f s / h e a d s / m a i n % x 1 f 0 f 9 6 …  ← VĂN BẢN, SAI

$ git for-each-ref --format='%(refname)%1f%(objectname)' | od -c
  r e f s / h e a d s / m a i n 037 0 f 9 6 …      ← 037, ĐÚNG
```

Tại sao lỗi này nguy hiểm: lệnh **thoát 0**, stderr **rỗng**, stdout **có dữ liệu trông
hợp lý**. Chỉ là không có dấu phân tách nào — nên `parse_refs` thấy mỗi dòng có đúng một
trường, bỏ hết theo đúng quy tắc "dòng thiếu trường thì bỏ", và thanh bên **rỗng hoàn
toàn** mà không một thông báo lỗi nào.

**Và cả 19 test đơn vị vẫn xanh**, vì chúng tự dựng buffer với `0x1f` thật. Chỉ có test
tích hợp chạy git thật bắt được. Đã thêm `refs_format_sinh_byte_phan_tach_that_khong_sinh_van_ban`
khẳng định stdout **chứa byte 0x1f** và **không chứa** văn bản `%x1f` — khẳng định thứ
hai là phần nói cho người sau biết *vì sao* nó đỏ.

### 🔴 Lỗi 2: `%(objectname)` của tag có chú thích không phải mã commit

Plan đặc tả năm trường với `%(objectname)` làm `Ref::target`. Đo thật:

```
refs/tags/v1.0 | 5d051f68…  | objecttype=commit | *objectname=
refs/tags/v2.0 | dbf19145…  | objecttype=tag    | *objectname=5d051f68…

$ git cat-file -t dbf191451e74280f939ba8252d20a79ecafab9fd
tag
```

Với tag **có chú thích**, `%(objectname)` là mã của *đối tượng tag*. Mã đó không xuất
hiện ở bất kỳ hàng nào trong `git log`, nên nhãn tag **không neo được vào dòng nào** —
nó biến mất khỏi đồ thị. Điều này vi phạm trực tiếp must-have của plan: *"Ref trỏ đúng
mã commit để giao diện neo nhãn vào hàng commit"*, và doc comment của `domain::Ref.target`
(viết ở 02-02) đã hứa *"đã giải tham chiếu nếu là tag có chú thích"* — `%(objectname)`
không làm việc đó.

**Đã sửa:** thêm `%(*objectname)` làm trường thứ sáu (đặt **cuối** để đầu ra cũ vẫn đọc
được phần đầu), lấy nó khi khác rỗng. Không repo mẫu nào có tag, nên test tích hợp tự
dựng repo tạm với một tag nhẹ và một tag có chú thích, rồi khẳng định `target` của **cả
hai** bằng `git rev-parse HEAD`, cộng một khẳng định tiền đề rằng mã đối tượng tag thật
sự khác mã commit.

## Chữ ký cuối của bốn command

```rust
#[tauri::command]
pub async fn get_commit_page(
    repo_id: String, skip: usize, limit: usize, state: State<'_, AppState>,
) -> Result<CommitPage>;

#[tauri::command]
pub async fn list_refs(repo_id: String, state: State<'_, AppState>) -> Result<Vec<Ref>>;

#[tauri::command]
pub async fn get_commit_detail(
    repo_id: String, commit_id: String, state: State<'_, AppState>,
) -> Result<CommitDetail>;

#[tauri::command]
pub async fn search_commits(
    repo_id: String, query: String, state: State<'_, AppState>,
) -> Result<Vec<String>>;
```

Cả bốn nhận `repo_id` **tường minh** và dùng `state.get_repo()`. Lời gọi `active_repo()`
duy nhất trong tệp nằm ở dòng 208 và **không** dùng để tra repo — nó nói cho cache biết
repo nào không được loại bỏ.

### Hình dạng payload

```rust
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct CommitPage {
    pub commits: Vec<Commit>,       // → commits
    pub graph_rows: Vec<GraphRow>,  // → graphRows   (LUÔN cùng độ dài với commits)
    pub total: usize,               // → total       (tổng TOÀN BỘ lịch sử, không phải trang)
    pub skipped_records: usize,     // → skippedRecords
}

#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub commit: Commit,             // → commit
    pub files: Vec<FileChange>,     // → files
    pub truncated: bool,            // → truncated
}

#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct FileChange {
    pub status: String,             // → status    ("M", "A", "R100", "C75", …)
    pub path: String,               // → path
    pub old_path: Option<String>,   // → oldPath   (null trừ R và C)
}
```

`status` là `String` chứ không `char` như plan viết: git in kèm điểm tương đồng
(`R100`, `C75`), và một `char` không chứa được `R100`.

### Hạn giờ đã chọn: 120 giây, và lý do

`HISTORY_TIMEOUT = Duration::from_secs(120)` áp cho `git log --all` và cho lệnh tìm theo
đường dẫn.

- **`DEFAULT_TIMEOUT` 30s quá ngắn.** Repo mẫu 100k commit mất 667–693ms trên máy nhanh
  với bộ nhớ đệm hệ điều hành đã ấm. Một repo hàng triệu commit trên đĩa nguội vượt 30
  giây dễ dàng, và khi đó người dùng thấy `GitError::Timeout` thay vì thấy lịch sử.
- **Không mượn `NETWORK_TIMEOUT`** (cũng 120s): đây là lệnh **cục bộ**, không đi qua
  mạng. Dùng chung hằng số nghĩa là ngày nào đó ai chỉnh hạn giờ mạng sẽ vô tình đổi
  hành vi nạp lịch sử, và không ai liên hệ được hai việc đó với nhau.
- **Vẫn phải có một hạn giờ** (T-02-14): thiếu nó thì một repo hỏng treo giao diện vĩnh
  viễn.

### Commit gốc trong `get_commit_detail`

Commit gốc **không có** `<commit>^`, nên `git diff <gốc>^ <gốc>` thất bại với `unknown
revision`. Một cài đặt chỉ nối `^` sẽ lỗi ở đúng commit đầu tiên của **mọi** repo.

Cách đã chọn: phát hiện bằng `commit.parents.is_empty()` rồi dùng cây rỗng
`4b825dc642cb6eb9a060e54bf8d69288fbee4904` làm vế trái. Thực tế mã viết gọn hơn —
`commit.parents.first().cloned().unwrap_or(EMPTY_TREE)` — nên commit gốc và commit
thường đi qua **cùng một** dòng lệnh, chỉ khác vế trái.

**Vì sao không dùng `git show --name-status`** (plan cho chọn một trong hai): `git show`
mặc định **không in gì** cho merge commit, nên `get_commit_detail` sẽ có hai hành vi tuỳ
commit là gốc hay merge. `git diff` cho cả ba trường hợp giữ đúng một đường phân tích
đầu ra, và điểm khác biệt duy nhất là mã ở vế trái. Merge commit diff với `parents[0]`.

Test `chi_tiet_commit_goc_khong_loi_va_co_tep` khẳng định cả kết quả (`A\0` là byte đầu)
**và tiền đề** (`git rev-parse <gốc>^` thoát khác 0) — không có khẳng định thứ hai thì
test vẫn đỗ trên một repo mà commit đầu tiên tình cờ có cha.

### Đối chiếu `GitErrorCode` ↔ `GitError::code()`

Plan yêu cầu ghi kết quả đối chiếu vào summary. **Khớp chính xác, chín đổi chín, không
cần đổi gì:**

| `GitError::code()` | `GitErrorCode` trong `ipc.ts` |
|---|---|
| `spawn_failed` | ✅ |
| `timeout` | ✅ |
| `stdin_write_failed` | ✅ |
| `command_failed` | ✅ |
| `not_a_repository` | ✅ |
| `no_repository_open` | ✅ |
| `unknown_repository` | ✅ |
| `parse_failed` | ✅ |
| `io` | ✅ |

Plan này **không thêm variant nào**. `InvalidPageRange` mà plan nêu làm ví dụ hoá ra
không cần: `skip` ngoài phạm vi trả **trang rỗng** chứ không trả lỗi — một trang rỗng là
câu trả lời đúng cho "cho tôi commit thứ 1 000 000", không phải một tình huống lỗi. Dùng
lại `UnknownRepository` và `ParseFailed`.

## Xác minh — đầu ra thật, không diễn giải

```
cargo test                                → 149 đỗ + 1 ignored (7 suite)  [nền 90, thêm 59]
cargo clippy --all-targets -- -D warnings → No issues found
cargo fmt --all --check                   → exit 0
npm run typecheck                         → exit 0
npm test                                  → 68 đỗ (6 tệp)                 [nền 57, thêm 11]
npx tauri build --debug --no-bundle       → Built application at target\debug\git-plum.exe
```

Cổng grep của `<verification>`:

```
/usr/bin/grep -c 'get_commit_page' src-tauri/src/lib.rs                            → 1   (≥1 ✓)
/usr/bin/grep -v '^\s*//' .../history.rs | /usr/bin/grep -c 'runner.write'         → 0   ✓
```

### Ngân sách đường nóng — 100 007 commit, profile release

```
=== Ngân sách đường nóng, 100007 commit ===
  git log               666.9 ms   (wave 2/3)
  parse_log              67.0 ms   (wave 2)
  assign                 59.0 ms   (wave 3)
  ---- plan 02-04 thêm vào ----
  cache put             0.003 ms
  cache get             0.000 ms
  cắt trang  100        0.049 ms
  JSON   77765 byte     0.851 ms
  TỔNG THÊM VÀO         0.903 ms   (ngân sách ~168 ms)
  TỔNG ĐƯỜNG NÓNG       793.9 ms   (mốc 1000 ms)

  Trang thứ hai (cache hit, không git): 0.636 ms
```

Phần plan này thêm dùng **0,54% ngân sách ~168ms**. `parse_log` 67ms và `assign` 59ms
tái lập đúng số của wave 3 (82ms/57ms, trong nhiễu), nên phép đo so sánh được.

**Một cái bẫy đo đạc đã gặp và ghi lại:** chạy cùng test ở profile **debug** cho
`parse_log` **455ms**, `assign` **220ms**, tổng đường nóng **1366ms** — tức *vượt* mốc
một giây. Debug chậm gần **7 lần** ở `parse_log`. Bảng của wave 3 đo bằng `criterion`
(luôn dựng release), nên chỉ số release là chỉ số so sánh được. Đọc số debug rồi kết
luận "trượt Core Value" hay "cache quá đắt" đều sai. Đã ghi bảng debug-vs-release vào
doc comment của `tests/history_budget.rs` để người sau không mắc lại.

### Kiểm chứng đột biến — đã thực sự chạy

| # | Đột biến | Kết quả | Test nào bắt |
|---|---|---|---|
| 1 | `%1f` → `%x1f` (hoàn tác sửa định dạng) | **5 test tích hợp đỏ**, thông báo dán nguyên stdout `refs/heads/main%x1f0f96fe…` | `refs_format_sinh_byte_phan_tach_that…` nêu đúng nguyên nhân. **19 test đơn vị vẫn XANH** |
| 2 | Bỏ `%(*objectname)`, luôn dùng `objectname` | **2 test đơn vị + 1 tích hợp đỏ.** *"tag cochuthich phải trỏ mã COMMIT bec4638b… nhưng trỏ 1621306…"* | đơn vị `tag_co_chu_thich_tra_ma_commit…` + tích hợp trên git thật |
| 3 | `is_head = fields[4].contains('*')` → `!fields[4].is_empty()` | **1 đơn vị + 1 tích hợp đỏ.** *"HEAD tách rời: không ref nào được có is_head = true, nhưng ["refs/heads/main", "refs/remotes/origin/HEAD", "refs/remotes/origin/main"] có"* | `head_dau_sao_bat_co_dau_cach_thi_khong`, `detached_khong_ref_nao_la_head` |
| 4 | Xoá vòng loại bỏ của cache (bỏ chặn trên) | **3 test đỏ.** *"sau 3 lần put, cache giữ 3 lịch sử — vượt chặn trên 2"* | `khong_bao_gio_giu_nhieu_hon_max_cached_histories` |
| 5 | Loại bỏ không xét repo đang hoạt động | **1 test đỏ.** *"repo đang hoạt động phải được giữ lại"* | `loai_bo_giu_lai_repo_dang_hoat_dong` |
| 6 | Thay dòng đăng ký bằng chú thích `// TODO: dang ky get_commit_page` | **Cổng grep của plan XANH (trả 1).** Test mới **đỏ**: *"thiếu `commands::get_commit_page,` trong generate_handler!"* | `bon_command_deu_nam_trong_generate_handler` |
| 7 | Xoá `.arg("--")` khỏi `search_commits` | **Lần đầu: KHÔNG test nào đỏ.** Sau khi thêm test nguồn: **1 test đỏ** | `search_commits_dung_dau_gach_ngang_truoc_pathspec` |
| 8 | `{ repoId }` → `{ repo_id: repoId }` trong `getCommitPage` | **2 test frontend đỏ**, diff in `- "repoId"` / `+ "repo_id"` | `getCommitPage gọi get_commit_page…`, `mọi hàm dùng khoá camelCase repoId…` |

**Đột biến 1 và 7 là hai kết quả đáng đọc nhất.**

Đột biến 1 chứng minh vì sao test tích hợp không phải thứ trang trí: lỗi định dạng làm
thanh bên rỗng hoàn toàn trong khi **toàn bộ 19 test đơn vị vẫn xanh**, vì test đơn vị
tự chèn `0x1f` vào buffer của chúng. Không có test chạy git thật thì lỗi này ship ra.

Đột biến 7 là **một test của tôi đã không đủ tốt**: `tim_theo_duong_dan_co_dau_gach_ngang_ngan_cach`
tự dựng lệnh git của nó, nên nó chứng minh `--` *có tác dụng* mà không chứng minh
`search_commits` *dùng* nó. Xoá `.arg("--")` khỏi mã sản phẩm, test vẫn xanh. Đã thêm
`search_commits_dung_dau_gach_ngang_truoc_pathspec` đọc thân hàm thật (bỏ dòng chú
thích) và khẳng định `--` xuất hiện **trước** pathspec.

## Chệch khỏi plan

### `[Rule 1 - Bug]` `%x1f` → `%1f` trong `REFS_FORMAT`

Xem mục "Hai lỗi của kế hoạch" ở trên. Bốn tài liệu cùng sai.

### `[Rule 1 - Bug]` Thêm trường thứ sáu `%(*objectname)`

Plan đặc tả năm trường. Xem mục trên.

### `[Rule 1 - Bug]` `FileChange.status` là `String`, không phải `char`

Plan viết `status: char→String`. `char` không chứa được `R100` hay `C75`, mà git **luôn**
in điểm tương đồng khi có `--find-renames`. Giữ nguyên chuỗi git trả về.

### `[Rule 2 - Missing critical]` `R`/`C` của `--name-status` chiếm HAI đường dẫn

Plan không nhắc. Đo thật:

```
M\0file.txt\0
R100\0local1.txt\0renamed1.txt\0
```

Đọc `R`/`C` như bản ghi một đường dẫn làm **mọi** bản ghi sau nó lệch một nấc: đường dẫn
mới bị đọc thành trạng thái, trạng thái kế tiếp bị đọc thành đường dẫn. Hỏng cả danh
sách chứ không chỉ một dòng. Test `name_status_doi_ten_chiem_hai_duong_dan_khong_lam_lech_ban_ghi_sau`
đặt thêm hai bản ghi **sau** bản ghi `R` để bắt đúng dạng lệch đó.

### `[Rule 2 - Missing critical]` `close_repo` giải phóng cache

Plan chỉ yêu cầu thêm `cache` vào `AppState`. Không gọi `cache.invalidate` trong
`close_repo` thì một lịch sử ~67MB nằm lại trong RAM cho repo người dùng **đã đóng**, và
`MAX_CACHED_HISTORIES` sẽ bảo vệ một thứ không ai còn nhìn. Đây là ràng buộc RAM dưới
150MB của `PROJECT.md`, không phải dọn dẹp cho gọn. Thêm test `closing_repo_frees_its_cache`.

### `[Rule 2 - Missing critical]` `invalidate` xoá cả refs, không chỉ lịch sử

Plan đặc tả `invalidate(repo_id)` mà không nói xoá những gì. Xoá cả hai có chủ ý: ref trỏ
vào mã commit, nên một danh sách ref còn sống cạnh một lịch sử đã bỏ sẽ cho nhãn trỏ vào
hàng không tồn tại.

### `[Rule 2 - Missing critical]` `put_history_keeping` — biến `keep` thành tham số

Plan viết *"đọc `active_repo()` từ `AppState`"* trong `put_history`. Làm đúng chữ đó
nghĩa là `cache` phải biết về `AppState`, tức phụ thuộc vòng (`state` đã giữ `cache`).
Đã đảo hướng: `RepoCache::put_history_keeping(repo, h, keep)` nhận `keep` làm tham số, và
`commands::history` truyền `state.active_repo()` vào. Cache vẫn không biết gì về
`AppState`, nên test được mà không cần dựng state.

Thêm một ca mà plan không nêu: **repo vừa `put` chính là repo đang hoạt động**. Một cài
đặt "luôn giữ repo đang hoạt động" viết sơ sài sẽ kết luận không có gì loại được rồi giữ
cả ba. Đã xử lý bằng một bước lùi (nếu không còn ứng viên thì repo đang hoạt động cũng
phải nhường) và có test riêng.

### `[Rule 2 - Missing critical]` Bỏ `\r` cuối dòng trong `parse_refs`

Plan không nhắc. Trên Windows git có thể kết thúc dòng bằng `\r\n`, và trường cuối của
`REFS_FORMAT` là `*objectname` — một `\r` sót lại làm mã commit dài **41** ký tự và mọi
phép neo nhãn thất bại trong im lặng. Cùng hạng lỗi với `\n` đầu bản ghi mà 02-02 đã
gặp. Test `ket_thuc_dong_crlf_khong_de_lot_ky_tu_cr` khẳng định `target.len() == 40`.

### `[Rule 3 - Blocking]` Ba lỗi môi trường, không phải lỗi mã

1. **`git-plum.exe` bị giữ handle.** `cargo test` thất bại với `LNK1104 cannot open file`
   / `Access is denied (os error 5)` vì một tiến trình `git-plum.exe` từ phiên trước còn
   chạy, cộng các tiến trình con `msedgewebview2.exe`. Xử lý: `taskkill //IM git-plum.exe //F`
   rồi xoá `target/debug/git-plum.exe`. Tái diễn vài lần trong phiên.
2. **Doctest đỏ giả** với `extern location for git_plum_lib does not exist` khi lần build
   trước bị ngắt giữa chừng — `cargo build --lib` rồi chạy lại là hết. Không phải lỗi mã.
3. **`perl -0pi` làm hỏng UTF-8.** Một lần chèn test bằng `perl -0pe` với nội dung tiếng
   Việt biến toàn bộ tệp thành mojibake (`MÃ£ cÃ¢y rá»ng`) và vẫn thoát 0. Khôi phục bằng
   `git checkout --` rồi chèn lại bằng công cụ Edit. `head -N` cũng làm hỏng một tệp
   tương tự. Với tệp có tiếng Việt, **không dùng** `perl -0p`/`head` để sửa tại chỗ.

## Plan sai ở đâu

### Cổng `grep -c 'get_commit_page' src-tauri/src/lib.rs` ≥ 1 là cổng vô dụng

Đúng hạng lỗi mà ba wave trước đều mắc và mà hướng dẫn chất lượng của phase cảnh báo.
Đã kiểm bằng đột biến 6: thay dòng đăng ký thật bằng

```rust
// TODO: dang ky get_commit_page
```

thì cổng của plan trả **1** và **xanh**, trong khi `invoke('get_commit_page')` sẽ thất
bại lúc chạy với "command not found" — đúng cái lỗi mà plan gọi là "lỗi kinh điển".

**Cổng đã sửa:** test `bon_command_deu_nam_trong_generate_handler` tách khối
`generate_handler![...]`, **bỏ dòng chú thích**, rồi đòi `commands::<tên>,` cho cả bốn
command. Đỏ đúng trên tệp bị phá.

Cổng số 4 (`runner.write`) thì **lành**: đã kiểm nó đỏ được (chèn lời gọi ghi thật → đếm
4), và bộ lọc `grep -v '^\s*//'` chặn đúng cả `//` lẫn `///`. Vẫn nhân đôi bằng test
`khong_co_loi_goi_ghi_nao_trong_command_lich_su` để CI bắt được mà không phụ thuộc ai
nhớ chạy lệnh.

### `terminates` — plan nói đúng, và điều chỉnh của wave 3 đã được tôn trọng

Hướng dẫn thực thi nhắc rằng biên shallow **không** nhận ra được từ `%P`, và `terminates`
thật sự thuộc ca **phân trang**. Xác nhận: plan này **không** cố suy shallow từ danh sách
cha, và **không** đọc `.git/shallow` — vì `get_commit_page` nạp `git log --all` **toàn
bộ** nên trong lịch sử đã cache không có cha nào bị thiếu, và `terminates` là `false` ở
mọi hàng. Đường `terminates == true` mà 02-03 kiểm bằng `--max-count` sẽ chỉ xuất hiện
nếu plan sau chuyển sang nạp từng trang thật.

Nếu giao diện cần vẽ dấu "còn tiếp" cho bản sao nông thì việc đọc `.git/shallow` vẫn
**chưa làm** — xem phần Known Stubs.

### Con số bộ nhớ của plan không kiểm lại được trong phase này

`<objective>` đưa bảng ~67MB/lịch sử dựa trên `size_of` và giả định bề rộng nhánh. Tôi
**dùng** con số đó để suy `MAX_CACHED_HISTORIES = 2` và ghi phép tính vào doc comment,
nhưng **không đo RSS thật** — đo RAM lúc rảnh của ứng dụng đã dựng là việc của checkpoint
02-07, không phải của một `cargo test`. Ghi rõ ở đây để không ai đọc summary này rồi
tưởng hạn 150MB đã được xác nhận bằng thực nghiệm.

## Cổng an toàn đã cài (threat model)

- **T-02-11 (Tampering)** — `search_commits` luôn có `.arg("--")` **trước** pathspec, và
  điều đó được khẳng định trên **thân hàm thật** sau khi đột biến 7 cho thấy test cũ
  không đủ. `GitCommand::args` truyền argv trực tiếp cho `tokio::process`, không qua
  shell, nên không có đường tiêm shell.
- **T-02-12 (EoP)** — cả bốn command tra repo bằng `state.get_repo(&repo_id)`, chỉ trả
  repo **đã mở** qua `open_repository`. Test `repo_id_la_khong_tra_ve_repo_nao` thử cả
  `"C:/Windows/System32"` và `"../../../etc/passwd"` — đều `None`. Đường dẫn không bao
  giờ đến từ webview ở các command này.
- **T-02-13 (DoS)** — `limit` kẹp ở `MAX_PAGE_LIMIT = 10_000`; cắt trang dùng
  `skip.min(total)` và `saturating_add`. Test chạy `skip` bằng `usize::MAX` và `limit`
  bằng `usize::MAX` — trang rỗng / kẹp đúng, không panic, không tràn.
- **T-02-14 (DoS)** — `HISTORY_TIMEOUT` 120s tường minh trên `git log --all` và trên lệnh
  tìm theo đường dẫn. `GitError::Timeout` mang argv để hiện cho người dùng.
- **T-02-15 (Repudiation)** — chấp nhận, đã ghi doc comment ở **hai** chỗ (`RepoCache` và
  `lich_su_cua`) nói rõ cache hit cố ý không sinh dòng nhật ký, để người đọc nhật ký
  không tưởng là thiếu.

Thêm ngoài register: `MAX_FILES_PER_COMMIT = 5_000` kèm cờ `truncated`, và
`MAX_SEARCH_RESULTS = 1_000`.

## Known Stubs

Không có stub theo nghĩa mã giả vờ hoạt động. Ba việc **cố ý** chưa làm, đều có lý do
thuộc phạm vi:

1. **`CommitPage.skipped_records` luôn trả `0`.** Bộ đếm thật nằm trong `LogParseResult`
   và được ghi vào `tracing::warn!` lúc nạp, nhưng nó không được cất vào `RepoHistory`
   nên trang cắt từ cache không biết lần nạp đã bỏ bao nhiêu bản ghi. Trường vẫn có
   trong payload và trong kiểu TS để hợp đồng không phải đổi về sau. Muốn nó mang số
   thật thì `RepoHistory` cần thêm một trường — một thay đổi nhỏ, nhưng thuộc plan nào
   thật sự **hiển thị** cảnh báo đó cho người dùng.
2. **Nhận biết bản sao nông chưa đọc `.git/shallow`.** Cần cho việc vẽ dấu "còn tiếp" ở
   biên nông. Không chặn gì trong phase này vì `git log --all` nạp toàn bộ.
3. **`RepoCache` chưa có đường làm mất hiệu lực khi repo đổi trên đĩa.** `invalidate` đã
   có và `close_repo` gọi nó, nhưng chưa ai theo dõi `.git` để gọi tự động. Phase 2 thuần
   đọc nên lịch sử không tự đổi dưới chân người dùng; từ Phase 4 (có lệnh ghi) thì đây
   trở thành việc bắt buộc.

## Ghi chú cho plan sau

**Cho 02-05 (bộ vẽ):** `GraphRow` trong `ipc.ts` đã có, tên khoá ghim bằng test hai
phía. `lane` **có thể tới 24** trên fixture `wide` và `passthrough` có thể chứa
`toLane ≥ 20` — bộ vẽ **phải** gập lane ≥ 20 vào cột cuối, kiểu TS cố ý không cấm giá
trị lớn và có test khẳng định điều đó.

**Cho 02-06 (giao diện):**
- `ipc.getCommitPage(repoId, skip, limit)` là nguồn dữ liệu. `total` là tổng **toàn bộ**
  lịch sử, dùng cho chiều cao vùng cuộn ảo hoá; `commits.length` chỉ là kích thước trang.
- `graphRows` **luôn** cùng độ dài `commits`, khớp theo chỉ số. Vẽ hai cột từ cùng một
  mảng `virtualItems`.
- `GitRef.target` là mã **commit** đã giải tham chiếu — neo nhãn vào
  `GraphRow.commitId` bằng phép so bằng thẳng, không cần xử lý gì thêm cho tag.
- `upstream === null` nghĩa là chưa đặt thượng nguồn. `[gone]` cho `upstream` **khác
  null** với `ahead === 0 && behind === 0` — hai trạng thái khác nhau, đừng gộp.
- `isHead` có thể **false ở mọi ref** (HEAD tách rời). Đó là trạng thái hợp lệ.
- `useInfiniteQuery` khớp trực tiếp với `getCommitPage`; `staleTime: Infinity` đúng vì
  đối tượng git là bất biến.

**Cho 02-07 (checkpoint #1):** chạy
`cargo test --release --test history_budget -- --nocapture --ignored`. **Phải là
`--release`** — xem bảng debug-vs-release ở trên.

**Cho Phase 3 (diff):** `FileChange.path` là đầu vào. Cache diff dùng crate `lru` khoá
theo SHA commit như STACK.md nói; **không** mở rộng `RepoCache` cho việc đó — doc comment
của `MAX_CACHED_HISTORIES` ghi rõ vì sao hai cache là hai thứ khác nhau.

## Trạng thái requirement

**HIST-01, HIST-02, HIST-06, HIST-07, HIST-10, HIST-11 giữ `Pending`** — theo đúng tiền
lệ mà 02-01, 02-02 và 02-03 đã lập.

Cả sáu mô tả hành vi người dùng **quan sát được**: HIST-01 là *"người dùng thấy danh
sách commit, nạp theo trang"*, HIST-06/07 là *"người dùng thấy nhãn và thanh bên"*,
HIST-10 là *"gõ từ khoá thì tìm được"*, HIST-11 là *"vẫn hiển thị được"*. Sau plan này
backend đã **đủ** cho cả sáu và được kiểm trên dữ liệu git thật, nhưng **chưa có gì
hiển thị** — không component, không bộ vẽ, `AppLayout` vẫn là ba chỗ trống ghi "Phase 2
sẽ điền". Trường `requirements:` của plan hiểu là *"góp phần vào"*.

HIST-11 được ghim thêm một bậc ở tầng ref: tên nhánh chứa byte không UTF-8 vẫn ra một
`Ref` và **không làm mất ref nào sau nó** (test đặt một ref hợp lệ ngay sau ref xấu).

## Self-Check: PASSED

Tệp đã tạo (đều tồn tại):
- `src-tauri/src/git/parsers/refs.rs`
- `src-tauri/src/cache/mod.rs`
- `src-tauri/src/cache/repo_cache.rs`
- `src-tauri/src/commands/history.rs`
- `src-tauri/tests/refs_fixtures.rs`
- `src-tauri/tests/history_commands.rs`
- `src-tauri/tests/history_budget.rs`
- `src/lib/ipc.history.test.ts`

Commit đã tạo (đều có trong `git log`):
- `c3d2673` test(02-04): pin ref parsing and per-repo cache behaviour before implementing
- `0ef9507` feat(02-04): parse refs and cache history per repository
- `60aca27` feat(02-04): add the four read-only history commands
- `5fbbffe` feat(02-04): type the history IPC boundary and pin the call contract
- `cafc1cb` test(02-04): measure what the cache layer adds to the hot path
