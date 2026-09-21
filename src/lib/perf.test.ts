/**
 * Test cho dụng cụ đo hiệu năng.
 *
 * Hai điều đáng ghim nhất, và cả hai đều là điều kiện trong `<threat_model>` của
 * plan 02-07:
 *
 * 1. **Cờ tắt → không một `requestAnimationFrame` nào được đăng ký** (T-02-25).
 *    Kiểm bằng cách đếm số lần hàm bị gọi, không bằng cách đọc mã. "Tôi tin là
 *    nó không chạy" không phải bằng chứng — bộ đo nằm cùng bundle với mã sản
 *    phẩm nên một vòng khung chạy lén là chi phí thật của người dùng thật.
 *
 * 2. **`p1Fps` và `longestFrameMs` tính đúng trên dữ liệu khung dựng sẵn.** Đo
 *    bằng `requestAnimationFrame` thật thì thời gian không lặp lại được và test
 *    sẽ chập chờn; vì vậy phép tính tách ra thành `tinhBaoCao` và được kiểm trên
 *    mảng khoảng-cách cố định.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { isPerfEnabled, measureFirstPaint, measureScrollFps, tinhBaoCao } from './perf'

function batCo() {
  globalThis.localStorage.setItem('gitPlumPerf', '1')
}

function tatCo() {
  globalThis.localStorage.removeItem('gitPlumPerf')
}

beforeEach(() => {
  tatCo()
  vi.restoreAllMocks()
})

afterEach(() => {
  tatCo()
})

describe('isPerfEnabled', () => {
  it('tắt mặc định khi không có cờ nào', () => {
    expect(isPerfEnabled()).toBe(false)
  })

  it('bật khi localStorage.gitPlumPerf = "1"', () => {
    batCo()
    expect(isPerfEnabled()).toBe(true)
  })

  it('giá trị khác "1" vẫn là tắt — không nhận "true" hay "0"', () => {
    globalThis.localStorage.setItem('gitPlumPerf', 'true')
    expect(isPerfEnabled()).toBe(false)
    globalThis.localStorage.setItem('gitPlumPerf', '0')
    expect(isPerfEnabled()).toBe(false)
  })
})

describe('measureScrollFps — cờ tắt', () => {
  /**
   * T-02-25. Đây là test quan trọng nhất trong tệp: nó chứng minh bản release
   * không trả phí cho bộ đo.
   */
  it('KHÔNG đăng ký requestAnimationFrame nào khi cờ tắt', async () => {
    const raf = vi.spyOn(globalThis, 'requestAnimationFrame')

    const report = await measureScrollFps(document.createElement('div'), 1000)

    expect(raf).not.toHaveBeenCalled()
    expect(report.frames).toBe(0)
    expect(report.avgFps).toBe(0)
    expect(report.p1Fps).toBe(0)
    expect(report.longestFrameMs).toBe(0)
  })

  it('trả báo cáo rỗng ngay, không chờ hết durationMs', async () => {
    const truoc = Date.now()
    await measureScrollFps(document.createElement('div'), 5000)
    // Nếu nó thật sự chờ 5 giây thì test này hết giờ; mốc rộng rãi để không
    // chập chờn trên máy CI chậm.
    expect(Date.now() - truoc).toBeLessThan(1000)
  })
})

describe('measureScrollFps — cờ bật', () => {
  it('đếm khung từ requestAnimationFrame giả lập và tự dừng', async () => {
    batCo()

    // requestAnimationFrame giả lập: mỗi khung cách nhau đúng 16ms, dừng sau
    // khi vượt durationMs. Thời gian do test điều khiển hoàn toàn nên kết quả
    // lặp lại được.
    let dongHo = 0
    vi.spyOn(performance, 'now').mockImplementation(() => dongHo)
    const raf = vi
      .spyOn(globalThis, 'requestAnimationFrame')
      .mockImplementation((cb: FrameRequestCallback) => {
        dongHo += 16
        // Gọi bất đồng bộ để không tràn ngăn xếp khi số khung lớn.
        setTimeout(() => cb(dongHo), 0)
        return 0
      })

    const report = await measureScrollFps(document.createElement('div'), 160)

    expect(raf).toHaveBeenCalled()
    // 160ms / 16ms mỗi khung = 10 khung.
    expect(report.frames).toBe(10)
    expect(report.durationMs).toBe(160)
    expect(report.avgFps).toBeCloseTo(62.5, 1)
    expect(report.longestFrameMs).toBe(16)
  })
})

