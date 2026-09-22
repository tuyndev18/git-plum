/**
 * Dựng tài liệu và decoration CodeMirror **từ dữ liệu `git diff`** — không để
 * CodeMirror tính lại diff.
 *
 * # Vì sao tầng này là hàm thuần, tách khỏi `.tsx`
 *
 * happy-dom không tính layout CSS, nên mọi logic đặt trong một component sẽ chỉ
 * được kiểm bằng mắt người. Đây đúng bài học ba lỗi hiển thị của Phase 2 (badge
 * phá chiều cao hàng, đường vẽ tràn hàng, sai hệ quy chiếu toạ độ canvas) — cả
 * ba **qua hết 212 test tự động**. Nên mọi thứ tính được thì tính ở đây, nơi test
 * chạy thật.
 *
 * # 🔴 Hai hệ đếm: `Span` là BYTE, CodeMirror là UTF-16
 *
 * Đây là chỗ nguy hiểm nhất của bàn giao từ wave 3, và nó cùng **lớp lỗi** với
 * "sai hệ quy chiếu toạ độ canvas" mà Phase 2 đã trả giá.
 *
 * | ký tự | byte (UTF-8) | đơn vị UTF-16 |
 * |---|---|---|
 * | `a` | 1 | 1 |
 * | `é` | 2 | 1 |
 * | `ỏ` | 3 | 1 |
 * | `🙂` (ngoài BMP) | 4 | **2** |
 *
 * Trên `'xéy dỏng TEST'`, `TEST` ở **byte 12** nhưng **UTF-16 index 9**. Dùng
 * thẳng con số của Rust đặt decoration **sai chỗ** trên **mọi** dòng tiếng Việt
 * — và dự án này viết tài liệu, comment và chuỗi giao diện bằng tiếng Việt.
 *
 * Phép chuyển dùng `TextEncoder`/`TextDecoder`, **không** `Buffer`: mã sản phẩm
 * là browser-only (không có `tauri-plugin-fs`), xem chú thích cùng nội dung
 * trong `app.css.test.ts`.
 *
 * # `spans` rỗng là ca BÌNH THƯỜNG
 *
 * Bốn đường dẫn hợp lệ tới một `spans` rỗng: dòng ngữ cảnh (không đổi), tệp chỉ
 * thêm, tệp chỉ xoá, và tệp sửa quá 2000 dòng (suy giảm có chủ ý của wave 3).
 * Tô **cả dòng** trong bốn ca đó là suy giảm đúng, không phải trạng thái lỗi cần
 * báo.
 */

import { Decoration, type DecorationSet } from '@codemirror/view'
import { RangeSetBuilder } from '@codemirror/state'

import type { DiffLine, Hunk, LineKind, Span } from '@/lib/ipc'

/** Phía tài liệu cần dựng. `'unified'` giữ mọi dòng theo đúng thứ tự git in. */
export type DiffSide = 'old' | 'new' | 'unified'

/** Khoảng đã chuyển sang **đơn vị UTF-16** — an toàn để đưa cho CodeMirror. */
export interface Utf16Span {
  from: number
  to: number
}

/**
 * Thông tin một dòng của tài liệu đã dựng.
 *
 * `spans` ở đây **đã** ở hệ UTF-16. Kiểu riêng (`Utf16Span`, không `Span`) là có
 * chủ ý: nó làm việc truyền lẫn hai hệ thành lỗi **biên dịch**, không phải lỗi
 * hiển thị im lặng.
 */
export interface LineMeta {
  kind: LineKind
  oldLine: number | null
  newLine: number | null
  /** Chỉ số hunk chứa dòng này, đếm từ 0. Dùng cho việc nhảy khối. */
  hunkIndex: number
  /** Khoảng chữ thay đổi, **đơn vị UTF-16**. Rỗng là ca bình thường. */
  spans: Utf16Span[]
}

export interface DiffDoc {
  /** Nội dung tài liệu, các dòng nối bằng `\n`. */
  text: string
  /** **Luôn cùng độ dài** với số dòng của `text` — lệch một phần tử là lệch hàng. */
  lineMeta: LineMeta[]
}

