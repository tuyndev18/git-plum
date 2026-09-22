/**
 * `FileHistory` — DIFF-05.
 *
 * # Điều test này KHÔNG chứng minh được (đọc trước khi tin nó xanh)
 *
 * happy-dom không tính layout CSS. Nên không test nào ở đây nói được rằng bốn cột
 * của một hàng phiên bản **thẳng hàng**, rằng tiêu đề commit dài bị cắt kèm ellipsis
 * thay vì biến mất, hay rằng panel lịch sử không đẩy diff ra khỏi khung. Ba lỗi hiển
 * thị của Phase 2 qua hết 212 test tự động; những điều đó là việc của cổng thoát.
 *
 * Cổng cấp hai cho bố cục là `app.css.test.ts` (đọc nguồn CSS), giống 03-04.
 *
 * # 🔴 `await Promise.resolve()` KHÔNG flush render của React 19
 *
 * Bài học 03-04, khiếm khuyết B của mutation #8: ba `await Promise.resolve()` liên
 * tiếp **không** đủ để React 19 flush một `setState` gọi từ callback của promise, nên
 * test chống đua xanh **kể cả trên cài đặt đã bị đột biến** — nó đo quá sớm. Mọi chỗ
 * cần DOM đã cập nhật ở đây dùng `act()` hoặc `waitFor()`.
 */

import { act, cleanup, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { clearCommands, getCommand, runCommand } from '@/lib/commands'
import type { FileHistory as FileHistoryData, FileVersion } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import { useSelectionStore } from '@/stores/selectionStore'

import { FileHistory, MAX_FILE_HISTORY, formatVersionTime } from './FileHistory'

const getFileHistory = vi.fn<(r: string, p: string) => Promise<FileHistoryData>>()

vi.mock('@/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/ipc')>()
  return {
    ...actual,
    ipc: { ...actual.ipc, getFileHistory: (r: string, p: string) => getFileHistory(r, p) },
  }
})

const REPO = 'repo-1'

function phienBan(over: Partial<FileVersion> = {}): FileVersion {
  return {
    commitId: 'a'.repeat(40),
    authorName: 'tuyenpn',
    authorTime: 1790000000,
    subject: 'sửa một thứ',
    status: 'M',
    path: 'a.txt',
    oldPath: null,
    ...over,
  }
}

function lichSu(versions: FileVersion[], truncated = false): FileHistoryData {
  return { path: 'a.txt', versions, truncated }
}

/** Mở panel với một tệp đang chọn. `selectFile` tự đóng panel, nên thứ tự quan trọng. */
function moPanel(path = 'a.txt') {
  useDiffStore.getState().selectFile(REPO, path)
  useDiffStore.getState().openHistory()
}

beforeEach(() => {
  clearCommands()
  getFileHistory.mockReset()
  getFileHistory.mockResolvedValue(lichSu([phienBan()]))
  useDiffStore.setState({
    selectedFileByRepo: {},
    viewMode: 'unified',
    showWhitespace: false,
    historyOpen: false,
    historyCommitOverride: null,
  })
  useSelectionStore.setState({ selectedByRepo: {} })
})

afterEach(() => {
  cleanup()
  clearCommands()
})

