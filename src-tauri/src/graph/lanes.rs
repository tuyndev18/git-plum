//! Thuật toán gán lane — biến danh sách commit theo thứ tự topo thành hình học từng
//! dòng. **Trái tim của Core Value**: nếu tệp này sai, đồ thị sai ở mọi màn hình.
//!
//! # Thuật toán, bốn bước, chép từ `docs/01-research-competitors.md` mục 4.2
//!
//! ```text
//! Trạng thái: lanes = mảng, lanes[i] = mã commit mà lane i đang "chờ"
//!
//! Với mỗi commit C tại hàng r:
//!   1. Tìm lane đầu tiên đang chờ C  -> đó là lane của C.
//!      Không lane nào chờ C          -> cấp lane trống đầu tiên (đây là đầu nhánh).
//!   2. Mọi lane KHÁC cũng đang chờ C -> điểm hợp nhánh: vẽ đường chéo từ lane đó về
//!                                       lane của C, rồi giải phóng lane.
//!   3. Gán cha — LẶP qua toàn bộ danh sách cha:
//!      - parents[0]    -> tiếp tục chiếm lane của C.
//!      - parents[1..n] -> MỖI cha một lane trống mới + một đường chéo rẽ nhánh riêng.
//!      - không cha     -> giải phóng lane của C.
//!   4. Mọi lane khác đang hoạt động -> một đoạn thẳng dọc đi xuyên qua hàng r.
//! ```
//!
//! Độ phức tạp `O(n × số_lane_hoạt_động)`. Số lane hoạt động hiếm khi vượt 20 kể cả
//! trên repo thật (repo hiệu năng 100k commit của dự án đo được tối đa 21), nên thực tế
//! gần tuyến tính.
//!
//! # Ba chỗ dễ sai, đã có test riêng cho từng chỗ
//!
//! **Bước 3 phải là vòng lặp.** `for parent in commit.parents.iter().skip(1)`, không
//! bao giờ `parents[1]`. Merge octopus bốn cha phải sinh bốn cạnh, không phải hai; repo
//! hiệu năng của dự án chứa 320 merge từ ba cha trở lên. Đây là ràng buộc số 3 của
//! ROADMAP, và chính là lỗi mà công cụ của Microsoft từng mắc
//! (`.planning/research/PITFALLS.md` mục 2). Tên biến kiểu `parent1`/`parent2` bị cấm.
//!
//! **Cha ngoài tập đã nạp không phải cha.** Bản sao nông có commit biên khai báo một
//! cha không tồn tại trong dữ liệu. Cấp lane cho nó là cấp cho một commit không bao giờ
//! tới — lane đó rò rỉ vĩnh viễn và mọi hàng sau đều mang thêm một đường đi xuyên qua
//! ma. [`assign`] dựng một `HashSet` mã commit **một lần** trước vòng lặp và chỉ cấp
//! lane cho cha nằm trong tập; cha ngoài tập đặt [`GraphRow::terminates`].
//!
//! **Giới hạn hiển thị áp cho cha thêm, KHÔNG áp cho hàng.** Xem [`allocate_row_lane`]
//! và [`allocate_parent_lane`] — hai hàm, hai chữ ký, có chủ ý.
//!
//! # Vì sao KHÔNG dùng `rayon`
//!
//! `rayon` nằm trong danh sách "không dùng" của `.planning/research/STACK.md`, và lý do
//! ở đây mạnh hơn "không cần thiết": gán lane **vốn tuần tự**. Trạng thái `lanes` ở
//! hàng `r` là kết quả của toàn bộ hàng `0..r`; không có cách nào chia đôi danh sách mà
//! nửa sau biết được lane nào đang trống. Song song hoá không phải vô ích — nó **sai về
//! mặt thuật toán**. Người đọc thấy vòng lặp nóng chạy 100k lần và nghĩ tới `par_iter`
//! nên dừng lại ở dòng này.
//!
//! # Hàm thuần
//!
//! Không IO, không async, không `Mutex`, không biết gì về Tauri.
//! `.planning/research/ARCHITECTURE.md` mục "graph/lanes.rs isolated and IO-free" là
//! ràng buộc, không gợi ý: nhờ vậy toàn bộ thuật toán kiểm được bằng dữ liệu dựng tay,
//! và benchmark của criterion đo được đúng nó chứ không đo lẫn thời gian chạy git.

use std::collections::HashSet;

use crate::domain::Commit;
use crate::graph::types::{Edge, GraphRow, LANE_COLORS, MAX_VISIBLE_LANES};

/// Màu của một lane. Hợp đồng với bộ vẽ: `color == lane % LANE_COLORS` ở mọi nơi.
#[inline]
fn color_of(lane: u16) -> u8 {
    (lane % LANE_COLORS as u16) as u8
}

