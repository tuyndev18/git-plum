/**
 * Cổng 11 của `<verification>` plan 03-04: gói `@codemirror/lang-*` phải nằm ở
 * **chunk riêng**, không gộp vào chunk vào (entry).
 *
 * # 🔴 Assertion của plan SAI — ghi rõ và sửa
 *
 * `<done>` Task 2 viết: *"khẳng định ≥2 tệp js mà tên chứa `lang` hoặc
 * `javascript`/`rust`"*. Đo thật trên Vite 8.3.0 cho thấy phép kiểm đó
 * **không thực hiện được**: Vite 8 đặt tên chunk theo **tên tệp entry của gói**,
 * và mọi gói `@codemirror/lang-*` có entry là `dist/index.js` — nên bảy chunk
 * lang đều tên `dist-<băm>.js`. Không tệp nào chứa chuỗi `lang`, `javascript`
 * hay `rust` trong **tên**.
 *
 * ```
 * dist/assets/dist-3aoZtzzH.js    92.88 kB   ← lang-javascript
 * dist/assets/dist-DS9PrsoY.js    83.68 kB   ← lang-rust
 * dist/assets/dist-BTn_YdOr.js    43.85 kB   ← lang-markdown + lang-html
 * ```
 *
 * Plan nói rõ phải làm gì trong ca này: *"không đổi cổng thành một phép kiểm dễ
 * hơn mà vô nghĩa."* Nên cổng dưới đây kiểm **đúng bất biến mà phase cần**, chỉ
 * bằng một phép đo khác:
 *
 * 1. Chunk **entry** (`index-*.js`) **không** chứa dấu hiệu nào của các bộ phân
 *    tích ngôn ngữ. Đây là điều thật sự quan trọng — Core Value là thời gian
 *    khởi động, và nó bị hại đúng khi mã lang nằm trong chunk entry.
 * 2. Có **≥4** chunk không-entry mang dấu hiệu ngôn ngữ, tức việc tách chunk có
 *    thật sự xảy ra (không phải "0 chunk lang vì không ai nhập chúng cả" — một
 *    ca cũng làm điều kiện 1 xanh, nên nó phải được loại riêng).
 *
 * # Chọn dấu hiệu — BA lần thử, ghi cả ba
 *
 * Phần này dài vì nó là bài học: cổng đầu tiên và cổng thứ hai **đều đo sai đại
 * lượng**, đúng lớp lỗi mà 03-02 gặp hai lần ở phía Rust.
 *
 * **Lần một (sai, cho 2/6):** tên định danh nội bộ (`javascriptLanguage`,
 * `rustLanguage`, …). Bộ rút gọn của Vite 8 **đổi tên** phần lớn các định danh
 * đó, nên phép đếm nói "2 chunk" trong khi năm chunk tồn tại thật.
 *
 * **Lần hai (sai, cho 0/6):** `LRParser.deserialize(`. Cũng bị rút gọn: trong
 * chunk thật nó là `z=l.deserialize(` — tên nhập `LRParser` thành `l`. Lần đo
 * đầu tưởng nó có vì phép probe dùng `OR` với `deserialize(`, tức probe đo một
 * chuỗi khác chuỗi mà cổng dùng.
 *
 * **Lần ba (dùng):** `stateData:`, `nodeNames:`, `tokenPrec:` — **khoá của object
 * literal** trong bảng parser đã serialize. Khoá object literal **không** bị rút
 * gọn (đổi chúng sẽ phá chính `LRParser.deserialize`). Đòi **cả ba** cùng có để
 * một chuỗi trùng ngẫu nhiên không làm cổng xanh. Đo thật: 5/8 chunk không-entry
 * có cả ba; chunk entry (619 KB) **không** có khoá nào.
 *
 * Cố ý **không** dùng các từ chung như `rust` hay `json`: hai từ đó có trong
 * chunk entry vì lý do khác (khoá phần mở rộng trong bảng tra của
 * `langLoader.ts`, chuỗi giao diện), và một cổng dùng chúng sẽ **đỏ vì lý do
 * sai** — tệ hơn một cổng luôn xanh, vì nó dạy người sau bỏ qua cổng.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs'
import path from 'node:path'

const assetsDir = path.join(process.cwd(), 'dist', 'assets')

/**
 * Khoá object literal trong bảng parser đã serialize của mọi gói
 * `@codemirror/lang-*`. Khoá object literal **không** bị bộ rút gọn đổi tên —
 * xem "Chọn dấu hiệu" ở doc comment đầu tệp cho hai dấu hiệu đã thử và bỏ.
 *
 * Đòi **cả ba** cùng có: một chuỗi trùng ngẫu nhiên không làm cổng xanh.
 */
const DAU_HIEU_PARSER = ['stateData:', 'nodeNames:', 'tokenPrec:']

let files
try {
  files = readdirSync(assetsDir).filter((f) => f.endsWith('.js'))
} catch {
  console.error(`LỖI: không đọc được ${assetsDir} — chạy \`npm run build\` trước.`)
  process.exit(1)
}

const entry = files.filter((f) => f.startsWith('index-'))
if (entry.length !== 1) {
  console.error(`LỖI: phải có ĐÚNG một chunk entry \`index-*.js\`, thấy ${entry.length}.`)
  process.exit(1)
}

const doc = (f) => readFileSync(path.join(assetsDir, f), 'utf8')
const coParser = (src) => DAU_HIEU_PARSER.every((k) => src.includes(k))

// --- Điều kiện 1: chunk entry SẠCH mã ngôn ngữ ---
const entryFile = entry[0]
if (coParser(doc(entryFile))) {
  console.error(
    `LỖI: chunk entry ${entryFile} chứa \`${DAU_HIEU_PARSER}\` — mã bộ phân tích ngôn ngữ.\n` +
      `Gói lang-* đã vào bundle khởi động — nạp lười bị phá, và thời gian khởi động\n` +
      `là Core Value của dự án.`,
  )
  process.exit(1)
}

// --- Điều kiện 2: việc tách chunk THẬT SỰ xảy ra ---
//
// Tên trường là `laLang`, KHÔNG `coParser`: trùng tên với hàm `coParser` làm
// `.filter((c) => c.coParser)` đọc một hàm luôn truthy ở lần viết đầu, và cổng
// cho **0 chunk** trong khi sáu chunk tồn tại thật. Ghi lại vì nó là đúng lớp
// "cổng đo sai đại lượng" mà tệp này đang nói về.
const chunkNgonNgu = files
  .filter((f) => f !== entryFile)
  .map((f) => ({ f, laLang: coParser(doc(f)), bytes: statSync(path.join(assetsDir, f)).size }))
  .filter((c) => c.laLang)

if (chunkNgonNgu.length < 4) {
  console.error(
    `LỖI: chỉ ${chunkNgonNgu.length} chunk mang mã ngôn ngữ (cần ≥4).\n` +
      `Điều kiện 1 xanh cũng có thể chỉ vì KHÔNG AI nhập gói lang-* cả — cổng này\n` +
      `loại ca đó ra.`,
  )
  process.exit(1)
}

console.log(`OK — chunk entry ${entryFile} sạch mã ngôn ngữ.`)
console.log(`${chunkNgonNgu.length} chunk ngôn ngữ tách riêng:`)
for (const c of chunkNgonNgu.sort((a, b) => b.bytes - a.bytes)) {
  console.log(`  ${c.f.padEnd(28)} ${String(c.bytes).padStart(7)} B`)
}
