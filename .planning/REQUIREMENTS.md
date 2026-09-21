# Requirements: git-plum

**Defined:** 2026-09-21
**Core Value:** Đọc và hiểu lịch sử của một repository phải tức thì — đồ thị commit mở ra
trong dưới một giây và cuộn mượt kể cả trên repo hàng chục nghìn commit.

> Phạm vi đã được chốt sau vòng nghiên cứu bốn chiều (STACK, FEATURES, ARCHITECTURE,
> PITFALLS) và một vòng duyệt của chủ dự án. Các quyết định thay đổi so với lần khởi tạo
> được ghi trong `.planning/PROJECT.md` mục Key Decisions và trong
> `.planning/research/SUMMARY.md` mục 2.

---

## v1 Requirements

### PLAT — Nền tảng

Lớp nền mà mọi thứ khác dựng lên. Bốn mục **PLAT-02, PLAT-03, PLAT-04 và PLAT-05** thuộc loại
"làm ngay hoặc không bao giờ" — bỏ qua bây giờ thì sau này phải sửa lại toàn bộ mã đã viết.

- [ ] **PLAT-01**: Ứng dụng khởi động và hiển thị cửa sổ chính trên Windows, macOS và Linux
- [ ] **PLAT-02**: Mọi tiến trình git do ứng dụng sinh ra đều đi qua một lớp bọc duy nhất có
      ghim sẵn `LC_ALL=C`, `LANG=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_ASKPASS` rỗng, stdin đóng,
      và cờ `CREATE_NO_WINDOW` trên Windows
- [ ] **PLAT-03**: Hai thao tác ghi vào cùng một repository không bao giờ chạy chồng nhau —
      mọi lệnh làm thay đổi repo được xếp hàng tuần tự theo từng repository
- [ ] **PLAT-04**: Mọi thao tác người dùng gọi được đều đăng ký trong một sổ lệnh trung tâm,
      không gắn trực tiếp vào chỗ xử lý sự kiện bấm nút
- [ ] **PLAT-05**: Trạng thái trong ứng dụng được lưu theo khoá định danh repository, cho phép
      thêm nhiều repository mở cùng lúc về sau mà không phải viết lại
- [ ] **PLAT-06**: Người dùng mở được một repository bằng hộp thoại chọn thư mục
- [ ] **PLAT-07**: Người dùng mở lại được repository đã dùng gần đây từ một danh sách
- [ ] **PLAT-08**: Người dùng nhìn thấy đúng những lệnh git mà ứng dụng đã chạy, kèm mã thoát
      và thời gian chạy
- [ ] **PLAT-09**: Người dùng kéo được ranh giới giữa ba vùng giao diện, và kích thước đó
      được giữ nguyên ở lần mở ứng dụng sau
- [ ] **PLAT-10**: Khi git trả về lỗi, người dùng nhận được thông báo đọc hiểu được kèm lệnh
      đã chạy, thay vì ứng dụng treo hoặc im lặng

### HIST — Đọc lịch sử

Nhóm chứa Core Value. Mốc kiểm chứng hiệu năng nằm trong chính nhóm này, không để cuối dự án.

- [ ] **HIST-01**: Người dùng thấy danh sách commit của repository đang mở, nạp theo trang
- [ ] **HIST-02**: Người dùng thấy đồ thị nhánh nhiều làn có màu, vẽ bên trái danh sách commit
- [ ] **HIST-03**: Đồ thị vẽ đúng với merge nhiều hơn hai cha, nhánh mồ côi, lịch sử không liên
      quan, bản sao nông và HEAD tách rời
- [ ] **HIST-04**: Đồ thị luôn thẳng hàng với danh sách commit ở mọi vị trí cuộn, không lệch
- [ ] **HIST-05**: Trên repository 100k commit, đồ thị hiện ra trong dưới một giây và cuộn
      không giật
- [ ] **HIST-06**: Người dùng thấy nhãn nhánh và nhãn tag neo đúng hàng commit, phân biệt được
      nhánh local với nhánh remote
- [ ] **HIST-07**: Người dùng thấy danh sách nhánh local, nhánh remote và tag ở thanh bên, kèm
      số lượng từng loại
- [ ] **HIST-08**: Người dùng chọn một commit và thấy tiêu đề, nội dung đầy đủ, tác giả, thời
      gian, mã commit cha và danh sách tệp thay đổi
- [ ] **HIST-09**: Người dùng chuyển được danh sách tệp giữa dạng đường dẫn phẳng và dạng cây
      thư mục
