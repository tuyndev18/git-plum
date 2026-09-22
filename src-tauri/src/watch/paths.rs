//! Đường dẫn nào trong `.git` đáng theo dõi — WORK-10.
//!
//! Tách **riêng** khỏi `mod.rs`, và đây là chủ ý: phép lọc là logic **thuần**
//! (`&Path -> bool`), test được mà không cần hệ tệp, không cần thời gian, không cần
//! watcher. Đó là phần mà một cổng có thể thật sự kiểm — phần còn lại của watcher
//! chỉ test được qua thời gian thật hoặc đồng hồ tiêm vào.
//!
//! # 🔴 Theo dõi HẸP, không đệ quy cả `.git`
//!
//! `.git/objects` có **hàng trăm nghìn** tệp trên repo lớn và đổi liên tục lúc git
//! ghi, mà không nói gì về trạng thái người dùng nhìn thấy. Thư mục làm việc còn tệ
//! hơn: `node_modules` ở repo này hàng **chục nghìn** tệp và `target/` hàng **trăm
//! nghìn**. Một watcher không lọc ăn hết CPU và vi phạm ràng buộc RAM < 150 MB của
//! `PROJECT.md` — đó là ràng buộc 2.2 của `CONTEXT.md`, không phải tối ưu.
//!
//! # ⚠️ Hai lớp phòng thủ, có chủ ý
//!
//! [`should_notify`] **vẫn** từ chối `objects/**` và thư mục làm việc kể cả khi
//! [`WATCHED`] không đăng ký theo dõi chúng. Lý do: `notify` trên Windows dùng
//! `ReadDirectoryChangesW` và **có thể** trả sự kiện cho thư mục cha khi ta đăng ký
//! theo dõi một thư mục. Đăng ký hẹp là lớp một; phép lọc là lớp hai. Khuôn
//! `isPerfEnabled` của `CommitList.tsx` đã dùng cách này.

use std::path::{Component, Path};

/// Các đường dẫn **tương đối trong `.git`** mà một thay đổi ở đó đổi thứ người dùng
/// nhìn thấy.
///
/// Danh sách này là **đầy đủ theo chủ ý** — thêm mục là một quyết định, không phải
/// một lần sửa cho tiện. Test khẳng định **từng tên một**, không chỉ đếm: đếm cho
/// phép đổi `packed-refs` thành `packed-ref` mà test vẫn xanh.
///
/// * `HEAD` — checkout, commit, reset đều chạm nó.
/// * `index` — mọi lần stage/unstage, và mọi lần `git add` ở terminal.
/// * `refs` — nhánh và tag **chưa gói**. Theo dõi **đệ quy** (xem [`WATCHED_RECURSIVE`]).
/// * `packed-refs` — nhánh và tag **đã gói**. 🔴 Thiếu mục này thì một `git gc` hoặc
///   một repo mới clone (mọi ref đều gói sẵn) không sinh sự kiện nào, và thanh bên
///   ref đứng im. Đột biến M2 ghim nó.
/// * `MERGE_HEAD`, `CHERRY_PICK_HEAD`, `REVERT_HEAD`, `BISECT_LOG` — trạng thái thao
///   tác đang dở. Chúng đổi cả câu hỏi "commit tới đây là commit gì".
/// * `rebase-merge`, `rebase-apply` — thư mục trạng thái rebase.
pub const WATCHED: &[&str] = &[
    "HEAD",
    "index",
    "refs",
    "packed-refs",
    "MERGE_HEAD",
    "CHERRY_PICK_HEAD",
    "REVERT_HEAD",
    "BISECT_LOG",
    "rebase-merge",
    "rebase-apply",
];

