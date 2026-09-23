//! Test tích hợp cho an toàn khi huỷ — plan 05-03, WORK-06 và WORK-07.
//!
//! # Vì sao chạy git THẬT
//!
//! Câu hỏi của wave này là *"nội dung bị huỷ có còn lấy lại được không"*, và chỉ git
//! thật trả lời được — cụ thể là **Test D**, thứ chạy `git gc --prune=now` và hỏi object
//! có sống không. Không buffer tự dựng nào mô phỏng được phép dọn rác của git.
//!
//! # 🔴 `git gc` CHỈ chạy trong repo `tempfile`
//!
//! Không bao giờ trong repo dự án, và không bao giờ trong
//! `D:/MyCompanyProjects/quanly-truong-phong-so` (repo thật của chủ dự án, chỉ-đọc).
//! Mỗi test dựng repo riêng bằng `tempfile::tempdir()`, và mọi lệnh đều nhận `dir.path()`
//! tường minh làm thư mục làm việc.
//!
//! # 🔴 Không test nào ở đây render một component
//!
//! Tầng Rust của Phase 5 phải **tự đứng được** (CONTEXT.md mục 0.1).

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use git_plum_lib::commands::trash::{
    danh_sach_thung_rac, huy_hunk, huy_tep, khoi_phuc, luu_truoc_khi_huy,
};
use git_plum_lib::state::{AppState, RepoHandle};

/// Chạy một lệnh git đồng bộ trong lúc **dựng bối cảnh**. Không đi qua `GitRunner` có
/// chủ ý: đây là phần dựng bối cảnh, không phải phần được kiểm.
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
/// 🔴 Hai luồng tách riêng, không ghép — cùng lý do với `tests/hunk_commands.rs`: bắt
/// đầu ra bằng `$(...)` trong shell làm cảnh báo clean filter rơi vào cùng luồng với nội
/// dung thật.
fn git_thu(dir: &Path, args: &[&str]) -> (i32, Vec<u8>, Vec<u8>) {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("không chạy được git {args:?}: {e}"));
    (out.status.code().unwrap_or(-1), out.stdout, out.stderr)
}

/// `git diff` dạng byte thô.
fn diff_tho(dir: &Path, args: &[&str]) -> Vec<u8> {
    let (ma, stdout, stderr) = git_thu(dir, args);
    assert!(
        ma == 0 || ma == 1,
        "git {args:?} thoát {ma}: {}",
        String::from_utf8_lossy(&stderr)
    );
    stdout
}

/// Số hunk trong một bản vá: đếm dòng **bắt đầu** bằng `@@`.
///
/// 🔴 `starts_with`, không phải "có chứa" — một dòng **nội dung** hợp lệ có thể là
/// ` @@ ...` (dấu cách ngữ cảnh + văn bản), và đếm nó thành header làm con số nói dối.
fn dem_hunk(patch: &[u8]) -> usize {
    patch
        .split(|b| *b == b'\n')
        .filter(|d| d.starts_with(b"@@"))
        .count()
}

/// Mã băm blob hiện tại của một tệp — thứ giao diện gửi kèm khi bấm huỷ khối.
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

/// Repo tạm rỗng, đã cấu hình. `core.autocrlf=false` để một phép chuyển đổi CRLF không
/// làm `git status` báo tệp sửa ngoài ý muốn (Test E đặt lại tường minh nếu cần).
fn repo_tam() -> (tempfile::TempDir, AppState, Arc<RepoHandle>) {
    let dir = tempfile::tempdir().expect("dựng được thư mục tạm");
    let p = dir.path();

    git(p, &["init", "--initial-branch=main"]);
    git(p, &["config", "user.name", "Test"]);
    git(p, &["config", "user.email", "test@example.com"]);
    git(p, &["config", "core.autocrlf", "false"]);

    let state = AppState::new();
    let repo = state.open_repo(p);
    (dir, state, repo)
}

// ---------------------------------------------------------------------------
// Test A — vòng huỷ → khôi phục, tệp ĐÃ THEO DÕI
// ---------------------------------------------------------------------------

