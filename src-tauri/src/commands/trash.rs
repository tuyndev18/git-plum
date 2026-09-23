//! An toàn khi huỷ — WORK-06 và WORK-07.
//!
//! # Vì sao module này tồn tại
//!
//! WORK-07 tồn tại vì đúng **một** lý do: huỷ nhầm mất việc (rủi ro R3). Phase 5 là
//! phase đầu tiên ghi vào **nội dung tệp** của người dùng chứ không chỉ vào index, nên
//! một lần huỷ sai là mất dữ liệu **thật**, không phải một hiển thị sai vẽ lại được.
//!
//! # Ràng buộc trung tâm: không đường huỷ nào bỏ qua bước lưu
//!
//! Cài bằng **kiểu**, không bằng kỷ luật. [`huy_tep`] và [`huy_hunk`] đòi một
//! [`BienNhan`], và chỉ [`luu_truoc_khi_huy`] tạo được nó (trường private,
//! `pub(crate) fn moi`). Một đường huỷ mới viết trong tương lai mà quên lưu sẽ **không
//! biên dịch được**. Đó là mối đe doạ T-05-09, đóng bằng kiểu chứ không bằng một dòng
//! ghi chú "nhớ gọi hàm lưu trước nhé".
//!
//! # 🔴 "Lưu" nghĩa là object phải **TỚI ĐƯỢC**, không chỉ "đã tạo"
//!
//! `git stash create` tạo một commit **không có ref nào trỏ tới**. Một object không tới
//! được là một object cách `git gc` đúng một bước. Đo được trên git 2.54.0.windows.1:
//!
//! ```text
//! stash create, KHÔNG update-ref, rồi `git gc --prune=now --aggressive`:
//!   git cat-file -e <sha>  →  exit 1      ← object ĐÃ MẤT
//! stash create, CÓ update-ref, rồi cùng lệnh gc:
//!   git show <ref>:a.txt   →  nội dung cũ ← còn nguyên
//! ```
//!
//! Nên [`luu_truoc_khi_huy`] **luôn** `git update-ref`, và Test D của
//! `tests/trash_commands.rs` chạy `git gc --prune=now` thật để chứng minh điều đó. Một
//! cài đặt quên `update-ref` vẫn **xanh** ở mọi test khác (object còn trong kho cho tới
//! lần gc kế tiếp) và chỉ **đỏ ở Test D** — đó là cả lý do Test D tồn tại.
//!
//! # 🔴 `git stash create` im lặng bỏ qua tệp chưa theo dõi
//!
//! Đo được, và nó trái với giả định tự nhiên theo **hai** mức:
//!
//! ```text
//! chỉ có tệp chưa theo dõi bẩn:   git stash create → []   (chuỗi RỖNG, exit 0)
//! tệp đã theo dõi bẩn + tệp chưa theo dõi:
//!                                 git stash create → 356fea29...  (sha KHÁC rỗng)
//!                                 git ls-tree -r 356fea29 → chỉ có a.txt
//!                                 git show 356fea29:u.txt → fatal: không có trong cây
//! ```
//!
//! Mức thứ hai là mức nguy hiểm: một cài đặt chỉ kiểm "stdout có rỗng không" sẽ đi
//! nhánh stash ở ca **hỗn hợp**, thấy một sha hợp lệ, ghi ref, và **mất tệp chưa theo
//! dõi** — đúng dữ liệu mà cơ chế này tồn tại để giữ. Nên phép phân nhánh ở đây là
//! **theo từng đường dẫn** (`la_chua_theo_doi`), không theo việc lời gọi stash trả về gì.
//!
//! # 🔴 Với tệp chưa theo dõi, động từ là "Xoá"
//!
//! ROADMAP, không thương lượng. Ghim ở phía Rust qua [`crate::domain::dong_tu_cho`] để
//! giao diện không tự đoán — hai nguồn sự thật cho cùng một câu hỏi nghĩa là nhánh đoán
//! không có test nào (lỗi #9 của `CONTEXT.md` §4.1).

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::State;

use crate::domain::trash::{dong_tu_cho, BienNhan, MucThungRac, TIEN_TO_REF};
use crate::domain::{RepoStatus, StatusGroup};
use crate::error::{GitError, Result};
use crate::git::exec::DEFAULT_TIMEOUT;
use crate::git::patch_build::{dung_ban_va_mot_hunk, tach_hunk_tho};
use crate::git::GitCommand;
use crate::state::{AppState, RepoHandle};

use super::hunk::kiem_blob_hash;
use super::worktree::{chay_lenh_ghi_co_thu_lai_phan_loai, lay_trang_thai, repo_cua};

/// Mốc đánh dấu vị trí một pathspec trong argv — cùng khuôn (và cùng tên) với mốc của
/// [`super::hunk`], [`super::diff`] và [`super::worktree`].
///
/// Hàm đồng nhất; nó tồn tại để cổng
/// [`tests::moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path`] khẳng định được
/// *thứ tự* của `--` và pathspec bằng cách tìm một chuỗi ổn định trong mã nguồn. Khai
/// lại ở đây chứ không dùng chung: cổng của mỗi tệp đọc `include_str!` của **chính tệp
/// đó**, nên cổng của các tệp kia không phủ được lệnh nằm ở đây.
#[allow(non_snake_case)]
fn PATHSPEC_SAU_DAU_GACH(path: &str) -> &str {
    path
}

/// Số mục tối đa trả về cho danh sách "Vừa huỷ gần đây".
///
/// Đây là một **danh sách gần đây**, không phải một kho lưu trữ: người dùng tìm thứ họ
/// vừa huỷ nhầm vài giây trước, không lục lại tháng trước. Ref cũ **không** bị xoá —
/// chúng vẫn tới được, chỉ không hiện trong danh sách (T-05-10: tích luỹ được **chấp
/// nhận**, dọn định kỳ để v2).
pub const GIOI_HAN_DANH_SACH: usize = 50;

/// Giây Unix hiện tại.
fn giay_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Mili-giây Unix — dùng cho tên ref.
///
/// 🔴 **Mili-giây, không phải giây.** Hai lần huỷ trong cùng một giây là chuyện bình
/// thường khi người dùng bấm nhanh, và trùng tên ref nghĩa là `update-ref` **ghi đè**
/// bản lưu của lần trước — mất đúng dữ liệu mà cơ chế này tồn tại để giữ. Hậu tố `-<n>`
/// đóng nốt phần còn lại (nhiều tệp chưa theo dõi trong **một** lần huỷ).
fn mili_giay_unix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

