//! Command lịch sử — HIST-01, HIST-02, HIST-06, HIST-07, HIST-10.
//!
//! Bốn command, tất cả **chỉ đọc**: Phase 2 không thay đổi repository, nên không lời
//! gọi nào ở đây dùng `GitRunner::write`.
//!
//! # `repo_id` tường minh, **không** `active_repo()`
//!
//! Mọi command nhận `repo_id: String` từ webview và tra bằng
//! [`AppState::get_repo`](crate::state::AppState::get_repo). `active_repo()` là lối tắt
//! của Phase 1 và nó sai ngay khi có hai repo mở cùng lúc (PLAT-05): hai cửa sổ hỏi
//! hai repo khác nhau sẽ cùng nhận dữ liệu của repo được mở sau cùng.
//!
//! Đây cũng là cổng an toàn T-02-12: `get_repo` chỉ trả về repo **đã mở** qua
//! `open_repository`, nên một `repo_id` bịa từ webview cho `UnknownRepository` chứ
//! không mở được thư mục tuỳ ý. Đường dẫn không bao giờ đến từ webview ở đây.

use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::State;

use crate::cache::RepoHistory;
use crate::domain::{Commit, Ref};
use crate::error::{GitError, Result};
use crate::git::parsers::log::{parse_log, LOG_ARGS, LOG_FORMAT};
use crate::git::parsers::refs::{parse_refs, REFS_ARGS, REFS_FORMAT};
use crate::git::GitCommand;
use crate::graph::GraphRow;
use crate::state::{AppState, RepoHandle};

/// Hạn giờ cho lần nạp lịch sử đầu tiên.
///
/// `DEFAULT_TIMEOUT` là 30 giây, hợp cho lệnh đọc nhỏ nhưng **quá ngắn** cho
/// `git log --all` trên repo lớn: chỉ riêng repo mẫu 100k commit đã mất 693ms trên
/// máy nhanh với bộ nhớ đệm hệ điều hành đã ấm, và một repo hàng triệu commit trên đĩa
/// nguội dễ dàng vượt 30 giây. Hết hạn ở đây nghĩa là người dùng thấy lỗi thay vì thấy
/// lịch sử.
///
/// **Không** dùng `NETWORK_TIMEOUT`: đây là lệnh cục bộ, không đi qua mạng, và mượn
/// hằng số của nhóm khác sẽ khiến việc chỉnh hạn giờ mạng vô tình đổi hành vi ở đây.
///
/// Vẫn **phải có** một hạn giờ (T-02-14): thiếu nó thì một repo hỏng làm treo giao
/// diện vĩnh viễn, không có đường thoát.
const HISTORY_TIMEOUT: Duration = Duration::from_secs(120);

/// Chặn trên số commit trả về trong một trang (T-02-13).
///
/// `limit` đến từ webview và không đáng tin. Một `limit` khổng lồ sẽ bắt Rust
/// serialize hàng chục MB JSON trong một lời gọi IPC và làm đơ webview — tự gây từ chối
/// dịch vụ. Giao diện ảo hoá chỉ hiện vài chục dòng một lúc, nên 10 000 đã rộng rãi.
const MAX_PAGE_LIMIT: usize = 10_000;

/// Chặn trên số tệp trả về trong `get_commit_detail`.
///
/// Một commit nhập kho ban đầu hoặc một lần chạy công cụ sinh mã có thể đổi hàng trăm
/// nghìn tệp. Trả hết sẽ làm treo giao diện; cắt và bật cờ `truncated` để người dùng
/// biết mình đang xem một phần.
const MAX_FILES_PER_COMMIT: usize = 5_000;

/// Chặn trên số kết quả tìm kiếm.
const MAX_SEARCH_RESULTS: usize = 1_000;

