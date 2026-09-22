/**
 * Trình xem diff — DIFF-01..04 và DIFF-06. Đây là plan mà người dùng **thấy được**.
 *
 * # Bộ dựng nằm sau một interface, và đó là BẮT BUỘC
 *
 * Tệp này nhập **đúng một** hàm dựng (`createCodeMirrorRenderer`) và không lời
 * gọi CodeMirror nào khác. Lý do là điều kiện của
 * `docs/09-phase3-diff-decision.md` mục 8: checkpoint #3 **bị bỏ qua**, nên
 * đường A (`@codemirror/merge`) được chọn **vì nó là mặc định khi thiếu bằng
 * chứng**, không vì đã chứng minh đủ nhanh. Nếu nó hoá ra chậm trên tệp 630 KB
 * thật, việc chuyển sang đường B phải là **sửa một tệp**, không phải sửa cả giao
 * diện. Tiền lệ là `interface GraphRenderer` của Phase 2.
 *
 * Đổi đường A → B: viết `hunkRenderer.ts` rồi đổi một dòng `import` dưới đây.
 *
 * # Bố cục: DiffViewer chiếm vùng `main`, **không** thêm panel thứ tư
 *
 * `<action>` của plan cho hai lựa chọn và đòi ghi lý do. Chọn **(a)**: khi có
 * tệp đang chọn, `DiffViewer` chiếm vùng `main` và đồ thị commit nhường chỗ.
 *
 * Vì sao không chọn (b) — thêm một `Panel` thứ tư, vốn an toàn hơn về hồi quy:
 * chế độ **hai cột** cần hai cột nội dung **cộng** hai cột số dòng trong một
 * panel. Thêm panel thứ tư chia bề rộng cửa sổ thành bốn phần, và
 * `<layout_constraints>` mục 4 gọi hẹp là "ca tệ nhất của chế độ hai cột". Cách
 * (a) cho diff đúng vùng rộng nhất (`main`, `defaultSize="52%"`) mà **không đổi
 * một dòng nào** trong `AppLayout.tsx` — tức không đụng bố cục Phase 2 vốn vừa
 * qua hai vòng checkpoint. Đổi lại, đồ thị bị che khi đang xem diff; nút "Đóng
 * diff" đưa nó về.
 *
 * 🔴 Đây là **thay đổi bố cục**, và Phase 2 cho thấy bố cục là chỗ hay sai nhất
 * — nên nó là **bước 1** của checkpoint Task 3 để chủ dự án xác nhận hoặc chọn
 * cách khác.
 *
 * # Điều tệp này KHÔNG kiểm được bằng test tự động
 *
 * happy-dom không tính layout CSS và không có cuộn thật. Chữ có **hiển thị** hay
 * bị co mất, hai cột có **thẳng hàng**, khoảng trắng có **hiện ra**, word-level
 * có tô **đúng chỗ** — bốn điều đó là việc của checkpoint Task 3. Ba lỗi hiển
 * thị của Phase 2 qua hết 212 test tự động; không lặp lại việc tin test thay
 * cho mắt.
 *
 * Cũng vì vậy `shouldForceUnified(width)` là một **hàm quyết định** thuần tách
 * ra: happy-dom không có `ResizeObserver` thật nên chỉ hàm đó test được.
 */

import { useCallback, useEffect, useRef, useState } from 'react'

import { describeError, ipc, type FileDiff } from '@/lib/ipc'
import { hunksToDoc, nextHunkLine, prevHunkLine, type LineMeta } from '@/lib/diff-render/decorations'
// 🔴 ĐÂY là dòng phải đổi nếu chuyển sang đường B. Không lời gọi CodeMirror nào
// khác trong tệp này — xem doc comment đầu tệp và `lib/diff-render/types.ts`.
import {
  createCodeMirrorRenderer,
  rendererStats,
} from '@/lib/diff-render/codemirrorRenderer'
import { loadLanguage } from '@/lib/diff-render/langLoader'
import type { DiffRenderer } from '@/lib/diff-render/types'
import { measureFirstPaint } from '@/lib/perf'
import { useDiffStore } from '@/stores/diffStore'
import { useSelectionStore } from '@/stores/selectionStore'
import { DiffNotice, DiffToolbar } from './DiffToolbar'

