# Phase 4 — Xác minh từng requirement

**Ngày:** 2026-09-22
**Trạng thái phase:** **4/5 wave đã thực thi** (04-01..04-04). Wave 5 (04-05) **bị chặn ở
Task 0** — xem mục "Cổng Task 0" bên dưới.

🔴 **Phase 4 CHƯA ĐÓNG.** Hai cổng chặn của wave 5 (checkpoint hiển thị 12 bước, và exit
gate dogfood) **chưa chạy**, và phần nối dây mà chúng kiểm **chưa tồn tại**. Không
requirement nào dưới đây có bằng chứng từ mắt người.

Tài liệu này nêu **bằng chứng**, không nêu ý kiến. Ba loại bằng chứng, theo đúng yêu cầu
của 04-05 Task 5:

- **`test tự động`** — có test xanh, **chưa ai xem bằng mắt**.
- **`mắt người (release)`** — chủ dự án đã xem trên bản release, **kèm dấu thời gian exe**.
- **`chưa kiểm`** — không có bằng chứng nào.

**Quy tắc cứng (CONTEXT.md 3.4):** mọi khẳng định về **bố cục** (chiều cao hàng WIP, thẳng
cột đồ thị/văn bản, ba nhóm tệp không tràn) mà chỉ có test happy-dom phải ghi **`chưa
kiểm`**, **không** phải `Đạt`. happy-dom không tính CSS layout và không có cuộn thật — đó là
lý do **cả năm** lỗi hiển thị của Phase 3 đi qua 435 test tự động và chỉ lộ ra khi chủ dự án
mở ứng dụng.

---

## Lượt thứ hai (2026-09-22, sau `0633e4e`) — cổng Task 0 **VẪN ĐỎ**

Lượt này được giao với tiền đề *"bốn tệp đã commit và sạch ở HEAD `0633e4e`"*. **Tiền đề
đó sai**, và đây là số đo:

```console
$ git show HEAD:src/components/history/CommitList.tsx | grep -c "avatarTuSinh"
0
$ grep -c "avatarTuSinh" src/components/history/CommitList.tsx      # trên đĩa
1
$ git ls-files src/lib/avatar.ts src/components/Avatar.tsx src/components/icons.tsx
(rỗng — cả ba vẫn CHƯA ĐƯỢC THEO DÕI)
```

| Tệp | `git status --porcelain` | Kết luận |
|---|---|---|
| `src/App.tsx` | ` M` | **bẩn** — avatar/Gravatar của session khác |
| `src/components/history/CommitList.tsx` | ` M` | **bẩn** — `avatarTuSinh`, `scrollToCommit`, `maxLane` |
| `src/styles/app.css` | ` M` | **bẩn** — 5 hunk, chỉ hunk cuối thuộc wave 5 |
| `src/components/history/CommitList.test.tsx` | *(sạch)* | ✅ tệp **duy nhất** thật sự được giao lại |

🔴 **Vì sao lượt đầu tưởng là sạch.** Phép kiểm mở màn chạy qua `rtk proxy git status
--porcelain <paths>` và trả **rỗng**. Chạy lại bằng `git` trần trên cùng đường dẫn thì ra
` M`. Đây đúng lớp lỗi mà CONTEXT.md 3.1 gọi là xanh giả: một cổng trả rỗng **không phân
biệt được** với "không có gì để báo". Bài học ghi lại: **mọi cổng mà câu trả lời là
exit-code hoặc một phép đếm phải chạy bằng `git` trần**, không qua rtk.

Thêm một dữ kiện: HEAD **đã chạy trong lúc làm** (`0633e4e` → `b467194` → `69ec3ad`) —
một session khác commit xen vào. Năm session đang sống trên repo này.

### Việc đã làm được, và nó đang nằm ở đâu

Phần nối dây **đã viết xong và xanh**, nhưng **chưa commit được** vì nó nằm xen kẽ với
việc của session khác trong cùng ba tệp. Commit nó là commit hộ phần việc không hiểu ý
định — đúng thứ `<blocking_precondition>` cấm.

