//! Hợp đồng dữ liệu của trình xem diff — DIFF-01, DIFF-06.
//!
//! Đây là thứ mà `get_file_diff` trả lên webview và là thứ mà 03-04 dựng giao diện
//! từ đó. Mọi struct/enum dùng `#[serde(rename_all = "camelCase")]`; tên khoá phải
//! khớp **từng chữ** với `src/lib/ipc.ts`. Lệch một bên **không** gây lỗi biên dịch ở
//! cả hai phía — giao diện chỉ nhận `undefined` rồi hiện trống, đúng lớp lỗi mà
//! Phase 2 phải ghim bằng test ở cả hai bên.

use serde::Serialize;

/// Loại của một dòng trong bản vá.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LineKind {
    /// Dòng không đổi, có mặt ở **cả hai** phía.
    Context,
    /// Dòng chỉ có ở phía **mới**.
    Added,
    /// Dòng chỉ có ở phía **cũ**.
    Removed,
}

/// Một dòng trong một hunk.
///
/// # Vì sao `old_line` và `new_line` là hai trường riêng, đều `Option`
///
/// Chế độ **hai cột** (DIFF-02) dựng bố cục từ đúng hai số này: dòng ngữ cảnh chiếm
/// một hàng ở cả hai cột, dòng `added` chiếm một hàng chỉ ở cột phải, dòng `removed`
/// chỉ ở cột trái. Gộp thành một `line_no` duy nhất thì hai cột lệch hàng, và lệch
/// hàng chính là lớp lỗi đã gây hai vòng checkpoint thất bại ở Phase 2 (đồ thị lệch
/// so với danh sách commit).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: LineKind,
    /// Nội dung dòng **không** kèm ký tự tiền tố (dấu cách / `+` / `-`) và **không**
    /// kèm ký tự kết thúc dòng (`\n` hay `\r\n`).
    ///
    /// Giải mã lossy — đây là trường **hiển thị**. Đường bản vá của Phase 5 không
    /// được dùng trường này; nó cần byte thô từ đầu tới cuối (WORK-04).
    pub content: String,
    /// Số dòng ở phía **cũ**, đếm từ 1. `None` với dòng `Added`.
    pub old_line: Option<u32>,
    /// Số dòng ở phía **mới**, đếm từ 1. `None` với dòng `Removed`.
    pub new_line: Option<u32>,
    /// `true` khi ngay sau dòng này git in `\ No newline at end of file`.
    ///
    /// Bản thân dòng đó **không** phải một `DiffLine` — đọc nó thành dòng nội dung
    /// sẽ thêm một dòng giả vào diff.
    pub no_newline_at_eof: bool,
}

/// Một khối thay đổi liền mạch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    /// Phần văn bản sau cặp `@@` thứ hai. Git đặt tên hàm chứa hunk vào đó khi tìm
    /// được; hữu ích cho DIFF-03 (nhảy giữa các hunk) để hiện người dùng đang ở đâu.
    /// Rỗng khi git không in gì.
    pub heading: String,
    pub lines: Vec<DiffLine>,
}

