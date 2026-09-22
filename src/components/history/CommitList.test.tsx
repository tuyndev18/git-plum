/**
 * Test `CommitList` — trái tim của tiêu chí thành công số 2 (đồ thị luôn thẳng
 * hàng với văn bản). `@testing-library/react` vì đây thật sự render.
 *
 * `happy-dom` không cài `getContext('2d')` cho canvas thật, nên
 * `GraphCanvas`/`createCanvasRenderer` sẽ ném khi chạy trong JSDOM/happy-dom
 * trừ khi được vá cục bộ ở đây — vá đúng trong tệp test này, không toàn cục.
 */

import { createRef } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

import { CommitList, type CommitListHandle } from '@/components/history/CommitList'
import { useHistoryStore } from '@/stores/historyStore'
import { useRefsStore } from '@/stores/refsStore'
import { useStatusStore } from '@/stores/statusStore'
import type { Commit, GitRef, GraphRow, RepoStatus, StatusEntry } from '@/lib/ipc'
import { colorFor, graphWidth } from '@/lib/graph-render/geometry'
import {
  commitRowY,
  contentOffset,
  virtualizerCount,
  WIP_ROW_HEIGHT,
} from '@/lib/graph-render/wipRow'

// `CommitList` đọc `statusStore` từ 04-05 (hàng WIP), và `statusStore` nhập
// `describeError` + `ngheTrangThaiNgoai` từ cùng module này. Giả lập thiếu một
// trong hai làm cả suite **không nạp được** — và một suite không nạp được báo
// `total 0 / failed 0`, tức XANH GIẢ (lỗi #8 của CONTEXT.md 3.1). Đó là lý do
// mấy khoá dưới đây có mặt dù test trong tệp này không gọi tới chúng.
vi.mock('@/lib/ipc', () => ({
  ipc: {
    getCommitPage: vi.fn().mockResolvedValue({
      commits: [],
      graphRows: [],
      total: 0,
      skippedRecords: 0,
    }),
    getStatus: vi.fn().mockResolvedValue({
      branch: { head: null, oid: null, upstream: null, ahead: null, behind: null },
      entries: [],
      hasConflicts: false,
    }),
    stageFiles: vi.fn(),
    unstageFiles: vi.fn(),
  },
  describeError: (e: unknown) => String(e),
  ngheTrangThaiNgoai: vi.fn().mockResolvedValue(() => {}),
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
    fillRect: vi.fn(),
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
  useRefsStore.setState({ byRepo: {} })
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
      fillRect: vi.fn(),
      beginPath: vi.fn(),
      moveTo: vi.fn(),
      lineTo: vi.fn(),
      bezierCurveTo: vi.fn(),
      arc: vi.fn(),
      stroke: vi.fn(),
      fill: vi.fn(),
      fillText: vi.fn(),
      setLineDash: vi.fn(),
      // Theo dõi CẢ HAI kênh màu. Màu lane của một nút commit thường nằm ở
      // `strokeStyle` (nút là vòng tròn viền dày, tâm tô màu nền để cắt đường
      // lane phía sau), còn `fillStyle` mang màu nền nút và màu tâm của nút
      // merge. Chỉ theo dõi `fillStyle` thì test này không còn đo được điều nó
      // muốn đo — đó chính là cách nó đỏ khi nút chuyển từ chấm đặc sang vòng
      // viền.
      colorHistory: [] as string[],
      lineWidth: 1,
      get fillStyle() {
        return ''
      },
      set fillStyle(v: string) {
        this.colorHistory.push(v)
      },
      get strokeStyle() {
        return ''
      },
      set strokeStyle(v: string) {
        this.colorHistory.push(v)
      },
    }
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue(
      fakeCtx as unknown as CanvasRenderingContext2D,
    )

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    // Ba hàng có color 0/1/2 khác nhau. Khẳng định đúng ba màu lane ĐÓ có mặt,
    // không chỉ đếm số màu khác nhau: đếm suông sẽ xanh cả khi bộ vẽ dùng ba
    // màu tuỳ ý nào đó (kể cả màu nền nút và vòng chọn), tức không chứng minh
    // được nó đọc `row.color`.
    const used = new Set(fakeCtx.colorHistory)
    for (const colorIndex of [0, 1, 2]) {
      expect(
        used,
        `thiếu màu lane ${colorIndex} (${colorFor(colorIndex)}) — bộ vẽ không đọc row.color`,
      ).toContain(colorFor(colorIndex))
    }
  })
})

