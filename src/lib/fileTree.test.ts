/**
 * Test `buildFileTree` — hàm thuần, không React. HIST-09.
 */

import { describe, expect, it } from 'vitest'

import { buildFileTree } from '@/lib/fileTree'
import type { FileChange } from '@/lib/ipc'

function fc(path: string, overrides: Partial<FileChange> = {}): FileChange {
  return { status: 'M', path, oldPath: null, ...overrides }
}

describe('buildFileTree', () => {
  it("['a.txt'] -> một nút tệp ở gốc", () => {
    const tree = buildFileTree([fc('a.txt')])

    expect(tree).toHaveLength(1)
    expect(tree[0]).toMatchObject({ kind: 'file', name: 'a.txt', path: 'a.txt' })
  })

  it("['src/a.ts', 'src/b.ts'] -> một nút thư mục src chứa hai tệp", () => {
    const tree = buildFileTree([fc('src/a.ts'), fc('src/b.ts')])

    expect(tree).toHaveLength(1)
    const dir = tree[0]
    expect(dir?.kind).toBe('dir')
    if (dir?.kind !== 'dir') throw new Error('expected dir')
    expect(dir.name).toBe('src')
    expect(dir.children).toHaveLength(2)
    expect(dir.children.map((c) => c.name)).toEqual(['a.ts', 'b.ts'])
  })

  it("['src/x/y.ts', 'src/z.ts'] -> src chứa thư mục x và tệp z.ts; lồng đúng độ sâu", () => {
    const tree = buildFileTree([fc('src/x/y.ts'), fc('src/z.ts')])

    expect(tree).toHaveLength(1)
    const src = tree[0]
    if (src?.kind !== 'dir') throw new Error('expected dir')
    expect(src.children.map((c) => c.name)).toEqual(['x', 'z.ts'])

    const x = src.children[0]
    if (x?.kind !== 'dir') throw new Error('expected dir x')
    expect(x.children).toHaveLength(1)
    expect(x.children[0]).toMatchObject({ kind: 'file', name: 'y.ts', path: 'src/x/y.ts' })
  })

  it('đường dẫn chứa ký tự Unicode và khoảng trắng -> giữ nguyên, không tách sai', () => {
    const tree = buildFileTree([fc('thư mục/tệp có dấu.txt')])

    expect(tree).toHaveLength(1)
    const dir = tree[0]
    if (dir?.kind !== 'dir') throw new Error('expected dir')
    expect(dir.name).toBe('thư mục')
    expect(dir.children[0]).toMatchObject({ name: 'tệp có dấu.txt' })
  })

  it('đường dẫn chứa \\ (Windows) -> KHÔNG tách theo \\; là một tệp tên a\\b.txt', () => {
    const tree = buildFileTree([fc('a\\b.txt')])

    expect(tree).toHaveLength(1)
    expect(tree[0]).toMatchObject({ kind: 'file', name: 'a\\b.txt', path: 'a\\b.txt' })
  })

  it('mảng rỗng -> cây rỗng, không lỗi', () => {
    expect(buildFileTree([])).toEqual([])
  })

  it('thư mục sắp trước tệp, mỗi nhóm sắp theo tên — thứ tự tất định', () => {
    const tree = buildFileTree([
      fc('zeta.txt'),
      fc('alpha/inner.txt'),
      fc('beta.txt'),
      fc('alpha.txt'),
    ])

    // alpha (dir) trước alpha.txt/beta.txt/zeta.txt (files), files sắp theo tên
    expect(tree.map((n) => n.name)).toEqual(['alpha', 'alpha.txt', 'beta.txt', 'zeta.txt'])
    expect(tree[0]?.kind).toBe('dir')
  })

  it('mỗi nút tệp giữ tham chiếu FileChange gốc (change)', () => {
    const change = fc('a.txt', { status: 'A' })
    const tree = buildFileTree([change])

    const node = tree[0]
    if (node?.kind !== 'file') throw new Error('expected file')
    expect(node.change).toBe(change)
  })
})
