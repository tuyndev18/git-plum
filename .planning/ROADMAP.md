# Roadmap: git-plum

**Created:** 2026-09-21
**Granularity:** standard
**Mode:** mvp (Vertical MVP — mỗi phase giao một khả năng dùng được đầu-cuối, từ React UI qua Tauri IPC xuống lớp Rust gọi `git`)
**Core Value:** Đọc và hiểu lịch sử của một repository phải tức thì — đồ thị commit mở ra trong dưới một giây và cuộn mượt kể cả trên repo hàng chục nghìn commit.

**Coverage:** 59/59 v1 requirements đã được ánh xạ (không có requirement mồ côi, không có requirement trùng phase).

---

## Nguyên tắc sắp xếp

Thứ tự phase bám theo chuỗi phụ thuộc thực tế của công việc, không theo tầng kỹ thuật:

- **Phase 1 chứa toàn bộ nhóm "làm ngay hoặc không bao giờ"** (PLAT-02, PLAT-03, PLAT-04, PLAT-05). Sửa sau nghĩa là viết lại mọi thứ đã dựng bên trên.
- **Core Value (HIST-05) được kiểm chứng ngay trong Phase 2**, không dời sang phase đánh bóng. Nếu kiến trúc không đạt mốc dưới một giây, phải lộ ra lúc còn rẻ để đổi.
- **Nhóm WORK bị tách làm hai phase.** Nghiên cứu nâng mức độ khó của staging theo khối từ MEDIUM lên HIGH. Tách ra để vòng lặp commit cơ bản dùng được sớm (Phase 4), rồi mới đầu tư vào phần khó (Phase 5).
- **BRANCH-05 (giải quyết xung đột) nằm cùng phase với merge/rebase** vì dùng chung máy trạng thái đang-dở (MERGE_HEAD, rebase-merge, CHERRY_PICK_HEAD).
- **AI xếp cuối và là nhánh lá** — không có gì phụ thuộc vào nó, nên đây là thứ được phép cắt bỏ nếu tiến độ căng, và là ứng viên chạy song song tốt nhất với Phase 6.
- **REL bị xé đôi.** REL-04 (CI ba nền tảng) vào Phase 1 vì nó bắt lỗi Linux WebKitGTK và macOS lúc còn rẻ. REL-01/02/03 (bộ cài, tự cập nhật, giấy phép) ở phase cuối.
- **Cổng dogfood** đặt ở Phase 3, 4 và 6: tác giả phải hoàn thành một luồng git thật chỉ bằng git-plum trước khi phase được coi là xong. Đây là biện pháp chống lại kiểu hỏng đã được ghi nhận — dự án GUI Git nghiệp dư trông có vẻ tiến xa nhưng chưa bao giờ dùng được cho một luồng công việc thật.

---

## Phases

- [ ] **Phase 1: Nền tảng và lớp bọc git** - Mở được repo thật trong cửa sổ ba vùng, thấy nhật ký lệnh git; toàn bộ ràng buộc kiến trúc không thể sửa sau được chốt tại đây
- [ ] **Phase 2: Lịch sử và đồ thị nhánh** - Đọc được lịch sử commit qua đồ thị nhiều lane, có tìm kiếm, đạt mốc dưới một giây trên repo 100k commit
- [ ] **Phase 3: Xem khác biệt** - Chọn tệp trong commit và đọc được nội dung thay đổi có tô màu cú pháp, hai chế độ hiển thị
- [ ] **Phase 4: Vòng lặp commit theo tệp** - Tạo được commit thật từ thư mục làm việc chỉ bằng git-plum
- [ ] **Phase 5: Staging theo khối và an toàn khi huỷ** - Đưa từng khối thay đổi vào vùng chờ mà không hỏng dữ liệu, huỷ bỏ có đường lùi
- [ ] **Phase 6: Nhánh, remote và xung đột** - Fetch/pull/push, merge/rebase, và giải quyết xung đột ngay trong ứng dụng
- [ ] **Phase 7: Hỗ trợ AI soạn thông điệp** - Sinh và soạn lại thông điệp commit qua nhà cung cấp tuỳ chọn, khoá lưu trong kho khoá hệ điều hành
- [ ] **Phase 8: Phát hành** - Người ngoài tải về, cài đặt và tự nhận bản cập nhật trên cả ba nền tảng

---

## Phase Details

