# Phase 5 — Verification

**Ngày:** 2026-09-23
**Bản dựng release:** `2026-09-23 12:04:50`, commit `ee71d3b`

---

## 🔴 Quy ước của tệp này

`CONTEXT.md` mục 0 ghi lại rằng `VERIFICATION.md` của **Phase 4** dùng chữ *"mắt người
(release)"* **ba lần** mà **không cấp nó cho thứ gì** — một lần ở danh sách định nghĩa,
hai lần ở các câu **phủ định**. Tệp này không lặp lại khuôn đó.

Mỗi dòng dưới đây mang **đúng một** trong bốn trạng thái:

| Trạng thái | Nghĩa |
|---|---|
| **MÁY kiểm** | có một lệnh chạy lại được, và số đo được dán trong tệp này |
| **có mã, chưa kiểm** | mã tồn tại và có test happy-dom, nhưng điều được khẳng định **không quan sát được** bằng test đó |
| **chưa chạy** | cổng tồn tại, chưa ai chạy |
| **nợ** | chưa có mã |

**Không dòng nào ghi "Đạt" cho một thứ chưa đo.**

---

## 1. Số đo — lệnh đầy đủ, chạy hôm nay

| Phép đo | Trước wave 5 | Sau wave 5 | Lệnh |
|---|---:|---:|---|
| Frontend | **703 passed, 0 failed** | **731 passed, 0 failed** (+28) | `npx vitest run --reporter=json --outputFile=.vitest/json/final05.json` |
| Bộ test frontend | 242 | **257** | cùng lệnh |
| `npx tsc --noEmit` | sạch | **sạch** (`exit=0`) | `npx tsc --noEmit` |
| `cargo test --lib` | 350 | **350 passed, 0 failed** | `cargo test --lib` |
| `cargo test --lib --tests` | 481 passed, 1 ignored | **481 passed, 1 ignored, 0 failed** | `cargo test --lib --tests` |
| `npm run build` | 645 kB cảnh báo | **✓ built in 403ms**, 656 kB | `npm run build` |
| `npm run tauri build` | — | **✓ 2m 47s**, exe + msi | `npm run tauri build` |

Con số frontend đọc **sau** khi lệnh xong, từ tệp `--outputFile` **riêng** cho từng lần
chạy, `rm -f` trước mỗi lần — `rtk` ghi đè `.vitest/json/output.json` sau lệnh của chính
mình (`CONTEXT.md` mục 5).

Mã thoát `tsc` đọc từ một lời gọi **riêng, không qua đường ống**: `$?` sau một pipeline
đọc mã của lệnh **cuối**.

> ⚠️ **Cảnh báo `npm run build` CÓ TRƯỚC.** `index-*.js` vượt ngưỡng 500 kB. Wave 4 đo
> 645,53 kB; hôm nay 656,66 kB — phần wave này thêm là ~11 kB. **Không sửa**; ghi lại
> để không ai tưởng nó mới.

### 1.1 Rust — `cargo test`, chạy hôm nay sau bản dựng release

```text
cargo test --lib          → running 350 tests
                            test result: ok. 350 passed; 0 failed; 0 ignored
cargo test --lib --tests  → 481 passed, 1 ignored (19 suites, 33.11s), exit 0
```

**Không đổi so với mốc trước wave 5, và đó là điều đúng:** wave này không sửa một dòng
`src-tauri/**` nào (mọi thay đổi nằm ở `src/`, `scripts/`, `docs/`, `.planning/`).

`--lib --tests` chạy được vì **không có `git-plum.exe` nào đang chạy** — kiểm bằng
`tasklist` trước khi gọi. `CONTEXT.md` mục 5: một app đang mở khoá lệnh này với
`os error 5` vì cargo relink binary.

---

## 2. Bốn tiêu chí thành công của Phase 5

### ✅ Tiêu chí 2 (WORK-04) — **MÁY kiểm**, đóng được không cần mắt người

