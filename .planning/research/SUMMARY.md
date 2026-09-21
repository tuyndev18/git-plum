# Project Research Summary

**Project:** git-plum — cross-platform open-source Git GUI desktop client
**Domain:** Desktop developer tool (Tauri v2 + React/TypeScript frontend, Rust backend shelling out to the `git` CLI)
**Researched:** 2026-09-21
**Confidence:** HIGH on stack and pitfalls, HIGH on feature evidence, MEDIUM-HIGH on architecture
**Inputs:** [STACK.md](STACK.md) · [FEATURES.md](FEATURES.md) · [ARCHITECTURE.md](ARCHITECTURE.md) · [PITFALLS.md](PITFALLS.md) · [docs/01-research-competitors.md](../../docs/01-research-competitors.md)

---

## 1. Executive Summary

- **The platform bet is sound and needs no revisiting.** Tauri v2 + React + Rust + `git` CLI is validated by a same-stack precedent: `gitlanes` (Tauri 2 + React, canvas graph + virtual scrolling) loads a **32k-commit repo in ~300ms at 60fps**. The Core Value ("graph opens in under 1 second") is achievable with the planned architecture — paginated commands, lane computation in Rust, canvas rendering inside a single virtualized scroll container.
- **The single most important finding is that the merge conflict resolver is a hidden table stake, not a v2 feature.** SourceGit closed issue **#892** with verbatim the reasoning in PROJECT.md, held that position for 13 months under user pressure, and reversed via PR **#2070** — merged in **under four hours**. See §2.1.
- **PROJECT.md's stated security constraint is currently unimplementable as assumed.** There is **no official Tauri v2 keychain plugin**, and the obvious candidate (`tauri-plugin-stronghold`) would *violate* the constraint rather than satisfy it. The `keyring` crate 4.2.0 is the answer. See §2.2. Note this also contradicts ARCHITECTURE.md, which suggests stronghold as an acceptable option.
- **Eight items appear to be scope oversights, not deliberate exclusions** — they are absent from both the Active list and the Out-of-Scope list. Chief among them: commit/history search, merge/rebase continue-abort-skip, git command log, reset-to-commit, discard safety net. See §2.3.
- **Four things must be built in Phase 1 or become expensive retrofits:** the git environment-pinning wrapper, the command registry, per-repo-keyed state on both sides of the IPC boundary, and a per-repo write-serialization mutex. Each has documented evidence of being painful to add later. See §5.
- **Hunk staging is the most under-estimated item in the plan.** `docs/01` §5.5 gives it a paragraph; FEATURES.md rates it **HIGH** complexity with a hard correctness rule: patches must be read and written as **raw bytes, never a decoded string**, or CRLF and non-UTF-8 content corrupt silently. See §2.4.
- **The 11–14 week estimate in `docs/01` §8 is under-budgeted** and does not include any of the recommended promotions. See §2.5.
- **Two pinned technology decisions are counter-intuitive and must not be "corrected" later:** TypeScript **6.0.3, not 7.0.2** (typescript-eslint peers `<6.1.0`), and **do not add `tauri-plugin-shell`** (git spawning belongs in Rust).
- **Timing is favorable.** SourceGit — the closest open-source competitor — currently has an open issue titled *"[Announcement] Find volunteers to take over the project."* The open-source niche is actively destabilizing.
- **Linux is a structural advantage, not a checkbox.** GitHub Desktop issue **r4851 "GitHub Desktop for Linux?"** is the **single highest-reaction issue in either tracker, by 3.5x**, and the non-availability of Fork on Linux is its most-cited weakness. Do not treat Linux as an afterthought in CI or testing.
- **The biggest project risk is not technical, it is discipline.** PITFALLS.md #10 and `docs/01` §7 independently name scope creep / infrastructure polish as the #1 killer. The mitigation is procedural: dogfood gates between phases, not a code fix.

---

## 2. Challenges to PROJECT.md

**This is the section that most needs a decision before the roadmap is written.** Each item below is a place where research contradicts a decision already recorded in PROJECT.md. They are presented as challenges, not as resolved edits.

### 2.1 Merge conflict resolver: currently Out of Scope, should be in v1

| | |
|---|---|
| **PROJECT.md says** | Out of Scope: *"Trình giải quyết xung đột merge dạng đồ hoạ — Là một sản phẩm con riêng biệt. v1 chỉ phát hiện xung đột, liệt kê tệp, và mở trình soạn thảo bên ngoài."* Key Decisions row: *"v1 chỉ phát hiện xung đột merge, không giải quyết."* |
| **Research found** | SourceGit issue **#892** ("[feature suggestion] Integrated merge-tool", 10 reactions) was closed **2025-12-29** by the maintainer with: *"There's no plan to implement a integrated merge-tool… it's very hard to implement a full-featured merge-tool (text editor)"* and *"After configuring an external merge tool… its usage is the same as calling the integrated editor."* A user rebutted: *"Unfortunately, this is not true. Showing actual differences instead of short 'conflict detected' is much more convenient."* Users pressed for 13 months. On **2026-01-26** PR **#2070 "Built-in merge conflict solver"** was opened and **merged in under four hours**. Follow-up issue **#2168** (10 reactions, open) shows users then wanted it *better*, not removed. |
| **Evidence strength** | **HIGH.** This is a controlled natural experiment on the identical argument, run by the nearest competitor, with a documented reversal. Corroborated by reviewers listing conflict resolution among the handful of genuinely daily-use GUI features, and by it being the specific thing users say they miss moving from GitKraken to Fork. |
| **The reasoning error** | PROJECT.md conflates two different products. A **merge tool** (three-pane BASE/LOCAL/REMOTE text editor with its own diff engine) genuinely is a sub-product — correctly out of scope, and arguably a bad design anyway (eseth.org argues 3-way tools do a "blind diff" and discard the work git already did). A **conflict resolver** (two-way view over the conflict markers git already wrote, with accept-ours / accept-theirs / accept-both per block, then write file + `git add`) is *a diff view with three buttons*. |
| **Recommended change** | Promote a **two-way conflict resolver** into v1. Set `merge.conflictStyle=zdiff3`, consider `mergetool.hideResolved` (git ≥2.31) so only genuine conflicts surface. Keep out of v1: free-text editing in the conflict view (that *is* the text-editor sub-product — SourceGit #2168) and semantic/binary merging. Keep "open in external tool" as an escape hatch. Never auto-pick a side for binary files (SourceGit shipped that bug). |
| **Cost** | MEDIUM, and mostly already paid. Dependencies: the diff viewer (built in the Diff phase), the conflicted-file list (already in MVP), and the merge in-progress state machine (required by merge/rebase regardless). The incremental cost is per-block accept actions + writing the resolved file. |
| **Benefit** | Conflicts are the moment of maximum user stress. "Conflict detected in 3 files, go use another program" is precisely when a tool feels like a toy. |
| **Residual risk** | The four-hour merge was in a mature Avalonia codebase with an existing text editor component. Effort **in our CodeMirror 6 stack is unverified** — FEATURES.md explicitly flags this as needing a phase-level spike. |

### 2.2 OS keychain: the stated constraint has no official Tauri path, and the obvious plugin violates it

| | |
|---|---|
| **PROJECT.md says** | Constraint: *"Khoá API của nhà cung cấp AI lưu trong keychain hệ điều hành, không bao giờ ghi vào tệp cấu hình dạng văn bản thuần."* Active requirement: *"Lưu khoá API vào keychain của hệ điều hành."* |
| **Research found** | STACK.md enumerated all 30 official Tauri v2 plugins: **there is no keychain/credential plugin.** Two traps: (a) `tauri-plugin-stronghold` (2.3.2) is an IOTA-derived **encrypted database file**, not the OS keychain — it requires a master password *you* must obtain from the user, which means either prompting on every launch or storing that password somewhere, **recreating the exact problem the constraint exists to solve**; its 370k downloads vs the dialog plugin's 16M indicates how lightly it is exercised. (b) The community `tauri-plugin-keyring` is 0.1.0, last published **2024-12-23**, abandoned. |
| **Evidence strength** | **HIGH** on "no official plugin" and on the `keyring` 4.2.0 feature manifest (read from the crates.io API). **MEDIUM** on the exact v4 API surface. |
| **Inter-document conflict** | ARCHITECTURE.md's Integration Points table says *"Use a Tauri keychain/stronghold-style plugin (`keyring` crate on the Rust side, **or Tauri's stronghold plugin**)."* **STACK.md is right and ARCHITECTURE.md is wrong here.** Treat STACK.md as authoritative. |
| **Recommended change** | Use the **`keyring` crate 4.2.0** directly in Rust. Its default `v1` feature resolves to exactly the three backends needed (macOS Keychain, Windows Credential Manager, Linux Secret Service) with no feature tinkering. Wrap in two thin Tauri commands (`set_api_key`, `get_api_key`) and **never send the key to the frontend** — the AI provider call happens in Rust. Add an explicit statement to PROJECT.md that stronghold is rejected and why, so it is not re-proposed later. |
| **Cost** | Near zero — one crate, two commands. The real cost is the **degradation path**: Linux headless/CI/minimal-WM has no Secret Service and `keyring` will error there. Requirement: disable AI features with a clear message; **never** fall back to a plaintext file, as that would violate the constraint. |
| **Benefit** | The constraint becomes actually satisfiable rather than aspirational. Avoids shipping stronghold and discovering at release that it needs a master password. |
| **Action** | Verify import paths against docs.rs/keyring/4.2.0 at implementation time — v4 restructured heavily from v3 and most existing documentation describes the v3 `Entry::new(service, user)` API. |

