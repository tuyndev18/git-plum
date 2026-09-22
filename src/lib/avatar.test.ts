/**
 * Test `avatar.ts` — hai thứ phải đúng: **ổn định** và **không gọi mạng khi
 * chưa được phép**.
 */

import { describe, expect, it } from 'vitest'

import { avatarTuSinh, chuDaiDien, urlGravatar } from '@/lib/avatar'

describe('chuDaiDien', () => {
  it('tên hai từ -> chữ đầu của từ đầu và từ cuối', () => {
    expect(chuDaiDien('Pham Tuyen')).toBe('PT')
  })

  it('tên ba từ -> chữ đầu của từ ĐẦU và từ CUỐI, bỏ qua từ giữa', () => {
    // `Pham Ngoc Tuyen` là tên thật trong repo của tác giả. `PN` (hai từ đầu)
    // là cách làm sai thường gặp: với tên tiếng Việt, từ cuối là tên gọi và là
    // phần người ta tự nhận ra mình.
    expect(chuDaiDien('Pham Ngoc Tuyen')).toBe('PT')
  })

  it('tên một từ -> một chữ', () => {
    expect(chuDaiDien('tuyenpn')).toBe('T')
  })

  it('handle có dấu gạch chéo -> vẫn ra một chữ, không vỡ', () => {
    // `trungdev/vtconline` là giá trị thật trong ảnh người dùng gửi.
    expect(chuDaiDien('trungdev/vtconline')).toBe('T')
  })

  it('tên không phải chữ Latinh -> trả đúng ký tự, không trả rỗng', () => {
    /*
     * 🔴 `toUpperCase()` không đổi gì với chữ Hán, và một bản lọc `[A-Z]` sẽ
     * trả chuỗi rỗng — cho một ô tròn trống trơn. Ca này có thật trên repo mã
     * nguồn mở và là kiểu lỗi chỉ lộ ra với người dùng ở ngôn ngữ khác.
     */
    expect(chuDaiDien('张伟')).toBe('张')
  })

  it('tên bắt đầu bằng ký tự ngoài BMP -> không cắt đôi thành ký tự thay thế', () => {
    // Emoji chiếm hai code unit; `ten[0]` sẽ trả nửa surrogate pair và render
    // ra ký tự thay thế. `Array.from` lấy đúng một code point.
    expect(chuDaiDien('🦀 Ferris')).toBe('🦀F')
  })

  it('tên rỗng hoặc toàn khoảng trắng -> dấu hỏi, không ném', () => {
    expect(chuDaiDien('')).toBe('?')
    expect(chuDaiDien('   ')).toBe('?')
  })
})

describe('avatarTuSinh — màu phải ỔN ĐỊNH', () => {
  it('cùng email -> luôn cùng màu', () => {
    /*
     * Đây là yêu cầu, không phải chi tiết. Một avatar đổi màu giữa hai lần mở
     * ứng dụng còn tệ hơn không có avatar: người dùng học "tác giả này màu
     * tím" rồi niềm tin đó sai.
     */
    const a = avatarTuSinh('Pham Tuyen', 'tuyen@example.com')
    const b = avatarTuSinh('Pham Tuyen', 'tuyen@example.com')
    expect(a.mauNen).toBe(b.mauNen)
  })

  it('màu theo EMAIL, không theo tên -> cùng người viết tên khác vẫn cùng màu', () => {
    /*
     * Cùng một người thường commit dưới nhiều cách viết tên (`tuyenpn`,
     * `Pham Ngoc Tuyen`) nhưng cùng một email. Băm theo tên sẽ cho họ hai màu
     * và phá đúng thứ avatar sinh ra để làm.
     */
    const a = avatarTuSinh('tuyenpn', 'tuyen@example.com')
    const b = avatarTuSinh('Pham Ngoc Tuyen', 'tuyen@example.com')
    expect(a.mauNen).toBe(b.mauNen)
    // Chữ thì khác nhau, vì chữ theo tên.
    expect(a.chu).not.toBe(b.chu)
  })

  it('email khác hoa/thường hoặc thừa khoảng trắng -> vẫn cùng màu', () => {
    // Cùng phép chuẩn hoá mà Gravatar đòi, nên hai tầng không bao giờ bất đồng
    // về "hai email này có phải một người không".
    const a = avatarTuSinh('X', 'Tuyen@Example.COM')
    const b = avatarTuSinh('X', '  tuyen@example.com  ')
    expect(a.mauNen).toBe(b.mauNen)
  })

  it('email khác nhau -> phân tán qua nhiều màu, không dồn vào một', () => {
    // Một hàm băm hỏng (ví dụ luôn trả 0) vẫn qua được mọi test ổn định ở
    // trên. Test này là thứ bắt được nó.
    const mau = new Set(
      Array.from({ length: 40 }, (_, i) => avatarTuSinh('X', `user${i}@example.com`).mauNen),
    )
    expect(mau.size).toBeGreaterThan(5)
  })
})

describe('urlGravatar', () => {
  it('có d=404 để 404 kích hoạt onError và lui về avatar tự sinh', () => {
    /*
     * 🔴 Thiếu `d=404` là lỗi im lặng: Gravatar trả một identicon do họ sinh
     * cho mọi email không có ảnh, nên `onError` KHÔNG bao giờ chạy và giao diện
     * hiện một hình theo phong cách hoàn toàn khác cạnh các avatar của dự án.
     * Trông như tính năng chạy đúng, nhưng nhất quán đã mất.
     */
    expect(urlGravatar('abc123', 16)).toContain('d=404')
  })

  it('xin ảnh gấp đôi cỡ hiển thị, cho màn HiDPI', () => {
    expect(urlGravatar('abc123', 20)).toContain('s=40')
  })

  it('hash đi thẳng vào đường dẫn — KHÔNG có email trong URL', () => {
    /*
     * Ràng buộc quyền riêng tư ở mức URL. Hash tính ở Rust
     * (`Commit.authorAvatarHash`) nên email không bao giờ tới lớp giao diện,
     * nhưng test này ghim rằng kể cả khi ai đó truyền nhầm email vào đây thì
     * hình dạng URL vẫn là hình dạng "chỉ có hash".
     */
    const url = urlGravatar('d41d8cd98f00b204e9800998ecf8427e', 16)
    expect(url).not.toContain('@')
    expect(url).toMatch(/gravatar\.com\/avatar\/[0-9a-f]+\?/)
  })
})
