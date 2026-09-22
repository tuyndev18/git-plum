//! Test tích hợp cho `get_file_history` — plan 03-05, DIFF-05.
//!
//! # Vì sao chạy git THẬT, không dùng buffer tự dựng
//!
//! Bộ phân tích đã có test đơn vị riêng (`git::parsers::file_history`), và ở đó buffer
//! tự dựng là đúng vì thứ được kiểm là một *hàm thuần*. Ở đây thứ được kiểm là **git
//! có trả về điều ta tưởng không** — và chỉ git thật trả lời được.
//!
//! Bài học `%x1f` của 02-04, chép nguyên: **19 test đơn vị xanh trên buffer tự dựng**
//! trong khi lệnh git thật trả về một định dạng khác, và thanh bên rỗng hoàn toàn mà
//! không một thông báo lỗi nào.
//!
//! # Fixture thiếu thì BỎ QUA ồn ào, không đỏ mù mờ
//!
//! `target/fixtures` **đã từng bị dọn mất một lần** (ghi trong 02-07-SUMMARY).

use std::sync::Arc;

use git_plum_lib::commands::diff::{lay_lich_su_tep, MAX_FILE_HISTORY};
use git_plum_lib::state::AppState;
use git_plum_lib::testing::{require_diff_fixture, DiffFixture};

fn mo() -> Option<(AppState, Arc<git_plum_lib::state::RepoHandle>, DiffFixture)> {
    let fx = require_diff_fixture()?;
    let state = AppState::new();
    let repo = state.open_repo(&fx.repo);
    Some((state, repo, fx))
}

/// Một tệp bị sửa qua nhiều commit → nhiều phiên bản, `commit_id` là 40 hex thật.
#[tokio::test]
async fn tep_nhieu_commit_tra_nhieu_phien_ban_voi_sha_40_hex() {
    let Some((state, repo, _fx)) = mo() else { return };

    let ls = lay_lich_su_tep(&state, repo, "text-simple.txt")
        .await
        .expect("tệp có trong lịch sử phải trả Ok");

    assert!(
        ls.versions.len() >= 2,
        "text-simple.txt phải có ≥2 phiên bản, đọc được {}",
        ls.versions.len()
    );

    for v in &ls.versions {
        assert_eq!(
            v.commit_id.len(),
            40,
            "commit_id phải là 40 hex, đọc được {:?}",
            v.commit_id
        );
        assert!(
            v.commit_id.chars().all(|c| c.is_ascii_hexdigit()),
            "commit_id phải toàn hex: {:?}",
            v.commit_id
        );
        assert!(
            !v.status.is_empty() && !v.status.contains('\n'),
            "🔴 status méo nghĩa là dấu phân tách \\0\\n chưa được cắt: {:?}",
            v.status
        );
        assert!(
            v.author_time > 0,
            "timestamp thật phải > 0, đọc được {}",
            v.author_time
        );
        assert!(!v.subject.is_empty(), "tiêu đề commit không được rỗng");
    }
}

/// 🔴 **Mutation `--follow`.** Lịch sử phải VƯỢT QUA chỗ đổi tên.
///
/// Repo mẫu có `renamed.txt` → `renamed-new.txt` (commit `rename_mod`, `R077`). Bỏ
/// `--follow` khỏi lệnh làm test này đỏ: không có nó, git dừng ở commit đổi tên và
/// phiên bản `A renamed.txt` phía trước **biến mất** — người dùng không có cách nào
/// biết phần lịch sử đó tồn tại. Hỏng im lặng, tệ hơn chậm.
#[tokio::test]
async fn lich_su_vuot_qua_cho_doi_ten_va_noi_ra_ten_cu() {
    let Some((state, repo, _fx)) = mo() else { return };

    let ls = lay_lich_su_tep(&state, repo, "renamed-new.txt")
        .await
        .expect("tệp đã đổi tên phải trả Ok");

    assert!(
        ls.versions.len() >= 2,
        "🔴 không có --follow thì lịch sử ĐỨT ở chỗ đổi tên và chỉ còn {} phiên bản. \
         Phải có ≥2: bản ghi R và bản ghi A của tên cũ phía trước nó",
        ls.versions.len()
    );

    let doi_ten = ls
        .versions
        .iter()
        .find(|v| v.old_path.is_some())
        .expect("🔴 phải có một phiên bản mang old_path — đó là thứ --follow mua được");

    assert_eq!(
        doi_ten.old_path.as_deref(),
        Some("renamed.txt"),
        "old_path phải là tên CŨ"
    );
    assert_eq!(
        doi_ten.path, "renamed-new.txt",
        "path phải là tên MỚI tại commit đó"
    );
    assert!(
        doi_ten.status.starts_with('R'),
        "bản ghi đổi tên phải mang status R*, đọc được {:?}",
        doi_ten.status
    );

    // Phiên bản **trước** lần đổi tên mang tên CŨ — bằng chứng lịch sử đã vượt qua.
    assert!(
        ls.versions.iter().any(|v| v.path == "renamed.txt"),
        "🔴 phải có ít nhất một phiên bản mang tên cũ `renamed.txt`; thiếu nó nghĩa là \
         lịch sử dừng ở chỗ đổi tên"
    );
}

