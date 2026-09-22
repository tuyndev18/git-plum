/**
 * Test render App — quy tắc MVP của phase: sau plan 02-05 mở một repository
 * phải thấy CommitList thật, không phải "sẽ nối sau".
 *
 * Chặn `@/lib/ipc` ở ranh giới module, giống mọi test khác trong dự án.
 * `restoreMocks: true` trong `vite.config.ts` xoá `mockResolvedValue` đặt
 * trong factory `vi.mock` giữa các test — dựng lại mặc định trong
 * `beforeEach` là bắt buộc.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { act, render, screen } from '@testing-library/react'

import { App } from '@/App'
import { ipc, type RepoInfo } from '@/lib/ipc'
import { useHistoryStore } from '@/stores/historyStore'
import { useRepoStore } from '@/stores/repoStore'
import { useRefsStore } from '@/stores/refsStore'
import { useSelectionStore } from '@/stores/selectionStore'

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

vi.mock('@/lib/ipc', () => ({
  ipc: {
    openRepository: vi.fn(),
    closeRepository: vi.fn(),
    currentBranch: vi.fn(),
    gitVersion: vi.fn(),
    getCommitPage: vi.fn(),
    commandLog: vi.fn(),
    listRefs: vi.fn(),
    getCommitDetail: vi.fn(),
    searchCommits: vi.fn(),
  },
  describeError: (e: unknown) => (e instanceof Error ? e.message : String(e)),
}))

vi.mock('@/lib/recentRepos', () => ({
  loadRecentRepos: vi.fn().mockResolvedValue([]),
  rememberRepo: vi.fn().mockResolvedValue([]),
  forgetRepo: vi.fn().mockResolvedValue([]),
}))

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

function repoInfo(id: string): RepoInfo {
  return { id, path: `C:/kho/${id}`, name: id }
}

beforeEach(() => {
  vi.clearAllMocks()
  stubCanvasContext()
  stubViewportSize()
  useRepoStore.setState({ byRepo: {}, activeRepoId: null, isOpening: false, recent: [] })
  useHistoryStore.setState({ byRepo: {} })
  useRefsStore.setState({ byRepo: {} })
  useSelectionStore.setState({ selectedByRepo: {} })
  vi.mocked(ipc.gitVersion).mockResolvedValue('git version 2.54.0')
  vi.mocked(ipc.getCommitPage).mockResolvedValue({
    commits: [],
    graphRows: [],
    total: 0,
    skippedRecords: 0,
  })
  vi.mocked(ipc.commandLog).mockResolvedValue([])
  vi.mocked(ipc.listRefs).mockResolvedValue([])
  vi.mocked(ipc.searchCommits).mockResolvedValue([])
})

describe('App', () => {
  it('chưa mở repository thì hiện empty-state', () => {
    render(<App />)

    expect(screen.getByText('Chưa mở repository nào')).toBeTruthy()
  })

  it('mở repository thì vùng main hiện CommitList, không còn chữ chỗ trống cũ', async () => {
    useRepoStore.setState({
      byRepo: { r1: { info: repoInfo('r1'), currentBranch: 'main', error: null } },
      activeRepoId: 'r1',
      isOpening: false,
      recent: [],
    })
    useHistoryStore.setState({
      byRepo: {
        r1: {
          commits: [
            {
              id: 'c1',
              parents: [],
              authorName: 'A',
              authorEmail: 'a@x.com',
              authorTime: 1700000000,
              committerName: 'A',
              committerEmail: 'a@x.com',
              committerTime: 1700000000,
              subject: 'commit dau tien',
              body: '',
              hasInvalidUtf8: false,
            },
          ],
          graphRows: [
            {
              commitId: 'c1',
              lane: 0,
              color: 0,
              passthrough: [],
              outEdges: [],
              truncatedParents: 0,
              terminates: false,
            },
          ],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })

    render(<App />)

    expect(screen.getByText('commit dau tien')).toBeTruthy()
    expect(screen.queryByText('Đồ thị commit và danh sách lịch sử thuộc Phase 2')).toBeNull()
  })

  it('activeRepoId đổi thì tự gọi ipc.getCommitPage để nạp trang đầu', async () => {
    // Khác với test trên (dữ liệu bơm sẵn vào historyStore), test này khẳng
    // định chính đường nối App -> loadFirstPage -> ipc.getCommitPage chạy
    // thật khi mở một repository — đây là quy tắc MVP "không có bước sẽ nối
    // sau" của Task 3, không phải hành vi render của CommitList (đã kiểm ở
    // CommitList.test.tsx).
    useRepoStore.setState({
      byRepo: { r1: { info: repoInfo('r1'), currentBranch: 'main', error: null } },
      activeRepoId: 'r1',
      isOpening: false,
      recent: [],
    })

    render(<App />)

    await vi.waitFor(() => {
      expect(ipc.getCommitPage).toHaveBeenCalledWith('r1', 0, expect.any(Number))
    })
  })

  it('mở repository -> sidebar và vùng chi tiết không còn placeholder "Phase 2 sẽ điền"', async () => {
    useRepoStore.setState({
      byRepo: { r1: { info: repoInfo('r1'), currentBranch: 'main', error: null } },
      activeRepoId: 'r1',
      isOpening: false,
      recent: [],
    })

    render(<App />)

    await vi.waitFor(() => expect(ipc.listRefs).toHaveBeenCalledWith('r1'))

    expect(screen.queryByText('Plan 02-06 sẽ điền phần này.')).toBeNull()
  })

  it('chọn một commit -> vùng chi tiết hiện subject của commit đó (nối qua selectionStore)', async () => {
    useRepoStore.setState({
      byRepo: { r1: { info: repoInfo('r1'), currentBranch: 'main', error: null } },
      activeRepoId: 'r1',
      isOpening: false,
      recent: [],
    })
    useHistoryStore.setState({
      byRepo: {
        r1: {
          commits: [
            {
              id: 'c1',
              parents: [],
              authorName: 'A',
              authorEmail: 'a@x.com',
              authorTime: 1700000000,
              committerName: 'A',
              committerEmail: 'a@x.com',
              committerTime: 1700000000,
              subject: 'commit chon duoc',
              body: '',
              hasInvalidUtf8: false,
            },
          ],
          graphRows: [
            {
              commitId: 'c1',
              lane: 0,
              color: 0,
              passthrough: [],
              outEdges: [],
              truncatedParents: 0,
              terminates: false,
            },
          ],
          total: 1,
          loadedRanges: [[0, 1]],
          isLoading: false,
          error: null,
        },
      },
    })
    vi.mocked(ipc.getCommitDetail).mockResolvedValue({
      commit: {
        id: 'c1',
        parents: [],
        authorName: 'A',
        authorEmail: 'a@x.com',
        authorTime: 1700000000,
        committerName: 'A',
        committerEmail: 'a@x.com',
        committerTime: 1700000000,
        subject: 'commit chon duoc',
        body: '',
        hasInvalidUtf8: false,
      },
      files: [],
      truncated: false,
    })

    render(<App />)

    act(() => screen.getByText('commit chon duoc').click())

    await vi.waitFor(() => {
      // Subject xuất hiện ở CẢ hàng CommitList lẫn CommitDetail sau khi chọn.
      expect(screen.getAllByText('commit chon duoc').length).toBeGreaterThanOrEqual(2)
    })
  })
})

/*
 * 🔴 Lỗi 2 người dùng báo ở checkpoint vòng 2 plan 03-04: trình xem diff "chưa
 * full height" — nó cao khoảng nửa cửa sổ, còn khoảng trống bên dưới.
 *
 * # Nguyên nhân gốc, đo bằng Chromium thật (Playwright), KHÔNG phải đọc CSS
 *
 * Chuỗi chiều cao `.app` → … → `.cm-mergeView` **không hề đứt**. Đi ngược cây
 * từ `.cm-mergeView` lên `body` ở cửa sổ 1600×900, mọi mắt đều truyền đúng:
 *
 * | # | mắt | chiều cao đo được |
 * |---|---|---|
 * | 2 | `.app` | 900 |
 * | 3 | `.body` | 831.5 |
 * | 4 | `.layout` | 831.5 |
 * | 5 | `[data-panel=top]` `flex: 70 1 0px` | **581.3** ← tụt 250px ở ĐÂY |
 * | 7 | `.pane-group` | 581.3 |
 * | 10 | `.main` | 581.3 |
 * | 11 | `.diff-viewer` | 581.3 |
 * | 14 | `.cm-mergeView` | 544.3 (scrollHeight 1394, `overflow-y: auto` — cuộn ĐÚNG) |
 *
 * Nên đây **không** phải lỗi CSS. Chỗ tụt 250px là `Panel id="bottom"` —
 * **nhật ký lệnh** — chiếm 30% chiều cao dọc (đo được: 249.2px) ngay từ lần mở
 * đầu, trong khi nội dung nó hiện chỉ là dòng *"Chưa có lệnh nào được chạy."*
 *
 * `logVisible` khởi tạo `useState(true)`. PLAT-08 đòi người dùng **nhìn thấy
 * được** những lệnh git đã chạy — nó không đòi panel đó mở sẵn. Một panel chẩn
 * đoán mở mặc định lấy 30% chiều cao của Core Value ("đọc lịch sử phải tức
 * thì") là một đánh đổi sai, và nó đúng thứ người dùng chụp ảnh gửi lại.
 *
 * Sửa: mặc định **ẩn**. Nút thanh công cụ và `Ctrl+\`` vẫn mở được, nên PLAT-08
 * vẫn trọn.
 */
