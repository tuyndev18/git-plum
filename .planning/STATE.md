---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: blocked_on_checkpoint
last_updated: "2026-09-21T10:23:33.000Z"
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 11
  completed_plans: 8
  percent: 73
---

# Project State: git-plum

**Last updated:** 2026-09-21

---

## Project Reference

**Core Value:** Đọc và hiểu lịch sử của một repository phải tức thì — đồ thị commit mở ra trong dưới một giây và cuộn mượt kể cả trên repo hàng chục nghìn commit.

**Current focus:** Phase 2 — Lịch sử và đồ thị nhánh (Core Value)

**Mode:** mvp (Vertical MVP) · **Granularity:** standard · **Parallelization:** enabled

---

## Current Position

| | |
|---|---|
| **Phase** | 2 — Lịch sử và đồ thị nhánh |
| **Plan** | 02-05 — Task 1-3/4 xong (hình học + canvas + CommitList + App.tsx nối thật); **DỪNG ở Task 4** |
| **Status** | **Chờ checkpoint người dùng** — Task 4 của 02-05 là `checkpoint:human-verify gate="blocking"`, đòi sáu bước kiểm bằng mắt (thẳng hàng lúc cuộn nhanh, độ nét HiDPI, bấm chọn qua đồ thị) mà agent không tự trả lời được. Xem `.planning/phases/02-history-graph/02-05-SUMMARY.md` mục "Sáu bước cần làm". |
| **Progress** | Phase 1/8 · Phase 2 plan 4/7 hoàn tất + 02-05 dở (3/4 task) |

```
[#.......] 1/8 phases
```

**Phase 1 đóng ở trạng thái nào.** Bốn plan xong, mọi kiểm thử tự động xanh (23 test Rust,
57 test frontend), bản release dựng được. Nhưng **ba tiêu chí thành công chưa kiểm chứng**
vì chủ dự án hoãn phần QA thủ công để đi tiếp Phase 2:

| Tiêu chí | Chưa biết gì |
|---|---|
| 1 — danh sách repo gần đây | `tauri-plugin-store` bị giả lập trong mọi test. Chưa rõ `recent-repos.json` có ghi đúng thư mục dữ liệu ứng dụng, và `autoSave` có kịp ghi trước khi tiến trình thoát |
| 2 — bố cục còn nguyên sau khởi động lại | Chưa ai chạy thử. Và đã biết **không trọn** — xem G8 |
| 5 — không nháy cửa sổ console | Chỉ hiện ở bản release, đọc mã không thay được việc chạy thử |

**Cách trả nợ này:** bấm đúp `src-tauri/target/release/git-plum.exe` (đã dựng sẵn) rồi chạy
sáu kịch bản trong `docs/03-phase1-qa-windows.md`. Không phụ thuộc tiến độ Phase 2, làm lúc
nào cũng được. Chi tiết đầy đủ ở `.planning/phases/01-platform-git-layer/VERIFICATION.md`.

---

## Performance Metrics

| Metric | Value |
|---|---|
| Phases completed | 1 / 8 (có nợ) |
| Plans completed | 8 |
| v1 requirements delivered | 7 / 59 đã kiểm chứng · 2 nữa có mã nhưng chưa kiểm thật (PLAT-07, PLAT-09) · PLAT-01 và REL-04 đạt ở mức yếu hơn ROADMAP |

| Plan | Thời lượng | Tasks | Files |
|---|---|---|---|
| Phase 1 P01 | 18min | 2 tasks | 2 files |
| Phase 1 P02 | 9min | 3 tasks | 6 files |
| Phase 1 P03 | 14min | 3 tasks | 6 files |
| Phase 2 P01 | 50min | 3 tasks | 9 files |
| Phase 2 P02 | 35min | 2 tasks | 8 files |
| Phase 2 P03 | 65min | 3 tasks | 14 files |
| Phase 2 P04 | 85min | 3 tasks | 14 files |
| Phase 2 P05 (Task 1-3, dở) | ~110min | 3/4 tasks | 12 files |
| Phase 2 P05 checkpoint round 1 (điều tra + sửa) | ~90min | 0 task mới | 4 files |

---

## Accumulated Context

### Quyết định đã chốt