### Phase 1: Nền tảng và lớp bọc git
**Goal**: Người dùng mở được một repository thật và thấy ứng dụng trả lời bằng dữ liệu git thật, đồng thời mọi ràng buộc kiến trúc không thể sửa sau đã nằm đúng chỗ
**Mode:** mvp
**Depends on**: Không (phase đầu)
**Requirements**: PLAT-01, PLAT-02, PLAT-03, PLAT-04, PLAT-05, PLAT-06, PLAT-07, PLAT-08, PLAT-09, PLAT-10, REL-04
**Success Criteria** (what must be TRUE):
  1. Người dùng chọn một thư mục repository qua hộp thoại, ứng dụng nhận đúng repo đó, và lần mở sau repo xuất hiện trong danh sách gần đây
  2. Người dùng kéo ranh giới ba vùng giao diện và kích thước đó còn nguyên sau khi đóng mở lại ứng dụng
  3. Người dùng nhìn thấy bảng nhật ký liệt kê đúng từng lệnh git ứng dụng đã chạy, kèm mã thoát và thời gian chạy
  4. Khi chọn một thư mục không phải repository, người dùng nhận thông báo đọc hiểu được kèm lệnh đã chạy — ứng dụng không treo và không im lặng
  5. Chạy bản dựng release từ Explorer (không phải `tauri dev`) và thực hiện vài thao tác git: không có cửa sổ console đen nào nháy lên
**Ràng buộc bắt buộc (không thể sửa sau)**:
  - Việc đầu tiên của phase là dựng toolchain: VS Build Tools 2022 (workload Desktop development with C++) trước, rồi rustup stable-msvc. Bước xác minh tường minh: `cargo build` chạy thành công. Không có bước này thì không kiểm chứng được gì khác.
  - PLAT-02: một lớp bọc sinh tiến trình **duy nhất**, ghim `LC_ALL=C`, `LANG=C`, `GIT_TERMINAL_PROMPT=0`, `GIT_ASKPASS` rỗng, `GCM_INTERACTIVE=never`, stdin đóng, `CREATE_NO_WINDOW` trên Windows, `log.showSignature=false`, xoá `GIT_AUTHOR_*` / `GIT_COMMITTER_*` thừa hưởng, ghim `--cleanup=whitespace`, `diff.noprefix`, `diff.external`, `format.coverLetter`. Cộng thêm hạn giờ cứng 30–60s cho thao tác mạng.
  - PLAT-03: hàng đợi/mutex ghi theo từng repository, bọc quanh mọi lệnh làm thay đổi repo. Đây cũng là gốc của việc sau này watcher chỉ phải xử lý thay đổi thật sự từ bên ngoài.
  - PLAT-04: sổ lệnh trung tâm — mọi thao tác là một lệnh có tên và tham số, không gắn thẳng vào chỗ xử lý sự kiện bấm nút.
  - PLAT-05: trạng thái hai phía IPC đều khoá theo `repo_id` (map, không phải một handle Option đơn lẻ). Đây là quyết định frontend đắt nhất nếu sửa sau.
  - `error.rs` với `GitError` và phần `Serialize` viết tay; mọi command trả `Result<T, GitError>`.
  - Kỷ luật capability file Tauri v2: phạm vi hẹp ngay từ đầu, quyền cấp nằm cùng commit với chỗ dùng plugin.
  - REL-04: khung CI ba nền tảng (`windows-latest`, `ubuntu-22.04`, `macos-latest`) dựng được ngay từ phase này. Dùng `libwebkit2gtk-4.1-dev`, không phải 4.0. Không dùng `ubuntu-latest`.
  - Không có parser, không có domain type nào ngoài repo handle tối thiểu. Việc của phase này là chứng minh cầu IPC và lớp sinh tiến trình chạy đúng trên cả ba nền tảng trước khi có bất kỳ logic git nào đặt lên trên.
