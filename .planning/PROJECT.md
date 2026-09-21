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
- [ ] Lớp bọc tiến trình git có ghim sẵn biến môi trường (`LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`,
      đóng stdin, `CREATE_NO_WINDOW` trên Windows) — bắt buộc từ đầu, sửa sau rất tốn
- [ ] Mọi thao tác ghi vào repo được xếp hàng tuần tự theo từng repo, tránh đụng `index.lock`
- [ ] Sổ đăng ký lệnh (command registry): mọi thao tác đăng ký vào một nơi thay vì gắn thẳng
      vào nút bấm — điều kiện để sau này có command palette và phím tắt tuỳ biến
- [ ] Nhật ký lệnh git: hiển thị đúng những lệnh mà ứng dụng đã chạy, kèm kết quả
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
- [ ] Tìm kiếm trong lịch sử theo thông điệp commit, tác giả, mã commit và tên tệp

**Xem khác biệt**

- [ ] Trình xem diff dựa trên CodeMirror 6, có tô màu cú pháp
- [ ] Hai chế độ: hợp nhất (unified) và cạnh nhau (split)
- [ ] Điều hướng giữa các hunk, bật tắt hiển thị ký tự khoảng trắng
- [ ] Xem lịch sử của riêng một tệp
- [ ] Nhận biết và hiển thị đúng tệp nhị phân, tệp quá lớn và con trỏ Git LFS — không đổ
      nội dung thô ra màn hình

**Thay đổi repository**

- [ ] Bảng thư mục làm việc: tệp đã staged, chưa staged, chưa theo dõi
- [ ] Stage và unstage theo tệp
- [ ] Stage và unstage theo từng hunk
- [ ] Huỷ bỏ thay đổi (discard) theo tệp và theo hunk
- [ ] An toàn khi huỷ bỏ: tự động stash trước mỗi lần discard, có danh sách khôi phục
- [ ] Soạn commit message và tạo commit
- [ ] Sửa commit gần nhất (amend)

**Nhánh và remote**

- [ ] Fetch, pull, push (kèm tuỳ chọn force-with-lease)
- [ ] Tạo nhánh, chuyển nhánh, xoá nhánh, đổi tên nhánh
- [ ] Merge và rebase, có phát hiện xung đột và liệt kê tệp xung đột
- [ ] Tiếp tục, huỷ bỏ và bỏ qua khi đang dở merge, rebase hoặc cherry-pick
- [ ] Giải quyết xung đột dạng hai khung: hiển thị khối xung đột kèm nút nhận bên ta,
      bên họ, hoặc cả hai. Không phải trình soạn thảo ba khung.
- [ ] Đặt lại nhánh về một commit (reset soft, mixed, hard)
- [ ] Stash: tạo, xem danh sách, áp dụng, pop, xoá
- [ ] Tạo và xoá tag
- [ ] Cherry-pick và revert một commit
- [ ] Mở repository bằng terminal, trình soạn thảo hoặc trình quản lý tệp bên ngoài (cấu hình được)

**AI**

- [ ] Lớp trừu tượng hoá nhà cung cấp AI: Claude API, OpenAI API, và model local qua Ollama
- [ ] Sinh commit message từ diff đang staged
- [ ] Soạn lại (recompose) commit message của một commit đã có
- [ ] Lưu khoá API vào keychain của hệ điều hành qua crate `keyring` (Windows Credential
      Manager, macOS Keychain, Linux Secret Service), không lưu vào tệp cấu hình dạng văn bản
- [ ] Quét nội dung diff tìm dấu vết khoá bí mật trước khi gửi ra nhà cung cấp AI

**Phát hành**

- [ ] Bộ cài đặt cho Windows (MSI), macOS (DMG) và Linux (AppImage, deb)
- [ ] Tự động cập nhật
- [ ] README, hướng dẫn build, giấy phép mã nguồn mở
- [ ] Quy trình CI dựng gói cho cả ba nền tảng

### Out of Scope

<!-- Explicit boundaries. Includes reasoning to prevent re-adding. -->

- **Undo / Redo tổng quát cho thao tác git** — Rất khó làm đúng (phải theo dõi reflog, xử lý
  thao tác không thể đảo ngược). GitKraken mất nhiều năm mới ổn định. Để sau v1. *Lưu ý: phần
  an toàn khi huỷ bỏ thay đổi đã được tách ra và đưa vào v1, vì discard là thao tác duy nhất
  trong MVP phá dữ liệu mà không để lại dấu vết trong reflog.*
- **Trình soạn thảo hợp nhất ba khung (merge tool đầy đủ)** — Là một sản phẩm con riêng biệt.
  *Lưu ý: cần phân biệt với trình giải quyết xung đột hai khung, thứ đã được đưa vào v1 — xem
  mục "Nhánh và remote" ở phần Active.*
- **Terminal tích hợp** — Người dùng đã có terminal riêng. Khảo sát khoảng hai mươi yêu cầu
  liên quan tới terminal trong dự án SourceGit cho thấy toàn bộ đều xin nút mở terminal *bên
  ngoài*, không có yêu cầu nào xin terminal nhúng. v1 sẽ có nút mở terminal, trình soạn thảo
  và trình quản lý tệp bên ngoài — rẻ hơn nhiều và đúng thứ người dùng cần.
- **Tích hợp Issues, Teams, Pull Request (GitHub/GitLab/Jira)** — Cần OAuth, nhiều API khác nhau,
  mỗi nhà cung cấp một kiểu. Là cả một milestone riêng, không thuộc v1.
