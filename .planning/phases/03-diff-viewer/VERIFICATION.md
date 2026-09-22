# Phase 3 — Xác minh từng requirement

**Ngày:** 2026-09-22
**Trạng thái phase:** **5/5 wave đã thực thi** (03-01..03-05). DIFF-05 nay **có mã** (wave 5).

Nhưng **cả sáu requirement vẫn ở mức "Có mã, chưa kiểm"**, vì cả ba cổng dùng-mắt-người đều
chưa chạy hoặc bị bỏ qua:

- **checkpoint #3 của 03-01 BỊ BỎ QUA** (hiệu năng diff — không có số nào)
- **checkpoint 11 bước của 03-04 CHƯA CHẠY**
- **cổng thoát dogfood của 03-05 CHƯA CHẠY** — xem `docs/08-phase3-dogfood.md`

Nên **Phase 3 chưa đóng**. Đó là **phase thứ ba liên tiếp** đóng với nợ kiểm chứng
(Phase 1: ba tiêu chí; Phase 2: hai checkpoint; Phase 3: cổng thoát), và khuôn hình đó
được ghi ra ở đây thay vì để nó tích lại im lặng.

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
| DIFF-05 | **Có mã, chưa kiểm** | `parse_file_history` + `get_file_history` + `FileHistory.tsx`; 15 test đơn vị, 10 tích hợp **chạy git thật**, 25 test giao diện. Mutation `\0\n` → **7 đỏ**; bỏ `--follow` → **2 đỏ**. **Chưa ai thấy danh sách trên màn hình** |
| DIFF-06 | **Có mã, chưa kiểm** | Năm thông báo khác nhau + 21 test; backend có test tích hợp chạy git thật (03-02). Giao diện chưa ai mở |
| Word-level | **Có mã, chưa kiểm** | `spanToUtf16` + mutation #5 (**4 đỏ**, tiếng Việt và emoji). **Tô đúng chỗ về pixel chưa kiểm** |
| Hiệu năng diff | **Chưa đo** | Checkpoint #3 **bị bỏ qua**. `MergeView` trên tệp 630 KB: **không có con số nào** |

**Đạt: 0** · **Có mã, chưa kiểm: 7** · **Chưa đo: 1** (hiệu năng) · **Chưa tới lượt: 0**

Con số này **sẽ đổi** sau checkpoint 11 bước và sau cổng thoát. Trước đó nó là trạng thái
thật: **không một requirement nào của Phase 3 có bằng chứng từ mắt người.**

**Một giới hạn đã biết, KHÔNG phải một ô chưa kiểm:** merge commit không xuất hiện trong
lịch sử tệp (DIFF-05). Đo được, có test ghim, và cờ trông-như-bản-sửa
(`--diff-merges=first-parent`) đã đo là **tệ hơn** — xem mục DIFF-05 và
`docs/08-phase3-dogfood.md`.

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

### DIFF-05 — lịch sử thay đổi của riêng một tệp · **Có mã, chưa kiểm**

**Đã cài ở wave 5** (plan `03-05`). Trước đó ô này là "chưa tới lượt"; nay có mã và
có test tự động, nhưng **chưa ai thấy danh sách trên màn hình**.

**Có, ở hai tầng:**

*Tầng backend (`git/parsers/file_history.rs` + `commands/diff.rs`):*
- `parse_file_history` đọc `git log --follow --max-count=200 --format=<FMT>
  --name-status -z -- <path>`. **15 test đơn vị** cộng **10 test tích hợp chạy git
  thật** trên repo mẫu.
- 🔴 **Dấu phân tách `\0\n` đã xử lý**, và đây là chỗ hỏng-im-lặng duy nhất của tệp.
  Đo bằng `od -c` trên git 2.54.0.windows.1: git kết thúc phần `--format=` bằng `\0`
  rồi in `\n` **trước** khối `--name-status`. Một bộ phân tích chỉ tách `\0` đọc
  `status` thành `"\nM"`, phép so `status == "M"` trượt **không một tiếng nào**, và
  danh sách về rỗng. Mutation bỏ bước cắt → **7 test đỏ** (5 đơn vị + 2 tích hợp),
  trong đó một test in ra đúng `left: "\nM" / right: "M"`.
- **Đổi tên lần theo được.** Repo mẫu có `renamed.txt` → `renamed-new.txt` (`R077`);
  test khẳng định có phiên bản mang `old_path` **và** có phiên bản mang tên cũ phía
  trước chỗ đổi tên. Mutation bỏ `--follow` → **2 đỏ**. Đây là cổng đúng cho `--follow`:
  một `grep` sẽ khớp doc comment giải thích quyết định dùng nó.
- `R`/`C` chiếm **hai** đường dẫn; test đặt **hai** bản ghi sau bản ghi `R` để bắt
  lệch nấc. Mutation đọc `R` như một đường dẫn → **3 đỏ**, và đỏ ở bản ghi **sau** nó
  (`left: 1 / right: 3`).
- `--` **trước** pathspec (T-03-32), kiểm bằng test **đọc thân hàm thật** đã bỏ chú
  thích. Mutation xoá `.arg("--")` → **2 đỏ** (sau khi sửa một cổng xanh sai — xem
  `03-05-SUMMARY.md`).
