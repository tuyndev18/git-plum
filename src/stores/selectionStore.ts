/**
 * Trạng thái commit đang chọn — ARCHITECTURE.md Pattern 3.
 *
 * ⚠️ Store này CHỈ giữ `selectedCommitId`, KHÔNG BAO GIỜ giữ `Commit`, `files`
 * hay `graphRows`. Thêm dữ liệu commit vào đây nghĩa là mỗi lần chọn commit
 * khác sẽ làm MỌI thành phần đọc store render lại — gồm cả `CommitList` — và
 * đó chính là phản mẫu "dựng lại toàn bộ graph khi chọn commit khác" mà
 * nghiên cứu cạnh tranh nêu tên rõ ràng ("Không dựng lại toàn bộ graph khi
 * chọn commit khác. Tách trạng thái chọn ra khỏi trạng thái dữ liệu graph").
 *
 * Vùng chi tiết (`CommitDetail`) tự đọc `selectedCommitId` rồi tự lấy dữ liệu
 * từ `historyStore` (nếu đã nạp) hoặc `ipc.getCommitDetail` (cho danh sách
 * tệp). Nó KHÔNG đọc dữ liệu commit từ store này.
 *
 * Khuôn `selectedByRepo[repoId]` giống `historyStore.ts`/`repoStore.ts`
 * (PLAT-05): chọn ở repo A không ảnh hưởng repo B.
 */

import { create } from 'zustand'

interface SelectionState {
  selectedByRepo: Record<string, string | null>
  select: (repoId: string, commitId: string) => void
  clear: (repoId: string) => void
}

export const useSelectionStore = create<SelectionState>((set) => ({
  selectedByRepo: {},

  select: (repoId, commitId) =>
    set((s) => ({ selectedByRepo: { ...s.selectedByRepo, [repoId]: commitId } })),

  clear: (repoId) => set((s) => ({ selectedByRepo: { ...s.selectedByRepo, [repoId]: null } })),
}))
