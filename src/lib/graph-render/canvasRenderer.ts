/**
 * Bản cài canvas 2D của `GraphRenderer` — checkpoint #2 (ROADMAP), nhánh "dựng
 * canvas trước".
 *
 * **Chỉ vẽ.** Không tính lane, không suy ra liên thông, không đọc
 * `commit.parents` — toàn bộ hình học đến từ `GraphRow` mà Rust đã tính (ràng
 * buộc số 1 của CONTEXT.md). Không gắn event listener nào lên canvas (ràng
 * buộc số 5 — canvas thuần trình bày; bắt sự kiện bấm là việc của `div` hàng
 * trong `CommitList.tsx`).
 */

import { colorFor, laneX, ROW_HEIGHT, NODE_RADIUS } from './geometry'
import type { GraphRenderer, GraphRendererFactory, GraphRenderRow } from './types'
import type { Edge } from '@/lib/ipc'

/** Ngữ cảnh vẽ 2D tối thiểu mà bản cài này cần — cho phép tiêm giả lập ở test. */
export interface DrawingContext2D {
  save(): void
  restore(): void
  scale(x: number, y: number): void
  clearRect(x: number, y: number, w: number, h: number): void
  beginPath(): void
  moveTo(x: number, y: number): void
  lineTo(x: number, y: number): void
  bezierCurveTo(
    cp1x: number,
    cp1y: number,
    cp2x: number,
    cp2y: number,
    x: number,
    y: number,
  ): void
  arc(x: number, y: number, radius: number, startAngle: number, endAngle: number): void
  stroke(): void
  fill(): void
  fillText(text: string, x: number, y: number): void
  setLineDash(segments: number[]): void
  strokeStyle: string
  fillStyle: string
  lineWidth: number
}

/** Điểm tiêm dùng riêng cho test — sản xuất thật dùng mặc định (canvas DOM thật). */
export interface CanvasRendererOptions {
  createCanvasElement?: () => HTMLCanvasElement
}

/** Vẽ một đoạn nối giữa hai lane. Dọc nếu cùng lane, cong (bezier) nếu khác lane. */
function drawEdge(ctx: DrawingContext2D, edge: Edge, yStart: number, yEnd: number): void {
  const xStart = laneX(edge.fromLane)
  const xEnd = laneX(edge.toLane)

  ctx.strokeStyle = colorFor(edge.color)
  ctx.beginPath()
  ctx.moveTo(xStart, yStart)

  if (edge.fromLane === edge.toLane) {
    ctx.lineTo(xEnd, yEnd)
  } else {
    // Đường cong bezier mượt giữa hai lane khác nhau, kiểm soát điểm giữa
    // theo chiều Y để đường rẽ không gãy góc.
    const midY = (yStart + yEnd) / 2
    ctx.bezierCurveTo(xStart, midY, xEnd, midY, xEnd, yEnd)
  }

  ctx.stroke()
}

