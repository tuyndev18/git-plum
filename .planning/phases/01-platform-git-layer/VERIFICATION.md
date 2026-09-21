# Phase 1 — Kiểm chứng: Nền tảng và lớp bọc git

**Ngày:** 2026-09-21
**Kết luận:** Đạt phần tự động hoá được. **Ba tiêu chí thành công chưa kiểm chứng** vì phần
QA thủ công bị hoãn theo quyết định của chủ dự án.

---

## 1. Kết quả kiểm thử tự động — số thật, chép nguyên

| Lệnh | Kết quả | Mốc trước phase |
|---|---|---|
| `cargo test` (trong `src-tauri`) | **23 đỗ**, 0 lỗi, 3 suite | 19 đỗ |
| `cargo clippy --all-targets -- -D warnings` | sạch | sạch |
| `cargo fmt --all --check` | exit 0 | exit 0 |
| `npm test` | **57 đỗ**, 5 tệp | **exit 1 — không có tệp test nào** |
| `npm run typecheck` | exit 0 | exit 0 |
| `npm run build` | 273.83 kB (gzip 85.96 kB) | 268.01 kB |
| `npx tauri build` (release đầy đủ) | `git-plum.exe` 4.06 MB + MSI 1.98 MB | chưa dựng |

Thay đổi đáng kể nhất: `npm test` từ **thoát mã 1** thành **57 test đỗ**. Job `frontend`
trong `.github/workflows/ci.yml` trước phase này chạy vào khoảng không — không có tệp cấu
hình vitest nào tồn tại.

Bản release đã được xác minh khách quan là đúng loại artifact: đọc PE header của
`git-plum.exe` cho **subsystem = 2 (WINDOWS_GUI)**; bản `--debug` sẽ là 3 (WINDOWS_CUI).
Tức `windows_subsystem = "windows"` trong `main.rs` thật sự đang hoạt động.

---

## 2. Năm tiêu chí thành công

| # | Tiêu chí | Trạng thái |
|---|---|---|
| 1 | Mở repo qua hộp thoại, nhận đúng repo, lần sau xuất hiện trong danh sách gần đây | ⚠️ **Chưa kiểm chứng** — mã đã viết và có 57 test, nhưng `tauri-plugin-store` bị **giả lập** trong mọi test. Chưa ai xác nhận `recent-repos.json` ghi đúng thư mục dữ liệu ứng dụng, hay `autoSave` kịp ghi xuống đĩa trước khi tiến trình thoát. |
| 2 | Kéo ranh giới ba vùng, kích thước còn nguyên sau khi đóng mở lại | ⚠️ **Chưa kiểm chứng** — và đã biết **không trọn vẹn**, xem G8 bên dưới. |
| 3 | Nhật ký liệt kê đúng từng lệnh git, kèm mã thoát và thời gian chạy | ✅ Có 5 test Rust cho `command_log`. Phần hiển thị (`CommandLogPanel.tsx`) chưa có test render, chỉ có KB-3 thủ công — chưa chạy. |
| 4 | Thư mục không phải repo cho thông báo đọc hiểu được kèm lệnh đã chạy, không treo | ✅ `GitError::NotARepository` có test. `GIT_TERMINAL_PROMPT=0` + stdin đóng + hạn giờ 30s chặn treo, có test hạn giờ. |
| 5 | Bản release chạy từ Explorer, không nháy cửa sổ console | ⚠️ **Chưa kiểm chứng** — hiện tượng này *chỉ* xuất hiện ở bản release, nên đọc mã không thay được việc chạy thử. Lớp 1 (`windows_subsystem`) đã xác minh qua PE header. Lớp 2 (`CREATE_NO_WINDOW` cho từng tiến trình git con) vẫn chưa biết. |

**Ba tiêu chí 1, 2, 5 cần đúng một việc để đóng:** bấm đúp
`src-tauri/target/release/git-plum.exe` và chạy sáu kịch bản trong
`docs/03-phase1-qa-windows.md`. Bản release đã dựng sẵn, không phải chờ gì.

---

## 3. Bảy khoảng cách đã xác định đầu phase

