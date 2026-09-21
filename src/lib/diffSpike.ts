/**
 * Hai đường đo của checkpoint #3 — plan 03-01. **Spike, không phải tính năng.**
 *
 * Câu hỏi cần trả lời bằng số: `@codemirror/merge` nhận **hai tài liệu đầy đủ** rồi
 * tự tính diff, trong khi `git diff` đã đưa sẵn diff. Trên tệp lớn, việc tính lại đó
 * có nhanh đủ để dùng làm khung hiển thị của Phase 3 hay không?
 *
 * - **Đường A** (`measurePathA`) — `MergeView` trên `oldText`/`newText`. CodeMirror
 *   tự tính diff. Ít mã hơn nhiều nếu đủ nhanh.
 * - **Đường B** (`measurePathB`) — một `EditorView` chỉ-đọc trên `newText`, kèm
 *   decoration dựng từ `patch`. CodeMirror **không** tính lại gì.
 *
 * Cả hai nhận cùng một `SpikeBlobPair` nên chúng đo trên **cùng byte đầu vào**; nếu
 * mỗi đường tự lấy dữ liệu riêng thì phép so sánh khác nhau ở hai biến và mất nghĩa.
 *
 * # Ba quy tắc đo, mỗi quy tắc chống một cách đo sai cụ thể
 *
 * 1. **Đo tới lúc DOM thật sự có nội dung**, không tới lúc constructor trả về.
 *    `new MergeView(...)` trả về trước khi trình duyệt dựng bố cục, nên đo ở đó cho
 *    một con số đẹp vô nghĩa. Chờ **hai** khung `requestAnimationFrame`: khung đầu
 *    có thể chạy trước lúc bố cục được tính. Cùng bài học với `measureFirstPaint`
 *    của Phase 2 — nó đo tới `commits.length > 0`, không tới `useEffect` đầu tiên.
 * 2. **Chạy ba lần, báo cả ba số**, không báo trung bình. Lần đầu gánh chi phí nạp
 *    module; gộp vào trung bình che mất cả hai thông tin.
 * 3. **Tháo view giữa hai lần đo** rồi dựng lại từ đầu, nếu không lần sau đo trên
 *    một view đã ấm.
 */

import { EditorState, type Extension } from '@codemirror/state'
import { Decoration, EditorView, type DecorationSet } from '@codemirror/view'
import { MergeView } from '@codemirror/merge'

import type { SpikeBlobPair } from './ipc'

/** Số lần chạy mỗi đường. Ba lần: một lần ấm module, hai lần so lại với nhau. */
export const SO_LAN_DO = 3

/** Một khối thay đổi đã đọc từ `git diff --unified=3`. */
export interface Hunk {
  /** Dòng bắt đầu của hunk ở phía **mới**, đếm từ 1 như git in. */
  newStart: number
  /** Dòng bắt đầu của hunk ở phía **cũ**, đếm từ 1. */
  oldStart: number
  /** Nội dung các dòng được thêm, đã bỏ tiền tố `+`. */
  added: string[]
  /** Nội dung các dòng bị xoá, đã bỏ tiền tố `-`. */
  removed: string[]
  /**
   * Số dòng **phía mới** của từng dòng trong `added`.
   *
   * Dòng bị xoá **không** tồn tại ở phía mới nên không tăng bộ đếm này. Đếm lẫn là
   * lỗi làm decoration gắn lệch dòng, và nó chỉ hiện rõ ở hunk thứ hai trở đi.
   */
  addedLineNumbers: number[]
}

/**
 * Đọc `git diff --unified=3` thành danh sách hunk.
 *
 * Chỉ đọc đủ cho phép đo: vị trí hunk và các dòng thêm/xoá. Đây **không** phải bộ
 * phân tích bản vá của Phase 5 — bộ đó phải giữ byte thô và xử lý cả `\ No newline
 * at end of file`.
 *
 * Bốn cái bẫy đã tính tới:
 * - `+++ b/file` và `--- a/file` bắt đầu bằng `+`/`-` nhưng **không** phải dòng
 *   thay đổi. Chỉ đọc dòng sau khi đã gặp một đầu `@@`.
 * - `@@ -5 +5 @@` (không dấu phẩy) là dạng git dùng cho hunk một dòng.
 * - git trả `\r\n` trên Windows; `\r` sót lại làm mọi phép so khớp lệch.
 * - Dòng ngữ cảnh bắt đầu bằng một dấu cách, dòng rỗng trong ngữ cảnh có thể là
 *   chuỗi rỗng hoàn toàn.
 */
