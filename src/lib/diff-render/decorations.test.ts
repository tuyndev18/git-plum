/**
 * `decorations.ts` — hàm **thuần**, test được không cần layout.
 *
 * Đây là lý do tầng này tách khỏi `.tsx`: happy-dom không tính layout CSS, nên
 * mọi logic phải nằm ở chỗ test được *thật*. Cái gì còn lại trong `.tsx` thì
 * chỉ mắt người hoặc Chromium thật trả lời được (checkpoint Task 3).
 *
 * # Mutation then chốt của plan: byte → UTF-16
 *
 * `Span` từ wave 3 mang chỉ số **byte**; CodeMirror đánh chỉ số theo **UTF-16
 * code unit**. Trên `'xéy dỏng TEST'` thì `TEST` ở **byte 12** nhưng **UTF-16
 * index 9** — dùng thẳng con số của Rust tô lệch trên **mọi** dòng tiếng Việt,
 * và tiếng Việt có ở khắp nơi trong dự án này.
 *
 * `ipc.diff.test.ts` (wave 3) đã ghim cả phép chuyển đúng lẫn phép cắt sai. Test
 * ở đây là cổng cho **tầng vẽ**, và plan tự nêu điều kiện của nó: **nội dung
 * phải ngoài ASCII**, nếu không mutation bỏ phép chuyển sẽ không đỏ. Hai ca dưới
 * đây dùng tiếng Việt thật và một emoji ngoài BMP (4 byte / 2 đơn vị UTF-16).
 *
 * `TextEncoder`, **không** `Buffer`: mã sản phẩm là browser-only (không có
 * `tauri-plugin-fs`), xem chú thích cùng nội dung trong `app.css.test.ts`.
 */

import { describe, expect, it } from 'vitest'

import type { DiffLine, Hunk, Span } from '@/lib/ipc'
import {
  buildDecorations,
  hunksToDoc,
  nextHunkLine,
  prevHunkLine,
  spanToUtf16,
} from './decorations'

/** Dựng một `DiffLine` gọn cho test. */
function dl(
  kind: DiffLine['kind'],
  content: string,
  oldLine: number | null,
  newLine: number | null,
  spans: Span[] = [],
): DiffLine {
  return { kind, content, oldLine, newLine, noNewlineAtEof: false, spans }
}

function hunk(lines: DiffLine[], overrides: Partial<Hunk> = {}): Hunk {
  return {
    oldStart: 1,
    oldCount: lines.filter((l) => l.oldLine !== null).length,
    newStart: 1,
    newCount: lines.filter((l) => l.newLine !== null).length,
    heading: '',
    lines,
    ...overrides,
  }
}

