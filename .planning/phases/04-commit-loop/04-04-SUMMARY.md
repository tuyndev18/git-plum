---
phase: 04-commit-loop
plan: 04
wave: 4
subsystem: watcher + hàng WIP
tags: [WORK-10, WORK-11, watcher, notify, wip-row, lane-clamp]
requires: [04-02, 04-03]
provides:
  - "watch::SoTayWatcher — watcher .git theo RepoId, gộp 280ms"
  - "watch::should_notify — phép lọc thuần, từ chối cây làm việc"
  - "wipRow.ts — NGUỒN DUY NHẤT của mọi phép ±1 của hàng WIP"
  - "wipEdge — cạnh WIP nối xuống lane THẬT của HEAD, có biến thể clamped"
  - "scripts/fixtures/make-lane-clamp-fixture.sh — fixture suy từ cap"
affects: [04-05]
tech-stack:
  added: ["notify 8.2.0", "notify-debouncer-full 0.6.0"]
key-files:
  created:
    - src-tauri/src/watch/mod.rs
    - src-tauri/src/watch/paths.rs
    - src/lib/graph-render/wipRow.ts
    - src/lib/graph-render/wipRow.test.ts
    - src/stores/statusStore.watcher.test.ts
    - scripts/fixtures/make-lane-clamp-fixture.sh
  modified:
    - src-tauri/src/state/mod.rs
    - src-tauri/src/commands/repo.rs
    - src/lib/ipc.ts
    - src/stores/statusStore.ts
metrics:
  rust_tests: "277 → 309 (--lib)"
  frontend_tests: "594 → 645"
  completed: 2026-09-22
---

# Phase 4 Plan 04: Watcher `.git` + toàn bộ logic hàng WIP — Summary

Watcher `.git` hẹp có gộp 280 ms (WORK-10) và **toàn bộ** số học chỉ số + hình học của
hàng WIP gom vào một module thuần có cổng ghim (WORK-11), với lỗi "cạnh WIP bị clamp vẽ
sai cột" được tái hiện bằng fixture và ghim bằng hai test đối nghịch.

## Số đo trước/sau

| Phép đo | Trước | Sau | Lệnh |
|---|---:|---:|---|
| Rust `cargo test --lib` | 277 | **309** | `cd src-tauri && cargo test --lib` |
| Frontend | 594 | **645** | `npx vitest run --reporter=json` |
| `npx tsc --noEmit` | sạch | **sạch** | — |
| `npm run build` | — | **exit 0** | — |
| `cargo clippy --all-targets` | 2 warning | **0 warning** | xem ghi chú dưới |

**`cargo test --lib --tests` là CHƯA ĐO.** Ứng dụng của chủ dự án (`tauri dev`, PID
58556/28552/7736 ở các thời điểm khác nhau) giữ `target/debug/git-plum.exe` gần như
suốt phiên, và CONTEXT.md 3.6 nói rõ chỉ định target không thoát được — cargo relink
binary chính kể cả khi chỉ yêu cầu test target. Theo đúng quy tắc của mục đó: báo
`--lib` là số **đo được**, `--lib --tests` là **chưa đo**, không suy ra tổng.

**Clippy 0 warning KHÔNG phải công của wave này.** Baseline ghi 2 warning
(`diff.rs:1708`, `rowdump.rs:35`); cả hai giờ im vì một session khác đã thêm
`#[allow(clippy::assertions_on_constants)]` ở `diff.rs:1713` và `rowdump.rs` là tệp
chưa theo dõi của họ. Đã kiểm: tôi **không** sửa tệp nào trong hai tệp đó. Ghi ở đây
để không ai đọc nhầm thành "wave 4 dọn được clippy".

## 🔴 `lanedist` trên fixture mới — bằng chứng fixture phân biệt được