/// Commit đụng nhiều tệp: git **đã lọc sẵn** theo pathspec.
///
/// 🔴 Test này ghim một phát hiện ĐO ĐƯỢC ngược với `<behavior>` của plan. Plan viết
/// *"`--follow` vẫn in mọi tệp của commit đó"* và đòi một bước lọc theo path đang
/// theo. Đo thật trên git 2.54: đầu ra chỉ chứa path được hỏi, kể cả với commit sửa
/// nhiều tệp. Xem `03-05-SUMMARY.md` để có đoạn `od -c`.
///
/// Nó vẫn là một cổng có giá trị: nếu một phiên bản git về sau đổi hành vi này thì
/// test đỏ, và ta biết phải thêm phép lọc chứ không phải phát hiện qua người dùng.
#[tokio::test]
async fn mot_commit_dung_nhieu_tep_git_da_loc_san_theo_pathspec() {
    let Some((state, repo, _fx)) = mo() else { return };

    let ls = lay_lich_su_tep(&state, repo, "text-simple.txt")
        .await
        .expect("phải trả Ok");

    for v in &ls.versions {
        assert!(
            v.path == "text-simple.txt" || v.old_path.is_some(),
            "🔴 lịch sử của một tệp KHÔNG được chứa tệp khác. Thấy {:?} (old_path {:?}). \
             Nếu git đổi hành vi lọc thì phải thêm phép lọc theo path đang theo",
            v.path,
            v.old_path
        );
    }

    assert!(
        !ls.versions.is_empty(),
        "tiền đề: phải có phiên bản để kiểm, nếu không test này luôn xanh"
    );
}

/// Tệp không có trong lịch sử → 0 phiên bản, **không** lỗi.
#[tokio::test]
async fn tep_khong_ton_tai_cho_danh_sach_rong_khong_loi() {
    let Some((state, repo, _fx)) = mo() else { return };

    let ls = lay_lich_su_tep(&state, repo, "khong-he-ton-tai-bao-gio.txt")
        .await
        .expect("tệp không có trong lịch sử là ca BÌNH THƯỜNG, không phải ca lỗi");

    assert_eq!(ls.versions.len(), 0);
    assert!(!ls.truncated);
    assert_eq!(ls.path, "khong-he-ton-tai-bao-gio.txt");
}

/// Tên tệp không UTF-8 → vẫn trả về, bản ghi không mất (HIST-11 ở tầng lịch sử tệp).
#[tokio::test]
async fn ten_tep_khong_utf8_van_doc_duoc_lich_su() {
    let Some(fx) = git_plum_lib::testing::fixture_path("non-utf8") else {
        return;
    };
    let state = AppState::new();
    let repo = state.open_repo(&fx);

    // `café.txt` có ký tự ngoài ASCII; repo `non-utf8` cũng có tên byte thô.
    let ls = lay_lich_su_tep(&state, repo, "café.txt")
        .await
        .expect("tên tệp có dấu phải đọc được lịch sử, không panic");

    // Không khẳng định số lượng: repo mẫu có thể quote path khác nhau theo nền tảng.
    // Điều được khẳng định là **không panic** và trường hiển thị đọc được.
    for v in &ls.versions {
        assert!(!v.commit_id.is_empty());
    }
}

