/**
 * Test cho `HunkTable` — **đường vào thứ hai** của staging theo khối.
 *
 * # 🔴 Cổng chịu lực của tệp này: `hunktable_render_mot_minh_khong_phase3_khong_phase4`
 *
 * Đó là đột biến **M28**, và nó là đột biến quan trọng nhất của plan 05-05.
 *
 * `CONTEXT.md` mục 0, hệ quả 2, nguyên văn:
 *
 * > *"Tầng giao diện của Phase 5 phải chịu được việc Phase 4 còn lỗi. Nếu `ChangeList`
 * > hoá ra hiển thị sai, staging theo khối vẫn phải dùng được từ một đường vào khác."*
 *
 * Wave 4 đóng được mệnh đề **một** (`HunkBar` **có thể** sống không cần Phase 4, ghim
 * bằng M25/M25b) và ghi thẳng rằng mệnh đề **hai** còn để ngỏ:
 *
 * > *"Người dùng **có** một đường tới nó không qua Phase 4 — ❌ Chưa, và chưa có
 * > đường nào cả. […] nếu đường vào duy nhất của staging theo khối là 'mở diff từ
 * > `ChangeList`', thì toàn bộ công của wave này **không mua được gì** — component
 * > sạch mà đường tới nó thì không."*
 *
 * `HunkTable` là mệnh đề hai. Nó tự nạp diff thư mục làm việc, tự dựng bảng khối, và
 * **không nhập** một module nào của Phase 3 (CodeMirror, `hunksToDoc`, `DiffViewer`)
 * hay Phase 4 (`ChangeList`, `CommitBox`).
 *
 * # ⚠️ Không test nào ở đây kiểm được BỐ CỤC
 *
 * happy-dom không tính CSS layout (`CONTEXT.md` 4.5) — checkpoint 05-05.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { HunkTable } from '@/components/worktree/HunkTable'
import { ipc, type FileDiff, type Hunk, type RepoStatus } from '@/lib/ipc'
import { useHunkStore } from '@/stores/hunkStore'
import { useStatusStore } from '@/stores/statusStore'

vi.mock('@/lib/ipc', async () => {
  const thuc = await vi.importActual<typeof import('@/lib/ipc')>('@/lib/ipc')
  return {
    ...thuc,
    ipc: {
      getStatus: vi.fn(),
      getFileDiff: vi.fn(),
      stageFiles: vi.fn(),
      unstageFiles: vi.fn(),
      getWorktreeDiff: vi.fn(),
      stageHunk: vi.fn(),
      unstageHunk: vi.fn(),
      discardHunk: vi.fn(),
      discardFiles: vi.fn(),
      listTrash: vi.fn(),
      restoreTrash: vi.fn(),
    },
  }
})

const worktreeDiffMock = vi.mocked(ipc.getWorktreeDiff)
const stageHunkMock = vi.mocked(ipc.stageHunk)

const REPO = 'repo-1'
const PATH = 'src/a.txt'

function trangThaiRong(): RepoStatus {
  return {
    branch: { head: 'main', oid: null, upstream: null, ahead: null, behind: null },
    entries: [],
    hasConflicts: false,
  }
}

function khoi(heading: string, oldStart: number, newStart: number): Hunk {
  return {
    oldStart,
    oldCount: 3,
    newStart,
    newCount: 3,
    heading,
    lines: [
      {
        kind: 'context',
        content: 'giu nguyen',
        oldLine: oldStart,
        newLine: newStart,
        noNewlineAtEof: false,
        spans: [],
      },
      {
        kind: 'removed',
        content: 'cu',
        oldLine: oldStart + 1,
        newLine: null,
        noNewlineAtEof: false,
        spans: [],
      },
      {
        kind: 'added',
        content: 'moi',
        oldLine: null,
        newLine: newStart + 1,
        noNewlineAtEof: false,
        spans: [],
      },
    ],
  } as unknown as Hunk
}

/** Diff **ba** khối — xem chú thích của `BA_KHOI` dưới. */
function diffBaKhoi(): FileDiff {
  return {
    path: PATH,
    oldPath: null,
    status: 'M',
    kind: {
      kind: 'text',
      hunks: [khoi('fn mot', 10, 10), khoi('fn hai', 40, 40), khoi('fn ba', 90, 90)],
      truncated: false,
      contextOnly: false,
    },
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  useStatusStore.setState({ byRepo: {} })
  useHunkStore.setState({ dangChon: null, canLamMoi: null })
  worktreeDiffMock.mockResolvedValue(diffBaKhoi())
  stageHunkMock.mockResolvedValue(trangThaiRong())
})

