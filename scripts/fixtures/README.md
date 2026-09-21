# Repo mẫu cho thuật toán lane

Bộ repo mẫu này là **điều kiện bắt buộc trước khi viết thuật toán lane** (ROADMAP Phase 2).
Không có nó, thuật toán lane chỉ được kiểm trên lịch sử tuyến tính đơn giản — trong khi cả
bảy hình dạng dưới đây **đều có thật trong repo thật**.

Phục vụ HIST-03 (đồ thị nhánh nhiều lane) và HIST-11 (dữ liệu không phải UTF-8).

## Cách chạy

```bash
# Bộ chín repo mẫu — nhanh, vài giây
bash scripts/fixtures/make-fixtures.sh
# hoặc
npm run fixtures

# Thư mục đích khác (phải nằm dưới gốc dự án)
bash scripts/fixtures/make-fixtures.sh target/fixtures-thu

# Repo đo hiệu năng — CHẬM, xem cảnh báo ở cuối tệp
bash scripts/fixtures/make-perf-repo.sh
```

Mặc định sinh vào `target/fixtures/`. Thư mục này **không vào git** (đã có trong
`.gitignore`) — dựng lại được bằng script nên không có lý do gì commit nó.

## Chỉ cho test Rust thấy thư mục khác

Test Rust tìm fixture qua `git_plum_lib::testing::fixture_root()`, đọc biến môi trường
`GIT_PLUM_FIXTURES`; không có biến thì dùng `<src-tauri>/../target/fixtures`.

```bash
GIT_PLUM_FIXTURES=/duong/dan/khac cargo test
```

Thiếu fixture thì test **bỏ qua một cách ồn ào** — in ra tên fixture thiếu kèm lệnh cần
chạy, rồi `return`. Không panic: người mới clone repo về chạy `cargo test` phải thấy xanh
kèm lời nhắc, không thấy một bức tường đỏ.

## Chín repo mẫu

| Tên thư mục | Hình dạng | Kiểm điều gì |
|---|---|---|
| `linear` | ~20 commit một đường thẳng | Ca cơ sở, snapshot phải bất biến |
| `octopus` | một merge **bốn** cha | Cài đặt chỉ đọc `parents[1]` sẽ bỏ sót |
| `unrelated` | hai gốc rời, hợp bằng `--allow-unrelated-histories` | Số lane tăng đột biến |
| `orphan` | một nhánh `checkout --orphan` | Không cha chung, dễ bị coi là gốc giữa lịch sử |
| `wide` | 24 nhánh sống đồng thời rồi merge dần | Giới hạn lane hiển thị, cách vẽ suy giảm |
| `non-utf8` | tên tệp byte Latin-1 + thông điệp emoji | HIST-11, phân tích byte |
| `shallow` | bản sao nông `--depth=3` của `linear` | Cha trỏ tới thứ không có trong tập dữ liệu |
| `detached` | HEAD tách rời tại commit giữa | Không nhãn nhánh nào neo vào |
| `submodule` | một commit có submodule | Nghiên cứu ghi "chưa ai kiểm" — phải có repo để kiểm |

### Số thật đo được trên máy phát triển

Đo bằng lệnh git, không bằng mắt. Các plan sau dùng trực tiếp những con số này.

| Repo | Commit | Nhánh | Số thật đáng chú ý |
|---|---|---|---|
| `linear` | 20 | 1 | không merge, không rẽ |
| `octopus` | 7 | 5 | HEAD có **4 cha** |
| `unrelated` | 7 | 2 | **2 gốc** rời, merge có 2 cha |
| `orphan` | 8 | 2 | **2 gốc**, không merge nào nối chúng |
| `wide` | 73 | 25 | **25 lane đồng thời** tại commit gốc, **24 merge** `--no-ff` |
| `non-utf8` | 4 | 1 | 2 đường dẫn byte thô, 1 thông điệp có `\xff`, 1 có emoji |
| `shallow` | 3 / 20 | 1 | biên nông khai báo `parent` **không có trong tập dữ liệu** |
| `detached` | 20 | 1 | `symbolic-ref HEAD` rỗng |
| `submodule` | 2 | 1 | một gitlink trỏ tới `linear` |

