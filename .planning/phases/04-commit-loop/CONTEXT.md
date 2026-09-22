# Phase 4: Vòng lặp commit theo tệp — Context

**Ngày:** 2026-09-22
**Mode:** mvp (slice dọc theo tính năng, không tầng ngang)
**Goal (ROADMAP):** Người dùng hoàn thành được một vòng làm việc thật — xem thay đổi,
chọn tệp, viết thông điệp, tạo commit — mà không rời khỏi git-plum.
**Requirements:** WORK-01, WORK-02, WORK-08, WORK-09, WORK-10, **WORK-11**

---

## 0. Trạng thái khi phase này bắt đầu — đọc trước mọi thứ khác

Phase 4 bắt đầu **trên nợ kiểm chứng của ba phase trước**, và điều đó đổi cách lập kế
hoạch, không chỉ là một ghi chú.

| Phase | Mã | Cổng người-kiểm | Trạng thái |
|---|---|---|---|
| 1 | xong | 3 tiêu chí (`docs/03-phase1-qa-windows.md`) | **chưa kiểm** |
| 2 | 7/7 plan | checkpoint 12 bước (02-06) | **vòng 1 bị từ chối**, đã sửa, chờ vòng 2 |
| 2 | — | checkpoint #1 phần người thấy (02-07) | **thiếu** (phần Rust đã đo: 787 ms/1000 ms) |
| 3 | 5/5 plan | checkpoint 11 bước (03-04) | **chưa chạy** |
| 3 | — | **exit gate dogfood** (03-05) | **chưa chạy** |

Chủ dự án đã chọn (2026-09-22): chạy `--from 4`, **dừng và báo** ở cổng bắt buộc, nợ
Phase 1–3 giữ nguyên và ghi nhận. Quyết định đó là của chủ dự án; tài liệu này không
lật nó, nhưng phải ghi **hệ quả kỹ thuật** của nó:

> **WORK-01 đọc trực tiếp trình xem diff của Phase 3, vốn chưa ai dùng thật.** Chủ dự
> án đã tìm **năm** lỗi hiển thị của trình xem đó chỉ bằng cách mở ứng dụng, và cả năm
> đều đi qua 435 test tự động. Nghĩa là xác suất còn lỗi hiển thị chưa biết trong thứ
> Phase 4 xây lên trên là **cao**, không phải giả định bi quan.
>
> **Hệ quả cho việc lập kế hoạch:** mọi plan của Phase 4 phải tự đứng được khi trình
> xem diff hoá ra còn lỗi. Cụ thể: danh sách tệp (WORK-01) và vòng commit (WORK-09)
> **không** được phụ thuộc vào việc diff render đúng — chọn một tệp mà diff hỏng thì
> vẫn phải stage/commit được tệp đó. Nếu một plan làm vòng commit chết theo trình xem
> diff, plan đó sai.

### Ba mâu thuẫn tài liệu đã biết, **không** sửa trong phase này

1. **`ROADMAP.md` mục Progress lạc hậu.** Nó ghi Phase 2 = `3/7` và Phase 3 = `4/5`;
   trên đĩa có **7** và **5** tệp SUMMARY, và cả hai `VERIFICATION.md` ghi 7/7 và 5/5
   wave đã thực thi. **Tin `VERIFICATION.md` và `git log`, không tin bảng Progress.**
   Không sửa vì `ROADMAP.md` đang có thay đổi chưa commit của một session khác; sửa sẽ
   chồng việc.
2. **`STATE.md`** có `last_updated` cũ ở một số chỗ và từng mô tả Phase 3 ở wave 4.
   Đã cập nhật phần Phase 2/nợ ở `46398a9`, phần còn lại vẫn có chỗ lạc hậu.
3. **Số test trong `STATE.md`** (212 frontend + 149 Rust) là số **lúc đóng Phase 2**.
   Số hiện tại: **435 frontend + 284 Rust (+1 ignored)**. Đã chú thích, không viết lại.

### Hai lỗi đã đo, chưa sửa, **cả hai đụng vào Phase 4**

