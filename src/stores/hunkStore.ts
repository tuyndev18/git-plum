/**
 * Chọn khối và ba lệnh ghi theo khối — WORK-03, WORK-05, WORK-06.
 *
 * # 🔴 Vì sao đây là store RIÊNG, không phải một lát của `statusStore`
 *
 * `CONTEXT.md` mục 0, hệ quả 2, nguyên văn:
 *
 * > *"Tầng giao diện của Phase 5 phải chịu được việc Phase 4 còn lỗi. Nếu `ChangeList`
 * > hoá ra hiển thị sai, staging theo khối vẫn phải dùng được từ một đường vào khác."*
 *
 * Và lý do điều đó **không** phải thận trọng thừa: **không requirement nào của Phase 4
 * có bằng chứng từ mắt người.** Hai cổng của nó còn đỏ. Chủ dự án đã tìm **năm** lỗi
 * hiển thị của Phase 3 chỉ bằng cách mở ứng dụng, và cả năm đi qua hàng trăm test tự
 * động. Wave 5 của Phase 4 còn tìm được một **vòng khoá chết bốn điều kiện** mà không
 * test cũ nào thấy: `ChangeList` là chỗ **duy nhất** gọi `refresh`; nó chỉ mount khi
 * vùng soạn commit đã mở; đường **duy nhất** mở vùng soạn là bấm hàng WIP; hàng WIP
 * chỉ hiện khi store đã có dữ liệu.
 *
 * Bài học của vòng khoá đó không phải "sửa `ChangeList`" — nó là: **một trạng thái chỉ
 * có một đường vào thì đường đó là một điểm hỏng đơn.** Nên trạng thái chọn khối sống
 * ở một store **không ai của Phase 4 sở hữu**, và `HunkBar` đọc nó trực tiếp. Một
 * `ChangeList` hỏng, một `DiffViewer` ném lỗi khi render, một provider Phase 4 không
 * bao giờ mount — không cái nào chạm tới đường này.
 *
 * Có test ghim: `HunkBar.test.tsx` render `HunkBar` **một mình**, và đột biến **M25**
 * (bắt `HunkBar` đọc một store Phase 4 rồi `return null`) phải **đỏ**.
 *
 * # 🔴 Ràng buộc 2.5: ghi `RepoStatus` TỪ GIÁ TRỊ TRẢ VỀ, không gọi `refresh`
 *
 * Cùng ràng buộc và cùng lý do với `statusStore` — đọc doc comment đầu tệp đó. Cả ba
 * lệnh `stage_hunk` / `unstage_hunk` / `discard_hunk` phía Rust trả `Result<RepoStatus>`
 * chính vì điều này. Có test ghim rằng `getStatus` được gọi **0 lần** (khuôn đột biến
 * M2 của 04-02).
 *
 * # 🔴 Phân nhánh lỗi theo `code`, KHÔNG theo chuỗi `message`
 *
 * `isGitError` có sẵn cho đúng việc này. So khớp chuỗi thông điệp vỡ ngay lần đầu ai đó
 * sửa một dấu câu ở phía Rust — và phía Rust là nơi câu đó được ghim bằng test, nên nó
 * *sẽ* được sửa ở đó chứ không ở đây. Có test ghim ca "mã khác nhưng message chứa
 * nguyên văn 'hãy làm mới'" → **không** đặt `canLamMoi`.
 */

import { create } from 'zustand'

import { describeError, ipc, isGitError, type RepoStatus } from '@/lib/ipc'
import { useStatusStore } from './statusStore'

/** Lỗi WORK-05 đang hiện, kèm đường dẫn để nút làm mới biết phải làm mới cái gì. */
export interface CanLamMoi {
  path: string
  /** Nguyên văn thông điệp Rust — chứa câu *"hãy làm mới"*. */
  message: string
}

interface HunkState {
  /**
   * Khối đang chọn, dạng `` `${path}#${index}` ``. `null` = chưa chọn.
   *
   * Một chuỗi chứ không phải `{ path, index }`: khoá phẳng so sánh được bằng `===`
   * trong `HunkBar`, nên một khối chỉ render lại khi **chính nó** đổi trạng thái chọn,
   * không phải mỗi lần bất kỳ khối nào đổi.
   */
  dangChon: string | null
  canLamMoi: CanLamMoi | null
  chon: (path: string, index: number) => void
  boChon: () => void
  stageHunk: (repoId: string, path: string, index: number, blobHash: string) => Promise<void>
  unstageHunk: (repoId: string, path: string, index: number, blobHash: string) => Promise<void>
  discardHunk: (repoId: string, path: string, index: number, blobHash: string) => Promise<void>
  xoaCanLamMoi: () => void
}