- **Tauri v2 + React + TypeScript + Rust, gọi thẳng `git` CLI.** Không dùng libgit2/gitoxide.
- **Bốn mục "làm ngay hoặc không bao giờ" nằm trọn trong Phase 1**: PLAT-02 (ghim biến môi trường), PLAT-03 (xếp hàng ghi theo repo), PLAT-04 (sổ lệnh trung tâm), PLAT-05 (trạng thái khoá theo repo id).
- **Nhóm WORK tách làm hai phase** (4 và 5). Nghiên cứu nâng độ khó staging theo khối lên HIGH; tách ra để vòng lặp commit cơ bản dùng được sớm.
- **BRANCH-05 nằm cùng phase với merge/rebase** vì dùng chung máy trạng thái đang-dở.
- **AI (Phase 7) là nhánh lá** — không có gì phụ thuộc vào nó, được phép cắt, và là ứng viên chạy song song với Phase 6.
- **REL bị xé đôi**: REL-04 (CI ba nền tảng) vào Phase 1; REL-01/02/03 vào Phase 8.
- **Không cam kết tổng thời gian.** Chuỗi phase là kế hoạch, thời lượng là đầu ra.
- **PLAT-02 (plan 01-01)**: `GIT_CONFIG_PARAMETERS` ghim đủ `log.showSignature`, `diff.noprefix`, `format.coverLetter` qua hằng `PINNED_GIT_CONFIG` trong `src-tauri/src/git/exec.rs`. `diff.external` ghim riêng bằng `GIT_EXTERNAL_DIFF=""` vì biến môi trường thắng cấu hình.
- **`--cleanup=whitespace` bàn giao cho Phase 4** qua `docs/02-phase4-commit-notes.md`: nó là tham số dòng lệnh của `git commit`, không đặt được trong lớp ghim môi trường, và Phase 1 chưa có lệnh commit nào.
- **Hạ tầng kiểm thử giao diện** (plan 01-02): cấu hình vitest sống trong khối `test` của `vite.config.ts`, không tách `vitest.config.ts` riêng — một nguồn sự thật cho `resolve.alias`. Giả lập IPC ở ranh giới module `@/lib/ipc`, không vá `invoke` toàn cục. Test logic store gọi thẳng `useRepoStore.getState()`, không render component.
- **Danh sách repository gần đây** (plan 01-03, PLAT-07): lưu qua `tauri-plugin-store` vào `recent-repos.json`, giới hạn `MAX_RECENT = 10`. Logic thuần tuý (`mergeRecent`, `sanitizeRecent`, `normalizeRepoPath`) tách khỏi vỏ bọc Tauri để kiểm thử được trực tiếp. `normalizeRepoPath` sao chép chính xác `state::repo_id_for` bên Rust — đổi `\` thành `/`, cắt `/` ở cuối, **không** đổi chữ thường — để phép khử trùng lặp phía giao diện không bất đồng với `RepoId` phía backend.
- **Danh sách gần đây là tiện ích, không phải dữ liệu quan trọng**: mọi lời gọi store bọc `try/catch` và trả mảng rỗng khi hỏng. `rememberRepo` thất bại **không được** làm hỏng `openRepository` — người dùng đã mở được repo rồi.
- **Đếm commit bằng `git rev-list`, không bao giờ bằng `git log`** (plan 02-01): `git log` áp *history simplification* và **lược bỏ** commit có tree trùng một cha — một merge không sửa tệp nào sẽ biến mất khỏi `git log` dù repo có merge thật. `--sparse`, `--full-history`, `--boundary`, `-m` đều không cứu được. Hệ quả cho fixture: **mọi** commit phải đổi nội dung tệp, kể cả commit merge.
- **`git fast-import` là cách duy nhất giữ được byte thô không-UTF-8** (plan 02-01, HIST-11): shell chuyển `printf 'caf\351.txt'` thành UTF-8 trước khi tới hệ thống tệp, và `git commit -F` chuyển `\377` thành `\303\277`. Chỉ luồng fast-import giữ nguyên byte ở cả đường dẫn và thông điệp. Repo mẫu `non-utf8` vì vậy dựng bằng fast-import.
- **Merge octopus: không dùng `--no-ff`** (plan 02-01): `--no-ff` thêm chính `main` làm một cha nữa (5 cha thay vì 4). Và `main` phải có commit riêng trước khi rẽ nhánh, nếu không chiến lược octopus fast-forward và chỉ ra 3 cha.
- **`module testing` là `pub`, không phải `#[cfg(test)]`** (plan 02-01): benchmark `criterion` là target riêng và không thấy mã dưới `#[cfg(test)]`, mà checkpoint #1 đòi benchmark gán lane ở 100k commit. Thiếu fixture thì `require_fixture` bỏ qua **ồn ào** (`eprintln!` + `None`), không panic — người mới clone repo chạy `cargo test` phải thấy xanh kèm lời nhắc.
- **Repo mẫu phải tất định tuyệt đối** (plan 02-01): ghim cả sáu biến `GIT_AUTHOR_*`/`GIT_COMMITTER_*`, dấu thời gian tăng dần cố định, `--initial-branch=main`, `core.autocrlf=false` (đặt **lúc clone**, không phải sau). Lý do: plan 02-03 chụp snapshot `insta` của đầu ra gán lane, SHA trôi thì snapshot đỏ vô cớ.
- **`parse_log` trả `LogParseResult` chứ không trả `Result`** (plan 02-02, HIST-11): một bản ghi méo không được làm đổ cả trang lịch sử — đó là DoS (T-02-04) vì commit hỏng do người khác tạo ra nhiều năm trước. Nhưng cũng không được biến mất im lặng, nên `skipped_records` và `bad_timestamps` đi kèm kết quả và plan 02-04 ghi `tracing::warn!` khi khác 0.
- **`LOG_FORMAT` và `LOG_ARGS` là hằng duy nhất chứa định dạng và `--topo-order`** (plan 02-02): call site không được chép lại chuỗi. `LOG_FORMAT` với `FIELD_COUNT` là hợp đồng hai chiều — sửa một bên quên bên kia làm mọi bản ghi lệch một nấc mà không báo lỗi, nên có test đếm số `%x1f`.
- **git chèn `\n` sau mỗi `\x1e`** (plan 02-02, đo bằng `xxd`: buffer chứa `1e 0a`): không cắt byte đó thì **mọi** mã commit dài 41 ký tự và mọi phép so mã về sau thất bại trong im lặng. Đây là bước `trim` đầu mỗi bản ghi trong `parse_record`.
- **Phần thừa sau dấu `\x1f` thứ chín nằm lại trong `%b`** (plan 02-02, T-02-05): người tạo commit có thể cố ý chèn `\x1f` vào thông điệp; điều đó không được sinh ra bản ghi giả. Vòng lặp `break` khi đủ chín dấu thay vì tách hết rồi nối lại — cùng kết quả byte, không cấp phát.
- **`bstr` không dùng trong `parse_log`** (plan 02-02): tách theo hai byte là việc của `memchr`, giải mã lossy là việc của `std`. Crate vẫn giữ trong `Cargo.toml` cho plan 02-04/02-05, nơi `for-each-ref` và `diff --name-status -z` có thể cần thao tác chuỗi byte thật sự.
- **Ngân sách hiệu năng Core Value, đo thật ở 100k commit** (plan 02-02): `git log` mười trường 693ms / 18,9MB + `parse_log` 63ms = **758ms**. Còn khoảng **240ms** cho gán lane trước khi chạm mốc một giây của tiêu chí 1.
- **`MAX_VISIBLE_LANES = 20`, chốt bằng lập luận HIỂN THỊ chứ không bằng số đo fixture** (plan 02-03, đóng `<open_questions>` số 2 của CONTEXT.md): `(1440 × 52% × 40% − 8) ÷ 14 = 20,9 → 20`. Chốt từ số lane lớn nhất của repo mẫu `wide` là **vòng tròn** — mọi giá trị ≥ số đo bảo đảm cap không bao giờ chạm, nên `truncated_parents` luôn bằng 0 và cả nhánh vẽ suy giảm ship ra mà chưa chạy lần nào. Ở mức 20 thì `wide` (25 lane) và repo hiệu năng (21 lane) **đều** chạm cap. Phép tính đầy đủ ở `docs/04-phase2-degraded-graph.md`.
- **Giới hạn hiển thị KHÔNG áp cho lane của hàng — hai hàm cấp lane, hai chữ ký** (plan 02-03): `allocate_row_lane -> u16` (không cap) tách hẳn `allocate_parent_lane -> Option<u16>` (có cap). Gộp làm một hàm trả `Option` là nguyên nhân gốc của lỗi **mất commit**: bước 1 gặp `None` thì `unwrap()` panic, `continue` đánh rơi hàng, gán lane ngoài giới hạn phá assertion. Đo bằng đột biến: gộp lại làm **mất 20 trong 80 hàng** trong khi **16/17 test vẫn xanh**. `rows.len() == commits.len()` là HIST-04 ở tầng dữ liệu và không được phụ thuộc hằng số hiển thị; việc gập lane vượt cap là của frontend (plan 02-05).
- **Fixture `shallow` KHÔNG kiểm được `terminates` — git ghép biên nông** (plan 02-03, đính chính CONTEXT.md và summary 02-02): đối tượng commit trên đĩa có `parent 7dae333f…`, nhưng `git log --format=%P` trả **rỗng** vì git graft biên bản sao nông. Từ dữ liệu `assign` nhận được, biên nông không phân biệt được với gốc thật, nên `terminates == false` là câu trả lời **đúng** — `assign` không được bịa cha mà git đã che. Phát hiện bản sao nông phải đọc `.git/shallow`, là việc của tầng repository. Ca `terminates` thật trong sản phẩm là **phân trang** (`--max-count`), nơi hàng cuối trang khai báo cha chưa nạp — và đó là ca HIST-01 gặp mỗi lần cuộn.
- **Ngân sách Core Value đã đo đủ ba khâu** (plan 02-03): `git log` 693ms + `parse_log` 82ms + `assign` **57ms** = **832ms** trên repo thật 100 007 commit, dưới mốc một giây, còn dư ~168ms. `assign` chỉ dùng 24% ngân sách 240ms của nó. **git chiếm 83% đường nóng** — nếu checkpoint #1 trượt thì chỗ phải sửa là bộ nạp (`Channel`/phân trang), không phải thuật toán lane.
- **Benchmark nhúng nguồn dữ liệu vào tên ca đo** (plan 02-03): `parse_log/repo-that/100000` vs `parse_log/tong-hop/100000`. Không có nhãn đó thì số tổng hợp bị đọc nhầm thành số của checkpoint #1 — và đã suýt xảy ra thật: `make-perf-repo.sh` ghi ra thư mục con `perf-100k/` chứ không biến `fixtures-perf/` thành repo, nên bản đầu của benchmark im lặng rơi về dữ liệu tổng hợp dù repo 100k nằm ngay đó.
- **Job `bench` riêng, không thêm bước vào job `rust`** (plan 02-03): job `rust` chạy ma trận ba nền tảng, một bước trơn sẽ chạy benchmark ba lần mỗi push cho ba con số không so được với nhau. Không đặt ngưỡng thất bại tự động: runner CI dùng chung tài nguyên và nhiễu tới hàng chục phần trăm, ngưỡng cứng chỉ sinh báo động giả rồi bị bỏ qua — tệ hơn là không có cổng nào.
- **🔴 `git for-each-ref` dùng escape `%1f`, KHÔNG `%x1f`** (plan 02-04, đính chính CONTEXT.md, plan 02-04 và `docs/01-research-competitors.md` mục 5.2 — cả ba sai): `%x1f` là escape của `git log` pretty-format. `for-each-ref` dùng ngôn ngữ `--format` của ref, nơi escape byte thô là `%<hai chữ số hex>`. Đo trên git 2.54: `git log --format='%H%x1f%P'` cho byte `037`; `for-each-ref --format='%(refname)%x1f...'` in ra **văn bản** `%x1f`. Hỏng trong im lặng ở mức tệ nhất — lệnh thoát 0, stderr rỗng, stdout có dữ liệu trông hợp lý, chỉ là không có dấu phân tách nên bộ phân tích bỏ **mọi** dòng và thanh bên rỗng hoàn toàn. **Cả 19 test đơn vị vẫn xanh** vì chúng tự chèn `0x1f`; chỉ test tích hợp chạy git thật bắt được.
- **`%(objectname)` của tag CÓ CHÚ THÍCH không phải mã commit** (plan 02-04): nó là mã của *đối tượng tag* (`git cat-file -t` trả `tag`), và mã đó không xuất hiện ở hàng nào trong `git log` — nhãn tag không neo được vào dòng nào và biến mất khỏi đồ thị. Phải thêm `%(*objectname)` (mã đã giải tham chiếu, rỗng với ref khác) và lấy nó khi khác rỗng. Không repo mẫu nào có tag nên test tự dựng repo tạm.
- **Gán lane chạy trên TOÀN BỘ lịch sử; phân trang chỉ là slice lúc trả về** (plan 02-04, theo ARCHITECTURE.md): lane của hàng 5000 là kết quả của mọi hàng 0..4999, nên `assign` trên một lát cắt cho đồ thị **trông hợp lý mà sai** — mọi đường nối bắt đầu lại từ lane 0, và trang **đầu** luôn đúng nên lỗi gần như không thấy trên repo nhỏ. `get_commit_page` vì vậy nạp `git log --all` một lần, tính lane cho hết, cache, rồi mọi trang sau là slice. Phân trang tiết kiệm **payload IPC**, không tiết kiệm bộ nhớ.
- **`MAX_CACHED_HISTORIES = 2`, suy từ ~67MB mỗi lịch sử 100k so với hạn RAM 150MB** (plan 02-04): 2 × 67MB = 134MB đã sát hạn; 3 × 67MB vượt hạn trước khi cộng baseline Tauri/WebView2 80–150MB. Loại bỏ **ưu tiên repo không phải repo đang hoạt động** — loại ngẫu nhiên có thể ném đúng repo người dùng đang xem và buộc nạp lại gần một giây. **Không** dùng crate `lru`: chỉ hai phần tử và quy tắc loại bỏ phụ thuộc *repo nào đang hiển thị*, thứ nằm ngoài cache và `lru` không diễn đạt được. `lru` vẫn dành cho cache **diff** của Phase 3 (khoá theo SHA, hàng nghìn phần tử). Chưa đo RSS thật — việc của checkpoint 02-07.
- **`close_repo` phải gọi `cache.invalidate`** (plan 02-04): không làm thì một lịch sử ~67MB nằm lại trong RAM cho repo người dùng đã đóng. `invalidate` xoá **cả** lịch sử **và** refs — ref trỏ vào mã commit, nên refs còn sống cạnh lịch sử đã bỏ cho nhãn trỏ vào hàng không tồn tại.
- **`get_commit_detail`: commit gốc so với cây rỗng `4b825dc6…`, merge so với `parents[0]`** (plan 02-04): commit gốc không có `<commit>^` nên `git diff <gốc>^ <gốc>` thất bại với `unknown revision` — một cài đặt chỉ nối `^` lỗi ở đúng commit đầu tiên của **mọi** repo. Dùng `git diff` cho cả hai (chỉ đổi vế trái) chứ **không** `git show`: `git show` mặc định không in gì cho merge, nên sẽ có hai hành vi tuỳ commit là gốc hay merge, và hai đường phân tích đầu ra.
- **`R`/`C` của `git diff --name-status -z` chiếm HAI đường dẫn** (plan 02-04, đo thật: `R100\0cu.txt\0moi.txt\0`): đọc như bản ghi một đường dẫn làm **mọi** bản ghi sau nó lệch một nấc — đường dẫn mới bị đọc thành trạng thái. Hỏng cả danh sách, không chỉ một dòng. `FileChange.status` vì vậy là `String` chứ không `char`: git luôn in điểm tương đồng (`R100`, `C75`) khi có `--find-renames`.
- **`HISTORY_TIMEOUT = 120s` riêng cho `git log --all`** (plan 02-04, T-02-14): `DEFAULT_TIMEOUT` 30s quá ngắn (repo 100k đã mất ~690ms khi bộ nhớ đệm ấm; repo hàng triệu commit trên đĩa nguội vượt 30s dễ dàng). **Không** mượn `NETWORK_TIMEOUT` dù cũng 120s: đây là lệnh cục bộ, và dùng chung hằng số nghĩa là ai chỉnh hạn giờ mạng sẽ vô tình đổi hành vi nạp lịch sử.
- **Command lịch sử nhận `repo_id` tường minh, KHÔNG `active_repo()`** (plan 02-04, PLAT-05 + T-02-12): `active_repo()` là lối tắt Phase 1, sai ngay khi có hai repo mở. `get_repo` cũng là cổng an toàn — chỉ trả repo **đã mở** qua `open_repository`, nên `repo_id` bịa cho `UnknownRepository` chứ không mở được thư mục tuỳ ý. Lời gọi `active_repo()` duy nhất còn lại chỉ nói cho cache biết repo nào không được loại bỏ.
- **Ba trục tìm kiếm lọc trong bộ nhớ, một trục hỏi git** (plan 02-04, HIST-10, ARCHITECTURE.md Anti-Pattern 4): thông điệp/tác giả/mã commit lọc trên lịch sử đã cache vì chạy git mỗi lần gõ một ký tự là chống chỉ định. Trục đường dẫn tệp buộc phải hỏi git (cache không giữ danh sách tệp), và lệnh đó **luôn** có `--` trước pathspec (T-02-11) để từ khoá trùng tên nhánh không bị hiểu thành revision.
- **Cổng `grep -c '<tên>' lib.rs` ≥ 1 là cổng VÔ DỤNG cho việc đăng ký command** (plan 02-04, đã kiểm bằng đột biến): thay dòng đăng ký thật bằng `// TODO: dang ky get_commit_page` thì cổng trả **1 và xanh** trong khi `invoke` sẽ thất bại lúc chạy. Cổng đúng là **test** tách khối `generate_handler![...]`, bỏ dòng chú thích, rồi đòi `commands::<tên>,`. Cùng hạng lỗi với cổng `parents[1]` của 02-03 và `grep -v '^#'` của CI. Một test tự dựng lệnh git cũng **không** chứng minh được mã sản phẩm dùng cờ đó — xoá `.arg("--")` khỏi `search_commits` mà test `--` vẫn xanh, nên cổng thật phải đọc thân hàm.
- **🔴 Đo hiệu năng PHẢI ở profile `--release`** (plan 02-04): cùng test, cùng repo 100 007 commit — debug cho `parse_log` **455ms** / `assign` **220ms** / đường nóng **1366ms** (tức *vượt* mốc một giây); release cho **67ms** / **59ms** / **793,9ms**. Chậm gần **7 lần** ở `parse_log`. Bảng benchmark của wave 3 đo bằng `criterion` vốn luôn dựng release, nên chỉ số release là chỉ số so sánh được. Đọc số debug rồi kết luận "trượt Core Value" hay "cache quá đắt" đều sai.
- **Không dùng `perl -0p` hay `head -N` để sửa tệp có tiếng Việt** (plan 02-04): một lần chèn test bằng `perl -0pe` biến toàn bộ tệp thành mojibake (`MÃ£ cÃ¢y rá»ng`) **và vẫn thoát 0**; `head -N` cũng làm hỏng một tệp khác. Dùng công cụ Edit hoặc heredoc `cat >>`.
- **Hai thao tác có tham số đi ngoài sổ đăng ký PLAT-04** (`onOpen`/`onForget` truyền bằng prop): `Command.run` có chữ ký `() => void | Promise<void>`, không nhận tham số. Mở rộng sổ đăng ký cho lệnh có tham số để dành cho v2 lúc làm bảng lệnh gõ nhanh. `repo.open` và `repo.close` vẫn đi qua sổ đăng ký như cũ.
- **`ROW_HEIGHT=28, LANE_WIDTH=14, GRAPH_PADDING_LEFT=8, MAX_VISIBLE_LANES=20` chốt ở plan 02-05, khớp phép tính hiển thị của `docs/04-phase2-degraded-graph.md`**: đồ thị và cột văn bản trong `CommitList.tsx` dựng từ CÙNG một mảng `virtualItems` của `@tanstack/react-virtual@3.14.13` (ghim chính xác) — đây là điều khiến lệch hàng bất khả thi về mặt cấu trúc, không phải "được sửa cho thẳng". `historyStore.ts` giữ mảng sparse cấp trước tới `total`; ô chưa nạp là `undefined`.
- **🔴 Hai cổng grep của plan 02-05 sai về cấu trúc, đã sửa** (xem `02-05-SUMMARY.md` mục "Plan sai ở đâu"): `grep -rc 'useVirtualizer' src/components/history/` không thể bằng 1 vì bất kỳ cài đặt đúng nào cũng khớp ít nhất 2 dòng (import + lời gọi) — sửa thành đếm điểm gọi `useVirtualizer(`. `grep -c 'devicePixelRatio' canvasRenderer.ts` trỏ sai tệp: theo đúng chỉ dẫn testability của chính plan, tham số đó phải tên `dpr` trong tệp này, còn `window.devicePixelRatio` chỉ đọc ở `GraphCanvas.tsx`.
- **`selectedCommitId` dùng `useState` trong `App.tsx` ở plan 02-05, chưa nâng lên `selectionStore`**: ARCHITECTURE.md Pattern 3 khuyên store riêng khi vùng chi tiết (02-06) không phải con của `App`. Cần quyết định lúc lập plan 02-06.
- **🔴 Checkpoint round 1 của plan 02-05 bị từ chối vì `.commit-row` co cột subject về 0px** (đo thật bằng Chromium/Playwright, không phải happy-dom): `minmax(0, 2fr)` không có sàn, và cột gutter đồ thị (tới 288px theo lane) cộng hai cột ngày/mã commit `max-content` (không co) ăn hết chỗ ở cửa sổ hẹp (900px, mức tối thiểu) kèm lane cao. **Không phải lỗi dữ liệu** — parser Rust, historyStore, CommitList, canvasRenderer đều đã kiểm chứng đúng riêng biệt trước khi tìm ra nguyên nhân CSS. Sửa: `minmax(120px, 2fr)` cho subject, `minmax(<n>px, max-content)` cho time/sha để chúng nhường chỗ được. Bài học lớn hơn: **happy-dom không tính layout CSS thật** (grid track sizing), nên lớp bug này chỉ lộ ra khi đo bằng trình duyệt thật — 103 test cũ đều xanh trong khi lỗi hiện rõ trên màn hình người dùng thật. Xem `02-05-SUMMARY.md` mục "Checkpoint round 1: REJECTED" cho quy trình điều tra đầy đủ (kể cả cách dùng Playwright cài tạm ở thư mục scratch để đo layout thật mà không thêm dependency vào dự án).
- Chi tiết đầy đủ các quyết định kỹ thuật: xem `.planning/PROJECT.md` mục Key Decisions và `.planning/research/SUMMARY.md` mục 3.

