# git-plum

## What This Is

git-plum là một ứng dụng desktop mã nguồn mở để quản lý repository Git bằng giao diện đồ hoạ,
dành cho lập trình viên đã quen dùng Git nhưng muốn một công cụ nhanh và nhẹ hơn GitKraken,
đồng thời đầy đủ tính năng hơn GitHub Desktop. Ứng dụng tập trung vào ba việc mà lập trình
viên làm nhiều nhất hằng ngày: đọc lịch sử commit qua đồ thị nhánh, xem khác biệt giữa các
phiên bản, và tạo commit sạch sẽ thông qua staging theo từng khối thay đổi.

## Core Value

Đọc và hiểu lịch sử của một repository phải **tức thì** — đồ thị commit mở ra trong dưới một
giây và cuộn mượt kể cả trên repo hàng chục nghìn commit. Nếu mọi tính năng khác thất bại,
riêng điều này vẫn phải đúng.

## Requirements

### Validated

<!-- Shipped and confirmed valuable. -->

(Chưa có — phải phát hành mới xác nhận được)

### Active

<!-- Current scope. Building toward these. -->

**Nền tảng**

- [ ] Ứng dụng Tauri v2 khởi động được trên Windows, macOS và Linux
- [ ] Lớp Rust gọi `git` CLI, phân tích đầu ra, trả JSON qua IPC
- [ ] Mở repository từ hộp thoại chọn thư mục, nhớ danh sách repo gần đây
- [ ] Bố cục ba vùng: sidebar, vùng chính, bảng chi tiết — kích thước kéo được và được ghi nhớ

**Đọc lịch sử**

- [ ] Đọc lịch sử commit có phân trang qua `git log --topo-order`
- [ ] Thuật toán gán lane dựng đồ thị nhánh, tính trong Rust
- [ ] Vẽ đồ thị nhiều lane có màu, căn thẳng hàng tuyệt đối với danh sách commit
- [ ] Danh sách commit ảo hoá, cuộn mượt ở mức 100k commit
- [ ] Nhãn nhánh và tag neo đúng hàng commit, phân biệt local và remote
- [ ] Sidebar liệt kê nhánh local, nhánh remote, tag kèm số đếm
- [ ] Bảng chi tiết commit: tiêu đề, nội dung, tác giả, ngày, mã commit cha, danh sách tệp
- [ ] Danh sách tệp thay đổi có hai chế độ hiển thị: theo đường dẫn phẳng và theo cây thư mục

**Xem khác biệt**

- [ ] Trình xem diff dựa trên CodeMirror 6, có tô màu cú pháp
- [ ] Hai chế độ: hợp nhất (unified) và cạnh nhau (split)
- [ ] Điều hướng giữa các hunk, bật tắt hiển thị ký tự khoảng trắng
- [ ] Xem lịch sử của riêng một tệp

**Thay đổi repository**

- [ ] Bảng thư mục làm việc: tệp đã staged, chưa staged, chưa theo dõi
- [ ] Stage và unstage theo tệp
- [ ] Stage và unstage theo từng hunk
- [ ] Huỷ bỏ thay đổi (discard) theo tệp và theo hunk
- [ ] Soạn commit message và tạo commit
- [ ] Sửa commit gần nhất (amend)

**Nhánh và remote**

- [ ] Fetch, pull, push (kèm tuỳ chọn force-with-lease)
- [ ] Tạo nhánh, chuyển nhánh, xoá nhánh, đổi tên nhánh
- [ ] Merge và rebase, có phát hiện xung đột và liệt kê tệp xung đột
- [ ] Stash: tạo, xem danh sách, áp dụng, pop, xoá
- [ ] Tạo và xoá tag
- [ ] Cherry-pick và revert một commit

**AI**

- [ ] Lớp trừu tượng hoá nhà cung cấp AI: Claude API, OpenAI API, và model local qua Ollama
- [ ] Sinh commit message từ diff đang staged
- [ ] Soạn lại (recompose) commit message của một commit đã có
- [ ] Lưu khoá API vào keychain của hệ điều hành, không lưu vào tệp cấu hình dạng văn bản

