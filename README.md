# git-plum

Ứng dụng desktop mã nguồn mở để quản lý repository Git bằng giao diện đồ hoạ, dành cho lập
trình viên đã quen dùng Git nhưng muốn một công cụ nhanh và nhẹ hơn GitKraken, đồng thời đầy
đủ tính năng hơn GitHub Desktop.

## Core Value

Đọc và hiểu lịch sử của một repository phải **tức thì** — đồ thị commit mở ra trong dưới một
giây và cuộn mượt kể cả trên repo hàng chục nghìn commit. Nếu mọi tính năng khác thất bại,
riêng điều này vẫn phải đúng.

## Tính năng hiện có

Dự án đang trong quá trình phát triển tích cực theo mô hình MVP theo từng phase. Các mục dưới
đây **đã có mã chạy được**, nhưng một số checkpoint kiểm chứng bằng mắt người và hiệu năng vẫn
còn treo — xem [`.planning/ROADMAP.md`](.planning/ROADMAP.md) để theo dõi tiến độ chi tiết theo
từng requirement. Không mục nào dưới đây được xem là "hoàn tất" hay "sẵn sàng dùng"; các mục
này là "đã có" hoặc "đang phát triển".

**Nền tảng**
- Mở repository, bố cục ba vùng (đồ thị / chi tiết commit / diff)
- Nhật ký lệnh git tập trung
- Danh sách repository gần đây (lưu qua `tauri-plugin-store`)

**Đọc lịch sử** (đang phát triển, chưa qua kiểm chứng cuối phase)
- Đồ thị nhánh nhiều lane vẽ trên canvas
- Danh sách commit ảo hoá (`@tanstack/react-virtual`), cuộn được trên repo lớn
- Chi tiết commit, tìm kiếm theo thông điệp/tác giả/mã commit/đường dẫn tệp
- Nhãn nhánh và tag trên đồ thị

**Xem khác biệt** (đang phát triển, chưa qua kiểm chứng cuối phase)
- Trình xem diff CodeMirror 6 với tô màu cú pháp nạp lười theo phần mở rộng tệp
- Hai chế độ hiển thị: hợp nhất (unified) và hai cột (split), nhảy khối thay đổi
- Diff mức từ (word-level diff), bật/tắt hiển thị khoảng trắng
- Lịch sử của một tệp riêng lẻ (theo dõi cả đổi tên qua `git log --follow`)
- Nhận diện tệp nhị phân, tệp quá lớn, và con trỏ Git LFS

## Lộ trình (chưa có)

Các nhóm sau **chưa có mã**, chỉ nằm trong kế hoạch:

- Vòng lặp commit (tạo commit, amend)
- Staging theo khối thay đổi (hunk) và an toàn khi huỷ (discard)
- Nhánh, remote, xung đột merge/rebase
- Hỗ trợ AI soạn commit message
- Phát hành: bộ cài đặt, tự cập nhật (updater)

## Tech stack

**Frontend**
- Tauri v2 (`@tauri-apps/api` 2.11.1, `@tauri-apps/cli` 2.11.5)
- React 19.3.0 + TypeScript 6.0.3 + Vite 8.3.0
- Zustand 5.0.15 (state), `@tanstack/react-virtual` 3.14.13 (ảo hoá danh sách)
- CodeMirror 6 (`@codemirror/merge` 6.12.2 và các package `@codemirror/*` khác) cho trình xem diff
- `react-resizable-panels` 4.13.1 (bố cục ba vùng kéo được)
- Plugin Tauri đã dùng: `tauri-plugin-dialog` 2.7.3, `tauri-plugin-store` 2.4.5

**Backend (Rust, `src-tauri/`)**
- Gọi `git` CLI qua `tokio::process` — **không** dùng libgit2/gitoxide
- `serde` / `serde_json`, `thiserror` 2.0.20, `bstr` 1.13.1, `memchr` 2.8.3, `lru` 0.18.4,
  `parking_lot` 0.12.5, `tracing` 0.1.44, `md-5` 0.10.6, `notify` 8.2.0 +
  `notify-debouncer-full` 0.6.0 (file watcher, dùng từ Phase 4)
- Test/bench: `tempfile` 3.27.0, `insta` 1.48.0 (snapshot), `criterion` 0.8.2 (benchmark)
- Rust edition 2021, `rust-version = "1.82"`

## Yêu cầu môi trường

- **Node.js** `^20.19` hoặc `>=22.12` (Vite 8 yêu cầu)
- **Rust** `stable` (theo `rust-toolchain.toml`), kèm component `rustfmt` và `clippy`.
  Trên **Windows** phải dùng target **MSVC** (không dùng GNU) — cần Visual Studio Build Tools
  2022 với workload "Desktop development with C++", cộng rustup.
