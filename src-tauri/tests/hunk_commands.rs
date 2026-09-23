//! Test tích hợp cho `stage_hunk` / `unstage_hunk` — plan 05-02, WORK-03/04/05.
//!
//! # Vì sao chạy git THẬT, và vì sao điều đó là cả điểm của wave này
//!
//! `patch_build.rs` đã có test đơn vị trên byte tự dựng (plan 05-01), và ở đó buffer tự
//! dựng là **đúng** vì thứ được kiểm là một hàm thuần. Ở đây thứ được kiểm là **git có
//! chấp nhận bản vá ta dựng không** và **index có đổi đúng chỗ không** — chỉ git thật
//! trả lời được cả hai. 05-01 dựng bản vá con nhưng **chưa bao giờ** đưa chúng qua
//! `git apply`; đó là việc của tệp này.
//!
//! Bài học `%x1f` của 02-04: 19 test đơn vị xanh trên buffer tự dựng trong khi lệnh git
//! thật trả về định dạng khác, và thanh bên rỗng hoàn toàn trong im lặng.
//!
//! # Mỗi test dựng repo RIÊNG
//!
//! Mọi test ở đây **ghi vào index**, nên không test nào dùng được một fixture chỉ đọc.
//! `tempfile::tempdir()` cho mỗi test, đúng khuôn `repo_ban()` của
//! `tests/worktree_commands.rs`.
//!
//! # 🔴 Không test nào ở đây render một component
//!
//! Tầng Rust của Phase 5 phải **tự đứng được** (CONTEXT.md mục 0.1): Phase 4 giao cả
//! sáu requirement mà không requirement nào có bằng chứng từ mắt người, nên xây một cổng
//! của Phase 5 lên `ChangeList` nghĩa là xây lên nền chưa ai nhìn.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use git_plum_lib::commands::hunk::{stage_mot_khoi, unstage_mot_khoi, CO_APPLY};
use git_plum_lib::state::{AppState, RepoHandle};

/// Chạy một lệnh git đồng bộ trong lúc dựng bối cảnh. Không đi qua `GitRunner` có chủ
/// ý: đây là phần **dựng bối cảnh**, không phải phần được kiểm.
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

/// Chạy git và trả `(exit code, stdout, stderr)` **tách riêng**, không đòi thành công.
///
/// 🔴 Hai luồng **tách riêng**, không ghép. Đo được hôm nay: bắt đầu ra bằng `$(...)`
/// trong shell làm `warning: LF will be replaced by CRLF` của clean filter rơi vào cùng
/// luồng với nội dung thật. Đây đúng là lý do ROADMAP đòi tách hai luồng, và nó áp cho
/// `git apply` y hệt.
fn git_thu(dir: &Path, args: &[&str]) -> (i32, Vec<u8>, Vec<u8>) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("không chạy được git {args:?}: {e}"));
    (
        out.status.code().unwrap_or(-1),
        out.stdout.clone(),
        out.stderr,
    )
}

/// `git diff` dạng byte thô. `read_ok` tương đương của phần dựng bối cảnh.
fn diff_tho(dir: &Path, args: &[&str]) -> Vec<u8> {
    let (ma, stdout, stderr) = git_thu(dir, args);
    assert!(
        ma == 0 || ma == 1,
        "git {args:?} thoát {ma}: {}",
        String::from_utf8_lossy(&stderr)
    );
    stdout
}

/// Số hunk trong một bản vá: đếm dòng bắt đầu bằng `@@`.
///
/// 🔴 So khớp ở **đầu dòng**, không phải "có chứa" — một dòng **nội dung** hợp lệ có
/// thể là ` @@ ...` (dấu cách ngữ cảnh + văn bản), và đếm nó thành header làm con số
/// nói dối. Cùng lý do `DAU_HUNK` của `patch_build.rs` dùng `starts_with`.
fn dem_hunk(patch: &[u8]) -> usize {
    patch
        .split(|b| *b == b'\n')
        .filter(|d| d.starts_with(b"@@"))
        .count()
}

/// Mã băm blob hiện tại của một tệp — thứ giao diện sẽ gửi kèm khi bấm stage.
fn bam_blob(dir: &Path, path: &str) -> String {
    let (ma, stdout, stderr) = git_thu(dir, &["hash-object", "--", path]);
    assert_eq!(
        ma,
        0,
        "hash-object thất bại: {}",
        String::from_utf8_lossy(&stderr)
    );
    String::from_utf8_lossy(&stdout).trim().to_owned()
}

