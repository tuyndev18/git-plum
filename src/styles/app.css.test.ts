/**
 * Test hồi quy checkpoint round 1 — cột `.commit-subject` KHÔNG BAO GIỜ được
 * co về 0px.
 *
 * Bối cảnh: `minmax(0, 2fr)` cho cột subject để nó tự do co giãn theo tỉ lệ —
 * đúng ý định ban đầu, nhưng sai ở chỗ "0" là sàn thấp nhất hợp lệ. Ở cửa sổ
 * hẹp (900px, mức tối thiểu ghi trong `docs/04-phase2-degraded-graph.md`) kèm
 * `maxLane` cao (nhiều nhánh sống đồng thời — có thật trên repo lớn), cột
 * gutter đồ thị (tới 288px ở `MAX_VISIBLE_LANES=20`) cộng hai cột ngày/mã
 * commit cố định ăn hết chỗ trống, và subject co xuống ĐÚNG 0px thật —
 * `getBoundingClientRect().width === 0` đo bằng Chromium thật (không phải suy
 * đoán). Dữ liệu DOM vẫn có `textContent` đúng, nhưng không một ký tự nào
 * hiển thị được — đây là lỗi checkpoint round 1 ("hầu hết dòng chỉ hiện dấu
 * '.'") mà 103 test cũ (chỉ render 1-3 hàng, subject ngắn, cửa sổ mặc định
 * trong test không mô phỏng bề rộng thật) không bắt được.
 *
 * happy-dom KHÔNG tính layout CSS thật (grid track sizing, `max-content`,
 * `minmax`) nên không viết được test DOM-level bắt đúng lỗi này — đã xác nhận
 * bằng cách viết test render 30 hàng trong `CommitList.test.tsx`: nó xanh cả
 * trước lẫn sau khi sửa vì happy-dom không mô phỏng việc co cột.
 *
 * Bằng chứng thật nằm ở `02-05-SUMMARY.md` mục "checkpoint round 1", đo bằng
 * Playwright + Chromium thật: trước sửa `subjectWidth: 0` tại 900px/maxLane
 * 19; sau sửa `subjectWidth: 120` (đúng sàn `minmax(120px, ...)` dưới đây).
 *
 * Test này là **lưới an toàn cấp hai**: đọc thẳng nguồn CSS và khẳng định
 * quy tắc `grid-template-columns` của `.commit-row` có một con số px cứng làm
 * sàn cho cột subject — không cho phép ai đổi ngược về `minmax(0, ...)` mà
 * không có ai để ý. Đây không thay thế được việc đo bằng trình duyệt thật khi
 * đổi bố cục — chỉ chặn đúng một kiểu hồi quy: xoá mất sàn px.
 */

/// <reference types="node" />
//
// Chỉ tệp test này cần kiểu Node (`readFileSync`) — dự án không thêm `"node"`
// vào mảng `types` của `tsconfig.json` vì đó là cấu hình toàn dự án và mã sản
// phẩm (browser-only, không có `tauri-plugin-fs`, xem CLAUDE.md) không được
// phép chạy Node API. `@types/node` đã có sẵn trong `node_modules` (phụ
// thuộc bắc cầu của các gói dev khác) nên reference cục bộ này biên dịch
// được mà không đổi phạm vi toàn cục.
//
// Không dùng hậu tố `?raw` của Vite: import đó trả CHUỖI RỖNG khi chạy qua
// vitest (môi trường Node của test runner không đi qua đường biến đổi
// asset-as-string của Vite dev/build pipeline) — đã xác nhận bằng debug in
// `css.length === 0`. `readFileSync` đọc trực tiếp từ đĩa, không phụ thuộc
// hành vi transform nào của bundler.
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

const cssPath = path.join(process.cwd(), 'src', 'styles', 'app.css')
const css = readFileSync(cssPath, 'utf8')

/** Trích nguyên văn khối `.commit-row { ... }` đầu tiên trong app.css. */
function extractCommitRowBlock(source: string): string {
  const start = source.indexOf('.commit-row {')
  if (start === -1) throw new Error('Không tìm thấy khối .commit-row trong app.css')
  const end = source.indexOf('}', start)
  return source.slice(start, end)
}

describe('.commit-row grid-template-columns', () => {
  it('cột subject (thứ hai) có sàn px cứng, không phải minmax(0, ...)', () => {
    const block = extractCommitRowBlock(css)
    const gridLine = block
      .split('\n')
      .find((line) => line.includes('grid-template-columns'))
    expect(gridLine, 'phải có dòng grid-template-columns trong .commit-row').toBeTruthy()

    // Khớp minmax(<số>px, 2fr) — cột thứ hai (subject) phải có sàn px > 0.
    const match = gridLine?.match(/minmax\((\d+)px,\s*2fr\)/)
    expect(
      match,
      `cột subject phải là minmax(<N>px, 2fr) với N > 0, dòng thật: ${gridLine}`,
    ).toBeTruthy()

    const floorPx = Number(match?.[1])
    expect(floorPx).toBeGreaterThan(0)
    // 80px là ngưỡng tối thiểu còn đọc được vài ký tự trước dấu "…" — thấp
    // hơn nữa thì coi như quay lại lỗi cũ.
    expect(floorPx).toBeGreaterThanOrEqual(80)
  })

  it('không còn minmax(0, 2fr) — chuỗi chính xác gây lỗi checkpoint round 1', () => {
    const block = extractCommitRowBlock(css)
    expect(block).not.toContain('minmax(0, 2fr)')
  })
})

