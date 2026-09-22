//! Command trạng thái thư mục làm việc và stage/unstage theo tệp — WORK-01, WORK-02.
//!
//! # Lệnh ghi trả trạng thái MỚI, không trả `()`
//!
//! Đây là ràng buộc 2.5 của `CONTEXT.md`, và kiểu trả về là cách bảo đảm nó:
//!
//! > Thao tác ghi trả trạng thái mới **trực tiếp**. Không chờ watcher. Watcher vẫn
//! > phát sự kiện để phòng thủ, nhưng đường chính là: ghi → trả trạng thái mới → giao
//! > diện cập nhật.
//!
//! Một command trả `()` **buộc** giao diện phải chờ watcher, và hai thứ hỏng theo:
//! giao diện trễ 250–300 ms sau **mỗi** cú bấm (khoảng gộp-và-trì-hoãn của watcher),
//! và nếu watcher chết thì giao diện **đứng im mà không ai biết** — không lỗi, không
//! thông báo, chỉ một danh sách không bao giờ đổi. Nên `Result<RepoStatus>` ở đây
//! không phải tiện lợi; nó là chỗ ràng buộc được ghim bằng *kiểu*.
//!
//! Wave này (04-02) **chưa có watcher** — nó tới ở 04-04. Đó là bằng chứng ràng buộc
//! được tôn trọng theo kiến trúc: giao diện cập nhật ngay dù không có gì để chờ.
//!
//! # `.git/index.lock` KHÔNG BAO GIỜ bị xoá
//!
//! Xem [`chay_lenh_ghi_co_thu_lai`]. Ràng buộc này ở mức "không thương lượng" trong
//! ROADMAP (CONTEXT.md 2.3, rủi ro R3).
//!
//! # Phân loại lỗi theo stderr, không theo mã thoát
//!
//! `git status` thoát khác 0 vì **nhiều** nguyên nhân: repo bị từ chối
//! (`dubious ownership`, exit 128), `index.lock`, repo hỏng. `open_repository` đã mắc
//! đúng lỗi này — nó coi "exit khác 0" là **một** nguyên nhân và báo sai "Không phải
//! một repository git" cho một repo hợp lệ (`docs/03-phase1-qa-windows.md` ca KB-4b).
//!
//! Ở đây phân loại theo **stderr** rồi trả mã lỗi khác nhau, và **luôn** kèm stderr
//! nguyên văn. Coi stderr là văn bản chẩn đoán **mờ**, không phải hợp đồng phân tích
//! được: phép so khớp chỉ dùng để *chọn thông báo tốt hơn*, và khi nó trượt thì ta
//! rơi về [`GitError::CommandFailed`] — vẫn mang đủ stderr cho người đọc.

use std::sync::Arc;
use std::time::Duration;

use tauri::State;

use crate::domain::{RepoStatus, STATUS_ARGS};
use crate::error::{GitError, Result};
use crate::git::exec::{GitOutput, DEFAULT_TIMEOUT};
use crate::git::parsers::status::parse_status;
use crate::git::GitCommand;
use crate::state::{AppState, RepoHandle};

/// Số lần **thử** một lệnh ghi khi gặp `.git/index.lock` (lần đầu cộng các lần lại).
///
/// Bốn lần thử là ba lần chờ: xem [`GIAN_CACH_MS`].
const SO_LAN_THU: usize = 4;

/// Giãn cách giữa các lần thử, mili giây — **tổng có chặn trên**.
///
/// `50 + 150 + 450 = 650 ms` cộng thời gian chạy của bốn lệnh `git add` (mỗi lệnh vài
/// chục ms trên repo thường) là dưới một giây.
///
/// # Vì sao có chặn trên, và vì sao nó thấp
///
/// Hai lý do, và lý do thứ hai quan trọng hơn:
///
/// 1. **Giao diện không được treo.** Người dùng bấm "stage" và chờ — mọi thứ trên một
///    giây là một giao diện trông như đã chết.
/// 2. **Chờ lâu cũng không giúp gì.** Ca thật của `index.lock` không phải một lệnh
///    git chạy 20 ms; nó là người dùng đang `git rebase --interactive` ở terminal với
///    một cửa sổ editor đang mở. Lock đó sống hàng phút. Thử lại chỉ có tác dụng cho
///    ca *đua ngắn* (một lệnh git khác đang chạy và sắp xong), và ca đó xong trong
///    vài trăm mili giây. Ngoài khoảng đó thì câu trả lời đúng là **nói cho người
///    dùng biết**, không phải chờ thêm.
const GIAN_CACH_MS: &[u64] = &[50, 150, 450];

/// Tra một repo đang mở theo `repo_id`.
///
/// Sao lại `repo_cua` của `diff.rs`/`history.rs`, và cùng lý do: `get_repo` chỉ trả
/// repo **đã mở** qua `open_repository`, nên một `repo_id` bịa từ webview cho
/// [`GitError::UnknownRepository`] chứ không mở được thư mục tuỳ ý. Đường dẫn không
/// bao giờ đến từ webview.
pub(crate) fn repo_cua(state: &AppState, repo_id: &str) -> Result<Arc<RepoHandle>> {
    state
        .get_repo(repo_id)
        .ok_or_else(|| GitError::UnknownRepository(repo_id.to_owned()))
}

