/**
 * `FileList` — bấm một hàng tệp để xem diff (bổ sung của plan 03-04).
 *
 * Tách khỏi `FileList.test.tsx` để hai lớp bất biến không lẫn nhau: tệp kia nói
 * về HIST-09 (phẳng ↔ cây), tệp này nói về việc mở diff.
 *
 * Cổng quan trọng: bấm hàng phải đi **qua sổ đăng ký lệnh** (PLAT-04). Plan nêu
 * rõ vì sao không dùng `grep -c 'runCommand' FileList.tsx`: `runCommand` đã có
 * trong tệp từ 02-06 cho nút chuyển dạng, nên phép đếm chuỗi xanh cả khi
 * `onClick` mới gắn hàm trực tiếp. Cổng đúng là thay `run` của lệnh đã đăng ký
 * bằng spy rồi bấm — nếu `onClick` không đi qua sổ đăng ký thì spy im.
 */

import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { FileList } from '@/components/history/FileList'
import { clearCommands, getCommand, listCommands } from '@/lib/commands'
import type { FileChange } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import { useUiStore } from '@/stores/uiStore'

function fc(path: string, overrides: Partial<FileChange> = {}): FileChange {
  return { status: 'M', path, oldPath: null, ...overrides }
}

beforeEach(() => {
  clearCommands()
  useUiStore.setState({ fileListView: 'flat' })
  useDiffStore.setState({
    selectedFileByRepo: {},
    viewMode: 'unified',
    showWhitespace: false,
  })
})

afterEach(() => {
  cleanup()
  clearCommands()
})

describe('bấm hàng tệp → chọn tệp để xem diff', () => {
  it('đăng ký lệnh diff.selectFile khi mount', () => {
    render(<FileList files={[fc('src/a.ts')]} truncated={false} repoId="r" />)
    expect(listCommands().map((c) => c.id)).toContain('diff.selectFile')
  })

  it('gỡ đăng ký khi unmount — mount lần hai không ném', () => {
    const a = render(<FileList files={[fc('src/a.ts')]} truncated={false} repoId="r" />)
    a.unmount()
    expect(() =>
      render(<FileList files={[fc('src/a.ts')]} truncated={false} repoId="r" />),
    ).not.toThrow()
  })

  it('🔴 bấm hàng đặt đúng đường dẫn vào diffStore theo repoId', () => {
    render(
      <FileList files={[fc('src/a.ts'), fc('src/b.rs')]} truncated={false} repoId="repo-1" />,
    )
    const rows = screen.getAllByTestId('file-row')
    fireEvent.click(rows[1]!)

    expect(useDiffStore.getState().selectedFileByRepo['repo-1']).toBe('src/b.rs')
  })

  it('🔴 bấm hàng đi QUA runCommand, không gắn hàm thẳng vào onClick (PLAT-04)', () => {
    render(<FileList files={[fc('src/a.ts')]} truncated={false} repoId="r" />)
    const spy = vi.fn()
    getCommand('diff.selectFile')!.run = spy

    fireEvent.click(screen.getAllByTestId('file-row')[0]!)
    expect(
      spy,
      'onClick phải gọi runCommand("diff.selectFile") — gắn hàm trực tiếp làm ' +
        'lệnh này vô hình với bảng lệnh gõ nhanh của v2',
    ).toHaveBeenCalledTimes(1)
  })

  it('bấm hàng ở dạng CÂY cũng chọn được, và dùng đường dẫn ĐẦY ĐỦ', () => {
    // Ở dạng cây, hàng chỉ hiện `a.ts` nhưng `getFileDiff` cần `src/a.ts`.
    useUiStore.setState({ fileListView: 'tree' })
    render(<FileList files={[fc('src/deep/a.ts')]} truncated={false} repoId="r" />)

    fireEvent.click(screen.getAllByTestId('file-row')[0]!)
    expect(useDiffStore.getState().selectedFileByRepo['r']).toBe('src/deep/a.ts')
  })

  it('tệp đổi tên: chọn đường dẫn PHÍA MỚI (path), không phải oldPath', () => {
    // `get_file_diff` nhận path phía mới; 03-02 tự tra `oldPath` từ
    // `--name-status`. Truyền `oldPath` làm git báo tệp bị xoá.
    render(
      <FileList
        files={[fc('moi.txt', { status: 'R77', oldPath: 'cu.txt' })]}
        truncated={false}
        repoId="r"
      />,
    )
    fireEvent.click(screen.getAllByTestId('file-row')[0]!)
    expect(useDiffStore.getState().selectedFileByRepo['r']).toBe('moi.txt')
  })

  it('hàng đang chọn có class .file-row-selected', () => {
    render(<FileList files={[fc('a.ts'), fc('b.ts')]} truncated={false} repoId="r" />)
    fireEvent.click(screen.getAllByTestId('file-row')[0]!)

    const rows = screen.getAllByTestId('file-row')
    expect(rows[0]!.className).toContain('file-row-selected')
    expect(rows[1]!.className).not.toContain('file-row-selected')
  })

  it('không truyền repoId → hàng vẫn render, chỉ là bấm không làm gì', () => {
    // `FileList` được dùng trong test cũ mà không có repoId; giữ prop optional
    // để không phá 7 test hiện có.
    render(<FileList files={[fc('a.ts')]} truncated={false} />)
    expect(screen.getAllByTestId('file-row')).toHaveLength(1)
    expect(() => fireEvent.click(screen.getAllByTestId('file-row')[0]!)).not.toThrow()
  })
})
