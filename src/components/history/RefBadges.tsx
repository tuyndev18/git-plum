/**
 * Nhãn nhánh/tag trong hàng commit — HIST-06.
 *
 * Props `refs: GitRef[]` truyền vào từ ngoài (KHÔNG tự gọi store) — hàng
 * render tới hàng nghìn lần khi cuộn, mỗi component tự subscribe
 * `refsStore` là chi phí vô ích. `CommitList` tra `refsStore.refsForCommit`
 * một lần cho mỗi hàng đang hiển thị rồi truyền xuống.
 *
 * Ba class riêng theo `kind` (không phải một class dùng inline style khác
 * màu) — HIST-06 đòi *phân biệt được* local với remote.
 */

import type { GitRef } from '@/lib/ipc'

interface Props {
  refs: GitRef[]
}

const MAX_VISIBLE_BADGES = 3

function classFor(kind: GitRef['kind']): string {
  if (kind === 'localBranch') return 'ref-badge--local'
  if (kind === 'remoteBranch') return 'ref-badge--remote'
  if (kind === 'tag') return 'ref-badge--tag'
  return 'ref-badge--other'
}

export function RefBadges({ refs }: Props) {
  if (refs.length === 0) return null

  const visible = refs.slice(0, MAX_VISIBLE_BADGES)
  const overflow = refs.slice(MAX_VISIBLE_BADGES)

  return (
    <span className="ref-badges">
      {visible.map((r) => (
        <span key={r.fullName} className={`ref-badge ${classFor(r.kind)}`} title={r.fullName}>
          {r.shortName}
          {r.isHead && <span className="ref-badge-head"> HEAD</span>}
        </span>
      ))}
      {overflow.length > 0 && (
        <span
          className="ref-badge ref-badge--overflow"
          title={overflow.map((r) => r.shortName).join(', ')}
        >
          +{overflow.length}
        </span>
      )}
    </span>
  )
}
