---
phase: 02-history-graph
plan: 03
subsystem: graph-lane-algorithm
tags: [HIST-02, HIST-03, HIST-04, HIST-05, lanes, insta, criterion, benchmark]
requires:
  - "02-01 — chín repo mẫu ở target/fixtures/ và repo hiệu năng 100k commit"
  - "02-02 — domain::Commit, graph::{GraphRow, Edge}, parse_log, LOG_ARGS, LOG_FORMAT"
provides:
  - "graph::lanes::assign(&[Commit]) -> Vec<GraphRow> — hàm thuần, không IO"
  - "MAX_VISIBLE_LANES = 20 đã chốt bằng lập luận hiển thị"
  - "chín snapshot insta ghim hình dạng đồ thị của chín repo mẫu"
  - "benches/graph.rs — criterion cho parse_log và assign ở 1k/10k/100k"
  - "job bench trong .github/workflows/ci.yml"
  - "docs/04-phase2-degraded-graph.md — thiết kế vẽ suy giảm đã chốt"
affects:
  - "02-04 (command): assign sẵn sàng, nối vào sau parse_log"
  - "02-05 (bộ vẽ): phải cài GẬP lane ≥ 20 vào cột cuối; LANE_WIDTH=14, GRAPH_PADDING_LEFT=8 là số mà cap 20 suy ra từ đó"
  - "02-06 (giao diện): vẽ chỉ báo `+N cha nữa` từ GraphRow.truncated_parents"
  - "02-07 (checkpoint #1): so trực tiếp với số benchmark trong summary này"
tech-stack:
  added: []
  patterns:
    - "hai hàm cấp lane với hai chữ ký (u16 vs Option<u16>) — cap hiển thị KHÔNG áp cho lane của hàng"
    - "snapshot chiếu commit_id thành chỉ số hàng để snapshot độc lập với nội dung commit"
    - "assertion cứng ngoài snapshot để `cargo insta accept` không bless được hành vi sai"
    - "benchmark nhúng nguồn dữ liệu (repo-that/tong-hop) vào tên ca đo"
key-files:
  created:
    - "src-tauri/src/graph/lanes.rs (855 dòng, 17 test đơn vị)"
    - "src-tauri/tests/graph_fixtures.rs (14 test tích hợp)"
    - "src-tauri/tests/snapshots/graph_fixtures__*.snap (9 tệp)"
    - "src-tauri/benches/graph.rs"
    - "docs/04-phase2-degraded-graph.md"
  modified:
    - "src-tauri/src/graph/mod.rs — khai báo lanes, re-export assign"
    - "src-tauri/src/graph/types.rs — MAX_VISIBLE_LANES 32 → 20, đảo test cận"
    - "src-tauri/Cargo.toml — [[bench]] name=graph harness=false"
    - ".github/workflows/ci.yml — job bench riêng trên windows-latest"
decisions:
  - "MAX_VISIBLE_LANES = 20 chốt từ bề rộng hiển thị (1440×52%×40% − 8) ÷ 14, KHÔNG từ số đo fixture"
  - "allocate_row_lane (u16, không cap) tách khỏi allocate_parent_lane (Option<u16>, có cap)"
  - "fixture `shallow` KHÔNG kiểm được terminates — git ghép biên nông, %P rỗng; ca thật là phân trang"
  - "assertion 'mọi Edge.to_lane < MAX' là SAI; đúng là 'mọi cạnh RẼ NHÁNH < MAX'"
  - "job bench riêng, không thêm bước vào job rust ma trận ba nền tảng"
  - "không đặt ngưỡng thất bại tự động cho benchmark — runner CI nhiễu hàng chục phần trăm"
  - "HIST-02/03/05 giữ Pending: thuật toán có nhưng chưa hiển thị gì"
metrics:
  duration: "~65 phút"
  completed: "2026-09-21"
  tasks: 3
  commits: 3
  cargo_test: "90 đỗ (58 → 90, thêm 32)"
  npm_test: "57 đỗ (không đổi)"
  assign_100k: "56,9–59,4 ms trên repo thật 100 007 commit"
