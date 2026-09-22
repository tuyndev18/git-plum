//! Test tích hợp cho `get_status` / `stage_files` / `unstage_files` — plan 04-02,
//! WORK-01 và WORK-02.
//!
//! # Vì sao chạy git THẬT
//!
//! `parse_status` đã có 26 test đơn vị trên byte tự dựng (plan 04-01), và ở đó buffer
//! tự dựng là **đúng** vì thứ được kiểm là một hàm thuần. Ở đây thứ được kiểm là
//! **git có trả về điều ta tưởng không**, cộng với việc lệnh ghi có thật sự đổi
//! index hay không — và chỉ git thật trả lời được cả hai.
//!
//! Bài học `%x1f` của 02-04: 19 test đơn vị xanh trên buffer tự dựng trong khi lệnh
//! git thật trả về định dạng khác, và thanh bên rỗng hoàn toàn trong im lặng.
//!
//! # Mỗi test dựng repo RIÊNG, không dùng `status-cases`
//!
//! `status-cases` là fixture **chỉ đọc**: giá trị của nó nằm ở thư mục làm việc bẩn,
//! và `require_status_fixture` ghi rõ rằng chạy `git commit`/`git reset` trong đó là
//! **phá fixture**. Test của `stage_files` **phải** ghi vào index, nên nó dựng repo
//! tạm riêng qua `tempfile`. Chỉ test `get_status` (đọc thuần) dùng fixture.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use git_plum_lib::commands::worktree::{lay_trang_thai, stage_duong_dan, unstage_duong_dan};
use git_plum_lib::domain::StatusGroup;
use git_plum_lib::error::GitError;
use git_plum_lib::state::{AppState, RepoHandle};
use git_plum_lib::testing::require_status_fixture;

/// Chạy một lệnh git đồng bộ trong lúc dựng repo. Không đi qua `GitRunner` có chủ ý:
/// đây là phần **dựng bối cảnh**, không phải phần được kiểm.
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

/// Repo tạm có một commit gốc, cộng ba tệp bẩn: một sửa đã stage, một sửa chưa stage,
/// một chưa theo dõi.
fn repo_ban() -> (tempfile::TempDir, AppState, Arc<RepoHandle>) {
    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    git(p, &["config", "user.name", "Test"]);
    git(p, &["config", "user.email", "test@example.com"]);
    // `core.autocrlf=false`: một chuyển đổi CRLF làm `git status` báo tệp sửa mà
    // test không mong.
    git(p, &["config", "core.autocrlf", "false"]);

    std::fs::write(p.join("da_stage.txt"), "goc\n").unwrap();
    std::fs::write(p.join("chua_stage.txt"), "goc\n").unwrap();
    git(p, &["add", "da_stage.txt", "chua_stage.txt"]);
    git(p, &["commit", "-m", "commit goc"]);

    std::fs::write(p.join("da_stage.txt"), "doi roi\n").unwrap();
    git(p, &["add", "da_stage.txt"]);
    std::fs::write(p.join("chua_stage.txt"), "doi roi\n").unwrap();
    std::fs::write(p.join("chua_theo_doi.txt"), "moi\n").unwrap();

    let state = AppState::new();
    let repo = state.open_repo(p);
    (dir, state, repo)
}

/// Các nhóm mà `path` thuộc về trong một status.
fn nhom_cua(st: &git_plum_lib::domain::RepoStatus, path: &str) -> Vec<StatusGroup> {
    st.entries
        .iter()
        .filter(|e| e.path == path)
        .map(|e| e.group)
        .collect()
}

