//! Theo dõi `.git` và làm mới trạng thái tự động — WORK-10.
//!
//! # Vì sao watcher xuất hiện ở **phase này**, không sớm hơn
//!
//! Trước Phase 4 ứng dụng chỉ **đọc**; đọc dữ liệu cũ thì sai mắt nhìn. Từ Phase 4
//! ứng dụng **ghi**, và ghi dựa trên trạng thái cũ thì **hỏng repo của người dùng**.
//! Chủ dự án dùng terminal song song — đó là cả điểm của requirement này.
//!
//! # 🔴 R2 — vòng lặp ghi → sự kiện → đọc → ghi. Hai lớp phòng thủ.
//!
//! Thao tác ghi của chính ứng dụng cũng chạm `.git`, nên về nguyên tắc một sự kiện
//! của chính ta có thể sinh ra một lần làm mới sinh ra một thao tác sinh ra một sự
//! kiện. Hai thứ chặn nó:
//!
//! 1. **Thao tác ghi trả trạng thái trực tiếp** (đã có từ 04-02: `stage_files` trả
//!    `Result<RepoStatus>`, không phải `Result<()>`), nên đường chính **không** phụ
//!    thuộc watcher.
//! 2. **Sự kiện của watcher chỉ ĐỌC rồi phát.** Nó **không bao giờ** khởi tạo một
//!    thao tác ghi. Không có đường nào để sự kiện sinh ra sự kiện — đây là một bất
//!    biến về *cấu trúc*, không phải một phép kiểm lúc chạy.
//!
//! Phép gộp 250–300 ms cũng hấp thu sự kiện của chính ta trong đa số ca.
//!
//! # 🔴 Watcher TÍCH LUỸ, vì mở repo mới KHÔNG đóng repo cũ
//!
//! Kiểm chứng trong mã, không suy đoán:
//!
//! * `repoStore.openRepository` **gộp** vào `byRepo` và chỉ **gán lại** `activeRepoId`.
//!   Repo cũ ở lại.
//! * `closeRepository` **chỉ** chạy từ lệnh "Đóng repository" tường minh.
//! * Backend khớp: [`crate::state::AppState::open_repo`] dùng `or_insert_with`, nên mở
//!   lại **cùng** đường dẫn trả cùng handle — nhưng mở đường dẫn **khác** thì thêm mục
//!   mới và không bỏ mục cũ.
//!
//! **Hệ quả:** map watcher theo `RepoId` nghĩa là mở 5 repo trong một phiên = **5
//! watcher sống**, mỗi cái giữ handle `ReadDirectoryChangesW` cộng buffer cho mỗi
//! đường theo dõi trong `.git`. Đây **không** phải lỗi của thiết kế map — nó là hệ quả
//! của PLAT-05 và của việc PLAT-11 (nhiều repo theo thẻ) vừa được kéo lên v1, nên
//! nhiều repo mở cùng lúc là ca **thường**.
//!
//! Số đo RAM/handle ở 1 repo so với 5 repo nằm trong `04-04-SUMMARY.md`. Lựa chọn
//! giữa "watcher cho **mọi** repo mở" và "watcher chỉ cho repo **đang hoạt động**" là
//! một quyết định thiết kế thuộc chủ dự án, vì nó đụng thẳng WORK-10: repo **không**
//! hoạt động có được cập nhật khi git chạy ở terminal không? Bản này chọn **mọi repo
//! mở**, vì đó là hành vi mà WORK-10 mô tả nguyên văn.
//!
//! [`close_repo`](crate::state::AppState::close_repo) **thật sự** giải phóng watcher —
//! có test ghim. Nếu không, mở/đóng lặp lại là rò rỉ, và đó là ca **duy nhất** hiện có
//! đường gọi.

pub mod paths;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use notify::RecursiveMode;
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use parking_lot::Mutex;

use crate::state::RepoId;

pub use paths::should_notify;