### 2.3 Eight items missing from Active that appear to be oversights, not exclusions

None of these are on the Out-of-Scope list, so none is a deliberate cut. FEATURES.md rates each as a gap.

| Missing item | Evidence | Complexity | Why it matters |
|---|---|---|---|
| **Commit / history search** | GitHub Desktop r96 "Commit Searchability and Navigation", r20, r39; **27 search-related issues in SourceGit** (all refinements — meaning the feature is used constantly) | MEDIUM | A 100k-commit graph you cannot search is a browsing toy, not a tool. Minimum bar: filter by message / author / SHA. Pickaxe (`-S`/`-G`) can wait for v1.x. |
| **Merge / rebase / cherry-pick: continue, abort, skip** | SourceGit state-machine bugs, e.g. "[BUG] Unstaging all Staged file-changes resets Conflicts and exits Merging state!" | MEDIUM | **Without abort, a user who starts a merge in git-plum can only escape via the CLI.** This is required even under the current "detect conflicts only" plan, and is currently un-budgeted. Requires detecting `.git/MERGE_HEAD`, `rebase-merge/`, `CHERRY_PICK_HEAD` on open *and on app restart*. |
| **Git command log (show the argv)** | SourceGit ships it and has a request to attach the command to every operation; Sublime Merge praised for "constructs command lines for you… rather than magical"; danmackinlay names the core critique of GitKraken as "the relationship to the git command line is opaque" | **LOW** | **Highest ROI item in the entire research.** We already build an argv and spawn a process — recording and displaying it is near-free, and it converts our architecture into a user-visible trust feature. Direct antidote to the "hiding Git" anti-feature. Caveat: redact credentials from logged remote URLs. |
| **Reset branch to commit (soft/mixed/hard)** | Universal in competitors; SourceGit lists Reset as a headline operation | LOW | The universal recovery operation. Without it a user cannot undo a local commit. Trivial given branch operations already exist. |
| **Auto-updater** | SourceGit r14 "Feature request: add an updater" (13 comments) — **2nd-highest open request** | MEDIUM | For a downloaded open-source desktop app, no updater means users stay on the version they installed forever. `docs/01` §8 phase 6 budgets it; PROJECT.md Active does not list it. Make it opt-out-able (GH Desktop r105). Tauri's updater uses its own minisign keypair — **no paid code-signing certificate required**. |
| **Command registry** (underpinning palette + customizable shortcuts) | The most-praised trait of Sublime Merge; SourceGit closed "[Feature] Command Palette" (r6) and has two open issues asking for customizable shortcuts | MEDIUM | **Architectural, phase 1.** If UI actions are inline click handlers, a palette later requires refactoring every one. If they are registered commands from day one, palette + customizable shortcuts + command log all fall out nearly free. Retrofitting is the reason SourceGit still lacks them. |
| **Binary / large-file / LFS-pointer handling in diff** | GH Desktop r27 "show image previews for images tracked by lfs", r14 "Display SVG file and diff not as plaintext" | LOW-MEDIUM | The LFS exclusion in PROJECT.md is *mostly* right ("call the CLI and LFS works"), but LFS **pointer files diff as meaningless text blobs**. Minimum v1 bar: size threshold before attempting a diff, binary detection, and a graceful "binary file / LFS object, N bytes" placeholder. Without it a single 200MB file freezes the app. |
| **Discard safety net (auto-stash before discard)** | The most-quoted GitKraken testimonial is *"accidentally discarded all my changes but now they are back again"*; Atom/GitHub issue #1001 shipped a discard that deleted user files; Tower is singled out for CMD+Z on a wrongful discard | LOW | The Out-of-Scope "Undo/Redo" bullet **bundles two different mechanisms**. Reflog-based undo covers ref moves — genuinely hard, correctly deferred. But **discard has no reflog entry and no recovery path**, and v1 ships discard-by-file *and* discard-by-hunk. Recommended: before any discard, `git stash create` the affected paths and keep the object reachable under a dedicated trash ref namespace, plus a "Recently discarded" restore list. One git command and one ref write. For untracked files the verb must be **"Delete"**, not "Discard" — the Atom lesson. |

**Additional near-gaps worth an explicit decision (not oversights, but ambiguous):**

- **Multi-repo tabs.** `docs/01` §2.4 item 1 marks tabs as MVP; PROJECT.md Active commits only to *"nhớ danh sách repo gần đây"*. Clarify which. The ARCHITECTURE.md advice holds either way — key state by `repo_id` from day one (see §5).
- **"Open in Terminal / Editor / File Manager", configurable.** PROJECT.md correctly cuts the *embedded* terminal, and the evidence for that cut is unusually clean: across ~20 SourceGit terminal issues, **every one** asks for launching an *external* terminal; **zero** ask for an embedded emulator. But the external launcher itself is not in PROJECT.md. It costs hours and satisfies 100% of observed demand. Recommend: cut the embedded terminal **permanently** (not "v2"), and add the launcher to v1.
- **Explicit submodule non-goal.** PROJECT.md implies submodules "just work" via the CLI. True for *correctness*, false for *UX* — **82 submodule issues in GitHub Desktop alone**, including a 73-comment thread on branch switching. Recommend an explicit stated boundary: display submodule entries read-only, never manage them, never hang the UI on a submodule fetch.
- **Squash / fixup / reword of a non-HEAD commit.** Deferring the drag-and-drop interactive rebase editor is correct (the 22 interactive-rebase issues in SourceGit are overwhelmingly bugs). Deferring *all* history editing is not — SourceGit closed "Support for `git commit --fixup` and `git rebase -i --autosquash`" at **r9 with 14 comments**.
- **Blame.** PROJECT.md defers it as not being in the daily workflow. Partly right — it is in the *investigation* loop, which is half of why people open a GUI. GH Desktop has four separate blame requests (top at r42); SourceGit has 13 blame issues that are almost all *refinement bugs*, the signature of a constantly-used feature. Cost is low and **the dependency (file history viewer) is already in MVP**. Recommend: v1 polish phase if schedule holds, read-only gutter only.

### 2.4 Hunk staging is rated HIGH complexity, not a paragraph

