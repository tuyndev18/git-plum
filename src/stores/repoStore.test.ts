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

const moRepository = vi.mocked(ipc.openRepository)
const dongRepository = vi.mocked(ipc.closeRepository)
const nhanhHienTai = vi.mocked(ipc.currentBranch)

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
  useRepoStore.setState({ byRepo: {}, activeRepoId: null, isOpening: false })
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
