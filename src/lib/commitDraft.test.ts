/**
 * Bản nháp thông điệp commit — tiêu chí thành công số 6, R7.
 *
 * Giả lập ở ranh giới `@tauri-apps/plugin-store`, đúng khuôn `recentRepos.test.ts`.
 *
 * # 🔴 Đồng hồ giả, không phải thời gian thật
 *
 * Mọi test có trì hoãn ở đây dùng `vi.useFakeTimers()`. Lý do nằm ở lỗi #6 của
 * `CONTEXT.md` 3.1: một suite bất đồng bộ chạy **một** lần và xanh không chứng minh
 * gì — cổng đó đỏ 1/5 lần và sống qua cả một wave. Đồng hồ giả làm phép đo **tất
 * định**: `advanceTimersByTime(DRAFT_DEBOUNCE_MS)` nổ đúng hẹn giờ đó, không phụ
 * thuộc máy đang bận hay rảnh.
 *
 * Con số trì hoãn đọc từ `DRAFT_DEBOUNCE_MS` chứ **không** viết cứng: một test chờ
 * 200 ms cho một debounce 300 ms là một test đỏ ngẫu nhiên, và một test viết cứng 300
 * sẽ im lặng ngừng kiểm đúng thứ nó định kiểm khi ai đó đổi hằng số.
 */

import { load } from '@tauri-apps/plugin-store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import {
  DRAFT_DEBOUNCE_MS,
  __resetDraftForTests,
  clearDraft,
  draftKey,
  flushDraft,
  loadDraft,
  saveDraft,
} from '@/lib/commitDraft'

vi.mock('@tauri-apps/plugin-store', () => ({ load: vi.fn() }))

const moStore = vi.mocked(load)

const REPO_A = 'repo-a'
const REPO_B = 'repo-b'

/**
 * Store giả **có trạng thái thật**: `set` ghi vào map, `get` đọc từ map, `delete` xoá.
 *
 * Một `vi.fn().mockResolvedValue(undefined)` trơn cho `set` sẽ làm mọi test "ghi rồi
 * đọc lại" vô nghĩa — nó chỉ chứng minh hàm được gọi, không chứng minh giá trị nào
 * đến nơi. Và với M14 thì đó là **chính** phép phân biệt: đột biến ghi đúng số lần,
 * đúng hình dạng đối số, chỉ sai **khoá**.
 */
function storeGia(banDau: Record<string, unknown> = {}) {
  const data = new Map<string, unknown>(Object.entries(banDau))
  return {
    data,
    get: vi.fn(async (k: string) => data.get(k)),
    set: vi.fn(async (k: string, v: unknown) => {
      data.set(k, v)
    }),
    delete: vi.fn(async (k: string) => data.delete(k)),
    save: vi.fn(async () => undefined),
  }
}

let store: ReturnType<typeof storeGia>

beforeEach(() => {
  vi.clearAllMocks()
  __resetDraftForTests()
  store = storeGia()
  moStore.mockResolvedValue(store as never)
})

afterEach(() => {
  vi.useRealTimers()
  __resetDraftForTests()
})

/**
 * Cho hẹn giờ nổ **và** để những promise mà nó sinh ra chạy xong.
 *
 * `advanceTimersByTime` gọi callback đồng bộ, nhưng callback đó chỉ *bắt đầu* một
 * chuỗi `await` (lấy store, rồi `set`). Không nhường vòng lặp sự kiện thì phép ghi
 * chưa tới nơi và test đo quá sớm — đúng khiếm khuyết B của mutation #8 ở plan 03-04,
 * nơi ba `await Promise.resolve()` liên tiếp vẫn không đủ.
 */
async function choGhiXong(): Promise<void> {
  await vi.advanceTimersByTimeAsync(DRAFT_DEBOUNCE_MS)
  await vi.runAllTimersAsync()
}

describe('draftKey', () => {
  it('khoá chứa repoId — hai repo cho hai khoá khác nhau', () => {
    // Tiền đề của mọi test "chuyển repo" bên dưới. Một khoá dùng chung là chính cái
    // lỗi mà tiêu chí thành công số 6 tồn tại để chặn (đột biến M7).
    expect(draftKey(REPO_A)).toContain(REPO_A)
    expect(draftKey(REPO_B)).toContain(REPO_B)
    expect(draftKey(REPO_A)).not.toBe(draftKey(REPO_B))
  })
})

describe('loadDraft', () => {
  it('trả chuỗi RỖNG khi chưa có nháp, không trả undefined', async () => {
    // `undefined` đi vào `value` của textarea hiện đúng chữ "undefined" cho người dùng.
    const kq = await loadDraft(REPO_A)

    expect(kq).toBe('')
    expect(kq).not.toBeUndefined()
  })

  it('trả chuỗi rỗng khi giá trị trên đĩa không phải chuỗi', async () => {
    // Tệp store nằm trong thư mục dữ liệu người dùng: sửa tay được, và có thể sót lại
    // từ một phiên bản mang hình dạng khác.
    store.data.set(draftKey(REPO_A), { khong: 'phai chuoi' })

    expect(await loadDraft(REPO_A)).toBe('')
  })

  it('KHÔNG ném lỗi khi plugin store hỏng, và giao diện vẫn có chuỗi rỗng để hiện', async () => {
    // Nháp là tiện lợi, không phải dữ liệu không được mất. Vùng soạn commit phải sống
    // sót một tệp store hỏng.
    moStore.mockRejectedValue(new Error('tệp store hỏng'))
    __resetDraftForTests()

    await expect(loadDraft(REPO_A)).resolves.toBe('')
  })
})

