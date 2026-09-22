# Phase 5: Staging theo khối và an toàn khi huỷ — Context

**Ngày:** 2026-09-22
**Mode:** mvp (slice dọc theo tính năng, không tầng ngang)
**Goal (ROADMAP):** Người dùng tạo được commit sạch sẽ bằng cách chọn từng khối thay đổi,
và **không bao giờ mất dữ liệu** vì một lần huỷ nhầm.
**Requirements:** WORK-03, WORK-04, WORK-05, WORK-06, WORK-07

---

## 0. Phase này bắt đầu trên nền CHƯA ĐƯỢC NHÌN — đọc trước mọi thứ khác

Phase 4 có **đủ mã** cho cả sáu requirement, test xanh (668 frontend, 309 `cargo test
--lib`), bản release dựng lúc **2026-09-22 21:18:40** từ commit `9fef159`. Nhưng:

> **Không requirement nào của Phase 4 có bằng chứng từ mắt người.**
> `VERIFICATION.md` của nó ghi `mắt người (release)` đúng ba lần: một lần ở danh sách
> định nghĩa, hai lần ở các câu **phủ định**. Nó không được cấp cho thứ gì.

Hai cổng còn đỏ: checkpoint 12 bước và **exit gate dogfood** (một commit thật, chỉ bằng
git-plum). `docs/09-phase4-dogfood.md` **chưa tồn tại**.

### Điều đó đổi cách lập kế hoạch Phase 5, không chỉ là một ghi chú

Phase 5 xây thẳng lên `ChangeList`, `statusStore`, `get_worktree_diff` của Phase 4 —
những thứ **chưa ai bấm thử**. Xác suất còn lỗi hiển thị là **cao**, không phải giả
định bi quan: chủ dự án đã tìm **năm** lỗi hiển thị của Phase 3 chỉ bằng cách mở ứng
dụng, và cả năm đều đi qua hàng trăm test tự động.

Wave 5 của Phase 4 còn tìm được một lỗi **vòng khoá chết** mà không test cũ nào thấy:
`ChangeList` là chỗ duy nhất gọi `refresh`; nó chỉ mount khi vùng soạn commit đã mở;
đường duy nhất mở vùng soạn là bấm hàng WIP; hàng WIP chỉ hiện khi `statusStore` đã có
dữ liệu. Bốn điều kiện khoá vòng → **hàng WIP không bao giờ hiện**. Cùng lớp lỗi có thể
đang nằm ở những đường chưa ai đi.

**Hệ quả bắt buộc cho việc chia plan:**

1. **Tầng Rust của Phase 5 phải tự đứng được.** Áp bản vá byte thô, `git apply --check`,
   kiểm blob hash, `--recount` — tất cả kiểm được bằng repo mẫu và `git` thật, **không**
   phụ thuộc giao diện Phase 4 đúng. Wave đầu phải là tầng này.
2. **Tầng giao diện của Phase 5 phải chịu được việc Phase 4 còn lỗi.** Nếu `ChangeList`
   hoá ra hiển thị sai, staging theo khối vẫn phải dùng được từ một đường vào khác.
3. **Không plan nào được ghi "Đạt" cho một tiêu chí hình ảnh** dựa trên test happy-dom.

---

## 1. Phạm vi

| Req | Nội dung |
|---|---|
| **WORK-03** | Đưa **một khối** thay đổi vào vùng chờ và lấy ra khỏi vùng chờ |
| **WORK-04** | CRLF, thiếu dòng trống cuối, byte không UTF-8 — **giữ nguyên** sau staging một phần |
| **WORK-05** | Tệp đã đổi từ lúc vẽ diff → **báo và đòi làm mới**, không áp bản vá không khớp |
| **WORK-06** | Huỷ bỏ thay đổi theo **tệp** và theo **khối** |
| **WORK-07** | Trước mỗi lần huỷ, **tự lưu** nội dung bị huỷ; khôi phục được từ danh sách |

**Ngoài phạm vi:** staging theo **dòng** — ROADMAP nói rõ "khó hơn hẳn và không có
trong Active", để v2.

---

