//! Bộ phân tích `git diff --word-diff=porcelain` — diff mức **từ** (03-03).
//!
//! RED phase: chỉ có chữ ký, chưa có cài đặt. Cài đặt ở commit GREEN ngay sau.

use crate::domain::diff::Span;

/// Một dòng nguồn đã dựng lại từ porcelain, cộng các khoảng chữ thay đổi trong nó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordLine {
    pub line_no: u32,
    pub content: String,
    pub spans: Vec<Span>,
}

/// Kết quả phân tích một đầu ra `--word-diff=porcelain`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WordDiffParse {
    pub old_lines: Vec<WordLine>,
    pub new_lines: Vec<WordLine>,
    pub skipped: usize,
}

/// Phân tích đầu ra `git diff --word-diff=porcelain`.
pub fn parse_word_diff(_stdout: &[u8]) -> WordDiffParse {
    WordDiffParse::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Header đầy đủ, để test không chạy trên một đầu vào đã bị đơn giản hoá tới mức
    /// không còn giống đầu ra git.
    const HEADER: &str =
        "diff --git a/f.txt b/f.txt\nindex 1234567..89abcde 100644\n--- a/f.txt\n+++ b/f.txt\n";

    fn p(s: &str) -> WordDiffParse {
        parse_word_diff(s.as_bytes())
    }

    /// `(line_no, content)` của một phía — dạng gọn để ghim trong assertion.
    fn cap(v: &[WordLine]) -> Vec<(u32, &str)> {
        v.iter().map(|w| (w.line_no, w.content.as_str())).collect()
    }

    /// Chuỗi con mà một span trỏ tới. Panic khi span vượt biên hoặc cắt giữa ký tự —
    /// đó là điều ta muốn thấy đỏ.
    fn chu(w: &WordLine, i: usize) -> &str {
        let s = w.spans[i];
        &w.content[s.start..s.end]
    }

    // -----------------------------------------------------------------------
    // Ca của chủ dự án — nguyên văn
    // -----------------------------------------------------------------------

    /// **Ca mà chủ dự án nêu tên tường minh.** Đầu vào là đầu ra porcelain THẬT
    /// (đo trên git 2.54.0.windows.1).
    ///
    /// Yêu cầu của chủ dự án: `if (typeof cellData == "object")` → `===`, và **chỉ**
    /// `==`/`===` được tô, phần còn lại của dòng để nguyên.
    ///
    /// Khẳng định then chốt là **span HẸP HƠN cả dòng**. Một cài đặt trả về một span
    /// phủ toàn dòng qua được mọi phép kiểm "có span" — và nó đúng là thứ chủ dự án
    /// nói là KHÔNG muốn.
    #[test]
    fn ca_chu_du_an_chi_to_phan_dau_bang_khong_to_ca_dong() {
        let ra = p(&format!(
            "{HEADER}@@ -2 +2 @@ function f(cellData) {{\n  if (typeof cellData \n-==\n+===\n  \"object\") {{\n~\n"
        ));

        assert_eq!(ra.skipped, 0, "đầu ra porcelain hợp lệ không được bỏ dòng nào");
        assert_eq!(ra.new_lines.len(), 1, "một dòng nguồn → một WordLine phía mới");
        assert_eq!(ra.old_lines.len(), 1, "và một WordLine phía cũ");

        let moi = &ra.new_lines[0];
        assert_eq!(
            moi.content, "  if (typeof cellData === \"object\") {",
            "dòng nguồn phía MỚI phải được dựng lại nguyên vẹn từ các đoạn từ"
        );
        assert_eq!(moi.spans.len(), 1, "đúng MỘT khoảng thay đổi, nhận: {:?}", moi.spans);
        assert_eq!(chu(moi, 0), "===", "khoảng phải trỏ đúng vào `===`");

        let cu = &ra.old_lines[0];
        assert_eq!(cu.content, "  if (typeof cellData == \"object\") {");
        assert_eq!(cu.spans.len(), 1);
        assert_eq!(chu(cu, 0), "==", "phía cũ trỏ vào `==`");

        // 🔴 Khẳng định then chốt của cả plan.
        for (ten, w) in [("mới", moi), ("cũ", cu)] {
            let s = w.spans[0];
            assert!(
                s.end - s.start < w.content.len(),
                "phía {ten}: khoảng phải HẸP HƠN cả dòng. Chủ dự án nêu rõ \"không chỉ \
                 tô cả dòng\" — một cài đặt trả [0, len) qua được mọi phép kiểm \"có \
                 span\" nhưng làm sai đúng điều được yêu cầu. \
                 Khoảng [{}, {}) trên dòng dài {} byte",
                s.start,
                s.end,
                w.content.len()
            );
        }
    }

    // -----------------------------------------------------------------------
    // Quy tắc đẩy ở `~` — bảng đo ba-quy-tắc × ba-fixture
    // -----------------------------------------------------------------------

    /// 🔴 **Ca `mixed` — fixture DUY NHẤT phân biệt được cả ba quy tắc đẩy.**
    ///
    /// Đầu vào là đầu ra porcelain THẬT của `mixed.txt` (đã đo lại bằng `cat -A`,
    /// khớp từng byte với điều 03-02 dán vào script fixture).
    ///
    /// **Phải ghim `line_no`, không chỉ nội dung.** Hai cấu hình sai chỉ lệch ở
    /// `line_no` chứ không lệch nội dung (xem bảng ở tài liệu module), nên một test
    /// chỉ so `content` sẽ xanh trên cả hai. Và Task 2 khớp `WordLine` với `DiffLine`
    /// **theo số dòng**, nên `line_no` sai nghĩa là span rơi vào dòng khác.
    #[test]
    fn mixed_ghim_ca_line_no_lan_noi_dung_o_ca_hai_phia() {
        let ra = p(&format!(
            "{HEADER}@@ -1,4 +1,4 @@\n a\n~\n \n~\n-DELETED\n~\n keep\n~\n+ADDED\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "a"), (2, ""), (3, "keep"), (4, "ADDED")],
            "phía MỚI: bốn dòng, dòng 2 rỗng, và `ADDED` ở dòng 4.\n\
             Sai `line_no` ở đây nghĩa là bộ đếm đã tăng cho một phía KHÔNG có dòng — \
             xem bảng ba-quy-tắc trong tài liệu module"
        );
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "a"), (2, ""), (3, "DELETED"), (4, "keep")],
            "phía CŨ: `DELETED` ở dòng 3, `keep` ở dòng 4"
        );

        assert!(
            ra.new_lines.iter().all(|w| w.spans.is_empty())
                || ra.new_lines[3].spans.len() == 1,
            "dòng chỉ-thêm được phép có span phủ cả dòng; dòng ngữ cảnh thì không"
        );
        for w in ra.new_lines.iter().take(3) {
            assert!(
                w.spans.is_empty(),
                "dòng ngữ cảnh (chung cả hai phía) KHÔNG có khoảng thay đổi. \
                 Dòng {}: {:?}",
                w.line_no,
                w.spans
            );
        }
    }

    /// 🔴 **Dòng chỉ có ở MỘT phía** — loại bỏ quy tắc "luôn đẩy cả hai phía".
    ///
    /// Đầu ra porcelain thật của `one-side.txt`. Tại `~` đầu, phía mới **không có
    /// dòng nào** ở vị trí đó.
    ///
    /// Khẳng định **SỐ LƯỢNG** `WordLine` của từng phía, không chỉ nội dung: một cài
    /// đặt "luôn đẩy cả hai" cho đúng nội dung ở mọi dòng khác và chỉ sai ở số lượng
    /// của phía trống (nó sinh một bản ghi rỗng GIẢ).
    ///
    /// ⚠️ Ca này **không** phân biệt được `len() > 0` với cờ `touched` — đã đo, hai
    /// quy tắc cho y hệt nhau ở đây. Mutation đó phải chạy trên `mixed`.
    #[test]
    fn one_side_khong_sinh_ban_ghi_rong_gia_o_phia_khong_co_dong() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n-DELETED_LINE\n~\n keep2\n~\n+ADDED_LINE\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            ra.new_lines.len(),
            2,
            "phía MỚI có ĐÚNG hai dòng. Ba nghĩa là một bản ghi rỗng giả đã được đẩy \
             ở chỗ phía mới không có dòng nào — đó là quy tắc \"luôn đẩy cả hai phía\" \
             và nó sai. Nhận: {:?}",
            cap(&ra.new_lines)
        );
        assert_eq!(
            ra.old_lines.len(),
            2,
            "phía CŨ cũng đúng hai dòng. Nhận: {:?}",
            cap(&ra.old_lines)
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "keep2"), (2, "ADDED_LINE")]);
        assert_eq!(cap(&ra.old_lines), vec![(1, "DELETED_LINE"), (2, "keep2")]);
    }

    /// Dòng rỗng trong nguồn: porcelain in một đoạn `␠` với nội dung rỗng.
    ///
    /// Đầu ra thật của `empty-line.txt`. Quy tắc `len() > 0` bỏ hẳn dòng 2 ở đây —
    /// và mất một dòng làm **mọi** `line_no` sau nó lệch một nấc.
    #[test]
    fn dong_rong_van_la_mot_dong_va_khong_lam_lech_dong_sau() {
        let ra = p(&format!(
            "{HEADER}@@ -1,5 +1,5 @@\n header\n~\n \n~\n func a() {{\n~\n   x \n-==\n+===\n  1\n~\n }}\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![
                (1, "header"),
                (2, ""),
                (3, "func a() {"),
                (4, "  x === 1"),
                (5, "}")
            ],
            "dòng rỗng ở vị trí 2 phải CÓ MẶT với nội dung rỗng. Bỏ nó đi (quy tắc \
             `len() > 0`) làm mọi dòng sau lệch một nấc — và Task 2 khớp span theo \
             số dòng"
        );
        assert_eq!(
            cap(&ra.old_lines),
            vec![
                (1, "header"),
                (2, ""),
                (3, "func a() {"),
                (4, "  x == 1"),
                (5, "}")
            ]
        );

        // Span nằm đúng dòng 4, và hẹp hơn cả dòng.
        let d4 = &ra.new_lines[3];
        assert_eq!(d4.spans.len(), 1, "dòng 4 có đúng một khoảng");
        assert_eq!(chu(d4, 0), "===");
        assert!(d4.spans[0].end - d4.spans[0].start < d4.content.len());
    }

    /// **Bộ đếm đi qua NHIỀU dòng rỗng liên tiếp.**
    ///
    /// Ca này và ca `mixed` ghim hai thứ khác nhau và không thay được nhau: ở đây bộ
    /// đếm đi qua nhiều dòng rỗng liền nhau, còn ở `mixed` bộ đếm gặp tình huống
    /// **một phía không có dòng**.
    #[test]
    fn hai_dong_rong_lien_tiep_ghim_line_no() {
        let ra = p(&format!(
            "{HEADER}@@ -1,7 +1,7 @@\n header\n~\n \n~\n mid\n~\n \n~\n func a() {{\n~\n   x \n-==\n+===\n  1\n~\n }}\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![
                (1, "header"),
                (2, ""),
                (3, "mid"),
                (4, ""),
                (5, "func a() {"),
                (6, "  x === 1"),
                (7, "}")
            ],
            "bộ đếm phải tiến qua TỪNG dòng rỗng. Mỗi dòng rỗng bị bỏ làm mọi dòng \
             sau lệch thêm một nấc, nên ca hai dòng rỗng lệch HAI nấc"
        );
    }

    // -----------------------------------------------------------------------
    // Hình dạng span
    // -----------------------------------------------------------------------

    /// **Nhiều khoảng trong một dòng**, hai chỗ cách xa nhau → đúng **hai** span,
    /// không gộp thành một span phủ từ chỗ đầu tới chỗ cuối.
    #[test]
    fn hai_cho_sua_cach_xa_nhau_cho_hai_span_rieng() {
        let ra = p(&format!(
            "{HEADER}@@ -1 +1 @@\n let \n-a\n+A\n  = b + \n-c\n+C\n  + d\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[0];
        assert_eq!(moi.content, "let A = b + C + d");
        assert_eq!(
            moi.spans.len(),
            2,
            "hai chỗ sửa CÁCH XA nhau phải cho HAI khoảng riêng. Một khoảng nghĩa là \
             cài đặt đã gộp từ chỗ đầu tới chỗ cuối và tô luôn `= b +` ở giữa. \
             Nhận: {:?} trên {:?}",
            moi.spans,
            moi.content
        );
        assert_eq!(chu(moi, 0), "A");
        assert_eq!(chu(moi, 1), "C");
        assert!(
            moi.spans[0].end < moi.spans[1].start,
            "hai khoảng phải RỜI nhau, không chạm nhau"
        );

        let cu = &ra.old_lines[0];
        assert_eq!(cu.content, "let a = b + c + d");
        assert_eq!(cu.spans.len(), 2);
        assert_eq!(chu(cu, 0), "a");
        assert_eq!(chu(cu, 1), "c");
    }

    /// **Hai đoạn `+` LIÊN TIẾP phải gộp thành một span.**
    ///
    /// Git tách được một vùng thay đổi thành nhiều đoạn liền kề. Không gộp thì 03-04
    /// vẽ hai decoration chạm nhau và một đường viền hiện ra giữa dòng.
    ///
    /// Đây là ca cho mutation "bỏ bước gộp span liền kề" — không có ca này thì
    /// mutation đó cho 0 test đỏ.
    #[test]
    fn hai_doan_them_lien_tiep_gop_thanh_mot_span() {
        let ra = p(&format!(
            "{HEADER}@@ -1 +1 @@\n giu \n-cu\n+mot\n+hai\n  giu2\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[0];
        assert_eq!(moi.content, "giu mot hai giu2");
        assert_eq!(
            moi.spans.len(),
            1,
            "hai đoạn `+` LIỀN KỀ phải gộp thành MỘT khoảng. Hai khoảng chạm nhau làm \
             03-04 vẽ hai decoration sát nhau và viền hiện giữa vùng tô. \
             Nhận: {:?} trên {:?}",
            moi.spans,
            moi.content
        );
        assert_eq!(
            chu(moi, 0),
            "mot hai",
            "khoảng gộp phải phủ cả hai đoạn cộng dấu cách nối giữa chúng"
        );
    }

    /// **Dòng chỉ THÊM** (không có đối ứng bên cũ): span phủ toàn dòng là **đúng** ở
    /// đây — ca này khác hẳn ca "dòng sửa", và trộn hai ca là cách dễ nhất để có một
    /// cài đặt sai mà test vẫn xanh.
    #[test]
    fn dong_chi_them_duoc_phep_co_span_phu_ca_dong() {
        let ra = p(&format!(
            "{HEADER}@@ -1,1 +1,2 @@\n giu\n~\n+dong moi hoan toan\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(cap(&ra.new_lines), vec![(1, "giu"), (2, "dong moi hoan toan")]);
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "giu")],
            "phía cũ KHÔNG có dòng thứ hai"
        );

        let them = &ra.new_lines[1];
        assert_eq!(them.spans.len(), 1);
        let s = them.spans[0];
        assert_eq!(
            (s.start, s.end),
            (0, them.content.len()),
            "với dòng chỉ-thêm, span phủ TOÀN dòng là đúng: cả dòng đều là chữ mới"
        );
    }

    /// **Dòng không đổi** giữa hai dòng đã sửa: `spans` rỗng, nhưng dòng **vẫn ra**.
    #[test]
    fn dong_khong_doi_co_spans_rong_nhung_van_ra() {
        let ra = p(&format!(
            "{HEADER}@@ -1,3 +1,3 @@\n a \n-x\n+X\n\n~\n khong doi\n~\n b \n-y\n+Y\n\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(ra.new_lines.len(), 3, "cả ba dòng phải ra");
        assert_eq!(ra.new_lines[1].content, "khong doi");
        assert!(
            ra.new_lines[1].spans.is_empty(),
            "dòng không đổi KHÔNG có khoảng thay đổi. Nhận: {:?}",
            ra.new_lines[1].spans
        );
        assert!(!ra.new_lines[0].spans.is_empty(), "tiền đề: dòng 1 có sửa");
        assert!(!ra.new_lines[2].spans.is_empty(), "tiền đề: dòng 3 có sửa");
    }

    // -----------------------------------------------------------------------
    // UTF-8 nhiều byte — T-03-18
    // -----------------------------------------------------------------------

    /// **Mọi khoảng phải nằm trên biên ký tự ở CẢ HAI đầu.**
    ///
    /// Đầu vào là đầu ra porcelain THẬT trên `xéy dỏng test` → `xéy dỏng TEST`
    /// (đo trên git 2.54; `é` là 2 byte, `ỏ` là 3 byte).
    ///
    /// Cổng này tồn tại để người sau đổi sang `--word-diff-regex=.` sẽ thấy nó ĐỎ,
    /// chứ không phát hiện qua báo lỗi người dùng: `.` trong regex của git khớp theo
    /// **byte**, nên một ký tự tiếng Việt ba byte bị tách thành ba "từ" và khoảng có
    /// thể cắt vào giữa ký tự.
    #[test]
    fn khoang_khong_bao_gio_cat_giua_ky_tu_nhieu_byte() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n xey dong test\n~\n xéy dỏng \n-test\n+TEST\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[1];
        assert_eq!(
            moi.content, "xéy dỏng TEST",
            "byte UTF-8 phải đi qua nguyên vẹn khi dựng lại dòng"
        );

        // Tiền đề: dòng PHẢI có ký tự nhiều byte, nếu không cổng này luôn xanh một
        // cách vô nghĩa (cổng tự vô hiệu hoá).
        assert!(
            moi.content.chars().count() < moi.content.len(),
            "tiền đề: dòng phải chứa ký tự nhiều byte, nếu không cổng biên ký tự \
             không kiểm được gì. {} ký tự / {} byte",
            moi.content.chars().count(),
            moi.content.len()
        );

        for w in ra.new_lines.iter().chain(ra.old_lines.iter()) {
            for s in &w.spans {
                assert!(
                    w.content.is_char_boundary(s.start),
                    "đầu khoảng {} cắt giữa một ký tự của {:?} — cắt chuỗi ở đó làm \
                     `&content[..]` panic ở Rust và hỏng UTF-8 ở JS (T-03-18)",
                    s.start,
                    w.content
                );
                assert!(
                    w.content.is_char_boundary(s.end),
                    "cuối khoảng {} cắt giữa một ký tự của {:?} (T-03-18)",
                    s.end,
                    w.content
                );
                assert!(
                    s.end <= w.content.len(),
                    "khoảng [{}, {}) vượt biên chuỗi dài {} byte (T-03-17)",
                    s.start,
                    s.end,
                    w.content.len()
                );
            }
        }

        assert_eq!(chu(moi, 0), "TEST", "khoảng phải trỏ đúng vào từ đã đổi");
        assert!(
            moi.spans[0].start > 0,
            "khoảng phải bắt đầu SAU phần `xéy dỏng ` không đổi — bắt đầu ở 0 nghĩa \
             là cài đặt đã tô cả dòng"
        );
    }

    // -----------------------------------------------------------------------
    // Ca biên của định dạng
    // -----------------------------------------------------------------------

    /// **Tiền tố là đúng MỘT ký tự, và ký tự đó có thể là dấu cách.**
    ///
    /// Cắt bằng `trim_start()` thay vì bỏ đúng một byte sẽ ăn mất khoảng trắng THỤT
    /// LỀ của nội dung thật, làm lệch mọi khoảng sau đó trên dòng.
    #[test]
    fn thut_le_dau_dong_khong_bi_an_mat() {
        let ra = p(&format!(
            "{HEADER}@@ -1 +1 @@\n        sau tam dau cach \n-cu\n+moi\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[0];
        assert_eq!(
            moi.content, "       sau tam dau cach moi",
            "một byte tiền tố bị bỏ, BẢY dấu cách thụt lề còn lại phải nguyên vẹn. \
             `trim_start()` ăn hết cả tám và làm mọi khoảng sau đó lệch bảy byte"
        );
        assert!(
            moi.content.starts_with("       s"),
            "thụt lề phải còn. Nhận: {:?}",
            moi.content
        );
        assert_eq!(chu(moi, 0), "moi", "khoảng vẫn trỏ đúng sau khi giữ thụt lề");
    }

    /// **Bộ đệm phải được XOÁ ở mỗi `~`.** Không xoá thì mọi dòng nối lại thành một.
    #[test]
    fn bo_dem_duoc_xoa_o_moi_dau_ngã() {
        let ra = p(&format!("{HEADER}@@ -1,3 +1,3 @@\n mot\n~\n hai\n~\n ba\n~\n"));

        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "mot"), (2, "hai"), (3, "ba")],
            "ba dòng RIÊNG. Nội dung kiểu `mothai` hay `mothaiba` nghĩa là bộ đệm \
             không được xoá ở `~`"
        );
        for w in &ra.new_lines {
            assert!(
                !w.content.contains("mothai"),
                "dòng {:?} chứa hai dòng nguồn dính vào nhau",
                w.content
            );
        }
    }

    /// **CRLF**: `\r` không được lọt vào `content` — cùng bất biến với `parse_patch`.
    ///
    /// Đầu vào là đầu ra porcelain THẬT của `crlf.txt` (đo bằng `od -c`). Lưu ý hình
    /// dạng đặc biệt: git tách `\r` thành một đoạn từ **riêng** (` \r`) nằm SAU đoạn
    /// `+BETA DA SUA`, chứ không dính vào đoạn đó.
    #[test]
    fn crlf_khong_lot_vao_noi_dung() {
        let buf = b"diff --git a/c.txt b/c.txt\nindex b4ec4d1..bf24d7b 100644\n\
                    --- a/c.txt\n+++ b/c.txt\n@@ -1,3 +1,3 @@\n alpha\r\n~\n\
                    -beta\n+BETA DA SUA\n \r\n~\n gamma\r\n~\n";
        let ra = parse_word_diff(buf);

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "alpha"), (2, "BETA DA SUA"), (3, "gamma")],
            "`\\r` là ký tự VÔ HÌNH: lọt vào `content` thì phép so `word_line.content \
             == diff_line.content` ở Task 2 trượt trên MỌI dòng của repo CRLF, và \
             word-level tắt im lặng cho cả tệp"
        );
        for w in ra.new_lines.iter().chain(ra.old_lines.iter()) {
            assert!(
                !w.content.contains('\r'),
                "không `content` nào được chứa `\\r`. Nhận: {:?}",
                w.content
            );
        }
    }

    /// 🔴 **Tệp không có dòng trống cuối: `~` VẪN ĐƯỢC IN, và porcelain KHÔNG in
    /// `\ No newline at end of file`.**
    ///
    /// Đầu vào là đầu ra porcelain THẬT của `no-eol.txt` (đo trên git 2.54). Đây là
    /// điểm mà porcelain khác `--unified`: cùng tệp đó, `--unified=3` in
    /// `\ No newline at end of file` **hai lần**, còn porcelain không in dòng nào.
    ///
    /// Hệ quả bắt buộc: `no_newline_at_eof` **không lấy được** từ porcelain, nên nó
    /// không thuộc phạm vi bộ phân tích này — trường đó do `parse_patch` cung cấp.
    #[test]
    fn khong_newline_cuoi_van_in_dau_nga_va_khong_co_dong_bao_hieu() {
        let ra = p(&format!(
            "{HEADER}@@ -1,3 +1,3 @@\n mot\n~\n hai\n~\n-ba\n+BA DA SUA\n~\n"
        ));

        assert_eq!(
            ra.skipped, 0,
            "porcelain của tệp không-newline-cuối là đầu vào HỢP LỆ; `skipped > 0` ở \
             đây làm Task 2 bỏ TOÀN BỘ spans của tệp"
        );
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "mot"), (2, "hai"), (3, "BA DA SUA")],
            "dòng cuối vẫn phải ra đầy đủ dù tệp không kết thúc bằng newline"
        );
        assert_eq!(cap(&ra.old_lines), vec![(1, "mot"), (2, "hai"), (3, "ba")]);
    }

    /// Nếu một phiên bản git nào đó **có** in `\ No newline at end of file`, nó
    /// **không được** tính vào `skipped`.
    ///
    /// Lý do: bước 6 của Task 2 bỏ **toàn bộ** spans khi `skipped > 0`, nên đếm một
    /// dòng vô hại vào đó sẽ tắt word-level cho cả tệp. Bỏ qua nó như dòng header.
    #[test]
    fn dong_bao_khong_newline_khong_duoc_tinh_vao_skipped() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n giu\n~\n-cu\n\\ No newline at end of file\n+moi\n\\ No newline at end of file\n~\n"
        ));

        assert_eq!(
            ra.skipped, 0,
            "`\\ No newline at end of file` là dòng VÔ HẠI — đếm nó vào `skipped` làm \
             Task 2 tắt word-level cho cả tệp vì một dòng không mang thông tin nào"
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "giu"), (2, "moi")]);
        assert_eq!(cap(&ra.old_lines), vec![(1, "giu"), (2, "cu")]);
    }

    /// Đầu ra **rỗng**: 0 dòng, không lỗi, không panic.
    #[test]
    fn dau_ra_rong_cho_khong_dong_khong_loi() {
        let ra = parse_word_diff(b"");
        assert_eq!(ra.old_lines.len(), 0);
        assert_eq!(ra.new_lines.len(), 0);
        assert_eq!(ra.skipped, 0);

        // Chỉ header, không hunk nào.
        let ra = p(HEADER);
        assert_eq!(ra.new_lines.len(), 0);
        assert_eq!(
            ra.skipped, 0,
            "dòng header không phải dòng bị bỏ — đếm chúng làm Task 2 tắt word-level \
             cho MỌI tệp"
        );
    }

    /// **Đầu vào không phải porcelain** (ai đó truyền stdout của diff thường vào):
    /// `skipped > 0` và không panic. Hỏng to thì phải thấy được.
    #[test]
    fn dau_vao_khong_phai_porcelain_thi_skipped_khac_khong() {
        // Diff thường: dòng `-cu` và `+moi` KHÔNG kèm `~` nào, và có dòng `?` lạ.
        let ra = p(&format!("{HEADER}@@ -1,2 +1,2 @@\n giu\n?rac khong hieu\n-cu\n+moi\n"));
        assert!(
            ra.skipped > 0,
            "một dòng không khớp khuôn nào phải được ĐẾM, không im lặng bỏ qua — \
             đó là cách duy nhất Task 2 biết mà tắt word-level thay vì tô sai chỗ \
             (T-03-23)"
        );
    }

    /// `@@` khởi tạo hai bộ đếm theo `old_start`/`new_start` của **chính hunk đó**.
    /// Số dòng là cách DUY NHẤT khớp `WordLine` với `DiffLine` ở Task 2.
    #[test]
    fn dau_hunk_khoi_tao_bo_dem_theo_start_cua_chinh_no() {
        let ra = p(&format!(
            "{HEADER}@@ -10,2 +20,2 @@ fn x()\n giu\n~\n-cu\n+moi\n~\n"
        ));

        assert_eq!(
            cap(&ra.old_lines),
            vec![(10, "giu"), (11, "cu")],
            "phía cũ bắt đầu từ `old_start` = 10"
        );
        assert_eq!(
            cap(&ra.new_lines),
            vec![(20, "giu"), (21, "moi")],
            "phía mới bắt đầu từ `new_start` = 20 — HAI bộ đếm độc lập, không phải một"
        );
    }

    /// Hai hunk trong một tệp: bộ đếm phải **đặt lại** theo `@@` của hunk thứ hai.
    #[test]
    fn hunk_thu_hai_dat_lai_bo_dem() {
        let ra = p(&format!(
            "{HEADER}@@ -1,1 +1,1 @@\n-a\n+A\n~\n@@ -50,1 +50,1 @@\n-b\n+B\n~\n"
        ));

        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "A"), (50, "B")],
            "hunk thứ hai KHÔNG được tiếp tục bộ đếm của hunk đầu — nó đặt lại theo \
             `@@` của chính nó"
        );
    }

    /// Dòng `diff --git` / `index` / `---` / `+++` bị bỏ qua và **không** tính vào
    /// `skipped`.
    ///
    /// ⚠️ `---` và `+++` là cái bẫy thật: chúng bắt đầu bằng `-` và `+`, đúng hai
    /// tiền tố của đoạn từ. Đọc chúng thành đoạn từ sẽ nhét `-- a/f.txt` vào bộ đệm
    /// phía cũ.
    #[test]
    fn header_khong_bi_doc_thanh_doan_tu() {
        let ra = p(&format!("{HEADER}@@ -1 +1 @@\n-cu\n+moi\n~\n"));

        assert_eq!(ra.skipped, 0, "header không phải dòng bị bỏ");
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "cu")],
            "`--- a/f.txt` bắt đầu bằng `-` nhưng KHÔNG phải đoạn từ. Nội dung kiểu \
             `-- a/f.txtcu` nghĩa là header đã bị đọc thành đoạn từ"
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "moi")]);
        for w in ra.old_lines.iter().chain(ra.new_lines.iter()) {
            assert!(
                !w.content.contains("f.txt"),
                "đường dẫn từ header lọt vào nội dung dòng: {:?}",
                w.content
            );
        }
    }
}
