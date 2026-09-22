/**
 * `langLoader` — nạp **lười** chế độ ngôn ngữ CodeMirror theo phần mở rộng.
 *
 * # Vì sao có một test đọc mã nguồn trong tệp này
 *
 * Mọi test "khớp đúng ngôn ngữ" dưới đây **xanh như nhau** với hai cài đặt khác
 * nhau về bản chất:
 *
 * ```ts
 * import { javascript } from '@codemirror/lang-javascript'   // ❌ tĩnh
 * const BANG = { ts: () => javascript({ typescript: true }) }
 * ```
 *
 * ```ts
 * const BANG = { ts: () => import('@codemirror/lang-javascript').then(...) }  // ✅ lười
 * ```
 *
 * Bản tĩnh kéo **cả sáu** gói `lang-*` vào bundle khởi động — hại thời gian khởi
 * động, tức hại Core Value — mà **không đổi một hành vi nào** quan sát được qua
 * test hành vi. Đây đúng lớp "cổng tự vô hiệu hoá" mà Phase 2 và 03-02 đã gặp,
 * nên phải có một cổng đọc **mã nguồn** neo vào `import(` ở trong bảng tra.
 *
 * Mutation #1 của Task 1 là đổi sang `import` tĩnh; test
 * `bang_tra_dung_import_dong` là cổng của nó.
 */

/// <reference types="node" />
//
// Như `app.css.test.ts`: chỉ tệp test này cần kiểu Node (`readFileSync`). Mã
// sản phẩm là browser-only (không `tauri-plugin-fs`) nên `"node"` không vào
// mảng `types` toàn dự án.
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  LANG_LOADERS,
  __resetLangCacheForTest,
  extensionOf,
  loadLanguage,
} from './langLoader'

beforeEach(() => {
  __resetLangCacheForTest()
})

describe('extensionOf — lấy phần mở rộng cuối, chuẩn hoá chữ thường', () => {
  it('đường dẫn thường', () => {
    expect(extensionOf('src/lib/ipc.ts')).toBe('ts')
    expect(extensionOf('src-tauri/src/main.rs')).toBe('rs')
  })

  it('hoa/thường lẫn → khớp như chữ thường', () => {
    expect(extensionOf('A.TS')).toBe('ts')
    expect(extensionOf('main.Rs')).toBe('rs')
  })

  it('nhiều dấu chấm → lấy phần mở rộng CUỐI', () => {
    expect(extensionOf('src/a.test.ts')).toBe('ts')
    expect(extensionOf('.eslintrc.json')).toBe('json')
  })

  it('🔴 KHÔNG có dấu chấm nào → null, không phải cả đường dẫn', () => {
    // `lastIndexOf('.')` trả `-1`, và `slice(-1 + 1)` = `slice(0)` trả **cả
    // đường dẫn** — một khoá tra sai nhưng không ném, nên lỗi chạy im lặng.
    expect(extensionOf('Makefile')).toBeNull()
    expect(extensionOf('LICENSE')).toBeNull()
    expect(extensionOf('src/scripts/build')).toBeNull()
  })

  it('tệp dot-file không phần mở rộng thật → null', () => {
    // `.gitignore` — dấu chấm ở vị trí 0 là tiền tố tên tệp, không phải ranh
    // giới phần mở rộng.
    expect(extensionOf('.gitignore')).toBeNull()
    expect(extensionOf('src/.env')).toBeNull()
  })

  it('dấu chấm nằm trong thư mục nhưng không trong tên tệp → null', () => {
    // `a.b/c` — `lastIndexOf('.')` thấy dấu chấm ở THƯ MỤC. Lấy `b/c` làm phần
    // mở rộng là sai.
    expect(extensionOf('a.b/c')).toBeNull()
  })

  it('tệp kết thúc bằng dấu chấm → null (phần mở rộng rỗng)', () => {
    expect(extensionOf('weird.')).toBeNull()
  })
})