| Hạng mục | Trạng thái | Ghi chú |
|---|---|---|
| 9 test RED cho hàng WIP | ✅ **đã commit** `b467194` | tệp test sạch nên commit được |
| `CommitList.tsx` nối hàng WIP | ✅ viết xong, xanh | **chưa commit** — tệp bẩn |
| `wipCountsFromStatus` (`wipRow.ts`) | ✅ viết xong, xanh | **chưa commit** — `wipRow.ts` cũng bẩn |
| `WipEdgeRender` + `draw` tham số thứ 3 | ✅ viết xong, xanh | **chưa commit** |
| `GraphCanvas` truyền `wip` | ✅ viết xong, xanh | **chưa commit** — một mình nó **không biên dịch được** (cần `WipEdgeRender` ở `types.ts`), nên tách ra commit riêng sẽ tạo một commit hỏng |
| Khối CSS 28px hàng WIP | ✅ viết xong | **chưa commit** — `app.css` bẩn |
| `App.tsx`: mount watcher + render `ChangeList`/`CommitBox` | ❌ **chưa làm** | `App.tsx` bẩn từ đầu; không đụng vào |

Bản sao của sáu tệp đang làm dở nằm ở `%TEMP%\gsd-04-05-wip\` để không mất khi ai đó
`git checkout`.

### Số đo lượt này

| Phép đo | Kết quả | Lệnh |
|---|---|---|
| Frontend vitest, **trước** | **645 passed, 0 failed** (227 suite) | `rm -f .vitest/json/output.json && npx vitest run --reporter=json --outputFile=…` |
| Frontend vitest, **sau** | **658 passed, 0 failed** (228 suite) | cùng lệnh; +13 test của hàng WIP |
| `npx tsc --noEmit` | **sạch** | — |
| `cargo test --lib` | **309 passed, 0 failed** | `cd src-tauri && cargo test --lib` |
| `cargo test --lib --tests` | 🔴 **KHÔNG ĐO ĐƯỢC** | `git-plum.exe` (PID 12580) đang chạy → `Access is denied. (os error 5)` lúc cargo relink. **Không suy ra con số 425.** |
| Cổng `useVirtualizer` | `OK: dung 1 loi goi` | vẫn đúng 1 sau khi sửa |
| Cổng số học `wipRow.test.ts` | xanh | `CommitList.tsx` gọi `commitRowY(...)`, không tự cộng |

### Bảng đột biến (chứng minh cổng không rỗng)

| # | Đột biến | Kỳ vọng | Kết quả thật |
|---|---|---|---|
| M1 | `count: virtualizerCount(total, hasWip)` → `count: total + 1` (cách A qua cửa sau) | đỏ | ✅ **1 đỏ**: *"count của virtualizer === total KỂ CẢ khi có hàng WIP (cách B)"*. Suite vẫn nạp (28 total) nên đỏ là do hành vi, không do import hỏng. |
| RED | chạy 9 test trước khi viết mã | đỏ | ✅ **9 đỏ / 28 total** — suite **nạp được**, nên đây là thiếu hành vi chứ không phải xanh giả #8 |

M2 (`marginTop: contentOffset(hasWip)` → `0`) **không thu được kết quả dùng được**: một
`git checkout --` của chính tôi đã hoàn nguyên tệp trước khi đột biến kịp áp, nên 14 đỏ
quan sát được là của tệp **chưa sửa**, không phải của đột biến. Ghi ra đây thay vì im
lặng bỏ — *"một đột biến không diễn đạt được khiếm khuyết thì không phải bằng chứng về
cổng"*, và một đột biến **không áp được** càng không phải.

---

## Cổng Task 0 của 04-05 — **ĐỎ**, và đây là lý do wave 5 dừng (lượt đầu, giữ nguyên)

04-05 mở đầu bằng một `checkpoint:human-action` chặn: bốn tệp mà wave 5 **buộc phải sửa**
phải sạch trước khi sửa.

```bash
git status --porcelain=v2 -z --untracked-files=all | tr '\0' '\n' \
  | grep -E "src/App.tsx|CommitList|app.css"