/**
 * Khoá của một khối trong [`HunkState.dangChon`].
 *
 * `#` là dấu phân cách và nó **bắt buộc**: nối trần `path + index` biến `a#1` + `1`
 * thành `a#11`, tức hai khối khác nhau cho **cùng một** khoá. Có test ghim (`1` vs `11`).
 */
export function khoaKhoi(path: string, index: number): string {
  return `${path}#${index}`
}

export const useHunkStore = create<HunkState>((set) => ({
  dangChon: null,
  canLamMoi: null,

  chon: (path, index) => set({ dangChon: khoaKhoi(path, index) }),

  boChon: () => set({ dangChon: null }),

  xoaCanLamMoi: () => set({ canLamMoi: null }),

  stageHunk: async (repoId, path, index, blobHash) => {
    await ghiKhoi(set, repoId, path, () => ipc.stageHunk(repoId, path, index, blobHash))
  },

  unstageHunk: async (repoId, path, index, blobHash) => {
    await ghiKhoi(set, repoId, path, () => ipc.unstageHunk(repoId, path, index, blobHash))
  },

  discardHunk: async (repoId, path, index, blobHash) => {
    await ghiKhoi(set, repoId, path, () => ipc.discardHunk(repoId, path, index, blobHash))
  },
}))

type Set = (partial: Partial<HunkState>) => void

/**
 * Đường ghi dùng chung cho cả ba lệnh theo khối.
 *
 * # Thành công
 *
 * Ghi `RepoStatus` **từ giá trị trả về** vào `statusStore` qua `applyExternal` — **không**
 * gọi `refresh`. Xem doc comment đầu tệp.
 *
 * `applyExternal` là điểm vào đúng: nó nhận một `RepoStatus` **đã có sẵn** và không tự
 * gọi IPC, trong khi `refresh` sinh thêm một tiến trình `git status` cho một trạng thái
 * ta vừa nhận xong. Nó được khai tách riêng ở 04-02 đúng vì ca này.
 *
 * # Thất bại `file_changed` — WORK-05
 *
 * Đặt `canLamMoi` và **không** ghi gì vào `statusStore`. Trạng thái cũ giữ nguyên: bản
 * vá **không** được áp, nên trạng thái cũ vẫn là trạng thái đúng. Ghi một trạng thái
 * mới ở đây sẽ nói dối người dùng rằng có gì đó vừa đổi.
 *
 * # Thất bại khác
 *
 * Đi đường lỗi bình thường của `statusStore` (giữ `status` cũ, đặt `error`) và **không**
 * đặt `canLamMoi`. Một `index.lock` không phải "tệp đã đổi"; hiện sai câu làm người
 * dùng đi tìm một thay đổi không tồn tại. Có test ghim.
 */
async function ghiKhoi(
  set: Set,
  repoId: string,
  path: string,
  goi: () => Promise<RepoStatus>,
): Promise<void> {
  try {
    const status = await goi()
    useStatusStore.getState().applyExternal(repoId, status)
  } catch (e) {
    if (isGitError(e) && e.code === 'file_changed') {
      set({ canLamMoi: { path, message: e.message } })
      return
    }
    ghiLoiVaoStatusStore(repoId, describeError(e))
  }
}

/**
 * Đặt `error` của repo trong `statusStore` mà **giữ nguyên** `status` cũ.
 *
 * Xoá `status` khi thất bại làm danh sách **biến mất** dưới tay người dùng vì một lỗi
 * tạm thời (`index.lock` là ca thường — họ đang chạy git ở terminal). Họ mất cả chỗ
 * đang đứng lẫn khả năng thử lại. Cùng quyết định với `statusStore.ghi`.
 */
function ghiLoiVaoStatusStore(repoId: string, error: string): void {
  useStatusStore.setState((s) => ({
    byRepo: {
      ...s.byRepo,
      [repoId]: {
        status: s.byRepo[repoId]?.status,
        isLoading: false,
        error,
      },
    },
  }))
}
