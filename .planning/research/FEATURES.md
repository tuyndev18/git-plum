# Feature Research

**Domain:** Cross-platform open-source Git GUI desktop client (git-plum)
**Researched:** 2026-09-21
**Confidence:** HIGH for table stakes and anti-features (direct issue-tracker evidence). MEDIUM for differentiators (mix of issue data and review/forum sentiment). MEDIUM for complexity estimates (derived from competitor bug reports, not from our own build).

This document builds on `docs/01-research-competitors.md` (market comparison, GitKraken screenshot analysis, graph algorithm). It does not repeat that material. Its job is to **pressure-test the MVP scope already committed in `.planning/PROJECT.md`**, using evidence from real issue trackers rather than assertion.

---

## Evidence Base

All claims below trace to one of these. Where a claim is opinion or weak-sourced, it is marked LOW.

| Source | What it gives us |
|---|---|
| `sourcegit-scm/sourcegit` issue tracker (1692 issues, queried by reactions) | The closest direct competitor. Tells us what users of a *free, open-source, fast* Git GUI actually ask for — the exact audience git-plum targets. |
| `desktop/desktop` (GitHub Desktop) open issues (1004, by reactions) | Tells us what users miss from a *deliberately simplified* client. This is the "what happens if you cut too much" dataset. |
| SourceGit issue #892 → PR #2070 | A complete natural experiment on the merge-conflict question. See "The #892 Experiment" below — it is the single most important finding in this document. |
| `Siomkin/GitLane` PR #425 | A real catalog of git-CLI write-layer bugs. Direct evidence for the "looks small but isn't" complexity flags. |
| Atlassian community threads, devRant, danmackinlay.name, eseth.org, sublimemerge.com docs, gitkraken.com help | Sentiment and design-philosophy evidence. Weaker; used only for anti-features and differentiators. |

### The #892 Experiment (read this before auditing scope)

SourceGit issue **#892, "[feature suggestion] Integrated merge-tool"** (10 reactions, opened Dec 2024) asked for a built-in conflict resolver. The maintainer closed it on **2025-12-29** with this reasoning:

> "There's no plan to implement a integrated merge-tool in `SourceGit`. Most editors support to be used as git merge tool. And it's very hard to implement a full-featured merge-tool (text editor)."

He further argued the external path is equivalent:

> "After configuring an external merge tool in `SourceGit` (`Preferences` window), its usage is the same as calling the integrated editor (both need to start a new process dialog)."

A user rebutted directly:

> "Unfortunately, this is not true. Showing actual differences instead of short 'conflict detected' is much more convenient."

**This reasoning is verbatim the reasoning in git-plum's PROJECT.md** ("Là một sản phẩm con riêng biệt… v1 chỉ phát hiện xung đột, liệt kê tệp, và mở trình soạn thảo bên ngoài").

What happened next: users kept pressing for over a year, and on **2026-01-26** a contributor opened PR **#2070 "Built-in merge conflict solver"** — merged **under four hours later**. Its description:

> "Adds a native merge conflict editor directly within SourceGit… The editor provides a **two-way** merge view with synchronized scrolling… Accept theirs/ours/both options per conflict block… full undo support… Right-click context menu integration — Keyboard shortcut support"

And the follow-up issue **#2168 "[Feature Request] Allow editing text in Merge Conflicts Window"** (10 reactions, open) shows users immediately wanted it *better*, not removed.

**Three conclusions, all load-bearing for this audit:**

1. The "conflict resolver is a separate sub-product" argument was tested in production by our nearest competitor and **lost**. Community pressure overturned it.
2. The *scoped* version is small. Not a three-pane BASE/LOCAL/REMOTE mergetool — a **two-way view over git's own conflict markers with accept-ours/accept-theirs/accept-both per block**. Reviewed and merged in hours, not a milestone.
3. The maintainer's error was equating "full-featured merge tool (text editor)" with "conflict resolver." They are different products. PROJECT.md currently makes the same conflation.

---

## Feature Landscape

### Table Stakes (Users Expect These)

Missing any of these and a developer who already knows Git goes back to the CLI or to Fork. Evidence column cites the source; "already in MVP" means PROJECT.md Active already covers it.

| Feature | Why Expected (evidence) | Complexity | Notes |
|---|---|---|---|
| Commit graph, multi-lane, colored, aligned to commit rows | This is the reason to open a GUI at all. `docs/01` §3.3. Reviewers frame it as "seeing it rendered as an interactive graph you can click" vs ASCII. | HIGH | Already in MVP. Core Value. Algorithm known (`docs/01` §4). |
| Virtualized commit list, smooth at scale | GitHub Desktop r13 "Even minimized and with no data loaded CPU is constantly being used"; Sourcetree 4.0 "painfully slow, pretty much unusable" (Atlassian community). Slowness is the #1 abandonment cause. | MEDIUM | Already in MVP. Fixed row height is the enabling constraint. |
| Diff viewer: unified + split, syntax highlight, whitespace toggle | Universal across all seven competitors. | MEDIUM | Already in MVP. CodeMirror 6 de-risks this. |
| Stage/unstage by file, and **by hunk** | `git add -p` is the workflow a GUI is supposed to beat. SourceGit closed "A way to stage/unstage/reset a chunk of file" (r3) as a real request. | **HIGH — see trap list** | Already in MVP, but see "Complexity Traps" — this is underestimated. |
| Discard by file and by hunk | Already in MVP. **Must be paired with a safety net** — see Anti-Features and the undo verdict. | MEDIUM | |
| Commit + amend | Universal. | LOW | Already in MVP. |
| Fetch / pull / push, incl. force-with-lease | Universal. | MEDIUM | Already in MVP. Auth is free via CLI credential helper (`docs/01` §7). |
| Branch create / checkout / delete / rename | Universal. | LOW | Already in MVP. |
| Merge, rebase, cherry-pick, revert | Sourcetree/GitKraken/Fork/SourceGit all have these. GitHub Desktop's lack of them is its top criticism. | MEDIUM | Already in MVP. |
| Stash create/list/apply/pop/drop | GitHub Desktop r201 "Stash specific file/s", r131 "does not seem to support multiple stashes", r20 "Apply stash from other branch" — three separate high-reaction issues from *lacking depth here*. | LOW–MEDIUM | Already in MVP. Multiple stashes is non-negotiable; GitHub Desktop's single-stash model is a known failure. |
| Tags create/delete | Universal. | LOW | Already in MVP. |
| **Merge conflict resolution, in-app** | **The #892 Experiment.** Also: "the built-in conflict resolution tool is something users miss when switching from GitKraken to Fork"; reviewers list "merge conflict resolution is a key time saver" as a daily-use feature. | MEDIUM (scoped 2-way) / HIGH (3-pane) | **Currently Out of Scope. This is a hidden table stake.** See verdict. |
| **Commit search / history search** | GitHub Desktop r96 "Commit Searchability and Navigation", r20 "Need a search option on commit history list", r39 "search for tags". SourceGit has 27 search-related issues — all refinements, meaning the feature is used constantly. | MEDIUM | **Not in PROJECT.md Active at all.** Gap. See verdict. |
| **Reset branch to commit (soft/mixed/hard)** | SourceGit lists Reset as a headline operation. Without it a user cannot undo a local commit — the most common recovery need. | LOW | **Not in PROJECT.md Active.** Gap. Trivial to add. |
| **"Open in terminal" / "Open in editor" / "Open in file manager"** | SourceGit has ~20 terminal issues, and they are *all* about launching an external terminal: "2025.17 missing 'Open In Terminal' option" (r2), "supports other shells not only git bash" (r2, 14 comments), "MacOS - support configuring other terminals", "add a short cut for open in terminal". **Zero** ask for an embedded terminal. | LOW | **Not in PROJECT.md.** This is the correct, cheap resolution of the "integrated terminal" question. |
| Multiple repos open (tabs) | `docs/01` §2.4 item 1 marks this MVP. GitHub Desktop's one-repo-at-a-time model is called "limiting" by reviewers; r224 "Support multiple windows". | MEDIUM | PROJECT.md says "nhớ danh sách repo gần đây" but does not explicitly commit to tabs. Clarify. |
| Correct behavior under dirty tree / rejected push / conflicted pull | See "Behavior Expectations" section — this is where toys are separated from tools. | MEDIUM | Not a feature; a quality bar across features. |
| Cross-platform, including Linux | GitHub Desktop r4851 "GitHub Desktop for Linux?" — **the single highest-reaction issue in either tracker, by 3.5x.** Fork's non-availability on Linux is its most-cited weakness. | MEDIUM | Already in MVP. This is git-plum's biggest structural advantage over Fork; do not treat Linux as an afterthought. |
| Auto-updater | SourceGit r14 "Feature request: add an updater" (13 comments) — 2nd-highest open request. For a downloaded open-source desktop app, no updater means users stay on the version they installed. | MEDIUM | `docs/01` §8 phase 6 mentions it; PROJECT.md Active does **not**. Gap. |