| | |
|---|---|
| **`docs/01` §5.5 says** | Roughly one paragraph: get the diff, let the user pick hunks, rebuild a patch, `git apply --cached -`. (The doc has already been patched with the blob-hash re-verification step.) |
| **Research found** | FEATURES.md rates this **HIGH** complexity, citing GitLane PR #425, which enumerates concrete shipped bugs: patches read as **text lose the trailing CR on CRLF files**; **non-UTF-8 bytes become replacement characters, silently corrupting the file on stage**; **clean-filter warnings on stdout get parsed as patch content**, causing permanent false "stale" detection. Sourcetree shipped SRCTREE-3047 (`git apply` refuses whitespace-only reverts). Magit has a long-standing issue (#3172) where non-ASCII content in a hunk causes "patch does not apply". PITFALLS.md #4 adds the stale-diff race as a separate critical pitfall. |
| **Evidence strength** | **HIGH** — these are shipped bugs in real competitors, not speculation. |
| **The hard correctness rule** | **Read and write patches as raw BYTES, never as a decoded string.** Also: pass `--recount`; handle `core.autocrlf` explicitly; **isolate stderr from stdout** so filter warnings never enter the patch. |
| **Additional required behavior** | Use `git apply --check --cached` as a dry run first, so the failure path is fast and never touches the index. On failure, **never** silently retry with fuzzy matching or `--whitespace=fix` — surface "file changed since you viewed this diff, please refresh". Silent fuzzy application stages the wrong lines, which is far worse than a visible failure for a tool whose value proposition is trustworthy history. |
| **Recommended change** | Budget **2–3x** the current estimate for the working-tree phase. Add explicit test cases: binary file, file with no trailing newline, file renamed *and* modified, CRLF/LF normalization interacting with partial staging, and the stale-diff race (edit the file between diff-open and stage-click). |
| **Related confirmation** | **Line-level staging must stay out of v1.** It is strictly harder — GitLane #425 documents that zero-context fragments anchored to worktree line numbers misalign when other pending changes exist; the fix was to emit *full hunks with unselected changes neutralized*, not minimal fragments. Even SourceGit has not made it pleasant (open r3). It is not currently in Active; keep it that way. |

### 2.5 The 11–14 week estimate is under-budgeted

| | |
|---|---|
| **`docs/01` §8 says** | 11–14 weeks to MVP for one full-time developer, with a usable read-only build at ~5–6 weeks. |
| **Research found** | Three independent pressures, from two documents. (1) FEATURES.md: the estimate "does not include the promotions recommended here, and **already under-budgets hunk staging**." (2) STACK.md implies additional unbudgeted Phase-1 work: Windows MSVC Build Tools plus rustup toolchain setup from scratch (Rust is **not yet installed** on the dev machine), Tauri v2 capability-file discipline, and the env-pinning spawn wrapper. (3) PITFALLS.md adds mandatory fixture-repo construction (non-UTF-8 filenames, octopus merges, orphan branches, 20+-lane stress cases) before the lane algorithm is written. |
| **Evidence strength** | **MEDIUM** — this is an inference from two documents, not an independently re-derived estimate. No researcher produced a revised number. |
| **Recommended change** | Do not treat 11–14 weeks as a planning constraint. Treat the **phase sequence** as the plan and the duration as an output. If a date is needed, expect the working-tree phase alone to absorb 2–3x its current 2-week allocation. |
| **What to trade if time runs short** | FEATURES.md is explicit: **AI is the droppable item.** It is a leaf dependency — nothing else needs it — its value is contested ("mildly valued, definitely not a switching reason, and actively distrusted by a vocal segment"), and it is the right thing to trade away if the conflict resolver, search, and reset need the time. Keep it in scope but **schedule it last**, and position "improve/refine my draft" at least as prominently as "generate from scratch" — that is what users actually asked for (the one AI issue in SourceGit with traction is *"Letting the AI improve the commit instead of generating it"*). Do **not** market AI as the differentiator; that puts git-plum in direct comparison with GitKraken on GitKraken terms. |

### 2.6 Where the research documents disagree with each other

Stated explicitly rather than silently harmonized:

| Topic | Conflict | Resolution |
|---|---|---|
| **Keychain** | ARCHITECTURE.md permits `tauri-plugin-stronghold`; STACK.md rejects it as violating the security constraint | **STACK.md wins.** It verified the plugin mechanism against the crates.io manifest and the official plugin index. |
| **Graph rendering: canvas vs SVG** | STACK.md recommends **canvas** (one DOM node; the `gitlanes` precedent). The Pattern 2 code sample in ARCHITECTURE.md shows per-row `<GraphCell>` SVG, then in prose recommends a single full-height SVG **or** canvas overlay | Both converge on *"one overlay driven by the same virtualItems/scrollTop"*, and both reject per-row elements and dual scroll containers. The canvas-vs-SVG choice is genuinely open. **Keep the renderer behind an interface so swapping is a one-file change.** Flag as a validation checkpoint (§6). |
| **Non-UTF-8 handling** | PITFALLS.md #3 says convert with `String::from_utf8_lossy()` at parse boundaries. STACK.md recommends the `bstr` crate to parse `Vec<u8>` and avoid decoding at all. The hunk-staging rule in FEATURES.md says **never decode** | Not actually contradictory, but the *boundary matters*: lossy conversion is acceptable for **display** fields (commit messages, author names). It is **forbidden** on the **patch path** — patches stay bytes end to end. Make this distinction explicit in the git layer API. |
| **Watcher debounce** | ARCHITECTURE.md suggests 150–300ms; PITFALLS.md #8 suggests 250–500ms | Trivial. Use 250–300ms. Both agree on the substance: watch `.git/HEAD`, `.git/index`, `.git/refs/**`, `.git/packed-refs`, and the rebase/merge state files — **never** recursively watch the working tree unfiltered. |

---

## 3. Locked Technical Decisions

Versions verified live against the npm registry and crates.io API on 2026-09-21. Confidence **HIGH** on every number below.

### Core platform

| Technology | Version | Purpose / note |
|---|---|---|
| `tauri` / `tauri-build` (crates) | **2.11.6** / **2.6.3** | Keep in the same 2.x line |
| `@tauri-apps/api` / `@tauri-apps/cli` | **2.11.1** / **2.11.5** | |
| React | **19.3.0** | |
| **TypeScript** | **6.0.3** | **NOT 7.0.2** — see the do-not-use table |
| Vite / `@vitejs/plugin-react` | **8.3.0** / **6.1.1** | plugin-react peers `vite ^8.0.0` only — hard-locks Vite 8. Node 22.16 satisfies the Vite 8 requirement of `>=22.12`. |
| Rust toolchain | **stable-msvc** | Windows: MSVC target, *not* GNU. Requires VS Build Tools 2022 "Desktop development with C++" **first** — without MSVC, `cargo build` fails at the link step with `link.exe not found`. |

### Rust crates

| Crate | Version | Why |
|---|---|---|
| `tokio` | **1.53.1** | `features = ["process","io-util","rt-multi-thread","macros","sync"]`. `tokio::process` streams `git log` stdout incrementally instead of buffering a 50MB `Output`. |
| `serde` / `serde_json` | **1.0.229** / **1.0.151** | |
| `thiserror` | **2.0.20** | v2, not v1. Tauri commands need `E: Serialize` — hand-write the `Serialize` impl in `error.rs` in Phase 1; every command returns `Result<T, GitError>`. |
| `bstr` | **1.13.1** | **The underrated pick.** git output is bytes, not guaranteed UTF-8. Avoids `from_utf8` panics and lossy corruption on real repos. |
| `memchr` | **2.8.3** | SIMD splitting on the unit/record separators and NUL across large buffers |
| `lru` | **0.18.4** | Bounded diff cache keyed by commit SHA |
| `parking_lot` | **0.12.5** | Caveat: use `std::sync` if a lock is held across `.await` — parking_lot guards are not `Send`-safe across await points |
| `tracing` | **0.1.44** | Pairs with `tauri-plugin-log` |
| **`keyring`** | **4.2.0** | Default `v1` feature covers macOS Keychain + Windows Credential Manager + Linux Secret Service. See §2.2. |

### Frontend libraries

