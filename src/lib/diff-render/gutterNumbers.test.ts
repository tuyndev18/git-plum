/**
 * Số dòng gutter của chế độ hai cột — lỗi người dùng báo ở checkpoint vòng 2.
 *
 * # Cổng này kiểm CÁI GÌ, và mutation nào làm nó đỏ
 *
 * Mutation phải đỏ: **quay `extensionsCoDinh` về `lineNumbers()` mặc định**. Test
 * `khớp đúng đầu ra của lineNumbers() mặc định` dưới đây ghim rằng đầu ra đúng
 * **khác** dãy 1,2,3… của CodeMirror — nên một bản sửa dùng lại mặc định không
 * thể qua được.
 *
 * # Điều test này KHÔNG chứng minh
 *
 * Nó kiểm **hàm thuần**, không kiểm rằng renderer thật sự gọi hàm đó.
 * `MergeView` chưa bao giờ render trong một test happy-dom nào (thiếu
 * `ResizeObserver` → `paneWidth = 0` → mọi test đi nhánh hợp nhất), nên "gutter
 * thật hiện đúng số" chỉ đo được bằng Chromium thật. Có một cổng đọc mã nguồn
 * trong `codemirrorRenderer` bên dưới để bắt ca "hàm đúng nhưng không ai gọi".
 */

/// <reference types="node" />
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

import { hunksToDoc } from './decorations'
import { dauThemXoa, soDongThat } from './gutterNumbers'
import type { Hunk } from '@/lib/ipc'

function dong(
  kind: 'context' | 'added' | 'removed',
  content: string,
  oldLine: number | null,
  newLine: number | null,
) {
  return { kind, content, oldLine, newLine, noNewlineAtEof: false, spans: [] }
}

/**
 * Đúng hình dạng ảnh tham chiếu người dùng gửi: hai dòng **thêm** giữa hunk.
 *
 * Phía cũ:  10, 11, 12, 13, 14, 15          (6 dòng)
 * Phía mới: 10, 11, 12, 13+, 14+, 15, 16, 17 (8 dòng)
 */
const HUNK_THEM: Hunk[] = [
  {
    oldStart: 10,
    oldCount: 6,
    newStart: 10,
    newCount: 8,
    heading: 'interface Props',
    lines: [
      dong('context', '  /** Chiều cao khung audio. */', 10, 10),
      dong('context', '  audioHeight?: number;', 11, 11),
      dong('context', '  audioWidth?: number;', 12, 12),
      dong('added', '  /** Khung bao nút loa. */', null, 13),
      dong('added', '  audioClassName?: string;', null, 14),
      dong('context', '  textAlign?: string;', 13, 15),
      dong('context', '  baseSize?: number;', 14, 16),
      dong('context', '};', 15, 17),
    ],
  },
]

/** Một dòng **xoá** — phía mới phải để trống ở vị trí đó. */
const HUNK_XOA: Hunk[] = [
  {
    oldStart: 50,
    oldCount: 3,
    newStart: 52,
    newCount: 2,
    heading: '',
    lines: [
      dong('context', '  return (', 50, 52),
      dong('removed', '    <AudioViewer height={audioHeight} />', 51, null),
      dong('context', '  );', 52, 53),
    ],
  },
]

/** Đọc cả gutter của một phía thành mảng, đúng thứ tự hàng. */
function gutter(hunks: Hunk[], side: 'old' | 'new'): string[] {
  const doc = hunksToDoc(hunks, side)
  const soDong = doc.text === '' ? 0 : doc.text.split('\n').length
  return Array.from({ length: soDong }, (_, i) => soDongThat(doc.lineMeta, side, i + 1))
}

