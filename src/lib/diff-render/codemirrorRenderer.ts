/**
 * Cài đặt `DiffRenderer` bằng CodeMirror — **đường A**, và là tệp DUY NHẤT trong
 * dự án nhập `@codemirror/merge`.
 *
 * # Đây là tệp phải sửa nếu đổi sang đường B
 *
 * `docs/09-phase3-diff-decision.md` mục 7: đường A được chọn **không có số đo**
 * (checkpoint #3 bị bỏ qua ngày 2026-09-22). Nếu `@codemirror/merge` hoá ra chậm
 * trên tệp 630 KB thật, việc chuyển sang đường B là viết một
 * `hunkRenderer.ts` cạnh tệp này rồi đổi **một** dòng import trong
 * `DiffViewer.tsx`. Không một lời gọi CodeMirror nào rò ra ngoài đây — cùng cổng
 * `grep getContext GraphCanvas.tsx` → 0 của Phase 2.
 *
 * # Điều đường A làm tốt hơn đường A "thuần"
 *
 * Kể cả ở đường A, chế độ **hợp nhất** **không** để CodeMirror tính lại diff:
 * `git diff` đã tính rồi, và `decorations.ts` dựng `DecorationSet` từ đầu ra đó.
 * Chỉ chế độ **hai cột** dùng `MergeView`, vì đó là thứ `@codemirror/merge` cho
 * mà tự viết thì tốn nhiều mã (cuộn đồng bộ, căn hàng hai phía).
 *
 * Hệ quả đo được, ghi rõ để không ai nhầm: **nguy cơ hiệu năng của checkpoint #3
 * chỉ áp cho chế độ hai cột**, và chế độ hai cột **không** phải mặc định
 * (`diffStore.viewMode = 'unified'`). Nếu `MergeView` chậm trên tệp lớn thì
 * đường thoát tạm thời của người dùng là chuyển về hợp nhất — một suy giảm có
 * thật, không phải treo giao diện.
 *
 * # Word-level lấy từ git, không để CodeMirror tự tìm
 *
 * `MergeView` có lớp `.cm-changedText` riêng, nhưng dự án dùng `spans` từ
 * `git diff --word-diff-regex=...` (wave 3) cho **cả hai** chế độ. Lý do: biên từ
 * của git là thứ đã được kiểm bằng byte thật ở wave 3, và trộn hai nguồn
 * word-level cho hai chế độ khác nhau nghĩa là người dùng thấy hai kết quả khác
 * nhau cho cùng một dòng.
 */

import { Compartment, EditorState, type Extension } from '@codemirror/state'
import {
  EditorView,
  GutterMarker,
  gutter,
  highlightWhitespace,
  lineNumbers,
  type DecorationSet,
} from '@codemirror/view'
import { MergeView } from '@codemirror/merge'
import { syntaxHighlighting, defaultHighlightStyle } from '@codemirror/language'

import { buildDecorations, type LineMeta } from './decorations'
import { dauThemXoa, soDongThat, type GutterSide } from './gutterNumbers'
import { diffTheme } from './theme'
import type { DiffRenderer, DiffRendererOptions } from './types'

/**
 * Đếm số `EditorView`/`MergeView` đã dựng và đã tháo — **chỉ** cho test.
 *
 * `DiffViewer.test.tsx` cần đếm số view **còn sống** sau khi đổi tệp ba lần, và
 * happy-dom giữ lại node DOM của view cũ theo cách khác nên không đếm được từ
 * DOM. Đây là cách duy nhất chứng minh `destroy()` thật sự được gọi.
 */
export const rendererStats = {
  created: 0,
  destroyed: 0,
  lastInstance: null as unknown,
  reset(): void {
    this.created = 0
    this.destroyed = 0
    this.lastInstance = null
  },
}

/** `Compartment` cho cờ khoảng trắng — đổi extension **tại chỗ**, không dựng lại. */
const whitespaceCompartment = new Compartment()
/** `Compartment` cho ngôn ngữ — đổi tệp không phải dựng lại view. */
const languageCompartment = new Compartment()

