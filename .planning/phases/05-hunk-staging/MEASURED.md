# Phase 5 — Số đo thật lúc lập kế hoạch

**Ngày:** 2026-09-22 · **git:** 2.54.0.windows.1 · đo trong repo tạm ở thư mục scratchpad

Tệp này giữ lại **phép đo**, không phải suy luận. Mỗi mục dưới đây đã đổi một quyết định
trong plan. Executor không cần đo lại, nhưng nếu đo lại thì phải ra cùng kết quả.

---

## 1. 🔴 Fixture "chọn hunk thứ hai trong ba hunk" KHÔNG phân biệt được `--recount`

CONTEXT.md mục 2.2 nói: *"Chọn khối thứ hai trong một tệp có ba khối. Chỉ chọn khối đầu
thì header tình cờ vẫn đúng và đột biến bỏ `--recount` không đỏ."*

**Đã đo — chọn khối thứ hai cũng không đỏ.**

Repo: 26 dòng (`a`..`z`), sửa dòng 2, 13, 25 → đúng 3 hunk ở `-U3`. Lấy hunk 2, chép
header **nguyên văn**:

```
@@ -10,7 +10,7 @@ i
 j
 k
 l
-m
+M2
 n
 o
 p
```

```
$ git apply --check --cached --recount h2.patch   → exit 0
$ git apply --check --cached           h2.patch   → exit 0      ← KHÔNG ĐỎ
```

**Vì sao:** số đếm trong `@@` là **của riêng hunk đó**, không tích luỹ qua cả tệp. Chép
nguyên văn header của hunk 2 thì nó **vẫn đúng**, bất kể bỏ bao nhiêu hunk khác.

Đây đúng lớp lỗi "hình dạng đúng, dữ liệu vô hại" (CONTEXT.md 4.2) — fixture trông như
đang kiểm `--recount` nhưng không kiểm gì.

## 2. ✅ Fixture phân biệt được: cắt thân hunk, giữ nguyên số đếm

Cái làm header **thật sự sai** là **cắt bớt dòng thân** trong khi giữ số đếm cũ — đúng
việc mà một bộ dựng bản vá con làm khi nó tỉa ngữ cảnh:

```
@@ -10,7 +10,7 @@ i      ← nói 7 dòng
 j
 k
 l
-m
+M2
 n                       ← chỉ còn 5 dòng thân
```

```
$ git apply --check --cached --recount h2_trim.patch  → exit 0
$ git apply --check --cached           h2_trim.patch  → exit 128
                                  error: corrupt patch at h2_trim.patch:12
```

**Đây là fixture được CHỨNG MINH phân biệt được, dùng ở plan 05-02 Task 2 Test B.**

## 3. ✅ Staging một phần hoạt động đúng trên git thật

```
$ git apply --cached --recount h2.patch
$ git diff --cached -- f.txt | grep -c '^@@'   → 1     (đúng hunk giữa)
$ git diff          -- f.txt | grep -c '^@@'   → 2     (hai hunk kia còn ở worktree)
```

## 4. ✅ WORK-04 — CRLF + thiếu dòng cuối + byte 0xE9 giữ nguyên

Repo `core.autocrlf=true`, tệp 30 dòng CRLF, **không** dòng cuối, `0xE9` ở dòng 6, ba
hunk. Sau khi stage hunk giữa:

| Phép đo | Trước | Sau |
|---|---|---|
| số `\r` trong worktree | 29 | **29** |
| byte cuối worktree | `9` (không phải `\n`) | **`9`** |
| `0xE9` trong worktree | có | **có** |
| `0xE9` trong blob đã stage | — | **có** |

Lưu ý: `git diff` với `autocrlf=true` in bản vá bằng **LF** (clean filter đã bỏ `\r`), và
blob trong index cũng là LF. Đó là **đúng** — `autocrlf=true` nghĩa là vậy. Điều WORK-04
bảo vệ là **tệp trong thư mục làm việc**, và nó không đổi.

## 5. 🔴 `git stash create` trả RỖNG khi chỉ có tệp chưa theo dõi

```
$ printf 'untracked content\n' > u.txt      # tệp DUY NHẤT thay đổi
$ git stash create "t2"
[]                                           ← chuỗi rỗng, exit 0
```

Nó **im lặng không lưu gì**. Một cài đặt tin `stash create` luôn trả sha sẽ làm người
dùng mất tệp chưa theo dõi **không thông báo**. Đây đúng ca mà ROADMAP bắt dùng động từ
**"Xoá"** — tức ca này chắc chắn xảy ra.

Đường thay thế đã đo chạy được:

```
$ B=$(git hash-object -w -- u.txt)   → 3f4c56d3d8d73bb43575b07d84c82e30210900d1
$ git cat-file -p $B                 → untracked content
```

## 6. ✅ `git update-ref` chấp nhận trỏ thẳng vào một BLOB

```
$ git update-ref refs/git-plum-trash/blobtest 3f4c56d3...   → exit 0
$ git for-each-ref refs/git-plum-trash/
96b5c6f6... commit  refs/git-plum-trash/1790087324
3f4c56d3... blob    refs/git-plum-trash/blobtest
```

Nên **cả hai** dạng object neo được dưới cùng một không gian ref. Không cần dựng commit
giả cho tệp chưa theo dõi.

## 7. ✅ Vòng huỷ → khôi phục đầy đủ

```
$ printf 'one\nCHANGED\nthree\n' > a.txt
$ SHA=$(git stash create "trash")                  # 96b5c6f6...
$ git update-ref refs/git-plum-trash/1790087324 $SHA
$ git checkout -- a.txt                            # huỷ
$ cat a.txt                                        → one/two/three   (mất CHANGED)
$ git show refs/git-plum-trash/1790087324:a.txt    → one/CHANGED/three  ✅
```

---

## Bẫy công cụ gặp lại trong lúc đo

- `rtk` chặn `git diff > file` — tệp không được tạo, exit 1. Dùng `rtk proxy git ...`
  cho mọi lệnh cần **byte thô** của git.
- `rtk` làm hỏng mẫu `grep` có `\|` → `0 matches` cho mẫu thật sự khớp. Dùng công cụ
  Grep khi kết quả quan trọng. **Đã gặp lại hai lần hôm nay.**
- `rtk` lọc cả đầu ra `git stash create` (in `ok stash create` thay vì sha).
- **Không có `python`** trên máy này. Dùng `node -e` cho thao tác byte
  (`Buffer.from(s,"binary")`).
