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
import { avatarTuSinh } from '@/lib/avatar'

import { ROW_HEIGHT, colorFor, graphWidth } from '@/lib/graph-render/geometry'
import type { GraphRenderRow, WipEdgeRender } from '@/lib/graph-render/types'
import {
  commitRowY,
  contentOffset,
  hasWipRow,
  virtualizerCount,
  wipCountsFromStatus,
  wipEdge,
  WIP_ROW_HEIGHT,
} from '@/lib/graph-render/wipRow'
import { isPerfEnabled, measureFirstPaint, measureScrollFps } from '@/lib/perf'
import { useHistoryStore } from '@/stores/historyStore'
import { useRefsStore } from '@/stores/refsStore'
import { useStatusStore } from '@/stores/statusStore'
import { GraphCanvas } from './GraphCanvas'
import { RefBadges } from './RefBadges'

interface Props {
  repoId: string
  selectedCommitId: string | null
  onSelect: (commitId: string) => void
  /**
   * Bấm hàng WIP → mở vùng soạn commit (WORK-11).
   *
   * 🔴 Một handler **riêng**, không tái dùng `onSelect`. Hàng WIP không có SHA,
   * nên gọi `onSelect` ở đó buộc phải bịa một commitId — và `CommitDetail` sẽ
   * đi hỏi backend về một sha không tồn tại. Có test ghim rằng `onSelect`
   * **không** được gọi khi bấm hàng WIP.
   */
  onOpenCommitBox?: () => void
}

/**
 * API lộ ra ngoài qua `ref` — HIST-10 cần `scrollToIndex` cho `CommitSearch`
 * mà KHÔNG tạo virtualizer thứ hai. Đây là điểm nối duy nhất: mọi component
 * khác muốn cuộn tới một hàng đi qua handle này, không tự gọi
 * `useVirtualizer` ở nơi khác.
 */
export interface CommitListHandle {
  scrollToIndex: (index: number) => void
  /**
   * Cuộn tới hàng mang `commitId`. Trả `false` khi commit đó **chưa nằm trong
   * phần lịch sử đã nạp** — phía gọi phải nói ra, không được nuốt im lặng.
   *
   * Vì sao trả `boolean` chứ không tự nạp thêm: `historyStore` nạp theo trang
   * 1000 dòng và `ensureRange` chỉ biết **chỉ số hàng**, không biết sha. Đổi
   * sha → chỉ số cho một commit nằm ngoài vùng đã nạp cần `git rev-list
   * --count` phía Rust, tức một lệnh IPC mới — cố ý để ngoài phạm vi. Ở đây
   * `false` là hợp đồng để giao diện hiện "nằm ngoài phần lịch sử đã nạp",
   * cùng cách `CommitSearch` đã đếm `skippedCount`.
   */
  scrollToCommit: (commitId: string) => boolean
}

const OVERSCAN = 10

/*
 * `04/22/2026 @ 10:02 AM` — kiểu GitKraken: ngày số cố định bề rộng (đọc dọc cột
 * thẳng hàng, khác `medium` có tên tháng dài ngắn khác nhau), giờ tách bằng `@`.
 * Thứ tự ngày/tháng và 12/24 giờ vẫn theo locale người dùng. Tạo formatter một
 * lần: hàm này chạy cho mọi hàng đang thấy ở mỗi lần render khi cuộn.
 */
const DATE_FMT = new Intl.DateTimeFormat(undefined, {
  year: 'numeric',
  month: '2-digit',
  day: '2-digit',
})
const TIME_FMT = new Intl.DateTimeFormat(undefined, { hour: 'numeric', minute: '2-digit' })

function formatTime(unixSeconds: number): string {
  const d = new Date(unixSeconds * 1000)
  return `${DATE_FMT.format(d)} @ ${TIME_FMT.format(d)}`
}

