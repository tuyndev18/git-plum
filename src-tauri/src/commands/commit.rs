//! Tạo commit và sửa commit gần nhất — WORK-08, WORK-09.
//!
//! # Hook chạy MẶC ĐỊNH; `--no-verify` là công tắc tường minh
//!
//! ROADMAP nguyên văn, và đây là ràng buộc không thương lượng (CONTEXT.md 2.4):
//!
//! > Chạy `pre-commit` và `commit-msg` **mặc định**. Có công tắc `no-verify` **tường
//! > minh** ở giao diện. Đây là một trong những lý do dự án chọn `git` CLI thay vì
//! > libgit2.
//!
//! Bỏ hook là bỏ chính lý do tồn tại của lựa chọn kiến trúc đó. Nên ở đây có **đúng
//! một** đường thêm cờ `--no-verify`, và nó đứng sau một `if` trên tham số `no_verify`.
//! Có cổng đọc mã nguồn ghim rằng không có đường thứ hai.
//!
//! # Thông điệp đi qua stdin `-F -`, KHÔNG qua `-m`
//!
//! Ba lý do, và cả ba đều là lỗi thật chứ không phải sở thích:
//!
//! 1. **Thông điệp là dữ liệu tuỳ ý từ webview.** `-m` đặt nó vào **argv**, nơi dấu
//!    ngoặc kép, dấu xuống dòng và ký tự điều khiển bị từng nền tảng xử lý khác nhau.
//!    Windows dựng dòng lệnh thành **một chuỗi** rồi để tiến trình con tự tách, còn
//!    Unix truyền một mảng — nên cùng một thông điệp cho hai kết quả khác nhau.
//! 2. **`GitCommand` đã có [`GitCommand::stdin_bytes`]**, viết cho đúng loại việc này
//!    (bản vá của WORK-04 đi cùng đường). Không phải phát minh gì mới.
//! 3. **Thông điệp có thể chứa byte không phải UTF-8.** `Vec<u8>` đi thẳng qua stdin
//!    nguyên vẹn; một `String` trong argv thì không — nó phải qua `to_string_lossy` ở
//!    [`GitCommand::arg`] và byte hỏng bị thay bằng `U+FFFD` **vĩnh viễn**, ghi thẳng
//!    vào lịch sử repo của người dùng.
//!
//! # `--cleanup=whitespace` — ĐÃ ĐO, và phép đo đổi kết luận
//!
//! `docs/02-phase4-commit-notes.md` bàn giao ràng buộc này từ Phase 1: chế độ
//! `default` của git **xoá mọi dòng bắt đầu bằng `#`**, nên một người gõ `#123` ở đầu
//! dòng thấy dòng đó biến mất không một lời giải thích.
//!
//! Đo trên git 2.54.0.windows.1, và kết quả **hẹp hơn** ghi chú bàn giao:
//!
//! ```text
//! -F -, không cờ, không cấu hình     -> dòng `#123` CÒN   (mặc định của -F là whitespace)
//! -F -, không cờ, commit.cleanup=strip -> dòng `#123` MẤT  🔴
//! -F -, CÓ cờ,   commit.cleanup=strip -> dòng `#123` CÒN
//! ```
//!
//! Tức: khi thông điệp đến từ **tệp/stdin**, mặc định của git đã là `whitespace` và cờ
//! không đổi gì — **cho tới khi** người dùng đặt `commit.cleanup=strip` trong cấu hình
//! của họ. Lúc đó cờ là thứ duy nhất cứu được nội dung họ vừa gõ.
//!
//! Hệ quả cho **test**: một ca kiểm "dòng `#` còn nguyên" chạy trên repo cấu hình mặc
//! định **không phân biệt được** đột biến bỏ cờ — nó xanh ở cả hai bên. Đó đúng là lỗi
//! #7 của CONTEXT.md 3.1 (hình dạng đúng, dữ liệu vô hại). Nên test của ca này **đặt
//! `commit.cleanup=strip`** vào repo mẫu, để dữ liệu trở nên có hại và đột biến để lại
//! dấu vết quan sát được.
//!
//! # Phân loại lỗi đọc CẢ HAI luồng — đã đo, và đây là chỗ dễ sai nhất
//!
//! `git commit` thoát khác 0 vì **nhiều** nguyên nhân, và chúng **không** dùng chung
//! một luồng. Đo trên git 2.54.0.windows.1:
//!
//! ```text
//! nguyên nhân            exit  stdout                              stderr
//! hook pre-commit từ chối   1  (rỗng)                              đầu ra hook, CẢ hai luồng của hook
//! không có gì để commit     1  "nothing to commit, working tree…"  (rỗng)
//! thông điệp rỗng           1  (rỗng)                              "Aborting commit due to empty…"
//! ```
//!
//! 🔴 Hai điều rút ra, cả hai đều ngược trực giác:
//!
//! 1. **git gộp CẢ stdout LẪN stderr của hook vào stderr của chính nó.** Một hook
//!    `echo` ra stdout vẫn tới ta qua `stderr`. Một bộ phân loại chỉ đọc stdout nuốt
//!    **toàn bộ** thông báo của hook, im lặng.
//! 2. **"nothing to commit" nằm ở STDOUT**, không phải stderr. Một bộ phân loại chỉ
//!    đọc stderr thấy stderr **rỗng** cho ca này và rơi về một thông báo vô nghĩa.
//!
//! Nên [`phan_loai_loi_commit`] nối cả hai luồng lại trước khi so khớp, và **luôn**
//! chuyển tiếp nguyên văn cả hai cho người dùng.
//!
//! Và 🔴 **đừng lặp khuôn `open_repository`**: nó gán **một** nguyên nhân cho **mọi**
//! exit khác 0 và báo "Không phải một repository git" cho một repo hoàn toàn hợp lệ mà
//! git từ chối vì `dubious ownership` (`docs/03-phase1-qa-windows.md` ca KB-4b). Ở đây
//! mỗi nguyên nhân có variant riêng, và nhánh cuối **giữ nguyên văn** thay vì đoán.

