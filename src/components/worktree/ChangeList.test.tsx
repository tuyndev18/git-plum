/**
 * Test `ChangeList` — WORK-01 (ba nhóm tệp, chọn tệp mở diff) và WORK-02 (stage /
 * unstage theo tệp).
 *
 * # Cổng chịu lực của tệp này: `nut_stage_van_bam_duoc_khi_diff_hong`
 *
 * Đó là đột biến **M9**, đột biến quan trọng nhất của plan 04-02. Nó ghim ràng buộc
 * R8 — thứ duy nhất ngăn vòng commit chết theo một trình xem diff mà **chưa ai dùng
 * thật**, và mà chủ dự án đã tìm thấy năm lỗi hiển thị chỉ bằng cách mở ứng dụng.
 *
 * # ⚠️ Không test nào ở đây kiểm được BỐ CỤC
 *
 * happy-dom không tính CSS layout và không có cuộn thật (CONTEXT.md 3.4). "Ba nhóm
 * không tràn", "chiều cao hàng", "nút không đè lên tên tệp dài" là **có mã, chưa
 * kiểm** cho tới khi có người xem trên Chromium thật (checkpoint 04-05).
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'

import { ChangeList } from '@/components/worktree/ChangeList'
import { clearCommands } from '@/lib/commands'
import { ipc, type RepoStatus, type StatusEntry } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import { useStatusStore } from '@/stores/statusStore'

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
  }
})

const getStatusMock = vi.mocked(ipc.getStatus)
const stageMock = vi.mocked(ipc.stageFiles)
const unstageMock = vi.mocked(ipc.unstageFiles)
const worktreeDiffMock = vi.mocked(ipc.getWorktreeDiff)

const REPO = 'repo-1'

function muc(path: string, xy: string, group: StatusEntry['group']): StatusEntry {
  return { path, oldPath: null, xy, group, hasInvalidUtf8: false }
}

function status(entries: StatusEntry[]): RepoStatus {
  return {
    branch: { head: 'main', oid: 'a'.repeat(40), upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

/** 2 staged, 3 unstaged, 1 untracked — đúng hình dạng plan yêu cầu: sáu hàng. */
const SAU_TEP = status([
  muc('s1.txt', 'M.', 'staged'),
  muc('s2.txt', 'A.', 'staged'),
  muc('u1.txt', '.M', 'unstaged'),
  muc('u2.txt', '.M', 'unstaged'),
  muc('u3.txt', '.D', 'unstaged'),
  muc('n1.txt', '??', 'untracked'),
])

beforeEach(() => {
  clearCommands()
  getStatusMock.mockReset()
  stageMock.mockReset()
  unstageMock.mockReset()
  worktreeDiffMock.mockReset()

  getStatusMock.mockResolvedValue(SAU_TEP)
  stageMock.mockResolvedValue(SAU_TEP)
  unstageMock.mockResolvedValue(SAU_TEP)
  worktreeDiffMock.mockResolvedValue({
    path: 'u1.txt',
    oldPath: null,
    status: 'M',
    kind: { kind: 'unchanged' },
  })

  useStatusStore.setState({ byRepo: {} })
  useDiffStore.setState({ selectedFileByRepo: {} })
})

/**
 * Chờ danh sách đã nạp xong.
 *
 * 🔴 Neo vào **một hàng tệp**, không vào `change-list` (vỏ panel).
 *
 * Lỗi cổng thứ **sáu** của CONTEXT.md 3.1 là đúng chuyện này: `waitFor` neo vào một
 * vỏ panel có mặt ở **mọi** trạng thái kể cả lúc đang nạp, nên nó thoả mãn **tức
 * thì** và cổng đỏ 1-trên-5 lần, sống qua cả một wave. `change-row` chỉ tồn tại sau
 * khi `getStatus` đã trả về, nên nó phân biệt được "đã nạp" với "đang nạp".
 */
async function choNapXong() {
  await waitFor(() => expect(screen.getAllByTestId('change-row').length).toBeGreaterThan(0))
}

