/**
 * Test cho `TrashList` — "Vừa huỷ gần đây", WORK-07.
 *
 * # ⚠️ Không test nào ở đây kiểm được BỐ CỤC
 *
 * happy-dom không tính CSS layout và không có cuộn thật (`CONTEXT.md` 4.5). "Danh
 * sách không tràn khi có nhiều mục", "chữ không bị cắt" là **có mã, chưa kiểm** cho
 * tới khi có người xem trên Chromium thật (checkpoint 05-05, bước 8 và câu bố cục thứ
 * ba). Đó chính là cách cả **năm** lỗi hiển thị Phase 3 và lỗi `marginTop`→`paddingTop`
 * của Phase 4 sống sót qua 666 test.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'

import { TrashList } from '@/components/worktree/TrashList'
import { ipc, type MucThungRac, type RepoStatus } from '@/lib/ipc'
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

const listTrashMock = vi.mocked(ipc.listTrash)
const restoreTrashMock = vi.mocked(ipc.restoreTrash)

const REPO = 'repo-1'

function trangThaiRong(): RepoStatus {
  return {
    branch: { head: 'main', oid: null, upstream: null, ahead: null, behind: null },
    entries: [],
    hasConflicts: false,
  }
}

/**
 * Một mục thùng rác. `refName` mang **mốc thời gian mili giây** đúng như Rust dựng nó:
 * `refs/git-plum-trash/{epoch_ms}-{n}` (xem `commands/trash.rs`, `format!`).
 */
function muc(epochMs: number, n: number, laBlob = false): MucThungRac {
  return {
    refName: `refs/git-plum-trash/${epochMs}-${n}`,
    objectId: 'f'.repeat(40),
    laBlob,
    // 🔴 Ba trường này rỗng **đúng như backend trả về hôm nay** — `for-each-ref`
    // không giữ được chúng. Fixture điền sẵn `paths`/`nhan` sẽ là fixture nói dối
    // về dữ liệu thật, và giao diện dựng trên nó sẽ trống rỗng lúc chạy thật.
    paths: [],
    luc: 0,
    nhan: '',
  }
}

beforeEach(() => {
  vi.clearAllMocks()
  useStatusStore.setState({ byRepo: {} })
  listTrashMock.mockResolvedValue([])
  restoreTrashMock.mockResolvedValue(trangThaiRong())
})

describe('TrashList — danh sách rỗng', () => {
  /**
   * Test 1 — khuôn "nhóm rỗng KHÔNG hiện tiêu đề" của `ChangeList` (đột biến M8 của
   * 04-02). Một tiêu đề "Vừa huỷ gần đây" trên một danh sách rỗng là một dòng nhiễu
   * nói "không có gì" bằng hai dòng.
   */
  it('danh sách rỗng: hiện MỘT câu nói rõ chưa huỷ gì, KHÔNG hiện tiêu đề trống', async () => {
    listTrashMock.mockResolvedValue([])
    render(<TrashList repoId={REPO} />)

    const rong = await screen.findByTestId('trash-empty')
    expect(rong.textContent).toContain('Chưa huỷ')

    // Phép kiểm **phủ định** — đây là nửa mang tải của test.
    //
    // Chỉ khẳng định "có câu rỗng" thì một cài đặt hiện **cả** tiêu đề **lẫn** câu
    // rỗng vẫn qua. Khẳng định tiêu đề **vắng mặt** là thứ phân biệt được.
    expect(screen.queryByTestId('trash-heading')).toBeNull()
    expect(screen.queryAllByTestId('trash-row')).toHaveLength(0)
  })
})

