/**
 * Test ranh giới IPC cho `get_file_history` (DIFF-05).
 *
 * Không kiểm logic — khoá lại **hợp đồng gọi**: tên command và hình dạng đối số. Cả
 * hai chỉ thất bại **lúc chạy**: gõ sai `get_file_history` thành `getFileHistory`, hay
 * truyền `repo_id` thay `repoId`, đều biên dịch sạch ở cả TypeScript lẫn Rust rồi ném
 * "command not found" trên máy người dùng.
 *
 * Khuôn `ipc.history.test.ts`.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

import { invoke } from '@tauri-apps/api/core'
import { ipc, type FileHistory, type FileVersion } from './ipc'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

const invokeMock = vi.mocked(invoke)

describe('ipc.getFileHistory', () => {
  beforeEach(() => {
    invokeMock.mockReset()
    invokeMock.mockResolvedValue({ path: 'a.txt', versions: [], truncated: false })
  })

  it('gọi đúng tên command `get_file_history`', async () => {
    await ipc.getFileHistory('repo-1', 'src/lib/ipc.ts')
    expect(invokeMock).toHaveBeenCalledWith('get_file_history', expect.anything())
  })

  it('truyền đối số dạng camelCase `{ repoId, path }`', async () => {
    await ipc.getFileHistory('repo-1', 'src/lib/ipc.ts')

    const [, doiSo] = invokeMock.mock.calls[0]!
    expect(doiSo).toEqual({ repoId: 'repo-1', path: 'src/lib/ipc.ts' })
    // Khẳng định **phủ định**: Tauri v2 nhận `repoId`, không nhận `repo_id`. Một
    // khoá snake_case ở đây làm `invoke` ném lúc chạy, không lúc biên dịch.
    expect(doiSo).not.toHaveProperty('repo_id')
  })

  it('KHÔNG truyền commitId — lịch sử tệp hỏi theo tệp, không theo commit', async () => {
    await ipc.getFileHistory('repo-1', 'a.txt')

    const [, doiSo] = invokeMock.mock.calls[0]!
    expect(doiSo).not.toHaveProperty('commitId')
  })

  it('hình dạng FileVersion: bảy trường, oldPath nullable', () => {
    const v: FileVersion = {
      commitId: 'a'.repeat(40),
      authorName: 'tuyenpn',
      authorTime: 1790000000,
      subject: 'sửa một thứ',
      status: 'M',
      path: 'a.txt',
      oldPath: null,
    }

    expect(v.commitId).toHaveLength(40)
    expect(v.oldPath).toBeNull()
    // `status` là chuỗi kèm điểm tương đồng, không phải một ký tự.
    const doiTen: FileVersion = { ...v, status: 'R077', oldPath: 'cu.txt', path: 'moi.txt' }
    expect(doiTen.status).toMatch(/^R\d+$/)
    expect(doiTen.oldPath).toBe('cu.txt')
  })

  it('🔴 authorTime là GIÂY Unix — nhân 1000 mới ra Date đúng', () => {
    // Ca này nằm trong checklist checkpoint của 02-06, tức một lỗi **đã được lường
    // trước**: `new Date(giây)` cho một ngày năm 1970.
    const v: FileVersion = {
      commitId: 'b'.repeat(40),
      authorName: 'x',
      authorTime: 1790000000,
      subject: 's',
      status: 'M',
      path: 'a.txt',
      oldPath: null,
    }

    expect(new Date(v.authorTime * 1000).getUTCFullYear()).toBe(2026)
    // Và hai cách thật sự khác nhau — nếu không, test trên không chứng minh gì.
    expect(new Date(v.authorTime).getUTCFullYear()).toBe(1970)
  })

  it('FileHistory mang path, versions và cờ truncated', async () => {
    const ls: FileHistory = {
      path: 'a.txt',
      versions: [],
      truncated: true,
    }
    invokeMock.mockResolvedValue(ls)

    const ra = await ipc.getFileHistory('r', 'a.txt')
    expect(ra.truncated).toBe(true)
    expect(ra.versions).toEqual([])
    expect(ra.path).toBe('a.txt')
  })
})
