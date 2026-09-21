/**
 * Danh sách repository gần đây — PLAT-07.
 *
 * Lưu qua `tauri-plugin-store` để người dùng mở lại repository cũ bằng một lần
 * bấm thay vì đi qua hộp thoại chọn thư mục mỗi lần chạy ứng dụng.
 *
 * Tệp được tách làm hai nửa có chủ ý:
 *
 * - **Phần thuần tuý** (`mergeRecent`, `sanitizeRecent`, `normalizeRepoPath`)
 *   không biết Tauri tồn tại. Toàn bộ quyết định về hành vi — thứ tự, khử trùng
 *   lặp, giới hạn số mục — nằm ở đây, nên kiểm thử được trực tiếp mà không cần
 *   giả lập gì.
 * - **Phần đụng Tauri** là một vỏ mỏng, và nó **không bao giờ ném lỗi**.
 *
 * Về việc nuốt lỗi: danh sách gần đây là một tiện ích, không phải dữ liệu quan
 * trọng. Nếu tệp store không đọc được hoặc không ghi được, thứ đúng đắn là mất
 * một mục trong danh sách, chứ không phải làm hỏng thao tác mở repository mà
 * người dùng vừa thực hiện thành công. Mọi lời gọi `store.*` vì thế được bọc
 * `try/catch` và chỉ `console.warn`.
 */

import { load, type Store } from '@tauri-apps/plugin-store'

/** Một mục trong danh sách gần đây. */
export interface RecentRepo {
  /** Đường dẫn thô do Rust trả về (`RepoInfo.path`, đã qua `--show-toplevel`). */
  path: string
  name: string
  /** Thời điểm mở gần nhất, để sắp thứ tự mới-nhất-trước. */
  openedAtMs: number
}

/**
 * Số mục tối đa giữ lại.
 *
 * Chặn ở cả `mergeRecent` (lúc ghi) và `sanitizeRecent` (lúc đọc) — T-01-12.
 * Chặn một đầu thôi thì một tệp store sót lại từ phiên bản có giới hạn lớn hơn
 * sẽ vượt rào.
 */
export const MAX_RECENT = 10

/**
 * Chuẩn hoá đường dẫn để so sánh, **khớp chính xác với phía Rust**.
 *
 * `state::repo_id_for` trong `src-tauri/src/state/mod.rs` làm đúng hai việc:
 * `replace('\\', "/")` rồi `trim_end_matches('/')`. Hàm này phải làm y như vậy,
 * không hơn không kém. Cụ thể là **không** đổi chữ thường dù Windows không phân
 * biệt hoa thường: phía Rust không làm thế, nên nếu ở đây làm thì hai mục mà
 * frontend coi là một lại cho ra hai `RepoId` khác nhau ở backend — đúng loại
 * bất đồng khó lần ra nhất.
 *
 * Giá trị trả về chỉ dùng để **so sánh**. Đường dẫn lưu xuống store vẫn là
 * chuỗi thô của Rust, vì đó là thứ sẽ được truyền lại xuống `open_repository`.
 */
export function normalizeRepoPath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/+$/, '')
}

/**
 * Đặt `entry` lên đầu danh sách, gỡ mục cũ trỏ cùng repository, cắt còn
 * `MAX_RECENT`.
 *
 * Trả về mảng mới, không sửa `list` tại chỗ — dữ liệu này đi vào store zustand,
 * và sửa tại chỗ thì React không thấy tham chiếu đổi nên không vẽ lại.
 */
export function mergeRecent(list: RecentRepo[], entry: RecentRepo): RecentRepo[] {
  const khoa = normalizeRepoPath(entry.path)
  const conLai = list.filter((m) => normalizeRepoPath(m.path) !== khoa)
  return [entry, ...conLai].slice(0, MAX_RECENT)
}

/** Một phần tử đọc từ đĩa có đúng hình dạng `RecentRepo` hay không. */
function laMucHopLe(raw: unknown): raw is RecentRepo {
  if (typeof raw !== 'object' || raw === null) return false
  const m = raw as Partial<RecentRepo>
  return (
    typeof m.path === 'string' &&
    m.path.length > 0 &&
    typeof m.name === 'string' &&
    m.name.length > 0 &&
    typeof m.openedAtMs === 'number' &&
    Number.isFinite(m.openedAtMs)
  )
}

