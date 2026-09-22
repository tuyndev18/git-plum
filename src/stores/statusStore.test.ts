/**
 * Test cho `statusStore` — WORK-01, WORK-02, PLAT-05.
 *
 * Cổng chịu lực của tệp này là `duong_stage_KHONG_goi_getStatus` (đột biến M2). Nó
 * ghim ràng buộc 2.5 của CONTEXT.md ở tầng giao diện: lệnh ghi đã trả trạng thái mới,
 * nên đường stage **không** được đọc lại. Gọi thêm là một lệnh git thừa cho **mỗi** cú
 * bấm, và nó xoá mất cả điểm của việc lệnh ghi trả trạng thái.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

import { ipc, type RepoStatus, type StatusEntry } from '@/lib/ipc'
import { useStatusStore } from './statusStore'

vi.mock('@/lib/ipc', async () => {
  const thuc = await vi.importActual<typeof import('@/lib/ipc')>('@/lib/ipc')
  return {
    ...thuc,
    ipc: {
      getStatus: vi.fn(),
      stageFiles: vi.fn(),
      unstageFiles: vi.fn(),
      getWorktreeDiff: vi.fn(),
    },
  }
})

const getStatusMock = vi.mocked(ipc.getStatus)
const stageMock = vi.mocked(ipc.stageFiles)
const unstageMock = vi.mocked(ipc.unstageFiles)

function muc(path: string, xy: string, group: StatusEntry['group']): StatusEntry {
  return { path, oldPath: null, xy, group, hasInvalidUtf8: false }
}

function status(entries: StatusEntry[]): RepoStatus {
  return {
    branch: { head: 'main', oid: 'a'.repeat(40), upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

const TRUOC = status([muc('a.txt', '.M', 'unstaged'), muc('b.txt', 'M.', 'staged')])
const SAU_STAGE = status([muc('a.txt', 'M.', 'staged'), muc('b.txt', 'M.', 'staged')])

beforeEach(() => {
  // `restoreMocks: true` gỡ mọi `mockResolvedValue` giữa các test — dựng lại là bắt
  // buộc, không phải cẩn thận thừa (cùng ghi chú với `ipc.history.test.ts`).
  getStatusMock.mockReset()
  stageMock.mockReset()
  unstageMock.mockReset()
  getStatusMock.mockResolvedValue(TRUOC)
  stageMock.mockResolvedValue(SAU_STAGE)
  unstageMock.mockResolvedValue(TRUOC)

  useStatusStore.setState({ byRepo: {} })
})

describe('refresh', () => {
  it('nạp status vào byRepo theo repoId', async () => {
    await useStatusStore.getState().refresh('repo-1')

    expect(getStatusMock).toHaveBeenCalledWith('repo-1')
    expect(useStatusStore.getState().byRepo['repo-1']?.status).toEqual(TRUOC)
    expect(useStatusStore.getState().byRepo['repo-1']?.isLoading).toBe(false)
    expect(useStatusStore.getState().byRepo['repo-1']?.error).toBeNull()
  })

  /**
   * 🔴 Trạng thái khoá theo repo (PLAT-05): nạp repo B không được đụng repo A.
   *
   * Một `RepoStatus` duy nhất ở cấp store làm người dùng thấy danh sách tệp của repo A
   * sau khi chuyển sang repo B, và ở phase này họ có thể **bấm stage** trên nó.
   */
  it('hai repo giữ trạng thái độc lập', async () => {
    const khac = status([muc('chi-co-o-b.txt', '??', 'untracked')])

    getStatusMock.mockResolvedValueOnce(TRUOC)
    await useStatusStore.getState().refresh('repo-a')
    getStatusMock.mockResolvedValueOnce(khac)
    await useStatusStore.getState().refresh('repo-b')

    expect(useStatusStore.getState().byRepo['repo-a']?.status).toEqual(TRUOC)
    expect(useStatusStore.getState().byRepo['repo-b']?.status).toEqual(khac)
  })

  /**
   * 🔴 Hai `refresh` **đồng thời** cho cùng repo → **một** lời gọi IPC.
   *
   * Khuôn "đang bay" của `historyStore.ensureRange`. Ca thật: `ChangeList` mount và
   * một `useEffect` của panel cha cùng gọi trong một khung render.
   *
   * Phép đo phải **không** `await` lời gọi đầu trước khi gọi lời gọi thứ hai — `await`
   * giữa hai lời gọi làm lời đầu xong hẳn và test sẽ xanh kể cả khi không có phép
   * chống trùng nào.
   */
  it('hai refresh đồng thời chỉ gọi IPC một lần', async () => {
    let giai!: (v: RepoStatus) => void
    getStatusMock.mockReturnValueOnce(
      new Promise<RepoStatus>((r) => {
        giai = r
      }),
    )

    const mot = useStatusStore.getState().refresh('repo-1')
    const hai = useStatusStore.getState().refresh('repo-1')

    expect(getStatusMock).toHaveBeenCalledTimes(1)

    giai(TRUOC)
    await Promise.all([mot, hai])

    expect(getStatusMock).toHaveBeenCalledTimes(1)
    expect(useStatusStore.getState().byRepo['repo-1']?.status).toEqual(TRUOC)
  })

  /** Sau khi lời gọi đầu **xong**, một `refresh` mới phải gọi IPC lại. */
  it('refresh sau khi lời gọi trước đã xong thì gọi IPC lần nữa', async () => {
    await useStatusStore.getState().refresh('repo-1')
    await useStatusStore.getState().refresh('repo-1')

    expect(getStatusMock).toHaveBeenCalledTimes(2)
  })

  it('lỗi được ghi vào error và isLoading về false', async () => {
    getStatusMock.mockRejectedValueOnce(new Error('git hong'))

    await useStatusStore.getState().refresh('repo-1')

    expect(useStatusStore.getState().byRepo['repo-1']?.error).toContain('git hong')
    expect(useStatusStore.getState().byRepo['repo-1']?.isLoading).toBe(false)
  })

  /** Lời gọi thất bại phải **giải phóng** khoá "đang bay", nếu không repo kẹt vĩnh viễn. */
  it('refresh thất bại rồi refresh lại vẫn gọi được IPC', async () => {
    getStatusMock.mockRejectedValueOnce(new Error('hong'))
    await useStatusStore.getState().refresh('repo-1')

    getStatusMock.mockResolvedValueOnce(TRUOC)
    await useStatusStore.getState().refresh('repo-1')

    expect(getStatusMock).toHaveBeenCalledTimes(2)
    expect(useStatusStore.getState().byRepo['repo-1']?.status).toEqual(TRUOC)
  })
})

