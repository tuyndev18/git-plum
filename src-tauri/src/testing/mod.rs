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

// --- Repo mẫu cho trình xem diff (Phase 3) ---------------------------------

/// Lệnh sinh repo mẫu diff. In ra trong thông báo bỏ qua test.
pub const DIFF_FIXTURES_COMMAND: &str = "bash scripts/fixtures/make-diff-fixtures.sh";

/// Thư mục con chứa repo mẫu diff, dưới [`fixture_root`].
pub const DIFF_FIXTURES_DIR: &str = "diff-cases";

/// Repo mẫu diff cộng bảng SHA của nó.
///
/// Khác bộ fixture của Phase 2 (chín repo, mỗi repo một hình dạng đồ thị): ở đây là
/// **một** repo mang nhiều hình dạng **tệp**, vì thứ được kiểm là cách đọc bản vá của
/// một tệp chứ không phải hình học lịch sử.
pub struct DiffFixture {
    pub repo: PathBuf,
    /// Nhãn → SHA, đọc từ `shas.txt` mà script in ra.
    shas: std::collections::HashMap<String, String>,
}

impl DiffFixture {
    /// SHA của commit mang nhãn `nhan`.
    ///
    /// Panic khi nhãn không có: nhãn là hằng viết trong test, nên thiếu nhãn là lỗi
    /// lập trình chứ không phải thiếu fixture (thiếu fixture đã được
    /// [`require_diff_fixture`] xử lý trước đó). Thông điệp liệt kê nhãn có sẵn để
    /// người sửa không phải mở script ra tra.
    pub fn sha(&self, nhan: &str) -> &str {
        self.shas.get(nhan).map(String::as_str).unwrap_or_else(|| {
            let mut co: Vec<&str> = self.shas.keys().map(String::as_str).collect();
            co.sort_unstable();
            panic!(
                "không có nhãn commit '{nhan}' trong repo mẫu diff.\n  \
                 Nhãn có sẵn: {}\n  \
                 Sinh lại bằng: {DIFF_FIXTURES_COMMAND}",
                co.join(", ")
            )
        })
    }
}

/// Lấy repo mẫu diff, hoặc `None` **kèm lời nhắc ra stderr** nếu thiếu.
///
/// Cùng hợp đồng với [`require_fixture`] và cùng lý do: người mới clone repo về chạy
/// `cargo test` phải thấy xanh kèm lời nhắc, không thấy một bức tường đỏ. Fixture
/// **đã từng bị dọn mất một lần** (ghi trong 02-07-SUMMARY), nên đường đi "thiếu
/// fixture" là đường đi có thật, không phải phòng xa.
///
/// Kiểm **cả ba** thứ: thư mục repo, `.git` bên trong, và `shas.txt`. Một thư mục còn
/// sót từ lần chạy script thất bại có hai thứ đầu mà không có thứ ba, và test dựa vào
/// nó sẽ đỏ vì một lý do hoàn toàn không liên quan.
pub fn require_diff_fixture() -> Option<DiffFixture> {
    let goc = fixture_root().join(DIFF_FIXTURES_DIR);
    let repo = goc.join("repo");
    let bang = goc.join("shas.txt");

    if !repo.is_dir() || !repo.join(".git").exists() || !bang.is_file() {
        eprintln!(
            "BỎ QUA TEST: thiếu repo mẫu diff (tìm ở {}).\n  \
             Sinh lại bằng: {}\n  \
             Hoặc trỏ tới thư mục khác bằng biến môi trường {}.",
            goc.display(),
            DIFF_FIXTURES_COMMAND,
            FIXTURES_ENV,
        );
        return None;
    }

    let noi_dung = std::fs::read_to_string(&bang).ok()?;
    let shas = noi_dung
        .lines()
        .filter_map(|l| {
            let mut it = l.split_whitespace();
            Some((it.next()?.to_owned(), it.next()?.to_owned()))
        })
        .collect();

    Some(DiffFixture { repo, shas })
}

// --- Repo mẫu cho trạng thái thư mục làm việc (Phase 4) --------------------