| Mã | Nội dung | Kết quả |
|---|---|---|
| G1 | PLAT-02 thiếu ghim `diff.noprefix`, `format.coverLetter` | ✅ Đóng. Hằng `PINNED_GIT_CONFIG` trong `exec.rs`, 4 test chạy git thật đọc ngược giá trị từ tiến trình con. Kiểm chứng đột biến: xoá một khoá → đúng 2 test đỏ. |
| G2 | PLAT-07 danh sách repo gần đây chưa tồn tại | ⚠️ Mã xong, test xong (`recentRepos.ts`, `RecentRepoList.tsx`), **kiểm chứng thật chưa** — plugin bị giả lập. |
| G3 | PLAT-09 chưa ai xác nhận bố cục sống sót qua khởi động lại | ⚠️ Chưa kiểm chứng. |
| G4 | Không có hạ tầng test giao diện | ✅ Đóng. Cấu hình vitest trong khối `test` của `vite.config.ts`, `resolve.alias` cho `@`, 57 test. Kiểm chứng đột biến trên cả hai bộ test ràng buộc kiến trúc. |
| G5 | Kiểm chứng macOS/Linux | ⛔ **Ngoài phạm vi** theo quyết định chủ dự án 2026-09-21: chưa tạo remote, tập trung Windows. |
| G6 | Validation checkpoint #7 — locale khác tiếng Anh | ⚠️ **Đạt một phần.** Lớp ghim chứng minh được (ép `vi_VN.UTF-8` ở tiến trình cha → `cargo test` vẫn 23 đỗ y nguyên). Phần "hệ điều hành thật sự nói ngôn ngữ khác" không kiểm được: máy là `en-US` và Git for Windows này không có catalog dịch nào. |
| G7 | Tiêu chí 5 — cửa sổ console ở bản release | ⚠️ Chưa kiểm chứng. |
| G8 | PLAT-09 chưa trọn — trạng thái bật tắt bảng nhật ký không persist | ⚠️ **Defect đã xác nhận, chưa sửa.** Xem mục 5. |

---

## 4. Mười một requirement

| Requirement | Trạng thái | Bằng chứng |
|---|---|---|
| PLAT-01 | ⚠️ Đạt **chỉ trên Windows** | `tauri build` cho `.exe` chạy được. macOS/Linux ngoài phạm vi. |
| PLAT-02 | ✅ | Một lớp bọc duy nhất, ghim đủ env + config, 7 test trong `git::exec` |
| PLAT-03 | ✅ | `tokio::sync::Mutex` theo từng repo, test chứng minh khoá chặn đúng |
| PLAT-04 | ✅ | Sổ đăng ký lệnh, 12 test. Đột biến: gỡ khối `throw` → 2 test đỏ |
| PLAT-05 | ✅ | `HashMap<RepoId, _>` bên Rust, `byRepo[repoId]` bên React, 10 test. Đột biến: bỏ `...s.byRepo` → 2 test đỏ |
| PLAT-06 | ✅ | `rev-parse --show-toplevel`, nhận đúng repo cha từ thư mục con |
| PLAT-07 | ⚠️ Mã xong, chưa kiểm thật | Plugin store giả lập trong test |
| PLAT-08 | ✅ | 5 test Rust. Phần hiển thị chưa có test render |
| PLAT-09 | ⚠️ **Không trọn** | Kích thước vùng: chưa kiểm. Bật tắt bảng nhật ký: **defect đã xác nhận** (G8) |
| PLAT-10 | ✅ | `GitError` + `Serialize` viết tay mang theo lệnh đã chạy, 2 test |
| REL-04 | ⚠️ **Diễn giải yếu hơn ROADMAP** | Chỉ đạt "tệp `ci.yml` soát bằng mắt", **không** phải "CI chạy xanh". Đã soát: `libwebkit2gtk-4.1-dev` (không phải 4.0), `ubuntu-22.04` (không phải `ubuntu-latest`), `working-directory: src-tauri` ở mọi bước cargo, ma trận đủ ba nền tảng, job `frontend` nay có 57 test thật. |

---

## 5. Khoảng cách còn mở khi đóng phase

### G8 — trạng thái bật tắt bảng nhật ký không sống sót qua khởi động lại

**Defect đã xác nhận bằng cách đọc mã, không phải giả thuyết.**

- `src/App.tsx` — `const [logVisible, setLogVisible] = useState(true)`. State React thuần,
  không ghi vào đâu. Mỗi lần mở lại ứng dụng luôn quay về bật.
- `src/components/AppLayout.tsx` — `Group` ngoài (thứ mang `useDefaultLayout` cho
  `git-plum-outer`) chỉ được render khi prop `bottom` khác `undefined`. Tắt bảng nhật ký là
  gỡ luôn `Group` đó khỏi cây React.

Hệ quả thứ hai đáng chú ý hơn hệ quả thứ nhất: **kịch bản kiểm thử nào không đụng tới nút
bật tắt sẽ đỗ nhầm**, trong khi một nửa PLAT-09 vẫn hỏng. Đây chính là lý do KB-2b tồn tại
như một bước riêng.

Chưa sửa vì đây là quyết định phạm vi, không phải lỗi cài đặt: PLAT-09 nói "kích thước còn
nguyên", không nói rõ trạng thái hiển thị panel có thuộc "bố cục" hay không.