**Phát hành**

- [ ] Bộ cài đặt cho Windows (MSI), macOS (DMG) và Linux (AppImage, deb)
- [ ] README, hướng dẫn build, giấy phép mã nguồn mở
- [ ] Quy trình CI dựng gói cho cả ba nền tảng

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- **Undo / Redo cho thao tác git** — Rất khó làm đúng (phải theo dõi reflog, xử lý thao tác
  không thể đảo ngược). GitKraken mất nhiều năm mới ổn định. Để sau v1.
- **Trình giải quyết xung đột merge dạng đồ hoạ** — Là một sản phẩm con riêng biệt. v1 chỉ
  phát hiện xung đột, liệt kê tệp, và mở trình soạn thảo bên ngoài để người dùng tự sửa.
- **Terminal tích hợp** — Người dùng đã có terminal riêng. Giá trị thêm vào thấp so với công sức.
- **Tích hợp Issues, Teams, Pull Request (GitHub/GitLab/Jira)** — Cần OAuth, nhiều API khác nhau,
  mỗi nhà cung cấp một kiểu. Là cả một milestone riêng, không thuộc v1.
- **Blame theo tệp** — Hữu ích nhưng không nằm trong luồng công việc hằng ngày. Đẩy sang v2.
- **Quản lý worktree** — Ít người dùng. v2.
- **Rebase tương tác dạng kéo thả** — Phức tạp cao, rủi ro mất dữ liệu. v2.
- **Git LFS có giao diện riêng** — Gọi `git` CLI thì LFS tự hoạt động ở lớp dưới; không cần
  giao diện riêng trong v1.
- **Đồng bộ đám mây, tài khoản người dùng, telemetry** — Đây là công cụ mã nguồn mở chạy cục bộ.
- **Hỗ trợ repo trên 100k commit một cách tối ưu** — Thiết kế để *chịu được* mức này, nhưng không
  làm tầng cache trên đĩa hay index tăng dần trong v1. Nếu người dùng thật báo chậm thì làm sau.

## Context

**Nguồn gốc dự án**

Xuất phát từ việc nghiên cứu GitKraken Desktop (xem `docs/screenshots/`) và các công cụ cùng
loại. Tài liệu phân tích đầy đủ nằm ở `docs/01-research-competitors.md`, gồm: so sánh bảy sản
phẩm đang có trên thị trường, phân tích chi tiết ba ảnh chụp giao diện GitKraken, thuật toán
gán lane cho đồ thị commit, các lệnh git có định dạng đầu ra ổn định để phân tích, và bảng
đánh giá rủi ro.

**Khoảng trống thị trường**

Fork thì nhanh nhưng đóng nguồn và phải trả tiền. SourceGit mã nguồn mở và nhẹ nhưng UI chưa
được đánh bóng. GitKraken đẹp nhất nhưng nặng và tính phí cho repo private. Sourcetree miễn
phí nhưng hai codebase tách rời và bản Windows hay lỗi. git-plum nhắm vào: nhanh như Fork,
mã nguồn mở như SourceGit, chất lượng đồ thị ở mức GitKraken.

**Quyết định kỹ thuật nền tảng**

Backend gọi thẳng `git` CLI thay vì dùng libgit2. Đây là lựa chọn mà cả Fork lẫn SourceGit
đều theo, vì khi gọi CLI thì credential helper của hệ điều hành, Git LFS, hook phía client,
submodule, và thuật toán merge/rebase đều hoạt động đúng mà không phải tự cài đặt lại.
Đổi lại phải sinh tiến trình và phân tích chuỗi, nhưng git có sẵn các định dạng đầu ra dành
cho máy đọc (`--porcelain=v2`, `--format` với ký tự phân tách tuỳ chọn) nên việc phân tích
an toàn và ổn định.

**Môi trường phát triển**

- Máy phát triển chính: Windows 11 Pro
- Node.js 22.16, npm 10.9
- Git 2.54.0 cho Windows
- Rust: **chưa cài** tại thời điểm khởi tạo dự án. Phase 1 bao gồm bước dựng toolchain,
  cần cả rustup lẫn Visual Studio Build Tools (thành phần MSVC) — trên Windows, thiếu Build
  Tools thì `cargo build` sẽ lỗi ở bước liên kết.