/// `get_status` trên repo mẫu `status-cases` phải cho **cả ba** nhóm.
///
/// Đây là ca đi qua `STATUS_ARGS` thật, `git/exec.rs` thật, và `parse_status` thật —
/// tức nó bắt được cả lớp lỗi "cờ đúng nhưng lệnh không chạy" mà test đơn vị không thấy.
#[tokio::test]
async fn get_status_tren_fixture_cho_ca_ba_nhom() {
    let Some(repo_path) = require_status_fixture() else {
        return;
    };
    let state = AppState::new();
    let repo = state.open_repo(&repo_path);

    let st = lay_trang_thai(&state, repo).await.expect("status phải Ok");

    let nhom: std::collections::HashSet<StatusGroup> = st.entries.iter().map(|e| e.group).collect();

    assert!(
        nhom.contains(&StatusGroup::Staged),
        "fixture phải có nhóm đã stage, đọc được: {:?}",
        st.entries
    );
    assert!(
        nhom.contains(&StatusGroup::Unstaged),
        "fixture phải có nhóm chưa stage, đọc được: {:?}",
        st.entries
    );
    assert!(
        nhom.contains(&StatusGroup::Untracked),
        "fixture phải có nhóm chưa theo dõi, đọc được: {:?}",
        st.entries
    );

    // Tệp đổi tên của fixture (bản ghi dạng `2`) phải mang `old_path` — nếu lệnh đi
    // qua `--porcelain=v1` thì trường này luôn `None` và test này đỏ.
    assert!(
        st.entries.iter().any(|e| e.old_path.is_some()),
        "fixture có một tệp đổi tên; `old_path` rỗng hết nghĩa là không phải v2"
    );
}

/// 🔴 **M1: `stage_files` trả `RepoStatus` MỚI, không trả `()`.**
///
/// Ràng buộc 2.5 của CONTEXT.md: thao tác ghi trả trạng thái mới **trực tiếp**, không
/// chờ watcher. Kiểu trả về là cách bảo đảm nó — một command trả `()` **buộc** giao
/// diện phải chờ watcher, và nếu watcher chết thì giao diện đứng im mà không ai biết.
///
/// Test này đọc **giá trị trả về**, không gọi lại `lay_trang_thai`: gọi lại sẽ xanh
/// kể cả khi hàm trả `()`, tức cổng sẽ không phân biệt được đột biến M1.
#[tokio::test]
async fn stage_tra_repostatus_moi_voi_tep_da_chuyen_nhom() {
    let (_dir, state, repo) = repo_ban();

    let truoc = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status ban đầu phải Ok");
    assert_eq!(
        nhom_cua(&truoc, "chua_stage.txt"),
        vec![StatusGroup::Unstaged],
        "tiền đề: chua_stage.txt phải ở nhóm chưa stage trước khi stage"
    );

    let sau = stage_duong_dan(&state, Arc::clone(&repo), &["chua_stage.txt".to_owned()])
        .await
        .expect("stage phải Ok");

    assert_eq!(
        nhom_cua(&sau, "chua_stage.txt"),
        vec![StatusGroup::Staged],
        "🔴 giá trị TRẢ VỀ của stage_files phải là trạng thái MỚI — tệp vừa stage \
         phải nằm ở nhóm Staged trong chính giá trị đó, không phải sau một lần đọc lại"
    );
}

/// `unstage_files` trả `RepoStatus` mới, tệp về nhóm chưa stage.
#[tokio::test]
async fn unstage_tra_repostatus_moi_voi_tep_ve_nhom_chua_stage() {
    let (_dir, state, repo) = repo_ban();

    let truoc = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status ban đầu phải Ok");
    assert_eq!(
        nhom_cua(&truoc, "da_stage.txt"),
        vec![StatusGroup::Staged],
        "tiền đề: da_stage.txt phải ở nhóm đã stage"
    );

    let sau = unstage_duong_dan(&state, Arc::clone(&repo), &["da_stage.txt".to_owned()])
        .await
        .expect("unstage phải Ok");

    assert_eq!(
        nhom_cua(&sau, "da_stage.txt"),
        vec![StatusGroup::Unstaged],
        "giá trị trả về của unstage_files phải cho tệp ở nhóm chưa stage"
    );
}