/// Những mục của [`WATCHED`] là **thư mục** và phải theo dõi **đệ quy**.
///
/// `refs/` là cây (`refs/heads/feature/x`), nên theo dõi không đệ quy chỉ thấy
/// `refs/heads` xuất hiện một lần rồi im. `rebase-merge`/`rebase-apply` cũng là thư
/// mục nhưng nông; theo dõi đệ quy chúng không tốn gì.
///
/// Mọi mục khác theo dõi **không đệ quy** — và `.git` gốc thì **không bao giờ** đệ
/// quy, vì đệ quy `.git` kéo cả `objects/` vào. Xem doc comment đầu tệp.
pub const WATCHED_RECURSIVE: &[&str] = &["refs", "rebase-merge", "rebase-apply"];

/// Những thư mục con của `.git` **luôn** bị từ chối, kể cả khi sự kiện lọt vào.
///
/// * `objects` — hàng trăm nghìn tệp, đổi liên tục lúc git ghi, và **không** nói gì
///   về trạng thái người dùng thấy. Một commit ghi object **trước** khi cập nhật
///   `refs`, nên nghe `objects` chỉ làm ta làm mới **sớm** và thấy trạng thái cũ.
/// * `logs` — reflog. Đổi theo mọi thứ khác nhưng không phải nguồn sự thật nào.
/// * `lfs`, `modules` — có thể rất lớn, và thay đổi ở đó đi kèm thay đổi ở `refs`.
const TU_CHOI_TRONG_GIT: &[&str] = &["objects", "logs", "lfs", "modules"];

/// Một thay đổi ở `path` có đáng làm mới trạng thái không?
///
/// Nhận đường dẫn **tuyệt đối hoặc tương đối bất kỳ** — hàm tự tìm đoạn `.git` trong
/// đó. Đường dẫn **không** đi qua `.git` là đường dẫn trong thư mục làm việc và bị
/// từ chối thẳng.
///
/// # Vì sao thư mục làm việc bị từ chối hoàn toàn
///
/// Nghe trực tiếp cây làm việc là cách chắc chắn nhất để vi phạm ràng buộc RAM và
/// CPU (`node_modules`, `target/`). Và nó **không cần thiết**: mọi thao tác git đổi
/// trạng thái đều chạm `.git/index` hoặc `.git/HEAD`. Ca duy nhất bỏ sót là "người
/// dùng sửa một tệp trong editor mà không chạy git" — ca đó `git status` vẫn bắt
/// được ở lần làm mới kế tiếp, và WORK-10 nói về **lệnh git chạy ở terminal ngoài**.
///
/// ⚠️ Đây là đánh đổi **có ý thức**, ghi ở đây để lần sau không ai "sửa" nó thành
/// theo dõi đệ quy cây làm việc.
pub fn should_notify(path: &Path) -> bool {
    let Some(sau_git) = phan_sau_git(path) else {
        // Không đi qua `.git` → thư mục làm việc → từ chối.
        return false;
    };

    // `.git` trần (sự kiện cho chính thư mục) — không có gì cụ thể để làm mới.
    let Some(dau) = sau_git.first() else {
        return false;
    };

    if TU_CHOI_TRONG_GIT.contains(&dau.as_str()) {
        return false;
    }

    // `.git/index.lock` và mọi `*.lock` khác: **không** làm mới.
    //
    // 🔴 Đây là nửa còn lại của ràng buộc 2.3 ("không bao giờ tự xoá `index.lock`").
    // Tệp lock xuất hiện rồi biến mất ở **mỗi** lệnh git, kể cả lệnh của chính ứng
    // dụng, nên nghe nó là tự nhân đôi số sự kiện cho mọi thao tác — và làm mới
    // **giữa lúc** một lệnh git khác đang ghi là đọc đúng trạng thái nửa vời.
    if let Some(cuoi) = sau_git.last() {
        if cuoi.ends_with(".lock") {
            return false;
        }
    }

    WATCHED.contains(&dau.as_str())
}

