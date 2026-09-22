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

import { MIN_SPLIT_WIDTH } from '@/components/diff/DiffViewer'
import { REF_COL_WIDTH } from '@/lib/graph-render/geometry'

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

/*
 * Nguyên nhân B của checkpoint round 1 (plan 02-06): `RefBadges` từng render
 * BÊN TRONG `.commit-subject`, nên badge và chữ message cạnh tranh CÙNG một
 * cột grid. Đo thật bằng Chromium nạp Segoe UI thật: nhóm 4 badge chiếm
 * 258px trong khi cột subject chỉ còn 332px (cửa sổ 1440px) rồi 144px (cửa
 * sổ 900px) — chữ message hiển thị 27% rồi **0%**. Khớp chính xác ảnh chụp
 * của người dùng: chỉ hàng nhiều badge mất chữ, hàng không badge vẫn bình
 * thường.
 *
 * Hai test dưới đây chặn hồi quy về CẤU TRÚC, thứ mà test đọc-chuỗi-CSS ở
 * trên không thể bắt: một là cột badge phải tồn tại trong grid, hai là
 * `RefBadges` không được nằm lại trong `.commit-subject` ở JSX.
 */
describe('nhãn ref phải có cột grid riêng, không dùng chung cột với chữ message', () => {
  it('.commit-row có ĐÚNG 6 cột — cột nhãn ref là cột riêng', () => {
    const block = extractCommitRowBlock(css)
    const match = block.match(/grid-template-columns:\s*([^;]+);/)
    const declaration = match?.[1]
    expect(declaration, 'phải tìm thấy grid-template-columns trong .commit-row').toBeDefined()

    // Đếm số track: tách theo khoảng trắng nhưng giữ nguyên các hàm
    // minmax(...)/max-content thành một track.
    const tracks = (declaration ?? '')
      .trim()
      .split(/\s+(?![^(]*\))/)
      .filter(Boolean)

    expect(
      tracks.length,
      `phải có 6 cột (graph, nhãn ref, message, tác giả, thời gian, sha) nhưng thấy ${tracks.length}: ${tracks.join(' | ')}`,
    ).toBe(6)
  })

  /*
   * Cột nhãn từng là `minmax(0, max-content)` — co theo nội dung. Giờ là bề
   * rộng CỐ ĐỊNH `var(--ref-col-width)`, vì canvas đồ thị phải biết dịch sang
   * phải bao nhiêu px để vẽ vào cột 2: cột co theo nội dung buộc canvas đo DOM
   * mỗi lần nhãn đổi, thành nguồn số liệu thứ hai có thể lệch — đúng lớp lỗi đã
   * gây hai vòng checkpoint thất bại. Nhãn dài giờ bị cắt ellipsis, như tham
   * chiếu, chứ không nới cột.
   */
  it('cột nhãn ref dùng bề rộng cố định --ref-col-width, không co theo nội dung', () => {
    const block = extractCommitRowBlock(css)
    const gridLine = block.split('\n').find((line) => line.includes('grid-template-columns'))

    expect(
      gridLine,
      'cột 1 phải là var(--ref-col-width) — canvas dịch theo đúng con số đó',
    ).toContain('var(--ref-col-width)')
  })

  it('.commit-ref-cell co được bên trong cột cố định để nhãn dài bị cắt, không tràn', () => {
    const start = css.indexOf('.commit-ref-cell {')
    expect(start, 'phải tìm thấy khối .commit-ref-cell — ô grid của cột nhãn').toBeGreaterThan(-1)
    const block = css.slice(start, css.indexOf('}', start))

    expect(block, '.commit-ref-cell phải có min-width: 0 để nhãn dài cắt được').toContain(
      'min-width: 0',
    )
    expect(block, '.commit-ref-cell phải có overflow: hidden').toContain('overflow: hidden')
  })
})

/*
 * Chặn ở tầng JSX, không chỉ tầng CSS: cột grid riêng trong `app.css` là vô
 * nghĩa nếu `CommitList.tsx` lại đặt `RefBadges` vào trong `.commit-subject`
 * như trước. Test này đọc thẳng nguồn JSX vì happy-dom không tính layout nên
 * không có cách nào đo được hệ quả bằng render.
 */
describe('CommitList.tsx — RefBadges không được nằm trong .commit-subject', () => {
  const tsxPath = path.join(process.cwd(), 'src', 'components', 'history', 'CommitList.tsx')
  const tsx = readFileSync(tsxPath, 'utf8')

  it('RefBadges nằm trong .commit-ref-cell, không nằm trong .commit-subject', () => {
    const refCellIdx = tsx.indexOf('className="commit-ref-cell"')
    expect(refCellIdx, 'CommitList phải render ô grid .commit-ref-cell').toBeGreaterThan(-1)

    const badgeIdx = tsx.indexOf('<RefBadges')
    expect(badgeIdx, 'CommitList phải render <RefBadges').toBeGreaterThan(-1)

    // RefBadges phải xuất hiện NGAY SAU .commit-ref-cell và TRƯỚC
    // .commit-subject — nếu nó nằm sau thẻ mở .commit-subject thì badge lại
    // dùng chung cột với chữ message, đúng lỗi của checkpoint round 1.
    const subjectIdx = tsx.indexOf('className="commit-subject"')
    expect(subjectIdx, 'CommitList phải render .commit-subject').toBeGreaterThan(-1)

    expect(
      badgeIdx > refCellIdx && badgeIdx < subjectIdx,
      `<RefBadges phải nằm giữa .commit-ref-cell (${refCellIdx}) và .commit-subject (${subjectIdx}) ` +
        `nhưng đang ở ${badgeIdx} — nếu nó nằm sau .commit-subject thì badge lại ăn vào cột chữ message`,
    ).toBe(true)
  })
})

