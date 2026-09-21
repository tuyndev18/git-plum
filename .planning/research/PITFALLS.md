# Pitfalls Research

**Domain:** Git GUI desktop client — Tauri v2 (Rust) wrapping `git` CLI + React/TypeScript frontend
**Researched:** 2026-09-21
**Confidence:** MEDIUM-HIGH (git CLI behavior and Tauri v2 breaking changes are HIGH confidence, verified against official docs/issues; commit-graph algorithm edge cases and "why hobby Git GUIs die" are MEDIUM — pattern-matched from community reports, not a single authoritative source)

This document is organized by the 8 questions in the research brief. Each pitfall maps to one
of the project's 6 requirement groups (Nền tảng/Platform, Đọc lịch sử/History, Xem khác biệt/Diff,
Thay đổi repository/Working-tree, Nhánh và remote/Branch-remote, AI) so it plugs directly into
roadmap phase design.

---

## Critical Pitfalls

### Pitfall 1: Locale-dependent git output breaks parsers on non-English systems

**What goes wrong:**
`docs/01-research-competitors.md` §5 specifies `git log --format=...%x1f...` and
`git status --porcelain=v2` but does **not** set `LC_ALL=C` or `LANG=C` on the spawned process.
Git's porcelain formats are stable in *structure*, but several pieces of output are still
locale-sensitive: date formats if `--date` is not pinned to `iso-strict`/`unix`, warning/error
text on stderr (which the app will want to surface to the user), and — critically — some
git builds emit translated branch-state text even in porcelain paths (e.g. "typo in ahead/behind
phrasing", `interactive.singlekey` prompts). A developer's own Windows machine is usually
`en-US`, so this passes every manual test and only breaks on a contributor's or user's
non-English-locale machine (common in EU/Asia), surfacing as silent parse failures or garbled
error dialogs weeks after release.

**Why it happens:**
Developers test on their own machine, which has the locale they wrote the parser against.
`--porcelain=v2` guarantees the *data* fields are stable, but git still localizes anything
that isn't explicitly part of the machine format (advice text, hints, stderr messages,
`%cd`/`%ad` when not given an explicit `--date=` mode).

**How to avoid:**
- Force `LC_ALL=C` (or `LC_ALL=C.UTF-8` if available, falls back to `C`) and `LANG=C` as
  environment variables on **every** spawned git process, not just the ones with custom format
  strings. This is cheap insurance with no downside since the app never displays raw git stderr
  verbatim to end users anyway (it should translate/wrap errors).
  Verification: this directly extends the decision already made in §5 of the research doc —
  the format strings are correct, but the missing piece is the environment the process runs in.