/// 🔴 **Khẳng định CẢ HAI vế: nội dung mất đi, rồi quay lại.**
///
/// Chỉ khẳng định vế khôi phục thì một cài đặt **không huỷ gì cả** cũng xanh — nội dung
/// "vẫn đúng" vì nó chưa bao giờ bị đụng tới. Vế thứ nhất (nội dung ĐÃ mất trên đĩa) là
/// thứ phân biệt "huỷ rồi khôi phục" với "không làm gì".
#[tokio::test]
async fn vong_huy_khoi_phuc_tep_da_theo_doi() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("a.txt"), b"one\ntwo\nthree\n").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "base"]);

    let da_sua: &[u8] = b"one\nCHANGED\nthree\n";
    std::fs::write(p.join("a.txt"), da_sua).unwrap();

    // --- Tiền đề: tệp thật sự bẩn -----------------------------------------
    let (_, st, _) = git_thu(p, &["status", "--porcelain", "--", "a.txt"]);
    assert!(
        !st.is_empty(),
        "🔴 tiền đề sai: `a.txt` phải bẩn trước khi huỷ, nếu không thì test không đo \
         được gì. `git status --porcelain` trả rỗng"
    );

    let paths = vec!["a.txt".to_owned()];
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths)
        .await
        .expect("lưu trước khi huỷ phải Ok");
    let ref_name = bien_nhan.muc().ref_name.clone();
    assert!(
        !bien_nhan.muc().la_blob,
        "🔴 tệp đã theo dõi phải lưu qua `stash create` (một COMMIT), không qua blob"
    );

    huy_tep(&state, Arc::clone(&repo), &paths, bien_nhan)
        .await
        .expect("huỷ phải Ok");

    // --- Vế 1: nội dung ĐÃ MẤT trên đĩa ------------------------------------
    let sau_huy = std::fs::read(p.join("a.txt")).unwrap();
    assert_ne!(
        sau_huy, da_sua,
        "🔴 nội dung vẫn còn sau khi huỷ — lệnh huỷ không làm gì. Không có vế này thì \
         một cài đặt rỗng cũng xanh ở vế khôi phục"
    );
    assert_eq!(
        sau_huy, b"one\ntwo\nthree\n",
        "🔴 sau khi huỷ, tệp phải về đúng nội dung của HEAD"
    );

    // --- Vế 2: khôi phục lấy lại ĐÚNG byte cũ ------------------------------
    khoi_phuc(&state, Arc::clone(&repo), &ref_name, &paths)
        .await
        .expect("khôi phục phải Ok");

    let sau_kp = std::fs::read(p.join("a.txt")).unwrap();
    assert_eq!(
        sau_kp, da_sua,
        "🔴 khôi phục phải trả lại ĐÚNG byte đã bị huỷ. Đây là cả lý do WORK-07 tồn tại"
    );
}

// ---------------------------------------------------------------------------
// Test B — vòng huỷ → khôi phục, tệp CHƯA THEO DÕI
// ---------------------------------------------------------------------------