use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::domain::RepoStatus;
use crate::error::{GitError, Result};
use crate::git::exec::DEFAULT_TIMEOUT;
use crate::git::GitCommand;
use crate::git::GitOutput;
use crate::state::{AppState, RepoHandle};

use super::worktree::{chay_lenh_ghi_co_thu_lai_phan_loai, lay_trang_thai, repo_cua};

/// Chế độ dọn thông điệp, ghim cho **mọi** đường sinh commit — PLAT-02.
///
/// Xem ghi chú đầu module: hằng này là thứ duy nhất cứu được dòng `#` của người dùng
/// khi họ có `commit.cleanup=strip` trong cấu hình. Là một hằng chứ không phải chuỗi
/// viết thẳng để hai đường (`create_commit`, `amend_commit`) không lệch nhau — và để
/// cổng đọc mã nguồn đếm được số chỗ dùng.
const CLEANUP_MODE: &str = "--cleanup=whitespace";

/// Kết quả của một lần sửa commit gần nhất — WORK-09.
///
/// # Vì sao `was_pushed` đi kèm kết quả chứ không phải một lệnh hỏi riêng
///
/// Nó là **cảnh báo sau khi làm**, không phải một cổng trước khi làm. ROADMAP nguyên
/// văn: amend trên commit đã push **cảnh báo nhưng KHÔNG chặn**. Người dùng biết họ
/// đang làm gì; một hộp thoại "không cho phép" ở đây là sai yêu cầu.
///
/// Và nó suy từ [`crate::domain::BranchInfo`] của **cùng** lời gọi status đã chạy để
/// lấy [`AmendResult::status`] — **không** có lệnh git thứ hai. Cùng tinh thần ràng
/// buộc đếm của WORK-11 (CONTEXT.md 2.6): *"Số đếm lấy từ cùng lời gọi
/// `git status --porcelain=v2`. Không thêm lệnh git thứ hai chỉ để đếm."*
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AmendResult {
    /// Trạng thái thư mục làm việc **sau** khi amend (CONTEXT.md 2.5).
    pub status: RepoStatus,

    /// Commit vừa bị sửa đã từng được đẩy lên upstream hay chưa.
    ///
    /// **Đây là cảnh báo, không phải phép chặn.** Command trả `Ok` trong cả hai ca;
    /// amend đã **chạy xong** lúc giá trị này tới tay giao diện.
    pub was_pushed: bool,
}

/// Commit gần nhất có nằm trên upstream không? — WORK-09, rủi ro R6.
///
/// # 🔴 `None` ≠ `Some(0)` — "chưa từng push" khác "đã push và đang đồng bộ"
///
/// Dòng `# branch.ab` **vắng mặt hoàn toàn** khi nhánh không có upstream; git **không**
/// in `+0 -0` (đo ở plan 04-01, và [`crate::domain::BranchInfo::ahead`] ghi lại).
///
/// Ba ca, và chỉ một ca cảnh báo:
///
/// | upstream | `ahead`   | nghĩa                       | `was_pushed` |
/// |----------|-----------|-----------------------------|--------------|
/// | `None`   | `None`    | nhánh cục bộ, chưa từng push | `false`     |
/// | `Some`   | `Some(0)` | đã push, đang đồng bộ        | **`true`**  |
/// | `Some`   | `Some(n)` | có n commit chưa push        | `false`     |
///
/// Ca `Some(n)` là `false` vì commit gần nhất nằm **trong** phần chưa đẩy — sửa nó
/// không viết lại thứ ai khác đã thấy.
///
/// Mặc định hoá `None` thành `Some(0)` làm cảnh báo nổ ở **mọi** repo cục bộ, và người
/// dùng sẽ học cách bỏ qua nó — một cảnh báo luôn bật là một cảnh báo đã chết.
fn da_push(branch: &crate::domain::BranchInfo) -> bool {
    branch.upstream.is_some() && branch.ahead == Some(0)
}

/// Thông điệp có nội dung thật không? — rủi ro R5.
///
/// # Chặn ở ĐÂY, trước khi chạy git, chứ không để git từ chối
///
/// git **cũng** từ chối thông điệp rỗng (`"Aborting commit due to empty commit
/// message."`, đã đo), nên phép chặn này trông như thừa. Nó không thừa, vì hai lý do:
///
/// 1. **Thông báo của git nói sai chuyện** khi có hook: `commit-msg` chạy **trước**
///    phép kiểm rỗng của git, nên một thông điệp rỗng trên repo có hook cho người dùng
///    đầu ra của hook thay vì câu "thông điệp rỗng". Họ đi sửa nhầm chỗ.
/// 2. **Chạy hook `pre-commit` cho một lệnh chắc chắn thất bại** là lãng phí có thể
///    rất đắt — `pre-commit` của một dự án thật chạy linter cả cây, hàng chục giây.
///
/// 🔴 **Không tự sửa thông điệp** (R5): hàm này chỉ **trả lời** rỗng hay không. Nó
/// không `trim`, không thêm nội dung, không sinh một dòng subject mặc định. Thông điệp
/// đi tới git đúng **từng byte** như người dùng gõ.
fn thong_diep_rong(message: &str) -> bool {
    message.trim().is_empty()
}

/// Đầu ra của `git commit` gộp cả hai luồng, để so khớp và để hiển thị.
///
/// Xem ghi chú đầu module: nguyên nhân thất bại nằm ở **stdout** với ca "không có gì
/// để commit" và ở **stderr** với ca hook cùng ca thông điệp rỗng. Đọc một luồng là
/// mất một nửa số ca, **im lặng**.
fn ca_hai_luong(ra: &GitOutput) -> String {
    let stdout = String::from_utf8_lossy(&ra.stdout);
    let stderr = String::from_utf8_lossy(&ra.stderr);

    let mut ket = String::new();
    if !stdout.trim().is_empty() {
        ket.push_str(stdout.trim());
    }
    if !stderr.trim().is_empty() {
        if !ket.is_empty() {
            ket.push('\n');
        }
        ket.push_str(stderr.trim());
    }
    ket
}

