/**
 * Test bản cài canvas — dùng ngữ cảnh vẽ giả lập ghi lại lời gọi, KHÔNG vá
 * `getContext` toàn cục. `createCanvasRenderer` nhận một tham số tuỳ chọn thứ
 * hai để tiêm ngữ cảnh giả — xem `types.ts` cho chữ ký đầy đủ.
 */

import { describe, expect, it, vi } from 'vitest'

import { createCanvasRenderer } from './canvasRenderer'
import { ROW_HEIGHT } from './geometry'
import type { GraphRenderRow } from './types'
import type { GraphRow } from '@/lib/ipc'

/** Ngữ cảnh 2D giả — ghi lại mọi lời gọi vẽ mà không chạm DOM canvas thật. */
function fakeContext() {
  return {
    calls: [] as Array<{ name: string; args: unknown[] }>,
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
    set strokeStyle(v: string) {
      this._strokeStyle = v
    },
    get strokeStyle() {
      return this._strokeStyle
    },
    set fillStyle(v: string) {
      this._fillStyle = v
    },
    get fillStyle() {
      return this._fillStyle
    },
    set lineWidth(v: number) {
      this._lineWidth = v
    },
    get lineWidth() {
      return this._lineWidth
    },
    _strokeStyle: '',
    _fillStyle: '',
    _lineWidth: 1,
  }
}

/** `<canvas>` giả — chỉ đủ để `createCanvasRenderer` không chạm DOM thật. */
function fakeHost(ctx: ReturnType<typeof fakeContext>) {
  const canvasEl = {
    width: 0,
    height: 0,
    style: { width: '', height: '' },
    getContext: vi.fn(() => ctx),
  }
  const host = {
    appendChild: vi.fn(),
    removeChild: vi.fn(),
  }
  return { host, canvasEl }
}

function baseRow(overrides: Partial<GraphRow> = {}): GraphRow {
  return {
    commitId: 'c1',
    lane: 0,
    color: 0,
    passthrough: [],
    outEdges: [],
    truncatedParents: 0,
    terminates: false,
    ...overrides,
  }
}

function renderRow(row: GraphRow, index = 0): GraphRenderRow {
  return { row, index, y: index * ROW_HEIGHT }
}

