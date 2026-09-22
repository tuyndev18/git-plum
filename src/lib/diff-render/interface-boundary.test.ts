/**
 * Cổng biên giới: `@codemirror/merge` **không** được rò ra ngoài
 * `codemirrorRenderer.ts`.
 *
 * # Vì sao cổng này tồn tại
 *
 * Checkpoint #3 của plan 03-01 **bị bỏ qua** (`docs/09-phase3-diff-decision.md`
 * mục 7, commit `6910732`). Đường A (`@codemirror/merge`) được chọn **vì nó là
 * mặc định khi thiếu bằng chứng**, không vì đã chứng minh đủ nhanh trên tệp
 * 630 KB thật. Mục 8 của tài liệu đó nêu điều kiện:
 *
 * > *"03-04 phải giữ trình xem sau một interface để việc đổi đường là sửa một
 * > tệp, không phải sửa cả giao diện."*
 *
 * Một interface mà không ai kiểm sẽ bị rò qua trong vòng vài plan — đó là lý do
 * Phase 2 có cổng `grep getContext GraphCanvas.tsx` → 0 cho `GraphRenderer`. Đây
 * là cổng tương đương, ở dạng test thật thay vì grep.
 *
 * # Vì sao là test đọc mã nguồn, không phải grep
 *
 * `<verification>` của plan nêu rõ: mọi cổng `grep -c ... == 0` trên tệp có doc
 * comment tiếng Việt là **cổng tự vô hiệu hoá**, vì tệp trong plan này có comment
 * dày dẫn chiếu tên API. `DiffViewer.tsx` **nhắc** `@codemirror/merge` trong doc
 * comment giải thích quyết định A/B — nên cổng phải lọc chú thích trước khi tìm,
 * và phải kiểm **dòng `import`** chứ không phải sự xuất hiện của chuỗi.
 */

/// <reference types="node" />
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'

/**
 * Tệp được phép nhập trực tiếp từ CodeMirror, và **vì sao từng tệp**.
 *
 * Danh sách này là hợp đồng: thêm một tệp vào đây là một quyết định kiến trúc,
 * không phải một bước sửa cho test xanh.
 */
const DUOC_PHEP: Record<string, string> = {
  'src/lib/diff-render/codemirrorRenderer.ts':
    'cài đặt đường A — tệp DUY NHẤT được nhập @codemirror/merge',
  'src/lib/diff-render/decorations.ts':
    'dựng DecorationSet; chỉ nhập /view và /state, KHÔNG /merge',
  'src/lib/diff-render/theme.ts': 'EditorView.theme; chỉ nhập /view',
  'src/lib/diff-render/langLoader.ts':
    'chỉ `import type { Extension }` từ /state — biên KIỂU, không mã chạy. ' +
    '`Extension` là kiểu trả về của bảng tra, và một kiểu `unknown` ở đây sẽ ' +
    'chuyển việc kiểm sang lúc chạy.',
  'src/lib/diff-render/types.ts':
    'cũng chỉ `import type { Extension }` — chính tệp khai interface, nên nó phải ' +
    'nói được kiểu của extension mà bộ dựng nhận.',
  'src/lib/diffSpike.ts': 'spike của 03-01, xoá cùng SpikeHarness sau wave 5',
  'src/components/diff/SpikeHarness.tsx': 'giao diện của spike, cùng lý do',
}

/** Mọi tệp `.ts`/`.tsx` trong `src`, trừ tệp test. */
function moiTepNguon(dir: string, acc: string[] = []): string[] {
  const { readdirSync, statSync } = require('node:fs') as typeof import('node:fs')
  for (const name of readdirSync(dir)) {
    const full = path.join(dir, name)
    if (statSync(full).isDirectory()) {
      moiTepNguon(full, acc)
    } else if (/\.tsx?$/.test(name) && !/\.test\.tsx?$/.test(name)) {
      acc.push(path.relative(process.cwd(), full).split(path.sep).join('/'))
    }
  }
  return acc
}

const tepNguon = moiTepNguon(path.join(process.cwd(), 'src'))

/** Nguồn đã bỏ mọi chú thích — doc comment nhắc tên API là ca thật ở đây. */
function maKhongChuThich(file: string): string {
  return readFileSync(path.join(process.cwd(), file), 'utf8')
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/^[ \t]*\/\/.*$/gm, '')
}

