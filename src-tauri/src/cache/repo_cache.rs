//! Cache lịch sử và ref theo `RepoId` — HIST-01, HIST-05, PLAT-05.
//!
//! # Vì sao cache phải tồn tại
//!
//! Gán lane **không tính theo trang được**. ARCHITECTURE.md nói rõ *"lane assignment
//! needs global context, can't be computed per-page in isolation"*: lane của hàng 5000
//! phụ thuộc mọi hàng trước nó. Vì vậy `Vec<GraphRow>` tính **một lần cho toàn bộ lịch
//! sử** rồi nằm ở đây. Không có cache thì mỗi lần cuộn sang trang mới là một lần chạy
//! lại `git log --all` (693ms) cộng `parse_log` (82ms) cộng `assign` (57ms) — mốc dưới
//! một giây của Core Value mất ngay ở trang thứ hai.
//!
//! # Chi phí bộ nhớ thật — đã đo, không đoán
//!
//! Ở 100k commit, một `RepoHistory` tốn khoảng **67MB**:
//!
//! | Thành phần | Chi phí |
//! |---|---|
//! | `Vec<GraphRow>` (80 B × 100k inline + heap) | ~17 MB |
//! | `Vec<Commit>` (216 B × 100k inline + heap) | ~50 MB |
//!
//! `PROJECT.md` ràng buộc **RAM lúc rảnh dưới 150MB**. Đó là lý do
//! [`MAX_CACHED_HISTORIES`] tồn tại và bằng 2, không phải một con số tuỳ ý.

use std::collections::HashMap;
use std::sync::Arc;

use parking_lot::RwLock;

use crate::domain::{Commit, Ref};
use crate::graph::GraphRow;
use crate::state::RepoId;

/// Số lịch sử repo được giữ trong RAM cùng lúc.
///
/// # Con số này suy ra từ đâu — **không** phải một phỏng đoán
///
/// Một lịch sử 100k commit tốn ~67MB (bảng ở đầu module). `PROJECT.md` ràng buộc RAM
/// lúc rảnh **dưới 150MB**, và baseline Tauri/WebView2 đã chiếm 80–150MB trước khi
/// ứng dụng cache gì:
///
/// ```text
///   2 × 67 MB  = 134 MB   ← chỉ riêng cache, đã sát hạn 150 MB
///   3 × 67 MB  = 201 MB   ← vượt hạn, không cần cộng baseline
/// ```
///
/// Nên 2 là **chặn trên**, không phải mức mong muốn. Phần lớn thời gian chỉ có một
/// repo được cache; con số 2 tồn tại để việc chuyển qua lại giữa hai repo (PLAT-05 cho
/// phép) không phải nạp lại từ đầu mỗi lần.
///
/// # Vì sao **không** dùng crate `lru` ở đây
///
/// `lru` có trong `Cargo.toml` nhưng nó dành cho cache **diff** của Phase 3: khoá theo
/// SHA commit, hàng nghìn phần tử, quy tắc loại bỏ thuần theo thứ tự truy cập. Ở đây
/// chỉ có hai phần tử, và quy tắc loại bỏ phụ thuộc **repo nào đang hiển thị** — một
/// thông tin nằm ngoài cache và `lru` không diễn đạt được. Đây là lựa chọn có chủ ý,
/// không phải bỏ sót. Xem [`RepoCache::put_history_keeping`].
pub const MAX_CACHED_HISTORIES: usize = 2;

/// Toàn bộ lịch sử đã nạp của một repository, cộng hình học đồ thị của nó.
///
/// `commits` và `graph_rows` **luôn cùng độ dài** — bất biến mà plan 02-03 thiết lập
/// (`rows.len() == commits.len()`) và là HIST-04 ở tầng dữ liệu. Hai vector nằm cạnh
/// nhau trong cùng một struct chính vì thế: tách chúng ra hai khoá cache riêng sẽ cho
/// phép một cái hết hạn mà cái kia không, và khi đó hàng commit lệch với hàng đồ thị.
pub struct RepoHistory {
    /// Toàn bộ lịch sử đã nạp, theo thứ tự topo mà `--topo-order` bảo đảm.
    pub commits: Vec<Commit>,

