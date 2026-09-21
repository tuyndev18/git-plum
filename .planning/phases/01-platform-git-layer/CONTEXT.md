# Phase 1: Nền tảng và lớp bọc git — Context

**Gathered:** 2026-09-21
**Status:** Ready for planning
**Source:** Đối chiếu trực tiếp mã nguồn đã có với ROADMAP.md

---

<domain>
## Phase Boundary

Phase này chứng minh **cầu IPC và lớp sinh tiến trình git chạy đúng**, và chốt bốn ràng
buộc kiến trúc không sửa được về sau. Không có bộ phân tích đầu ra git, không có kiểu miền
nào ngoài repo handle tối thiểu — logic git thật bắt đầu từ Phase 2.

Người dùng cuối phase phải làm được: mở một repository thật qua hộp thoại, thấy ứng dụng
trả lời bằng dữ liệu git thật, thấy nhật ký liệt kê đúng lệnh đã chạy, và nhận thông báo
đọc hiểu được khi chọn nhầm thư mục.

</domain>

---

<existing_work>
## Đã làm rồi — commit `1002fa5`

**Quan trọng cho người lập kế hoạch: phần lớn Phase 1 đã có mã chạy được.** Kế hoạch phải
là *hoàn thiện và kiểm chứng phần còn thiếu*, không phải viết lại từ đầu. Đọc mã trước khi
lập kế hoạch cho bất kỳ mục nào bên dưới.

### Đã xong và đã kiểm chứng

| Requirement | Nơi cài đặt | Trạng thái |
|---|---|---|
| PLAT-01 | `src-tauri/`, `src/` | ✅ `tauri build --debug` cho ra `git-plum.exe` chạy được. **Chưa kiểm trên macOS và Linux.** |
| PLAT-02 | `src-tauri/src/git/exec.rs` | ⚠️ Gần xong — thiếu 3 mục, xem bảng khoảng cách |
| PLAT-03 | `src-tauri/src/state/mod.rs`, `git/runner.rs` | ✅ Mutex ghi theo từng repo, có test chứng minh khoá chặn đúng |
| PLAT-04 | `src/lib/commands.ts` | ✅ Sổ đăng ký đầy đủ, ba lệnh đã đăng ký. **Chưa có test.** |
| PLAT-05 | `src-tauri/src/state/mod.rs`, `src/stores/repoStore.ts` | ✅ `HashMap<RepoId, _>` và `byRepo[repoId]`, có test nhiều repo cùng tồn tại |
| PLAT-06 | `src-tauri/src/commands/repo.rs` | ✅ `open_repository` dùng `rev-parse --show-toplevel` |
| PLAT-08 | `src-tauri/src/state/command_log.rs`, `src/components/CommandLogPanel.tsx` | ✅ Ghi và hiển thị, 5 test |
| PLAT-09 | `src/components/AppLayout.tsx` | ⚠️ Có `useDefaultLayout` nhưng **chưa kiểm chứng kích thước còn nguyên sau khi đóng mở lại** |
| PLAT-10 | `src-tauri/src/error.rs` | ✅ `GitError` + `Serialize` viết tay, mang theo lệnh đã chạy, 2 test |
| REL-04 | `.github/workflows/ci.yml` | ⚠️ Đã viết, dùng `libwebkit2gtk-4.1-dev` đúng. **Chưa chạy lần nào — repo chưa có remote.** |

### Kiểm chứng đã chạy thật

```
cargo build              ✅
cargo test               ✅ 19/19
cargo clippy -D warnings ✅ sạch
cargo fmt --check        ✅
npm run typecheck        ✅
npm run build            ✅ 268KB (gzip 84KB)
tauri build --debug      ✅ git-plum.exe 16MB
npm audit                ✅ 0 lỗ hổng
```

### Năm thứ đã sửa nhờ chạy thật — đừng để tái diễn

1. `vite@8.0.0` có 5 lỗ hổng dev-server → đã nâng `8.3.0`
2. Vite 8 dùng Rolldown, không có esbuild → đã bỏ `minify: 'esbuild'`
3. `react-resizable-panels` **v4 đổi hết API**: `Group`/`Separator`/`orientation`,
   không phải `PanelGroup`/`PanelResizeHandle`/`direction`. Ví dụ viết cho v3 sẽ không chạy.
4. `thiserror` coi field tên `source` là lỗi nguồn → đã đổi thành `reason`
5. Phiên bản plugin npm phải khớp crate Rust theo minor, nếu không `tauri build` chặn hẳn.
   Hiện ghim: dialog `2.7.3`, store `2.4.5`.

</existing_work>

---

<gaps>
## Khoảng cách — đây mới là việc của phase này

### G1. PLAT-02 thiếu ba mục ghim cấu hình