describe('loadLanguage — bảng tra cố định, không nội suy (T-03-26)', () => {
  it('nhận đúng ngôn ngữ cho .ts/.tsx/.js/.jsx → javascript', async () => {
    for (const p of ['a.ts', 'a.tsx', 'a.js', 'a.jsx']) {
      const ext = await loadLanguage(p)
      expect(ext, `${p} phải nạp được extension`).not.toBeNull()
    }
  })

  it('nhận đúng ngôn ngữ cho .rs/.json/.css/.md/.html', async () => {
    for (const p of ['a.rs', 'a.json', 'a.css', 'a.md', 'a.html']) {
      const ext = await loadLanguage(p)
      expect(ext, `${p} phải nạp được extension`).not.toBeNull()
    }
  })

  it('🔴 phần mở rộng KHÔNG biết → null, KHÔNG ném', async () => {
    await expect(loadLanguage('a.xyzzy')).resolves.toBeNull()
    await expect(loadLanguage('a.zip')).resolves.toBeNull()
  })

  it('🔴 tệp không phần mở rộng (Makefile, LICENSE) → null, KHÔNG ném', async () => {
    await expect(loadLanguage('Makefile')).resolves.toBeNull()
    await expect(loadLanguage('LICENSE')).resolves.toBeNull()
  })

  it('hoa/thường lẫn vẫn nạp đúng', async () => {
    await expect(loadLanguage('A.TS')).resolves.not.toBeNull()
    await expect(loadLanguage('Main.Rs')).resolves.not.toBeNull()
  })

  it('🔴 gọi HAI lần cùng phần mở rộng → hàm nạp chạy ĐÚNG MỘT lần', async () => {
    // Cổng của yêu cầu "nhớ kết quả trong Map". Thay hàm nạp trong bảng bằng
    // một spy để đếm số lần thật, chứ không suy ra từ thời gian.
    const goc = LANG_LOADERS['ts']!
    const spy = vi.fn(goc)
    LANG_LOADERS['ts'] = spy
    try {
      const a = await loadLanguage('x.ts')
      const b = await loadLanguage('y.tsx')
      const c = await loadLanguage('z.ts')

      expect(spy).toHaveBeenCalledTimes(1)
      // Và cùng một extension được trả lại, không dựng mới.
      expect(a).toBe(c)
      expect(b).not.toBeNull()
    } finally {
      LANG_LOADERS['ts'] = goc
    }
  })

  it('phần mở rộng không biết cũng được nhớ — không tra lại bảng mỗi lần', async () => {
    // Không có gì để đếm, nhưng khẳng định hành vi ổn định: hai lần gọi cho
    // cùng kết quả `null` và không ném.
    expect(await loadLanguage('a.xyzzy')).toBeNull()
    expect(await loadLanguage('a.xyzzy')).toBeNull()
  })

  it('bảng tra là hằng, khoá là phần mở rộng đã chuẩn hoá chữ thường', () => {
    for (const key of Object.keys(LANG_LOADERS)) {
      expect(key, `khoá bảng tra phải chữ thường: ${key}`).toBe(key.toLowerCase())
      expect(key, 'khoá không được chứa dấu chấm').not.toContain('.')
    }
    // Bảy phần mở rộng theo `<behavior>`: ts, tsx, js, jsx, rs, json, css, md, html.
    expect(Object.keys(LANG_LOADERS).sort()).toEqual(
      ['css', 'html', 'js', 'json', 'jsx', 'md', 'rs', 'ts', 'tsx'].sort(),
    )
  })

  it('giá trị bảng tra là HÀM, không phải module đã nạp', () => {
    // Một bảng `{ ts: javascript({...}) }` (giá trị là extension đã dựng) buộc
    // mọi lang-* được nạp và dựng ngay lúc nhập module.
    for (const [key, value] of Object.entries(LANG_LOADERS)) {
      expect(typeof value, `LANG_LOADERS['${key}'] phải là hàm`).toBe('function')
    }
  })
})

