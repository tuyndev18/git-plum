---
phase: 02-history-graph
plan: 02
subsystem: domain-contracts-log-parser
tags: [HIST-01, HIST-11, parser, bytes, serde, memchr, contracts]
requires:
  - "02-01 — repo mẫu ở target/fixtures/ và git_plum_lib::testing"
provides:
  - "domain::Commit — hợp đồng dữ liệu commit, parents: Vec<String>"
  - "domain::{Ref, RefKind} — hợp đồng tham chiếu cho plan 02-04"
  - "graph::{GraphRow, Edge, LANE_COLORS, MAX_VISIBLE_LANES} — hợp đồng hình học cho plan 02-03"
  - "git::parsers::log::{parse_log, LogParseResult, LOG_FORMAT, LOG_ARGS}"
affects:
  - "02-03 (thuật toán lane): GraphRow/Edge đã khai báo, chỉ còn cài thuật toán; MAX_VISIBLE_LANES chờ chốt"
  - "02-04 (refs + command): dùng LOG_FORMAT và LOG_ARGS, KHÔNG chép lại chuỗi định dạng"
  - "02-06 (giao diện): tên khoá camelCase đã ghim bằng test, đọc bảng trường trong summary này"
tech-stack:
  added: []
  patterns:
    - "parse_log trả LogParseResult (có bộ đếm) thay vì Result — bản ghi méo không làm đổ cả trang"
    - "hàm lossy(bytes, &mut dirty) là đường giải mã DUY NHẤT, đặt cờ has_invalid_utf8"
    - "memchr_iter tách bản ghi/trường; dừng tách ở dấu thứ 9 để phần thừa nằm lại trong %b"
    - "so số commit với `git log --all --topo-order`, KHÔNG với `git rev-list --all --count`"
key-files:
  created:
    - "src-tauri/src/domain/commit.rs (183 dòng)"
    - "src-tauri/src/domain/refs.rs (131 dòng)"
    - "src-tauri/src/graph/mod.rs (10 dòng)"
    - "src-tauri/src/graph/types.rs (208 dòng)"
    - "src-tauri/src/git/parsers/log.rs (836 dòng)"
  modified:
    - "src-tauri/src/domain/mod.rs — khai báo commit, refs"
    - "src-tauri/src/git/parsers/mod.rs — khai báo log, thêm hai quy tắc chung"
    - "src-tauri/src/lib.rs — thêm pub mod graph"
decisions:
  - "parse_log không trả Result: HIST-11 cấm một bản ghi méo làm mất cả trang"
  - "Phần thừa sau dấu \\x1f thứ 9 nằm lại trong %b, không nối lại — rẻ hơn và cùng kết quả"
  - "MAX_VISIBLE_LANES = 32 TẠM THỜI, plan 02-03 chốt từ bề rộng hiển thị"
  - "LANE_COLORS = 7 (số nguyên tố) để lane kề nhau không trùng màu sớm"
  - "bstr có trong Cargo.toml nhưng parse_log KHÔNG dùng — std + memchr là đủ"
  - "HIST-01 và HIST-11 giữ Pending: parser là điều kiện cần, chưa đủ; chưa có gì hiển thị"
metrics:
  duration: "~35 phút"
  completed: "2026-09-21"
  tasks: 2
  commits: 2
  cargo_test: "58 đỗ (27 → 58, thêm 31)"
  npm_test: "57 đỗ (không đổi)"
  parse_100k: "61–65ms cho 100 007 commit / 18,9 MB"
---

# Phase 2 Plan 02: Hợp đồng dữ liệu và bộ phân tích `git log` — Summary

Bốn kiểu dữ liệu mà toàn bộ phase tiêu thụ, cộng bộ phân tích `git log` theo byte chạy
**61ms trên 100 007 commit** và không đánh rơi bản ghi nào trên cả chín repo mẫu — kể cả
repo có byte thô không hợp lệ trong cả đường dẫn lẫn thông điệp.

## Đã làm gì

### Task 1 — Hợp đồng dữ liệu (commit `df251ee`)

Bốn kiểu, tất cả `#[serde(rename_all = "camelCase")]`, tên khoá ghim bằng test.

**`domain::Commit`** — plan 02-03, 02-04, 02-06 đọc bảng này thay vì đi tìm trong mã:

| Trường Rust | Khoá JSON | Kiểu |
|---|---|---|
| `id` | `id` | `String` (40 hex) |
| `parents` | `parents` | `Vec<String>` |
| `author_name` | `authorName` | `String` |
| `author_email` | `authorEmail` | `String` |
| `author_time` | `authorTime` | `i64` (giây Unix) |
| `committer_name` | `committerName` | `String` |
| `committer_email` | `committerEmail` | `String` |
| `committer_time` | `committerTime` | `i64` (giây Unix) |
| `subject` | `subject` | `String` |
| `body` | `body` | `String` |
| `has_invalid_utf8` | `hasInvalidUtf8` | `bool` |

