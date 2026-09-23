---
phase: 05-hunk-staging
plan: 04
subsystem: hunk-staging-ui
tags: [WORK-03, WORK-05, WORK-06, zustand, chong-phase-4, happy-dom]
requires:
  - "05-02 (stage_hunk / unstage_hunk — Tauri command đã đăng ký)"
  - "05-03 (discard_hunk / discard_files / list_trash / restore_trash, MucThungRac)"
  - "04-02 (statusStore.applyExternal — điểm vào ghi status từ giá trị trả về)"
provides:
  - "lib/ipc: mã lỗi `file_changed`, kiểu `MucThungRac`/`DongTu`, `nhanDongTu`"
  - "lib/ipc: stageHunk / unstageHunk / discardHunk / discardFiles / listTrash / restoreTrash"
  - "stores/hunkStore: `useHunkStore`, `khoaKhoi`, kiểu `CanLamMoi`"
  - "components/worktree/HunkBar: thanh nút một khối, KHÔNG phụ thuộc Phase 4"
affects:
  - "05-05 (nối HunkBar vào App.tsx — nơi có mắt người)"
tech-stack:
  added: []
key-files:
  created:
    - "src/stores/hunkStore.ts (168 dòng)"
    - "src/stores/hunkStore.test.ts (322 dòng, 15 test)"
    - "src/components/worktree/HunkBar.tsx (200 dòng)"
    - "src/components/worktree/HunkBar.test.tsx (20 test)"
  modified:
    - "src/lib/ipc.ts (+125 dòng: mã lỗi, kiểu thùng rác, sáu hàm bọc)"
    - "src/styles/app.css (+63 dòng, kèm khối 'CHƯA KIỂM' liệt kê bốn khẳng định bố cục)"
decisions:
  - "`onLamMoi` là PROP chứ không phải `useDiffStore()` — đọc diffStore ở HunkBar sẽ dựng đúng phụ thuộc mà M25 tồn tại để cấm"
  - "M25 quá rộng (16 đỏ) nên dựng thêm M25b (chỉ vô hiệu nút) để cô lập đúng khuyết tật — khuôn M14/M14c của wave 3"
  - "M26 (`role=\"alert\"`) 0 đỏ → grep cho thấy không test nào hỏi → viết test, M26 thành ĐỎ. Lỗi #9 tái hiện sống"
metrics:
  duration: "~1h"
  completed: "2026-09-23"
---

# 05-04 — `hunkStore` + `HunkBar`: staging theo khối tới được giao diện

Store và component cho staging theo khối, **không** phụ thuộc bất kỳ thành phần nào của
Phase 4, và điều đó được ghim bằng test chứ không bằng lời hứa. Cộng thông báo WORK-05
mang **nguyên văn** câu ROADMAP.

## Số đo — tôi tự chạy, lệnh đầy đủ

| Phép đo | Trước | Sau | Lệnh |
|---|---:|---:|---|
| Toàn suite frontend | **668 passed, 0 failed** | **703 passed, 0 failed** (+35) | `npx vitest run --reporter=json --outputFile=.vitest/json/all05.json` |
| Suite (số bộ) | 229 | **242** | cùng lệnh |
| `hunkStore.test.ts` | — | **15 passed, 0 failed** | `npx vitest run src/stores/hunkStore.test.ts` |
| `HunkBar.test.tsx` | — | **20 passed, 0 failed** | `npx vitest run src/components/worktree/HunkBar.test.tsx` |
| `npx tsc --noEmit` | sạch | **sạch** (`exit=0`) | `npx tsc --noEmit` |
| `npm run build` | — | **✓ built in 543ms** | `npm run build` |

Con số đọc **sau** khi lệnh xong, từ tệp `--outputFile` **riêng** cho mỗi lần chạy
(`base05.json` / `all05.json` / `hunk.json` / `hunkbar.json` / `mut.json`), `rm -f`
trước mỗi lần — `rtk` ghi đè `.vitest/json/output.json` sau lệnh của chính mình.

Mã thoát `tsc` đọc từ một lời gọi **riêng, không qua đường ống**: `$?` sau một pipeline
đọc mã của lệnh **cuối**, không phải của `tsc` (bẫy wave 3 suýt ghi sai).

> ⚠️ **`npm run build` có một cảnh báo, và nó CÓ TRƯỚC:** `index-UmHo2kvK.js` 645,53 kB
> vượt ngưỡng 500 kB. Phần tôi thêm vào là ~5 kB; cảnh báo này đã có từ trước và không
> thuộc phạm vi plan. **Không sửa**, ghi lại để không ai tưởng nó mới.

