//! Hình học đồ thị nhánh — hợp đồng giữa thuật toán gán lane (Rust) và bộ vẽ (React).
//!
//! # Rust tính, frontend chỉ vẽ
//!
//! Quyết định đã chốt trong CONTEXT.md: **toàn bộ** hình học từng dòng tính ở Rust.
//! Frontend nhận [`GraphRow`] và vẽ nó, không bao giờ tự suy ra liên thông giữa các
//! dòng. Đây là điểm nghẽn hiệu năng chính của sản phẩm — tiêu chí thành công số 1 đòi
//! 100k commit hiện ra dưới một giây — và để vòng lặp đó trong JavaScript là sai.
//!
//! Kiểu ở đây khai báo tại plan 02-02 nhưng **sinh ra** tại plan 02-03. Khai báo sớm
//! để 02-03 chỉ phải nghĩ về thuật toán, và để 02-04 (command) cùng 02-06 (giao diện)
//! biết hình dạng payload mà không phải chờ thuật toán xong.

use serde::Serialize;

/// Số màu trong bảng màu lane. Màu của một lane là `lane % LANE_COLORS`.
///
/// # Vì sao bằng đúng [`MAX_VISIBLE_LANES`], không phải một số nhỏ hơn
///
/// Với `7` (giá trị cũ) và cap 13 lane, **lane 0 và lane 7 nhận cùng một màu**, lane 1
/// và lane 8 cũng vậy, … Trên repo thật có 13 nhánh sống cùng lúc, đó là sáu cặp cột
/// dọc không phân biệt nổi — người dùng báo "khó nhìn quá" kèm ảnh 13 lane chính là ca
/// này. Lập luận cũ ("số nguyên tố nên hai lane cạnh nhau không trùng màu sớm") chỉ
/// đúng cho lane **kề nhau**; nó không nói gì về lane cách nhau đúng 7 cột, mà ở mật độ
/// cao thì hai cột cách 7 vẫn nằm gọn trong tầm mắt.
///
/// Đặt bằng cap hiển thị thì **không hai lane nào vẽ được cùng lúc mà trùng màu** —
/// bất biến này là thứ giữ cho đồ thị đọc được ở mật độ tối đa, và nó tự đúng theo định
/// nghĩa chứ không phải nhờ chọn khéo con số.
///
/// Bảng màu thật nằm ở phía giao diện (`src/lib/graph-render/geometry.ts`); ở đây chỉ
/// là số lượng, để Rust và React đồng ý về cùng một phép chia dư. Bảng phía giao diện
/// **phải có đủ 13 màu** — có test ghim hai phía.
pub const LANE_COLORS: u8 = 20;

/// Số lane tối đa được vẽ trước khi chuyển sang **cách vẽ suy giảm có chủ ý**.
///
/// # Đã chốt ở plan 02-03 — **từ bề rộng hiển thị**, không từ số đo fixture
///
/// Phép tính, ghi đầy đủ ở `docs/04-phase2-degraded-graph.md`:
///
/// ```text
///   cửa sổ mặc định                    1440 px   (tauri.conf.json "width")
///   × vùng giữa                        × 52%     (AppLayout Panel id="main")
///   = vùng giữa                        ≈ 749 px
///   × ngân sách cột đồ thị             × 40%     (60% còn lại cho thông điệp commit)
///   = cột đồ thị                       ≈ 300 px
///   − lề trái GRAPH_PADDING_LEFT       −  12 px
///   ÷ LANE_WIDTH                       ÷  14 px
///   = 20,6                             → 20 lane
/// ```
///
/// # Lịch sử con số này — đọc trước khi đổi lần nữa
///
/// `20 → 13` (`544ae5f`) khi `LANE_WIDTH` tăng 14 → 22px cho khớp
/// `docs/screenshots/`. Rồi **`13 → 20` ngày 2026-09-22, có số đo**: cap 13 làm
/// **19,99%** hàng của repo perf 100 007 commit bị gập vào cột 12 — tới tám lane
/// khác nhau vẽ chung một cột, **không chỉ báo gì** (badge `+N` chỉ phủ
/// `truncated_parents`, 0,67%). Cap 20 cho **0,71%**. Gấp 28 lần.
///
/// Điều làm nó nghiêm trọng hơn một khiếm khuyết thẩm mỹ: đồ thị vẽ **sai một
/// cách tự tin** thay vì suy giảm thấy được. Ca thứ hai đo được cùng lúc: cạnh
/// của hàng WIP (Phase 4, `WORK-11`) nối xuống HEAD, và nếu HEAD ở lane ≥ cap
/// thì cạnh đó bị gập vào cột cuối và **chỉ sang commit khác**. Điều kiện kích
/// hoạt là 14+ lane song song với HEAD không ở nhánh mới nhất theo topo — một
/// repo feature-branch bình thường.
///
/// Đánh đổi nhận lại: lane 14px chật hơn 22px, khác ảnh tham chiếu. Chủ dự án
/// chốt 2026-09-22 sau khi thấy cả hai số.
///
/// Phần vượt cap vẫn hiện bằng chỉ báo
/// `+N` qua [`GraphRow::truncated_parents`].
///
/// **PHẢI khớp `MAX_VISIBLE_LANES` ở `src/lib/graph-render/geometry.ts`.** Lệch
/// hai phía là lỗi im lặng: backend cấp lane 19 mà frontend chỉ vẽ tới 13 thì
/// hai nhánh khác nhau bị vẽ đè lên cùng một cột.
///
/// **Cách chốt SAI mà đã tránh:** lấy số lane lớn nhất đo được trên repo mẫu `wide`
/// (25 lane đồng thời) rồi đặt giới hạn cao hơn nó. Lập luận đó vòng tròn — nó bảo đảm
/// đúng một điều là fixture không bao giờ kích hoạt đường vẽ suy giảm, tức là đường đó
/// không bao giờ được kiểm. Số đo của `wide` (25) và của repo hiệu năng (21) dùng để
/// **kiểm chứng** rằng bộ dữ liệu thật đủ rộng để chạm giới hạn 20 này, không dùng để
/// **chọn** nó.
///
/// # Giới hạn này áp cho CÁI GÌ — đọc kỹ, đây là chỗ dễ sai nhất của cả phase
///
/// Áp cho **lane của cha thêm** (bước 3 của thuật toán): merge có nhiều cha hơn số lane
/// trống thì cha không vẽ được đếm vào [`GraphRow::truncated_parents`].
///
/// **KHÔNG** áp cho **lane của chính hàng** (bước 1). Mọi commit trong đầu vào đều nhận
/// một lane thật, dù lane đó vượt con số này. Bất biến `rows.len() == commits.len()` là
/// HIST-04 ở tầng dữ liệu và không được phụ thuộc vào một hằng số *hiển thị*. Việc gập
/// lane vượt giới hạn vào cột cuối là của frontend (plan 02-05).
pub const MAX_VISIBLE_LANES: u16 = 20;