**`graph::GraphRow`** và **`graph::Edge`**:

| Trường Rust | Khoá JSON | Kiểu |
|---|---|---|
| `GraphRow.commit_id` | `commitId` | `String` |
| `GraphRow.lane` | `lane` | `u16` |
| `GraphRow.color` | `color` | `u8` |
| `GraphRow.passthrough` | `passthrough` | `Vec<Edge>` |
| `GraphRow.out_edges` | `outEdges` | `Vec<Edge>` |
| `GraphRow.truncated_parents` | `truncatedParents` | `u16` |
| `GraphRow.terminates` | `terminates` | `bool` |
| `Edge.from_lane` | `fromLane` | `u16` |
| `Edge.to_lane` | `toLane` | `u16` |
| `Edge.color` | `color` | `u8` |

**`domain::Ref`** cho plan 02-04: `fullName`, `shortName`, `kind`, `target`, `upstream`
(`null` khi chưa đặt), `ahead`, `behind`, `isHead`. `RefKind` serialize thành chuỗi
`"localBranch"` / `"remoteBranch"` / `"tag"` / `"other"` — enum chứ không phải cờ boolean
vì HIST-06 đòi ba nhóm riêng kèm số đếm.

**Hằng số:**
- `LANE_COLORS = 7`. Số nguyên tố có chủ ý: lane kề nhau và các bước nhảy đều đặn không
  rơi trùng màu sớm như với một số chẵn.
- `MAX_VISIBLE_LANES = 32` — **GIÁ TRỊ TẠM**, doc comment ghi rõ plan 02-03 chốt lại.

### Task 2 — `parse_log` (commit `77b3bc9`)

**Chữ ký cuối cùng:**

```rust
pub const LOG_FORMAT: &str =
    "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%b%x1e";
pub const LOG_ARGS: &[&str] = &["log", "--all", "--topo-order"];

pub fn parse_log(stdout: &[u8]) -> LogParseResult;

pub struct LogParseResult {
    pub commits: Vec<Commit>,
    pub skipped_records: usize,
    pub bad_timestamps: usize,
}
```

Mười một ca trong `<behavior>` đều có test riêng, cộng ba test tích hợp chạy git thật.

## Xác minh

```
cargo test                                → 58 đỗ (3 suite)   [nền 27, thêm 31]
cargo clippy --all-targets -- -D warnings → No issues found
cargo fmt --all --check                   → exit 0
npm run typecheck                         → exit 0
npm test                                  → 57 đỗ (5 tệp)
```

### Hiệu năng trên repo 100k commit — đo thật

```
git log       : 693 ms, 18 896 105 byte
parse_log #0 : 65 ms, 100007 commit, bỏ 0, timestamp hỏng 0
parse_log #1 : 63 ms, 100007 commit, bỏ 0, timestamp hỏng 0
parse_log #2 : 61 ms, 100007 commit, bỏ 0, timestamp hỏng 0
```

Tổng khoảng **758ms**, nằm trong ngân sách một giây của tiêu chí Core Value. Phần phân
tích chỉ chiếm **8–9%**; phần còn lại là git.

Lưu ý một khác biệt so với mốc của plan 02-01: mốc 847ms/8,4MB đo trên định dạng **hai
trường** (`%H%x1f%P`). Định dạng thật mười trường cho **18,9 MB** — hơn gấp đôi dữ liệu —
mà git lại chỉ mất **693ms**, nhanh hơn. Nguyên nhân gần như chắc chắn là bộ nhớ đệm hệ
điều hành đã ấm sau lần chạy trước, chứ không phải mười trường rẻ hơn hai trường. Con số
đáng tin ở đây là **chi phí của `parse_log`**, vốn đo lặp ba lần trên cùng buffer.

### Kiểm chứng đột biến — đã thực sự chạy

| Đột biến | Kết quả |
|---|---|
| `memchr_iter(RECORD_SEP, …)` → `memchr_iter(UNIT_SEP, …)` | **14 test đỏ.** HIST-11: *"git in 4 bản ghi, phân tích được 0 (bỏ 35)"* |
| Xoá `trim_ascii_whitespace(record)` đầu bản ghi | **3 test đỏ.** *"mã commit phải đúng 40 ký tự, nhận được `"\nbbbb…"`"* — đúng 41 ký tự như dự đoán |
| `from_utf8_lossy` → `from_utf8().unwrap()` | **3 test đỏ, panic.** `FromUtf8Error { … 255 … valid_up_to: 29 }` trên fixture thật |

