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
import type { GraphRenderer, GraphRenderRow } from '@/lib/graph-render/types'

interface Props {
  rows: GraphRenderRow[]
  width: number
  height: number
  selectedCommitId: string | null
}

export function GraphCanvas({ rows, width, height, selectedCommitId }: Props) {
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

  // `devicePixelRatio` đọc lại ở đây (không đọc trong canvasRenderer) để bộ
  // vẽ vẫn test được mà không phải vá `window` — người dùng kéo cửa sổ sang
  // màn hình khác thì effect này chạy lại vì `width`/`height` đổi theo layout.
  useEffect(() => {
    rendererRef.current?.resize(width, height, window.devicePixelRatio)
  }, [width, height])

  useEffect(() => {
    rendererRef.current?.draw(rows, selectedCommitId)
  }, [rows, selectedCommitId])

  // Canvas thuần trình bày (ràng buộc số 5): không bắt sự kiện bấm nào ở đây
  // hay trong CSS `.graph-canvas` (`pointer-events: none`). Chọn hàng là việc
  // của `div.commit-row` trong `CommitList.tsx`.
  return <div ref={hostRef} className="graph-canvas" style={{ width, height }} />
}