/// Biến một `git commit` thất bại thành lỗi có kiểu.
///
/// Phân loại theo **nội dung hai luồng**, không theo mã thoát — xem ghi chú đầu module
/// về lý do (mọi ca đều thoát 1, nên mã thoát không phân biệt được gì).
///
/// Mọi nhánh giữ đầu ra **nguyên văn**. Với ca hook đó là cả điểm (R5: *"không tự sửa
/// thông điệp; hiện nguyên văn lỗi hook"*); với các ca khác đó là phòng thủ, vì phép so
/// khớp của ta có thể trượt trên một phiên bản git khác hoặc một bản dịch khác.
fn phan_loai_loi_commit(args: Vec<String>, ra: &GitOutput) -> GitError {
    let dau_ra = ca_hai_luong(ra);

    // Ca lock: `git commit` cũng lấy `index.lock` để ghi index. Đi trước mọi phép so
    // khớp khác vì nó có đường xử lý riêng (thử lại, KHÔNG xoá tệp lock).
    if dau_ra.contains("index.lock") {
        return GitError::IndexLocked {
            args,
            stderr: dau_ra,
        };
    }

    // Ca "không có gì để commit". 🔴 Câu này nằm ở **stdout** — đã đo.
    //
    // Phép so khớp hẹp có chủ ý: chỉ nhận đúng câu git in ra. Trượt thì rơi xuống
    // nhánh cuối, nơi người dùng vẫn đọc được nguyên văn — thà một thông báo chung
    // còn hơn một thông báo **sai**, đó là bài học KB-4b.
    if dau_ra.contains("nothing to commit")
        || dau_ra.contains("no changes added to commit")
        || dau_ra.contains("nothing added to commit")
    {
        return GitError::NothingToCommit { output: dau_ra };
    }

    // Ca thông điệp rỗng. `thong_diep_rong` đã chặn trước khi chạy git, nên tới được
    // đây nghĩa là git thấy rỗng còn ta thì không — ví dụ một `commit-msg` hook đã
    // **viết lại** thông điệp thành rỗng. Giữ ca riêng để thông báo vẫn đúng chuyện.
    if dau_ra.contains("empty commit message") {
        return GitError::EmptyCommitMessage;
    }

    // Còn lại: hook từ chối, hoặc một nguyên nhân ta chưa biết.
    //
    // 🔴 **Không** cố phân biệt "hook" với "chưa biết" bằng một phép so khớp nữa. Đầu
    // ra của hook là chuỗi **tuỳ ý do người dùng viết** — không có dấu hiệu nào ổn
    // định để nhận ra nó, và mọi phép đoán sẽ sai ở một repo nào đó. Gán cả hai vào
    // `HookRejected` là trung thực: nó nói "git từ chối, đây là nguyên văn nó nói",
    // và với người dùng đó **là** thông tin họ cần.
    GitError::HookRejected {
        args,
        status: ra.status,
        output: dau_ra,
    }
}

/// Dựng một lệnh `git commit` theo đúng ràng buộc của module này.
///
/// **Nơi duy nhất** trong ứng dụng dựng lệnh commit. Hai đường gọi (`tao_commit` và
/// `sua_commit_gan_nhat`) đi qua đây để `--cleanup` và cách truyền thông điệp không
/// lệch nhau — hai bản sao sẽ lệch ngay lần đầu ai đó chỉnh một chỗ.
///
/// # `no_verify` có ĐÚNG MỘT đường vào
///
/// Cờ `--no-verify` chỉ được thêm ở dòng dưới, sau một `if` trên tham số. Không đường
/// nào khác, không cấu hình, không biến môi trường. Cổng
/// [`tests::chi_mot_duong_them_no_verify`] ghim điều đó ở mức mã nguồn.
fn dung_lenh_commit(
    repo_path: &std::path::Path,
    message: Vec<u8>,
    no_verify: bool,
    amend: bool,
) -> GitCommand {
    let mut cmd = GitCommand::new(repo_path).arg("commit");

    if amend {
        cmd = cmd.arg("--amend");
    }

    if no_verify {
        cmd = cmd.arg("--no-verify");
    }

    cmd.arg(CLEANUP_MODE)
        // `-F -` đọc thông điệp từ **stdin**. Xem ghi chú đầu module về lý do không
        // dùng `-m`. Cặp cờ này phải đi cùng `stdin_bytes` — thiếu một trong hai thì
        // git chờ nhập liệu và lệnh treo tới hết hạn giờ.
        .args(["-F", "-"])
        .stdin_bytes(message)
        .timeout(DEFAULT_TIMEOUT)
}

/// Tạo một commit từ những gì đang ở index — WORK-08.
///
/// Trả [`RepoStatus`] **mới** (CONTEXT.md 2.5 — xem ghi chú đầu `worktree.rs`).
///
/// Tách khỏi `#[tauri::command]` để test tích hợp gọi được mà không cần dựng một `App`
/// có webview, cùng khuôn [`super::worktree::stage_duong_dan`].
///
/// # Hook chạy trừ khi `no_verify` bật — và nó chặn commit THẬT
///
/// Đã đo trên repo mẫu `hook-reject`: `pre-commit` thoát 1 làm `git commit` thoát 1,
/// **không** tạo commit (`git rev-parse HEAD` không đổi), và tệp đã stage **vẫn** ở
/// index. Với `no_verify = true` trên **cùng** repo đó, commit thành công. Đó là phép
/// phân biệt chứng minh công tắc nối thật vào git chứ không phải một ô tick trang trí.
pub async fn tao_commit(
    state: &AppState,
    repo: Arc<RepoHandle>,
    message: &str,
    no_verify: bool,
) -> Result<RepoStatus> {
    if thong_diep_rong(message) {
        // 🔴 Trả lỗi **trước** khi chạy git — xem `thong_diep_rong`. Không lệnh nào
        // được sinh ra ở nhánh này; có test đo bằng `CommandLog`.
        return Err(GitError::EmptyCommitMessage);
    }

    let byte = message.as_bytes().to_vec();
    let repo_path = repo.path.clone();

    chay_lenh_ghi_co_thu_lai_phan_loai(
        state,
        &repo,
        || dung_lenh_commit(&repo_path, byte.clone(), no_verify, false),
        phan_loai_loi_commit,
    )
    .await?;

    lay_trang_thai(state, repo).await
}