---

# Phase 2 Plan 03: Thuật toán gán lane — Summary

`assign` biến `Vec<Commit>` theo thứ tự topo thành `Vec<GraphRow>` **cùng độ dài** trong
**57ms trên 100 007 commit** — một phần tư ngân sách 240ms — và giữ đủ hàng trên cả chín
hình dạng repo thật, kể cả repo rộng hơn giới hạn hiển thị.

## Đã làm gì

### Task 1 — `assign` với chuỗi RED→GREEN thật (commit `d6e3300` → `517d539`)

17 test đơn vị viết trước và **xác nhận đỏ** (16 đỏ, 1 xanh — ca đầu vào rỗng, đúng như
nó phải thế với cài đặt trả `Vec::new()`), rồi mới cài thuật toán.

Bốn bước chép đúng từ `docs/01-research-competitors.md` mục 4.2. Hàm **thuần**: không IO,
không async, không `Mutex`, không biết gì về Tauri. Không dùng `rayon` — và doc comment
ghi rõ lý do mạnh hơn "không cần": gán lane **vốn tuần tự**, trạng thái hàng `r` là kết
quả của toàn bộ `0..r`, nên song song hoá **sai về mặt thuật toán** chứ không chỉ vô ích.

**Quyết định trung tâm của cả plan — hai hàm cấp lane, hai chữ ký:**

```rust
fn allocate_row_lane(lanes: &mut Vec<Option<String>>, id: &str) -> u16          // KHÔNG cap
fn allocate_parent_lane(lanes: &mut Vec<Option<String>>, id: &str) -> Option<u16>  // CÓ cap
```

`/usr/bin/grep -n 'parents\[1\]'` khớp **4 dòng, tất cả là văn xuôi** (3 comment + 1 chuỗi
thông báo lỗi trong test). Không có chỉ mục `parents[1]` thật nào — bước 3 là
`for parent in commit.parents.iter().skip(1)`.

### Task 2 — Snapshot insta trên chín repo mẫu (commit `6cf3206`)

Chín snapshot, mỗi hàng một dòng, **mã commit thay bằng chỉ số hàng** nên snapshot đọc
được và không vỡ khi script fixture đổi nội dung tệp mà giữ hình dạng:

```
r58 lane=20 color=6 pass=[0->0, 1->1, …, 19->19] out=[20->20] trunc=0 term=false
```

Năm assertion **cứng** nằm ngoài snapshot (T-02-10): `rows.len() == commits.len()` cộng
thứ tự hàng, `color == lane % LANE_COLORS`, giới hạn lane của cạnh rẽ nhánh, sổ sách cha
cân, và `terminates` chỉ bật khi thật sự có cha ngoài tập. `INSTA_UPDATE=no cargo test`
đỗ — snapshot **khớp** chứ không được tự ghi lại.

**Số thật đo được trên từng fixture:**

| Fixture | Commit | Hàng | Lane lớn nhất | Cha bị cắt |
|---|---|---|---|---|
| `linear` | 20 | 20 | 0 | 0 |
| `octopus` | 7 | 7 | 3 | 0 |
| `unrelated` | 7 | 7 | 1 | 0 |
| `orphan` | 8 | 8 | 0 | 0 |
| **`wide`** | **73** | **73** | **24** | **5** |
| `non-utf8` | 4 | 4 | 0 | 0 |
| `shallow` | 3 | 3 | 0 | 0 |
| `detached` | 20 | 20 | 0 | 0 |
| `submodule` | 2 | 2 | 0 | 0 |

`wide` chi tiết: **25 lane sống đồng thời, 10 hàng vượt cap 20, `truncated_parents` tổng
5**. Nghĩa là nhánh vẽ suy giảm **được kiểm trên dữ liệu git thật** — điều mà cap 32 của
wave 2 không bao giờ cho.

### Task 3 — Benchmark + tài liệu (commit `faebb20`)

