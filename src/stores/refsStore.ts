/**
 * Trạng thái nhánh/tag — HIST-06, HIST-07. Khuôn `byRepo[repoId]` giống
 * `historyStore.ts` (PLAT-05).
 *
 * `byCommit` là chỉ mục `Map<commitId, GitRef[]>` tính MỘT LẦN mỗi khi `refs`
 * đổi (trong `load`), không quét tuyến tính mỗi hàng — với 100k hàng ×
 * hàng nghìn ref, quét mỗi hàng là O(n·m) trong lúc cuộn.
 */

import { create } from 'zustand'
import { ipc, type GitRef } from '@/lib/ipc'

interface RefsCounts {
  localBranches: number
  remoteBranches: number
  tags: number
}

interface RefsSlice {
  refs: GitRef[]
  /** Chỉ mục tính sẵn: `GitRef.target` (mã commit) -> mọi ref trỏ tới nó. */
  byCommit: Map<string, GitRef[]>
  isLoading: boolean
  error: string | null
}

interface RefsState {
  byRepo: Record<string, RefsSlice>
  load: (repoId: string) => Promise<void>
  refsForCommit: (repoId: string, commitId: string) => GitRef[]
  counts: (repoId: string) => RefsCounts
}

function emptySlice(): RefsSlice {
  return { refs: [], byCommit: new Map(), isLoading: false, error: null }
}

function buildIndex(refs: GitRef[]): Map<string, GitRef[]> {
  const index = new Map<string, GitRef[]>()
  for (const ref of refs) {
    const list = index.get(ref.target)
    if (list) list.push(ref)
    else index.set(ref.target, [ref])
  }
  return index
}

export const useRefsStore = create<RefsState>((set, get) => ({
  byRepo: {},

  load: async (repoId) => {
    set((s) => ({
      byRepo: { ...s.byRepo, [repoId]: { ...(s.byRepo[repoId] ?? emptySlice()), isLoading: true } },
    }))

    try {
      const refs = await ipc.listRefs(repoId)
      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [repoId]: { refs, byCommit: buildIndex(refs), isLoading: false, error: null },
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

  refsForCommit: (repoId, commitId) => {
    const slice = get().byRepo[repoId]
    if (!slice) return []
    return slice.byCommit.get(commitId) ?? []
  },

  counts: (repoId) => {
    const slice = get().byRepo[repoId]
    const refs = slice?.refs ?? []
    let localBranches = 0
    let remoteBranches = 0
    let tags = 0
    for (const ref of refs) {
      if (ref.kind === 'localBranch') localBranches += 1
      else if (ref.kind === 'remoteBranch') remoteBranches += 1
      else if (ref.kind === 'tag') tags += 1
    }
    return { localBranches, remoteBranches, tags }
  },
}))