/**
 * Chuyển một `Span` (byte) sang `Utf16Span`, hay `null` khi span không dùng được.
 *
 * `null` cho **bốn** ca, và cả bốn là "bỏ span đó", không phải "ném":
 *
 * 1. `start === end` — khoảng rỗng, một mark rộng 0 không vẽ được gì.
 * 2. `start > end` — dữ liệu backend lỗi.
 * 3. `end` vượt độ dài byte của `content` — dữ liệu backend lỗi.
 * 4. offset âm.
 *
 * **Vì sao bỏ chứ không ném (T-03-24).** CodeMirror **ném** khi range vượt biên
 * document, và một ngoại lệ ở tầng này làm **trắng cả panel** — người dùng mất
 * luôn phần diff đúng vì một span sai. Bỏ span cho họ một dòng tô cả dòng, tức
 * đúng cách suy giảm mà ca `spans` rỗng vốn đã dùng.
 */
export function spanToUtf16(content: string, span: Span): Utf16Span | null {
  const { start, end } = span
  if (!Number.isFinite(start) || !Number.isFinite(end)) return null
  if (start < 0 || end < 0) return null
  if (start >= end) return null

  const bytes = new TextEncoder().encode(content)
  if (end > bytes.length) return null

  const decoder = new TextDecoder()
  const from = decoder.decode(bytes.slice(0, start)).length
  const to = decoder.decode(bytes.slice(0, end)).length

  // Phòng thủ cuối: nếu phép chuyển ra một khoảng vô nghĩa (chỉ xảy ra khi
  // `start`/`end` không nằm trên biên ký tự — phía Rust bảo đảm là có, nhưng
  // bảo đảm của người khác không phải phép kiểm của mình).
  if (from >= to || to > content.length) return null

  return { from, to }
}

/**
 * Dựng tài liệu cho một phía, cùng `lineMeta` song song.
 *
 * Phép lọc theo phía đọc **`oldLine`/`newLine`**, không đọc `kind`: đó là hai
 * trường mà 03-02 tách ra chính cho mục đích này, và một dòng ngữ cảnh có **cả
 * hai** nên nó xuất hiện ở cả hai phía — đúng điều chế độ hai cột cần để hai bên
 * thẳng hàng.
 */
export function hunksToDoc(hunks: Hunk[], side: DiffSide): DiffDoc {
  const contents: string[] = []
  const lineMeta: LineMeta[] = []

  const nhan = (line: DiffLine): boolean => {
    if (side === 'unified') return true
    if (side === 'old') return line.oldLine !== null
    return line.newLine !== null
  }

  hunks.forEach((h, hunkIndex) => {
    for (const line of h.lines) {
      if (!nhan(line)) continue

      contents.push(line.content)
      lineMeta.push({
        kind: line.kind,
        oldLine: line.oldLine,
        newLine: line.newLine,
        hunkIndex,
        // Chuyển hệ **ở đây**, một lần, ngay lúc dữ liệu vào tầng vẽ. Chuyển
        // muộn hơn (trong `buildDecorations`) nghĩa là `lineMeta` mang byte và
        // một người gọi khác sẽ dùng thẳng con số đó.
        spans: line.spans
          .map((s) => spanToUtf16(line.content, s))
          .filter((s): s is Utf16Span => s !== null),
      })
    }
  })

  return { text: contents.join('\n'), lineMeta }
}

/** Kết quả của `buildDecorations` — kèm hai con số để test đếm được. */
export interface BuiltDecorations {
  set: DecorationSet
  /** Số `Decoration.line` (nền dòng thêm/xoá). */
  lineCount: number
  /** Số `Decoration.mark` (khoảng chữ thay đổi trong dòng). */
  markCount: number
}

/** Class CSS cho nền dòng. Màu thật khai trong `app.css` — xem `theme.ts`. */
const LINE_CLASS: Partial<Record<LineKind, string>> = {
  added: 'diff-line-added',
  removed: 'diff-line-removed',
  // `context` cố ý không có class: dòng không đổi thì không tô nền.
}

