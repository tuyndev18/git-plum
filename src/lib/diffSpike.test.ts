/**
 * Test cho bộ phân tích hunk của đường B — plan 03-01, Task 2.
 *
 * # Tệp này KHÔNG khẳng định gì về thời gian
 *
 * happy-dom **không tính bố cục CSS** và không có cuộn thật, nên một con số ms đo
 * trong môi trường này không nói được điều gì về `MergeView` trên WebView2. Bài học
 * Phase 2: ba lỗi hiển thị (badge phá chiều cao hàng, đường vẽ tràn hàng, sai hệ quy
 * chiếu toạ độ canvas) **qua hết 212 test tự động**.
 *
 * Nên phân chia rõ: phép đo thời gian là việc của Task 3, trên bản dựng release,
 * trong WebView2 thật, trên một tệp lớn thật. Tệp này chỉ kiểm phần **thuần logic**
 * — bộ phân tích `@@` — vì đó là phần duy nhất kiểm được đúng đắn mà không cần bố cục.
 *
 * Một bộ phân tích hunk sai làm đường B đo ra con số **đẹp** (ít decoration hơn thực
 * tế) và như vậy phép so sánh A-hay-B bị lệch về phía B mà không ai thấy.
 */

import { describe, it, expect } from 'vitest'

import { phanTichHunk, type Hunk } from './diffSpike'

/**
 * Lấy hunk thứ `i` và khẳng định nó tồn tại.
 *
 * `noUncheckedIndexedAccess` bật nên truy cập chỉ mục cho `Hunk | undefined`. Dùng
 * hàm này thay vì `!`: nếu bộ phân tích trả ít hunk hơn mong đợi thì test thất bại
 * với thông điệp đọc được, chứ không ném `undefined` ở một dòng assert khác.
 */
function hunk(hunks: Hunk[], i: number): Hunk {
  const h = hunks[i]
  expect(h, `phải có hunk thứ ${i + 1}`).toBeDefined()
  return h as Hunk
}

/** Patch thật của `git diff --unified=3`, hai hunk, có cả thêm và xoá. */
const PATCH_HAI_HUNK = `diff --git a/a.js b/a.js
index 350be77..f5b7c83 100644
--- a/a.js
+++ b/a.js
@@ -1,6 +1,7 @@
 dong giu 1
 dong giu 2
-dong bi xoa
+dong moi a
+dong moi b
 dong giu 3
 dong giu 4
 dong giu 5
@@ -20,4 +21,4 @@ function f() {
 giu x
-cu y
+moi y
 giu z
`

describe('phanTichHunk', () => {
  it('đọc đúng hai hunk từ một patch hai hunk', () => {
    const hunks = phanTichHunk(PATCH_HAI_HUNK)

    expect(hunks).toHaveLength(2)
  })

  it('đọc đúng số dòng đầu của mỗi hunk từ đầu `@@`', () => {
    const hunks = phanTichHunk(PATCH_HAI_HUNK)
    const h1 = hunk(hunks, 0)
    const h2 = hunk(hunks, 1)

    // `@@ -1,6 +1,7 @@` — phía mới bắt đầu ở dòng 1.
    expect(h1.newStart).toBe(1)
    // `@@ -20,4 +21,4 @@` — phía mới bắt đầu ở dòng 21, KHÔNG phải 20. Lẫn hai con
    // số này là lỗi off-by-many kinh điển: decoration sẽ được gắn lệch một dòng và
    // trình xem diff tô sai dòng.
    expect(h2.newStart).toBe(21)
  })

  it('phân loại đúng dòng thêm và dòng xoá', () => {
    const h1 = hunk(phanTichHunk(PATCH_HAI_HUNK), 0)

    expect(h1.added).toEqual(['dong moi a', 'dong moi b'])
    expect(h1.removed).toEqual(['dong bi xoa'])
  })

  it('gán đúng số dòng phía mới cho từng dòng thêm', () => {
    const h1 = hunk(phanTichHunk(PATCH_HAI_HUNK), 0)

    // Hunk bắt đầu ở dòng 1: hai dòng ngữ cảnh (1, 2), rồi một dòng xoá (không tốn
    // số dòng phía mới), rồi hai dòng thêm ở dòng 3 và 4.
    expect(h1.addedLineNumbers).toEqual([3, 4])
  })

  it('KHÔNG đếm dòng xoá vào số dòng phía mới', () => {
    const h2 = hunk(phanTichHunk(PATCH_HAI_HUNK), 1)

    // `@@ -20,4 +21,4 @@`: một dòng ngữ cảnh (21), một dòng xoá (không tốn số),
    // rồi dòng thêm ở 22. Nếu cài đặt đếm cả dòng xoá thì ra 23 — lệch một dòng.
    expect(h2.addedLineNumbers).toEqual([22])
  })

  it('bỏ qua phần đầu tệp (`diff --git`, `index`, `---`, `+++`)', () => {
    const hunks = phanTichHunk(PATCH_HAI_HUNK)

    // `+++ b/a.js` bắt đầu bằng `+` nhưng KHÔNG phải một dòng thêm. Đọc nó thành
    // dòng thêm là lỗi mà mọi bộ phân tích diff viết tay đều mắc lần đầu.
    const moiDaDoc = hunks.flatMap((h) => h.added)
    expect(moiDaDoc).not.toContain('++ b/a.js')
    expect(moiDaDoc.some((l) => l.includes('b/a.js'))).toBe(false)

    // Tương tự `--- a/a.js` không phải dòng xoá.
    const cuDaDoc = hunks.flatMap((h) => h.removed)
    expect(cuDaDoc.some((l) => l.includes('a/a.js'))).toBe(false)
  })

  it('patch rỗng cho danh sách rỗng, không ném lỗi', () => {
    expect(phanTichHunk('')).toEqual([])
  })

  it('đọc được `@@` dạng không có dấu phẩy (hunk một dòng)', () => {
    // git in `@@ -5 +5 @@` khi hunk chỉ có một dòng — thiếu ca này thì một bộ phân
    // tích đòi dấu phẩy sẽ bỏ hẳn hunk đó trong im lặng.
    const hunks = phanTichHunk(`@@ -5 +5 @@\n-cu\n+moi\n`)

    expect(hunks).toHaveLength(1)
    const h = hunk(hunks, 0)
    expect(h.newStart).toBe(5)
    expect(h.added).toEqual(['moi'])
    expect(h.removed).toEqual(['cu'])
  })

  it('xử lý được CRLF — git trả `\\r\\n` trên Windows', () => {
    // Không trim `\r` thì dòng thêm mang một `\r` ở cuối và mọi phép so khớp nội
    // dung sau đó lệch. Cùng lớp lỗi với SHA mang `\r` mà CLAUDE.md cảnh báo.
    const h = hunk(phanTichHunk('@@ -1 +1 @@\r\n-cu\r\n+moi\r\n'), 0)

    expect(h.added).toEqual(['moi'])
    expect(h.removed).toEqual(['cu'])
  })
})