/// Cửa sổ gộp-và-trì-hoãn.
///
/// 🔴 Ràng buộc ROADMAP: **250–300 ms**. Một lần `git commit` chạm `index`, `HEAD` và
/// `refs/heads/<nhánh>` — **ba** sự kiện cho **một** hành động, và ba lần làm mới
/// nghĩa là ba tiến trình `git status` cộng ba lần nhảy giao diện.
///
/// Test khẳng định **cả hai** biên, không chỉ biên dưới: một hằng số 5 ms cũng "gộp
/// được" (đột biến M3) mà vi phạm ràng buộc, và một hằng số 5000 ms (đột biến M4)
/// cũng "gộp được" mà làm giao diện trễ 5 giây sau mỗi lệnh terminal.
pub const CUA_SO_TRI_HOAN: Duration = Duration::from_millis(280);

/// Biên dưới và biên trên của ràng buộc ROADMAP. Công khai để test ghim được cả hai.
pub const TRI_HOAN_TOI_THIEU: Duration = Duration::from_millis(250);
/// Xem [`TRI_HOAN_TOI_THIEU`].
pub const TRI_HOAN_TOI_DA: Duration = Duration::from_millis(300);

// ---------------------------------------------------------------------------
// Phần lõi: phép gộp thuần, đồng hồ TIÊM VÀO
// ---------------------------------------------------------------------------

/// Phép gộp sự kiện, tách thành logic **thuần** nhận `Instant` tiêm vào.
///
/// # ⚠️ Vì sao tách ra thay vì test thẳng watcher
///
/// Test có thời gian là test bất đồng bộ, và test bất đồng bộ đỏ **lẻ tẻ** — đúng lớp
/// lỗi #6 của `CONTEXT.md` 3.1 (đỏ 1/5 lần và sống qua cả một wave vì suite chỉ chạy
/// một lần). Phần **quyết định** ("ba sự kiện này có gộp thành một không?") không cần
/// thời gian thật để đúng; nó chỉ cần biết hai mốc thời gian cách nhau bao lâu.
///
/// Nên phần lõi ở đây **tất định** và test được không cần ngủ, còn chỉ một test mỏng
/// ở dưới chạm thời gian thật để chứng minh dây nối đúng.
#[derive(Debug)]
pub struct BoGop {
    cua_so: Duration,
    /// Mốc thời gian của lần **phát** gần nhất. `None` = chưa phát lần nào.
    lan_phat_cuoi: Option<Instant>,
    /// Số sự kiện đã nuốt kể từ lần phát gần nhất — chỉ để chẩn đoán.
    da_nuot: usize,
}

impl BoGop {
    pub fn moi(cua_so: Duration) -> Self {
        Self {
            cua_so,
            lan_phat_cuoi: None,
            da_nuot: 0,
        }
    }

    /// Một sự kiện đến lúc `bay_gio`. Trả `true` nếu **phải làm mới**.
    ///
    /// Sự kiện đầu tiên luôn làm mới. Sự kiện tiếp theo **trong** cửa sổ bị nuốt; sự
    /// kiện **ngoài** cửa sổ làm mới lần nữa.
    ///
    /// 🔴 Phép phân biệt "ngoài cửa sổ thì làm mới lần nữa" là thứ chứng minh phép
    /// gộp **không phải** "bỏ hết trừ cái đầu" — thiếu nó, một cài đặt chỉ làm mới
    /// đúng một lần trong cả đời watcher cũng thoả test "ba sự kiện → một lần".
    pub fn nhan(&mut self, bay_gio: Instant) -> bool {
        match self.lan_phat_cuoi {
            Some(truoc) if bay_gio.duration_since(truoc) < self.cua_so => {
                self.da_nuot += 1;
                false
            }
            _ => {
                self.lan_phat_cuoi = Some(bay_gio);
                self.da_nuot = 0;
                true
            }
        }
    }