describe('🔴 spanToUtf16 — chuyển chỉ số BYTE sang đơn vị UTF-16', () => {
  it('ASCII thuần: hai hệ trùng nhau (nên ca này KHÔNG chứng minh được gì)', () => {
    const content = 'if (a == b)'
    const byteStart = new TextEncoder().encode(content.slice(0, content.indexOf('=='))).length
    const chuyen = spanToUtf16(content, { start: byteStart, end: byteStart + 2 })
    expect(content.slice(chuyen!.from, chuyen!.to)).toBe('==')
    // Ghi rõ vì sao ca này không đủ: hai hệ đếm bằng nhau nên mutation bỏ phép
    // chuyển vẫn xanh ở đây. Các ca dưới mới là cổng.
    expect(chuyen!.from).toBe(byteStart)
  })

  it('🔴 tiếng Việt thật: byte 12 ≠ UTF-16 index 9', () => {
    // Đúng ca mà `ipc.diff.test.ts` ghim: `é` 2 byte, `ỏ` 3 byte.
    const content = 'xéy dỏng TEST'
    const byteStart = new TextEncoder().encode(content.slice(0, content.indexOf('TEST'))).length
    expect(byteStart, 'tiền đề: TEST phải ở byte 12').toBe(12)

    const chuyen = spanToUtf16(content, { start: byteStart, end: byteStart + 4 })
    expect(chuyen).not.toBeNull()
    expect(chuyen!.from, 'UTF-16 index của TEST phải là 9').toBe(9)
    expect(content.slice(chuyen!.from, chuyen!.to)).toBe('TEST')

    // Và khẳng định hai hệ THẬT SỰ khác nhau ở ca này — nếu không, test vô nghĩa
    // và mutation bỏ phép chuyển sẽ xanh.
    expect(chuyen!.from).not.toBe(byteStart)
  })

  it('🔴 emoji ngoài BMP: 4 byte nhưng 2 đơn vị UTF-16', () => {
    const content = 'x🙂y CHANGED'
    const byteStart = new TextEncoder().encode(content.slice(0, content.indexOf('CHANGED'))).length
    // 'x' 1 + '🙂' 4 + 'y' 1 + ' ' 1 = 7 byte, nhưng UTF-16: 1 + 2 + 1 + 1 = 5.
    expect(byteStart).toBe(7)

    const chuyen = spanToUtf16(content, { start: byteStart, end: byteStart + 7 })
    expect(chuyen!.from).toBe(5)
    expect(content.slice(chuyen!.from, chuyen!.to)).toBe('CHANGED')
    expect(chuyen!.from).not.toBe(byteStart)
  })

  it('span phủ CẢ DÒNG vẫn chuyển được, không ném', () => {
    const content = 'cả dòng đổi'
    const byteLen = new TextEncoder().encode(content).length
    const chuyen = spanToUtf16(content, { start: 0, end: byteLen })
    expect(chuyen).toEqual({ from: 0, to: content.length })
  })

  it('span RỖNG (start === end) → bỏ, không dựng mark rộng 0', () => {
    expect(spanToUtf16('abc', { start: 1, end: 1 })).toBeNull()
  })

  it('🔴 dữ liệu backend LỖI: start > end → bỏ span, KHÔNG ném (T-03-24)', () => {
    // CodeMirror **ném** khi range vượt biên document, và một ngoại lệ ở tầng
    // này làm TRẮNG cả panel.
    expect(() => spanToUtf16('abcdef', { start: 4, end: 2 })).not.toThrow()
    expect(spanToUtf16('abcdef', { start: 4, end: 2 })).toBeNull()
  })

  it('🔴 dữ liệu backend LỖI: end vượt độ dài byte → bỏ span, KHÔNG ném (T-03-24)', () => {
    expect(() => spanToUtf16('abc', { start: 0, end: 999 })).not.toThrow()
    expect(spanToUtf16('abc', { start: 0, end: 999 })).toBeNull()
    expect(spanToUtf16('abc', { start: 100, end: 200 })).toBeNull()
  })

  it('offset âm → bỏ span', () => {
    expect(spanToUtf16('abc', { start: -1, end: 2 })).toBeNull()
  })
})

describe('hunksToDoc — dựng tài liệu cho từng phía', () => {
  const h = hunk([
    dl('context', 'giu nguyen', 1, 1),
    dl('removed', 'cu', 2, null),
    dl('added', 'moi', null, 2),
    dl('context', 'cuoi', 3, 3),
  ])

  it("side 'unified': MỌI dòng, đúng thứ tự git in", () => {
    const { text, lineMeta } = hunksToDoc([h], 'unified')
    expect(text.split('\n')).toEqual(['giu nguyen', 'cu', 'moi', 'cuoi'])
    expect(lineMeta.map((m) => m.kind)).toEqual(['context', 'removed', 'added', 'context'])
  })

  it("side 'old': chỉ dòng có oldLine (context + removed)", () => {
    const { text, lineMeta } = hunksToDoc([h], 'old')
    expect(text.split('\n')).toEqual(['giu nguyen', 'cu', 'cuoi'])
    expect(lineMeta.map((m) => m.oldLine)).toEqual([1, 2, 3])
    expect(lineMeta.every((m) => m.newLine === null || m.kind === 'context')).toBe(true)
  })

  it("side 'new': chỉ dòng có newLine (context + added)", () => {
    const { text, lineMeta } = hunksToDoc([h], 'new')
    expect(text.split('\n')).toEqual(['giu nguyen', 'moi', 'cuoi'])
    expect(lineMeta.map((m) => m.newLine)).toEqual([1, 2, 3])
  })

  it('lineMeta LUÔN cùng độ dài với số dòng của text', () => {
    // Cùng bất biến mà `CommitPage.graphRows` phải giữ: lệch một phần tử là
    // decoration gắn lệch hàng.
    for (const side of ['unified', 'old', 'new'] as const) {
      const { text, lineMeta } = hunksToDoc([h], side)
      expect(lineMeta.length, `side=${side}`).toBe(text.split('\n').length)
    }
  })

  it('🔴 lineMeta mang spans đã chuyển sang UTF-16, không phải byte thô', () => {
    const content = 'nếu (x == y)'
    const byteStart = new TextEncoder().encode(content.slice(0, content.indexOf('=='))).length
    const utf16Start = content.indexOf('==')
    expect(byteStart, 'tiền đề: hai hệ phải khác nhau ở dòng này').not.toBe(utf16Start)

    const h2 = hunk([dl('added', content, null, 1, [{ start: byteStart, end: byteStart + 2 }])])
    const { lineMeta } = hunksToDoc([h2], 'new')

    expect(lineMeta[0]!.spans).toEqual([{ from: utf16Start, to: utf16Start + 2 }])
    // Và khẳng định nó KHÔNG mang con số byte — đây là cổng của mutation #1.
    expect(lineMeta[0]!.spans[0]!.from).not.toBe(byteStart)
  })

  it('spans RỖNG là ca bình thường — lineMeta có mảng rỗng, không undefined', () => {
    const { lineMeta } = hunksToDoc([h], 'unified')
    for (const m of lineMeta) {
      expect(Array.isArray(m.spans)).toBe(true)
      expect(m.spans).toHaveLength(0)
    }
  })

  it('nhiều hunk: dòng nối tiếp nhau, hunkIndex tăng', () => {
    const a = hunk([dl('added', 'a1', null, 1)], { newStart: 1 })
    const b = hunk([dl('added', 'b1', null, 40)], { newStart: 40 })
    const { text, lineMeta } = hunksToDoc([a, b], 'new')

    expect(text.split('\n')).toEqual(['a1', 'b1'])
    expect(lineMeta.map((m) => m.hunkIndex)).toEqual([0, 1])
  })

  it('hunks rỗng → tài liệu rỗng, lineMeta rỗng, không ném', () => {
    const { text, lineMeta } = hunksToDoc([], 'unified')
    expect(text).toBe('')
    expect(lineMeta).toEqual([])
  })

  it('dòng RỖNG trong ngữ cảnh giữ nguyên là dòng rỗng', () => {
    // Fixture `empty-line.txt` của 03-02: dòng ngữ cảnh rỗng là ca thật.
    const h2 = hunk([dl('context', '', 1, 1), dl('added', 'x', null, 2)])
    const { text, lineMeta } = hunksToDoc([h2], 'unified')
    expect(text.split('\n')).toEqual(['', 'x'])
    expect(lineMeta).toHaveLength(2)
  })
})

