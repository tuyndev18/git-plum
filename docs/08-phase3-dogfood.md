# Phase 3 — Exit gate: dùng thật trên repo thật

**Trạng thái: ⏸️ CHƯA CHẠY.** Bản release đã dựng; đang chờ chủ dự án dùng.

Đây là **cổng thoát phase**, không phải điều nên có. ROADMAP ghi:

> Bản chỉ-đọc lịch sử-cộng-diff phải được tác giả dùng hằng ngày trên repo thật của
> mình — gồm cả repo của chính dự án này và ít nhất một repo bên thứ ba lớn, lộn xộn
> — **trước khi** bắt đầu bất kỳ việc nào thuộc thư mục làm việc.

Nó khác checkpoint 11 bước của 03-04 (đã chạy, kiểm bằng mắt theo danh sách). Cổng
này không có danh sách bước: nó đo xem công cụ **có dùng được** cho một luồng công
việc thật không, và điều đó chỉ lộ ra sau 10 phút dùng liên tục hoặc sau lần thứ năm
làm cùng một việc.

Nghiên cứu cạnh tranh nêu kiểu hỏng này tường minh — dự án GUI Git nghiệp dư trông có
vẻ tiến xa nhưng chưa bao giờ dùng được cho một luồng công việc thật.

---

## Bản để dùng

```
src-tauri/target/release/git-plum.exe
```

Dựng lúc: **xem mục "Bản release" cuối tài liệu này** (dấu thời gian thật, đối chiếu
được). Ghim vào taskbar.

Vài ngày tới, khi cần đọc lịch sử hay xem một diff, **mở nó trước** thay vì
`git log` / VS Code / GitKraken.

## Repo cần dùng

| Repo | Vai trò | Có trên máy này? |
|---|---|---|
| **git-plum** | repo của chính dự án — ROADMAP đòi tường minh | ✅ (0 merge) |
| **`quanly-truong-phong-so`** | 398 commit nhiều merge — đây là "lớn, lộn xộn" | ❓ không thấy khi dựng bản này |
| **`dau-tri-toan-hoc`** | 1140 commit, **39 merge** — lớn và lộn xộn hơn cả repo trên | ✅ |
| `thithu-web`, `client_cocos_test` | nếu có dịp | ❓ |

`dau-tri-toan-hoc` được thêm vào danh sách vì nó có thật trên máy và **lộn xộn hơn**
repo mà plan dự kiến: 1140 commit và 39 merge, so với 398 commit. Nó cũng là repo đã
dùng để đo giới hạn merge ở mục dưới.

## Không có danh sách bước — nhưng những việc này bạn làm hằng ngày

Nếu vài ngày trôi qua mà chưa việc nào xảy ra thì có thể chưa dùng đủ:

- [ ] Tìm xem một hàm bị đổi lúc nào và bởi ai
- [ ] Đọc một merge commit lớn để hiểu nó mang gì vào
- [ ] So một tệp hôm nay với bản của tuần trước
- [ ] Lần theo lịch sử một tệp đã từng đổi tên
- [ ] Xem một commit đụng nhiều tệp, bấm qua từng tệp

---

## Ghi chép — ghi khi xảy ra, đừng chờ nhớ lại

### 1. Chỗ nào làm bạn quay về terminal hoặc công cụ khác?

> Đây là dữ liệu giá trị nhất của cả cổng. Nếu có việc bạn **không** làm được trong
> git-plum, nó là một khoảng cách thật, dù mọi requirement đánh dấu đạt.

*(chưa có — cổng chưa chạy)*

### 2. Chỗ nào chậm đủ để bạn nhận ra?

> Nêu thao tác và repo. "Mở diff của tệp lớn trong `quanly-truong-phong-so` có một
> nhịp chờ" đáng giá hơn mọi con số benchmark.

*(chưa có — cổng chưa chạy)*

**Câu hỏi còn mở mà cổng này là chỗ trả lời:** thời gian mở diff của `MergeView` trên
một tệp lớn thật (`yarn.lock` ~630 KB) vẫn **chưa có số**. Checkpoint #3 bị bỏ qua
(`docs/09-phase3-diff-decision.md` mục 7-8), và mục 8 nói rõ: *"nếu mở diff của một
tệp lớn cảm thấy chậm, đó **là** phép đo"*. Ngưỡng đã chốt trước khi có số nào: lần
tốt nhất trong 3 ≤ **250 ms**, tệ nhất ≤ **400 ms**; vượt → chuyển đường B.

Một phần nợ đó đã được trả bằng đo Chromium ở wave 4 (200 nghìn dòng = 229ms, chế độ
hai cột), nên khả năng cao là đủ nhanh — nhưng đó là dòng sinh tổng hợp, không phải
`yarn.lock` thật.

### 3. Chỗ nào bạn bấm sai vì giao diện làm bạn tưởng khác?

> Lỗi thiết kế, không phải lỗi mã.

*(chưa có — cổng chưa chạy)*

### 4. Chỗ nào bạn thấy sai, nhưng test đang xanh?

> Ba lỗi hiển thị của Phase 2 thuộc loại này. Nếu tìm được cái thứ tư, nó cũng cần
> một test mới cùng lúc với bản sửa.

*(chưa có — cổng chưa chạy)*

**Vùng nghi ngờ cao nhất, ghi trước để đối chiếu:** chế độ **hai cột** vẫn **không có
test tự động nào chạy qua nó** (happy-dom không có `ResizeObserver` → mọi test đi
nhánh hợp nhất). Wave 4 đã tìm ra **năm** lỗi chỉ tồn tại trong chế độ đó, và cả năm
đều do người dùng mở app thật mới thấy. Danh sách phiên bản của DIFF-05 cũng chưa ai
thấy trên màn hình: bốn cột có thẳng hàng không, tiêu đề dài có bị cắt kèm ellipsis
hay biến mất, panel có đẩy diff ra khỏi khung không.