/**
 * Gutter số dòng **thật trong tệp** cho một phía.
 *
 * 🔴 Không phải `lineNumbers()` trần. Mặc định đếm 1,2,3… theo **tài liệu của
 * editor này**, và trong chế độ hai cột hai editor mang hai tài liệu khác nhau
 * (phía cũ bỏ dòng `added`, phía mới bỏ dòng `removed`). Nên con số mặc định
 * không phải số dòng trong tệp của người dùng — đúng lỗi họ báo ở checkpoint
 * vòng 2, và đo được bằng Chromium thật: cả hai gutter đọc `1,2,3,…` giống nhau
 * trong khi phía mới phải là `10..17` và phía cũ `10..15`.
 *
 * `soDongThat` trả **chuỗi rỗng** ở vị trí phía này không có dòng. Chỗ trống đó
 * là thông tin: `@codemirror/merge` đã chèn widget `.cm-mergeSpacer` để căn hàng
 * (đo được: 2 spacer, cao 36px và 72px cho hunk 2 và 4 dòng), nên hai cột vẫn
 * thẳng theo nội dung — nhưng một **con số** ở hàng spacer sẽ nói sai.
 *
 * `lineMeta` truyền vào là mảng của **đúng tài liệu** mà editor này giữ, nên
 * `lineNumber` mà CodeMirror đưa vào là chỉ số 1-based hợp lệ cho nó.
 */
function gutterSoDong(lineMeta: LineMeta[], side: GutterSide): Extension {
  return lineNumbers({
    formatNumber: (lineNumber) => soDongThat(lineMeta, side, lineNumber),
  })
}

/**
 * Gutter dấu `+` / `−`, ngay sau số dòng.
 *
 * Phân biệt thêm/xoá **không chỉ bằng màu** — nền dòng của dự án cố ý nhạt
 * (`16%` alpha) để nền word-level đậm hơn còn nổi lên được, và hai nền nhạt
 * đỏ/lục là ca không phân biệt được với người mù màu đỏ-lục.
 */
function gutterDau(lineMeta: LineMeta[]): Extension {
  return gutter({
    class: 'cm-diffSignGutter',
    lineMarker: (view, line) => {
      const n = view.state.doc.lineAt(line.from).number
      const dau = dauThemXoa(lineMeta, n)
      return dau === '' ? null : new DauMarker(dau)
    },
    // Không có dấu nào cũng vẫn giữ cột: thiếu nó thì gutter co giãn theo vùng
    // nhìn và hai cột lệch ngang khi cuộn.
    initialSpacer: () => new DauMarker('+'),
  })
}

/** `GutterMarker` hiện đúng một ký tự `+` hoặc `−`. */
class DauMarker extends GutterMarker {
  constructor(private readonly dau: string) {
    super()
  }

  override eq(other: DauMarker): boolean {
    return other.dau === this.dau
  }

  override toDOM(): Text {
    return document.createTextNode(this.dau)
  }
}

/** Extension cố định dùng cho mọi view của trình xem. */
function extensionsCoDinh(
  deco: DecorationSet,
  language: Extension | null,
  ws: boolean,
  lineMeta: LineMeta[],
  side: GutterSide,
) {
  return [
    gutterSoDong(lineMeta, side),
    gutterDau(lineMeta),
    diffTheme,
    syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
    EditorView.editable.of(false),
    EditorState.readOnly.of(true),
    // Decoration dựng từ dữ liệu `git diff` — CodeMirror không tính lại gì.
    EditorView.decorations.of(deco),
    languageCompartment.of(language ?? []),
    whitespaceCompartment.of(ws ? highlightWhitespace() : []),
  ]
}

/** Dựng trình xem **hợp nhất**: một `EditorView` chỉ-đọc trên tài liệu gộp. */
function dungHopNhat(options: DiffRendererOptions): DiffRenderer {
  const { parent, newDoc, language, showWhitespace } = options
  const { set: deco } = buildDecorations(newDoc.text, newDoc.lineMeta)

  const view = new EditorView({
    parent,
    state: EditorState.create({
      doc: newDoc.text,
      /*
       * Chế độ hợp nhất dùng **một** editor trên tài liệu gộp (mọi dòng, cả
       * `added` lẫn `removed`), nên gutter số dòng ở đây hiện `newLine`.
       *
       * Vì sao `'new'` chứ không phải `'old'`, và vì sao không hiện **cả hai**:
       *
       * - `'new'` là thứ người dùng đang đọc. Sau khi commit được áp, tệp trên
       *   đĩa của họ mang chính hệ số này; mở tệp trong editor rồi nhảy tới số
       *   đó là thao tác thật và thường xuyên.
       * - Hai cột số trong chế độ hợp nhất tốn ~80px bề ngang ở một panel vốn
       *   đã hẹp (`MIN_SPLIT_WIDTH = 720` là mức mà **hai cột nội dung** mới
       *   vừa đủ), và nó đẩy chế độ hợp nhất — vốn là **mặc định** và là đường
       *   thoát khi panel hẹp — về đúng vấn đề bề rộng mà nó tồn tại để tránh.
       * - Dòng `removed` có `newLine === null` nên gutter để **trống** ở đó,
       *   cộng dấu `−` của `gutterDau` là đủ để đọc: chỗ trống + `−` nói "dòng
       *   này không còn ở bản mới".
       *
       * Ai cần đối chiếu số dòng **cả hai phía** thì chuyển sang hai cột, nơi
       * mỗi phía có gutter riêng của nó.
       */
      extensions: extensionsCoDinh(deco, language, showWhitespace, newDoc.lineMeta, 'new'),
    }),
  })

  rendererStats.created++
  rendererStats.lastInstance = view

  return {
    setShowWhitespace: (show) => {
      view.dispatch({
        effects: whitespaceCompartment.reconfigure(show ? highlightWhitespace() : []),
      })
    },
    setLanguage: (lang) => {
      view.dispatch({ effects: languageCompartment.reconfigure(lang ?? []) })
    },
    scrollToLine: (line) => {
      const soDong = view.state.doc.lines
      const n = Math.min(Math.max(line, 1), soDong)
      const pos = view.state.doc.line(n).from
      view.dispatch({
        selection: { anchor: pos },
        effects: EditorView.scrollIntoView(pos, { y: 'start' }),
      })
    },
    currentLine: () => {
      const pos = view.state.selection.main.head
      return view.state.doc.lineAt(pos).number
    },
    destroy: () => {
      view.destroy()
      rendererStats.destroyed++
    },
  }
}

