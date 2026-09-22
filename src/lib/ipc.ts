/**
 * Cầu nối sang lớp Rust.
 *
 * Mọi lệnh gọi xuống backend đi qua tệp này. Nhờ vậy việc giả lập trong kiểm thử
 * chỉ cần chặn một chỗ, và kiểu dữ liệu khớp với `src-tauri/src` nằm tập trung.
 */

import { invoke } from '@tauri-apps/api/core'

/** Thông tin một repository đang mở. Khớp `state::RepoInfo` bên Rust. */
export interface RepoInfo {
  id: string
  path: string
  name: string
}

/** Một dòng trong nhật ký lệnh. Khớp `state::CommandLogEntry` bên Rust. */
export interface CommandLogEntry {
  seq: number
  command: string
  cwd: string
  exitCode: number | null
  durationMs: number
  error: string | null
  startedAtMs: number
}

/**
 * Lỗi trả về từ Rust. Khớp phần `Serialize` viết tay của `GitError`.
 *
 * Phân nhánh xử lý theo `code`, không so khớp `message` — thông điệp là để cho
 * người đọc, có thể đổi bất cứ lúc nào.
 */
export interface GitErrorPayload {
  code: GitErrorCode
  message: string
  /** Lệnh git đã chạy, dạng `git rev-parse --show-toplevel`. */
  command: string | null
}

export type GitErrorCode =
  | 'spawn_failed'
  | 'timeout'
  | 'stdin_write_failed'
  | 'command_failed'
  | 'not_a_repository'
  /**
   * `.git/index.lock` đang bị một tiến trình git khác giữ — WORK-02.
   *
   * 🔴 Mã **riêng**, không gộp vào `command_failed`, vì giao diện phải nói một câu
   * khác hẳn: đây không phải lỗi mà là một trạng thái **tạm thời có thể phục hồi**.
   * Người dùng đang chạy git ở terminal (chủ dự án làm việc như vậy — đó là cả điểm
   * của WORK-10), nên câu đúng là "đang chờ một tiến trình git khác, thử lại sau",
   * không phải "git thoát với mã 128".
   *
   * Phía Rust đã thử lại có giãn cách (tổng dưới 1 giây) trước khi trả mã này, và nó
   * **không bao giờ** xoá tệp lock — xem `commands::worktree`.
   */
  | 'index_locked'
  | 'no_repository_open'
  | 'unknown_repository'
  | 'parse_failed'
  | 'io'
  /**
   * Thông điệp commit rỗng hoặc chỉ khoảng trắng — WORK-08.
   *
   * Lỗi **trước** khi chạy git, mã riêng. R5 của `CONTEXT.md`: **không tự sửa thông
   * điệp**, không tự thêm nội dung. Gộp nó vào `command_failed` sẽ khiến giao diện
   * hiện stderr thô của git cho một ca mà ta biết chính xác vấn đề là gì.
   */
  | 'empty_message'
  /**
   * Không có gì để commit (không tệp nào đã stage).
   *
   * Mã riêng vì câu đúng cho người dùng là "hãy stage một tệp trước", không phải một
   * dòng stderr của git.
   */
  | 'nothing_to_commit'
  /**
   * Hook `pre-commit` hoặc `commit-msg` **từ chối** — WORK-08, ràng buộc 2.4.
   *
   * 🔴 `message` mang **nguyên văn** stdout+stderr của hook. Với người dùng đây là
   * một **thông báo cần đọc**, không phải lỗi của ứng dụng: chính họ (hoặc dự án của
   * họ) viết hook đó, và đầu ra của nó có xuống dòng và thụt lề **mang nghĩa**. Giao
   * diện hiện nó trong `<pre>`.
   *
   * Mã riêng vì `git commit` thoát khác 0 vì **nhiều** nguyên nhân — hook, không có
   * gì để commit, `index.lock`, thông điệp sai. Gán một nguyên nhân cho mọi exit
   * khác 0 là đúng lỗi KB-4b mà `CONTEXT.md` mục 0 cấm lặp lại.
   */
  | 'hook_rejected'

/** Nhận biết lỗi đến từ lớp Rust, phân biệt với lỗi JavaScript thường. */
export function isGitError(e: unknown): e is GitErrorPayload {
  return (
    typeof e === 'object' &&
    e !== null &&
    'code' in e &&
    'message' in e &&
    typeof (e as GitErrorPayload).message === 'string'
  )
}