- **Chồng cột lane 19,99 %** (`docs/04-phase2-degraded-graph.md` mục 2.3). Trên repo
  100 007 commit thật, 19 995 hàng có `lane >= 13` nên `laneX` clamp chúng về cột 12 —
  tám lane vẽ đè một cột, **không có chỉ báo**. Ở cap 20 là 0,71 %. **Liên quan Phase 4
  ở WORK-11:** hàng WIP cần **một lane riêng** nối xuống HEAD; cấp lane đó ở đâu thì
  phải biết rằng cột 12 đã là chỗ chen chúc của 20 % hàng.
- **`open_repository` báo sai "Không phải một repository git"** khi git từ chối repo
  (`dubious ownership`, exit 128) — `docs/03-phase1-qa-windows.md` ca KB-4b. **Liên
  quan Phase 4:** watcher và `git status` sẽ gặp cùng lớp lỗi "exit khác 0 có nhiều hơn
  một nguyên nhân". Đừng lặp lại khuôn đó trong mã mới.

---

## 1. Phạm vi

### Trong phạm vi

| Req | Nội dung |
|---|---|
| **WORK-01** | Ba nhóm tệp: đã stage, chưa stage, chưa theo dõi. Chọn tệp → thấy diff của nó |
| **WORK-02** | Stage / unstage **theo tệp** |
| **WORK-08** | Soạn thông điệp và tạo commit |
| **WORK-09** | Sửa commit gần nhất (amend), cảnh báo nếu đã push nhưng **không chặn** |
| **WORK-10** | Thay đổi từ ngoài ứng dụng → giao diện tự cập nhật dưới 1 giây, không cần bấm |
| **WORK-11** | Hàng WIP ở đầu đồ thị, kèm số tệp sửa / tệp mới, bấm mở vùng soạn commit |

Cộng thêm, từ Success Criteria số 6 của ROADMAP: **bản nháp thông điệp còn nguyên** sau
khi đóng mở lại ứng dụng và sau khi chuyển repo.

### Ngoài phạm vi — để Phase 5

Staging **theo khối** (WORK-03…07), huỷ bỏ thay đổi có đường lùi, xử lý CRLF / thiếu
dòng cuối / byte không UTF-8 ở mức khối, phát hiện tệp đã đổi từ lúc vẽ diff. Phase 4
chỉ stage **cả tệp**.

---

## 2. Ràng buộc kỹ thuật ROADMAP đã chốt — không thương lượng

### 2.1 Lệnh trạng thái

`git status --porcelain=v2 --branch -z --untracked-files=all`

- **`--porcelain=v2`**, không `v1`: v2 mang XY, mode, SHA, và thông tin đổi tên trong
  cùng một dòng, nên không cần lệnh thứ hai để biết tệp đổi tên.
- **`--branch`** để lấy nhánh hiện tại, ahead/behind — cần cho cảnh báo amend của
  WORK-09 (đã push hay chưa).
- **`-z`** kết thúc bằng NUL: tên tệp có khoảng trắng, dấu ngoặc kép, ký tự không
  UTF-8. **Đây là chỗ Phase 2 và Phase 3 đã bị cắn hai lần** — xem mục 4.
- **`--untracked-files=all`**: liệt kê **từng** tệp chưa theo dõi, không gộp thư mục.
  Gộp thư mục thì số đếm của WORK-11 sai và người dùng không stage được một tệp trong
  thư mục mới.

### 2.2 Watcher — giới thiệu **ở phase này**, không sớm hơn

Đây là điểm đầu tiên mà **dữ liệu cũ thật sự nguy hiểm**: trước Phase 4 ứng dụng chỉ
đọc, đọc dữ liệu cũ thì sai mắt nhìn; từ Phase 4 ứng dụng **ghi**, và ghi dựa trên
trạng thái cũ thì hỏng repo của người dùng.

- Theo dõi **hẹp bên trong `.git`**: `HEAD`, `index`, `refs/**`, `packed-refs`, các tệp
  trạng thái merge/rebase.
- **Không bao giờ** theo dõi đệ quy thư mục làm việc không lọc. `node_modules` và
  `target/` ở repo này lần lượt hàng chục nghìn và hàng trăm nghìn tệp; một watcher
  không lọc sẽ ăn hết CPU và vi phạm ràng buộc RAM < 150 MB.
