# Phase 2 — Xác minh từng requirement

**Ngày:** 2026-09-21
**Trạng thái phase:** 7/7 plan đã thực thi; **hai checkpoint người kiểm chưa chạy**, nên
7 trong 11 requirement còn `Pending`.

Tài liệu này nêu **bằng chứng** cho từng requirement, không nêu ý kiến. Ba mức:

- **Đạt** — có bằng chứng kiểm chứng được (test tự động chạy trên dữ liệu git thật, hoặc
  người dùng đã xác nhận bằng mắt).
- **Có mã, chưa kiểm** — đã cài và test tự động xanh, nhưng tiêu chí của requirement nói
  về thứ **người dùng quan sát được** mà chưa ai quan sát.
- **Chưa đo** — tiêu chí là một con số và con số đó chưa tồn tại.

---

## Tóm tắt

| Req | Trạng thái | Bằng chứng ngắn |
|---|---|---|
| HIST-01 | **Đạt** | `CommitList` render trên repo thật; người dùng đã thấy danh sách commit qua ảnh chụp |
| HIST-02 | **Đạt** | Đồ thị nhiều lane có màu; người dùng đã xác nhận qua ảnh chụp app thật |
| HIST-03 | **Có mã, chưa kiểm** | Snapshot 9 hình dạng repo xanh; `octopus`/`wide` chưa ai mở bằng mắt |
| HIST-04 | **Đạt một phần** | Bất biến `rows.len() == commits.len()` + test chặn toạ độ; **giữa/cuối repo 100k chưa kiểm** |
| HIST-05 | **Chưa đo** | Phía Rust 762 ms; **thời gian vẽ lần đầu đầu-tới-cuối và FPS chưa đo** |
| HIST-06 | **Có mã, chưa kiểm** | `RefBadges` + test; vòng 1 checkpoint từ chối, đã sửa, chờ vòng 2 |
| HIST-07 | **Có mã, chưa kiểm** | `RefSidebar` + test đếm; chưa đối chiếu với `git branch \| wc -l` thật |
| HIST-08 | **Có mã, chưa kiểm** | `CommitDetail` + test 4 cha octopus; chưa ai mở bằng mắt |
| HIST-09 | **Có mã, chưa kiểm** | `buildFileTree` + test; chưa ai bấm nút chuyển phẳng/cây |
| HIST-10 | **Có mã, chưa kiểm** | `CommitSearch` 4 trục + test debounce; chưa ai gõ thử |
| HIST-11 | **Đạt** | Ghim ở 3 tầng: parser, ref, hiển thị — có test mỗi tầng |

**Đạt: 3** (HIST-01, 02, 11) · **Đạt một phần: 1** (HIST-04) · **Có mã, chưa kiểm: 6** ·
**Chưa đo: 1** (HIST-05)

---

## Chi tiết

### HIST-01 — thấy danh sách commit, nạp theo trang · **Đạt**

- `src/components/history/CommitList.tsx` dựng từ `useVirtualizer`, `ensureRange` nạp
  trang khi cuộn tới dải chưa có.
- Test: `CommitList.test.tsx` render 30 commit, khẳng định **từng subject xuất hiện đúng
  một lần** (bắt được lỗi trộn dữ liệu giữa các hàng).
- Test dedup: `historyStore.ensureRange` không gọi trùng cho cùng dải — mutation bỏ
  `isCovered` cho 3 lời gọi thay vì 1, test đỏ.
- **Bằng chứng người dùng:** ảnh chụp app thật hiển thị danh sách commit của repo
  `quanly-truong-phong-so` và của chính git-plum.

### HIST-02 — đồ thị nhánh nhiều làn có màu, bên trái danh sách · **Đạt**

- Canvas vẽ từ cùng mảng `virtualItems` mà cột văn bản dùng.
- Test màu: 3 hàng `color` 0/1/2 phải xuất hiện **đúng ba màu lane cụ thể** trong lịch sử
  `fillStyle`/`strokeStyle` — không phải chỉ "đếm được 3 màu khác nhau". Mutation
  `colorFor(row.color)` → `colorFor(0)` làm test đỏ.
- **Bằng chứng người dùng:** ảnh chụp cho thấy đồ thị nhiều màu sau khi sửa ba lỗi vẽ.

### HIST-03 — merge nhiều hơn hai cha, nhánh mồ côi, lịch sử không liên thông · **Có mã, chưa kiểm**

- Snapshot `insta` trên **9 hình dạng repo thật**: `linear`, `octopus` (4 cha),
  `unrelated` (2 gốc rời), `orphan`, `wide` (25 nhánh/24 merge), `non-utf8`, `shallow`,
  `detached`, `submodule`.