`[[bench]] name = "graph" / harness = false`. Hai nhóm tách riêng vì checkpoint #1 cần
biết thời gian nằm ở đâu.

Job `bench` **riêng** trên `windows-latest`, không phải một bước trong job `rust` (vốn
chạy ma trận ba nền tảng — thêm bước trơn sẽ chạy bench ba lần cho ba số không so được
với nhau). Không đặt ngưỡng thất bại tự động; lý do ghi thành comment trong yml.

## Xác minh — đầu ra thật

```
cargo test                                → 90 đỗ (4 suite)  [nền 58, thêm 32]
cargo clippy --all-targets -- -D warnings → No issues found
cargo fmt --all --check                   → exit 0
INSTA_UPDATE=no cargo test graph          → 23 đỗ, không ghi lại snapshot
cargo bench --bench graph                 → 6 ca in số (2 nhóm × 3 mức)
npm run typecheck                         → exit 0
npm test                                  → 57 đỗ (5 tệp)
```

Cổng CI, kiểm đúng dạng plan yêu cầu:

```
/usr/bin/grep -qE '^[[:space:]]*run:.*cargo bench' .github/workflows/ci.yml   → exit 0
# và chứng minh cổng BẮT được việc xoá bước run:
sed 's/^        run: cargo bench.*$/        run: echo XOA/' … | grep -qE …    → exit 1 ✓
```

### Benchmark trên repo thật 100 007 commit — số cho checkpoint #1

| Nhóm | 1k | 10k | 100k |
|---|---|---|---|
| `parse_log` | 674 µs | 7,55 ms | **81,9 ms** |
| `assign_lanes` | 347 µs | 4,31 ms | **56,9 ms** |

Chạy thứ hai độc lập cho 84,5ms / 59,4ms — chênh trong nhiễu.

**Ngân sách:**

```
git log      693 ms   (wave 2 đo)
parse_log     82 ms
assign        57 ms   ← plan này, ngân sách ~240 ms, dùng 24%
─────────────────────
             832 ms   < 1000 ms ✓   còn dư ~168 ms
```

`assign` **không** phải điểm nghẽn — git chiếm 83% đường nóng. Nếu checkpoint #1 trượt
thì chỗ phải sửa là bộ nạp (`Channel`/phân trang), không phải thuật toán lane. Hình dạng
dữ liệu đo được xác nhận benchmark không suy biến: 100 000 commit → 100 000 hàng, 3 182
merge, lane lớn nhất 20.

### Kiểm chứng đột biến — đã thực sự chạy

| Đột biến | Kết quả |
|---|---|
| `.iter().skip(1)` → `.iter().take(2).skip(1)` | **5 test đỏ.** Octopus: *"merge BỐN cha phải cho BỐN cạnh ra… left: 2, right: 4"* |
| Vô hiệu `HashSet` kiểm cha tồn tại (`if known.contains` → `if true` / `if !known.contains` → `if false`) | **3 test đỏ.** *"cha 'deadbeef' không có trong tập đã nạp → terminates phải true"* |
| **`allocate_row_lane` → `allocate_parent_lane` + `continue`** (ngoài yêu cầu plan) | **ĐÚNG MỘT test đỏ:** *"mất hoặc thừa hàng: 80 commit vào, 60 hàng ra"* |

Đột biến thứ ba là lý do cả plan này được viết lại, và kết quả xác nhận mối lo là có
thật: cài đặt gộp hai hàm làm một **đánh rơi 20 trong 80 hàng**, và **16 trong 17 test
vẫn xanh**. Chỉ `gioi_han_lane_khong_ap_cho_hang_khong_bo_hang_nao` bắt được. Không có
test đó thì lỗi này ship ra, xanh toàn bộ, và chỉ lộ trên repo thật của người dùng.

## Chệch khỏi plan

### `[Rule 1 - Bug]` Assertion "mọi `Edge.to_lane < MAX_VISIBLE_LANES`" là SAI

**Phát hiện ở:** Task 2, đỏ thật trên fixture `wide` hàng 58 rồi hàng 60.

