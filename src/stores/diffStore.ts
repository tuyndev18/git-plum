/**
 * Trạng thái của trình xem diff — hai nhóm **khác nhau về bản chất**.
 *
 * # Vì sao `selectedFile` theo repo mà `viewMode` thì không
 *
 * Đây không phải sở thích; hai thứ này trả lời hai câu hỏi khác nhau, và đặt sai
 * nhóm là một lỗi UX im lặng (không lỗi biên dịch, không test hành vi nào đỏ nếu
 * cổng viết sai).
 *
 * **`selectedFileByRepo[repoId]` — theo repo, khuôn `selectionStore.ts`
 * (PLAT-05).** "Đang xem tệp nào" là *vị trí* trong một repo cụ thể. Mở repo thứ
 * hai rồi quay lại repo đầu, người dùng mong thấy lại chỗ mình đang đứng; và
 * `src/lib/ipc.ts` chắc chắn không có tệp nào tên `src-tauri/src/main.rs`.
 *
 * **`viewMode` và `showWhitespace` — KHÔNG theo repo, khuôn `uiStore.ts`.** Đây
 * là *lựa chọn của người dùng về cách xem*, đúng loại với `uiStore.fileListView`:
 * "người dùng đã chọn cách xem, không phải chọn lại mỗi lần" (`<behavior>` Task 2,
 * plan 02-06 → HIST-09). Một người thích hai cột thì thích hai cột ở mọi repo.
 * Nhóm chúng theo repo làm lựa chọn tự đặt lại mỗi lần đổi repo — đúng phàn nàn
 * mà HIST-09 tồn tại để chặn.
 *
 * # Store chỉ giữ ĐƯỜNG DẪN, không giữ `FileDiff`
 *
 * Cùng lý do `selectionStore` chỉ giữ `selectedCommitId` (ARCHITECTURE.md
 * Pattern 3): nhét dữ liệu diff vào đây làm **mọi** thành phần đọc store render
 * lại mỗi lần chọn tệp — gồm cả `FileList` và, qua đó, `CommitDetail`. Dữ liệu
 * diff sống trong `DiffViewer` (và trong cache phía Rust, vốn trả lời lần gọi
 * thứ hai mà không sinh tiến trình git nào).
 *
 * # Đổi commit → `selectedFile` về `null`
 *
 * Tệp đang chọn có thể **không tồn tại** trong commit mới. Giữ lại nghĩa là gọi
 * `getFileDiff` với một path không có trong commit đó, và người dùng nhận một
 * thông báo lỗi cho thao tác hoàn toàn bình thường "chọn commit khác".
 */

import { create } from 'zustand'

/** Hợp nhất (một cột) hay hai cột cạnh nhau — DIFF-02. */
export type DiffViewMode = 'unified' | 'split'

interface DiffState {
  /** Đường dẫn tệp đang chọn, theo repo. `null` = đã bỏ chọn; `undefined` = chưa chọn gì. */
  selectedFileByRepo: Record<string, string | null>
  /** Chế độ hiển thị. **Không** theo repo — xem doc comment đầu tệp. */
  viewMode: DiffViewMode
  /** Hiện ký tự khoảng trắng (DIFF-04). **Không** theo repo. */
  showWhitespace: boolean

  selectFile: (repoId: string, path: string) => void
  clearFile: (repoId: string) => void
  /** Gọi khi commit đang chọn đổi: tệp cũ có thể không có trong commit mới. */
  commitChanged: (repoId: string) => void

  setViewMode: (mode: DiffViewMode) => void
  toggleViewMode: () => void
  toggleWhitespace: () => void
}

export const useDiffStore = create<DiffState>((set) => ({
  selectedFileByRepo: {},
  viewMode: 'unified',
  showWhitespace: false,

  selectFile: (repoId, path) =>
    set((s) => ({ selectedFileByRepo: { ...s.selectedFileByRepo, [repoId]: path } })),

  clearFile: (repoId) =>
    set((s) => ({ selectedFileByRepo: { ...s.selectedFileByRepo, [repoId]: null } })),

  commitChanged: (repoId) =>
    set((s) => ({ selectedFileByRepo: { ...s.selectedFileByRepo, [repoId]: null } })),

  setViewMode: (mode) => set({ viewMode: mode }),

  toggleViewMode: () =>
    set((s) => ({ viewMode: s.viewMode === 'unified' ? 'split' : 'unified' })),

  toggleWhitespace: () => set((s) => ({ showWhitespace: !s.showWhitespace })),
}))
