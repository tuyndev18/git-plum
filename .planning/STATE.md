---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-09-21T08:16:18.000Z"
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 11
  completed_plans: 6
  percent: 55
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
| **Plan** | 2 / 7 xong (02-02 — hợp đồng dữ liệu và bộ phân tích `git log`) |
| **Status** | Wave 2 của Phase 2 xong; wave 3 (thuật toán gán lane) sẵn sàng chạy |
| **Progress** | Phase 1/8 · Phase 2 plan 2/7 |

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
| Plans completed | 6 |
| v1 requirements delivered | 7 / 59 đã kiểm chứng · 2 nữa có mã nhưng chưa kiểm thật (PLAT-07, PLAT-09) · PLAT-01 và REL-04 đạt ở mức yếu hơn ROADMAP |

| Plan | Thời lượng | Tasks | Files |
|---|---|---|---|
| Phase 1 P01 | 18min | 2 tasks | 2 files |
| Phase 1 P02 | 9min | 3 tasks | 6 files |
| Phase 1 P03 | 14min | 3 tasks | 6 files |
| Phase 2 P01 | 50min | 3 tasks | 9 files |
| Phase 2 P02 | 35min | 2 tasks | 8 files |

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
- **Hai thao tác có tham số đi ngoài sổ đăng ký PLAT-04** (`onOpen`/`onForget` truyền bằng prop): `Command.run` có chữ ký `() => void | Promise<void>`, không nhận tham số. Mở rộng sổ đăng ký cho lệnh có tham số để dành cho v2 lúc làm bảng lệnh gõ nhanh. `repo.open` và `repo.close` vẫn đi qua sổ đăng ký như cũ.
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

Không có vướng mắc nào đang chặn. Toolchain đã đủ:

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

**Việc tiếp theo:** `/gsd-plan-phase 2` — lịch sử và đồ thị nhánh.

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
