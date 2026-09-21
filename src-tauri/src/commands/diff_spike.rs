//! Command spike cho checkpoint #3 — **không phải tính năng của phase**.
//!
//! Phase 3 phải chọn giữa hai đường dựng trình xem diff:
//!
//! - **Đường A** — `@codemirror/merge` nhận **hai tài liệu đầy đủ** rồi *tự tính diff*.
//!   Ít mã hơn nhiều, nhưng nó làm lại một việc `git diff` đã làm xong.
//! - **Đường B** — dựng decoration CodeMirror **từ đầu ra `git diff`**, không để
//!   CodeMirror tính lại gì.
//!
//! ROADMAP nêu mối lo: trên tệp lớn, việc tính lại đó có thể quá chậm. Tệp này sinh
//! dữ liệu để đo mối lo đó bằng số thật thay vì lập luận.
//!
//! # Vì sao **một** command trả cả hai dạng dữ liệu
//!
//! Hai đường đo phải chạy trên **cùng một byte đầu vào**. Nếu mỗi đường tự gọi git
//! lấy dữ liệu riêng thì hai phép đo khác nhau ở hai biến (dữ liệu *và* cách dựng
//! view), và con số so sánh không còn nghĩa. Nên `spike_blob_pair` trả `old_text` +
//! `new_text` (đầu vào đường A) **và** `patch` (đầu vào đường B) trong một lần gọi.
//!
//! # Vòng đời
//!
//! Task 3 của plan 03-01 quyết định giữ hay xoá tệp này sau khi có số đo. Mã ở đây
//! được phép thô — nó không nằm trên đường người dùng, chỉ chạy khi
//! `localStorage.gitPlumPerf = '1'`.

use std::sync::Arc;
use std::time::Instant;

use serde::Serialize;
use tauri::State;

use crate::error::{GitError, Result};
use crate::git::GitCommand;
use crate::state::{AppState, RepoHandle};

/// Mã của **cây rỗng** trong git.
///
/// Bản gốc của hằng này là `EMPTY_TREE` trong [`crate::commands::history`], nơi có
/// phần giải thích đầy đủ vì sao dùng nó làm vế trái cho commit gốc. Ở đó nó `private`
/// nên khai lại ở đây thay vì mở rộng phạm vi hiển thị của mã Phase 2 cho một spike
/// sẽ bị xoá.
///
/// Đây là một hằng số **của chính git** — giống nhau ở mọi repo, không phải một mã
/// ngẫu nhiên ai đó dán vào.
const CAY_RONG: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// Dữ liệu đầu vào cho **cả hai** đường đo của checkpoint #3.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpikeBlobPair {
    /// Nội dung phía **cũ**, giải mã lossy. Rỗng khi tệp mới được thêm.
    /// Đầu vào của đường A.
    pub old_text: String,
    /// Nội dung phía **mới**, giải mã lossy. Đầu vào của đường A.
    pub new_text: String,
    /// `git diff --unified=3` cho đúng cặp đó, giải mã lossy. Đầu vào của đường B.
    pub patch: String,

    pub old_bytes: usize,
    pub new_bytes: usize,
    pub patch_bytes: usize,
    pub old_lines: usize,
    pub new_lines: usize,

    /// Thời gian phía Rust — cả ba lệnh git cộng lại.
    ///
    /// Cần nó để **tách** "git chậm" khỏi "CodeMirror chậm". Phase 2 học được đúng
    /// bài này: `git log` chiếm 84% tổng thời gian nạp lịch sử, và một con số tổng
    /// gộp không trả lời được câu hỏi phải tối ưu ở đâu. Nếu `rust_ms` đã là 300ms
    /// thì cả hai đường đều vượt ngưỡng vì cùng một lý do, và lý do đó không phải
    /// CodeMirror.
    pub rust_ms: u64,
}

/// Số dòng của một chuỗi, đếm theo kiểu "dòng nội dung".
///
/// Chuỗi rỗng cho 0. Chuỗi kết thúc bằng `\n` **không** tính một dòng rỗng ở cuối:
/// `"a\nb\n"` là hai dòng, không phải ba. Đây là cách `wc -l` và trình soạn thảo
/// đếm, nên con số trong tài liệu khớp với điều người đọc kiểm lại được bằng tay.
fn dem_dong(s: &str) -> usize {
    if s.is_empty() {
        return 0;
    }
    s.lines().count()
}

