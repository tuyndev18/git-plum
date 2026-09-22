/**
 * Cổng cờ perf của `SpikeHarness` — plan 03-01, Task 2.
 *
 * # Vì sao phải có test này chứ không "tin là nó không chạy"
 *
 * Cùng lập luận với T-02-25 trong `perf.ts`: một vòng đo chạy mãi trong bản release
 * là tự bắn vào chân mình, và "tôi tin là nó tắt" không phải bằng chứng. Ở đây điều
 * cần chứng minh mạnh hơn một chút — không chỉ là *không render*, mà là **không nạp
 * mô-đun CodeMirror**, vì mục tiêu của cổng là giữ bundle khởi động nhỏ.
 *
 * Nên test kiểm hai điều tách biệt:
 * 1. Cờ tắt → `App` không render `SpikeHarness`.
 * 2. `App.tsx` nạp `SpikeHarness` bằng `lazy(() => import(...))`, không nhập tĩnh —
 *    nhập tĩnh thì webpack/vite gộp CodeMirror vào bundle chính bất kể cờ, và điều
 *    đó **không** phát hiện được bằng cách render (component vẫn không hiện).
 *
 * Điều 2 kiểm bằng cách đọc mã nguồn `App.tsx`. Đó là phép kiểm duy nhất đúng: hình
 * dạng bundle không quan sát được từ trong một test happy-dom.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

import { App } from '@/App'
import { ipc } from '@/lib/ipc'
import { useRepoStore } from '@/stores/repoStore'
import { useHistoryStore } from '@/stores/historyStore'
import { useRefsStore } from '@/stores/refsStore'
import { useSelectionStore } from '@/stores/selectionStore'

/** Đọc `App.tsx` từ gốc dự án — `import.meta.url` không phải scheme `file:` dưới vitest. */
function docAppTsx(): string {
  return readFileSync(resolve(process.cwd(), 'src/App.tsx'), 'utf8')
}

// Chặn ở ranh giới module `@/lib/ipc`, cùng khuôn với `App.test.tsx` và mọi test
// khác trong dự án. `spikeBlobPair` có trong danh sách vì `SpikeHarness` gọi nó.
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

/*
 * Bề mặt vòng commit (Phase 4) **thêm vào** mock, không thay gì — cùng lý do và
 * cùng khuôn với `App.test.tsx`: tệp này render `App` trọn vẹn, nên nó phải mock
 * mọi thứ `App` đụng tới lúc gắn kết, kể cả những thứ không liên quan gì tới cờ perf
 * mà nó đang kiểm.
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
    spikeBlobPair: vi.fn(),
    getStatus: vi.fn(),
    stageFiles: vi.fn(),
    unstageFiles: vi.fn(),
    getWorktreeDiff: vi.fn(),
    createCommit: vi.fn(),
    amendCommit: vi.fn(),
  },
  ngheTrangThaiNgoai: vi.fn(async () => () => {}),
  describeError: (e: unknown) => String(e),
  isGitError: () => false,
}))

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

/**
 * happy-dom không có canvas 2d thật. `GraphCanvas` ném nếu `getContext('2d')` trả
 * `null`, và nhánh "có repo mở" của `App` render `GraphCanvas` — nên thiếu stub này
 * thì test thất bại vì một lý do không liên quan gì tới cờ perf.
 *
 * Sao lại nguyên khuôn `App.test.tsx`.
 */
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

/** Ảo hoá cuộn cần kích thước thật; happy-dom báo 0 cho mọi chiều. */
function stubViewportSize() {
  for (const [prop, value] of [
    ['offsetHeight', 600],
    ['offsetWidth', 800],
    ['clientHeight', 600],
    ['clientWidth', 800],
  ] as const) {
    Object.defineProperty(HTMLElement.prototype, prop, { configurable: true, value })
  }
}

/**
 * Mở sẵn một repository trong store.
 *
 * **Bắt buộc, không phải trang trí.** `SpikeHarness` nằm trong nhánh
 * `activeRepo ? … : …` của `App`, nên khi không có repo nào mở thì nó không hiện
 * **bất kể cờ perf**. Đã kiểm bằng đột biến: bỏ cổng `isPerfEnabled()` mà test vẫn
 * xanh — cổng vô dụng đúng như bài học "kiểm cổng có thể đỏ trước khi tin nó xanh".
 * Có repo mở thì cờ perf trở thành thứ **duy nhất** chặn render.
 */
function moSanRepo() {
  useRepoStore.setState({
    byRepo: {
      'repo-1': {
        info: { id: 'repo-1', path: 'C:/kho/repo-1', name: 'repo-1' },
        currentBranch: 'main',
        error: null,
      },
    },
    activeRepoId: 'repo-1',
  })
}