export const CommitList = forwardRef<CommitListHandle, Props>(function CommitList(
  { repoId, selectedCommitId, onSelect, onOpenCommitBox },
  ref,
) {
  const scrollRef = useRef<HTMLDivElement>(null)
  const slice = useHistoryStore((s) => s.byRepo[repoId])
  const ensureRange = useHistoryStore((s) => s.ensureRange)
  const refsByCommit = useRefsStore((s) => s.byRepo[repoId]?.byCommit)
  const status = useStatusStore((s) => s.byRepo[repoId]?.status)

  const commits = slice?.commits ?? []
  const graphRows = slice?.graphRows ?? []
  const total = slice?.total ?? 0

  // Hàng WIP — WORK-11. Đọc `statusStore` ở đây chứ không nhận qua prop: cột
  // đồ thị và cột văn bản của hàng WIP đều dựng trong tệp này, nên một nguồn
  // duy nhất ngay tại chỗ dùng là đường ngắn nhất để hai bên không lệch.
  const wipCounts = useMemo(() => wipCountsFromStatus(status), [status])
  const hasWip = hasWipRow(wipCounts)

  // Đúng một lời gọi hook ảo hoá trong toàn bộ src/components/history/ — cả
  // GraphCanvas (đồ thị) và cột văn bản dưới đây đều dựng từ CÙNG mảng
  // virtualItems mà hook này trả ra. Không có phép tính song song nào khác
  // sinh toạ độ Y — đó là điều khiến lệch hàng bất khả thi về mặt cấu trúc.
  // Đừng "dọn dẹp" điều này thành hai nguồn số liệu.
  //
  // 🔴 `count` là `virtualizerCount(total, hasWip)`, và hàm đó trả **`total`**
  // trong cả hai ca — hàng WIP KHÔNG vào `count`. Đó là cách B, chốt ở 04-04:
  // `count: total + 1` sẽ rải một phép `-1` ra bảy chỗ tiêu thụ `virtualItems`
  // (`commits[v.index]`, `graphRows[v.index]`, `ensureRange`, `indexById`,
  // `scrollToIndex`, `scrollToCommit`, `renderRows`), và "cột đồ thị lệch cột
  // văn bản đúng một hàng" đã xảy ra HAI lần ở Phase 2.
  //
  // Gọi qua hàm chứ không viết `count: total` thẳng: nó buộc chỗ này **khai
  // báo** rằng câu hỏi "có hàng WIP không" đã được tính đến, và nó là chỗ duy
  // nhất phải sửa nếu ai đó buộc phải đổi sang cách A sau này.
  const virtualizer = useVirtualizer({
    count: virtualizerCount(total, hasWip),
    estimateSize: () => ROW_HEIGHT,
    getScrollElement: () => scrollRef.current,
    overscan: OVERSCAN,
  })

  const virtualItems = virtualizer.getVirtualItems()

  // Chỉ mục commitId -> vị trí hàng. Cùng khuôn `CommitSearch` đã dùng, dựng
  // một lần khi `commits` đổi — bấm một nhánh không được quét tuyến tính qua
  // 100k phần tử.
  //
  // `commits` là mảng **thưa**: `mergePage` cấp trước tới `total` và để các vị
  // trí chưa nạp ở `undefined`, nên vòng lặp phải bỏ qua lỗ thay vì tin rằng
  // mọi chỉ số đều có commit.
  const indexById = useMemo(() => {
    const map = new Map<string, number>()
    for (let i = 0; i < commits.length; i += 1) {
      const c = commits[i]
      if (c) map.set(c.id, i)
    }
    return map
  }, [commits])

  // Điểm nối duy nhất cho HIST-10: `CommitSearch` gọi `ref.current.scrollToIndex`
  // thay vì tự tạo một virtualizer thứ hai chỉ để cuộn. `RefSidebar` (qua `App`)
  // đi vào bằng `scrollToCommit` — cùng một virtualizer, chỉ khác đầu vào là sha
  // thay vì chỉ số.
  useImperativeHandle(
    ref,
    () => ({
      scrollToIndex: (index: number) => virtualizer.scrollToIndex(index),
      scrollToCommit: (commitId: string) => {
        const index = indexById.get(commitId)
        if (index === undefined) return false
        virtualizer.scrollToIndex(index, { align: 'center' })
        return true
      },
    }),
    [virtualizer, indexById],
  )

  useEffect(() => {
    const first = virtualItems[0]
    const last = virtualItems[virtualItems.length - 1]
    if (!first || !last) return
    void ensureRange(repoId, first.index, last.index + 1)
  }, [repoId, virtualItems, ensureRange])

  // Đồng hồ "thời gian vẽ lần đầu" của checkpoint #1 — Core Value.
  //
  // Bấm đồng hồ khi `repoId` đổi (tức bắt đầu nạp một repo mới) và dừng ở lần
  // render ĐẦU TIÊN THẬT SỰ CÓ DỮ LIỆU. Mốc dừng là `commits.length > 0`, không
  // phải lần `useEffect` đầu tiên: lần đó chạy khi màn hình còn trống, nên dừng
  // ở đó sẽ cho một con số đẹp nhưng vô nghĩa — đúng kiểu số tự lừa mình mà
  // plan này tồn tại để tránh.
  //
  // Cờ tắt → `measureFirstPaint` trả hàm rỗng, hai effect này không tốn gì.
  const dungDongHo = useRef<(() => number) | null>(null)
  const daGhiLanDau = useRef(false)

  useEffect(() => {
    daGhiLanDau.current = false
    dungDongHo.current = measureFirstPaint('commit-list-first-paint')
  }, [repoId])

  useEffect(() => {
    if (daGhiLanDau.current) return
    if (commits.length === 0) return
    daGhiLanDau.current = true
    dungDongHo.current?.()
  }, [commits.length])

  // Cổng cho checkpoint #1 bước 4: đo FPS lúc cuộn từ console của webview.
  //
  // Phơi ra `window` thay vì đăng ký vào sổ lệnh vì `Command.run` không nhận
  // tham số (xem ghi chú dài trong `App.tsx`), mà phép đo cần `durationMs`. Chỉ
  // gắn khi cờ bật, nên bản release bình thường không có thuộc tính này.
  useEffect(() => {
    const el = scrollRef.current
    if (!el) return

    // Cờ tắt thì không gắn gì cả: bản release bình thường không có thuộc tính
    // lạ nào trên `window`. `measureScrollFps` tự trả báo cáo rỗng khi tắt, nên
    // đây là lớp phòng thủ thứ hai chứ không phải lớp duy nhất.
    if (!isPerfEnabled()) return

    const w = globalThis as unknown as Record<string, unknown>
    w.gitPlumMeasureScrollFps = (durationMs = 10_000) => measureScrollFps(el, durationMs)
    return () => {
      delete w.gitPlumMeasureScrollFps
    }
  }, [])

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
  // `y` đi qua `commitRowY(v.start, scrollTop, hasWip)`: vẫn MỘT nguồn toạ độ
  // duy nhất (virtualizer), chỉ đổi hệ quy chiếu từ "trong danh sách" sang
  // "trong canvas", cộng độ lệch của hàng WIP. Cột văn bản nằm trong một div
  // cao bằng cả danh sách nên nó dùng `translateY(v.start)` và nhận cùng độ
  // lệch qua `paddingTop` của div đó — xem chỗ render bên dưới.
  //
  // 🔴 KHÔNG viết `v.start - scrollTop + contentOffset(hasWip)` ở đây. Đó là
  // dựng lại công thức của `commitRowY` ở chỗ thứ hai, và hai công thức là hai
  // dịp để lệch một hàng. Cổng số học của `wipRow.test.ts` đỏ nếu ai làm vậy.
  const renderRows: GraphRenderRow[] = useMemo(
    () =>
      virtualItems
        .map((v): GraphRenderRow | null => {
          const row = graphRows[v.index]
          if (!row) return null
          /*
           * Avatar tác giả tính **ở đây**, không trong bộ vẽ.
           *
           * `canvasRenderer` không được biết tới `avatarTuSinh` hay bảng màu
           * của nó: interface `GraphRenderer` là điểm nối của checkpoint #2
           * (canvas hay SVG), và một bộ vẽ SVG sau này phải nhận **cùng** dữ
           * liệu chứ không phải tự tính lại. Tính sẵn ở đây giữ cả hai bộ vẽ
           * nhất quán theo định nghĩa.
           *
           * `commits` là mảng thưa nên `commit` có thể `undefined` — hàng đó
           * vẫn vẽ nút bình thường, chỉ không có avatar.
           */
          const commit = commits[v.index]
          const avatar = commit ? avatarTuSinh(commit.authorName, commit.authorEmail) : undefined
          const hasRefs = commit ? (refsByCommit?.get(commit.id)?.length ?? 0) > 0 : false
          return {
            row,
            index: v.index,
            y: commitRowY(v.start, scrollTop, hasWip),
            avatar,
            hasRefs,
          }
        })
        .filter((r): r is GraphRenderRow => r !== null),
    [virtualItems, graphRows, commits, scrollTop, hasWip, refsByCommit],
  )

  /**
   * Cạnh nối hàng WIP xuống HEAD, và vị trí Y của hàng WIP trong canvas.
   *
   * 🔴 Lane lấy từ hàng mang **sha của HEAD** (`status.branch.oid`), KHÔNG từ
   * hàng 0. `git log --all --topo-order` xếp commit mới nhất theo topo của MỌI
   * ref ở hàng 0; nếu người dùng đang đứng ở một nhánh không phải nhánh mới
   * nhất thì HEAD nằm giữa danh sách và lane của nó là bất kỳ. Đã tái hiện có
   * số trên repo thật — xem doc comment của `wipEdge`.
   *
   * `undefined` khi HEAD không nằm trong phần lịch sử đã nạp: `wipEdge` biến
   * nó thành biến thể `unknownHead` và **không vẽ đường nào**, thay vì đoán
   * lane 0.
   *
   * Hàng WIP ghim ở **đầu vùng nhìn thấy** nên `y` của nó trong canvas là 0 —
   * canvas cao đúng vùng nhìn thấy (`scrollHeight`). Đó là một hằng của cách
   * B, không phải một phép tính.
   */
  const wipCanh = useMemo(() => {
    if (!hasWip) return null
    const headOid = status?.branch.oid ?? null
    const headIndex = headOid === null ? undefined : indexById.get(headOid)
    const headLane = headIndex === undefined ? null : (graphRows[headIndex]?.lane ?? null)
    return wipEdge({ headLane })
  }, [hasWip, status, indexById, graphRows])

  const wipRender: WipEdgeRender | null = useMemo(() => {
    if (!wipCanh) return null
    return wipCanh.kind === 'normal'
      ? { kind: 'normal', x: wipCanh.x, y: 0 }
      : { kind: wipCanh.kind, y: 0 }
  }, [wipCanh])

  /**
   * Lane lớn nhất **thật sự được vẽ** trên các hàng đang thấy.
   *
   * 🔴 Bản trước chỉ đọc `r.row.lane` và đó là lỗi người dùng báo bằng ảnh: hai
   * nút ở lane 1 nằm lơ lửng, không đường lane nào nối tới.
   *
   * Vì sao `row.lane` là chưa đủ: một hàng ở lane 0 vẫn **vẽ** ở lane 1, 2, 3…
   * qua `passthrough` (nhánh song song đi xuyên qua nó) và `outEdges` (cạnh rẽ
   * sang lane khác). Dữ liệu backend đúng — kiểm bằng `cargo run --bin rowdump`
   * trên repo thật: hàng 106 của `dau-tri-toan-hoc` là `lane 0` nhưng mang
   * `passthrough 1->1` và `outEdges 0->0, 0->2`.
   *
   * `maxLane` đi thẳng vào `graphWidth()`, vốn đặt **cả** bề rộng cột CSS **và**
   * bề rộng canvas. Nên cuộn tới một vùng toàn hàng lane 0 làm cột co về đúng
   * một lane, và mọi cạnh ở lane ≥ 1 bị vẽ **ra ngoài canvas** — biến mất. Nút
   * của hàng lane 1 thì vẫn vẽ được nếu nó lọt trong bề rộng, nên kết quả đúng
   * như ảnh: chấm còn, đường mất.
   *
   * Lỗi chỉ lộ ra khi **cuộn**, vì `renderRows` là các hàng đang thấy chứ không
   * phải cả lịch sử — đó là lý do nó qua được mọi test render tĩnh.
   */
  const maxLane = useMemo(
    () =>
      renderRows.reduce((max, r) => {
        let m = Math.max(max, r.row.lane)
        for (const e of r.row.passthrough) m = Math.max(m, e.fromLane, e.toLane)
        for (const e of r.row.outEdges) m = Math.max(m, e.fromLane, e.toLane)
        return m
      }, 0),
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
        wip={wipRender}
      />
      {/*
        Hàng WIP — WORK-11, **cách B**: một phần tử ANH EM của div nội dung,
        ghim ở đầu vùng cuộn, **ngoài** div cao `getTotalSize()`.

        # 🔴 Hệ quả phải chấp nhận: hàng WIP KHÔNG cuộn đi

        `position: sticky; top: 0` giữ nó ở đỉnh khung nhìn khi danh sách cuộn
        xuống. Với WORK-11 đó là hành vi **đúng** — hàng này là chỗ vào vùng
        soạn commit nên nó phải luôn bấm được — nhưng nó KHÁC một hàng commit
        bình thường, và nó là điều duy nhất trong tính năng này mà không test
        nào chứng minh được là dễ chịu. Nó nằm trong bước 5 của checkpoint
        Task 3 để chủ dự án nói có hay không, chứ không được lặng lẽ "sửa".

        Vì sao không đưa nó vào virtualizer: xem khối comment ở `count` trên.
      */}
      {hasWip && (
        <div
          className="wip-row"
          data-testid="wip-row"
          data-wip-edge={wipCanh?.kind}
          data-wip-lane={wipCanh?.kind === 'normal' ? String(wipCanh.lane) : undefined}
          data-wip-modified={String(wipCounts.modified)}
          data-wip-added={String(wipCounts.added)}
          // Chiều cao đặt inline từ `WIP_ROW_HEIGHT` (dẫn xuất từ `ROW_HEIGHT`),
          // không từ một số 28 thứ hai — hai nguồn cho một con số là đúng lớp
          // lỗi làm cột đồ thị lệch cột văn bản ở Phase 2.
          style={{ height: WIP_ROW_HEIGHT }}
          onClick={() => onOpenCommitBox?.()}
          role="button"
          tabIndex={0}
          onKeyDown={(e) => {
            if (e.key === 'Enter' || e.key === ' ') onOpenCommitBox?.()
          }}
          title="Thay đổi chưa commit — bấm để mở vùng soạn commit"
        >
          <span className="wip-ref-cell" />
          <span className="commit-graph-gutter" style={{ width: graphColWidth }} />
          <span className="wip-subject">
            Thay đổi chưa commit
            {/* Chỉ báo suy giảm: lane của HEAD vượt tầm cột đồ thị nên KHÔNG có
                cạnh nào được vẽ. Nói ra, thay vì để một chấm trôi không cạnh
                trông như một lỗi vẽ. */}
            {wipCanh?.kind === 'clamped' && (
              <span className="wip-degraded" title={`HEAD ở lane ${wipCanh.realLane}, ngoài tầm cột đồ thị`}>
                {' '}
                (HEAD ngoài tầm)
              </span>
            )}
            {wipCanh?.kind === 'unknownHead' && (
              <span className="wip-degraded" title="HEAD không nằm trong phần lịch sử đã nạp">
                {' '}
                (chưa thấy HEAD)
              </span>
            )}
          </span>
          <span className="wip-counts">
            <span className="wip-count-modified" title={`${wipCounts.modified} tệp sửa`}>
              ✏{wipCounts.modified}
            </span>
            <span className="wip-count-added" title={`${wipCounts.added} tệp mới`}>
              +{wipCounts.added}
            </span>
          </span>
        </div>
      )}
      {/*
        Vùng nội dung lệch xuống đúng `contentOffset(hasWip)` px để nhường chỗ
        cho hàng WIP. 🔴 Đây là **phép cộng duy nhất** của cả tính năng và nó
        nằm trong `wipRow.ts` — ở đây chỉ *đọc* kết quả, không tự cộng.

        `marginTop`, KHÔNG `paddingTop`: các hàng bên trong là
        `position: absolute`, và phần tử định vị tuyệt đối neo theo **padding
        box** của khối chứa — nên `paddingTop` sẽ dịch chỗ trống mà KHÔNG dịch
        các hàng, tức cột văn bản đứng yên trong khi cột đồ thị (đã cộng
        `contentOffset` qua `commitRowY`) dịch xuống. Đó đúng là "lệch một
        hàng", lớp lỗi đã xảy ra hai lần ở Phase 2. `marginTop` dịch cả hộp nên
        hai cột đi cùng nhau.
      */}
      <div
        style={{
          height: virtualizer.getTotalSize(),
          position: 'relative',
          marginTop: contentOffset(hasWip),
        }}
      >
        {virtualItems.map((v) => {
          const commit = commits[v.index]
          const isSelected = commit !== undefined && commit.id === selectedCommitId

          /*
           * Màu nhánh của hàng, đẩy sang CSS qua biến để `.commit-row` tô một
           * dải nền rất nhạt theo nhánh.
           *
           * Vì sao có: ở 20 lane, lần theo *một* nhánh bằng cách bám sợi đường
           * trong cột đồ thị là việc khó — mắt phải giữ một đường mảnh 1,5px
           * qua hàng chục hàng. Một dải nền ám màu nhánh trải suốt bề ngang
           * hàng cho mắt một mỏ neo rộng hơn nhiều, và nó chạy sang tận cột
           * thông điệp nên đọc hàng nào cũng biết hàng đó thuộc nhánh nào. Đây
           * là thứ tham chiếu người dùng gửi làm (vùng teal/đỏ trầm trong ảnh).
           *
           * Alpha phải RẤT thấp (`app.css` dùng 7%): nền hàng nằm TRÊN canvas
           * đồ thị, nên mọi phần đục sẽ xoá đường lane phía sau — cùng lỗi mà
           * `.commit-row:hover` từng mắc với `--bg-inset` đục.
           *
           * `graphRows` là mảng thưa (`mergePage` để lỗ `undefined` ở trang
           * chưa nạp), nên phải kiểm trước khi đọc `.color`.
           */
          const graphRow = graphRows[v.index]
          const laneColor = graphRow ? colorFor(graphRow.color) : undefined
          const refs = commit ? (refsByCommit?.get(commit.id) ?? []) : []

          return (
            <div
              key={v.key}
              className={`commit-row${isSelected ? ' selected' : ''}${refs.length > 0 ? ' has-refs' : ''}`}
              style={
                {
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  right: 0,
                  transform: `translateY(${v.start}px)`,
                  height: v.size,
                  ...(laneColor ? { '--lane-color': laneColor } : {}),
                } as CSSProperties
              }
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
                    <RefBadges refs={refs} />
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
