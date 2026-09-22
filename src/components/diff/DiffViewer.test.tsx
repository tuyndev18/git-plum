/**
 * `DiffViewer` — bốn bất biến mà happy-dom **có thể** kiểm, và một ghi chú rõ
 * ràng về ba thứ nó **không** kiểm được.
 *
 * # Điều test này KHÔNG chứng minh được (đọc trước khi tin nó xanh)
 *
 * happy-dom không tính layout CSS và không có cuộn thật. Nên **không** test nào
 * ở đây nói được:
 *
 * - chữ có **hiển thị** hay bị co mất (lỗi `minmax(0, 2fr)` của Phase 2);
 * - hai cột có **thẳng hàng**;
 * - khoảng trắng có **hiện ra** trên màn hình;
 * - word-level có tô **đúng chỗ** về mặt pixel.
 *
 * Bốn điều đó là việc của checkpoint Task 3 (mắt người trên bản release). Cũng
 * vì vậy `shouldForceUnified(width)` được tách thành **hàm quyết định** thuần:
 * happy-dom không có `ResizeObserver` thật nên chỉ hàm đó test được, còn bề
 * rộng thật là việc của checkpoint.
 *
 * # Bốn bất biến test được
 *
 * 1. **Chống đua** — chọn tệp A rồi ngay tệp B, phản hồi A về **sau** B → hiện
 *    B. Cách sai cho ra "diff của tệp khác" mà **không lỗi nào** (T-03-31).
 * 2. **`view.destroy()` ở cleanup** — không có nó thì mỗi lần chọn tệp rò một
 *    `EditorView`, và người dùng bấm qua hàng chục tệp trong một phiên (T-03-28).
 * 3. **`Compartment` cho cờ khoảng trắng** — bật tắt **không dựng lại** view.
 *    Dựng lại làm mất vị trí cuộn, và người dùng bật cờ giữa lúc đang đọc.
 * 4. **Bốn dạng không phải `text` KHÔNG dựng `EditorView`** (T-03-29).
 */

import { act, cleanup, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { clearCommands } from '@/lib/commands'
import type { FileDiff } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import { useSelectionStore } from '@/stores/selectionStore'
import {
  DiffViewer,
  MIN_SPLIT_WIDTH,
  __diffViewerStatsForTest,
  shouldForceUnified,
} from './DiffViewer'

const getFileDiff = vi.fn<(r: string, c: string, p: string) => Promise<FileDiff>>()

vi.mock('@/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/ipc')>()
  return {
    ...actual,
    ipc: { ...actual.ipc, getFileDiff: (r: string, c: string, p: string) => getFileDiff(r, c, p) },
  }
})

function textDiff(path: string, content: string): FileDiff {
  return {
    path,
    oldPath: null,
    status: 'M',
    kind: {
      kind: 'text',
      truncated: false,
      hunks: [
        {
          oldStart: 1,
          oldCount: 1,
          newStart: 1,
          newCount: 1,
          heading: '',
          lines: [
            {
              kind: 'added',
              content,
              oldLine: null,
              newLine: 1,
              noNewlineAtEof: false,
              spans: [],
            },
          ],
        },
      ],
    },
  }
}

beforeEach(() => {
  clearCommands()
  getFileDiff.mockReset()
  __diffViewerStatsForTest.reset()
  useDiffStore.setState({
    selectedFileByRepo: {},
    viewMode: 'unified',
    showWhitespace: false,
  })
  useSelectionStore.setState({ selectedByRepo: { r: 'commit-1' } })
})

afterEach(() => {
  cleanup()
  clearCommands()
})

describe('shouldForceUnified — hàm QUYẾT ĐỊNH thuần (ResizeObserver là việc của checkpoint)', () => {
  it(`dưới ${MIN_SPLIT_WIDTH}px → buộc hợp nhất`, () => {
    expect(shouldForceUnified(300)).toBe(true)
    expect(shouldForceUnified(MIN_SPLIT_WIDTH - 1)).toBe(true)
  })

  it(`từ ${MIN_SPLIT_WIDTH}px trở lên → cho hai cột`, () => {
    expect(shouldForceUnified(MIN_SPLIT_WIDTH)).toBe(false)
    expect(shouldForceUnified(1400)).toBe(false)
  })

  it('bề rộng 0 (chưa đo được / panel chưa gắn) → buộc hợp nhất, không đoán', () => {
    // `ResizeObserver` chưa chạy lần đầu → 0. Đoán "rộng" ở đây sẽ nháy một
    // khung hai cột 0px rồi sửa lại.
    expect(shouldForceUnified(0)).toBe(true)
  })
})