### Việc cần làm

- [ ] **Sau khi Phase 2 xong: index dự án bằng gitnexus.** Quyết định của chủ dự án
      2026-09-21. Lý do đợi: index bây giờ sẽ lỗi thời sau mỗi đợt, vì Phase 2 đang thêm
      parser, thuật toán lane, command và toàn bộ lớp giao diện. Xong Phase 2 thì codebase
      mới đủ hình dạng để đồ thị tri thức đáng giá. Chạy: `gitnexus analyze` ở gốc dự án.
      Ba công cụ hỗ trợ đã cài sẵn và đang hoạt động — rtk 0.42.1, gitnexus (7 skill + MCP),
      caveman (6 skill).
- [x] ~~**Phase 1, việc đầu tiên**: dựng toolchain Rust~~ — xong. rustc/cargo 1.98.1,
      toolchain `stable-x86_64-pc-windows-msvc`, workload C++ đã thêm vào VS Community 2022.
      `cargo build`, `cargo test` và `npx tauri build` đều chạy tới đích.
- [x] **Phase 2, chuẩn bị trước khi viết thuật toán lane**: xong ở plan 02-01 — chín repo mẫu tất định trong `target/fixtures/`, sinh bằng `bash scripts/fixtures/make-fixtures.sh`.
- [x] **Phase 2**: repo đo hiệu năng — xong ở plan 02-01, nhưng **sinh** chứ không tải Linux kernel (`bash scripts/fixtures/make-perf-repo.sh`): 100 007 commit, 3 182 merge, `.git` 32MB, sinh trong 24 giây.
- [ ] **Phase 6, trước khi chốt phạm vi**: chạy spike có giới hạn thời gian cho trình giải quyết xung đột trên CodeMirror 6.
- [ ] **Câu hỏi còn mở, quyết khi tới nơi**: nhiều repo mở theo thẻ (v1 hay v2 — kiến trúc đã sẵn sàng nhờ PLAT-05) · blame ở phase đánh bóng v1 hay v1.x (quyết ở cuối Phase 3, theo lịch thực tế) · giá trị thật của AI (xác thực với người dùng beta trước khi đầu tư quá 3–5 ngày công).