Thêm một điểm cho người làm Phase 2: tài liệu `react-resizable-panels` v4 cảnh báo rằng
`Group` chứa `Panel` render có điều kiện **phải** truyền `panelIds` cho `useDefaultLayout`,
nếu không bố cục lần đầu sẽ sai. `AppLayout` đúng là trường hợp đó và hiện **không** truyền.
Ngoài ra tham số `groupId` đã bị đánh dấu deprecated trong v4, thay bằng `id`.

### QA thủ công chưa chạy

Sáu kịch bản KB-1 đến KB-5 trong `docs/03-phase1-qa-windows.md` chưa được thực hiện. Bản
release đã dựng sẵn tại `src-tauri/target/release/git-plum.exe`. Đây là nợ kỹ thuật có thể
trả bất cứ lúc nào, không phụ thuộc tiến độ Phase 2.

### Rủi ro đã biết và chấp nhận

| Rủi ro | Vì sao chấp nhận | Khi nào trả |
|---|---|---|
| Lỗi riêng của Linux/macOS ẩn tới Phase 8 | Chưa tạo remote nên CI không chạy được. Quyết định của chủ dự án. | Trước khi đóng gói phát hành |
| Parser có thể vỡ khi git nói ngôn ngữ khác | Không kiểm được trên máy `en-US` với git không có bản dịch. Lớp ghim môi trường đã có bằng chứng thật, và mọi tiến trình git buộc đi qua một hàm duy nhất. | Trước khi phát hành công khai |
| PLAT-07 chưa kiểm chứng thật | Mã và test đầy đủ; rủi ro là đường ghi đĩa, không phải logic | Khi chạy KB-1 |

---

## 6. Ghi chú cho người đọc sau

Ba điều đáng nhớ từ phase này, vì chúng đều là **giả định bị dữ liệu bác bỏ**:

1. **`diff.external` không hề thiếu.** CONTEXT.md ban đầu ghi là thiếu; thực ra nó đã được
   ghim bằng `GIT_EXTERNAL_DIFF=""` từ commit đầu tiên — biến môi trường, mạnh hơn config.

   > 🔴 **ĐÍNH CHÍNH 2026-09-22 (tìm thấy ở plan 03-01).** Điều trên đúng về *vị trí* nhưng
   > **sai về cơ chế**, và bản thân nó là một lỗi. `GIT_EXTERNAL_DIFF=""` **không** vô hiệu
   > hoá trình diff ngoài: git đem chuỗi rỗng đi spawn như tên chương trình và chết với
   > `error: cannot spawn : No such file or directory` / `fatal: external diff died`.
   >
   > Hệ quả: **mọi lệnh git sinh bản vá thoát 128 với stdout rỗng, im lặng** — không lỗi
   > biên dịch, không test nào của Phase 1 hay Phase 2 đỏ. Phase 2 sống sót vì nó chỉ dùng
   > `--name-status`, lệnh không gọi tới trình diff. Lỗi chỉ lộ ở Phase 3, nơi đọc bản vá là
   > toàn bộ mục đích — gồm cả `--word-diff=porcelain` mà plan 03-03 dựa vào.
   >
   > Ghim `diff.external=` rỗng qua config có **cùng** lỗi (đã đo). Cơ chế đúng là cờ
   > `--no-ext-diff` của chính git, thắng cả config lẫn biến môi trường. Đã sửa ở `f5c4c17`:
   > `env_remove("GIT_EXTERNAL_DIFF")` cộng chèn `--no-ext-diff` tập trung trong
   > `GitCommand::run()` cho `diff`/`show`/`log`/`diff-tree`.
   >
   > **Bài học đúng của mục này hoá ra ngược lại lời nó viết:** một ràng buộc "đã ghim rồi"
   > mà chưa có test đọc ngược kết quả từ tiến trình con thì chưa được ghim. Mục 1 gốc kết
   > luận từ việc *đọc mã*, không từ việc *chạy git* — và đọc mã thì cả ý định lẫn lỗi đều
   > trông như nhau.
2. **Máy phát triển không phải Windows tiếng Việt.** Sáu nguồn độc lập cho `en-US`. Giả định
   này làm cả một khoảng cách (G6) được lên kế hoạch sai.
3. **`npm test` chưa từng chạy.** `package.json` khai báo `"test": "vitest run"` nhưng không
   có tệp cấu hình nào, nên CI job `frontend` chạy vào khoảng không mà vẫn "xanh" trong tưởng
   tượng của người viết.

Cả ba đều chỉ lộ ra khi có người chạy lệnh thay vì đọc mã.

---
*Kiểm chứng ghi: 2026-09-21*
