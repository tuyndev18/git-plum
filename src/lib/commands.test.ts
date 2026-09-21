/**
 * Test ràng buộc kiến trúc PLAT-04 — sổ đăng ký lệnh.
 *
 * Các test ở đây không kiểm tra tính năng người dùng nhìn thấy; chúng khoá lại
 * ba quy tắc của sổ đăng ký mà một lần tái cấu trúc vô ý có thể gỡ bỏ: không ghi
 * đè im lặng, lệnh lạ phải ném lỗi, lệnh bị vô hiệu hoá không được chạy. Ai định
 * xoá một test ở đây thì đọc dòng bình luận trên nó trước.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import {
  clearCommands,
  getCommand,
  listCommands,
  listEnabledCommands,
  registerCommand,
  registerCommands,
  runCommand,
  unregisterCommand,
  type Command,
} from '@/lib/commands'

/** Dựng một lệnh tối thiểu, cho phép ghi đè từng trường trong mỗi test. */
function makeCommand(overrides: Partial<Command> = {}): Command {
  return {
    id: 'repo.mo',
    title: 'Mở repository',
    category: 'repo',
    run: vi.fn(),
    ...overrides,
  }
}

// Sổ đăng ký là một `Map` ở phạm vi module — trạng thái dùng chung giữa các
// test. Không dọn ở đây thì kết quả phụ thuộc thứ tự chạy.
beforeEach(() => {
  clearCommands()
})

describe('đăng ký lệnh', () => {
  it('đăng ký rồi lấy lại được đúng lệnh đó', () => {
    // Ràng buộc: sổ đăng ký là nguồn dữ liệu cho bảng lệnh gõ nhanh ở v2.
    const command = makeCommand()

    registerCommand(command)

    expect(getCommand('repo.mo')).toBe(command)
  })

  it('từ chối mã định danh trùng và nêu tên mã trong thông điệp lỗi', () => {
    // Ràng buộc PLAT-04: không im lặng ghi đè. Ghi đè im lặng nghĩa là hai tính
    // năng cùng giành một mã lệnh và cái thua biến mất không dấu vết.
    registerCommand(makeCommand({ id: 'repo.trung' }))

    expect(() => registerCommand(makeCommand({ id: 'repo.trung' }))).toThrow(/repo\.trung/)
  })

  it('giữ nguyên lệnh đăng ký trước khi lần đăng ký trùng thất bại', () => {
    // Ràng buộc: lần đăng ký hỏng không được làm hỏng luôn lệnh đang có.
    const dauTien = makeCommand({ id: 'repo.trung', title: 'Bản gốc' })
    registerCommand(dauTien)

    expect(() => registerCommand(makeCommand({ id: 'repo.trung', title: 'Bản chèn' }))).toThrow()
    expect(getCommand('repo.trung')).toBe(dauTien)
  })

  it('registerCommands đăng ký được nhiều lệnh một lượt', () => {
    // Ràng buộc: một module tính năng khai báo trọn bộ lệnh của nó ở một chỗ.
    registerCommands([makeCommand({ id: 'a.mot' }), makeCommand({ id: 'b.hai' })])

    const ids = listCommands().map((c) => c.id)
    expect(ids).toContain('a.mot')
    expect(ids).toContain('b.hai')
    expect(listCommands()).toHaveLength(2)
  })
})

describe('gỡ đăng ký và dọn sổ', () => {
  it('unregisterCommand làm lệnh không còn tra ra được', () => {
    // Ràng buộc: tính năng gỡ đi thì lệnh của nó cũng phải biến khỏi bảng lệnh.
    registerCommand(makeCommand({ id: 'view.an' }))

    unregisterCommand('view.an')

    expect(getCommand('view.an')).toBeUndefined()
  })

  it('clearCommands làm sổ đăng ký rỗng', () => {
    // Ràng buộc: có đường dọn sạch, nếu không mỗi tệp test rò rỉ sang tệp khác.
    registerCommands([makeCommand({ id: 'a.mot' }), makeCommand({ id: 'b.hai' })])

    clearCommands()

    expect(listCommands()).toHaveLength(0)
  })
})