describe('saveDraft — ghi có trì hoãn', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  it('KHÔNG ghi đĩa ngay lập tức — gõ từng ký tự không thành hàng trăm lần ghi', async () => {
    saveDraft(REPO_A, 'm')
    saveDraft(REPO_A, 'mo')
    saveDraft(REPO_A, 'mot')

    expect(store.set).not.toHaveBeenCalled()
  })

  it('gộp nhiều lần gõ thành ĐÚNG MỘT lần ghi, giữ giá trị cuối', async () => {
    saveDraft(REPO_A, 'm')
    saveDraft(REPO_A, 'mo')
    saveDraft(REPO_A, 'một thông điệp')

    await choGhiXong()

    expect(store.set).toHaveBeenCalledTimes(1)
    expect(store.data.get(draftKey(REPO_A))).toBe('một thông điệp')
  })

  it('round-trip nguyên vẹn văn bản nhiều dòng có tiếng Việt có dấu', async () => {
    const thongDiep = 'Sửa lỗi hiển thị đồ thị\n\n- Lane bị clamp ở cột 12\n- Chữ "đường dẫn" bị cắt'

    saveDraft(REPO_A, thongDiep)
    await choGhiXong()

    expect(await loadDraft(REPO_A)).toBe(thongDiep)
  })

  it('nháp rỗng thì XOÁ khoá, không ghi rác cho mỗi repo từng mở', async () => {
    store.data.set(draftKey(REPO_A), 'cũ')

    saveDraft(REPO_A, '')
    await choGhiXong()

    expect(store.delete).toHaveBeenCalledWith(draftKey(REPO_A))
    expect(store.data.has(draftKey(REPO_A))).toBe(false)
  })

  it('KHÔNG ném lỗi khi ghi thất bại', async () => {
    store.set.mockRejectedValue(new Error('đĩa đầy'))

    saveDraft(REPO_A, 'văn bản')

    await expect(choGhiXong()).resolves.toBeUndefined()
  })
})

describe('flushDraft — cách hỏng số 1: ghi treo bị HUỶ (M8)', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  it('xả ngay phép ghi đang treo, KHÔNG chờ hết trì hoãn', async () => {
    saveDraft(REPO_A, 'ký tự cuối chưa kịp ghi')
    expect(store.set).not.toHaveBeenCalled()

    await flushDraft()

    expect(store.data.get(draftKey(REPO_A))).toBe('ký tự cuối chưa kịp ghi')
  })

  it('🔴 gõ rồi đổi repo NGAY → quay lại thấy văn bản ĐẦY ĐỦ, không mất ký tự cuối', async () => {
    // Đây là ca mà một test ngây thơ bỏ lọt. "Gõ → **chờ** → đổi repo" xanh kể cả khi
    // `flushDraft` bị bỏ hẳn, vì hẹn giờ đã tự nổ trước lúc đổi. Chỉ "gõ → đổi
    // **ngay**" mới đỏ. Trì hoãn chính là thứ giấu lỗi, nên test phải **đua** với nó
    // một cách có chủ ý.
    const daDay = 'thông điệp đầy đủ tới ký tự cuối cùng'
    saveDraft(REPO_A, daDay)

    // KHÔNG advance timers ở đây — đó là cả điểm.
    await flushDraft()
    await loadDraft(REPO_B)

    expect(await loadDraft(REPO_A)).toBe(daDay)
  })

  it('không có gì treo thì flush là phép không làm gì, và không ném lỗi', async () => {
    await expect(flushDraft()).resolves.toBeUndefined()
    expect(store.set).not.toHaveBeenCalled()
  })
})