/// Repo tạm với một tệp `f.txt` 26 dòng (`a`..`z`), đã commit, rồi sửa dòng 2/13/25 →
/// **đúng ba hunk** ở `-U3`.
///
/// # Vì sao ba hunk cách xa nhau, không phải ba dòng cạnh nhau
///
/// Với `-U3`, hai thay đổi cách nhau dưới 7 dòng bị git **gộp** thành một hunk. Ba dòng
/// 2/13/25 cách nhau 11 và 12 dòng, nên chúng chắc chắn là ba hunk riêng. Một fixture
/// "đúng hình dạng" (ba thay đổi) mà git gộp thành một hunk là fixture **vô hại**:
/// `hunk_index=1` sẽ trượt biên và test đỏ vì lý do sai, hoặc — tệ hơn — một cài đặt
/// stage cả tệp vẫn xanh (CONTEXT.md 4.2).
///
/// Mọi test khẳng định số hunk **trước** khi tin vào chỉ số.
fn repo_ba_hunk() -> (tempfile::TempDir, AppState, Arc<RepoHandle>) {
    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    git(p, &["config", "user.name", "Test"]);
    git(p, &["config", "user.email", "test@example.com"]);
    // `core.autocrlf=false`: một chuyển đổi CRLF làm `git status` báo tệp sửa mà test
    // không mong. Test C đặt `true` **tường minh** vì ở đó đó chính là thứ được kiểm.
    git(p, &["config", "core.autocrlf", "false"]);

    let goc: Vec<String> = (b'a'..=b'z').map(|c| (c as char).to_string()).collect();
    std::fs::write(p.join("f.txt"), format!("{}\n", goc.join("\n"))).unwrap();
    git(p, &["add", "f.txt"]);
    git(p, &["commit", "-m", "commit goc"]);

    let mut sua = goc.clone();
    sua[1] = "M1".to_owned();
    sua[12] = "CHANGED15".to_owned();
    sua[24] = "M3".to_owned();
    std::fs::write(p.join("f.txt"), format!("{}\n", sua.join("\n"))).unwrap();

    let state = AppState::new();
    let repo = state.open_repo(p);
    (dir, state, repo)
}

// ---------------------------------------------------------------------------
// Test A — stage hunk THỨ HAI của một tệp ba hunk
// ---------------------------------------------------------------------------

/// 🔴 **Stage đúng khối thứ hai: index có đúng khối đó, worktree còn đúng hai khối.**
///
/// # Vì sao khẳng định CẢ HAI số
///
/// Chỉ khẳng định "cached có hunk giữa" thì một cài đặt stage **cả tệp** vẫn **xanh** —
/// cả tệp cũng chứa hunk giữa. Con số thứ hai (`git diff` chưa stage còn đúng 2 hunk) là
/// thứ phân biệt "stage một khối" với "stage cả tệp". Đây đúng lớp lỗi "hình dạng đúng,
/// dữ liệu vô hại" của CONTEXT.md 4.2.
///
/// Và khẳng định **nội dung** (`CHANGED15`), không chỉ số đếm: một cài đặt stage hunk
/// **0** cũng cho "cached đúng 1 hunk, worktree đúng 2 hunk" — đột biến M9 sống sót qua
/// một cổng chỉ đếm.
#[tokio::test]
async fn stage_hunk_thu_hai_dua_dung_khoi_giua_vao_index() {
    let (dir, state, repo) = repo_ba_hunk();
    let p = dir.path();

    // --- Tiền đề: bản vá phải có ĐÚNG ba hunk ------------------------------
    let truoc = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&truoc),
        3,
        "🔴 tiền đề sai: fixture phải cho đúng 3 hunk, nếu không thì `hunk_index=1` \
         không có nghĩa là 'khối giữa'. Bản vá đọc được:\n{}",
        String::from_utf8_lossy(&truoc)
    );

    let hash = bam_blob(p, "f.txt");

    let sau = stage_mot_khoi(&state, Arc::clone(&repo), "f.txt", 1, &hash)
        .await
        .expect("stage khối giữa phải Ok");

    // --- Index: ĐÚNG MỘT hunk, và nó là khối GIỮA ---------------------------
    let cached = diff_tho(p, &["diff", "--cached", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&cached),
        1,
        "🔴 index phải chứa đúng MỘT khối. Nhiều hơn nghĩa là cả tệp vào index. \
         Đọc được:\n{}",
        String::from_utf8_lossy(&cached)
    );

    let cached_str = String::from_utf8_lossy(&cached);
    assert!(
        cached_str.contains("CHANGED15"),
        "🔴 khối vào index phải là khối GIỮA. Chỉ đếm hunk thì đột biến \
         `dung_ban_va_mot_hunk(0)` vẫn cho đúng 1 hunk và cổng không thấy gì. \
         Đọc được:\n{cached_str}"
    );
    assert!(
        !cached_str.contains("M1") && !cached_str.contains("M3"),
        "🔴 index KHÔNG được chứa khối đầu (`M1`) hay khối cuối (`M3`). \
         Đọc được:\n{cached_str}"
    );

    // --- Worktree: hai khối kia CÒN chưa stage ------------------------------
    let chua_stage = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&chua_stage),
        2,
        "🔴 hai khối kia phải CÒN ở worktree chưa stage. Con số này là thứ phân biệt \
         'stage một khối' với 'stage cả tệp' — thiếu nó thì một cài đặt stage cả tệp \
         vẫn xanh ở vế trên. Đọc được:\n{}",
        String::from_utf8_lossy(&chua_stage)
    );

    // --- Giá trị TRẢ VỀ là trạng thái mới, không phải `()` -----------------
    assert!(
        sau.entries.iter().any(|e| e.path == "f.txt"),
        "giá trị trả về phải là trạng thái MỚI có chứa tệp vừa đổi (CONTEXT.md 2.5)"
    );
}

