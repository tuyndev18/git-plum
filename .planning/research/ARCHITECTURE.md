# Architecture Research

**Domain:** Cross-platform Git GUI desktop client (Tauri v2 + Rust backend shelling out to `git` CLI, React/TypeScript frontend)
**Researched:** 2026-09-21
**Confidence:** HIGH (component boundaries, data flow, build order — grounded in Tauri v2 official docs and the project's own pre-solved lane algorithm/git command research) / MEDIUM (graph-virtualization sync pattern — no single canonical OSS reference architecture found, pattern synthesized from React virtualization libraries + observed git-client implementations)

## Standard Architecture

### System Overview

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         FRONTEND (WebView / React)                        │
├───────────────┬───────────────────────────────┬──────────────────────────┤
│  Sidebar       │  History / Working-Dir View    │  Detail Panel            │
│  (branches,    │  (graph + virtualized commit   │  (commit meta, file      │
│   remotes,     │   list, OR staging view)       │   list, diff viewer)     │
│   tags)        │                                │                          │
├───────────────┴───────────────────────────────┴──────────────────────────┤
│                    App/Repo State (Zustand or Jotai stores)               │
│   repoStore (active repo, HEAD) · refsStore · commitsStore (cache)        │
│   selectionStore (selected commit/file) · diffCache · uiStore (layout)    │
├─────────────────────────────────────────────────────────────────────────┤
│                    Tauri IPC bridge (invoke + event listen)               │
└──────────────────────────────┬────────────────────────────────────────────┘
                                │ JSON commands / JSON events
┌──────────────────────────────▼────────────────────────────────────────────┐
│                          BACKEND (Rust, src-tauri)                        │
├─────────────────────────────────────────────────────────────────────────┤
│  commands/*  — thin Tauri #[command] handlers, one file per domain area   │
│  (log.rs, status.rs, diff.rs, branch.rs, remote.rs, stash.rs, commit.rs)  │
├─────────────────────────────────────────────────────────────────────────┤
│  git/exec.rs   — process spawn layer (tokio::process, arg building,       │
│                  timeout, cwd = repo path, env sanitization)              │
│  git/parsers/* — one parser module per git subcommand output format       │
│                  (log.rs, status.rs, diff.rs, refs.rs, stash.rs)          │
│  domain/*      — plain Rust structs: Commit, Ref, DiffHunk, FileStatus,   │
│                  WorkingTreeStatus, StashEntry (Serialize for IPC)        │
│  graph/lanes.rs — lane assignment algorithm (pure fn: Vec<Commit> ->      │
│                   Vec<GraphRow>), no IO                                   │
│  cache/*        — per-repo in-memory caches (commit page cache, diff      │
│                   cache, refs cache) keyed by repo path + revision        │
│  watcher/*      — notify-based .git watcher, debounced, emits Tauri       │
│                   events to invalidate frontend + backend caches          │
│  state/*        — Tauri managed state: open repos registry, active       │
│                   repo handle, AI provider config                        │
└─────────────────────────────────────────────────────────────────────────┘
                                │ spawns
┌──────────────────────────────▼────────────────────────────────────────────┐
│                    `git` CLI (subprocess per invocation)                  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility | Typical Implementation |
|-----------|----------------|------------------------|
| `commands/*` | Validate IPC args, call into `git/`, `domain/`, `cache/`, serialize response | Thin `#[tauri::command] async fn` — no parsing logic lives here |
| `git/exec.rs` | Spawn `git` subprocess, build args safely, capture stdout/stderr, timeout/kill | `tokio::process::Command`, returns raw `Vec<u8>` or `String` |
| `git/parsers/*` | Turn one specific git output format into domain structs | Pure functions, one per subcommand format (`%x1f`/`%x1e` split, `--porcelain=v2` line parse, unified diff parse) |
| `domain/*` | Canonical in-memory representation shared across parsers/cache/commands | Plain `struct`/`enum` with `serde::Serialize`, no git-specific logic |
| `graph/lanes.rs` | Assign lane + edges per commit row | Pure function per PROJECT's own algorithm (doc section 4), unit-testable in isolation |
| `cache/*` | Avoid re-spawning git / re-parsing for unchanged data | `HashMap<RepoId, RepoCache>` behind `Mutex`/`RwLock` in Tauri managed state |
| `watcher/*` | Detect external repo changes, trigger targeted invalidation | `notify` crate watching `.git/HEAD`, `.git/refs/**`, `.git/index`, debounced (~150-300ms) |
| `state/*` | Track open repo(s), active repo, app-level config | Tauri `State<Mutex<AppState>>` |
| Sidebar (React) | Render branches/remotes/tags from `refsStore`; dispatch checkout/delete actions | Component owns no fetch logic itself, reads store |
| History view (React) | Render graph column + virtualized commit rows in one scroll container | `commitsStore` (paged cache) + `@tanstack/react-virtual` |
| Detail panel (React) | Render selected commit's metadata + file list + diff | Reacts to `selectionStore.selectedCommitId`, fetches on change, own local diff cache |
| Working-dir view (React) | Staged/unstaged/untracked file lists, hunk staging UI | Own store (`workingTreeStore`), refetches on `status-changed` event |

## Recommended Project Structure

```
src-tauri/
├── src/
│   ├── main.rs                  # entry point, Tauri builder, plugin registration
│   ├── lib.rs                   # app setup, command registration, state init
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── repo.rs               # open_repo, list_recent_repos, close_repo
│   │   ├── log.rs                 # get_commit_page, get_graph_rows
│   │   ├── refs.rs                # list_branches, list_tags, list_remotes
│   │   ├── status.rs              # get_working_tree_status
│   │   ├── diff.rs                # get_commit_diff, get_working_diff, get_file_history
│   │   ├── stage.rs               # stage_hunk, unstage_hunk, stage_file, discard_hunk
│   │   ├── commit.rs               # create_commit, amend_commit
│   │   ├── branch.rs               # create_branch, checkout, delete_branch, rename_branch
│   │   ├── remote.rs                # fetch, pull, push
│   │   ├── merge.rs                  # merge, rebase, abort, continue
│   │   ├── stash.rs                   # stash_create, stash_list, stash_apply, stash_pop, stash_drop
│   │   ├── tag.rs                      # create_tag, delete_tag
│   │   ├── cherry_pick.rs               # cherry_pick, revert
│   │   └── ai.rs                         # generate_commit_message, recompose_commit
│   ├── git/
│   │   ├── mod.rs
│   │   ├── exec.rs                # process spawn layer, GitError type
│   │   ├── args.rs                 # safe arg-builder helpers (avoid shell injection via -- separators)
│   │   └── parsers/
│   │       ├── mod.rs
│   │       ├── log.rs               # parses %x1f/%x1e log format -> Vec<Commit>
│   │       ├── refs.rs               # for-each-ref format -> Vec<Ref>
│   │       ├── status.rs              # --porcelain=v2 -z -> WorkingTreeStatus
│   │       ├── diff.rs                 # unified diff -> Vec<DiffHunk>
│   │       └── stash.rs                 # stash list format -> Vec<StashEntry>
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── commit.rs               # Commit, CommitSummary
│   │   ├── refs.rs                  # Branch, Tag, Remote, RefKind
│   │   ├── status.rs                  # WorkingTreeStatus, FileStatus, StageState
│   │   ├── diff.rs                     # DiffHunk, DiffLine, FileDiff
│   │   └── stash.rs                     # StashEntry
│   ├── graph/
│   │   ├── mod.rs
│   │   ├── lanes.rs                # lane assignment algorithm (pure, unit-tested)
│   │   └── types.rs                 # GraphRow, Edge
│   ├── cache/
│   │   ├── mod.rs
│   │   ├── repo_cache.rs           # per-repo commit page cache + refs cache
│   │   └── diff_cache.rs            # commit-id-keyed diff cache (LRU)
│   ├── watcher/
│   │   ├── mod.rs
│   │   └── git_watcher.rs          # notify watcher on .git/, debounce, emits events
│   ├── state/
│   │   ├── mod.rs
│   │   └── app_state.rs             # AppState { open_repos, recent_repos, ai_config }
│   └── ai/
│       ├── mod.rs
│       ├── provider.rs              # trait AiProvider { generate_commit_message(...) }
│       ├── claude.rs
│       ├── openai.rs
│       └── ollama.rs
└── Cargo.toml

src/  (React frontend)
├── main.tsx
├── app/
│   ├── App.tsx                    # three-pane layout shell, resizable panels
│   └── routes or view-mode switch (History vs Working-Dir vs Diff-only)
├── stores/
│   ├── repoStore.ts                # active repo path, HEAD, recent repos list
│   ├── refsStore.ts                 # branches/tags/remotes, refetched on refs-changed event
│   ├── commitsStore.ts               # paged commit cache + graph rows, keyed by repo+revwalk params
│   ├── selectionStore.ts              # selectedCommitId, selectedFilePath, diff view mode
│   ├── workingTreeStore.ts             # staged/unstaged/untracked, refetched on status-changed event
│   └── diffCacheStore.ts                # commit-id-keyed diff cache (frontend mirror, short TTL)
├── features/
│   ├── sidebar/                    # BranchList, RemoteList, TagList
│   ├── history/                     # GraphColumn, CommitRow, CommitList (virtualized)
│   ├── working-directory/            # StatusPanel, FileList, HunkView
│   ├── diff-viewer/                   # CodeMirror6 wrapper, UnifiedDiff, SplitDiff, HunkNav
│   ├── detail-panel/                   # CommitMeta, FileTree/FlatList toggle
│   └── ai/                              # commit-message generate/recompose UI
├── ipc/
│   ├── commands.ts                 # typed wrappers around invoke() per command
│   └── events.ts                    # typed wrappers around listen() for backend events
└── lib/
    ├── virtualization/              # shared row-height constants, scroll math helpers
    └── graph-render/                  # SVG/canvas renderer consuming GraphRow[] from backend
```

### Structure Rationale

- **`commands/` split by git subcommand domain, not by CRUD verb:** matches PROJECT.md's requirement groups almost 1:1 (log.rs/refs.rs → history reading, diff.rs → diff viewing, stage.rs/commit.rs → working tree changes, branch.rs/remote.rs/merge.rs/stash.rs/tag.rs/cherry_pick.rs → branches+remote). This makes phase-to-module mapping direct and keeps each phase's Rust surface additive, never touching prior phases' files.
- **`git/` vs `domain/` separation:** `git/parsers` know about git's text formats (`%x1f`, `--porcelain=v2`); `domain/` knows nothing about git at all — it's just data. This means if a future milestone swaps the parsing strategy (unlikely, but e.g. adding libgit2 for read-only fast paths) only `git/parsers` changes, not every command handler.
- **`graph/lanes.rs` isolated and IO-free:** the lane algorithm is already fully designed (see `docs/01-research-competitors.md` section 4). Keeping it a pure function (`Vec<Commit> -> Vec<GraphRow>`) with no file IO or async makes it trivially unit-testable against fixed commit-graph fixtures (linear history, merge, octopus merge, orphan branch) — critical since this is the Core Value.
- **`cache/` as its own module, not embedded in commands:** cache keys need to span multiple commands (e.g., `get_commit_page` and `get_graph_rows` should share the same underlying revwalk result). Centralizing avoids each command reinventing invalidation logic.
- **`watcher/` isolated, event-driven, not polling:** it only touches Tauri's event system and `cache/` invalidation calls — it does not know about parsers or domain types directly, it just says "something in refs/index changed, re-fetch."
- **Frontend `stores/` split by data ownership, not by view:** `commitsStore`, `refsStore`, `workingTreeStore` are independent from which panel renders them, so `history/` and `detail-panel/` can both read `commitsStore` without duplicate fetches (see Data Flow below).
- **`ipc/commands.ts` and `ipc/events.ts` as the only files that call `invoke`/`listen`:** every store goes through these typed wrappers, so there is exactly one place that knows the Rust command names/payload shapes — renaming a Tauri command touches one frontend file.

## Architectural Patterns

### Pattern 1: Thin command handlers, fat pure modules

**What:** `#[tauri::command]` functions do argument validation + orchestration only (call exec → parse → cache → return). All actual logic (parsing, lane computation, diff hunk reconstruction) lives in plain Rust functions with no Tauri/async dependency.
**When to use:** Always, for this codebase. It's what makes the Rust layer testable without spinning up a Tauri runtime, and keeps PROJECT.md's risk item ("Rust là rào cản với nhóm quen web... không async phức tạp") true in practice.
**Trade-offs:** Slightly more files/indirection up front; pays off immediately once you have 6+ command domains and need `cargo test` on parsers without mocking IPC.

```rust
// commands/log.rs
#[tauri::command]
async fn get_commit_page(
    state: tauri::State<'_, AppState>,
    repo_id: String,
    skip: u32,
    limit: u32,
) -> Result<CommitPage, String> {
    let repo = state.get_repo(&repo_id)?;
    if let Some(cached) = repo.cache.get_page(skip, limit) {
        return Ok(cached);
    }
    let raw = git::exec::run(&repo.path, &["log", "--all", "--topo-order", "--format=...", ...]).await?;
    let commits = git::parsers::log::parse(&raw)?;
    let graph_rows = graph::lanes::assign(&commits);
    let page = CommitPage { commits, graph_rows };
    repo.cache.put_page(skip, limit, page.clone());
    Ok(page)
}
```

### Pattern 2: Single shared scroll container for graph + commit list ("virtual row" is the source of truth)

**What:** The graph column and the commit-message/metadata columns are **not** two synced scrollable components — they are one virtualized list where each row renders a `<div>` containing both the graph cell (SVG/canvas segment for that row) and the text cells, laid out with CSS grid/flex inside the same row element. There is exactly one scroll container, one virtualizer instance (e.g. `@tanstack/react-virtual`'s `useVirtualizer`), and the graph geometry is precomputed per-row so each row can render its own graph slice independently, statelessly, at mount time.
**When to use:** Always for this feature — this is the "hardest UI problem" called out in the question, and the dual-scroll-container approach is the documented anti-pattern (see Anti-Patterns below; `react-virtualized`'s own `ScrollSync` has open, long-standing issues with lag under fast scroll, confirmed by community reports).
**Trade-offs:** Requires fixed row height (already a stated requirement — "Chiều cao hàng cố định" in the competitor research) and requires the backend to hand over complete per-row graph geometry (lane, color, passthrough edges, out edges) so the frontend never has to compute connectivity across rows — it only draws what's given.

```tsx
// features/history/CommitList.tsx
const virtualizer = useVirtualizer({
  count: totalCommitCount,
  getScrollElement: () => scrollRef.current,
  estimateSize: () => ROW_HEIGHT, // fixed, e.g. 28px
  overscan: 20,
});

return (
  <div ref={scrollRef} className="commit-list-scroll">
    <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
      {virtualizer.getVirtualItems().map(vRow => {
        const row = commitsStore.getRow(vRow.index); // from cached page, may trigger prefetch
        return (
          <div
            key={vRow.key}
            style={{ position: 'absolute', top: vRow.start, height: ROW_HEIGHT, display: 'flex', width: '100%' }}
          >
            <GraphCell graphRow={row?.graphRow} rowHeight={ROW_HEIGHT} /> {/* SVG <g>, positioned via viewBox math from vRow.index */}
            <CommitMetaCell commit={row?.commit} selected={row?.commit?.id === selectedCommitId} />
          </div>
        );
      })}
    </div>
  </div>
);
```

The graph itself is best rendered as **one absolutely-positioned SVG (or single `<canvas>`) overlay sized to the full virtual scroll height, translated with the same scroll offset**, rather than one SVG per row — this avoids hundreds of independent SVG elements and guarantees pixel alignment because it's driven by the exact same `scrollTop` the row virtualizer uses. Each `GraphRow.lane` maps to `x = laneWidth * lane`, each row index maps to `y = rowIndex * ROW_HEIGHT`, so edges between adjacent rows are just line segments computed from two adjacent `GraphRow`s — no re-layout needed on scroll, only re-clipping to the visible viewport.

### Pattern 3: Selection state decoupled from data-fetch state

**What:** `selectionStore` holds only `selectedCommitId` (and `selectedFilePath` for diff drill-down). It contains no commit data itself. The detail panel and diff viewer subscribe to `selectionStore.selectedCommitId`, and independently fetch/cache the commit detail (via `commitsStore` if already paged in, else a dedicated `get_commit_detail` IPC call) and diff (via `diffCacheStore`). Clicking a different commit never touches `commitsStore`'s paged data or triggers a graph recompute.
**When to use:** Always — this is explicitly called out in the competitor research ("Không dựng lại toàn bộ graph khi chọn commit khác. Tách trạng thái chọn ra khỏi trạng thái dữ liệu graph").
**Trade-offs:** None significant; this is a standard state-slicing practice, just worth being explicit about given it's named as an anti-pattern to avoid.

## Data Flow

### Key Data Flows

**1. Open repo:**
```
User picks folder (Tauri dialog plugin)
  → invoke('open_repo', { path })
    → Rust: validate .git exists, register in AppState.open_repos, add to recent_repos (persisted to app config dir)
    → Rust: spawn watcher::git_watcher for this repo path (watches .git/HEAD, .git/refs/**, .git/index, .git/MERGE_HEAD)
    → Rust: kick off initial fetches concurrently: refs (for-each-ref), first commit page (log --max-count=..), working tree status
    → returns { repoId, initialBranch, recentRepos }
  → Frontend: repoStore.setActiveRepo(repoId); refsStore.load(repoId); commitsStore.loadFirstPage(repoId); workingTreeStore.load(repoId)
  → History view renders as soon as first commit page + graph rows arrive (target: under 1s per Core Value)
```

**2. Click a commit (history view):**
```
User clicks row in virtualized list
  → selectionStore.setSelectedCommit(commitId)   [pure client-side state change, no IPC yet]
  → DetailPanel subscribes to selectionStore, sees change
    → commitsStore.getCommit(commitId) — already in page cache from step 1? use directly, no IPC
    → invoke('get_commit_diff', { repoId, commitId }) only if not in diffCacheStore
      → Rust: cache/diff_cache lookup by commit_id (commits are immutable, so this cache never needs invalidation for historical commits)
      → miss: git diff --patch --no-color --find-renames <parent>..<commit>, parse, store in cache, return
  → DiffViewer renders from diffCacheStore
```
Note the key property: selecting a commit is **zero IPC calls** if metadata is already paged in; only the diff fetch is a real round trip, and it's cached forever per commit-id since historical diffs never change.

**3. Stage a hunk (working tree view):**
```
User clicks "stage hunk" on a hunk in the working-dir diff
  → invoke('stage_hunk', { repoId, filePath, hunkId })
    → Rust: reconstruct a patch containing only that hunk (from the already-fetched working diff, held in backend or re-diffed) preserving file header
    → git apply --cached -  (patch piped via stdin)
    → on success: re-run `git status --porcelain=v2 -z` (cheap), return fresh WorkingTreeStatus
  → Frontend: workingTreeStore.setStatus(freshStatus) — no need to wait for watcher event, this is a direct response
  → Rust ALSO emits a 'status-changed' event in case other windows/views care (defensive, but avoids relying solely on watcher for self-triggered changes since watcher debounce could lag)
```

**4. Click Pull:**
```
User clicks Pull
  → invoke('pull', { repoId, options: { rebase?: bool } })
    → Rust: git pull [--rebase] [--ff-only|...], stream stdout/stderr as progress events if long-running (emit 'git-operation-progress')
    → on success or conflict: 
        - refs changed → invalidate repo_cache refs + commit page cache (new commits arrived)
        - working tree changed → refresh WorkingTreeStatus
        - conflict → return structured ConflictResult { conflictedFiles: Vec<String> }, do NOT throw generic error
  → Frontend: on success, refsStore.reload(); commitsStore.invalidateAndReloadFirstPage(); workingTreeStore.reload()
  → on conflict: surface conflict banner + file list, per PROJECT.md's "phát hiện xung đột, liệt kê tệp, mở editor ngoài" scope
```

### Cache Layers and Invalidation

| Cache | Location | Key | Invalidated by |
|-------|----------|-----|-----------------|
| Commit page cache | Rust `cache/repo_cache.rs` (+ mirrored in `commitsStore` frontend) | `(repo_id, skip, limit, revwalk_params)` | New commit created locally (commit/amend/merge/rebase/cherry-pick/revert), refs changed externally (watcher event on `.git/refs/**` or `.git/HEAD` or packed-refs), pull/fetch bringing new commits |
| Refs cache (branches/tags/remotes + ahead/behind) | Rust `cache/repo_cache.rs` | `repo_id` | Any branch/tag/remote mutation command, watcher event on `.git/refs/**`, `.git/packed-refs`, fetch/pull/push |
| Diff cache (per-commit, historical) | Rust `cache/diff_cache.rs` (LRU, bounded, e.g. 200 entries) | `(repo_id, commit_id)` | **Never** — historical commit diffs are immutable by definition. Only evicted by LRU size limit, never by correctness concerns |
| Working-tree status | Not cached, or cached with immediate self-invalidation | `repo_id` | Every stage/unstage/discard/commit action re-fetches directly; watcher also triggers refresh for external changes (edits from an editor, `git commit` from terminal) |
| Working-tree diff (staged/unstaged) | Short-TTL or no cache — recomputed on status change | `(repo_id, filePath, staged|unstaged)` | Any working-tree-status-changing action, watcher event on working tree files (optional — see below) |

**External change detection (the `.git` watcher question):** yes, a file watcher is required — without one, a user running `git commit` in a terminal alongside git-plum sees stale state until they manually refresh, which fails the product's premise of being a fast daily-driver GUI. Use the `notify` crate (already Tauri's own recommended cross-platform FS watch dependency) watching specifically:
- `.git/HEAD` — branch switches
- `.git/refs/**` and `.git/packed-refs` — new commits, branch/tag changes
- `.git/index` — staging changes made outside the app
- `.git/MERGE_HEAD`, `.git/rebase-merge/`, `.git/rebase-apply/` — conflict/rebase state changes

Do **not** watch the full working tree recursively by default — that's expensive on large repos and mostly redundant, since `git status` is cheap enough to re-run on a debounced timer or on window-focus-regained as a fallback for working-file edits. Watching `.git/index` catches the common case (another tool staged something). Debounce all watcher events 150-300ms (git operations touch multiple files in quick succession — e.g. a commit touches `HEAD`, the ref file, and `index` almost simultaneously) and coalesce into a single `repo-changed` event carrying a change-kind hint (`refs`, `index`, `rebase-state`) so the frontend can invalidate narrowly instead of doing a full reload every time.

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| Small repos (<5k commits) | Current design as specified is more than sufficient; even non-paged full log load would work, but keep pagination for consistency |
| Medium repos (5k-100k commits, the stated target) | Pagination (`--skip`/`--max-count`) is mandatory, as already planned; lane algorithm must run over the *entire* topo-ordered history up front (lane assignment needs global context, can't be computed per-page in isolation) — cache the full `Vec<GraphRow>` (lane/color/edges only, not full commit bodies) in memory since it's cheap (a few bytes/row), and page only the heavier commit metadata (message, author, etc.) lazily |
| Very large repos (>100k commits, explicitly out of scope for v1 per PROJECT.md) | Would require incremental/on-disk indexing and virtualized revwalk — explicitly deferred; current architecture doesn't block adding this later since `cache/repo_cache.rs` is already the single seam where an on-disk cache (e.g. sled/sqlite) could replace the in-memory `HashMap` without touching commands or frontend |

### Scaling Priorities

1. **First bottleneck:** spawning `git log` repeatedly on every scroll/page request. Mitigated by the page cache — once a page is fetched it's never re-spawned for the same revwalk params.
2. **Second bottleneck:** lane computation over very large histories recomputed from scratch on every new commit. Mitigate by only recomputing lanes for the affected window (new commits at HEAD) rather than the full history when the invalidation reason is "new local commit" — full recompute is fine (and correct) when the reason is "external refs changed in a way that could affect ancestor topology" (e.g., after a `fetch` bringing many new branches).

## Anti-Patterns

### Anti-Pattern 1: Dual scroll containers with a "ScrollSync" wrapper

**What people do:** Render the graph column and the commit-list column as two separate scrollable `<div>`s and sync their `scrollTop` via a wrapper component or manual `onScroll` handlers (the `react-virtualized` `ScrollSync` pattern).
**Why it's wrong:** Even with synced `scrollTop`, there is inherent event-loop lag between the two `onScroll` handlers firing, which is visibly jittery during fast scroll/flick — this is a long-documented, unresolved complaint against that exact pattern. For a Core Value feature ("cuộn mượt kể cả trên repo hàng chục nghìn commit"), this jitter is unacceptable.
**Do this instead:** One scroll container, one virtualizer, row = graph-cell + text-cells rendered together per Pattern 2 above. If a canvas/SVG overlay is used for the graph line segments, position it with `transform: translateY(-scrollTop)` inside the same scrolling element, not as a separately-scrolled sibling.

### Anti-Pattern 2: Computing lane assignment in the frontend

**What people do:** Send raw commit+parent data to the frontend and compute lanes/colors in JavaScript/TypeScript on each render or on data arrival.
**Why it's wrong:** Explicitly flagged as the wrong approach in the project's own research ("Tính toán lane trong Rust, không trong JavaScript... Đây là điểm nghẽn hiệu năng chính"). JS lane computation blocks the render thread and duplicates logic that Rust already computes correctly once per page/invalidation.
**Do this instead:** Rust always returns `GraphRow[]` (lane, color, passthrough edges, out edges) alongside commit metadata. Frontend is a pure renderer of this structure, never recomputes topology.

### Anti-Pattern 3: One Tauri command per tiny UI action

**What people do:** Create extremely granular commands like `get_commit_author`, `get_commit_message`, `get_commit_parents` called separately per commit row.
**Why it's wrong:** Each `invoke()` has IPC serialization overhead; doing this per-row for a virtualized list of 100k rows would make scrolling IPC-bound instead of render-bound.
**Do this instead:** Batch by page (`get_commit_page(skip, limit)` returns full `Commit[]` + `GraphRow[]` for the whole page in one call). Only fetch per-commit-id for genuinely expensive, deferred data (diff content, full file tree at that commit).

### Anti-Pattern 4: Treating `git` subprocess calls as always-cheap and always-safe to run on the main async runtime without bounds

**What people do:** Spawn unbounded concurrent `git` processes per rapid user interaction (e.g., every keystroke while typing a branch filter re-runs `for-each-ref`).
**Why it's wrong:** On Windows especially, process spawn cost is non-trivial (PROJECT.md flags Windows as the primary dev/test target and calls out path/CRLF/file-lock quirks); unbounded concurrent git invocations against the same repo can also contend on `.git/index.lock`.
**Do this instead:** Debounce/cancel-in-flight for interactive-triggered commands (filter-as-you-type should hit the already-fetched refs cache client-side, not re-invoke git), and serialize write operations (stage/commit/branch mutations) per-repo via a mutex in `AppState` so two concurrent write commands never race on `index.lock`.

## Integration Points

### External Services

| Service | Integration Pattern | Notes |
|---------|---------------------|-------|
| `git` CLI (subprocess) | `git/exec.rs` spawns via `tokio::process::Command`, repo path as `cwd` or `-C <path>`, capture stdout as bytes (not lossy UTF-8 string, to handle exotic filenames/encodings safely) | Credential helpers, SSH agent, LFS, hooks all work for free per PROJECT.md's platform decision — no separate integration needed |
| AI providers (Claude API, OpenAI API, Ollama) | `ai/provider.rs` defines a `trait AiProvider`; each backend (`claude.rs`, `openai.rs`, `ollama.rs`) implements it; commands/ai.rs picks the configured provider from `AppState.ai_config` | Keep this behind the trait from day one per PROJECT.md's Key Decision — do not hardcode Claude in `commands/ai.rs` even for the first working version |
| OS keychain (API key storage) | Use a Tauri keychain/stronghold-style plugin (`keyring` crate on the Rust side, or Tauri's stronghold plugin) — never write keys to `tauri.conf.json` or a plaintext store | Required by PROJECT.md's Security constraint |
| OS file dialog (repo picker) | Tauri's `dialog` plugin, invoked from `commands/repo.rs` or directly from frontend via Tauri's JS API | Standard Tauri v2 plugin, no custom code needed |

### Internal Boundaries

| Boundary | Communication | Notes |
|----------|---------------|-------|
| `commands/*` ↔ `git/exec.rs` | Direct function call (async) | Commands never build git args themselves inline for anything beyond trivial cases — route through `git/args.rs` helpers to avoid injection via unsanitized branch names/paths (always use `--` separator before pathspecs) |
| `git/parsers/*` ↔ `domain/*` | Parsers return `domain::*` structs directly | Parsers are the only code that imports both raw-string handling and domain types |
| `commands/*` ↔ `cache/*` | Cache checked before spawning git, populated after | Cache module never calls back into `commands` — one-directional dependency |
| `watcher/*` ↔ `cache/*` + Tauri event emitter | Watcher calls `cache.invalidate(...)` then `app_handle.emit("repo-changed", payload)` | Watcher never talks to `git/parsers` or `domain` directly — it only knows "something changed," not what |
| Rust backend ↔ React frontend | `invoke()` for request/response, `listen()` for backend-initiated events (`repo-changed`, `git-operation-progress`) | All payloads are `serde`-serialized domain structs; frontend has matching TS types (hand-written or generated via `specta`/`tauri-specta` to keep them in sync — recommended given the number of domain types this project will accumulate) |
| `commitsStore` ↔ `selectionStore` | No direct coupling — `selectionStore` holds only an ID, components read both stores independently | Prevents the "rebuild whole graph on selection change" anti-pattern from competitor research |
| History view ↔ Working-dir view | Mutually exclusive view mode in `uiStore` (or route), not simultaneously mounted | Matches GitKraken's pattern of the center column switching between commit-list and diff/status content; keeps `commitsStore` and `workingTreeStore` independently lazy-loaded (don't fetch working tree status if user is only browsing history, and vice versa) |

## Suggested Build Order (mapped to PROJECT.md's six requirement groups)

This maps directly to phase boundaries — each phase is additive to the module layout above, never restructuring prior phases.

**Phase 1 — Nền tảng (Platform):**
Establish `src-tauri` skeleton, `git/exec.rs` (process spawn layer with a trivial `git --version` smoke-test command), Tauri command registration pattern, three-pane resizable layout shell in React (empty panels), repo-picker dialog + `open_repo` command + recent-repos list persisted to app config dir, `state/app_state.rs`. No parsing, no domain types beyond a minimal `RepoHandle` yet. This phase's job is proving the IPC bridge and process-spawn layer work cross-platform (Windows first, per constraint) before any git-specific logic is built on top.

**Phase 2 — Đọc lịch sử (History reading):**
Depends on Phase 1's `git/exec.rs` + IPC pattern. Build `git/parsers/log.rs`, `git/parsers/refs.rs`, `domain::commit`, `domain::refs`, `graph/lanes.rs` (unit-test this heavily and independently, it's pure), `cache/repo_cache.rs`, `commands/log.rs` + `commands/refs.rs`. Frontend: `commitsStore`, `refsStore`, the virtualized `CommitList` + `GraphColumn` (Pattern 2 — this is the highest-risk UI work, build and performance-test it here against a large real repo before adding anything else), sidebar branch/tag/remote lists. This phase alone delivers the Core Value and should be performance-validated against a 50k-100k commit repo before moving on.

**Phase 3 — Xem khác biệt (Diff viewing):**
Depends on Phase 2 (needs a selected commit to diff). Build `git/parsers/diff.rs`, `domain::diff`, `cache/diff_cache.rs`, `commands/diff.rs` (commit diff + file history). Frontend: `selectionStore`, detail panel (commit meta + file list, Path/Tree toggle), CodeMirror 6 diff viewer (unified + split), hunk navigation, whitespace toggle. This is also where the "selection decoupled from graph data" pattern (Pattern 3) gets exercised for the first time.

**Phase 4 — Thay đổi repository (Working tree changes):**
Depends on Phase 3's diff viewer (hunk-level UI is reused for staging) and Phase 1's watcher scaffolding should be introduced here (first point where external-change detection actually matters — before this phase, the app is read-only and staleness is low-stakes). Build `commands/status.rs`, `commands/stage.rs`, `commands/commit.rs`, `watcher/git_watcher.rs` (now it has real invalidation targets: refs cache + working tree status). Frontend: `workingTreeStore`, working-directory view (staged/unstaged/untracked), hunk staging UI reusing the diff viewer, commit composer, amend.

**Phase 5 — Nhánh và remote (Branch + remote operations):**
Depends on Phase 2 (refs/branches already modeled) and Phase 4 (write-command patterns, index locking, watcher invalidation established). Build `commands/branch.rs`, `commands/remote.rs`, `commands/merge.rs`, `commands/stash.rs`, `commands/tag.rs`, `commands/cherry_pick.rs`. This phase has the most write-operations and the most conflict/error-surface handling (merge/rebase conflict detection returning structured results, not generic errors) — the serialized-write-per-repo mutex (Anti-Pattern 4) becomes load-bearing here since fetch/pull/push/merge/rebase can be triggered close together by an impatient user.

**Phase 6 — AI:**
Independent of git internals — depends only on Phase 3 (diff content to summarize) and Phase 4 (staged diff as generation input) being available. Build `ai/provider.rs` trait + three implementations, `commands/ai.rs`, keychain storage integration. Can be developed in parallel with Phase 5 by a second workstream since it touches almost entirely disjoint files (`ai/*`, `commands/ai.rs`, a small AI settings UI) — flag this as a parallelizable phase if team size allows.

**Phát hành (Release) is not a phase with architectural dependencies** — it wraps around all of the above (CI matrix, installers) and can start its scaffolding (CI pipeline skeleton, code signing setup) as early as Phase 1 in parallel, since it doesn't block or get blocked by feature phases.

**Why this order and not another:** History reading must come before diff viewing (you need a selected commit to diff). Diff viewing must come before working-tree staging (the hunk-selection UI and patch-reconstruction logic are shared — building staging first means either duplicating that logic or building it in a vacuum with no visual diff to validate against). Working-tree changes should precede branch/remote operations because the write-path safety patterns (index locking, structured conflict results, watcher-driven invalidation) are simpler to establish on the single-file staging use case before extending to the higher-stakes multi-file merge/rebase use case. AI is deliberately last among features because it's the only group with zero dependency on git plumbing correctness — it only needs diff text as input — so it's the safest phase to defer or parallelize without risking rework.

## Multi-Repo / Multi-Window (forward-compatibility note)

PROJECT.md's stated v1 scope is a single open repo with a recent-repos list (no tabs required yet), but the screenshots referenced show GitKraken's tab UI, so the architecture should not block adding it later.

**Design for this now, without building it now:**
- `state/app_state.rs`'s `AppState.open_repos` should already be a `HashMap<RepoId, RepoHandle>` (not a single `Option<RepoHandle>`), even though v1 only ever populates one entry and the frontend only ever renders one. This costs nothing extra to write correctly the first time and avoids a breaking change to every command's signature later (commands already take `repo_id: String` per the Data Flow section above, precisely for this reason).
- `cache/repo_cache.rs` and `watcher/git_watcher.rs` are already keyed/scoped per-repo-id, so opening a second repo concurrently just means a second cache entry and a second watcher instance — no architectural change, just no UI to trigger it yet.
- Frontend stores (`commitsStore`, `refsStore`, `workingTreeStore`, `selectionStore`) should be structured as **per-repo-id slices** from the start (e.g., `commitsStore.byRepo[repoId]`) rather than flat global state, even though v1's UI only ever shows `byRepo[activeRepoId]`. This is the one frontend decision that's expensive to retrofit later (flat-to-keyed state migration touches every consumer component) but nearly free to do correctly up front.
- Adding tabs later becomes: a tab-bar component that switches `repoStore.activeRepoId`, plus relaxing the "one repo open" assumption in `open_repo`'s command (already supports it) — no rewrite of commands, caching, or watcher layers required.
- Multi-*window* (as opposed to multi-tab) is a slightly different concern (separate OS windows via Tauri's multi-window API) — if ever needed, it's compatible with this design since `AppState` is already managed centrally and keyed by repo, not by window; a new window would just need to pick which `repoId` it displays.

## Sources

- Tauri v2 official docs — [Project Structure](https://v2.tauri.app/start/project-structure/), [Architecture](https://v2.tauri.app/concept/architecture/), [Calling Rust from Frontend](https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/develop/calling-rust.mdx) — HIGH confidence, official source
- [How to Reasonably Keep Your Tauri Commands Organized in Rust](https://dev.to/n3rd/how-to-reasonably-keep-your-tauri-commands-organized-in-rust-2gmo) — MEDIUM confidence, community best-practice, cross-checked against official project-structure docs
- `tauri-plugin-fs-watch` / `notify` crate — [tauri-apps/tauri-plugin-fs-watch](https://github.com/tauri-apps/tauri-plugin-fs-watch), [notify-rs/notify](https://github.com/notify-rs/notify) — HIGH confidence for the watcher mechanism choice; MEDIUM confidence on exact `.git`-specific watch-path recommendations (synthesized from general git-internals knowledge, not a specific documented Tauri+git tutorial)
- `react-virtualized` `ScrollSync` known lag issue — [bvaughn/react-virtualized#369](https://github.com/bvaughn/react-virtualized/issues/369) — HIGH confidence this is a real, long-standing, unresolved community-reported issue; used as direct evidence for the dual-scroll-container anti-pattern
- Observed git-client graph/virtualization implementation notes (tiled canvas graph synced to virtual rows, IndexedDB-backed topology cache) — [ionalexandru99/rebase-git PR #415](https://github.com/ionalexandru99/rebase-git/pull/415) — LOW/MEDIUM confidence (single community project, not an authoritative reference architecture), used only as corroborating evidence that "graph must scroll with virtual rows in the same container" is a real problem others have hit and solved the same direction
- Project's own prior research — `docs/01-research-competitors.md` (lane algorithm, git output formats, performance/correctness lessons from Fork/SourceGit/GitKraken/Sourcetree) — HIGH confidence, treated as authoritative input per task instructions, not re-derived
- `.planning/PROJECT.md` — requirement groups, constraints, out-of-scope boundaries — HIGH confidence, primary source of truth for scope and build order

---
*Architecture research for: Cross-platform Git GUI desktop client (Tauri v2 + Rust + React)*
*Researched: 2026-09-21*