/// Lấy cặp blob cộng patch cho `(commit_id, path)`.
///
/// Tách khỏi `#[tauri::command]` để test tích hợp gọi được mà không cần dựng một
/// `App` có webview — cùng cách `history_commands.rs` làm.
///
/// # Ba lệnh git, tất cả qua `GitRunner::read`
///
/// 1. `rev-parse <commit_id>^` tìm vế trái. Thất bại (commit gốc) → dùng cây rỗng.
/// 2. `show <rev>:<path>` cho mỗi phía. Thất bại ở phía cũ → `""`.
/// 3. `diff --unified=3 <ve_trai> <commit_id> -- <path>` cho patch.
///
/// # T-03-01 — `path` từ webview vào argv
///
/// `path` ở spike do người chạy **tự gõ vào ô nhập**, nên nó là chuỗi tuỳ ý. Lệnh
/// diff có `--` **trước** path để một path trùng tên nhánh không bị git đọc thành
/// revision (T-02-11).
///
/// `git show <rev>:<path>` **không có** vị trí `--`, nên ở đó `rev` là kết quả đã đi
/// qua `rev-parse` chứ không phải chuỗi người dùng — chuỗi người dùng chỉ nằm ở phần
/// sau dấu hai chấm, vị trí mà git luôn đọc là đường dẫn.
pub async fn lay_cap_blob(
    state: &AppState,
    repo: Arc<RepoHandle>,
    commit_id: &str,
    path: &str,
) -> Result<SpikeBlobPair> {
    let bat_dau = Instant::now();
    let runner = state.runner(Arc::clone(&repo));

    // --- Lệnh 1: vế trái ---------------------------------------------------
    //
    // Không có `history` trong tay như `get_commit_detail`, nên hỏi thẳng git.
    // `rev-parse <sha>^` thoát khác 0 trên commit gốc; đó là câu trả lời "không có
    // cha", không phải lỗi.
    let ra_rev = runner
        .read(GitCommand::new(&repo.path).args(["rev-parse", &format!("{commit_id}^")]))
        .await?;

    let ve_trai = if ra_rev.is_success() {
        let s = String::from_utf8_lossy(&ra_rev.stdout).trim().to_owned();
        if s.is_empty() {
            CAY_RONG.to_owned()
        } else {
            s
        }
    } else {
        CAY_RONG.to_owned()
    };

    // --- Lệnh 2: hai phía nội dung ----------------------------------------
    //
    // Với cây rỗng làm vế trái, `git show <cay-rong>:<path>` thất bại vì cây rỗng
    // không chứa tệp nào. Tệp mới thêm là ca **bình thường** — trả `""` chứ không
    // `Err`, nếu không thì commit đầu tiên của mọi repo không bao giờ đo được.
    let old_text = doc_blob(state, &repo, &ve_trai, path).await?;
    let new_text = doc_blob(state, &repo, commit_id, path).await?;

    // --- Lệnh 3: patch ----------------------------------------------------
    let ra_diff = runner
        .read(
            GitCommand::new(&repo.path)
                .args([
                    "-c",
                    "core.quotepath=false",
                    "diff",
                    "--unified=3",
                    &ve_trai,
                    commit_id,
                ])
                // `--` NGĂN CÁCH revision với pathspec (T-03-01). Mốc dưới đây là
                // thứ test `lenh_diff_co_dau_gach_ngang_truoc_path` neo vào để
                // khẳng định thứ tự — đặt `--` sau path thì nó không ngăn cách gì.
                .arg("--")
                .arg(PATH_SAU_DAU_GACH(path)),
        )
        .await?;

    // Dùng `read` chứ không `read_ok`: một path không tồn tại trong cặp commit đó
    // làm git thoát khác 0, và ở spike thì người chạy tự gõ path nên gõ sai là ca
    // thường gặp. Patch rỗng là câu trả lời dùng được — nó cho đường B "không có
    // hunk nào", và `<verify>` phía TS bắt được điều đó.
    let patch = if ra_diff.is_success() {
        String::from_utf8_lossy(&ra_diff.stdout).into_owned()
    } else {
        String::new()
    };

    let rust_ms = bat_dau.elapsed().as_millis() as u64;

    Ok(SpikeBlobPair {
        old_bytes: old_text.len(),
        new_bytes: new_text.len(),
        patch_bytes: patch.len(),
        old_lines: dem_dong(&old_text),
        new_lines: dem_dong(&new_text),
        old_text,
        new_text,
        patch,
        rust_ms,
    })
}

/// Mốc đánh dấu vị trí `path` trong lệnh diff.
///
/// Hàm đồng nhất — nó **không** biến đổi gì. Nó tồn tại để test khẳng định được
/// *thứ tự* của `--` và `path` trong argv bằng cách tìm một chuỗi ổn định trong mã
/// nguồn. Neo vào tên biến `path` không được: chữ `path` xuất hiện khắp tệp (kể cả
/// trong `repo.path`), nên phép `find` sẽ khớp một chỗ khác và cổng trở thành vô
/// dụng — đúng loại lỗi mà bài học "cổng grep dễ vô dụng" của Phase 2 nói tới.
#[allow(non_snake_case)]
fn PATH_SAU_DAU_GACH(path: &str) -> &str {
    path
}

