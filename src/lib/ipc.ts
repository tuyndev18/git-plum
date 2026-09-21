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

// --- Các lệnh -------------------------------------------------------------

export const ipc = {
  openRepository: (path: string) => invoke<RepoInfo>('open_repository', { path }),

  listRepositories: () => invoke<RepoInfo[]>('list_repositories'),

  closeRepository: (id: string) => invoke<void>('close_repository', { id }),

  commandLog: () => invoke<CommandLogEntry[]>('command_log'),

  clearCommandLog: () => invoke<void>('clear_command_log'),

  gitVersion: () => invoke<string>('git_version'),

  currentBranch: () => invoke<string>('current_branch'),
}