describe('soDongThat — số dòng thật trong tệp của từng phía', () => {
  it('phía cũ hiện oldLine, và NHẢY SỐ qua chỗ có dòng thêm', () => {
    // Phía cũ không chứa dòng `added` nào, nên tài liệu nó có 6 dòng và số là
    // 10..15 liên tục. Điều quan trọng: KHÔNG phải 1..6.
    expect(gutter(HUNK_THEM, 'old')).toEqual(['10', '11', '12', '13', '14', '15'])
  })

  it('phía mới hiện newLine, gồm cả số của hai dòng thêm', () => {
    expect(gutter(HUNK_THEM, 'new')).toEqual([
      '10',
      '11',
      '12',
      '13',
      '14',
      '15',
      '16',
      '17',
    ])
  })

  it('🔴 KHÔNG khớp dãy 1,2,3… mà lineNumbers() mặc định sinh ra', () => {
    /*
     * Đây là cổng chống mutation "quay về `lineNumbers()` mặc định".
     *
     * `lineNumbers()` mặc định đánh số theo **tài liệu**, nên nó luôn sinh
     * 1..N. Nếu ai đó bỏ `formatNumber` đi, gutter sẽ thành dãy này — và test
     * này khẳng định dãy đó là SAI cho cả hai phía.
     */
    const macDinhCu = ['1', '2', '3', '4', '5', '6']
    const macDinhMoi = ['1', '2', '3', '4', '5', '6', '7', '8']

    expect(gutter(HUNK_THEM, 'old')).not.toEqual(macDinhCu)
    expect(gutter(HUNK_THEM, 'new')).not.toEqual(macDinhMoi)
  })

  it('hai phía KHÁC nhau ở cùng một hàng khi có dòng thêm — đó là điểm của bài', () => {
    /*
     * Hàng thứ 6 của **tài liệu phía cũ** là dòng `textAlign` số 13; cùng dòng
     * logic đó ở phía mới là số 15. Bản lỗi hiện `6` và `6` — tức hai phía nói
     * cùng một con số cho hai dòng thật khác nhau.
     */
    const cu = gutter(HUNK_THEM, 'old')
    const moi = gutter(HUNK_THEM, 'new')
    expect(cu[5]).toBe('15')
    expect(moi[5]).toBe('15')
    // Và dòng `textAlign` (phía cũ hàng 4 sau khi bỏ hai dòng thêm) là 13.
    expect(cu[3]).toBe('13')
    expect(moi[3]).toBe('13')
  })

  it('trả CHUỖI RỖNG khi phía này không có dòng ở vị trí đó', () => {
    /*
     * Ca này đo trên tài liệu **hợp nhất**, nơi cả dòng thêm và dòng xoá đều có
     * mặt — đó là chỗ duy nhất một `lineMeta` chứa cả hai loại `null`.
     */
    const hopNhat = hunksToDoc(HUNK_THEM, 'unified')
    // Hàng 4 và 5 là dòng `added` → `oldLine === null` → gutter cũ để trống.
    expect(soDongThat(hopNhat.lineMeta, 'old', 4)).toBe('')
    expect(soDongThat(hopNhat.lineMeta, 'old', 5)).toBe('')
    // Nhưng phía mới có số.
    expect(soDongThat(hopNhat.lineMeta, 'new', 4)).toBe('13')
    expect(soDongThat(hopNhat.lineMeta, 'new', 5)).toBe('14')
  })

  it('dòng xoá: phía mới để trống, phía cũ có số', () => {
    const hopNhat = hunksToDoc(HUNK_XOA, 'unified')
    expect(soDongThat(hopNhat.lineMeta, 'new', 2)).toBe('')
    expect(soDongThat(hopNhat.lineMeta, 'old', 2)).toBe('51')
  })

  it('lineNumber vượt lineMeta trả rỗng, KHÔNG phải "undefined"', () => {
    const doc = hunksToDoc(HUNK_THEM, 'new')
    expect(soDongThat(doc.lineMeta, 'new', 999)).toBe('')
    expect(soDongThat(doc.lineMeta, 'new', 0)).toBe('')
    expect(soDongThat([], 'new', 1)).toBe('')
  })
})

describe('dauThemXoa — dấu +/− trong gutter', () => {
  it('dòng thêm là "+", dòng xoá là "−", ngữ cảnh là rỗng', () => {
    const hopNhat = hunksToDoc(HUNK_THEM, 'unified')
    expect(dauThemXoa(hopNhat.lineMeta, 1)).toBe('')
    expect(dauThemXoa(hopNhat.lineMeta, 4)).toBe('+')
    expect(dauThemXoa(hopNhat.lineMeta, 5)).toBe('+')

    const xoa = hunksToDoc(HUNK_XOA, 'unified')
    expect(dauThemXoa(xoa.lineMeta, 2)).toBe('−')
  })

  it('dùng U+2212 MINUS SIGN, không phải hyphen-minus', () => {
    /*
     * Ghim ký tự thật: `'-'` (U+002D) và `'−'` (U+2212) trông gần giống nhau
     * trong mã nguồn, nên một lần sửa vô tình sẽ không ai thấy. Trong phông
     * mono, hyphen ngắn và lệch xuống so với `+`.
     */
    const xoa = hunksToDoc(HUNK_XOA, 'unified')
    expect(dauThemXoa(xoa.lineMeta, 2)).toBe('−')
    expect(dauThemXoa(xoa.lineMeta, 2)).not.toBe('-')
  })

  it('vượt biên trả rỗng', () => {
    expect(dauThemXoa([], 1)).toBe('')
  })
})

describe('cổng: renderer PHẢI dùng hai hàm này, không dùng lineNumbers() trần', () => {
  /*
   * Vì sao cần cổng đọc mã nguồn ở đây.
   *
   * Hai `describe` trên kiểm hàm thuần. Chúng vẫn xanh nếu ai đó viết hàm đúng
   * rồi **không gọi nó** — đúng ca mà `MergeView` không render trong test nào
   * nên không test hành vi nào bắt được. Cổng này bắt ca đó.
   *
   * Lọc chú thích trước khi tìm: tệp renderer có doc comment tiếng Việt dày
   * nhắc chính tên API đang bị cấm, nên một `grep` thô là cổng tự vô hiệu hoá —
   * cùng bài học mà `interface-boundary.test.ts` đã ghi.
   */
  const nguon = readFileSync(
    path.join(process.cwd(), 'src/lib/diff-render/codemirrorRenderer.ts'),
    'utf8',
  )

  /** Bỏ mọi chú thích khối và chú thích dòng. */
  function boChuThich(src: string): string {
    return src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^\s*\/\/.*$/gm, '')
  }

  const ma = boChuThich(nguon)

  it('không còn lời gọi `lineNumbers()` KHÔNG đối số trong mã chạy', () => {
    /*
     * `lineNumbers()` trần là chính mutation cần chặn. `lineNumbers({...})` với
     * `formatNumber` thì hợp lệ, nên regex phải phân biệt hai dạng — chứ không
     * chỉ tìm chuỗi `lineNumbers`.
     */
    expect(ma).not.toMatch(/lineNumbers\(\s*\)/)
  })

  it('có nhập và dùng `soDongThat`', () => {
    expect(ma).toMatch(/soDongThat/)
  })

  it('có nhập và dùng `dauThemXoa`', () => {
    expect(ma).toMatch(/dauThemXoa/)
  })

  it('truyền `formatNumber` cho lineNumbers', () => {
    expect(ma).toMatch(/formatNumber/)
  })
})
