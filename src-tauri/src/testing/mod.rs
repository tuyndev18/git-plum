//! Trợ giúp tìm repo mẫu cho test và benchmark — HIST-03, HIST-11.
//!
//! Repo mẫu do `scripts/fixtures/make-fixtures.sh` sinh ra, **không** nằm trong git
//! (dựng lại được bằng script nên không có lý do commit chúng). Vì vậy mã ở đây phải
//! xử lý được trường hợp fixture chưa tồn tại.
//!
//! # Vì sao module này `pub` chứ không nằm dưới `#[cfg(test)]`
//!
//! Benchmark của `criterion` là một target riêng (`benches/`), nó **không** thấy mã
//! nằm dưới `#[cfg(test)]`. Checkpoint #1 của phase đòi benchmark gán lane ở mức 100k
//! commit, và benchmark đó cần đúng những hàm này để tìm repo. Nên module là `pub`
//! bình thường.
//!
//! **Chỉ dùng cho test và benchmark.** Mã ứng dụng thật không bao giờ gọi vào đây —
//! đường dẫn repo của người dùng đến từ hộp thoại chọn thư mục, không từ biến môi
//! trường.
//!
//! # Thiếu fixture thì bỏ qua **ồn ào**, không panic
//!
//! Người mới clone repo về chạy `cargo test` phải thấy **xanh kèm lời nhắc**, không
//! thấy một bức tường đỏ vì chưa chạy script sinh fixture. Nhưng cũng không được im
//! lặng — bỏ qua âm thầm thì test coi như không tồn tại mà chẳng ai biết. Vì vậy
//! [`require_fixture`] in ra `eprintln!` nói rõ fixture nào thiếu và lệnh nào cần chạy,
//! rồi trả `None` để test `return` sớm.

use std::path::{Path, PathBuf};

/// Biến môi trường trỏ tới thư mục chứa repo mẫu.
///
/// Không đặt thì dùng `<src-tauri>/../target/fixtures`.
pub const FIXTURES_ENV: &str = "GIT_PLUM_FIXTURES";

/// Lệnh sinh lại bộ repo mẫu. In ra trong thông báo bỏ qua test.
pub const FIXTURES_COMMAND: &str = "bash scripts/fixtures/make-fixtures.sh";

/// Chín repo mẫu mà `make-fixtures.sh` sinh ra.
///
/// Thứ tự giữ giống bảng trong `scripts/fixtures/README.md` để đối chiếu cho dễ.
/// Mỗi tên mang một hình dạng làm vỡ cài đặt lane ngây thơ:
///
/// | Tên | Hình dạng |
/// |---|---|
/// | `linear` | 20 commit một đường thẳng — ca cơ sở |
/// | `octopus` | một merge **bốn** cha |
/// | `unrelated` | hai gốc rời, hợp bằng `--allow-unrelated-histories` |
/// | `orphan` | một nhánh `checkout --orphan`, không merge |
/// | `wide` | 24 nhánh sống đồng thời rồi merge dần |
/// | `non-utf8` | tên tệp byte Latin-1 + thông điệp có `\xff` và emoji |
/// | `shallow` | bản sao nông, cha trỏ ra ngoài tập dữ liệu |
/// | `detached` | HEAD tách rời |
/// | `submodule` | một commit có submodule |
pub const FIXTURE_NAMES: &[&str] = &[
    "linear",
    "octopus",
    "unrelated",
    "orphan",
    "wide",
    "non-utf8",
    "shallow",
    "detached",
    "submodule",
];

