//! Cache lịch sử và ref theo `RepoId`.

use std::sync::Arc;

use crate::domain::{Commit, Ref};
use crate::graph::GraphRow;
use crate::state::RepoId;

/// Số lịch sử repo được giữ trong RAM cùng lúc.
pub const MAX_CACHED_HISTORIES: usize = 2;

/// Toàn bộ lịch sử đã nạp của một repository, cộng hình học đồ thị của nó.
pub struct RepoHistory {
    pub commits: Vec<Commit>,
    pub graph_rows: Vec<GraphRow>,
    pub loaded_at_ms: u64,
}

/// Cache khoá theo `RepoId`.
#[derive(Default)]
pub struct RepoCache {}

impl RepoCache {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_history(&self, _repo: &str) -> Option<Arc<RepoHistory>> {
        None
    }

    pub fn put_history(&self, _repo: &str, _h: RepoHistory) {}

    /// Giữ lịch sử cho `keep`, loại bỏ repo khác khi vượt [`MAX_CACHED_HISTORIES`].
    pub fn put_history_keeping(&self, _repo: &str, _h: RepoHistory, _keep: Option<&str>) {}

    pub fn get_refs(&self, _repo: &str) -> Option<Arc<Vec<Ref>>> {
        None
    }

    pub fn put_refs(&self, _repo: &str, _refs: Vec<Ref>) {}

    pub fn invalidate(&self, _repo: &str) {}

    pub fn cached_history_ids(&self) -> Vec<RepoId> {
        Vec::new()
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
