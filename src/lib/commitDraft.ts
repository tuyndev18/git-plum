/**
 * Bản nháp thông điệp commit, bền vững, **khoá theo `repoId`** — tiêu chí thành
 * công số 6 của Phase 4, và R7 của `CONTEXT.md`.
 *
 * Tệp này chép **đúng khuôn** của `src/lib/recentRepos.ts` (PLAT-07) và không phát
 * minh khuôn thứ hai: cùng cách lấy store (`load` với `autoSave`, promise được đệm ở
 * cấp module), cùng cách xử lý lỗi (**không bao giờ ném**, chỉ `console.warn`), cùng
 * cách tiêm giả lập ở test (`vi.mock('@tauri-apps/plugin-store')` + `__resetStoreForTests`).
 *
 * # Nháp là tiện lợi, không phải dữ liệu không được mất
 *
 * `tauri-plugin-store` hỏng thì thứ đúng đắn là mất bản nháp trên đĩa, **không** phải
 * làm sập vùng soạn commit mà người dùng đang gõ vào. Mọi lời gọi `store.*` được bọc
 * `try/catch`. Nháp vẫn hoạt động đầy đủ trong phiên vì nguồn sự thật lúc chạy là
 * `commitStore.draftByRepo`, không phải tệp trên đĩa.
 *
 * # 🔴 Ghi có trì hoãn, và `repoId` được ĐÓNG GÓI LÚC LÊN LỊCH
 *
 * Người dùng gõ từng ký tự. Ghi đĩa mỗi lần gõ là hàng trăm lần ghi cho một thông
 * điệp, nên phép ghi được gộp lại sau `DRAFT_DEBOUNCE_MS`. Nhưng **một phép ghi trì
 * hoãn có HAI cách hỏng, không phải một**, và chúng cần hai biện pháp khác nhau:
 *
 * | | Hỏng thế nào | Biện pháp ở đây |
 * |---|---|---|
 * | 1 | Ghi treo bị **huỷ** (đổi repo, đóng ứng dụng) → mất ký tự cuối | `flushDraft()` — gọi **trước** khi đổi repo |
 * | 2 | Ghi treo **chạy sau khi repo đã đổi** và ghi **đè** nháp của repo mới | `repoId` đóng gói lúc lên lịch |
 *
 * Cách 2 tệ hơn hẳn: nó phá dữ liệu người dùng **đã lưu**, không chỉ vài ký tự vừa gõ.
 *
 * 🔴 **Quy tắc bắt buộc:** phép ghi treo giữ đúng `repoId` mà `saveDraft` được gọi
 * cùng, kể cả khi 300 ms sau `repoId` hiện hành đã khác. Đọc `repoId` từ store **bên
 * trong** hàm ghi là **sai** — và đó chính là cái bẫy, vì chữ ký hàm vẫn trông như có
 * tham số. Ở đây `repoId` nằm trong `pending`, một biến cục bộ module, được ghi lúc
 * `saveDraft` chạy và đọc lại nguyên vẹn lúc hẹn giờ nổ. Đột biến M14 ghim điều này.
 *
 * Cả hai cách hỏng **trốn được** một test ngây thơ: "gõ → chờ → đổi repo" xanh ở cả
 * hai. Chỉ "gõ → đổi **ngay**" mới đỏ.
 */

import { load, type Store } from '@tauri-apps/plugin-store'

const STORE_FILE = 'commit-drafts.json'

/**
 * Khoảng gộp ghi đĩa.
 *
 * Xuất ra để test dùng đúng con số này thay vì viết cứng một số khác — một test chờ
 * 200 ms cho một debounce 300 ms là một test đỏ ngẫu nhiên.
 */
export const DRAFT_DEBOUNCE_MS = 300

/**
 * Khoá lưu trữ của một repo.
 *
 * 🔴 **Chứa `repoId`** (R7 + PLAT-05). Một khoá dùng chung cho mọi repo là **chính**
 * cái lỗi mà tiêu chí thành công số 6 tồn tại để chặn: gõ nháp ở repo A, mở repo B, và
 * thấy thông điệp của A trong ô soạn của B. Đột biến M7 ghim điều này.
 */
export function draftKey(repoId: string): string {
  return `draft:${repoId}`
}

let storeDangCho: Promise<Store> | null = null

