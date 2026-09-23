/**
 * Vùng "Thay đổi chưa commit" — **đường vào thứ hai** của staging theo khối.
 *
 * Danh sách tệp riêng của Phase 5, bảng khối (`HunkTable`), và danh sách "Vừa huỷ gần
 * đây" (`TrashList`), trên **một** màn hình.
 *
 * # 🔴 Đây là mệnh đề hai của ràng buộc chống-Phase-4, và nó là cả lý do plan tồn tại
 *
 * `CONTEXT.md` mục 0, hệ quả 2: *"Nếu `ChangeList` hoá ra hiển thị sai, staging theo
 * khối vẫn phải dùng được từ một đường vào khác."*
 *
 * 05-04-SUMMARY đóng được *"`HunkBar` **có thể** sống không cần Phase 4"* và ghi thẳng
 * rằng *"người dùng **có** đường tới nó"* thì **chưa**, kèm cảnh báo:
 *
 * > *"nếu đường vào duy nhất của staging theo khối là 'mở diff từ `ChangeList`', thì
 * > toàn bộ công của wave này **không mua được gì**."*
 *
 * ## Ba câu hỏi vòng khoá chết, trả lời cho MÃ NÀY
 *
 * Wave 5 Phase 4 tìm được vòng **bốn điều kiện**, và mọi component trong vòng đó đều
 * có test xanh — lỗi nằm ở **quan hệ giữa** chúng. Nên ba câu hỏi được trả lời bằng
 * mã, và có cổng ở `WorktreePane.test.tsx` cho **từng** câu:
 *
 * **1. Cái gì mount nó?** `App.tsx` render nó bất cứ khi nào có repo đang mở và người
 * dùng đã chọn thẻ "Thay đổi". Không có điều kiện "dữ liệu đã có" nào ở phía trước.
 * So với Phase 4: ở đó `ChangeList` cần vùng soạn **đã mở**, vùng soạn cần một cú bấm
 * vào hàng WIP, và hàng WIP cần dữ liệu — ba điều kiện nối tiếp. Ở đây: **một**, và
 * nó là "có repo".
 *
 * **2. Nó có phụ thuộc thứ mà chính nó tạo ra không?** **Không.** Nó **tự gọi**
 * `getStatus` trong `useEffect` lúc mount và render danh sách **từ kết quả đó**. Vòng
 * Phase 4 có đúng hình dạng ngược lại: `ChangeList` vừa là chỗ **duy nhất** gọi
 * `refresh` **vừa** chỉ mount khi kết quả của `refresh` đã có. Ở đây lời gọi nạp đứng
 * **trước** mọi điều kiện hiển thị, không sau.
 *
 * **3. Có đường nào tới nó không đi qua một component Phase 4 không?** **Có, và đó là
 * đường duy nhất.** `WorktreePane` không nhập `ChangeList`, không nhập `CommitBox`,
 * không đọc `commitStore`, và không liên quan tới hàng WIP của `CommitList`. Nó cũng
 * **không** nhập `DiffViewer` hay bất kỳ module Phase 3 nào — `HunkTable` dựng bảng
 * từ `FileDiff.kind.hunks` bằng HTML thường.
 *
 * 🔴 Câu trả lời đầy đủ, kể cả phần không thoải mái, ở `05-05-SUMMARY.md`.
 *
 * # 🔴 Vì sao danh sách tệp ở đây KHÔNG dùng lại `ChangeList`
 *
 * Dùng lại nó sẽ dựng **đúng** phụ thuộc mà cả phase này tồn tại để cấm: một lỗi
 * hiển thị của `ChangeList` — thứ **chưa ai bấm thử**, và Phase 3 có **năm** lỗi loại
 * đó lọt qua hàng trăm test — sẽ kéo theo cả đường staging theo khối.
 *
 * Cái giá là **hai** danh sách tệp trong ứng dụng, và đó là một cái giá thật, không
 * phải một chi tiết. Nó được ghi vào SUMMARY như **nợ có chủ ý**: khi Phase 4 đóng
 * được hai cổng của nó và `ChangeList` đã có bằng chứng từ mắt người, hai danh sách
 * nên gộp lại. Gộp **trước** khi có bằng chứng đó là đánh đổi sai chiều.
 *
 * # ⚠️ Bố cục CHƯA KIỂM CHỨNG
 *
 * happy-dom không tính CSS layout và không có cuộn thật (`CONTEXT.md` 4.5). "Danh sách
 * tệp và bảng khối không đè nhau", "bảng khối cuộn được khi tệp có nhiều khối",
 * "`TrashList` không bị đẩy khuất" là **có mã, chưa kiểm** cho tới khi có người xem
 * trên Chromium thật — checkpoint 05-05. Khuôn doc comment chép từ `ChangeList.tsx`.
 */