describe('biên giới bộ dựng diff — @codemirror không rò ra ngoài', () => {
  it('tiền đề: tìm thấy đủ tệp nguồn và cả năm tệp được phép đều tồn tại', () => {
    // Nếu phép quét hỏng thì mọi khẳng định dưới đây thành luôn-xanh.
    expect(tepNguon.length).toBeGreaterThan(20)
    for (const f of Object.keys(DUOC_PHEP)) {
      expect(tepNguon, `tệp được phép ${f} phải có trong phép quét`).toContain(f)
    }
  })

  it('🔴 CHỈ codemirrorRenderer.ts nhập @codemirror/merge', () => {
    const viPham = tepNguon.filter(
      (f) =>
        f !== 'src/lib/diff-render/codemirrorRenderer.ts' &&
        !f.includes('diffSpike') &&
        !f.includes('SpikeHarness') &&
        /^\s*import\s[^\n]*from\s+['"]@codemirror\/merge['"]/m.test(maKhongChuThich(f)),
    )
    expect(
      viPham,
      `@codemirror/merge phải nằm trong ĐÚNG MỘT tệp — đó là điều làm việc đổi\n` +
        `đường A → B thành "sửa một tệp" (docs/09-phase3-diff-decision.md mục 8).\n` +
        `Tệp vi phạm: ${viPham.join(', ')}`,
    ).toEqual([])
  })

  it('🔴 DiffViewer.tsx KHÔNG nhập gì từ @codemirror', () => {
    const ma = maKhongChuThich('src/components/diff/DiffViewer.tsx')
    const nhapCm = ma.match(/^\s*import\s[^\n]*from\s+['"]@codemirror\/[^'"]+['"]/gm)
    expect(
      nhapCm,
      `DiffViewer là tầng giao diện: nó chỉ được biết \`DiffRenderer\` (interface),\n` +
        `không biết CodeMirror. Dòng vi phạm:\n${(nhapCm ?? []).join('\n')}`,
    ).toBeNull()
  })

  it('🔴 DiffViewer.tsx nhập ĐÚNG MỘT hàm dựng — đổi đường là đổi một dòng', () => {
    const ma = maKhongChuThich('src/components/diff/DiffViewer.tsx')
    const nhapBoDung = ma.match(/from\s+['"]@\/lib\/diff-render\/\w*[Rr]enderer['"]/g)
    expect(
      nhapBoDung?.length,
      'phải có đúng một dòng nhập bộ dựng — nhiều hơn nghĩa là đổi đường phải sửa ' +
        'nhiều chỗ, tức interface không còn giữ được lời hứa của nó',
    ).toBe(1)
  })

  it('không tệp nào ngoài danh sách được phép nhập bất kỳ gói @codemirror nào', () => {
    const viPham = tepNguon.filter(
      (f) =>
        !(f in DUOC_PHEP) &&
        /^\s*import\s[^\n]*from\s+['"]@codemirror\//m.test(maKhongChuThich(f)),
    )
    expect(
      viPham,
      `Thêm một tệp vào \`DUOC_PHEP\` là quyết định kiến trúc, không phải bước sửa\n` +
        `cho test xanh. Tệp vi phạm: ${viPham.join(', ')}`,
    ).toEqual([])
  })

  /*
   * Danh sách `DUOC_PHEP` một mình là **quá lỏng**: nó cho phép cả nhập kiểu lẫn
   * nhập mã chạy. Hai tệp trong danh sách (`langLoader.ts`, `types.ts`) được thêm
   * vào với lý do "chỉ nhập KIỂU", và test này ghim đúng lời hứa đó — nếu không,
   * lý do trong danh sách chỉ là một câu văn không ai kiểm.
   *
   * `import type` bị xoá hoàn toàn lúc biên dịch nên nó **không** đưa một byte
   * nào của CodeMirror vào bundle — đó là điều làm hai tệp đó vô hại, và là điều
   * phải giữ.
   */
  for (const tep of ['src/lib/diff-render/langLoader.ts', 'src/lib/diff-render/types.ts']) {
    it(`${tep} chỉ nhập KIỂU từ @codemirror, không nhập mã chạy`, () => {
      const ma = maKhongChuThich(tep)
      const nhapCm = ma.match(/^\s*import\s[^\n]*from\s+['"]@codemirror\/[^'"]+['"]/gm) ?? []
      expect(nhapCm.length, 'tiền đề: phải có ít nhất một dòng nhập để kiểm').toBeGreaterThan(0)

      for (const dong of nhapCm) {
        expect(
          dong,
          `${tep} phải dùng \`import type\` — một nhập mã chạy đưa CodeMirror vào\n` +
            `bundle khởi động và phá đúng lý do tệp này nằm trong DUOC_PHEP.\n` +
            `Dòng: ${dong.trim()}`,
        ).toMatch(/^\s*import\s+type\s/)
      }
    })
  }
})
