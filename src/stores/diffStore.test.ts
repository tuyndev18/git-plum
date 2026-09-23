/**
 * `diffStore` — hai nhóm trạng thái tách rõ, và bài học HIST-09.
 *
 * Trạng thái của trình xem diff chia làm **hai loại khác nhau về bản chất**, và
 * việc đặt sai loại là một lỗi UX im lặng:
 *
 * - **Theo repo** (`selectedFileByRepo`) — khuôn `selectionStore` (PLAT-05). Tệp
 *   đang chọn là *dữ liệu vị trí* trong một repo cụ thể. Chọn tệp ở repo A không
 *   được ảnh hưởng repo B.
 * - **Không theo repo** (`viewMode`, `showWhitespace`) — khuôn `uiStore`. Đây là
 *   *lựa chọn của người dùng* về cách xem, đúng loại với `fileListView`: "người
 *   dùng đã chọn cách xem, không phải chọn lại mỗi lần" (`<behavior>` Task 2,
 *   plan 02-06 → HIST-09).
 *
 * Mutation #2 của plan đổi `viewMode` sang khuôn theo repo; các test "đổi repo"
 * dưới đây là cổng cho nó.
 */

import { beforeEach, describe, expect, it } from 'vitest'

import { useDiffStore } from './diffStore'

beforeEach(() => {
  useDiffStore.setState({
    selectedFileByRepo: {},
    viewMode: 'unified',
    showWhitespace: false,
  })
})

describe('selectedFileByRepo — khuôn theo repo (PLAT-05)', () => {
  it('chọn tệp ở repo A không ảnh hưởng repo B', () => {
    const s = useDiffStore.getState()
    s.selectFile('repoA', 'src/a.ts')
    s.selectFile('repoB', 'src/b.rs')

    const sau = useDiffStore.getState().selectedFileByRepo
    expect(sau['repoA']).toBe('src/a.ts')
    expect(sau['repoB']).toBe('src/b.rs')
  })

  it('repo chưa chọn gì trả về undefined, không phải chuỗi rỗng', () => {
    expect(useDiffStore.getState().selectedFileByRepo['chua-mo']).toBeUndefined()
  })

  it('clearFile đặt về null cho đúng repo đó, không xoá repo khác', () => {
    const s = useDiffStore.getState()
    s.selectFile('repoA', 'src/a.ts')
    s.selectFile('repoB', 'src/b.rs')
    useDiffStore.getState().clearFile('repoA')

    const sau = useDiffStore.getState().selectedFileByRepo
    expect(sau['repoA']).toBeNull()
    expect(sau['repoB']).toBe('src/b.rs')
  })

  it('store CHỈ giữ đường dẫn, không giữ FileDiff', () => {
    useDiffStore.getState().selectFile('r', 'src/a.ts')
    const giaTri = useDiffStore.getState().selectedFileByRepo['r']

    // Cùng lý do `selectionStore` chỉ giữ `selectedCommitId`: nhét dữ liệu diff
    // vào đây làm mọi thành phần đọc store render lại mỗi lần chọn tệp.
    expect(typeof giaTri).toBe('string')
    expect(giaTri).toBe('src/a.ts')
  })
})

describe('đổi commit → selectedFile của repo đó về null', () => {
  /*
   * Tệp cũ có thể KHÔNG tồn tại trong commit mới. Giữ nó nghĩa là gọi
   * `getFileDiff` với một path không có trong commit và hiện LỖI cho một thao
   * tác hoàn toàn bình thường (chọn commit khác).
   *
   * Mutation #3 của plan bỏ bước reset này.
   */
  it('commitChanged đặt selectedFile của repo đó về null', () => {
    useDiffStore.getState().selectFile('r', 'src/chi-co-o-commit-cu.ts')
    useDiffStore.getState().commitChanged('r')

    expect(useDiffStore.getState().selectedFileByRepo['r']).toBeNull()
  })

  it('commitChanged ở repo A không xoá lựa chọn của repo B', () => {
    const s = useDiffStore.getState()
    s.selectFile('repoA', 'a.ts')
    s.selectFile('repoB', 'b.ts')
    useDiffStore.getState().commitChanged('repoA')

    const sau = useDiffStore.getState().selectedFileByRepo
    expect(sau['repoA']).toBeNull()
    expect(sau['repoB']).toBe('b.ts')
  })

  it('commitChanged KHÔNG làm mất viewMode — đó là lựa chọn người dùng, không phải dữ liệu', () => {
    useDiffStore.getState().setViewMode('split')
    useDiffStore.getState().selectFile('r', 'a.ts')
    useDiffStore.getState().commitChanged('r')

    expect(useDiffStore.getState().viewMode).toBe('split')
  })
})

