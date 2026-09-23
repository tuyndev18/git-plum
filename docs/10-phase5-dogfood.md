# Phase 5 — Dogfood: staging theo khối và an toàn khi huỷ

**Ngày lập:** 2026-09-23
**Trạng thái:** 🔴 **CHƯA CHẠY.** Hai cổng dưới đây là **cổng người kiểm bắt buộc**, và
executor **không** được tự phê duyệt chúng.

---

## 🔴 Đọc trước: vì sao tệp này trống ở cột kết quả

`CONTEXT.md` mục 0 ghi lại rằng `VERIFICATION.md` của Phase 4 dùng chữ *"mắt người
(release)"* **ba lần** mà **không cấp nó cho thứ gì**. Tệp này không lặp lại khuôn đó.

Mọi ô "Đạt/Không" bên dưới **để trống** cho tới khi chủ dự án thật sự chạy các bước.
Một ô đánh dấu Đạt bởi executor là một dòng **sai sự thật**, và nó tệ hơn một ô trống:
một ô trống nói đúng rằng chưa ai kiểm.

**Điều máy đã kiểm được** nằm ở `VERIFICATION.md` và ở `scripts/verify-work04-bytes.sh`.
Tiêu chí thành công **2** (WORK-04) đã đóng **bằng máy** và **không** cần kiểm lại bằng
mắt ở đây.

---

## Cổng 1 — Mười bước trên bản dựng release

**Kiểm những gì:** WORK-03 (staging theo khối), WORK-05 (thông báo đòi làm mới),
WORK-06 (huỷ theo tệp và theo khối), WORK-07 (danh sách Vừa huỷ gần đây).

**Vì sao cần mắt người:** happy-dom **không tính CSS layout và không có cuộn thật**
(`CONTEXT.md` 4.5). Cả **năm** lỗi hiển thị của Phase 3 và lỗi `marginTop`→`paddingTop`
của Phase 4 đều đi qua hàng trăm test tự động. 731 test frontend của hôm nay **không**
trả lời được một câu nào dưới đây.

### Chuẩn bị

```bash
npm run build && npm run tauri build
```

Mở một repo thật **bạn sửa được**.
🔴 `D:/MyCompanyProjects/quanly-truong-phong-so` là **chỉ-đọc** — dùng bản sao hoặc repo khác.

### 🔴 Đường vào ĐÃ ĐỔI so với bản mô tả trong plan

Plan 05-05 viết *"mở tệp đó trong git-plum"* và *"nút trong `DiffToolbar`"*. Cả hai
**không đúng với mã đã dựng**, và lý do ở `05-05-SUMMARY.md` mục "điều plan nói mà hoá
ra sai". Đường thật:

> **Bấm nút `Thay đổi` ở thanh công cụ trên cùng bên phải** (cạnh nút đóng repository).
> Vùng chính đổi sang hai cột: danh sách tệp bên trái, **bảng khối** bên phải, và
> **"Vừa huỷ gần đây"** ở cuối cột trái.
>
> Bấm `Đồ thị` để quay lại lịch sử commit.

### Mười bước

| # | Bước | Câu hỏi | Đạt/Không | Hiện tượng thật |
|---|---|---|---|---|
| 1 | Sửa một tệp ở **ba chỗ cách xa nhau**. Bấm `Thay đổi`, rồi bấm tệp đó. | Thấy **đúng ba** khối, mỗi khối một thanh nút riêng? | | |
| 2 | Bấm "Đưa khối vào vùng chờ" ở khối **GIỮA**. 📷 | Đúng khối **bạn nhắm** vào vùng chờ? (tiêu chí 1) | | |
| 3 | Nhìn tệp ở nhóm "Chưa stage". | Còn **đúng hai** khối kia? | | |
| 4 | Soạn thông điệp và commit *(xem ghi chú dưới bảng)*. Rồi `git show HEAD`. | Commit chứa **đúng** khối giữa, không nhiều hơn? (tiêu chí 1) | | |
| 5 | Mở terminal **ngoài**, sửa chính tệp đó. Về git-plum, bấm đưa một khối vào vùng chờ **không** bấm làm mới. 📷 | Có hiện *"tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới"*? Câu đó **đọc là hiểu ngay**, hay phải đoán? (tiêu chí 3) | | |
| 6 | Bấm "Làm mới" trong băng đó. | Diff cập nhật và thao tác lại được? | | |
| 7 | Bấm "Huỷ bỏ khối" ở **một** khối. | Đúng khối đó biến mất, các khối khác **còn nguyên**? | | |
| 8 | Tìm danh sách "Vừa huỷ gần đây". 📷 | Bạn **tìm thấy nó** mà không phải hỏi nó ở đâu? (tiêu chí 4) | | |
| 9 | Bấm "Khôi phục" ở mục vừa huỷ. | Nội dung **trở lại đúng như trước**? | | |
| 10 | Tạo một tệp **mới** (chưa theo dõi), chọn nó. 📷 | Động từ trên nút huỷ là **"Xoá"**, không phải "Huỷ bỏ"? | | |