describe('buildDecorations — line-deco cho nền dòng, mark-deco cho từng span', () => {
  it('🔴 dòng CÓ span: có CẢ line-deco LẪN ≥1 mark-deco', () => {
    const content = 'nếu (x == y)'
    const byteStart = new TextEncoder().encode(content.slice(0, content.indexOf('=='))).length
    const h = hunk([dl('added', content, null, 1, [{ start: byteStart, end: byteStart + 2 }])])
    const { text, lineMeta } = hunksToDoc([h], 'new')

    const { lineCount, markCount } = buildDecorations(text, lineMeta)
    expect(lineCount, 'nền dòng thêm').toBe(1)
    expect(markCount, 'một mark cho span word-level').toBe(1)
  })

  it('🔴 dòng KHÔNG span: chỉ có line-deco, không mark nào', () => {
    const h = hunk([dl('added', 'ca dong nay deu moi', null, 1)])
    const { text, lineMeta } = hunksToDoc([h], 'new')

    const { lineCount, markCount } = buildDecorations(text, lineMeta)
    expect(lineCount).toBe(1)
    expect(markCount, 'spans rỗng → tô CẢ DÒNG, suy giảm đúng').toBe(0)
  })

  it('dòng ngữ cảnh: KHÔNG line-deco (không tô nền), không mark', () => {
    const h = hunk([dl('context', 'khong doi', 1, 1)])
    const { text, lineMeta } = hunksToDoc([h], 'unified')
    const { lineCount, markCount } = buildDecorations(text, lineMeta)
    expect(lineCount).toBe(0)
    expect(markCount).toBe(0)
  })

  it('đếm ĐÚNG số deco cho một lineMeta đã biết (không grep "Decoration")', () => {
    // `<verification>` nêu rõ vì sao không dùng `grep -c 'Decoration'`: nó khớp
    // cả `import` lẫn doc comment. Cổng đúng là đếm deco THẬT.
    const h = hunk([
      dl('context', 'a', 1, 1), // 0 line, 0 mark
      dl('removed', 'b', 2, null), // 1 line, 0 mark
      dl('added', 'B', null, 2, [{ start: 0, end: 1 }]), // 1 line, 1 mark
      dl('added', 'C', null, 3), // 1 line, 0 mark
    ])
    const { text, lineMeta } = hunksToDoc([h], 'unified')
    const { lineCount, markCount, set } = buildDecorations(text, lineMeta)

    expect(lineCount).toBe(3)
    expect(markCount).toBe(1)
    expect(set.size, 'tổng deco = 3 line + 1 mark').toBe(4)
  })

  it('🔴 span LỖI từ backend không làm ném — panel không trắng (T-03-24)', () => {
    const h = hunk([
      dl('added', 'abc', null, 1, [
        { start: 0, end: 999 }, // vượt biên
        { start: 3, end: 1 }, // đảo ngược
        { start: 1, end: 2 }, // hợp lệ
      ]),
    ])
    const { text, lineMeta } = hunksToDoc([h], 'new')

    expect(() => buildDecorations(text, lineMeta)).not.toThrow()
    const { markCount } = buildDecorations(text, lineMeta)
    expect(markCount, 'chỉ span hợp lệ được dựng').toBe(1)
  })

  it('nhiều span trong một dòng → nhiều mark', () => {
    const h = hunk([
      dl('added', 'aa bb cc', null, 1, [
        { start: 0, end: 2 },
        { start: 6, end: 8 },
      ]),
    ])
    const { text, lineMeta } = hunksToDoc([h], 'new')
    expect(buildDecorations(text, lineMeta).markCount).toBe(2)
  })

  it('tài liệu rỗng → DecorationSet rỗng, không ném', () => {
    expect(() => buildDecorations('', [])).not.toThrow()
    expect(buildDecorations('', []).set.size).toBe(0)
  })
})