/*
 * `REF_COL_WIDTH` tồn tại ở hai nơi — `geometry.ts` và biến `--ref-col-width`
 * trong `app.css` — và phải bằng nhau.
 *
 * Canvas đồ thị dịch sang phải đúng `--ref-col-width` để vẽ vào cột 2. Lệch hai
 * phía thì đồ thị vẽ đè lên cột nhãn hoặc bỏ trống một dải — lỗi im lặng, mỗi
 * phía tự nó vẫn nhất quán.
 */
describe('REF_COL_WIDTH phải khớp giữa geometry.ts và app.css', () => {
  it('--ref-col-width trong CSS bằng REF_COL_WIDTH trong geometry.ts', () => {
    const match = css.match(/--ref-col-width:\s*(\d+)px/)
    const cssValue = match?.[1]
    expect(cssValue, 'phải tìm thấy --ref-col-width trong app.css').toBeDefined()

    expect(
      Number(cssValue),
      `CSS có --ref-col-width: ${cssValue}px nhưng geometry.ts có REF_COL_WIDTH = ${REF_COL_WIDTH}. ` +
        `Canvas dịch theo con số CSS, còn hình học tính theo con số TS — lệch nhau thì ` +
        `đồ thị vẽ đè lên cột nhãn.`,
    ).toBe(REF_COL_WIDTH)
  })

  it('canvas dịch sang phải qua cột nhãn, không nằm sát lề', () => {
    const start = css.indexOf('.graph-canvas {')
    expect(start, 'phải tìm thấy khối .graph-canvas').toBeGreaterThan(-1)
    const block = css.slice(start, css.indexOf('}', start))

    expect(
      block,
      '.graph-canvas phải có margin-left tính theo --ref-col-width — thiếu nó thì ' +
        'đồ thị vẽ đè lên cột nhãn ở cột 1',
    ).toContain('--ref-col-width')
  })
})

/*
 * Canvas đồ thị nằm DƯỚI các hàng commit (`.graph-canvas` có `z-index: 0`, các
 * `.commit-row` đến sau trong DOM). Nên **mọi** nền hàng phải trong suốt một
 * phần: một nền đục xoá sạch đoạn đồ thị của đúng hàng đó.
 *
 * Lỗi đã xảy ra thật: `.commit-row:hover { background: var(--bg-inset) }` —
 * `--bg-inset` là màu đục, nên rê chuột qua một hàng làm cả đoạn lane/nút của
 * hàng đó biến mất thành một vệt đen ngang. `.commit-row.selected` dùng
 * `color-mix(... var(--bg))` cũng ra màu đục theo cách tương tự.
 *
 * Tham chiếu làm ngược lại: đo `docs/screenshots/main-4.png` cho hàng đang
 * chọn màu `#1b2b32` trên nền `#1c1e23` — một lớp phủ teal **sáng lên**, và
 * các lane vẫn hiện nguyên xuyên qua nó.
 */
describe('nền hàng commit phải trong suốt để không xoá đồ thị', () => {
  /** Trích thân của một quy tắc CSS theo selector chính xác. */
  function ruleBody(selector: string): string {
    const marker = `${selector} {`
    const start = css.indexOf(marker)
    expect(start, `phải tìm thấy quy tắc \`${selector}\` trong app.css`).toBeGreaterThan(-1)
    return css.slice(start + marker.length, css.indexOf('}', start))
  }

  for (const selector of ['.commit-row:hover', '.commit-row.selected']) {
    it(`${selector} dùng nền trong suốt một phần`, () => {
      const body = ruleBody(selector)
      const background = body
        .split('\n')
        .map((l) => l.trim())
        .find((l) => l.startsWith('background'))

      expect(background, `${selector} phải khai background`).toBeTruthy()

      // `transparent` là thành phần thứ hai của color-mix => kết quả có alpha.
      expect(
        background,
        `${selector} có nền \`${background}\` — nếu nó đục thì canvas đồ thị bên dưới ` +
          `bị xoá đúng trên hàng đó (vệt đen ngang khi hover). Phải trộn với ` +
          `\`transparent\` để giữ alpha.`,
      ).toContain('transparent')

      // Các biến nền đặc của bảng màu không được dùng trực tiếp ở đây.
      for (const opaque of ['var(--bg-inset)', 'var(--bg-raised)', 'var(--bg)']) {
        expect(
          background,
          `${selector} không được dùng ${opaque} làm nền đặc — đó chính là lỗi đã sửa`,
        ).not.toContain(`${opaque};`)
      }
    })
  }
})