```text
repo: ../target/fixtures/lane-clamp
commit: 23, hang: 23
MAX_VISIBLE_LANES = 20, lane lon nhat that = 21
hang co lane >= cap (bi clamp vao cot 19): 2 (8.6957%)
hang co cha bi luoc: 0 (0.0000%), tong cha bi luoc: 0
hang 0: lane 0 (commit moi nhat theo topo cua MOI ref)
HEAD that su: hang 21, lane 21 -> canh hang WIP BI CLAMP, se ve sai cot
```

`hang 0: lane 0` trong khi `HEAD that su: hang 21, lane 21` — **đúng** hình dạng mà lỗi
cần: hàng 0 và HEAD là hai thứ khác nhau, và `laneX(21)` gập về cột 19. `21 >= cap 20`
nên fixture **kích hoạt được** ca clamp. Repo mẫu `wide` đã có không dùng được (kiểm
lại: HEAD ở hàng 0, lane 0).

Số nhánh **suy từ cap**, không viết cứng: script đọc `MAX_VISIBLE_LANES` từ
`src-tauri/src/graph/types.rs` rồi sinh `CAP + 2` nhánh (hiện là 22). Cap đổi lần thứ
ba sẽ không tốn gì. Script **tự kiểm** và thất bại nếu HEAD rơi vào lane `< CAP`.

## 🔴 RAM và handle: 1 repo so với 5 repo (release)

Đo bằng một bin `watchmem` dùng một lần, profile **release**, `Win32_Process
WorkingSetSize` + `HandleCount` của chính tiến trình:

```text
 watcher  workingSet_MB        them_MB     handle them_handle
       0           4.66           0.00        104          0
       1           4.77           0.11        111          7
       2           4.85           0.19        118         14
       3           4.95           0.29        125         21
       4           5.02           0.36        132         28
       5           5.11           0.45        139         35

TONG them cho 5 watcher: 0.45 MB, 35 handle
TRUNG BINH moi watcher: 0.09 MB, 7.0 handle

=== sau khi dung TAT CA watcher ===
so watcher song: 0
workingSet 4.76 MB (dinh 5.11 MB), handle 104 (dinh 139)
handle tra lai: 35
```

**Kết luận: nó vừa, rất thoải mái.** Mỗi watcher tốn **0,09 MB và 7 handle**, tuyến
tính sạch. Năm repo mở cùng lúc thêm **0,45 MB** vào ngân sách RAM lúc rảnh **150 MB** —
0,3%. Kể cả 50 repo mở cùng lúc cũng chỉ là 4,5 MB và 350 handle, vẫn xa mọi ngưỡng.