    /// Hình học đồ thị, **cùng độ dài** với `commits`, chỉ số khớp một-một.
    pub graph_rows: Vec<GraphRow>,

    /// Thời điểm nạp, mili giây từ epoch. Dùng để hiện "dữ liệu lúc ..." và để plan
    /// sau quyết định khi nào nạp lại.
    pub loaded_at_ms: u64,
}

impl RepoHistory {
    /// Số commit trong lịch sử. Bằng số hàng đồ thị theo bất biến của 02-03.
    pub fn len(&self) -> usize {
        self.commits.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commits.is_empty()
    }
}

/// Phần dữ liệu nằm sau khoá.
#[derive(Default)]
struct Inner {
    histories: HashMap<RepoId, Arc<RepoHistory>>,
    refs: HashMap<RepoId, Arc<Vec<Ref>>>,
    /// Số thứ tự lần ghi của từng lịch sử, để chọn repo **cũ nhất** khi loại bỏ.
    /// Dùng bộ đếm đơn điệu chứ không dùng `loaded_at_ms`: hai lần nạp trong cùng một
    /// mili giây sẽ hoà nhau và việc chọn trở thành ngẫu nhiên.
    put_seq: HashMap<RepoId, u64>,
    next_seq: u64,
}

/// Cache khoá theo `RepoId` (PLAT-05).
///
/// Khoá theo `RepoId` chứ **không** phải một `Option` đơn lẻ hay biến toàn cục: ngay
/// khi có hai repo mở cùng lúc, một cache đơn lẻ sẽ hiện lịch sử của repo này trong
/// cửa sổ của repo kia.
///
/// Dùng `parking_lot::RwLock` chứ không `dashmap` — STACK.md nói rõ *"cache của bạn
/// nằm sau một khoá"*. Một map hai phần tử không có tranh chấp để `dashmap` giải quyết.
///
/// # Ghi chú cho người đọc nhật ký lệnh (T-02-15)
///
/// Cache hit **cố ý không** sinh dòng nào trong `CommandLog`. Nhật ký ghi lại các lệnh
/// git đã chạy, không ghi lời gọi IPC — thấy một lời gọi `get_commit_page` mà không
/// thấy `git log` tương ứng nghĩa là cache đã trả lời, không phải nhật ký bị thiếu.
#[derive(Default)]
pub struct RepoCache {
    inner: RwLock<Inner>,
}

