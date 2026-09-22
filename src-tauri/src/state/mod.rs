//! Trạng thái ứng dụng — PLAT-03, PLAT-05, PLAT-08.
//!
//! Ba ràng buộc kiến trúc nằm ở đây, và cả ba đều thuộc loại không sửa được về sau:
//!
//! * **PLAT-05** — trạng thái khoá theo `RepoId`, không phải một handle đơn lẻ.
//!   Nhờ vậy thêm nhiều repository mở cùng lúc về sau không phải viết lại.
//! * **PLAT-03** — mỗi repository có một khoá ghi riêng. Mọi lệnh làm thay đổi
//!   repository phải đi qua khoá này, nếu không sẽ đụng `index.lock`.
//! * **PLAT-08** — nhật ký lệnh ghi lại đúng những gì đã chạy.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use parking_lot::RwLock;
use serde::Serialize;

pub mod command_log;

pub use command_log::{CommandLog, CommandLogEntry};

/// Mã định danh một repository đang mở.
pub type RepoId = String;

/// Một repository đang mở.
pub struct RepoHandle {
    pub id: RepoId,
    /// Thư mục gốc của cây làm việc.
    pub path: PathBuf,
    /// Tên hiển thị — tên thư mục cuối cùng.
    pub name: String,
    /// Khoá ghi (PLAT-03). Mọi lệnh làm thay đổi repository phải giữ khoá này.
    /// Dùng `tokio::sync::Mutex` chứ không phải `parking_lot`, vì khoá cần được
    /// giữ qua các điểm `.await`.
    write_lock: Arc<tokio::sync::Mutex<()>>,
}

impl RepoHandle {
    fn new(id: RepoId, path: PathBuf) -> Self {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());

        Self {
            id,
            path,
            name,
            write_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }

    /// Lấy khoá ghi của repository này (PLAT-03).
    ///
    /// Giữ giá trị trả về trong suốt thời gian chạy lệnh làm thay đổi repository.
    /// Thả khoá ra là lệnh khác được phép chạy.
    pub fn write_lock(&self) -> Arc<tokio::sync::Mutex<()>> {
        Arc::clone(&self.write_lock)
    }
}

/// Thông tin repository gửi sang phía giao diện.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    pub id: RepoId,
    pub path: String,
    pub name: String,
}

impl From<&RepoHandle> for RepoInfo {
    fn from(h: &RepoHandle) -> Self {
        Self {
            id: h.id.clone(),
            path: h.path.to_string_lossy().into_owned(),
            name: h.name.clone(),
        }
    }
}

/// Trạng thái toàn ứng dụng.
///
/// `open_repos` là một map ngay từ đầu (PLAT-05), dù v1 chỉ mở một repository
/// tại một thời điểm. Đổi từ `Option<RepoHandle>` sang map về sau là thay đổi
/// đắt nhất ở phía giao diện, nên làm đúng ngay từ bây giờ.
pub struct AppState {
    open_repos: RwLock<HashMap<RepoId, Arc<RepoHandle>>>,
    /// Repository đang hiển thị. v1 chỉ có một; về sau đây là repo của thẻ đang chọn.
    active_repo: RwLock<Option<RepoId>>,
    /// Nhật ký lệnh dùng chung cho toàn ứng dụng (PLAT-08).
    /// Là `Arc` để `GitRunner` giữ được một tham chiếu — mọi lệnh phải đổ vào
    /// cùng một nhật ký, nếu không người dùng sẽ thấy nhật ký thiếu.
    pub command_log: Arc<CommandLog>,
    /// Cache lịch sử và ref, khoá theo `RepoId` (PLAT-05, HIST-01).
    ///
    /// Nằm ở `AppState` chứ không phải biến toàn cục hay `thread_local`: Tauri quản lý
    /// vòng đời của `AppState`, nên cache sống và chết cùng ứng dụng mà không cần
    /// `lazy_static`. Là `Arc` để command mượn được mà không giữ khoá của state.
    ///
    /// Chặn trên số lịch sử được giữ nằm ở `cache::MAX_CACHED_HISTORIES`.
    pub cache: Arc<crate::cache::RepoCache>,

    /// Cache diff đã phân tích, khoá theo `(repo_id, sha, path)` — DIFF-01, T-03-12.
    ///
    /// Nằm **cạnh** `cache`, không nằm **trong** nó. Hai cache giải hai bài toán khác
    /// nhau: `RepoCache` giữ 2 mục cỡ 67 MB với quy tắc loại bỏ phụ thuộc repo đang
    /// hiển thị; `DiffCache` giữ 200 mục cỡ KB với quy tắc LRU thuần. Gộp chúng lại
    /// buộc một trong hai phải chịu quy tắc của cái kia. Xem `cache::diff_cache`.
    pub diff_cache: Arc<crate::cache::DiffCache>,