describe('TrashList — khôi phục', () => {
  /**
   * Test 2 — 🔴 bấm mục **THỨ HAI** trong danh sách **BA** mục.
   *
   * Bấm mục đầu **không phân biệt được** với một cài đặt hardcode `muc[0].refName`
   * (đột biến M27). Đúng lớp lỗi "hình dạng đúng, dữ liệu vô hại" của `CONTEXT.md`
   * 4.2, thứ đã xảy ra ba lần qua hai phase.
   *
   * Ba mục chứ không hai: với hai mục, "mục thứ hai" cũng là "mục cuối", nên một cài
   * đặt hardcode `muc[muc.length - 1]` cũng qua được. Ba mục loại được cả hai.
   */
  it('bấm "Khôi phục" ở mục THỨ HAI gọi restore_trash với ĐÚNG refName của mục đó', async () => {
    const ds = [muc(3_000_000_000_000, 0), muc(2_000_000_000_000, 0), muc(1_000_000_000_000, 0)]
    listTrashMock.mockResolvedValue(ds)

    render(<TrashList repoId={REPO} />)

    await waitFor(() => expect(screen.getAllByTestId('trash-row')).toHaveLength(3))

    const nut = screen.getAllByTestId('trash-restore')
    fireEvent.click(nut[1]!)

    await waitFor(() => expect(restoreTrashMock).toHaveBeenCalledTimes(1))

    // Danh sách hiện **mới nhất trước** (Test 3), nên hàng thứ hai trên màn hình là
    // mục có mốc `2_000_000_000_000`. Khẳng định **chính xác** refName đó.
    expect(restoreTrashMock.mock.calls[0]![1]).toBe('refs/git-plum-trash/2000000000000-0')

    // Phép kiểm **phủ định**: KHÔNG phải mục đầu. Một cài đặt hardcode `[0]` sẽ gọi
    // đúng hàm, đúng số lần, và chỉ sai ở đối số này.
    expect(restoreTrashMock.mock.calls[0]![1]).not.toBe('refs/git-plum-trash/3000000000000-0')
  })

  /**
   * `restoreTrash` trả `RepoStatus` **mới** (ràng buộc 2.5). Giao diện phải ghi nó
   * **từ giá trị trả về** chứ không gọi `getStatus` sau đó — cùng khuôn `hunkStore`
   * và đột biến M2 của 04-02.
   */
  it('khôi phục ghi RepoStatus TỪ GIÁ TRỊ TRẢ VỀ, KHÔNG gọi getStatus', async () => {
    listTrashMock.mockResolvedValue([muc(1_000_000_000_000, 0)])
    render(<TrashList repoId={REPO} />)

    await waitFor(() => expect(screen.getAllByTestId('trash-row')).toHaveLength(1))
    fireEvent.click(screen.getAllByTestId('trash-restore')[0]!)

    await waitFor(() => expect(restoreTrashMock).toHaveBeenCalledTimes(1))
    expect(vi.mocked(ipc.getStatus)).toHaveBeenCalledTimes(0)
    await waitFor(() =>
      expect(useStatusStore.getState().byRepo[REPO]?.status).toBeDefined(),
    )
  })
})

describe('TrashList — thứ tự', () => {
  /**
   * Test 3 — 🔴 fixture **KHÔNG sắp sẵn**.
   *
   * Một fixture đã sắp sẵn **không phân biệt được** một cài đặt không sắp gì cả với
   * một cài đặt sắp đúng — lỗi #4 của `CONTEXT.md` 4.1. Nên ba mốc vào theo thứ tự
   * giữa/cũ nhất/mới nhất, và test đòi thứ tự ra là mới → cũ.
   *
   * Đột biến **M26** (bỏ phép sắp xếp) phải làm test này đỏ.
   */
  it('mục sắp MỚI NHẤT TRƯỚC, kể cả khi backend trả về không theo thứ tự', async () => {
    const CU = 1_000_000_000_000
    const GIUA = 2_000_000_000_000
    const MOI = 3_000_000_000_000

    // Thứ tự vào: GIỮA, CŨ, MỚI — không tăng, không giảm.
    listTrashMock.mockResolvedValue([muc(GIUA, 0), muc(CU, 0), muc(MOI, 0)])

    render(<TrashList repoId={REPO} />)
    await waitFor(() => expect(screen.getAllByTestId('trash-row')).toHaveLength(3))

    const ten = screen.getAllByTestId('trash-row').map((el) => el.getAttribute('data-ref'))
    expect(ten).toEqual([
      `refs/git-plum-trash/${MOI}-0`,
      `refs/git-plum-trash/${GIUA}-0`,
      `refs/git-plum-trash/${CU}-0`,
    ])
  })

  /**
   * 🔴 Thời gian đọc **từ `refName`**, không từ `luc`.
   *
   * `CONTEXT.md` / 05-03-SUMMARY ghi rõ backend trả `luc: 0` cho **mọi** mục vì
   * `for-each-ref` không giữ được nó. Một giao diện đọc `luc` sẽ hiện
   * **1970-01-01 cho mọi mục** — tức không có thông tin, và người dùng không phân
   * biệt được mục vừa huỷ với mục huỷ tuần trước.
   *
   * Mốc mili giây **nằm sẵn trong tên ref** (`{epoch_ms}-{n}`, xem `trash.rs`), nên
   * nó lấy lại được **không cần chỗ lưu phụ nào**. Test này ghim điều đó.
   */
  it('thời gian đọc TỪ refName, không từ trường luc (backend trả luc: 0)', async () => {
    // 2026-09-23T10:00:00Z. `luc` của fixture là **0** — đúng như backend thật.
    const MS = Date.UTC(2026, 8, 23, 10, 0, 0)
    listTrashMock.mockResolvedValue([muc(MS, 0)])

    render(<TrashList repoId={REPO} />)
    const hang = await screen.findByTestId('trash-row')

    const thoiGian = hang.querySelector('[data-testid="trash-time"]')
    expect(thoiGian).not.toBeNull()

    // Không khẳng định chuỗi định dạng (nó theo locale của máy chạy test —
    // `Intl.DateTimeFormat`, CLAUDE.md loại `chrono`). Khẳng định thứ **phân biệt
    // được**: nó KHÔNG phải năm 1970, tức nó không đọc `luc: 0`.
    expect(thoiGian!.textContent).not.toBe('')
    expect(thoiGian!.textContent).not.toContain('1970')
    expect(thoiGian!.getAttribute('data-ms')).toBe(String(MS))
  })
})