/// 🔴 **M4: `paths` rỗng KHÔNG sinh lệnh git nào.**
///
/// `git add` không pathspec stage **cả cây** — một lỗi phá hoại, và mảng rỗng đến từ
/// webview nên nó là đầu vào có thật.
///
/// Cổng đo bằng **`CommandLog`**, không chỉ bằng giá trị trả về: chỉ nhật ký lệnh
/// phân biệt được "không chạy" với "chạy rồi thất bại".
#[tokio::test]
async fn paths_rong_khong_sinh_lenh_git_add_nao() {
    let (_dir, state, repo) = repo_ban();

    let truoc = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status phải Ok");
    state.command_log.clear();

    let sau = stage_duong_dan(&state, Arc::clone(&repo), &[])
        .await
        .expect("paths rỗng không phải lỗi — nó là không-việc-gì");

    let lenh: Vec<String> = state
        .command_log
        .entries()
        .iter()
        .map(|e| e.command.clone())
        .collect();

    assert!(
        !lenh.iter().any(|c| c.contains(" add")),
        "🔴 paths rỗng phải KHÔNG sinh lệnh `git add` nào — `git add` không pathspec \
         stage CẢ CÂY. Lệnh đã chạy: {lenh:?}"
    );
    assert!(
        !lenh.iter().any(|c| c.contains("restore")),
        "paths rỗng cũng không được sinh `git restore`. Lệnh đã chạy: {lenh:?}"
    );
    assert_eq!(
        sau.entries.len(),
        truoc.entries.len(),
        "paths rỗng phải trả trạng thái HIỆN TẠI, không đổi gì"
    );

    assert_eq!(
        nhom_cua(&sau, "chua_theo_doi.txt"),
        vec![StatusGroup::Untracked],
        "🔴 tệp chưa theo dõi vẫn phải chưa theo dõi — nếu nó thành Staged thì \
         `git add` đã chạy không pathspec và stage cả cây"
    );
}

/// `unstage_files` với `paths` rỗng cũng không sinh lệnh nào.
#[tokio::test]
async fn unstage_paths_rong_khong_sinh_lenh_nao() {
    let (_dir, state, repo) = repo_ban();
    state.command_log.clear();

    let sau = unstage_duong_dan(&state, Arc::clone(&repo), &[])
        .await
        .expect("paths rỗng không phải lỗi");

    let lenh: Vec<String> = state
        .command_log
        .entries()
        .iter()
        .map(|e| e.command.clone())
        .collect();
    assert!(
        !lenh.iter().any(|c| c.contains("restore")),
        "paths rỗng phải không sinh `git restore`. Lệnh đã chạy: {lenh:?}"
    );
    assert_eq!(
        nhom_cua(&sau, "da_stage.txt"),
        vec![StatusGroup::Staged],
        "tệp đã stage phải giữ nguyên khi paths rỗng"
    );
}

/// 🔴 **Đường dẫn bắt đầu bằng `-` không bị git đọc thành cờ.**
///
/// Đây là lý do `--` phải có. `path` đến từ webview, tức chuỗi tuỳ ý.
#[tokio::test]
async fn duong_dan_bat_dau_bang_gach_ngang_khong_thanh_co() {
    let (dir, state, repo) = repo_ban();
    std::fs::write(dir.path().join("-rf"), "noi dung\n").unwrap();

    let sau = stage_duong_dan(&state, Arc::clone(&repo), &["-rf".to_owned()])
        .await
        .expect("tệp tên `-rf` phải stage được, không bị đọc thành cờ");

    assert_eq!(
        nhom_cua(&sau, "-rf"),
        vec![StatusGroup::Staged],
        "🔴 tệp tên `-rf` phải vào nhóm đã stage. Thiếu `--` thì git đọc nó thành cờ \
         và báo lỗi tham số. Đọc được: {:?}",
        sau.entries
    );
}

/// 🔴 **Đường dẫn trùng tên nhánh (`main`) xử lý như đường dẫn, không như revision.**
#[tokio::test]
async fn duong_dan_trung_ten_nhanh_xu_ly_nhu_duong_dan() {
    let (dir, state, repo) = repo_ban();
    std::fs::write(dir.path().join("main"), "toi la mot tep\n").unwrap();

    let sau = stage_duong_dan(&state, Arc::clone(&repo), &["main".to_owned()])
        .await
        .expect("tệp tên `main` phải stage được dù `main` cũng là tên nhánh");

    assert_eq!(
        nhom_cua(&sau, "main"),
        vec![StatusGroup::Staged],
        "🔴 tệp tên `main` phải được xử lý như ĐƯỜNG DẪN. Đọc được: {:?}",
        sau.entries
    );
}

