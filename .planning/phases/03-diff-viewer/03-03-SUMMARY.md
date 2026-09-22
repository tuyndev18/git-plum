---
phase: 03-diff-viewer
plan: 03
subsystem: diff-viewer
tags: [backend, parser, word-diff, diff-01, ipc-contract]
requires:
  - "src-tauri/src/domain/diff.rs (DiffLine, Hunk — từ 03-02)"
  - "src-tauri/src/git/parsers/patch.rs (parse_patch — bất biến so khớp byte)"
  - "src-tauri/src/commands/diff.rs (lay_diff_tep, bảy bước của 03-02)"
  - "target/fixtures/diff-cases (empty-line, one-side, mixed, dup-lines từ 03-02)"
provides:
  - "domain::diff::Span { start, end } — khoảng BYTE vào DiffLine.content"
  - "domain::diff::DiffLine.spans: Vec<Span> — mặc định rỗng, #[serde(default)]"
  - "git::parsers::word_diff::parse_word_diff(&[u8]) -> WordDiffParse"
  - "git::parsers::word_diff::WORD_DIFF_REGEX — biên từ dùng chung hai bên"
  - "commands::diff::WORD_DIFF_MAX_CHANGED_LINES = 2000"
  - "ipc.ts Span + DiffLine.spans"
affects:
  - "03-04 (giao diện) — vẽ decoration từ spans; PHẢI chuyển byte sang UTF-16 trước"
  - "Phase 5 (staging theo khối) — no_newline_at_eof vẫn do parse_patch giữ, không bị ghi đè"
tech-stack:
  added: []
  patterns:
    - "Biên từ của word-diff phải cho khoảng trắng thành một token riêng, nếu không việc dựng lại dòng nguồn là bất khả"
    - "Cờ trạng thái (`touched`) trả lời 'phía này có dòng ở đây không'; `len()` trả lời câu khác"
    - "Fixture cho một mutation phải được KIỂM là có phân biệt được, không chỉ được chỉ định"
key-files:
  created:
    - "src-tauri/src/git/parsers/word_diff.rs"
    - "src-tauri/tests/word_diff_integration.rs"
    - "src-tauri/tests/word_diff_commands.rs"
  modified:
    - "src-tauri/src/domain/diff.rs"
    - "src-tauri/src/git/parsers/patch.rs"
    - "src-tauri/src/git/parsers/mod.rs"
    - "src-tauri/src/commands/diff.rs"
    - "src/lib/ipc.ts"
    - "src/lib/ipc.diff.test.ts"
decisions:
  - "🔴 Biên từ là `--word-diff-regex=[^[:space:]]+|[[:space:]]+`, KHÔNG phải mặc định — plan sai, xem mục Phát hiện đo được #1"
  - "Quy tắc đẩy là cờ `touched` CỘNG bộ đếm chỉ tăng cho phía được đẩy — bảng đo của plan đúng, đã kiểm chứng lại"
  - "Span là chỉ số BYTE; 03-04 chịu trách nhiệm chuyển sang UTF-16 code unit"
  - "no_newline_at_eof KHÔNG đến từ porcelain và Task 2 không ghi đè nó"
metrics:
  duration: "~95 phút"
  completed: "2026-09-22"
  tasks_completed: "2/2"
  tests_added: "41 Rust + 3 frontend"
  tests_total: "257 Rust (+1 ignored), 236 frontend"
---

# Phase 3 Plan 03: Diff mức từ Summary

Diff mức **từ** lấy từ `git diff --word-diff=porcelain`: bộ phân tích riêng dựng lại
dòng nguồn từ các đoạn từ, sinh khoảng byte vào `DiffLine.content`, và một lệnh git thứ
hai **có điều kiện** trong `get_file_diff` — cộng một phát hiện làm đổ một assertion
cứng của plan.

---

## Hình dạng JSON thật (03-04 vẽ từ đây)

```jsonc
// DiffLine — trường `spans` là phần MỚI của 03-03
{ "kind": "added",
  "content": "  x === 1",
  "oldLine": null, "newLine": 4,
  "noNewlineAtEof": false,
  "spans": [ { "start": 4, "end": 7 } ] }   // ← content[4..7] == "==="
```

`Span` có **đúng hai khoá**, đã ghim bằng test liệt kê toàn bộ khoá rồi so bằng — đúng
phép kiểm đã bắt lỗi `old_size`/`oldSize` của 03-02.

**`spans` rỗng là ca BÌNH THƯỜNG**, không phải lỗi. Bốn đường dẫn tới nó, tất cả đều
đúng: dòng `context`; tệp chỉ thêm; tệp chỉ xoá; tệp sửa quá 2000 dòng. Giao diện
**phải** vẽ được khi mảng rỗng — tô cả dòng là suy giảm đúng.