    /// Số sự kiện đã nuốt kể từ lần phát gần nhất.
    pub fn da_nuot(&self) -> usize {
        self.da_nuot
    }
}

// ---------------------------------------------------------------------------
// Phần vỏ: watcher thật, theo RepoId
// ---------------------------------------------------------------------------

/// Một watcher đang sống cho một repository.
///
/// Giữ `Debouncer` để nó **không** bị drop — drop `Debouncer` là dừng luồng theo dõi
/// và đóng handle hệ điều hành. Đó cũng chính là cách [`SoTayWatcher::dung`] giải
/// phóng tài nguyên: không có API "stop" nào cần gọi, chỉ cần bỏ giá trị đi.
struct WatcherSong {
    /// Kiểu đầy đủ của `notify-debouncer-full` 0.6: `Debouncer<T, C>` với `T` là bộ
    /// theo dõi khuyến nghị của nền tảng và `C` là cache của nó.
    _debouncer: Debouncer<notify::RecommendedWatcher, RecommendedCache>,
    /// Đường dẫn `.git` đang theo dõi — để chẩn đoán và để test khẳng định.
    thu_muc_git: PathBuf,
}

/// Sổ tay các watcher đang sống, khoá theo [`RepoId`].
///
/// Khuôn map của [`crate::state::AppState`] (PLAT-05) — xem doc comment đầu module về
/// hệ quả tích luỹ.
#[derive(Default)]
pub struct SoTayWatcher {
    dang_song: Mutex<HashMap<RepoId, WatcherSong>>,
}

