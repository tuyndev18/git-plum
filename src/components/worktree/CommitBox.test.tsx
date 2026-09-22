/**
 * `CommitBox` — WORK-08, WORK-09.
 *
 * # Điều test này KHÔNG chứng minh được (đọc trước khi tin nó xanh)
 *
 * happy-dom không tính CSS layout (`CONTEXT.md` 3.4). Nên không test nào ở đây nói
 * được rằng textarea đủ cao để soạn một thông điệp nhiều dòng, rằng khối `<pre>` lỗi
 * hook không đẩy nút commit ra khỏi khung, hay rằng vùng soạn không tràn khi danh sách
 * tệp dài. Năm lỗi hiển thị của Phase 3 đi qua hết 435 test tự động; những điều đó là
 * việc của checkpoint 04-05.
 *
 * # `waitFor` neo vào phần tử CHỈ tồn tại ở trạng thái đang kiểm
 *
 * Lỗi #6 của `CONTEXT.md` 3.1 neo vào vỏ panel có mặt ở **mọi** trạng thái nên thoả
 * mãn tức thì, đỏ 1/5 lần, và sống qua cả một wave. Ở đây không có phép chờ nào neo
 * vào `commit-box` (nó có mặt ngay từ lần render đầu); mọi phép chờ neo vào
 * `commit-hook-error`, `amend-warning` hoặc một giá trị `mock.calls` cụ thể.
 */

import { act, cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { clearCommands } from '@/lib/commands'
import { __resetDraftForTests } from '@/lib/commitDraft'
import type { AmendResult, GitErrorPayload, RepoStatus, StatusEntry } from '@/lib/ipc'
import { useCommitStore } from '@/stores/commitStore'
import { useHistoryStore } from '@/stores/historyStore'
import { useStatusStore } from '@/stores/statusStore'

import { CommitBox } from './CommitBox'

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

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(async () => ({
    get: vi.fn(async () => undefined),
    set: vi.fn(async () => undefined),
    delete: vi.fn(async () => undefined),
    save: vi.fn(async () => undefined),
  })),
}))

const REPO = 'repo-1'

/** Nguyên văn đầu ra hook của fixture Task 1 — chuỗi **nhận diện được** (M5). */
const HOOK_NGUYEN_VAN =
  'HOOK-PRE-COMMIT-REJECTED\n  src/a.ts:12  thiếu dấu chấm phẩy\n  src/b.ts:4   biến không dùng'

function phanTu(over: Partial<StatusEntry> = {}): StatusEntry {
  return { path: 'a.txt', oldPath: null, xy: 'M.', group: 'staged', hasInvalidUtf8: false, ...over }
}

function trangThai(entries: StatusEntry[]): RepoStatus {
  return {
    branch: { head: 'main', oid: 'a'.repeat(40), upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

function loiHook(nguyenVan: string): GitErrorPayload {
  return { code: 'hook_rejected', message: nguyenVan, command: 'git commit -F -' }
}

/**
 * Bấm một phần tử và để React xử lý xong mọi cập nhật trạng thái sinh ra.
 *
 * `act` quanh cú bấm là bắt buộc ở React 19: `onClick` ở đây khởi động một chuỗi
 * `await` (sổ lệnh → store → IPC), và không bọc `act` thì phép khẳng định ngay sau
 * đó đo **trước** khi React flush — đúng khiếm khuyết B của mutation #8 ở plan 03-04,
 * nơi test chống đua xanh kể cả trên cài đặt đã bị đột biến vì nó đo quá sớm.
 */
async function bam(el: HTMLElement): Promise<void> {
  await act(async () => {
    el.click()
  })
}

/** Render với một repo có **một tệp đã stage** — trạng thái commit được. */
function dungCanh(entries: StatusEntry[] = [phanTu()]) {
  useStatusStore.setState({
    byRepo: { [REPO]: { status: trangThai(entries), isLoading: false, error: null } },
  })
  return render(<CommitBox repoId={REPO} />)
}

beforeEach(() => {
  vi.clearAllMocks()
  clearCommands()
  __resetDraftForTests()
  useCommitStore.setState({
    draftByRepo: {},
    isCommitting: false,
    error: null,
    errorIsHookOutput: false,
    noVerify: false,
    amendMode: false,
    wasPushed: false,
  })
  useStatusStore.setState({ byRepo: {} })
  useHistoryStore.setState({ byRepo: {} })
  createCommit.mockResolvedValue(trangThai([]))
  amendCommit.mockResolvedValue({ status: trangThai([]), wasPushed: false })
})

afterEach(() => {
  cleanup()
  clearCommands()
  __resetDraftForTests()
})

describe('textarea và nháp', () => {
  it('hiện nháp từ commitStore', () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'nháp đã có' } })
    dungCanh()

    expect((screen.getByTestId('commit-message') as HTMLTextAreaElement).value).toBe('nháp đã có')
  })

  it('gõ vào textarea cập nhật nháp trong store', async () => {
    dungCanh()

    fireEvent.change(screen.getByTestId('commit-message'), { target: { value: 'feat: abc' } })

    expect(useCommitStore.getState().draftByRepo[REPO]).toBe('feat: abc')
  })

  it('nháp rỗng hiện chuỗi rỗng, KHÔNG hiện chữ "undefined"', () => {
    dungCanh()

    expect((screen.getByTestId('commit-message') as HTMLTextAreaElement).value).toBe('')
  })
})