### Vướng mắc

**Đang chặn: checkpoint Task 4 của plan 02-05, VÒNG 2 (canvas hay SVG, checkpoint #2 của
ROADMAP).** Vòng 1 người dùng đã tự chạy app thật và **từ chối**: cột thông điệp commit co
về gần như trống ở cửa sổ hẹp + nhiều lane (đo thật bằng Chromium/Playwright:
`subjectWidth === 0px` tại 900px/maxLane~19 — lỗi CSS grid, KHÔNG phải lỗi parser/store/canvas,
cả ba đã kiểm chứng đúng riêng biệt), và đồ thị ít màu (bản chất dữ liệu, không phải lỗi —
xem `02-05-SUMMARY.md` mục "Checkpoint round 1: REJECTED" để đọc đầy đủ quy trình điều tra).
Đã sửa (`c2f6714` test, `cc270e4` fix: `.commit-subject` có sàn 120px, `CommitList` dùng
`ResizeObserver` thay vì đọc `clientHeight` trực tiếp trong thân render) và xác nhận lại bằng
đo Chromium thật — **chưa tự phê duyệt lại**. Cần người dùng chạy `npm run tauri:dev` (hoặc
`dev.cmd`), mở repo `git-plum`, làm lại sáu bước kiểm bằng mắt ở `02-05-SUMMARY.md` (chú ý
thêm: kéo panel giữa hẹp lại để xác nhận cột thông điệp không còn biến mất), rồi trả lời
"approved" hoặc nêu bước nào còn trượt.