import { useCallback, useEffect, useState } from 'react'

import { describeError, ipc, type StatusEntry } from '@/lib/ipc'
import { useHunkStore } from '@/stores/hunkStore'
import { useStatusStore } from '@/stores/statusStore'
import { HunkTable } from './HunkTable'
import { TrashList } from './TrashList'

interface Props {
  repoId?: string
}

/** Thứ tự ba nhóm, và tiêu đề. Cùng thứ tự `ChangeList` để hai chỗ không nói ngược nhau. */
const NHOM: { group: StatusEntry['group']; tieuDe: string }[] = [
  { group: 'staged', tieuDe: 'Đã stage' },
  { group: 'unstaged', tieuDe: 'Chưa stage' },
  { group: 'untracked', tieuDe: 'Chưa theo dõi' },
]

export function WorktreePane({ repoId }: Props) {
  /**
   * Tệp đang chọn ở **pane này**, state cục bộ.
   *
   * 🔴 **Không** dùng `useDiffStore.selectedFileByRepo`. Đó là store của Phase 3, và
   * đọc nó ở đây nối `WorktreePane` vào vòng đời `DiffViewer`: chọn một tệp ở đây sẽ
   * mở trình xem diff Phase 3 ở vùng `main` (xem `App.tsx`), tức một lỗi render của
   * Phase 3 làm **cả màn hình** sập trong khi người dùng đang staging theo khối.
   *
   * `group` đi cùng `path` chứ không tra lại: một tệp `MM` xuất hiện ở **hai** nhóm
   * với cùng `path`, nên `path` một mình không xác định được `staged`. Tra lại bằng
   * `find(e => e.path === path)` sẽ luôn trả nhóm **đầu tiên** — tức bấm hàng "Chưa
   * stage" của một tệp `MM` lại nạp diff `--cached`.
   */
  const [dangXem, setDangXem] = useState<{ path: string; group: StatusEntry['group'] } | null>(
    null,
  )
  const [loi, setLoi] = useState<string | null>(null)

  const lat = useStatusStore((s) => (repoId ? s.byRepo[repoId] : undefined))
  const boChon = useHunkStore((s) => s.boChon)
  const xoaCanLamMoi = useHunkStore((s) => s.xoaCanLamMoi)

  /*
   * 🔴 Tự nạp — đây là câu trả lời cho câu hỏi vòng khoá số 2.
   *
   * `getStatus` gọi **thẳng** rồi ghi vào store qua `applyExternal`, không qua
   * `statusStore.refresh`. Lý do là ràng buộc của phase này chứ không phải sở thích:
   * `refresh` là API mà `ChangeList` sở hữu và gộp lời gọi theo `repoId` (`dangBay`).
   * Đi qua nó nghĩa là đường vào thứ hai chia một cơ chế gộp với đường thứ nhất, và
   * một lỗi ở cơ chế đó dừng cả hai — tức hai đường vào chỉ còn là một.
   *
   * `applyExternal` là điểm vào đúng: nó nhận một `RepoStatus` **đã có sẵn** và không
   * tự gọi IPC (ràng buộc 2.5, khuôn `hunkStore.ghiKhoi`).
   */
  const nap = useCallback(async (id: string) => {
    try {
      const st = await ipc.getStatus(id)
      useStatusStore.getState().applyExternal(id, st)
      setLoi(null)
    } catch (e) {
      setLoi(describeError(e))
    }
  }, [])

  useEffect(() => {
    if (repoId) void nap(repoId)
  }, [repoId, nap])

  /*
   * Đổi repo → bỏ chọn. Giữ nguyên `dangXem` qua một lần đổi repo làm bảng khối nạp
   * `getWorktreeDiff` với một đường dẫn **của repo cũ** trong repo **mới** — git sẽ
   * trả lỗi "không có tệp đó", nhưng ở nhánh may mắn hơn (repo mới có tệp cùng tên)
   * nó trả diff của một tệp **khác** mà không lỗi nào. Khuôn `commitStore.switchRepo`.
   */
  useEffect(() => {
    setDangXem(null)
    boChon()
    xoaCanLamMoi()
  }, [repoId, boChon, xoaCanLamMoi])

  if (!repoId) return null

  const status = lat?.status
  const entries = status?.entries ?? []

  return (
    <div className="worktree-pane" data-testid="worktree-pane">
      {(loi ?? lat?.error) && (
        // 🔴 Phần tử **chỉ** tồn tại ở trạng thái lỗi — khuôn M26b của wave 4. Gắn
        // `role="alert"` lên vỏ pane (có mặt ở mọi trạng thái) là lỗi #6.
        <div className="worktree-error" data-testid="worktree-error" role="alert">
          {loi ?? lat?.error}
        </div>
      )}

      <div className="worktree-cols">
        <div className="worktree-files">
          <div className="worktree-files-head">
            <h3>Tệp thay đổi</h3>
            <button
              type="button"
              className="worktree-refresh"
              data-testid="worktree-lam-moi"
              onClick={() => void nap(repoId)}
            >
              Làm mới
            </button>
          </div>

          {status === undefined ? (
            <p className="placeholder" data-testid="worktree-loading">
              Đang đọc trạng thái…
            </p>
          ) : entries.length === 0 ? (
            <p className="placeholder" data-testid="worktree-empty">
              Không có thay đổi nào.
            </p>
          ) : (
            NHOM.map(({ group, tieuDe }) => {
              const cua_nhom = entries.filter((e) => e.group === group)
              // Nhóm rỗng KHÔNG hiện tiêu đề — khuôn `ChangeList`, đột biến M8.
              if (cua_nhom.length === 0) return null

              return (
                <section key={group} className="worktree-group">
                  <h4 className="worktree-group-heading">
                    {tieuDe} <span className="worktree-group-count">({cua_nhom.length})</span>
                  </h4>
                  {cua_nhom.map((e) => {
                    const daChon = dangXem?.path === e.path && dangXem.group === e.group
                    return (
                      <button
                        // Khoá gồm **cả** nhóm: tệp `MM` có mặt ở hai nhóm với cùng
                        // `path`. Khuôn `ChangeList`.
                        key={`${group}:${e.path}`}
                        type="button"
                        className={`worktree-file${daChon ? ' worktree-file-selected' : ''}`}
                        data-testid="worktree-file"
                        data-path={e.path}
                        data-group={e.group}
                        // 🔴 `disabled` **không** phụ thuộc dữ liệu ngoài nào. Một nút
                        // vô hiệu "cho an toàn tới khi biết X" là đúng đột biến M25b
                        // của wave 4 — nó render nhưng không dùng được, và đó là một
                        // đường vào chết trá hình.
                        onClick={() => {
                          setDangXem({ path: e.path, group: e.group })
                          // Băng "hãy làm mới" thuộc về **tệp cũ**. Giữ nó khi đổi
                          // tệp làm người dùng thấy cảnh báo của tệp A trên khối của
                          // tệp B — `HunkBar` đã lọc theo `path`, nhưng xoá ở đây làm
                          // ý định tường minh thay vì dựa vào phép lọc đó.
                          xoaCanLamMoi()
                          boChon()
                        }}
                      >
                        <span className="worktree-file-path">{e.path}</span>
                        {e.hasInvalidUtf8 && (
                          /*
                           * 🔴 `hasInvalidUtf8` — `StatusEntry` nói rõ: đường dẫn này
                           * đã mất byte gốc khi giải mã lossy, và **không** dùng làm
                           * đối số git được. Nên nói ra thay vì để người dùng bấm rồi
                           * nhận một lỗi "không có tệp đó" không hiểu nổi.
                           */
                          <span className="worktree-file-warn" title="Đường dẫn có byte không phải UTF-8">
                            ⚠
                          </span>
                        )}
                      </button>
                    )
                  })}
                </section>
              )
            })
          )}

          <TrashList repoId={repoId} />
        </div>

        <div className="worktree-hunks">
          {dangXem === null ? (
            <p className="placeholder" data-testid="worktree-chua-chon">
              Bấm một tệp bên trái để xem và chọn từng khối thay đổi.
            </p>
          ) : (
            <HunkTable
              // 🔴 `key` gồm cả đường dẫn và nhóm: đổi tệp phải **dựng lại** bảng, không
              // tái dùng state cũ. Không có nó, `diff` của tệp trước còn nằm trong state
              // trong khi `path` đã đổi — tức một cú bấm "Đưa khối vào vùng chờ" gửi chỉ
              // số khối của tệp A kèm đường dẫn của tệp B xuống git. Một lỗi **ghi**.
              key={`${dangXem.group}:${dangXem.path}`}
              repoId={repoId}
              path={dangXem.path}
              staged={dangXem.group === 'staged'}
              // 🔴 Chỗ QUYẾT ĐỊNH động từ huỷ. ROADMAP không thương lượng: tệp chưa
              // theo dõi → "Xoá". `HunkBar` có cổng hai chiều (M21 + M22 của wave 4)
              // cho việc *hiển thị* nó đúng; cổng cho việc *truyền* đúng nằm ở
              // `WorktreePane.test.tsx`, vì một `untracked={false}` cứng ở đây làm cả
              // hai cổng kia vô dụng mà không test nào của wave 4 thấy.
              untracked={dangXem.group === 'untracked'}
            />
          )}
        </div>
      </div>
    </div>
  )
}
