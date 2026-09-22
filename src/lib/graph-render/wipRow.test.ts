/**
 * Test cho `wipRow.ts` — nguồn duy nhất của phép ánh xạ chỉ số hàng WIP (WORK-11).
 *
 * Ba nhóm, ba lớp lỗi khác nhau:
 *
 * 1. **Chỉ số** — lớp lỗi "cột đồ thị lệch cột văn bản đúng một hàng", đã xảy ra
 *    **hai lần** ở Phase 2.
 * 2. **Cổng "đúng một chỗ"** — đọc nguồn của `graph-render/` và `components/history/`
 *    và khẳng định không tệp nào ngoài `wipRow.ts` làm số học với `WIP_ROW_HEIGHT`.
 * 3. **Cạnh WIP nối xuống HEAD** — lỗi clamp đã được **tái hiện có số** 2026-09-22.
 *
 * 🔴 **Mọi ngưỡng lane viết theo `MAX_VISIBLE_LANES`, không viết số.** Cap đã đổi
 * 13 → 20 trong hai ngày; lần sau nó lại đổi. Xem `CAP` / `LANE_TRAN` / `LANE_VUOT`.
 */

/// <reference types="node" />
//
// Cùng lý do với `src/styles/app.css.test.ts`: chỉ tệp test này cần kiểu Node
// (`readFileSync`/`readdirSync`) cho cổng đọc nguồn. Không thêm `"node"` vào
// `types` của `tsconfig.json` vì mã sản phẩm không được phép chạy Node API.
import { readFileSync, readdirSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

import { laneX, MAX_VISIBLE_LANES, ROW_HEIGHT } from '@/lib/graph-render/geometry'
import {
  commitIndexFor,
  commitRowY,
  contentOffset,
  hasWipRow,
  wipEdge,
  WIP_ROW_COUNT,
  WIP_ROW_HEIGHT,
  virtualizerCount,
} from '@/lib/graph-render/wipRow'

/**
 * Ba đại lượng của bài kiểm clamp, **suy từ cap**, không viết số.
 *
 * `LANE_TRAN` là cột mà `laneX` gập **mọi** lane cao về — đó là chỗ một cạnh WIP
 * sai sẽ hiện ra. `LANE_VUOT` là một lane vượt cap rõ ràng (không phải đúng biên),
 * để phân biệt với ca `LANE_TRAN` ngay bên dưới.
 */
const CAP = MAX_VISIBLE_LANES
const LANE_TRAN = CAP - 1
const LANE_VUOT = CAP + 6

describe('hằng số hàng WIP', () => {
  it('WIP_ROW_COUNT là 1 — hàng WIP là MỘT hàng, không phải một vùng', () => {
    expect(WIP_ROW_COUNT).toBe(1)
  })

  /**
   * M5: `WIP_ROW_HEIGHT` đặt cứng `30` thay vì `= ROW_HEIGHT`.
   *
   * Một hằng số chiều cao thứ hai là đúng lớp lỗi mà Phase 2 đã phải thêm test ghim
   * hai phía cho `MAX_VISIBLE_LANES` và `REF_COL_WIDTH`.
   */
  it('WIP_ROW_HEIGHT === ROW_HEIGHT — hàng WIP cao ĐÚNG bằng hàng commit', () => {
    expect(WIP_ROW_HEIGHT).toBe(ROW_HEIGHT)
  })
})

describe('contentOffset — phép cộng DUY NHẤT của cả tính năng', () => {
  it('hasWip = true → ROW_HEIGHT', () => {
    expect(contentOffset(true)).toBe(ROW_HEIGHT)
  })

  /** M10: `contentOffset` trả `ROW_HEIGHT` cả khi `hasWip = false`. */
  it('hasWip = false → 0 (test HỒI QUY: bật tính năng không đổi hành vi cũ)', () => {
    expect(contentOffset(false)).toBe(0)
  })
})

describe('commitRowY — MỘT hàm cho CẢ cột đồ thị lẫn cột văn bản', () => {
  /**
   * R1 đòi ≥ 3 vị trí cuộn. Hai lời gọi cùng tham số phải cho **cùng** số: đó là
   * toàn bộ nội dung của bất biến "một nguồn toạ độ Y".
   */
  it.each([0, 137, 4021, 99_988])(
    'cùng đầu vào → cùng số, ở scrollTop = %i (đồ thị và văn bản không thể lệch)',
    (scrollTop) => {
      for (const start of [0, ROW_HEIGHT, ROW_HEIGHT * 37]) {
        for (const hasWip of [true, false]) {
          const cotDoThi = commitRowY(start, scrollTop, hasWip)
          const cotVanBan = commitRowY(start, scrollTop, hasWip)
          expect(cotDoThi).toBe(cotVanBan)
        }
      }
    },
  )

  /**
   * Test **hồi quy**: không có thay đổi chưa commit thì mọi con số phải y hệt giá
   * trị hiện tại của `CommitList.tsx` (`v.start - scrollTop`).
   */
  it.each([
    [0, 0],
    [ROW_HEIGHT, 0],
    [ROW_HEIGHT * 50, 137],
    [ROW_HEIGHT * 3571, 99_988],
  ])('hasWip = false → start %i - scrollTop %i, y HỆT hành vi hiện tại', (start, scrollTop) => {
    expect(commitRowY(start, scrollTop, false)).toBe(start - scrollTop)
  })

  it('hasWip = true → đúng bằng hành vi cũ CỘNG contentOffset, không hơn', () => {
    for (const [start, scrollTop] of [
      [0, 0],
      [ROW_HEIGHT, 0],
      [ROW_HEIGHT * 50, 137],
      [ROW_HEIGHT * 3571, 99_988],
    ] as const) {
      expect(commitRowY(start, scrollTop, true)).toBe(start - scrollTop + contentOffset(true))
    }
  })

  it('chênh lệch giữa hai ca đúng bằng MỘT chiều cao hàng, không hai', () => {
    const start = ROW_HEIGHT * 12
    const scrollTop = 40
    expect(commitRowY(start, scrollTop, true) - commitRowY(start, scrollTop, false)).toBe(
      WIP_ROW_HEIGHT,
    )
  })
})

describe('commitIndexFor — ĐỒNG NHẤT có chủ ý', () => {
  /**
   * M8: `commitIndexFor` trả `i - 1`.
   *
   * Hàm này tồn tại để **chứng minh** không có phép ±1 trên chỉ số, và để một cổng
   * đọc được nó. Nó là mã chết theo nghĩa hẹp và **không phải** mã chết theo nghĩa
   * rộng — xem doc comment của `wipRow.ts`.
   */
  it.each([0, 1, 999])('commitIndexFor(%i) === chính nó', (i) => {
    expect(commitIndexFor(i)).toBe(i)
  })
})

describe('virtualizerCount — hàng WIP KHÔNG vào total', () => {
  /** M9: `virtualizerCount` trả `total + 1`. */
  it.each([
    [0, true],
    [0, false],
    [1, true],
    [1, false],
    [100_007, true],
    [100_007, false],
  ])('total %i, hasWip %s → total không đổi', (total, hasWip) => {
    expect(virtualizerCount(total, hasWip)).toBe(total)
  })
})

describe('hasWipRow — tiêu chí 7: commit xong thì hàng biến mất', () => {
  /** M16: hàng WIP hiện khi `wipCounts = {0,0}`. */
  it('{ modified: 0, added: 0 } → KHÔNG có hàng WIP', () => {
    expect(hasWipRow({ modified: 0, added: 0 })).toBe(false)
  })

  it.each([
    [{ modified: 1, added: 0 }],
    [{ modified: 0, added: 1 }],
    [{ modified: 3, added: 1 }],
  ])('%o → CÓ hàng WIP', (counts) => {
    expect(hasWipRow(counts)).toBe(true)
  })

  it('undefined (chưa nạp status lần nào) → KHÔNG có hàng WIP', () => {
    expect(hasWipRow(undefined)).toBe(false)
  })
})

describe('wipEdge — cạnh nối xuống LANE CỦA HEAD, không xuống hàng 0', () => {
  it('headLane = 0 → cạnh thẳng ở laneX(0). Ca cơ sở.', () => {
    const e = wipEdge({ headLane: 0 })
    expect(e.kind).toBe('normal')
    if (e.kind !== 'normal') throw new Error('tiền đề')
    expect(e.x).toBe(laneX(0))
    expect(e.lane).toBe(0)
  })

  it('headLane = 5 → cạnh thẳng ở laneX(5); hàng WIP dùng CÙNG lane với HEAD', () => {
    const e = wipEdge({ headLane: 5 })
    expect(e.kind).toBe('normal')
    if (e.kind !== 'normal') throw new Error('tiền đề')
    expect(e.x).toBe(laneX(5))
    expect(e.lane).toBe(5)
  })

  /**
   * 🔴 **Ca chịu lực.** M11: `wipEdge` gọi `laneX(headLane)` rồi vẽ, bỏ phép kiểm cap.
   *
   * `laneX(LANE_VUOT) === laneX(LANE_TRAN)` do clamp, nên vẽ ở đó là nối hàng WIP
   * vào **một commit khác** — đồ thị vẽ sai một cách tự tin, không phải suy giảm
   * nhìn-là-thấy.
   */
  it(`headLane = ${LANE_VUOT} (>= cap ${CAP}) → cạnh KHÔNG vẽ ở laneX(${LANE_TRAN})`, () => {
    const e = wipEdge({ headLane: LANE_VUOT })
    expect(
      e.kind,
      `lane ${LANE_VUOT} vượt cap ${CAP}; vẽ cạnh ở cột ${LANE_TRAN} là nối vào commit khác`,
    ).not.toBe('normal')
    if (e.kind === 'normal') throw new Error('tiền đề')
    // Không một biến thể nào được mang toạ độ x của cột gập.
    expect(
      (e as { x?: number }).x,
      `không được trả toạ độ vẽ ${laneX(LANE_TRAN)} cho một lane bị clamp`,
    ).toBeUndefined()
  })

  /**
   * 🔴 M12: `wipEdge` bỏ cạnh khi `headLane >= CAP - 1`.
   *
   * **Phép phân biệt bắt buộc.** Một cài đặt "hễ >= LANE_TRAN thì bỏ cạnh" cũng thoả
   * ca trên nhưng **sai** ở đây. Không có test này thì test trên không chứng minh gì.
   */
  it(`headLane = ${LANE_TRAN} (ĐÚNG biên, không vượt cap) → cạnh CÓ vẽ ở laneX(${LANE_TRAN})`, () => {
    const e = wipEdge({ headLane: LANE_TRAN })
    expect(e.kind, `lane ${LANE_TRAN} < cap ${CAP} nên là ca BÌNH THƯỜNG`).toBe('normal')
    if (e.kind !== 'normal') throw new Error('tiền đề')
    expect(e.x).toBe(laneX(LANE_TRAN))
    expect(e.lane).toBe(LANE_TRAN)
  })

  it(`headLane = ${CAP} (vượt cap ĐÚNG một đơn vị) → đường suy giảm, kiểm biên dưới`, () => {
    const e = wipEdge({ headLane: CAP })
    expect(e.kind).toBe('clamped')
  })

  /**
   * M15: `wipEdge` trả `null` khi vượt cap (im lặng).
   *
   * Một hàng WIP không cạnh mà không nói gì là "một chấm trôi không có cạnh" mà
   * ROADMAP cấm. Biến thể phải mang lane **thật** để giao diện hiện được chỉ báo.
   */
  it('vượt cap → biến thể `clamped` mang realLane THẬT, không phải null im lặng', () => {
    const e = wipEdge({ headLane: LANE_VUOT })
    expect(e).not.toBeNull()
    expect(e.kind).toBe('clamped')
    if (e.kind !== 'clamped') throw new Error('tiền đề')
    expect(e.realLane, 'phải mang lane thật để giao diện hiện chỉ báo').toBe(LANE_VUOT)
  })

  /**
   * 🔴 M13: `wipEdge` lấy lane từ `graphRows[0].lane`.
   *
   * **Gốc của cả lỗi này.** `git log --all --topo-order` xếp commit mới nhất theo
   * topo của MỌI ref ở hàng 0; HEAD ở một nhánh cũ nằm **giữa** danh sách. Hàng 0 và
   * HEAD là hai thứ khác nhau.
   */
  it('hàng 0 có lane 0 nhưng headLane cao → kết quả theo HEAD, KHÔNG theo hàng 0', () => {
    const e = wipEdge({ headLane: LANE_VUOT, rowZeroLane: 0 })
    expect(
      e.kind,
      'lấy lane từ hàng 0 (= 0) sẽ cho ca normal; phải theo headLane',
    ).toBe('clamped')
    if (e.kind !== 'clamped') throw new Error('tiền đề')
    expect(e.realLane).toBe(LANE_VUOT)
  })

  it('hàng 0 lane cao nhưng headLane = 0 → kết quả theo HEAD (chiều ngược lại)', () => {
    const e = wipEdge({ headLane: 0, rowZeroLane: LANE_VUOT })
    expect(e.kind).toBe('normal')
    if (e.kind !== 'normal') throw new Error('tiền đề')
    expect(e.x).toBe(laneX(0))
  })

  /**
   * M14: `headLane === undefined` → vẽ vào lane 0.
   *
   * Vẽ vào lane 0 là **đoán**, và đoán sai một cách tự tin. Ca thật: HEAD nằm ngoài
   * phần lịch sử đã nạp (nhánh cũ hơn số hàng đã nạp).
   */
  it('headLane = undefined → biến thể riêng, KHÔNG panic, KHÔNG vẽ vào lane 0', () => {
    const e = wipEdge({ headLane: undefined })
    expect(e.kind).toBe('unknownHead')
    expect((e as { x?: number }).x, 'không được đoán một toạ độ vẽ').toBeUndefined()
    expect((e as { lane?: number }).lane, 'không được đoán lane 0').toBeUndefined()
  })

  it('headLane = null (trường IPC vắng) → cùng biến thể unknownHead', () => {
    const e = wipEdge({ headLane: null })
    expect(e.kind).toBe('unknownHead')
  })

  it('headLane âm (dữ liệu hỏng) → unknownHead, không vẽ vào lane 0', () => {
    const e = wipEdge({ headLane: -1 })
    expect(e.kind).toBe('unknownHead')
  })
})

// --- Cổng "đúng một chỗ" ---------------------------------------------------
//
// 🔴 Bốn ràng buộc bắt buộc, mỗi cái chống một lỗi cổng đã xảy ra (CONTEXT.md 3.1):
//
// 1. **Bỏ chú thích và chuỗi trước khi tìm** (lỗi #5 và #1).
// 2. **Khẳng định tiền đề** — không đọc được tệp, hoặc `wipRow.ts` không chứa hằng
//    đó → cổng ĐỎ (lỗi #3, #8).
// 3. **Không đòi "đúng 1 khớp"** (lỗi #2) — đếm khớp trong biểu thức SỐ HỌC.
// 4. Chạy được lúc test bằng `fs`, theo khuôn `src/styles/app.css.test.ts`.

const GOC = process.cwd()
const THU_MUC_QUET = [
  path.join(GOC, 'src', 'lib', 'graph-render'),
  path.join(GOC, 'src', 'components', 'history'),
]
const TEP_NGUON_WIP = path.join(GOC, 'src', 'lib', 'graph-render', 'wipRow.ts')

/**
 * Bỏ chú thích khối, chú thích dòng, và chuỗi ký tự khỏi nguồn TypeScript.
 *
 * Thay bằng khoảng trắng chứ không xoá hẳn, để số dòng không đổi — thông điệp lỗi
 * của cổng còn chỉ đúng chỗ.
 *
 * 🔴 Đây là phần chống **lỗi #5**: grep trên nguồn **thô** khớp một chú thích do
 * chính mutation sinh ra, và cổng đỏ vì lý do sai. Đột biến M7 của bảng là đột biến
 * **nghịch** ghim đúng điều này — nó phải cho **0** test đỏ.
 */
export function boChuThichVaChuoi(src: string): string {
  let out = ''
  let i = 0
  const n = src.length
  while (i < n) {
    const c = src[i]
    const c2 = src[i + 1]
    // Chú thích khối
    if (c === '/' && c2 === '*') {
      const het = src.indexOf('*/', i + 2)
      const ket = het === -1 ? n : het + 2
      out += src.slice(i, ket).replace(/[^\n]/g, ' ')
      i = ket
      continue
    }
    // Chú thích dòng
    if (c === '/' && c2 === '/') {
      let j = i
      while (j < n && src[j] !== '\n') j++
      out += ' '.repeat(j - i)
      i = j
      continue
    }
    // Chuỗi nháy đơn / nháy kép / template
    if (c === "'" || c === '"' || c === '`') {
      const dau = c
      let j = i + 1
      while (j < n) {
        if (src[j] === '\\') {
          j += 2
          continue
        }
        if (src[j] === dau) {
          j++
          break
        }
        j++
      }
      out += src.slice(i, j).replace(/[^\n]/g, ' ')
      i = j
      continue
    }
    out += c
    i++
  }
  return out
}

/** Mọi tệp `.ts`/`.tsx` **không phải test** trong các thư mục quét. */
function tepNguonCanQuet(): { duongDan: string; ten: string }[] {
  const ket: { duongDan: string; ten: string }[] = []
  for (const thuMuc of THU_MUC_QUET) {
    for (const ten of readdirSync(thuMuc)) {
      if (!/\.tsx?$/.test(ten)) continue
      if (/\.test\.tsx?$/.test(ten)) continue
      if (ten === 'wipRow.ts') continue
      ket.push({ duongDan: path.join(thuMuc, ten), ten })
    }
  }
  return ket
}

/**
 * Tìm `WIP_ROW_HEIGHT` / `WIP_ROW_COUNT` / `contentOffset` đứng trong một **biểu
 * thức số học** (`+`, `-`, `*`, `/` ngay trước hoặc ngay sau).
 *
 * Không đếm mọi lần xuất hiện: import và chỗ gọi luôn ≥ 2 và cả hai đều hợp lệ —
 * đó là **lỗi #2** của bảng.
 */
function khopSoHoc(nguonDaLam: string): string[] {
  // `contentOffset(...)` nằm trong danh sách vì nó **là** phép cộng: một tệp khác
  // viết `v.start - scrollTop + contentOffset(hasWip)` đã dựng lại công thức của
  // `commitRowY` ở chỗ thứ hai, và hai công thức là hai dịp để lệch một hàng — đúng
  // thứ cổng này tồn tại để chặn. Chỗ gọi hợp lệ là `commitRowY(...)`, không phải tự
  // cộng lấy. Nhờ nó, `wipRow.ts` cũng THẬT SỰ chứa một khớp, nên tiền đề của cổng
  // không rỗng — bản đầu dùng toán tử ba ngôi nên không khớp gì và cổng vô nghĩa.
  const ten = '(?:WIP_ROW_HEIGHT|WIP_ROW_COUNT|contentOffset\\s*\\([^)]*\\))'
  const re = new RegExp(`(?:[-+*/]\\s*${ten})|(?:${ten}\\s*[-+*/])`, 'g')
  return nguonDaLam.match(re) ?? []
}

describe('🔴 cổng: mọi phép số học của hàng WIP nằm ở ĐÚNG MỘT chỗ', () => {
  it('TIỀN ĐỀ: đọc được wipRow.ts và nó CHỨA các hằng cần ghim', () => {
    // Lỗi #3: đường dẫn sai → không tìm thấy gì → xanh. Không tìm thấy = ĐỎ.
    const src = readFileSync(TEP_NGUON_WIP, 'utf8')
    expect(src.length, 'wipRow.ts không được rỗng').toBeGreaterThan(200)
    const sach = boChuThichVaChuoi(src)
    expect(sach, 'wipRow.ts phải khai WIP_ROW_HEIGHT').toContain('WIP_ROW_HEIGHT')
    expect(sach, 'wipRow.ts phải khai WIP_ROW_COUNT').toContain('WIP_ROW_COUNT')
    // Và nó PHẢI là chỗ có phép số học — nếu không, cổng dưới đây không ghim gì.
    expect(
      khopSoHoc(sach).length,
      'wipRow.ts phải là nơi CHỨA phép số học, nếu không cổng vô nghĩa',
    ).toBeGreaterThan(0)
  })

  it('TIỀN ĐỀ: quét được ít nhất 3 tệp nguồn khác', () => {
    // Lỗi #8 phiên bản đọc-tệp: quét 0 tệp thì "không tệp nào vi phạm" là vô nghĩa.
    const tep = tepNguonCanQuet()
    expect(tep.length, `chỉ quét được ${tep.length} tệp — đường dẫn sai?`).toBeGreaterThanOrEqual(3)
  })

  it('KHÔNG tệp nào khác trong graph-render/ và components/history/ làm số học với hằng WIP', () => {
    const viPham: string[] = []
    for (const { duongDan, ten } of tepNguonCanQuet()) {
      const sach = boChuThichVaChuoi(readFileSync(duongDan, 'utf8'))
      const khop = khopSoHoc(sach)
      if (khop.length > 0) viPham.push(`${ten}: ${khop.join(', ')}`)
    }
    expect(
      viPham,
      'Mọi phép ±1 của hàng WIP phải nằm ở wipRow.ts. Hai chỗ là đủ để cột đồ thị ' +
        'lệch cột văn bản đúng một hàng — đã xảy ra HAI lần ở Phase 2.',
    ).toEqual([])
  })

  /**
   * Cổng tự kiểm: phép bỏ chú thích **thật sự** bỏ được chú thích.
   *
   * Không có test này thì `boChuThichVaChuoi` có thể là hàm đồng nhất và cổng trên
   * vẫn "xanh" — đúng lớp lỗi #5 đi vào bằng cửa sau.
   */
  it('phép bỏ chú thích thật sự loại được chú thích và chuỗi (tự kiểm)', () => {
    const mau = [
      '// WIP_ROW_HEIGHT + 1',
      '/* WIP_ROW_HEIGHT * 2 */',
      'const s = "WIP_ROW_HEIGHT - 1"',
      'const t = `WIP_ROW_HEIGHT + 3`',
    ].join('\n')
    const sach = boChuThichVaChuoi(mau)
    expect(khopSoHoc(sach), 'chú thích và chuỗi KHÔNG phải mã').toEqual([])

    // Chiều ngược lại: mã thật thì PHẢI bị bắt.
    expect(khopSoHoc(boChuThichVaChuoi('const y = start + WIP_ROW_HEIGHT'))).toHaveLength(1)
    expect(khopSoHoc(boChuThichVaChuoi('const y = WIP_ROW_HEIGHT - 1'))).toHaveLength(1)
    expect(
      khopSoHoc(boChuThichVaChuoi('const y = v.start - scrollTop + contentOffset(hasWip)')),
    ).toHaveLength(1)

    // Chiều thứ ba: chỗ gọi **hợp lệ** KHÔNG bị bắt. Một cổng bắt cả `commitRowY(...)`
    // và dòng import sẽ cấm luôn việc *dùng* module — nó sẽ đỏ mãi mãi ở 04-05 và bị
    // ai đó tắt đi, tức là cổng tự huỷ.
    expect(khopSoHoc(boChuThichVaChuoi('const y = commitRowY(v.start, top, hasWip)'))).toEqual([])
    expect(khopSoHoc(boChuThichVaChuoi("import { WIP_ROW_HEIGHT } from './wipRow'"))).toEqual([])
  })
})