beforeEach(() => {
  vi.clearAllMocks()
  globalThis.localStorage?.clear()
  stubCanvasContext()
  stubViewportSize()
  useRepoStore.setState({ byRepo: {}, activeRepoId: null, isOpening: false, recent: [] })
  useHistoryStore.setState({ byRepo: {} })
  useRefsStore.setState({ byRepo: {} })
  useSelectionStore.setState({ selectedByRepo: {} })
  vi.mocked(ipc.gitVersion).mockResolvedValue('git version 2.54.0')
  vi.mocked(ipc.commandLog).mockResolvedValue([])
  vi.mocked(ipc.listRefs).mockResolvedValue([])
  vi.mocked(ipc.getCommitPage).mockResolvedValue({
    commits: [],
    graphRows: [],
    total: 0,
    skippedRecords: 0,
  })
})

describe('cổng cờ perf của SpikeHarness', () => {
  it('cờ tắt: không render SpikeHarness dù đã mở repository', async () => {
    // Không đặt `gitPlumPerf` — đây là cấu hình mặc định của người dùng thật.
    moSanRepo()
    render(<App />)

    // Chốt tiền đề trước: nếu `CommitList` cũng không hiện thì test không chứng
    // minh được gì về cờ perf, nó chỉ chứng minh `App` chưa render nhánh có repo.
    expect(
      screen.queryByTestId('commit-scroll'),
      'tiền đề: nhánh có repo phải được render, nếu không cổng perf không phải thứ đang chặn',
    ).not.toBeNull()

    // **Phải chờ.** `SpikeHarness` nạp qua `lazy()`, nên nó tới sau một microtask.
    // Một `queryByTestId` đồng bộ ngay sau `render` luôn trả `null` — kể cả khi cổng
    // đã bị bỏ — và cổng trở thành vô dụng. Đã kiểm bằng đột biến: với phép kiểm
    // đồng bộ, bỏ cổng `isPerfEnabled()` vẫn cho 5/5 test xanh.
    await expect(
      screen.findByTestId('spike-harness', {}, { timeout: 300 }),
    ).rejects.toThrow()
  })

  it('cờ bật: SpikeHarness được render', async () => {
    globalThis.localStorage.setItem('gitPlumPerf', '1')
    moSanRepo()
    render(<App />)

    // `lazy()` nên component tới ở một microtask sau.
    expect(await screen.findByTestId('spike-harness')).toBeTruthy()
  })

  it('cờ tắt là mặc định, không phải thứ phải tự đặt', () => {
    expect(globalThis.localStorage.getItem('gitPlumPerf')).toBeNull()
  })

  /**
   * **Mutation đã kiểm:** đổi `lazy(() => import(...))` thành `import ... from` tĩnh
   * trong `App.tsx` → test này đỏ. Đổi `isPerfEnabled() && (...)` thành `true && (...)`
   * → test đầu tiên ở trên đỏ. Hai cổng bắt hai lỗi khác nhau.
   */
  it('App.tsx nạp SpikeHarness bằng import động, không nhập tĩnh', () => {
    const src = docAppTsx()

    expect(
      src.includes("import('@/components/diff/SpikeHarness')"),
      'phải nạp bằng import() động để CodeMirror không vào bundle đường chính',
    ).toBe(true)

    // Không có dòng nhập tĩnh nào cho SpikeHarness. Regex bám vào dạng `import ...
    // from '...SpikeHarness'` ở đầu dòng, nên nó không khớp lời gọi `import()` động.
    const nhapTinh = /^import\s+.*\bfrom\s+['"][^'"]*SpikeHarness['"]/m
    expect(
      nhapTinh.test(src),
      'nhập tĩnh SpikeHarness sẽ gộp CodeMirror vào bundle chính bất kể cờ perf',
    ).toBe(false)
  })

  /**
   * Cổng phải đứng ở `App.tsx`, không chỉ ở trong `SpikeHarness`.
   *
   * Nếu cổng nằm *bên trong* component thì React vẫn phải nạp mô-đun để biết nó trả
   * `null` — tức CodeMirror vẫn được tải. Vị trí của cổng là cả nội dung của nó.
   */
  it('cổng isPerfEnabled đứng ở App.tsx, bao quanh SpikeHarness', () => {
    const src = docAppTsx()

    const viTriCong = src.indexOf('isPerfEnabled()')
    const viTriRender = src.indexOf('<SpikeHarness')

    expect(viTriCong).toBeGreaterThan(-1)
    expect(viTriRender).toBeGreaterThan(-1)
    expect(
      viTriCong < viTriRender,
      'cổng phải đứng TRƯỚC chỗ render, nếu không nó không chặn gì',
    ).toBe(true)
  })
})
