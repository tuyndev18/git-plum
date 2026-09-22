//! Test tích hợp cho `get_worktree_diff` — plan 04-02, WORK-01 tiêu chí 2.
//!
//! # 🔴 Thứ được kiểm quan trọng nhất ở đây: diff thư mục làm việc KHÔNG vào cache
//!
//! Cache diff của Phase 3 đúng vì diff của một commit **lịch sử** là bất biến theo
//! `(sha, path)`. Diff của thư mục làm việc đổi **mỗi lần người dùng gõ**. Một
//! `put_diff` vô tình ở đây cho người dùng nội dung cũ, và Phase 4 là phase **đầu
//! tiên** mà dữ liệu cũ **nguy hiểm** chứ không chỉ sai mắt nhìn (CONTEXT.md 2.2):
//! trước Phase 4 ứng dụng chỉ đọc; từ Phase 4 nó **ghi**, và ghi dựa trên trạng thái
//! cũ thì hỏng repo của người dùng.
//!
//! Cổng đo bằng **hai** cách, vì một cách không đủ:
//! 1. `diff_cache.len()` không tăng sau lời gọi — bắt `put_diff`;
//! 2. sửa tệp rồi gọi lại → nội dung **mới** — bắt cả `get_diff` (một cài đặt chỉ đọc
//!    cache mà không ghi vẫn trả nội dung cũ nếu mục đó do đường khác đặt vào).

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use git_plum_lib::commands::diff::lay_diff_thu_muc_lam_viec;
use git_plum_lib::domain::diff::DiffKind;
use git_plum_lib::state::{AppState, RepoHandle};

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

/// Repo có một commit gốc và một tệp sửa **chưa** stage.
fn repo_co_tep_sua() -> (tempfile::TempDir, AppState, Arc<RepoHandle>) {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    git(p, &["config", "user.name", "Test"]);
    git(p, &["config", "user.email", "test@example.com"]);
    git(p, &["config", "core.autocrlf", "false"]);

    std::fs::write(p.join("a.txt"), "dong mot\ndong hai\ndong ba\n").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "commit goc"]);

    std::fs::write(p.join("a.txt"), "dong mot\nDA SUA\ndong ba\n").unwrap();

    let state = AppState::new();
    let repo = state.open_repo(p);
    (dir, state, repo)
}

/// Mọi dòng của mọi hunk, kèm loại — để khẳng định nội dung gọn.
fn cac_dong(kind: &DiffKind) -> Vec<String> {
    match kind {
        DiffKind::Text { hunks, .. } => hunks
            .iter()
            .flat_map(|h| &h.lines)
            .map(|l| l.content.clone())
            .collect(),
        khac => panic!("mong DiffKind::Text, nhận được {khac:?}"),
    }
}

/// `staged = false` → diff của **thư mục làm việc** so với index.
#[tokio::test]
async fn khong_staged_cho_diff_cua_thu_muc_lam_viec() {
    let (_dir, state, repo) = repo_co_tep_sua();

    let fd = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", false)
        .await
        .expect("diff thư mục làm việc phải Ok");

    let dong = cac_dong(&fd.kind);
    assert!(
        dong.iter().any(|d| d.contains("DA SUA")),
        "diff chưa stage phải chứa dòng vừa sửa. Đọc được: {dong:?}"
    );
    assert_eq!(fd.path, "a.txt");
}

/// `staged = true` → `git diff --cached`, tức chỉ thấy thứ **đã** vào index.
#[tokio::test]
async fn staged_cho_diff_cua_index_khong_cua_thu_muc_lam_viec() {
    let (dir, state, repo) = repo_co_tep_sua();

    // Trước khi `git add`: index sạch, nên diff `--cached` phải KHÔNG có dòng sửa.
    let truoc = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", true)
        .await
        .expect("diff --cached phải Ok kể cả khi index sạch");
    assert!(
        matches!(truoc.kind, DiffKind::Unchanged),
        "🔴 index chưa có gì thì diff --cached phải là Unchanged, không phải diff của \
         thư mục làm việc. Nhận được: {:?}",
        truoc.kind
    );

    git(dir.path(), &["add", "a.txt"]);

    let sau = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", true)
        .await
        .expect("diff --cached sau khi add phải Ok");
    let dong = cac_dong(&sau.kind);
    assert!(
        dong.iter().any(|d| d.contains("DA SUA")),
        "sau `git add`, diff --cached phải chứa dòng vừa sửa. Đọc được: {dong:?}"
    );
}

