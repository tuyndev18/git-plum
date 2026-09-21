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
/// Là số nguyên tố có chủ ý: hai lane cạnh nhau (chênh 1) và các bước nhảy đều đặn
/// khác đều không rơi trùng màu sớm, nên đường kề nhau khó lẫn hơn so với một số chẵn
/// như 8. Bảng màu thật nằm ở phía giao diện (plan 02-06); ở đây chỉ là số lượng, để
/// Rust và React đồng ý về cùng một phép chia dư.
pub const LANE_COLORS: u8 = 7;

/// Số lane tối đa được vẽ trước khi chuyển sang **cách vẽ suy giảm có chủ ý**.
///
/// # GIÁ TRỊ TẠM — plan 02-03 chốt lại con số này
///
/// Đây là một trong hai câu hỏi còn ngỏ của `CONTEXT.md` (`<open_questions>`), nên
/// không được lặng lẽ đoán một số rồi coi như xong.
///
/// **Cách chốt đúng là gì:** con số này phải suy ra từ *bề rộng hiển thị thật* — bao
/// nhiêu lane lọt vào cột đồ thị ở độ rộng mặc định mà vẫn bấm được và nhìn được. Đó là
/// một đối số của hàm, không phải một hằng số vũ trụ, nên 02-03 nhiều khả năng biến nó
/// thành tham số truyền vào.
///
/// **Cách chốt SAI là gì:** lấy số lane lớn nhất đo được trên repo mẫu `wide` (25 lane
/// đồng thời) rồi đặt giới hạn cao hơn nó. Lập luận đó vòng tròn — nó bảo đảm đúng một
/// điều là fixture không bao giờ kích hoạt đường vẽ suy giảm, tức là đường đó không bao
/// giờ được kiểm. Repo thật rộng hơn `wide` rất nhiều.
///
/// Giá trị 32 ở đây chỉ để mã biên dịch và để đường suy giảm có chỗ neo. Khi vượt giới
/// hạn, cha không vẽ được đếm vào [`GraphRow::truncated_parents`].
pub const MAX_VISIBLE_LANES: u16 = 32;

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

    /// Giới hạn lane phải lớn hơn số lane đo được trên repo mẫu `wide` (25 đồng thời),
    /// nếu không mọi fixture đều đi vào đường vẽ suy giảm và ca thường không được kiểm.
    /// Đây là cận **dưới** mềm, không phải cách chốt giá trị — xem doc comment của hằng.
    #[test]
    fn gioi_han_lane_du_lon_cho_ca_thuong() {
        const {
            assert!(
                MAX_VISIBLE_LANES > 25,
                "giới hạn phải vượt 25 lane đồng thời của repo mẫu `wide`"
            )
        };
    }
}
