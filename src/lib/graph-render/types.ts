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

  /**
   * Avatar tác giả vẽ đè lên nút commit. `undefined` khi hàng chưa nạp xong
   * dữ liệu commit (mảng `commits` là **thưa**).
   *
   * # Vì sao ở ĐÂY chứ không phải trên `GraphRow`
   *
   * `GraphRow` là kiểu **IPC**, do Rust sinh ra và Phase 4 cũng đang đọc. Nó
   * mô tả *hình học nhánh* — lane, màu, cạnh — và không biết gì về tác giả.
   * Thêm tên/email vào đó sẽ trộn hai tầng dữ liệu và bắt backend gửi kèm
   * thông tin tác giả trong mọi phép tính lane.
   *
   * `GraphRenderRow` thì ngược lại: nó là kiểu **của riêng bộ vẽ**, dựng ở
   * `CommitList` bằng cách ghép `GraphRow` với vị trí Y mà virtualizer tính.
   * Ghép thêm thông tin tác giả ở cùng chỗ đó là tự nhiên — `CommitList` đã
   * có `commits[index]` trong tay.
   */
  avatar:
    | {
        /** Chữ cái đại diện, đã tính sẵn ở `CommitList` (`chuDaiDien`). */
        chu: string
        /** Màu nền, đã tính sẵn (`avatarTuSinh`). */
        mauNen: string
      }
    | undefined
}

/**
 * Cạnh của hàng WIP cần vẽ, kèm vị trí Y của hàng WIP trong canvas — WORK-11.
 *
 * # 🔴 Vì sao đi qua **tham số của `draw`** chứ không phải một phương thức mới
 *
 * Checkpoint #2 của ROADMAP đòi đổi canvas ↔ SVG là sửa **một** tệp. Thêm một
 * `drawWipEdge()` lên [`GraphRenderer`] làm mọi bộ vẽ tương lai phải cài thêm
 * một phương thức, và nó mở đường cho phương thức thứ ba, thứ tư — tức là hợp
 * đồng phình ra theo tính năng, đúng thứ interface này tồn tại để chặn.
 *
 * Một tham số **tuỳ chọn** của `draw` thì ngược lại: bộ vẽ SVG viết sau vẫn
 * cài đúng ba phương thức cũ, và nó nhận cùng dữ liệu đã tính sẵn — cùng lập
 * luận mà `GraphRenderRow.avatar` đã dùng.
 *
 * `edge` là kết quả của `wipEdge()`, tính ở `CommitList`. Bộ vẽ **không** được
 * tự suy ra lane của HEAD: phép clamp có điều kiện nằm ở `wipRow.ts` và chỉ ở
 * đó.
 */
export interface WipEdgeRender {
  /**
   * Biến thể cạnh do `wipEdge()` trả về.
   *
   * `clamped` và `unknownHead` → bộ vẽ **không vẽ đường nào**. Vẽ vào lane 0
   * hoặc vào cột bị clamp là nối hàng WIP vào một commit **khác** — một đồ thị
   * sai một cách tự tin. Chỉ báo suy giảm là việc của DOM, không của canvas.
   */
  kind: 'normal' | 'clamped' | 'unknownHead'
  /** Toạ độ x của lane HEAD, px. Chỉ có ở `kind === 'normal'`. */
  x?: number
  /** Toạ độ Y (px, trong canvas) của **mép trên** hàng WIP. */
  y: number
}

/**
 * Hợp đồng mà mọi bộ vẽ đồ thị phải cài, bất kể canvas hay SVG.
 *
 * `resize`/`draw`/`dispose` là toàn bộ bề mặt — không thêm phương thức nào
 * đặc thù canvas (ví dụ không có `getContext()` trên interface này), và
 * **không** thêm phương thức riêng cho hàng WIP (xem [`WipEdgeRender`]).
 */
export interface GraphRenderer {
  /** Đặt kích thước CSS và devicePixelRatio. Gọi lại khi kích thước hoặc dpr đổi. */
  resize(cssWidth: number, cssHeight: number, dpr: number): void
  /**
   * Vẽ lại toàn bộ các hàng hiện có trong viewport.
   *
   * `wip` vắng mặt (`undefined`) = không có hàng WIP → hành vi **y hệt** trước
   * 04-05. Đó là lý do nó là tham số tuỳ chọn thứ ba chứ không phải một đổi
   * thay phá vỡ chữ ký.
   */
  draw(
    rows: GraphRenderRow[],
    selectedCommitId: string | null,
    wip?: WipEdgeRender | null,
  ): void
  /** Dọn dẹp phần tử đã tạo trong host. Gọi một lần lúc unmount. */
  dispose(): void
}

/** Hàm dựng một bộ vẽ, gắn vào `host`. `createCanvasRenderer` cài đúng chữ ký này. */
export type GraphRendererFactory = (host: HTMLElement) => GraphRenderer
