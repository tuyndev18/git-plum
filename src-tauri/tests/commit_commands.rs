//! Test tích hợp cho `create_commit` / `amend_commit` — plan 04-03, WORK-08 và WORK-09.
//!
//! # Vì sao chạy git THẬT, và vì sao hook THẬT
//!
//! Ràng buộc trung tâm của plan này — *"hook `pre-commit` và `commit-msg` chạy MẶC
//! ĐỊNH; `no-verify` là công tắc tường minh"* — **chỉ** kiểm được bằng một hook thật
//! chặn một commit thật. Không có buffer tự dựng nào chứng minh được rằng cờ ta thêm
//! vào argv thật sự nối tới hành vi của git.
//!
//! Ca then chốt của cả plan nằm ở đây: **cùng một repo**, `no_verify = false` → bị từ
//! chối, `no_verify = true` → thành công. Hai nửa trên hai repo khác nhau không chứng
//! minh được gì (repo thứ hai có thể chưa từng có hook).
//!
//! # Mỗi test SAO CHÉP repo mẫu hook
//!
//! `require_hook_fixture` trả repo **gốc**. Test ở đây commit vào đó, và một commit
//! thành công **tiêu thụ** tệp đã stage của fixture — lần chạy thứ hai sẽ gặp
//! "nothing to commit" và đỏ vì một lý do hoàn toàn khác. Lỗi đó chỉ lộ ra ở lần chạy
//! **thứ hai**, tức nó đi lọt qua được một lần đo.
//!
//! # 🔴 KHÔNG dùng `git log` ở bất kỳ đâu
//!
//! Trần cứng 50 dòng trong môi trường này, và nó thất bại **im lặng**: `git log | grep
//! <sha>` trả exit 1 với 0 khớp cho một commit **có thật** (CONTEXT.md 3.6). Mọi câu
//! hỏi về tồn tại/số lượng đi qua `git rev-parse` hoặc `git rev-list --count`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use git_plum_lib::commands::commit::{sua_commit_gan_nhat, tao_commit};
use git_plum_lib::commands::worktree::lay_trang_thai;
use git_plum_lib::error::GitError;
use git_plum_lib::state::{AppState, RepoHandle};
use git_plum_lib::testing::require_hook_fixture;

