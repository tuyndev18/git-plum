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
import { ipc, type RepoInfo, type RepoStatus } from '@/lib/ipc'
import { useHistoryStore } from '@/stores/historyStore'
import { useRepoStore } from '@/stores/repoStore'
import { useRefsStore } from '@/stores/refsStore'
import { useSelectionStore } from '@/stores/selectionStore'
import { useStatusStore } from '@/stores/statusStore'
import { useCommitStore } from '@/stores/commitStore'
import { ngheTrangThaiNgoai } from '@/lib/ipc'

vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

/*
 * Bề mặt của vòng commit (Phase 4) được **thêm vào** mock này, không thay gì cả.
 *
 * `App` nay gắn watcher `.git` (`ngheTrangThaiNgoai`, WORK-10) và render
 * `ChangeList`/`CommitBox` khi vùng soạn mở. Mock của tệp này là bản thay thế
 * **trọn gói** (không `importActual`), nên mọi export mà đường render đụng tới phải
 * có mặt ở đây — thiếu một cái là `App` ném lúc gắn kết và **mọi** test trong tệp
 * đỏ vì một lý do không liên quan tới điều nó khẳng định.
 *
 * 🔴 Không khẳng định nào bên dưới được sửa để cho xanh. Chín test đỏ lúc nối dây
 * đều đỏ với **cùng một** thông báo "No ngheTrangThaiNgoai export is defined", tức
 * lỗi nằm ở độ đầy đủ của mock, không ở hành vi.
 *
 * `ngheTrangThaiNgoai` trả một hàm huỷ đăng ký: `App` gọi nó lúc dọn effect, nên
 * trả `undefined` sẽ ném ở chế độ Strict của React 19.
 */
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
    getStatus: vi.fn(),
    stageFiles: vi.fn(),
    unstageFiles: vi.fn(),
    getWorktreeDiff: vi.fn(),
    createCommit: vi.fn(),
    amendCommit: vi.fn(),
  },
  ngheTrangThaiNgoai: vi.fn(async () => () => {}),
  describeError: (e: unknown) => (e instanceof Error ? e.message : String(e)),
  isGitError: () => false,
}))

/*
 * Nháp commit bền vững đi qua `@tauri-apps/plugin-store`, không có trong happy-dom.
 * `commitStore.switchRepo` chạy khi `activeRepoId` đổi — tức trong mọi test mở
 * repository — nên thiếu stub này là một lời từ chối promise không ai bắt.
 */
vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(async () => ({
    get: vi.fn(async () => undefined),
    set: vi.fn(async () => undefined),
    delete: vi.fn(async () => undefined),
    save: vi.fn(async () => undefined),
  })),
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
  // Mặc định "không có thay đổi nào": nhánh đường chính của mọi test cũ, và nó
  // giữ hàng WIP TẮT ở đó — nếu không, mọi test cũ bỗng render thêm một hàng.
  vi.mocked(ipc.getStatus).mockResolvedValue({
    branch: { head: 'main', oid: 'c1', upstream: null, ahead: null, behind: null },
    entries: [],
    hasConflicts: false,
  })
  useStatusStore.setState({ byRepo: {} })
  useCommitStore.setState({ draftByRepo: {} })
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

/**
 * Nối dây vòng commit vào `App` — wave 5 của Phase 4.
 *
 * ⚠️ **Phạm vi của những test này, nói thẳng.** Chúng chứng minh các **đường nối**
 * tồn tại và chạy: watcher được đăng ký và huỷ, `ChangeList`/`CommitBox` có mặt
 * trong cây khi vùng soạn mở, hàng WIP mở được vùng đó. Chúng **không** chứng minh
 * được bất cứ điều gì về **bố cục** — happy-dom không tính CSS và không có cuộn
 * thật, và đó chính là lý do cả 5 lỗi hiển thị của Phase 3 đi qua 435 test xanh.
 * Bằng chứng về bố cục chỉ đến từ bước 1 và bước 5 của checkpoint 04-05 Task 3.
 */
