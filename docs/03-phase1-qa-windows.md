# Kịch bản QA thủ công cho Windows — Phase 1

**Tạo:** 2026-09-21
**Phạm vi:** năm tiêu chí thành công của Phase 1, cộng khoảng cách G3, G7 và G8
**Nền tảng:** Windows 11 (nền tảng ưu tiên theo quyết định 2026-09-21)

Tài liệu này để **lặp lại được**. Mỗi kịch bản viết rõ bấm gì, thấy gì thì đỗ, thấy gì
thì trượt. Không có bước nào dạng "kiểm tra xem nó có chạy không".

Ba thứ trong đây **không đọc mã mà biết được**:

- **G3** — kích thước ba vùng còn nguyên sau khi đóng mở lại (`localStorage` trong webview
  Tauri có bền qua các lần chạy hay không là câu hỏi thực nghiệm).
- **G7** — cửa sổ console nháy, **chỉ hiện ở bản release**.
- **G8** — trạng thái bật tắt bảng nhật ký sống sót qua lần khởi động hay không.

---

## Chuẩn bị

### 1. Dựng bản release

Tại thư mục gốc dự án:

```
npx tauri build
```

LTO đang bật nên lần dựng đầu mất vài phút. Sản phẩm:

| Thứ | Đường dẫn |
|---|---|
| Tệp thực thi | `src-tauri/target/release/git-plum.exe` |
| Bộ cài MSI | `src-tauri/target/release/bundle/msi/` |

> **Nếu `cargo` không có trên PATH:** dùng `dev.cmd` ở thư mục gốc dự án — nó thêm
> `.cargo\bin` vào PATH trước khi chạy. Cần thiết với các cửa sổ `cmd` mở từ trước khi
> cài rustup.

### 2. Bắt buộc dùng bản release, không dùng `tauri dev` và không dùng `--debug`

Phần **KB-5** (cửa sổ console) chỉ có nghĩa trên bản release. Lý do nằm ở dòng đầu
`src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
```

Thuộc tính `windows_subsystem = "windows"` gắn với `not(debug_assertions)`, nên bản
`--debug` **không** bật nó. Chạy bản debug rồi kết luận "không có cửa sổ console" là
kết luận sai — và tệ hơn, nó là kết luận sai theo hướng làm ta yên tâm.

Tương tự, chạy `git-plum.exe` **từ terminal** cũng làm mất ý nghĩa phép kiểm: tiến trình
con thừa hưởng console đang có sẵn nên không có cửa sổ mới nào phải cấp. Phải bấm đúp
từ Explorer.

### 3. Ba thư mục để thử

| Nhãn | Nội dung | Dùng cho |
|---|---|---|
| (a) | Một repository git **thật**, có ít nhất một commit | KB-1, KB-2, KB-2b, KB-3 |
| (b) | Một thư mục **con** nằm bên trong (a) — ví dụ `src/` | KB-1 |
| (c) | Một thư mục hoàn toàn **không** phải repository | KB-4 |

Thư mục (b) là phần quan trọng nhất của KB-1: ứng dụng phải nhận đúng repository **cha**,
đó là tác dụng của `git rev-parse --show-toplevel`.

### 4. Cách thoát hẳn ứng dụng

Nhiều kịch bản dưới đây đòi "thoát hẳn". Nghĩa là:

1. Đóng cửa sổ ứng dụng.
2. Mở Task Manager (`Ctrl+Shift+Esc`), tab **Details**.
3. Xác nhận **không còn** tiến trình `git-plum.exe` nào.

Mở lại khi tiến trình cũ còn sống thì kết quả vô nghĩa — trạng thái đọc ra từ bộ nhớ của
tiến trình cũ, không phải từ đĩa.

---

## KB-1 — Mở repository và danh sách gần đây

**Tiêu chí thành công 1 · PLAT-06 + PLAT-07**

### Các bước

1. Bấm **"Mở repository"** trên thanh công cụ.
2. Chọn thư mục **(a)**. Quan sát thanh công cụ.
3. Bấm **"Đóng"**.
4. Bấm **"Mở repository"** lần nữa, lần này chọn thư mục **(b)** — thư mục con.
5. Bấm **"Đóng"**. Màn hình trống hiện danh sách gần đây.
6. **Thoát hẳn ứng dụng** (xem mục Chuẩn bị 4).
7. Mở lại từ Explorer.
8. Bấm vào mục trong danh sách gần đây.