/// 🔴 **M6: đường xử lý `index.lock` KHÔNG xoá tệp lock.**
///
/// Xoá nó là làm hỏng repo của người dùng đang `git rebase` ở terminal (CONTEXT.md
/// 2.3, R3). Chủ dự án **dùng terminal song song** — đó là cả điểm của WORK-10.
///
/// Test khẳng định **ba** điều, và cả ba đều cần:
/// 1. lệnh thất bại (không âm thầm coi như thành công);
/// 2. tệp `index.lock` **vẫn còn** trên đĩa sau khi thất bại;
/// 3. mã lỗi là mã **riêng** của ca lock, không phải `command_failed` chung.
#[tokio::test]
async fn index_lock_ton_tai_thi_that_bai_va_lock_van_con() {
    let (dir, state, repo) = repo_ban();
    let lock = dir.path().join(".git").join("index.lock");
    std::fs::write(&lock, b"").expect("dựng được index.lock giả");
    assert!(lock.is_file(), "tiền đề: index.lock phải tồn tại");

    let ket_qua = stage_duong_dan(&state, Arc::clone(&repo), &["chua_stage.txt".to_owned()]).await;

    let loi = ket_qua.expect_err("index.lock tồn tại thì stage phải thất bại");

    assert!(
        lock.is_file(),
        "🔴 `index.lock` phải VẪN CÒN sau khi thất bại. Xoá nó là phá repo của \
         người dùng đang chạy `git rebase` ở terminal (CONTEXT.md 2.3, R3)."
    );

    assert_eq!(
        loi.code(),
        "index_locked",
        "ca lock phải có mã lỗi RIÊNG, không phải `command_failed` chung — giao diện \
         cần nói 'đang chờ một tiến trình git khác'. Nhận được lỗi: {loi}"
    );
    assert!(
        matches!(loi, GitError::IndexLocked { .. }),
        "phải là variant IndexLocked"
    );

    let thong_bao = loi.to_string();
    assert!(
        thong_bao.contains("git"),
        "thông báo phải nói người dùng có thể đang chạy git ở nơi khác: {thong_bao:?}"
    );
}

/// Đường xử lý lock **thử lại** trước khi bỏ, và có **chặn trên** thời gian chờ.
///
/// Không được treo giao diện: nếu người dùng đang rebase thật thì chờ lâu cũng không
/// giúp gì.
#[tokio::test]
async fn index_lock_thu_lai_nhieu_lan_nhung_khong_treo() {
    let (dir, state, repo) = repo_ban();
    std::fs::write(dir.path().join(".git").join("index.lock"), b"").unwrap();

    let bat_dau = std::time::Instant::now();
    let ket_qua = stage_duong_dan(&state, Arc::clone(&repo), &["chua_stage.txt".to_owned()]).await;
    let troi_qua = bat_dau.elapsed();

    assert!(
        ket_qua.is_err(),
        "lock giữ nguyên thì cuối cùng phải thất bại"
    );

    // Phải có thử lại: một lần chạy duy nhất nghĩa là không có giãn cách.
    let so_lan_add = state
        .command_log
        .entries()
        .iter()
        .filter(|e| e.command.contains(" add"))
        .count();
    assert!(
        so_lan_add >= 2,
        "phải THỬ LẠI khi gặp index.lock, thấy {so_lan_add} lần chạy `git add`"
    );

    assert!(
        troi_qua < std::time::Duration::from_secs(5),
        "tổng thời gian chờ phải có chặn trên — giao diện không được treo. Mất {troi_qua:?}"
    );
}

/// Lệnh ghi phải giữ `write_lock` (PLAT-03) — hai lệnh ghi không chồng nhau.
///
/// 🔴 **M5.** Đo bằng cách giữ khoá **trước** rồi khẳng định lời gọi không kết thúc
/// được: nếu `stage_files` không lấy khoá, nó chạy xong ngay và `timeout` trả `Ok`.
#[tokio::test]
async fn stage_giu_write_lock_plat_03() {
    let (_dir, state, repo) = repo_ban();

    let lock = repo.write_lock();
    let guard = lock.lock().await;

    let ket_qua = tokio::time::timeout(
        std::time::Duration::from_millis(400),
        stage_duong_dan(&state, Arc::clone(&repo), &["chua_stage.txt".to_owned()]),
    )
    .await;

    assert!(
        ket_qua.is_err(),
        "🔴 `stage_files` phải BỊ CHẶN khi khoá ghi đang bị giữ (PLAT-03). Nó chạy \
         xong nghĩa là nó không lấy khoá, và hai lệnh ghi sẽ đụng index.lock của nhau."
    );

    drop(guard);

    // Thả khoá xong thì nó phải chạy được — nếu không thì test trên xanh vì một lý do
    // khác (ví dụ hàm treo vô điều kiện), và cổng sẽ không phân biệt được gì.
    let sau = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        stage_duong_dan(&state, Arc::clone(&repo), &["chua_stage.txt".to_owned()]),
    )
    .await
    .expect("thả khoá rồi thì phải chạy xong")
    .expect("stage phải Ok");
    assert_eq!(nhom_cua(&sau, "chua_stage.txt"), vec![StatusGroup::Staged]);
}

