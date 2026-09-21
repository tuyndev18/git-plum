/**
 * Test ranh giới IPC cho các command lịch sử (Phase 2).
 *
 * Các test ở đây không kiểm logic — chúng khoá lại **hợp đồng gọi**: tên command và
 * hình dạng đối số. Cả hai thứ đó chỉ thất bại **lúc chạy**: gõ sai `get_commit_page`
 * thành `getCommitPage`, hay truyền `repo_id` thay cho `repoId`, đều biên dịch sạch ở
 * cả TypeScript lẫn Rust rồi ném "command not found" trên máy người dùng.
 *
 * Tauri v2 chuyển tham số `snake_case` của Rust thành `camelCase` ở phía JS. Đó là lý
 * do khoá đối số là `repoId` chứ không phải `repo_id`.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'
import {
  ipc,
  type CommitDetail,
  type CommitPage,
  type FileChange,
  type GitRef,
  type GraphRow,
} from './ipc'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

beforeEach(() => {
  // `vite.config.ts` đặt `restoreMocks: true`, nên mọi `mockResolvedValue` đặt trong
  // factory của `vi.mock` bị gỡ giữa các test và hàm giả sẽ trả `undefined`. Dựng lại
  // giá trị mặc định ở đây là **bắt buộc**, không phải cẩn thận thừa.
  invokeMock.mockReset()
  invokeMock.mockResolvedValue(undefined)
})

describe('tên command và hình dạng đối số', () => {
  it('getCommitPage gọi get_commit_page với { repoId, skip, limit }', async () => {
    await ipc.getCommitPage('repo-1', 100, 50)

    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(invokeMock).toHaveBeenCalledWith('get_commit_page', {
      repoId: 'repo-1',
      skip: 100,
      limit: 50,
    })
  })

  it('listRefs gọi list_refs với { repoId }', async () => {
    await ipc.listRefs('repo-1')

    expect(invokeMock).toHaveBeenCalledWith('list_refs', { repoId: 'repo-1' })
  })

  it('getCommitDetail gọi get_commit_detail với { repoId, commitId }', async () => {
    await ipc.getCommitDetail('repo-1', 'abc123')

    expect(invokeMock).toHaveBeenCalledWith('get_commit_detail', {
      repoId: 'repo-1',
      commitId: 'abc123',
    })
  })

  it('searchCommits gọi search_commits với { repoId, query }', async () => {
    await ipc.searchCommits('repo-1', 'sửa lỗi')

    expect(invokeMock).toHaveBeenCalledWith('search_commits', {
      repoId: 'repo-1',
      query: 'sửa lỗi',
    })
  })

  /**
   * Khoá đối số phải là `repoId` (camelCase), **không** `repo_id`. Một test chỉ kiểm
   * tên command sẽ bỏ lọt lỗi này, và nó là lỗi lúc chạy.
   */
  it('mọi hàm dùng khoá camelCase repoId, không dùng repo_id', async () => {
    await ipc.getCommitPage('r', 0, 1)
    await ipc.listRefs('r')
    await ipc.getCommitDetail('r', 'c')
    await ipc.searchCommits('r', 'q')

    for (const [, args] of invokeMock.mock.calls) {
      const payload = args as Record<string, unknown>
      expect(Object.keys(payload)).toContain('repoId')
      expect(Object.keys(payload)).not.toContain('repo_id')
    }
  })

  /** Tên command bên Rust là snake_case; gửi camelCase sẽ "command not found". */
  it('tên command gửi xuống là snake_case', async () => {
    await ipc.getCommitPage('r', 0, 1)
    await ipc.listRefs('r')
    await ipc.getCommitDetail('r', 'c')
    await ipc.searchCommits('r', 'q')

    const names = invokeMock.mock.calls.map(([name]) => name)
    expect(names).toEqual([
      'get_commit_page',
      'list_refs',
      'get_commit_detail',
      'search_commits',
    ])
    for (const n of names) {
      expect(n).not.toMatch(/[A-Z]/)
    }
  })
})

