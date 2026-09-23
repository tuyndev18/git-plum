/**
 * Test cho `WorktreePane` — **đường tới** staging theo khối, và cổng chống vòng khoá.
 *
 * # 🔴 Vì sao tệp này tồn tại tách khỏi `HunkTable.test.tsx`
 *
 * `HunkTable.test.tsx` trả lời *"bảng khối **có thể** chạy không cần Phase 3/4"*.
 * Tệp này trả lời câu **khác** và câu khó hơn: *"người dùng **có** một đường tới nó"*.
 *
 * Wave 5 của Phase 4 tìm được một vòng khoá chết **bốn điều kiện** mà không test cũ
 * nào thấy, và mọi component trong vòng đó **đều** có test xanh:
 *
 * 1. `ChangeList` là chỗ **duy nhất** gọi `refresh`;
 * 2. nó chỉ mount khi vùng soạn commit đã mở;
 * 3. đường **duy nhất** mở vùng soạn là bấm hàng WIP;
 * 4. hàng WIP chỉ hiện khi `statusStore` đã có dữ liệu
 *    → hàng WIP **không bao giờ** hiện.
 *
 * Mỗi mắt xích đúng; cái vòng thì sai. Không test component nào bắt được điều đó, vì
 * lỗi nằm ở **quan hệ giữa** các mắt xích chứ không ở mắt xích nào.
 *
 * Nên tệp này kiểm **ba điều kiện của vòng** một cách tường minh:
 *
 * - `WorktreePane` mount mà **không** cần bấm gì trước;
 * - nó **không** chờ một dữ liệu mà chính nó là chỗ duy nhất nạp;
 * - đường tới nó **không** đi qua `ChangeList` / `CommitBox` / hàng WIP.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { WorktreePane } from '@/components/worktree/WorktreePane'
import { ipc, type FileDiff, type RepoStatus, type StatusEntry } from '@/lib/ipc'
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

const REPO = 'repo-1'

function muc(path: string, xy: string, group: StatusEntry['group']): StatusEntry {
  return { path, oldPath: null, xy, group, hasInvalidUtf8: false }
}

function status(entries: StatusEntry[]): RepoStatus {
  return {
    branch: { head: 'main', oid: 'c1', upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

function diffMotKhoi(): FileDiff {
  return {
    path: 'src/a.txt',
    oldPath: null,
    status: 'M',
    kind: {
      kind: 'text',
      hunks: [
        {
          oldStart: 10,
          oldCount: 1,
          newStart: 10,
          newCount: 1,
          heading: 'fn mot',
          lines: [
            {
              kind: 'added',
              content: 'moi',
              oldLine: null,
              newLine: 10,
              noNewlineAtEof: false,
              spans: [],
            },
          ],
        },
      ],
      truncated: false,
      contextOnly: false,
    },
  } as unknown as FileDiff
}

beforeEach(() => {
  vi.clearAllMocks()
  useStatusStore.setState({ byRepo: {} })
  useHunkStore.setState({ dangChon: null, canLamMoi: null })
  vi.mocked(ipc.getStatus).mockResolvedValue(
    status([muc('src/a.txt', '.M', 'unstaged'), muc('src/b.txt', '??', 'untracked')]),
  )
  vi.mocked(ipc.getWorktreeDiff).mockResolvedValue(diffMotKhoi())
  vi.mocked(ipc.listTrash).mockResolvedValue([])
  vi.mocked(ipc.stageHunk).mockResolvedValue(status([]))
})

describe('WorktreePane — ba điều kiện của vòng khoá chết', () => {
  /**
   * Điều kiện 1 — **cái gì mount nó?**
   *
   * Trả lời phải là *"chỉ cần có repo"*, không phải *"phải bấm X trước"*. Test render
   * với `statusStore` **rỗng** và không bấm gì: danh sách tệp phải xuất hiện.
   *
   * Nếu component này đòi một cú bấm mở nó ra, thì cú bấm đó là điều kiện thứ hai của
   * một vòng tiềm năng — đúng chỗ Phase 4 sai.
   */
  it('🔴 vòng-1: mount và hiện tệp mà KHÔNG cần bấm gì trước, statusStore rỗng lúc bắt đầu', async () => {
    expect(useStatusStore.getState().byRepo).toEqual({})

    render(<WorktreePane repoId={REPO} />)

    // Không `fireEvent` nào ở trên. Danh sách tệp tự có.
    await waitFor(() => expect(screen.getAllByTestId('worktree-file')).toHaveLength(2))
  })

  /**
   * Điều kiện 2 — **nó có chờ một thứ mà CHÍNH NÓ là chỗ duy nhất tạo ra không?**
   *
   * Đây là hình dạng chính xác của vòng Phase 4: `ChangeList` là chỗ duy nhất gọi
   * `refresh`, **và** nó chỉ mount khi dữ liệu của `refresh` đã có.
   *
   * `WorktreePane` **tự gọi** `getStatus` lúc mount, và render danh sách **từ kết quả
   * đó**. Không có trạng thái "chưa nạp" nào chặn việc nó tự nạp. Test khẳng định
   * `getStatus` được gọi trên một store rỗng — tức lời gọi **không** bị một điều kiện
   * "đã có dữ liệu" chặn lại.
   */
  it('🔴 vòng-2: tự gọi getStatus lúc mount, KHÔNG chờ statusStore có sẵn dữ liệu', async () => {
    expect(useStatusStore.getState().byRepo[REPO]).toBeUndefined()

    render(<WorktreePane repoId={REPO} />)

    await waitFor(() => expect(vi.mocked(ipc.getStatus)).toHaveBeenCalled())
    expect(vi.mocked(ipc.getStatus).mock.calls[0]![0]).toBe(REPO)
  })

  /**
   * Điều kiện 3 — **có đường nào tới nó KHÔNG đi qua một component Phase 4 không?**
   *
   * Đây là câu wave 4 trả lời "❌ chưa". Nó được đóng bằng **hai** khẳng định, và
   * cần cả hai:
   *
   * a) `WorktreePane` render đầy đủ mà **không** có `ChangeList` / `CommitBox` /
   *    hàng WIP nào trên màn hình — kiểm bằng `data-testid` của chúng;
   * b) từ lúc mount tới lúc gọi `stage_hunk` **không** có cú bấm nào chạm một
   *    component Phase 4.
   *
   * (a) một mình chưa đủ: một cài đặt *nhập* `ChangeList` rồi ẩn nó bằng CSS vẫn qua.
   * (b) là thứ đo **đường đi thật**.
   */
  it('🔴 vòng-3: chọn tệp → chọn khối → stage_hunk, KHÔNG có ChangeList/CommitBox/hàng WIP nào trên đường', async () => {
    render(<WorktreePane repoId={REPO} />)

    await waitFor(() => expect(screen.getAllByTestId('worktree-file')).toHaveLength(2))

    // (a) Không component Phase 4 nào có mặt.
    expect(screen.queryByTestId('change-list')).toBeNull()
    expect(screen.queryByTestId('commit-box')).toBeNull()
    expect(screen.queryByTestId('wip-row')).toBeNull()

    // (b) Đường đi thật: bấm tệp → bảng khối hiện → bấm nút khối → IPC.
    fireEvent.click(screen.getAllByTestId('worktree-file')[0]!)

    await waitFor(() => expect(screen.getAllByTestId('hunk-table-row')).toHaveLength(1))
    expect(screen.queryByTestId('change-list')).toBeNull()

    fireEvent.click(screen.getAllByTestId('hunk-nut-vung-cho')[0]!)
    await waitFor(() => expect(vi.mocked(ipc.stageHunk)).toHaveBeenCalledTimes(1))

    expect(vi.mocked(ipc.stageHunk).mock.calls[0]![1]).toBe('src/a.txt')
  })
})

