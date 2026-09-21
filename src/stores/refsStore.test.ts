/**
 * Test `refsStore` — khuôn `byRepo[repoId]` giống `historyStore.ts` (PLAT-05).
 *
 * Chỉ mục `commitId -> GitRef[]` phải tính MỘT LẦN khi `refs` đổi, không quét
 * tuyến tính mỗi hàng — với 100k hàng × hàng nghìn ref đó là O(n·m) trong lúc
 * cuộn.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'

import { ipc, type GitRef } from '@/lib/ipc'
import { useRefsStore } from '@/stores/refsStore'

vi.mock('@/lib/ipc', () => ({
  ipc: {
    listRefs: vi.fn(),
  },
}))

const listRefs = vi.mocked(ipc.listRefs)

function ref(overrides: Partial<GitRef> & { fullName: string; target: string }): GitRef {
  return {
    shortName: overrides.fullName.split('/').pop() ?? overrides.fullName,
    kind: 'localBranch',
    upstream: null,
    ahead: 0,
    behind: 0,
    isHead: false,
    ...overrides,
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  useRefsStore.setState({ byRepo: {} })
})

describe('load', () => {
  it('load(repoId) -> byRepo[repoId].refs có dữ liệu', async () => {
    listRefs.mockResolvedValueOnce([
      ref({ fullName: 'refs/heads/main', target: 'c1', kind: 'localBranch' }),
    ])

    await useRefsStore.getState().load('repo-1')

    expect(useRefsStore.getState().byRepo['repo-1']?.refs).toHaveLength(1)
  })

  it('counts trả đúng số local/remote/tag', async () => {
    listRefs.mockResolvedValueOnce([
      ref({ fullName: 'refs/heads/main', target: 'c1', kind: 'localBranch' }),
      ref({ fullName: 'refs/heads/dev', target: 'c2', kind: 'localBranch' }),
      ref({ fullName: 'refs/remotes/origin/main', target: 'c1', kind: 'remoteBranch' }),
      ref({ fullName: 'refs/tags/v1', target: 'c1', kind: 'tag' }),
    ])

    await useRefsStore.getState().load('repo-1')

    expect(useRefsStore.getState().counts('repo-1')).toEqual({
      localBranches: 2,
      remoteBranches: 1,
      tags: 1,
    })
  })

  it('load thất bại -> error được đặt, không ném ra ngoài', async () => {
    listRefs.mockRejectedValueOnce(new Error('boom'))

    await expect(useRefsStore.getState().load('repo-1')).resolves.not.toThrow()

    expect(useRefsStore.getState().byRepo['repo-1']?.error).toBeTruthy()
  })
})

describe('refsForCommit', () => {
  it('chỉ trả ref có target === commitId', async () => {
    listRefs.mockResolvedValueOnce([
      ref({ fullName: 'refs/heads/main', target: 'c1' }),
      ref({ fullName: 'refs/heads/dev', target: 'c2' }),
    ])
    await useRefsStore.getState().load('repo-1')

    const result = useRefsStore.getState().refsForCommit('repo-1', 'c1')

    expect(result).toHaveLength(1)
    expect(result[0]?.fullName).toBe('refs/heads/main')
  })

  it('hai ref cùng trỏ một commit (nhánh + tag) -> trả cả hai', async () => {
    listRefs.mockResolvedValueOnce([
      ref({ fullName: 'refs/heads/main', target: 'c1', kind: 'localBranch' }),
      ref({ fullName: 'refs/tags/v1', target: 'c1', kind: 'tag' }),
    ])
    await useRefsStore.getState().load('repo-1')

    const result = useRefsStore.getState().refsForCommit('repo-1', 'c1')

    expect(result).toHaveLength(2)
  })

  it('commit không có ref nào -> mảng rỗng, không undefined', async () => {
    listRefs.mockResolvedValueOnce([ref({ fullName: 'refs/heads/main', target: 'c1' })])
    await useRefsStore.getState().load('repo-1')

    const result = useRefsStore.getState().refsForCommit('repo-1', 'commit-khong-ton-tai')

    expect(result).toEqual([])
  })

  it('không quét tuyến tính mỗi lần gọi — chỉ mục tính sẵn khi refs đổi', async () => {
    listRefs.mockResolvedValueOnce([
      ref({ fullName: 'refs/heads/main', target: 'c1' }),
      ref({ fullName: 'refs/heads/dev', target: 'c2' }),
    ])
    await useRefsStore.getState().load('repo-1')

    // Chỉ mục phải tồn tại sẵn trong slice (không phải hàm quét mảng refs mỗi lần gọi).
    const slice = useRefsStore.getState().byRepo['repo-1']
    expect(slice?.byCommit).toBeDefined()
    expect(slice?.byCommit.get('c1')).toHaveLength(1)
  })
})
