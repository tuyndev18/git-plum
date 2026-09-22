# Phase 3 — Xác minh từng requirement

**Ngày:** 2026-09-22
**Trạng thái phase:** 4/5 wave đã thực thi (03-01..03-04). **Checkpoint 11 bước của 03-04
CHƯA CHẠY**, và **checkpoint #3 của 03-01 BỊ BỎ QUA** — nên năm requirement (DIFF-01, 02, 03,
04, 06) còn ở mức "Có mã, chưa kiểm". **DIFF-05 chưa có mã vì wave 5 chưa chạy** (plan `03-05` đã nhận nó);
đó là một lỗ trong kế hoạch phase, không phải một ô chưa kiểm.

Tài liệu này nêu **bằng chứng** cho từng requirement, không nêu ý kiến. Ba mức, **đúng ba mức
mà Phase 2 đã dùng**:

- **Đạt** — có bằng chứng kiểm chứng được (test tự động chạy trên dữ liệu git thật, hoặc
  người dùng đã xác nhận bằng mắt).
- **Có mã, chưa kiểm** — đã cài và test tự động xanh, nhưng tiêu chí của requirement nói về
  thứ **người dùng quan sát được** mà chưa ai quan sát.
- **Chưa đo** — tiêu chí là một con số và con số đó chưa tồn tại.

**Không ô nào dưới đây được lấp bằng suy luận từ test tự động.** Đó là lỗi Phase 1 đã mắc
("đạt phần tự động hoá được" trong khi ba tiêu chí chưa ai chạy) và 02-07 đã ghi lại như một
điều không lặp. happy-dom **không tính layout CSS và không có cuộn thật**; ba lỗi hiển thị của
Phase 2 qua hết 212 test tự động, nên "test xanh" **không** là bằng chứng cho một tiêu chí nói
về thứ nhìn thấy được.

---

## Tóm tắt

| Req | Trạng thái | Bằng chứng ngắn |
|---|---|---|
| DIFF-01 | **Có mã, chưa kiểm** | `loadLanguage` + 20 test; 5 chunk lang tách riêng trong `dist`. **Chưa ai thấy màu trên màn hình** |
| DIFF-02 | **Có mã, chưa kiểm** | `viewMode` không-theo-repo + mutation #2 (5 đỏ). **Hai cột chưa bao giờ được render** — `ResizeObserver` không có trong happy-dom |
| DIFF-03 | **Có mã, chưa kiểm** | `nextHunkLine`/`prevHunkLine` trả `null` khi hết + mutation #6 (2 đỏ). Chưa ai bấm thử |
| DIFF-04 | **Có mã, chưa kiểm** | `Compartment` + mutation #9 (1 đỏ, cùng instance `EditorView`). **Vị trí cuộn chưa kiểm được** — không có cuộn thật |
| DIFF-05 | **Chưa cài — theo kế hoạch** | Là việc của **wave 5** (`03-05-PLAN.md`, `requirements: [DIFF-05]`, có `file_history.rs` + `FileHistory.tsx`). Chưa chạy, nên chưa có mã. **Không** phải lỗ kế hoạch — xem đính chính ở `03-04-SUMMARY.md` |
| DIFF-06 | **Có mã, chưa kiểm** | Năm thông báo khác nhau + 21 test; backend có test tích hợp chạy git thật (03-02). Giao diện chưa ai mở |
| Word-level | **Có mã, chưa kiểm** | `spanToUtf16` + mutation #5 (**4 đỏ**, tiếng Việt và emoji). **Tô đúng chỗ về pixel chưa kiểm** |
| Hiệu năng diff | **Chưa đo** | Checkpoint #3 **bị bỏ qua**. `MergeView` trên tệp 630 KB: **không có con số nào** |

**Đạt: 0** · **Có mã, chưa kiểm: 6** · **Chưa đo: 1** (hiệu năng) · **Chưa tới lượt: 1** (DIFF-05, wave 5)

Con số này **sẽ đổi** sau checkpoint 11 bước. Trước đó nó là trạng thái thật.

---

## Chi tiết

### DIFF-01 — tô màu cú pháp theo loại tệp · **Có mã, chưa kiểm**

**Có:**
- `src/lib/diff-render/langLoader.ts` — bảng tra cố định 9 phần mở rộng
  (`ts/tsx/js/jsx/rs/json/css/md/html`) → hàm `import()` động, nhớ kết quả trong `Map`.
