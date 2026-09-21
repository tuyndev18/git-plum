---
phase: 02-history-graph
plan: 01
subsystem: testing-fixtures
tags: [fixtures, git, fast-import, determinism, HIST-03, HIST-11]
requires: []
provides:
  - "scripts/fixtures/make-fixtures.sh — sinh chín repo mẫu tất định"
  - "scripts/fixtures/make-perf-repo.sh — sinh repo 100k commit có nhánh thật"
  - "git_plum_lib::testing::{fixture_root, fixture_path, require_fixture, FIXTURE_NAMES}"
  - "insta 1.48.0 + criterion 0.8.2 (dev-dependencies), lru 0.18.4 (dependency)"
affects:
  - "02-02 (phân tích git log): mốc so sánh 847ms / 8,4MB đã đo"
  - "02-03 (thuật toán lane + snapshot insta): chín hình dạng repo để kiểm"
  - "02-05 (cache diff): lru đã có trong Cargo.toml"
  - "02-06 (giao diện): @tanstack/react-virtual CHƯA cài, cố ý để plan đó cài"
tech-stack:
  added:
    - "insta 1.48.0 (dev) — snapshot test đầu ra gán lane"
    - "criterion 0.8.2 (dev) — benchmark lane + phân tích log ở 100k commit"
    - "lru 0.18.4 — cache diff theo SHA commit cho plan 02-05"
  patterns:
    - "git fast-import thay git commit khi cần hàng nghìn commit"
    - "git fast-import là cách duy nhất giữ byte thô không-UTF-8"
    - "đếm commit bằng git rev-list, không bằng git log (history simplification)"
    - "luồng thử nhỏ trước luồng lớn để bắt lỗi cú pháp trong một giây"
key-files:
  created:
    - "scripts/fixtures/make-fixtures.sh (438 dòng)"
    - "scripts/fixtures/make-perf-repo.sh (407 dòng)"
    - "scripts/fixtures/README.md (267 dòng)"
    - "src-tauri/src/testing/mod.rs (188 dòng)"
  modified:
    - "src-tauri/Cargo.toml — thêm insta, criterion, lru"
    - "src-tauri/src/lib.rs — khai báo pub mod testing"
    - "package.json — thêm script npm run fixtures"
    - ".gitignore — thêm target/"
decisions:
  - "Merge octopus KHÔNG dùng --no-ff (cho năm cha, không phải bốn); main phải có commit riêng trước khi rẽ nhánh"
  - "non-utf8 dựng bằng git fast-import vì shell và git commit -F đều chuyển byte thô thành UTF-8"
  - "emit_data của perf script chỉ nhận ASCII + có cổng chặn, đổi lấy tốc độ nhanh hơn 650 lần"
  - "testing/mod.rs là pub module, không phải #[cfg(test)], vì criterion bench là target riêng"
  - "require_fixture bỏ qua ồn ào (eprintln + None), không panic"
metrics:
  duration: "~50 phút"
  completed: "2026-09-21"
  tasks: 3
  commits: 5
  cargo_test: "27 đỗ (23 → 27)"
  npm_test: "57 đỗ (không đổi)"
---

# Phase 2 Plan 01: Bộ repo mẫu và bộ sinh repo hiệu năng — Summary

Chín repo mẫu tất định cùng bộ sinh repo 100k commit có hình dạng nhánh thật, đặt nền
cho mọi plan còn lại của phase: từ đây thuật toán lane được kiểm trên các hình dạng có
thật trong repo thật, không chỉ trên lịch sử tuyến tính.

## Đã làm gì

### Task 1 — `scripts/fixtures/make-fixtures.sh` (commit `734b809`)

Chín repo mẫu, tất cả đều dựng được trên máy này. Thư mục mặc định
`target/fixtures/`, ghi đè bằng đối số thứ nhất.

| Repo | Commit | Nhánh | Số thật đo được |
|---|---|---|---|
| `linear` | 20 | 1 | đường thẳng, không merge |
| `octopus` | 7 | 5 | HEAD có **4 cha** |
| `unrelated` | 7 | 2 | **2 gốc** rời, merge 2 cha |
| `orphan` | 8 | 2 | **2 gốc**, không merge nào nối |
| `wide` | 73 | 25 | **25 lane đồng thời** tại commit gốc, 24 merge `--no-ff` |
| `non-utf8` | 4 | 1 | 2 đường dẫn byte thô, 1 thông điệp có `\xff`, 1 có emoji |
| `shallow` | 3/20 | 1 | biên nông khai báo `parent` **không có trong tập dữ liệu** |
| `detached` | 20 | 1 | `symbolic-ref HEAD` rỗng |
| `submodule` | 2 | 1 | gitlink trỏ tới `linear` |

**Không repo nào phải bỏ** — kể cả `submodule`, vốn được phép thiếu theo plan.