/// 🔴 **Ca mà `git stash create` im lặng bỏ qua.**
///
/// Fixture có **duy nhất** một tệp chưa theo dõi thay đổi. Điều đó là bắt buộc: nếu repo
/// còn một tệp đã theo dõi bẩn thì `stash create` trả một sha khác rỗng và ca này
/// **không phân biệt được gì** — đúng lớp lỗi "hình dạng đúng, dữ liệu vô hại"
/// (CONTEXT.md 4.2). Test khẳng định tiền đề đó bằng `git status --porcelain` trước.
///
/// Đột biến M15 (bỏ nhánh `hash-object -w`) phải **đỏ ở đây**.
#[tokio::test]
async fn vong_huy_khoi_phuc_tep_chua_theo_doi() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("base.txt"), b"base\n").unwrap();
    git(p, &["add", "base.txt"]);
    git(p, &["commit", "-m", "base"]);

    let noi_dung: &[u8] = b"untracked content\n";
    std::fs::write(p.join("u.txt"), noi_dung).unwrap();

    // --- 🔴 Tiền đề: DUY NHẤT `u.txt` bẩn, và nó chưa theo dõi -------------
    let (_, st, _) = git_thu(p, &["status", "--porcelain"]);
    let st = String::from_utf8_lossy(&st);
    let dong: Vec<&str> = st.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        dong.len(),
        1,
        "🔴 tiền đề sai: phải có ĐÚNG MỘT mục bẩn. Một tệp đã theo dõi bẩn làm \
         `stash create` trả sha khác rỗng, và ca này mất hết khả năng phân biệt — \
         nhánh `hash-object` sẽ không bao giờ được đo. status đọc được:\n[{st}]"
    );
    assert!(
        dong[0].starts_with("??"),
        "🔴 tiền đề sai: mục bẩn duy nhất phải là tệp CHƯA THEO DÕI (`??`). \
         Đọc được: [{}]",
        dong[0]
    );

    // --- 🔴 Và khẳng định `stash create` THẬT SỰ trả rỗng ở fixture này ----
    //
    // Đây là phép chứng minh fixture phân biệt được, đo **bên trong test** thay vì tin
    // vào một phép đo ghi trong tài liệu. Nếu git đổi hành vi, test này nói ngay.
    let (_, sha_out, _) = git_thu(p, &["stash", "create", "probe"]);
    assert!(
        String::from_utf8_lossy(&sha_out).trim().is_empty(),
        "🔴 tiền đề sai: `git stash create` trả sha KHÁC rỗng ở fixture chỉ-có-tệp-chưa\
         -theo-dõi. Nếu git đổi hành vi thì nhánh `hash-object` không còn là đường duy \
         nhất giữ được tệp này, và test không còn đo thứ nó định đo. Đọc được: [{}]",
        String::from_utf8_lossy(&sha_out).trim()
    );

    let paths = vec!["u.txt".to_owned()];
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths)
        .await
        .expect(
            "🔴 lưu tệp chưa theo dõi phải Ok. `stash create` trả rỗng ở đây, nên một \
             cài đặt chỉ dùng stash sẽ thất bại — đó là điểm của test này",
        );
    let ref_name = bien_nhan.muc().ref_name.clone();
    assert!(
        bien_nhan.muc().la_blob,
        "🔴 tệp chưa theo dõi phải lưu thành BLOB qua `hash-object -w`"
    );

    // Nội dung trong object đúng byte.
    let (ma, ra, _) = git_thu(p, &["cat-file", "-p", &ref_name]);
    assert_eq!(ma, 0, "🔴 `cat-file -p <ref>` phải đọc được object đã lưu");
    assert_eq!(
        ra, noi_dung,
        "🔴 blob phải giữ ĐÚNG byte của tệp chưa theo dõi"
    );

    huy_tep(&state, Arc::clone(&repo), &paths, bien_nhan)
        .await
        .expect("huỷ phải Ok");

    // --- Vế 1: tệp KHÔNG CÒN trên đĩa --------------------------------------
    assert!(
        !p.join("u.txt").exists(),
        "🔴 tệp chưa theo dõi phải bị XOÁ khỏi đĩa — git không quản nó, nên không lệnh \
         git nào gỡ được; đường huỷ phải tự xoá"
    );

    // --- Vế 2: khôi phục đưa nó TRỞ LẠI ------------------------------------
    khoi_phuc(&state, Arc::clone(&repo), &ref_name, &paths)
        .await
        .expect("khôi phục phải Ok");

    assert!(
        p.join("u.txt").exists(),
        "🔴 khôi phục phải đưa tệp trở lại đĩa"
    );
    assert_eq!(
        std::fs::read(p.join("u.txt")).unwrap(),
        noi_dung,
        "🔴 khôi phục phải cho ĐÚNG byte cũ"
    );
}

// ---------------------------------------------------------------------------
// Test C — huỷ theo KHỐI
// ---------------------------------------------------------------------------

