# Khoảng cách giữa giao diện hiện tại và ảnh tham chiếu

**Ngày:** 2026-09-21

> ## ⛔ ĐÃ ĐÓNG — phần còn lại KHÔNG LÀM (quyết định 2026-09-21)
>
> Chủ dự án quyết định **không làm tiếp** phần giống tham chiếu còn lại: toolbar
> (Undo/Redo/Pull/Push/Branch/Stash/Pop), tab repo nhiều repo, sidebar có icon và gập,
> panel phải đầy đủ (avatar, `N modified`, `View all files`, icon màu theo trạng thái
> tệp), nhãn thời gian tương đối dạng bong bóng, thanh trạng thái đầy đủ hơn.
>
> Tài liệu này giữ lại làm **ghi chép khảo sát**, không phải kế hoạch. Không lập phase UI
> riêng. Mục 4 ("Điều cần quyết trước khi lập phase UI") vì vậy cũng không còn hiệu lực.
>
> **Đã làm và giữ lại** (commit `544ae5f`, `157b398`): thứ tự cột NHÃN → ĐỒ THỊ → THÔNG
> ĐIỆP, header cột, nút commit vòng viền dày (merge lớn hơn, có tâm đặc), đường lane 2px,
> `LANE_WIDTH` 22px kèm cap lane 13, nền cột đồ thị, hàng xen kẽ, đường nối nhãn, phần
> body xám nối sau subject.

**Bối cảnh ban đầu:** Người dùng đối chiếu app sau wave 6 với `docs/screenshots/` và kết
luận "không giống thiết kế". Quyết định lúc đó: đóng wave 6 trước (chỉ cần 3 lỗi
checkpoint đã hết), việc dựng lại layout theo tham chiếu tách thành phase riêng — **phần
tách ra đó nay đã bị bỏ, xem khung trên**.

Xem thêm `.planning/PROJECT.md` mục Constraints về quyết định sao chép màu/icon/font của
GitKraken và rủi ro giấy phép kèm theo (commit `5417400`). Quyết định bỏ phần UI còn lại
**không** xoá bỏ ghi chú rủi ro đó: những gì đã sao chép (bố cục cột, hình dạng nút, cách
vẽ đường gập vuông) vẫn nằm trong mã.

---

## 1. Ảnh tham chiếu nào nói gì

| Ảnh | Nội dung | Thuộc phase |
|---|---|---|
| `main-1.png` | Màn lịch sử: header cột, cột BRANCH/TAG, graph, message + body, sidebar, toolbar, panel phải | **Phase 2 + phase UI mới** |
| `main-2.png` | Màn diff viewer: File View / Diff View, Revert Hunk, Blame/History, số dòng hai bên | **Phase 3** (chưa làm) |
| `main-3.png` | (chưa phân tích) | — |
| `main-4.png` | (chưa phân tích) | — |

---

## 2. Khoảng cách trên màn lịch sử (`main-1.png`)

Đối chiếu ảnh tham chiếu với app sau commit `970e31c`.

### 2.1 Cấu trúc cột — SAI THỨ TỰ

Tham chiếu, từ trái sang phải:

```
BRANCH / TAG  |  GRAPH  |  COMMIT MESSAGE
```

App hiện tại:

```
GRAPH  |  nhãn ref  |  message  |  tác giả  |  thời gian  |  sha
```

Hai khác biệt:

1. **Cột nhãn phải nằm TRƯỚC graph**, không phải sau. Tham chiếu đặt `BRANCH / TAG` ở
   ngoài cùng bên trái, graph ở giữa. Wave 6 đã tách nhãn thành cột riêng (đúng hướng)
   nhưng đặt sai vị trí — sau graph.
2. **Không có header cột.** Tham chiếu có dải header `BRANCH / TAG` / `GRAPH` /
   `COMMIT MESSAGE` cố định trên đầu danh sách. App không có gì.

### 2.2 Message thiếu phần body

Tham chiếu hiện **subject đậm rồi body xám nhạt ngay sau, cùng một dòng**:

```
refactor(description): dùng chung DescriptionArea cho ChooseAnswersImage   ChooseAnswersImage tự dựng lại nhánh switch theo dataType để render | AudioViewer — đúng việc DescriptionArea đã làm. Bỏ …
```

App hiện tại chỉ hiện `subject`. `Commit.body` đã có sẵn từ wave 2 (`LOG_FORMAT` có `%b`)
nhưng chưa dùng ở tầng hiển thị.

### 2.3 Thiếu toolbar

Tham chiếu có dải toolbar ngang: **Undo · Redo · Pull · Push · Branch · Stash · Pop ·
Terminal**, mỗi cái một icon, và nhóm **Actions · Search** bên phải. Có cả dropdown mũi
tên nhỏ cạnh Pull.

App hiện tại chỉ có ba nút chữ: `Mở repository` / `Đóng` / `Nhật ký lệnh`.

