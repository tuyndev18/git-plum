/**
 * Test cho `HunkBar` — WORK-03, WORK-05, WORK-06.
 *
 * # 🔴 Cổng chịu lực của tệp này: `hunkbar_render_mot_minh_khong_co_phase4`
 *
 * Đó là đột biến **M25**, đột biến quan trọng nhất của plan 05-04. Nó ghim ràng buộc
 * của `CONTEXT.md` mục 0 hệ quả 2 — thứ duy nhất ngăn staging theo khối chết theo một
 * tầng giao diện Phase 4 mà **không requirement nào có bằng chứng từ mắt người**, và mà
 * wave 5 của nó đã tìm được một **vòng khoá chết bốn điều kiện** không test cũ nào thấy.
 *
 * Nếu M25 cho 0 test đỏ thì `HunkBar` đang lệ thuộc Phase 4 và plan chưa xong.
 *
 * # ⚠️ Không test nào ở đây kiểm được BỐ CỤC
 *
 * happy-dom không tính CSS layout và không có cuộn thật (`CONTEXT.md` 4.5). "Ba nút
 * không đè lên nhau", "thanh không tràn khỏi khối", "băng `file_changed` không che mất
 * nội dung diff" là **có mã, chưa kiểm** cho tới khi có người xem trên Chromium thật
 * (checkpoint 05-05). Đó chính là cách cả **năm** lỗi hiển thị Phase 3 và lỗi
 * `marginTop`→`paddingTop` của Phase 4 sống sót qua 666 test.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'

import { HunkBar } from '@/components/worktree/HunkBar'
import { ipc, type GitErrorPayload, type RepoStatus, type StatusEntry } from '@/lib/ipc'
import { useHunkStore } from '@/stores/hunkStore'
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
      stageHunk: vi.fn(),
      unstageHunk: vi.fn(),
      discardHunk: vi.fn(),
      discardFiles: vi.fn(),
      listTrash: vi.fn(),
      restoreTrash: vi.fn(),
    },
  }
})

const stageHunkMock = vi.mocked(ipc.stageHunk)
const unstageHunkMock = vi.mocked(ipc.unstageHunk)
const discardHunkMock = vi.mocked(ipc.discardHunk)

const REPO = 'repo-1'
const PATH = 'src/a.txt'
const HASH = 'a'.repeat(40)

/**
 * 🔴 `index` khác 0, có chủ ý.
 *
 * `index: 0` **không phân biệt được** với một cài đặt hardcode `0` — đúng lớp lỗi
 * "hình dạng đúng, dữ liệu vô hại" của `CONTEXT.md` 4.2, thứ đã xảy ra ba lần qua hai
 * phase. Đột biến M23 ghim điều này.
 */
const INDEX = 1

function muc(path: string, xy: string, group: StatusEntry['group']): StatusEntry {
  return { path, oldPath: null, xy, group, hasInvalidUtf8: false }
}

function status(entries: StatusEntry[]): RepoStatus {
  return {
    branch: { head: 'main', oid: 'b'.repeat(40), upstream: null, ahead: null, behind: null },
    entries,
    hasConflicts: false,
  }
}

const SAU = status([muc(PATH, 'MM', 'staged')])

function loiFileChanged(): GitErrorPayload {
  return {
    code: 'file_changed',
    message: 'Tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới',
    command: null,
  }
}

beforeEach(() => {
  stageHunkMock.mockReset()
  unstageHunkMock.mockReset()
  discardHunkMock.mockReset()
  stageHunkMock.mockResolvedValue(SAU)
  unstageHunkMock.mockResolvedValue(SAU)
  discardHunkMock.mockResolvedValue(SAU)

  useStatusStore.setState({ byRepo: {} })
  useHunkStore.setState({ dangChon: null, canLamMoi: null })
})

/** Props tối thiểu, mặc định "chưa stage, đã theo dõi". */
function props(ghiDe: Partial<React.ComponentProps<typeof HunkBar>> = {}) {
  return {
    repoId: REPO,
    path: PATH,
    index: INDEX,
    blobHash: HASH,
    staged: false,
    untracked: false,
    ...ghiDe,
  }
}

// --- Test 1 — nhãn stage/unstage, CẢ HAI chiều ----------------------------

