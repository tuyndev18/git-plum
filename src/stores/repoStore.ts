/**
 * Trạng thái repository — PLAT-05.
 *
 * Dữ liệu lưu theo `byRepo[repoId]` ngay từ đầu, dù v1 chỉ mở một repository tại
 * một thời điểm. Đây là quyết định frontend đắt nhất nếu sửa sau: chuyển từ một
 * biến phẳng sang map nghĩa là đụng vào mọi component đọc trạng thái.
 */

import { create } from 'zustand'
import { ipc, type RepoInfo } from '@/lib/ipc'

/** Dữ liệu gắn với một repository cụ thể. */
interface RepoSlice {
  info: RepoInfo
  currentBranch: string | null
  /** Lỗi gần nhất khi thao tác trên repository này. */
  error: string | null
}

interface RepoState {
  /** Mọi repository đang mở, khoá theo id. */
  byRepo: Record<string, RepoSlice>
  /** Repository đang hiển thị. `null` khi chưa mở cái nào. */
  activeRepoId: string | null
  /** Đang có thao tác mở repository chạy dở. */
  isOpening: boolean

  openRepository: (path: string) => Promise<void>
  closeRepository: (id: string) => Promise<void>
  setActiveRepo: (id: string | null) => void
  refreshBranch: (id: string) => Promise<void>
  setError: (id: string, error: string | null) => void
}

export const useRepoStore = create<RepoState>((set, get) => ({
  byRepo: {},
  activeRepoId: null,
  isOpening: false,

  openRepository: async (path) => {
    set({ isOpening: true })
    try {
      const info = await ipc.openRepository(path)
      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [info.id]: { info, currentBranch: null, error: null },
        },
        activeRepoId: info.id,
        isOpening: false,
      }))
      await get().refreshBranch(info.id)
    } catch (e) {
      set({ isOpening: false })
      throw e
    }
  },

  closeRepository: async (id) => {
    await ipc.closeRepository(id)
    set((s) => {
      const { [id]: _removed, ...rest } = s.byRepo
      return {
        byRepo: rest,
        activeRepoId: s.activeRepoId === id ? null : s.activeRepoId,
      }
    })
  },

  setActiveRepo: (id) => set({ activeRepoId: id }),

  refreshBranch: async (id) => {
    try {
      const branch = await ipc.currentBranch()
      set((s) => {
        const slice = s.byRepo[id]
        if (!slice) return s
        return {
          byRepo: { ...s.byRepo, [id]: { ...slice, currentBranch: branch, error: null } },
        }
      })
    } catch {
      // Repository chưa có commit nào thì `rev-parse HEAD` thất bại — đó là
      // trạng thái bình thường, không phải lỗi cần báo động.
      set((s) => {
        const slice = s.byRepo[id]
        if (!slice) return s
        return { byRepo: { ...s.byRepo, [id]: { ...slice, currentBranch: null } } }
      })
    }
  },

  setError: (id, error) =>
    set((s) => {
      const slice = s.byRepo[id]
      if (!slice) return s
      return { byRepo: { ...s.byRepo, [id]: { ...slice, error } } }
    }),
}))

/** Repository đang hiển thị, hoặc `null`. */
export function useActiveRepo(): RepoSlice | null {
  return useRepoStore((s) => (s.activeRepoId ? s.byRepo[s.activeRepoId] ?? null : null))
}
