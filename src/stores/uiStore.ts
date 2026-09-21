/**
 * Trạng thái giao diện nhỏ, không gắn với một repo cụ thể — khác
 * `selectionStore`/`historyStore`/`refsStore` (khuôn `byRepo[repoId]`).
 *
 * `fileListView` là lựa chọn hiển thị của NGƯỜI DÙNG (phẳng hay cây), không
 * phải dữ liệu theo repo — giữ nó sống qua việc đổi commit hay đổi repo là
 * đúng ý định UX ("người dùng đã chọn cách xem, không phải chọn lại mỗi
 * lần" — `<behavior>` của Task 2, plan 02-06).
 */

import { create } from 'zustand'

export type FileListView = 'flat' | 'tree'

interface UiState {
  fileListView: FileListView
  toggleFileListView: () => void
}

export const useUiStore = create<UiState>((set) => ({
  fileListView: 'flat',

  toggleFileListView: () =>
    set((s) => ({ fileListView: s.fileListView === 'flat' ? 'tree' : 'flat' })),
}))