describe('nạp diff theo (commitId, path) đang chọn', () => {
  it('chưa chọn tệp → không gọi getFileDiff, hiện gợi ý', () => {
    render(<DiffViewer repoId="r" />)
    expect(getFileDiff).not.toHaveBeenCalled()
    expect(screen.getByTestId('diff-empty')).toBeTruthy()
  })

  it('chọn tệp → gọi getFileDiff với đúng ba đối số', async () => {
    getFileDiff.mockResolvedValue(textDiff('a.ts', 'xin chao'))
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    render(<DiffViewer repoId="r" />)

    await waitFor(() => expect(getFileDiff).toHaveBeenCalledWith('r', 'commit-1', 'a.ts'))
  })

  it('đang chờ → hiện dấu hiệu nạp, KHÔNG hiện diff của tệp trước', async () => {
    let giaiPhong!: (v: FileDiff) => void
    getFileDiff.mockReturnValue(
      new Promise<FileDiff>((res) => {
        giaiPhong = res
      }),
    )
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    render(<DiffViewer repoId="r" />)

    expect(screen.getByTestId('diff-loading')).toBeTruthy()
    giaiPhong(textDiff('a.ts', 'xong'))
    await waitFor(() => expect(screen.queryByTestId('diff-loading')).toBeNull())
  })

  it('getFileDiff ném → hiện describeError (kèm lệnh git đã chạy, PLAT-10)', async () => {
    getFileDiff.mockRejectedValue({
      code: 'command_failed',
      message: 'git diff thất bại',
      command: 'git diff --unified=3 abc def -- a.ts',
    })
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    render(<DiffViewer repoId="r" />)

    const el = await waitFor(() => screen.getByTestId('diff-error'))
    expect(el.textContent).toContain('git diff thất bại')
    expect(el.textContent, 'describeError kèm lệnh đã chạy').toContain('git diff --unified=3')
  })
})

describe('🔴 cổng chống đua — phản hồi về sai thứ tự (T-03-31)', () => {
  it('🔴 chọn A rồi B, A về SAU B → hiện B, không hiện A', async () => {
    // Mutation #4 của Task 2: bỏ cổng này. Không có nó, giao diện hiện nội dung
    // của tệp khác mà KHÔNG lỗi nào — cực kỳ thường gặp khi bấm nhanh bằng
    // bàn phím.
    let giaiPhongA!: (v: FileDiff) => void
    let giaiPhongB!: (v: FileDiff) => void

    getFileDiff.mockImplementation((_r, _c, p) => {
      if (p === 'a.ts') return new Promise<FileDiff>((res) => (giaiPhongA = res))
      return new Promise<FileDiff>((res) => (giaiPhongB = res))
    })

    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    const view = render(<DiffViewer repoId="r" />)

    // Chọn B trước khi A về.
    useDiffStore.setState({ selectedFileByRepo: { r: 'b.ts' } })
    view.rerender(<DiffViewer repoId="r" />)

    // B về trước, rồi A về SAU.
    giaiPhongB(textDiff('b.ts', 'NOI DUNG CUA B'))
    // 🔴 Đọc `diff-content-path` — nó render từ `diff.path` (DỮ LIỆU đã về).
    // Lần viết đầu test này đọc `diff-path`, vốn render từ `selectedFile`
    // (TRẠNG THÁI store) nên nó hiện `b.ts` bất kể phản hồi nào đã ghi đè — và
    // mutation #4 cho **0 test đỏ**. Cổng đo sai đại lượng, đúng lớp lỗi mà
    // 03-02 gặp hai lần. Xem SUMMARY mục "cổng xanh sai".
    await waitFor(() =>
      expect(screen.getByTestId('diff-content-path').textContent).toBe('b.ts'),
    )

    giaiPhongA(textDiff('a.ts', 'NOI DUNG CUA A'))

    /*
     * Phải chờ React **thật sự flush** một lần render nữa, không chỉ chờ
     * microtask của promise.
     *
     * Ba `await Promise.resolve()` là **không đủ** và đó là lý do mutation #4
     * cho 0 test đỏ ở lần chạy thứ hai: React 19 gom `setDiff` từ một callback
     * promise vào một lượt render sau, nên ngay ở cài đặt ĐÃ BỊ đột biến, DOM
     * vẫn chưa kịp mang `a.ts` lúc `expect` chạy. Test xanh vì đo quá sớm — đúng
     * lớp lỗi "đo sai thời điểm" mà `measureFirstPaint` của Phase 2 đã dạy (đo
     * tới `useEffect` đầu tiên thay vì tới lúc có dữ liệu).
     *
     * `act()` bọc việc flush, rồi khẳng định trạng thái ĐÃ ỔN ĐỊNH.
     */
    await act(async () => {
      await new Promise((r) => setTimeout(r, 0))
    })

    expect(
      screen.getByTestId('diff-content-path').textContent,
      'nội dung đang hiện phải thuộc tệp ĐANG CHỌN (b.ts). Phản hồi của tệp không ' +
        'còn được chọn phải bị BỎ, không ghi đè — không có cổng này thì giao diện ' +
        'hiện diff của tệp khác mà KHÔNG lỗi nào (T-03-31).',
    ).toBe('b.ts')
  })
})