/// Sửa commit gần nhất — WORK-09.
///
/// # 🔴 CẢNH BÁO, KHÔNG CHẶN
///
/// ROADMAP nguyên văn: amend trên một commit đã push **cảnh báo nhưng không chặn**.
/// Hàm này **không** có nhánh nào trả lỗi vì [`AmendResult::was_pushed`]; giá trị đó
/// được tính **sau** khi amend đã chạy xong, và nó đi kèm kết quả **thành công**.
///
/// Một `if was_pushed { return Err(...) }` ở đây là sai yêu cầu, và nó là loại thay
/// đổi mà một người nghĩ mình đang giúp sẽ thêm vào. Đột biến M9 của plan ghim nó.
///
/// # `was_pushed` không tốn thêm một lệnh git
///
/// Nó suy từ `branch` của **cùng** [`RepoStatus`] đã phải lấy để trả về. Xem
/// [`da_push`].
pub async fn sua_commit_gan_nhat(
    state: &AppState,
    repo: Arc<RepoHandle>,
    message: &str,
    no_verify: bool,
) -> Result<AmendResult> {
    if thong_diep_rong(message) {
        return Err(GitError::EmptyCommitMessage);
    }

    let byte = message.as_bytes().to_vec();
    let repo_path = repo.path.clone();

    // 🔴 `was_pushed` phải đo TRƯỚC khi amend — đo sau cho kết quả luôn SAI.
    //
    // Đây là một lỗi thật, tìm ra bằng một test đỏ chứ không bằng suy luận. Bản đầu
    // đọc `ahead` từ lời gọi status **sau** khi amend, theo đúng chữ của plan ("suy từ
    // `BranchInfo` của cùng lời gọi status"). Nhưng amend **viết lại** commit gần nhất,
    // nên ngay sau nó nhánh cục bộ **phân kỳ** khỏi upstream: `ahead` nhảy từ `Some(0)`
    // lên `Some(1)`, và `da_push` trả `false` **đúng ở ca duy nhất phải cảnh báo**.
    //
    // Câu hỏi mà cảnh báo trả lời là *"commit tôi SẮP viết lại có nằm trên remote
    // không?"* — một câu hỏi về trạng thái **trước** thao tác. Đo sau là đo nhầm câu.
    //
    // Vẫn **không có lệnh git thứ hai chỉ để đếm**: lời gọi status này là lời gọi ta
    // vốn đã phải chạy để biết có gì ở index; ràng buộc của WORK-11 được tôn trọng.
    let truoc = lay_trang_thai(state, Arc::clone(&repo)).await?;
    let was_pushed = da_push(&truoc.branch);

    chay_lenh_ghi_co_thu_lai_phan_loai(
        state,
        &repo,
        || dung_lenh_commit(&repo_path, byte.clone(), no_verify, true),
        phan_loai_loi_commit,
    )
    .await?;

    // Trạng thái trả về là trạng thái **mới** (CONTEXT.md 2.5) — đọc lại sau thao tác.
    // Đây là **cảnh báo**; không có `return Err` nào dựa trên `was_pushed`.
    let status = lay_trang_thai(state, repo).await?;

    Ok(AmendResult { status, was_pushed })
}

/// Tạo commit — WORK-08. Trả trạng thái **mới** (CONTEXT.md 2.5).
#[tauri::command]
pub async fn create_commit(
    repo_id: String,
    message: String,
    no_verify: bool,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    tao_commit(&state, repo, &message, no_verify).await
}