/// Bốn dạng kết quả của một lần xem diff, cộng dạng "không đổi".
///
/// Serialize theo `#[serde(tag = "kind")]` để phía TypeScript đọc được bằng
/// *discriminated union*: `{ kind: "text", hunks: [...], truncated: false }`.
/// Nhờ vậy giao diện phân nhánh theo `kind` mà TypeScript thu hẹp kiểu giúp — không
/// cần kiểm `if ('hunks' in d)`.
///
/// # DIFF-06 được chặn ở tầng **kiểu**, không ở tầng logic
///
/// [`DiffKind::Binary`] **không có** trường nội dung. Một cài đặt lỡ tay muốn nhét
/// byte tệp nhị phân vào payload sẽ không biên dịch được, chứ không phải đi qua một
/// nhánh `if` nào đó mà người sau có thể xoá (T-03-13).
///
/// # ⚠️ `rename_all` phải lặp lại trên **từng biến thể** — đã đo, không đọc tài liệu
///
/// `#[serde(rename_all = "camelCase")]` đặt ở **cấp enum** chỉ đổi tên **biến thể**
/// (`LfsPointer` thành `lfsPointer`). Nó **không** chạm tới tên **trường bên trong**
/// biến thể. Bản đầu của tệp này chỉ có một dòng ở cấp enum và serialize ra
/// `old_size`/`new_size` — trong khi `src/lib/ipc.ts` đọc `oldSize`/`newSize`, tức
/// giao diện nhận `undefined` và hiện kích thước trống, **không** lỗi biên dịch ở
/// bên nào.
///
/// Đây đúng lớp lỗi mà CONTEXT.md gọi là "lệch một bên không gây lỗi biên dịch ở cả
/// hai phía". Nó bị bắt vì test `binary_khong_mang_byte_noi_dung` liệt kê **toàn bộ**
/// khoá JSON rồi so bằng, chứ không chỉ kiểm khoá mình mong có mặt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DiffKind {
    /// Tệp văn bản: các hunk đã phân tích.
    #[serde(rename_all = "camelCase")]
    Text {
        hunks: Vec<Hunk>,
        /// `true` khi bản vá bị cắt ở `MAX_HUNKS` hoặc `MAX_DIFF_LINES`.
        truncated: bool,
    },
    /// Tệp nhị phân theo phán quyết của **chính git** (`--numstat` in hai dấu gạch).
    /// Chỉ có kích thước hai phía — không byte nội dung nào.
    #[serde(rename_all = "camelCase")]
    Binary { old_size: u64, new_size: u64 },
    /// Tệp vượt ngưỡng. `size` là `max(phía cũ, phía mới)`; `limit` là ngưỡng đang áp.
    /// Giao diện hiện cả hai số, nên người dùng biết vượt bao nhiêu chứ không chỉ
    /// biết "quá lớn".
    #[serde(rename_all = "camelCase")]
    TooLarge { size: u64, limit: u64 },
    /// Con trỏ Git LFS. `oid` và `size` đọc **từ nội dung con trỏ**, không phải kích
    /// thước của chính tệp con trỏ (~130 byte) — hai số hoàn toàn khác nhau.
    #[serde(rename_all = "camelCase")]
    LfsPointer { oid: String, size: u64 },
    /// Hai phía giống hệt nhau. Xảy ra thật với commit chỉ đổi mode tệp
    /// (`100644` sang `100755`): git không in hunk nào.
    Unchanged,
}

