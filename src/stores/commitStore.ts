/**
 * Vùng soạn commit — WORK-08, WORK-09, PLAT-05.
 *
 * Khuôn `byRepo[repoId]` chép từ `statusStore.ts` / `historyStore.ts`. Ở đây nó chịu
 * lực hơn mọi store trước: bản nháp thông điệp là **công sức người dùng vừa bỏ ra**,
 * và một store cấp ứng dụng (một `draft` duy nhất) sẽ hiện thông điệp của repo A
 * trong ô soạn của repo B — đúng cái lỗi mà tiêu chí thành công số 6 tồn tại để chặn.
 *
 * # 🔴 Commit THẤT BẠI thì nháp CÒN NGUYÊN
 *
 * Đây là hành vi quan trọng nhất của tệp này, và nó dễ bị "dọn dẹp" mất: xoá nháp
 * trong một khối `finally` trông gọn hơn, và nó **sai**. Hook từ chối là ca **thường**
 * (`pre-commit` chạy lint, lint đỏ), không phải ca hiếm. Mất thông điệp vừa gõ vì hook
 * từ chối là cách nhanh nhất để người ta bỏ một công cụ.
 *
 * Nên `clearDraft` chỉ được gọi trên đường **thành công**. Đột biến M6 ghim điều này.
 *
 * # Nguồn sự thật lúc chạy là `draftByRepo`, đĩa chỉ là bản sao
 *
 * `commitDraft.ts` ghi xuống đĩa **có trì hoãn**, và nó có thể thất bại im lặng (tệp
 * store hỏng). Nếu ô soạn đọc thẳng từ đĩa thì một lần ghi hỏng làm chữ người dùng
 * vừa gõ biến mất. Ở đây `setDraft` cập nhật store **đồng bộ, ngay lập tức**, rồi mới
 * lên lịch ghi đĩa. Đĩa phục vụ ca "đóng mở lại ứng dụng"; trong một phiên nó không
 * bao giờ được đọc lại đè lên bộ nhớ.
 *
 * # `switchRepo` xả nháp TRƯỚC khi đổi
 *
 * Xem `commitDraft.ts` về hai cách hỏng của một phép ghi trì hoãn. Ở đây chỉ cần biết:
 * mọi đường đổi repo phải đi qua `switchRepo`, không tự gọi `hydrate` thẳng.
 */

import { create } from 'zustand'

import { clearDraft, flushDraft, loadDraft, saveDraft } from '@/lib/commitDraft'
import { describeError, ipc, isGitError } from '@/lib/ipc'

interface CommitState {
  /** Nháp **trong bộ nhớ**, khoá theo repo. Nguồn sự thật của ô soạn. */
  draftByRepo: Record<string, string>
  /** Đang có một lời gọi commit/amend bay. Nút phải vô hiệu — đột biến M13. */
  isCommitting: boolean
  /** Thông điệp lỗi hiện cho người dùng. `null` = không có lỗi. */
  error: string | null
  /**
   * `true` khi lỗi hiện tại là **đầu ra nguyên văn của một hook**, không phải một
   * thông báo của ứng dụng.
   *
   * Giao diện cần biết để hiện nó trong `<pre>` (giữ xuống dòng và thụt lề) thay vì
   * một dòng `<p>`. Một cờ riêng chứ không phải phép dò chuỗi trong `error`: dò chuỗi
   * là đoán, và nó sẽ đoán sai ngay khi ai đó đổi văn bản thông báo.
   */
  errorIsHookOutput: boolean
  /** Công tắc `no-verify` — **mặc định TẮT**, ràng buộc 2.4. */
  noVerify: boolean
  amendMode: boolean
  /**
   * `true` khi lần amend **vừa chạy xong** đụng vào một commit đã push — WORK-09.
   *
   * 🔴 Đây là kết quả của một thao tác **đã hoàn thành**, không phải một câu hỏi trước
   * khi chạy. Không có đường nào trong tệp này đọc nó để quyết định có gọi IPC hay
   * không — "cảnh báo, KHÔNG chặn" là nguyên văn ROADMAP, và đột biến M9 ghim nó.
   */
  wasPushed: boolean

  setDraft: (repoId: string, text: string) => void
  /** Nạp nháp bền vững của một repo vào bộ nhớ. Gọi khi mở repo. */
  hydrate: (repoId: string) => Promise<void>
  /** Xả nháp treo rồi nạp nháp của repo mới. **Mọi** đường đổi repo dùng hàm này. */
  switchRepo: (repoId: string) => Promise<void>
  setNoVerify: (v: boolean) => void
  setAmendMode: (v: boolean) => void
  /** Tạo commit. Trả `true` khi **thành công**. */
  commit: (repoId: string) => Promise<boolean>
  /** Sửa commit gần nhất. Trả `true` khi **thành công**. */
  amend: (repoId: string) => Promise<boolean>
  clearError: () => void
  reset: (repoId: string) => void
}

/**
 * Bóc lỗi thành cặp (thông điệp, có phải đầu ra hook không).
 *
 * `hook_rejected` mang **nguyên văn** đầu ra hook trong `message`, nên nó **không**
 * đi qua `describeError`: hàm đó nối thêm một dòng `Lệnh đã chạy: git commit …` vào
 * cuối, và với một khối văn bản của hook thì cái đuôi đó làm người đọc tưởng nó là
 * một phần của hook. Với mọi mã khác, dòng lệnh là thông tin hữu ích (PLAT-10) nên giữ.
 */