### Task 2 — `scripts/fixtures/make-perf-repo.sh` (commit `977fabc`)

Repo 100k commit, hình dạng nhánh thật: 8–20 nhánh sống đồng thời, merge `--no-ff`,
rải merge octopus ba cha.

| Chỉ số | Giá trị |
|---|---|
| `git rev-list --all --count` | **100 007** |
| `git log --all --topo-order` (dòng) | **100 007** — khớp, không commit nào bị lược |
| Commit merge | **3 182** (trong đó **320** octopus ≥3 cha) |
| Số cha lớn nhất / commit | 3 |
| Số con lớn nhất / commit | **21** (21 lane đồng thời) |
| Kích thước `.git` | **32 MB** |
| Thời gian sinh (fast-import) | **24 giây** |
| Tổng thời gian script | **32 giây** |

**Mốc so sánh cho plan 02-02:**
`git log --all --topo-order --format=%H%x1f%P%x1e | wc -c` → **847ms, 8 444 123 byte**.
Đó là thời gian git *chỉ để sinh và ống dữ liệu*, chưa phân tích gì. Core Value đòi dưới
một giây, nên phần phân tích của Rust phải rất mỏng.

### Task 3 — `src-tauri/src/testing/mod.rs` (commit `5465a23`)

`fixture_root()` đọc `GIT_PLUM_FIXTURES`, mặc định `<src-tauri>/../target/fixtures`.
`fixture_path()` trả `None` khi thiếu. `require_fixture()` in `eprintln!` nói rõ fixture
nào thiếu kèm lệnh cần chạy rồi trả `None` — **bỏ qua ồn ào, không panic**.

Dev-dependency đúng phiên bản chốt ở STACK.md: `insta = "1.48.0"`, `criterion = "0.8.2"`;
`lru = "0.18.4"` vào `[dependencies]`. `cargo tree` xác nhận cả ba phân giải đúng.

## Xác minh

Cả tám bước trong `<verification>` của plan đều chạy và đỗ:

```
make-fixtures.sh chạy hai lần            → run1 OK, run2 OK
octopus: git cat-file -p HEAD | grep -c '^parent '  → 4
test -f target/fixtures/shallow/.git/shallow        → exit 0
make-perf-repo.sh ... 2000               → rev-list 2009 (≥1900), 64 merge, 9 octopus
cargo test                               → 27 đỗ (yêu cầu ≥25)
cargo clippy --all-targets -- -D warnings → No issues found
cargo fmt --all --check                  → exit 0
npm test                                 → 57 đỗ; npm run typecheck → exit 0; build → OK
```

**Tính tất định** kiểm theo đúng cách plan yêu cầu: dựng lại rồi so `git rev-parse HEAD`.
Cả chín repo mẫu cho mã y hệt giữa hai lần chạy; repo 100k cho `HEAD` y hệt giữa hai lần
chạy độc lập (`ab648b3bd1f4dbd2edb92543110ed4c3f7622431`).

Một probe tích hợp tạm (đã xoá, không commit) xác nhận cả chín fixture tìm được qua
`fixture_path` từ Rust, và `require_fixture` in đúng lời nhắc cho tên không tồn tại.

## Chệch khỏi plan

### `[Rule 1 - Bug]` Merge octopus `--no-ff` cho năm cha, không phải bốn

**Phát hiện ở:** Task 1. Plan viết `git merge f1 f2 f3 f4`; tôi thêm `--no-ff` khi thấy
git in `Fast-forwarding to: f1`, tưởng đó là dấu hiệu sai.

**Thực tế đo được:** `--no-ff` khiến git thêm chính `main` làm một cha nữa → **5 cha**.
Merge thường cho đúng 4. Dòng `Fast-forwarding to: f1` chỉ là bước nội bộ của chiến lược
octopus, commit cuối vẫn đủ bốn cha.

**Nhưng** `main` phải có **một commit riêng** trước khi rẽ nhánh. Nếu `main` chỉ có commit
gốc thì nó là tổ tiên thuần của cả bốn nhánh, git tiến `main` tới `f1` và merge chỉ còn
**3 cha** — đây là điều plan không lường tới. Đã thêm `commit gốc 2` vào `octopus`.

**Sửa ở:** `make-fixtures.sh`, cổng `exit 1` trong script bắt được cả hai hướng sai.

### `[Rule 1 - Bug]` `core.autocrlf` phải đặt lúc clone, không phải sau

**Phát hiện ở:** Task 1, repo `detached`. `git clone` rồi `git config core.autocrlf false`
làm cây làm việc lệch với index (tệp đã checkout dạng CRLF), và `git checkout --detach`
kế tiếp thất bại: *"Your local changes to the following files would be overwritten"*.

**Sửa:** dùng `git -c core.autocrlf=false clone` cho cả `shallow` và `detached`.