/// Mốc đánh dấu vị trí một pathspec trong argv của một lệnh ghi.
///
/// Hàm đồng nhất — nó **không** biến đổi gì. Nó tồn tại để cổng
/// [`tests::moi_lenh_ghi_co_pathspec_deu_co_dau_gach_ngang_truoc_path`] khẳng định được
/// *thứ tự* của `--` và pathspec bằng cách tìm một chuỗi ổn định trong mã nguồn.
///
/// Cùng mẫu (và cùng tên) với `PATHSPEC_SAU_DAU_GACH` của [`super::diff`]. Khai lại ở
/// đây chứ không dùng chung: cổng của `diff.rs` đọc `include_str!("diff.rs")` và cắt
/// theo thân hàm của **tệp đó**, nên nó **không** phủ được lệnh nằm ở tệp này. Hai
/// mốc cùng tên ở hai tệp là hai cổng độc lập, đúng như plan yêu cầu.
///
/// Neo vào tên biến `path` không được: chữ đó xuất hiện khắp tệp (kể cả `repo.path`),
/// nên phép `find` sẽ khớp một chỗ khác và cổng trở thành vô dụng — bài học HIST-10.
#[allow(non_snake_case)]
fn PATHSPEC_SAU_DAU_GACH(path: &str) -> &str {
    path
}

/// stderr của git có nói rằng `.git/index.lock` đang bị giữ không?
///
/// # Nhận biết bằng stderr, KHÔNG bằng `std::fs::exists`
///
/// Kiểm sự tồn tại của tệp lock là một **cuộc đua**: giữa lúc ta `exists()` và lúc git
/// mở tệp, tiến trình kia có thể đã thả (ta báo lock trong khi lệnh sẽ thành công) hoặc
/// vừa lấy (ta báo không lock rồi lệnh vẫn thất bại). stderr là câu trả lời của **chính
/// git** về lần thử **thật** vừa rồi, nên nó không đua với gì cả.
///
/// Phép so khớp bao cả hai cách git diễn đạt, đã đọc từ `lockfile.c` và đo trên
/// git 2.54.0.windows.1:
///
/// ```text
/// fatal: Unable to create '<...>/.git/index.lock': File exists.
/// fatal: Unable to write new index file
/// ```
///
/// Chỉ khớp `index.lock` — `shallow.lock`, `config.lock`, `packed-refs.lock` là những
/// ca khác và không được lẫn vào đây, vì thông báo cho người dùng sẽ sai.
fn stderr_noi_ve_index_lock(stderr: &str) -> bool {
    stderr.contains("index.lock")
}

/// Chạy một lệnh **ghi** và thử lại khi gặp `.git/index.lock`.
///
/// Dùng chung cho [`stage_duong_dan`] và [`unstage_duong_dan`] — hai bản sao của phép
/// giãn cách sẽ lệch nhau ngay lần đầu ai đó chỉnh một con số.
///
/// # 🔴 KHÔNG BAO GIỜ xoá tệp lock — ràng buộc, không phải lựa chọn
///
/// `CONTEXT.md` 2.3 và rủi ro R3:
///
/// > Tự xoá `index.lock` là cách làm hỏng repo khi có một tiến trình git khác đang
/// > chạy thật — người dùng có thể đang `git rebase` ở terminal.
///
/// Cụ thể hơn: `index.lock` là *chính* cơ chế loại trừ lẫn nhau của git. Xoá nó không
/// "giải phóng" gì; nó **vô hiệu hoá** phép loại trừ, và tiến trình kia sẽ ghi index
/// dựa trên một trạng thái đã cũ. Kết quả là một index hỏng ở giữa một `rebase` —
/// đúng ca mà người dùng mất việc và không có đường lùi.
///
/// Chủ dự án **dùng terminal song song**; đó là cả điểm của WORK-10. Nên ca này không
/// phải phòng xa, nó là đường đi thường ngày.
///
/// Có test ghim trên đĩa (`index_lock_ton_tai_thi_that_bai_va_lock_van_con`) rằng tệp
/// lock **vẫn còn** sau khi lời gọi thất bại.
///
/// # Thất bại vì lock cho [`GitError::IndexLocked`], không phải `CommandFailed`
///
/// Xem tài liệu của variant đó về lý do.
pub(crate) async fn chay_lenh_ghi_co_thu_lai(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    dung_lenh: impl Fn() -> GitCommand,
) -> Result<GitOutput> {
    chay_lenh_ghi_co_thu_lai_phan_loai(state, repo, dung_lenh, |args, out| {
        GitError::CommandFailed {
            args,
            status: out.status,
            stderr: out.stderr_lossy(),
        }
    })
    .await
}

