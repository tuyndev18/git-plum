/**
 * Danh sách tệp thay đổi của một commit — HIST-09, dạng phẳng ↔ cây.
 *
 * Trạng thái chọn dạng sống ở `uiStore` (không `useState` cục bộ) vì nó phải
 * sống qua việc đổi commit — "người dùng đã chọn cách xem, không phải chọn
 * lại mỗi lần" (`<behavior>` Task 2, plan 02-06).
 *
 * Bấm một hàng tệp **mở diff của tệp đó** (plan 03-04). Trước 03-04 thì không —
 * `<scope_boundary>` của CONTEXT.md đặt việc đó vào Phase 3, và Phase 3 đã tới.
 *
 * ## Vì sao `onClick` đi qua `runCommand` dù `Command.run` không nhận tham số
 *
 * PLAT-04 đòi mọi thao tác đi qua sổ đăng ký, nhưng `Command.run` có chữ ký
 * `() => void | Promise<void>` — "mở tệp *này*" là thao tác *có tham số*.
 * `App.tsx` đã gặp đúng vấn đề này với "mở repo gần đây" và chọn **không** đi
 * qua sổ đăng ký, vì ở đó tham số là một đường dẫn tuỳ ý từ một danh sách động.
 *
 * Ở đây chọn khác, và lý do là hình dạng của tham số: đường dẫn tệp đang được
 * bấm là một **trạng thái giao diện** mà `diffStore` vốn đã giữ. Nên `onClick`
 * ghi path vào `diffStore` **trước** rồi gọi `runCommand('diff.selectFile')` —
 * lệnh trong sổ đăng ký đọc lại từ store. Đó không phải "tham số trá hình qua
 * biến toàn cục" mà `App.tsx` từ chối: store *là* nơi trạng thái đó sống dù có
 * lệnh hay không, và một lệnh có tên `diff.selectFile` trong bảng lệnh gõ nhanh
 * của v2 ("chọn lại tệp đang trỏ") vẫn có nghĩa mà không cần tham số.
 */

import { useEffect } from 'react'

import { buildFileTree, type FileTreeNode } from '@/lib/fileTree'
import type { FileChange } from '@/lib/ipc'
import { registerCommands, runCommand, unregisterCommand } from '@/lib/commands'
import { useDiffStore } from '@/stores/diffStore'
import { useUiStore } from '@/stores/uiStore'

interface Props {
  files: FileChange[]
  truncated: boolean
  /**
   * Repo chứa các tệp này. Không truyền → hàng vẫn render nhưng bấm không làm
   * gì: `selectedFileByRepo` cần một khoá repo, và bịa một khoá `''` sẽ làm
   * `DiffViewer` của repo khác đọc nhầm.
   */
  repoId?: string
}

/**
 * Đường dẫn đang chờ `diff.selectFile` đọc.
 *
 * Sống ở cấp module chứ không trong `diffStore`: nó là **đối số của một lời gọi**
 * (sống vài micro giây giữa `onClick` và `run`), không phải trạng thái ứng dụng.
 * Đưa vào store làm mọi thành phần đọc store render lại hai lần cho một cú bấm.
 */
let pendingSelection: { repoId: string; path: string } | null = null

const STATUS_LABELS: Record<string, string> = {
  A: 'Thêm',
  M: 'Sửa',
  D: 'Xoá',
  T: 'Đổi kiểu',
}

/** `status` có thể mang điểm tương đồng (`R100`, `C75`) — chỉ đọc ký tự đầu. */
function statusLabel(status: string): string {
  const code = status.charAt(0)
  if (code === 'R') return `Đổi tên${status.length > 1 ? ` (${status.slice(1)}%)` : ''}`
  if (code === 'C') return `Sao chép${status.length > 1 ? ` (${status.slice(1)}%)` : ''}`
  return STATUS_LABELS[code] ?? status
}

/**
 * Bấm một hàng: ghi lựa chọn rồi chạy lệnh (xem doc comment đầu tệp).
 *
 * Luôn dùng `change.path` — đường dẫn phía **mới**. `get_file_diff` nhận path
 * phía mới và 03-02 tự tra `oldPath` từ `--name-status`; truyền `oldPath` cho
 * một tệp đổi tên làm git báo tệp bị xoá.
 */
function chonTep(repoId: string | undefined, path: string): void {
  if (!repoId) return
  pendingSelection = { repoId, path }
  void runCommand('diff.selectFile')
}

