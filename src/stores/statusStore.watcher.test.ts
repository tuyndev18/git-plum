/**
 * Test dây nối watcher `.git` → `statusStore.applyExternal` — WORK-10.
 *
 * Tệp **riêng** khỏi `statusStore.test.ts` có chủ ý: tệp kia mock `ipc` thành một
 * object chỉ có bốn hàm, nên `ngheTrangThaiNgoai` (một export **cấp module**, không
 * nằm trong `ipc`) sẽ không tồn tại ở đó. Nhồi thêm vào mock cũ là sửa một tệp đang
 * xanh để phục vụ một tính năng khác — tách ra rẻ hơn.
 *
 * 🔴 Ba thứ được ghim ở đây, và cái thứ ba là cái đắt nhất:
 *
 * 1. Sự kiện đến → `applyExternal` được gọi với đúng `repoId`.
 * 2. Đường này dùng `applyExternal`, **không** `refresh` — payload đã mang sẵn
 *    `RepoStatus`, nên `refresh` là một lệnh `git status` thừa và còn **chậm hơn**.
 * 3. **Tên sự kiện khớp hai phía Rust/TS.** Lệch tên là lỗi im lặng hoàn toàn:
 *    `listen` chỉ đơn giản không bao giờ chạy — không lỗi, không cảnh báo, chỉ một
 *    giao diện đứng im. Cùng khuôn test hai phía với `MAX_VISIBLE_LANES`.
 */

/// <reference types="node" />
//
// Chỉ tệp test này cần kiểu Node (`readFileSync`) cho phép kiểm hai phía — cùng lý do
// và cùng khuôn với `src/styles/app.css.test.ts`.
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, it, expect, beforeEach, vi } from 'vitest'

import { ipc, type RepoStatus, type TrangThaiNgoai } from '@/lib/ipc'
import { noiWatcherVaoStore, useStatusStore } from './statusStore'

/** Handler mà `ngheTrangThaiNgoai` đã đăng ký, để test tự bắn sự kiện vào. */
let handlerDaDangKy: ((p: TrangThaiNgoai) => void) | undefined
const huyDangKy = vi.fn()

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
    ngheTrangThaiNgoai: vi.fn(),
  }
})

const { ngheTrangThaiNgoai } = await import('@/lib/ipc')
const ngheMock = vi.mocked(ngheTrangThaiNgoai)
const getStatusMock = vi.mocked(ipc.getStatus)

function status(head: string): RepoStatus {
  return {
    branch: { head, oid: 'a'.repeat(40), upstream: null, ahead: null, behind: null },
    entries: [],
    hasConflicts: false,
  }
}

beforeEach(() => {
  handlerDaDangKy = undefined
  huyDangKy.mockReset()
  getStatusMock.mockReset()
  getStatusMock.mockResolvedValue(status('khong-bao-gio-duoc-goi'))

  ngheMock.mockReset()
  ngheMock.mockImplementation(async (xuLy) => {
    handlerDaDangKy = xuLy
    return huyDangKy
  })

  useStatusStore.setState({ byRepo: {} })
})