/**
 * Bề rộng tối thiểu (px) của panel để chế độ **hai cột** còn dùng được.
 *
 * 🔴 Con số này sống ở **hai** nơi: đây và `--min-split-width` trong `app.css`.
 * Lệch hai phía là **lỗi im lặng** — mỗi phía tự nó vẫn nhất quán, nhưng có một
 * dải bề rộng mà JS nói "hai cột" trong khi CSS đã co cột xuống dưới mức đọc
 * được. Có **test ghim hai phía** trong `app.css.test.ts` theo đúng tiền lệ
 * `REF_COL_WIDTH` (`geometry.ts` + `app.css`) của Phase 2.
 *
 * Vì sao 720: hai cột nội dung cần mỗi cột ~300px để đọc được một dòng mã bình
 * thường, cộng hai cột số dòng ~40px mỗi cột, cộng viền ngăn. Dưới mức đó là hai
 * dải hẹp vô dụng, và **im lặng** hiện hai cột 80px mỗi cột tệ hơn việc tự
 * chuyển về hợp nhất kèm một dòng giải thích — đây là **suy giảm có chủ ý**,
 * cùng khuôn `MAX_VISIBLE_LANES` + chỉ báo `+N cha nữa` của Phase 2.
 */
export const MIN_SPLIT_WIDTH = 720

/**
 * Có phải buộc về chế độ hợp nhất ở bề rộng này?
 *
 * Tách thành hàm thuần vì happy-dom **không** có `ResizeObserver` thật, nên đây
 * là phần duy nhất của quyết định đó mà test tự động kiểm được. Bề rộng thật là
 * việc của checkpoint Task 3 (bước 3 và 4).
 *
 * `width === 0` (chưa đo được, `ResizeObserver` chưa chạy lần đầu) → buộc hợp
 * nhất. Đoán "rộng" ở đây sẽ nháy một khung hai cột 0px rồi sửa lại.
 */
export function shouldForceUnified(width: number): boolean {
  return width < MIN_SPLIT_WIDTH
}

/** Thống kê view — **chỉ** cho test. Xem `codemirrorRenderer.rendererStats`. */
export const __diffViewerStatsForTest = rendererStats

interface Props {
  repoId: string
}