- Always use explicit `--date=unix` or `--date=iso-strict` in `git log` format strings, never
  rely on default date formatting (confirmed correct already in the project's likely
  implementation, but call it out — it's the same locale trap in a different spot).
- Never parse stderr for control flow (see Pitfall 2) — treat stderr purely as diagnostic text
  for a generic error toast, since its wording is not a stable contract even with `LC_ALL=C`.

**Warning signs:** Parser works in dev, bug reports mention garbled dates or "unknown state"
errors from users outside the US/UK; CI tests only run in one locale.

**Phase to address:** Platform (Nền tảng) — the git-process-spawning wrapper is foundational
infrastructure built in Phase 1. Every later phase (history, diff, working-tree, branch/remote)
calls through this wrapper, so the env vars must be baked in once, centrally, not per-call-site.

---

### Pitfall 2: Git process hangs forever on credential prompts (classic trap)

**What goes wrong:**
Any command that talks to a remote (`fetch`, `pull`, `push`, `ls-remote` for ahead/behind) can,
if no cached credential exists, try to prompt interactively. When spawned from a GUI app with
no attached TTY and stdin not explicitly closed, git's default behavior on some platforms/configs
is to block indefinitely waiting for input that will never come — the process just hangs. The UI
shows an infinite spinner. This is one of the most commonly hit "worked in every demo, froze on
a real user's clone" bugs in git-wrapping tools (confirmed as a recurring bug pattern in
multiple non-interactive git-wrapper projects, e.g. nanocoder issue #1344: "execProcess in git
tools lacks stdin closure, credentials timeout, and GIT_TERMINAL_PROMPT=0").

**Why it happens:**
Git assumes a terminal is available unless told otherwise. GUI apps spawn subprocesses with
stdin as a pipe (not a TTY), and some credential helpers / SSH clients still attempt to read
from it. Developers test this only against repos where credentials are already cached in
Windows Credential Manager or an SSH agent, so the hang never triggers locally — it appears the
first time a real user clones a repo with expired/missing credentials.

**How to avoid:**
- Set `GIT_TERMINAL_PROMPT=0` on every spawned git process (forces git to fail fast with a
  clear error instead of prompting on a non-existent terminal).
- Explicitly close/redirect stdin on every spawned child process (do not inherit the parent's
  stdin — Tauri apps have no console stdin to inherit anyway, but be explicit rather than
  relying on default `Stdio` behavior in Rust's `std::process::Command`).
- For SSH-based remotes, also consider setting `GIT_SSH_COMMAND` with
  `-o BatchMode=yes` to make SSH fail immediately instead of prompting for a passphrase.
- Add a hard timeout (e.g. 30-60s) on network-touching git operations as a last-resort
  circuit breaker, surfaced as "operation timed out — check your credentials" in the UI.
- Let the OS credential helper (Windows Credential Manager / GCM) do its job for the *normal*
  case — this is correct per the project's CLI-over-libgit2 decision — but never let the
  fallback path (no cached credential, no GCM configured) hang the UI thread.

**Warning signs:** Any manual QA step involving `push`/`pull`/`fetch` against a *fresh* clone
with no cached credentials; a spinner that never resolves; support reports of "app frozen."

**Phase to address:** Platform (Nền tảng), specifically the process-spawning layer — but the
symptom is only exercised in Branch/Remote phase testing (fetch/pull/push). Recommend: bake the
env vars + timeout into the spawn wrapper in Phase 1, but write an explicit test case
("fetch against repo with no cached credentials must fail within N seconds, not hang") as an
acceptance criterion of the Branch/Remote phase, since that's the first phase that actually
exercises network commands.

---

### Pitfall 3: Non-UTF-8 filenames and commit messages corrupt JSON over IPC

**What goes wrong:**
Git does not enforce UTF-8 for filenames or commit messages. Older Windows repos, repos
touched by legacy tools, or filenames created on Latin-1/Shift-JIS systems can contain
byte sequences that are not valid UTF-8. When Rust reads process stdout as bytes and tries to
convert to a `String` (Rust strings are UTF-8-only), this either panics, silently uses lossy
replacement (`\u{FFFD}`), or — if using `String::from_utf8` unchecked — corrupts the JSON
payload sent to the frontend over IPC, causing the whole response to fail to deserialize
(one bad commit or filename breaks the entire page of history, not just that row).

**Why it happens:**
The project's own filenames and commits are all clean UTF-8, so this never surfaces in the
developer's own test repos. It only appears when a real user opens an older repo, a repo with
mixed contributor tooling history, or one with filenames from a non-UTF-8 locale. `git log`
does re-encode commit message *content* to UTF-8 by default (via the `encoding` header) unless
told otherwise, but **filenames are not re-encoded** — they are raw bytes from the filesystem/tree
object, and `core.quotepath` will octal-escape non-ASCII bytes in human-readable output unless
disabled or `-z`/NUL-terminated output is used.

**How to avoid:**
- Use `-z` (NUL-terminated, unescaped) output wherever git supports it for anything involving
  filenames: `git status --porcelain=v2 -z` (already chosen correctly in §5.3),
  and also apply `-z` to any `git diff --name-status`, `git log --name-status`, or similar
  filename-bearing commands added later. This avoids `core.quotepath` octal-escaping entirely.
- Parse stdout as raw bytes (`Vec<u8>`) in Rust, then convert to `String` with
  `String::from_utf8_lossy()` at parse boundaries rather than `String::from_utf8()` — never let
  one bad byte sequence panic or fail an entire batch. Replacement characters are an acceptable
  degraded UX; a crashed history pane is not.
  Do this deliberately and document it, since lossy conversion is a real (if rare) display
  fidelity tradeoff, not a free win.
  Also explicitly set `-c core.quotepath=false` on log/status calls even when using `-z`, since
  it affects some auxiliary fields.
- For commit messages, git already handles re-encoding via `i18n.commitEncoding` /
  `i18n.logOutputEncoding` — trust it, but still lossy-convert at the Rust boundary as defense
  in depth (a corrupt encoding header should degrade, not crash).

**Warning signs:** Testing exclusively against the project's own repo (small, clean, all-ASCII
filenames); no test repo with intentionally non-ASCII/non-UTF-8 filenames in the seed/fixture
set; IPC deserialization errors that only reproduce on specific external repos.

**Phase to address:** History reading (Đọc lịch sử) — this is where filenames and commit
messages first flow through the parser into the UI. Add a fixture repo with non-UTF-8 filenames
(e.g. Latin-1-encoded filename, emoji in commit message, embedded NUL-adjacent bytes) to the
Phase 1/2 test suite before building the diff and staging phases on top of the same parsing
primitives.

---

### Pitfall 4: `git apply --cached` race between reading status and applying the patch

**What goes wrong:**
The hunk-staging flow described in §5.5 is: (1) get full diff of a file, (2) user picks hunks in
the UI, (3) reconstruct a patch from just those hunks, (4) `git apply --cached -` via stdin.
Between step 1 (diff read) and step 4 (patch applied), the working tree or index can change —
the user edits the file in their editor with autosave, another git operation runs, or a file
watcher's own debounce delivers a stale diff. The reconstructed patch's context lines then no
longer match the current index/working-tree state, and `git apply` fails with
"patch does not apply" or, worse, silently applies to the wrong context if `--unidiff-zero` /
fuzzy matching is used carelessly. Related real-world bug: Sourcetree's own bug tracker has a
recorded "Error encountered when clicking Stage hunk button" issue with this exact class of
failure, and Magit has a long-standing issue where non-ASCII content in a hunk causes
"patch does not apply" (magit#3172) — showing this is a recurring cross-tool failure mode, not
a one-off.

**Why it happens:**
The UI necessarily has some latency between showing a diff and the user clicking "stage this
hunk" — the diff is a snapshot, not a live view. Developers build and test the happy path
(open diff, immediately stage) and never simulate the file changing mid-interaction.

**How to avoid:**
- Re-fetch the file's diff immediately before constructing and applying the patch (or at
  minimum, verify the diff's blob hash / mtime is unchanged since it was fetched) — treat the
  staging action as "diff, then immediately re-verify, then apply" rather than trusting a
  UI-cached diff that may be seconds or minutes old.
- On `git apply` failure, do **not** silently retry with fuzzy matching (`-C` context reduction
  or `--recount`) — surface a clear "file changed since you viewed this diff, please refresh"
  error and force the user to re-open the diff. Silent fuzzy application risks staging the wrong
  lines, which is far worse than a visible failure for a tool whose core value is trustworthy
  history.
- Never apply with `--whitespace=fix` or other "helpful" normalization flags by default for
  hunk staging — they can shift line matching in ways the user didn't ask for.
- Use `git apply --check --cached` first (dry run) to detect the mismatch cheaply before
  attempting the real apply, so the error path is fast and doesn't touch the index at all.

**Warning signs:** Manual QA only tests "click stage hunk immediately after opening diff";
no test where the file is modified (even just touched/saved) between diff-open and stage-click;
no test with an autosaving editor open on the same file during a staging session.

**Phase to address:** Working-tree changes (Thay đổi repository) — this is exactly the phase
that implements hunk staging. Add "stale diff / file changed mid-stage" as an explicit test
case, not just "happy path stage/unstage."

---

### Pitfall 5: Windows console window flashes on every git subprocess spawn

**What goes wrong:**
On Windows, spawning a console subprocess (`git.exe`) from a GUI application causes a visible
black console window to flash open and immediately close, once per git invocation — and this
app will spawn git dozens of times per user session (every status refresh, every log page,
every diff). This is a well-documented Tauri-on-Windows issue (tauri-apps/tauri discussion
#11446, "Terminal Window Briefly Opens and Closes During Tauri App Runtime in Production on
Windows") and immediately reads as "buggy, unpolished software" to users — directly undermining
the "fast and polished, GitKraken-quality" positioning from PROJECT.md.

**Why it happens:**
Windows' `CreateProcess` by default allocates a new console window for a child console
application unless told not to. Rust's `std::process::Command` does not suppress this by
default; neither does the Tauri v2 shell plugin unless explicitly configured. It's invisible
in `cargo run` dev builds launched from an existing terminal (the child inherits the parent
console, so nothing new flashes) — which is precisely why it's easy to miss until testing a
bundled release build launched by double-clicking the icon, i.e. late.

**How to avoid:**
- Set the Windows-specific creation flag `CREATE_NO_WINDOW` (`0x08000000`) via
  `std::os::windows::process::CommandExt::creation_flags(0x08000000)` on every spawned git
  process, Windows-only (behind `#[cfg(windows)]`).
- If using the Tauri shell plugin instead of raw `std::process::Command`, verify it exposes
  equivalent flag control for this Tauri v2 version — if not, prefer raw
  `std::process::Command` for git invocations, since the shell plugin is designed more for
  user-facing "open this file in an external app" scenarios than for a hot-path internal
  process-spawning wrapper.
- Test this specifically in a **bundled release build** (MSI-installed or at least
  `cargo tauri build`), not `tauri dev` — the flashing is invisible in the dev console-attached
  scenario, which is exactly how this bug hides until late.

**Warning signs:** Everything looks fine in `npm run tauri dev`; QA only happens via dev mode;
no one has run a release-mode `.exe` double-clicked from Explorer until late in the project.

**Phase to address:** Platform (Nền tảng) — same spawning wrapper as Pitfalls 1 and 2. This is
cheap to fix once, expensive to retrofit across every call site if discovered late. Add
"launch the bundled .exe from Explorer, not dev mode, and watch for console flash" as an
explicit manual QA step at the end of Phase 1.

---

### Pitfall 6: Tauri v2 capability/permission misconfiguration silently blocks commands

**What goes wrong:**
Tauri v2 replaced v1's allowlist with a capability-based ACL system: the webview is untrusted
by default, and every IPC command plus every plugin (fs, dialog, shell, os) requires an explicit
permission grant in a capability file. A command that isn't granted doesn't error loudly in an
obvious way in all cases — depending on how the invoke wrapper is written, missing permissions
can manifest as a generic "command not allowed" rejection that's easy to misdiagnose as a
frontend bug, especially for junior/web-focused contributors unfamiliar with Rust/Tauri's ACL
model (PROJECT.md itself flags "Rust là rào cản với nhóm quen web" as a known risk). A very
common specific trip: the folder-picker dialog (`Mở repository từ hộp thoại chọn thư mục`)
requires the `dialog:allow-open` permission with an explicit scope, and file system access for
reading `.git` config or opening an external editor requires `fs` scope entries — both are easy
to forget since they're additive and not implied by adding the plugin to `Cargo.toml`.

**Why it happens:**
Adding a plugin dependency in `Cargo.toml` doesn't grant it access — that's a deliberate v2
security design, but it's an easy step to skip because the code compiles fine and only fails
at runtime when the command is actually invoked. It's also easy to grant permissions broadly
during early development (e.g. wildcard scopes) to "get it working," and never tighten them
before release — leaving an over-permissioned app that's a worse security story than it needs
to be for an open-source tool people are asked to trust with their credentials.

**How to avoid:**
- Treat capability files as part of the definition-of-done for each feature that touches a new
  Tauri API surface (dialog, fs, shell, os, clipboard) — not an afterthought.
  Grant the narrowest scope that works (e.g. specific path patterns, not `$HOME/**`).
- When a command silently fails, check the capability file / permission grant *first* before
  assuming a frontend or IPC bug — document this as a known first-check in project dev docs to
  avoid wasted debugging time, given the team's admitted unfamiliarity with the Rust/Tauri side.
- Audit capability scopes before each release milestone, specifically before the "Phát hành"
  (release/packaging) phase, since dev-time convenience scopes (e.g. broad fs read access for
  testing) are exactly the kind of thing that gets left in and shipped if not explicitly
  reviewed.

**Warning signs:** A command works in one dev session but fails after adding a new plugin or
after a clean capability-file edit; "command not allowed" style errors; broad wildcard scopes
still present close to release.

**Phase to address:** Platform (Nền tảng) for the initial capability setup pattern — establish
the discipline (narrow scopes, review checklist) in Phase 1 when the dialog/fs plugins are first
wired up, since every later phase adds new IPC commands that need equivalent scrutiny.

---

### Pitfall 7: Naive lane-assignment algorithm breaks on octopus merges and multi-root histories

**What goes wrong:**
The lane algorithm in §4.2 handles the common cases (linear history, two-parent merges,
branch/fork points) correctly, but the pseudocode's step 3 ("Cha thứ hai trở đi (merge commit)
-> cấp lane trống mới") doesn't explicitly address **octopus merges** (3+ parents in one commit,
legal in git via `git merge branch1 branch2 branch3`) where *each additional parent beyond the
first* needs its own lane allocation and its own diagonal edge — a naive implementation that
only special-cases "parent 2" and treats the rest like parent 1 will draw a visually broken or
overlapping graph. Similarly unaddressed: **multiple root commits** (a repo with no single
common ancestor — common after a history rewrite, an `--orphan` branch used for a `gh-pages` or
docs branch, or two histories merged with `--allow-unrelated-histories`), where the "no parent"
case needs to free the lane but the algorithm must also handle *starting* a fresh lane for an
unrelated commit stream appearing later in `--topo-order` output. Real precedent: even git's own
`commit-graph` machinery has shipped bugs specifically in octopus-merge parent-offset handling
(microsoft/git PR #167), showing this is a genuinely easy class of bug to get subtly wrong even
in mature codebases.

**Why it happens:**
Octopus merges and orphan branches are rare in the developer's own day-to-day repos (most
projects merge two branches at a time), so hand-written test cases default to 2-parent merges.
The algorithm looks complete because it passes on every "normal" repo tested during
development, and the bug only surfaces on a specific real-world repo shape encountered by an
actual user months later — exactly the "discovered in week 10" failure mode this research is
meant to prevent.

**How to avoid:**
- Explicitly test the lane algorithm against: (a) a synthetic repo with a 4-parent octopus merge,
  (b) a repo with two unrelated root commits merged via `--allow-unrelated-histories`,
  (c) an `--orphan` branch with no shared history with `main`, (d) a very wide "many concurrent
  feature branches merging into main" shape (20+ simultaneous active lanes) to check the color
  palette cycling and lane-reuse logic under pressure, not just correctness.
  These are cheap to generate as fixture repos with a shell script and should be part of the
  automated test suite, not just manual QA.
- Explicitly generalize step 3 of the pseudocode: "for each parent after the first, in order,
  allocate a new lane and draw a diagonal edge" (loop, not a fixed 2-parent special case).
- Decide and document a deliberate degraded rendering for pathological cases (e.g. an octopus
  merge with 15 parents, which is legal but absurd) — e.g. cap visible lanes and collapse extras
  into a "+N more parents" indicator — rather than letting the graph render arbitrarily wide and
  break the "opens in under a second" performance goal.
- Detached HEAD and shallow clones (`--depth=N`) don't break lane assignment directly, but
  shallow clones produce **grafted** commits (parents that reference objects not present locally)
  — the algorithm needs to treat a missing/unfetchable parent as a terminal node (like a root
  commit) rather than crashing or hanging on a lookup for a commit that doesn't exist in the
  loaded set.

**Warning signs:** Test fixture repos are all authored by one person with simple merge/rebase
flows; no fixture repo has 3+ way merges or unrelated history; graph rendering code has a
hardcoded assumption of exactly 2 parents (e.g. `parent[0]`/`parent[1]` named variables instead
of a loop over `parents`).

**Phase to address:** History reading (Đọc lịch sử) — this is precisely the lane-algorithm
implementation phase. Build the octopus-merge and orphan-branch fixture repos *before* writing
the algorithm (test-first for this specific component is worth the extra time, since the
algorithm is the Core Value of the entire product per PROJECT.md).

---

### Pitfall 8: File-watching `.git` for external-change detection causes event storms and lock contention

**What goes wrong:**
To detect when the user runs `git` from an external terminal while the GUI is open (question 6),
the natural approach is to watch the `.git` directory (and working tree) with a filesystem
watcher (e.g. Rust's `notify` crate). Two failure modes commonly appear: (1) **event storms** —
watching the whole working tree recursively (not just `.git`) picks up every build artifact,
`node_modules` change, or editor temp file, firing far more refresh events than needed, causing
UI jank or excessive redundant `git status` calls; (2) **lock contention / self-triggering
loops** — the app's own git operations (status, stage, commit) also touch `.git/index`,
`.git/HEAD`, and `.git/refs/*`, so a naive watcher reacts to the *app's own writes*, potentially
triggering a redundant status refresh mid-operation, or worse, racing with an in-flight
`.git/index.lock` held by a concurrent operation and surfacing a confusing "unable to create
'.git/index.lock': File exists" error to the user when two operations (e.g. an autorefresh
triggered by the watcher and a user-initiated stage) overlap.

**Why it happens:**
Watching "the repo" sounds simple, but `.git` internals are noisy (packed-refs rewrites, loose
object writes, reflog updates) and the working tree is even noisier. Developers who test with
small, quiet repos and single-user, single-action-at-a-time usage never trigger the storm or the
lock race; it appears under real usage with build tooling running, an editor autosaving, or the
user issuing rapid-fire actions.

**How to avoid:**
- Watch narrowly and specifically: `.git/HEAD`, `.git/index`, `.git/refs/**`,
  `.git/packed-refs`, and separately the working tree filtered through the same ignore rules
  git itself uses (respect `.gitignore` — don't watch ignored paths at all). Do not recursively
  watch the entire working tree unfiltered.
- Debounce aggressively (250-500ms is the commonly cited window in comparable tools) and coalesce
  multiple events into a single refresh — never fire one `git status` per raw filesystem event.
- Treat `.git/index.lock` existing as an expected, recoverable transient state, not an error:
  retry with backoff before surfacing anything to the user, since another legitimate git process
  (including the app's own concurrent operation) may simply be mid-write.
  Never auto-delete `index.lock` — that's a data-loss-risk action a GUI tool should never take
  silently (unlike a human debugging via command line who can judge whether it's stale).
  Also handle `.git/index.lock` errors returned directly *from the app's own spawned git
  processes* the same way, since two features triggered close together (e.g. auto-refresh timer
  and user-initiated stage) are the app racing itself, not just external interference.
- Serialize the app's own git-mutating operations through a single queue/mutex in the Rust
  backend so the app never races itself, independent of what external tools do. This is the
  actual root fix for the self-triggering case — file watching only needs to handle genuinely
  *external* changes, not the app's own writes, if internal operations are already serialized.

**Warning signs:** No test scenario has an external terminal (or a second app window) running
git commands against the same repo while the GUI is open; rapid clicking of stage/unstage in
the UI isn't tested; watcher implementation recursively watches the full working tree with no
`.gitignore`-aware filtering.

**Phase to address:** Working-tree changes (Thay đổi repository) is where staging/status live
and where the first real need for live refresh appears; but the *serialization of the app's own
git operations* should be established as an architectural pattern in the Platform (Nền tảng)
phase, since it's foundational to avoiding self-races in every later phase, not just working-tree.

---

### Pitfall 9: AI diff integration leaks secrets and blows token limits silently

**What goes wrong:**
The AI commit-message feature sends staged diff content to an external API (Claude/OpenAI) or
local Ollama. Two distinct failure modes: (1) a staged diff can contain `.env` file contents,
API keys, private key files, or other secrets that the user is about to commit — sending that
diff to a third-party API before the user has even committed it is a meaningfully worse privacy
exposure than the commit itself (which at least stays local until pushed), yet nothing in
PROJECT.md's requirements currently calls out secret-scanning before the AI call; (2) large
diffs (a big refactor, a generated-file change, a lockfile update) can exceed the provider's
context/token limit, and a naive implementation either truncates silently (producing a
commit message based on a fraction of the actual change — actively misleading) or the API call
fails with an opaque error that looks like a bug rather than an expected limit.

**Why it happens:**
The feature is built and demoed against small, clean, intentional diffs during development.
Secret-bearing diffs and huge diffs are both edge cases that don't show up in a developer's own
"add a function, stage it, generate a message" test loop, but are common in real usage
(committing a config file, a large dependency bump, a big rename).

**How to avoid:**
- Run a lightweight secret-pattern scan (common patterns: AWS keys, private key PEM headers,
  generic `API_KEY=`/`SECRET=` assignments, high-entropy strings in `.env`-like filenames)
  over the diff **before** sending to any external provider, and block/warn rather than silently
  send. This should apply even for cloud-provider calls; local Ollama is lower-risk but the
  scan is cheap enough to run universally.
- Respect `.gitignore`/`.env` conventions as a first line of defense, but don't rely on them
  alone — many real leaks happen precisely because a secret ended up in a *tracked* file.
- Measure diff size (character/token count, roughly) before sending; if it exceeds a threshold,
  either summarize per-file instead of sending the raw diff, or explicitly tell the user
  "diff too large for AI summary, showing file list only" rather than silently truncating and
  presenting a message as if it reflects the whole change.
- Make the token-limit behavior explicit and visible in the UI (e.g. "AI summary based on N of
  M changed files" if truncation is unavoidable) — never let a partial-context AI output look
  authoritative.
- Confirm rate-limit and API-error handling produce a clear, actionable message
  ("rate limited, try again in a moment" vs. a generic failure) — this is standard API
  hygiene but easy to skip since the happy path works fine in a demo.

**Warning signs:** No test diff contains an `.env`-like file or key-shaped string; no test diff
exceeds a few hundred lines; error handling for the AI call only covers "network unreachable,"
not "provider returned 429" or "context length exceeded."

**Phase to address:** AI — this is entirely within the AI integration phase's scope, but the
secret-scan should be treated as a hard requirement of that phase's definition-of-done, not an
optional polish item, given PROJECT.md's own stated privacy/security constraints
("Nội dung diff có thể chứa mã nguồn nhạy cảm... Không gửi gì ra ngoài khi chưa được cho phép rõ ràng").

---

### Pitfall 10: Hobby Git GUI projects stall on infrastructure polish, never reach a usable core loop

**What goes wrong:**
This is a well-documented pattern in "build my own Git GUI" side projects (and PROJECT.md's own
§7 risk table names scope creep as the single highest risk). The specific failure shape:
weeks are spent perfecting the three-pane layout, theming, window chrome, and general Tauri/Rust
IPC plumbing — all visible, satisfying, demo-able work — while the actual hard, unglamorous
core loop (staging correctness, diff accuracy on real-world edge-case files, graph correctness on
real-world branchy repos) stays underbuilt. The project *looks* far along (a polished shell) but
isn't *usable* for a single real day-to-day workflow, so it never gets dogfooded, so its rough
edges never get discovered, so it stalls indefinitely without ever reaching the "I use this
instead of my old tool" threshold that would sustain motivation.

**Why it happens:**
UI/layout work has fast, satisfying feedback loops (visible progress every commit) while
correctness work on git edge cases has slow, unglamorous feedback loops (you only find out it's
wrong when a weird repo breaks it). Solo/small-team side projects naturally gravitate toward the
former. There's also no external forcing function (no paying customer, no team depending on it)
to prioritize correctness over polish.

**How to avoid:**
- PROJECT.md already states the right instinct ("Bản chỉ đọc dùng được sau khoảng 5–6 tuần" —
  a read-only build should be usable early). Make this literal and enforced: the read-only
  history+diff milestone (end of the "Đọc lịch sử" + "Xem khác biệt" phases) should be dogfooded
  daily by the author on their *own* real repos (including this project's own repo, and at least
  one large/messy third-party repo) before any working-tree/staging work begins — not as an
  optional nice-to-have, but as a phase-exit gate.
- Resist adding UI polish (themes, animations, settings panels) until the three-verb core loop
  (view history -> view diff -> stage & commit) works end-to-end on the author's real daily repo.
  GitKraken/Fork-level visual polish is explicitly a non-goal until the core loop is trustworthy.
- Track "have I committed a real change to a real repo using only git-plum today" as a literal
  personal milestone once the working-tree phase lands — the moment this becomes uncomfortable
  or the tool gets abandoned in favor of the terminal for a real task, that's the signal for
  what to fix next, not what feature to add next.
- Watch for the specific warning sign called out in PROJECT.md's own risk table: time spent on
  anything from the "Out of Scope" list (undo/redo, merge conflict UI, integrated terminal,
  interactive rebase drag-drop) before the MVP feature list is fully solid — these are exactly
  the kind of visually appealing, scope-creep-inviting features that kill similar projects.

**Warning signs:** Multiple consecutive weeks of commits touching only CSS/layout/theming with
no commits touching `src-tauri/src/git/` parsing or staging logic; the project's own repo has
never actually been committed-to using the in-progress tool; features from "Out of Scope" start
reappearing in discussion "just as a quick add."

**Phase to address:** This is cross-cutting, not a single phase — but it should be made
*procedural*: define an explicit "dogfood gate" between the Diff-viewing phase and the
Working-tree phase (must use the tool daily on a real repo before proceeding), and another
between Working-tree and Branch/Remote (must complete a real commit + push cycle on a real repo
using only the tool). This converts an abstract "don't scope-creep" risk into a concrete,
checkable phase-exit criterion.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|-----------------|------------------|
| Parse `git log` output by splitting on newlines instead of `%x1f`/`%x1e` | Slightly simpler code | Breaks on multi-line commit bodies, silently misaligns fields | Never — §5.1's separator choice is correct, don't regress it |
| Skip `LC_ALL=C`/`GIT_TERMINAL_PROMPT=0` env vars "since it works on my machine" | Saves 10 minutes now | Locale-dependent parse bugs + credential-prompt hangs discovered by real users, not devs | Never — near-zero cost to add upfront in Phase 1 |
| Use `String::from_utf8().unwrap()` instead of lossy conversion at IPC boundaries | Simpler error-free-looking code | Panics/crashes on any non-UTF-8 filename or commit message from a real-world repo | Never in the parsing layer; acceptable only in throwaway prototype scripts |
| Recursive unfiltered filesystem watch on the whole working tree | Fastest to wire up | Event storms, high CPU on large repos, self-triggered refresh loops | Acceptable only for a very first internal prototype, must be narrowed before Working-tree phase ships |
| Retry `git apply` with fuzzy/context-reduced matching on failure | Fewer visible "stage failed" errors in demos | Risk of applying a hunk against the wrong lines — silent data corruption in the user's commit | Never — fail loud and ask the user to refresh instead |
| Grant broad Tauri capability scopes (e.g. `$HOME/**` fs read) during early development | Unblocks development faster | Ships an open-source tool with a worse trust/security story than necessary | Acceptable during Phase 1 spike only, must be narrowed before "Phát hành" phase |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|-------------------|
| `git` CLI subprocess | Trusting stderr text as a stable, parseable contract | Treat stderr as opaque diagnostic text only; use exit codes + porcelain stdout for control flow |
| `git` CLI subprocess | Not pinning `--date=`, causing locale-dependent date parsing | Always use `--date=unix` or `--date=iso-strict` in format strings |
| Windows Credential Manager / GCM | Assuming credentials are always cached; no fallback path tested | Explicitly test fetch/pull/push against a repo with zero cached credentials, verify fast, clear failure |
| Tauri v2 dialog/fs plugins | Assuming `Cargo.toml` dependency = granted permission | Add capability-file grant as part of the same commit that adds the plugin usage |
| Claude/OpenAI/Ollama APIs | No abstraction until "we'll add the second provider later" | PROJECT.md already commits to the abstraction upfront — verify diff-size and secret-scan logic lives in the shared layer, not duplicated per-provider |
| Ollama (local model) | Assuming it's always running / installed, same error handling as cloud APIs | Detect "connection refused to localhost" distinctly from cloud API errors, guide user to start Ollama or switch provider |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|------------------|
| Loading full `git log` output with no `--max-count`/pagination | Slow first paint on large repos, huge memory buffer | Page with `--skip`/`--max-count` as already planned in §5.1; never call unpaginated on repos > few thousand commits | Repos beyond ~5k-10k commits without pagination |
| Buffering entire subprocess stdout before parsing | High peak memory, UI freeze on huge diffs/logs | Stream stdout incrementally (read + parse in chunks) rather than waiting for process exit and reading all bytes at once | `git log` on repos with 100k+ commits, or diffs on files with huge generated content |
| Recomputing full lane/graph layout on every commit selection change | Janky UI on every click in a large history view | Separate "selected commit" UI state from graph layout data (already correctly identified in §3.3); layout computed once per page load, not per selection | Noticeable above a few thousand visible rows without virtualization |
| Watching + reacting to every raw filesystem event without debounce | High CPU, redundant `git status` calls, UI flicker | Debounce 250-500ms, filter to `.gitignore`-aware relevant paths only | Any repo with active build tooling or an autosaving editor open |
| Re-diffing unchanged commits on every graph scroll/re-render | Redundant subprocess spawns, sluggish scroll | Cache diff results keyed by commit SHA (already correctly identified in §3.1 point 4) | Large repos with heavy back-and-forth scrolling through history |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Sending staged diffs to AI providers without secret-pattern scanning | Credential/key leakage to a third party before the user even commits | Scan diff content for secret-shaped patterns before any external API call; block/warn, don't silently send |
| Storing AI provider API keys in a plaintext config file "just for now during dev" | Keys leak via screen-share, repo commit, or file sync (OneDrive is literally the project's own working directory here) | Use OS keychain from day one of the AI phase, as already correctly specified in PROJECT.md's constraints — don't let a dev-time shortcut slip into a release |
| Overly broad Tauri fs/shell capability scopes | Larger attack surface than necessary for an open-source tool asking for user trust | Narrow scopes to exactly the paths/commands needed (repo directories the user opens, not `$HOME/**`) |
| Logging full command-line args (which may include remote URLs with embedded credentials, e.g. `https://user:token@host/repo.git`) to a debug log file | Credential leakage via log files that might be attached to bug reports | Redact credentials from any logged git command lines/URLs before writing to disk or including in crash reports |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-------------------|
| Silent hang on credential-needed network operations (Pitfall 2) | User thinks the app is frozen/broken, force-quits mid-operation (risking a corrupted partial state) | Fast, explicit failure with a clear "credentials needed" message within seconds, not an infinite spinner |
| Windows console flash on every git call (Pitfall 5) | Reads as buggy/unpolished, undermines the "GitKraken-quality" positioning | Suppress via `CREATE_NO_WINDOW`, verify in release builds specifically |
| Generic "command failed" errors surfaced from raw git stderr | Users can't tell a merge conflict from a network failure from a permissions problem | Classify common git exit codes/stderr patterns into specific, actionable UI messages per operation type |
| Truncated/partial AI commit message with no indication it's partial | User trusts an AI summary that silently ignored most of a large diff | Explicit "based on N of M files" indicator when truncation happens |
| Stage-hunk button that fails with a raw "patch does not apply" error | Confusing, looks like an app bug rather than "file changed, please refresh" | Detect the stale-diff case specifically (Pitfall 4) and message it as such |

## "Looks Done But Isn't" Checklist

- [ ] **Git process spawning wrapper:** Often missing `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, stdin
      closure, and `CREATE_NO_WINDOW` (Windows) — verify by testing against a non-English-locale
      environment, a repo with no cached credentials, and a release-mode bundled build launched
      from Explorer (not `tauri dev`).
- [ ] **History/log parsing:** Often missing non-UTF-8 filename/commit-message handling —
      verify with a fixture repo containing at least one non-ASCII, non-UTF-8-encoded filename
      and a commit message with an embedded emoji or non-Latin script.
- [ ] **Commit graph lane algorithm:** Often missing octopus-merge (3+ parent) and
      orphan/unrelated-history handling — verify with synthetic fixture repos for both, plus a
      20+-lane wide-branching stress case.
- [ ] **Hunk staging:** Often missing the stale-diff race case and binary-file/no-trailing-newline
      edge cases — verify by editing a file between opening its diff and clicking stage, and by
      testing staging on a binary file and a file with no trailing newline.
- [ ] **External change detection:** Often missing `.gitignore`-aware filtering and debounce —
      verify CPU usage stays low with a build tool actively writing to the working tree while
      the app is open, and verify no `index.lock` errors appear when the app's own auto-refresh
      overlaps a user-initiated stage/commit.
- [ ] **AI integration:** Often missing secret-scanning and token-limit handling — verify by
      staging a diff containing an `.env`-style fake key and a very large (multi-thousand-line)
      diff, confirming neither is sent/processed unsafely.
- [ ] **Tauri capabilities:** Often missing narrowed scopes before release — verify by grepping
      capability files for wildcard (`**`) scopes shortly before the "Phát hành" phase and
      justifying or narrowing each one.

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|----------------|------------------|
| Missing locale/env-var hardening discovered post-release | LOW | Central fix in the single spawn wrapper function; no data migration needed, just a patch release |
| Non-UTF-8 filename crash discovered post-release | LOW-MEDIUM | Swap `from_utf8` to `from_utf8_lossy` at parse boundaries; add regression fixture repo; patch release |
| Lane algorithm wrong on octopus merges, discovered after graph code is deeply integrated with UI | MEDIUM | Algorithm is isolated per §4.3's clean data-structure output (`GraphRow`/`Edge`) — fixing the Rust algorithm alone shouldn't require UI changes if the contract is respected; still requires careful regression testing against the fixture repos |
| Console-window-flash discovered late (post-beta) | LOW | Single flag addition (`creation_flags`) in the spawn wrapper; no architectural change |
| Self-triggered `.git/index.lock` races discovered after file-watching and staging are both built independently | MEDIUM-HIGH | Requires retrofitting a serialization/mutex layer around all git-mutating operations after the fact — touches every write-path call site; cheaper to build this queue upfront (see Pitfall 8) |
| Secret leaked via AI API call discovered post-release | HIGH (trust/reputational, not just code) | Immediate patch adding secret-scan gate; public disclosure/changelog note given open-source trust positioning; cannot un-send data already sent to a third-party API |
| Scope creep / stalled project (Pitfall 10) discovered mid-project | MEDIUM | Requires a deliberate reset: freeze all non-core-loop feature work, force a dogfooding cycle on the current state before continuing — a process fix, not a code fix |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|-------------------|----------------|
| 1. Locale-dependent parsing | Platform (Nền tảng) | Run test suite / manual QA with system locale set to non-English (e.g. `vi-VN` or `ja-JP`) |
| 2. Credential-prompt hangs | Platform (spawn wrapper), verified in Branch/Remote | Fetch/pull/push against a repo with zero cached credentials must fail within a bounded timeout, never hang |
| 3. Non-UTF-8 filenames/messages | History reading (Đọc lịch sử) | Fixture repo with non-UTF-8 filename and non-Latin commit message parses without crash or IPC failure |
| 4. Stale-diff hunk-staging race | Working-tree changes (Thay đổi repository) | Test: edit file between diff-open and stage-click, confirm clear error not silent misapplication |
| 5. Windows console flash | Platform (Nền tảng) | Manual check: launch bundled release `.exe` from Explorer, trigger several git operations, confirm no console flash |
| 6. Tauri capability misconfiguration | Platform (Nền tảng), audited pre-release | Capability files reviewed for wildcard scopes before "Phát hành" phase; each new IPC command's capability grant added in the same commit |
| 7. Lane algorithm on octopus/orphan | History reading (Đọc lịch sử) | Fixture repos: octopus merge (3+ parents), unrelated-history merge, orphan branch, 20+-lane wide branch — all render without crash or visual corruption |
| 8. File-watcher event storms / lock races | Platform (serialize git ops) + Working-tree changes (watcher itself) | Stress test: build tool actively writing to working tree, CPU stays bounded; concurrent auto-refresh + user stage produces no `index.lock` error surfaced to user |
| 9. AI secret leakage / token limits | AI | Test diffs: one with a fake secret pattern (blocked/warned), one oversized (explicit truncation notice, not silent) |
| 10. Scope creep / stalled project | Cross-cutting, enforced via dogfood gates between phases | Dogfood gate: author uses the tool for a real commit on a real repo before proceeding from Diff-viewing to Working-tree, and again before Working-tree to Branch/Remote |

## Sources

- [tauri-apps/tauri Discussion #11446 — Terminal Window Briefly Opens/Closes on Windows](https://github.com/tauri-apps/tauri/discussions/11446)
- [tauri-apps/tauri Issue #11513 — Command spawn hanging in production](https://github.com/tauri-apps/tauri/issues/11513)
- [Tauri v2 Shell plugin docs](https://v2.tauri.app/plugin/shell/)
- [Tauri v2 Upgrade Guide from v1](https://v2.tauri.app/start/migrate/from-tauri-1/)
- [git-scm.com — git-log documentation (date formats, encoding)](https://git-scm.com/docs/git-log)
- [git-scm.com — git-commit documentation (i18n.commitEncoding)](https://git-scm.com/docs/git-commit/2.13.7)
- [git-scm.com — git-apply documentation (--cached, atomicity)](https://git-scm.com/docs/git-apply)
- [Nano-Collective/nanocoder Issue #1344 — execProcess lacks stdin closure, GIT_TERMINAL_PROMPT](https://github.com/Nano-Collective/nanocoder/issues/1344)
- [Microsoft/Git-Credential-Manager-for-Windows Issue #250 — Askpass mode support](https://github.com/Microsoft/Git-Credential-Manager-for-Windows/issues/250)
- [magit/magit Issue #3172 — "Patch does not apply" staging hunks with non-ASCII text](https://github.com/magit/magit/issues/3172)
- [Atlassian Jira SRCTREE-2789 — Error encountered when clicking Stage hunk button](https://jira.atlassian.com/browse/SRCTREE-2789)
- [microsoft/git PR #167 — Bugfix: commit-graph octopus merge offset error](https://github.com/microsoft/git/pull/167)
- [ship-studio Issue #860 — index.lock contention from undo/redo git calls not retrying](https://github.com/ship-studio/ship-studio/issues/860)
- [hegsie/gitnado PR #485 — Classify watcher events relative to .git directory](https://github.com/hegsie/gitnado/pull/485)
- [Andrew Lock — Fixing MAX_PATH issues in GitLab (Windows long path)](https://andrewlock.net/fixing-max_path-issues-in-gitlab/)
- `docs/01-research-competitors.md` §5 (existing git command/flag decisions, verified and extended above) and §7 (existing risk table, cross-referenced)
- `.planning/PROJECT.md` (constraints, out-of-scope decisions, stated risks)

---
*Pitfalls research for: Git GUI desktop client (Tauri v2 + Rust + git CLI wrapping)*
*Researched: 2026-09-21*