> *"Sau staging một phần: CRLF giữ nguyên, thiếu dòng cuối giữ nguyên, byte không UTF-8
> không bị thay."*

```bash
bash scripts/verify-work04-bytes.sh; echo "[exit=$?]"
```

Đầu ra thật, chạy trên `git 2.54.0.windows.1` + `node v22.16.0`:

```text
LƯỢT 1: CRLF thật  (core.autocrlf=true, 0xE9 dòng 6, thiếu dòng cuối)
  [số khối trong diff đầy đủ = 3]   (cần >= 3)
  [git apply --cached --recount exit = 0]
  [vùng chờ: '+MOD 16' = 1 (cần 1)   '+MOD 03'/'+MOD 29' = 0 (cần 0)]
  [\r  trước = 29   sau = 29]
  [0xE9 trước = CO   sau = CO]
  [byte cuối là \n?  trước = KHONG   sau = KHONG]

  ✅ (0) chỉ KHỐI GIỮA vào vùng chờ: +MOD 16 = 1, hai khối kia = 0
  ✅ (1) CRLF giữ nguyên:      \r 29 → 29
  ✅ (2) vẫn thiếu dòng cuối:  byte cuối là \n? KHONG → KHONG
  ✅ (3) byte 0xE9 giữ nguyên: CO → CO

LƯỢT 2: đối chứng LF thuần
  [\r  trước = 0   sau = 0]
  ✅ PHÂN BIỆT ĐƯỢC — lượt 1 \r=29, lượt 2 \r=0 (khác nhau).

    [lượt 1: CRLF thật    ] \r trước=29   sau=29   → ĐẠT
    [lượt 2: đối chứng LF ] \r trước=0    sau=0    → KHÁC lượt 1 (đúng)
✅ WORK-04 / tiêu chí thành công 2 — ĐẠT, do MÁY kiểm.
[exit=0]
```

**Cổng này đã được chứng minh phân biệt được** — hai đột biến, đo thật:

| # | Đột biến | Kết quả |
|---|---|---|
| MG1 | `--cached --recount` → `--recount --index` | 🔴 **ĐỎ** — apply exit 1 `does not match index`; (1)(2)(3) vẫn ✅ |
| MG2 | bản vá chứa **cả ba** khối | 🔴 đỏ nhưng **VÔ HIỆU** — `patch does not apply`, tức nó đo lỗi cú pháp chứ không đo "stage cả tệp" |
| **MG2b** | `git add -- muctieu.txt` thay vì áp bản vá khối giữa | 🔴 **ĐỎ** — apply **exit 0**, `\r 29→29` ✅, `0xE9` ✅, dòng cuối ✅, **chỉ (0) đỏ** |

🔴 **MG2b là phép đo quan trọng nhất của Task 1, và phép kiểm (0) KHÔNG có trong plan.**
Stage cả tệp giữ nguyên **hoàn hảo** mọi byte trong thư mục làm việc (nhờ `--cached`),
nên ba phép kiểm byte mà plan yêu cầu đều xanh trên một cài đặt **không hề staging một
phần**. Thiếu (0), cổng này là một **cổng xanh giả** — đúng lớp lỗi "hình dạng đúng, dữ
liệu vô hại" của `CONTEXT.md` 4.2.

MG2 được ghi lại **nguyên trạng** dù nó vô hiệu: `CONTEXT.md` 4.1 nói *"đột biến không
biểu diễn được lỗi thì không nói gì về cổng"*, và MG2 → MG2b là lần thứ **ba** dự án
phải thu hẹp một đột biến (M14→M14c wave 3, M25→M25b wave 4).

---

### ⏸ Tiêu chí 1 (WORK-03) — **có mã, chưa kiểm**

> *"Chọn một khối trong tệp nhiều thay đổi → commit chứa **đúng** khối đó."*

