/**
 * Tệp setup chạy trước mọi tệp test (khai báo ở `test.setupFiles` trong
 * `vite.config.ts`).
 *
 * Tồn tại vì `@testing-library/react` gắn component vào `document.body` thật;
 * không dọn dẹp thì DOM của test trước còn nguyên ở test sau, và `screen` sẽ
 * tìm thấy nhiều phần tử trùng nhau. `cleanup()` tự động chỉ hoạt động khi
 * testing-library thấy các hook toàn cục — ở đây gọi tường minh cho chắc.
 */

import { afterEach } from 'vitest'
import { cleanup } from '@testing-library/react'

afterEach(() => {
  cleanup()
})