/// 🔴 **Huỷ đúng một khối, hai khối kia còn nguyên.**
///
/// Fixture ba hunk cách xa nhau (dòng 2/13/25): với `-U3`, hai thay đổi cách nhau dưới 7
/// dòng bị git **gộp** thành một hunk, và một fixture "đúng hình dạng" mà git gộp thành
/// một hunk là fixture vô hại. Test khẳng định số hunk **trước** khi tin vào chỉ số.
///
/// Đột biến M16 (bỏ `--reverse`) phải **đỏ ở đây**.
#[tokio::test]
async fn huy_mot_khoi_giu_nguyen_hai_khoi_kia() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    let goc: Vec<String> = (b'a'..=b'z').map(|c| (c as char).to_string()).collect();
    std::fs::write(p.join("f.txt"), format!("{}\n", goc.join("\n"))).unwrap();
    git(p, &["add", "f.txt"]);
    git(p, &["commit", "-m", "base"]);

    let mut sua = goc.clone();
    sua[1] = "M1".to_owned();
    sua[12] = "CHANGED15".to_owned();
    sua[24] = "M3".to_owned();
    std::fs::write(p.join("f.txt"), format!("{}\n", sua.join("\n"))).unwrap();

    // --- Tiền đề: ĐÚNG ba hunk --------------------------------------------
    let truoc = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&truoc),
        3,
        "🔴 tiền đề sai: fixture phải cho đúng 3 hunk, nếu không `hunk_index=1` không \
         có nghĩa là 'khối giữa'. Bản vá đọc được:\n{}",
        String::from_utf8_lossy(&truoc)
    );

    let hash = bam_blob(p, "f.txt");
    let paths = vec!["f.txt".to_owned()];

    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths)
        .await
        .expect("lưu phải Ok");
    let ref_name = bien_nhan.muc().ref_name.clone();

    huy_hunk(&state, Arc::clone(&repo), "f.txt", 1, &hash, bien_nhan)
        .await
        .expect("huỷ khối giữa phải Ok");

    // --- Khối GIỮA mất, hai khối kia CÒN -----------------------------------
    let sau = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&sau),
        2,
        "🔴 sau khi huỷ một khối, thư mục làm việc phải còn ĐÚNG HAI khối. Nhiều hơn \
         nghĩa là không huỷ được gì; ít hơn nghĩa là huỷ quá tay. Đọc được:\n{}",
        String::from_utf8_lossy(&sau)
    );

    let tren_dia = std::fs::read(p.join("f.txt")).unwrap();
    let tren_dia_s = String::from_utf8_lossy(&tren_dia);
    assert!(
        !tren_dia_s.contains("CHANGED15"),
        "🔴 khối GIỮA phải biến mất khỏi tệp. Còn `CHANGED15` nghĩa là huỷ sai khối \
         hoặc không huỷ gì — một cài đặt huỷ khối 0 vẫn cho 'còn 2 khối'"
    );
    assert!(
        tren_dia_s.contains("M1") && tren_dia_s.contains("M3"),
        "🔴 hai khối KIA phải còn nguyên. Mất chúng nghĩa là bản vá áp cả tệp thay vì \
         một khối"
    );

    // --- Khôi phục: đủ lại 3 khối ------------------------------------------
    khoi_phuc(&state, Arc::clone(&repo), &ref_name, &paths)
        .await
        .expect("khôi phục phải Ok");

    let sau_kp = diff_tho(p, &["diff", "--unified=3", "--", "f.txt"]);
    assert_eq!(
        dem_hunk(&sau_kp),
        3,
        "🔴 khôi phục phải trả lại đủ BA khối. Đọc được:\n{}",
        String::from_utf8_lossy(&sau_kp)
    );
    assert!(
        String::from_utf8_lossy(&std::fs::read(p.join("f.txt")).unwrap()).contains("CHANGED15"),
        "🔴 khối vừa huỷ phải quay lại sau khôi phục"
    );

    // --- 🔴 Và nội dung quay lại ở trạng thái CHƯA STAGE --------------------
    //
    // Đo được: `git checkout <ref> -- <path>` ghi vào **cả** index, nên không có bước
    // gỡ index thì người dùng nhận nội dung ở trạng thái đã stage — khác trạng thái họ
    // ở trước khi bấm huỷ, và lần commit kế tiếp sẽ lặng lẽ mang nó theo.
    //
    // Khẳng định này cũng là thứ làm phép đếm hunk ở trên CÓ NGHĨA: với index bẩn,
    // `git diff` so worktree với **index** và trả 0 hunk dù nội dung đã về đúng.
    let (_, st_sau, _) = git_thu(p, &["status", "--porcelain", "--", "f.txt"]);
    let st_sau = String::from_utf8_lossy(&st_sau);
    assert!(
        st_sau.starts_with(" M"),
        "🔴 sau khôi phục, tệp phải ở trạng thái CHƯA STAGE (` M`). Đọc được: \
         [{}] — `M ` nghĩa là khôi phục đã stage hộ người dùng một thay đổi họ không \
         yêu cầu, và lần commit kế tiếp sẽ mang nó theo trong im lặng",
        st_sau.trim_end()
    );
}

