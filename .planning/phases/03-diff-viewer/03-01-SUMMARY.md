---
phase: 03-diff-viewer
plan: 01
subsystem: diff-viewer
tags: [spike, performance, codemirror, checkpoint]
requires:
  - "src-tauri/src/git/exec.rs (GitCommand, GitRunner)"
  - "src/lib/perf.ts (isPerfEnabled)"
  - "src/lib/ipc.ts (object ipc)"
provides:
  - "spike_blob_pair — hai phía nội dung + patch thô cho cùng một cặp (sha, path)"
  - "SpikeHarness — khung đo hai đường A/B trong app thật"
  - "docs/09-phase3-diff-decision.md — ngưỡng chốt trước, chờ số đo"
  - "--no-ext-diff tập trung trong GitCommand::run() — sửa lỗi Phase 1"
affects:
  - "03-03 (word-level diff) — phụ thuộc --word-diff=porcelain, nay đã chạy được"
  - "03-04 (trình xem diff) — đọc quyết định A/B làm đầu vào"
  - "Phase 5 (bản vá) — mọi đường bản vá đều đi qua bản sửa --no-ext-diff"
tech-stack:
  added:
    - "@codemirror/state 6.7.5"
    - "@codemirror/view 6.43.12"
    - "@codemirror/merge 6.12.2"
  patterns:
    - "Test đọc thân hàm thật để ghim argv (HIST-10), không tự dựng lệnh git riêng"
    - "Cờ đo đọc localStorage lúc chạy → đo được trên bản release đã dựng"
    - "lazy(import()) để gói nặng không vào bundle đường chính"
key-files:
  created:
    - "src-tauri/src/commands/diff_spike.rs"
    - "src-tauri/tests/diff_spike.rs"
    - "src/lib/diffSpike.ts"
    - "src/lib/diffSpike.test.ts"
    - "src/components/diff/SpikeHarness.tsx"
    - "src/components/diff/SpikeHarness.test.tsx"
    - "docs/09-phase3-diff-decision.md"
  modified:
    - "src-tauri/src/git/exec.rs (sửa lỗi GIT_EXTERNAL_DIFF)"
    - "src-tauri/src/commands/mod.rs"
    - "src-tauri/src/lib.rs"
    - "src/lib/ipc.ts"
    - "src/App.tsx"
    - "src/styles/app.css"
    - "package.json"
decisions:
  - "Quyết định A/B CHƯA CÓ — cố ý. Checkpoint #3 là cổng người kiểm, plan không được đoán con số"
  - "Sửa GIT_EXTERNAL_DIFF bằng --no-ext-diff chứ không bằng diff.external= (cùng lỗi)"
  - "Cổng isPerfEnabled() đặt ở App.tsx, không trong SpikeHarness — lệch plan, có lý do"
metrics:
  duration: "~50 phút"
  completed: "2026-09-21"
  tasks_completed: "2/3 (Task 3 là checkpoint chờ người kiểm)"
  tests_added: "9 Rust + 14 frontend"
  tests_total: "163 Rust (+1 ignored), 226 frontend"
---

# Phase 3 Plan 01: Spike đo `@codemirror/merge` Summary

Dựng công cụ đo hai đường A/B cho checkpoint #3 và chốt ngưỡng trước khi đo — rồi
phát hiện một lỗi Phase 1 làm **mọi** lệnh `git diff` sinh bản vá thất bại trong im lặng.

---

## Đã làm

### Task 1 — `spike_blob_pair` (Rust)

Một command trả **cả hai** dạng dữ liệu cho cùng một cặp `(sha, path)`:
`oldText`/`newText` (đầu vào đường A) và `patch` (đầu vào đường B). Một lời gọi chứ
không hai, vì hai đường phải đo trên **cùng byte đầu vào** — nếu mỗi đường tự lấy dữ
liệu riêng thì phép so sánh khác nhau ở hai biến và mất nghĩa.

`rustMs` trả riêng để tách "git chậm" khỏi "CodeMirror chậm" — bài học Phase 2, nơi
`git log` chiếm 84% tổng thời gian và một con số tổng gộp không nói được phải tối ưu
ở đâu.

Ba test tích hợp chạy git **thật** trên repo tạm: tệp sửa thường, **commit gốc**
(phía cũ rỗng, trả `Ok` chứ không `Err` — tệp mới thêm là ca bình thường), và cổng
`--`.

### Task 2 — Khung đo A/B (React)

Ba quy tắc đo, mỗi quy tắc chống một cách đo sai cụ thể:

1. **Chờ hai khung `requestAnimationFrame`**, không một. Khung đầu có thể chạy trước
   lúc trình duyệt dựng bố cục. Cùng bài học với `measureFirstPaint` của Phase 2.