| Library | Version | Why this one |
|---|---|---|
| `@tanstack/react-virtual` | **3.14.13** | **Headless** — `useVirtualizer` returns index/start/size and you own the DOM. This is what makes rendering the graph column and text rows from the *same* virtualItems array in the *same* scroll container possible, guaranteeing pixel alignment by construction. The binding constraint is alignment, not raw scroll speed. |
| `@tanstack/react-query` | **5.103.1** | This app state is mostly server-ish data from Rust; `invoke()` is structurally a `fetch()`. A query key of `['diff', sha, path]` with `staleTime: Infinity` **is** the diff cache (git objects are immutable). `useInfiniteQuery` maps onto `--skip`/`--max-count`. Set `retry: false` — a git error is a real error. |
| `zustand` | **5.0.15** | UI flags. Its imperative `subscribe` (outside React) feeds selection changes to the canvas renderer without re-rendering the tree — directly serves the "do not rebuild the graph on selection change" requirement. |
| `react-resizable-panels` | **4.13.1** | Has `onLayout` and `autoSaveId`; persist via `tauri-plugin-store` |
| `@codemirror/state` / `view` / `merge` / `language` / `commands` | **6.7.5** / **6.43.12** / **6.12.2** / **6.12.4** / **6.11.1** | `@codemirror/merge` provides **both** required modes: `MergeView` = split, `unifiedMergeView` = unified. No custom work needed. |
| `tailwindcss` + `@tailwindcss/vite` | **4.3.3** | v4 uses the Vite plugin, **not** a PostCSS config — v3 tutorials are wrong |
| `clsx` | **2.1.1** | |

### Tauri plugins

**Required:** `dialog` 2.7.3 (folder picker) · `store` 2.4.5 (recent repos, prefs, pane sizes — **not** API keys) · `window-state` 2.4.1 · `opener` 2.5.5 (replaces the v1 `shell.open`) · `log` 2.9.2.
**Deferred to the release phase:** `updater` 2.12.0 (plus `process` 2.3.1 for `relaunch()`) · `single-instance` 2.4.5 (nice-to-have).

### Testing

**Rust — the highest-value tests in this project:** built-in `#[test]` / `#[tokio::test]` for parsers (pure byte-slice to struct functions) · `insta` **1.48.0** for lane-assignment snapshots · `tempfile` **3.27.0** for throwaway fixture repos · `criterion` **0.8.2** benchmarking lane assignment and log parsing at 100k commits — **make this a CI job, it directly guards the Core Value** · `proptest` 1.11.0 optional for the hunk-patch reconstructor.

**Frontend:** `vitest` **5.0.1** · `@testing-library/react` **16.3.3** · `happy-dom` **20.14.5**. Mock `invoke` at the module boundary so component tests run without a Tauri runtime.

**E2E — the weakest link:** the official `tauri-driver` supports **Windows and Linux only** (macOS has no WKWebView driver). `@wdio/tauri-service` 1.4.0 runs an embedded WebDriver server and covers all three. **Do not invest in E2E in Phase 1** — put the effort into Rust parser/lane tests and criterion benchmarks, which cover the actual risk. Add smoke tests in the release-polish phase.

### CI

GitHub Actions matrix: `macos-latest` for both aarch64 and x86_64 · **`ubuntu-22.04`** · `windows-latest`; using `tauri-apps/tauri-action@v1` and `swatinem/rust-cache@v2`.

**Code signing:** Windows — an EV cert is **not required** (Microsoft removed the preferred status of EV in 2024); ship unsigned for v1 and document the SmartScreen warning; add Azure Trusted Signing (~$10/mo) when there are real users. macOS — Apple Developer Program at **$99/yr is a hard requirement** (unsigned DMGs are *blocked* by Gatekeeper, not merely warned); defer, and document the quarantine-attribute workaround. Linux — nothing required. **The Tauri updater uses its own minisign keypair, independent of OS code signing — auto-updates need no paid certificate.**

### Do NOT use

| Avoid | Reason |
|---|---|
| **`tauri-plugin-shell`** | Exists so **JavaScript** can spawn processes. Every git call originates in **Rust**. Adding it is dead weight at best; at worst someone routes git through JS, putting the full argv under webview control and forfeiting the security benefit of the architecture. It also drags in v2 capability scoping for shell commands. **Rule: if you are importing `@tauri-apps/plugin-shell`, you have taken a wrong turn.** |
| **`tauri-plugin-stronghold`** | Encrypted DB file, not the OS keychain; needs a master password, recreating the problem. **Violates the stated security constraint.** Use `keyring` 4.2.0. |
| **`tauri-plugin-keyring`** (community) | 0.1.0, last published 2024-12-23, abandoned |
| **`typescript@7.0.2`** | `typescript-eslint@8.70.0` declares a peer range of `typescript: >=4.8.4 <6.1.0` — **TS 7 is excluded.** TS 7.0 shipped without a stable programmatic compiler API (expected in 7.1), so every tool that consumes the TS API cannot run on it. Revisit after 7.1. Use `typescript@6.0.3`. |
| **`tauri-plugin-fs`** | Exposes the filesystem to JavaScript; all file access is in Rust. Pure attack surface. Use `std::fs` / `tokio::fs`. |
| **`anyhow`** | `anyhow::Error` does not impl `Serialize`, which Tauri commands require. You also want *typed* errors so the UI can distinguish "not a repo" from "merge conflict". Use `thiserror` 2.0.20. |
| **`libgit2` / `git2` / `gitoxide`** | Excluded by project constraint |
| **`patch` 0.7.0 / `unidiff` 0.4.1** | Abandoned (last published 2022-12-28) / negligible adoption, and this sits on the critical path. Hand-write the parser (~150–250 lines). |
| **`similar` / `imara-diff` / `diffy`** | These *compute* diffs. `git diff` already did. Dead weight. |
| **`chrono`** | git gives Unix timestamps as integers. Pass the raw i64 over IPC and format with JS `Intl.DateTimeFormat`, which handles locale and timezone correctly for free. |
| **`rayon`** | Lane assignment is inherently **sequential** (row N depends on N−1); parsing is I/O-bound. No parallelism to extract. |
| **`dashmap`** | Concurrent hashmap for a problem you do not have. Use `lru` plus `parking_lot::Mutex`. |
| **`codemirror` meta-package** | Bundles `basicSetup` (autocomplete, brackets, history) — useless weight for a read-only diff viewer, and it costs startup time, which is a Core Value. Compose the ~6 individual extensions you want. |
| **`@tauri-apps/api/tauri` import path** | Tauri **v1**; does not exist in v2. Use `@tauri-apps/api/core`. |
| **`allowlist` in tauri.conf.json** | Tauri **v1**. v2 uses capability files in `src-tauri/capabilities/*.json`. |
| **`libwebkit2gtk-4.0-dev`** in CI | Tauri v1 needed 4.0; **v2 needs 4.1**. The #1 Linux CI failure for people copying v1 workflows. |
| **`ubuntu-latest`** in CI | A newer glibc makes the AppImage refuse to run on older distros. Build on **`ubuntu-22.04`**. |
| **Events for bulk data** | 100k emit calls means 100k JS evaluations, which locks the webview. Use paginated commands, or a Channel. |
| **Monaco `DiffEditor`** | ~5MB+ and a heavy worker architecture — directly conflicts with the under-20MB install, under-150MB RAM, and under-1s startup constraints. |
| **`react-virtuoso`** | Optimized for *dynamic* heights and auto-measurement, which we explicitly do not want. All cost, no benefit for a fixed-row-height list. |
| **CrabNebula tauri-driver** (macOS) | Requires a paid API key — wrong for an MIT open-source project |

### Windows runtime gotchas (mandatory, not cosmetic)

