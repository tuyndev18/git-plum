/**
 * Dụng cụ đo hiệu năng phía giao diện — phục vụ validation checkpoint #1.
 *
 * Tiêu chí thành công số 1 của Phase 2 (và Core Value của cả dự án) là một con
 * số: đồ thị hiện trong **dưới một giây** trên repo 100k commit, cuộn không
 * giật. Tệp này là thứ sinh ra con số đó. Không có nó thì kết luận "nhanh" chỉ
 * là cảm giác, và `gitlanes` — trình xem cùng stack Tauri 2 + React — đã có số
 * thật (32k commit ~300ms ở 60fps) nên lập luận suông không đủ để so.
 *
 * # Vì sao `p1Fps` và `longestFrameMs` quan trọng hơn `avgFps`
 *
 * "Cuộn không giật" là chuyện của khung **tệ nhất**, không phải trung bình. Một
 * lần cuộn có FPS trung bình 58 mà chứa đúng một khung 400ms thì người dùng
 * thấy khựng rất rõ — nhưng `avgFps` vẫn báo "đạt". Vì vậy báo cáo mang cả ba
 * số và quy tắc kết luận của checkpoint #1 ràng buộc vào `p1Fps` (phân vị 1,
 * tức 1% khung tệ nhất) và `longestFrameMs`, không vào `avgFps`.
 *
 * # Tắt mặc định, và tắt nghĩa là KHÔNG CHẠY GÌ
 *
 * T-02-25 (từ chối dịch vụ): một vòng `requestAnimationFrame` của bộ đo mà chạy
 * mãi trong bản release là tự bắn vào chân mình. Nên khi cờ tắt,
 * `measureScrollFps` trả báo cáo rỗng **ngay lập tức** và không đăng ký một
 * khung nào — có test khẳng định đúng điều này, vì "tôi tin là nó không chạy"
 * không phải bằng chứng.
 *
 * T-02-24 (lộ thông tin): mọi thứ ghi ra console ở đây chỉ gồm **số đo và
 * nhãn**. Không đường dẫn repo, không tên tệp, không nội dung commit. Nhãn do
 * người gọi truyền vào, và người gọi trong mã này chỉ truyền hằng chuỗi tĩnh.
 */

/** Báo cáo FPS của một lần đo cuộn. */
export interface FpsReport {
  /** Số khung đếm được trong khoảng đo. */
  frames: number
  /** Độ dài thật của khoảng đo, ms. Có thể lệch nhẹ so với `durationMs` yêu cầu. */
  durationMs: number
  /** Khung trên giây, trung bình trên cả khoảng. */
  avgFps: number
  /**
   * FPS ở phân vị 1 của khoảng-giữa-khung: tức FPS quy đổi từ khung **tệ** nhất
   * trong 1% khung tệ nhất. Đây là số phản ánh cảm giác giật, không phải `avgFps`.
   */
  p1Fps: number
  /** Khoảng giữa hai khung dài nhất, ms. */
  longestFrameMs: number
}

/** Tiền tố ổn định để lọc lại bằng mắt hoặc bằng grep khi chép số vào tài liệu. */
const PREFIX = '[perf]'

/** Báo cáo rỗng trả về khi cờ tắt. Không cấp phát vòng đo nào. */
const BAO_CAO_RONG: FpsReport = {
  frames: 0,
  durationMs: 0,
  avgFps: 0,
  p1Fps: 0,
  longestFrameMs: 0,
}

/**
 * Cờ bật đo. Tắt mặc định.
 *
 * Hai đường bật, cố ý:
 * - `VITE_GSD_PERF=1` lúc dựng — dùng cho bản dựng đo chuyên dụng.
 * - `localStorage.gitPlumPerf = '1'` lúc chạy — bật được trên **bản release đã
 *   dựng sẵn** mà không phải dựng lại. Checkpoint #1 bắt buộc đo trên bản
 *   release (WebView2 ở chế độ dev có công cụ gỡ lỗi gắn vào và số sẽ đẹp hơn
 *   thực tế một cách sai lệch), nên phải có đường bật lúc chạy.
 *
 * `localStorage` có thể ném khi bị chính sách trình duyệt chặn — bọc `try` để
 * dụng cụ đo không bao giờ làm sập ứng dụng thật.
 */
export function isPerfEnabled(): boolean {
  if (import.meta.env?.VITE_GSD_PERF === '1') return true
  try {
    return globalThis.localStorage?.getItem('gitPlumPerf') === '1'
  } catch {
    return false
  }
}