describe('nút commit vô hiệu — và NÓI LÝ DO', () => {
  it('thông điệp rỗng → nút vô hiệu VÀ lý do hiện trên màn hình', () => {
    dungCanh()

    expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(true)
    // Nút xám không rõ vì sao là một người dùng bị kẹt.
    expect(screen.getByTestId('commit-disabled-reason')).toBeTruthy()
  })

  it('thông điệp chỉ khoảng trắng → vẫn vô hiệu', () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: '   \n  ' } })
    dungCanh()

    expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(true)
  })

  it('không tệp nào đã stage → vô hiệu, và lý do nói HÀNH ĐỘNG tiếp theo', () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'có thông điệp' } })
    dungCanh([phanTu({ group: 'unstaged', xy: '.M' })])

    expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(true)
    expect(screen.getByTestId('commit-disabled-reason').textContent).toMatch(/stage/i)
  })

  it('có thông điệp VÀ có tệp đã stage → nút bấm được, không còn lý do vô hiệu', () => {
    // Tiền đề của mọi test bấm nút bên dưới: nếu nút luôn vô hiệu thì chúng vô nghĩa.
    useCommitStore.setState({ draftByRepo: { [REPO]: 'feat: xong' } })
    dungCanh()

    expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(false)
    expect(screen.queryByTestId('commit-disabled-reason')).toBeNull()
  })
})

describe('công tắc no-verify (M1, M2)', () => {
  it('🔴 mặc định TẮT — hook chạy mặc định (ràng buộc 2.4)', () => {
    dungCanh()

    expect((screen.getByTestId('no-verify-toggle') as HTMLInputElement).checked).toBe(false)
  })

  it('nhãn nói rõ nó BỎ QUA HOOK, không phải một chữ viết tắt', () => {
    dungCanh()

    // Người dùng phải đọc được hậu quả trước khi bật.
    expect(screen.getByLabelText(/bỏ qua hook/i)).toBeTruthy()
  })

  it('🔴 tắt → createCommit nhận noVerify=false', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'để hook chạy' } })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(createCommit).toHaveBeenCalledTimes(1))
    expect(createCommit).toHaveBeenCalledWith(REPO, 'để hook chạy', false)
  })

  it('🔴 bật → createCommit nhận noVerify=true (công tắc thật, không phải ô tick trang trí)', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'bỏ qua hook' } })
    dungCanh()

    await bam(screen.getByTestId('no-verify-toggle'))
    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(createCommit).toHaveBeenCalledTimes(1))
    expect(createCommit).toHaveBeenCalledWith(REPO, 'bỏ qua hook', true)
  })
})

