# Nghiên cứu Git GUI client hiện có

Tài liệu này phân tích các Git GUI desktop đang phổ biến, để rút ra tập tính năng và
quyết định kiến trúc cho dự án (tên tạm: **git-plum**).

---

## 1. Bối cảnh thị trường

| Sản phẩm | Nền tảng kỹ thuật | Giấy phép | Điểm mạnh | Điểm yếu |
|---|---|---|---|---|
| **GitKraken Desktop** | Electron + React | Thương mại (có bản free hạn chế) | Graph đẹp nhất thị trường, UX chải chuốt, tích hợp AI, tích hợp Jira/GitHub/GitLab | Nặng (RAM 500MB–1.5GB), khởi động chậm, trả phí cho repo private |
| **Sourcetree** | WPF (Windows) / Cocoa (macOS) | Miễn phí (Atlassian) | Miễn phí, đầy đủ tính năng, gitflow tích hợp | Hai codebase tách rời, bản Windows hay lỗi, UI cũ, phát triển chậm |
| **Fork** | Native (WPF / Cocoa) | Trả phí một lần (~50 USD) | Rất nhanh, nhẹ, merge conflict resolver tốt, UI gọn | Không mã nguồn mở, ít tích hợp bên thứ ba |
| **SourceGit** | Avalonia + .NET | MIT, mã nguồn mở | Nhẹ, đa nền tảng, miễn phí, tính năng gần ngang Fork | Cộng đồng nhỏ, đánh bóng UI chưa bằng Fork |
| **GitHub Desktop** | Electron + React | MIT, mã nguồn mở | Đơn giản, hợp người mới | Thiếu tính năng nâng cao (rebase tương tác, cherry-pick hạn chế), gắn chặt GitHub |
| **Lazygit** | Go, giao diện terminal | MIT | Cực nhanh, chạy trong terminal | Không phải GUI đồ hoạ, đường học dốc |
| **Tower** | Native | Thuê bao | Đánh bóng cao, undo mạnh | Đắt |

**Kết luận rút ra:** khoảng trống thị trường nằm ở chỗ *nhanh và nhẹ như Fork, mã nguồn mở
như SourceGit, nhưng graph và UX ở mức GitKraken*. Đây là định vị của git-plum.

---

## 2. Phân tích ảnh chụp màn hình GitKraken đã cung cấp

Ba ảnh trong `docs/screenshots/` cho thấy ba trạng thái giao diện chính.

### 2.1 `main-1.png` — Chế độ xem lịch sử (History view)

Bố cục chia ba cột:

```
┌──────────┬────────────────────────────────────┬──────────────────┐
│ Sidebar  │ Commit graph + danh sách commit    │ Chi tiết commit  │
│          │                                    │                  │
│ LOCAL    │ ● │ refactor(description): ...     │ Tiêu đề commit   │
│ REMOTE   │ ●─┤ fix(text-fitting): ...         │ Nội dung mô tả   │
│ WORKTREES│ ● │ feat(debug-extension): ...     │ Tác giả + ngày   │
│ ISSUES   │ ● │ ...                            │ ───────────────  │
│ TEAMS    │                                    │ 3 tệp thay đổi   │
│ TAGS     │                                    │ [Path] [Tree]    │
│          │                                    │ • file1.tsx      │
└──────────┴────────────────────────────────────┴──────────────────┘
│ Terminal tích hợp (bật/tắt được)                                  │
└───────────────────────────────────────────────────────────────────┘
```

Chi tiết cần lưu ý:

- **Thanh công cụ trên cùng**: Undo, Redo, Pull (có nút xổ chọn chế độ), Push, Branch,
  Stash, Pop, Terminal. Bên phải: Actions, Search.
- **Bộ chọn repo và nhánh**: `repository: thithu-web` và `branch: master`, mỗi cái một
  dropdown riêng.
- **Cột BRANCH/TAG**: nhãn nhánh neo vào đúng hàng commit tương ứng
  (`master`, `dev/dongnq`, `v2.01.004`). Nhãn có icon phân biệt local / remote / cả hai.
- **Cột GRAPH**: các nút tròn có avatar tác giả nằm trên đường nối. Nhiều lane song song.
- **Cột COMMIT MESSAGE**: dòng tiêu đề in đậm, phần thân commit nối tiếp cùng hàng bằng
  chữ xám mờ, cắt cụt khi tràn.