describe('WorktreePane — động từ huỷ theo nhóm', () => {
  /**
   * 🔴 Tệp **chưa theo dõi** → động từ "Xoá". ROADMAP, không thương lượng.
   *
   * `HunkBar` đã có cổng hai chiều cho việc này (M21 + M22 của wave 4), nhưng cổng đó
   * kiểm `HunkBar` **khi được truyền** `untracked` đúng. Chỗ **quyết định** giá trị
   * đó là đây, và một `untracked={false}` cứng ở chỗ gọi làm mọi cổng của wave 4 vô
   * dụng mà không test nào của wave 4 thấy — đúng lỗi #9 nhìn từ một tầng lên.
   */
  it('chọn tệp CHƯA THEO DÕI → nút huỷ dùng động từ "Xoá", không "Huỷ bỏ"', async () => {
    render(<WorktreePane repoId={REPO} />)
    await waitFor(() => expect(screen.getAllByTestId('worktree-file')).toHaveLength(2))

    // Tệp thứ hai của fixture là `src/b.txt`, nhóm `untracked`.
    fireEvent.click(screen.getAllByTestId('worktree-file')[1]!)
    await waitFor(() => expect(screen.getAllByTestId('hunk-nut-huy')).toHaveLength(1))

    const nut = screen.getAllByTestId('hunk-nut-huy')[0]!
    expect(nut.textContent).toContain('Xoá')
    expect(nut.textContent).not.toContain('Huỷ bỏ')
  })

  it('chọn tệp ĐÃ THEO DÕI → nút huỷ dùng động từ "Huỷ bỏ", không "Xoá"', async () => {
    render(<WorktreePane repoId={REPO} />)
    await waitFor(() => expect(screen.getAllByTestId('worktree-file')).toHaveLength(2))

    fireEvent.click(screen.getAllByTestId('worktree-file')[0]!)
    await waitFor(() => expect(screen.getAllByTestId('hunk-nut-huy')).toHaveLength(1))

    const nut = screen.getAllByTestId('hunk-nut-huy')[0]!
    expect(nut.textContent).toContain('Huỷ bỏ')
    expect(nut.textContent).not.toContain('Xoá')
  })

  /**
   * `staged` truyền xuống theo **nhóm của tệp**, không cứng `false`.
   *
   * Không có cổng này, chọn một tệp ở nhóm "đã stage" nạp diff **chưa stage** — tức
   * bảng hiện những khối không có trong vùng chờ, và nút "Lấy khối khỏi vùng chờ"
   * thao tác lên chỉ số của một tập khối khác. Một lỗi **ghi**, không phải hiển thị.
   */
  it('tệp ở nhóm đã stage → getWorktreeDiff nhận staged=true', async () => {
    vi.mocked(ipc.getStatus).mockResolvedValue(status([muc('src/c.txt', 'M.', 'staged')]))

    render(<WorktreePane repoId={REPO} />)
    await waitFor(() => expect(screen.getAllByTestId('worktree-file')).toHaveLength(1))

    fireEvent.click(screen.getAllByTestId('worktree-file')[0]!)
    await waitFor(() => expect(vi.mocked(ipc.getWorktreeDiff)).toHaveBeenCalledTimes(1))

    expect(vi.mocked(ipc.getWorktreeDiff).mock.calls[0]).toEqual([REPO, 'src/c.txt', true])
  })
})

describe('WorktreePane — danh sách Vừa huỷ gần đây tìm thấy được', () => {
  /**
   * Tiêu chí thành công 4 hỏi *"bạn **tìm thấy nó** mà không phải hỏi nó ở đâu?"*.
   *
   * Phần "tìm thấy được" là **mắt người** và nằm ở checkpoint — happy-dom không trả
   * lời được. Phần máy trả lời được là: nó **có mặt** trên cùng màn hình, không nằm
   * sau một menu.
   */
  it('TrashList có mặt ngay trên pane, không nằm sau một cú bấm nào', async () => {
    render(<WorktreePane repoId={REPO} />)
    expect(await screen.findByTestId('trash-list')).toBeTruthy()
  })
})