- **`CREATE_NO_WINDOW` (0x08000000)** on every spawned git process, behind `#[cfg(windows)]`. Otherwise a black console window flashes per spawn — and this app spawns git dozens of times per session. **Invisible in `tauri dev`** (the child inherits the parent console); only visible in a bundled release executable launched from Explorer.
- **CRLF**: trim the carriage return when splitting on newlines, or SHAs silently carry a trailing carriage return.
- **Long paths**: MAX_PATH (260) still bites on deep `node_modules`. Use `PathBuf` throughout; normalize separators only at the IPC boundary.
- **WebView2** is already present on Windows 11 — nothing to install.

---

## 4. Architecture Shape

Compressed to what determines phase boundaries. Full detail in [ARCHITECTURE.md](ARCHITECTURE.md).

### Backend module boundaries (Rust, `src-tauri/src/`)

| Module | Responsibility | Why it is a phase seam |
|---|---|---|
| `git/exec.rs` | Spawn git via `tokio::process::Command`; arg building; **env pinning**; timeout/kill; returns raw bytes | Foundation. Every later phase calls through it. Built once in Phase 1. |
| `git/args.rs` | Safe arg-builder helpers; always a `--` separator before pathspecs | Prevents injection via branch names and paths |
| `git/parsers/*` | One pure function per git output format (log, refs, status, diff, stash): bytes in, domain structs out | **The highest-value tests in the project.** Testable with zero Tauri runtime. |
| `domain/*` | Plain structs with `Serialize`. **Knows nothing about git.** | Stable IPC contract; parsers are the only code importing both raw bytes and domain types |
| `graph/lanes.rs` | Commits in, GraphRows out. Pure, no IO, no async | The Core Value. Isolating it means a lane bug is fixed in Rust alone without touching the UI, provided the GraphRow/Edge contract holds. |
| `cache/*` | Per-repo commit-page cache, refs cache, LRU diff cache | The single seam where an on-disk cache could later replace the in-memory map without touching commands or frontend |
| `watcher/*` | A `notify` watcher on `.git` internals, debounced, emitting a coalesced repo-changed event with a change-kind hint | Knows only that something changed; never talks to parsers or domain types |
| `state/*` | Tauri managed state: open repos registry, active repo, AI config | **Must be a map keyed by RepoId from day one** (see §5) |
| `commands/*` | Thin command handlers, **one file per git domain area** | This split maps **almost 1:1 onto the six requirement groups**, which is why each phase Rust surface is purely additive and never touches prior phase files |
| `ai/*` | An `AiProvider` trait plus claude/openai/ollama implementations | Leaf. Parallelizable; disjoint files. |

### Frontend boundaries (`src/`)

`ipc/commands.ts` and `ipc/events.ts` are **the only files that call invoke/listen** — renaming a Rust command touches one frontend file. Stores are split by **data ownership, not by view** (commitsStore, refsStore, workingTreeStore, selectionStore, diffCacheStore) so the history view and detail panel read the same store without duplicate fetches. Consider `specta`/`tauri-specta` to keep TS types in sync, given the number of domain types this will accumulate.

### The three patterns that shape everything

1. **Thin command handlers, fat pure modules.** Commands do validation and orchestration only (exec, parse, cache, return). All real logic is plain Rust with no Tauri or async dependency. This is what keeps the Rust layer testable without a Tauri runtime, and keeps the "Rust is a barrier for a web-familiar team" mitigation true in practice.
2. **One scroll container, one virtualizer.** The graph column and text columns are **not** two synced scrollables. They are one virtualized list; the graph is a single overlay (canvas or SVG) positioned with the same scroll offset, redrawn from the **same virtualItems array** that renders the text. Drift becomes structurally impossible. Requires fixed row height (already a stated requirement) and requires Rust to hand over **complete per-row geometry** (lane, color, passthrough edges, out edges) so the frontend never computes connectivity.
3. **Selection state decoupled from data state.** The selection store holds only the selected commit id and file path — no commit data. Clicking a different commit is **zero IPC calls** if metadata is already paged in, and never triggers a graph recompute.

### Data flow properties that matter for phasing

- **Open repo** kicks off refs, the first commit page, and status **concurrently**; the history view renders as soon as page 1 plus graph rows arrive (the under-1s target).
- **The diff cache never needs invalidation** for historical commits — commit diffs are immutable by definition. Only LRU eviction. This is why an infinite stale time is correct.
- **Write operations return fresh state directly** rather than waiting for the watcher (whose debounce could lag), *and* emit an event defensively.
- **Lane assignment needs global context** — it cannot be computed per-page in isolation. Cache the full GraphRow vector (lane/color/edges only, a few bytes per row) in memory; page only the heavier commit metadata lazily.

### Anti-patterns to encode as review criteria