function gitRef(overrides: Partial<GitRef> & { fullName: string; target: string }): GitRef {
  return {
    shortName: overrides.fullName.split('/').pop() ?? overrides.fullName,
    kind: 'localBranch',
    upstream: null,
    ahead: 0,
    behind: 0,
    isHead: false,
    ...overrides,
  }
}

describe('nhãn ref trên hàng (plan 02-06, HIST-06)', () => {
  it('hàng có ref hiện RefBadges, hàng không ref không hiện gì thêm', () => {
    seedRepo('repo-1', [commit('a', 'co nhan'), commit('b', 'khong nhan')], [
      graphRow('a'),
      graphRow('b'),
    ])
    useRefsStore.setState({
      byRepo: {
        'repo-1': {
          refs: [gitRef({ fullName: 'refs/heads/main', target: 'a', kind: 'localBranch' })],
          byCommit: new Map([
            ['a', [gitRef({ fullName: 'refs/heads/main', target: 'a', kind: 'localBranch' })]],
          ]),
          isLoading: false,
          error: null,
        },
      },
    })

    const { container } = render(
      <CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />,
    )

    expect(container.querySelector('.ref-badge--local')).not.toBeNull()
  })

  it('chiều cao hàng KHÔNG đổi theo số nhãn (HIST-04) — mọi hàng vẫn ROW_HEIGHT cố định', () => {
    const refs = Array.from({ length: 8 }, (_, i) =>
      gitRef({ fullName: `refs/heads/b${i}`, target: 'a', shortName: `b${i}`, kind: 'localBranch' }),
    )
    seedRepo('repo-1', [commit('a', 'nhieu nhan'), commit('b', 'khong nhan')], [
      graphRow('a'),
      graphRow('b'),
    ])
    useRefsStore.setState({
      byRepo: {
        'repo-1': {
          refs,
          byCommit: new Map([['a', refs]]),
          isLoading: false,
          error: null,
        },
      },
    })

    const { container } = render(
      <CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />,
    )

    const rows = container.querySelectorAll('.commit-row')
    const heights = Array.from(rows).map((r) => (r as HTMLElement).style.height)
    // Mọi hàng cùng chiều cao dù số nhãn khác nhau (0 vs 8) — không có hàng
    // nào giãn ra theo nội dung nhãn.
    expect(new Set(heights).size).toBe(1)
  })
})

