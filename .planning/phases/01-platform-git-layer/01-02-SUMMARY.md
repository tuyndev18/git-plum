---
phase: 01-platform-git-layer
plan: 02
subsystem: testing
tags: [vitest, happy-dom, testing-library, zustand, vite, typescript]

# Dependency graph
requires:
  - phase: 01-platform-git-layer
    provides: "`src/lib/commands.ts` (PLAT-04), `src/stores/repoStore.ts` (PLAT-05), `src/lib/ipc.ts` — đã có sẵn từ lần scaffold, plan này chỉ phủ test lên"
provides:
  - "Hạ tầng kiểm thử phía giao diện chạy được: khối `test` của vitest trong `vite.config.ts`, `environment: 'happy-dom'`, `setupFiles`"
  - "`resolve.alias` tường minh cho `@` — trước đó alias chỉ tồn tại trong `tsconfig.paths`"
  - "Job `frontend` trong `.github/workflows/ci.yml` từ đỏ chuyển sang xanh và có việc thật để làm"
  - "Test khoá ràng buộc kiến trúc PLAT-04 (sổ đăng ký lệnh) và PLAT-05 (`byRepo` dạng map)"
  - "Khuôn mẫu giả lập IPC ở ranh giới module `@/lib/ipc` cho mọi test store về sau"
affects: [mọi plan phía giao diện từ Phase 2 trở đi, command palette v2, đa repository]

# Tech tracking
tech-stack:
  added: []  # vitest, happy-dom, @testing-library/react đã cài sẵn từ trước; plan này chỉ cấu hình
  patterns:
    - "Cấu hình vitest sống trong `vite.config.ts`, không tách `vitest.config.ts` riêng — một nguồn sự thật cho alias"
    - "Giả lập IPC bằng `vi.mock('@/lib/ipc', ...)` ở ranh giới module, không vá `invoke` toàn cục"
    - "Test logic store gọi thẳng `useRepoStore.getState()`, không render component"
    - "Mỗi `it` kèm một dòng bình luận nêu ràng buộc nó khoá lại"

key-files:
  created:
    - src/test/setup.ts
    - src/test/setup.test.tsx
    - src/lib/commands.test.ts
    - src/stores/repoStore.test.ts
  modified:
    - vite.config.ts
    - tsconfig.json

key-decisions:
  - "Nhập `defineConfig` từ `'vitest/config'` thay vì `'vite'` để khối `test` có kiểu, tránh lỗi thuộc tính lạ khi `tsc --noEmit`"
  - "Thêm cả `\"vite/client\"` bên cạnh `\"vitest/globals\"` vào `compilerOptions.types` — khai báo `types` tắt việc tự nạp mọi `@types/*` khác"
  - "Test khói nhập `@/lib/commands` chứ không phải `@/lib/ipc`, vì `ipc.ts` nhập `@tauri-apps/api/core` ở cấp module và thứ đó không chạy ngoài webview Tauri"
  - "Kiểm chứng giá trị của test bằng phép đột biến thủ công: gỡ ràng buộc trong mã nguồn phải làm test đỏ, sau đó hoàn nguyên"

patterns-established:
  - "Dọn trạng thái dùng chung trong `beforeEach`: `clearCommands()` cho sổ đăng ký, `useRepoStore.setState(...)` cho store — cả hai đều là singleton ở phạm vi module"
  - "Với `noUncheckedIndexedAccess`, đọc `byRepo[id]` qua một hàm trợ giúp khẳng định tồn tại rồi thu hẹp kiểu, không dùng `!` để làm câm trình kiểm kiểu"
  - "Chứng minh `await` thật sự xảy ra bằng promise giải quyết thủ công, không dùng mock đã resolve sẵn"

requirements-completed: [PLAT-04, PLAT-05]

# Metrics
duration: 9min
completed: 2026-09-21
---

# Phase 1 Plan 02: Hạ tầng kiểm thử giao diện Summary

**Dựng khối cấu hình vitest còn thiếu để `npm test` thôi thoát mã 1, rồi khoá hai ràng buộc kiến trúc loại "làm ngay hoặc không bao giờ" — PLAT-04 và PLAT-05 — bằng 23 test có kiểm chứng đột biến.**

## Performance

- **Duration:** 9 phút
- **Started:** 2026-09-21T12:16:00Z
- **Completed:** 2026-09-21T12:25:00Z
- **Tasks:** 3/3
- **Files modified:** 6 (4 tạo mới, 2 sửa)

## Accomplishments

