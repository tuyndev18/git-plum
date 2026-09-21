# Phase 3: Xem khác biệt — Context

**Ngày:** 2026-09-21
**Goal (ROADMAP):** Người dùng đọc được chính xác một commit đã thay đổi những gì, ở mức
từng dòng.

---

## 1. Phạm vi đã chốt với chủ dự án

### Trong phạm vi

| Req | Nội dung |
|---|---|
| DIFF-01 | Chọn tệp trong commit → thấy nội dung thay đổi, **có tô màu cú pháp** |
| DIFF-02 | Chuyển được giữa chế độ **hợp nhất** và chế độ **hai cột** |
| DIFF-03 | Nhảy tới khối thay đổi **kế tiếp và trước đó** |
| DIFF-04 | **Bật tắt** hiển thị ký tự khoảng trắng |
| DIFF-05 | Xem **lịch sử thay đổi của riêng một tệp** |
| DIFF-06 | Tệp nhị phân, tệp vượt ngưỡng, con trỏ **Git LFS** → thông báo phù hợp, **không đổ nội dung thô** |

### Thêm ngoài requirements — chủ dự án yêu cầu rõ

**Diff mức TỪ (word-level).** Không chỉ tô cả dòng: trong một dòng đã sửa, phải tô **riêng
phần chữ thay đổi**. Ví dụ chủ dự án đưa: dòng `if (typeof cellData == "object")` →
`=== "object"`, chỉ `==` → `===` được tô đậm, phần còn lại của dòng để nguyên.

Đây **không** nằm trong DIFF-01..06 và `@codemirror/merge` **không cho sẵn** — cần
`git diff --word-diff` (hoặc `--word-diff-porcelain`) hoặc tự tính ở tầng hiển thị. Xem
mục 4.

### Ngoài phạm vi — đã cân nhắc và bỏ

Ảnh tham chiếu chủ dự án gửi có mấy thứ nữa; chủ dự án nói rõ **"ảnh chỉ là mẫu ví dụ
thôi"** và chọn giữ đúng DIFF-01..06 cộng word-level diff. Những thứ **không làm**:

- **Minimap** bên phải (dải màu chỉ vị trí thay đổi trong cả tệp)
- **Nhãn encoding tệp** (`UTF-8` ở góc)
- **Blame** — xem ai sửa dòng nào. Không có trong DIFF-01..06, là việc riêng khá lớn
- **Edit in Working Directory** — thuộc Phase 4 (thư mục làm việc)

Cũng **không** làm nốt phần giao diện giống GitKraken còn lại (toolbar, tab repo, sidebar
icon, panel phải) — quyết định 2026-09-21, xem `docs/07-ui-reference-gap.md`.

---

## 2. Ràng buộc từ ROADMAP — không thương lượng

- **Nạp lười chế độ ngôn ngữ CodeMirror** theo phần mở rộng tệp. Có hơn 30 gói
  `@codemirror/lang-*`; nhập sẵn tất cả sẽ phình bundle và hại thời gian khởi động — mà
  khởi động nhanh là **Core Value** của dự án.
- **Không** cài gói meta `codemirror`. Nó kéo theo `basicSetup` (autocomplete, brackets,
  history) — vô dụng cho trình xem diff chỉ-đọc và tốn thời gian khởi động. Compose từng
  gói `@codemirror/*` riêng.
- **Cache diff theo LRU ~200 mục.** Diff của một commit lịch sử là **bất biến**, nên cache
  **không bao giờ cần vô hiệu hoá** — khác `RepoCache` của Phase 2 (xem mục 3).
- **Dùng đầu ra kết thúc NUL** cho mọi biến thể diff có mang tên tệp. Cùng lý do như
  Phase 2: tên tệp có thể chứa byte lạ, ký tự xuống dòng, dấu ngoặc kép.

---

## 3. Những gì Phase 1–2 đã có, phải dùng lại chứ không viết lại