| Phần | Trạng thái | Bằng chứng |
|---|---|---|
| Tầng Rust sinh & áp bản vá đúng | **MÁY kiểm** | 05-02, `tests/hunk_commands.rs`; cộng `verify-work04-bytes.sh` phép kiểm (0) hôm nay |
| Giao diện truyền **chỉ số khối thật** | **MÁY kiểm** | đột biến **M29** đỏ (1 test), M23 của wave 4 đỏ (4 test) |
| Người dùng **chọn đúng khối mình muốn** | 🔴 **chưa chạy** | cổng mắt người, `docs/10-phase5-dogfood.md` bước 1–4 |

---

### ⏸ Tiêu chí 3 (WORK-05) — **có mã, chưa kiểm**

> *"Tệp đổi ở nơi khác → báo và đòi làm mới, không áp bản vá không khớp."*

| Phần | Trạng thái | Bằng chứng |
|---|---|---|
| Rust từ chối và ném `file_changed` | **MÁY kiểm** | 05-02 |
| Băng mang **nguyên văn** "hãy làm mới", `role="alert"` | **MÁY kiểm** | M24, M26, M26b của wave 4 |
| Nút "Làm mới" **thật sự nạp lại** diff | **MÁY kiểm** | test mới của wave 5 (lời gọi `getWorktreeDiff` **thứ hai**) |
| Câu đó **đọc là hiểu ngay** | 🔴 **chưa chạy** | bước 5 của dogfood |

🔴 **Nợ mới của wave 5:** `blobHash` truyền xuống từ `HunkTable` là `''` — `getWorktreeDiff`
không trả mã băm blob. Hệ quả: giao diện **mất lớp cảnh báo sớm**. Lớp bảo vệ **không**
mất (Rust vẫn tự đọc lại hash và `git apply --check` vẫn từ chối), nhưng WORK-05 chỉ còn
**một** lớp thay vì hai. Xem `05-05-SUMMARY.md`.

---

### ⏸ Tiêu chí 4 (WORK-06 / WORK-07) — **có mã, chưa kiểm**, và có khoảng trống đã biết

> *"Huỷ theo tệp và theo khối, rồi khôi phục lại được từ danh sách."*

| Phần | Trạng thái | Bằng chứng |
|---|---|---|
| Mọi đường huỷ tự lưu trước (`BienNhan`) | **MÁY kiểm** | 05-03, T-05-13 |
| Động từ "Xoá" cho tệp chưa theo dõi | **MÁY kiểm** | M21+M22 wave 4 (hai chiều), **M31** wave 5 (chỗ truyền) |
| `TrashList` sắp mới nhất trước | **MÁY kiểm** | **M26** đỏ |
| Nút khôi phục dùng đúng `refName` | **MÁY kiểm** | **M27** đỏ |
| Thời điểm huỷ hiện đúng | **MÁY kiểm** | test `data-ms`; đọc từ `refName`, không từ `luc` |
| **Tên tệp của mục vừa huỷ** | 🔴 **nợ** | `for-each-ref` không giữ `paths`/`nhan`; danh sách **nói ra** điều đó |
| `restore_trash` tự lưu trước khi ghi đè | 🔴 **nợ (T-05-11)** | `khoi_phuc` ghi đè thẳng — 05-03, chưa cài |
| Người dùng **tìm thấy** danh sách | 🔴 **chưa chạy** | bước 8 của dogfood |

---

## 3. Bảng đột biến wave 5 — chạy thật, kết quả đo được