/** Thông điệp hiển thị cho người dùng, kèm lệnh đã chạy khi có (PLAT-10). */
export function describeError(e: unknown): string {
  if (isGitError(e)) {
    return e.command ? `${e.message}\n\nLệnh đã chạy: ${e.command}` : e.message
  }
  if (e instanceof Error) return e.message
  return String(e)
}

// --- Kiểu của lớp lịch sử (Phase 2) ---------------------------------------
//
// Mọi tên khoá dưới đây khớp **từng chữ** với JSON mà Rust sinh ra. Phía Rust dùng
// `#[serde(rename_all = "camelCase")]` và tên khoá đã được ghim bằng test ở
// `domain/commit.rs`, `domain/refs.rs`, `graph/types.rs` và `commands/history.rs`.
// Đổi một bên mà quên bên kia không gây lỗi biên dịch ở cả hai phía — giao diện chỉ
// nhận `undefined` và hiển thị trống.

/** Một commit. Khớp `domain::Commit` bên Rust. */
export interface Commit {
  id: string
  parents: string[]
  authorName: string
  authorEmail: string
  /** Giây Unix. Định dạng bằng `Intl.DateTimeFormat` — Rust không gửi chuỗi ngày. */
  authorTime: number
  committerName: string
  committerEmail: string
  committerTime: number
  subject: string
  body: string
  /** `true` khi commit này có byte không phải UTF-8 đã đi qua đường giải mã lossy. */
  hasInvalidUtf8: boolean
}

/** Một đường nối giữa hai lane trên một dòng. Khớp `graph::Edge`. */
export interface Edge {
  fromLane: number
  toLane: number
  color: number
}

/**
 * Hình học một dòng đồ thị. Khớp `graph::GraphRow`.
 *
 * Rust tính toàn bộ hình học; frontend **chỉ vẽ**, không tự suy ra liên thông.
 *
 * `lane` có thể **vượt** giới hạn hiển thị (20): giới hạn chỉ áp cho lane của cha, còn
 * mọi commit đều nhận một lane thật. Bộ vẽ phải gập lane ≥ 20 vào cột cuối — xem
 * `docs/04-phase2-degraded-graph.md`.
 */
export interface GraphRow {
  commitId: string
  lane: number
  color: number
  passthrough: Edge[]
  outEdges: Edge[]
  /** Số cha không vẽ được vì hết lane. Giao diện hiện chỉ báo `+N cha nữa`. */
  truncatedParents: number
  /** `true` khi commit khai báo một cha chưa được nạp (xảy ra ở biên mỗi trang). */
  terminates: boolean
}

/** Một trang lịch sử. Khớp `commands::history::CommitPage`. */
export interface CommitPage {
  commits: Commit[]
  /** **Luôn cùng độ dài** với `commits` — lệch một phần tử là đồ thị lệch hàng. */
  graphRows: GraphRow[]
  /** Tổng số commit trong toàn bộ lịch sử, để đặt chiều cao vùng cuộn ảo hoá. */
  total: number
  skippedRecords: number
}

/** Loại tham chiếu. Khớp chuỗi mà `domain::RefKind` serialize ra. */
export type RefKind = 'localBranch' | 'remoteBranch' | 'tag' | 'other'

/**
 * Một tham chiếu. Khớp `domain::Ref` bên Rust.
 *
 * Tên là `GitRef` chứ không phải `Ref`: `Ref` trùng với kiểu của React và sẽ gây lẫn
 * trong tệp component.
 */
export interface GitRef {
  /** Tên đầy đủ, ví dụ `refs/heads/main`. An toàn để truyền lại cho lệnh git. */
  fullName: string
  /** Tên hiển thị, ví dụ `main` hoặc `origin/main`. */
  shortName: string
  kind: RefKind
  /**
   * Mã **commit** mà ref trỏ tới, đã giải tham chiếu với tag có chú thích. Dùng nó để
   * neo nhãn vào đúng hàng `GraphRow.commitId`.
   */
  target: string
  /** `null` khi chưa đặt thượng nguồn. Với `[gone]` thì tên vẫn còn — hai ca khác nhau. */
  upstream: string | null
  ahead: number
  behind: number
  isHead: boolean
}