describe('🔴 hook từ chối (M5, M6)', () => {
  it('hiện NGUYÊN VĂN đầu ra hook trên màn hình', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'thông điệp' } })
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    // Neo vào phần tử CHỈ tồn tại khi có lỗi hook — không neo vào `commit-box`.
    const khoi = await screen.findByTestId('commit-hook-error')
    expect(khoi.textContent).toContain('HOOK-PRE-COMMIT-REJECTED')
    // Và cả phần thân: người dùng cần biết dòng nào hỏng.
    expect(khoi.textContent).toContain('src/b.ts:4')
  })

  it('đầu ra hook hiện trong <pre> — xuống dòng và thụt lề mang nghĩa', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'thông điệp' } })
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    const khoi = await screen.findByTestId('commit-hook-error')
    expect(khoi.tagName).toBe('PRE')
  })

  it('🔴 hook từ chối → textarea VẪN GIỮ thông điệp người dùng vừa gõ', async () => {
    const congSuc = 'feat: thông điệp dài mà người dùng vừa bỏ công viết'
    useCommitStore.setState({ draftByRepo: { [REPO]: congSuc } })
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await screen.findByTestId('commit-hook-error')
    expect((screen.getByTestId('commit-message') as HTMLTextAreaElement).value).toBe(congSuc)
  })

  it('hook từ chối → đồ thị KHÔNG được làm mới (không có commit mới)', async () => {
    const loadFirstPage = vi.fn(async () => undefined)
    useHistoryStore.setState({ loadFirstPage })
    useCommitStore.setState({ draftByRepo: { [REPO]: 'thông điệp' } })
    createCommit.mockRejectedValue(loiHook(HOOK_NGUYEN_VAN))
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await screen.findByTestId('commit-hook-error')
    expect(loadFirstPage).not.toHaveBeenCalled()
  })
})

describe('commit thành công', () => {
  it('textarea rỗng sau khi commit', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'feat: xong việc' } })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect((screen.getByTestId('commit-message') as HTMLTextAreaElement).value).toBe(''))
  })

  it('🔴 đồ thị được làm mới — tiêu chí 3', async () => {
    const reset = vi.fn()
    const loadFirstPage = vi.fn(async () => undefined)
    useHistoryStore.setState({ reset, loadFirstPage })
    useCommitStore.setState({ draftByRepo: { [REPO]: 'feat: xong việc' } })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(loadFirstPage).toHaveBeenCalledWith(REPO))
    expect(reset).toHaveBeenCalledWith(REPO)
  })
})

describe('chế độ amend — WORK-09', () => {
  it('bật amend → nhãn nút ĐỔI', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa thông điệp' } })
    dungCanh()

    const truoc = screen.getByTestId('commit-button').textContent
    await bam(screen.getByTestId('amend-toggle'))

    expect(screen.getByTestId('commit-button').textContent).not.toBe(truoc)
    expect(screen.getByTestId('commit-button').textContent).toMatch(/amend|sửa/i)
  })

  it('bật amend → gọi amendCommit chứ KHÔNG gọi createCommit', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa thông điệp' } })
    dungCanh()

    await bam(screen.getByTestId('amend-toggle'))
    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(amendCommit).toHaveBeenCalledTimes(1))
    expect(createCommit).not.toHaveBeenCalled()
  })

  it('amend KHÔNG đòi có tệp đã stage — sửa riêng thông điệp là ca dùng thường nhất', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'chỉ sửa thông điệp' }, amendMode: true })
    dungCanh([])

    expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(false)

    await bam(screen.getByTestId('commit-button'))
    await waitFor(() => expect(amendCommit).toHaveBeenCalledTimes(1))
  })
})