describe('ba nhóm tệp (WORK-01 tiêu chí 1)', () => {
  it('2 staged + 3 unstaged + 1 untracked → ba tiêu đề và sáu hàng', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    expect(screen.getAllByTestId('change-row')).toHaveLength(6)

    expect(screen.getByTestId('change-group-staged')).toBeTruthy()
    expect(screen.getByTestId('change-group-unstaged')).toBeTruthy()
    expect(screen.getByTestId('change-group-untracked')).toBeTruthy()

    expect(screen.getByText(/Đã stage/)).toBeTruthy()
    expect(screen.getByText(/Chưa stage/)).toBeTruthy()
    expect(screen.getByText(/Chưa theo dõi/)).toBeTruthy()
  })

  it('mỗi hàng ở đúng nhóm của nó', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    const trongNhom = (g: string) =>
      Array.from(screen.getByTestId(`change-group-${g}`).querySelectorAll('[data-path]')).map(
        (el) => el.getAttribute('data-path'),
      )

    expect(trongNhom('staged')).toEqual(['s1.txt', 's2.txt'])
    expect(trongNhom('unstaged')).toEqual(['u1.txt', 'u2.txt', 'u3.txt'])
    expect(trongNhom('untracked')).toEqual(['n1.txt'])
  })

  /**
   * 🔴 **Đột biến M8: nhóm rỗng KHÔNG hiện tiêu đề.**
   *
   * Ba tiêu đề trên một repo gần sạch là ba dòng nhiễu nói "không có gì" ba lần.
   */
  it('nhóm rỗng không hiện tiêu đề', async () => {
    getStatusMock.mockResolvedValue(status([muc('chi-mot.txt', '.M', 'unstaged')]))

    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    expect(screen.getByTestId('change-group-unstaged')).toBeTruthy()
    expect(screen.queryByTestId('change-group-staged')).toBeNull()
    expect(screen.queryByTestId('change-group-untracked')).toBeNull()
    expect(screen.queryByText(/Đã stage/)).toBeNull()
    expect(screen.queryByText(/Chưa theo dõi/)).toBeNull()
  })

  it('repo sạch → không tiêu đề nào, một câu nói rõ', async () => {
    getStatusMock.mockResolvedValue(status([]))

    render(<ChangeList repoId={REPO} />)

    await waitFor(() => expect(screen.getByTestId('change-list-empty')).toBeTruthy())
    expect(screen.queryByTestId('change-row')).toBeNull()
    expect(screen.queryByTestId('change-group-staged')).toBeNull()
  })

  /**
   * Một tệp `MM` hiện ở **cả hai** nhóm — và bấm stage ở nhóm chưa stage chỉ gửi
   * **một** path (không phải hai, dù tệp xuất hiện hai lần).
   */
  it('tệp MM hiện ở cả hai nhóm, stage gửi đúng một path', async () => {
    getStatusMock.mockResolvedValue(
      status([muc('ca_hai.txt', 'MM', 'staged'), muc('ca_hai.txt', 'MM', 'unstaged')]),
    )

    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    expect(screen.getAllByTestId('change-row')).toHaveLength(2)

    const nhomChuaStage = screen.getByTestId('change-group-unstaged')
    const nut = nhomChuaStage.querySelector<HTMLButtonElement>('[data-testid="stage-button"]')
    expect(nut).toBeTruthy()
    nut!.click()

    await waitFor(() => expect(stageMock).toHaveBeenCalledTimes(1))
    expect(stageMock).toHaveBeenCalledWith(REPO, ['ca_hai.txt'])
  })

  it('tệp đổi tên hiện cả tên cũ và tên mới', async () => {
    getStatusMock.mockResolvedValue(
      status([{ path: 'moi.txt', oldPath: 'cu.txt', xy: 'R.', group: 'staged', hasInvalidUtf8: false }]),
    )

    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    expect(screen.getByText('cu.txt')).toBeTruthy()
    expect(screen.getByText('moi.txt')).toBeTruthy()
  })
})

describe('chọn tệp (WORK-01 tiêu chí 2)', () => {
  it('bấm một hàng → diffStore.selectedFileByRepo[repoId] thành path đó', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    screen.getByText('u1.txt').click()

    await waitFor(() =>
      expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe('u1.txt'),
    )
  })

  it('bấm hàng khác → lựa chọn đổi theo', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    screen.getByText('u1.txt').click()
    await waitFor(() => expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe('u1.txt'))

    screen.getByText('s1.txt').click()
    await waitFor(() => expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe('s1.txt'))
  })
})