// ---------------------------------------------------------------------------
// Test B — ca `--recount`, fixture ĐÃ ĐO là phân biệt được
// ---------------------------------------------------------------------------

/// 🔴 **Bỏ `--recount` làm một bản vá cắt-thân THẤT BẠI — chứng minh trong chính test.**
///
/// # Fixture này được CHỨNG MINH phân biệt được, không được chỉ định
///
/// Giả định tự nhiên — *"chọn hunk thứ hai trong ba hunk thì bỏ `--recount` sẽ đỏ"* —
/// đã được **đo và bác bỏ** trên git 2.54.0.windows.1:
///
/// ```text
/// hunk 2 chép NGUYÊN VĂN header:
///   git apply --check --cached --recount h2.patch → exit 0
///   git apply --check --cached           h2.patch → exit 0    ← KHÔNG đỏ
/// ```
///
/// Số đếm trong `@@` là **của riêng từng hunk**, không tích luỹ qua cả tệp, nên chép
/// nguyên header hunk 2 thì nó vẫn đúng dù bỏ bao nhiêu hunk khác. Fixture đó **trông
/// như** đang kiểm `--recount` mà **không kiểm gì** — CONTEXT.md 4.2 ở dạng thuần khiết
/// nhất.
///
/// Thứ làm header **thật sự sai** là **cắt bớt dòng thân** trong khi giữ số đếm cũ —
/// đúng việc một bộ dựng bản vá con làm khi nó tỉa ngữ cảnh:
///
/// ```text
/// @@ -10,7 +10,7 @@ i      ← khai 7 dòng
///  j / k / l / -m / +M2 / n   ← thân chỉ còn 6
///   với    --recount → exit 0
///   không  --recount → exit 128  "corrupt patch"
/// ```
///
/// # Vì sao test này KHÔNG gọi `stage_mot_khoi`
///
/// Một cài đặt **đúng** không sinh ra header lệch, nên gọi `stage_mot_khoi` sẽ không bao
/// giờ chạm tới ca `--recount` bảo vệ. Test này kiểm trực tiếp rằng lệnh `git apply` của
/// ta **mang** `--recount`, bằng cách chạy **chính tập cờ** `CO_APPLY` mà `hunk.rs`
/// dùng.
///
/// # Vì sao đọc `CO_APPLY` chứ không chép danh sách cờ
///
/// Chép lại là **hai nguồn lệch nhau**: ai đó bỏ `--recount` khỏi mã thật thì test vẫn
/// chạy trên bản chép của nó và vẫn xanh. `STATUS_ARGS` tồn tại từ 04-01 đúng vì lý do
/// này. Bước 4 dưới đây chứng minh fixture phân biệt được **ngay trong chính test**, nên
/// cổng không thể âm thầm mất hiệu lực.
#[tokio::test]
async fn bo_recount_lam_ban_va_cat_than_that_bai() {
    let (dir, _state, _repo) = repo_ba_hunk();
    let p = dir.path();

    // 🔴 Khẳng định TIỀN ĐỀ: không có `--recount` trong `CO_APPLY` thì test này không
    // kiểm gì và phải ĐỎ, không phải xanh.
    assert!(
        CO_APPLY.contains(&"--recount"),
        "🔴 tiền đề sai: `CO_APPLY` không có `--recount`, test này không kiểm gì. \
         CO_APPLY = {CO_APPLY:?}"
    );

    // --- Dựng bản vá CẮT THÂN, header để nguyên -----------------------------
    let day_du = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    let dong: Vec<&[u8]> = day_du.split(|b| *b == b'\n').collect();

    let mut vi_tri_hunk: Vec<usize> = Vec::new();
    for (i, d) in dong.iter().enumerate() {
        if d.starts_with(b"@@") {
            vi_tri_hunk.push(i);
        }
    }
    assert_eq!(
        vi_tri_hunk.len(),
        3,
        "🔴 tiền đề sai: cần đúng 3 hunk để cắt được hunk giữa"
    );

    let dau_h2 = vi_tri_hunk[1];
    let cuoi_h2 = vi_tri_hunk[2];

    let mut cat_than: Vec<u8> = Vec::new();
    // Phần đầu tệp: mọi dòng trước `@@` đầu tiên. Không có nó, git không biết bản vá
    // nói về tệp nào và từ chối với "no such file" — một lý do đỏ SAI.
    for d in &dong[..vi_tri_hunk[0]] {
        cat_than.extend_from_slice(d);
        cat_than.push(b'\n');
    }
    // Hunk giữa, **thiếu hai dòng thân cuối**: header vẫn khai số cũ.
    let than_h2 = &dong[dau_h2..cuoi_h2];
    assert!(
        than_h2.len() >= 4,
        "🔴 tiền đề sai: hunk giữa chỉ có {} dòng, cắt 2 dòng không còn gì để git đọc",
        than_h2.len()
    );
    for d in &than_h2[..than_h2.len() - 2] {
        cat_than.extend_from_slice(d);
        cat_than.push(b'\n');
    }

    // 🔴 Chứng minh fixture KHÁC bản vá hunk-2-nguyên-văn. Nếu phép cắt trượt và cho ra
    // một bản vá nguyên vẹn thì cả hai bước dưới đều exit 0 và test xanh mà không kiểm
    // gì — đúng lớp lỗi mà chính test này tồn tại để chống.
    let mut nguyen_van: Vec<u8> = Vec::new();
    for d in &dong[..vi_tri_hunk[0]] {
        nguyen_van.extend_from_slice(d);
        nguyen_van.push(b'\n');
    }
    for d in than_h2 {
        nguyen_van.extend_from_slice(d);
        nguyen_van.push(b'\n');
    }
    assert_ne!(
        cat_than, nguyen_van,
        "🔴 phép cắt thân không cắt được gì — fixture trở thành hunk-2-nguyên-văn, đã \
         ĐO là KHÔNG phân biệt được `--recount`"
    );

    let duong_va = p.join("cat_than.patch");
    std::fs::write(&duong_va, &cat_than).unwrap();
    let ten_va = "cat_than.patch";

    // --- Bước 3: VỚI `--recount` (chính tập cờ của hunk.rs) → exit 0 --------
    let mut co_recount: Vec<&str> = CO_APPLY.to_vec();
    co_recount.push("--check");
    co_recount.push(ten_va);
    let (ma_co, _out_co, err_co) = git_thu(p, &co_recount);
    assert_eq!(
        ma_co,
        0,
        "🔴 VỚI `--recount`, bản vá cắt-thân phải áp được (git tính lại số đếm từ nội \
         dung thật). Thất bại ở đây nghĩa là `CO_APPLY` mất `--recount`, hoặc phép \
         dựng bản vá của test hỏng. git nói: {}",
        String::from_utf8_lossy(&err_co)
    );

    // --- Bước 4: KHÔNG `--recount` → exit khác 0 ---------------------------
    //
    // Đây là bước làm cổng CÓ KHẢ NĂNG đỏ: nó chứng minh fixture phân biệt được ngay
    // trong chính test. Thiếu nó, bước 3 xanh trên một bản vá mà `--recount` không hề
    // cần thiết, và cổng không nói gì về `--recount` cả.
    let mut khong_recount: Vec<&str> = CO_APPLY
        .iter()
        .copied()
        .filter(|c| *c != "--recount")
        .collect();
    khong_recount.push("--check");
    khong_recount.push(ten_va);
    let (ma_khong, _out_khong, err_khong) = git_thu(p, &khong_recount);

    assert_ne!(
        ma_khong, 0,
        "🔴 fixture KHÔNG phân biệt được `--recount`: bỏ cờ mà git vẫn chấp nhận bản \
         vá. Cổng này đang đo con số không. Bản vá đã dựng:\n{}",
        String::from_utf8_lossy(&cat_than)
    );
    assert!(
        String::from_utf8_lossy(&err_khong).contains("corrupt patch"),
        "🔴 thiếu `--recount`, git phải từ chối vì header khai sai số dòng \
         (`corrupt patch`). Một lý do đỏ KHÁC nghĩa là fixture đỏ vì chuyện khác và \
         không nói gì về `--recount`. git nói: {}",
        String::from_utf8_lossy(&err_khong)
    );
}

