---
phase: 02-history-graph
plan: 07
subsystem: performance-measurement
tags: [HIST-05, HIST-04, HIST-03, perf, checkpoint, validation]
requires:
  - "02-05 — CommitList + canvas, nơi gắn dụng cụ đo"
  - "02-06 — bề mặt hiển thị đủ để đo trên app thật"
provides:
  - "src/lib/perf.ts — measureFirstPaint, measureScrollFps, isPerfEnabled"
  - "window.gitPlumMeasureScrollFps(ms) khi cờ bật"
  - "docs/05-phase2-performance.md — khung kết luận + số Rust thật"
  - ".planning/phases/02-history-graph/VERIFICATION.md — trạng thái 11 requirement"
affects:
  - "Phase 3 (diff): dùng lại perf.ts cho phép đo mở diff tệp lớn"
  - "Quyết định JSON-hay-nhị-phân (checkpoint #4) HOÃN — cần phần dư IPC từ checkpoint #1"
tech-stack:
  added: []
  patterns:
    - "cờ đo bật được LÚC CHẠY qua localStorage, không chỉ lúc dựng — đo được trên bản release"
    - "đo tới lần render đầu tiên THẬT SỰ CÓ DỮ LIỆU, không tới useEffect đầu tiên"
    - "chặn gọi hai lần: StrictMode chạy effect hai lượt, lần hai cho số lớn hơn"
    - "ô chưa đo ghi '⏳ chưa đo', KHÔNG lấp bằng số ước lượng hay nội suy"
key-files:
  created:
    - "src/lib/perf.ts"
    - "docs/05-phase2-performance.md"
    - ".planning/phases/02-history-graph/VERIFICATION.md"
  modified:
    - "src/components/history/CommitList.tsx — gắn measureFirstPaint + lộ gitPlumMeasureScrollFps"
decisions:
  - "Cờ đo đọc localStorage LÚC CHẠY: checkpoint #1 bắt buộc đo trên bản release, mà dev mode có devtools gắn vào làm số đẹp sai lệch"
  - "measureFirstPaint đo tới commits.length > 0, không tới useEffect đầu — cái sau xảy ra khi màn hình còn trống"
  - "Ngưỡng kết luận chốt TRƯỚC khi xem số: <1000ms, p1Fps>=50, longestFrameMs<=100"
  - "Checkpoint #1 BỎ QUA theo quyết định chủ dự án — tài liệu ghi '⏭️ bỏ qua', không lấp số"
  - "Checkpoint #4 HOÃN: quy tắc là phép AND hai điều kiện, điều kiện (1) cần phần dư IPC từ checkpoint #1 nên không đánh giá được → giữ JSON, ghi rõ là hoãn chứ không phải kết luận"
  - "67MB/lịch sử dán nhãn ƯỚC TÍNH từ size_of, hạn 150MB ở trạng thái CHƯA KIỂM CHỨNG"
  - "Đính chính: ipc::Channel VẪN LÀ JSON — đường nhị phân thật là ipc::Response"
metrics:
  duration: "~45 phút (Task 1-3) + đo lại sau khi repo fixture bị dọn"
  completed: "2026-09-21"
  tasks: 3
  commits: 4
  cargo_test: "149 đỗ + 1 ignored (không đổi)"
  npm_test: "212 đỗ"
  hot_path_rust_release: "749.7ms (lần 1) / 762.2ms (lần 2), mốc 1000ms"
---

# Phase 2 Plan 07: Đo hiệu năng — Summary

Plan này **không tối ưu gì**. Nó dựng dụng cụ sinh số thật cho validation checkpoint #1
và #4, đo phần đo được (phía Rust), và ghi phần chưa đo **đúng là chưa đo**.

Kết quả thẳng thắn: **checkpoint #1 chưa chạy**, nên requirement HIST-05 vẫn `Pending` và
Core Value của dự án — "đồ thị mở dưới một giây trên 100k commit" — **chưa được kiểm
chứng đầu-tới-cuối**.

## Số đã đo được

Phía Rust, profile **release**, repo 100 007 commit, **hai lần đo độc lập**:

| Đoạn | Lần 1 | Lần 2 | % |
|---|---:|---:|---:|
| `git log --all --topo-order` + đọc stdout | 633.8 ms | 640.4 ms | **84%** |
| `parse_log` | 63.1 ms | 66.9 ms | 8.8% |
| `lanes::assign` | 51.9 ms | 54.0 ms | 7.1% |
| cache + cắt trang + JSON (100 commit) | 0.86 ms | 0.88 ms | 0.1% |
| **Tổng phía Rust** | **749.7 ms** | **762.2 ms** | mốc 1000 ms |
| Trang thứ hai (cache hit) | 0.646 ms | 0.732 ms | |

Lần 2 đo lại sau khi thư mục fixture bị dọn mất và repo được **sinh lại từ đầu** bằng
cùng `make-perf-repo.sh` (cùng hình dạng: 100 007 commit, 3 182 merge, 320 octopus).
Chênh 1.7% — trong nhiễu, nên cả hai dùng được.

**Điều đáng giá nhất:** `git log` chiếm **84%**. Hai hệ quả:

1. Tối ưu `parse_log` và `assign` (cộng lại 16%) không đổi được con số người dùng cảm
   nhận. Nếu phép đo thật trượt mốc, chỗ phải động vào là khâu **nạp** — và đường lùi
   `Channel` tấn công đúng chỗ đó: nó không làm `git log` nhanh hơn, nó cho React vẽ
   *trong lúc* `git log` còn chạy.
2. Benchmark `criterion` của plan 02-03 chỉ đo `parse_log` + `assign`, tức **16% của bài
   toán**. Vẫn hữu ích để bắt hồi quy thuật toán, nhưng **không phải** thước đo Core Value.

## Số CHƯA đo — và tại sao không lấp

22 ô trong `docs/05-phase2-performance.md` ghi `⏳ chưa đo` hoặc `⏭️ bỏ qua`. **Không ô
nào** chứa số ước lượng, nội suy, hay chép từ phép đo khác. Những thứ còn thiếu:

- **Thời gian vẽ lần đầu đầu-tới-cuối** — con số duy nhất đóng được checkpoint #1
- **FPS ba kiểu cuộn** (kéo thanh cuộn / lăn chuột / giữ PageDown — ba tần suất sự kiện
  khác nhau, và kiểu tệ nhất mới là kiểu người dùng nhớ)
- **Thẳng hàng ở giữa/cuối repo 100k**
- **Payload ở `PAGE_SIZE=1000`**, tách riêng `commits` và `graph_rows`
- **Bộ nhớ thường trú** so với hạn 150 MB

Điểm dễ chép nhầm nhất, đã ghi rõ trong tài liệu: **762 ms là phía Rust**, chưa gồm IPC và
render React, nên **không phải** thời gian vẽ lần đầu. Ai chép nó thành kết luận checkpoint
#1 là sai.

Quan sát định tính duy nhất có được (repo 4 037 commit, cuộn "khá mượt", bằng mắt) đã ghi
kèm **ba giới hạn chính xác của nó**: 4k là ~1/25 của mốc 100k; "khá mượt" không phải
`p1Fps`; không biết đo trên bản dựng nào.

## Dụng cụ đo — ba quyết định thiết kế

**1. Cờ bật được lúc chạy, không chỉ lúc dựng.** `VITE_GSD_PERF=1` cho bản dựng chuyên
dụng, `localStorage.gitPlumPerf = '1'` cho **bản release đã dựng sẵn**. Đường thứ hai là
bắt buộc: checkpoint #1 phải đo trên release, mà bản dev có devtools gắn vào nên số sẽ đẹp
hơn thực tế một cách sai lệch. Cùng lý do đó, phép đo debug hoàn toàn vô nghĩa ở đây —
cùng máy, cùng repo, debug cho `parse_log` **455 ms** và `assign` **220 ms**, chậm gần 7
lần và làm tổng vượt mốc một giây.

**2. Đo tới lần render đầu tiên THẬT SỰ CÓ DỮ LIỆU** (`commits.length > 0`), không tới
`useEffect` đầu tiên — cái sau xảy ra khi màn hình còn trống, đo nhầm sẽ cho một con số
đẹp vô nghĩa.