describe('scrollToIndex lộ ra qua ref (HIST-10, không tạo virtualizer thứ hai)', () => {
  it('gọi ref.current.scrollToIndex(n) không ném lỗi', () => {
    seedRepo('repo-1', [commit('a', 'x'), commit('b', 'y')], [graphRow('a'), graphRow('b')])
    const ref = createRef<CommitListHandle>()

    render(<CommitList ref={ref} repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(() => ref.current?.scrollToIndex(1)).not.toThrow()
  })
})

describe('scrollToCommit — bấm nhánh/tag ở thanh bên nhảy tới commit', () => {
  /*
   * 🔴 Lỗi người dùng báo: "bấm vào branch, tag ở cột bên trái không bị nhảy
   * đến commit". Nguyên nhân: `RefSidebar` chỉ ghi `selectionStore`, và không
   * một đường nào nối từ đó tới virtualizer của `CommitList` — `scrollToIndex`
   * tồn tại nhưng chỉ `CommitSearch` gọi, mà thanh bên không đi qua nó.
   *
   * `scrollToCommit` là điểm nối còn thiếu: nhận **sha** thay vì chỉ số, vì
   * thanh bên chỉ biết `ref.target`, không biết hàng thứ mấy.
   */
  it('commit đã nạp -> trả true', () => {
    seedRepo('repo-1', [commit('a', 'x'), commit('b', 'y')], [graphRow('a'), graphRow('b')])
    const ref = createRef<CommitListHandle>()

    render(<CommitList ref={ref} repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(ref.current?.scrollToCommit('b')).toBe(true)
  })

  it('sha không có trong lịch sử đã nạp -> trả false, KHÔNG ném', () => {
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])
    const ref = createRef<CommitListHandle>()

    render(<CommitList ref={ref} repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(ref.current?.scrollToCommit('khong-ton-tai')).toBe(false)
  })

  it('mảng commits THƯA (trang chưa nạp) -> lỗ không làm hỏng chỉ mục', () => {
    /*
     * `mergePage` cấp trước `commits` tới `total` và để các vị trí chưa nạp ở
     * `undefined`. Một vòng lặp dựng chỉ mục mà tin rằng mọi phần tử đều có
     * commit sẽ ném ở đây — và ném đúng trên repo lớn, tức đúng ca mà tính
     * năng này tồn tại để phục vụ.
     */
    const commits: Commit[] = []
    commits.length = 2000
    commits[0] = commit('a', 'dau')
    commits[1500] = commit('z', 'cuoi')
    const rows: GraphRow[] = []
    rows.length = 2000
    rows[0] = graphRow('a')
    rows[1500] = graphRow('z')

    seedRepo('repo-1', commits, rows, 2000)
    const ref = createRef<CommitListHandle>()

    render(<CommitList ref={ref} repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(ref.current?.scrollToCommit('z')).toBe(true)
    // Vị trí 999 là một lỗ — không có commit nào mang sha đó.
    expect(ref.current?.scrollToCommit('chua-nap')).toBe(false)
  })
})

describe('bề rộng cột đồ thị tính CẢ lane của cạnh, không chỉ lane của hàng', () => {
  /*
   * 🔴 Lỗi người dùng báo bằng ảnh: hai nút ở lane 1 nằm lơ lửng, không một
   * đường lane nào nối tới chúng.
   *
   * Dữ liệu backend ĐÚNG — kiểm bằng `cargo run --bin rowdump` trên repo thật:
   * hàng 106 của `dau-tri-toan-hoc` mang `lane 0` nhưng có `passthrough 1->1`
   * và `outEdges 0->0, 0->2`. Chuỗi lane liền mạch từ đầu tới cuối.
   *
   * Lỗi ở frontend: `maxLane` chỉ đọc `row.lane` của các hàng ĐANG THẤY, rồi
   * `graphWidth(maxLane)` đặt cả bề rộng cột CSS lẫn bề rộng canvas. Cuộn tới
   * một vùng toàn hàng lane 0 → cột co về đúng một lane → mọi cạnh ở lane ≥ 1
   * bị vẽ ra ngoài canvas và biến mất.
   *
   * Vì sao không test nào cũ bắt được: lỗi cần một hàng có `lane` THẤP nhưng
   * cạnh ở lane CAO. Fixture nào đặt `lane` bằng lane cao nhất của chính nó
   * đều xanh — đúng khuôn "hình dạng đúng, dữ liệu vô hại" đã dính ba lần.
   */
  it('hàng lane 0 mang passthrough ở lane 2 -> cột vẫn đủ rộng cho lane 2', () => {
    const commits = [commit('a', 'hang mot'), commit('b', 'hang hai')]
    const rows: GraphRow[] = [
      {
        ...graphRow('a'),
        lane: 0,
        // Nhánh song song đi xuyên qua hàng này ở lane 2 — hàng ở lane 0 nhưng
        // vẫn PHẢI vẽ tới lane 2.
        passthrough: [{ fromLane: 2, toLane: 2, color: 2 }],
      },
      { ...graphRow('b'), lane: 0 },
    ]
    seedRepo('repo-1', commits, rows)

    const { container } = render(
      <CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />,
    )

    const canvas = container.querySelector('canvas')
    expect(canvas, 'phải có canvas đồ thị').not.toBeNull()
    // `graphWidth(2)` cần chỗ cho ba lane (0,1,2). `graphWidth(0)` chỉ cho một.
    expect(
      Number((canvas as HTMLCanvasElement).getAttribute('width')),
      'bề rộng canvas phải đủ cho lane 2 của passthrough, không co theo row.lane',
    ).toBe(graphWidth(2))
  })

  it('hàng lane 0 mang outEdges rẽ sang lane 3 -> cột đủ rộng cho lane 3', () => {
    // Ca merge: `outEdges 0->0, 0->3` là hình dạng thật ở hàng 111 của
    // `dau-tri-toan-hoc` (`merge(23-04)`).
    const commits = [commit('a', 'merge')]
    const rows: GraphRow[] = [
      {
        ...graphRow('a'),
        lane: 0,
        outEdges: [
          { fromLane: 0, toLane: 0, color: 0 },
          { fromLane: 0, toLane: 3, color: 3 },
        ],
      },
    ]
    seedRepo('repo-1', commits, rows)

    const { container } = render(
      <CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />,
    )

    const canvas = container.querySelector('canvas') as HTMLCanvasElement
    expect(Number(canvas.getAttribute('width'))).toBe(graphWidth(3))
  })
})

/* ------------------------------------------------------------------------ *
 * Hàng WIP — WORK-11, plan 04-05 Task 1
 *
 * 🔴 Ranh giới của những test này, nói thẳng: chúng chứng minh các **con số**
 * khớp nhau. Chúng **không** chứng minh mắt người thấy thẳng hàng. happy-dom
 * không tính CSS layout và không có cuộn thật (CONTEXT.md 3.4) — cả năm lỗi
 * hiển thị của Phase 3 đi qua 435 test tự động. Việc kiểm bằng mắt là Task 3.
 * ------------------------------------------------------------------------ */

function muc(path: string, xy: string, group: StatusEntry['group']): StatusEntry {
  return { path, oldPath: null, xy, group, hasInvalidUtf8: false }
}

function trangThai(entries: StatusEntry[], headOid: string | null = null): RepoStatus {
  return {
    branch: { head: 'main', oid: headOid, upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

function seedStatus(repoId: string, status: RepoStatus | undefined) {
  useStatusStore.setState((s) => ({
    byRepo: { ...s.byRepo, [repoId]: { status, isLoading: false, error: null } },
  }))
}

describe('hàng WIP trong CommitList (WORK-11)', () => {
  beforeEach(() => {
    useStatusStore.setState({ byRepo: {} })
  })

  it('TIỀN ĐỀ: không có status → KHÔNG hàng WIP, và mọi hàng commit vẫn render', () => {
    // Đây là ca hồi quy của toàn bộ 04-05: bật tính năng lên **không được**
    // đổi hành vi khi repo sạch. Nếu test này đỏ, mọi test dưới nó vô nghĩa.
    seedRepo('repo-1', [commit('a', 'khong co thay doi')], [graphRow('a')])

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(screen.queryByTestId('wip-row')).toBeNull()
    expect(screen.getByText('khong co thay doi')).toBeTruthy()
  })

  it('có tệp sửa + tệp mới → hàng WIP hiện, mang chuỗi nhận diện dem sua va dem moi', () => {
    seedRepo('repo-1', [commit('a', 'commit dau')], [graphRow('a')])
    seedStatus(
      'repo-1',
      trangThai([
        muc('x1.txt', '.M', 'unstaged'),
        muc('x2.txt', '.M', 'unstaged'),
        muc('x3.txt', '.M', 'unstaged'),
        muc('moi.txt', '??', 'untracked'),
      ]),
    )

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    const hang = screen.getByTestId('wip-row')
    expect(hang.getAttribute('data-wip-modified')).toBe('3')
    expect(hang.getAttribute('data-wip-added')).toBe('1')
    // Và con số phải thật sự HIỆN RA, không chỉ nằm trong thuộc tính data.
    expect(hang.textContent).toContain('3')
    expect(hang.textContent).toContain('1')
  })

  it('tệp XY = MM sinh HAI phần tử nhưng đếm MỘT — đếm đường dẫn duy nhất', () => {
    // Cùng bất biến mà `RepoStatus::wip_counts` phía Rust ghim. Phía TS phải
    // tự dẫn xuất (trường đó KHÔNG đi qua dây IPC), nên nó cần cổng riêng.
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])
    seedStatus(
      'repo-1',
      trangThai([muc('same.txt', 'MM', 'staged'), muc('same.txt', 'MM', 'unstaged')]),
    )

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(screen.getByTestId('wip-row').getAttribute('data-wip-modified')).toBe('1')
  })

  it('tệp đã stage thêm mới (A.) đếm là TỆP MỚI, không phải tệp sửa', () => {
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])
    seedStatus('repo-1', trangThai([muc('moi.txt', 'A.', 'staged')]))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    const hang = screen.getByTestId('wip-row')
    expect(hang.getAttribute('data-wip-added')).toBe('1')
    expect(hang.getAttribute('data-wip-modified')).toBe('0')
  })

  it('🔴 tiêu chí 7: entries rỗng (đã commit hết) → hàng WIP BIẾN MẤT', () => {
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])
    seedStatus('repo-1', trangThai([]))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(screen.queryByTestId('wip-row')).toBeNull()
  })

  it('bấm hàng WIP gọi onOpenCommitBox, và KHÔNG gọi onSelect với commitId bịa', () => {
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')]))

    const onSelect = vi.fn()
    const onOpenCommitBox = vi.fn()
    render(
      <CommitList
        repoId="repo-1"
        selectedCommitId={null}
        onSelect={onSelect}
        onOpenCommitBox={onOpenCommitBox}
      />,
    )

    screen.getByTestId('wip-row').click()

    expect(onOpenCommitBox).toHaveBeenCalledTimes(1)
    // 🔴 Hàng WIP KHÔNG có SHA. Gọi `onSelect` ở đây nghĩa là bịa một commitId
    // và `CommitDetail` sẽ đi hỏi backend về một sha không tồn tại.
    expect(onSelect).not.toHaveBeenCalled()
  })

  it('hàng WIP KHÔNG vào historyStore.commits — số commit không đổi khi bật/tắt', () => {
    const commits = [commit('a', 'x'), commit('b', 'y')]
    seedRepo('repo-1', commits, [graphRow('a'), graphRow('b')])

    const truoc = useHistoryStore.getState().byRepo['repo-1']?.commits.length
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')]))
    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)
    const sau = useHistoryStore.getState().byRepo['repo-1']?.commits.length

    expect(screen.getByTestId('wip-row')).toBeTruthy()
    expect(sau, 'hàng WIP không được chen vào mảng commits').toBe(truoc)
    expect(sau).toBe(2)
  })

  it('🔴 count của virtualizer === total KỂ CẢ khi có hàng WIP (cách B)', () => {
    // Cách A (`count: total + 1`) rải một phép `-1` ra bảy chỗ tiêu thụ
    // `virtualItems`. Test này ghim cách B ở mức số học; cổng số học của
    // `wipRow.test.ts` ghim nó ở mức nguồn.
    expect(virtualizerCount(2, true)).toBe(2)
    expect(virtualizerCount(2, false)).toBe(2)

    seedRepo('repo-1', [commit('a', 'x'), commit('b', 'y')], [graphRow('a'), graphRow('b')])
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')]))

    const { container } = render(
      <CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />,
    )

    // Đúng 2 hàng commit được dựng — không 3. Hàng WIP là anh em NGOÀI vùng đó.
    expect(container.querySelectorAll('.commit-row').length).toBe(2)
  })

  it('🔴 THẲNG CỘT ở 3 vị trí cuộn: cột đồ thị và cột văn bản dùng CÙNG một y', () => {
    /*
     * ⚠️ Đây là test **số học**, không phải bằng chứng hiển thị. happy-dom
     * không cuộn thật, nên nó khẳng định đúng một điều: với cùng đầu vào, hai
     * cột đọc **cùng** hàm `commitRowY` nên chúng **không thể** ra hai số khác
     * nhau. Việc mắt người thấy thẳng hàng là bước 5 của checkpoint Task 3.
     *
     * R1 đòi ≥ 3 vị trí cuộn: đầu, giữa, cuối.
     */
    const viTriCuon = [0, 1400, 27_972]
    for (const scrollTop of viTriCuon) {
      for (const start of [0, 28, 1400]) {
        const yCoWip = commitRowY(start, scrollTop, true)
        const yKhongWip = commitRowY(start, scrollTop, false)
        // Cột đồ thị và cột văn bản gọi CÙNG hàm này với CÙNG tham số, nên
        // đẳng thức dưới đây là điều duy nhất cần chứng minh về "thẳng cột".
        expect(yCoWip - yKhongWip, `lech tai scrollTop=${scrollTop}`).toBe(WIP_ROW_HEIGHT)
        expect(yCoWip - yKhongWip, 'lech DUNG mot hang, khong hai').toBe(contentOffset(true))
      }
    }
  })

  it('headLane ngoài tầm (>= MAX_VISIBLE_LANES) → chỉ báo suy giảm, không vẽ vào cột 12', () => {
    // HEAD ở lane 25 trên một repo nhiều nhánh. `laneX` clamp vô điều kiện,
    // nên vẽ thẳng sẽ nối hàng WIP vào một commit KHÁC — sai một cách tự tin.
    const commits = [commit('head-sha', 'HEAD o lane cao')]
    const rows: GraphRow[] = [{ ...graphRow('head-sha'), lane: 25 }]
    seedRepo('repo-1', commits, rows)
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')], 'head-sha'))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    const hang = screen.getByTestId('wip-row')
    expect(
      hang.getAttribute('data-wip-edge'),
      'lane HEAD vuot cap phai bao `clamped`, khong im lang ve sai cot',
    ).toBe('clamped')
  })

  it('HEAD nằm trong tầm → cạnh WIP ở biến thể normal, mang đúng lane của HEAD', () => {
    const commits = [commit('c0', 'moi nhat theo topo'), commit('head-sha', 'HEAD that su')]
    const rows: GraphRow[] = [
      { ...graphRow('c0'), lane: 0 },
      { ...graphRow('head-sha'), lane: 3 },
    ]
    seedRepo('repo-1', commits, rows)
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')], 'head-sha'))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    const hang = screen.getByTestId('wip-row')
    expect(hang.getAttribute('data-wip-edge')).toBe('normal')
    // 🔴 Lane 3 của HEAD, KHÔNG phải lane 0 của hàng 0. Nhầm hai thứ này là
    // gốc của cả lỗi — xem doc comment của `wipEdge`.
    expect(hang.getAttribute('data-wip-lane')).toBe('3')
  })

  it('HEAD không nằm trong lịch sử đã nạp → unknownHead, không đoán lane 0', () => {
    seedRepo('repo-1', [commit('a', 'x')], [{ ...graphRow('a'), lane: 0 }])
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')], 'sha-khong-co-trong-trang'))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    expect(screen.getByTestId('wip-row').getAttribute('data-wip-edge')).toBe('unknownHead')
  })

  it('hàng WIP cao ĐÚNG WIP_ROW_HEIGHT trong style — Phase 2 hỏng hai lần ở đây', () => {
    // ⚠️ Khẳng định về **style inline**, không phải về layout đã tính. CSS
    // padding/border vẫn có thể phá chiều cao và happy-dom sẽ không thấy —
    // bước 5 của checkpoint Task 3 mới kiểm được điều đó.
    seedRepo('repo-1', [commit('a', 'x')], [graphRow('a')])
    seedStatus('repo-1', trangThai([muc('x1.txt', '.M', 'unstaged')]))

    render(<CommitList repoId="repo-1" selectedCommitId={null} onSelect={vi.fn()} />)

    const hang = screen.getByTestId('wip-row') as HTMLElement
    expect(hang.style.height).toBe(`${WIP_ROW_HEIGHT}px`)
  })
})