/// Chạy một lệnh git đồng bộ trong lúc **dựng bối cảnh**. Không đi qua `GitRunner` có
/// chủ ý: đây không phải phần được kiểm.
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("không chạy được git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} thất bại: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Đọc stdout của một lệnh git, đã trim. Dùng cho `rev-parse` / `rev-list --count`.
fn git_ra(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("không chạy được git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} thất bại: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// SHA của HEAD. **`rev-parse`, không `git log`** — xem ghi chú đầu tệp.
fn head_sha(dir: &Path) -> String {
    git_ra(dir, &["rev-parse", "HEAD"])
}

/// Số commit reachable từ HEAD. **`rev-list --count`, không `git log | wc -l`.**
fn so_commit(dir: &Path) -> u32 {
    git_ra(dir, &["rev-list", "--count", "HEAD"])
        .parse()
        .expect("rev-list --count phải in một số")
}

/// Thông điệp của HEAD, nguyên văn (`%B`).
fn thong_diep_head(dir: &Path) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["show", "-s", "--format=%B", "HEAD"])
        .output()
        .expect("chạy được git show");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Sao chép đệ quy một thư mục. `.git` đi theo — đó là cả điểm.
fn sao_chep(tu: &Path, toi: &Path) {
    std::fs::create_dir_all(toi).expect("tạo được thư mục đích");
    for muc in std::fs::read_dir(tu).expect("đọc được thư mục nguồn") {
        let muc = muc.expect("đọc được mục");
        let dich = toi.join(muc.file_name());
        if muc.file_type().expect("đọc được loại").is_dir() {
            sao_chep(&muc.path(), &dich);
        } else {
            std::fs::copy(muc.path(), &dich).expect("sao chép được tệp");
        }
    }
}

/// Bản sao dùng-một-lần của một repo mẫu hook.
///
/// Trả `None` khi fixture chưa được sinh — test **bỏ qua**, không đỏ, cùng khuôn
/// `require_*_fixture` của các wave trước.
///
/// # Quyền chạy của hook phải sống sót qua phép sao chép
///
/// `std::fs::copy` giữ bit quyền trên Unix. Trên Windows không có bit executable và
/// git chạy hook qua shell đi kèm, nên phép sao chép đủ. Hàm này **khẳng định** hook
/// còn ở đúng chỗ sau khi chép — một hook mất tích cho mọi test hook xanh mà chẳng
/// kiểm gì (cổng #4 của CONTEXT.md 3.1).
fn repo_hook(ten: &str) -> Option<(tempfile::TempDir, AppState, Arc<RepoHandle>, PathBuf)> {
    let goc = require_hook_fixture(ten)?;

    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let dich = dir.path().join("repo");
    sao_chep(&goc, &dich);

    let ten_hook = if ten == "hook-reject" {
        "pre-commit"
    } else {
        "commit-msg"
    };
    let hook = dich.join(".git").join("hooks").join(ten_hook);
    assert!(
        hook.is_file(),
        "🔴 tiền đề: hook '{ten_hook}' phải còn sau khi sao chép fixture. Thiếu nó thì \
         MỌI test hook dưới đây xanh mà chẳng kiểm gì — đúng khuôn cổng #4 của \
         CONTEXT.md 3.1 (fixture không phân biệt được đột biến)"
    );

    // 🔴 Bản sao TỰ DỰNG tệp đã stage của nó, không nhận từ fixture.
    //
    // Đây là một lỗi đã xảy ra thật trong lúc viết plan này, không phải phòng xa. Repo
    // mẫu được script sinh ra **có** một `them.txt` đã stage; tôi chạy một lệnh
    // `git commit --no-verify` bằng tay trên repo gốc để đo hành vi hook, và lệnh đó
    // **tiêu thụ** tệp đó. Mọi lần chạy test sau đều thấy một repo sạch và đỏ ở phép
    // khẳng định "tệp vẫn stage" — vì một lý do **không liên quan** tới thứ nó kiểm.
    //
    // Bài học chung: một fixture mà test **dựa vào trạng thái tích luỹ** của nó là một
    // fixture chỉ đúng cho tới lần đầu ai đó chạm vào repo gốc. Dựng lại ở đây làm mỗi
    // test độc lập với lịch sử của fixture, kể cả khi ai đó đã commit vào nó.
    // ⚠️ Nội dung phải KHÁC thứ đang nằm ở HEAD, nếu không `git add` là một lệnh
    // không-làm-gì và index vẫn sạch. Đã mắc đúng lỗi này một lần: ghi lại đúng chuỗi
    // `"de commit\n"` mà fixture đã commit, rồi đỏ ở phép khẳng định "tệp vẫn stage"
    // với `entries: []` — trông y hệt một lỗi của mã sản phẩm.
    std::fs::write(dich.join("them.txt"), "noi dung rieng cua ban sao test\n")
        .expect("ghi được tệp");
    git(&dich, &["add", "--", "them.txt"]);

    let state = AppState::new();
    let repo = state.open_repo(&dich);

    // Khẳng định tiền đề: bản sao PHẢI có thứ để commit. Không có nó, ca then chốt
    // dưới đây đổi màu vì "nothing to commit" thay vì vì hành vi của hook.
    let ra = Command::new("git")
        .current_dir(&dich)
        .args(["diff", "--cached", "--name-only"])
        .output()
        .expect("chạy được git diff --cached");
    let da_stage = String::from_utf8_lossy(&ra.stdout);
    assert!(
        da_stage.contains("them.txt"),
        "🔴 tiền đề: bản sao phải có tệp đã stage để commit. Đọc được: {da_stage:?}"
    );

    Some((dir, state, repo, dich))
}

/// Repo tạm sạch, có một commit nền và một tệp đã stage sẵn để commit.
///
/// Không có hook — dùng cho các ca không liên quan tới hook.
fn repo_san_sang() -> (tempfile::TempDir, AppState, Arc<RepoHandle>, PathBuf) {
    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let p = dir.path().join("repo");
    std::fs::create_dir_all(&p).unwrap();

    git(&p, &["init", "--initial-branch=main"]);
    git(&p, &["config", "user.name", "Test"]);
    git(&p, &["config", "user.email", "test@example.com"]);
    // Một chuyển đổi CRLF làm `git status` báo tệp sửa mà test không mong.
    git(&p, &["config", "core.autocrlf", "false"]);
    git(&p, &["config", "commit.gpgsign", "false"]);

    std::fs::write(p.join("nen.txt"), "nen\n").unwrap();
    git(&p, &["add", "--", "nen.txt"]);
    git(&p, &["commit", "-m", "commit nen"]);

    std::fs::write(p.join("them.txt"), "de commit\n").unwrap();
    git(&p, &["add", "--", "them.txt"]);

    let state = AppState::new();
    let repo = state.open_repo(&p);
    (dir, state, repo, p)
}

// ===========================================================================
// Ca then chốt: hook chạy MẶC ĐỊNH, `no_verify` là công tắc THẬT
// ===========================================================================

/// 🔴 **Ca then chốt của cả plan — đột biến M1 và M2.**
///
/// **Cùng một repo**, hai giá trị `no_verify`:
/// * `false` → hook chạy, commit **bị chặn**, `HEAD` **không đổi**, tệp **vẫn** stage;
/// * `true`  → commit **thành công**, `HEAD` đổi.
///
/// Hai nửa phải ở **cùng** repo. Chạy chúng trên hai repo khác nhau không chứng minh
/// được gì — repo thứ hai có thể đơn giản là chưa từng có hook, và cổng sẽ xanh với
/// một `no_verify` bị nối dây sai hoàn toàn.
///
/// Nửa đầu bắt đột biến **M1** (thêm `--no-verify` vô điều kiện → hook không bao giờ
/// chạy → commit thành công → `HEAD` đổi).
/// Nửa sau bắt đột biến **M2** (bỏ hẳn nhánh thêm cờ → hook luôn chạy → commit luôn bị
/// chặn).
#[tokio::test]
async fn cung_repo_no_verify_false_bi_chan_true_thi_qua() {
    let Some((_dir, state, repo, p)) = repo_hook("hook-reject") else {
        return;
    };

    let truoc = head_sha(&p);
    let so_truoc = so_commit(&p);

    // --- Nửa 1: mặc định (no_verify = false) -> hook CHẶN ---
    let loi = tao_commit(&state, Arc::clone(&repo), "thu commit", false)
        .await
        .expect_err(
            "🔴 hook `pre-commit` thoát 1 phải CHẶN commit khi no_verify=false. Nó \
             thành công nghĩa là `--no-verify` đang được thêm VÔ ĐIỀU KIỆN (đột biến \
             M1) — và hook không bao giờ chạy, dù ROADMAP nói hook chạy MẶC ĐỊNH",
        );

    assert_eq!(
        head_sha(&p),
        truoc,
        "🔴 hook từ chối thì KHÔNG được có commit nào được tạo. HEAD đã đổi."
    );
    assert_eq!(
        so_commit(&p),
        so_truoc,
        "số commit không được tăng khi hook từ chối"
    );

    // Đầu ra hook phải tới người dùng NGUYÊN VĂN — đột biến M5.
    let thong_bao = loi.to_string();
    assert!(
        thong_bao.contains("HOOK-PRE-COMMIT-REJECTED"),
        "🔴 NGUYÊN VĂN đầu ra hook phải có trong lỗi (R5, đột biến M5). Đầu ra hook là \
         thứ DUY NHẤT nói cho người dùng biết phải sửa gì; thay nó bằng một thông báo \
         chung là vứt đi toàn bộ thông tin có ích đúng lúc họ cần nhất.\n\nNhận được: \
         {thong_bao}"
    );
    assert_eq!(
        loi.code(),
        "hook_rejected",
        "hook từ chối phải có mã riêng, không phải command_failed chung: {loi}"
    );

    // Tệp đã stage phải VẪN stage — đo hành vi này, không suy từ tài liệu.
    let st = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("đọc status sau khi hook từ chối phải Ok");
    assert!(
        st.entries.iter().any(|e| e.path == "them.txt"),
        "🔴 tệp đã stage phải CÒN sau khi hook từ chối — nếu không, ca `no_verify=true` \
         dưới đây sẽ không có gì để commit và nó sẽ đổi màu vì một lý do khác hẳn. \
         Đọc được: {:?}",
        st.entries
    );

    // --- Nửa 2: CÙNG repo, no_verify = true -> commit ĐI QUA ---
    let sau = tao_commit(&state, Arc::clone(&repo), "thu commit", true)
        .await
        .expect(
            "🔴 với no_verify=true, CÙNG repo đó phải commit THÀNH CÔNG. Thất bại nghĩa \
             là nhánh thêm `--no-verify` đã bị bỏ (đột biến M2) — công tắc chỉ là một ô \
             tick trang trí, không nối vào git",
        );

    assert_ne!(
        head_sha(&p),
        truoc,
        "🔴 no_verify=true phải tạo được commit thật — HEAD phải đổi"
    );
    assert_eq!(so_commit(&p), so_truoc + 1, "đúng MỘT commit được thêm");
    assert!(
        !sau.entries.iter().any(|e| e.path == "them.txt"),
        "sau khi commit, tệp vừa commit không còn ở nhóm đã stage. Đọc được: {:?}",
        sau.entries
    );
}

/// Hook `commit-msg` từ chối → cùng hành vi, lỗi mang nguyên văn đầu ra hook.
///
/// Hook này chạy ở một **giai đoạn khác** của `git commit` so với `pre-commit` (sau khi
/// thông điệp đã được đọc), nên nó là một đường mã khác của git. Một phép phân loại
/// lỗi đúng cho `pre-commit` không tự động đúng cho ca này.
#[tokio::test]
async fn hook_commit_msg_tu_choi_cung_giu_nguyen_van() {
    let Some((_dir, state, repo, p)) = repo_hook("hook-msg-reject") else {
        return;
    };

    let truoc = head_sha(&p);

    let loi = tao_commit(&state, Arc::clone(&repo), "thu commit", false)
        .await
        .expect_err("hook `commit-msg` thoát 1 phải chặn commit");

    assert_eq!(head_sha(&p), truoc, "HEAD không được đổi khi hook từ chối");
    assert!(
        loi.to_string().contains("HOOK-COMMIT-MSG-REJECTED"),
        "🔴 nguyên văn đầu ra hook `commit-msg` phải tới người dùng. Nhận được: {loi}"
    );

    // Cùng repo, no_verify=true -> `commit-msg` cũng bị bỏ qua.
    tao_commit(&state, Arc::clone(&repo), "thu commit", true)
        .await
        .expect("no_verify=true bỏ qua CẢ `commit-msg`, không chỉ `pre-commit`");
    assert_ne!(head_sha(&p), truoc, "no_verify=true phải tạo được commit");
}

// ===========================================================================
// Thông điệp: rỗng, nhiều dòng, bắt đầu bằng `-`, dòng `#`
// ===========================================================================

/// 🔴 **Đột biến M4: thông điệp rỗng bị chặn TRƯỚC khi chạy git.**
///
/// Đo bằng **`CommandLog`**, không chỉ bằng `is_err()`: git **cũng** từ chối thông điệp
/// rỗng, nên một phép khẳng định chỉ đọc `Err` xanh ở cả hai bên đột biến và không
/// phân biệt được gì. Chỉ nhật ký lệnh phân biệt "không chạy" với "chạy rồi thất bại".
#[tokio::test]
async fn thong_diep_rong_khong_sinh_lenh_git_nao() {
    let (_dir, state, repo, p) = repo_san_sang();
    let truoc = head_sha(&p);

    for rong in ["", "   ", "\n\n", " \t\r\n "] {
        let loi = tao_commit(&state, Arc::clone(&repo), rong, false)
            .await
            .expect_err("thông điệp rỗng/chỉ khoảng trắng phải là lỗi");

        assert_eq!(
            loi.code(),
            "empty_commit_message",
            "phải có mã lỗi riêng, không phải lỗi chung của git: {loi}"
        );
    }

    let so_lenh_commit = state
        .command_log
        .entries()
        .iter()
        .filter(|e| e.command.contains("commit"))
        .count();
    assert_eq!(
        so_lenh_commit, 0,
        "🔴 thông điệp rỗng KHÔNG được sinh lệnh git nào (đột biến M4). Chạy git ở đây \
         nghĩa là hook `pre-commit` — thường là linter cả cây, hàng chục giây — vừa \
         chạy cho một lệnh chắc chắn thất bại. Và trên repo có `commit-msg`, người dùng \
         sẽ nhận đầu ra hook thay vì câu 'thông điệp rỗng' rồi đi sửa nhầm chỗ. \
         Đọc được {so_lenh_commit} lệnh commit trong nhật ký"
    );
    assert_eq!(head_sha(&p), truoc, "không commit nào được tạo");
}

/// 🔴 **Đột biến M3: thông điệp nhiều dòng giữ nguyên xuống dòng.**
///
/// Subject + dòng trống + body là khuôn chuẩn của git, và nó phải sống sót qua đường
/// stdin `-F -`.
#[tokio::test]
async fn thong_diep_nhieu_dong_giu_nguyen_xuong_dong() {
    let (_dir, state, repo, p) = repo_san_sang();

    let msg = "dong subject\n\ndong body mot\ndong body hai\n";
    tao_commit(&state, Arc::clone(&repo), msg, false)
        .await
        .expect("commit nhiều dòng phải Ok");

    let doc = thong_diep_head(&p);
    assert!(doc.contains("dong subject"), "mất dòng subject: {doc:?}");
    assert!(
        doc.contains("dong body mot") && doc.contains("dong body hai"),
        "🔴 mất dòng body — thông điệp nhiều dòng phải giữ nguyên xuống dòng. Đây là \
         đột biến M3 (đổi `-F -` thành `-m`): trên Windows dòng lệnh là MỘT chuỗi rồi \
         tiến trình con tự tách, nên xuống dòng không sống sót. Đọc được: {doc:?}"
    );
    assert!(
        doc.contains("dong subject\n\ndong body mot"),
        "🔴 dòng TRỐNG giữa subject và body phải còn — thiếu nó, git coi cả khối là một \
         subject dài và mọi công cụ đọc `%s` sẽ hiện sai. Đọc được: {doc:?}"
    );
}

/// Thông điệp bắt đầu bằng `-` không bị git đọc thành **cờ**.
///
/// Đây là một lý do nữa để dùng `-F -`: với `-m`, một thông điệp `-n` trở thành một cờ
/// của `git commit`, và thông điệp thật biến mất. `message` đến từ webview nên nó là
/// chuỗi tuỳ ý — cùng lớp rủi ro với đường dẫn tên `-rf` của 04-02.
#[tokio::test]
async fn thong_diep_bat_dau_bang_gach_ngang_khong_thanh_co() {
    let (_dir, state, repo, p) = repo_san_sang();

    let msg = "--amend is not a flag here\n";
    tao_commit(&state, Arc::clone(&repo), msg, false)
        .await
        .expect("thông điệp bắt đầu bằng `-` phải commit được, không bị đọc thành cờ");

    let doc = thong_diep_head(&p);
    assert!(
        doc.contains("--amend is not a flag here"),
        "🔴 thông điệp bắt đầu bằng `-` phải vào commit NGUYÊN VĂN. Đọc được: {doc:?}"
    );
    assert_eq!(
        so_commit(&p),
        2,
        "🔴 phải là một commit MỚI, không phải một amend — nếu `--amend` trong thông \
         điệp bị git đọc thành cờ thì số commit vẫn là 1"
    );
}

/// 🔴 **`--cleanup=whitespace` giữ dòng bắt đầu bằng `#`** — PLAT-02.
///
/// # Repo này CỐ Ý đặt `commit.cleanup=strip`, và đó là cả điểm
///
/// Đã đo trên git 2.54.0.windows.1: với `-F -` (thông điệp từ tệp/stdin), mặc định của
/// git **đã là** `whitespace`, nên trên một repo cấu hình mặc định dòng `#` sống sót
/// **dù có cờ hay không**. Một test chạy ở đó **không phân biệt được** đột biến bỏ cờ
/// — nó xanh ở cả hai bên. Đó đúng là lỗi #7 của CONTEXT.md 3.1: hình dạng đúng, dữ
/// liệu **vô hại**.
///
/// Đặt `commit.cleanup=strip` làm dữ liệu trở nên **có hại**: đo được rằng không cờ
/// thì dòng `#789` **mất**, có cờ thì **còn**. Và người dùng đặt `commit.cleanup` trong
/// cấu hình toàn cục là ca thật, không phải giả định.
#[tokio::test]
async fn cleanup_whitespace_giu_dong_hash_ke_ca_khi_cau_hinh_doi_nguoc() {
    let (_dir, state, repo, p) = repo_san_sang();

    // 🔴 Cấu hình làm dữ liệu CÓ HẠI. Thiếu dòng này, test xanh ở cả hai bên đột biến.
    git(&p, &["config", "commit.cleanup", "strip"]);

    let msg = "dong subject\n\n#789 dong nay PHAI con nguyen\n";
    tao_commit(&state, Arc::clone(&repo), msg, false)
        .await
        .expect("commit phải Ok");

    let doc = thong_diep_head(&p);
    assert!(
        doc.contains("#789 dong nay PHAI con nguyen"),
        "🔴 dòng bắt đầu bằng `#` phải CÒN NGUYÊN. Repo này có `commit.cleanup=strip`, \
         nên thiếu cờ `--cleanup=whitespace` thì git XOÁ dòng đó — im lặng, không một \
         lời giải thích. Người dùng gõ `#123` tham chiếu issue thấy nó biến mất và \
         không có cách nào đoán ra nguyên nhân (docs/02-phase4-commit-notes.md).\
         \n\nĐọc được: {doc:?}"
    );
}

/// Không có gì ở index → lỗi **đọc hiểu được**, không phải stderr thô.
///
/// 🔴 Câu này git in ra **stdout**, stderr **rỗng** — đã đo. Một bộ phân loại chỉ đọc
/// stderr thấy chuỗi rỗng và rơi vào nhánh sai.
#[tokio::test]
async fn khong_co_gi_de_commit_cho_loi_doc_hieu_duoc() {
    let (_dir, state, repo, p) = repo_san_sang();

    // Bỏ tệp đã stage ra khỏi index -> không còn gì để commit.
    git(&p, &["restore", "--staged", "--", "them.txt"]);
    std::fs::remove_file(p.join("them.txt")).unwrap();

    let loi = tao_commit(&state, Arc::clone(&repo), "khong co gi", false)
        .await
        .expect_err("không có gì ở index thì commit phải thất bại");

    assert_eq!(
        loi.code(),
        "nothing_to_commit",
        "🔴 ca này phải có mã riêng. Câu 'nothing to commit' nằm ở STDOUT với stderr \
         RỖNG — một bộ phân loại chỉ đọc stderr sẽ không thấy gì và rơi vào nhánh sai. \
         Nhận được: {loi}"
    );
    let thong_bao = loi.to_string();
    assert!(
        thong_bao.contains("stage"),
        "thông báo phải nói VIỆC CẦN LÀM (chọn tệp để stage), không phải mã thoát của \
         git — với người dùng đây không phải một lỗi, họ chỉ chưa chọn tệp: {thong_bao:?}"
    );
}

// ===========================================================================
// amend — WORK-09
// ===========================================================================

/// 🔴 **Đột biến M10: amend SỬA commit gần nhất, không tạo commit mới.**
///
/// Phép phân biệt là **số commit không tăng** trong khi **SHA đổi**. Chỉ một trong hai
/// là không đủ: "SHA đổi" đúng cả với một commit mới, và "số không tăng" đúng cả với
/// một lệnh chẳng làm gì.
#[tokio::test]
async fn amend_doi_sha_nhung_khong_tang_so_commit() {
    let (_dir, state, repo, p) = repo_san_sang();

    // Commit bình thường trước, để có một commit để sửa.
    tao_commit(&state, Arc::clone(&repo), "thong diep goc", false)
        .await
        .expect("commit đầu phải Ok");

    let sha_truoc = head_sha(&p);
    let so_truoc = so_commit(&p);

    let kq = sua_commit_gan_nhat(&state, Arc::clone(&repo), "thong diep DA SUA", false)
        .await
        .expect("amend phải Ok");

    assert_ne!(
        head_sha(&p),
        sha_truoc,
        "amend phải viết lại commit nên SHA phải đổi"
    );
    assert_eq!(
        so_commit(&p),
        so_truoc,
        "🔴 amend KHÔNG được tăng số commit (đột biến M10: bỏ `--amend` thì đây là một \
         commit mới). Đo bằng `rev-list --count`, không bằng `git log` — trần cứng 50 \
         dòng và thất bại im lặng (CONTEXT.md 3.6)"
    );

    let doc = thong_diep_head(&p);
    assert!(
        doc.contains("thong diep DA SUA"),
        "thông điệp mới phải thay thông điệp cũ: {doc:?}"
    );
    assert!(
        !doc.contains("thong diep goc"),
        "thông điệp CŨ không được còn sót: {doc:?}"
    );

    assert!(
        !kq.was_pushed,
        "repo cục bộ không có upstream → was_pushed phải false"
    );
}

/// 🔴 **Đột biến M11: không upstream → `was_pushed == false`.**
///
/// Dòng `# branch.ab` **vắng mặt hoàn toàn** khi không có upstream — git không in
/// `+0 -0` (đo ở 04-01). Mặc định hoá `None` thành `Some(0)` làm cảnh báo nổ ở **mọi**
/// repo cục bộ, và người dùng sẽ học cách bỏ qua nó. Một cảnh báo luôn bật là một cảnh
/// báo đã chết.
#[tokio::test]
async fn amend_khong_upstream_thi_was_pushed_false() {
    let (_dir, state, repo, p) = repo_san_sang();

    tao_commit(&state, Arc::clone(&repo), "goc", false)
        .await
        .expect("commit phải Ok");

    // Tiền đề: repo này thật sự KHÔNG có upstream.
    let st = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status phải Ok");
    assert!(
        st.branch.upstream.is_none(),
        "tiền đề: repo tạm không được có upstream, nếu không test đang đo ca khác"
    );
    assert!(
        st.branch.ahead.is_none(),
        "🔴 tiền đề: không upstream thì `# branch.ab` VẮNG MẶT nên ahead là None — \
         KHÔNG phải Some(0). Đọc được: {:?}",
        st.branch.ahead
    );

    let kq = sua_commit_gan_nhat(&state, Arc::clone(&repo), "sua", false)
        .await
        .expect("amend phải Ok");

    assert!(
        !kq.was_pushed,
        "🔴 chưa từng push → KHÔNG cảnh báo (đột biến M11). `None` ≠ `Some(0)`: \
         'chưa từng push' và 'đã push, đang đồng bộ' là hai trạng thái khác nhau"
    );
    assert_eq!(&p, &p);
}

/// 🔴 **Ca trung tâm của WORK-09: đã push → CẢNH BÁO, nhưng amend VẪN CHẠY.**
///
/// ROADMAP nguyên văn: *cảnh báo, **không chặn***. Test khẳng định **cả hai** vế:
/// `was_pushed == true` (cảnh báo có) **và** SHA đã đổi (amend thật sự chạy).
///
/// Chỉ khẳng định `was_pushed == true` là **không đủ** — nó xanh với một cài đặt chặn
/// hoàn toàn, miễn là cài đặt đó vẫn trả cờ trước khi từ chối. Đột biến M9 đúng là
/// thứ đó.
#[tokio::test]
async fn amend_tren_commit_da_push_canh_bao_nhung_van_chay() {
    let (_dir, state, repo, p) = repo_san_sang();

    // Dựng một upstream thật: clone bare rồi push, để `# branch.ab +0 -0` xuất hiện.
    let bare = _dir.path().join("remote.git");
    git(
        &p,
        &[
            "init",
            "--bare",
            "--initial-branch=main",
            bare.to_str().unwrap(),
        ],
    );
    tao_commit(&state, Arc::clone(&repo), "commit se duoc push", false)
        .await
        .expect("commit phải Ok");
    git(&p, &["remote", "add", "origin", bare.to_str().unwrap()]);
    git(&p, &["push", "-u", "origin", "main"]);

    // Tiền đề: trạng thái phải thật sự là "đã push, đang đồng bộ".
    let st = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status phải Ok");
    assert!(
        st.branch.upstream.is_some(),
        "tiền đề: phải có upstream sau `push -u`. Đọc được: {:?}",
        st.branch
    );
    assert_eq!(
        st.branch.ahead,
        Some(0),
        "🔴 tiền đề: đã push và đồng bộ thì ahead phải là Some(0). Không có tiền đề này \
         thì test dưới xanh vì một lý do khác. Đọc được: {:?}",
        st.branch
    );

    let sha_truoc = head_sha(&p);
    let so_truoc = so_commit(&p);

    let kq = sua_commit_gan_nhat(&state, Arc::clone(&repo), "sua sau khi push", false)
        .await
        .expect(
            "🔴 amend trên commit ĐÃ PUSH phải VẪN CHẠY. Một `Err` ở đây là phép CHẶN, \
             và ROADMAP nói nguyên văn 'cảnh báo, KHÔNG chặn' (WORK-09, đột biến M9). \
             Người dùng biết họ đang làm gì",
        );

    assert!(
        kq.was_pushed,
        "🔴 cảnh báo phải BẬT: upstream có và ahead == Some(0) nghĩa là commit vừa sửa \
         đã nằm trên remote, tức ta vừa viết lại thứ người khác có thể đã thấy"
    );
    assert_ne!(
        head_sha(&p),
        sha_truoc,
        "🔴 amend phải THẬT SỰ CHẠY dù có cảnh báo — SHA phải đổi. Khẳng định này là \
         thứ phân biệt 'cảnh báo' với 'chặn': chỉ kiểm `was_pushed == true` sẽ xanh \
         với một cài đặt từ chối hoàn toàn, miễn là nó vẫn trả cờ trước khi từ chối"
    );
    assert_eq!(so_commit(&p), so_truoc, "amend không tăng số commit");
}

/// Amend cũng chạy hook mặc định, và cũng tôn trọng `no_verify`.
///
/// Một cài đặt thêm `--no-verify` vào `amend` "cho tiện" (vì amend hay chạy lại hook)
/// đi lọt qua mọi test của `create_commit`. Ca này là cổng cho đường đó.
#[tokio::test]
async fn amend_cung_chay_hook_mac_dinh() {
    let Some((_dir, state, repo, p)) = repo_hook("hook-reject") else {
        return;
    };

    // Bỏ tệp đã stage ra: amend không cần nó, và giữ lại làm ca phức tạp hơn cần thiết.
    git(&p, &["restore", "--staged", "--", "them.txt"]);

    let truoc = head_sha(&p);

    let loi = sua_commit_gan_nhat(&state, Arc::clone(&repo), "sua thu", false)
        .await
        .expect_err(
            "🔴 `amend` cũng phải chạy hook MẶC ĐỊNH. Nó thành công nghĩa là đường amend \
             thêm `--no-verify` mà đường create thì không — một đường thứ hai mà mọi \
             test của create_commit không thấy",
        );
    assert!(
        loi.to_string().contains("HOOK-PRE-COMMIT-REJECTED"),
        "nguyên văn đầu ra hook phải tới người dùng ở cả đường amend: {loi}"
    );
    assert_eq!(head_sha(&p), truoc, "hook từ chối thì HEAD không đổi");

    // Cùng repo, no_verify=true -> amend đi qua.
    sua_commit_gan_nhat(&state, Arc::clone(&repo), "sua thu", true)
        .await
        .expect("no_verify=true thì amend phải đi qua hook");
    assert_ne!(head_sha(&p), truoc, "amend với no_verify phải đổi HEAD");
}

// ===========================================================================
// PLAT-03 / index.lock — dùng CHUNG đường của 04-02
// ===========================================================================

/// 🔴 **Đột biến M12: `create_commit` giữ `write_lock` (PLAT-03).**
///
/// Đo bằng cách giữ khoá **trước** rồi khẳng định lời gọi **không kết thúc được**: nếu
/// nó không lấy khoá, nó chạy xong ngay và `timeout` trả `Ok`.
///
/// Vế thứ hai (thả khoá rồi phải chạy được) là bắt buộc — thiếu nó, cổng xanh với một
/// hàm treo **vô điều kiện**, và khi đó nó không phân biệt được gì.
#[tokio::test]
async fn create_commit_giu_write_lock_plat_03() {
    let (_dir, state, repo, p) = repo_san_sang();

    let lock = repo.write_lock();
    let guard = lock.lock().await;

    let ket_qua = tokio::time::timeout(
        std::time::Duration::from_millis(400),
        tao_commit(&state, Arc::clone(&repo), "thu", false),
    )
    .await;

    assert!(
        ket_qua.is_err(),
        "🔴 `create_commit` phải BỊ CHẶN khi khoá ghi đang bị giữ (PLAT-03). Nó chạy \
         xong nghĩa là nó không lấy khoá, và hai lệnh ghi sẽ đụng index.lock của nhau"
    );

    drop(guard);

    let sau = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tao_commit(&state, Arc::clone(&repo), "thu", false),
    )
    .await
    .expect("thả khoá rồi thì phải chạy xong")
    .expect("commit phải Ok");
    assert!(
        !sau.entries.iter().any(|e| e.path == "them.txt"),
        "commit xong thì tệp không còn ở nhóm đã stage"
    );
    assert_eq!(so_commit(&p), 2, "đúng một commit được thêm");
}

/// `amend_commit` cũng giữ khoá ghi.
#[tokio::test]
async fn amend_commit_giu_write_lock_plat_03() {
    let (_dir, state, repo, _p) = repo_san_sang();

    let lock = repo.write_lock();
    let guard = lock.lock().await;

    let ket_qua = tokio::time::timeout(
        std::time::Duration::from_millis(400),
        sua_commit_gan_nhat(&state, Arc::clone(&repo), "thu", false),
    )
    .await;

    assert!(
        ket_qua.is_err(),
        "🔴 `amend_commit` cũng phải giữ khoá ghi (PLAT-03)"
    );

    drop(guard);

    tokio::time::timeout(
        std::time::Duration::from_secs(30),
        sua_commit_gan_nhat(&state, Arc::clone(&repo), "thu", false),
    )
    .await
    .expect("thả khoá rồi thì phải chạy xong")
    .expect("amend phải Ok");
}

/// 🔴 **`.git/index.lock` tồn tại → thất bại, và tệp lock VẪN CÒN.**
///
/// Dùng **cùng** đường thử-lại của 04-02, không viết lại. Xoá `index.lock` là phá repo
/// của người dùng đang `git rebase` ở terminal (CONTEXT.md 2.3, R3) — và chủ dự án
/// **dùng terminal song song**, đó là cả điểm của WORK-10.
#[tokio::test]
async fn index_lock_ton_tai_thi_commit_that_bai_va_lock_van_con() {
    let (_dir, state, repo, p) = repo_san_sang();

    let lock = p.join(".git").join("index.lock");
    std::fs::write(&lock, b"").expect("dựng được index.lock giả");
    assert!(lock.is_file(), "tiền đề: index.lock phải tồn tại");

    let truoc = head_sha(&p);
    let loi = tao_commit(&state, Arc::clone(&repo), "thu commit", false)
        .await
        .expect_err("index.lock tồn tại thì commit phải thất bại");

    assert!(
        lock.is_file(),
        "🔴 `index.lock` phải VẪN CÒN sau khi thất bại. Xoá nó không 'giải phóng' gì — \
         nó VÔ HIỆU HOÁ chính cơ chế loại trừ của git, và tiến trình kia sẽ ghi index \
         dựa trên trạng thái đã cũ (CONTEXT.md 2.3, R3)"
    );
    assert_eq!(head_sha(&p), truoc, "không commit nào được tạo");
    assert!(
        matches!(loi, GitError::IndexLocked { .. }),
        "ca lock phải giữ mã riêng ở đường commit, giống đường stage của 04-02. \
         Nhận được: {loi} (mã {})",
        loi.code()
    );

    // Phải có THỬ LẠI, không phải một lần chạy rồi bỏ.
    let so_lan = state
        .command_log
        .entries()
        .iter()
        .filter(|e| e.command.contains("commit"))
        .count();
    assert!(
        so_lan >= 2,
        "phải THỬ LẠI khi gặp index.lock (dùng chung đường của 04-02), thấy {so_lan} lần"
    );
}

/// Commit trả [`RepoStatus`] **mới** trực tiếp — CONTEXT.md 2.5.
///
/// Đọc **giá trị trả về**, không gọi lại `lay_trang_thai`: gọi lại sẽ xanh kể cả khi
/// hàm trả một trạng thái cũ, tức cổng sẽ không phân biệt được gì.
#[tokio::test]
async fn commit_tra_trang_thai_moi_truc_tiep() {
    let (_dir, state, repo, _p) = repo_san_sang();

    let truoc = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status ban đầu phải Ok");
    assert!(
        truoc.entries.iter().any(|e| e.path == "them.txt"),
        "tiền đề: them.txt phải đang ở index trước khi commit"
    );

    let sau = tao_commit(&state, Arc::clone(&repo), "commit thu", false)
        .await
        .expect("commit phải Ok");

    assert!(
        !sau.entries.iter().any(|e| e.path == "them.txt"),
        "🔴 giá trị TRẢ VỀ của create_commit phải là trạng thái MỚI — tệp vừa commit \
         không còn ở index trong chính giá trị đó, không phải sau một lần đọc lại. \
         Trả trạng thái cũ buộc giao diện chờ watcher, và nếu watcher chết thì giao \
         diện đứng im mà không ai biết (CONTEXT.md 2.5). Đọc được: {:?}",
        sau.entries
    );
}