function bocLoi(e: unknown): { message: string; isHook: boolean } {
  if (isGitError(e) && e.code === 'hook_rejected') {
    return { message: e.message, isHook: true }
  }
  return { message: describeError(e), isHook: false }
}

export const useCommitStore = create<CommitState>((set, get) => ({
  draftByRepo: {},
  isCommitting: false,
  error: null,
  errorIsHookOutput: false,
  noVerify: false,
  amendMode: false,
  wasPushed: false,

  setDraft: (repoId, text) => {
    // Bộ nhớ trước, đồng bộ: ô soạn phải hiện ký tự vừa gõ ngay, không sau 300 ms.
    set((s) => ({ draftByRepo: { ...s.draftByRepo, [repoId]: text } }))
    // 🔴 `repoId` đi vào `saveDraft` làm **tham số** và được đóng gói ở đó. Xem
    // `commitDraft.ts` — đột biến M14.
    saveDraft(repoId, text)
  },

  hydrate: async (repoId) => {
    const text = await loadDraft(repoId)
    set((s) => ({
      draftByRepo: {
        ...s.draftByRepo,
        // Nháp đã có trong bộ nhớ **thắng** thứ đọc từ đĩa: người dùng có thể đã gõ
        // tiếp trong lúc `loadDraft` còn bay, và ghi đè sẽ nuốt mất những ký tự đó.
        [repoId]: s.draftByRepo[repoId] ?? text,
      },
    }))
  },

  switchRepo: async (repoId) => {
    // 🔴 Xả **trước**, nếu không ký tự gõ ngay trước lúc chuyển repo bị mất cùng phép
    // ghi treo bị huỷ — cách hỏng số 1, đột biến M8.
    await flushDraft()
    await get().hydrate(repoId)
  },

  setNoVerify: (v) => set({ noVerify: v }),
  setAmendMode: (v) => set({ amendMode: v }),
  clearError: () => set({ error: null, errorIsHookOutput: false }),

  commit: async (repoId) => {
    const message = get().draftByRepo[repoId] ?? ''
    return ghi(set, get, repoId, message, async () => {
      await ipc.createCommit(repoId, message, get().noVerify)
      return false
    })
  },

  amend: async (repoId) => {
    const message = get().draftByRepo[repoId] ?? ''
    return ghi(set, get, repoId, message, async () => {
      const kq = await ipc.amendCommit(repoId, message, get().noVerify)
      return kq.wasPushed
    })
  },

  reset: (repoId) =>
    set((s) => {
      const { [repoId]: _bo, ...conLai } = s.draftByRepo
      return { draftByRepo: conLai }
    }),
}))

type Set = (p: Partial<CommitState> | ((s: CommitState) => Partial<CommitState>)) => void
type Get = () => CommitState

/**
 * Đường ghi dùng chung cho `commit` và `amend`.
 *
 * 🔴 Ba bất biến, và cả ba đều có đột biến ghim:
 *
 * 1. **Thất bại → nháp còn nguyên.** `clearDraft` chỉ nằm trên nhánh thành công. Không
 *    có `finally` nào chạm vào nháp (M6).
 * 2. **`wasPushed` không quyết định có chạy hay không.** Nó chỉ được **ghi lại sau
 *    khi** lời gọi hoàn thành. Không có `if (wasPushed) return` ở đâu (M9).
 * 3. **Hai lần gọi chồng nhau thì lần sau bị bỏ.** Một cú bấm đôi trên nút commit là
 *    hai commit, và commit thứ hai sẽ rỗng hoặc lấy mất thứ người dùng không định
 *    commit. `isCommitting` chặn ở đây **cộng thêm** `disabled` ở nút (M13) — hai lớp
 *    có chủ ý: `disabled` là trải nghiệm, phép chặn này là tính đúng đắn, và `disabled`
 *    không chặn được đường bàn phím hay một lệnh gọi từ sổ lệnh.
 */
async function ghi(
  set: Set,
  get: Get,
  repoId: string,
  message: string,
  goi: () => Promise<boolean>,
): Promise<boolean> {
  if (get().isCommitting) return false

  // Chặn thông điệp rỗng **trước** khi gọi IPC. Phía Rust cũng chặn (mã
  // `empty_commit_message`) — hai lớp, có chủ ý: phía này cho phản hồi tức thì và không sinh
  // một tiến trình git, phía kia là bất biến thật vì nó gần `git` hơn.
  // **Không tự sửa thông điệp** — R5. Không trim rồi gửi, không thêm nội dung.
  if (message.trim() === '') {
    set({
      error: 'Thông điệp commit đang rỗng. Hãy viết một dòng mô tả thay đổi này.',
      errorIsHookOutput: false,
    })
    return false
  }

  set({ isCommitting: true, error: null, errorIsHookOutput: false, wasPushed: false })

  try {
    const wasPushed = await goi()
    set({ isCommitting: false, wasPushed })
    // ✅ **Chỉ** ở đây. Đường thất bại bên dưới không chạm vào nháp.
    await clearDraft(repoId)
    set((s) => ({ draftByRepo: { ...s.draftByRepo, [repoId]: '' } }))
    return true
  } catch (e) {
    const { message: thongBao, isHook } = bocLoi(e)
    // 🔴 `draftByRepo` **không** xuất hiện trong khối này. Xem bất biến 1 ở trên.
    set({ isCommitting: false, error: thongBao, errorIsHookOutput: isHook })
    return false
  }
}
