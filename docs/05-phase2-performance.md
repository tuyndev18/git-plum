# Phase 2 — Hiệu năng: số đo của validation checkpoint #1 và #4

**Trạng thái:** 🟡 **CHƯA ĐÓNG.** Phần đo phía Rust đã có số thật. Phần đo
**đầu-tới-cuối** (thời gian vẽ lần đầu người dùng cảm nhận, FPS lúc cuộn) **chưa đo**
— cần một lượt chạy tay trên bản release theo checkpoint #1, Task 2 của plan 02-07.

**Nguyên tắc của tài liệu này:** mỗi ô hoặc mang một con số đo được, hoặc nói thẳng
rằng nó chưa được đo và ai phải đo. **Không ô nào chứa số ước lượng, số nội suy, hay
số chép từ một phép đo khác rồi dán nhãn khác.** Phase 1 đã kết luận "đạt phần tự động
hoá được" trong khi ba tiêu chí chưa ai chạy; tài liệu này tồn tại để Phase 2 không lặp
lại kiểu đó với Core Value.

---

## 0. Cấu hình máy đo

Thiếu cấu hình máy thì con số không so lại được sau này, và một con số không so lại
được thì gần như vô dụng (T-02-26).

| Hạng mục | Giá trị |
|---|---|
| CPU | 11th Gen Intel Core i5-11400 @ 2.60GHz (6 nhân / 12 luồng) |
| RAM | 25 632 780 288 byte ≈ **23.9 GiB** |
| Hệ điều hành | Windows 11 Pro 10.0.22621 |
| Phiên bản git | **2.54.0.windows.1** |
| Repo đo | `target/fixtures-perf/perf-100k`, sinh bằng `scripts/fixtures/make-perf-repo.sh` |
| Số commit thật | **100 007** (`git rev-list --all --count`) |
| Ngày đo (phần Rust) | 2026-09-21 |
| Ngày đo (phần đầu-tới-cuối) | ⏳ chưa đo |

> ⚠️ Đĩa là OneDrive-synced folder (`C:\Users\tuyen\OneDrive\Desktop\git-plum`). Đây là
> biến số thật cho phép đo I/O: OneDrive có thể chen vào đường đọc tệp. Khi đo lại ở
> máy khác, ghi rõ repo nằm trên đĩa thường hay thư mục đồng bộ — chênh lệch `git log`
> giữa hai môi trường này không phải nhiễu ngẫu nhiên.

---

## 1. Checkpoint #1 — hiệu năng repo lớn

### 1.1 Quy tắc kết luận — chốt TRƯỚC khi xem số

Ghi lại nguyên văn từ plan 02-07 để không ai nới mốc sau khi đã thấy kết quả:

> **Đạt** nếu trung vị thời gian vẽ lần đầu **< 1000ms** *và* `p1Fps` **≥ 50** ở cả ba
> kiểu cuộn *và* `longestFrameMs` **≤ 100**.

Cả ba điều kiện, không phải một. Trượt thì **không** sửa mốc — đường lùi đã định sẵn
trong ROADMAP: nâng bộ nạp commit lên `tauri::ipc::Channel` để React vẽ 500 dòng đầu
trong khi Rust còn phân tích phần còn lại.

**Mốc tham chiếu:** `gitlanes` — cùng stack Tauri 2 + React, canvas + virtual scroll —
báo 32k commit nạp trong ~300ms ở 60fps.

### 1.2 Phân rã phía Rust — ✅ ĐÃ ĐO

Đo bằng `cargo test --release --test history_budget -- --nocapture --ignored`,
profile **release**, repo 100 007 commit, ngày 2026-09-21.

| Đoạn | Thời gian | Ghi chú |
|---|---:|---|
| (a) chạy `git log --all --topo-order` + đọc hết stdout | **633.8 ms** | 84.5% tổng đường nóng |
| (b) `parse_log` | **63.1 ms** | 8.4% |
| (c) `lanes::assign` | **51.9 ms** | 6.9% |
| cache put + get + cắt trang + JSON một trang 100 commit | **0.86 ms** | 0.1% |
| **Tổng đường nóng phía Rust** | **749.7 ms** | mốc 1000 ms |
| Trang thứ hai (cache hit, không sinh git) | **0.646 ms** | |

**Điều quan trọng nhất đọc được từ bảng này:** `git log` chiếm **84.5%**. Tối ưu
`parse_log` và `assign` — cộng lại chỉ 15.3% — không thể đổi được con số người dùng
cảm nhận một cách đáng kể. Nếu cần rút ngắn, chỗ phải động vào là (a), và đường lùi
`Channel` tấn công đúng chỗ đó: nó không làm `git log` nhanh hơn, nó cho React vẽ
trong lúc `git log` còn đang chạy.