/** Một tệp thay đổi. Khớp `commands::history::FileChange`. */
export interface FileChange {
  /** Chữ trạng thái của git, kèm điểm tương đồng với đổi tên: `M`, `A`, `R100`, … */
  status: string
  path: string
  /** Chỉ khác `null` với `R` (đổi tên) và `C` (sao chép). */
  oldPath: string | null
}

/** Chi tiết một commit. Khớp `commands::history::CommitDetail`. */
export interface CommitDetail {
  commit: Commit
  files: FileChange[]
  /** `true` khi danh sách tệp đã bị cắt vì commit đổi quá nhiều tệp. */
  truncated: boolean
}

// --- Kiểu của trình xem diff (Phase 3) ------------------------------------
//
// Khớp `domain::diff` bên Rust, và tên khoá đã được ghim bằng test ở
// `domain/diff.rs`. Lưu ý một cái bẫy đã gặp thật khi viết phía Rust:
// `#[serde(rename_all = "camelCase")]` đặt ở **cấp enum** chỉ đổi tên **biến thể**,
// KHÔNG đổi tên trường bên trong biến thể — nên mỗi biến thể mang dữ liệu phải lặp
// lại thuộc tính đó. Bản đầu serialize `old_size` trong khi phía này đọc `oldSize`,
// và không bên nào lỗi biên dịch.

/** Loại của một dòng trong bản vá. Khớp `domain::diff::LineKind`. */
export type LineKind = 'context' | 'added' | 'removed'

/**
 * Khoảng **byte** vào `DiffLine.content` — phần chữ thay đổi trong dòng (diff mức từ).
 * Khớp `domain::diff::Span`.
 *
 * ## ⚠️ Đơn vị là BYTE, còn CodeMirror đánh chỉ số theo UTF-16 code unit
 *
 * Hai hệ này **khác nhau** ngay khi dòng có ký tự ngoài ASCII:
 *
 * | ký tự | byte (UTF-8) | UTF-16 code unit |
 * |---|---|---|
 * | `a`   | 1 | 1 |
 * | `é`   | 2 | 1 |
 * | `ỏ`   | 3 | 1 |
 * | emoji ngoài BMP | 4 | **2** |
 *
 * Nên **không** truyền thẳng `start`/`end` làm offset cho decoration của CodeMirror.
 * Phải chuyển hệ trước, ví dụ bằng cách đếm lại trên chuỗi JS:
 *
 * ```ts
 * // byte offset -> UTF-16 index, dùng chính `content` mà Rust gửi kèm
 * const utf16 = new TextDecoder().decode(
 *   new TextEncoder().encode(content).slice(0, byteOffset),
 * ).length
 * ```
 *
 * Byte được chọn làm hệ gốc vì đó là hệ mà git nói, và mọi phép chuyển ở phía Rust
 * cũng chỉ là một cơ hội lệch một nấc nữa.
 *
 * ## Bất biến do phía Rust bảo đảm
 *
 * Hai đầu luôn nằm trên **biên ký tự** — không bao giờ cắt một ký tự nhiều byte làm
 * đôi. Có test ghim ở `git::parsers::word_diff` trên nội dung tiếng Việt.
 */
export interface Span {
  /** Chỉ số byte đầu, **bao gồm**. */
  start: number
  /** Chỉ số byte cuối, **không** bao gồm. */
  end: number
}

/**
 * Một dòng trong một hunk. Khớp `domain::diff::DiffLine`.
 *
 * `oldLine` và `newLine` là hai trường riêng vì chế độ **hai cột** (DIFF-02) dựng bố
 * cục từ đúng hai số này: dòng ngữ cảnh chiếm một hàng ở cả hai cột, dòng `added` chỉ
 * ở cột phải, dòng `removed` chỉ ở cột trái.
 */
export interface DiffLine {
  kind: LineKind
  /** Không kèm ký tự tiền tố và không kèm ký tự kết thúc dòng (kể cả `\r`). */
  content: string
  /** `null` với dòng `added`. */
  oldLine: number | null
  /** `null` với dòng `removed`. */
  newLine: number | null
  /** `true` khi ngay sau dòng này git in `\ No newline at end of file`. */
  noNewlineAtEof: boolean
  /**
   * Khoảng chữ **thay đổi** trong dòng — diff mức từ.
   *
   * Mảng **rỗng** nghĩa là "không có thông tin mức từ cho dòng này", và đó là ca
   * bình thường chứ không phải lỗi:
   *
   * - dòng `context` không bao giờ có khoảng (nó không đổi);
   * - tệp **chỉ thêm** hoặc **chỉ xoá** bỏ hẳn lệnh word-diff — mọi dòng đều mới
   *   hoặc đều mất, nên "phần chữ thay đổi" là cả dòng;
   * - tệp sửa quá 2000 dòng bỏ hẳn word-level (suy giảm có chủ ý).
   *
   * Giao diện **phải** vẽ được dòng khi mảng rỗng: tô cả dòng là suy giảm đúng.
   */
  spans: Span[]
}

