//! Thuật toán gán lane — biến danh sách commit (cha luôn đứng sau con) thành hình
//! học từng dòng. **Trái tim của Core Value**: nếu tệp này sai, đồ thị sai ở mọi
//! màn hình.
//!
//! # Thuật toán: "straight branches" của pvigier (2019)
//!
//! Nguồn: <https://pvigier.github.io/2019/05/06/commit-graph-drawing-algorithms.html>
//! (thuật toán của gitamine). Thay thuật toán "lane chờ" trước đây — thuật toán đó
//! lấy lane chờ có chỉ số nhỏ nhất bất kể lane chờ qua cha thứ nhất hay cha merge,
//! nên chuỗi first-parent bị bẻ sang cột merge và merge liên tiếp mở lane bậc thang.
//!
//! Hai loại con của một commit `c`:
//!
//! - **branch child** `d`: `d.parents[0] == c` — `d` kéo dài nhánh của `c`.
//! - **merge child** `d`: `c` nằm trong `d.parents[1..]`.
//!
//! ```text
//! B = danh sách cột đang có nhánh (None = cột trống, KHÔNG xoá để cột không dịch)
//! với mỗi commit c theo thứ tự hàng:
//!   J(c) = các cột có dùng ở hàng nào đó trong (min hàng merge child của c, c)
//!   nếu có branch child d mà cột d không thuộc J(c): c THAY d trong B (cùng cột)
//!   ngược lại: c vào cột trống đầu tiên không thuộc J(c) (hoặc cột mới)
//!   mọi branch child khác của c: giải phóng cột của nó
//! ```
//!
//! Vì sao cần J(c): cạnh từ merge child `m` tới `c` đi **ngang** ở hàng `m` sang cột
//! của `c` rồi **dọc** xuống tới `c`. Cột của `c` vì vậy phải trống suốt từ hàng `m`
//! tới hàng `c`, nếu không cạnh merge đè lên một đường khác.
//!
//! Hình học cạnh suy ra từ cột:
//!
//! - cạnh cha thứ nhất `c -> p`: dọc trong cột **của `c`** tới hàng `p`, gập vào `p`
//!   ở chính hàng `p` nếu `p` nằm cột khác (nhánh `c` nhập vào nhánh `p`).
//! - cạnh cha merge `c -> p`: ngang ở hàng `c` sang cột **của `p`**, rồi dọc xuống.
//!
//! Thứ tự hàng phải là "temporal topological" của bài viết — chính là
//! `git log --date-order` mà `LOG_ARGS` ghim.
//!
//! # Ba chỗ dễ sai, đã có test riêng cho từng chỗ
//!
//! **Cha thêm phải là vòng lặp.** `parents.iter().skip(1)`, không bao giờ
//! `parents[1]`. Merge octopus bốn cha phải sinh bốn cạnh. Ràng buộc số 3 của ROADMAP
//! (`.planning/research/PITFALLS.md` mục 2). Tên biến kiểu `parent1`/`parent2` bị cấm.
//!
//! **Cha ngoài tập đã nạp không phải cha.** Bản sao nông có commit biên khai báo một
//! cha không tồn tại trong dữ liệu. Không vẽ cạnh cho nó; hàng đặt
//! [`GraphRow::terminates`] và cột được giải phóng ngay.
//!
//! **Giới hạn hiển thị áp cho cạnh merge, KHÔNG áp cho hàng.** Mọi commit luôn có
//! cột (HIST-04). Cạnh merge tới một cha nằm ở cột `>= MAX_VISIBLE_LANES` bị lược và
//! tăng [`GraphRow::truncated_parents`] — suy giảm hiển thị, không suy giảm dữ liệu.
//!
//! # Vì sao KHÔNG dùng `rayon`
//!
//! Gán cột **vốn tuần tự**: trạng thái `B` ở hàng `r` là kết quả của toàn bộ hàng
//! `0..r`. Song song hoá không phải vô ích — nó **sai về mặt thuật toán**.
//!
//! # Hàm thuần
//!
//! Không IO, không async, không `Mutex`, không biết gì về Tauri
//! (`.planning/research/ARCHITECTURE.md` mục "graph/lanes.rs isolated and IO-free").

