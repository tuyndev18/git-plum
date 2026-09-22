# Phase 4 — Xác minh từng requirement

**Ngày:** 2026-09-22
**Trạng thái phase:** **5/5 wave đã thực thi** (04-01..04-05). Phần **mã** của wave 5 đã
xong và đã vào nhánh (`25b0afe`, `6be8743`, `2d7b3e4`) — xem "Lượt thứ ba" ngay dưới.

🔴 **Phase 4 VẪN CHƯA ĐÓNG.** Mã xong **không phải** điều kiện đóng phase. Hai cổng chặn
của wave 5 — checkpoint hiển thị 12 bước và exit gate dogfood — **chưa chạy**, và chúng
**không chạy được** vì bản release hiện có (11:53:54) cũ hơn mã khoảng 9 giờ và không dựng
lại được khi `git-plum.exe` đang chạy.

**Không requirement nào dưới đây có bằng chứng từ mắt người.** Trạng thái đúng của phần
hiển thị là **"có mã, chưa kiểm"** — không phải "Đạt".

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

## Lượt thứ ba (2026-09-22, tại `2d7b3e4`) — cổng Task 0 **XANH**, nối dây **xong**, hai cổng người-kiểm **vẫn chưa chạy**

Cổng Task 0 chạy lại bằng `git` trần trên bốn tệp, in **nguyên văn** và bọc trong `[...]`
để phân biệt rỗng với trắng (bài học lượt hai):

```console
$ echo "[$(git status --porcelain src/App.tsx src/components/history/CommitList.tsx \
    src/components/history/CommitList.test.tsx src/styles/app.css)]"
[]          # chạy 3 lần: porcelain, porcelain=v2, và `git diff HEAD --stat` — cả ba rỗng
```

**Sạch.** Bốn tệp đã được chủ dự án giao lại qua `25b0afe` và `6be8743`.

### Phát hiện: phần lớn wave 5 **đã nằm trong nhánh** rồi

Lượt hai ghi "viết xong, chưa commit được". Kiểm bằng nội dung tại HEAD chứ không bằng
trí nhớ:

```console
$ echo "[$(git show HEAD:src/lib/graph-render/wipRow.ts | grep -c wipCountsFromStatus)]"
[1]
$ for c in 25b0afe 6be8743 b467194 40c939c; do
    echo -n "$c: ["; git show $c:src/components/history/CommitList.tsx | grep -c wipRow; echo "]"
  done
25b0afe: [3]   6be8743: [3]   b467194: [0]   40c939c: [0]
```

| Hạng mục | Nằm ở đâu | Ghi chú |
|---|---|---|
| 9 test RED hàng WIP | `b467194` | ✅ |
| `wipRow.ts` (`wipCountsFromStatus`, `commitRowY`, `virtualizerCount`) | `6be8743` | ✅ |
| `WipEdgeRender` + `draw` tham số thứ 3 + `GraphCanvas` truyền `wip` | `6be8743` | ✅ |
| **`CommitList.tsx` nối hàng WIP** (`virtualizerCount`, `commitRowY`, **`marginTop`**, anh em ghim) | `25b0afe` | ✅ — **đã vào**, khác với điều lượt hai ghi |
| Khối CSS 28px hàng WIP | `25b0afe` | ✅ |
| **`App.tsx`: watcher + `ChangeList` + `CommitBox`** | **`2d7b3e4`** (lượt này) | ✅ |
| `.detail-header` CSS | `2d7b3e4` | ✅ |