## 2. Quy tắc đúng đắn cứng — ROADMAP gọi là "không thương lượng"

Chép nguyên văn vì mỗi dòng là một lớp lỗi đã biết:

- **Bản vá là byte thô từ đầu đến cuối. Không bao giờ giải mã thành chuỗi.** Truyền
  `--recount`. **Tách stderr khỏi stdout** để cảnh báo của clean filter không bao giờ
  lọt vào nội dung bản vá.
- **Kiểm lại mã băm blob của tệp ngay trước khi áp**; chạy `git apply --check --cached`
  trước.
- Khi thất bại, **không bao giờ** thử lại bằng khớp mờ, unidiff không ngữ cảnh, hay
  `--whitespace=fix`. Hiện thông báo *"tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm
  mới"*.
- Xử lý `core.autocrlf` **tường minh**.
- Với tệp **chưa theo dõi**, động từ phải là **"Xoá"**, không phải "Huỷ bỏ".
- **An toàn khi huỷ:** trước mỗi lần huỷ, `git stash create` các đường dẫn bị ảnh hưởng
  và **giữ object đó có thể tới được** dưới một không gian ref riêng cho thùng rác, kèm
  danh sách "Vừa huỷ gần đây" để khôi phục.

### 2.1 Vì sao "byte thô" là điều kiện sống còn ở đây

Phase 3 và Phase 4 đã trả giá hai lần cho việc giải mã sớm:

- `GitCommand::arg` gọi `to_string_lossy()`, nên một thông điệp commit không UTF-8
  **không thể** đi qua đường đó nguyên vẹn — wave 3 ghi nhận và nói rõ ca đó **không
  tới được** qua API hiện tại vì `create_commit` nhận `String`.
- Bản vá thì **khác hẳn**: nội dung tệp của người dùng đi thẳng vào bản vá. Một tệp
  Latin-1, một tệp có BOM, một tệp nhị phân lọt vào đường text — tất cả đều là dữ liệu
  thật trên máy người dùng, không phải ca giả định.

**Ranh giới bắt buộc:** bản vá đi từ `git diff` đến `git apply` mà **không** qua `String`
ở bất kỳ điểm nào. Dùng `Vec<u8>` / `bstr` suốt đường, đúng như `CLAUDE.md` đã chốt.

### 2.2 `--recount` và vì sao nó không phải tuỳ chọn

Khi người dùng chọn một tập con các khối, số dòng trong header `@@` **không còn đúng**.
`--recount` bảo git tính lại từ nội dung thật. Thiếu nó, git từ chối bản vá hoặc — tệ
hơn — áp sai vị trí.

**Ca kiểm bắt buộc:** chọn khối **thứ hai** trong một tệp có ba khối. Chỉ chọn khối đầu
thì header tình cờ vẫn đúng và đột biến bỏ `--recount` **không đỏ** — đúng lớp lỗi "hình
dạng đúng, dữ liệu vô hại" ở mục 4.2.

---

## 3. Rủi ro riêng — ROADMAP xếp phase này HIGH

> *"Đây là phase nhiều khả năng vượt kế hoạch nhất. Nghiên cứu xếp staging theo khối ở
> mức HIGH và khuyến nghị ngân sách gấp 2–3 lần ước lượng ngây thơ. Đánh giá lại lịch ở
> cuối phase này trước khi cam kết phạm vi Phase 6."*

| # | Rủi ro | Vì sao thật | Giảm nhẹ |
|---|---|---|---|
| R1 | Bản vá áp sai vị trí, hỏng tệp người dùng | Đây là phase **đầu tiên** ghi vào nội dung tệp, không chỉ vào index | `--check` trước, blob hash trước, **không bao giờ** thử lại mờ |
| R2 | CRLF bị chuẩn hoá mất | `core.autocrlf` mặc định `true` trên Windows — máy chủ dự án | Ca kiểm riêng, đo bằng `od -c` chứ không bằng mắt |
| R3 | Huỷ nhầm mất việc | Chính requirement WORK-07 tồn tại vì điều này | `git stash create` + ref riêng **trước** mỗi lần huỷ |
| R4 | Tệp đổi giữa lúc xem diff và lúc bấm stage | Chủ dự án chạy git ở terminal song song — cả điểm của WORK-10 | So blob hash, báo và đòi làm mới |
| R5 | `index.lock` khi làm mới tự động chồng lên stage do người dùng bấm | ROADMAP liệt kê tường minh trong "ngoài đường hạnh phúc" | Wave 2 Phase 4 đã có `GitError::IndexLocked` + thử lại có giãn cách — **dùng lại**, đừng viết mới |
| R6 | Xây trên `ChangeList` chưa ai nhìn | Mục 0 | Tầng Rust tự đứng; giao diện có đường vào thay thế |
| R7 | Tệp nhị phân lọt vào đường text | ROADMAP liệt kê | Phát hiện sớm, từ chối rõ ràng, **không** đoán |