Plan yêu cầu assertion cứng *"Mọi fixture: mọi `Edge.to_lane < MAX_VISIBLE_LANES`"*. Chạy
trên dữ liệu thật thì nó đỏ — và **assertion là thứ sai, không phải `assign`**:

```
r57 lane=0  trunc=1              ← lane 0..19 đã đầy, cắt cha thứ hai (ĐÚNG)
r58 lane=20 out=[20->20]         ← đầu nhánh MỚI; allocate_row_lane không cap (ĐÚNG)
r60 lane=0  pass=[…, 20->20]     ← lane 20 đi xuyên qua (HỆ QUẢ tất yếu)
```

Cap **không** áp cho lane của hàng — đó là cả điểm của `allocate_row_lane`, và r58 phải
có hàng (HIST-04). Một khi lane 20 sống, nó tất yếu đi xuyên qua các hàng sau ở bước 4.
Kẹp nó lại thì hoặc mất hàng, hoặc vẽ hai nhánh chồng lên nhau.

**Đã sửa thành:** mọi cạnh **rẽ nhánh** (`to_lane != r.lane`) phải `< MAX_VISIBLE_LANES`.
Đó là điều `allocate_parent_lane` thật sự bảo đảm. Sửa ở cả `tests/graph_fixtures.rs` và
test đơn vị tương ứng trong `lanes.rs`.

### `[Rule 1 - Bug]` Fixture `shallow` KHÔNG kiểm được `terminates` — git ghép biên nông

**Phát hiện ở:** Task 2, assertion *"`shallow`: tồn tại ít nhất một hàng
`terminates == true`"* đỏ.

Plan, CONTEXT.md và summary 02-02 đều nói commit biên của bản sao nông *khai báo* một cha
vắng mặt (`7dae333f…`). Đúng với **đối tượng commit trên đĩa**, sai với **thứ `parse_log`
nhận được**:

```
$ git cat-file -p 6b5521ba…          $ git log --all --topo-order --format='%H|%P'
tree 82eb4fab…                       6b5521ba…|            ← RỖNG
parent 7dae333f…   ← có
```

git **ghép** (graft) biên bản sao nông và trả `%P` rỗng. Từ dữ liệu `assign` nhận được,
biên nông **không phân biệt được** với gốc thật, nên `terminates == false` là câu trả lời
**đúng** — `assign` không được bịa ra một cha mà git đã cố tình che. Muốn vẽ dấu "còn
tiếp" cho bản sao nông thì phải đọc `.git/shallow`, là việc của tầng repository.

**Đã làm:** đổi test `shallow` thành khẳng định điều đúng (git ghép biên → đúng một
commit không cha, mọi hàng `terminates == false`), và thêm test
`phan_trang_sinh_terminates` kiểm đường `terminates` ở **ca thật sự xảy ra trong sản
phẩm**: phân trang `--max-count=3` trên `linear`, nơi hàng cuối trang khai báo một cha
chưa nạp. Đó cũng là ca HIST-01 sẽ gặp mỗi lần cuộn.

### `[Rule 1 - Bug]` Benchmark im lặng dùng dữ liệu tổng hợp dù repo thật có sẵn

`make-perf-repo.sh` ghi ra **thư mục con** `target/fixtures-perf/perf-100k/`, không biến
`fixtures-perf` thành repo. Bản đầu của `benches/graph.rs` chỉ kiểm `fixtures-perf/.git`
nên rơi về dữ liệu tổng hợp trong khi repo 100k đang nằm ngay đó — và số báo cho
checkpoint #1 sẽ là số tổng hợp mà không ai biết. Chỉ lộ ra vì benchmark nhúng nguồn dữ
liệu vào tên ca đo (`repo-that` / `tong-hop`). Đã thử cả hai đường dẫn.

### `[Rule 2 - Missing critical]` Đảo test cận của `MAX_VISIBLE_LANES`