**Điểm tham chiếu về giao diện**

Học các khái niệm từ GitKraken (bố cục ba vùng, đồ thị nhiều lane, bảng chi tiết bên phải)
nhưng tự thiết kế lại phần nhìn. Không sao chép bảng màu, icon hay kiểu chữ của GitKraken.

## Constraints

- **Tech stack**: Tauri v2 + React + TypeScript ở lớp trên, Rust ở lớp dưới — Cần vừa xử lý
  dữ liệu nhanh (phân tích hàng chục nghìn commit, tính lane, phân tích diff) vừa dựng giao
  diện phức tạp (đồ thị, trình xem diff, cây thư mục). Tauri cho cả hai với dung lượng cài
  đặt dưới 20MB và RAM lúc rảnh dưới 150MB.
- **Backend Git**: Chỉ gọi `git` CLI, không dùng libgit2 hay gitoxide — Bảo đảm tính đúng đắn
  và tận dụng miễn phí credential helper, LFS, hook, submodule.
- **Compatibility**: Windows, macOS, Linux — Là sản phẩm mã nguồn mở công khai, người dùng
  ở cả ba nền tảng. Windows được ưu tiên phát triển và kiểm thử trước vì đó là máy của tác giả.
- **Performance**: Đồ thị mở dưới 1 giây và cuộn mượt ở mức 100k commit — Đây là Core Value,
  không phải mục tiêu phụ.
- **Security**: Khoá API của nhà cung cấp AI lưu trong keychain hệ điều hành, không bao giờ
  ghi vào tệp cấu hình dạng văn bản thuần — Đây là công cụ mã nguồn mở, người dùng phải tin
  tưởng được.
- **Privacy**: Tính năng AI phải chọn tham gia (opt-in) và cho phép dùng model local — Nội dung
  diff có thể chứa mã nguồn nhạy cảm. Không gửi gì ra ngoài khi chưa được cho phép rõ ràng.
- **Licensing**: Giấy phép mã nguồn mở (dự kiến MIT) — Phù hợp định vị sản phẩm công khai.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Tauri v2 thay vì Electron | Cài đặt 8–20MB so với 120–180MB, RAM 80–150MB so với 300–600MB, khởi động 0.3–0.8s so với 1.5–4s. Đây là ứng dụng nặng về xử lý dữ liệu, Rust phù hợp hơn Node. | — Pending |
| Gọi `git` CLI thay vì libgit2 | Credential helper, LFS, hook, submodule, merge/rebase đều đúng mà không phải tự làm. Fork và SourceGit đều chọn cách này. | — Pending |
| Tính toán lane trong Rust, không trong JavaScript | Đây là điểm nghẽn hiệu năng chính. Giữ luồng giao diện luôn rảnh. | — Pending |
| Dùng CodeMirror 6 cho trình xem diff | Tự viết trình xem diff dễ sai ở phần tô màu cú pháp và ảo hoá. CodeMirror đã giải xong. | — Pending |
| Ký tự phân tách `%x1f` / `%x1e` khi phân tích `git log` | Ký tự Unit Separator và Record Separator gần như không bao giờ có trong nội dung commit, an toàn hơn nhiều so với tách theo dòng. | — Pending |
| v1 chỉ phát hiện xung đột merge, không giải quyết | Trình giải quyết xung đột đồ hoạ là một sản phẩm con. Phát hiện và liệt kê đã đủ dùng cho v1. | — Pending |
| AI trừu tượng hoá theo nhà cung cấp ngay từ đầu | Thêm khoảng ba ngày công nhưng tránh khoá cứng vào một nhà cung cấp, và cho phép người dùng chạy model local khi codebase nhạy cảm. | — Pending |
| Học khái niệm từ GitKraken nhưng tự thiết kế phần nhìn | Tránh vấn đề bản quyền thiết kế, đồng thời tạo bản sắc riêng. | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-09-21 after initialization*