```

Kết quả đo lúc 2026-09-22 18:30 — **cả bốn tệp còn `.M`**:

| Tệp | Trạng thái | Khối lượng thay đổi chưa commit | Nội dung (KHÔNG thuộc Phase 4 wave 5) |
|---|---|---|---|
| `src/App.tsx` | `.M` | +93 −5 | Nút icon ở toolbar, công tắc Gravatar, `handleSelectRef` |
| `src/components/history/CommitList.tsx` | `.M` | +116 −14 | `scrollToCommit`, `indexById`, avatar tác giả, màu nhánh, sửa `maxLane` |
| `src/components/history/CommitList.test.tsx` | `.M` | +126 −1 | Test cho `scrollToCommit` và bề rộng cột đồ thị |
| `src/styles/app.css` | `.M` | +580 −3 | Trộn phần xoá của session khác với phần thêm của các wave trước |

Đây **không** phải rác cũ còn sót. Đó là việc **đang làm, có chủ đích, chưa commit** của một
session khác (mtime 16:43–16:44 cùng ngày), và nó phụ thuộc vào ba tệp **chưa được theo
dõi**: `src/components/icons.tsx`, `src/lib/avatar.ts`, `src/components/Avatar.tsx`.

Plan nói thẳng cách xử lý, và tôi đã theo:

> **Còn `.M`** → **DỪNG. Báo chủ dự án.** Không merge, không stash, không commit thay session
> khác. Hai session sửa cùng tệp là cách mất việc của một trong hai.

`CommitList.tsx` là **đúng** tệp mà WORK-11 phải sửa; không có đường tránh. Sửa nó bây giờ là
hoặc ghi đè việc của session kia, hoặc commit hộ họ phần việc tôi không hiểu ý định.

---

## Số đo nền, tại `80222e6` (đo được, không suy ra)

| Phép đo | Kết quả | Lệnh |
|---|---|---|
| Rust `cargo test --lib` | **309 passed, 0 failed** | `cd src-tauri && cargo test --lib` |
| Frontend vitest | **645 passed, 0 failed** (227 suite) | `rm -f .vitest/json/output.json && npx vitest run --reporter=json --outputFile=…` |
| `npx tsc --noEmit` | **sạch** | — |

`cargo test --lib --tests` **không đo lại trong lượt này** — số 425 passed / 1 ignored là do
chủ dự án cung cấp, ghi lại ở đây như dữ liệu nhận được chứ không phải tôi đo.

---

## Requirement

| Req | Nội dung | Trạng thái | **Loại bằng chứng** |
|---|---|---|---|
| **WORK-01** | Ba nhóm tệp; chọn tệp → thấy diff của nó | Có mã (04-01, 04-02), **chưa nối vào `App.tsx`** | `chưa kiểm` — không có đường nào từ giao diện tới `ChangeList`; và bố cục ba nhóm là khẳng định layout |
| **WORK-02** | Stage / unstage theo tệp | Có mã + test (04-01, 04-02) | `test tự động` cho logic store; `chưa kiểm` cho thao tác thật (chưa render trong ứng dụng) |
| **WORK-08** | Soạn thông điệp và tạo commit | Có mã + test (04-03) | `test tự động`; `chưa kiểm` — `CommitBox` chưa được `App.tsx` render, chưa ai gõ một thông điệp thật |
| **WORK-09** | Amend, cảnh báo nếu đã push nhưng không chặn | Có mã + test (04-03) | `test tự động`; `chưa kiểm` — chưa ai bấm amend trên repo thật |
| **WORK-10** | Thay đổi ngoài ứng dụng → tự cập nhật < 1 s | Có mã + 8 test (04-04) | `test tự động`; 🔴 **`noiWatcherVaoStore()` chưa được gọi ở đâu cả** — watcher tồn tại nhưng **không chạy trong ứng dụng**. Tiêu chí "< 1 giây, không bấm gì" **chưa kiểm** |
| **WORK-11** | Hàng WIP đầu đồ thị, số đếm, bấm mở vùng soạn | Mã nối dây **đã viết, 13 test xanh**, nhưng **CHƯA COMMIT được** (tệp bẩn — xem lượt hai) | `test tự động` cho **số học** và cho DOM ở happy-dom; 🔴 `chưa kiểm` cho **mọi** khẳng định hiển thị. Và vì mã chưa vào nhánh, ngay cả `test tự động` cũng **chưa tái lập được từ một lần clone sạch** |

**Không requirement nào đạt mức `mắt người (release)`.** Đó là điều kiện để đóng phase và nó
chưa được thoả.

---

## Bảy tiêu chí thành công

| # | Tiêu chí | Trạng thái | **Loại bằng chứng** |
|---|---|---|---|
| 1 | Ba nhóm tệp hiện đúng, đúng tệp trong từng nhóm | Chưa nối vào giao diện | `chưa kiểm` (bố cục — happy-dom không tính được) |
| 2 | Stage/unstage đổi nhóm **ngay**, không phải bấm làm mới | Logic có test | `test tự động` cho store; `chưa kiểm` cho phản hồi thật |
| 3 | Chọn tệp → diff của **đúng** tệp đó | Dựa trên trình xem diff Phase 3 **chưa ai dùng thật** | `chưa kiểm` |
| 4 | Commit xong → đồ thị cập nhật ngay, ô soạn rỗng lại | Có mã (04-03) | `chưa kiểm` |
| 5 | Thay đổi từ terminal → giao diện tự cập nhật < 1 s | Watcher có mã, **chưa mount** | `chưa kiểm` — chỉ kiểm được bằng cách chạy ứng dụng thật |
| 6 | Nháp thông điệp còn sau khi đóng/mở lại và khi chuyển repo | Có mã + test (04-03) | `test tự động`; `chưa kiểm` cho vòng đời đóng/mở ứng dụng thật |
| 7 | Hết thay đổi → hàng WIP **biến mất** | `hasWipRow({0,0}) === false` có test | `test tự động` cho hàm thuần; `chưa kiểm` cho thứ nhìn thấy trên màn hình |

🔴 **Tiêu chí 5 và tiêu chí "thẳng cột" của hàng WIP không thể kiểm bằng test** — happy-dom
không có cuộn thật. Lớp lỗi "cột đồ thị lệch cột văn bản đúng một hàng" đã xảy ra **hai lần**
ở Phase 2; chỉ mắt người trên bản release phân biệt được.

---

## Bản dựng release — **KHÔNG hợp lệ để kiểm**

```text
-rwxr-xr-x 2 tuyen 197609 4619264 2026-09-22 11:53:54.612877500 +0700
  src-tauri/target/release/git-plum.exe