- Gắn vào view qua `Compartment` nên đổi tệp không phải dựng lại `EditorView`.
- 20 test: khớp ngôn ngữ, phần mở rộng lạ → `null` (không ném), tệp không phần mở rộng
  (`Makefile`, `LICENSE`) → `null`, hoa/thường lẫn, bốn lớp đường dẫn biên.
- **Nạp lười đã kiểm ở tầng bundle**: `node scripts/check-lang-chunks.mjs` khẳng định chunk
  entry **không** chứa bảng parser nào, và 5 chunk không-entry có. Cổng **đã kiểm là đỏ được**
  (đổi sang `import` tĩnh → cổng đỏ với tên chunk và dấu hiệu thật).

**Chưa kiểm:** **không ai đã thấy một ký tự có màu.** Test khẳng định `loadLanguage` trả một
`Extension` khác `null`; nó **không** khẳng định CodeMirror vẽ token nào ra màn hình.
→ bước 6 của checkpoint.

---

### DIFF-02 — chuyển hợp nhất ↔ hai cột · **Có mã, chưa kiểm**

**Có:**
- `diffStore.viewMode` là trạng thái **không theo repo** (khuôn `uiStore`, bài học HIST-09).
- Mutation #2 (đổi sang `viewModeByRepo`) → **5 test đỏ**, gồm đúng ca "đổi chế độ ở repo A →
  repo B thấy cùng chế độ".
- Tự chuyển về hợp nhất dưới `MIN_SPLIT_WIDTH = 720px`, kèm một dòng nói vì sao. Hằng số sống
  hai phía (TS + CSS) và có test ghim; mutation lệch một phía → 1 đỏ.

**Chưa kiểm — và đây là lỗ lớn nhất của requirement này:** **chế độ hai cột chưa bao giờ được
render trong một test nào.** happy-dom không có `ResizeObserver`, nên `paneWidth` giữ `0` và
`shouldForceUnified(0) === true` — mọi test chạy ở nhánh **hợp nhất**. `MergeView` chỉ được
dựng trong mã sản phẩm.

Hệ quả cụ thể: "hai cột có **thẳng hàng**" hoàn toàn chưa có bằng chứng.
→ bước 3, 4 và 9 của checkpoint.

---

### DIFF-03 — nhảy tới khối thay đổi kế tiếp / trước đó · **Có mã, chưa kiểm**

**Có:**
- `nextHunkLine`/`prevHunkLine` trả `null` khi **hết khối**, và giao diện **nói ra**
  (`data-testid="diff-hunk-status"`: *"Đây là khối cuối — không nhảy vòng về đầu tệp."*).
- Neo vào `hunkIndex`, không vào "dòng thêm/xoá kế tiếp" — nên **hai hunk liền nhau** (không
  dòng ngữ cảnh giữa) vẫn là **hai** khối. Có test riêng cho ca đó.
- Mutation #6 (nhảy vòng thay vì `null`) → **2 đỏ**.
- Cả hai lệnh đi qua sổ đăng ký (PLAT-04), có test bấm nút + spy trên lệnh đã đăng ký.

**Chưa kiểm:** chưa ai bấm "khối sau" trên một tệp nhiều khối thật và thấy view cuộn tới đó.
Test khẳng định **số dòng đích** đúng; nó không khẳng định `scrollIntoView` làm gì.
→ bước 7 của checkpoint.

**Một điều cần chủ dự án xác nhận:** *không* nhảy vòng là quyết định có chủ ý, không phải
thiếu tính năng. Bước 7 hỏi rõ điều này.

---

### DIFF-04 — bật tắt hiển thị ký tự khoảng trắng · **Có mã, chưa kiểm**

**Có:**
- `highlightWhitespace()` gắn qua `Compartment`, đổi extension **tại chỗ**.
- Mutation #9 (đổi sang dựng lại `EditorView`) → **1 đỏ**: test khẳng định `EditorView` là
  **cùng một instance** trước và sau khi bật cờ, đọc từ `rendererStats.lastInstance`.
- Cờ là trạng thái **không theo repo**; có test "bật rồi đổi repo và đổi commit → còn nguyên".

**Chưa kiểm — hai điều:**
1. **Dấu cách và tab có hiện ra** trên màn hình: không; happy-dom không render glyph.
2. **Vị trí cuộn có giữ nguyên**: chưa. Test "cùng một instance" là điều kiện **cần** cho việc
   giữ cuộn, **không** đủ — một `dispatch` khác trong cùng instance vẫn có thể làm mất vị trí.
   happy-dom không có cuộn thật nên không đo được.

→ bước 8 của checkpoint, và nó hỏi đúng hai điều trên.

---

### DIFF-05 — lịch sử thay đổi của riêng một tệp · **Chưa cài, theo kế hoạch**