describe('HunkTable — đường vào thứ hai, ĐỘC LẬP Phase 3 và Phase 4', () => {
  /**
   * 🔴 **Test 4 của plan. Đột biến M28 phải làm test này đỏ.**
   *
   * Render `HunkTable` **một mình**: không `DiffViewer`, không CodeMirror, không
   * `ChangeList`, không provider nào, và `statusStore` **rỗng** — tức mọi store của
   * Phase 4 ở trạng thái "chưa nạp".
   *
   * Nếu bảng khối đọc một store của `DiffViewer`/Phase 4, nó sẽ không render nút hoặc
   * vô hiệu nút ở đây, và test đỏ. Đó là đúng hình dạng vòng khoá chết bốn điều kiện
   * mà wave 5 Phase 4 tìm được.
   *
   * Khuôn chép từ `nut_stage_van_bam_duoc_khi_diff_hong` của 04-02 và
   * `hunkbar_render_mot_minh_khong_co_phase4` của 05-04.
   */
  it('🔴 M28: render MỘT MÌNH, statusStore rỗng, không component Phase 3/4 nào → nút khối vẫn bấm được và vẫn gọi IPC', async () => {
    expect(useStatusStore.getState().byRepo).toEqual({})

    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)

    await waitFor(() => expect(screen.getAllByTestId('hunk-table-row')).toHaveLength(3))

    const nut = screen.getAllByTestId('hunk-nut-vung-cho')
    expect(nut).toHaveLength(3)

    // Phép kiểm **vô hiệu** là nửa mang tải: một cài đặt vẫn *render* nút nhưng
    // `disabled` khi store Phase 4 chưa có dữ liệu sẽ qua được `toHaveLength(3)`.
    // Đó chính là đột biến M25b của wave 4, và nó là lỗi thật một lập trình viên
    // viết ("tôi vô hiệu cho an toàn tới khi biết tệp có theo dõi không").
    for (const n of nut) expect((n as HTMLButtonElement).disabled).toBe(false)

    fireEvent.click(nut[1]!)
    await waitFor(() => expect(stageHunkMock).toHaveBeenCalledTimes(1))
  })

  /**
   * 🔴 Đường này **không đi qua** `getFileDiff` — lệnh diff của **commit**.
   *
   * Phát hiện của plan này: `DiffViewer` gọi `getFileDiff(repoId, commitId, path)`,
   * tức diff của một commit **lịch sử**. Gắn nút "đưa khối vào vùng chờ" lên đó là
   * vô nghĩa — không có gì để stage từ một commit đã có. Staging theo khối cần
   * `getWorktreeDiff`, và trước plan này **không chỗ nào trong `src/` gọi nó**.
   *
   * Test này ghim rằng bảng khối đọc **đúng** nguồn.
   */
  it('nạp diff bằng getWorktreeDiff, KHÔNG bằng getFileDiff (diff của commit)', async () => {
    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    await waitFor(() => expect(worktreeDiffMock).toHaveBeenCalledTimes(1))

    expect(worktreeDiffMock.mock.calls[0]).toEqual([REPO, PATH, false])
    expect(vi.mocked(ipc.getFileDiff)).toHaveBeenCalledTimes(0)
  })

  /**
   * `staged` đi thẳng vào `getWorktreeDiff` — `true` → `git diff --cached`.
   *
   * Không có test này, một cài đặt hardcode `false` hiện diff **chưa stage** trong
   * khi người dùng đang xem nhóm **đã stage**, và nút "Lấy khối khỏi vùng chờ" sẽ
   * thao tác trên những khối không có ở đó.
   */
  it('staged=true truyền xuống getWorktreeDiff, không bị bỏ đi', async () => {
    render(<HunkTable repoId={REPO} path={PATH} staged untracked={false} />)
    await waitFor(() => expect(worktreeDiffMock).toHaveBeenCalledTimes(1))
    expect(worktreeDiffMock.mock.calls[0]![2]).toBe(true)
  })
})

