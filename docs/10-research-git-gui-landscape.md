# Khảo sát toàn cảnh Git GUI 2026 — tính năng đáng học và giao diện tối ưu

**Ngày:** 2026-09-22
**Phạm vi:** ~30 công cụ Git có giao diện (desktop thương mại, desktop mã nguồn mở, tích hợp IDE,
TUI, và thế hệ mới Sapling/Jujutsu). Nguồn: tài liệu chính thức, release notes, issue tracker,
Hacker News, dev.to, Atlassian Community, benchmark công khai. Reddit chặn crawler nên không
trích trực tiếp.

**Quan hệ với tài liệu cũ:** `docs/01-research-competitors.md` bao 7 sản phẩm ở mức bảng tổng
quan và thuật toán. `.planning/research/FEATURES.md` phân loại table-stake/differentiator dựa
trên issue tracker của SourceGit và GitHub Desktop. Tài liệu này **không lặp lại** hai tài liệu
đó; nó đi rộng hơn (thêm ~20 công cụ) và sâu hơn về **tương tác cụ thể** và **tần suất dùng thực
tế**. Mục 9 đối chiếu với roadmap hiện tại.

---

## 1. Kết luận ngắn

1. **"GUI để nhìn, CLI để làm."** Đây là mẫu dùng phổ biến nhất trong mọi thread. Ba việc người
   ta mở GUI nhiều lần mỗi ngày: xem diff working tree trước khi commit, đọc đồ thị lịch sử,
   stage theo hunk/dòng. Core Value của git-plum trúng đúng ba việc đó.
2. **Hai tính năng biến "người xem" thành "người dùng hằng ngày":** rebase tương tác trực quan và
   trình giải quyết xung đột ba khung. Người dùng terminal cũng mượn GUI cho hai việc này.
3. **Tốc độ là lý do chuyển app số một.** Benchmark mở repo Chromium: GitComet/GitFiend 1 giây,
   SourceGit 3,5 giây, SmartGit 18 giây, GitKraken 25 giây. Mọi thread "vì sao tôi bỏ X" đều nêu
   chậm trước, giá sau.
4. **Minh bạch lệnh git là tính năng xây dựng niềm tin rẻ nhất.** Sublime Merge, Git Extensions,
   lazygit, IntelliJ đều được khen vì "cho xem lệnh thật". Kiến trúc gọi `git` CLI của git-plum
   biến điều này thành gần miễn phí.
5. **Undo theo reflog/oplog đã trở thành kỳ vọng**, không còn là xa xỉ. Tower (Cmd+Z), GitKraken
   (một bước), lazygit (`z`/`Z`), GitButler (oplog), mọi GUI của Jujutsu. Kể cả người ghét
   GitButler cũng khen oplog của nó.
6. **Không được sở hữu thư mục làm việc.** Lời chê lớn nhất của GitButler là làm ngoài app thì
   hỏng trạng thái. Đọc nhiều, ghi ít, trung thành với CLI, không ref ẩn.
7. **Trạng thái toàn cục phải luôn thấy được** (nhánh, ahead/behind, số tệp bẩn, banner
   merge/rebase đang dở). SourceGit 2026.20 đưa nó khỏi tầm nhìn và bị đòi revert ngay.