Hệ quả thứ hai: benchmark `criterion` của plan 02-03 chỉ đo (b) và (c), tức đang đo
15.3% của bài toán. Nó vẫn có ích để bắt hồi quy thuật toán, nhưng **không** phải
thước đo của Core Value.

> ⚠️ **Profile release là bắt buộc.** Cùng máy, cùng repo, profile debug cho
> `parse_log` 455 ms và `assign` 220 ms — chậm gần 7 lần và làm tổng vượt mốc một
> giây. Đọc số debug rồi kết luận "trượt Core Value" là sai.

### 1.3 Thời gian vẽ lần đầu, đầu-tới-cuối — ⏳ CHƯA ĐO

Đây là con số **thật sự** đóng được checkpoint #1, và nó **chưa tồn tại**. Bảng 1.2 chỉ
đo phía Rust; nó **không** gồm IPC (serialize + qua cầu + deserialize) và **không** gồm
thời gian React dựng DOM và canvas vẽ khung đầu.

| Lần đo | Thời gian (ms) | Ghi chú |
|---|---:|---|
| Lần 1 (đĩa nguội) | ⏳ chưa đo | tính riêng, không vào trung vị |
| Lần 2 | ⏳ chưa đo | |
| Lần 3 | ⏳ chưa đo | |
| Lần 4 | ⏳ chưa đo | |
| Lần 5 | ⏳ chưa đo | |
| **Trung vị lần 2–5** | ⏳ **chưa đo** | so với mốc **< 1000 ms** |

**Ai đo và đo thế nào:** chủ dự án, theo bảy bước trong Task 2 của
`.planning/phases/02-history-graph/02-07-PLAN.md`. Tóm tắt đường chạy:

```powershell
# 1. Dựng bản đo (KHÔNG dùng `tauri dev` — WebView2 chế độ dev có công cụ gỡ lỗi
#    gắn vào và số sẽ đẹp hơn thực tế một cách sai lệch)
npx tauri build --debug --no-bundle

# 2. Bật cờ đo phía Rust rồi chạy tệp .exe sinh ra
$env:GIT_PLUM_PERF = "1"
$env:RUST_LOG = "git_plum_lib=info"
.\src-tauri\target\debug\git-plum.exe
```

Trong webview (DevTools console), bật cờ phía giao diện rồi mở repo:

```js
localStorage.gitPlumPerf = '1'   // rồi tải lại cửa sổ
```

Mở `target/fixtures-perf/perf-100k`. Dòng cần chép có dạng:

```
[perf] commit-list-first-paint: <số>ms
```

**Phần dư IPC + render** = (trung vị ở bảng này) − 749.7 ms. Đó chính là đầu vào của
checkpoint #4 ở mục 2.2, nên không bỏ qua được.

### 1.4 FPS lúc cuộn — ⏳ CHƯA ĐO

Đo bằng lệnh đã phơi sẵn trên `window` (chỉ gắn khi cờ bật):

```js
await window.gitPlumMeasureScrollFps(10000)
// -> { frames, durationMs, avgFps, p1Fps, longestFrameMs }
```

Cuộn liên tục trong suốt 10 giây đo. **Ba kiểu cuộn riêng**, vì chúng sinh tần suất sự
kiện khác nhau và kiểu tệ nhất mới là kiểu người dùng nhớ:

| Kiểu cuộn | `avgFps` | `p1Fps` (mốc ≥ 50) | `longestFrameMs` (mốc ≤ 100) |
|---|---:|---:|---:|
| Kéo thanh cuộn nhanh | ⏳ chưa đo | ⏳ chưa đo | ⏳ chưa đo |
| Lăn chuột | ⏳ chưa đo | ⏳ chưa đo | ⏳ chưa đo |
| Giữ PageDown | ⏳ chưa đo | ⏳ chưa đo | ⏳ chưa đo |

**Vì sao `p1Fps` và `longestFrameMs` mới là số quyết định, không phải `avgFps`:** một
lần cuộn có `avgFps` 58 mà chứa đúng một khung 400 ms thì người dùng thấy khựng rất rõ,
nhưng `avgFps` vẫn nằm trên mốc 50. Có test dựng đúng ca đó trong `src/lib/perf.test.ts`
(`một khung 400ms lẫn trong 99 khung đẹp`).

### 1.5 Thẳng hàng ở quy mô 100k (HIST-04) — ⏳ CHƯA KIỂM

