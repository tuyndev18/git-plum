/**
 * Test danh sách repository gần đây — PLAT-07.
 *
 * Phần lớn test nhắm vào hai hàm thuần tuý `mergeRecent` và `sanitizeRecent`.
 * Đó là chỗ chứa toàn bộ quyết định về hành vi: thứ tự, khử trùng lặp, giới hạn
 * số mục, và việc chịu đựng dữ liệu rác trên đĩa (T-01-09). Vỏ bọc Tauri chỉ
 * được test ở đúng điều quan trọng nhất của nó: **không bao giờ ném lỗi**.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { load } from '@tauri-apps/plugin-store'

import {
  MAX_RECENT,
  forgetRepo,
  loadRecentRepos,
  mergeRecent,
  normalizeRepoPath,
  rememberRepo,
  __resetStoreForTests,
  sanitizeRecent,
  type RecentRepo,
} from '@/lib/recentRepos'

// Giả lập ở ranh giới module plugin, theo đúng khuôn mẫu mà plan 01-02 đặt ra
// cho `@/lib/ipc`. `vi.mock` được cẩu lên đầu tệp nên factory không tham chiếu
// biến ngoài.
vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(),
}))

const moStore = vi.mocked(load)

/** Store giả với ba phương thức plan cho phép dùng dưới `store:default`. */
function storeGia(giaTriBanDau: unknown) {
  return {
    get: vi.fn().mockResolvedValue(giaTriBanDau),
    set: vi.fn().mockResolvedValue(undefined),
    save: vi.fn().mockResolvedValue(undefined),
  }
}

function muc(path: string, name = path, openedAtMs = 1000): RecentRepo {
  return { path, name, openedAtMs }
}

beforeEach(() => {
  vi.clearAllMocks()
  // Vỏ bọc đệm đối tượng store ở phạm vi module để không mở lại tệp mỗi lần
  // gọi. Không reset thì store giả của test trước còn sống ở test sau.
  __resetStoreForTests()
})

describe('mergeRecent', () => {
  it('đặt mục mới lên đầu danh sách', () => {
    const ketQua = mergeRecent([muc('C:/kho/a'), muc('C:/kho/b')], muc('C:/kho/c'))

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/c', 'C:/kho/a', 'C:/kho/b'])
  })

  it('đưa repository đã có lên đầu mà không làm danh sách dài thêm', () => {
    // Ràng buộc cốt lõi: mở lại một repo cũ là thao tác thường gặp nhất. Nếu
    // không khử trùng lặp, danh sách mười mục biến thành mười bản sao của repo
    // người dùng dùng nhiều nhất.
    const truoc = [muc('C:/kho/a'), muc('C:/kho/b'), muc('C:/kho/c')]

    const ketQua = mergeRecent(truoc, muc('C:/kho/c', 'c', 9999))

    expect(ketQua).toHaveLength(3)
    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/c', 'C:/kho/a', 'C:/kho/b'])
    expect(ketQua[0]?.openedAtMs).toBe(9999)
  })

  it('coi đường dẫn khác dấu gạch chéo và khác dấu cuối là cùng một repository', () => {
    // Ràng buộc: phía Rust chuẩn hoá `\` thành `/` và cắt `/` ở cuối trước khi
    // băm ra `RepoId` (`state::repo_id_for`). Nếu phép khử trùng lặp ở đây
    // không đồng ý với nó, một repository sinh ra hai mục trong danh sách
    // trong khi backend vẫn chỉ thấy một.
    const truoc = [muc('C:/kho/a')]

    const ketQua = mergeRecent(truoc, muc('C:\\kho\\a\\', 'a', 5000))

    expect(ketQua).toHaveLength(1)
    expect(ketQua[0]?.openedAtMs).toBe(5000)
  })

  it('cắt bớt từ cuối, giữ đúng MAX_RECENT mục', () => {
    // Ràng buộc T-01-12: danh sách phình to vô hạn là một dạng tự gây từ chối
    // dịch vụ — tệp store lớn dần và màn hình trống dài ra vô tận.
    const truoc = Array.from({ length: MAX_RECENT }, (_, i) => muc(`C:/kho/${i}`))

    const ketQua = mergeRecent(truoc, muc('C:/kho/moi'))

    expect(ketQua).toHaveLength(MAX_RECENT)
    expect(ketQua[0]?.path).toBe('C:/kho/moi')
    // Mục cũ nhất (ở cuối) là mục bị loại, không phải mục nào ở giữa.
    expect(ketQua.map((m) => m.path)).not.toContain(`C:/kho/${MAX_RECENT - 1}`)
  })

  it('không sửa đổi danh sách được truyền vào', () => {
    // Ràng buộc: danh sách trong store zustand là dữ liệu bất biến. Sửa tại chỗ
    // thì React không thấy tham chiếu đổi và giao diện không vẽ lại.
    const truoc = [muc('C:/kho/a')]

    mergeRecent(truoc, muc('C:/kho/b'))

    expect(truoc.map((m) => m.path)).toEqual(['C:/kho/a'])
  })
})