- **Chấm dứt tình trạng job `frontend` chạy vào khoảng không.** Trước plan này `npm test` thoát mã 1 với "No test files found": `vitest` đã cài, `npm test` đã khai báo trong `package.json`, `.github/workflows/ci.yml` đã gọi, nhưng không có tệp cấu hình vitest nào trong toàn dự án. Giờ `npm test` thoát mã 0 với 23 test đỗ trên 3 tệp.
- **Bổ sung `resolve.alias` cho `@` vào `vite.config.ts`.** Tám tệp nguồn nhập qua `@/`; trước đó alias chỉ tồn tại trong `tsconfig.paths`, thứ dạy trình kiểm kiểu chứ không dạy trình giải mô-đun. `npm run typecheck` và `vite build` vẫn xanh nhờ những đường khác, nhưng vitest thì không có đường nào.
- **Khoá ba quy tắc PLAT-04** bằng `src/lib/commands.test.ts` (12 test): trùng mã định danh thì ném lỗi chứ không im lặng ghi đè, lệnh chưa đăng ký thì reject chứ không thành nút bấm chết, lệnh bị vô hiệu hoá thì không chạy nhưng cũng không ném lỗi.
- **Khoá ràng buộc cốt lõi PLAT-05** bằng `src/stores/repoStore.test.ts` (10 test): hai repository cùng tồn tại trong `byRepo`, đóng cái này không đụng cái kia.
- **Kiểm chứng test thật sự có răng.** Mỗi bộ test đều qua một phép đột biến thủ công: gỡ ràng buộc khỏi mã nguồn, xác nhận test đỏ, rồi hoàn nguyên. Chi tiết ở mục "Decisions Made".

## Task Commits

1. **Task 1: Dựng cấu hình vitest và tệp setup** — `27af0cf` (chore)
2. **Task 2: Test sổ đăng ký lệnh — khoá ràng buộc PLAT-04** — `da5c595` (test)
3. **Task 3: Test repoStore — khoá ràng buộc PLAT-05** — `354752f` (test)

Lưu ý: nhật ký git xen kẽ commit của plan 01-01 chạy song song (`9928194`, `3d3e11e`, `e1dc4f8`). Ba commit ở trên chỉ chạm đúng sáu tệp trong `files_modified` của plan này; không có tệp Rust nào bị đụng tới.

## Files Created/Modified

- `vite.config.ts` — thêm `resolve.alias` cho `@` (dùng `fileURLToPath(new URL('./src', import.meta.url))` vì tệp này là ESM, không có `__dirname`) và khối `test` của vitest với `environment: 'happy-dom'`, `globals: true`, `setupFiles`, `include`, `restoreMocks: true`. `defineConfig` chuyển sang nhập từ `'vitest/config'`.
- `tsconfig.json` — thêm `"types": ["vitest/globals", "vite/client"]`.
- `src/test/setup.ts` — gọi `cleanup()` của `@testing-library/react` trong `afterEach` để DOM giữa các test không rò rỉ.
- `src/test/setup.test.tsx` — test khói: render qua `@testing-library/react` và nhập qua `@/` để chứng minh cả `happy-dom` lẫn alias thật sự hoạt động dưới vitest.
- `src/lib/commands.test.ts` — 12 test ràng buộc PLAT-04.
- `src/stores/repoStore.test.ts` — 10 test ràng buộc PLAT-05.

## Decisions Made

**Thêm `"vite/client"` bên cạnh `"vitest/globals"`.** Plan đã cảnh báo trước khả năng này: khai báo `compilerOptions.types` tắt việc tự nạp mọi `@types/*` khác. Thay vì thử rồi chờ lỗi, tôi đưa luôn `"vite/client"` vào từ đầu. `npm run typecheck` xanh ngay lần chạy đầu.

**Test khói nhập `@/lib/commands`, không phải `@/lib/ipc`.** Plan yêu cầu test khói dùng `@/` trong một import để chứng minh alias hoạt động nhưng không chỉ định mô-đun nào. `ipc.ts` nhập `invoke` từ `@tauri-apps/api/core` ở cấp module, thứ này không chạy ngoài webview Tauri; nhập nó vào test khói sẽ biến một test hạ tầng đơn giản thành test cần giả lập. `commands.ts` thuần tuý, không phụ thuộc gì.