/**
 * Dựng trình xem **hai cột** bằng `MergeView`.
 *
 * 🔴 Đây là đường mã mà checkpoint #3 lo và **chưa ai đo**: `MergeView` nhận hai
 * tài liệu đầy đủ rồi tự tính diff. Trên `yarn.lock` 632 KB nó diff lại hai bản
 * ~630 KB mỗi lần mở.
 */
function dungHaiCot(options: DiffRendererOptions): DiffRenderer {
  const { parent, newDoc, oldDoc, language, showWhitespace } = options
  const { set: decoMoi } = buildDecorations(newDoc.text, newDoc.lineMeta)
  const { set: decoCu } = buildDecorations(oldDoc.text, oldDoc.lineMeta)

  const merge = new MergeView({
    parent,
    /*
     * Mỗi phía nhận `lineMeta` của **chính tài liệu nó** và phía gutter tương
     * ứng: A ↔ `oldLine`, B ↔ `newLine`. Đổi chỗ hai đối số này là lỗi im lặng
     * — gutter vẫn hiện số, chỉ là số của phía kia — nên `gutterNumbers.test.ts`
     * ghim rằng hai phía cho dãy KHÁC nhau trên cùng một hunk.
     */
    a: {
      doc: oldDoc.text,
      extensions: extensionsCoDinh(decoCu, language, showWhitespace, oldDoc.lineMeta, 'old'),
    },
    b: {
      doc: newDoc.text,
      extensions: extensionsCoDinh(decoMoi, language, showWhitespace, newDoc.lineMeta, 'new'),
    },
    // Chỉ-đọc ở cả hai phía: gộp khối (`collapseUnchanged`/nút merge) là việc
    // của Phase 5 (staging theo khối), không của trình **xem**.
    revertControls: undefined,
  })

  rendererStats.created++
  rendererStats.lastInstance = merge

  const caHaiPhia = (fn: (v: EditorView) => void) => {
    fn(merge.a)
    fn(merge.b)
  }

  return {
    setShowWhitespace: (show) => {
      caHaiPhia((v) =>
        v.dispatch({
          effects: whitespaceCompartment.reconfigure(show ? highlightWhitespace() : []),
        }),
      )
    },
    setLanguage: (lang) => {
      caHaiPhia((v) => v.dispatch({ effects: languageCompartment.reconfigure(lang ?? []) }))
    },
    scrollToLine: (line) => {
      // Cuộn phía **mới**: `MergeView` đồng bộ phía cũ theo nó.
      const soDong = merge.b.state.doc.lines
      const n = Math.min(Math.max(line, 1), soDong)
      const pos = merge.b.state.doc.line(n).from
      merge.b.dispatch({
        selection: { anchor: pos },
        effects: EditorView.scrollIntoView(pos, { y: 'start' }),
      })
    },
    currentLine: () => {
      const pos = merge.b.state.selection.main.head
      return merge.b.state.doc.lineAt(pos).number
    },
    destroy: () => {
      merge.destroy()
      rendererStats.destroyed++
    },
  }
}

/**
 * Bộ dựng của đường A.
 *
 * `DiffViewer.tsx` nhập **đúng hàm này** và không gì khác từ CodeMirror. Đổi sang
 * đường B là đổi một dòng import ở đó.
 */
export function createCodeMirrorRenderer(options: DiffRendererOptions): DiffRenderer {
  return options.mode === 'split' ? dungHaiCot(options) : dungHopNhat(options)
}
