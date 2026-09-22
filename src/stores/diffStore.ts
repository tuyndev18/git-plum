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

  /**
   * Panel lịch sử tệp có đang mở không (DIFF-05).
   *
   * **Không** theo repo, cùng lý do với `viewMode`: đây là lựa chọn "đang xem theo
   * cách nào". Nhưng nó **bị đóng** khi đổi tệp hoặc đổi commit, vì lúc đó danh sách
   * phiên bản đang hiện thuộc về một tệp khác — xem `selectFile`/`commitChanged`.
   */
  historyOpen: boolean
  /**
   * Commit mà `DiffViewer` phải vẽ, **ghi đè** `selectionStore.selectedCommitId`.
   *
   * # 🔴 Vì sao là OVERRIDE, không phải ghi vào `selectionStore`
   *
   * Người dùng đang đọc lịch sử của một tệp và bấm qua từng phiên bản để xem nó đổi
   * thế nào. Nếu mỗi lần bấm cũng đổi commit đang chọn trên **đồ thị**, họ **mất chỗ
   * của mình** trong lịch sử: hàng đang sáng nhảy đi, và khi đóng panel lịch sử tệp
   * thì không còn đường về chỗ vừa đứng. Hai việc — "đọc lịch sử một tệp" và "đang
   * đứng ở commit nào trên đồ thị" — là hai việc độc lập, và giữ chúng độc lập là cả
   * điểm của trường này.
   *
   * `null` = không ghi đè; `DiffViewer` dùng commit đang chọn trên đồ thị.
   */
  historyCommitOverride: string | null

  selectFile: (repoId: string, path: string) => void
  clearFile: (repoId: string) => void
  /** Gọi khi commit đang chọn đổi: tệp cũ có thể không có trong commit mới. */
  commitChanged: (repoId: string) => void

  setViewMode: (mode: DiffViewMode) => void
  toggleViewMode: () => void
  toggleWhitespace: () => void

  openHistory: () => void
  closeHistory: () => void
  /** Bấm một phiên bản: đặt commit ghi đè, **không** chạm `selectionStore`. */
  selectHistoryVersion: (commitId: string) => void
}

export const useDiffStore = create<DiffState>((set) => ({
  selectedFileByRepo: {},
  viewMode: 'unified',
  showWhitespace: false,
  historyOpen: false,
  historyCommitOverride: null,

  /*
   * Đổi tệp → **đóng** panel lịch sử và xoá ghi đè.
   *
   * Danh sách phiên bản đang hiện thuộc về tệp cũ. Giữ nó mở nghĩa là người dùng
   * chọn `b.ts` rồi thấy lịch sử của `a.ts` — và `FileHistory` sẽ nạp lại, nên trong
   * khoảng giữa hai lần nạp có một danh sách sai trên màn hình. Đóng nó ở đây là
   * lớp phòng thủ **thứ nhất**; cổng chống đua trong `FileHistory` là lớp thứ hai,
   * cho ca người dùng tự mở lại ngay.
   */
  selectFile: (repoId, path) =>
    set((s) => ({
      selectedFileByRepo: { ...s.selectedFileByRepo, [repoId]: path },
      historyOpen: false,
      historyCommitOverride: null,
    })),

  clearFile: (repoId) =>
    set((s) => ({
      selectedFileByRepo: { ...s.selectedFileByRepo, [repoId]: null },
      historyOpen: false,
      historyCommitOverride: null,
    })),

  commitChanged: (repoId) =>
    set((s) => ({
      selectedFileByRepo: { ...s.selectedFileByRepo, [repoId]: null },
      historyOpen: false,
      historyCommitOverride: null,
    })),

  setViewMode: (mode) => set({ viewMode: mode }),

  toggleViewMode: () =>
    set((s) => ({ viewMode: s.viewMode === 'unified' ? 'split' : 'unified' })),

  toggleWhitespace: () => set((s) => ({ showWhitespace: !s.showWhitespace })),

  openHistory: () => set({ historyOpen: true, historyCommitOverride: null }),

  /*
   * Đóng panel cũng **xoá ghi đè**, nên `DiffViewer` trở về commit đang chọn trên đồ
   * thị. Giữ lại ghi đè sau khi đóng làm người dùng xem diff của một commit mà không
   * có gì trên màn hình nói đó là commit nào.
   */
  closeHistory: () => set({ historyOpen: false, historyCommitOverride: null }),

  selectHistoryVersion: (commitId) => set({ historyCommitOverride: commitId }),
}))