ROADMAP đòi ghim `--cleanup=whitespace`, `diff.noprefix`, `format.coverLetter`.
`exec.rs` hiện chỉ ghim `log.showSignature=false` qua `GIT_CONFIG_PARAMETERS`.

> **Đính chính (2026-09-21):** bản đầu của mục này còn liệt kê `diff.external` là thiếu.
> Sai. Nó đã được ghim từ commit `1002fa5`, bằng biến `GIT_EXTERNAL_DIFF=""` tại
> `src-tauri/src/git/exec.rs:199` chứ không qua `GIT_CONFIG_PARAMETERS` — biến môi trường
> thắng cấu hình nên cách này mạnh hơn. Chỉ còn `diff.noprefix` và `format.coverLetter`
> là thiếu thật.

Cả bốn phải nằm cùng một biến, dạng
`'log.showSignature=false' 'diff.noprefix=false' 'format.coverLetter=false'`.
`--cleanup=whitespace` là tham số dòng lệnh của `git commit`, không phải biến môi trường —
cần chỗ đặt riêng, và Phase 1 chưa có lệnh commit nên có thể chỉ cần ghi chú lại cho Phase 4.

Cần test chứng minh **từng biến** thực sự tới được tiến trình con, không chỉ đọc mã bằng mắt.

### G2. PLAT-07 chưa có — danh sách repository gần đây

Chưa cài đặt. `tauri-plugin-store` đã khai báo trong `Cargo.toml`, `package.json` và
`capabilities/default.json` nhưng chưa dùng ở đâu.

Cần: lưu danh sách repo đã mở, hiện ở màn hình trống, mở lại được bằng một lần bấm.
Tiêu chí thành công số 1 của phase đòi hỏi mục này.

### G3. PLAT-09 chưa được kiểm chứng

`useDefaultLayout` có trong mã nhưng chưa ai xác nhận kích thước vùng còn nguyên sau khi
đóng và mở lại ứng dụng. Đây là tiêu chí thành công số 2.

Lưu ý: `useDefaultLayout` ghi vào `localStorage`, mà `localStorage` trong webview Tauri
tồn tại qua các lần chạy — nhưng phải kiểm thật, không được giả định.

### G4. Chưa có test phía giao diện

Không có tệp test nào trong `src/`. `vitest` và `@testing-library/react` đã cài, `npm test`
đã khai báo trong `package.json`, nhưng chưa có gì để chạy. Job `frontend` trong CI sẽ chạy
`npm test` và không có test nào.

Cần ít nhất: sổ đăng ký lệnh (`src/lib/commands.ts`) và `repoStore` — hai thứ mang ràng buộc
kiến trúc PLAT-04 và PLAT-05.

### ~~G5. Chưa kiểm chứng trên macOS và Linux~~ — ĐƯA RA NGOÀI PHẠM VI PHASE 1

Chủ dự án đã quyết định **không tạo remote** ở giai đoạn này, nên CI không chạy được và
không có cách nào kiểm chứng macOS/Linux từ một máy Windows.

Hệ quả, ghi rõ để không ai nhầm:
- **REL-04 trong Phase 1 chỉ đạt tới mức "tệp `ci.yml` viết đúng"**, không phải "CI đã chạy
  xanh". Đây là cách diễn giải yếu hơn ý định ban đầu của ROADMAP.
- **PLAT-01 trong Phase 1 chỉ chứng minh được trên Windows.** Phần macOS và Linux vẫn là
  giả định chưa kiểm.
- Rủi ro đã biết và chấp nhận: lỗi riêng của Linux (đặc biệt phiên bản WebKitGTK) và của
  macOS sẽ chỉ lộ ra khi tới việc đóng gói phát hành ở Phase 8, lúc đó sửa đắt hơn.
- Khi nào có remote thì chạy CI là xong ngay — tệp cấu hình đã sẵn sàng.

Người lập kế hoạch **không** đưa việc kiểm chứng đa nền tảng vào Phase 1.

### G6. Validation checkpoint #7 — locale không phải tiếng Anh

ROADMAP ghi rõ: phải chạy kiểm thử và QA thủ công với locale hệ thống không phải tiếng Anh.