- **Gộp và trì hoãn 250–300 ms.** Một lần `git commit` chạm `index`, `HEAD` và
  `refs/heads/<nhánh>` — ba sự kiện cho một hành động.

### 2.3 `.git/index.lock` — không bao giờ tự xoá

Coi là trạng thái tạm thời có thể phục hồi; **thử lại có giãn cách**. Tự xoá
`index.lock` là cách làm hỏng repo khi có một tiến trình git khác đang chạy thật —
người dùng có thể đang `git rebase` ở terminal.

### 2.4 Hook

Chạy `pre-commit` và `commit-msg` **mặc định**. Có công tắc `no-verify` **tường minh**
ở giao diện. Đây là một trong những lý do dự án chọn `git` CLI thay vì libgit2.

### 2.5 Thao tác ghi trả trạng thái mới **trực tiếp**

Không chờ watcher. Watcher vẫn phát sự kiện để phòng thủ, nhưng đường chính là: ghi →
trả trạng thái mới → giao diện cập nhật. Chờ watcher làm giao diện trễ 250–300 ms sau
mỗi lần bấm, và nếu watcher chết thì giao diện đứng im mà không ai biết.

### 2.6 WORK-11 — ràng buộc riêng, đây là phần chịu lực

Nguyên văn ROADMAP, cộng ghi chú của tôi:

> Hàng WIP là hàng **ảo ở chỉ số 0**, không phải một commit — không có SHA, không nằm
> trong `historyStore.commits`, và **không được** đếm vào `total` của virtualizer theo
> cách làm lệch ánh xạ `commits[index]`.

**Vì sao đây là chỗ nguy hiểm nhất của phase.** `CommitList` có **đúng một** lời gọi
`useVirtualizer`, và `GraphCanvas` vẽ từ **cùng** mảng `virtualItems`. Đó là bất biến
cốt lõi của Phase 2 và nó tồn tại vì lớp lỗi "cột đồ thị lệch cột văn bản đúng một
hàng" **đã xảy ra hai lần** và được ghi trong `docs/` Phase 2.

Mọi phép cộng/trừ chỉ số phải nằm ở **một** chỗ. Hai chỗ là đủ để lệch.

Hàng WIP cũng cần **lane riêng** trong `GraphRow` nối xuống HEAD — thiếu nó thì nó là
một chấm trôi không có cạnh. Xem lại mục 0 về chồng cột trước khi chọn lane.

#### 🔴 Cạnh hàng WIP bị clamp khi HEAD ở lane ≥ 13 — **đã tái hiện, có số**

`git-plum-a3` nêu rủi ro này; tôi dựng repo và **tái hiện được**, nên nó không còn là
giả thuyết.

**Hai thứ khác nhau mà dễ lẫn:** hàng **0** và **HEAD**. `git log --all --topo-order`
xếp commit mới nhất theo topo của **mọi ref** ở hàng 0. Nếu người dùng đang ở một nhánh
**không** phải nhánh mới nhất, HEAD nằm **giữa** danh sách và lane của nó là bất kỳ.
Hàng WIP nối xuống **HEAD**, không nối xuống hàng 0.

Trên repo perf 100 007 commit thì HEAD tình cờ **trùng** hàng 0 (lane 0), nên phép đo
đó **không** phát hiện được vấn đề. Dựng repo 21 commit / 20 nhánh song song rồi
`git checkout br1` (nhánh **đầu**, không phải nhánh cuối):

```text
MAX_VISIBLE_LANES = 13, lane lớn nhất thật = 19
hàng có lane >= cap (bị clamp vào cột 12): 7 (33.3333%)
hàng 0: lane 0 (commit mới nhất theo topo của MỌI ref)
HEAD thật sự: hàng 19, lane 19 -> cạnh hàng WIP BỊ CLAMP, sẽ vẽ sai cột
```

`laneX` clamp lane 19 về cột 12, nên cạnh từ hàng WIP đi xuống **cột 12** trong khi
HEAD thật nằm ở lane 19 — người dùng thấy hàng WIP nối vào **một commit khác**. Đây
không phải suy giảm nhìn-là-thấy; nó là một đồ thị **vẽ sai một cách tự tin**.