- Mutation `.skip(1)` → `.take(2).skip(1)` (chỉ đọc 2 cha đầu của octopus): **5 test đỏ**,
  thông báo `left: 2, right: 4`.
- **Còn thiếu:** bước 3 và 6 của checkpoint — mở `octopus` và `wide` trong app, xem merge
  4 cha có bốn đường rẽ và `wide` có hiện `+N` chứ không tràn ngang.

### HIST-04 — đồ thị luôn thẳng hàng ở mọi vị trí cuộn · **Đạt một phần**

Đạt ở tầng dữ liệu và tầng toạ độ:

- Bất biến `rows.len() == commits.len()`, không ngoại lệ kể cả khi số nhánh sống vượt cap.
  Mutation gộp hai allocator lane: **đúng 1 test đỏ** (`80 commit vào, 60 hàng ra`) —
  16/17 test khác vẫn xanh, nên nếu không có test đó thì lỗi ship ra.
- Một nguồn toạ độ Y duy nhất (`virtualizer`), không có phép tính song song.
- 3 test chặn hệ quy chiếu: hàng đầu vùng nhìn thấy luôn `y=0` dù cuộn bao xa; mọi hàng
  nhìn thấy nằm trong chiều cao canvas. Mutation bỏ `- scrollTop`: 2 test đỏ.
- 3 test chặn đường vẽ không tràn khỏi ô hàng. Mutation trả về `y + 1.5 * ROW_HEIGHT`:
  3 test đỏ.

**Chưa kiểm:** bước 1–3 của checkpoint ở **giữa và cuối repo 100k**. Ba lỗi thẳng hàng đã
gặp trong phase này (badge phá chiều cao hàng, đường vẽ tràn hàng, sai hệ quy chiếu toạ
độ) **đều không bị test tự động nào bắt** — happy-dom không tính layout CSS và không có
cuộn thật. Đó là lý do bước kiểm bằng mắt không thể bỏ.

### HIST-05 — repo 100k: đồ thị dưới một giây, cuộn mượt · **Chưa đo**

Đã đo (release, repo 100 007 commit, hai lần độc lập):

| Đoạn | Lần 1 | Lần 2 |
|---|---:|---:|
| `git log --all --topo-order` | 633.8 ms | 640.4 ms |
| `parse_log` | 63.1 ms | 66.9 ms |
| `lanes::assign` | 51.9 ms | 54.0 ms |
| cache + cắt trang + JSON | 0.86 ms | 0.88 ms |
| **Tổng phía Rust** | **749.7 ms** | **762.2 ms** |

**Con số này KHÔNG đóng được requirement.** Nó là phía Rust, chưa gồm IPC và render
React. Số đóng được là `[perf] commit-list-first-paint` đo từ trong app đang chạy, cộng
ba phép đo FPS (kéo thanh cuộn / lăn chuột / giữ PageDown). Cả bốn **chưa có**.

Ngưỡng đã chốt **trước** khi xem số: vẽ lần đầu < 1000 ms, `p1Fps >= 50`,
`longestFrameMs <= 100`.

Đọc được từ bảng: `git log` chiếm **84%**. Nếu phép đo thật trượt mốc, chỗ phải động vào
là khâu nạp (`Channel`/phân trang thật), không phải `parse_log` hay `assign` — hai thứ đó
cộng lại chỉ 16%.

Bộ nhớ: con số ~67 MB mỗi lịch sử là **ước tính** từ `size_of`, chưa ai đo RSS thật, nên
hạn 150 MB của `PROJECT.md` đang ở trạng thái **chưa kiểm chứng** — không đánh "đạt" cũng
không đánh "vượt".

### HIST-06 — nhãn nhánh và tag neo đúng hàng, phân biệt local/remote · **Có mã, chưa kiểm**

- `%(*objectname)` giải tham chiếu tag có chú thích — thiếu nó thì nhãn tag không neo được
  vào hàng nào. Mutation bỏ nó: 2 test đơn vị + 1 tích hợp đỏ.
- Nhãn có **cột grid riêng** (không dùng chung cột với chữ message).
- **Checkpoint vòng 1 bị người dùng từ chối** với ba lỗi bố cục, cả ba đã sửa. Chờ vòng 2.

### HIST-07 — thanh bên liệt kê nhánh local/remote/tag kèm số đếm · **Có mã, chưa kiểm**

- `RefSidebar` + `refsStore`. Mutation hardcode `hasHead = true`: 1 test đỏ (ca detached
  HEAD).