use std::collections::HashMap;

use crate::domain::Commit;
use crate::graph::types::{Edge, GraphRow, LANE_COLORS, MAX_VISIBLE_LANES};

/// Màu của một lane. Hợp đồng với bộ vẽ: `color == lane % LANE_COLORS` ở mọi nơi.
#[inline]
fn color_of(lane: u16) -> u8 {
    (lane % LANE_COLORS as u16) as u8
}

/// Thêm một cạnh đi xuyên hàng nếu hàng chưa có đúng cạnh đó — hai cạnh merge cùng
/// về một cha chạy chung cột, vẽ hai lần chỉ tốn công.
fn push_pass(row: &mut GraphRow, from_lane: u16, to_lane: u16, color: u8) {
    if row
        .passthrough
        .iter()
        .any(|e| e.from_lane == from_lane && e.to_lane == to_lane)
    {
        return;
    }
    row.passthrough.push(Edge {
        from_lane,
        to_lane,
        color,
    });
}

/// Gán lane cho danh sách commit (cha luôn đứng sau con) và sinh hình học từng dòng.
///
/// Trả đúng một [`GraphRow`] cho mỗi [`Commit`], cùng thứ tự, cùng độ dài — bất biến
/// này không có ngoại lệ, kể cả khi số nhánh sống vượt [`MAX_VISIBLE_LANES`].
///
/// # Đầu vào phải có cha sau con
///
/// `LOG_ARGS` ghim `--date-order`. Một cha đứng **trước** con (đầu vào sai thứ tự,
/// hoặc dữ liệu có vòng từ một repo thù địch) bị coi như cha ngoài tập: không vẽ cạnh,
/// đặt `terminates`. Hàm không sập, không lặp vô hạn (T-02-08) — hai lượt tuyến tính.
pub fn assign(commits: &[Commit]) -> Vec<GraphRow> {
    let n = commits.len();

    // Mã commit -> hàng. Giữ lần xuất hiện ĐẦU nếu trùng mã (đầu vào thù địch).
    let mut index: HashMap<&str, usize> = HashMap::with_capacity(n);
    for (i, c) in commits.iter().enumerate() {
        index.entry(c.id.as_str()).or_insert(i);
    }

    // Cha đã phân giải thành hàng; None = cha ngoài tập (hoặc sai thứ tự).
    let parents: Vec<Vec<Option<usize>>> = commits
        .iter()
        .enumerate()
        .map(|(i, c)| {
            c.parents
                .iter()
                .map(|p| index.get(p.as_str()).copied().filter(|&pi| pi > i))
                .collect()
        })
        .collect();

    // Con của từng commit: branch child và hàng nhỏ nhất trong các merge child.
    let mut branch_children: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut merge_min: Vec<Option<usize>> = vec![None; n];
    for (i, ps) in parents.iter().enumerate() {
        let first = ps.first().copied().flatten();
        if let Some(p) = first {
            branch_children[p].push(i);
        }
        for p in ps.iter().skip(1).copied().flatten() {
            if Some(p) == first {
                continue;
            }
            // `i` tăng dần nên lần gán đầu là hàng nhỏ nhất.
            merge_min[p].get_or_insert(i);
        }
    }

    // --- Lượt 1: gán cột (straight branches).
    let mut col: Vec<u16> = vec![0; n];
    // b[j] = commit đang giữ cột j (cột bận từ commit đó xuống tới cha thứ nhất).
    let mut b: Vec<Option<usize>> = Vec::new();
    // last_used[j] = hàng gần nhất cột j đã được dùng bởi một thứ đã kết thúc; -1 = chưa.
    let mut last_used: Vec<i64> = Vec::new();

    for i in 0..n {
        let first_parent = parents[i].first().copied().flatten();
        let bc = &branch_children[i];

        // Cột j có bị dùng ở hàng nào đó trong (a, i) không, KHÔNG tính cạnh đang
        // đổ vào chính `i` từ một branch child (cạnh đó kết thúc tại `i`, trùng đích).
        let forbidden = |j: usize| -> bool {
            let Some(a) = merge_min[i] else {
                return false;
            };
            let end = match b[j] {
                Some(d) if parents[d].first().copied().flatten() == Some(i) => d as i64,
                Some(_) => i as i64,
                None => last_used[j],
            };
            end > a as i64
        };

        let chosen = bc
            .iter()
            .copied()
            .filter(|&d| !forbidden(col[d] as usize))
            .min_by_key(|&d| col[d]);

        let j = match chosen {
            Some(d) => col[d] as usize,
            None => match (0..b.len()).find(|&j| b[j].is_none() && !forbidden(j)) {
                Some(j) => j,
                None => {
                    b.push(None);
                    last_used.push(-1);
                    b.len() - 1
                }
            },
        };

        // Branch child khác: nhánh của nó nhập vào `i` tại hàng này, cột được trả.
        for &d in bc {
            if Some(d) == chosen {
                continue;
            }
            let dj = col[d] as usize;
            b[dj] = None;
            last_used[dj] = i as i64;
        }

        col[i] = j as u16;
        if first_parent.is_some() {
            b[j] = Some(i);
        } else {
            // Gốc thật hoặc biên nông: nhánh dừng ở đây.
            b[j] = None;
            last_used[j] = i as i64;
        }
    }

    // --- Lượt 2: hình học từng hàng từ cột.
    let mut rows: Vec<GraphRow> = commits
        .iter()
        .enumerate()
        .map(|(i, c)| GraphRow {
            commit_id: c.id.clone(),
            lane: col[i],
            color: color_of(col[i]),
            passthrough: Vec::new(),
            out_edges: Vec::new(),
            truncated_parents: 0,
            terminates: false,
        })
        .collect();

    for i in 0..n {
        let cj = col[i];
        for (k, p) in parents[i].iter().enumerate() {
            let Some(p) = *p else {
                // Cha khai báo nhưng không có trong dữ liệu: hàng "còn tiếp".
                rows[i].terminates = true;
                continue;
            };
            let pj = col[p];
            if k == 0 {
                // Cạnh cha thứ nhất: dọc trong cột của `i`, gập vào `p` ở hàng `p`.
                let color = color_of(cj);
                rows[i].out_edges.push(Edge {
                    from_lane: cj,
                    to_lane: cj,
                    color,
                });
                for r in (i + 1)..p {
                    push_pass(&mut rows[r], cj, cj, color);
                }
                if pj != cj {
                    push_pass(&mut rows[p], cj, pj, color);
                }
            } else {
                // Cạnh cha merge: ngang ở hàng `i` sang cột của `p`, rồi dọc xuống.
                // `.skip(1)`-tương đương: mọi cha từ vị trí 1 trở đi, bốn cha bốn cạnh.
                if pj >= MAX_VISIBLE_LANES {
                    rows[i].truncated_parents += 1;
                    continue;
                }
                let color = color_of(pj);
                if !rows[i]
                    .out_edges
                    .iter()
                    .any(|e| e.from_lane == cj && e.to_lane == pj)
                {
                    rows[i].out_edges.push(Edge {
                        from_lane: cj,
                        to_lane: pj,
                        color,
                    });
                }
                for r in (i + 1)..p {
                    push_pass(&mut rows[r], pj, pj, color);
                }
            }
        }
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

    // ---------------------------------------------------------------- straight branches

    /// Mẫu "merge nhánh feature vào main nhiều lần liên tiếp" của repo thật
    /// `quanly-truong-phong-so`: M_k = merge(M_{k-1}, D_k), D_k có cha D_{k-1}. Theo
    /// `--date-order`, D_k đứng ngay dưới M_k. Đồ thị đúng chỉ cần HAI cột: main thẳng
    /// ở cột 0, nhánh thẳng ở cột 1. Thuật toán cũ + `--topo-order` mở một cột mới cho
    /// mỗi merge (bậc thang 20 cột).
    #[test]
    fn merge_lien_tiep_chi_can_hai_cot() {
        let commits = vec![
            commit("m3", &["m2", "d3"]),
            commit("d3", &["d2"]),
            commit("m2", &["m1", "d2"]),
            commit("d2", &["d1"]),
            commit("m1", &["r", "d1"]),
            commit("d1", &["r"]),
            commit("r", &[]),
        ];
        let rows = assign(&commits);
        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        let lanes: Vec<u16> = rows.iter().map(|r| r.lane).collect();
        assert_eq!(lanes, vec![0, 1, 0, 1, 0, 1, 0], "main cột 0, nhánh cột 1, thẳng suốt");
    }

    /// Chuỗi first-parent giữ nguyên cột dù có cạnh merge chờ cùng commit: commit
    /// THAY branch child của nó trong cùng cột (pvigier), không nhảy sang cột merge.
    #[test]
    fn chuoi_first_parent_giu_nguyen_cot() {
        let commits = vec![
            commit("a", &["b"]),
            commit("x", &["y", "b"]),
            commit("b", &["c"]),
            commit("y", &["c"]),
            commit("c", &[]),
        ];
        let rows = assign(&commits);
        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        // a ở cột 0; b là cha thứ nhất của a -> b cũng cột 0, c cũng cột 0.
        assert_eq!(rows[0].lane, 0);
        assert_eq!(rows[2].lane, 0, "b phải nối tiếp cột của a");
        assert_eq!(rows[4].lane, 0, "c phải nối tiếp cột của b");
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

    /// Lane sinh ra cho cha thêm **không được** kèm passthrough ở chính hàng merge.
    ///
    /// Bước 3 cấp lane cho cha thứ hai, rồi bước 4 quét mọi lane đang sống — nếu bước
    /// 4 không loại lane vừa sinh, nó phát một `Edge { from: 1, to: 1 }` mô tả một
    /// nhánh đi **xuyên qua cả ô** của hàng merge. Nhưng lane đó mới ra đời ở TÂM
    /// hàng: nửa trên của ô nó chưa tồn tại. Bộ vẽ dựng đúng theo dữ liệu sẽ cho một
    /// đoạn thẳng cụt lơ lửng bên cạnh nút merge, không nối lên đâu — cộng thêm việc
    /// `out_edges` đã có cạnh `0 -> 1` mô tả chính nhánh đó, thành vẽ hai lần.
    ///
    /// Đây là nguyên nhân của đoạn nhánh "gãy" trong `docs/screenshots/`.
    #[test]
    fn hang_merge_khong_co_passthrough_tren_lane_vua_sinh() {
        // Hình dạng thật của một merge: `m` gộp `p1` và `p2`, cả hai cùng về `base`.
        let commits = vec![
            commit("m", &["p1", "p2"]),
            commit("p1", &["base"]),
            commit("p2", &["base"]),
            commit("base", &[]),
        ];
        let rows = assign(&commits);

        khang_dinh_mot_hang_moi_commit(&commits, &rows);
        khang_dinh_mau_khop_lane(&rows);

        assert_eq!(
            rows[0].passthrough,
            vec![],
            "hàng merge không có nhánh nào đi xuyên qua: lane 1 mới sinh ở chính hàng \
             này, và `out_edges` đã mô tả nó bằng cạnh 0 -> 1"
        );
        assert_eq!(
            lanes_cua(&rows[0].out_edges),
            vec![0, 1],
            "nhánh rẽ ra mô tả bằng cạnh ra, không phải bằng passthrough"
        );

        // Hàng NGAY SAU merge thì lane 1 đã sống từ trước → passthrough là đúng.
        assert_eq!(
            rows[1].passthrough.len(),
            1,
            "hàng sau merge: lane 1 đã tồn tại từ hàng trên nên đi xuyên qua"
        );
        assert_eq!(rows[1].passthrough[0].from_lane, 1);
        assert_eq!(rows[1].passthrough[0].to_lane, 1);
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