/// Mã của **cây rỗng** trong git — một hằng số của chính git, giống nhau ở mọi repo.
///
/// Dùng làm "cha" của commit gốc: commit gốc không có `<commit>^`, nên
/// `git diff <commit>^ <commit>` thất bại với `unknown revision`. So với cây rỗng cho
/// đúng thứ ta muốn — mọi tệp trong commit gốc hiện ra dưới trạng thái `A`.
///
/// # Vì sao chọn cách này chứ không `git show --name-status`
///
/// `git show` **cũng** làm được, nhưng nó đi một đường mã khác của git: với merge
/// commit nó mặc định không in gì cả, nên `get_commit_detail` sẽ có hai hành vi tuỳ
/// commit là gốc hay không. Dùng `git diff` cho cả hai trường hợp giữ đúng **một**
/// đường phân tích đầu ra, và khác biệt duy nhất là chọn mã nào làm vế trái.
const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// Một trang lịch sử cộng hình học đồ thị của đúng trang đó.
///
/// `commits` và `graph_rows` **luôn cùng độ dài** — cùng bất biến với
/// [`RepoHistory`], chỉ là đã cắt. Giao diện vẽ hai cột từ cùng một mảng
/// `virtualItems`, nên hai vector lệch nhau một phần tử là đồ thị lệch hàng.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitPage {
    pub commits: Vec<Commit>,
    pub graph_rows: Vec<GraphRow>,
    /// Tổng số commit trong **toàn bộ** lịch sử, không phải số commit trong trang.
    /// Giao diện cần nó để đặt chiều cao vùng cuộn ảo hoá.
    pub total: usize,
    /// Số bản ghi `git log` mà bộ phân tích phải bỏ. Bình thường là 0.
    pub skipped_records: usize,
}

/// Một tệp thay đổi trong một commit.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    /// Chữ trạng thái của git: `A`, `M`, `D`, `R`, `C`, `T`, `U`.
    ///
    /// Là `String` chứ không phải `char`: git in kèm điểm tương đồng (`R100`, `C75`),
    /// và `serde` serialize `char` thành chuỗi một ký tự nên phía TypeScript sẽ nhận
    /// một kiểu khác với điều mình mong. Giữ nguyên chuỗi git trả về.
    pub status: String,
    pub path: String,
    /// Đường dẫn cũ, chỉ có với `R` (đổi tên) và `C` (sao chép).
    pub old_path: Option<String>,
}

/// Chi tiết một commit: metadata cộng danh sách tệp thay đổi.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub commit: Commit,
    pub files: Vec<FileChange>,
    /// `true` khi danh sách tệp đã bị cắt ở [`MAX_FILES_PER_COMMIT`].
    pub truncated: bool,
}

/// Tra một repo đang mở theo `repo_id` (T-02-12).
fn repo_cua(state: &AppState, repo_id: &str) -> Result<Arc<RepoHandle>> {
    state
        .get_repo(repo_id)
        .ok_or_else(|| GitError::UnknownRepository(repo_id.to_owned()))
}