/*
 * Cổng đọc mã nguồn — mutation #1 của Task 1.
 *
 * Neo vào `import(` **bên trong thân của bảng tra** (không phải ở đâu đó trong
 * tệp), theo bài học 03-02: cổng phải kiểm trong phạm vi của **chính khối** mang
 * bất biến, không trên cả tệp. Và lọc chú thích trước khi tìm, vì doc comment ở
 * đầu tệp nguồn CÓ nhắc cả `import` tĩnh lẫn `import()` động trong phần giải
 * thích — một cổng đọc CSS/TS thô sẽ khớp nhầm chính đoạn văn đó (đúng lỗi
 * `.graph-canvas` của `app.css.test.ts`).
 */
describe('langLoader.ts — bảng tra PHẢI dùng import() động', () => {
  const src = readFileSync(
    path.join(process.cwd(), 'src', 'lib', 'diff-render', 'langLoader.ts'),
    'utf8',
  )
  /** Nguồn đã bỏ mọi chú thích khối và chú thích dòng. */
  const ma = src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/^[ \t]*\/\/.*$/gm, '')

  it('tiền đề: phép lọc chú thích không ăn mất bảng tra', () => {
    // Nếu phép lọc ăn mất mã thì mọi khẳng định dưới đây thành luôn-xanh.
    expect(ma, 'phải còn khai báo LANG_LOADERS sau khi lọc chú thích').toContain('LANG_LOADERS')
  })

  it('🔴 KHÔNG có `import ... from "@codemirror/lang-*"` ở cấp module', () => {
    // Đây là mutation #1: một dòng nhập tĩnh làm mọi test hành vi ở trên vẫn
    // xanh trong khi ràng buộc bundle của phase bị phá hoàn toàn.
    const nhapTinh = ma.match(/^\s*import\s+[^\n]*from\s+['"]@codemirror\/lang-[^'"]+['"]/gm)
    expect(
      nhapTinh,
      `nhập tĩnh gói lang-* kéo cả sáu gói vào bundle khởi động. Dòng vi phạm:\n` +
        `${(nhapTinh ?? []).join('\n')}`,
    ).toBeNull()
  })

  it('🔴 thân bảng tra LANG_LOADERS chứa import() động cho MỌI khoá', () => {
    const start = ma.indexOf('LANG_LOADERS')
    expect(start, 'phải tìm thấy LANG_LOADERS trong mã đã lọc chú thích').toBeGreaterThan(-1)
    // Cắt từ khai báo tới dấu `}` cuối của object literal.
    const openBrace = ma.indexOf('{', start)
    let depth = 0
    let end = openBrace
    for (let i = openBrace; i < ma.length; i++) {
      if (ma[i] === '{') depth++
      else if (ma[i] === '}') {
        depth--
        if (depth === 0) {
          end = i
          break
        }
      }
    }
    const than = ma.slice(openBrace, end + 1)

    const soImportDong = (than.match(/import\(/g) ?? []).length
    expect(
      soImportDong,
      `thân LANG_LOADERS phải có một \`import(\` cho mỗi khoá (${Object.keys(LANG_LOADERS).length} khoá), ` +
        `đếm được ${soImportDong}. Thân thật:\n${than}`,
    ).toBe(Object.keys(LANG_LOADERS).length)
  })

  it('🔴 KHÔNG nội suy đường dẫn module từ phần mở rộng (T-03-26)', () => {
    // `import(\`@codemirror/lang-${ext}\`)` cho `path` đến từ webview điều khiển
    // đường dẫn module được nạp. Bảng tra cố định là phép chặn duy nhất đúng.
    expect(ma).not.toMatch(/import\(\s*[`'"]@codemirror\/lang-\$\{/)
    expect(ma, 'không được nối chuỗi vào đường dẫn module').not.toMatch(
      /import\(\s*['"`]@codemirror\/lang-['"`]\s*\+/,
    )
  })
})
