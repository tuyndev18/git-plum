# Quick Task 260923-kzl: Remake giao diện theo design system Cursor - Context

**Gathered:** 2026-09-23
**Status:** Ready for planning

<domain>
## Task Boundary

Reskin toàn bộ giao diện git-plum theo `skills/design/Cursor/SKILL.md`. CHỈ đổi phần nhìn
(màu, chữ, bo góc, độ sâu, khoảng cách nhỏ). KHÔNG đổi bố cục, cấu trúc DOM, logic, kích
thước cố định đã ghim bằng test (ROW_HEIGHT, --ref-col-width, --min-split-width).

</domain>

<decisions>
## Implementation Decisions

### Theme
- Theme SÁNG (cream Cursor) là mặc định trong `:root`: canvas #f7f7f4, canvas-soft #fafaf7,
  surface-card #ffffff, surface-strong/hairline #e6e5e0, hairline-soft #efeee8,
  hairline-strong #cfcdc4, ink #26251e, body #5a5852, muted #807d72, muted-soft #a09c92.
- Theme TỐI ấm dưới `@media (prefers-color-scheme: dark) { :root:not([data-theme='light']) {...} }`
  — đảo mực/giấy (giống pricing-tier-featured): nền ~#1b1a16 / raised ~#23221d / inset ~#151410,
  hairline ~#34322b / strong ~#46443b, text #f0efe9 / dim #b3b0a5 / faint #807d72, accent #ff6a1f.
  Đảo nhánh hiện tại (đang là tối mặc định + sáng trong media query).
- Map token hiện có: --bg=canvas, --bg-raised=surface-card, --bg-inset=canvas-soft/surface-strong
  (chọn sao cho vùng inset vẫn tách khỏi nền), --border=hairline, --border-strong=hairline-strong,
  --text=ink, --text-dim=body, --text-faint=muted.
- --accent = Cursor Orange #f54e00 (active #d04200 → --accent-dim). Dùng tiết kiệm: nút chính,
  focus, trạng thái chọn. --danger=#cf2d56, --success=#1f8a65. Thêm --warn (vàng ấm #c08532).
- --row-highlight: đổi từ teal sang cam/ink ấm phù hợp (vẫn dùng qua color-mix với transparent —
  test ghim hàng commit phải có nền alpha).
- --graph-node-fill theo --bg của từng theme.

### Typography
- --font-ui: 'Inter', system-ui, -apple-system, 'Segoe UI', 'Helvetica Neue', Arial, sans-serif
  (CursorGothic có license → Inter). Không tải font từ mạng (app desktop offline, privacy) —
  chỉ khai trong stack.
- --font-mono: 'JetBrains Mono', 'Cascadia Code', Consolas, monospace.
- Tiêu đề/nhãn lớn weight 400 với letter-spacing âm nhẹ; nhãn section uppercase 11px/600/0.88px.
  Không dùng bold 700 cho display.

### Shape & depth
- Radius: nút/input 8px, thẻ/panel/popover 12px, tag inline 4px, badge/pill 9999px, hàng gọn 6px.
- Hairline-only: bỏ mọi box-shadow đổ bóng (giữ box-shadow inset dùng làm viền/chỉ báo 1px
  vì nó là hairline, không phải elevation).

### Hardcoded colors
- Khai trong :root (cả 2 theme) các biến đang bị dùng mà chưa định nghĩa: --color-danger,
  --color-danger-bg, --color-bg-subtle, --color-row-hover, --color-row-selected,
  --color-selected-bg, --color-accent, --color-warn, --color-notice-bg, --color-border,
  --color-subtle-bg, --color-added, --color-removed, --warn, --ok — trỏ về token mới.
- Thay hex cứng trong app.css (#d9a441, #8a6d3b, #555, #222, #444, rgba(40,140,60..), #e05252, #fff…)
  bằng var(...) token.
- `SELECTION_RING` trong src/lib/graph-render/geometry.ts (#e6edf3) vô hình trên nền cream →
  cho canvas đọc từ biến CSS `--graph-selection-ring` theo đúng pattern `readNodeFill`
  (fallback hằng số), cập nhật NODE_FILL fallback = cream.
- Lane palette & avatar palette: GIỮ NGUYÊN (có test độ tách màu), ngoài phạm vi.
- diff-render/theme.ts không mang màu → không đổi.

### Claude's Discretion
- Giá trị chính xác của theme tối, cường độ color-mix, chi tiết từng component.
- Timeline pastels của Cursor: KHÔNG dùng (không có agent timeline trong app).

</decisions>

<canonical_refs>
## Canonical References

- skills/design/Cursor/SKILL.md — design system nguồn
- src/styles/app.css.test.ts — test ghim cấu trúc CSS, phải xanh
</canonical_refs>
