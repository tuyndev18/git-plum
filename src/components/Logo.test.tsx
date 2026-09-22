/**
 * Test `<Logo/>` — xác nhận SVG trang trí đúng quy tắc 2 của `icons.tsx`
 * (`aria-hidden`) và chống hồi quy quyết định KHÔNG dùng `currentColor` cho
 * màu thương hiệu (xem doc comment đầu `Logo.tsx`).
 */

import { describe, it, expect } from 'vitest'
import { render } from '@testing-library/react'

import { Logo } from '@/components/Logo'

describe('Logo', () => {
  it('render một svg với aria-hidden="true"', () => {
    const { container } = render(<Logo />)
    const svg = container.querySelector('svg')

    expect(svg).not.toBeNull()
    expect(svg?.getAttribute('aria-hidden')).toBe('true')
  })

  it('không dùng currentColor cho phần tử vẽ chính — màu thương hiệu cố định qua biến CSS', () => {
    const { container } = render(<Logo />)

    expect(container.innerHTML).not.toContain('currentColor')
  })

  it('nhận size tuỳ chỉnh, mặc định 20', () => {
    const macDinh = render(<Logo />)
    const svgMacDinh = macDinh.container.querySelector('svg')
    expect(svgMacDinh?.getAttribute('width')).toBe('20')
    expect(svgMacDinh?.getAttribute('height')).toBe('20')

    const tuyChinh = render(<Logo size={32} />)
    const svgTuyChinh = tuyChinh.container.querySelector('svg')
    expect(svgTuyChinh?.getAttribute('width')).toBe('32')
    expect(svgTuyChinh?.getAttribute('height')).toBe('32')
  })
})