describe('TrashList — khoảng trống đã biết phải NÓI RA', () => {
  /**
   * 🔴 `paths` và `nhan` **rỗng** ở mọi mục, và đó không phải lỗi của giao diện.
   *
   * 05-03-SUMMARY + doc comment của `ipc.listTrash`: `for-each-ref` không giữ được
   * ba trường đó. Wave 4 ghi nó vào doc comment **cốt để wave này không dựng một
   * danh sách trống rồi mới hiểu tại sao**.
   *
   * Lựa chọn ở đây: **nói ra**. Một danh sách hiện ba dòng trắng trơn là một danh
   * sách người dùng không tin được — họ không biết mục nào chứa gì, nên họ không
   * dám bấm "Khôi phục". Thà nói thẳng "chưa có tên tệp" và cho họ mã object để đối
   * chiếu, còn hơn hiện một khoảng trắng và để họ đoán.
   *
   * Test này là **cổng** cho điều đó: nếu ai gỡ câu giải thích đi vì thấy nó xấu,
   * test đỏ và họ phải đọc lý do.
   */
  it('mục không có tên tệp: NÓI RA là chưa có, và hiện mã object để đối chiếu', async () => {
    listTrashMock.mockResolvedValue([muc(2_000_000_000_000, 0)])

    render(<TrashList repoId={REPO} />)
    const hang = await screen.findByTestId('trash-row')

    // Có một chỗ thay cho tên tệp, và nó nói ra rằng tên tệp chưa có.
    const nhan = hang.querySelector('[data-testid="trash-nhan"]')
    expect(nhan).not.toBeNull()
    expect(nhan!.textContent).toContain('chưa ghi lại tên tệp')

    // Và mã object hiện ra — thứ DUY NHẤT phân biệt hai mục với nhau hôm nay.
    expect(hang.textContent).toContain('f'.repeat(8))
  })

  /**
   * Khi backend **có** `nhan` (sau khi chỗ lưu phụ được thêm), giao diện phải hiện
   * nó thay vì câu giải thích. Không có test này, một cài đặt bỏ qua `nhan` hoàn
   * toàn — luôn hiện "chưa ghi lại tên tệp" — sẽ qua được test trên, và khoảng
   * trống sẽ không bao giờ đóng được dù backend đã sửa.
   */
  it('mục CÓ nhan: hiện nhan thay cho câu giải thích', async () => {
    const m = muc(2_000_000_000_000, 0)
    m.nhan = 'src/a.txt và 2 tệp nữa'
    m.paths = ['src/a.txt', 'src/b.txt', 'src/c.txt']
    listTrashMock.mockResolvedValue([m])

    render(<TrashList repoId={REPO} />)
    const hang = await screen.findByTestId('trash-row')

    const nhan = hang.querySelector('[data-testid="trash-nhan"]')!
    expect(nhan.textContent).toContain('src/a.txt và 2 tệp nữa')
    expect(nhan.textContent).not.toContain('chưa ghi lại tên tệp')
  })
})

describe('TrashList — lỗi', () => {
  it('listTrash ném lỗi: hiện băng role="alert", KHÔNG làm sập component', async () => {
    listTrashMock.mockRejectedValue(new Error('git for-each-ref hỏng'))

    render(<TrashList repoId={REPO} />)

    const bang = await screen.findByRole('alert')
    expect(bang.textContent).toContain('git for-each-ref hỏng')
  })

  /**
   * Phép kiểm **phủ định** cho `role="alert"` — khuôn M26b của wave 4.
   *
   * Không có nó, một cài đặt gắn `role="alert"` lên **vỏ** danh sách (tức có mặt ở
   * **mọi** trạng thái) qua được test dương ở trên mà vẫn sai hoàn toàn: trình đọc
   * màn hình sẽ đọc to danh sách mỗi lần nó render. Đúng lỗi #6.
   */
  it('KHÔNG lỗi → KHÔNG có phần tử role="alert" nào', async () => {
    listTrashMock.mockResolvedValue([muc(2_000_000_000_000, 0)])

    render(<TrashList repoId={REPO} />)
    await screen.findByTestId('trash-row')

    expect(screen.queryByRole('alert')).toBeNull()
  })
})
