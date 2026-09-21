/**
 * Test khói cho hạ tầng kiểm thử.
 *
 * Hai bộ test còn lại (`commands.test.ts`, `repoStore.test.ts`) là logic thuần,
 * không render component nào. Nếu cấu hình `happy-dom` hoặc alias `@` sai, chúng
 * vẫn xanh và không ai biết cho tới lúc viết test component đầu tiên. Tệp này
 * chứng minh cả hai thứ đó thật sự hoạt động.
 */

import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
// Import qua `@/` là chủ ý: chứng minh `resolve.alias` hoạt động dưới vitest,
// thứ mà `tsconfig.paths` không bảo chứng được.
import { listCommands } from '@/lib/commands'

describe('hạ tầng kiểm thử', () => {
  it('render được component qua happy-dom và giải được alias @', () => {
    render(<p>xin chào git-plum</p>)

    expect(screen.getByText('xin chào git-plum')).toBeDefined()
    // Sổ đăng ký nhập qua `@/lib/commands` gọi được — alias đã giải đúng.
    expect(Array.isArray(listCommands())).toBe(true)
  })
})