### 🔴 Cảnh báo cho 03-04: byte ≠ UTF-16 code unit

`start`/`end` là chỉ số **byte**. CodeMirror đánh chỉ số theo **UTF-16 code unit**.

| ký tự | byte | UTF-16 |
|---|---|---|
| `a` | 1 | 1 |
| `é` | 2 | 1 |
| `ỏ` | 3 | 1 |
| emoji ngoài BMP | 4 | **2** |

Trên `xéy dỏng TEST`, từ `TEST` bắt đầu ở **byte 12** nhưng ở **UTF-16 index 9**. Dùng
thẳng con số của Rust làm offset cho decoration sẽ tô lệch trên mọi dòng tiếng Việt.
Có test ở `src/lib/ipc.diff.test.ts` ghim cả phép chuyển đúng lẫn phép cắt sai, để
03-04 không phải tự phát hiện lại.

Byte được chọn làm hệ gốc vì đó là hệ mà git nói, và mọi phép chuyển ở phía Rust cũng
chỉ là một cơ hội lệch một nấc nữa — 03-04 dù sao cũng phải chuyển.

---

## 🔴 Phát hiện đo được — chỗ git thật khác điều plan khẳng định

### 1. 🔴 Plan SAI: biên từ **mặc định** làm việc dựng lại dòng nguồn trở nên bất khả

Plan chỉ thị rõ: *"Biên từ: mặc định của git (khoảng trắng), KHÔNG `--word-diff-regex=.`"*,
kèm lý do đúng về `.` (khớp theo byte, cắt ký tự UTF-8). Lý do đó đúng; **kết luận thì
sai**, vì plan không kiểm phương án thứ ba.

Với biên mặc định, khoảng trắng ngăn cách **không thuộc đoạn nào** khi hai đoạn khác
phía gặp nhau. Hai phép đo trên git 2.54.0.windows.1:

```text
(1) `alpha beta` → `alpha`          (xoá từ CUỐI)
     ␠alpha                            dựng lại phía CŨ = "alpha" + "beta"
     -beta                                              = "alphabeta"   ← MẤT dấu cách
     ~                                 unified cho      = "alpha beta"

(2) `dong hai` → `dong hai da sua`   (thêm vào CUỐI)
     ␠dong hai ␠                       dựng lại phía CŨ = "dong hai "   ← THỪA dấu cách
     +da sua                           unified cho      = "dong hai"
     ~
```

Ca (2) **nằm ngay trong `text-simple.txt`** — fixture đầu tiên, hình dạng thường gặp
nhất có thể tưởng tượng (thêm chữ vào cuối một dòng). Không có phép hậu xử lý nào lấy
lại được: thông tin "dấu cách này thuộc phía nào" **không có** trong đầu ra.

Hệ quả: bất biến quan trọng nhất của plan — *"`content` dựng ra phải bằng **từng byte**
với `content` của `parse_patch`"* — **không đạt được** với biên mặc định. Và chính bất
biến đó là thứ cho phép `Span` (chỉ số vào chuỗi của `parse_patch`) được tính trên chuỗi
của `parse_word_diff`.

**Đã sửa assertion:** dùng `--word-diff-regex=[^[:space:]]+|[[:space:]]+` — hoặc một vệt
ký tự không khoảng trắng, hoặc một vệt khoảng trắng. Mọi byte thuộc đúng một token, nên
nối các đoạn cho **đúng** dòng nguồn:

```text
(1) ␠alpha / -␠beta / ~       → cũ = "alpha beta" ✅   mới = "alpha" ✅
(2) ␠dong hai / +␠da sua / ~  → cũ = "dong hai" ✅     mới = "dong hai da sua" ✅
```

**Cảnh báo của plan về `.` vẫn được tôn trọng.** `[^[:space:]]+` khớp một **vệt** byte
không khoảng trắng, và mọi byte của một ký tự UTF-8 nhiều byte đều không phải khoảng
trắng — chúng luôn nằm trong cùng một vệt. Đo trên `xéy dỏng test` → `xéy dỏng TEST`
xác nhận `é` (2 byte) và `ỏ` (3 byte) đi qua nguyên vẹn, và ca `==` → `===` của chủ dự
án cho đầu ra **y hệt** biên mặc định.

**Cái gì bắt được lỗi này:** test tích hợp `noi_dung_dung_lai_bang_tung_byte_voi_parse_patch`,
ngay lần chạy đầu, trên `text_simple`:

```
assertion `left == right` failed: [text_simple] dòng 2 phía Removed LỆCH giữa hai bộ phân tích.
  porcelain dựng lại : "dong hai "
  unified cho        : "dong hai"
```