`wide` là repo để quyết câu hỏi còn ngỏ trong CONTEXT.md — "số lane tối đa hiển thị trước
khi chuyển sang cách vẽ suy giảm". 25 lane đồng thời là dữ liệu thật để thử.

`shallow` là repo quan trọng nhất cho tính bền vững: commit biên của nó khai báo một dòng
`parent <sha>` mà `<sha>` **không tồn tại** trong repo. Thuật toán lane phải kết thúc lane
đó gọn gàng, không được sập và không được rò lane.

## Tính tất định — vì sao phải ghim mọi thứ

Plan 02-03 chụp ảnh (snapshot) đầu ra gán lane bằng `insta`. Nếu mã commit đổi mỗi lần chạy
script thì snapshot đỏ vô cớ và không ai tin nó nữa. Script vì vậy ghim:

* cả sáu biến `GIT_AUTHOR_NAME` / `GIT_AUTHOR_EMAIL` / `GIT_AUTHOR_DATE` /
  `GIT_COMMITTER_NAME` / `GIT_COMMITTER_EMAIL` / `GIT_COMMITTER_DATE`;
* dấu thời gian là chuỗi tăng dần cố định từ `2020-01-01T00:00:00+00:00`, mỗi commit +60 giây;
* `git init --initial-branch=main` — để `init.defaultBranch` của máy không lọt vào;
* `core.autocrlf=false` mỗi repo — nếu không, Windows và Linux sinh blob khác nhau cho cùng
  nội dung và mã commit lệch theo.

Kiểm tính tất định: chạy script hai lần rồi so `git rev-parse HEAD`. Đã xác minh cả chín
repo cho mã y hệt giữa hai lần chạy.

## Những cái bẫy đã trả giá — đọc trước khi sửa script

### `git log` **lược bỏ** commit, `git rev-list` thì không

Đo trên git 2.54.0.windows.1:

```
# Merge có tree TRÙNG một cha
git rev-list --all --count        → 3
git log --all --format=%H | wc -l → 2      ← commit merge BỊ LƯỢC
git log --all --format=%P | grep -c ' ' → 0  ← cổng đếm merge trả 0 dù repo CÓ merge

# Merge có tree KHÁC cả hai cha
git rev-list --all --count        → 4
git log --all --format=%H | wc -l → 4      ← khớp
```

`git log` mặc định áp *history simplification*. `--sparse`, `--full-history`, `--boundary`,
`-m` **đều không** cứu được — đã thử cả bốn. Hệ quả:

1. **Mọi commit phải thay đổi nội dung tệp**, kể cả commit merge. Script ghi thêm một dòng
   vào `merges.log` rồi `--amend` sau mỗi merge đúng vì lý do này.
2. Đếm merge bằng `git rev-list --all --format=%P` (lọc dòng `^commit `), **không** dùng
   `git log`.

### Merge octopus: **không** dùng `--no-ff`

Đã đo: `git merge --no-ff f1 f2 f3 f4` cho **năm** cha, vì git thêm cả `main` làm một cha
nữa. Merge thường cho đúng bốn. Dòng `Fast-forwarding to: f1` mà git in ra chỉ là bước nội
bộ của chiến lược octopus, commit cuối vẫn đủ bốn cha.

Nhưng `main` **phải có một commit riêng** trước khi rẽ nhánh. Nếu `main` chỉ có commit gốc
thì nó là tổ tiên thuần của cả bốn nhánh, git tiến `main` tới `f1` và merge chỉ còn **ba** cha.

### `git fast-import`: `data <n>` đếm **byte**

`n` đếm byte, không đếm ký tự. Lệch một byte thì fast-import chết theo cách vô dụng:

```
fatal: unsupported command: ommit refs/heads/main
fast-import: dumping crash report to .git/fast_import_crash_28492
```

