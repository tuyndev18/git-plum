<!-- GSD:project-start source:PROJECT.md -->
## Project

**git-plum**

git-plum là một ứng dụng desktop mã nguồn mở để quản lý repository Git bằng giao diện đồ hoạ,
dành cho lập trình viên đã quen dùng Git nhưng muốn một công cụ nhanh và nhẹ hơn GitKraken,
đồng thời đầy đủ tính năng hơn GitHub Desktop. Ứng dụng tập trung vào ba việc mà lập trình
viên làm nhiều nhất hằng ngày: đọc lịch sử commit qua đồ thị nhánh, xem khác biệt giữa các
phiên bản, và tạo commit sạch sẽ thông qua staging theo từng khối thay đổi.

**Core Value:** Đọc và hiểu lịch sử của một repository phải **tức thì** — đồ thị commit mở ra trong dưới một
giây và cuộn mượt kể cả trên repo hàng chục nghìn commit. Nếu mọi tính năng khác thất bại,
riêng điều này vẫn phải đúng.

### Constraints

- **Tech stack**: Tauri v2 + React + TypeScript ở lớp trên, Rust ở lớp dưới — Cần vừa xử lý
  dữ liệu nhanh (phân tích hàng chục nghìn commit, tính lane, phân tích diff) vừa dựng giao
  diện phức tạp (đồ thị, trình xem diff, cây thư mục). Tauri cho cả hai với dung lượng cài
  đặt dưới 20MB và RAM lúc rảnh dưới 150MB.
- **Backend Git**: Chỉ gọi `git` CLI, không dùng libgit2 hay gitoxide — Bảo đảm tính đúng đắn
  và tận dụng miễn phí credential helper, LFS, hook, submodule.
- **Compatibility**: Windows, macOS, Linux — Là sản phẩm mã nguồn mở công khai, người dùng
  ở cả ba nền tảng. Windows được ưu tiên phát triển và kiểm thử trước vì đó là máy của tác giả.
- **Performance**: Đồ thị mở dưới 1 giây và cuộn mượt ở mức 100k commit — Đây là Core Value,
  không phải mục tiêu phụ.
- **Security**: Khoá API của nhà cung cấp AI lưu trong keychain hệ điều hành, không bao giờ
  ghi vào tệp cấu hình dạng văn bản thuần — Đây là công cụ mã nguồn mở, người dùng phải tin
  tưởng được.
- **Privacy**: Tính năng AI phải chọn tham gia (opt-in) và cho phép dùng model local — Nội dung
  diff có thể chứa mã nguồn nhạy cảm. Không gửi gì ra ngoài khi chưa được cho phép rõ ràng.