describe('createCanvasRenderer', () => {
  it('outEdges 0->0 sinh đúng một đường thẳng dọc', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const row = baseRow({ outEdges: [{ fromLane: 0, toLane: 0, color: 0 }] })
    renderer.draw([renderRow(row)], null)

    // Đường dọc: moveTo và lineTo dùng cùng toạ độ x (không có bezier — đường
    // thẳng không cần cong).
    expect(ctx.moveTo).toHaveBeenCalled()
    expect(ctx.lineTo).toHaveBeenCalled()
    const moveArgs = ctx.moveTo.mock.calls[0] as [number, number]
    const lineArgs = ctx.lineTo.mock.calls[0] as [number, number]
    expect(moveArgs[0]).toBe(lineArgs[0])
    expect(ctx.bezierCurveTo).not.toHaveBeenCalled()
  })

  it('outEdges 0->2 sinh một đường chéo có toạ độ x đầu/cuối khác nhau', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const row = baseRow({ outEdges: [{ fromLane: 0, toLane: 2, color: 0 }] })
    renderer.draw([renderRow(row)], null)

    expect(ctx.moveTo).toHaveBeenCalled()
    // Đường chéo dùng bezier để cong mượt giữa hai lane khác nhau.
    expect(ctx.bezierCurveTo).toHaveBeenCalled()
    const moveArgs = ctx.moveTo.mock.calls[0] as [number, number]
    const bezierArgs = ctx.bezierCurveTo.mock.calls[0] as [
      number,
      number,
      number,
      number,
      number,
      number,
    ]
    const endX = bezierArgs[4]
    expect(moveArgs[0]).not.toBe(endX)
  })

  it('passthrough hai cạnh sinh hai đoạn thẳng dọc, cộng nút tròn của hàng đó', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const row = baseRow({
      passthrough: [
        { fromLane: 3, toLane: 3, color: 1 },
        { fromLane: 5, toLane: 5, color: 2 },
      ],
    })
    renderer.draw([renderRow(row)], null)

    // Hai đoạn passthrough → ít nhất hai cặp moveTo/lineTo, cộng arc cho nút.
    expect(ctx.moveTo.mock.calls.length).toBeGreaterThanOrEqual(2)
    expect(ctx.arc).toHaveBeenCalled()
  })

  it('truncatedParents > 0 sinh một chỉ báo, không bị bỏ qua', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const row = baseRow({ truncatedParents: 3 })
    renderer.draw([renderRow(row)], null)

    // Chỉ báo là lời gọi vẽ chữ hoặc một hình riêng — ở đây bản cài dùng fillText.
    expect(ctx.fillText).toHaveBeenCalled()
    const text = ctx.fillText.mock.calls.map((c) => String(c[0])).join(' ')
    expect(text).toContain('3')
  })

  it('terminates === true sinh dấu hiệu kết thúc khác với nút tròn thường', () => {
    const ctxNormal = fakeContext()
    const { host: hostNormal, canvasEl: canvasNormal } = fakeHost(ctxNormal)
    const rendererNormal = createCanvasRenderer(hostNormal as unknown as HTMLElement, {
      createCanvasElement: () => canvasNormal as unknown as HTMLCanvasElement,
    })
    rendererNormal.resize(800, 600, 1)
    rendererNormal.draw([renderRow(baseRow())], null)

    const ctxTerminates = fakeContext()
    const { host: hostTerm, canvasEl: canvasTerm } = fakeHost(ctxTerminates)
    const rendererTerm = createCanvasRenderer(hostTerm as unknown as HTMLElement, {
      createCanvasElement: () => canvasTerm as unknown as HTMLCanvasElement,
    })
    rendererTerm.resize(800, 600, 1)
    rendererTerm.draw([renderRow(baseRow({ terminates: true }))], null)

    // Hàng terminates dùng setLineDash (nét đứt) hoặc số lời gọi arc khác —
    // bản cài dùng setLineDash làm dấu hiệu riêng cho nút kết thúc.
    expect(ctxTerminates.setLineDash).toHaveBeenCalled()
    expect(ctxNormal.setLineDash.mock.calls.length).not.toBe(
      ctxTerminates.setLineDash.mock.calls.length,
    )
  })

  it('devicePixelRatio 2: canvas.width gấp đôi chiều rộng CSS, đúng một lần scale(2,2)', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })

    renderer.resize(400, 300, 2)

    expect(canvasEl.width).toBe(800)
    expect(canvasEl.height).toBe(600)
    expect(canvasEl.style.width).toBe('400px')
    expect(canvasEl.style.height).toBe('300px')
    expect(ctx.scale).toHaveBeenCalledTimes(1)
    expect(ctx.scale).toHaveBeenCalledWith(2, 2)
  })

  it('color vượt số màu trong bảng vẫn trả một màu hợp lệ (không undefined) khi vẽ', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    // color=99 vượt xa bảng màu — không được làm fillStyle/strokeStyle thành
    // undefined hay ném lỗi.
    const row = baseRow({ color: 99, outEdges: [{ fromLane: 0, toLane: 0, color: 99 }] })
    expect(() => renderer.draw([renderRow(row)], null)).not.toThrow()
  })

  it('dispose không ném lỗi và có thể gọi an toàn', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)
    renderer.draw([renderRow(baseRow())], null)

    expect(() => renderer.dispose()).not.toThrow()
  })
})

/*
 * Mọi đường của một hàng phải nằm TRONG ô của hàng đó: `[y, y + ROW_HEIGHT]`.
 *
 * Bản cài đầu tiên vẽ `outEdges` tới `y + 1.5 * ROW_HEIGHT` — tràn nửa hàng
 * xuống ô của hàng dưới và vẽ đè lên đường mà hàng đó tự vẽ, ở lane khác nhau.
 * Đó là nguyên nhân người dùng báo đồ thị "rất khó nhìn và vỡ" khi có nhánh.
 *
 * Các test `outEdges` phía trên KHÔNG bắt được lỗi này vì chúng chỉ khẳng định
 * toạ độ **x** (`moveArgs[0]` so với `lineArgs[0]`) và việc có gọi `bezierCurveTo`
 * — không một test nào kiểm toạ độ **y**. Đây chính là loại lỗ hổng "test xanh
 * không chứng minh được hành vi" đã gặp ở mutation #4 của plan 02-06 và
 * mutation #6 của plan 02-05.
 */
