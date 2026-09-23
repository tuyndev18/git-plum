/**
 * Test cho `hunkStore` — WORK-03, WORK-05, WORK-06.
 *
 * # Hai cổng chịu lực của tệp này
 *
 * 1. `duong_stage_hunk_KHONG_goi_getStatus` — khuôn của đột biến **M2** ở 04-02, áp
 *    cho đường khối. Lệnh ghi phía Rust trả `RepoStatus` **mới** (kiểu trả về của
 *    `stage_hunk` là `Result<RepoStatus>`, không phải `Result<()>`, chính vì điều
 *    này). Đọc lại bằng `getStatus` là một lệnh git thừa cho **mỗi** cú bấm, và nó
 *    xoá mất cả điểm của kiểu trả về kia.
 *
 * 2. `loi_khac_file_changed_KHONG_dat_canLamMoi` — **Test 4** của plan. Không có nó,
 *    một cài đặt đặt `canLamMoi` cho **mọi** lỗi vẫn xanh ở Test 3. Đó là lỗi #4 của
 *    `CONTEXT.md` §4.1: fixture **không phân biệt được** đột biến. Hai test phải tồn
 *    tại cùng nhau hoặc cả hai đều vô nghĩa.
 *
 * # Khoá JSON được khẳng định **bằng nhau** với phía Rust
 *
 * `muc_thung_rac_khoa_json_day_du` ở `src-tauri/src/domain/trash.rs` liệt kê toàn bộ
 * khoá phía Rust. Ở đây ta khẳng định danh sách TypeScript **bằng** danh sách đó —
 * không phải `contains`. Lệch tên khoá **không gây lỗi biên dịch ở bên nào**; nó chỉ
 * làm trường thành `undefined` lúc chạy, đúng lỗi `oldSize`/`old_size` của Phase 3.
 */

import { describe, it, expect, beforeEach, vi } from 'vitest'

import {
  ipc,
  type GitErrorPayload,
  type MucThungRac,
  type RepoStatus,
  type StatusEntry,
} from '@/lib/ipc'
import { useStatusStore } from './statusStore'
import { useHunkStore } from './hunkStore'

vi.mock('@/lib/ipc', async () => {
  const thuc = await vi.importActual<typeof import('@/lib/ipc')>('@/lib/ipc')
  return {
    ...thuc,
    ipc: {
      getStatus: vi.fn(),
      stageFiles: vi.fn(),
      unstageFiles: vi.fn(),
      getWorktreeDiff: vi.fn(),
      stageHunk: vi.fn(),
      unstageHunk: vi.fn(),
      discardHunk: vi.fn(),
      discardFiles: vi.fn(),
      listTrash: vi.fn(),
      restoreTrash: vi.fn(),
    },
  }
})

const getStatusMock = vi.mocked(ipc.getStatus)
const stageHunkMock = vi.mocked(ipc.stageHunk)
const unstageHunkMock = vi.mocked(ipc.unstageHunk)
const discardHunkMock = vi.mocked(ipc.discardHunk)

const REPO = 'repo-1'
const PATH = 'src/a.txt'
const HASH = 'a'.repeat(40)

function muc(path: string, xy: string, group: StatusEntry['group']): StatusEntry {
  return { path, oldPath: null, xy, group, hasInvalidUtf8: false }
}