/// Mili giây từ epoch. `0` nếu đồng hồ hệ thống đặt trước 1970 — không phải lý do để
/// làm hỏng một lời gọi command.
fn bay_gio_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Nạp **toàn bộ** lịch sử của một repo và tính hình học đồ thị cho nó.
///
/// # Vì sao nạp hết chứ không nạp theo trang — đọc trước khi "tối ưu"
///
/// Gán lane **phải** chạy trên toàn bộ lịch sử. ARCHITECTURE.md nói rõ *"lane assignment
/// needs global context, can't be computed per-page in isolation"*: lane của hàng 5000
/// là kết quả của mọi hàng từ 0 tới 4999. Chạy `assign` trên một lát cắt
/// `commits[5000..5100]` sẽ ra một đồ thị **trông hợp lý** nhưng sai — mọi đường nối
/// bắt đầu lại từ lane 0, và lỗi đó gần như không thấy được trên repo nhỏ vì trang đầu
/// tiên luôn đúng.
///
/// Vì vậy: phân trang áp dụng cho **việc trả dữ liệu**, không cho việc tính toán. Cắt
/// trang chỉ là một phép slice trên kết quả đã tính xong.
///
/// Chi phí đã đo ở 100k commit: `git log` 693ms + `parse_log` 82ms + `assign` 57ms.
/// Chạy **một lần** rồi vào cache; những lần gọi sau không sinh tiến trình git nào.
async fn nap_lich_su(state: &AppState, repo: &Arc<RepoHandle>) -> Result<RepoHistory> {
    let runner = state.runner(Arc::clone(repo));

    let out = runner
        .read_ok(
            GitCommand::new(&repo.path)
                .args(LOG_ARGS)
                .arg(LOG_FORMAT)
                .timeout(HISTORY_TIMEOUT),
        )
        .await?;

    let parsed = parse_log(&out.stdout);

    // HIST-11 nói không được mất dòng. Nếu vẫn mất thì phải có vết — im lặng ở đây
    // nghĩa là lỗi sẽ được báo cáo dưới dạng "thiếu commit" nhiều tháng sau, khi không
    // còn cách nào truy ra nguyên nhân.
    if parsed.skipped_records > 0 {
        tracing::warn!(
            repo = %repo.id,
            skipped = parsed.skipped_records,
            "parse_log bỏ qua bản ghi — LOG_FORMAT và parse_log có thể đã lệch nhau"
        );
    }
    if parsed.bad_timestamps > 0 {
        tracing::warn!(
            repo = %repo.id,
            bad_timestamps = parsed.bad_timestamps,
            "parse_log gặp timestamp không đọc được, đã thay bằng 0"
        );
    }

    let graph_rows = crate::graph::assign(&parsed.commits);
    debug_assert_eq!(
        graph_rows.len(),
        parsed.commits.len(),
        "bất biến của 02-03: mỗi commit đúng một hàng đồ thị"
    );

    Ok(RepoHistory {
        commits: parsed.commits,
        graph_rows,
        loaded_at_ms: bay_gio_ms(),
    })
}

/// Lấy lịch sử từ cache, nạp nếu chưa có.
///
/// Cache hit **cố ý không** sinh dòng nào trong `CommandLog` (T-02-15): nhật ký ghi các
/// lệnh git đã chạy, không ghi lời gọi IPC.
async fn lich_su_cua(state: &AppState, repo: &Arc<RepoHandle>) -> Result<Arc<RepoHistory>> {
    if let Some(h) = state.cache.get_history(&repo.id) {
        return Ok(h);
    }

    let history = nap_lich_su(state, repo).await?;
    let active = state.active_repo().map(|r| r.id.clone());
    state
        .cache
        .put_history_keeping(&repo.id, history, active.as_deref());

    // `put_history_keeping` có thể đã loại chính repo này nếu chặn trên là 0. Đọc lại
    // thay vì giả định, và nạp lại nếu thật sự không có — không `unwrap`.
    match state.cache.get_history(&repo.id) {
        Some(h) => Ok(h),
        None => Ok(Arc::new(nap_lich_su(state, repo).await?)),
    }
}

/// Một trang lịch sử kèm hình học đồ thị — HIST-01, HIST-02.
///
/// Lần gọi đầu tiên cho một repo chạy **một** lệnh `git log --all --topo-order` rồi
/// tính lane cho toàn bộ lịch sử và cất vào cache. Mọi lần gọi sau cắt trang từ cache
/// và **không sinh tiến trình git nào**.
///
/// `skip` và `limit` đến từ webview nên được kẹp biên: `skip` vượt tổng số commit trả
/// về trang rỗng chứ không panic, và `limit` bị chặn trên ở [`MAX_PAGE_LIMIT`]
/// (T-02-13).
#[tauri::command]
pub async fn get_commit_page(
    repo_id: String,
    skip: usize,
    limit: usize,
    state: State<'_, AppState>,
) -> Result<CommitPage> {
    let repo = repo_cua(&state, &repo_id)?;
    let history = lich_su_cua(&state, &repo).await?;

    let total = history.commits.len();
    let limit = limit.min(MAX_PAGE_LIMIT);

    // `skip` ngoài phạm vi phải cho trang rỗng, không panic. `min(total)` làm hai việc:
    // chặn chỉ mục đầu, và khiến `end` bên dưới không bao giờ nhỏ hơn `start`.
    let start = skip.min(total);
    let end = start.saturating_add(limit).min(total);

    let commits = history.commits[start..end].to_vec();
    let graph_rows = history.graph_rows[start..end].to_vec();
    debug_assert_eq!(commits.len(), graph_rows.len());

    Ok(CommitPage {
        commits,
        graph_rows,
        total,
        skipped_records: 0,
    })
}