// ---------------------------------------------------------------------------
// Test C — WORK-04, đo bằng BYTE
// ---------------------------------------------------------------------------

/// 🔴 **Sau staging một phần: CRLF giữ nguyên, thiếu dòng cuối giữ nguyên, `0xE9` còn.**
///
/// # Đo trên BYTE, không đọc bằng `read_to_string`
///
/// `read_to_string` **lỗi** trên `0xE9` (không phải UTF-8 hợp lệ), và một biến thể lossy
/// của nó đổi byte đó thành U+FFFD — **ba** byte — rồi test sẽ nói dối về thứ nằm trên
/// đĩa. `std::fs::read` → `Vec<u8>` là cách duy nhất câu trả lời còn đúng.
///
/// # `core.autocrlf=true` — tường minh, vì đó chính là thứ được kiểm
///
/// Mặc định `true` trên Windows, tức máy chủ dự án. Với `autocrlf=true`, `git diff` in
/// bản vá bằng **LF** (clean filter đã bỏ `\r`) và blob trong index cũng là LF — đó là
/// **đúng**, `autocrlf=true` nghĩa là vậy. Thứ WORK-04 bảo vệ là **tệp trong thư mục
/// làm việc**, và nó không được đổi một byte.
#[tokio::test]
async fn staging_mot_phan_giu_nguyen_crlf_thieu_dong_cuoi_va_byte_khong_utf8() {
    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    git(p, &["config", "user.name", "Test"]);
    git(p, &["config", "user.email", "test@example.com"]);
    // 🔴 TƯỜNG MINH `true` — đây là cấu hình mặc định trên máy người dùng Windows, và
    // là thứ test này tồn tại để kiểm.
    git(p, &["config", "core.autocrlf", "true"]);

    // 30 dòng CRLF, `0xE9` (`é` Latin-1) ở dòng 6, KHÔNG có dòng trống cuối.
    let dung_tep = |d2: &str, d13: &str, d25: &str| -> Vec<u8> {
        let mut ra: Vec<u8> = Vec::new();
        for i in 0..30 {
            let noi_dung: Vec<u8> = match i {
                1 => d2.as_bytes().to_vec(),
                5 => vec![b'l', b'a', b't', b'i', b'n', 0xE9],
                12 => d13.as_bytes().to_vec(),
                24 => d25.as_bytes().to_vec(),
                _ => format!("dong{i}").into_bytes(),
            };
            ra.extend_from_slice(&noi_dung);
            if i < 29 {
                ra.extend_from_slice(b"\r\n");
            }
            // Dòng cuối KHÔNG có byte kết dòng — ca `\ No newline at end of file`.
        }
        ra
    };

    std::fs::write(p.join("f.txt"), dung_tep("goc2", "goc13", "goc25")).unwrap();
    git(p, &["add", "f.txt"]);
    git(p, &["commit", "-m", "goc"]);

    let moi = dung_tep("M1", "CHANGED15", "M3");
    std::fs::write(p.join("f.txt"), &moi).unwrap();

    // --- Đo TRƯỚC, trên byte -----------------------------------------------
    let truoc_byte = std::fs::read(p.join("f.txt")).unwrap();
    let truoc_cr = truoc_byte.iter().filter(|b| **b == b'\r').count();
    assert_eq!(
        truoc_cr, 29,
        "🔴 tiền đề sai: tệp phải có 29 byte `\\r`. Một fixture toàn LF là fixture \
         'đúng hình dạng, dữ liệu vô hại' — nó không cho lỗi CRLF để lại dấu vết nào"
    );
    assert_ne!(
        truoc_byte.last(),
        Some(&b'\n'),
        "🔴 tiền đề sai: tệp KHÔNG được có dòng trống cuối"
    );
    assert!(
        truoc_byte.contains(&0xE9),
        "🔴 tiền đề sai: tệp phải có byte 0xE9 — một fixture toàn ASCII không cho lỗi \
         giải mã để lại dấu vết nào"
    );

    let state = AppState::new();
    let repo = state.open_repo(p);

    let day_du = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&day_du),
        3,
        "🔴 tiền đề sai: cần đúng 3 hunk. Đọc được:\n{}",
        String::from_utf8_lossy(&day_du)
    );

    let hash = bam_blob(p, "f.txt");
    stage_mot_khoi(&state, Arc::clone(&repo), "f.txt", 1, &hash)
        .await
        .expect("stage khối giữa phải Ok");

    // --- Đo SAU, trên byte --------------------------------------------------
    let sau_byte = std::fs::read(p.join("f.txt")).unwrap();

    assert_eq!(
        sau_byte.iter().filter(|b| **b == b'\r').count(),
        truoc_cr,
        "🔴 WORK-04: số byte `\\r` trong THƯ MỤC LÀM VIỆC phải KHÔNG ĐỔI sau staging \
         một phần. Đổi nghĩa là `git apply` đã ghi vào tệp thật — tức thiếu `--cached`"
    );
    assert_ne!(
        sau_byte.last(),
        Some(&b'\n'),
        "🔴 WORK-04: tệp KHÔNG có dòng trống cuối trước khi stage, nên nó cũng không \
         được có sau. Thêm một byte `\\n` vào cuối tệp người dùng là sửa nội dung của họ"
    );
    assert!(
        sau_byte.contains(&0xE9),
        "🔴 WORK-04: byte 0xE9 phải còn NGUYÊN. Mất nó nghĩa là bản vá đã đi qua một \
         phép giải mã lossy: U+FFFD là BA byte và git ghi cả ba vào tệp thật"
    );
    assert_eq!(
        sau_byte, truoc_byte,
        "🔴 WORK-04: `--cached` nghĩa là index đổi, THƯ MỤC LÀM VIỆC không đổi MỘT \
         BYTE. Phép so trên toàn bộ nội dung là phép kiểm mạnh nhất có được ở đây"
    );

    // --- Index: đúng khối giữa, và `0xE9` vẫn tới được ---------------------
    let cached = diff_tho(p, &["diff", "--cached", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&cached),
        1,
        "index phải chứa đúng một khối. Đọc được:\n{}",
        String::from_utf8_lossy(&cached)
    );
    assert!(
        String::from_utf8_lossy(&cached).contains("CHANGED15"),
        "khối vào index phải là khối GIỮA"
    );

    let noi_dung_blob = diff_tho(p, &["show", ":f.txt"]);
    assert!(
        noi_dung_blob.contains(&0xE9),
        "🔴 byte 0xE9 phải có mặt trong BLOB đã stage, không chỉ trong worktree — \
         nếu nó mất ở đây thì commit sinh ra sẽ mang nội dung sai"
    );
}