function status(entries: StatusEntry[]): RepoStatus {
  return {
    branch: { head: 'main', oid: 'b'.repeat(40), upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

const TRUOC = status([muc(PATH, '.M', 'unstaged')])
const SAU = status([muc(PATH, 'MM', 'staged')])

/** Lỗi WORK-05 đúng hình dạng Rust ném ra — mã `file_changed`, câu nguyên văn ROADMAP. */
function loiFileChanged(): GitErrorPayload {
  return {
    code: 'file_changed',
    message: 'Tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới',
    command: 'git hash-object -- src/a.txt',
  }
}

beforeEach(() => {
  getStatusMock.mockReset()
  stageHunkMock.mockReset()
  unstageHunkMock.mockReset()
  discardHunkMock.mockReset()

  getStatusMock.mockResolvedValue(TRUOC)
  stageHunkMock.mockResolvedValue(SAU)
  unstageHunkMock.mockResolvedValue(TRUOC)
  discardHunkMock.mockResolvedValue(TRUOC)

  useStatusStore.setState({ byRepo: {} })
  useHunkStore.setState({ dangChon: null, canLamMoi: null })
})

// --- Test 1 ---------------------------------------------------------------

describe('gọi IPC', () => {
  it('stageHunk gọi ipc.stageHunk ĐÚNG MỘT LẦN với đúng tham số', async () => {
    await useHunkStore.getState().stageHunk(REPO, PATH, 2, HASH)

    expect(stageHunkMock).toHaveBeenCalledTimes(1)
    expect(stageHunkMock).toHaveBeenCalledWith(REPO, PATH, 2, HASH)
    // 🔴 `index` khác 0: một cài đặt hardcode `0` không phân biệt được với
    // `index: 0` — lớp lỗi "hình dạng đúng, dữ liệu vô hại" (CONTEXT.md 4.2).
    expect(stageHunkMock.mock.calls[0]?.[2]).toBe(2)
  })

  it('unstageHunk và discardHunk gọi đúng lệnh của mình, không lẫn sang lệnh kia', async () => {
    await useHunkStore.getState().unstageHunk(REPO, PATH, 3, HASH)
    expect(unstageHunkMock).toHaveBeenCalledTimes(1)
    expect(unstageHunkMock).toHaveBeenCalledWith(REPO, PATH, 3, HASH)
    expect(discardHunkMock).toHaveBeenCalledTimes(0)
    expect(stageHunkMock).toHaveBeenCalledTimes(0)

    await useHunkStore.getState().discardHunk(REPO, PATH, 4, HASH)
    expect(discardHunkMock).toHaveBeenCalledTimes(1)
    expect(discardHunkMock).toHaveBeenCalledWith(REPO, PATH, 4, HASH)
    expect(unstageHunkMock).toHaveBeenCalledTimes(1)
  })
})

// --- Test 2 — khuôn đột biến M2 của 04-02 ---------------------------------

describe('🔴 ràng buộc 2.5: ghi status TỪ GIÁ TRỊ TRẢ VỀ', () => {
  it('stageHunk thành công ghi RepoStatus trả về vào statusStore qua applyExternal', async () => {
    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)

    expect(useStatusStore.getState().byRepo[REPO]?.status).toEqual(SAU)
    expect(useStatusStore.getState().byRepo[REPO]?.error).toBeNull()
  })

  /**
   * Đột biến **M2 của 04-02**, áp cho đường khối.
   *
   * Nếu 0 đỏ thì đường khối đang đọc lại status sau mỗi cú bấm — một `git status`
   * thừa cho mỗi khối người dùng stage, và kiểu trả về `Result<RepoStatus>` phía
   * Rust thành trang trí.
   */
  it('đường stage/unstage/discard khối KHÔNG gọi getStatus', async () => {
    getStatusMock.mockClear()

    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)
    expect(getStatusMock).toHaveBeenCalledTimes(0)

    await useHunkStore.getState().unstageHunk(REPO, PATH, 1, HASH)
    expect(getStatusMock).toHaveBeenCalledTimes(0)

    await useHunkStore.getState().discardHunk(REPO, PATH, 1, HASH)
    expect(getStatusMock).toHaveBeenCalledTimes(0)
  })
})

// --- Test 3 — WORK-05 -----------------------------------------------------

describe('🔴 WORK-05: ca `file_changed`', () => {
  it('file_changed → canLamMoi có path đúng và message chứa nguyên văn "hãy làm mới"', async () => {
    stageHunkMock.mockRejectedValueOnce(loiFileChanged())

    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)

    const can = useHunkStore.getState().canLamMoi
    expect(can).not.toBeNull()
    expect(can?.path).toBe(PATH)
    // 🔴 Nguyên văn câu ROADMAP. Đổi nó là đổi hợp đồng với người dùng.
    expect(can?.message).toContain('hãy làm mới')
  })

  it('file_changed → statusStore KHÔNG bị ghi (không có gì lọt vào)', async () => {
    stageHunkMock.mockRejectedValueOnce(loiFileChanged())

    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)

    expect(useStatusStore.getState().byRepo[REPO]?.status).toBeUndefined()
  })

  it('xoaCanLamMoi gỡ thông báo', async () => {
    stageHunkMock.mockRejectedValueOnce(loiFileChanged())
    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)
    expect(useHunkStore.getState().canLamMoi).not.toBeNull()

    useHunkStore.getState().xoaCanLamMoi()
    expect(useHunkStore.getState().canLamMoi).toBeNull()
  })

  it('đường unstage và discard cũng đặt canLamMoi cho file_changed', async () => {
    unstageHunkMock.mockRejectedValueOnce(loiFileChanged())
    await useHunkStore.getState().unstageHunk(REPO, PATH, 1, HASH)
    expect(useHunkStore.getState().canLamMoi?.path).toBe(PATH)

    useHunkStore.getState().xoaCanLamMoi()

    discardHunkMock.mockRejectedValueOnce(loiFileChanged())
    await useHunkStore.getState().discardHunk(REPO, PATH, 1, HASH)
    expect(useHunkStore.getState().canLamMoi?.path).toBe(PATH)
  })
})

// --- Test 4 — fixture phân biệt được (lỗi #4) -----------------------------