describe('hiện danh sách phiên bản', () => {
  it('có tệp đang chọn và panel mở → hiện ngày, tác giả, tiêu đề, mã ngắn', async () => {
    getFileHistory.mockResolvedValue(
      lichSu([
        phienBan({ commitId: 'b'.repeat(40), subject: 'thay đổi thứ hai', authorName: 'ai_dev1' }),
      ]),
    )
    moPanel()
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-${'b'.repeat(40)}`)).toBeTruthy())

    const hang = screen.getByTestId(`file-version-${'b'.repeat(40)}`)
    expect(hang.textContent).toContain('ai_dev1')
    expect(hang.textContent).toContain('thay đổi thứ hai')
    // Mã commit **viết ngắn**: tám ký tự đầu, không phải cả 40.
    expect(hang.textContent).toContain('bbbbbbbb')
    expect(hang.textContent).not.toContain('b'.repeat(40))
  })

  it('panel đóng → không render gì', () => {
    useDiffStore.getState().selectFile(REPO, 'a.txt')
    render(<FileHistory repoId={REPO} />)
    expect(screen.queryByTestId('file-history')).toBeNull()
  })

  it('không có tệp đang chọn → không render, và không gọi IPC', async () => {
    useDiffStore.getState().openHistory()
    render(<FileHistory repoId={REPO} />)

    await act(async () => {
      await new Promise((r) => setTimeout(r, 0))
    })

    expect(screen.queryByTestId('file-history')).toBeNull()
    expect(getFileHistory).not.toHaveBeenCalled()
  })

  it('🔴 KHÔNG hiện 1970 cho một timestamp hợp lệ', async () => {
    // `authorTime` là **giây** Unix; `new Date(giây)` cho một ngày năm 1970. Ca này
    // nằm trong checklist checkpoint của 02-06 — một lỗi đã được lường trước.
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([phienBan({ authorTime: 1790000000 })]))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId('file-history')).toBeTruthy())

    const panel = screen.getByTestId('file-history')
    expect(panel.textContent).not.toContain('1970')
    expect(panel.textContent).toContain('2026')
  })

  it('formatVersionTime nhân 1000, và hai cách thật sự khác nhau', () => {
    // Khẳng định thứ hai là cả điểm: nếu hai cách cho cùng kết quả thì khẳng định
    // thứ nhất không chứng minh gì.
    expect(formatVersionTime(1790000000)).toContain('2026')
    expect(new Date(1790000000).getUTCFullYear()).toBe(1970)
  })
})

describe('bấm một phiên bản', () => {
  it('🔴 đặt historyCommitOverride, KHÔNG đổi selectedCommitId trên đồ thị', async () => {
    const SHA_DO_THI = 'c'.repeat(40)
    const SHA_PHIEN_BAN = 'd'.repeat(40)

    useSelectionStore.getState().select(REPO, SHA_DO_THI)
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([phienBan({ commitId: SHA_PHIEN_BAN })]))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-${SHA_PHIEN_BAN}`)).toBeTruthy())

    act(() => {
      screen.getByTestId(`file-version-${SHA_PHIEN_BAN}`).click()
    })

    expect(useDiffStore.getState().historyCommitOverride).toBe(SHA_PHIEN_BAN)
    expect(useSelectionStore.getState().selectedByRepo[REPO]).toBe(SHA_DO_THI)
  })

  it('phiên bản đang xem được đánh dấu aria-current', async () => {
    const SHA = 'e'.repeat(40)
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([phienBan({ commitId: SHA })]))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-${SHA}`)).toBeTruthy())
    expect(screen.getByTestId(`file-version-${SHA}`).getAttribute('aria-current')).toBe('false')

    act(() => {
      screen.getByTestId(`file-version-${SHA}`).click()
    })

    expect(screen.getByTestId(`file-version-${SHA}`).getAttribute('aria-current')).toBe('true')
  })
})

describe('chỗ đổi tên hiện ra', () => {
  it('🔴 phiên bản có oldPath → hiện "đổi tên từ <tên cũ>"', async () => {
    const SHA = 'f'.repeat(40)
    moPanel('renamed-new.txt')
    getFileHistory.mockResolvedValue(
      lichSu([
        phienBan({
          commitId: SHA,
          status: 'R077',
          path: 'renamed-new.txt',
          oldPath: 'renamed.txt',
        }),
      ]),
    )
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-rename-${SHA}`)).toBeTruthy())

    const nhan = screen.getByTestId(`file-version-rename-${SHA}`)
    expect(nhan.textContent).toContain('đổi tên từ')
    expect(nhan.textContent).toContain('renamed.txt')
  })

  it('phiên bản KHÔNG đổi tên → không có nhãn đổi tên', async () => {
    const SHA = '1'.repeat(40)
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([phienBan({ commitId: SHA, oldPath: null })]))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-${SHA}`)).toBeTruthy())
    expect(screen.queryByTestId(`file-version-rename-${SHA}`)).toBeNull()
  })
})

describe('ba tình huống rỗng là BA câu khác nhau', () => {
  it('truncated → nói "200 phiên bản gần nhất"', async () => {
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([phienBan()], true))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId('file-history-truncated')).toBeTruthy())
    expect(screen.getByTestId('file-history-truncated').textContent).toContain(
      String(MAX_FILE_HISTORY),
    )
  })

  it('danh sách rỗng → "Tệp này không có trong lịch sử", KHÁC ca lỗi và ca đang nạp', async () => {
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([]))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId('file-history-empty')).toBeTruthy())
    expect(screen.queryByTestId('file-history-error')).toBeNull()
    expect(screen.queryByTestId('file-history-loading')).toBeNull()
  })

  it('IPC lỗi → hiện lỗi, KHÔNG hiện "không có trong lịch sử"', async () => {
    moPanel()
    getFileHistory.mockRejectedValue(new Error('git đã chết'))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId('file-history-error')).toBeTruthy())
    expect(screen.queryByTestId('file-history-empty')).toBeNull()
  })
})

describe('cổng chống đua khi đổi tệp (T-03-37)', () => {
  it('🔴 phản hồi của tệp CŨ về sau KHÔNG được ghi lên danh sách của tệp MỚI', async () => {
    const SHA_CU = '2'.repeat(40)
    const SHA_MOI = '3'.repeat(40)

    let giaiPhongCu: (v: FileHistoryData) => void = () => {}
    const choCu = new Promise<FileHistoryData>((res) => {
      giaiPhongCu = res
    })

    getFileHistory.mockImplementation((_r, p) => {
      if (p === 'cu.ts') return choCu
      return Promise.resolve({
        path: 'moi.ts',
        versions: [phienBan({ commitId: SHA_MOI, path: 'moi.ts', subject: 'của tệp MỚI' })],
        truncated: false,
      })
    })

    moPanel('cu.ts')
    const { rerender } = render(<FileHistory repoId={REPO} />)

    // Đổi sang tệp mới trong lúc lời gọi của tệp cũ CHƯA về. `selectFile` đóng panel
    // (lớp phòng thủ thứ nhất), nên mở lại ngay — đó đúng là ca mà cổng chống đua
    // phải chặn, và là ca người dùng gặp khi bấm nhanh bằng bàn phím.
    await act(async () => {
      useDiffStore.getState().selectFile(REPO, 'moi.ts')
      useDiffStore.getState().openHistory()
      await new Promise((r) => setTimeout(r, 0))
    })
    rerender(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-${SHA_MOI}`)).toBeTruthy())

    // Giờ mới cho phản hồi của tệp CŨ về.
    await act(async () => {
      giaiPhongCu({
        path: 'cu.ts',
        versions: [phienBan({ commitId: SHA_CU, path: 'cu.ts', subject: 'của tệp CŨ' })],
        truncated: false,
      })
      await new Promise((r) => setTimeout(r, 0))
    })

    const panel = screen.getByTestId('file-history')
    expect(panel.textContent).toContain('của tệp MỚI')
    expect(panel.textContent).not.toContain('của tệp CŨ')
    expect(screen.queryByTestId(`file-version-${SHA_CU}`)).toBeNull()
    // Và tiêu đề khớp tệp đang chọn — nhãn đọc `selectedFile`, nên nó là bằng chứng
    // "danh sách đang hiện thuộc tệp nào" chỉ khi hai khẳng định trên cũng đúng.
    expect(screen.getByTestId('file-history-path').textContent).toBe('moi.ts')
  })

  it('đổi tệp trong lúc panel mở → danh sách nạp lại cho tệp MỚI', async () => {
    moPanel('cu.ts')
    const { rerender } = render(<FileHistory repoId={REPO} />)
    await waitFor(() => expect(getFileHistory).toHaveBeenCalledWith(REPO, 'cu.ts'))

    await act(async () => {
      useDiffStore.getState().selectFile(REPO, 'moi.ts')
      useDiffStore.getState().openHistory()
      await new Promise((r) => setTimeout(r, 0))
    })
    rerender(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(getFileHistory).toHaveBeenCalledWith(REPO, 'moi.ts'))
  })
})