// ---------------------------------------------------------------------------
// Test D — WORK-05, tệp đã đổi sau khi vẽ diff
// ---------------------------------------------------------------------------

/// 🔴 **Tệp đổi sau khi lấy `blob_hash` → `file_changed`, và index vẫn SẠCH.**
///
/// # Vì sao khẳng định cả index sạch, không chỉ mã lỗi
///
/// Mã lỗi một mình không phân biệt được "từ chối trước khi làm gì" với "áp một phần rồi
/// mới báo lỗi". Đột biến M7 (bỏ bước `--check`, áp thẳng) giữ nguyên mã lỗi ở một số
/// đường mà vẫn để lại index **bẩn** — và một index nửa vời là thứ người dùng không có
/// cách nào biết.
///
/// Chủ dự án chạy git ở terminal song song (WORK-10), nên ca này là đường đi thường
/// ngày, không phải phòng xa.
#[tokio::test]
async fn tep_doi_sau_khi_ve_diff_cho_file_changed_va_index_van_sach() {
    let (dir, state, repo) = repo_ba_hunk();
    let p = dir.path();

    // Giao diện vẽ diff và ghi lại hash ở thời điểm đó.
    let hash_cu = bam_blob(p, "f.txt");

    // ...rồi tệp đổi ở nơi khác: một `git checkout`, một lần lưu trong editor, một bộ
    // formatter chạy lúc lưu.
    std::fs::write(p.join("f.txt"), "noi dung hoan toan khac\n").unwrap();
    let hash_moi = bam_blob(p, "f.txt");
    assert_ne!(
        hash_cu, hash_moi,
        "🔴 tiền đề sai: tệp phải thật sự đổi hash, nếu không test này không kiểm gì"
    );

    let loi = stage_mot_khoi(&state, Arc::clone(&repo), "f.txt", 1, &hash_cu)
        .await
        .expect_err("🔴 stage với hash CŨ phải thất bại — bản vá dựng từ nội dung cũ áp lên tệp mới là đúng ca áp SAI VỊ TRÍ (R1)");

    assert_eq!(
        loi.code(),
        "file_changed",
        "🔴 mã lỗi phải là `file_changed` để giao diện hiện được đường làm mới. \
         Đọc được `{}`, thông điệp: {loi}",
        loi.code()
    );
    assert!(
        loi.to_string().contains("hãy làm mới"),
        "🔴 thông điệp phải nói việc cần làm — NGUYÊN VĂN ROADMAP: {loi}"
    );

    // 🔴 Index vẫn SẠCH: không một byte nào lọt vào.
    let cached = diff_tho(p, &["diff", "--cached", "--unified=3", "--", "f.txt"]);
    assert!(
        cached.is_empty(),
        "🔴 thất bại phải để lại index SẠCH. Bất cứ gì ở đây nghĩa là bản vá đã được \
         áp một phần trước khi lỗi được phát hiện — tức bước `--check` không còn đứng \
         trước lần áp thật. Đọc được:\n{}",
        String::from_utf8_lossy(&cached)
    );
}

