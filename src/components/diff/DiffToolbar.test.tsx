/**
 * `DiffToolbar` (bốn nút, tất cả qua `runCommand` — PLAT-04) và **năm dạng thông
 * báo DIFF-06**.
 *
 * Hai điều đáng nói về cách các test này được viết:
 *
 * 1. **Không dùng `grep -c 'runCommand'`.** `<verification>` của plan nêu rõ lý
 *    do: `runCommand` đã có trong nhiều tệp cho việc khác, nên phép đếm chuỗi
 *    xanh cả khi `onClick` gắn hàm trực tiếp. Thay bằng render thật +
 *    `fireEvent.click` + spy trên lệnh đã đăng ký.
 * 2. **Thông báo `lfsPointer` khẳng định con số là `kind.size`**, không phải cỡ
 *    tệp con trỏ. 03-02 đã tách hai số này (~130 byte cho tệp con trỏ, 45 MB cho
 *    nội dung thật) và mutation #4 của Task 1 là trộn chúng lại.
 */

import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { clearCommands, getCommand, listCommands } from '@/lib/commands'
import type { DiffKind } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import { DiffNotice, DiffToolbar, formatBytes } from './DiffToolbar'

beforeEach(() => {
  clearCommands()
  useDiffStore.setState({
    selectedFileByRepo: {},
    viewMode: 'unified',
    showWhitespace: false,
  })
})

afterEach(() => {
  cleanup()
  clearCommands()
})

describe('formatBytes — Intl.NumberFormat, không tự chia tay', () => {
  it('byte thô dưới 1 KB', () => {
    expect(formatBytes(132)).toMatch(/132/)
  })

  it('MB có phần thập phân một chữ số', () => {
    // 1.258.291 byte ≈ 1,2 MB
    expect(formatBytes(1_258_291)).toMatch(/1[.,]2\s*MB/)
  })

  it('5.242.880 (ngưỡng DIFF-06) hiện là 5,0 MB', () => {
    expect(formatBytes(5_242_880)).toMatch(/5[.,]0\s*MB/)
  })

  it('0 byte là ca hợp lệ (tệp bị xoá có phía mới 0)', () => {
    expect(formatBytes(0)).toMatch(/0/)
  })
})