/// Một đường nối giữa hai lane trên một dòng.
///
/// Toạ độ là **chỉ số lane**, không phải pixel: bộ vẽ tự quy đổi ra pixel theo bề rộng
/// lane và chiều cao dòng của nó. Nhờ vậy đổi từ canvas sang SVG chỉ phải sửa một tệp
/// (quyết định "bộ vẽ nằm sau một interface" trong CONTEXT.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    /// Lane ở cạnh **trên** của dòng.
    pub from_lane: u16,
    /// Lane ở cạnh **dưới** của dòng. Khác `from_lane` nghĩa là đường xiên.
    pub to_lane: u16,
    /// Chỉ số màu, `lane % LANE_COLORS`. Giữ trong `Edge` chứ không để frontend tính
    /// lại: đường xiên phải mang màu của lane **nguồn**, và biết nguồn là lane nào chỉ
    /// có thuật toán mới nắm được.
    pub color: u8,
}

/// Hình học của **đúng một dòng** trong danh sách lịch sử.
///
/// Một `GraphRow` ứng một-một với một [`crate::domain::Commit`] cùng chỉ số trong mảng.
/// Đây là lý do cột đồ thị và cột văn bản không thể lệch hàng: chúng vẽ từ cùng một
/// mảng `virtualItems`, không phải từ hai vùng cuộn đồng bộ với nhau.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphRow {
    /// Mã commit của dòng này — để khẳng định đồ thị và văn bản đúng là cùng một dòng.
    pub commit_id: String,

    /// Lane chứa chấm tròn của commit này.
    pub lane: u16,

    /// Màu của chấm, `lane % LANE_COLORS`.
    pub color: u8,

    /// Đường **đi xuyên qua** dòng này mà không dừng lại — nhánh khác đang chạy song
    /// song. Vẽ trước, nằm dưới chấm tròn.
    pub passthrough: Vec<Edge>,

    /// Đường đi **xuống** từ commit này tới các cha của nó.
    ///
    /// Commit gốc thật có `out_edges` rỗng và [`GraphRow::terminates`] bằng `false`.
    pub out_edges: Vec<Edge>,

    /// Số cha **không** vẽ được vì vượt [`MAX_VISIBLE_LANES`].
    ///
    /// Đây là chỗ neo của "cách vẽ suy giảm có chủ ý" mà ROADMAP đòi: giao diện hiện
    /// chỉ báo `+N cha nữa` thay vì vẽ tràn ra ngoài cột hoặc âm thầm bỏ đường nối.
    /// Bằng `0` trong mọi ca thường, kể cả merge octopus bốn cha.
    pub truncated_parents: u16,

    /// `true` khi lane kết thúc ở đây vì cha **không có trong tập đã nạp** — bản sao
    /// nông, hoặc trang hiện tại chưa nạp tới cha.
    ///
    /// # Phân biệt với commit gốc thật
    ///
    /// Repo mẫu `shallow` có commit biên khai báo một `parent` không tồn tại trong
    /// repo. Trông từ dữ liệu thì nó giống hệt một commit gốc, nhưng vẽ nó như gốc là
    /// nói dối người dùng rằng lịch sử kết thúc ở đó. Cờ này cho frontend vẽ dấu hiệu
    /// "còn tiếp" thay vì dấu chấm hết.
    ///
    /// Commit gốc thật: `terminates == false`, `out_edges` rỗng.
    /// Biên bản sao nông: `terminates == true`.
    pub terminates: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn edge_mau() -> Edge {
        Edge {
            from_lane: 1,
            to_lane: 3,
            color: 1,
        }
    }

    fn row_mau() -> GraphRow {
        GraphRow {
            commit_id: "d".repeat(40),
            lane: 2,
            color: 2,
            passthrough: vec![edge_mau()],
            out_edges: vec![edge_mau()],
            truncated_parents: 0,
            terminates: false,
        }
    }

    #[test]
    fn graph_row_serialize_dung_ten_khoa_camel_case() {
        let json = serde_json::to_value(row_mau()).unwrap();

        assert_eq!(json["commitId"], "d".repeat(40));
        assert_eq!(json["lane"], 2);
        assert_eq!(json["color"], 2);
        assert!(json["passthrough"].is_array());
        assert!(json["outEdges"].is_array());
        assert_eq!(json["truncatedParents"], 0);
        assert_eq!(json["terminates"], false);

        assert!(json.get("commit_id").is_none());
        assert!(json.get("out_edges").is_none());
        assert!(json.get("truncated_parents").is_none());
    }

    #[test]
    fn edge_serialize_dung_ten_khoa_camel_case() {
        let json = serde_json::to_value(edge_mau()).unwrap();

        assert_eq!(json["fromLane"], 1);
        assert_eq!(json["toLane"], 3);
        assert_eq!(json["color"], 1);

        assert!(json.get("from_lane").is_none());
        assert!(json.get("to_lane").is_none());
    }

    /// Dòng không có đường nối nào vẫn phải cho mảng rỗng chứ không phải `null` —
    /// bộ vẽ lặp qua hai mảng này trên **mọi** dòng hiện trên màn hình.
    #[test]
    fn mang_edge_rong_serialize_thanh_mang_rong() {
        let mut row = row_mau();
        row.passthrough.clear();
        row.out_edges.clear();

        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["passthrough"].as_array().unwrap().len(), 0);
        assert_eq!(json["outEdges"].as_array().unwrap().len(), 0);
        assert!(!json["passthrough"].is_null());
        assert!(!json["outEdges"].is_null());
    }

    /// `color = lane % LANE_COLORS` phải nằm trong bảng màu với mọi lane, kể cả lane
    /// vượt giới hạn hiển thị. Một chỉ số màu tràn bảng sẽ cho `undefined` ở JavaScript
    /// và vẽ ra đường vô hình.
    #[test]
    fn mau_lane_luon_nam_trong_bang() {
        const { assert!(LANE_COLORS > 0, "bảng màu rỗng sẽ chia cho 0") };
        for lane in 0u16..1000 {
            let color = (lane % LANE_COLORS as u16) as u8;
            assert!(color < LANE_COLORS, "lane {lane} cho màu ngoài bảng");
        }
    }

    /// Giới hạn lane phải **nhỏ hơn** số lane đo được trên repo mẫu `wide` (25 đồng
    /// thời) và của repo hiệu năng (21) — nếu không thì không bộ dữ liệu nào ta có
    /// chạm tới đường vẽ suy giảm, `truncated_parents` luôn bằng 0, và cả nhánh mã đó
    /// ship ra mà chưa từng chạy trên dữ liệu thật.
    ///
    /// Đây **không** phải cách chốt giá trị — giá trị chốt từ bề rộng hiển thị, xem
    /// doc comment của hằng. Test này chỉ kiểm rằng con số đã chốt tình cờ nằm đúng
    /// phía để fixture kiểm được nhánh suy giảm.
    #[test]
    fn gioi_han_lane_du_nho_de_fixture_cham_toi() {
        const {
            assert!(
                MAX_VISIBLE_LANES < 21,
                "giới hạn phải nhỏ hơn 21 lane của repo hiệu năng, \
                 nếu không nhánh vẽ suy giảm không bao giờ được kiểm"
            )
        };
    }

    /// Nhưng cũng không được nhỏ tới mức ca thường (merge octopus bốn cha, vài nhánh
    /// song song) rơi vào nhánh suy giảm. Cận dưới mềm.
    #[test]
    fn gioi_han_lane_du_lon_cho_ca_thuong() {
        const {
            assert!(
                MAX_VISIBLE_LANES >= 8,
                "giới hạn quá nhỏ thì merge octopus thường ngày cũng bị cắt"
            )
        };
    }
}
