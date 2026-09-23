/**
 * Bảng khối — **đường vào thứ hai** của staging theo khối. WORK-03, WORK-05, WORK-06.
 *
 * # 🔴 Đây là mệnh đề mà wave 4 để ngỏ, và nó là cả lý do plan này tồn tại
 *
 * `CONTEXT.md` mục 0, hệ quả 2, nguyên văn:
 *
 * > *"Tầng giao diện của Phase 5 phải chịu được việc Phase 4 còn lỗi. Nếu `ChangeList`
 * > hoá ra hiển thị sai, staging theo khối vẫn phải dùng được từ một đường vào khác."*
 *
 * 05-04-SUMMARY đóng được mệnh đề **một** và ghi thẳng rằng mệnh đề **hai** còn để ngỏ:
 *
 * | Mệnh đề | Trạng thái sau wave 4 |
 * |---|---|
 * | `HunkBar` **có thể** hoạt động không cần Phase 4 | ✅ ghim bằng M25 + M25b |
 * | Người dùng **có** đường tới nó không qua Phase 4 | ❌ **chưa có đường nào cả** |
 *
 * và cảnh báo:
 *
 * > *"nếu đường vào duy nhất của staging theo khối là 'mở diff từ `ChangeList`', thì
 * > toàn bộ công của wave này **không mua được gì** — component sạch mà đường tới nó
 * > thì không."*
 *
 * Tệp này là đường đó. Cách ràng buộc được **cài**, chứ không chỉ được hứa:
 *
 * * **Không** nhập một module Phase 3 nào: không `codemirrorRenderer`, không
 *   `hunksToDoc`, không `DiffViewer`, không `@codemirror/*`. Nó đọc thẳng
 *   `FileDiff.kind.hunks` và render HTML thường.
 * * **Không** nhập một component Phase 4 nào: không `ChangeList`, không `CommitBox`.
 * * **Không** đọc `useDiffStore`. Nó tự giữ `diff` trong state cục bộ — nên không có
 *   "store Phase 3 chưa nạp" nào để kẹt vào. Đây chính là đột biến **M28**.
 * * **Không** `disabled` nào phụ thuộc dữ liệu ngoài. Có test khẳng định điều đó
 *   (khuôn M25b của wave 4: "vẫn render" là chưa đủ, phải "vẫn bấm được").
 * * Nó **tự nạp** diff của mình, nên nó không chờ ai mount trước.
 *
 * # 🔴 Phát hiện của plan này: `DiffViewer` là diff của COMMIT, không phải worktree
 *
 * Plan 05-05 viết *"nối `HunkBar` vào `DiffViewer`: một `HunkBar` cho mỗi phần tử của
 * `kindText.hunks`"*. Đo được thì việc đó **sai**:
 *
 * `DiffViewer.tsx` nạp bằng `ipc.getFileDiff(repoId, selectedCommitId, selectedFile)`
 * — diff của một commit **đã có trong lịch sử**, và nó `return` sớm khi không có
 * `selectedCommitId`. Gắn nút "Đưa khối vào vùng chờ" lên đó là gắn một thao tác ghi
 * lên một thứ **không có gì để ghi**: khối của một commit cũ đã ở trong lịch sử rồi.
 *
 * Staging theo khối cần `ipc.getWorktreeDiff(repoId, path, staged)`. Và grep toàn bộ
 * `src/**` trừ tệp test cho thấy hàm đó **không có chỗ gọi nào** trước plan này —
 * `ChangeList` chỉ ghi lựa chọn vào `diffStore`, và có test ghim rằng nó **không** tự
 * gọi `getWorktreeDiff`. Tức trước plan này ứng dụng **không có** màn hình nào xem
 * diff thư mục làm việc.
 *
 * Nên bảng này không phải "đường dự phòng cho một đường chính đã chạy". Nó là đường
 * **duy nhất** tới staging theo khối, và nó độc lập Phase 3 + Phase 4 theo thiết kế.
 * Xem SUMMARY, mục "điều plan nói mà hoá ra sai".
 *
 * # `blobHash`
 *
 * 🔴 Giao diện **không** phải chỗ tin cậy cho phép kiểm "tệp có đổi không" (T-05-15).
 * Phía Rust tự đọc lại mã băm ngay trước khi áp và ném `file_changed` khi lệch. Giá
 * trị truyền xuống đây là *"thứ giao diện đã thấy"*, và `''` là một giá trị **hợp lệ**
 * mang nghĩa "giao diện không biết" — phía Rust vẫn kiểm.
 *
 * `getWorktreeDiff` **không** trả mã băm blob, nên hôm nay nó luôn là `''`. Đây là
 * **nợ đã biết**, ghi ở SUMMARY: nó làm WORK-05 mất lớp cảnh báo *sớm* (giao diện
 * không phát hiện được lệch trước khi gọi), nhưng **không** làm mất lớp bảo vệ — bản
 * vá không khớp vẫn bị `git apply --check` từ chối, và thông báo "hãy làm mới" vẫn
 * hiện. Lấp được bằng cách thêm một trường vào `FileDiff` của đường worktree.
 *
 * # ⚠️ Bố cục CHƯA KIỂM CHỨNG
 *
 * happy-dom không tính CSS layout và không có cuộn thật (`CONTEXT.md` 4.5). "Khối
 * đang chọn nhìn ra khác khối không chọn", "thanh nút không đè lên nội dung khối",
 * "bảng không tràn" là **có mã, chưa kiểm** cho tới khi có người xem trên Chromium
 * thật — checkpoint 05-05. Khuôn doc comment chép từ `ChangeList.tsx`.
 */