Không nói byte nào sai, không nói dòng nào — chữ `c` của `commit` bị khối `data` trước ăn mất.
Đã trả giá đúng lỗi này: đếm tay `"nội dung a\n"` ra 14 trong khi thật ra là **13** byte
(tiếng Việt có ký tự đa byte).

Vì vậy script **không bao giờ** viết `n` bằng tay. Mọi khối `data` đi qua hai hàm
`emit_data` / `emit_data_file`, và `n` luôn do `wc -c` đo trên đúng chuỗi byte sắp ghi ra.
`${#var}` đếm **ký tự** — dùng nó là sai.

### Chỉ `git fast-import` giữ được byte thô (HIST-11)

Trên máy này (Git for Windows 2.54.0), **không** tạo được dữ liệu không-UTF-8 bằng
`git add` / `git commit`:

* `printf 'caf\351.txt'` trong shell bị chuyển thành UTF-8 (`caf\303\251.txt`) **trước khi**
  tới hệ thống tệp — nên ghi tệp rồi `git add` không cho tên tệp byte thô;
* `git commit -F <tệp chứa \377>` cũng chuyển `\377` thành `\303\277` và in
  `warning: commit message did not conform to UTF-8`.

`git fast-import` **giữ nguyên byte thô** ở cả hai chỗ: đường dẫn (dạng `"caf\351.txt"` có
dấu ngoặc kép) và thông điệp (khối `data <n>`). Đã xác minh:

```
git ls-tree -r -z --name-only HEAD | xxd
→ 6361 66e9 2e74 7874   ("caf\xe9.txt")
→ 64ee 722f 66i9 fc6c   ("d\xeer/fi\xfcle.txt")

git cat-file commit <sha> | xxd
→ 6279 7465 2074 68c3 b420 ff20   ("byte thô \xff ")
```

Nên `non-utf8` bắt buộc dựng bằng fast-import. Đây là dữ liệu đầu vào thật của HIST-11:
`String::from_utf8` sẽ thất bại trên cả bốn commit.

### `git clone --depth` bị **bỏ qua trong im lặng** với đường dẫn cục bộ

Clone theo đường dẫn cục bộ mặc định dùng hardlink và bỏ qua `--depth`, cho ra repo đầy đủ
chứ không nông — không có cảnh báo nào. `--no-local` là **bắt buộc**. Script kiểm
`shallow/.git/shallow` tồn tại và `exit 1` nếu không.

### Submodule qua giao thức file bị chặn theo mặc định

git ≥2.38 chặn submodule qua giao thức `file` (CVE-2022-39253). Cần
`-c protocol.file.allow=always`. Cờ này chỉ đặt cho **một lệnh** trong script sinh fixture
cục bộ, **tuyệt đối không** đặt trong `exec.rs` của ứng dụng — ứng dụng thật không bao giờ
nới cờ này (threat T-02-02).

Repo `submodule` thất bại thì script **không** thất bại theo: ghi cảnh báo và bỏ qua, vì tám
repo còn lại quan trọng hơn.

### `core.autocrlf` phải đặt **lúc clone**, không phải sau

`git clone` rồi `git config core.autocrlf false` sau đó làm cây làm việc lệch với index
(tệp đã checkout dạng CRLF), và `git checkout --detach` kế tiếp thất bại với
"Your local changes would be overwritten". Đặt bằng `git -c core.autocrlf=false clone`.

## Cổng an toàn khi xoá thư mục (T-02-01)

Script xoá thư mục đích để chạy lại được. Xoá đệ quy theo đối số người dùng là việc dễ gây
tai hoạ, nên script từ chối chạy nếu đường dẫn đích rỗng, là `/`, trùng gốc dự án, hoặc
không nằm **dưới** gốc dự án.

## Repo đo hiệu năng

`make-perf-repo.sh` sinh repo có hình dạng nhánh thật — 8–20 nhánh sống đồng thời, merge
`--no-ff`, rải thêm merge octopus ba cha. Một đường thẳng 100k commit **không đo được gì**
về lane, mà lane chính là thứ cần đo.