describe('🔴 amend trên commit đã push: CẢNH BÁO, KHÔNG CHẶN (M9)', () => {
  it('wasPushed=true → cảnh báo HIỆN', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa commit đã push' }, amendMode: true })
    amendCommit.mockResolvedValue({ status: trangThai([]), wasPushed: true })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    // Neo vào phần tử CHỈ tồn tại khi có cảnh báo.
    await screen.findByTestId('amend-warning')
  })

  it('🔴 cảnh báo hiện rồi, nút VẪN BẤM ĐƯỢC và amend VẪN CHẠY lần nữa', async () => {
    // Nguyên văn ROADMAP: cảnh báo, **KHÔNG chặn**. Dễ bị "sửa" thành chặn bởi một
    // người nghĩ mình đang giúp. Phép khẳng định then chốt là amend được gọi **sau
    // khi** cảnh báo đã hiện — không chỉ là cảnh báo có tồn tại.
    useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa commit đã push' }, amendMode: true })
    amendCommit.mockResolvedValue({ status: trangThai([]), wasPushed: true })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))
    await screen.findByTestId('amend-warning')
    expect(amendCommit).toHaveBeenCalledTimes(1)

    // Cảnh báo đang hiện. Gõ lại rồi bấm lần nữa.
    //
    // `act` quanh `setState` là bắt buộc: component đăng ký store qua
    // `useCommitStore(selector)`, và một `setState` ngoài `act` chưa flush lúc
    // phép khẳng định ngay sau đó chạy — nó sẽ đọc DOM của lần render TRƯỚC (ô soạn
    // còn rỗng sau khi amend thành công) và báo nút vô hiệu một cách sai lệch.
    // Đã chẩn đoán bằng số: store giữ 'sửa tiếp lần hai' trong khi textarea còn ''.
    await act(async () => {
      useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa tiếp lần hai' } })
    })
    expect(screen.getByTestId('amend-warning')).toBeTruthy()
    expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(false)

    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(amendCommit).toHaveBeenCalledTimes(2))
  })

  it('cảnh báo KHÔNG phải hộp thoại chặn đường — nó là chữ, có role="status"', async () => {
    // Một hộp thoại "bạn có chắc?" là bước đầu của việc chặn. `role="alertdialog"`
    // hoặc một phần tử `<dialog>` ở đây là sai yêu cầu.
    useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa commit đã push' }, amendMode: true })
    amendCommit.mockResolvedValue({ status: trangThai([]), wasPushed: true })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    const canhBao = await screen.findByTestId('amend-warning')
    expect(canhBao.getAttribute('role')).toBe('status')
    expect(canhBao.tagName).not.toBe('DIALOG')
  })

  it('wasPushed=false → KHÔNG hiện cảnh báo (tiền đề: cảnh báo phân biệt được)', async () => {
    useCommitStore.setState({ draftByRepo: { [REPO]: 'sửa commit chưa push' }, amendMode: true })
    amendCommit.mockResolvedValue({ status: trangThai([]), wasPushed: false })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(amendCommit).toHaveBeenCalledTimes(1))
    expect(screen.queryByTestId('amend-warning')).toBeNull()
  })
})

describe('🔴 bấm đôi chỉ gửi một commit (M13)', () => {
  it('nút VÔ HIỆU trong lúc đang commit', async () => {
    let moKhoa: (v: RepoStatus) => void = () => {}
    createCommit.mockReturnValue(
      new Promise<RepoStatus>((res) => {
        moKhoa = res
      }),
    )
    useCommitStore.setState({ draftByRepo: { [REPO]: 'một commit thôi' } })
    dungCanh()

    await bam(screen.getByTestId('commit-button'))

    await waitFor(() => expect(screen.getByTestId('commit-button').hasAttribute('disabled')).toBe(true))

    moKhoa(trangThai([]))
    await waitFor(() => expect(createCommit).toHaveBeenCalledTimes(1))
  })

  it('bấm hai lần liên tiếp → IPC chỉ được gọi MỘT lần', async () => {
    let moKhoa: (v: RepoStatus) => void = () => {}
    createCommit.mockReturnValue(
      new Promise<RepoStatus>((res) => {
        moKhoa = res
      }),
    )
    useCommitStore.setState({ draftByRepo: { [REPO]: 'một commit thôi' } })
    dungCanh()

    const nut = screen.getByTestId('commit-button')
    await bam(nut)
    await bam(nut)

    expect(createCommit).toHaveBeenCalledTimes(1)

    moKhoa(trangThai([]))
    await waitFor(() => expect(useCommitStore.getState().isCommitting).toBe(false))
  })
})

describe('không có repoId', () => {
  it('không render gì — mọi thứ ở đây khoá theo repoId (PLAT-05)', () => {
    render(<CommitBox />)

    expect(screen.queryByTestId('commit-box')).toBeNull()
  })
})