export function DiffViewer({ repoId }: Props) {
  const selectedCommitId = useSelectionStore((s) => s.selectedByRepo[repoId] ?? null)
  const selectedFile = useDiffStore((s) => s.selectedFileByRepo[repoId] ?? null)
  const viewMode = useDiffStore((s) => s.viewMode)
  const showWhitespace = useDiffStore((s) => s.showWhitespace)
  const clearFile = useDiffStore((s) => s.clearFile)

  const [diff, setDiff] = useState<FileDiff | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [hunkStatus, setHunkStatus] = useState<string | null>(null)
  const [paneWidth, setPaneWidth] = useState(0)

  const hostRef = useRef<HTMLDivElement | null>(null)
  const paneRef = useRef<HTMLDivElement | null>(null)
  const rendererRef = useRef<DiffRenderer | null>(null)
  const lineMetaRef = useRef<LineMeta[]>([])

  /*
   * Bề rộng panel đo bằng `ResizeObserver`, **không** `window.innerWidth`.
   *
   * Panel kéo được (`react-resizable-panels`), nên bề rộng cửa sổ không nói gì
   * về bề rộng panel: người dùng có thể kéo panel diff hẹp lại trong một cửa sổ
   * rộng 2560px. Đúng lớp lỗi "đo đại lượng toàn cục cho một bất biến cục bộ"
   * mà mutation #8 của plan 03-02 đã gặp ở phía Rust.
   *
   * happy-dom không có `ResizeObserver` — bọc `typeof` để test không nổ, và ghi
   * rõ hệ quả: `paneWidth` giữ 0 trong test, tức test luôn chạy ở nhánh hợp
   * nhất. Chế độ hai cột chỉ được kiểm bởi checkpoint Task 3.
   */
  useEffect(() => {
    const el = paneRef.current
    if (!el || typeof ResizeObserver === 'undefined') return

    const ro = new ResizeObserver((entries) => {
      const w = entries[0]?.contentRect.width
      if (typeof w === 'number') setPaneWidth(w)
    })
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  const forcedUnified = viewMode === 'split' && shouldForceUnified(paneWidth)
  const hieuLuc = forcedUnified ? 'unified' : viewMode

  /*
   * Nạp diff.
   *
   * 🔴 **Cổng chống đua (T-03-31).** `requestId` tăng mỗi lần effect chạy, và
   * phản hồi chỉ được nhận khi `requestId` của nó vẫn là lần mới nhất. Không có
   * cổng này, chọn tệp A rồi ngay tệp B với phản hồi A về **sau** B làm giao
   * diện hiện nội dung của **tệp khác** mà **không lỗi nào** — cực kỳ thường
   * gặp khi bấm nhanh bằng bàn phím, và `cancelled = true` kiểu cũ của
   * `CommitDetail` không đủ vì nó chỉ chặn effect **đã bị dọn**, không chặn hai
   * lời gọi song song về sai thứ tự.
   */
  const requestIdRef = useRef(0)

  useEffect(() => {
    if (!selectedCommitId || !selectedFile) {
      setDiff(null)
      setError(null)
      setIsLoading(false)
      return
    }

    const id = ++requestIdRef.current
    setIsLoading(true)
    setError(null)
    setHunkStatus(null)

    // Nhãn **tĩnh** (T-03-30 / khuôn T-02-24): không đường dẫn tệp nào vào
    // console qua dụng cụ đo.
    const xong = measureFirstPaint('diff-first-paint')

    ipc
      .getFileDiff(repoId, selectedCommitId, selectedFile)
      .then((result) => {
        if (id !== requestIdRef.current) return
        setDiff(result)
        xong()
      })
      .catch((e) => {
        if (id !== requestIdRef.current) return
        setError(describeError(e))
      })
      .finally(() => {
        if (id !== requestIdRef.current) return
        setIsLoading(false)
      })
  }, [repoId, selectedCommitId, selectedFile])

  const kindText = diff?.kind.kind === 'text' ? diff.kind : null

  /*
   * Dựng / tháo trình xem.
   *
   * Phụ thuộc **không** gồm `showWhitespace`: cờ đó đổi qua `Compartment` ở
   * effect riêng bên dưới. Đưa nó vào đây làm view bị dựng lại và **mất vị trí
   * cuộn** — mutation #5 của plan là đúng việc đó, và bước 8 của checkpoint hỏi
   * đúng câu "vị trí cuộn có giữ nguyên".
   *
   * `hieuLuc` (chế độ *thật sự* đang dùng) **có** trong phụ thuộc: hai chế độ là
   * hai cây DOM khác hẳn nên đổi chế độ *phải* dựng lại.
   */
  useEffect(() => {
    const host = hostRef.current
    if (!host || !kindText || !selectedFile) return

    const newDoc = hunksToDoc(kindText.hunks, 'new')
    const oldDoc = hunksToDoc(kindText.hunks, 'old')
    lineMetaRef.current = newDoc.lineMeta

    const renderer = createCodeMirrorRenderer({
      parent: host,
      newDoc,
      oldDoc,
      language: null,
      showWhitespace: useDiffStore.getState().showWhitespace,
      mode: hieuLuc,
    })
    rendererRef.current = renderer

    // Tô màu cú pháp (DIFF-01) nạp **lười** theo phần mở rộng, rồi gắn vào qua
    // `Compartment` — nên đổi tệp không phải dựng lại view, và không gói
    // `lang-*` nào vào bundle khởi động.
    let huy = false
    void loadLanguage(selectedFile).then((lang) => {
      if (huy) return
      renderer.setLanguage(lang)
    })

    return () => {
      huy = true
      // 🔴 Thiếu dòng này thì mỗi lần chọn tệp rò một `EditorView`, và người
      // dùng bấm qua hàng chục tệp trong một phiên (T-03-28).
      renderer.destroy()
      rendererRef.current = null
      lineMetaRef.current = []
    }
  }, [kindText, selectedFile, hieuLuc])

  /*
   * Cờ khoảng trắng (DIFF-04) đi qua `Compartment` — **không** dựng lại view.
   *
   * Effect riêng, phụ thuộc chỉ `showWhitespace`: đó là điều làm nó không chạm
   * tới vòng dựng/tháo ở trên.
   */
  useEffect(() => {
    rendererRef.current?.setShowWhitespace(showWhitespace)
  }, [showWhitespace, kindText, hieuLuc])

  /*
   * Nhảy khối (DIFF-03). `null` từ `nextHunkLine`/`prevHunkLine` nghĩa là **hết
   * khối**, và giao diện **nói ra** — không nhảy vòng im lặng về đầu tệp.
   */
  const nhayKhoi = useCallback((huong: 'next' | 'prev') => {
    const renderer = rendererRef.current
    const meta = lineMetaRef.current
    if (!renderer || meta.length === 0) return

    const hienTai = renderer.currentLine()
    const dich = huong === 'next' ? nextHunkLine(meta, hienTai) : prevHunkLine(meta, hienTai)

    if (dich === null) {
      setHunkStatus(
        huong === 'next'
          ? 'Đây là khối cuối — không nhảy vòng về đầu tệp.'
          : 'Đây là khối đầu — không nhảy vòng về cuối tệp.',
      )
      return
    }
    setHunkStatus(null)
    renderer.scrollToLine(dich)
  }, [])

  const onNextHunk = useCallback(() => nhayKhoi('next'), [nhayKhoi])
  const onPrevHunk = useCallback(() => nhayKhoi('prev'), [nhayKhoi])

  if (!selectedCommitId || !selectedFile) {
    return (
      <div className="diff-viewer diff-viewer-empty" data-testid="diff-empty">
        <p className="placeholder">Bấm một tệp trong danh sách thay đổi để xem diff.</p>
      </div>
    )
  }

  return (
    <div className="diff-viewer" ref={paneRef}>
      <div className="diff-header">
        <span className="diff-path" data-testid="diff-path">
          {diff?.oldPath ? `${diff.oldPath} → ${selectedFile}` : selectedFile}
        </span>
        <DiffToolbar onNextHunk={onNextHunk} onPrevHunk={onPrevHunk} />
        <button className="diff-close" onClick={() => clearFile(repoId)}>
          Đóng diff
        </button>
      </div>

      {forcedUnified && (
        <div className="diff-notice diff-notice-degraded" data-testid="diff-forced-unified" role="status">
          Panel hẹp hơn {MIN_SPLIT_WIDTH}px nên đang hiện <strong>hợp nhất</strong>. Kéo rộng
          panel để về hai cột.
        </div>
      )}

      {hunkStatus && (
        <div className="diff-notice diff-notice-hunk" data-testid="diff-hunk-status" role="status">
          {hunkStatus}
        </div>
      )}

      {error ? (
        <div className="diff-notice diff-notice-error" data-testid="diff-error" role="alert">
          <pre>{error}</pre>
        </div>
      ) : isLoading ? (
        <div className="diff-pane diff-loading" data-testid="diff-loading">
          <p className="placeholder">Đang đọc diff…</p>
        </div>
      ) : diff ? (
        <>
          {/* Bốn dạng không phải `text` render thông báo và **không** dựng
              `EditorView` — nội dung của chúng không bao giờ được đổ ra màn
              hình (T-03-29). `DiffNotice` trả `null` cho `text` không bị cắt. */}
          <DiffNotice kind={diff.kind} />
          {kindText && (
            <div className={`diff-pane${hieuLuc === 'split' ? ' diff-split' : ''}`}>
              <div className="diff-host" ref={hostRef} />
            </div>
          )}
        </>
      ) : null}
    </div>
  )
}