**3. Chặn gọi hai lần.** React StrictMode chạy effect hai lượt lúc phát triển và lần thứ
hai ghi một số **lớn hơn** vào console; người chép số sẽ lấy nhầm. Lần gọi thừa trả lại
đúng số của lần đầu.

Cờ tắt → `measureFirstPaint` trả hàm rỗng, `measureScrollFps` trả báo cáo rỗng, không
đăng ký khung nào (T-02-25 — không có đường nào để vòng đo sống mãi; tự dừng sau
`durationMs`).

## Checkpoint #4 — HOÃN, không phải kết luận

Quy tắc chốt sẵn là **phép AND hai điều kiện**. Điều kiện (1) cần **phần dư IPC** =
(thời gian vẽ lần đầu) − (762 ms phía Rust). Checkpoint #1 không chạy nên phép trừ đó
không thực hiện được, nên phép AND không sinh ra quyết định "chuyển sang nhị phân".

Kết quả: **giữ JSON**, và tài liệu ghi rõ đây là quyết định **hoãn**, không phải kết luận
có số liệu. Thêm điều kiện xem lại: trả nợ chính hai phép đo còn thiếu.

**Một đính chính kỹ thuật giữ nguyên từ bản trước:** `ipc::Channel` **vẫn là JSON** — nó
giải bài toán *độ trễ cảm nhận* (vẽ dần trong lúc nạp), không phải bài toán *kích thước
payload*. Đường nhị phân thật là `ipc::Response` với `Uint16Array` đóng gói. Hai thứ giải
hai bài toán khác nhau; lẫn chúng vào nhau sẽ chọn sai công cụ.

## Bộ nhớ — chưa kiểm chứng

Con số ~67 MB mỗi lịch sử (từ đó suy ra `MAX_CACHED_HISTORIES = 2`) là **ước tính** tính
từ `size_of::<GraphRow>()` = 80 byte cộng `Vec<Commit>`, **không phải** đo RSS thật. Nên
hạn 150 MB của `PROJECT.md` đang ở trạng thái **chưa kiểm chứng** — tài liệu không đánh
"đạt" cũng không đánh "vượt".

## Việc ngoài plan đã làm trong wave này

**Fixtures đã mất hết và được tạo lại.** Cả `target/fixtures/` (9 repo mẫu) lẫn
`target/fixtures-perf/` đều trống khi kiểm — không rõ bị dọn từ lúc nào. Đã chạy lại
`make-fixtures.sh` và `make-perf-repo.sh`; phép đo lần 2 ở trên là trên repo sinh lại.
Điều này giải thích vì sao phải đo hai lần.

**Sửa lỗi đánh số tài liệu của chính tôi.** Tôi từng tạo `docs/05-ui-reference-gap.md`
trong khi số `05` đã thuộc `05-phase2-performance.md`. Đã đổi thành
`docs/07-ui-reference-gap.md` và cập nhật 3 chỗ tham chiếu (`02-06-SUMMARY.md`,
`STATE.md`).

## Trạng thái requirement

Xem `VERIFICATION.md` cho bằng chứng từng requirement. Tóm tắt:

- **Đạt (3):** HIST-01, HIST-02, HIST-11
- **Đạt một phần (1):** HIST-04 — bất biến dữ liệu và hệ quy chiếu toạ độ có test chặn,
  nhưng giữa/cuối repo 100k chưa kiểm bằng mắt
- **Có mã, chưa kiểm (6):** HIST-03, 06, 07, 08, 09, 10
- **Chưa đo (1):** HIST-05

Ba lỗi thẳng hàng đã gặp trong phase này (badge phá chiều cao hàng; đường vẽ tràn sang ô
hàng dưới; sai hệ quy chiếu toạ độ canvas) **đều không bị test tự động nào bắt** —
happy-dom không tính layout CSS và không có cuộn thật. Đó là lập luận cụ thể, có bằng
chứng, cho việc hai checkpoint người kiểm không thể bỏ.