describe('mọi đường vẽ phải nằm trong ô của hàng — không tràn sang hàng dưới', () => {
  /** Thu mọi toạ độ y mà một lượt vẽ chạm tới. */
  function allDrawnY(ctx: ReturnType<typeof fakeContext>): number[] {
    const ys: number[] = []
    for (const call of ctx.moveTo.mock.calls) ys.push((call as [number, number])[1])
    for (const call of ctx.lineTo.mock.calls) ys.push((call as [number, number])[1])
    for (const call of ctx.bezierCurveTo.mock.calls) {
      const [, cp1y, , cp2y, , yEnd] = call as [number, number, number, number, number, number]
      ys.push(cp1y, cp2y, yEnd)
    }
    return ys
  }

  it('outEdges dọc không vẽ quá mép dưới của hàng', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    // Hàng thứ 3 để mép trên/dưới khác 0 — lỗi tràn sẽ lộ rõ hơn hàng 0.
    const index = 3
    const y = index * ROW_HEIGHT
    const row = baseRow({ outEdges: [{ fromLane: 0, toLane: 0, color: 0 }] })
    renderer.draw([renderRow(row, index)], null)

    const ys = allDrawnY(ctx)
    expect(ys.length, 'phải có toạ độ y được vẽ').toBeGreaterThan(0)
    for (const drawnY of ys) {
      expect(
        drawnY,
        `đường vẽ tới y=${drawnY} nhưng ô của hàng ${index} chỉ là [${y}, ${y + ROW_HEIGHT}] — ` +
          `vẽ quá mép dưới nghĩa là đè lên ô của hàng kế tiếp`,
      ).toBeLessThanOrEqual(y + ROW_HEIGHT)
      expect(drawnY, `đường vẽ tới y=${drawnY}, trên mép trên ${y} của hàng`).toBeGreaterThanOrEqual(y)
    }
  })

  it('outEdges chéo (bezier) cũng không vẽ quá mép dưới của hàng', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const index = 3
    const y = index * ROW_HEIGHT
    // Rẽ nhánh sang lane 2 — đây là ca tạo ra hình "vỡ" mà người dùng thấy.
    const row = baseRow({ outEdges: [{ fromLane: 0, toLane: 2, color: 0 }] })
    renderer.draw([renderRow(row, index)], null)

    for (const drawnY of allDrawnY(ctx)) {
      expect(
        drawnY,
        `đường chéo vẽ tới y=${drawnY} nhưng ô của hàng ${index} chỉ là [${y}, ${y + ROW_HEIGHT}]`,
      ).toBeLessThanOrEqual(y + ROW_HEIGHT)
    }
  })

  it('outEdges bắt đầu từ TÂM hàng, passthrough từ MÉP TRÊN — hai nửa khớp nhau', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const index = 2
    const y = index * ROW_HEIGHT

    // Chỉ passthrough: phải chạy hết chiều cao hàng, mép trên -> mép dưới.
    const pass = baseRow({ passthrough: [{ fromLane: 0, toLane: 0, color: 0 }] })
    renderer.draw([renderRow(pass, index)], null)
    expect(ctx.moveTo.mock.calls[0]?.[1], 'passthrough phải bắt đầu ở mép TRÊN hàng').toBe(y)
    expect(ctx.lineTo.mock.calls[0]?.[1], 'passthrough phải kết thúc ở mép DƯỚI hàng').toBe(
      y + ROW_HEIGHT,
    )

    // Chỉ outEdges: ô của hàng phải được LÁT KÍN theo chiều dọc trên lane của
    // nút — đoạn vào (mép trên -> tâm) rồi đoạn ra (tâm -> mép dưới).
    //
    // Khẳng định theo TẬP đoạn được vẽ chứ không theo thứ tự lời gọi: đoạn vào
    // là một nét riêng do `drawRow` sinh (không đến từ `passthrough` hay
    // `outEdges` của backend), nên bám `moveTo.mock.calls[0]` sẽ gãy chỉ vì
    // đổi thứ tự vẽ, trong khi điều thật sự phải đúng là "không còn khe hở
    // giữa mép trên và mép dưới".
    const out = fakeContext()
    const h2 = fakeHost(out)
    const r2 = createCanvasRenderer(h2.host as unknown as HTMLElement, {
      createCanvasElement: () => h2.canvasEl as unknown as HTMLCanvasElement,
    })
    r2.resize(800, 600, 1)
    const outRow = baseRow({ outEdges: [{ fromLane: 0, toLane: 0, color: 0 }] })
    r2.draw([renderRow(outRow, index)], null)

    const segments = out.moveTo.mock.calls.map((call, i) => ({
      from: (call as [number, number])[1],
      to: (out.lineTo.mock.calls[i] as [number, number] | undefined)?.[1],
    }))

    expect(segments, 'phải có đoạn vào: mép TRÊN -> TÂM (nếu thiếu, nút trôi nổi)').toContainEqual({
      from: y,
      to: y + ROW_HEIGHT / 2,
    })
    expect(segments, 'phải có đoạn ra: TÂM -> mép DƯỚI').toContainEqual({
      from: y + ROW_HEIGHT / 2,
      to: y + ROW_HEIGHT,
    })
  })

  /**
   * `passthrough` KHÔNG phải lúc nào cũng thẳng đứng.
   *
   * `lanes.rs` bước 2 đẩy vào `passthrough` cả các cạnh HỢP NHÁNH
   * (`from_lane != to_lane`): một lane khác đang chờ đúng commit này nên nó rẽ
   * vào lane của hàng rồi CHẾT ở đó. Bộ vẽ từng giả định mọi phần tử của
   * `passthrough` là đường dọc mép-trên -> mép-dưới, nên nó kéo cạnh hợp nhánh
   * xuyên qua mép dưới vào một lane đã được giải phóng — sinh ra các đoạn cụt
   * không nối vào đâu, đúng hiện tượng "đồ thị vỡ" người dùng báo.
   */
  it('passthrough hợp nhánh (fromLane != toLane) dừng ở TÂM, không xuyên mép dưới', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(800, 600, 1)

    const index = 2
    const y = index * ROW_HEIGHT
    // Lane 2 chờ commit này, hàng nằm ở lane 0 → cạnh hợp nhánh 2 -> 0.
    const row = baseRow({ lane: 0, passthrough: [{ fromLane: 2, toLane: 0, color: 2 }] })
    renderer.draw([renderRow(row, index)], null)

    const endsAtBottom = ctx.lineTo.mock.calls.some(
      (call) => (call as [number, number])[1] === y + ROW_HEIGHT,
    )
    expect(
      endsAtBottom,
      'cạnh hợp nhánh không được chạy xuống mép dưới — lane nguồn đã chết ở hàng này',
    ).toBe(false)
  })
})