describe('nhật ký lệnh — mặc định ẩn để trình xem diff được full height', () => {
  it('lần mở đầu KHÔNG hiện panel nhật ký lệnh', () => {
    render(<App />)

    /*
     * Tìm theo **role heading**, không theo chữ: nút thanh công cụ mang đúng
     * nhãn "Nhật ký lệnh", nên `queryByText` khớp cả nút và làm cổng này vô
     * nghĩa (đã gặp thật ở lần chạy đầu). `<h2>` là thứ chỉ `CommandLogPanel`
     * render, tức nó có mặt đúng khi panel đang mở và đang chiếm 30% chiều cao.
     */
    expect(
      screen.queryByRole('heading', { name: 'Nhật ký lệnh' }),
      'panel nhật ký lệnh mở mặc định lấy ~250px (30%) chiều cao ở cửa sổ 900px, ' +
        'làm trình xem diff chỉ còn ~581px — đúng lỗi "chưa full height" người dùng báo',
    ).toBeNull()
  })

  it('bấm nút thanh công cụ thì nhật ký hiện ra — PLAT-08 vẫn trọn', async () => {
    render(<App />)

    const nut = screen.getByRole('button', { name: 'Nhật ký lệnh' })
    await act(async () => {
      nut.click()
    })

    expect(
      screen.queryByRole('heading', { name: 'Nhật ký lệnh' }),
      'PLAT-08 đòi người dùng nhìn thấy được lệnh git đã chạy — mặc định ẩn ' +
        'chỉ hợp lệ nếu vẫn mở được',
    ).not.toBeNull()
  })
})