> **Đính chính 2026-09-22.** Bản đầu của mục này kết luận DIFF-05 là một **lỗ kế hoạch**.
> Sai. Orchestrator kiểm lại: `03-05-PLAN.md` nhận nó tường minh và có đủ tệp. Executor của
> 03-04 ghi "03-05 chưa lập kế hoạch chi tiết" trong khi plan đã tồn tại từ vòng lập kế hoạch
> ban đầu (`8d59856`, sửa ở `48d9944`/`857aee9`) — nó đọc thiếu.

DIFF-05 thuộc Phase 3 (ROADMAP: `Requirements: DIFF-01..DIFF-06`, tiêu chí thành công số 3:
*"Người dùng mở lịch sử thay đổi của riêng một tệp và lần theo được các phiên bản của nó"*)
và **đã được giao cho wave 5**:

| Wave | Phạm vi | DIFF-05? |
|---|---|---|
| 03-01 | spike đo A/B | không |
| 03-02 | backend diff, cổng DIFF-06 | không |
| 03-03 | word-level diff | không |
| 03-04 | giao diện DIFF-01..04, 06 | không (`requirements: [DIFF-01, DIFF-02, DIFF-03, DIFF-04, DIFF-06]`) |
| 03-05 | **DIFF-05** + exit gate dogfood | ✅ **CÓ** — `requirements: [DIFF-05]` |

Tệp mà `03-05` sẽ tạo: `src-tauri/src/git/parsers/file_history.rs`,
`src/components/diff/FileHistory.tsx`, cộng command và kiểu TS.

**Hiện chưa có mã** — không `git log --follow -- <path>` ở Rust, không component ở TS. Đúng
như kế hoạch: wave 5 chưa chạy. Đây là một ô **chưa tới lượt**, không phải ô bị bỏ sót.

Ghi chú thiết kế đã có sẵn trong `03-05-PLAN.md`: giới hạn một-đường-dẫn của `--follow`
**khớp chính xác** phạm vi DIFF-05 ("lịch sử của riêng **một** tệp"), nên nó không phải đánh
đổi mà là sự trùng khớp. Chi phí chặn bằng `--max-count=200`; timeout 30s riêng, không mượn
120s của `git log --all`; **không cache** vì lịch sử tệp phụ thuộc HEAD mà watcher chỉ tới ở
Phase 4.

---

### DIFF-06 — tệp nhị phân / quá lớn / LFS hiện thông báo nêu lý do · **Có mã, chưa kiểm**

**Có, ở hai tầng:**

*Tầng backend (03-02) — đây là tầng có bằng chứng mạnh nhất của cả phase:*
- Cổng DIFF-06 chạy **trước** `git diff`; mutation dời cổng xuống sau → test tích hợp đỏ kèm
  **bằng chứng lệnh đã chạy** từ `CommandLog`.
- `DiffKind::Binary` **không có trường nội dung** ở cả hai phía — chặn ở tầng **kiểu**
  (T-03-13), có test liệt kê toàn bộ khoá JSON.
- `lfsPointer.size` là số **trong con trỏ**; mutation đổi sang cỡ blob con trỏ → 1 đỏ
  (`132` vs `1048576`).
- Chạy trên repo mẫu thật `target/fixtures/diff-cases` (26 commit, 15 hình dạng).

*Tầng giao diện (plan này):*
- Năm dạng → **năm** thông báo khác nhau; có test khẳng định bốn chuỗi là **bốn chuỗi khác
  nhau** (không dùng chung một câu chung chung).
- `tooLarge` nêu **cả** kích thước thật **và** ngưỡng — có test cho cả hai số.
- `lfsPointer` nêu `kind.size`, và có khẳng định **phủ định** rằng không con số ~130 byte nào
  hiện ra. Mutation #4 → 1 đỏ.
- Bốn dạng không-`text` **không dựng `EditorView`** — 4 test khẳng định `created === 0`
  (T-03-29).

**Chưa kiểm:** chưa ai mở `binary.png`, `large.txt`, `pointer.bin` trong app thật và đọc thông
báo. Và câu hỏi *"ứng dụng có đứng hình dù một nhịp nào không"* không có test nào trả lời được.
→ bước 10 của checkpoint.

---

### Word-level (yêu cầu tường minh của chủ dự án) · **Có mã, chưa kiểm**

Không phải DIFF-01..06 nhưng là thứ chủ dự án yêu cầu riêng, nên nó có ô riêng.