### Ca kiểm ngoài đường hạnh phúc, nguyên văn ROADMAP

sửa tệp giữa lúc mở diff và lúc bấm stage · tệp nhị phân · tệp không có dòng trống cuối ·
tệp vừa đổi tên vừa sửa · chuẩn hoá CRLF/LF giao với staging một phần · một công cụ build
đang ghi vào thư mục làm việc (CPU phải có chặn trên) · một lần tự làm mới chồng lên một
lần stage do người dùng khởi tạo (không được để lỗi index-lock lọt tới người dùng).

---

## 4. Bài học đã trả giá — áp dụng, không đọc rồi bỏ

### 4.1 Cổng không thể fail — đã xảy ra **chín** lần

| # | Phase | Sai vì |
|---|---|---|
| 1 | 3 | grep khớp **chú thích và văn xuôi**, mã thật sạch |
| 2 | 3 | đòi **đúng 1** khớp grep, nhưng import + chỗ gọi luôn ≥ 2 |
| 3 | 3 | đường dẫn tệp sai → grep không thấy gì → xanh |
| 4 | 3 | fixture **không phân biệt** được đột biến |
| 5 | 3 | grep trên nguồn **thô** khớp chú thích **do chính mutation sinh ra** |
| 6 | 3 | `waitFor` neo vào phần tử có mặt ở **mọi** trạng thái |
| 7 | 4 | fixture đúng **hình dạng** nhưng dữ liệu vô hại → lệch nấc không quan sát được |
| 8 | 4 | cổng chỉ kiểm `numFailedTests > 0` → **xanh trên suite chạy 0 test** |
| 9 | 4 | **không có test nào hỏi về thuộc tính đang dùng** → đổi `marginTop`→`paddingTop` qua cả suite |

**Lỗi #9 là lỗi đắt nhất về bản chất:** một từ, đúng lớp lỗi "lệch một hàng" đã xảy ra
hai lần ở Phase 2, và **0 đỏ trên 666 test**. Nó sống sót vì **không ai từng viết test
hỏi về thuộc tính đó**. Cách phát hiện: grep toàn bộ `src/**/*.test.*` cho tên thuộc
tính → **rỗng**.

**Quy tắc bắt buộc cho Phase 5:**

- Cổng đọc nguồn **phải bỏ chú thích trước khi tìm**.
- Cổng phải **khẳng định tiền đề**: không tìm thấy ⇒ **đỏ**, không phải xanh.
- Mọi cổng vitest phải khẳng định `numTotalTests` tối thiểu **trước** khi tin
  `numFailedTests`.
- Fixture cho một đột biến phải được **chứng minh phân biệt được**.
- **Trước khi kết luận một đột biến sống sót vì mã đúng**, kiểm xem có test nào **hỏi
  về thứ đó** không. Rỗng nghĩa là cổng thiếu, không phải mã đúng.
- **Đột biến không biểu diễn được lỗi thì không nói gì về cổng.** Wave 3 tốn ba lần thử
  ở M14: hai lần đầu 0 đỏ vì đột biến **vô hiệu**, không phải vì cổng yếu.
- **Không bao giờ `git checkout --` một tệp chưa commit để hoàn nguyên đột biến** — đã
  xoá mất việc của một executor. Chép tệp ra chỗ khác trước.

### 4.2 "Hình dạng đúng, dữ liệu vô hại" — ba lần, hai phase