### `[Rule 2 - Missing critical]` `non-utf8` phải dựng bằng `fast-import`

**Phát hiện ở:** Task 1. Plan dự trù rằng Windows NTFS có thể từ chối byte Latin-1 và cho
phương án dự phòng là đặt byte thô vào **thông điệp** qua `git commit -F -`.

**Đo thật cho thấy cả hai đường đều không giữ được byte thô:**

| Cách | Kết quả |
|---|---|
| `printf 'caf\351.txt'` rồi `git add` | shell chuyển thành UTF-8 `caf\303\251.txt` **trước khi** tới hệ thống tệp |
| `git commit -F <tệp có \377>` | git chuyển `\377` → `\303\277`, in *"commit message did not conform to UTF-8"* |
| **`git fast-import`** | **giữ nguyên byte thô** ở cả đường dẫn và thông điệp |

Xác minh bằng `xxd`: tree chứa `6361 66e9` (`caf\xe9.txt`) và `64ee 722f 66i9 fc6c`
(`d\xeer/fi\xfcle.txt`); commit object chứa `6279 7465 2074 68c3 b420 ff20`.

**Sửa:** `non-utf8` dựng hoàn toàn bằng luồng `fast-import`. Kết quả **mạnh hơn** phương án
dự phòng của plan: có cả tên tệp byte thô (`\xe9`, `\xee`, `\xfc`) **và** thông điệp có
`\xff` đơn lẻ — `String::from_utf8` thất bại trên cả bốn commit. Đúng dữ liệu HIST-11 cần.

### `[Rule 1 - Bug]` `data <n>` đếm byte — đã trả giá đúng cái bẫy plan cảnh báo

**Phát hiện ở:** Task 1. Tôi viết `data 14` cho `"nội dung a\n"` (thật ra **13** byte —
tiếng Việt có ký tự đa byte). fast-import chết:

```
fatal: unsupported command: ommit refs/heads/main
```

Chữ `c` của `commit` bị khối `data` trước ăn mất. Ba số hardcode đều sai (14/13, 30/29, 27/25).

**Sửa:** bỏ hẳn số viết tay. Mọi khối `data` đi qua `emit_data`/`emit_data_file`, `n` luôn
do `wc -c` đo trên đúng chuỗi byte sắp ghi.

### `[Rule 3 - Blocking]` Perf script bản đầu quá chậm để dùng được

**Phát hiện ở:** Task 2. Bản đầu theo đúng chữ của plan (`wc -c` cho mỗi khối `data`):
**2 000 commit mất 328 giây** → 100k commit khoảng **4,5 giờ**. Không dùng được, mà plan
đòi chạy đầy đủ 100k ít nhất một lần.

**Nguyên nhân:** mỗi commit sinh ~3 tiến trình con (`wc`, `cat`, `$(rand)`); toàn bộ thời
gian là fork, không phải việc của git.

**Sửa:** tính byte trong bash (`${#s} + 1`), `rand` trả qua biến toàn cục thay vì `$(...)`,
`for (( ))` thay `$(seq)`, biến đếm thay `wc -w`. → **24 giây cho 100k commit**, nhanh hơn
khoảng **650 lần**.

**Đánh đổi có cổng bảo vệ:** `emit_data` chỉ nhận ASCII thuần và có
`case $s in *[!\ -~]*)` chặn ngay nếu lọt ký tự khác; `export LC_ALL=C` để cả cổng và
`${#s}` (đếm byte) không đổi theo locale. Nội dung không-ASCII vẫn đi đường `wc -c` như
`make-fixtures.sh` làm cho `non-utf8`.

**Lưu ý:** cổng ASCII viết sai lần đầu (`*[!$' \t!-~']*` khớp **cả** chuỗi ASCII thuần).
Luồng thử 12 commit mà plan yêu cầu đã bắt được trong **một giây** — đúng giá trị của nó.

### `[Rule 2 - Missing critical]` `.gitignore` bỏ `target/` chứ không chỉ `target/fixtures/`

Plan yêu cầu thêm `target/fixtures/`. Nhưng `make-perf-repo.sh` ghi vào
`target/fixtures-perf/`, nên chỉ chặn `target/fixtures/` sẽ để repo 32MB lọt vào git.
Đã thêm `target/` (`src-tauri/target/` vốn đã bị chặn riêng). Đã xác minh
`git ls-files | grep '^target/'` rỗng.

## Cổng an toàn đã cài (threat model)

- **T-02-01** — cả hai script từ chối chạy nếu đường dẫn đích rỗng, là `/`, trùng gốc dự
  án, hoặc không nằm **dưới** gốc dự án. Kiểm bằng `case` trên đường dẫn đã `cd` + `pwd`
  nên không lừa được bằng `..`.
