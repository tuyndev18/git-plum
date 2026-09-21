# Phase 2: Lịch sử và đồ thị nhánh — Context

**Gathered:** 2026-09-21
**Status:** Ready for planning
**Source:** ROADMAP.md Phase 2 + nghiên cứu đã có + đối chiếu mã nguồn sau Phase 1

---

<domain>
## Phase Boundary

Đây là phase chứa **Core Value** của toàn dự án: *đọc và hiểu lịch sử của một repository
phải tức thì*. Nếu mọi phase khác thành công mà phase này chậm, sản phẩm thất bại.

Cuối phase, người dùng mở một repository và đọc được toàn bộ lịch sử qua đồ thị nhánh nhiều
làn, cuộn mượt ở mức 100k commit, chọn commit thấy chi tiết, tìm được commit theo từ khoá.

**Vẫn chưa có trong phase này:** xem nội dung khác biệt của tệp (Phase 3), thay đổi gì trong
repository (Phase 4 trở đi). Phase 2 thuần đọc.

</domain>

---

<inherited_state>
## Thừa hưởng từ Phase 1 — đọc mã trước khi lập kế hoạch

Phase 1 đã đóng. Những thứ sau **đã có và đang chạy**, không được viết lại:

| Thứ có sẵn | Ở đâu | Dùng thế nào ở Phase 2 |
|---|---|---|
| Lớp sinh tiến trình git | `src-tauri/src/git/exec.rs` — `GitCommand` | **Mọi** lệnh git mới phải đi qua đây. Đã ghim `LC_ALL=C`, `GIT_TERMINAL_PROMPT=0`, stdin đóng, `CREATE_NO_WINDOW`, `log.showSignature=false`. Thêm lệnh mới là thêm `.args([...])`, không tự sinh tiến trình. |
| Chạy có ghi nhật ký + khoá ghi | `src-tauri/src/git/runner.rs` — `GitRunner` | `read()` cho lệnh chỉ đọc (song song được), `write()` cho lệnh thay đổi repo. Phase 2 chỉ dùng `read()`. |
| Trạng thái khoá theo repo | `src-tauri/src/state/mod.rs` — `HashMap<RepoId, RepoHandle>` | Cache commit và refs phải khoá theo `RepoId`, không phải biến toàn cục. |
| Kiểu lỗi có `Serialize` | `src-tauri/src/error.rs` — `GitError` | Command mới trả `Result<T, GitError>`. Thêm variant mới thì nhớ cập nhật `code()` và `command_args()`. |
| Nhật ký lệnh | `src-tauri/src/state/command_log.rs` | Tự động — mọi lệnh qua `GitRunner` đều vào nhật ký. |
| Sổ đăng ký lệnh | `src/lib/commands.ts` | Thao tác mới đăng ký ở đây, gọi qua `runCommand(id)`. **`Command.run` không nhận tham số** — thao tác có tham số thì truyền handler qua prop, xem ghi chú dài trong `src/App.tsx`. |
| Store khoá theo repo | `src/stores/repoStore.ts` — `byRepo[repoId]` | Store mới cho commit/refs theo cùng khuôn. |
| Hạ tầng test | `vite.config.ts` khối `test`, `src/test/setup.ts` | 57 test frontend + 23 test Rust đang xanh. Giả lập IPC ở `vi.mock('@/lib/ipc')`, không vá `invoke` toàn cục. Test logic store gọi `useRepoStore.getState()`. |
| Bố cục ba vùng | `src/components/AppLayout.tsx` | Sidebar (nhánh), giữa (đồ thị + danh sách), chi tiết (commit). Hiện cả ba đều là chỗ trống ghi "Phase 2 sẽ điền". |

**Nợ Phase 1 mang sang, không chặn Phase 2:** ba tiêu chí thành công của Phase 1 chưa chạy QA
thủ công, và G8 (trạng thái bật tắt bảng nhật ký không persist) là defect đã xác nhận chưa
sửa. Xem `.planning/phases/01-platform-git-layer/VERIFICATION.md`.

**Một điểm liên quan trực tiếp tới Phase 2:** tài liệu `react-resizable-panels` v4 cảnh báo
`Group` chứa `Panel` render có điều kiện **phải** truyền `panelIds` cho `useDefaultLayout`,
nếu không bố cục lần đầu sẽ sai. `AppLayout` đúng là trường hợp đó và hiện **không** truyền.
Nếu Phase 2 thêm panel hoặc đổi bố cục, xử lý luôn. Tham số `groupId` cũng đã deprecated
trong v4, thay bằng `id`.