Không test đơn vị nào bắt được — chúng chạy trên buffer tự gõ, và tôi đã gõ đúng thứ
mình tưởng. Đúng bài học `%x1f` của 02-04 lặp lại ở một tầng khác.

### 2. Plan SAI (nhỏ): git **không** tách vùng thay đổi thành hai đoạn cùng phía liên tiếp

Plan đòi mutation "bỏ bước gộp span liền kề" phải cho ≥1 test đỏ, với lý do "git có thể
tách". Đo ba ca, cả ba đều cho thấy git **gộp sẵn**:

```text
`giu cu giu2` → `giu mot hai giu2`    -cu / +mot hai          ← MỘT đoạn, không phải hai
`aaa bbb ccc` → `AAA BBB ccc`         -aaa bbb / +AAA BBB     ← MỘT đoạn mỗi phía
cùng ca, --word-diff-regex=.          -aaa/+AAA/␠␠/-bbb/+BBB  ← vẫn xen đoạn chung
```

Nên mutation đó **không kiểm được bằng dữ liệu git thật**. Bước gộp vẫn giữ (định dạng
porcelain không hứa gì về việc gộp, và một bước tiền xử lý tương lai có thể tách), và
test của nó chạy trên đầu vào **tổng hợp** — được ghi rõ là tổng hợp trong doc comment,
không giấu đi. Chi tiết ở mục "Mutation" #4.

### 3. Ba assertion cứng của plan được kiểm chứng lại — đều ĐÚNG

| Assertion của plan | Kết quả đo |
|---|---|
| `--word-diff-porcelain` (gạch ngang) không tồn tại, thoát 129 + in usage | ✅ đúng, có test chạy git thật ghim cả hai dạng |
| `~` **vẫn in** cho dòng cuối không newline, và porcelain **không** in `\ No newline at end of file` | ✅ đúng, đo trên `no-eol.txt` |
| Bảng ba-quy-tắc × ba-fixture (cờ `touched` + bộ đếm theo phía là tổ hợp duy nhất đúng) | ✅ đúng **từng ô** — xem mục Mutation #5/6/7 |

Bảng đo của plan khớp hoàn toàn với kết quả mutation thật, gồm cả hai cảnh báo tinh tế
nhất: `one-side` cho **0 đỏ** với mutation `touched`→`len()`, và cờ đúng + bộ đếm vô
điều kiện vẫn sai `line_no`.

---

## Kiểm mutation — 12/12 đã chạy, **hai** cổng ban đầu vô dụng phải sửa

| # | Đột biến | Kết quả | Fixture / test đỏ |
|---|---|---|---|
| 1 | span dòng sửa → `[0, len)` phủ cả dòng | ✅ **8 đỏ** | `ca_chu_du_an...` + 6 unit + 1 tích hợp |
| 2 | cắt tiền tố `split_first` → `trim_start()` | ✅ **17 đỏ** | `thut_le_dau_dong_khong_bi_an_mat` |
| 3 | bỏ hai lời gọi `clear()` ở `~` | ⚠️ **0 đỏ** → là **mã chết**, đã xoá | — |
| 3b | `mem::take` → `clone` (phép reset THẬT) | ✅ **13 đỏ** | `bo_dem_duoc_xoa_o_moi_dau_ngã` |
| 4 | bỏ bước gộp khoảng liền kề | ⚠️ **không kiểm được bằng dữ liệu git thật** | git gộp sẵn — xem Phát hiện #2 |
| 5 | điều kiện đẩy → "luôn đẩy cả hai phía" | ✅ **16 đỏ** | `one_side...` (đỏ vì **số lượng**) |
| 6 | cờ `touched` → `len() > 0 \|\| có span` | ✅ **3 đỏ trên `mixed`**, **0 đỏ trên `one-side`** | `mixed_ghim_ca_line_no...` |
| 7 | tăng `line_no` ở mọi `~` (không theo phía) | ✅ **2 đỏ** | `mixed...` (đúng nội dung, **sai `line_no`**) |
| 8 | bỏ cổng chỉ-thêm/chỉ-xoá | ✅ **2 đỏ** | `tep_chi_them...`, `tep_chi_xoa...` (`CommandLog`) |
| 9 | khớp theo **nội dung** thay vì số dòng | ⚠️ **0 đỏ lần đầu** → sửa fixture → ✅ **1 đỏ** | xem "Cổng vô dụng" #2 |
| 10 | bỏ cổng an toàn `content ==` | ⚠️ **0 đỏ — không kiểm được bằng dữ liệu hiện có** | ghi rõ, không bịa ca |
| 11 | dời bước word-level xuống **sau** `put_diff` | ✅ **4 tích hợp + 1 unit đỏ** | `goi_lan_hai_dung_cache...` |
| 12 | Task 2 ghi đè `no_newline_at_eof` | ✅ **1 tích hợp + 1 unit đỏ** | `buoc_word_level_khong_ghi_de...` |