`types.rs` có sẵn test `gioi_han_lane_du_lon_cho_ca_thuong` khẳng định
`MAX_VISIBLE_LANES > 25` — **chính là lập luận vòng tròn mà plan cấm**, đóng băng thành
test. Đã đảo thành `MAX_VISIBLE_LANES < 21` (để `wide` và repo hiệu năng chạm được cap)
cộng một cận dưới mềm `>= 8` (để merge octopus thường ngày không bị cắt).

### `[Rule 2 - Missing critical]` Hai assertion cứng ngoài danh sách plan

Plan liệt kê sáu assertion cứng. Đã thêm: **sổ sách cha cân** (`out_edges.len() +
truncated_parents == số cha trong tập`) ở mọi hàng của mọi fixture, và **`terminates`
chỉ bật khi thật sự có cha ngoài tập** (kiểm hai chiều, `assert_eq!` chứ không phải
`assert!`). Cái thứ hai bắt được cả trường hợp bật thiếu lẫn bật thừa; chỉ kiểm một
chiều thì một cài đặt bật `terminates` cho mọi hàng vẫn đỗ.

### `[Rule 3 - Blocking]` clippy `needless_range_loop` và `manual_find`

Bước 2 viết `for i in 0..lanes.len()` — clippy từ chối với `-D warnings`. Đổi sang
`iter_mut().enumerate()`, gọn hơn và bỏ được một lần chỉ mục. Tương tự `manual_find`
trong `benches/graph.rs`.

## Plan sai ở đâu

**Cổng `grep -n 'parents\[1\]'` không thể khớp rỗng.** `<done>` của Task 1 đòi
`/usr/bin/grep -n 'parents\[1\]' src-tauri/src/graph/lanes.rs` **không khớp gì**. Nhưng
chính plan đó yêu cầu viết doc comment giải thích *"không bao giờ `parents[1]`"*, và ca
kiểm octopus chỉ có ích khi thông báo lỗi của nó nói rõ cài đặt sai trông như thế nào —
tức cũng nhắc đúng chuỗi ấy. Cổng vì vậy khớp **4 dòng văn xuôi** (3 comment + 1 chuỗi
thông báo lỗi) dù mã hoàn toàn sạch. Đây **cùng hạng lỗi với cổng `grep -v '^#'` mà chính
plan này đã sửa cho CI**: đếm một chuỗi xuất hiện ở đâu đó trong tệp, thay vì kiểm ngữ
nghĩa.

Lọc comment bằng `grep -nE '^[^/]*parents\[1\]'` **vẫn chưa đủ** — nó vẫn khớp dòng 502
vì chuỗi đó nằm trong một literal thụt lề, không phải sau `//`. Cổng thật sự kiểm được
điều muốn kiểm là bắt chính **phép chỉ mục**:

```
/usr/bin/grep -nE '\.parents\[[0-9]' src-tauri/src/graph/lanes.rs   → không khớp ✓
```

Đã chạy: **không khớp**. Bước 3 là `for parent in commit.parents.iter().skip(1)`, và
kiểm chứng đột biến ở trên chứng minh điều đó mạnh hơn mọi phép `grep`.

**Ba tài liệu cùng sai về fixture `shallow`.** Xem mục chệch thứ hai. `terminates` vẫn là
cơ chế đúng và vẫn được kiểm, nhưng ở ca phân trang chứ không phải ca bản sao nông.

**"Số lane lớn nhất của `wide` là 24" — plan đoán đúng.** Mô phỏng trong execution context
nói 25 lane đồng thời / lane lớn nhất 24; đo thật cho đúng cả hai.

## Cổng an toàn đã cài (threat model)

- **T-02-08 (DoS)** — vòng lặp đi đúng một lượt qua `commits`, không đi ngược, nên dữ
  liệu có vòng không treo được ứng dụng. `MAX_VISIBLE_LANES` chặn trên số lane cấp cho
  cha. Test merge 15 cha khẳng định sổ sách cân mà không panic.