describe('🔴 view.destroy() ở cleanup — không rò EditorView (T-03-28)', () => {
  it('🔴 đổi tệp BA lần → số view còn sống không tăng theo số lần đổi', async () => {
    // Mutation #3 của Task 2: bỏ `view.destroy()`. `__diffViewerStatsForTest`
    // đếm số lần dựng và số lần destroy THẬT trong mã sản phẩm — không suy ra
    // từ DOM, vì happy-dom giữ lại node của view cũ theo cách khác.
    getFileDiff.mockImplementation((_r, _c, p) => Promise.resolve(textDiff(p, `noi dung ${p}`)))

    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    const view = render(<DiffViewer repoId="r" />)
    await waitFor(() => expect(__diffViewerStatsForTest.created).toBeGreaterThan(0))

    for (const p of ['b.ts', 'c.ts', 'd.ts']) {
      useDiffStore.setState({ selectedFileByRepo: { r: p } })
      view.rerender(<DiffViewer repoId="r" />)
      await waitFor(() => expect(screen.getByTestId('diff-content-path').textContent).toBe(p))
    }

    const { created, destroyed } = __diffViewerStatsForTest
    expect(created, 'phải dựng ít nhất bốn view cho bốn tệp').toBeGreaterThanOrEqual(4)
    expect(
      created - destroyed,
      `dựng ${created} view nhưng chỉ destroy ${destroyed} — mỗi lần chọn tệp rò một ` +
        `EditorView, và người dùng bấm qua hàng chục tệp trong một phiên`,
    ).toBeLessThanOrEqual(1)
  })

  it('unmount component → view cuối cùng cũng bị destroy', async () => {
    getFileDiff.mockResolvedValue(textDiff('a.ts', 'x'))
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    const view = render(<DiffViewer repoId="r" />)
    await waitFor(() => expect(__diffViewerStatsForTest.created).toBeGreaterThan(0))

    view.unmount()
    expect(__diffViewerStatsForTest.created - __diffViewerStatsForTest.destroyed).toBe(0)
  })
})