/** Một khối thay đổi liền mạch. Khớp `domain::diff::Hunk`. */
export interface Hunk {
  oldStart: number
  oldCount: number
  newStart: number
  newCount: number
  /** Phần sau cặp `@@` thứ hai — git đặt tên hàm chứa hunk vào đó. Rỗng khi không có. */
  heading: string
  lines: DiffLine[]
}

/**
 * Năm dạng kết quả xem diff. Khớp `domain::diff::DiffKind`, serialize với
 * `#[serde(tag = "kind")]` nên đây là *discriminated union* phẳng: phân nhánh theo
 * `kind` và TypeScript thu hẹp kiểu giúp.
 *
 * `binary` **không có** trường nội dung — DIFF-06 chặn ở tầng kiểu ở cả hai phía
 * (T-03-13), không ở một nhánh `if` mà người sau có thể xoá.
 */
export type DiffKind =
  /**
   * `truncated` và `contextOnly` là **hai cờ độc lập** mang hai nghĩa khác nhau,
   * và giao diện phải nói hai câu khác nhau:
   *
   * | cờ | nghĩa | người dùng mất gì |
   * |---|---|---|
   * | `truncated` | bản vá **bị cắt mất nội dung** | **có** — không thấy hết thay đổi |
   * | `contextOnly` | thấy **đủ mọi thay đổi**, chỉ thiếu ngữ cảnh không đổi ở giữa | **không** mất gì |
   *
   * `contextOnly: true` nghĩa là backend đã lùi về `--unified=3` thay vì hiện
   * toàn tệp, vì tệp vượt `MAX_BYTE_TOAN_TEP` (512 KB) — xem
   * `commands::diff::so_dong_ngu_canh`.
   *
   * 🔴 Cờ này **phải** được hiển thị khi `true`. Im lặng lùi về rút gọn chính là
   * việc đã xảy ra ngày 2026-09-22: người dùng thấy số dòng nhảy (5 → 24 → 37),
   * tưởng trình xem lỗi, và báo **hai lần**. Một cờ đúng ở backend mà giao diện
   * bỏ qua thì không sửa được gì — nó chỉ chuyển lỗi im lặng sang tầng khác.
   * `DiffToolbar.test.tsx` có test ghim cả băng thông báo lẫn việc nó **không**
   * dùng chữ "bị cắt".
   */
  | { kind: 'text'; hunks: Hunk[]; truncated: boolean; contextOnly: boolean }
  | { kind: 'binary'; oldSize: number; newSize: number }
  /** `limit` là ngưỡng đang áp (5 MB). Hiện cả hai số để người dùng biết vượt bao nhiêu. */
  | { kind: 'tooLarge'; size: number; limit: number }
  /** `size` là số **trong con trỏ**, không phải kích thước tệp con trỏ (~130 byte). */
  | { kind: 'lfsPointer'; oid: string; size: number }
  /** Hai phía giống hệt nhau — xảy ra với commit chỉ đổi mode tệp. */
  | { kind: 'unchanged' }

/** Diff của một tệp trong một commit. Khớp `domain::diff::FileDiff`. */
export interface FileDiff {
  /** Đường dẫn ở phía mới. */
  path: string
  /** Chỉ khác `null` khi tệp bị đổi tên hoặc sao chép. */
  oldPath: string | null
  /** Chữ trạng thái git kèm điểm tương đồng: `M`, `A`, `D`, `R77`, `C75`. */
  status: string
  kind: DiffKind
}

/**
 * Dữ liệu đầu vào cho **cả hai** đường đo của checkpoint #3. Khớp
 * `commands::diff_spike::SpikeBlobPair` bên Rust.
 *
 * Hai đường phải đo trên **cùng một byte đầu vào**, nên một lời gọi trả cả hai
 * dạng: `oldText`/`newText` cho đường A (`@codemirror/merge` tự tính diff) và
 * `patch` cho đường B (decoration dựng từ đầu ra `git diff`).
 *
 * **Chỉ phục vụ spike.** Xoá cùng lúc với `SpikeHarness` khi Task 3 của plan
 * 03-01 chốt xong đường đi.
 */