describe('🔴 M14 — cách hỏng số 2: ghi treo chạy SAU khi repo đã đổi', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  /**
   * 🔴 Fixture chịu lực của cả phase, và lý do nó trông như thế này.
   *
   * Repo B phải có một nháp **KHÁC RỖNG từ trước**. Đây không phải chi tiết trang trí
   * — nó là phép phân biệt:
   *
   * - B rỗng từ trước: đột biến ghi chuỗi của A đè lên ô rỗng của B, và phép khẳng
   *   định "B không chứa chữ của A"… vẫn có thể viết được, nhưng phép khẳng định tự
   *   nhiên hơn ("B rỗng") **xanh dưới cả hai cài đặt**. Đúng lỗi #7 của `CONTEXT.md`
   *   3.1: hình dạng đúng, dữ liệu vô hại, lỗi không để lại dấu vết quan sát được.
   * - B có nháp riêng khác rỗng: đột biến **phá dữ liệu người dùng đã lưu**, và điều
   *   đó quan sát được trực tiếp — `loadDraft(B)` trả chữ của A.
   *
   * Hai chuỗi cũng phải **khác nhau rõ rệt**, không phải hai biến thể của cùng một câu.
   */
  const NHAP_CUA_B = 'CÔNG VIỆC RIÊNG CỦA REPO B — không được mất'
  const NHAP_CUA_A = 'chữ của repo A'

  it('nháp KHÁC RỖNG của B không bị ghi đè khi ghi treo của A nổ sau lúc đổi repo', async () => {
    // Tiền đề: B thật sự có nháp khác rỗng trước khi bất cứ chuyện gì xảy ra. Thiếu
    // phép khẳng định này, một fixture hỏng (B rỗng) làm cổng xanh mà không kiểm gì.
    store.data.set(draftKey(REPO_B), NHAP_CUA_B)
    expect(await loadDraft(REPO_B)).toBe(NHAP_CUA_B)

    // Người dùng gõ ở repo A…
    saveDraft(REPO_A, NHAP_CUA_A)

    // …rồi đổi sang repo B **trước** khi hết trì hoãn. Repo hiện hành giờ là B.
    await loadDraft(REPO_B)

    // Bây giờ hẹn giờ của A mới nổ. Nếu phép ghi đọc "repo hiện hành" thay vì dùng
    // `repoId` đã đóng gói lúc lên lịch, nó ghi chữ của A vào khoá của B.
    await choGhiXong()

    expect(await loadDraft(REPO_B)).toBe(NHAP_CUA_B)
    expect(await loadDraft(REPO_B)).not.toBe(NHAP_CUA_A)
  })

  it('và chữ của A vẫn về đúng khoá của A', async () => {
    // Nửa còn lại của cùng bất biến: đóng gói `repoId` phải làm phép ghi **đúng chỗ**,
    // không chỉ làm nó **không sai chỗ**. Một cài đặt bỏ hẳn phép ghi cũng qua được
    // phép khẳng định ở test trên.
    store.data.set(draftKey(REPO_B), NHAP_CUA_B)

    saveDraft(REPO_A, NHAP_CUA_A)
    await loadDraft(REPO_B)
    await choGhiXong()

    expect(await loadDraft(REPO_A)).toBe(NHAP_CUA_A)
  })

  it('phép ghi mang đúng KHOÁ của repo đã lên lịch, không phải khoá của repo hiện hành', async () => {
    // Cổng ở mức đối số, độc lập với trạng thái cuối. Nó bắt được cả ca mà hai phép
    // ghi tình cờ cho cùng kết quả cuối.
    store.data.set(draftKey(REPO_B), NHAP_CUA_B)

    saveDraft(REPO_A, NHAP_CUA_A)
    await loadDraft(REPO_B)
    await choGhiXong()

    expect(store.set).toHaveBeenCalledWith(draftKey(REPO_A), NHAP_CUA_A)
    expect(store.set).not.toHaveBeenCalledWith(draftKey(REPO_B), NHAP_CUA_A)
  })
})

describe('chuyển repo — nửa "chuyển repo" của tiêu chí 6 (M7)', () => {
  it('nháp của A không rò sang B; quay lại A thấy lại nguyên văn', async () => {
    vi.useFakeTimers()

    saveDraft(REPO_A, 'việc đang làm dở ở repo A')
    await choGhiXong()

    // Sang B: ô soạn phải RỖNG.
    expect(await loadDraft(REPO_B)).toBe('')

    // Quay lại A: nguyên văn.
    expect(await loadDraft(REPO_A)).toBe('việc đang làm dở ở repo A')
  })
})

describe('clearDraft', () => {
  it('xoá nháp của đúng repo đó', async () => {
    store.data.set(draftKey(REPO_A), 'xong rồi')
    store.data.set(draftKey(REPO_B), 'của B')

    await clearDraft(REPO_A)

    expect(await loadDraft(REPO_A)).toBe('')
    expect(await loadDraft(REPO_B)).toBe('của B')
  })

  it('huỷ phép ghi treo của repo đó, nếu không nó ghi lại thông điệp vừa commit', async () => {
    vi.useFakeTimers()

    // Người dùng gõ xong rồi bấm commit ngay — hẹn giờ vẫn đang treo lúc commit chạy.
    saveDraft(REPO_A, 'thông điệp vừa được commit')
    await clearDraft(REPO_A)

    // 300 ms sau, hẹn giờ cũ không được phép sống lại và ghi đè khoá vừa xoá.
    await choGhiXong()

    expect(await loadDraft(REPO_A)).toBe('')
  })

  it('KHÔNG ném lỗi khi plugin store hỏng', async () => {
    store.delete.mockRejectedValue(new Error('đĩa hỏng'))

    await expect(clearDraft(REPO_A)).resolves.toBeUndefined()
  })
})