/*
 * Canvas đồ thị **không được** định vị bằng `margin` phần trăm.
 *
 * Bản trước: `position: sticky; top: 0` + `margin-bottom: -100%`. Phần tử
 * sticky vẫn chiếm chỗ trong luồng, nên nó đẩy danh sách xuống đúng chiều cao
 * của nó và margin âm là để kéo ngược lên. Nhưng **margin phần trăm — kể cả
 * `margin-bottom` — quy chiếu theo BỀ RỘNG khối chứa, không phải chiều cao**
 * (CSS spec). Vùng cuộn rộng 740px, canvas cao 550px => `-100%` kéo lên 740px
 * thay vì 550px, lố 190px, mấy hàng đầu bị cắt cụt ("bị che phần đầu"). Sai số
 * đổi theo bề rộng pane nên lỗi lúc ẩn lúc hiện.
 */
describe('.graph-canvas không định vị bằng margin phần trăm', () => {
  const start = css.indexOf('.graph-canvas {')
  // Bỏ comment trước khi khẳng định: khối này CÓ nhắc `margin-bottom: -100%`
  // trong phần giải thích vì sao đã bỏ nó, và test đọc CSS thô sẽ khớp nhầm
  // chính đoạn văn đó thay vì khớp một khai báo thật.
  const block = css.slice(start, css.indexOf('}', start)).replace(/\/\*[\s\S]*?\*\//g, '')

  it('không dùng margin-bottom âm theo phần trăm', () => {
    expect(
      block,
      '`margin-bottom: -100%` quy chiếu theo BỀ RỘNG khối chứa, không phải chiều cao — ' +
        'canvas cao 550px trong vùng cuộn rộng 740px sẽ bị kéo lố 190px và cắt mất ' +
        'các hàng đầu. Đưa canvas ra khỏi luồng bằng `position: absolute` thay vì bù ' +
        'bằng margin âm.',
    ).not.toMatch(/margin-bottom:\s*-\d+%/)
  })

  it('ra khỏi luồng bằng position: absolute', () => {
    expect(
      block,
      'canvas phải `position: absolute` để không chiếm chỗ trong luồng — `sticky` chiếm ' +
        'chỗ nên lại cần margin âm bù lại, đúng cái bẫy vừa sửa',
    ).toMatch(/position:\s*absolute/)
  })
})

/*
 * ============================================================================
 * Bất biến bố cục của trình xem diff — plan 03-04.
 * ============================================================================
 *
 * Bốn test dưới đây là **lưới an toàn cấp hai**, cùng khuôn với các test ở trên:
 * đọc thẳng nguồn CSS, chặn đúng một kiểu hồi quy. Chúng **không** thay thế việc
 * đo bằng trình duyệt thật — happy-dom không tính layout CSS, và ba lỗi hiển thị
 * của Phase 2 qua hết 212 test tự động.
 *
 * Mỗi test đã được **kiểm là có thể đỏ**: đổi CSS cho vi phạm từng điều, xác
 * nhận đúng test đó đỏ, rồi hoàn nguyên. Số đo dán trong `03-04-SUMMARY.md`.
 */
describe('.diff-viewer containment cứng — grid/flex item không được tự giãn', () => {
  function ruleBody(selector: string): string {
    const marker = `${selector} {`
    const start = css.indexOf(marker)
    expect(start, `phải tìm thấy quy tắc \`${selector}\` trong app.css`).toBeGreaterThan(-1)
    return css.slice(start + marker.length, css.indexOf('}', start))
  }

  it('.diff-viewer có overflow: hidden VÀ min-height: 0', () => {
    // Bài học `.commit-row` sau checkpoint vòng 1: grid/flex item mặc định có
    // `min-height: auto`, KHÔNG phải `0` — nên nội dung con ép track cao lên
    // bất kể `overflow: hidden` ở phần tử con. `DiffViewer` chứa một
    // `EditorView` có thể cao hàng nghìn px; thiếu containment thì nó đẩy cả
    // panel giãn ra thay vì tự cuộn bên trong.
    const body = ruleBody('.diff-viewer')
    expect(body, '.diff-viewer phải có overflow: hidden').toContain('overflow: hidden')
    expect(body, '.diff-viewer phải có min-height: 0').toContain('min-height: 0')
  })

  it('.diff-pane cũng có min-height: 0 — containment phải đi hết chuỗi cha-con', () => {
    // `min-height: 0` trên đúng một cấp là vô dụng nếu cấp dưới lại `auto`:
    // chuỗi containment hỏng ở bất kỳ mắt nào là hỏng cả chuỗi.
    const body = ruleBody('.diff-pane')
    expect(body).toContain('min-height: 0')
    expect(body).toContain('overflow')
  })
})

describe('cột nội dung diff dùng sàn px cứng, KHÔNG minmax(0, ...)', () => {
  it('không có chuỗi `minmax(0,` trong bất kỳ quy tắc .diff-* nào', () => {
    /*
     * `minmax(0, 2fr)` là chuỗi CHÍNH XÁC đã gây lỗi checkpoint vòng 1 của
     * plan 02-05: `0` là sàn hợp lệ, và ở cửa sổ hẹp cột co về ĐÚNG 0px — chữ
     * hiển thị 0% trong khi DOM vẫn có `textContent` đúng
     * (`getBoundingClientRect().width === 0`, đo bằng Chromium thật).
     *
     * `DiffViewer` nằm trong một `Panel` KÉO ĐƯỢC nên bề rộng có thể nhỏ tuỳ ý
     * — đúng điều kiện đã kích hoạt lỗi đó.
     *
     * Lọc chú thích trước khi tìm: khối CSS của diff CÓ nhắc `minmax(0, ...)`
     * trong phần giải thích vì sao không dùng nó, và một test đọc CSS thô sẽ
     * khớp nhầm chính đoạn văn đó (đúng lỗi `.graph-canvas` ở trên).
     */
    const khongChuThich = css.replace(/\/\*[\s\S]*?\*\//g, '')

    // Cắt lấy mọi quy tắc có selector bắt đầu bằng `.diff-`.
    const khoiDiff: string[] = []
    const re = /(^|\n)\s*(\.diff-[^{]*)\{([^}]*)\}/g
    let m: RegExpExecArray | null
    while ((m = re.exec(khongChuThich)) !== null) {
      khoiDiff.push(`${m[2]}{${m[3]}}`)
    }

    expect(
      khoiDiff.length,
      'tiền đề: phải tìm thấy ít nhất một quy tắc .diff-* sau khi lọc chú thích — ' +
        'nếu 0 thì cổng này tự vô hiệu hoá',
    ).toBeGreaterThan(0)

    for (const khoi of khoiDiff) {
      expect(khoi, `quy tắc diff dùng minmax(0, ...) — sàn 0 làm cột co mất chữ:\n${khoi}`).not.toContain(
        'minmax(0,',
      )
    }
  })
})

describe('.diff-word-changed là LỚP PHỦ, không nền đục', () => {
  it('dùng nền có alpha, không dùng var(--bg*) làm nền đặc', () => {
    /*
     * `.diff-word-changed` nằm TRÊN nền dòng thêm/xoá. Một nền đục xoá mất tín
     * hiệu "dòng này đã thêm" — cùng lớp lỗi với `.commit-row:hover` dùng
     * `var(--bg-inset)` và xoá sạch đoạn đồ thị canvas của hàng đó (vệt đen
     * ngang khi hover, đã sửa ở Phase 2).
     */
    const start = css.indexOf('.diff-word-changed {')
    expect(start, 'phải tìm thấy .diff-word-changed trong app.css').toBeGreaterThan(-1)
    const body = css.slice(start + '.diff-word-changed {'.length, css.indexOf('}', start))

    const background = body
      .split('\n')
      .map((l) => l.trim())
      .find((l) => l.startsWith('background'))
    expect(background, '.diff-word-changed phải khai background').toBeTruthy()

    // `color-mix(... transparent)` hoặc `rgb(... / ...)` — cả hai cho alpha.
    const coAlpha =
      (background ?? '').includes('transparent') || /rgba?\([^)]*\//.test(background ?? '')
    expect(
      coAlpha,
      `.diff-word-changed có nền \`${background}\` — nền đục phủ mất nền dòng ` +
        `thêm/xoá bên dưới, xoá tín hiệu "dòng này đã thêm". Phải trộn với ` +
        `\`transparent\` hoặc dùng cú pháp alpha.`,
    ).toBe(true)

    for (const opaque of ['var(--bg-inset)', 'var(--bg-raised)', 'var(--bg)']) {
      expect(
        background,
        `.diff-word-changed không được dùng ${opaque} làm nền đặc`,
      ).not.toContain(`${opaque};`)
    }
  })
})

/*
 * `MIN_SPLIT_WIDTH` sống ở HAI nơi — `DiffViewer.tsx` (quyết định có tự chuyển
 * về hợp nhất) và biến `--min-split-width` trong `app.css` (sàn bề rộng của
 * vùng hai cột) — và phải bằng nhau.
 *
 * Lệch hai phía là **lỗi im lặng**: mỗi phía tự nó vẫn nhất quán, nhưng có một
 * DẢI bề rộng mà JS nói "hai cột" trong khi CSS đã co cột xuống dưới mức đọc
 * được (hoặc ngược lại, CSS giữ sàn trong khi JS đã chuyển về hợp nhất và sàn
 * thành một dải trống).
 *
 * Đúng tiền lệ `REF_COL_WIDTH` (geometry.ts + app.css) và `MAX_VISIBLE_LANES`
 * (Rust + TS) của Phase 2 — cả hai giờ đều có test đọc thẳng file kia. Đây là
 * cổng 12 của `<verification>`.
 */
describe('MIN_SPLIT_WIDTH phải khớp giữa DiffViewer.tsx và app.css', () => {
  it('--min-split-width trong CSS bằng MIN_SPLIT_WIDTH trong DiffViewer.tsx', () => {
    const match = css.match(/--min-split-width:\s*(\d+)px/)
    const cssValue = match?.[1]
    expect(cssValue, 'phải tìm thấy --min-split-width trong app.css').toBeDefined()

    expect(
      Number(cssValue),
      `CSS có --min-split-width: ${cssValue}px nhưng DiffViewer.tsx có ` +
        `MIN_SPLIT_WIDTH = ${MIN_SPLIT_WIDTH}. JS quyết định "hai cột hay hợp nhất" ` +
        `theo con số TS, còn CSS đặt sàn cột theo con số CSS — lệch nhau thì có một ` +
        `dải bề rộng mà JS hiện hai cột trong khi CSS đã co cột xuống dưới mức đọc được.`,
    ).toBe(MIN_SPLIT_WIDTH)
  })

  it('vùng hai cột thật sự DÙNG biến đó, không dán số cứng ở chỗ khác', () => {
    // Một biến khai mà không ai dùng làm test trên thành vô nghĩa — cùng bài
    // học `--row-height: 26px` từng tồn tại song song `ROW_HEIGHT = 28` và chỉ
    // vô hại vì không chỗ nào dùng tới nó.
    const start = css.indexOf('.diff-split {')
    expect(start, 'phải tìm thấy .diff-split — vùng hai cột').toBeGreaterThan(-1)
    const body = css.slice(start, css.indexOf('}', start))
    expect(body, '.diff-split phải dùng var(--min-split-width)').toContain(
      'var(--min-split-width)',
    )
  })
})

/*
 * 🔴 Lỗi người dùng báo ở checkpoint 11 bước của plan 03-04: bật chế độ **hai
 * cột** thì nội dung bị cắt, không cuộn được.
 *
 * Nguyên nhân: `@codemirror/merge` ghi đè theme của dự án bằng `!important`
 * (`node_modules/@codemirror/merge/dist/index.js`):
 *
 *   ".cm-mergeView & .cm-scroller, .cm-mergeView &": {
 *       height: "auto !important", overflowY: "visible !important" }
 *
 * nên `height: 100%` và `.cm-scroller { overflow: auto }` mà
 * `diff-render/theme.ts` đặt **bị vô hiệu** trong chế độ hai cột. Thư viện dồn
 * việc cuộn về `.cm-mergeView` (`overflowY: auto`) để hai phía cuộn cùng nhau —
 * nhưng phần tử đó nằm trong `.diff-host { overflow: hidden }` và **không nhận
 * chiều cao từ đâu**, nên nó cao theo nội dung rồi bị cha cắt cứng.
 *
 * Vì sao 365 test tự động không bắt được: `MergeView` **chưa bao giờ được render
 * trong một test nào** — happy-dom không có `ResizeObserver`, `paneWidth = 0`,
 * mọi test chạy nhánh hợp nhất. Cùng lớp lỗi với ba lỗi bố cục của Phase 2, và
 * cùng cách phát hiện: người dùng mở app thật.
 *
 * Hai test dưới đây là lưới an toàn cấp hai (đọc nguồn CSS, không đo layout).
 * Chúng **không** thay thế việc kiểm bằng mắt trên WebView2 — xem doc comment
 * đầu tệp này.
 */
describe('chế độ hai cột — .cm-mergeView phải là vùng cuộn thật', () => {
  it('.cm-mergeView có chiều cao xác định, nếu không overflowY:auto của thư viện không kích hoạt', () => {
    const start = css.indexOf('.cm-mergeView {')
    expect(
      start,
      'phải có quy tắc .cm-mergeView — thiếu nó thì hai cột bị cắt, không cuộn được',
    ).toBeGreaterThan(-1)
    const block = css.slice(start, css.indexOf('}', start))

    expect(
      block,
      '.cm-mergeView phải có chiều cao xác định. Nó nằm trong .diff-host{overflow:hidden} ' +
        'nên không có chiều cao thì nó cao theo nội dung rồi bị cắt, và overflowY:auto mà ' +
        '@codemirror/merge đặt không bao giờ kích hoạt.',
    ).toMatch(/height:\s*100%/)
  })

  it('.cm-mergeViewEditor có min-width: 0 để dòng dài không đẩy cột kia hẹp lại', () => {
    const start = css.indexOf('.cm-mergeViewEditor {')
    expect(start, 'phải có quy tắc .cm-mergeViewEditor').toBeGreaterThan(-1)
    const block = css.slice(start, css.indexOf('}', start))

    // Flex item mặc định `min-width: auto` → không co dưới bề rộng nội dung.
    expect(
      block,
      '.cm-mergeViewEditor là flex item (thư viện đặt flexGrow/flexBasis); thiếu ' +
        'min-width: 0 thì một dòng dài đẩy cột kia hẹp lại thay vì cuộn ngang trong cột của nó',
    ).toContain('min-width: 0')
  })
})

/*
 * 🔴 Ba lỗi nữa người dùng báo ở checkpoint **vòng 2** của plan 03-04, cả ba chỉ
 * thấy được trong chế độ hai cột trên bản release thật.
 *
 * Chúng đến cùng một nguồn: `@codemirror/merge` mang **theme riêng** và theme đó
 * thắng cascade của dự án trong chế độ hai cột. Đo bằng Chromium thật
 * (Playwright, quy trình của 02-05) trên một hunk có 2 dòng thêm + 1 dòng xoá:
 *
 * | phần tử | class thật đo được | backgroundColor đo được | đáng ra |
 * |---|---|---|---|
 * | dòng xoá phía A | `cm-line diff-line-removed cm-changedLine` | `rgba(160,128,100,.08)` | màu `--danger` của dự án |
 * | dòng thêm phía B | `cm-line diff-line-added cm-changedLine` | `rgba(100,160,128,.08)` | màu `--success` của dự án |
 * | `.cm-mergeSpacer` | `cm-mergeSpacer` | `rgba(0,0,0,0)` + `background-image: none` | nền sọc chéo |
 *
 * Class của dự án **có** được gắn — nên `decorations.ts` không sai. Điều sai là
 * **độ cụ thể**: selector của thư viện là `&.cm-merge-a .cm-changedLine`, tức
 * hai class cộng một quan hệ hậu duệ, thắng `.diff-line-removed` một class.
 * Trong chế độ hợp nhất không có `.cm-changedLine` nào nên màu của dự án đúng —
 * đó là lý do lỗi này sống sót qua cả checkpoint vòng 1.
 */
describe('chế độ hai cột — màu của DỰ ÁN phải thắng theme của @codemirror/merge', () => {
  /*
   * Vì sao kiểm bằng "có `.cm-changedLine` trong selector" chứ không kiểm
   * `!important`: `!important` là một cách thắng, nhưng nâng độ cụ thể là cách
   * đúng hơn và không chặn theme sáng ghi đè sau. Cổng này đòi **có một selector
   * đủ cụ thể**, không đòi một kỹ thuật cụ thể — miễn nó nhắc `.cm-changedLine`
   * thì nó đã ở cùng hoặc trên mức của thư viện.
   *
   * 🔴 Phải tìm trên nguồn đã **bỏ chú thích**. Doc comment của chính mục này
   * dẫn nguyên văn `cm-line diff-line-added cm-changedLine` trong bảng đo, nên
   * tìm thô trúng chú thích và cổng thành **tự vô hiệu hoá** — đã chứng minh
   * bằng đột biến: xoá hẳn quy tắc ghi đè mà cả ba cổng vẫn xanh.
   */
  const cssKhongChuThich = css.replace(/\/\*[\s\S]*?\*\//g, '')

  it('có quy tắc ghi đè nền dòng THÊM trong ngữ cảnh .cm-changedLine', () => {
    expect(
      cssKhongChuThich,
      'thiếu quy tắc này thì dòng thêm ở chế độ hai cột hiện màu lục-nâu ' +
        'rgba(100,160,128,.08) của @codemirror/merge, không phải var(--success) của dự án',
    ).toMatch(/\.diff-line-added\.cm-changedLine|\.cm-changedLine\.diff-line-added/)
  })

  it('có quy tắc ghi đè nền dòng XOÁ trong ngữ cảnh .cm-changedLine', () => {
    expect(
      cssKhongChuThich,
      'thiếu quy tắc này thì dòng xoá ở chế độ hai cột hiện màu nâu ' +
        'rgba(160,128,100,.08) của @codemirror/merge, không phải var(--danger) của dự án',
    ).toMatch(/\.diff-line-removed\.cm-changedLine|\.cm-changedLine\.diff-line-removed/)
  })

  it('nền word-level cũng phải thắng .cm-changedText của thư viện', () => {
    /*
     * Thư viện tô `.cm-changedText` bằng một `linear-gradient` gạch chân
     * (`bottom/100% 2px no-repeat`). Dự án dùng `spans` từ
     * `git diff --word-diff-regex` cho **cả hai** chế độ (quyết định của
     * `codemirrorRenderer.ts`), nên hai nguồn word-level không được trộn: người
     * dùng sẽ thấy hai kết quả khác nhau cho cùng một dòng.
     */
    expect(
      cssKhongChuThich,
      'phải tắt .cm-changedText của @codemirror/merge trong chế độ hai cột — ' +
        'dự án dùng spans của git cho word-level ở cả hai chế độ',
    ).toMatch(/\.cm-changedText/)
  })
})

/*
 * 🔴 Vùng căn hàng phải có **nền sọc chéo**, không để trống trơn.
 *
 * Người dùng gửi hai ảnh tham chiếu: ở chỗ một phía thiếu dòng, phía kia hiện
 * một khối gạch chéo cao đúng số dòng thiếu. Nó nói "bên này không có gì ở đây"
 * thay vì để người đọc tự đoán khoảng trống nghĩa là gì.
 *
 * Phần khó **thư viện đã làm**: đo bằng Chromium thật thấy `@codemirror/merge`
 * tự chèn widget `.cm-mergeSpacer` với chiều cao đúng bằng số dòng thiếu
 * (36px cho hunk 2 dòng, 72px cho hunk 4 dòng — hai lần `line-height` 18px).
 * Nên đây chỉ là việc tô nền, KHÔNG phải dựng widget.
 */
describe('chế độ hai cột — vùng căn hàng có nền sọc chéo', () => {
  it('.cm-mergeSpacer có nền sọc chéo, không trống trơn', () => {
    const start = css.indexOf('.cm-mergeSpacer {')
    expect(
      start,
      'phải có quy tắc .cm-mergeSpacer. Đo được: thư viện chèn widget này để căn ' +
        'hàng nhưng để nó TRONG SUỐT (background-image: none), nên người đọc không ' +
        'biết khoảng trống đó nghĩa là "phía này không có dòng".',
    ).toBeGreaterThan(-1)
    const block = css.slice(start, css.indexOf('}', start))

    expect(
      block,
      '.cm-mergeSpacer phải dùng repeating-linear-gradient để vẽ sọc chéo',
    ).toContain('repeating-linear-gradient')
  })
})


/*
 * 🔴 Lỗi full-height, VÒNG 2 lần thứ hai: thanh cuộn ngang nằm GIỮA màn hình.
 *
 * Người dùng khoanh đỏ trong ảnh: thanh cuộn ngang của hai cột nằm ngay dưới
 * dòng nội dung cuối (~2/3 chiều cao cửa sổ), rồi một dải trống lớn xuống đáy.
 *
 * # Vì sao vòng sửa trước không bắt được
 *
 * Phép đo trước dùng một tệp **DÀI** (60 dòng đệm). Nội dung dài tự lấp hết
 * khung nên mọi mắt tình cờ cao bằng cha, và lỗi vô hình. Lỗi chỉ lộ với tệp
 * **NGẮN**, nơi nội dung KHÔNG lấp hết khung. Harness đo có cờ `?short=1` đúng
 * vì lý do này.
 *
 * # Đo bằng Chromium thật, tệp NGẮN, cửa sổ 1600×900 — TRƯỚC khi sửa
 *
 * | mắt | chiều cao | ghi chú |
 * |---|---|---|
 * | `.diff-pane` | 794.5 (bottom 875) | đúng |
 * | `.diff-host` | 794.5 | đúng |
 * | `.cm-mergeView` | 794.5 | đúng — `height: 100%` của vòng 1 CÓ hiệu lực |
 * | `.cm-mergeViewEditors` | **314** | 🔴 ĐỨT — co theo nội dung |
 * | `.cm-editor` | 314 | theo cha |
 * | `.cm-scroller` | 314, **bottom 394** | thanh cuộn ngang ở y=394 |
 *
 * `.diff-pane` bottom 875 − `.cm-scroller` bottom 394 = **khoảng hở đáy 481px**,
 * đúng dải trống người dùng khoanh.
 *
 * # Nguyên nhân gốc
 *
 * Thư viện khai `.cm-mergeViewEditors { display: flex; align-items: stretch }`
 * nhưng **không** đặt `height`, và đặt `height: auto !important` cho
 * `.cm-mergeView &` (tức `.cm-editor`) cùng `.cm-scroller`. `align-items:
 * stretch` căng các **con** theo trục ngang của một flex-row — nó không cho bản
 * thân phần tử chiều cao. Nên cả chuỗi dưới `.cm-mergeView` rơi về `auto` = co
 * theo nội dung.
 *
 * # 🔴 `min-height`, KHÔNG phải `height` — bẫy của bản sửa này
 *
 * Bản sửa đầu dùng `height: 100%` và nó **làm hỏng việc cuộn dọc**. Đo được:
 *
 * | tệp | `height: 100%` | `min-height: 100%` |
 * |---|---|---|
 * | NGẮN | hở đáy 0px ✅ | hở đáy 0px ✅ |
 * | DÀI | scrollH 795 = clientH 795 → **KHÔNG cuộn**, 1394px nội dung bị cắt 🔴 | scrollH 1394 > clientH 795 → cuộn ĐÚNG ✅ |
 *
 * `.cm-mergeView` là vùng cuộn; nó chỉ biết phải cuộn khi **con** cao hơn nó.
 * Kẹp con về đúng 100% thì không bao giờ có overflow — thanh cuộn dọc không bao
 * giờ hiện và nội dung dư bị cắt im lặng, tệ hơn lỗi ban đầu.
 *
 * Nên cổng dưới đây đòi **`min-height`** và đòi **không** có `height: 100%`.
 * Một cổng chỉ kiểm "có chiều cao" sẽ xanh với chính bản sửa gây hồi quy.
 */
describe('chế độ hai cột — chuỗi chiều cao phải đi hết xuống .cm-scroller', () => {
  /*
   * 🔴 Phải bỏ chú thích TRƯỚC khi tìm.
   *
   * Doc comment của chính các quy tắc này dẫn nguyên văn khai báo của thư viện
   * (`.cm-mergeViewEditors { display: flex; align-items: stretch }`) và cả
   * chuỗi `height: 100%` trong bảng so sánh ở trên — nên tìm thô sẽ trúng **chú
   * thích** trước khi trúng quy tắc thật. Cổng tự vô hiệu hoá, đã gặp thật ở lần
   * chạy đầu. Cùng bài học `interface-boundary.test.ts` đã ghi cho phía TS.
   */
  const cssKhongChuThich = css.replace(/\/\*[\s\S]*?\*\//g, '')

  /**
   * Thân quy tắc có selector **đúng bằng** `selector`, trên nguồn đã bỏ chú thích.
   *
   * Cắt và tìm `}` **trên cùng một chuỗi**: trộn offset của chuỗi đã bỏ chú
   * thích với chuỗi gốc cho ra một đoạn nằm giữa hai quy tắc khác nhau — đã gặp
   * thật, và nó làm cổng báo đỏ vì lý do sai.
   *
   * `\n` trước selector để `.cm-mergeViewEditor` không trúng
   * `.cm-mergeViewEditors`.
   */
  function than(selector: string): string | null {
    const i = cssKhongChuThich.indexOf(`\n${selector} {`)
    if (i === -1) return null
    const mo = cssKhongChuThich.indexOf('{', i)
    return cssKhongChuThich.slice(mo + 1, cssKhongChuThich.indexOf('}', mo))
  }

  it('.cm-mergeViewEditors dùng min-height: 100% (căng khi ngắn, giãn khi dài)', () => {
    const block = than('.cm-mergeViewEditors')
    expect(
      block,
      'phải có quy tắc .cm-mergeViewEditors. Đo được bằng Chromium trên tệp NGẮN: ' +
        'thư viện đặt display:flex + align-items:stretch nhưng KHÔNG đặt height, nên ' +
        'nó cao 314px trong khi .cm-mergeView cao 794.5px — thanh cuộn ngang nằm giữa ' +
        'màn hình và còn 481px trống bên dưới.',
    ).not.toBeNull()

    expect(block, '.cm-mergeViewEditors phải có min-height: 100%').toMatch(
      /min-height:\s*100%/,
    )
  })

  it('🔴 .cm-mergeViewEditors KHÔNG được dùng height: 100% — nó chặn cuộn dọc', () => {
    /*
     * Đây là cổng chống đúng bản sửa sai mà tôi đã viết ra một lần: `height:
     * 100%` kẹp con về bằng vùng cuộn, nên `.cm-mergeView` không bao giờ thấy
     * overflow (đo được: scrollHeight 795 = clientHeight 795 trên tệp có 1394px
     * nội dung) và phần dư bị cắt im lặng.
     */
    const block = than('.cm-mergeViewEditors')
    expect(block).not.toBeNull()
    expect(
      block,
      'height: 100% ở đây làm .cm-mergeView không còn overflow để cuộn — tệp dài ' +
        'bị cắt mất phần dưới. Dùng min-height: 100%.',
    ).not.toMatch(/(^|[^-])height:\s*100%/)
  })

  it('.cm-editor trong .cm-mergeView có min-height: 100%', () => {
    /*
     * Thư viện đặt `height: auto !important` cho `.cm-mergeView &`. `min-height`
     * thắng được nó vì đây là **hai thuộc tính khác nhau** — `!important` của
     * `height` không nói gì về `min-height`, và `min-height` luôn thắng `height`
     * trong thuật toán tính kích thước. Đó là điều làm bản vá không cần
     * `!important` cho chính nó.
     */
    const block = than('.cm-mergeView .cm-editor')
    expect(
      block,
      'phải có .cm-mergeView .cm-editor { min-height: 100% } — thiếu nó thì editor ' +
        'co theo nội dung dù .cm-mergeViewEditors đã cao đủ khung',
    ).not.toBeNull()
    expect(block).toMatch(/min-height:\s*100%/)
  })

  it('.cm-scroller trong .cm-mergeView có flex-grow: 1', () => {
    /*
     * Đo được: `.cm-editor` là `display: flex; flex-direction: column` và
     * `.cm-scroller` là con duy nhất với `flex: 0 1 auto`. Thiếu `flex-grow`
     * thì editor cao đủ khung mà scroller vẫn co theo nội dung — thanh cuộn
     * ngang lại nằm giữa khung, đúng chỗ người dùng khoanh đỏ.
     */
    const block = than('.cm-mergeView .cm-scroller')
    expect(
      block,
      'phải có .cm-mergeView .cm-scroller { flex-grow: 1 } — nếu không thanh cuộn ' +
        'ngang vẫn nằm ở đáy NỘI DUNG chứ không ở đáy KHUNG',
    ).not.toBeNull()
    expect(block).toMatch(/flex-grow:\s*1/)
  })
})
