---
phase: 01-platform-git-layer
plan: 03
subsystem: frontend
tags: [tauri-plugin-store, zustand, react, vitest, plat-07, persistence]

# Dependency graph
requires:
  - phase: 01-platform-git-layer
    provides: "`src/stores/repoStore.ts` (PLAT-05) và `src/lib/ipc.ts` — điểm móc để ghi nhớ repository sau khi mở"
  - phase: 01-platform-git-layer
    provides: "Hạ tầng kiểm thử vitest từ plan 01-02 — khối `test` trong `vite.config.ts`, khuôn mẫu giả lập ở ranh giới module"
provides:
  - "`src/lib/recentRepos.ts` — đọc ghi danh sách repository gần đây qua `tauri-plugin-store`, kèm logic thuần tuý tách riêng"
  - "`normalizeRepoPath` — phép chuẩn hoá đường dẫn phía giao diện khớp chính xác `state::repo_id_for` bên Rust"
  - "Trường `recent` và hành động `loadRecent`/`setRecent` trong `repoStore`"
  - "`src/components/RecentRepoList.tsx` — danh sách gần đây ở màn hình trống, mở bằng một lần bấm"
  - "Lần dùng thật đầu tiên của `tauri-plugin-store` trong dự án — chứng minh quyền `store:default` đủ dùng"
affects: [Phase 2 khi thêm thao tác có tham số vào sổ đăng ký lệnh, bảng lệnh gõ nhanh v2]

# Tech tracking
tech-stack:
  added: []  # @tauri-apps/plugin-store 2.4.5 đã có trong package.json từ lần scaffold, plan này là chỗ dùng đầu tiên
  patterns:
    - "Tách module thành phần thuần tuý (kiểm thử trực tiếp) và vỏ mỏng đụng Tauri (chỉ test điều kiện không-bao-giờ-ném-lỗi)"
    - "Chuẩn hoá đường dẫn phía giao diện phải sao chép chính xác phía Rust, kể cả những gì Rust **không** làm"
    - "Tiện ích không được ném lỗi vào luồng thao tác chính — bọc hai lớp khi lớp trong nằm ở tệp khác"
    - "Mảng rỗng trả về từ một hàm nuốt lỗi vừa nghĩa là 'thành công và rỗng' vừa nghĩa là 'thất bại' — phải phân biệt trước khi ghi đè dữ liệu người dùng nhìn thấy"

key-files:
  created:
    - src/lib/recentRepos.ts
    - src/lib/recentRepos.test.ts
    - src/components/RecentRepoList.tsx
    - src/components/RecentRepoList.test.tsx
  modified:
    - src/stores/repoStore.ts
    - src/stores/repoStore.test.ts
    - src/App.tsx
    - src/styles/app.css

key-decisions:
  - "`normalizeRepoPath` cố tình **không** đổi chữ thường, dù Windows không phân biệt hoa thường — vì `state::repo_id_for` bên Rust cũng không"
  - "Hai thao tác có tham số (`onOpen`, `onForget`) truyền bằng prop thay vì qua sổ đăng ký PLAT-04; không đổi chữ ký `Command`"
  - "`rememberRepo` được bọc `try/catch` hai lớp có chủ ý: lớp trong ở `recentRepos.ts`, lớp ngoài ở `repoStore.ts`"
  - "Mảng rỗng từ `rememberRepo`/`forgetRepo` không ghi đè danh sách đang hiển thị"
  - "Không thêm `@testing-library/jest-dom` chỉ để dùng một matcher — khẳng định bằng `container.innerHTML`"

requirements-completed: [PLAT-07]

# Metrics
duration: 14min
completed: 2026-09-21
---

# Phase 1 Plan 03: Danh sách repository gần đây Summary

**Đóng khoảng cách G2 bằng cách biến `tauri-plugin-store` — thứ đã khai báo ở ba chỗ nhưng chưa dùng ở đâu — thành một danh sách repository gần đây mở được bằng một lần bấm, với phép khử trùng lặp sao chép chính xác cách backend sinh `RepoId`.**

## Performance

- **Duration:** 14 phút
- **Tasks:** 3/3
- **Files:** 8 (4 tạo mới, 4 sửa)
- **Test:** 23 → 57 (thêm 34)

## Accomplishments