### Differentiators (Competitive Advantage)

| Feature | Value Proposition | Complexity | Notes |
|---|---|---|---|
| **Graph opens <1s, scrolls smoothly at 100k commits** | This *is* the wedge. Every competitor fails it somewhere: Sourcetree hangs, GitKraken eats GB of RAM, Electron clients "struggle with monorepos." Being demonstrably fastest is a headline a reviewer can verify in 30 seconds. | HIGH | Already the Core Value. Correct call. Keep it as the marketing claim. |
| **Command palette (Ctrl/Cmd+P) + full keyboard control** | Sublime Merge's most-praised trait: "gives keyboard-driven access to every action… without touching the mouse"; "search capabilities, keyboard shortcuts, and rendering performance are all class-leading." SourceGit closed "[Feature] Command Palette" (r6) and has two open issues asking for **customizable** shortcuts (r4, r3). Nobody in the Fork/SourceGit tier has nailed this. | MEDIUM | **Strongly recommend promoting into v1.** Cheap if the action layer is designed as a command registry from day one; expensive to retrofit. This is the highest value-to-cost differentiator available. |
| **Show the git command being run (command log)** | SourceGit already ships "Git command logs," and has a request: "对于任何git的操作，请将git 命令附带上" (attach the git command to every operation, inline or as tooltip). Sublime Merge is praised because it "constructs command lines for you, making it transparent what the software is doing for you, rather than magical." danmackinlay criticizes GitKraken precisely because "the relationship to the git command line is opaque… you remain dependent on the GUI." | **LOW** | **Promote to v1.** We already shell out to `git` — logging the argv is nearly free. It converts our architecture into a *user-visible feature*, builds trust for an open-source tool, and is the direct antidote to the "hiding Git" anti-feature. Highest ROI item in this document. |
| **Truly open source + free + all three platforms** | Fork: closed, paid, no Linux. GitKraken: paid for private repos, heavy. Sourcetree: effectively abandoned ("No significant updates since 2019/2020", forced Atlassian account). SourceGit: **currently has an open issue titled "[Announcement] Find volunteers to take over the project"** — the maintainer is seeking a successor. The open-source niche is actively destabilizing. | — | This is positioning, not code. But it means **timing is favorable** and a well-maintained newcomer has an unusual opening. |
| **AI commit message generation (local-model capable, opt-in)** | Genuinely contested — see below. | MEDIUM | Keep, but **demote its priority**. |
| Conventional-commit helper / commit template | SourceGit shipped both (README: "Built-in conventional commit message helper"; closed r4 "Add Commit template"). Cheap, deterministic, and serves the same need as AI for many users. | LOW | Consider as a v1 companion to (or partial substitute for) AI. |
| Per-repo visual identity / repo list with branch + dirty count | SourceGit closed r4 "Display current branch and number of uncommitted files in repositories tab"; GitHub Desktop r21 "Add branch name to UI repository selection menu", r28 "Change the order of repositories". Small, repeatedly requested. | LOW | Nice polish, cheap. |

#### On AI commit messages: is it valued or a gimmick?

Honest answer from the evidence: **mildly valued, definitely not a switching reason, and actively distrusted by a vocal segment.** Confidence MEDIUM.

For:
- SourceGit ships it and it is not a top complaint source, i.e. it works well enough to be uncontroversial.
- The most-cited defense of the category: *"There's three types of people: those who already write excellent commit messages explaining the why, those who write decent ones explaining the what, and those who write garbage commit messages. Empirically, the first set is small. This tool will help the middle type be more efficient, and help the last type drastically."*
- GitKraken puts "Recompose commit with AI" in its detail panel (`docs/01` §2.1), so the framing is validated by a commercial product.

Against:
- The recurring critique is structural, not about model quality: *"A commit that restates what the code does is just a worse version of the diff — the message needs to carry the why, the reason that isn't in the code."* AI sees only the diff, so it produces the *what*.
- Reported trust damage: a commit claiming "fixed critical bug" that only updated a README.
- The one AI-related SourceGit issue with traction is **"[Enhancement] Letting the AI improve the commit instead of generating it"** (r3, open) — users asking to be *assisted*, not *replaced*. That is a meaningful design signal.

**Recommendation:** keep AI in v1 (it is budgeted at only 3–5 days per `docs/01` §8, and the provider-abstraction decision is sound), but:
- Position "improve/refine my draft" at least as prominently as "generate from scratch" — that is what users asked for.
- Do **not** market AI as the differentiator. Speed, keyboard-first UX, and transparency are the switching reasons. Marketing AI as the headline would put git-plum in direct comparison with GitKraken on GitKraken's terms.
- Keep opt-in + local-model support. Already a PROJECT.md constraint; it is correct and is itself a trust differentiator.

### Anti-Features (Commonly Requested, Often Problematic)

