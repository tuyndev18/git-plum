/**
 * `commitStore` — WORK-08, WORK-09.
 *
 * Giả lập **hai** ranh giới, và có lý do cho từng cái:
 *
 * - `@/lib/ipc` — khuôn của `statusStore.test.ts`. Chữ ký của hàm giả khớp **từng
 *   tham số** với thứ `ipc.ts` khai, nên phần Rust về sau không buộc phải viết lại
 *   những test này.
 * - `@tauri-apps/plugin-store` — **không** giả lập `@/lib/commitDraft`. Giả lập nó sẽ
 *   làm test "commit thất bại → nháp còn nguyên" chỉ chứng minh `clearDraft` **không
 *   được gọi**, chứ không chứng minh nháp còn đó. Đi qua module thật lên tới ranh giới
 *   plugin thì phép khẳng định đọc được **giá trị thật**.
 */

import { load } from '@tauri-apps/plugin-store'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { DRAFT_DEBOUNCE_MS, __resetDraftForTests, draftKey } from '@/lib/commitDraft'
import type { AmendResult, GitErrorPayload, RepoStatus } from '@/lib/ipc'
import { useCommitStore } from '@/stores/commitStore'

const createCommit = vi.fn<(r: string, m: string, n: boolean) => Promise<RepoStatus>>()
const amendCommit = vi.fn<(r: string, m: string, n: boolean) => Promise<AmendResult>>()

vi.mock('@/lib/ipc', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/lib/ipc')>()
  return {
    ...actual,
    ipc: {
      ...actual.ipc,
      createCommit: (r: string, m: string, n: boolean) => createCommit(r, m, n),
      amendCommit: (r: string, m: string, n: boolean) => amendCommit(r, m, n),
    },
  }
})

vi.mock('@tauri-apps/plugin-store', () => ({ load: vi.fn() }))

const moStore = vi.mocked(load)

const REPO_A = 'repo-a'
const REPO_B = 'repo-b'

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

function trangThai(): RepoStatus {
  return {
    branch: { head: 'main', oid: 'a'.repeat(40), upstream: null, ahead: null, behind: null },
    entries: [],
    hasConflicts: false,
  }
}

/** Lỗi đúng hình dạng `GitErrorPayload` mà Rust ném cho một hook từ chối. */
function loiHook(nguyenVan: string): GitErrorPayload {
  return { code: 'hook_rejected', message: nguyenVan, command: 'git commit -F -' }
}

/**
 * Nguyên văn đầu ra hook mà fixture của Task 1 in ra.
 *
 * Chuỗi **nhận diện được**, không phải một câu chung chung: một test khẳng định "có
 * lỗi" xanh kể cả khi ứng dụng nuốt mất đầu ra hook và thay bằng thông báo của chính
 * nó — đúng đột biến M5. Nhiều dòng, vì đầu ra hook thật có nhiều dòng và phần giữ
 * xuống dòng là thứ `CommitBox` phải bảo toàn.
 */
const HOOK_NGUYEN_VAN =
  'HOOK-PRE-COMMIT-REJECTED\n  src/a.ts:12  thiếu dấu chấm phẩy\n  src/b.ts:4   biến không dùng'

beforeEach(() => {
  vi.clearAllMocks()
  __resetDraftForTests()
  store = storeGia()
  moStore.mockResolvedValue(store as never)
  useCommitStore.setState({
    draftByRepo: {},
    isCommitting: false,
    error: null,
    errorIsHookOutput: false,
    noVerify: false,
    amendMode: false,
    wasPushed: false,
  })
  createCommit.mockResolvedValue(trangThai())
  amendCommit.mockResolvedValue({ status: trangThai(), wasPushed: false })
})

afterEach(() => {
  vi.useRealTimers()
  __resetDraftForTests()
})

describe('mặc định', () => {
  it('🔴 noVerify mặc định TẮT — hook chạy MẶC ĐỊNH (ràng buộc 2.4)', () => {
    // Bỏ hook là bỏ một trong những lý do dự án chọn `git` CLI thay vì libgit2.
    expect(useCommitStore.getState().noVerify).toBe(false)
  })

  it('amendMode mặc định tắt', () => {
    expect(useCommitStore.getState().amendMode).toBe(false)
  })
})

describe('setDraft', () => {
  it('cập nhật nháp ĐỒNG BỘ, không chờ hết trì hoãn ghi đĩa', () => {
    // Ô soạn phải hiện ký tự vừa gõ ngay. Chờ 300 ms là một ô soạn giật.
    useCommitStore.getState().setDraft(REPO_A, 'xin chào')

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('xin chào')
  })

  it('nháp của hai repo độc lập nhau trong bộ nhớ', () => {
    useCommitStore.getState().setDraft(REPO_A, 'của A')
    useCommitStore.getState().setDraft(REPO_B, 'của B')

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('của A')
    expect(useCommitStore.getState().draftByRepo[REPO_B]).toBe('của B')
  })
})

