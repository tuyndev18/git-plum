/**
 * Danh sách commit ảo hoá + cột đồ thị — trái tim của tiêu chí thành công số
 * 2 (đồ thị luôn thẳng hàng tuyệt đối với văn bản).
 *
 * MỘT scroll container, MỘT lời gọi hook ảo hoá của `@tanstack/react-virtual`.
 * Hai vùng cuộn đồng bộ là bị cấm (`ScrollSync` của react-virtualized có lỗi
 * trễ chưa sửa nhiều năm, issue #369) — xem CONTEXT.md. Cách làm ở đây khiến
 * lệch hàng bất khả thi về mặt cấu trúc, không phải "được sửa cho thẳng".
 */

import { forwardRef, useEffect, useImperativeHandle, useMemo, useRef, useState } from 'react'
import { useVirtualizer } from '@tanstack/react-virtual'

import { ROW_HEIGHT, graphWidth } from '@/lib/graph-render/geometry'
import type { GraphRenderRow } from '@/lib/graph-render/types'
import { useHistoryStore } from '@/stores/historyStore'
import { useRefsStore } from '@/stores/refsStore'
import { GraphCanvas } from './GraphCanvas'
import { RefBadges } from './RefBadges'

interface Props {
  repoId: string
  selectedCommitId: string | null
  onSelect: (commitId: string) => void
}

/**
 * API lộ ra ngoài qua `ref` — HIST-10 cần `scrollToIndex` cho `CommitSearch`
 * mà KHÔNG tạo virtualizer thứ hai. Đây là điểm nối duy nhất: mọi component
 * khác muốn cuộn tới một hàng đi qua handle này, không tự gọi
 * `useVirtualizer` ở nơi khác.
 */
export interface CommitListHandle {
  scrollToIndex: (index: number) => void
}

const OVERSCAN = 10

function formatTime(unixSeconds: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(unixSeconds * 1000))
}

export const CommitList = forwardRef<CommitListHandle, Props>(function CommitList(
  { repoId, selectedCommitId, onSelect },
  ref,
) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const slice = useHistoryStore((s) => s.byRepo[repoId])
  const ensureRange = useHistoryStore((s) => s.ensureRange)
  const refsByCommit = useRefsStore((s) => s.byRepo[repoId]?.byCommit)

  const commits = slice?.commits ?? []
  const graphRows = slice?.graphRows ?? []
  const total = slice?.total ?? 0

  // Đúng một lời gọi hook ảo hoá trong toàn bộ src/components/history/ — cả
  // GraphCanvas (đồ thị) và cột văn bản dưới đây đều dựng từ CÙNG mảng
  // virtualItems mà hook này trả ra. Không có phép tính song song nào khác
  // sinh toạ độ Y — đó là điều khiến lệch hàng bất khả thi về mặt cấu trúc.
  // Đừng "dọn dẹp" điều này thành hai nguồn số liệu.
  const virtualizer = useVirtualizer({
    count: total,
    estimateSize: () => ROW_HEIGHT,
    getScrollElement: () => scrollRef.current,
    overscan: OVERSCAN,
  })

  const virtualItems = virtualizer.getVirtualItems()

  // Điểm nối duy nhất cho HIST-10: `CommitSearch` gọi `ref.current.scrollToIndex`
  // thay vì tự tạo một virtualizer thứ hai chỉ để cuộn.
  useImperativeHandle(
    ref,
    () => ({
      scrollToIndex: (index: number) => virtualizer.scrollToIndex(index),
    }),
    [virtualizer],
  )

  useEffect(() => {
    const first = virtualItems[0]
    const last = virtualItems[virtualItems.length - 1]
    if (!first || !last) return
    void ensureRange(repoId, first.index, last.index + 1)
  }, [repoId, virtualItems, ensureRange])

  // Đồ thị vẽ từ CÙNG virtualItems, khớp theo chỉ số với graphRows.
  const renderRows: GraphRenderRow[] = useMemo(
    () =>
      virtualItems
        .map((v) => {
          const row = graphRows[v.index]
          if (!row) return null
          return { row, index: v.index, y: v.start }
        })
        .filter((r): r is GraphRenderRow => r !== null),
    [virtualItems, graphRows],
  )

  const maxLane = useMemo(
    () => renderRows.reduce((max, r) => Math.max(max, r.row.lane), 0),
    [renderRows],
  )

  // `scrollRef.current?.clientHeight` đọc trực tiếp trong thân render từng bị
  // kẹt ở 0: lần render đầu tiên `scrollRef.current` còn `null` (ref chưa gắn),
  // và không có gì buộc component render lại SAU KHI container có kích thước
  // thật — GraphCanvas có thể nhận `height=0` vĩnh viễn nếu không có re-render
  // nào khác xảy ra tình cờ. `ResizeObserver` theo dõi kích thước thật và ép
  // một lần render lại đúng lúc container có chiều cao, để canvas luôn vẽ
  // đúng vùng nhìn thấy được.
  const [scrollHeight, setScrollHeight] = useState(0)

  useEffect(() => {
    const el = scrollRef.current
    if (!el) return

    setScrollHeight(el.clientHeight)

    if (typeof ResizeObserver === 'undefined') return
    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (entry) setScrollHeight(entry.contentRect.height)
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  return (
    <div ref={scrollRef} className="commit-scroll" data-testid="commit-scroll">
      <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
        <GraphCanvas
          rows={renderRows}
          width={graphWidth(maxLane)}
          height={scrollHeight}
          selectedCommitId={selectedCommitId}
        />
        {virtualItems.map((v) => {
          const commit = commits[v.index]
          const isSelected = commit !== undefined && commit.id === selectedCommitId

          return (
            <div
              key={v.key}
              className={`commit-row${isSelected ? ' selected' : ''}`}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                right: 0,
                transform: `translateY(${v.start}px)`,
                height: v.size,
              }}
              onClick={() => {
                if (commit) onSelect(commit.id)
              }}
            >
              {commit ? (
                <>
                  <span className="commit-graph-gutter" style={{ width: graphWidth(maxLane) }} />
                  <span className="commit-subject" title={commit.subject}>
                    <RefBadges refs={refsByCommit?.get(commit.id) ?? []} />
                    {commit.subject}
                    {commit.hasInvalidUtf8 && (
                      <span className="commit-encoding-flag" title="Chứa byte không phải UTF-8">
                        {' '}
                        ⚠
                      </span>
                    )}
                  </span>
                  <span className="commit-author">{commit.authorName}</span>
                  <span className="commit-time">{formatTime(commit.authorTime)}</span>
                  <span className="commit-sha">{commit.id.slice(0, 7)}</span>
                </>
              ) : (
                <span className="commit-row-loading" />
              )}
            </div>
          )
        })}
      </div>
    </div>
  )
})
