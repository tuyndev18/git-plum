# Phase 2 — Hiệu năng: số đo của validation checkpoint #1 và #4

**Trạng thái:** 🔴 **KHÔNG ĐÓNG ĐƯỢC — checkpoint #1 bị BỎ QUA theo quyết định của chủ
dự án ngày 2026-09-21.** Phần đo phía Rust đã có số thật. Phần đo **đầu-tới-cuối**
(thời gian vẽ lần đầu người dùng cảm nhận, FPS lúc cuộn, bộ nhớ thường trú) **không
bao giờ được chạy**. Phase 2 đóng lại với tiêu chí Core Value **chưa kiểm chứng**.

**Nguyên tắc của tài liệu này:** mỗi ô hoặc mang một con số đo được, hoặc nói thẳng
rằng nó không được đo và vì sao. **Không ô nào chứa số ước lượng, số nội suy, hay số
chép từ một phép đo khác rồi dán nhãn khác.** Phase 1 đã kết luận "đạt phần tự động
hoá được" trong khi ba tiêu chí chưa ai chạy; tài liệu này tồn tại để Phase 2 **không
giả vờ** rằng nó làm khác đi.

---

## Quyết định bỏ qua checkpoint #1 — ghi nguyên văn

**Ngày:** 2026-09-21. **Người quyết:** chủ dự án.

Chủ dự án chọn **bỏ qua** phép đo của checkpoint #1 (dựng bản release, mở repo
`perf-100k`, đo năm lần thời gian vẽ lần đầu, đo FPS ba kiểu cuộn, đo bộ nhớ trong
Task Manager). Lý do nêu ra: *"thế khó tạm bỏ qua đi tôi thấy kéo xuống cũng khá mượt
với repo quan-ly-truong-phong-so"*.

Checkpoint này vì vậy **không đạt và cũng không trượt — nó bị bỏ qua.** Ba trạng thái
này khác nhau và tài liệu không được trộn chúng: "đạt" cần số, "trượt" cũng cần số, chỉ
"bỏ qua" là không cần gì cả — và cũng không cho biết gì cả.

### Quan sát định tính duy nhất có được, kèm giới hạn chính xác của nó

| Hạng mục | Giá trị |
|---|---|
| Repo | `D:/MyCompanyProjects/quanly-truong-phong-so` |
| Số commit thật | **4 037** (`git rev-list --all --count`, đã chạy để xác nhận) |
| Quan sát | cuộn "**khá mượt**" |
| Cách quan sát | **bằng mắt**, không dụng cụ đo, không con số |
| Bản dựng | **không rõ** — nhiều khả năng `tauri dev`, không phải release |
| `avgFps` / `p1Fps` / `longestFrameMs` | **không có** |

**Quan sát này chứng minh được gì:** ở quy mô ~4k commit, không có gì hỏng thảm hoạ.
Danh sách ảo hoá chạy, đồ thị vẽ được, cuộn không đứng hình.

**Quan sát này KHÔNG chứng minh được gì — và đây mới là phần quan trọng:**

1. **4 037 commit là khoảng 1/25 của mốc 100 000.** Đây đúng là cái bẫy "chạy thử trên
   repo nhỏ thấy ổn" mà plan 02-07 nêu đích danh. `git log` trên 4k commit tốn khoảng
   một phần hai mươi lăm thời gian của 100k; cache ~67 MB của repo 100k ở đây chỉ còn
   vài MB. Không có gì trong phép quan sát này chạm tới vùng mà kiến trúc có thể vỡ.
2. **"Khá mượt" không phải `p1Fps`.** "Cuộn không giật" trong tiêu chí thành công số 1
   là chuyện của **khung tệ nhất**, không phải cảm giác trung bình. Một lượt cuộn có
   `avgFps` 58 kèm đúng một khung 400 ms vẫn cho cảm giác "khá mượt" ở phần lớn thời
   gian, trong khi `longestFrameMs` = 400 đã vượt mốc 100 ms gấp bốn lần. Mắt người
   không đọc được `p1Fps`; đó chính là lý do `perf.ts` tồn tại.
3. **Không biết đo trên bản dựng nào.** Nếu là `tauri dev` thì WebView2 có công cụ gỡ
   lỗi gắn vào và số lệch theo hướng khó đoán; và profile debug của Rust đã được đo là
   chậm gần **7 lần** (xem cảnh báo ở mục 1.2).