describe('tinhBaoCao', () => {
  it('khung đều 16ms cho ~62.5 FPS ở cả trung bình lẫn phân vị 1', () => {
    const khung = Array.from({ length: 100 }, () => 16)
    const r = tinhBaoCao(khung, 1600)

    expect(r.frames).toBe(100)
    expect(r.avgFps).toBeCloseTo(62.5, 1)
    expect(r.p1Fps).toBeCloseTo(62.5, 1)
    expect(r.longestFrameMs).toBe(16)
  })

  /**
   * Đây là ca biện minh cho việc tồn tại của `p1Fps`: 99 khung đẹp cộng đúng
   * một khung 400ms. `avgFps` vẫn trên 50 và sẽ báo "đạt", nhưng người dùng
   * thấy khựng rõ. `p1Fps` và `longestFrameMs` bắt được, `avgFps` thì không.
   */
  it('một khung 400ms lẫn trong 99 khung đẹp: avgFps vẫn "đạt" nhưng p1Fps thì không', () => {
    const khung = [...Array.from({ length: 99 }, () => 16), 400]
    const tong = 99 * 16 + 400
    const r = tinhBaoCao(khung, tong)

    expect(r.longestFrameMs).toBe(400)
    expect(r.p1Fps).toBeCloseTo(2.5, 1)

    // Chính là cái bẫy: trung bình vượt ngưỡng 50 trong khi trải nghiệm thật đã hỏng.
    expect(r.avgFps).toBeGreaterThan(50)
    expect(r.p1Fps).toBeLessThan(50)
  })

  it('mảng rỗng cho báo cáo rỗng, không chia cho 0', () => {
    const r = tinhBaoCao([], 0)
    expect(r.frames).toBe(0)
    expect(r.avgFps).toBe(0)
    expect(r.p1Fps).toBe(0)
    expect(Number.isNaN(r.p1Fps)).toBe(false)
  })

  it('tổng thời gian bằng 0 không sinh Infinity', () => {
    const r = tinhBaoCao([16, 16], 0)
    expect(Number.isFinite(r.avgFps)).toBe(true)
    expect(r.avgFps).toBe(0)
  })
})

describe('measureFirstPaint', () => {
  it('cờ tắt trả hàm rỗng, không ghi console', () => {
    const info = vi.spyOn(console, 'info').mockImplementation(() => {})
    const xong = measureFirstPaint('nap-trang-dau')
    expect(xong()).toBe(0)
    expect(info).not.toHaveBeenCalled()
  })

  it('cờ bật ghi một dòng có tiền tố [perf] và trả số ms', () => {
    batCo()
    const info = vi.spyOn(console, 'info').mockImplementation(() => {})

    let dongHo = 0
    vi.spyOn(performance, 'now').mockImplementation(() => dongHo)

    const xong = measureFirstPaint('nap-trang-dau')
    dongHo = 742
    const ms = xong()

    expect(ms).toBeCloseTo(742, 0)
    expect(info).toHaveBeenCalledTimes(1)
    expect(String(info.mock.calls[0]?.[0])).toContain('[perf]')
    expect(String(info.mock.calls[0]?.[0])).toContain('742')
  })

  /**
   * React StrictMode chạy effect hai lượt lúc phát triển. Không chặn thì console
   * có hai dòng với hai số khác nhau và người chép số vào tài liệu lấy nhầm số
   * lớn hơn.
   */
  it('gọi lần hai không ghi thêm dòng nào', () => {
    batCo()
    const info = vi.spyOn(console, 'info').mockImplementation(() => {})

    const xong = measureFirstPaint('nap-trang-dau')
    xong()
    xong()

    expect(info).toHaveBeenCalledTimes(1)
  })

  /**
   * T-02-24: nhãn là thứ duy nhất đi vào console, nên phải chắc nó không mang
   * theo đường dẫn. Test này ghim hành vi "chỉ ghi nhãn và số" — nếu ai đó sau
   * này nối thêm `repoPath` vào dòng log, test vẫn đỗ nhưng đã có chỗ để bàn.
   */
  it('chỉ ghi nhãn và số, không có gì khác trong dòng log', () => {
    batCo()
    const info = vi.spyOn(console, 'info').mockImplementation(() => {})

    let dongHo = 0
    vi.spyOn(performance, 'now').mockImplementation(() => dongHo)
    const xong = measureFirstPaint('nap-trang-dau')
    dongHo = 100
    xong()

    const dong = String(info.mock.calls[0]?.[0])
    expect(dong).toMatch(/^\[perf\] nap-trang-dau: [\d.]+ms$/)
  })
})
