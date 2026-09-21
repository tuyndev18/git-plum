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
}

impl AppState {
    pub fn new() -> Self {
        Self {
            open_repos: RwLock::new(HashMap::new()),
            active_repo: RwLock::new(None),
            command_log: Arc::new(CommandLog::new()),
            cache: Arc::new(crate::cache::RepoCache::new()),
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
    pub fn close_repo(&self, id: &str) {
        self.open_repos.write().remove(id);
        self.cache.invalidate(id);
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