- **Licensing**: Giấy phép mã nguồn mở (dự kiến MIT) — Phù hợp định vị sản phẩm công khai.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Technologies
| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `tauri` (crate) | **2.11.6** | Desktop shell, IPC, bundler | Current stable. v2 IPC is a custom-protocol/fetch hybrid (not v1's string-based `postMessage`), which is what makes streaming 50k commits viable at all. |
| `tauri-build` (crate) | **2.6.3** | Build script codegen | Must be kept in the same 2.x line as `tauri`. |
| `@tauri-apps/api` | **2.11.1** | JS side of IPC (`invoke`, `Channel`, `event`) | Pairs with `tauri` 2.11.x. |
| `@tauri-apps/cli` | **2.11.5** | `tauri dev` / `tauri build` | Dev dependency only. |
| React | **19.3.0** | UI layer | React 19 is the baseline everything else now peers against. No reason to pin 18. |
| TypeScript | **6.0.3** ⚠️ | Type checking | **NOT 7.0.2.** See the critical section below — TS 7 has no stable programmatic API, so `typescript-eslint` refuses to run on it. |
| Vite | **8.3.0** | Dev server + bundler | Tauri's dev server integration expects a standard Vite dev server. Vite 8 requires Node `^20.19 \|\| >=22.12` — your Node 22.16 satisfies this. |
| `@vitejs/plugin-react` | **6.1.1** | React fast-refresh | Peers `vite: ^8.0.0`, so it locks you to Vite 8. Includes React Compiler support via `babel-plugin-react-compiler`. |
| Rust toolchain | **stable-msvc** | Backend | On Windows, the MSVC target — *not* the GNU target. See Windows setup. |
### Rust Crates (`src-tauri/Cargo.toml`)
| Crate | Version | Purpose | Why |
|-------|---------|---------|-----|
| `tokio` | **1.53.1** | Async runtime + **`tokio::process::Command`** | Tauri already pulls tokio in. Use `features = ["process", "io-util", "rt", "macros"]`. `tokio::process` lets you stream `git log` stdout incrementally instead of buffering a 50 MB `Output` struct. |
| `serde` | **1.0.229** | Derive `Serialize`/`Deserialize` | `features = ["derive"]`. Non-negotiable — every IPC type needs it. |
| `serde_json` | **1.0.151** | JSON for IPC | Already a transitive dep of `tauri`; declare it explicitly anyway so you control the version. |
| `thiserror` | **2.0.20** | Library error types | **v2, not v1.** Use for your typed `GitError` enum. Tauri commands require the error type to impl `Serialize` — `thiserror` gives you `Display`, you hand-write the `Serialize` impl (see pattern below). |
| `bstr` | **1.13.1** | Byte-string parsing | **This is the underrated pick.** `git` output is *bytes*, not guaranteed UTF-8 (filenames on Linux, commit messages in legacy encodings). Parsing `Vec<u8>` with `bstr` avoids `String::from_utf8` panics/lossy corruption on real-world repos. |
| `memchr` | **2.8.3** | SIMD byte scanning | Splitting on `\x1f` / `\x1e` / `\0` across a 50 MB buffer. Transitive via `bstr`, but you'll call it directly in the hot split loop. |
| `lru` | **0.18.4** | Diff cache | The docs already call for caching diffs by commit SHA. Bounded LRU prevents unbounded RAM growth. |
| `parking_lot` | **0.12.5** | `Mutex`/`RwLock` for `tauri::State` | Faster and no poisoning semantics vs `std::sync`. Note: use `std::sync` if you need to hold a lock across `.await` — `parking_lot` guards aren't `Send`-safe across await points. |
| `tracing` | **0.1.44** | Structured logging | Pairs with `tauri-plugin-log`. |
### Tauri Plugins
| Plugin | Rust crate | npm package | Needed? | Notes |
|--------|-----------|-------------|---------|-------|
| **dialog** | `tauri-plugin-dialog` **2.7.3** | `@tauri-apps/plugin-dialog` **2.7.3** | ✅ Required | Folder picker for "Open repository". Desktop-only. |
| **store** | `tauri-plugin-store` **2.4.5** | `@tauri-apps/plugin-store` **2.4.5** | ✅ Required | Recent-repos list, UI prefs, pane sizes. JSON key-value. **Not for API keys.** |
| **window-state** | `tauri-plugin-window-state` **2.4.1** | `@tauri-apps/plugin-window-state` **2.4.1** | ✅ Required | Remembers window size/position. Requirement in PROJECT.md. |
| **opener** | `tauri-plugin-opener` **2.5.5** | `@tauri-apps/plugin-opener` **2.5.5** | ✅ Required | Open external editor for merge conflicts (v1 scope says "open external editor"). **This replaced v1's `shell.open`.** |
| **log** | `tauri-plugin-log` **2.9.2** | `@tauri-apps/plugin-log` **2.9.2** | ✅ Recommended | Bug reports from users of an OSS tool are useless without logs. |
| **updater** | `tauri-plugin-updater` **2.12.0** | `@tauri-apps/plugin-updater` **2.12.0** | ⏳ Defer to release phase | Requires a signing keypair + a static `latest.json` endpoint. Don't wire it up in Phase 1. |
| **single-instance** | `tauri-plugin-single-instance` **2.4.5** | — | ⏳ Nice-to-have | Prevents two copies fighting over the same repo. Desktop-only. |
| **process** | `tauri-plugin-process` **2.3.1** | `@tauri-apps/plugin-process` **2.3.1** | ⏳ Only with updater | `relaunch()` after update. Does **not** spawn children — common misreading of the name. |
| **os** | `tauri-plugin-os` **2.3.2** | `@tauri-apps/plugin-os` **2.3.2** | ⚠️ Probably not | You need platform info in *Rust*, where `cfg!(windows)` is free. Only add if the frontend needs it. |
| **shell** | `tauri-plugin-shell` **2.3.6** | `@tauri-apps/plugin-shell` **2.3.6** | ❌ **Do not add** | See below — this is the single most common wrong turn for this app. |
| **stronghold** | `tauri-plugin-stronghold` **2.3.2** | `@tauri-apps/plugin-stronghold` **2.3.2** | ❌ **Do not add** | See the keychain section. Wrong tool, and its 370k downloads vs dialog's 16M tells you how little it's exercised. |
| **fs** | `tauri-plugin-fs` **2.5.2** | `@tauri-apps/plugin-fs` **2.5.2** | ❌ Not needed | Exposes the filesystem to *JavaScript*. Your file access is all in Rust via `std::fs`. Adding it widens the attack surface for zero gain. |
#### ❌ Why NOT `tauri-plugin-shell`
## 🔴 CRITICAL: No official Tauri keychain plugin
### ✅ Use the `keyring` crate directly
| Crate | Version | Notes |
|-------|---------|-------|
| `keyring` | **4.2.0** | Actively maintained (open-source-cooperative), 26M downloads, released 2026-08-29. |
# src-tauri/Cargo.toml
- **keyring v4 restructured heavily from v3** (API moved into `keyring-core`, stores into separate crates). Training data and most blog posts describe the v3 `Entry::new(service, user)` API. Verify against [docs.rs/keyring/4.2.0](https://docs.rs/keyring/4.2.0/keyring/) when implementing — the `v1` feature exists precisely to preserve the old ergonomics, but confirm the import path.
- **Linux headless/CI has no Secret Service.** `keyring` will error there. Degrade gracefully: if the keychain is unavailable, disable AI features with a clear message rather than falling back to a plaintext file (that would violate the stated constraint).
## 🔴 CRITICAL: TypeScript 7 vs TypeScript 6
| Option | Verdict |
|--------|---------|
| **`typescript@6.0.3`** | ✅ **Recommended.** Latest stable in the 6.x line, inside the `<6.1.0` peer range, works with `typescript-eslint@8.70.0` + `eslint@10.11.0`. |
| `typescript@7.0.2` | ❌ Only if you drop `typescript-eslint` entirely and lint with **`oxlint@1.83.0`** (Rust-based, doesn't use the TS API). Defensible for a greenfield project that wants speed, but it's a second unusual bet stacked on an already-unusual stack. Not worth it in Phase 1. |
## Tauri v2 IPC Patterns
### The three mechanisms
| Mechanism | v2 API | Use for | Don't use for |
|-----------|--------|---------|---------------|
| **Command** | `#[tauri::command] async fn` + `invoke()` | Request/response: load a page of commits, get one diff, stage a hunk | Returning 50k commits in one shot |
| **Channel** | `tauri::ipc::Channel<T>` | **Streaming**: incremental commit batches, long `git clone`/`fetch` progress | Small one-off results (overkill) |
| **Event** | `Emitter::emit` / `listen()` | Broadcast notifications: "repo changed on disk", "background fetch done" | Bulk data — events serialize to JSON *and* evaluate JavaScript per emit |
### ⭐ Recommended pattern for 50k–100k commits: paginated commands + channel for the tail
#[tauri::command]
#[tauri::command]
### Binary escape hatch (only if JSON profiles badly)
#[tauri::command]
### Error handling pattern (required boilerplate)
#[derive(Debug, thiserror::Error)]
### ⚠️ v1 → v2 API differences (most common source of wrong info)
| Task | Tauri **v1** (wrong) | Tauri **v2** (correct) |
|------|---------------------|------------------------|
| Import `invoke` | `@tauri-apps/api/tauri` | **`@tauri-apps/api/core`** |
| Open a file/URL externally | `shell.open()` | **`@tauri-apps/plugin-opener`** |
| Permissions | `tauri.conf.json` → `allowlist` | **Capability files** in `src-tauri/capabilities/*.json` |
| Streaming | events only | **`tauri::ipc::Channel`** |
| Dialog/fs/store | built into core API | **separate plugins** |
| Window type | `WebviewWindow` from `window` module | **`WebviewWindow`** from `webviewWindow` |
## React Side
### Virtualization — ✅ TanStack Virtual
| Library | Version | Verdict |
|---------|---------|---------|
| **`@tanstack/react-virtual`** | **3.14.13** | ✅ **Recommended** |
| `react-window` | 2.3.1 | Viable runner-up |
| `react-virtuoso` | 4.18.14 | ❌ Wrong fit here |
### State management — ✅ Zustand + TanStack Query
| Library | Version | Role |
|---------|---------|------|
| **`zustand`** | **5.0.15** | UI state: selected commit SHA, active pane, diff view mode (unified/split), sidebar collapsed, whitespace toggle |
| **`@tanstack/react-query`** | **5.103.1** | Server-ish state: everything that comes from Rust |
- **Caching diffs by commit SHA** — this is literally a requirement (§3.1 item 4). `queryKey: ['diff', sha, path]` with a long `staleTime` *is* the diff cache. Immutable git objects mean `staleTime: Infinity` is correct.
- **`useInfiniteQuery`** maps directly onto `git log --skip/--max-count` pagination.
- **Invalidation** — after commit/checkout/fetch, `invalidateQueries(['status'])` refreshes exactly the right panels.
- **Deduplication** — rapid arrow-key scrubbing through commits won't fire duplicate `git diff` processes.
### Supporting frontend libraries
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| `react-resizable-panels` | **4.13.1** | Three-pane resizable layout | Directly satisfies "kích thước kéo được và được ghi nhớ". Has `onLayout` + `autoSaveId` — persist via `tauri-plugin-store`. Actively maintained (updated 2026-09-20). Prefer over `allotment@1.20.5` (last touched 2025-12). |
| `tailwindcss` + `@tailwindcss/vite` | **4.3.3** | Styling | v4 uses the Vite plugin, **not** a PostCSS config — v3 tutorials are wrong. Optional but keeps a custom-designed UI tractable. |
| `clsx` | **2.1.1** | Conditional classes | Trivial, universal. |
### Build tooling
## Diff Viewer — ✅ CodeMirror 6 + `@codemirror/merge`
- `MergeView({ a: {...}, b: {...}, parent })` → **split view** ✅
- `unifiedMergeView({ original })` as an EditorView extension → **unified view** ✅
### Packages to install
| Package | Version | Required? |
|---------|---------|-----------|
| `@codemirror/state` | **6.7.5** | ✅ Core |
| `@codemirror/view` | **6.43.12** | ✅ Core |
| `@codemirror/merge` | **6.12.2** | ✅ The diff UI |
| `@codemirror/language` | **6.12.4** | ✅ Syntax highlighting infra |
| `@codemirror/commands` | **6.11.1** | ✅ Keymap/history |
| `@codemirror/search` | 6.7.2 | ⏳ In-diff search |
| `@codemirror/theme-one-dark` | 6.1.3 | ⏳ Or write your own theme |
| `@codemirror/lang-*` | varies | ✅ Per-language, **lazy-load** |
### Alternatives (rejected)
| Alternative | Version | Why not |
|-------------|---------|---------|
| Monaco `DiffEditor` | `monaco-editor` 0.56.0 | Excellent diff editor, but ~5 MB+ and a heavy worker-based architecture. Directly conflicts with the <20MB install / <150MB RAM / <1s startup constraints. Also `@monaco-editor/react` (4.7.0) hasn't been updated since 2025-11. |
| `shiki` + custom renderer | 4.4.3 | Beautiful highlighting, but you'd hand-roll virtualization, selection, and hunk UI — exactly the "trình xem diff tự viết bị lỗi hiển thị" risk the research doc calls out. Reasonable *later* for a read-only fast path. |
## Graph Rendering — ✅ Canvas (with SVG as the fallback)
### Rationale
| Approach | At ~50 visible rows × ~10 lanes | Verdict |
|----------|-------------------------------|---------|
| **Canvas 2D** | ~500 line segments + ~50 arcs per frame. Trivial for canvas. One DOM node total. | ✅ **Recommended** |
| SVG | ~500 DOM elements created/destroyed per scroll frame. Works, but GC pressure and style recalc are real at 60fps in a WebView (and WebView2 ≠ Chrome perf). | ⚠️ Acceptable fallback |
| WebGL | Massively overkill. Hundreds of lines is nothing. Adds shader complexity, context-loss handling, text-rendering pain. | ❌ No |
### The alignment pattern (the thing that actually matters)
## Testing + CI
### Rust
| Crate | Version | Purpose |
|-------|---------|---------|
| built-in `#[test]` / `#[tokio::test]` | — | Parser unit tests. **The parsers are the highest-value tests in this project** — pure functions from `&[u8]` → structs. |
| `insta` | **1.48.0** | Snapshot testing. Ideal for lane-assignment output: capture a `GraphRow` vec for a known repo shape, diff on change. |
| `tempfile` | **3.27.0** | Create throwaway repos: `git init`, scripted commits, assert parser output. |
| `criterion` | **0.8.2** | Benchmark lane assignment + log parsing at 100k commits. **Directly guards the Core Value** — make this a CI job, not an afterthought. |
| `proptest` | 1.11.0 | ⏳ Optional. Good for the hunk-patch reconstructor (§5.5), where off-by-one in `@@` headers is the classic bug. |
### Frontend
| Tool | Version | Notes |
|------|---------|-------|
| `vitest` | **5.0.1** | ⚠️ **Requires Node `^22.12 \|\| ^24 \|\| >=26`.** Your Node 22.16 is fine. Peers `vite: ^6.4 \|\| ^7 \|\| ^8` — compatible with Vite 8. |
| `@testing-library/react` | **16.3.3** | Component tests. |
| `happy-dom` | **20.14.5** | Faster than `jsdom@30.1.0`; either works. |
### E2E — ⚠️ the weakest link
| Option | Verdict |
|--------|---------|
| **`@wdio/tauri-service` 1.4.0** + `webdriverio` 9.32.0 | ✅ **Best current option.** Runs an *embedded* WebDriver server inside the app (via `tauri-plugin-wdio-webdriver`), giving Windows + Linux + **macOS**. Actively maintained (2026-09-06). |
| Official `tauri-driver` | ⏳ Fine if you only gate CI on Windows + Linux. |
| CrabNebula's driver | ❌ macOS support requires a paid `CN_API_KEY`. Wrong for an MIT OSS project. |
### GitHub Actions
### Code signing per platform
| Platform | Requirement | v1 recommendation |
|----------|-------------|-------------------|
| **Windows** | **EV cert is NOT required.** Microsoft removed EV's preferred status in its Trusted Root Program (2024); OV and EV now build SmartScreen reputation identically. **Azure Trusted Signing** (~$10/mo) is the cheapest credible path. Unsigned still *runs* — users just click through a SmartScreen warning. | ⏳ Ship unsigned for v1, document the warning in the README. Add Azure Trusted Signing when there are real users. |
| **macOS** | Apple Developer Program **$99/yr**, hard requirement. Unsigned+unnotarized DMGs are blocked by Gatekeeper (not just warned — *blocked*). Needs `APPLE_CERTIFICATE`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` secrets. | ⏳ Defer. Document the `xattr -d com.apple.quarantine` workaround. |
| **Linux** | Nothing required. AppImage + deb ship unsigned normally. | ✅ Free. |
## Windows-Specific Setup (dev machine is Windows 11 Pro)
### Windows runtime gotchas for a git GUI
- **Hide the console window.** `tokio::process::Command` on Windows flashes a black `cmd` window per spawn. With ~10 git calls on repo open, this is very visible. Fix:
- **Long paths.** Windows `MAX_PATH` (260) still bites on deep `node_modules`. Use `PathBuf` throughout and normalize separators only at the IPC boundary (as PROJECT.md's risk table already says).
- **CRLF.** `git` may return `\r\n`. Trim `\r` when splitting on `\n` in the parsers — otherwise SHAs silently carry a trailing `\r`.
- **Non-UTF-8 output.** Reinforces the `bstr` recommendation above.
## Installation
# Scaffold (create-tauri-app 4.7.4)
# --- Frontend: core ---
# --- Frontend: state + virtualization + layout ---
# --- Frontend: diff viewer ---
# language modes as needed, lazy-loaded:
# --- Dev dependencies ---
# NOTE: typescript@6, NOT 7 — see the TypeScript section.
# src-tauri/Cargo.toml
## Alternatives Considered
| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `@tanstack/react-virtual` 3.14.13 | `react-window` 2.3.1 | If you prefer a component API and are willing to solve graph alignment separately. Genuinely maintained (v2.3.1, 2026-09). |
| Canvas graph | SVG graph | If debuggability beats raw perf, or lane counts stay tiny. Keep the renderer behind an interface either way. |
| `@codemirror/merge` | `shiki` + custom hunk renderer | If CM's re-diffing of full documents proves too slow on large files. Read-only only. |
| `@codemirror/merge` | Monaco `DiffEditor` | Only if you abandon the <20MB / <1s startup constraints. You shouldn't. |
| `typescript@6.0.3` | `typescript@7.0.2` + `oxlint@1.83.0` | If build speed matters more than the mature ESLint ecosystem. Revisit after TS 7.1. |
| Zustand 5.0.15 | Jotai 3.0.0 | If UI state turns out highly granular and re-render-sensitive. Wait for 3.x to settle. |
| Paginated commands | `tauri::ipc::Channel` streaming | When first-paint latency (not total throughput) is the measured bottleneck. |
| JSON IPC | `tauri::ipc::Response` (raw bytes) | Phase 2 optimization for packed numeric graph-lane data specifically. |
| `keyring` crate | — | No good alternative. Stronghold is not equivalent. |
## What NOT to Use
| Avoid | Why | Use Instead |
|-------|-----|-------------|
| **`tauri-plugin-shell`** | Exists for **JS** to spawn processes. Your git calls are in Rust. Adds permission-scope complexity and risks moving argv control into the webview. | `tokio::process::Command` directly in Rust |
| **`tauri-plugin-stronghold`** | Encrypted DB file, **not** the OS keychain. Needs a master password, which recreates the problem. Only 370k downloads → lightly exercised. Violates the stated security constraint. | `keyring` crate 4.2.0 |
| **`tauri-plugin-keyring`** (community) | 0.1.0, last published **2024-12-23**, 44k downloads. Abandoned. | `keyring` crate 4.2.0 |
| **`typescript@7.0.2`** | `typescript-eslint@8.70.0` peers `typescript: >=4.8.4 <6.1.0` — TS 7 is excluded. No stable programmatic API until 7.1. | `typescript@6.0.3` |
| **`tauri-plugin-fs`** | Exposes filesystem to JS. All your file access is in Rust. Pure attack surface. | `std::fs` / `tokio::fs` in Rust |
| **`anyhow`** | Great for applications, wrong here: Tauri commands need `E: Serialize`, and `anyhow::Error` isn't. You also want *typed* errors so the UI can distinguish "not a repo" from "merge conflict". | `thiserror` 2.0.20 + hand-written `Serialize` |
| **`libgit2` / `git2` / `gitoxide`** | Explicitly excluded by project constraints. | `git` CLI |
| **`patch` crate (0.7.0)** | Unified-diff parser, but **last published 2022-12-28**. Effectively abandoned. | Hand-write the parser (~150–250 lines per the research doc) |
| **`unidiff` (0.4.1)** | Only ~0.9M lifetime downloads — negligible adoption for something on your critical path. | Hand-write the parser |
| **`similar` / `imara-diff` / `diffy`** | These *compute* diffs. You don't need to — `git diff` already did. Pure dead weight. | `git diff --patch` output |
| **`chrono`** | You get Unix timestamps (`%at`/`%ct`) from git as integers. Pass the raw i64 across IPC and format with JS `Intl.DateTimeFormat`, which handles the user's locale and timezone correctly for free. | `i64` + `Intl.DateTimeFormat` |
| **`rayon`** | Lane assignment is inherently **sequential** (row N depends on N-1). Parsing is I/O-bound. No parallelism to extract. | `tokio` |
| **`dashmap`** | Concurrent hashmap for a problem you don't have. Your cache is behind one lock. | `lru` + `parking_lot::Mutex` |
| **`codemirror` meta-package** | Bundles `basicSetup` (autocomplete, brackets, history) — all useless for a read-only diff viewer, and it costs startup time. | Compose individual `@codemirror/*` packages |
| **`@tauri-apps/api/tauri` import path** | **Tauri v1.** Does not exist in v2. | `@tauri-apps/api/core` |
| **`allowlist` in tauri.conf.json** | **Tauri v1.** v2 uses capability files. | `src-tauri/capabilities/*.json` |
| **`libwebkit2gtk-4.0-dev`** in CI | **Tauri v1.** v2 needs 4.1. Top Linux CI failure. | `libwebkit2gtk-4.1-dev` |
| **`ubuntu-latest`** in CI | Newer glibc → AppImage won't run on older distros. | `ubuntu-22.04` |
| **Events for bulk data** | 100k `emit` calls = 100k JS evaluations. Locks the webview. | Paginated commands, or `Channel` |
| **CrabNebula tauri-driver** (macOS) | Requires a paid `CN_API_KEY`. | `@wdio/tauri-service` 1.4.0 |
## Stack Patterns by Variant
- Switch `load_commits` from a plain command to `tauri::ipc::Channel<CommitBatch>`
- Because it lets React render the first 500 rows while Rust still parses the rest — attacks *perceived* latency, which is what the Core Value actually measures.
- Return graph-lane data via `tauri::ipc::Response` as a packed `Uint16Array`/`Uint8Array`
- Because `GraphRow` is fixed-width numeric data — the worst possible fit for JSON and the best possible fit for a typed array feeding a canvas.
- Render hunks yourself with CodeMirror decorations from git's own diff output
- Because git already computed the diff; making CM re-diff two 10k-line documents is redundant work.
- Fall back to SVG rendered from the same `virtualItems` array
- Because at ~50 visible rows the DOM cost is survivable, and devtools inspection speeds up development. Keep the renderer behind an interface so this stays a one-file change.
- Detect the `keyring` error and disable AI features with an explicit message
- Because falling back to a plaintext file would violate a stated project constraint.
## Version Compatibility
| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `tauri` 2.11.6 (crate) | `@tauri-apps/api` 2.11.1 | Keep crate and npm API in the same minor line. |
| `@tauri-apps/cli` 2.11.5 | `tauri` 2.11.6 | CLI may run slightly ahead; fine within 2.x. |
| Tauri plugins | Rust crate ≠ npm version | e.g. `tauri-plugin-store` 2.4.5 / `@tauri-apps/plugin-store` 2.4.5 happen to match, but **do not assume**. Check each. |
| `vite` 8.3.0 | Node `^20.19 \|\| >=22.12` | ✅ Node 22.16 OK |
| `vitest` 5.0.1 | Node `^22.12 \|\| ^24 \|\| >=26`; `vite ^6.4 \|\| ^7 \|\| ^8` | ✅ Node 22.16 + Vite 8 OK |
| `@vitejs/plugin-react` 6.1.1 | `vite ^8.0.0` **only** | Hard-locks you to Vite 8. |
| `typescript-eslint` 8.70.0 | `typescript >=4.8.4 <6.1.0` | 🔴 **Excludes TS 7.** The reason for pinning TS 6.0.3. |
| `eslint` 10.11.0 | `typescript-eslint` 8.70.0 | Peer allows `^8.57 \|\| ^9 \|\| ^10`. ✅ |
| `@tanstack/react-virtual` 3.14.13 | React 16.8–19 | ✅ React 19.3 OK |
| `@codemirror/*` | All 6.x, mutually compatible | CM6 keeps packages independently versioned but API-stable. |
| `keyring` 4.2.0 | default `v1` feature | Covers Windows Credential Manager + macOS Keychain + Linux Secret Service. |
| `thiserror` 2.x | — | v2 is source-compatible with v1 for common usage; don't mix. |
| `libwebkit2gtk` **4.1** | Tauri **v2** | v1 used 4.0. |
## Confidence Summary
| Area | Confidence | Basis |
|------|------------|-------|
| All version numbers | **HIGH** | Queried npm registry + crates.io API live on 2026-09-21 |
| Tauri plugin official/unofficial status | **HIGH** | Official Tauri v2 plugin index |
| No official keychain plugin → use `keyring` | **HIGH** | Enumerated all 30 official plugins; verified `keyring` 4.2.0 `v1` feature manifest |
| `keyring` v4 exact API surface | **MEDIUM** | v4 restructured from v3; verify import paths against docs.rs at implementation time |
| TS 7 breaks `typescript-eslint` | **HIGH** | Peer range read directly from npm |
| Channel vs command vs event semantics | **HIGH** | Official Tauri v2 docs + `IpcResponse` docs.rs |
| Channel messages are JSON (not binary) | **HIGH** | `impl<T> IpcResponse for T where T: Serialize` blanket impl |
| TanStack Virtual over react-window | **MEDIUM-HIGH** | Judgement call; grounded in the project's own alignment requirement |
| Canvas over SVG for the graph | **MEDIUM-HIGH** | Reasoning + `gitlanes` same-stack precedent (32k commits, ~300ms, 60fps) |
| `@codemirror/merge` covers split + unified | **HIGH** | Read from package README source |
| E2E / tauri-driver macOS story | **MEDIUM** | Ecosystem genuinely in flux; multiple competing solutions |
| Windows prerequisites | **HIGH** | Official Tauri v2 prerequisites page |
| Code signing requirements | **MEDIUM-HIGH** | Official Tauri signing docs |
## Sources
- **npm registry API** (`npm view`, `registry.npmjs.org`) — queried 2026-09-21 for every JS version cited, including peer-dependency ranges
- **crates.io API** (`crates.io/api/v1/crates/*`) — queried 2026-09-21 for every Rust version cited, plus the `keyring` 4.2.0 feature manifest
- https://v2.tauri.app/develop/calling-rust/ — commands, async, `ipc::Response`, `ipc::Channel`, error serialization
- https://v2.tauri.app/develop/calling-frontend/ — events vs channels, explicit streaming guidance
- https://docs.rs/tauri/latest/tauri/ipc/struct.Channel.html — `IpcResponse` blanket impl (proves channels are JSON)
- https://v2.tauri.app/plugin/ — complete official plugin list (confirms no keychain plugin)
- https://v2.tauri.app/start/prerequisites/ — Windows MSVC / WebView2 / rustup-msvc
- https://v2.tauri.app/distribute/pipelines/github/ — CI matrix, `libwebkit2gtk-4.1-dev`, tauri-action@v1
- https://v2.tauri.app/distribute/sign/windows/ — EV no longer required, Azure Trusted Signing
- https://v2.tauri.app/develop/tests/webdriver/ — tauri-driver Windows/Linux only
- https://webdriver.io/docs/desktop-testing/tauri/platform-support/ — `@wdio/tauri-service` embedded WebDriver, macOS support
- https://raw.githubusercontent.com/codemirror/merge/main/README.md — `MergeView` (split) + `unifiedMergeView` (unified)
- https://github.com/nobel6018/gitlanes — same-stack (Tauri 2 + React) canvas commit graph; 32k commits ~300ms, 60fps
- https://github.com/open-source-cooperative/keyring-rs — keyring v4 restructure, platform stores
- https://www.infoq.com/news/2026/08/typescript-7-released/ — TS 7.0 GA, missing programmatic API until 7.1 — MEDIUM confidence
- Project-internal: `.planning/PROJECT.md`, `docs/01-research-competitors.md` — constraints, lane algorithm, git commands, risk table
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, `.github/skills/`, or `.codex/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