### Đỗ khi

- Bước 2: thanh công cụ hiện **tên repository** và **tên nhánh hiện tại**.
- Bước 4: ứng dụng mở đúng repository **cha** (a), **không** phải thư mục con (b). Tên
  repo hiển thị là tên của (a).
- Bước 5: danh sách gần đây có **đúng một** mục cho repository đó — hai lần mở (a) và (b)
  gộp lại thành một, vì cả hai phân giải về cùng một `RepoId`.
- Bước 7: mục đó **vẫn còn** sau khi khởi động lại.
- Bước 8: repository mở ra bằng **đúng một lần bấm**.

### Trượt khi

- Bước 4 mở ra thư mục con như thể nó là một repository riêng.
- Bước 5 có **hai** mục cho cùng một repository (phép khử trùng lặp không khớp
  `state::repo_id_for` bên Rust).
- Bước 7 danh sách **rỗng** → đây là lỗi bền vững, xem mục Chẩn đoán ngay dưới.

### Chẩn đoán nếu danh sách rỗng sau khi khởi động lại

Kiểm theo thứ tự này. **Kiểm quyền trước khi nghi mã frontend** — Tauri v2 hỏng
**im lặng ở thời điểm chạy** khi thiếu quyền capability, không có lỗi nào hiện ra.

1. `src-tauri/capabilities/default.json` còn cấp `store:default` không?
2. Tệp `recent-repos.json` có tồn tại không, và ở đâu? Đường dẫn dự kiến:
   ```
   %APPDATA%\dev.gitplum.desktop\recent-repos.json
   ```
   (`identifier` trong `src-tauri/tauri.conf.json` là `dev.gitplum.desktop`.)
   Nếu tệp **không có** → lời gọi `set` không tới được đĩa.
   Nếu tệp **có mà rỗng hoặc thiếu mục** → `autoSave` chưa kịp đẩy xuống đĩa trước khi
   tiến trình thoát.
3. Chỉ khi hai bước trên đều sạch thì mới xem `src/lib/recentRepos.ts`.

---

## KB-2 — Bố cục còn nguyên sau khi khởi động lại

**Tiêu chí thành công 2 · PLAT-09 · đây là G3**

### Các bước

1. Mở repository **(a)** để ba vùng có nội dung.
2. Kéo thanh dọc giữa vùng **"Nhánh"** (sidebar) và vùng giữa sang một vị trí **rõ ràng
   khác mặc định** — ví dụ kéo cho sidebar rộng gấp đôi. Đừng kéo hai ba pixel; phải
   lệch đủ để mắt nhận ra ngay.
3. Kéo thanh ngang giữa vùng trên và bảng **nhật ký lệnh** lên cao hẳn, cho bảng nhật ký
   chiếm khoảng một nửa cửa sổ.
4. **Chụp ảnh màn hình.** Đây là mốc so sánh, đừng dựa vào trí nhớ.
5. **Thoát hẳn ứng dụng** (xem mục Chuẩn bị 4).
6. Mở lại từ Explorer.
7. So với ảnh chụp ở bước 4.

### Đỗ khi

**Cả hai** kích thước còn đúng như trước khi thoát — cả thanh dọc (nhóm
`git-plum-inner`) và thanh ngang (nhóm `git-plum-outer`).

### Trượt khi

Bất kỳ vùng nào nhảy về `defaultSize`:

| Nhóm | Vùng | `defaultSize` |
|---|---|---|
| `git-plum-inner` | sidebar / main / detail | 20% / 52% / 28% |
| `git-plum-outer` | top / bottom | 70% / 30% |

Thấy đúng các con số này sau khi mở lại nghĩa là bố cục đã lưu **không** được đọc lại.

> Phải kiểm **cả hai** nhóm. Chỉ kéo một thanh rồi kết luận là bỏ sót một nửa tính năng.

---

## KB-2b — Trạng thái bật tắt bảng nhật ký

**PLAT-09, phần còn thiếu · đây là G8 · bước bắt buộc, không được bỏ**

### Vì sao bước này bắt buộc

`src/components/AppLayout.tsx` chỉ render `Group` ngoài — thứ mang `useDefaultLayout` của
`git-plum-outer` — khi prop `bottom` khác `undefined`:

```tsx
if (!bottom) {
  return <div className="layout">{panes}</div>
}
```