/// 🔴 **M10: diff thư mục làm việc KHÔNG được ghi vào `DiffCache`.**
///
/// Đo bằng `diff_cache.len()`: một `put_diff` làm số này tăng. Giá trị trả về
/// **không** phân biệt được — một cài đặt có cache vẫn trả đúng diff ở lời gọi đầu.
#[tokio::test]
async fn diff_thu_muc_lam_viec_khong_vao_cache() {
    let (_dir, state, repo) = repo_co_tep_sua();

    assert_eq!(
        state.diff_cache.len(),
        0,
        "tiền đề: cache phải rỗng lúc bắt đầu"
    );

    lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", false)
        .await
        .expect("diff phải Ok");

    assert_eq!(
        state.diff_cache.len(),
        0,
        "🔴 diff của THƯ MỤC LÀM VIỆC không được vào `DiffCache`. Cache của Phase 3 \
         đúng vì diff của commit lịch sử là BẤT BIẾN; diff thư mục làm việc đổi mỗi \
         lần người dùng gõ. Một mục cache ở đây cho người dùng nội dung CŨ, và đây là \
         phase đầu tiên mà dữ liệu cũ nguy hiểm chứ không chỉ sai mắt nhìn \
         (CONTEXT.md 2.2)."
    );

    // Gọi thêm hai lần nữa: nếu cache được ghi ở một nhánh khác (ví dụ chỉ ở đường
    // `Unchanged` hay `Binary`), số vẫn phải là 0.
    lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", true)
        .await
        .expect("diff --cached phải Ok");
    lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "khong-co.txt", false)
        .await
        .ok();
    assert_eq!(
        state.diff_cache.len(),
        0,
        "không nhánh nào của đường này được ghi cache"
    );
}

/// 🔴 **Sửa tệp rồi gọi lại phải cho nội dung MỚI.**
///
/// Đây là nửa thứ hai của cổng M10, và nó bắt một thứ khác: một cài đặt **đọc** cache
/// (`get_diff`) mà không ghi vẫn trả nội dung cũ nếu mục đó do đường lịch sử đặt vào
/// cùng khoá. Đây đúng là ca người dùng gặp: gõ, xem diff, gõ tiếp, xem lại.
#[tokio::test]
async fn sua_tep_roi_goi_lai_cho_noi_dung_moi() {
    let (dir, state, repo) = repo_co_tep_sua();

    let lan_dau = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", false)
        .await
        .expect("lần đầu phải Ok");
    assert!(
        cac_dong(&lan_dau.kind).iter().any(|d| d.contains("DA SUA")),
        "tiền đề: lần đầu phải thấy nội dung lần đầu"
    );

    std::fs::write(
        dir.path().join("a.txt"),
        "dong mot\nSUA LAN HAI HOAN TOAN KHAC\ndong ba\n",
    )
    .unwrap();

    let lan_hai = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "a.txt", false)
        .await
        .expect("lần hai phải Ok");
    let dong = cac_dong(&lan_hai.kind);

    assert!(
        dong.iter().any(|d| d.contains("SUA LAN HAI")),
        "🔴 gọi lại sau khi sửa tệp phải cho nội dung MỚI. Đọc được: {dong:?}"
    );
    assert!(
        !dong.iter().any(|d| d.contains("DA SUA")),
        "🔴 nội dung CŨ không được còn — đó là dấu hiệu đường này đọc cache. \
         Đọc được: {dong:?}"
    );
}

/// Tệp chưa theo dõi: `git diff` không thấy nó, nên kết quả là `Unchanged`.
///
/// Đây là một **giới hạn đã biết**, không phải lỗi, và nó được ghi lại chứ không lấp:
/// `git diff` chỉ so index với cây làm việc cho tệp **đã theo dõi**. Giao diện phải
/// nói được điều đó thay vì hiện một trình xem trống không lời giải thích.
#[tokio::test]
async fn tep_chua_theo_doi_cho_unchanged_khong_phai_loi() {
    let (dir, state, repo) = repo_co_tep_sua();
    std::fs::write(dir.path().join("moi.txt"), "toi chua theo doi\n").unwrap();

    let fd = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "moi.txt", false)
        .await
        .expect("tệp chưa theo dõi KHÔNG được cho lỗi — nó là ca bình thường");

    assert!(
        matches!(fd.kind, DiffKind::Unchanged),
        "tệp chưa theo dõi cho Unchanged (git diff không thấy nó). Nhận được: {:?}",
        fd.kind
    );
}

