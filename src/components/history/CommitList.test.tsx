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

  // --- Checkpoint round 1 REJECTED — test bổ sung sau khi người dùng phát
  // hiện lỗi bằng mắt trên repo thật mà 103 test cũ không bắt được ---

  it('30 commit có subject tiếng Việt khác nhau: mỗi hàng hiện ĐÚNG subject của chính nó, không hàng nào lặp hay rớt xuống "."', () => {
    // Đây chính là kiểu dữ liệu thật gây lỗi quan sát được: nhiều hàng, subject
    // dài, có dấu tiếng Việt, một số commit thật sự có message ngắn/đặc biệt
    // (dấu "." là message hợp lệ, xem 02-05-SUMMARY.md "checkpoint round 1").
    // Test này khẳng định KHÔNG có sự trộn lẫn/rớt dữ liệu giữa các hàng khi
    // danh sách đủ dài để vượt viewport ảo hoá.
    const subjects = [
      'feat(clb): kết bạn theo friendRelation',
      "Merge branch 'feat/update_pool'",
      'fix: sửa lỗi đăng nhập',
      '.', // message hợp lệ thật sự chỉ có một dấu chấm — không phải lỗi hiển thị
      'update',
      ...Array.from({ length: 25 }, (_, i) => `commit số ${i}: nội dung thay đổi lần thứ ${i}`),
    ]
    const commits = subjects.map((s, i) => commit(`c${i}`, s))
    const rows = commits.map((c) => graphRow(c.id))
    seedRepo('repo-1', commits, rows)

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    // Khẳng định từng subject xuất hiện ĐÚNG MỘT LẦN — nếu một hàng bị lệch dữ
    // liệu hoặc DOM tái dùng sai, một subject sẽ xuất hiện nhiều lần trong khi
    // subject đúng của hàng khác biến mất.
    for (const subject of subjects) {
      expect(screen.getAllByText(subject, { exact: true })).toHaveLength(1)
    }

    // Không được có nhiều hơn MỘT hàng hiện dấu "." — nếu lỗi hiển thị làm mọi
    // subject dài bị cắt xuống một dấu chấm, cổng này bắt được ngay lập tức.
    expect(screen.getAllByText('.', { exact: true })).toHaveLength(1)
  })

  it('nhiều lane với color khác nhau: canvas vẽ đúng LANE_COLORS[row.color] cho từng hàng, không phải một màu cố định', () => {
    // Tái hiện lỗi quan sát được ở checkpoint round 1: đồ thị chỉ thấy một
    // đường màu đơn. Test này dựng dữ liệu có NHIỀU giá trị color khác nhau
    // (0, 1, 2) và khẳng định canvas thực sự gọi fillStyle với từng màu tương
    // ứng — không phải luôn luôn LANE_COLORS[0].
    const commits = [commit('a', 'lane 0'), commit('b', 'lane 1'), commit('c', 'lane 2')]
    const rows: GraphRow[] = [
      { commitId: 'a', lane: 0, color: 0, passthrough: [], outEdges: [], truncatedParents: 0, terminates: false },
      { commitId: 'b', lane: 1, color: 1, passthrough: [], outEdges: [], truncatedParents: 0, terminates: false },
      { commitId: 'c', lane: 2, color: 2, passthrough: [], outEdges: [], truncatedParents: 0, terminates: false },
    ]
    seedRepo('repo-1', commits, rows)

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
      fillStyleHistory: [] as string[],
      strokeStyle: '',
      lineWidth: 1,
      get fillStyle() {
        return ''
      },
      set fillStyle(v: string) {
        this.fillStyleHistory.push(v)
      },
    }
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
      fakeCtx as unknown as CanvasRenderingContext2D,
    )

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    // Ba hàng có color 0/1/2 khác nhau — nếu bộ vẽ hardcode một màu, chỉ có
    // một giá trị duy nhất xuất hiện trong lịch sử fillStyle thay vì ba.
    const distinctColorsUsed = new Set(fakeCtx.fillStyleHistory)
    expect(distinctColorsUsed.size).toBeGreaterThanOrEqual(3)
  })
})