- **Bảng chi tiết bên phải**: tiêu đề, phần thân, khối tác giả (avatar, tên, ngày giờ),
  mã commit cha, số tệp thay đổi, chuyển đổi Path/Tree, tuỳ chọn "View all files".
- **Nút "Recompose commit with AI"** — tính năng AI đặt ngay trong bảng chi tiết.
- **Thanh trạng thái dưới cùng**: trạng thái Pull Request, mức thu phóng, phiên bản.

### 2.2 `main-2.png` — Chế độ xem khác biệt (Diff view)

Cột giữa chuyển từ danh sách commit sang trình xem diff:

- Đường dẫn tệp trên đầu, mã hoá ký tự (UTF-8), nút đóng.
- Nút **Edit This File**.
- Chuyển đổi **File View / Diff View**.
- **Blame** và **History** cho riêng tệp đó.
- Điều hướng lên/xuống giữa các hunk.
- Ba chế độ hiển thị: hợp nhất (unified), danh sách, cạnh nhau (split).
- Bật/tắt hiện ký tự khoảng trắng, bật/tắt xuống dòng mềm.
- Mỗi hunk có tiêu đề `@@ -10,6 +10,8 @@` và nút **Revert Hunk**.
- Hai cột số dòng (bên cũ / bên mới), tô xanh cho dòng thêm, đỏ cho dòng xoá.
- Cú pháp được tô màu.
- Sidebar thu hẹp thành dải icon dọc, vẫn giữ số đếm (2 local, 3 remote, 1 worktree, 12 tag).

### 2.3 `main-3.png` — Diff view kèm cây tệp đầy đủ

Giống 2.2, khác ở bảng bên phải: bật "View all files" thì danh sách tệp đổi thành **cây
thư mục đầy đủ của repo** tại commit đó, có ô lọc tệp, nút "Expand All", và huy hiệu đếm
số tệp thay đổi trên từng thư mục (`src` hiện `3` và `1 file selected`).

### 2.4 Bảng tính năng suy ra từ ảnh

| # | Tính năng | Thuộc MVP? |
|---|---|---|
| 1 | Nhiều repo mở theo tab | Có |
| 2 | Sidebar: nhánh local / remote / worktree / tag, có số đếm | Có (bỏ worktree) |
| 3 | Đồ thị commit nhiều lane có màu | Có |
| 4 | Nhãn nhánh và tag neo vào hàng commit | Có |
| 5 | Danh sách commit ảo hoá (virtualized) | Có |
| 6 | Bảng chi tiết commit | Có |
| 7 | Danh sách tệp thay đổi, chế độ Path và Tree | Có |
| 8 | Diff view: unified và split | Có |
| 9 | Tô màu cú pháp trong diff | Có |
| 10 | Revert Hunk | Có |
| 11 | Blame cho từng tệp | Không — để v2 |
| 12 | History cho từng tệp | Có |
| 13 | Terminal tích hợp | Không — để v2 |
| 14 | Undo / Redo thao tác git | Không — để v2 (rất khó làm đúng) |
| 15 | Pull / Push / Fetch | Có |
| 16 | Stash / Pop | Có |
| 17 | Thao tác nhánh: tạo, checkout, xoá, merge, rebase | Có |
| 18 | AI soạn lại commit message | Có |
| 19 | Avatar tác giả | Có (Gravatar) |
| 20 | Tích hợp Issues / Teams / Pull Request | Không — ngoài phạm vi |

---

## 3. Bài học kỹ thuật từ từng sản phẩm

### 3.1 Bài học về hiệu năng

**Vấn đề:** repo Linux kernel có hơn 1 triệu commit. Sourcetree treo, GitKraken ăn vài GB RAM.

**Cách các client nhanh xử lý:**

1. **Không nạp toàn bộ lịch sử.** Chỉ nạp 1000–5000 commit đầu, nạp thêm khi cuộn.
2. **Ảo hoá danh sách.** Chỉ dựng DOM cho các hàng đang nhìn thấy.
3. **Tính toán lane ở lớp dưới (Rust), không làm trong JavaScript.**
4. **Bộ nhớ đệm phân tích diff.** Diff của cùng một commit không đổi, cache theo mã commit.
5. **Tiến trình git chạy nền, không chặn giao diện.**

### 3.2 Bài học về tính đúng đắn

