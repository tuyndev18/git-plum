/**
 * Danh sách tệp thay đổi của một commit — HIST-09, dạng phẳng ↔ cây.
 *
 * Trạng thái chọn dạng sống ở `uiStore` (không `useState` cục bộ) vì nó phải
 * sống qua việc đổi commit — "người dùng đã chọn cách xem, không phải chọn
 * lại mỗi lần" (`<behavior>` Task 2, plan 02-06).
 *
 * KHÔNG có `onClick` trên hàng tệp — bấm để xem diff là Phase 3
 * (`<scope_boundary>` CONTEXT.md).
 */

import { useEffect } from 'react'

import { buildFileTree, type FileTreeNode } from '@/lib/fileTree'
import type { FileChange } from '@/lib/ipc'
import { registerCommands, runCommand, unregisterCommand } from '@/lib/commands'
import { useUiStore } from '@/stores/uiStore'

interface Props {
  files: FileChange[]
  truncated: boolean
}

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

function FlatRow({ change }: { change: FileChange }) {
  return (
    <div className="file-row" data-testid="file-row">
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

function TreeNode({ node, depth }: { node: FileTreeNode; depth: number }) {
  if (node.kind === 'file') {
    return (
      <div
        className="file-row file-row-tree"
        data-testid="file-row"
        style={{ paddingLeft: depth * 16 }}
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
        <TreeNode key={child.path} node={child} depth={depth + 1} />
      ))}
    </div>
  )
}

export function FileList({ files, truncated }: Props) {
  const view = useUiStore((s) => s.fileListView)

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
    ])
    return () => unregisterCommand('history.toggleFileView')
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
            <FlatRow key={`${f.path}:${f.oldPath ?? ''}`} change={f} />
          ))}
        </div>
      ) : (
        <div className="file-list-rows file-list-tree">
          {buildFileTree(files).map((node) => (
            <TreeNode key={node.path} node={node} depth={0} />
          ))}
        </div>
      )}
    </div>
  )
}