/**
 * Dựng `DecorationSet` từ `lineMeta`.
 *
 * Trả về cả hai con số đếm được vì `<verification>` của plan nêu rõ vì sao không
 * dùng `grep -c 'Decoration' decorations.ts` (khớp cả `import`, cả doc comment):
 * cổng đúng là khẳng định **số deco thật** cho một `lineMeta` đã biết.
 *
 * `RangeSetBuilder` đòi range được thêm theo **thứ tự vị trí tăng dần**. Line-deco
 * của một dòng phải vào trước mark-deco trong dòng đó, và các span trong một dòng
 * phải đã sắp xếp — nên `spans` được sắp lại tại đây thay vì tin phía gọi.
 */
export function buildDecorations(text: string, lineMeta: LineMeta[]): BuiltDecorations {
  const builder = new RangeSetBuilder<Decoration>()
  let lineCount = 0
  let markCount = 0

  // Vị trí bắt đầu của từng dòng trong tài liệu. Tính một lượt thay vì gọi
  // `doc.line(n)` — tầng này không có `EditorState` và không cần một cái.
  let offset = 0
  const lines = text.length === 0 && lineMeta.length === 0 ? [] : text.split('\n')

  for (let i = 0; i < lines.length; i++) {
    const meta = lineMeta[i]
    const content = lines[i] ?? ''
    if (!meta) {
      offset += content.length + 1
      continue
    }

    const lineClass = LINE_CLASS[meta.kind]
    if (lineClass) {
      builder.add(offset, offset, Decoration.line({ class: lineClass }))
      lineCount++
    }

    for (const span of [...meta.spans].sort((a, b) => a.from - b.from)) {
      // Kẹp vào biên dòng: `spanToUtf16` đã kiểm theo `content` mà Rust gửi,
      // nhưng dòng trong tài liệu là thứ CodeMirror thật sự đánh chỉ số.
      if (span.from >= span.to) continue
      if (span.to > content.length) continue

      builder.add(
        offset + span.from,
        offset + span.to,
        Decoration.mark({ class: 'diff-word-changed' }),
      )
      markCount++
    }

    // `+1` cho ký tự `\n` ngăn cách. Dòng cuối không có `\n` nhưng offset của
    // nó không còn được dùng sau đó.
    offset += content.length + 1
  }

  return { set: builder.finish(), lineCount, markCount }
}

/**
 * Số dòng (đếm từ 1) đầu của khối **kế tiếp**, hay `null` khi đang ở khối cuối.
 *
 * **`null` là hợp đồng, không phải ca lỗi.** Nó cho giao diện nói "đây là khối
 * cuối" thay vì nhảy vòng lại im lặng. DIFF-03 viết "nhảy tới kế tiếp và trước
 * đó" — nó **không** viết "nhảy vòng", và một cú nhảy vòng im lặng làm người
 * dùng tưởng mình đang ở khối mới trong khi đã quay về đầu tệp.
 *
 * Phép tìm neo vào **`hunkIndex`**, không vào "dòng thêm/xoá kế tiếp". Hai hunk
 * **liền nhau** (không dòng ngữ cảnh nào giữa chúng) vẫn là **hai** khối, và một
 * cài đặt kiểu "tìm dòng đổi kế tiếp" sẽ gộp chúng thành một.
 */
export function nextHunkLine(lineMeta: LineMeta[], currentLine: number): number | null {
  if (lineMeta.length === 0) return null

  const hienTai = lineMeta[currentLine - 1]?.hunkIndex ?? -1
  for (let i = 0; i < lineMeta.length; i++) {
    const idx = lineMeta[i]!.hunkIndex
    if (idx > hienTai) return i + 1
  }
  return null
}

/** Số dòng đầu của khối **trước đó**, hay `null` khi đang ở khối đầu. */
export function prevHunkLine(lineMeta: LineMeta[], currentLine: number): number | null {
  if (lineMeta.length === 0) return null

  const hienTai = lineMeta[currentLine - 1]?.hunkIndex
  if (hienTai === undefined || hienTai <= 0) return null

  const dich = hienTai - 1
  for (let i = 0; i < lineMeta.length; i++) {
    if (lineMeta[i]!.hunkIndex === dich) return i + 1
  }
  return null
}
