/**
 * Thanh bên nhánh/tag — HIST-07.
 *
 * Ba nhóm (Nhánh local, Nhánh remote, Tag) mỗi nhóm có số đếm trong tiêu đề,
 * kể cả khi rỗng (người dùng biết là rỗng, không phải bị thiếu). Bấm một
 * nhánh gọi `onSelectCommit(ref.target)` — điều hướng trong Phase 2, KHÔNG
 * checkout (đó là Phase 6).
 */

import { useEffect } from 'react'

import type { GitRef } from '@/lib/ipc'
import { useRefsStore } from '@/stores/refsStore'

interface Props {
  repoId: string
  onSelectCommit: (commitId: string) => void
}

function BranchRow({ r, onSelectCommit }: { r: GitRef; onSelectCommit: (id: string) => void }) {
  return (
    <li className={`ref-sidebar-item${r.isHead ? ' ref-sidebar-item--head' : ''}`}>
      <button className="ref-sidebar-link" onClick={() => onSelectCommit(r.target)}>
        <span className="ref-sidebar-name">{r.shortName}</span>
        {r.isHead && <span className="ref-sidebar-head-flag">HEAD</span>}
        {(r.ahead !== 0 || r.behind !== 0) && (
          <span className="ref-sidebar-tracking">
            {r.ahead !== 0 && <span className="ref-ahead">↑{r.ahead}</span>}
            {r.behind !== 0 && <span className="ref-behind">↓{r.behind}</span>}
          </span>
        )}
      </button>
    </li>
  )
}

function TagRow({ r, onSelectCommit }: { r: GitRef; onSelectCommit: (id: string) => void }) {
  return (
    <li className="ref-sidebar-item">
      <button className="ref-sidebar-link" onClick={() => onSelectCommit(r.target)}>
        <span className="ref-sidebar-name">{r.shortName}</span>
      </button>
    </li>
  )
}

export function RefSidebar({ repoId, onSelectCommit }: Props) {
  const slice = useRefsStore((s) => s.byRepo[repoId])
  const load = useRefsStore((s) => s.load)

  useEffect(() => {
    void load(repoId)
  }, [repoId, load])

  const refs = slice?.refs ?? []
  const localBranches = refs.filter((r) => r.kind === 'localBranch')
  const remoteBranches = refs.filter((r) => r.kind === 'remoteBranch')
  const tags = refs.filter((r) => r.kind === 'tag')
  const hasHead = refs.some((r) => r.isHead)

  return (
    <div className="ref-sidebar">
      {slice?.error && (
        <p className="ref-sidebar-error" role="alert">
          {slice.error}
        </p>
      )}

      {!hasHead && refs.length > 0 && (
        <p className="ref-sidebar-detached">HEAD đang ở trạng thái tách rời (detached HEAD).</p>
      )}

      <section className="ref-sidebar-group">
        <h3>Nhánh local ({localBranches.length})</h3>
        <ul>
          {localBranches.map((r) => (
            <BranchRow key={r.fullName} r={r} onSelectCommit={onSelectCommit} />
          ))}
        </ul>
      </section>

      <section className="ref-sidebar-group">
        <h3>Nhánh remote ({remoteBranches.length})</h3>
        <ul>
          {remoteBranches.map((r) => (
            <BranchRow key={r.fullName} r={r} onSelectCommit={onSelectCommit} />
          ))}
        </ul>
      </section>

      <section className="ref-sidebar-group">
        <h3>Tag ({tags.length})</h3>
        <ul>
          {tags.map((r) => (
            <TagRow key={r.fullName} r={r} onSelectCommit={onSelectCommit} />
          ))}
        </ul>
      </section>
    </div>
  )
}
