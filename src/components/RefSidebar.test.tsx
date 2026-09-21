/**
 * Test `RefSidebar` — HIST-07, thanh bên nhánh/tag kèm số đếm.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

import { RefSidebar } from '@/components/RefSidebar'
import { ipc, type GitRef } from '@/lib/ipc'
import { useRefsStore } from '@/stores/refsStore'

vi.mock('@/lib/ipc', () => ({
  ipc: {
    listRefs: vi.fn(),
  },
}))

const listRefs = vi.mocked(ipc.listRefs)

function ref(overrides: Partial<GitRef> & { fullName: string }): GitRef {
  return {
    shortName: overrides.fullName.split('/').pop() ?? overrides.fullName,
    kind: 'localBranch',
    target: 'c1',
    upstream: null,
    ahead: 0,
    behind: 0,
    isHead: false,
    ...overrides,
  }
}

const REPO_ID = 'repo-1'

beforeEach(() => {
  vi.clearAllMocks()
  useRefsStore.setState({ byRepo: {} })
})

async function loadAndRender(refs: GitRef[], onSelectCommit = vi.fn()) {
  listRefs.mockResolvedValueOnce(refs)
  await useRefsStore.getState().load(REPO_ID)
  return render(<RefSidebar repoId={REPO_ID} onSelectCommit={onSelectCommit} />)
}

describe('ba nhóm có số đếm', () => {
  it('Nhánh local, Nhánh remote, Tag — mỗi nhóm có số đếm trong tiêu đề', async () => {
    await loadAndRender([
      ref({ fullName: 'refs/heads/main', kind: 'localBranch' }),
      ref({ fullName: 'refs/heads/dev', kind: 'localBranch' }),
      ref({ fullName: 'refs/remotes/origin/main', kind: 'remoteBranch' }),
      ref({ fullName: 'refs/tags/v1', kind: 'tag' }),
    ])

    expect(screen.getByText(/nhánh local.*2|2.*nhánh local/i)).toBeTruthy()
    expect(screen.getByText(/nhánh remote.*1|1.*nhánh remote/i)).toBeTruthy()
    expect(screen.getByText(/tag.*1|1.*tag/i)).toBeTruthy()
  })

  it('nhóm rỗng vẫn hiện tiêu đề với số 0', async () => {
    await loadAndRender([ref({ fullName: 'refs/heads/main', kind: 'localBranch' })])

    expect(screen.getByText(/nhánh remote.*0|0.*nhánh remote/i)).toBeTruthy()
    expect(screen.getByText(/tag.*0|0.*tag/i)).toBeTruthy()
  })
})

describe('ahead/behind', () => {
  it('nhánh có ahead/behind khác 0 -> hiện hai số đó', async () => {
    await loadAndRender([
      ref({ fullName: 'refs/heads/main', kind: 'localBranch', ahead: 3, behind: 2 }),
    ])

    expect(screen.getByText(/↑3|3.*↑|ahead.*3/i)).toBeTruthy()
    expect(screen.getByText(/↓2|2.*↓|behind.*2/i)).toBeTruthy()
  })
})

describe('HEAD', () => {
  it('nhánh là HEAD -> có dấu hiệu riêng', async () => {
    await loadAndRender([
      ref({ fullName: 'refs/heads/main', kind: 'localBranch', isHead: true }),
    ])

    expect(screen.getByText(/HEAD/)).toBeTruthy()
  })

  it('HEAD tách rời (không ref nào isHead) -> hiện dòng nói rõ', async () => {
    await loadAndRender([ref({ fullName: 'refs/heads/main', kind: 'localBranch', isHead: false })])

    expect(screen.getByText(/tách rời/i)).toBeTruthy()
  })
})

describe('bấm nhánh', () => {
  it('bấm một nhánh -> gọi onSelectCommit với ref.target', async () => {
    const onSelectCommit = vi.fn()
    await loadAndRender(
      [ref({ fullName: 'refs/heads/main', kind: 'localBranch', target: 'commit-abc' })],
      onSelectCommit,
    )

    screen.getByText('main').click()

    expect(onSelectCommit).toHaveBeenCalledWith('commit-abc')
  })
})
