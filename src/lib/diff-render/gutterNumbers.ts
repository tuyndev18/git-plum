/**
 * Số dòng **thật trong tệp** cho gutter của trình xem diff — hàm thuần.
 *
 * # Vì sao tệp này tồn tại (lỗi người dùng báo ở checkpoint vòng 2)
 *
 * Bản trước dùng `lineNumbers()` **mặc định** của CodeMirror ở cả hai phía. Mặc
 * định đó đếm 1, 2, 3… theo **tài liệu của từng editor**, và trong chế độ hai
 * cột hai editor là **hai tài liệu khác nhau** (phía cũ bỏ dòng `added`, phía
 * mới bỏ dòng `removed`). Hệ quả đo được bằng Chromium thật trên một hunk có
 * hai dòng thêm:
 *
 * | hàng | gutter trái (trước) | gutter phải (trước) | trái (đúng) | phải (đúng) |
 * |---|---|---|---|---|
 * | 1-3 | 1,2,3 | 1,2,3 | 1,2,3 | 1,2,3 |
 * | 4-5 (chỉ phía mới) | *(khoảng trống)* | 4,5 | *(trống)* | 4,5 |
 * | 6-8 | 4,5,6 | 6,7,8 | **4,5,6** | **6,7,8** |
 *
 * Nhìn bảng thì cột "trước" và "đúng" trùng nhau ở phía trái — đó là vì
 * `@codemirror/merge` **đã** chèn widget `.cm-mergeSpacer` để căn hàng, nên hai
 * phía vẫn thẳng theo nội dung. Điều **sai** là con số không phải số dòng thật:
 * `lineNumbers()` đếm theo tài liệu đã lọc, nên một tệp có N dòng xoá ở đầu sẽ
 * hiện số lệch N ở **mọi** dòng phía sau. Người dùng đối chiếu số đó với editor
 * của họ và nó không khớp.
 *
 * Sửa đúng là đọc **`oldLine`/`newLine`** từ `lineMeta` — hai trường mà 03-02
 * tách ra chính cho mục đích này (`ipc.ts`: *"hai trường riêng vì chế độ hai cột
 * (DIFF-02) dựng bố cục từ đúng hai số này"*). Renderer trước đó **chưa dùng**
 * chúng. Không lệnh git nào phải thêm; hợp đồng IPC không đổi.
 *
 * # Vì sao là hàm thuần tách riêng, không phải đóng gói trong renderer
 *
 * Cùng lý do `shouldForceUnified` và `buildDecorations` là hàm thuần:
 * `MergeView` **chưa bao giờ được render trong một test happy-dom nào** (không
 * có `ResizeObserver` → `paneWidth = 0` → mọi test chạy nhánh hợp nhất). Logic
 * nằm trong renderer là logic chỉ kiểm được bằng mắt người. Ở đây nó kiểm được
 * thật, và mutation "quay về `lineNumbers()` mặc định" làm test đỏ.
 */

import type { LineMeta } from './decorations'

/** Phía gutter cần đánh số. */
export type GutterSide = 'old' | 'new'

/**
 * Số dòng thật để hiện ở gutter của hàng thứ `lineNumber` (đếm từ 1).
 *
 * Trả **chuỗi rỗng** khi phía này không có dòng ở vị trí đó — đó là điều làm hai
 * cột đọc được: một con số ở chỗ không có dòng nói sai, còn chỗ trống nói đúng
 * ("bên này không có gì ở đây").
 *
 * `lineNumber` vượt `lineMeta` cũng trả rỗng. Ca đó xảy ra thật: CodeMirror gọi
 * `formatNumber` cho dòng cuối của tài liệu rỗng, và một `undefined` lọt ra sẽ
 * hiện chữ `"undefined"` trong gutter.
 */
export function soDongThat(lineMeta: LineMeta[], side: GutterSide, lineNumber: number): string {
  const meta = lineMeta[lineNumber - 1]
  if (!meta) return ''
  const n = side === 'old' ? meta.oldLine : meta.newLine
  return n === null ? '' : String(n)
}

/**
 * Dấu `+` / `−` cho hàng thứ `lineNumber`, hay chuỗi rỗng với dòng ngữ cảnh.
 *
 * Vì sao đáng làm: phân biệt thêm/xoá **không chỉ bằng màu**. Nền dòng của dự án
 * là `color-mix(... 16%, transparent)` — rất nhạt có chủ ý, để nền word-level
 * đậm hơn còn nổi lên được. Người mù màu đỏ-lục không phân biệt được hai nền
 * nhạt đó, và đây là công cụ mã nguồn mở nên ca đó là ca thật.
 *
 * Dùng `−` (U+2212 MINUS SIGN), không `-` (hyphen-minus): trong phông mono dấu
 * trừ thật cân bằng thị giác với `+`, còn hyphen ngắn và lệch xuống.
 */
export function dauThemXoa(lineMeta: LineMeta[], lineNumber: number): string {
  const meta = lineMeta[lineNumber - 1]
  if (!meta) return ''
  if (meta.kind === 'added') return '+'
  if (meta.kind === 'removed') return '−'
  return ''
}