describe('🔴 nhảy khối — null khi HẾT, không nhảy vòng im lặng (DIFF-03)', () => {
  /** Ba hunk, mỗi hunk một dòng ngữ cảnh rồi một dòng thêm. */
  function baKhoi() {
    const mk = (n: number) =>
      hunk([dl('context', `ctx${n}`, n, n), dl('added', `add${n}`, null, n + 1)])
    return hunksToDoc([mk(1), mk(10), mk(20)], 'unified').lineMeta
  }

  it('nextHunkLine từ dòng 1 → dòng đầu của khối 2', () => {
    const meta = baKhoi()
    // Sáu dòng: [h0 ctx, h0 add, h1 ctx, h1 add, h2 ctx, h2 add]
    expect(meta.map((m) => m.hunkIndex)).toEqual([0, 0, 1, 1, 2, 2])
    expect(nextHunkLine(meta, 1)).toBe(3)
  })

  it('nextHunkLine từ giữa khối 2 → dòng đầu của khối 3', () => {
    expect(nextHunkLine(baKhoi(), 4)).toBe(5)
  })

  it('🔴 ở khối CUỐI → nextHunkLine trả NULL (không nhảy vòng về đầu)', () => {
    // Mutation #2 của Task 2: trả về khối đầu. `null` là hợp đồng để giao diện
    // nói "đây là khối cuối"; DIFF-03 nói "kế tiếp và trước đó", không nói "vòng".
    expect(nextHunkLine(baKhoi(), 5)).toBeNull()
    expect(nextHunkLine(baKhoi(), 6)).toBeNull()
  })

  it('🔴 ở khối ĐẦU → prevHunkLine trả NULL', () => {
    expect(prevHunkLine(baKhoi(), 1)).toBeNull()
    expect(prevHunkLine(baKhoi(), 2)).toBeNull()
  })

  it('prevHunkLine từ khối 3 → dòng đầu của khối 2', () => {
    expect(prevHunkLine(baKhoi(), 5)).toBe(3)
  })

  it('🔴 diff MỘT khối → cả hai hàm trả null NGAY', () => {
    const meta = hunksToDoc([hunk([dl('added', 'x', null, 1)])], 'new').lineMeta
    expect(nextHunkLine(meta, 1)).toBeNull()
    expect(prevHunkLine(meta, 1)).toBeNull()
  })

  it('lineMeta rỗng → null, không ném', () => {
    expect(nextHunkLine([], 1)).toBeNull()
    expect(prevHunkLine([], 1)).toBeNull()
  })

  it('🔴 hai khối LIỀN NHAU (không dòng ngữ cảnh giữa) vẫn là HAI khối', () => {
    // Một cài đặt "tìm dòng thêm/xoá kế tiếp" sẽ GỘP chúng thành một khối và
    // `nextHunkLine` trả `null` ngay ở khối đầu.
    const a = hunk([dl('added', 'a', null, 1)], { newStart: 1 })
    const b = hunk([dl('added', 'b', null, 2)], { newStart: 2 })
    const meta = hunksToDoc([a, b], 'new').lineMeta

    expect(meta.map((m) => m.hunkIndex)).toEqual([0, 1])
    expect(nextHunkLine(meta, 1), 'hai hunk liền nhau vẫn nhảy được').toBe(2)
    expect(prevHunkLine(meta, 2)).toBe(1)
  })

  it('currentLine ngoài biên (0 hay quá lớn) không ném', () => {
    const meta = baKhoi()
    expect(() => nextHunkLine(meta, 0)).not.toThrow()
    expect(() => prevHunkLine(meta, 9999)).not.toThrow()
    expect(nextHunkLine(meta, 0)).toBe(1)
  })
})