/// 🔴 **Đường dẫn trùng tên nhánh không bị đọc thành revision.**
///
/// Cùng lý do `--` phải có ở `stage_files`. Ở đây nặng hơn: `git diff main` là một
/// lệnh **hợp lệ** so HEAD với nhánh `main`, nên thiếu `--` không báo lỗi — nó trả
/// một diff **sai** trong im lặng.
#[tokio::test]
async fn duong_dan_trung_ten_nhanh_khong_thanh_revision() {
    let (dir, state, repo) = repo_co_tep_sua();

    std::fs::write(dir.path().join("main"), "goc\n").unwrap();
    git(dir.path(), &["add", "main"]);
    git(dir.path(), &["commit", "-m", "them tep ten main"]);
    std::fs::write(dir.path().join("main"), "da sua tep ten main\n").unwrap();

    let fd = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "main", false)
        .await
        .expect("tệp tên `main` phải diff được");

    let dong = cac_dong(&fd.kind);
    assert!(
        dong.iter().any(|d| d.contains("da sua tep ten main")),
        "🔴 `main` phải được xử lý như ĐƯỜNG DẪN. `git diff main` không có `--` là một \
         lệnh HỢP LỆ so với nhánh main, nên thiếu `--` cho một diff sai trong im lặng. \
         Đọc được: {dong:?}"
    );
    assert_eq!(fd.path, "main");
}

/// Tệp nhị phân đi qua cổng nhị phân **đã có**, không phải một phép đoán mới.
#[tokio::test]
async fn tep_nhi_phan_cho_kind_binary() {
    let (dir, state, repo) = repo_co_tep_sua();

    // Byte 0 ở giữa làm git phán quyết "binary" — dùng phán quyết của chính git.
    std::fs::write(dir.path().join("b.bin"), [0u8, 1, 2, 0, 255, 254]).unwrap();
    git(dir.path(), &["add", "b.bin"]);
    git(dir.path(), &["commit", "-m", "them nhi phan"]);
    std::fs::write(dir.path().join("b.bin"), [0u8, 9, 9, 0, 1, 2, 3]).unwrap();

    let fd = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "b.bin", false)
        .await
        .expect("tệp nhị phân phải Ok, không phải lỗi");

    assert!(
        matches!(fd.kind, DiffKind::Binary { .. }),
        "tệp nhị phân phải cho DiffKind::Binary — tái dùng cổng đã có, không viết \
         lại phép đoán. Nhận được: {:?}",
        fd.kind
    );
}

/// Cổng kích thước 5 MB vẫn áp cho đường thư mục làm việc.
///
/// Không tái dùng cổng nghĩa là một tệp 200 MB bẩn sẽ được nạp hết vào RAM — đúng ca
/// mà DIFF-06 tồn tại để chặn, và người dùng gặp nó ở thư mục làm việc **dễ hơn** ở
/// lịch sử (một tệp log, một dump cơ sở dữ liệu).
#[tokio::test]
async fn tep_vuot_nguong_cho_too_large_khong_nap_noi_dung() {
    let (dir, state, repo) = repo_co_tep_sua();

    let to = "x".repeat(6 * 1024 * 1024);
    std::fs::write(dir.path().join("to.txt"), "nho\n").unwrap();
    git(dir.path(), &["add", "to.txt"]);
    git(dir.path(), &["commit", "-m", "them tep nho"]);
    std::fs::write(dir.path().join("to.txt"), &to).unwrap();

    let fd = lay_diff_thu_muc_lam_viec(&state, Arc::clone(&repo), "to.txt", false)
        .await
        .expect("tệp quá lớn phải Ok với kind tooLarge, không phải lỗi");

    assert!(
        matches!(fd.kind, DiffKind::TooLarge { .. }),
        "tệp 6 MB phải cho DiffKind::TooLarge (ngưỡng 5 MB). Nhận được: {:?}",
        fd.kind
    );
}