function getStore(): Promise<Store> {
  if (!storeDangCho) {
    storeDangCho = load(STORE_FILE, { autoSave: true }).catch((e: unknown) => {
      storeDangCho = null
      throw e
    })
  }
  return storeDangCho
}

/**
 * Phép ghi đang treo. **Nhiều nhất một**, vì người dùng gõ vào một ô tại một thời điểm.
 *
 * 🔴 `repoId` nằm **trong** đối tượng này, không được đọc lại từ đâu lúc hẹn giờ nổ.
 * Xem khối doc comment đầu tệp — đây là chỗ cài đặt của quy tắc đó.
 */
let pending: { repoId: string; text: string } | null = null
let timer: ReturnType<typeof setTimeout> | null = null

/** Đọc nháp bền vững của một repo. Không có → chuỗi **rỗng**, không `undefined`. */
export async function loadDraft(repoId: string): Promise<string> {
  // Phép ghi đang treo là phiên bản mới hơn thứ nằm trên đĩa. Đọc đĩa mà bỏ qua nó
  // sẽ trả một bản cũ hơn thứ người dùng vừa gõ.
  if (pending && pending.repoId === repoId) return pending.text

  try {
    const store = await getStore()
    const raw = await store.get(draftKey(repoId))
    // `undefined` đi thẳng vào `value` của textarea sẽ hiện đúng chữ "undefined" cho
    // người dùng — một lỗi nhìn thấy được, nên chặn ở đây chứ không ở từng component.
    return typeof raw === 'string' ? raw : ''
  } catch (e) {
    console.warn('[commitDraft] không đọc được bản nháp:', e)
    return ''
  }
}

/**
 * Lên lịch ghi nháp của **`repoId` này**. Trì hoãn `DRAFT_DEBOUNCE_MS`.
 *
 * 🔴 `repoId` được **đóng gói ngay tại đây**. Đừng đổi cài đặt này thành đọc repo
 * hiện hành từ store lúc hẹn giờ nổ, dù chữ ký hàm vẫn giữ tham số: xem đầu tệp,
 * cách hỏng số 2, và đột biến M14.
 */
export function saveDraft(repoId: string, text: string): void {
  pending = { repoId, text }
  if (timer !== null) clearTimeout(timer)
  timer = setTimeout(() => {
    timer = null
    void ghiNgay()
  }, DRAFT_DEBOUNCE_MS)
}

/**
 * Ghi ngay phép ghi đang treo, nếu có. **Phải gọi khi đổi repo và khi đóng ứng dụng.**
 *
 * Thiếu nó, ký tự gõ ngay trước lúc chuyển repo bị mất — cách hỏng số 1, đột biến M8.
 */
export async function flushDraft(): Promise<void> {
  if (timer !== null) {
    clearTimeout(timer)
    timer = null
  }
  await ghiNgay()
}

async function ghiNgay(): Promise<void> {
  const viec = pending
  if (!viec) return
  pending = null

  try {
    const store = await getStore()
    // Nháp rỗng thì **xoá khoá** thay vì ghi chuỗi rỗng: không để lại rác trong tệp
    // store cho mỗi repo người dùng từng mở rồi không gõ gì.
    if (viec.text === '') await store.delete(draftKey(viec.repoId))
    else await store.set(draftKey(viec.repoId), viec.text)
  } catch (e) {
    console.warn('[commitDraft] không ghi được bản nháp:', e)
  }
}

/** Xoá nháp của một repo — gọi sau khi commit **thành công**. */
export async function clearDraft(repoId: string): Promise<void> {
  // Huỷ phép ghi treo **của chính repo này**, nếu không nó sẽ ghi lại thông điệp vừa
  // được commit vào đúng khoá ta vừa xoá, 300 ms sau.
  if (pending && pending.repoId === repoId) {
    pending = null
    if (timer !== null) {
      clearTimeout(timer)
      timer = null
    }
  }

  try {
    const store = await getStore()
    await store.delete(draftKey(repoId))
  } catch (e) {
    console.warn('[commitDraft] không xoá được bản nháp:', e)
  }
}

/** Chỉ dùng trong kiểm thử: xoá phần đệm store và mọi phép ghi treo. */
export function __resetDraftForTests(): void {
  storeDangCho = null
  pending = null
  if (timer !== null) {
    clearTimeout(timer)
    timer = null
  }
}