- **PLAT-07 từ không có mã nào thành một lát cắt dọc chạy được.** Trước plan này màn hình trống là ngõ cụt: đường duy nhất quay lại repository hôm qua là hộp thoại chọn thư mục, mỗi lần khởi động. Giờ `openRepository` tự ghi nhớ, `App.tsx` nạp danh sách lúc gắn kết, và mỗi mục mở ra bằng một lần bấm.
- **Phép khử trùng lặp không bất đồng với backend.** `normalizeRepoPath` sao chép đúng hai việc `state::repo_id_for` làm (`replace('\\', "/")` rồi `trim_end_matches('/')`) và cố tình dừng ở đó. Chi tiết ở mục "Decisions Made" — đây là quyết định đáng chú ý nhất của plan.
- **34 test mới, trong đó 21 nhắm vào logic thuần tuý** không cần giả lập gì. Plan yêu cầu tối thiểu 7 cho Task 1.
- **Xác minh `store:default` thật sự đủ**, không đoán. Đọc `src-tauri/gen/schemas/acl-manifests.json` bằng `node` và đối chiếu: bộ quyền mặc định của plugin store gồm `allow-load`, `allow-get`, `allow-set`, `allow-save` cùng 10 quyền khác; module chỉ dùng `load`, `get`, `set`. **`src-tauri/capabilities/default.json` không bị sửa một dòng nào**, đúng như plan yêu cầu.
- **Không đụng một tệp Rust nào.** `cargo test` vẫn 23 đỗ, không thêm không bớt.

## Task Commits

1. **Task 1: Module lưu trữ danh sách repository gần đây** — `cfac15e` (feat)
2. **Task 2: Ghi nhớ repository sau mỗi lần mở thành công** — `29825cf` (feat)
3. **Task 3: Hiển thị danh sách gần đây và mở bằng một lần bấm** — `ab8f353` (feat)

## Verification

Toàn bộ chạy thật, không suy ra từ đọc mã:

```
npm test                          5 tệp, 57 đỗ   (nền: 3 tệp, 23 đỗ)
npm run typecheck                 sạch
npm run build                     273.83 kB │ gzip: 85.96 kB
cargo test                        23 đỗ, 3 suite (không đổi)
npx tauri build --debug --no-bundle   git-plum.exe dựng xong
```

`npx tauri build` là phép kiểm quan trọng ở đây chứ không phải thủ tục: nó là thứ duy nhất xác nhận npm `@tauri-apps/plugin-store` 2.4.5 và crate Rust cùng tên vẫn khớp theo minor. Lệch minor thì Tauri **từ chối chạy**, và plan này là lần đầu tiên plugin đó thật sự được nhập vào mã.

## Decisions Made