describe('noiWatcherVaoStore', () => {
  it('đăng ký một handler nghe sự kiện trạng thái ngoài', async () => {
    await noiWatcherVaoStore()
    expect(ngheMock).toHaveBeenCalledTimes(1)
    expect(handlerDaDangKy, 'tiền đề: phải đăng ký được một handler').toBeTypeOf('function')
  })

  it('sự kiện đến → trạng thái của ĐÚNG repo được cập nhật', async () => {
    await noiWatcherVaoStore()
    if (!handlerDaDangKy) throw new Error('tiền đề: chưa đăng ký handler')

    handlerDaDangKy({ repoId: 'repo-a', status: status('nhanh-moi') })

    expect(useStatusStore.getState().byRepo['repo-a']?.status?.branch.head).toBe('nhanh-moi')
  })

  it('sự kiện của repo A KHÔNG đụng trạng thái của repo B', async () => {
    useStatusStore.setState({
      byRepo: { 'repo-b': { status: status('cua-b'), isLoading: false, error: null } },
    })
    await noiWatcherVaoStore()
    if (!handlerDaDangKy) throw new Error('tiền đề: chưa đăng ký handler')

    handlerDaDangKy({ repoId: 'repo-a', status: status('cua-a') })

    expect(useStatusStore.getState().byRepo['repo-a']?.status?.branch.head).toBe('cua-a')
    expect(
      useStatusStore.getState().byRepo['repo-b']?.status?.branch.head,
      'PLAT-05: sự kiện khoá theo repoId, không được ghi đè repo khác',
    ).toBe('cua-b')
  })

  /**
   * 🔴 Đường watcher dùng `applyExternal`, **không** `refresh`.
   *
   * Cùng lập luận với đột biến M2 của 04-02: payload đã mang sẵn `RepoStatus` mà Rust
   * vừa đọc, nên gọi `getStatus` ở đây là một tiến trình git **thừa** cho mỗi sự kiện
   * — và nó chạy **sau** 250–300 ms trì hoãn nên còn chậm hơn dữ liệu đã có trong tay.
   */
  it('KHÔNG gọi getStatus — payload đã mang sẵn trạng thái', async () => {
    await noiWatcherVaoStore()
    if (!handlerDaDangKy) throw new Error('tiền đề: chưa đăng ký handler')

    handlerDaDangKy({ repoId: 'repo-a', status: status('nhanh-moi') })

    expect(
      getStatusMock,
      'watcher đã đọc trạng thái rồi; đọc lại là một lệnh git thừa cho MỖI sự kiện',
    ).not.toHaveBeenCalled()
  })

  it('trả hàm huỷ đăng ký — bỏ sót là handler tích luỹ mỗi lần mount', async () => {
    const huy = await noiWatcherVaoStore()
    expect(huy).toBeTypeOf('function')
    huy()
    expect(huyDangKy).toHaveBeenCalledTimes(1)
  })

  it('sự kiện xoá được cờ lỗi cũ — trạng thái mới là trạng thái mới', async () => {
    useStatusStore.setState({
      byRepo: { 'repo-a': { status: undefined, isLoading: true, error: 'index.lock' } },
    })
    await noiWatcherVaoStore()
    if (!handlerDaDangKy) throw new Error('tiền đề: chưa đăng ký handler')

    handlerDaDangKy({ repoId: 'repo-a', status: status('ok') })

    const lat = useStatusStore.getState().byRepo['repo-a']
    expect(lat?.error).toBeNull()
    expect(lat?.isLoading).toBe(false)
  })
})

// --- Hợp đồng hai phía Rust ↔ TS -------------------------------------------

describe('🔴 tên sự kiện khớp HAI phía Rust và TS', () => {
  const nguonRust = path.join(process.cwd(), 'src-tauri', 'src', 'watch', 'mod.rs')

  it('TIỀN ĐỀ: đọc được watch/mod.rs và nó khai hằng tên sự kiện', () => {
    // Lỗi #3 của CONTEXT.md 3.1: đường dẫn sai → không tìm thấy gì → xanh.
    // Không tìm thấy = ĐỎ.
    const src = readFileSync(nguonRust, 'utf8')
    expect(src.length, 'watch/mod.rs không được rỗng').toBeGreaterThan(200)
    expect(src).toContain('SU_KIEN_TRANG_THAI_NGOAI')
  })

  it('hằng bên Rust bằng ĐÚNG hằng bên TS', async () => {
    const src = readFileSync(nguonRust, 'utf8')
    const khop = src.match(
      /pub const SU_KIEN_TRANG_THAI_NGOAI:\s*&str\s*=\s*"([^"]+)"/,
    )
    expect(khop, 'phải trích được giá trị hằng từ nguồn Rust').toBeTruthy()

    const { SU_KIEN_TRANG_THAI_NGOAI } = await vi.importActual<typeof import('@/lib/ipc')>(
      '@/lib/ipc',
    )
    expect(
      khop?.[1],
      'Lệch tên sự kiện là lỗi IM LẶNG HOÀN TOÀN: `listen` chỉ đơn giản không bao giờ ' +
        'chạy — không lỗi, không cảnh báo, chỉ một giao diện đứng im.',
    ).toBe(SU_KIEN_TRANG_THAI_NGOAI)
  })
})