describe('viewMode — KHÔNG theo repo (khuôn uiStore, bài học HIST-09)', () => {
  it('mặc định là unified', () => {
    expect(useDiffStore.getState().viewMode).toBe('unified')
  })

  it('toggleViewMode đổi qua lại giữa unified và split', () => {
    useDiffStore.getState().toggleViewMode()
    expect(useDiffStore.getState().viewMode).toBe('split')
    useDiffStore.getState().toggleViewMode()
    expect(useDiffStore.getState().viewMode).toBe('unified')
  })

  it('🔴 đổi chế độ rồi chọn commit khác → chế độ CÒN NGUYÊN', () => {
    useDiffStore.getState().setViewMode('split')
    useDiffStore.getState().commitChanged('r')

    expect(useDiffStore.getState().viewMode).toBe('split')
  })

  it('🔴 đổi chế độ rồi chọn TỆP KHÁC → chế độ CÒN NGUYÊN', () => {
    useDiffStore.getState().setViewMode('split')
    useDiffStore.getState().selectFile('r', 'khac.ts')

    expect(useDiffStore.getState().viewMode).toBe('split')
  })

  it('🔴 đổi chế độ ở repo A → repo B thấy CÙNG chế độ (không theo repo)', () => {
    // Cổng của mutation #2: một cài đặt `viewModeByRepo` sẽ cho repo B chế độ
    // mặc định `unified` ở đây. `viewMode` là lựa chọn NGƯỜI DÙNG về cách xem,
    // không phải trạng thái của một repo — đúng loại với `uiStore.fileListView`.
    useDiffStore.getState().selectFile('repoA', 'a.ts')
    useDiffStore.getState().setViewMode('split')

    // "Chuyển sang repo B" ở tầng store nghĩa là đọc lại cùng một giá trị.
    useDiffStore.getState().selectFile('repoB', 'b.ts')
    expect(useDiffStore.getState().viewMode).toBe('split')

    // Và khẳng định mạnh hơn: không có khoá TRẠNG THÁI nào mang chế độ theo
    // repo. Lọc bỏ các hành động (giá trị là hàm) — `setViewMode`/`toggleViewMode`
    // đều hợp lệ và đều khớp `/viewMode/i`, nên một cổng khớp trên MỌI khoá sẽ
    // đỏ vì lý do sai (đã gặp: nó đỏ ngay ở cài đặt đúng).
    const state = useDiffStore.getState() as unknown as Record<string, unknown>
    const khoaTrangThai = Object.keys(state).filter((k) => typeof state[k] !== 'function')
    expect(
      khoaTrangThai.filter((k) => /viewmode/i.test(k)),
      `chỉ được có ĐÚNG một khoá trạng thái viewMode (không theo repo), ` +
        `thấy: ${khoaTrangThai.join(', ')}`,
    ).toEqual(['viewMode'])
    // Và không khoá trạng thái nào có hậu tố `ByRepo` ngoài hai khoá VỊ TRÍ:
    // tệp đang chọn, và nguồn của nó (thư mục làm việc hay commit).
    expect(khoaTrangThai.filter((k) => k.endsWith('ByRepo')).sort()).toEqual([
      'selectedFileByRepo',
      'worktreeByRepo',
    ])
  })
})

describe('showWhitespace — cũng KHÔNG theo repo (DIFF-04)', () => {
  it('mặc định tắt', () => {
    expect(useDiffStore.getState().showWhitespace).toBe(false)
  })

  it('toggleWhitespace bật tắt được', () => {
    useDiffStore.getState().toggleWhitespace()
    expect(useDiffStore.getState().showWhitespace).toBe(true)
    useDiffStore.getState().toggleWhitespace()
    expect(useDiffStore.getState().showWhitespace).toBe(false)
  })

  it('🔴 bật khoảng trắng rồi đổi repo và đổi commit → CÒN NGUYÊN', () => {
    useDiffStore.getState().toggleWhitespace()
    useDiffStore.getState().selectFile('repoB', 'b.ts')
    useDiffStore.getState().commitChanged('repoB')

    expect(useDiffStore.getState().showWhitespace).toBe(true)
  })

  it('không có khoá TRẠNG THÁI nào mang cờ khoảng trắng theo repo', () => {
    const state = useDiffStore.getState() as unknown as Record<string, unknown>
    const khoaTrangThai = Object.keys(state).filter((k) => typeof state[k] !== 'function')
    expect(khoaTrangThai.filter((k) => /whitespace/i.test(k))).toEqual(['showWhitespace'])
  })
})