/// Kết quả xem diff của **một** tệp trong **một** commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiff {
    /// Đường dẫn ở phía **mới**.
    pub path: String,
    /// Đường dẫn ở phía **cũ**, chỉ khác `None` khi tệp bị đổi tên hay sao chép.
    pub old_path: Option<String>,
    /// Chữ trạng thái của git kèm điểm tương đồng: `M`, `A`, `D`, `R100`, `C75`.
    /// Là `String` chứ không `char` vì đúng lý do đã ghi ở
    /// [`crate::commands::history::FileChange`].
    pub status: String,
    pub kind: DiffKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Hợp đồng IPC.** `#[serde(tag = "kind")]` phải sinh đúng khoá `kind` với
    /// đúng năm giá trị, và các trường phải nằm **cùng cấp** với `kind` (không lồng
    /// trong một object con) — `src/lib/ipc.ts` khai *discriminated union* theo hình
    /// dạng phẳng đó.
    #[test]
    fn diff_kind_serialize_thanh_discriminated_union_phang() {
        let v = serde_json::to_value(DiffKind::Text {
            hunks: Vec::new(),
            truncated: false,
        })
        .unwrap();
        assert_eq!(v["kind"], "text");
        assert!(v["hunks"].is_array(), "`hunks` phải cùng cấp với `kind`");
        assert_eq!(v["truncated"], false);

        let v = serde_json::to_value(DiffKind::Binary {
            old_size: 10,
            new_size: 20,
        })
        .unwrap();
        assert_eq!(v["kind"], "binary");
        assert_eq!(v["oldSize"], 10);
        assert_eq!(v["newSize"], 20);

        let v = serde_json::to_value(DiffKind::TooLarge {
            size: 9_000_000,
            limit: 5_242_880,
        })
        .unwrap();
        assert_eq!(v["kind"], "tooLarge");
        assert_eq!(v["size"], 9_000_000);
        assert_eq!(v["limit"], 5_242_880);

        let v = serde_json::to_value(DiffKind::LfsPointer {
            oid: "ab".repeat(32),
            size: 12345,
        })
        .unwrap();
        assert_eq!(v["kind"], "lfsPointer");
        assert_eq!(v["size"], 12345);

        let v = serde_json::to_value(DiffKind::Unchanged).unwrap();
        assert_eq!(v["kind"], "unchanged");
    }

    /// `DiffKind::Binary` **không được** có trường nội dung — T-03-13 chặn ở tầng kiểu.
    /// Test đọc thẳng JSON: một trường nội dung thêm vào sẽ xuất hiện ở đây.
    #[test]
    fn binary_khong_mang_byte_noi_dung() {
        let v = serde_json::to_value(DiffKind::Binary {
            old_size: 4096,
            new_size: 8192,
        })
        .unwrap();
        let obj = v.as_object().expect("phải là object");
        let mut khoa: Vec<&str> = obj.keys().map(String::as_str).collect();
        khoa.sort_unstable();
        assert_eq!(
            khoa,
            vec!["kind", "newSize", "oldSize"],
            "Binary chỉ được mang kích thước; bất kỳ khoá nào khác là một đường rò \
             nội dung tệp nhị phân lên webview (T-03-13)"
        );
    }

    /// Tên khoá của `DiffLine` và `Hunk` — `src/lib/ipc.ts` đọc đúng những tên này.
    #[test]
    fn hunk_va_diff_line_dung_ten_khoa_camel_case() {
        let hunk = Hunk {
            old_start: 3,
            old_count: 3,
            new_start: 3,
            new_count: 4,
            heading: "fn main()".into(),
            lines: vec![DiffLine {
                kind: LineKind::Added,
                content: "x".into(),
                old_line: None,
                new_line: Some(4),
                no_newline_at_eof: true,
            }],
        };
        let v = serde_json::to_value(&hunk).unwrap();
        for khoa in [
            "oldStart",
            "oldCount",
            "newStart",
            "newCount",
            "heading",
            "lines",
        ] {
            assert!(v.get(khoa).is_some(), "Hunk phải có khoá `{khoa}`");
        }

        let dong = &v["lines"][0];
        for khoa in ["kind", "content", "oldLine", "newLine", "noNewlineAtEof"] {
            assert!(dong.get(khoa).is_some(), "DiffLine phải có khoá `{khoa}`");
        }
        assert_eq!(dong["kind"], "added");
        assert!(dong["oldLine"].is_null(), "dòng `added` không có số dòng cũ");
        assert_eq!(dong["newLine"], 4);
        assert_eq!(dong["noNewlineAtEof"], true);
    }

    /// Ba giá trị của `LineKind` phải serialize thành đúng ba chuỗi thường.
    #[test]
    fn line_kind_serialize_thanh_chuoi_thuong() {
        assert_eq!(serde_json::to_value(LineKind::Context).unwrap(), "context");
        assert_eq!(serde_json::to_value(LineKind::Added).unwrap(), "added");
        assert_eq!(serde_json::to_value(LineKind::Removed).unwrap(), "removed");
    }

    /// `FileDiff` lồng `DiffKind` dưới khoá `kind`, và `DiffKind` tự nó có khoá
    /// `kind` phân biệt dạng. Ghim hình dạng thật để 03-04 không phải đoán.
    #[test]
    fn file_diff_long_diff_kind_duoi_khoa_kind() {
        let fd = FileDiff {
            path: "a/b.txt".into(),
            old_path: None,
            status: "M".into(),
            kind: DiffKind::Unchanged,
        };
        let v = serde_json::to_value(&fd).unwrap();
        assert_eq!(v["path"], "a/b.txt");
        assert!(v["oldPath"].is_null());
        assert_eq!(v["status"], "M");
        assert_eq!(
            v["kind"]["kind"], "unchanged",
            "DiffKind nằm dưới khoá `kind` và tự nó có khoá `kind` phân biệt dạng"
        );
    }
}
