/**
 * Bọc mỏng quanh bộ vẽ đồ thị — checkpoint #2 (canvas hay SVG).
 *
 * **Không import gì từ canvas API.** Chỉ import `GraphRenderer`,
 * `createCanvasRenderer` — đổi sang SVG là viết `svgRenderer.ts` cài đúng
 * `GraphRenderer` rồi đổi một dòng import dưới đây, không sửa gì khác trong
 * tệp này.
 */

import { useEffect, useRef } from 'react'

import { createCanvasRenderer } from '@/lib/graph-render/canvasRenderer'
import type {
  GraphRenderer,
  GraphRenderRow,
  WipEdgeRender,
} from '@/lib/graph-render/types'

interface Props {
  rows: GraphRenderRow[]
  width: number
  height: number
  selectedCommitId: string | null
  /**
   * Cạnh hàng WIP — WORK-11. `null`/vắng = không có hàng WIP.
   *
   * Đi qua **prop rồi qua tham số của `draw`**, không qua một phương thức mới
   * trên `GraphRenderer`: xem `WipEdgeRender` trong `types.ts` về lý do, và
   * checkpoint #2 của ROADMAP (đổi canvas ↔ SVG phải là sửa một tệp).
   */
  wip?: WipEdgeRender | null
}

export function GraphCanvas({ rows, width, height, selectedCommitId, wip }: Props) {
  const hostRef = useRef<HTMLDivElement>(null)
  const rendererRef = useRef<GraphRenderer | null>(null)

  // Tạo renderer đúng một lần lúc gắn kết, dọn dẹp lúc unmount.
  useEffect(() => {
    const host = hostRef.current
    if (!host) return

    const renderer = createCanvasRenderer(host)
    rendererRef.current = renderer

    return () => {
      renderer.dispose()
      rendererRef.current = null
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // `resize` và `draw` phải nằm trong CÙNG một effect, theo đúng thứ tự.
  //
  // Gán `canvas.width`/`canvas.height` **xoá sạch** canvas và đặt lại ma trận
  // biến đổi — đó là hành vi của chính DOM, không phải của bộ vẽ. Tách làm hai
  // effect với hai mảng phụ thuộc khác nhau thì có một đường chạy để lại màn
  // hình sai: `width` đổi (rất hay xảy ra — `graphWidth(maxLane)` đổi mỗi khi
  // số lane lớn nhất trong vùng nhìn thấy đổi lúc cuộn) trong khi `rows` giữ
  // nguyên danh tính, nên `resize` chạy và xoá canvas còn `draw` KHÔNG chạy
  // lại. Kết quả: cột đồ thị trống trơn hoặc còn lại hình vẽ dở của lần trước,
  // trong khi cột chữ vẫn đủ hàng.
  //
  // `devicePixelRatio` đọc ở đây (không đọc trong `canvasRenderer`) để bộ vẽ
  // vẫn test được mà không phải vá `window`.
  useEffect(() => {
    const renderer = rendererRef.current
    if (!renderer) return
    renderer.resize(width, height, window.devicePixelRatio)
    renderer.draw(rows, selectedCommitId, wip)
    // `wip` nằm trong mảng phụ thuộc vì cùng lý do `width` nằm trong đó: nó đổi
    // được (stage hết tệp → hàng WIP biến mất) trong khi `rows` giữ nguyên danh
    // tính, và thiếu nó thì canvas giữ lại cạnh WIP của lần vẽ trước.
  }, [width, height, rows, selectedCommitId, wip])

  // Canvas thuần trình bày (ràng buộc số 5): không bắt sự kiện bấm nào ở đây
  // hay trong CSS `.graph-canvas` (`pointer-events: none`). Chọn hàng là việc
  // của `div.commit-row` trong `CommitList.tsx`.
  return <div ref={hostRef} className="graph-canvas" style={{ width, height }} />
}