### 5. Chỗ nào bạn bất ngờ vì nó tốt?

> Cần biết để không phá đi trong Phase 4.

*(chưa có — cổng chưa chạy)*

---

## Khi nào cổng được coi là đóng

**Bạn nói nó đóng.** Không phải khi hết một số ngày, không phải khi danh sách trống.

Cụ thể: khi bạn **tin** rằng mình sẽ mở git-plum để đọc git kể cả khi không đang phát
triển nó.

**Nếu cổng KHÔNG đóng được** — bạn thấy nó chưa dùng nổi cho công việc thật — nói ra
kèm điều còn thiếu. Kết quả đúng khi đó là một **plan 03-06 đóng khoảng cách**, không
phải đi tiếp Phase 4. Phase 4 dựng ngay trên trình xem diff này (WORK-01: *"chọn một
tệp thì thấy ngay nội dung thay đổi của nó **trong trình xem diff**"*), nên xây tiếp
trên một trình xem chưa dùng được là nhân lỗi lên, không phải tiến độ.

**Nếu bạn muốn hoãn cổng này để đi tiếp Phase 4:** đó là quyết định của bạn, và nó sẽ
được ghi vào đây cùng `VERIFICATION.md` rằng cổng thoát **chưa chạy**. Hệ quả ghi đúng:
Phase 3 **chưa đóng**, và đó là **phase thứ ba liên tiếp** đóng với nợ kiểm chứng —
Phase 1 (ba tiêu chí chưa kiểm chứng), Phase 2 (hai checkpoint còn nợ), Phase 3 (cổng
thoát). Ba phase mở liên tiếp là một khuôn hình cần thấy rõ, nên tài liệu nói ra thay
vì để nó tích lại im lặng.

---

## Giới hạn đã biết của DIFF-05 — đọc trước khi coi là lỗi

Hai điều dưới đây là **đo được** và **có chủ ý**, không phải lỗi. Ghi ra để nếu bạn
gặp chúng trong lúc dùng thật thì không mất thời gian báo lại một thứ đã biết.

### Merge commit không xuất hiện trong lịch sử tệp

`git log --follow` bỏ merge commit khỏi **cả danh sách commit**, kể cả một merge đã
**thật sự giải quyết xung đột** trong chính tệp đang xem. Có test ghim hành vi này
(`merge_commit_khong_vao_danh_sach_phien_ban_gioi_han_da_biet`).

Đã thử `--diff-merges=first-parent` và nó **tệ hơn**, đo trên `dau-tri-toan-hoc`:

| tệp | không cờ | `--diff-merges=first-parent` |
|---|---|---|
| `.planning/STATE.md` | 104 bản ghi | **5715** |
| `.planning/ROADMAP.md` | 118 bản ghi | **4655** |

Cờ đó làm git in nội dung bản vá ra cùng luồng, nên `--max-count=200` mất tác dụng và
bộ phân tích nhận hàng nghìn dòng không phải bản ghi tệp. Nó phá cả chặn trên DoS
(T-03-33) lẫn chặn 200 hàng DOM (T-03-34) — tức đổi một khoảng thiếu **nhìn thấy
được** thành hai lỗi **im lặng**.

**Việc cho Phase 4 nếu cổng thoát cho thấy nó đáng:** một lệnh **riêng** cho merge
(`git show -m` trên đúng một commit), không phải một cờ thêm vào lệnh lịch sử tệp.

### Chặn 200 phiên bản

Lịch sử tệp trả tối đa 200 phiên bản gần nhất, và giao diện **nói ra** khi chạm chặn.
Đó là thứ biến chi phí `--follow` (git tính điểm tương đồng ở **mỗi** commit đụng
path) thành hằng số. Nếu bạn gặp một tệp mà 200 không đủ, nói ra — con số này là một
hằng số, không phải một giới hạn kiến trúc.

---

## Việc cho Phase 4 rút ra từ tài liệu này

*(sẽ điền sau khi cổng chạy — mỗi mục phải truy được về một ghi chép ở trên, không
được thêm từ suy luận)*

Hai mục đã có, từ phép đo chứ không từ việc dùng:

1. **Lệnh riêng cho merge commit** — xem "Giới hạn đã biết" ở trên.
2. **Vô hiệu hoá cache khi HEAD đổi.** Lịch sử tệp cố ý **không** cache vì nó phụ
   thuộc HEAD và chưa có gì để dọn cache. Watcher của Phase 4 là thứ làm cache đó khả
   thi — và khi thêm, `lay_lich_su_tep` là chỗ đầu tiên hưởng lợi.

---

## Bản release

| Dựng lúc | Commit | Ghi chú |
|---|---|---|
| **2026-09-22 11:27:48 +0700** | `d1b59cc` | `npx tauri build --no-bundle`, 4 620 800 byte. Gồm **toàn bộ** DIFF-05 (backend + giao diện). |

Đối chiếu trước khi báo lỗi:

```bash
ls -l --time-style=full-iso src-tauri/target/release/git-plum.exe
```

Nếu dấu thời gian **cũ hơn** bảng trên thì bạn đang chạy một exe cũ — dựng lại bằng
`npx tauri build --no-bundle` (mất ~2 phút).

**Bài học quy trình từ wave 4, áp cho cổng này:** mỗi vòng phải kết thúc bằng một lần
dựng release, và dấu thời gian exe phải được ghi ra để đối chiếu. Vòng trước mất một
lượt qua lại vì người dùng thử một exe **cũ hơn bản sửa 32 phút**. Người dùng kiểm
bằng **exe**, không bằng `npm run dev`.
