//! Command xem khác biệt một tệp — DIFF-01, DIFF-06.
//!
//! # Thứ tự các bước là phần quan trọng nhất của tệp này
//!
//! DIFF-06 đòi "thông báo phù hợp **trong thời gian bình thường**, không đứng hình".
//! Điều đó không phải là *trả về* `kind: tooLarge` — một cài đặt chạy `git diff` trên
//! tệp 200 MB rồi mới xem kích thước cũng trả về đúng giá trị đó, sau khi đã nạp
//! 200 MB vào RAM và bắt người dùng chờ. Điều đó là **không chạy lệnh nào** trên tệp
//! vượt ngưỡng.
//!
//! Vì vậy thứ tự dưới đây là hợp đồng, không phải một chi tiết cài đặt:
//!
//! 1. tra cache — hit thì **không lệnh git nào**
//! 2. tìm vế trái (`rev-parse <sha>^`, thất bại → cây rỗng)
//! 3. **cổng kích thước** (`cat-file --batch-check`) — biết kích thước mà **không đọc byte nào**
//! 4. cổng LFS — chỉ khi phía mới < 1 KB
//! 5. cổng nhị phân (`diff --numstat`) — phán quyết của chính git
//! 6. chỉ tới đây mới `git diff` thật
//!
//! Sự khác biệt giữa "chặn trước" và "lọc sau" **không quan sát được** từ giá trị trả
//! về; nó chỉ quan sát được từ `CommandLog`. Đó là lý do test của tệp này đọc nhật ký
//! lệnh chứ không chỉ đọc kết quả.
//!
//! # stdout và stderr không bao giờ trộn vào nhau
//!
//! `GitOutput` đã tách sẵn hai luồng, nên ở đây chỉ cần **không làm gì** để phá điều
//! đó. Quan trọng ở đúng chỗ này vì git in cảnh báo chuyển đổi dòng ra **stderr**:
//!
//! ```text
//! warning: in the working copy of 'crlf.txt', LF will be replaced by CRLF
//! ```
//!
//! Một cảnh báo lọt vào stdout sẽ được `parse_patch` đọc thành một dòng diff giả —
//! dòng đó bắt đầu bằng `w`, tức rơi vào nhánh "tiền tố lạ" và chỉ làm tăng
//! `skipped_lines`. Im lặng, và sai (T-03-15).

use std::sync::Arc;

use tauri::State;

use crate::cache::DiffKey;
use crate::domain::diff::{DiffKind, FileDiff, FileHistory, Hunk, LineKind};
use crate::error::{GitError, Result};
use crate::git::exec::DEFAULT_TIMEOUT;
use crate::git::parsers::file_history::{parse_file_history, FILE_HISTORY_FORMAT};
use crate::git::parsers::patch::parse_patch;
use crate::git::parsers::word_diff::{parse_word_diff, WORD_DIFF_REGEX};
use crate::git::GitCommand;
use crate::state::{AppState, RepoHandle};

/// Ngưỡng kích thước blob cho DIFF-06 — **5 MB mỗi phía**.
///
/// # Vì sao 5 MB, không phải 200 MB
///
/// ROADMAP nêu "tệp 200MB" làm **ví dụ tiêu chí**, không phải ngưỡng. Ngưỡng phải là
/// mức mà **trình xem** còn dùng được, không phải mức mà máy còn chịu nổi:
///
/// * 5 MB văn bản là ~100 nghìn dòng. CodeMirror ảo hoá được, nhưng việc **tính** diff
///   (đường A) hoặc dựng decoration (đường B) trên 100 nghìn dòng đã nằm ngoài mọi
///   ngân sách thời gian mà checkpoint #3 đặt ra.
/// * 5 MB cũng là ngưỡng mà GitHub từ chối hiện diff trong giao diện web — một mốc đã
///   được kiểm nghiệm trên lượng người dùng rất lớn.
/// * Ngưỡng thấp mà **nói rõ** ("tệp 8 MB, vượt ngưỡng 5 MB") tốt hơn ngưỡng cao mà
///   treo. DIFF-06 đòi thông báo phù hợp trong thời gian bình thường.
///
/// # Áp cho **mỗi phía**, dùng `max(cũ, mới)`
///
/// Một tệp 10 MB **bị xoá** cũng phải bị chặn, dù phía mới là 0 byte. Lấy trung bình
/// hay chỉ nhìn phía mới đều để lọt ca đó.
pub const MAX_DIFF_BLOB_BYTES: u64 = 5 * 1024 * 1024;

/// Chỉ đọc nội dung blob để kiểm con trỏ LFS khi nó nhỏ hơn mức này.
///
/// Con trỏ LFS theo spec v1 **luôn** ba dòng ASCII, ~130 byte. Chặn ở 1 KB làm cổng
/// LFS gần như miễn phí: ta chỉ đọc những blob quá nhỏ để là bất cứ thứ gì khác.
/// Không có chặn này thì "kiểm nội dung" nghĩa là đọc mọi tệp, và cổng kích thước ở
/// bước trước mất tác dụng.
const LFS_POINTER_MAX_BYTES: u64 = 1024;

/// Dòng đầu bắt buộc của một con trỏ Git LFS (spec v1).
const LFS_MAGIC: &[u8] = b"version https://git-lfs.github.com/spec/v1";

/// Mã của **cây rỗng** trong git — hằng số của chính git, giống nhau ở mọi repo.
///
/// Bản gốc là `EMPTY_TREE` trong [`crate::commands::history`], nơi có phần giải thích
/// đầy đủ. Khai lại ở đây vì ở đó nó `private`; có test đọc chéo để hai hằng không
/// bao giờ lệch nhau.
const CAY_RONG: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";

/// Số dòng ngữ cảnh khi hiện **toàn tệp** — mặc định của trình xem.
///
/// # Vì sao toàn tệp, không phải `--unified=3`
///
/// Người dùng báo (2026-09-22): *"đáng nhẽ nó phải diff toàn bộ file chứ nhỉ đâu chỉ là
/// mỗi phần thay đổi đâu"*. Với `--unified=3`, trình xem chỉ hiện các khối đổi kèm 3 dòng
/// ngữ cảnh, nên **số dòng nhảy** (đo trên ảnh người dùng gửi: 5 → 24 → 37 → 39) và người
/// đọc mất ngữ cảnh của tệp. Ảnh tham chiếu ở `docs/screenshots/` hiện toàn tệp, số dòng
/// chạy liên tục.
///
/// # Vì sao 1 000 000, không phải `usize::MAX`
///
/// `git diff -U<n>` nhận một số hữu hạn; `usize::MAX` làm git từ chối tham số. Một triệu
/// dòng lớn hơn mọi tệp mà cổng [`MAX_DIFF_BLOB_BYTES`] (5 MB) cho đi qua: 5 MB văn bản ở
/// mức ~50 byte/dòng là ~100 nghìn dòng, còn xa một triệu. Nên trong thực tế con số này
/// luôn nghĩa là "toàn tệp".
const UNIFIED_TOAN_TEP: usize = 1_000_000;

/// Ngưỡng lùi về diff rút gọn: tệp lớn hơn mức này **không** hiện toàn tệp.
///
/// # Đánh đổi mà hằng số này giải
///
/// Hiện toàn tệp nghĩa là `MergeView` phải diff và render **toàn bộ** hai tài liệu, không
/// phải vài khối. Đó đúng là rủi ro mà validation checkpoint #3 đặt ra — và phép đo của
/// checkpoint đó **đã bị bỏ qua** (xem `docs/09-phase3-diff-decision.md` mục 7-8), nên ta
/// không có con số nào về việc `MergeView` chịu được bao nhiêu.
///
/// Thiếu số đo thì cách đúng là một ngưỡng thủ công, **và nói cho người dùng biết** khi nó
/// kích hoạt — xem `DiffKind::Text::context_only`. Im lặng lùi về rút gọn sẽ tái diễn đúng
/// việc vừa xảy ra: người dùng thấy số dòng nhảy rồi tưởng là lỗi, báo hai lần.
///
/// # Vì sao đo bằng BYTE, không bằng số dòng
///
/// Đếm dòng cần đọc nội dung — tức thêm một lệnh git, hoặc nạp blob chỉ để đếm. Byte thì
/// [`kich_thuoc_hai_phia`] đã lấy sẵn bằng `cat-file --batch-check` **mà không đọc nội
/// dung**, và cổng [`MAX_DIFF_BLOB_BYTES`] ở ngay trước cũng dùng chính con số đó. Dùng
/// cùng một phép đo cho cả hai cổng nghĩa là không có lệnh git nào thêm, và hai ngưỡng
/// không thể lệch đơn vị.
///
/// # Vì sao 512 KB
///
/// * Cổng [`MAX_DIFF_BLOB_BYTES`] là 5 MB. Ngưỡng này thấp hơn **mười lần**, nên nó thật
///   sự kích hoạt trước khi tệp đủ lớn để làm trình xem khó thở — một cổng nằm sát 5 MB
///   thì gần như không bao giờ chạy.
/// * 512 KB văn bản ở mức ~50 byte/dòng là ~10 nghìn dòng. Cùng cỡ với `MAX_PAGE_LIMIT`
///   của Phase 2 (10 000 hàng commit, đã biết là cuộn mượt ở mức đó) — mốc duy nhất trong
///   dự án này có kinh nghiệm thật đằng sau.
/// * `yarn.lock` 632 KB của repo công ty — tệp mà checkpoint #3 định đo — nằm **trên**
///   ngưỡng này. Có chủ ý: đó đúng là ca ta không có số đo, nên nó phải đi đường an toàn.
///
/// Con số này **là phỏng đoán có căn cứ, không phải kết quả đo.** Khi nào có số thật của
/// checkpoint #3 thì xem lại nó cùng lúc.
const MAX_BYTE_TOAN_TEP: u64 = 512 * 1024;