**Cổng 5 và 6 làm hai test KHÁC nhau đỏ** như plan yêu cầu: #5 đỏ ở
`one_side_khong_sinh_ban_ghi_rong_gia...` (số lượng bản ghi), #6 đỏ ở
`mixed_ghim_ca_line_no...` (dòng rỗng bị bỏ). Bộ test phân biệt được "dòng rỗng có tồn
tại" với "dòng không tồn tại ở phía này".

### Output đỏ thật (trích nguyên)

**#1** — mutation then chốt của plan, mô phỏng đúng thứ chủ dự án nói là **không** muốn:
```
panicked at src\git\parsers\word_diff.rs:512:9:
assertion `left == right` failed: khoảng phải trỏ đúng vào `===`
  left: "  if (typeof cellData ==="
 right: "==="
test result: FAILED. 14 passed; 7 failed
```

**#2**
```
panicked at src\git\parsers\word_diff.rs:982:9:
assertion `left == right` failed: một byte tiền tố bị bỏ, BẢY dấu cách thụt lề còn lại
phải nguyên vẹn. `trim_start()` ăn hết cả tám và làm mọi khoảng sau đó lệch bảy byte
  left: "sau tam dau cach +moi"
 right: "       sau tam dau cach moi"
test result: FAILED. 4 passed; 17 failed
```

**#3b** — phép reset thật nằm ở `mem::take`, không ở `clear()`:
```
panicked at src\git\parsers\word_diff.rs:998:9:
assertion `left == right` failed: ba dòng RIÊNG. Nội dung kiểu `mothai` hay `mothaiba`
nghĩa là bộ đệm không được xoá ở `~`
  left: [(1, "mot"), (2, "mothai"), (3, "mothaiba")]
 right: [(1, "mot"), (2, "hai"), (3, "ba")]
```

**#5** — đỏ vì **số lượng**, đúng như plan dự đoán (bản ghi rỗng giả ở vị trí 1 và 4):
```
panicked at src\git\parsers\word_diff.rs:618:9:
assertion `left == right` failed: phía MỚI có ĐÚNG hai dòng. Ba nghĩa là một bản ghi
rỗng giả đã được đẩy ở chỗ phía mới không có dòng nào — đó là quy tắc "luôn đẩy cả hai
phía" và nó sai. Nhận: [(1, ""), (2, "keep2"), (3, "ADDED_LINE"), (4, "")]
  left: 4
 right: 2
```

**#6** — 🔴 chạy trên **`mixed`**, và **`one-side` cho 0 đỏ** đúng như bảng đo của plan:
```
panicked at src\git\parsers\word_diff.rs:571:9:
assertion `left == right` failed: phía MỚI: bốn dòng, dòng 2 rỗng, và `ADDED` ở dòng 4.
  left: [(1, "a"), (2, "keep"), (3, "ADDED")]        ← dòng rỗng BỊ BỎ, mọi dòng sau lệch
 right: [(1, "a"), (2, ""), (3, "keep"), (4, "ADDED")]

# cùng mutation, chạy trên one-side.txt:
test result: ok. 1 passed; 0 failed        ← ĐÚNG như plan cảnh báo
```

**#7** — nội dung **đúng**, chỉ `line_no` sai. Đây chính là lý do plan bắt ghim `line_no`:
```
panicked at src\git\parsers\word_diff.rs:571:9:
  left: [(1, "a"), (2, ""), (4, "keep"), (5, "ADDED")]   ← bộ đếm tăng cho phía không có dòng
 right: [(1, "a"), (2, ""), (3, "keep"), (4, "ADDED")]
```

**#8** — bằng chứng là chuỗi argv của lệnh thừa, không phải một con số:
```
panicked at tests\word_diff_commands.rs:213:5:
KHÔNG lệnh `--word-diff` nào được chạy trên tệp chỉ THÊM ... Lệnh đã chạy: [
  "git -c core.quotepath=false diff --word-diff=porcelain --word-diff-regex=... -- text-simple.txt",
  "git -c core.quotepath=false diff --unified=3 --find-renames ... -- text-simple.txt",
  ...
]
  left: 1
 right: 0
```

**#11**
```
panicked at src\commands\diff.rs:1135:9:
bước diff mức từ phải chạy TRƯỚC khi mục được đặt vào cache. Đặt sau thì lần gọi ĐẦU
vẫn đúng và chỉ lần thứ hai mất `spans` — một lỗi chỉ hiện ra khi người dùng bấm lại
vào cùng một tệp
```