> **Đính chính (2026-09-21):** bản đầu của mục này viết "máy phát triển là Windows tiếng Việt,
> nên kiểm được ngay". **Sai, và đây là giả định của tôi chứ không phải dữ liệu.** Khi plan
> 01-04 đo thật, sáu nguồn độc lập đều cho `en-US`: `Get-Culture`, `CurrentUICulture`,
> `Get-WinUserLanguageList`, `Get-WinSystemLocale`, registry
> `HKCU\Control Panel\International`, và `systeminfo`. Không có gói ngôn ngữ hiển thị tiếng
> Việt nào được cài.
>
> Nặng hơn: bản Git for Windows trên máy này **không có catalog dịch nào** (0 tệp `.mo`).
> Kiểm bằng hành vi: `git rev-parse --show-toplevel` ngoài repo với `LC_ALL=vi_VN.UTF-8` vẫn
> trả `fatal: not a git repository` y nguyên tiếng Anh.
>
> Nghĩa là **checkpoint #7 không thể kiểm trọn trên máy này**, dù ghim môi trường có đúng hay
> sai. Kết luận đã chốt: **đạt một phần** — phần lớp ghim `LC_ALL=C` có bằng chứng thật (ép
> `vi_VN.UTF-8` ở tiến trình cha, `cargo test` vẫn 23 đỗ y nguyên); phần "hệ điều hành thật sự
> nói ngôn ngữ khác" cần một máy hoặc máy ảo khác, làm trước khi phát hành công khai ở Phase 8.
> Chi tiết đầy đủ ở `.planning/STATE.md` và `docs/03-phase1-qa-windows.md` Phụ lục A.

### G8. PLAT-09 chưa trọn — trạng thái bật tắt bảng nhật ký không sống sót qua lần khởi động

Phát hiện khi rà soát kế hoạch, không phải lúc viết mã. Đã kiểm chứng trực tiếp trong mã:

- `src/App.tsx:18` — `const [logVisible, setLogVisible] = useState(true)`. Là state React
  thuần, không ghi vào đâu cả. Mỗi lần mở lại ứng dụng luôn quay về bật.
- `src/components/AppLayout.tsx` — `Group` ngoài (mang `useDefaultLayout` của
  `git-plum-outer`) chỉ được render khi prop `bottom` khác `undefined`.

Hai điều trên cộng lại gây hai hệ quả:

1. Người dùng tắt bảng nhật ký rồi thoát, mở lại thấy nó bật trở lại. Đây là **một phần
   của bố cục không được ghi nhớ**, trong khi PLAT-09 nói kích thước và bố cục phải còn
   nguyên sau khi mở lại.
2. Khi bảng nhật ký đang tắt, `Group` ngoài bị gỡ khỏi cây React nên `git-plum-outer`
   không ghi thêm gì. Kịch bản kiểm thử nào không đụng tới nút bật tắt sẽ **đỗ nhầm**.

Ngoài ra, tài liệu của `react-resizable-panels` v4 cảnh báo riêng cho trường hợp `Group`
chứa `Panel` render có điều kiện: phải truyền `panelIds` cho `useDefaultLayout`, nếu không
bố cục lần đầu sẽ sai. `AppLayout` đúng là trường hợp đó. Đây là nguyên nhân khả dĩ nhất
nếu phần kiểm bố cục trượt.

Thêm một điểm cho Phase 2 biết: tham số `groupId` của `useDefaultLayout` **đã bị đánh dấu
deprecated** trong v4, thay bằng `id`. Mã hiện dùng `groupId`, vẫn chạy nhưng nên đổi.

### G7. Tiêu chí thành công số 5 chưa kiểm — cửa sổ console

Phải chạy bản dựng **release** từ Explorer (không phải `tauri dev`) và làm vài thao tác git,
xác nhận không có cửa sổ console đen nào nháy lên. Cờ `CREATE_NO_WINDOW` đã đặt trong mã,
nhưng hiện tượng này *chỉ hiện ở bản release*, nên đọc mã không thay được việc chạy thử.

</gaps>

---

<decisions>
## Quyết định đã chốt

### Không bàn lại
- Tauri v2 + React + TypeScript + Rust, gọi thẳng `git` CLI. Không libgit2, không gitoxide.
- Không thêm `tauri-plugin-shell`: git sinh từ Rust bằng `tokio::process`, đưa quyền dựng
  tham số vào webview là vứt bỏ lợi ích bảo mật của việc giữ git ở lớp Rust.
- `keyring` crate 4.2 cho khoá API (Phase 7). `tauri-plugin-stronghold` vi phạm ràng buộc
  bảo mật vì cần mật khẩu chủ do người dùng nhập.
- `typescript@6.0.3`, không phải 7.x — `typescript-eslint` khai báo phạm vi phụ thuộc `<6.1.0`.
- Không cam kết tổng thời gian. Chuỗi phase là kế hoạch, thời lượng là đầu ra.

### Windows là nền tảng ưu tiên — quyết định của chủ dự án, 2026-09-21

Toàn bộ công sức Phase 1 dồn vào Windows. Cụ thể:

- Mọi tiêu chí thành công được kiểm trên Windows 11. Đạt trên Windows là phase xong.
- **Không** tạo remote, **không** chạy CI, **không** kiểm macOS hay Linux trong phase này.
- Mã viết ra vẫn phải đa nền tảng về nguyên tắc — dùng `PathBuf` thay vì nối chuỗi đường dẫn,
  giữ phần riêng của Windows trong `#[cfg(windows)]`, không ghim đường dẫn tuyệt đối kiểu
  `C:\`. Tức là **không tự trói mình vào Windows**, chỉ là chưa kiểm hai nền tảng kia.
- Tệp `.github/workflows/ci.yml` giữ nguyên cả ba nền tảng. Nó đúng, chỉ là chưa chạy.

### Ranh giới phạm vi phase này
- **Không** viết bộ phân tích đầu ra git nào. Không `git log`, không `git status`,
  không `git diff`. Phase 2 mới bắt đầu.
- **Không** thêm kiểu miền nào ngoài `RepoInfo` đã có.
- Lệnh `current_branch` hiện có là *lệnh mẫu chứng minh chuỗi IPC chạy thông*, không phải
  khởi đầu của lớp đọc dữ liệu.

### Claude's Discretion
- Cách tổ chức test phía giao diện (gộp một tệp hay tách theo module).
- Hình thức hiển thị danh sách repository gần đây.
- Có gỡ `SSH_ASKPASS`, `GIT_PAGER`, `PAGER` khỏi `exec.rs` hay giữ — chúng không nằm trong
  yêu cầu của ROADMAP nhưng vô hại và có ích.

</decisions>

---

<canonical_refs>
## Canonical References

**Người lập kế hoạch và người thực thi phải đọc trước khi làm.**

### Mã nguồn đã có — đọc trước tiên
- `src-tauri/src/git/exec.rs` — lớp sinh tiến trình, PLAT-02
- `src-tauri/src/git/runner.rs` — chạy có ghi nhật ký và có khoá ghi, PLAT-03 + PLAT-08
- `src-tauri/src/state/mod.rs` — trạng thái khoá theo repo, PLAT-05
- `src-tauri/src/state/command_log.rs` — nhật ký lệnh, PLAT-08
- `src-tauri/src/error.rs` — `GitError` và `Serialize` viết tay, PLAT-10
- `src-tauri/src/commands/repo.rs` — các Tauri command, PLAT-06
- `src/lib/commands.ts` — sổ đăng ký lệnh, PLAT-04
- `src/stores/repoStore.ts` — trạng thái `byRepo`, PLAT-05
- `src/components/AppLayout.tsx` — bố cục ba vùng, PLAT-09
- `.github/workflows/ci.yml` — CI ba nền tảng, REL-04

### Tài liệu kế hoạch
- `.planning/ROADMAP.md` — mục "Phase 1", đặc biệt khối "Ràng buộc bắt buộc"
- `.planning/REQUIREMENTS.md` — nhóm PLAT và REL-04
- `.planning/PROJECT.md` — Core Value, bảng Key Decisions

### Nghiên cứu
- `.planning/research/PITFALLS.md` — cạm bẫy 1, 2, 5, 6 thuộc phase này
- `.planning/research/STACK.md` — phiên bản đã chốt, danh sách "không dùng"
- `.planning/research/ARCHITECTURE.md` — ranh giới module Rust và React
- `docs/01-research-competitors.md` mục 5.0 — ghim môi trường tiến trình git

</canonical_refs>

---

<success_criteria>
## Tiêu chí thành công của phase — chép từ ROADMAP

1. Người dùng chọn một thư mục repository qua hộp thoại, ứng dụng nhận đúng repo đó, và lần
   mở sau repo xuất hiện trong danh sách gần đây
2. Người dùng kéo ranh giới ba vùng giao diện và kích thước đó còn nguyên sau khi đóng mở lại
3. Người dùng nhìn thấy bảng nhật ký liệt kê đúng từng lệnh git ứng dụng đã chạy, kèm mã thoát
   và thời gian chạy
4. Khi chọn một thư mục không phải repository, người dùng nhận thông báo đọc hiểu được kèm lệnh
   đã chạy — ứng dụng không treo và không im lặng
5. Chạy bản dựng release từ Explorer (không phải `tauri dev`) và thực hiện vài thao tác git:
   không có cửa sổ console đen nào nháy lên

Tiêu chí 1 cần G2. Tiêu chí 2 cần G3. Tiêu chí 5 cần G7. Ba tiêu chí còn lại đã đạt nhưng
phải chạy lại để xác nhận sau khi sửa.

</success_criteria>

---

<blockers>
## Vướng mắc đang mở

Không có vướng mắc nào chặn việc lập kế hoạch. Vấn đề CI đã được chủ dự án quyết: không tạo
remote lúc này, tập trung Windows trước.

</blockers>

---
*Context gathered: 2026-09-21*
