/**
 * Test `RefBadges` — HIST-06, nhãn nhánh/tag trên hàng commit.
 *
 * Props `refs: GitRef[]` truyền vào từ ngoài (không tự gọi store) — hàng
 * render tới hàng nghìn lần khi cuộn, mỗi component tự subscribe store là
 * chi phí vô ích.
 */

import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'

import { RefBadges } from '@/components/history/RefBadges'
import type { GitRef } from '@/lib/ipc'

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

describe('local vs remote', () => {
  it('refs/heads/main -> nhãn main với class nhánh local', () => {
    const { container } = render(
      <RefBadges refs={[ref({ fullName: 'refs/heads/main', kind: 'localBranch' })]} />,
    )

    const badge = container.querySelector('.ref-badge--local')
    expect(badge).not.toBeNull()
    expect(badge?.textContent).toContain('main')
  })

  it('refs/remotes/origin/main -> nhãn origin/main với class nhánh remote, KHÁC class local', () => {
    const { container } = render(
      <RefBadges
        refs={[
          ref({
            fullName: 'refs/remotes/origin/main',
            shortName: 'origin/main',
            kind: 'remoteBranch',
          }),
        ]}
      />,
    )

    const remoteBadge = container.querySelector('.ref-badge--remote')
    expect(remoteBadge).not.toBeNull()
    expect(remoteBadge?.textContent).toContain('origin/main')
    expect(container.querySelector('.ref-badge--local')).toBeNull()
  })

  it('tag -> nhãn kiểu tag, khác cả hai loại trên', () => {
    const { container } = render(
      <RefBadges refs={[ref({ fullName: 'refs/tags/v1', shortName: 'v1', kind: 'tag' })]} />,
    )

    expect(container.querySelector('.ref-badge--tag')).not.toBeNull()
    expect(container.querySelector('.ref-badge--local')).toBeNull()
    expect(container.querySelector('.ref-badge--remote')).toBeNull()
  })
})

describe('HEAD', () => {
  it('isHead === true -> có thêm dấu hiệu HEAD', () => {
    render(
      <RefBadges
        refs={[ref({ fullName: 'refs/heads/main', kind: 'localBranch', isHead: true })]}
      />,
    )

    expect(screen.getByText(/HEAD/)).toBeTruthy()
  })
})

describe('rỗng', () => {
  it('commit không ref nào -> không render gì, không chiếm chiều cao', () => {
    const { container } = render(<RefBadges refs={[]} />)

    expect(container.firstChild).toBeNull()
  })
})

describe('nhiều nhãn', () => {
  it('12 ref -> hiện một số nhãn rồi chỉ báo +N, không tràn', () => {
    const refs = Array.from({ length: 12 }, (_, i) =>
      ref({ fullName: `refs/heads/b${i}`, shortName: `b${i}`, kind: 'localBranch' }),
    )
    render(<RefBadges refs={refs} />)

    expect(screen.getByText(/\+\d+/)).toBeTruthy()
  })
})