impl RepoCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Lịch sử đã cache của một repo, `None` khi chưa nạp hoặc đã bị loại bỏ.
    ///
    /// Trả `Arc` chứ **không** clone `Vec<Commit>`: ở 100k commit, một lần clone là
    /// ~50MB sao chép và hàng trăm nghìn lần cấp phát — chính xác thứ làm mất mốc dưới
    /// một giây mà cache sinh ra để bảo vệ.
    pub fn get_history(&self, repo: &str) -> Option<Arc<RepoHistory>> {
        self.inner.read().histories.get(repo).cloned()
    }

    /// Ghi lịch sử, không biết repo nào đang hoạt động.
    ///
    /// Khi vượt [`MAX_CACHED_HISTORIES`] thì loại repo cũ nhất. Mã ứng dụng nên gọi
    /// [`Self::put_history_keeping`] để repo người dùng đang xem không bị ném đi.
    pub fn put_history(&self, repo: &str, h: RepoHistory) {
        self.put_history_keeping(repo, h, None);
    }

    /// Ghi lịch sử và **giữ lại** lịch sử của `keep` khi phải loại bỏ.
    ///
    /// `keep` là `RepoId` của repo đang hoạt động (`AppState::active_repo`). Loại bỏ
    /// ngẫu nhiên có thể ném đúng repo người dùng đang nhìn, và lần cuộn tiếp theo phải
    /// chạy lại `git log --all` — người dùng thấy một khựng gần một giây mà không hiểu
    /// vì sao.
    ///
    /// Repo vừa ghi **không bao giờ** bị chọn để loại: ghi vào rồi loại ngay ra là vô
    /// nghĩa và làm mọi lời gọi tiếp theo trượt cache.
    pub fn put_history_keeping(&self, repo: &str, h: RepoHistory, keep: Option<&str>) {
        let mut inner = self.inner.write();

        let seq = inner.next_seq;
        inner.next_seq += 1;
        inner.histories.insert(repo.to_owned(), Arc::new(h));
        inner.put_seq.insert(repo.to_owned(), seq);

        while inner.histories.len() > MAX_CACHED_HISTORIES {
            // Ứng viên loại bỏ: mọi repo trừ repo vừa ghi và repo đang hoạt động.
            let mut nan_nhan = inner
                .histories
                .keys()
                .filter(|id| id.as_str() != repo && Some(id.as_str()) != keep)
                .min_by_key(|id| inner.put_seq.get(*id).copied().unwrap_or(0))
                .cloned();

            // Không còn ứng viên nào nghĩa là repo đang hoạt động cũng phải nhường —
            // chặn trên là chặn trên. Bỏ bước này thì cache phình vô hạn khi người dùng
            // mở nhiều repo và chuyển qua lại, tức là mất đúng tác dụng của hằng số.
            if nan_nhan.is_none() {
                nan_nhan = inner
                    .histories
                    .keys()
                    .filter(|id| id.as_str() != repo)
                    .min_by_key(|id| inner.put_seq.get(*id).copied().unwrap_or(0))
                    .cloned();
            }

            let Some(id) = nan_nhan else {
                // Chỉ còn đúng repo vừa ghi. Không loại nó — nếu `MAX_CACHED_HISTORIES`
                // là 0 thì việc cache mất nghĩa hoàn toàn. Thoát để không lặp vô hạn.
                break;
            };

            inner.histories.remove(&id);
            inner.put_seq.remove(&id);
            tracing::debug!(
                repo = %id,
                "loại lịch sử khỏi cache để giữ dưới chặn trên {MAX_CACHED_HISTORIES}"
            );
        }
    }

    /// Danh sách ref đã cache của một repo.
    pub fn get_refs(&self, repo: &str) -> Option<Arc<Vec<Ref>>> {
        self.inner.read().refs.get(repo).cloned()
    }

    /// Ghi danh sách ref.
    ///
    /// **Không** bị [`MAX_CACHED_HISTORIES`] chặn: danh sách ref của một repo là vài KB
    /// kể cả trên repo lớn, còn một lịch sử là hàng chục MB. Chặn nó lại chỉ làm thanh
    /// bên phải hỏi git lại liên tục mà không tiết kiệm được gì đáng kể.
    pub fn put_refs(&self, repo: &str, refs: Vec<Ref>) {
        self.inner
            .write()
            .refs
            .insert(repo.to_owned(), Arc::new(refs));
    }

    /// Xoá **mọi** thứ đã cache của một repo — cả lịch sử lẫn ref.
    ///
    /// Xoá cả hai cùng lúc có chủ ý: ref trỏ vào mã commit, nên một danh sách ref còn
    /// sống cạnh một lịch sử đã bỏ sẽ cho nhãn trỏ vào hàng không tồn tại.
    pub fn invalidate(&self, repo: &str) {
        let mut inner = self.inner.write();
        inner.histories.remove(repo);
        inner.refs.remove(repo);
        inner.put_seq.remove(repo);
    }

    /// Các `RepoId` đang có lịch sử trong cache. Dùng để kiểm chặn trên trong test và
    /// để hiện chẩn đoán.
    pub fn cached_history_ids(&self) -> Vec<RepoId> {
        self.inner.read().histories.keys().cloned().collect()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RefKind;

    fn commit_mau(id: &str) -> Commit {
        Commit {
            id: id.to_owned(),
            parents: Vec::new(),
            author_name: "A".into(),
            author_email: "a@b.c".into(),
            author_time: 0,
            committer_name: "A".into(),
            committer_email: "a@b.c".into(),
            committer_time: 0,
            subject: "s".into(),
            body: String::new(),
            has_invalid_utf8: false,
        }
    }

    fn history_mau(id: &str) -> RepoHistory {
        let commits = vec![commit_mau(id)];
        let graph_rows = crate::graph::assign(&commits);
        RepoHistory {
            commits,
            graph_rows,
            loaded_at_ms: 1,
        }
    }

    fn ref_mau(name: &str) -> Ref {
        Ref {
            full_name: format!("refs/heads/{name}"),
            short_name: name.to_owned(),
            kind: RefKind::LocalBranch,
            target: "a".repeat(40),
            upstream: None,
            ahead: 0,
            behind: 0,
            is_head: false,
        }
    }

    #[test]
    fn put_roi_get_cung_repo_id_tra_du_lieu() {
        let cache = RepoCache::new();
        cache.put_history("repo-a", history_mau("aaa"));

        let got = cache
            .get_history("repo-a")
            .expect("put rồi get cùng RepoId phải trả dữ liệu");
        assert_eq!(got.commits.len(), 1);
        assert_eq!(got.commits[0].id, "aaa");
    }

    /// PLAT-05: cache khoá theo `RepoId`, không phải một biến toàn cục. Một `RepoId`
    /// khác phải **không** thấy dữ liệu của repo kia — nếu thấy thì hai repo mở cùng
    /// lúc sẽ hiện lịch sử của nhau.
    #[test]
    fn repo_id_khac_tra_none() {
        let cache = RepoCache::new();
        cache.put_history("repo-a", history_mau("aaa"));

        assert!(
            cache.get_history("repo-b").is_none(),
            "RepoId khác không được thấy lịch sử của repo khác"
        );
    }

    #[test]
    fn invalidate_lam_get_sau_do_tra_none() {
        let cache = RepoCache::new();
        cache.put_history("repo-a", history_mau("aaa"));
        cache.put_refs("repo-a", vec![ref_mau("main")]);
        assert!(cache.get_history("repo-a").is_some());
        assert!(cache.get_refs("repo-a").is_some());

        cache.invalidate("repo-a");

        assert!(
            cache.get_history("repo-a").is_none(),
            "invalidate phải xoá lịch sử"
        );
        assert!(
            cache.get_refs("repo-a").is_none(),
            "invalidate phải xoá cả refs, không chỉ lịch sử"
        );
    }

    #[test]
    fn put_refs_roi_get_refs_tra_dung_danh_sach() {
        let cache = RepoCache::new();
        cache.put_refs("repo-a", vec![ref_mau("main"), ref_mau("dev")]);

        let got = cache.get_refs("repo-a").expect("phải trả danh sách ref");
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].short_name, "main");
        assert!(cache.get_refs("repo-b").is_none());
    }

    /// `get_*` trả `Arc`, không clone `Vec`. 100k commit clone mỗi lần gọi command là
    /// chính xác cái làm mất mốc dưới một giây. Kiểm bằng `Arc::ptr_eq`: hai lần gọi
    /// phải cho **cùng một** vùng dữ liệu.
    #[test]
    fn get_tra_arc_dung_chung_khong_clone_du_lieu() {
        let cache = RepoCache::new();
        cache.put_history("repo-a", history_mau("aaa"));

        let a = cache.get_history("repo-a").unwrap();
        let b = cache.get_history("repo-a").unwrap();
        assert!(
            Arc::ptr_eq(&a, &b),
            "hai lần get phải trả cùng một Arc, không phải hai bản sao"
        );
    }

    /// Ghi lại cùng `RepoId` thì thay dữ liệu cũ, không thêm phần tử thứ hai.
    #[test]
    fn put_lai_cung_repo_id_thay_du_lieu_cu() {
        let cache = RepoCache::new();
        cache.put_history("repo-a", history_mau("aaa"));
        cache.put_history("repo-a", history_mau("bbb"));

        let got = cache.get_history("repo-a").unwrap();
        assert_eq!(got.commits[0].id, "bbb");
        assert_eq!(
            cache.cached_history_ids().len(),
            1,
            "ghi lại cùng khoá không được làm phình cache"
        );
    }

    /// Chặn trên số lịch sử được cache. ~67MB mỗi repo lớn và hạn 150MB của
    /// PROJECT.md nên hai là chặn trên, không phải con số tuỳ ý.
    #[test]
    fn khong_bao_gio_giu_nhieu_hon_max_cached_histories() {
        let cache = RepoCache::new();
        for i in 0..6 {
            cache.put_history(&format!("repo-{i}"), history_mau(&format!("c{i}")));
            assert!(
                cache.cached_history_ids().len() <= MAX_CACHED_HISTORIES,
                "sau {} lần put, cache giữ {} lịch sử — vượt chặn trên {}",
                i + 1,
                cache.cached_history_ids().len(),
                MAX_CACHED_HISTORIES
            );
        }
    }

    /// Khi vượt chặn trên, loại bỏ lịch sử của repo **không phải repo đang hoạt động**.
    /// Loại bỏ ngẫu nhiên có thể ném đúng repo người dùng đang xem, buộc nạp lại
    /// `git log --all` ngay lần cuộn tiếp theo.
    #[test]
    fn loai_bo_giu_lai_repo_dang_hoat_dong() {
        let cache = RepoCache::new();
        cache.put_history_keeping("repo-a", history_mau("aaa"), Some("repo-a"));
        cache.put_history_keeping("repo-b", history_mau("bbb"), Some("repo-a"));
        // Cache đầy (2). Thêm repo-c trong khi repo-a đang hoạt động.
        cache.put_history_keeping("repo-c", history_mau("ccc"), Some("repo-a"));

        assert!(
            cache.get_history("repo-a").is_some(),
            "repo đang hoạt động phải được giữ lại"
        );
        assert!(
            cache.get_history("repo-c").is_some(),
            "repo vừa nạp phải có trong cache"
        );
        assert!(
            cache.get_history("repo-b").is_none(),
            "repo không hoạt động và cũ nhất phải bị loại"
        );
    }

    /// Repo vừa `put` **chính là** repo đang hoạt động thì vẫn phải giữ nó và loại một
    /// repo khác. Một cài đặt "luôn giữ repo đang hoạt động" viết sơ sài có thể kết
    /// luận không có gì loại được rồi giữ cả ba.
    #[test]
    fn repo_vua_nap_cung_la_repo_dang_hoat_dong_thi_van_ton_trong_chan_tren() {
        let cache = RepoCache::new();
        cache.put_history_keeping("repo-a", history_mau("aaa"), None);
        cache.put_history_keeping("repo-b", history_mau("bbb"), None);
        cache.put_history_keeping("repo-c", history_mau("ccc"), Some("repo-c"));

        assert!(cache.get_history("repo-c").is_some());
        assert_eq!(
            cache.cached_history_ids().len(),
            MAX_CACHED_HISTORIES,
            "phải vẫn đúng chặn trên"
        );
    }

    /// `put_refs` **không** bị chặn bởi `MAX_CACHED_HISTORIES`: danh sách ref của một
    /// repo là vài KB, không phải hàng chục MB. Chặn nó lại sẽ làm thanh bên nhấp nháy
    /// mà không tiết kiệm đáng kể.
    #[test]
    fn cache_refs_khong_bi_chan_boi_gioi_han_lich_su() {
        let cache = RepoCache::new();
        for i in 0..5 {
            cache.put_refs(&format!("repo-{i}"), vec![ref_mau("main")]);
        }
        for i in 0..5 {
            assert!(
                cache.get_refs(&format!("repo-{i}")).is_some(),
                "refs của repo-{i} phải còn"
            );
        }
    }
}