/// Cấp lane cho **chính hàng** (bước 1). **KHÔNG BAO GIỜ giới hạn** — luôn trả một lane.
///
/// # Vì sao hàm này không trả `Option`
///
/// Mỗi commit trong đầu vào **phải** nhận được một lane, bất kể [`MAX_VISIBLE_LANES`]
/// là bao nhiêu. Bất biến `rows.len() == commits.len()` là HIST-04 ở tầng dữ liệu và nó
/// không được phụ thuộc vào một hằng số *hiển thị*.
///
/// Dùng chung **một** hàm trả `Option` cho cả bước 1 và bước 3 là nguyên nhân gốc của
/// một lỗi mất hàng: bước 1 nhận `None` thì không có cách xử lý đúng nào — `unwrap()`
/// panic, `continue` âm thầm đánh rơi một commit, gán một lane ngoài giới hạn phá chính
/// assertion mà cap dựng ra. Trên repo mẫu `wide` (25 lane đồng thời) với giới hạn 20,
/// cài đặt dùng chung một hàm đánh rơi 10 trong 73 hàng — trong khi `linear`, `octopus`
/// và `orphan` vẫn xanh hết vì chúng không bao giờ chạm cap. Đúng dạng lỗi "đồ thị sai
/// ship ra trong khi trông đúng trên repo nhỏ".
///
/// Trả **lane trống có chỉ số nhỏ nhất**, không phải một lane mới ở cuối mảng: thiếu
/// điều này thì mảng `lanes` chỉ dài ra mãi và đồ thị loãng dần trên lịch sử dài.
fn allocate_row_lane(lanes: &mut Vec<Option<String>>, id: &str) -> u16 {
    if let Some(i) = lanes.iter().position(|l| l.is_none()) {
        lanes[i] = Some(id.to_string());
        return i as u16;
    }
    lanes.push(Some(id.to_string()));
    (lanes.len() - 1) as u16
}

/// Cấp lane cho **một cha thêm** (bước 3). **CÓ giới hạn** — trả `None` khi đã đầy.
///
/// Đây là chỗ neo của cách vẽ suy giảm có chủ ý mà ROADMAP ràng buộc số 4 đòi: khi
/// không còn lane nào dưới [`MAX_VISIBLE_LANES`], cha đó không được cấp lane và hàng
/// tăng [`GraphRow::truncated_parents`] để giao diện hiện chỉ báo `+N cha nữa` thay vì
/// vẽ tràn ra ngoài cột.
///
/// Khác [`allocate_row_lane`] đúng ở một điểm: ở đây `None` là một câu trả lời **đúng**
/// và gọi được — cha không vẽ vẫn còn nguyên trong `Commit.parents`, chỉ đường kẻ bị
/// lược. Suy giảm **hiển thị**, không phải suy giảm **dữ liệu**.
fn allocate_parent_lane(lanes: &mut Vec<Option<String>>, id: &str) -> Option<u16> {
    if let Some(i) = lanes.iter().position(|l| l.is_none()) {
        if i >= MAX_VISIBLE_LANES as usize {
            return None;
        }
        lanes[i] = Some(id.to_string());
        return Some(i as u16);
    }
    if lanes.len() >= MAX_VISIBLE_LANES as usize {
        return None;
    }
    lanes.push(Some(id.to_string()));
    Some((lanes.len() - 1) as u16)
}