### 🔴 Lỗi #8 tái hiện sống, hai lần, ở chính lượt chạy RED

Cả hai lần RED đều cho:

```text
[total=0 failed=0 suitesFailed=1]
Failed to resolve import "./hunkStore" from "src/stores/hunkStore.test.ts". Does the file exist?
```

`total=0 failed=0`. Một cổng chỉ kiểm `numFailedTests !== 0` sẽ gọi đây là **XANH** —
đúng lỗi #8 của `CONTEXT.md` §4.1, xảy ra **ngay trong lượt chạy đầu tiên của plan này**.
Khẳng định `numTotalTests >= 5` là thứ bắt được nó, và đó là lý do plan đòi nó.

## Bảng đột biến — chạy thật, dán đầu ra đỏ

Hoàn nguyên bằng `cp` từ bản sao dựng bằng `git show <sha>:<path> > file`, **không bao
giờ** `git checkout --`. Sau **mỗi** lần: `git status --porcelain` trên ba tệp trả
**rỗng** (in dạng `[...]`, nhìn bằng mắt, không đếm).

| # | Đột biến | Đỏ mong đợi | **Đo được** |
|---|---|---|---|
| M-A | Thêm `refresh` sau `applyExternal` trong `hunkStore` | ràng buộc 2.5 | ✅ **2 đỏ** — đúng nhóm "ghi status TỪ GIÁ TRỊ TRẢ VỀ" |
| M21 | Nhãn huỷ luôn `"Huỷ bỏ"` | Test 2 | ✅ **2 đỏ** — cả hai ở nhánh `untracked=true` |
| M22 | Nhãn huỷ luôn `"Xoá"` | Test 2 | ✅ **1 đỏ** — ở nhánh `untracked=false` |
| M23 | `onClick` truyền `0` thay vì `index` | Test 3 | ✅ **4 đỏ** |
| M24 | Thông báo đổi `"hãy làm mới"` → `"thử lại"` | Test 5 | ✅ **1 đỏ** |
| M25 | `HunkBar` đọc store Phase 4, `return null` | Test 4 | ✅ **16 đỏ** — nhưng **quá rộng**, xem dưới |
| M25b | `HunkBar` đọc store Phase 4, chỉ **vô hiệu nút** | Test 4 | ✅ **10 đỏ** — đây mới là phép đo cô lập |
| M26 | Bỏ `role="alert"` khỏi băng | (không dự đoán) | ⚠️ **0 đỏ → ĐỎ sau khi viết cổng**, xem dưới |
| M26b | Gắn `role="alert"` lên **vỏ** (đối chứng) | test phủ định | ✅ **2 đỏ** |

### 🔴 M25 — ĐỎ, và nó là bằng chứng ràng buộc được CÀI

Đây là đột biến quan trọng nhất của plan. Đầu ra thô:

```text
[total=33 failed=16]
  ĐỎ: 🔴 M25: HunkBar dùng được KHÔNG CẦN thành phần nào của Phase 4 render MỘT MÌNH, statusStore rỗng → nút vẫn hiện và vẫn gọi IPC
  ĐỎ: 🔴 M25: HunkBar dùng được KHÔNG CẦN thành phần nào của Phase 4 không nút nào bị vô hiệu khi chưa có dữ liệu Phase 4
  ĐỎ: nhãn nút staged=false → "Đưa khối vào vùng chờ"
  ĐỎ: nhãn nút staged=true → "Lấy khối khỏi vùng chờ"
  … (12 test còn lại)
```

Đột biến là đúng hình dạng vòng khoá chết mà wave 5 Phase 4 tìm được:

```tsx
const trangThai = useStatusStore((s) => s.byRepo[repoId]?.status)
if (!trangThai) return null
```