**Fork và SourceGit đều gọi thẳng `git` CLI** thay vì dùng libgit2, vì:

- Credential helper (Windows Credential Manager, SSH agent, GCM) hoạt động sẵn.
- Git LFS hoạt động sẵn.
- Hook phía client (`pre-commit`, `commit-msg`) chạy đúng.
- Submodule, sparse-checkout, worktree đều đúng hành vi.
- Rebase và merge dùng đúng thuật toán của git, không lệch.

libgit2 nhanh hơn khi *đọc*, nhưng khi *ghi* thì thiếu quá nhiều thứ, phải tự cài đặt lại
và rất dễ sai.

### 3.3 Bài học về giao diện

- **Graph phải căn thẳng hàng tuyệt đối với danh sách commit.** Nếu dùng hai vùng cuộn
  riêng, chúng sẽ lệch. Phải chung một vùng cuộn, graph vẽ theo chỉ số hàng.
- **Chiều cao hàng cố định.** Ảo hoá chỉ đơn giản khi mọi hàng cùng chiều cao.
- **Màu lane lặp theo chu kỳ.** Dùng bảng màu 8–12 màu, lane thứ n lấy `màu[n % 12]`.
- **Không dựng lại toàn bộ graph khi chọn commit khác.** Tách trạng thái chọn ra khỏi
  trạng thái dữ liệu graph.

---

## 4. Thuật toán dựng đồ thị commit

Đây là phần được cho là khó nhất, nhưng thực chất đã có lời giải chuẩn.

### 4.1 Bài toán

Cho danh sách commit theo thứ tự tô-pô (cha luôn đứng sau con), mỗi commit có danh sách
mã cha. Cần gán cho mỗi commit một **lane** (cột dọc) và sinh ra các **đoạn nối** giữa
các hàng.

### 4.2 Thuật toán gán lane

```
Trạng thái: lanes = mảng, lanes[i] = mã commit mà lane i đang "chờ"

Với mỗi commit C tại hàng r:
  1. Tìm lane đầu tiên đang chờ C  -> đó là lane của C.
     Nếu không có lane nào chờ C   -> cấp lane trống đầu tiên cho C (đây là đầu nhánh).

  2. Mọi lane khác cũng đang chờ C (nhiều nhánh cùng trỏ về C)
     -> đó là các điểm hợp nhánh. Vẽ đường chéo từ lane đó về lane của C, rồi giải phóng lane.

  3. Gán cha — LẶP qua toàn bộ danh sách cha, không giả định chỉ có tối đa hai:
     - parents[0]      -> tiếp tục chiếm lane của C.
     - parents[1..n]   -> với MỖI cha còn lại, cấp một lane trống mới và vẽ một đường
                          chéo rẽ nhánh riêng. Merge octopus có 3, 4 hay nhiều cha đều
                          phải chạy đúng.
     - Nếu C không có cha -> giải phóng lane của C (commit gốc hoặc nhánh mồ côi).

  4. Mọi lane khác đang hoạt động -> vẽ một đoạn thẳng dọc đi xuyên qua hàng r.
```

Độ phức tạp `O(n × số_lane_đang_hoạt_động)`, thực tế gần như tuyến tính vì số lane hoạt
động hiếm khi vượt 20.

**Các trường hợp biên bắt buộc phải có kiểm thử riêng.** Cách cài đặt ngây thơ chỉ xử lý
merge hai cha sẽ vỡ ở những trường hợp sau, và đây đều là dữ liệu có thật trong repo thật:

| Trường hợp | Vì sao vỡ | Cách xử lý |
|---|---|---|
| **Merge octopus** (3 cha trở lên) | Cài đặt chỉ đọc `parents[1]` sẽ bỏ sót các cha còn lại, đường nối biến mất và lane bị rò rỉ | Lặp qua `parents[1..]`, mỗi cha một lane mới |
| **Nhánh mồ côi** (`git checkout --orphan`) | Không có cha chung với phần còn lại, thuật toán tưởng là commit gốc giữa chừng lịch sử | Xử lý như commit không cha: giải phóng lane, không nối ngược |
| **Lịch sử không liên quan** (`merge --allow-unrelated-histories`) | Hai cây tách rời gặp nhau, số lane tăng đột biến | Không cần xử lý riêng nếu bước 3 lặp đúng |
| **Shallow clone** (`--depth`) | Commit ở biên có mã cha trỏ tới thứ không tồn tại trong tập dữ liệu | Bỏ qua cha không có trong tập đã nạp, kết thúc lane bằng dấu hiệu "còn tiếp" |
| **HEAD tách rời** | Không có nhãn nhánh nào neo vào, dễ bị coi là rác | Neo nhãn `HEAD` riêng, không lẫn với nhãn nhánh |
| **Commit submodule** | Không ảnh hưởng lane, nhưng cần kiểm tra để chắc chắn | Thêm một repo mẫu có submodule vào bộ kiểm thử |

