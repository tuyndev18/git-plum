/**
 * Bộ icon SVG inline — nguồn icon DUY NHẤT của dự án.
 *
 * # Vì sao tự vẽ, không dùng thư viện
 *
 * Ràng buộc gói cài đặt dưới 20MB (CLAUDE.md) không phải lý do chính — `lucide-react`
 * tree-shake xuống ~1KB mỗi icon nên nó cũng vừa. Lý do thật là **số lượng**: cả ứng
 * dụng cần khoảng mười hình, mỗi hình hai ba đường `path`. Kéo một dependency npm về
 * cho mười `path` là thêm một thứ phải cập nhật, phải kiểm license, và phải giải thích
 * cho người đóng góp — trong khi tệp này đọc hết trong một phút.
 *
 * # Hai ràng buộc mà mọi icon thêm vào PHẢI giữ
 *
 * 1. **`stroke="currentColor"`, không bao giờ mã màu cứng.** Nút chứa icon đã có
 *    `color` từ `app.css`, gồm cả nhánh `@media (prefers-color-scheme: light)` và
 *    trạng thái `:disabled`. Một `stroke="#e4e4ea"` trong đây sẽ giữ màu tối khi cả
 *    giao diện đã sang sáng — cùng lớp lỗi mà `theme.ts` của trình xem diff đã ghi.
 * 2. **`aria-hidden`.** Icon là trang trí; chữ thật nằm ở `aria-label` của nút bọc
 *    ngoài. Thiếu nó thì trình đọc màn hình đọc cả hai, hoặc đọc một `<svg>` rỗng.
 *
 * `viewBox="0 0 16 16"` cho mọi icon: lưới 16px khớp cỡ chữ giao diện, nên icon và
 * chữ cùng hàng không lệch đường cơ sở.
 */

interface IconProps {
  /** Cỡ ô vuông (px). Mặc định 16 — khớp lưới `viewBox`. */
  size?: number
  className?: string
}

/**
 * Thuộc tính chung. Tách ra hằng số để không một icon nào vô tình thiếu
 * `currentColor` hay `aria-hidden` — hai thứ mà quên là lỗi im lặng.
 */
function svgProps(size: number, className?: string) {
  return {
    width: size,
    height: size,
    viewBox: '0 0 16 16',
    fill: 'none',
    stroke: 'currentColor',
    strokeWidth: 1.5,
    strokeLinecap: 'round' as const,
    strokeLinejoin: 'round' as const,
    'aria-hidden': true,
    focusable: false,
    className,
  }
}

/** Thư mục mở — "Mở repository". */
export function IconFolderOpen({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <path d="M1.5 12.5V3.5a1 1 0 0 1 1-1h3.2l1.5 1.8h5.3a1 1 0 0 1 1 1v1.2" />
      <path d="M1.5 12.5l1.9-5.1a1 1 0 0 1 .94-.65h10.2a.6.6 0 0 1 .56.81l-1.7 4.6a1 1 0 0 1-.94.65H2.5a1 1 0 0 1-1-1z" />
    </svg>
  )
}

/** Dấu nhân — "Đóng repository". */
export function IconClose({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <path d="M4 4l8 8M12 4l-8 8" />
    </svg>
  )
}

/** Dấu nhắc dòng lệnh — "Nhật ký lệnh". */
export function IconTerminal({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" />
      <path d="M4.5 6.5L6.5 8l-2 1.5M8.5 10h3" />
    </svg>
  )
}

/**
 * Hai cột cạnh nhau — "Xem hai cột".
 *
 * Đường dọc giữa là thứ phân biệt nó với `IconViewUnified`; hai hình chỉ khác
 * nhau đúng một nét nên chúng phải luôn nằm cạnh nhau khi ai đó sửa.
 */
export function IconViewSplit({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" />
      <path d="M8 2.5v11" />
    </svg>
  )
}

/** Các dòng xếp chồng — "Xem hợp nhất". */
export function IconViewUnified({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <rect x="1.5" y="2.5" width="13" height="11" rx="1.5" />
      <path d="M4 6h8M4 8.5h8M4 11h5" />
    </svg>
  )
}

/**
 * Dấu chấm giữa và mũi tên — "Hiện/ẩn ký tự khoảng trắng".
 *
 * Vẽ đúng hai glyph mà tính năng bật ra: chấm giữa cho dấu cách, mũi tên cho tab.
 * Một icon "con mắt" chung chung sẽ không nói được nút này làm gì.
 */
export function IconWhitespace({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <circle cx="4" cy="8" r="1" fill="currentColor" stroke="none" />
      <path d="M7.5 8h5.5M10.8 5.8L13 8l-2.2 2.2" />
    </svg>
  )
}

/** Đồng hồ có mũi tên lùi — "Lịch sử tệp". */
export function IconHistory({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <path d="M2.2 8a5.8 5.8 0 1 0 1.9-4.3" />
      <path d="M1.8 2.2v3h3" />
      <path d="M8 5v3.2l2.1 1.3" />
    </svg>
  )
}

/** Mũi tên lên — "Khối thay đổi trước đó". */
export function IconChevronUp({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <path d="M4 10l4-4 4 4" />
    </svg>
  )
}

/** Mũi tên xuống — "Khối thay đổi kế tiếp". */
export function IconChevronDown({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <path d="M4 6l4 4 4-4" />
    </svg>
  )
}

/**
 * Vòng tròn quay — trạng thái "Đang mở…".
 *
 * Quay bằng CSS (`.icon-spin` trong `app.css`), không bằng `<animateTransform>`:
 * SMIL không chạy đều trong WebView2 và không tôn trọng
 * `prefers-reduced-motion`, còn quy tắc CSS thì có.
 */
export function IconSpinner({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className ? `icon-spin ${className}` : 'icon-spin')}>
      <path d="M8 1.8a6.2 6.2 0 1 0 6.2 6.2" />
    </svg>
  )
}

/** Người trong vòng tròn — công tắc ảnh đại diện từ Gravatar. */
export function IconAvatar({ size = 16, className }: IconProps) {
  return (
    <svg {...svgProps(size, className)}>
      <circle cx="8" cy="8" r="6.2" />
      <circle cx="8" cy="6.4" r="2" />
      <path d="M3.9 13.2a4.4 4.4 0 0 1 8.2 0" />
    </svg>
  )
}