**#12**
```
panicked at tests\word_diff_commands.rs:548:5:
phải còn ít nhất một dòng mang `no_newline_at_eof` SAU khi bước word-level chạy...
Các dòng: [(Context, "mot", false), (Context, "hai", false), (Removed, "ba", false),
           (Added, "BA DA SUA", false)]
```

---

## Hai cổng vô dụng phải sửa — chi tiết, vì chúng là bài học

### Mutation #9: **fixture** mà plan chỉ định không phân biệt được hai phép khớp

Plan chỉ định `dup-lines.txt` cho mutation "khớp theo nội dung thay vì số dòng". Chạy
mutation đó: **0 test đỏ**.

Lý do nằm ở hình dạng thật của fixture, đo bằng chính `parse_word_diff`:

```text
 start / TRUNG / mid / TRUNG    →    start / TRUNG / mid / TRUNG DA SUA

OLD 2 "TRUNG" spans=[]          ← cả hai bản ghi có spans RỖNG
OLD 4 "TRUNG" spans=[]
NEW 4 "TRUNG DA SUA" spans=[Span { start: 5, end: 12 }]
```

Git coi thay đổi đó là **thêm** ` DA SUA`, nên phía cũ không có khoảng nào. Khớp theo
nội dung chọn nhầm bản ghi dòng 2 — nhưng bản ghi đó **cũng** có `spans` rỗng, nên kết
quả y hệt và không test nào phân biệt được.

Đây là một dạng "cổng tự vô hiệu hoá" mà 03-02 chưa gặp: không phải cổng sai, mà **dữ
liệu** sai. Bài học tổng quát: *fixture cho một mutation phải được **kiểm** là có phân
biệt được, không chỉ được **chỉ định**.* Plan đã áp đúng nguyên tắc này cho `one-side`
vs `mixed` (có đo, có bảng) nhưng không áp cho `dup-lines`.

**Sửa:** cần hai dòng **cùng nội dung ở phía được khớp** mà **khoảng khác nhau**. Đạt
được bằng cách cho hai dòng khác nhau ở phía cũ **hội tụ** về cùng nội dung ở phía mới,
đổi ở **hai vị trí khác nhau trong dòng**:

```text
 X b c / a b X      →      a b c / a b c

NEW 2 "a b c" spans=[0,1)    ← đổi ở ĐẦU dòng
NEW 4 "a b c" spans=[4,5)    ← đổi ở CUỐI dòng — cùng nội dung, KHÁC khoảng
```

Chạy lại mutation → đỏ, và thông điệp nói rõ hậu quả nhìn thấy được:

```
panicked at tests\word_diff_commands.rs:304:5:
dòng thêm THỨ HAI đổi ở CUỐI dòng (`a b X` → `a b c`), nên khoảng phải trỏ vào `c`.
Nhận `a` — nếu nó là `a` thì phép khớp đã chọn theo NỘI DUNG và gán khoảng của dòng
thứ nhất cho dòng này
  left: "a"
 right: "c"
```

Fixture dựng **trong test** bằng `tempfile`, **không** sửa `make-diff-fixtures.sh` —
script đó không nằm trong `files_modified` của plan này, đúng như plan dặn.

### Mutation #3: cổng nhắm vào **mã chết**

Bản đầu có hai lời gọi `clear()` sau khối `if` trong `ket_thuc_dong`, đặt vào với lý do
"đặt lại cả ba bất kể có đẩy hay không". Mutation xoá chúng cho **0 test đỏ**.

Phân tích cho thấy vì sao: `them()` là **nơi duy nhất** ghi vào `buf`, và nó **luôn**
đặt `touched = true`. Nên "bộ đệm không rỗng" kéo theo "`touched` đúng", tức nhánh
`else` luôn có bộ đệm đã rỗng sẵn. Hai lời gọi đó **không bao giờ chạy**.

**Sửa:** xoá chúng. Giữ lại thì có hại chứ không vô hại — chúng làm người đọc tin rằng
có một đường đi cần dọn, và làm một mutation tương lai xoá chúng trông như "cổng hỏng"
trong khi thật ra là "mã chết". Bất biến thật do `mem::take` giữ, và mutation đổi
`mem::take` → `clone` cho **13 test đỏ**.

Cùng họ với `assert!(MAX_CACHED_DIFFS > MAX_CACHED_HISTORIES)` mà clippy bắt ở 03-02:
một phép khẳng định không quan sát được gì lúc chạy.

### Mutation #10: không kiểm được bằng dữ liệu hiện có — **nói thật, không bịa ca**

Cổng an toàn `word_line.content == diff_line.content` (bước 5 của Task 2): mutation bỏ
nó cho **0 test đỏ**, và đó là kết quả **đúng đắn** chứ không phải cổng hỏng.