/**
 * Lọc dữ liệu thô đọc từ đĩa thành danh sách dùng được — T-01-09.
 *
 * Tệp store nằm trong thư mục dữ liệu ứng dụng của người dùng: sửa tay được, và
 * có thể sót lại từ một phiên bản mang hình dạng khác. Phần tử hỏng bị **lọc
 * bỏ** chứ không làm cả danh sách vô hiệu, và hàm này không ném lỗi với bất kỳ
 * đầu vào nào.
 */
export function sanitizeRecent(raw: unknown): RecentRepo[] {
  if (!Array.isArray(raw)) return []

  const daThay = new Set<string>()
  const ketQua: RecentRepo[] = []

  for (const m of raw) {
    if (!laMucHopLe(m)) continue
    const khoa = normalizeRepoPath(m.path)
    // Trùng lặp trong tệp bị sửa tay: giao diện dùng `path` làm `key` của React
    // nên hai mục cùng khoá sinh cảnh báo key trùng.
    if (daThay.has(khoa)) continue
    daThay.add(khoa)
    ketQua.push({ path: m.path, name: m.name, openedAtMs: m.openedAtMs })
    if (ketQua.length >= MAX_RECENT) break
  }

  return ketQua
}

// --- Vỏ bọc quanh plugin store -------------------------------------------

const STORE_FILE = 'recent-repos.json'
const STORE_KEY = 'repos'

/**
 * Đối tượng store được đệm lại để không mở lại tệp mỗi lần gọi.
 *
 * Đệm chính promise chứ không đệm kết quả: hai lời gọi chạy song song lúc khởi
 * động (`loadRecent` của App và `rememberRepo` của lần mở đầu tiên) nếu không
 * thì cùng gọi `load()` và mở tệp hai lần.
 */
let storeDangCho: Promise<Store> | null = null

function getStore(): Promise<Store> {
  if (!storeDangCho) {
    // `autoSave: true` ghi xuống đĩa sau mỗi `set`, nên không cần `save()` tay.
    storeDangCho = load(STORE_FILE, { autoSave: true }).catch((e: unknown) => {
      // Xoá phần đệm để lần sau còn thử lại được — có thể chỉ là lỗi tạm thời.
      storeDangCho = null
      throw e
    })
  }
  return storeDangCho
}

/**
 * Chỉ dùng trong kiểm thử: xoá phần đệm store ở phạm vi module.
 *
 * Phần đệm là trạng thái sống suốt đời tiến trình, nên không reset thì store
 * giả của test trước còn sống ở test sau. Không gọi hàm này từ mã sản phẩm.
 */
export function __resetStoreForTests(): void {
  storeDangCho = null
}

export async function loadRecentRepos(): Promise<RecentRepo[]> {
  try {
    const store = await getStore()
    return sanitizeRecent(await store.get(STORE_KEY))
  } catch (e) {
    console.warn('[recentRepos] không đọc được danh sách gần đây:', e)
    return []
  }
}

/**
 * Ghi nhớ một repository vừa mở. Trả về danh sách sau khi ghi.
 *
 * Thất bại thì trả về danh sách rỗng và ghi cảnh báo — không ném lỗi, vì luồng
 * gọi hàm này là luồng mở repository đã thành công.
 */
export async function rememberRepo(info: {
  path: string
  name: string
}): Promise<RecentRepo[]> {
  try {
    const store = await getStore()
    const hienCo = sanitizeRecent(await store.get(STORE_KEY))
    const moi = mergeRecent(hienCo, {
      path: info.path,
      name: info.name,
      openedAtMs: Date.now(),
    })
    await store.set(STORE_KEY, moi)
    return moi
  } catch (e) {
    console.warn('[recentRepos] không ghi được danh sách gần đây:', e)
    return []
  }
}

/** Xoá một mục khỏi danh sách. Trả về danh sách sau khi xoá. */
export async function forgetRepo(path: string): Promise<RecentRepo[]> {
  try {
    const store = await getStore()
    const khoa = normalizeRepoPath(path)
    const conLai = sanitizeRecent(await store.get(STORE_KEY)).filter(
      (m) => normalizeRepoPath(m.path) !== khoa,
    )
    await store.set(STORE_KEY, conLai)
    return conLai
  } catch (e) {
    console.warn('[recentRepos] không xoá được mục khỏi danh sách gần đây:', e)
    return []
  }
}