| # | Đột biến | Đỏ mong đợi | **Đo được** |
|---|---|---|---|
| MG1 | cổng byte: bỏ `--cached` | phép kiểm (0) | ✅ **ĐỎ** |
| MG2 | cổng byte: bản vá cả ba khối | phép kiểm (0) | ⚠️ đỏ nhưng **VÔ HIỆU** |
| MG2b | cổng byte: `git add` cả tệp | phép kiểm (0) | ✅ **ĐỎ**, và **chỉ** (0) đỏ |
| M26 | `TrashList` bỏ phép sắp xếp | Test 3 | ✅ **1 đỏ** |
| M27 | nút khôi phục luôn dùng `daSap[0]` | Test 2 | ✅ **1 đỏ** |
| M28 | bảng khối đọc store `DiffViewer`, **`fieldset disabled`** | Test 4 | ⚠️ **0 đỏ — đột biến VÔ HIỆU**, xem dưới |
| **M28b** | bảng khối đọc store `DiffViewer`, **không render `HunkBar`** | Test 4 | ✅ **6 đỏ** |
| M29 | `HunkBar` nhận `0` thay vì `index` | test nối dây | ✅ **1 đỏ** |
| M30 | bỏ cổng chống đua của `HunkTable` | (không dự đoán) | ⚠️ **0 đỏ → ĐỎ sau khi viết cổng** |
| M31 | `WorktreePane` truyền `untracked={false}` cứng | (không dự đoán) | ✅ **1 đỏ** |

### 🔴 M28 vô hiệu, M28b là phép đo thật

M28 bản đầu bọc `HunkBar` trong `<fieldset disabled>`. **0 đỏ trên 27 test.** Trước khi
kết luận bất cứ điều gì, tôi viết một test **thăm dò**:

```text
[total=2 failed=2]
  FAILED : button.disabled trong fieldset disabled === true?
  FAILED : click bị chặn?
```

**happy-dom KHÔNG lan `disabled` từ `fieldset` xuống `button`** — cả thuộc tính lẫn việc
chặn click. Nên đột biến đó **không biểu diễn được khuyết tật nào**, và 0 đỏ của nó nói
về **happy-dom**, không về cổng. `CONTEXT.md` 4.1: *"đột biến không biểu diễn được lỗi
thì không nói gì về cổng."*

M28b dùng `{tepPhase3 !== null && <HunkBar …>}` — đúng hình dạng vòng khoá chết:

```text
[total=27 failed=6 suitesFailed=7]
  ĐỎ: 🔴 M28: render MỘT MÌNH, statusStore rỗng, không component Phase 3/4 nào
      → nút khối vẫn bấm được và vẫn gọi IPC
  ĐỎ: 🔴 M29: mỗi khối truyền chỉ số THẬT của nó xuống stage_hunk
  ĐỎ: bấm "Làm mới" trên băng file_changed nạp lại diff THẬT (lời gọi thứ hai)
  ĐỎ: 🔴 vòng-3: chọn tệp → chọn khối → stage_hunk, KHÔNG có ChangeList/CommitBox/
      hàng WIP nào trên đường
  ĐỎ: chọn tệp CHƯA THEO DÕI → nút huỷ dùng động từ "Xoá", không "Huỷ bỏ"
  ĐỎ: chọn tệp ĐÃ THEO DÕI → nút huỷ dùng động từ "Huỷ bỏ", không "Xoá"
```