/// Nhánh local, nhánh remote và tag của một repo — HIST-06, HIST-07.
///
/// Cache theo `RepoId` như lịch sử; lần gọi thứ hai không sinh tiến trình git.
#[tauri::command]
pub async fn list_refs(repo_id: String, state: State<'_, AppState>) -> Result<Vec<Ref>> {
    let repo = repo_cua(&state, &repo_id)?;

    if let Some(refs) = state.cache.get_refs(&repo.id) {
        return Ok((*refs).clone());
    }

    let runner = state.runner(Arc::clone(&repo));
    let out = runner
        .read_ok(GitCommand::new(&repo.path).args(REFS_ARGS).arg(REFS_FORMAT))
        .await?;

    let refs = parse_refs(&out.stdout);
    state.cache.put_refs(&repo.id, refs.clone());
    Ok(refs)
}

/// Chi tiết một commit kèm danh sách tệp thay đổi — tiêu chí thành công số 3.
///
/// # Hai ca biên mà một cài đặt ngây thơ làm sai
///
/// **Commit gốc không có `<commit>^`.** `git diff <gốc>^ <gốc>` thất bại với
/// `unknown revision`, nên một cài đặt chỉ nối `^` sẽ báo lỗi ở đúng commit đầu tiên
/// của mọi repo. Phát hiện bằng `commit.parents.is_empty()` và so với [`EMPTY_TREE`].
///
/// **Merge commit có nhiều cha.** Diff với `parents[0]` — nhánh mà merge này *đi vào*.
/// Đó là lựa chọn của mọi công cụ git GUI và là thứ người dùng mong đợi khi bấm vào một
/// merge; diff kết hợp nhiều cha (`git diff -c`) là một chế độ hiển thị khác, thuộc
/// Phase 3.
#[tauri::command]
pub async fn get_commit_detail(
    repo_id: String,
    commit_id: String,
    state: State<'_, AppState>,
) -> Result<CommitDetail> {
    let repo = repo_cua(&state, &repo_id)?;
    let history = lich_su_cua(&state, &repo).await?;

    let commit = history
        .commits
        .iter()
        .find(|c| c.id == commit_id)
        .ok_or_else(|| GitError::ParseFailed(format!("không có commit {commit_id} trong lịch sử")))?
        .clone();

    // Commit gốc so với cây rỗng; commit thường và merge so với cha đầu tiên.
    let ve_trai = commit
        .parents
        .first()
        .cloned()
        .unwrap_or_else(|| EMPTY_TREE.to_owned());

    let runner = state.runner(Arc::clone(&repo));
    let out = runner
        .read_ok(GitCommand::new(&repo.path).args([
            // `core.quotepath=false` cùng với `-z` để tên tệp Unicode không bị git
            // escape thành bát phân. `-z` một mình đã đủ, nhưng PITFALLS mục 3 khuyến
            // nghị cả hai và chi phí bằng không.
            "-c",
            "core.quotepath=false",
            "diff",
            "--name-status",
            "-z",
            "--find-renames",
            &ve_trai,
            &commit.id,
        ]))
        .await?;

    let (files, truncated) = parse_name_status(&out.stdout, MAX_FILES_PER_COMMIT);

    Ok(CommitDetail {
        commit,
        files,
        truncated,
    })
}