/** Vẽ toàn bộ nội dung của một hàng: passthrough, outEdges, nút tròn, chỉ báo. */
function drawRow(ctx: DrawingContext2D, item: GraphRenderRow, selectedCommitId: string | null) {
  const { row, y } = item
  const yCenter = y + ROW_HEIGHT / 2
  const yBottom = y + ROW_HEIGHT

  // Đoạn đi ngang qua (không dừng ở hàng này) — luôn dọc theo định nghĩa của
  // `passthrough` (backend chỉ sinh passthrough khi from === to, xem
  // graph::types::Edge doc comment).
  for (const edge of row.passthrough) {
    drawEdge(ctx, edge, y, yBottom)
  }

  // Cạnh nối từ nút hàng này xuống MÉP DƯỚI của chính hàng này — **không** kéo
  // sang tận tâm hàng kế tiếp.
  //
  // Mỗi hàng chỉ được vẽ trong ô của chính nó: nửa dưới do `outEdges` của hàng
  // này vẽ (tâm -> mép dưới), nửa trên của hàng kế tiếp do `passthrough` hoặc
  // `outEdges` của hàng đó vẽ. Hai nửa gặp nhau ở mép chung nên đường liền
  // mạch.
  //
  // Vẽ tới `y + 1.5 * ROW_HEIGHT` (bản trước) làm đường tràn vào ô của hàng
  // dưới và **vẽ đè** lên đường mà hàng đó tự vẽ, ở lane khác nhau — đó là
  // nguyên nhân đồ thị trông chồng chéo và "vỡ" khi có nhánh. Lỗi càng rõ khi
  // virtualizer chỉ dựng các hàng đang thấy: đường tràn vẫn vẽ xuống một hàng
  // có thể chưa được dựng, nên nó không bao giờ khớp với gì cả.
  for (const edge of row.outEdges) {
    drawEdge(ctx, edge, yCenter, yBottom)
  }

  // Nút tròn của chính hàng này.
  const nodeX = laneX(row.lane)
  ctx.fillStyle = colorFor(row.color)
  ctx.beginPath()

  if (row.terminates) {
    // Hàng kết thúc (biên trang/shallow) — nét đứt quanh nút để phân biệt
    // trực quan với nút commit bình thường, không phải chỉ khác màu.
    ctx.setLineDash([2, 2])
    ctx.arc(nodeX, yCenter, NODE_RADIUS, 0, Math.PI * 2)
    ctx.strokeStyle = colorFor(row.color)
    ctx.stroke()
    ctx.setLineDash([])
  } else {
    ctx.arc(nodeX, yCenter, NODE_RADIUS, 0, Math.PI * 2)
    ctx.fill()
  }

  if (row.commitId === selectedCommitId) {
    ctx.beginPath()
    ctx.arc(nodeX, yCenter, NODE_RADIUS + 2, 0, Math.PI * 2)
    ctx.strokeStyle = colorFor(row.color)
    ctx.lineWidth = 2
    ctx.stroke()
    ctx.lineWidth = 1
  }

  if (row.truncatedParents > 0) {
    ctx.fillStyle = colorFor(row.color)
    ctx.fillText(`+${row.truncatedParents}`, nodeX + NODE_RADIUS + 2, yCenter + 3)
  }
}

/** `GraphRendererFactory` cài bằng canvas 2D. Tham số thứ hai chỉ dùng ở test. */
export const createCanvasRenderer: (
  host: HTMLElement,
  options?: CanvasRendererOptions,
) => GraphRenderer = (host, options) => {
  const canvas =
    options?.createCanvasElement?.() ??
    (typeof document !== 'undefined' ? document.createElement('canvas') : undefined)

  if (!canvas) {
    throw new Error('createCanvasRenderer: không có document và không có createCanvasElement')
  }

  canvas.style.position = canvas.style.position || 'absolute'
  host.appendChild(canvas)

  const ctx = canvas.getContext('2d') as unknown as DrawingContext2D | null
  if (!ctx) {
    throw new Error('createCanvasRenderer: getContext("2d") trả null')
  }

  let cssWidth = 0
  let cssHeight = 0

  const renderer: GraphRenderer = {
    resize(width, height, dpr) {
      cssWidth = width
      cssHeight = height
      canvas.width = Math.round(width * dpr)
      canvas.height = Math.round(height * dpr)
      canvas.style.width = `${width}px`
      canvas.style.height = `${height}px`
      // `scale` phải gọi đúng MỘT lần sau mỗi resize — gọi tích luỹ nhiều lần
      // sẽ khiến hình vẽ ra sai tỉ lệ ở lần resize thứ hai.
      ctx.scale(dpr, dpr)
    },

    draw(rows, selectedCommitId) {
      ctx.clearRect(0, 0, cssWidth, cssHeight)
      for (const item of rows) {
        drawRow(ctx, item, selectedCommitId)
      }
    },

    dispose() {
      host.removeChild(canvas)
    },
  }

  return renderer
}

// Khẳng định kiểu tĩnh: `createCanvasRenderer` (khi gọi với một tham số) khớp
// `GraphRendererFactory` — điểm nối của checkpoint #2.
const _typeCheck: GraphRendererFactory = (h) => createCanvasRenderer(h)
void _typeCheck