**Điều kiện xảy ra:** ≥ 14 nhánh sống song song **và** HEAD không phải nhánh mới nhất
theo topo. Repo nhiều nhánh feature là ca thường, không dị biệt.

**Bắt buộc với plan:** phải có test ghim ca này — ít nhất 14 lane đồng thời, HEAD ở
lane ≥ 13, và khẳng định cạnh hàng WIP **không** vẽ vào cột 12 khi HEAD không ở lane 12.
Đo bằng `cargo run --release --bin lanedist <repo>`, đã in sẵn lane của HEAD.

Số đếm (tệp sửa / tệp mới) lấy từ **cùng** lời gọi `git status --porcelain=v2` của
WORK-01. **Không** thêm lệnh git thứ hai chỉ để đếm.

---

## 3. Bài học từ Phase 1–3 — áp dụng, không đọc rồi bỏ

Những chỗ dưới đây đã tốn thời gian thật. Mỗi mục là một ca kiểm bắt buộc, không phải
lời khuyên.

### 3.1 Cổng không thể fail — đã xảy ra **sáu** lần

| # | Phase | Cổng sai vì |
|---|---|---|
| 1 | 3 | grep khớp **chú thích và văn xuôi**, mã thật sạch |
| 2 | 3 | đòi **đúng 1** khớp grep, nhưng import + chỗ gọi luôn ≥ 2 |
| 3 | 3 | đường dẫn tệp sai → grep không tìm thấy gì → xanh |
| 4 | 3 | fixture **không phân biệt** được đột biến |
| 5 | 3 | grep trên nguồn **thô** khớp chú thích **do chính mutation sinh ra** |
| 6 | 3 | `waitFor` neo vào phần tử có mặt ở **mọi** trạng thái → thoả mãn tức thì |
| 7 | **4** | fixture có **đúng hình dạng** nhưng dữ liệu vô hại nên lệch nấc **không quan sát được** |

**Lỗi #7 — và đây là lần ĐẦU một executor tự bắt cổng của chính nó trước khi giao.**
Sáu lần trước đều do orchestrator hoặc chủ dự án tìm ra sau.

Wave 1 (04-01) viết một test cho mutation M1 (bản ghi dạng `2` chỉ tiêu thụ **một**
đoạn NUL thay vì hai). Fixture dùng đường dẫn cũ `a_old.txt` — đúng hình dạng plan
đòi: một bản ghi dạng `2` đứng trước ≥ 2 bản ghi khác. **M1 cho 0 đỏ.**

Vì sao: M1 làm đoạn đường-dẫn-cũ rò sang vòng lặp sau. Nhưng `a_old.txt` **không khớp
dạng bản ghi nào** (không có khoảng trắng sau ký tự đầu), nên nó rơi vào `_ => {}` và
bị **bỏ im lặng**. Bộ phân tích vẫn trả đúng 3 phần tử với đúng đường dẫn. Lệch nấc
xảy ra thật nhưng **không để lại dấu vết quan sát được**.

Khẳng định `old_path` **cũng không** bắt được — tôi đã kiểm: M1 chỉ bỏ bước *tiêu thụ*,
`duong_dan_cu` vẫn được đọc đúng nên `old_path` vẫn đúng.

Sửa: đường dẫn cũ phải **trông như một bản ghi** để đoạn rò trở thành phần tử rác
đếm được. `? cu.txt` làm được (git cho phép mọi byte trừ NUL và `/` trong tên tệp, nên
đây là dữ liệu hợp lệ). Sau sửa M1 cho **2 đỏ**, và output đỏ nói đúng cơ chế:

```
đường dẫn cũ `? cu.txt` phải được TIÊU THỤ như một phần của bản ghi đổi tên, không
được đọc lại thành một bản ghi dạng `?`.
Được: [("moi.txt", Staged), ("cu.txt", Untracked), ("sau.txt", Staged)]
  left: 3   right: 2
```

**Tôi kiểm chứng độc lập cả hai chiều:** đổi 9 chỗ `? cu.txt` về `a_old.txt` **trong
khi M1 vẫn áp dụng** → `242 passed, 0 đỏ`. Lỗi đi qua hoàn toàn. Với `? cu.txt` → 2 đỏ.