describe('HunkTable — chỉ số khối', () => {
  /**
   * 🔴 Đột biến **M29**: `HunkBar` nhận `0` thay vì chỉ số thật.
   *
   * Bấm khối **thứ hai** và **thứ ba**, rồi khẳng định `hunkIndex` là `1` và `2`.
   * Chỉ bấm khối đầu không phân biệt được với hardcode `0` — đúng lớp lỗi 4.2, và
   * đúng đột biến M23 mà wave 4 đã đo đỏ ở tầng `HunkBar`.
   */
  it('🔴 M29: mỗi khối truyền chỉ số THẬT của nó xuống stage_hunk', async () => {
    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    await waitFor(() => expect(screen.getAllByTestId('hunk-nut-vung-cho')).toHaveLength(3))

    const nut = screen.getAllByTestId('hunk-nut-vung-cho')

    fireEvent.click(nut[1]!)
    await waitFor(() => expect(stageHunkMock).toHaveBeenCalledTimes(1))
    expect(stageHunkMock.mock.calls[0]![2]).toBe(1)

    fireEvent.click(nut[2]!)
    await waitFor(() => expect(stageHunkMock).toHaveBeenCalledTimes(2))
    expect(stageHunkMock.mock.calls[1]![2]).toBe(2)
  })

  /**
   * Bảng hiện `heading` và khoảng dòng của **từng** khối.
   *
   * Đây là thứ làm bảng **dùng được** thay vì chỉ là ba nút giống hệt nhau: người
   * dùng phải biết mình đang stage khối nào. Một bảng ba nút không nhãn thoả mãn mọi
   * test trên mà vô dụng trên màn hình — và tiêu chí thành công 1 hỏi đúng câu
   * *"người dùng có chọn đúng khối mình muốn không"*.
   */
  it('mỗi hàng hiện heading và khoảng dòng của đúng khối đó', async () => {
    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    const hang = await screen.findAllByTestId('hunk-table-row')

    expect(hang[0]!.textContent).toContain('fn mot')
    expect(hang[1]!.textContent).toContain('fn hai')
    expect(hang[2]!.textContent).toContain('fn ba')

    // Khoảng dòng: khối 2 bắt đầu ở dòng 40 phía mới.
    expect(hang[1]!.textContent).toContain('40')

    // Phép kiểm **phủ định** — mỗi hàng chỉ mang heading CỦA NÓ. Một cài đặt render
    // cả ba heading vào mỗi hàng qua được ba khẳng định dương ở trên.
    expect(hang[0]!.textContent).not.toContain('fn hai')
    expect(hang[2]!.textContent).not.toContain('fn mot')
  })
})