- [ ] **HIST-10**: Người dùng tìm được commit theo thông điệp, tên tác giả, mã commit hoặc
      đường dẫn tệp
- [ ] **HIST-11**: Tên tệp và thông điệp commit chứa ký tự không phải UTF-8 vẫn hiển thị được,
      không làm hỏng dữ liệu trả về

### DIFF — Xem khác biệt

- [ ] **DIFF-01**: Người dùng chọn một tệp trong commit và thấy nội dung thay đổi, có tô màu
      cú pháp
- [ ] **DIFF-02**: Người dùng chuyển được giữa chế độ hợp nhất và chế độ hai cột
- [ ] **DIFF-03**: Người dùng nhảy được tới khối thay đổi kế tiếp và trước đó
- [ ] **DIFF-04**: Người dùng bật tắt được hiển thị ký tự khoảng trắng
- [ ] **DIFF-05**: Người dùng xem được lịch sử thay đổi của riêng một tệp
- [ ] **DIFF-06**: Tệp nhị phân, tệp vượt ngưỡng kích thước, và con trỏ Git LFS được nhận biết
      và hiển thị bằng thông báo phù hợp, không đổ nội dung thô ra màn hình

### WORK — Thay đổi thư mục làm việc

Nhóm có rủi ro tiến độ cao nhất. Staging theo khối là việc khó, không phải việc vừa.

- [ ] **WORK-01**: Người dùng thấy ba nhóm tệp: đã đưa vào vùng chờ, chưa đưa vào, và chưa
      được theo dõi
- [ ] **WORK-02**: Người dùng đưa một tệp vào vùng chờ và lấy ra khỏi vùng chờ
- [ ] **WORK-03**: Người dùng đưa một khối thay đổi vào vùng chờ và lấy ra khỏi vùng chờ
- [ ] **WORK-04**: Nội dung tệp dùng CRLF, tệp không có dòng trống cuối, và tệp chứa byte
      không phải UTF-8 vẫn nguyên vẹn sau khi đưa một phần vào vùng chờ
- [ ] **WORK-05**: Khi tệp đã bị sửa từ lúc giao diện vẽ khác biệt, ứng dụng báo cho người
      dùng làm mới thay vì áp bản vá không khớp
- [ ] **WORK-06**: Người dùng huỷ bỏ thay đổi theo tệp và theo khối
- [ ] **WORK-07**: Trước mỗi lần huỷ bỏ, ứng dụng tự lưu lại nội dung bị huỷ, và người dùng
      khôi phục được từ một danh sách
- [ ] **WORK-08**: Người dùng soạn thông điệp và tạo commit
- [ ] **WORK-09**: Người dùng sửa được commit gần nhất
- [ ] **WORK-10**: Khi có thay đổi đến từ bên ngoài ứng dụng, giao diện tự cập nhật theo

### BRANCH — Nhánh, remote và thao tác lịch sử

- [ ] **BRANCH-01**: Người dùng lấy về (fetch), kéo về (pull) và đẩy lên (push), có tuỳ chọn
      đẩy đè an toàn
- [ ] **BRANCH-02**: Người dùng tạo, chuyển sang, đổi tên và xoá nhánh
- [ ] **BRANCH-03**: Người dùng hợp nhất một nhánh và chuyển gốc một nhánh
- [ ] **BRANCH-04**: Khi có xung đột, người dùng thấy danh sách tệp xung đột
- [ ] **BRANCH-05**: Người dùng giải quyết xung đột ngay trong ứng dụng: mỗi khối xung đột
      hiện kèm nút nhận bên ta, nhận bên họ, hoặc nhận cả hai
- [ ] **BRANCH-06**: Người dùng tiếp tục, huỷ bỏ hoặc bỏ qua khi đang dở hợp nhất, chuyển gốc
      hoặc bốc commit
- [ ] **BRANCH-07**: Người dùng đặt lại nhánh về một commit theo ba mức: giữ vùng chờ, bỏ vùng
      chờ, và xoá cả thay đổi
- [ ] **BRANCH-08**: Người dùng cất tạm thay đổi, xem danh sách đã cất, áp dụng lại, lấy ra
      và xoá
- [ ] **BRANCH-09**: Người dùng tạo và xoá tag
- [ ] **BRANCH-10**: Người dùng bốc một commit sang nhánh hiện tại, và đảo ngược một commit
- [ ] **BRANCH-11**: Người dùng mở repository bằng terminal, trình soạn thảo hoặc trình quản
      lý tệp bên ngoài, chọn được dùng chương trình nào