describe('nhãn nút', () => {
  it('staged=false → "Đưa khối vào vùng chờ"', () => {
    render(<HunkBar {...props({ staged: false })} />)

    expect(screen.getByRole('button', { name: 'Đưa khối vào vùng chờ' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Lấy khối khỏi vùng chờ' })).toBeNull()
  })

  it('staged=true → "Lấy khối khỏi vùng chờ"', () => {
    render(<HunkBar {...props({ staged: true })} />)

    expect(screen.getByRole('button', { name: 'Lấy khối khỏi vùng chờ' })).toBeTruthy()
    expect(screen.queryByRole('button', { name: 'Đưa khối vào vùng chờ' })).toBeNull()
  })

  it('có đúng hai nút: một nút vùng chờ và một nút huỷ', () => {
    render(<HunkBar {...props()} />)

    expect(screen.getAllByRole('button').length).toBe(2)
  })
})

// --- Test 2 — 🔴 ROADMAP không thương lượng: động từ huỷ ------------------

describe('🔴 động từ huỷ: "Xoá" cho tệp chưa theo dõi (ROADMAP, không thương lượng)', () => {
  /**
   * "Huỷ bỏ" hàm ý *quay lại phiên bản đã lưu* — có một bản gốc ở đâu đó. Với tệp
   * **chưa theo dõi** thì **không có** bản gốc nào. Dùng sai động từ là nói sai mức độ
   * nghiêm trọng đúng lúc người dùng đang quyết định có bấm hay không (T-05-14).
   *
   * Đột biến M21 (luôn "Huỷ bỏ") và M22 (luôn "Xoá") **đều** phải đỏ ở nhóm này. Nếu
   * chỉ một trong hai đỏ thì test chỉ kiểm một nhánh — lỗi #4 của `CONTEXT.md` 4.1.
   */
  it('untracked=true → nút huỷ ghi "Xoá"', () => {
    render(<HunkBar {...props({ untracked: true })} />)

    const nut = screen.getByTestId('hunk-nut-huy')
    expect(nut.textContent).toContain('Xoá')
  })

  /**
   * 🔴 Nhánh `untracked` **không** được chứa chuỗi `"Huỷ bỏ"`.
   *
   * Một nhãn `"Xoá / Huỷ bỏ"` — hay một cài đặt hiện cả hai để "cho an toàn" — qua
   * được phép kiểm `toContain('Xoá')` ở trên. Phép kiểm phủ định này là thứ phân biệt
   * "chọn đúng động từ" với "hiện mọi động từ".
   */
  it('untracked=true → nút huỷ KHÔNG chứa "Huỷ bỏ"', () => {
    render(<HunkBar {...props({ untracked: true })} />)

    const nut = screen.getByTestId('hunk-nut-huy')
    expect(nut.textContent).not.toContain('Huỷ bỏ')
  })

  it('untracked=false → nút huỷ ghi "Huỷ bỏ" và KHÔNG chứa "Xoá"', () => {
    render(<HunkBar {...props({ untracked: false })} />)

    const nut = screen.getByTestId('hunk-nut-huy')
    expect(nut.textContent).toContain('Huỷ bỏ')
    expect(nut.textContent).not.toContain('Xoá')
  })
})

// --- Test 3 — bấm nút gọi đúng hành động với đúng index -------------------

describe('bấm nút gọi đúng hành động', () => {
  it('bấm "Đưa khối vào vùng chờ" gọi stageHunk với đúng index (KHÁC 0)', async () => {
    render(<HunkBar {...props({ staged: false })} />)

    screen.getByRole('button', { name: 'Đưa khối vào vùng chờ' }).click()

    await waitFor(() => expect(stageHunkMock).toHaveBeenCalledTimes(1))
    expect(stageHunkMock).toHaveBeenCalledWith(REPO, PATH, INDEX, HASH)
    // 🔴 Khẳng định rời cho chính `index`: một cài đặt hardcode `0` phải đỏ ở đây.
    expect(stageHunkMock.mock.calls[0]?.[2]).toBe(INDEX)
    expect(unstageHunkMock).toHaveBeenCalledTimes(0)
    expect(discardHunkMock).toHaveBeenCalledTimes(0)
  })

  it('bấm "Lấy khối khỏi vùng chờ" gọi unstageHunk với đúng index', async () => {
    render(<HunkBar {...props({ staged: true })} />)

    screen.getByRole('button', { name: 'Lấy khối khỏi vùng chờ' }).click()

    await waitFor(() => expect(unstageHunkMock).toHaveBeenCalledTimes(1))
    expect(unstageHunkMock).toHaveBeenCalledWith(REPO, PATH, INDEX, HASH)
    expect(unstageHunkMock.mock.calls[0]?.[2]).toBe(INDEX)
    expect(stageHunkMock).toHaveBeenCalledTimes(0)
  })

  it('bấm nút huỷ gọi discardHunk với đúng index', async () => {
    render(<HunkBar {...props()} />)

    screen.getByTestId('hunk-nut-huy').click()

    await waitFor(() => expect(discardHunkMock).toHaveBeenCalledTimes(1))
    expect(discardHunkMock).toHaveBeenCalledWith(REPO, PATH, INDEX, HASH)
    expect(discardHunkMock.mock.calls[0]?.[2]).toBe(INDEX)
  })
})

// --- Test 4 — 🔴 RÀNG BUỘC CHỐNG-PHASE-4 (đột biến M25) -------------------

describe('🔴 M25: HunkBar dùng được KHÔNG CẦN thành phần nào của Phase 4', () => {
  /**
   * **Test quan trọng nhất của plan 05-04.**
   *
   * Khuôn chép từ `nut_stage_van_bam_duoc_khi_diff_hong` của 04-02 — cùng lớp ràng
   * buộc, cùng cách ghim.
   *
   * `render(<HunkBar … />)` ở đây **không** bọc trong provider nào, **không** có
   * `ChangeList`, **không** có `DiffViewer`, và `statusStore` để **rỗng** (`byRepo: {}`
   * ở `beforeEach`) — tức đúng trạng thái "Phase 4 chưa bao giờ chạy". Nút vẫn phải
   * hiện và vẫn phải gọi IPC.
   *
   * Nếu ai đó thêm một `useXStore()` của Phase 4 vào `HunkBar` và `return null` khi
   * chưa có dữ liệu — đúng hình dạng vòng khoá chết mà wave 5 Phase 4 tìm được — test
   * này đỏ ngay.
   */
  it('render MỘT MÌNH, statusStore rỗng → nút vẫn hiện và vẫn gọi IPC', async () => {
    // Tiền đề: không có gì của Phase 4 trong store. Khẳng định nó, không giả định.
    expect(useStatusStore.getState().byRepo).toEqual({})

    const { container } = render(<HunkBar {...props()} />)

    // Component thật sự render ra cái gì đó — `return null` phải đỏ ở đây, không phải
    // im lặng qua được vì `getAllByRole` trên cây rỗng cũng "không lỗi" nếu ta quên.
    expect(container.querySelector('[data-testid="hunk-bar"]')).not.toBeNull()

    const nut = screen.getByRole('button', { name: 'Đưa khối vào vùng chờ' })
    expect((nut as HTMLButtonElement).disabled).toBe(false)

    nut.click()

    await waitFor(() => expect(stageHunkMock).toHaveBeenCalledTimes(1))
    expect(stageHunkMock).toHaveBeenCalledWith(REPO, PATH, INDEX, HASH)
  })

  /**
   * Không nút nào bị vô hiệu vì Phase 4 chưa nạp.
   *
   * Khẳng định trên **mọi** nút, không chỉ nút đang quan tâm: một cài đặt vô hiệu
   * "nút huỷ khi chưa biết tệp có theo dõi không" vẫn để nút stage bật, nên một test
   * chỉ nhìn một nút có thể trượt. Cùng lập luận với 04-02.
   */
  it('không nút nào bị vô hiệu khi chưa có dữ liệu Phase 4', () => {
    render(<HunkBar {...props()} />)

    const nut = screen.getAllByRole('button')
    expect(nut.length).toBe(2)
    for (const n of nut) {
      expect((n as HTMLButtonElement).disabled).toBe(false)
    }
  })

  /**
   * `HunkBar` **không** tự gọi `getStatus` hay `getWorktreeDiff`.
   *
   * Đây là cách ràng buộc được cài ở mức **cấu trúc**, không chỉ ở mức hành vi: nếu
   * component này tự nạp trạng thái thư mục làm việc thì nó có một phụ thuộc thời-điểm
   * lên chính thứ Phase 4 sở hữu, và một lỗi ở đó gỡ luôn cả nút.
   */
  it('HunkBar không tự gọi getStatus hay getWorktreeDiff', () => {
    render(<HunkBar {...props()} />)

    expect(vi.mocked(ipc.getStatus)).toHaveBeenCalledTimes(0)
    expect(vi.mocked(ipc.getWorktreeDiff)).toHaveBeenCalledTimes(0)
  })
})

// --- Test 5 — WORK-05: băng "hãy làm mới" ---------------------------------

describe('🔴 WORK-05: thông báo "hãy làm mới"', () => {
  /**
   * 🔴 Neo truy vấn vào `data-testid="hunk-can-lam-moi"` — phần tử **chỉ** có ở trạng
   * thái này.
   *
   * Lỗi #6 của `CONTEXT.md` 4.1: một cổng neo vào vỏ panel có mặt ở **mọi** trạng thái
   * kể cả lúc đang nạp; nó đỏ 1 trên 5 lần chạy và sống sót cả một wave. Vỏ `hunk-bar`
   * **không** dùng được làm mỏ neo ở đây vì nó có mặt ở mọi trạng thái.
   */
  it('canLamMoi khác null → hiện băng chứa nguyên văn "hãy làm mới"', async () => {
    stageHunkMock.mockRejectedValueOnce(loiFileChanged())

    render(<HunkBar {...props()} />)

    // Tiền đề: băng CHƯA có trước khi lỗi xảy ra. Không có khẳng định này thì một
    // băng hiện ở mọi trạng thái cũng qua được test.
    expect(screen.queryByTestId('hunk-can-lam-moi')).toBeNull()

    screen.getByRole('button', { name: 'Đưa khối vào vùng chờ' }).click()

    const bang = await screen.findByTestId('hunk-can-lam-moi')
    expect(bang.textContent).toContain('hãy làm mới')
  })

  it('băng có một nút làm mới bấm được, và bấm nó gỡ băng đi', async () => {
    stageHunkMock.mockRejectedValueOnce(loiFileChanged())

    render(<HunkBar {...props()} />)
    screen.getByRole('button', { name: 'Đưa khối vào vùng chờ' }).click()

    await screen.findByTestId('hunk-can-lam-moi')

    const nutLamMoi = screen.getByTestId('hunk-nut-lam-moi')
    expect((nutLamMoi as HTMLButtonElement).disabled).toBe(false)

    nutLamMoi.click()

    await waitFor(() => expect(screen.queryByTestId('hunk-can-lam-moi')).toBeNull())
  })

  /**
   * Băng **chỉ** hiện cho khối của **đúng tệp** đang gặp lỗi.
   *
   * `canLamMoi` là một trường ở cấp store, nên một cài đặt ngây thơ sẽ hiện băng trên
   * **mọi** `HunkBar` đang mount, kể cả những khối của tệp khác — và người dùng sẽ
   * thấy "tệp đã đổi" trên một tệp không đổi gì.
   */
  it('băng KHÔNG hiện trên HunkBar của tệp khác', async () => {
    useHunkStore.setState({
      canLamMoi: { path: 'src/khac.txt', message: 'Tệp đã đổi …, hãy làm mới' },
    })

    render(<HunkBar {...props({ path: PATH })} />)

    expect(screen.queryByTestId('hunk-can-lam-moi')).toBeNull()
  })

  /**
   * Lỗi **khác** `file_changed` → **không** có băng.
   *
   * Cặp với test trên: không có nó, một cài đặt hiện băng cho mọi lỗi vẫn xanh.
   */
  it('lỗi index_locked → KHÔNG hiện băng "hãy làm mới"', async () => {
    stageHunkMock.mockRejectedValueOnce({
      code: 'index_locked',
      message: 'Đang chờ một tiến trình git khác, thử lại sau',
      command: null,
    } satisfies GitErrorPayload)

    render(<HunkBar {...props()} />)
    screen.getByRole('button', { name: 'Đưa khối vào vùng chờ' }).click()

    await waitFor(() => expect(stageHunkMock).toHaveBeenCalledTimes(1))
    expect(screen.queryByTestId('hunk-can-lam-moi')).toBeNull()
  })
})