/// Thư mục chứa repo mẫu.
///
/// Đọc [`FIXTURES_ENV`]; không có thì suy ra từ `CARGO_MANIFEST_DIR` (thư mục
/// `src-tauri`) cộng `../target/fixtures`.
///
/// Không bảo đảm thư mục trả về **tồn tại** — dùng [`fixture_path`] để biết điều đó.
pub fn fixture_root() -> PathBuf {
    if let Some(dir) = std::env::var_os(FIXTURES_ENV) {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    // CARGO_MANIFEST_DIR là src-tauri/, còn target/fixtures nằm ở gốc dự án.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("fixtures")
}

/// Đường dẫn tới một repo mẫu, `None` nếu nó chưa tồn tại trên đĩa.
///
/// Kiểm cả thư mục repo **và** thư mục `.git` bên trong: một thư mục rỗng còn sót lại
/// từ lần chạy script thất bại không phải là repo dùng được.
///
/// `shallow` và `detached` là bản clone nên `.git` của chúng là thư mục thật; repo do
/// `git init` sinh ra cũng vậy. Không repo mẫu nào là worktree (nơi `.git` là *tệp*),
/// nên kiểm `.git` tồn tại là đủ, không cần phân biệt tệp hay thư mục.
pub fn fixture_path(name: &str) -> Option<PathBuf> {
    let path = fixture_root().join(name);
    if path.is_dir() && path.join(".git").exists() {
        Some(path)
    } else {
        None
    }
}

/// Lấy đường dẫn repo mẫu, hoặc `None` **kèm lời nhắc in ra stderr** nếu thiếu.
///
/// Dùng ở đầu test cần fixture:
///
/// ```no_run
/// # use git_plum_lib::testing::require_fixture;
/// #[test]
/// fn phan_tich_duoc_merge_octopus() {
///     let Some(repo) = require_fixture("octopus") else {
///         return; // đã in lời nhắc, không phải lỗi
///     };
///     // ... dùng repo
/// }
/// ```
///
/// Không panic **có chủ ý**: xem ghi chú ở đầu module.
pub fn require_fixture(name: &str) -> Option<PathBuf> {
    match fixture_path(name) {
        Some(path) => Some(path),
        None => {
            eprintln!(
                "BỎ QUA TEST: thiếu repo mẫu '{}' (tìm ở {}).\n  \
                 Sinh lại bằng: {}\n  \
                 Hoặc trỏ tới thư mục khác bằng biến môi trường {}.",
                name,
                fixture_root().join(name).display(),
                FIXTURES_COMMAND,
                FIXTURES_ENV,
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `fixture_root()` phải tôn trọng `GIT_PLUM_FIXTURES`.
    ///
    /// Test này đặt biến môi trường nên **không** chạy song song với test khác cùng
    /// đọc biến đó. Ở đây chỉ có một test đặt biến, và nó dọn lại ngay sau khi đọc.
    ///
    /// `set_var`/`remove_var` là `unsafe` từ Rust 2024 (đổi môi trường không an toàn
    /// khi có nhiều luồng). Dự án dùng edition 2021 nên vẫn gọi được trực tiếp; bọc
    /// trong khối hẹp và dọn ngay để giảm thời gian biến tồn tại.
    #[test]
    fn fixture_root_doc_bien_moi_truong() {
        let goc_ban_dau = std::env::var_os(FIXTURES_ENV);

        std::env::set_var(FIXTURES_ENV, "/duong/dan/gia/cho/test");
        let duoc = fixture_root();

        // Dọn trước khi assert: assert thất bại sẽ panic và bỏ qua phần dọn.
        match goc_ban_dau {
            Some(cu) => std::env::set_var(FIXTURES_ENV, cu),
            None => std::env::remove_var(FIXTURES_ENV),
        }

        assert_eq!(duoc, PathBuf::from("/duong/dan/gia/cho/test"));
    }

    /// `fixture_path` trả `None` cho tên không tồn tại — kể cả khi thư mục fixture
    /// thật đã được sinh ra. Đây là điều kiện để `require_fixture` bỏ qua test thay
    /// vì panic.
    #[test]
    fn fixture_path_tra_none_cho_ten_khong_ton_tai() {
        assert!(
            fixture_path("khong-he-co-repo-mau-ten-nay").is_none(),
            "tên fixture không tồn tại phải cho None, không phải một đường dẫn"
        );
    }

    /// Bảng tên fixture phải khớp bảng trong README: chín repo, không trùng lặp.
    /// Trùng tên là lỗi sao chép dán, và nó làm test lặng lẽ kiểm hai lần một hình dạng.
    #[test]
    fn danh_sach_fixture_du_chin_va_khong_trung() {
        assert_eq!(FIXTURE_NAMES.len(), 9, "phải có đúng chín repo mẫu");

        let mut da_sap_xep = FIXTURE_NAMES.to_vec();
        da_sap_xep.sort_unstable();
        let so_luong_ban_dau = da_sap_xep.len();
        da_sap_xep.dedup();
        assert_eq!(
            da_sap_xep.len(),
            so_luong_ban_dau,
            "FIXTURE_NAMES có tên trùng lặp"
        );
    }
}