**Validation checkpoint**: Chạy bộ kiểm thử và QA thủ công với locale hệ thống không phải tiếng Anh (checkpoint #7). Chi phí sửa thấp vì nằm ở một hàm trung tâm — nhưng phải kiểm, không được giả định.
**Plans**: TBD
**UI hint**: yes

### Phase 2: Lịch sử và đồ thị nhánh
**Goal**: Người dùng mở một repository và đọc được toàn bộ lịch sử của nó qua đồ thị nhánh — tức thì, kể cả trên repo rất lớn
**Mode:** mvp
**Depends on**: Phase 1
**Requirements**: HIST-01, HIST-02, HIST-03, HIST-04, HIST-05, HIST-06, HIST-07, HIST-08, HIST-09, HIST-10, HIST-11
**Success Criteria** (what must be TRUE):
  1. Người dùng cuộn qua lịch sử commit của một repository 100k commit, đồ thị hiện ra trong dưới một giây và cuộn không giật
  2. Đồ thị nhiều lane có màu luôn thẳng hàng tuyệt đối với từng dòng commit ở mọi vị trí cuộn, kể cả khi repo có merge nhiều hơn hai cha, nhánh mồ côi, lịch sử không liên quan, bản sao nông hoặc HEAD tách rời
  3. Người dùng chọn một commit và thấy ngay tiêu đề, nội dung đầy đủ, tác giả, thời gian, mã commit cha và danh sách tệp thay đổi — chuyển được giữa dạng đường dẫn phẳng và dạng cây
  4. Người dùng gõ một từ khoá và tìm được commit theo thông điệp, tên tác giả, mã commit hoặc đường dẫn tệp
  5. Người dùng thấy nhãn nhánh và tag neo đúng hàng commit, phân biệt local với remote, và thanh bên liệt kê nhánh local, nhánh remote, tag kèm số đếm
  6. Mở một repository có tên tệp và thông điệp commit chứa ký tự không phải UTF-8: trang lịch sử vẫn hiện đầy đủ, không có dòng nào bị mất
**Chuẩn bị bắt buộc trước khi viết thuật toán lane**: dựng sẵn bộ repo mẫu — octopus merge bốn cha, hai gốc không liên quan hợp nhất bằng `--allow-unrelated-histories`, một nhánh mồ côi, một hình dạng rẽ nhánh rộng 20+ lane đồng thời, một repo có tên tệp không phải UTF-8 và thông điệp emoji, một bản sao nông.
**Validation checkpoints** (là tiêu chí thoát phase, không dời về sau):
  - #1 Hiệu năng repo lớn: sao chép Linux kernel hoặc Chromium, đo thời gian vẽ lần đầu và FPS lúc cuộn. Mốc cần đạt: `gitlanes` làm được 32k commit trong ~300ms ở 60fps trên đúng stack này. Gắn benchmark `criterion` cho gán lane và phân tích `git log` ở mức 100k commit vào CI ngay trong phase này. Nếu trượt: nâng bộ nạp commit từ command phân trang lên `tauri::ipc::Channel` để React vẽ 500 dòng đầu trong khi Rust còn đang phân tích phần còn lại.
  - #2 Cách vẽ đồ thị — canvas hay lớp phủ SVG ở mức 100k dòng. Dựng bản canvas trước, nhưng **giữ bộ vẽ sau một interface** để đổi là sửa một tệp. Xử lý device pixel ratio nếu không sẽ mờ trên màn HiDPI. Canvas thuần trình bày — bắt sự kiện bấm dùng div của dòng, không dùng toạ độ canvas.
  - #4 IPC JSON hay nhị phân cho dữ liệu lane. Ship JSON trước rồi đo. Gộp lô lớn — 1000 commit một thông điệp, không phải 1. Nếu JSON chiếm phần lớn profile: dùng `tauri::ipc::Response` trả byte thô, hợp với GraphRow vì đó là dữ liệu số cố định chiều rộng.
**Ràng buộc**: Một scroll container, một virtualizer — cột đồ thị và cột văn bản vẽ từ **cùng một mảng virtualItems**, không phải hai vùng cuộn đồng bộ. Rust phải trả **đầy đủ hình học từng dòng** (lane, màu, cạnh đi xuyên, cạnh đi ra) để frontend không bao giờ tự tính liên thông. Bước ba của thuật toán lane phải là **vòng lặp qua mọi cha sau cha đầu**, không phải trường hợp đặc biệt hai cha; cha thiếu do bản sao nông phải kết thúc lane chứ không làm sập. Đặt và ghi lại một cách vẽ suy giảm có chủ ý cho ca bệnh lý (giới hạn số lane hiện, hiện chỉ báo "+N cha nữa").
**Plans**: TBD
**UI hint**: yes

### Phase 3: Xem khác biệt
**Goal**: Người dùng đọc được chính xác một commit đã thay đổi những gì, ở mức từng dòng
**Mode:** mvp
**Depends on**: Phase 2
**Requirements**: DIFF-01, DIFF-02, DIFF-03, DIFF-04, DIFF-05, DIFF-06
**Success Criteria** (what must be TRUE):
  1. Người dùng chọn một tệp trong một commit và thấy nội dung thay đổi có tô màu cú pháp, chuyển được giữa chế độ hợp nhất và chế độ hai cột
  2. Người dùng nhảy tới khối thay đổi kế tiếp và trước đó bằng một thao tác, và bật tắt được hiển thị ký tự khoảng trắng
  3. Người dùng mở lịch sử thay đổi của riêng một tệp và lần theo được các phiên bản của nó
  4. Người dùng chọn một tệp nhị phân, một tệp 200MB, hoặc một con trỏ Git LFS: ứng dụng hiện thông báo phù hợp trong thời gian bình thường, không đổ nội dung thô ra màn hình và không đứng hình
  5. Chọn đi chọn lại giữa các commit đã xem: nội dung khác biệt hiện ra tức thì, không tính lại
**Ràng buộc**: Nạp lười các chế độ ngôn ngữ CodeMirror theo phần mở rộng tệp — có hơn 30 gói, nhập sẵn tất cả sẽ phình bundle và hại thời gian khởi động, mà đó là Core Value. Không cài gói meta `codemirror`. Bộ nhớ đệm diff theo LRU (~200 mục), **không bao giờ cần vô hiệu hoá** với commit lịch sử vì diff của commit là bất biến. Dùng đầu ra kết thúc bằng NUL cho mọi biến thể diff có mang tên tệp.
**Validation checkpoint**: #3 `@codemirror/merge` **tự tính diff từ hai tài liệu đầy đủ** — trong khi git đã đưa sẵn diff. Đo với một tệp lớn thật. Nếu chậm: tự vẽ hunk bằng decoration của CodeMirror, điều khiển bằng đầu ra `git diff`.
**Exit gate (dogfood, bắt buộc)**: Bản chỉ-đọc lịch sử-cộng-diff phải được tác giả dùng hằng ngày trên repo thật của mình — gồm cả repo của chính dự án này và ít nhất một repo bên thứ ba lớn, lộn xộn — **trước khi bắt đầu bất kỳ việc nào thuộc thư mục làm việc.** Đây là cổng thoát phase, không phải điều nên có.
**Plans**: TBD
**UI hint**: yes

### Phase 4: Vòng lặp commit theo tệp
**Goal**: Người dùng hoàn thành được một vòng làm việc thật — xem thay đổi, chọn tệp, viết thông điệp, tạo commit — mà không rời khỏi git-plum
**Mode:** mvp
**Depends on**: Phase 3
**Requirements**: WORK-01, WORK-02, WORK-08, WORK-09, WORK-10
**Success Criteria** (what must be TRUE):
  1. Người dùng thấy ba nhóm tệp rõ ràng: đã đưa vào vùng chờ, chưa đưa vào, và chưa được theo dõi — chọn một tệp thì thấy ngay nội dung thay đổi của nó trong trình xem diff
  2. Người dùng đưa tệp vào vùng chờ và lấy ra khỏi vùng chờ, danh sách cập nhật đúng sau mỗi lần
  3. Người dùng soạn thông điệp, tạo commit, và commit đó xuất hiện ngay trên đồ thị lịch sử ở Phase 2
  4. Người dùng sửa commit gần nhất (amend) và thấy kết quả phản ánh đúng; nếu commit đó đã được đẩy lên thì có cảnh báo — nhưng không bị chặn
  5. Người dùng chạy `git` từ terminal bên ngoài trong lúc ứng dụng đang mở: giao diện tự cập nhật theo trong khoảng dưới một giây, không cần bấm làm mới
  6. Bản nháp thông điệp commit còn nguyên sau khi đóng mở lại ứng dụng và sau khi chuyển repo
**Ràng buộc**: Lệnh trạng thái dùng `--porcelain=v2` kèm thông tin nhánh, liệt kê toàn bộ tệp chưa theo dõi, kết thúc bằng NUL. Watcher được giới thiệu **ở phase này**, không sớm hơn: đây là điểm đầu tiên mà dữ liệu cũ thật sự nguy hiểm. Watcher chỉ theo dõi hẹp bên trong `.git` (`HEAD`, `index`, `refs/**`, `packed-refs`, các tệp trạng thái merge/rebase) — **không bao giờ** theo dõi đệ quy thư mục làm việc không lọc; gộp và trì hoãn 250–300ms. Không bao giờ tự xoá `.git/index.lock`; coi nó là trạng thái tạm thời có thể phục hồi, thử lại có giãn cách. Chạy hook `pre-commit` và `commit-msg` mặc định, có công tắc `no-verify` tường minh. Thao tác ghi trả về trạng thái mới trực tiếp chứ không chờ watcher, đồng thời vẫn phát sự kiện để phòng thủ.
**Exit gate (dogfood, bắt buộc)**: Một commit thật vào một repository thật, tạo ra **chỉ bằng git-plum**.
**Plans**: TBD
**UI hint**: yes

### Phase 5: Staging theo khối và an toàn khi huỷ
**Goal**: Người dùng tạo được commit sạch sẽ bằng cách chọn từng khối thay đổi, và không bao giờ mất dữ liệu vì một lần huỷ nhầm
**Mode:** mvp
**Depends on**: Phase 4
**Requirements**: WORK-03, WORK-04, WORK-05, WORK-06, WORK-07
**Success Criteria** (what must be TRUE):
  1. Người dùng chọn một khối thay đổi trong một tệp có nhiều thay đổi, đưa riêng khối đó vào vùng chờ, và commit chỉ chứa đúng khối đó
  2. Sau khi đưa một phần vào vùng chờ, tệp dùng CRLF giữ nguyên ký tự xuống dòng, tệp không có dòng trống cuối giữ nguyên trạng thái đó, và tệp chứa byte không phải UTF-8 không bị thay bằng ký tự thay thế
  3. Người dùng sửa tệp ở nơi khác rồi bấm đưa khối vào vùng chờ: ứng dụng báo tệp đã đổi và yêu cầu làm mới, thay vì áp một bản vá không khớp
  4. Người dùng huỷ bỏ thay đổi theo tệp và theo khối, rồi khôi phục lại được nội dung vừa huỷ từ một danh sách
**Quy tắc đúng đắn cứng (không thương lượng)**:
  - Bản vá là **byte thô từ đầu đến cuối**. Không bao giờ giải mã thành chuỗi. Truyền `--recount`. **Tách stderr khỏi stdout** để cảnh báo của clean filter không bao giờ lọt vào nội dung bản vá.
  - Kiểm lại mã băm blob của tệp ngay trước khi áp; chạy thử `git apply --check --cached` trước.
  - Khi thất bại, **không bao giờ** thử lại bằng khớp mờ, unidiff không ngữ cảnh, hay `--whitespace=fix`. Hiện thông báo "tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới".
  - Xử lý `core.autocrlf` một cách tường minh.
  - Với tệp chưa theo dõi, động từ phải là **"Xoá"**, không phải "Huỷ bỏ".
  - An toàn khi huỷ: trước mỗi lần huỷ, `git stash create` các đường dẫn bị ảnh hưởng và giữ object đó có thể tới được dưới một không gian ref riêng dành cho thùng rác, kèm danh sách "Vừa huỷ gần đây" để khôi phục.
  - Staging theo dòng vẫn **nằm ngoài v1** — khó hơn hẳn và không có trong Active.
**Ca kiểm thử ngoài đường hạnh phúc**: sửa tệp giữa lúc mở diff và lúc bấm stage · tệp nhị phân · tệp không có dòng trống cuối · tệp vừa đổi tên vừa sửa · chuẩn hoá CRLF/LF giao với staging một phần · một công cụ build đang ghi vào thư mục làm việc (CPU phải có chặn trên) · một lần tự làm mới chồng lên một lần stage do người dùng khởi tạo (không được để lỗi index-lock lọt tới người dùng).
**Rủi ro tiến độ**: Đây là phase nhiều khả năng vượt kế hoạch nhất. Nghiên cứu xếp staging theo khối ở mức HIGH và khuyến nghị ngân sách gấp 2–3 lần ước lượng ngây thơ. Đánh giá lại lịch ở cuối phase này trước khi cam kết phạm vi Phase 6.
**Plans**: TBD
**UI hint**: yes

### Phase 6: Nhánh, remote và xung đột
**Goal**: Người dùng làm được toàn bộ công việc hằng ngày với nhánh và remote trong git-plum, kể cả lúc căng thẳng nhất là khi có xung đột
**Mode:** mvp
**Depends on**: Phase 5
**Requirements**: BRANCH-01, BRANCH-02, BRANCH-03, BRANCH-04, BRANCH-05, BRANCH-06, BRANCH-07, BRANCH-08, BRANCH-09, BRANCH-10, BRANCH-11, BRANCH-12
**Success Criteria** (what must be TRUE):
  1. Người dùng fetch, pull và push được; khi push bị từ chối, người dùng thấy giải thích rằng remote có N commit mình chưa có và được đưa đúng ba lựa chọn: pull rồi merge, pull rồi rebase, hoặc đẩy đè an toàn — không bao giờ hiện stderr thô rồi dừng
  2. Người dùng tạo, chuyển sang, đổi tên và xoá nhánh; chuyển nhánh khi thư mục làm việc còn thay đổi thì ứng dụng thử trước, chỉ can thiệp khi git thật sự từ chối, rồi đưa lựa chọn cất tạm-và-chuyển hoặc huỷ-và-chuyển — không bao giờ từ chối thẳng
  3. Người dùng hợp nhất hoặc chuyển gốc một nhánh, thấy danh sách tệp xung đột, giải quyết từng khối ngay trong ứng dụng bằng nút nhận bên ta / bên họ / cả hai, đánh dấu đã giải quyết, và hoàn tất thao tác
  4. Người dùng đóng ứng dụng giữa lúc đang dở merge rồi mở lại: ứng dụng nhận ra trạng thái đó, hiện băng thông báo nêu rõ thao tác, các nhánh liên quan, số tệp còn xung đột, và luôn có sẵn nút huỷ bỏ
  5. Người dùng đặt lại nhánh về một commit theo cả ba mức, cất tạm nhiều lần và áp lại đúng cái mình chọn, tạo và xoá tag, bốc và đảo ngược một commit
  6. Người dùng mở repository bằng terminal, trình soạn thảo hoặc trình quản lý tệp bên ngoài do chính mình chọn; và khi một thao tác mạng cần thông tin đăng nhập mà máy chưa lưu, ứng dụng báo lỗi rõ ràng trong thời gian có hạn thay vì đứng im
**Validation checkpoint**: #5 Công sức làm trình giải quyết xung đột **trên stack CodeMirror 6 của chúng ta là chưa được kiểm chứng**. Con số "merge trong bốn giờ" của SourceGit **không chuyển giao được** — họ có sẵn codebase Avalonia trưởng thành kèm component soạn thảo. Chạy một spike có giới hạn thời gian **trước khi chốt phạm vi phase**: dựng marker xung đột của git trong một view CM6, có nút nhận bên ta/bên họ/cả hai theo từng khối, ghi tệp, stage. Nếu spike vượt hạn: đường lùi là ship phần phát hiện xung đột cộng nút mở công cụ ngoài cho v1 và đẩy trình giải quyết thành mục v1.x đầu tiên — nhưng quyết định đó phải dựa trên bằng chứng từ spike, không dựa trên lập luận #892 vốn đã bị chứng minh sai.
**Ràng buộc hành vi**:
  - Đặt `merge.conflictStyle=zdiff3`; cân nhắc `mergetool.hideResolved` (git ≥2.31) để chỉ xung đột thật nổi lên.
  - Mọi thao tác đẩy đè mặc định là force-with-lease, **không bao giờ** force trần, và hộp thoại phải nêu rõ khác biệt.
  - Pull khi có thay đổi cục bộ: phát hiện **trước khi chạy**, đưa lựa chọn cất tạm / pull / lấy ra / huỷ. Nếu lấy ra tự động bị xung đột thì nói rõ và dẫn thẳng vào trình giải quyết.
  - Phát hiện trạng thái đang dở bằng cách đọc `MERGE_HEAD`, thư mục `rebase-merge`, và `CHERRY_PICK_HEAD` — cả lúc mở repo lẫn lúc khởi động lại ứng dụng.
  - Phân biệt xung đột nội dung với add/add, delete/modify và xung đột nhị phân. **Không bao giờ tự chọn một bên cho tệp nhị phân.**
  - Đánh dấu một tệp đã giải quyết phải stage nó và cập nhật số còn lại; lấy một tệp ra khỏi vùng chờ **không được** âm thầm thoát khỏi trạng thái đang merge.
  - Nhiều stash là yêu cầu bắt buộc — mô hình một stash duy nhất của GitHub Desktop là thất bại đã được ghi nhận.
  - Không sửa nội dung tự do trong khung xung đột (đó chính là sản phẩm con trình soạn thảo). Giữ nút "mở bằng công cụ ngoài" làm lối thoát.
  - Phân loại mã thoát git phổ biến thành thông báo cụ thể có hành động; coi stderr chỉ là văn bản chẩn đoán mờ, **không bao giờ** là hợp đồng phân tích được.
  - Kiểm thử chấp nhận tường minh cho BRANCH-12: fetch vào một repo không có thông tin đăng nhập nào được lưu phải thất bại trong thời gian có hạn, không bao giờ treo. Đây là phase đầu tiên thật sự chạy lệnh mạng.
**Exit gate (dogfood, bắt buộc)**: Hoàn thành một luồng nhánh thật đầu-cuối chỉ bằng git-plum — tạo nhánh, commit, push, mở và giải quyết ít nhất một xung đột merge thật, rồi hợp nhất xong.
**Plans**: TBD
**UI hint**: yes

### Phase 7: Hỗ trợ AI soạn thông điệp
**Goal**: Người dùng nhờ được AI viết hoặc cải thiện thông điệp commit, với nhà cung cấp do mình chọn và không có nội dung nào rời máy khi chưa cho phép
**Mode:** mvp
**Depends on**: Phase 5 (nội dung đang chờ trong vùng chờ làm đầu vào). Không có phase nào phụ thuộc vào phase này — đây là nhánh lá, ứng viên chạy song song tốt nhất với Phase 6, và là thứ được phép cắt bỏ nếu tiến độ căng.
**Requirements**: AI-01, AI-02, AI-03, AI-04, AI-05, AI-06
**Success Criteria** (what must be TRUE):
  1. Ở lần cài mới, tính năng AI đang tắt; người dùng phải chủ động bật thì mới có bất kỳ lời gọi ra ngoài nào
  2. Người dùng chọn Claude, OpenAI hoặc mô hình nội bộ qua Ollama, nhập khoá một lần, và khoá đó nằm trong kho khoá của hệ điều hành — không có mặt trong bất kỳ tệp cấu hình dạng văn bản nào
  3. Người dùng sinh được thông điệp commit từ nội dung đang chờ trong vùng chờ, và nhờ soạn lại thông điệp của một commit đã có
  4. Khi nội dung khác biệt chứa dấu vết khoá bí mật, người dùng được cảnh báo **trước khi** bất cứ thứ gì được gửi đi, và có thể dừng lại
  5. Khi nội dung khác biệt quá lớn so với giới hạn, người dùng được cho biết điều đó — thông điệp không bao giờ được trình bày như thể phản ánh toàn bộ thay đổi trong khi thực ra chỉ dựa trên một phần
**Ràng buộc**:
  - Trừu tượng hoá theo nhà cung cấp **ngay từ bản chạy được đầu tiên** — không hardcode Claude rồi tách ra sau.
  - Khoá **không bao giờ** được gửi xuống frontend; lời gọi tới nhà cung cấp AI xảy ra trong Rust. Hai command mỏng: `set_api_key`, `get_api_key`.
  - Quét mẫu khoá bí mật (khoá AWS, đầu tệp khoá riêng PEM, phép gán api-key/secret, chuỗi entropy cao trong tệp dạng env) chạy **phổ quát** — kể cả với Ollama nội bộ, vì phép quét rẻ. Nhiều rò rỉ thật nằm trong tệp **đã được theo dõi**, nên quy tắc ignore không đủ để phòng.
  - Logic đo kích thước diff và quét khoá nằm ở **lớp dùng chung**, không lặp lại theo từng nhà cung cấp.
  - Phân biệt lỗi từ chối kết nối tới localhost (Ollama chưa chạy) với lỗi API đám mây, và phân biệt giới hạn tần suất với lỗi chung.
  - Nếu kho khoá không dùng được (Linux không có Secret Service): tắt tính năng AI kèm thông báo rõ ràng, **không bao giờ** lùi về tệp văn bản.
  - Đặt "cải thiện bản nháp của tôi" ít nhất ngang hàng với "sinh từ đầu" trên giao diện — đó là thứ người dùng thật sự xin.
**Validation checkpoint**: #6 API của `keyring` v4 đã tái cấu trúc mạnh so với v3 (phần lớn tài liệu sẵn có mô tả API `Entry::new` của v3). Đối chiếu đường dẫn import với docs.rs cho keyring 4.2.0 tại thời điểm hiện thực; xác nhận feature `v1` giữ lại ergonomics cũ.
**Plans**: TBD
**UI hint**: yes

### Phase 8: Phát hành
**Goal**: Một người lạ tải git-plum về, cài được trên nền tảng của họ, và từ đó tự nhận bản cập nhật
**Mode:** mvp
**Depends on**: Phase 6 (và Phase 7 nếu AI không bị cắt). Khung CI đã có từ Phase 1.
**Requirements**: REL-01, REL-02, REL-03
**Success Criteria** (what must be TRUE):
  1. Người dùng tải bộ cài từ trang phát hành và cài đặt thành công trên Windows (MSI), macOS (DMG) và Linux (AppImage và deb)
  2. Người dùng đang chạy bản cũ được ứng dụng thông báo có bản mới, cài được nó, và tắt được tính năng tự cập nhật nếu muốn
  3. Một lập trình viên lạ đọc README, làm theo hướng dẫn dựng từ mã nguồn, và dựng được ứng dụng trên máy của họ
  4. Giấy phép mã nguồn mở (MIT) có mặt trong kho mã và nêu rõ trong README
**Ràng buộc**:
  - Bộ cập nhật của Tauri dùng cặp khoá minisign riêng, **độc lập với chữ ký mã của hệ điều hành** — tự cập nhật không cần chứng chỉ trả tiền.
  - Windows: chứng chỉ EV **không bắt buộc** (Microsoft đã bỏ vị thế ưu tiên của EV năm 2024). Ship không ký cho v1 và ghi lại cảnh báo SmartScreen trong tài liệu; thêm Azure Trusted Signing khi có người dùng thật.
  - macOS: Apple Developer Program 99$/năm là bắt buộc cứng (DMG không ký bị Gatekeeper **chặn**, không phải chỉ cảnh báo). Hoãn lại, ghi lại cách xử lý thuộc tính quarantine trong tài liệu.
  - Linux: không cần gì. Dựng trên `ubuntu-22.04`, không phải `ubuntu-latest` — glibc mới hơn khiến AppImage từ chối chạy trên bản phân phối cũ.
  - **Soát toàn bộ tệp capability tìm phạm vi dạng ký tự đại diện ở phase này**, biện minh hoặc thu hẹp từng cái. Đây là công cụ mã nguồn mở có xin thông tin đăng nhập — câu chuyện tin cậy phải sạch.
  - Thêm kiểm thử khói WebDriver ở đây, không sớm hơn. `tauri-driver` chính thức chỉ hỗ trợ Windows và Linux; `@wdio/tauri-service` phủ cả ba. Không dùng tauri-driver của CrabNebula (cần khoá API trả tiền — sai với dự án MIT).
**Plans**: TBD
**UI hint**: no

---

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Nền tảng và lớp bọc git | 0/? | Not started | - |
| 2. Lịch sử và đồ thị nhánh | 0/? | Not started | - |
| 3. Xem khác biệt | 0/? | Not started | - |
| 4. Vòng lặp commit theo tệp | 0/? | Not started | - |
| 5. Staging theo khối và an toàn khi huỷ | 0/? | Not started | - |
| 6. Nhánh, remote và xung đột | 0/? | Not started | - |
| 7. Hỗ trợ AI soạn thông điệp | 0/? | Not started | - |
| 8. Phát hành | 0/? | Not started | - |

---

## Requirement Coverage

| Phase | Requirements | Số lượng |
|---|---|---|
| 1 | PLAT-01…PLAT-10, REL-04 | 11 |
| 2 | HIST-01…HIST-11 | 11 |
| 3 | DIFF-01…DIFF-06 | 6 |
| 4 | WORK-01, WORK-02, WORK-08, WORK-09, WORK-10 | 5 |
| 5 | WORK-03, WORK-04, WORK-05, WORK-06, WORK-07 | 5 |
| 6 | BRANCH-01…BRANCH-12 | 12 |
| 7 | AI-01…AI-06 | 6 |
| 8 | REL-01, REL-02, REL-03 | 3 |
| **Tổng** | | **59 / 59 ✓** |

Không có requirement nào không được phủ. Không có requirement nào xuất hiện ở hai phase.

---

## Không cam kết thời gian

Theo quyết định đã ghi trong PROJECT.md: không cam kết tổng thời gian. Chuỗi phase **là** kế hoạch; thời lượng là đầu ra. Đánh giá lại sau mỗi phase, đặc biệt là sau Phase 5 (phase nhiều khả năng vượt kế hoạch nhất) trước khi chốt phạm vi Phase 6.

---
*Roadmap created: 2026-09-21*