| Feature | Why Requested | Why Problematic | Alternative |
|---|---|---|---|
| **Embedded terminal emulator** | It is in GitKraken's toolbar (`docs/01` §2.1), so it looks standard. | Requires a full PTY + VT100 emulator per platform (ConPTY on Windows). Large surface, endless shell/encoding bugs, and users already have a better terminal. Evidence is decisive: across ~20 SourceGit terminal issues, **every one** concerns *launching an external terminal*; not one asks for an embedded one. | "Open in Terminal" with a **configurable shell** (the repeated ask: "supports other shells not only git bash", macOS terminal choice) + a keyboard shortcut. Plus "Open in Editor", "Open in File Manager". Hours of work, satisfies the entire demand. |
| **Hiding Git concepts behind friendly verbs ("Sync")** | Lowers the onboarding cliff; GitHub Desktop's model. | Our users already know Git. Abstraction actively harms them: GitHub Desktop's top issues are all *"let me do the real Git thing"* — manage multiple accounts (r1376), gpg signing (r340), stash individual files (r201), multiple stashes (r131), checkout a remote branch whose name collides (r47). danmackinlay's core complaint about GitKraken is that "the relationship to the git command line is opaque… you remain dependent on the GUI." | Use Git's own vocabulary. Show the command. Never invent a verb that maps to more than one git operation. |
| **Three-pane BASE/LOCAL/REMOTE merge editor** | It is what GitKraken and Tower advertise. | Argued against on technical grounds: classic 3-way tools do a "blind diff" of LOCAL vs REMOTE and ignore the work git already did, forcing users to "re-resolve those differences by hand" and producing "visual clutter without added value" (eseth.org). Sublime Merge has open issues (#1079, #1869) about its 3-way UI being confusing. It is also the framing that made SourceGit's maintainer call the feature impossible. | **Two-way view over git's conflict markers**, accept-ours / accept-theirs / accept-both per block — exactly what SourceGit PR #2070 shipped. Optionally use git's `merge.conflictStyle=zdiff3` / `mergetool.hideResolved` (git ≥2.31) so only genuine conflicts are shown. Much smaller, and arguably better. |
| **Issues / PR / Teams / Jira integrations** | GitKraken's sidebar shows them (`docs/01` §2.1); they look like parity items. | OAuth per provider, N different APIs, token storage, rate limits, and permanent maintenance drag. Even Fork, a mature paid product, does not do this — and its PR support is described as requiring "the browser for full PR review workflows" without that being a common abandonment reason. | Correctly **Out of Scope**. A cheap 90% substitute: linkify issue references in commit messages (SourceGit's `.issuetracker` file, closed r5) — pure regex, no auth. Consider for v1.x. |
| **"Safe" wrappers that block or auto-decide destructive operations** | Prevents user error. | Two failure modes, both observed. (a) Blocking: GitHub Desktop r13 "'Force-pull' button in dropdown" and r47 "unable to checkout remote branch when existing local branch has the same name" — users blocked from legitimate operations, with no escape hatch. (b) Auto-deciding: SourceGit "[BUG] Conflict resolution is always selecting OURS for binary files and not prompting for select" — the wrapper silently chose wrong. | Never block. Explain the risk, name the exact command, require confirmation for destructive ops, and **make recovery possible** (see undo verdict). Default to force-*with-lease*, never bare force. |
| **Cloud sync / accounts / telemetry** | Analytics, cross-device settings. | Kills trust for a local open-source tool. Sourcetree's forced Atlassian login is one of the loudest abandonment complaints on devRant and the Atlassian forums. | Correctly **Out of Scope.** Keep it that way. Note a related real request: SourceGit r5 "[Feature] Separate settings from state (`preference.json`)" — users want config in a plain, diffable, sync-it-yourself file. |
| **Big-bang UI redesigns** | Modernization. | SourceGit r11, 25 comments: **"Revert to the old(2026.19) UI."** One of its highest-engagement open issues. Redesigns of a tool people use hourly generate immediate backlash. | Not a v1 concern, but a governance note: pick the visual language in v1 and hold it. |
| Markdown rendering, spell check, SVG preview, image diff modes, avatars everywhere | Each appears in SourceGit's tracker (Markdown r5, spell check r4, SVG r3, image diff in README, avatars `docs/01` §2.4 item 19). | Individually cheap, collectively a bottomless polish backlog that produces zero switching. This is the "feature bloat" failure mode `docs/01` §7 already flags as the #1 risk. | Gravatar avatars are already in MVP and are fine (they're in the graph, which is the Core Value). Everything else → backlog, revisit only after v1 users ask. |

---

## Complexity Traps: "Looks Small But Isn't"

These are the estimates most likely to be wrong. Evidence is from competitors' bug trackers, i.e. the bugs they actually shipped.

| Feature | Naive estimate | Real complexity | Why |
|---|---|---|---|
| **Hunk staging** | Small ("rebuild a patch, `git apply --cached`") — `docs/01` §5.5 presents it as ~a paragraph | **HIGH** | GitLane PR #425 enumerates the actual failures: patches read as text **lose the trailing CR on CRLF files**; **non-UTF-8 bytes become replacement characters** (silently corrupting the file on stage); **clean-filter warnings on stdout** get parsed as patch content and cause permanent false "stale" detection. Sourcetree shipped SRCTREE-3047: `git apply` refuses whitespace-only reverts. **Mitigations: read and write patches as raw bytes, never as a decoded string; pass `--recount`; set `core.autocrlf`-aware handling; isolate stderr from stdout.** Budget 2–3x. |
| **Line-level staging** | "Hunk staging but finer" | **HIGH, and strictly harder** | Must recompute `@@` headers and convert unselected +/- lines into context lines (`docs/01` §5.5 notes this). GitLane #425 documents the non-obvious killer: **zero-context fragments anchored to worktree line numbers misalign when other pending changes exist**; the fix was to emit *full hunks with unselected changes neutralized*, not minimal fragments. SourceGit has an open "[Feature Request] Easier per line staging" (r3) — i.e. even a mature client hasn't made this pleasant. **Recommendation: keep line-level staging out of v1.** It is not currently in PROJECT.md Active; keep it that way. |
| **Rename detection** | "`--find-renames` flag, done" | **MEDIUM, with UI cost** | The flag is easy; the *UI* is not. GitHub Desktop r21: "Desktop does not display diff changes after renaming and staging file"; r14 "File renaming/moving support". Renames must render as one row with old→new path, must survive the flat/tree view toggle, and the file-history view must follow renames (`--follow`), which interacts badly with pagination. |
| **Submodules** | "Git CLI handles it, free" | **HIGH — do not touch in v1** | **82 issues in GitHub Desktop alone**, including a dedicated "[Tracking issue] Improve the ease of working with submodules" (r15). Concrete failures: "unable to commit submodules changes from superproject" (r20), "clone with inaccessible submodule fails the clone process" (r11, open), "Switching to a branch with submodules is not possible" (r7, **73 comments**), "hangs while trying to clone a repository containing a submodule". PROJECT.md's "call the CLI so submodules just work" is true for *correctness* but false for *UX*. **Explicit v1 stance needed: display submodule entries read-only, never try to manage them, and never hang the UI on a submodule fetch.** |
| **Large files / binary files** | "Diff viewer handles it" | **MEDIUM** | Needs a size threshold before a diff is even attempted, binary detection, and a graceful "binary file, N bytes" placeholder. Without it a single 200MB file freezes the app. Related: GitHub Desktop r27 "show image previews for images tracked by lfs", r14 "Display SVG file and diff not as plaintext". Minimum v1 bar: **do not hang, and say why**. |
| **Merge/rebase in-progress state** | "Run the command, show the result" | **MEDIUM** | Every operation can leave the repo mid-merge/mid-rebase/mid-cherry-pick. The UI must detect this on open (`.git/MERGE_HEAD`, `rebase-merge/`, `CHERRY_PICK_HEAD`) and offer continue/abort/skip. SourceGit bug: "[BUG] Unstaging all Staged file-changes resets Conflicts and exits Merging state!" — state machine bugs here corrupt real work. **This is required even in the "detect conflicts only" plan and may already be under-budgeted.** |
| **Locale / environment contamination of `git` output** | "Parse the output" | **MEDIUM, and invisible until it breaks** | GitLane #425: they had to pin `LC_MESSAGES=C` and clear `LC_ALL`/`LANGUAGE` so parsing survives non-English users; pin `log.showSignature=false` so GPG verification text doesn't corrupt `git log` parsing; clear inherited `GIT_AUTHOR_*`/`GIT_COMMITTER_*`; pin `--cleanup=whitespace` so a user's `commit.cleanup=strip` doesn't silently delete `#`-prefixed lines from commit messages; pin `diff.noprefix`/`diff.external`/`format.coverLetter` so user config doesn't break patch parsing. **This is a cross-cutting requirement on the Rust git layer and belongs in the foundation phase, not discovered in phase 5.** It will otherwise produce a long tail of "works on my machine" bug reports from exactly the international users an open-source project attracts. |
| **Amend** | "`git commit --amend`" | LOW-MEDIUM | Must correctly handle amending a pushed commit (warn), amending with a partially staged tree, and preserving the original author. |
| Auto-updater | "Ship a binary" | MEDIUM | Three platforms, code signing (macOS notarization, Windows Authenticode), and a hosted update feed. SourceGit's users asked for Velopack. GitHub Desktop r105 "Opt-out of auto-updating new app versions" — make it opt-out-able. |