Ngoài checkpoint trên, không có vướng mắc nào khác đang chặn. Toolchain đã đủ:

- ✅ rustc 1.98.1, cargo 1.98.1, toolchain `stable-x86_64-pc-windows-msvc`
- ✅ Visual Studio Community 2022 **kèm workload "Desktop development with C++"** — đã thêm,
  `link.exe` có, `cargo build` và `npx tauri build` chạy tới đích
- ✅ WebView2 153.0.4234.48 — có sẵn theo Windows 11
- ✅ Node 22.16, npm 10.9.2, git 2.54.0.windows.1

### Hai điều về môi trường, không chặn nhưng sẽ cắn lại

- **PATH của cửa sổ dòng lệnh mở trước lúc cài rustup không có `.cargo\bin`.** Windows không
  đẩy PATH mới vào tiến trình đang chạy. Triệu chứng: `npm run tauri:dev` báo
  `failed to run 'cargo metadata' ... program not found` trong khi `cargo` chạy bình thường ở
  chỗ khác. Cách chạy app không phụ thuộc điều này: bấm đúp `dev.cmd` ở gốc dự án — nó tự
  thêm `.cargo\bin` vào PATH của phiên. Khởi động lại Windows một lần là hết hẳn.
- **OneDrive giữ handle thư mục không đều.** `rm -rf` thất bại *một phần* nhưng vẫn trả mã 0,
  nên script chạy tiếp trên thư mục còn sót rồi vỡ ở chỗ khác với thông điệp vô nghĩa
  (`fatal: cannot lock ref 'HEAD'`). `scripts/fixtures/make-fixtures.sh` nay phát hiện và dừng
  ngay (commit `65a32bd`). Gặp lỗi này thì chạy lại thường là xong.