/// Lệnh sinh repo mẫu trạng thái. In ra trong thông báo bỏ qua test.
///
/// # 🔴 KHÔNG phải [`FIXTURES_COMMAND`]
///
/// `status-cases` do một script **riêng** sinh ra, nên `make-fixtures.sh` **không**
/// dựng nó. In nhầm lệnh ở đây nghĩa là người mới clone repo thấy test bỏ qua, chạy
/// đúng lệnh được bảo, và test **vẫn** bỏ qua — không có gì nói cho họ biết tại sao.
/// Cùng lý do [`DIFF_FIXTURES_COMMAND`] tồn tại tách khỏi [`FIXTURES_COMMAND`].
pub const STATUS_FIXTURES_COMMAND: &str = "bash scripts/fixtures/make-status-fixtures.sh";

/// Thư mục con chứa repo mẫu trạng thái, dưới [`fixture_root`].
pub const STATUS_FIXTURES_DIR: &str = "status-cases";

/// Lấy repo mẫu trạng thái, hoặc `None` **kèm lời nhắc ra stderr** nếu thiếu.
///
/// Repo này khác mọi fixture trước ở một điểm: giá trị của nó nằm ở **thư mục làm việc
/// bẩn**, không ở lịch sử. Nó mang một tệp đổi tên (`a_old.txt` → `a_new.txt`, bản ghi
/// dạng `2`) đứng **trước** ít nhất hai bản ghi khác, đúng hình dạng làm lệch nấc một
/// bộ phân tích tách NUL trơn.
///
/// Chạy `git commit`, `git reset` hay `git clean` trong repo đó là **phá fixture**.
pub fn require_status_fixture() -> Option<PathBuf> {
    let goc = fixture_root().join(STATUS_FIXTURES_DIR);
    let repo = goc.join("repo");

    if !repo.is_dir() || !repo.join(".git").exists() {
        eprintln!(
            "BỎ QUA TEST: thiếu repo mẫu trạng thái (tìm ở {}).\n  \
             Sinh lại bằng: {}\n  \
             Hoặc trỏ tới thư mục khác bằng biến môi trường {}.",
            repo.display(),
            STATUS_FIXTURES_COMMAND,
            FIXTURES_ENV,
        );
        return None;
    }

    Some(repo)
}

/// Tên hai repo mẫu mang hook **từ chối**, dưới [`STATUS_FIXTURES_DIR`] — plan 04-03.
///
/// `hook-reject` có `.git/hooks/pre-commit` thoát 1; `hook-msg-reject` có `commit-msg`
/// thoát 1. Cả hai in một chuỗi nhận diện ra **cả stdout lẫn stderr** của hook, để test
/// khẳng định được rằng phép phân loại lỗi đọc **cả hai** luồng.
pub const HOOK_FIXTURE_NAMES: [&str; 2] = ["hook-reject", "hook-msg-reject"];

/// Lấy một repo mẫu có hook từ chối, hoặc `None` **kèm lời nhắc ra stderr** nếu thiếu.
///
/// # 🔴 Trả về repo GỐC — người gọi phải SAO CHÉP trước khi commit vào đó
///
/// Khác [`require_status_fixture`] ở một điểm quan trọng: test hook **phải** chạy
/// `git commit` thật (đó là cả điểm của nó), và một commit **thành công** qua đường
/// `--no-verify` **tiêu thụ** tệp đã stage của fixture. Lần chạy test thứ hai sẽ gặp
/// một repo không còn gì để commit, và ca "no-verify thì thành công" sẽ đỏ vì
/// *"nothing to commit"* — một lý do hoàn toàn khác với thứ nó định kiểm.
///
/// Đó là lớp lỗi "test đổi màu vì một lý do khác hẳn", và nó chỉ lộ ra ở **lần chạy
/// thứ hai** — tức nó đi lọt qua được một lần đo, đúng kiểu cổng sống sót cả một wave.
pub fn require_hook_fixture(ten: &str) -> Option<PathBuf> {
    assert!(
        HOOK_FIXTURE_NAMES.contains(&ten),
        "'{ten}' không phải một repo mẫu hook; có: {HOOK_FIXTURE_NAMES:?}"
    );

    let repo = fixture_root().join(STATUS_FIXTURES_DIR).join(ten);

    if !repo.is_dir() || !repo.join(".git").exists() {
        eprintln!(
            "BỎ QUA TEST: thiếu repo mẫu hook '{}' (tìm ở {}).\n  \
             Sinh lại bằng: {}\n  \
             Hoặc trỏ tới thư mục khác bằng biến môi trường {}.",
            ten,
            repo.display(),
            STATUS_FIXTURES_COMMAND,
            FIXTURES_ENV,
        );
        return None;
    }

    Some(repo)
}