/// Gán lane cho một danh sách commit **theo thứ tự topo** và sinh hình học từng dòng.
///
/// Trả đúng một [`GraphRow`] cho mỗi [`Commit`], cùng thứ tự, cùng độ dài — bất biến
/// này không có ngoại lệ, kể cả khi số nhánh sống vượt [`MAX_VISIBLE_LANES`].
///
/// # Đầu vào phải theo `--topo-order`
///
/// Thuật toán giả định **cha luôn xuất hiện sau con**. `LOG_ARGS` của
/// `crate::git::parsers::log` ghim `--topo-order` đúng vì lý do này. Thứ tự theo thời
/// gian không dùng được: rebase, cherry-pick và đồng hồ lệch đều sinh ra cha *mới hơn*
/// con. Đầu vào sai thứ tự không làm hàm sập — nó vẫn trả đủ hàng — nhưng đường nối sẽ
/// trỏ ngược.
///
/// # Không thể lặp vô hạn
///
/// Vòng lặp đi đúng một lượt qua `commits` và không bao giờ đi ngược, nên dữ liệu có
/// vòng (một điều git không tạo ra nhưng một repo thù địch có thể chứa) cũng chỉ làm
/// đường nối vô nghĩa, không treo ứng dụng (T-02-08).
pub fn assign(commits: &[Commit]) -> Vec<GraphRow> {
    // Tập mã commit CÓ trong dữ liệu này. Dựng một lần, O(n).
    //
    // Đây là thứ phân biệt "commit gốc thật" với "biên bản sao nông": cả hai đều trông
    // như điểm cuối, nhưng gốc thật có `parents` rỗng còn biên nông khai báo một cha
    // mà ta không có. Thiếu tập này thì lane cấp cho cha đó không bao giờ được thu hồi.
    let known: HashSet<&str> = commits.iter().map(|c| c.id.as_str()).collect();

    // lanes[i] = mã commit mà lane i đang chờ; None là lane trống.
    let mut lanes: Vec<Option<String>> = Vec::new();
    let mut rows: Vec<GraphRow> = Vec::with_capacity(commits.len());

    for commit in commits {
        let id = commit.id.as_str();

        // --- Bước 1: lane của hàng. Lane đầu tiên đang chờ commit này, hoặc lane
        // trống nhỏ nhất nếu không ai chờ (đây là đầu nhánh). KHÔNG giới hạn.
        let lane = match lanes.iter().position(|l| l.as_deref() == Some(id)) {
            Some(i) => i as u16,
            None => allocate_row_lane(&mut lanes, id),
        };
        let color = color_of(lane);

        let mut passthrough: Vec<Edge> = Vec::new();
        let mut out_edges: Vec<Edge> = Vec::new();

        // --- Bước 2: mọi lane KHÁC cũng đang chờ commit này là một điểm hợp nhánh.
        // Đường chéo từ lane đó về lane của hàng, rồi giải phóng lane.
        //
        // Đường chéo mang màu của lane **nguồn**: nó là phần đuôi của nhánh đang chết,
        // và tô nó theo màu đích sẽ làm nhánh như đổi màu giữa chừng.
        for (i, slot) in lanes.iter_mut().enumerate() {
            if i as u16 == lane {
                continue;
            }
            if slot.as_deref() == Some(id) {
                passthrough.push(Edge {
                    from_lane: i as u16,
                    to_lane: lane,
                    color: color_of(i as u16),
                });
                *slot = None;
            }
        }

        // --- Bước 3: gán cha. LẶP qua toàn bộ danh sách — ràng buộc số 3 của ROADMAP.
        let mut truncated_parents: u16 = 0;
        let mut terminates = false;

        match commit.parents.first() {
            // Cha đầu tiếp tục chiếm lane của hàng này.
            Some(first) if known.contains(first.as_str()) => {
                lanes[lane as usize] = Some(first.clone());
                out_edges.push(Edge {
                    from_lane: lane,
                    to_lane: lane,
                    color,
                });
            }
            // Cha đầu KHÔNG có trong tập đã nạp: bản sao nông hoặc trang chưa nạp tới.
            // Lane kết thúc ở đây, và cờ cho frontend vẽ "còn tiếp" thay vì dấu hết.
            Some(_) => {
                lanes[lane as usize] = None;
                terminates = true;
            }
            // Không cha: commit gốc THẬT. Giải phóng lane, `terminates` giữ false.
            None => {
                lanes[lane as usize] = None;
            }
        }

        // Cha thứ hai trở đi: MỖI cha một lane mới và một đường chéo rẽ nhánh riêng.
        // `.skip(1)` chứ không `parents[1]` — bốn cha phải cho bốn cạnh.
        for parent in commit.parents.iter().skip(1) {
            if !known.contains(parent.as_str()) {
                // Cha ngoài tập: không cấp lane, nhưng hàng vẫn "còn tiếp".
                terminates = true;
                continue;
            }
            // Cha này đã có lane đang chờ nó rồi (hai nhánh cùng trỏ về một commit)?
            // Nối vào lane đó thay vì cấp thêm một lane trùng — nếu không, hai lane
            // cùng chờ một commit và bước 2 phải dọn, làm đồ thị rộng ra vô cớ.
            if let Some(i) = lanes
                .iter()
                .position(|l| l.as_deref() == Some(parent.as_str()))
            {
                out_edges.push(Edge {
                    from_lane: lane,
                    to_lane: i as u16,
                    color: color_of(i as u16),
                });
                continue;
            }
            match allocate_parent_lane(&mut lanes, parent) {
                Some(moi) => out_edges.push(Edge {
                    from_lane: lane,
                    to_lane: moi,
                    color: color_of(moi),
                }),
                // Hết lane vẽ được: cha vẫn còn trong `Commit.parents`, chỉ đường kẻ
                // bị lược. Giao diện hiện `+N cha nữa`.
                None => truncated_parents += 1,
            }
        }

        // --- Bước 4: mọi lane khác đang hoạt động đi thẳng xuyên qua hàng này.
        for (i, slot) in lanes.iter().enumerate() {
            if i as u16 == lane || slot.is_none() {
                continue;
            }
            passthrough.push(Edge {
                from_lane: i as u16,
                to_lane: i as u16,
                color: color_of(i as u16),
            });
        }

        // Cắt đuôi các lane trống ở cuối mảng: giữ `lanes.len()` bằng số lane thật sự
        // đang sống. Thiếu bước này thì bước 4 phải quét qua một cái đuôi dài mãi trên
        // lịch sử có nhiều nhánh đã chết, biến O(n × lane_sống) thành
        // O(n × lane_từng_sống) — khác biệt thật trên repo 100k commit.
        while matches!(lanes.last(), Some(None)) {
            lanes.pop();
        }

        rows.push(GraphRow {
            commit_id: commit.id.clone(),
            lane,
            color,
            passthrough,
            out_edges,
            truncated_parents,
            terminates,
        });
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::types::{Edge, LANE_COLORS, MAX_VISIBLE_LANES};
    use std::collections::HashSet;

    /// Dựng một `Commit` tối thiểu: chỉ `id` và `parents` ảnh hưởng tới gán lane.
    ///
    /// Mọi trường còn lại là dữ liệu hiển thị và thuật toán không được đọc tới. Nếu
    /// một ngày nào đó `assign` bắt đầu phụ thuộc vào `author_time` hay `subject` thì
    /// nó đã thôi là hàm thuần về hình dạng đồ thị — helper này giữ ranh giới đó rõ.
    fn commit(id: &str, parents: &[&str]) -> Commit {
        Commit {
            id: id.to_string(),
            parents: parents.iter().map(|p| p.to_string()).collect(),
            author_name: "T".into(),
            author_email: "t@example.com".into(),
            author_time: 0,
            committer_name: "T".into(),
            committer_email: "t@example.com".into(),
            committer_time: 0,
            subject: "s".into(),
            body: String::new(),
            has_invalid_utf8: false,
        }
    }

    /// Bất biến HIST-04 ở tầng dữ liệu, kiểm ở **mọi** test: một hàng cho mỗi commit,
    /// không hơn không kém, và hàng thứ `i` đúng là commit thứ `i`.
    fn khang_dinh_mot_hang_moi_commit(commits: &[Commit], rows: &[GraphRow]) {
        assert_eq!(
            rows.len(),
            commits.len(),
            "mất hoặc thừa hàng: {} commit vào, {} hàng ra",
            commits.len(),
            rows.len()
        );
        for (i, (c, r)) in commits.iter().zip(rows).enumerate() {
            assert_eq!(r.commit_id, c.id, "hàng {i} không khớp commit {i}");
        }
    }

    /// `color == lane % LANE_COLORS` ở mọi hàng — hợp đồng với bộ vẽ.
    fn khang_dinh_mau_khop_lane(rows: &[GraphRow]) {
        for (i, r) in rows.iter().enumerate() {
            assert_eq!(
                r.color,
                (r.lane % LANE_COLORS as u16) as u8,
                "hàng {i}: màu {} không khớp lane {}",
                r.color,
                r.lane
            );
        }
    }

    fn lanes_cua(edges: &[Edge]) -> Vec<u16> {
        let mut v: Vec<u16> = edges.iter().map(|e| e.to_lane).collect();
        v.sort_unstable();
        v
    }

    // ---------------------------------------------------------------- ca cơ sở

    #[test]
    fn dau_vao_rong_cho_vec_rong() {
        let rows = assign(&[]);
        assert!(rows.is_empty(), "không commit nào thì không hàng nào");
    }

    #[test]
    fn mot_commit_duy_nhat_lane_0() {
        let commits = vec![commit("a", &[])];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        assert_eq!(rows[0].lane, 0);
        assert_eq!(rows[0].color, 0);
        assert!(rows[0].out_edges.is_empty(), "commit gốc không có cạnh ra");
        assert!(!rows[0].terminates, "commit gốc THẬT: terminates == false");
    }

    /// Lịch sử tuyến tính ba commit: tất cả ở lane 0, không có đường đi xuyên qua, và
    /// mỗi hàng trừ hàng cuối có đúng một cạnh ra thẳng đứng.
    #[test]
    fn lich_su_tuyen_tinh_ba_commit_deu_o_lane_0() {
        let commits = vec![commit("c", &["b"]), commit("b", &["a"]), commit("a", &[])];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        for (i, r) in rows.iter().enumerate() {
            assert_eq!(r.lane, 0, "hàng {i} phải ở lane 0");
            assert_eq!(r.color, 0, "hàng {i} phải màu 0");
            assert!(
                r.passthrough.is_empty(),
                "hàng {i}: lịch sử một nhánh không có đường đi xuyên qua"
            );
            assert_eq!(r.truncated_parents, 0);
            assert!(!r.terminates, "hàng {i}: mọi cha đều có trong tập");
        }

        for (i, r) in rows.iter().take(2).enumerate() {
            assert_eq!(r.out_edges.len(), 1, "hàng {i} phải có đúng một cạnh ra");
            assert_eq!(r.out_edges[0].from_lane, 0);
            assert_eq!(r.out_edges[0].to_lane, 0);
        }
        assert!(rows[2].out_edges.is_empty(), "commit gốc không có cạnh ra");
    }

    /// Commit gốc **giải phóng** lane của nó. Không có bước này thì lane rò rỉ và
    /// commit độc lập đứng sau nó trong danh sách sẽ bị hút vào lane đã chết.
    #[test]
    fn commit_goc_giai_phong_lane() {
        // "a" là gốc ở hàng 1; "z" ở hàng 2 hoàn toàn độc lập.
        let commits = vec![commit("b", &["a"]), commit("a", &[]), commit("z", &[])];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        assert_eq!(rows[1].lane, 0, "gốc 'a' ở lane 0");
        assert_eq!(
            rows[2].lane, 0,
            "lane 0 đã được giải phóng nên 'z' tái dùng được nó"
        );
        assert!(
            rows[2].passthrough.is_empty(),
            "không lane nào còn sống sau khi gốc giải phóng lane 0"
        );
        // Và quan trọng nhất: không có cạnh nào nối 'a' xuống 'z'.
        assert!(
            rows[1].out_edges.is_empty(),
            "gốc không được nối xuống commit sau nó trong danh sách"
        );
    }

    // ---------------------------------------------------------- rẽ nhánh, hợp nhánh

    /// Hai con cùng một cha: con thứ hai lấy lane mới; tại hàng của cha hai lane hợp
    /// về một và lane thừa được giải phóng.
    #[test]
    fn re_nhanh_don_hai_con_mot_cha_hop_lai() {
        let commits = vec![commit("c1", &["p"]), commit("c2", &["p"]), commit("p", &[])];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(rows[0].lane, 0, "con thứ nhất ở lane 0");
        assert_eq!(
            rows[1].lane, 1,
            "con thứ hai phải lấy lane MỚI, không trùng"
        );
        assert_eq!(rows[2].lane, 0, "cha nhận lane thấp nhất đang chờ nó");

        // Hàng của cha: lane 1 hợp về lane 0 bằng một đường chéo.
        let hop: Vec<&Edge> = rows[2]
            .passthrough
            .iter()
            .chain(rows[2].out_edges.iter())
            .filter(|e| e.from_lane == 1 && e.to_lane == 0)
            .collect();
        assert_eq!(
            hop.len(),
            1,
            "phải có đúng một đường chéo hợp nhánh từ lane 1 về lane 0, \
             thấy passthrough={:?} out={:?}",
            rows[2].passthrough,
            rows[2].out_edges
        );

        // Cha là gốc nên sau hàng đó không lane nào còn sống.
        assert!(rows[2].out_edges.is_empty(), "cha là gốc, không có cạnh ra");
    }

    /// Merge **hai** cha: đúng hai cạnh ra — một giữ lane, một sang lane mới.
    #[test]
    fn merge_hai_cha_cho_dung_hai_canh_ra() {
        let commits = vec![
            commit("m", &["p1", "p2"]),
            commit("p1", &[]),
            commit("p2", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(rows[0].lane, 0);
        assert_eq!(rows[0].out_edges.len(), 2, "merge hai cha → hai cạnh ra");
        assert_eq!(rows[0].truncated_parents, 0);
        assert_eq!(
            lanes_cua(&rows[0].out_edges),
            vec![0, 1],
            "một cạnh giữ lane 0, một cạnh sang lane mới 1"
        );
        for e in &rows[0].out_edges {
            assert_eq!(
                e.from_lane, 0,
                "mọi cạnh ra đều xuất phát từ lane của merge"
            );
        }
    }

    /// **Test quan trọng nhất về octopus.** Merge bốn cha sinh đúng bốn cạnh ra: một
    /// giữ lane và **ba** lane mới. Cài đặt chỉ đọc `parents[1]` cho hai cạnh sẽ đỏ
    /// ngay ở đây.
    #[test]
    fn merge_octopus_bon_cha_cho_dung_bon_canh_ra() {
        let commits = vec![
            commit("m", &["p1", "p2", "p3", "p4"]),
            commit("p1", &[]),
            commit("p2", &[]),
            commit("p3", &[]),
            commit("p4", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(
            rows[0].out_edges.len(),
            4,
            "merge BỐN cha phải cho BỐN cạnh ra, không phải hai — \
             cài đặt chỉ đọc parents[1] sẽ cho 2"
        );
        assert_eq!(rows[0].truncated_parents, 0, "bốn lane thừa sức vẽ hết");
        assert_eq!(
            lanes_cua(&rows[0].out_edges),
            vec![0, 1, 2, 3],
            "một cạnh giữ lane 0, ba cạnh sang ba lane mới riêng biệt"
        );

        // Bốn cha phải đáp xuống bốn lane khác nhau, mỗi cha đúng một lane.
        let mut lanes_cha: Vec<u16> = rows[1..5].iter().map(|r| r.lane).collect();
        lanes_cha.sort_unstable();
        assert_eq!(
            lanes_cha,
            vec![0, 1, 2, 3],
            "mỗi cha của octopus phải đáp xuống lane riêng mà merge đã cấp"
        );
    }

    /// Merge **mười lăm** cha: hợp pháp nhưng phi lý. Không panic, và sổ sách phải cân
    /// — `out_edges.len() + truncated_parents == 15`. Không được mất cha nào trong
    /// sổ sách dù không vẽ hết (T-02-08).
    #[test]
    fn merge_muoi_lam_cha_khong_mat_cha_nao_trong_so_sach() {
        let ten_cha: Vec<String> = (0..15).map(|i| format!("p{i}")).collect();
        let cha_ref: Vec<&str> = ten_cha.iter().map(|s| s.as_str()).collect();

        let mut commits = vec![commit("m", &cha_ref)];
        for p in &cha_ref {
            commits.push(commit(p, &[]));
        }

        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        let so_sach = rows[0].out_edges.len() + rows[0].truncated_parents as usize;
        assert_eq!(
            so_sach,
            15,
            "sổ sách không cân: {} cạnh vẽ + {} cha bị cắt != 15 cha",
            rows[0].out_edges.len(),
            rows[0].truncated_parents
        );
    }

    // ------------------------------------------------------- bản sao nông, mồ côi

    /// **Bản sao nông.** Cha không có trong tập đã nạp → `terminates == true`, lane
    /// được giải phóng, không panic, không lặp vô hạn (T-02-09).
    #[test]
    fn cha_khong_co_trong_tap_da_nap_dat_terminates() {
        let commits = vec![commit("bien", &["deadbeef"]), commit("doc_lap", &[])];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);

        assert!(
            rows[0].terminates,
            "cha 'deadbeef' không có trong tập đã nạp → terminates phải true"
        );
        assert!(
            rows[0].out_edges.is_empty(),
            "không vẽ cạnh tới một commit sẽ không bao giờ xuất hiện"
        );
        assert_eq!(
            rows[1].lane, 0,
            "lane của hàng biên phải được giải phóng, không rò rỉ vĩnh viễn"
        );
        assert!(
            rows[1].passthrough.is_empty(),
            "lane rò rỉ sẽ hiện ra ở đây dưới dạng một đường đi xuyên qua thừa"
        );
    }

    /// Phân biệt gốc thật với biên nông: hình dạng dữ liệu gần giống nhau nhưng ý
    /// nghĩa ngược nhau. Vẽ biên nông như gốc là nói dối rằng lịch sử kết thúc ở đó.
    #[test]
    fn goc_that_va_bien_nong_khac_nhau_o_terminates() {
        let commits = vec![commit("goc", &[]), commit("bien", &["khong_ton_tai"])];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        assert!(!rows[0].terminates, "gốc THẬT: terminates == false");
        assert!(rows[1].terminates, "biên nông: terminates == true");
        assert!(rows[0].out_edges.is_empty());
        assert!(rows[1].out_edges.is_empty());
    }

    /// **Nhánh mồ côi.** Hai chuỗi không chia sẻ cha nào, xen kẽ trong đầu vào: mỗi
    /// chuỗi ra lane riêng và không có cạnh nào nối giữa hai chuỗi.
    #[test]
    fn nhanh_mo_coi_hai_chuoi_khong_noi_voi_nhau() {
        // Xen kẽ có chủ ý: a1, b1, a2, b2, a3(gốc), b3(gốc).
        let commits = vec![
            commit("a1", &["a2"]),
            commit("b1", &["b2"]),
            commit("a2", &["a3"]),
            commit("b2", &["b3"]),
            commit("a3", &[]),
            commit("b3", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        let lane_a = rows[0].lane;
        let lane_b = rows[1].lane;
        assert_ne!(lane_a, lane_b, "hai chuỗi mồ côi phải ở hai lane khác nhau");

        // Chuỗi a giữ nguyên lane của nó suốt, chuỗi b cũng vậy.
        assert_eq!(rows[2].lane, lane_a, "a2 phải nối tiếp lane của a1");
        assert_eq!(rows[3].lane, lane_b, "b2 phải nối tiếp lane của b1");
        assert_eq!(rows[4].lane, lane_a, "a3 phải nối tiếp lane của a2");
        assert_eq!(rows[5].lane, lane_b, "b3 phải nối tiếp lane của b2");

        // Không cạnh nào bắc cầu giữa hai lane.
        for (i, r) in rows.iter().enumerate() {
            for e in &r.out_edges {
                assert_eq!(
                    e.from_lane, e.to_lane,
                    "hàng {i}: hai chuỗi mồ côi không được có cạnh chéo nối nhau"
                );
            }
        }
    }

    /// **Hai gốc không liên quan hợp nhất** (`merge --allow-unrelated-histories`):
    /// đúng hai cạnh ra, và không lane nào bị bỏ sót.
    #[test]
    fn hai_goc_khong_lien_quan_hop_nhat() {
        let commits = vec![
            commit("m", &["x1", "y1"]),
            commit("x1", &["x2"]),
            commit("y1", &["y2"]),
            commit("x2", &[]),
            commit("y2", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(
            rows[0].out_edges.len(),
            2,
            "merge hai cây rời → hai cạnh ra"
        );
        let max_lane = rows.iter().map(|r| r.lane).max().unwrap();
        assert!(
            max_lane <= 1,
            "hai cây rời chỉ cần hai lane, thấy lane cao nhất {max_lane}"
        );
        for r in &rows {
            assert!(!r.terminates, "mọi cha đều có trong tập, không ca nông nào");
        }
    }

    /// **`passthrough`.** Khi một nhánh khác đang sống mà hàng này không thuộc về nó,
    /// hàng đó phải mang một `Edge` đi thẳng qua với `from_lane == to_lane`.
    #[test]
    fn hang_co_nhanh_khac_dang_song_sinh_passthrough() {
        // m mở hai lane; hàng x1 ở lane 0 trong khi lane 1 (chờ y1) vẫn sống.
        let commits = vec![
            commit("m", &["x1", "y1"]),
            commit("x1", &["x2"]),
            commit("y1", &[]),
            commit("x2", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);

        assert_eq!(rows[1].commit_id, "x1");
        assert_eq!(rows[1].lane, 0);
        assert_eq!(
            rows[1].passthrough.len(),
            1,
            "lane 1 (đang chờ y1) phải đi thẳng xuyên qua hàng x1, thấy {:?}",
            rows[1].passthrough
        );
        let p = rows[1].passthrough[0];
        assert_eq!(p.from_lane, 1);
        assert_eq!(p.to_lane, 1, "đường đi xuyên qua phải thẳng đứng");
        assert_eq!(
            p.color,
            (1 % LANE_COLORS as u16) as u8,
            "đường đi xuyên qua mang màu của lane nó chạy trên"
        );

        // Hàng của chính lane 1 thì không tự đi xuyên qua mình.
        assert_eq!(rows[2].commit_id, "y1");
        assert!(
            rows[2]
                .passthrough
                .iter()
                .all(|e| e.from_lane != rows[2].lane),
            "hàng không được passthrough chính lane của nó"
        );
    }

    /// **Tái dùng lane.** Sau khi một lane được giải phóng, commit tiếp theo cần lane
    /// mới phải lấy **lane trống có chỉ số nhỏ nhất**, không cấp thêm ở cuối mảng.
    /// Thiếu điều này thì đồ thị loãng dần và số lane tăng vô hạn trên repo dài.
    #[test]
    fn tai_dung_lane_trong_co_chi_so_nho_nhat() {
        // m mở lane 0,1,2. y1 (lane 1) là gốc nên lane 1 chết ở hàng 2.
        // Sau đó 'moi' là đầu nhánh mới: phải lấy lane 1, không phải lane 3.
        let commits = vec![
            commit("m", &["x1", "y1", "z1"]),
            commit("x1", &["x2"]),
            commit("y1", &[]),
            commit("moi", &[]),
            commit("z1", &[]),
            commit("x2", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(rows[2].commit_id, "y1");
        let lane_da_chet = rows[2].lane;

        assert_eq!(rows[3].commit_id, "moi");
        assert_eq!(
            rows[3].lane, lane_da_chet,
            "'moi' phải tái dùng lane {lane_da_chet} vừa giải phóng, \
             không cấp lane mới ở cuối mảng (thấy lane {})",
            rows[3].lane
        );

        let max_lane = rows.iter().map(|r| r.lane).max().unwrap();
        assert_eq!(
            max_lane, 2,
            "ba nhánh sống đồng thời chỉ cần ba lane; lane cao hơn nghĩa là đồ thị loãng"
        );
    }

    // ------------------------------------------------------- giới hạn hiển thị

    /// **Giới hạn ÁP cho cha thêm.** Một merge có nhiều cha hơn số lane còn trống:
    /// `truncated_parents > 0` và mọi `Edge.to_lane < MAX_VISIBLE_LANES`.
    #[test]
    fn gioi_han_lane_ap_cho_cha_them() {
        // Merge có MAX_VISIBLE_LANES + 5 cha: không thể cấp lane cho tất cả.
        let so_cha = MAX_VISIBLE_LANES as usize + 5;
        let ten_cha: Vec<String> = (0..so_cha).map(|i| format!("p{i}")).collect();
        let cha_ref: Vec<&str> = ten_cha.iter().map(|s| s.as_str()).collect();

        let mut commits = vec![commit("m", &cha_ref)];
        for p in &cha_ref {
            commits.push(commit(p, &[]));
        }

        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);

        assert!(
            rows[0].truncated_parents > 0,
            "merge {so_cha} cha với giới hạn {MAX_VISIBLE_LANES} lane phải cắt bớt"
        );
        assert_eq!(
            rows[0].out_edges.len() + rows[0].truncated_parents as usize,
            so_cha,
            "sổ sách phải cân kể cả khi cắt"
        );
        // Cạnh **rẽ nhánh** không bao giờ được cấp một lane vượt giới hạn. Khẳng định
        // hẹp đúng chỗ: cạnh *giữ lane* của một hàng vốn đã vượt giới hạn thì được
        // phép vượt theo — xem ghi chú dài ở assertion (3) của
        // `tests/graph_fixtures.rs`, nơi bản khẳng định rộng hơn đã đỏ trên `wide`.
        for (i, r) in rows.iter().enumerate() {
            for e in &r.out_edges {
                if e.to_lane != r.lane {
                    assert!(
                        e.to_lane < MAX_VISIBLE_LANES,
                        "hàng {i}: cạnh rẽ nhánh tới lane {} vượt giới hạn hiển thị {}",
                        e.to_lane,
                        MAX_VISIBLE_LANES
                    );
                }
            }
        }
    }

    /// **Giới hạn KHÔNG áp cho hàng — test quan trọng nhất của cả plan.**
    ///
    /// 40 nhánh sống đồng thời trong khi `MAX_VISIBLE_LANES` là 20. Không hàng nào
    /// được bỏ, không panic, và **phải** tồn tại hàng có `lane >= MAX_VISIBLE_LANES`:
    /// lane của hàng được phép vượt giới hạn hiển thị, việc gập là của frontend.
    ///
    /// Không có test này thì một cài đặt **bỏ hàng khi hết lane** sẽ đỗ mọi test khác,
    /// vì các fixture nhỏ không bao giờ đụng giới hạn. Đúng dạng lỗi "đồ thị sai ship
    /// ra trong khi trông đúng trên repo nhỏ".
    #[test]
    fn gioi_han_lane_khong_ap_cho_hang_khong_bo_hang_nao() {
        const SO_NHANH: usize = 40;
        assert!(
            SO_NHANH > MAX_VISIBLE_LANES as usize,
            "test này chỉ có ý nghĩa khi số nhánh vượt giới hạn hiển thị"
        );

        // 40 đầu nhánh độc lập, mỗi nhánh hai commit → 40 lane sống cùng lúc.
        let mut commits = Vec::new();
        for i in 0..SO_NHANH {
            commits.push(commit(&format!("head{i}"), &[&format!("tail{i}")]));
        }
        for i in 0..SO_NHANH {
            commits.push(commit(&format!("tail{i}"), &[]));
        }

        let rows = assign(&commits);

        // Bất biến số một: KHÔNG hàng nào bị bỏ.
        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        // Lane của hàng ĐƯỢC PHÉP vượt giới hạn hiển thị.
        let max_lane = rows.iter().map(|r| r.lane).max().unwrap();
        assert!(
            max_lane >= MAX_VISIBLE_LANES,
            "với {SO_NHANH} nhánh sống và giới hạn {MAX_VISIBLE_LANES}, phải có hàng \
             vượt giới hạn; lane cao nhất thấy được là {max_lane}. Nếu số này bị kẹp \
             vào giới hạn thì cài đặt đang áp cap sai chỗ (bước 1 thay vì bước 3)."
        );

        // Mỗi nhánh vẫn phải có lane RIÊNG — không hai hàng nào bị nhồi chung lane.
        let lanes_dau_nhanh: HashSet<u16> = rows[..SO_NHANH].iter().map(|r| r.lane).collect();
        assert_eq!(
            lanes_dau_nhanh.len(),
            SO_NHANH,
            "{SO_NHANH} nhánh sống đồng thời phải chiếm {SO_NHANH} lane khác nhau"
        );
    }

    /// `assign(x).len() == x.len()` phát biểu một lần nữa trên một tập hình dạng trộn
    /// lẫn, để bất biến không chỉ đúng ở từng ca riêng lẻ mà cả khi chúng gặp nhau.
    #[test]
    fn so_hang_luon_bang_so_commit_tren_hinh_dang_tron_lan() {
        let commits = vec![
            commit("m4", &["a", "b", "c", "d"]),
            commit("a", &["chung"]),
            commit("b", &["chung"]),
            commit("c", &["ngoai_tap"]),
            commit("d", &[]),
            commit("chung", &[]),
            commit("le_loi", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(rows[0].out_edges.len(), 4, "octopus bốn cha giữ bốn cạnh");
        assert!(rows[3].terminates, "'c' có cha ngoài tập → terminates");
        assert!(!rows[5].terminates, "'chung' là gốc thật");
    }
}