describe('lệnh đi qua sổ đăng ký (PLAT-04)', () => {
  it('đăng ký diff.toggleFileHistory, và mọi nút đi qua runCommand', async () => {
    moPanel()
    render(<FileHistory repoId={REPO} />)
    await waitFor(() => expect(screen.getByTestId('file-history')).toBeTruthy())

    expect(getCommand('diff.toggleFileHistory')).toBeDefined()

    // Nút đóng chạy LỆNH, không một handler riêng: thay `run` bằng spy và bấm nút.
    const spy = vi.fn()
    const lenh = getCommand('diff.toggleFileHistory')!
    lenh.run = spy

    act(() => {
      screen.getByTestId('file-history-close').click()
    })

    await waitFor(() => expect(spy).toHaveBeenCalled())
  })

  it('lệnh `enabled` là false khi chưa chọn tệp → runCommand không làm gì', async () => {
    render(<FileHistory repoId={REPO} />)

    const lenh = getCommand('diff.toggleFileHistory')!
    expect(lenh.enabled?.()).toBe(false)

    await runCommand('diff.toggleFileHistory')
    expect(useDiffStore.getState().historyOpen).toBe(false)
  })

  it('lệnh bật tắt: gọi lần một mở, lần hai đóng', async () => {
    useDiffStore.getState().selectFile(REPO, 'a.txt')
    render(<FileHistory repoId={REPO} />)

    await act(async () => {
      await runCommand('diff.toggleFileHistory')
    })
    expect(useDiffStore.getState().historyOpen).toBe(true)

    await act(async () => {
      await runCommand('diff.toggleFileHistory')
    })
    expect(useDiffStore.getState().historyOpen).toBe(false)
  })

  it('🔴 gỡ đăng ký khi unmount — mount HAI lần không ném', async () => {
    // `registerCommand` ném khi trùng id, nên thiếu `unregisterCommand` ở cleanup
    // làm component vỡ ở lần mount thứ hai. Test mount hai lần là cách duy nhất
    // quan sát được điều đó.
    moPanel()
    const lan1 = render(<FileHistory repoId={REPO} />)
    await waitFor(() => expect(getCommand('diff.toggleFileHistory')).toBeDefined())

    lan1.unmount()
    expect(getCommand('diff.toggleFileHistory')).toBeUndefined()

    expect(() => render(<FileHistory repoId={REPO} />)).not.toThrow()
    await waitFor(() => expect(getCommand('diff.toggleFileHistory')).toBeDefined())
  })
})

describe('đóng panel trả DiffViewer về commit trên đồ thị', () => {
  it('closeHistory xoá historyCommitOverride', async () => {
    const SHA = '4'.repeat(40)
    moPanel()
    getFileHistory.mockResolvedValue(lichSu([phienBan({ commitId: SHA })]))
    render(<FileHistory repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId(`file-version-${SHA}`)).toBeTruthy())
    act(() => {
      screen.getByTestId(`file-version-${SHA}`).click()
    })
    expect(useDiffStore.getState().historyCommitOverride).toBe(SHA)

    act(() => {
      useDiffStore.getState().closeHistory()
    })

    expect(useDiffStore.getState().historyOpen).toBe(false)
    expect(useDiffStore.getState().historyCommitOverride).toBeNull()
  })
})
