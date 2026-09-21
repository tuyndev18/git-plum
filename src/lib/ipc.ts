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
  | 'no_repository_open'
  | 'unknown_repository'
  | 'parse_failed'
  | 'io'

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
  | { kind: 'text'; hunks: Hunk[]; truncated: boolean }
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

  // --- Spike đo hiệu năng diff (plan 03-01, checkpoint #3) ---
  //
  // Không nằm trên đường người dùng: chỉ `SpikeHarness` gọi, và harness chỉ hiện
  // khi `localStorage.gitPlumPerf === '1'`.
  spikeBlobPair: (repoId: string, commitId: string, path: string) =>
    invoke<SpikeBlobPair>('spike_blob_pair', { repoId, commitId, path }),
}