**Vì vậy quan sát này được ghi lại như một quan sát định tính ở quy mô 4k, và không
bao giờ được dùng thay cho phép đo ở 100k.** Không ô nào trong các bảng dưới đây được
điền bằng nó.

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
| Ngày đo (phần đầu-tới-cuối) | ⏭️ **không có** — checkpoint bị bỏ qua 2026-09-21 |

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
| Đoạn | Lần 1 | Lần 2 | Ghi chú |
|---|---:|---:|---|
| (a) chạy `git log --all --topo-order` + đọc hết stdout | **633.8 ms** | **640.4 ms** | 84–84.5% tổng đường nóng |
| (b) `parse_log` | **63.1 ms** | **66.9 ms** | 8.4–8.8% |
| (c) `lanes::assign` | **51.9 ms** | **54.0 ms** | 6.9–7.1% |
| cache put + get + cắt trang + JSON một trang 100 commit | **0.86 ms** | **0.88 ms** | 0.1% |
| **Tổng đường nóng phía Rust** | **749.7 ms** | **762.2 ms** | mốc 1000 ms |
| Trang thứ hai (cache hit, không sinh git) | **0.646 ms** | **0.732 ms** | |

**Hai lần đo độc lập.** Lần 1 trên repo 100k sinh ngày 2026-09-21 sáng; lần 2 trên repo
**sinh lại từ đầu** cùng ngày sau khi thư mục fixture bị dọn mất (cùng
`make-perf-repo.sh`, cùng hình dạng: 100 007 commit, 3 182 merge, 320 octopus). Chênh
lệch **1.7%** trên tổng — nằm trong nhiễu của phép đo một lần, nên cả hai dùng được và
không cần chọn một con số "đúng". Điều đáng tin là **tỉ lệ**: `git log` chiếm 84% ở cả hai
lần.

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

### 1.3 Thời gian vẽ lần đầu, đầu-tới-cuối — ⏭️ BỎ QUA, KHÔNG ĐO

Đây là con số **thật sự** đóng được checkpoint #1, và nó **không tồn tại**. Bảng 1.2 chỉ
đo phía Rust; nó **không** gồm IPC (serialize + qua cầu + deserialize) và **không** gồm
thời gian React dựng DOM và canvas vẽ khung đầu.

| Lần đo | Thời gian (ms) | Ghi chú |
|---|---:|---|
| Lần 1 (đĩa nguội) | ⏭️ bỏ qua — checkpoint không chạy | tính riêng, không vào trung vị |
| Lần 2 | ⏭️ bỏ qua — checkpoint không chạy | |
| Lần 3 | ⏭️ bỏ qua — checkpoint không chạy | |
| Lần 4 | ⏭️ bỏ qua — checkpoint không chạy | |
| Lần 5 | ⏭️ bỏ qua — checkpoint không chạy | |
| **Trung vị lần 2–5** | ⏭️ **bỏ qua — không có số** | mốc **< 1000 ms** không được kiểm |

**Hạ tầng đo đã có sẵn và vẫn chạy được.** `measureFirstPaint` đã gắn vào `CommitList`
(commit `697f6c3`, 14 test trong `src/lib/perf.test.ts`). Chỉ phép đo là không được
thực hiện. Ai muốn trả nợ này về sau chạy theo bảy bước trong Task 2 của
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

**Phần dư IPC + render** = (trung vị ở bảng này) − 749.7 ms. Không có trung vị thì
**không tính được phần dư**, và không có phần dư thì điều kiện thứ nhất của checkpoint
#4 (mục 2.4) **không đánh giá được**. Hai checkpoint dính vào nhau đúng ở chỗ này.

### 1.4 FPS lúc cuộn — ⏭️ BỎ QUA, KHÔNG ĐO

Đo bằng lệnh đã phơi sẵn trên `window` (chỉ gắn khi cờ bật):

```js
await window.gitPlumMeasureScrollFps(10000)
// -> { frames, durationMs, avgFps, p1Fps, longestFrameMs }
```

Cuộn liên tục trong suốt 10 giây đo. **Ba kiểu cuộn riêng**, vì chúng sinh tần suất sự
kiện khác nhau và kiểu tệ nhất mới là kiểu người dùng nhớ:

| Kiểu cuộn | `avgFps` | `p1Fps` (mốc ≥ 50) | `longestFrameMs` (mốc ≤ 100) |
|---|---:|---:|---:|
| Kéo thanh cuộn nhanh | ⏭️ bỏ qua | ⏭️ bỏ qua | ⏭️ bỏ qua |
| Lăn chuột | ⏭️ bỏ qua | ⏭️ bỏ qua | ⏭️ bỏ qua |
| Giữ PageDown | ⏭️ bỏ qua | ⏭️ bỏ qua | ⏭️ bỏ qua |

> Quan sát định tính duy nhất có được — cuộn "khá mượt" trên repo **4 037** commit — nằm
> ở phần "Quyết định bỏ qua" đầu tài liệu. Nó **không** điền được vào bảng này: nó không
> phải `p1Fps`, không phải `longestFrameMs`, và không ở quy mô 100k.

**Vì sao `p1Fps` và `longestFrameMs` mới là số quyết định, không phải `avgFps`:** một
lần cuộn có `avgFps` 58 mà chứa đúng một khung 400 ms thì người dùng thấy khựng rất rõ,
nhưng `avgFps` vẫn nằm trên mốc 50. Có test dựng đúng ca đó trong `src/lib/perf.test.ts`
(`một khung 400ms lẫn trong 99 khung đẹp`).

### 1.5 Thẳng hàng ở quy mô 100k (HIST-04) — ⏭️ BỎ QUA, KHÔNG KIỂM

Plan 02-05 chỉ kiểm thẳng hàng trên repo nhỏ. Bước kiểm ở 100k commit (cuộn tới giữa
rồi tới cuối bằng Ctrl+End) nằm trong checkpoint bị bỏ qua.

| Vị trí cuộn | Kết quả |
|---|---|
| Đầu danh sách | ⏭️ bỏ qua — checkpoint không chạy |
| Khoảng giữa | ⏭️ bỏ qua — checkpoint không chạy |
| Cuối (Ctrl+End) | ⏭️ bỏ qua — checkpoint không chạy |

