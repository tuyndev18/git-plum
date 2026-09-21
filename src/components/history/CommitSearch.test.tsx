/**
 * Test `CommitSearch` — HIST-10, tìm kiếm commit theo bốn trục có trì hoãn.
 *
 * Trì hoãn 250ms qua `vi.useFakeTimers` — ARCHITECTURE.md Anti-Pattern 4 cấm
 * chạy git mỗi lần gõ.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { act, render, screen } from '@testing-library/react'

import { CommitSearch } from '@/components/history/CommitSearch'
import { ipc, type Commit } from '@/lib/ipc'
import { useHistoryStore } from '@/stores/historyStore'

vi.mock('@/lib/ipc', async () => {
  const actual = await vi.importActual<typeof import('@/lib/ipc')>('@/lib/ipc')
  return {
    ...actual,
    ipc: {
      searchCommits: vi.fn(),
    },
  }
})

const searchCommits = vi.mocked(ipc.searchCommits)

const REPO_ID = 'repo-1'

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

function seedRepo(commits: Commit[]) {
  useHistoryStore.setState({
    byRepo: {
      [REPO_ID]: {
        commits,
        graphRows: [],
        total: commits.length,
        loadedRanges: [[0, commits.length]],
        isLoading: false,
        error: null,
      },
    },
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  searchCommits.mockResolvedValue([])
  vi.useFakeTimers()
  useHistoryStore.setState({ byRepo: {} })
})

afterEach(() => {
  vi.useRealTimers()
})

function typeInto(input: HTMLElement, value: string) {
  // happy-dom + fake timers: gán value trực tiếp rồi bắn sự kiện input, tránh
  // phụ thuộc userEvent (chưa cài trong dự án).
  Object.defineProperty(input, 'value', { value, writable: true, configurable: true })
  input.dispatchEvent(new Event('input', { bubbles: true }))
}

describe('trì hoãn 250ms', () => {
  it('gõ vào hộp -> KHÔNG gọi searchCommits ngay; chỉ gọi sau khi ngừng gõ', async () => {
    seedRepo([commit('a')])
    render(<CommitSearch repoId={REPO_ID} scrollToIndex={vi.fn()} />)

    const input = screen.getByRole('textbox')
    act(() => typeInto(input, 'tu khoa'))

    expect(searchCommits).not.toHaveBeenCalled()

    await act(() => vi.advanceTimersByTimeAsync(250))

    expect(searchCommits).toHaveBeenCalledWith(REPO_ID, 'tu khoa')
  })

  it('xoá hộp về rỗng -> xoá kết quả, không gọi IPC', async () => {
    seedRepo([commit('a')])
    render(<CommitSearch repoId={REPO_ID} scrollToIndex={vi.fn()} />)

    const input = screen.getByRole('textbox')
    act(() => typeInto(input, 'x'))
    await act(() => vi.advanceTimersByTimeAsync(250))
    searchCommits.mockClear()

    act(() => typeInto(input, ''))
    await act(() => vi.advanceTimersByTimeAsync(250))

    expect(searchCommits).not.toHaveBeenCalled()
  })
})

describe('kết quả', () => {
  it('kết quả có mã commit -> hiện số lượng khớp và cho nhảy tới khớp kế/trước', async () => {
    seedRepo([commit('a'), commit('b')])
    searchCommits.mockResolvedValueOnce(['a', 'b'])
    render(<CommitSearch repoId={REPO_ID} scrollToIndex={vi.fn()} />)

    act(() => typeInto(screen.getByRole('textbox'), 'x'))
    await act(() => vi.advanceTimersByTimeAsync(250))

    expect(screen.getByText(/1\s*\/\s*2/)).toBeTruthy()
  })

  it('không khớp -> thông báo "không tìm thấy", không bảng rỗng im lặng', async () => {
    seedRepo([commit('a')])
    searchCommits.mockResolvedValueOnce([])
    render(<CommitSearch repoId={REPO_ID} scrollToIndex={vi.fn()} />)

    act(() => typeInto(screen.getByRole('textbox'), 'khong ton tai'))
    await act(() => vi.advanceTimersByTimeAsync(250))

    expect(screen.getByText(/không tìm thấy/i)).toBeTruthy()
  })

  it('searchCommits ném lỗi -> hiện lỗi, hộp vẫn gõ được', async () => {
    seedRepo([commit('a')])
    searchCommits.mockRejectedValueOnce(new Error('boom'))
    render(<CommitSearch repoId={REPO_ID} scrollToIndex={vi.fn()} />)

    act(() => typeInto(screen.getByRole('textbox'), 'x'))
    await act(() => vi.advanceTimersByTimeAsync(250))

    expect(screen.getByText(/boom/)).toBeTruthy()

    const input = screen.getByRole('textbox') as HTMLInputElement
    expect(input.disabled).toBe(false)
  })

  it('nhảy tới một khớp -> gọi scrollToIndex với đúng chỉ số hàng của mã đó', async () => {
    seedRepo([commit('a'), commit('b'), commit('c')])
    searchCommits.mockResolvedValueOnce(['c'])
    const scrollToIndex = vi.fn()
    render(<CommitSearch repoId={REPO_ID} scrollToIndex={scrollToIndex} />)

    act(() => typeInto(screen.getByRole('textbox'), 'x'))
    await act(() => vi.advanceTimersByTimeAsync(250))

    expect(screen.getByText(/1\s*\/\s*1/)).toBeTruthy()
    expect(scrollToIndex).toHaveBeenCalledWith(2)
  })
})