Lỗi sai ở merge octopus không phải giả thuyết — chính công cụ của Microsoft từng có lỗi
tính lệch vị trí khi dựng commit-graph cho merge nhiều cha.

### 4.3 Cấu trúc dữ liệu đầu ra

Mỗi hàng sinh ra:

```rust
struct GraphRow {
    commit_id: String,
    lane: u16,              // lane chứa nút tròn
    color: u8,              // chỉ số màu, = lane % SO_MAU
    passthrough: Vec<Edge>, // các đường đi thẳng xuyên qua hàng
    out_edges: Vec<Edge>,   // đường rẽ xuống hàng dưới
}

struct Edge {
    from_lane: u16,
    to_lane: u16,
    color: u8,
}
```

Giao diện chỉ việc vẽ SVG từ cấu trúc này. Không cần tính toán gì thêm ở phía React.

---

## 5. Phân tích đầu ra của git

### 5.0 Bắt buộc: cấu hình môi trường cho mọi tiến trình git

**Mọi** lệnh git sinh ra từ ứng dụng phải đi qua một lớp bọc chung, thiết lập sẵn các biến
môi trường sau. Thiếu chúng thì các định dạng dành cho máy đọc ở những mục bên dưới vẫn
không đủ an toàn.

```rust
cmd.env("LC_ALL", "C");             // Ngày tháng, stderr, chữ gợi ý — đều phụ thuộc locale
cmd.env("LANG", "C");
cmd.env("GIT_TERMINAL_PROMPT", "0"); // Chặn git hỏi thông tin đăng nhập
cmd.env("GIT_ASKPASS", "");          // Chặn cả hộp thoại hỏi mật khẩu dạng đồ hoạ
cmd.env("GCM_INTERACTIVE", "never");
cmd.stdin(Stdio::null());            // Đóng stdin — nếu không, git chờ nhập liệu vô hạn
```

Thêm nữa, trên Windows phải đặt cờ `CREATE_NO_WINDOW` khi sinh tiến trình:

```rust
#[cfg(windows)]
{
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}
```

**Vì sao quan trọng:**

| Thiếu thứ gì | Hậu quả |
|---|---|
| `GIT_TERMINAL_PROMPT=0` và đóng stdin | Khi repo cần đăng nhập mà chưa có thông tin lưu sẵn, git chờ nhập mật khẩu trên stdin. Giao diện **treo vĩnh viễn**, không có thông báo lỗi. Đây là cái bẫy kinh điển nhất khi bọc git bằng GUI. |
| `LC_ALL=C` | Trên máy đặt ngôn ngữ khác tiếng Anh, chuỗi ngày tháng và thông báo lỗi đổi định dạng, bộ phân tích vỡ trên máy người dùng nhưng chạy tốt trên máy lập trình viên. |
| `CREATE_NO_WINDOW` (Windows) | Cửa sổ console đen nháy lên mỗi lần gọi git. **Không thấy khi chạy `tauri dev`, chỉ thấy ở bản đóng gói release.** Phải kiểm bằng tệp `.exe` release thật. |

Ngoài ra, mọi thao tác *ghi* vào repo (stage, commit, merge, rebase, checkout) phải được
xếp hàng tuần tự qua một khoá duy nhất cho mỗi repo. Chạy song song sẽ đụng `index.lock`.

### 5.1 Đọc lịch sử

```bash
git log --all --topo-order \
  --format="%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%b%x1e" \
  --max-count=5000 --skip=0
```

- `%x1f` là ký tự phân tách trường (Unit Separator), `%x1e` phân tách bản ghi
  (Record Separator). Hai ký tự này gần như không bao giờ xuất hiện trong nội dung commit,
  nên phân tách an toàn hơn nhiều so với dùng dấu xuống dòng.
