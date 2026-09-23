//! Staging theo **khối** — WORK-03, WORK-04, WORK-05.
//!
//! Nối tầng byte thô của [`crate::git::patch_build`] vào `git apply --cached`. Đây là
//! phase **đầu tiên** ghi vào nội dung tệp của người dùng chứ không chỉ vào index, nên
//! mọi ràng buộc dưới đây ROADMAP xếp ở mức "không thương lượng".
//!
//! # Đường đi, và vì sao nó có đúng bấy nhiêu bước
//!
//! 1. **Kiểm blob hash** — tệp có còn đúng thứ giao diện đã vẽ diff không (WORK-05).
//! 2. `git diff` → byte thô của bản vá đầy đủ.
//! 3. Tách khối, dựng bản vá con chứa **đúng một** khối.
//! 4. `git apply --check --cached --recount` — thử khan, **không** đổi gì.
//! 5. `git apply --cached --recount` — áp thật.
//! 6. Trả trạng thái **mới**.
//!
//! Bước 4 và 5 là **hai** lệnh, không phải một. `--check` cho ta biết bản vá không khớp
//! **trước** khi bất kỳ byte nào vào index, nên ca thất bại để lại index **sạch**. Gộp
//! hai bước thành một `git apply` trơn nghĩa là một bản vá khớp một phần được áp một
//! phần, và người dùng nhận một index nửa vời mà không có gì nói cho họ biết.
//!
//! # 🔴 Bản vá là byte thô từ đầu đến cuối
//!
//! `stdout` của `git diff` là `Vec<u8>` và nó đi thẳng vào `stdin_bytes` của
//! `git apply`. **Không** `String` ở bất kỳ điểm nào. `String::from_utf8_lossy` đổi mỗi
//! byte không hợp lệ thành U+FFFD — **ba** byte — và git sẽ ghi ba byte đó vào tệp thật
//! của người dùng. Một tệp Latin-1 hay một tệp có BOM là dữ liệu có thật trên máy người
//! dùng, không phải ca giả định. Cổng `duong_ban_va_khong_bao_gio_giai_ma` của
//! `patch_build.rs` ghim nửa dưới của ràng buộc này; cổng
//! [`tests::duong_apply_khong_bao_gio_giai_ma_ban_va`] ghim nửa ở đây.
//!
//! # 🔴 stderr KHÔNG BAO GIỜ ghép vào bản vá
//!
//! `GitOutput` tách sẵn hai luồng; việc duy nhất phải làm là **không ghép**. Đo được
//! hôm nay: `git stash create` in `warning: in the working copy of 'a.txt', LF will be
//! replaced by CRLF` — và với `$(...)` trong shell nó rơi vào **cùng luồng** với sha.
//! Một cảnh báo của clean filter lọt vào giữa nội dung bản vá làm git từ chối bản vá,
//! hoặc tệ hơn, áp sai chỗ. Nên chỉ `ra.stdout` đi vào `stdin_bytes`.
//!
//! # 🔴 Thất bại KHÔNG BAO GIỜ được thử lại lỏng hơn
//!
//! Không `--whitespace=fix`, không `--unidiff-zero`, không `--3way`, không `--reject`,
//! không `-C1`. Đây là loại ràng buộc mà một lần "sửa cho nó chạy" trong tương lai sẽ
//! phá, nên nó có **cổng** chứ không có lời hứa:
//! [`tests::duong_apply_khong_bao_gio_khop_mo`].
//!
//! Lý do cụ thể: khớp mờ nghĩa là git tìm một chỗ **gần giống** để áp bản vá. Khi tệp
//! đã đổi — tức đúng ca này — "gần giống" là một chỗ **sai**, và kết quả là nội dung
//! của người dùng bị ghi đè ở một vị trí họ không chọn. Câu trả lời đúng là nói cho họ
//! biết và dừng lại: [`GitError::FileChanged`].

use std::sync::Arc;

use tauri::State;

use crate::domain::RepoStatus;
use crate::error::{GitError, Result};
use crate::git::patch_build::{dung_ban_va_mot_hunk, tach_hunk_tho};
use crate::git::exec::DEFAULT_TIMEOUT;
use crate::git::GitCommand;
use crate::state::{AppState, RepoHandle};