⚠️ Bản sao ở `%TEMP%\gsd-04-05-wip\` **giống hệt** bản trên đĩa/HEAD (`diff` rỗng cho cả 6
tệp) — nó là bản **trước** khi sửa, không phải bản đang làm dở. Không dùng nó làm nguồn.

### 🔴 Lỗi khoá vòng tìm thấy khi nối dây (quy tắc 1 — tự sửa)

Bản nối dây đầu để `ChangeList` tự gọi `statusStore.refresh` lúc mount, với lý lẽ "gọi
thêm ở `App` là một tiến trình `git status` thừa". Ba test mới đỏ ngay, và lý lẽ đó **sai**:

- `ChangeList` là chỗ **duy nhất** trong toàn bộ `src/` gọi `refresh` (đã đếm).
- Nó chỉ được mount khi vùng soạn **đang mở**.
- Đường duy nhất mở vùng soạn là **bấm hàng WIP**.
- Hàng WIP chỉ hiện khi `statusStore` đã có `RepoStatus`.

Bốn điều đó khoá vòng: mở một repo có thay đổi chưa commit thì hàng WIP **không bao giờ**
xuất hiện. WORK-11 đòi thấy hàng WIP **ngay khi mở repo**, nên chủ sở hữu phép nạp đầu là
vòng đời **repo** (`App`), không phải một component có thể chưa tồn tại. `App` nay gọi
`refresh` theo `activeRepoId`; `statusStore` gộp lời gọi trùng theo `dangBay` nên phép gọi
thứ hai của `ChangeList` không sinh tiến trình thừa.

### Số đo lượt này (đo được, không suy ra)

| Phép đo | Trước | Sau | Lệnh |
|---|---|---|---|
| Frontend vitest | **658 passed / 0 failed** | **668 passed / 0 failed** | `rm -f .vitest/json/output.json && npx vitest run --reporter=json --outputFile=.vitest/json/output.json` |
| Chạy 3 lần liên tiếp | — | **668 / 668 / 668**, 0 đỏ | mỗi lần `rm -f` trước, và **mỗi lần khẳng định `numTotalTests >= 660` trước khi tin `numFailedTests`** |
| `npx tsc --noEmit` | sạch | **sạch** | — |
| `npm run build` | — | **thành công** (353 ms) | — |
| Cổng `useVirtualizer` | — | `OK: dung 1 loi goi useVirtualizer [ 'CommitList.tsx:1' ]` | cổng khẳng định tiền đề: đỏ nếu không thấy lời gọi nào |
| `cargo test --lib` | 309 | **309 passed / 0 failed** | không đụng Rust ở lượt này |
| `cargo test --lib --tests` | — | 🔴 **KHÔNG ĐO ĐƯỢC** | `git-plum.exe` **PID 49416** đang chạy → `error: failed to remove file ...\target\debug\git-plum.exe / Access is denied. (os error 5)`. **Không suy ra con số 425.** |

+10 test: 8 cho đường nối ở `App.test.tsx`, 2 cho `marginTop` ở `CommitList.test.tsx`.

**Không khẳng định nào của test cũ bị sửa để cho xanh.** Chín test đỏ lúc nối dây đều đỏ
với **cùng một** thông báo `No "ngheTrangThaiNgoai" export is defined on the "@/lib/ipc"
mock` — tức thiếu sót ở **độ đầy đủ của mock**, không ở hành vi. Chỉ các mục trong factory
`vi.mock` được **thêm**.

### Bảng đột biến (mỗi đột biến được kiểm là **đã áp** trước khi chạy)

| # | Đột biến | Kết quả | Đầu ra đỏ |
|---|---|---|---|
| **M1** | Bỏ `void useStatusStore.getState().refresh(activeRepoId)` khỏi `App.tsx` (chính là lỗi khoá vòng) | ✅ **3 đỏ** | *hàng WIP hiện, và bấm nó mở ChangeList + CommitBox* · *bấm hàng WIP KHÔNG chọn một commit nào* · *đóng vùng soạn → quay lại chi tiết commit* |
| **M2** | Bỏ `huy?.()` trong đường dọn effect watcher | ✅ **1 đỏ** | *unmount App thì huỷ đăng ký watcher — không rò người nghe qua các lần mở* |
| **M3** | Hàng WIP gọi thêm `onSelect('wip')` (bịa một SHA) | ✅ **2 đỏ** | `App.test.tsx :: bấm hàng WIP KHÔNG chọn một commit nào` · `CommitList.test.tsx :: bấm hàng WIP gọi onOpenCommitBox, và KHÔNG gọi onSelect với commitId bịa` |
| **M4** | `marginTop: contentOffset(hasWip)` → **`paddingTop`** | 🔴 **SỐNG SÓT — 0 đỏ / 666 total** | *(không có)* — xem dưới |
| **M4b** | *cùng đột biến*, sau khi thêm test | ✅ **2 đỏ** | `AssertionError: expected '' to be '28px'` · `AssertionError: expected '0px' to be ''` |
| **M5** | `virtualizerCount` trả `total + 1` khi `hasWip` (cách A qua cửa sau) | ✅ **4 đỏ** | 3 test của `wipRow.test.ts` (*total không đổi* ở 0 / 1 / 100007) + `CommitList.test.tsx :: count của virtualizer === total KỂ CẢ khi có hàng WIP` |

🔴 **M4 là kết quả quan trọng nhất của lượt này.** `marginTop` → `paddingTop` là một phép
sửa **một từ**, nó **tái tạo đúng** lớp lỗi "cột đồ thị lệch cột văn bản một hàng" đã xảy
ra **hai lần** ở Phase 2 (hàng commit là `position: absolute`, mà phần tử tuyệt đối neo
theo **padding box** — nên `paddingTop` dịch chỗ trống mà **không** dịch các hàng), và nó
đi qua **toàn bộ** bộ test mà **không một khẳng định nào đỏ**.

Đã kiểm rằng đột biến **diễn đạt được** khiếm khuyết trước khi kết luận: dòng bị sửa nằm
đúng trên đường render (`sed -n '480,492p'`), và `grep` toàn bộ `src/**/*.test.*` cho
`marginTop|paddingTop` trả **rỗng** — không test nào ở đâu hỏi tới hai thuộc tính này.
Cổng đã đóng bằng hai test mới; M4b cho 2 đỏ với đầu ra dán ở trên.

⚠️ Test mới ghim **ý định** (đúng tên thuộc tính trong style inline), **không** chứng minh
kết quả trên màn hình — happy-dom không tính layout. Bước 5 của checkpoint mới làm được.

### Bản dựng release — **KHÔNG dựng được ở lượt này**

```console
$ tasklist /FI "IMAGENAME eq git-plum.exe"
git-plum.exe                 49416 RDP-Tcp#1                  1     30,464 K

