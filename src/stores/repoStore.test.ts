/**
 * Test ràng buộc kiến trúc PLAT-05 — trạng thái repository theo map.
 *
 * Quyết định đắt nhất phía giao diện: dữ liệu lưu ở `byRepo[repoId]` ngay từ
 * đầu, dù v1 chỉ mở một repository tại một thời điểm. Rút gọn thành
 * `activeRepo: RepoInfo | null` ở Phase 2 sẽ nhìn như một bước "dọn dẹp" hợp lý
 * cho tới lúc phải làm nhiều tab repository. Các test dưới đây là thứ chặn bước
 * đó lại.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'
import { ipc, type RepoInfo } from '@/lib/ipc'
import { loadRecentRepos, rememberRepo } from '@/lib/recentRepos'
import { useRepoStore } from '@/stores/repoStore'

// Chặn ở ranh giới module `@/lib/ipc` chứ không vá `invoke` toàn cục: đó chính
// là lý do `ipc.ts` tồn tại như một điểm thắt nút duy nhất (T-01-06). `vi.mock`
// được cẩu lên đầu tệp nên factory không tham chiếu biến ngoài.
vi.mock('@/lib/ipc', () => ({
  ipc: {
    openRepository: vi.fn(),
    closeRepository: vi.fn(),
    currentBranch: vi.fn(),
  },
}))

// `@/lib/recentRepos` cũng bị chặn ở ranh giới module: nó nhập
// `@tauri-apps/plugin-store` ở cấp module, thứ không chạy ngoài webview Tauri.
// Test riêng cho chính nó nằm ở `src/lib/recentRepos.test.ts`.
vi.mock('@/lib/recentRepos', () => ({
  loadRecentRepos: vi.fn().mockResolvedValue([]),
  rememberRepo: vi.fn().mockResolvedValue([]),
  forgetRepo: vi.fn().mockResolvedValue([]),
}))

const moRepository = vi.mocked(ipc.openRepository)
const dongRepository = vi.mocked(ipc.closeRepository)
const nhanhHienTai = vi.mocked(ipc.currentBranch)
const ghiNho = vi.mocked(rememberRepo)
const napGanDay = vi.mocked(loadRecentRepos)

function repoInfo(id: string): RepoInfo {
  return { id, path: `C:/kho/${id}`, name: id }
}

/** Đọc slice và khẳng định nó tồn tại — `noUncheckedIndexedAccess` bắt buộc. */
function laySlice(id: string) {
  const slice = useRepoStore.getState().byRepo[id]
  expect(slice).toBeDefined()
  if (!slice) throw new Error(`không có slice cho ${id}`)
  return slice
}

/** Mở một repository qua store với IPC đã giả lập sẵn. */
async function mo(id: string, nhanh = 'main'): Promise<void> {
  moRepository.mockResolvedValueOnce(repoInfo(id))
  // `openRepository` gọi `refreshBranch` ngay sau khi thành công, nên
  // `currentBranch` phải trả promise ở mọi lần mở thành công.
  nhanhHienTai.mockResolvedValueOnce(nhanh)
  await useRepoStore.getState().openRepository(`C:/kho/${id}`)
}

// Store zustand là singleton ở phạm vi module — không reset thì test rò rỉ.
beforeEach(() => {
  vi.clearAllMocks()
  // `clearAllMocks` xoá cả giá trị trả về đã đặt trong factory của `vi.mock`,
  // nên phải dựng lại mặc định ở đây. Thiếu bước này thì `rememberRepo` trả
  // `undefined` và `openRepository` vỡ ở chỗ đọc `.length`.
  napGanDay.mockResolvedValue([])
  ghiNho.mockResolvedValue([])
  useRepoStore.setState({ byRepo: {}, activeRepoId: null, isOpening: false, recent: [] })
})