**`normalizeRepoPath` không đổi chữ thường — và đó là điểm mấu chốt.** Trực giác nói rằng trên Windows `C:/Kho/A` và `c:/kho/a` là một repository, nên nên gộp lại. Nhưng `state::repo_id_for` trong `src-tauri/src/state/mod.rs` chỉ làm hai việc: đổi `\` thành `/` và cắt `/` ở cuối. Nó **không** đổi chữ thường. Nếu phía giao diện "thông minh hơn" một chút, hai mục mà danh sách gần đây coi là một lại sinh ra hai `RepoId` khác nhau ở backend — `byRepo` có hai khoá trong khi danh sách chỉ có một dòng. Đây đúng là loại bất đồng không ai tìm ra bằng cách đọc mã, vì mỗi nửa đọc riêng đều hợp lý. Nên hàm này sao chép phía Rust chính xác, kể cả phần Rust không làm, và bình luận trong mã nói rõ vì sao. Có test đối chiếu trực tiếp (`normalizeRepoPath` describe block).

**Hai thao tác có tham số đi ngoài sổ đăng ký PLAT-04.** Plan đã chốt sẵn hướng này và tôi theo, nhưng đây là chỗ duy nhất trong dự án quy ước PLAT-04 bị phá nên viết rõ lý do tại chính chỗ gọi: `Command.run` có chữ ký `() => void | Promise<void>`, không nhận tham số, còn "mở *repository này*" thì cần biết đường dẫn nào. Hai đường nhồi nó vào sổ đăng ký hiện tại đều tệ hơn — đăng ký một lệnh cho mỗi mục (sổ đăng ký thành dữ liệu động, mã định danh hết ổn định, trái hẳn ý định PLAT-04), hoặc nhét đường dẫn vào biến toàn cục rồi để lệnh đọc ra (một tham số trá hình, khó lần ra hơn hẳn truyền thẳng). `repo.open` và `repo.close` vẫn đi qua sổ đăng ký nguyên vẹn.

**Bọc `try/catch` hai lớp quanh `rememberRepo`.** `recentRepos.ts` đã tự nuốt lỗi rồi, nên lớp ngoài ở `repoStore.ts` nhìn như thừa. Giữ vì lớp trong nằm ở tệp khác và có thể bị sửa mất trong một lần dọn dẹp tương lai, còn hậu quả thì không tương xứng: người dùng vừa mở repository **thành công**, và một banner lỗi sau đó là bug tệ hơn chính lỗi mà nó báo. Test `hoàn tất việc mở repository bình thường khi rememberRepo ném lỗi` khoá ràng buộc này lại — gỡ lớp ngoài ra, test đỏ ngay (đã kiểm chứng, xem dưới).

**Mảng rỗng không được ghi đè danh sách đang hiển thị.** Đây là thứ plan không nêu và tôi phát hiện khi viết. `rememberRepo` và `forgetRepo` nuốt lỗi rồi trả `[]`, nên `[]` mang hai nghĩa: "thành công, danh sách rỗng" và "ghi thất bại". Nếu cứ `set({ recent })` thì một lần ghi hỏng xoá sạch danh sách người dùng đang nhìn — mất dữ liệu nhìn thấy được vì một lỗi mà thiết kế đã chủ động coi là không quan trọng. Xử lý: `openRepository` chỉ cập nhật khi `length > 0`; `handleForgetRecent` gặp `[]` thì nạp lại từ đĩa để phân biệt hai trường hợp (xoá thật thì đọc lại vẫn rỗng, ghi hỏng thì danh sách cũ hiện lại). Có test cho nhánh đầu.

**Kiểm chứng đột biến cho cả ba ràng buộc quan trọng**, theo khuôn mẫu plan 01-02 đặt ra. Test chỉ mô tả mã hiện tại thì vô dụng; phải chứng minh nó khoá được thứ gì:

| Đột biến | Kết quả |
|---|---|
| Bỏ `.slice(0, MAX_RECENT)` trong `mergeRecent` | 1 đỏ / 20 đỗ |
| Bỏ phép lọc trùng `path` trong `mergeRecent` | 2 đỏ / 19 đỗ |
| Bỏ `try/catch` lớp ngoài quanh `rememberRepo` | 1 đỏ / 15 đỗ |

Cả ba đã hoàn nguyên; `git status` sạch trước mỗi commit.

## Deviations from Plan

Không có sai lệch nào cần áp dụng Rule 1–4. Kế hoạch chạy đúng như viết. Bốn điều chỉnh nhỏ, tất cả nằm trong khoảng tự do plan cho phép:

**1. Thêm `normalizeRepoPath` và `setRecent` ngoài danh sách export plan liệt kê.** Plan nêu `loadRecentRepos`, `rememberRepo`, `forgetRepo`, `MAX_RECENT`, `mergeRecent`, `sanitizeRecent`. `normalizeRepoPath` xuất thêm để test được đối chiếu trực tiếp với `repo_id_for` — phần "Design notes" của brief nêu đúng mối lo này ("dedupe logic should not disagree with it"), và khoá nó bằng test đòi hỏi hàm phải gọi được. `setRecent` cần cho `handleForgetRecent` ở `App.tsx`.

**2. `sanitizeRecent` khử trùng lặp, không chỉ lọc phần tử hỏng.** Plan mô tả `sanitizeRecent` là lọc hình dạng + cắt `MAX_RECENT`. Thêm khử trùng lặp vì component dùng `item.path` làm `key` của React: một tệp bị sửa tay chứa hai mục cùng khoá sẽ sinh cảnh báo key trùng. Đây là Rule 2 ở mức nhẹ — hình dạng dữ liệu hợp lệ phải bao gồm "không có khoá trùng" khi khoá đó được dùng làm `key`.

**3. Nút "Xoá khỏi danh sách" nằm **cạnh** nút mở, không lồng trong nó.** Plan viết "Mỗi mục có một nút phụ ... Nút này phải `e.stopPropagation()`". HTML không cho lồng `<button>` trong `<button>`, nên hai nút là anh em trong một `<li>` dùng flexbox. `stopPropagation()` vẫn giữ theo đúng yêu cầu plan, và có test chứng minh bấm xoá không kích hoạt mở — nếu sau này ai bọc cả dòng vào một vùng bấm được thì nó đã sẵn đúng.

**4. Thêm `src/components/RecentRepoList.test.tsx` (7 test), ngoài `files_modified` của plan.** Plan liệt kê test cho Task 1 và Task 2 nhưng không cho Task 3, trong khi `must_haves.truths` lại đòi hành vi kiểm chứng được cho danh sách rỗng và việc mở bằng một lần bấm. Brief cũng nói rõ "make it a tested behaviour rather than a crash". Đây là tệp duy nhất trong plan thật sự render, nên dùng `@testing-library/react`; các test store khác vẫn gọi `getState()` theo khuôn mẫu 01-02.

## Issues Encountered

**`toBeEmptyDOMElement` không tồn tại.** Matcher đó thuộc `@testing-library/jest-dom`, gói chưa có trong dự án (`package.json` chỉ có `@testing-library/react`). Vitest báo `Invalid Chai property`. Đổi sang `expect(container.innerHTML).toBe('')` thay vì cài thêm một phụ thuộc chỉ để đọc cho đẹp hơn một dòng.

**`vi.clearAllMocks()` xoá cả giá trị trả về đặt trong factory của `vi.mock`.** `beforeEach` sẵn có trong `repoStore.test.ts` gọi `clearAllMocks()`, nên `rememberRepo` khai báo `.mockResolvedValue([])` trong factory trở thành trả `undefined` ở mọi test, và `openRepository` vỡ ở chỗ đọc `.length`. Phải dựng lại mặc định trong `beforeEach`. Ghi lại ở đây vì mọi plan sau dùng khuôn mẫu giả lập này sẽ gặp đúng bẫy.

**Phần đệm store ở phạm vi module làm test khó cô lập.** Đúng như plan dự đoán. Giải bằng `__resetStoreForTests()` như plan gợi ý, có ghi chú rõ mục đích trong mã. Đệm chính `Promise<Store>` chứ không đệm kết quả, để hai lời gọi song song lúc khởi động (`loadRecent` của `App` và `rememberRepo` của lần mở đầu) không mở tệp hai lần — có test `chỉ mở tệp store một lần dù gọi nhiều lần`.

**`gsd-sdk query state.record-metric` ghi sai chỗ trong `STATE.md`.** Đúng như brief cảnh báo về định dạng tiếng Việt viết tay: dòng metric bị chèn **sau** dấu `---` kết thúc mục, tức nằm ngoài bảng. Đã sửa tay đưa vào đúng bảng. `roadmap.update-plan-progress` và `requirements.mark-complete` chạy đúng.

## Điều đáng chú ý cho người đọc sau

**Kiểm chứng thật của PLAT-07 vẫn chưa xảy ra.** Mọi thứ trong plan này chạy dưới vitest với `@tauri-apps/plugin-store` bị giả lập. Câu hỏi thật — "đóng ứng dụng, mở lại, repository còn trong danh sách không" — đòi chạy bản dựng thật và không test tự động nào trả lời được. Plan 01-04 giữ phần đó. Cụ thể còn chưa biết: `recent-repos.json` rơi vào đúng thư mục dữ liệu ứng dụng chưa, và `autoSave: true` có thật sự ghi xuống đĩa trước khi tiến trình thoát không. Nếu 01-04 thấy danh sách rỗng sau khi khởi động lại, hai chỗ đó là nơi cần nhìn trước — và theo cảnh báo T-01-13, kiểm `capabilities/default.json` trước cả mã frontend, vì Tauri v2 hỏng **im lặng ở thời điểm chạy** khi thiếu quyền.

**Repository không còn tồn tại đã có đường xử lý nhưng cũng chưa kiểm bằng tay.** `handleOpenRecent` bắt lỗi và hiện qua `describeError` đúng như nhánh `repo.open`, và **không** tự xoá mục khỏi danh sách — ổ mạng hôm nay không vào được mai lại vào được, người dùng tự quyết. Đường mã có, nhưng xác nhận thông báo đọc hiểu được thì thuộc phần QA thủ công của 01-04.

**Bản dựng tăng 5.8 kB JS và 5.69 kB CSS** (268.01 → 273.83 kB). Tương xứng với một module, một component và một khối CSS; không có phụ thuộc mới nào lọt vào.

## User Setup Required

Không có. `@tauri-apps/plugin-store` đã nằm trong `dependencies` và đã cài; plugin đã khởi tạo trong `src-tauri/src/lib.rs`; quyền `store:default` đã cấp từ trước.

## Next Phase Readiness

Sẵn sàng cho 01-04. Ba khoảng cách còn lại của Phase 1 — G3 (PLAT-09 kích thước vùng qua các lần chạy), G6 (locale không phải tiếng Anh), G7 (cửa sổ console ở bản release) — đều là việc chạy bản dựng thật, cùng loại với việc kiểm chứng PLAT-07 còn thiếu ở trên. Gom chung một phiên QA thủ công là hợp lý.

Một việc bàn giao cho v2: `src/App.tsx` có bình luận dài ngay trên `handleOpenRecent` giải thích vì sao hai thao tác này đi ngoài sổ đăng ký lệnh. Khi làm bảng lệnh gõ nhanh và mở rộng `Command.run` cho tham số, đó là chỗ đầu tiên cần quay lại.

---
*Phase: 01-platform-git-layer*
*Completed: 2026-09-21*

## Self-Check: PASSED

Đã xác minh trên đĩa: cả 8 tệp trong `key-files` tồn tại, cả 3 commit
(`cfac15e`, `29825cf`, `ab8f353`) có trong `git log`. Không commit nào của plan
này chạm vào `src-tauri/` — commit gần nhất đụng thư mục đó vẫn là `3d3e11e`
của plan 01-01, xác nhận `capabilities/default.json` không bị sửa.

`npm test` 57 đỗ / 5 tệp, `npm run typecheck` sạch, `npm run build` và
`npx tauri build --debug --no-bundle` cùng thoát mã 0, `cargo test` 23 đỗ.
