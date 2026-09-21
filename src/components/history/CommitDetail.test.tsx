/**
 * Test `CommitDetail` — HIST-08. Giả lập `@/lib/ipc` ở ranh giới module.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'

import { CommitDetail } from '@/components/history/CommitDetail'
import { ipc, type Commit, type CommitDetail as CommitDetailPayload } from '@/lib/ipc'
import { useHistoryStore } from '@/stores/historyStore'
import { useSelectionStore } from '@/stores/selectionStore'

vi.mock('@/lib/ipc', async () => {
  const actual = await vi.importActual<typeof import('@/lib/ipc')>('@/lib/ipc')
  return {
    ...actual,
    ipc: {
      getCommitDetail: vi.fn(),
    },
  }
})

const getCommitDetail = vi.mocked(ipc.getCommitDetail)

function commit(id: string, overrides: Partial<Commit> = {}): Commit {
  return {
    id,
    parents: [],
    authorName: 'Nguyen Van A',
    authorEmail: 'a@x.com',
    authorTime: 1700000000,
    committerName: 'Nguyen Van A',
    committerEmail: 'a@x.com',
    committerTime: 1700000000,
    subject: 'feat: something',
    body: 'dong 1\ndong 2\ndong 3',
    hasInvalidUtf8: false,
    ...overrides,
  }
}

function detail(c: Commit, overrides: Partial<CommitDetailPayload> = {}): CommitDetailPayload {
  return { commit: c, files: [], truncated: false, ...overrides }
}

const REPO_ID = 'repo-1'

beforeEach(() => {
  vi.clearAllMocks()
  useHistoryStore.setState({ byRepo: {} })
  useSelectionStore.setState({ selectedByRepo: {} })
})

describe('selectedCommitId null', () => {
  it('hiện trạng thái rỗng đọc được, không lỗi', () => {
    useSelectionStore.getState().select(REPO_ID, null as unknown as string)
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: null } })

    render(<CommitDetail repoId={REPO_ID} />)

    expect(screen.getByText(/chọn một commit/i)).toBeInTheDocument()
  })
})

describe('có id -> hiện metadata', () => {
  it('hiện subject, TOÀN BỘ body, authorName, authorEmail, thời gian, mã commit đầy đủ', async () => {
    const c = commit('abc123def456', { body: 'dong 1\ndong 2\ndong 3' })
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c],
          graphRows: [],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c.id } })
    getCommitDetail.mockResolvedValueOnce(detail(c))

    render(<CommitDetail repoId={REPO_ID} />)

    await waitFor(() => expect(getCommitDetail).toHaveBeenCalled())

    expect(screen.getByText('feat: something')).toBeInTheDocument()
    expect(screen.getByText(/dong 1/)).toBeInTheDocument()
    expect(screen.getByText(/dong 2/)).toBeInTheDocument()
    expect(screen.getByText(/dong 3/)).toBeInTheDocument()
    expect(screen.getByText('Nguyen Van A')).toBeInTheDocument()
    expect(screen.getByText('a@x.com')).toBeInTheDocument()
    expect(screen.getByText('abc123def456')).toBeInTheDocument()
  })

  it('commit hai cha -> hiện cả hai mã cha', async () => {
    const c = commit('c1', { parents: ['p1', 'p2'] })
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c],
          graphRows: [],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c.id } })
    getCommitDetail.mockResolvedValueOnce(detail(c))

    render(<CommitDetail repoId={REPO_ID} />)
    await waitFor(() => expect(getCommitDetail).toHaveBeenCalled())

    expect(screen.getByText('p1')).toBeInTheDocument()
    expect(screen.getByText('p2')).toBeInTheDocument()
  })

  it('commit BỐN cha (octopus) -> hiện cả bốn mã cha', async () => {
    const c = commit('c1', { parents: ['p1', 'p2', 'p3', 'p4'] })
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c],
          graphRows: [],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c.id } })
    getCommitDetail.mockResolvedValueOnce(detail(c))

    render(<CommitDetail repoId={REPO_ID} />)
    await waitFor(() => expect(getCommitDetail).toHaveBeenCalled())

    expect(screen.getByText('p1')).toBeInTheDocument()
    expect(screen.getByText('p2')).toBeInTheDocument()
    expect(screen.getByText('p3')).toBeInTheDocument()
    expect(screen.getByText('p4')).toBeInTheDocument()
  })

  it('commit gốc (parents rỗng) -> hiện nhãn "commit gốc", không danh sách cha rỗng', async () => {
    const c = commit('c1', { parents: [] })
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c],
          graphRows: [],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c.id } })
    getCommitDetail.mockResolvedValueOnce(detail(c))

    render(<CommitDetail repoId={REPO_ID} />)
    await waitFor(() => expect(getCommitDetail).toHaveBeenCalled())

    expect(screen.getByText(/commit gốc/i)).toBeInTheDocument()
  })

  it('getCommitDetail ném lỗi -> hiện thông báo qua describeError, không trắng vùng', async () => {
    const c = commit('c1')
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c],
          graphRows: [],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c.id } })
    getCommitDetail.mockRejectedValueOnce(new Error('boom'))

    render(<CommitDetail repoId={REPO_ID} />)

    await waitFor(() => expect(screen.getByText(/boom/)).toBeInTheDocument())
  })

  it('truncated === true -> hiện cảnh báo danh sách tệp bị cắt kèm số', async () => {
    const c = commit('c1')
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c],
          graphRows: [],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c.id } })
    getCommitDetail.mockResolvedValueOnce(
      detail(c, {
        truncated: true,
        files: [{ status: 'M', path: 'a.ts', oldPath: null }],
      }),
    )

    render(<CommitDetail repoId={REPO_ID} />)

    await waitFor(() => expect(screen.getByText(/cắt/i)).toBeInTheDocument())
  })

  it('đổi selectedCommitId -> gọi getCommitDetail cho id mới; chọn lại CÙNG id -> KHÔNG gọi lại', async () => {
    const c1 = commit('c1')
    const c2 = commit('c2')
    useHistoryStore.setState({
      byRepo: {
        [REPO_ID]: {
          commits: [c1, c2],
          graphRows: [],
          total: 2,
          loadedRanges: [[0, 2]],
          isLoading: false,
          error: null,
        },
      },
    })
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c1.id } })
    getCommitDetail.mockResolvedValue(detail(c1))

    const { rerender } = render(<CommitDetail repoId={REPO_ID} />)
    await waitFor(() => expect(getCommitDetail).toHaveBeenCalledTimes(1))

    // Chọn lại cùng id -> không gọi lại IPC (cache theo commitId).
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c1.id } })
    rerender(<CommitDetail repoId={REPO_ID} />)
    await new Promise((r) => setTimeout(r, 10))
    expect(getCommitDetail).toHaveBeenCalledTimes(1)

    // Đổi sang id khác -> gọi lại.
    getCommitDetail.mockResolvedValueOnce(detail(c2))
    useSelectionStore.setState({ selectedByRepo: { [REPO_ID]: c2.id } })
    rerender(<CommitDetail repoId={REPO_ID} />)
    await waitFor(() => expect(getCommitDetail).toHaveBeenCalledTimes(2))
  })
})