/// Phân tích `git diff --name-status -z`.
///
/// Định dạng, đo thật trên git 2.54: các trường ngăn nhau bằng `\0`, không có ký tự
/// xuống dòng nào. Mỗi bản ghi là `<trạng thái>\0<đường dẫn>\0`, **trừ** `R` và `C`
/// vốn mang điểm tương đồng và chiếm **hai** đường dẫn:
///
/// ```text
/// M\0file.txt\0
/// R100\0local1.txt\0renamed1.txt\0
/// ```
///
/// Đọc `R`/`C` như bản ghi một đường dẫn sẽ làm **mọi** bản ghi sau nó lệch một nấc:
/// đường dẫn mới bị đọc thành trạng thái và trạng thái kế tiếp bị đọc thành đường dẫn.
/// Một lỗi làm hỏng cả danh sách chứ không chỉ một dòng.
///
/// Giải mã lossy: tên tệp trên Linux không bảo đảm UTF-8 (HIST-11).
fn parse_name_status(stdout: &[u8], max_files: usize) -> (Vec<FileChange>, bool) {
    let mut files = Vec::new();
    let mut truncated = false;

    let mut fields = stdout
        .split(|&b| b == 0)
        .filter(|f| !f.is_empty())
        .map(|f| String::from_utf8_lossy(f).into_owned());

    while let Some(status) = fields.next() {
        if files.len() >= max_files {
            truncated = true;
            break;
        }

        // `R`/`C` mang điểm tương đồng và hai đường dẫn.
        let doi_ten = status.starts_with('R') || status.starts_with('C');

        let Some(first) = fields.next() else {
            // Bản ghi cụt ở cuối buffer. Bỏ, đừng đoán.
            break;
        };

        if doi_ten {
            let Some(second) = fields.next() else {
                break;
            };
            files.push(FileChange {
                status,
                path: second,
                old_path: Some(first),
            });
        } else {
            files.push(FileChange {
                status,
                path: first,
                old_path: None,
            });
        }
    }

    (files, truncated)
}