import { useCallback, useEffect, useRef, useState } from 'react'

import { describeError, ipc, type FileDiff } from '@/lib/ipc'
import { khoaKhoi, useHunkStore } from '@/stores/hunkStore'
import { HunkBar } from './HunkBar'

interface Props {
  repoId: string
  path: string
  /** `true` → `git diff --cached` ("thứ sẽ vào commit tới"); `false` → chưa stage. */
  staged: boolean
  /** Quyết định động từ huỷ: `true` → "Xoá", `false` → "Huỷ bỏ". Xem `nhanDongTu`. */
  untracked: boolean
}

/** Một dòng diff rút gọn cho bảng — tối đa `SO_DONG_XEM` dòng mỗi khối. */
const SO_DONG_XEM = 6

export function HunkTable({ repoId, path, staged, untracked }: Props) {
  const [diff, setDiff] = useState<FileDiff | null>(null)
  const [loi, setLoi] = useState<string | null>(null)
  const [dangDoc, setDangDoc] = useState(false)

  const dangChon = useHunkStore((s) => s.dangChon)
  const chon = useHunkStore((s) => s.chon)

  /*
   * 🔴 Cổng chống đua — khuôn T-03-31 của `DiffViewer`, và cùng lý do.
   *
   * `requestId` tăng mỗi lần nạp, và phản hồi chỉ được nhận khi nó vẫn là lần mới
   * nhất. Không có cổng này, đổi tệp A → B với phản hồi A về **sau** B làm bảng hiện
   * khối của **tệp khác** mà không lỗi nào. Trên đường chỉ-xem đó là khó chịu; ở đây
   * nó nghĩa là người dùng bấm "Đưa khối vào vùng chờ" trên một khối **của tệp
   * khác** — `path` truyền xuống IPC là prop hiện tại, nên git sẽ áp chỉ số khối của
   * tệp B lên tệp A. Một lỗi ghi, không phải một lỗi hiển thị.
   */
  const lanNap = useRef(0)

  const nap = useCallback(async () => {
    const id = ++lanNap.current
    setDangDoc(true)
    try {
      const kq = await ipc.getWorktreeDiff(repoId, path, staged)
      if (id !== lanNap.current) return
      setDiff(kq)
      setLoi(null)
    } catch (e) {
      if (id !== lanNap.current) return
      setLoi(describeError(e))
    } finally {
      if (id === lanNap.current) setDangDoc(false)
    }
  }, [repoId, path, staged])

  useEffect(() => {
    void nap()
  }, [nap])

  if (loi !== null) {
    return (
      <div className="hunk-table" data-testid="hunk-table">
        {/*
          🔴 `role="alert"` và phần tử **riêng** cho trạng thái lỗi — khuôn M26/M26b
          của wave 4. Gắn `role` lên vỏ `hunk-table` (có mặt ở mọi trạng thái) là lỗi
          #6, và có test phủ định ghim.
        */}
        <div className="hunk-table-error" data-testid="hunk-table-error" role="alert">
          {loi}
        </div>
        <button type="button" className="hunk-table-retry" onClick={() => void nap()}>
          Thử lại
        </button>
      </div>
    )
  }

  if (diff === null) {
    return (
      <div className="hunk-table" data-testid="hunk-table">
        <p className="placeholder" data-testid="hunk-table-loading">
          Đang đọc khác biệt…
        </p>
      </div>
    )
  }

  const kind = diff.kind

  if (kind.kind !== 'text') {
    /*
     * Bốn dạng không phải `text` **không có khối** để stage. Nói ra lý do cụ thể chứ
     * không một câu chung: DIFF-06 của Phase 3 đã học rằng "không xem được" là bốn
     * tình huống khác nhau với bốn quyết định khác nhau. Ở đây quyết định của người
     * dùng cũng khác: tệp nhị phân thì họ stage cả tệp (R7 — từ chối rõ ràng, không
     * đoán); tệp không đổi thì không có gì để làm.
     */
    const cau =
      kind.kind === 'binary'
        ? 'Tệp nhị phân — không có khối thay đổi theo dòng. Hãy stage cả tệp.'
        : kind.kind === 'tooLarge'
          ? 'Tệp vượt ngưỡng xem khác biệt — không dựng được bảng khối. Hãy stage cả tệp.'
          : kind.kind === 'lfsPointer'
            ? 'Con trỏ Git LFS — nội dung thật nằm ngoài repo. Hãy stage cả tệp.'
            : 'Tệp không thay đổi nội dung (có thể chỉ đổi quyền truy cập tệp).'

    return (
      <div className="hunk-table" data-testid="hunk-table">
        <p className="hunk-table-notice" data-testid="hunk-table-khong-text">
          {cau}
        </p>
      </div>
    )
  }

  const hunks = kind.hunks

  return (
    <div className="hunk-table" data-testid="hunk-table">
      <div className="hunk-table-head">
        <span className="hunk-table-path">{path}</span>
        <span className="hunk-table-count">
          {hunks.length} khối · {staged ? 'đã stage' : 'chưa stage'}
        </span>
        <button
          type="button"
          className="hunk-table-retry"
          data-testid="hunk-table-lam-moi"
          disabled={dangDoc}
          onClick={() => void nap()}
        >
          Làm mới
        </button>
      </div>

      {kind.truncated && (
        /*
         * 🔴 `truncated` **phải** hiện ra, và trên đường này nó quan trọng hơn trên
         * đường chỉ-xem. Người dùng không thấy hết khối, nên họ sẽ tin mình đã stage
         * mọi thứ cần stage. `DiffToolbar` của Phase 3 đã có cổng cho đúng cờ này;
         * đường thứ hai không được im lặng bỏ qua nó.
         */
        <div
          className="hunk-table-notice hunk-table-notice-warn"
          data-testid="hunk-table-truncated"
          role="status"
        >
          ⚠️ Danh sách khối đã bị cắt — bạn đang xem <strong>một phần</strong> các thay
          đổi của tệp này.
        </div>
      )}

      {kind.contextOnly && (
        /*
         * KHÔNG dùng `⚠️` và KHÔNG dùng chữ "bị cắt" — khuôn `DiffNotice` của Phase 3,
         * và cùng lý do: người dùng **không mất khối nào** ở đây. Một cảnh báo kiểu
         * "bị cắt" cho một tệp lớn bình thường làm họ mất tin vào công cụ đúng lúc
         * họ sắp bấm một nút ghi.
         */
        <div className="hunk-table-notice" data-testid="hunk-table-context-only" role="status">
          Tệp lớn nên mỗi khối hiện kèm 3 dòng ngữ cảnh thay vì toàn tệp. Mọi khối thay
          đổi đều có mặt.
        </div>
      )}

      {hunks.length === 0 ? (
        <p className="hunk-table-notice" data-testid="hunk-table-rong">
          Không có khối thay đổi nào ở {staged ? 'vùng chờ' : 'thư mục làm việc'} cho tệp
          này.
        </p>
      ) : (
        <ol className="hunk-table-rows">
          {hunks.map((h, i) => {
            const khoa = khoaKhoi(path, i)
            const daChon = dangChon === khoa
            return (
              <li
                // 🔴 Khoá gồm **chỉ số**, không phải `heading`: hai khối trong cùng
                // một hàm có **cùng** heading (git in tên hàm chứa khối), nên
                // `heading` một mình không phải khoá duy nhất. Đúng lớp lỗi mà
                // `ChangeList` gặp với tệp `MM` xuất hiện ở hai nhóm.
                key={khoa}
                className={`hunk-table-row${daChon ? ' hunk-table-row-selected' : ''}`}
                data-testid="hunk-table-row"
                data-index={i}
                onClick={() => chon(path, i)}
              >
                <div className="hunk-table-row-head">
                  <span className="hunk-table-row-range">
                    {/* Khoảng dòng phía **mới** — thứ người dùng đang nhìn trong
                        trình soạn thảo của họ. Phía cũ chỉ có nghĩa khi so lịch sử. */}
                    Khối {i + 1} · dòng {h.newStart}–{h.newStart + Math.max(h.newCount - 1, 0)}
                  </span>
                  {h.heading !== '' && (
                    <span className="hunk-table-row-heading">{h.heading}</span>
                  )}
                </div>

                {/*
                  Xem trước vài dòng — **không** phải một trình xem diff.
                  Cố tình thô: mục đích của bảng này là còn dùng được khi trình xem
                  diff hỏng, nên nó không được mượn một mẩu nào của trình xem đó.
                */}
                <pre className="hunk-table-preview">
                  {h.lines.slice(0, SO_DONG_XEM).map((d, j) => (
                    <span key={j} className={`hunk-line hunk-line-${d.kind}`}>
                      {d.kind === 'added' ? '+' : d.kind === 'removed' ? '-' : ' '}
                      {d.content}
                      {'\n'}
                    </span>
                  ))}
                  {h.lines.length > SO_DONG_XEM && (
                    <span className="hunk-line hunk-line-more">
                      … còn {h.lines.length - SO_DONG_XEM} dòng nữa{'\n'}
                    </span>
                  )}
                </pre>

                <HunkBar
                  repoId={repoId}
                  path={path}
                  // 🔴 Chỉ số **thật** trong mảng, không `0`. Đột biến M29, và đột
                  // biến M23 của wave 4 đã đo đỏ cho cùng lỗi ở tầng `HunkBar`.
                  index={i}
                  // Xem doc comment đầu tệp: `''` = "giao diện không biết", và phía
                  // Rust vẫn tự kiểm. Nợ đã biết, ghi ở SUMMARY.
                  blobHash=""
                  staged={staged}
                  untracked={untracked}
                  // 🔴 Đây là chỗ `onLamMoi` của wave 4 CUỐI CÙNG được truyền.
                  //
                  // Wave 4 thêm prop này và ghi rõ nó "chưa được truyền ở đâu cả".
                  // Không truyền nó, nút "Làm mới" trên băng `file_changed` **nói
                  // dối**: băng biến mất, diff vẫn cũ, và lần bấm kế tiếp lại thất
                  // bại. Có test ghim rằng bấm nó sinh lời gọi `getWorktreeDiff`
                  // **thứ hai**.
                  onLamMoi={() => void nap()}
                />
              </li>
            )
          })}
        </ol>
      )}
    </div>
  )
}
