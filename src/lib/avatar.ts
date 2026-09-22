/**
 * Avatar tác giả — **hai tầng, tầng mặc định không gọi mạng**.
 *
 * # Vì sao tầng mặc định phải là tầng tự sinh
 *
 * Ràng buộc Privacy của dự án (`CLAUDE.md`): "Không gửi gì ra ngoài khi chưa
 * được cho phép rõ ràng." Ràng buộc đó viết cho tính năng AI, nhưng nguyên tắc
 * áp nguyên vào đây: một lần fetch Gravatar gửi hash email của **tác giả
 * commit** tới máy chủ bên thứ ba, kèm theo — qua thời điểm và tần suất — thông
 * tin ai đang đọc repo nào lúc nào. Người dùng không mong đợi một trình xem
 * lịch sử git mở kết nối ra ngoài.
 *
 * Nên:
 *
 * | Tầng | Khi nào | Mạng |
 * |---|---|---|
 * | **Tự sinh** | luôn luôn, là mặc định | **không** |
 * | Gravatar | chỉ khi người dùng bật tường minh | có, và cache |
 *
 * Tầng tự sinh không phải chỗ dựa tạm — nó là thứ hầu hết người dùng sẽ thấy,
 * nên nó phải thật sự dùng được: phân biệt được tác giả ở một cái liếc, ổn định
 * giữa các lần chạy, và đọc được trên cả hai theme.
 *
 * # Ổn định là yêu cầu, không phải chi tiết
 *
 * Cùng một email PHẢI luôn cho cùng một màu. Một avatar đổi màu giữa hai lần mở
 * ứng dụng còn tệ hơn không có avatar: người dùng học "tác giả này màu tím" rồi
 * niềm tin đó sai. Vì vậy màu đến từ một hàm băm xác định của email, không từ
 * chỉ số hàng, không từ thứ tự xuất hiện, không từ `Math.random`.
 */

/**
 * Băm FNV-1a 32-bit của một chuỗi.
 *
 * Cùng thuật toán mà `repo_id_for` dùng ở `src-tauri/src/state/mod.rs` — chọn
 * lại nó thay vì một hàm băm khác để dự án chỉ có **một** kiểu băm không mật mã
 * mà người đọc phải hiểu.
 *
 * Không cần chống va chạm có chủ ý: đây là chỉ số vào một bảng màu 16 phần tử,
 * và hai tác giả trùng màu là chuyện bình thường ở mọi bảng màu hữu hạn.
 */
function bam32(s: string): number {
  let h = 0x811c9dc5
  for (let i = 0; i < s.length; i += 1) {
    h ^= s.charCodeAt(i)
    // `Math.imul` chứ không phải `*`: nhân 32-bit tràn số trong JS sẽ mất chính
    // xác qua ngưỡng 2^53 và cho kết quả khác nhau tuỳ độ dài chuỗi.
    h = Math.imul(h, 0x01000193)
  }
  return h >>> 0
}

/**
 * Bảng màu nền avatar — **tách hẳn khỏi `LANE_COLORS`**.
 *
 * Hai bảng khác nhau là có chủ ý. Màu lane mã hoá *cấu trúc nhánh*; màu avatar
 * mã hoá *danh tính người*. Dùng chung một bảng sẽ khiến một avatar tình cờ
 * trùng màu lane trông như thể nó thuộc về nhánh đó — một quan hệ không có
 * thật, và là loại nhiễu khó nhận ra vì nó *trông* có nghĩa.
 *
 * Độ bão hoà thấp hơn `LANE_COLORS` (55% so với 74%): avatar là ô nền có chữ
 * đè lên, không phải nét vẽ mảnh cần nổi. Nền quá rực làm chữ khó đọc và kéo
 * chú ý khỏi đồ thị, vốn mới là thứ chính.
 */
const MAU_NEN: readonly string[] = [
  '#8d6a9f',
  '#5f7d95',
  '#7a8f6b',
  '#a1785c',
  '#96636b',
  '#5c8a86',
  '#87789e',
  '#6e8b5e',
  '#9c7a52',
  '#657a9b',
  '#8f6f8c',
  '#6b9080',
  '#a06e6e',
  '#5d8a9c',
  '#93855c',
  '#7b6f9a',
]

/** Avatar tự sinh cho một tác giả: chữ cái đầu + màu nền ổn định theo email. */
export interface AvatarTuSinh {
  /** Một hoặc hai ký tự hoa. */
  chu: string
  /** Mã màu nền, dạng `#rrggbb`. */
  mauNen: string
}