Lý do: test tích hợp `noi_dung_dung_lai_bang_tung_byte_voi_parse_patch` khẳng định hai
bộ phân tích đồng ý **từng byte** trên bảy hình dạng tệp. Nếu chúng luôn đồng ý thì
không dữ liệu nào làm cổng an toàn kích hoạt. Hai khẳng định đó **mâu thuẫn theo thiết
kế** — một cái nói "không bao giờ lệch", cái kia bắt "lệch thì phải bị chặn".

Cổng vẫn giữ, vì hậu quả của việc sai là **panic ở tầng giao diện** (`&content[a..b]`
với chỉ số vượt biên), và một repo thật có thể mang hình dạng mà fixture chưa có. Nó là
mã phòng thủ cho ca chưa đo được, không phải mã chết theo nghĩa của #3: #3 là mã mà
**cấu trúc chương trình** chứng minh không chạy được; #10 là mã mà **dữ liệu hiện có**
chưa kích hoạt.

---

## Chi phí thật của lệnh git thứ hai

Đọc từ `CommandLog`, profile **`--release`** (bài học 02-07: số debug vô nghĩa):

| Commit / tệp | lệnh **word-diff** | lệnh diff **chính** | tổng lệnh |
|---|---|---|---|
| `emptyline_mod` / `empty-line.txt` | **36 ms** | 35 ms | 7 |
| `mixed_mod` / `mixed.txt` | **32 ms** | 32 ms | 7 |
| `text_simple` / `text-simple.txt` | **39 ms** | 32 ms | 7 |

**Kết luận: lệnh word-diff tốn xấp xỉ ĐÚNG BẰNG lệnh diff chính** (32–39ms so với
32–35ms). Con số này bị chi phối bởi **chi phí sinh tiến trình**, không phải bởi khối
lượng phân tích — mọi lệnh trong bảng, kể cả `git rev-parse` (31–38ms) và
`git cat-file --batch-check` (31–34ms), đều rơi vào cùng dải 30–40ms trên Windows.

Ba hệ quả cho việc quyết định có tối ưu thêm hay không:

1. **Ba cổng bỏ qua là phần tiết kiệm thật.** Tệp chỉ thêm / chỉ xoá / vượt 2000 dòng
   không trả 30–40ms nào. Trên một commit toàn tệp mới, đó là toàn bộ chi phí.
2. **Cache khoá `(repo_id, sha, path)` xoá chi phí cho lần mở thứ hai.** Có test khẳng
   định lần gọi thứ hai thêm **đúng 0** entry vào nhật ký.
3. **Nếu về sau cần nhanh hơn, hướng đúng là giảm SỐ LỆNH, không phải tối ưu phân
   tích.** Bảy lệnh × ~33ms ≈ 230ms cho một lần mở diff nguội trên Windows. Đây là số
   đo trên repo mẫu nhỏ; chưa đo trên repo thật lớn, và **chưa** thuộc phạm vi plan này.

⚠️ Đây là số trên **repo mẫu** (`target/fixtures/diff-cases`), tệp vài dòng. Thời gian
mở diff của một tệp lớn thật thuộc **checkpoint #3** (03-01) và **vẫn chưa có số**.
Plan này không đo lại và không chép số từ đâu.

---

## Quy tắc đẩy ở `~` — bảng đo đã chép vào doc comment

Yêu cầu số 11 của plan: bảng ba-quy-tắc × ba-fixture nằm trong doc comment của
`word_diff.rs`, để một mutation không đỏ trong tương lai được chẩn đoán là **fixture
sai** chứ không phải **mã đúng**. Đã chép nguyên, kèm ba hệ quả đọc được và một dòng
chỉ thẳng: *"Nếu nó cho 0 test đỏ thì kiểm test đang chạy trên tệp nào **trước khi**
kết luận mã đúng."*

Câu hỏi mà cờ `touched` trả lời — và `len()` không trả lời được — là **"phía này có
dòng ở vị trí này không"**, không phải "bộ đệm có byte không". Một dòng rỗng *có tồn
tại* nhưng *không có byte*; một phía bị xoá dòng *không tồn tại* ở vị trí đó.

---

## Cổng verification

| Cổng | Kết quả |
|---|---|
| `cargo test` | ✅ **257 passed**, 0 failed, 1 ignored (mốc 216) |
| `cargo clippy --all-targets` | ✅ 0 warning, 0 error |
| `npm run typecheck` | ✅ sạch |
| `npm test` | ✅ **236 passed**, 24 tệp (mốc 233) |
| 12 mutation của bảng `<verification>` | ✅ tất cả đã chạy; 9 đỏ, 3 ghi rõ là không kiểm được |