export interface SpikeBlobPair {
  oldText: string
  newText: string
  patch: string
  oldBytes: number
  newBytes: number
  patchBytes: number
  oldLines: number
  newLines: number
  /** Thời gian phía Rust (ba lệnh git), để tách "git chậm" khỏi "CodeMirror chậm". */
  rustMs: number
}


/**
 * Một phiên bản của **một** tệp — một commit đã sửa nó (DIFF-05).
 *
 * Khớp `domain::diff::FileVersion`. Tên khoá đã ghim bằng test ở phía Rust.
 */
export interface FileVersion {
  /** Mã commit đầy đủ (40 hex). Giao diện tự cắt ngắn khi hiện. */
  commitId: string
  authorName: string
  /**
   * **Giây** Unix, không phải milli.
   *
   * 🔴 `new Date(authorTime)` cho một ngày năm 1970 — `Date` nhận milli. Phải là
   * `new Date(authorTime * 1000)`. Lỗi này đã nằm trong checklist checkpoint của
   * 02-06, tức là một lỗi **đã được lường trước** và vẫn đáng một test riêng.
   */
  authorTime: number
  subject: string
  /** Chữ trạng thái của git kèm điểm tương đồng: `M`, `A`, `D`, `R100`, `C075`. */
  status: string
  /** Đường dẫn của tệp **tại commit đó** — đổi ở mỗi lần đổi tên. */
  path: string
  /**
   * Tên **cũ**, chỉ khác `null` ở bản ghi `R` (đổi tên) và `C` (sao chép).
   *
   * 🔴 Giao diện **phải hiện nó ra**. Đây là thứ `--follow` mua được, và một lịch sử
   * lần qua chỗ đổi tên trong im lặng làm người dùng không hiểu vì sao đường dẫn ở
   * hàng dưới khác hàng trên.
   */
  oldPath: string | null
}

/** Lịch sử của một tệp — DIFF-05. Khớp `domain::diff::FileHistory`. */
export interface FileHistory {
  /** Đường dẫn được hỏi (tên **hiện tại** của tệp). */
  path: string
  /** Phiên bản, mới nhất trước. */
  versions: FileVersion[]
  /**
   * `true` khi đã chạm chặn trên 200 phiên bản.
   *
   * Phải hiện **trên giao diện**, không chỉ nằm trong payload (T-03-38): một lịch sử
   * bị cắt mà không ai nói ra là một lịch sử sai mà không ai biết.
   */
  truncated: boolean
}
// --- Kiểu của trạng thái thư mục làm việc (Phase 4) -----------------------
//
// Khớp `domain::status` bên Rust, `#[serde(rename_all = "camelCase")]`. Tên khoá đã
// được ghim bằng test ở phía Rust (`domain/status.rs`). Lưu ý cùng cái bẫy mà khối
// diff ở trên ghi lại: đặt `rename_all` ở **cấp enum** chỉ đổi tên biến thể, không
// đổi tên trường bên trong.

/**
 * Ba nhóm tệp của WORK-01. Khớp `domain::StatusGroup`.
 *
 * Phép phân nhóm nằm ở **Rust**, không ở đây: suy ra nhóm từ hai ký tự XY (`.M` là
 * chưa stage, `M.` là đã stage, `MM` là **cả hai**) là *logic*, không phải trình bày,
 * và để nó ở giao diện nghĩa là nó không có một test Rust nào.
 */
export type StatusGroup = 'staged' | 'unstaged' | 'untracked'

/**
 * Một tệp trong một nhóm trạng thái. Khớp `domain::StatusEntry`.
 *
 * 🔴 Một tệp XY = `MM` sinh **hai** phần tử — một `staged`, một `unstaged` — vì nó
 * thật sự xuất hiện ở cả hai nhóm trên giao diện (khuôn GitHub Desktop). Nên `path`
 * **không** phải khoá duy nhất của danh sách; khoá React phải gồm cả `group`.
 */
