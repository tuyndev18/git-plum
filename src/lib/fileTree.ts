/**
 * Cây thư mục từ danh sách tệp thay đổi — HIST-09. Hàm THUẦN, không React.
 *
 * Tách theo `/` DUY NHẤT — git luôn dùng `/` kể cả trên Windows. Tách theo `\`
 * sẽ hiểu sai một tên tệp hợp lệ chứa dấu gạch chéo ngược thành thư mục.
 */

import type { FileChange } from '@/lib/ipc'

export type FileTreeNode =
  | { kind: 'dir'; name: string; path: string; children: FileTreeNode[] }
  | { kind: 'file'; name: string; path: string; change: FileChange }

interface MutableDir {
  kind: 'dir'
  name: string
  path: string
  children: Map<string, MutableDir | MutableFile>
}

interface MutableFile {
  kind: 'file'
  name: string
  path: string
  change: FileChange
}

function sortChildren(nodes: (MutableDir | MutableFile)[]): (MutableDir | MutableFile)[] {
  return nodes.sort((a, b) => {
    if (a.kind !== b.kind) return a.kind === 'dir' ? -1 : 1
    return a.name.localeCompare(b.name)
  })
}

function freeze(node: MutableDir | MutableFile): FileTreeNode {
  if (node.kind === 'file') {
    return { kind: 'file', name: node.name, path: node.path, change: node.change }
  }
  return {
    kind: 'dir',
    name: node.name,
    path: node.path,
    children: sortChildren([...node.children.values()]).map(freeze),
  }
}

export function buildFileTree(files: FileChange[]): FileTreeNode[] {
  const root = new Map<string, MutableDir | MutableFile>()

  for (const change of files) {
    const segments = change.path.split('/')
    let level = root
    let accumulatedPath = ''

    for (let i = 0; i < segments.length; i += 1) {
      const segment = segments[i]
      if (segment === undefined) continue
      const isLast = i === segments.length - 1
      accumulatedPath = accumulatedPath ? `${accumulatedPath}/${segment}` : segment

      if (isLast) {
        level.set(segment, {
          kind: 'file',
          name: segment,
          path: accumulatedPath,
          change,
        })
      } else {
        let existing = level.get(segment)
        if (!existing || existing.kind !== 'dir') {
          existing = { kind: 'dir', name: segment, path: accumulatedPath, children: new Map() }
          level.set(segment, existing)
        }
        level = existing.children
      }
    }
  }

  return sortChildren([...root.values()]).map(freeze)
}