/// Các đoạn đường dẫn **sau** thư mục `.git`, hoặc `None` nếu không đi qua `.git`.
///
/// Dùng `Component::Normal` chứ không tách chuỗi theo `/`: trên Windows đường dẫn
/// dùng `\`, và `notify` trả đường dẫn theo quy ước hệ điều hành. Tách theo `/` ở đây
/// là đúng lớp lỗi "chuẩn hoá dấu gạch chéo ở sai tầng" mà `repo_id_for` đã phải xử
/// lý riêng.
fn phan_sau_git(path: &Path) -> Option<Vec<String>> {
    let mut sau: Option<Vec<String>> = None;
    for c in path.components() {
        let Component::Normal(ten) = c else { continue };
        let ten = ten.to_string_lossy();
        match sau {
            None => {
                // `.git` là thư mục thường; với worktree/submodule nó là **tệp** trỏ
                // đi nơi khác, nhưng `notify` khi đó theo dõi thư mục đích thật và
                // đường dẫn sự kiện vẫn chứa một đoạn `.git` của kho thật.
                if ten == ".git" {
                    sau = Some(Vec::new());
                }
            }
            Some(ref mut v) => v.push(ten.into_owned()),
        }
    }
    sau
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn p(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    /// 🔴 Đột biến M2: bỏ `packed-refs` khỏi [`WATCHED`].
    ///
    /// Khẳng định **từng tên một**, không chỉ đếm — đếm cho phép đổi `packed-refs`
    /// thành `packed-ref` mà test vẫn xanh.
    #[test]
    fn danh_sach_theo_doi_co_dung_tung_ten_mot() {
        for ten in [
            "HEAD",
            "index",
            "refs",
            "packed-refs",
            "MERGE_HEAD",
            "CHERRY_PICK_HEAD",
            "REVERT_HEAD",
            "BISECT_LOG",
            "rebase-merge",
            "rebase-apply",
        ] {
            assert!(
                WATCHED.contains(&ten),
                "`{ten}` phải nằm trong WATCHED — thiếu nó là một lớp thay đổi im lặng"
            );
        }
    }

    /// `packed-refs` có test riêng vì nó là mục dễ quên nhất: một repo vừa clone có
    /// **mọi** ref đã gói, nên thiếu nó thì thanh bên ref đứng im ở đúng repo mà
    /// người dùng vừa mở.
    #[test]
    fn packed_refs_duoc_theo_doi() {
        assert!(WATCHED.contains(&"packed-refs"));
        assert!(should_notify(&p("C:/work/repo/.git/packed-refs")));
    }

    #[test]
    fn thu_muc_de_quy_la_tap_con_cua_watched() {
        for ten in WATCHED_RECURSIVE {
            assert!(
                WATCHED.contains(ten),
                "`{ten}` theo dõi đệ quy nhưng không có trong WATCHED"
            );
        }
    }

    /// 🔴 Đột biến M1: `should_notify` trả `true` cho mọi đường dẫn.
    ///
    /// Ba đường dẫn thư mục làm việc, khẳng định `false` cho **cả ba**.
    #[test]
    fn tu_choi_duong_dan_trong_thu_muc_lam_viec() {
        for duong_dan in [
            "C:/work/repo/node_modules/x",
            "C:/work/repo/target/y",
            "C:/work/repo/src/main.rs",
        ] {
            assert!(
                !should_notify(&p(duong_dan)),
                "`{duong_dan}` nằm trong thư mục làm việc — theo dõi nó là hàng chục nghìn \
                 tệp và vi phạm ràng buộc RAM < 150MB"
            );
        }
    }

    /// `.git/objects/**` đổi liên tục lúc git ghi và không nói gì về trạng thái người
    /// dùng thấy. Bị từ chối kể cả khi sự kiện lọt vào từ thư mục cha (Windows).
    #[test]
    fn tu_choi_objects() {
        assert!(!should_notify(&p("C:/work/repo/.git/objects/ab/cdef")));
        assert!(!should_notify(&p("C:/work/repo/.git/objects/pack/pack-1.pack")));
        assert!(!should_notify(&p("C:/work/repo/.git/objects")));
    }

    #[test]
    fn tu_choi_logs_va_cac_thu_muc_lon_khac() {
        assert!(!should_notify(&p("C:/work/repo/.git/logs/HEAD")));
        assert!(!should_notify(&p("C:/work/repo/.git/lfs/objects/aa")));
        assert!(!should_notify(&p("C:/work/repo/.git/modules/sub/HEAD")));
    }

    /// Ba đường dẫn mà **một** lần `git commit` chạm — xem test gộp ở `mod.rs`.
    #[test]
    fn chap_nhan_ba_duong_dan_cua_mot_lan_commit() {
        assert!(should_notify(&p("C:/work/repo/.git/index")));
        assert!(should_notify(&p("C:/work/repo/.git/HEAD")));
        assert!(should_notify(&p("C:/work/repo/.git/refs/heads/main")));
    }

    #[test]
    fn chap_nhan_ref_long_nhieu_tang() {
        assert!(should_notify(&p("C:/work/repo/.git/refs/heads/feature/abc")));
        assert!(should_notify(&p("C:/work/repo/.git/refs/tags/v1.0")));
        assert!(should_notify(&p("C:/work/repo/.git/refs/remotes/origin/main")));
    }

    #[test]
    fn chap_nhan_tep_trang_thai_merge_rebase() {
        assert!(should_notify(&p("C:/work/repo/.git/MERGE_HEAD")));
        assert!(should_notify(&p("C:/work/repo/.git/CHERRY_PICK_HEAD")));
        assert!(should_notify(&p("C:/work/repo/.git/rebase-merge/done")));
        assert!(should_notify(&p("C:/work/repo/.git/rebase-apply/next")));
    }

    /// `*.lock` không làm mới: nó xuất hiện rồi biến mất ở **mỗi** lệnh git, và làm
    /// mới giữa lúc một lệnh git khác đang ghi là đọc trạng thái nửa vời.
    #[test]
    fn tu_choi_tep_lock() {
        assert!(!should_notify(&p("C:/work/repo/.git/index.lock")));
        assert!(!should_notify(&p("C:/work/repo/.git/HEAD.lock")));
        assert!(!should_notify(&p("C:/work/repo/.git/refs/heads/main.lock")));
    }

    /// `notify` trả đường dẫn theo quy ước hệ điều hành — trên Windows là `\`.
    /// Tách chuỗi theo `/` sẽ trượt hoàn toàn ở đây.
    #[test]
    fn hieu_dau_gach_cheo_nguoc_cua_windows() {
        assert!(should_notify(&p(r"C:\work\repo\.git\index")));
        assert!(should_notify(&p(r"C:\work\repo\.git\refs\heads\main")));
        assert!(!should_notify(&p(r"C:\work\repo\node_modules\x")));
        assert!(!should_notify(&p(r"C:\work\repo\.git\objects\ab\cd")));
    }

    /// Một tệp trong `.git` **không** nằm trong danh sách (ví dụ `config`,
    /// `COMMIT_EDITMSG`) không làm mới. `COMMIT_EDITMSG` đặc biệt đáng từ chối: git
    /// ghi nó ở **mỗi** lần commit, trước cả khi commit thành công.
    #[test]
    fn tu_choi_tep_git_khong_trong_danh_sach() {
        assert!(!should_notify(&p("C:/work/repo/.git/config")));
        assert!(!should_notify(&p("C:/work/repo/.git/COMMIT_EDITMSG")));
        assert!(!should_notify(&p("C:/work/repo/.git/description")));
    }

    #[test]
    fn duong_dan_khong_di_qua_git_bi_tu_choi() {
        assert!(!should_notify(&p("C:/work/repo")));
        assert!(!should_notify(&p("")));
        // Một thư mục **tên** `.gitignore` không phải `.git`.
        assert!(!should_notify(&p("C:/work/repo/.gitignore")));
    }
}
