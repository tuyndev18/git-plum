/**
 * Test `selectionStore` — ARCHITECTURE.md Pattern 3: store này CHỈ giữ
 * `selectedCommitId`, không bao giờ giữ dữ liệu commit. Test khoá của
 * `getState()` là cách duy nhất giữ được ràng buộc này qua thời gian.
 */

import { beforeEach, describe, expect, it } from 'vitest'

import { useSelectionStore } from '@/stores/selectionStore'

beforeEach(() => {
  useSelectionStore.setState({ selectedByRepo: {} })
})

describe('select', () => {
  it('select(repoId, commitId) -> selectedByRepo[repoId] === commitId', () => {
    useSelectionStore.getState().select('repo-1', 'commit-a')

    expect(useSelectionStore.getState().selectedByRepo['repo-1']).toBe('commit-a')
  })

  it('chọn ở repo A không ảnh hưởng repo B (PLAT-05)', () => {
    useSelectionStore.getState().select('repo-a', 'commit-1')
    useSelectionStore.getState().select('repo-b', 'commit-2')

    expect(useSelectionStore.getState().selectedByRepo['repo-a']).toBe('commit-1')
    expect(useSelectionStore.getState().selectedByRepo['repo-b']).toBe('commit-2')
  })
})

describe('clear', () => {
  it('clear(repoId) -> null', () => {
    useSelectionStore.getState().select('repo-1', 'commit-a')
    useSelectionStore.getState().clear('repo-1')

    expect(useSelectionStore.getState().selectedByRepo['repo-1']).toBeNull()
  })
})

describe('Pattern 3 — không được chứa dữ liệu commit', () => {
  it('getState() không có khoá nào ngoài selectedByRepo và các hành động', () => {
    useSelectionStore.getState().select('repo-1', 'commit-a')

    const keys = Object.keys(useSelectionStore.getState())
    const dataLikeKeys = keys.filter(
      (k) => !['selectedByRepo', 'select', 'clear'].includes(k),
    )

    expect(dataLikeKeys).toEqual([])
  })

  it('selectedByRepo chỉ chứa string | null, không phải object (Commit)', () => {
    useSelectionStore.getState().select('repo-1', 'commit-a')

    const value = useSelectionStore.getState().selectedByRepo['repo-1']
    expect(typeof value === 'string' || value === null).toBe(true)
  })
})