/**
 * Test hồi quy checkpoint round 1 CỦA PLAN 02-06 — hàng có nhiều badge ref
 * (HIST-06) KHÔNG BAO GIỜ được cao hơn hàng không có badge.
 *
 * Bối cảnh: người dùng chạy app thật, chụp ảnh xác nhận hàng đầu tiên (có 4
 * badge: `master HEAD`, `origin/HEAD`, `origin/master`, `+1`) cao hơn rõ rệt
 * so với các hàng khác — kéo theo chấm đồ thị (canvas, vẽ theo `ROW_HEIGHT`
 * cố định từ `geometry.ts`) lệch khỏi tâm hàng đó, vì DOM row cao hơn nhưng
 * canvas vẫn vẽ ở toạ độ `rowY(index) = index * ROW_HEIGHT` cũ.
 *
 * Nguyên nhân gốc suy luận từ CSS: `.commit-row` là container CSS Grid, và
 * grid item mặc định có `min-height: auto` — không phải `0`. Điều đó nghĩa
 * là nội dung con (`.ref-badges` với nhiều badge, đặc biệt khi phông chữ hệ
 * thống thật — Segoe UI trên Windows — đo dòng cao hơn phông thay thế của
 * môi trường test headless) có thể ép TRACK grid cao lên vượt `height: 28px`
 * mà virtualizer đặt qua inline style, bất kể `.commit-subject` bên trong có
 * `overflow: hidden` hay không — `overflow: hidden` trên một phần tử con chỉ
 * cắt được NỘI DUNG của chính nó, không ngăn track cha của CSS Grid giãn nếu
 * track đó không tự giới hạn bằng `overflow: hidden` + `min-height: 0`.
 *
 * Không tái hiện được bằng số đo Playwright/Chromium headless trong phiên
 * điều tra này (phông thay thế cho Segoe UI có thể đo khác WebView2 thật) —
 * ghi rõ để trung thực, không giả vờ đã tái hiện được. Cách sửa (containment
 * cứng: `overflow: hidden` + `min-height: 0` trên `.commit-row`, giới hạn
 * `max-height` trên `.ref-badges`/`.ref-badge`) loại bỏ toàn bộ LỚP lỗi này
 * về mặt cấu trúc bất kể nguyên nhân đo chữ chính xác trên WebView2 là gì:
 * containment trên container Grid luôn buộc track tôn trọng `height` đã đặt.
 *
 * Test này (như `.commit-row grid-template-columns` ở trên) là lưới an toàn
 * cấp hai — đọc thẳng nguồn CSS, không đo layout đã tính ra. Không thay thế
 * việc đo bằng trình duyệt thật (lý tưởng nhất là WebView2 thật, không phải
 * Chromium độc lập) khi có nghi ngờ hồi quy tương tự.
 */
describe('.commit-row containment — badge không được phá chiều cao hàng cố định', () => {
  it('.commit-row có overflow: hidden VÀ min-height: 0 (chặn grid track tự giãn)', () => {
    const block = extractCommitRowBlock(css)
    expect(block, '.commit-row phải có overflow: hidden').toContain('overflow: hidden')
    expect(block, '.commit-row phải có min-height: 0').toContain('min-height: 0')
  })

  it('.ref-badges có max-height và overflow: hidden — không được giãn theo nội dung', () => {
    const start = css.indexOf('.ref-badges {')
    expect(start, 'phải tìm thấy khối .ref-badges trong app.css').toBeGreaterThan(-1)
    const end = css.indexOf('}', start)
    const block = css.slice(start, end)

    expect(block).toContain('max-height')
    expect(block).toContain('overflow: hidden')
    // flex-wrap: nowrap tường minh — không dựa vào giá trị mặc định của
    // trình duyệt, để không ai vô tình bật wrap khi thêm thuộc tính khác.
    expect(block).toContain('flex-wrap: nowrap')
  })

  it('.ref-badge có max-height khớp line-height — không cho một badge tự cao hơn các badge khác', () => {
    const start = css.indexOf('.ref-badge {')
    expect(start, 'phải tìm thấy khối .ref-badge trong app.css').toBeGreaterThan(-1)
    const end = css.indexOf('}', start)
    const block = css.slice(start, end)

    expect(block).toContain('max-height')
    expect(block).toContain('overflow: hidden')
  })
})