2. **Ba lần, báo cả ba số**, không trung bình — lần đầu gánh chi phí nạp module.
3. **Tháo view giữa hai lần đo**, nếu không lần sau đo trên view đã ấm.

Chỉ cài ba gói `@codemirror/*` cần thiết; gói meta `codemirror` **không** có mặt.
Chưa cài `@codemirror/language` hay `lang-*`: spike đo **thời gian tính diff**, thêm
tô màu cú pháp vào sẽ trộn hai biến vào một con số.

**Bằng chứng gói CodeMirror không vào bundle chính** — đầu ra `vite build` thật:

```
dist/assets/SpikeHarness-BSM5Bioi.js    4.46 kB │ gzip:  1.44 kB
dist/assets/diffSpike-DTjaXijx.js     253.96 kB │ gzip: 82.30 kB   ← chunk riêng
dist/assets/index-DrMy3mq_.js         321.47 kB │ gzip: 100.30 kB  ← bundle chính
```

### Task 3 — ⏸️ Checkpoint, chờ người kiểm

Công cụ xong, ngưỡng chốt xong, dữ liệu đo tìm xong. **Con số chưa tồn tại** và plan
này không được phép đoán nó.

---

## Deviations from Plan

### 1. [Rule 1 - Bug] `GIT_EXTERNAL_DIFF=""` làm mọi `git diff` sinh bản vá thất bại

**Tìm thấy ở:** Task 1, lần chạy test đầu tiên (`patch` rỗng).

**Đây là lỗi của Phase 1, không phải của spike.** `src-tauri/src/git/exec.rs` đặt:

```rust
cmd.env("GIT_EXTERNAL_DIFF", "");   // ❌
```

Ý định đúng (vô hiệu hoá trình diff ngoài của người dùng), cơ chế sai. Git không hiểu
chuỗi rỗng là "không có trình nào" — nó thấy biến đã đặt rồi spawn chương trình tên rỗng:

```
error: cannot spawn : No such file or directory
fatal: external diff died, stopping at file.txt
```

**Phạm vi:** mọi lệnh diff sinh nội dung bản vá thoát 128 với stdout **rỗng**, im lặng.
Gồm cả `--word-diff=porcelain` mà **plan 03-03 dựa vào**.

**Vì sao Phase 2 không phát hiện:** nó chỉ dùng `--name-status`, vốn không gọi tới
trình diff nên không spawn gì. Đã kiểm: `--name-status` chạy tốt trong khi
`--unified=3` và `--word-diff=porcelain` cùng thoát 128.

**Sửa:** đặt `diff.external=` rỗng có **cùng lỗi** (đã đo, nên bỏ cách đó). Cách đúng
là cờ `--no-ext-diff`, thắng **cả** `diff.external` trong cấu hình **và**
`GIT_EXTERNAL_DIFF` trong môi trường — đã đo cả ba tổ hợp. Cờ chèn tập trung trong
`GitCommand::run()` cho `diff`/`show`/`log`/`diff-tree`, vì người gọi sẽ quên, và
việc người gọi quên chính là lỗi này.

**Commit:** `f5c4c17`

### 2. Cổng `isPerfEnabled()` đặt ở `App.tsx`, không trong `SpikeHarness`

Plan `key_links` mong `from '@/lib/perf'` xuất hiện trong `SpikeHarness.tsx`. Đặt ở
đó thì **cổng không làm được việc của nó**: React vẫn phải nạp mô-đun để biết nó trả
`null`, tức CodeMirror vẫn được tải và ràng buộc bundle của phase bị phá. Cổng nằm ở
`App.tsx` bao quanh `lazy()`. Có test ghim **vị trí** cổng, không chỉ sự tồn tại.

### 3. Baseline test Rust là 154, không phải 149

Brief ghi 149. Đo thật trước khi sửa gì: **154 passed, 1 ignored**. Không phải hồi
quy — chênh lệch do test phụ thuộc fixture. Dùng 154 làm mốc.

---

## Kiểm mutation — 5/5, có hai lần phải sửa cổng

| # | Đột biến | Kết quả |
|---|---|---|
| 1 | Xoá `.arg("--")` khỏi `diff_spike.rs` | ✅ đỏ ngay |
| 2 | Hoàn nguyên bản sửa `GIT_EXTERNAL_DIFF` | ✅ 2 test đỏ |
| 3 | Bỏ cổng `isPerfEnabled()` trong `App.tsx` | ⚠️ **xanh lần đầu** → sửa cổng → ✅ đỏ |
| 4 | Đổi `lazy(import())` thành nhập tĩnh | ✅ đỏ ngay |
| 5 | Thêm gói meta `codemirror` vào `package.json` | ✅ đỏ ngay |

