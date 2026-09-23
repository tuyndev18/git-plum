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
 *
 * Nhiều ref trên một commit — theo GitKraken:
 * - Nhánh local và remote CÙNG tên (`main` + `origin/main`) gộp thành một
 *   nhãn, kèm icon máy tính (local) và/hoặc đám mây (remote).
 * - Hàng chỉ hiện MỘT nhãn chính (ưu tiên nhánh HEAD, rồi local, remote, tag)
 *   cộng chỉ báo `+N`. Cột nhãn hẹp (≤220px) — nhiều nhãn cắt còn vài ký tự
 *   không đọc được, một nhãn đọc trọn tốt hơn.
 * - Rê chuột vào nhóm nhãn mở danh sách đầy đủ. Danh sách render qua portal
 *   với `position: fixed` vì hàng và ô nhãn đều `overflow: hidden` (bắt buộc
 *   để giữ chiều cao hàng cố định, xem `app.css`) — render tại chỗ sẽ bị cắt.
 */

import { useRef, useState } from 'react'
import { createPortal } from 'react-dom'

import type { GitRef } from '@/lib/ipc'

interface Props {
  refs: GitRef[]
}

/** Một nhãn hiển thị: một nhánh (local, remote hoặc cả hai) hoặc một tag/ref khác. */
interface RefGroup {
  key: string
  name: string
  kind: GitRef['kind']
  local: GitRef | null
  remotes: GitRef[]
  isHead: boolean
  fullNames: string[]
}

function classFor(kind: GitRef['kind']): string {
  if (kind === 'localBranch') return 'ref-badge--local'
  if (kind === 'remoteBranch') return 'ref-badge--remote'
  if (kind === 'tag') return 'ref-badge--tag'
  return 'ref-badge--other'
}

/** `origin/feature/x` -> `feature/x`. Tên remote không chứa `/`. */
function stripRemote(shortName: string): string {
  const slash = shortName.indexOf('/')
  return slash === -1 ? shortName : shortName.slice(slash + 1)
}

function rank(g: RefGroup): number {
  if (g.isHead) return 0
  if (g.kind === 'localBranch') return 1
  if (g.kind === 'remoteBranch') return 2
  if (g.kind === 'tag') return 3
  return 4
}

/**
 * Gộp local + remote cùng tên, sắp theo độ ưu tiên. Sắp ổn định: trong cùng
 * hạng giữ thứ tự từ backend.
 */
export function groupRefs(refs: GitRef[]): RefGroup[] {
  const branches = new Map<string, RefGroup>()
  const groups: RefGroup[] = []

  for (const r of refs) {
    if (r.kind === 'localBranch' || r.kind === 'remoteBranch') {
      const name = r.kind === 'localBranch' ? r.shortName : stripRemote(r.shortName)
      // Remote HEAD (`origin/HEAD`) chỉ là con trỏ tới nhánh mặc định — không gộp
      // theo tên "HEAD" với nhánh nào cả, giữ nguyên tên đầy đủ.
      const mergeKey = name === 'HEAD' ? `ref:${r.fullName}` : `branch:${name}`
      let g = branches.get(mergeKey)
      if (!g) {
        g = {
          key: mergeKey,
          name: name === 'HEAD' ? r.shortName : name,
          kind: r.kind,
          local: null,
          remotes: [],
          isHead: false,
          fullNames: [],
        }
        branches.set(mergeKey, g)
        groups.push(g)
      }
      if (r.kind === 'localBranch') {
        g.local = r
        g.kind = 'localBranch'
      } else {
        g.remotes.push(r)
      }
      g.isHead ||= r.isHead
      g.fullNames.push(r.fullName)
    } else {
      groups.push({
        key: `ref:${r.fullName}`,
        name: r.shortName,
        kind: r.kind,
        local: null,
        remotes: [],
        isHead: r.isHead,
        fullNames: [r.fullName],
      })
    }
  }

  // Nhánh chỉ có ở MỘT remote, không có local: giữ tên đầy đủ `origin/main` —
  // bỏ tiền tố thì không phân biệt được với nhánh local cùng tên ở commit khác.
  for (const g of groups) {
    if (!g.local && g.remotes.length === 1) g.name = g.remotes[0]!.shortName
  }

  return groups
    .map((g, i) => ({ g, i }))
    .sort((a, b) => rank(a.g) - rank(b.g) || a.i - b.i)
    .map(({ g }) => g)
}

/** Icon tag 10px sau tên tag, như GitKraken. `aria-hidden`: tên đã đủ nghĩa. */
function IconTag() {
  return (
    <svg
      className="ref-badge-icon"
      width="10"
      height="10"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
    >
      <path d="M2 2h6l6 6-6 6-6-6z" strokeLinejoin="round" />
      <circle cx="5.5" cy="5.5" r="1.2" fill="currentColor" stroke="none" />
    </svg>
  )
}

function IconCheck() {
  return (
    <svg
      className="ref-badge-icon"
      width="10"
      height="10"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="2.2"
      aria-hidden="true"
    >
      <path d="M3 8.5l3.2 3.2L13 4.8" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  )
}

function IconLocal() {
  return (
    <svg
      className="ref-badge-icon"
      width="11"
      height="10"
      viewBox="0 0 18 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      aria-hidden="true"
    >
      <rect x="3" y="2.5" width="12" height="8" rx="1" />
      <path d="M1 13.5h16" strokeLinecap="round" />
    </svg>
  )
}

function IconRemote() {
  return (
    <svg
      className="ref-badge-icon"
      width="11"
      height="10"
      viewBox="0 0 18 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      aria-hidden="true"
    >
      <path
        d="M5 13h8.5a3 3 0 0 0 .4-6A4.5 4.5 0 0 0 5.3 6 3.5 3.5 0 0 0 5 13z"
        strokeLinejoin="round"
      />
    </svg>
  )
}

function GroupBadge({ g }: { g: RefGroup }) {
  const label = g.isHead ? `${g.fullNames.join(', ')} (HEAD)` : g.fullNames.join(', ')
  return (
    <span className={`ref-badge ${classFor(g.kind)}`} title={label}>
      {g.isHead && (
        <span className="ref-badge-head" aria-label="HEAD">
          <IconCheck />
        </span>
      )}
      <span className="ref-badge-name">{g.name}</span>
      {g.local && <IconLocal />}
      {g.remotes.length > 0 && <IconRemote />}
      {g.kind === 'tag' && <IconTag />}
    </span>
  )
}

export function RefBadges({ refs }: Props) {
  const anchor = useRef<HTMLSpanElement>(null)
  const [popover, setPopover] = useState<{ top: number; left: number } | null>(null)

  const groups = groupRefs(refs)
  const [primary, ...rest] = groups
  if (!primary) return null

  function open() {
    if (rest.length === 0 || !anchor.current) return
    const r = anchor.current.getBoundingClientRect()
    setPopover({ top: r.bottom + 2, left: r.left })
  }

  return (
    <span
      ref={anchor}
      className="ref-badges"
      onMouseEnter={open}
      onMouseLeave={() => setPopover(null)}
    >
      <GroupBadge g={primary} />
      {rest.length > 0 && (
        <span className="ref-badge ref-badge--overflow" aria-label={`còn ${rest.length} nhãn`}>
          +{rest.length}
        </span>
      )}
      {popover &&
        createPortal(
          <div
            className="ref-popover"
            role="tooltip"
            style={{ top: popover.top, left: popover.left }}
          >
            {groups.map((g) => (
              <GroupBadge key={g.key} g={g} />
            ))}
          </div>,
          document.body,
        )}
    </span>
  )
}
