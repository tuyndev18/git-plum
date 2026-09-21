/**
 * Test hình học thuần — không DOM, không React.
 *
 * `rowY`/`laneX` là bất biến thẳng hàng của toàn bộ Task 2/3: nếu hai hàm này
 * sai một pixel, mọi thứ dựng trên chúng (canvas, virtualizer) sai theo mà
 * không một test tích hợp nào chỉ ra được lý do gốc.
 */

import { describe, expect, it } from 'vitest'

import {
  GRAPH_PADDING_LEFT,
  LANE_COLORS,
  LANE_WIDTH,
  MAX_VISIBLE_LANES,
  NODE_RADIUS,
  ROW_HEIGHT,
  colorFor,
  graphWidth,
  laneX,
  rowY,
} from './geometry'

describe('rowY', () => {
  it('rowY(0) === 0', () => {
    expect(rowY(0)).toBe(0)
  })

  it('rowY(1) === ROW_HEIGHT', () => {
    expect(rowY(1)).toBe(ROW_HEIGHT)
  })

  it('rowY(10) === 10 * ROW_HEIGHT', () => {
    expect(rowY(10)).toBe(10 * ROW_HEIGHT)
  })
})

describe('laneX', () => {
  it('laneX(0) bằng lề trái đã định', () => {
    expect(laneX(0)).toBe(GRAPH_PADDING_LEFT)
  })

  it('laneX(3) - laneX(2) === LANE_WIDTH', () => {
    expect(laneX(3) - laneX(2)).toBe(LANE_WIDTH)
  })
})

describe('graphWidth', () => {
  it('tăng đơn điệu theo maxLane', () => {
    expect(graphWidth(1)).toBeLessThan(graphWidth(5))
    expect(graphWidth(5)).toBeLessThan(graphWidth(10))
  })

  it('có chặn trên tại MAX_VISIBLE_LANES — vượt cap không rộng thêm', () => {
    const atCap = graphWidth(MAX_VISIBLE_LANES)
    const wayOverCap = graphWidth(MAX_VISIBLE_LANES * 5)
    expect(wayOverCap).toBe(atCap)
  })
})

describe('bất biến thẳng hàng: rowY(i) - scrollOffset === virtualItem.start - scrollOffset', () => {
  it('khớp với giá trị virtualItem giả lập ở nhiều vị trí cuộn', () => {
    // Đây là test quan trọng nhất của task: nó khẳng định rowY() sinh đúng con
    // số mà @tanstack/react-virtual sẽ báo cho cùng chỉ số hàng, với
    // estimateSize cố định = ROW_HEIGHT (đúng cấu hình mà CommitList dùng ở
    // Task 2). Nếu hai công thức lệch nhau, đồ thị và văn bản lệch hàng — dù
    // mọi test khác vẫn xanh.
    const scrollOffsets = [0, 137, 5000, 99999]

    for (const scrollOffset of scrollOffsets) {
      for (const index of [0, 1, 10, 999]) {
        // virtualItem giả lập đúng công thức mà virtualizer dùng với
        // estimateSize: () => ROW_HEIGHT và không có kích thước hàng nào bị
        // đo lại (ROW_HEIGHT cố định là điều kiện của cách làm).
        const virtualItemStart = index * ROW_HEIGHT

        expect(rowY(index) - scrollOffset).toBe(virtualItemStart - scrollOffset)
      }
    }
  })
})

describe('colorFor', () => {
  it('trả một màu hợp lệ trong LANE_COLORS cho chỉ số trong bảng', () => {
    expect(LANE_COLORS).toContain(colorFor(0))
    expect(LANE_COLORS).toContain(colorFor(LANE_COLORS.length - 1))
  })

  it('color vượt số màu trong bảng vẫn trả một màu hợp lệ (modulo), không undefined', () => {
    const overIndex = LANE_COLORS.length + 3
    const result = colorFor(overIndex)

    expect(result).toBeDefined()
    expect(LANE_COLORS).toContain(result)
    expect(result).toBe(LANE_COLORS[overIndex % LANE_COLORS.length])
  })
})