// ---------------------------------------------------------------------------
// Test D — 🔴 object SỐNG qua `git gc --prune=now`
// ---------------------------------------------------------------------------

/// 🔴 **Đây là test chứng minh "tới được", không chỉ "đã tạo".**
///
/// # Vì sao test này không thừa, đo được chứ không suy luận
///
/// `git stash create` tạo một commit **không ref nào trỏ tới**. Nó vẫn đọc được ngay sau
/// đó, nên Test A **xanh** kể cả khi `update-ref` bị bỏ hoàn toàn — object còn nằm trong
/// kho cho tới lần dọn rác kế tiếp. Đo được trên git 2.54.0.windows.1:
///
/// ```text
/// stash create, KHÔNG update-ref, `git gc --prune=now --aggressive`:
///   git cat-file -e <sha>  →  exit 1     ← ĐÃ MẤT
/// ```
///
/// Tức đột biến M14 (bỏ `update-ref`) **biểu diễn được** ở đây và **chỉ** ở đây. Một
/// object không tới được là một object cách `git gc` đúng một bước, và `git gc` chạy
/// **tự động** — người dùng không cần gõ gì.
///
/// # 🔴 `git gc` chạy trong repo `tempfile`, không bao giờ trong repo dự án
#[tokio::test]
async fn object_thung_rac_song_qua_git_gc_prune_now() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("a.txt"), b"one\ntwo\nthree\n").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "base"]);

    let da_sua: &[u8] = b"one\nCHANGED\nthree\n";
    std::fs::write(p.join("a.txt"), da_sua).unwrap();

    let paths = vec!["a.txt".to_owned()];
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths)
        .await
        .expect("lưu phải Ok");
    let ref_name = bien_nhan.muc().ref_name.clone();
    let object_id = bien_nhan.muc().object_id.clone();

    huy_tep(&state, Arc::clone(&repo), &paths, bien_nhan)
        .await
        .expect("huỷ phải Ok");

    // --- Tiền đề: ref THẬT SỰ tồn tại trước khi gc ------------------------
    //
    // Không có khẳng định này, một cài đặt không ghi ref nào sẽ làm `gc` dọn sạch và
    // test đỏ ở dưới **vì lý do đúng** — nhưng ta muốn biết nó đỏ ở bước nào.
    let (ma_rp, _, _) = git_thu(p, &["rev-parse", "--verify", &ref_name]);
    assert_eq!(
        ma_rp, 0,
        "🔴 tiền đề sai: ref `{ref_name}` không tồn tại ngay sau khi lưu. Bước \
         `update-ref` đã không chạy, hoặc nó ghi vào một tên khác"
    );

    // --- 🔴 DỌN RÁC THẬT ---------------------------------------------------
    //
    // `--prune=now` bỏ hết thời gian ân hạn: mọi object không tới được bị xoá ngay.
    // Đo được: một stash object KHÔNG có ref chết ở đúng lệnh này, kể cả khi không
    // chạy `reflog expire` trước.
    let (ma_gc, _, err_gc) = git_thu(p, &["gc", "--prune=now", "--aggressive"]);
    assert_eq!(
        ma_gc,
        0,
        "🔴 `git gc` phải chạy được, nếu không thì test không đo được gì: {}",
        String::from_utf8_lossy(&err_gc)
    );

    // --- Object còn sống ---------------------------------------------------
    //
    // `cat-file -e` là phép hỏi TRỰC TIẾP "object này còn không" — nó trả exit code,
    // không trả văn bản, nên không phụ thuộc vào câu chữ thông báo lỗi của git.
    let (ma_e, _, _) = git_thu(p, &["cat-file", "-e", &object_id]);
    assert_eq!(
        ma_e, 0,
        "🔴 object `{object_id}` ĐÃ BỊ DỌN bởi `git gc --prune=now`. Nó được tạo nhưng \
         không có ref nào trỏ tới — tức nội dung người dùng vừa huỷ đã MẤT VĨNH VIỄN. \
         Đây chính xác là thất bại mà WORK-07 tồn tại để ngăn, và là lý do bước \
         `git update-ref` không bỏ được"
    );

    // --- Và nội dung vẫn đọc được QUA REF ----------------------------------
    let (ma_show, ra_show, err_show) = git_thu(p, &["show", &format!("{ref_name}:a.txt")]);
    assert_eq!(
        ma_show,
        0,
        "🔴 `git show {ref_name}:a.txt` thất bại sau gc: {}",
        String::from_utf8_lossy(&err_show)
    );
    assert_eq!(
        ra_show, da_sua,
        "🔴 nội dung đọc qua ref thùng rác phải đúng byte đã huỷ, kể cả sau khi dọn rác"
    );

    // --- Và khôi phục vẫn chạy được sau gc ---------------------------------
    khoi_phuc(&state, Arc::clone(&repo), &ref_name, &paths)
        .await
        .expect("khôi phục phải Ok kể cả sau `git gc --prune=now`");
    assert_eq!(
        std::fs::read(p.join("a.txt")).unwrap(),
        da_sua,
        "🔴 khôi phục sau gc phải cho đúng byte cũ"
    );
}