Không hồi quy: 216 → 257 Rust (+41), 233 → 236 frontend (+3).

**Cổng grep KHÔNG dùng** (plan nêu rõ, đã tôn trọng):
- `grep -c 'word-diff=porcelain'` trên `commands/diff.rs` — chuỗi này xuất hiện trong
  doc comment giải thích chính tả đúng. Thay bằng test đọc **chuỗi argv thật** từ
  `CommandLog` (`chuoi_tham_so_that_dung_chinh_ta_dau_bang`), cộng một test chạy git
  thật cả hai dạng cờ và khẳng định dạng gạch ngang thoát khác 0 + in usage.
- Mọi cổng grep trên `word_diff.rs` — tệp có doc comment dày dán nguyên đầu ra porcelain
  làm ví dụ, gồm cả những chuỗi mà cổng muốn tìm.

Cổng đọc mã nguồn có dùng, nhưng theo bài học 03-02: neo vào **quyết định** (vị trí
`gan_khoang_muc_tu` so với `ket_thuc_voi_ten_cu`), cắt theo **từng lệnh**
(`GitCommand::new`), và mỗi cổng có một khẳng định **tiền đề** để phép lọc ăn mất mã
không làm cổng luôn xanh.

Một cổng của 03-02 đã **đỏ đúng lúc**: `moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path`
bắt được lệnh git mới ngay khi tôi thêm nó (`left: 4, right: 3`). Cổng đó hoạt động
đúng như 03-02 thiết kế; đã cập nhật hai con số (3→4 mốc, 2→3 lệnh) kèm lý do.

---

## Deviations from Plan

### 1. [Rule 1 - Bug] Biên từ mặc định làm việc dựng lại dòng nguồn sai

Xem "Phát hiện đo được" #1. Assertion cứng của plan sai; đã sửa sang
`--word-diff-regex=[^[:space:]]+|[[:space:]]+`, ghi cả phép đo lẫn lý do vào doc
comment, và đặt biên từ thành hằng `WORD_DIFF_REGEX` dùng chung để hai bên không lệch.
**Commit:** `4a7bc64`

### 2. [Rule 1 - Bug] Fixture của mutation #9 không phân biệt được hai phép khớp

Xem "Cổng vô dụng" #9. `dup-lines.txt` cho 0 test đỏ; đã dựng fixture đúng hình dạng
trong test bằng `tempfile`, không sửa script fixture của 03-02. **Commit:** `54132f4`

### 3. [Rule 1 - Bug] Hai lời gọi `clear()` là mã chết

Xem "Cổng vô dụng" #3. Đã xoá, ghi lý do đầy đủ vào chỗ chúng từng đứng.
**Commit:** `4a7bc64`

### 4. Mutation #4 không kiểm được bằng dữ liệu git thật

Xem "Phát hiện đo được" #2. Git gộp sẵn các đoạn cùng phía. Bước gộp vẫn giữ và vẫn có
test, nhưng đầu vào là **tổng hợp** và được ghi rõ là tổng hợp.

### 5. Thêm test ngoài plan

Plan đòi "≥14 test parser, ≥7 test tích hợp, ≥21 test mới". Thực tế: 21 test đơn vị
parser + 2 test hợp đồng `Span`, 3 test tích hợp parser, 13 test tích hợp command,
2 cổng đọc mã nguồn, 3 test frontend. Phần vượt chủ yếu là cổng phải thêm sau khi
mutation cho 0 test đỏ, cộng test ghim phép chuyển byte→UTF-16 cho 03-04.

---

## Known Stubs

Không có. `spans` được sinh từ dữ liệu git thật và có test tích hợp chạy git thật trên
bảy hình dạng tệp.

Một điểm **chưa kiểm chứng**, ghi rõ để không ai nhầm: plan này không sửa tệp `.tsx`
nào, nên **không có gì để nhìn bằng mắt**. Khoảng có **trông đúng chỗ** trên màn hình
hay không thuộc 03-04, và ở đó phải nhìn bằng mắt — `spans` đúng về chỉ số vẫn có thể
vẽ sai nếu decoration đặt sai hệ toạ độ (đúng lớp lỗi "sai hệ quy chiếu toạ độ canvas"
của Phase 2 mà không test tự động nào bắt được). Phép chuyển byte→UTF-16 là chỗ nguy
hiểm nhất, và nó đã có test ghim ở phía TS.

---

## Threat Flags

Không có bề mặt an ninh mới ngoài `<threat_model>` của plan. Bảy threat từ T-03-17 đến
T-03-23 đều có đường mã và ít nhất một test:

| Threat | Cổng |
|---|---|
| T-03-17 (span vượt biên → panic) | cổng `content ==` trước khi gán; test tích hợp khẳng định `s.end <= content.len()` trên mọi span |
| T-03-18 (span cắt giữa ký tự nhiều byte) | biên từ khớp theo **vệt**, không theo byte; test `is_char_boundary` hai đầu trên nội dung tiếng Việt |
| T-03-19 (lệnh git thứ hai mỗi lần mở diff) | ba cổng bỏ qua + cache dùng chung mục; bốn test đọc `CommandLog` |
| T-03-20 (`spans` phình payload) | `WORD_DIFF_MAX_CHANGED_LINES = 2000`, có test đọc giá trị hằng |
| T-03-21 (nội dung dòng vào log) | `warn!` ghi `repo.id` và **số dòng**, không ghi `content` |
| T-03-22 (cảnh báo CRLF thành đoạn từ) | chỉ `out.stdout` vào `parse_word_diff`; đã quan sát cảnh báo thật trên máy này |
| T-03-23 (word-level sai âm thầm) | `skipped > 0` → bỏ **toàn bộ** spans + `warn!`; test đầu vào không-phải-porcelain |

Ghi nhận một điểm **giảm** rủi ro ngoài dự kiến: vì `\ No newline at end of file` được
bỏ qua như header chứ **không** tính vào `skipped`, một phiên bản git in dòng đó sẽ
không làm tắt word-level cho cả tệp. Plan yêu cầu đúng điều này và có test riêng.

---

## Ghi chú cho wave 4 (03-04, giao diện)

1. **🔴 `Span` là chỉ số BYTE. CodeMirror dùng UTF-16 code unit.** Phải chuyển hệ trước
   khi đặt decoration. Test `ipc.diff.test.ts` ghim cả phép chuyển đúng lẫn phép cắt
   sai; đọc nó trước khi viết tầng vẽ. Đây là chỗ nguy hiểm nhất của bàn giao này.

2. **`spans` rỗng là ca bình thường, không phải lỗi.** Bốn đường dẫn hợp lệ tới nó
   (dòng ngữ cảnh, tệp chỉ thêm, tệp chỉ xoá, vượt 2000 dòng sửa). Giao diện phải vẽ
   được — tô cả dòng là suy giảm đúng, không phải trạng thái lỗi cần báo.

3. **Plan này KHÔNG phụ thuộc quyết định A/B của checkpoint #3, nhưng 03-04 thì có.**
   `spans` là dữ liệu thuần, dùng được cho cả `@codemirror/merge` lẫn tự vẽ hunk.

4. **`no_newline_at_eof` vẫn do `parse_patch` giữ** và bước word-level không chạm vào
   nó. Có hai cổng (một tích hợp, một đọc mã nguồn) chặn việc ghi đè.

5. **Bài học cổng mới, dùng ngay được:** *fixture cho một mutation phải được **kiểm** là
   có phân biệt được, không chỉ được **chỉ định**.* Mutation #9 của plan này xanh lần
   đầu không vì cổng sai mà vì **dữ liệu** sai — một dạng "cổng tự vô hiệu hoá" chưa
   gặp ở Phase 2 hay 03-02. Khi một mutation cho 0 đỏ, kiểm **fixture** trước khi kết
   luận mã đúng.

6. **Chi phí: mỗi lệnh git ≈ 30–40ms trên Windows, bị chi phối bởi chi phí sinh tiến
   trình.** Một lần mở diff nguội chạy **bảy** lệnh. Nếu 03-04 thấy chậm, hướng đúng là
   giảm **số lệnh**, không phải tối ưu phân tích.

---

## Self-Check: PASSED

Tệp đã kiểm tồn tại, số dòng đọc bằng `wc -l` chứ không ước lượng:

| Tệp | Dòng | Mốc plan |
|---|---|---|
| `src-tauri/src/git/parsers/word_diff.rs` | 1198 | ≥180 ✅ |
| `src-tauri/src/domain/diff.rs` | 377 | — (có `pub struct Span` ✅) |
| `src-tauri/src/commands/diff.rs` | 1212 | — |
| `src-tauri/tests/word_diff_integration.rs` | 349 | — |
| `src-tauri/tests/word_diff_commands.rs` | 648 | — |
| `src/lib/ipc.ts` | 378 | — |

Commit đã kiểm có trong `git log`, chuỗi RED→GREEN đầy đủ cho **cả hai** task:

```
54132f4 test(03-03): add a line-matching gate that can actually fail
51e4c26 feat(03-03): attach word-level spans in get_file_diff          ← GREEN task 2
718191f test(03-03): add failing tests for word-level step             ← RED task 2
4a7bc64 feat(03-03): implement word-diff porcelain parser              ← GREEN task 1
19aba52 test(03-03): add failing tests for word-level diff parser      ← RED task 1
```
