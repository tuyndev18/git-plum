/**
 * Interface của **bộ dựng trình xem diff** — cùng khuôn `interface GraphRenderer`
 * của Phase 2 (`src/lib/graph-render/types.ts`).
 *
 * # 🔴 Vì sao interface này là BẮT BUỘC, không phải "nên có"
 *
 * Checkpoint #3 của plan 03-01 — phép đo chọn giữa đường A (`@codemirror/merge`
 * tự tính diff từ hai tài liệu đầy đủ) và đường B (dựng decoration từ đầu ra
 * `git diff`) — **bị bỏ qua** theo quyết định chủ dự án ngày 2026-09-22
 * (`docs/09-phase3-diff-decision.md` mục 7, commit `6910732`).
 *
 * Đường A được chọn **vì nó là mặc định khi thiếu bằng chứng**, không vì đã
 * chứng minh đủ nhanh. Rủi ro cụ thể đang nhận, ghi nguyên văn từ mục 8 của tài
 * liệu đó: với `yarn.lock` 632 KB — tệp thật trong repo của chủ dự án —
 * `@codemirror/merge` sẽ diff lại hai bản ~630 KB **mỗi lần mở**, và không ai
 * biết việc đó mất bao lâu. Ba khả năng không phân biệt được mà không đo, và
 * khả năng thứ ba là "phải viết lại toàn bộ trình xem theo đường B".
 *
 * Nên tài liệu đó tự nêu điều kiện: *"03-04 phải giữ trình xem sau một interface
 * để việc đổi đường là sửa một tệp, không phải sửa cả giao diện."*
 *
 * Tiền lệ là `GraphRenderer`: đổi canvas ↔ SVG là viết một tệp mới rồi đổi **một**
 * dòng import. Làm y vậy ở đây.
 *
 * # Cổng kiểm việc này
 *
 * Không lời gọi API nào của `@codemirror/merge` được rò ra ngoài tệp cài đặt —
 * tương tự cổng `grep getContext GraphCanvas.tsx` → 0 của Phase 2. Có test đọc
 * mã nguồn trong `DiffViewer.test.tsx`/`diffRenderer.test.ts` khẳng định điều đó.
 */

import type { Extension } from '@codemirror/state'

import type { DiffDoc } from './decorations'
import type { DiffViewMode } from '@/stores/diffStore'

/** Đối số dựng một trình xem. Thuần dữ liệu — không phần tử DOM nào của CodeMirror. */
export interface DiffRendererOptions {
  /** Phần tử cha. Bộ dựng tự quản lý mọi thứ bên trong nó. */
  parent: HTMLElement
  /**
   * Hợp nhất hay hai cột.
   *
   * Là **đối số dựng**, không phải một `setMode()`: hai chế độ là hai cây DOM
   * khác hẳn, nên đổi chế độ *phải* dựng lại. Phân biệt rõ với cờ khoảng trắng
   * và ngôn ngữ, vốn là extension và đổi được tại chỗ qua `Compartment` —
   * `DiffViewer.test.tsx` có test cho cả hai hành vi.
   */
  mode: DiffViewMode
  /** Tài liệu phía **mới** (dùng cho cả hợp nhất). */
  newDoc: DiffDoc
  /** Tài liệu phía **cũ**. Chỉ chế độ hai cột dùng. */
  oldDoc: DiffDoc
  /** Extension ngôn ngữ đã nạp lười, hay `null` khi không nhận ra phần mở rộng. */
  language: Extension | null
  /** Hiện ký tự khoảng trắng (DIFF-04). */
  showWhitespace: boolean
}

/**
 * Một trình xem diff đang sống.
 *
 * Mọi thao tác mà giao diện cần đều đi qua đây; `DiffViewer.tsx` **không** giữ
 * một `EditorView` hay `MergeView` nào. Đó là điều làm việc đổi A ↔ B thành
 * việc sửa một tệp.
 */
export interface DiffRenderer {
  /**
   * Đổi cờ khoảng trắng **không dựng lại** trình xem.
   *
   * Bắt buộc là một phương thức riêng chứ không phải "dựng lại với option mới":
   * dựng lại làm **mất vị trí cuộn**, và người dùng bật cờ này giữa lúc đang đọc
   * một chỗ cụ thể. Cài đặt dùng `Compartment` của CodeMirror.
   */
  setShowWhitespace: (show: boolean) => void

  /** Đổi extension ngôn ngữ **không dựng lại** trình xem (cũng qua `Compartment`). */
  setLanguage: (language: Extension | null) => void

  /** Cuộn tới một dòng (đếm từ 1) của tài liệu phía mới, và đặt con trỏ ở đó. */
  scrollToLine: (line: number) => void

  /** Dòng (đếm từ 1) đang ở đầu vùng nhìn — điểm neo cho việc nhảy khối. */
  currentLine: () => number

  /**
   * Giải phóng mọi tài nguyên.
   *
   * Thiếu nó thì mỗi lần chọn tệp rò một `EditorView`, và người dùng bấm qua
   * hàng chục tệp trong một phiên (T-03-28).
   */
  destroy: () => void
}

/** Hàm dựng. Đổi đường A ↔ B là đổi **một** dòng import trỏ tới hàm này. */
export type CreateDiffRenderer = (options: DiffRendererOptions) => DiffRenderer