// ---------------------------------------------------------------------------
// Test E — tệp nhị phân (R7, T-05-06)
// ---------------------------------------------------------------------------

/// 🔴 **Tệp nhị phân → lỗi rõ ràng, KHÔNG panic, và không áp gì.**
///
/// `git diff` trên tệp nhị phân in `Binary files ... differ` và **không** có dòng `@@`,
/// nên `tach_hunk_tho` cho `hunks.len() == 0` và `dung_ban_va_mot_hunk` trả `None` cho
/// mọi chỉ số.
///
/// # Vì sao "không panic" là một khẳng định thật
///
/// `hunk_index` đến từ webview, tức là **số tuỳ ý**. Index trực tiếp `hunks[chi_so]` sẽ
/// panic, và một panic trong một Tauri command làm chết **cả tiến trình backend** —
/// người dùng mất mọi repo đang mở, không có thông báo nào. Đây là T-05-06.
///
/// Test cũng phủ chỉ số ngoài biên trên một tệp văn bản **bình thường**: cùng đường mã,
/// nhưng nó là ca xảy ra thật khi giao diện gửi chỉ số của một bản vá đã cũ.
#[tokio::test]
async fn tep_nhi_phan_va_chi_so_ngoai_bien_cho_loi_ro_rang_khong_panic() {
    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    git(p, &["config", "user.name", "Test"]);
    git(p, &["config", "user.email", "test@example.com"]);
    git(p, &["config", "core.autocrlf", "false"]);

    // Byte NUL ở đầu: git nhận ra là nhị phân.
    std::fs::write(p.join("b.bin"), [0u8, 1, 2, 3, 0, 4, 5, 6]).unwrap();
    std::fs::write(p.join("t.txt"), "mot\nhai\nba\n").unwrap();
    git(p, &["add", "b.bin", "t.txt"]);
    git(p, &["commit", "-m", "goc"]);

    std::fs::write(p.join("b.bin"), [0u8, 9, 9, 9, 0, 8, 8, 8]).unwrap();
    std::fs::write(p.join("t.txt"), "MOT\nhai\nba\n").unwrap();

    let state = AppState::new();
    let repo = state.open_repo(p);

    // --- Tiền đề: git thật sự coi tệp này là nhị phân ----------------------
    let dv = diff_tho(p, &["diff", "--unified=3", "--", "b.bin"]);
    assert_eq!(
        dem_hunk(&dv),
        0,
        "🔴 tiền đề sai: `git diff` trên tệp nhị phân không được có dòng `@@`. \
         Đọc được:\n{}",
        String::from_utf8_lossy(&dv)
    );

    let hash_bin = bam_blob(p, "b.bin");
    let loi = stage_mot_khoi(&state, Arc::clone(&repo), "b.bin", 0, &hash_bin)
        .await
        .expect_err("🔴 tệp nhị phân phải cho lỗi, không phải Ok — không có khối nào để chọn");

    assert!(
        !loi.to_string().is_empty(),
        "lỗi phải có thông điệp đọc được"
    );

    let cached = diff_tho(p, &["diff", "--cached", "--", "b.bin"]);
    assert!(
        cached.is_empty(),
        "🔴 tệp nhị phân: KHÔNG được áp gì cả. Đọc được:\n{}",
        String::from_utf8_lossy(&cached)
    );

    // --- Chỉ số ngoài biên trên tệp văn bản: cùng đường, ca có thật --------
    let dt = diff_tho(p, &["diff", "--unified=3", "--", "t.txt"]);
    assert_eq!(
        dem_hunk(&dt),
        1,
        "🔴 tiền đề sai: `t.txt` phải cho đúng 1 khối, nên chỉ số 99 là ngoài biên"
    );

    let hash_txt = bam_blob(p, "t.txt");
    let loi2 = stage_mot_khoi(&state, Arc::clone(&repo), "t.txt", 99, &hash_txt)
        .await
        .expect_err("🔴 chỉ số ngoài biên phải cho lỗi, KHÔNG panic — `hunk_index` đến từ webview và một panic làm chết cả tiến trình backend");

    assert!(
        !loi2.to_string().is_empty(),
        "lỗi chỉ số ngoài biên phải có thông điệp đọc được"
    );

    let cached2 = diff_tho(p, &["diff", "--cached", "--", "t.txt"]);
    assert!(
        cached2.is_empty(),
        "🔴 chỉ số ngoài biên: KHÔNG được áp gì. Đọc được:\n{}",
        String::from_utf8_lossy(&cached2)
    );
}

