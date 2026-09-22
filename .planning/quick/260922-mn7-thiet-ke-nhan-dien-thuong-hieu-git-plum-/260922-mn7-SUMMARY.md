---
phase: quick-260922-mn7
plan: 01
subsystem: brand-identity
tags: [logo, svg, icon, branding, frontend]
dependency-graph:
  requires: []
  provides: [src/assets/logo-mark.svg, src/assets/logo-icon.svg, src/components/Logo.tsx, docs/BRAND.md]
  affects: [src/App.tsx, src-tauri/icons/*]
tech-stack:
  added: []
  patterns:
    - "Inline SVG component with CSS custom properties for theme-aware brand colors (not currentColor)"
    - "Dual SVG source files: transparent-bg mark for in-app use, opaque-tile icon for OS icon generation"
key-files:
  created:
    - src/assets/logo-mark.svg
    - src/assets/logo-icon.svg
    - src/components/Logo.tsx
    - src/components/Logo.test.tsx
    - docs/BRAND.md
  modified:
    - src/App.tsx
    - src-tauri/icons/icon.png
    - src-tauri/icons/icon.ico
    - src-tauri/icons/icon.icns
    - src-tauri/icons/32x32.png
    - src-tauri/icons/64x64.png
    - src-tauri/icons/128x128.png
    - src-tauri/icons/128x128@2x.png
    - src-tauri/icons/Square30x30Logo.png
    - src-tauri/icons/Square44x44Logo.png
    - src-tauri/icons/Square71x71Logo.png
    - src-tauri/icons/Square89x89Logo.png
    - src-tauri/icons/Square107x107Logo.png
    - src-tauri/icons/Square142x142Logo.png
    - src-tauri/icons/Square150x150Logo.png
    - src-tauri/icons/Square284x284Logo.png
    - src-tauri/icons/Square310x310Logo.png
    - src-tauri/icons/StoreLogo.png
decisions:
  - "Plum body drawn with 6 cubic Bézier segments producing an asymmetric sphere with a visible inward cleft on the right side (radius dips 14.0 -> 8.7 -> 15.0 units), sitting low in the 64x64 viewBox (y: 24.7-53) so the branch occupies the top with optical balance"
  - "Branch forks from a single trunk at (32,16) into two curves ending at commit nodes (22,7) and (44,9), separated by 22 units (34% of viewBox width) by branch end for legibility at 16px"
  - "Logo.tsx deliberately does not reuse icons.tsx svgProps()/currentColor convention — brand colors are fixed via var(--accent)/var(--success) so the mark keeps its identity across themes instead of inheriting surrounding text color"
metrics:
  duration: "~45 minutes"
  completed: "2026-09-22"
---

# Phase quick-260922-mn7 Plan 01: Brand Identity (Logo + App Icon) Summary

Tay vẽ logo git-plum (quả mận + nhánh git rẽ đôi mang 2 nút commit) làm hai bản SVG, sinh lại toàn bộ icon ứng dụng qua `tauri icon`, gắn `<Logo/>` vào toolbar, và viết `docs/BRAND.md`.

## What Was Built

**Task 1 — Hai bản SVG logo** (`3505ca3`)
- `src/assets/logo-mark.svg`: nền trong suốt, thân mận `var(--accent)`, nhánh+nút `var(--success)`, viewBox `0 0 64 64`.
- `src/assets/logo-icon.svg`: cùng hình học, màu hex tĩnh (`#9d7cd8`/`#86b384`), thêm `<rect>` nền tile bo góc hex `#16161a`.
- Hình học tính toán và kiểm chứng bằng số trước khi viết: thân mận 6 đoạn cubic Bézier đóng kín (đã xác minh điểm đầu = điểm cuối), có cleft lõm rõ ràng bên phải (bán kính từ tâm dao động 14.0 → 8.7 → 15.0 đơn vị dọc theo phần tư trên-phải — một vết lõm thấy được, không phải một đường cong trơn). Thân nằm trong khung x:17–47, y:24.7–53 của viewBox 64 — sà thấp có chủ ý để cân bằng với nhánh chiếm phần trên.
- Nhánh: thân cây thẳng từ đỉnh mận (32,25) lên điểm rẽ (32,16), rồi rẽ hai đường cubic tới hai nút commit tại (22,7) và (44,9) — khoảng cách hai đầu nhánh 22 đơn vị (34% bề rộng viewBox), đủ tách biệt để đọc được ở kích thước 16px.

**Task 2 — Sinh bộ icon** (`f3ae096`)
- Chạy `npx tauri icon src/assets/logo-icon.svg`, ghi đè toàn bộ `src-tauri/icons/*.png`, `.ico`, `.icns`, và bộ `Square*Logo`/`StoreLogo`.
- Xác nhận bằng số: `icon.png` 1815 byte (mặc định Tauri) → 11481 byte. Toàn bộ 17 tệp trong `src-tauri/icons/` đổi kích thước byte so với bản mặc định.
- `app-icon.png` giữ nguyên 1820 byte — xác nhận nó KHÔNG nằm trong output của `tauri icon` và không được tham chiếu ở đâu trong `src-tauri` (kể cả `tauri.conf.json` `bundle.icon`, chỉ có 4 mục: `32x32.png`, `128x128.png`, `icon.ico`, `icon.icns`) — rác sót từ `create-tauri-app`, ngoài phạm vi task này, không sửa.

**Task 3 — Component `<Logo/>`, gắn App.tsx, viết BRAND.md** (`b6215b3`)
- `src/components/Logo.tsx`: render SVG inline (copy hình học từ `logo-mark.svg`, chuyển `class`/`stroke-width` sang `className`/`strokeWidth`), props `{ size = 20, className? }`, `aria-hidden="true"` + `focusable="false"`. Doc comment đầu file giải thích vì sao KHÔNG dùng `svgProps()`/`currentColor` của `icons.tsx`.
- `src/components/Logo.test.tsx`: 3 test — `aria-hidden="true"` có mặt, markup không chứa `currentColor` (chống hồi quy), `size` mặc định/tuỳ chỉnh đúng.
- `src/App.tsx`: sửa đổi CỘNG THÊM DUY NHẤT — thêm `import { Logo } from '@/components/Logo'` và `<Logo size={20} />` ngay trước `<span className="brand">git-plum</span>` trong `.toolbar-left`. Xác nhận bằng `git diff`: đúng 2 hunk, không chạm dòng nào khác — không va chạm việc dở `handleSelectRef`/cuộn đồ thị của phiên khác trong cùng tệp.
- `docs/BRAND.md`: 5 mục — khái niệm (D-01), tên hiển thị (D-02, xác nhận `package.json`.name và `tauri.conf.json`.productName đã là `git-plum` chữ thường), bảng màu (trích đúng hex từ `app.css` cho cả hai theme), hai biến thể tệp SVG + lệnh tái sinh icon, quy tắc dùng đúng/sai.

## Verification Results

- `npx tsc --noEmit`: **xanh**, không lỗi.
- `npx vitest run src/components/Logo.test.tsx`: **3/3 xanh**.
- `npx vitest run` (toàn bộ): **560/561 xanh**. 1 test thất bại — xem "Deferred Issues" bên dưới, không liên quan tới thay đổi của plan này.
- Icon: `icon.png` xác nhận bằng số (11481 byte, > ngưỡng 1800 byte mặc định) và bằng mắt (xem ảnh trong quá trình thực thi) — quả mận tím với cleft rõ, nhánh xanh rẽ đôi mang hai nút, đọc được cả ở 128×128 lẫn 32×32.
- Không chạy `npm run tauri dev`/build thủ công trong phiên này (nằm ngoài checklist bắt buộc của constraints; icon và component đã xác nhận qua kiểm thử tự động + xem trực tiếp tệp PNG sinh ra).

## Deviations from Plan

### Auto-fixed Issues

None — không có deviation nào cần auto-fix theo Rule 1-3. `tauri icon` sinh thêm bộ `Square*Logo`/`StoreLogo`/`64x64.png` ngoài 4 tệp liệt kê trong `bundle.icon`, nhưng đây là hành vi mặc định của CLI (không phải lỗi) và các tệp đó đã có sẵn trong `src-tauri/icons/` trước khi plan này chạy — commit chỉ cập nhật nội dung, không thêm tệp mới ngoài dự kiến.

## Deferred Issues

**1 test thất bại, KHÔNG liên quan tới plan này — `src/styles/app.css.test.ts`:**
`quy tắc tắt .cm-changedText phải đủ ĐỘ CỤ THỂ để thắng thư viện` — assertion về CSS specificity của một selector `@codemirror/merge` trong `app.css`. Xác nhận: `git diff --stat src/styles/app.css` không có thay đổi nào từ plan này (tệp cuối commit ở `807d942`, plan 03-05 — trước plan này). Đây là lỗi tồn tại sẵn trong cây làm việc, ngoài phạm vi (scope boundary của deviation rules: "Only auto-fix issues DIRECTLY caused by the current task's changes"). Không có `deferred-items.md` cấp phase cho quick task này nên ghi trực tiếp ở đây.

## Self-Check: PASSED

Verified files exist:
- FOUND: src/assets/logo-mark.svg
- FOUND: src/assets/logo-icon.svg
- FOUND: src/components/Logo.tsx
- FOUND: src/components/Logo.test.tsx
- FOUND: docs/BRAND.md
- FOUND: src-tauri/icons/icon.png (11481 bytes, changed from stock)

Verified commits exist:
- FOUND: 3505ca3 (Task 1)
- FOUND: f3ae096 (Task 2)
- FOUND: b6215b3 (Task 3)

---

## Bổ sung sau khi orchestrator kiểm hình (commit `5697f4a`)

Bản logo do executor giao **không dùng được**, phát hiện khi orchestrator
rasterize và XEM chính ảnh PNG đã sinh thay vì chỉ đọc lại toạ độ. Ba lỗi,
đều thuộc loại chỉ lộ ra khi nhìn ảnh:

1. **Đường bao thân bị khoét lõm** ở góc trên phải (đoạn `C38,36 42,34
   47,39`). Executor mô tả đây là "cleft quả mận" và tự kiểm bằng cách lấy
   mẫu bán kính — phép đo xác nhận có vết lõm, nhưng không nói được rằng vết
   lõm đó *trông như lỗi render*. Rãnh thật của quả mận chạy dọc MẶT TRƯỚC,
   không cắt vào đường bao.
2. **Hai cuống đối xứng đọc ra râu côn trùng.** Thử tiếp phương án hai nhánh
   cùng vươn chéo lên — vẫn ra râu. Nguyên nhân: mắt người gộp hai nét đối
   xứng thành một cặp. Chốt bất đối xứng.
3. **Nhánh rẽ gần nằm ngang** đọc ra sợi dây có hạt.

### Bài học cho các task đồ hoạ sau

Tự kiểm bằng số đo toạ độ KHÔNG thay được việc xem ảnh render. Cả hai phép
tự kiểm của executor (sampling bán kính, verify path khép kín) đều PASS trên
một hình mà mắt người nhìn vào là thấy hỏng ngay. Task đồ hoạ phải rasterize
rồi đọc ảnh như một bước bắt buộc.

### Tình trạng cuối

- Hình học đồng bộ ở cả ba nơi: `logo-mark.svg`, `logo-icon.svg`, `Logo.tsx`.
- `docs/BRAND.md` mục 1 viết lại: mô tả hình đúng như đã ship, thêm mục
  "Hai ràng buộc hình học đã trả giá để học".
- `vitest run` toàn bộ: **594/594 PASS**. Lỗi `app.css.test.ts` mà executor
  báo là "pre-existing" đã hết sau khi merge — nó là hệ quả của việc worktree
  đứng trên base cũ, không phải lỗi có sẵn của repo.
- Ghi chú kỹ thuật: KHÔNG viết hai dấu gạch ngang liền nhau trong chú thích
  XML của SVG. Parser của `tauri icon` panic `InvalidComment` và không sinh
  được icon nào.