export interface StatusEntry {
  /** Đường dẫn hiện tại, tương đối gốc repo, dấu `/` như git in ra. */
  path: string
  /**
   * Đường dẫn **cũ** của tệp đổi tên/sao chép (bản ghi `--porcelain=v2` dạng `2`).
   * `null` cho mọi dạng khác. git in **mới trước, cũ sau** — đã đo bằng `od -c`.
   */
  oldPath: string | null
  /** Hai ký tự XY, ví dụ `M.`, `.M`, `MM`, `R.`, `??`, `UU`. */
  xy: string
  group: StatusGroup
  /**
   * `true` khi đường dẫn phải giải mã lossy (byte không phải UTF-8 hợp lệ).
   *
   * Giao diện **không** được dùng `path` của phần tử này làm đối số cho `stageFiles`:
   * chuỗi đã mất byte gốc và git sẽ không khớp tệp nào.
   */
  hasInvalidUtf8: boolean
}

/** Nhánh hiện tại và quan hệ với upstream. Khớp `domain::BranchInfo`. */
export interface BranchInfo {
  /** `null` khi HEAD tách rời. */
  head: string | null
  /** SHA của HEAD; `null` ở repo chưa có commit nào. */
  oid: string | null
  upstream: string | null
  /**
   * 🔴 `null` ≠ `0`. Dòng `# branch.ab` **vắng mặt hoàn toàn** khi nhánh không có
   * upstream; git **không** in `+0 -0`.
   *
   * WORK-09 phải phân biệt "chưa từng push" (`null` — amend vô hại) với "đã push và
   * đang đồng bộ" (`0` — amend viết lại lịch sử người khác đã thấy, phải cảnh báo).
   */
  ahead: number | null
  behind: number | null
}

/** Số đếm hàng WIP của WORK-11 (`✏3 +1`). Khớp `domain::WipCounts`. */
export interface WipCounts {
  modified: number
  added: number
}

/** Trạng thái thư mục làm việc. Khớp `domain::RepoStatus`. */
export interface RepoStatus {
  branch: BranchInfo
  /** Mọi phần tử, **giữ nguyên thứ tự git in ra**. Xem `StatusEntry` về tệp `MM`. */
  entries: StatusEntry[]
  /** `true` khi có xung đột chưa giải quyết — nó **chặn** commit của WORK-08. */
  hasConflicts: boolean
}

/**
 * Kết quả của `amendCommit` — WORK-09. Khớp `commands::commit::AmendResult`.
 *
 * 🔴 `wasPushed` là một **cảnh báo**, không phải một phép chặn.
 *
 * Nguyên văn ROADMAP: amend trên commit đã push **cảnh báo nhưng KHÔNG chặn**. Nên
 * trường này về cùng **kết quả của một thao tác đã chạy xong**, không phải một câu
 * hỏi trước khi chạy. Người dùng biết họ đang làm gì; một hộp thoại "bạn có chắc?"
 * ở đây là sai yêu cầu, và nó là bước đầu của việc chặn.
 *
 * Suy từ `BranchInfo` của **cùng** lời gọi status, **không** phải một lệnh git thứ
 * hai — cùng tinh thần với ràng buộc đếm của WORK-11.
 */
export interface AmendResult {
  /** Trạng thái **sau** khi amend. Cùng ràng buộc 2.5 với mọi lệnh ghi khác. */
  status: RepoStatus
  /**
   * `true` khi nhánh có upstream **và** `ahead === 0` — tức commit vừa bị viết lại
   * là commit người khác đã thấy.
   *
   * 🔴 `ahead === null` (không có upstream) → `false`, **không** `true`. Dòng
   * `# branch.ab` vắng mặt hoàn toàn khi không có upstream; git không in `+0 -0`.
   * Đột biến M11 ghim ca này.
   */
  wasPushed: boolean
}

// --- Các lệnh -------------------------------------------------------------