- **Git CLI** bản gần đây (ứng dụng chỉ gọi `git` CLI, không dùng libgit2/gitoxide)
- **Windows**: cần WebView2 Runtime (thường có sẵn trên Windows 10/11 đã cập nhật)
- **Linux**: cần `libwebkit2gtk-4.1-dev` (KHÔNG phải 4.0 — Tauri v2 yêu cầu 4.1), cộng
  `libappindicator3-dev`, `librsvg2-dev`, `patchelf`, `libssl-dev`, `build-essential`.
  Khuyến nghị Ubuntu 22.04 trở lên.
- **macOS**: chưa được kiểm thử/phát triển chính thức (Windows là máy phát triển chính), nhưng
  CI có job build trên `macos-latest`.

## Cách chạy

**Cài đặt**
```bash
npm install
```
Cargo sẽ tự tải dependency Rust khi build lần đầu.

**Chạy dev**
```bash
npm run tauri:dev
```
Trên Windows, có thể chạy `dev.cmd` ở gốc repo — script này tự thêm
`%USERPROFILE%\.cargo\bin` vào `PATH` và dọn tiến trình mồ côi trước khi gọi
`npm run tauri:dev`.

**Build**
```bash
npm run build        # chỉ build frontend (kiểm type + đóng gói Vite)
npm run tauri:build  # đóng gói ứng dụng desktop đầy đủ
```

**Test frontend**
```bash
npm test          # chạy một lần
npm run test:watch # chạy theo dõi thay đổi
```

**Test Rust** (trong `src-tauri/`)
```bash
cd src-tauri
cargo test
cargo test --all-features   # như CI
```

**Kiểm khác**
```bash
npm run typecheck
npm run check:chunks   # kiểm bundle size lazy-load CodeMirror lang

# trong src-tauri/
cargo clippy --all-targets -- -D warnings
cargo fmt --all --check
cargo bench --bench graph   # benchmark thuật toán gán lane (không nêu số kết quả cụ thể)
```

**Sinh fixture** (repo git mẫu dùng cho test)
```bash
npm run fixtures
```
Cần Git Bash trên Windows.

## Cấu trúc thư mục

- `src/` — React + TypeScript: `components/`, `stores/` (Zustand), `lib/`, `styles/`, `test/`,
  `assets/`, cộng `App.tsx`, `main.tsx`
- `src-tauri/` — Rust backend: `src/`, `benches/` (criterion), `tests/`, `capabilities/`
  (capability file Tauri v2), `icons/`, `Cargo.toml`, `tauri.conf.json`
- `docs/` — tài liệu nghiên cứu và quyết định kỹ thuật (xem mục "Tài liệu tham khảo" dưới đây)
- `scripts/` — `fixtures/` (sinh repo git mẫu cho test), `check-lang-chunks.mjs` (cổng kiểm
  bundle size cho lazy-load CodeMirror lang), `verify-work04-bytes.sh`
- `.planning/` — tài liệu quản lý dự án theo phương pháp GSD (PROJECT.md, ROADMAP.md,
  STATE.md, các phase con) — công cụ nội bộ của chủ dự án, không phải tài liệu cho người dùng
  cuối

File cấu hình gốc đáng chú ý: `dev.cmd` (script khởi chạy dev trên Windows, tự dọn tiến trình
mồ côi), `rust-toolchain.toml`, `tsconfig.json`, `vite.config.ts`.

## Tài liệu tham khảo

Các tài liệu nghiên cứu và quyết định kỹ thuật trong `docs/`:

- `01-research-competitors.md` — nghiên cứu đối thủ cạnh tranh
- `02-phase4-commit-notes.md` — ghi chú quyết định kỹ thuật cho Phase 4 (vòng lặp commit)
- `03-phase1-qa-windows.md` — kịch bản QA thủ công trên Windows cho Phase 1
- `04-phase2-degraded-graph.md` — ghi chú về đồ thị suy giảm (degraded graph) Phase 2
- `05-phase2-performance.md` — số đo hiệu năng Phase 2
- `06-graph-render-model.md` — mô hình dựng hình đồ thị commit
- `07-ui-reference-gap.md` — ghi chú khoảng cách so với tham chiếu giao diện
- `08-phase3-dogfood.md` — khung ghi chép dogfood (dùng thật) cho Phase 3
- `09-phase3-diff-decision.md` — quyết định kỹ thuật cho trình xem diff Phase 3
- `10-phase5-dogfood.md` — khung ghi chép dogfood cho Phase 5
- `10-research-git-gui-landscape.md` — nghiên cứu bối cảnh các công cụ GUI Git hiện có
- `BRAND.md` — tài liệu định vị thương hiệu
- `screenshots/` — ảnh chụp màn hình minh hoạ

## Giấy phép

MIT, theo khai báo `"license": "MIT"` trong `package.json`. Repo hiện **chưa có** tệp
`LICENSE` chính thức — tệp này sẽ được thêm ở Phase 8 (Phát hành, REL-03) theo lộ trình.

## Đóng góp / liên hệ

Dự án đang ở giai đoạn phát triển nội bộ, quản lý theo phương pháp GSD (Get Shit Done). Xem
thư mục `.planning/` để hiểu quy trình lập kế hoạch và thực thi hiện tại của dự án.
