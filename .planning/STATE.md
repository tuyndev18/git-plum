---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-09-22T04:30:00.000Z"
progress:
  total_phases: 8
  completed_phases: 1
  total_plans: 16
  completed_plans: 16
  # % tính trên số PLAN ĐÃ LẬP (Phase 1 + 2 = 11, Phase 3 = 5 → 16), KHÔNG phải
  # toàn dự án: Phase 4–8 chưa lập plan nào.
  #
  # "16/16" là MÃ đã thực thi, không phải phase đã đóng. Ba cổng người-kiểm còn nợ:
  #   - Phase 2: hai checkpoint (02-06 vòng 2, 02-07 hiệu năng 100k)
  #   - Phase 3: checkpoint 11 bước của 03-04, checkpoint #3 của 03-01 (bị bỏ qua),
  #              và CỔNG THOÁT dogfood của 03-05 — xem Session Continuity.
  # Nên Phase 2 VÀ Phase 3 đều chưa đóng. completed_phases: 1 là Phase 1 (có nợ).
  percent: 100
---

# Project State: git-plum

**Last updated:** 2026-09-22

---

## Project Reference

**Core Value:** Đọc và hiểu lịch sử của một repository phải tức thì — đồ thị commit mở ra trong dưới một giây và cuộn mượt kể cả trên repo hàng chục nghìn commit.

**Current focus:** Phase 3 — Xem khác biệt (wave 4 mã xong; **checkpoint 11 bước chờ chủ dự án**; Phase 2 còn hai checkpoint nợ)

**Mode:** mvp (Vertical MVP) · **Granularity:** standard · **Parallelization:** enabled

---

## Current Position

### Phase 3 — Xem khác biệt (đang chạy)