/// Sửa commit gần nhất — WORK-09. **Cảnh báo** qua `wasPushed`, không chặn.
#[tauri::command]
pub async fn amend_commit(
    repo_id: String,
    message: String,
    no_verify: bool,
    state: State<'_, AppState>,
) -> Result<AmendResult> {
    let repo = repo_cua(&state, &repo_id)?;
    sua_commit_gan_nhat(&state, repo, &message, no_verify).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::BranchInfo;

    /// Thân phần **không-test** của tệp này, đã **bỏ dòng chú thích**.
    ///
    /// # Vì sao bỏ chú thích là bắt buộc
    ///
    /// Cổng thứ **năm** trong bảy cổng không-thể-fail của dự án (CONTEXT.md 3.1) chết
    /// đúng vì điều này: nó grep trên nguồn **thô** và khớp một chú thích **do chính
    /// mutation sinh ra**. Tệp này đặc biệt nguy hiểm theo hướng đó — mọi doc comment
    /// ở đây nói về `--no-verify`, `-F -` và `--cleanup`, nên một cổng đọc nguồn thô
    /// sẽ khớp văn xuôi và **xanh vĩnh viễn** dù mã bị gỡ sạch.
    ///
    /// Cắt ở `#[cfg(test)]` để không đọc chính mã test — test có những chuỗi đó trong
    /// phép khẳng định.
    fn ma_khong_chu_thich() -> String {
        let src = include_str!("commit.rs");
        src.lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// **Đúng MỘT đường thêm `--no-verify`, và nó có điều kiện.**
    ///
    /// 🔴 Đột biến M1 của plan (thêm `--no-verify` **vô điều kiện**) và M2 (bỏ hẳn
    /// nhánh) đều bị test hành vi trên repo có hook bắt. Cổng này là lớp thứ hai và nó
    /// bắt một thứ khác: một đường thêm cờ **thứ hai** — ví dụ ai đó thêm nó vào
    /// `amend` "cho tiện" vì amend hay chạy lại hook. Test hành vi cho `create_commit`
    /// sẽ **không** thấy điều đó.
    #[test]
    fn chi_mot_duong_them_no_verify() {
        let ma = ma_khong_chu_thich();

        // 🔴 Khẳng định **tiền đề** trước: không tìm thấy thứ cần kiểm thì cổng phải
        // ĐỎ, không phải xanh. Cổng thứ ba của CONTEXT.md 3.1 chết vì đường dẫn sai →
        // grep không thấy gì → xanh.
        assert!(
            ma.contains("GitCommand::new"),
            "tiền đề: phép lọc chú thích phải giữ lại được thân hàm thật"
        );
        assert!(
            ma.contains("no_verify"),
            "tiền đề: tệp này phải có tham số no_verify"
        );

        let so_cho = ma.matches("--no-verify").count();
        assert_eq!(
            so_cho, 1,
            "đúng MỘT chỗ viết chuỗi `--no-verify` trong mã không-chú-thích. Nhiều hơn \
             nghĩa là có đường thứ hai thêm cờ, và một test hành vi cho đường này sẽ \
             không thấy đường kia. Đọc được {so_cho}"
        );

        // Cờ phải nằm trong một nhánh điều kiện trên `no_verify`. Một dòng
        // `.arg("--no-verify")` không có `if` phía trước là đột biến M1.
        let vi_tri = ma.find("--no-verify").expect("vừa khẳng định là có");
        let truoc = &ma[..vi_tri];
        let dong_if = truoc.rfind("if no_verify");
        assert!(
            dong_if.is_some(),
            "🔴 `--no-verify` phải nằm sau `if no_verify`. Thêm nó VÔ ĐIỀU KIỆN nghĩa \
             là hook KHÔNG BAO GIỜ chạy, và ROADMAP nói hook chạy MẶC ĐỊNH — đó là một \
             trong những lý do dự án chọn git CLI thay vì libgit2 (CONTEXT.md 2.4)"
        );
        // `if` phải ở gần, không phải một `if no_verify` của một hàm khác phía trên.
        let khoang_cach = vi_tri - dong_if.expect("vừa kiểm");
        assert!(
            khoang_cach < 200,
            "`if no_verify` gần nhất cách chỗ thêm cờ {khoang_cach} ký tự — quá xa để \
             là điều kiện của nó. Cổng này đang đo nhầm một nhánh khác"
        );
    }

    /// **Thông điệp đi qua stdin, KHÔNG qua `-m`** — đột biến M3.
    ///
    /// # 🔴 Cổng này là cổng DUY NHẤT cho M3, và đó là một phát hiện đo được
    ///
    /// Plan 04-03 ghi rằng đột biến M3 sẽ làm *"test thông điệp nhiều dòng giữ xuống
    /// dòng"* đỏ. **Đo được rằng điều đó SAI trên nền tảng này.** Áp M3 thật rồi chạy
    /// cả bộ test tích hợp: **15 passed, 0 failed**.
    ///
    /// Vì sao: lý lẽ "argv trên Windows là một chuỗi nên xuống dòng không sống sót"
    /// đúng cho một **shell**, nhưng `std::process::Command` của Rust **không** đi qua
    /// shell — nó dựng chuỗi dòng lệnh bằng phép trích dẫn đúng của Windows, nên một
    /// đối số chứa `\n` tới `git` nguyên vẹn. Đã đo trực tiếp:
    ///
    /// ```text
    /// git commit --cleanup=whitespace -m $'subject\n\nbody one\nbody two'
    ///   -> %B đọc lại: "subject\n\nbody one\nbody two\n"   (NGUYÊN VẸN)
    /// ```
    ///
    /// Ràng buộc `-F -` **vẫn đúng**, nhưng lý do thật là ca **byte không phải UTF-8**:
    /// [`GitCommand::arg`] gọi `to_string_lossy()`, nên một byte hỏng trong argv bị
    /// thay bằng `U+FFFD` **vĩnh viễn** và ghi thẳng vào lịch sử repo. Ca đó **không**
    /// kiểm được qua API công khai hiện tại vì `create_commit` nhận `String` (đã là
    /// UTF-8 hợp lệ theo kiểu) — nên không có test hành vi nào phân biệt được M3, và
    /// cổng đọc mã nguồn này là cổng duy nhất.
    ///
    /// Ghi lại đầy đủ thay vì im lặng: một cổng được ghi là "lớp thứ hai" trong khi nó
    /// thật ra là lớp **duy nhất** sẽ bị ai đó xoá đi vì tưởng có cổng khác đỡ.
    #[test]
    fn thong_diep_di_qua_stdin_khong_qua_dau_m() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("stdin_bytes"),
            "🔴 tiền đề + ràng buộc: thông điệp phải đi qua `stdin_bytes`. Thiếu nó \
             nghĩa là thông điệp nằm trong argv, nơi Windows dựng dòng lệnh thành MỘT \
             chuỗi rồi để tiến trình con tự tách — xuống dòng và dấu ngoặc không sống \
             sót qua đó"
        );
        assert!(
            ma.contains(r#""-F""#),
            "phải có cờ `-F` để git đọc thông điệp từ tệp/stdin"
        );

        // `-m` không được xuất hiện như một đối số của lệnh commit.
        assert!(
            !ma.contains(r#".arg("-m")"#) && !ma.contains(r#""-m","#) && !ma.contains(r#""-m"]"#),
            "🔴 tìm thấy `-m` trong mã: thông điệp là dữ liệu TUỲ Ý từ webview và có \
             thể chứa byte không phải UTF-8. `-m` đặt nó vào argv, nơi `GitCommand::arg` \
             gọi `to_string_lossy` và thay byte hỏng bằng U+FFFD VĨNH VIỄN — ghi thẳng \
             vào lịch sử repo của người dùng"
        );
    }

    /// **`--cleanup=whitespace` tới CẢ HAI đường commit, và không bị ghi đè.**
    ///
    /// Một hằng dùng chung, dùng đúng một lần ở nơi dựng lệnh — nên cả `create` lẫn
    /// `amend` đều mang nó mà không có bản sao nào lệch được.
    #[test]
    fn cleanup_whitespace_ghim_cho_moi_duong_commit() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("--cleanup=whitespace"),
            "🔴 tiền đề: PLAT-02 đòi mọi lệnh commit mang `--cleanup=whitespace`. \
             Thiếu nó, một người dùng có `commit.cleanup=strip` trong cấu hình sẽ thấy \
             mọi dòng bắt đầu bằng `#` biến mất khỏi thông điệp — im lặng, không một \
             lời giải thích (docs/02-phase4-commit-notes.md)"
        );

        // Không được có một `--cleanup=` thứ hai với giá trị khác: cờ sau ghi đè cờ
        // trước trong git, nên một dòng `--cleanup=default` thêm vào sẽ vô hiệu hoá
        // hằng này mà hằng vẫn còn nguyên trong mã.
        let so_cleanup = ma.matches("--cleanup=").count();
        assert_eq!(
            so_cleanup, 1,
            "đúng MỘT chỗ đặt `--cleanup=`. Git lấy cờ CUỐI CÙNG, nên một cờ thứ hai \
             ghi đè cờ đầu trong khi cờ đầu vẫn còn nguyên trong mã — cổng đọc nguồn \
             nào chỉ tìm sự TỒN TẠI của chuỗi sẽ xanh. Đọc được {so_cleanup}"
        );

        // Nơi dựng lệnh dùng chung phải là nơi duy nhất dựng `git commit`.
        let so_commit = ma.matches(r#"arg("commit")"#).count();
        assert_eq!(
            so_commit, 1,
            "đúng MỘT nơi dựng lệnh `git commit` (`dung_lenh_commit`). Một đường thứ \
             hai sẽ không đi qua `--cleanup` và `-F -` của nơi này. Đọc được {so_commit}"
        );
    }

    /// **`was_pushed` phân biệt ĐÚNG BA ca** — đột biến M11.
    ///
    /// 🔴 Ca `None` (không upstream) là ca dễ sai nhất, và nó **không** giống `Some(0)`.
    /// Xem [`da_push`] và `BranchInfo::ahead`.
    #[test]
    fn was_pushed_phan_biet_khong_upstream_voi_da_dong_bo() {
        let dung = |upstream: Option<&str>, ahead: Option<u32>| BranchInfo {
            head: Some("main".into()),
            oid: Some("abc".into()),
            upstream: upstream.map(str::to_owned),
            ahead,
            behind: Some(0),
        };

        assert!(
            !da_push(&dung(None, None)),
            "🔴 nhánh KHÔNG có upstream → chưa từng push → KHÔNG cảnh báo. Dòng \
             `# branch.ab` VẮNG MẶT ở ca này (git không in `+0 -0`), nên `ahead` là \
             None. Mặc định hoá None thành Some(0) làm cảnh báo nổ ở MỌI repo cục bộ, \
             và một cảnh báo luôn bật là một cảnh báo đã chết"
        );

        assert!(
            da_push(&dung(Some("origin/main"), Some(0))),
            "🔴 có upstream và ahead == 0 → commit gần nhất ĐÃ nằm trên remote → phải \
             cảnh báo. Đây là ca duy nhất cảnh báo"
        );

        assert!(
            !da_push(&dung(Some("origin/main"), Some(3))),
            "có upstream nhưng còn 3 commit chưa đẩy → commit gần nhất nằm TRONG phần \
             chưa đẩy → sửa nó không viết lại thứ ai khác đã thấy → không cảnh báo"
        );

        assert!(
            !da_push(&dung(None, Some(0))),
            "ca không nhất quán (không upstream nhưng có ahead) phải nghiêng về KHÔNG \
             cảnh báo — cảnh báo sai làm người dùng mất tin vào nó"
        );
    }

    /// 🔴 **Không có đường nào CHẶN amend vì `was_pushed`** — đột biến M9, phía Rust.
    ///
    /// ROADMAP nguyên văn: *cảnh báo, KHÔNG chặn*. Đây là loại thay đổi mà một người
    /// nghĩ mình đang giúp sẽ thêm vào ("chắc chắn chứ?"), nên nó cần một cổng.
    ///
    /// `CommitBox` có cổng tương ứng ở phía giao diện; cổng này phủ phía Rust, nơi một
    /// `return Err` sẽ chặn **kể cả khi** giao diện cho bấm.
    #[test]
    fn khong_co_duong_nao_chan_amend_vi_da_push() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("was_pushed"),
            "tiền đề: tệp này phải tính was_pushed, nếu không cổng đang đọc sai chỗ"
        );

        // Lấy thân `sua_commit_gan_nhat` — nơi duy nhất một phép chặn có thể nằm.
        let than = ma
            .split("pub async fn sua_commit_gan_nhat")
            .nth(1)
            .expect("tiền đề: phải có hàm sua_commit_gan_nhat");
        // Cắt ở khai báo tiếp theo để chỉ đọc thân hàm này. Mốc phải là một thứ CÓ
        // THẬT trong tệp — cắt ở một tên đã bị xoá làm `split` trả cả phần còn lại của
        // tệp, và cổng lặng lẽ đo rộng hơn nó tưởng. Có khẳng định tiền đề bên dưới.
        assert!(
            than.contains("#[tauri::command]"),
            "tiền đề: phải còn khai báo sau `sua_commit_gan_nhat` để cắt thân hàm; \
             thiếu nó thì cổng đang đọc cả phần còn lại của tệp"
        );
        let than = than.split("#[tauri::command]").next().unwrap_or(than);

        for cam in [
            "if was_pushed",
            "if da_push",
            "was_pushed {\n        return Err",
        ] {
            assert!(
                !than.contains(cam),
                "🔴 tìm thấy `{cam}` trong `sua_commit_gan_nhat`: amend trên commit đã \
                 push phải CẢNH BÁO, KHÔNG CHẶN (ROADMAP nguyên văn, WORK-09). Người \
                 dùng biết họ đang làm gì.\n\nThân hàm:\n{than}"
            );
        }

        // `was_pushed` chỉ được dùng để **dựng kết quả**, không để rẽ nhánh.
        assert!(
            than.contains("Ok(AmendResult"),
            "hàm phải trả Ok kèm AmendResult trong mọi ca amend chạy được"
        );
    }

    /// **Thông điệp rỗng bị chặn TRƯỚC khi chạy git** — đột biến M4.
    ///
    /// Test hành vi (đo bằng `CommandLog` rằng không lệnh nào sinh ra) là cổng chính.
    /// Ở đây kiểm chính vị từ, gồm ca chỉ-khoảng-trắng.
    #[test]
    fn thong_diep_chi_khoang_trang_la_rong() {
        assert!(thong_diep_rong(""), "chuỗi rỗng là rỗng");
        assert!(thong_diep_rong("   "), "chỉ dấu cách là rỗng");
        assert!(thong_diep_rong("\n\n"), "chỉ xuống dòng là rỗng");
        assert!(
            thong_diep_rong(" \t \r\n "),
            "chỉ khoảng trắng hỗn hợp là rỗng"
        );

        assert!(!thong_diep_rong("a"), "một ký tự là KHÔNG rỗng");
        assert!(
            !thong_diep_rong("#123 chi co dong hash"),
            "🔴 một thông điệp CHỈ có dòng bắt đầu bằng `#` là KHÔNG rỗng. Coi nó là \
             rỗng nghĩa là ta tự áp luật `--cleanup=default` mà ta vừa tắt đi — và \
             người dùng gõ `#123` làm subject sẽ bị từ chối không hiểu vì sao"
        );
        assert!(
            !thong_diep_rong("  co noi dung  "),
            "khoảng trắng bao quanh không làm thông điệp thành rỗng"
        );
    }

    /// 🔴 **Không tự sửa thông điệp** (R5): `thong_diep_rong` chỉ TRẢ LỜI, không biến đổi.
    ///
    /// Một `message.trim()` đưa vào `stdin_bytes` sẽ **im lặng** cắt khoảng trắng đầu
    /// dòng của người dùng — thụt lề trong thân thông điệp là có nghĩa (danh sách, khối
    /// mã). Cổng này ghim rằng byte đi tới git là byte người dùng gõ.
    #[test]
    fn thong_diep_toi_git_khong_bi_bien_doi() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains("message.as_bytes().to_vec()"),
            "🔴 thông điệp phải đi tới git NGUYÊN VĂN từng byte. Tìm không thấy phép \
             chuyển thẳng `message.as_bytes().to_vec()` — nếu ai đó chèn `.trim()` hay \
             một phép chuẩn hoá vào đây thì thụt lề và dòng trống của người dùng bị \
             cắt IM LẶNG (R5: không tự sửa thông điệp)"
        );
        assert!(
            !ma.contains("message.trim().as_bytes"),
            "🔴 `trim()` trên đường đi tới git là phép tự sửa thông điệp — R5 cấm"
        );
    }

    /// Phân loại lỗi đọc **cả hai luồng** — ghim bằng hành vi của hàm thuần.
    ///
    /// 🔴 Đây là ca mà một bộ phân loại "hợp lý" sai: đo trên git 2.54.0.windows.1,
    /// *"nothing to commit"* nằm ở **stdout** còn *"empty commit message"* ở **stderr**,
    /// và git gộp **cả hai** luồng của hook vào **stderr** của chính nó.
    #[test]
    fn phan_loai_doc_ca_stdout_lan_stderr() {
        let ra_stdout = GitOutput {
            stdout: b"On branch main\nnothing to commit, working tree clean\n".to_vec(),
            stderr: Vec::new(),
            status: 1,
        };
        let loi = phan_loai_loi_commit(Vec::new(), &ra_stdout);
        assert_eq!(
            loi.code(),
            "nothing_to_commit",
            "🔴 ca 'không có gì để commit' in ra STDOUT, stderr RỖNG. Một bộ phân loại \
             chỉ đọc stderr thấy chuỗi rỗng và rơi vào nhánh sai. Nhận được: {loi}"
        );

        let ra_stderr = GitOutput {
            stdout: Vec::new(),
            stderr: b"Aborting commit due to empty commit message.\n".to_vec(),
            status: 1,
        };
        assert_eq!(
            phan_loai_loi_commit(Vec::new(), &ra_stderr).code(),
            "empty_commit_message",
            "ca thông điệp rỗng in ra STDERR — luồng ngược lại với ca trên"
        );

        // Ca hook: git gộp cả stdout lẫn stderr của hook vào stderr của nó.
        let ra_hook = GitOutput {
            stdout: Vec::new(),
            stderr: b"HOOK-PRE-COMMIT-REJECTED (stdout)\nHOOK-PRE-COMMIT-REJECTED (stderr)\n"
                .to_vec(),
            status: 1,
        };
        let loi_hook = phan_loai_loi_commit(Vec::new(), &ra_hook);
        assert_eq!(loi_hook.code(), "hook_rejected");
        assert!(
            loi_hook.to_string().contains("HOOK-PRE-COMMIT-REJECTED"),
            "🔴 đầu ra hook phải đi tới người dùng NGUYÊN VĂN (R5, đột biến M5). Một \
             thông báo chung như 'commit thất bại' vứt đi chính thứ duy nhất nói cho \
             họ biết phải sửa gì. Nhận được: {loi_hook}"
        );

        // Ca lock đi trước mọi phép so khớp khác.
        let ra_lock = GitOutput {
            stdout: Vec::new(),
            stderr: b"fatal: Unable to create '/r/.git/index.lock': File exists.".to_vec(),
            status: 128,
        };
        assert_eq!(
            phan_loai_loi_commit(Vec::new(), &ra_lock).code(),
            "index_locked",
            "ca lock phải giữ mã riêng ở đường commit, giống đường stage của 04-02"
        );
    }

    /// `ca_hai_luong` không **mất** luồng nào, và không dính hai dòng vào nhau.
    #[test]
    fn ca_hai_luong_giu_du_ca_hai() {
        let ra = GitOutput {
            stdout: b"dong stdout\n".to_vec(),
            stderr: b"dong stderr\n".to_vec(),
            status: 1,
        };
        let s = ca_hai_luong(&ra);
        assert!(s.contains("dong stdout"), "mất stdout: {s:?}");
        assert!(s.contains("dong stderr"), "mất stderr: {s:?}");
        assert!(
            s.contains("dong stdout\ndong stderr"),
            "hai luồng phải ngăn nhau bằng xuống dòng, không dính liền: {s:?}"
        );

        // Luồng rỗng không sinh dòng trống thừa ở đầu — người dùng đọc thông báo này.
        let chi_stderr = GitOutput {
            stdout: Vec::new(),
            stderr: b"chi co stderr".to_vec(),
            status: 1,
        };
        assert_eq!(ca_hai_luong(&chi_stderr), "chi co stderr");
    }

    /// **Lệnh commit KHÔNG mang pathspec** — nên cổng `--` của 03-02 không áp ở đây.
    ///
    /// Ghi lại thành một cổng chứ không một câu chú thích: nếu ai đó thêm pathspec vào
    /// `git commit` (ví dụ `git commit -- <path>` để commit một phần), lệnh đó **phải**
    /// mang `--` và cổng này đỏ để nhắc họ cập nhật khuôn của `worktree.rs`.
    #[test]
    fn lenh_commit_khong_mang_pathspec() {
        let ma = ma_khong_chu_thich();

        assert!(
            ma.contains(r#"arg("commit")"#),
            "tiền đề: tệp này phải dựng một lệnh `git commit`"
        );
        assert!(
            !ma.contains("PATHSPEC_SAU_DAU_GACH"),
            "🔴 lệnh commit của wave này KHÔNG mang pathspec (nó commit cả index). \
             Thêm pathspec vào đây nghĩa là phải thêm `--` TRƯỚC nó và mang mốc \
             `PATHSPEC_SAU_DAU_GACH` theo khuôn `worktree.rs`, cộng cập nhật cổng đếm \
             của tệp đó"
        );
    }

    /// Cả hai đường commit đi qua đường **chạy-có-thử-lại** của 04-02 — đột biến M12.
    ///
    /// Đường đó mang hai thứ mà một `runner.write` trần không có: giữ `write_lock`
    /// (PLAT-03) và xử lý `index.lock` bằng **thử lại, không xoá** (CONTEXT.md 2.3).
    /// Plan nói rõ dùng **cùng** hàm, không viết lại — nên phần thử lại được dùng lại
    /// nguyên vẹn, chỉ phép **phân loại lỗi** được tiêm vào (xem
    /// `chay_lenh_ghi_co_thu_lai_phan_loai`).
    #[test]
    fn hai_duong_commit_dung_chung_duong_thu_lai_cua_04_02() {
        let ma = ma_khong_chu_thich();

        let so = ma.matches("chay_lenh_ghi_co_thu_lai_phan_loai").count();
        assert_eq!(
            so, 3,
            "ba chỗ trong mã không-chú-thích: dòng `use` nhập hàm, chỗ gọi trong \
             `tao_commit`, chỗ gọi trong `sua_commit_gan_nhat`. Số khác 3 nghĩa là một \
             đường commit đi vòng qua nó — và đường đó sẽ không giữ write_lock, cũng \
             không thử lại khi gặp index.lock. Đọc được {so}"
        );

        // Cả hai đường phải tiêm CÙNG phép phân loại. Một đường quên nó sẽ nhận
        // `CommandFailed` chung và người dùng mất nguyên văn đầu ra hook.
        let so_phan_loai = ma.matches("phan_loai_loi_commit").count();
        assert_eq!(
            so_phan_loai, 3,
            "ba chỗ: khai báo hàm, và hai chỗ tiêm vào (tao_commit, sua_commit_gan_nhat). \
             Một đường quên tiêm sẽ rơi về `CommandFailed` chung — và người dùng mất \
             NGUYÊN VĂN đầu ra hook, đúng đột biến M5. Đọc được {so_phan_loai}"
        );

        assert!(
            !ma.contains("runner.write") && !ma.contains("runner\n"),
            "🔴 tệp này KHÔNG được tự gọi `runner.write`: làm vậy là bỏ qua phép thử \
             lại index.lock và phép giữ write_lock mà đường chung mang"
        );
    }

    /// `AmendResult` serialize sang **camelCase** — hợp đồng IPC với phía giao diện.
    ///
    /// Đột biến M11 của 04-02 (đổi `repoId` thành `repo_id`) cho thấy lớp lỗi này im
    /// lặng hoàn toàn: JSON vẫn hợp lệ, TypeScript đọc `undefined`, và `undefined` là
    /// falsy — nên một cảnh báo "đã push" sẽ **không bao giờ hiện** mà không ai biết.
    #[test]
    fn amend_result_dung_camel_case() {
        let kq = AmendResult {
            status: RepoStatus {
                branch: BranchInfo {
                    head: Some("main".into()),
                    oid: None,
                    upstream: None,
                    ahead: None,
                    behind: None,
                },
                entries: Vec::new(),
                has_conflicts: false,
            },
            was_pushed: true,
        };

        let json = serde_json::to_value(&kq).unwrap();
        assert_eq!(
            json["wasPushed"], true,
            "🔴 khoá phải là `wasPushed` (camelCase). Với `was_pushed`, TypeScript đọc \
             `undefined` — và `undefined` là falsy, nên cảnh báo 'đã push' KHÔNG BAO \
             GIỜ hiện, im lặng hoàn toàn. Đọc được: {json}"
        );
        assert!(
            json.get("was_pushed").is_none(),
            "không được có cả hai dạng khoá"
        );
        assert!(
            json["status"].is_object(),
            "trạng thái mới phải đi kèm (CONTEXT.md 2.5)"
        );
    }
}