describe('stage / unstage', () => {
  it('stage gọi stageFiles với đúng repoId và paths', async () => {
    await useStatusStore.getState().stage('repo-1', ['a.txt'])

    expect(stageMock).toHaveBeenCalledWith('repo-1', ['a.txt'])
  })

  /**
   * 🔴 **Ghi `RepoStatus` TỪ GIÁ TRỊ TRẢ VỀ** của `stageFiles`.
   *
   * Không phải từ một lần đọc lại. Test này đỏ nếu đường stage bỏ qua giá trị trả về.
   */
  it('stage ghi RepoStatus từ giá trị trả về của stageFiles', async () => {
    await useStatusStore.getState().refresh('repo-1')
    expect(useStatusStore.getState().byRepo['repo-1']?.status).toEqual(TRUOC)

    await useStatusStore.getState().stage('repo-1', ['a.txt'])

    expect(useStatusStore.getState().byRepo['repo-1']?.status).toEqual(SAU_STAGE)
    expect(
      useStatusStore.getState().byRepo['repo-1']?.status?.entries.find((e) => e.path === 'a.txt')
        ?.group,
    ).toBe('staged')
  })

  /**
   * 🔴 **Đột biến M2: đường stage KHÔNG gọi `getStatus`.**
   *
   * Lệnh ghi phía Rust đã trả trạng thái mới (kiểu trả về của nó là
   * `Result<RepoStatus>` chính vì điều này). Gọi `refresh` sau đó là một lệnh `git
   * status` thừa cho **mỗi** cú bấm, và nó xoá mất cả điểm của việc lệnh ghi trả trạng
   * thái — lần "dọn dẹp" sau sẽ đổi kiểu trả về về `void` mà không test nào đỏ.
   *
   * Đo `toHaveBeenCalledTimes(0)` sau khi **đã dọn** hàm giả, chứ không đo số lần gọi
   * tổng: đo tổng sẽ lẫn với lần `refresh` hợp lệ ở bước dựng bối cảnh.
   */
  it('đường stage KHÔNG gọi getStatus', async () => {
    await useStatusStore.getState().refresh('repo-1')
    getStatusMock.mockClear()

    await useStatusStore.getState().stage('repo-1', ['a.txt'])

    expect(getStatusMock).toHaveBeenCalledTimes(0)
    expect(stageMock).toHaveBeenCalledTimes(1)
  })

  /** Cùng ràng buộc cho đường unstage. */
  it('đường unstage KHÔNG gọi getStatus', async () => {
    await useStatusStore.getState().refresh('repo-1')
    getStatusMock.mockClear()

    await useStatusStore.getState().unstage('repo-1', ['b.txt'])

    expect(getStatusMock).toHaveBeenCalledTimes(0)
    expect(unstageMock).toHaveBeenCalledTimes(1)
  })

  /**
   * 🔴 Stage **thất bại** → trạng thái cũ **giữ nguyên**, lỗi hiện ra.
   *
   * Xoá `status` khi thất bại làm danh sách biến mất dưới tay người dùng vì một lỗi
   * tạm thời (`index.lock` là ca thường — họ đang chạy git ở terminal, R3), và họ mất
   * cả chỗ đang đứng lẫn khả năng thử lại.
   */
  it('stage thất bại giữ nguyên status cũ và ghi lỗi', async () => {
    await useStatusStore.getState().refresh('repo-1')
    stageMock.mockRejectedValueOnce({
      code: 'index_locked',
      message: 'một tiến trình git khác đang giữ .git/index.lock',
      command: 'git add -- a.txt',
    })

    await useStatusStore.getState().stage('repo-1', ['a.txt'])

    const lat = useStatusStore.getState().byRepo['repo-1']
    expect(lat?.status).toEqual(TRUOC)
    expect(lat?.error).toContain('index.lock')
    expect(lat?.isLoading).toBe(false)
  })

  /** Stage trên một repo chưa từng `refresh` vẫn phải chạy được. */
  it('stage không cần refresh trước đó', async () => {
    await useStatusStore.getState().stage('repo-moi', ['a.txt'])

    expect(useStatusStore.getState().byRepo['repo-moi']?.status).toEqual(SAU_STAGE)
    expect(getStatusMock).toHaveBeenCalledTimes(0)
  })
})