Plan 02-05 chỉ kiểm thẳng hàng trên repo nhỏ. Ở 100k commit, cuộn tới giữa rồi tới cuối
(Ctrl+End) và xác nhận đồ thị còn thẳng hàng với từng hàng commit.

| Vị trí cuộn | Kết quả |
|---|---|
| Đầu danh sách | ⏳ chưa kiểm |
| Khoảng giữa | ⏳ chưa kiểm |
| Cuối (Ctrl+End) | ⏳ chưa kiểm |

### 1.6 Kết luận checkpoint #1

⏳ **CHƯA KẾT LUẬN ĐƯỢC.** Thiếu trung vị thời gian vẽ lần đầu (1.3) và ba bộ số FPS
(1.4). Phần Rust (749.7 ms, còn dư ~250 ms so với mốc 1000 ms) là **dấu hiệu thuận
lợi**, không phải kết luận: phần dư đó phải gánh cả IPC lẫn render, và chưa ai biết nó
tốn bao nhiêu.

---

## 2. Checkpoint #4 — IPC JSON hay nhị phân cho dữ liệu lane

### 2.1 Kích thước payload JSON — 🟡 ĐO MỘT PHẦN

Đã đo trên trang **100 commit** (từ `history_budget.rs`, cắt ở giữa lịch sử):

| Hạng mục | Giá trị |
|---|---:|
| Payload JSON một trang 100 commit | **77 657 byte** (~777 byte/commit) |
| Thời gian serialize trang đó | **0.808 ms** |

⏳ **Chưa đo ở `PAGE_SIZE` thật là 1000**, và **chưa tách riêng** `commits` với
`graph_rows`. Hạ tầng đo đã có (Task 1 thêm log `payload_bytes`, `commits_bytes`,
`graph_rows_bytes`, `graph_rows_percent` vào `get_commit_page`); nó chỉ chưa được chạy
trên ứng dụng thật.

| Hạng mục | Giá trị |
|---|---:|
| Payload JSON một trang **1000** commit | ⏳ chưa đo |
| Riêng `commits` | ⏳ chưa đo |
| Riêng `graph_rows` | ⏳ chưa đo |
| **Tỉ lệ `graph_rows` trong payload** | ⏳ **chưa đo** |

### 2.2 Phần của IPC trong tổng thời gian — ⏳ CHƯA ĐO

Tính bằng: (trung vị mục 1.3) − 749.7 ms. Cần mục 1.3 trước.

| Hạng mục | Giá trị |
|---|---:|
| Phần dư IPC + render | ⏳ chưa đo |

### 2.3 Bộ nhớ thường trú — ⏳ CHƯA ĐO

Plan 02-04 **ước tính** `RepoHistory` ≈ 67 MB cho repo 100k commit (`Vec<GraphRow>`
~17 MB + `Vec<Commit>` ~50 MB), so với ràng buộc **RAM lúc rảnh dưới 150 MB** trong
`PROJECT.md`. Đó là ước tính từ `size_of` và nó **bỏ qua phân mảnh malloc trên ~700k
allocation rời** — nên phải đo thật, không được dùng con số ước tính làm kết luận.

Đo trong Task Manager tab Details, cột **Memory (private working set)**:

| Mốc | `git-plum.exe` | `msedgewebview2.exe` (tổng) | Tổng hai |
|---|---:|---:|---:|
| Baseline (mở app, chưa mở repo) | ⏳ chưa đo | ⏳ chưa đo | ⏳ chưa đo |
| Sau khi nạp xong repo 100k | ⏳ chưa đo | ⏳ chưa đo | ⏳ chưa đo |
| **Hiệu số (chi phí thật của cache)** | ⏳ chưa đo | ⏳ chưa đo | ⏳ chưa đo |

**Đối chiếu với hạn 150 MB của `PROJECT.md`:** ⏳ chưa kết luận được.

> **Nếu vượt hạn:** đây là một ràng buộc đã ghi của dự án bị phá, không phải chuyện nhỏ
> bỏ qua được. **Không tự ý nới hạn 150 MB.** Hai đường xử lý đã biết, nêu kèm chi phí:
> **(a)** bỏ `body` khỏi cache và lấy lười theo từng commit khi người dùng chọn — cắt
> phần lớn của ~28 MB heap trong `Vec<Commit>`, đổi lại mỗi lần chọn commit tốn một lần
> đọc; **(b)** giảm `MAX_CACHED_HISTORIES` xuống 1 — đơn giản hơn nhưng làm việc chuyển
> qua lại giữa hai repo phải nạp lại từ đầu.

### 2.4 Quyết định JSON hay nhị phân