/// `unstage_files` cũng phải giữ khoá ghi.
#[tokio::test]
async fn unstage_giu_write_lock_plat_03() {
    let (_dir, state, repo) = repo_ban();

    let lock = repo.write_lock();
    let guard = lock.lock().await;

    let ket_qua = tokio::time::timeout(
        std::time::Duration::from_millis(400),
        unstage_duong_dan(&state, Arc::clone(&repo), &["da_stage.txt".to_owned()]),
    )
    .await;
    assert!(
        ket_qua.is_err(),
        "`unstage_files` phải bị chặn khi khoá ghi đang bị giữ (PLAT-03)"
    );
    drop(guard);
}

/// `get_status` là lệnh **đọc** — nó KHÔNG được giữ khoá ghi.
///
/// Giữ khoá ở đường đọc làm mọi lần làm mới giao diện xếp hàng sau một lệnh ghi đang
/// chạy, và watcher của 04-04 sẽ đọc trạng thái trong lúc một lệnh ghi còn giữ khoá.
#[tokio::test]
async fn get_status_khong_giu_write_lock() {
    let (_dir, state, repo) = repo_ban();

    let lock = repo.write_lock();
    let _guard = lock.lock().await;

    let st = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        lay_trang_thai(&state, Arc::clone(&repo)),
    )
    .await
    .expect("get_status phải chạy được KHI khoá ghi đang bị giữ — nó là lệnh đọc")
    .expect("status phải Ok");

    assert!(!st.entries.is_empty(), "repo bẩn phải có phần tử");
}

/// Thư mục **không phải** repository git → lỗi phân loại được, kèm stderr nguyên văn.
///
/// 🔴 Không lặp lại khuôn của `open_repository`, vốn báo sai "Không phải một
/// repository git" khi git từ chối repo vì `dubious ownership` (exit 128). Ở đây
/// phân loại theo **stderr** và **luôn** kèm stderr nguyên văn.
#[tokio::test]
async fn khong_phai_repo_thi_loi_co_kem_stderr_nguyen_van() {
    let dir = tempfile::tempdir().unwrap();
    let state = AppState::new();
    let repo = state.open_repo(dir.path());

    let loi = lay_trang_thai(&state, repo)
        .await
        .expect_err("thư mục không phải repo git phải cho lỗi");

    let thong_bao = loi.to_string();
    assert!(
        thong_bao.to_lowercase().contains("repository") || thong_bao.to_lowercase().contains("repo"),
        "thông báo phải nhắc tới repository. Nhận được: {thong_bao:?}"
    );
    assert_ne!(
        loi.code(),
        "index_locked",
        "thư mục không phải repo KHÔNG được phân loại thành ca lock — đó đúng là \
         lớp lỗi 'exit khác 0 có nhiều hơn một nguyên nhân' của open_repository"
    );
}

/// Một tệp `MM` (sửa cả hai phía) hiện ở **cả hai** nhóm, và stage nó cho `M.`.
#[tokio::test]
async fn tep_mm_o_ca_hai_nhom_va_stage_duoc() {
    let (dir, state, repo) = repo_ban();
    // `da_stage.txt` đã `M.`; sửa tiếp ở worktree để thành `MM`.
    std::fs::write(dir.path().join("da_stage.txt"), "doi lan hai\n").unwrap();

    let truoc = lay_trang_thai(&state, Arc::clone(&repo))
        .await
        .expect("status phải Ok");
    let nhom = nhom_cua(&truoc, "da_stage.txt");
    assert_eq!(
        nhom.len(),
        2,
        "tệp MM phải sinh HAI phần tử (một Staged, một Unstaged). Đọc được: {nhom:?}"
    );
    assert!(nhom.contains(&StatusGroup::Staged));
    assert!(nhom.contains(&StatusGroup::Unstaged));

    let sau = stage_duong_dan(&state, Arc::clone(&repo), &["da_stage.txt".to_owned()])
        .await
        .expect("stage phải Ok");
    assert_eq!(
        nhom_cua(&sau, "da_stage.txt"),
        vec![StatusGroup::Staged],
        "stage một tệp MM phải làm nó chỉ còn ở nhóm đã stage"
    );
}
