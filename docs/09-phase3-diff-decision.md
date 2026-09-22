# Phase 3 — quyết định A hay B cho khung hiển thị diff

**Ngày lập:** 2026-09-21
**Plan:** `03-01` (wave 1), checkpoint #3
**Trạng thái:** ⏭️ **BỎ QUA** — chủ dự án quyết định bỏ qua phép đo ngày 2026-09-22
khi được hỏi trực tiếp (hai lựa chọn: tự đo ~5 phút, hoặc bỏ qua kèm hệ quả đã nêu).
Chọn **đường A** (`@codemirror/merge`) **không có bằng chứng đo**.

Không ô nào dưới đây bị lấp bằng số ước lượng hay nội suy. Ô trống vẫn là ô trống —
xem mục 8 cho hệ quả và điều kiện xem lại.

---

## 1. Câu hỏi

ROADMAP nêu mối lo: `@codemirror/merge` **tự tính diff từ hai tài liệu đầy đủ**,
trong khi `git diff` đã đưa sẵn diff. Với tệp lớn, việc tính lại đó có thể quá chậm.

| Đường | Cách làm | Khi nào chọn |
|---|---|---|
| **A** — `@codemirror/merge` | `MergeView` (hai cột) + `unifiedMergeView` (hợp nhất). Ít mã hơn nhiều | Nếu phép đo cho thấy đủ nhanh trên tệp lớn thật |
| **B** — tự vẽ hunk | Dựng decoration CodeMirror **từ đầu ra `git diff`**, không để CM tính lại | Nếu A chậm. ROADMAP gọi đây là đường lùi chính thức |

Chủ dự án chọn đo ở **wave 1, trước khi viết UI diff**. Lý do nêu rõ: Phase 2 phải
sửa đồ thị **ba vòng** vì viết xong mới phát hiện sai.

---

## 2. Ngưỡng kết luận — 🔒 **CHỐT TRƯỚC KHI ĐO**

> Phần này được viết và commit **trước** khi bất kỳ con số nào tồn tại. Đây là điều
> kiện chủ dự án đặt ra ("chốt trước khi xem số"), và là cùng kỷ luật đã dùng ở
> checkpoint #1 của Phase 2 (`<1000ms, p1Fps>=50, longestFrameMs<=100`).

Với một tệp lớn thật, ở bản dựng **release**:

| Con số | Ngưỡng | Nếu vượt |
|---|---|---|
| Đường A, **lần đo tốt nhất trong 3** (bỏ lần đầu nếu nó là lần nạp module) | **≤ 250 ms** | → chọn **B** |
| Đường A, lần tệ nhất trong 3 | **≤ 400 ms** | → chọn **B** |
| Đường B, lần tốt nhất | *ghi lại, không có ngưỡng* | B là đường lùi; nó không phải thi với ai |

**Vì sao 250 ms, không phải một con số khác:**

- Đây là thời gian **sau** khi người dùng đã bấm vào một tệp, nên nó cộng vào cảm
  giác "mở diff", không cộng vào "mở repo". Ngân sách 1000 ms của Core Value thuộc
  về đường mở repo → vẽ đồ thị (Phase 2, đã đo phía Rust 762 ms) và **không** được
  chia lại cho diff.
- 250 ms là mốc quen trong tài liệu tương tác: dưới mốc này thao tác được cảm nhận
  là "phản hồi ngay", trên mốc này người dùng thấy độ trễ. Chọn nó vì nó nói về
  *cảm nhận*, đúng thứ Core Value của dự án này đo.
- Giới hạn lần tệ nhất **400 ms** riêng biệt, vì bài học `p1Fps` của Phase 2: một
  phép đo trung bình 200 ms mà chứa một lần 900 ms thì người dùng thấy khựng rất rõ,
  nhưng trung bình vẫn báo "đạt". Khung tệ nhất mới là khung người dùng nhớ.

**Quy tắc bổ sung, cũng chốt trước:** nếu đường A đạt ngưỡng nhưng kiểm word-level
cho thấy `@codemirror/merge` **không** cho word-level trong dòng, thì **vẫn chọn A
cho khung hiển thị** và lấy word-level từ git (plan 03-03). Hai việc đó độc lập;
đừng để một kết quả word-level âm tính lật ngược quyết định khung hiển thị.