8. **Hàng WIP ở đầu đồ thị** là chuẩn (GitKraken, Sublime Merge, SmartGit, GitLens 19, Tower).
   Fork thiếu nó và đó là yêu cầu mở lâu nhất (#308).
9. **Mã nguồn mở, không tài khoản, không telemetry** tự thân là tính năng giữ chân người dùng.
   GitKraken đổi gói free hai lần, Sourcetree bắt tài khoản Atlassian, SmartGit ép chạy bản mới
   nhất, GitLens khóa Pro — tất cả tạo ra làn sóng bỏ đi.
10. **Cửa sổ thị trường đang mở:** SourceGit (OSS Fork-alike năng động nhất) đăng tìm người kế
    nhiệm ngày 2026-09-18; Guitar cũng vậy; Gittyup ngưng từ 2023. Người dùng "nhanh, miễn phí,
    nhánh cổ điển" đang tìm chỗ mới.

---

## 2. Bản đồ thị trường 2026

| Sản phẩm | Loại | Nền tảng kỹ thuật | Giấy phép / giá | Tình trạng 9/2026 | Tốc độ (mở Chromium, benchmark GitComet 4/2026) |
|---|---|---|---|---|---|
| **GitKraken Desktop** 12.5 | Desktop thương mại | Electron 41 + React 17, libgit2 riêng + chế độ "Git Executable" | Free hạn chế; Pro ~$96/năm; Advanced $14/tháng | Hằng tháng; 12.0 thêm Agent Mode (worktree + Claude Code/Codex) | 25 s / 2 GB |
| **Fork** 2.70 | Desktop thương mại | Swift/Cocoa (Mac) + .NET/WPF (Win), 2 codebase tay | $59,99 một lần, eval vô hạn | 3–6 tuần/bản; không Linux (issue #1 của họ) | Không có bản Linux; "chỉ GUI Mac đủ nhanh cho monorepo của tôi" |
| **Tower** Mac 17.3 / Win 14.0 | Desktop thương mại | Obj-C/Swift + C#/WPF, 2 codebase | Thuê bao $69–149/năm | Hằng tháng; Windows tụt sau Mac 3 phiên bản lớn | Không đo; RAM rảnh 180–220 MB, poll git mỗi 5 s |
| **Sublime Merge** build 2132 | Desktop thương mại | C++ toolkit riêng + OpenGL, reader Git riêng | $99 một lần (3 năm update) | Nhịp thất thường (im 2022–2024); reftable, worktree 2026 | Mở gần tức thì; nhưng 10–15 s treo và ~1 GB trên commit 114k tệp |
| **SmartGit** 26.1 | Desktop thương mại | Java/SWT, JGit + git CLI | Free phi thương mại (phải chạy bản mới nhất); $99 perpetual | Hằng năm; thêm GitLab MR/CI, Commit Overlap | 18 s / 4,8 GB |
| **SourceGit** 2026.21 | Desktop OSS | C# + Avalonia, git CLI | MIT | 2 bản/tháng; **maintainer tìm người kế nhiệm 2026-09-18** | 3,5 s / 301 MB |
| **GitHub Desktop** 3.6.6 | Desktop OSS | Electron 44 + React | MIT | Hằng tháng; 3.6 thêm worktree + Copilot conflict | Không có graph; RAM 100–300 MB rảnh, >2 GB sau vài ngày |
| **Sourcetree** Win 3.4.30 / Mac 4.2.19 | Desktop miễn phí đóng | WPF + Cocoa, 2 codebase | Freeware, bắt tài khoản Atlassian | Bảo trì; Win 3/2026, Mac 7/2026 | "15–30 s để bắt đầu pull"; 1,7 GB RAM khi lỗi popup |
| **GitButler** 0.22 | Desktop, mô hình mới | **Tauri + Svelte + Rust**, git2 + gitoxide + SQLite | Fair Source (FSL → MIT sau 2 năm) | Rất năng động; ~315 issue mở | Chậm trên repo lớn vì diff toàn bộ index liên tục |
| **GitComet** 0.2.5 | Desktop mới (Rust toàn phần) | gitoxide + GPUI | AGPL-3 + bản Pro sắp có | Đối thủ "nhanh" trực diện nhất | **1 s / 265 MB** — stream lịch sử theo trang trong khi walk còn chạy |
| **GitFiend** 0.45 | Desktop miễn phí đóng | Rust core + Electron shell | Freeware, không mở nguồn | Ổn định, ít tính năng | 1 s / 289 MB |
| **GitDesktop** 0.12 | Desktop OSS mới | **Tauri 2 + React 19**, git porcelain v2 | Apache-2.0 | 225 sao; MSI 19 MB; palette Ctrl+K, PR/Actions | Chưa đo |
| **t4-git-ui** 0.7 | Desktop OSS mới | Tauri 2 + React 19, git2 đọc + git CLI ghi | OSS | Một người; grid kiểu Git Extensions | Chưa đo |
| **Gitnuro** 2.0 beta | Desktop OSS | Kotlin/Compose + JGit | GPL-3 | Năng động; 1.5 thêm LFS, lazy-load, stash vẽ trong graph | Chưa đo |
| **Gittyup** 2.0 | Desktop OSS | C++/Qt6 + libgit2 | MIT | Chỉ nightly từ 11/2023 | 43 s / 2,5 GB + 1,5 GB indexer |
| **Git Extensions** 7.2.1 | Desktop OSS Windows | C#/WinForms .NET 10 | GPL-3 | Đều; MSI 25 MB | Không đo |
| **TortoiseGit** 2.19.1 | Shell Explorer | C++ + libgit2 1.9 | GPL | Đều; TortoiseGitMerge vẫn là chuẩn merge editor sửa được | — |
| **Git Cola** 4.19 | Desktop OSS | Python/Qt | GPL | Đều; mới thêm DAG inline, đang làm undo stage | — |
| **Guitar**, **gitg**, **Gitte** | Desktop OSS nhỏ | Qt / GTK / Rust+GTK4 | GPL/AGPL | Guitar tìm người kế nhiệm; gitg crash trên Chromium; Gitte đang lên | — |
| **VS Code SCM + Git Graph + GitLens 19** | IDE | TypeScript | MIT / GitLens Pro $9,5/tháng | Git Graph bỏ hoang (7M cài, không license để fork); GitLens 19 (8/2026) làm lại Commit Graph | ~1 GB RAM với extension |
| **JetBrains Git** | IDE | Java | Theo IDE | Đều; 4/2026 tăng tốc rebase tương tác | "checkout 5 phút" trên repo lớn |
| **Zed Git** | IDE | Rust | Apache | 2025+; mục tiêu "nhanh hơn CLI" | — |
| **lazygit** 0.65 | TUI | Go | MIT | ~82k sao; hằng tháng | 57 s / 2,6 GB nạp log kernel |
| **gitui** | TUI | Rust | MIT | ~22k sao | 24 s / 0,17 GB; khởi động 20 ms |
| **tig**, **Magit**, **Neogit**, **fugitive**, **diffview.nvim** | TUI / Emacs / Neovim | C / Elisp / Lua | OSS | Đều | tig khởi động 10 ms |
| **Sapling ISL** | Web GUI (Meta) | TypeScript | MIT | Bản Git-only trên HN được khen nhưng đóng nguồn | Smartlog chỉ vẽ commit liên quan |
| **GG**, **jjui**, **lazyjj**, **Ukemi**, **JayJay** | GUI/TUI cho Jujutsu | Tauri+Svelte / Go / Rust | OSS | Hệ sinh thái nở 2026 | Undo qua operation log |

Bài học từ bảng: **nhóm ~1 giây** đều là Rust/native và **stream lịch sử theo trang**. Nhóm
18–43 giây đều nạp toàn bộ hoặc chạy indexer không giới hạn. Không ai vừa nhanh vừa nhẹ vừa đủ
tính năng vừa mã nguồn mở vừa đa nền tảng. Đó là khe hở.

---

## 3. Người dùng thật mở GUI để làm gì

### 3.1 Dữ liệu khảo sát

- Stack Overflow 2022 (khảo sát lớn duy nhất hỏi cách tương tác VCS, chọn nhiều): **83% dùng
  CLI**, ~56% qua IDE, ~30% qua GUI riêng. Đọc thực tế: **khoảng một phần ba dùng GUI riêng, gần
  như tất cả dùng song song với CLI.**
- Poll dev.to ~90 câu trả lời: CLI-only 28, VS Code/GitLens 12, GitKraken 11, Fork 7, Sourcetree 7,
  JetBrains 5, GitHub Desktop 5, Sublime Merge 4, Magit 3. **CLI trước, IDE sau, GUI riêng phân
  mảnh thứ ba.**
- Khi GitKraken thu hẹp gói free (2019, rồi gói sinh viên 9/2025), người dùng công khai nơi họ đi:
  Fork, SourceGit, lazygit, Sublime Merge. **Tất cả đích đến đều nhanh hơn và rẻ hơn, không phải
  nhiều tính năng hơn.**
- GitKraken "State of AI" 2026: 28% lập trình viên làm việc chủ yếu qua agent (từ 7,6% 9/2025).
  Hệ quả cho GUI: nhiều commit/nhánh nhỏ hơn để duyệt → **trình xem lịch sử và diff nhanh tăng giá
  trị**; worktree trở thành tính năng hằng ngày (GitKraken 12 Agent Mode, Tower, GitHub Desktop
  3.6 đều thêm worktree trong 2025–2026 vì lý do này).

### 3.2 Tần suất thao tác qua GUI

H = nhiều lần/ngày, M = hằng ngày–hằng tuần, L = hiếm.

| Thao tác | Tần suất qua GUI | Bằng chứng tiêu biểu |
|---|---|---|
| Xem diff working tree trước khi commit | **H** | "I really like being able to look at a diff of every file before I commit, and easily choosing which files to include" (HN 27580353) |
| Đọc lịch sử / đồ thị | **H** | "much harder to learn what's going on without a good view of the commit graph"; yêu cầu cứng số một của bài "Git clients are disappointing" là graph mặc định |
| Stage hunk/dòng rồi commit | **H** | "selecting and committing individual lines from the working copy" (HN 45878578); lazygit `<C-p>` "easily the best feature" |
| Fetch/pull/push | H (nhưng hay dùng CLI) | GUI chủ yếu auto-fetch nền |
| Chuyển/tạo nhánh | H | Table stake; thường double-click trên graph |
| Giải quyết xung đột | **M, giá trị rất cao** | "I will race you in resolving complicated merge conflicts in the CLI while I do it with jetbrains 3 window conflict editor"; người dùng terminal giữ lại đúng tính năng này của IDE |
| Rebase tương tác | **M** | GitHub Desktop #12354 mở từ 2021: "one of the crowning features of git"; "None of the IDEs I've tried support fixup" |
| Amend | M | Nằm trong ba thứ lazygit thắng: amend, split, cherry-pick |
| Stash | M | GitHub Desktop bị chê vì stash phải xuống CLI |
| Blame / lịch sử tệp | M | GitHub Desktop #2310 mở từ 2017; blame của Fork "very helpful when debugging" |
| Cherry-pick | L–M | Bị chê khi GUI làm **không hỏi** ("cherry-picked into the branch without asking me") |
| Tìm kiếm lịch sử | L–M | GitKraken: "Search commits beyond graph limit" |
| Revert / reset | L–M | "Where the GUI falls down for me is hard resetting a branch to origin" |
| Merge cục bộ | L–M | Thường merge qua PR trên host |
| Tag | L | |
| Worktree | L, **đang tăng** vì agent AI | Lỗi worktree ở Fork #947/#1278/#2586, Desktop #19307 |
| Submodule | L | Nguồn lỗi dai dẳng ở mọi app |
| Bisect | L | Gần như không ai hỏi |

**Ý nghĩa cho git-plum:** ba hàng H đầu chính là phase 2–5. Hai hàng M "xung đột" và "rebase
tương tác" là điểm chuyển đổi từ trình xem sang công cụ chính.

---

## 4. Vì sao người ta bỏ một GUI (theo thứ tự hay gặp)

1. **Chậm trên repo lớn.** Sourcetree: "Every action takes upwards of 5-10 minutes… literally
   useless" (máy i7/32 GB). GitKraken: treo vài phút khi chọn commit >10k tệp; cách chữa chính
   thức là **giảm số commit vẽ** (mặc định 2000) và "khởi động lại mỗi ngày". Ngay cả app native
   cũng vấp ở **commit khổng lồ**: Sublime Merge treo 10–15 s / 1 GB trên commit 114k tệp, Fork
   treo tương tự. → Không chỉ graph mà **danh sách tệp và diff của mega-commit cũng phải ảo hoá
   và nạp lười.**
2. **RAM / Electron.** GitKraken 500 MB+ với repo lớn, 2 GB trong benchmark; Sourcetree 1,7 GB
   khi lỗi; GitHub Desktop tăng RAM theo thời gian; VS Code + extension ~1 GB.
3. **Không biết GUI sẽ chạy lệnh gì.** "I don't know what a GUI is actually going to execute";
   "mildly terrifying when the IDE gives you a button with no explanation"; "strong correlation
   between using a GUI, and messing up the repository". Phản chứng: Git Extensions hiện dòng lệnh
   trong dialog và được coi là "learning tool"; Sublime Merge bán đúng điểm này.
4. **GUI tự làm việc không được yêu cầu.** Auto-stash khi chuyển nhánh (GitKraken, yêu cầu tắt
   lâu năm; "will confuse git veterans"); VS Code sync+autostash lặp stash/pop; GitButler mất
   5 phút nhận ra thay đổi, không báo tiến độ. Auto-fetch cũng là nguồn chậm và bão popup mật khẩu.
5. **Auto-refresh sai.** Sourcetree 2017: tệp "xuất hiện rồi biến mất thất thường… tôi commit
   tưởng đủ, hoá ra 2–3 tệp mới lại hiện". Ngược lại, trạng thái cũ cho đến khi bấm refresh cũng
   phá niềm tin. → Watcher gộp + debounce + đọc lại nguyên tử (watcher 280 ms hiện tại đúng hướng).
6. **Đổi giấy phép / bắt tài khoản / tăng giá.** GitKraken bỏ repo private khỏi gói free (2019),
   giá gấp đôi từ 2017, 12.0 âm thầm cài hook Claude Code vào config toàn cục rồi tự cài lại sau
   khi người dùng xoá (xin lỗi ở 12.0.1). Tower thuê bao: "am I paying $80 or $400?". Sourcetree
   vòng lặp đăng nhập. SmartGit ép bản mới nhất. GitLens khoá Pro. Sublime Merge bị nghi bỏ hoang.
7. **Credential / SSH.** Sourcetree popup mật khẩu "mỗi 10 giây, không nói tài khoản nào", khoá
   tài khoản do thử lại; GitHub Desktop "fetch vô hạn" khi khoá SSH có passphrase; SourceGit
   askpass hiện cho mọi thao tác trên Linux (issue nhiều reaction nhất, #1577). Nguyên nhân chung:
   **GUI đóng gói git riêng hoặc ghi đè credential.helper**. Cách chữa người dùng tìm ra: "Use
   System Git".
8. **Thiếu bàn phím / không tuỳ biến.** GitKraken: yêu cầu remap shortcut, font monospace thật
   trong diff (mở 4 năm), đổi cỡ font graph. Sourcetree: cỡ font nhỏ trên màn hi-res. "You have to
   move your hand away from the keyboard for every little thing".
9. **Line ending trên Windows.** Sourcetree đổi CRLF→LF khi bấm "Discard Hunk" (SRCTREEWIN-2294,
   17 vote). Đây chính là lớp lỗi của bộ dựng patch hunk: **giữ byte `\r\n` nguyên vẹn**.
10. **Worktree / submodule / LFS.** Fork commit lỗi trong worktree vì giả định `.git` là thư mục
    (#1278); LFS hỏi mật khẩu không huỷ được (#783). → Luôn resolve qua
    `git rev-parse --git-dir / --git-common-dir`, không giả định bố cục.

---

## 5. Bố cục giao diện tối ưu — tổng hợp từ những gì được khen

### 5.1 Khung ba vùng + hai thanh

Mọi app được khen (Fork, Sublime Merge, SourceGit, Tower, SmartGit, Git Extensions) hội tụ về:

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Toolbar: [Undo][Redo] │ [Fetch][Pull ▾][Push] (ahead/behind trên nút)  │
│          [Branch][Stash][Pop] │ … │ [⌘P palette] │ tab repo (⌘1..9)    │
├────────────┬─────────────────────────────────┬───────────────────────────┤
│ Sidebar    │ Graph / danh sách commit        │ Chi tiết                  │
│ Working    │ ┌ hàng WIP (N tệp bẩn) ────────┐│ ┌ meta + avatar ─────────┐│
│  Copy (N)  │ │ nhãn │ lane │ thông điệp │…  ││ │ danh sách tệp (cây/list)│
│ History    │ │  ●───┤      │ feat: …        ││ ├─────────────────────────┤│
│ Stashes    │ │  │   ●      │ fix: …         ││ │ diff (split/unified)    ││
│ Branches ▾ │ │  ├───┘      │ merge …        ││ │  word-level, whitespace ││
│ Remotes ▾  │ │  …                            ││ │  ⌘↑/↓ hunk kế           ││
│ Tags ▾     │ └───────────────────────────────┘│ └─────────────────────────┘│
│ Worktrees  │                                 │                           │
│ Submodules │                                 │                           │
├────────────┴─────────────────────────────────┴───────────────────────────┤
│ Status bar: nhánh │ ↑2 ↓1 │ 5 modified 2 staged │ [REBASE 3/7 đang dở]  │
└──────────────────────────────────────────────────────────────────────────┘
```

Điểm bắt buộc rút ra từ lời chê khi thiếu:

- **Danh sách tệp và diff thấy đồng thời** trong khung chi tiết (Sublime Merge bị đòi cây tệp
  #418; GitHub Desktop không có khung meta bị chê).
- **Hàng WIP ở đỉnh graph**, bấm vào đổi khung phải sang staging (GitKraken, GitLens 19, Sublime,
  SmartGit, Tower). GitKraken 12.3: **một hàng WIP cho mỗi worktree**.
- **Nhãn ref trên hàng graph**: cột riêng (GitKraken, git-plum hiện tại) hoặc chip inline đầu
  thông điệp (Fork, Sourcetree, SourceGit, SmartGit). Cả hai đều được chấp nhận; cột riêng đọc
  sạch hơn khi nhiều nhãn.
- **Banner trạng thái** khi detached HEAD / merge / rebase / cherry-pick đang dở (t4-git-ui,
  Tower nút Abort trên Working Copy). Không giấu trong menu.
- **Thanh trạng thái** có nhánh, ahead/behind, số tệp, bisect (Fork 2.64 bấm được vào ref).
- **Kích thước pane kéo được và nhớ**; phím tắt gập khung chi tiết (Fork ⌘D).
- **Hàng WIP + trạng thái toàn cục luôn hiện**, bất kể đang ở trang nào (bài học SourceGit #2714).

### 5.2 Biến thể đáng cân nhắc

- **Diff mở inline dưới hàng commit** (Sublime Merge, Git Graph): không đổi ngữ cảnh, hợp "nhìn
  nhanh". Ctrl+click mở commit ra tab riêng. Có thể làm chế độ tuỳ chọn sau.
- **Trang chủ = trạng thái gập theo mục** (Magit/Neogit): Untracked / Unstaged / Staged / Stash /
  Unpushed / Unpulled / Conflicts, mỗi mục gập được và nhớ. Khi có xung đột, danh sách thay đổi
  **biến thành danh sách xung đột** với dấu tick từng tệp và một nút Continue/Abort (ISL, lazygit).
- **Repository manager / start page** với nhóm thư mục (Fork, Sourcetree bookmark, SourceGit
  workspace, Git Extensions). HN nhắc nhiều lần "repo manager which many tools are missing".
- **Tab dọc** (Fork 2.70) cho người mở nhiều repo.

### 5.3 Ngữ pháp bàn phím được khen

| Phím | Ý nghĩa | Ai làm |
|---|---|---|
| ⌘/Ctrl+P hoặc ⌘⇧O | Palette mờ: mọi hành động + checkout nhánh + lịch sử tệp + đường dẫn tệp | Sublime, GitKraken, Fork Quick Launch, Tower Quick Actions, GitComet, GitDesktop |
| Space | Popup chi tiết commit | Fork 2.67 |
| ⌘D | Gập khung chi tiết | Fork |
| Tab | Vòng sidebar → list → chi tiết | Fork 2.66 |
| ⌘1..n | Đổi view / tab repo | Tower, GitKraken |
| ⌘↑/↓ | Hunk trước/sau | Fork |
| ⌘Z / ⇧⌘Z | Undo / Redo thao tác git | Tower, GitKraken |
| Alt+↑/↓ | Nhảy giữa kết quả type-to-search trong grid | Git Extensions |
| Ctrl+Shift+G | "Go to" nhận biểu thức rev-parse | Git Extensions |
| `?` | Menu hành động theo ngữ cảnh của panel đang focus | lazygit, Magit |
| `/` | Lọc danh sách đang focus | lazygit |

Yêu cầu mở dai dẳng: **bảng tra phím tắt** (Fork #309), **remap phím** (GitKraken #284988,
SourceGit r4/r3). Làm registry lệnh ngay từ đầu thì cả palette, bảng tra, remap đều rẻ.

### 5.4 Ngữ pháp kéo-thả được khen

| Kéo | Thả lên | Kết quả | Ai làm |
|---|---|---|---|
| Nhánh | Nhánh HEAD | Popover: merge / rebase (Alt=rebase) / start iRebase / push / pull | Tower, GitKraken, Fork, SmartGit |
| Commit | Nhánh / Working Copy | Cherry-pick (**hỏi xác nhận**) | Fork, Tower, GitHub Desktop |
| Commit | Commit khác | Squash (Alt = fixup) | Tower, SmartGit, GitHub Desktop |
| Commit | Mục Branches / Tags ở sidebar | Tạo nhánh / tag | Tower |
| Commit / cả stack | Vị trí khác trong graph | Rebase cả stack kèm **ghost preview**; khoá khi working tree bẩn và **nói lý do** | ISL, GG (jj), SmartGit |
| Tệp / hunk | Stash | Stash một phần | Tower |
| Một tệp trong stash | Working Copy | Áp dụng đúng tệp đó | Tower |
| Tệp / hunk | Commit khác | Chuyển thay đổi giữa commit (split/squash trực quan) | GG, GitButler |
| Nhãn ref / mũi tên HEAD | Commit | Di chuyển ref | SmartGit |
| Hàng trong editor rebase | Hàng khác | Sắp lại thứ tự | Fork, GitKraken, Tower, GitComet |

---

## 6. Kho tính năng theo khu vực — ai làm tốt nhất và nên học gì

Mỗi mục: **tương tác cụ thể** → ai làm → giá trị. Đánh giá cho git-plum ở mục 9.

### 6.1 Đồ thị và lịch sử

| Tính năng | Mô tả tương tác | Ai | Ghi chú |
|---|---|---|---|
| Nạp tăng dần, không cap cứng | Vẽ màn đầu <1 s, nạp tiếp sau viewport trong khi `git log` còn chạy | GitComet, Gitnuro 1.5, Sublime | GitKraken cap 2000 và SourceGit cap theo cấu hình (#2466 graph dừng ở 2019) đều sinh lời chê |
| Hàng WIP ở đỉnh | Một hàng mỗi worktree, badge số tệp bẩn, bấm → staging | GitKraken 12.3, GitLens 19, Sublime, SmartGit | Fork #308 |
| **Ghost ref khi hover** | Hover commit → hiện nhánh gần nhất chứa nó; hover nhãn nhánh → làm mờ mọi commit không thuộc nhánh | GitKraken | Rẻ, rất dễ đọc |
| Solo / Hide nhánh | Icon mắt trên nhánh → chỉ vẽ nhánh đó; ẩn từng nhánh | GitKraken, Sublime, SmartGit (checkbox Branches là filter graph) | |
| Smart visibility / Smartlog | Mặc định chỉ vẽ nhánh hiện tại + target + upstream (GitKraken 11.10); hoặc chỉ nhánh local + merge-base với main + HEAD (Sapling ISL) — cách Meta làm graph triệu commit đọc được | GitKraken, ISL, GitLens 19 "Focus Branch" | Toggle về full graph |
| Gập lane (lane folding) | Thu các lane không liên quan để graph bận đọc như log một nhánh | GitLens 19 | |
| Pin nhánh sang lane trái nhất | main/release luôn ở lane 0; icon pin nhảy tới | GitKraken 11.10 | |
| Làm mờ trước fork-point | Commit trước điểm rẽ nhánh vẽ xám | Tower 15 | |
| **Recyclable Commits** | Checkbox vẽ commit chỉ còn trong reflog (dangling) vào graph thường | SmartGit, Git Extensions (toggle reflog) | HN: "brilliant implementation of the reflog" |
| Đánh dấu commit chưa push | Marker trên hàng; ahead-of-remote | Tower, SmartGit | |
| Nhấn mạnh first-parent | Đường first-parent đậm hơn | SmartGit | |
| Commit Overlap | Đánh dấu commit chạm cùng tệp → dự đoán xung đột / ứng viên squash | SmartGit 26.1 | |
| Type-to-search trong grid | Gõ là nhảy, Alt+↑/↓ giữa kết quả | Git Extensions | |
| Ref finder type-ahead | Nhảy tới nhánh/tag/SHA không rời graph | GitLens 19 | |
| **Search DSL** | `author:me path:src/*.rs message:fix` tìm trong toàn lịch sử, kết quả tức thì | Sublime Merge | Điểm mạnh nhất của Sublime |
| Quick filter nhận tham số `git log` thô | Gõ `--since=2.weeks --author=x` | Git Extensions | Rẻ với kiến trúc CLI |
| Nhóm theo ngày/tuần/tháng | Header nhóm trong list | Tower | |
| Ctrl+click hai hàng → diff hai revision | So sánh bất kỳ | Git Extensions, SmartGit, Fork | Sublime thiếu và đó là issue top #255 |
| Gập hàng, nhớ trạng thái | Collapse commit, persist | Fork 2.67 | |
| Copy SHA một click | Bấm SHA là copy; tooltip SHA đầy đủ + ngày tuyệt đối | Sublime 2126, SourceGit | GitHub Desktop bỏ và bị chê (#9037) |
| Avatar | Gravatar mặc định, cache đĩa, URL tuỳ chỉnh cho doanh nghiệp, **không chặn render** | Mọi app | Gitnuro #292 |
| Cột tuỳ chọn | Author / Date / SHA bật tắt, kéo sắp | GitKraken, Git Extensions | |
| Màu lane | 10 màu tái dùng theo queue; ancestry của commit đang chọn tô màu, phần còn lại xám alpha 0,4 | SourceGit | Fork phải thêm 4 màu vì trùng |

### 6.2 Trình xem diff

| Tính năng | Ai | Ghi chú |
|---|---|---|
| Split / unified toggle, word-level highlight, syntax highlight | Tất cả | Sublime "diff rendering among the most readable"; GitKraken thiếu monospace thật 4 năm |
| Font monospace và cỡ font tuỳ chỉnh | Sublime, Fork | Yêu cầu mở ở GitKraken, Sourcetree |
| Ignore whitespace **đúng ngữ nghĩa `-w`** | Tất cả | Lỗi khi làm nửa vời: Fork #1834, SourceGit #1752 |
| Word-wrap toggle, Markdown Code/Preview | GitKraken 11.9/11.7 | |
| Minimap tự ẩn khi vừa màn | Fork | |
| Hunk kế / trước bằng phím | Fork ⌘↑/↓ | |
| Search overlay trong diff | Fork | |
| **Diff ảnh**: side-by-side / swipe / onion / difference, 16-bit RGBA | Fork, GitHub Desktop, SourceGit | Điểm cộng, không bắt buộc |
| Diff LFS pointer, submodule | SourceGit | Không render blob LFS |
| Danh sách tệp ảo hoá + diff lười cho commit 10k tệp | Không ai làm tốt | Sublime/Fork/GitKraken đều treo ở đây — khe hở |
| Tree view / list view chuyển đổi | Fork, SourceGit | Sublime bị đòi #418 |
| "Tree" duyệt toàn repo tại commit | Fork, Tower Tree mode, SmartGit | |
| Diff of diffs: hunk staged và unstaged phân biệt màu trong một multibuffer | Zed | Dòng xoá là text thật, tìm được |
| Cache diff theo SHA, scrub mũi tên tức thì | Mọi app nhanh | Đã có trong kế hoạch git-plum |

### 6.3 Staging và soạn commit

| Tính năng | Ai | Ghi chú |
|---|---|---|
| Stage tệp / hunk / dòng bằng click gutter | Sourcetree (nơi nhiều người học hunk staging), Fork, Sublime, GitHub Desktop | GitHub Desktop hover quá mờ bị chê (#1688/#2181) |
| **Kéo qua số dòng để chọn dải** rồi stage/discard | GitDesktop, GitHub Desktop | |
| Checkbox từng chunk + "Include selected lines" từ chuột phải | JetBrains | Bản mouse-native của `git add -p`, dễ khám phá nhất |
| **Act-on-selection**: một lệnh Stage, phạm vi = cái đang chọn (tệp/hunk/dòng), **tự nhảy tới hunk kế** | Magit | Không cần mode riêng |
| Discard hunk/dòng ngang với stage | Sourcetree, Fork | GitHub Desktop #3225 mở từ 2017 |
| Focus ổn định: hàng đổi trạng thái không nhảy list dưới con trỏ | Zed (ship cả hai layout xen kẽ/tách sau tranh luận #26862) | |
| Tệp untracked hiện nội dung trong diff | Tower 13 | |
| **Commit editor giàu** (Tower — tốt nhất nhóm): subject hard-wrap + đếm ký tự, autocomplete `#issue`, `c:commit`, `/file`, `fixup!`/`squash!`, Gitmoji `::`, menu template, **giữ Alt biến Commit thành Amend** | Tower | Yêu cầu mở nhiều nơi: conventional-commit picker, Co-Authored-By (Fork #890) |
| Conventional commit helper + template | SourceGit | Rẻ, xác định, thay được AI với nhiều người |
| Commit & Push một nút | Fork, JetBrains, SmartGit | |
| Pre-commit checks gắn nút commit (format, lint, test) | JetBrains | |
| Split commit / Compose: gom thay đổi lớn thành nhiều commit trong một luồng | GitKraken AI Commit Composer, GitLens 19 Compose, SmartGit split | Hợp thời agent AI |
| Extend / instant fixup một click: amend không sửa message; fixup vào commit chọn rồi autosquash ngay | Magit, lazygit | |
| Right-click working changes → "Fixup into this commit" | JetBrains, lazygit | HN: tính năng thiếu được đòi nhất |
| Changelist / shelve một phần | JetBrains | Gây bối rối cho người quen Git thuần |

### 6.4 Viết lại lịch sử

| Tính năng | Ai | Ghi chú |
|---|---|---|
| Editor rebase trực quan: kéo sắp, squash/fixup/reword/edit/drop, avatar, hotkey P/S/R/D | Fork (được khen "fantastic"), GitKraken, Tower, GitComet | Sublime còn dùng dialog, bị đòi DnD (#1194) |
| `--update-refs` cho stacked branch | Fork 2.61–2.63, Tower Pro | Nay là kỳ vọng |
| "Rebase last N commits" bằng shift-click dải trong graph | GitKraken 11.10 | |
| **Rewrite ngay trong graph** không cần editor: kéo commit lên commit = move/squash; kéo nhánh = rebase; kéo ref = move ref; "Modify" = iRebase dừng tại đó | SmartGit | |
| Right-click commit trong log → Squash / Fixup / Drop / Reword / Extract selected changes to separate commit; IDE tự tổng hợp rebase | JetBrains | Không mở editor đầy đủ vẫn làm được 80% việc |
| Thao tác trực tiếp trên hàng: `s` squash, `f` fixup, `d` drop, `r` reword, Ctrl+j/k move, thực thi ngay | lazygit | Viết lại lịch sử thành thao tác biên tập thường |
| Cherry-pick = copy/paste: chọn commit ở nhánh này, paste sang nhánh khác, có "clipboard" thấy được | lazygit | |
| **Custom patch**: chọn dòng trong commit cũ → chuyển vào index / commit khác / commit mới / áp ngược | lazygit Ctrl+P | Bản trực quan: kéo tệp/hunk giữa commit (GG, GitButler) |
| Conflict pre-check trước cherry-pick/revert | Fork 2.66, Tower (dự đoán N xung đột trong dialog merge và compare) | |
| Amend preview bằng Ctrl+click | Git Extensions | |
| Autosquash toggle | GitComet, SmartGit | Rẻ |

### 6.5 Xung đột

| Tính năng | Ai | Ghi chú |
|---|---|---|
| Ba khung ours / result (sửa được) / theirs; chevron nhận từng bên; **"Resolve simple conflicts"** đũa thần; "Apply all non-conflicting" | JetBrains | Chuẩn mà người dùng terminal cũng mượn |
| Merge editor đủ với biên tập text thật | Sublime Merge ("sublimely good at 3-way merges"), TortoiseGitMerge, GitComet, Fork | |
| Result **read-only** | SourceGit | Lời chê top (#2168) — không lặp lại |
| Không có editor, chỉ Conflict Wizard "Version A / Version B / Resolve using X" + tool ngoài | Tower | Bị nhắc là điểm yếu dai dẳng |
| Danh sách xung đột thay danh sách thay đổi, tick từng tệp, một nút Continue/Abort | ISL, lazygit | |
| Per-hunk pick ours/theirs/both | lazygit, JayJay | |
| Mở trong editor/mergetool ngoài luôn có | Tất cả | Nhiều người thích resolver của VS Code |
| AI giải thích / đề xuất xung đột | GitKraken 11.2, GitHub Desktop 3.6, GitLens 19 | Preview; giá trị chưa rõ |
| Conflicts-as-data: commit xung đột tồn tại, resolve sau; rebase không bao giờ dừng nửa đường | Jujutsu, GitButler | Ý tưởng dài hạn; không làm được trên git thuần |

### 6.6 Undo

| Mức | Ai | Cách làm |
|---|---|---|
| Không có | Fork, SourceGit, Sourcetree, GitHub Desktop (chỉ undo commit cuối) | Khoảng trống lớn nhất của Fork so với Tower |
| Một bước, thao tác gần nhất | GitKraken (checkout, commit, discard, delete branch, reset, rebase/cherry-pick từ 11.8) | |
| **⌘Z / ⇧⌘Z nhiều bước** cho stage/unstage, discard chunk, delete file, delete branch/tag, commit, checkout, merge, rebase, publish, reset | Tower | Lý do "vì sao tôi trả tiền Tower" được nhắc nhiều nhất; "genuinely liberating" |
| `z`/`Z` đi lùi/tiến reflog, diễn giải từng entry thành mô tả người đọc được; undo được cả việc làm ngoài app | lazygit | Nói rõ không undo được: push, working tree, giữa rebase |
| Operation log toàn workspace, timeline, restore về bất kỳ điểm | GitButler, jj (GG, jjui) | Thứ duy nhất người ghét GitButler cũng khen |
| Recyclable Commits (khôi phục thay undo) | SmartGit | |

Cách làm khả thi trên git thuần: reflog + **journal phía GUI ghi ref trước mỗi thao tác** + snapshot
working tree (stash ẩn) trước thao tác phá huỷ. Danh sách entry đọc được: "Rebased 3 commits onto
main", "Discarded 2 hunks in foo.rs".

### 6.7 Nhánh, đồng bộ, vệ sinh nhánh

| Tính năng | Ai | Ghi chú |
|---|---|---|
| Ahead/behind vẽ trên nút Pull/Push | Fork 2.62 | Không phải nhìn đâu khác |
| Pull với mode nhớ (ff-only / rebase / merge) | GitKraken | |
| Sync = pull+push một nút | Tower | |
| Force-push-with-lease mặc định | Tower, GitComet | |
| Auto-fetch **thấy được và tắt được**, mặc định tắt hoặc rất rõ | — | GitKraken/Sourcetree khuyên tắt khi chậm; Tower 5 s/3–5 process bị chê |
| Không auto-stash khi checkout; hỏi | — | GitKraken #198443 |
| **Branches Review**: mọi nhánh vs base, ahead/behind, merged?, PR, filter Active/Stale/Merged/Conflicts | Tower 2026 | Mặt trận 2026 |
| **Fully-merged với bằng chứng**: badge "Merged via PR #6 into main" + link merge commit trước khi xoá; 4 phép kiểm | Tower | Rẻ: `branch --merged` + `merge-base` + remote-tracking gone |
| Archived branches, Stale badge | Tower | |
| "My History" nhánh dùng gần đây | SmartGit | |
| Icon worktree trên nhánh đang checkout ở nơi khác; cảnh báo upstream hỏng | Fork, SmartGit 26.1 | |
| Compare branch: commit riêng của mỗi bên + số xung đột dự đoán | Tower, Fork ("click two times… half a second") | Sublime thiếu (#255) |
| Sidebar filter nhận `origin/master` hay `origin:master` | GitKraken | |
| Nhánh lồng theo thư mục `feature/x/y` | SourceGit, Fork | |
| Tạo PR: link mở trình duyệt theo forge + pill PR trên nhánh + phát hiện PR đã merge | SourceGit (link), Tower (đầy đủ) | Mức mỏng phủ 80% nhu cầu |

### 6.8 Stash, worktree, submodule, LFS, khác

| Tính năng | Ai | Ghi chú |
|---|---|---|
| Stash nhiều cái, có message, apply/pop/drop, **stash một phần** (chọn tệp / hunk / kéo tệp vào stash) | Tower, Sourcetree 3.4.30, SourceGit, SmartGit | GitHub Desktop một stash/nhánh là thất bại có tên |
| Áp một tệp từ stash | Tower | |
| Snapshot = stash nhưng giữ working tree | Tower | |
| Save stash as patch; ⌘V áp patch từ clipboard | Fork 2.69/2.64 | |
| Stash vẽ trong graph | Gitnuro 1.5 | |
| Shelf + Stash một tab | JetBrains | |
| Worktree: list sidebar, tạo từ palette, checkout remote branch as worktree, xoá nhánh + worktree, dirty state, tab tự nhóm theo repo | Fork 2.63–2.68, GitKraken 12.x, Tower 9, Sublime 2026, SmartGit 26.1 | Cả 5 app thương mại làm worktree 2025–26 vì agent AI |
| Agent Mode: tạo worktree + chạy setup + mở Claude Code/Codex/Copilot CLI một bước; card mỗi worktree | GitKraken 12.0 | Cảnh báo: 12.0 âm thầm cài hook vào config người dùng |
| Submodule: trạng thái + update, còn lại coi là opaque | Tất cả | Nguồn lỗi dai dẳng (82 issue GitHub Desktop) |
| LFS: hiện pointer vs expanded, lock | SmartGit, Fork, SourceGit | Không render blob |
| Reflog view | Fork (⌘⇧.), Tower, Sublime, SmartGit | |
| Blame: màu theo commit, phát hiện code di chuyển, "blame parent" nhảy, Ctrl+click SHA rebuild blame tại revision | Sublime, SourceGit 2026.20, SmartGit Investigate | GitHub Desktop #2310 từ 2017 |
| Lịch sử tệp theo rename; **lịch sử của một dải dòng/hunk** (`git log -L`) | Sublime, Fork 2.59–2.60, diffview.nvim | Differentiator 2025+ ít GUI có |
| Bisect UI với tô màu good/bad, trạng thái ở status bar | Fork, SourceGit, SmartGit Visual Bisect | Gần như không ai hỏi |
| Git-flow | Sourcetree, Fork, SourceGit, Sublime | |
| Issue-link regex → clickable | SourceGit, Sourcetree | Rẻ |
| Custom actions/scripts trên repo/nhánh/commit, gán phím | SourceGit, Sourcetree, Fork, lazygit (YAML với prompt, template `{{.SelectedLocalCommit.Sha}}`, output mode) | Hút script của team thay vì bị bỏ vì thiếu |
| Chạy git subcommand bất kỳ, stream output vào dock có Cancel | t4-git-ui, lazyjj (`:`), fugitive `:Git` | Rẻ, người quen CLI thích |
| Shallow/partial clone trong dialog clone | GitKraken 11.8 | |
| Mở trong editor/terminal/file manager, cấu hình được shell | Tất cả | ~20 issue SourceGit đều xin *bên ngoài*, không ai xin nhúng |
| CLI `smerge .` mở app từ terminal | Sublime | Chiều ngược lại |

### 6.9 Minh bạch và bàn phím

| Tính năng | Ai | Ghi chú |
|---|---|---|
| **Command log**: mọi lệnh git chạy, live, copy được | lazygit (panel dưới), Sublime Git Output, SourceGit, JetBrains Console, Git Extensions (dialog hiện dòng lệnh) | "all commands are printed and rather complicated rebases become a breeze" |
| Preview lệnh trong dialog trước khi bấm | Git Extensions, Sublime "Show Git command" | |
| **Transient popover** thay dialog nhiều checkbox: nút Push mở popover toggle `--force-with-lease`, `--set-upstream`, `--tags`, hiện dòng lệnh kết quả, nhớ mặc định | Magit `transient` | Áp vào GUI: popover gắn nút toolbar |
| Palette mờ mọi hành động, hiện phím tắt, lọc theo ngữ cảnh focus | Sublime, Fork, Tower, GitKraken, GitComet (~50 lệnh), GitDesktop | |
| Keymap/menu/palette cấu hình JSON | Sublime | |
| Menu `?` theo ngữ cảnh | lazygit, Magit | Bản GUI = chuột phải + palette lọc theo focus |
| Optimistic UI: hiện trạng thái dự đoán ngay, đối soát khi git xong | ISL, Zed | Cảm giác nhanh |
| Watch `.git` để phản chiếu thao tác CLI tức thì | Zed, GitButler, GitFiend, mọi app tốt | Trộn GUI/terminal không ma sát |
| Tiến độ + Cancel trên thao tác dài | — | GitButler "no indication how long you'll be stuck" |
| Xác nhận trước thao tác phá huỷ / viết lại (reset --hard, force push, cherry-pick) | — | |
| Benchmark công khai tái lập được (kernel/Chromium, cold open → cuộn được, RSS) | Không ai độc lập | Vị trí marketing còn trống; nhóm này đọc số |

### 6.10 Đa repo

Tab (⌘1..9, kéo sắp, alias, màu, đổi tên) + start page/repository manager nhóm thư mục + recent
list + workspace (SourceGit, Fork, Sourcetree, Git Extensions, GitKraken, Tower). Tab hiện nhánh +
số tệp bẩn (SourceGit r4). GitHub Desktop một repo/lần bị chê trong mọi review (#20026, #19030,
#12578).

### 6.11 AI (2025–2026)

| Ai | Có gì | Phản ứng |
|---|---|---|
| GitKraken | Commit message, giải thích thay đổi/nhánh, xung đột, PR title; Gemini mặc định; tín dụng theo gói; BYOK gồm Ollama | Yêu cầu tắt cho repo private (#444602); "removed the AI stuff and focused on core" |
| GitHub Desktop 3.5–3.6 | Copilot commit message (GA 6/2025), giải thích xung đột; đọc `AGENTS.md` | "should be opt-in, not on by default"; "too verbose"; đòi Conventional Commits; đòi Ollama (#20165 đóng duplicate); "total nonsense" (#20676) |
| Tower 16 | Commit message qua Claude Code / OpenAI Codex, prompt tuỳ chỉnh | |
| Fork 2.59–2.70 | Commit message qua Claude / Codex / Cursor agent, prompt sửa được | |
| SourceGit | OpenAI-compatible endpoint (Ollama chạy), conventional-commit helper | Không gây tranh cãi |
| GitLens 19 | Compose, AI rebase, AI resolve, theo dõi phiên Claude Code từ graph | Sau paywall |
| Cursor | Sparkle commit message, "Resolve in Chat", Cursor Blame phân biệt người/agent | |
| Sublime Merge, SmartGit (Ask-AI có), gitui, lazygit | Không / ít | |

Kết luận không đổi so với FEATURES.md: **opt-in, local-first, khoá trong keychain, biết
Conventional Commits, "cải thiện bản nháp" nổi hơn "sinh từ đầu", "giải thích diff/commit" được
coi giá trị hơn "viết message".** Điểm mới 2026: **agent attribution** (blame phân biệt
người/agent, trailer "Made with …") bắt đầu là kỳ vọng.

---

## 7. Thực tế Windows

- **Console nhấp nháy**: spawn `git.exe` không có `CREATE_NO_WINDOW` (0x08000000) → một cửa sổ
  đen mỗi lệnh. Cách chữa sai (đổi parent thành GUI-subsystem) làm một tool Rust tệ hơn 17 lần
  (squeez #231). Release blocker.
- **Credential**: dùng `git.exe` của người dùng và GCM đã cấu hình; không đóng gói git; không ghi
  đè `credential.helper`; tôn trọng `GIT_SSH` / `core.sshCommand`; để prompt OpenSSH agent hiện
  được (GitHub Desktop fetch vô hạn im lặng là bài học).
- **CRLF**: parse output git dạng byte; chỉ bỏ `\r` khi tách `\n` để hiển thị; patch hunk giữ
  nguyên byte (SRCTREEWIN-2294).
- **Long path**: `core.longpaths` có từ Git for Windows 1.9.5 nhưng Windows vẫn giới hạn 260 mặc
  định; hiện lỗi git nguyên văn + gợi ý một click bật `core.longpaths`.
- **Antivirus**: "Every file read triggered a scan… thousands of small file scans" (post-mortem
  GitKraken). Giảm số spawn mỗi refresh (một `status --porcelain=v2 -z`, không gọi theo tệp); cân
  nhắc phát hiện chậm bất thường và gợi ý exclusion.
- **WSL**: repo `\\wsl$\…` chậm 10–30 s từ client Windows; GitKraken bị đòi WSL2 (#194156). v1:
  phát hiện UNC/WSL và cảnh báo; v2: chạy `wsl git`.
- **OneDrive**: ".git folder in OneDrive is corrupted after the sync finishes"; "OneDrive and git —
  don't do it". Phát hiện `OneDrive` trong đường dẫn (repo dev này đang nằm trong OneDrive) và
  cảnh báo không chặn; chờ đợi watcher event giả và race `index.lock`.
- **High-DPI**: Sourcetree 125% chồng icon/text; canvas graph phải dùng `devicePixelRatio`.
- **`index.lock`**: auto-fetch/status đua với terminal → "another git process seems to be
  running"; retry backoff, **không bao giờ tự xoá lock**.
- **Env**: GitButler spawn login shell ở thread nền trên Windows để lấy PATH/ssh-agent đúng mà
  không chặn khởi động.

---

## 8. Kiến trúc đáng tham khảo từ app cùng stack

### 8.1 GitButler (Tauri + Svelte + Rust)

- Monorepo `apps/desktop` + ~45 crate `but-*`; luật CI: crate mới không phụ thuộc crate legacy.
- **Proc-macro `#[but_api]`**: một `fn(ctx, params) -> Result<T>` sinh ra (a) `#[tauri::command]`,
  (b) handler JSON-RPC cho `but-server` (Axum, dùng cho web/CLI), (c) napi binding; tự remap kiểu
  (`gix::ObjectId` → hex), token quyền `&mut RepoExclusive` / `&RepoShared`, `catch_unwind`, và
  **tag cache khai báo `provides=[…]` / `invalidates=[…]`** phát event invalidation cho RTK Query
  phía frontend.
- Frontend: `tauriInvoke<T>` chuẩn hoá lỗi thành `IpcError`; `tauriListen` cho event; watcher
  `gitbutler-filemonitor` push event tăng dần.
- Bài học ngược: **Tauri không phải nút thắt**; chậm vì mô hình virtual branch phải diff toàn index
  liên tục (LLVM import vài phút #2916, 20k tệp #2938, kéo hunk treo trên monorepo #3235).
- Dùng git2 + gitoxide nên phải tự làm `but-askpass` và shim `gitbutler-git` — thứ git-plum được
  miễn phí nhờ gọi CLI.

### 8.2 SourceGit (Avalonia)

- `git log --no-show-signature --decorate=full --format=%H%x00%P%x00%D%x00%aN±%aE%x00%at%x00%cN±%cE%x00%ct%x00%s` — 8 trường tách NUL, parse từng dòng; search mode cap `-1000 --date-order`.
- Lane O(n) một lượt: `unitWidth=12`, `PathHelper` mỗi lane, first-parent tiếp lane, parent phụ
  thành Bezier "link"; 10 màu queue; dot Default / Head (vòng đôi) / Merge (vòng + chữ thập).
- Render custom control chỉ vẽ dải Y nhìn thấy (`PushClip` + translate); ancestry của commit đang
  chọn tô màu, còn lại xám alpha 0,4; hàng = `rowHeight × index`. **Đúng mẫu canvas + hàng ảo hoá
  git-plum đang dùng.**
- Điểm yếu để vượt: cap cứng số commit, merge result read-only, không undo/oplog, không palette,
  askpass storm, tab switch vài giây (#1808), macOS stage/unstage chậm (#1720).

### 8.3 GitComet (Rust + GPUI)

"History loads in pages and renders while the walk is still running" — chính là mẫu
Channel-streaming trong STACK.md. 1 s / 265 MB trên Chromium là con số mục tiêu để so.

---

## 9. Đối chiếu với roadmap git-plum hiện tại

Đánh giá dựa trên `.planning/PROJECT.md` (Out of Scope), `.planning/ROADMAP.md` (8 phase),
`FEATURES.md` (table stake/differentiator). Không đề xuất đổi phase đang chạy; chỉ nêu bằng chứng
mới và mức ưu tiên gợi ý.

### 9.1 Đã đúng hướng — giữ nguyên

| Quyết định hiện tại | Bằng chứng mới củng cố |
|---|---|
| Core Value graph <1 s, 100k commit, canvas + virtual rows | Benchmark 1 s vs 25 s là chỉ số churn số một; không ai độc lập đo → làm benchmark công khai (phase 8) |
| Gọi git CLI, không libgit2 | GitKraken phải thêm "Git Executable mode"; GitButler phải tự làm askpass; SourceGit askpass storm; Sourcetree "Use System Git" |
| Command log v1 | Được khen ở 5 công cụ khác nhau; lý do bỏ GUI số 3 |
| Palette v1 | Có ở mọi app thương mại 2026; GitComet/GitDesktop/t4 đều ship ở v0.x |
| Không terminal nhúng, có "Open in …" | Không đổi |
| Xung đột 2 khung v1, không merge tool 3 khung đầy đủ | **Cần điều chỉnh nhẹ:** result phải **sửa được** (SourceGit read-only là lời chê top). Hai khung với result sửa được + take-ours/theirs/both từng khối + mở tool ngoài là mức tối thiểu được chấp nhận |
| Discard safety net v1 | Không đổi |
| Watcher gộp 280 ms | Đúng hình dạng; cả hai chiều lỗi (thất thường / cũ) đều phá niềm tin |
| AI opt-in, local, keychain, "cải thiện" ≥ "sinh" | Không đổi; thêm Conventional Commits awareness |
| Không tài khoản, không telemetry, MIT | Là tính năng giữ chân, không chỉ ràng buộc |

### 9.2 Rẻ, nên thêm vào v1 nếu chưa có (không đổi phạm vi phase)

| Tính năng | Vì sao rẻ | Phase gần nhất |
|---|---|---|
| Hàng WIP ở đỉnh graph với badge số tệp bẩn (đã có `wipRow.ts`) — đảm bảo bấm vào đổi khung phải sang staging | Đã có nền | 2/4 |
| Ahead/behind trên nút Pull/Push; unpushed marker trên hàng | Một `rev-list --left-right --count` | 6 |
| Banner trạng thái detached/merge/rebase/cherry-pick dở + nút Abort | Đọc `.git/MERGE_HEAD`, `rebase-merge/` | 6 |
| Copy SHA một click; tooltip SHA đầy đủ + ngày tuyệt đối | Trivial | 2 |
| Ghost ref khi hover commit; làm mờ commit ngoài nhánh khi hover nhãn | Có sẵn dữ liệu lane/ancestry | 2 |
| Quick filter nhận tham số `git log` thô (`--since`, `--author`) | Pass-through | 2 |
| Không auto-stash khi checkout; hỏi. Auto-fetch mặc định tắt hoặc rõ ràng | Chính sách | 6 |
| Xác nhận trước reset --hard / force push / cherry-pick, dialog hiện dòng lệnh | Có command log rồi | 6 |
| Conventional-commit prefix picker + template + đếm ký tự subject + Alt=Amend | UI thuần | 4 |
| Ignore whitespace đúng `-w`; font monospace + cỡ font cấu hình | CodeMirror | 3 |
| Detect OneDrive / UNC / WSL path → cảnh báo không chặn | String match | 1 |
| Long path error → gợi ý `core.longpaths` | Match stderr | 1 |
| `index.lock` retry backoff | Wrapper git | 1 |
| Stash nhiều cái + message + stash tệp chọn | `stash push -m -- <paths>` | 6 |
| Issue-link regex per repo | Regex | 2 |
| Tab hiện nhánh + số tệp bẩn | Có dữ liệu | 1 |
| Fully-merged hint có bằng chứng trước khi xoá nhánh | `branch --merged` + `merge-base` + tracking gone | 6 |

### 9.3 Bằng chứng mới thách thức Out of Scope — cân nhắc cho v1.x, không phải v1

| Mục Out of Scope | Bằng chứng mới | Gợi ý |
|---|---|---|
| **Undo tổng quát** | Đã là kỳ vọng 2026 (Tower, GitKraken, lazygit, GitButler, jj). Lý do "vì sao trả tiền Tower" số một. lazygit chứng minh reflog-based undo làm được trên git thuần với phạm vi nói rõ | Giữ ngoài v1, nhưng **thiết kế journal thao tác từ phase 6** (ghi ref trước/sau mỗi lệnh ghi) để v1.x thêm undo không phải retrofit. Bắt đầu bằng danh sách "thao tác gần đây" chỉ đọc |
| **Blame** | GitHub Desktop #2310 mở từ 2017; Fork blame được khen khi debug; **lịch sử dải dòng** là differentiator 2025+ | v1.x. Gutter read-only là đủ |
| **Worktree** | Cả 5 app thương mại ship worktree 2025–26 vì agent AI; GitKraken 12 xây Agent Mode quanh nó; 28% dev làm việc qua agent | v1.x, ưu tiên cao hơn PROJECT.md hiện ghi. Tối thiểu: list + tạo + xoá + icon trên nhánh + hàng WIP mỗi worktree. **Ngay v1: resolve `--git-dir/--git-common-dir` đúng** để không lỗi kiểu Fork #1278 |
| **Rebase tương tác kéo thả** | GitHub Desktop #12354 từ 2021; "one of the crowning features"; JetBrains chứng minh **right-click Squash/Fixup/Drop/Reword trên log** phủ 80% mà không cần editor; `--update-refs` nay là kỳ vọng | Giữ editor DnD ở v2. v1 đã có reset + fixup/squash non-HEAD; **thêm right-click Drop/Reword/Squash-into-parent** là bước nhỏ tiếp theo cho v1.x |
| **Tích hợp PR/Issues** | Mọi app hoặc quá nặng hoặc không có; mức mỏng "mở PR trong trình duyệt + pill PR trên nhánh + phát hiện PR đã merge" phủ 80% | v1.x, mức link-out như SourceGit |
| **Tối ưu >100k** | GitComet/GitFiend 1 s trên Chromium (~1,5M commit) đặt chuẩn | Không đổi; benchmark công khai ở phase 8 quyết định |

### 9.4 Không nên làm (xác nhận lại)

- Terminal nhúng. Bisect UI (không ai hỏi). Mô hình virtual branch / sở hữu working dir. Đóng gói
  git riêng. Cap cứng số commit. Indexer nền không giới hạn (Gittyup 1,5 GB). Auto-stash im lặng.
  AI mặc định bật. Cài hook vào config người dùng (GitKraken 12.0).

---

## 10. Nguồn chính

**Benchmark / dữ liệu**
- https://gitcomet.dev/best-git-gui-for-linux/ — Chromium open-time, 10 client Linux, 4/2026 (vendor)
- https://www.pistack.xyz/posts/2026-08-29-lazygit-vs-gitui-vs-tig-git-terminal-ui-comparison/
- https://survey.stackoverflow.co/2022/ ; https://www.adesso.de/en/news/blog/stack-overflow-developer-survey-2022-part-2-2.jsp
- https://dev.to/madza/what-git-gui-client-do-you-use-3h9h/comments
- https://gitkraken.com/reports/state-of-ai ; https://www.gitkraken.com/blog/2024-state-of-git-report-gitkraken-jetbrains
- https://github.com/orgs/community/discussions/173802 ; https://lemmy.world/post/19913337 — churn GitKraken

**Thương mại**
- GitKraken: https://help.gitkraken.com/release-notes/current ; https://help.gitkraken.com/gitkraken-desktop/interface/ ; https://help.gitkraken.com/gitkraken-desktop/performance-issues/ ; https://help.gitkraken.com/gitkraken-desktop/undo-and-redo/ ; https://gitkraken.com/blog/gitkraken-desktop-12-0-1-update ; https://www.gitkraken.com/pricing
- Fork: https://releasebot.io/updates/git-fork ; https://github.com/fork-dev/Tracker/issues/153 ; https://github.com/fork-dev/TrackerWin/issues/1278 ; HN 31567702, 20511555, 17448551
- Tower: https://www.git-tower.com/release-notes ; https://www.git-tower.com/help/guides/faq-and-tips/undoing-things/mac ; https://www.git-tower.com/help/guides/faq-and-tips/tips-and-tricks/drag-drop/mac ; https://www.git-tower.com/pricing ; HN 17401398, 28892556
- Sublime Merge: https://www.sublimemerge.com/docs/getting_started ; https://github.com/sublimehq/sublime_merge/issues/138 , /255 , /1194 , /1937 ; HN 40862144, 27236358, 18030446
- SmartGit: http://blog.syntevo.com/smartgit/2026/05/18/smartgit-26-1-released.html ; https://docs.syntevo.com/SmartGit/Latest/Manual/GUI/Branch/Rebase-Interactive ; HN 17633470

**Mã nguồn mở**
- SourceGit: https://github.com/sourcegit-scm/sourcegit ; issues #2722 (kế nhiệm), #1577 (askpass), #2168 (merge read-only), #2466 (cap), #2714 (layout 2026.20), #1720, #1808 ; src `Models/CommitGraph.cs`, `Views/CommitGraph.cs`, `Commands/QueryCommits.cs`
- GitHub Desktop: https://github.blog/changelog/2026-06-26-github-desktop-3-6-worktrees-and-deeper-copilot-integration/ ; issues #1634, #13365, #18390, #19135 (graph), #12354 (iRebase), #2310 (blame), #3225 (discard hunk), #1688/#2181 (staging feedback), #9037 (copy SHA), #20026/#19030/#12578 (multi-repo), #8761/#16542 (SSH), #20165 (Ollama), #20676
- Sourcetree: release notes Win 3.4.30 / Mac 4.2.19 ; Atlassian Community threads qaq-p/786541, 582066, 594324, 579905, 2599952, 3198022, 1385328 ; SRCTREEWIN-2294 ; SRCTREE-1286
- GitButler: https://github.com/gitbutlerapp/gitbutler ; `crates/but-api-macros/src/lib.rs` ; `crates/gitbutler-tauri/src/main.rs` ; issues #2916, #2938, #3235 ; HN 41184037, 41926942, 39359118, 39594164, 46767534 ; https://docs.gitbutler.com/overview
- GitComet https://gitcomet.dev/features ; GitDesktop https://github.com/theBGuy/GitDesktop ; t4-git-ui https://github.com/toperux/t4-git-ui ; Gitnuro https://github.com/JetpackDuba/Gitnuro/releases ; Gittyup https://github.com/Murmele/Gittyup/releases ; Git Extensions https://github.com/gitextensions/gitextensions/releases ; TortoiseGit https://tortoisegit.org/docs/releasenotes/ ; Git Cola https://git-cola.readthedocs.io/en/latest/relnotes.html ; Guitar https://github.com/soramimi/Guitar ; Gitte https://www.phoronix.com/news/Gitte-0.10-Released

**IDE / TUI / thế hệ mới**
- VS Code SCM https://code.visualstudio.com/docs/sourcecontrol/overview ; issues #13740, #48323, #160544 ; Git Graph #838, #927 ; GitLens 19 https://gitkraken.com/blog/gitlens-19-the-commit-graph-reimagined-for-parallel-development ; https://help.gitkraken.com/gitlens/gl-commit-graph/
- JetBrains: https://www.jetbrains.com/help/idea/commit-and-push-changes.html ; /edit-project-history.html ; /resolve-conflicts.html ; https://blog.jetbrains.com/platform/2026/04/speeding-up-interactive-rebase-in-jetbrains-ides/
- lazygit: https://github.com/jesseduffield/lazygit ; docs/Undoing.md ; docs/Custom_Command_Keybindings.md ; discussion #3590 ; HN 36782018, 37009879
- gitui https://github.com/gitui-org/gitui ; tig https://github.com/jonas/tig ; Magit https://docs.magit.vc/transient/ , https://emacsair.me/2017/09/01/magit-walk-through/ ; Neogit https://github.com/NeogitOrg/neogit ; diffview.nvim https://github.com/sindrets/diffview.nvim
- Zed: https://zed.dev/blog/git ; https://zed.dev/docs/git ; discussion #26862 ; Cursor https://cursor.com/help/integrations/git
- Sapling ISL https://sapling-scm.com/docs/addons/isl/ ; HN 39730891 ; Jujutsu: https://docs.jj-vcs.dev/latest/operation-log/ , /conflicts/ ; GG https://github.com/gulbanana/gg ; jjui https://github.com/idursun/jjui ; lazyjj https://github.com/Cretezy/lazyjj ; https://github.com/jj-vcs/jj/wiki/GUI-and-TUI

**Thread "GUI vs CLI"**
- HN 38466232, 27580353, 22853063, 45878578 ; https://dev.to/jimmymcbride/do-you-still-use-git-in-the-terminal-2n7a/comments ; https://dev.to/stefnotch/git-clients-are-disappointing-bl9 ; https://danmackinlay.name/notebook/git_guis.html ; https://gitting.substack.com/p/cli-vs-guis ; https://jonathansblog.co.uk/best-git-ui-clients-2026

**Windows**
- https://github.com/claudioemmanuel/squeez/issues/231 (console flash) ; https://owain.codes/blog/2025/december/when-gitkraken-desktop-runs-slow (AV) ; https://techcommunity.microsoft.com/discussions/onedriveforbusiness/onedrive-is-corrupting-my-git-repositories/3898283 ; https://www.dustinbriles.com/onedrive-and-git-dont-do-it/ ; https://linuxvox.com/blog/why-is-git-on-wsl2-with-soucetree-so-slow-for-me/ ; https://feedback.gitkraken.com/suggestions/194156/support-for-wsl2-on-windows

**Giới hạn bằng chứng:** Reddit không truy cập được; fork.dev và sublimemerge.com từ chối kết
nối (dùng mirror); mọi benchmark tốc độ đều do vendor công bố; số liệu khảo sát GUI/IDE 2022 là
xấp xỉ; tần suất thao tác suy ra từ thread và issue, không có telemetry.