/// Đọc nội dung một blob. Thất bại → chuỗi rỗng.
///
/// Thất bại ở đây có ba nguyên nhân bình thường: vế trái là cây rỗng, tệp mới được
/// thêm nên chưa có ở phía cũ, hoặc tệp đã bị xoá nên không có ở phía mới. Cả ba đều
/// là dữ liệu hợp lệ cho phép đo, không phải lỗi.
async fn doc_blob(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    rev: &str,
    path: &str,
) -> Result<String> {
    let runner = state.runner(Arc::clone(repo));
    let ra = runner
        .read(GitCommand::new(&repo.path).args(["show", &format!("{rev}:{path}")]))
        .await?;

    if ra.is_success() {
        Ok(String::from_utf8_lossy(&ra.stdout).into_owned())
    } else {
        Ok(String::new())
    }
}

/// Trả hai phía nội dung tệp cộng patch thô cho cùng một cặp `(commit_id, path)`.
///
/// Chỉ phục vụ phép đo của checkpoint #3. Xem chú thích đầu tệp.
#[tauri::command]
pub async fn spike_blob_pair(
    repo_id: String,
    commit_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<SpikeBlobPair> {
    // T-03-02: tra repo theo `repo_id` tường minh (T-02-12). `get_repo` chỉ trả repo
    // **đã mở** qua `open_repository`, nên một `repo_id` bịa từ webview cho
    // `UnknownRepository` chứ không mở được thư mục tuỳ ý. Đường dẫn không bao giờ
    // đến từ webview.
    let repo = state
        .get_repo(&repo_id)
        .ok_or_else(|| GitError::UnknownRepository(repo_id.clone()))?;

    lay_cap_blob(&state, repo, &commit_id, &path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mã cây rỗng là hằng số của git, không phải mã ngẫu nhiên. Ghim để không ai
    /// "sửa" thành mã khác, và để nó không lệch khỏi bản gốc trong `history.rs`.
    #[test]
    fn hang_cay_rong_khop_ban_goc_trong_history() {
        assert_eq!(CAY_RONG, "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
        assert_eq!(CAY_RONG.len(), 40);

        // Đọc thẳng `history.rs` để hai hằng không bao giờ lệch nhau. Bài học Phase 2:
        // hai hằng ở hai phía phải có test ghim đọc thẳng tệp kia — lệch nhau là lỗi
        // im lặng.
        let src = include_str!("history.rs");
        assert!(
            src.contains(&format!(r#"EMPTY_TREE: &str = "{CAY_RONG}""#)),
            "CAY_RONG phải khớp EMPTY_TREE trong history.rs"
        );
    }

    /// Cách đếm dòng phải khớp điều người đọc tài liệu kiểm lại được bằng tay.
    #[test]
    fn dem_dong_khong_tinh_dong_rong_cuoi() {
        assert_eq!(dem_dong(""), 0, "chuỗi rỗng là 0 dòng");
        assert_eq!(dem_dong("a\n"), 1);
        assert_eq!(dem_dong("a\nb\n"), 2, "`\\n` cuối không tạo dòng thứ ba");
        assert_eq!(
            dem_dong("a\nb"),
            2,
            "dòng cuối không có `\\n` vẫn là một dòng"
        );
    }

    /// **Hợp đồng IPC:** tên khoá JSON là thứ `src/lib/ipc.ts` đọc. Đổi tên trường
    /// bên Rust mà quên bên TypeScript không gây lỗi biên dịch ở cả hai phía — giao
    /// diện chỉ nhận `undefined`, và ở đây điều đó nghĩa là bảng số đo hiện `NaN`.
    #[test]
    fn serialize_dung_ten_khoa_camel_case() {
        let cap = SpikeBlobPair {
            old_text: "a".into(),
            new_text: "b".into(),
            patch: "@@".into(),
            old_bytes: 1,
            new_bytes: 1,
            patch_bytes: 2,
            old_lines: 1,
            new_lines: 1,
            rust_ms: 7,
        };
        let json = serde_json::to_value(&cap).unwrap();

        for khoa in [
            "oldText",
            "newText",
            "patch",
            "oldBytes",
            "newBytes",
            "patchBytes",
            "oldLines",
            "newLines",
            "rustMs",
        ] {
            assert!(json.get(khoa).is_some(), "phải có khoá `{khoa}`");
        }
        assert_eq!(json["rustMs"], 7);
    }
}
