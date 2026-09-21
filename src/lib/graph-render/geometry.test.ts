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
