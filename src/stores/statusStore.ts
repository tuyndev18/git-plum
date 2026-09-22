/**
 * Trạng thái thư mục làm việc — WORK-01, WORK-02, PLAT-05.
 *
 * Khuôn `byRepo[repoId]` chép từ `historyStore.ts` / `repoStore.ts`. Không phải sở
 * thích: "trạng thái thư mục làm việc của repo nào" là đúng loại câu hỏi mà PLAT-05
 * tồn tại để trả lời, và một `RepoStatus` duy nhất ở cấp store sẽ hiện danh sách tệp
 * của repo A sau khi người dùng chuyển sang repo B — trong khoảng giữa hai lần nạp.
 * Ở phase này hậu quả nặng hơn mọi phase trước: người dùng có thể **bấm stage** trên
 * danh sách sai.
 *
 * # 🔴 Đường stage/unstage ghi `RepoStatus` TỪ GIÁ TRỊ TRẢ VỀ, không gọi lại `refresh`
 *
 * Đây là ràng buộc 2.5 của CONTEXT.md nhìn từ phía giao diện. Lệnh ghi phía Rust đã
 * trả trạng thái **mới** (kiểu trả về của nó là `Result<RepoStatus>`, không phải
 * `Result<()>`, chính vì điều này). Gọi `refresh` sau đó làm hỏng cả hai đầu:
 *
 * * một lệnh `git status` **thừa** cho **mỗi** cú bấm — trên một repo lớn đó là hàng
 *   chục mili giây và một tiến trình con, nhân với số tệp người dùng stage;
 * * và nó **xoá mất cả điểm** của việc lệnh ghi trả trạng thái: nếu giao diện vẫn
 *   phải đọc lại thì kiểu trả về kia chỉ là trang trí, và lần "dọn dẹp" sau sẽ đổi nó
 *   về `void` mà không test nào đỏ.
 *
 * Có test ghim rằng `getStatus` **không** được gọi trong đường stage (đột biến M2).
 *
 * # `applyExternal` khai **ở đây**, dùng ở 04-04
 *
 * Watcher của plan 04-04 cần một điểm vào để đẩy trạng thái mới đến từ **bên ngoài**
 * ứng dụng (người dùng chạy git ở terminal — WORK-10). Khai nó ngay bây giờ để 04-04
 * chỉ phải nối dây, không phải đổi hình dạng store; đổi hình dạng store ở wave sau
 * nghĩa là sửa lại mọi component đọc nó.
 *
 * Nó **tách riêng** khỏi `refresh` có chủ ý: `refresh` tự gọi IPC, còn `applyExternal`
 * nhận dữ liệu đã có sẵn. Gộp chúng lại buộc watcher phải sinh thêm một lệnh
 * `git status` cho một trạng thái nó vừa đọc xong.
 */

import { create } from 'zustand'
import { describeError, ipc, type RepoStatus } from '@/lib/ipc'

interface StatusSlice {
  /** `undefined` = chưa nạp lần nào. Khác `entries: []` nghĩa là "đã nạp, repo sạch". */
  status: RepoStatus | undefined
  isLoading: boolean
  error: string | null
}

interface StatusState {
  byRepo: Record<string, StatusSlice>
  refresh: (repoId: string) => Promise<void>
  stage: (repoId: string, paths: string[]) => Promise<void>
  unstage: (repoId: string, paths: string[]) => Promise<void>
  /** Điểm vào cho watcher của 04-04 — xem doc comment đầu tệp. */
  applyExternal: (repoId: string, status: RepoStatus) => void
  reset: (repoId: string) => void
}

function emptySlice(): StatusSlice {
  return { status: undefined, isLoading: false, error: null }
}

