//! Cache diff khoá theo `(repo_id, sha, path)` — DIFF-01, T-03-12.
//!
//! # Vì sao **không** mở rộng [`RepoCache`](super::repo_cache::RepoCache)
//!
//! Hai bài toán khác nhau, không phải hai biến thể của một bài toán:
//!
//! | | `RepoCache` | `DiffCache` |
//! |---|---|---|
//! | cỡ một mục | ~67 MB (100k commit) | vài KB (hunk của một tệp) |
//! | số mục | 2 | 200 |
//! | quy tắc loại bỏ | phụ thuộc repo nào đang hiển thị — thông tin **ngoài** cache | thuần theo thứ tự truy cập |
//!
//! `RepoCache` phải tự viết vòng loại bỏ vì quy tắc của nó cần biết repo đang hiển
//! thị, thứ mà `lru` không diễn đạt được. Ở đây thì quy tắc đúng là LRU thuần, nên
//! dùng crate `lru` (đã có trong `Cargo.toml` từ Phase 1, dòng `lru = "0.18.4"`).
//!
//! # Cache này **không bao giờ cần vô hiệu hoá**
//!
//! Khoá gồm SHA commit, và **diff của một commit lịch sử là bất biến**: cùng một cặp
//! (commit, tệp) luôn cho cùng một bản vá, mãi mãi. Không có TTL, không watcher,
//! không `refresh`. Xem [`MAX_CACHED_DIFFS`] để biết vì sao thêm TTL "cho an toàn" là
//! một hồi quy chứ không phải một lớp phòng thủ.
//!
//! Ngoại lệ duy nhất là [`DiffCache::invalidate_repo`], và nó **không** phải vô hiệu
//! hoá theo thời gian: nó giải phóng RAM khi người dùng đóng repo.

use std::num::NonZeroUsize;
use std::sync::Arc;

use lru::LruCache;
use parking_lot::Mutex;

use crate::domain::diff::FileDiff;
use crate::state::RepoId;

/// Số mục diff giữ trong RAM cùng lúc.
///
/// # Con số này khác [`MAX_CACHED_HISTORIES`](super::MAX_CACHED_HISTORIES) hai bậc
/// độ lớn, và đó là đúng
///
/// Một mục ở đây là **các hunk đã phân tích của một tệp** — một `Vec<Hunk>` cỡ KB.
/// Một mục của `RepoCache` là toàn bộ lịch sử một repo, ~67 MB. Ràng buộc RAM 150 MB
/// của `PROJECT.md` cho phép 2 mục ở đó và 200 mục ở đây mà không mâu thuẫn.
///
/// Con số 200 đến từ ràng buộc ROADMAP ("cache diff theo LRU ~200 mục"). Nó rộng rãi
/// so với cách người ta thật sự đọc diff: một commit thường đổi dưới 20 tệp, nên 200
/// mục giữ được khoảng mười commit gần nhất — đủ để việc bấm qua lại giữa vài commit
/// không sinh lệnh git nào.
///
/// # Vì sao **không** thêm TTL
///
/// Diff của một commit lịch sử bất biến. Một TTL "cho an toàn" chỉ làm hai việc: sinh
/// thêm lệnh git cho dữ liệu không thể đổi, và làm tiêu chí thành công số 5 của phase
/// ("chọn đi chọn lại: hiện ra tức thì, không tính lại") **thỉnh thoảng** trượt — tức
/// loại lỗi khó tái hiện nhất. `DiffCache` cố ý không phơi ra `expire`/`refresh`/
/// `invalidate_stale`, và có một test khẳng định điều đó.
pub const MAX_CACHED_DIFFS: usize = 200;