Lưu ý phạm vi: phần lớn các lệnh này (Pull/Push/Branch/Stash/Pop) là **Phase 4** (lệnh
ghi). Phase UI mới nên dựng **khung toolbar** với các nút vô hiệu hoá, để Phase 4 chỉ cần
nối hành động vào chứ không phải dựng lại layout.

### 2.4 Thiếu tab repo

Tham chiếu có tab ở trên cùng (`thithu-web ×`) kèm nút `+` để mở repo khác, và nút
`Launchpad`. App hiện tại hiện tên repo dạng chữ tĩnh trên một dòng.

Hạ tầng đã sẵn: `AppState.open_repos` là `HashMap<RepoId, _>` từ ngày đầu (PLAT-03), nên
nhiều repo cùng mở đã hỗ trợ ở tầng Rust. Chỉ thiếu UI.

### 2.5 Sidebar thiếu icon và cấu trúc gập

Tham chiếu:

```
> 🖥  LOCAL        2
> ☁  REMOTE       3
> ⚙  WORKTREES    1
> ≡  ISSUES
> 👥 TEAMS
> 🏷  TAGS        12
```

Mỗi nhóm có mũi tên gập, icon, tên in hoa, số đếm bên phải. Có cả nhóm app chưa có
(`WORKTREES`, `ISSUES`, `TEAMS`, `STASHES`).

App hiện tại: `NHÁNH LOCAL (3)` dạng chữ, không icon, không gập được.

### 2.6 Panel phải thiếu nhiều thành phần

Tham chiếu có:

- Dòng trên: `commit: d32b46` + nút `Recompose commit with AI`
- Tiêu đề commit cỡ lớn
- Body commit đầy đủ, có phân đoạn
- **Avatar tác giả** (ảnh vuông màu), tên, `authored <ngày> @ <giờ>`, `parent: b2e758`
  bên phải
- `✏ 3 modified` — số tệp đổi kèm icon
- Nút chuyển `≡ Path` / `⊞ Tree`, ô tick `View all files`, nút sắp xếp `⇅`
- Danh sách tệp: mỗi dòng có icon bút chì **màu**, đường dẫn xám + tên tệp trắng đậm

App hiện tại có: tác giả, thời gian, mã commit, cha, danh sách tệp, nút `Xem dạng phẳng`.
Thiếu: avatar, `N modified` có icon, `View all files`, nút sắp xếp, icon màu theo trạng
thái tệp, và phân biệt đường dẫn/tên tệp bằng màu.

### 2.7 Thiếu thanh trạng thái dưới

Tham chiếu: `⚑ No Pull Requests` bên trái; bên phải các icon chế độ xem, `🔍 100%`,
`Support`, badge `PRO`, số phiên bản `12.5.0`.

App hiện tại: `git version 2.54.0.windows.1` bên trái, `git-plum 0.1.0` bên phải. Đúng
tinh thần nhưng ít thành phần hơn.

---

## 3. Việc thuộc phase nào

| Việc | Phase |
|---|---|
| Đảo cột nhãn ra trước graph, thêm header cột | Phase UI mới |
| Message có body xám sau subject | Phase UI mới |
| Khung toolbar (nút vô hiệu hoá) | Phase UI mới |
| Tab repo nhiều repo | Phase UI mới (backend đã sẵn từ PLAT-03) |
| Sidebar icon + gập + số đếm + nhóm mới | Phase UI mới |
| Panel phải: avatar, N modified, View all files, icon màu | Phase UI mới |
| Thanh trạng thái đầy đủ hơn | Phase UI mới |
| Nối hành động Pull/Push/Branch/Stash/Pop vào toolbar | **Phase 4** (lệnh ghi) |
| Màn diff `main-2.png` (File/Diff View, Revert Hunk, Blame) | **Phase 3** (diff) |
| Nhóm ISSUES / TEAMS trong sidebar | **chưa có trong phạm vi v1** — cần quyết định |

---

## 4. Điều cần quyết trước khi lập phase UI

1. **Nhóm `ISSUES` và `TEAMS`** trong sidebar tham chiếu là tính năng đám mây của
   GitKraken (kết nối GitHub/GitLab). `PROJECT.md` mục "Không làm" ghi rõ **không** làm
   đồng bộ đám mây / tài khoản người dùng. Nên bỏ hai nhóm này, hoặc xác nhận lại.
2. **Nút `Recompose commit with AI`** ở panel phải trùng với tính năng AI soạn commit
   message của v1 — nhưng ở đây nó áp cho commit **đã có**, không phải commit đang soạn.
   Cần xác định có làm hay không.
3. **Badge `PRO`, `Start a Trial`, `Support`** là thành phần thương mại của GitKraken.
   Không sao chép.
4. **Bảng màu/icon/font**: người dùng đã chọn sao chép sát (xem `PROJECT.md`). Cần lấy
   mã màu thật từ ảnh tham chiếu và chọn bộ icon tương đương.