---

## 3. Dữ liệu đo — tệp lớn thật, không phải tệp tự sinh

Repo `git-plum` **không dùng được** để đo: blob **văn bản** lớn nhất là
`src-tauri/Cargo.lock` **126 342 byte** (~123 KB), dưới ngưỡng 200 KB mà phép đo cần.
(Blob lớn nhất tuyệt đối là `docs/screenshots/main-1.png` 259 KB, nhưng nó nhị phân
nên vô dụng.)

Nên phép đo chạy trên repo công ty **`D:/MyCompanyProjects/quanly-truong-phong-so`**
(398 commit, nhiều merge — cùng repo mà mục 5 của CONTEXT.md đặt làm exit gate của
phase). Tệp đo: **`yarn.lock`**, văn bản thuần, 631–665 KB tuỳ phiên bản.

Ba cặp commit, đã đo bằng `git diff --numstat` (git 2.54.0.windows.1):

| Vai trò | Cặp commit | Kích thước tệp mới | Thay đổi |
|---|---|---|---|
| **Ca nặng nhất** | `d46cd3b5..519fa0b0` | 631 868 byte | **12 982 dòng thêm, 0 xoá** |
| Ca vừa | `bcecca66..6f3cbb2a` | 639 509 byte | 128 thêm / 1 xoá |
| **Ca sửa hai phía** | `6f5f2b25..1a0d6a6f` | 653 259 byte | 66 thêm / 49 xoá |

- **Ca nặng nhất** dùng cho phép đo chính — nó đúng là thứ checkpoint #3 lo:
  `@codemirror/merge` phải tính lại diff từ toàn bộ hai tài liệu ~630 KB.
- **Ca sửa hai phía** dùng cho phép đo word-level, vì nó có cả thêm lẫn xoá nên
  `--word-diff=porcelain` sinh span thật.

Repo công ty là repo làm việc thật của chủ dự án: phép đo **chỉ đọc**.

---

## 4. Bảng số đo thật

> ⏭️ **BỎ QUA** (2026-09-22) — xem mục 7 và 8. Ô này để trống có chủ ý; công cụ đo vẫn còn nếu muốn điền sau.
> **Không** điền bằng số ước lượng — một ô trống trung thực hơn một con số bịa.

| Đường | Lần 1 | Lần 2 | Lần 3 | Tốt nhất | Tệ nhất | Đạt ngưỡng? |
|---|---|---|---|---|---|---|
| A — `@codemirror/merge` tự tính diff | | | | | | |
| B — decoration từ `git diff` | | | | | | *(không có ngưỡng)* |

Dữ liệu đầu vào của lần đo (harness in ra sẵn để chép):

| Trường | Giá trị |
|---|---|
| `rustMs` | |
| `oldLines` / `newLines` | |
| `oldBytes` / `newBytes` | |
| `patchBytes` | |

---

## 5. Kết quả kiểm word-level của đường A

> ⏭️ **BỎ QUA** (2026-09-22) — xem mục 7 và 8.

CONTEXT.md mục 4 viết "`@codemirror/merge` *có thể* tự cho word-level — cần kiểm,
không giả định". Harness có nút riêng dựng đúng ca chủ dự án đưa:

```
if (typeof cellData == "object")     →     if (typeof cellData === "object")
```

Harness **đếm bằng mã** thay vì chỉ để nhìn bằng mắt, vì "tôi thấy nó có tô" không
phân biệt được hai lớp CSS mà CodeMirror 6 dùng:

- `.cm-changedText` — phần chữ thay đổi **bên trong** một dòng → có word-level.
- `.cm-changedLine` — tô **cả dòng** → không có word-level.

| Số đo | Giá trị |
|---|---|
| `.cm-changedText` | |
| `.cm-changedLine` | |
| Kết luận | ⏭️ bỏ qua |

---

## 6. Cách chạy phép đo

```bash
export PATH="$USERPROFILE/.cargo/bin:$PATH"
taskkill //F //IM git-plum.exe 2>/dev/null || true
npm run tauri:build
```