- [ ] **BRANCH-12**: Khi thao tác cần thông tin đăng nhập mà máy chưa lưu, ứng dụng báo lỗi
      rõ ràng thay vì đứng im chờ

### AI — Hỗ trợ soạn thông điệp commit

Nhóm xếp cuối và được phép cắt bỏ nếu tiến độ căng. Nghiên cứu cho thấy đây là thứ có giá trị
vừa phải, không phải lý do khiến người dùng đổi công cụ.

- [ ] **AI-01**: Người dùng chọn được nhà cung cấp: Claude, OpenAI, hoặc mô hình chạy nội bộ
      qua Ollama
- [ ] **AI-02**: Khoá truy cập được lưu trong kho khoá của hệ điều hành, không nằm trong tệp
      cấu hình dạng văn bản
- [ ] **AI-03**: Người dùng sinh thông điệp commit từ nội dung đang chờ trong vùng chờ
- [ ] **AI-04**: Người dùng nhờ soạn lại thông điệp của một commit đã có
- [ ] **AI-05**: Trước khi gửi bất kỳ nội dung nào ra ngoài, ứng dụng quét tìm dấu vết khoá bí
      mật và cảnh báo người dùng
- [ ] **AI-06**: Tính năng gọi ra bên ngoài phải do người dùng chủ động bật, mặc định tắt

### REL — Phát hành

- [ ] **REL-01**: Có bộ cài đặt cho Windows, macOS và Linux
- [ ] **REL-02**: Ứng dụng tự kiểm tra và cài bản cập nhật mới
- [ ] **REL-03**: Kho mã có README, hướng dẫn dựng từ mã nguồn, và giấy phép mã nguồn mở
- [ ] **REL-04**: Quy trình tích hợp liên tục dựng được gói cài cho cả ba nền tảng

---

## v2 Requirements

Hoãn lại, có theo dõi nhưng không nằm trong lộ trình hiện tại.

### Lịch sử nâng cao

- **V2-01**: Xem ai sửa dòng nào trong một tệp (blame)
- **V2-02**: Chuyển gốc tương tác dạng kéo thả
- **V2-03**: Gộp, sửa và đổi thông điệp của commit không phải commit gần nhất
- **V2-04**: Hoàn tác và làm lại thao tác git nói chung, dựa trên reflog

### Môi trường làm việc

- **V2-05**: Nhiều repository mở cùng lúc theo thẻ
- **V2-06**: Bảng lệnh gõ nhanh và phím tắt tuỳ biến, dựng trên sổ lệnh đã có từ v1
- **V2-07**: Quản lý worktree
- **V2-08**: Giao diện riêng cho Git LFS

### Quy mô lớn

- **V2-09**: Bộ nhớ đệm trên đĩa và chỉ mục tăng dần cho repository trên 100k commit

---

## Out of Scope

Loại trừ rõ ràng, ghi lại để tránh đưa vào sau này.

| Tính năng | Lý do |
|---|---|
| Trình soạn thảo hợp nhất ba khung | Là một sản phẩm con riêng. Khác với trình giải quyết xung đột hai khung đã nằm trong BRANCH-05. |
| Terminal nhúng trong ứng dụng | Khảo sát khoảng hai mươi yêu cầu liên quan trong dự án SourceGit: toàn bộ xin nút mở terminal bên ngoài, không có yêu cầu nào xin terminal nhúng. Đã thay bằng BRANCH-11. |
| Tích hợp Issues, Teams, Pull Request | Cần OAuth và mỗi nhà cung cấp một kiểu API. Là một milestone riêng. |
| Đồng bộ đám mây, tài khoản người dùng | Công cụ chạy cục bộ, mã nguồn mở. |
| Thu thập dữ liệu sử dụng | Không phù hợp định vị sản phẩm. |
| Giao diện quản lý submodule | Gọi git CLI thì submodule hoạt động đúng ở lớp dưới. Giao diện riêng là việc lớn (riêng GitHub Desktop có tới 82 yêu cầu liên quan). |

---

## Traceability

Phase nào phủ requirement nào. Bảng này do bước lập lộ trình điền.

| Requirement | Phase | Status |
|---|---|---|
| (chưa lập lộ trình) | — | Pending |

**Coverage:**
- v1 requirements: 59 tổng cộng
- Mapped to phases: 0
- Unmapped: 59 ⚠️

---
*Requirements defined: 2026-09-21*
*Last updated: 2026-09-21 after research-informed scope revision*