📷 = chụp ảnh màn hình cho bước này.

> 🔴 **Ghi chú cho bước 4 — đây là một khoảng trống, không phải một bước.**
>
> Vùng soạn commit (`CommitBox`, Phase 4) **không** nằm trong màn hình `Thay đổi`. Nó
> mở bằng cách bấm hàng WIP trên đồ thị commit — và đó chính là **vòng khoá chết bốn
> điều kiện** mà wave 5 Phase 4 tìm ra và sửa, nhưng chưa ai xác nhận bằng mắt.
>
> Nếu bước 4 **không làm được từ trong ứng dụng**, đó là một kết quả **có giá trị** —
> ghi lại đúng như vậy. Nó nói rằng vòng commit và staging theo khối **chưa gặp nhau**,
> và đó là việc của một plan sau. Xem `05-05-SUMMARY.md`, mục "nợ đã biết".

### Ba câu về bố cục — thứ happy-dom không trả lời được

| Câu | Đạt/Không | Hiện tượng thật |
|---|---|---|
| Khối đang chọn **nhìn ra khác** khối không chọn? | | |
| Thanh nút có **đè** lên nội dung khối hoặc tràn ra ngoài không? | | |
| Danh sách "Vừa huỷ" có tràn hoặc cắt chữ khi có nhiều mục không? | | |

### 🔴 Điều bước 8 CHẮC CHẮN sẽ thấy, và nó không phải lỗi mới

Danh sách "Vừa huỷ gần đây" hiện **thời điểm** đúng, nhưng **không hiện tên tệp**. Mỗi
mục ghi *"Bản lưu này chưa ghi lại tên tệp — danh sách dựng từ tên ref"* kèm 8 ký tự
đầu của mã object.

Đây là khoảng trống **đã biết từ 05-03**: phía Rust dựng danh sách bằng
`git for-each-ref`, và `for-each-ref` không giữ được `paths`/`nhan`. Xem
`05-05-SUMMARY.md`. Câu hỏi cho bạn ở bước 8 vẫn nguyên giá trị: **danh sách như thế
này có dùng được không**, hay nó vô dụng tới mức phải lấp khoảng trống trước khi đóng
phase.

---

## Cổng 2 — Exit gate: một commit thật chỉ bằng staging theo khối

**Trạng thái:** 🔴 **CHƯA CHẠY.**

Trên bản release, trong một repo thật bạn sửa được:

1. Làm **nhiều** thay đổi trong **một** tệp — một số thuộc về commit này, một số không.
2. Chỉ bằng git-plum (**không** chạm terminal): chọn **đúng những khối** thuộc về commit
   này, đưa vào vùng chờ, soạn thông điệp, commit.
3. Ở terminal: `git show --stat HEAD` và `git show HEAD`.
   → Commit chứa **đúng** những khối bạn chọn, không nhiều không ít?
4. Thư mục làm việc còn **đúng** những thay đổi bạn **không** chọn?

| | Kết quả |
|---|---|
| Exit gate đạt? | |
| **Điều gì khó chịu trong lúc dùng** *(mục này đáng giá hơn mục trên)* | |

> 🔴 Bước 2 gặp cùng khoảng trống với bước 4 của cổng 1: vùng soạn commit nằm ở màn
> hình **khác** với vùng chọn khối. Nếu điều đó làm exit gate **không chạy được**, đó
> là câu trả lời của cổng — ghi lại, đừng lách.

---

## 🔴 Nợ kiểm chứng của Phase 4 — nêu rõ, không tự quyết định là không quan trọng

`CONTEXT.md` mục 7: Phase 4 còn **hai cổng đỏ** và `docs/09-phase4-dogfood.md` **chưa
tồn tại**. Đóng Phase 5 mà Phase 4 chưa đóng nghĩa là **bốn phase liên tiếp** đóng với
nợ kiểm chứng.

Điều đó **không** được quyết định trong tệp này. Nó là câu hỏi cho chủ dự án.

Và nó có một hệ quả cụ thể cho chính tài liệu này: bước 4 và exit gate **đều** đi qua
`CommitBox` — mã Phase 4 chưa ai bấm thử. Nếu chúng hỏng, hai cổng của Phase 5 sẽ đỏ vì
một khuyết tật **của Phase 4**, và phải ghi đúng như vậy chứ không tính vào Phase 5.