- **Blame theo tệp** — Hữu ích nhưng không nằm trong luồng công việc hằng ngày. Đẩy sang v2.
- **Quản lý worktree** — Ít người dùng. v2.
- **Rebase tương tác dạng kéo thả** — Phức tạp cao, rủi ro mất dữ liệu. v2. *Lưu ý: hai thao
  tác rẻ nhất trong nhóm này — đặt lại nhánh về một commit, và sửa/gộp một commit không phải
  HEAD — đã được tách riêng đưa vào v1.*
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

**Linux là lợi thế cấu trúc, không phải việc phụ**

Yêu cầu nhiều lượt ủng hộ nhất trong toàn bộ lịch sử dự án GitHub Desktop là "GitHub Desktop
cho Linux?" — gấp khoảng ba lần rưỡi yêu cầu đứng thứ hai. Fork không có bản Linux. Đây là
nhu cầu lớn nhất chưa được đáp ứng trong cả phân khúc, nên Linux phải được kiểm thử ngang
hàng với Windows chứ không phải làm cho có.

Thêm nữa, SourceGit — đối thủ gần nhất về định vị — hiện đang có một thông báo mở tìm người
tiếp quản dự án. Thời điểm gia nhập thuận lợi.

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
| ~~v1 chỉ phát hiện xung đột merge, không giải quyết~~ **ĐẢO NGƯỢC:** v1 có trình giải quyết xung đột hai khung | Quyết định ban đầu dựa trên lập luận "editor ngoài đã lo được". Dự án SourceGit đã đóng yêu cầu #892 với đúng lập luận này, bị người dùng phản đối suốt mười ba tháng, rồi hợp nhất PR #2070 trong chưa đầy bốn giờ. Sai lầm nằm ở chỗ lẫn lộn *trình soạn thảo hợp nhất ba khung* (đúng là sản phẩm con) với *trình giải quyết xung đột hai khung* (chỉ là diff view trên marker sẵn có của git kèm ba nút chọn). | ✓ Good — sửa trước khi viết dòng code nào |
| Dùng crate `keyring` 4.2 thay vì plugin Tauri | Tauri v2 không có plugin keychain chính thức. `tauri-plugin-stronghold` là tệp CSDL mã hoá cần mật khẩu chủ do người dùng nhập — tức là vi phạm chính ràng buộc bảo mật đã đặt ra. Plugin keyring của cộng đồng đã bỏ hoang từ 2024. | — Pending |
| Không dùng `tauri-plugin-shell` | Plugin đó tồn tại để *JavaScript* sinh tiến trình. git-plum sinh git từ *Rust*, dùng thẳng `tokio::process::Command`. Thêm plugin là đưa quyền dựng tham số dòng lệnh vào webview, vứt bỏ đúng cái lợi ích bảo mật của việc giữ git ở lớp Rust. | — Pending |
| Ghim `typescript@6.0.3`, không dùng TypeScript 7 | TS 7.0 đã phát hành nhưng `typescript-eslint` khai báo phạm vi phụ thuộc `<6.1.0`, vì TS 7.0 ra mắt mà chưa có API biên dịch lập trình ổn định. Xem lại khi 7.1 ra. | — Pending |
| Sổ đăng ký lệnh làm ngay từ phase nền tảng | Gắn thẳng thao tác vào nút bấm thì rẻ lúc đầu, nhưng muốn có command palette hay phím tắt tuỳ biến sau này phải sửa lại mọi chỗ xử lý sự kiện. Chưa có đối thủ cùng phân khúc nào làm được điều này. | — Pending |
| Nhật ký lệnh git hiển thị cho người dùng | Gần như miễn phí vì lớp bọc đã dựng sẵn tham số dòng lệnh. Là câu trả lời trực tiếp cho chỉ trích phổ biến nhất nhắm vào các công cụ đồ hoạ: che giấu Git khiến người dùng không học được và không tự cứu được khi hỏng. | — Pending |
| Bản vá khi stage theo hunk phải xử lý ở dạng byte thô | Nếu giải mã thành chuỗi, tệp dùng CRLF mất ký tự xuống dòng và nội dung không phải UTF-8 biến thành ký tự thay thế — hỏng dữ liệu âm thầm ngay lúc stage. | — Pending |
| Không cam kết tổng thời gian, đánh giá lại sau mỗi phase | Ước lượng ban đầu mười một đến mười bốn tuần đã thiếu ngân sách cho staging theo hunk, chưa tính các mục bổ sung. Roadmap theo phase có điểm kiểm soát thay cho một mốc thời gian dễ sai. | — Pending |
| AI trừu tượng hoá theo nhà cung cấp ngay từ đầu | Thêm khoảng ba ngày công nhưng tránh khoá cứng vào một nhà cung cấp, và cho phép người dùng chạy model local khi codebase nhạy cảm. | — Pending |
| Học khái niệm từ GitKraken nhưng tự thiết kế phần nhìn | Tránh vấn đề bản quyền thiết kế, đồng thời tạo bản sắc riêng. | — Pending |
| Windows trước, macOS và Linux sau | Máy phát triển là Windows, và chưa tạo remote nên không chạy được CI để kiểm hai nền tảng kia. Mã vẫn viết theo lối đa nền tảng (dùng `PathBuf`, phần riêng của Windows nằm trong `#[cfg(windows)]`), chỉ là chưa kiểm chứng. Đổi lại, lỗi riêng của Linux và macOS sẽ lộ ra muộn — chấp nhận rủi ro này. | — Pending |

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
*Last updated: 2026-09-21 after research — scope revised per FEATURES/STACK findings*
