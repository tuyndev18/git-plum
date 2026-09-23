/**
 * Test `RefBadges` — HIST-06, nhãn nhánh/tag trên hàng commit.
 *
 * Props `refs: GitRef[]` truyền vào từ ngoài (không tự gọi store) — hàng
 * render tới hàng nghìn lần khi cuộn, mỗi component tự subscribe store là
 * chi phí vô ích.
 */

import { describe, expect, it } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'

import { groupRefs, RefBadges } from '@/components/history/RefBadges'
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

    expect(screen.getByLabelText('HEAD')).toBeTruthy()
  })

  it('nhánh HEAD luôn là nhãn chính dù backend trả nó sau tag', () => {
    const { container } = render(
      <RefBadges
        refs={[
          ref({ fullName: 'refs/tags/v1', shortName: 'v1', kind: 'tag' }),
          ref({ fullName: 'refs/heads/master', kind: 'localBranch', isHead: true }),
        ]}
      />,
    )

    const primary = container.querySelector('.ref-badges > .ref-badge')
    expect(primary?.textContent).toContain('master')
  })
})

describe('gộp local + remote', () => {
  it('main + origin/main -> MỘT nhãn main, không có +N', () => {
    const { container } = render(
      <RefBadges
        refs={[
          ref({ fullName: 'refs/heads/main', kind: 'localBranch' }),
          ref({
            fullName: 'refs/remotes/origin/main',
            shortName: 'origin/main',
            kind: 'remoteBranch',
          }),
        ]}
      />,
    )

    const badges = container.querySelectorAll('.ref-badges > .ref-badge')
    expect(badges).toHaveLength(1)
    expect(badges[0]?.textContent).toBe('main')
    expect(container.querySelector('.ref-badge--overflow')).toBeNull()
  })

  it('origin/HEAD không bị gộp theo tên "HEAD"', () => {
    const groups = groupRefs([
      ref({ fullName: 'refs/remotes/origin/HEAD', shortName: 'origin/HEAD', kind: 'remoteBranch' }),
      ref({ fullName: 'refs/remotes/upstream/HEAD', shortName: 'upstream/HEAD', kind: 'remoteBranch' }),
    ])
    expect(groups.map((g) => g.name)).toEqual(['origin/HEAD', 'upstream/HEAD'])
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

    expect(screen.getByText('+11')).toBeTruthy()
  })

  it('rê chuột vào nhóm nhãn -> popover liệt kê đủ mọi nhãn; rời chuột -> đóng', () => {
    const refs = [
      ref({ fullName: 'refs/heads/master', kind: 'localBranch', isHead: true }),
      ref({ fullName: 'refs/tags/v1.16.013', shortName: 'v1.16.013', kind: 'tag' }),
    ]
    const { container } = render(<RefBadges refs={refs} />)
    const group = container.querySelector('.ref-badges')!

    fireEvent.mouseEnter(group)
    const popover = screen.getByRole('tooltip')
    expect(popover.textContent).toContain('master')
    expect(popover.textContent).toContain('v1.16.013')

    fireEvent.mouseLeave(group)
    expect(screen.queryByRole('tooltip')).toBeNull()
  })
})