- `--topo-order` bảo đảm cha luôn đứng sau con, điều kiện bắt buộc cho thuật toán lane.
- Phân trang bằng `--skip` và `--max-count`.

### 5.2 Đọc tham chiếu (nhánh, tag)

```bash
git for-each-ref \
  --format="%(refname)%x1f%(objectname)%x1f%(upstream)%x1f%(upstream:track)%x1f%(HEAD)"
```

Một lệnh lấy hết nhánh local, nhánh remote, tag, kèm thông tin ahead/behind.

### 5.3 Đọc trạng thái thư mục làm việc

```bash
git status --porcelain=v2 --branch --untracked-files=all -z
```

`--porcelain=v2` là định dạng dành cho máy đọc, Git cam kết giữ ổn định. `-z` dùng ký tự
NUL phân tách nên an toàn với tên tệp chứa khoảng trắng hoặc ký tự đặc biệt.

### 5.4 Đọc khác biệt

```bash
# Diff của một commit so với cha
git diff --patch --no-color --find-renames <parent>..<commit>

# Diff của thư mục làm việc so với index
git diff --patch --no-color

# Diff của index so với HEAD
git diff --patch --no-color --cached
```

Kết quả là unified diff chuẩn, tự viết bộ phân tích khoảng 150–250 dòng.

**Khi lệnh diff trả về tên tệp thì phải dùng `-z`.** Bản thân nội dung bản vá thì không cần,
nhưng mọi biến thể liệt kê tên tệp đều cần, nếu không `core.quotepath` sẽ bọc và thoát ký tự
với tên tệp chứa dấu cách hoặc ký tự Unicode:

```bash
git diff --name-status -z --find-renames <parent>..<commit>
git diff --name-only -z --cached
```

Lý do giống hệt `git status --porcelain=v2 -z` ở mục 5.3.

### 5.5 Staging theo hunk

Đây là phần tinh tế nhất. Cách làm:

1. Lấy diff đầy đủ của tệp, **ghi lại mã băm blob của tệp tại thời điểm đọc**.
2. Người dùng chọn một hoặc vài hunk.
3. **Đọc lại trạng thái tệp và so mã băm với bước 1.** Nếu khác, tệp đã bị sửa từ lúc
   hiển thị diff — huỷ thao tác và báo người dùng làm mới, không được cố áp vào.
4. Dựng lại một bản vá chỉ chứa các hunk được chọn (giữ nguyên phần header của tệp).
5. Đưa vào git qua stdin:

```bash
git apply --cached -
```

Muốn bỏ staging thì thêm `--reverse`.

Chọn theo *từng dòng* thì phức tạp hơn: phải tự tính lại số đếm trong header `@@`, và
chuyển các dòng không được chọn thành dòng ngữ cảnh.

**Bước 3 không được bỏ qua.** Nếu người dùng (hoặc trình soạn thảo, hoặc một tiến trình
khác) sửa tệp trong khoảng thời gian giữa lúc giao diện vẽ diff và lúc bấm nút stage, bản
vá dựng lại sẽ không khớp với nội dung thật. Khi đó `git apply` hoặc báo lỗi, hoặc tệ hơn
là áp nhầm vị trí do cơ chế so khớp mờ. Cả Sourcetree lẫn Magit đều từng có lỗi thuộc đúng
lớp này. Cách xử lý đúng là báo cho người dùng biết tệp đã đổi, chứ không im lặng áp bừa.

**Các trường hợp biên của staging cần kiểm thử riêng:** tệp nhị phân, tệp không có dòng
trống cuối cùng (`\ No newline at end of file`), tệp vừa đổi tên vừa sửa nội dung, và
tương tác giữa chuẩn hoá xuống dòng CRLF/LF với việc chỉ stage một phần.

---

## 6. So sánh phương án nền tảng

| Tiêu chí | Tauri + React | Electron + React | Avalonia .NET |
|---|---|---|---|
| Kích thước cài đặt | 8–20 MB | 120–180 MB | 60–90 MB |
| RAM lúc rảnh | 80–150 MB | 300–600 MB | 100–200 MB |
| Thời gian khởi động | 0.3–0.8 giây | 1.5–4 giây | 0.5–1.5 giây |
| Hệ sinh thái giao diện | Toàn bộ web | Toàn bộ web | Hẹp hơn nhiều |
| Ngôn ngữ lớp dưới | Rust | Node.js | C# |
| Tốc độ xử lý dữ liệu | Rất nhanh | Trung bình | Nhanh |
| Đường học | Dốc (Rust) | Thoải | Trung bình |
| Đa nền tảng | Tốt | Tốt | Tốt |