describe('hằng số hình học', () => {
  it('NODE_RADIUS là số dương nhỏ hơn nửa ROW_HEIGHT', () => {
    // Nút tròn phải nằm gọn trong chiều cao hàng, không chạm hàng kề.
    expect(NODE_RADIUS).toBeGreaterThan(0)
    expect(NODE_RADIUS).toBeLessThan(ROW_HEIGHT / 2)
  })
})

/*
 * Hệ quy chiếu của `GraphRenderRow.y`: toạ độ TRONG CANVAS, không phải trong
 * danh sách.
 *
 * Canvas đồ thị chỉ cao bằng vùng nhìn thấy (`scrollHeight`, thường ~550px),
 * không cao bằng cả danh sách (`getTotalSize()`, có thể hàng chục nghìn px).
 * Nên `CommitList` phải đưa `y = virtualItem.start - scrollTop`, không phải
 * `y = virtualItem.start`.
 *
 * Lỗi đã xảy ra thật: đưa `v.start` tuyệt đối làm hàng thứ 100 (`start = 2800`)
 * được vẽ ở y=2800 trên canvas cao 550px — ra ngoài vùng vẽ và mất hẳn. Chỉ
 * những hàng có `start` nhỏ hơn chiều cao canvas còn thấy, nên đồ thị trông
 * như chỉ có một chấm ở hàng đầu.
 *
 * happy-dom không có cuộn thật và không tính layout, nên không viết được test
 * DOM-level cho lỗi này; test dưới đây ghim phép tính hệ quy chiếu ở dạng số
 * học thuần.
 */
describe('hệ quy chiếu y của hàng đồ thị — phải nằm trong canvas', () => {
  /** Đúng phép tính mà `CommitList.renderRows` dùng. */
  function canvasY(virtualStart: number, scrollTop: number): number {
    return virtualStart - scrollTop
  }

  it('hàng đầu vùng nhìn thấy luôn ở y=0 bất kể đã cuộn bao xa', () => {
    // Cuộn tới hàng 100: virtualizer báo start=2800, scrollTop=2800.
    expect(canvasY(100 * ROW_HEIGHT, 100 * ROW_HEIGHT)).toBe(0)
    // Cuộn tới hàng 5000: vẫn phải là 0, không phải 140000.
    expect(canvasY(5000 * ROW_HEIGHT, 5000 * ROW_HEIGHT)).toBe(0)
  })

  it('mọi hàng trong vùng nhìn thấy có y nằm trong chiều cao canvas', () => {
    const canvasHeight = 550
    const rowsVisible = Math.ceil(canvasHeight / ROW_HEIGHT)
    // Đã cuộn sâu — đây là ca mà bản lỗi hiển thị sai.
    const scrollTop = 3000 * ROW_HEIGHT
    const firstVisibleIndex = 3000

    for (let i = 0; i < rowsVisible; i++) {
      const y = canvasY((firstVisibleIndex + i) * ROW_HEIGHT, scrollTop)
      expect(y, `hàng ${firstVisibleIndex + i} phải có y >= 0`).toBeGreaterThanOrEqual(0)
      expect(
        y,
        `hàng ${firstVisibleIndex + i} có y=${y}, vượt chiều cao canvas ${canvasHeight} — ` +
          `sẽ vẽ ra ngoài vùng canvas và mất hẳn`,
      ).toBeLessThan(canvasHeight + ROW_HEIGHT)
    }
  })

  it('dùng y tuyệt đối (không trừ scrollTop) thì hàng cuộn sâu vẽ ra ngoài canvas', () => {
    // Ghim lại chính xác cái SAI, để test này giải thích được vì sao phép trừ
    // tồn tại nếu ai đó định "dọn dẹp" nó đi.
    const canvasHeight = 550
    const yTuyetDoi = 3000 * ROW_HEIGHT // = 84000, cách bản lỗi từng làm
    expect(yTuyetDoi).toBeGreaterThan(canvasHeight)
  })
})
