/**
 * `<Logo/>` — nhãn hiệu git-plum, KHÔNG dùng `svgProps()`/`currentColor` của
 * `src/components/icons.tsx`.
 *
 * `icons.tsx` bắt icon giao diện luôn `stroke="currentColor"` để tự khớp màu
 * chữ xung quanh — đúng cho icon nút bấm/trạng thái, nhưng logo là DẤU HIỆU
 * THƯƠNG HIỆU: nếu tô bằng `currentColor`, quả mận sẽ đổi màu theo `color`
 * của `.toolbar` bất kỳ lúc nào và mất luôn bản sắc (xem D-01/D-02 và
 * `docs/BRAND.md`). Thay vào đó, màu thân mận/nhánh git đặt trực tiếp bằng
 * `var(--accent)`/`var(--success)` trong SVG — hai biến này tự đổi theo
 * `prefers-color-scheme`, nên logo vẫn theo đúng theme mà không cần
 * `currentColor`.
 *
 * Render markup SVG INLINE (không `<img src=...>`): import qua `<img>` nạp
 * SVG như tài nguyên tĩnh biệt lập, không thừa hưởng biến CSS của trang cha,
 * nên `var(--accent)` bên trong sẽ không resolve được.
 */

interface LogoProps {
  size?: number
  className?: string
}

export function Logo({ size = 20, className }: LogoProps) {
  return (
    <svg
      viewBox="0 0 64 64"
      width={size}
      height={size}
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      {/* Thân quả mận — hình học khớp src/assets/logo-mark.svg */}
      <path
        d="M32,22 C41.5,22 49,28.5 49,37 C49,45.5 41.5,52 32,52 C22.5,52 15,45.5 15,37 C15,28.5 22.5,22 32,22 Z"
        fill="var(--accent)"
      />
      {/* Rãnh dọc mặt trước quả */}
      <path
        d="M34,23 C31,30.5 31,44 34,51"
        fill="none"
        stroke="var(--accent-dim)"
        strokeWidth={2.2}
        strokeLinecap="round"
      />
      {/* Nhánh git: thân chính thẳng đứng + một nhánh rẽ phải (bất đối xứng
          có chủ ý — xem chú thích hình học ở logo-mark.svg) */}
      <path d="M32,23 L32,13" fill="none" stroke="var(--success)" strokeWidth={3} strokeLinecap="round" />
      <path
        d="M32,20 C36,20 41,19 43,15"
        fill="none"
        stroke="var(--success)"
        strokeWidth={3}
        strokeLinecap="round"
      />
      <circle cx={32} cy={11} r={4} fill="var(--success)" />
      <circle cx={44} cy={13} r={3.5} fill="var(--success)" />
    </svg>
  )
}