describe('sanitizeRecent', () => {
  it('trả mảng rỗng khi dữ liệu không phải mảng', () => {
    // Tệp store nằm trên đĩa người dùng (T-01-09): sửa tay được, và có thể sót
    // lại từ một phiên bản trước mang hình dạng khác.
    expect(sanitizeRecent(undefined)).toEqual([])
    expect(sanitizeRecent(null)).toEqual([])
    expect(sanitizeRecent('mot chuoi')).toEqual([])
    expect(sanitizeRecent(42)).toEqual([])
    expect(sanitizeRecent({ repos: [] })).toEqual([])
  })

  it('lọc bỏ phần tử hỏng và giữ phần còn lại', () => {
    const rac = [
      muc('C:/kho/tot'),
      { name: 'thieu path', openedAtMs: 1 },
      { path: 'C:/kho/thieu-name', openedAtMs: 1 },
      { path: 'C:/kho/thieu-thoi-diem', name: 'x' },
      { path: '', name: 'path rong', openedAtMs: 1 },
      { path: 'C:/kho/thoi-diem-khong-huu-han', name: 'y', openedAtMs: Number.NaN },
      null,
      'mot chuoi',
      muc('C:/kho/tot-2'),
    ]

    const ketQua = sanitizeRecent(rac)

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/tot', 'C:/kho/tot-2'])
  })

  it('cắt còn MAX_RECENT mục khi tệp trên đĩa dài hơn giới hạn', () => {
    // Giới hạn phải chặn ở cả hai đầu: khi ghi (`mergeRecent`) và khi đọc. Một
    // tệp sót lại từ phiên bản có MAX_RECENT lớn hơn không được vượt rào.
    const dai = Array.from({ length: MAX_RECENT + 20 }, (_, i) => muc(`C:/kho/${i}`))

    expect(sanitizeRecent(dai)).toHaveLength(MAX_RECENT)
  })

  it('khử trùng lặp dữ liệu đọc từ đĩa', () => {
    // Một tệp bị sửa tay có thể chứa hai mục cùng trỏ một repository. Giao diện
    // dùng `path` làm `key` của React, nên trùng lặp sinh cảnh báo key trùng.
    const ketQua = sanitizeRecent([muc('C:/kho/a'), muc('C:\\kho\\a'), muc('C:/kho/b')])

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/a', 'C:/kho/b'])
  })

  it('không ném lỗi với bất kỳ đầu vào rác nào', () => {
    for (const rac of [undefined, null, 0, '', [], [[]], [{}], { a: 1 }, true]) {
      expect(() => sanitizeRecent(rac)).not.toThrow()
    }
  })
})

describe('normalizeRepoPath', () => {
  it('đổi dấu gạch chéo ngược thành xuôi và cắt dấu ở cuối, khớp phía Rust', () => {
    // Đối chiếu trực tiếp với `repo_id_for` trong `src-tauri/src/state/mod.rs`:
    // `replace('\\', "/")` rồi `trim_end_matches('/')`. Cố tình **không** đổi
    // chữ thường: phía Rust không làm thế, làm ở đây là để hai bên bất đồng.
    expect(normalizeRepoPath('C:\\kho\\a')).toBe('C:/kho/a')
    expect(normalizeRepoPath('C:/kho/a/')).toBe('C:/kho/a')
    expect(normalizeRepoPath('C:\\kho\\a\\\\')).toBe('C:/kho/a')
    expect(normalizeRepoPath('/home/ai/kho')).toBe('/home/ai/kho')
  })
})

