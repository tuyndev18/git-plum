/**
 * Theme CodeMirror cho trình xem diff — **chỉ đặt cấu trúc, không đặt màu**.
 *
 * # Vì sao theme không mang một màu nào
 *
 * `EditorView.theme({...})` nhận một object JS và CodeMirror sinh ra các quy tắc
 * CSS với class băm (`cm-xxxxx`). Nếu đặt màu ở đây thì dự án có **hai** bảng
 * màu: một trong `app.css` (`:root { --bg, --text, --accent, … }`, có cả nhánh
 * theme sáng) và một trong tệp này. Đó đúng lớp lỗi mà `REF_COL_WIDTH` đã dạy —
 * một giá trị ở hai nơi là **lỗi im lặng**, mỗi phía tự nó vẫn nhất quán.
 *
 * Tệ hơn: theme sáng của dự án bật qua `@media (prefers-color-scheme: light)`
 * trong `app.css`. Màu đặt trong object JS **không** đi qua cascade đó, nên
 * trình xem diff sẽ giữ màu tối trong khi cả phần còn lại của giao diện đã đổi
 * sang sáng.
 *
 * Nên cách sạch nhất: theme ở đây chỉ đặt những thứ **phải** là JS (bố cục của
 * chính editor, phông chữ kế thừa, bỏ outline), còn mọi màu khai trong `app.css`
 * theo class ổn định mà `decorations.ts` gắn (`.diff-line-added`,
 * `.diff-line-removed`, `.diff-word-changed`). `app.css.test.ts` đọc thẳng
 * nguồn CSS nên nó cũng chỉ kiểm được các class ổn định đó — một màu nằm trong
 * object JS sẽ vô hình với cả bốn test bố cục của plan này.
 */

import { EditorView } from '@codemirror/view'
import type { Extension } from '@codemirror/state'

/**
 * Theme của trình xem diff.
 *
 * Mọi giá trị dưới đây là **cấu trúc**, không phải bảng màu: `inherit` đẩy quyết
 * định màu và phông về `app.css`, nơi biến CSS và nhánh theme sáng đang sống.
 */
export const diffTheme: Extension = EditorView.theme({
  '&': {
    // Kế thừa phông và màu từ `.diff-viewer` trong `app.css` — gồm
    // `var(--font-mono)` và nhánh theme sáng.
    fontFamily: 'inherit',
    fontSize: 'inherit',
    color: 'inherit',
    backgroundColor: 'transparent',
    height: '100%',
  },
  '&.cm-focused': {
    // Trình xem là chỉ-đọc; một viền focus xanh mặc định của trình duyệt trên
    // một thứ không sửa được chỉ gây nhiễu.
    outline: 'none',
  },
  '.cm-scroller': {
    fontFamily: 'inherit',
    lineHeight: 'inherit',
    // Cuộn nằm **bên trong** editor, không đẩy panel giãn — cùng lý do
    // `.diff-viewer` phải có `overflow: hidden` + `min-height: 0`.
    overflow: 'auto',
  },
  '.cm-gutters': {
    backgroundColor: 'transparent',
    color: 'inherit',
    border: 'none',
    opacity: '0.55',
  },
  '.cm-content': {
    // Con trỏ chèn không có nghĩa trên tài liệu chỉ-đọc.
    caretColor: 'transparent',
  },
  '.cm-activeLine': {
    // Nền dòng đang trỏ của CodeMirror sẽ **phủ lên** nền dòng thêm/xoá và xoá
    // mất tín hiệu "dòng này đã thêm" — cùng lỗi `.commit-row:hover` xoá đồ thị
    // canvas ở Phase 2. Tắt hẳn; dòng đang trỏ không mang thông tin nào ở đây.
    backgroundColor: 'transparent',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'transparent',
  },
})