use super::worktree::{chay_lenh_ghi_co_thu_lai_phan_loai, lay_trang_thai, repo_cua};

/// Mốc đánh dấu vị trí một pathspec trong argv.
///
/// Hàm đồng nhất — nó **không** biến đổi gì. Nó tồn tại để cổng
/// [`tests::moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path`] khẳng định được
/// *thứ tự* của `--` và pathspec bằng cách tìm một chuỗi ổn định trong mã nguồn.
///
/// Cùng mẫu (và cùng tên) với mốc của [`super::diff`] và [`super::worktree`]. Khai lại
/// ở đây chứ không dùng chung: cổng của mỗi tệp đọc `include_str!` của **chính tệp
/// đó**, nên cổng của hai tệp kia không phủ được lệnh nằm ở đây.
#[allow(non_snake_case)]
fn PATHSPEC_SAU_DAU_GACH(path: &str) -> &str {
    path
}

/// Tập cờ dùng cho **mọi** lời gọi `git apply` của tệp này.
///
/// # Vì sao là một hằng, không phải chép vào từng chỗ gọi
///
/// Test tích hợp `tests/hunk_commands.rs` đọc **chính hằng này** để chứng minh rằng
/// bản vá cắt-thân thất bại khi bỏ `--recount`. Chép danh sách cờ vào test là hai
/// nguồn sự thật lệch nhau: ai đó bỏ `--recount` khỏi mã thật thì test vẫn chạy trên
/// bản chép của nó và vẫn xanh. `STATUS_ARGS` tồn tại từ 04-01 đúng vì lý do này.
///
/// # 🔴 `--recount` là bắt buộc
///
/// Khi người dùng chọn một tập con các khối, số dòng trong header `@@` **không còn
/// đúng**; `--recount` bảo git tính lại từ nội dung thật. Thiếu nó, git từ chối bản vá
/// hoặc — tệ hơn — áp sai vị trí.
///
/// Đo được trên git 2.54.0.windows.1, và phép đo **bác** giả định tự nhiên:
///
/// ```text
/// hunk 2 chép NGUYÊN VĂN header:
///   --recount → exit 0 ;  không --recount → exit 0    ← KHÔNG phân biệt được
/// hunk 2 CẮT BỚT THÂN, header để nguyên:
///   --recount → exit 0 ;  không --recount → exit 128 "corrupt patch"
/// ```
///
/// Số đếm trong `@@` là **của riêng từng hunk**, không tích luỹ qua cả tệp — nên chép
/// nguyên header hunk 2 thì nó vẫn đúng dù bỏ bao nhiêu hunk khác. Thứ làm header
/// **thật sự sai** là cắt bớt dòng thân, đúng việc một bộ dựng bản vá con làm khi nó
/// tỉa ngữ cảnh. Test B của `tests/hunk_commands.rs` dùng đúng fixture đó, và nó chứng
/// minh phép phân biệt **bên trong chính test**.
///
/// # 🔴 `--cached`: chỉ index, KHÔNG chạm thư mục làm việc
///
/// Staging theo khối đưa thay đổi vào **vùng chờ**; tệp trong thư mục làm việc phải
/// **không đổi một byte** (WORK-04). Bỏ `--cached` làm git ghi cả vào tệp thật, tức
/// hai khối kia biến mất khỏi thư mục làm việc của người dùng.
pub const CO_APPLY: &[&str] = &["apply", "--cached", "--recount"];