</inherited_state>

---

<decisions>
## Quyết định đã chốt — không bàn lại

### Thuật toán và kiến trúc

- **Rust tính toàn bộ hình học từng dòng.** Trả `GraphRow { commit_id, lane, color,
  passthrough: Vec<Edge>, out_edges: Vec<Edge> }`. Frontend **chỉ vẽ**, không bao giờ tự tính
  liên thông. Đây là điểm nghẽn hiệu năng chính; để trong JavaScript là sai.
- **Một scroll container, một virtualizer.** Cột đồ thị và cột văn bản vẽ từ **cùng một mảng
  `virtualItems`**. Không dùng hai vùng cuộn đồng bộ — `ScrollSync` của react-virtualized có
  lỗi trễ khi cuộn nhanh, tồn tại nhiều năm chưa sửa (issue #369).
- **Bước gán cha trong thuật toán lane phải là vòng lặp qua mọi cha sau cha đầu**, không phải
  trường hợp đặc biệt hai cha. Merge octopus bốn cha phải chạy đúng. Cha thiếu do bản sao nông
  phải kết thúc lane, không làm sập.
- **Bộ vẽ đồ thị nằm sau một interface.** Dựng bản canvas trước, nhưng đổi sang SVG chỉ được
  sửa một tệp. Nghiên cứu để ngỏ có chủ ý.
- **Canvas thuần trình bày.** Bắt sự kiện bấm bằng `div` của dòng, **không** dùng toạ độ
  canvas. Xử lý device pixel ratio, nếu không sẽ mờ trên màn HiDPI.
- **Cách vẽ suy giảm có chủ ý cho ca bệnh lý** — giới hạn số lane hiển thị, hiện chỉ báo
  "+N cha nữa". Phải thiết kế và ghi lại, không để nó tự vỡ.

### Lệnh git dùng ở phase này

```bash
# Lịch sử, phân trang. %x1f = Unit Separator, %x1e = Record Separator —
# hai ký tự này gần như không bao giờ có trong nội dung commit.
git log --all --topo-order \
  --format="%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%b%x1e" \
  --max-count=<n> --skip=<n>

# Nhánh, tag, ahead/behind — một lệnh lấy hết
git for-each-ref \
  --format="%(refname)%x1f%(objectname)%x1f%(upstream)%x1f%(upstream:track)%x1f%(HEAD)"

# Danh sách tệp của một commit. -z vì tên tệp có thể chứa dấu cách/Unicode
git diff --name-status -z --find-renames <parent>..<commit>
```

`--topo-order` là **bắt buộc**: thuật toán lane đòi cha luôn đứng sau con.

### Đầu ra git là BYTE, không phải chuỗi

Tên tệp trên Linux và thông điệp commit mã hoá cũ không bảo đảm UTF-8 hợp lệ. Dùng `bstr` phân tích `Vec<u8>`. Giải mã lossy **chỉ** cho trường hiển thị; trên đường đi của dữ liệu thì không.
`String::from_utf8` sẽ panic hoặc hỏng dữ liệu âm thầm trên repo thật.

### Thư viện đã chốt

| Thứ | Phiên bản | Ghi chú |
|---|---|---|
| `@tanstack/react-virtual` | 3.14.13 | Chọn vì **headless** — vẽ đồ thị và văn bản từ cùng `virtualItems`, nên lệch hàng là bất khả thi về mặt cấu trúc |
| `bstr` | 1.13.1 | Phân tích byte |
| `memchr` | 2.8.3 | Tách theo `\x1f`/`\x1e`/`\0` trên buffer lớn |
| `lru` | 0.18.4 | Cache diff theo SHA commit |

**Không dùng:** `rayon` (gán lane vốn tuần tự), `chrono` (git trả Unix timestamp, dùng
`Intl.DateTimeFormat` bên JS), `similar`/`imara-diff`/`diffy` (tính lại diff đã có sẵn).

### Tiền lệ đáng đọc trước khi viết

`gitlanes` — trình xem commit graph mã nguồn mở **cùng stack Tauri 2 + React**, dùng canvas +
virtual scroll, báo **32k commit nạp trong ~300ms ở 60fps**. Rất gần mục tiêu Core Value.

</decisions>

---

<preparation>
## Chuẩn bị bắt buộc TRƯỚC khi viết thuật toán lane

ROADMAP yêu cầu dựng sẵn bộ repo mẫu. Không có bộ này thì thuật toán lane chỉ được kiểm trên
lịch sử tuyến tính đơn giản, và các ca dưới đây **đều có thật trong repo thật**:

| Repo mẫu | Kiểm điều gì |
|---|---|
| Merge octopus **bốn** cha | Cài đặt chỉ đọc `parents[1]` sẽ bỏ sót, đường nối mất, lane rò rỉ |
| Hai gốc không liên quan, hợp bằng `--allow-unrelated-histories` | Số lane tăng đột biến |
| Nhánh mồ côi (`checkout --orphan`) | Không có cha chung, dễ bị coi là commit gốc giữa lịch sử |
| Hình rẽ nhánh rộng **20+ lane đồng thời** | Giới hạn hiển thị, cách vẽ suy giảm |
| Tên tệp không UTF-8 + thông điệp emoji | HIST-11 — phân tích byte |
| Bản sao nông (`--depth`) | Cha trỏ tới thứ không có trong tập dữ liệu |
| HEAD tách rời | Không nhãn nhánh nào neo vào |

**Repo đo hiệu năng:** chủ dự án đã chọn **sinh repo giả 100k commit** bằng script, không tải
Linux kernel. Script này là một deliverable của phase, phải sinh được hình dạng nhánh thật
(không chỉ một đường thẳng 100k commit — thẳng thì không đo được gì về lane).

</preparation>

---

<validation_checkpoints>
## Ba checkpoint — là tiêu chí thoát phase, không dời về sau

### #1 Hiệu năng repo lớn

Đo thời gian vẽ lần đầu và FPS lúc cuộn trên repo ~100k commit.
**Mốc cần đạt:** `gitlanes` làm được 32k commit trong ~300ms ở 60fps trên đúng stack này.
Gắn benchmark `criterion` cho gán lane và phân tích `git log` ở mức 100k commit **vào CI ngay
trong phase này**, không để sau.

*Nếu trượt:* nâng bộ nạp commit từ command phân trang lên `tauri::ipc::Channel`, để React vẽ
500 dòng đầu trong khi Rust còn đang phân tích phần còn lại.

### #2 Canvas hay lớp phủ SVG ở mức 100k dòng

Dựng canvas trước. **Giữ bộ vẽ sau interface** để đổi là sửa một tệp.

### #4 IPC JSON hay nhị phân cho dữ liệu lane

Ship JSON trước rồi **đo**. Gộp lô lớn — 1000 commit một thông điệp, không phải 1.

*Đính chính một hiểu nhầm phổ biến:* `tauri::ipc::Channel` **vẫn là JSON** — nó cho thứ tự và
chia khối, **không** phải đường nhị phân nhanh. Đường nhị phân thật là `tauri::ipc::Response`
trả mảng byte, hợp với `GraphRow` vì đó là dữ liệu số cố định chiều rộng.

</validation_checkpoints>

---

<pitfalls>
## Cạm bẫy riêng của phase này

Từ `.planning/research/PITFALLS.md`, những mục thuộc Phase 2:

1. **Tên tệp và thông điệp commit không phải UTF-8** — làm hỏng IPC/JSON. Cần repo mẫu trong
   bộ kiểm thử, không chỉ xử lý trong mã.
2. **Thuật toán lane vỡ ở merge octopus và nhánh mồ côi** — chính công cụ của Microsoft từng
   có lỗi tính lệch vị trí khi dựng commit-graph cho merge nhiều cha.
3. **`git log` trên repo lớn sinh hàng trăm MB** — phải đọc theo luồng, không nạp cả
   `Output` vào bộ nhớ. `tokio::process` cho phép việc này.
4. **Commit submodule** — chưa ai kiểm xem nó có làm lệch gán lane không. Thêm một repo mẫu.
5. **Đồ thị lệch hàng khi cuộn nhanh** — nguyên nhân gốc là dùng hai vùng cuộn. Đã chốt
   giải pháp một scroll container.

</pitfalls>

---

<canonical_refs>
## Canonical References

**Đọc trước khi lập kế hoạch hoặc viết mã.**

### Tài liệu kế hoạch
- `.planning/ROADMAP.md` — mục "Phase 2", đặc biệt khối **Ràng buộc** và ba validation checkpoint
- `.planning/REQUIREMENTS.md` — nhóm HIST (11 requirement)
- `.planning/PROJECT.md` — Core Value, bảng Key Decisions
- `.planning/phases/01-platform-git-layer/VERIFICATION.md` — nợ mang sang

### Nghiên cứu
- `docs/01-research-competitors.md` **mục 4** — thuật toán gán lane, cấu trúc `GraphRow`, và
  bảng bảy trường hợp biên bắt buộc có kiểm thử riêng
- `docs/01-research-competitors.md` **mục 5** — các lệnh git và định dạng đầu ra ổn định
- `.planning/research/ARCHITECTURE.md` — ranh giới module, bài toán đồng bộ đồ thị với danh
  sách ảo hoá, bảng cache và invalidation
- `.planning/research/STACK.md` — phiên bản đã chốt, danh sách "không dùng"
- `.planning/research/PITFALLS.md` — mục 1, 4

### Mã nguồn thừa hưởng
Xem bảng ở phần `<inherited_state>`.

### Ảnh tham chiếu
- `docs/screenshots/main-1.png` — bố cục lịch sử của GitKraken: cột BRANCH/TAG, cột GRAPH,
  cột COMMIT MESSAGE, bảng chi tiết bên phải. Học khái niệm và bố cục, **tự thiết kế phần
  nhìn** — không sao chép bảng màu, icon hay kiểu chữ.

</canonical_refs>

---

<success_criteria>
## Sáu tiêu chí thành công — chép từ ROADMAP

1. Cuộn qua lịch sử repository 100k commit, đồ thị hiện ra trong **dưới một giây**, cuộn
   không giật
2. Đồ thị nhiều lane có màu **luôn thẳng hàng tuyệt đối** với từng dòng commit ở mọi vị trí
   cuộn, kể cả khi repo có merge nhiều hơn hai cha, nhánh mồ côi, lịch sử không liên quan,
   bản sao nông hoặc HEAD tách rời
3. Chọn một commit → thấy ngay tiêu đề, nội dung đầy đủ, tác giả, thời gian, mã commit cha và
   danh sách tệp thay đổi; chuyển được giữa dạng đường dẫn phẳng và dạng cây
4. Gõ một từ khoá → tìm được commit theo thông điệp, tên tác giả, mã commit hoặc đường dẫn tệp
5. Nhãn nhánh và tag neo đúng hàng commit, phân biệt local với remote; thanh bên liệt kê nhánh
   local, nhánh remote, tag kèm số đếm
6. Mở repository có tên tệp và thông điệp commit chứa ký tự không phải UTF-8: trang lịch sử
   vẫn hiện đầy đủ, **không có dòng nào bị mất**

Tiêu chí 1 là Core Value. Tiêu chí 2 và 6 cần bộ repo mẫu ở phần `<preparation>`.

</success_criteria>

---

<scope_boundary>
## Ranh giới — không làm gì ở phase này

- **Không** xem nội dung khác biệt của tệp. Danh sách tệp thay đổi thì có (tiêu chí 3), nhưng
  bấm vào tệp để xem diff là Phase 3.
- **Không** thao tác nào làm thay đổi repository. Phase 2 thuần đọc, chỉ dùng `GitRunner::read()`.
- **Không** blame, không file history (Phase 3 hoặc v2).
- **Không** bộ nhớ đệm trên đĩa hay chỉ mục tăng dần — đã ghi Out of Scope trong PROJECT.md.
  Cache trong bộ nhớ theo `RepoId` thì có.
- **Không** sửa G8 của Phase 1 trừ khi việc đổi bố cục ở phase này làm nó thành chặn đường.

</scope_boundary>

---

<open_questions>
## Câu hỏi để ngỏ, quyết khi tới nơi

- **Nhiều repository mở theo thẻ** — v1 hay v2? Kiến trúc đã sẵn sàng nhờ PLAT-05. Quyết khi
  thấy rõ giao diện Phase 2 chịu được bao nhiêu.
- **Số lane tối đa hiển thị** trước khi chuyển sang cách vẽ suy giảm. Cần dữ liệu thật từ repo
  mẫu 20+ lane.

</open_questions>

---
*Context gathered: 2026-09-21*
