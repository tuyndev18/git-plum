/**
 * Test `FileList` — HIST-09, chuyển phẳng/cây danh sách tệp thay đổi.
 *
 * Trạng thái chọn dạng (phẳng/cây) sống ở `uiStore`, không `useState` cục bộ —
 * phải còn nguyên khi component cha đổi `files` prop (đổi commit khác).
 */

import { beforeEach, describe, expect, it } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'

import { FileList } from '@/components/history/FileList'
import { clearCommands } from '@/lib/commands'
import { useUiStore } from '@/stores/uiStore'
import type { FileChange } from '@/lib/ipc'

function fc(path: string, overrides: Partial<FileChange> = {}): FileChange {
  return { status: 'M', path, oldPath: null, ...overrides }
}

beforeEach(() => {
  clearCommands()
  useUiStore.setState({ fileListView: 'flat' })
})

describe('dạng phẳng (mặc định)', () => {
  it('mỗi FileChange một hàng, hiện status và path đầy đủ', () => {
    render(<FileList files={[fc('src/a.ts'), fc('src/b.ts', { status: 'A' })]} truncated={false} />)

    expect(screen.getByText('src/a.ts')).toBeTruthy()
    expect(screen.getByText('src/b.ts')).toBeTruthy()
  })
})

describe('chuyển dạng', () => {
  it('bấm nút chuyển -> dạng cây: thấy tên thư mục, tệp hiện tên ngắn không kèm đường dẫn cha', async () => {
    render(<FileList files={[fc('src/a.ts')]} truncated={false} />)

    screen.getByRole('button', { name: /cây|dạng/i }).click()

    await waitFor(() => expect(screen.getByText('src')).toBeTruthy())
    expect(screen.getByText('a.ts')).toBeTruthy()
    expect(screen.queryByText('src/a.ts')).toBeNull()
  })

  it('bấm lại -> về dạng phẳng', async () => {
    render(<FileList files={[fc('src/a.ts')]} truncated={false} />)

    const toggle = screen.getByRole('button', { name: /cây|dạng/i })
    toggle.click()
    await waitFor(() => expect(screen.getByText('src')).toBeTruthy())
    toggle.click()

    await waitFor(() => expect(screen.getByText('src/a.ts')).toBeTruthy())
  })

  it('trạng thái chọn dạng còn nguyên khi đổi sang commit khác (files prop đổi)', async () => {
    const { rerender } = render(<FileList files={[fc('src/a.ts')]} truncated={false} />)

    screen.getByRole('button', { name: /cây|dạng/i }).click()
    await waitFor(() => expect(screen.getByText('src')).toBeTruthy())

    rerender(<FileList files={[fc('lib/b.ts')]} truncated={false} />)

    expect(screen.getByText('lib')).toBeTruthy()
    expect(screen.queryByText('lib/b.ts')).toBeNull()
  })
})

describe('đổi tên', () => {
  it("oldPath khác null -> hiện cả đường dẫn cũ và mới", () => {
    render(
      <FileList
        files={[fc('new.ts', { status: 'R100', oldPath: 'old.ts' })]}
        truncated={false}
      />,
    )

    expect(screen.getByText(/old\.ts/)).toBeTruthy()
    expect(screen.getByText(/new\.ts/)).toBeTruthy()
  })
})

describe('danh sách rỗng', () => {
  it('thông báo đọc được, không bảng rỗng', () => {
    render(<FileList files={[]} truncated={false} />)

    expect(screen.queryByRole('table')).toBeNull()
    expect(screen.getByText(/không có tệp|rỗng/i)).toBeTruthy()
  })
})

describe('byte lossy (HIST-11)', () => {
  it('đường dẫn chứa U+FFFD vẫn hiện một hàng', () => {
    render(<FileList files={[fc('caf�.txt')]} truncated={false} />)

    expect(screen.getByText(/caf.*\.txt/)).toBeTruthy()
  })
})

describe('trạng thái nhãn', () => {
  it.each([
    ['A', 'Thêm'],
    ['M', 'Sửa'],
    ['D', 'Xoá'],
    ['R100', 'Đổi tên'],
    ['C75', 'Sao chép'],
    ['T', 'Đổi kiểu'],
  ])('status %s -> nhãn chứa %s', (status, expected) => {
    render(<FileList files={[fc('x.ts', { status })]} truncated={false} />)
    expect(screen.getByText(new RegExp(expected))).toBeTruthy()
  })

  it('ký tự lạ -> hiện nguyên ký tự, không bỏ hàng', () => {
    render(<FileList files={[fc('x.ts', { status: 'Z' })]} truncated={false} />)
    expect(screen.getByText('x.ts')).toBeTruthy()
  })
})

describe('phạm vi Phase 3 — không có onClick dẫn tới diff', () => {
  it('hàng tệp không có handler bấm', () => {
    render(<FileList files={[fc('src/a.ts')]} truncated={false} />)

    const row = screen.getByText('src/a.ts').closest('[data-testid="file-row"]')
    expect(row).not.toBeNull()
    expect(row?.getAttribute('onclick')).toBeNull()
    // Không có role button/link bọc quanh hàng tệp.
    expect(row?.tagName.toLowerCase()).not.toBe('button')
  })
})
