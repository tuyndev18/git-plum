/**
 * Test ranh giới IPC cho các command thư mục làm việc (Phase 4) — WORK-01, WORK-02.
 *
 * Các test ở đây không kiểm logic — chúng khoá lại **hợp đồng gọi**: tên command và
 * hình dạng đối số. Cả hai thứ đó chỉ thất bại **lúc chạy**: gõ sai `stage_files`
 * thành `stageFiles`, hay truyền `repo_id` thay cho `repoId`, đều biên dịch sạch ở cả
 * TypeScript lẫn Rust rồi ném "command not found" trên máy người dùng.
 *
 * Tauri v2 chuyển tham số `snake_case` của Rust thành `camelCase` ở phía JS. Đó là lý
 * do khoá đối số là `repoId` chứ không phải `repo_id` — đột biến M11 của plan.
 *
 * Khuôn chép từ `ipc.history.test.ts`, kể cả phần `beforeEach` dựng lại hàm giả:
 * `vite.config.ts` đặt `restoreMocks: true`, nên `mockResolvedValue` đặt trong factory
 * của `vi.mock` bị gỡ giữa các test.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'
import { ipc, type FileDiff, type RepoStatus, type StatusEntry } from './ipc'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

beforeEach(() => {
  invokeMock.mockReset()
  invokeMock.mockResolvedValue(undefined)
})

describe('tên command và hình dạng đối số', () => {
  it('getStatus gọi get_status với { repoId }', async () => {
    await ipc.getStatus('repo-1')

    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(invokeMock).toHaveBeenCalledWith('get_status', { repoId: 'repo-1' })
  })

  it('stageFiles gọi stage_files với { repoId, paths }', async () => {
    await ipc.stageFiles('repo-1', ['a.txt', 'thu muc/b.txt'])

    expect(invokeMock).toHaveBeenCalledWith('stage_files', {
      repoId: 'repo-1',
      paths: ['a.txt', 'thu muc/b.txt'],
    })
  })

  it('unstageFiles gọi unstage_files với { repoId, paths }', async () => {
    await ipc.unstageFiles('repo-1', ['a.txt'])

    expect(invokeMock).toHaveBeenCalledWith('unstage_files', {
      repoId: 'repo-1',
      paths: ['a.txt'],
    })
  })

  it('getWorktreeDiff gọi get_worktree_diff với { repoId, path, staged }', async () => {
    await ipc.getWorktreeDiff('repo-1', 'a.txt', true)

    expect(invokeMock).toHaveBeenCalledWith('get_worktree_diff', {
      repoId: 'repo-1',
      path: 'a.txt',
      staged: true,
    })
  })

  /**
   * `staged` phải đi qua đúng như nhận vào — `false` là một giá trị, không phải
   * "không truyền". Một cài đặt bỏ khoá khi `staged === false` làm Rust nhận
   * `undefined` và `invoke` thất bại lúc chạy vì thiếu tham số bắt buộc.
   */
  it('getWorktreeDiff giữ staged=false, không bỏ khoá đi', async () => {
    await ipc.getWorktreeDiff('repo-1', 'a.txt', false)

    const payload = invokeMock.mock.calls[0]![1] as Record<string, unknown>
    expect(Object.keys(payload)).toContain('staged')
    expect(payload.staged).toBe(false)
  })

  /**
   * 🔴 Đột biến M11. Khoá phải là `repoId` (camelCase), **không** `repo_id`. Một test
   * chỉ kiểm tên command sẽ bỏ lọt lỗi này, và nó là lỗi lúc chạy.
   */
  it('mọi hàm dùng khoá camelCase repoId, không dùng repo_id', async () => {
    await ipc.getStatus('r')
    await ipc.stageFiles('r', ['a'])
    await ipc.unstageFiles('r', ['a'])
    await ipc.getWorktreeDiff('r', 'a', false)

    expect(invokeMock.mock.calls).toHaveLength(4)
    for (const [, args] of invokeMock.mock.calls) {
      const payload = args as Record<string, unknown>
      expect(Object.keys(payload)).toContain('repoId')
      expect(Object.keys(payload)).not.toContain('repo_id')
    }
  })

  /** Tên command bên Rust là snake_case; gửi camelCase sẽ "command not found". */
  it('tên command gửi xuống là snake_case', async () => {
    await ipc.getStatus('r')
    await ipc.stageFiles('r', ['a'])
    await ipc.unstageFiles('r', ['a'])
    await ipc.getWorktreeDiff('r', 'a', false)

    const names = invokeMock.mock.calls.map(([name]) => name)
    expect(names).toEqual([
      'get_status',
      'stage_files',
      'unstage_files',
      'get_worktree_diff',
    ])
    for (const n of names) {
      expect(n).not.toMatch(/[A-Z]/)
    }
  })

  /**
   * 🔴 Mảng `paths` đi qua **nguyên vẹn**, kể cả khi rỗng.
   *
   * Phía Rust chặn mảng rỗng (nó không sinh lệnh git nào — `git add` không pathspec
   * stage cả cây). Phía này **không** được tự ý bỏ lời gọi hay đổi mảng rỗng thành
   * một thứ khác: nếu nó im lặng không gọi thì store không nhận `RepoStatus` mới và
   * giao diện đứng im, mà không ai biết vì sao.
   */
  it('paths rỗng vẫn được gửi xuống nguyên vẹn', async () => {
    await ipc.stageFiles('r', [])

    expect(invokeMock).toHaveBeenCalledWith('stage_files', { repoId: 'r', paths: [] })
  })
})

