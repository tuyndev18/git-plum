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
      <path
        d="M32,25 C37,25 40,27 39,32 C38,36 42,34 47,39 C47,47 40,53 32,53 C24,53 17,47 17,39 C17,32 21,26 27,25 C29,24.5 30.5,24.7 32,25 Z"
        fill="var(--accent)"
      />
      <path d="M32,25 L32,16" fill="none" stroke="var(--success)" strokeWidth={3.5} strokeLinecap="round" />
      <path
        d="M32,16 C32,11 26,8 22,7"
        fill="none"
        stroke="var(--success)"
        strokeWidth={3.5}
        strokeLinecap="round"
      />
      <path
        d="M32,16 C32,12 39,10 44,9"
        fill="none"
        stroke="var(--success)"
        strokeWidth={3.5}
        strokeLinecap="round"
      />
      <circle cx={22} cy={7} r={3.5} fill="var(--success)" />
      <circle cx={44} cy={9} r={3.5} fill="var(--success)" />
    </svg>
  )
}
