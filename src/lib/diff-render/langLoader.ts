/**
 * Nạp **lười** chế độ ngôn ngữ CodeMirror theo phần mở rộng tệp — DIFF-01.
 *
 * # Vì sao bắt buộc nạp lười, không phải "nên"
 *
 * Có hơn 30 gói `@codemirror/lang-*`. Nhập sẵn tất cả (hay chỉ sáu gói dưới đây)
 * ở đầu tệp đưa toàn bộ chúng vào chunk khởi động, và thời gian khởi động là
 * **Core Value** của dự án này ("đồ thị mở ra trong dưới một giây"). Một người
 * dùng chỉ xem tệp `.rs` không có lý do gì phải tải bộ phân tích Markdown.
 *
 * Wave 1 đã chứng minh cơ chế này hoạt động trên Vite 8: `import()` động trong
 * `App.tsx` cho `SpikeHarness` sinh một chunk riêng (`diffSpike-*.js` 253 KB tách
 * khỏi `index-*.js`). Cùng cơ chế, cùng tính chất cần giữ.
 *
 * Hệ quả cho hình dạng của `LANG_LOADERS`: giá trị là **hàm trả promise**, không
 * phải extension đã dựng. Một bảng `{ ts: javascript({...}) }` buộc mọi gói được
 * nạp *và dựng* ngay lúc nhập module này — và mọi test hành vi vẫn xanh, nên có
 * một cổng đọc mã nguồn trong `langLoader.test.ts` neo vào `import(` bên trong
 * thân bảng.
 *
 * # Bảng tra CỐ ĐỊNH, không nội suy (T-03-26)
 *
 * `path` đến từ webview — nó là đường dẫn tệp trong repo, tức chuỗi tuỳ ý do dữ
 * liệu repo điều khiển. `import(\`@codemirror/lang-${ext}\`)` cho chuỗi đó chọn
 * module nào được nạp. Bảng tra khoá theo phần mở rộng **đã chuẩn hoá** là phép
 * chặn duy nhất đúng: một khoá không có trong bảng trả `null`, không đi thử nạp.
 */

import type { Extension } from '@codemirror/state'

/**
 * Bảng tra phần mở rộng → hàm nạp.
 *
 * Xuất ra (không `const` riêng tư) để test thay một phần tử bằng spy mà đếm được
 * số lần nạp thật — cách duy nhất chứng minh phép nhớ hoạt động mà không dựa vào
 * thời gian.
 *
 * Mỗi giá trị là một hàm gọi `import()`; không dòng nào trong tệp này nhập tĩnh
 * một gói `lang-*`.
 */
export const LANG_LOADERS: Record<string, () => Promise<Extension>> = {
  ts: () => import('@codemirror/lang-javascript').then((m) => m.javascript({ typescript: true })),
  tsx: () =>
    import('@codemirror/lang-javascript').then((m) =>
      m.javascript({ typescript: true, jsx: true }),
    ),
  js: () => import('@codemirror/lang-javascript').then((m) => m.javascript()),
  jsx: () => import('@codemirror/lang-javascript').then((m) => m.javascript({ jsx: true })),
  rs: () => import('@codemirror/lang-rust').then((m) => m.rust()),
  json: () => import('@codemirror/lang-json').then((m) => m.json()),
  css: () => import('@codemirror/lang-css').then((m) => m.css()),
  md: () => import('@codemirror/lang-markdown').then((m) => m.markdown()),
  html: () => import('@codemirror/lang-html').then((m) => m.html()),
}

/**
 * Kết quả đã nạp, khoá theo phần mở rộng.
 *
 * `undefined` không có trong Map nghĩa là "chưa thử"; một mục mang `null` nghĩa
 * là "đã thử và không có ngôn ngữ nào" — phân biệt hai ca đó để không tra bảng
 * lại mỗi lần người dùng bấm một tệp `.lock`.
 */
const cache = new Map<string, Extension | null>()

/** Xoá bộ nhớ đệm — **chỉ** dùng trong test. */
export function __resetLangCacheForTest(): void {
  cache.clear()
}

/**
 * Phần mở rộng đã chuẩn hoá của một đường dẫn, hay `null` khi không có.
 *
 * Bốn ca phải xử lý đúng, và cả bốn đều có test:
 *
 * 1. **Không dấu chấm nào** (`Makefile`, `LICENSE`). `lastIndexOf('.')` trả `-1`
 *    và `slice(-1 + 1)` = `slice(0)` trả **cả đường dẫn** — một khoá tra sai mà
 *    không ném, nên lỗi này chạy im lặng. Phải kiểm trước.
 * 2. **Dot-file** (`.gitignore`, `.env`). Dấu chấm ở vị trí đầu **tên tệp** là
 *    tiền tố, không phải ranh giới phần mở rộng.
 * 3. **Dấu chấm chỉ có trong thư mục** (`a.b/c`). `lastIndexOf` trên cả đường dẫn
 *    thấy dấu chấm ở thư mục và lấy `b/c` làm phần mở rộng. Phải cắt tên tệp trước.
 * 4. **Nhiều dấu chấm** (`a.test.ts`, `.eslintrc.json`) → phần mở rộng **cuối**.
 */
export function extensionOf(filePath: string): string | null {
  // Cắt lấy tên tệp trước: `/` và `\` đều là ngăn cách (đường dẫn Windows vào
  // được qua IPC dù 03-02 chuẩn hoá — phòng thủ hai lớp, không tốn gì).
  const slash = Math.max(filePath.lastIndexOf('/'), filePath.lastIndexOf('\\'))
  const fileName = slash === -1 ? filePath : filePath.slice(slash + 1)

  const dot = fileName.lastIndexOf('.')
  // `dot <= 0` gộp cả ca "không có dấu chấm" (-1) và ca dot-file (0).
  if (dot <= 0) return null

  const ext = fileName.slice(dot + 1).toLowerCase()
  return ext.length > 0 ? ext : null
}

/**
 * Extension CodeMirror cho tệp này, hay `null` khi không nhận ra ngôn ngữ.
 *
 * **Không ném.** Một phần mở rộng lạ là ca hoàn toàn bình thường (`.lock`,
 * `.toml`, `.snap`, tệp không phần mở rộng) và người dùng phải thấy nội dung
 * không tô màu, không thấy panel trắng.
 */
export async function loadLanguage(filePath: string): Promise<Extension | null> {
  const ext = extensionOf(filePath)
  if (ext === null) return null

  if (cache.has(ext)) return cache.get(ext) ?? null

  const loader = LANG_LOADERS[ext]
  if (!loader) {
    cache.set(ext, null)
    return null
  }

  try {
    const extension = await loader()
    cache.set(ext, extension)
    return extension
  } catch {
    // Chunk nạp thất bại (mạng, tệp chunk thiếu sau một bản dựng lỗi). Suy giảm
    // về "không tô màu" chứ không để `DiffViewer` nhận một promise bị reject và
    // trắng cả panel.
    cache.set(ext, null)
    return null
  }
}