**Quy tắc rút ra, bổ sung cho mục này:** một fixture đúng *hình dạng* vẫn có thể không
phân biệt được nếu *dữ liệu* của nó vô hại. Hình dạng là điều kiện cần, không đủ. Phải
chạy đột biến và **thấy** đỏ, không phải lập luận rằng fixture bao được.

#### Lớp lỗi "hình dạng đúng, dữ liệu vô hại" — đã gặp BA lần, hai phase

Đủ số lần để đặt tên và kiểm chủ động, không chờ gặp lần thứ tư.

| # | Ở đâu | Fixture có gì đúng | Dữ liệu vô hại thế nào | Hậu quả |
|---|---|---|---|---|
| 1 | Phase 2, nguyên nhân B (`5ac43ef`) | Có badge ref, có chữ message | Badge **ngắn** | Badge thật 258px nuốt cột 332px → chữ hiện **0%**. Test xanh suốt |
| 2 | Phase 4 wave 1, M1 (`c0c4264`) | Bản ghi dạng `2` + ≥2 bản ghi sau | Đường dẫn cũ `a_old.txt` **không khớp dạng nào** | Lệch nấc thật nhưng đoạn rò bị `_ => {}` bỏ im lặng → **0 đỏ** |
| 3 | Phase 4 wave 3, M14 (chưa chạy) | Hai repo, đổi repo giữa lúc trì hoãn | Nháp repo B **rỗng** | Đè chuỗi của A lên ô rỗng vẫn trông như "B rỗng" → xanh |

**Lỗi chiều cao hàng (`7596e63`) KHÔNG thuộc lớp này** — đừng gộp vào. `STATE.md` ghi
trung thực nó "không tái hiện được bằng số đo": happy-dom không tính layout nên fixture
nào cũng không đo được. Đó là lớp lỗi ở mục 3.4, khác hẳn.

**Cách kiểm chủ động, trước khi viết fixture:** hỏi *dữ liệu này có thể khiến lỗi để lại
dấu vết quan sát được không?* — chứ không hỏi *fixture này có đúng hình dạng không*.

Instance #1 là ca đắt nhất và đáng nhớ nhất: fixture không chỉ vô hại, nó **không đại
diện đúng ở chính chiều đang quyết định kết quả** (bề rộng chữ với font thật ở bề rộng
khung thật). Khoảng cách chỉ lộ ra khi đo bằng Segoe UI nạp từ `C:/Windows/Fonts`.

Điều này cũng nói về **mã**, không chỉ về test: lệch nấc chỉ im lặng khi đường dẫn cũ
tình cờ vô hại. Một đường dẫn cũ có khoảng trắng, hoặc bắt đầu bằng ký tự dạng bản ghi,
sinh ra bản ghi **sai thật sự** — nên đây là lỗi có thể gặp trên repo thật.

**Quy tắc bắt buộc cho Phase 4:**

- Cổng đọc nguồn **phải bỏ chú thích trước khi tìm**.
- Cổng phải **khẳng định tiền đề**: nếu không tìm thấy thứ cần kiểm thì cổng **đỏ**,
  không phải xanh.
- Fixture cho một đột biến phải **được chứng minh phân biệt được**, không phải được
  chỉ định.