/// Khoá của một mục diff.
///
/// # Là **struct**, không phải chuỗi nối — và đây là một lỗi thật, không phải lo xa
///
/// Một cài đặt `format!("{repo_id}{sha}{path}")` làm hai cặp khác nhau **trùng khoá**
/// khi ranh giới trường trôi:
///
/// ```text
/// ("r", "abc", "d")   ->  "rabcd"
/// ("r", "ab",  "cd")  ->  "rabcd"   ← cùng chuỗi, cùng mục cache
/// ```
///
/// Hệ quả: người dùng bấm tệp `cd` của commit `ab` và thấy diff của tệp `d` trong
/// commit `abc`. SHA thật dài 40 ký tự nên ca này hiếm khi xảy ra **ngẫu nhiên**, và
/// đó chính là lý do phải chặn bằng kiểu: một lỗi không bao giờ lộ ra khi dùng bình
/// thường thì cũng không bao giờ được sửa.
///
/// `repo_id` nằm trong khoá vì hai lý do: [`DiffCache::invalidate_repo`] cần nó, và
/// hai repo mở cùng lúc không được tráo dữ liệu cho nhau (PLAT-05).
///
/// # Nếu sau này thêm `-w`, cờ đó **phải** vào đây
///
/// `-w` (bỏ qua khoảng trắng khi **tính** diff) đổi **dữ liệu** — cùng một
/// `(sha, path)` cho hai tập hunk khác nhau. Không thêm vào khoá thì bật/tắt `-w` sẽ
/// trả về kết quả của lần trước.
///
/// Ngược lại, hai thứ **không** được vào khoá:
///
/// * chế độ hợp nhất / hai cột (DIFF-02) — cùng tập hunk, hai cách bố trí khi **vẽ**
/// * hiện ký tự khoảng trắng (DIFF-04) — cùng tập hunk, một lớp decoration khi **vẽ**
///
/// Đưa chúng vào khoá làm mỗi lần bấm đổi chế độ sinh một lệnh `git diff` mới, vi
/// phạm trực tiếp tiêu chí thành công số 5 của phase.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct DiffKey {
    pub repo_id: RepoId,
    pub sha: String,
    pub path: String,
}

impl DiffKey {
    pub fn new(repo_id: impl Into<String>, sha: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            repo_id: repo_id.into(),
            sha: sha.into(),
            path: path.into(),
        }
    }
}

/// Cache LRU cho diff đã phân tích.
///
/// Trả [`Arc<FileDiff>`] chứ không clone `FileDiff` — cùng khuôn với `RepoCache` trả
/// `Arc<RepoHistory>`. Một `FileDiff` của tệp lớn có hàng nghìn `DiffLine`, và clone
/// nó ở mỗi lần đọc biến một cache "tránh tính lại" thành một cache "tránh gọi git
/// nhưng vẫn sao chép hàng MB".
///
/// # Ghi chú cho người đọc nhật ký lệnh (T-02-15)
///
/// Cache hit **cố ý không** sinh dòng nào trong `CommandLog`: nhật ký ghi lệnh git đã
/// chạy, không ghi lời gọi IPC. Thấy `get_file_diff` mà không thấy `git diff` tương
/// ứng nghĩa là cache đã trả lời — đó là dấu hiệu đúng, không phải nhật ký thiếu. Test
/// tích hợp của Task 3 dùng chính điều này để chứng minh cache hoạt động.
pub struct DiffCache {
    inner: Mutex<LruCache<DiffKey, Arc<FileDiff>>>,
}