**Quyết định: Tauri + React + TypeScript.**

Lý do: đây là ứng dụng *nặng về xử lý dữ liệu* (phân tích hàng chục nghìn commit, tính lane,
phân tích diff) nhưng *giao diện phức tạp* (graph, diff viewer, cây tệp). Tauri cho đúng cả
hai: Rust lo phần tính toán, web lo phần giao diện. Phần Rust cần viết không dùng tính năng
nâng cao — chủ yếu là sinh tiến trình, phân tích chuỗi, và cấu trúc dữ liệu.

---

## 7. Đánh giá rủi ro

| Rủi ro | Xác suất | Ảnh hưởng | Cách giảm thiểu |
|---|---|---|---|
| Phạm vi phình to (GitKraken có hàng trăm tính năng) | **Cao** | **Cao** | Chốt cứng danh sách MVP ở mục 2.4. Mọi thứ khác vào backlog, không bàn lại. |
| Hiệu năng trên repo rất lớn | Trung bình | Cao | Phân trang, ảo hoá, tính lane trong Rust, cache diff. Kiểm thử sớm trên repo lớn. |
| Giải quyết xung đột merge | Trung bình | Trung bình | v1 chỉ *phát hiện* xung đột và liệt kê tệp. Mở trình soạn thảo ngoài để sửa. Trình giải quyết đồ hoạ để v2. |
| Khác biệt hành vi trên Windows (đường dẫn, CRLF, khoá tệp) | Cao | Trung bình | Phát triển chính trên Windows. Dùng `PathBuf` phía Rust, chuẩn hoá dấu gạch chéo ở ranh giới IPC. |
| Trình xem diff tự viết bị lỗi hiển thị | Trung bình | Trung bình | Dùng CodeMirror 6 thay vì tự viết. Đã có sẵn tô màu cú pháp, ảo hoá, chế độ merge. |
| Rust là rào cản với nhóm quen web | Trung bình | Trung bình | Giữ lớp Rust mỏng và thuần tuý: sinh tiến trình + phân tích + trả JSON. Không async phức tạp, không lifetime rắc rối. |
| Xác thực (SSH key, token) | Thấp | Cao | Không tự làm. Gọi `git` CLI thì credential helper của hệ thống tự lo. |

---

## 8. Ước lượng công sức (một lập trình viên toàn thời gian)

| Giai đoạn | Nội dung | Thời gian |
|---|---|---|
| 0 | Dựng khung dự án, IPC, bố cục ba cột | 1 tuần |
| 1 | Đọc lịch sử, thuật toán lane, vẽ graph, danh sách ảo hoá | 2–3 tuần |
| 2 | Bảng chi tiết commit, danh sách tệp, trình xem diff | 2 tuần |
| 3 | Thư mục làm việc, staging theo tệp và theo hunk, commit, amend | 2 tuần |
| 4 | Thao tác nhánh và remote: fetch, pull, push, merge, rebase, stash | 2–3 tuần |
| 5 | Tích hợp AI soạn commit message | 3–5 ngày |
| 6 | Đánh bóng, xử lý lỗi biên, bộ cài đặt, tự cập nhật | 2–3 tuần |
| | **Tổng đến MVP** | **11–14 tuần** |

Bản chỉ đọc (xong giai đoạn 2) dùng được sau khoảng 5–6 tuần.

---

## 9. Kết luận

Dự án **khả thi**. Không có hạng mục nào cần nghiên cứu mới:

- Backend git: gọi CLI, đầu ra có định dạng ổn định dành cho máy đọc.
- Thuật toán graph: đã biết, khoảng 150 dòng.
- Trình xem diff: dùng thư viện có sẵn.
- Nền tảng: Tauri v2 đã ổn định.

Rủi ro lớn nhất không nằm ở kỹ thuật mà ở **kỷ luật phạm vi**. GitKraken là sản phẩm của
nhiều năm với cả một đội ngũ. Muốn ra được bản dùng thật, phải chấp nhận làm ít tính năng
hơn nhiều và làm cho tốt phần đã chọn.