describe('applyExternal', () => {
  /**
   * Điểm vào cho watcher của 04-04 — nó ghi trạng thái **mà không** gọi IPC.
   *
   * Khai ở wave này để 04-04 chỉ phải nối dây. Gộp nó vào `refresh` buộc watcher sinh
   * thêm một lệnh `git status` cho một trạng thái nó vừa đọc xong.
   */
  it('ghi status mà không gọi IPC nào', () => {
    useStatusStore.getState().applyExternal('repo-1', SAU_STAGE)

    expect(useStatusStore.getState().byRepo['repo-1']?.status).toEqual(SAU_STAGE)
    expect(getStatusMock).toHaveBeenCalledTimes(0)
    expect(stageMock).toHaveBeenCalledTimes(0)
  })

  it('xoá lỗi cũ: thay đổi từ bên ngoài nghĩa là repo lại đọc được', async () => {
    getStatusMock.mockRejectedValueOnce(new Error('hong'))
    await useStatusStore.getState().refresh('repo-1')
    expect(useStatusStore.getState().byRepo['repo-1']?.error).not.toBeNull()

    useStatusStore.getState().applyExternal('repo-1', TRUOC)

    expect(useStatusStore.getState().byRepo['repo-1']?.error).toBeNull()
  })
})

describe('reset', () => {
  it('xoá slice của một repo, giữ repo khác', async () => {
    await useStatusStore.getState().refresh('repo-a')
    await useStatusStore.getState().refresh('repo-b')

    useStatusStore.getState().reset('repo-a')

    expect(useStatusStore.getState().byRepo['repo-a']).toBeUndefined()
    expect(useStatusStore.getState().byRepo['repo-b']?.status).toBeDefined()
  })
})