- **T-02-09 (DoS)** — `HashSet` mã commit dựng **một lần** trước vòng lặp; cha ngoài tập
  bị xử lý như nút kết thúc, không cấp lane. Có test riêng cho cả ca bản sao nông (git
  ghép) lẫn ca phân trang (cha thật sự vắng).
- **T-02-10 (Tampering)** — năm assertion **cứng** nằm ngoài snapshot; `cargo insta
  accept` không bless được chúng. CI chạy `INSTA_UPDATE=no`.

## Known Stubs

Không có stub. `assign` đầy đủ chức năng và được kiểm trên dữ liệu git thật.

Hai việc **cố ý** để lại cho plan sau, đã ghi trong
`docs/04-phase2-degraded-graph.md` mục 5: gập lane ≥ 20 vào cột cuối (02-05) và vẽ chỉ
báo `+N cha nữa` (02-06). Đó là phần **hiển thị** của cơ chế mà plan này đã cài phần
**dữ liệu** — `truncated_parents` và lane thật đều đã có trong payload.

## Ghi chú cho plan sau

**Cho 02-04 (command):** `assign` là hàm thuần, gọi ngay sau `parse_log`. Chi phí 57ms ở
100k commit — tính trong cùng lời gọi command được, không cần tách luồng.

**Cho 02-05 (bộ vẽ):** `LANE_WIDTH = 14` và `GRAPH_PADDING_LEFT = 8` là hai số mà cap 20
suy ra từ đó (`docs/04-phase2-degraded-graph.md` mục 2.1). Đổi chúng thì phải tính lại
cap. **Phải cài việc gập lane ≥ 20** — Rust trả lane thật tới 24 trên fixture `wide`, và
`passthrough` có thể chứa `to_lane ≥ 20`. Bộ vẽ không gập thì đường kẻ vẽ ra ngoài cột.

**Cho 02-06 (giao diện):** `truncated_parents > 0` xảy ra thật (5 lần trên `wide`) — chỉ
báo `+N cha nữa` sẽ hiện trên dữ liệu mẫu, kiểm được bằng mắt.

**Cho 02-07 (checkpoint #1):** so với bảng benchmark ở trên. Chạy lại bằng
`cd src-tauri && cargo bench --bench graph`; benchmark tự tìm
`target/fixtures-perf/perf-100k`. Nếu tên nhóm hiện `tong-hop` thì repo thật chưa sinh và
**số đó không dùng để báo cáo được**.

## Trạng thái requirement

**HIST-02, HIST-03, HIST-05 giữ `Pending`.** Cả ba mô tả hành vi người dùng **quan sát
được**: HIST-02/03 là đồ thị hiện ra đúng, HIST-05 là hiệu năng cảm nhận khi cuộn. Sau
plan này thuật toán tồn tại và được chứng minh đúng trên chín hình dạng repo thật, nhưng
**chưa có gì hiển thị** — không command, không bộ vẽ. Trường `requirements:` hiểu là
*"góp phần vào"*, theo đúng tiền lệ plan 02-01 và 02-02 đã lập.

HIST-04 (không mất dòng) được ghim **rất chặt** ở tầng dữ liệu bằng
`rows.len() == commits.len()` trên mọi test và mọi fixture, nhưng cũng giữ `Pending` cùng
lý do.

## Self-Check: PASSED

Tệp đã tạo (đều tồn tại):
- `src-tauri/src/graph/lanes.rs`
- `src-tauri/tests/graph_fixtures.rs`
- `src-tauri/tests/snapshots/graph_fixtures__{linear,octopus,unrelated,orphan,wide,non-utf8,shallow,detached,submodule}.snap` (9 tệp)
- `src-tauri/benches/graph.rs`
- `docs/04-phase2-degraded-graph.md`

Commit đã tạo (đều có trong `git log`):
- `d6e3300` test(02-03): pin lane assignment behaviour before writing the algorithm
- `517d539` feat(02-03): assign lanes with the display cap off the row-lane path
- `6cf3206` test(02-03): snapshot lane output against nine real repository shapes
- `faebb20` feat(02-03): measure both halves of the history hot path in CI