1. **Dual scroll containers with a ScrollSync wrapper** — inherent event-loop lag between the two scroll handlers is visibly jittery on fast scroll (react-virtualized#369, long-standing and unresolved). Unacceptable for a Core Value feature.
2. **Computing lanes in the frontend** — blocks the render thread and duplicates logic Rust already computes once.
3. **One Tauri command per tiny UI action** — a per-row invoke over a 100k-row virtualized list makes scrolling IPC-bound instead of render-bound. Batch by page.
4. **Unbounded concurrent git spawns** — process spawn is non-trivial on Windows, and concurrent invocations contend on `.git/index.lock`. Debounce interactive-triggered commands; **serialize writes per repo**.

---

## 5. Phase-Shaping Constraints

### Must be in Phase 1 — retrofitting is documented to be expensive

| Item | Why it cannot wait | Retrofit cost |
|---|---|---|
| **Git env pinning in the spawn wrapper** | `LC_ALL=C`, `LANG=C`, `GIT_TERMINAL_PROMPT=0`, an empty `GIT_ASKPASS`, `GCM_INTERACTIVE=never`, a null stdin, and `CREATE_NO_WINDOW` on Windows. Plus the GitLane #425 additions: pin `log.showSignature=false` (GPG verification text corrupts `git log` parsing), clear inherited `GIT_AUTHOR_*` and `GIT_COMMITTER_*`, pin `--cleanup=whitespace` (a user setting of `commit.cleanup=strip` silently deletes comment-prefixed lines from commit messages), and pin `diff.noprefix`, `diff.external`, `format.coverLetter`. | LOW as a code change (one central function) but **requires re-testing every command** that was built against the unpinned environment. FEATURES.md: "retrofitting it means re-testing everything." |
| **Command registry** (every action is a named, parameterized command) | Enables the command palette, customizable shortcuts, and **the git command log** — all nearly free. | HIGH. If UI actions are written as inline click handlers, a palette later requires refactoring every one. This is why SourceGit has two open shortcut-customization issues it cannot easily close. |
| **Per-repo keying on both sides** | The open-repos state is a map from RepoId to RepoHandle (not a single optional handle); every command takes a repo id; frontend stores are per-repo-id slices. | Backend: MEDIUM (a breaking change to every command signature). **Frontend: this is the one frontend decision that is expensive to retrofit** — a flat-to-keyed state migration touches every consumer component. Nearly free to do correctly up front. |
| **Write serialization mutex per repo** | A single queue or mutex around all git-mutating operations, so the app never races itself on `.git/index.lock`. | **MEDIUM-HIGH** — PITFALLS.md lists this explicitly as requiring "retrofitting a serialization/mutex layer around all git-mutating operations after the fact — touches every write-path call site; cheaper to build this queue upfront." This is also the *root fix* for watcher self-triggering: if internal operations are serialized, the watcher only needs to handle genuinely external changes. |
| **GitError plus the Serialize boilerplate in `error.rs`** | Every command returns a Result with this error type | LOW, but trivially cheap now and annoying later |
| **Tauri capability-file discipline** | Narrow scopes from the start; the capability grant lands in the **same commit** as the plugin usage | Dev-time wildcard scopes get shipped if not explicitly audited — a worse trust story than necessary for an open-source tool asking for credentials |

### Group 1 — Nền tảng (Platform)

**Preconditions:** The Rust toolchain does not yet exist on the dev machine. Install **in order**: VS Build Tools 2022 with the Desktop development with C++ workload, then rustup with the stable-msvc default and the x86_64-pc-windows-msvc target. Node 22.16, npm 10.9, Git 2.54.0, and WebView2 are already satisfied.

**Must deliver:** A Tauri v2 skeleton launching on Windows; `git/exec.rs` with **all env pinning baked in centrally** (validated by a trivial `git --version` smoke command); `error.rs`; the capability-file pattern; the three-pane resizable layout shell with empty panels; repo picker plus an open-repo command plus a recent-repos list; app state **keyed by repo id**; the write-serialization mutex; the **command registry scaffold**; and a CI matrix skeleton (which can run in parallel and blocks nothing).

**Pitfalls it must address:** **#1 locale-dependent parsing** · **#2 credential-prompt hangs** (env vars plus stdin closure plus a 30–60s hard timeout on network operations) · **#5 Windows console flash** · **#6 Tauri capability misconfiguration** · **#8 partial — self-race serialization**.

**Non-obvious exit criteria:** Launch the **bundled release executable from Explorer** (not `tauri dev`) and trigger several git operations — confirm there is no console flash. Dev mode structurally cannot detect this. Also: no parsing and no domain types beyond a minimal repo handle. The job of this phase is proving the IPC bridge and spawn layer work cross-platform before any git logic sits on top.

### Group 2 — Đọc lịch sử (History reading) — the Core Value phase

**Preconditions:** `git/exec.rs` and the IPC pattern from Phase 1. **Fixture repos must exist before the lane algorithm is written** (test-first is worth it here specifically): an octopus merge with 4 parents · two unrelated roots merged with `--allow-unrelated-histories` · an orphan branch · a 20+-simultaneous-lane wide-branching shape · a repo with a non-UTF-8 filename and a non-Latin or emoji commit message · a shallow clone.

**Must deliver:** The log and refs parsers, the commit and refs domain types, **`graph/lanes.rs` as a pure heavily unit-tested function**, the repo cache, and the log and refs commands. Frontend: commitsStore, refsStore, the virtualized commit list plus graph overlay (**the highest-risk UI work in the project**), sidebar branch/tag/remote lists with counts, and branch/tag labels anchored to commit rows. **Plus the promoted gap: commit and history search** (message, author, SHA).

**Pitfalls it must address:** **#3 non-UTF-8 filenames and commit messages corrupting JSON over IPC** — one bad byte sequence must not break an entire page of history; use NUL-terminated output wherever git supports it, set `core.quotepath=false`, parse as raw bytes, and lossy-convert only at display boundaries. **#7 octopus merges and multi-root histories** — step 3 of the pseudocode must be a **loop over all parents after the first**, not a two-parent special case; a missing or unfetchable parent (a shallow-clone graft) must terminate the lane rather than crash. Decide and document a **deliberate degraded rendering** for pathological cases (cap visible lanes, show a "+N more parents" indicator) so an absurd octopus merge cannot blow the under-1s target.

**Non-obvious exit criteria:** **Performance-validate against a real 50k–100k-commit repo (Linux kernel or Chromium) here, not at the end.** FEATURES.md is explicit: performance is the Core Value and the headline claim, so if the architecture cannot hit the number, that must surface while it is still cheap to change. Wire the criterion benchmark for lane assignment and log parsing into CI in this phase.

### Group 3 — Xem khác biệt (Diff viewing)

**Preconditions:** Phase 2, since you need a selected commit to diff. The decoupled-selection pattern gets exercised for the first time here.

**Must deliver:** The diff parser, the diff domain types, the LRU diff cache (~200 entries, **never invalidated** for historical commits), and the diff commands (commit diff plus file history). Frontend: the detail panel (commit metadata, parent SHAs, changed-file list with a **flat/tree toggle**), the CodeMirror 6 viewer using MergeView for split and unifiedMergeView for unified, hunk navigation, a whitespace toggle, and per-file history. **Plus the promoted gap: binary, large-file, and LFS-pointer handling.**

**Constraints:** Lazy-load the CodeMirror language modes keyed off file extension — there are 30+ packages, and eager imports balloon the bundle and hurt startup, which is a Core Value. Do **not** install the codemirror meta-package. Use NUL-terminated output for any filename-bearing diff variant.

**Pitfalls it must address:** The performance trap of **re-diffing unchanged commits on every scroll** — cache by SHA. Minimum bar for binary and large files: **do not hang, and say why** (a size threshold before a diff is attempted, binary detection, and an "LFS object (N MB)" placeholder rather than raw pointer text).

**Exit gate (procedural, from PITFALLS.md #10):** **Dogfood gate.** The read-only history-plus-diff build must be used daily by the author on their own real repos — including this project repo and at least one large, messy third-party repo — **before any working-tree work begins.** This is a phase-exit gate, not a nice-to-have. It is the specific countermeasure to the documented failure mode where a hobby Git GUI looks far along (a polished shell) but was never usable for a single real workflow.

### Group 4 — Thay đổi repository (Working tree) — the highest-complexity phase

**Preconditions:** The Phase 3 diff viewer, because the hunk-selection UI is reused for staging — building staging first means either duplicating that logic or building it blind with no visual diff to validate against. The watcher is introduced *here*, not earlier: this is the first point where staleness is actually high-stakes.

**Must deliver:** The status command (porcelain v2, branch info, all untracked files, NUL-terminated), the staging commands, and the commit commands, plus the git watcher. Frontend: workingTreeStore, the staged/unstaged/untracked panels, the hunk staging UI, the commit composer, and amend. **Plus the promoted gaps: the discard safety net, and reset branch to commit (soft/mixed/hard).**

**Budget:** FEATURES.md says **2–3x** the naive estimate for hunk staging. Treat this as the phase most likely to overrun.

**Hard correctness rules (non-negotiable):**
- Patches are **raw bytes end to end**. Never decode to a string. Pass `--recount`. **Isolate stderr from stdout** so clean-filter warnings never enter the patch.
- Re-verify the file blob hash immediately before applying; run a `git apply --check --cached` dry run first.
- On failure, **never** retry with fuzzy matching, zero-context unidiff, or whitespace fixing. Surface "file changed since you viewed this diff, please refresh."
- Never auto-delete `.git/index.lock`. Treat its existence as an expected, recoverable transient — retry with backoff.
- Run pre-commit and commit-msg hooks by default (free via the CLI); offer an explicit no-verify toggle and make sure it works **in the conflict path too** (SourceGit bug #2707).
- Preserve the commit message draft across app restart and repo switch. Never silently discard typed text.
- Amending an already-pushed commit: **warn, do not block.**

**Pitfalls it must address:** **#4 the stale-diff apply race** · **#8 watcher event storms and lock contention** (watch `.git` internals narrowly; **never** recursively watch the working tree unfiltered; respect ignore rules; debounce 250–300ms and coalesce).

**Test cases beyond the happy path:** edit the file between diff-open and stage-click · a binary file · a file with no trailing newline · a file renamed *and* modified · CRLF/LF normalization with partial staging · a build tool actively writing to the working tree (CPU must stay bounded) · a concurrent auto-refresh overlapping a user-initiated stage (no index-lock error may reach the user).

**Exit gate:** A real commit made to a real repo **using only git-plum**.

### Group 5 — Nhánh và remote (Branch and remote)

**Preconditions:** Phase 2 (refs already modeled) and Phase 4 (write-path patterns, index locking, and watcher invalidation established on the simpler single-file staging case before extending to the higher-stakes multi-file merge/rebase case).

**Must deliver:** The branch, remote, merge, stash, tag, and cherry-pick commands. Fetch, pull, and push with force-with-lease; branch create/checkout/delete/rename; merge and rebase with structured conflict results; stash create/list/apply/pop/drop (**multiple stashes is non-negotiable — the single-stash model in GitHub Desktop is a known failure with three separate high-reaction issues**); tags; and cherry-pick and revert. **Plus the promoted gaps: continue/abort/skip for merge, rebase and cherry-pick · the two-way conflict resolver · squash/fixup/reword of a non-HEAD commit.**

**This phase carries the most write operations and the most error surface.** The per-repo write mutex becomes load-bearing here, since fetch, pull, push, merge, and rebase can all be triggered close together by an impatient user.

**Behavioral acceptance criteria — these separate a toy from a tool:**
- **Push rejected:** never show raw stderr and stop. Explain that the remote has N commits the user does not have, and offer exactly three actions: pull and merge, pull and rebase, or force push with lease. Default any force to force-with-lease, **never bare force**, and state the difference in the dialog. Show ahead/behind counts before and after — the upstream-track format in for-each-ref already provides this.
- **Pull with local changes:** detect the case **before** running; offer stash, pull, pop, or cancel. Surface the choice of merge vs rebase vs fast-forward-only explicitly, honoring the user pull.rebase setting. If the auto-stash pop conflicts, say so clearly and route into the conflict resolver.
- **Branch switch with a dirty tree:** try the checkout — git itself permits it when files do not conflict. Only intervene on actual failure, then offer stash-and-switch, discard-and-switch (with the safety net), or cancel. **Never just refuse.** Handle checking out a remote branch whose name collides with an existing local branch — GH Desktop r47, with 55 comments, is exactly this failing. Do not hang on submodules.
- **Merge conflict:** detect on entry **and on app restart** by reading MERGE_HEAD, the rebase-merge directory, and CHERRY_PICK_HEAD. Show a persistent banner naming the operation, the branches, and how many files remain conflicted. **Always offer abort.** Distinguish content conflicts from add/add, delete/modify, and binary conflicts. **Never auto-pick a side for binary files.** Marking a file resolved must stage it and update the remaining count; unstaging must not silently exit the merging state.

**Pitfalls it must address:** **#2 is verified here** — write an explicit acceptance test: a fetch against a repo with zero cached credentials must fail within a bounded time, never hang. This is the first phase that actually exercises network commands. Also: classify common git exit codes into specific actionable messages, and treat stderr as opaque diagnostic text only, never as a parseable contract.

### Group 6 — AI

**Preconditions:** Phase 3 (diff content) and Phase 4 (the staged diff as generation input). **Nothing depends on AI** — it is a leaf. It is the safest thing to defer, drop, or parallelize, since it touches almost entirely disjoint files.

**Must deliver:** An AiProvider trait plus Claude, OpenAI, and Ollama implementations (**abstract from day one — do not hardcode Claude even for the first working version**), the AI commands, and the keyring integration. Generate-from-staged-diff and recompose-existing-message. **Position "improve my draft" at least as prominently as "generate from scratch"** — that is what users actually asked for.

**Pitfall #9 is a hard definition-of-done, not polish:**
- **Secret-pattern scan the diff BEFORE any external API call** (AWS keys, PEM private key headers, API-key and secret assignments, high-entropy strings in env-like filenames). Block or warn; never silently send. Run it universally — local Ollama is lower-risk but the scan is cheap. Note that many real leaks are in *tracked* files, so ignore rules are not sufficient defense.
- **Measure diff size before sending.** If it exceeds the limit, summarize per-file or state that the diff is too large for an AI summary. **Never silently truncate** and present a message as if it reflects the whole change; if truncation is unavoidable, show "based on N of M changed files".
- Distinguish a connection refused to localhost (Ollama not running) from cloud API errors, and rate limits from generic failures.
- The diff-size and secret-scan logic must live in the **shared** layer, not be duplicated per provider.
- If the keychain is unavailable (Linux headless), disable AI with a clear message and **never** fall back to a file.

### Phát hành (Release) — wraps around, not sequential

CI matrix and code-signing setup can start scaffolding as early as Phase 1 in parallel; release neither blocks nor is blocked by feature phases. Must deliver: MSI, DMG, AppImage and deb installers; README, build guide, and MIT license; and three-platform CI. **Plus the promoted gap: the opt-out-able auto-updater.** Audit all capability files for wildcard scopes here and justify or narrow each one. Add the WebDriver smoke tests here, not earlier.

---

## 6. Validation Checkpoints

Places where the roadmap needs a **benchmark or a spike**, not an assumption.

| # | Checkpoint | Phase | The question | How to resolve | If it fails |
|---|---|---|---|---|---|
| 1 | **Large-repo graph performance** | 2 (**not 6**) | Does the graph open in under 1s and scroll at 60fps on a real 50k–100k-commit repo? | Clone the Linux kernel or Chromium. Measure first paint and scroll FPS. Run a criterion benchmark on lane assignment and log parsing at 100k commits, **wired into CI**. The `gitlanes` project reports 32k commits in ~300ms at 60fps on this exact stack — that is the bar to match. | Escalate the commit loader from a paginated command to a `tauri::ipc::Channel`, which lets React render the first 500 rows while Rust is still parsing the rest. This attacks *perceived* latency, which is what the Core Value actually measures. |
| 2 | **Graph render approach: canvas vs SVG** | 2 | The two research docs lean different ways (§2.6). Canvas means ~500 line segments plus 50 arcs per frame and one DOM node. SVG means ~500 DOM elements created and destroyed per scroll frame — workable, but GC pressure and style recalculation are real at 60fps in a WebView, and WebView2 is not Chrome. | Build the canvas version first (per STACK.md and the gitlanes precedent), but **keep the renderer behind an interface** so swapping is a one-file change. Handle device pixel ratio or it renders blurry on HiDPI. Keep canvas strictly presentational — hit-testing for which commit was clicked uses the row divs, not canvas coordinates. | Fall back to SVG rendered from the same virtualItems array. More debuggable in devtools, and CSS transitions come free. At ~50 visible rows the DOM cost is survivable. |
| 3 | **CodeMirror merge on large files** | 3 | `@codemirror/merge` **computes its own diff from the two full documents** — but git already gave us the diff. Handing CM two 10k-line documents is redundant work. Is it fast enough? | Benchmark with a large real file diff. Ship the package as-is for v1 and measure. | Render hunks yourself using CodeMirror decorations driven by the git diff output. A shiki-plus-custom-renderer is a read-only-only fallback beyond that. |
| 4 | **IPC: JSON vs binary** | 2, deferred optimization | Tauri channel messages are **JSON**, same as commands — verified from the blanket IpcResponse implementation over any Serialize type. A channel buys ordering and lower per-message overhead, **not** a binary fast path. Does JSON serialization dominate the profile? | **Ship JSON first, then measure.** Batch aggressively — 1000 commits per message, not 1. | `tauri::ipc::Response` returns raw bytes without JSON serialization. This is a genuinely good fit for **graph lane data specifically**, since a GraphRow is fixed-width numeric data — the worst possible fit for JSON and the best possible fit for a packed typed array read directly by the canvas renderer. **Phase-2 optimization only.** |
| 5 | **Two-way conflict resolver effort in our stack** | 4 to 5 boundary | The SourceGit version merged in four hours — but in a mature Avalonia codebase with an existing text editor component. Our CodeMirror 6 path *should* be comparable but is **unverified**. | Run a time-boxed spike before committing the phase scope: render git conflict markers in a CM6 view with accept-ours/theirs/both per block, write the file, stage it. | If the spike overruns, the escape is to ship conflict *detection* plus external-tool launch for v1 and treat the resolver as the first v1.x item — but make that decision on spike evidence, not on the #892 reasoning that was already falsified. |
| 6 | **keyring v4 API surface** | 6 | v4 restructured heavily from v3 (the API moved into keyring-core and the stores into separate crates). Most existing documentation and training data describe the v3 Entry API. | Verify import paths against docs.rs for keyring 4.2.0 at implementation time. The v1 feature exists precisely to preserve the old ergonomics — confirm that. | Low risk; the crate is actively maintained, with 26M downloads and a release dated 2026-08-29. |
| 7 | **Non-English locale behavior** | 1, verified continuously | Does the parser survive a Vietnamese or Japanese system locale? | Run the test suite and manual QA with the system locale set to non-English. CI currently runs in one locale only. | A central fix in the single spawn wrapper — LOW recovery cost, which is exactly why it must be verified rather than assumed. |

---

## 7. Open Questions

Genuinely unresolved. Each should become phase-specific research or early-user validation, not a guess baked into the roadmap.

1. **Two-way vs three-pane conflict UX.** The case against three-pane rests on one strong essay (eseth.org, arguing that 3-way tools do a blind diff and discard the work git already did, producing visual clutter without added value), on the 3-way usability issues Sublime Merge has open (#1079, #1869), and on the choice SourceGit shipped. It is a defensible design position, **not a settled fact.** Worth validating with early users. Route to Phase 5 research or beta feedback.

2. **Revised schedule.** No researcher produced a replacement for the 11–14 week estimate. The direction is known (longer) but the magnitude is not. Re-estimate during roadmap construction using the promoted scope, not the `docs/01` §8 table.

3. **Multi-repo tabs: in or out of v1?** `docs/01` §2.4 says MVP; PROJECT.md commits only to a recent-repos list. The architecture accommodates either at near-zero cost *if* state is keyed by repo id from Phase 1, but the UI work is MEDIUM. Decide during roadmap construction.

4. **Blame: v1 polish phase or v1.x?** FEATURES.md rates it borderline at MEDIUM confidence, inferred from the *character* of bug reports rather than a direct survey. Cost is low and the dependency is already paid for. Decide at the Phase 3 exit, based on actual schedule.

5. **AI value.** Rated "mildly valued, definitely not a switching reason, and actively distrusted by a vocal segment" — MEDIUM confidence from commentary and one open SourceGit issue, with **no quantitative data**. Validate with beta users before investing beyond the 3–5 day budget.

6. **Command palette differentiation.** MEDIUM confidence. Sublime Merge praise plus SourceGit r6 plus two shortcut-customization requests make it plausible, but it is unproven **for this specific audience**. Note the asymmetry: the *command registry* is cheap and must happen in Phase 1 regardless, since it also enables the command log, which is independently the highest-ROI item found. Only the *palette UI* is the bet.

7. **Structural evidence gaps that cannot be closed.** No quantitative abandonment data exists — everything infers demand from issue reactions and forum sentiment, which measure the vocal minority rather than the median user. **Fork has no public issue tracker**, so the closest-positioned competitor contributes zero direct user-complaint data. No client has usage telemetry, and git-plum will not either, by design. Frequency-of-use claims for blame and hunk staging are inferences from the character of bug reports. Treat all feature-demand rankings as directional, not precise.

---

## Confidence Assessment

| Area | Confidence | Basis |
|---|---|---|
| **Stack — version numbers** | **HIGH** | npm registry and crates.io API queried live on 2026-09-21, including peer-dependency ranges |
| **Stack — no keychain plugin exists** | **HIGH** | All 30 official Tauri v2 plugins enumerated; the keyring 4.2.0 v1 feature manifest verified |
| **Stack — TS 7 breaks linting** | **HIGH** | Peer range read directly from npm |
| **Stack — judgement calls** (TanStack Virtual over react-window; canvas over SVG) | **MEDIUM-HIGH** | Reasoning grounded in the stated alignment requirement of the project, plus the gitlanes same-stack precedent |
| **Features — table stakes and anti-features** | **HIGH** | Direct issue-tracker evidence with reaction counts; the #892 to #2070 reversal is a controlled natural experiment |
| **Features — hunk staging under-estimated** | **HIGH** | GitLane #425 enumerates concrete shipped bugs; SRCTREE-3047 and magit#3172 corroborate |
| **Features — differentiators and complexity estimates** | **MEDIUM** | A mix of issue data and forum sentiment; complexity derived from competitor bug trackers, not from our own build |
| **Architecture — component boundaries, data flow, build order** | **HIGH** | Tauri v2 official docs plus the pre-solved lane algorithm and git command formats already in docs/01 |
| **Architecture — graph/virtualization sync pattern** | **MEDIUM** | No single canonical open-source reference architecture found; synthesized from React virtualization libraries plus observed implementations. The dual-scroll anti-pattern evidence is HIGH. |
| **Pitfalls — git CLI behavior and Tauri v2 breaking changes** | **HIGH** | Verified against official docs and issue trackers |
| **Pitfalls — lane algorithm edge cases, why hobby GUIs die** | **MEDIUM** | Pattern-matched from community reports, not a single authoritative source |

**Overall: HIGH.** The four documents converge rather than conflict on all load-bearing decisions, and the few genuine disagreements (§2.6) have clear resolutions. The residual uncertainty is concentrated in effort estimation and in feature-demand *magnitude*, not direction.

### Gaps to Address

- **Effort estimation** is the weakest area — treat the phase *sequence* as the plan and duration as an output rather than a constraint.
- **Conflict-resolver effort in our stack** is unverified, and needs the spike in checkpoint 5 before Phase 5 scope is fixed.
- **Early-user validation** is the only way to close the two-way-vs-three-pane question and the AI-value question.
- **Cross-locale and cross-platform verification** is structurally under-tested: the dev machine is Windows 11 in English, and macOS has the weakest E2E story in the entire Tauri ecosystem.

## Sources

### Primary — HIGH confidence
- npm registry API and crates.io API — queried 2026-09-21 for every version and peer range cited
- Tauri v2 official docs — calling-rust, calling-frontend, the plugin index, prerequisites, GitHub pipelines, Windows signing, WebDriver testing, project structure, architecture
- docs.rs for tauri — the `ipc::Channel` blanket IpcResponse implementation, proving channels are JSON
- `@codemirror/merge` README — MergeView for split plus unifiedMergeView for unified
- sourcegit-scm/sourcegit issue #892, PR #2070, issue #2168 — the conflict-resolver natural experiment
- sourcegit-scm/sourcegit issues (1692) and desktop/desktop issues (1004) — queried by reactions and keyword
- Siomkin/GitLane PR #425 — hardening the git CLI write layer against the user environment; the byte-vs-string patch rule and the env-pinning catalog
- git-scm.com — git-log, git-commit, and git-apply documentation
- nobel6018/gitlanes — same-stack Tauri 2 plus React canvas commit graph; 32k commits in ~300ms at 60fps
- open-source-cooperative/keyring-rs — the v4 restructure and platform stores

### Secondary — MEDIUM confidence
- tauri-apps/tauri discussion #11446 and issue #11513 — Windows console flash; production spawn hangs
- magit#3172, SRCTREE-3047, SRCTREE-2789 — hunk staging failures
- microsoft/git PR #167 — commit-graph octopus merge offset bug
- atom/github#1001 — discard-induced data loss
- bvaughn/react-virtualized#369 — ScrollSync lag, the dual-scroll anti-pattern
- Nano-Collective/nanocoder#1344 — stdin closure and GIT_TERMINAL_PROMPT
- eseth.org on mergetools — the case against three-way merge tools; hideResolved in git 2.31+
- sublimehq/sublime_merge #1079 and #1869 — 3-way merge UI usability
- Sublime Merge docs — command palette and key bindings
- GitKraken undo/redo help — supported undo actions including Discard

### Tertiary — LOW confidence, needs validation
- danmackinlay.name on git GUIs — the opaque-command-line critique
- Atlassian community Sourcetree 4.0 performance threads; devRant — forced-account abandonment
- jonathansblog.co.uk, makerstack.co, dev.to, biggo.com — reviews and AI-commit-message sentiment
- infoq.com on the TypeScript 7 release — the 7.1 programmatic-API timeline is MEDIUM at best
- ionalexandru99/rebase-git PR #415 — a single community project; corroborating only

### Internal
- `.planning/PROJECT.md` — the scope under audit
- `docs/01-research-competitors.md` — market comparison, GitKraken screenshot analysis, the lane algorithm in §4, git output formats in §5, the risk table in §7, and the effort estimate in §8. **Already patched with the three gaps PITFALLS.md found**: §5.0 env pinning, the §4.2 octopus-merge loop, and the §5.5 blob-hash re-verification step.

---
*Research completed: 2026-09-21*
*Ready for roadmap: yes — but §2 requires a scope decision before phases are fixed*
