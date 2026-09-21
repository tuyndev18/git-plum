/**
 * Hộp tìm kiếm commit — HIST-10, bốn trục: thông điệp, tác giả, mã commit,
 * đường dẫn tệp (backend `search_commits` gộp cả bốn, xem `historyStore`/
 * `docs/01-research-competitors.md` mục 5).
 *
 * Trì hoãn 250ms (ARCHITECTURE.md Anti-Pattern 4: chạy git mỗi lần gõ là
 * DoS). Huỷ kết quả của lời gọi cũ bằng số thứ tự tăng dần — gõ/cuộn nhanh
 * không được làm kết quả về ngược thứ tự.
 *
 * `scrollToIndex` nhận từ ngoài (prop, lấy từ virtualizer của `CommitList`)
 * — KHÔNG tạo virtualizer thứ hai ở đây.
 */

import { useEffect, useMemo, useRef, useState } from 'react'

import { describeError, ipc } from '@/lib/ipc'
import { registerCommands, unregisterCommand } from '@/lib/commands'
import { useHistoryStore } from '@/stores/historyStore'

const DEBOUNCE_MS = 250

interface Props {
  repoId: string
  scrollToIndex: (index: number) => void
}

export function CommitSearch({ repoId, scrollToIndex }: Props) {
  const inputRef = useRef<HTMLInputElement>(null)
  const [query, setQuery] = useState('')
  const [matchIds, setMatchIds] = useState<string[] | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [activeIndex, setActiveIndex] = useState(0)

  const commits = useHistoryStore((s) => s.byRepo[repoId]?.commits)

  // Chỉ mục commitId -> vị trí hàng, dựng một lần khi `commits` đổi — không
  // quét tuyến tính mỗi lần nhảy tới khớp.
  const indexById = useMemo(() => {
    const map = new Map<string, number>()
    if (commits) {
      for (let i = 0; i < commits.length; i += 1) {
        const c = commits[i]
        if (c) map.set(c.id, i)
      }
    }
    return map
  }, [commits])

  // Số thứ tự tăng dần để huỷ kết quả của lời gọi cũ — gõ nhanh không làm
  // kết quả về ngược thứ tự (T-02-21 một phần: DoS qua bão lời gọi + kết quả
  // lộn xộn).
  const requestSeq = useRef(0)

  useEffect(() => {
    registerCommands([
      {
        id: 'history.search',
        title: 'Tìm kiếm commit',
        category: 'history',
        keybinding: 'Ctrl+F',
        run: () => inputRef.current?.focus(),
      },
    ])
    return () => unregisterCommand('history.search')
  }, [])

  useEffect(() => {
    if (query.trim() === '') {
      setMatchIds(null)
      setError(null)
      return
    }

    const timer = setTimeout(() => {
      const seq = ++requestSeq.current
      ipc
        .searchCommits(repoId, query)
        .then((ids) => {
          if (seq !== requestSeq.current) return // kết quả cũ, bỏ qua
          setMatchIds(ids)
          setError(null)
          setActiveIndex(0)
        })
        .catch((e) => {
          if (seq !== requestSeq.current) return
          setError(describeError(e))
          setMatchIds(null)
        })
    }, DEBOUNCE_MS)

    return () => clearTimeout(timer)
  }, [repoId, query])

  // Nhảy tới khớp hiện tại khi danh sách kết quả đổi hoặc activeIndex đổi.
  useEffect(() => {
    if (!matchIds || matchIds.length === 0) return
    const id = matchIds[activeIndex]
    if (id === undefined) return
    const rowIndex = indexById.get(id)
    if (rowIndex !== undefined) scrollToIndex(rowIndex)
  }, [matchIds, activeIndex, indexById, scrollToIndex])

  const skippedCount = matchIds ? matchIds.filter((id) => !indexById.has(id)).length : 0

  const goNext = () => {
    if (!matchIds || matchIds.length === 0) return
    setActiveIndex((i) => (i + 1) % matchIds.length)
  }

  const goPrev = () => {
    if (!matchIds || matchIds.length === 0) return
    setActiveIndex((i) => (i - 1 + matchIds.length) % matchIds.length)
  }

  return (
    <div className="commit-search">
      <input
        ref={inputRef}
        type="text"
        className="commit-search-input"
        placeholder="Tìm theo thông điệp, tác giả, mã commit, đường dẫn tệp…"
        value={query}
        disabled={false}
        onChange={(e) => setQuery(e.target.value)}
      />

      <button onClick={goPrev} disabled={!matchIds || matchIds.length === 0} title="Khớp trước">
        ↑
      </button>
      <button onClick={goNext} disabled={!matchIds || matchIds.length === 0} title="Khớp kế tiếp">
        ↓
      </button>

      {matchIds && matchIds.length > 0 && (
        <span className="commit-search-count">
          {activeIndex + 1}/{matchIds.length}
        </span>
      )}

      {matchIds && matchIds.length === 0 && (
        <span className="commit-search-empty">Không tìm thấy</span>
      )}

      {skippedCount > 0 && (
        <span className="commit-search-skipped" title="Một số khớp nằm ngoài phần lịch sử đã nạp">
          (bỏ qua {skippedCount})
        </span>
      )}

      {error && (
        <span className="commit-search-error" role="alert">
          {error}
        </span>
      )}
    </div>
  )
}