describe('stage / unstage theo tệp (WORK-02)', () => {
  it('bấm nút stage của hàng chưa stage → gọi stageFiles MỘT lần với đúng path', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    screen.getByRole('button', { name: 'Stage u2.txt' }).click()

    await waitFor(() => expect(stageMock).toHaveBeenCalledTimes(1))
    expect(stageMock).toHaveBeenCalledWith(REPO, ['u2.txt'])
    expect(unstageMock).not.toHaveBeenCalled()
  })

  it('bấm nút bỏ stage của hàng đã stage → gọi unstageFiles với đúng path', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    screen.getByRole('button', { name: 'Bỏ stage s1.txt' }).click()

    await waitFor(() => expect(unstageMock).toHaveBeenCalledTimes(1))
    expect(unstageMock).toHaveBeenCalledWith(REPO, ['s1.txt'])
    expect(stageMock).not.toHaveBeenCalled()
  })

  /**
   * 🔴 Store nhận `RepoStatus` **từ giá trị trả về**, và **không** gọi lại `getStatus`.
   *
   * Ràng buộc 2.5 của CONTEXT.md ở tầng component. Đo sau khi dọn hàm giả để không
   * lẫn với lần `refresh` lúc mount.
   */
  it('tệp chuyển nhóm ngay từ giá trị trả về, không gọi lại getStatus', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()
    getStatusMock.mockClear()

    // Sau khi stage `u1.txt`, nó về nhóm đã stage.
    stageMock.mockResolvedValue(
      status([
        muc('s1.txt', 'M.', 'staged'),
        muc('s2.txt', 'A.', 'staged'),
        muc('u1.txt', 'M.', 'staged'),
        muc('u2.txt', '.M', 'unstaged'),
        muc('u3.txt', '.D', 'unstaged'),
        muc('n1.txt', '??', 'untracked'),
      ]),
    )

    screen.getByRole('button', { name: 'Stage u1.txt' }).click()

    await waitFor(() => {
      const trongStaged = Array.from(
        screen.getByTestId('change-group-staged').querySelectorAll('[data-path]'),
      ).map((el) => el.getAttribute('data-path'))
      expect(trongStaged).toContain('u1.txt')
    })

    expect(getStatusMock).toHaveBeenCalledTimes(0)
  })

  /**
   * 🔴 **Đột biến M7: bấm nút KHÔNG làm đổi tệp đang chọn.**
   *
   * Một cú bấm = một ý định. Thiếu `stopPropagation`, bấm stage tệp B trong lúc đang
   * đọc diff tệp A làm trình xem nhảy sang B và người dùng mất chỗ đang đọc.
   */
  it('bấm nút stage KHÔNG làm đổi tệp đang chọn', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    // Tiền đề: chọn một tệp trước, để phép khẳng định phân biệt được "không đổi" với
    // "chưa bao giờ đặt". Không có bước này thì test xanh cả khi `stopPropagation`
    // bị xoá nhưng lựa chọn tình cờ trùng.
    screen.getByText('u1.txt').click()
    await waitFor(() => expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe('u1.txt'))

    screen.getByRole('button', { name: 'Stage u2.txt' }).click()
    await waitFor(() => expect(stageMock).toHaveBeenCalledTimes(1))

    expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe(
      'u1.txt',
    )
  })

  /**
   * 🔴 `ipc.stageFiles` **ném lỗi** → hàng vẫn còn, thông báo lỗi hiện, giao diện
   * **không** trắng.
   *
   * Đây là R3 (`index.lock`) ở tầng giao diện: người dùng đang chạy git ở terminal —
   * ca thường, không phải phòng xa. Danh sách biến mất vì một lỗi tạm thời làm họ mất
   * cả chỗ đang đứng lẫn khả năng thử lại.
   */
  it('stageFiles ném lỗi → hàng vẫn còn, lỗi hiện ra, không trắng màn hình', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    stageMock.mockRejectedValueOnce({
      code: 'index_locked',
      message: 'một tiến trình git khác đang giữ .git/index.lock',
      command: 'git add -- u1.txt',
    })

    screen.getByRole('button', { name: 'Stage u1.txt' }).click()

    await waitFor(() => expect(screen.getByRole('alert').textContent).toContain('index.lock'))

    // Sáu hàng vẫn còn — giao diện không trắng và không mất danh sách.
    expect(screen.getAllByTestId('change-row')).toHaveLength(6)
    expect(screen.getByText('u1.txt')).toBeTruthy()
  })

  /** Bấm nút hai lần → hai lời gọi, mỗi lời một path (không gộp, không nuốt). */
  it('bấm stage hai tệp khác nhau gửi hai lời gọi riêng', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    screen.getByRole('button', { name: 'Stage u1.txt' }).click()
    await waitFor(() => expect(stageMock).toHaveBeenCalledTimes(1))
    screen.getByRole('button', { name: 'Stage u2.txt' }).click()
    await waitFor(() => expect(stageMock).toHaveBeenCalledTimes(2))

    expect(stageMock.mock.calls[0]![1]).toEqual(['u1.txt'])
    expect(stageMock.mock.calls[1]![1]).toEqual(['u2.txt'])
  })
})