describe('hình dạng dữ liệu trả về', () => {
  it('getCommitPage trả CommitPage với graphRows cùng độ dài commits', async () => {
    const page: CommitPage = {
      commits: [
        {
          id: 'a'.repeat(40),
          parents: [],
          authorName: 'A',
          authorEmail: 'a@b.c',
          authorTime: 1_700_000_000,
          committerName: 'A',
          committerEmail: 'a@b.c',
          committerTime: 1_700_000_000,
          subject: 'commit đầu',
          body: '',
          hasInvalidUtf8: false,
        },
      ],
      graphRows: [
        {
          commitId: 'a'.repeat(40),
          lane: 0,
          color: 0,
          passthrough: [],
          outEdges: [],
          truncatedParents: 0,
          terminates: false,
        },
      ],
      total: 1,
      skippedRecords: 0,
    }
    invokeMock.mockResolvedValue(page)

    const got = await ipc.getCommitPage('r', 0, 10)

    expect(got.commits).toHaveLength(got.graphRows.length)
    expect(got.graphRows[0]!.commitId).toBe(got.commits[0]!.id)
    expect(got.total).toBe(1)
  })

  /**
   * `lane` có thể vượt giới hạn hiển thị 20 — đó là hành vi **đúng** mà plan 02-03
   * chốt: giới hạn chỉ áp cho lane của cha. Kiểu TS không được cấm điều đó.
   */
  it('GraphRow chấp nhận lane vượt giới hạn hiển thị', () => {
    const row: GraphRow = {
      commitId: 'b'.repeat(40),
      lane: 24,
      color: 3,
      passthrough: [{ fromLane: 20, toLane: 20, color: 6 }],
      outEdges: [{ fromLane: 24, toLane: 24, color: 3 }],
      truncatedParents: 2,
      terminates: true,
    }
    expect(row.lane).toBeGreaterThan(20)
    expect(row.passthrough[0]!.fromLane).toBe(20)
  })

  it('GitRef phân biệt bốn loại và giữ upstream khi [gone]', async () => {
    const refs: GitRef[] = [
      {
        fullName: 'refs/heads/main',
        shortName: 'main',
        kind: 'localBranch',
        target: 'c'.repeat(40),
        upstream: 'refs/remotes/origin/main',
        ahead: 3,
        behind: 1,
        isHead: true,
      },
      {
        fullName: 'refs/remotes/origin/main',
        shortName: 'origin/main',
        kind: 'remoteBranch',
        target: 'c'.repeat(40),
        upstream: null,
        ahead: 0,
        behind: 0,
        isHead: false,
      },
      {
        fullName: 'refs/tags/v1.0',
        shortName: 'v1.0',
        kind: 'tag',
        target: 'c'.repeat(40),
        upstream: null,
        ahead: 0,
        behind: 0,
        isHead: false,
      },
      {
        fullName: 'refs/stash',
        shortName: 'refs/stash',
        kind: 'other',
        target: 'd'.repeat(40),
        upstream: null,
        ahead: 0,
        behind: 0,
        isHead: false,
      },
    ]
    invokeMock.mockResolvedValue(refs)

    const got = await ipc.listRefs('r')

    expect(got.map((r) => r.kind)).toEqual([
      'localBranch',
      'remoteBranch',
      'tag',
      'other',
    ])
    // Chỉ một ref là HEAD; với HEAD tách rời thì không ref nào cả.
    expect(got.filter((r) => r.isHead)).toHaveLength(1)
    // `upstream` là `null` chứ không phải chuỗi rỗng — frontend phân biệt được.
    expect(got[1]!.upstream).toBeNull()
  })

  it('CommitDetail mang files và cờ truncated', async () => {
    const doiTen: FileChange = {
      status: 'R100',
      path: 'moi.txt',
      oldPath: 'cu.txt',
    }
    const sua: FileChange = { status: 'M', path: 'khac.txt', oldPath: null }

    const detail: CommitDetail = {
      commit: {
        id: 'e'.repeat(40),
        parents: ['f'.repeat(40)],
        authorName: 'A',
        authorEmail: 'a@b.c',
        authorTime: 1,
        committerName: 'A',
        committerEmail: 'a@b.c',
        committerTime: 1,
        subject: 's',
        body: 'b',
        hasInvalidUtf8: false,
      },
      files: [doiTen, sua],
      truncated: true,
    }
    invokeMock.mockResolvedValue(detail)

    const got = await ipc.getCommitDetail('r', 'e'.repeat(40))

    expect(got.files).toHaveLength(2)
    expect(got.files[0]!.oldPath).toBe('cu.txt')
    expect(got.files[1]!.oldPath).toBeNull()
    expect(got.truncated).toBe(true)
  })

  it('searchCommits trả mảng mã commit, không trả Commit đầy đủ', async () => {
    invokeMock.mockResolvedValue(['a'.repeat(40), 'b'.repeat(40)])

    const got = await ipc.searchCommits('r', 'lỗi')

    expect(got).toHaveLength(2)
    for (const id of got) {
      expect(typeof id).toBe('string')
      expect(id).toHaveLength(40)
    }
  })
})