Điều này **không** làm HIST-04 mất hết bằng chứng: một scroll container và một
virtualizer là ràng buộc kiến trúc có test ràng buộc ở tầng frontend, và kiểm bằng mắt
ở repo nhỏ (plan 02-05 checkpoint #2, vòng 2) đã đạt. Nhưng "ở mọi vị trí cuộn" trên
100k — đúng chỗ nó dễ vỡ nhất — **không ai kiểm.**

### 1.6 Kết luận checkpoint #1

⏭️ **BỎ QUA — KHÔNG KẾT LUẬN.** Không có trung vị thời gian vẽ lần đầu (1.3), không có
ba bộ số FPS (1.4), không có kiểm thẳng hàng ở 100k (1.5).

Phần Rust (**749.7 ms**, còn dư ~250 ms so với mốc 1000 ms) là **dấu hiệu thuận lợi**,
và nó là một con số thật. Nhưng nó **không** phải kết luận của checkpoint: phần dư
~250 ms đó phải gánh cả IPC lẫn React render lẫn lần vẽ canvas đầu tiên, và không ai
biết ba thứ đó tốn bao nhiêu. Một con số phía Rust không trả lời được một tiêu chí nói
về thứ người dùng **nhìn thấy**.

**Trạng thái của tiêu chí thành công số 1 và của HIST-05 vì vậy là "chưa kiểm chứng" —
không phải "đạt", không phải "có thể đạt", không phải "đạt một phần".**

---

## 2. Checkpoint #4 — IPC JSON hay nhị phân cho dữ liệu lane

### 2.1 Kích thước payload JSON — 🟡 ĐO MỘT PHẦN

Đã đo trên trang **100 commit** (từ `history_budget.rs`, cắt ở giữa lịch sử):

| Hạng mục | Giá trị |
|---|---:|
| Payload JSON một trang 100 commit | **77 657 byte** (~777 byte/commit) |
| Thời gian serialize trang đó | **0.808 ms** |

⏭️ **Không đo ở `PAGE_SIZE` thật là 1000**, và **không tách riêng** `commits` với
`graph_rows` — cả ba số này lấy từ log của ứng dụng đang chạy, tức từ chính lượt chạy
bị bỏ qua. Hạ tầng đo đã có và vẫn đúng (Task 1 thêm log `payload_bytes`,
`commits_bytes`, `graph_rows_bytes`, `graph_rows_percent` vào `get_commit_page`,
`src-tauri/src/commands/history.rs` dòng 334–337); nó chỉ chưa bao giờ được chạy trên
ứng dụng thật.

| Hạng mục | Giá trị |
|---|---:|
| Payload JSON một trang **1000** commit | ⏭️ bỏ qua — checkpoint không chạy |
| Riêng `commits` | ⏭️ bỏ qua — checkpoint không chạy |
| Riêng `graph_rows` | ⏭️ bỏ qua — checkpoint không chạy |
| **Tỉ lệ `graph_rows` trong payload** | ⏭️ **bỏ qua — không có số** |

> Không ngoại suy 77 657 byte × 10 thành số của trang 1000 commit. Quan hệ đó **gần**
> tuyến tính nhưng không đúng tuyến tính (thông điệp commit dài ngắn khác nhau theo
> vùng lịch sử; trang 100 commit ở trên cắt ở giữa lịch sử của một repo **sinh tự
> động**, nơi mọi thông điệp gần như cùng độ dài — điều không đúng với repo thật). Một
> số nội suy dán nhãn "đo được" là đúng thứ tài liệu này tồn tại để ngăn.

### 2.2 Phần của IPC trong tổng thời gian — ⏭️ BỎ QUA, KHÔNG ĐO

Tính bằng: (trung vị mục 1.3) − 749.7 ms. Mục 1.3 bị bỏ qua, nên **phép trừ này không
thực hiện được**.

| Hạng mục | Giá trị |
|---|---:|
| Phần dư IPC + render | ⏭️ bỏ qua — không tính được, thiếu mục 1.3 |

Đây là hệ quả dây chuyền đáng chú ý nhất của việc bỏ qua checkpoint #1: **nó cũng lấy
mất điều kiện thứ nhất của checkpoint #4.** Xem mục 2.4.

### 2.3 Bộ nhớ thường trú — ⏭️ BỎ QUA, KHÔNG ĐO

Plan 02-04 **ước tính** `RepoHistory` ≈ 67 MB cho repo 100k commit (`Vec<GraphRow>`
~17 MB + `Vec<Commit>` ~50 MB), so với ràng buộc **RAM lúc rảnh dưới 150 MB** trong
`PROJECT.md`. Đó là ước tính từ `size_of` và nó **bỏ qua phân mảnh malloc trên ~700k
allocation rời** — nên phải đo thật, không được dùng con số ước tính làm kết luận.

Đo trong Task Manager tab Details, cột **Memory (private working set)**:

| Mốc | `git-plum.exe` | `msedgewebview2.exe` (tổng) | Tổng hai |
|---|---:|---:|---:|
| Baseline (mở app, chưa mở repo) | ⏭️ bỏ qua | ⏭️ bỏ qua | ⏭️ bỏ qua |
| Sau khi nạp xong repo 100k | ⏭️ bỏ qua | ⏭️ bỏ qua | ⏭️ bỏ qua |
| **Hiệu số (chi phí thật của cache)** | ⏭️ bỏ qua | ⏭️ bỏ qua | ⏭️ bỏ qua |

**Đối chiếu với hạn 150 MB của `PROJECT.md`:** ⏭️ **không kết luận được — chưa kiểm
chứng.**

Con số **67 MB** của plan 02-04 là một **ước tính từ `size_of`**, không phải một phép
đo. Nó bỏ qua phân mảnh malloc trên ~700k allocation rời, và nó chỉ đếm `RepoHistory`
— không đếm baseline của Tauri và WebView2 (được ước khoảng 80–150 MB ở plan 02-04,
cũng là ước tính). Đặt cạnh hạn 150 MB:

| | Giá trị | Loại |
|---|---:|---|
| `RepoHistory` cho một repo 100k | ~67 MB | **ước tính** (`size_of`, plan 02-04) |
| `MAX_CACHED_HISTORIES` = 2 → hai lịch sử | ~134 MB | **ước tính** suy từ trên |
| Baseline Tauri + WebView2 | 80–150 MB | **ước tính**, không đo |
| Hạn `PROJECT.md` | 150 MB | ràng buộc đã ghi |
| **Kết luận** | ⏭️ **chưa kiểm chứng** | — |

Phép cộng hai dòng ước tính đầu đã vượt hạn trước khi cộng baseline. Điều đó **gợi ý**
rằng đây là chỗ đáng lo, nhưng một số ước tính không chứng minh được vi phạm cũng như
không chứng minh được tuân thủ. **Không đánh dấu "vượt hạn" và cũng không đánh dấu
"đạt".** Nợ này ghi ở mục 3.

> **Nếu vượt hạn:** đây là một ràng buộc đã ghi của dự án bị phá, không phải chuyện nhỏ
> bỏ qua được. **Không tự ý nới hạn 150 MB.** Hai đường xử lý đã biết, nêu kèm chi phí:
> **(a)** bỏ `body` khỏi cache và lấy lười theo từng commit khi người dùng chọn — cắt
> phần lớn của ~28 MB heap trong `Vec<Commit>`, đổi lại mỗi lần chọn commit tốn một lần
> đọc; **(b)** giảm `MAX_CACHED_HISTORIES` xuống 1 — đơn giản hơn nhưng làm việc chuyển
> qua lại giữa hai repo phải nạp lại từ đầu.

### 2.4 Quyết định JSON hay nhị phân — **GIỮ JSON, QUYẾT ĐỊNH HOÃN**

Quy tắc quyết, chốt trước khi có số:

> Chuyển `graph_rows` sang `tauri::ipc::Response` (byte thô) **chỉ khi** phần dư IPC ở
> mục 2.2 là **thành phần lớn nhất** của tổng thời gian **và** `graph_rows` chiếm
> **phần lớn** payload ở mục 2.1. **Cả hai** điều kiện, không phải một.

Đánh giá từng điều kiện với dữ liệu thật sự có:

| Điều kiện | Dữ liệu cần | Trạng thái |
|---|---|---|
| (1) Phần dư IPC là thành phần lớn nhất của tổng thời gian | mục 2.2 | ⏭️ **KHÔNG ĐÁNH GIÁ ĐƯỢC** — mục 2.2 bị bỏ qua |
| (2) `graph_rows` chiếm phần lớn payload | mục 2.1 | ⏭️ **KHÔNG ĐÁNH GIÁ ĐƯỢC** — tỉ lệ ở `PAGE_SIZE`=1000 bị bỏ qua |

Quy tắc là phép **và** của hai điều kiện, và **không điều kiện nào đánh giá được**. Một
quy tắc không đánh giá được thì không sinh ra quyết định "chuyển".

**Quyết định vì vậy là: giữ JSON, và ghi nhận rằng đây là quyết định HOÃN, không phải
quyết định đã được số liệu xác nhận.** Phân biệt này quan trọng: "giữ JSON vì đo thấy
IPC không phải điểm nghẽn" là một kết luận; "giữ JSON vì không ai đo nên không có căn
cứ để đổi" là một chỗ trống được đánh dấu. Đây là loại thứ hai.

**Điều duy nhất biết chắc, và nó thật sự nghiêng về giữ JSON:** phía Rust serialize một
trang 100 commit hết **0.808 ms**, tức **0.1%** đường nóng, trong khi `git log` chiếm
**84.5%**. Đó là số đo thật (mục 1.2 và 2.1). Nó nói rằng *khâu serialize phía Rust*
không phải điểm nghẽn. Nó **không** nói gì về khâu đi qua cầu IPC và khâu deserialize
phía JavaScript — hai khâu còn lại của phần dư, và là hai khâu mà JSON thật sự đắt.

Điều kiện xem lại quyết định hoãn này nằm ở mục 2.7, và điều kiện đầu tiên trong đó
chính là: **có ai đó thật sự đo mục 2.1 và 2.2.**

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

`PAGE_SIZE` hiện là **1000** commit một thông điệp (`src/stores/historyStore.ts` dòng
13), theo ROADMAP ("1000 commit một thông điệp, không phải 1"). **Giữ nguyên 1000** —
không có số ở 2.1 và 2.2 để đề xuất đổi, và đổi một hằng số điều chỉnh hiệu năng mà
không có phép đo là đúng thứ checkpoint này tồn tại để ngăn.

### 2.7 Điều kiện xem lại quyết định giữ JSON

Một quyết định có điều kiện hết hạn thì hữu ích hơn một quyết định trống. Quyết định ở
2.4 là **hoãn**, nên điều kiện đầu tiên khác với bốn điều kiện còn lại: nó không phải
một thay đổi trong tương lai mà là phần việc còn thiếu ở hiện tại.

**Điều kiện 0 — trả nợ phép đo (khác loại với bốn điều dưới):**

- **Có ai đó thật sự chạy mục 2.1 và 2.2.** Chạy ứng dụng với `GIT_PLUM_PERF=1` và
  `RUST_LOG=git_plum_lib=info`, mở repo 100k, đọc dòng log chứa `payload_bytes`,
  `commits_bytes`, `graph_rows_bytes`, `graph_rows_percent`. Hạ tầng đã có; phần này
  **không** cần bản release và **không** cần đo bằng đồng hồ bấm tay, nên nó rẻ hơn
  nhiều so với phần bị bỏ qua của checkpoint #1. Ngay khi có bốn con số đó cộng với
  trung vị của 1.3, quy tắc hai điều kiện ở 2.4 đánh giá được và quyết định thật sự
  được chốt.

**Bốn điều kiện xem lại thông thường** — nếu bất kỳ điều nào sau xảy ra, quyết định
giữ JSON phải được xem lại kể cả khi đã có số:

- `PAGE_SIZE` tăng lên ~10 000 (chi phí serialize tăng tuyến tính, phần dư IPC có thể
  vượt qua `git log` để thành thành phần lớn nhất);
- Phase 3 thêm dữ liệu vào **cùng** payload (ví dụ đính kèm danh sách tệp thay đổi theo
  từng commit);
- `graph_rows` đổi hình dạng sang nhiều trường số hơn (tỉ lệ ở 2.1 dịch chuyển);
- Có người báo giật lúc cuộn trên repo lớn **sau khi** đã loại trừ (a) `git log`.

---

## 3. Nợ mang sang — việc phải làm để đóng hai checkpoint

Phase 2 đóng lại **mà không** làm những việc dưới đây. Đây là nợ kỹ thuật đã ghi, không
phải việc bị quên. Hạ tầng đo cho **cả sáu** dòng đã có sẵn và có test — chỉ phép đo là
chưa chạy, nên nợ này trả được bất cứ lúc nào, không phụ thuộc tiến độ Phase 3.

| # | Việc | Ai | Chặn cái gì | Rẻ hay đắt |
|---|---|---|---|---|
| 1 | Đo thời gian vẽ lần đầu, 5 lần, bản release, repo 100k (mục 1.3) | chủ dự án | **HIST-05**, tiêu chí thành công số 1 (Core Value) | đắt — cần dựng bản release |
| 2 | Đo FPS ba kiểu cuộn (mục 1.4) | chủ dự án | **HIST-05**, "cuộn không giật" | đắt — cần cuộn tay 3×10 giây |
| 3 | Kiểm thẳng hàng ở giữa và cuối repo 100k (mục 1.5) | chủ dự án | HIST-04 ở quy mô lớn | rẻ — kèm theo lượt chạy ở dòng 1 |
| 4 | Ghi bốn số payload ở `PAGE_SIZE`=1000 (mục 2.1) | tự động, khi app chạy với cờ | checkpoint #4 điều kiện (2) | **rẻ nhất** — chỉ cần mở repo với cờ bật, đọc log |
| 5 | Đo bộ nhớ thường trú baseline + sau nạp (mục 2.3) | chủ dự án | ràng buộc RAM 150 MB của `PROJECT.md` | rẻ — đọc Task Manager hai lần |
| 6 | Viết kết luận 1.6 và 2.4 từ số thu được | — | đóng checkpoint #1 và #4 thật sự | — |

**Dòng 4 đáng làm trước.** Nó không cần bản release, không cần đồng hồ bấm tay, không
cần cuộn bằng tay — chỉ cần chạy ứng dụng với `GIT_PLUM_PERF=1` rồi mở repo 100k và đọc
một dòng log. Nó đóng được một nửa điều kiện của checkpoint #4 với chi phí gần bằng 0.

---
*Tài liệu tạo: 2026-09-21. Phần Rust đo cùng ngày. Phần đầu-tới-cuối **bị bỏ qua** theo
quyết định chủ dự án 2026-09-21 và trở thành nợ mang sang; xem mục 3.*
