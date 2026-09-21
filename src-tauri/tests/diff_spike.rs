//! Test tích hợp cho `spike_blob_pair` — plan 03-01, checkpoint #3.
//!
//! # Vì sao test chạy git THẬT trên repo tạm, không dùng buffer tự dựng
//!
//! `spike_blob_pair` không có bộ phân tích nào để kiểm riêng: nó chỉ ghép ba lệnh
//! git lại. Một test dựng sẵn `stdout` rồi kiểm cách nó được ghép sẽ kiểm đúng cái
//! nó tự bịa ra. Điều thật sự dễ vỡ ở đây là **ba lệnh git có chạy được không** trên
//! ca commit gốc — và chỉ git thật trả lời được.
//!
//! Cùng lý do với `history_commands.rs`: test gọi trực tiếp hàm logic mà command gọi,
//! với `AppState` thật và `GitRunner` thật, chứ không đi qua `invoke` (việc đó cần một
//! `App` có webview, không chạy được trên CI không đầu).

use std::path::Path;
use std::process::Command;

use git_plum_lib::commands::diff_spike::lay_cap_blob;
use git_plum_lib::state::AppState;

/// Chạy một lệnh git đồng bộ trong lúc dựng repo mẫu.
///
/// Dùng `std::process::Command` trực tiếp, không qua `GitCommand`: đây là phần
/// **dựng dữ liệu cho test**, không phải phần được kiểm. Đi qua lớp thật ở đây chỉ
/// khiến một lỗi trong lớp thật làm test thất bại vì lý do sai chỗ.
fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .env("GIT_AUTHOR_NAME", "Test")
        .env("GIT_AUTHOR_EMAIL", "test@example.com")
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "test@example.com")
        .output()
        .expect("phải chạy được git");
    assert!(
        out.status.success(),
        "git {args:?} thất bại: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Dựng một repo có hai commit: commit gốc thêm `file.txt`, commit sau sửa nó.
///
/// Trả về `(thư mục tạm, sha gốc, sha thứ hai)`. Giữ `TempDir` sống trong người gọi
/// — thả nó ra là xoá cả repo.
fn repo_hai_commit() -> (tempfile::TempDir, String, String) {
    let dir = tempfile::tempdir().expect("phải tạo được thư mục tạm");
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    std::fs::write(p.join("file.txt"), "dong mot\ndong hai\n").unwrap();
    git(p, &["add", "file.txt"]);
    git(p, &["commit", "-m", "commit goc"]);

    let goc = rev_parse(p, "HEAD");

    std::fs::write(p.join("file.txt"), "dong mot\ndong hai da sua\ndong ba\n").unwrap();
    git(p, &["add", "file.txt"]);
    git(p, &["commit", "-m", "sua file"]);

    let sau = rev_parse(p, "HEAD");

    (dir, goc, sau)
}

fn rev_parse(dir: &Path, rev: &str) -> String {
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", rev])
        .output()
        .expect("phải chạy được git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_owned()
}

/// Ca thường: tệp bị sửa giữa hai commit.
///
/// Khẳng định cả **ba** trường mà hai đường đo cần: `old_text` và `new_text` khác
/// nhau (đầu vào đường A), và `patch` có `@@` (đầu vào đường B). Thiếu bất kỳ trường
/// nào thì một trong hai đường không đo được, và phép so sánh của checkpoint #3 mất
/// nghĩa.
#[tokio::test]
async fn tep_sua_tra_hai_phia_khac_nhau_va_patch_co_hunk() {
    let (dir, _goc, sau) = repo_hai_commit();

    let state = AppState::new();
    let repo = state.open_repo(dir.path());

    let cap = lay_cap_blob(&state, repo, &sau, "file.txt")
        .await
        .expect("spike_blob_pair phải chạy được trên tệp bị sửa");

    assert!(
        cap.old_text.contains("dong hai\n"),
        "old_text phải là nội dung phía CŨ, nhận được: {:?}",
        cap.old_text
    );
    assert!(
        cap.new_text.contains("dong hai da sua"),
        "new_text phải là nội dung phía MỚI, nhận được: {:?}",
        cap.new_text
    );
    assert_ne!(
        cap.old_text, cap.new_text,
        "hai phía phải khác nhau, nếu không đường A không có gì để tính diff"
    );

    assert!(
        cap.patch.contains("@@"),
        "patch phải có đầu hunk `@@` — đó là thứ duy nhất đường B đọc để dựng \
         decoration; patch rỗng làm đường B đo ra một con số đẹp vô nghĩa. \
         Nhận được: {:?}",
        cap.patch
    );

    // Các trường kích thước để tài liệu ghi được "đo trên tệp bao nhiêu dòng",
    // không chỉ ghi ms. Một con số ms không kèm kích thước đầu vào thì không so
    // lại được ở lần đo sau.
    assert_eq!(cap.old_bytes, cap.old_text.len());
    assert_eq!(cap.new_bytes, cap.new_text.len());
    assert_eq!(cap.patch_bytes, cap.patch.len());
    assert_eq!(cap.old_lines, 2, "phía cũ có hai dòng");
    assert_eq!(cap.new_lines, 3, "phía mới có ba dòng");
}

/// **Ca commit gốc** — ca mà một cài đặt chỉ nối `^` vào sha sẽ vỡ.
///
/// `git rev-parse <goc>^` thất bại vì commit gốc không có cha, và
/// `git show <cay-rong>:file.txt` cũng thất bại vì cây rỗng không chứa tệp nào.
/// Cả hai đều là ca **bình thường**, không phải lỗi: tệp mới thêm thì phía cũ rỗng.
/// Trả `Err` ở đây nghĩa là không bao giờ đo được diff của commit đầu tiên.
#[tokio::test]
async fn commit_goc_cho_phia_cu_rong_chu_khong_bao_loi() {
    let (dir, goc, _sau) = repo_hai_commit();

    let state = AppState::new();
    let repo = state.open_repo(dir.path());

    let cap = lay_cap_blob(&state, repo, &goc, "file.txt")
        .await
        .expect("commit gốc phải trả Ok — tệp mới thêm là ca bình thường, không phải lỗi");

    assert_eq!(
        cap.old_text, "",
        "commit gốc không có phía cũ, old_text phải rỗng"
    );
    assert_eq!(cap.old_bytes, 0);
    assert_eq!(cap.old_lines, 0);

    assert!(
        cap.new_text.contains("dong mot"),
        "new_text của commit gốc phải có nội dung"
    );
    assert!(
        cap.patch.contains("@@"),
        "diff với cây rỗng vẫn phải sinh hunk"
    );
}

/// **T-03-01: `--` phải có mặt trong lệnh diff của `spike_blob_pair` — đọc ở chính
/// mã nguồn, không ở một lệnh git do test tự dựng.**
///
/// Đây là bài học HIST-10 chép nguyên: test
/// `tim_theo_duong_dan_co_dau_gach_ngang_ngan_cach` của Phase 2 dựng lệnh git riêng
/// nên nó chứng minh `--` *có tác dụng* mà không chứng minh hàm thật *dùng* nó — đã
/// kiểm bằng đột biến: xoá `.arg("--")` khỏi `search_commits` mà test kia vẫn xanh.
///
/// Thiếu `--`, một `path` trùng tên nhánh (`main`, `dev`, `HEAD`) được git đọc thành
/// **revision** thay vì pathspec. Ở spike thì `path` do người chạy tự gõ vào ô nhập,
/// tức chuỗi tuỳ ý đi thẳng vào argv.
#[test]
fn lenh_diff_co_dau_gach_ngang_truoc_path() {
    let src = include_str!("../src/commands/diff_spike.rs");

    let (_, than) = src
        .split_once("async fn lay_cap_blob")
        .expect("phải có hàm lay_cap_blob");

    // Chỉ xét thân hàm, bỏ dòng chú thích — một `--` trong chú thích không ngăn cách
    // gì cả, và đó đúng là cách một cổng grep trở thành vô dụng.
    let than_ma: String = than
        .lines()
        .take_while(|l| !l.starts_with("#[cfg(test)]"))
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        than_ma.contains(r#""--""#),
        "lay_cap_blob phải truyền `--` ngăn cách revision với path (T-03-01)"
    );

    let vi_tri_gach = than_ma.find(r#""--""#).expect("vừa khẳng định là có");
    let vi_tri_path = than_ma
        .find("PATH_SAU_DAU_GACH")
        .expect("lay_cap_blob phải truyền path qua mốc PATH_SAU_DAU_GACH");
    assert!(
        vi_tri_gach < vi_tri_path,
        "`--` phải đứng TRƯỚC path, nếu không nó không ngăn cách gì cả"
    );
}