export function phanTichHunk(patch: string): Hunk[] {
  const hunks: Hunk[] = []
  let hienTai: Hunk | null = null
  let dongMoi = 0

  // `\r` bị cắt ở từng dòng thay vì thay thế cả chuỗi, để một `\r` giữa dòng (có
  // trong tệp thật) không bị đụng tới.
  for (const raw of patch.split('\n')) {
    const line = raw.endsWith('\r') ? raw.slice(0, -1) : raw

    if (line.startsWith('@@')) {
      // `@@ -oldStart[,oldCount] +newStart[,newCount] @@ [ngữ cảnh]`
      const m = /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@/.exec(line)
      if (!m) continue

      hienTai = {
        oldStart: Number(m[1]),
        newStart: Number(m[2]),
        added: [],
        removed: [],
        addedLineNumbers: [],
      }
      hunks.push(hienTai)
      dongMoi = hienTai.newStart
      continue
    }

    // Trước đầu `@@` đầu tiên là phần đầu tệp — bỏ qua hoàn toàn. Đây là thứ chặn
    // `+++ b/file` bị đọc thành dòng thêm.
    if (!hienTai) continue

    if (line.startsWith('+')) {
      hienTai.added.push(line.slice(1))
      hienTai.addedLineNumbers.push(dongMoi)
      dongMoi += 1
    } else if (line.startsWith('-')) {
      // Dòng xoá không có ở phía mới → KHÔNG tăng `dongMoi`.
      hienTai.removed.push(line.slice(1))
    } else if (line.startsWith('\\')) {
      // `\ No newline at end of file` — không phải một dòng nội dung.
      continue
    } else {
      // Dòng ngữ cảnh (tiền tố dấu cách) hoặc dòng rỗng ở cuối patch.
      dongMoi += 1
    }
  }

  return hunks
}

/**
 * Chờ tới khi trình duyệt đã dựng bố cục xong.
 *
 * Hai khung, không một: khung đầu tiên có thể chạy **trước** lúc bố cục được tính,
 * nên đo ở đó vẫn sớm. Xem quy tắc 1 ở đầu tệp.
 */
function choVeXong(): Promise<void> {
  return new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
  })
}

/** Kết quả một lần chạy một đường: ms và thứ cần tháo sau đó. */
interface LanDo {
  ms: number
  thao: () => void
}

/**
 * Đo `SO_LAN_DO` lần một đường, tháo view giữa các lần.
 *
 * Trả **cả ba** số theo thứ tự chạy. Không trung bình hoá — xem quy tắc 2.
 */
async function doNhieuLan(chay: () => Promise<LanDo>): Promise<number[]> {
  const ketQua: number[] = []
  for (let i = 0; i < SO_LAN_DO; i += 1) {
    const { ms, thao } = await chay()
    ketQua.push(ms)
    // Quy tắc 3: tháo rồi dựng lại từ đầu, nếu không lần sau đo trên view đã ấm.
    thao()
  }
  return ketQua
}

/** Dọn sạch mọi con của một phần tử, để lần đo sau bắt đầu từ DOM rỗng. */
function donSach(parent: HTMLElement): void {
  while (parent.firstChild) parent.removeChild(parent.firstChild)
}

/**
 * **Đường A** — để `@codemirror/merge` tự tính diff từ hai tài liệu đầy đủ.
 *
 * Đây chính là thứ đang bị nghi chậm: `MergeView` nhận `a`/`b` là hai chuỗi hoàn
 * chỉnh và tự chạy thuật toán diff, dù `git diff` đã tính xong việc đó.
 *
 * @returns ba số ms theo thứ tự chạy.
 */
export async function measurePathA(
  pair: SpikeBlobPair,
  parent: HTMLElement,
): Promise<number[]> {
  return doNhieuLan(async () => {
    donSach(parent)

    const batDau = performance.now()

    const view = new MergeView({
      a: { doc: pair.oldText, extensions: [EditorView.editable.of(false)] },
      b: { doc: pair.newText, extensions: [EditorView.editable.of(false)] },
      parent,
    })

    await choVeXong()
    const ms = performance.now() - batDau

    return { ms, thao: () => view.destroy() }
  })
}

/** Decoration tô cả dòng cho dòng được thêm. */
const NEN_THEM = Decoration.line({ class: 'spike-added' })
/** Decoration tô cả dòng cho dòng có nội dung bị xoá ngay trước nó. */
const NEN_XOA = Decoration.line({ class: 'spike-removed' })

/**
 * Dựng `DecorationSet` từ danh sách hunk, trên tài liệu phía mới.
 *
 * Tách riêng để đo được phần này mà không cần bố cục, và để `measurePathB` đọc
 * được ngắn gọn.
 */