describe('hydrate / switchRepo', () => {
  it('nạp nháp bền vững của repo khi mở', async () => {
    store.data.set(draftKey(REPO_A), 'việc dở từ phiên trước')

    await useCommitStore.getState().hydrate(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('việc dở từ phiên trước')
  })

  it('không có nháp trên đĩa → chuỗi rỗng, KHÔNG undefined', async () => {
    await useCommitStore.getState().hydrate(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('')
  })

  it('🔴 switchRepo XẢ nháp treo trước khi đổi — không mất ký tự cuối (M8)', async () => {
    vi.useFakeTimers()
    // Gõ rồi đổi repo NGAY. Không advance timers: đó là cả điểm. Một test gõ → chờ →
    // đổi repo xanh kể cả khi `flushDraft` bị bỏ hẳn.
    useCommitStore.getState().setDraft(REPO_A, 'ký tự cuối cùng vừa gõ')

    await useCommitStore.getState().switchRepo(REPO_B)

    expect(store.data.get(draftKey(REPO_A))).toBe('ký tự cuối cùng vừa gõ')
  })
})

describe('commit — đường thành công', () => {
  it('gọi createCommit với repoId, nguyên văn thông điệp, và noVerify hiện hành', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'feat: thêm vùng soạn commit')

    await useCommitStore.getState().commit(REPO_A)

    expect(createCommit).toHaveBeenCalledWith(REPO_A, 'feat: thêm vùng soạn commit', false)
  })

  it('🔴 noVerify=true thì cờ được truyền xuống — công tắc thật, không phải ô tick trang trí (M2)', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'bỏ qua hook lần này')
    useCommitStore.getState().setNoVerify(true)

    await useCommitStore.getState().commit(REPO_A)

    expect(createCommit).toHaveBeenCalledWith(REPO_A, 'bỏ qua hook lần này', true)
  })

  it('🔴 noVerify=false được truyền TƯỜNG MINH — kiểm cả hai giá trị, không chỉ true (M1)', async () => {
    // Test chỉ kiểm `true` không chứng minh mặc định là `false`: một cài đặt thêm
    // `--no-verify` vô điều kiện vẫn qua được nó.
    useCommitStore.getState().setDraft(REPO_A, 'để hook chạy')

    await useCommitStore.getState().commit(REPO_A)

    const [, , coBo] = createCommit.mock.calls[0]!
    expect(coBo).toBe(false)
  })

  it('thông điệp nhiều dòng giữ nguyên xuống dòng (M3)', async () => {
    const nhieuDong = 'feat: tiêu đề\n\nThân thông điệp dòng một.\nThân dòng hai.'
    useCommitStore.getState().setDraft(REPO_A, nhieuDong)

    await useCommitStore.getState().commit(REPO_A)

    expect(createCommit).toHaveBeenCalledWith(REPO_A, nhieuDong, false)
  })

  it('🔴 M3 — chuỗi đi xuống IPC giống HỆT chuỗi người dùng gõ, từng ký tự', async () => {
    /*
     * Cổng riêng cho M3, đọc **đối số thật** thay vì so khớp cả lời gọi.
     *
     * Vì sao cần cổng riêng: phép khẳng định `toHaveBeenCalledWith` ở test trên đỏ
     * dưới **nhiều** đột biến (M1 làm nó đỏ vì tham số thứ ba), nên một mình nó không
     * nói được rằng phần **thông điệp** mới là thứ được bảo vệ. Cổng này chỉ đọc tham
     * số thứ hai, nên nó đỏ **chỉ khi** thông điệp bị biến dạng.
     *
     * Ba thứ được ghim, và cả ba là cách một đường `-m <message>` làm hỏng dữ liệu:
     * xuống dòng bị nuốt, khoảng trắng đầu dòng bị cắt, và dấu `-` ở đầu bị git đọc
     * thành cờ.
     */
    const kho = '-feat: mở đầu bằng dấu gạch\n\n  thụt lề hai dấu cách\n\ttab\ncuối'
    useCommitStore.getState().setDraft(REPO_A, kho)

    await useCommitStore.getState().commit(REPO_A)

    const [, thongDiepThat] = createCommit.mock.calls[0]!
    expect(thongDiepThat).toBe(kho)
    // Tiền đề: chuỗi fixture thật sự mang những ca nó định kiểm. Thiếu phép khẳng
    // định này, ai đó "dọn" fixture thành một dòng trơn và cổng im lặng ngừng kiểm.
    expect(kho).toContain('\n')
    expect(kho.startsWith('-')).toBe(true)
  })

  it('thành công → nháp bị XOÁ ở cả bộ nhớ lẫn đĩa', async () => {
    store.data.set(draftKey(REPO_A), 'thông điệp')
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('')
    expect(store.data.has(draftKey(REPO_A))).toBe(false)
  })

  it('thành công → trả true và không còn lỗi', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'ok')

    await expect(useCommitStore.getState().commit(REPO_A)).resolves.toBe(true)
    expect(useCommitStore.getState().error).toBeNull()
  })
})

