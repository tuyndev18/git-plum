/**
 * Ba nhóm tệp của thư mục làm việc — WORK-01, WORK-02.
 *
 * Bấm một hàng **mở diff của tệp đó**; bấm nút stage/unstage của hàng **chuyển nhóm
 * tệp đó ngay**, không cần bấm làm mới và không chờ watcher.
 *
 * # 🔴 Ràng buộc R8: component này KHÔNG được phụ thuộc vào việc diff render đúng
 *
 * `CONTEXT.md` mục 0, nguyên văn hệ quả:
 *
 * > WORK-01 đọc trực tiếp trình xem diff của Phase 3, vốn **chưa ai dùng thật**. Chủ
 * > dự án đã tìm **năm** lỗi hiển thị của trình xem đó chỉ bằng cách mở ứng dụng, và
 * > cả năm đều đi qua 435 test tự động. […] Chọn một tệp mà diff hỏng thì vẫn phải
 * > stage/commit được tệp đó. Nếu một plan làm vòng commit chết theo trình xem diff,
 * > plan đó sai.
 *
 * Cách ràng buộc đó được **cài** ở đây, chứ không chỉ được hứa:
 *
 * * Nút stage/unstage **không bao giờ** đọc trạng thái của diff. Không có prop nào từ
 *   trình xem diff đi vào component này, và không có `disabled` nào phụ thuộc diff.
 * * Bấm nút **không** làm đổi tệp đang chọn (`stopPropagation`), nên người dùng không
 *   bị buộc phải mở diff của một tệp trước khi stage nó.
 * * Component đọc **chỉ** `statusStore`. Một `DiffViewer` ném lỗi khi render nằm ở
 *   một nhánh cây React khác và không chạm tới đây.
 *
 * Có test ghim (`nut_stage_van_bam_duoc_khi_diff_hong`, đột biến M9 của plan — đột
 * biến quan trọng nhất của plan này).
 *
 * # Một cú bấm = một ý định
 *
 * Nút stage là **phần tử riêng**, không phải cùng phần tử với hàng. Nếu bấm hàng vừa
 * mở diff vừa stage thì người dùng không xem được trước khi stage — và nếu diff hỏng
 * thì họ mất luôn đường stage. Nút gọi `stopPropagation`; có test khẳng định bấm nút
 * **không** làm đổi tệp đang chọn (đột biến M7).
 *
 * # Nhóm rỗng KHÔNG hiện tiêu đề
 *
 * Ba tiêu đề trên một repo sạch là ba dòng nhiễu nói "không có gì" ba lần. Đột biến
 * M8 ghim điều này.
 *
 * # Chưa ảo hoá danh sách này, có chủ ý
 *
 * Thư mục làm việc có hàng chục tới hàng trăm tệp, không phải 100k — một repo có hàng
 * nghìn tệp **bẩn cùng lúc** nghĩa là người dùng vừa chạy một lệnh sinh mã, không
 * phải đang soạn commit. `useVirtualizer` ở đây thêm một nguồn tính chỉ số vào đúng
 * phase đang đụng vào bất biến virtualizer của WORK-11 (`CommitList` có **đúng một**
 * lời gọi `useVirtualizer`, và `GraphCanvas` vẽ từ **cùng** mảng `virtualItems` —
 * lớp lỗi "cột đồ thị lệch cột văn bản đúng một hàng" đã xảy ra **hai lần** ở Phase
 * 2). Nếu dogfood cho thấy chậm thì đó là một plan riêng, **có số đo**.
 *
 * # ⚠️ Bố cục CHƯA KIỂM CHỨNG
 *
 * happy-dom không tính CSS layout và không có cuộn thật (CONTEXT.md 3.4). Mọi khẳng
 * định về **bố cục** ở đây — ba nhóm không tràn, chiều cao hàng, nút không đè lên tên
 * tệp dài — là **có mã, chưa kiểm** cho tới khi có người xem trên Chromium thật. Việc
 * kiểm mắt người nằm ở checkpoint của plan 04-05.
 */

import { useEffect } from 'react'

import { runCommand, registerCommands, unregisterCommand } from '@/lib/commands'
import type { StatusEntry, StatusGroup } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import { useStatusStore } from '@/stores/statusStore'

