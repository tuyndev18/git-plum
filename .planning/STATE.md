---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: executing
last_updated: "2026-09-21T12:25:00.000Z"
progress:
  total_phases: 8
  completed_phases: 0
  total_plans: 4
  completed_plans: 2
  percent: 50
---

# Project State: git-plum

**Last updated:** 2026-09-21

---

## Project Reference

**Core Value:** Đọc và hiểu lịch sử của một repository phải tức thì — đồ thị commit mở ra trong dưới một giây và cuộn mượt kể cả trên repo hàng chục nghìn commit.

**Current focus:** Phase 1 — Nền tảng và lớp bọc git

**Mode:** mvp (Vertical MVP) · **Granularity:** standard · **Parallelization:** enabled

---

## Current Position

| | |
|---|---|
| **Phase** | 1 — Nền tảng và lớp bọc git |
| **Plan** | 2/4 hoàn thành (01-01, 01-02) |
| **Status** | Đang thực thi Phase 1 |
| **Progress** | Phase 0/8 hoàn thành · Plan 2/4 của Phase 1 |

```
[........] 0/8 phases
```

---

## Performance Metrics

| Metric | Value |
|---|---|
| Phases completed | 0 / 8 |
| Plans completed | 2 |
| v1 requirements delivered | 3 / 59 |

| Plan | Thời lượng | Tasks | Files |
|---|---|---|---|
| Phase 1 P01 | 18min | 2 tasks | 2 files |
| Phase 1 P02 | 9min | 3 tasks | 6 files |

---

## Accumulated Context

### Quyết định đã chốt

- **Tauri v2 + React + TypeScript + Rust, gọi thẳng `git` CLI.** Không dùng libgit2/gitoxide.
- **Bốn mục "làm ngay hoặc không bao giờ" nằm trọn trong Phase 1**: PLAT-02 (ghim biến môi trường), PLAT-03 (xếp hàng ghi theo repo), PLAT-04 (sổ lệnh trung tâm), PLAT-05 (trạng thái khoá theo repo id).
- **Nhóm WORK tách làm hai phase** (4 và 5). Nghiên cứu nâng độ khó staging theo khối lên HIGH; tách ra để vòng lặp commit cơ bản dùng được sớm.
- **BRANCH-05 nằm cùng phase với merge/rebase** vì dùng chung máy trạng thái đang-dở.
- **AI (Phase 7) là nhánh lá** — không có gì phụ thuộc vào nó, được phép cắt, và là ứng viên chạy song song với Phase 6.
- **REL bị xé đôi**: REL-04 (CI ba nền tảng) vào Phase 1; REL-01/02/03 vào Phase 8.
- **Không cam kết tổng thời gian.** Chuỗi phase là kế hoạch, thời lượng là đầu ra.
- **PLAT-02 (plan 01-01)**: `GIT_CONFIG_PARAMETERS` ghim đủ `log.showSignature`, `diff.noprefix`, `format.coverLetter` qua hằng `PINNED_GIT_CONFIG` trong `src-tauri/src/git/exec.rs`. `diff.external` ghim riêng bằng `GIT_EXTERNAL_DIFF=""` vì biến môi trường thắng cấu hình.
- **`--cleanup=whitespace` bàn giao cho Phase 4** qua `docs/02-phase4-commit-notes.md`: nó là tham số dòng lệnh của `git commit`, không đặt được trong lớp ghim môi trường, và Phase 1 chưa có lệnh commit nào.
- **Hạ tầng kiểm thử giao diện** (plan 01-02): cấu hình vitest sống trong khối `test` của `vite.config.ts`, không tách `vitest.config.ts` riêng — một nguồn sự thật cho `resolve.alias`. Giả lập IPC ở ranh giới module `@/lib/ipc`, không vá `invoke` toàn cục. Test logic store gọi thẳng `useRepoStore.getState()`, không render component.
- Chi tiết đầy đủ các quyết định kỹ thuật: xem `.planning/PROJECT.md` mục Key Decisions và `.planning/research/SUMMARY.md` mục 3.

### Việc cần làm