1. Mở `src-tauri/target/release/git-plum.exe` **từ Explorer** (không `tauri dev` —
   WebView2 ở chế độ dev có devtools gắn vào và số sẽ đẹp hơn thực tế một cách
   sai lệch).
2. Devtools console → `localStorage.gitPlumPerf = '1'` → tải lại (F5).
3. Mở repo `D:/MyCompanyProjects/quanly-truong-phong-so`.
4. Dán `commitId` = `519fa0b0`, `path` = `yarn.lock` → bấm **Đo hai đường**.
5. Chép bảng kết quả (có sẵn dòng `spike-copy` để chép một phát).
6. Bấm **Kiểm word-level** → chép hai con số.

Đo ở profile **release** là bắt buộc: Phase 2 đo `parse_log` debug chậm gần **7
lần** (455 ms so với 67 ms) và làm tổng vượt mốc một giây. Số debug vô nghĩa.

---

## 7. Quyết định

> ⏭️ **ĐƯỜNG A (`@codemirror/merge`) — chọn KHÔNG CÓ SỐ ĐO.**
>
> Chủ dự án bỏ qua phép đo ngày 2026-09-22. Đây **không** phải kết luận từ ngưỡng mục 2:
> ngưỡng đó chưa được đối chiếu với bất kỳ con số nào. Đường A được chọn vì nó là mặc
> định khi thiếu bằng chứng — ít mã hơn — chứ không vì nó đã chứng minh đủ nhanh.

Ngưỡng ở mục 2 (250 ms lần tốt nhất / 400 ms lần tệ nhất) **vẫn còn hiệu lực** cho bất
kỳ lần đo nào về sau. Nó được chốt trước khi có số nào tồn tại (commit `d2b6dca`, kiểm
được bằng `git log`), nên nếu sau này đo và vượt ngưỡng thì kết luận là **chuyển sang
đường B**, không phải nới ngưỡng.

---

## 8. Hệ quả của việc bỏ qua, và điều kiện xem lại

**Rủi ro cụ thể đang nhận.** Checkpoint #3 tồn tại vì `@codemirror/merge` **tự tính diff
từ hai tài liệu đầy đủ**, trong khi `git diff` đã đưa sẵn diff. Với `yarn.lock` 632 KB —
tệp thật trong repo của chủ dự án — CodeMirror sẽ diff lại hai bản ~630 KB mỗi lần mở.
Không ai biết việc đó mất bao lâu. Ba khả năng, không phân biệt được mà không đo:

1. Đủ nhanh → đường A đúng, không mất gì.
2. Chậm thấy được nhưng dùng được → sẽ thành một phàn nàn ở exit gate dogfood (wave 5).
3. Chậm tới mức treo giao diện → phải viết lại toàn bộ trình xem theo đường B.

**Vì sao điều này đã từng xảy ra.** Phase 2 phải sửa đồ thị **ba vòng** vì viết xong mới
phát hiện sai. Chủ dự án chọn đặt checkpoint #3 ở wave 1 đúng để tránh lặp lại — rồi bỏ
qua chính phép đo đó. Ghi lại nguyên văn, không phán xét: đây là đánh đổi có ý thức giữa
5 phút bây giờ và rủi ro viết lại sau.

**Điều kiện xem lại, cụ thể để nhận ra được:**

- **Ở wave 5 (exit gate dogfood):** nếu mở diff của một tệp lớn cảm thấy chậm, đó **là**
  phép đo — công cụ vẫn còn (`SpikeHarness` + `spike_blob_pair`), chạy mục 6 rồi đối chiếu
  ngưỡng mục 2.
- **Trước khi xoá `SpikeHarness`:** đừng xoá cho tới khi wave 5 đóng. Nó là đường duy nhất
  lấy được số này mà không phải dựng lại.
- **Nếu 03-04 phải viết lại theo đường B:** `interface GraphRenderer` của Phase 2 là tiền
  lệ — 03-04 phải giữ trình xem sau một interface để việc đổi đường là sửa một tệp, không
  phải sửa cả giao diện.

**Hai thứ KHÔNG bị ảnh hưởng bởi quyết định này**, đã kiểm:

- **Word-level diff** lấy từ `git diff --word-diff=porcelain` (wave 3, đã xong). `spans` là
  dữ liệu thuần, dùng được cho cả A lẫn B.
- **Toàn bộ backend** (wave 2): parser, cache LRU, cổng DIFF-06. Không tệp nào của wave 2
  đọc quyết định A/B.

---

## 8. Những gì phép đo này KHÔNG nói

Phase 2 học được rằng đây là phần dễ bị chép nhầm thành kết luận nhất, nên nó được
viết ra **trước** khi có số:

1. **Không nói gì về tô màu cú pháp.** Spike cố ý **không** cài `@codemirror/language`
   hay `lang-*`. Tô màu là việc của 03-04; thêm nó vào phép đo sẽ trộn hai biến vào
   một con số. Đường A đạt ngưỡng ở đây **không** bảo đảm nó còn đạt khi có tô màu.
2. **Không nói gì về chế độ hợp nhất.** Phép đo dựng `MergeView` (hai cột).
   `unifiedMergeView` là một đường mã khác của cùng gói và chưa được đo.
3. **Không nói gì về cuộn.** Phép đo dừng ở lần vẽ đầu tiên. FPS lúc cuộn một diff
   lớn là câu hỏi riêng, và happy-dom không trả lời được (bài học Phase 2: ba lỗi
   hiển thị qua hết 212 test tự động).
4. **Không nói gì về tệp > 665 KB.** `yarn.lock` là tệp văn bản lớn nhất có thật
   trong các repo đang có. DIFF-06 (ngưỡng kích thước tệp) vẫn là câu hỏi mở của
   plan 03-02.
5. **Đường B ở đây là bản thô.** Nó tô cả dòng, không xử lý đổi tên, không xử lý
   `\ No newline at end of file`. Con số của B là **chi phí sàn**, không phải chi
   phí của một đường B hoàn chỉnh — một đường B thật sẽ chậm hơn con số này.
6. **Một lần đo trên một máy.** Máy phát triển là Windows 11 Pro với WebView2 bản
   hiện tại. Máy yếu hơn sẽ chậm hơn, và tỷ lệ chưa chắc tuyến tính.

---

## 9. Phát hiện ngoài dự kiến: một lỗi Phase 1 chặn cả Phase 3

Trong lúc dựng spike, phép đo đầu tiên trả `patch` **rỗng**. Nguyên nhân không nằm
ở spike mà ở `src-tauri/src/git/exec.rs` của Phase 1:

```rust
cmd.env("GIT_EXTERNAL_DIFF", "");   // ❌ sai
```

Ý định đúng (vô hiệu hoá trình diff ngoài của người dùng), cơ chế sai: git **không**
hiểu chuỗi rỗng là "không có trình nào". Nó thấy biến đã được đặt rồi đi spawn đúng
chương trình tên rỗng:

```
error: cannot spawn : No such file or directory
fatal: external diff died, stopping at file.txt
```

Hậu quả: **mọi** lệnh git diff sinh nội dung bản vá thoát 128 với stdout rỗng — im
lặng, không lỗi biên dịch. Gồm cả `--word-diff=porcelain` mà plan 03-03 dựa vào.

**Vì sao Phase 2 không phát hiện:** Phase 2 chỉ dùng `git diff --name-status`, vốn
**không gọi tới trình diff** nên không spawn gì. Lỗi ngủ yên đúng tới lúc Phase 3
cần bản vá thật.

**Cách sửa.** Đặt `diff.external=` rỗng trong `GIT_CONFIG_PARAMETERS` có **cùng
lỗi** (đã đo). Cách đúng là cờ `--no-ext-diff` của chính git: nó thắng **cả**
`diff.external` trong cấu hình **và** `GIT_EXTERNAL_DIFF` trong môi trường (đã đo cả
ba tổ hợp). Cờ được chèn tập trung trong `GitCommand::run()` cho `diff`/`show`/`log`/
`diff-tree`, vì người gọi sẽ quên — và việc người gọi quên chính là lỗi này.

Ba test hồi quy đi kèm, đã kiểm bằng đột biến: hoàn nguyên bản sửa → hai test đỏ.