// ---------------------------------------------------------------------------
// Test E — byte KHÔNG UTF-8 qua vòng khôi phục
// ---------------------------------------------------------------------------

/// 🔴 **So byte với byte, không bao giờ `read_to_string`.**
///
/// `String::from_utf8_lossy` đổi mỗi byte không hợp lệ thành U+FFFD — **ba** byte
/// (`EF BF BD`) — nên một vòng huỷ→khôi phục đi qua `String` làm tệp **dài ra** và nội
/// dung sai. Một tệp Latin-1 là dữ liệu có thật trên máy người dùng.
///
/// Đột biến M17 (`khoi_phuc` đọc qua `String::from_utf8_lossy`) phải **đỏ ở đây**.
#[tokio::test]
async fn byte_khong_utf8_giu_nguyen_qua_vong_khoi_phuc() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("base.txt"), b"base\n").unwrap();
    git(p, &["add", "base.txt"]);
    git(p, &["commit", "-m", "base"]);

    // `0xE9` là `é` trong Latin-1 và là byte **không hợp lệ** trong UTF-8.
    let noi_dung: &[u8] = b"caf\xE9 latin-1\n\xFF\xFE binary-ish\n";
    std::fs::write(p.join("l1.txt"), noi_dung).unwrap();

    // --- Tiền đề: nội dung THẬT SỰ không phải UTF-8 hợp lệ -----------------
    //
    // Không có khẳng định này, một fixture toàn ASCII trông đúng hình dạng nhưng
    // `from_utf8_lossy` sẽ là phép đồng nhất trên nó — đột biến M17 thành **vô hiệu**
    // và test xanh mà không đo gì (CONTEXT.md 4.2).
    assert!(
        String::from_utf8(noi_dung.to_vec()).is_err(),
        "🔴 tiền đề sai: fixture phải chứa byte KHÔNG hợp lệ trong UTF-8. Với nội dung \
         toàn ASCII, `from_utf8_lossy` là phép đồng nhất và đột biến M17 không biểu \
         diễn được — test sẽ XANH với cả mã sai"
    );

    let paths = vec!["l1.txt".to_owned()];
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths)
        .await
        .expect("lưu phải Ok");
    let ref_name = bien_nhan.muc().ref_name.clone();

    huy_tep(&state, Arc::clone(&repo), &paths, bien_nhan)
        .await
        .expect("huỷ phải Ok");
    assert!(
        !p.join("l1.txt").exists(),
        "🔴 tệp chưa theo dõi phải bị xoá khỏi đĩa"
    );

    khoi_phuc(&state, Arc::clone(&repo), &ref_name, &paths)
        .await
        .expect("khôi phục phải Ok");

    // --- 🔴 `std::fs::read`, KHÔNG `read_to_string` ------------------------
    let sau = std::fs::read(p.join("l1.txt")).unwrap();
    assert_eq!(
        sau,
        noi_dung,
        "🔴 byte không UTF-8 bị đổi qua vòng khôi phục. `from_utf8_lossy` đổi mỗi byte \
         không hợp lệ thành U+FFFD (EF BF BD) — BA byte — nên tệp dài ra và nội dung \
         sai. Độ dài: mong {} nhận {}",
        noi_dung.len(),
        sau.len()
    );
}

// ---------------------------------------------------------------------------
// Test F — ca HỖN HỢP: stash trả sha khác rỗng mà tệp chưa theo dõi vẫn phải sống
// ---------------------------------------------------------------------------