/// Chọn `-U<n>` cho một tệp: toàn tệp, hay rút gọn về 3 dòng ngữ cảnh.
///
/// Trả [`UNIFIED_TOAN_TEP`] cho tệp dưới [`MAX_BYTE_TOAN_TEP`], `3` cho tệp lớn hơn.
///
/// Tách thành hàm riêng để test được **cả hai nhánh** mà không cần repo — và để có đúng
/// một chỗ quyết định. Hai lệnh git (diff chính và word-diff) đọc cùng kết quả của hàm
/// này, nên chúng không thể lệch nhau.
fn so_dong_ngu_canh(byte_lon_nhat: u64) -> usize {
    if byte_lon_nhat > MAX_BYTE_TOAN_TEP {
        3
    } else {
        UNIFIED_TOAN_TEP
    }
}

/// Kích thước hai phía của một blob, đọc **mà không nạp nội dung**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct KichThuocHaiPhia {
    cu: u64,
    moi: u64,
}

impl KichThuocHaiPhia {
    /// Phía lớn hơn — con số đem so với ngưỡng.
    fn lon_nhat(&self) -> u64 {
        self.cu.max(self.moi)
    }

    /// Cả hai phía đều không tồn tại: tệp không có ở cả hai commit.
    fn ca_hai_deu_thieu(&self, cu_ton_tai: bool, moi_ton_tai: bool) -> bool {
        let _ = self;
        !cu_ton_tai && !moi_ton_tai
    }
}

/// Tra một repo đang mở theo `repo_id` — T-02-12, T-03-09.
///
/// Sao lại `repo_cua` của `history.rs`: `get_repo` chỉ trả repo **đã mở** qua
/// `open_repository`, nên một `repo_id` bịa từ webview cho `UnknownRepository` chứ
/// không mở được thư mục tuỳ ý. Đường dẫn không bao giờ đến từ webview.
fn repo_cua(state: &AppState, repo_id: &str) -> Result<Arc<RepoHandle>> {
    state
        .get_repo(repo_id)
        .ok_or_else(|| GitError::UnknownRepository(repo_id.to_owned()))
}

/// Mốc đánh dấu vị trí `path` trong argv của lệnh diff.
///
/// Hàm đồng nhất — nó **không** biến đổi gì. Nó tồn tại để test khẳng định được *thứ
/// tự* của `--` và `path` bằng cách tìm một chuỗi ổn định trong mã nguồn. Neo vào tên
/// biến `path` không được: chữ đó xuất hiện khắp tệp (kể cả `repo.path`), nên phép
/// `find` sẽ khớp một chỗ khác và cổng trở thành vô dụng.
///
/// Cùng mẫu với `PATH_SAU_DAU_GACH` của `diff_spike.rs`, và cùng lý do: bài học
/// HIST-10, nơi mutation xoá `.arg("--")` cho **0** test đỏ vì test tự dựng lệnh git
/// riêng thay vì đọc thân hàm thật.
#[allow(non_snake_case)]
fn PATHSPEC_SAU_DAU_GACH(path: &str) -> &str {
    path
}