impl DiffCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(LruCache::new(
                NonZeroUsize::new(MAX_CACHED_DIFFS).expect("MAX_CACHED_DIFFS phải khác 0"),
            )),
        }
    }

    /// Lấy một mục và **đánh dấu nó vừa được dùng** (đẩy nó về cuối hàng loại bỏ).
    ///
    /// Đây là điểm phân biệt LRU với FIFO: một mục cũ nhưng vẫn được đọc phải sống
    /// lâu hơn một mục mới hơn mà không ai đọc.
    pub fn get_diff(&self, key: &DiffKey) -> Option<Arc<FileDiff>> {
        self.inner.lock().get(key).cloned()
    }

    pub fn put_diff(&self, key: DiffKey, diff: Arc<FileDiff>) {
        self.inner.lock().put(key, diff);
    }

    /// Giải phóng mọi mục của một repo — gọi từ `AppState::close_repo`.
    ///
    /// **Không** phải vô hiệu hoá theo thời gian (xem tài liệu đầu module): dữ liệu
    /// vẫn đúng, chỉ là không ai còn nhìn nó nữa. Giữ lại thì `MAX_CACHED_DIFFS` sẽ
    /// bảo vệ một thứ đã đóng, và các mục của repo đang mở bị loại bỏ sớm hơn cần
    /// thiết.
    pub fn invalidate_repo(&self, repo_id: &str) {
        let mut cache = self.inner.lock();
        let can_xoa: Vec<DiffKey> = cache
            .iter()
            .filter(|(k, _)| k.repo_id == repo_id)
            .map(|(k, _)| k.clone())
            .collect();
        for k in can_xoa {
            cache.pop(&k);
        }
    }

    /// Số mục đang giữ. Chỉ dùng cho test và chẩn đoán.
    pub fn len(&self) -> usize {
        self.inner.lock().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for DiffCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::diff::DiffKind;

    fn diff(path: &str) -> Arc<FileDiff> {
        Arc::new(FileDiff {
            path: path.to_owned(),
            old_path: None,
            status: "M".into(),
            kind: DiffKind::Unchanged,
        })
    }

    /// Ca cơ sở: đặt vào rồi lấy ra đúng mục đó.
    #[test]
    fn put_roi_get_tra_dung_muc() {
        let c = DiffCache::new();
        let k = DiffKey::new("repo-1", "abc123", "src/a.rs");

        assert!(c.get_diff(&k).is_none(), "cache rỗng phải trả None");

        c.put_diff(k.clone(), diff("src/a.rs"));
        let ra = c.get_diff(&k).expect("vừa đặt vào thì phải lấy ra được");
        assert_eq!(ra.path, "src/a.rs");
    }

    /// Khoá là **cặp ba trường**: cùng sha khác path là hai mục khác nhau, và ngược lại.
    #[test]
    fn khoa_phan_biet_ca_sha_lan_path() {
        let c = DiffCache::new();

        c.put_diff(DiffKey::new("r", "sha-1", "a.rs"), diff("a.rs"));
        c.put_diff(DiffKey::new("r", "sha-1", "b.rs"), diff("b.rs"));
        c.put_diff(DiffKey::new("r", "sha-2", "a.rs"), diff("a.rs@2"));

        assert_eq!(c.len(), 3, "ba cặp khác nhau là ba mục");
        assert_eq!(
            c.get_diff(&DiffKey::new("r", "sha-1", "a.rs")).unwrap().path,
            "a.rs"
        );
        assert_eq!(
            c.get_diff(&DiffKey::new("r", "sha-1", "b.rs")).unwrap().path,
            "b.rs",
            "cùng sha khác path phải là mục riêng"
        );
        assert_eq!(
            c.get_diff(&DiffKey::new("r", "sha-2", "a.rs")).unwrap().path,
            "a.rs@2",
            "cùng path khác sha phải là mục riêng"
        );
    }

    /// **Ca biên giới trường** — cổng chặn cài đặt nối chuỗi.
    ///
    /// `("abc", "d")` và `("ab", "cd")` nối lại cho **cùng một chuỗi** `"abcd"`. Với
    /// `DiffKey` là struct thì chúng là hai khoá khác nhau; với `format!` thì mục sau
    /// ghi đè mục trước và người dùng thấy diff của tệp khác.
    #[test]
    fn ranh_gioi_truong_khong_bi_nhap_nhang() {
        let c = DiffCache::new();

        let k1 = DiffKey::new("r", "abc", "d");
        let k2 = DiffKey::new("r", "ab", "cd");

        c.put_diff(k1.clone(), diff("tep-cua-abc"));
        c.put_diff(k2.clone(), diff("tep-cua-ab"));

        assert_eq!(
            c.len(),
            2,
            "hai cặp khác nhau phải là hai mục; nối chuỗi làm chúng trùng khoá"
        );
        assert_eq!(
            c.get_diff(&k1).unwrap().path,
            "tep-cua-abc",
            "mục đầu KHÔNG được bị mục sau ghi đè"
        );
        assert_eq!(c.get_diff(&k2).unwrap().path, "tep-cua-ab");

        // Cùng ranh giới, lần này giữa `repo_id` và `sha`.
        let c = DiffCache::new();
        let a = DiffKey::new("re", "po", "f.rs");
        let b = DiffKey::new("r", "epo", "f.rs");
        c.put_diff(a.clone(), diff("A"));
        c.put_diff(b.clone(), diff("B"));
        assert_eq!(c.len(), 2, "ranh giới repo_id/sha cũng không được nhập nhằng");
        assert_eq!(c.get_diff(&a).unwrap().path, "A");
        assert_eq!(c.get_diff(&b).unwrap().path, "B");
    }

    /// Vượt [`MAX_CACHED_DIFFS`] thì mục **ít dùng gần đây nhất** bị loại.
    ///
    /// Test chỉ kiểm số lượng, **không** kiểm mục nào bị loại — việc đó thuộc test
    /// LRU-vs-FIFO bên dưới. Tách ra để đột biến "đổi 200 thành 1" chỉ đỏ ở đây và
    /// người đọc biết ngay cổng nào bắt được nó.
    #[test]
    fn vuot_chan_tren_thi_loai_bo() {
        let c = DiffCache::new();

        for i in 0..(MAX_CACHED_DIFFS + 50) {
            c.put_diff(DiffKey::new("r", format!("sha-{i}"), "f.rs"), diff("f.rs"));
        }

        assert_eq!(
            c.len(),
            MAX_CACHED_DIFFS,
            "số mục không bao giờ vượt chặn trên — đó là lý do chặn trên tồn tại (T-03-12)"
        );
        assert!(
            c.get_diff(&DiffKey::new("r", "sha-0", "f.rs")).is_none(),
            "mục cũ nhất và không được đọc lại phải bị loại"
        );
        assert!(
            c.get_diff(&DiffKey::new(
                "r",
                format!("sha-{}", MAX_CACHED_DIFFS + 49),
                "f.rs"
            ))
            .is_some(),
            "mục vừa đặt vào phải còn"
        );
    }

    /// **LRU thật, không phải FIFO.** Một mục cũ ở *giữa* được `get` lại phải sống
    /// sót qua đợt loại bỏ, trong khi mục kế nó (cùng tuổi, không được đọc) thì không.
    ///
    /// FIFO loại theo thứ tự **đặt vào** nên nó sẽ loại cả hai. Đây là khác biệt duy
    /// nhất quan sát được giữa hai chiến lược, và nếu không kiểm nó thì `lru` có thể
    /// bị thay bằng một `VecDeque` mà mọi test khác vẫn xanh.
    #[test]
    fn la_lru_that_khong_phai_fifo() {
        let c = DiffCache::new();

        for i in 0..MAX_CACHED_DIFFS {
            c.put_diff(DiffKey::new("r", format!("sha-{i}"), "f.rs"), diff("f.rs"));
        }

        // Đọc lại một mục **cũ ở giữa**, làm nó thành mục vừa-dùng-gần-nhất.
        let cu_o_giua = DiffKey::new("r", "sha-5", "f.rs");
        assert!(
            c.get_diff(&cu_o_giua).is_some(),
            "tiền đề: mục sha-5 phải đang có trong cache"
        );

        // Đẩy thêm 10 mục mới, ép loại bỏ 10 mục.
        for i in 0..10 {
            c.put_diff(
                DiffKey::new("r", format!("moi-{i}"), "f.rs"),
                diff("moi.rs"),
            );
        }

        assert!(
            c.get_diff(&cu_o_giua).is_some(),
            "sha-5 vừa được ĐỌC nên phải sống sót — FIFO sẽ loại nó vì nó cũ"
        );
        assert!(
            c.get_diff(&DiffKey::new("r", "sha-6", "f.rs")).is_none(),
            "sha-6 cùng tuổi với sha-5 nhưng KHÔNG được đọc lại, nên phải bị loại. \
             Nếu cả hai cùng sống thì cache chưa loại bỏ gì cả và test trên mới là \
             cổng thật"
        );
    }

    /// `invalidate_repo` xoá đúng repo đó, **giữ** mục của repo khác (PLAT-05).
    #[test]
    fn invalidate_repo_chi_xoa_repo_do() {
        let c = DiffCache::new();

        c.put_diff(DiffKey::new("repo-a", "sha", "1.rs"), diff("1"));
        c.put_diff(DiffKey::new("repo-a", "sha", "2.rs"), diff("2"));
        c.put_diff(DiffKey::new("repo-b", "sha", "1.rs"), diff("3"));
        assert_eq!(c.len(), 3, "tiền đề: ba mục của hai repo");

        c.invalidate_repo("repo-a");

        assert_eq!(c.len(), 1, "chỉ còn mục của repo-b");
        assert!(c.get_diff(&DiffKey::new("repo-a", "sha", "1.rs")).is_none());
        assert!(c.get_diff(&DiffKey::new("repo-a", "sha", "2.rs")).is_none());
        assert!(
            c.get_diff(&DiffKey::new("repo-b", "sha", "1.rs")).is_some(),
            "mục của repo KHÁC không được đụng tới — đóng một repo không được làm \
             chậm repo còn lại"
        );
    }

    /// **Cổng chặn TTL.** `DiffCache` không được phơi ra hàm vô hiệu hoá theo thời
    /// gian hay theo watcher.
    ///
    /// Diff của một commit lịch sử bất biến; thêm TTL "cho an toàn" chỉ sinh lệnh git
    /// thừa và làm tiêu chí thành công số 5 thỉnh thoảng trượt. Test đọc **mã nguồn**
    /// vì Rust không cho hỏi "kiểu này có phương thức tên X không" lúc chạy.
    ///
    /// Lọc chú thích trước khi tìm: tệp này có doc comment tiếng Việt dày và chính
    /// những chữ đó xuất hiện trong phần giải thích vì sao không dùng chúng. Một cổng
    /// `contains` không lọc chú thích sẽ tự đỏ vì tài liệu của chính nó — đúng kiểu
    /// "cổng grep dễ vô dụng" mà Phase 2 ghi lại, chỉ là hỏng theo chiều ngược lại.
    #[test]
    fn khong_phoi_ra_ham_vo_hieu_hoa_theo_thoi_gian() {
        let src = include_str!("diff_cache.rs");

        let ma: String = src
            .lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("///") && !t.starts_with("//!")
            })
            .collect::<Vec<_>>()
            .join("\n");

        for cam in ["invalidate_stale", "fn refresh", "fn expire", "ttl", "Instant"] {
            assert!(
                !ma.contains(cam),
                "DiffCache không được có `{cam}`: diff của commit lịch sử là BẤT BIẾN, \
                 nên vô hiệu hoá theo thời gian chỉ thêm lệnh git cho dữ liệu không \
                 thể đổi. Xem tài liệu của MAX_CACHED_DIFFS."
            );
        }

        // Tiền đề: phép lọc trên phải giữ lại được mã thật, nếu không cổng luôn xanh
        // vì `ma` rỗng.
        assert!(
            ma.contains("pub fn invalidate_repo"),
            "phép lọc chú thích đã ăn mất cả mã — cổng trên sẽ luôn xanh vô nghĩa"
        );
    }

    /// Giá trị của hằng chặn trên, đọc thẳng. Không dùng `grep -c`: hằng số xuất hiện
    /// ở khai báo, ở chú thích giải thích con số, và ở test — đếm số lần khớp không
    /// nói được điều gì.
    ///
    /// Bản đầu của test này còn có một `assert!(MAX_CACHED_DIFFS > MAX_CACHED_HISTORIES)`.
    /// Clippy bắt nó là `assertions_on_constants`: **cả hai vế là hằng số nên phép so
    /// được tính lúc biên dịch** và assertion không quan sát được gì lúc chạy. Đó đúng
    /// là một "cổng tự vô hiệu hoá" theo nghĩa của bài học Phase 2, chỉ ở dạng Rust
    /// chứ không phải grep. Đã bỏ; quan hệ giữa hai con số nằm trong tài liệu của
    /// [`MAX_CACHED_DIFFS`], nơi nó thật sự được đọc.
    #[test]
    fn chan_tren_dung_hai_tram() {
        assert_eq!(
            MAX_CACHED_DIFFS, 200,
            "ràng buộc ROADMAP: cache diff theo LRU ~200 mục"
        );
    }

    /// Chặn trên phải là thứ cache **thật sự** tôn trọng, không chỉ một con số trong
    /// tệp nguồn.
    ///
    /// Đọc nó qua hành vi: nhồi gấp đôi chặn trên rồi đếm. Một cài đặt truyền nhầm
    /// một hằng khác vào `LruCache::new` sẽ qua được test đọc giá trị ở trên nhưng
    /// không qua được test này.
    #[test]
    fn cache_ton_trong_dung_gia_tri_hang_chan_tren() {
        let c = DiffCache::new();
        for i in 0..(MAX_CACHED_DIFFS * 2) {
            c.put_diff(DiffKey::new("r", format!("s{i}"), "f"), diff("f"));
        }
        assert_eq!(
            c.len(),
            MAX_CACHED_DIFFS,
            "sức chứa thật của cache phải bằng đúng MAX_CACHED_DIFFS"
        );
    }
}