describe('hình dạng dữ liệu trả về', () => {
  /** Một `RepoStatus` mẫu có đủ ba nhóm cộng một tệp `MM` ở hai nhóm. */
  function statusMau(): RepoStatus {
    const entries: StatusEntry[] = [
      {
        path: 'da_stage.txt',
        oldPath: null,
        xy: 'M.',
        group: 'staged',
        hasInvalidUtf8: false,
      },
      {
        path: 'ca_hai.txt',
        oldPath: null,
        xy: 'MM',
        group: 'staged',
        hasInvalidUtf8: false,
      },
      {
        path: 'ca_hai.txt',
        oldPath: null,
        xy: 'MM',
        group: 'unstaged',
        hasInvalidUtf8: false,
      },
      {
        path: 'moi.txt',
        oldPath: 'cu.txt',
        xy: 'R.',
        group: 'staged',
        hasInvalidUtf8: false,
      },
      {
        path: 'chua_theo_doi.txt',
        oldPath: null,
        xy: '??',
        group: 'untracked',
        hasInvalidUtf8: false,
      },
    ]
    return {
      branch: {
        head: 'main',
        oid: 'a'.repeat(40),
        upstream: 'origin/main',
        ahead: 3,
        behind: 0,
      },
      entries,
      hasConflicts: false,
    }
  }

  it('getStatus trả RepoStatus có ba nhóm và giữ oldPath của tệp đổi tên', async () => {
    invokeMock.mockResolvedValue(statusMau())

    const got = await ipc.getStatus('r')

    const nhom = new Set(got.entries.map((e) => e.group))
    expect(nhom).toEqual(new Set(['staged', 'unstaged', 'untracked']))
    expect(got.entries.find((e) => e.path === 'moi.txt')?.oldPath).toBe('cu.txt')
    expect(got.entries.find((e) => e.path === 'da_stage.txt')?.oldPath).toBeNull()
    expect(got.hasConflicts).toBe(false)
  })

  /**
   * Một tệp `MM` xuất hiện ở **cả hai** nhóm — `path` không phải khoá duy nhất.
   *
   * Ghim ở tầng kiểu để `ChangeList` không dựng khoá React chỉ từ `path`: trùng khoá
   * làm React cảnh báo và có thể tái dùng sai node giữa hai nhóm.
   */
  it('một tệp MM xuất hiện ở cả hai nhóm với cùng path', async () => {
    invokeMock.mockResolvedValue(statusMau())

    const got = await ipc.getStatus('r')
    const caHai = got.entries.filter((e) => e.path === 'ca_hai.txt')

    expect(caHai).toHaveLength(2)
    expect(new Set(caHai.map((e) => e.group))).toEqual(new Set(['staged', 'unstaged']))
  })

  /**
   * 🔴 `ahead`/`behind` là `null` khi chưa có upstream — **không** phải `0`.
   *
   * WORK-09 dùng đúng phân biệt này để không cảnh báo amend ở repo chưa từng push.
   * Kiểu TS phải cho phép `null`, nếu không phía Rust gửi `null` và TS nói dối rằng
   * nó là `number`.
   */
  it('ahead/behind là null khi chưa có upstream, khác hẳn 0', async () => {
    const chuaPush: RepoStatus = {
      branch: { head: 'main', oid: 'b'.repeat(40), upstream: null, ahead: null, behind: null },
      entries: [],
      hasConflicts: false,
    }
    invokeMock.mockResolvedValue(chuaPush)

    const got = await ipc.getStatus('r')

    expect(got.branch.ahead).toBeNull()
    expect(got.branch.ahead).not.toBe(0)
    expect(got.branch.upstream).toBeNull()
  })

  /** HEAD tách rời → `head` là `null`, không phải chuỗi `(detached)`. */
  it('HEAD tách rời cho head null', async () => {
    invokeMock.mockResolvedValue({
      branch: { head: null, oid: 'c'.repeat(40), upstream: null, ahead: null, behind: null },
      entries: [],
      hasConflicts: false,
    } satisfies RepoStatus)

    const got = await ipc.getStatus('r')
    expect(got.branch.head).toBeNull()
  })

  /**
   * 🔴 `stageFiles` trả `RepoStatus`, **không** `void` — đột biến M1 ở tầng TypeScript.
   *
   * Ràng buộc 2.5 của CONTEXT.md. Một lệnh ghi trả `void` buộc giao diện chờ watcher.
   */
  it('stageFiles trả RepoStatus mới, không trả void', async () => {
    const sau = statusMau()
    invokeMock.mockResolvedValue(sau)

    const got = await ipc.stageFiles('r', ['chua_stage.txt'])

    expect(got).toBeDefined()
    expect(got.entries).toHaveLength(sau.entries.length)
    expect(got.branch.head).toBe('main')
  })

  it('unstageFiles trả RepoStatus mới', async () => {
    invokeMock.mockResolvedValue(statusMau())

    const got = await ipc.unstageFiles('r', ['da_stage.txt'])

    expect(got.entries.length).toBeGreaterThan(0)
  })

  it('getWorktreeDiff trả FileDiff với discriminated union theo kind', async () => {
    const fd: FileDiff = {
      path: 'a.txt',
      oldPath: null,
      status: 'M',
      kind: {
        kind: 'text',
        hunks: [
          {
            oldStart: 1,
            oldCount: 3,
            newStart: 1,
            newCount: 3,
            heading: '',
            lines: [
              {
                kind: 'removed',
                content: 'cu',
                oldLine: 2,
                newLine: null,
                noNewlineAtEof: false,
                spans: [],
              },
              {
                kind: 'added',
                content: 'moi',
                oldLine: null,
                newLine: 2,
                noNewlineAtEof: false,
                spans: [],
              },
            ],
          },
        ],
        truncated: false,
        contextOnly: false,
      },
    }
    invokeMock.mockResolvedValue(fd)

    const got = await ipc.getWorktreeDiff('r', 'a.txt', false)

    expect(got.kind.kind).toBe('text')
    if (got.kind.kind === 'text') {
      expect(got.kind.hunks[0]!.lines).toHaveLength(2)
      expect(got.kind.contextOnly).toBe(false)
    }
  })
})

describe('mã lỗi index_locked', () => {
  /**
   * 🔴 `index_locked` là một `GitErrorCode` **hợp lệ và riêng biệt**.
   *
   * Gộp nó vào `command_failed` làm giao diện nói sai: đây không phải lỗi mà là trạng
   * thái tạm thời — người dùng đang chạy git ở terminal (WORK-10 tồn tại chính vì
   * điều đó), và câu đúng là "đang chờ một tiến trình git khác".
   */
  it('index_locked là một mã lỗi riêng, không phải command_failed', () => {
    const loi = {
      code: 'index_locked' as const,
      message: 'Không ghi được vào repository: một tiến trình git khác đang giữ .git/index.lock.',
      command: 'git add -- a.txt',
    }

    // Phép gán này chỉ biên dịch nếu `index_locked` nằm trong union `GitErrorCode`.
    const payload: import('./ipc').GitErrorPayload = loi
    expect(payload.code).toBe('index_locked')
    expect(payload.code).not.toBe('command_failed')
    expect(payload.message).toContain('index.lock')
  })
})