- `waitFor` phải neo vào phần tử **chỉ tồn tại ở trạng thái đang kiểm**.
- Suite có test bất đồng bộ phải chạy **nhiều lần**; một lần xanh không chứng minh ổn
  định. (Lỗi #6 đỏ 1/5 lần và sống qua cả một wave vì suite chỉ chạy một lần.)

### 3.2 Phân tích byte — hai lần hỏng im lặng

- **`%x1f` trong `--format`** (plan 02-04): thoát **0**, stdout trông hợp lý, thanh bên
  **rỗng hoàn toàn** trong im lặng.
- **`git log --follow --name-status -z` phân tách bằng `\0\n`**, không phải `\0` trơn
  (kiểm bằng `od -c`: `\0 \n M \0`). Bộ phân tích tách `\0` đọc status thành `"\nM"`,
  so `status == "M"` trượt **không một tiếng nào**, danh sách về rỗng.
- Wave 5 còn tìm thêm: phải tách theo **cả** `\x1f` **và** `\0`, vì các trường
  `--format=` ngăn nhau bằng `\x1f`.

**Cho Phase 4:** `--porcelain=v2 -z` có **nhiều dạng dòng** (`1`, `2`, `u`, `?`, `!`,
`#`), và dòng dạng `2` (đổi tên/copy) mang **hai** đường dẫn ngăn bằng NUL **bên trong**
một bản ghi. Đọc sai là lệch nấc toàn bộ phần còn lại — đúng lớp lỗi của bản ghi `R`
trong wave 5. **Phải có test đặt ≥ 2 bản ghi SAU một bản ghi dạng `2`** để bắt lệch nấc;
một bản ghi sau nó là không đủ.

### 3.3 `GIT_EXTERNAL_DIFF` — lỗi Phase 1 tốn nhất

`cmd.env("GIT_EXTERNAL_DIFF", "")` **không** tắt external diff; git spawn một chương
trình tên `""` và **mọi** lệnh sinh bản vá thoát 128 với stdout rỗng, **im lặng**. Tệ
hơn: một chương trình **không tồn tại** làm `show`/`diff`/`diff-tree` thoát **0**, 0
hunk, **không stderr**. Đã sửa bằng `env_remove` + `--no-ext-diff` tiêm tập trung
(`f5c4c17`), và cờ phải đặt **sau** subcommand.

**Cho Phase 4:** mọi lệnh git mới đi qua `git/exec.rs`. Không tự spawn git ở chỗ khác.

### 3.4 happy-dom **không** tính CSS layout, **không** có scroll thật

Đây là lý do cả năm lỗi hiển thị của Phase 3 đều do chủ dự án mở ứng dụng mà tìm ra,
không phải do test. `MergeView` **chưa từng** được render trong bất kỳ test nào.

**Cho Phase 4:** mọi khẳng định về **layout** (chiều cao hàng WIP, thẳng hàng cột đồ
thị với cột văn bản, ba nhóm tệp không tràn) là **chưa kiểm chứng** cho tới khi có
người xem trên Chromium thật. Plan **không** được ghi "đạt" cho một tiêu chí layout dựa
trên test happy-dom. Ghi "có mã, chưa kiểm".

### 3.5 Pathspec và thứ tự cờ

- **`git diff --name-status` không được mang pathspec**: nó lọc tên cũ **trước** khi
  phát hiện đổi tên chạy, nên git báo `A` thay vì `R080`.
- **`--` phải nằm trước pathspec** (T-03-32), và có cổng của 03-02 ghim việc này cho
  **mọi** lệnh có pathspec. Lệnh mới của Phase 4 phải cập nhật cổng đó (nó đếm số lệnh
  và sẽ đỏ đúng cách khi thiếu).

### 3.6 Đo đúng cách

- **Profile release là bắt buộc.** Debug cho `parse_log` 455 ms vs release 63 ms — chậm
  gần 7 lần. Đọc số debug rồi kết luận "trượt Core Value" là sai.
- **Baseline criterion bị ghi đè** bởi lần chạy mới nhất; muốn so ngược phải stash về
  commit cũ rồi chạy lại.
- **`git log` bị cắt còn 50 dòng** trong môi trường này — **trần cứng**, không phụ
  thuộc cách gọi. Đo bare, là lệnh duy nhất trong lượt gọi:

  ```text
  git log --all --oneline | wc -l    ->  50
  git log --all --format=%h | wc -l  ->  50
  git rev-list --all --count         -> 151
  git rev-list HEAD --count          -> 151
  ```

  Dùng `git rev-list` hoặc `git merge-base --is-ancestor` cho **mọi** câu hỏi
  reachability hoặc tồn tại. **Thất bại im lặng:** `git log | grep <sha>` trả exit 1 và
  0 khớp — không phân biệt được với một commit thật sự không tồn tại. Một session khác
  đã kết luận sai rằng ba commit của nó bị xoá và lịch sử bị viết lại; cả ba đều
  reachable. Tôi cũng từng ghi sai rằng việc cắt này "phụ thuộc cách gọi" — nó không;
  tôi đã nhận con số 151 từ `rev-list` rồi gán sai nguyên nhân.

  `rev-list --all` và `rev-list HEAD` **bằng nhau** ở repo này, nên không có ref nào
  unreachable từ HEAD — đừng đọc chênh lệch hai số đó thành "có việc bị mất".
- **`.vitest/json/output.json` bị cache**: phải `rm -f` trước mỗi lần chạy, nếu không
  đọc lại kết quả cũ. Tôi đã đo sáu lần "ổn định" trên một tệp stale trước khi phát
  hiện.
- 🔴 **Ứng dụng đang mở khoá `cargo test --lib --tests`.** Nếu
  `src-tauri/target/debug/git-plum.exe` đang chạy (session khác hoặc chủ dự án mở app):

  ```text
  error: failed to remove file `...\target\debug\git-plum.exe`
  Caused by: Access is denied. (os error 5)
  ```

  **Chỉ định target không thoát được**: cargo relink binary chính kể cả khi chỉ yêu cầu
  test target. Tôi đã kiểm cả `--lib --tests --no-run` và `--test <một-tên>` — **cả hai
  exit 101**. `CARGO_TARGET_DIR` riêng cũng thất bại (rebuild 292 crate →
  `paging file is too small`, os error 1455).

  **Chạy được:** `cargo test --lib` (không cần bin). Wave 1 đo được 242 ở đó.

  **Không kill tiến trình đó** nếu nó không phải của mình — có thể là app chủ dự án đang
  dùng để chạy cổng dogfood. Ghi con số `--lib` và ghi `--lib --tests` là **chưa đo**,
  đừng suy ra tổng rồi báo như đã đo.
- **`rtk` lọc đầu ra `cargo test`** nhưng **có** ghi một dòng tổng vào tệp khi
  redirect (`cargo test: 284 passed, 1 ignored`), nên `> tệp` cho tổng số. Muốn **tên
  từng test** thì cần `rtk proxy cargo test`.

### 3.7 Kiểm tra bản dựng người dùng nhận

Đã xảy ra: chủ dự án kiểm một exe cũ hơn bản sửa **32 phút**. Quy tắc từ đó: **mỗi vòng
checkpoint kết thúc bằng một bản dựng release, và lần giao lại phải dán dấu thời gian
tệp exe.** Kiểm bằng `ls -l --time-style=full-iso src-tauri/target/release/git-plum.exe`.

---

## 4. Rủi ro riêng của Phase 4

| # | Rủi ro | Vì sao thật | Giảm nhẹ |
|---|---|---|---|
| R1 | Hàng WIP làm lệch ánh xạ `commits[index]` | Lớp lỗi này đã xảy ra **hai lần** ở Phase 2 | Mọi phép ±1 ở **một** chỗ; test ghim cột đồ thị và cột văn bản cùng hàng ở ≥ 3 vị trí cuộn |
| R2 | Watcher gây vòng lặp: ghi → sự kiện → đọc → ghi | Thao tác ghi cũng chạm `.git` | Thao tác ghi trả trạng thái trực tiếp; sự kiện của chính mình phải nhận ra được |
| R3 | `index.lock` khi người dùng chạy git ở terminal | Chủ dự án **dùng terminal song song** — đó là cả điểm của WORK-10 | Thử lại có giãn cách, **không** xoá; thông báo nói rõ đang chờ |
| R4 | `--porcelain=v2` dòng dạng `2` lệch nấc | Đúng lớp lỗi bản ghi `R` của wave 5 | Test có ≥ 2 bản ghi **sau** một bản ghi dạng `2` |
| R5 | Commit rỗng / chỉ khoảng trắng trong thông điệp | `commit-msg` hook có thể từ chối | Không tự sửa thông điệp; hiện nguyên văn lỗi hook |
| R6 | amend trên commit đã push | Ràng buộc nói **cảnh báo, không chặn** | Lấy ahead/behind từ `--branch` của cùng lệnh status |
| R7 | Bản nháp mất khi chuyển repo | Tiêu chí thành công số 6 | Nháp khoá theo `repo_id`, lưu qua `tauri-plugin-store`; **`flushDraft()` khi đổi repo** — ghi đĩa trì hoãn ~300 ms nên ký tự gõ ngay trước lúc chuyển sẽ mất nếu không xả |
| R8 | Xây trên diff viewer chưa kiểm | Năm lỗi hiển thị đã tìm thấy bằng mắt | Vòng commit **không** phụ thuộc diff render đúng (mục 0) |

---

## 5. Điều kiện đóng phase

**Exit gate (dogfood, bắt buộc):** một commit **thật** vào một repository **thật**, tạo
ra **chỉ bằng git-plum**.

Cổng này **không** tự phê duyệt được, và chủ dự án đã chọn "dừng lại, báo tôi" cho cổng
bắt buộc. Kết quả đúng khi cổng không đóng được là **một plan vá khoảng cách**, không
phải đi tiếp Phase 5.

Bảy tiêu chí thành công của ROADMAP nằm ở mục Phase 4 của `.planning/ROADMAP.md`; tiêu
chí 5 (cập nhật dưới 1 giây khi chạy git từ terminal ngoài) và 7 (hàng WIP xuất hiện và
biến mất đúng lúc) **chỉ kiểm được bằng người chạy ứng dụng thật**.

---

## 6. Ghi chú phối hợp nhiều session

Ba session Claude đang mở trên cùng repo này (2026-09-22). Đã xác nhận qua tin nhắn:

- **`git-plum-a3`** thêm WORK-11 vào ROADMAP/REQUIREMENTS theo yêu cầu tường minh của
  chủ dự án (từ một ảnh GitKraken: hàng `// WIP` với `✏3 +1`). Hai tệp đó **chưa
  commit**. Nó đã xác nhận **không** sở hữu Phase 4.
- **`git-plum-2a`** làm Phase 2 (đồ thị) và dừng ở checkpoint 02-07. Sáu tệp Phase 2/3
  chưa commit trong cây (`App.tsx`, `RefSidebar`, `CommitList`, `app.css`) là của nó —
  hai bản sửa lỗi chủ dự án báo. **Không đụng.**

**Nghĩa là:** phase này chỉ ghi vào `.planning/phases/04-commit-loop/` và mã mới của
Phase 4. Không sửa `ROADMAP.md`, `REQUIREMENTS.md`, hay sáu tệp Phase 2 đó khi chúng
còn chưa commit.

### Cập nhật 2026-09-22: PLAT-11 (nhiều repo mở theo thẻ) kéo lên v1

Chủ dự án kéo V2-05 lên v1 thành **PLAT-11**, giao cho **Phase 2** (không phải Phase 4)
— đó là việc nền tảng/vỏ, và Phase 2 đã sở hữu thanh bên với đồ thị mà thanh thẻ ngồi
lên trên. Tổng requirement **60 → 61**. **Phạm vi Phase 4 không đổi**: vẫn sáu
WORK-01, 02, 08, 09, 10, 11.

**Chạm Phase 4 ở đúng một chỗ, và chỗ đó đã xử lý sẵn.** Nếu thẻ về trước, thì "chuyển
repo" ở tiêu chí thành công số 6 nghĩa là "chuyển thẻ". Nháp đã khoá theo `repoId`
(`commitStore.draftByRepo` + `repoId` **trong khoá lưu trữ** của `commitDraft.ts`), nên
đổi thẻ là đổi `repoId` và hành vi đúng tự nhiên. Ca hẹp `flushDraft()` ở R7 áp y hệt
cho đổi thẻ.

**Ca của Phase 2 thì khác và khó hơn — ghi ở đây để không ai nhầm nó với ca nháp:**
`scrollTop` của virtualizer **không nằm trong store nào**, nó là trạng thái DOM. PLAT-05
khoá mọi thứ khác theo `repo_id` nên đổi thẻ gần như miễn phí, riêng `scrollTop` thì
không. Hai đường: giữ cây component của thẻ không hoạt động **mounted** (tốn RAM, đụng
ràng buộc < 150 MB) hoặc lưu `scrollTop` theo `repoId` rồi khôi phục. Việc của plan
02-08, không phải của phase này.