describe('🔴 commit THẤT BẠI — nháp CÒN NGUYÊN (M6, đột biến quan trọng nhất)', () => {
  it('hook từ chối → nháp còn nguyên trong bộ nhớ', async () => {
    // Hook `pre-commit` chạy lint và lint đỏ là ca THƯỜNG, không phải ca hiếm. Mất
    // thông điệp vừa gõ vì nó là cách nhanh nhất để người ta bỏ một công cụ.
    const congSuc = 'feat: một thông điệp dài mà người dùng vừa bỏ công viết'
    useCommitStore.getState().setDraft(REPO_A, congSuc)
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe(congSuc)
  })

  it('hook từ chối → nháp trên ĐĨA cũng còn nguyên', async () => {
    // Nửa còn lại: xoá khỏi bộ nhớ và giữ trên đĩa (hay ngược lại) đều là mất dữ liệu
    // ở một trong hai đường người dùng đi.
    const congSuc = 'công sức người dùng'
    store.data.set(draftKey(REPO_A), congSuc)
    useCommitStore.setState({ draftByRepo: { [REPO_A]: congSuc } })
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await useCommitStore.getState().commit(REPO_A)

    expect(store.data.get(draftKey(REPO_A))).toBe(congSuc)
  })

  it('🔴 lỗi mang NGUYÊN VĂN đầu ra hook, không phải thông báo chung của ứng dụng (M5)', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().error).toContain('HOOK-PRE-COMMIT-REJECTED')
    // Và giữ **cả** phần thân nhiều dòng: người dùng cần đọc dòng nào hỏng, không chỉ
    // biết là có hook nào đó đã từ chối.
    expect(useCommitStore.getState().error).toContain('src/b.ts:4')
  })

  it('lỗi hook được ĐÁNH DẤU là đầu ra hook, để giao diện hiện trong <pre>', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().errorIsHookOutput).toBe(true)
  })

  it('lỗi KHÔNG phải hook thì không bị đánh dấu là đầu ra hook', async () => {
    // Tiền đề của test trên: cờ phải phân biệt được, không phải luôn `true`.
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')
    createCommit.mockRejectedValue({
      code: 'index_locked',
      message: 'đang chờ một tiến trình git khác',
      command: 'git commit -F -',
    } satisfies GitErrorPayload)

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().errorIsHookOutput).toBe(false)
    expect(useCommitStore.getState().error).toContain('đang chờ một tiến trình git khác')
  })

  it('thất bại → trả false', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await expect(useCommitStore.getState().commit(REPO_A)).resolves.toBe(false)
  })
})

describe('thông điệp rỗng (M4)', () => {
  it('🔴 thông điệp rỗng → lỗi, và KHÔNG gọi IPC', async () => {
    useCommitStore.getState().setDraft(REPO_A, '')

    await useCommitStore.getState().commit(REPO_A)

    expect(createCommit).not.toHaveBeenCalled()
    expect(useCommitStore.getState().error).not.toBeNull()
  })

  it('chỉ khoảng trắng và xuống dòng cũng là rỗng, và KHÔNG gọi IPC', async () => {
    useCommitStore.getState().setDraft(REPO_A, '   \n\t  \n ')

    await useCommitStore.getState().commit(REPO_A)

    expect(createCommit).not.toHaveBeenCalled()
  })

  it('KHÔNG tự sửa thông điệp — nháp giữ nguyên khoảng trắng người dùng gõ (R5)', async () => {
    // "Không tự sửa thông điệp" nghĩa là không trim rồi gửi, cũng không trim tại chỗ
    // trong ô soạn dưới tay người dùng.
    useCommitStore.getState().setDraft(REPO_A, '   \n  ')

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('   \n  ')
  })
})