describe('DiffToolbar — bốn nút, TẤT CẢ đi qua runCommand (PLAT-04)', () => {
  it('đăng ký đúng bốn lệnh khi mount', () => {
    render(<DiffToolbar />)
    const ids = listCommands().map((c) => c.id).sort()
    expect(ids).toEqual(
      [
        'diff.nextHunk',
        'diff.prevHunk',
        'diff.toggleViewMode',
        'diff.toggleWhitespace',
      ].sort(),
    )
  })

  it('🔴 gỡ đăng ký khi unmount — mount lần hai KHÔNG ném', () => {
    // `registerCommand` ném khi trùng id, nên thiếu bước gỡ làm component vỡ ở
    // lần mount thứ hai — trong test là từ test thứ hai trở đi.
    const a = render(<DiffToolbar />)
    a.unmount()
    expect(listCommands()).toHaveLength(0)
    expect(() => render(<DiffToolbar />)).not.toThrow()
  })

  it('🔴 bấm nút "chế độ" gọi runCommand, không gọi hàm gắn thẳng', () => {
    render(<DiffToolbar />)
    // Thay `run` của lệnh đã đăng ký bằng spy: nếu `onClick` gắn hàm trực tiếp
    // (không qua sổ đăng ký) thì spy KHÔNG được gọi.
    const cmd = getCommand('diff.toggleViewMode')!
    const spy = vi.fn()
    cmd.run = spy

    fireEvent.click(screen.getByTestId('diff-toggle-view-mode'))
    expect(spy).toHaveBeenCalledTimes(1)
  })

  it('🔴 bấm nút "khoảng trắng" gọi runCommand', () => {
    render(<DiffToolbar />)
    const spy = vi.fn()
    getCommand('diff.toggleWhitespace')!.run = spy

    fireEvent.click(screen.getByTestId('diff-toggle-whitespace'))
    expect(spy).toHaveBeenCalledTimes(1)
  })

  it('🔴 bấm nút "khối kế tiếp" và "khối trước" gọi runCommand', () => {
    render(<DiffToolbar />)
    const next = vi.fn()
    const prev = vi.fn()
    getCommand('diff.nextHunk')!.run = next
    getCommand('diff.prevHunk')!.run = prev

    fireEvent.click(screen.getByTestId('diff-next-hunk'))
    fireEvent.click(screen.getByTestId('diff-prev-hunk'))
    expect(next).toHaveBeenCalledTimes(1)
    expect(prev).toHaveBeenCalledTimes(1)
  })

  it('lệnh chế độ thật sự đổi viewMode trong store', () => {
    render(<DiffToolbar />)
    fireEvent.click(screen.getByTestId('diff-toggle-view-mode'))
    expect(useDiffStore.getState().viewMode).toBe('split')
  })

  it('lệnh khoảng trắng thật sự bật cờ trong store', () => {
    render(<DiffToolbar />)
    fireEvent.click(screen.getByTestId('diff-toggle-whitespace'))
    expect(useDiffStore.getState().showWhitespace).toBe(true)
  })

  it('nhãn nút chế độ phản ánh chế độ đang dùng', () => {
    /*
     * Đọc `aria-label`, không phải `textContent`: nút nay chỉ có icon, và
     * `<svg>` trong `icons.tsx` mang `aria-hidden` nên `textContent` là chuỗi
     * rỗng. Chữ sống ở `aria-label` + `title`.
     *
     * Phép khẳng định vẫn giữ nguyên giá trị nó vốn có — nhãn phải **đổi theo
     * trạng thái**, không đứng yên. Bỏ chữ khỏi giao diện không được phép bỏ
     * luôn cái cổng đó, vì nút chỉ có icon mà nhãn sai thì người dùng bàn phím
     * không còn đường nào khác để biết nút làm gì.
     */
    render(<DiffToolbar />)
    const nut = () => screen.getByTestId('diff-toggle-view-mode')
    expect(nut().getAttribute('aria-label')).toMatch(/hai cột/i)
    fireEvent.click(nut())
    expect(nut().getAttribute('aria-label')).toMatch(/hợp nhất/i)
  })

  it('nút chỉ-icon PHẢI có cả title lẫn aria-label, và hai chuỗi phải khớp nhau', () => {
    /*
     * 🔴 Lớp lỗi im lặng của việc bỏ chữ: quên một trong hai thuộc tính thì
     * giao diện **trông vẫn đúng**.
     *
     * - Thiếu `title` → người dùng chuột thấy một nút câu đố, không có tooltip.
     * - Thiếu `aria-label` → nút **câm hoàn toàn** với trình đọc màn hình, vì
     *   `<svg>` có `aria-hidden` nên không còn nguồn chữ nào.
     *
     * Và vì nhãn đổi theo trạng thái, hai thuộc tính phải đọc **cùng** biểu
     * thức — lệch nhau thì tooltip nói một đằng, trình đọc màn hình đọc một nẻo,
     * mà không màn hình nào lộ ra điều đó.
     */
    render(<DiffToolbar />)
    const ids = [
      'diff-toggle-view-mode',
      'diff-toggle-whitespace',
      'diff-toggle-file-history',
      'diff-prev-hunk',
      'diff-next-hunk',
    ]
    for (const id of ids) {
      const el = screen.getByTestId(id)
      const label = el.getAttribute('aria-label')
      const title = el.getAttribute('title')
      expect(label, `${id} thiếu aria-label — nút chỉ-icon sẽ câm với trình đọc màn hình`).toBeTruthy()
      expect(title, `${id} thiếu title — người dùng chuột không có tooltip`).toBeTruthy()
      expect(el.textContent, `${id} phải là nút chỉ-icon, không còn chữ`).toBe('')
      expect(title, `${id}: title và aria-label lệch nhau`).toBe(label)
    }
  })
})