describe('🔴 Compartment cho cờ khoảng trắng — KHÔNG dựng lại view (DIFF-04)', () => {
  it('🔴 bật cờ khoảng trắng → CÙNG MỘT instance EditorView trước và sau', async () => {
    // Mutation #5 của Task 2: đổi `Compartment` sang dựng lại view. Dựng lại
    // làm MẤT vị trí cuộn, và người dùng bật cờ này giữa lúc đang đọc một chỗ
    // cụ thể — nên bước 8 của checkpoint hỏi đúng câu "vị trí cuộn có giữ
    // nguyên".
    getFileDiff.mockResolvedValue(textDiff('a.ts', 'co   khoang   trang'))
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    const view = render(<DiffViewer repoId="r" />)
    await waitFor(() => expect(__diffViewerStatsForTest.created).toBeGreaterThan(0))

    const truoc = __diffViewerStatsForTest.created
    const instanceTruoc = __diffViewerStatsForTest.lastInstance

    useDiffStore.setState({ showWhitespace: true })
    view.rerender(<DiffViewer repoId="r" />)
    await waitFor(() => expect(useDiffStore.getState().showWhitespace).toBe(true))

    expect(
      __diffViewerStatsForTest.created,
      'bật cờ khoảng trắng KHÔNG được dựng thêm EditorView nào — Compartment ' +
        'đổi extension tại chỗ',
    ).toBe(truoc)
    expect(
      __diffViewerStatsForTest.lastInstance,
      'phải là CÙNG MỘT instance EditorView',
    ).toBe(instanceTruoc)
  })

  it('đổi chế độ hợp nhất ↔ hai cột thì ĐƯỢC dựng lại (bố cục khác hẳn)', async () => {
    // Phân biệt rõ với ca trên: cờ khoảng trắng là một extension, còn chế độ
    // hai cột là một CÂY DOM khác. Dựng lại ở đây là đúng, không phải lỗi.
    getFileDiff.mockResolvedValue(textDiff('a.ts', 'x'))
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    const view = render(<DiffViewer repoId="r" />)
    await waitFor(() => expect(__diffViewerStatsForTest.created).toBeGreaterThan(0))

    const truoc = __diffViewerStatsForTest.created
    useDiffStore.setState({ viewMode: 'split' })
    view.rerender(<DiffViewer repoId="r" />)

    await waitFor(() =>
      expect(__diffViewerStatsForTest.created).toBeGreaterThanOrEqual(truoc),
    )
  })
})

describe('🔴 bốn dạng không phải text KHÔNG dựng EditorView (T-03-29)', () => {
  const kinds: FileDiff['kind'][] = [
    { kind: 'binary', oldSize: 4096, newSize: 8192 },
    { kind: 'tooLarge', size: 8_808_038, limit: 5_242_880 },
    { kind: 'lfsPointer', oid: 'ab'.repeat(32), size: 47_185_920 },
    { kind: 'unchanged' },
  ]

  for (const kind of kinds) {
    it(`dạng ${kind.kind}: hiện thông báo, KHÔNG dựng EditorView`, async () => {
      getFileDiff.mockResolvedValue({ path: 'x', oldPath: null, status: 'M', kind })
      useDiffStore.setState({ selectedFileByRepo: { r: 'x' } })
      render(<DiffViewer repoId="r" />)

      await waitFor(() => expect(screen.getByTestId('diff-notice')).toBeTruthy())
      expect(
        __diffViewerStatsForTest.created,
        `dạng ${kind.kind} không được dựng EditorView — nội dung của nó không ` +
          `bao giờ được đổ ra màn hình`,
      ).toBe(0)
    })
  }

  it('dạng text bị cắt: hiện băng cảnh báo VÀ dựng view (người dùng xem một phần)', async () => {
    const d = textDiff('a.ts', 'x')
    if (d.kind.kind === 'text') d.kind.truncated = true
    getFileDiff.mockResolvedValue(d)
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    render(<DiffViewer repoId="r" />)

    await waitFor(() => expect(screen.getByTestId('diff-truncated')).toBeTruthy())
    expect(__diffViewerStatsForTest.created).toBeGreaterThan(0)
  })
})

describe('nhảy khối — giao diện NÓI RA khi hết khối (DIFF-03)', () => {
  it('diff một khối: bấm "khối sau" hiện thông báo hết khối, không nhảy vòng', async () => {
    getFileDiff.mockResolvedValue(textDiff('a.ts', 'x'))
    useDiffStore.setState({ selectedFileByRepo: { r: 'a.ts' } })
    render(<DiffViewer repoId="r" />)
    // Chờ tới lúc trình xem THẬT SỰ được dựng: `diff-path` render ngay từ khung
    // đầu (nó chỉ đọc `selectedFile`), nên chờ nó là chờ sai thứ — và
    // `lineMetaRef` còn rỗng thì `nhayKhoi` thoát sớm và không đặt thông báo nào.
    await waitFor(() => expect(__diffViewerStatsForTest.created).toBeGreaterThan(0))

    const { runCommand } = await import('@/lib/commands')
    await runCommand('diff.nextHunk')

    await waitFor(() => expect(screen.getByTestId('diff-hunk-status')).toBeTruthy())
    expect(screen.getByTestId('diff-hunk-status').textContent).toMatch(/khối cuối|hết khối/i)
  })
})