### Cổng dogfood đang chờ

| Phase | Cổng |
|---|---|
| 3 | Bản chỉ-đọc lịch sử+diff được tác giả dùng hằng ngày trên repo thật, gồm cả một repo bên thứ ba lớn và lộn xộn, **trước khi** bắt đầu việc thư mục làm việc |
| 4 | Một commit thật vào repo thật, chỉ bằng git-plum |
| 6 | Một luồng nhánh thật đầu-cuối chỉ bằng git-plum: tạo nhánh, commit, push, giải quyết một xung đột merge thật, hợp nhất xong |

### Validation checkpoint chưa giải quyết

| # | Phase | Câu hỏi |
|---|---|---|
| ~~7~~ | 1 | **ĐẠT MỘT PHẦN** — quyết định của chủ dự án 2026-09-21. Chi tiết bên dưới. |
| 1 | 2 | Đồ thị có mở dưới 1s và cuộn 60fps trên repo 50k–100k commit thật không? |
| 2 | 2 | Vẽ đồ thị bằng canvas hay lớp phủ SVG ở mức 100k dòng? (giữ sau interface) |
| 4 | 2 | JSON có chiếm phần lớn profile IPC cho dữ liệu lane không? (ship JSON trước rồi đo) |
| 3 | 3 | `@codemirror/merge` tự tính diff từ hai tài liệu đầy đủ — có đủ nhanh với tệp lớn không? |
| 5 | 6 | Công sức làm trình giải quyết xung đột trên stack CodeMirror 6 — con số 4 giờ của SourceGit không chuyển giao được |
| 6 | 7 | Đường dẫn import của `keyring` v4 (API đã tái cấu trúc mạnh so với v3) |