| | |
|---|---|
| **Wave 1 (03-01)** | ✅ Mã xong. Spike đo A/B dựng xong; **checkpoint #3 CHƯA CHẠY** — chủ dự án chưa đo, nên quyết định A/B vẫn chưa có. Tìm và sửa một lỗi Phase 1: `GIT_EXTERNAL_DIFF=""` làm **mọi** lệnh git sinh bản vá thất bại trong im lặng (`f5c4c17`, đo thêm ở `43183d4`). |
| **Wave 2 (03-02)** | ✅ Xong, `autonomous` nên không phụ thuộc checkpoint #3. Backend diff đầy đủ: hợp đồng dữ liệu 5 dạng, `parse_patch` viết tay, `DiffCache` LRU 200 mục, `get_file_diff` với cổng DIFF-06 chạy **trước** `git diff`. **216 test Rust** (mốc 163) + **233 test frontend** (mốc 226); `clippy`/`typecheck`/`tauri:build` đều xanh. **9/9 mutation đã chạy**, trong đó **ba cổng vô dụng phải sửa rồi chạy lại**. Repo mẫu `target/fixtures/diff-cases` 26 commit, gồm ba fixture mà wave 3 phụ thuộc. |
| **Wave 3 (03-03)** | ✅ Xong, `autonomous` nên không phụ thuộc checkpoint #3. Diff mức **từ** lấy từ `git diff --word-diff=porcelain`: bộ phân tích riêng dựng lại dòng nguồn từ các đoạn từ (`parse_word_diff`), `DiffLine.spans` là khoảng **byte** vào `content`, và một lệnh git thứ hai **có điều kiện** trong `get_file_diff` với ba cổng bỏ qua. **257 test Rust** (mốc 216) + **236 test frontend** (mốc 233); `clippy`/`typecheck` xanh. **12/12 mutation đã chạy**: 9 đỏ, 3 ghi rõ là không kiểm được (hai trong đó vì **dữ liệu**, không vì cổng sai). Chi phí đo được: lệnh word-diff ≈ **32–39ms**, xấp xỉ bằng lệnh diff chính — bị chi phối bởi chi phí sinh tiến trình. |
| **Wave 4 (03-04)** | ⏸️ **Mã xong, CHECKPOINT 11 BƯỚC CHƯA CHẠY.** Trình xem diff thật — đây là wave đầu tiên có thứ nhìn được trên màn hình: tô màu cú pháp nạp lười theo phần mở rộng, hai chế độ hiển thị, nhảy khối có **nói ra khi hết** (không nhảy vòng im lặng), bật tắt khoảng trắng qua `Compartment`, word-level từ `spans` của wave 3, và năm dạng thông báo DIFF-06. Toàn bộ nằm sau **`interface DiffRenderer`** (tiền lệ `GraphRenderer` của Phase 2) với **cổng kiểm biên giới**: `@codemirror/merge` chỉ được nhập trong **một** tệp, nên đổi đường A→B là sửa một tệp. **365 test frontend** (mốc 236, **+129**); `cargo test` **257 + 1 ignored, không đổi**; `typecheck`/`build`/`clippy`/`tauri:build` đều xanh, MSI đã dựng. **13/13 mutation đã chạy**, trong đó **ba cổng xanh sai phải sửa** — một cổng xanh sai **hai lần liên tiếp vì hai nguyên nhân độc lập**. |
| **Wave 5 (03-05)** | ⏸️ **Mã xong, EXIT GATE DOGFOOD CHƯA CHẠY.** DIFF-05 xong: `parse_file_history` đọc `git log --follow --max-count=200 --name-status -z`, lần theo được đổi tên và **nói ra** chỗ đổi tên trên giao diện, chặn 200 phiên bản có cờ `truncated` hiện ra, timeout 30s riêng, **không cache** (lịch sử tệp phụ thuộc HEAD; watcher chỉ tới ở Phase 4). Bấm một phiên bản dùng `historyCommitOverride` nên **không** làm mất chỗ người dùng trên đồ thị. **284 test Rust + 2 doctest** (mốc 259, **+25**) + **435 test frontend** (mốc 397, **+38**); `clippy`/`typecheck`/`build`/`tauri build` xanh. **9/9 mutation đã chạy**, trong đó **một cổng xanh sai phải sửa** và **một cổng của plan là bất khả**. |
| **Phát hiện đo được (wave 5)** | 🔴 **Plan 03-05 sai hai chỗ, cả hai đo được.** (1) `<behavior>` nói *"`--follow` vẫn in mọi tệp của commit đó"* và đòi một phép lọc theo path — **sai**: git với pathspec **đã lọc** `--name-status` xuống đúng path đó (đếm được `grep -c other.txt` → **0** trên commit sửa hai tệp). Phép lọc đó sẽ là mã chết, và tệ hơn: nó cần biết "path đang theo", vốn **đổi** ở mỗi lần đổi tên, nên nó sẽ xoá đúng các phiên bản trước lần đổi tên mà `--follow` vừa mua được. (2) `<behavior>` nói merge commit *"xuất hiện với danh sách tệp rỗng"* — **sai**: nó **không xuất hiện** trong danh sách commit ở tất cả, kể cả merge đã thật sự giải quyết xung đột trong chính tệp đó.<br>🔴 **Bẫy plan không nêu:** phải tách theo **CẢ** `\x1f` **VÀ** `\0`. Plan chỉ nói về `\0\n`; nhưng bốn trường của `--format=` ngăn nhau bằng `\x1f`, nên tách chỉ theo `\0` làm cả header thành **một** trường và **mọi** bản ghi bị tính là méo — đo được `skipped_records = 9` trên buffer ba commit hợp lệ.<br>🔴 **Cổng đọc nguồn khớp chú thích do chính MUTATION sinh ra** — lần thứ **năm** dự án gặp lớp lỗi này, và lần đầu chú thích gây nhiễu không có sẵn trong mã mà do phép kiểm tạo ra. Nên một cổng đọc nguồn thô sống qua mọi lần review và chỉ hỏng **đúng lúc đang được kiểm**. Cổng **đã có** của 03-02 bắt được mutation này ngay — chỉ cổng mới sai.<br>🔴 **`--diff-merges=first-parent` trông như bản sửa nhưng PHÁ hai chặn trên** — đo trên repo thật `dau-tri-toan-hoc` (1140 commit, 39 merge): `.planning/STATE.md` **104 → 5715** bản ghi, `.planning/ROADMAP.md` **118 → 4655**. Cờ đó làm git in bản vá ra cùng luồng nên `--max-count` mất tác dụng — phá cả T-03-33 (DoS) lẫn T-03-34 (200 hàng DOM). Giới hạn merge được **nhận và ghim bằng test**, không lấp bằng cờ tệ hơn. |
| **Quyết định bố cục (cần xác nhận)** | `DiffViewer` chiếm vùng **`main`** khi có tệp đang chọn, **không** thêm panel thứ tư. Lý do: hai cột cần bề rộng, và thêm panel thứ tư chia cửa sổ thành bốn phần — làm ca hẹp (ca tệ nhất của chế độ hai cột) tệ hơn. Cách đã chọn **không đổi một dòng nào** trong `AppLayout.tsx`. Đây là **bước 1** của checkpoint. |
| **Phát hiện đo được** | 🔴 **Plan 03-04 sai một chỗ:** cổng bundle theo **tên chunk** là **bất khả** trên Vite 8 — Vite đặt tên chunk theo tên tệp entry của gói, và mọi `@codemirror/lang-*` có entry `dist/index.js`, nên năm chunk lang đều tên `dist-<băm>.js`. Đã thay bằng cổng neo vào **nội dung** chunk entry (`scripts/check-lang-chunks.mjs`), và **kiểm là đỏ được**. Chọn dấu hiệu mất **ba** lần thử: tên định danh và tên nhập đều bị bộ rút gọn đổi; chỉ **khoá object literal** (`stateData:`/`nodeNames:`/`tokenPrec:`) sống qua rút gọn.<br>🔴 **`await Promise.resolve()` KHÔNG flush render của React 19** — ba lần liên tiếp vẫn không đủ, nên test chống đua xanh **kể cả ở cài đặt đã bị đột biến**. Phải `act()`. Lớp lỗi "đo sai **thời điểm**", họ hàng bài học `measureFirstPaint` của Phase 2.<br>🔴 **Nhãn đọc TRẠNG THÁI store che mất lỗi đua** — `diff-path` render từ `selectedFile` nên nó hiện tệp *đang chọn* bất kể phản hồi nào đã ghi đè nội dung. Không chỉ là cổng yếu: người dùng thật sẽ thấy tiêu đề `b.ts` trên nội dung `a.ts` mà không có cách nào biết. Sửa ở **mã sản phẩm**.<br>🔴 **Plan 03-02 sai một chỗ:** `git diff --name-status` **mang pathspec** làm git báo `A` thay vì `R077` cho tệp đổi tên. Đã sửa và ghim bằng ba test.<br>🔴 **Plan 03-03 sai một chỗ:** biên từ **mặc định** của git **mất** khoảng trắng ngăn cách (`alpha beta` → `alpha` dựng lại thành `alphabeta`; `dong hai` → `dong hai da sua` để lại dấu cách thừa ở phía cũ), làm bất biến "dựng lại bằng từng byte với `parse_patch`" **bất khả**. Ca thứ hai nằm ngay trong `text-simple.txt`. Đã sửa sang `--word-diff-regex=[^[:space:]]+\|[[:space:]]+` — vẫn an toàn với UTF-8 vì nó khớp theo **vệt** byte, không theo byte lẻ như `.`. |
| **Tiếp theo** | ⛔ **CỔNG THOÁT PHASE 3 — chủ dự án dùng thật vài ngày.** Bản release đã dựng: `src-tauri/target/release/git-plum.exe`, **2026-09-22 11:38:59 +0700**, commit `0845b24`. Khung ghi chép ở `docs/08-phase3-dogfood.md`. **Không còn wave nào để hoãn việc sang** — đây là cổng cuối trước Phase 4, vốn dựng ngay trên trình xem diff này (WORK-01). Checkpoint 11 bước của 03-04 cũng vẫn chưa chạy và có thể gộp vào cùng lần dùng thật. |
| **DIFF-05** | ✅ **Có mã** (wave 5). `parse_file_history` + `get_file_history` + `FileHistory.tsx`; 15 test đơn vị, **10 test tích hợp chạy git thật**, 25 test giao diện, 6 test IPC, 7 test CSS. Mức requirement: **"Có mã, chưa kiểm"** — chưa ai thấy danh sách phiên bản trên màn hình.<br>⚠️ **Giới hạn đã biết, KHÔNG phải ô chưa kiểm:** merge commit không xuất hiện trong lịch sử tệp. Đo được, có test ghim, và cờ trông-như-bản-sửa đã đo là **tệ hơn** (xem "Phát hiện đo được (wave 5)"). Việc cho Phase 4 nếu cổng thoát cho thấy nó đáng: một lệnh **riêng** cho merge, không một cờ thêm vào lệnh lịch sử tệp. |
| **Requirement** | `VERIFICATION.md` của phase đã cập nhật: **cả bảy ô (DIFF-01..06 + word-level) ở mức "Có mã, chưa kiểm"**; hiệu năng mở diff ở mức **"Chưa đo"**. **Đạt: 0.** Không một requirement nào của Phase 3 có bằng chứng từ mắt người, vì cả ba cổng dùng-mắt đều chưa chạy hoặc bị bỏ qua (checkpoint #3 bỏ qua, checkpoint 11 bước chưa chạy, cổng thoát chưa chạy). happy-dom không tính layout CSS nên 435 test xanh **không** là bằng chứng cho tiêu chí nói về thứ nhìn thấy được. Chế độ **hai cột vẫn chưa bao giờ được render trong một test nào**. |
| **🔴 Cây làm việc** | Bốn tệp có thay đổi **không thuộc plan nào đã chạy**, chưa commit: `src/App.tsx`, `src/components/RefSidebar.tsx`, `src/components/history/CommitList.tsx`, `src/styles/app.css` (hai hunk đầu). Nội dung là việc **Phase 2**: bấm nhánh/tag ở thanh bên thì cuộn đồ thị tới commit (`handleSelectRef`, `scrollToCommit`). Executor của 03-05 **không** commit và **không** stash chúng (stash có thể làm mất việc của người khác); nó stage từng tệp một và với `app.css` thì tách diff để chỉ đưa hunk của mình vào commit. **Chủ dự án cần quyết định:** việc đang làm dở, hay cần một commit riêng. Đây cũng là lý do mốc frontend là 397 chứ không 396. |

### Phase 2 — nợ cũ

| | |
|---|---|
| **Phase** | 2 — Lịch sử và đồ thị nhánh |
| **Plan** | **7 / 7 đã thực thi**; hai checkpoint người kiểm CHƯA CHẠY nên 7/11 requirement còn Pending |
| **Status** | Mã của cả 7 plan đã xong: **212 test frontend + 149 test Rust (+1 ignored)** *(số lúc đóng Phase 2; nay là 435 frontend + 284 Rust sau Phase 3)*, `typecheck`/`build`/`tauri build --release` đều xanh. Còn **hai việc nợ, cả hai cần người chạy app**: (1) **checkpoint 12 bước của 02-06** — vòng 1 bị từ chối với BA nguyên nhân độc lập, cả ba đã sửa (`7596e63` containment chiều cao Grid; `5ac43ef`/`970e31c` nhãn ref và chữ message tranh cùng cột grid, tái hiện bằng số đo Segoe UI thật 258px vs 144px → chữ hiện 0%; `60a0caa` cạnh `outEdges` vẽ tràn nửa hàng), chờ vòng 2; (2) **checkpoint #1 của 02-07** — đo hiệu năng repo 100k. Phần **đo được bằng máy đã chạy xong** 2026-09-22 (`9e5d113`): `cargo bench --bench graph` trên repo 100 007 commit **thật** cho `parse_log` **87.9ms** và `assign_lanes` **64.4ms** (trung bình 100 mẫu; cao hơn 20–39% so với hai lần đo một-phát 749.7ms/762.2ms, vì criterion tính cả phương sai — không phải hồi quy). Tổng đường nóng Rust bảo thủ: **787.0ms** / mốc 1000ms, `git log` chiếm **80.5%**. Nhưng ba phép đo nói về thứ **người dùng nhìn thấy** (vẽ lần đầu, FPS cuộn, thẳng hàng ở 100k) vẫn thiếu, nên HIST-05 và tiêu chí thành công 1 **vẫn Pending** — số Rust không đóng được checkpoint này. Checkpoint #4 **hoãn** (giữ JSON) vì cần phần dư IPC từ checkpoint #1. Xem `VERIFICATION.md` cho bằng chứng từng requirement. |
| **Progress** | Phase 1/8 · Phase 2: 7/7 plan đã thực thi, 2 checkpoint người kiểm còn nợ |

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
| Phases completed | 1 / 8 (có nợ) — Phase 3 có đủ mã cho DIFF-01..06 nhưng **0 requirement nào có bằng chứng từ mắt người** |
| Plans completed | 16 (mã đã thực thi; **Phase 2 VÀ Phase 3 đều chưa đóng** — 5 cổng người-kiểm còn nợ, xem Session Continuity) |
| v1 requirements delivered | 12 / 60 đã kiểm chứng (thêm HIST-01, HIST-02, HIST-04, HIST-11 ở plan 02-05, checkpoint #2 người dùng chấp thuận) · 2 nữa có mã nhưng chưa kiểm thật (PLAT-07, PLAT-09) · PLAT-01 và REL-04 đạt ở mức yếu hơn ROADMAP |

| Plan | Thời lượng | Tasks | Files |
|---|---|---|---|
| Phase 1 P01 | 18min | 2 tasks | 2 files |
| Phase 1 P02 | 9min | 3 tasks | 6 files |
| Phase 1 P03 | 14min | 3 tasks | 6 files |
| Phase 2 P01 | 50min | 3 tasks | 9 files |
| Phase 2 P02 | 35min | 2 tasks | 8 files |
| Phase 2 P03 | 65min | 3 tasks | 14 files |
| Phase 2 P04 | 85min | 3 tasks | 14 files |
| Phase 2 P05 | ~110min (Task 1-3) + ~90min (checkpoint round 1: điều tra + sửa) | 4/4 tasks | 16 files |
| Phase 2 P06 | ~95min (Task 1-3) + ~45min (round 1 fix A: chiều cao) + ~40min (round 1 fix B: cột grid riêng cho nhãn) + ~25min (round 1 fix C: cạnh đồ thị) — chờ vòng 2 | 3/4 tasks | 18 files |
| Phase 3 P01 | ~50min | 2/3 tasks (Task 3 là checkpoint chờ người kiểm) | 13 files |
| Phase 3 P02 | ~85min | 3/3 tasks | 15 files |
| Phase 3 P03 | ~75min | 3/3 tasks | 13 files |
| Phase 3 P04 | ~150min | 2/3 tasks (Task 3 là checkpoint 11 bước, chờ người kiểm) | 16 files |
| Phase 3 P05 | ~75min | 2/3 tasks (Task 3 là **EXIT GATE dogfood**, chờ chủ dự án dùng thật) | 11 files |

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
- **PLAT-02 (plan 01-01)**: `GIT_CONFIG_PARAMETERS` ghim đủ `log.showSignature`, `diff.noprefix`, `format.coverLetter` qua hằng `PINNED_GIT_CONFIG` trong `src-tauri/src/git/exec.rs`.
- **🔴 Trình diff ngoài: dùng cờ `--no-ext-diff`, KHÔNG biến môi trường rỗng** (sửa ở plan 03-01, commit `f5c4c17`; đính chính quyết định của 01-01). `GIT_EXTERNAL_DIFF=""` **không** vô hiệu hoá trình diff ngoài — git spawn chương trình tên rỗng và chết với `error: cannot spawn : No such file or directory`, làm **mọi lệnh git sinh bản vá thoát 128 với stdout rỗng, im lặng**. Ghim `diff.external=` rỗng qua config có cùng lỗi (đã đo). Phase 2 không phát hiện vì chỉ dùng `--name-status`, lệnh không gọi trình diff. Nay `env_remove("GIT_EXTERNAL_DIFF")` cộng chèn `--no-ext-diff` tập trung trong `GitCommand::run()` cho `diff`/`show`/`log`/`diff-tree` (cả `log` và `show` vì hai lệnh đó in được bản vá). Cờ phải đứng **sau** lệnh con. Bài học: một ràng buộc "đã ghim rồi" mà chưa có test đọc ngược kết quả **từ tiến trình con** thì chưa được ghim — đọc mã thì ý định và lỗi trông giống nhau.
- **🔴 `git diff --name-status` KHÔNG được mang pathspec khi cần phát hiện đổi tên** (đo ở plan 03-02, commit `a52a018`). Git ghép **tập** tệp bị xoá với **tập** tệp được thêm để nhận ra đổi tên; một pathspec lọc đường dẫn cũ ra **trước khi** phép ghép chạy, nên git báo `A` thay vì `R077`. Cùng lý do, `git diff --unified=3 -- <tên mới>` in `new file mode` với **toàn bộ tệp là dòng thêm** thay vì một hunk sửa vài dòng — người dùng bấm vào tệp đổi tên sẽ thấy cả tệp sáng xanh. Cách đúng: `--name-status` chạy **không** pathspec rồi tự lọc bản ghi; lệnh sinh bản vá truyền **cả hai** đường dẫn. Bài học cùng họ với `%x1f` của 02-04: một cờ trông vô hại (`-- <path>`) đổi ngữ nghĩa của một cờ khác (`--find-renames`) mà không báo lỗi gì.
- **Ngưỡng DIFF-06 là 5 MB mỗi phía** (`MAX_DIFF_BLOB_BYTES`), dùng `max(cũ, mới)` để tệp lớn **bị xoá** cũng bị chặn. Ngưỡng của **trình xem**, không phải của máy: 5 MB ≈ 100 nghìn dòng, ngoài mọi ngân sách của checkpoint #3; cũng là mốc GitHub từ chối hiện diff. Con số 200 MB trong ROADMAP là **ví dụ tiêu chí**, không phải ngưỡng.
- **Nhận biết con trỏ Git LFS bằng NỘI DUNG blob, không `git check-attr`** (câu 3 CONTEXT.md, đo lại ở 03-02). `check-attr -z filter` trả `unspecified` cho một con trỏ LFS **thật** khi repo không có `.gitattributes` — nó trả lời "repo có **cấu hình** lfs cho path này không", không trả lời "blob này **có phải** con trỏ không". Chỉ đọc blob khi `< 1 KB` (con trỏ luôn ~130 byte) nên cổng gần như miễn phí.
- **Cache diff khoá `(repo_id, sha, path)` dạng struct, không bao giờ vô hiệu hoá theo thời gian.** Diff của commit lịch sử là **bất biến**. Nếu sau này thêm `-w` (bỏ qua khoảng trắng khi **tính** diff) thì cờ đó **phải** vào khoá vì nó đổi dữ liệu; chế độ hợp nhất/hai cột và hiện ký tự khoảng trắng thì **không** — chúng chỉ đổi cách vẽ.
- **`#[serde(rename_all)]` ở cấp enum KHÔNG đổi tên trường bên trong biến thể** (đo ở 03-02, commit `5d51f65`) — nó chỉ đổi tên biến thể. Mỗi biến thể mang dữ liệu phải lặp lại thuộc tính đó, nếu không JSON ra `old_size` trong khi TS đọc `oldSize`, và **không bên nào lỗi biên dịch**.
- **🔴 `Span` là chỉ số BYTE, CodeMirror đánh chỉ số theo đơn vị UTF-16** (wave 3 → wave 4). Phép chuyển sống ở `spanToUtf16` trong `src/lib/diff-render/decorations.ts`, gọi **một lần** ngay lúc dữ liệu vào tầng vẽ. Số đo đã ghim: `'xéy dỏng TEST'` → `TEST` ở **byte 12** nhưng **UTF-16 index 9**; `'x🙂y CHANGED'` → byte 7 nhưng UTF-16 5. Kiểu trả về đổi tên trường (`Utf16Span { from, to }` vs `Span { start, end }`) để việc truyền lẫn hai hệ thành lỗi **biên dịch**, không phải lỗi hiển thị im lặng. `TextEncoder`, **không** `Buffer` — mã sản phẩm browser-only. Mutation bỏ phép chuyển → **4 test đỏ**, và test **phải** dùng nội dung ngoài ASCII (ca ASCII thuần xanh ở cả hai cài đặt, nên nó được giữ kèm comment nói rõ nó **không** phải cổng).
- **Trình xem diff nằm sau `interface DiffRenderer`, và `@codemirror/merge` chỉ được nhập trong MỘT tệp** (wave 4). Điều kiện của `docs/09-phase3-diff-decision.md` mục 8, vì đường A được chọn **không có số đo** (checkpoint #3 bỏ qua). Đổi A→B = viết `hunkRenderer.ts` rồi đổi **một** dòng import trong `DiffViewer.tsx`. Cổng: `src/lib/diff-render/interface-boundary.test.ts`, tương đương `grep getContext GraphCanvas.tsx` → 0 của Phase 2 nhưng ở dạng test thật (plan cấm cổng grep `== 0` trên tệp có doc comment tiếng Việt dày — và `DiffViewer.tsx` **có** nhắc `@codemirror/merge` trong comment, nên grep sẽ khớp nhầm chính đoạn văn đó). Danh sách cho-phép còn có test riêng ghim rằng hai tệp trong đó chỉ nhập `import type`.
- **Chế độ hợp nhất KHÔNG để CodeMirror tính lại diff**, chỉ chế độ hai cột dùng `MergeView` (wave 4). Hệ quả: nguy cơ hiệu năng của checkpoint #3 áp cho **một** chế độ, và chế độ đó **không** phải mặc định. Đường thoát của người dùng nếu chậm là chuyển về hợp nhất.
- **🔴 Cổng bundle không được neo vào TÊN chunk** (wave 4). Vite 8 đặt tên chunk theo tên tệp entry của gói; mọi `@codemirror/lang-*` có entry `dist/index.js` nên năm chunk lang đều tên `dist-<băm>.js`. Cổng đúng neo vào **nội dung chunk entry** (`scripts/check-lang-chunks.mjs`). Và dấu hiệu phải là **khoá object literal** (`stateData:`/`nodeNames:`/`tokenPrec:`) — tên định danh (`javascriptLanguage`) và tên nhập (`LRParser.deserialize(`) **đều** bị bộ rút gọn đổi, mất hai lần thử mới ra.
- **🔴 Nhãn hiển thị phải đọc DỮ LIỆU đã về, không TRẠNG THÁI store** (wave 4). `diff-path` render từ `selectedFile` làm nó hiện tệp *đang chọn* bất kể phản hồi nào đã ghi đè nội dung — tức nó **che** đúng lỗi mà cổng chống đua (T-03-31) tồn tại để chặn. Không chỉ là cổng yếu: người dùng thật thấy tiêu đề `b.ts` trên nội dung `a.ts` mà không có cách nào biết.
- **🔴 `await Promise.resolve()` KHÔNG flush render của React 19** (wave 4). Ba lần liên tiếp vẫn không đủ cho một `setState` gọi từ callback promise, nên một test có thể xanh **kể cả ở cài đặt đã bị đột biến** vì nó đo trước khi DOM cập nhật. Phải `act(async () => { await new Promise((r) => setTimeout(r, 0)) })`. Lớp lỗi "đo sai **thời điểm**", họ hàng với bài học `measureFirstPaint` của Phase 2.
- **`MIN_SPLIT_WIDTH = 720` đo bằng `ResizeObserver` trên panel, KHÔNG `window.innerWidth`** (wave 4). Panel kéo được nên bề rộng cửa sổ không nói gì về bề rộng panel. Hằng số sống hai phía (`DiffViewer.tsx` + `--min-split-width` trong `app.css`) và **có test ghim**, theo tiền lệ `REF_COL_WIDTH`. Dưới ngưỡng → tự chuyển hợp nhất **kèm một dòng nói vì sao**; im lặng hiện hai cột 80px mỗi cột tệ hơn.
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
- **`selectionStore` nâng cấp thật ở plan 02-06** (đóng câu hỏi dòng trên): `selectedByRepo[repoId]`, chỉ giữ id, không giữ `Commit`/`files`. `CommitDetail` và `RefSidebar` đều đọc/ghi độc lập, không component nào là con của component kia trong `AppLayout` — đúng như Pattern 3 mô tả. Test khẳng định bằng cách soát khoá của `getState()` (chặn được đột biến thêm trường dữ liệu, đã kiểm mutation thật).
- **Nhãn ref (`RefBadges`, HIST-06) đặt INLINE trong `.commit-subject` hiện có, KHÔNG phải cột CSS Grid mới** (plan 02-06, học trực tiếp từ bài học checkpoint round 1 của 02-05): thêm một cột `max-content`/`minmax` mới sẽ lặp lại đúng phép tính co cột đã gây lỗi trước đó (nhiều cột không co cạnh tranh không gian ở cửa sổ hẹp + nhiều lane). Đặt nhãn làm nội dung inline trước subject nghĩa là chúng dùng chung ngân sách bề rộng với subject và bị cắt cùng nhau bằng `text-overflow: ellipsis` sẵn có — không có cột nào mới để co về 0px. `white-space: nowrap` giữ chiều cao hàng cố định bất kể số nhãn (HIST-04).
- **`CommitList.tsx` là `forwardRef<CommitListHandle>` từ plan 02-06**: `scrollToIndex` lộ ra qua `useImperativeHandle`, dùng bởi `CommitSearch` (HIST-10) — tránh tạo `useVirtualizer` thứ hai. Vẫn đúng MỘT tệp gọi `useVirtualizer(` trong toàn `src/`, đã kiểm bằng `grep -rln 'useVirtualizer(' src/`.
- **`CommitDetail` cache `CommitDetail` payload theo `commitId` trong `Map` cấp module, chặn 200 mục** (plan 02-06): diff/danh sách tệp của commit lịch sử bất biến nên cache không bao giờ cần vô hiệu hoá. Có `__resetCommitDetailCacheForTest()` chỉ dùng trong test — cache cấp module sống qua mount/unmount làm rò rỉ trạng thái giữa các test dùng chung `commitId`.
- **🔴 Đột biến kiểm ra assertion debounce ban đầu của `CommitSearch` không phân biệt được 250ms với 0ms** (plan 02-06): `advanceTimersByTimeAsync(250)` đi qua cả hai mốc. Sửa test thành kiểm chưa gọi ở 249ms rồi mới advance nốt 1ms — cùng lớp bài học với đột biến 6 của 02-05 (test "xanh" không chứng minh được hành vi nếu assertion không đủ chặt).
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

**🔴 Hai lỗi tìm được 2026-09-22 ngoài mọi cổng, cả hai CHƯA SỬA, cả hai là nợ phase cũ:**

1. **Hạ cap lane 20→13 làm chồng cột tăng 28 lần** (`9e5d113`, chi tiết ở
   `docs/04-phase2-degraded-graph.md` mục 2.3). Đo bằng `src-tauri/src/bin/lanedist.rs`
   trên repo 100 007 commit thật: **19 995 hàng = 19,99 %** có `lane >= 13`, nên `laneX`
   clamp chúng về cột 12 — tám lane khác nhau vẽ đè cùng một cột, và **không có chỉ báo
   nào** (chỉ báo `+N` chỉ dành cho `truncated_parents`, 0,67 %). Ở cap 20 tỉ lệ là
   **0,71 %**. Phát hiện khi chạy checkpoint #1 và thấy benchmark in `lane lớn nhất 20`
   trong khi cap là 13. **Không sửa**: cap 13 suy từ `LANE_WIDTH` 22 px đo từ ảnh tham
   chiếu mà chủ dự án yêu cầu khớp, nên chọn giữa "13 lane thoáng + 20 % chồng" và
   "20 lane chật + 0,7 % chồng" là quyết định **thiết kế**, không phải lỗi có đáp án
   đúng. Ba đường đi đã ghi ở mục 2.3. `docs/04` mục 2 cũng đã lạc hậu từ `544ae5f`
   (còn kể cap 20 và `LANE_WIDTH` 14 px) — đã sửa trong cùng commit.

2. **Repo bị git từ chối báo sai "Không phải một repository git"** (`c9f3e09`, ca kiểm
   KB-4b ở `docs/03-phase1-qa-windows.md`). Nhiều repo trong `D:/MyCompanyProjects/`
   thuộc SID Windows khác → `git rev-parse` exit 128 kèm `dubious ownership`.
   `open_repository` (`src-tauri/src/commands/repo.rs:41`) rẽ **mọi** `!is_success()`
   sang `GitError::NotARepository`, biến thể chỉ mang `path` **không** mang `stderr`,
   nên câu `git config --global --add safe.directory …` — thứ duy nhất giúp sửa —
   không tới giao diện. Nợ **PLAT-10** của Phase 1. KB-4 hiện có **không** bắt được vì
   nó chỉ kiểm thư mục không-phải-repo và coi thông báo đó là đúng.

**Đang chờ: checkpoint Task 4 của plan 02-06 (`gate=blocking`) — VÒNG 1 BỊ TỪ CHỐI, đã sửa,
chờ vòng 2.** Người dùng tự chạy app thật (WebView2/Windows), báo ba lỗi cụ thể ở bước 6 kèm
ảnh chụp thật: (1) hàng có nhiều badge ref cao hơn hàng thường, (2) chấm đồ thị lệch khỏi tâm
hàng đó (hệ quả trực tiếp của lỗi 1 — canvas vẽ theo `ROW_HEIGHT` cố định trong khi DOM row
thật đã cao hơn), (3) badge chồng/tràn khỏi cột thay vì co gọn hiện `+N`.

**Ba lỗi đó có HAI nguyên nhân độc lập, cả hai đã sửa:**

- **Nguyên nhân A (chiều cao) — `7596e63`.** `.commit-row` là container Grid, grid item mặc
  định `min-height: auto` (không phải `0`), nên nội dung con có thể ép track Grid cao lên vượt
  `height: 28px` dù `.commit-subject` bên trong có `overflow: hidden`. Sửa bằng containment
  cứng: `overflow: hidden; min-height: 0;` trên `.commit-row`, `max-height` + `overflow:
  hidden` trên `.ref-badges`/`.ref-badge`. **Không tái hiện được bằng số đo** (ghi trung thực)
  nhưng đúng theo đặc tả CSS. 3 test hồi quy trong `app.css.test.ts`, kiểm mutation cả ba.
- **🔴 Nguyên nhân B (badge ăn hết chỗ của chữ) — `5ac43ef`, `970e31c`. ĐÂY LÀ NGUYÊN NHÂN
  CHÍNH.** Tìm ra ở vòng điều tra 2 sau khi coordinator đọc mã và chỉ ra
  `CommitList.tsx:164-165`: `RefBadges` render **bên trong** `.commit-subject` nên badge và chữ
  message cạnh tranh **cùng một cột grid**. **Đã tái hiện được bằng số đo** với Segoe UI THẬT
  (nạp từ `C:/Windows/Fonts` qua `@font-face`) ở đúng bề rộng vùng `main` thật (52% cửa sổ):
  nhóm 4 badge chiếm **258px** trong khi cột subject chỉ còn 332px (cửa sổ 1440px) → 144px
  (cửa sổ 900px), chữ message hiển thị **27% → 0%**. Hàng không badge vẫn hiện chữ bình thường
  — khớp chính xác ảnh người dùng. Dấu `.` người dùng thấy **không phải** message thật, mà là
  phần đuôi ellipsis còn sót (ngược với 02-05 round 1, nơi `.` là message thật). Sửa bằng
  **quyết định kiến trúc**: badge có **cột grid riêng** (`.commit-ref-cell`, cột 2,
  `minmax(0, max-content)`), khớp tham chiếu GitKraken (`docs/screenshots/main-1.png`,
  `main-4.png` — cột `BRANCH / TAG` tách biệt khỏi `COMMIT MESSAGE`). 3 test hồi quy **cấu
  trúc** (đếm cột, sàn 0 của cột badge, vị trí `RefBadges` trong JSX), kiểm mutation cả hai
  chiều. Sau sửa: chữ message hiện 66% (1440px) / 44% (900px, giữ sàn 120px).

- **🔴 Nguyên nhân C (cạnh đồ thị vẽ tràn sang hàng kế tiếp) — `60a0caa`.** Người dùng báo tiếp
  sau khi A và B đã sửa: đồ thị "rất khó đọc và bị vỡ". `outEdges` trong `canvasRenderer.ts` vẽ
  từ tâm hàng tới `y + 1.5 * ROW_HEIGHT` — **nửa hàng vượt quá ô của chính nó**, đè vào ô hàng
  kế tiếp ở lane khác, nên mọi hàng có rẽ nhánh vẽ một đường xuyên qua hàng bên cạnh. Với
  virtualizer chỉ dựng hàng đang thấy, đường tràn còn có thể chạy xuống hàng **chưa được dựng**.
  Sửa: mỗi hàng vẽ **strictly** trong `[y, y + ROW_HEIGHT]` — `outEdges` đi từ tâm tới mép dưới,
  nửa trên của hàng kế tiếp do `passthrough` của hàng đó vẽ, hai nửa gặp ở mép chung nên liền
  mạch. **Không test nào trong 179 test cũ bắt được** vì hai test `outEdges` chỉ khẳng định toạ
  độ **X** — không một test nào đọc toạ độ **Y**. Đã thêm 3 test containment trục Y, mutation đỏ
  cả ba. Bài học: test hình học chỉ kiểm một trục là test một nửa.

**Vì sao vòng điều tra 1 cho âm tính giả:** đo trong Chromium **không có Segoe UI** (rơi về
phông thay thế hẹp hơn) **và** ở full viewport thay vì vùng `main` 52% thật — hai sai số cộng
dồn. Bài học: đo layout phải khớp CẢ phông CẢ bề rộng vùng chứa thật; và test đọc chuỗi CSS
không thể bắt lỗi *quan hệ cấu trúc* (badge dùng chung cột với message), phải test cấu trúc.

Xây lại `tauri build --debug --no-bundle` xong sau CẢ HAI fix. **KHÔNG tự phê duyệt lại** —
chờ người dùng chạy lại 12 bước, đặc biệt **bước 5** (chữ message có hiện đủ trên hàng có
nhãn), bước 6, bước 1-2, và kéo panel giữa qua nhiều bề rộng. Xem `02-06-SUMMARY.md` mục
"Checkpoint round 1: REJECTED" cho điều tra đầy đủ.

**Yêu cầu riêng của người dùng về đổi theme/icon/font giống GitKraken hơn** đã ghi vào
`PROJECT.md` (`5417400`) nhưng người dùng nói "sau này sửa sau" — KHÔNG làm trong lần sửa
checkpoint round 1 này, chỉ sửa đúng ba lỗi layout CSS nêu trên. Xác nhận qua `git show
7596e63`, `git show 5ac43ef`, `git show 970e31c` không có dòng nào đổi màu/font/icon.

Checkpoint #2 (canvas hay SVG, plan 02-05) đã **đóng**:
vòng 1 người dùng từ chối vì cột thông điệp commit co về gần như trống ở cửa sổ hẹp + nhiều
lane (đo thật bằng Chromium/Playwright: `subjectWidth === 0px` tại 900px/maxLane~19 — lỗi CSS
grid, không phải lỗi parser/store/canvas), đã sửa (`c2f6714` test, `cc270e4` fix) và xác nhận
lại bằng đo Chromium thật; vòng 2 người dùng tự chạy app thật, kiểm lại, trả lời "approved".
Chốt **canvas**. Chi tiết đầy đủ ở `02-05-SUMMARY.md` mục "Checkpoint round 1: REJECTED" /
"Checkpoint round 2: APPROVED".

Toolchain đã đủ:

- ✅ rustc 1.98.1, cargo 1.98.1, toolchain `stable-x86_64-pc-windows-msvc`
- ✅ Visual Studio Community 2022 **kèm workload "Desktop development with C++"** — đã thêm,
  `link.exe` có, `cargo build` và `npx tauri build` chạy tới đích
- ✅ WebView2 153.0.4234.48 — có sẵn theo Windows 11
- ✅ Node 22.16, npm 10.9.2, git 2.54.0.windows.1

### Quick Tasks Completed

| # | Description | Date | Commit | Directory |
|---|-------------|------|--------|-----------|
| 260923-lhl | Reskin giao diện theo design system Linear (tối mặc định, lavender #5e6ad2), font Inter + cv11/ss01/ss03 | 2026-09-23 | dc9c4ac | [260923-lhl-reskin-giao-dien-theo-design-system-line](./quick/260923-lhl-reskin-giao-dien-theo-design-system-line/) |
| fast | Màu diff preview dễ đọc trên nền tối: classHighlighter + --syn-*, word-level theo màu dòng, gutter bỏ opacity | 2026-09-23 | cffdfa5 | — |
| fast | Màu cú pháp diff theo VS Code Dark+/Light+ (tham chiếu GitKraken), nền dòng thêm/xoá đậm hơn | 2026-09-23 | aca0609 | — |

Last activity: 2026-09-23 - Completed quick task 260923-lhl: Reskin giao diện theo design system Linear

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
| 3 | ⛔ **ĐANG CHỜ, là việc kế tiếp.** Bản chỉ-đọc lịch sử+diff được tác giả dùng hằng ngày trên repo thật, gồm cả một repo bên thứ ba lớn và lộn xộn, **trước khi** bắt đầu việc thư mục làm việc. Bản release: **2026-09-22 11:38:59 +0700** (`0845b24`). Khung ghi chép: `docs/08-phase3-dogfood.md`. Repo lớn/lộn xộn có trên máy: `dau-tri-toan-hoc` (1140 commit, 39 merge) |
| 4 | Một commit thật vào repo thật, chỉ bằng git-plum |
| 6 | Một luồng nhánh thật đầu-cuối chỉ bằng git-plum: tạo nhánh, commit, push, giải quyết một xung đột merge thật, hợp nhất xong |

### Validation checkpoint chưa giải quyết

| # | Phase | Câu hỏi |
|---|---|---|
| ~~7~~ | 1 | **ĐẠT MỘT PHẦN** — quyết định của chủ dự án 2026-09-21. Chi tiết bên dưới. |
| 1 | 2 | Đồ thị có mở dưới 1s và cuộn 60fps trên repo 50k–100k commit thật không? — **phần Rust đã đo xong** (787.0ms/1000ms, `9e5d113`); còn thiếu vẽ lần đầu, FPS cuộn, thẳng hàng 100k, cả ba đều cần người chạy app |
| ~~2~~ | 2 | **CHỐT: CANVAS** — plan 02-05, người dùng chấp thuận qua app thật ở checkpoint vòng 2 (vòng 1 bị từ chối vì lỗi CSS cột subject, đã sửa). `interface GraphRenderer` vẫn giữ làm đường lùi. Chi tiết ở `02-05-SUMMARY.md`. |
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

**Việc tiếp theo:** ⛔ **CỔNG THOÁT PHASE 3 (Task 3 của `03-05-PLAN.md`) — chủ dự án dùng
git-plum thật vài ngày.** Đây **không** phải checklist; nó là cổng đo xem công cụ **có dùng
được** cho một luồng công việc thật không. KHÔNG cần agent viết thêm mã cho tới khi cổng trả
về kết quả.

**Bản release để dùng:**

```
src-tauri/target/release/git-plum.exe
dựng 2026-09-22 11:38:59 +0700, commit 0845b24, 4 620 800 byte
```

Đối chiếu bằng `ls -l --time-style=full-iso src-tauri/target/release/git-plum.exe` trước khi
báo lỗi — vòng checkpoint của wave 4 mất một lượt qua lại vì người dùng thử một exe **cũ hơn
bản sửa 32 phút**. Người dùng kiểm bằng **exe**, không bằng `npm run dev`.

**Để resume:** đọc `docs/08-phase3-dogfood.md` (có khung năm câu hỏi và hai giới hạn đã biết,
để không mất thời gian báo lại thứ đã đo). Ghim exe vào taskbar, và vài ngày tới khi cần đọc
lịch sử hay xem diff thì **mở nó trước** thay vì `git log`/VS Code/GitKraken. Repo: **git-plum**
(ROADMAP đòi tường minh) và **`dau-tri-toan-hoc`** (1140 commit, 39 merge — lớn và lộn xộn hơn
cả `quanly-truong-phong-so` mà plan dự kiến, và nó **có** trên máy này).

**Checkpoint 11 bước của 03-04 cũng vẫn chưa chạy** và có thể gộp vào cùng lần dùng thật —
nguyên văn 11 bước ở `03-04-SUMMARY.md`. Gộp là hợp lý: cả hai đo cùng một bản exe.

**Ba kết quả có thể, và hệ quả của từng cái:**

| Kết quả | Việc kế tiếp |
|---|---|
| **"Cổng đóng"** | Điền ghi chép vào `docs/08-phase3-dogfood.md`, cập nhật `VERIFICATION.md` (các ô "Có mã, chưa kiểm" → **"Đạt"**, kèm bằng chứng là lời xác nhận của chủ dự án), đánh dấu DIFF-01..06 Done trong `REQUIREMENTS.md`, `state advance-plan`, rồi **sang Phase 4**. |
| **"Còn thiếu X"** | Lập **`03-06-PLAN.md`** đóng khoảng cách. **KHÔNG** sang Phase 4 — Phase 4 dựng ngay trên trình xem diff này (WORK-01 đọc diff viewer tường minh), nên xây tiếp trên một trình xem chưa dùng được là nhân lỗi lên. |
| **"Hoãn"** | Ghi vào `docs/08-phase3-dogfood.md` và `VERIFICATION.md` rằng cổng thoát **chưa chạy**, và ghi đúng hệ quả: Phase 3 **chưa đóng**, và đó là **phase thứ ba liên tiếp** đóng với nợ kiểm chứng (Phase 1: ba tiêu chí; Phase 2: hai checkpoint; Phase 3: cổng thoát). |

**Một việc nhỏ cần quyết định trước:** bốn tệp trong cây làm việc có thay đổi **không thuộc
plan nào đã chạy** (`App.tsx`, `RefSidebar.tsx`, `CommitList.tsx`, hai hunk đầu của `app.css`)
— việc Phase 2 về cuộn đồ thị tới nhánh/tag. Chưa commit. Xem hàng "🔴 Cây làm việc" ở
Current Position.