describe('🔴 lỗi khác file_changed KHÔNG đặt canLamMoi', () => {
  /**
   * Không có test này, một cài đặt `catch (e) { set({ canLamMoi: ... }) }` — đặt cờ
   * cho **mọi** lỗi — vẫn xanh ở Test 3. Đó đúng là lỗi #4 của `CONTEXT.md` §4.1.
   *
   * `index_locked` được chọn có chủ ý: nó là lỗi **tạm thời** người dùng gặp thường
   * xuyên (họ chạy git ở terminal — cả điểm của WORK-10). Hiện "tệp đã đổi, hãy làm
   * mới" cho một `index.lock` là nói sai nguyên nhân, và người dùng sẽ đi tìm một
   * thay đổi không tồn tại.
   */
  it('index_locked KHÔNG đặt canLamMoi', async () => {
    const loi: GitErrorPayload = {
      code: 'index_locked',
      message: 'Đang chờ một tiến trình git khác, thử lại sau',
      command: 'git apply --cached --recount',
    }
    stageHunkMock.mockRejectedValueOnce(loi)

    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)

    expect(useHunkStore.getState().canLamMoi).toBeNull()
  })

  it('lỗi JavaScript thường (không phải GitErrorPayload) KHÔNG đặt canLamMoi', async () => {
    stageHunkMock.mockRejectedValueOnce(new Error('mạng chết'))

    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)

    expect(useHunkStore.getState().canLamMoi).toBeNull()
  })

  /**
   * 🔴 Phân nhánh theo `code`, **không** theo chuỗi `message`.
   *
   * Một lỗi mã **khác** nhưng mang đúng câu "hãy làm mới" trong `message` không được
   * đặt `canLamMoi`. Nếu test này đỏ thì cài đặt đang so khớp chuỗi, và nó sẽ vỡ lần
   * đầu ai đó sửa một dấu câu trong thông điệp Rust.
   */
  it('mã khác nhưng message chứa "hãy làm mới" vẫn KHÔNG đặt canLamMoi', async () => {
    const loi: GitErrorPayload = {
      code: 'command_failed',
      message: 'một thứ gì đó, hãy làm mới',
      command: null,
    }
    stageHunkMock.mockRejectedValueOnce(loi)

    await useHunkStore.getState().stageHunk(REPO, PATH, 1, HASH)

    expect(useHunkStore.getState().canLamMoi).toBeNull()
  })
})

// --- Test 5 — chọn khối ---------------------------------------------------

describe('chọn khối', () => {
  it('chon đặt dangChon dạng `path#index`; boChon gỡ về null', () => {
    useHunkStore.getState().chon(PATH, 2)
    expect(useHunkStore.getState().dangChon).toBe(`${PATH}#2`)

    useHunkStore.getState().boChon()
    expect(useHunkStore.getState().dangChon).toBeNull()
  })

  /**
   * Chọn khối khác **thay thế** khối cũ, không tích luỹ.
   *
   * Một cài đặt dùng `Set` hay mảng sẽ để hai khối cùng "đang chọn", và giao diện sẽ
   * tô sáng hai chỗ trong khi người dùng chỉ bấm một.
   */
  it('chọn khối khác THAY THẾ khối cũ, không tích luỹ', () => {
    useHunkStore.getState().chon(PATH, 0)
    useHunkStore.getState().chon(PATH, 5)
    expect(useHunkStore.getState().dangChon).toBe(`${PATH}#5`)

    useHunkStore.getState().chon('src/b.txt', 1)
    expect(useHunkStore.getState().dangChon).toBe('src/b.txt#1')
  })

  it('index khác nhau trên cùng tệp cho khoá khác nhau', () => {
    useHunkStore.getState().chon(PATH, 1)
    const a = useHunkStore.getState().dangChon
    useHunkStore.getState().chon(PATH, 11)
    const b = useHunkStore.getState().dangChon

    // 🔴 `1` và `11` là ca thật: một dấu phân cách thiếu biến `a#1` + `1` thành `a#11`.
    expect(a).not.toBe(b)
  })
})

// --- Hợp đồng khoá JSON với Rust -----------------------------------------

describe('🔴 khoá JSON khớp phía Rust', () => {
  /**
   * So **bằng**, không `contains`. Danh sách bên phải chép từ
   * `muc_thung_rac_khoa_json_day_du` ở `src-tauri/src/domain/trash.rs`, nơi cùng phép
   * so bằng chạy trên `serde_json::to_value`.
   *
   * Một khoá **thừa** hay **thiếu** đều là lỗi, và `contains` chỉ bắt được khoá thiếu.
   */
  it('MucThungRac có ĐÚNG sáu khoá, bằng danh sách phía Rust', () => {
    const m: MucThungRac = {
      refName: 'refs/git-plum-trash/1-0',
      objectId: 'deadbeef',
      laBlob: false,
      paths: ['a.txt'],
      luc: 42,
      nhan: 'Huỷ bỏ 1 tệp',
    }

    const khoa = Object.keys(m).sort()

    expect(khoa).toEqual(['laBlob', 'luc', 'nhan', 'objectId', 'paths', 'refName'])
  })
})