export const ipc = {
  openRepository: (path: string) => invoke<RepoInfo>('open_repository', { path }),

  listRepositories: () => invoke<RepoInfo[]>('list_repositories'),

  closeRepository: (id: string) => invoke<void>('close_repository', { id }),

  commandLog: () => invoke<CommandLogEntry[]>('command_log'),

  clearCommandLog: () => invoke<void>('clear_command_log'),

  gitVersion: () => invoke<string>('git_version'),

  currentBranch: () => invoke<string>('current_branch'),

  // --- Lịch sử (Phase 2) ---
  //
  // Tauri v2 chuyển tham số `snake_case` của Rust thành `camelCase` ở phía JS: tham số
  // Rust `repo_id` gọi bằng `repoId`. Sai chỗ này thì `invoke` thất bại **lúc chạy**,
  // không lúc biên dịch — nên `ipc.history.test.ts` khẳng định cả tên command lẫn hình
  // dạng đối số cho từng hàm.

  getCommitPage: (repoId: string, skip: number, limit: number) =>
    invoke<CommitPage>('get_commit_page', { repoId, skip, limit }),

  listRefs: (repoId: string) => invoke<GitRef[]>('list_refs', { repoId }),

  getCommitDetail: (repoId: string, commitId: string) =>
    invoke<CommitDetail>('get_commit_detail', { repoId, commitId }),

  searchCommits: (repoId: string, query: string) =>
    invoke<string[]>('search_commits', { repoId, query }),

  // --- Trình xem diff (Phase 3) ---
  //
  // Lời gọi thứ hai cho cùng `(commitId, path)` được cache phía Rust trả lời và
  // **không sinh tiến trình git nào** — tiêu chí thành công số 5 của phase. Nên phía
  // này không cần tự nhớ kết quả; `staleTime: Infinity` của TanStack Query là đủ, và
  // cả hai lớp cùng dựa trên một sự thật: diff của commit lịch sử là bất biến.
  getFileDiff: (repoId: string, commitId: string, path: string) =>
    invoke<FileDiff>('get_file_diff', { repoId, commitId, path }),


  /**
   * Lịch sử thay đổi của một tệp — DIFF-05.
   *
   * 🔴 **Không cache phía nào.** Khác `getFileDiff` (bất biến theo `(commitId, path)`,
   * nên `staleTime: Infinity` là đúng), lịch sử tệp phụ thuộc **HEAD**: commit mới
   * xuất hiện sẽ đổi kết quả cho cùng một `path`. Phía Rust cố ý không cache — xem
   * `lay_lich_su_tep`. Đặt `staleTime` dài ở đây sẽ dựng lại đúng lớp dữ liệu cũ mà
   * phía Rust vừa từ chối tạo ra, và hệ quả là người dùng commit rồi mở lại lịch sử
   * tệp mà **không thấy commit của chính mình**.
   */
  getFileHistory: (repoId: string, path: string) =>
    invoke<FileHistory>('get_file_history', { repoId, path }),
  // --- Spike đo hiệu năng diff (plan 03-01, checkpoint #3) ---
  //
  // Không nằm trên đường người dùng: chỉ `SpikeHarness` gọi, và harness chỉ hiện
  // khi `localStorage.gitPlumPerf === '1'`.
  spikeBlobPair: (repoId: string, commitId: string, path: string) =>
    invoke<SpikeBlobPair>('spike_blob_pair', { repoId, commitId, path }),

  // --- Avatar ---

  /**
   * MD5 của một email, cho URL Gravatar.
   *
   * 🔴 **Chỉ gọi khi người dùng đã bật Gravatar**, và chỉ cho commit đang chọn.
   *
   * Vì sao là một command riêng chứ không phải một trường trên `Commit`: hash
   * hex là 32 ký tự, nên gắn nó vào mỗi bản ghi thêm ~3,2MB JSON trên repo
   * 100k commit — cho một tính năng **mặc định tắt**, trên đúng đường nóng của
   * Core Value. Xem doc comment đầy đủ ở `src-tauri/src/commands/avatar.rs`.
   *
   * Băm ở Rust để **email không bao giờ rời khỏi Rust**: Web Crypto của trình
   * duyệt không có MD5, nên đường thay thế là gửi email sang đây để băm.
   */
  avatarHash: (email: string) => invoke<string>('avatar_hash', { email }),

  // --- Thư mục làm việc (Phase 4) ---
  //
  // 🔴 `stageFiles` và `unstageFiles` trả **`RepoStatus` mới**, không trả `void`.
  //
  // Đây là ràng buộc 2.5 của CONTEXT.md, và kiểu trả về là cách bảo đảm nó. Một lệnh
  // ghi trả `void` **buộc** giao diện phải chờ watcher, và hai thứ hỏng theo: trễ
  // 250–300 ms sau **mỗi** cú bấm (khoảng gộp-và-trì-hoãn của watcher), và nếu watcher
  // chết thì danh sách đứng im mà không ai biết — không lỗi, không thông báo.
  //
  // Nên `statusStore.stage` ghi `RepoStatus` **từ giá trị trả về** và **không** gọi
  // `getStatus` sau đó. Có test ghim (`statusStore.test.ts`).

  /** Trạng thái thư mục làm việc — WORK-01. */
  getStatus: (repoId: string) => invoke<RepoStatus>('get_status', { repoId }),

  /** Stage các tệp — WORK-02. Trả trạng thái **mới**. */
  stageFiles: (repoId: string, paths: string[]) =>
    invoke<RepoStatus>('stage_files', { repoId, paths }),

  /** Bỏ stage các tệp — WORK-02. Trả trạng thái **mới**. */
  unstageFiles: (repoId: string, paths: string[]) =>
    invoke<RepoStatus>('unstage_files', { repoId, paths }),

  /**
   * Diff của **thư mục làm việc** — WORK-01 tiêu chí 2.
   *
   * `staged: true` → `git diff --cached` ("thứ sẽ vào commit tới");
   * `staged: false` → `git diff` ("thứ chưa stage").
   *
   * 🔴 **Không đặt `staleTime` dài cho lời gọi này.** Khác `getFileDiff` (bất biến
   * theo `(commitId, path)`, nên `staleTime: Infinity` là đúng), diff thư mục làm việc
   * đổi **mỗi lần người dùng gõ**. Phía Rust cố ý **không** cache nó — xem
   * `commands::diff::lay_diff_thu_muc_lam_viec`. Đặt cache ở đây sẽ dựng lại đúng lớp
   * dữ liệu cũ mà phía Rust vừa từ chối tạo ra, và ở phase này dữ liệu cũ **nguy
   * hiểm**: người dùng quyết định stage cái gì dựa trên diff họ đang xem.
   */
  getWorktreeDiff: (repoId: string, path: string, staged: boolean) =>
    invoke<FileDiff>('get_worktree_diff', { repoId, path, staged }),

  // --- Vòng commit (Phase 4, WORK-08 / WORK-09) ---
  //
  // Cả hai là **lệnh ghi** và trả trạng thái **mới** trực tiếp — ràng buộc 2.5 của
  // `CONTEXT.md`, cùng lập luận đã ghi ở `stageFiles` phía trên: chờ watcher làm
  // giao diện trễ 250–300 ms sau mỗi cú bấm, và watcher chết thì giao diện đứng im
  // mà không ai biết.

  /**
   * Tạo commit từ những tệp **đã stage** — WORK-08. Trả trạng thái **mới**.
   *
   * 🔴 `noVerify` **mặc định `false`**, và đó là một quyết định của ROADMAP chứ
   * không phải một giá trị mặc định tiện tay: hook `pre-commit` và `commit-msg`
   * chạy **MẶC ĐỊNH**, và `no-verify` là công tắc **tường minh** người dùng phải tự
   * bật. Bỏ hook là bỏ một trong những lý do dự án chọn `git` CLI thay vì libgit2.
   *
   * Tham số được khai **bắt buộc** ở đây, không có giá trị mặc định trong TypeScript:
   * mặc định nằm ở ô tick của `CommitBox` (tắt) và ở chữ ký Rust. Thêm `= false`
   * ở đây nghĩa là có **hai** chỗ khai cùng một mặc định, và chúng lệch nhau được.
   *
   * Hook từ chối → ném `GitErrorPayload` mã `hook_rejected`, `message` mang
   * **nguyên văn** đầu ra hook. Thông điệp người dùng vừa gõ **không** bị mất — đó là
   * việc của `commitStore`, và đột biến M6 ghim nó.
   */
  createCommit: (repoId: string, message: string, noVerify: boolean) =>
    invoke<RepoStatus>('create_commit', { repoId, message, noVerify }),

  /**
   * Sửa commit gần nhất — WORK-09. Trả `AmendResult`, **không** chỉ `RepoStatus`.
   *
   * Kiểu trả về khác `createCommit` vì `wasPushed` chỉ có nghĩa ở đường amend.
   * Nhồi nó vào `RepoStatus` sẽ đặt một trường luôn `false` lên mọi lời gọi
   * `getStatus`, và trường luôn `false` là trường không ai kiểm.
   *
   * Command **không chặn** trong bất kỳ ca nào — xem `AmendResult.wasPushed`.
   */
  amendCommit: (repoId: string, message: string, noVerify: boolean) =>
    invoke<AmendResult>('amend_commit', { repoId, message, noVerify }),
}
