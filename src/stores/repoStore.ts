/**
 * Trạng thái repository — PLAT-05.
 *
 * Dữ liệu lưu theo `byRepo[repoId]` ngay từ đầu, dù v1 chỉ mở một repository tại
 * một thời điểm. Đây là quyết định frontend đắt nhất nếu sửa sau: chuyển từ một
 * biến phẳng sang map nghĩa là đụng vào mọi component đọc trạng thái.
 */

import { create } from 'zustand'
import { ipc, type RepoInfo } from '@/lib/ipc'
import { loadRecentRepos, rememberRepo, type RecentRepo } from '@/lib/recentRepos'

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
  /**
   * Danh sách repository gần đây (PLAT-07), mới nhất trước.
   *
   * Nằm trong store thay vì để component tự gọi `loadRecentRepos()`: như vậy
   * giao diện chỉ đọc một mảng đồng bộ, không phải tự quản lý vòng đời bất đồng
   * bộ, và mỗi lần mở repository thành công là một lần danh sách tự cập nhật.
   */
  recent: RecentRepo[]

  openRepository: (path: string) => Promise<void>
  closeRepository: (id: string) => Promise<void>
  setActiveRepo: (id: string | null) => void
  refreshBranch: (id: string) => Promise<void>
  setError: (id: string, error: string | null) => void
  /** Nạp danh sách gần đây từ đĩa. Gọi một lần lúc ứng dụng khởi động. */
  loadRecent: () => Promise<void>
  setRecent: (recent: RecentRepo[]) => void
}

export const useRepoStore = create<RepoState>((set, get) => ({
  byRepo: {},
  activeRepoId: null,
  isOpening: false,
  recent: [],

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

      // Ghi nhớ repository cho danh sách gần đây (PLAT-07).
      //
      // Thứ tự ưu tiên ở đây là tuyệt đối: người dùng đã mở được repository
      // rồi. Mất một mục trong danh sách gần đây là chuyện nhỏ và tự khỏi ở lần
      // mở sau; ném lỗi lên banner sau một thao tác đã thành công mới là chuyện
      // to. Vì thế lời gọi này được bọc riêng và lỗi của nó bị nuốt tại đây,
      // ngoài việc bản thân `rememberRepo` cũng đã tự bọc `try/catch` — hai lớp
      // là có chủ ý, vì lớp trong nằm ở tệp khác và có thể bị sửa mất.
      try {
        const recent = await rememberRepo({ path: info.path, name: info.name })
        // Danh sách rỗng nghĩa là ghi thất bại; đừng xoá sạch cái đang hiển thị.
        if (recent.length > 0) set({ recent })
      } catch (e) {
        console.warn('[repoStore] không ghi nhớ được repository gần đây:', e)
      }
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

  loadRecent: async () => {
    // `loadRecentRepos` đã tự nuốt lỗi và trả mảng rỗng, nên không cần bọc thêm.
    set({ recent: await loadRecentRepos() })
  },

  setRecent: (recent) => set({ recent }),
}))

/** Repository đang hiển thị, hoặc `null`. */
export function useActiveRepo(): RepoSlice | null {
  return useRepoStore((s) => (s.activeRepoId ? s.byRepo[s.activeRepoId] ?? null : null))
}
