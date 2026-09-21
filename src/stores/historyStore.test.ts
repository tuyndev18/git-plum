/**
 * Test `historyStore` — khuôn `byRepo[repoId]` giống `repoStore.ts` (PLAT-05).
 *
 * Chặn ở ranh giới module `@/lib/ipc`, không vá `invoke` toàn cục — cùng quy
 * ước với `repoStore.test.ts`.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'

import { ipc, type Commit, type CommitPage, type GraphRow } from '@/lib/ipc'
import { PAGE_SIZE, useHistoryStore } from '@/stores/historyStore'

vi.mock('@/lib/ipc', () => ({
  ipc: {
    getCommitPage: vi.fn(),
  },
}))

const getCommitPage = vi.mocked(ipc.getCommitPage)

function commit(id: string): Commit {
  return {
    id,
    parents: [],
    authorName: 'a',
    authorEmail: 'a@x.com',
    authorTime: 0,
    committerName: 'a',
    committerEmail: 'a@x.com',
    committerTime: 0,
    subject: id,
    body: '',
    hasInvalidUtf8: false,
  }
}

function graphRow(id: string): GraphRow {
  return {
    commitId: id,
    lane: 0,
    color: 0,
    passthrough: [],
    outEdges: [],
    truncatedParents: 0,
    terminates: false,
  }
}

function page(ids: string[], total: number): CommitPage {
  return {
    commits: ids.map(commit),
    graphRows: ids.map(graphRow),
    total,
    skippedRecords: 0,
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  useHistoryStore.setState({ byRepo: {} })
})

describe('loadFirstPage', () => {
  it('nạp commits, graphRows, total vào byRepo[repoId]', async () => {
    // `total` lớn hơn trang đã nạp: mảng đích được cấp trước tới `total` để
    // virtualizer đọc theo chỉ số tuyệt đối; vị trí chưa nạp giữ `undefined` —
    // đúng ngữ nghĩa mà `CommitList` cần để quyết định gọi `ensureRange`.
    getCommitPage.mockResolvedValueOnce(page(['a', 'b'], 500))

    await useHistoryStore.getState().loadFirstPage('repo-1')

    const slice = useHistoryStore.getState().byRepo['repo-1']
    expect(slice).toBeDefined()
    expect(slice?.commits[0]?.id).toBe('a')
    expect(slice?.commits[1]?.id).toBe('b')
    expect(slice?.commits.length).toBe(500)
    expect(slice?.commits[2]).toBeUndefined()
    expect(slice?.graphRows[0]?.commitId).toBe('a')
    expect(slice?.graphRows[1]?.commitId).toBe('b')
    expect(slice?.total).toBe(500)
    expect(slice?.isLoading).toBe(false)
    expect(slice?.error).toBeNull()
  })

  it('gọi getCommitPage với skip=0 và PAGE_SIZE', async () => {
    getCommitPage.mockResolvedValueOnce(page(['a'], 1))

    await useHistoryStore.getState().loadFirstPage('repo-1')

    expect(getCommitPage).toHaveBeenCalledWith('repo-1', 0, PAGE_SIZE)
  })
})

describe('cách li theo repoId (PLAT-05)', () => {
  it('dữ liệu của hai repoId khác nhau không trộn vào nhau', async () => {
    getCommitPage.mockResolvedValueOnce(page(['a1', 'a2'], 2))
    await useHistoryStore.getState().loadFirstPage('repo-a')

    getCommitPage.mockResolvedValueOnce(page(['b1'], 1))
    await useHistoryStore.getState().loadFirstPage('repo-b')

    const a = useHistoryStore.getState().byRepo['repo-a']
    const b = useHistoryStore.getState().byRepo['repo-b']
    expect(a?.commits.map((c) => c.id)).toEqual(['a1', 'a2'])
    expect(b?.commits.map((c) => c.id)).toEqual(['b1'])
  })
})

describe('xử lý lỗi', () => {
  it('ipc.getCommitPage ném lỗi -> đặt error, isLoading về false, store không ném ra ngoài', async () => {
    getCommitPage.mockRejectedValueOnce(new Error('boom'))

    await expect(useHistoryStore.getState().loadFirstPage('repo-1')).resolves.not.toThrow()

    const slice = useHistoryStore.getState().byRepo['repo-1']
    expect(slice?.error).toBeTruthy()
    expect(slice?.isLoading).toBe(false)
  })

  it('commits.length !== graphRows.length -> ghi cảnh báo, không crash', async () => {
    const malformed: CommitPage = {
      commits: [commit('a'), commit('b')],
      graphRows: [graphRow('a')],
      total: 2,
      skippedRecords: 0,
    }
    getCommitPage.mockResolvedValueOnce(malformed)
    const warnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

    await expect(useHistoryStore.getState().loadFirstPage('repo-1')).resolves.not.toThrow()

    expect(warnSpy).toHaveBeenCalled()
    warnSpy.mockRestore()
  })
})

describe('ensureRange', () => {
  it('gọi getCommitPage một lần cho một dải chưa có', async () => {
    getCommitPage.mockResolvedValueOnce(page(['a'], 1))
    // Nạp trang đầu để có slice tồn tại trước.
    await useHistoryStore.getState().loadFirstPage('repo-1')
    getCommitPage.mockClear()

    getCommitPage.mockResolvedValueOnce(page(Array.from({ length: 50 }, (_, i) => `x${i}`), 5000))
    await useHistoryStore.getState().ensureRange('repo-1', 2000, 2050)

    expect(getCommitPage).toHaveBeenCalledTimes(1)
  })

  it('không gọi lại getCommitPage cho cùng dải đã nạp (chống bão IPC)', async () => {
    getCommitPage.mockResolvedValueOnce(page(['a'], 1))
    await useHistoryStore.getState().loadFirstPage('repo-1')
    getCommitPage.mockClear()

    getCommitPage.mockResolvedValueOnce(page(Array.from({ length: 50 }, (_, i) => `x${i}`), 5000))
    await useHistoryStore.getState().ensureRange('repo-1', 2000, 2050)
    await useHistoryStore.getState().ensureRange('repo-1', 2000, 2050)
    await useHistoryStore.getState().ensureRange('repo-1', 2010, 2040)

    expect(getCommitPage).toHaveBeenCalledTimes(1)
  })
})

describe('reset', () => {
  it('xoá dữ liệu của repoId khỏi byRepo', async () => {
    getCommitPage.mockResolvedValueOnce(page(['a'], 1))
    await useHistoryStore.getState().loadFirstPage('repo-1')
    expect(useHistoryStore.getState().byRepo['repo-1']).toBeDefined()

    useHistoryStore.getState().reset('repo-1')

    expect(useHistoryStore.getState().byRepo['repo-1']).toBeUndefined()
  })
})