describe('amend — WORK-09', () => {
  it('gọi amendCommit chứ không phải createCommit', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'sửa lại thông điệp cũ')

    await useCommitStore.getState().amend(REPO_A)

    expect(amendCommit).toHaveBeenCalledWith(REPO_A, 'sửa lại thông điệp cũ', false)
    expect(createCommit).not.toHaveBeenCalled()
  })

  it('🔴 wasPushed=true → amend VẪN CHẠY, store chỉ GHI LẠI cảnh báo (M9)', async () => {
    // Nguyên văn ROADMAP: cảnh báo, **KHÔNG chặn**. Dễ bị "sửa" thành chặn bởi một
    // người nghĩ mình đang giúp.
    useCommitStore.getState().setDraft(REPO_A, 'sửa commit đã push')
    amendCommit.mockResolvedValue({ status: trangThai(), wasPushed: true })

    const ok = await useCommitStore.getState().amend(REPO_A)

    expect(amendCommit).toHaveBeenCalledTimes(1)
    expect(ok).toBe(true)
    expect(useCommitStore.getState().wasPushed).toBe(true)
  })

  it('wasPushed=true vẫn xoá nháp — thao tác đã THÀNH CÔNG, cảnh báo không đổi điều đó', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'sửa commit đã push')
    amendCommit.mockResolvedValue({ status: trangThai(), wasPushed: true })

    await useCommitStore.getState().amend(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('')
  })

  it('wasPushed=false → không cảnh báo', async () => {
    // Tiền đề: cờ phải phân biệt được, không phải luôn `true` (M11 nhìn từ phía này).
    useCommitStore.getState().setDraft(REPO_A, 'sửa commit chưa push')
    amendCommit.mockResolvedValue({ status: trangThai(), wasPushed: false })

    await useCommitStore.getState().amend(REPO_A)

    expect(useCommitStore.getState().wasPushed).toBe(false)
  })

  it('amend thất bại → nháp còn nguyên, giống hệt đường commit', async () => {
    const congSuc = 'thông điệp sửa lại'
    useCommitStore.getState().setDraft(REPO_A, congSuc)
    amendCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await useCommitStore.getState().amend(REPO_A)

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe(congSuc)
  })
})

describe('🔴 bấm đôi chỉ gửi MỘT commit (M13)', () => {
  it('lời gọi thứ hai bị bỏ khi lời gọi thứ nhất còn bay', async () => {
    // Một cú bấm đôi trên nút commit là hai commit, và commit thứ hai sẽ rỗng hoặc
    // lấy mất thứ người dùng không định commit.
    let moKhoa: (v: RepoStatus) => void = () => {}
    createCommit.mockReturnValue(
      new Promise<RepoStatus>((res) => {
        moKhoa = res
      }),
    )
    useCommitStore.getState().setDraft(REPO_A, 'một commit thôi')

    const mot = useCommitStore.getState().commit(REPO_A)
    const hai = useCommitStore.getState().commit(REPO_A)

    expect(createCommit).toHaveBeenCalledTimes(1)

    moKhoa(trangThai())
    await Promise.all([mot, hai])
  })

  it('isCommitting bật lúc đang bay và tắt khi xong', async () => {
    let moKhoa: (v: RepoStatus) => void = () => {}
    createCommit.mockReturnValue(
      new Promise<RepoStatus>((res) => {
        moKhoa = res
      }),
    )
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')

    const p = useCommitStore.getState().commit(REPO_A)
    expect(useCommitStore.getState().isCommitting).toBe(true)

    moKhoa(trangThai())
    await p
    expect(useCommitStore.getState().isCommitting).toBe(false)
  })

  it('isCommitting tắt cả khi THẤT BẠI — nếu không nút kẹt vĩnh viễn', async () => {
    useCommitStore.getState().setDraft(REPO_A, 'thông điệp')
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))

    await useCommitStore.getState().commit(REPO_A)

    expect(useCommitStore.getState().isCommitting).toBe(false)
  })
})

describe('nháp bền vững qua plugin store — nửa "đóng mở lại ứng dụng" của tiêu chí 6', () => {
  it('gõ → ghi xuống đĩa với khoá CHỨA repoId (M7)', async () => {
    vi.useFakeTimers()
    useCommitStore.getState().setDraft(REPO_A, 'sẽ còn sau khi đóng ứng dụng')

    await vi.advanceTimersByTimeAsync(DRAFT_DEBOUNCE_MS)
    await vi.runAllTimersAsync()

    expect(store.set).toHaveBeenCalledWith(draftKey(REPO_A), 'sẽ còn sau khi đóng ứng dụng')
    expect(draftKey(REPO_A)).toContain(REPO_A)
  })

  it('🔴 plugin store NÉM LỖI → store vẫn dùng được, nháp vẫn sống trong phiên', async () => {
    // Nháp là tiện lợi, không phải dữ liệu không được mất. Một tệp store hỏng không
    // được làm sập vùng soạn commit.
    vi.useFakeTimers()
    moStore.mockRejectedValue(new Error('tệp store hỏng'))
    __resetDraftForTests()

    useCommitStore.getState().setDraft(REPO_A, 'vẫn gõ được')
    await vi.advanceTimersByTimeAsync(DRAFT_DEBOUNCE_MS)
    await vi.runAllTimersAsync()

    expect(useCommitStore.getState().draftByRepo[REPO_A]).toBe('vẫn gõ được')
  })
})