/// Như [`chay_lenh_ghi_co_thu_lai`], nhưng người gọi tự **phân loại** thất bại.
///
/// # Vì sao tham số này tồn tại — plan 04-03
///
/// Phép thử lại `index.lock` và phép **phân loại lỗi** là hai việc khác nhau, và bản
/// đầu của hàm này trộn chúng: nó tự trả [`GitError::CommandFailed`] cho **mọi** thất
/// bại không phải lock. Với `git add` thì đúng — exit khác 0 ở đó gần như chỉ có một
/// nghĩa. Với `git commit` thì **sai**: exit 1 ở đó là hook từ chối, hoặc không có gì
/// để commit, hoặc thông điệp rỗng — ba thông báo hoàn toàn khác nhau cho người dùng.
///
/// 🔴 Đây đúng là khuôn lỗi KB-4b của `open_repository` (gán **một** nguyên nhân cho
/// **mọi** exit khác 0), và nó được phát hiện bằng một test đỏ chứ không bằng suy luận:
/// `create_commit` trả `command_failed` trong khi test đòi `hook_rejected`.
///
/// Đường thử lại **không** bị sao chép — plan nói rõ *"dùng cùng hàm thử-lại của 04-02,
/// không viết lại"*. Chỉ phép phân loại được tiêm vào.
pub(crate) async fn chay_lenh_ghi_co_thu_lai_phan_loai(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    dung_lenh: impl Fn() -> GitCommand,
    phan_loai: impl Fn(Vec<String>, &GitOutput) -> GitError,
) -> Result<GitOutput> {
    let runner = state.runner(Arc::clone(repo));

    for lan in 0..SO_LAN_THU {
        let cmd = dung_lenh();
        let args: Vec<String> = cmd.display_args().to_vec();

        // `runner.write` giữ khoá ghi của repository suốt thời gian chạy (PLAT-03).
        // Khoá được lấy **lại** ở mỗi lần thử, không giữ xuyên qua các lần chờ: giữ
        // qua `sleep` sẽ chặn một lệnh ghi khác của chính ứng dụng trong 650 ms mà
        // không có lý do — lock ta đang chờ là của một tiến trình **bên ngoài**.
        let out = runner.write(cmd).await?;

        if out.is_success() {
            return Ok(out);
        }

        // 🔴 Nhận biết lock phải đọc **cả hai** luồng, không chỉ stderr.
        //
        // `git commit` in một số thông báo ra **stdout** (đo được: "nothing to commit"),
        // và tuy câu lock của git nằm ở stderr, phép ghép hai luồng ở đây làm phép nhận
        // biết không phụ thuộc vào việc lệnh con chọn luồng nào — thứ không có gì bảo
        // đảm là bất biến qua các lệnh git và các phiên bản.
        let stderr = out.stderr_lossy();
        let stdout_loi = String::from_utf8_lossy(&out.stdout);
        let la_lock = stderr_noi_ve_index_lock(&stderr) || stderr_noi_ve_index_lock(&stdout_loi);

        if !la_lock {
            // Không phải ca lock: thất bại ngay, đừng thử lại. Thử lại một lệnh sai
            // tham số chỉ làm người dùng chờ 650 ms để nhận cùng một lỗi.
            return Err(phan_loai(args, &out));
        }

        match GIAN_CACH_MS.get(lan) {
            // Còn lần thử nữa: chờ rồi thử lại. **Không** chạm tới tệp lock.
            Some(&ms) => tokio::time::sleep(Duration::from_millis(ms)).await,
            // Hết lần thử. Trả lỗi có mã riêng, giữ stderr nguyên văn.
            None => return Err(GitError::IndexLocked { args, stderr }),
        }
    }

    // Không tới được: vòng lặp trên luôn `return` ở lần thử cuối vì `GIAN_CACH_MS` có
    // đúng `SO_LAN_THU - 1` phần tử. Khẳng định điều đó để một lần chỉnh hằng số
    // không âm thầm đổi hành vi thành "im lặng thành công".
    debug_assert_eq!(GIAN_CACH_MS.len(), SO_LAN_THU - 1);
    Err(GitError::IndexLocked {
        args: Vec::new(),
        stderr: String::new(),
    })
}