/**
 * Các `refresh` **đang bay**, khoá theo `repoId`.
 *
 * Sống ở cấp module chứ không trong store: nó là chi tiết của một lời gọi đang chạy,
 * không phải trạng thái ứng dụng, và đưa vào store làm mọi component đọc store render
 * lại hai lần cho một lần nạp. Cùng lập luận với `pendingSelection` của `FileList.tsx`.
 *
 * Khuôn "đang bay" chép từ `historyStore.ensureRange`, và cùng lý do: hai component
 * cùng gọi `refresh(repoId)` trong cùng một khung render (ví dụ `ChangeList` mount và
 * một `useEffect` của panel cha) sinh **hai** tiến trình `git status` cho một câu hỏi.
 */
const dangBay = new Map<string, Promise<void>>()

export const useStatusStore = create<StatusState>((set, get) => ({
  byRepo: {},

  refresh: async (repoId) => {
    // Đã có một lời gọi đang bay cho repo này → chờ chính nó, **không** gọi IPC lần
    // nữa. Trả lại cùng promise nghĩa là người gọi thứ hai thấy cùng kết quả và cùng
    // thời điểm hoàn thành.
    const daCo = dangBay.get(repoId)
    if (daCo) return daCo

    const chay = (async () => {
      set((s) => ({
        byRepo: {
          ...s.byRepo,
          [repoId]: { ...(s.byRepo[repoId] ?? emptySlice()), isLoading: true, error: null },
        },
      }))

      try {
        const status = await ipc.getStatus(repoId)
        set((s) => ({
          byRepo: { ...s.byRepo, [repoId]: { status, isLoading: false, error: null } },
        }))
      } catch (e) {
        set((s) => ({
          byRepo: {
            ...s.byRepo,
            [repoId]: {
              ...(s.byRepo[repoId] ?? emptySlice()),
              isLoading: false,
              error: describeError(e),
            },
          },
        }))
      } finally {
        dangBay.delete(repoId)
      }
    })()

    dangBay.set(repoId, chay)
    return chay
  },

  stage: async (repoId, paths) => {
    await ghi(set, get, repoId, () => ipc.stageFiles(repoId, paths))
  },

  unstage: async (repoId, paths) => {
    await ghi(set, get, repoId, () => ipc.unstageFiles(repoId, paths))
  },

  applyExternal: (repoId, status) =>
    set((s) => ({
      byRepo: { ...s.byRepo, [repoId]: { status, isLoading: false, error: null } },
    })),

  reset: (repoId) =>
    set((s) => {
      const { [repoId]: _removed, ...rest } = s.byRepo
      dangBay.delete(repoId)
      return { byRepo: rest }
    }),
}))

type Set = (fn: (s: StatusState) => Partial<StatusState>) => void
type Get = () => StatusState

/**
 * Đường ghi dùng chung cho `stage` và `unstage`.
 *
 * 🔴 Ghi `RepoStatus` **từ giá trị trả về** của lời gọi IPC. **Không** gọi `refresh`
 * sau đó — xem doc comment đầu tệp về lý do, và đột biến M2 của plan.
 *
 * # Thất bại thì giữ nguyên trạng thái cũ và hiện lỗi
 *
 * Xoá `status` khi thất bại làm danh sách **biến mất** dưới tay người dùng vì một lỗi
 * tạm thời (`index.lock` là ca thường — họ đang chạy git ở terminal). Họ mất cả chỗ
 * đang đứng lẫn khả năng thử lại. Nên `status` giữ nguyên, chỉ `error` được đặt.
 */
async function ghi(
  set: Set,
  _get: Get,
  repoId: string,
  goi: () => Promise<RepoStatus>,
): Promise<void> {
  set((s) => ({
    byRepo: {
      ...s.byRepo,
      [repoId]: { ...(s.byRepo[repoId] ?? emptySlice()), isLoading: true, error: null },
    },
  }))

  try {
    const status = await goi()
    set((s) => ({
      byRepo: { ...s.byRepo, [repoId]: { status, isLoading: false, error: null } },
    }))
  } catch (e) {
    set((s) => ({
      byRepo: {
        ...s.byRepo,
        [repoId]: {
          // Giữ `status` cũ — xem doc comment trên.
          ...(s.byRepo[repoId] ?? emptySlice()),
          isLoading: false,
          error: describeError(e),
        },
      },
    }))
  }
}
