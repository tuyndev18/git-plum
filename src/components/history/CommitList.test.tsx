/**
 * Test `CommitList` — trái tim của tiêu chí thành công số 2 (đồ thị luôn thẳng
 * hàng với văn bản). `@testing-library/react` vì đây thật sự render.
 *
 * `happy-dom` không cài `getContext('2d')` cho canvas thật, nên
 * `GraphCanvas`/`createCanvasRenderer` sẽ ném khi chạy trong JSDOM/happy-dom
 * trừ khi được vá cục bộ ở đây — vá đúng trong tệp test này, không toàn cục.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

import { CommitList } from '@/components/history/CommitList'
import { useHistoryStore } from '@/stores/historyStore'
import type { Commit, GraphRow } from '@/lib/ipc'

vi.mock('@/lib/ipc', () => ({
  ipc: {
    getCommitPage: vi.fn().mockResolvedValue({
      commits: [],
      graphRows: [],
      total: 0,
      skippedRecords: 0,
    }),
  },
}))

function commit(id: string, subject: string, overrides: Partial<Commit> = {}): Commit {
  return {
    id,
    parents: [],
    authorName: 'Nguyen Van A',
    authorEmail: 'a@x.com',
    authorTime: 1700000000,
    committerName: 'Nguyen Van A',
    committerEmail: 'a@x.com',
    committerTime: 1700000000,
    subject,
    body: '',
    hasInvalidUtf8: false,
    ...overrides,
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

// `HTMLCanvasElement.getContext` không có bản cài thật trong happy-dom. Vá cục
// bộ trong tệp test này (không phải `src/test/setup.ts` toàn cục) để
// `createCanvasRenderer` chạy được mà không ném lúc test render CommitList.
function stubCanvasContext() {
  const fakeCtx = {
    save: vi.fn(),
    restore: vi.fn(),
    scale: vi.fn(),
    clearRect: vi.fn(),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    bezierCurveTo: vi.fn(),
    arc: vi.fn(),
    stroke: vi.fn(),
    fill: vi.fn(),
    fillText: vi.fn(),
    setLineDash: vi.fn(),
    strokeStyle: '',
    fillStyle: '',
    lineWidth: 1,
  }
  vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
    fakeCtx as unknown as CanvasRenderingContext2D,
  )
}

/**
 * happy-dom không tính layout thật — `@tanstack/react-virtual` đo viewport
 * bằng `element.offsetWidth`/`offsetHeight` (xem
 * `virtual-core/dist/esm/index.js` hàm `getRect`), và happy-dom luôn trả 0
 * cho cả hai. Không vá thì hook ảo hoá nghĩ viewport cao 0px và không render
 * hàng nào — đây là cách làm chuẩn khi test thư viện này ngoài trình duyệt
 * thật, không phải việc né tránh lỗi thật.
 */
function stubViewportSize() {
  Object.defineProperty(HTMLElement.prototype, 'offsetHeight', {
    configurable: true,
    value: 600,
  })
  Object.defineProperty(HTMLElement.prototype, 'offsetWidth', {
    configurable: true,
    value: 800,
  })
  Object.defineProperty(HTMLElement.prototype, 'clientHeight', {
    configurable: true,
    value: 600,
  })
  Object.defineProperty(HTMLElement.prototype, 'clientWidth', {
    configurable: true,
    value: 800,
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  useHistoryStore.setState({ byRepo: {} })
  stubCanvasContext()
  stubViewportSize()
})

function seedRepo(
  repoId: string,
  commits: Commit[],
  graphRows: GraphRow[],
  total = commits.length,
) {
  useHistoryStore.setState((s) => ({
    byRepo: {
      ...s.byRepo,
      [repoId]: {
        commits,
        graphRows,
        total,
        loadedRanges: [[0, commits.length]],
        isLoading: false,
        error: null,
      },
    },
  }))
}

describe('CommitList', () => {
  it('render với 3 commit giả hiện đủ ba subject', () => {
    const commits = [commit('a', 'sua loi A'), commit('b', 'them tinh nang B'), commit('c', 'don dep C')]
    const rows = commits.map((c) => graphRow(c.id))
    seedRepo('repo-1', commits, rows)

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(screen.getByText('sua loi A')).toBeTruthy()
    expect(screen.getByText('them tinh nang B')).toBeTruthy()
    expect(screen.getByText('don dep C')).toBeTruthy()
  })

  it('có đúng một phần tử cuộn data-testid="commit-scroll"', () => {
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])

    const { container } = render(
      <CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />,
    )

    expect(container.querySelectorAll('[data-testid="commit-scroll"]')).toHaveLength(1)
  })

  it('bấm một hàng gọi onSelect với đúng commitId của hàng đó', () => {
    seedRepo('repo-1', [commit('a', 'hang mot'), commit('b', 'hang hai')], [
      graphRow('a'),
      graphRow('b'),
    ])
    const onSelect = vi.fn()

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={onSelect} />)
    screen.getByText('hang hai').click()

    expect(onSelect).toHaveBeenCalledWith('b')
  })

  it('commit hasInvalidUtf8 vẫn render một hàng, không bị lọc bỏ', () => {
    const commits = [commit('a', 'binh thuong'), commit('b', 'loi utf8', { hasInvalidUtf8: true })]
    seedRepo('repo-1', commits, commits.map((c) => graphRow(c.id)))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(screen.getByText('binh thuong')).toBeTruthy()
    expect(screen.getByText('loi utf8')).toBeTruthy()
  })

  it('total lớn hơn số commit đã nạp vẫn render, không lỗi', () => {
    seedRepo('repo-1', [commit('a', 'trang dau')], [graphRow('a')], 100000)

    expect(() =>
      render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />),
    ).not.toThrow()

    expect(screen.getByText('trang dau')).toBeTruthy()
  })
})