/// Trạng thái thư mục làm việc — WORK-01.
///
/// Tách khỏi `#[tauri::command]` để test tích hợp gọi được mà không cần dựng một `App`
/// có webview — cùng cách [`super::diff::lay_diff_tep`] và
/// [`super::diff::lay_lich_su_tep`] làm.
///
/// # Lệnh **đọc**: không giữ khoá ghi
///
/// Đi qua `runner.read`, không `runner.write`. Giữ khoá ghi ở đường đọc làm mọi lần
/// làm mới giao diện xếp hàng sau một lệnh ghi đang chạy, và watcher của 04-04 (vốn
/// đọc trạng thái sau **mỗi** sự kiện) sẽ bị chặn bởi chính thao tác ghi đã sinh ra
/// sự kiện đó. Có test ghim.
///
/// # Đối số lấy từ [`STATUS_ARGS`], không viết lại ở đây
///
/// Hai nguồn là hai nguồn lệch nhau: bộ phân tích của 04-01 được viết cho **đúng**
/// tập cờ đó, và test của nó khẳng định từng cờ theo tên. Viết lại danh sách ở đây
/// nghĩa là bỏ `--untracked-files=all` ở một chỗ mà không test nào thấy.
pub async fn lay_trang_thai(state: &AppState, repo: Arc<RepoHandle>) -> Result<RepoStatus> {
    let runner = state.runner(Arc::clone(&repo));

    let ra = runner
        .read(
            GitCommand::new(&repo.path)
                // `core.quotepath=false` để tên tệp ngoài ASCII về ở dạng byte thô
                // thay vì escape bát phân — bộ phân tích giải mã lossy nên nó cần
                // byte thật (HIST-11, và `-z` đã tắt trích dẫn nhưng cờ này vô hại
                // và giữ khuôn chung với các lệnh khác).
                .args(["-c", "core.quotepath=false"])
                .args(STATUS_ARGS)
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    if !ra.is_success() {
        return Err(phan_loai_loi_status(&repo.path.to_string_lossy(), &ra));
    }

    Ok(parse_status(&ra.stdout))
}

/// Biến một `git status` thất bại thành lỗi có kiểu, phân loại theo **stderr**.
///
/// Xem ghi chú đầu module về lý do không phân loại theo mã thoát. Mọi nhánh đều giữ
/// stderr nguyên văn: người đọc thông báo cần nó, và phép so khớp của ta có thể trượt.
fn phan_loai_loi_status(duong_dan_repo: &str, ra: &GitOutput) -> GitError {
    let stderr = ra.stderr_lossy();

    // Ca lock: `git status` cũng đụng lock khi nó cần làm mới index.
    if stderr_noi_ve_index_lock(&stderr) {
        return GitError::IndexLocked {
            args: STATUS_ARGS.iter().map(|s| (*s).to_owned()).collect(),
            stderr,
        };
    }

    // Ca "thật sự không phải repo". Hẹp có chủ ý: chỉ khi git **nói** như vậy.
    //
    // 🔴 `dubious ownership` cũng thoát 128 và **không** được rơi vào đây — đó đúng là
    // lỗi KB-4b của `open_repository`. Nó đi xuống nhánh cuối, nơi người dùng thấy
    // stderr của git (vốn nói rõ phải chạy `git config --global --add
    // safe.directory <...>`) thay vì một câu sai.
    if stderr.contains("not a git repository") || stderr.contains("Not a git repository") {
        return GitError::NotARepository {
            path: duong_dan_repo.to_owned(),
        };
    }

    // Mọi nguyên nhân còn lại: repo bị từ chối, index hỏng, quyền truy cập, và những
    // ca ta chưa biết. Giữ đủ thông tin để người dùng gửi được một báo cáo dùng được.
    GitError::CommandFailed {
        args: STATUS_ARGS.iter().map(|s| (*s).to_owned()).collect(),
        status: ra.status,
        stderr,
    }
}

/// Stage các tệp theo đường dẫn — WORK-02.
///
/// Trả [`RepoStatus`] **mới** (xem ghi chú đầu module về lý do không trả `()`).
///
/// # `paths` rỗng → **không lệnh git nào**, trả trạng thái hiện tại
///
/// 🔴 `git add` **không** pathspec stage **cả cây**. Một mảng rỗng tới đây và chạy
/// lệnh nghĩa là người dùng bấm stage cho một tệp và nhận cả repo trong index — một
/// lỗi phá hoại, không phải một lỗi nhìn-là-thấy, vì giao diện sau đó **vẫn đúng**
/// (nó hiện cái đang có trong index thật).
///
/// Và mảng rỗng là đầu vào **có thật**: nó đến từ webview. Một vòng lặp chọn tệp sai,
/// một lời gọi trước khi status nạp xong, hay một `filter` không khớp gì đều cho nó.
/// Nên phép chặn nằm **trước** khi dựng lệnh, không phải một `if` bên trong lệnh.
///
/// Có test đo bằng `CommandLog` — giá trị trả về **không** phân biệt được ca này, vì
/// git từ chối pathspec rỗng nên "chạy rồi thất bại" cũng cho đúng status.
pub async fn stage_duong_dan(
    state: &AppState,
    repo: Arc<RepoHandle>,
    paths: &[String],
) -> Result<RepoStatus> {
    if paths.is_empty() {
        return lay_trang_thai(state, repo).await;
    }

    let duong_dan = paths.to_vec();
    let repo_path = repo.path.clone();

    chay_lenh_ghi_co_thu_lai(state, &repo, || {
        let mut cmd = GitCommand::new(&repo_path)
            .args(["add", "--verbose"])
            // 🔴 `--` NGĂN CÁCH cờ/revision với pathspec (T-02-11, T-03-07). Thiếu nó
            // thì một tệp tên `-rf` được git đọc thành **cờ**, và một tệp tên `main`
            // (trùng tên nhánh) được đọc thành **revision**. `paths` đến từ webview,
            // tức chuỗi tuỳ ý — cả hai ca đều có test.
            .arg("--");
        for p in &duong_dan {
            cmd = cmd.arg(PATHSPEC_SAU_DAU_GACH(p));
        }
        cmd.timeout(DEFAULT_TIMEOUT)
    })
    .await?;

    lay_trang_thai(state, repo).await
}

/// Bỏ stage các tệp theo đường dẫn — WORK-02.
///
/// Trả [`RepoStatus`] mới, cùng lý do với [`stage_duong_dan`].
///
/// # Vì sao `git restore --staged`, không phải `git reset HEAD --`
///
/// Hai lệnh cho cùng kết quả ở ca thường, nhưng khác nhau ở ca **repo chưa có commit
/// nào**: `git reset HEAD -- <path>` thất bại vì `HEAD` chưa phân giải được, còn
/// `git restore --staged -- <path>` chạy đúng (nó lùi index về cây rỗng). Một repo mới
/// `git init` rồi `git add` là ca đầu tiên người dùng gặp khi thử ứng dụng, nên nó
/// không được là ca hỏng.
///
/// `restore` cũng là lệnh git **khuyến nghị** cho việc này từ 2.23 — `reset` mang quá
/// nhiều nghĩa khác và một lần gõ thiếu pathspec sẽ xoá cả index.
pub async fn unstage_duong_dan(
    state: &AppState,
    repo: Arc<RepoHandle>,
    paths: &[String],
) -> Result<RepoStatus> {
    if paths.is_empty() {
        return lay_trang_thai(state, repo).await;
    }

    let duong_dan = paths.to_vec();
    let repo_path = repo.path.clone();

    chay_lenh_ghi_co_thu_lai(state, &repo, || {
        let mut cmd = GitCommand::new(&repo_path)
            .args(["restore", "--staged"])
            // 🔴 `--` TRƯỚC pathspec — xem ghi chú ở `stage_duong_dan`. Ở `restore` ca
            // này còn nặng hơn: thiếu `--`, một tệp tên `main` làm git hiểu là
            // "restore từ nhánh main", tức nó đọc một **nguồn** khác.
            .arg("--");
        for p in &duong_dan {
            cmd = cmd.arg(PATHSPEC_SAU_DAU_GACH(p));
        }
        cmd.timeout(DEFAULT_TIMEOUT)
    })
    .await?;

    lay_trang_thai(state, repo).await
}

/// Trạng thái thư mục làm việc — WORK-01.
#[tauri::command]
pub async fn get_status(repo_id: String, state: State<'_, AppState>) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    lay_trang_thai(&state, repo).await
}

/// Stage các tệp — WORK-02. Trả trạng thái **mới** (CONTEXT.md 2.5).
#[tauri::command]
pub async fn stage_files(
    repo_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    stage_duong_dan(&state, repo, &paths).await
}

/// Bỏ stage các tệp — WORK-02. Trả trạng thái **mới** (CONTEXT.md 2.5).
#[tauri::command]
pub async fn unstage_files(
    repo_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    unstage_duong_dan(&state, repo, &paths).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thân phần **không-test** của tệp này, đã **bỏ dòng chú thích**.
    ///
    /// # Vì sao bỏ chú thích là bắt buộc, không phải cẩn thận thừa
    ///
    /// Cổng thứ **năm** trong bảy cổng không-thể-fail của dự án (CONTEXT.md 3.1) chết
    /// đúng vì điều này: nó grep trên nguồn **thô** và khớp một chú thích **do chính
    /// mutation sinh ra**. Mọi doc comment trong tệp này nói về `--` và về
    /// `PATHSPEC_SAU_DAU_GACH`; đọc nguồn thô thì cổng khớp văn xuôi và xanh vĩnh viễn.
    ///
    /// Cắt ở `#[cfg(test)]` để không đọc chính mã test — test có `"--"` trong chuỗi
    /// khẳng định.
    fn ma_khong_chu_thich() -> String {
        let src = include_str!("worktree.rs");
        src.lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// **`--` phải đứng TRƯỚC pathspec trong MỌI lệnh ghi của tệp này.**
    ///
    /// Đọc thân hàm **thật**, không dựng lệnh git riêng — bài học HIST-10: một test tự
    /// dựng lệnh chứng minh `--` *có tác dụng* mà không chứng minh hàm thật *dùng* nó,
    /// và Phase 2 đã đo rằng nó cho 0 test đỏ khi xoá `.arg("--")`.
    ///
    /// # Cổng của `diff.rs` KHÔNG phủ được tệp này
    ///
    /// `moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path` đọc
    /// `include_str!("diff.rs")` và cắt theo thân hàm của **tệp đó**. `stage_files` và
    /// `unstage_files` sống ở đây, nên chúng cần cổng riêng cùng khuôn — plan 04-02
    /// nói rõ điều này.
    ///
    /// # Kiểm trong phạm vi TỪNG LỆNH, không kiểm cả tệp như một khối
    ///
    /// Đọc khối comment "Bản đầu của cổng này VÔ DỤNG" trong `diff.rs` trước khi sửa
    /// test này. Tóm tắt: phép tìm vị trí `--` **đầu tiên** rồi đòi mọi mốc pathspec
    /// nằm sau nó cho **0 test đỏ** khi xoá `--` của một lệnh cụ thể, vì `--` của một
    /// lệnh khác phía trên vẫn còn và vẫn nằm trước mọi mốc. Một đại lượng **toàn cục**
    /// không đo được một bất biến **cục bộ**.
    #[test]
    fn moi_lenh_ghi_co_pathspec_deu_co_dau_gach_ngang_truoc_path() {
        let ma = ma_khong_chu_thich();

        // 🔴 Khẳng định **tiền đề** trước: không tìm thấy thứ cần kiểm thì cổng phải
        // ĐỎ, không phải xanh. Cổng thứ ba của CONTEXT.md 3.1 chết vì đường dẫn tệp
        // sai → grep không thấy gì → xanh.
        assert!(
            ma.contains("GitCommand::new"),
            "tiền đề: phép lọc chú thích phải giữ lại được thân hàm thật"
        );

        let so_moc = ma.matches("PATHSPEC_SAU_DAU_GACH").count();
        assert_eq!(
            so_moc, 3,
            "ba vị trí mốc trong mã không-chú-thích: khai báo hàm `fn \
             PATHSPEC_SAU_DAU_GACH`, chỗ gọi trong `stage_duong_dan`, và chỗ gọi trong \
             `unstage_duong_dan`. Số khác 3 nghĩa là một lệnh mất mốc, hoặc có lệnh \
             mới mang pathspec mà chưa được cổng này kiểm. Đọc được {so_moc}"
        );

        // Cắt theo `GitCommand::new` — mỗi lần xuất hiện là một lệnh mới. Phần tử đầu
        // (trước lần xuất hiện đầu tiên) bị `skip(1)` bỏ đi vì nó không phải một lệnh.
        let cac_lenh: Vec<&str> = ma.split("GitCommand::new").skip(1).collect();
        assert_eq!(
            cac_lenh.len(),
            3,
            "ba lệnh git trong tệp này: `status` (đọc, KHÔNG pathspec), `add`, và \
             `restore --staged`. Đọc được {}",
            cac_lenh.len()
        );

        let mut so_lenh_co_pathspec = 0;
        for (i, lenh) in cac_lenh.iter().enumerate() {
            if !lenh.contains("PATHSPEC_SAU_DAU_GACH") {
                continue;
            }
            so_lenh_co_pathspec += 1;

            let gach = lenh.find(r#""--""#).unwrap_or_else(|| {
                panic!(
                    "lệnh git thứ {} mang pathspec nhưng KHÔNG có `--` ngăn cách nó \
                     với cờ/revision. Thiếu `--`: một tệp tên `-rf` bị git đọc thành \
                     CỜ, và một tệp tên `main` bị đọc thành REVISION. `paths` đến từ \
                     webview nên nó là chuỗi tuỳ ý.\n\nLệnh:\n{lenh}",
                    i + 1
                )
            });
            let moc = lenh
                .find("PATHSPEC_SAU_DAU_GACH")
                .expect("vừa khẳng định là có");
            assert!(
                gach < moc,
                "lệnh git thứ {} đặt `--` SAU pathspec — đặt sau thì nó không ngăn \
                 cách gì cả.\n\nLệnh:\n{lenh}",
                i + 1
            );
        }

        assert_eq!(
            so_lenh_co_pathspec, 2,
            "đúng HAI lệnh mang pathspec: `git add` và `git restore --staged`. Lệnh \
             `status` cố ý không mang (nó đọc cả cây). Con số khác 2 nghĩa là phép cắt \
             theo `GitCommand::new` đã lệch và vòng lặp trên không còn kiểm đúng thứ \
             nó tưởng"
        );
    }

    /// **Lệnh ghi KHÔNG BAO GIỜ xoá tệp lock** — ghim ở mức mã nguồn.
    ///
    /// Test tích hợp `index_lock_ton_tai_thi_that_bai_va_lock_van_con` đo trên đĩa và
    /// là cổng chính. Cổng này là lớp thứ hai, rẻ, và nó bắt một thứ khác: một đường
    /// xoá lock nằm ở nhánh mà test tích hợp chưa đi qua (ví dụ chỉ xoá sau lần thử
    /// thứ tư) vẫn bị bắt ở đây.
    #[test]
    fn khong_co_duong_nao_xoa_tep_lock() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("index.lock"),
            "tiền đề: tệp này phải nói về index.lock, nếu không cổng đang đọc sai chỗ"
        );

        for cam in ["remove_file", "remove_dir", "fs::remove"] {
            assert!(
                !ma.contains(cam),
                "🔴 tìm thấy `{cam}` trong mã: KHÔNG BAO GIỜ được xoá \
                 `.git/index.lock`. Nó là chính cơ chế loại trừ của git; xoá nó làm \
                 tiến trình kia ghi index dựa trên trạng thái cũ và làm hỏng repo của \
                 người dùng đang `git rebase` (CONTEXT.md 2.3, R3)"
            );
        }
    }

    /// Nhận biết lock bằng **stderr**, không bằng phép kiểm tệp tồn tại.
    ///
    /// Kiểm tệp là một cuộc đua (xem [`stderr_noi_ve_index_lock`]). Cổng này ghim việc
    /// không có ai lặng lẽ "sửa" nó thành `Path::exists`.
    #[test]
    fn nhan_biet_lock_khong_dung_phep_kiem_tep_ton_tai() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("stderr_noi_ve_index_lock"),
            "tiền đề: phải có hàm nhận biết lock theo stderr"
        );
        for cam in [".exists()", "try_exists", "is_file()"] {
            assert!(
                !ma.contains(cam),
                "🔴 tìm thấy `{cam}`: nhận biết lock bằng sự tồn tại của tệp là một \
                 CUỘC ĐUA — giữa lúc ta kiểm và lúc git mở tệp, tiến trình kia có thể \
                 đã thả hoặc vừa lấy. stderr là câu trả lời của chính git về lần thử \
                 thật vừa rồi"
            );
        }
    }

    /// Phép so khớp stderr nhận đúng câu git in ra, và **không** nhận lock khác.
    #[test]
    fn stderr_lock_nhan_dung_cau_cua_git() {
        assert!(stderr_noi_ve_index_lock(
            "fatal: Unable to create '/r/.git/index.lock': File exists."
        ));
        assert!(stderr_noi_ve_index_lock(
            "error: could not lock config file .git/index.lock"
        ));

        assert!(
            !stderr_noi_ve_index_lock("fatal: not a git repository"),
            "thư mục không phải repo KHÔNG phải ca lock"
        );
        assert!(
            !stderr_noi_ve_index_lock("fatal: detected dubious ownership in repository at '/r'"),
            "🔴 `dubious ownership` KHÔNG phải ca lock — gộp chúng lại là lặp lại đúng \
             lỗi KB-4b của open_repository"
        );
        assert!(
            !stderr_noi_ve_index_lock("fatal: Unable to create '/r/.git/shallow.lock'"),
            "lock KHÁC (shallow/config/packed-refs) không được lẫn vào ca index.lock: \
             thông báo cho người dùng sẽ sai"
        );
        assert!(!stderr_noi_ve_index_lock(""), "stderr rỗng không phải lock");
    }

    /// Tổng thời gian chờ có **chặn trên** dưới một giây, và số lần thử khớp số giãn cách.
    ///
    /// Đọc **giá trị hằng**, không grep: hằng số xuất hiện ở khai báo, trong chú thích
    /// giải thích con số, và trong test — `grep -c` sẽ đếm cả ba (cổng thứ nhất của
    /// CONTEXT.md 3.1).
    #[test]
    fn tong_thoi_gian_cho_lock_co_chan_tren_duoi_mot_giay() {
        assert_eq!(
            GIAN_CACH_MS.len(),
            SO_LAN_THU - 1,
            "n lần thử cần n-1 lần chờ; lệch nghĩa là lần thử cuối vẫn chờ rồi mới \
             trả lỗi, hoặc có lần thử không bao giờ chạy"
        );

        let tong: u64 = GIAN_CACH_MS.iter().sum();
        assert!(
            tong < 1_000,
            "tổng thời gian chờ phải dưới 1 giây — giao diện không được treo, và nếu \
             người dùng đang rebase thật thì chờ lâu cũng không giúp gì. Tổng: {tong}ms"
        );
        assert!(
            tong >= 100,
            "phải có chờ thật: giãn cách 0 ms nghĩa là bốn lần thử trong cùng một \
             micro giây, và ca đua ngắn mà việc thử lại tồn tại để giải sẽ không bao \
             giờ thắng. Tổng: {tong}ms"
        );

        // Giãn cách phải **tăng**: chờ đều đặn không phải giãn cách, và plan yêu cầu
        // "giãn cách theo số lần".
        for cua_so in GIAN_CACH_MS.windows(2) {
            assert!(
                cua_so[1] > cua_so[0],
                "giãn cách phải tăng dần, thấy {GIAN_CACH_MS:?}"
            );
        }
    }

    /// `STATUS_ARGS` được **dùng lại**, không viết lại danh sách cờ trong tệp này.
    ///
    /// Hai nguồn là hai nguồn lệch nhau: bộ phân tích của 04-01 chỉ đúng cho đúng tập
    /// cờ đó, và test khẳng định từng cờ theo tên sống ở `domain/status.rs`. Một danh
    /// sách thứ hai ở đây nghĩa là bỏ `--untracked-files=all` mà không test nào thấy.
    #[test]
    fn dung_lai_status_args_khong_viet_lai_danh_sach_co() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("STATUS_ARGS"),
            "tiền đề: lệnh status phải đi qua hằng STATUS_ARGS"
        );
        assert!(
            !ma.contains("--porcelain"),
            "🔴 tìm thấy `--porcelain` viết thẳng trong tệp này: tập cờ của lệnh \
             trạng thái chỉ được khai ở `domain::STATUS_ARGS`. Hai nguồn sẽ lệch nhau."
        );
        assert!(
            !ma.contains("--untracked-files"),
            "🔴 tìm thấy `--untracked-files` viết thẳng: xem trên"
        );
    }

    /// Gộp mọi chuỗi khoảng trắng thành **không có gì**.
    ///
    /// # Vì sao cần: `rustfmt` ngắt dòng giữa người nhận và tên phương thức
    ///
    /// Đây là một cổng không-thể-fail mà tôi **đã gặp thật** khi viết tệp này, và nó
    /// đáng ghi lại vì nó đi theo hướng ngược với sáu cổng trước: bản đầu tìm chuỗi
    /// `"runner.read"` và **đỏ** dù mã hoàn toàn đúng, vì `rustfmt` viết
    ///
    /// ```text
    /// let ra = runner
    ///     .read(
    /// ```
    ///
    /// tức chuỗi `runner.read` **không tồn tại** trong tệp. Lần này phép khẳng định
    /// tiền đề đã bắt được nó ("tiền đề: lay_trang_thai phải gọi runner") thay vì để
    /// cổng xanh — đúng công dụng mà CONTEXT.md 3.1 đòi.
    ///
    /// Cách sai để sửa: đổi mã nguồn cho vừa cổng (viết một dòng, thêm
    /// `#[rustfmt::skip]`). Làm vậy là để một cổng quyết định cách định dạng mã, và
    /// lần `cargo fmt` sau sẽ phá lại. Cách đúng: **cổng phải đo bất biến, không đo
    /// cách trình bày** — nên ta xoá hết khoảng trắng trước khi tìm.
    fn khong_khoang_trang(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    /// Đường **đọc** dùng `runner.read`; đường **ghi** dùng `runner.write` (PLAT-03).
    ///
    /// Test tích hợp đo hành vi thật bằng cách giữ khoá trước rồi `timeout` — đó là
    /// cổng chính. Cổng này ghim việc `lay_trang_thai` **không** lặng lẽ đổi sang
    /// `write` trong một lần "cho nhất quán", vì đổi vậy làm mọi lần làm mới giao diện
    /// xếp hàng sau lệnh ghi.
    #[test]
    fn duong_doc_dung_read_duong_ghi_dung_write() {
        let ma = ma_khong_chu_thich();

        let (_, than_status) = ma
            .split_once("pub async fn lay_trang_thai")
            .expect("phải có hàm lay_trang_thai");
        let than_status = khong_khoang_trang(
            &than_status
                .lines()
                .take_while(|l| !l.starts_with("fn phan_loai_loi_status"))
                .collect::<Vec<_>>()
                .join("\n"),
        );

        assert!(
            than_status.contains("runner.read("),
            "tiền đề: `lay_trang_thai` phải gọi `runner.read(` — không tìm thấy nghĩa \
             là cổng này đang đọc sai phạm vi, và nó phải ĐỎ chứ không được xanh"
        );
        assert!(
            !than_status.contains("runner.write("),
            "🔴 `lay_trang_thai` là lệnh ĐỌC — nó không được giữ khoá ghi. Giữ khoá ở \
             đây làm watcher của 04-04 bị chặn bởi chính thao tác ghi đã sinh ra sự \
             kiện nó đang xử lý"
        );

        let (_, than_ghi) = ma
            .split_once("async fn chay_lenh_ghi_co_thu_lai")
            .expect("phải có hàm chạy lệnh ghi");
        let than_ghi = khong_khoang_trang(
            &than_ghi
                .lines()
                .take_while(|l| !l.starts_with("pub async fn lay_trang_thai"))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        assert!(
            than_ghi.contains("runner.write("),
            "🔴 đường ghi phải đi qua `runner.write(` để giữ khoá ghi (PLAT-03)"
        );
        assert!(
            !than_ghi.contains("runner.read("),
            "đường ghi không được lặng lẽ đổi sang `read` — nó sẽ mất khoá ghi"
        );
    }

    /// Phân loại lỗi status: `dubious ownership` **không** thành "không phải repo".
    ///
    /// Đây là ca KB-4b của `open_repository`, ghim để mã mới không lặp lại nó.
    #[test]
    fn dubious_ownership_khong_bi_bao_thanh_khong_phai_repo() {
        let ra = GitOutput {
            stdout: Vec::new(),
            stderr: b"fatal: detected dubious ownership in repository at 'C:/r'\n\
                      To add an exception for this directory, call:\n\
                      \tgit config --global --add safe.directory C:/r"
                .to_vec(),
            status: 128,
        };

        let loi = phan_loai_loi_status("C:/r", &ra);

        assert_ne!(
            loi.code(),
            "not_a_repository",
            "🔴 `dubious ownership` là một repo HỢP LỆ mà git từ chối. Báo nó thành \
             'Không phải một repository git' là đúng lỗi KB-4b của open_repository, và \
             người dùng không có cách nào biết phải chạy `git config safe.directory`"
        );
        assert!(
            loi.to_string().contains("safe.directory"),
            "stderr NGUYÊN VĂN phải đi kèm — nó chứa đúng lệnh người dùng cần chạy. \
             Nhận được: {loi}"
        );
    }

    /// Phân loại lỗi status: git **nói** không phải repo thì trả đúng mã đó.
    #[test]
    fn khong_phai_repo_that_thi_tra_not_a_repository() {
        let ra = GitOutput {
            stdout: Vec::new(),
            stderr: b"fatal: not a git repository (or any of the parent directories): .git"
                .to_vec(),
            status: 128,
        };

        assert_eq!(
            phan_loai_loi_status("C:/tmp", &ra).code(),
            "not_a_repository"
        );
    }

    /// Phân loại lỗi status: lock trong lúc **đọc** cũng cho mã `index_locked`.
    ///
    /// `git status` làm mới index nên nó cũng đụng lock — người dùng bấm làm mới trong
    /// lúc rebase sẽ gặp ca này, và họ cần cùng một câu giải thích.
    #[test]
    fn lock_trong_luc_doc_status_cung_cho_ma_index_locked() {
        let ra = GitOutput {
            stdout: Vec::new(),
            stderr: b"fatal: Unable to create 'C:/r/.git/index.lock': File exists.".to_vec(),
            status: 128,
        };

        assert_eq!(phan_loai_loi_status("C:/r", &ra).code(), "index_locked");
    }

    /// Nguyên nhân **chưa biết** rơi về `command_failed`, giữ mã thoát và stderr.
    ///
    /// Đây là nhánh quan trọng nhất của phép phân loại: nó là chỗ ta thừa nhận rằng
    /// stderr là văn bản mờ và phép so khớp của ta sẽ trượt.
    #[test]
    fn nguyen_nhan_chua_biet_roi_ve_command_failed_giu_stderr() {
        let ra = GitOutput {
            stdout: Vec::new(),
            stderr: b"fatal: mot loi hoan toan moi chua ai thay".to_vec(),
            status: 129,
        };

        let loi = phan_loai_loi_status("C:/r", &ra);
        assert_eq!(loi.code(), "command_failed");
        assert!(
            loi.to_string().contains("mot loi hoan toan moi"),
            "stderr nguyên văn phải đi kèm ở MỌI nhánh: {loi}"
        );
        assert!(
            loi.to_string().contains("129"),
            "mã thoát phải đi kèm để báo cáo lỗi dùng được: {loi}"
        );
    }

    /// Lệnh ghi trả `RepoStatus`, **không** trả `()` — ghim ở mức chữ ký.
    ///
    /// 🔴 Đột biến M1 đỏ **lúc biên dịch** ở test tích hợp, và đó là đủ theo plan.
    /// Cổng này thêm một lớp đọc được bằng mắt trong chính tệp: nó bắt cả ca ai đó đổi
    /// chữ ký **và** sửa test tích hợp cho khớp.
    #[test]
    fn lenh_ghi_tra_repostatus_khong_tra_unit() {
        let ma = ma_khong_chu_thich();

        for ten in [
            "stage_duong_dan",
            "unstage_duong_dan",
            "stage_files",
            "unstage_files",
        ] {
            let (_, sau) = ma
                .split_once(&format!("fn {ten}("))
                .unwrap_or_else(|| panic!("phải có hàm {ten}"));
            let chu_ky: String = sau.lines().take(8).collect::<Vec<_>>().join(" ");
            assert!(
                chu_ky.contains("Result<RepoStatus>"),
                "🔴 `{ten}` phải trả `Result<RepoStatus>`, không `Result<()>`. Một lệnh \
                 ghi trả `()` BUỘC giao diện chờ watcher: trễ 250–300 ms mỗi cú bấm, và \
                 nếu watcher chết thì giao diện đứng im mà không ai biết (CONTEXT.md \
                 2.5). Chữ ký đọc được: {chu_ky}"
            );
        }
    }

    /// Cả ba command phải đăng ký trong `generate_handler!` của `lib.rs`.
    ///
    /// Một `#[tauri::command]` không đăng ký biên dịch **sạch** ở cả hai phía và chỉ
    /// thất bại lúc chạy với "command not found" — đúng lớp lỗi mà
    /// `ipc.status.test.ts` chặn ở phía TypeScript, chặn ở đây cho phía Rust.
    #[test]
    fn ba_command_da_dang_ky_trong_generate_handler() {
        let lib = include_str!("../lib.rs");
        let ma: String = lib
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            ma.contains("generate_handler!"),
            "tiền đề: lib.rs phải có generate_handler!"
        );
        for ten in ["get_status", "stage_files", "unstage_files"] {
            assert!(
                ma.contains(&format!("commands::{ten}")),
                "command `{ten}` chưa đăng ký trong generate_handler! của lib.rs — \
                 nó sẽ biên dịch sạch rồi thất bại lúc chạy với 'command not found'"
            );
        }
    }
}