/// Lấy diff của `(commit_id, path)`, qua cache.
///
/// Tách khỏi `#[tauri::command]` để test tích hợp gọi được mà không cần dựng một `App`
/// có webview — cùng cách `history_commands.rs` và `diff_spike.rs` làm.
///
/// # Hạn giờ: [`DEFAULT_TIMEOUT`] (30 giây), **không** mượn `HISTORY_TIMEOUT`
///
/// Đây là diff của **một tệp**, không phải `git log --all` trên toàn lịch sử. Mượn
/// hằng số của nhóm khác khiến việc chỉnh hạn giờ lịch sử về sau vô tình đổi hành vi
/// ở đây — đúng khuôn lập luận mà 02-04 dùng khi từ chối mượn `NETWORK_TIMEOUT`. Vẫn
/// **phải có** một hạn giờ: thiếu nó thì một repo hỏng treo giao diện vĩnh viễn.
pub async fn lay_diff_tep(
    state: &AppState,
    repo: Arc<RepoHandle>,
    commit_id: &str,
    path: &str,
) -> Result<Arc<FileDiff>> {
    // --- Bước 1: tra cache -------------------------------------------------
    //
    // TRƯỚC mọi thứ khác. Hit thì không lệnh git nào chạy, nên không dòng nào vào
    // `CommandLog` (T-02-15: nhật ký ghi lệnh git, không ghi lời gọi IPC). Đây là
    // tiêu chí thành công số 5 của phase, và đếm entry nhật ký là cách duy nhất
    // chứng minh nó — đo thời gian thì nhiễu.
    let khoa = DiffKey::new(repo.id.clone(), commit_id, path);
    if let Some(da_co) = state.diff_cache.get_diff(&khoa) {
        return Ok(da_co);
    }

    let runner = state.runner(Arc::clone(&repo));

    // --- Bước 2: vế trái ---------------------------------------------------
    //
    // `rev-parse <sha>^` thoát khác 0 trên commit gốc; đó là câu trả lời "không có
    // cha", không phải lỗi (câu 6 của CONTEXT.md — so với cây rỗng, đúng như
    // `get_commit_detail` đã làm).
    //
    // T-03-08: `commit_id` đi qua `rev-parse` trước. Một `commit_id` bắt đầu bằng `-`
    // làm `rev-parse` thất bại, nên nó không bao giờ tới lệnh diff ở dạng nguyên bản.
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

    // --- Bước 3: cổng kích thước, TRƯỚC khi chạy `git diff` ----------------
    //
    // `cat-file --batch-check` cho kích thước chính xác mà **không đọc byte nào** của
    // nội dung. Đọc blob rồi đo `len()` thì đã nạp 200 MB vào RAM — thua trước khi
    // kiểm (T-03-10).
    let (kich_thuoc, cu_ton_tai, moi_ton_tai) =
        kich_thuoc_hai_phia(state, &repo, &ve_trai, commit_id, path).await?;

    // Tệp không có ở **cả hai** phía nghĩa là `path` không tồn tại trong cặp commit
    // đó. Đây KHÁC với "tệp không đổi": giao diện phải nói khác nhau, nên trả `Err`
    // chứ không trả `Text` với 0 hunk.
    if kich_thuoc.ca_hai_deu_thieu(cu_ton_tai, moi_ton_tai) {
        return Err(GitError::ParseFailed(format!(
            "không có tệp '{path}' trong commit {commit_id} (cũng không có ở commit cha)"
        )));
    }

    if kich_thuoc.lon_nhat() > MAX_DIFF_BLOB_BYTES {
        return Ok(ket_thuc(
            state,
            khoa,
            path,
            "M",
            DiffKind::TooLarge {
                size: kich_thuoc.lon_nhat(),
                limit: MAX_DIFF_BLOB_BYTES,
            },
        ));
    }

    // --- Bước 4: cổng LFS, chỉ khi phía mới đủ nhỏ -------------------------
    //
    // Nhận biết bằng **nội dung blob**, KHÔNG bằng `git check-attr` (câu 3 của
    // CONTEXT.md). Đã đo: `check-attr -z filter` trả `unspecified` cho một con trỏ LFS
    // thật khi repo không có `.gitattributes`, vì nó trả lời câu "repo này có **cấu
    // hình** lfs cho path này không", không trả lời câu "blob này **có phải** con trỏ
    // lfs không". Repo đã clone mà chưa cài git-lfs, hay tệp con trỏ còn sót sau khi
    // `.gitattributes` bị xoá, đều cho `unspecified`.
    if moi_ton_tai && kich_thuoc.moi > 0 && kich_thuoc.moi < LFS_POINTER_MAX_BYTES {
        if let Some(kind) = thu_doc_con_tro_lfs(state, &repo, commit_id, path).await? {
            return Ok(ket_thuc(state, khoa, path, "M", kind));
        }
    }

    // --- Bước 5: cổng nhị phân --------------------------------------------
    //
    // `--numstat` in `-\t-` cho tệp nhị phân. Đây là phán quyết của **chính git**,
    // rẻ hơn và đúng hơn phép đoán "có byte 0 trong 8000 byte đầu" tự viết.
    // Đầu ra rỗng nghĩa là hai phía giống nhau.
    let ra_numstat = runner
        .read(
            GitCommand::new(&repo.path)
                .args([
                    "-c",
                    "core.quotepath=false",
                    "diff",
                    "--numstat",
                    "-z",
                    "--find-renames",
                    &ve_trai,
                    commit_id,
                ])
                .arg("--")
                .arg(PATHSPEC_SAU_DAU_GACH(path))
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    if ra_numstat.stdout.is_empty() {
        return Ok(ket_thuc(state, khoa, path, "M", DiffKind::Unchanged));
    }

    if la_nhi_phan(&ra_numstat.stdout) {
        return Ok(ket_thuc(
            state,
            khoa,
            path,
            "M",
            DiffKind::Binary {
                old_size: kich_thuoc.cu,
                new_size: kich_thuoc.moi,
            },
        ));
    }

    // --- Bước 6: trạng thái và đường dẫn cũ --------------------------------
    let (status, old_path) = trang_thai_va_ten_cu(state, &repo, &ve_trai, commit_id, path).await?;

    // --- Bước 6b: toàn tệp hay rút gọn -------------------------------------
    //
    // Người dùng muốn thấy **toàn bộ** tệp, không chỉ các khối đổi (2026-09-22). Quyết
    // định ở đây, TRƯỚC khi dựng lệnh, vì con số này phải đi vào **cả hai** lệnh: lệnh
    // diff ở bước 7 và lệnh word-diff trong [`gan_khoang_muc_tu`]. Lệch nhau thì hai
    // đầu ra có tập dòng khác nhau và phép khớp theo số dòng trượt ở biên hunk — xem
    // doc comment của lệnh word-diff.
    //
    // Dùng `kich_thuoc` đã đọc ở bước 3 (`cat-file --batch-check`), nên **không thêm
    // lệnh git nào**. Đó là lý do ngưỡng đo bằng byte chứ không bằng số dòng — đếm dòng
    // cần đọc nội dung.
    let unified = so_dong_ngu_canh(kich_thuoc.lon_nhat());
    let rut_gon = unified != UNIFIED_TOAN_TEP;
    let unified_arg = format!("--unified={unified}");

    // --- Bước 7: chỉ tới đây mới chạy diff thật ----------------------------
    //
    // Pathspec phải gồm **cả hai** đường dẫn khi tệp bị đổi tên. Xem
    // [`trang_thai_va_ten_cu`] để biết vì sao — tóm tắt: git phát hiện đổi tên bằng
    // cách so *tập* tệp bị xoá với *tập* tệp được thêm, nên một pathspec chỉ có tên
    // mới lọc mất tên cũ TRƯỚC khi phép phát hiện chạy, và git báo `new file` với
    // **toàn bộ** nội dung là dòng thêm thay vì một hunk sửa vài dòng.
    let mut lenh_diff = GitCommand::new(&repo.path)
        .args([
            "-c",
            "core.quotepath=false",
            "diff",
            &unified_arg,
            "--find-renames",
            &ve_trai,
            commit_id,
        ])
        // `--` NGĂN CÁCH revision với pathspec (T-02-11, T-03-07). Một `path` trùng
        // tên nhánh (`main`, `HEAD`) thiếu `--` sẽ được git đọc thành **revision**.
        // `path` đến từ webview, tức chuỗi tuỳ ý.
        .arg("--")
        .arg(PATHSPEC_SAU_DAU_GACH(path));
    if let Some(cu) = old_path.as_deref() {
        lenh_diff = lenh_diff.arg(PATHSPEC_SAU_DAU_GACH(cu));
    }

    let ra_diff = runner.read_ok(lenh_diff.timeout(DEFAULT_TIMEOUT)).await?;

    // `parse_patch` chỉ nhận **stdout**. Xem ghi chú T-03-15 ở đầu module.
    let phan_tich = parse_patch(&ra_diff.stdout);

    if phan_tich.skipped_lines > 0 {
        // T-03-16: log ghi `repo.id` (mã băm) và **số**, KHÔNG ghi đường dẫn tệp —
        // khuôn T-02-24 của Phase 2.
        tracing::warn!(
            repo = %repo.id,
            skipped_lines = phan_tich.skipped_lines,
            "bộ phân tích bản vá bỏ qua một số dòng không hiểu"
        );
    }

    // --- Bước 8: diff mức TỪ (03-03) --------------------------------------
    //
    // Nằm SAU `parse_patch` và **TRƯỚC** `put_diff`. Thứ tự này là hợp đồng, không
    // phải sở thích: `spans` phải nằm trong mục cache, nếu không lần mở thứ hai của
    // cùng một tệp mất word-level — và đó đúng là thao tác thường gặp nhất (bấm qua
    // lại giữa các tệp trong một commit).
    let mut hunks = phan_tich.hunks;
    gan_khoang_muc_tu(state, &repo, &ve_trai, commit_id, path, unified, &mut hunks).await;

    let kind = DiffKind::Text {
        hunks,
        truncated: phan_tich.truncated,
        context_only: rut_gon,
    };

    Ok(ket_thuc_voi_ten_cu(
        state, khoa, path, &status, old_path, kind,
    ))
}

/// Chặn trên số dòng **đã sửa** để còn chạy word-level — T-03-19, T-03-20.
///
/// Vượt thì bỏ hẳn lệnh git thứ hai và để mọi `spans` rỗng.
///
/// # Vì sao có chặn trên, và vì sao là 2000
///
/// Word-level tốn **một lệnh git thứ hai** cho mỗi lần mở diff, và `spans` làm payload
/// JSON phình theo số dòng sửa. Một tệp với 50 nghìn dòng sửa (tệp sinh tự động bị ghi
/// đè toàn bộ, tệp lock) thì:
///
/// * không ai đọc word-level trên nó — ở mức đó người dùng đọc "cả tệp đổi", không
///   đọc "chữ nào trong dòng đổi";
/// * `spans` cho 50 nghìn dòng là hàng trăm nghìn cặp số trong JSON.
///
/// 2000 dòng sửa là ngưỡng mà một người còn cuộn qua được trong một lần đọc. Vượt thì
/// **suy giảm có chủ ý**: giao diện vẫn đúng, chỉ bớt mịn — cùng khuôn với
/// `MAX_VISIBLE_LANES` và chỉ báo `+N cha nữa` của Phase 2.
pub const WORD_DIFF_MAX_CHANGED_LINES: usize = 2_000;

/// Chạy lệnh word-diff và gắn khoảng vào các `DiffLine` đã phân tích — 03-03.
///
/// Không trả `Result`: word-level là phần **trang trí**. Mọi đường thất bại đều để
/// `spans` rỗng và ghi `tracing::warn!`, không bao giờ làm hỏng cả lời gọi. Mất phần
/// tô chữ thì người dùng vẫn đọc được diff; mất cả diff thì không.
async fn gan_khoang_muc_tu(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    ve_trai: &str,
    commit_id: &str,
    path: &str,
    // `unified` PHẢI là đúng con số mà lệnh diff chính đã dùng — người gọi truyền xuống
    // thay vì hàm này tự quyết, để hai lệnh không thể lệch. Xem `so_dong_ngu_canh`.
    unified: usize,
    hunks: &mut [Hunk],
) {
    // --- Ba cổng bỏ qua, mỗi cổng tránh một loại lệnh git vô ích -----------
    //
    // Đếm trước khi chạy. `added == 0` (tệp bị xoá) và `removed == 0` (tệp mới) đều
    // làm word-level vô nghĩa: không có cặp dòng nào để so chữ. Chạy lệnh rồi vứt kết
    // quả trả về CÙNG một `FileDiff`, nên khác biệt duy nhất quan sát được nằm trong
    // `CommandLog` — đó là lý do test của bước này đọc nhật ký lệnh.
    let mut them = 0usize;
    let mut xoa = 0usize;
    for l in hunks.iter().flat_map(|h| &h.lines) {
        match l.kind {
            LineKind::Added => them += 1,
            LineKind::Removed => xoa += 1,
            LineKind::Context => {}
        }
    }

    if them == 0 || xoa == 0 || them + xoa > WORD_DIFF_MAX_CHANGED_LINES {
        return;
    }

    // --- Lệnh git thứ hai --------------------------------------------------
    //
    // `--word-diff=porcelain` với dấu **BẰNG**. Dạng `--word-diff-porcelain` không
    // tồn tại: git thoát 129 và in usage (đã đo). Xem tài liệu của
    // `git::parsers::word_diff` để biết cả phép đo lẫn vì sao lỗi này ồn chứ không
    // im lặng như `%x1f` của 02-04.
    //
    // `--word-diff-regex` là bắt buộc, không phải tinh chỉnh: biên từ mặc định của
    // git **mất** khoảng trắng ngăn cách và làm việc dựng lại dòng nguồn sai. Hằng
    // sống ở module bộ phân tích để hai bên không bao giờ lệch nhau.
    //
    // `--unified=3` phải KHỚP lệnh diff chính ở bước 7. Lệch thì hai đầu ra có tập
    // dòng khác nhau và phép khớp theo số dòng trượt ở biên hunk.
    //
    // Dùng `read` chứ không `read_ok`: lệnh này thất bại thì ta mất phần trang trí,
    // không được mất cả lời gọi.
    let runner = state.runner(Arc::clone(repo));
    let mut lenh = GitCommand::new(&repo.path)
        .args([
            "-c",
            "core.quotepath=false",
            "diff",
            "--word-diff=porcelain",
            &format!("--word-diff-regex={WORD_DIFF_REGEX}"),
            &format!("--unified={unified}"),
            "--find-renames",
            ve_trai,
            commit_id,
        ])
        // `--` NGĂN CÁCH revision với pathspec (T-02-11, T-03-07), như mọi lệnh khác
        // trong tệp này mang pathspec.
        .arg("--")
        .arg(PATHSPEC_SAU_DAU_GACH(path));
    lenh = lenh.timeout(DEFAULT_TIMEOUT);

    let ra = match runner.read(lenh).await {
        Ok(ra) => ra,
        Err(e) => {
            tracing::warn!(
                repo = %repo.id,
                loi = %e,
                "không chạy được lệnh diff mức từ; bỏ qua word-level cho tệp này"
            );
            return;
        }
    };

    if !ra.is_success() {
        tracing::warn!(
            repo = %repo.id,
            "lệnh diff mức từ thất bại; `spans` để rỗng và diff vẫn trả về bình thường"
        );
        return;
    }

    // `parse_word_diff` chỉ nhận **stdout**. Git in cảnh báo CRLF ra stderr, và một
    // cảnh báo lọt vào stdout sẽ được đọc thành một đoạn từ giả (T-03-22).
    let wd = parse_word_diff(&ra.stdout);

    if wd.skipped > 0 {
        // T-03-23: đầu ra không đọc được hết nghĩa là ta không hiểu định dạng, và gán
        // khoảng theo một phép đọc sai còn tệ hơn không gán. Bỏ TOÀN BỘ, ghi log.
        tracing::warn!(
            repo = %repo.id,
            skipped = wd.skipped,
            "bộ phân tích diff mức từ bỏ qua một số dòng; bỏ toàn bộ `spans` cho tệp này"
        );
        return;
    }

    // --- Khớp vào `DiffLine` theo SỐ DÒNG ----------------------------------
    //
    // Khớp theo số dòng, **không** theo nội dung: hai dòng giống hệt nhau trong một
    // hunk là chuyện thường (fixture `dup-lines.txt` tồn tại để ghim đúng điều đó), và
    // khớp theo nội dung sẽ gán khoảng của dòng này cho dòng kia — đúng dòng, sai chỗ.
    for dong in hunks.iter_mut().flat_map(|h| &mut h.lines) {
        let (ds, so) = match dong.kind {
            LineKind::Removed => (&wd.old_lines, dong.old_line),
            LineKind::Added => (&wd.new_lines, dong.new_line),
            // Dòng ngữ cảnh không đổi nên không có chữ nào để tô. Bỏ qua tường minh
            // chứ không dựa vào "tình cờ không có khoảng": một `WordLine` ngữ cảnh
            // vẫn tồn tại ở cả hai phía, chỉ là `spans` rỗng.
            LineKind::Context => continue,
        };
        let Some(so) = so else { continue };
        let Some(w) = ds.iter().find(|w| w.line_no == so) else {
            continue;
        };

        // --- Cổng an toàn: chỉ gán khi hai bộ phân tích ĐỒNG Ý về dòng đó ---
        //
        // `Span` là chỉ số byte vào `DiffLine::content`, nhưng nó được **tính** trên
        // `WordLine::content`. Hai chuỗi lệch nhau thì gán khoảng sang là tô sai chỗ
        // — hoặc panic ở 03-04 khi chỉ số vượt biên (T-03-17).
        //
        // Có test tích hợp khẳng định hai chuỗi bằng nhau từng byte trên bảy hình
        // dạng tệp, nên đường này KHÔNG được mong đợi chạy. Nó tồn tại vì hậu quả
        // của việc sai là panic ở tầng giao diện, và một repo thật có thể mang hình
        // dạng mà fixture chưa có.
        if w.content != dong.content {
            // T-03-21: log ghi `repo.id` và **số dòng**, KHÔNG ghi nội dung — khuôn
            // T-02-24 của Phase 2. Nội dung dòng là mã nguồn của người dùng.
            tracing::warn!(
                repo = %repo.id,
                dong = so,
                "hai bộ phân tích bất đồng về nội dung một dòng; bỏ `spans` của dòng đó"
            );
            continue;
        }

        // ⚠️ Gán **chỉ** `spans`. KHÔNG chạm `no_newline_at_eof`: porcelain không
        // cung cấp trường đó (nó in `~` bình thường và không in
        // `\ No newline at end of file` — đã đo), nên mọi giá trị suy từ đây đều sai.
        // Trường đó do `parse_patch` đặt ở bước 7.
        dong.spans.clone_from(&w.spans);
    }
}

/// Dựng `FileDiff`, đặt vào cache, trả `Arc`.
fn ket_thuc(
    state: &AppState,
    khoa: DiffKey,
    path: &str,
    status: &str,
    kind: DiffKind,
) -> Arc<FileDiff> {
    ket_thuc_voi_ten_cu(state, khoa, path, status, None, kind)
}

fn ket_thuc_voi_ten_cu(
    state: &AppState,
    khoa: DiffKey,
    path: &str,
    status: &str,
    old_path: Option<String>,
    kind: DiffKind,
) -> Arc<FileDiff> {
    let fd = Arc::new(FileDiff {
        path: path.to_owned(),
        old_path,
        status: status.to_owned(),
        kind,
    });
    state.diff_cache.put_diff(khoa, Arc::clone(&fd));
    fd
}

/// Kích thước blob ở hai phía, **không đọc nội dung** — T-03-10.
///
/// Một lệnh `cat-file --batch-check` với stdin hai dòng. Dạng đầu ra, đo thật trên
/// git 2.54.0.windows.1:
///
/// ```text
/// <oid> SP blob SP <size> LF        ← tồn tại
/// <spec> SP missing LF              ← không tồn tại
/// ```
///
/// `missing` là **bình thường** cho tệp mới thêm (phía cũ thiếu) và tệp bị xoá (phía
/// mới thiếu) — coi phía đó là 0 byte, không `unwrap` và không trả lỗi.
///
/// Trả `(kích thước, phía cũ tồn tại, phía mới tồn tại)`. Hai cờ tồn tại cần thiết vì
/// "0 byte" và "không có" là hai chuyện khác nhau: một tệp rỗng thật cũng cho 0.
async fn kich_thuoc_hai_phia(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    ve_trai: &str,
    commit_id: &str,
    path: &str,
) -> Result<(KichThuocHaiPhia, bool, bool)> {
    let runner = state.runner(Arc::clone(repo));

    let stdin = format!("{ve_trai}:{path}\n{commit_id}:{path}\n").into_bytes();
    let ra = runner
        .read(
            GitCommand::new(&repo.path)
                .args(["cat-file", "--batch-check"])
                .stdin_bytes(stdin)
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    let mut dong = ra.stdout.split(|b| *b == b'\n').filter(|l| !l.is_empty());
    let (cu, cu_co) = doc_batch_check(dong.next().unwrap_or(b""));
    let (moi, moi_co) = doc_batch_check(dong.next().unwrap_or(b""));

    Ok((KichThuocHaiPhia { cu, moi }, cu_co, moi_co))
}

/// Đọc một dòng `cat-file --batch-check`. Trả `(kích thước, tồn tại)`.
fn doc_batch_check(dong: &[u8]) -> (u64, bool) {
    // Cắt `\r` treo: trên Windows git vẫn dùng LF ở đây, nhưng cắt là miễn phí và
    // Phase 2 đã trả giá cho một `\r` treo ở tầng SHA.
    let dong = match dong.last() {
        Some(b'\r') => &dong[..dong.len() - 1],
        _ => dong,
    };

    let phan: Vec<&[u8]> = dong.split(|b| *b == b' ').collect();
    // `<oid> blob <size>` — ba trường. `<spec> missing` — hai trường.
    if phan.len() < 3 || phan[1] != b"blob" {
        return (0, false);
    }
    let so = std::str::from_utf8(phan[2])
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok());
    match so {
        Some(n) => (n, true),
        None => (0, false),
    }
}

/// `git diff --numstat -z` báo tệp nhị phân bằng `-\t-\t<path>\0`.
///
/// Đo thật:
///
/// ```text
/// $ git diff --numstat -z -- c.bin | od -c
/// 0000000   -  \t   -  \t   c   .   b   i   n  \0
/// ```
///
/// Hai dấu gạch ngang thay cho số dòng thêm/xoá. Kiểm **hai trường đầu**, không kiểm
/// cả dòng: đường dẫn nằm sau và nó chứa được bất cứ byte nào.
fn la_nhi_phan(stdout: &[u8]) -> bool {
    stdout.starts_with(b"-\t-\t")
}

/// Đọc con trỏ LFS từ nội dung blob. `None` khi blob không phải con trỏ.
///
/// # `size` là số **trong con trỏ**, không phải kích thước của tệp con trỏ
///
/// Tệp con trỏ ~130 byte; số trong dòng `size` là kích thước của tệp **thật** mà con
/// trỏ đại diện, thường hàng MB. Lẫn hai số này là lỗi dễ mắc nhất ở đây, và nó im
/// lặng: giao diện vẫn hiện một con số, chỉ là sai con số.
async fn thu_doc_con_tro_lfs(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    commit_id: &str,
    path: &str,
) -> Result<Option<DiffKind>> {
    let runner = state.runner(Arc::clone(repo));

    // `git cat-file blob <rev>:<path>` **không có** vị trí `--`. Ở đây `commit_id` là
    // chuỗi đã đi qua bước `rev-parse` phía trên ở dạng `<sha>^`, còn `path` nằm sau
    // dấu hai chấm — vị trí mà git **luôn** đọc là đường dẫn, không bao giờ là cờ.
    let ra = runner
        .read(
            GitCommand::new(&repo.path)
                .args(["cat-file", "blob", &format!("{commit_id}:{path}")])
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    if !ra.is_success() {
        return Ok(None);
    }

    let mut oid: Option<String> = None;
    let mut size: Option<u64> = None;
    let mut co_magic = false;

    for (i, dong) in ra.stdout.split(|b| *b == b'\n').enumerate() {
        let dong = match dong.last() {
            Some(b'\r') => &dong[..dong.len() - 1],
            _ => dong,
        };
        if i == 0 {
            if !dong.starts_with(LFS_MAGIC) {
                return Ok(None);
            }
            co_magic = true;
            continue;
        }
        if let Some(rest) = dong.strip_prefix(b"oid sha256:") {
            oid = std::str::from_utf8(rest).ok().map(|s| s.trim().to_owned());
        } else if let Some(rest) = dong.strip_prefix(b"size ") {
            size = std::str::from_utf8(rest)
                .ok()
                .and_then(|s| s.trim().parse::<u64>().ok());
        }
    }

    match (co_magic, oid, size) {
        // Ba dòng đều có: con trỏ hợp lệ.
        (true, Some(oid), Some(size)) => Ok(Some(DiffKind::LfsPointer { oid, size })),
        // Có dòng `version` nhưng thiếu `oid` hoặc `size`: con trỏ hỏng. Coi như
        // **không phải** con trỏ và đi tiếp đường văn bản — người dùng thấy nội dung
        // thật của tệp, đọc được, thay vì một `LfsPointer` với oid rỗng.
        _ => Ok(None),
    }
}

/// Chữ trạng thái và đường dẫn cũ, từ `git diff --name-status -z --find-renames`.
///
/// Dùng cùng lệnh và cùng định dạng mà `get_commit_detail` (02-04) dùng, để hai command
/// không bao giờ báo hai trạng thái khác nhau cho **cùng một** (commit, tệp).
///
/// # 🔴 Chạy **KHÔNG** pathspec — đã đo, và plan dự đoán sai chỗ này
///
/// Plan 03-02 viết lệnh này kèm `-- <path>`. Đo thật trên git 2.54.0.windows.1 với
/// commit vừa-đổi-tên-vừa-sửa `renamed.txt` → `renamed-new.txt` (77% giống nhau):
///
/// ```text
/// $ git diff --name-status -z --find-renames A B -- renamed-new.txt
/// A\0renamed-new.txt\0                               ← "tệp MỚI"
///
/// $ git diff --name-status -z --find-renames A B
/// R077\0renamed.txt\0renamed-new.txt\0               ← đổi tên, 77%
/// ```
///
/// Nguyên nhân: git phát hiện đổi tên bằng cách ghép **tập** tệp bị xoá với **tập**
/// tệp được thêm. Pathspec lọc `renamed.txt` ra khỏi tập đầu vào **trước khi** phép
/// ghép chạy, nên git chỉ còn thấy một tệp xuất hiện từ hư không và gọi nó là `A`.
///
/// Hệ quả nghiêm trọng hơn nhiều so với một chữ trạng thái sai: cùng lý do đó,
/// `git diff --unified=3 ... -- renamed-new.txt` in `new file mode` và **toàn bộ 8
/// dòng là dòng thêm**, thay vì một hunk sửa đúng một dòng. Người dùng bấm vào một
/// tệp đổi tên sẽ thấy cả tệp sáng xanh thay vì thấy thay đổi thật.
///
/// Vì vậy hàm này chạy **không** pathspec rồi tự tìm bản ghi khớp. Chi phí: với commit
/// đổi nhiều nghìn tệp, đầu ra lớn hơn. Chấp nhận được vì đầu ra `--name-status` là
/// tên tệp chứ không phải nội dung, và `get_commit_detail` đã chạy đúng lệnh này với
/// cùng chi phí ở bước trước đó.
///
/// # `R`/`C` chiếm **hai** đường dẫn
///
/// `R87\0cũ\0mới\0` — đọc nó như bản ghi một đường dẫn làm **mọi** bản ghi sau lệch
/// một nấc: đường dẫn mới bị đọc thành trạng thái. Một lỗi làm hỏng cả danh sách chứ
/// không chỉ một dòng. Cùng cái bẫy mà `parse_name_status` của 02-04 ghi lại.
async fn trang_thai_va_ten_cu(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    ve_trai: &str,
    commit_id: &str,
    path: &str,
) -> Result<(String, Option<String>)> {
    let runner = state.runner(Arc::clone(repo));

    let ra = runner
        .read(
            GitCommand::new(&repo.path)
                .args([
                    "-c",
                    "core.quotepath=false",
                    "diff",
                    "--name-status",
                    "-z",
                    "--find-renames",
                    ve_trai,
                    commit_id,
                ])
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    Ok(chon_ban_ghi_khop(&ra.stdout, path))
}

/// Tìm bản ghi `--name-status -z` có đường dẫn **mới** khớp `path`.
///
/// Tách khỏi phần gọi git để kiểm được bằng test đơn vị trên đúng byte mà git in ra.
/// Không tìm thấy → `("M", None)`: `path` có trong diff (bước trước đã xác nhận blob
/// tồn tại) nhưng bản ghi không khớp vì lý do nào đó; `M` là giả định an toàn nhất và
/// nó chỉ ảnh hưởng chữ trạng thái hiển thị, không ảnh hưởng nội dung hunk.
fn chon_ban_ghi_khop(stdout: &[u8], path: &str) -> (String, Option<String>) {
    let mut truong = stdout
        .split(|b| *b == 0)
        .filter(|f| !f.is_empty())
        .map(|f| String::from_utf8_lossy(f).into_owned());

    while let Some(status) = truong.next() {
        if status.starts_with('R') || status.starts_with('C') {
            // Hai đường dẫn: cũ rồi mới.
            let Some(cu) = truong.next() else { break };
            let Some(moi) = truong.next() else { break };
            if moi == path {
                return (status, Some(cu));
            }
        } else {
            let Some(p) = truong.next() else { break };
            if p == path {
                return (status, None);
            }
        }
    }

    ("M".to_owned(), None)
}

/// Chặn trên số phiên bản trả về cho một tệp — **200 commit**.
///
/// # Vì sao có chặn trên, và vì sao nó đủ
///
/// `--follow` tốn kém vì nó tính điểm tương đồng ở **mỗi** commit đụng tới path, để
/// nhận ra chỗ đổi tên. Không chặn thì chi phí tỉ lệ với **độ dài lịch sử** — trên
/// một repo triệu commit đó là một cách treo giao diện (T-03-33).
///
/// `--max-count=200` biến chi phí đó thành **hằng số**: git dừng ngay khi đủ 200
/// commit, nên không có phép tính tương đồng nào cho phần lịch sử phía sau.
///
/// 200 đủ cho mọi tệp thật. Một tệp có hơn 200 commit sửa nó là tệp mà người dùng
/// không cuộn hết danh sách; chạm chặn thì cờ `truncated` bật và giao diện nói
/// "200 phiên bản gần nhất" — **nói ra**, không im lặng (T-03-38).
///
/// # Và vì sao 200 hàng DOM là lý do không cần ảo hoá
///
/// Chặn ở tầng git nghĩa là `FileHistory.tsx` không bao giờ dựng hơn 200 hàng, nên
/// `useVirtualizer` ở đó là phức tạp không mua được gì (T-03-34). `CommitList` cần nó
/// vì 100 nghìn hàng; đây thì không.
pub const MAX_FILE_HISTORY: usize = 200;

/// Lịch sử thay đổi của **một** tệp — DIFF-05.
///
/// Tách khỏi `#[tauri::command]` để test tích hợp gọi được mà không cần dựng một
/// `App` có webview — cùng cách `lay_diff_tep` làm.
///
/// # Hạn giờ: [`DEFAULT_TIMEOUT`] (30 giây), **không** `HISTORY_TIMEOUT` (120 giây)
///
/// Đây là truy vấn **một tệp** có `--max-count`, không phải `git log --all` trên toàn
/// lịch sử. Mượn hằng số của nhóm khác khiến việc chỉnh hạn giờ lịch sử về sau vô
/// tình đổi hành vi ở đây — đúng khuôn lập luận mà 02-04 dùng khi từ chối mượn
/// `NETWORK_TIMEOUT`. Vẫn **phải có** một hạn giờ, và nó là chặn cứng thứ hai sau
/// `--max-count`: thiếu nó thì một repo hỏng treo giao diện vĩnh viễn (T-02-14).
///
/// # 🔴 KHÔNG cache, và đó là một quyết định có lý do
///
/// Diff của một commit lịch sử là **bất biến** theo `(sha, path)`, nên cache nó là
/// đúng. Lịch sử tệp thì **phụ thuộc HEAD**: một commit mới xuất hiện sẽ đổi kết quả
/// cho cùng một `path`. Cache nó cần một cơ chế vô hiệu hoá, và cái duy nhất đúng là
/// theo dõi thư mục làm việc — thứ chỉ tới ở Phase 4.
///
/// Một truy vấn 200 commit cho một path là rẻ. Cache nó bây giờ là tự tạo ra một lớp
/// dữ liệu cũ mà chưa có gì để dọn: người dùng commit rồi mở lại lịch sử tệp và
/// **không thấy commit của chính mình**. Ghi lý do ở đây để người sau không "tối ưu"
/// nó vào một lỗi.
///
/// # `--no-ext-diff` không phải việc của hàm này
///
/// `GitCommand::run()` chèn nó cho mọi lệnh con có thể gọi trình khác biệt, và `log`
/// nằm trong danh sách đó (xem `them_no_ext_diff`). Thêm lần nữa ở đây là mã chết.
pub async fn lay_lich_su_tep(
    state: &AppState,
    repo: Arc<RepoHandle>,
    path: &str,
) -> Result<FileHistory> {
    let runner = state.runner(Arc::clone(&repo));

    let ra = runner
        .read(
            GitCommand::new(&repo.path)
                // `core.quotepath=false` để tên tệp có ký tự ngoài ASCII về ở dạng
                // byte thô thay vì escape bát phân — bộ phân tích giải mã lossy, nên
                // nó cần byte thật (HIST-11).
                .args(["-c", "core.quotepath=false", "log"])
                .arg("--follow")
                .arg(format!("--max-count={MAX_FILE_HISTORY}"))
                .arg(FILE_HISTORY_FORMAT)
                .args(["--name-status", "-z"])
                // 🔴 `--` TRƯỚC pathspec (T-02-11 / T-03-32). Thiếu nó thì một `path`
                // trùng tên nhánh bị git hiểu thành một **revision**, và người dùng
                // nhận lịch sử của một nhánh thay vì của tệp mình chọn.
                .arg("--")
                .arg(PATHSPEC_SAU_DAU_GACH(path))
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    // git thoát khác 0 khi `path` chưa từng tồn tại. Đó **không** phải lỗi: "tệp này
    // không có trong lịch sử" là một câu trả lời bình thường mà giao diện hiện bằng
    // một câu riêng, khác hẳn ca lỗi. Nên đi tiếp và để bộ phân tích trả danh sách
    // rỗng.
    let phan_tich = parse_file_history(&ra.stdout, MAX_FILE_HISTORY);

    if phan_tich.skipped_records > 0 {
        // Ghi `repo.id` (mã băm) và **số**, KHÔNG ghi `path` — khuôn T-02-24/T-03-36:
        // đường dẫn tệp của người dùng không vào log.
        tracing::warn!(
            repo = %repo.id,
            skipped = phan_tich.skipped_records,
            "bỏ qua bản ghi méo khi đọc lịch sử tệp"
        );
    }

    Ok(FileHistory {
        path: path.to_owned(),
        versions: phan_tich.versions,
        truncated: phan_tich.truncated,
    })
}

/// Lịch sử thay đổi của một tệp — DIFF-05.
///
/// Xem [`lay_lich_su_tep`] cho hạn giờ, chặn trên và lý do **không** cache.
#[tauri::command]
pub async fn get_file_history(
    repo_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<FileHistory> {
    let repo = repo_cua(&state, &repo_id)?;
    lay_lich_su_tep(&state, repo, &path).await
}


/// Diff của một tệp trong một commit — DIFF-01, DIFF-06.
///
/// Xem tài liệu đầu module về thứ tự các bước; nó là hợp đồng của DIFF-06.
#[tauri::command]
pub async fn get_file_diff(
    repo_id: String,
    commit_id: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<FileDiff> {
    let repo = repo_cua(&state, &repo_id)?;
    let fd = lay_diff_tep(&state, repo, &commit_id, &path).await?;
    // Tauri command phải trả giá trị sở hữu để serialize. Clone ở **ranh giới IPC**,
    // không ở trong cache: cache vẫn giữ `Arc` và lời gọi thứ hai vẫn không sinh lệnh
    // git nào — đó là điều tiêu chí thành công số 5 đòi hỏi.
    Ok((*fd).clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mã cây rỗng phải khớp bản gốc trong `history.rs`. Hai hằng ở hai tệp lệch nhau
    /// là lỗi im lặng — bài học Phase 2, ghim bằng test đọc thẳng tệp kia.
    #[test]
    fn hang_cay_rong_khop_ban_goc_trong_history() {
        assert_eq!(CAY_RONG, "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
        let src = include_str!("history.rs");
        assert!(
            src.contains(&format!(r#"EMPTY_TREE: &str = "{CAY_RONG}""#)),
            "CAY_RONG phải khớp EMPTY_TREE trong history.rs"
        );
    }

    /// Ngưỡng 5 MB, đọc **giá trị** chứ không `grep -c`: hằng số xuất hiện ở khai báo,
    /// ở chú thích giải thích con số, và ở test.
    #[test]
    fn nguong_kich_thuoc_dung_nam_mb() {
        assert_eq!(MAX_DIFF_BLOB_BYTES, 5 * 1024 * 1024);
        assert_eq!(MAX_DIFF_BLOB_BYTES, 5_242_880, "con số mà giao diện hiển thị");
    }

    /// Phân tích dòng `cat-file --batch-check` cho cả hai dạng đầu ra thật.
    #[test]
    fn doc_batch_check_ca_hai_dang() {
        let (n, co) = doc_batch_check(b"206878cc12b0a23ba14b9b25bdabde2c0618552e blob 49648");
        assert_eq!((n, co), (49648, true));

        let (n, co) = doc_batch_check(b"deadbeef:khong-co.txt missing");
        assert_eq!(
            (n, co),
            (0, false),
            "`missing` là ca BÌNH THƯỜNG cho tệp thêm/xoá — không được panic"
        );

        let (n, co) = doc_batch_check(b"");
        assert_eq!((n, co), (0, false), "dòng rỗng không được panic");

        // `\r` treo.
        let (n, co) = doc_batch_check(b"abc blob 12\r");
        assert_eq!((n, co), (12, true), "`\\r` treo không được làm hỏng số");
    }

    /// `max(cũ, mới)`: một tệp lớn **bị xoá** (phía mới 0 byte) vẫn phải bị chặn.
    #[test]
    fn nguong_ap_cho_phia_lon_nhat() {
        let bi_xoa = KichThuocHaiPhia {
            cu: 10 * 1024 * 1024,
            moi: 0,
        };
        assert!(
            bi_xoa.lon_nhat() > MAX_DIFF_BLOB_BYTES,
            "tệp 10 MB bị xoá phải vượt ngưỡng dù phía mới là 0 byte"
        );

        let moi_them = KichThuocHaiPhia {
            cu: 0,
            moi: 10 * 1024 * 1024,
        };
        assert!(moi_them.lon_nhat() > MAX_DIFF_BLOB_BYTES);

        let nho = KichThuocHaiPhia { cu: 100, moi: 200 };
        assert!(nho.lon_nhat() < MAX_DIFF_BLOB_BYTES);
    }

    /// Nhận biết nhị phân đọc đúng hai trường đầu của `--numstat -z`.
    #[test]
    fn nhan_biet_nhi_phan_tu_numstat() {
        assert!(la_nhi_phan(b"-\t-\tbinary.png\0"));
        assert!(!la_nhi_phan(b"3\t1\ttext.txt\0"), "tệp văn bản có hai số");
        assert!(!la_nhi_phan(b""), "đầu ra rỗng là hai phía giống nhau");
        assert!(
            !la_nhi_phan(b"-\t1\tla.txt\0"),
            "chỉ một dấu gạch không phải nhị phân"
        );
    }

    /// **T-03-07: `--` phải đứng TRƯỚC pathspec trong CẢ BA lệnh có pathspec.**
    ///
    /// Đọc thân hàm thật, không dựng lệnh git riêng — bài học HIST-10: test tự dựng
    /// lệnh chứng minh `--` *có tác dụng* mà không chứng minh hàm thật *dùng* nó, và
    /// đã kiểm bằng đột biến ở Phase 2 rằng nó cho 0 test đỏ.
    #[test]
    fn moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path() {
        let src = include_str!("diff.rs");
        let (_, than) = src
            .split_once("pub async fn lay_diff_tep")
            .expect("phải có hàm lay_diff_tep");

        // Bỏ dòng chú thích: một `--` trong chú thích không ngăn cách gì cả, và đó
        // đúng là cách một cổng grep trở thành vô dụng.
        let ma: String = than
            .lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        let so_moc = ma.matches("PATHSPEC_SAU_DAU_GACH").count();
        assert_eq!(
            so_moc, 5,
            "năm vị trí pathspec từ `lay_diff_tep` tới hết phần không-test: lệnh \
             --numstat, HAI đường dẫn của lệnh diff thật (tên mới cộng tên cũ khi đổi \
             tên), lệnh --word-diff của bước mức từ (03-03), và lệnh \
             `log --follow` của `lay_lich_su_tep` (03-05, DIFF-05). Lệnh --name-status \
             cố ý KHÔNG có pathspec — xem tài liệu của `trang_thai_va_ten_cu`. \
             Số mốc khác 5 nghĩa là một lệnh mất mốc hoặc có lệnh mới chưa được kiểm"
        );

        // Kiểm **từng lệnh một**, không kiểm cả thân hàm như một khối.
        //
        // # Bản đầu của cổng này VÔ DỤNG — đã chạy mutation và nó xanh
        //
        // Bản đầu tìm vị trí `--` **đầu tiên** trong thân hàm rồi đòi mọi mốc pathspec
        // nằm sau nó. Xoá `.arg("--")` khỏi **lệnh diff thật** (mutation #8 của plan)
        // cho **0 test đỏ**: `--` của lệnh `--numstat` phía trên vẫn còn và nằm trước
        // mọi mốc, nên phép so vị trí vẫn đúng trong khi lệnh quan trọng nhất đã mất
        // dấu ngăn cách.
        //
        // Đây đúng là lỗi HIST-10 của Phase 2 lặp lại: một cổng đọc đúng tệp, đúng
        // hàm, đúng ý định, nhưng đo một đại lượng **toàn cục** cho một bất biến
        // **cục bộ**. Cách sửa là cắt thân hàm theo từng lệnh git rồi kiểm trong
        // phạm vi của chính lệnh đó.
        //
        // Cắt theo `GitCommand::new` — mỗi lần xuất hiện là một lệnh mới.
        let cac_lenh: Vec<&str> = ma.split("GitCommand::new").skip(1).collect();
        assert!(
            cac_lenh.len() >= 3,
            "phải có ít nhất ba lệnh git dựng trong `lay_diff_tep`, thấy {}",
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
                    "lệnh git thứ {} mang pathspec nhưng KHÔNG có `--` ngăn cách \
                     revision với nó (T-03-07). Thiếu `--`, một `path` trùng tên \
                     nhánh (`main`, `HEAD`) được git đọc thành REVISION, và `path` \
                     đến từ webview nên nó là chuỗi tuỳ ý.\n\nLệnh:\n{lenh}",
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
            so_lenh_co_pathspec, 4,
            "đúng bốn lệnh mang pathspec: `--numstat`, lệnh diff thật, lệnh \
             `--word-diff` của bước mức từ (03-03), và `log --follow` của \
             `lay_lich_su_tep` (03-05). Lệnh `--name-status` cố ý không mang (xem \
             `trang_thai_va_ten_cu`). Con số khác 4 nghĩa là phép cắt theo \
             `GitCommand::new` đã lệch và vòng lặp trên không còn kiểm đúng thứ nó tưởng"
        );
    }

    /// Chọn bản ghi `--name-status -z` khớp `path`, trên đúng byte git in ra.
    ///
    /// Định dạng đo thật (git 2.54): `R077\0renamed.txt\0renamed-new.txt\0` — `R`/`C`
    /// chiếm **hai** đường dẫn, mọi trạng thái khác chiếm một.
    #[test]
    fn chon_dung_ban_ghi_trong_name_status() {
        // Đổi tên: khớp theo đường dẫn MỚI, trả đường dẫn CŨ làm old_path.
        let (st, cu) = chon_ban_ghi_khop(b"R077\0renamed.txt\0renamed-new.txt\0", "renamed-new.txt");
        assert_eq!(st, "R077");
        assert_eq!(cu.as_deref(), Some("renamed.txt"));

        // Bản ghi đổi tên nằm GIỮA các bản ghi khác: đọc `R` như bản ghi một đường
        // dẫn sẽ làm mọi bản ghi sau nó lệch một nấc, và ta sẽ tìm nhầm.
        let buf = b"M\0a.txt\0R077\0cu.txt\0moi.txt\0A\0them.txt\0";
        assert_eq!(chon_ban_ghi_khop(buf, "a.txt"), ("M".into(), None));
        assert_eq!(
            chon_ban_ghi_khop(buf, "moi.txt"),
            ("R077".into(), Some("cu.txt".into()))
        );
        assert_eq!(
            chon_ban_ghi_khop(buf, "them.txt"),
            ("A".into(), None),
            "bản ghi SAU một bản ghi đổi tên phải đọc đúng — đây là lỗi lệch một nấc"
        );

        // Không khớp bản ghi nào, và đầu vào rỗng: không panic.
        assert_eq!(chon_ban_ghi_khop(buf, "khong-co.txt"), ("M".into(), None));
        assert_eq!(chon_ban_ghi_khop(b"", "bat-ky.txt"), ("M".into(), None));

        // Bản ghi cụt (git bị cắt giữa chừng): không panic, không lặp vô hạn.
        assert_eq!(chon_ban_ghi_khop(b"R077\0chi-co-mot\0", "x"), ("M".into(), None));

        // `C` (sao chép) cũng chiếm hai đường dẫn.
        assert_eq!(
            chon_ban_ghi_khop(b"C75\0nguon.txt\0dich.txt\0", "dich.txt"),
            ("C75".into(), Some("nguon.txt".into()))
        );
    }

    /// **Lệnh `--name-status` cố ý KHÔNG mang pathspec** — hồi quy của một lỗi đo được.
    ///
    /// Thêm `-- <path>` vào lệnh đó làm git báo `A` thay vì `R077` cho tệp đổi tên,
    /// vì pathspec lọc mất đường dẫn cũ **trước khi** phép phát hiện đổi tên chạy.
    /// Chi tiết và số đo trong tài liệu của `trang_thai_va_ten_cu`.
    #[test]
    fn lenh_name_status_khong_mang_pathspec() {
        let src = include_str!("diff.rs");
        let (_, than) = src
            .split_once("async fn trang_thai_va_ten_cu")
            .expect("phải có hàm trang_thai_va_ten_cu");
        let ma: String = than
            .lines()
            .take_while(|l| !l.starts_with("fn chon_ban_ghi_khop"))
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            ma.contains("--name-status"),
            "tiền đề: phép lọc phải giữ lại được thân hàm thật"
        );
        assert!(
            !ma.contains("PATHSPEC_SAU_DAU_GACH"),
            "lệnh --name-status KHÔNG được mang pathspec: nó làm git báo `A` thay vì \
             `R` cho tệp đổi tên, và kéo theo bản vá in TOÀN BỘ tệp là dòng thêm"
        );
    }

    /// **Thứ tự các bước là hợp đồng của DIFF-06.** Cổng kích thước phải nằm TRƯỚC
    /// mọi lệnh `diff`, và tra cache phải nằm trước tất cả.
    ///
    /// Test đọc thân hàm vì đây là thứ **không quan sát được từ giá trị trả về**: một
    /// cài đặt lọc-sau trả đúng `kind: tooLarge`. Test tích hợp đọc `CommandLog` là
    /// cổng chính; test này là cổng thứ hai, rẻ, chạy cả khi không có fixture.
    #[test]
    fn cong_kich_thuoc_nam_truoc_moi_lenh_diff() {
        let src = include_str!("diff.rs");
        let (_, than) = src
            .split_once("pub async fn lay_diff_tep")
            .expect("phải có hàm lay_diff_tep");
        let ma: String = than
            .lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        let tra_cache = ma.find("get_diff(&khoa)").expect("phải tra cache");
        let do_kich_thuoc = ma
            .find("kich_thuoc_hai_phia")
            .expect("phải đo kích thước hai phía");
        let lenh_diff_dau = ma.find(r#""diff""#).expect("phải có lệnh diff");

        // Neo vào **quyết định trả TooLarge**, không vào phép **đo** kích thước.
        //
        // Bản đầu của cổng này neo vào `kich_thuoc_hai_phia` và nó **xanh** dưới đúng
        // đột biến nó phải bắt (đã chạy thật, mutation #6 của plan): dời khối
        // `if ... > MAX_DIFF_BLOB_BYTES { return TooLarge }` xuống sau `git diff` để
        // nguyên lời gọi đo ở chỗ cũ, nên phép so vị trí vẫn đúng trong khi cài đặt
        // đã thành "lọc sau". Chỉ test tích hợp đọc `CommandLog` bắt được.
        //
        // Thứ quyết định "chặn trước hay lọc sau" là vị trí của `return`, nên cổng
        // phải neo vào đó. Đây là bài học "cổng grep dễ vô dụng" của Phase 2 ở dạng
        // tinh vi hơn: cổng neo đúng tệp, đúng hàm, đúng ý định — chỉ neo nhầm token.
        let quyet_dinh_too_large = ma
            .find("DiffKind::TooLarge")
            .expect("phải có nhánh trả TooLarge");

        assert!(
            tra_cache < do_kich_thuoc,
            "tra cache phải là việc ĐẦU TIÊN — hit thì không lệnh git nào được chạy"
        );
        assert!(
            do_kich_thuoc < quyet_dinh_too_large,
            "phải đo kích thước trước khi quyết định theo nó"
        );
        assert!(
            quyet_dinh_too_large < lenh_diff_dau,
            "nhánh trả `TooLarge` phải `return` TRƯỚC mọi lệnh `diff`. Chặn-trước và \
             lọc-sau trả về CÙNG một giá trị, nên chỉ vị trí của `return` này phân \
             biệt được chúng (T-03-10)"
        );
    }

    /// **Bước word-level phải nằm TRƯỚC khi mục được đặt vào cache** — 03-03.
    ///
    /// Đặt sau thì lần gọi **đầu** vẫn trả đúng `spans` và chỉ lần thứ hai trở đi mới
    /// mất — tức lỗi chỉ hiện ra khi người dùng bấm lại vào cùng một tệp, thao tác
    /// thường gặp nhất khi đọc một commit.
    ///
    /// Cổng neo vào **quyết định** (lời gọi `gan_khoang_muc_tu` và lời gọi
    /// `ket_thuc_voi_ten_cu` đặt mục vào cache), không vào phép **đo**. Bài học
    /// mutation #6 của 03-02: bản đầu của cổng kích thước neo vào lời gọi *đo kích
    /// thước* và **xanh** dưới đúng đột biến nó phải bắt, vì đột biến dời khối
    /// `return` mà để nguyên lời gọi đo.
    ///
    /// Test tích hợp `goi_lan_hai_dung_cache_va_giu_nguyen_spans` là cổng chính; cổng
    /// này là cổng thứ hai, rẻ, chạy cả khi không có fixture.
    #[test]
    fn buoc_muc_tu_nam_truoc_khi_dat_vao_cache() {
        let src = include_str!("diff.rs");
        let (_, than) = src
            .split_once("pub async fn lay_diff_tep")
            .expect("phải có hàm lay_diff_tep");
        // Cắt ở cuối hàm — `ket_thuc_voi_ten_cu` cũng xuất hiện ở định nghĩa hàm phía
        // dưới, và một lần xuất hiện ngoài thân hàm làm phép so vị trí vô nghĩa.
        let ma: String = than
            .lines()
            .take_while(|l| !l.starts_with("/// Chặn trên số dòng"))
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        // Tiền đề: phép lọc phải giữ lại được thân hàm thật. Không có khẳng định này
        // thì một phép cắt ăn mất mã làm cổng luôn xanh.
        assert!(
            ma.contains("parse_patch"),
            "tiền đề: phép cắt phải giữ lại thân hàm `lay_diff_tep`"
        );

        let phan_tich = ma.find("parse_patch(&ra_diff.stdout)").expect("phải phân tích bản vá");
        let muc_tu = ma
            .find("gan_khoang_muc_tu(")
            .expect("phải gọi bước diff mức từ");
        let vao_cache = ma
            .find("ket_thuc_voi_ten_cu(")
            .expect("phải đặt mục vào cache ở cuối");

        assert!(
            phan_tich < muc_tu,
            "phải phân tích bản vá TRƯỚC khi gắn khoảng — khoảng được khớp vào \
             `DiffLine` đã có"
        );
        assert!(
            muc_tu < vao_cache,
            "bước diff mức từ phải chạy TRƯỚC khi mục được đặt vào cache. Đặt sau thì \
             lần gọi ĐẦU vẫn đúng và chỉ lần thứ hai mất `spans` — một lỗi chỉ hiện \
             ra khi người dùng bấm lại vào cùng một tệp"
        );
    }

    /// **Bước word-level KHÔNG được chạm `no_newline_at_eof`.**
    ///
    /// Porcelain không cung cấp trường đó (đã đo: nó in `~` bình thường và không in
    /// `\ No newline at end of file`, khác `--unified` trên cùng tệp). Mọi giá trị suy
    /// từ đầu ra porcelain đều sai, và ghi đè sẽ xoá mất giá trị đúng mà `parse_patch`
    /// đã đặt.
    ///
    /// Đọc thân hàm vì đây là thứ dễ "tiện tay" thêm vào khi ai đó mở rộng `WordLine`.
    #[test]
    fn buoc_muc_tu_khong_ghi_de_no_newline_at_eof() {
        let src = include_str!("diff.rs");
        let (_, than) = src
            .split_once("async fn gan_khoang_muc_tu")
            .expect("phải có hàm gan_khoang_muc_tu");
        let ma: String = than
            .lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");

        assert!(
            ma.contains("dong.spans"),
            "tiền đề: hàm phải thật sự gán `spans`, nếu không khẳng định dưới vô nghĩa"
        );
        assert!(
            !ma.contains("no_newline_at_eof"),
            "bước diff mức từ KHÔNG được chạm `no_newline_at_eof`. Porcelain không \
             cung cấp trường đó (đã đo), nên mọi giá trị suy từ nó đều sai và sẽ xoá \
             mất giá trị đúng mà `parse_patch` đặt. Hệ quả: trình xem mất chỉ báo \
             \"không kết thúc bằng dòng mới\", và Phase 5 dựng lại bản vá thiếu dòng \
             `\\ No newline...` — bản vá như vậy `git apply` từ chối"
        );
    }

    /// `get_file_diff` phải có trong `generate_handler!` của `lib.rs`.
    ///
    /// **Không dùng `grep -c`** — lý do ghi trong 02-04-SUMMARY: cổng grep cũ xanh
    /// trong khi command chưa đăng ký, vì tên xuất hiện ở cả `use` lẫn chú thích.
    /// Ở đây test **phân tích khối** `generate_handler!`: cắt lấy đúng phần trong
    /// ngoặc, bỏ chú thích, rồi tách theo dấu phẩy và so khớp từng mục.
    #[test]
    fn get_file_diff_da_dang_ky_trong_generate_handler() {
        let src = include_str!("../lib.rs");

        let (_, sau) = src
            .split_once("tauri::generate_handler![")
            .expect("lib.rs phải có khối generate_handler!");
        let (khoi, _) = sau
            .split_once(']')
            .expect("khối generate_handler! phải được đóng");

        let muc: Vec<String> = khoi
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ")
            .split(',')
            .map(|s| s.trim().to_owned())
            .filter(|s| !s.is_empty())
            .collect();

        assert!(
            muc.iter().any(|m| m == "commands::get_file_diff"),
            "`commands::get_file_diff` phải là một MỤC trong generate_handler!, không \
             chỉ xuất hiện đâu đó trong tệp. Các mục hiện có: {muc:#?}"
        );
    }
}

#[cfg(test)]
mod tests_toan_tep {
    use super::*;

    /// Tệp thường → **toàn tệp**. Đây là mặc định người dùng yêu cầu (2026-09-22).
    #[test]
    fn tep_thuong_hien_toan_tep() {
        assert_eq!(so_dong_ngu_canh(0), UNIFIED_TOAN_TEP, "tệp rỗng");
        assert_eq!(so_dong_ngu_canh(10_000), UNIFIED_TOAN_TEP, "10 KB");
        assert_eq!(so_dong_ngu_canh(MAX_BYTE_TOAN_TEP), UNIFIED_TOAN_TEP, "đúng ngưỡng");
    }

    /// Vượt ngưỡng → rút gọn về 3. Ghim con số `3`, không chỉ ghim "khác toàn tệp":
    /// một cài đặt trả `0` cũng khác `UNIFIED_TOAN_TEP` nhưng `-U0` bỏ hết ngữ cảnh và
    /// làm phép khớp word-level theo số dòng trượt.
    #[test]
    fn tep_lon_lui_ve_ba_dong_ngu_canh() {
        assert_eq!(so_dong_ngu_canh(MAX_BYTE_TOAN_TEP + 1), 3, "vượt một byte");
        assert_eq!(so_dong_ngu_canh(5 * 1024 * 1024), 3, "5 MB");
    }

    /// Ngưỡng phải nằm **dưới** cổng kích thước, nếu không nó gần như không bao giờ
    /// chạy: mọi tệp qua được cổng 5 MB sẽ hiện toàn tệp, kể cả tệp 4,9 MB.
    #[test]
    fn nguong_toan_tep_thap_hon_cong_kich_thuoc() {
        assert!(
            MAX_BYTE_TOAN_TEP < MAX_DIFF_BLOB_BYTES,
            "MAX_BYTE_TOAN_TEP ({MAX_BYTE_TOAN_TEP}) phải < MAX_DIFF_BLOB_BYTES \
             ({MAX_DIFF_BLOB_BYTES}), nếu không cổng rút gọn vô dụng"
        );
    }

    /// `yarn.lock` 632 KB của repo công ty — tệp mà checkpoint #3 định đo nhưng phép đo
    /// bị bỏ qua — **phải** đi đường rút gọn. Ghim đúng con số đó vì nó là ca thật duy
    /// nhất ta biết kích thước, và là ca ta không có số liệu hiệu năng.
    #[test]
    fn tep_632kb_cua_repo_cong_ty_di_duong_an_toan() {
        assert_eq!(
            so_dong_ngu_canh(631_868),
            3,
            "yarn.lock 631 868 byte phải rút gọn: đây đúng ca checkpoint #3 không đo"
        );
    }
}