/// Nhóm trạng thái của từng đường dẫn, đọc từ `lay_trang_thai`.
///
/// # Vì sao đọc từ status chứ không đoán từ "tệp có trong index không"
///
/// Plan nói rõ: phân nhánh theo [`StatusGroup`], **không** đoán. Một tệp có thể vừa
/// `Staged` vừa `Unstaged` (XY = `MM` sinh **hai** `StatusEntry`), và phép đoán "không
/// có trong index ⇒ chưa theo dõi" sai với tệp vừa bị `git rm --cached`.
///
/// Trả `true` khi đường dẫn **chỉ** xuất hiện ở nhóm `Untracked` — tức git thật sự
/// chưa biết gì về nó.
async fn la_chua_theo_doi(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    path: &str,
) -> Result<bool> {
    let tt = lay_trang_thai(state, Arc::clone(repo)).await?;

    let mut thay = false;
    for e in &tt.entries {
        if e.path == path {
            if e.group != StatusGroup::Untracked {
                // Xuất hiện ở một nhóm đã-theo-dõi ⇒ git biết tệp này.
                return Ok(false);
            }
            thay = true;
        }
    }

    Ok(thay)
}

/// Lưu nội dung sắp bị huỷ, và trả **bằng chứng** đã lưu — WORK-07.
///
/// # Đường đi
///
/// 1. Tách `paths` thành nhóm **đã theo dõi** và nhóm **chưa theo dõi**.
/// 2. Nhóm đã theo dõi → `git stash create` → một commit giữ **cả** bản index lẫn bản
///    thư mục làm việc.
/// 3. Nhóm chưa theo dõi → `git hash-object -w` cho **từng** tệp → một blob mỗi tệp.
///    `stash create` **không bao giờ** giữ nhóm này (đo được — xem đầu tệp).
/// 4. `git update-ref <TIEN_TO_REF><epoch_ms>-<n> <oid>` cho từng object.
/// 5. 🔴 Đọc lại bằng `git rev-parse --verify` **trước** khi trả biên nhận.
///
/// # 🔴 Vì sao bước 5 không bỏ được
///
/// Biên nhận là một lời khẳng định "nội dung đã an toàn", và mọi đường huỷ tin vào lời
/// khẳng định đó để cho phép ghi đè tệp người dùng. Nếu `update-ref` thất bại mà ta vẫn
/// trả `BienNhan` thì **kiểu đang nói dối** và toàn bộ ràng buộc mất giá trị: người
/// dùng mất tệp, danh sách khôi phục rỗng, và không có thông báo nào. Thất bại ở đây
/// phải là `Err` — **không huỷ gì cả**.
///
/// # Ca không có gì để lưu
///
/// Tệp sạch (không bẩn, đã theo dõi) cho stash rỗng và không có blob nào. Trả `Err`
/// chứ không phải một biên nhận rỗng: một biên nhận không giữ gì vẫn mở đường cho lệnh
/// huỷ chạy, tức đúng thứ ràng buộc kiểu này ngăn.
pub async fn luu_truoc_khi_huy(
    state: &AppState,
    repo: Arc<RepoHandle>,
    paths: &[String],
) -> Result<BienNhan> {
    if paths.is_empty() {
        return Err(GitError::ParseFailed(
            "không có đường dẫn nào để lưu trước khi huỷ".to_owned(),
        ));
    }

    let runner = state.runner(Arc::clone(&repo));

    // --- Phân nhóm THEO TỪNG ĐƯỜNG DẪN ------------------------------------
    //
    // 🔴 Không phân nhánh theo "stash create có trả rỗng không". Ca hỗn hợp (một tệp đã
    // theo dõi bẩn + một tệp chưa theo dõi) cho một sha KHÁC rỗng mà cây của nó **không
    // chứa** tệp chưa theo dõi — đo được. Tin vào sha đó là mất tệp chưa theo dõi trong
    // im lặng.
    let mut da_theo_doi: Vec<String> = Vec::new();
    let mut chua_theo_doi: Vec<String> = Vec::new();
    for p in paths {
        if la_chua_theo_doi(state, &repo, p).await? {
            chua_theo_doi.push(p.clone());
        } else {
            da_theo_doi.push(p.clone());
        }
    }

    let luc = giay_unix();
    let moc = mili_giay_unix();
    let mut muc: Vec<MucThungRac> = Vec::new();

    // --- Nhóm ĐÃ THEO DÕI: một stash commit chung --------------------------
    if !da_theo_doi.is_empty() {
        let ra = runner
            .read_ok(
                GitCommand::new(&repo.path)
                    .args(["stash", "create", "git-plum trash"])
                    .timeout(DEFAULT_TIMEOUT),
            )
            .await?;

        // 🔴 CHỈ `stdout`. `git stash create` in `warning: in the working copy of
        // 'a.txt', LF will be replaced by CRLF` ra **stderr**, và đo được hôm nay là
        // với `$(...)` trong shell nó rơi vào cùng luồng với sha. Ghép hai luồng ở đây
        // nghĩa là phân tích một sha ra khỏi một dòng cảnh báo — sớm hay muộn cũng
        // phân tích nhầm. `GitOutput` tách sẵn; việc duy nhất phải làm là không ghép.
        let sha = String::from_utf8_lossy(&ra.stdout).trim().to_owned();

        if !sha.is_empty() {
            let ten = format!("{TIEN_TO_REF}{moc}-{}", muc.len());
            ghi_ref(state, &repo, &ten, &sha).await?;
            muc.push(MucThungRac {
                ref_name: ten,
                object_id: sha,
                la_blob: false,
                paths: da_theo_doi.clone(),
                luc,
                nhan: nhan_cho(&da_theo_doi, false),
            });
        }
    }

    // --- Nhóm CHƯA THEO DÕI: một blob cho MỖI tệp -------------------------
    for p in &chua_theo_doi {
        let ra = runner
            .read_ok(
                GitCommand::new(&repo.path)
                    .args(["hash-object", "-w"])
                    // 🔴 `--` trước pathspec. `path` đến từ webview, tức chuỗi tuỳ ý:
                    // một tệp tên `--stdin` làm git chờ stdin vô hạn cho tới khi quá
                    // hạn giờ.
                    .arg("--")
                    .arg(PATHSPEC_SAU_DAU_GACH(p))
                    .timeout(DEFAULT_TIMEOUT),
            )
            .await?;

        let oid = String::from_utf8_lossy(&ra.stdout).trim().to_owned();
        if oid.is_empty() {
            return Err(GitError::ParseFailed(format!(
                "`git hash-object -w` không trả object id cho `{p}` — không lưu được \
                 nội dung, nên KHÔNG huỷ"
            )));
        }

        let ten = format!("{TIEN_TO_REF}{moc}-{}", muc.len());
        ghi_ref(state, &repo, &ten, &oid).await?;
        muc.push(MucThungRac {
            ref_name: ten,
            object_id: oid,
            la_blob: true,
            paths: vec![p.clone()],
            luc,
            nhan: nhan_cho(std::slice::from_ref(p), true),
        });
    }

    // --- Không lưu được gì ⇒ KHÔNG cấp biên nhận --------------------------
    let Some(dau) = muc.into_iter().next() else {
        return Err(GitError::ParseFailed(format!(
            "không có nội dung nào để lưu cho {paths:?} — `git stash create` trả rỗng \
             và không có tệp chưa theo dõi nào. Huỷ một tệp sạch là thao tác rỗng, và \
             cấp biên nhận ở đây sẽ mở đường cho một lần huỷ không có bản lưu"
        )));
    };

    Ok(BienNhan::moi(dau))
}

