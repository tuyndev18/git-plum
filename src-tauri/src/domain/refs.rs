//! `Ref` — hợp đồng dữ liệu của một tham chiếu (nhánh, tag, HEAD).
//!
//! Khai báo ở plan 02-02 nhưng **điền dữ liệu** ở plan 02-04, nơi cài bộ phân tích
//! `git for-each-ref`. Khai báo sớm để 02-04 và 02-06 biết hình dạng payload mà không
//! phải chờ nhau.

use serde::Serialize;

/// Loại tham chiếu.
///
/// # Vì sao là enum chứ không phải một cờ `is_remote: bool`
///
/// HIST-06 đòi thanh bên liệt kê **nhánh local, nhánh remote và tag** thành ba nhóm
/// riêng kèm số đếm. Một cờ boolean biểu diễn được hai trạng thái; ở đây có ít nhất
/// bốn. Dùng cờ rồi suy ra tag bằng cách so chuỗi tiền tố `refs/tags/` ở phía giao
/// diện là đẩy việc phân tích sang JavaScript — đúng thứ kiến trúc của phase này cấm.
///
/// Serialize thành chuỗi camelCase: `"localBranch"`, `"remoteBranch"`, `"tag"`, `"other"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RefKind {
    /// `refs/heads/*`
    LocalBranch,
    /// `refs/remotes/*`
    RemoteBranch,
    /// `refs/tags/*` — cả tag nhẹ lẫn tag có chú thích.
    Tag,
    /// `refs/stash`, `refs/notes/*`, và mọi thứ khác. Không vứt đi: một ref lạ hiện ra
    /// dưới nhóm "khác" tốt hơn là biến mất và khiến người dùng tưởng mình mất dữ liệu.
    Other,
}

/// Một tham chiếu đã phân tích từ `git for-each-ref`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ref {
    /// Tên đầy đủ, ví dụ `refs/heads/main`. Đây là tên **duy nhất** an toàn để truyền
    /// lại cho một lệnh git: tên ngắn có thể trùng giữa nhánh và tag.
    pub full_name: String,

    /// Tên rút gọn để hiển thị, ví dụ `main` hoặc `origin/main`.
    pub short_name: String,

    /// Nhánh local, nhánh remote, tag, hay thứ khác.
    pub kind: RefKind,

    /// Mã commit mà ref này trỏ tới (đã giải tham chiếu nếu là tag có chú thích).
    pub target: String,

    /// Nhánh thượng nguồn dạng tên đầy đủ, `None` khi chưa đặt.
    pub upstream: Option<String>,

    /// Số commit có ở đây mà thượng nguồn chưa có. `0` khi không có thượng nguồn.
    pub ahead: u32,

    /// Số commit có ở thượng nguồn mà đây chưa có. `0` khi không có thượng nguồn.
    pub behind: u32,

    /// `true` khi `HEAD` đang trỏ tới ref này.
    ///
    /// Repo mẫu `detached` có `git symbolic-ref HEAD` rỗng — HEAD tách rời. Khi đó
    /// **mọi** ref đều có `is_head == false` và đó là trạng thái hợp lệ, không phải lỗi.
    pub is_head: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_mau() -> Ref {
        Ref {
            full_name: "refs/heads/main".into(),
            short_name: "main".into(),
            kind: RefKind::LocalBranch,
            target: "c".repeat(40),
            upstream: Some("refs/remotes/origin/main".into()),
            ahead: 2,
            behind: 3,
            is_head: true,
        }
    }

    #[test]
    fn ref_serialize_dung_ten_khoa_camel_case() {
        let json = serde_json::to_value(ref_mau()).unwrap();

        assert_eq!(json["fullName"], "refs/heads/main");
        assert_eq!(json["shortName"], "main");
        assert_eq!(json["target"], "c".repeat(40));
        assert_eq!(json["upstream"], "refs/remotes/origin/main");
        assert_eq!(json["ahead"], 2);
        assert_eq!(json["behind"], 3);
        assert_eq!(json["isHead"], true);

        assert!(json.get("full_name").is_none());
        assert!(json.get("is_head").is_none());
    }

    /// HIST-06 phân biệt local với remote bằng giá trị enum này, nên chuỗi nó sinh ra
    /// là một phần của hợp đồng IPC chứ không phải chi tiết nội bộ.
    #[test]
    fn ref_kind_serialize_thanh_chuoi_camel_case() {
        assert_eq!(
            serde_json::to_value(RefKind::LocalBranch).unwrap(),
            "localBranch"
        );
        assert_eq!(
            serde_json::to_value(RefKind::RemoteBranch).unwrap(),
            "remoteBranch"
        );
        assert_eq!(serde_json::to_value(RefKind::Tag).unwrap(), "tag");
        assert_eq!(serde_json::to_value(RefKind::Other).unwrap(), "other");
    }

    /// Không có thượng nguồn thì `upstream` là `null` — frontend phân biệt được
    /// "chưa đặt" với "đặt rồi mà bằng nhau" (ahead 0, behind 0).
    #[test]
    fn khong_co_thuong_nguon_thi_upstream_la_null() {
        let mut r = ref_mau();
        r.upstream = None;
        r.ahead = 0;
        r.behind = 0;

        let json = serde_json::to_value(&r).unwrap();
        assert!(json["upstream"].is_null());
        assert_eq!(json["ahead"], 0);
    }
}