describe('mở repository', () => {
  it('đưa repository vào byRepo dưới khoá đúng bằng id từ IPC', async () => {
    // Ràng buộc PLAT-05: khoá của map là id do backend cấp, không phải đường dẫn.
    await mo('alpha')

    expect(laySlice('alpha').info).toEqual(repoInfo('alpha'))
    expect(useRepoStore.getState().activeRepoId).toBe('alpha')
  })

  it('giữ hai repository cùng tồn tại trong byRepo', async () => {
    // Ràng buộc cốt lõi PLAT-05: trạng thái là map, không phải một biến phẳng.
    // Test này là thứ ngăn ai đó rút gọn byRepo thành activeRepo ở Phase 2.
    await mo('alpha')
    await mo('beta')

    const { byRepo } = useRepoStore.getState()
    expect(Object.keys(byRepo).sort()).toEqual(['alpha', 'beta'])
    expect(laySlice('alpha').info.name).toBe('alpha')
    expect(laySlice('beta').info.name).toBe('beta')
  })

  it('trả isOpening về false và ném lỗi ra ngoài khi IPC thất bại', async () => {
    // Ràng buộc: không kẹt ở trạng thái "Đang mở…" vĩnh viễn khi mở hỏng.
    moRepository.mockRejectedValueOnce(new Error('không phải repository'))

    await expect(
      useRepoStore.getState().openRepository('C:/khong-phai-kho'),
    ).rejects.toThrow(/không phải repository/)

    expect(useRepoStore.getState().isOpening).toBe(false)
    expect(useRepoStore.getState().byRepo).toEqual({})
  })
})

describe('đóng repository', () => {
  it('chỉ gỡ repository được đóng, giữ nguyên cái còn lại', async () => {
    // Ràng buộc cốt lõi PLAT-05: các repository độc lập nhau, đóng cái này
    // không được đụng vào dữ liệu cái kia.
    await mo('alpha')
    await mo('beta')
    dongRepository.mockResolvedValueOnce(undefined)

    await useRepoStore.getState().closeRepository('alpha')

    const { byRepo } = useRepoStore.getState()
    expect(Object.keys(byRepo)).toEqual(['beta'])
    expect(laySlice('beta').info.name).toBe('beta')
  })

  it('đặt activeRepoId về null khi đóng chính repository đang hoạt động', async () => {
    // Ràng buộc: không trỏ tới một id đã biến mất khỏi map.
    await mo('alpha')
    dongRepository.mockResolvedValueOnce(undefined)

    await useRepoStore.getState().closeRepository('alpha')

    expect(useRepoStore.getState().activeRepoId).toBeNull()
  })

  it('giữ nguyên activeRepoId khi đóng repository không hoạt động', async () => {
    // Ràng buộc PLAT-05: đóng một tab nền không được kéo người dùng ra khỏi
    // repository họ đang xem.
    await mo('alpha')
    await mo('beta') // beta trở thành repository đang hoạt động
    dongRepository.mockResolvedValueOnce(undefined)

    await useRepoStore.getState().closeRepository('alpha')

    expect(useRepoStore.getState().activeRepoId).toBe('beta')
  })
})

describe('làm mới nhánh', () => {
  it('ghi tên nhánh vào slice và xoá lỗi khi thành công', async () => {
    // Ràng buộc: nhánh hiện tại thuộc về từng repository, không phải biến chung.
    await mo('alpha')
    useRepoStore.getState().setError('alpha', 'lỗi cũ')
    nhanhHienTai.mockResolvedValueOnce('phat-trien')

    await useRepoStore.getState().refreshBranch('alpha')

    expect(laySlice('alpha').currentBranch).toBe('phat-trien')
    expect(laySlice('alpha').error).toBeNull()
  })

  it('đặt currentBranch thành null và không ném lỗi khi IPC thất bại', async () => {
    // Ràng buộc: repository chưa có commit nào thì `rev-parse HEAD` thất bại —
    // đó là trạng thái bình thường, không phải sự cố cần báo động.
    await mo('alpha')
    nhanhHienTai.mockRejectedValueOnce(new Error('không có HEAD'))

    await expect(useRepoStore.getState().refreshBranch('alpha')).resolves.toBeUndefined()

    expect(laySlice('alpha').currentBranch).toBeNull()
  })

  it('bỏ qua im lặng khi làm mới một id không có trong byRepo', async () => {
    // Ràng buộc: phản hồi IPC đến muộn sau khi repository đã đóng không được
    // dựng lại một slice ma.
    nhanhHienTai.mockResolvedValueOnce('main')

    await useRepoStore.getState().refreshBranch('khong-ton-tai')

    expect(useRepoStore.getState().byRepo).toEqual({})
  })
})