    /// Các watcher `.git` đang sống, khoá theo `RepoId` — WORK-10.
    ///
    /// 🔴 **Tích luỹ theo số repo mở**, không phải một cái duy nhất: `open_repo` dùng
    /// `or_insert_with` nên mở một đường dẫn **khác** thêm một mục mà không bỏ mục cũ,
    /// và `close_repo` chỉ chạy từ lệnh "Đóng repository" tường minh. Mở 5 repo trong
    /// một phiên = **5 watcher sống**, mỗi cái giữ một handle `ReadDirectoryChangesW`.
    ///
    /// Nằm ở đây, cạnh hai cache, vì nó chịu **cùng** vòng đời: [`Self::close_repo`]
    /// phải giải phóng cả ba, và gom chúng vào một chỗ làm việc bỏ sót khó xảy ra hơn.
    /// Xem doc comment module `crate::watch`.
    pub watchers: Arc<crate::watch::SoTayWatcher>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            open_repos: RwLock::new(HashMap::new()),
            active_repo: RwLock::new(None),
            command_log: Arc::new(CommandLog::new()),
            cache: Arc::new(crate::cache::RepoCache::new()),
            diff_cache: Arc::new(crate::cache::DiffCache::new()),
            watchers: Arc::new(crate::watch::SoTayWatcher::new()),
        }
    }

    /// Dựng một `GitRunner` cho repository, dùng chung nhật ký của ứng dụng.
    /// Đây là cách duy nhất nên dùng để tạo runner — tự tạo `CommandLog` mới
    /// sẽ khiến lệnh không xuất hiện trong nhật ký người dùng nhìn thấy.
    pub fn runner(&self, repo: Arc<RepoHandle>) -> crate::git::GitRunner {
        crate::git::GitRunner::new(repo, Arc::clone(&self.command_log))
    }

    /// Mở một repository và đặt nó làm repository đang hoạt động.
    /// Mở lại repository đã mở thì trả về đúng handle cũ, không tạo bản sao.
    pub fn open_repo(&self, path: impl AsRef<Path>) -> Arc<RepoHandle> {
        let path = path.as_ref().to_path_buf();
        let id = repo_id_for(&path);

        let mut repos = self.open_repos.write();
        let handle = repos
            .entry(id.clone())
            .or_insert_with(|| Arc::new(RepoHandle::new(id.clone(), path)))
            .clone();

        *self.active_repo.write() = Some(id);
        handle
    }

    pub fn get_repo(&self, id: &str) -> Option<Arc<RepoHandle>> {
        self.open_repos.read().get(id).cloned()
    }

    pub fn active_repo(&self) -> Option<Arc<RepoHandle>> {
        let id = self.active_repo.read().clone()?;
        self.get_repo(&id)
    }

    /// Đóng một repository và **giải phóng cache của nó**.
    ///
    /// Không gọi `cache.invalidate` ở đây thì một lịch sử ~67MB nằm lại trong RAM cho
    /// một repo người dùng đã đóng, và `MAX_CACHED_HISTORIES` sẽ bảo vệ một thứ không
    /// ai còn nhìn. Đây là ràng buộc RAM dưới 150MB của `PROJECT.md`, không phải dọn
    /// dẹp cho gọn.
    ///
    /// **Cả hai** cache phải được giải phóng, không chỉ `cache`. `diff_cache` giữ tới
    /// 200 mục và chặn trên của nó là toàn cục chứ không theo repo — bỏ sót ở đây thì
    /// các mục của repo đã đóng vẫn chiếm chỗ và đẩy mục của repo đang mở ra sớm hơn
    /// cần thiết, tức người dùng mất cache hit ở repo họ đang thật sự dùng.
    ///
    /// 🔴 **Và watcher `.git` (WORK-10).** Đây là ca nặng hơn hai cache: một watcher
    /// giữ một handle `ReadDirectoryChangesW` của hệ điều hành cộng buffer cho mỗi
    /// đường theo dõi, và đây là **đường gọi duy nhất** giải phóng nó. Bỏ sót thì
    /// mở/đóng lặp lại rò rỉ handle mà **không** lỗi, **không** log — chỉ một tiến
    /// trình lớn dần. Có test ghim (`watch::tests::dung_giai_phong_watcher` và
    /// `mo_dong_lap_lai_khong_tich_luy`).
    pub fn close_repo(&self, id: &str) {
        self.open_repos.write().remove(id);
        self.cache.invalidate(id);
        self.diff_cache.invalidate_repo(id);
        self.watchers.dung(id);
        let mut active = self.active_repo.write();
        if active.as_deref() == Some(id) {
            *active = None;
        }
    }

    pub fn list_repos(&self) -> Vec<RepoInfo> {
        self.open_repos
            .read()
            .values()
            .map(|h| RepoInfo::from(h.as_ref()))
            .collect()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Sinh mã định danh ổn định từ đường dẫn.
///
/// Chuẩn hoá dấu gạch chéo để cùng một repository trên Windows không sinh ra hai
/// mã khác nhau khi đường dẫn viết bằng `/` hay `\`.
fn repo_id_for(path: &Path) -> RepoId {
    let normalized = path.to_string_lossy().replace('\\', "/");
    let normalized = normalized.trim_end_matches('/');

    // Băm đơn giản, đủ dùng làm khoá map. Không cần chống va chạm có chủ ý.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in normalized.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_path_yields_same_id() {
        let a = repo_id_for(Path::new("C:/work/repo"));
        let b = repo_id_for(Path::new("C:\\work\\repo"));
        let c = repo_id_for(Path::new("C:/work/repo/"));
        assert_eq!(a, b, "dấu gạch chéo ngược phải cho cùng mã");
        assert_eq!(a, c, "dấu gạch chéo thừa ở cuối phải cho cùng mã");
    }

    #[test]
    fn different_paths_yield_different_ids() {
        assert_ne!(
            repo_id_for(Path::new("C:/work/repo-a")),
            repo_id_for(Path::new("C:/work/repo-b"))
        );
    }

    #[test]
    fn reopening_returns_the_same_handle() {
        let state = AppState::new();
        let a = state.open_repo("C:/work/repo");
        let b = state.open_repo("C:/work/repo");
        assert!(Arc::ptr_eq(&a, &b), "mở lại phải trả về đúng handle cũ");
        assert_eq!(state.list_repos().len(), 1);
    }

    #[test]
    fn multiple_repos_coexist() {
        // PLAT-05: kiến trúc phải chứa được nhiều repository ngay từ bây giờ,
        // kể cả khi giao diện v1 chỉ hiện một.
        let state = AppState::new();
        state.open_repo("C:/work/repo-a");
        state.open_repo("C:/work/repo-b");
        assert_eq!(state.list_repos().len(), 2);
        assert_eq!(state.active_repo().unwrap().name, "repo-b");
    }

    #[test]
    fn closing_active_repo_clears_active() {
        let state = AppState::new();
        let h = state.open_repo("C:/work/repo");
        state.close_repo(&h.id);
        assert!(state.active_repo().is_none());
        assert!(state.list_repos().is_empty());
    }

    /// Đóng repository phải giải phóng cache của nó. Một lịch sử 100k commit là ~67MB;
    /// giữ lại sau khi đóng là rò rỉ đúng nghĩa với ràng buộc RAM dưới 150MB.
    #[test]
    fn closing_repo_frees_its_cache() {
        let state = AppState::new();
        let h = state.open_repo("C:/work/repo");

        state.cache.put_refs(&h.id, Vec::new());
        assert!(
            state.cache.get_refs(&h.id).is_some(),
            "tiền đề: cache có dữ liệu"
        );

        state.close_repo(&h.id);

        assert!(
            state.cache.get_refs(&h.id).is_none(),
            "đóng repository phải giải phóng cache của nó"
        );
    }

    /// Đóng repository cũng phải giải phóng **cache diff** của nó, không chỉ cache
    /// lịch sử.
    ///
    /// `MAX_CACHED_DIFFS` là chặn trên **toàn cục** (200 mục cho mọi repo cộng lại),
    /// không phải chặn trên theo repo. Nên mục của một repo đã đóng không chỉ lãng phí
    /// RAM — chúng còn đẩy mục của repo đang mở ra khỏi cache sớm hơn cần thiết, và
    /// người dùng mất cache hit ở đúng repo họ đang dùng.
    #[test]
    fn closing_repo_frees_its_diff_cache() {
        use crate::cache::DiffKey;
        use crate::domain::diff::{DiffKind, FileDiff};

        let state = AppState::new();
        let a = state.open_repo("C:/work/repo-a");
        let b = state.open_repo("C:/work/repo-b");

        let khoa_a = DiffKey::new(a.id.clone(), "sha1", "f.rs");
        let khoa_b = DiffKey::new(b.id.clone(), "sha1", "f.rs");
        let mau = || {
            Arc::new(FileDiff {
                path: "f.rs".into(),
                old_path: None,
                status: "M".into(),
                kind: DiffKind::Unchanged,
            })
        };
        state.diff_cache.put_diff(khoa_a.clone(), mau());
        state.diff_cache.put_diff(khoa_b.clone(), mau());
        assert_eq!(state.diff_cache.len(), 2, "tiền đề: cả hai repo có mục");

        state.close_repo(&a.id);

        assert!(
            state.diff_cache.get_diff(&khoa_a).is_none(),
            "đóng repository phải giải phóng cache diff của nó"
        );
        assert!(
            state.diff_cache.get_diff(&khoa_b).is_some(),
            "mục của repo còn mở KHÔNG được đụng tới"
        );
    }

    /// 🔴 Đóng repository cũng phải giải phóng **watcher `.git`** của nó — WORK-10.
    ///
    /// Nặng hơn hai cache ở trên: một watcher giữ một handle `ReadDirectoryChangesW`
    /// của hệ điều hành, không chỉ RAM trong tiến trình. Và `close_repo` là **đường
    /// gọi duy nhất** hiện có giải phóng nó, nên bỏ sót ở đây rò rỉ hoàn toàn im lặng:
    /// không lỗi, không log, chỉ một handle mỗi lần mở/đóng.
    #[test]
    fn closing_repo_stops_its_watcher() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(tmp.path().join(".git")).expect("mkdir .git");

        let state = AppState::new();
        let h = state.open_repo(tmp.path());

        state
            .watchers
            .bat_dau(h.id.clone(), tmp.path(), |_| {})
            .expect("bắt đầu theo dõi");
        assert!(
            state.watchers.dang_theo_doi(&h.id),
            "tiền đề: watcher đang sống"
        );

        state.close_repo(&h.id);

        assert!(
            !state.watchers.dang_theo_doi(&h.id),
            "đóng repository phải DỪNG watcher của nó — nếu không, mở/đóng lặp lại là \
             rò rỉ handle hệ điều hành"
        );
        assert_eq!(state.watchers.so_luong(), 0);
    }

    /// Đóng một repo **không** đụng watcher của repo còn mở. Cùng phép phân biệt với
    /// `closing_repo_frees_its_diff_cache`: một `close_repo` dọn sạch **mọi** watcher
    /// cũng thoả test trên, và nó sẽ làm repo còn mở im lặng ngừng cập nhật.
    #[test]
    fn closing_one_repo_leaves_other_watchers_alive() {
        let tmp_a = tempfile::tempdir().expect("tempdir a");
        let tmp_b = tempfile::tempdir().expect("tempdir b");
        std::fs::create_dir_all(tmp_a.path().join(".git")).expect("mkdir .git a");
        std::fs::create_dir_all(tmp_b.path().join(".git")).expect("mkdir .git b");

        let state = AppState::new();
        let a = state.open_repo(tmp_a.path());
        let b = state.open_repo(tmp_b.path());
        state
            .watchers
            .bat_dau(a.id.clone(), tmp_a.path(), |_| {})
            .expect("a");
        state
            .watchers
            .bat_dau(b.id.clone(), tmp_b.path(), |_| {})
            .expect("b");
        assert_eq!(state.watchers.so_luong(), 2, "tiền đề: hai watcher sống");

        state.close_repo(&a.id);

        assert!(!state.watchers.dang_theo_doi(&a.id));
        assert!(
            state.watchers.dang_theo_doi(&b.id),
            "watcher của repo CÒN MỞ không được đụng tới"
        );
    }

    #[tokio::test]
    async fn write_lock_serializes_access() {
        // PLAT-03: hai thao tác ghi không bao giờ chạy chồng nhau.
        let state = AppState::new();
        let repo = state.open_repo("C:/work/repo");

        let lock = repo.write_lock();
        let guard = lock.lock().await;

        let lock2 = repo.write_lock();
        assert!(
            lock2.try_lock().is_err(),
            "khoá thứ hai phải bị chặn khi khoá thứ nhất còn giữ"
        );

        drop(guard);
        assert!(lock2.try_lock().is_ok(), "thả khoá xong phải lấy được");
    }
}
