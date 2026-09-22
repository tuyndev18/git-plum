//! Băm email tác giả cho avatar Gravatar — **một command, không chạm `Commit`**.
//!
//! # Vì sao là một command riêng chứ không phải một trường trên `Commit`
//!
//! Cách hiển nhiên là thêm `author_avatar_hash` vào [`crate::domain::Commit`] và
//! để `get_commit_page` trả sẵn. Không làm vậy, vì hai lý do đo được:
//!
//! 1. **Chi phí trả cho một tính năng mặc định TẮT.** Hash MD5 dạng hex là 32
//!    ký tự. Trên repo 100k commit, thêm nó vào mỗi bản ghi là ~3,2MB JSON thừa
//!    **mỗi lần nạp** — và Gravatar mặc định tắt, nên gần như mọi người dùng trả
//!    chi phí đó mà không bao giờ dùng tới. `get_commit_page` là đường nóng của
//!    Core Value (đã đo: 749,7ms trên repo 100k); không nhét gì vào đó mà không
//!    có lý do bắt buộc.
//! 2. **`Commit` là kiểu dùng chung.** Phase 4 đang đọc nó song song. Thêm
//!    trường nghĩa là đổi `LOG_FORMAT` và bộ phân tích byte trong
//!    `git/parsers/log.rs` — nơi dự án này **đã** có hai lỗi im lặng
//!    (`%x1f` thoát 0 với stdout trông hợp lệ nhưng thanh bên rỗng; `\0\n` so
//!    với `\0` trong bộ phân tích `--follow`).
//!
//! Command này không cần cả hai: nó băm từ `author_email` mà git **đã** trả, nên
//! `LOG_FORMAT` không đổi một ký tự nào.
//!
//! # Email không rời khỏi Rust
//!
//! Đó là cả điểm của việc băm ở đây. Gravatar định danh bằng MD5 và Web Crypto
//! của trình duyệt không có MD5, nên lựa chọn thay thế là gửi email sang lớp
//! giao diện để băm ở đó. Với ràng buộc Privacy của dự án ("không gửi gì ra
//! ngoài khi chưa được cho phép rõ ràng"), giữ email trong Rust là đường ngắn
//! hơn: lớp giao diện chỉ thấy 32 ký tự hex.

use md5::{Digest, Md5};

/// Băm một email theo đúng quy tắc Gravatar.
///
/// Quy tắc của Gravatar: **cắt khoảng trắng hai đầu, hạ về chữ thường, rồi
/// MD5**. Cả ba bước bắt buộc và theo đúng thứ tự — bỏ bước hạ chữ thường làm
/// `Tuyen@Example.com` và `tuyen@example.com` ra hai hash khác nhau, tức cùng
/// một người hiện hai avatar khác nhau tuỳ commit họ dùng cách viết nào.
///
/// Phép chuẩn hoá này **phải khớp** `avatarTuSinh` ở `src/lib/avatar.ts`, vốn
/// cũng `trim().toLowerCase()` trước khi băm màu. Lệch nhau thì tầng tự sinh và
/// tầng Gravatar bất đồng về "hai email này có phải một người không", và người
/// dùng thấy avatar đổi màu khi bật Gravatar lên.
///
/// MD5 ở đây **không** cho mục đích mật mã — nó chỉ là khoá tra cứu mà Gravatar
/// quy định. Không dùng hàm này cho bất cứ việc gì cần chống va chạm.
fn bam_email(email: &str) -> String {
    let chuan = email.trim().to_lowercase();
    let mut hasher = Md5::new();
    hasher.update(chuan.as_bytes());
    // `{:x}` cho hex chữ thường — Gravatar đòi chữ thường, chữ hoa trả 404.
    format!("{:x}", hasher.finalize())
}

/// MD5 của một email, cho URL Gravatar.
///
/// Giao diện chỉ gọi command này khi người dùng **đã bật** lấy ảnh từ Gravatar,
/// và chỉ cho commit đang chọn — không cho cả trang lịch sử.
///
/// Không trả `Result`: không có cách nào thất bại. Email rỗng vẫn cho một hash
/// hợp lệ (MD5 của chuỗi rỗng), và Gravatar trả 404 cho nó, nên giao diện lui
/// về avatar tự sinh qua đúng đường đã có — không cần một nhánh lỗi riêng cho
/// một ca không phải lỗi.
#[tauri::command]
pub fn avatar_hash(email: String) -> String {
    bam_email(&email)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test vector chuẩn của MD5 — nếu crate hoặc cách gọi sai, cái này đỏ.
    ///
    /// Đáng có dù `md-5` là crate được kiểm nhiều: nó ghim rằng **cách dùng** ở
    /// đây đúng (update rồi finalize, format hex chữ thường), không chỉ rằng
    /// thư viện đúng.
    #[test]
    fn md5_chuoi_rong_khop_test_vector_chuan() {
        assert_eq!(bam_email(""), "d41d8cd98f00b204e9800998ecf8427e");
    }

    #[test]
    fn md5_khop_test_vector_gravatar_chinh_thuc() {
        // Ví dụ trong tài liệu Gravatar.
        assert_eq!(
            bam_email("MyEmailAddress@example.com "),
            "0bc83cb571cd1c50ba6f3e8a78ef1346"
        );
    }

    /// 🔴 Ba bước chuẩn hoá, kiểm từng bước một.
    ///
    /// Thiếu bất kỳ bước nào cũng cho một hash **trông hợp lệ** — 32 ký tự hex,
    /// không ném, không cảnh báo — chỉ là hash của người khác. Biểu hiện duy
    /// nhất là "avatar sai" và không ai báo lỗi đó vì không ai biết avatar đúng
    /// phải là gì.
    #[test]
    fn chuan_hoa_phai_cat_khoang_trang_va_ha_chu_thuong() {
        let chuan = bam_email("tuyen@example.com");
        assert_eq!(bam_email("  tuyen@example.com  "), chuan, "phải cắt khoảng trắng");
        assert_eq!(bam_email("Tuyen@Example.COM"), chuan, "phải hạ về chữ thường");
        assert_eq!(bam_email("  TUYEN@EXAMPLE.COM  "), chuan, "phải làm cả hai");
    }

    #[test]
    fn email_khac_nhau_cho_hash_khac_nhau() {
        assert_ne!(bam_email("a@example.com"), bam_email("b@example.com"));
    }

    /// Hash phải là hex **chữ thường** — Gravatar trả 404 cho chữ hoa.
    #[test]
    fn hash_la_32_ky_tu_hex_chu_thuong() {
        let h = bam_email("tuyen@example.com");
        assert_eq!(h.len(), 32);
        assert!(
            h.chars().all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "hash phải là hex chữ thường, nhận được: {h}"
        );
    }

    /// Email không phải ASCII không được panic.
    ///
    /// `to_lowercase()` của Rust xử lý Unicode đúng, nhưng ca này có thật (tên
    /// miền quốc tế hoá) và một bản viết bằng `make_ascii_lowercase` sẽ im lặng
    /// bỏ qua phần không phải ASCII.
    #[test]
    fn email_unicode_khong_panic() {
        let h = bam_email("Nguyễn@Việt.vn");
        assert_eq!(h.len(), 32);
    }
}