- **Còn thiếu:** bước 7 và 8 của checkpoint — đối chiếu ba số đếm với
  `git branch | wc -l`, `git branch -r | wc -l`, `git tag | wc -l` chạy trong terminal, và
  mở `target/fixtures/detached` xem sidebar có nói rõ detached HEAD.

### HIST-08 — chọn commit thấy tiêu đề, nội dung, tác giả, thời gian, cha, tệp · **Có mã, chưa kiểm**

- `CommitDetail`. Commit gốc so với cây rỗng `4b825dc6…` (không có `<commit>^`); merge so
  với `parents[0]`.
- Mutation `.slice(0, 2)` trên danh sách cha: 1 test đỏ (ca octopus 4 cha).
- `R`/`C` của `--name-status -z` chiếm **hai** đường dẫn — đọc sai làm lệch mọi bản ghi
  sau; có test đặt hai bản ghi sau bản ghi `R` để bắt đúng dạng lệch.
- **Còn thiếu:** bước 1, 3, 4 của checkpoint (gồm cả kiểm thời gian không phải 1970).

### HIST-09 — chuyển danh sách tệp giữa dạng phẳng và dạng cây · **Có mã, chưa kiểm**

- `buildFileTree` là hàm thuần, test được không cần DOM. `uiStore` giữ chế độ xem.
- **Còn thiếu:** bước 2 của checkpoint, gồm cả việc chế độ xem có **giữ nguyên** khi chọn
  commit khác rồi quay lại.

### HIST-10 — tìm commit theo thông điệp, tác giả, mã commit, đường dẫn tệp · **Có mã, chưa kiểm**

- Ba trục lọc trong bộ nhớ (thông điệp/tác giả/SHA), trục đường dẫn tệp hỏi git — luôn có
  `--` **trước** pathspec (T-02-11).
- Debounce 250 ms. Mutation `DEBOUNCE_MS` 250 → 0: **lần đầu 0 test đỏ**, vì assertion cũ
  không phân biệt được 0 ms với 250 ms. Đã siết (chia 249 ms rồi +1 ms) và chạy lại
  mutation: 1 test đỏ.
- Mutation xoá `.arg("--")`: **lần đầu 0 test đỏ** — test cũ tự dựng lệnh git của nó nên
  chứng minh `--` *có tác dụng* mà không chứng minh `search_commits` *dùng* nó. Đã thêm
  test đọc thân hàm thật.
- **Còn thiếu:** bước 9, 10, 11 của checkpoint, gồm đếm số lệnh `git log` thật trong nhật
  ký lệnh khi gõ nhanh.

### HIST-11 — ký tự không phải UTF-8 vẫn hiển thị, không hỏng danh sách · **Đạt**

Ghim ở **ba tầng**, mỗi tầng một test:

1. **Parser** (wave 2): giải mã lossy chỉ cho trường hiển thị; bản ghi xấu không làm rơi
   bản ghi sau.
2. **Ref** (wave 4): tên nhánh chứa byte không UTF-8 vẫn ra một `Ref` và **không làm mất
   ref nào sau nó** — test đặt một ref hợp lệ ngay sau ref xấu.
3. **Hiển thị** (wave 5): `hasInvalidUtf8` render bình thường kèm cảnh báo `⚠`; test
   khẳng định hàng **không bị lọc bỏ**.

Bước 12 của checkpoint (so số dòng trên màn với `git log … | wc -l`) sẽ là bằng chứng
thứ tư, nhưng ba tầng trên đã đủ để đánh **Đạt** — đây là requirement về *không mất dữ
liệu*, và điều đó kiểm được bằng test trên byte thật.

---

## Hai việc còn nợ

1. **Checkpoint 12 bước của plan 02-06** (kiểm bằng mắt) — mở chốt HIST-03, 06, 07, 08,
   09, 10 và phần còn lại của HIST-04.
2. **Checkpoint #1 của plan 02-07** (đo hiệu năng trên repo 100k) — mở chốt HIST-05, và
   cung cấp phần dư IPC cần cho quyết định JSON-hay-nhị-phân của checkpoint #4 (hiện
   **hoãn**, giữ JSON, không phải kết luận có số liệu).

Dụng cụ và dữ liệu cho cả hai đã sẵn: bản release tại
`src-tauri/target/release/git-plum.exe`, 9 repo mẫu tại `target/fixtures/`, repo 100k tại
`target/fixtures-perf/perf-100k`, dụng cụ đo bật bằng `localStorage.gitPlumPerf = '1'`.