| Ở đâu | Fixture đúng gì | Dữ liệu vô hại thế nào |
|---|---|---|
| Phase 2 nguyên nhân B | có badge ref, có chữ | badge **ngắn** → badge thật 258px nuốt cột, chữ hiện **0%** |
| Phase 4 wave 1 M1 | bản ghi `2` + ≥2 bản ghi sau | đường dẫn cũ **không khớp dạng nào** → lệch nấc bị `_ => {}` nuốt |
| Phase 4 wave 3 M14 (dự đoán) | hai repo, đổi giữa trì hoãn | *(dự đoán của tôi **sai** ở cài đặt này — đo mới biết)* |

**Phép kiểm:** hỏi *dữ liệu này có cho lỗi để lại dấu vết quan sát được không* — không
hỏi *fixture có đúng hình dạng không*.

Với Phase 5 điều này rất cụ thể: một bản vá chỉ có **một** khối, hoặc một tệp chỉ dùng
LF, hoặc một tệp toàn ASCII — tất cả đều là fixture "đúng hình dạng, vô hại".

### 4.3 Phân tích byte — ba lần hỏng im lặng

- `%x1f` trong `--format` (02-04): thoát **0**, stdout trông hợp lý, thanh bên **rỗng**.
- `git log --follow --name-status -z` phân tách bằng `\0\n`, không phải `\0` trơn.
- `--porcelain=v2` bản ghi dạng `2` mang **hai** đoạn NUL trong **một** bản ghi.

**Cho Phase 5:** đầu ra `git diff` là byte, và bản vá đưa vào `git apply` qua **stdin**
cũng là byte. Mọi chỗ đọc/ghi phải `od -c` một lần trên git thật trước khi tin.

### 4.4 `GIT_EXTERNAL_DIFF` — lỗi Phase 1 tốn nhất

`cmd.env("GIT_EXTERNAL_DIFF", "")` **không** tắt external diff; git spawn chương trình
tên `""` và **mọi** lệnh sinh bản vá thoát 128 với stdout rỗng, **im lặng**. Một chương
trình **không tồn tại** còn tệ hơn: `show`/`diff`/`diff-tree` thoát **0**, 0 hunk, không
stderr. Đã sửa bằng `env_remove` + `--no-ext-diff` tiêm tập trung (`f5c4c17`), cờ đặt
**sau** subcommand.

**Cho Phase 5:** mọi lệnh git mới đi qua `git/exec.rs`. Không tự spawn git chỗ khác.

### 4.5 happy-dom không tính CSS layout, không có cuộn thật

Đây là lý do cả năm lỗi hiển thị Phase 3 và lỗi `marginTop` Phase 4 đều thoát test.

**Cho Phase 5:** mọi khẳng định về **layout** (khối được chọn nhìn ra khác khối không
chọn, nút stage-khối nằm đúng chỗ, danh sách "vừa huỷ" không tràn) là **chưa kiểm** cho
tới khi có người xem trên Chromium thật.

---

## 5. Bẫy môi trường — đã tốn thời gian thật hôm nay

- 🔴 **Một con số có thể nói dối, và nó đã nói dối đúng lúc ra quyết định.**
  `git status --porcelain <paths> | wc -l` trả `0` trong khi cùng lệnh không qua ống trả
  ` M src/App.tsx`. `rtk proxy git status --porcelain <paths>` **cũng** trả rỗng cho tệp
  dirty, không đường ống nào. Rồi 20 lần chạy sau đó đều đúng — **chập chờn**.

  **Với phép kiểm quyết định có chạy tiếp hay không: in đầu ra thô và nhìn.** Bọc
  `[$(cmd)]` để phân biệt "rỗng" với "trắng". Đếm chỉ dùng cho việc không quan trọng.
- 🔴 **`rtk` làm hỏng mẫu `grep` có `\|`** — trả `0 matches` cho mẫu thật sự khớp. Dùng
  công cụ Grep thay vì `grep` qua shell khi kết quả quan trọng.
