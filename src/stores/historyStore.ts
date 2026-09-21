/**
 * Trạng thái lịch sử commit — PLAT-05, khuôn `byRepo[repoId]` chép từ
 * `repoStore.ts`.
 *
 * `PAGE_SIZE = 1000` đến từ checkpoint #4 của ROADMAP: "Gộp lô lớn — 1000
 * commit một thông điệp, không phải 1." Xuất ra làm hằng số để plan 02-07 đo
 * được với giá trị khác.
 */

import { create } from 'zustand'
import { ipc, type Commit, type GraphRow } from '@/lib/ipc'

export const PAGE_SIZE = 1000

/** Một dải chỉ số hàng đã nạp hoặc đang bay: [start, end). */
type Range = [number, number]

interface HistorySlice {
  commits: Commit[]
  graphRows: GraphRow[]
  total: number
  loadedRanges: Range[]
  isLoading: boolean
  error: string | null
}

interface HistoryState {
  byRepo: Record<string, HistorySlice>
  loadFirstPage: (repoId: string) => Promise<void>
  ensureRange: (repoId: string, start: number, end: number) => Promise<void>
  reset: (repoId: string) => void
}

function emptySlice(): HistorySlice {
  return { commits: [], graphRows: [], total: 0, loadedRanges: [], isLoading: false, error: null }
}

/** Dải `[start, end)` đã được một dải đã nạp/đang bay che phủ hoàn toàn chưa. */
function isCovered(ranges: Range[], start: number, end: number): boolean {
  return ranges.some(([rangeStart, rangeEnd]) => start >= rangeStart && end <= rangeEnd)
}

/**
 * Ghép trang mới vào slice hiện có tại vị trí `skip`. Mảng đích được cấp
 * trước tới `total` phần tử; các vị trí chưa nạp giữ `undefined` (đọc bằng
 * `commits[i]` trả `undefined`, đúng ngữ nghĩa "hàng chưa có dữ liệu" mà
 * `CommitList` cần).
 */
function mergePage(
  slice: HistorySlice,
  skip: number,
  commits: Commit[],
  graphRows: GraphRow[],
  total: number,
): HistorySlice {
  const nextCommits = slice.commits.slice()
  const nextGraphRows = slice.graphRows.slice()
  nextCommits.length = Math.max(nextCommits.length, total)
  nextGraphRows.length = Math.max(nextGraphRows.length, total)

  for (let i = 0; i < commits.length; i += 1) {
    const commit = commits[i]
    const row = graphRows[i]
    if (commit) nextCommits[skip + i] = commit
    if (row) nextGraphRows[skip + i] = row
  }

  return {
    commits: nextCommits,
    graphRows: nextGraphRows,
    total,
    loadedRanges: [...slice.loadedRanges, [skip, skip + commits.length]],
    isLoading: false,
    error: null,
  }
}

export const useHistoryStore = create<HistoryState>((set, get) => ({
  byRepo: {},

  loadFirstPage: async (repoId) => {
    set((s) => ({
      byRepo: { ...s.byRepo, [repoId]: { ...(s.byRepo[repoId] ?? emptySlice()), isLoading: true } },
    }))

    try {
      const result = await ipc.getCommitPage(repoId, 0, PAGE_SIZE)

      // Bất biến của backend: `graphRows` PHẢI cùng độ dài `commits`. Vi phạm
      // là lỗi backend, nhưng giao diện không được sập nếu nó xảy ra — hiện
      // cảnh báo và cắt theo mảng ngắn hơn để không lệch chỉ số.
      if (result.commits.length !== result.graphRows.length) {
        console.warn(
          `[historyStore] commits.length (${result.commits.length}) !== graphRows.length (${result.graphRows.length}) cho repo ${repoId} — cắt theo mảng ngắn hơn`,
        )
      }
      const len = Math.min(result.commits.length, result.graphRows.length)

      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [repoId]: mergePage(
            s.byRepo[repoId] ?? emptySlice(),
            0,
            result.commits.slice(0, len),
            result.graphRows.slice(0, len),
            result.total,
          ),
        },
      }))
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e)
      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [repoId]: { ...(s.byRepo[repoId] ?? emptySlice()), isLoading: false, error: message },
        },
      }))
    }
  },

  ensureRange: async (repoId, start, end) => {
    const slice = get().byRepo[repoId] ?? emptySlice()
    if (isCovered(slice.loadedRanges, start, end)) return

    // Đánh dấu dải này "đang bay" NGAY, trước khi `await`, để lệnh gọi thứ
    // hai xảy ra trong lúc lệnh gọi đầu còn chưa xong không lặp lại IPC —
    // đây là phần "và dải đang bay" của yêu cầu, không chỉ "dải đã nạp".
    const page = Math.floor(start / PAGE_SIZE)
    const skip = page * PAGE_SIZE
    const inFlightEnd = skip + PAGE_SIZE

    set((s) => {
      const current = s.byRepo[repoId] ?? emptySlice()
      return {
        byRepo: {
          ...s.byRepo,
          [repoId]: { ...current, loadedRanges: [...current.loadedRanges, [skip, inFlightEnd]] },
        },
      }
    })

    try {
      const result = await ipc.getCommitPage(repoId, skip, PAGE_SIZE)
      const len = Math.min(result.commits.length, result.graphRows.length)

      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [repoId]: mergePage(
            s.byRepo[repoId] ?? emptySlice(),
            skip,
            result.commits.slice(0, len),
            result.graphRows.slice(0, len),
            result.total,
          ),
        },
      }))
    } catch (e) {
      // Nuốt lỗi — cuộn nhanh qua vùng chưa nạp không được làm sập giao diện.
      // Dải "đang bay" đã được ghi ở trên vẫn giữ nguyên trong `loadedRanges`
      // để tránh bão lặp lại; `error` phản ánh lần thất bại gần nhất.
      const message = e instanceof Error ? e.message : String(e)
      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [repoId]: { ...(s.byRepo[repoId] ?? emptySlice()), error: message },
        },
      }))
    }
  },

  reset: (repoId) =>
    set((s) => {
      const { [repoId]: _removed, ...rest } = s.byRepo
      return { byRepo: rest }
    }),
}))
