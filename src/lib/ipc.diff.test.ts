/**
 * Test ranh giới IPC cho `get_file_diff` (Phase 3, plan 03-02).
 *
 * Không kiểm logic — khoá lại **hợp đồng gọi**: tên command và hình dạng đối số. Cả
 * hai chỉ thất bại **lúc chạy**: gõ `getFileDiff` thay vì `get_file_diff`, hay truyền
 * `commit_id` thay cho `commitId`, đều biên dịch sạch ở cả TypeScript lẫn Rust rồi ném
 * "command not found" trên máy người dùng.
 *
 * Tauri v2 chuyển tham số `snake_case` của Rust thành `camelCase` ở phía JS. Cùng
 * khuôn với `ipc.history.test.ts`.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'
import { ipc, type DiffKind, type FileDiff, type Hunk } from './ipc'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

beforeEach(() => {
  // `vite.config.ts` đặt `restoreMocks: true`, nên giá trị mặc định phải dựng lại ở
  // đây — không phải cẩn thận thừa (xem ghi chú trong ipc.history.test.ts).
  invokeMock.mockReset()
  invokeMock.mockResolvedValue(undefined)
})

describe('tên command và hình dạng đối số', () => {
  it('getFileDiff gọi get_file_diff với { repoId, commitId, path }', async () => {
    await ipc.getFileDiff('repo-1', 'abc123', 'src/a.rs')

    expect(invokeMock).toHaveBeenCalledTimes(1)
    expect(invokeMock).toHaveBeenCalledWith('get_file_diff', {
      repoId: 'repo-1',
      commitId: 'abc123',
      path: 'src/a.rs',
    })
  })

  it('không đổi tên khoá đối số thành snake_case', async () => {
    await ipc.getFileDiff('r', 'c', 'p')

    const loiGoi = invokeMock.mock.calls[0]
    expect(loiGoi).toBeDefined()
    const khoa = Object.keys(loiGoi![1] as object).sort()
    expect(khoa).toEqual(['commitId', 'path', 'repoId'])
    // Ghim ngược: `repo_id` ở đây làm Tauri ném lỗi lúc chạy, không lúc biên dịch.
    expect(khoa).not.toContain('repo_id')
    expect(khoa).not.toContain('commit_id')
  })

  it('truyền path nguyên vẹn, kể cả khi nó trùng tên nhánh', async () => {
    // Phía Rust có `--` ngăn cách revision với pathspec (T-03-07). Phía này không
    // được "giúp" bằng cách thêm tiền tố hay escape — làm vậy sẽ gửi một path khác
    // với path người dùng bấm vào.
    await ipc.getFileDiff('r', 'c', 'main')

    expect(invokeMock).toHaveBeenCalledWith('get_file_diff', {
      repoId: 'r',
      commitId: 'c',
      path: 'main',
    })
  })
})

describe('DiffKind là discriminated union theo khoá kind', () => {
  /**
   * Phân nhánh theo `kind` phải thu hẹp kiểu đủ để đọc được trường riêng của từng
   * dạng **mà không cần ép kiểu**. Nếu union bị khai sai (ví dụ mọi trường thành
   * optional), đoạn dưới vẫn chạy nhưng `npm run typecheck` sẽ đỏ — đó chính là cổng.
   */
  function moTa(k: DiffKind): string {
    switch (k.kind) {
      case 'text':
        return `${k.hunks.length} hunk${k.truncated ? ' (đã cắt)' : ''}`
      case 'binary':
        return `nhị phân ${k.oldSize} → ${k.newSize}`
      case 'tooLarge':
        return `${k.size} vượt ngưỡng ${k.limit}`
      case 'lfsPointer':
        return `LFS ${k.oid.slice(0, 8)} cỡ ${k.size}`
      case 'unchanged':
        return 'không đổi'
    }
  }

  it('đọc được trường riêng của cả năm dạng', () => {
    expect(moTa({ kind: 'text', hunks: [], truncated: false })).toBe('0 hunk')
    expect(moTa({ kind: 'text', hunks: [], truncated: true })).toBe('0 hunk (đã cắt)')
    expect(moTa({ kind: 'binary', oldSize: 10, newSize: 20 })).toBe('nhị phân 10 → 20')
    expect(moTa({ kind: 'tooLarge', size: 9_000_000, limit: 5_242_880 })).toBe(
      '9000000 vượt ngưỡng 5242880',
    )
    expect(moTa({ kind: 'lfsPointer', oid: 'ab'.repeat(32), size: 1_048_576 })).toBe(
      'LFS abababab cỡ 1048576',
    )
    expect(moTa({ kind: 'unchanged' })).toBe('không đổi')
  })

  it('dạng binary không mang trường nội dung nào (T-03-13)', () => {
    const b: DiffKind = { kind: 'binary', oldSize: 4096, newSize: 8192 }
    // Chỉ ba khoá. Thêm một khoá nội dung ở phía Rust mà quên phía này thì giao diện
    // sẽ lặng lẽ có quyền đọc byte tệp nhị phân.
    expect(Object.keys(b).sort()).toEqual(['kind', 'newSize', 'oldSize'])
  })
})

describe('hình dạng dữ liệu mà giao diện 03-04 dựng bố cục từ đó', () => {
  it('dòng ngữ cảnh có cả hai số dòng, added/removed chỉ có một', () => {
    const h: Hunk = {
      oldStart: 3,
      oldCount: 3,
      newStart: 3,
      newCount: 4,
      heading: 'fn main()',
      lines: [
        { kind: 'context', content: 'a', oldLine: 3, newLine: 3, noNewlineAtEof: false },
        { kind: 'removed', content: 'b', oldLine: 4, newLine: null, noNewlineAtEof: false },
        { kind: 'added', content: 'B', oldLine: null, newLine: 4, noNewlineAtEof: false },
      ],
    }

    // Chế độ hai cột (DIFF-02) đọc đúng hai trường này. Gộp chúng thành một số duy
    // nhất làm hai cột lệch hàng — lớp lỗi đã gây hai vòng checkpoint thất bại ở
    // Phase 2.
    const cotTrai = h.lines.filter((l) => l.oldLine !== null).map((l) => l.oldLine)
    const cotPhai = h.lines.filter((l) => l.newLine !== null).map((l) => l.newLine)
    expect(cotTrai).toEqual([3, 4])
    expect(cotPhai).toEqual([3, 4])
  })

  it('FileDiff mang oldPath chỉ khi tệp bị đổi tên', () => {
    const sua: FileDiff = {
      path: 'a.txt',
      oldPath: null,
      status: 'M',
      kind: { kind: 'unchanged' },
    }
    const doiTen: FileDiff = {
      path: 'moi.txt',
      oldPath: 'cu.txt',
      status: 'R77',
      kind: { kind: 'unchanged' },
    }

    expect(sua.oldPath).toBeNull()
    expect(doiTen.oldPath).toBe('cu.txt')
    // `status` là chuỗi, không phải ký tự: git in kèm điểm tương đồng.
    expect(doiTen.status).toMatch(/^R\d+$/)
  })
})