- **`git log` bị cắt còn 50 dòng**, mọi cách gọi. Dùng `git rev-list` /
  `git merge-base --is-ancestor`. `git log | grep <sha>` trả exit 1 và 0 khớp cho commit
  **có tồn tại** — ba lần báo động sai "mất commit" hôm nay vì điều này.
- **`.vitest/json/output.json` bị cache và bị `rtk` ghi đè** sau lệnh `--outputFile` của
  chính mình. `rm -f` trước mỗi lần, đọc **sau** khi chạy xong, đọc lại nếu số vô lý.
- **Phân biệt cargo bị chặn với cargo đang làm bằng CPU, không bằng thời gian trôi:**
  bị chặn ≈ 0,8 s CPU / 17 phút; làm thật ≈ 16,7 s CPU / 0,3 phút. **Luôn in
  `CommandLine`** — một tiến trình tôi suýt gán cho mình là `tauri dev` của chủ dự án.
- **Ứng dụng đang mở khoá `cargo test --lib --tests`** (`os error 5`) vì cargo relink
  binary. `cargo test --lib` vẫn chạy. **Không kill** tiến trình không phải của mình —
  cổng dogfood cần app chạy được. Báo `--lib` là đo được, `--lib --tests` là **chưa đo**.
- **Đĩa chạm 99% hai lần hôm nay**, hỏng build với `LNK1318 PDB limit` và
  `paging file is too small` (os error 1455). `target/debug/incremental` là cache thuần,
  xoá được **không mất gì** — hôm nay giải phóng 6,5 GB bằng cách đó. `cargo clean` thì
  đắt hơn nhiều: rebuild 292 crate.
- **Ba đến năm phiên Claude cùng repo**, dùng chung một `target/`. Mọi lời gọi cargo
  nối đuôi trên `target/debug/.cargo-lock`.

---

## 6. Mốc hiện tại — đừng làm tụt

| | |
|---|---|
| Frontend | **668 passed, 0 failed** |
| `cargo test --lib` | **309** |
| `cargo test --lib --tests` | **425 passed, 1 ignored** (lần đo gần nhất được) |
| `npx tsc --noEmit` | sạch |
| `cargo clippy --all-targets` | cảnh báo còn lại là **có trước**, không thêm mới |
| Bản release | **2026-09-22 21:18:40**, commit `9fef159` |

**Không có `npm run lint`** — đừng đưa vào cổng.

---

## 7. Điều kiện đóng phase

ROADMAP không ghi exit gate dogfood riêng cho Phase 5, nhưng bốn tiêu chí thành công
đều nói về thứ **người dùng làm**:

1. Chọn một khối trong tệp nhiều thay đổi → commit chứa **đúng** khối đó
2. Sau staging một phần: CRLF giữ nguyên, thiếu dòng cuối giữ nguyên, byte không UTF-8
   **không** bị thay
3. Tệp đổi ở nơi khác → **báo và đòi làm mới**, không áp bản vá không khớp
4. Huỷ theo tệp và theo khối, rồi **khôi phục lại được** từ danh sách

Tiêu chí 2 kiểm được bằng `od -c` trên repo mẫu — **máy làm được**. Tiêu chí 1, 3, 4 có
phần máy kiểm được (bản vá sinh ra đúng chưa) và phần **chỉ người kiểm được** (người
dùng có chọn đúng khối mình muốn không, thông báo có đọc hiểu được không).

**Và phải nhắc lại:** Phase 4 vẫn nợ hai cổng. Đóng Phase 5 mà Phase 4 chưa đóng nghĩa
là **bốn phase liên tiếp** đóng với nợ kiểm chứng.

---

## 8. Phối hợp nhiều phiên

Ba đến năm phiên Claude đang mở trên repo này. Tại thời điểm viết, còn chưa commit và
**không thuộc phase này**: `src/components/RefSidebar.tsx` + test, và một số tệp
`.planning/*.md`.

**Quy tắc:** chỉ ghi vào `.planning/phases/05-hunk-staging/` và mã mới của Phase 5.
Stage theo **đường dẫn tường minh**, không bao giờ `git add -A`. Trước mỗi commit, in
`git diff --cached --name-only` dạng **thô** và nhìn.