**Nhưng 16 đỏ là quá rộng.** Đúng bài học M14 của wave 3 — *đột biến không biểu diễn
được đúng khuyết tật thì nói ít hơn nó có vẻ nói* — `return null` phá **mọi** test vì
component không render gì cả, nên nó không tách được "lệ thuộc Phase 4" khỏi "không
render". Nên tôi dựng **M25b**: giữ nguyên việc render, chỉ **vô hiệu nút** khi store
Phase 4 chưa có dữ liệu — đúng lỗi thật một lập trình viên sẽ viết (*"tôi vô hiệu cho
an toàn tới khi biết tệp có theo dõi không"*):

```text
[total=33 failed=10]
  ĐỎ: 🔴 M25: … render MỘT MÌNH, statusStore rỗng → nút vẫn hiện và vẫn gọi IPC
  ĐỎ: 🔴 M25: … không nút nào bị vô hiệu khi chưa có dữ liệu Phase 4
  ĐỎ: bấm nút gọi đúng hành động … (3 test)
  ĐỎ: 🔴 WORK-05 … (5 test)
```

**Ba test nhãn XANH** ở M25b (nhãn vẫn render đúng), và test `không nút nào bị vô hiệu`
đỏ. Đó là phép cô lập plan muốn: cổng đo **chính** tính "dùng được khi Phase 4 vắng
mặt", không chỉ đo "component có render không".

### 🔴 M26 — lỗi #9 tái hiện sống, và nó là phát hiện đáng giá nhất của wave này

M26 (bỏ `role="alert"` khỏi băng `file_changed`) cho **0 đỏ trên 33 test**.

Theo đúng quy tắc — *trước khi kết luận đột biến sống sót vì mã đúng, kiểm xem có test
nào **hỏi** về thứ đó không* — tôi grep toàn bộ `src/**/*.test.*` (bằng **công cụ Grep**,
không `grep` qua shell):

```text
src\components\worktree\ChangeList.test.tsx:324:  … screen.getByRole('alert').textContent … 'index.lock'
src\components\worktree\ChangeList.test.tsx:428:  … screen.getByRole('alert').textContent … 'repository'
```

Hai khớp, **cả hai ở `ChangeList.test.tsx`**, **không khớp nào** trong `HunkBar.test.tsx`.
**Rỗng nghĩa là cổng thiếu, không phải mã đúng.** Viết test, chạy lại:

```text
[total=35 failed=1]
  ĐỎ: 🔴 WORK-05: thông báo "hãy làm mới" băng mang role="alert" để trình đọc màn hình đọc ra
```

**M26: 0 đỏ → ĐỎ.** Cùng khuôn với M11 của wave 2 và M18 của wave 3 — ba wave liên tiếp,
mỗi wave tìm được đúng một ca lỗi #9 bằng cùng một phép kiểm.

**Vì sao thuộc tính này đáng có cổng, không phải thêm cho đủ bảng:** băng là thứ **duy
nhất** nói cho người dùng biết cú bấm của họ **không** có hiệu lực. Không có
`role="alert"`, trình đọc màn hình **không đọc nó ra** — người dùng bấm "Đưa khối vào
vùng chờ", không nghe gì, và tin rằng khối đã vào vùng chờ. Với một tính năng ghi vào
tệp của họ, im lặng là chế độ hỏng tệ nhất.

Và tôi thêm **M26b** làm đối chứng cho chính test mới: gắn `role="alert"` lên **vỏ**
`hunk-bar` (tức có mặt ở **mọi** trạng thái, kể cả lúc không có lỗi) → **2 đỏ**, trong
đó có test phủ định `không có lỗi → KHÔNG có phần tử role="alert" nào`. Không có đối
chứng đó, một cài đặt "báo động ở mọi trạng thái" sẽ qua được test dương — đúng lỗi #6.

### M21 + M22 — bằng chứng test phân biệt được CẢ HAI chiều

M21 đỏ ở nhánh `untracked=true`; M22 đỏ ở nhánh `untracked=false`. **Hai tập đỏ rời
nhau.** Nếu chỉ một trong hai đỏ thì Test 2 chỉ kiểm một nhánh và động từ "Xoá" của
ROADMAP không được bảo vệ ở chiều còn lại.

Phép kiểm **phủ định** là thứ làm nên khác biệt: `expect(nut.textContent).not.toContain('Huỷ bỏ')`.
Một nhãn `"Xoá / Huỷ bỏ"` — hay một cài đặt hiện cả hai "cho an toàn" — qua được
`toContain('Xoá')` mà không qua được phép kiểm này.

## 🔴 Ba câu hỏi về vòng khoá chết — trả lời thẳng, không làm mượt

Orchestrator đòi trả lời bằng chữ. Câu trả lời **không thoải mái**, và tôi ghi đúng như
nó là.

### 1. Cái gì mount `HunkBar`?

**Không cái gì cả.** Grep toàn bộ `src/**` trừ tệp test:

```text
HunkBar.tsx       — khai báo chính nó
hunkStore.ts      — nhắc trong chú thích
app.css           — nhắc trong chú thích
(không có chỗ gọi nào)
```

`HunkBar` **chưa được render ở đâu ngoài tệp test của chính nó**. Plan 05-04 cố ý dừng
trước việc nối vào `App.tsx` — đó là 05-05, "nơi có mắt người" (nguyên văn objective).

### 2. Thứ đó có phụ thuộc vào cái mà chính component tạo ra không?

**Ở trạng thái hiện tại, câu hỏi chưa áp dụng được** — không có "thứ đó". Nhưng đây
chính là chỗ vòng khoá chết Phase 4 sinh ra, nên tôi trả lời cho **thiết kế** thay vì
cho mã hiện có:

`HunkBar` **không tạo ra** dữ liệu nào mà chỗ mount nó cần. Nó không gọi `refresh`,
không nạp status, không nạp diff — nó chỉ **nhận** `path`/`index`/`blobHash`/`staged`/
`untracked` qua props. Nên nó **không thể** là mắt xích trong một vòng kiểu
"A chỉ hiện khi B có dữ liệu, mà B chỉ nạp khi A mount". Điều đó khác hẳn `ChangeList`,
thứ **vừa** là chỗ duy nhất gọi `refresh` **vừa** chỉ mount khi dữ liệu đã có.

Đó là tính chất tôi cài có chủ ý, và nó là tính chất **cấu trúc**, kiểm được bằng cách
đọc: component không có `useEffect` nào, không có lời gọi nạp nào.

### 3. Có đường tới nó không đi qua một thành phần Phase 4 không?

**Chưa có đường nào, nên chưa có đường không-đi-qua-Phase-4.** Đây là câu trả lời khó
chịu và tôi không làm nó tròn hơn.

Phân biệt phải giữ cho rõ, vì đúng nó là thứ wave 3 học được ở M18 (*"bước đọc lại phải
**tới được**, không chỉ **có mặt**"*):

| Mệnh đề | Trạng thái |
|---|---|
| `HunkBar` **có thể** hoạt động không cần Phase 4 | ✅ **Đã ghim bằng test**, M25 + M25b đo được đỏ |
| Người dùng **có** một đường tới nó không qua Phase 4 | ❌ **Chưa**, và chưa có đường nào cả |

Mệnh đề thứ hai là thứ vòng khoá chết Phase 4 vi phạm — ở đó mọi component **có thể**
hoạt động, nhưng **đường tới** hàng WIP đóng vòng lại chính nó. Plan 05-04 đóng được
mệnh đề **một**; mệnh đề **hai** là việc của 05-05, và nó là chỗ rủi ro thật còn lại
của ràng buộc chống-Phase-4.

🔴 **Hệ quả cụ thể cho 05-05:** khi nối `HunkBar` vào, phải kiểm rằng đường tới nó
**không** đi qua `ChangeList` → hàng WIP → vùng soạn commit. Nếu đường vào duy nhất của
staging theo khối là "mở diff từ `ChangeList`", thì toàn bộ công của wave này **không
mua được gì** — component sạch mà đường tới nó thì không.

## Điều plan nói mà hoá ra sai

### 1. "Ba nút hiện ra với nhãn đúng" — thật ra là **hai**

Plan Test 1 ghi *"ba nút hiện ra"*. Thiết kế đúng là **hai**: một nút vùng chờ (nhãn
đổi theo `staged`) và một nút huỷ. Ba nút nghĩa là hiện **cả** "Đưa vào" **lẫn** "Lấy
ra" cùng lúc, mà `staged` là boolean — một trong hai luôn vô nghĩa. Test khẳng định
`getAllByRole('button').length === 2` và khẳng định nút của chiều kia **vắng mặt**
(`queryBy… toBeNull`), chặt hơn "ba nút có mặt".

### 2. M25 "đỏ ở Test 4" — đúng, nhưng đó không phải phép đo có ích

Xem mục M25 ở trên. Plan dự đoán đúng **dấu**, nhưng 16 đỏ không cô lập được gì.
M25b mới là phép đo trả lời được câu hỏi.

### 3. Bảng đột biến của plan **thiếu** ca đắt nhất

Plan liệt kê M21–M25. Ca thật sự tìm ra khuyết tật là **M26**, không có trong bảng, và
nó chỉ lộ ra vì tôi chạy phép kiểm lỗi #9 trên một thuộc tính plan không nhắc tới. Bảng
đột biến do plan viết ra **không** là tập đủ — wave 2 và wave 3 đều ghi nhận điều này,
và wave này là lần thứ ba.

## Điều TÔI làm khác plan, và vì sao

**`onLamMoi` được thêm làm prop — plan không nhắc tới nó.**

Plan Test 5 đòi *"một nút làm mới **bấm được**"*. Cài đặt đầu của tôi đúng chữ: nút
`"Làm mới"` gỡ băng đi. Test xanh. Nhưng nút đó **nói dối**: người dùng bấm "Làm mới",
băng biến mất, **diff vẫn cũ y nguyên**, và lần bấm stage kế tiếp lại thất bại.

Đây đúng lớp lỗi #9 nhìn từ phía hành vi: "băng biến mất" là một thuộc tính quan sát
được, nhưng **không phải** thuộc tính cần. Nên tôi thêm `onLamMoi?: () => void` và một
test khẳng định nút **gọi** nó, cộng một test cho nhánh `undefined`.

🔴 **Nó là PROP chứ không phải `useDiffStore()` gọi thẳng**, và đó là điểm mấu chốt:
nạp lại diff là việc của trình xem diff, tức Phase 4. Đọc `diffStore` trong `HunkBar`
sẽ dựng **đúng** phụ thuộc mà M25 tồn tại để cấm.

🔴 **Và prop đó chưa được truyền ở đâu cả** — xem ba câu hỏi ở trên.

## Requirement

| Req | Trạng thái |
|---|---|
| **WORK-03** | Store + nút cho stage/unstage **một khối** có mã và có test. **Tầng giao diện: mã đạt, chưa ai bấm.** Tầng Rust đã đạt ở 05-02 |
| **WORK-05** | Lỗi `file_changed` phân nhánh theo **`code`**, băng mang **nguyên văn** "hãy làm mới", có nút làm mới gọi được. Phân biệt được với `index_locked` và với lỗi mã khác mang cùng câu chữ |
| **WORK-06** | Nút huỷ theo khối, động từ **"Xoá"** cho tệp chưa theo dõi, ghim hai chiều bằng M21+M22 |

**Không requirement nào ghi "Đạt".** Cả ba có phần **chỉ người kiểm được**, và
`CONTEXT.md` mục 0.3 cấm ghi "Đạt" cho tiêu chí hình ảnh dựa trên test happy-dom.

## 🔴 Chưa kiểm — ghi rõ, không làm mượt

### Mọi khẳng định về bố cục: **có mã, chưa kiểm**

happy-dom **không tính CSS layout và không có cuộn thật**. Cụ thể, **không một test nào**
trong 703 test quan sát được những điều sau, và chúng nằm trong `app.css` kèm chú thích
nói đúng như vậy:

- hai nút **không đè lên nhau** khi nhãn dài (`"Đưa khối vào vùng chờ"` + `"Huỷ bỏ khối"`);
- thanh **không tràn** ra khỏi bề ngang khối diff chứa nó;
- băng "hãy làm mới" **không che** mất dòng diff ngay bên dưới;
- thanh **không làm nhảy chỗ** nội dung khi băng hiện ra (`flex-basis: 100%` xuống dòng riêng);
- `min-width: 0` trên `.hunk-bar-can-lam-moi-text` có thật sự giữ nút "Làm mới" trong khung không.

Đây không phải từ chối lấy lệ: lỗi `marginTop`→`paddingTop` của Phase 4 đi qua **cả 666
test**, và **năm** lỗi hiển thị Phase 3 đều thoát toàn bộ test tự động. Việc kiểm mắt
người nằm ở checkpoint **05-05**.

### Khoảng trống kế thừa từ wave 3 — **không** được lấp ở wave này

- **`danh_sach_thung_rac` trả `paths: []`, `luc: 0`, `nhan: ""`.** `for-each-ref` không
  giữ được ba trường đó. Tôi **không** dựng giao diện danh sách "Vừa huỷ gần đây" ở wave
  này (plan không yêu cầu), nên khoảng trống **chưa bị chạm tới** và cũng **chưa được
  lấp**. Tôi ghi nó **tường minh vào doc comment của `ipc.listTrash`** để wave sau không
  phát hiện lại bằng cách dựng một danh sách trống rồi mới hiểu tại sao.
  🔴 **Hệ quả nếu 05-05 dựng danh sách đó:** nó sẽ hiện **không tên tệp, không thời
  điểm**. Cần một chỗ lưu phụ (ghi chú ref hoặc tệp chỉ mục) — việc có thật, không phải
  làm đẹp.
- **T-05-11 chưa cài** — `khoi_phuc` ghi đè thẳng, không tự lưu trước. Cũng ghi vào doc
  comment của `ipc.restoreTrash`. Vẫn là **nợ**.

### Chưa kiểm khác

- **Không có test tích hợp thật nào** cho đường này. Mọi lời gọi IPC đều là `vi.fn()`;
  không có test nào chạy `stage_hunk` thật qua Tauri. Hợp đồng tham số (`repoId`, `path`,
  `hunkIndex`, `blobHash`) được đối chiếu **bằng mắt** với chữ ký Rust ở
  `commands/hunk.rs:322` và `commands/trash.rs:744-786`, **không** bằng một phép kiểm
  tự động. Một lệch tên tham số sẽ chỉ lộ ra lúc chạy thật.
- **`nhanDongTu` là nguồn sự thật thứ HAI.** Rust có `DongTu::nhan()` với cùng nội dung
  (`"Huỷ bỏ"` / `"Xoá"`). Hai chỗ khai cùng một chuỗi, và **không có** test nào đối
  chiếu chúng — khác `MucThungRac`, nơi tôi so bằng danh sách khoá ở cả hai phía. Lệch
  một dấu sẽ không ai biết. Ghi là **nợ đã biết**; lấp được bằng cách trả `DongTu` từ
  backend thay vì suy từ `untracked` ở frontend.
- **`cargo` không chạy ở wave này** theo chỉ thị của orchestrator (bộ Rust đang dựng
  nền). Mọi con số Rust trong tài liệu này **chép từ** wave 2/wave 3, không phải tôi đo.
- **`dangChon` / `chon` / `boChon` chưa có ai dùng.** Store có chúng và có test, nhưng
  `HunkBar` **không** đọc `dangChon` — việc tô sáng khối đang chọn thuộc về trình xem
  diff ở 05-05. Đây là API viết trước chỗ dùng; nếu 05-05 chọn cách khác thì nó là mã
  chết cần xoá, không phải nền móng.

## Bẫy môi trường gặp phải

- **`rtk` làm hỏng `grep` nặng hơn cảnh báo.** `grep -n "..." <một tệp>` trả
  *"49 matches in 11 files"* và *"11 matches in 6 files"* cho một tệp duy nhất, và
  `grep -A 45` cắt mất khối cần đọc. Mọi phép tìm **quyết định** (nhất là phép kiểm lỗi
  #9 dẫn tới M26) chạy bằng **công cụ Grep**.
- **`.vitest/json/*.json` bị ghi đè**: dùng tên tệp riêng cho từng lần chạy, `rm -f`
  trước, đọc sau.
- **`grep -c` trả `0` thoát `1`**, làm một lệnh nối `&&` trông như thất bại trong khi
  nó đang báo đúng "không còn dấu đột biến". Tách lệnh để đọc.
- **Đĩa:** không chạy `cargo`, không tạo `CARGO_TARGET_DIR` — đúng chỉ thị. `dist/` do
  `npm run build` sinh ra là vài trăm kB, không đáng kể.
- **Phối hợp nhiều phiên:** `git status --porcelain` in dạng **thô** trước và sau toàn
  bộ lượt làm việc cho **đúng một** tập 8 mục giống hệt nhau — sáu tệp của phiên khác và
  hai tệp chưa theo dõi, không tệp nào bị tôi chạm. Mọi lần `git add` đều theo **đường
  dẫn tường minh** và `git diff --cached --name-only` được in thô trước mỗi commit.

## Commit

| SHA | Nội dung |
|---|---|
| `f47c8c2` | test(05-04): test đỏ cho `hunkStore` (RED) |
| `fc7548b` | feat(05-04): `hunkStore` + kiểu IPC (GREEN) |
| `184b071` | test(05-04): test đỏ cho `HunkBar` (RED) |
| `e9672de` | feat(05-04): `HunkBar` (GREEN) |
| `b09b533` | test(05-04): cổng thiếu cho `role="alert"` — lỗi #9 |

Cặp RED→GREEN đầy đủ hai lần. **0 tệp bị xoá** trong toàn bộ dải commit
(`git diff --diff-filter=D --name-only eac91ac..HEAD` → rỗng).

## Self-Check: PASSED

Tệp có thật trên đĩa: `src/stores/hunkStore.ts`, `src/stores/hunkStore.test.ts`,
`src/components/worktree/HunkBar.tsx`, `src/components/worktree/HunkBar.test.tsx`,
`.planning/phases/05-hunk-staging/05-04-SUMMARY.md`.

Commit có thật trong `git rev-list`: `f47c8c2`, `fc7548b`, `184b071`, `e9672de`,
`b09b533`.