```

🔴 Exe này **cũ hơn HEAD 6 giờ 28 phút** (HEAD `80222e6` lúc 18:22:22). Nó ra đời **trước
toàn bộ wave 4** (`c7ed8c0` 17:53 → `80222e6` 18:22), nên nó **không chứa** `wipRow.ts` lẫn
watcher.

Đây đúng là lỗi §3.7 mà CONTEXT.md ghi lại (chủ dự án từng kiểm một exe cũ hơn bản sửa 32
phút). **Không được dùng exe này cho checkpoint nào.** Tôi **không** dựng bản mới, vì phần
nối dây mà checkpoint cần kiểm chưa tồn tại — một bản dựng bây giờ chỉ tạo ra một exe mới
tinh **vẫn không có** hàng WIP trên màn hình, tức đúng loại bằng chứng giả mà tài liệu này
tồn tại để ngăn.

---

## Nợ kiểm chứng mang vào từ Phase 1–3 (CONTEXT.md mục 0) — vẫn nguyên

Ghi lại để không mất dấu; **không** mục nào được giải quyết trong phase này.

| Phase | Cổng người-kiểm | Trạng thái |
|---|---|---|
| 1 | 3 tiêu chí QA Windows (`docs/03-phase1-qa-windows.md`) | **chưa kiểm** |
| 2 | checkpoint 12 bước (02-06) | **vòng 1 bị từ chối**, đã sửa, **chờ vòng 2** |
| 2 | checkpoint #1 phần người thấy (02-07) | **thiếu** (phần Rust đã đo: 787 ms/1000 ms) |
| 3 | checkpoint 11 bước (03-04) | **chưa chạy** |
| 3 | **exit gate dogfood** (03-05) | **chưa chạy** |
| **4** | **checkpoint hiển thị 12 bước (04-05 Task 3)** | **chưa chạy — bị chặn ở Task 0** |
| **4** | **exit gate dogfood (04-05 Task 4)** | **chưa chạy — bị chặn ở Task 0** |

Phase 4 là **phase thứ tư liên tiếp** đứng trước nguy cơ đóng với nợ kiểm chứng. Khuôn hình
đó được ghi ra ở đây thay vì để nó tích lại im lặng.

---

## Việc còn lại của wave 5 (mở lại được ngay khi bốn tệp được giao lại)

Bốn wave đầu **đã xong và có test**. Phần còn thiếu là **nối dây**, và nó nhỏ:

1. `App.tsx`: một `useEffect` gọi `noiWatcherVaoStore()` theo vòng đời repo; render
   `ChangeList` + `CommitBox`; `statusStore.refresh` khi mở repo; `commitStore.hydrate`.
2. `CommitList.tsx`: `count: virtualizerCount(total, hasWip)`, hàng WIP là phần tử **anh em**
   ghim ngoài virtualizer (**cách B** — giữ nguyên, **không** đổi sang `count: total + 1`),
   `commitRowY(v.start, scrollTop, hasWip)` thay cho `v.start - scrollTop`.
3. `GraphCanvas`/`canvasRenderer`: vẽ cạnh WIP qua tham số của `draw`, giữ interface
   `GraphRenderer`.
4. CSS hàng WIP cao **đúng** `ROW_HEIGHT` (28px).
5. Dựng release, dán dấu thời gian, chạy checkpoint 12 bước rồi exit gate.

**Kết quả đúng khi cổng không đóng được là một plan vá khoảng cách, không phải đi tiếp
Phase 5** (ROADMAP + CONTEXT.md mục 5).