/**
 * Lấy chữ cái đại diện từ tên tác giả.
 *
 * Quy tắc: chữ cái đầu của **từ đầu** và **từ cuối**, viết hoa. `Pham Ngoc
 * Tuyen` → `PT`. Tên một từ → một chữ.
 *
 * # Hai ca thật mà bản ngây thơ làm hỏng
 *
 * 1. **Tên không phải chữ Latinh.** `张伟` không có khoảng trắng và
 *    `toUpperCase()` không đổi gì — trả `张` là đúng và đọc được. Một bản lọc
 *    `[A-Z]` sẽ trả chuỗi rỗng và cho một ô trống.
 * 2. **Tên là địa chỉ email hoặc handle** (`trungdev/vtconline` trong ảnh người
 *    dùng gửi). Tách theo khoảng trắng cho một từ duy nhất; lấy ký tự đầu là
 *    hợp lý và không cần ca riêng.
 *
 * Dùng `Array.from` chứ không phải `[0]`: emoji và ký tự ngoài BMP chiếm hai
 * code unit, nên `[0]` cắt đôi chúng thành ký tự thay thế.
 */
export function chuDaiDien(ten: string): string {
  const tu = ten.trim().split(/\s+/).filter(Boolean)
  if (tu.length === 0) return '?'

  const dau = Array.from(tu[0] ?? '')[0] ?? ''
  if (tu.length === 1) return dau.toUpperCase()

  const cuoi = Array.from(tu[tu.length - 1] ?? '')[0] ?? ''
  return (dau + cuoi).toUpperCase()
}

/**
 * Avatar tự sinh cho một tác giả.
 *
 * Màu theo **email**, chữ theo **tên**. Tách hai nguồn là có lý do: cùng một
 * người thường commit dưới nhiều cách viết tên (`tuyenpn`, `Pham Ngoc Tuyen`,
 * `tuyen`) nhưng cùng một email. Băm theo email giữ màu của họ ổn định qua cả
 * ba, nên mắt vẫn nhận ra cùng một người.
 *
 * Email chuẩn hoá về chữ thường và bỏ khoảng trắng — cùng phép chuẩn hoá mà
 * Gravatar đòi, nên tầng tự sinh và tầng Gravatar không bao giờ bất đồng về
 * "hai email này có phải một người không".
 */
export function avatarTuSinh(ten: string, email: string): AvatarTuSinh {
  const emailChuan = email.trim().toLowerCase()
  const chiSo = bam32(emailChuan) % MAU_NEN.length
  return {
    chu: chuDaiDien(ten),
    // `MAU_NEN` không rỗng và `chiSo` luôn trong khoảng, nhưng TS không biết
    // điều đó — `?? ` giữ kiểu là `string` thay vì `string | undefined`.
    mauNen: MAU_NEN[chiSo] ?? MAU_NEN[0] ?? '#8d6a9f',
  }
}

/**
 * URL Gravatar cho một hash đã tính sẵn — **chỉ gọi khi người dùng đã bật**.
 *
 * # Hash tính ở Rust, không ở đây
 *
 * Gravatar định danh bằng MD5 của email. Web Crypto của trình duyệt **không có
 * MD5** (chỉ SHA-1/256/384/512), nên lựa chọn là tự viết MD5 trong TS, thêm một
 * gói npm, hoặc tính ở Rust. Dự án chọn Rust: `Commit.authorAvatarHash` đến từ
 * backend đã băm sẵn, nên **email không bao giờ rời khỏi Rust** để đi vào lớp
 * giao diện. Muốn rò email ra ngoài cũng khó hơn hẳn, và đó là điều đáng có ở
 * một công cụ mã nguồn mở.
 *
 * `d=404` là phần quan trọng nhất của URL này: nó bảo Gravatar trả **404** khi
 * email không có ảnh, thay vì trả một ảnh mặc định do họ sinh. Nhờ vậy `onError`
 * của `<img>` kích hoạt và giao diện lui về avatar tự sinh — nhất quán với mọi
 * tác giả khác — thay vì hiện một identicon lạ theo phong cách khác hẳn.
 *
 * `s=` xin đúng kích thước hiển thị nhân hai, cho màn HiDPI.
 */
export function urlGravatar(hash: string, coPx: number): string {
  return `https://www.gravatar.com/avatar/${hash}?d=404&s=${coPx * 2}`
}