- Chi phí chặn hai lớp: `--max-count=200` (T-03-33) và `DEFAULT_TIMEOUT` 30 s riêng,
  **không** mượn `HISTORY_TIMEOUT` 120 s. **Không cache** — lịch sử tệp phụ thuộc
  HEAD, và watcher chỉ tới ở Phase 4.
- Bản ghi méo → bỏ, **đếm**, `tracing::warn!` ghi `repo.id` và **số**, không ghi path
  (T-03-36 / T-03-38). Tên tệp không UTF-8 → lossy, bản ghi sau **không mất** (HIST-11).

*Tầng giao diện (`components/diff/FileHistory.tsx`):*
- **25 test**. Danh sách phẳng, **không** ảo hoá: chặn 200 ở tầng git nghĩa là DOM
  không bao giờ vượt 200 hàng (T-03-34).
- Bấm một phiên bản đặt `diffStore.historyCommitOverride`, **không** ghi vào
  `selectionStore`. Mutation đổi sang `selectionStore.select()` → **3 đỏ**.
- Chỗ đổi tên **hiện ra** ("đổi tên từ `<tên cũ>`"); `truncated` hiện "200 phiên bản
  gần nhất" **trên giao diện**, không chỉ trong payload.
- Ba tình huống rỗng là **ba câu khác nhau** (đang nạp / lỗi / không có trong lịch sử),
  khuôn năm thông báo của DIFF-06.
- Cổng chống đua (T-03-37): mutation bỏ → **1 đỏ**. `authorTime` là **giây**; test
  khẳng định không hiện 1970 **và** khẳng định `new Date(giây)` *sẽ* cho 1970 — thiếu
  khẳng định thứ hai thì khẳng định thứ nhất không chứng minh gì.
- Lệnh qua sổ đăng ký (PLAT-04) và **gỡ khi unmount**; mutation bỏ `unregisterCommand`
  → **1 đỏ** (test mount hai lần).

**Chưa kiểm — và đây là toàn bộ phần còn lại:**

1. **Không ai đã thấy danh sách phiên bản trên màn hình.** happy-dom không tính layout
   CSS, nên bốn cột có **thẳng hàng** không, tiêu đề dài có bị cắt kèm ellipsis hay
   **biến mất**, panel có **đẩy diff ra khỏi khung** không — cả ba chưa có bằng chứng.
   Cổng cấp hai là `app.css.test.ts` (đọc nguồn CSS, 7 test mới, **đã kiểm là đỏ được**
   ở cả ba điều: containment, `minmax(0,`, sàn `min-width`).
2. **Chưa ai bấm một phiên bản trong app thật** và thấy diff đổi sang commit đó trong
   khi hàng đang sáng trên đồ thị **không nhảy**. Test khẳng định giá trị store; nó
   không khẳng định người dùng nhìn thấy điều đó.
3. ⚠️ **Giới hạn đã biết, không phải ô chưa kiểm:** merge commit **không** xuất hiện
   trong lịch sử tệp — kể cả merge đã giải quyết xung đột trong chính tệp đó. Đo được,
   có test ghim. `--diff-merges=first-parent` **không** phải bản sửa: đo trên
   `dau-tri-toan-hoc` (1140 commit, 39 merge) cho 104 → **5715** bản ghi cho
   `.planning/STATE.md`, vì cờ đó làm git in bản vá ra cùng luồng và `--max-count`
   mất tác dụng. Chi tiết ở `docs/08-phase3-dogfood.md`.

→ **cổng thoát** (Task 3 của `03-05`) là chỗ kiểm cả ba.

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
| DIFF-05 | **Cổng thoát dogfood** — ba điều ở mục DIFF-05: danh sách có hiển thị đúng, bấm một phiên bản có đổi diff mà không nhảy đồ thị, và bốn cột có thẳng hàng |
| Word-level | Checkpoint bước 5 |
| Hiệu năng | Mục 6 của `docs/09-phase3-diff-decision.md`, đối chiếu ngưỡng mục 2. **Hoặc** cổng thoát: mục 8 của tài liệu đó nói rõ *"nếu mở diff của một tệp lớn cảm thấy chậm, đó **là** phép đo"* |

**Không còn wave nào để hoãn việc sang.** 03-04 ghi rằng nếu checkpoint 11 bước bị hoãn thì
cổng thoát của 03-05 là "chỗ đầu tiên phát hiện lỗi hiển thị". Wave 5 nay đã xong, nên cổng
thoát **là** chỗ đó — và nó là cổng cuối trước khi Phase 4 dựng lên trên trình xem này
(WORK-01 đọc diff viewer tường minh).

**Nếu cổng thoát cũng bị hoãn:** cả bảy requirement giữ mức "Có mã, chưa kiểm", Phase 3 đóng
với nợ, và lỗi hiển thị đầu tiên sẽ được phát hiện trong lúc Phase 4 đang xây trên nó — tức
chi phí sửa nhân lên, đúng điều ROADMAP đặt cổng này để tránh.