- **T-02-02** — `protocol.file.allow=always` chỉ đặt cho **một lệnh** `submodule add`
  trong script fixture. **Không** có trong `exec.rs`. Đã xác minh bằng grep.
- **T-02-03** — `make-perf-repo.sh` tách riêng khỏi `make-fixtures.sh`, README ghi rõ dung
  lượng và cảnh báo không chạy trong CI mặc định.

### `[Rule 1 - Bug]` `rm -rf` thất bại một phần nhưng vẫn trả mã 0 (commit `65a32bd`)

**Phát hiện ở:** lần dựng lại fixture từ đầu sau khi đã commit. `rm -rf` in
*"Device or resource busy"* cho `target/fixtures/octopus` (OneDrive đang đồng bộ thư mục)
nhưng **vẫn thoát mã 0**. Script chạy tiếp trên thư mục còn sót, và repo `submodule` báo
**37 commit** kèm *"submodule add thất bại"* thay vì 2 commit thật — một kết quả trông
như lỗi mã nhưng thực ra là trạng thái thừa.

**Sửa:** `init_repo` kiểm thư mục đã xoá được thật, thử lại một lần sau khi chờ, và dừng
hẳn kèm thông báo nêu thủ phạm thường gặp (OneDrive, trình diệt virus, cửa sổ Explorer
đang mở) nếu vẫn còn. Dựng fixture đè lên thư mục xoá dở sinh ra repo có hình dạng **không
khớp tên của nó** — tệ hơn là không dựng.

**Ghi chú môi trường:** trên máy này OneDrive giữ handle thư mục trong lúc đồng bộ, nên
lần chạy ngay sau `rm -rf` có thể gặp lỗi tạm thời. Chạy lại là được. Đây là đặc tính của
máy, không phải của script.

### `[Rule 1 - Bug]` HIST-03 và HIST-11 giữ nguyên `Pending`, không đánh dấu `Complete`

Frontmatter của plan ghi `requirements: [HIST-03, HIST-11]`, và
`gsd-sdk query requirements.mark-complete` đã đánh dấu cả hai thành `Complete`. **Đã hoàn
tác.**

Lý do: HIST-03 là *"đồ thị vẽ đúng với merge nhiều hơn hai cha, nhánh mồ côi, …"* và
HIST-11 là *"tên tệp và thông điệp không UTF-8 vẫn hiển thị được"*. Plan này **chưa vẽ gì
và chưa hiển thị gì** — nó dựng bộ repo mẫu để các plan sau *kiểm chứng* hai yêu cầu đó.
Đánh dấu Complete ở đây sẽ báo tiến độ sai và làm phase mất cổng kiểm.

Hai yêu cầu này thuộc plan 02-03 (thuật toán lane) và 02-06/02-07 (giao diện). Trường
`requirements:` của plan nên hiểu là *"góp phần vào"*, không phải *"hoàn thành"*.

## Ghi chú cho plan sau

**Cho 02-02 (phân tích `git log`):** mốc là **847ms / 8 444 123 byte** ở 100k commit cho
riêng việc git sinh dữ liệu. Đếm commit phải dùng `git rev-list`, **không** dùng `git log`
— `git log` áp history simplification và lược commit có tree trùng cha (`--sparse`,
`--full-history`, `--boundary`, `-m` đều không cứu được).

**Cho 02-03 (thuật toán lane):**
- `octopus` có **4 cha** → bước gán cha phải lặp qua mọi cha sau cha đầu.
- `shallow` có commit biên khai báo `parent 7dae333f...` **không tồn tại** trong repo →
  lane phải kết thúc gọn, không sập, không rò.
- `wide` cho **25 lane đồng thời**, perf repo cho **21** → dữ liệu thật để quyết câu hỏi
  còn ngỏ "số lane tối đa hiển thị trước khi vẽ suy giảm".
- `unrelated` và `orphan` mỗi repo **2 gốc** → commit gốc xuất hiện giữa lịch sử.
- Mã commit tất định nên snapshot `insta` chụp được an toàn.

**Cho 02-06 (giao diện):** `@tanstack/react-virtual` **chưa** cài, đúng theo plan — cài ở
02-06 cùng commit với chỗ dùng.

## Self-Check: PASSED

Tệp đã tạo (đều tồn tại):
- `scripts/fixtures/make-fixtures.sh`
- `scripts/fixtures/make-perf-repo.sh`
- `scripts/fixtures/README.md`
- `src-tauri/src/testing/mod.rs`

Commit đã tạo (đều có trong `git log`):
- `734b809` feat(02-01): add nine fixture repos for lane algorithm testing
- `977fabc` feat(02-01): add 100k-commit perf repo generator with real branch shape
- `5465a23` feat(02-01): add fixture helper module and lane-testing dev-dependencies
- `65a32bd` fix(02-01): fail loudly when a fixture directory cannot be removed