---

## Feature Dependencies

```
Rust git-CLI layer (env pinning, raw-byte IO, porcelain=v2 parsing)
    ├──requires──> nothing (foundation)
    └──enables──> EVERYTHING below

Commit graph  ──requires──> git log --topo-order pagination + lane algorithm
    └──enables──> commit detail panel ──enables──> file history
                                       └──enables──> blame (blame links back to graph)

Diff viewer (CodeMirror 6)
    ├──enables──> hunk staging
    │                 └──enables──> line-level staging  [DEFER]
    ├──enables──> discard by hunk
    └──enables──> conflict resolver (2-way view = a diff view + per-block actions)

Working directory panel ──requires──> git status --porcelain=v2
    └──enables──> stage/commit/amend/discard

Merge / rebase / cherry-pick / revert / pull
    └──ALL require──> in-progress state machine (detect MERGE_HEAD, rebase-merge/, CHERRY_PICK_HEAD;
                      offer continue / abort / skip)
            └──requires──> conflict detection + file list
                    └──SHOULD enable──> conflict resolver   <-- currently cut; see verdict

Command registry (every action = named, parameterized command)
    ├──enables──> command palette
    ├──enables──> customizable keyboard shortcuts
    └──enables──> git command log  ──enables──> user trust / "not magical"

Reflog reader
    └──enables──> undo of ref-moving operations (commit, checkout, reset, merge, rebase, branch delete)
       (does NOT enable undo of discard — discard destroys untracked/unstaged data with no reflog entry)

Discard  ──DANGEROUSLY PAIRS WITH──> absence of undo
    └──mitigated by──> auto-stash-before-discard (cheap) OR confirmation + exact-command display

AI commit message ──requires──> staged diff + provider abstraction + OS keychain
    (leaf node — depends on staging, nothing depends on it. Safe to schedule last or drop.)
```

### Dependency Notes

- **Conflict resolver requires almost nothing new.** It needs the diff viewer (have it), the conflict file list (in MVP), and the in-progress state machine (required by merge/rebase anyway). The incremental cost over "detect and list" is the per-block accept-ours/theirs/both actions and writing the resolved file. This is why SourceGit's version merged in four hours.
- **Command palette must be designed in, not bolted on.** If UI actions are written as inline click handlers, a palette later requires refactoring every one. If they are registered commands from day one, palette + customizable shortcuts + command log all fall out nearly free. **This is a phase-1 architectural decision with a phase-6 payoff.** Two separate SourceGit issues ask for customizable shortcuts; retrofitting is why they don't have them.
- **Git command log is a byproduct of the chosen architecture.** We already build an argv and spawn a process. Recording and displaying it is near-zero marginal cost and is our strongest trust signal.
- **Undo of discard ≠ undo of everything.** Reflog-based undo covers ref moves. Discard has no reflog entry, yet it is the operation people actually lose work to. These are two different mechanisms; PROJECT.md's single "Undo/Redo" bullet conflates them, which is why it looks uniformly expensive.
- **Blame depends on file history, which is already in MVP.** "Xem lịch sử của riêng một tệp" is committed. Blame is `git blame --line-porcelain` plus a gutter renderer over a viewer we already have. The dependency is already paid for.
- **Environment pinning must precede every write operation.** Retrofitting it means re-testing every command.

---

## VERDICT: Audit of the PROJECT.md Out-of-Scope List

For each deferred item: **safe to defer**, **hidden table stake**, or **reframe**.

### 1. Merge conflict GUI resolver — **HIDDEN TABLE STAKE. Promote a scoped version into v1.** (Confidence: HIGH)

PROJECT.md: *"Là một sản phẩm con riêng biệt. v1 chỉ phát hiện xung đột, liệt kê tệp, và mở trình soạn thảo bên ngoài."*

This is the strongest finding in this research. See "The #892 Experiment" above: our nearest competitor deployed this exact reasoning, held it for 13 months under sustained user pressure, and reversed. The reversal shipped in **under four hours of review time** because the scoped feature is small.

The reasoning error is conflating two products:
- A **merge tool** (a text editor with three panes, semantic merging, its own diff engine) — genuinely a sub-product, correctly out of scope, and arguably a bad design anyway per eseth.org.
- A **conflict resolver** (a two-way view over the conflict markers git already wrote, with accept-ours / accept-theirs / accept-both per block, then write file + `git add`) — a diff view with three buttons.

Additional weight:
- Reviewers list conflict resolution among the handful of genuinely daily-use GUI features: *"merge conflict resolution is a key time saver."*
- It is the specific thing users say they miss when moving from GitKraken to Fork.
- Conflicts are the moment of maximum user stress. "Conflict detected in 3 files, go use another program" is precisely when a tool feels like a toy. The user rebuttal in #892 nails it: *"Showing actual differences instead of short 'conflict detected' is much more convenient."*