**Checkpoint #7 — đạt một phần.** Quyết định của chủ dự án ngày 2026-09-21, sau khi đo thật.

*Đã chứng minh:* lớp ghim môi trường hoạt động. Ép `LC_ALL`/`LANG`/`LC_MESSAGES` =
`vi_VN.UTF-8` ở tiến trình cha rồi chạy `cargo test` cho **23 test đỗ** — giống hệt lần chạy
bình thường. Số giống nhau chính là bằng chứng `apply_env_hardening` ghi đè được biến thừa
hưởng, chứ không phải máy tình cờ hợp.

*Chưa chứng minh được, và vì sao:* một giả định trong `CONTEXT.md` (G6) hoá ra **sai** — máy
phát triển **không** phải Windows tiếng Việt. Sáu nguồn độc lập đồng thuận `en-US`:
`Get-Culture`, `CurrentUICulture`, `Get-WinUserLanguageList`, `Get-WinSystemLocale`, registry
`HKCU\Control Panel\International`, và `systeminfo`. Thêm nữa, bản Git for Windows trên máy
này **không có tệp `.mo` nào** — kiểm bằng hành vi chứ không chỉ đếm tệp: chạy
`git rev-parse --show-toplevel` ngoài repo với `LC_ALL=vi_VN.UTF-8` vẫn ra
`fatal: not a git repository` **y nguyên tiếng Anh**. Trên máy này git không thể nói ngôn ngữ
khác dù có muốn, nên không phép kiểm nào ở đây chứng minh được bộ phân tích sống sót khi git
thật sự dịch đầu ra.