Tắt bảng nhật ký là gỡ luôn `Group` đó khỏi cây React. Kịch bản nào **không** đụng tới nút
bật tắt sẽ **đỗ nhầm** trong khi một nửa tính năng vẫn hỏng. KB-2 một mình không phát
hiện được điều này.

### Các bước

1. Làm xong **KB-2** ở trên (ứng dụng đang mở, bố cục đã kéo).
2. Bấm nút **"Nhật ký lệnh"** trên thanh công cụ để **tắt** bảng nhật ký.
   (Tương đương phím `Ctrl+` `` ` `` qua lệnh `view.toggleCommandLog`.)
3. Xác nhận bảng nhật ký đã **biến mất**.
4. **Thoát hẳn ứng dụng** (kiểm Task Manager như mục Chuẩn bị 4).
5. Mở lại từ Explorer.

### Đỗ khi

Bảng nhật ký **vẫn ở trạng thái tắt** như lúc thoát.

### Trượt khi

Bảng nhật ký **bật trở lại**.

### Dự báo: bước này nhiều khả năng sẽ TRƯỢT

Đã kiểm chứng trực tiếp trong mã, không phải phỏng đoán. `src/App.tsx`:

```tsx
const [logVisible, setLogVisible] = useState(true)
```

State React thuần, không ghi vào đâu cả — không `localStorage`, không store. Mỗi lần khởi
động luôn quay về `true`, tức luôn bật.

**Nếu trượt, đó là phát hiện đúng, không phải lỗi thao tác.** Ghi vào `VERIFICATION.md`
thành một khoảng cách của PLAT-09 kèm nguyên nhân đã biết này, rồi **dừng lại hỏi chủ
dự án** thay vì tự sửa — đây là quyết định phạm vi, không phải lỗi cài đặt.

---

## Chẩn đoán nếu KB-2 hoặc KB-2b trượt

Ba nhánh, ba cách sửa khác nhau. Mở DevTools của webview trước
(`Ctrl+Shift+I`, hoặc chuột phải → Inspect nếu bản dựng cho phép).

### Nhánh 1 — `localStorage` không có khoá nào

Trong DevTools → Application → Local Storage, tìm khoá chứa `git-plum-outer` và
`git-plum-inner`.

**Không có khoá** → `useDefaultLayout` không ghi gì cả. Kiểm xem hook có thật sự được
truyền vào `Group` qua spread (`{...outerLayout}`) không.

### Nhánh 2 — Có khoá mà bố cục vẫn nhảy về mặc định

Nó **ghi** được nhưng **không đọc lại** lúc gắn kết. Kiểm giá trị trong khoá có hợp lệ
không, và thứ tự khởi tạo hook so với lần render đầu.

### Nhánh 3 — Có khoá, đọc lại được, nhưng bố cục lệch

Kiểm tham số **`panelIds`** của `useDefaultLayout`. Tài liệu
`react-resizable-panels` v4 cảnh báo tường minh: với `Group` chứa `Panel` render **có điều
kiện**, `panelIds` phải khớp đúng các `Panel` được render lúc gắn kết, nếu không bố cục
lần đầu sẽ sai.

`AppLayout` chính là trường hợp đó — `Panel id="bottom"` render có điều kiện — và hiện
**không** truyền `panelIds`. Đây là nguyên nhân khả dĩ nhất nếu phần kiểm bố cục trượt.

> Ghi chú cho Phase 2: tham số `groupId` của `useDefaultLayout` **đã bị đánh dấu
> deprecated** trong v4, thay bằng `id`. Mã hiện dùng `groupId`, vẫn chạy nhưng nên đổi.

---

## KB-3 — Nhật ký lệnh

**Tiêu chí thành công 3 · PLAT-08**

### Các bước

1. Sau các thao tác của KB-1 và KB-2, bật bảng nhật ký lệnh (nút **"Nhật ký lệnh"**).
2. Đọc từng dòng trong bảng.

### Đỗ khi

- Bảng liệt kê **đúng** từng lệnh git ứng dụng đã chạy. Với một lần mở repository, dự kiến
  thấy:
  - `git rev-parse --show-toplevel`
  - `git rev-parse --abbrev-ref HEAD`
  - `git --version` (chạy một lần lúc khởi động, hiện ở thanh trạng thái)
- Mỗi dòng có **mã thoát** và **thời gian chạy tính bằng mili giây**.
- Số dòng **khớp** số thao tác đã làm: không thiếu dòng, không có dòng lạ.

### Trượt khi

- Thiếu lệnh đã chạy, hoặc có dòng cho lệnh chưa từng chạy.
- Thiếu mã thoát hoặc thiếu thời gian chạy.

---

## KB-4 — Thư mục không phải repository

**Tiêu chí thành công 4 · PLAT-10**

### Các bước

1. Bấm **"Mở repository"**.
2. Chọn thư mục **(c)** — thư mục không phải repository.
3. Đọc banner lỗi.
4. Thử tiếp một thao tác khác (mở lại thư mục (a)) để xác nhận ứng dụng chưa chết.
5. Mở bảng nhật ký lệnh, tìm dòng của lệnh vừa thất bại.

### Đỗ khi

- Banner lỗi hiện thông điệp **đọc hiểu được bằng tiếng Việt**:
  ```
  Không phải một repository git: <đường dẫn>
  ```
  (Nguồn: `src-tauri/src/error.rs` — biến thể `GitError::NotARepository`.)
- Thông điệp **kèm lệnh git đã chạy**.
- Ứng dụng **vẫn thao tác tiếp được** — bước 4 thành công.
- Dòng lệnh thất bại xuất hiện trong nhật ký với **mã thoát 128**.

### Trượt khi

- Ứng dụng treo hoặc đóng.
- Không hiện gì cả (thất bại im lặng).
- Hiện **stderr thô chưa gói** — ví dụ dán thẳng
  `fatal: not a git repository (or any of the parent directories): .git` ra màn hình.

---

## KB-4b — 🔴 Repo THẬT bị git từ chối (khoảng cách đã biết, chưa sửa)

**Tiêu chí thành công 4 · PLAT-10 · thêm 2026-09-22**

KB-4 chỉ kiểm thư mục **không** phải repository. Ca này ngược lại: một repository
**hợp lệ** mà git từ chối mở. Nó vẫn đi vào cùng một biến thể lỗi, nên thông báo
**sai**, và KB-4 không bắt được vì KB-4 coi thông báo đó là đúng.

### Vì sao ca này tồn tại

`open_repository` (`src-tauri/src/commands/repo.rs:41`) rẽ **mọi** `!is_success()` của
`rev-parse --show-toplevel` sang `GitError::NotARepository`. Biến thể đó chỉ mang
`path`. Nhưng exit khác 0 của `rev-parse` có **nhiều hơn một** nguyên nhân, và ít nhất
một nguyên nhân là một repo hoàn toàn bình thường.

### Dựng điều kiện

Đã có sẵn trên máy này — nhiều repo trong `D:/MyCompanyProjects/` thuộc một SID
Windows khác. Kiểm bằng dòng lệnh trước:

```bash
git -C D:/MyCompanyProjects/cocos-engine rev-parse --git-dir; echo "exit=$?"
# exit=128
# fatal: detected dubious ownership in repository at '...'
# To add an exception for this directory, call:
#     git config --global --add safe.directory D:/MyCompanyProjects/cocos-engine
```

Không có repo nào như vậy thì dựng bằng cách chép một repo sang thư mục thuộc người
dùng khác, hoặc tạm đặt `[safe] directory` thành một giá trị không khớp.

### Các bước

1. Bấm **"Mở repository"**, chọn repo mà dòng lệnh vừa cho exit 128.
2. Đọc banner lỗi.
3. Mở bảng nhật ký lệnh, tìm dòng thất bại.

### Hiện trạng (đo 2026-09-22, **chưa sửa**)

- Banner nói **"Không phải một repository git: &lt;đường dẫn&gt;"** — **sai**, đó *là*
  một repository.
- Câu `git config --global --add safe.directory …` mà git in ra — câu **duy nhất** cho
  người dùng biết cách sửa — **không** tới giao diện: `NotARepository` không mang
  `stderr`.
- `stderr` **có** vào nhật ký lệnh (`open_repository` ghi `out.stderr_lossy()` khi
  thất bại), nên thông tin không mất hẳn — nhưng người dùng phải biết mở nhật ký, và
  banner thì đang dẫn sai hướng.

### Đỗ khi (sau khi sửa)

- Banner **phân biệt** được hai ca: thư mục không phải repo, và repo bị git từ chối.
- Với ca bị từ chối, giao diện nói ra **cách sửa**, không chỉ nói đã thất bại.
- KB-4 vẫn đỗ — nghĩa là sửa không làm ca "thư mục không phải repo" rơi vào nhánh mới.

### Ghi chú

Tìm được ngoài mọi cổng, lúc dò tệp lớn để đo hiệu năng Phase 3. Không sửa trong
Phase 3 vì đó là mã Phase 1 ngoài phạm vi. Ghi ở đây để nợ nằm cạnh tiêu chí mà nó
thuộc về, chứ không nằm rải trong tài liệu của phase khác.

---

## KB-5 — Không nháy cửa sổ console

**Tiêu chí thành công 5 · đây là G7**

### Hai lớp phòng thủ, ẩn hai cửa sổ khác nhau

Cả hai đều đã có trong mã. Hiểu rõ sự khác nhau vì nếu trượt thì phải biết kiểm chỗ nào:

| Lớp | Ở đâu | Ẩn cửa sổ của |
|---|---|---|
| 1 | `src-tauri/src/main.rs` — `windows_subsystem = "windows"` | **chính ứng dụng** |
| 2 | `src-tauri/src/git/exec.rs` — `CREATE_NO_WINDOW` (`0x0800_0000`) qua `cmd.creation_flags(...)` trong khối `#[cfg(windows)]` | **từng tiến trình git con** |

Lớp 2 mới là thứ KB-5 kiểm. Lớp 1 ẩn console của ứng dụng nhưng **không** ẩn console mà
`CreateProcess` cấp cho `git.exe`. Phải thấy **cả hai** cùng im lặng.

### Các bước

1. Bấm đúp `src-tauri/target/release/git-plum.exe` **từ Explorer**. Không chạy từ
   terminal, không chạy bản `--debug`.
2. Thực hiện **liên tiếp ít nhất 10 thao tác sinh tiến trình git**:
   - mở repository (a)
   - đóng
   - mở lại từ danh sách gần đây
   - đóng, mở lại — lặp cho đủ số
   - làm mới bảng nhật ký lệnh nhiều lần
3. Quan sát nghiêm túc. Hai cách, nên làm cả hai:
   - **Quay màn hình** rồi xem lại **từng khung hình**. Một cửa sổ nháy 30ms lọt qua mắt
     nhưng không lọt qua bản ghi 60fps.
   - Chạy thao tác lặp **nhanh** và nhìn **thanh tác vụ**, xem có biểu tượng
     **Console Window Host** (`conhost.exe`) xuất hiện chớp nhoáng không.

### Đỗ khi

**Không** có cửa sổ đen nào nháy lên, kể cả trong một phần nhỏ của giây, qua toàn bộ 10+
thao tác.

### Trượt khi

Thấy bất kỳ cửa sổ console nào — dù chỉ một khung hình trong bản ghi.

Nếu trượt, kiểm theo thứ tự:

1. Khối `#[cfg(windows)]` trong `src-tauri/src/git/exec.rs` còn gọi
   `cmd.creation_flags(CREATE_NO_WINDOW)` không, và hằng có đúng `0x0800_0000` không.
2. Dòng `windows_subsystem` trong `src-tauri/src/main.rs` còn nguyên không.
3. Xác nhận lại đang chạy bản **release** chứ không phải `--debug` — đây là lỗi dễ mắc
   nhất và nó làm phép kiểm vô nghĩa theo hướng ngược lại (báo trượt oan nếu chạy debug).

---

## KB-6 — QA thủ công dưới locale hệ thống

**Phần còn lại của validation checkpoint #7 · xem thêm Phụ lục A**

### Các bước

1. Với bản release đang chạy, chạy lại **KB-1 tới KB-4**.
2. Ở **KB-4**, nhìn kỹ nguồn gốc của thông điệp lỗi.

### Đỗ khi

Thông điệp lỗi ở KB-4 là thông điệp tiếng Việt **do git-plum viết**:

```
Không phải một repository git: <đường dẫn>
```

Chuỗi này nằm trong `src-tauri/src/error.rs`, do dự án tự viết.

### Trượt khi

Thấy chữ **tiếng Việt lạ đến từ chính git** — tức git tự dịch đầu ra của nó. Nghĩa là
`LC_ALL=C` **không** tới được tiến trình con. Đó là khoảng cách thật, ghi lại.

> Xem **Phụ lục A** để biết locale thật của máy này và vì sao phép kiểm trên máy hiện tại
> yếu hơn dự kiến ban đầu.

---

## Soát REL-04 không chạy CI

**Đây là mức diễn giải YẾU HƠN ý định ban đầu của ROADMAP**, theo quyết định ngày
2026-09-21 của chủ dự án: chưa tạo remote, nên CI không chạy được, nên REL-04 trong Phase 1
chỉ đạt tới mức **"tệp `ci.yml` viết đúng"** — không phải "CI đã chạy xanh".

Rủi ro đã biết và chấp nhận: lỗi riêng của Linux (đặc biệt phiên bản WebKitGTK) và của
macOS sẽ chỉ lộ ra khi tới việc đóng gói phát hành ở Phase 8, lúc đó sửa đắt hơn.

Vì chưa chạy được, những mục dưới đây phải **soát bằng mắt** trong
`.github/workflows/ci.yml`:

| Mục phải soát | Giá trị đúng | Vì sao dễ sai |
|---|---|---|
| Gói WebKit trên Linux | `libwebkit2gtk-4.1-dev` | **Không phải 4.0**. Tauri v2 cần 4.1; đây là nguyên nhân hỏng CI Linux hay gặp nhất khi chuyển từ v1 sang v2 |
| Runner Linux | `ubuntu-22.04` | **Không phải `ubuntu-latest`** — bản nâng cấp runner của GitHub sẽ làm vỡ build vào một sáng nào đó |
| Thư mục làm việc cargo | `working-directory: src-tauri` ở **mọi** bước cargo | Crate Rust không nằm ở gốc repo; thiếu dòng này thì cargo không tìm thấy `Cargo.toml` |
| Job `frontend` | chạy `npm test` | Trước plan 01-02 job này chạy vào khoảng không (không có tệp cấu hình vitest nào). **Nay đã có 57 test thật để chạy** |
| Ma trận ba nền tảng | `ubuntu-22.04`, `windows-latest`, `macos-latest` | REL-04 đòi cả ba |

Khi nào có remote thì chạy CI là xong ngay — tệp cấu hình đã sẵn sàng.

---

## Phụ lục A — Kết quả checkpoint #7 (locale)

**Chạy:** 2026-09-21 · **Máy:** Windows 11 Pro 10.0.22621

### A.1 Locale thật của máy — chép nguyên đầu ra

**Phát hiện quan trọng: máy này KHÔNG phải Windows tiếng Việt.** Kế hoạch 01-04 và
`CONTEXT.md` (G6) đều giả định "máy phát triển là Windows tiếng Việt, nên kiểm được ngay".
Giả định đó **sai**. Đo thật:

```
Get-Culture Name              = en-US
Get-Culture DisplayName       = English (United States)
CurrentUICulture              = en-US
CurrentUICulture DisplayName  = English (United States)
chcp                          = Active code page: 437
```

Kiểm chéo bằng ba nguồn độc lập khác, tất cả đồng thuận:

```
Get-WinUserLanguageList       = en-US          (chỉ một ngôn ngữ duy nhất)
Get-WinSystemLocale           = en-US
Get-WinHomeLocation           = United States
HKCU\Control Panel\International → LocaleName = en-US, sLanguage = ENU
systeminfo → System Locale    = en-us;English (United States)
systeminfo → Input Locale     = en-us;English (United States)
```

Không có gói ngôn ngữ hiển thị tiếng Việt nào được cài. `vi` và `vi-VN` tồn tại như
culture của .NET nhưng đó chỉ là dữ liệu định dạng, không phải ngôn ngữ giao diện của hệ
điều hành.

### A.2 Kết quả bộ kiểm thử dưới locale của máy

| Lệnh | Kết quả | Số liệu |
|---|---|---|
| `cargo test` (trong `src-tauri`) | ✅ đỗ | **23 passed**, 0 failed, 3 suite, 0.15s |
| `cargo clippy --all-targets -- -D warnings` | ✅ sạch | không có cảnh báo |
| `cargo fmt --all --check` | ✅ sạch | exit 0 |
| `npm run typecheck` | ✅ sạch | exit 0 |
| `npm test` | ✅ đỗ | **57 passed**, 5 tệp, 1.27s |
| `npm run build` | ✅ đỗ | `index-BMeG-kyD.js` **273.83 kB** (gzip 85.96 kB), CSS 5.69 kB (gzip 1.74 kB) |

Phiên bản công cụ: `cargo 1.98.1`, `rustc 1.98.1`, `node v22.16.0`, `npm 10.9.2`,
`git 2.54.0.windows.1`, `vitest 5.0.1`, `vite 8.3.0`.

### A.3 Kiểm chéo bằng cách ép ngược locale — phần chứng minh thật

Chỉ chạy dưới một locale thì **không phân biệt được** "lớp bọc ghim locale đúng" với
"máy này tình cờ hợp". Nên chạy `cargo test` hai lần, lần thứ hai ép biến môi trường của
**tiến trình cha** sang tiếng Việt:

```
# Lần 1 — bình thường
cargo test
→ test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

# Lần 2 — ép locale ở tiến trình cha
export LC_ALL=vi_VN.UTF-8 LANG=vi_VN.UTF-8 LC_MESSAGES=vi_VN.UTF-8
cargo test
→ test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s
```

**Kết quả: 23 = 23, giống hệt.** Đây là bằng chứng `apply_env_hardening` ghi đè được biến
môi trường thừa hưởng. Nếu lớp bọc không ghim `LC_ALL=C`, biến của tiến trình cha sẽ lọt
xuống git con và test đọc đầu ra git sẽ lệch.

Xác nhận lớp ghim có thật trong mã — `src-tauri/src/git/exec.rs`:

```rust
cmd.env("LC_ALL", "C");        // dòng 191
cmd.env("LANG", "C");          // dòng 192
cmd.env("LC_MESSAGES", "C");   // dòng 193
```

### A.4 Giới hạn của phép kiểm này — đọc kỹ trước khi kết luận

Hai giới hạn thật, ghi lại để không ai đọc mục A.3 rồi tưởng checkpoint #7 đã đóng trọn.

**Giới hạn 1 — máy là `en-US`, không phải tiếng Việt.** ROADMAP đòi "chạy bộ kiểm thử và
QA thủ công với locale **hệ thống** không phải tiếng Anh". Phép kiểm ở A.3 ép locale qua
**biến môi trường POSIX** (`LC_ALL`/`LANG`/`LC_MESSAGES`), không đổi locale **hệ thống**
của Windows. Đây là hai thứ khác nhau: cái sau đòi cài gói ngôn ngữ hiển thị và đăng nhập
lại.

**Giới hạn 2 — bản Git for Windows này không có bản dịch nào.** Đo thật:

```
Số tệp *.mo trong C:\Program Files\Git\mingw64 = 0
```

Và kiểm bằng hành vi, trong một thư mục không phải repository:

```
git rev-parse --show-toplevel
→ fatal: not a git repository (or any of the parent directories): .git

LC_ALL=vi_VN.UTF-8 LANG=vi_VN.UTF-8 LC_MESSAGES=vi_VN.UTF-8 git rev-parse --show-toplevel
→ fatal: not a git repository (or any of the parent directories): .git
```

Đầu ra **y nguyên tiếng Anh** kể cả khi ép locale. Nghĩa là trên máy này git **không thể**
dịch đầu ra dù có muốn — không có catalog để dịch.

**Hệ quả cho cách đọc kết quả.** Phép kiểm A.3 chứng minh đúng một điều, và đó là điều
đáng giá: lớp bọc **ghim được** biến môi trường, tiếng Việt của tiến trình cha **không**
lọt xuống tiến trình con. Nhưng nó **chưa** chứng minh được bộ phân tích sống sót khi git
thật sự nói một ngôn ngữ khác, bởi vì trên máy này không có cách nào làm git nói tiếng
khác. Rủi ro còn lại: một người dùng có Git bản dịch đầy đủ cộng locale hệ thống không
phải tiếng Anh vẫn có thể gặp lỗi mà phép kiểm này không bắt được — nhưng chính lớp ghim
`LC_ALL=C` đã chứng minh ở A.3 là thứ bảo vệ họ.

**Kết luận cho checkpoint #7: ĐẠT MỘT PHẦN.**

- ✅ Lớp ghim môi trường **có** và **chứng minh được** hoạt động (A.3) — đây là phần
  quan trọng nhất, và chi phí sửa nó thấp vì nằm ở một hàm trung tâm.
- ⚠️ Chưa kiểm được trên locale **hệ thống** không phải tiếng Anh với một bản git **có
  bản dịch**. Cần một máy hoặc máy ảo có gói ngôn ngữ khác và Git for Windows bản đầy đủ.

---

*Phase: 01-platform-git-layer · Plan 01-04*