interface Props {
  /**
   * Repo đang hiển thị. Không truyền → component không render gì: mọi thứ ở đây khoá
   * theo `repoId` (PLAT-05), và bịa một khoá `''` làm store của repo khác bị ghi đè.
   */
  repoId?: string
}

/**
 * Đường dẫn đang chờ `worktree.selectFile` đọc.
 *
 * Sống ở cấp module chứ không trong store — nó là **đối số của một lời gọi** (sống vài
 * micro giây giữa `onClick` và `run`), không phải trạng thái ứng dụng. Cùng khuôn và
 * cùng lý do với `pendingSelection` của `FileList.tsx`; đọc khối doc comment dài ở đầu
 * tệp đó để hiểu vì sao đường này được chọn thay vì truyền tham số qua sổ lệnh.
 */
let pendingSelection: { repoId: string; path: string; staged: boolean } | null = null

/** Thứ tự hiển thị ba nhóm, cộng tiêu đề của từng nhóm. */
const NHOM: { group: StatusGroup; tieuDe: string }[] = [
  { group: 'staged', tieuDe: 'Đã stage' },
  { group: 'unstaged', tieuDe: 'Chưa stage' },
  { group: 'untracked', tieuDe: 'Chưa theo dõi' },
]

const NHAN_XY: Record<string, string> = {
  M: 'Sửa',
  A: 'Thêm',
  D: 'Xoá',
  T: 'Đổi kiểu',
  R: 'Đổi tên',
  C: 'Sao chép',
  U: 'Xung đột',
  '?': 'Mới',
}

/**
 * Nhãn trạng thái của một phần tử, đọc từ ký tự XY **thuộc về nhóm của nó**.
 *
 * Một tệp `MM` xuất hiện ở hai nhóm; ở nhóm đã stage thì ký tự có nghĩa là ký tự
 * **đầu** (X, index), ở nhóm chưa stage là ký tự **sau** (Y, worktree). Đọc luôn ký
 * tự đầu sẽ hiện sai nhãn cho ca `AM` (thêm mới rồi sửa tiếp) ở nhóm chưa stage.
 */
function nhanTrangThai(e: StatusEntry): string {
  if (e.group === 'untracked') return NHAN_XY['?']!
  const ky_tu = e.group === 'staged' ? e.xy.charAt(0) : e.xy.charAt(1)
  return NHAN_XY[ky_tu] ?? e.xy
}

/** Bấm một hàng: ghi lựa chọn rồi chạy lệnh (PLAT-04, khuôn `FileList.tsx`). */
function chonTep(repoId: string, path: string, staged: boolean): void {
  pendingSelection = { repoId, path, staged }
  void runCommand('worktree.selectFile')
}

function HangTep({
  entry,
  repoId,
  daChon,
}: {
  entry: StatusEntry
  repoId: string
  daChon: boolean
}) {
  const stage = useStatusStore((s) => s.stage)
  const unstage = useStatusStore((s) => s.unstage)

  const laStaged = entry.group === 'staged'
  const nhanNut = laStaged ? 'Bỏ stage' : 'Stage'

  /*
   * 🔴 `stopPropagation` — một cú bấm là MỘT ý định.
   *
   * Thiếu nó, bấm nút stage cũng kích hoạt `onClick` của hàng và đổi tệp đang chọn.
   * Hệ quả không phải thẩm mỹ: người dùng đang đọc diff của tệp A, bấm stage tệp B,
   * và trình xem nhảy sang B — họ mất chỗ đang đọc. Đột biến M7 ghim điều này.
   *
   * Và ràng buộc R8 dựa vào đây: nếu bấm nút cũng mở diff thì một diff hỏng kéo theo
   * cả đường stage.
   */
  function bamNut(e: React.MouseEvent): void {
    e.stopPropagation()
    if (laStaged) void unstage(repoId, [entry.path])
    else void stage(repoId, [entry.path])
  }

  return (
    <div
      className={`change-row${daChon ? ' change-row-selected' : ''}`}
      data-testid="change-row"
      data-path={entry.path}
      data-group={entry.group}
      onClick={() => chonTep(repoId, entry.path, laStaged)}
    >
      <span className="change-status">{nhanTrangThai(entry)}</span>
      <span className="change-path">
        {entry.oldPath ? (
          <>
            <span className="change-old-path">{entry.oldPath}</span>
            {' → '}
            <span className="change-new-path">{entry.path}</span>
          </>
        ) : (
          entry.path
        )}
      </span>
      <button
        type="button"
        className="change-stage-button"
        data-testid={laStaged ? 'unstage-button' : 'stage-button'}
        aria-label={`${nhanNut} ${entry.path}`}
        onClick={bamNut}
      >
        {nhanNut}
      </button>
    </div>
  )
}