/*
 * Chấm commit phải nằm ĐÚNG GIỮA hàng theo chiều dọc.
 *
 * Cột chữ dùng `.commit-row { align-items: center }` với hàng cao `ROW_HEIGHT`
 * đặt tại `translateY(v.start)`, nên chữ căn giữa tại `v.start + ROW_HEIGHT/2`.
 * Canvas phải vẽ chấm tại đúng cao độ đó (đã trừ `scrollTop` — xem
 * `CommitList.renderRows`), nếu không chấm trôi lên mép hàng và không còn chỉ
 * đúng vào dòng chữ của chính nó.
 *
 * git-plum KHÔNG có avatar tác giả (ràng buộc Privacy: không gọi mạng), nên
 * nút luôn là chấm tròn đặc màu lane — không có ca nào vẽ ảnh vuông lệch tâm.
 */
describe('chấm commit nằm giữa hàng theo chiều dọc', () => {
  it('tâm chấm = y + ROW_HEIGHT / 2 ở mọi hàng', () => {
    const ctx = fakeContext()
    const { host, canvasEl } = fakeHost(ctx)
    const renderer = createCanvasRenderer(host as unknown as HTMLElement, {
      createCanvasElement: () => canvasEl as unknown as HTMLCanvasElement,
    })
    renderer.resize(400, 400, 1)

    const indices = [0, 1, 5]
    renderer.draw(
      indices.map((i) => renderRow(baseRow({ commitId: `c${i}` }), i)),
      null,
    )

    // `arc` được gọi cho quầng nền và cho chấm; mọi lời gọi đều phải ở tâm hàng.
    const centers = ctx.arc.mock.calls.map((call) => (call as [number, number, number])[1])
    expect(centers.length, 'phải có chấm được vẽ').toBeGreaterThan(0)

    const expected = indices.map((i) => i * ROW_HEIGHT + ROW_HEIGHT / 2)
    for (const cy of centers) {
      expect(
        expected,
        `chấm vẽ ở y=${cy} nhưng tâm các hàng là ${expected.join(', ')} — ` +
          `chấm lệch khỏi giữa hàng thì nó không còn chỉ đúng dòng chữ của mình`,
      ).toContain(cy)
    }
  })
})