**Nghĩa là quyết định thiết kế "watcher cho mọi repo mở" KHÔNG bị ép bởi RAM.** Bản này
chọn nó vì đó là hành vi WORK-10 mô tả nguyên văn (repo không hoạt động **vẫn** thấy
`git commit` chạy ở terminal), và số đo nói rằng lựa chọn kia ("chỉ repo đang hoạt
động") không mua được gì đáng kể. Quyết định vẫn thuộc chủ dự án, nhưng nó không còn là
một đánh đổi — nó là một lựa chọn gần như miễn phí.

`handle tra lai: 35` (đúng bằng số đã cấp) xác nhận `dung()` giải phóng thật, khớp với
test `mo_dong_lap_lai_khong_tich_luy`.

⚠️ Con số này là phần **watcher đóng góp thêm** trong một tiến trình trần 4,66 MB, không
phải RAM của cả ứng dụng Tauri có webview. Ràng buộc 150 MB áp cho cả ứng dụng, và phần
đó **chưa đo** ở wave này.

## Bảng đột biến M1–M16 — tất cả đã CHẠY, không suy luận

| # | Đột biến | Kết quả | Test đỏ |
|---|---|---:|---|
| M1 | `should_notify` trả `true` mọi đường dẫn | **7 đỏ** | `tu_choi_duong_dan_trong_thu_muc_lam_viec`, `tu_choi_objects`, `tu_choi_logs_va_cac_thu_muc_lon_khac`, `tu_choi_tep_lock`, `tu_choi_tep_git_khong_trong_danh_sach`, `hieu_dau_gach_cheo_nguoc_cua_windows`, `duong_dan_khong_di_qua_git_bi_tu_choi` |
| M2 | Bỏ `packed-refs` khỏi `WATCHED` | **2 đỏ** | `danh_sach_theo_doi_co_dung_tung_ten_mot`, `packed_refs_duoc_theo_doi` |
| M3 | Cửa sổ trì hoãn về 0 ms | **4 đỏ** | `ba_su_kien_cua_mot_lan_commit_sinh_mot_lan_lam_moi`, `cua_so_tri_hoan_nam_trong_250_300ms`, `su_kien_cach_nhau_qua_cua_so_sinh_hai_lan_lam_moi`, `chuoi_su_kien_day_dac_bi_gop_manh` |
| M4 | Cửa sổ trì hoãn lên 5000 ms | **2 đỏ** | `cua_so_tri_hoan_nam_trong_250_300ms` (biên **trên**), `chuoi_su_kien_day_dac_bi_gop_manh` |
| M5 | `WIP_ROW_HEIGHT = 30` cứng | **2 đỏ** | `WIP_ROW_HEIGHT === ROW_HEIGHT`, `contentOffset hasWip=true` |
| M6 | Thêm **mã thật** `+ WIP_ROW_HEIGHT` vào `canvasRenderer.ts` | **1 đỏ** | cổng `KHÔNG tệp nào khác ... làm số học với hằng WIP` |
| M7 | **NGHỊCH** — chỉ thêm **chú thích** `// WIP_ROW_HEIGHT + 1` | **0 đỏ = ĐẠT** | — (xem dưới) |
| M8 | `commitIndexFor` trả `i - 1` | **3 đỏ** | `commitIndexFor(0/1/999) === chính nó` |
| M9 | `virtualizerCount` trả `total + 1` | **3 đỏ** | `total 0/1/100007, hasWip true → total không đổi` |
| M10 | `contentOffset` trả `ROW_HEIGHT` cả khi `hasWip=false` | **6 đỏ** | `hasWip=false → 0` + 4 ca hồi quy `commitRowY` + ca chênh lệch |
| M11 | 🔴 `wipEdge` bỏ phép kiểm cap | **4 đỏ** | `headLane=26 → KHÔNG ở laneX(19)`, `headLane=20 → clamped`, `clamped mang realLane`, `hàng 0 lane 0 + headLane cao` |
| M12 | 🔴 `wipEdge` bỏ cạnh khi `headLane >= CAP-1` | **1 đỏ** | `headLane=19 (ĐÚNG biên) → CÓ vẽ ở laneX(19)` |
| M13 | 🔴 `wipEdge` lấy lane từ `rowZeroLane` | **2 đỏ** | `hàng 0 lane 0 + headLane cao → theo HEAD`, và ca **chiều ngược lại** |
| M14 | `headLane === undefined` → vẽ vào lane 0 | **2 đỏ** | `undefined → unknownHead`, `null → unknownHead` |
| M15 | `wipEdge` trả `null` khi vượt cap | **4 đỏ** | 3 ca `TypeError: Cannot read properties of null`, 1 ca `expected null not to be null` |
| M16 | Hàng WIP hiện khi `wipCounts = {0,0}` | **1 đỏ** | `{modified:0, added:0} → KHÔNG có hàng WIP` |

### 🔴 M7 cho **0 đỏ** — và đó là kết quả ĐẠT

Đột biến nghịch, ghim rằng cổng **không** khớp chú thích. Đầu vào nguyên văn thêm vào
`canvasRenderer.ts`:

```ts
// WIP_ROW_HEIGHT + 1
/* doLech = WIP_ROW_HEIGHT * 2 */
const ghiChu = 'WIP_ROW_HEIGHT - 1'
void ghiChu
```

Đầu ra nguyên văn:

```text
=== M7 (NGHICH: chi them CHU THICH vao canvasRenderer.ts): total 78, ĐỎ 0 ===
```

**Cổng không đỏ = lỗi #5 của Phase 3 KHÔNG lặp lại.** Và M6 — cùng một tệp, cùng một
hằng, nhưng là **mã thật** — cho **1 đỏ**, nên "0 đỏ ở M7" là *phân biệt được*, không
phải *cổng chết*. Hai dòng này chỉ có nghĩa khi đọc cùng nhau.

Cổng còn tự kiểm `boChuThichVaChuoi` cả hai chiều trong chính test (chú thích/chuỗi →
0 khớp; mã thật → 1 khớp; `commitRowY(...)` và dòng `import` → 0 khớp). Không có phép
tự kiểm đó thì hàm bỏ chú thích có thể là hàm đồng nhất mà cổng vẫn "xanh".

### Một đột biến KHÔNG áp được cũng đã được báo là không-bằng-chứng

Lần chạy đầu của M1/M2 sai đường dẫn (`src-tauri/src-tauri/...`). Harness **từ chối**
báo số và in `KHÔNG ÁP ĐƯỢC ĐỘT BIẾN — không phải bằng chứng về cổng`, đúng bài học
"một đột biến không thể biểu đạt khuyết tật thì không phải bằng chứng về cổng". Chạy
lại sau khi sửa đường dẫn mới cho số ở bảng trên.

## Test có thời gian chạy 5 lần liên tiếp

```text
--- lần 1 --- test result: ok. 30 passed; 0 failed; 0 ignored; 279 filtered out; finished in 1.46s
--- lần 2 --- test result: ok. 30 passed; 0 failed; 0 ignored; 279 filtered out; finished in 1.46s
--- lần 3 --- test result: ok. 30 passed; 0 failed; 0 ignored; 279 filtered out; finished in 1.46s
--- lần 4 --- test result: ok. 30 passed; 0 failed; 0 ignored; 279 filtered out; finished in 1.46s
--- lần 5 --- test result: ok. 30 passed; 0 failed; 0 ignored; 279 filtered out; finished in 1.46s
```

5/5 xanh, thời gian y hệt nhau. Lý do ổn định: **phép quyết định không chạm thời gian
thật.** `BoGop` nhận `Instant` **tiêm vào**, nên "ba sự kiện → một lần làm mới" là một
test tất định. Chỉ **hai** test mỏng chạm hệ tệp thật
(`ghi_that_vao_git_head_lam_callback_chay` và
`ghi_vao_thu_muc_lam_viec_khong_lam_callback_chay`), và cả hai chờ ~3× cửa sổ rồi
khẳng định `>= 1` / `== 0` chứ không khẳng định một con số chính xác — khẳng định con
số chính xác ở đó là đòi hệ tệp phải tất định, mà nó không.

Suite frontend chạy **3 lần**, `rm -f .vitest/json/output.json` trước mỗi lần:
`645 / 645 / 645`, đều `failed 0`.

## Quyết định: **cách B** cho hàng WIP (để 04-05 chỉ phải nối dây)

`count: total` **không đổi**. Hàng WIP là phần tử **anh em** của vùng cuộn, ghim đầu
danh sách, cao đúng `ROW_HEIGHT`. Phép "+1" duy nhất là **độ lệch pixel**
`contentOffset`, và **cả** cột đồ thị **lẫn** cột văn bản đọc nó qua **cùng** hàm
`commitRowY`.

**Cách A** (`count: total + 1`, mọi chỗ đọc `commits[index - 1]`) bị từ chối vì nó đặt
một phép `-1` vào **bảy** chỗ tiêu thụ `virtualItems`: `renderRows`, `commits[v.index]`,
`graphRows[v.index]`, `ensureRange`, `indexById`, `scrollToIndex`, `scrollToCommit`.
Bảy chỗ là bảy dịp để lệch, và lớp lỗi đó đã xảy ra **hai lần** ở Phase 2.

Cách B thoả ROADMAP đúng nghĩa hơn: nó không đếm hàng WIP vào `total` **một cách nào
cả**, nên ánh xạ `commits[index]` **không thể** lệch về mặt **cấu trúc**, chứ không phải
nhờ sửa cho thẳng.

**Hệ quả phải chấp nhận:** hàng WIP **không cuộn đi** cùng danh sách. Với WORK-11 đó là
hành vi đúng (nó là chỗ vào vùng soạn commit nên luôn bấm được), nhưng đây là **điểm
cần mắt người xác nhận** ở checkpoint 04-05.

**Cho 04-05, bước nối dây là cơ học:**
`virtualizerCount(total, hasWip)` → `count`; `commitRowY(v.start, scrollTop, hasWip)`
cho **cả hai** cột; `commitIndexFor(v.index)` cho mọi phép tra `commits[]`;
`hasWipRow(status.wipCounts)` quyết định hiện/ẩn; `wipEdge({ headLane })` cho cạnh.
Không còn số học chỉ số nào phải nghĩ.

## Ngưỡng viết theo `MAX_VISIBLE_LANES`, không viết số

`CAP = MAX_VISIBLE_LANES` (nhập từ `geometry.ts`), `LANE_TRAN = CAP - 1`,
`LANE_VUOT = CAP + 6`. Tên test tự sinh theo cap — ở cap 20 chúng in ra là
`headLane = 26 (>= cap 20) → cạnh KHÔNG vẽ ở laneX(19)`. Cap đổi lần sau, test vẫn đúng
mà không ai phải nhớ sửa.

**Hai test đối nghịch, và chúng chỉ có nghĩa khi đi đôi:**
`headLane = LANE_VUOT` → **KHÔNG** `normal`; `headLane = LANE_TRAN` → **CÓ** `normal` ở
`laneX(LANE_TRAN)`. M11 (bỏ phép kiểm cap) đỏ ở ca một; M12 (bỏ cạnh với mọi lane
`>= CAP-1`) đỏ ở ca hai. Thiếu ca hai thì ca một không chứng minh gì.

`wipEdge` so ở miền **lane**, **trước** khi quy đổi pixel — `laneX(CAP+6) ===
laneX(CAP-1)` nên so ở miền pixel **không phân biệt được** hai ca, và đó chính là lỗi.
`laneX` **không** bị sửa (sửa nó là đổi hành vi vẽ của cả Phase 2).

## Deviations from Plan

### [Rule 1 — Bug] Cổng "đúng một chỗ" bản đầu là cổng VÔ NGHĨA, tiền đề bắt được

**Tìm thấy ở:** Task 2, lần chạy GREEN đầu tiên.
**Vấn đề:** `contentOffset` dùng toán tử **ba ngôi** (`hasWip ? WIP_ROW_HEIGHT : 0`),
không phải toán tử số học, nên bộ so khớp `[-+*/]` tìm thấy **0** khớp trong chính
`wipRow.ts` — tức cổng đang khẳng định "không tệp nào khác làm số học" trong khi **cũng
không có tệp nào** làm số học cả. Một cổng ghim một tập rỗng.
**Cách bắt:** test tiền đề `wipRow.ts phải là nơi CHỨA phép số học, nếu không cổng vô
nghĩa` đỏ ngay lần chạy đầu. Đây đúng là thứ ràng buộc "khẳng định tiền đề" tồn tại để
bắt, và nó bắt được ở lần đầu tiên nó có cơ hội.
**Sửa:** thêm `contentOffset\s*\([^)]*\)` vào bộ so khớp — nó **là** phép cộng, và một
tệp khác viết `v.start - scrollTop + contentOffset(hasWip)` đã dựng lại công thức của
`commitRowY` ở chỗ thứ hai. Thêm cả ba phép tự kiểm chiều ngược (chỗ gọi `commitRowY`
và dòng `import` **không** bị bắt), vì một cổng cấm luôn việc *dùng* module sẽ đỏ mãi
ở 04-05 rồi bị ai đó tắt đi.
**Tệp:** `src/lib/graph-render/wipRow.test.ts`. **Commit:** `c7ed8c0`.

### [Rule 2 — Thiếu chức năng thiết yếu] Lọc `*.lock` không có trong plan

Plan liệt kê danh sách theo dõi nhưng không nói gì về `.git/index.lock`. Thêm phép từ
chối mọi `*.lock` (Rule 2): tệp lock xuất hiện rồi biến mất ở **mỗi** lệnh git kể cả
lệnh của chính ứng dụng, nên nghe nó **nhân đôi** số sự kiện cho mọi thao tác — và làm
mới **giữa lúc** một lệnh git khác đang ghi là đọc đúng trạng thái nửa vời. Đây là nửa
còn lại của ràng buộc 2.3, có test ghim.

### [Rule 2] `open_repository` KHÔNG thất bại khi không theo dõi được

Plan không nói xử lý ra sao khi `bat_dau` lỗi. Chọn: ghi log rồi đi tiếp. Watcher là
**lớp phòng thủ**; đường chính là "thao tác ghi trả trạng thái trực tiếp" (2.5). Một
repo mở được nhưng không theo dõi được (đĩa mạng, quyền, hết handle) vẫn phải **dùng
được** — trả lỗi ở đó biến một suy giảm thành một lỗi chặn.

## Claim của plan hoá ra sai hoặc cần chỉnh

1. **`cargo add --dry-run` KHÔNG in cảnh báo rust-version.** Plan bảo "xác nhận lại
   bằng `cargo add --dry-run`". Làm thế cho kết quả **gây hiểu nhầm**: `--dry-run` với
   `notify-debouncer-full@0.7.0` in `Adding notify-debouncer-full v0.7.0` **không kèm
   cảnh báo** nào. Chỉ `cargo add` **thật** (không ghim phiên bản) mới lộ ra:
   ```text
   warning: ignoring notify-debouncer-full@0.7.0 (which requires rustc 1.85)
            to maintain git-plum's rust-version of 1.82
         Adding notify-debouncer-full v0.6.0 to dependencies
   ```
   **Nội dung** đính chính của plan đúng nguyên văn; chỉ **cách kiểm** nó đề xuất là
   không đủ. `cargo tree -i notify` xác nhận đúng một bản `notify 8.2.0` với cả
   `git-plum` và `notify-debouncer-full 0.6.0` treo trên nó.

2. **Baseline clippy "2 warning, cả hai pre-existing" giờ là 0** — vì một session khác
   đã thêm `#[allow]`. Không phải wave này sửa. Xem mục Số đo.

3. **`verification` của plan ghi `cargo clippy` phải "vẫn đúng 1 warning cũ
   (diff.rs:1470)"** trong khi phần brief ghi 2 warning ở `diff.rs:1708`/`rowdump.rs:35`.
   Ba con số, ba chỗ khác nhau. Thực tế đo được: **0**.

4. **Plan ghim `lanedist` phải in `hang 19, lane 19`.** Ở cap 20 nó in `hang 21, lane
   21`. Con số 19 suy từ cap 13 — cùng lớp lạc hậu mà chính plan cảnh báo ở chỗ khác.
   Điều kiện thật (`L >= CAP`) vẫn thoả.

## Chưa kiểm chứng — nêu thẳng, không suy ra

- **`cargo test --lib --tests`**: **chưa đo** (ứng dụng của chủ dự án giữ binary gần như
  suốt phiên). Không suy tổng từ `--lib`.
- **Tiêu chí 5 ("dưới một giây khi chạy git từ terminal ngoài")**: **chưa kiểm**. Test
  chứng minh watcher bắt được sự kiện và gộp đúng, nhưng độ trễ đầu-cuối tới mắt người
  chỉ kiểm được bằng người chạy ứng dụng — checkpoint 04-05.
- **Hai tiêu chí layout** (hàng WIP thẳng cột ở mọi vị trí cuộn; hiện/biến mất đúng
  lúc): **chưa kiểm**. happy-dom không tính layout (CONTEXT.md 3.4). Có mã, có test
  logic, **chưa** có mắt người.
- **RAM của cả ứng dụng Tauri** (webview + watcher): **chưa đo**. Con số 0,45 MB ở trên
  chỉ là phần watcher đóng góp thêm trong một tiến trình trần.
- **`notify` trên macOS/Linux**: **chưa kiểm**. Chỉ đo trên Windows 11.
- **Ca `refs/` đệ quy trên repo có hàng nghìn nhánh**: **chưa đo**. `refs/` là đường
  theo dõi đệ quy duy nhất và nó có thể lớn trên repo thật.

## 🔴 Ba tệp KHÔNG commit được — xung đột với session khác

Git không stage được một phần tệp, và ba tệp dưới đây mang thay đổi của session khác
trộn lẫn với của tôi. **Tất cả đều ở trong cây làm việc, đã kiểm biên dịch/test, nhưng
CHƯA commit:**

| Tệp | Thay đổi của tôi | Bị chặn bởi |
|---|---|---|
| `src-tauri/src/lib.rs` | `pub mod watch;` | `commands::avatar_hash` của session avatar |
| `src-tauri/Cargo.toml` | hai dependency `notify` + doc comment | `md-5`, `default-run` của session khác |
| `src-tauri/Cargo.lock` | khoá `notify` 8.2.0 / debouncer 0.6.0 | đi kèm `Cargo.toml` |

**Hệ quả thực tế:** ai `git checkout` commit `e05a29d` mà không có cây làm việc này sẽ
**không biên dịch được** — `watch/mod.rs` tồn tại nhưng `lib.rs` chưa khai `pub mod
watch;` và `Cargo.toml` chưa có `notify`. Ba dòng cần thêm khi session kia giao lại:

```toml
# src-tauri/Cargo.toml, mục [dependencies]
notify = "8.2.0"
notify-debouncer-full = "0.6.0"
```
```rust
// src-tauri/src/lib.rs, cạnh `pub mod state;`
pub mod watch;
```

Không dùng `git add -A`; mọi commit của wave này stage theo **đường dẫn tường minh**, và
mỗi lần đều chạy `git diff --cached | grep -iE "avatar|md-5|default-run|insta"` trước khi
commit. Một lần đã bắt được `commands::avatar_hash` lọt vào qua `lib.rs` và đã
`git restore --staged` ngay.

**Cũng không sửa:** `CommitList.tsx`, `GraphCanvas.tsx`, `App.tsx`, `app.css`,
`geometry.ts`, `types.ts`, `canvasRenderer.ts`, `ROADMAP.md`, `REQUIREMENTS.md`,
`STATE.md`. Đã xác nhận bằng `git status --porcelain=v2` sau mỗi lần chạy đột biến rằng
harness khôi phục nguyên trạng (so `sha256sum`).

Vì `App.tsx` thuộc session khác, `noiWatcherVaoStore()` **chưa được gọi** ở đâu — nó đã
có, đã test (8 test), nhưng việc mount nó là **một dòng** ở 04-05:

```ts
useEffect(() => { const p = noiWatcherVaoStore(); return () => { p.then(huy => huy()) } }, [])
```

## Commits

| Hash | Nội dung |
|---|---|
| `c7ed8c0` | `feat(04-04)`: `wipRow.ts` — nguồn duy nhất của phép ánh xạ chỉ số hàng WIP |
| `e05a29d` | `feat(04-04)`: watcher `.git` hẹp, gộp và trì hoãn 280 ms |
| `5d26a79` | `test(04-04)`: fixture repo kích hoạt được ca clamp cạnh WIP |

## Self-Check: PASSED