Đột biến thứ ba đáng chú ý: byte lỗi nằm ở `valid_up_to: 29` **sau** phần tiếng Việt UTF-8
hợp lệ (`th\xc3\xb4ng \xc4\x91i\xe1\xbb\x87p c\xc3\xb3 byte th\xc3\xb4 \xff`). Nghĩa là
fixture `non-utf8` của plan 02-01 kiểm đúng ca khó: byte xấu **giữa** dữ liệu hợp lệ, chứ
không phải cả trường rác.

Một đột biến thứ tư (ngoài yêu cầu của plan) chạy ở Task 1: xoá
`#[serde(rename_all = "camelCase")]` khỏi `Commit` → **2 test đỏ** (`left: Null`).

### HIST-11 chứng minh bằng số

Test `moi_repo_mau_phan_tich_tron_ven` chạy trên **cả chín** repo mẫu, không chỉ
`non-utf8`, và mỗi repo so số commit phân tích được với số dòng
`git log --all --topo-order --format=%H` của chính repo đó. Tất cả khớp, `skipped_records`
bằng 0 ở mọi repo — bao gồm `shallow` (cha trỏ ra ngoài tập dữ liệu), `detached`
(`symbolic-ref HEAD` rỗng) và `submodule` (gitlink).

Test `non_utf8_khong_mat_dong_nao` khẳng định thêm một điều mà chỉ đếm không bắt được:
**ít nhất một commit có `has_invalid_utf8 == true`**. Thiếu khẳng định này thì test vẫn
đỗ trên một repo ASCII thuần và HIST-11 coi như không được kiểm.

## Chệch khỏi plan

### `[Rule 1 - Bug]` Phần thừa sau `\x1f` thứ 9 **nằm lại** trong `%b`, không "nối lại"

**Phát hiện ở:** Task 2, lúc viết vòng lặp tách trường.

Plan viết: *"Nhiều hơn thì nối các phần thừa trở lại vào trường cuối (`%b`) bằng `\x1f`"*.
Làm đúng chữ đó nghĩa là tách hết rồi ghép lại — cấp phát thừa, và trên đường nóng chạy
100k lần.

**Đã làm:** dừng tách ngay khi đủ chín dấu phân tách (`break`), phần còn lại của bản ghi
đi thẳng vào `body` nguyên vẹn. Kết quả byte **y hệt** phương án của plan, nhưng không
cấp phát và không ghép chuỗi. Test `byte_1f_trong_body_khong_tach_thanh_hai_ban_ghi`
khẳng định `body == "than truoc\u{1f}than sau"`, tức `\x1f` được giữ nguyên.

### `[Rule 2 - Missing critical]` Thêm `bad_timestamps` vào `LogParseResult`

Plan mô tả hành vi *"Thất bại → `0` và tăng một cảnh báo"* nhưng struct trả về trong plan
chỉ có `commits` và `skipped_records` — không có chỗ nào cho "một cảnh báo" đi ra. Đã
thêm trường thứ ba. Không có nó thì timestamp hỏng biến mất trong im lặng, đúng thứ mà
lý lẽ của `skipped_records` nói là không chấp nhận được.

### `[Rule 2 - Missing critical]` Thêm hằng `LOG_ARGS`

Plan gợi ý *"Consider making the argument list a constant alongside `LOG_FORMAT`"* trong
ràng buộc số 3 nhưng phần `<action>` không yêu cầu. Đã thêm, kèm test
`log_args_luon_co_topo_order`. Lý do: `--topo-order` là bất biến mà thuật toán lane dựa
vào, và một bất biến chỉ sống trong văn bản kế hoạch thì không tồn tại — call site đầu
tiên quên nó sẽ không ai biết.

### `[Rule 2 - Missing critical]` Test `log_format_co_dung_so_truong`

`LOG_FORMAT` và `FIELD_COUNT` là hợp đồng hai chiều. Sửa một bên quên bên kia làm **mọi**
bản ghi lệch một nấc — dữ liệu sai chỗ mà không có lỗi nào được báo. Test đếm số `%x1f`
trong hằng và so với `FIELD_COUNT - 1`.

### `[Rule 3 - Blocking]` clippy `assertions_on_constants`

Hai `assert!` trên hằng số ở `graph/types.rs` bị clippy từ chối với `-D warnings`. Chuyển
sang `const { assert!(..) }` — kiểm ngay lúc biên dịch, mạnh hơn kiểm lúc chạy.

### Hai test tích hợp thêm ngoài yêu cầu

Plan đòi **một** test tích hợp trên `non-utf8`. Đã thêm hai nữa:
`octopus_doc_du_bon_cha_tu_repo_that` (bốn cha đọc từ dữ liệu git thật, không phải buffer
dựng tay) và `moi_repo_mau_phan_tich_tron_ven` (cả chín repo). Chi phí: 0,6 giây tổng.

## Plan sai ở đâu