- [ ] **Phase 1, việc đầu tiên**: dựng toolchain Rust. VS Build Tools 2022 (workload Desktop development with C++) **trước**, rồi rustup stable-msvc. Xác minh bằng `cargo build` chạy thành công. Không có bước này thì không kiểm chứng được gì khác.
- [ ] **Phase 2, chuẩn bị trước khi viết thuật toán lane**: dựng bộ repo mẫu (octopus 4 cha, hai gốc không liên quan, nhánh mồ côi, 20+ lane đồng thời, tên tệp không UTF-8 + thông điệp emoji, bản sao nông).
- [ ] **Phase 2**: sao chép sẵn một repo 50k–100k commit (Linux kernel hoặc Chromium) để đo hiệu năng.
- [ ] **Phase 6, trước khi chốt phạm vi**: chạy spike có giới hạn thời gian cho trình giải quyết xung đột trên CodeMirror 6.
- [ ] **Câu hỏi còn mở, quyết khi tới nơi**: nhiều repo mở theo thẻ (v1 hay v2 — kiến trúc đã sẵn sàng nhờ PLAT-05) · blame ở phase đánh bóng v1 hay v1.x (quyết ở cuối Phase 3, theo lịch thực tế) · giá trị thật của AI (xác thực với người dùng beta trước khi đầu tư quá 3–5 ngày công).

### Vướng mắc

- **Thiếu workload C++ trong Visual Studio.** Tình trạng máy phát triển tính đến 2026-09-21:
  - ✅ rustc 1.98.1, cargo 1.98.1, toolchain `stable-x86_64-pc-windows-msvc` — đã cài đúng
  - ✅ WebView2 153.0.4234.48 — có sẵn theo Windows 11
  - ✅ Visual Studio Community 2022 — đã cài
  - ❌ **Workload "Desktop development with C++" chưa được thêm vào bản VS Community đó**, nên không có `link.exe`. Thiếu nó thì `cargo build` vỡ ở bước liên kết.

  Cách xử lý: thêm workload vào bản VS đã có, không cài Build Tools riêng.
  ```powershell
  & "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vs_installer.exe" modify `
    --installPath "C:\Program Files\Microsoft Visual Studio\2022\Community" `
    --add Microsoft.VisualStudio.Workload.NativeDesktop --includeRecommended --quiet --norestart
  ```
  Xác minh: `cargo build` trong một crate bất kỳ chạy tới đích.

### Cổng dogfood đang chờ

| Phase | Cổng |
|---|---|
| 3 | Bản chỉ-đọc lịch sử+diff được tác giả dùng hằng ngày trên repo thật, gồm cả một repo bên thứ ba lớn và lộn xộn, **trước khi** bắt đầu việc thư mục làm việc |
| 4 | Một commit thật vào repo thật, chỉ bằng git-plum |
| 6 | Một luồng nhánh thật đầu-cuối chỉ bằng git-plum: tạo nhánh, commit, push, giải quyết một xung đột merge thật, hợp nhất xong |

### Validation checkpoint chưa giải quyết

| # | Phase | Câu hỏi |
|---|---|---|
| 7 | 1 | Parser có sống sót với locale hệ thống không phải tiếng Anh không? |
| 1 | 2 | Đồ thị có mở dưới 1s và cuộn 60fps trên repo 50k–100k commit thật không? |
| 2 | 2 | Vẽ đồ thị bằng canvas hay lớp phủ SVG ở mức 100k dòng? (giữ sau interface) |
| 4 | 2 | JSON có chiếm phần lớn profile IPC cho dữ liệu lane không? (ship JSON trước rồi đo) |
| 3 | 3 | `@codemirror/merge` tự tính diff từ hai tài liệu đầy đủ — có đủ nhanh với tệp lớn không? |
| 5 | 6 | Công sức làm trình giải quyết xung đột trên stack CodeMirror 6 — con số 4 giờ của SourceGit không chuyển giao được |
| 6 | 7 | Đường dẫn import của `keyring` v4 (API đã tái cấu trúc mạnh so với v3) |

---

## Session Continuity

**Việc tiếp theo:** thực thi các plan còn lại của Phase 1 (01-03, 01-04). Đã xong: 01-01 (PLAT-02), 01-02.

**Nếu mất ngữ cảnh, đọc theo thứ tự:**

1. `.planning/ROADMAP.md` — cấu trúc 8 phase, tiêu chí thành công, ràng buộc từng phase
2. `.planning/REQUIREMENTS.md` — 59 requirement v1 và bảng truy vết
3. `.planning/PROJECT.md` — Core Value, ràng buộc, bảng Key Decisions (một số quyết định đã bị đảo ngược sau nghiên cứu)
4. `.planning/research/SUMMARY.md` — mục 3 (phiên bản đã chốt), mục 5 (ràng buộc định hình phase), mục 6 (validation checkpoint)

**Lưu ý:** Tài liệu dự án viết bằng tiếng Việt; mã định danh requirement, tên crate/package và tên lệnh git giữ nguyên dạng gốc.

---
*State initialized: 2026-09-21 after roadmap creation*
