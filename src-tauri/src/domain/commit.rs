//! `Commit` — hợp đồng dữ liệu của một commit, dùng chung cho cả phase 2.
//!
//! Kiểu này là đầu ra của [`crate::git::parsers::log::parse_log`] và là đầu vào của
//! thuật toán gán lane (plan 02-03), của command lịch sử (plan 02-04) và của giao diện
//! (plan 02-06). Đổi hình dạng ở đây là đổi hợp đồng của cả bốn plan.

use serde::Serialize;

/// Một commit đã phân tích xong từ đầu ra byte của `git log`.
///
/// # Mọi trường chuỗi đều đã giải mã lossy
///
/// Các trường `String` ở đây là **trường hiển thị**: chúng đi qua
/// `String::from_utf8_lossy`, nên byte không hợp lệ đã biến thành U+FFFD và **không**
/// khôi phục được. Đừng dùng chúng làm đầu vào cho một lệnh git khác. Trường duy nhất
/// an toàn để làm việc đó là [`Commit::id`] và [`Commit::parents`], vốn là hex ASCII.
///
/// Xem [`Commit::has_invalid_utf8`] để biết commit nào đã đi qua đường lossy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    /// Mã commit đầy đủ, 40 ký tự hex (`%H`). Hex nên luôn là ASCII, an toàn làm `String`.
    pub id: String,

    /// Mã của **mọi** commit cha (`%P`), theo đúng thứ tự git in ra.
    ///
    /// # Đây là `Vec`, không phải hai trường riêng — đừng "tối giản"
    ///
    /// Ràng buộc số 3 của ROADMAP nằm ở chính dòng này: merge octopus có **bốn** cha
    /// phải biểu diễn được. Repo mẫu `octopus` trong `target/fixtures/` tồn tại để
    /// chứng minh điều đó, và repo hiệu năng 100k commit chứa 320 merge từ ba cha trở
    /// lên. Đổi thành `parent1`/`parent2` sẽ làm mất đường nối và rò lane ở mọi merge
    /// nhiều cha — chính lỗi mà công cụ của Microsoft từng mắc (xem
    /// `.planning/research/PITFALLS.md` mục 2).
    ///
    /// Rỗng nghĩa là commit gốc (không cha). Serialize thành `[]`, không phải `null`.
    pub parents: Vec<String>,

    /// Tên tác giả (`%an`). Dữ liệu tự khai trong commit, **không** được xác thực —
    /// xem T-02-07 trong threat model của plan 02-02. Không dùng để quyết định tin cậy.
    pub author_name: String,

    /// Địa chỉ thư của tác giả (`%ae`).
    pub author_email: String,

    /// Thời điểm tác giả, **giây Unix** (`%at`).
    ///
    /// # Vì sao là `i64` chứ không phải một kiểu thời gian
    ///
    /// `chrono` nằm trong danh sách "không dùng" của
    /// `.planning/research/STACK.md`: git vốn trả sẵn Unix timestamp, và phía giao
    /// diện định dạng bằng `Intl.DateTimeFormat` của trình duyệt — vốn đã biết múi giờ
    /// và ngôn ngữ của người dùng, thứ mà Rust không biết. Thêm `chrono` chỉ để rồi
    /// chuyển ngược về số là thêm phụ thuộc mà không được gì.
    ///
    /// `i64` chứ không phải `u64`: git chấp nhận timestamp âm (trước 1970) và repo
    /// thật có những commit như vậy.
    pub author_time: i64,

    /// Tên người commit (`%cn`). Khác tác giả khi commit được cherry-pick hoặc rebase.
    pub committer_name: String,

    /// Địa chỉ thư của người commit (`%ce`).
    pub committer_email: String,

    /// Thời điểm commit, giây Unix (`%ct`). Xem ghi chú ở [`Commit::author_time`].
    ///
    /// Đây là trường `--topo-order` **không** dùng để sắp xếp — thứ tự do topo quyết
    /// định, không phải thời gian. Repo thật có commit với thời gian lệch hẳn quá khứ.
    pub committer_time: i64,

    /// Dòng đầu của thông điệp (`%s`).
    pub subject: String,

    /// Phần thân thông điệp (`%b`), giữ nguyên xuống dòng.
    pub body: String,

    /// `true` khi ít nhất một trường hiển thị của commit này phải giải mã lossy.
    ///
    /// # Cờ này có chủ ý, không phải để báo lỗi cho người dùng
    ///
    /// Giao diện **không** cần hiện gì khi cờ bật — U+FFFD tự nó đã nói lên điều đó.
    /// Cờ tồn tại để test HIST-11 khẳng định được một điều mạnh hơn "không panic":
    /// rằng dòng này *đã đi qua* đường lossy **và vẫn còn nguyên trong danh sách*.
    /// Nếu không có cờ, một bộ phân tích âm thầm bỏ qua commit xấu vẫn sẽ đỗ mọi test
    /// còn lại.
    ///
    /// Không serialize sang camelCase khác thường: khoá JSON là `hasInvalidUtf8`.
    pub has_invalid_utf8: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit_mau() -> Commit {
        Commit {
            id: "a".repeat(40),
            parents: vec!["b".repeat(40)],
            author_name: "Nguyễn An".into(),
            author_email: "an@example.com".into(),
            author_time: 1_700_000_000,
            committer_name: "Trần Bình".into(),
            committer_email: "binh@example.com".into(),
            committer_time: 1_700_000_100,
            subject: "thêm tính năng".into(),
            body: "dòng một\ndòng hai".into(),
            has_invalid_utf8: false,
        }
    }

    /// Frontend đọc **đúng** những tên khoá này. Test khẳng định từng tên cụ thể chứ
    /// không chỉ "serialize được": đổi `rename_all` mà quên sửa TypeScript sẽ cho một
    /// giao diện trống rỗng không báo lỗi gì.
    #[test]
    fn commit_serialize_dung_ten_khoa_camel_case() {
        let json = serde_json::to_value(commit_mau()).unwrap();

        assert_eq!(json["id"], "a".repeat(40));
        assert_eq!(json["authorName"], "Nguyễn An");
        assert_eq!(json["authorEmail"], "an@example.com");
        assert_eq!(json["authorTime"], 1_700_000_000i64);
        assert_eq!(json["committerName"], "Trần Bình");
        assert_eq!(json["committerEmail"], "binh@example.com");
        assert_eq!(json["committerTime"], 1_700_000_100i64);
        assert_eq!(json["subject"], "thêm tính năng");
        assert_eq!(json["body"], "dòng một\ndòng hai");
        assert_eq!(json["hasInvalidUtf8"], false);
        assert!(json["parents"].is_array());

        // Tên snake_case tuyệt đối không được lọt ra JSON.
        assert!(json.get("author_name").is_none());
        assert!(json.get("has_invalid_utf8").is_none());
    }

    /// Commit gốc phải cho `[]`, **không** phải `null`. Frontend lặp qua mảng này;
    /// `null` sẽ ném `TypeError` giữa lúc vẽ và làm trắng cả trang.
    #[test]
    fn parents_rong_serialize_thanh_mang_rong() {
        let mut c = commit_mau();
        c.parents.clear();

        let json = serde_json::to_value(&c).unwrap();
        assert!(json["parents"].is_array(), "parents phải luôn là mảng");
        assert_eq!(json["parents"].as_array().unwrap().len(), 0);
        assert!(
            !json["parents"].is_null(),
            "parents rỗng không được là null"
        );
    }

    /// Bốn cha phải qua được cả đường serialize, không chỉ đường phân tích.
    #[test]
    fn merge_octopus_bon_cha_serialize_du_bon() {
        let mut c = commit_mau();
        c.parents = vec![
            "1".repeat(40),
            "2".repeat(40),
            "3".repeat(40),
            "4".repeat(40),
        ];

        let json = serde_json::to_value(&c).unwrap();
        let parents = json["parents"].as_array().unwrap();
        assert_eq!(parents.len(), 4, "merge octopus bốn cha phải giữ đủ bốn");
        assert_eq!(parents[3], "4".repeat(40));
    }

    /// Chuỗi chứa U+FFFD (kết quả của giải mã lossy) vẫn phải ra JSON hợp lệ —
    /// nếu không thì HIST-11 hỏng ở tầng IPC thay vì ở tầng phân tích.
    #[test]
    fn chuoi_co_u_fffd_van_serialize_duoc() {
        let mut c = commit_mau();
        c.subject = String::from_utf8_lossy(b"byte th\xf4 \xff cu\xf1i").into_owned();
        c.has_invalid_utf8 = true;

        let json = serde_json::to_value(&c).unwrap();
        let s = json["subject"].as_str().unwrap();
        assert!(s.contains('\u{FFFD}'), "phải giữ ký tự thay thế U+FFFD");
        assert_eq!(json["hasInvalidUtf8"], true);

        // Vòng đi vòng về qua chuỗi JSON thật, không chỉ qua Value.
        let text = serde_json::to_string(&c).unwrap();
        let lai: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(lai["subject"], json["subject"]);
    }
}