**Mốc hiệu năng của plan 02-01 không so sánh được trực tiếp.** 847ms/8,4MB đo trên định
dạng hai trường; định dạng thật cho 18,9MB. Xem phần Hiệu năng ở trên.

**`bstr` không cần thiết cho bộ phân tích này.** CONTEXT.md chốt *"Dùng `bstr` phân tích
`Vec<u8>`"* và crate đã có trong `Cargo.toml`, nhưng `parse_log` không dùng tới: việc cần
làm chỉ là tách theo hai byte (`memchr` lo) và giải mã lossy (`std` lo). Thêm `bstr` vào
đây sẽ là phụ thuộc trang trí. Crate vẫn còn trong `Cargo.toml` cho plan 02-04/02-05 —
bộ phân tích `for-each-ref` và `diff --name-status -z` có thể cần thao tác chuỗi byte
thật sự (tìm tiền tố, cắt đường dẫn), nơi `bstr` có giá trị.

## Cổng an toàn đã cài (threat model)

- **T-02-04 (DoS)** — `parse_log` không có `unwrap` trên `from_utf8`, không chỉ mục mảng
  trần (mọi truy cập `fields[i]` đứng sau kiểm `fields.len() < FIELD_COUNT`), và
  `parse_i64` dùng `checked_mul`/`checked_add` nên timestamp rác vài trăm chữ số cho
  `None` chứ không tràn. Test cho buffer rỗng và buffer cắt dở.
- **T-02-05 (Tampering)** — `\x1f` do người tạo commit chèn không sinh được bản ghi giả;
  hành vi có test tường minh.
- **T-02-06 (Info disclosure)** — chưa tới (plan 02-06).
- **T-02-07 (Spoofing)** — chấp nhận; doc comment của `author_name` ghi rõ trường này tự
  khai, không dùng cho quyết định tin cậy.

## Known Stubs

Không có stub. Hai kiểu `Ref` và `GraphRow` **chưa có bộ sinh dữ liệu** — đó là điều plan
cố ý (02-04 điền `Ref`, 02-03 điền `GraphRow`), không phải stub: chúng là khai báo hợp
đồng, không phải mã giả vờ hoạt động.

`MAX_VISIBLE_LANES = 32` là giá trị tạm đã ghi rõ trong doc comment và trong plan; 02-03
chốt lại.

## Ghi chú cho plan sau

**Cho 02-03 (thuật toán lane):**
- `GraphRow`/`Edge` đã sẵn, chỉ còn cài thuật toán.
- `MAX_VISIBLE_LANES = 32` cần chốt **từ bề rộng hiển thị**, không từ số lane của `wide`
  (lập luận vòng tròn — xem doc comment của hằng).
- `Commit.parents` rỗng = commit gốc thật → `terminates: false`. Cha không có trong tập
  đã nạp → `terminates: true`. Hai ca này khác nhau và fixture `shallow` phân biệt được.
- Ngân sách còn lại: git 693ms + parse 63ms = 758ms, còn khoảng **240ms** cho gán lane
  trước khi chạm mốc một giây.

**Cho 02-04 (refs + command):**
- Dùng `LOG_ARGS` và `LOG_FORMAT`, **không** chép lại chuỗi định dạng.
- Ghi `tracing::warn!` khi `skipped_records != 0` hoặc `bad_timestamps != 0`.
- `--max-count` / `--skip` nối thêm sau `LOG_ARGS` cho phân trang.
- `LogParseResult` chưa có `Serialize` — cố ý, nó là kiểu nội bộ. Command trả
  `Vec<Commit>` kèm log cảnh báo.

**Cho 02-06 (giao diện):** hai bảng trường ở trên là hợp đồng; tên khoá đã ghim bằng test
nên đổi phía Rust sẽ làm test đỏ trước khi làm giao diện trắng.

## Trạng thái requirement

**HIST-01 và HIST-11 giữ `Pending`.** Bộ phân tích là điều kiện **cần** cho cả hai nhưng
chưa **đủ**: HIST-01 là *"danh sách commit, nạp theo trang"* và HIST-11 là *"tên tệp và
thông điệp không UTF-8 vẫn hiển thị được"* — plan này chưa có command, chưa có phân
trang, và chưa hiển thị gì. Trường `requirements:` của plan hiểu là *"góp phần vào"*,
theo đúng tiền lệ plan 02-01 đã lập.

## Self-Check: PASSED

Tệp đã tạo (đều tồn tại):
- `src-tauri/src/domain/commit.rs`
- `src-tauri/src/domain/refs.rs`
- `src-tauri/src/graph/mod.rs`
- `src-tauri/src/graph/types.rs`
- `src-tauri/src/git/parsers/log.rs`

Commit đã tạo (đều có trong `git log`):
- `df251ee` feat(02-02): declare the phase-wide data contracts up front
- `77b3bc9` feat(02-02): parse git log from bytes so one bad commit cannot blank a page