/// Mã băm blob hiện tại của một tệp trong thư mục làm việc.
///
/// Đi qua `runner.read`: đây là lệnh **đọc**, và giữ khoá ghi ở đường đọc làm mọi lần
/// làm mới giao diện xếp hàng sau một lệnh ghi đang chạy (xem ghi chú đầu
/// `worktree.rs`).
async fn bam_blob_hien_tai(state: &AppState, repo: &Arc<RepoHandle>, path: &str) -> Result<String> {
    let runner = state.runner(Arc::clone(repo));

    let ra = runner
        .read_ok(
            GitCommand::new(&repo.path)
                .arg("hash-object")
                // 🔴 `--` NGĂN CÁCH cờ với pathspec (T-02-11, T-05-04). `path` đến từ
                // webview, tức chuỗi tuỳ ý: một tệp tên `-w` được git đọc thành **cờ**
                // (và `-w` thì *ghi* object vào kho, một tác dụng phụ ta không muốn),
                // một tệp tên `--stdin` làm git chờ stdin vô hạn cho tới khi quá hạn.
                .arg("--")
                .arg(PATHSPEC_SAU_DAU_GACH(path))
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    Ok(String::from_utf8_lossy(&ra.stdout).trim().to_owned())
}

/// Tệp có còn đúng thứ giao diện đã vẽ diff không — WORK-05, rủi ro R4.
///
/// # Vì sao phép kiểm này đứng TRƯỚC mọi lệnh khác
///
/// Chủ dự án chạy git ở terminal song song; một `git checkout`, một lần lưu tệp trong
/// editor, hay một bộ formatter chạy lúc lưu đều làm tệp đổi giữa lúc giao diện vẽ
/// diff và lúc người dùng bấm stage. Áp một bản vá dựng từ **nội dung cũ** lên một tệp
/// **mới** là đúng ca R1: bản vá áp sai vị trí, tệp người dùng hỏng.
///
/// # 🔴 So BẰNG chuỗi, không so tiền tố
///
/// `starts_with` ở đây là một lỗi có thật, không phải sự cẩn thận thừa: giao diện có
/// thể gửi một hash **rút gọn** (git in hash rút gọn ở khắp nơi), và trên một repo lớn
/// một hash rút gọn khớp tiền tố với một blob **khác** là ca đã biết — đó chính là lý
/// do git tự nới số ký tự rút gọn theo kích thước repo. Một phép so tiền tố biến "tệp
/// đã đổi" thành "tệp không đổi" đúng lúc nó nguy hiểm nhất.
///
/// Có test ghim: `hash_rut_gon_khong_duoc_coi_la_khop`.
pub async fn kiem_blob_hash(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    path: &str,
    mong_doi: &str,
) -> Result<()> {
    let hien_tai = bam_blob_hien_tai(state, repo, path).await?;

    if hien_tai != mong_doi.trim() {
        return Err(GitError::FileChanged {
            path: path.to_owned(),
        });
    }

    Ok(())
}

/// Đưa **một khối** vào vùng chờ (`true`) hoặc lấy ra khỏi vùng chờ (`false`).
///
/// Hai chiều dùng chung một hàm vì chúng khác nhau đúng **hai** chỗ: nguồn bản vá
/// (`git diff` vs `git diff --cached`) và chiều áp (`--reverse` hay không). Tách thành
/// hai bản sao nghĩa là một lần sửa đường `--check` chỉ vào một nửa.
///
/// # Chiều unstage: bản vá lấy từ `--cached`, rồi áp `--reverse`
///
/// Không lấy từ thư mục làm việc: khối đang nằm ở **index**, và chỉ `git diff --cached`
/// mới thấy nó. Rồi `--reverse` gỡ đúng khối đó ra.
async fn ap_mot_khoi(
    state: &AppState,
    repo: Arc<RepoHandle>,
    path: &str,
    hunk_index: usize,
    blob_hash: &str,
    vao_vung_cho: bool,
) -> Result<RepoStatus> {
    // --- Bước 1: tệp có còn đúng thứ giao diện đã vẽ không ------------------
    //
    // Đứng TRƯỚC mọi lệnh khác: thất bại ở đây phải để lại index **y nguyên**, và cách
    // rẻ nhất để bảo đảm điều đó là chưa chạy lệnh nào.
    kiem_blob_hash(state, &repo, path, blob_hash).await?;

    // --- Bước 2: byte thô của bản vá đầy đủ ---------------------------------
    let runner = state.runner(Arc::clone(&repo));

    let mut lenh_diff = GitCommand::new(&repo.path).args(["-c", "core.quotepath=false", "diff"]);
    if !vao_vung_cho {
        // Chiều unstage: khối cần gỡ nằm ở index, không ở thư mục làm việc.
        lenh_diff = lenh_diff.arg("--cached");
    }
    let ra_diff = runner
        .read_ok(
            lenh_diff
                .args(["--unified=3", "--no-color"])
                // 🔴 `--` TRƯỚC pathspec. Ở `git diff` thiếu `--` **im lặng hơn** mọi
                // chỗ khác: `git diff main` là một lệnh HỢP LỆ so HEAD với nhánh
                // `main`, nên một tệp tên `main` cho một diff SAI mà không lỗi nào —
                // rồi bản vá sai đó được áp vào index.
                .arg("--")
                .arg(PATHSPEC_SAU_DAU_GACH(path))
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    // --- Bước 3: tách khối, dựng bản vá con ---------------------------------
    //
    // 🔴 `ra_diff.stdout` — **chỉ** stdout. `stderr` mang cảnh báo của clean filter
    // (`LF will be replaced by CRLF`), và một dòng cảnh báo lọt vào giữa nội dung bản
    // vá làm git từ chối bản vá hoặc áp sai chỗ.
    let tach = tach_hunk_tho(&ra_diff.stdout)?;

    // Tệp nhị phân: `git diff` in `Binary files ... differ` và **không** có `@@`, nên
    // `hunks` rỗng (R7, T-05-06). `dung_ban_va_mot_hunk` trả `None` cho mọi chỉ số —
    // không panic, và không có gì được áp. Chỉ số ngoài biên đi cùng đường: nó đến từ
    // webview, tức là số tuỳ ý.
    let Some(ban_va) = dung_ban_va_mot_hunk(&tach, hunk_index) else {
        return Err(GitError::ParseFailed(format!(
            "không dựng được bản vá cho khối {hunk_index}: bản vá của `{path}` có {} \
             khối (tệp nhị phân cho 0 khối)",
            tach.so_hunk()
        )));
    };

    // --- Bước 4: thử khan --------------------------------------------------
    //
    // `--check` **không đổi gì**. Nó là thứ giữ cho ca thất bại để lại index sạch.
    let mut lenh_check = GitCommand::new(&repo.path).args(CO_APPLY).arg("--check");
    if !vao_vung_cho {
        lenh_check = lenh_check.arg("--reverse");
    }
    let ra_check = runner
        .read(lenh_check.stdin_bytes(ban_va.clone()).timeout(DEFAULT_TIMEOUT))
        .await?;

    if !ra_check.is_success() {
        // 🔴 KHÔNG thử lại. Không `--whitespace=fix`, không `--3way`, không `-C1`.
        //
        // Bản vá không khớp nghĩa là nội dung không còn như lúc bản vá được dựng.
        // Khớp mờ sẽ tìm một chỗ **gần giống** để áp — và khi tệp đã đổi, "gần giống"
        // là một chỗ **sai**. Câu trả lời đúng là nói cho người dùng biết và dừng lại.
        return Err(GitError::FileChanged {
            path: path.to_owned(),
        });
    }

    // --- Bước 5: áp thật ---------------------------------------------------
    //
    // Đi qua `chay_lenh_ghi_co_thu_lai_phan_loai` của 04-02 — DÙNG LẠI, không viết
    // mới. Nó đã có 3 lần thử, giãn cách, và **không bao giờ xoá `.git/index.lock`**.
    // Đó đúng là cơ chế ROADMAP đòi cho ca "một lần tự làm mới chồng lên một lần stage
    // do người dùng khởi tạo" (R5).
    let repo_path = repo.path.clone();
    let duong_dan = path.to_owned();
    chay_lenh_ghi_co_thu_lai_phan_loai(
        state,
        &repo,
        || {
            let mut cmd = GitCommand::new(&repo_path).args(CO_APPLY);
            if !vao_vung_cho {
                cmd = cmd.arg("--reverse");
            }
            cmd.stdin_bytes(ban_va.clone()).timeout(DEFAULT_TIMEOUT)
        },
        |_args, _out| {
            // Thất bại ở bước áp thật sau khi `--check` đã xanh nghĩa là có thứ gì đó
            // đổi giữa hai lệnh — đúng ca R4. Vẫn là `FileChanged`, vẫn không thử lại.
            GitError::FileChanged {
                path: duong_dan.clone(),
            }
        },
    )
    .await?;

    // --- Bước 6: trạng thái MỚI --------------------------------------------
    //
    // Kiểu trả về là chỗ ghim ràng buộc, không phải tiện lợi (CONTEXT.md Phase 4 mục
    // 2.5): thao tác ghi trả trạng thái mới **trực tiếp**, không chờ watcher.
    lay_trang_thai(state, repo).await
}

/// Đưa một khối vào vùng chờ — WORK-03. Tách khỏi `#[tauri::command]` để test tích hợp
/// gọi được mà không cần dựng một `App` có webview.
pub async fn stage_mot_khoi(
    state: &AppState,
    repo: Arc<RepoHandle>,
    path: &str,
    hunk_index: usize,
    blob_hash: &str,
) -> Result<RepoStatus> {
    ap_mot_khoi(state, repo, path, hunk_index, blob_hash, true).await
}

/// Lấy một khối ra khỏi vùng chờ — WORK-03.
pub async fn unstage_mot_khoi(
    state: &AppState,
    repo: Arc<RepoHandle>,
    path: &str,
    hunk_index: usize,
    blob_hash: &str,
) -> Result<RepoStatus> {
    ap_mot_khoi(state, repo, path, hunk_index, blob_hash, false).await
}

/// Đưa một khối vào vùng chờ — WORK-03. Trả trạng thái **mới** (CONTEXT.md 2.5).
#[tauri::command]
pub async fn stage_hunk(
    repo_id: String,
    path: String,
    hunk_index: usize,
    blob_hash: String,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    stage_mot_khoi(&state, repo, &path, hunk_index, &blob_hash).await
}

/// Lấy một khối ra khỏi vùng chờ — WORK-03. Trả trạng thái **mới**.
#[tauri::command]
pub async fn unstage_hunk(
    repo_id: String,
    path: String,
    hunk_index: usize,
    blob_hash: String,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    unstage_mot_khoi(&state, repo, &path, hunk_index, &blob_hash).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thân phần **không-test** của tệp này, đã **bỏ dòng chú thích**.
    ///
    /// # Vì sao bỏ chú thích là bắt buộc, không phải cẩn thận thừa
    ///
    /// Lỗi #1 và #5 trong chín cổng không-thể-fail của dự án (CONTEXT.md 4.1) chết đúng
    /// vì điều này: một cổng grep trên nguồn **thô** khớp một chú thích **do chính
    /// mutation sinh ra**. Tệp này đặc biệt nguy hiểm: doc comment của nó **liệt kê tên
    /// từng cờ bị cấm** (`--whitespace=fix`, `--3way`, ...) để giải thích *vì sao không
    /// dùng*. Một cổng đọc nguồn thô sẽ khớp chính lời giải thích đó và **đỏ vĩnh
    /// viễn** — hoặc, sau khi ai đó "sửa" nó bằng cách nới điều kiện, **xanh vĩnh
    /// viễn**.
    ///
    /// Cắt ở `mod tests` để không đọc chính mã test: test dưới đây có tên các cờ bị cấm
    /// trong chuỗi khẳng định của nó.
    fn than_khong_chu_thich() -> String {
        let src = include_str!("hunk.rs");
        let loc: String = src
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("///") && !t.starts_with("//!")
            })
            .collect::<Vec<_>>()
            .join("\n");

        loc.split("mod tests")
            .next()
            .expect("split luôn cho ít nhất một phần tử")
            .to_owned()
    }

    /// Phép lọc chú thích **thật sự** cắt được `mod tests` và các dòng `//`.
    ///
    /// Không có test này thì phép lọc hỏng theo **hai** hướng đều tạo ra một cổng hỏng,
    /// và không hướng nào gây lỗi biên dịch: lọc quá tay → thân rỗng → mọi cổng xanh;
    /// lọc thiếu → khớp chú thích → cổng đo văn xuôi thay vì đo mã.
    #[test]
    fn phep_loc_chu_thich_hoat_dong()  {
        let than = than_khong_chu_thich();

        assert!(
            !than.contains("mod tests"),
            "🔴 phép cắt `mod tests` hỏng — cổng đang đọc cả mã test, tức nó khớp \
             chính chuỗi khẳng định của chính nó"
        );
        assert!(
            !than.contains("Đây là phase **đầu tiên** ghi vào nội dung tệp"),
            "🔴 một dòng `//!` sống sót qua phép lọc — cổng đang đo văn xuôi"
        );
        assert!(
            than.contains("pub const CO_APPLY"),
            "🔴 phép lọc quá tay: thân đã mất cả khai báo `CO_APPLY`. Một thân rỗng \
             làm MỌI cổng dưới đây xanh"
        );
    }

    /// **Đường apply KHÔNG BAO GIỜ mang một cờ khớp mờ.**
    ///
    /// ROADMAP xếp ràng buộc này ở mức "không thương lượng", và nó là loại ràng buộc mà
    /// một lần "sửa cho nó chạy" trong tương lai sẽ phá — nên nó cần một cổng, không
    /// cần một lời hứa.
    ///
    /// # Vì sao khớp mờ là một cách làm hỏng tệp người dùng
    ///
    /// Khớp mờ bảo git tìm một chỗ **gần giống** để áp bản vá. Ca ta đang xử lý là ca
    /// bản vá **không khớp**, tức tệp đã đổi — và ở đó "gần giống" là một chỗ **sai**.
    /// Đây là phase đầu tiên ghi vào nội dung tệp chứ không chỉ vào index (rủi ro R1),
    /// nên một lần áp sai vị trí là mất dữ liệu thật.
    ///
    /// Đột biến M12 (thêm `--whitespace=fix` vào `CO_APPLY`) phải làm test này **đỏ**.
    /// Đột biến M13 (thêm `--3way` **chỉ trong một chú thích**) phải cho **0 đỏ** — nếu
    /// nó đỏ thì phép lọc chú thích hỏng và mọi kết luận khác từ cổng này vô giá trị.
    #[test]
    fn duong_apply_khong_bao_gio_khop_mo() {
        let than = than_khong_chu_thich();

        // 🔴 Khẳng định **tiền đề** trước (lỗi #3): không tìm thấy thứ cần kiểm thì
        // cổng phải ĐỎ, không phải xanh.
        assert!(
            than.contains("CO_APPLY"),
            "🔴 tiền đề sai: không thấy `CO_APPLY` trong thân đã lọc — cổng đang tìm \
             trong một chuỗi rỗng và sẽ XANH với MỌI mã, kể cả mã có `--3way`"
        );

        for cam in ["--whitespace=fix", "--unidiff-zero", "--3way", "--reject", "-C1"] {
            assert!(
                !than.contains(cam),
                "🔴 `{cam}` là khớp mờ — ROADMAP xếp 'không thương lượng'. Thất bại \
                 phải thành `GitError::FileChanged` (báo và đòi làm mới), KHÔNG thành \
                 một lần thử lại lỏng hơn. Khi tệp đã đổi, 'gần giống' là chỗ SAI"
            );
        }

        // Cổng phủ định một mình **xanh trên một tệp không có lệnh apply nào**, nên nó
        // cần một vế khẳng định đi kèm.
        assert!(
            than.contains("--recount"),
            "🔴 `--recount` phải có trên đường apply: khi chọn một tập con các khối, \
             số dòng trong header `@@` không còn đúng"
        );
    }

    /// `--check` và lần áp thật là **hai** lệnh, và `--check` đứng trước.
    ///
    /// Đây là thứ giữ cho ca thất bại để lại index **sạch** (đột biến M7). Không có nó,
    /// một bản vá khớp một phần được áp một phần và người dùng nhận một index nửa vời.
    #[test]
    fn co_buoc_check_truoc_khi_ap_that() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("CO_APPLY"),
            "🔴 tiền đề sai: thân đã lọc không chứa `CO_APPLY`"
        );
        assert!(
            than.contains("--check"),
            "🔴 thiếu bước `git apply --check`: thất bại sẽ để lại index BẨN vì bản vá \
             được áp một phần trước khi git phát hiện chỗ không khớp"
        );

        let vi_tri_check = than.find("--check").expect("vừa khẳng định là có");
        let sau_check = &than[vi_tri_check..];
        assert!(
            sau_check.contains("chay_lenh_ghi_co_thu_lai_phan_loai"),
            "🔴 lần áp THẬT phải đứng SAU bước `--check`. Thứ tự ngược lại nghĩa là \
             `--check` không bảo vệ gì cả"
        );
    }

    /// 🔴 Bản vá **không bao giờ** đi qua `String` trên đường tới `git apply`.
    ///
    /// # Vì sao đây là cổng bảo vệ WORK-04
    ///
    /// `String::from_utf8_lossy` đổi mỗi byte không hợp lệ thành U+FFFD — **ba** byte —
    /// và `git apply` sẽ ghi ba byte đó vào tệp thật của người dùng. Một tệp Latin-1,
    /// một tệp có BOM, một tệp nhị phân lọt vào đường text đều là dữ liệu có thật.
    ///
    /// Cổng anh em ở `patch_build.rs:667` phủ nửa **tách khối**; cổng này phủ nửa
    /// **áp**. Hai nửa cần hai cổng vì mỗi cổng chỉ đọc `include_str!` của tệp mình.
    ///
    /// Phép kiểm `bam_blob_hien_tai` dùng `from_utf8_lossy` một cách **hợp lệ** — hash
    /// là 40 ký tự hex ASCII, không phải nội dung người dùng — nên cổng cắt tệp ở
    /// `ap_mot_khoi` và chỉ kiểm từ đó trở đi.
    #[test]
    fn duong_apply_khong_bao_gio_giai_ma_ban_va() {
        let than = than_khong_chu_thich();

        let vi_tri = than.find("async fn ap_mot_khoi").unwrap_or_else(|| {
            panic!(
                "🔴 tiền đề sai: không thấy `async fn ap_mot_khoi` trong thân đã lọc. \
                 Cổng này đang tìm trong một chuỗi rỗng và sẽ XANH với MỌI mã. Nếu hàm \
                 đã đổi tên, cập nhật cổng — đừng xoá nó"
            )
        });
        let duong_ban_va = &than[vi_tri..];

        for cam in [
            "from_utf8_lossy",
            "String::from_utf8",
            "to_string_lossy",
            "str::from_utf8",
        ] {
            assert!(
                !duong_ban_va.contains(cam),
                "🔴 `{cam}` trên đường bản vá: nó đổi mỗi byte không hợp lệ thành \
                 U+FFFD (BA byte), và git sẽ ghi ba byte đó vào tệp THẬT của người \
                 dùng. WORK-04 đòi byte không UTF-8 giữ nguyên sau staging một phần"
            );
        }

        // Vế khẳng định: đường này thật sự có đưa byte vào git.
        assert!(
            duong_ban_va.contains("stdin_bytes"),
            "🔴 bản vá phải đi vào git qua `stdin_bytes` (`Vec<u8>`), không qua một \
             tham số chuỗi"
        );
    }

    /// stderr **không bao giờ** ghép vào nội dung bản vá.
    ///
    /// Đo được hôm nay: `git` in `warning: in the working copy of 'a.txt', LF will be
    /// replaced by CRLF` khi clean filter chạy. Một dòng cảnh báo lọt vào giữa bản vá
    /// làm git từ chối bản vá, hoặc — tệ hơn — áp sai chỗ. `GitOutput` đã tách sẵn hai
    /// luồng; việc duy nhất phải làm là **không ghép** (đột biến M10).
    #[test]
    fn stderr_khong_bao_gio_vao_ban_va() {
        let than = than_khong_chu_thich();

        let vi_tri = than
            .find("async fn ap_mot_khoi")
            .expect("🔴 tiền đề sai: không thấy `ap_mot_khoi`");
        let duong_ban_va = &than[vi_tri..];

        assert!(
            duong_ban_va.contains("ra_diff.stdout"),
            "🔴 tiền đề sai: không thấy chỗ lấy `ra_diff.stdout` — cổng không đo được gì"
        );
        assert!(
            !duong_ban_va.contains("ra_diff.stderr"),
            "🔴 `stderr` của `git diff` KHÔNG được vào bản vá: nó mang cảnh báo của \
             clean filter (`LF will be replaced by CRLF`), và một dòng cảnh báo giữa \
             nội dung bản vá làm git áp sai chỗ"
        );
    }

    /// **`--` phải đứng TRƯỚC pathspec trong MỌI lệnh của tệp này** — T-05-04.
    ///
    /// Đọc thân hàm **thật**, không dựng lệnh git riêng: bài học HIST-10 là một test tự
    /// dựng lệnh chứng minh `--` *có tác dụng* mà không chứng minh hàm thật *dùng* nó.
    ///
    /// `path` đến từ webview, tức chuỗi tuỳ ý. Thiếu `--` thì `git diff main` là một
    /// lệnh **hợp lệ** so HEAD với nhánh `main` — một diff SAI, không lỗi nào — và bản
    /// vá sai đó đi thẳng vào index.
    #[test]
    fn moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("GitCommand::new"),
            "🔴 tiền đề sai: phép lọc chú thích phải giữ lại được thân hàm thật"
        );

        let so_moc = than.matches("PATHSPEC_SAU_DAU_GACH").count();
        assert_eq!(
            so_moc, 3,
            "ba vị trí mốc trong mã không-chú-thích: khai báo `fn \
             PATHSPEC_SAU_DAU_GACH`, chỗ gọi trong `bam_blob_hien_tai`, và chỗ gọi \
             trong `ap_mot_khoi`. Số khác 3 nghĩa là một lệnh mất mốc, hoặc có lệnh mới \
             mang pathspec mà chưa được cổng này kiểm. Đọc được {so_moc}"
        );

        // Kiểm trong phạm vi TỪNG LỆNH, không kiểm cả tệp như một khối: một đại lượng
        // toàn cục không đo được một bất biến cục bộ (xem khối "Bản đầu của cổng này
        // VÔ DỤNG" trong `diff.rs`). `--` của một lệnh phía trên vẫn nằm trước mọi mốc
        // ngay cả khi lệnh phía dưới mất `--` của nó.
        let cac_lenh: Vec<&str> = than.split("GitCommand::new").skip(1).collect();
        for (i, lenh) in cac_lenh.iter().enumerate() {
            if !lenh.contains("PATHSPEC_SAU_DAU_GACH") {
                // Lệnh không mang pathspec (`git apply` đọc bản vá từ stdin).
                continue;
            }
            let vi_tri_gach = lenh.find(r#".arg("--")"#).unwrap_or_else(|| {
                panic!(
                    "🔴 lệnh git thứ {i} mang pathspec nhưng KHÔNG có `.arg(\"--\")`. \
                     Một tệp tên `main` sẽ được git đọc thành một REVISION"
                )
            });
            let vi_tri_moc = lenh
                .find("PATHSPEC_SAU_DAU_GACH(")
                .expect("vừa khẳng định là có");
            assert!(
                vi_tri_gach < vi_tri_moc,
                "🔴 lệnh git thứ {i}: `--` phải đứng TRƯỚC pathspec, không phải sau"
            );
        }
    }

    /// `CO_APPLY` mang đúng những gì nó phải mang.
    ///
    /// Đây là hằng mà test tích hợp `tests/hunk_commands.rs` đọc để chứng minh fixture
    /// cắt-thân phân biệt được `--recount`. Nếu nó mất `--recount`, test tích hợp Test B
    /// bước 3 đỏ; test này làm cùng phép khẳng định ở tầng `--lib` để lỗi hiện ra ngay
    /// cả khi `--tests` bị chặn bởi một `git-plum.exe` đang chạy.
    #[test]
    fn co_apply_mang_recount_va_cached() {
        assert!(
            CO_APPLY.contains(&"--recount"),
            "🔴 `--recount` là bắt buộc: khi chọn một tập con các khối, số dòng trong \
             header `@@` không còn đúng và git sẽ từ chối bản vá — hoặc áp sai vị trí. \
             CO_APPLY = {CO_APPLY:?}"
        );
        assert!(
            CO_APPLY.contains(&"--cached"),
            "🔴 `--cached` là bắt buộc: staging theo khối đưa thay đổi vào VÙNG CHỜ. \
             Thiếu nó, git ghi cả vào tệp thật và hai khối kia biến mất khỏi thư mục \
             làm việc của người dùng (WORK-04). CO_APPLY = {CO_APPLY:?}"
        );
        assert_eq!(
            CO_APPLY[0], "apply",
            "subcommand phải đứng đầu argv: `them_no_ext_diff` tiêm `--no-ext-diff` \
             NGAY SAU subcommand (f5c4c17), và nó xác định vị trí đó theo phần tử đầu"
        );
    }
}