// --- Repo mẫu cho staging theo khối (Phase 5) ------------------------------

/// Lệnh sinh repo mẫu hunk. In ra trong thông báo bỏ qua test.
///
/// # 🔴 KHÔNG phải [`FIXTURES_COMMAND`] và cũng không phải [`STATUS_FIXTURES_COMMAND`]
///
/// Lý do giống hệt hai hằng trước nó: `hunk-cases` do một script **riêng** sinh ra. In
/// nhầm lệnh ở đây nghĩa là người mới clone repo thấy test bỏ qua, chạy đúng lệnh được
/// bảo, và test **vẫn** bỏ qua — không có gì nói cho họ biết tại sao.
pub const HUNK_FIXTURES_COMMAND: &str = "bash scripts/fixtures/make-hunk-fixtures.sh";

/// Thư mục con chứa repo mẫu hunk, dưới [`fixture_root`].
pub const HUNK_FIXTURES_DIR: &str = "hunk-cases";

/// Lấy repo mẫu hunk, hoặc `None` **kèm lời nhắc ra stderr** nếu thiếu.
///
/// Cùng hợp đồng với [`require_status_fixture`]: **không panic**, in nhắc rồi trả
/// `None` để test `return` sớm. Xem ghi chú đầu module.
///
/// # Repo này mang gì
///
/// Bốn tệp, mỗi tệp một hình dạng làm vỡ một bộ **dựng bản vá con**:
///
/// | Tệp | Hình dạng | Vì sao |
/// |---|---|---|
/// | `ba-khoi.txt` | 26 dòng LF, sửa dòng 2/13/25 → **đúng 3** hunk ở `-U3` | ca tách hunk cơ bản |
/// | `crlf-khong-dong-cuoi.txt` | 30 dòng CRLF, **không** `\n` cuối, byte `0xE9` dòng 6 | WORK-04, ba lớp một tệp |
/// | `nhi-phan.bin` | có byte NUL | R7 — phải từ chối rõ ràng |
/// | `doi-ten-va-sua.txt` | `git mv` rồi sửa nội dung | ca ROADMAP "vừa đổi tên vừa sửa" |
///
/// # 🔴 `core.autocrlf` của repo này là `true`, KHÔNG phải `false`
///
/// Khác **cả ba** bộ fixture trước, vốn đặt `false` để đầu ra tất định. Ở đây `true` là
/// cả điểm: mặc định trên Windows là `true` và đó là máy chủ dự án, nên một fixture đặt
/// `false` sẽ né mất chính ca R2 (CRLF bị chuẩn hoá mất) mà nó tồn tại để kiểm.
///
/// **Hệ quả đã đo, và nó đổi cách viết test:** với `autocrlf=true`, `git diff` in bản
/// vá bằng **LF thuần** — clean filter bỏ `\r` trước khi git so sánh. Bản vá từ repo
/// này **không chứa byte `\r` nào**. Đó là đúng, không phải lỗi. Nên ca "CRLF không bị
/// trim khi tách hunk" **không dựng được** từ repo này; nó phải là test **byte-literal**
/// trong `git::patch_build`.
///
/// # Repo để BẨN có chủ ý
///
/// Giá trị nằm ở thư mục làm việc, không ở lịch sử. Chạy `git commit`, `git reset` hay
/// `git clean` trong đó là **phá fixture**.
pub fn require_hunk_fixture() -> Option<PathBuf> {
    let goc = fixture_root().join(HUNK_FIXTURES_DIR);
    let repo = goc.join("repo");

    if !repo.is_dir() || !repo.join(".git").exists() {
        eprintln!(
            "BỎ QUA TEST: thiếu repo mẫu hunk (tìm ở {}).\n  \
             Sinh lại bằng: {}\n  \
             Hoặc trỏ tới thư mục khác bằng biến môi trường {}.",
            repo.display(),
            HUNK_FIXTURES_COMMAND,
            FIXTURES_ENV,
        );
        return None;
    }

    Some(repo)
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

    /// `status-cases` **không** nằm trong [`FIXTURE_NAMES`], và đó là chủ ý.
    ///
    /// [`FIXTURE_NAMES`] là danh sách những repo mà [`FIXTURES_COMMAND`]
    /// (`make-fixtures.sh`) dựng được. `status-cases` do một script riêng dựng, nên nếu
    /// nó lọt vào danh sách đó thì [`require_fixture`] sẽ in ra lệnh **sai** và người
    /// mới clone repo chạy đúng lệnh được bảo mà test vẫn bỏ qua — không có gì nói cho
    /// họ biết tại sao. Test này ghim phân biệt đó để một lần "dọn dẹp" sau này không
    /// âm thầm gộp chúng lại.
    #[test]
    fn status_cases_khong_nam_trong_danh_sach_cua_make_fixtures() {
        assert!(
            !FIXTURE_NAMES.contains(&STATUS_FIXTURES_DIR),
            "'{STATUS_FIXTURES_DIR}' không được nằm trong FIXTURE_NAMES: \
             make-fixtures.sh KHÔNG dựng nó, nên require_fixture sẽ in lệnh sai"
        );
        assert_ne!(
            STATUS_FIXTURES_COMMAND, FIXTURES_COMMAND,
            "lệnh sinh status-cases phải khác lệnh sinh bộ fixture Phase 2"
        );
        assert!(
            STATUS_FIXTURES_COMMAND.contains("make-status-fixtures.sh"),
            "lệnh phải trỏ đúng script sinh status-cases, thấy: {STATUS_FIXTURES_COMMAND}"
        );
    }

    /// `hunk-cases` cũng **không** nằm trong [`FIXTURE_NAMES`], cùng lý do với
    /// `status-cases`, và lệnh sinh nó phải **khác cả hai** lệnh đã có.
    ///
    /// Test này tồn tại vì đây là lần thứ **ba** dự án thêm một bộ fixture có script
    /// riêng. Hai lần trước đều phải thêm một hằng `*_FIXTURES_COMMAND` tách biệt; một
    /// lần "dọn dẹp" sau này gộp chúng lại sẽ làm [`require_hunk_fixture`] in ra lệnh
    /// sai, và người chạy đúng lệnh được bảo vẫn thấy test bỏ qua — im lặng.
    #[test]
    fn hunk_cases_co_lenh_sinh_rieng_va_khong_nam_trong_make_fixtures() {
        assert!(
            !FIXTURE_NAMES.contains(&HUNK_FIXTURES_DIR),
            "'{HUNK_FIXTURES_DIR}' không được nằm trong FIXTURE_NAMES: \
             make-fixtures.sh KHÔNG dựng nó, nên require_fixture sẽ in lệnh sai"
        );
        assert_ne!(
            HUNK_FIXTURES_COMMAND, FIXTURES_COMMAND,
            "lệnh sinh hunk-cases phải khác lệnh sinh bộ fixture Phase 2"
        );
        assert_ne!(
            HUNK_FIXTURES_COMMAND, STATUS_FIXTURES_COMMAND,
            "lệnh sinh hunk-cases phải khác lệnh sinh status-cases"
        );
        assert!(
            HUNK_FIXTURES_COMMAND.contains("make-hunk-fixtures.sh"),
            "lệnh phải trỏ đúng script sinh hunk-cases, thấy: {HUNK_FIXTURES_COMMAND}"
        );
        assert_ne!(
            HUNK_FIXTURES_DIR, STATUS_FIXTURES_DIR,
            "hai bộ fixture không được dùng chung một thư mục"
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