/// 🔴 **Mutation `.arg("--")`.** Cổng đọc **thân hàm thật**, không tự dựng lệnh git.
///
/// Bài học HIST-10: lần đầu mutation xoá `.arg("--")` cho **0** test đỏ vì test tự
/// dựng một lệnh git riêng rồi kiểm lệnh đó — tức kiểm đúng cái nó tự bịa ra.
///
/// Neo vào `PATHSPEC_SAU_DAU_GACH` (hàm đồng nhất của `commands/diff.rs`) là cách
/// hiện có của dự án để cổng này quan sát được **thứ tự** thật trong argv.
///
/// # 🔴 Bản đầu của cổng này XANH SAI — và nó khớp chính chú thích của mutation
///
/// Bản đầu tìm `.arg("--")` trên thân hàm **thô**. Khi chạy mutation, dòng bị xoá
/// được thay bằng một chú thích `// MUTATION 5: .arg("--") removed` — và cổng khớp
/// **chuỗi trong chú thích đó**, nên nó xanh trong khi lệnh git thật đã mất dấu ngăn
/// cách. Đây là lần thứ **năm** dự án gặp lớp lỗi này (bốn lần trước ghi ở
/// `03-04-SUMMARY.md`), và nó đúng như quy tắc đã ghi ở đó:
///
/// > **Mọi cổng đọc nguồn phải bỏ chú thích trước khi tìm.**
///
/// Điều làm ca này đáng ghi riêng: chú thích gây nhiễu **không** có sẵn trong mã —
/// nó do chính phép kiểm mutation sinh ra. Nên một cổng đọc nguồn thô có thể sống
/// qua mọi lần review mà chỉ hỏng đúng lúc người ta đang kiểm nó.
#[test]
fn lenh_lich_su_tep_co_dau_gach_truoc_pathspec() {
    let ma = include_str!("../src/commands/diff.rs");
    let than = than_ham(ma, "pub async fn lay_lich_su_tep");

    // Tiền đề: thật sự lấy được thân hàm. Không có khẳng định này thì một phép cắt
    // hỏng làm `than` rỗng và mọi `contains` phía dưới xanh vì lý do sai — đã gặp
    // thật trong wave 4.
    assert!(
        than.len() > 200,
        "tiền đề: phải cắt được thân hàm lay_lich_su_tep, cắt được {} byte",
        than.len()
    );

    // 🔴 Tìm trên thân hàm ĐÃ BỎ CHÚ THÍCH — xem doc comment ở trên.
    let sach = bo_chu_thich(&than);
    assert!(
        sach.contains("GitCommand::new"),
        "tiền đề: bỏ chú thích không được ăn mất lệnh git.\n{sach}"
    );

    let vt_gach = sach
        .find(r#".arg("--")"#)
        .expect("🔴 thiếu .arg(\"--\") — một path trùng tên nhánh sẽ bị git hiểu là revision (T-03-32)");
    let vt_path = sach
        .find("PATHSPEC_SAU_DAU_GACH")
        .expect("pathspec phải đi qua PATHSPEC_SAU_DAU_GACH để cổng này quan sát được thứ tự");

    assert!(
        vt_gach < vt_path,
        "🔴 `--` phải đứng TRƯỚC pathspec (T-02-11/T-03-32)"
    );
}

/// `--follow` và `--max-count` có mặt trong **lệnh**, không chỉ trong chú thích.
///
/// 🔴 Cổng này bỏ chú thích trước khi tìm. Bài học wave 4, lần thứ **tư** dự án gặp
/// lớp lỗi này: ba cổng màu tìm trên CSS thô và khớp chính doc comment của mình, nên
/// vẫn xanh sau khi xoá hẳn quy tắc. `file_history.rs` và `diff.rs` **đều** có doc
/// comment tiếng Việt dẫn nguyên văn `--follow` và `--max-count` để giải thích quyết
/// định — nên một `grep` trên nguồn thô ở đây là một cổng tự vô hiệu hoá.
///
/// `plan` của 03-05 nói thẳng điều này trong mục "Cổng grep KHÔNG dùng", và cổng đúng
/// cho `--follow` là test đổi tên ở trên. Cổng này là lớp thứ hai cho `--max-count`,
/// vốn **không** có test hành vi nào (repo mẫu không đủ 200 commit nên đếm kết quả
/// không chứng minh được gì).
#[test]
fn than_ham_khong_chu_thich_co_follow_va_max_count() {
    let ma = include_str!("../src/commands/diff.rs");
    let than = than_ham(ma, "pub async fn lay_lich_su_tep");
    let sach = bo_chu_thich(&than);

    assert!(
        sach.len() > 150,
        "tiền đề: bỏ chú thích không được ăn hết thân hàm, còn {} byte",
        sach.len()
    );

    assert!(
        sach.contains("--follow"),
        "🔴 `--follow` phải ở trong LỆNH, không chỉ trong chú thích giải thích nó.\n\
         Thân hàm sau khi bỏ chú thích:\n{sach}"
    );
    assert!(
        sach.contains("--max-count"),
        "🔴 `--max-count` phải ở trong LỆNH — nó là chặn trên biến chi phí --follow \
         thành hằng số (T-03-33).\nThân hàm sau khi bỏ chú thích:\n{sach}"
    );
    assert!(
        sach.contains("MAX_FILE_HISTORY"),
        "con số phải đi qua hằng MAX_FILE_HISTORY, không gõ số rời"
    );
}

/// Chặn trên là 200, và giao diện cần con số đó để viết "200 phiên bản gần nhất".
#[test]
fn max_file_history_la_200() {
    assert_eq!(MAX_FILE_HISTORY, 200);
}

/// `get_file_history` đã đăng ký trong `generate_handler!`.
///
/// 🔴 Phân tích **khối** `generate_handler!`, không `grep -c`: 02-04 đã thay hai cổng
/// grep bằng test thật vì cổng cũ xanh khi command chưa đăng ký (chuỗi khớp một chỗ
/// khác trong tệp).
#[test]
fn get_file_history_da_dang_ky_trong_generate_handler() {
    let ma = include_str!("../src/lib.rs");
    let sach = bo_chu_thich(ma);

    let sau = sach
        .split_once("generate_handler![")
        .expect("lib.rs phải có generate_handler![")
        .1;
    let khoi = sau
        .split_once(']')
        .expect("khối generate_handler! phải đóng bằng ]")
        .0;

    assert!(
        khoi.contains("get_file_diff"),
        "tiền đề: khối phải chứa các command đã có, nếu không phép cắt sai"
    );
    assert!(
        khoi.contains("get_file_history"),
        "🔴 get_file_history chưa đăng ký — nó gọi được từ Rust nhưng webview nhận \
         lỗi lúc CHẠY, không lúc biên dịch.\nKhối generate_handler!:\n{khoi}"
    );
}

// --- Tiện ích đọc mã nguồn -------------------------------------------------

/// Cắt thân một hàm: từ sau `{` của khai báo tới `}` khớp ngoặc.
///
/// Đếm ngoặc chứ không tìm `\n}`: một hàm có khối lồng (và `lay_lich_su_tep` có) làm
/// phép tìm chuỗi cắt sớm, và cổng dựa trên đoạn cắt sai sẽ đỏ vì lý do sai — đã gặp
/// thật ở cổng bố cục của wave 4.
fn than_ham(ma: &str, khai_bao: &str) -> String {
    let sau = match ma.split_once(khai_bao) {
        Some((_, s)) => s,
        None => return String::new(),
    };
    let Some(mo) = sau.find('{') else {
        return String::new();
    };
    let mut sau_mo = 1usize;
    let byte = sau.as_bytes();
    let mut i = mo + 1;
    while i < byte.len() && sau_mo > 0 {
        match byte[i] {
            b'{' => sau_mo += 1,
            b'}' => sau_mo -= 1,
            _ => {}
        }
        i += 1;
    }
    sau[mo + 1..i.saturating_sub(1)].to_owned()
}

/// Bỏ chú thích `//`, `///` và `/* */` khỏi mã nguồn Rust.
///
/// 🔴 **Mọi cổng đọc nguồn trong dự án này phải bỏ chú thích trước khi tìm.** Wave 4
/// gặp lớp lỗi này lần thứ tư: tệp nào cũng có doc comment tiếng Việt dày dẫn chiếu
/// chính thứ đang bị cấm hoặc đang bị đòi, nên cổng khớp chú thích của mình và tự vô
/// hiệu hoá.
fn bo_chu_thich(ma: &str) -> String {
    let mut ra = String::with_capacity(ma.len());
    let mut trong_khoi = false;
    for dong in ma.lines() {
        let mut d = dong;
        if trong_khoi {
            match d.find("*/") {
                Some(k) => {
                    trong_khoi = false;
                    d = &d[k + 2..];
                }
                None => continue,
            }
        }
        // Chuỗi ký tự có thể chứa `//`; ở tệp này không có ca đó, và một phép bỏ
        // chú thích nhận biết chuỗi là một bộ phân tích Rust thu nhỏ. Thay vào đó
        // mỗi cổng có một khẳng định **tiền đề** về độ dài để phép lọc quá tay không
        // làm cổng luôn xanh.
        let d = match d.find("/*") {
            Some(k) => {
                trong_khoi = !d[k..].contains("*/");
                &d[..k]
            }
            None => d,
        };
        let d = match d.find("//") {
            Some(k) => &d[..k],
            None => d,
        };
        ra.push_str(d);
        ra.push('\n');
    }
    ra
}