/// Tìm commit theo từ khoá — HIST-10.
///
/// Trả **mã commit**, không trả `Commit` đầy đủ: giao diện đã có metadata của trang
/// đang hiện, nó chỉ cần biết hàng nào khớp để tô sáng.
///
/// # Ba trục tìm trong bộ nhớ, một trục hỏi git
///
/// Thông điệp, tên tác giả và mã commit lọc **trong bộ nhớ** trên lịch sử đã cache.
/// ARCHITECTURE.md Anti-Pattern 4 cấm chạy git mỗi lần người dùng gõ một ký tự, và ô
/// tìm kiếm là đúng nơi vi phạm đó xảy ra.
///
/// Trục **đường dẫn tệp** buộc phải hỏi git: cache giữ metadata commit, không giữ danh
/// sách tệp của từng commit. Lệnh có `--` trước pathspec (T-02-11) để một từ khoá trùng
/// tên nhánh không bị git hiểu thành một revision.
#[tauri::command]
pub async fn search_commits(
    repo_id: String,
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>> {
    let repo = repo_cua(&state, &repo_id)?;

    let query = query.trim().to_owned();
    if query.is_empty() {
        return Ok(Vec::new());
    }

    let history = lich_su_cua(&state, &repo).await?;
    let needle = query.to_lowercase();

    let mut ket_qua: Vec<String> = Vec::new();
    let mut da_co: std::collections::HashSet<String> = std::collections::HashSet::new();

    for c in &history.commits {
        if ket_qua.len() >= MAX_SEARCH_RESULTS {
            break;
        }
        let khop = c.subject.to_lowercase().contains(&needle)
            || c.body.to_lowercase().contains(&needle)
            || c.author_name.to_lowercase().contains(&needle)
            || c.id.to_lowercase().contains(&needle);
        if khop && da_co.insert(c.id.clone()) {
            ket_qua.push(c.id.clone());
        }
    }

    // Trục thứ tư: đường dẫn tệp. Chỉ hỏi git khi còn chỗ trong kết quả.
    if ket_qua.len() < MAX_SEARCH_RESULTS {
        let runner = state.runner(Arc::clone(&repo));
        let pathspec = format!("*{query}*");
        let out = runner
            .read(
                GitCommand::new(&repo.path)
                    .args([
                        "log",
                        "--all",
                        "--format=%H",
                        &format!("--max-count={MAX_SEARCH_RESULTS}"),
                    ])
                    // `--` NGĂN CÁCH revision với pathspec (T-02-11). Thiếu nó thì một
                    // từ khoá trùng tên nhánh được git hiểu thành revision, và lệnh
                    // trả về một tập commit hoàn toàn khác mà không báo lỗi.
                    .arg("--")
                    .arg(&pathspec)
                    .timeout(HISTORY_TIMEOUT),
            )
            .await?;

        // Pathspec không khớp gì làm git thoát khác 0 trên một số phiên bản. Đó không
        // phải lỗi của người dùng — dùng `read` chứ không `read_ok` và bỏ qua khi thất
        // bại, vì ba trục kia vẫn cho kết quả dùng được.
        if out.is_success() {
            for line in out.stdout.split(|&b| b == b'\n') {
                if ket_qua.len() >= MAX_SEARCH_RESULTS {
                    break;
                }
                let id = String::from_utf8_lossy(line).trim().to_owned();
                if id.is_empty() {
                    continue;
                }
                if da_co.insert(id.clone()) {
                    ket_qua.push(id);
                }
            }
        }
    }

    Ok(ket_qua)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_status_doc_ban_ghi_mot_duong_dan() {
        let (files, truncated) = parse_name_status(b"M\0file.txt\0", 100);
        assert!(!truncated);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].status, "M");
        assert_eq!(files[0].path, "file.txt");
        assert!(files[0].old_path.is_none());
    }

    /// `R`/`C` chiếm **hai** đường dẫn. Đọc sai thì mọi bản ghi sau nó lệch một nấc —
    /// đây là lý do test có thêm một bản ghi đứng sau.
    #[test]
    fn name_status_doi_ten_chiem_hai_duong_dan_khong_lam_lech_ban_ghi_sau() {
        let (files, _) =
            parse_name_status(b"R100\0cu.txt\0moi.txt\0M\0khac.txt\0A\0them.txt\0", 100);

        assert_eq!(files.len(), 3, "phải đọc được đúng ba bản ghi");

        assert_eq!(files[0].status, "R100");
        assert_eq!(files[0].old_path.as_deref(), Some("cu.txt"));
        assert_eq!(files[0].path, "moi.txt");

        assert_eq!(
            files[1].status, "M",
            "bản ghi ngay sau R không được lệch nấc"
        );
        assert_eq!(files[1].path, "khac.txt");

        assert_eq!(files[2].status, "A");
        assert_eq!(files[2].path, "them.txt");
    }

    #[test]
    fn name_status_sao_chep_cung_chiem_hai_duong_dan() {
        let (files, _) = parse_name_status(b"C75\0nguon.txt\0dich.txt\0", 100);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].old_path.as_deref(), Some("nguon.txt"));
        assert_eq!(files[0].path, "dich.txt");
    }

    #[test]
    fn name_status_buffer_rong_cho_danh_sach_rong() {
        let (files, truncated) = parse_name_status(b"", 100);
        assert!(files.is_empty());
        assert!(!truncated);
    }

    /// Bản ghi cụt ở cuối buffer bị bỏ, không panic.
    #[test]
    fn name_status_ban_ghi_cut_khong_panic() {
        let (files, _) = parse_name_status(b"M\0file.txt\0A\0", 100);
        assert_eq!(files.len(), 1, "bản ghi cụt bị bỏ, bản ghi đủ được giữ");
    }

    /// Tên tệp không UTF-8 vẫn ra một `FileChange` (HIST-11).
    #[test]
    fn name_status_ten_tep_khong_utf8_van_ra_mot_ban_ghi() {
        let (files, _) = parse_name_status(b"M\0t\xffep.txt\0", 100);
        assert_eq!(files.len(), 1);
        assert!(files[0].path.contains('\u{fffd}'));
    }

    /// Cắt ở `max_files` và bật cờ `truncated`.
    #[test]
    fn name_status_cat_va_bat_co_truncated() {
        let mut buf = Vec::new();
        for i in 0..10 {
            buf.extend_from_slice(format!("M\0f{i}.txt\0").as_bytes());
        }
        let (files, truncated) = parse_name_status(&buf, 3);
        assert_eq!(files.len(), 3);
        assert!(truncated, "cắt danh sách phải bật cờ truncated");
    }

    /// Mã cây rỗng là một hằng số của git, không phải một mã ngẫu nhiên. Ghim nó bằng
    /// test để không ai "sửa" thành mã khác.
    #[test]
    fn hang_cay_rong_dung_ma_cua_git() {
        assert_eq!(EMPTY_TREE, "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
        assert_eq!(EMPTY_TREE.len(), 40);
    }

    /// **Hợp đồng IPC:** tên khoá JSON của `CommitPage` và `FileChange` là thứ
    /// `src/lib/ipc.ts` đọc. Đổi tên trường bên Rust mà quên bên TypeScript làm giao
    /// diện nhận `undefined` — không lỗi biên dịch ở cả hai phía, chỉ là màn hình
    /// trống. Ghim bằng test, đúng tiền lệ plan 02-02 đã lập cho `Commit`.
    #[test]
    fn payload_serialize_dung_ten_khoa_camel_case() {
        let page = CommitPage {
            commits: Vec::new(),
            graph_rows: Vec::new(),
            total: 7,
            skipped_records: 0,
        };
        let json = serde_json::to_value(&page).unwrap();
        assert_eq!(json["total"], 7);
        assert!(json.get("graphRows").is_some(), "phải là graphRows");
        assert!(
            json.get("skippedRecords").is_some(),
            "phải là skippedRecords"
        );
        assert!(json.get("graph_rows").is_none(), "không được là graph_rows");
        assert!(json.get("skipped_records").is_none());

        let fc = FileChange {
            status: "R100".into(),
            path: "moi.txt".into(),
            old_path: Some("cu.txt".into()),
        };
        let json = serde_json::to_value(&fc).unwrap();
        assert_eq!(json["status"], "R100");
        assert_eq!(json["path"], "moi.txt");
        assert_eq!(json["oldPath"], "cu.txt");
        assert!(json.get("old_path").is_none(), "không được là old_path");

        // `old_path` là `None` phải cho `null`, để TypeScript dùng `string | null`.
        let fc = FileChange {
            status: "M".into(),
            path: "f.txt".into(),
            old_path: None,
        };
        assert!(serde_json::to_value(&fc).unwrap()["oldPath"].is_null());
    }

    /// `CommitDetail` mang cờ `truncated`, `files` là mảng, và `Commit` lồng bên trong
    /// vẫn giữ camelCase của riêng nó.
    #[test]
    fn commit_detail_serialize_dung_hinh_dang() {
        let detail = CommitDetail {
            commit: Commit {
                id: "a".repeat(40),
                parents: Vec::new(),
                author_name: "A".into(),
                author_email: "a@b.c".into(),
                author_time: 1,
                committer_name: "A".into(),
                committer_email: "a@b.c".into(),
                committer_time: 1,
                subject: "s".into(),
                body: String::new(),
                has_invalid_utf8: false,
            },
            files: Vec::new(),
            truncated: true,
        };
        let json = serde_json::to_value(&detail).unwrap();
        assert!(json["commit"].is_object());
        assert!(json["files"].is_array());
        assert_eq!(json["truncated"], true);
        assert!(json["commit"].get("authorName").is_some());
        assert!(json["commit"].get("hasInvalidUtf8").is_some());
    }
}