// ---------------------------------------------------------------------------
// Test F — unstage một khối
// ---------------------------------------------------------------------------

/// 🔴 **Unstage một khối gỡ ĐÚNG khối đó khỏi index; hai khối kia còn ở index.**
///
/// # Vì sao khẳng định cả hai vế
///
/// Chỉ khẳng định "khối giữa không còn ở index" thì một cài đặt unstage **cả tệp** vẫn
/// xanh — cả tệp cũng không còn khối giữa. Con số thứ hai (index còn đúng 2 khối) là
/// thứ phân biệt hai cài đặt. Cùng lớp lỗi với Test A.
#[tokio::test]
async fn unstage_mot_khoi_go_dung_khoi_do_khoi_index() {
    let (dir, state, repo) = repo_ba_hunk();
    let p = dir.path();

    // Stage cả tệp trước.
    git(p, &["add", "--", "f.txt"]);

    let cached_truoc = diff_tho(p, &["diff", "--cached", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&cached_truoc),
        3,
        "🔴 tiền đề sai: sau `git add` cả tệp, index phải có đúng 3 khối. Đọc được:\n{}",
        String::from_utf8_lossy(&cached_truoc)
    );

    let hash = bam_blob(p, "f.txt");
    unstage_mot_khoi(&state, Arc::clone(&repo), "f.txt", 1, &hash)
        .await
        .expect("unstage khối giữa phải Ok");

    let cached_sau = diff_tho(p, &["diff", "--cached", "--unified=3", "--", "f.txt"]);
    let cached_sau_str = String::from_utf8_lossy(&cached_sau);

    assert_eq!(
        dem_hunk(&cached_sau),
        2,
        "🔴 index phải còn đúng HAI khối. Số 0 nghĩa là unstage cả tệp — một cài đặt \
         như vậy vẫn xanh nếu chỉ kiểm 'khối giữa đã biến mất'. Đọc được:\n{cached_sau_str}"
    );
    assert!(
        !cached_sau_str.contains("CHANGED15"),
        "🔴 khối GIỮA phải rời khỏi index. Đọc được:\n{cached_sau_str}"
    );
    assert!(
        cached_sau_str.contains("M1") && cached_sau_str.contains("M3"),
        "🔴 khối đầu (`M1`) và khối cuối (`M3`) phải CÒN ở index — chỉ đúng một khối \
         được gỡ. Đọc được:\n{cached_sau_str}"
    );

    // Thay đổi không biến mất: nó quay về thư mục làm việc, không bị xoá.
    let worktree = std::fs::read(p.join("f.txt")).unwrap();
    assert!(
        String::from_utf8_lossy(&worktree).contains("CHANGED15"),
        "🔴 unstage KHÔNG được xoá thay đổi khỏi thư mục làm việc — `--cached` nghĩa \
         là chỉ index đổi. Mất nó ở đây là mất việc của người dùng"
    );
}

