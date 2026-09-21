/**
 * Điểm nối của checkpoint #2 (canvas hay SVG) — ROADMAP Phase 2.
 *
 * `GraphCanvas.tsx` (Task 2) chỉ gọi qua `GraphRenderer`, KHÔNG BAO GIỜ gọi
 * `getContext('2d')` hay bất kỳ API canvas nào trực tiếp. Đổi bộ vẽ sang SVG
 * nghĩa là viết một `svgRenderer.ts` mới cài đúng interface này, rồi đổi
 * **một** dòng import ở `GraphCanvas.tsx` — không sửa gì khác.
 *
 * Nếu một lời gọi canvas API rò ra ngoài `canvasRenderer.ts` (ví dụ
 * `GraphCanvas.tsx` tự gọi `getContext`), interface này đã hỏng và checkpoint
 * #2 mất đường lùi. Cổng `grep -c "getContext" GraphCanvas.tsx` → 0 kiểm đúng
 * điều đó.
 */

import type { GraphRow } from '@/lib/ipc'

/** Một hàng cần vẽ: dữ liệu hình học của backend + vị trí Y đã tính bởi virtualizer. */
export interface GraphRenderRow {
  row: GraphRow
  index: number
  /** Toạ độ Y (px) — PHẢI đến từ cùng `virtualItems` mà `CommitList` dùng cho văn bản. */
  y: number
}

/**
 * Hợp đồng mà mọi bộ vẽ đồ thị phải cài, bất kể canvas hay SVG.
 *
 * `resize`/`draw`/`dispose` là toàn bộ bề mặt — không thêm phương thức nào
 * đặc thù canvas (ví dụ không có `getContext()` trên interface này).
 */
export interface GraphRenderer {
  /** Đặt kích thước CSS và devicePixelRatio. Gọi lại khi kích thước hoặc dpr đổi. */
  resize(cssWidth: number, cssHeight: number, dpr: number): void
  /** Vẽ lại toàn bộ các hàng hiện có trong viewport. */
  draw(rows: GraphRenderRow[], selectedCommitId: string | null): void
  /** Dọn dẹp phần tử đã tạo trong host. Gọi một lần lúc unmount. */
  dispose(): void
}

/** Hàm dựng một bộ vẽ, gắn vào `host`. `createCanvasRenderer` cài đúng chữ ký này. */
export type GraphRendererFactory = (host: HTMLElement) => GraphRenderer