**Kiểm chứng đột biến cho cả hai bộ test.** `<done>` của Task 2 yêu cầu tường minh: "Xoá thử phép kiểm tra trùng `id` trong `registerCommand` làm ít nhất một test đỏ." Tôi làm đúng vậy — gỡ khối `if (registry.has(...)) throw` khỏi `src/lib/commands.ts`, chạy lại, **2 test đỏ / 10 đỗ**, rồi hoàn nguyên và xác nhận `git diff` rỗng.

Task 3 không yêu cầu bước này nhưng tôi làm tương tự vì ràng buộc PLAT-05 chính là thứ dễ bị "dọn dẹp" nhất ở Phase 2: sửa `openRepository` để `byRepo` chỉ giữ repository vừa mở (bỏ `...s.byRepo`) — đúng hình dạng của một lần rút gọn tưởng là vô hại — làm **2 test đỏ / 8 đỗ**, rồi hoàn nguyên. Không có bước này thì không có gì bảo đảm test đang khoá ràng buộc chứ chỉ đang mô tả mã hiện tại.

**Thêm một test ngoài danh sách `<behavior>` của Task 3.** `refreshBranch` với id không có trong `byRepo` bị bỏ qua im lặng (mã nguồn có nhánh `if (!slice) return s`). Đây là hành vi có thật, bảo vệ chống việc phản hồi IPC đến muộn dựng lại một slice ma sau khi repository đã đóng. Hành vi này có trong mã nhưng không có trong plan.

## Deviations from Plan

Không có sai lệch nào cần áp dụng Rule 1–4. Kế hoạch chạy đúng như viết. Hai điều chỉnh nhỏ nằm trong khoảng tự do mà chính plan cho phép, đã ghi ở mục "Decisions Made": thêm `"vite/client"` (plan nêu sẵn như một khả năng) và chọn mô-đun cho test khói (plan không chỉ định).

Số lượng test vượt ngưỡng tối thiểu: Task 2 có 12 test (yêu cầu ≥10), Task 3 có 10 test (yêu cầu ≥9), tổng 23 (yêu cầu ≥19).

## Issues Encountered

**`tsconfig.json` không sửa được bằng `python`.** Máy này không có Python (`Python was not found; run without arguments to install from the Microsoft Store`). Lần chỉnh đầu im lặng không có tác dụng — tệp giữ nguyên. Phát hiện ngay vì tôi đọc lại nội dung tệp sau khi ghi thay vì tin vào việc lệnh không báo lỗi. Làm lại bằng `sed -i`, thành công.

**Không có sự cố nào với chính vitest.** Vite 8 chạy trên Rolldown và không có esbuild; tôi không đặt `minify` và không thêm plugin nào phụ thuộc esbuild, nên cái bẫy đã biết từ lần scaffold không tái diễn. `vitest@5.0.1` khai báo peer `vite: ^6.4 || ^7 || ^8` và làm việc với `vite@8.3.0` không cần cờ nào đặc biệt.

## Điều đáng chú ý cho người đọc sau

**Bản dựng phát hành không đổi một byte nào.** Trước và sau khi thêm `resolve.alias`, `vite build` cho ra đúng cùng một tên tệp băm: `dist/assets/index-D20mc0tH.js` (268.01 kB). Alias không làm thay đổi kết quả đóng gói — nó chỉ vá lỗ hổng mà vitest sẽ rơi vào. Đây là bằng chứng thay đổi ở Task 1 an toàn với đường phát hành.

## User Setup Required

Không có — toàn bộ phụ thuộc đã nằm sẵn trong `devDependencies` và đã cài.

## Next Phase Readiness

Sẵn sàng. Từ giờ mọi plan phía giao diện có chỗ đặt test và một khuôn mẫu để theo: giả lập IPC ở `@/lib/ipc`, dọn singleton trong `beforeEach`, gọi store qua `getState()`.

Một lưu ý cho Phase 2: `restoreMocks: true` đã bật, nên `vi.fn()` tự khôi phục giữa các test; đừng thêm `vi.restoreAllMocks()` thủ công nữa. Và nếu ai định gỡ một test trong hai tệp `*.test.ts` này, dòng bình luận ngay trên mỗi `it` nói rõ ràng buộc nào đang bị gỡ theo.

---
*Phase: 01-platform-git-layer*
*Completed: 2026-09-21*

## Self-Check: PASSED

Đã xác minh trên đĩa: cả 6 tệp trong `files_modified` tồn tại, cả 3 commit
(`27af0cf`, `da5c595`, `354752f`) có trong `git log`. `npm test` 23 đỗ / 3 tệp,
`npm run typecheck` và `npm run build` cùng thoát mã 0.