🔴 Test **`vòng-3`** đỏ là điều đáng giá nhất trong bảng: nó đo **đường đi**, không đo
component. Nó là bằng chứng rằng mệnh đề hai của wave 4 (*"người dùng **có** đường tới
nó không qua Phase 4"*) nay **được ghim bằng test**, không chỉ được hứa.

### 🔴 M30 — lỗi #9 tái hiện sống, wave thứ TƯ liên tiếp

M30 (gỡ **cả hai** phép kiểm `id !== lanNap.current`) cho **0 đỏ trên 27 test**.

Theo quy tắc, grep toàn bộ `src/**/*.test.*` bằng **công cụ Grep** cho
`lanNap|requestId|chống đua`:

```text
DiffViewer.test.tsx:167   cổng chống đua (T-03-31)
FileHistory.test.tsx:252  cổng chống đua khi đổi tệp (T-03-37)
CommitBox.test.tsx:84     cảnh báo về test chống đua đo quá sớm
HunkTable                 — KHÔNG KHỚP NÀO
```

**Rỗng nghĩa là cổng thiếu, không phải mã đúng.** Viết test, chạy lại:

```text
[total=28 failed=1]
  ĐỎ: 🔴 M30: phản hồi của lần nạp CŨ về SAU phải bị BỎ, không ghi đè bảng đang hiện
AssertionError: bảng phải giữ kết quả của lần nạp MỚI NHẤT. […]
  expected [ …(1) ] to have a length of 2 but got 1
```

**M30: 0 đỏ → ĐỎ.** Cùng khuôn M11 (wave 2), M18 (wave 3), M26 (wave 4) — **bốn wave
liên tiếp**, mỗi wave tìm được đúng một ca lỗi #9 bằng cùng một phép kiểm. Và là wave
thứ **ba** liên tiếp ca đó **không có** trong bảng đột biến do plan viết.

**Vì sao thuộc tính này đáng có cổng:** trên `DiffViewer` một phản hồi về sai thứ tự làm
người dùng **đọc** nhầm tệp. Ở đây nó làm họ **ghi** nhầm tệp — và với `--recount` bản vá
**có thể áp thành công** vào sai tệp. Rủi ro R1 ở dạng tệ nhất: không lỗi, không cảnh
báo, tệp hỏng.

---

## 4. Hai cổng người kiểm — 🔴 CHƯA CHẠY

| Cổng | Trạng thái | Nơi ghi |
|---|---|---|
| Task 3 — mười bước trên release | 🔴 **CHƯA CHẠY** | `docs/10-phase5-dogfood.md` |
| Task 4 — exit gate, một commit thật | 🔴 **CHƯA CHẠY** | `docs/10-phase5-dogfood.md` |

Chủ dự án chọn **"dừng và báo"** cho cổng bắt buộc. Executor **không** phê duyệt chúng,
và **không** ghi "Đạt" cho bất kỳ bước nào.

Bản dựng release cho hai cổng đã sẵn:
`src-tauri/target/release/git-plum.exe`, **2026-09-23 12:04:50**, commit `ee71d3b`.
Đây là bản release **đầu tiên** chứa bất kỳ mã Phase 5 nào — bản cũ là
`2026-09-22 21:18:40` / `9fef159`, có **trước** cả wave 1.

---

## 5. Mọi khẳng định về bố cục: **có mã, chưa kiểm**

happy-dom **không tính CSS layout và không có cuộn thật** (`CONTEXT.md` 4.5). **Không một
test nào** trong 731 test quan sát được những điều sau:

- khối đang chọn **nhìn ra khác** khối không chọn;
- thanh nút không **đè** lên nội dung khối;
- hai cột (danh sách tệp | bảng khối) không đè nhau ở panel hẹp;
- `TrashList` không tràn hoặc cắt chữ khi có nhiều mục;
- bảng khối cuộn được khi tệp có hàng chục khối;
- băng "hãy làm mới" không che dòng ngay dưới.

Chúng nằm trong `app.css` kèm chú thích nói đúng như vậy. Đây không phải từ chối lấy lệ:
lỗi `marginTop`→`paddingTop` của Phase 4 đi qua **cả 666 test**, và **năm** lỗi hiển thị
Phase 3 thoát toàn bộ test tự động.

---

## 6. Số đo Rust

Xem mục 1.1 — đo hôm nay, `350` / `481 passed, 1 ignored`, cả hai **không đổi**.

---

## 7. 🔴 Nợ kiểm chứng của Phase 4 — nêu rõ, không tự quyết định

`CONTEXT.md` mục 7: Phase 4 còn **hai cổng đỏ** và `docs/09-phase4-dogfood.md` **chưa tồn
tại**. Đóng Phase 5 mà Phase 4 chưa đóng nghĩa là **bốn phase liên tiếp** đóng với nợ
kiểm chứng.

Hệ quả cụ thể cho hai cổng của Phase 5: bước 4 và exit gate **đều** đi qua `CommitBox`,
mã Phase 4 chưa ai bấm thử. Nếu chúng hỏng, hai cổng Phase 5 đỏ vì khuyết tật **của
Phase 4** — phải ghi đúng như vậy, không tính vào Phase 5.

Đây là câu hỏi cho **chủ dự án**, không phải quyết định của executor.