/// 🔴 **Ca mà plan không lường, và là ca mất dữ liệu im lặng nhất.**
///
/// Plan mô tả phép phân nhánh là *"`stash create` trả rỗng ⇒ ca chưa theo dõi"*. Đo được
/// hôm nay cho thấy điều đó **không đủ**:
///
/// ```text
/// repo có CẢ tệp đã theo dõi bẩn LẪN tệp chưa theo dõi:
///   git stash create      → 356fea29...        (sha KHÁC rỗng)
///   git ls-tree -r 356fea29 → chỉ có a.txt     (u.txt KHÔNG trong cây)
///   git show 356fea29:u.txt → fatal: không có trong cây
/// ```
///
/// Một cài đặt phân nhánh theo "stdout có rỗng không" sẽ đi nhánh stash ở đây, thấy một
/// sha hợp lệ, ghi ref, trả biên nhận — rồi xoá tệp chưa theo dõi mà **không có bản lưu
/// nào**. Không thông báo, không lỗi. Đó là đúng dữ liệu WORK-07 tồn tại để giữ.
#[tokio::test]
async fn ca_hon_hop_tep_chua_theo_doi_van_duoc_luu() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("a.txt"), b"one\ntwo\nthree\n").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "base"]);

    // Tệp đã theo dõi BẨN — thứ làm `stash create` trả sha khác rỗng.
    std::fs::write(p.join("a.txt"), b"one\nCHANGED\nthree\n").unwrap();
    // Và một tệp CHƯA THEO DÕI cùng lúc.
    let nd_u: &[u8] = b"untracked content\n";
    std::fs::write(p.join("u.txt"), nd_u).unwrap();

    // --- 🔴 Tiền đề đo được: stash create trả sha KHÁC rỗng ở fixture này --
    let (_, sha_out, _) = git_thu(p, &["stash", "create", "probe"]);
    let sha = String::from_utf8_lossy(&sha_out).trim().to_owned();
    assert!(
        !sha.is_empty(),
        "🔴 tiền đề sai: fixture hỗn hợp phải làm `stash create` trả sha KHÁC rỗng. \
         Nếu nó rỗng thì ca này trùng với Test B và không đo thêm gì"
    );

    // --- 🔴 Và tiền đề thứ hai: sha đó KHÔNG chứa tệp chưa theo dõi --------
    let (ma_show, _, _) = git_thu(p, &["show", &format!("{sha}:u.txt")]);
    assert_ne!(
        ma_show, 0,
        "🔴 tiền đề sai: cây của stash commit LẠI chứa `u.txt`. Nếu git đổi hành vi thì \
         nhánh `hash-object` không còn cần thiết cho ca này và test không đo thứ nó định \
         đo — đọc lại phép đo trước khi sửa mã"
    );

    let paths = vec!["a.txt".to_owned(), "u.txt".to_owned()];
    let bien_nhan = luu_truoc_khi_huy(&state, Arc::clone(&repo), &paths)
        .await
        .expect("lưu ca hỗn hợp phải Ok");
    drop(bien_nhan);

    // --- Cả HAI object phải có mặt trong thùng rác -------------------------
    let ds = danh_sach_thung_rac(&state, Arc::clone(&repo))
        .await
        .expect("liệt kê thùng rác phải Ok");

    assert!(
        ds.len() >= 2,
        "🔴 ca hỗn hợp phải lưu HAI object: một stash commit cho `a.txt` và một blob \
         cho `u.txt`. Chỉ thấy {} mục — tệp chưa theo dõi đã bị bỏ qua trong im lặng, \
         tức xoá nó sẽ mất vĩnh viễn",
        ds.len()
    );

    let co_blob = ds.iter().any(|m| m.la_blob);
    assert!(
        co_blob,
        "🔴 không có mục BLOB nào. Tệp chưa theo dõi `u.txt` không được lưu, dù \
         `stash create` trả một sha trông hợp lệ — đây là ca mất dữ liệu im lặng"
    );

    // Và blob đó thật sự giữ đúng nội dung.
    let blob = ds.iter().find(|m| m.la_blob).expect("vừa khẳng định có");
    let (ma, ra, _) = git_thu(p, &["cat-file", "-p", &blob.ref_name]);
    assert_eq!(ma, 0, "🔴 đọc được object blob đã lưu");
    assert_eq!(
        ra, nd_u,
        "🔴 blob phải giữ đúng byte của `u.txt`, không phải nội dung của tệp khác"
    );
}