⏳ **CHƯA QUYẾT ĐƯỢC.** Quy tắc quyết, chốt trước khi có số:

> Chuyển `graph_rows` sang `tauri::ipc::Response` (byte thô) **chỉ khi** phần dư IPC ở
> mục 2.2 là **thành phần lớn nhất** của tổng thời gian **và** `graph_rows` chiếm
> **phần lớn** payload ở mục 2.1. **Cả hai** điều kiện, không phải một.

Dấu hiệu hiện có nghiêng về **giữ JSON**, nhưng đây *không* phải kết luận: phía Rust
serialize một trang 100 commit hết 0.808 ms, tức 0.1% đường nóng, trong khi `git log`
chiếm 84.5%. Nếu phần dư IPC ở 2.2 cũng nhỏ, thì chuyển sang nhị phân là **tối ưu sai
chỗ** — tốn công mà không đổi được con số người dùng cảm nhận. Phải có số ở 2.1 và 2.2
mới ghi được kết luận vào đây.

### 2.5 🔴 Đính chính bắt buộc: `Channel` **vẫn là JSON**

`tauri::ipc::Channel` **không** phải đường nhị phân nhanh. Nó cho **thứ tự** và **chia
khối**, và mọi thông điệp vẫn đi qua `Serialize` → JSON (`impl<T> IpcResponse for T
where T: Serialize`).

Đường nhị phân **thật** là `tauri::ipc::Response` trả mảng byte thô. Nó hợp với
`GraphRow` vì đó là dữ liệu số **cố định chiều rộng** — trường hợp tệ nhất cho JSON và
tốt nhất cho typed array feeding thẳng vào canvas.

**Hai thứ này giải hai bài toán khác nhau, đừng lẫn:**

| | Giải bài toán gì | Thuộc checkpoint nào |
|---|---|---|
| `ipc::Channel` | vẽ sớm những dòng đầu trong khi phần còn lại còn đang phân tích (**độ trễ cảm nhận**) | đường lùi của **#1** |
| `ipc::Response` | giảm chi phí serialize/kích thước của dữ liệu số (**thông lượng**) | lựa chọn của **#4** |

Dùng `Channel` với hy vọng nó nhanh hơn về mặt serialize là đi sai đường. Đây là hiểu
nhầm phổ biến mà `CONTEXT.md` nêu đích danh.

### 2.6 Kích thước lô

`PAGE_SIZE` hiện là **1000** commit một thông điệp, theo ROADMAP ("1000 commit một
thông điệp, không phải 1"). ⏳ Chưa có số ở 2.1 và 2.2 để nói nên giữ hay đổi.

### 2.7 Điều kiện xem lại nếu giữ JSON

Một quyết định có điều kiện hết hạn thì hữu ích hơn một quyết định trống. Nếu kết luận
ở 2.4 là giữ JSON, xem lại khi **bất kỳ** điều nào sau xảy ra:

- `PAGE_SIZE` tăng lên ~10 000 (chi phí serialize tăng tuyến tính, phần dư IPC có thể
  vượt qua `git log` để thành thành phần lớn nhất);
- Phase 3 thêm dữ liệu vào **cùng** payload (ví dụ đính kèm danh sách tệp thay đổi theo
  từng commit);
- `graph_rows` đổi hình dạng sang nhiều trường số hơn (tỉ lệ ở 2.1 dịch chuyển);
- Có người báo giật lúc cuộn trên repo lớn **sau khi** đã loại trừ (a) `git log`.

---

## 3. Việc còn phải làm để đóng hai checkpoint

| # | Việc | Ai | Chặn cái gì |
|---|---|---|---|
| 1 | Đo thời gian vẽ lần đầu, 5 lần, bản release, repo 100k (mục 1.3) | chủ dự án | checkpoint #1, tiêu chí thành công số 1 |
| 2 | Đo FPS ba kiểu cuộn (mục 1.4) | chủ dự án | checkpoint #1 |
| 3 | Kiểm thẳng hàng ở giữa và cuối repo 100k (mục 1.5) | chủ dự án | HIST-04 ở quy mô lớn |
| 4 | Ghi ba số payload ở `PAGE_SIZE`=1000 (mục 2.1) | tự động, khi app chạy với cờ | checkpoint #4 |
| 5 | Đo bộ nhớ thường trú baseline + sau nạp (mục 2.3) | chủ dự án | ràng buộc RAM 150 MB |
| 6 | Viết kết luận 1.6 và 2.4 từ số thu được | — | đóng phase |

---
*Tài liệu tạo: 2026-09-21. Phần Rust đo cùng ngày; phần đầu-tới-cuối chờ checkpoint #1.*