describe('loadRecentRepos', () => {
  it('trả mảng rỗng khi store chưa có tệp (lần chạy đầu)', async () => {
    moStore.mockResolvedValue(storeGia(undefined) as never)

    await expect(loadRecentRepos()).resolves.toEqual([])
  })

  it('trả mảng rỗng thay vì ném lỗi khi mở store thất bại', async () => {
    // Ràng buộc quan trọng nhất của vỏ bọc: danh sách gần đây là tiện ích, sự
    // cố của nó không được lan ra thành lỗi hiển thị cho người dùng.
    moStore.mockRejectedValue(new Error('không mở được tệp store'))

    await expect(loadRecentRepos()).resolves.toEqual([])
  })

  it('lọc dữ liệu đọc được qua sanitizeRecent', async () => {
    moStore.mockResolvedValue(
      storeGia([muc('C:/kho/tot'), { name: 'hong' }]) as never,
    )

    const ketQua = await loadRecentRepos()

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/tot'])
  })

  it('chỉ mở tệp store một lần dù gọi nhiều lần', async () => {
    moStore.mockResolvedValue(storeGia([]) as never)

    await loadRecentRepos()
    await loadRecentRepos()

    expect(moStore).toHaveBeenCalledTimes(1)
  })
})

describe('rememberRepo', () => {
  it('ghi mục mới lên đầu và trả về danh sách sau khi ghi', async () => {
    const store = storeGia([muc('C:/kho/cu')])
    moStore.mockResolvedValue(store as never)

    const ketQua = await rememberRepo({ path: 'C:/kho/moi', name: 'moi' })

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/moi', 'C:/kho/cu'])
    expect(store.set).toHaveBeenCalledTimes(1)
    const [, giaTriDaGhi] = store.set.mock.calls[0] as [string, RecentRepo[]]
    expect(giaTriDaGhi.map((m) => m.path)).toEqual(['C:/kho/moi', 'C:/kho/cu'])
  })

  it('gắn openedAtMs là thời điểm hiện tại', async () => {
    moStore.mockResolvedValue(storeGia([]) as never)
    const truoc = Date.now()

    const ketQua = await rememberRepo({ path: 'C:/kho/a', name: 'a' })

    expect(ketQua[0]?.openedAtMs).toBeGreaterThanOrEqual(truoc)
  })

  it('không ném lỗi khi việc ghi thất bại', async () => {
    // Ràng buộc: người dùng đã mở được repository rồi. Mất một mục trong danh
    // sách gần đây là chuyện nhỏ; ném lỗi lên banner mới là chuyện to.
    const store = storeGia([])
    store.set.mockRejectedValue(new Error('đĩa đầy'))
    moStore.mockResolvedValue(store as never)

    await expect(rememberRepo({ path: 'C:/kho/a', name: 'a' })).resolves.toEqual([])
  })
})

describe('forgetRepo', () => {
  it('xoá đúng mục được chỉ định và giữ nguyên các mục khác', async () => {
    const store = storeGia([muc('C:/kho/a'), muc('C:/kho/b'), muc('C:/kho/c')])
    moStore.mockResolvedValue(store as never)

    const ketQua = await forgetRepo('C:/kho/b')

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/a', 'C:/kho/c'])
  })

  it('xoá được cả khi đường dẫn truyền vào khác dạng gạch chéo', async () => {
    const store = storeGia([muc('C:/kho/a'), muc('C:/kho/b')])
    moStore.mockResolvedValue(store as never)

    const ketQua = await forgetRepo('C:\\kho\\a\\')

    expect(ketQua.map((m) => m.path)).toEqual(['C:/kho/b'])
  })

  it('không ném lỗi khi store hỏng', async () => {
    moStore.mockRejectedValue(new Error('store hỏng'))

    await expect(forgetRepo('C:/kho/a')).resolves.toEqual([])
  })
})