describe('danh sách repository gần đây', () => {
  it('ghi nhớ repository đúng một lần sau mỗi lần mở thành công', async () => {
    // PLAT-07: nửa sau của tiêu chí thành công số 1 — lần mở sau repository
    // phải xuất hiện trong danh sách gần đây.
    await mo('alpha')

    expect(ghiNho).toHaveBeenCalledTimes(1)
    expect(ghiNho).toHaveBeenCalledWith({ path: 'C:/kho/alpha', name: 'alpha' })
  })

  it('không ghi nhớ khi việc mở repository thất bại', async () => {
    // Ràng buộc: một thư mục không phải repository không được lọt vào danh
    // sách gần đây, nếu không người dùng bấm lại nó và lại nhận đúng lỗi cũ.
    moRepository.mockRejectedValueOnce(new Error('không phải repository'))

    await expect(useRepoStore.getState().openRepository('C:/rac')).rejects.toThrow()

    expect(ghiNho).not.toHaveBeenCalled()
  })

  it('hoàn tất việc mở repository bình thường khi rememberRepo ném lỗi', async () => {
    // Ràng buộc quan trọng nhất của PLAT-07. Danh sách gần đây là tiện ích:
    // người dùng đã mở được repository rồi, sự cố của một tiện ích không được
    // biến thành lỗi hiển thị cho một thao tác đã thành công.
    ghiNho.mockRejectedValueOnce(new Error('đĩa đầy'))
    moRepository.mockResolvedValueOnce(repoInfo('alpha'))
    nhanhHienTai.mockResolvedValueOnce('main')

    await expect(
      useRepoStore.getState().openRepository('C:/kho/alpha'),
    ).resolves.toBeUndefined()

    expect(laySlice('alpha').info.name).toBe('alpha')
    expect(useRepoStore.getState().activeRepoId).toBe('alpha')
    expect(useRepoStore.getState().isOpening).toBe(false)
  })

  it('cập nhật recent bằng danh sách mà rememberRepo trả về', async () => {
    ghiNho.mockResolvedValueOnce([
      { path: 'C:/kho/alpha', name: 'alpha', openedAtMs: 123 },
    ])
    moRepository.mockResolvedValueOnce(repoInfo('alpha'))
    nhanhHienTai.mockResolvedValueOnce('main')

    await useRepoStore.getState().openRepository('C:/kho/alpha')

    expect(useRepoStore.getState().recent.map((m) => m.path)).toEqual(['C:/kho/alpha'])
  })

  it('giữ nguyên danh sách đang hiển thị khi rememberRepo trả mảng rỗng', async () => {
    // Mảng rỗng là tín hiệu ghi thất bại (vỏ bọc nuốt lỗi và trả `[]`). Xoá
    // sạch danh sách đang hiển thị vì một lần ghi hỏng là mất dữ liệu người
    // dùng nhìn thấy, tệ hơn hẳn việc để nó hơi cũ một nhịp.
    useRepoStore.setState({
      recent: [{ path: 'C:/kho/cu', name: 'cu', openedAtMs: 1 }],
    })
    ghiNho.mockResolvedValueOnce([])

    await mo('alpha')

    expect(useRepoStore.getState().recent.map((m) => m.path)).toEqual(['C:/kho/cu'])
  })

  it('loadRecent đưa đúng danh sách đọc được vào recent', async () => {
    napGanDay.mockResolvedValueOnce([
      { path: 'C:/kho/a', name: 'a', openedAtMs: 2 },
      { path: 'C:/kho/b', name: 'b', openedAtMs: 1 },
    ])

    await useRepoStore.getState().loadRecent()

    expect(useRepoStore.getState().recent.map((m) => m.path)).toEqual([
      'C:/kho/a',
      'C:/kho/b',
    ])
  })
})

describe('đặt lỗi', () => {
  it('chỉ gắn lỗi vào slice được chỉ định, không đụng slice khác', async () => {
    // Ràng buộc PLAT-05: lỗi thuộc về từng repository. Một biến lỗi chung nghĩa
    // là sự cố ở repository nền hiện lên trên repository đang xem.
    await mo('alpha')
    await mo('beta')

    useRepoStore.getState().setError('alpha', 'không lấy được nhánh')

    expect(laySlice('alpha').error).toBe('không lấy được nhánh')
    expect(laySlice('beta').error).toBeNull()
  })
})