describe('DiffNotice — năm dạng, NĂM thông báo khác nhau (DIFF-06)', () => {
  it('binary: nêu CẢ HAI kích thước, KHÔNG có nút "hiện dù sao"', () => {
    const kind: DiffKind = { kind: 'binary', oldSize: 1_258_291, newSize: 1_572_864 }
    render(<DiffNotice kind={kind} />)

    const el = screen.getByTestId('diff-notice')
    expect(el.textContent).toMatch(/nhị phân/i)
    expect(el.textContent).toMatch(/1[.,]2\s*MB/)
    expect(el.textContent).toMatch(/1[.,]5\s*MB/)
    // Không có đường nào để người dùng ép hiện byte thô (T-03-29).
    expect(screen.queryByRole('button')).toBeNull()
  })

  it('🔴 tooLarge: nêu CẢ kích thước thật LẪN ngưỡng', () => {
    // Nêu một số mà thiếu số kia thì người dùng không biết mình cách ngưỡng
    // bao xa — chỉ biết "quá lớn".
    const kind: DiffKind = { kind: 'tooLarge', size: 8_808_038, limit: 5_242_880 }
    render(<DiffNotice kind={kind} />)

    const text = screen.getByTestId('diff-notice').textContent ?? ''
    expect(text).toMatch(/8[.,]4\s*MB/)
    expect(text, 'phải nêu cả NGƯỠNG 5,0 MB').toMatch(/5[.,]0\s*MB/)
  })

  it('🔴 lfsPointer: con số là kind.size (nội dung thật), KHÔNG phải cỡ tệp con trỏ', () => {
    // Mutation #4 của Task 1: đổi sang hiện ~130 byte của chính tệp con trỏ.
    const kind: DiffKind = { kind: 'lfsPointer', oid: 'ab'.repeat(32), size: 47_185_920 }
    render(<DiffNotice kind={kind} />)

    const text = screen.getByTestId('diff-notice').textContent ?? ''
    expect(text).toMatch(/LFS/i)
    expect(text, 'phải hiện 45,0 MB — số TRONG con trỏ').toMatch(/45[.,]0\s*MB/)
    // Và khẳng định phủ định: không được hiện con số ~130 byte nào.
    expect(text, 'không được hiện kích thước tệp con trỏ (~130 byte)').not.toMatch(
      /\b1[23]\d\s*(B|byte)/i,
    )
    // oid viết ngắn, không dán cả 64 ký tự.
    expect(text).toContain('abababab')
    expect(text, 'oid phải viết NGẮN').not.toContain('ab'.repeat(32))
  })

  it('unchanged: thông báo RIÊNG, khác text-0-hunk và khác lỗi', () => {
    render(<DiffNotice kind={{ kind: 'unchanged' }} />)
    const text = screen.getByTestId('diff-notice').textContent ?? ''
    expect(text).toMatch(/không thay đổi nội dung/i)
    expect(text).toMatch(/quyền/i)
  })

  it('text: DiffNotice trả null — nội dung do DiffViewer dựng, không phải thông báo', () => {
    const { container } = render(
      <DiffNotice kind={{ kind: 'text', hunks: [], truncated: false, contextOnly: false }} />,
    )
    expect(container.textContent).toBe('')
  })

  it('🔴 bốn dạng cho bốn chuỗi KHÁC NHAU — không dùng chung một câu chung chung', () => {
    const kinds: DiffKind[] = [
      { kind: 'binary', oldSize: 1, newSize: 2 },
      { kind: 'tooLarge', size: 9_000_000, limit: 5_242_880 },
      { kind: 'lfsPointer', oid: 'cd'.repeat(32), size: 1_048_576 },
      { kind: 'unchanged' },
    ]
    const texts = kinds.map((k) => {
      const r = render(<DiffNotice kind={k} />)
      const t = screen.getByTestId('diff-notice').textContent ?? ''
      r.unmount()
      return t
    })
    expect(new Set(texts).size, `bốn thông báo phải khác nhau, thấy: ${texts.join(' | ')}`).toBe(4)
  })

  it('mỗi thông báo có role="status" để trình đọc màn hình đọc được', () => {
    render(<DiffNotice kind={{ kind: 'unchanged' }} />)
    expect(screen.getByRole('status')).toBeTruthy()
  })
})

describe('băng cảnh báo truncated — hiện CÙNG nội dung, không thay nó', () => {
  it('truncated: true → có băng cảnh báo', () => {
    render(<DiffNotice kind={{ kind: 'text', hunks: [], truncated: true, contextOnly: false }} />)
    // `DiffNotice` cho `text` không dựng thông báo chính, nhưng băng `truncated`
    // là ngoại lệ: người dùng đang xem MỘT PHẦN và phải biết điều đó.
    const el = screen.getByTestId('diff-truncated')
    expect(el.textContent).toMatch(/một phần|đã bị cắt|cắt/i)
  })

  it('truncated: false → KHÔNG có băng', () => {
    render(<DiffNotice kind={{ kind: 'text', hunks: [], truncated: false, contextOnly: false }} />)
    expect(screen.queryByTestId('diff-truncated')).toBeNull()
  })
})