/// Ghi một ref và **đọc lại** để chắc chắn nó tồn tại.
///
/// Xem ghi chú "vì sao bước 5 không bỏ được" ở [`luu_truoc_khi_huy`].
async fn ghi_ref(
    state: &AppState,
    repo: &Arc<RepoHandle>,
    ten_ref: &str,
    oid: &str,
) -> Result<()> {
    let repo_path = repo.path.clone();
    let ten = ten_ref.to_owned();
    let id = oid.to_owned();

    // Đi qua đường thử lại của 04-02 — DÙNG LẠI, không viết mới. Nó có 3 lần thử, có
    // giãn cách, và **không bao giờ xoá `.git/index.lock`**.
    let ten_loi = ten.clone();
    chay_lenh_ghi_co_thu_lai_phan_loai(
        state,
        repo,
        || {
            GitCommand::new(&repo_path)
                .args(["update-ref", &ten, &id])
                .timeout(DEFAULT_TIMEOUT)
        },
        move |args, out| GitError::CommandFailed {
            args,
            status: out.status,
            stderr: format!(
                "không ghi được ref thùng rác `{ten_loi}`: {}",
                out.stderr_lossy()
            ),
        },
    )
    .await?;

    // 🔴 Đọc lại. `update-ref` thoát 0 mà ref không tồn tại là một mâu thuẫn ta muốn
    // biết **trước** khi huỷ, không phải sau.
    let runner = state.runner(Arc::clone(repo));
    let ra = runner
        .read(
            GitCommand::new(&repo.path)
                .args(["rev-parse", "--verify", ten_ref])
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    if !ra.is_success() || String::from_utf8_lossy(&ra.stdout).trim().is_empty() {
        return Err(GitError::CommandFailed {
            args: vec!["rev-parse".to_owned(), "--verify".to_owned(), ten_ref.to_owned()],
            status: ra.status,
            stderr: format!(
                "ref thùng rác `{ten_ref}` KHÔNG đọc lại được sau `update-ref`. Cấp \
                 biên nhận ở đây nghĩa là kiểu đang nói dối: đường huỷ sẽ tin là nội \
                 dung đã an toàn và ghi đè tệp người dùng. Không huỷ. git nói: {}",
                ra.stderr_lossy()
            ),
        });
    }

    Ok(())
}

/// Nhãn hiện cho người dùng, dùng **đúng** động từ của nhóm.
fn nhan_cho(paths: &[String], chua_theo_doi: bool) -> String {
    let dt = if chua_theo_doi {
        dong_tu_cho(StatusGroup::Untracked)
    } else {
        dong_tu_cho(StatusGroup::Unstaged)
    };

    match paths {
        [mot] => format!("{} {mot}", dt.nhan()),
        nhieu => format!("{} {} tệp", dt.nhan(), nhieu.len()),
    }
}

/// Huỷ thay đổi theo **tệp** — WORK-06.
///
/// Đòi một [`BienNhan`]: không gọi được nếu chưa lưu. Xem đầu tệp.
///
/// # Phân nhánh
///
/// - Tệp **đã theo dõi** → `git checkout HEAD -- <path>`.
/// - Tệp **chưa theo dõi** → xoá tệp trên đĩa; git không biết nó nên không lệnh git nào
///   gỡ được.
///
/// # 🔴 Vì sao `checkout HEAD --` chứ không `checkout --`
///
/// Đo được trên git 2.54.0.windows.1, và nó trái với giả định tự nhiên:
///
/// ```text
/// tệp có thay đổi ĐÃ STAGE (`git add`), rồi `git checkout -- a.txt`:
///   status sau đó: [M  a.txt]        ← vẫn còn trong index
///   nội dung tệp:  one/STAGED/three  ← KHÔNG huỷ gì cả
/// cùng ca, `git checkout HEAD -- a.txt`:
///   status sau đó: []                ← sạch
/// ```
///
/// `git checkout -- <path>` khôi phục từ **index**, không từ HEAD. Với một thay đổi đã
/// stage thì index *chính là* thứ cần bỏ, nên lệnh đó là thao tác rỗng và người dùng
/// bấm "Huỷ bỏ" mà không có gì bị huỷ. `HEAD` làm cả hai tầng về gốc.
pub async fn huy_tep(
    state: &AppState,
    repo: Arc<RepoHandle>,
    paths: &[String],
    _bien_nhan: BienNhan,
) -> Result<RepoStatus> {
    for p in paths {
        if la_chua_theo_doi(state, &repo, p).await? {
            // Tệp chưa theo dõi: git không quản, nên xoá trên đĩa. Nội dung đã nằm
            // trong một blob có ref trỏ tới (bước lưu) nên khôi phục được.
            let day_du = repo.path.join(p);
            if let Err(e) = tokio::fs::remove_file(&day_du).await {
                if e.kind() != std::io::ErrorKind::NotFound {
                    return Err(GitError::Io(format!("không xoá được `{p}`: {e}")));
                }
            }
        } else {
            let repo_path = repo.path.clone();
            let duong_dan = p.clone();
            let loi_path = p.clone();
            chay_lenh_ghi_co_thu_lai_phan_loai(
                state,
                &repo,
                || {
                    GitCommand::new(&repo_path)
                        .args(["checkout", "HEAD"])
                        // 🔴 `--` trước pathspec: không có nó, một tệp tên `main` làm
                        // `git checkout HEAD main` đổi **nhánh** thay vì khôi phục tệp.
                        .arg("--")
                        .arg(PATHSPEC_SAU_DAU_GACH(&duong_dan))
                        .timeout(DEFAULT_TIMEOUT)
                },
                move |args, out| GitError::CommandFailed {
                    args,
                    status: out.status,
                    stderr: format!(
                        "không huỷ được thay đổi của `{loi_path}`: {}",
                        out.stderr_lossy()
                    ),
                },
            )
            .await?;
        }
    }

    lay_trang_thai(state, repo).await
}

/// Huỷ **một khối** thay đổi trong thư mục làm việc — WORK-06.
///
/// # Khác `unstage_hunk` ở chỗ nào
///
/// `unstage_hunk` (05-02) áp `--reverse` **`--cached`**, tức chỉ gỡ khối khỏi index;
/// nội dung vẫn còn trong thư mục làm việc. Ở đây `--reverse` áp **trên worktree**
/// (không `--cached`), tức xoá hẳn thay đổi đó khỏi tệp — đó là lý do đường này cần một
/// bản lưu còn đường kia thì không.
///
/// Kiểm `blob_hash` trước, cùng hợp đồng WORK-05 của 05-02: áp một bản vá dựng từ nội
/// dung **cũ** lên một tệp **mới** là đúng ca R1.
pub async fn huy_hunk(
    state: &AppState,
    repo: Arc<RepoHandle>,
    path: &str,
    hunk_index: usize,
    blob_hash: &str,
    _bien_nhan: BienNhan,
) -> Result<RepoStatus> {
    // --- Tệp có còn đúng thứ giao diện đã vẽ không (WORK-05) ---------------
    kiem_blob_hash(state, &repo, path, blob_hash).await?;

    let runner = state.runner(Arc::clone(&repo));

    // --- Byte thô của bản vá đầy đủ ----------------------------------------
    let ra_diff = runner
        .read_ok(
            GitCommand::new(&repo.path)
                .args(["-c", "core.quotepath=false", "diff", "--unified=3", "--no-color"])
                .arg("--")
                .arg(PATHSPEC_SAU_DAU_GACH(path))
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    // 🔴 `ra_diff.stdout` — CHỈ stdout, không bao giờ ghép stderr vào bản vá.
    let tach = tach_hunk_tho(&ra_diff.stdout)?;

    let Some(ban_va) = dung_ban_va_mot_hunk(&tach, hunk_index) else {
        return Err(GitError::ParseFailed(format!(
            "không dựng được bản vá cho khối {hunk_index}: bản vá của `{path}` có {} \
             khối (tệp nhị phân cho 0 khối)",
            tach.so_hunk()
        )));
    };

    // --- Thử khan trước --------------------------------------------------
    //
    // `--check` không đổi gì. Nó giữ cho ca thất bại để lại thư mục làm việc **sạch** —
    // quan trọng hơn ở đây so với `stage_hunk`, vì đường này ghi vào tệp thật.
    let ra_check = runner
        .read(
            GitCommand::new(&repo.path)
                .args(CO_HUY_HUNK)
                .arg("--check")
                .stdin_bytes(ban_va.clone())
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    if !ra_check.is_success() {
        // 🔴 KHÔNG thử lại lỏng hơn. Khớp mờ khi tệp đã đổi là áp vào chỗ **sai**, và
        // đây là đường ghi thẳng vào nội dung tệp người dùng.
        return Err(GitError::FileChanged {
            path: path.to_owned(),
        });
    }

    // --- Áp thật -----------------------------------------------------------
    let repo_path = repo.path.clone();
    let duong_dan = path.to_owned();
    chay_lenh_ghi_co_thu_lai_phan_loai(
        state,
        &repo,
        || {
            GitCommand::new(&repo_path)
                .args(CO_HUY_HUNK)
                .stdin_bytes(ban_va.clone())
                .timeout(DEFAULT_TIMEOUT)
        },
        move |_args, _out| GitError::FileChanged {
            path: duong_dan.clone(),
        },
    )
    .await?;

    lay_trang_thai(state, repo).await
}

/// Tập cờ cho **mọi** lời gọi `git apply` của đường huỷ khối.
///
/// # Khác [`super::hunk::CO_APPLY`] đúng một cờ, và khác biệt đó là cả điểm
///
/// Không `--cached`: huỷ một khối phải xoá thay đổi khỏi **thư mục làm việc**, không
/// phải khỏi index. Có `--reverse`: ta **gỡ** khối ra, không thêm vào.
///
/// # 🔴 `--recount` bắt buộc
///
/// Khi chọn một tập con các khối, số dòng trong header `@@` không còn đúng với thân đã
/// tỉa; `--recount` bảo git tính lại. Đo được: bản vá thân-bị-cắt-header-cũ cho
/// `exit 128 corrupt patch` khi thiếu `--recount` và `exit 0` khi có.
///
/// # 🔴 Không bao giờ có cờ khớp mờ
///
/// Không `--whitespace=fix`, không `--3way`, không `--reject`, không `--unidiff-zero`.
/// Có cổng ghim: [`tests::duong_huy_khong_bao_gio_khop_mo`].
pub const CO_HUY_HUNK: &[&str] = &["apply", "--recount", "--reverse"];

/// Danh sách "Vừa huỷ gần đây", **mới nhất trước** — WORK-07.
pub async fn danh_sach_thung_rac(
    state: &AppState,
    repo: Arc<RepoHandle>,
) -> Result<Vec<MucThungRac>> {
    let runner = state.runner(Arc::clone(&repo));

    // Tên ref là `<epoch_ms>-<n>` với `epoch_ms` rộng cố định 13 chữ số cho tới năm
    // 2286, nên sắp theo chuỗi giảm dần **là** sắp theo thời gian giảm dần. Đo được:
    // `--sort=-refname` cho đúng thứ tự trên các tên 13 chữ số.
    let ra = runner
        .read_ok(
            GitCommand::new(&repo.path)
                .args([
                    "for-each-ref",
                    "--format=%(refname)%09%(objectname)%09%(objecttype)",
                    "--sort=-refname",
                    TIEN_TO_REF,
                ])
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;

    let mut ra_ve: Vec<MucThungRac> = Vec::new();
    for dong in String::from_utf8_lossy(&ra.stdout).lines() {
        // 🔴 Cắt `\r` — trên Windows git có thể trả CRLF, và một SHA mang `\r` ở cuối
        // là một SHA không tra cứu được, im lặng (CLAUDE.md, bẫy Windows).
        let dong = dong.trim_end_matches('\r');
        if dong.is_empty() {
            continue;
        }

        let mut cot = dong.split('\t');
        let (Some(ten), Some(oid), Some(loai)) = (cot.next(), cot.next(), cot.next()) else {
            continue;
        };

        ra_ve.push(MucThungRac {
            ref_name: ten.to_owned(),
            object_id: oid.to_owned(),
            la_blob: loai == "blob",
            paths: Vec::new(),
            luc: 0,
            nhan: String::new(),
        });

        if ra_ve.len() >= GIOI_HAN_DANH_SACH {
            break;
        }
    }

    Ok(ra_ve)
}

/// Khôi phục nội dung từ một mục thùng rác — WORK-07.
///
/// # 🔴 `ref_name` đến từ webview, nên nó được kiểm tiền tố
///
/// Mối đe doạ T-05-12. Không có phép kiểm này thì một `ref_name` bằng `refs/heads/main`
/// biến lời gọi thành `git checkout refs/heads/main -- .`, tức ghi đè thư mục làm việc
/// bằng nội dung của một nhánh. Danh sách chỉ nhận ref **trong** không gian riêng.
///
/// # Hai đường khôi phục
///
/// - **blob** (tệp chưa theo dõi) → `git cat-file -p` rồi ghi **byte thô** ra tệp.
///   🔴 `Vec<u8>`, không qua `String`: đây là nội dung tệp người dùng, và
///   `String::from_utf8_lossy` đổi mỗi byte không hợp lệ thành U+FFFD — **ba** byte —
///   rồi ghi ba byte đó vào tệp thật (WORK-04).
/// - **commit** (stash) → `git checkout <ref> -- <paths>`.
pub async fn khoi_phuc(
    state: &AppState,
    repo: Arc<RepoHandle>,
    ref_name: &str,
    paths: &[String],
) -> Result<RepoStatus> {
    if !ref_name.starts_with(TIEN_TO_REF) {
        return Err(GitError::ParseFailed(format!(
            "ref `{ref_name}` không nằm trong không gian thùng rác `{TIEN_TO_REF}`. \
             Chỉ nhận ref có tiền tố đó: một `refs/heads/main` lọt qua đây sẽ thành \
             `git checkout refs/heads/main -- .`, tức ghi đè thư mục làm việc bằng nội \
             dung của một nhánh (T-05-12)"
        )));
    }

    let runner = state.runner(Arc::clone(&repo));

    // Loại object quyết định đường khôi phục.
    let ra_loai = runner
        .read_ok(
            GitCommand::new(&repo.path)
                .args(["cat-file", "-t", ref_name])
                .timeout(DEFAULT_TIMEOUT),
        )
        .await?;
    let la_blob = String::from_utf8_lossy(&ra_loai.stdout).trim() == "blob";

    if la_blob {
        let [duong_dan] = paths else {
            return Err(GitError::ParseFailed(format!(
                "mục blob giữ nội dung của ĐÚNG một tệp, nhưng nhận {} đường dẫn",
                paths.len()
            )));
        };

        let ra = runner
            .read_ok(
                GitCommand::new(&repo.path)
                    .args(["cat-file", "-p", ref_name])
                    .timeout(DEFAULT_TIMEOUT),
            )
            .await?;

        // 🔴 `ra.stdout` là `Vec<u8>` và đi thẳng ra đĩa. Không `String` ở bất kỳ điểm
        // nào trên đường này.
        let day_du = repo.path.join(duong_dan);
        if let Some(cha) = day_du.parent() {
            tokio::fs::create_dir_all(cha)
                .await
                .map_err(|e| GitError::Io(format!("không tạo được thư mục cha: {e}")))?;
        }
        tokio::fs::write(&day_du, &ra.stdout)
            .await
            .map_err(|e| GitError::Io(format!("không ghi được `{duong_dan}`: {e}")))?;
    } else {
        let repo_path = repo.path.clone();
        let ten = ref_name.to_owned();
        let ds = paths.to_vec();
        chay_lenh_ghi_co_thu_lai_phan_loai(
            state,
            &repo,
            || {
                let mut cmd = GitCommand::new(&repo_path)
                    .args(["checkout", &ten])
                    .arg("--");
                for p in &ds {
                    cmd = cmd.arg(PATHSPEC_SAU_DAU_GACH(p));
                }
                cmd.timeout(DEFAULT_TIMEOUT)
            },
            |args, out| GitError::CommandFailed {
                args,
                status: out.status,
                stderr: format!("không khôi phục được: {}", out.stderr_lossy()),
            },
        )
        .await?;

        // --- 🔴 Trả INDEX về HEAD ------------------------------------------
        //
        // `git checkout <ref> -- <path>` ghi vào **cả** thư mục làm việc **lẫn index**.
        // Đo được trên git 2.54.0.windows.1:
        //
        // ```text
        // sau `git checkout <ref> -- a.txt`:
        //   git status --porcelain  →  "M  a.txt"   ← đã STAGE
        //   git diff -- a.txt       →  0 hunk       ← worktree KHỚP index
        //   git diff HEAD -- a.txt  →  1 hunk
        // ```
        //
        // Nội dung **đúng**, nhưng người dùng nhận nó ở trạng thái **đã stage** — khác
        // trạng thái họ ở trước khi bấm huỷ. Khôi phục phải trả lại đúng thứ đã mất,
        // không kèm một thao tác stage mà người dùng không yêu cầu: lần commit kế tiếp
        // sẽ lặng lẽ mang theo nội dung này.
        //
        // `git reset HEAD -- <paths>` gỡ index về HEAD và **không đụng** thư mục làm
        // việc (đo được: nội dung tệp không đổi sau lệnh).
        let repo_path_reset = repo.path.clone();
        let ds_reset = paths.to_vec();
        chay_lenh_ghi_co_thu_lai_phan_loai(
            state,
            &repo,
            || {
                let mut cmd = GitCommand::new(&repo_path_reset)
                    .args(["reset", "--quiet", "HEAD"])
                    .arg("--");
                for p in &ds_reset {
                    cmd = cmd.arg(PATHSPEC_SAU_DAU_GACH(p));
                }
                cmd.timeout(DEFAULT_TIMEOUT)
            },
            |args, out| GitError::CommandFailed {
                args,
                status: out.status,
                stderr: format!(
                    "khôi phục xong nhưng không gỡ được index về HEAD: {}",
                    out.stderr_lossy()
                ),
            },
        )
        .await?;
    }

    lay_trang_thai(state, repo).await
}

// ---------------------------------------------------------------------------
// Tauri command — điểm vào DUY NHẤT của giao diện
// ---------------------------------------------------------------------------
//
// 🔴 Mỗi command tự gọi `luu_truoc_khi_huy` rồi chuyển `BienNhan` xuống. Giao diện
// **không bao giờ** gọi được một đường huỷ mà bỏ qua bước lưu, vì nó không có cách nào
// dựng ra một `BienNhan`.

/// Huỷ thay đổi theo tệp — WORK-06. Lưu trước, luôn luôn.
#[tauri::command]
pub async fn discard_files(
    repo_id: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths).await?;
    huy_tep(&state, repo, &paths, bien_nhan).await
}

/// Huỷ một khối thay đổi — WORK-06. Lưu trước, luôn luôn.
#[tauri::command]
pub async fn discard_hunk(
    repo_id: String,
    path: String,
    hunk_index: usize,
    blob_hash: String,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    let paths = vec![path.clone()];
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths).await?;
    huy_hunk(&state, repo, &path, hunk_index, &blob_hash, bien_nhan).await
}

/// Danh sách "Vừa huỷ gần đây" — WORK-07.
#[tauri::command]
pub async fn list_trash(
    repo_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<MucThungRac>> {
    let repo = repo_cua(&state, &repo_id)?;
    danh_sach_thung_rac(&state, repo).await
}

/// Khôi phục từ danh sách — WORK-07.
#[tauri::command]
pub async fn restore_trash(
    repo_id: String,
    ref_name: String,
    paths: Vec<String>,
    state: State<'_, AppState>,
) -> Result<RepoStatus> {
    let repo = repo_cua(&state, &repo_id)?;
    khoi_phuc(&state, repo, &ref_name, &paths).await
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thân **không-test** của tệp này, đã bỏ dòng chú thích.
    ///
    /// Cùng khuôn và cùng lý do với `commands/hunk.rs`: lỗi #1 và #5 của
    /// `CONTEXT.md` §4.1 là cổng grep khớp **chú thích** thay vì mã. Tệp này nguy hiểm
    /// đúng kiểu đó — doc comment liệt kê nguyên văn tên **từng cờ bị cấm**
    /// (`--whitespace=fix`, `--3way`, ...) để giải thích vì sao không dùng, và liệt kê
    /// cả `git checkout -- ` để giải thích vì sao dùng `checkout HEAD --`.
    fn than_khong_chu_thich() -> String {
        let src = include_str!("trash.rs");
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

    /// Phép lọc **tự nó** hoạt động — không có test này thì hai hướng hỏng đều tạo cổng
    /// hỏng và không hướng nào gây lỗi biên dịch.
    #[test]
    fn phep_loc_chu_thich_hoat_dong() {
        let than = than_khong_chu_thich();

        assert!(
            !than.contains("mod tests"),
            "🔴 phép cắt `mod tests` hỏng — cổng đọc cả mã test, tức khớp chính chuỗi \
             khẳng định của chính nó"
        );
        assert!(
            !than.contains("ROADMAP, không thương lượng"),
            "🔴 một dòng `//!` sống sót qua phép lọc — cổng đo văn xuôi, không đo mã"
        );
        assert!(
            than.contains("pub const CO_HUY_HUNK"),
            "🔴 phép lọc quá tay: thân mất cả khai báo `CO_HUY_HUNK`. Thân rỗng làm \
             MỌI cổng dưới đây XANH với mọi mã"
        );
    }

    /// **Đường huỷ không bao giờ mang cờ khớp mờ.**
    ///
    /// Đường này ghi thẳng vào **nội dung tệp** người dùng, nên khớp mờ ở đây tệ hơn ở
    /// `stage_hunk`: ở đó áp sai chỗ làm bẩn index (sửa được bằng `git reset`), ở đây
    /// nó **xoá mất** một đoạn nội dung ở vị trí người dùng không chọn.
    #[test]
    fn duong_huy_khong_bao_gio_khop_mo() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("CO_HUY_HUNK"),
            "🔴 tiền đề sai: không thấy `CO_HUY_HUNK` trong thân đã lọc — cổng đang \
             tìm trong chuỗi rỗng và sẽ XANH với MỌI mã, kể cả mã có `--3way`"
        );

        for cam in ["--whitespace=fix", "--unidiff-zero", "--3way", "--reject", "-C1"] {
            assert!(
                !than.contains(cam),
                "🔴 `{cam}` là khớp mờ — ROADMAP xếp 'không thương lượng'. Khi tệp đã \
                 đổi, 'gần giống' là chỗ SAI, và đây là đường ghi vào nội dung tệp thật"
            );
        }
    }

    /// `CO_HUY_HUNK` mang **đúng** tập cờ: có `--reverse`, có `--recount`, KHÔNG `--cached`.
    ///
    /// Ba khẳng định, ba đột biến khác nhau:
    /// - bỏ `--reverse` → áp **thêm** khối thay vì gỡ (M16);
    /// - bỏ `--recount` → `corrupt patch` trên bản vá thân-bị-tỉa;
    /// - thêm `--cached` → huỷ khỏi index mà **không** đụng tệp, tức người dùng bấm
    ///   "Huỷ bỏ" và thấy nội dung vẫn nguyên.
    #[test]
    fn co_huy_hunk_mang_dung_tap_co() {
        assert_eq!(
            CO_HUY_HUNK[0], "apply",
            "🔴 `apply` phải đứng đầu argv — git đọc subcommand ở vị trí đầu"
        );
        assert!(
            CO_HUY_HUNK.contains(&"--reverse"),
            "🔴 thiếu `--reverse`: bản vá sẽ được áp THÊM vào thay vì gỡ ra, tức bấm \
             'Huỷ bỏ' làm thay đổi nhân đôi"
        );
        assert!(
            CO_HUY_HUNK.contains(&"--recount"),
            "🔴 thiếu `--recount`: bản vá một khối có thân đã tỉa, và header `@@` cũ \
             không còn khớp — git trả `corrupt patch`"
        );
        assert!(
            !CO_HUY_HUNK.contains(&"--cached"),
            "🔴 `--cached` làm lệnh chỉ đụng index. Huỷ một khối phải xoá thay đổi \
             khỏi THƯ MỤC LÀM VIỆC — đó là khác biệt duy nhất với `unstage_hunk`"
        );
    }

    /// Mọi lệnh có pathspec đều có `--` **trước** đường dẫn.
    ///
    /// Kiểm **từng** lệnh, không kiểm toàn cục: một cổng đếm "số `--` >= số pathspec"
    /// xanh kể cả khi một lệnh có hai `--` và một lệnh không có cái nào.
    #[test]
    fn moi_lenh_co_pathspec_deu_co_dau_gach_ngang_truoc_path() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("PATHSPEC_SAU_DAU_GACH"),
            "🔴 tiền đề sai: không thấy mốc `PATHSPEC_SAU_DAU_GACH` — cổng tìm trong \
             chuỗi rỗng và XANH với mọi mã"
        );

        let so_moc = than.matches("PATHSPEC_SAU_DAU_GACH(").count();
        // Một lần khai báo hàm + mỗi chỗ gọi.
        assert!(
            so_moc >= 5,
            "🔴 chỉ thấy {so_moc} lần dùng mốc pathspec. Mỗi lệnh nhận đường dẫn từ \
             webview phải đi qua mốc này; ít hơn nghĩa là có lệnh dựng pathspec thẳng, \
             và cổng không nhìn thấy nó"
        );

        for (i, sau) in than.match_indices("PATHSPEC_SAU_DAU_GACH(").skip(1) {
            let _ = sau;
            let truoc = &than[..i];
            let doan = &truoc[truoc.len().saturating_sub(200)..];
            assert!(
                doan.contains("\"--\"") || doan.contains(".arg(\"--\")"),
                "🔴 một chỗ gọi mốc pathspec KHÔNG có `--` trong 200 ký tự trước nó. \
                 Thiếu `--`, một tệp tên `main` làm `git checkout HEAD main` đổi NHÁNH \
                 thay vì khôi phục tệp. Đoạn đọc được:\n{doan}"
            );
        }
    }

    /// **Đường huỷ khối không bao giờ giải mã bản vá thành chuỗi** — WORK-04.
    ///
    /// `String::from_utf8_lossy` đổi mỗi byte không hợp lệ thành U+FFFD (**ba** byte),
    /// và ở đường này git sẽ ghi ba byte đó vào tệp thật của người dùng.
    #[test]
    fn duong_huy_khong_bao_gio_giai_ma_ban_va() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("stdin_bytes"),
            "🔴 tiền đề sai: không thấy `stdin_bytes` — hoặc đường bản vá đã đổi cách \
             đưa dữ liệu vào git, hoặc cổng đang đọc chuỗi rỗng"
        );

        // Cắt lấy đúng thân `huy_hunk`: phép giải mã ở `danh_sach_thung_rac` (đọc tên
        // ref — là ASCII theo dựng) và ở `luu_truoc_khi_huy` (đọc SHA) là hợp lệ, nên
        // một cổng toàn tệp sẽ đỏ vĩnh viễn vì lý do sai.
        let bat_dau = than
            .find("pub async fn huy_hunk")
            .expect("🔴 tiền đề sai: không thấy `huy_hunk` trong thân đã lọc");
        let than_huy = &than[bat_dau..];
        let ket = than_huy
            .find("pub const CO_HUY_HUNK")
            .expect("🔴 tiền đề sai: không thấy mốc kết thúc `CO_HUY_HUNK`");
        let than_huy = &than_huy[..ket];

        assert!(
            than_huy.contains("tach_hunk_tho(&ra_diff.stdout)"),
            "🔴 tiền đề sai: `huy_hunk` không còn tách bản vá từ `ra_diff.stdout` — \
             cổng dưới đây sẽ đo nhầm đoạn mã"
        );

        for cam in [
            "from_utf8_lossy(&ra_diff",
            "String::from_utf8(ra_diff",
            "to_string_lossy",
            "ra_diff.stderr",
        ] {
            assert!(
                !than_huy.contains(cam),
                "🔴 `{cam}` trên đường bản vá của `huy_hunk`. Bản vá phải đi từ \
                 `git diff` tới `git apply` dạng BYTE, không qua `String` ở bất kỳ \
                 điểm nào (WORK-04), và stderr KHÔNG BAO GIỜ ghép vào nội dung bản vá"
            );
        }
    }

    /// **`khoi_phuc` ghi byte thô ra tệp, không qua `String`** — đột biến M17.
    #[test]
    fn khoi_phuc_ghi_byte_tho() {
        let than = than_khong_chu_thich();

        let bat_dau = than
            .find("pub async fn khoi_phuc")
            .expect("🔴 tiền đề sai: không thấy `khoi_phuc` trong thân đã lọc");
        let than_kp = &than[bat_dau..];

        assert!(
            than_kp.contains("tokio::fs::write(&day_du, &ra.stdout)"),
            "🔴 tiền đề sai: `khoi_phuc` không còn ghi thẳng `ra.stdout` ra tệp. Nếu \
             cách ghi đổi, cập nhật cổng — đừng xoá nó"
        );

        for cam in [
            "from_utf8_lossy(&ra.stdout)",
            "String::from_utf8(ra.stdout",
            "from_utf8_lossy(&ra.stdout).to_string()",
        ] {
            assert!(
                !than_kp.contains(cam),
                "🔴 `{cam}` trên đường khôi phục. Đây là NỘI DUNG TỆP của người dùng: \
                 `from_utf8_lossy` đổi mỗi byte không hợp lệ thành U+FFFD — ba byte — \
                 rồi ghi ba byte đó đè lên tệp thật. Một tệp Latin-1 là dữ liệu có thật \
                 trên máy người dùng, không phải ca giả định"
            );
        }
    }

    /// **`restore_trash` chỉ nhận ref trong không gian riêng** — T-05-12.
    #[test]
    fn khoi_phuc_kiem_tien_to_ref() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("starts_with(TIEN_TO_REF)"),
            "🔴 `khoi_phuc` không kiểm tiền tố ref. `ref_name` đến từ webview: một \
             `refs/heads/main` lọt qua biến lời gọi thành \
             `git checkout refs/heads/main -- .`, tức ghi đè thư mục làm việc bằng nội \
             dung của một nhánh (T-05-12)"
        );
    }

    /// **Bước `rev-parse --verify` còn nguyên trong đường lưu** — đột biến M18.
    ///
    /// # Vì sao cổng này tồn tại dù M18 không đỏ ở đường hạnh phúc
    ///
    /// Plan dự đoán M18 (bỏ `rev-parse --verify`) **không đỏ** ở đường hạnh phúc, và dự
    /// đoán đó đúng: khi `update-ref` thành công thì đọc lại chỉ xác nhận thứ đã đúng.
    /// Nhưng `CONTEXT.md` §4.1 nói rõ — *trước khi kết luận một đột biến sống sót vì mã
    /// đúng, kiểm xem có test nào **hỏi về thứ đó** không; rỗng nghĩa là cổng thiếu,
    /// không phải mã đúng* (lỗi #9). Không có cổng này thì **không test nào** hỏi về
    /// bước đọc lại, và ai đó xoá nó sẽ thấy 0 đỏ.
    ///
    /// Thứ bước đó bảo vệ: một `update-ref` thoát 0 mà ref không tồn tại làm biên nhận
    /// **nói dối**, và đường huỷ sẽ ghi đè tệp người dùng vì tin rằng nội dung đã an toàn.
    #[test]
    fn duong_luu_doc_lai_ref_truoc_khi_cap_bien_nhan() {
        let than = than_khong_chu_thich();

        let bat_dau = than
            .find("async fn ghi_ref")
            .expect("🔴 tiền đề sai: không thấy `ghi_ref` trong thân đã lọc");
        let than_ghi = &than[bat_dau..];
        let ket = than_ghi
            .find("fn nhan_cho")
            .expect("🔴 tiền đề sai: không thấy mốc kết thúc `nhan_cho`");
        let than_ghi = &than_ghi[..ket];

        assert!(
            than_ghi.contains("\"rev-parse\"") && than_ghi.contains("\"--verify\""),
            "🔴 `ghi_ref` không còn đọc lại ref bằng `rev-parse --verify`. Thiếu bước \
             này, một `update-ref` thoát 0 mà ref không tồn tại vẫn cấp biên nhận — và \
             biên nhận là thứ DUY NHẤT đường huỷ tin để cho phép ghi đè tệp người dùng"
        );
        assert!(
            than_ghi.contains("return Err("),
            "🔴 `ghi_ref` không trả `Err` khi đọc lại thất bại. Ghi nhật ký rồi đi \
             tiếp nghĩa là vẫn huỷ mà không có bản lưu"
        );
    }

    /// **Đường lưu không bao giờ ghép stderr vào sha.**
    ///
    /// Đo được hôm nay: `git stash create` in `warning: in the working copy of 'a.txt',
    /// LF will be replaced by CRLF`, và với `$(...)` trong shell cảnh báo đó rơi vào
    /// **cùng luồng** với sha. `GitOutput` tách sẵn hai luồng; việc duy nhất phải làm là
    /// không ghép chúng lại.
    #[test]
    fn duong_luu_khong_ghep_stderr_vao_sha() {
        let than = than_khong_chu_thich();

        let bat_dau = than
            .find("pub async fn luu_truoc_khi_huy")
            .expect("🔴 tiền đề sai: không thấy `luu_truoc_khi_huy` trong thân đã lọc");
        let than_luu = &than[bat_dau..];
        let ket = than_luu
            .find("async fn ghi_ref")
            .expect("🔴 tiền đề sai: không thấy mốc kết thúc `ghi_ref`");
        let than_luu = &than_luu[..ket];

        assert!(
            than_luu.contains("&ra.stdout"),
            "🔴 tiền đề sai: `luu_truoc_khi_huy` không còn đọc `ra.stdout` — cổng đo \
             nhầm đoạn mã"
        );
        assert!(
            !than_luu.contains("ra.stderr"),
            "🔴 `luu_truoc_khi_huy` chạm vào `stderr`. Một dòng `warning: LF will be \
             replaced by CRLF` ghép vào sha cho một SHA không tra cứu được, và ta sẽ \
             ghi một ref trỏ vào rác — rồi tin là đã lưu xong"
        );
    }

    /// **Phân nhánh chưa-theo-dõi đi theo TỪNG đường dẫn, không theo sha của stash.**
    ///
    /// Đây là cổng cho ca hỗn hợp mà plan không lường: đo được, một repo có **cả** tệp
    /// đã theo dõi bẩn **và** tệp chưa theo dõi cho `stash create` một sha **khác
    /// rỗng**, mà cây của sha đó **không chứa** tệp chưa theo dõi. Một cài đặt phân
    /// nhánh theo "stdout có rỗng không" sẽ đi nhánh stash, thấy sha hợp lệ, và mất tệp
    /// chưa theo dõi trong im lặng.
    #[test]
    fn phan_nhanh_chua_theo_doi_theo_tung_duong_dan() {
        let than = than_khong_chu_thich();

        assert!(
            than.contains("fn la_chua_theo_doi"),
            "🔴 tiền đề sai: không thấy `la_chua_theo_doi` — phép phân nhóm theo từng \
             đường dẫn đã biến mất"
        );
        assert!(
            than.contains("la_chua_theo_doi(state, &repo, p).await?"),
            "🔴 `luu_truoc_khi_huy` không phân nhóm theo TỪNG đường dẫn. Nếu nó phân \
             nhánh theo việc `stash create` trả rỗng hay không thì ca HỖN HỢP (một tệp \
             đã theo dõi bẩn + một tệp chưa theo dõi) sẽ đi nhánh stash và mất tệp chưa \
             theo dõi — đo được, cây của stash commit KHÔNG chứa nó"
        );
        assert!(
            than.contains("StatusGroup::Untracked"),
            "🔴 phép phân nhóm không còn đọc `StatusGroup`. Plan đòi phân nhánh theo \
             nhóm trạng thái, KHÔNG đoán theo việc tệp có trong index hay không"
        );
    }

    /// Nhãn dùng **đúng** động từ cho từng nhóm.
    #[test]
    fn nhan_dung_dong_tu_theo_nhom() {
        assert_eq!(
            nhan_cho(&["bar.txt".to_owned()], true),
            "Xoá bar.txt",
            "🔴 tệp chưa theo dõi phải dùng `Xoá` — nội dung đó không có ở đâu khác"
        );
        assert_eq!(
            nhan_cho(&["bar.txt".to_owned()], false),
            "Huỷ bỏ bar.txt",
            "🔴 tệp đã theo dõi dùng `Huỷ bỏ` — có bản gốc trong git để quay về"
        );
        assert_eq!(
            nhan_cho(&["a".to_owned(), "b".to_owned()], false),
            "Huỷ bỏ 2 tệp"
        );
        assert_ne!(
            nhan_cho(&["bar.txt".to_owned()], true),
            nhan_cho(&["bar.txt".to_owned()], false),
            "🔴 hai nhóm phải cho nhãn KHÁC nhau, nếu không thì phân nhánh vô nghĩa và \
             một cài đặt trả cùng một chuỗi cho mọi thứ vẫn xanh"
        );
    }

    /// Giới hạn danh sách là một **chặn trên có thật**.
    #[test]
    fn gioi_han_danh_sach_hop_ly() {
        assert_eq!(GIOI_HAN_DANH_SACH, 50);
        assert!(
            GIOI_HAN_DANH_SACH > 0,
            "🔴 giới hạn 0 làm danh sách LUÔN rỗng, tức không khôi phục được gì"
        );
    }
}