**Mutation 3 là ca đáng kể.** Cổng đầu tiên xanh với cổng đã bị xoá, vì hai lý do
cộng lại: test không mở repo nào (nên `SpikeHarness` nằm trong nhánh không được
render, chặn bởi `activeRepo` chứ không bởi cờ perf), và phép kiểm đồng bộ chạy
**trước** khi `lazy()` kịp nạp component. Sửa cả hai: mở sẵn repo trong store, khẳng
định **tiền đề** (`commit-scroll` phải hiện) trước khi khẳng định điều cần kiểm, và
`await` một `findByTestId` phải `rejects.toThrow()`.

Đây đúng là bài học "cổng grep dễ vô dụng" của Phase 2 ở một dạng khác — không phải
grep, mà một assertion đúng về hình thức nhưng không quan sát được thứ nó tưởng.

---

## Điều bắt buộc ghi lại (plan `<output>`)

1. **Chính tả `--word-diff=porcelain`.** `--word-diff-porcelain` **không tồn tại** —
   thoát khác 0 và in usage. Nay có test ghim **cả hai chiều**: dạng đúng phải chạy
   được và in `-==` / `+===` cho ca `==` → `===`; dạng sai phải thất bại. Plan 03-03
   phụ thuộc điều này, và nay nó còn phụ thuộc bản sửa `--no-ext-diff`: trước bản
   sửa, cờ đúng cũng thoát 128.

2. **Quyết định A/B: ⏳ CHƯA CÓ** — chờ checkpoint. Không có số ước lượng nào được
   điền vào `docs/09-phase3-diff-decision.md`.

3. **Kết quả word-level của `@codemirror/merge`: ⏳ CHƯA CÓ.** Harness đếm bằng mã
   (`.cm-changedText` so với `.cm-changedLine`) thay vì để nhìn bằng mắt, vì "tôi
   thấy nó có tô" không phân biệt được hai lớp đó — và 03-03 phụ thuộc vào việc phân
   biệt đúng.

4. **`SpikeHarness` vẫn còn trong `App.tsx`; `spike_blob_pair` vẫn còn.** Cả hai phải
   còn để chạy được phép đo. Xoá là việc **sau** khi có số — Task 3 của 03-02 quyết
   định giữ `spike_blob_pair` hay không.

---

## Cổng verification

| Cổng | Kết quả |
|---|---|
| `cargo test` | ✅ 163 passed, 0 failed, 1 ignored (mốc 154) |
| `cargo clippy --all-targets` | ✅ 0 warning, 0 error |
| `npm run typecheck` | ✅ sạch |
| `npm test` | ✅ 226 passed (mốc 212) |
| Cổng gói CodeMirror (`node -e`, **không** grep) | ✅ đúng yêu cầu |
| `npm run tauri:build` | ✅ dựng được, MSI xong |

Cổng gói dùng `node -e` đọc JSON rồi tra khoá, **không** `grep -c`: `grep codemirror`
khớp cả `@codemirror/state` nên không phân biệt được gói meta với gói con, và
`grep -c '@codemirror/merge'` cũng khớp một dòng chú thích. Plan nêu rõ điều này và
mục "cổng grep KHÔNG dùng, và vì sao" được tôn trọng.

---

## Known Stubs

Đường B trong `diffSpike.ts` là **bản thô có chủ ý** — nó tô cả dòng, không xử lý đổi
tên, không xử lý `\ No newline at end of file`. Ghi rõ trong mục 8 của tài liệu quyết
định: con số của B là **chi phí sàn**, không phải chi phí của một đường B hoàn chỉnh.
Đây là spike; 03-04 viết bản thật nếu đường B được chọn.

Không có stub nào ngăn plan này đạt mục tiêu: mục tiêu là **sinh ra công cụ đo**, và
công cụ đo chạy được trên dữ liệu thật.

---

## Threat Flags

Không có bề mặt an ninh mới ngoài `<threat_model>` của plan. `spike_blob_pair` là
lệnh chỉ đọc, đi qua `repo_cua()` (T-03-02) và có `--` trước path (T-03-01).

Ghi nhận một điểm **giảm** rủi ro ngoài dự kiến: bản sửa `--no-ext-diff` chặn một
đường thi hành mã mà trước đây người dùng đặt `diff.external` có thể kích hoạt trong
tiến trình con của app.

---

## Self-Check: PASSED

Tệp đã kiểm tồn tại: `diff_spike.rs` (312 dòng), `SpikeHarness.tsx` (223 dòng, mốc
80), `09-phase3-diff-decision.md` (220 dòng, có "Ngưỡng kết luận"), `diffSpike.ts`,
`diffSpike.test.ts`, `SpikeHarness.test.tsx`, `tests/diff_spike.rs`.

Commit đã kiểm có trong `git log`: `8d611b6`, `f5c4c17`, `92c3dac`, `5384708`,
`85e5d2a`, `d2b6dca`.
