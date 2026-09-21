/**
 * Test component danh sách repository gần đây — PLAT-07.
 *
 * Đây là chỗ duy nhất trong plan này thật sự render, nên dùng
 * `@testing-library/react`; test logic store vẫn gọi `getState()` trực tiếp
 * theo khuôn mẫu plan 01-02 đặt ra.
 */

import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'

import { RecentRepoList } from '@/components/RecentRepoList'
import type { RecentRepo } from '@/lib/recentRepos'

function muc(path: string, name: string): RecentRepo {
  return { path, name, openedAtMs: 1 }
}

describe('RecentRepoList', () => {
  it('không render gì khi danh sách rỗng', () => {
    // Màn hình trống đã có nút "Mở repository"; một tiêu đề trên khoảng trắng
    // chỉ thêm nhiễu vào đúng lúc người dùng chưa có gì để chọn.
    const { container } = render(
      <RecentRepoList items={[]} onOpen={vi.fn()} onForget={vi.fn()} />,
    )

    // Khẳng định bằng DOM thuần, không dùng `toBeEmptyDOMElement`: matcher đó
    // thuộc `@testing-library/jest-dom`, gói chưa có trong dự án. Thêm một phụ
    // thuộc chỉ để đọc cho đẹp hơn một dòng là không đáng.
    expect(container.innerHTML).toBe('')
  })

  it('hiện tên và đường dẫn của từng mục, giữ thứ tự được truyền vào', () => {
    render(
      <RecentRepoList
        items={[muc('C:/kho/a', 'alpha'), muc('C:/kho/b', 'beta')]}
        onOpen={vi.fn()}
        onForget={vi.fn()}
      />,
    )

    expect(screen.getByText('alpha')).toBeTruthy()
    expect(screen.getByText('C:/kho/a')).toBeTruthy()
    // Thứ tự mới-nhất-trước do `mergeRecent` quyết định; component không sắp lại.
    const ten = screen.getAllByRole('button').map((b) => b.textContent)
    expect(ten[0]).toContain('alpha')
  })

  it('gọi onOpen đúng một lần với đường dẫn của mục, bằng một lần bấm', () => {
    // Tiêu chí thành công số 1 của phase: "mở bằng đúng một lần bấm".
    const moRepo = vi.fn()
    render(
      <RecentRepoList items={[muc('C:/kho/a', 'alpha')]} onOpen={moRepo} onForget={vi.fn()} />,
    )

    screen.getByTitle('C:/kho/a').click()

    expect(moRepo).toHaveBeenCalledTimes(1)
    expect(moRepo).toHaveBeenCalledWith('C:/kho/a')
  })

  it('nút xoá gọi onForget mà không kích hoạt onOpen', () => {
    // Ràng buộc: hai thao tác nằm cạnh nhau trong cùng một dòng. Bấm xoá mà
    // repository cũng mở ra là lỗi khó chịu nhất của kiểu bố cục này.
    const moRepo = vi.fn()
    const xoa = vi.fn()
    render(
      <RecentRepoList items={[muc('C:/kho/a', 'alpha')]} onOpen={moRepo} onForget={xoa} />,
    )

    screen.getByLabelText('Xoá alpha khỏi danh sách gần đây').click()

    expect(xoa).toHaveBeenCalledWith('C:/kho/a')
    expect(moRepo).not.toHaveBeenCalled()
  })

  it('dùng phần tử button để bàn phím và trình đọc màn hình dùng được', () => {
    // Một `<div onClick>` trông giống hệt trên màn hình nhưng không tab tới
    // được và không được đọc là thao tác gọi được.
    render(
      <RecentRepoList items={[muc('C:/kho/a', 'alpha')]} onOpen={vi.fn()} onForget={vi.fn()} />,
    )

    const nut = screen.getByTitle('C:/kho/a')
    expect(nut.tagName).toBe('BUTTON')
  })

  it('đặt title là đường dẫn đầy đủ để đường dẫn dài vẫn đọc được khi hover', () => {
    // Đường dẫn bị cắt bằng CSS ellipsis, nên `title` là đường duy nhất để
    // người dùng thấy toàn bộ đường dẫn mà không phải mở repository ra.
    const dai = 'C:/mot/duong/dan/rat/dai/lam/vo/bo/cuc/neu/khong/cat/kho-cua-toi'
    render(
      <RecentRepoList items={[muc(dai, 'kho-cua-toi')]} onOpen={vi.fn()} onForget={vi.fn()} />,
    )

    expect(screen.getByTitle(dai)).toBeTruthy()
  })

  it('dùng path làm key nên hai repository cùng tên vẫn render cả hai', () => {
    // Trùng tên thư mục là chuyện thường (`web/app` và `mobile/app`). Nếu ai
    // đổi `key` sang `item.name` thì React gộp hai dòng thành một.
    render(
      <RecentRepoList
        items={[muc('C:/web/app', 'app'), muc('C:/mobile/app', 'app')]}
        onOpen={vi.fn()}
        onForget={vi.fn()}
      />,
    )

    expect(screen.getAllByText('app')).toHaveLength(2)
  })
})