```bash
bash scripts/fixtures/make-perf-repo.sh                                  # 100k commit, mặc định
bash scripts/fixtures/make-perf-repo.sh target/fixtures-perf/smoke 2000  # bản nhỏ để thử
```

⚠️ **Không chạy trong CI mặc định.** Script mất nhiều phút và chiếm dung lượng đĩa đáng kể
(threat T-02-03). Nó tách riêng khỏi `make-fixtures.sh` đúng vì lý do đó.

<!-- PERF-NUMBERS:start -->
### Số thật của lần chạy đầy đủ 100k

Đo trên máy phát triển (Windows 11, git 2.54.0.windows.1), lệnh
`bash scripts/fixtures/make-perf-repo.sh`:

| Chỉ số | Giá trị |
|---|---|
| `git rev-list --all --count` | **100 007** |
| `git log --all --topo-order` (số dòng) | **100 007** — khớp, không commit nào bị lược |
| Commit merge (nhiều cha) | **3 182** |
| trong đó octopus (≥3 cha) | **320** |
| Số cha lớn nhất trên một commit | **3** |
| Số con lớn nhất trên một commit | **21** (21 lane sống đồng thời) |
| Kích thước `.git` trên đĩa | **32 MB** |
| Thời gian sinh (fast-import) | **24 giây** |
| Thời gian đóng gói (`repack -ad`) | **1 giây** |
| Tổng thời gian chạy script | **32 giây** |

**Mốc so sánh cho plan 02-02.** Đây là lệnh mà plan 02-02 sẽ phân tích:

```
git log --all --topo-order --format=%H%x1f%P%x1e | wc -c
→ 847ms, 8 444 123 byte
```

847ms chỉ để git sinh ra và ống qua `wc` — chưa phân tích gì. Core Value đòi đồ thị mở
trong **dưới một giây**, nên phần phân tích của Rust phải rất mỏng, và đây là lý do
CONTEXT.md chốt "đọc theo luồng, không nạp cả `Output` vào bộ nhớ".

### Về tốc độ sinh: vì sao không dùng tệp tạm trong `emit_data`

Bản đầu của `emit_data` ghi một tệp tạm rồi gọi `wc -c` và `cat` cho **mỗi** khối `data`.
Đo thật: **2 000 commit mất 328 giây**, tức 100k commit mất khoảng **4,5 giờ**. Toàn bộ thời
gian là sinh tiến trình con, không phải việc của git.

Bản hiện tại tính số byte trong bash (`${#s} + 1`) và không sinh tiến trình nào; `rand` trả
kết quả qua biến toàn cục thay vì `$(...)`; các vòng lặp dùng `for (( ))` thay vì `$(seq)`.
Kết quả: **24 giây cho 100k commit** — nhanh hơn khoảng **650 lần**.

Đánh đổi: `emit_data` chỉ nhận **ASCII thuần**, và có cổng `case $s in *[!\ -~]*)` chặn
ngay nếu lọt ký tự khác. Nội dung không-ASCII phải đi đường `wc -c` như `make-fixtures.sh`
làm cho repo `non-utf8`. Script cũng `export LC_ALL=C` để cả cổng ASCII và `${#s}` (đếm
byte) không đổi hành vi theo locale của máy.

### Luồng thử 12 commit chạy trước luồng lớn

Script luôn import một luồng nhỏ vào thư mục tạm trước khi chạy luồng thật. Điều này đã
trả công ngay trong lúc phát triển: cổng ASCII viết sai lần đầu (`*[!$' \t!-~']*` khớp cả
chuỗi ASCII thuần) và luồng thử bắt được trong **một giây** thay vì sau nhiều phút.

Tính tất định đã kiểm: chạy hai lần độc lập ở mức 100k cho `HEAD` y hệt
(`ab648b3bd1f4dbd2edb92543110ed4c3f7622431`).
<!-- PERF-NUMBERS:end -->