// ---------------------------------------------------------------------------
// Test G — hash rút gọn KHÔNG được coi là khớp (đột biến M11)
// ---------------------------------------------------------------------------

/// 🔴 **`kiem_blob_hash` so BẰNG, không so tiền tố.**
///
/// # Vì sao test này tồn tại — lỗi #9 của CONTEXT.md 4.1
///
/// Đột biến M11 (`starts_with` thay `==`) được **dự đoán** là không đỏ, vì hai hash đủ
/// dài hiếm khi khớp tiền tố. Nhưng *"trước khi kết luận một đột biến sống sót vì mã
/// đúng, kiểm xem có test nào HỎI về thứ đó không"* — và không có. Lỗi #9 sống sót qua
/// 666 test đúng vì điều này.
///
/// Nên đây là test **hỏi về thuộc tính đó** thay vì một dòng "chấp nhận được" trong
/// SUMMARY: nó gửi một hash **cắt ngắn** (tiền tố thật sự của hash đúng), thứ mà
/// `starts_with` sẽ chấp nhận và `==` sẽ từ chối.
///
/// Ca này không phải giả định: git in hash **rút gọn** ở khắp nơi, và trên một repo lớn
/// một tiền tố khớp với một blob khác là ca đã biết — đó chính là lý do git tự nới số ký
/// tự rút gọn theo kích thước repo.
#[tokio::test]
async fn hash_rut_gon_khong_duoc_coi_la_khop() {
    let (dir, state, repo) = repo_ba_hunk();
    let p = dir.path();

    let day_du = bam_blob(p, "f.txt");
    assert_eq!(
        day_du.len(),
        40,
        "🔴 tiền đề sai: hash đầy đủ phải là 40 ký tự hex (SHA-1). Đọc được {:?}",
        day_du
    );

    let rut_gon = &day_du[..7];
    assert!(
        day_du.starts_with(rut_gon),
        "🔴 tiền đề sai: hash cắt ngắn phải là TIỀN TỐ THẬT của hash đầy đủ, nếu không \
         `starts_with` cũng từ chối nó và test không phân biệt được gì"
    );

    let loi = stage_mot_khoi(&state, Arc::clone(&repo), "f.txt", 1, rut_gon)
        .await
        .expect_err(
            "🔴 hash RÚT GỌN phải bị từ chối. `starts_with` sẽ chấp nhận nó; `==` từ \
             chối. Một phép so tiền tố biến 'tệp đã đổi' thành 'tệp không đổi' đúng \
             lúc nó nguy hiểm nhất — trên repo lớn, một tiền tố khớp với một blob KHÁC",
        );

    assert_eq!(
        loi.code(),
        "file_changed",
        "hash không so bằng được thì đó là ca tệp-đã-đổi, đọc được `{}`",
        loi.code()
    );

    let cached = diff_tho(p, &["diff", "--cached", "--", "f.txt"]);
    assert!(
        cached.is_empty(),
        "🔴 từ chối hash rút gọn phải để lại index SẠCH. Đọc được:\n{}",
        String::from_utf8_lossy(&cached)
    );
}