$ ls -l --time-style=full-iso src-tauri/target/release/git-plum.exe
-rwxr-xr-x 2 tuyen 197609 4619264 2026-09-22 11:53:54.612877500 +0700
```

🔴 Exe vẫn là bản **11:53:54**, tức **trước toàn bộ wave 4 và wave 5** — khoảng 9 giờ cũ.
Nó **không chứa** `wipRow.ts`, watcher, `CommitList` nối hàng WIP, lẫn `App.tsx` lượt này.

**Không dựng lại được**, và nói thẳng lý do thay vì dựng bừa:

1. `git-plum.exe` **PID 49416 đang chạy**. Chỉ dẫn của lượt này là **không giết nó**. Cargo
   cần ghi đè đúng tệp đó lúc link → `Access is denied. (os error 5)` (đã gặp thật ở
   `cargo test --lib --tests` ngay trên).
2. Đĩa C: còn **8.7 GB / 224 GB (97% đầy)**. Hai lần dựng hôm nay đã hỏng ở
   `LNK1318 PDB limit` và *paging file is too small*.

🔴 **Dấu thời gian exe hiện tại thuộc về commit nào:** không phải `2d7b3e4`, cũng không
phải `6be8743` hay `25b0afe` — nó có **trước** cả ba. Theo quy tắc 3.7 của CONTEXT.md,
**checkpoint chạy trên exe này là không hợp lệ.** Một exe cũ mang dấu thời gian mới còn tệ
hơn không có exe nào.

**Việc chủ dự án cần làm trước khi chạy checkpoint:** đóng `git-plum.exe` đang mở, rồi
`npm run tauri:build`, rồi dán lại dấu thời gian mới cùng `git rev-parse --short HEAD`.

### Hai cổng người-kiểm — **CHƯA CHẠY, và không tự phê duyệt được**

Chủ dự án đã chọn "dừng lại, báo tôi". Nguyên văn hai cổng được chuyển lại trong báo cáo
bàn giao; ở đây chỉ ghi **trạng thái**:

| Cổng | Trạng thái |
|---|---|
| 04-05 Task 3 — checkpoint hiển thị 12 bước | **chưa chạy** (thiếu bản dựng hợp lệ) |
| 04-05 Task 4 — exit gate dogfood | **chưa chạy** |

🔴 **Không bước nào trong 12 bước được ghi "Đạt".** Không có bằng chứng `mắt người
(release)` nào tồn tại ở phase này.

### Hệ quả thiết kế cần chủ dự án quyết (không tự sửa)

**Hàng WIP không cuộn đi** — nó là `position: sticky; top: 0`. Đó là hệ quả **chấp nhận
có chủ ý** của cách B: hàng này là chỗ vào vùng soạn commit nên phải luôn bấm được. Nhưng
nó **khác** một hàng commit bình thường, và không test nào chứng minh được là nó dễ chịu.
**Câu hỏi cho bước 5 của checkpoint: giữ hay bỏ?** Không tự đổi.

### Quyết định bố cục cần xác nhận (bước 1 của checkpoint)

`ChangeList` + `CommitBox` chiếm vùng **`detail`**, **thay** chi tiết commit, mở bằng cách
bấm hàng WIP và đóng bằng nút X. **Không đổi một dòng nào** trong `AppLayout.tsx` — cùng
tiền lệ 03-04 đã dùng cho diff viewer ở vùng `main`.

Vì sao `detail` chứ không `main`: hàng WIP — chỗ vào của cả vòng commit — **nằm trong** đồ
thị ở `main`. Đặt vùng soạn vào `main` sẽ che mất chính hàng vừa bấm để mở nó, và bước 5
(hàng WIP thẳng cột, cuộn lên xuống) không quan sát được cùng lúc với vùng soạn.

Đổi lại: chi tiết commit bị che khi vùng soạn mở. **Chủ dự án xác nhận hoặc yêu cầu đổi
sang `main`.**

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
*(Bảng dưới đây là trạng thái **lượt hai**, giữ lại để so sánh. Trạng thái **hiện tại** ở
bảng "Sau lượt ba" ngay bên dưới nó.)*

| **WORK-01** | Ba nhóm tệp; chọn tệp → thấy diff của nó | Có mã (04-01, 04-02), **chưa nối vào `App.tsx`** | `chưa kiểm` — không có đường nào từ giao diện tới `ChangeList`; và bố cục ba nhóm là khẳng định layout |
| **WORK-02** | Stage / unstage theo tệp | Có mã + test (04-01, 04-02) | `test tự động` cho logic store; `chưa kiểm` cho thao tác thật (chưa render trong ứng dụng) |
| **WORK-08** | Soạn thông điệp và tạo commit | Có mã + test (04-03) | `test tự động`; `chưa kiểm` — `CommitBox` chưa được `App.tsx` render, chưa ai gõ một thông điệp thật |
| **WORK-09** | Amend, cảnh báo nếu đã push nhưng không chặn | Có mã + test (04-03) | `test tự động`; `chưa kiểm` — chưa ai bấm amend trên repo thật |
| **WORK-10** | Thay đổi ngoài ứng dụng → tự cập nhật < 1 s | Có mã + 8 test (04-04) | `test tự động`; 🔴 **`noiWatcherVaoStore()` chưa được gọi ở đâu cả** — watcher tồn tại nhưng **không chạy trong ứng dụng**. Tiêu chí "< 1 giây, không bấm gì" **chưa kiểm** |
| **WORK-11** | Hàng WIP đầu đồ thị, số đếm, bấm mở vùng soạn | Mã nối dây **đã viết, 13 test xanh**, nhưng **CHƯA COMMIT được** (tệp bẩn — xem lượt hai) | `test tự động` cho **số học** và cho DOM ở happy-dom; 🔴 `chưa kiểm` cho **mọi** khẳng định hiển thị. Và vì mã chưa vào nhánh, ngay cả `test tự động` cũng **chưa tái lập được từ một lần clone sạch** |

### Sau lượt ba (`2d7b3e4`) — trạng thái **hiện tại**

🔴 **Cụm từ dùng thống nhất cho mọi mục hiển thị: "có mã, chưa kiểm".** Nó nói đúng hai
điều cùng lúc: đường nối **tồn tại và có test**, và **chưa một con mắt người nào** xác
nhận nó trên bản release. Không mục nào được ghi "Đạt".

| Req | Nội dung | Trạng thái | **Loại bằng chứng** |
|---|---|---|---|
| **WORK-01** | Ba nhóm tệp; chọn tệp → thấy diff của nó | **Có mã, chưa kiểm** — `ChangeList` đã render trong `App` (`2d7b3e4`) | `test tự động` cho đường nối (có mặt trong cây khi vùng soạn mở); 🔴 `chưa kiểm` cho bố cục ba nhóm và cho phép chọn tệp → diff (khẳng định layout + phụ thuộc trình xem diff Phase 3 **chưa ai dùng thật**) |
| **WORK-02** | Stage / unstage theo tệp | **Có mã, chưa kiểm** | `test tự động` cho logic store (04-02); 🔴 `chưa kiểm` cho việc bấm thật và thấy tệp chuyển nhóm **ngay** — bước 4 của checkpoint |
| **WORK-08** | Soạn thông điệp và tạo commit | **Có mã, chưa kiểm** — `CommitBox` đã render, `switchRepo` nối vào vòng đời repo | `test tự động` (04-03 + test đổi repo không rò nháp ở lượt này); 🔴 `chưa kiểm` — **chưa ai gõ một thông điệp thật và bấm commit** |
| **WORK-09** | Amend, cảnh báo nếu đã push nhưng không chặn | **Có mã, chưa kiểm** | `test tự động` (04-03); 🔴 `chưa kiểm` — chưa ai bấm amend trên repo thật (bước 12) |
| **WORK-10** | Thay đổi ngoài ứng dụng → tự cập nhật < 1 s | **Có mã, chưa kiểm** — `noiWatcherVaoStore()` **nay đã được gọi** ở `App` (`2d7b3e4`), có test cho cả đăng ký lẫn huỷ đăng ký | `test tự động` cho việc watcher được mount, huỷ đúng, và sự kiện ghi thẳng vào store **không** gọi thêm `git status`; 🔴 `chưa kiểm` cho **"< 1 giây"** — đó là phép đo thời gian thực, chỉ bước 11 của checkpoint làm được |
| **WORK-11** | Hàng WIP đầu đồ thị, số đếm, bấm mở vùng soạn | **Có mã, chưa kiểm** — toàn bộ đã vào nhánh; mã nay **tái lập được từ một lần clone sạch** | `test tự động` cho số học (`commitRowY`, `virtualizerCount`), cho DOM ở happy-dom, và cho `marginTop` (cổng mới, đột biến M4b); 🔴 `chưa kiểm` cho **mọi** khẳng định hiển thị: thẳng cột khi cuộn (bước 5), cạnh nối xuống HEAD (bước 6), chiều cao 28px thật |

**Vẫn không requirement nào đạt mức `mắt người (release)`.** Đó là điều kiện để đóng phase
và nó **chưa** được thoả — bản release hợp lệ còn chưa dựng được.

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

### Sau lượt ba (`2d7b3e4`) — bảy tiêu chí, trạng thái **hiện tại**

| # | Tiêu chí | Trạng thái | **Loại bằng chứng** |
|---|---|---|---|
| 1 | Ba nhóm tệp hiện đúng, đúng tệp trong từng nhóm | **Có mã, chưa kiểm** — `ChangeList` đã nối vào `App` | `test tự động` cho việc component có mặt; 🔴 `chưa kiểm` cho bố cục (happy-dom không tính CSS) |
| 2 | Stage/unstage đổi nhóm **ngay**, không phải bấm làm mới | **Có mã, chưa kiểm** | `test tự động` cho store ghi từ giá trị trả về; 🔴 `chưa kiểm` cho phản hồi thật trên màn hình |
| 3 | Chọn tệp → diff của **đúng** tệp đó | **Có mã, chưa kiểm** | 🔴 `chưa kiểm` — dựa trên trình xem diff Phase 3 **chưa ai dùng thật**; lỗi ở đây là lỗi Phase 3 lộ ra |
| 4 | Commit xong → đồ thị cập nhật ngay, ô soạn rỗng lại | **Có mã, chưa kiểm** | `test tự động` (04-03, `lamMoiDoThi`); 🔴 `chưa kiểm` — chưa ai tạo một commit thật qua giao diện |
| 5 | Thay đổi từ terminal → giao diện tự cập nhật < 1 s | **Có mã, chưa kiểm** — watcher **nay đã mount** (khác lượt trước: lúc đó **chưa mount**) | `test tự động` cho đường nối watcher → store; 🔴 `chưa kiểm` cho **phép đo "< 1 giây"** — chỉ chạy ứng dụng thật mới đo được |
| 6 | Nháp thông điệp còn sau khi đóng/mở lại và khi chuyển repo | **Có mã, chưa kiểm** | `test tự động` (04-03 + test "đổi repo không rò nháp" ở lượt này); 🔴 `chưa kiểm` cho vòng đời **đóng/mở lại ứng dụng thật** — không mô phỏng được ở happy-dom |
| 7 | Hết thay đổi → hàng WIP **biến mất** | **Có mã, chưa kiểm** | `test tự động` cho `hasWipRow({0,0}) === false` và cho việc không render; 🔴 `chưa kiểm` cho thứ nhìn thấy sau một lần commit thật |

🔴 **Không tiêu chí nào trong bảy tiêu chí đạt `mắt người (release)`.** Tiêu chí 5 và
"thẳng cột" của hàng WIP **về nguyên tắc** không kiểm được bằng test tự động — và đột biến
**M4** ở lượt này là bằng chứng cụ thể cho điều đó: một phép sửa **một từ** tái tạo đúng
lớp lỗi lệch-một-hàng của Phase 2 mà **0 test** bắt được, cho tới khi test được thêm.

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
| **4** | **checkpoint hiển thị 12 bước (04-05 Task 3)** | **chưa chạy** — Task 0 **đã xanh** và mã **đã xong**; nay chặn ở **thiếu bản release hợp lệ** (exe 11:53, ~9 giờ cũ; không dựng lại được khi `git-plum.exe` PID 49416 đang chạy, đĩa 97%) |
| **4** | **exit gate dogfood (04-05 Task 4)** | **chưa chạy** — cùng lý do |

Phase 4 là **phase thứ tư liên tiếp** đứng trước nguy cơ đóng với nợ kiểm chứng. Khuôn hình
đó được ghi ra ở đây thay vì để nó tích lại im lặng.

---

## Việc còn lại của wave 5 — **sau lượt ba**

Năm mục mà lượt hai liệt kê, mục 1–4 **đã xong và đã vào nhánh**:

1. ✅ `App.tsx`: `noiWatcherVaoStore()` mount theo vòng đời ứng dụng, huỷ đúng; render
   `ChangeList` + `CommitBox`; `statusStore.refresh` khi mở repo (**sửa lỗi khoá vòng**);
   `commitStore.switchRepo` (không phải `hydrate` thẳng). — `2d7b3e4`
2. ✅ `CommitList.tsx`: `virtualizerCount(total, hasWip)`, hàng WIP là phần tử **anh em**
   ghim ngoài virtualizer (**cách B** giữ nguyên), `commitRowY(v.start, scrollTop, hasWip)`.
   — `25b0afe`
3. ✅ `GraphCanvas`/`canvasRenderer`: cạnh WIP qua tham số thứ ba của `draw`, interface
   `GraphRenderer` giữ nguyên. — `6be8743`
4. ✅ CSS hàng WIP cao **đúng** `ROW_HEIGHT` (28px), phân tách bằng `box-shadow` chứ không
   `border` (border cộng vào chiều cao). — `25b0afe`
5. ❌ **Dựng release, dán dấu thời gian, chạy checkpoint 12 bước rồi exit gate.** ← **đây là
   toàn bộ phần còn lại của phase**, và nó **không phải việc của executor**.

### Ba việc, theo thứ tự, để đóng phase

1. **Chủ dự án đóng `git-plum.exe`** (PID 49416) — executor được chỉ thị **không giết nó**.
2. `npm run tauri:build`, rồi dán `ls -l --time-style=full-iso src-tauri/target/release/git-plum.exe`
   **cùng** `git rev-parse --short HEAD`. Dấu thời gian phải **sau** `2d7b3e4`.
3. Chạy checkpoint 12 bước (Task 3), rồi exit gate dogfood (Task 4).

🔴 **Kết quả đúng khi cổng không đóng được là một plan vá khoảng cách, không phải đi tiếp
Phase 5** (ROADMAP + CONTEXT.md mục 5). Phase 4 là **phase thứ tư liên tiếp** đứng trước
nguy cơ đóng với nợ kiểm chứng; mã xanh **không** thay được một con mắt người.