describe('HunkTable — bốn dạng không phải text', () => {
  /**
   * Tệp nhị phân **không có khối** để stage. Bảng phải nói ra, không hiện bảng rỗng.
   *
   * R7 của `CONTEXT.md`: *"tệp nhị phân lọt vào đường text — phát hiện sớm, từ chối
   * rõ ràng, KHÔNG đoán."*
   */
  it('tệp nhị phân: nói rõ không staging theo khối được, KHÔNG hiện nút khối nào', async () => {
    worktreeDiffMock.mockResolvedValue({
      path: PATH,
      oldPath: null,
      status: 'M',
      kind: { kind: 'binary', oldSize: 100, newSize: 200 },
    })

    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)

    const bao = await screen.findByTestId('hunk-table-khong-text')
    expect(bao.textContent).toContain('nhị phân')
    expect(screen.queryAllByTestId('hunk-nut-vung-cho')).toHaveLength(0)
  })

  it('diff text KHÔNG có khối nào: nói rõ, KHÔNG hiện bảng trống', async () => {
    worktreeDiffMock.mockResolvedValue({
      path: PATH,
      oldPath: null,
      status: 'M',
      kind: { kind: 'text', hunks: [], truncated: false, contextOnly: false },
    })

    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    expect(await screen.findByTestId('hunk-table-rong')).toBeTruthy()
    expect(screen.queryAllByTestId('hunk-table-row')).toHaveLength(0)
  })

  /**
   * 🔴 `truncated` **phải** hiện ra.
   *
   * Cờ này nghĩa là bản vá **mất nội dung** — người dùng không thấy hết khối. Trên
   * đường staging theo khối điều đó nguy hiểm hơn hẳn trên đường chỉ-xem: họ sẽ tin
   * mình đã stage mọi thứ cần stage. `DiffToolbar` của Phase 3 đã có cổng ghim cho
   * đúng cờ này; đường thứ hai không được im lặng bỏ qua nó.
   */
  it('truncated=true: cảnh báo hiện ra cùng với các khối', async () => {
    worktreeDiffMock.mockResolvedValue({
      path: PATH,
      oldPath: null,
      status: 'M',
      kind: {
        kind: 'text',
        hunks: [khoi('fn mot', 10, 10)],
        truncated: true,
        contextOnly: false,
      },
    })

    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)

    const bang = await screen.findByTestId('hunk-table-truncated')
    expect(bang.textContent).toContain('một phần')
    // Cảnh báo hiện **cùng với** khối, không thay chúng.
    expect(screen.getAllByTestId('hunk-table-row')).toHaveLength(1)
  })
})

describe('HunkTable — lỗi và làm mới', () => {
  it('getWorktreeDiff ném lỗi: hiện băng role="alert", KHÔNG sập', async () => {
    worktreeDiffMock.mockRejectedValue(new Error('git diff hỏng'))

    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    const bang = await screen.findByRole('alert')
    expect(bang.textContent).toContain('git diff hỏng')
  })

  it('KHÔNG lỗi → KHÔNG có phần tử role="alert" nào', async () => {
    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    await screen.findAllByTestId('hunk-table-row')
    expect(screen.queryByRole('alert')).toBeNull()
  })

  /**
   * 🔴 `onLamMoi` của `HunkBar` phải **thật sự nạp lại diff** — WORK-05.
   *
   * Wave 4 thêm prop này và ghi rõ nó **chưa được truyền ở đâu cả**. Đây là chỗ nó
   * được truyền, và test này là thứ chứng minh nó không phải một prop trang trí:
   * bấm "Làm mới" trên băng phải sinh một lời gọi `getWorktreeDiff` **thứ hai**.
   *
   * Không có test này, một `onLamMoi={() => {}}` qua được mọi test khác, và nút
   * "Làm mới" lại nói dối đúng như bản đầu của wave 4 — băng biến mất, diff vẫn cũ.
   */
  it('bấm "Làm mới" trên băng file_changed nạp lại diff THẬT (lời gọi thứ hai)', async () => {
    render(<HunkTable repoId={REPO} path={PATH} staged={false} untracked={false} />)
    await waitFor(() => expect(worktreeDiffMock).toHaveBeenCalledTimes(1))

    useHunkStore.setState({
      canLamMoi: { path: PATH, message: 'tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới' },
    })

    const nut = await screen.findAllByTestId('hunk-nut-lam-moi')
    fireEvent.click(nut[0]!)

    await waitFor(() => expect(worktreeDiffMock).toHaveBeenCalledTimes(2))
  })
})
