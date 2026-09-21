/**
 * Danh sách commit ảo hoá + cột đồ thị — trái tim của tiêu chí thành công số
 * 2 (đồ thị luôn thẳng hàng tuyệt đối với văn bản).
 *
 * MỘT scroll container, MỘT lời gọi hook ảo hoá của `@tanstack/react-virtual`.
 * Hai vùng cuộn đồng bộ là bị cấm (`ScrollSync` của react-virtualized có lỗi
 * trễ chưa sửa nhiều năm, issue #369) — xem CONTEXT.md. Cách làm ở đây khiến
 * lệch hàng bất khả thi về mặt cấu trúc, không phải "được sửa cho thẳng".
 */

import type { CSSProperties } from 'react'
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

  // Vị trí cuộn hiện tại. Cần vì canvas chỉ cao bằng VÙNG NHÌN THẤY
  // (`scrollHeight`) chứ không cao bằng cả danh sách, nên `y` đưa cho bộ vẽ
  // phải là toạ độ TRONG canvas, không phải toạ độ trong danh sách.
  //
  // Thiếu phép trừ này thì hàng thứ 100 có `v.start = 2800` được vẽ ở y=2800
  // trên một canvas cao ~550px — tức vẽ ra ngoài vùng canvas và mất hẳn. Chỉ
  // vài hàng đầu (`v.start < chiều cao canvas`) là còn thấy, đúng hiện tượng
  // "đồ thị chỉ có một chấm ở hàng đầu" mà người dùng báo.
  const [scrollTop, setScrollTop] = useState(0)

  useEffect(() => {
    const el = scrollRef.current
    if (!el) return

    setScrollTop(el.scrollTop)
    const onScroll = () => setScrollTop(el.scrollTop)
    el.addEventListener('scroll', onScroll, { passive: true })
    return () => el.removeEventListener('scroll', onScroll)
  }, [])

  // Đồ thị vẽ từ CÙNG virtualItems, khớp theo chỉ số với graphRows.
  //
  // `y` là `v.start - scrollTop`: vẫn MỘT nguồn toạ độ duy nhất (virtualizer),
  // chỉ đổi hệ quy chiếu từ "trong danh sách" sang "trong canvas". Cột văn bản
  // dùng `translateY(v.start)` vì nó nằm trong div cao bằng cả danh sách; canvas
  // dính theo vùng nhìn thấy nên phải trừ đi phần đã cuộn qua.
  const renderRows: GraphRenderRow[] = useMemo(
    () =>
      virtualItems
        .map((v) => {
          const row = graphRows[v.index]
          if (!row) return null
          return { row, index: v.index, y: v.start - scrollTop }
        })
        .filter((r): r is GraphRenderRow => r !== null),
    [virtualItems, graphRows, scrollTop],
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

  const graphColWidth = graphWidth(maxLane)

  return (
    <div className="commit-pane">
      {/*
        Header cột nằm NGOÀI vùng cuộn nên nó không cuộn theo danh sách, đúng
        như ảnh tham chiếu (`docs/screenshots/`).

        Bề rộng hai cột đầu lấy từ cùng `graphColWidth` mà canvas dùng, nên
        header luôn thẳng cột với nội dung — không có con số px nào viết cứng
        lặp lại ở đây.
      */}
      <div className="commit-header" aria-hidden="true">
        <span className="commit-header-ref">Nhánh / Tag</span>
        <span className="commit-header-graph" style={{ width: graphColWidth }}>
          Đồ thị
        </span>
        <span className="commit-header-subject">Thông điệp commit</span>
      </div>

      <div
        ref={scrollRef}
        className="commit-scroll"
        data-testid="commit-scroll"
        // Canvas `position: absolute` neo theo khối chứa nên nó cuộn đi cùng
        // danh sách; đẩy `top` theo `scrollTop` giữ nó đứng yên ở vùng nhìn
        // thấy. Đặt qua biến CSS thay vì style trực tiếp trên canvas để
        // `.graph-canvas` giữ trọn phần định vị trong `app.css` — xem khối
        // comment ở đó cho lý do bỏ `position: sticky`.
        style={{ '--graph-scroll-top': `${scrollTop}px` } as CSSProperties}
      >
        {/*
        Canvas nằm NGOÀI div nội dung và `position: absolute` + `top` chạy theo
        `scrollTop` để nó đứng yên ở vùng nhìn thấy thay vì cuộn đi cùng danh
        sách.

        Trước đây canvas nằm TRONG div cao `getTotalSize()` (có thể hàng chục
        nghìn px) nhưng bản thân chỉ cao `scrollHeight` (~550px) và neo ở top 0
        — nên nó chỉ phủ được phần đầu danh sách, và mọi hàng cuộn xuống dưới
        không còn canvas để vẽ lên. Kết hợp với `y` tuyệt đối (xem `renderRows`)
        là hai lỗi cùng gây ra hiện tượng "đồ thị chỉ còn một chấm".
      */}
      <GraphCanvas
        rows={renderRows}
        width={graphWidth(maxLane)}
        height={scrollHeight}
        selectedCommitId={selectedCommitId}
      />
      <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
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
                  {/*
                    Thứ tự cột theo ảnh tham chiếu: NHÃN trước, ĐỒ THỊ sau, rồi
                    thông điệp. Bản trước đặt đồ thị trước nhãn.

                    Ô bọc `.commit-ref-cell` LUÔN được render, kể cả khi không
                    có ref nào: `RefBadges` trả `null` theo đúng đặc tả ("không
                    có ref -> không render gì, không chiếm chiều cao"), và nếu
                    để nó tự làm ô grid thì hàng không nhãn sẽ THIẾU một ô —
                    mọi ô sau đó dồn sang trái một cột và lệch cột so với hàng
                    có nhãn. Ô rỗng có bề rộng 0 nên không tốn chỗ, chỉ giữ
                    đúng số lượng ô grid cho mọi hàng.

                    Nhãn có cột riêng, không nằm trong `.commit-subject` — sửa
                    nguyên nhân B của checkpoint round 1 (badge ăn hết không
                    gian chữ message; đo thật: nhóm 4 badge chiếm 258px trong
                    khi cột subject chỉ còn 144px ở cửa sổ 900px, chữ message
                    hiển thị 0%).
                  */}
                  <span className="commit-ref-cell">
                    <RefBadges refs={refsByCommit?.get(commit.id) ?? []} />
                  </span>
                  <span className="commit-graph-gutter" style={{ width: graphColWidth }} />
                  <span className="commit-subject" title={commit.subject}>
                    {commit.subject}
                    {commit.hasInvalidUtf8 && (
                      <span className="commit-encoding-flag" title="Chứa byte không phải UTF-8">
                        {' '}
                        ⚠
                      </span>
                    )}
                    {/*
                      Phần body xám nhạt nối ngay sau subject trên CÙNG một
                      dòng, như tham chiếu. `body` đã có sẵn từ `LOG_FORMAT`
                      (`%b`, wave 2) nhưng trước đây không dùng ở tầng hiển thị.

                      Gộp dòng: body của git xuống dòng thật, mà hàng chỉ cao
                      ROW_HEIGHT cố định nên phải rút về một dòng — nếu không,
                      chiều cao hàng vỡ và đồ thị lệch (đúng lớp lỗi của
                      checkpoint round 1).
                    */}
                    {commit.body.trim() !== '' && (
                      <span className="commit-body"> {commit.body.replace(/\s+/g, ' ').trim()}</span>
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
    </div>
  )
})