**Có:**
- `spans` từ `git diff --word-diff-regex=...` (wave 3), dựng thành `Decoration.mark`.
- 🔴 **Phép chuyển byte → UTF-16** — chỗ nguy hiểm nhất của bàn giao. Mutation #5 (bỏ phép
  chuyển) → **4 test đỏ** trên nội dung tiếng Việt (`'xéy dỏng TEST'`: byte 12 ≠ UTF-16 9) và
  một emoji ngoài BMP (`'x🙂y CHANGED'`: byte 7 ≠ UTF-16 5).
- Test có khẳng định hai hệ **thật sự khác nhau** ở mỗi ca — nếu không, test vô nghĩa và
  mutation sẽ xanh. Ca ASCII thuần được giữ lại **kèm comment nói rõ nó không phải cổng**.
- Dòng có span → **cả** line-deco **lẫn** ≥1 mark-deco; dòng không span → chỉ line-deco (tô cả
  dòng, suy giảm đúng). Có test đếm **số deco thật**, không grep `'Decoration'`.
- Span lỗi từ backend (`start > end`, `end > byteLength`) → **bỏ span**, không ném (T-03-24).

**Chưa kiểm:** span có tô **đúng chỗ về pixel** hay không. Test chứng minh **chỉ số** đúng;
nó không chứng minh CodeMirror vẽ nền ở đúng ký tự đó.
→ bước 5 của checkpoint, và plan gọi nó là *"ca của chính bạn"*.

---

### Hiệu năng mở diff · **Chưa đo**

**Không có con số nào.** Checkpoint #3 của plan 03-01 **bị chủ dự án bỏ qua** ngày 2026-09-22
(`docs/09-phase3-diff-decision.md` mục 7, commit `6910732`).

Rủi ro đang nhận, ghi nguyên văn từ mục 8 của tài liệu đó: `@codemirror/merge` tự tính diff từ
**hai tài liệu đầy đủ**, nên với `yarn.lock` 632 KB nó diff lại hai bản ~630 KB **mỗi lần
mở**. Ba khả năng, không phân biệt được mà không đo:

1. đủ nhanh → không mất gì;
2. chậm thấy được nhưng dùng được → thành phàn nàn ở exit gate (wave 5);
3. chậm tới mức treo giao diện → phải viết lại theo đường B.

**Ngưỡng vẫn còn hiệu lực** (chốt trước khi có số nào, commit `d2b6dca`): lần tốt nhất trong 3
≤ **250 ms**, lần tệ nhất ≤ **400 ms**. Vượt ngưỡng → **chuyển đường B**, không nới ngưỡng.

**Hai điều làm giảm rủi ro, đo được:**
- Chế độ **hợp nhất** **không** để CodeMirror tính lại diff (dựng `DecorationSet` từ đầu ra
  `git diff`), và hợp nhất là **mặc định**. Nguy cơ áp cho **một** chế độ không mặc định.
- Trình xem nằm sau `interface DiffRenderer` với cổng kiểm biên giới (7 test), nên chuyển sang
  đường B là **sửa một tệp** — đúng điều kiện mục 8 đặt ra.

**Công cụ đo vẫn còn:** `SpikeHarness` + `spike_blob_pair`. Mục 8 nói đừng xoá tới khi wave 5
đóng. Cách chạy: mục 6 của `docs/09-phase3-diff-decision.md`.

Bước 11 của checkpoint 03-04 trả **một phần** nợ này (đo bằng `localStorage.gitPlumPerf` trên
repo công ty 398 commit) nhưng nó **không** thay phép đo `yarn.lock` 632 KB của checkpoint #3.

---

## Điều gì sẽ đổi trạng thái các ô trên

| Ô | Việc cần làm |
|---|---|
| DIFF-01, 03, 06 | Checkpoint 11 bước — bước 6, 7, 10 |
| DIFF-02 | Checkpoint bước 3, 4, 9. **Hoặc** Playwright trên Chromium thật nếu checkpoint bị hoãn (quy trình 02-05: cài ở scratch, không vào `package.json`) |
| DIFF-04 | Checkpoint bước 8 — nó hỏi cả "có hiện ra" lẫn "cuộn có giữ nguyên" |
| Word-level | Checkpoint bước 5 |
| Hiệu năng | Mục 6 của `docs/09-phase3-diff-decision.md`, đối chiếu ngưỡng mục 2 |

**Nếu checkpoint bị hoãn:** cả sáu requirement giữ mức "Có mã, chưa kiểm", và Task 1 của plan
03-05 (exit gate) là chỗ đầu tiên phát hiện lỗi hiển thị — muộn hơn, nhưng vẫn trước khi đóng
phase.