describe('vòng commit nối vào App (WORK-10, WORK-11)', () => {
  function moRepo() {
    useRepoStore.setState({
      byRepo: { r1: { info: repoInfo('r1'), currentBranch: 'main', error: null } },
      activeRepoId: 'r1',
      isOpening: false,
      recent: [],
    })
  }

  /** `RepoStatus` có thay đổi chưa commit → hàng WIP bật. */
  function coThayDoi() {
    vi.mocked(ipc.getStatus).mockResolvedValue({
      branch: { head: 'main', oid: 'c1', upstream: null, ahead: null, behind: null },
      entries: [
        { path: 'a.ts', oldPath: null, xy: '.M', group: 'unstaged', hasInvalidUtf8: false },
        { path: 'moi.ts', oldPath: null, xy: '??', group: 'untracked', hasInvalidUtf8: false },
      ],
      hasConflicts: false,
    })
  }

  it('gắn kết App thì đăng ký watcher .git đúng MỘT lần (WORK-10)', async () => {
    moRepo()

    render(<App />)

    await vi.waitFor(() => {
      expect(vi.mocked(ngheTrangThaiNgoai)).toHaveBeenCalledTimes(1)
    })
  })

  it('unmount App thì huỷ đăng ký watcher — không rò người nghe qua các lần mở', async () => {
    const huy = vi.fn()
    vi.mocked(ngheTrangThaiNgoai).mockResolvedValue(huy)
    moRepo()

    const { unmount } = render(<App />)
    await vi.waitFor(() => expect(vi.mocked(ngheTrangThaiNgoai)).toHaveBeenCalled())

    unmount()

    // 🔴 Hàm huỷ về tay qua một promise, nên nó có thể tới SAU lúc unmount. Đường
    // dọn của `App` ghi lại ý định huỷ và `.then` tự gọi khi hàm về — `waitFor`
    // ở đây kiểm đúng đường đó, không phải một phép gọi đồng bộ.
    await vi.waitFor(() => expect(huy).toHaveBeenCalledTimes(1))
  })

  it('watcher phát sự kiện → statusStore nhận trạng thái mới mà KHÔNG gọi thêm git status', async () => {
    type XuLy = Parameters<typeof ngheTrangThaiNgoai>[0]
    let phat: XuLy | null = null
    vi.mocked(ngheTrangThaiNgoai).mockImplementation(async (cb) => {
      phat = cb
      return () => {}
    })
    moRepo()

    render(<App />)
    await vi.waitFor(() => expect(phat).not.toBeNull())

    const soLanTruoc = vi.mocked(ipc.getStatus).mock.calls.length
    const moi: RepoStatus = {
      branch: { head: 'main', oid: 'c9', upstream: null, ahead: null, behind: null },
      entries: [
        { path: 'ngoai.ts', oldPath: null, xy: '.M', group: 'unstaged', hasInvalidUtf8: false },
      ],
      hasConflicts: false,
    }

    await act(async () => {
      phat!({ repoId: 'r1', status: moi })
    })

    expect(useStatusStore.getState().byRepo.r1?.status).toEqual(moi)
    // Tiêu chí 5: watcher phía Rust ĐÃ đọc `git status` rồi mới phát; đọc lại là
    // một tiến trình thừa và nó còn chậm hơn vì nằm sau 250–300 ms trì hoãn.
    expect(vi.mocked(ipc.getStatus).mock.calls.length).toBe(soLanTruoc)
  })

  it('không có thay đổi → vùng chi tiết vẫn là chi tiết commit, chưa có hàng WIP', async () => {
    moRepo()

    render(<App />)

    await vi.waitFor(() => expect(ipc.listRefs).toHaveBeenCalled())
    expect(screen.queryByTestId('wip-row')).toBeNull()
    expect(screen.queryByTestId('change-list')).toBeNull()
    expect(screen.queryByTestId('commit-box')).toBeNull()
  })

  it('có thay đổi → hàng WIP hiện, và bấm nó mở ChangeList + CommitBox (WORK-11)', async () => {
    coThayDoi()
    moRepo()

    render(<App />)

    const hang = await screen.findByTestId('wip-row')
    // Vùng soạn CHƯA mở trước khi bấm — nếu nó mở sẵn thì phép bấm dưới đây
    // không chứng minh gì cả.
    expect(screen.queryByTestId('commit-box')).toBeNull()

    await act(async () => {
      hang.click()
    })

    expect(screen.getByTestId('change-list')).toBeTruthy()
    expect(screen.getByTestId('commit-box')).toBeTruthy()
  })

  it('bấm hàng WIP KHÔNG chọn một commit nào — hàng WIP không có SHA', async () => {
    coThayDoi()
    moRepo()

    render(<App />)
    const hang = await screen.findByTestId('wip-row')

    await act(async () => {
      hang.click()
    })

    // 🔴 Gọi `onSelect` ở đây buộc phải bịa một commitId, và `CommitDetail` sẽ đi
    // hỏi backend về một sha không tồn tại.
    expect(useSelectionStore.getState().selectedByRepo.r1).toBeUndefined()
  })

  it('đóng vùng soạn → vùng chi tiết quay lại chi tiết commit', async () => {
    coThayDoi()
    moRepo()

    render(<App />)
    const hang = await screen.findByTestId('wip-row')
    await act(async () => {
      hang.click()
    })
    expect(screen.getByTestId('commit-box')).toBeTruthy()

    await act(async () => {
      screen.getByLabelText('Đóng vùng soạn commit').click()
    })

    expect(screen.queryByTestId('commit-box')).toBeNull()
    expect(screen.queryByTestId('change-list')).toBeNull()
  })

  it('đổi repo → switchRepo chạy để xả nháp cũ rồi nạp nháp mới (WORK-08)', async () => {
    moRepo()
    const { rerender } = render(<App />)
    await vi.waitFor(() => expect(ipc.listRefs).toHaveBeenCalledWith('r1'))

    useCommitStore.setState({ draftByRepo: { r1: 'nua chung' } })

    await act(async () => {
      useRepoStore.setState({
        byRepo: {
          r1: { info: repoInfo('r1'), currentBranch: 'main', error: null },
          r2: { info: repoInfo('r2'), currentBranch: 'main', error: null },
        },
        activeRepoId: 'r2',
        isOpening: false,
        recent: [],
      })
      rerender(<App />)
    })

    // Nháp của r1 KHÔNG được rò sang r2 — tiêu chí thành công số 6.
    await vi.waitFor(() => {
      expect(useCommitStore.getState().draftByRepo.r2 ?? '').toBe('')
    })
    expect(useCommitStore.getState().draftByRepo.r1).toBe('nua chung')
  })
})