/**
 * Đo thời gian từ lúc gọi tới lúc gọi hàm trả về.
 *
 * Dùng cho "thời gian vẽ lần đầu": gọi khi bắt đầu nạp trang đầu, gọi hàm trả
 * về trong `useEffect` của lần render đầu tiên **thật sự có dữ liệu**. Đo tới
 * lúc `commits.length > 0` chứ không tới lúc `useEffect` đầu tiên chạy, vì cái
 * sau xảy ra khi màn hình còn trống — đo nhầm sẽ cho một con số đẹp vô nghĩa.
 *
 * Cờ tắt → trả hàm rỗng, không chạm `performance`.
 *
 * @param label nhãn tĩnh. **Không** truyền đường dẫn repo vào đây (T-02-24).
 * @returns hàm "xong"; gọi nó ghi ra console một dòng `[perf]` và trả số ms.
 */
export function measureFirstPaint(label: string): () => number {
  if (!isPerfEnabled()) return () => 0

  const batDau = performance.now()
  const markBatDau = `git-plum:${label}:start`
  try {
    performance.mark(markBatDau)
  } catch {
    // `performance.mark` có thể không có trong môi trường test rút gọn. Phép đo
    // bằng `performance.now()` bên dưới vẫn chạy được, nên nuốt lỗi ở đây.
  }

  let daGoi = false
  return () => {
    // Chặn gọi hai lần: React StrictMode chạy effect hai lượt lúc phát triển, và
    // lần thứ hai sẽ ghi một con số lớn hơn vào console rồi người chép số lấy
    // nhầm. Lần gọi thừa trả lại đúng số của lần đầu.
    if (daGoi) return 0
    daGoi = true

    const ms = performance.now() - batDau
    try {
      performance.measure(`git-plum:${label}`, markBatDau)
    } catch {
      // như trên
    }
    console.info(`${PREFIX} ${label}: ${ms.toFixed(1)}ms`)
    return ms
  }
}

/**
 * Đếm khung trong `durationMs` mili giây rồi trả báo cáo FPS.
 *
 * Người gọi cuộn `el` bằng tay trong lúc đo (kéo thanh cuộn, lăn chuột, giữ
 * PageDown — ba kiểu sinh tần suất sự kiện khác nhau, và checkpoint #1 yêu cầu
 * đo cả ba vì kiểu tệ nhất mới là kiểu người dùng nhớ).
 *
 * Tự dừng sau `durationMs` (T-02-25): không có đường nào để vòng đo sống mãi.
 *
 * @param el phần tử cuộn. Hiện chỉ dùng để giữ đúng chữ ký và cho phép bản sau
 *   gắn thêm listener; phép đếm khung là toàn cục theo `requestAnimationFrame`.
 * @param durationMs độ dài khoảng đo.
 */
export function measureScrollFps(el: HTMLElement, durationMs: number): Promise<FpsReport> {
  // Cờ tắt → không đăng ký khung nào. Đây là điều kiện T-02-25 và có test ghim
  // bằng cách đếm số lần `requestAnimationFrame` được gọi (phải là 0).
  if (!isPerfEnabled()) return Promise.resolve({ ...BAO_CAO_RONG })
  if (durationMs <= 0) return Promise.resolve({ ...BAO_CAO_RONG })
  void el

  return new Promise<FpsReport>((resolve) => {
    const khoangCach: number[] = []
    const batDau = performance.now()
    let truoc = batDau

    const buoc = (now: number) => {
      khoangCach.push(now - truoc)
      truoc = now

      if (now - batDau >= durationMs) {
        resolve(tinhBaoCao(khoangCach, now - batDau))
        return
      }
      requestAnimationFrame(buoc)
    }

    requestAnimationFrame(buoc)
  })
}

/**
 * Quy các khoảng-giữa-khung thành báo cáo.
 *
 * Tách khỏi `measureScrollFps` để test được phép tính trên dữ liệu khung dựng
 * sẵn, không phụ thuộc `requestAnimationFrame` thật — thời gian thật không lặp
 * lại được nên một test dựa vào nó sẽ chập chờn.
 *
 * `p1Fps` lấy theo phân vị 1 **từ đầu tệ**: sắp xếp khoảng cách giảm dần rồi
 * lấy phần tử ở vị trí `ceil(1% * n) - 1`. Với ít khung thì rơi về đúng khoảng
 * cách dài nhất, và đó là hành vi mong muốn — mẫu nhỏ thì khung tệ nhất chính
 * là ước lượng bi quan hợp lý.
 */
export function tinhBaoCao(khoangCach: number[], tongMs: number): FpsReport {
  const frames = khoangCach.length
  if (frames === 0 || tongMs <= 0) return { ...BAO_CAO_RONG }

  const giamDan = [...khoangCach].sort((a, b) => b - a)
  const viTriP1 = Math.max(0, Math.ceil(frames * 0.01) - 1)
  const khoangP1 = giamDan[viTriP1] ?? 0
  const daiNhat = giamDan[0] ?? 0

  return {
    frames,
    durationMs: tongMs,
    avgFps: (frames * 1000) / tongMs,
    p1Fps: khoangP1 > 0 ? 1000 / khoangP1 : 0,
    longestFrameMs: daiNhat,
  }
}