impl SoTayWatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Số watcher đang sống. Test đo rò rỉ dùng con số này.
    pub fn so_luong(&self) -> usize {
        self.dang_song.lock().len()
    }

    /// Repo này có watcher đang sống không?
    pub fn dang_theo_doi(&self, repo_id: &str) -> bool {
        self.dang_song.lock().contains_key(repo_id)
    }

    /// Đường dẫn `.git` mà repo này đang theo dõi.
    pub fn thu_muc_git_cua(&self, repo_id: &str) -> Option<PathBuf> {
        self.dang_song
            .lock()
            .get(repo_id)
            .map(|w| w.thu_muc_git.clone())
    }

    /// 🔴 Dừng và **giải phóng** watcher của một repo.
    ///
    /// Gọi từ [`crate::state::AppState::close_repo`]. Drop `WatcherSong` đóng handle
    /// `ReadDirectoryChangesW` và dừng luồng theo dõi — không có API "stop" nào khác
    /// cần gọi. Có test ghim, vì mở/đóng lặp lại là **đường gọi duy nhất** hiện có và
    /// một rò rỉ ở đây im lặng hoàn toàn.
    pub fn dung(&self, repo_id: &str) -> bool {
        self.dang_song.lock().remove(repo_id).is_some()
    }

    /// Bắt đầu theo dõi `.git` của một repository.
    ///
    /// `khi_doi` chạy trên luồng của debouncer mỗi khi có **ít nhất một** đường dẫn
    /// đáng quan tâm đổi trong một cửa sổ trì hoãn. Nó nhận `repo_id` để chỗ gọi biết
    /// repo nào cần đọc lại.
    ///
    /// 🔴 `khi_doi` chỉ được **đọc rồi phát**. Nó **không bao giờ** khởi tạo một thao
    /// tác ghi — xem R2 ở doc comment đầu module.
    ///
    /// Theo dõi lại một repo đã theo dõi sẽ **thay** watcher cũ, không thêm cái thứ
    /// hai (cùng tinh thần `or_insert_with` của `open_repo`).
    pub fn bat_dau<F>(&self, repo_id: RepoId, thu_muc_lam_viec: &Path, khi_doi: F) -> Result<(), String>
    where
        F: Fn(RepoId) + Send + 'static,
    {
        let thu_muc_git = thu_muc_lam_viec.join(".git");

        // 🔴 Repo không tồn tại → lỗi đọc hiểu được, **không** panic, và không làm sập
        // ứng dụng. `notify` sẽ trả `io::Error` ở `watch()`, nhưng kiểm sớm cho thông
        // điệp nói đúng vấn đề thay vì "The system cannot find the path specified".
        if !thu_muc_git.exists() {
            return Err(format!(
                "không theo dõi được: `{}` không tồn tại (không phải một repository git?)",
                thu_muc_git.display()
            ));
        }

        let id_cho_callback = repo_id.clone();
        let mut bo_gop = BoGop::moi(CUA_SO_TRI_HOAN);

        let mut debouncer = new_debouncer(
            CUA_SO_TRI_HOAN,
            None,
            move |ket_qua: DebounceEventResult| {
                let su_kien = match ket_qua {
                    Ok(v) => v,
                    Err(loi) => {
                        // Lỗi theo dõi **không** được làm sập ứng dụng. Ghi log rồi
                        // thôi: đường chính (thao tác ghi trả trạng thái trực tiếp)
                        // vẫn chạy mà không cần watcher.
                        tracing::warn!(?loi, "lỗi theo dõi .git — bỏ qua, không làm sập");
                        return;
                    }
                };

                // Lớp phòng thủ **thứ hai**: lọc lại kể cả khi đăng ký đã hẹp.
                // `notify` trên Windows có thể trả sự kiện cho thư mục cha.
                let co_lien_quan = su_kien
                    .iter()
                    .flat_map(|e| e.paths.iter())
                    .any(|p| should_notify(p));

                if !co_lien_quan {
                    return;
                }

                // `notify-debouncer-full` đã gộp theo cửa sổ của nó; [`BoGop`] là lớp
                // gộp **thứ hai**, và nó tồn tại vì debouncer gộp theo **đường dẫn**
                // còn ta cần gộp theo **repo**: ba đường dẫn khác nhau của một lần
                // `git commit` là ba mục riêng với debouncer nhưng **một** hành động
                // với người dùng.
                if bo_gop.nhan(Instant::now()) {
                    khi_doi(id_cho_callback.clone());
                }
            },
        )
        .map_err(|e| format!("không tạo được bộ theo dõi: {e}"))?;

        // 🔴 `.git` gốc: **KHÔNG đệ quy**. Đệ quy kéo cả `objects/` (hàng trăm nghìn
        // tệp trên repo lớn) vào — xem `paths.rs`.
        debouncer
            .watch(&thu_muc_git, RecursiveMode::NonRecursive)
            .map_err(|e| format!("không theo dõi được `{}`: {e}", thu_muc_git.display()))?;

        // `refs/` và thư mục trạng thái rebase: **đệ quy**, vì chúng là cây.
        for ten in paths::WATCHED_RECURSIVE {
            let duong_dan = thu_muc_git.join(ten);
            if !duong_dan.exists() {
                // `rebase-merge` chỉ tồn tại lúc đang rebase — vắng mặt là ca
                // **thường**, không phải lỗi.
                continue;
            }
            if let Err(e) = debouncer.watch(&duong_dan, RecursiveMode::Recursive) {
                tracing::warn!(?e, ?duong_dan, "không theo dõi được thư mục con, bỏ qua");
            }
        }

        self.dang_song.lock().insert(
            repo_id,
            WatcherSong {
                _debouncer: debouncer,
                thu_muc_git,
            },
        );

        Ok(())
    }
}

/// Tên sự kiện Tauri mà watcher phát khi trạng thái đổi từ **bên ngoài** ứng dụng.
///
/// Phía giao diện nối nó vào `statusStore.applyExternal(repoId, status)` — điểm vào đó
/// được khai sẵn ở 04-02 chính vì việc này, nên wave này chỉ phải nối dây chứ không
/// đổi hình dạng store.
///
/// **Không** đăng ký vào capability file: sự kiện đi từ Rust **ra**, và `core:default`
/// đã phủ `listen`.
pub const SU_KIEN_TRANG_THAI_NGOAI: &str = "repo-status-changed";

