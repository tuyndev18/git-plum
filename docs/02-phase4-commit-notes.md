# Ghi chú bàn giao cho Phase 4 — `--cleanup=whitespace`

Tệp này tồn tại để bàn giao một ràng buộc của **PLAT-02** mà Phase 1 không thi hành được:
`--cleanup=whitespace` là tham số dòng lệnh của `git commit`, mà Phase 1 chưa có lệnh commit
nào để gắn nó vào. Ghi lại ở đây để Phase 4 không phải khám phá lại ràng buộc này từ đầu.

---

## Ràng buộc

**Mọi lệnh `git commit` do git-plum sinh ra phải mang `--cleanup=whitespace`.**

Lý do: mặc định của git là `--cleanup=default`, và chế độ đó **xoá mọi dòng bắt đầu bằng `#`**
khỏi thông điệp commit. Người dùng gõ một dòng mở đầu bằng `#` — ví dụ tham chiếu issue
`#123` đặt ở đầu dòng — sẽ thấy dòng đó biến mất, không một lời giải thích. Với người dùng,
đây là ứng dụng lặng lẽ nuốt mất nội dung họ vừa gõ; họ không có cách nào đoán ra nguyên nhân.

Chế độ `whitespace` chỉ cắt khoảng trắng thừa ở cuối dòng và các dòng trống thừa ở cuối
thông điệp, **giữ nguyên các dòng bắt đầu bằng `#`**.

Ràng buộc này áp cho cả commit thường, `--amend`, và mọi đường đi khác có sinh ra
thông điệp commit.

---

## Nơi thi hành

Ở **chỗ dựng tham số lệnh commit trong Phase 4** — không phải `apply_env_hardening` trong
`src-tauri/src/git/exec.rs`.

Lý do tách bạch: `apply_env_hardening` chỉ ghim được **biến môi trường** và các khoá cấu hình
đi qua `GIT_CONFIG_PARAMETERS`. `--cleanup` không có khoá cấu hình tương đương nào mà git đọc
cho mọi lệnh commit, nên không có chỗ đặt nó trong lớp ghim môi trường. Nó bắt buộc phải là
một tham số dòng lệnh, do đó thuộc về nơi dựng lệnh.

Trong `src-tauri/src/git/exec.rs` đã có một bình luận `// TODO(Phase 4):` ngay dưới khối
`GIT_CONFIG_PARAMETERS` trỏ về tệp này, để người đọc `exec.rs` đi tìm `--cleanup` không kết
luận nhầm là nó bị bỏ quên.

---

## Cách kiểm chứng ở Phase 4

1. Tạo một commit qua git-plum với thông điệp có **một dòng bắt đầu bằng `#`**, ví dụ:

   ```
   Sửa lỗi phân tích đầu ra status

   #123 dòng này phải còn nguyên sau khi commit
   ```

2. Chạy:

   ```
   git log -1 --format=%B
   ```

3. Xác nhận dòng bắt đầu bằng `#` vẫn còn nguyên trong đầu ra. Nếu nó biến mất thì
   `--cleanup=whitespace` chưa được truyền vào lệnh commit.

Nên biến bước trên thành một test tự động ở Phase 4, không chỉ kiểm thủ công: đây là loại lỗi
âm thầm, không có thông báo nào, nên rất dễ hồi quy mà không ai nhận ra.

---

## Tham chiếu ngược

- `.planning/ROADMAP.md` — mục Phase 1, khối "Ràng buộc bắt buộc"
- `.planning/phases/01-platform-git-layer/CONTEXT.md` — mục G1
- `src-tauri/src/git/exec.rs` — hằng `PINNED_GIT_CONFIG` và bình luận `TODO(Phase 4)`