// ---------------------------------------------------------------------------
// Test G — `restore_trash` từ chối ref ngoài không gian riêng (T-05-12)
// ---------------------------------------------------------------------------

/// 🔴 **`ref_name` đến từ webview, tức chuỗi tuỳ ý.**
///
/// Không có phép kiểm tiền tố, một `ref_name` bằng `refs/heads/main` biến lời gọi thành
/// `git checkout refs/heads/main -- <paths>`, tức ghi đè thư mục làm việc bằng nội dung
/// của một nhánh — trên một đường đi mà người dùng nghĩ là "khôi phục thứ tôi vừa huỷ".
#[tokio::test]
async fn khoi_phuc_tu_choi_ref_ngoai_khong_gian_thung_rac() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("a.txt"), b"HEAD version\n").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "base"]);

    let hien_tai: &[u8] = b"my work in progress\n";
    std::fs::write(p.join("a.txt"), hien_tai).unwrap();

    let paths = vec!["a.txt".to_owned()];

    for ten_xau in [
        "refs/heads/main",
        "HEAD",
        "refs/stash",
        // 🔴 Tiền tố *gần giống* — phép kiểm phải so cả dấu `/` cuối, nếu không thì
        // `refs/git-plum-trash-evil/x` lọt qua một phép `starts_with` cẩu thả.
        "refs/git-plum-trash-evil/x",
    ] {
        let ket_qua = khoi_phuc(&state, Arc::clone(&repo), ten_xau, &paths).await;
        assert!(
            ket_qua.is_err(),
            "🔴 `{ten_xau}` phải bị TỪ CHỐI — nó nằm ngoài không gian ref thùng rác. \
             Chấp nhận nó nghĩa là webview điều khiển được `git checkout <ref> -- .`, \
             tức ghi đè thư mục làm việc bằng nội dung tuỳ ý (T-05-12)"
        );
    }

    // --- Và tệp KHÔNG bị đụng tới ------------------------------------------
    //
    // Khẳng định này là thứ phân biệt "trả Err" với "trả Err SAU KHI đã ghi đè".
    assert_eq!(
        std::fs::read(p.join("a.txt")).unwrap(),
        hien_tai,
        "🔴 các lời gọi bị từ chối vẫn ghi đè tệp. Trả `Err` sau khi đã phá dữ liệu \
         không phải là từ chối"
    );
}

// ---------------------------------------------------------------------------
// Test H — không có gì để lưu ⇒ KHÔNG cấp biên nhận
// ---------------------------------------------------------------------------

/// Một tệp **sạch** không có gì để lưu, nên không có biên nhận nào được cấp.
///
/// # Vì sao đây là một khẳng định về an toàn, không phải về tiện lợi
///
/// Biên nhận là thứ **duy nhất** mở đường cho lệnh huỷ chạy. Cấp một biên nhận không giữ
/// gì nghĩa là ràng buộc kiểu vẫn thoả mãn về mặt biên dịch trong khi lời hứa nó mang
/// ("nội dung đã an toàn") là sai — đúng thứ ràng buộc này tồn tại để ngăn.
#[tokio::test]
async fn tep_sach_khong_duoc_cap_bien_nhan() {
    let (dir, state, repo) = repo_tam();
    let p = dir.path();

    std::fs::write(p.join("a.txt"), b"clean\n").unwrap();
    git(p, &["add", "a.txt"]);
    git(p, &["commit", "-m", "base"]);

    // Tiền đề: cây SẠCH.
    let (_, st, _) = git_thu(p, &["status", "--porcelain"]);
    assert!(
        String::from_utf8_lossy(&st).trim().is_empty(),
        "🔴 tiền đề sai: cây phải sạch. status đọc được: [{}]",
        String::from_utf8_lossy(&st)
    );

    let ket_qua = luu_truoc_khi_huy(&state, Arc::clone(&repo), &["a.txt".to_owned()]).await;
    assert!(
        ket_qua.is_err(),
        "🔴 tệp sạch không có nội dung nào để lưu, nên KHÔNG được cấp biên nhận. Một \
         biên nhận rỗng nghĩa vẫn mở đường cho lệnh huỷ chạy mà không có bản lưu nào"
    );
}