function FlatRow({
  change,
  repoId,
  selected,
}: {
  change: FileChange
  repoId?: string
  selected: boolean
}) {
  return (
    <div
      className={`file-row${selected ? ' file-row-selected' : ''}`}
      data-testid="file-row"
      onClick={() => chonTep(repoId, change.path)}
    >
      <span className="file-status">{statusLabel(change.status)}</span>
      <span className="file-path">
        {change.oldPath ? (
          <>
            <span className="file-old-path">{change.oldPath}</span>
            {' → '}
            <span className="file-new-path">{change.path}</span>
          </>
        ) : (
          change.path
        )}
      </span>
    </div>
  )
}

function TreeNode({
  node,
  depth,
  repoId,
  selectedPath,
}: {
  node: FileTreeNode
  depth: number
  repoId?: string
  selectedPath: string | null
}) {
  if (node.kind === 'file') {
    // Hàng hiện tên NGẮN (`a.ts`) nhưng `getFileDiff` cần đường dẫn ĐẦY ĐỦ —
    // lấy từ `node.change.path`, không từ `node.name`.
    const selected = selectedPath === node.change.path
    return (
      <div
        className={`file-row file-row-tree${selected ? ' file-row-selected' : ''}`}
        data-testid="file-row"
        style={{ paddingLeft: depth * 16 }}
        onClick={() => chonTep(repoId, node.change.path)}
      >
        <span className="file-status">{statusLabel(node.change.status)}</span>
        <span className="file-path">{node.name}</span>
      </div>
    )
  }

  return (
    <div className="file-tree-dir">
      <div className="file-tree-dir-name" style={{ paddingLeft: depth * 16 }}>
        {node.name}
      </div>
      {node.children.map((child) => (
        <TreeNode
          key={child.path}
          node={child}
          depth={depth + 1}
          repoId={repoId}
          selectedPath={selectedPath}
        />
      ))}
    </div>
  )
}

export function FileList({ files, truncated, repoId }: Props) {
  const view = useUiStore((s) => s.fileListView)
  const selectedPath = useDiffStore((s) => (repoId ? (s.selectedFileByRepo[repoId] ?? null) : null))

  // Đăng ký lệnh chuyển dạng qua sổ đăng ký (PLAT-04). `FileList` có thể
  // render/unmount nhiều lần (đổi commit không unmount, nhưng đổi panel có
  // thể) — đăng ký lại là vô hại vì `id` ổn định và không gọi hai lần đồng
  // thời trong cùng một cây React.
  useEffect(() => {
    registerCommands([
      {
        id: 'history.toggleFileView',
        title: 'Chuyển dạng danh sách tệp (phẳng/cây)',
        category: 'history',
        run: () => useUiStore.getState().toggleFileListView(),
      },
      {
        id: 'diff.selectFile',
        title: 'Xem diff của tệp đang trỏ',
        category: 'view',
        run: () => {
          const chon = pendingSelection
          if (!chon) return
          pendingSelection = null
          useDiffStore.getState().selectFile(chon.repoId, chon.path)
        },
      },
    ])
    return () => {
      unregisterCommand('history.toggleFileView')
      unregisterCommand('diff.selectFile')
    }
  }, [])

  return (
    <div className="file-list">
      <div className="file-list-header">
        <span className="file-list-count">{files.length} tệp thay đổi</span>
        <button onClick={() => void runCommand('history.toggleFileView')}>
          {view === 'flat' ? 'Xem dạng cây' : 'Xem dạng phẳng'}
        </button>
      </div>

      {truncated && (
        <div className="file-list-truncated" role="alert">
          Danh sách tệp đã bị cắt vì commit này đổi quá nhiều tệp.
        </div>
      )}

      {files.length === 0 ? (
        <p className="file-list-empty">Không có tệp nào thay đổi.</p>
      ) : view === 'flat' ? (
        <div className="file-list-rows">
          {files.map((f) => (
            <FlatRow
              key={`${f.path}:${f.oldPath ?? ''}`}
              change={f}
              repoId={repoId}
              selected={selectedPath === f.path}
            />
          ))}
        </div>
      ) : (
        <div className="file-list-rows file-list-tree">
          {buildFileTree(files).map((node) => (
            <TreeNode
              key={node.path}
              node={node}
              depth={0}
              repoId={repoId}
              selectedPath={selectedPath}
            />
          ))}
        </div>
      )}
    </div>
  )
}