/*
 * 🔴 `contextOnly` — cờ "đang xem dạng rút gọn", thêm cùng việc hiện TOÀN TỆP.
 *
 * Backend mặc định gửi `--unified=1000000` (toàn tệp) nhưng lùi về `--unified=3`
 * khi tệp vượt `MAX_BYTE_TOAN_TEP` (512 KB), và đặt `contextOnly: true` khi lùi.
 *
 * # Vì sao cờ này BẮT BUỘC phải hiện ra
 *
 * Im lặng lùi về rút gọn chính là việc đã xảy ra ngày 2026-09-22: người dùng
 * thấy số dòng nhảy (5 → 24 → 37), tưởng trình xem lỗi, và **báo hai lần**. Một
 * cờ đúng ở backend mà giao diện bỏ qua thì không sửa được gì — nó chỉ chuyển
 * lỗi im lặng từ tầng này sang tầng khác.
 *
 * # `contextOnly` KHÁC `truncated` — hai câu khác nhau
 *
 * | cờ | nghĩa | người dùng mất gì |
 * |---|---|---|
 * | `truncated` | bản vá **bị cắt** | **có** mất thay đổi |
 * | `contextOnly` | thấy đủ mọi thay đổi, thiếu ngữ cảnh giữa | **không** mất gì |
 *
 * Nên `contextOnly` **không** được dùng chữ "bị cắt" và không được dùng `⚠️`:
 * một cảnh báo sai cho tệp 600 KB hoàn toàn bình thường làm người dùng mất tin
 * vào trình xem. Có test ghim đúng điều đó.
 */
describe('băng contextOnly — nói rõ đang xem rút gọn, KHÔNG gọi là "bị cắt"', () => {
  it('contextOnly: true → có băng riêng, testid riêng', () => {
    render(
      <DiffNotice kind={{ kind: 'text', hunks: [], truncated: false, contextOnly: true }} />,
    )

    const el = screen.getByTestId('diff-context-only')
    expect(
      el,
      'thiếu băng này thì việc lùi về rút gọn là IM LẶNG — đúng lỗi người dùng đã ' +
        'báo hai lần ngày 2026-09-22',
    ).toBeTruthy()
  })

  it('contextOnly: false → KHÔNG có băng', () => {
    render(
      <DiffNotice kind={{ kind: 'text', hunks: [], truncated: false, contextOnly: false }} />,
    )

    expect(screen.queryByTestId('diff-context-only')).toBeNull()
  })

  it('🔴 KHÔNG dùng chữ "cắt" — người dùng không mất thay đổi nào', () => {
    render(
      <DiffNotice kind={{ kind: 'text', hunks: [], truncated: false, contextOnly: true }} />,
    )

    const chu = screen.getByTestId('diff-context-only').textContent ?? ''
    expect(
      chu,
      'contextOnly nghĩa là thấy ĐỦ mọi thay đổi, chỉ thiếu ngữ cảnh không đổi. ' +
        'Gọi nó là "bị cắt" là một cảnh báo SAI cho một tệp hoàn toàn bình thường.',
    ).not.toMatch(/bị cắt|cắt bớt/)
  })

  it('nói rõ LÝ DO (tệp lớn) để người dùng biết đây không phải lỗi', () => {
    render(
      <DiffNotice kind={{ kind: 'text', hunks: [], truncated: false, contextOnly: true }} />,
    )

    const chu = screen.getByTestId('diff-context-only').textContent ?? ''
    expect(
      chu,
      'không nói lý do thì người dùng vẫn phải tự đoán vì sao số dòng nhảy — ' +
        'đúng chỗ họ đã đoán sai một lần',
    ).toMatch(/lớn|dung lượng|kích thước/i)
  })

  it('hai cờ ĐỘC LẬP: cùng true thì hiện CẢ HAI băng', () => {
    /*
     * Một tệp lớn **và** có quá nhiều khối đổi là ca thật, và hai câu nói hai
     * điều khác nhau. Gộp hoặc cho một cờ đè cờ kia sẽ giấu mất một nửa sự thật.
     */
    render(
      <DiffNotice kind={{ kind: 'text', hunks: [], truncated: true, contextOnly: true }} />,
    )

    expect(screen.getByTestId('diff-truncated')).toBeTruthy()
    expect(screen.getByTestId('diff-context-only')).toBeTruthy()
  })
})