function dungDecoration(state: EditorState, hunks: Hunk[]): DecorationSet {
  const tong = state.doc.lines
  const moc: ReturnType<typeof NEN_THEM.range>[] = []

  for (const h of hunks) {
    for (const so of h.addedLineNumbers) {
      // Patch có thể trỏ ra ngoài tài liệu nếu `patch` và `newText` lệch nhau (ví
      // dụ người chạy gõ sai path). Bỏ qua thay vì để CodeMirror ném.
      if (so < 1 || so > tong) continue
      moc.push(NEN_THEM.range(state.doc.line(so).from))
    }
    // Dòng bị xoá không có ở phía mới; neo dấu vào dòng đầu hunk để vẫn có một
    // decoration tương ứng, đủ cho phép đo chi phí vẽ.
    if (h.removed.length > 0) {
      const so = Math.min(Math.max(h.newStart, 1), tong)
      if (tong > 0) moc.push(NEN_XOA.range(state.doc.line(so).from))
    }
  }

  // `Decoration.set` đòi các mốc đã sắp xếp theo vị trí.
  return Decoration.set(moc, true)
}

/**
 * **Đường B** — decoration dựng từ đầu ra `git diff`, CodeMirror không tính lại gì.
 *
 * Đây là đường lùi chính thức mà ROADMAP nêu. Nó **không phải thi với ai**: ngưỡng
 * của checkpoint #3 chỉ áp cho đường A, còn số của đường B được ghi lại để biết chi
 * phí sàn của việc dựng một `EditorView` trên cùng tệp.
 *
 * @returns ba số ms theo thứ tự chạy.
 */
export async function measurePathB(
  pair: SpikeBlobPair,
  parent: HTMLElement,
): Promise<number[]> {
  return doNhieuLan(async () => {
    donSach(parent)

    const batDau = performance.now()

    // Phân tích patch nằm **trong** phép đo: đó là việc đường B phải làm mà đường A
    // không phải làm. Để nó ra ngoài sẽ làm đường B trông nhanh hơn thực tế.
    const hunks = phanTichHunk(pair.patch)

    const toMau: Extension = EditorView.decorations.compute(['doc'], (state) =>
      dungDecoration(state as EditorState, hunks),
    )

    const view = new EditorView({
      state: EditorState.create({
        doc: pair.newText,
        extensions: [EditorView.editable.of(false), toMau],
      }),
      parent,
    })

    await choVeXong()
    const ms = performance.now() - batDau

    return { ms, thao: () => view.destroy() }
  })
}

/**
 * Ca kiểm word-level của Task 2 bước 4 — đúng ca chủ dự án đưa.
 *
 * CONTEXT.md mục 4 viết "`@codemirror/merge` *có thể* tự cho word-level — cần kiểm".
 * Đây là dữ liệu để kiểm bằng mắt: nếu `MergeView` tô **riêng** `==`/`===` trong
 * dòng thì nó có word-level; nếu tô cả dòng thì không, và plan 03-03 phải lấy
 * word-level từ git.
 */
export const CA_WORD_LEVEL = {
  oldText: 'function f(cellData) {\n  if (typeof cellData == "object") {\n    return 1\n  }\n}\n',
  newText: 'function f(cellData) {\n  if (typeof cellData === "object") {\n    return 1\n  }\n}\n',
}

/**
 * Dựng `MergeView` cho ca word-level rồi **đếm** số phần tử tô trong dòng đã sửa.
 *
 * Trả về số phần tử `.cm-changedText` mà `@codemirror/merge` tạo ra. Đây là lớp mà
 * CodeMirror 6 dùng cho phần chữ thay đổi **bên trong** một dòng; `.cm-changedLine`
 * là lớp cho cả dòng. Nên:
 *
 * - `> 0` → có word-level thật, tô riêng `==`/`===`.
 * - `0` nhưng có `.cm-changedLine` → chỉ tô cả dòng, **không** có word-level.
 *
 * Đếm bằng mã thay vì chỉ nhìn bằng mắt, vì "tôi thấy nó có tô" không phân biệt
 * được hai lớp trên — và cả plan 03-03 phụ thuộc vào việc phân biệt đúng.
 */
export async function kiemWordLevel(parent: HTMLElement): Promise<{
  changedText: number
  changedLine: number
  /** HTML của dòng đã sửa, để dán vào tài liệu làm bằng chứng. */
  htmlDongSua: string
}> {
  donSach(parent)

  const view = new MergeView({
    a: { doc: CA_WORD_LEVEL.oldText, extensions: [EditorView.editable.of(false)] },
    b: { doc: CA_WORD_LEVEL.newText, extensions: [EditorView.editable.of(false)] },
    parent,
  })

  await choVeXong()

  const changedText = parent.querySelectorAll('.cm-changedText').length
  const changedLine = parent.querySelectorAll('.cm-changedLine').length

  const dong = Array.from(parent.querySelectorAll('.cm-changedLine')).find((el) =>
    el.textContent?.includes('typeof cellData'),
  )
  const htmlDongSua = dong?.innerHTML ?? ''

  view.destroy()

  return { changedText, changedLine, htmlDongSua }
}