export function ChangeList({ repoId }: Props) {
  const lat = useStatusStore((s) => (repoId ? s.byRepo[repoId] : undefined))
  const refresh = useStatusStore((s) => s.refresh)
  const selectedPath = useDiffStore((s) =>
    repoId ? (s.selectedFileByRepo[repoId] ?? null) : null,
  )
  // Tệp `MM` có ở hai nhóm với cùng `path` — chỉ sáng hàng của đúng nhóm đã bấm.
  const selectedStaged = useDiffStore((s) =>
    repoId ? (s.worktreeByRepo[repoId]?.staged ?? null) : null,
  )

  /*
   * Đăng ký lệnh chọn tệp qua sổ đăng ký (PLAT-04).
   *
   * Tên riêng `worktree.selectFile`, **không** dùng lại `diff.selectFile` của
   * `FileList`: hai component có thể cùng ở trên màn hình, và `registerCommand` ném
   * lỗi khi trùng mã định danh (nó **không** im lặng ghi đè, có chủ ý). Dùng chung mã
   * nghĩa là component nào mount sau sẽ làm sập cây React.
   */
  useEffect(() => {
    registerCommands([
      {
        id: 'worktree.selectFile',
        title: 'Xem diff của tệp đang trỏ trong thư mục làm việc',
        category: 'view',
        run: () => {
          const chon = pendingSelection
          if (!chon) return
          pendingSelection = null
          // Diff **thư mục làm việc**, không phải diff commit — xem
          // `diffStore.worktreeByRepo`.
          useDiffStore.getState().selectWorktreeFile(chon.repoId, chon.path, chon.staged)
        },
      },
    ])
    return () => unregisterCommand('worktree.selectFile')
  }, [])

  useEffect(() => {
    if (repoId) void refresh(repoId)
  }, [repoId, refresh])

  if (!repoId) return null

  const status = lat?.status

  return (
    <div className="change-list" data-testid="change-list">
      {lat?.error && (
        <div className="change-list-error" role="alert">
          {lat.error}
        </div>
      )}

      {status === undefined ? (
        <p className="change-list-loading" data-testid="change-list-loading">
          Đang đọc trạng thái…
        </p>
      ) : status.entries.length === 0 ? (
        <p className="change-list-empty" data-testid="change-list-empty">
          Không có thay đổi nào.
        </p>
      ) : (
        NHOM.map(({ group, tieuDe }) => {
          const cua_nhom = status.entries.filter((e) => e.group === group)

          // 🔴 Nhóm rỗng KHÔNG hiện tiêu đề — đột biến M8. Ba tiêu đề trên một repo
          // sạch là ba dòng nhiễu nói "không có gì" ba lần.
          if (cua_nhom.length === 0) return null

          return (
            <section key={group} className="change-group" data-testid={`change-group-${group}`}>
              <h3 className="change-group-heading">
                {tieuDe} <span className="change-group-count">({cua_nhom.length})</span>
              </h3>
              <div className="change-group-rows">
                {cua_nhom.map((e) => (
                  // Khoá gồm **cả** nhóm: một tệp `MM` xuất hiện ở hai nhóm với cùng
                  // `path`, nên `path` một mình không phải khoá duy nhất.
                  <HangTep
                    key={`${group}:${e.path}`}
                    entry={e}
                    repoId={repoId}
                    daChon={
                      selectedPath === e.path && selectedStaged === (e.group === 'staged')
                    }
                  />
                ))}
              </div>
            </section>
          )
        })
      )}
    </div>
  )
}