*Rủi ro còn lại:* thấp, nhưng có thật. Nếu người dùng chạy Git for Windows có gói ngôn ngữ,
và một chỗ nào đó trong mã quên đi qua lớp bọc, bộ phân tích sẽ vỡ trên máy họ mà không vỡ
trên máy phát triển. Cách giảm thiểu đã có sẵn: mọi tiến trình git **phải** đi qua
`GitCommand` trong `src-tauri/src/git/exec.rs` — đó là lý do PLAT-02 nói "nơi duy nhất".

*Khi nào đóng hẳn:* cần một máy hoặc máy ảo có ngôn ngữ hiển thị khác tiếng Anh **và** git
có catalog dịch. Đáng làm trước khi phát hành công khai (Phase 8), không chặn Phase 2.

---

## Session Continuity

**Việc tiếp theo:** hoàn thành checkpoint Task 4 của `02-05-PLAN.md`, **VÒNG 2** — vòng 1 đã
bị từ chối (cột subject co về 0px ở cửa sổ hẹp + nhiều lane), đã sửa và xác nhận lại bằng
Chromium/Playwright thật, xem `02-05-SUMMARY.md` mục "Checkpoint round 1: REJECTED". Chạy lại
sáu bước kiểm bằng mắt trên app thật, trả lời "approved" hoặc nêu bước trượt. Sau đó tiếp tục
`/gsd-plan-phase 2` cho các plan còn lại (02-06, 02-07).

Phase 1 đã đóng, 4/4 plan: 01-01 (PLAT-02 ghim cấu hình git), 01-02 (hạ tầng test giao diện),
01-03 (PLAT-07 danh sách repository gần đây), 01-04 (kịch bản QA + checkpoint locale).
Còn nợ phần QA thủ công — chi tiết ở mục Current Position bên trên.

**Còn lại của Phase 1 sau 01-03:** G3 (PLAT-09 chưa kiểm chứng kích thước vùng qua các lần chạy), G6 (locale không phải tiếng Anh), G7 (cửa sổ console ở bản release), và việc kiểm chứng thật PLAT-07 — mở lại ứng dụng và xác nhận danh sách gần đây sống sót. Ba thứ cuối chỉ kiểm được bằng cách chạy bản dựng thật, không kiểm được bằng test tự động.

**Nếu mất ngữ cảnh, đọc theo thứ tự:**

1. `.planning/ROADMAP.md` — cấu trúc 8 phase, tiêu chí thành công, ràng buộc từng phase
2. `.planning/REQUIREMENTS.md` — 59 requirement v1 và bảng truy vết
3. `.planning/PROJECT.md` — Core Value, ràng buộc, bảng Key Decisions (một số quyết định đã bị đảo ngược sau nghiên cứu)
4. `.planning/research/SUMMARY.md` — mục 3 (phiên bản đã chốt), mục 5 (ràng buộc định hình phase), mục 6 (validation checkpoint)

**Lưu ý:** Tài liệu dự án viết bằng tiếng Việt; mã định danh requirement, tên crate/package và tên lệnh git giữ nguyên dạng gốc.

---
*State initialized: 2026-09-21 after roadmap creation*