/// Payload của [`SU_KIEN_TRANG_THAI_NGOAI`].
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrangThaiNgoai {
    pub repo_id: RepoId,
    pub status: crate::domain::RepoStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Hằng số cửa sổ trì hoãn — đột biến M3 (về 0ms) và M4 (lên 5000ms)
    // -----------------------------------------------------------------------

    /// 🔴 Khẳng định **cả hai** biên.
    ///
    /// Chỉ kiểm biên dưới thì một hằng số 5000 ms đi qua (M4), và giao diện trễ 5 giây
    /// sau mỗi lệnh terminal. Chỉ kiểm biên trên thì một hằng số 5 ms đi qua (M3), và
    /// một lần `git commit` sinh ba lần làm mới.
    #[test]
    fn cua_so_tri_hoan_nam_trong_250_300ms() {
        assert!(
            CUA_SO_TRI_HOAN >= TRI_HOAN_TOI_THIEU,
            "cửa sổ {CUA_SO_TRI_HOAN:?} dưới biên 250ms — ba sự kiện của một `git commit` \
             sẽ không gộp được"
        );
        assert!(
            CUA_SO_TRI_HOAN <= TRI_HOAN_TOI_DA,
            "cửa sổ {CUA_SO_TRI_HOAN:?} trên biên 300ms — giao diện trễ sau mỗi lệnh terminal"
        );
    }

    #[test]
    fn hai_bien_cua_rang_buoc_dung_theo_roadmap() {
        assert_eq!(TRI_HOAN_TOI_THIEU, Duration::from_millis(250));
        assert_eq!(TRI_HOAN_TOI_DA, Duration::from_millis(300));
    }

    // -----------------------------------------------------------------------
    // Phép gộp — TẤT ĐỊNH, đồng hồ tiêm vào
    // -----------------------------------------------------------------------

    /// 🔴 Ca then chốt: **ba** sự kiện của **một** lần `git commit` (`index`, `HEAD`,
    /// `refs/heads/main`) sinh **MỘT** lần làm mới, không ba.
    #[test]
    fn ba_su_kien_cua_mot_lan_commit_sinh_mot_lan_lam_moi() {
        let mut g = BoGop::moi(CUA_SO_TRI_HOAN);
        let t0 = Instant::now();

        // Ba đường dẫn thật mà `git commit` chạm, đến cách nhau vài ms.
        let mut so_lan = 0;
        for tre_ms in [0u64, 3, 7] {
            if g.nhan(t0 + Duration::from_millis(tre_ms)) {
                so_lan += 1;
            }
        }

        assert_eq!(
            so_lan, 1,
            "ba sự kiện của MỘT hành động phải sinh MỘT lần làm mới, không ba"
        );
        assert_eq!(g.da_nuot(), 2, "hai sự kiện sau phải bị nuốt");
    }

    /// 🔴 Phép phân biệt: sự kiện cách nhau **quá** cửa sổ → **hai** lần làm mới.
    ///
    /// Thiếu test này thì một cài đặt "làm mới đúng một lần rồi thôi mãi mãi" cũng
    /// thoả test trên — và nó là một watcher chết mà không ai biết.
    #[test]
    fn su_kien_cach_nhau_qua_cua_so_sinh_hai_lan_lam_moi() {
        let mut g = BoGop::moi(CUA_SO_TRI_HOAN);
        let t0 = Instant::now();

        assert!(g.nhan(t0), "sự kiện đầu luôn làm mới");
        assert!(
            !g.nhan(t0 + Duration::from_millis(10)),
            "sự kiện trong cửa sổ phải bị nuốt"
        );
        assert!(
            g.nhan(t0 + CUA_SO_TRI_HOAN + Duration::from_millis(1)),
            "sự kiện NGOÀI cửa sổ phải làm mới lần nữa — nếu không thì phép gộp chỉ là \
             `bỏ hết trừ cái đầu`"
        );
    }

    #[test]
    fn su_kien_dau_tien_luon_lam_moi() {
        let mut g = BoGop::moi(CUA_SO_TRI_HOAN);
        assert!(g.nhan(Instant::now()));
    }

    /// Đúng biên: một sự kiện đến **đúng** lúc cửa sổ hết phải làm mới, không nuốt.
    #[test]
    fn dung_bien_cua_so_thi_lam_moi() {
        let mut g = BoGop::moi(CUA_SO_TRI_HOAN);
        let t0 = Instant::now();
        assert!(g.nhan(t0));
        assert!(
            g.nhan(t0 + CUA_SO_TRI_HOAN),
            "đúng biên cửa sổ là NGOÀI cửa sổ"
        );
    }

    /// Một chuỗi dài sự kiện dày đặc (người dùng chạy `git rebase` ở terminal) không
    /// được sinh một lần làm mới cho mỗi sự kiện.
    #[test]
    fn chuoi_su_kien_day_dac_bi_gop_manh() {
        let mut g = BoGop::moi(CUA_SO_TRI_HOAN);
        let t0 = Instant::now();

        let mut so_lan = 0;
        // 100 sự kiện trong 1 giây, cách nhau 10ms.
        for i in 0..100u64 {
            if g.nhan(t0 + Duration::from_millis(i * 10)) {
                so_lan += 1;
            }
        }

        // 1000ms / 280ms → 4 lần (t=0, 280, 560, 840). Khẳng định bằng khoảng để
        // không phải sửa test khi hằng số đổi trong phạm vi ràng buộc.
        assert!(
            (3..=5).contains(&so_lan),
            "100 sự kiện trong 1 giây phải gộp còn 3-5 lần làm mới, được {so_lan}"
        );
    }

    // -----------------------------------------------------------------------
    // Sổ tay watcher — vòng đời và giải phóng
    // -----------------------------------------------------------------------

    #[test]
    fn so_tay_rong_luc_dau() {
        let s = SoTayWatcher::new();
        assert_eq!(s.so_luong(), 0);
        assert!(!s.dang_theo_doi("bat-ky"));
    }

    /// 🔴 Repo **không tồn tại** → lỗi đọc hiểu được, **không** panic, không làm sập.
    #[test]
    fn repo_khong_ton_tai_tra_loi_khong_panic() {
        let s = SoTayWatcher::new();
        let ket_qua = s.bat_dau(
            "id-bia".to_string(),
            Path::new("C:/khong/ton/tai/o/dau/ca"),
            |_| {},
        );

        let loi = ket_qua.expect_err("repo không tồn tại phải trả lỗi");
        assert!(
            loi.contains("không tồn tại"),
            "thông điệp phải nói đúng vấn đề, được: {loi}"
        );
        assert_eq!(s.so_luong(), 0, "thất bại không được để lại watcher nửa vời");
    }

    /// 🔴 Đóng repo → watcher của repo đó dừng và **giải phóng**.
    ///
    /// Mở/đóng lặp lại là đường gọi **duy nhất** hiện có, nên rò rỉ ở đây im lặng hoàn
    /// toàn: không lỗi, không log, chỉ một handle hệ điều hành mỗi lần.
    #[test]
    fn dung_giai_phong_watcher() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".git")).expect("mkdir .git");

        let s = SoTayWatcher::new();
        s.bat_dau("repo-a".to_string(), tmp.path(), |_| {})
            .expect("bắt đầu theo dõi");

        assert_eq!(s.so_luong(), 1, "tiền đề: có một watcher đang sống");
        assert!(s.dang_theo_doi("repo-a"));

        assert!(s.dung("repo-a"), "dừng một watcher đang sống trả true");

        assert_eq!(s.so_luong(), 0, "đóng repo phải GIẢI PHÓNG watcher");
        assert!(!s.dang_theo_doi("repo-a"));
        assert!(!s.dung("repo-a"), "dừng lần hai trả false");
    }

    /// Mở/đóng **lặp lại** không tích luỹ watcher. Đây là phép đo rò rỉ thật sự: một
    /// lần mở/đóng có thể đúng do may, mười lần thì không.
    #[test]
    fn mo_dong_lap_lai_khong_tich_luy() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".git")).expect("mkdir .git");

        let s = SoTayWatcher::new();
        for _ in 0..10 {
            s.bat_dau("repo-a".to_string(), tmp.path(), |_| {})
                .expect("bắt đầu");
            assert_eq!(s.so_luong(), 1);
            assert!(s.dung("repo-a"));
            assert_eq!(s.so_luong(), 0);
        }
    }

    /// Nhiều repo mở cùng lúc = nhiều watcher sống. **Ghim hành vi tích luỹ**, vì đó
    /// là thứ quyết định con số RAM ở SUMMARY — xem doc comment đầu module.
    #[test]
    fn nhieu_repo_cho_nhieu_watcher_song() {
        let tmps: Vec<_> = (0..5)
            .map(|_| {
                let t = tempfile::tempdir().expect("tempdir");
                std::fs::create_dir_all(t.path().join(".git")).expect("mkdir .git");
                t
            })
            .collect();

        let s = SoTayWatcher::new();
        for (i, t) in tmps.iter().enumerate() {
            s.bat_dau(format!("repo-{i}"), t.path(), |_| {})
                .expect("bắt đầu");
        }

        assert_eq!(
            s.so_luong(),
            5,
            "mở 5 repo = 5 watcher sống; `openRepository` KHÔNG đóng repo cũ"
        );

        // Đóng một cái không đụng bốn cái kia.
        assert!(s.dung("repo-2"));
        assert_eq!(s.so_luong(), 4);
        assert!(s.dang_theo_doi("repo-0"));
        assert!(!s.dang_theo_doi("repo-2"));
    }

    /// Theo dõi lại cùng repo **thay** watcher cũ, không thêm cái thứ hai.
    #[test]
    fn theo_doi_lai_cung_repo_khong_them_watcher() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".git")).expect("mkdir .git");

        let s = SoTayWatcher::new();
        s.bat_dau("repo-a".to_string(), tmp.path(), |_| {})
            .expect("lần 1");
        s.bat_dau("repo-a".to_string(), tmp.path(), |_| {})
            .expect("lần 2");

        assert_eq!(s.so_luong(), 1, "theo dõi lại phải THAY, không thêm");
    }

    #[test]
    fn theo_doi_dung_thu_muc_git() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".git")).expect("mkdir .git");

        let s = SoTayWatcher::new();
        s.bat_dau("repo-a".to_string(), tmp.path(), |_| {})
            .expect("bắt đầu");

        let theo_doi = s.thu_muc_git_cua("repo-a").expect("phải có đường dẫn");
        assert!(
            theo_doi.ends_with(".git"),
            "phải theo dõi `.git`, không phải cây làm việc: {}",
            theo_doi.display()
        );
    }

    /// Tên sự kiện Tauri là một **hợp đồng** với phía giao diện — đổi nó mà không đổi
    /// bên kia là một lỗi im lặng hoàn toàn (`listen` chỉ đơn giản không bao giờ chạy).
    #[test]
    fn ten_su_kien_la_hop_dong_voi_giao_dien() {
        assert_eq!(SU_KIEN_TRANG_THAI_NGOAI, "repo-status-changed");
    }

    // -----------------------------------------------------------------------
    // MỘT test mỏng chạm thời gian THẬT — chứng minh dây nối đúng
    // -----------------------------------------------------------------------

    /// Ghi thật vào `.git/HEAD` → callback chạy.
    ///
    /// ⚠️ **Đây là test có thời gian, và nó là test bất đồng bộ.** Chờ ~3× cửa sổ trì
    /// hoãn trước khi đếm, nếu không nó đỏ lẻ tẻ trên máy tải cao — đúng lớp lỗi #6
    /// (đỏ 1/5 lần và sống qua cả một wave). Mọi phép **quyết định** đã được test tất
    /// định ở [`BoGop`]; test này chỉ chứng minh `notify` → bộ lọc → callback nối
    /// đúng, nên nó khẳng định `>= 1` chứ **không** khẳng định một con số chính xác.
    ///
    /// Khẳng định một con số chính xác ở đây là đòi hệ tệp phải tất định, và nó không
    /// tất định: `notify` trên Windows gộp sự kiện theo cách riêng của
    /// `ReadDirectoryChangesW`, và trình diệt virus/OneDrive chạm tệp thêm.
    #[test]
    fn ghi_that_vao_git_head_lam_callback_chay() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let tmp = tempfile::tempdir().expect("tempdir");
        let thu_muc_git = tmp.path().join(".git");
        std::fs::create_dir_all(&thu_muc_git).expect("mkdir .git");
        std::fs::write(thu_muc_git.join("HEAD"), b"ref: refs/heads/main\n").expect("ghi HEAD");

        let dem = Arc::new(AtomicUsize::new(0));
        let dem_cb = Arc::clone(&dem);

        let s = SoTayWatcher::new();
        s.bat_dau("repo-a".to_string(), tmp.path(), move |_id| {
            dem_cb.fetch_add(1, Ordering::SeqCst);
        })
        .expect("bắt đầu theo dõi");

        // Cho watcher kịp đăng ký trước khi ghi — nếu không, phép ghi xảy ra trước
        // khi `ReadDirectoryChangesW` bắt đầu nghe và test đỏ vì lý do sai.
        std::thread::sleep(Duration::from_millis(200));

        std::fs::write(thu_muc_git.join("HEAD"), b"ref: refs/heads/khac\n").expect("ghi lại HEAD");

        // ~3× cửa sổ trì hoãn cộng biên cho hệ tệp.
        std::thread::sleep(CUA_SO_TRI_HOAN * 3 + Duration::from_millis(400));

        assert!(
            dem.load(Ordering::SeqCst) >= 1,
            "ghi vào .git/HEAD phải làm callback chạy ít nhất một lần"
        );

        s.dung("repo-a");
    }

    /// Ghi vào **thư mục làm việc** → callback **KHÔNG** chạy.
    ///
    /// Đây là nửa còn lại của phép phân biệt: test trên một mình cũng thoả bởi một
    /// watcher nghe **mọi thứ**, và một watcher nghe mọi thứ là đúng thứ vi phạm ràng
    /// buộc RAM/CPU (`node_modules` hàng chục nghìn tệp).
    #[test]
    fn ghi_vao_thu_muc_lam_viec_khong_lam_callback_chay() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".git")).expect("mkdir .git");
        std::fs::create_dir_all(tmp.path().join("node_modules")).expect("mkdir node_modules");

        let dem = Arc::new(AtomicUsize::new(0));
        let dem_cb = Arc::clone(&dem);

        let s = SoTayWatcher::new();
        s.bat_dau("repo-a".to_string(), tmp.path(), move |_id| {
            dem_cb.fetch_add(1, Ordering::SeqCst);
        })
        .expect("bắt đầu theo dõi");

        std::thread::sleep(Duration::from_millis(200));

        for i in 0..20 {
            std::fs::write(
                tmp.path().join("node_modules").join(format!("t{i}.js")),
                b"x",
            )
            .expect("ghi");
        }
        std::fs::write(tmp.path().join("src.rs"), b"fn main() {}").expect("ghi");

        std::thread::sleep(CUA_SO_TRI_HOAN * 3 + Duration::from_millis(400));

        assert_eq!(
            dem.load(Ordering::SeqCst),
            0,
            "21 lần ghi vào thư mục làm việc KHÔNG được sinh lần làm mới nào"
        );

        s.dung("repo-a");
    }
}