**Recommendation:** promote a **two-way conflict resolver** into v1. Set `merge.conflictStyle=zdiff3` and consider `mergetool.hideResolved` so only real conflicts surface. Explicitly keep out of v1: free-text editing in the conflict view (SourceGit's #2168 — that *is* the text-editor sub-product), and semantic/binary merging. Keep the "open in external tool" escape hatch for the hard cases.

Also promote alongside: **continue / abort / skip** for merge, rebase and cherry-pick. Non-negotiable — without abort, a user who starts a merge in git-plum can be stranded in a state they can only exit via the CLI.

### 2. Blame — **Borderline. Recommend promoting to late v1; safe to defer only if schedule is tight.** (Confidence: MEDIUM)

PROJECT.md: *"Hữu ích nhưng không nằm trong luồng công việc hằng ngày."*

Partly right — blame is not in the commit loop. But the evidence says it is in the *investigation* loop, which is half of why people open a GUI:

- GitHub Desktop closed **four separate blame requests**, the top one at r42 ("Add a way to view 'git blame'"). For a deliberately minimal client, that is a lot of independent demand.
- SourceGit has **13 blame issues, and their character is telling**: almost all are bugs and refinements ("no ability to go to the previous revision", "Blame selection doesn't stick", "Cannot select the last line", "crashing when file was moved", "pass -w flag"). People only file that class of bug about features they use constantly.
- Reviewers name "investigating history" as one of three core GUI jobs.
- GitKraken puts Blame directly in the diff toolbar (`docs/01` §2.2) — one click from the main reading flow, not buried.

Cost is genuinely low: `git blame --line-porcelain`, plus a gutter over the CodeMirror viewer we already have, plus click-to-jump into the graph. **The dependency (file history viewer) is already in MVP.**

**Recommendation:** keep it out of the critical path but pull it into the v1 polish phase if the schedule holds. Ship the read-only version (author + date + short SHA gutter, click to select that commit in the graph). Explicitly defer: "blame previous revision" navigation, `-w`, and move-detection.

### 3. Interactive rebase (drag-and-drop) — **SAFE TO DEFER as a UI. But two cheap pieces should be promoted.** (Confidence: HIGH)

PROJECT.md: *"Phức tạp cao, rủi ro mất dữ liệu. v2."* Correct.

Evidence supports deferral: SourceGit's 22 interactive-rebase issues are overwhelmingly *bugs* — "Editing a commit with an interactive rebase is not properly handled" (19 comments), "endless 'Loading' in new interactive rebase window", "Unable to perform interactive rebase with merge commit", "Unexpected reordering of fixup commit", "Interactive rebase difficult to try again". This is a feature that is expensive to get right and generates a long bug tail. GitHub Desktop lacks it and it is not among its top complaints.

**But promote these two, which users conflate with "interactive rebase" and which are individually cheap:**
- **Squash / fixup / reword of recent commits** via targeted commands. SourceGit closed "Support for `git commit --fixup` and `git rebase -i --autosquash`" at **r9 with 14 comments** — one of its highest-engagement requests. SourceGit's README lists "Amend/Reword/Squash" as first-class, separate from interactive rebase. GitHub Desktop r40 "Edit Commit Message." Reword of HEAD is already covered by amend (in MVP); reword of an older commit and squash-with-parent are the gap.
- **Reset branch to commit (soft / mixed / hard)** — already flagged above as a missing table stake. It is the universal escape hatch.

Deferring the *drag-and-drop rebase editor* is correct. Deferring *all history editing* is not.

### 4. Integrated terminal — **SAFE TO CUT, and the reasoning is correct. But ship "Open in Terminal" instead, in v1.** (Confidence: HIGH)

PROJECT.md: *"Người dùng đã có terminal riêng. Giá trị thêm vào thấp so với công sức."* The evidence backs this unusually cleanly.

Across SourceGit's ~20 terminal-related issues, **every single one is about launching an external terminal**, and the recurring theme is *configurability*: "supports other shells not only git bash" (14 comments), "MacOS - support configuring other terminals", "Allow specifying startup options for Shell/Terminal", "add a short cut for open in terminal", and a regression report when the option briefly disappeared ("2025.17 missing 'Open In Terminal' option"). **Zero** requests for an embedded emulator.

**Recommendation:** cut the embedded terminal permanently (not "v2" — permanently). Add to v1: "Open in Terminal" with a **user-configurable shell and working directory**, plus "Open in Editor" and "Open in File Manager", plus keyboard shortcuts. Cost: hours. This also completes the "don't hide Git" posture — the escape hatch to the CLI is always one keystroke away.

### 5. Undo / Redo of git operations — **SPLIT THE ITEM. Full undo: safe to defer. Discard safety: hidden table stake.** (Confidence: MEDIUM-HIGH)

PROJECT.md: *"Rất khó làm đúng (phải theo dõi reflog, xử lý thao tác không thể đảo ngược). GitKraken mất nhiều năm mới ổn định."* True for the general feature. But the bullet bundles two very different things.

**Defer (safe):** general undo/redo across checkout, reset, merge, rebase, branch delete, remote removal. GitKraken's list of supported undo actions is long and each needs its own inverse; this genuinely took them years.

**Do not defer:** protection for **discard**. This is the one operation in the MVP that destroys data with **no reflog entry and no recovery path**, and git-plum ships discard-by-file *and* discard-by-hunk in v1. Evidence that this is where users actually lose work:
- GitKraken's undo explicitly covers Discard, and the most-quoted user testimonial is exactly that: *"accidentally discarded all my changes but now they are back again."*
- Tower is singled out for its discard UX and for CMD+Z on a wrongful discard.
- The Atom/GitHub package shipped a discard that deleted users' files (issue #1001), and the team's own remediation plan was to relabel it "Delete File"/"Move to Trash" for added files.
- Analysis of the problem is blunt: *"Anything you lose that was never committed is likely never to be seen again."*

**Recommendation for v1 — pick at least one, ideally both:**
- **Auto-stash-before-discard**: before any discard, silently `git stash create` the affected paths and keep the object reachable (e.g. a `refs/git-plum/trash/<timestamp>` ref). Add a "Recently discarded" list to restore from. Cheap — it's one git command and one ref write — and it converts the single most dangerous MVP operation into a recoverable one. This is a *differentiator* many paid clients lack.
- **Honest confirmation dialogs**: name the exact files, say explicitly that untracked files will be deleted (not just reverted), and show the command. Follow the Atom lesson: for untracked files the verb is "Delete", not "Discard".

Reflog-backed undo of ref-moving operations can then arrive in v1.x as a natural extension.

### 6. Worktree management — **SAFE TO DEFER.** (Confidence: MEDIUM)
Low demand in both trackers (SourceGit's worktree issues are minor: "Add search input for branches when adding a worktree"). `docs/01` §2.4 already excludes it. One caveat: if a user *has* worktrees, the sidebar should not misrepresent the repo. Detect and display them read-only; don't manage them.

### 7. Issues / PR / Teams integrations — **SAFE TO DEFER. Correctly reasoned.** (Confidence: HIGH)
OAuth × N providers is a milestone, as PROJECT.md says. Fork omits it and thrives. Cheap partial win for v1.x: regex-linkify issue references in commit messages (SourceGit's `.issuetracker`, r5), no auth needed.

### 8. Git LFS without dedicated UI — **MOSTLY SAFE. One caveat.** (Confidence: MEDIUM)
Correct that CLI invocation makes LFS work transparently. Caveat: LFS *pointer files* diff as meaningless text blobs. GitHub Desktop r27 asks for "image previews for images tracked by lfs." **Minimum v1 bar: detect an LFS pointer and render "LFS object (N MB)" rather than the pointer text.** Small, prevents a confusing first impression on LFS repos.

### 9. Cloud sync / accounts / telemetry — **SAFE TO CUT PERMANENTLY.** (Confidence: HIGH)
Strongly supported. Sourcetree's forced Atlassian account is among its loudest abandonment complaints. Keep it out. Related real want: plain-text, hand-editable settings separated from UI state (SourceGit r5).

### 10. Optimized support for >100k-commit repos — **SAFE TO DEFER, with the stated design discipline.** (Confidence: MEDIUM)
"Design to withstand it, don't build a disk cache in v1" is the right call. Pagination + virtualization + Rust lane computation is the 90% solution. But note: **performance is the Core Value and the headline claim.** Benchmark against a real large repo (Linux kernel, or Chromium) *during* phase 1, not at the end — if the architecture can't hit the number, that must surface while it's still cheap to change.

### Gaps: In Scope But Missing From PROJECT.md Active

These are not on the Out-of-Scope list, so they are not deliberate exclusions — they appear to be oversights.

| Missing | Evidence | Complexity | Recommendation |
|---|---|---|---|
| **Commit / history search** | GH Desktop r96, r20, r39; 27 SourceGit search issues | MEDIUM | **Add to v1.** A 100k-commit graph you cannot search is a browsing toy. Minimum: filter by message/author/SHA. Pickaxe (`-S`) can wait. |
| **Reset branch to commit (soft/mixed/hard)** | Universal in competitors; the primary recovery operation | LOW | **Add to v1.** Trivial given branch operations already exist. |
| **Merge/rebase/cherry-pick continue / abort / skip** | SourceGit state-machine bugs; required by the conflict flow | MEDIUM | **Add to v1.** Already implied by "conflict detection" but must be explicit — without abort, users get stranded. |
| **Open in Terminal / Editor / File Manager (configurable)** | ~20 SourceGit issues, all asking for this | LOW | **Add to v1.** Replaces the cut embedded terminal. |
| **Git command log (show the argv)** | Sublime Merge praise; SourceGit ships it; danmackinlay's core critique of GitKraken | LOW | **Add to v1.** Near-free given the architecture; top trust differentiator. |
| **Auto-updater** | SourceGit r14, 2nd-highest open request | MEDIUM | **Add to v1** (`docs/01` §8 already budgets phase 6 for it; PROJECT.md should say so). Make it opt-out-able (GH Desktop r105). |
| **Multiple repos in tabs** | `docs/01` §2.4 item 1 says MVP; GH Desktop r224 | MEDIUM | **Clarify in PROJECT.md.** Currently only "recent repo list" is committed. |
| **Environment pinning in the git layer** | GitLane PR #425, exhaustively | MEDIUM | **Add as a phase-1 foundation requirement.** Not a user-facing feature but a correctness prerequisite; retrofitting means re-testing everything. |
| **Binary / large-file / LFS-pointer handling in diff** | GH Desktop r27, r14 | LOW-MEDIUM | **Add to v1** as a minimum "don't hang, say what it is" bar. |
| **Explicit submodule non-goal** | 82 GH Desktop issues | LOW (as a stated boundary) | **Add to Out of Scope explicitly**, with the display-read-only/never-hang rule. Currently PROJECT.md implies submodules "just work" via CLI — true for correctness, false for UX. |

### Recommended Cuts From Current MVP

Nothing in the current Active list is outright wrong. Two things to re-rank rather than cut:

| Item | Recommendation | Why |
|---|---|---|
| **AI features (generate + recompose + 3-provider abstraction)** | Keep, but **schedule last** and treat as droppable. Prioritize "improve my draft" over "generate from scratch." | Contested value (see sentiment analysis). It is a leaf dependency — nothing else needs it. It is the right thing to trade away if the conflict resolver, search, and reset need the time. The 3-provider abstraction is only ~3 days per the Key Decisions table, so keeping it is cheap; just don't let it precede table stakes. |
| **Full-repo file tree at a commit ("View all files", `docs/01` §2.3)** | Not currently in PROJECT.md Active — **keep it out.** | Screenshot-derived from GitKraken, but no user-demand evidence found. The changed-files list (flat + tree) is what people use. Flagging so it isn't added later by screenshot-parity reasoning. |
| **Line-level staging** | Confirm it stays out of v1. | HIGH complexity per GitLane #425; even SourceGit hasn't made it pleasant (open r3). Hunk-level is the table stake; line-level is not. |

---

## MVP Definition (Revised Recommendation)

### Launch With (v1)

Everything in PROJECT.md Active, **plus** the following promotions, **minus** nothing:

- [ ] **Two-way merge conflict resolver** (accept ours / theirs / both per block; no free-text editing) — the #892 Experiment; conflicts are peak user stress
- [ ] **Merge / rebase / cherry-pick: continue, abort, skip** — without abort, users get stranded outside the app
- [ ] **Commit + history search** (message / author / SHA) — a 100k-commit graph without search is unusable
- [ ] **Reset branch to commit (soft / mixed / hard)** — the universal recovery operation
- [ ] **Discard safety net** (auto-stash-before-discard + "Recently discarded" restore; honest "Delete" wording for untracked files) — the only unrecoverable destructive op in the MVP
- [ ] **Open in Terminal / Editor / File Manager, configurable, with shortcuts** — replaces the cut embedded terminal; satisfies 100% of observed demand
- [ ] **Git command log** — near-free, top trust signal, direct antidote to "hiding Git"
- [ ] **Command registry underpinning a command palette + customizable shortcuts** — architectural in phase 1, differentiating at launch
- [ ] **Squash / fixup / reword of a non-HEAD commit** (targeted commands, not a drag-and-drop editor) — SourceGit r9 / 14 comments
- [ ] **Binary + large-file + LFS-pointer handling in the diff viewer** — don't hang; say what it is
- [ ] **Auto-updater** (opt-out-able) — SourceGit's 2nd-highest open request
- [ ] **Environment pinning in the Rust git layer** — correctness prerequisite; must be phase 1
- [ ] **Blame (read-only gutter)** — promote if schedule holds; dependency already paid for
- [ ] AI commit message generation — **keep, schedule last, treat as droppable**

### Add After Validation (v1.x)

- [ ] Reflog-backed undo for ref-moving operations — natural extension of the discard safety net
- [ ] Blame, if it slipped from v1; plus previous-revision navigation and `-w`
- [ ] Issue-reference linkification in commit messages (no OAuth)
- [ ] Pickaxe search (`-S` / `-G`)
- [ ] Conventional-commit helper / commit templates
- [ ] Branch diff, patch import/export, archive

### Future Consideration (v2+)

- [ ] Drag-and-drop interactive rebase — high complexity, long bug tail; defer confirmed
- [ ] Line-level staging — high complexity, low incremental value over hunk staging
- [ ] Worktree management — low demand
- [ ] Submodule *management* (display-only in v1) — 82 GH Desktop issues; enter deliberately or not at all
- [ ] GitFlow, bisect UI
- [ ] Issues / PR / Teams integrations — own milestone

### Permanently Out (never build)

- [ ] Embedded terminal emulator — zero observed demand; "Open in Terminal" is the whole ask
- [ ] Cloud sync, accounts, telemetry — kills trust for a local open-source tool
- [ ] Three-pane BASE/LOCAL/REMOTE merge editor — worse design than the two-way approach, and a sub-product
- [ ] Friendly abstractions over Git verbs — our users know Git; hiding it is the top criticism of GitKraken and GitHub Desktop alike

---

## Feature Prioritization Matrix

| Feature | User Value | Impl. Cost | Priority |
|---|---|---|---|
| Fast commit graph (<1s, 100k commits) | HIGH | HIGH | P1 — Core Value |
| Diff viewer (unified + split, syntax) | HIGH | MEDIUM | P1 |
| Hunk staging + commit + amend | HIGH | **HIGH (trap)** | P1 |
| Fetch / pull / push / branch ops | HIGH | MEDIUM | P1 |
| **Two-way conflict resolver** | HIGH | MEDIUM | **P1 — promote** |
| **Continue / abort / skip** | HIGH | MEDIUM | **P1 — promote** |
| **Commit search** | HIGH | MEDIUM | **P1 — promote** |
| **Reset to commit** | HIGH | LOW | **P1 — promote** |
| **Discard safety net** | HIGH | LOW | **P1 — promote** |
| **Git command log** | MEDIUM | **LOW** | **P1 — best ROI in this doc** |
| **Open in Terminal / Editor (configurable)** | MEDIUM | **LOW** | **P1 — promote** |
| **Command registry + palette + shortcuts** | HIGH | MEDIUM | **P1 — architectural, must be early** |
| Stash (multiple, per-file) | MEDIUM | LOW–MEDIUM | P1 |
| Cherry-pick / revert / tags | MEDIUM | MEDIUM | P1 |
| Squash / fixup / reword non-HEAD | MEDIUM | MEDIUM | P1 |
| Binary / large-file / LFS-pointer handling | MEDIUM | LOW–MEDIUM | P1 |
| Auto-updater | MEDIUM | MEDIUM | P1 |
| Environment pinning (git layer) | HIGH (correctness) | MEDIUM | P1 — phase 1 |
| Multi-repo tabs | MEDIUM | MEDIUM | P1 |
| Blame (read-only) | MEDIUM | LOW–MEDIUM | P1 if schedule holds, else P2 |
| AI commit message | MEDIUM (contested) | MEDIUM | P2 — schedule last, droppable |
| Reflog-backed general undo | MEDIUM | HIGH | P2 |
| Issue linkification | LOW | LOW | P2 |
| Interactive rebase (drag-drop) | MEDIUM | HIGH | P3 |
| Line-level staging | LOW | HIGH | P3 |
| Worktrees / submodule management | LOW | HIGH | P3 |
| PR / Issues / Teams integrations | LOW (for our users) | HIGH | P3 |
| Embedded terminal | **~ZERO (evidenced)** | HIGH | **Never** |

---

## Behavior Expectations: The Edge Cases That Separate a Toy From a Tool

For each core flow, what a good client must *do*. These are acceptance criteria, not features.

### Stage → commit
- Staging a hunk must not corrupt the file. Read/write patches as **raw bytes**; preserve CRLF; never decode to a string. (GitLane #425)
- Show `+N / -M` line counts (SourceGit closed r4 asking for this).
- Run `pre-commit` and `commit-msg` hooks by default (we get this free from the CLI). Offer an explicit `--no-verify` toggle — and make sure it *works in the conflict path too* (SourceGit bug #2707: "No-Verify does not work when resolving merge conflicts").
- Respect `commit.template`; pin `--cleanup=whitespace` so a user's `commit.cleanup=strip` doesn't silently delete `#`-prefixed lines. (GitLane #425)
- Preserve the message draft across app restart and repo switch. Never silently discard typed text.
- Amending a commit that is already pushed: warn, don't block.

### Pull with local changes
- Detect the conflict-with-local-changes case **before** running, and offer: stash → pull → pop, or cancel. Never leave the user with a half-failed pull and no explanation.
- Surface the choice of merge vs rebase vs ff-only explicitly, honoring `pull.rebase`. GitKraken's pull button has a mode dropdown (`docs/01` §2.1) for exactly this.
- If the auto-stash-pop conflicts, say so clearly and route into the conflict resolver.
- Related bug to avoid: SourceGit #2582 "Pulling fails if merge is selected and conflicts exists."

### Push rejected (non-fast-forward)
- Do not show raw git stderr and stop. Explain: "remote has N commits you don't have."
- Offer exactly three actions: **Pull and merge**, **Pull and rebase**, **Force push (with lease)**.
- Default any force to `--force-with-lease`, never bare `--force`. State the difference in the dialog.
- Show ahead/behind counts before and after. (`git for-each-ref` with `%(upstream:track)` already gives this — `docs/01` §5.2.)

### Branch switch with dirty tree
- Try the checkout; git itself permits it when files don't conflict. Only intervene on actual failure.
- On failure, offer: **stash and switch** (then optionally auto-pop), **discard and switch** (with the discard safety net), or cancel. Never just refuse.
- Handle the name-collision case: checking out `origin/foo` when a local `foo` exists must work. GitHub Desktop r47 (55 comments) is exactly this failing.
- Update the UI to the new branch state without a full graph rebuild (`docs/01` §3.3).
- If the repo has submodules, do not hang waiting on submodule updates (GH Desktop r7, 73 comments).

### Merge conflict
- Detect on entry *and* on app restart — a repo can already be mid-merge when opened. Read `.git/MERGE_HEAD`, `rebase-merge/`, `CHERRY_PICK_HEAD`.
- Show a persistent banner: which operation, which branches, how many files remain conflicted.
- Always offer **abort**. This is the single most important affordance; without it a user who starts a merge in git-plum can only escape via the CLI.
- List conflicted files, distinguishing content conflicts from add/add, delete/modify, and binary conflicts. **Never auto-pick a side for binary files** — SourceGit shipped that bug ("always selecting OURS for binary files and not prompting").
- Provide in-app two-way resolution with accept ours / theirs / both per block, plus an "open in external tool" escape hatch.
- Marking a file resolved must `git add` it and update the remaining count. Do not let unstaging a file silently exit the merging state (SourceGit bug #1373).
- After all files are resolved, offer **continue** with a prefilled merge message.

### Discard
- Name the exact files. For untracked files say **Delete** (they will be gone), not "Discard" — the Atom/GitHub lesson.
- Write a recoverable snapshot first (auto-stash-before-discard), and expose a "Recently discarded" restore list.
- Never make discard the default button in a dialog, and never bind it to a bare key.

### Open a repository
- Detect and recover from mid-operation states (merge / rebase / cherry-pick / bisect) on open.
- Degrade gracefully on: detached HEAD, empty repo with no commits, bare repo, shallow clone, repo on a network drive, and repos with submodules that can't be reached.
- Never block the UI thread on a git process (`docs/01` §3.1).

---

## Competitor Feature Analysis (deltas only; see `docs/01` §1 for the market table)

| Feature | GitKraken | Fork | SourceGit | GitHub Desktop | git-plum (recommended) |
|---|---|---|---|---|---|
| Conflict resolution | 3-pane + live preview, strongest in category | Good built-in; users miss GK's | **Two-way, shipped Jan 2026 after reversing a refusal** | Basic | **Two-way, in v1** (learn from #892) |
| Blame | In the diff toolbar | Yes | Yes (HEAD version only) | **No — 4 separate requests, top at r42** | Read-only gutter, late v1 |
| Interactive rebase | Drag-and-drop | Yes | Yes (long bug tail) | No | **Defer the editor; ship squash/fixup/reword + reset in v1** |
| Embedded terminal | Yes (toolbar) | No | **No — and ~20 issues all want the *external* launcher** | No | **Never. Ship configurable "Open in Terminal".** |
| Undo/redo | Yes, incl. Discard; the most-praised safety feature | Limited | No | No | **Discard safety net in v1; general undo v1.x** |
| Command palette | No | No | Requested (r6), not shipped | No | **Yes, v1 — open competitive gap** |
| Show git commands | Opaque (a named criticism) | No | Yes (command logs) | No | **Yes, v1 — near-free, high trust** |
| AI commit messages | Yes ("Recompose with AI") | No | Yes (OpenAI-compatible) | No | Yes, opt-in + local models, scheduled last |
| Search | Yes | Yes | Yes (27 refinement issues) | **Weak — r96, r20 open** | **Yes, v1** |
| Linux | Yes | **No** | Yes | **No — r4851, highest-reaction issue found** | **Yes — structural advantage over Fork** |
| Open source | No | No | Yes (**maintainer seeking successor**) | Yes | Yes — the opening is real and currently widening |
| Performance at scale | Poor (Electron, GB of RAM) | Excellent | Good | Poor | **The wedge — benchmark in phase 1, not phase 6** |

---

## Confidence and Gaps

| Claim | Confidence | Basis |
|---|---|---|
| Conflict resolution is a hidden table stake | **HIGH** | #892 → #2070 is a controlled natural experiment on the identical argument, with a documented reversal |
| Embedded terminal is correctly cut | **HIGH** | ~20 SourceGit issues, 100% about external launch, 0% embedded |
| Discard needs a safety net in v1 | **MEDIUM-HIGH** | GitKraken testimonial + Tower + Atom #1001 + the structural fact that discard has no reflog entry |
| Hunk staging is under-estimated | **HIGH** | GitLane #425 enumerates concrete shipped bugs; SRCTREE-3047 corroborates |
| Submodules are a trap | **HIGH** | 82 GH Desktop issues incl. a tracking issue and a 73-comment thread |
| Blame is borderline-table-stakes | **MEDIUM** | GH Desktop r42 + 13 SourceGit refinement bugs; inference from bug *character*, not a direct survey |
| AI is not a switching reason | **MEDIUM** | Commentary and one open SourceGit issue; no quantitative data |
| Command palette would differentiate | **MEDIUM** | Sublime Merge praise + SourceGit r6 and two shortcut-customization requests; plausible but unproven for this audience |
| Interactive rebase is safe to defer | **HIGH** | 22 SourceGit issues, overwhelmingly bugs; GH Desktop lacks it without top-tier complaint |
| Search is a missing table stake | **MEDIUM-HIGH** | GH Desktop r96 + r20 + r39; 27 SourceGit issues |

**Gaps this research could not close:**
- **No quantitative data on abandonment causes.** Everything here infers demand from issue reactions and forum sentiment. Reaction counts measure the vocal minority, not the median user.
- **Fork has no public issue tracker**, so the closest-positioned competitor contributes no direct user-complaint data. Its strengths are inferred from reviews.
- **No usage telemetry from any client** (and git-plum won't have any, by design). Frequency-of-use claims for blame and hunk staging are inferences from the character of bug reports.
- **The two-way vs three-pane conflict UX claim** rests on one strong essay (eseth.org), Sublime Merge's own 3-way usability issues, and SourceGit's shipped choice. It is a defensible design position, not a settled fact. Worth validating with early users.
- **Actual effort for the two-way conflict resolver in *our* stack** is unverified. SourceGit's merged in four hours of review, but that is a mature Avalonia codebase with an existing text editor component. Our CodeMirror 6 path should be comparable but needs a phase-level spike.
- **`docs/01` §8's 11–14 week MVP estimate** does not include the promotions recommended here, and already under-budgets hunk staging. Expect that estimate to move.

---

## Sources

**Issue trackers (primary evidence):**
- https://github.com/sourcegit-scm/sourcegit/issues — 1692 issues, queried by reactions and by keyword (conflict, blame, interactive rebase, terminal, search)
- https://github.com/sourcegit-scm/sourcegit/issues/892 — "[feature suggestion] Integrated merge-tool" (closed, r10, full comment thread quoted)
- https://github.com/sourcegit-scm/sourcegit/pull/2070 — "Built-in merge conflict solver" (merged 2026-01-27)
- https://github.com/sourcegit-scm/sourcegit/issues/2168 — "Allow editing text in Merge Conflicts Window" (open, r10)
- https://github.com/desktop/desktop/issues — 1004 open issues, queried by reactions and keyword (blame, submodule)
- https://github.com/Siomkin/GitLane/pull/425 — "Harden the git CLI write layer against the user's environment"
- https://jira.atlassian.com/browse/SRCTREE-3047 — `git apply` whitespace-revert failure
- https://github.com/atom/github/issues/1001 — discard-induced data loss
- https://github.com/sublimehq/sublime_merge/issues/1079, /1869 — 3-way merge UI usability

**Documentation:**
- https://github.com/sourcegit-scm/sourcegit — README feature list (the free/open-source baseline to clear)
- https://www.sublimemerge.com/docs/command_palette , /key_bindings
- https://help.gitkraken.com/gitkraken-desktop/undo-and-redo/ — supported undo actions incl. Discard
- https://gitkraken.com/features/merge-conflict-resolution-tool

**Sentiment and analysis:**
- https://www.eseth.org/2020/mergetools.html — the case against three-way merge tools; `hideResolved` (git ≥2.31)
- https://danmackinlay.name/notebook/git_guis.html — "hiding the details of the git command-line"; Sourcetree criticism
- https://jonathansblog.co.uk/best-git-ui-clients-2026 — daily-use feature list; abandonment causes
- https://community.atlassian.com/t5/Sourcetree-discussions/Sourcetree-4-0-Painfully-Slow-and-Buggy/td-p/1222526 and related threads
- https://devrant.com/rants/1872055/ — forced-account abandonment
- https://movingfulcrum.com/sourcetree-atlassians-most-epic-engineering-fail/
- https://biggo.com/news/202510280131_AI-commit-tools-debate — AI commit message sentiment, both sides
- https://dev.to/harsh2644/i-asked-ai-to-write-my-commit-messages-it-was-embarrassing-a6i
- https://dev.to/_d7eb1c1703182e3ce1782/best-git-gui-clients-in-2025-gitkraken-sourcetree-fork-and-more-compared-4gjd
- https://www.makerstack.co/reviews/sublimemerge-review/

**Internal:**
- `.planning/PROJECT.md` — MVP scope and out-of-scope list under audit
- `docs/01-research-competitors.md` — market comparison, GitKraken screenshot analysis, lane algorithm, effort estimate

---
*Feature research for: cross-platform open-source Git GUI desktop client*
*Researched: 2026-09-21*