describe('🔴 R8: diff hỏng KHÔNG được chặn stage (đột biến M9)', () => {
  /**
   * Đột biến **quan trọng nhất của plan 04-02**.
   *
   * `CONTEXT.md` mục 0: trình xem diff của Phase 3 **chưa ai dùng thật**, và chủ dự án
   * đã tìm **năm** lỗi hiển thị của nó chỉ bằng cách mở ứng dụng — cả năm đi qua toàn
   * bộ test tự động. Nên xác suất còn lỗi hiển thị chưa biết là **cao**, và vòng commit
   * **không** được chết theo nó.
   *
   * Nếu đột biến này cho 0 test đỏ thì `ChangeList` đang lệ thuộc diff và plan chưa xong.
   */
  it('getWorktreeDiff ném lỗi → nút stage VẪN bấm được và vẫn gọi stageFiles', async () => {
    worktreeDiffMock.mockRejectedValue(new Error('trình xem diff hỏng hoàn toàn'))

    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    // Chọn tệp trước — đây là đường mà diff sẽ được yêu cầu và sẽ hỏng.
    screen.getByText('u1.txt').click()
    await waitFor(() => expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe('u1.txt'))

    const nut = screen.getByRole('button', { name: 'Stage u1.txt' })
    expect((nut as HTMLButtonElement).disabled).toBe(false)

    nut.click()

    await waitFor(() => expect(stageMock).toHaveBeenCalledTimes(1))
    expect(stageMock).toHaveBeenCalledWith(REPO, ['u1.txt'])
  })

  /**
   * Nút stage **không bao giờ** `disabled` — bất kể trạng thái nào của trình xem diff.
   *
   * Khẳng định trên **mọi** hàng, không chỉ hàng đang chọn: một cài đặt vô hiệu nút
   * "của tệp đang xem" khi diff hỏng vẫn để các nút khác bật, nên một test chỉ nhìn
   * một nút có thể trượt.
   */
  it('không nút stage/unstage nào bị vô hiệu, kể cả khi diff hỏng', async () => {
    worktreeDiffMock.mockRejectedValue(new Error('diff hỏng'))

    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    const nut = screen.getAllByRole('button')
    expect(nut.length).toBe(6)
    for (const n of nut) {
      expect((n as HTMLButtonElement).disabled).toBe(false)
    }
  })

  /**
   * `ChangeList` **không** gọi `getWorktreeDiff` — nó chỉ ghi lựa chọn vào `diffStore`.
   *
   * Đây là cách R8 được cài ở mức cấu trúc, không chỉ ở mức hành vi: nếu component này
   * tự nạp diff thì một diff hỏng sống trong **cùng** cây con với nút stage, và một
   * lỗi render ở đó gỡ luôn cả nút. Việc nạp diff thuộc về trình xem, ở một nhánh khác.
   */
  it('ChangeList không tự gọi getWorktreeDiff', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    screen.getByText('u1.txt').click()
    await waitFor(() => expect(useDiffStore.getState().selectedFileByRepo[REPO]).toBe('u1.txt'))

    expect(worktreeDiffMock).toHaveBeenCalledTimes(0)
  })
})

describe('nạp và lỗi', () => {
  it('chưa truyền repoId → không render gì và không gọi IPC', () => {
    const { container } = render(<ChangeList />)

    expect(container.querySelector('[data-testid="change-list"]')).toBeNull()
    expect(getStatusMock).toHaveBeenCalledTimes(0)
  })

  it('getStatus ném lỗi → thông báo hiện ra, không trắng màn hình', async () => {
    getStatusMock.mockRejectedValue(new Error('không phải repository git'))

    render(<ChangeList repoId={REPO} />)

    await waitFor(() =>
      expect(screen.getByRole('alert').textContent).toContain('repository'),
    )
  })

  it('mount gọi refresh đúng một lần cho repo', async () => {
    render(<ChangeList repoId={REPO} />)
    await choNapXong()

    expect(getStatusMock).toHaveBeenCalledTimes(1)
    expect(getStatusMock).toHaveBeenCalledWith(REPO)
  })
})