describe('liệt kê lệnh', () => {
  it('listEnabledCommands bỏ lệnh vô hiệu hoá, giữ lệnh không khai báo enabled', () => {
    // Ràng buộc PLAT-04: `enabled` khuyết nghĩa là luôn gọi được, không phải
    // luôn bị chặn — bảng lệnh không được rỗng chỉ vì thiếu khai báo.
    registerCommands([
      makeCommand({ id: 'a.bat', enabled: () => true }),
      makeCommand({ id: 'b.tat', enabled: () => false }),
      makeCommand({ id: 'c.khuyet' }),
    ])

    const ids = listEnabledCommands().map((c) => c.id)
    expect(ids).toEqual(['a.bat', 'c.khuyet'])
  })
})

describe('chạy lệnh', () => {
  it('runCommand với mã chưa đăng ký thì reject và nêu mã đó', async () => {
    // Ràng buộc PLAT-04: lỗi lập trình phải lộ ra lúc phát triển, không biến
    // thành một nút bấm im lặng không làm gì trong tay người dùng.
    await expect(runCommand('ma.khong.ton.tai')).rejects.toThrow(/ma\.khong\.ton\.tai/)
  })

  it('gọi run đúng một lần khi lệnh đang bật', async () => {
    // Ràng buộc: giao diện gọi `runCommand(id)`, sổ đăng ký chịu trách nhiệm
    // gọi đúng một lần — không gọi lặp, không bỏ sót.
    const run = vi.fn()
    registerCommand(makeCommand({ id: 'repo.chay', enabled: () => true, run }))

    await runCommand('repo.chay')

    expect(run).toHaveBeenCalledTimes(1)
  })

  it('không gọi run và cũng không ném lỗi khi lệnh bị vô hiệu hoá', async () => {
    // Ràng buộc PLAT-04: lệnh bị vô hiệu hoá là trạng thái hợp lệ (chưa mở repo
    // thì không đóng được), không phải sự cố cần ném lỗi.
    const run = vi.fn()
    registerCommand(makeCommand({ id: 'repo.dong', enabled: () => false, run }))

    await expect(runCommand('repo.dong')).resolves.toBeUndefined()
    expect(run).not.toHaveBeenCalled()
  })

  it('chờ xong lệnh bất đồng bộ trước khi hoàn tất', async () => {
    // Ràng buộc PLAT-04: `run` trả Promise thì `runCommand` phải `await` nó.
    // Không await nghĩa là người gọi không biết lệnh đã xong hay chưa.
    let giaiQuyet!: () => void
    const chuaXong = new Promise<void>((r) => {
      giaiQuyet = r
    })
    const daHoanTat = vi.fn()

    registerCommand(
      makeCommand({
        id: 'repo.bat-dong-bo',
        run: () => chuaXong.then(daHoanTat),
      }),
    )

    const dangChay = runCommand('repo.bat-dong-bo')
    // Nhường một vòng microtask: nếu runCommand bỏ qua promise, nó đã xong rồi.
    await Promise.resolve()
    expect(daHoanTat).not.toHaveBeenCalled()

    giaiQuyet()
    await dangChay
    expect(daHoanTat).toHaveBeenCalledTimes(1)
  })

  it('đọc enabled tại thời điểm chạy, không phải lúc đăng ký', async () => {
    // Ràng buộc PLAT-04: `enabled` là hàm chứ không phải boolean, nên trạng
    // thái đổi sau khi đăng ký vẫn được tôn trọng.
    let choPhep = false
    const run = vi.fn()
    registerCommand(makeCommand({ id: 'repo.dong-thai', enabled: () => choPhep, run }))

    await runCommand('repo.dong-thai')
    expect(run).not.toHaveBeenCalled()

    choPhep = true
    await runCommand('repo.dong-thai')
    expect(run).toHaveBeenCalledTimes(1)
  })
})