| Có sẵn | Ở đâu | Dùng cho việc gì ở Phase 3 |
|---|---|---|
| Điểm sinh tiến trình git **duy nhất** | `src-tauri/src/git/exec.rs` | Mọi lệnh `git diff`/`git show`/`git log --follow` **phải** đi qua đây. Nó đã ghim `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_EXTERNAL_DIFF=""`, `GIT_PAGER=cat`, xoá `GIT_AUTHOR_*`, `CREATE_NO_WINDOW` trên Windows |
| `GitError` + `GitErrorCode` (9 giá trị) | `src-tauri/src/error.rs`, `src/lib/ipc.ts` | Thêm variant mới **chỉ khi** giao diện cần phân biệt. Phase 2 kết luận `skip` ngoài phạm vi trả trang rỗng chứ không trả lỗi — cùng tinh thần đó |
| `FileChange { status, path, oldPath }` | `commands/history.rs` | `path` là đầu vào của diff. `status` là `String` (chứa `"R100"`, `"C75"`), không phải `char` |
| `CommitDetail { commit, files, truncated }` | như trên | Nguồn danh sách tệp để bấm vào |
| `AppState.cache` (`RepoCache`) | `src-tauri/src/cache/` | **KHÔNG** mở rộng cho diff. `MAX_CACHED_HISTORIES = 2` vì mỗi lịch sử ~67MB; cache diff là hàng nghìn mục nhỏ khoá theo SHA — hai bài toán khác nhau. Dùng crate `lru` riêng như STACK.md nói |
| `perf.ts` (`measureFirstPaint`, `measureScrollFps`) | `src/lib/perf.ts` | Dùng lại cho checkpoint #3 (đo diff tệp lớn). Bật bằng `localStorage.gitPlumPerf = '1'` |
| 9 repo mẫu + repo 100k | `target/fixtures/`, `target/fixtures-perf/` | Sinh bằng `scripts/fixtures/`. **Lưu ý: đã từng bị dọn mất một lần** — kiểm tồn tại trước khi dựa vào |

### Bài học Phase 2 phải mang sang

1. **happy-dom không tính layout CSS và không có cuộn thật.** Ba lỗi hiển thị của Phase 2
   (badge phá chiều cao hàng, đường vẽ tràn hàng, sai hệ quy chiếu toạ độ canvas) **qua
   hết 212 test tự động**. Với Phase 3, mọi thứ liên quan bố cục/cuộn/đo kích thước phải
   đo bằng **trình duyệt thật** (Playwright + Chromium, nạp đúng phông hệ thống) hoặc chấp
   nhận là chưa kiểm chứng.
2. **Cổng grep dễ vô dụng.** Mọi wave Phase 2 đều có ít nhất một cổng grep hỏng: khớp
   dòng chú thích, khớp prose, đếm sai số khớp tối thiểu, hoặc kiểm *có đăng ký* mà không
   kiểm *có dùng*. Kiểm cổng **có thể đỏ** trước khi tin nó xanh.
3. **Đo hiệu năng phải ở profile `--release`.** Debug chậm gần **7 lần** ở `parse_log`
   (455ms vs 67ms) và làm tổng vượt mốc một giây. Số debug vô nghĩa.
4. **Hai hằng số ở hai phía phải có test ghim.** Phase 2 có `MAX_VISIBLE_LANES` (Rust +
   TS) và `REF_COL_WIDTH` (CSS + TS); cả hai giờ có test đọc thẳng file kia. Lệch nhau là
   lỗi im lặng.

---

## 4. Checkpoint #3 — đo TRƯỚC khi viết UI (chủ dự án đã chọn)

**Vấn đề ROADMAP nêu:** `@codemirror/merge` **tự tính diff từ hai tài liệu đầy đủ** —
trong khi `git diff` đã đưa sẵn diff. Với tệp lớn, việc tính lại đó có thể quá chậm.

**Chủ dự án chọn: đo ngay wave 1, trước khi viết UI diff.** Lý do nêu rõ: Phase 2 đã phải
sửa đồ thị **ba vòng** vì viết xong mới phát hiện sai; đo trước thì biết ngay phải đi
đường nào.

**Hai đường:**

| Đường | Cách làm | Khi nào chọn |
|---|---|---|
| A — `@codemirror/merge` | `MergeView` (hai cột) + `unifiedMergeView` (hợp nhất). Ít mã hơn nhiều | Nếu phép đo cho thấy đủ nhanh trên tệp lớn thật |
| B — tự vẽ hunk | Dựng decoration CodeMirror **từ đầu ra `git diff`**, không để CM tính lại | Nếu A chậm. ROADMAP gọi đây là đường lùi chính thức |

**Phép đo phải có trước khi chốt:** thời gian mở diff của một tệp lớn thật (không phải tệp
tự sinh) ở profile **release**, cho cả hai đường nếu cần. Ngưỡng kết luận **chốt trước khi
xem số** — planner phải đặt con số cụ thể vào plan, không để "đủ nhanh" chung chung.

**Lưu ý word-level diff giao với quyết định này.** Nếu chọn đường B (tự vẽ hunk từ
`git diff`), thì word-level diff **cũng phải lấy từ git** (`--word-diff-porcelain`) cho
nhất quán một nguồn. Nếu chọn đường A, `@codemirror/merge` có thể tự cho word-level —
cần kiểm, không giả định.

---

## 5. Exit gate — dùng thật, không phải kiểm checklist

ROADMAP đặt gate bắt buộc để **đóng** phase:

> Bản chỉ-đọc lịch sử-cộng-diff phải được tác giả dùng hằng ngày trên repo thật của mình —
> gồm cả repo của chính dự án này và ít nhất một repo bên thứ ba lớn, lộn xộn — **trước
> khi** bắt đầu bất kỳ việc nào thuộc thư mục làm việc. Đây là cổng thoát phase, không
> phải điều nên có.

**Chủ dự án đã chọn:** dùng các repo công ty sẵn có (`quanly-truong-phong-so` 398 commit
nhiều merge, `thithu-web`, `client_cocos_test`). **Không** clone thêm repo OSS lớn.

Gate này khác checkpoint: không phải kiểm 12 bước theo danh sách, mà **dùng thật để làm
việc thật** trong vài ngày. Nó bắt loại lỗi "dùng 10 phút mới thấy bất tiện" mà checklist
không bắt được.

---

## 6. Câu hỏi để planner tự trả lời (không chặn)

1. **Nguồn word-level diff**: `git diff --word-diff-porcelain` hay tự tính ở JS? Ràng buộc:
   nếu tự tính thì lại rơi vào đúng cái bẫy "CM tự tính diff" mà checkpoint #3 lo.
2. **Ngưỡng kích thước tệp** cho DIFF-06 là bao nhiêu? ROADMAP nêu "tệp 200MB" làm ví dụ
   tiêu chí nhưng không chốt ngưỡng. Cần một con số và lý do.
3. **Nhận biết con trỏ Git LFS**: đọc nội dung blob tìm `version https://git-lfs...`, hay
   hỏi `git check-attr filter`? Cái sau đúng hơn nhưng thêm một lệnh git.
4. **DIFF-05 (lịch sử một tệp)**: `git log --follow -- <path>` theo được đổi tên nhưng
   ROADMAP Phase 2 đã ghi `--follow` chỉ theo được **một** đường dẫn và có chi phí. Quyết
   định dùng hay không, kèm lý do.
5. **Cache diff khoá theo gì?** `(sha, path)` là tối thiểu; có cần thêm chế độ
   hợp nhất/hai cột và trạng thái hiện khoảng trắng vào khoá không, hay hai thứ đó chỉ đổi
   cách *vẽ* chứ không đổi *dữ liệu*?
6. **Tệp mới thêm và tệp bị xoá**: diff so với cây rỗng
   `4b825dc642cb6eb9a060e54bf8d69288fbee4904` như `get_commit_detail` của Phase 2 đã làm,
   hay đường riêng?

---

## 7. Ràng buộc kỹ thuật của dự án (nhắc lại từ CLAUDE.md)

- **Chỉ gọi `git` CLI.** Không libgit2, không gitoxide.
- **Không thêm `tauri-plugin-fs`** — mọi truy cập tệp ở Rust qua `std::fs`/`tokio::fs`.
- **Không thêm `tauri-plugin-shell`** — git chạy bằng `tokio::process::Command` trong Rust.
- **Đầu ra git là BYTE**, không bảo đảm UTF-8. Dùng `bstr`/`memchr`; giải mã lossy **chỉ**
  cho trường hiển thị.
- **Windows ưu tiên** phát triển và kiểm thử; macOS/Linux chưa kiểm chứng.
- Cài đặt dưới 20MB, RAM lúc rảnh dưới 150MB (hạn sau **chưa được kiểm chứng** — xem
  `02-07-SUMMARY.md`).

---

## 8. Nợ kỹ thuật mang vào từ Phase 2

Phase 2 **chưa đóng**. Hai checkpoint cần người chạy app vẫn nợ:

1. **Checkpoint 12 bước của plan 02-06** — mở chốt HIST-03, 06, 07, 08, 09, 10 và phần
   còn lại của HIST-04.
2. **Checkpoint #1 của plan 02-07** (đo hiệu năng repo 100k) — mở chốt **HIST-05, tức Core
   Value của dự án**. Phía Rust đã đo: 749.7ms / 762.2ms so với mốc 1000ms, `git log`
   chiếm 84%. Nhưng thời gian vẽ lần đầu đầu-tới-cuối và FPS **chưa đo**.

Phase 1 cũng còn nợ ba tiêu chí chưa kiểm chứng (`docs/03-phase1-qa-windows.md`).

**Hệ quả cho Phase 3:** Core Value ("đồ thị mở dưới một giây") chưa được kiểm chứng
đầu-tới-cuối, và Phase 3 **thêm việc vào cùng đường nóng đó** (mở diff sau khi chọn
commit). Planner nên cân nhắc: phép đo của checkpoint #3 có thể trả nợ một phần cho
checkpoint #1 nếu thiết kế phép đo bao cả đường mở repo → chọn commit → mở diff.
