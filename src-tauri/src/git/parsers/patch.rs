//! Bộ phân tích unified diff viết tay — DIFF-01.
//!
//! # Vì sao viết tay chứ không dùng crate
//!
//! STACK.md đã loại từng phương án: `patch` (0.7.0) xuất bản lần cuối 2022-12,
//! `unidiff` có ~0.9M lượt tải cả đời — quá ít cho một thứ nằm trên đường tới hạn.
//! `similar`/`imara-diff`/`diffy` *tính* diff, việc mà `git diff` đã làm xong. Định
//! dạng unified diff ổn định hàng chục năm và bộ phân tích này ~200 dòng.
//!
//! # Vì sao ở phía **Rust**, không phải TypeScript
//!
//! Phase 5 (staging theo khối) cần đúng bộ phân tích này để dựng lại bản vá từng
//! hunk. Nếu nó sống ở frontend thì Phase 5 phải viết bản thứ hai, và hai bộ phân
//! tích cùng một định dạng sẽ lệch nhau ở đúng những ca biên mà module này tồn tại
//! để xử lý.
//!
//! # Đầu vào là BYTE
//!
//! `parse_patch` nhận `&[u8]` chứ không `&str`: đầu ra của git không bảo đảm UTF-8.
//! Giải mã lossy **chỉ** ở trường `content` — đó là trường hiển thị. Đường bản vá
//! của Phase 5 sẽ đọc lại byte thô từ `git diff`, không đi qua trường này (WORK-04).

use crate::domain::diff::{DiffLine, Hunk, LineKind};

/// Chặn trên số hunk trong một tệp — T-03-11.
///
/// Một commit do công cụ sinh mã tạo ra (`schema.sql` 200k dòng, tệp lock) có thể có
/// hàng chục nghìn hunk. Trả hết sẽ nạp cả vào RAM rồi serialize sang JSON và làm đơ
/// webview — tự gây từ chối dịch vụ. Cắt và bật `truncated` để giao diện nói rõ người
/// dùng đang xem một phần.
pub const MAX_HUNKS: usize = 2_000;

/// Chặn trên **tổng** số dòng diff trong một tệp — T-03-11.
///
/// Áp song song với [`MAX_HUNKS`]: một tệp có thể chỉ một hunk nhưng hunk đó dài
/// 2 triệu dòng (tệp sinh tự động bị ghi đè toàn bộ). Một mình `MAX_HUNKS` không
/// chặn được ca đó.
pub const MAX_DIFF_LINES: usize = 100_000;

/// Kết quả phân tích một bản vá.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PatchParse {
    pub hunks: Vec<Hunk>,
    /// `true` khi đã chạm [`MAX_HUNKS`] hoặc [`MAX_DIFF_LINES`].
    pub truncated: bool,
    /// Số dòng nằm **trong** một hunk mà bộ phân tích không hiểu.
    ///
    /// Bình thường là 0. Khác 0 thì người gọi ghi `tracing::warn!` — cùng khuôn với
    /// `skipped_records` của `parse_log` (T-03-16): một bản vá đọc sai mà im lặng là
    /// thứ không truy được nguyên nhân về sau.
    pub skipped_lines: usize,
}

/// Phân tích đầu ra `git diff --unified=N` của **một** tệp thành các hunk.
///
/// # Không đọc đường dẫn từ header — có chủ ý
///
/// Mọi dòng trước `@@` đầu tiên bị bỏ qua: `diff --git`, `index`, `---`, `+++`,
/// `old mode`, `similarity index`. Người gọi (`get_file_diff`) **đã biết** đường dẫn
/// vì chính nó truyền vào lệnh git. Đọc lại từ header là tạo một **nguồn dữ liệu thứ
/// hai** cho cùng một sự thật, và hai nguồn thì lệch được — đúng lớp lỗi
/// `REF_COL_WIDTH` mà Phase 2 phải ghim bằng test đọc chéo hai tệp.
///
/// Điều này cũng làm `/dev/null` (tệp mới thêm / tệp bị xoá) trở thành ca không cần
/// xử lý riêng: nó chỉ xuất hiện ở header, và header không được đọc.
pub fn parse_patch(stdout: &[u8]) -> PatchParse {
    // RED: chưa cài đặt.
    let _ = stdout;
    PatchParse::default()
}

#[allow(dead_code)]
fn parse_patch_chua_cai_dat(stdout: &[u8]) -> PatchParse {
    let mut ra = PatchParse::default();

    let mut hunk_dang_mo: Option<Hunk> = None;
    let mut so_dong_cu: u32 = 0;
    let mut so_dong_moi: u32 = 0;
    let mut tong_dong: usize = 0;

    for dong in tach_dong(stdout) {
        // --- Đầu hunk ------------------------------------------------------
        if dong.starts_with(b"@@") {
            if let Some(h) = hunk_dang_mo.take() {
                ra.hunks.push(h);
            }
            if ra.hunks.len() >= MAX_HUNKS {
                ra.truncated = true;
                break;
            }
            match phan_tich_dau_hunk(dong) {
                Some(h) => {
                    so_dong_cu = h.old_start;
                    so_dong_moi = h.new_start;
                    hunk_dang_mo = Some(h);
                }
                None => {
                    // Dòng bắt đầu bằng `@@` mà không phân tích được. Không mở hunk
                    // mới; những dòng sau nó sẽ rơi vào nhánh "ngoài hunk" và bị bỏ
                    // qua, chứ không gán nhầm vào hunk trước đó.
                    ra.skipped_lines += 1;
                }
            }
            continue;
        }

        // Mọi thứ trước `@@` đầu tiên là header của tệp — bỏ qua, không đếm là lỗi.
        let Some(hunk) = hunk_dang_mo.as_mut() else {
            continue;
        };

        // --- `\ No newline at end of file` ---------------------------------
        //
        // KHÔNG phải một dòng nội dung. Nó nói về dòng **ngay trước** nó. Đọc nó
        // thành `DiffLine` sẽ thêm một dòng giả vào diff và làm số dòng lệch từ đó
        // trở đi.
        if dong.starts_with(b"\\") {
            if let Some(cuoi) = hunk.lines.last_mut() {
                cuoi.no_newline_at_eof = true;
            }
            continue;
        }

        if tong_dong >= MAX_DIFF_LINES {
            ra.truncated = true;
            break;
        }

        // --- Dòng nội dung --------------------------------------------------
        //
        // Ký tự đầu là tiền tố; phần còn lại là nội dung. Một dòng **hoàn toàn rỗng**
        // (0 byte) là dòng ngữ cảnh rỗng: chuẩn nói dòng ngữ cảnh rỗng phải in một
        // dấu cách, nhưng nhiều công cụ (và chính git ở một số đường) lược bỏ dấu
        // cách đó. Coi nó là kết thúc hunk sẽ cắt mất phần còn lại của bản vá.
        let (loai, noi_dung) = match dong.first() {
            None => (LineKind::Context, &b""[..]),
            Some(b' ') => (LineKind::Context, &dong[1..]),
            Some(b'+') => (LineKind::Added, &dong[1..]),
            Some(b'-') => (LineKind::Removed, &dong[1..]),
            Some(_) => {
                // Không phải tiền tố hợp lệ. Bỏ dòng, đếm nó, đọc tiếp — một dòng
                // hỏng không được làm hỏng cả bản vá.
                ra.skipped_lines += 1;
                continue;
            }
        };

        let (old_line, new_line) = match loai {
            LineKind::Context => {
                let cap = (Some(so_dong_cu), Some(so_dong_moi));
                so_dong_cu += 1;
                so_dong_moi += 1;
                cap
            }
            LineKind::Added => {
                let cap = (None, Some(so_dong_moi));
                so_dong_moi += 1;
                cap
            }
            LineKind::Removed => {
                let cap = (Some(so_dong_cu), None);
                so_dong_cu += 1;
                cap
            }
        };

        hunk.lines.push(DiffLine {
            kind: loai,
            content: String::from_utf8_lossy(noi_dung).into_owned(),
            old_line,
            new_line,
            no_newline_at_eof: false,
        });
        tong_dong += 1;
    }

    if let Some(h) = hunk_dang_mo.take() {
        ra.hunks.push(h);
    }

    ra
}

/// Tách theo `\n` rồi **cắt `\r` cuối mỗi dòng**.
///
/// # Vì sao phải cắt `\r`
///
/// Trên repo có tệp CRLF, git in nội dung nguyên vẹn — tức mỗi dòng diff kết thúc
/// bằng `\r\n`. Không cắt thì **mọi** `DiffLine.content` mang một ký tự vô hình ở
/// cuối: trình xem hiện một ô trống thừa, tìm kiếm trong diff không khớp, và so sánh
/// nội dung ở 03-03 (diff mức từ) lệch một ký tự. Phase 2 gặp đúng lớp lỗi này ở tầng
/// SHA, nơi mã commit mang `\r` treo.
///
/// Dùng `memchr` thay vì `split(|b| *b == b'\n')`: đầu ra diff của một tệp lớn là
/// hàng MB, và `memchr` quét bằng SIMD.
fn tach_dong(buf: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut vi_tri = 0usize;
    std::iter::from_fn(move || {
        if vi_tri >= buf.len() {
            return None;
        }
        let con_lai = &buf[vi_tri..];
        let (dong, buoc) = match memchr::memchr(b'\n', con_lai) {
            Some(i) => (&con_lai[..i], i + 1),
            None => (con_lai, con_lai.len()),
        };
        vi_tri += buoc;
        Some(match dong.last() {
            Some(b'\r') => &dong[..dong.len() - 1],
            _ => dong,
        })
    })
}

/// Phân tích `@@ -<os>[,<oc>] +<ns>[,<nc>] @@[ <heading>]`.
///
/// # Số đếm khuyết nghĩa là **1**, không phải 0
///
/// Git lược bỏ số đếm khi nó bằng 1: `@@ -1 +1 @@` thay vì `@@ -1,1 +1,1 @@`. Đây là
/// dạng **thường gặp**, không phải ca hiếm — mọi tệp một dòng và mọi hunk một dòng
/// đều in ra như vậy. Đọc thành 0 làm giao diện tin rằng hunk rỗng.
fn phan_tich_dau_hunk(dong: &[u8]) -> Option<Hunk> {
    // Bỏ `@@` mở đầu.
    let sau_at = dong.strip_prefix(b"@@")?;

    // Tìm `@@` đóng. Phần giữa là hai khoảng; phần sau là heading.
    let dong_cuoi = memchr::memmem::find(sau_at, b"@@")?;
    let giua = &sau_at[..dong_cuoi];
    let heading = &sau_at[dong_cuoi + 2..];

    let mut phan = giua
        .split(|b| *b == b' ')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .into_iter();

    let cu = phan.next()?.strip_prefix(b"-")?;
    let moi = phan.next()?.strip_prefix(b"+")?;

    let (old_start, old_count) = doc_khoang(cu)?;
    let (new_start, new_count) = doc_khoang(moi)?;

    Some(Hunk {
        old_start,
        old_count,
        new_start,
        new_count,
        heading: String::from_utf8_lossy(heading).trim().to_owned(),
        lines: Vec::new(),
    })
}

/// Đọc `<start>[,<count>]`. Thiếu `,<count>` thì count là **1**.
fn doc_khoang(s: &[u8]) -> Option<(u32, u32)> {
    match memchr::memchr(b',', s) {
        Some(i) => Some((doc_so(&s[..i])?, doc_so(&s[i + 1..])?)),
        None => Some((doc_so(s)?, 1)),
    }
}

fn doc_so(s: &[u8]) -> Option<u32> {
    if s.is_empty() {
        return None;
    }
    let mut n: u32 = 0;
    for b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        n = n.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Header đầy đủ của một bản vá thật, để test không kiểm trên một đầu vào đã bị
    /// đơn giản hoá tới mức không còn giống đầu ra git.
    const HEADER: &str = "diff --git a/f.txt b/f.txt\nindex 1234567..89abcde 100644\n--- a/f.txt\n+++ b/f.txt\n";

    fn p(s: &str) -> PatchParse {
        parse_patch(s.as_bytes())
    }

    /// Ca cơ sở: một hunk, ba dòng ngữ cảnh, một dòng sửa.
    #[test]
    fn mot_hunk_ba_dong_ngu_canh() {
        let ra = p(&format!(
            "{HEADER}@@ -3,3 +3,4 @@ fn main()\n a\n-b\n+B\n+C\n c\n"
        ));

        assert_eq!(ra.hunks.len(), 1, "phải đúng một hunk");
        let h = &ra.hunks[0];
        assert_eq!((h.old_start, h.old_count), (3, 3));
        assert_eq!((h.new_start, h.new_count), (3, 4));
        assert_eq!(h.heading, "fn main()", "heading là phần sau `@@` thứ hai");

        let loai: Vec<LineKind> = h.lines.iter().map(|l| l.kind).collect();
        assert_eq!(
            loai,
            vec![
                LineKind::Context,
                LineKind::Removed,
                LineKind::Added,
                LineKind::Added,
                LineKind::Context
            ],
            "thứ tự dòng phải giữ nguyên như trong bản vá"
        );
        let noi_dung: Vec<&str> = h.lines.iter().map(|l| l.content.as_str()).collect();
        assert_eq!(noi_dung, vec!["a", "b", "B", "C", "c"]);
        assert!(!ra.truncated);
        assert_eq!(ra.skipped_lines, 0);
    }

    /// **Số dòng gán cho từng `DiffLine`** — dữ liệu mà chế độ hai cột (DIFF-02)
    /// dựng bố cục từ đó. Sai nó thì hai cột lệch hàng.
    ///
    /// Bản vá: `@@ -3,3 +3,4 @@` với ` a` / `-b` / `+B` / `+C` / ` c`.
    ///
    /// | dòng | old | new |
    /// |---|---|---|
    /// | ` a` | 3 | 3 |
    /// | `-b` | 4 | — |
    /// | `+B` | — | 4 |
    /// | `+C` | — | 5 |
    /// | ` c` | 5 | 6 |
    #[test]
    fn so_dong_dung_cho_ca_ba_loai() {
        let ra = p(&format!("{HEADER}@@ -3,3 +3,4 @@\n a\n-b\n+B\n+C\n c\n"));
        let l = &ra.hunks[0].lines;

        assert_eq!(
            (l[0].old_line, l[0].new_line),
            (Some(3), Some(3)),
            "dòng ngữ cảnh phải có CẢ HAI số dòng"
        );
        assert_eq!(
            (l[1].old_line, l[1].new_line),
            (Some(4), None),
            "dòng `removed` chỉ có số dòng CŨ"
        );
        assert_eq!(
            (l[2].old_line, l[2].new_line),
            (None, Some(4)),
            "dòng `added` chỉ có số dòng MỚI"
        );
        assert_eq!(
            (l[3].old_line, l[3].new_line),
            (None, Some(5)),
            "dòng `added` thứ hai phải tăng bộ đếm cột mới"
        );
        assert_eq!(
            (l[4].old_line, l[4].new_line),
            (Some(5), Some(6)),
            "sau một dòng xoá và hai dòng thêm, hai cột lệch nhau đúng một nấc"
        );
    }

    /// Hai hunk trong một tệp: đúng 2, không gộp, không mất.
    #[test]
    fn hai_hunk_khong_gop_khong_mat() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n a\n-b\n+B\n@@ -10,2 +10,2 @@\n x\n-y\n+Y\n"
        ));

        assert_eq!(ra.hunks.len(), 2, "phải đúng hai hunk");
        assert_eq!(ra.hunks[0].old_start, 1);
        assert_eq!(ra.hunks[1].old_start, 10);
        assert_eq!(
            ra.hunks[1].lines[0].old_line,
            Some(10),
            "hunk thứ hai phải đặt lại bộ đếm theo `old_start` của chính nó"
        );
        assert_eq!(ra.hunks[0].lines.len(), 3);
        assert_eq!(ra.hunks[1].lines.len(), 3);
    }

    /// `@@ -1 +1 @@` — git **lược bỏ** số đếm khi nó bằng 1. Phải cho `1`, không
    /// phải `0`, và không panic.
    #[test]
    fn so_dem_khuyet_la_mot_khong_phai_khong() {
        let ra = p(&format!("{HEADER}@@ -1 +1 @@\n-cu\n+moi\n"));

        assert_eq!(ra.hunks.len(), 1);
        let h = &ra.hunks[0];
        assert_eq!(h.old_count, 1, "số đếm khuyết ở phía cũ phải là 1");
        assert_eq!(h.new_count, 1, "số đếm khuyết ở phía mới phải là 1");
        assert_eq!((h.old_start, h.new_start), (1, 1));

        // Dạng hỗn hợp: một phía có số đếm, phía kia khuyết.
        let ra = p(&format!("{HEADER}@@ -1 +1,3 @@\n-cu\n+a\n+b\n+c\n"));
        assert_eq!(ra.hunks[0].old_count, 1);
        assert_eq!(ra.hunks[0].new_count, 3);
    }

    /// `\ No newline at end of file` **không** phải một `DiffLine`; nó đặt cờ lên
    /// dòng ngay trước nó.
    #[test]
    fn khong_co_dong_cuoi_dat_co_len_dong_truoc() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n a\n-b\n\\ No newline at end of file\n+B\n\\ No newline at end of file\n"
        ));

        let l = &ra.hunks[0].lines;
        assert_eq!(
            l.len(),
            3,
            "ba dòng nội dung; dòng `\\ No newline` KHÔNG được đếm là dòng thứ tư. \
             Nhận được: {l:#?}"
        );
        assert!(!l[0].no_newline_at_eof);
        assert!(l[1].no_newline_at_eof, "cờ phải lên dòng `-b`");
        assert!(l[2].no_newline_at_eof, "cờ phải lên dòng `+B`");
        assert_eq!(l[2].content, "B");
    }

    /// Dòng ngữ cảnh **rỗng** ở cả hai dạng: một dấu cách, và hoàn toàn 0 byte.
    /// Cả hai phải cho một dòng ngữ cảnh rỗng, và **không** làm hunk kết thúc sớm.
    #[test]
    fn dong_ngu_canh_rong_ca_hai_dang() {
        // Dạng chuẩn: một dấu cách.
        let ra = p(&format!("{HEADER}@@ -1,3 +1,3 @@\n a\n \n-b\n+B\n"));
        let l = &ra.hunks[0].lines;
        assert_eq!(l.len(), 4, "dấu cách đơn là một dòng ngữ cảnh rỗng");
        assert_eq!(l[1].kind, LineKind::Context);
        assert_eq!(l[1].content, "");
        assert_eq!((l[1].old_line, l[1].new_line), (Some(2), Some(2)));

        // Dạng lược bỏ dấu cách: dòng 0 byte.
        let ra = p(&format!("{HEADER}@@ -1,3 +1,3 @@\n a\n\n-b\n+B\n"));
        let l = &ra.hunks[0].lines;
        assert_eq!(
            l.len(),
            4,
            "dòng 0 byte cũng là ngữ cảnh rỗng; coi nó là hết hunk sẽ cắt mất \
             phần còn lại. Nhận được: {l:#?}"
        );
        assert_eq!(l[1].kind, LineKind::Context);
        assert_eq!(l[1].content, "");
        assert_eq!(
            l[3].content, "B",
            "dòng sau dòng rỗng phải còn nguyên — đây là điều bị mất khi hunk kết thúc sớm"
        );
    }

    /// Tệp mới thêm: header có `--- /dev/null`, mọi dòng là `added`.
    /// `/dev/null` **không** được coi là một đường dẫn tệp — và vì bộ phân tích không
    /// đọc header nên nó không có cơ hội nhầm.
    #[test]
    fn tep_moi_them_moi_dong_la_added() {
        let ra = p(
            "diff --git a/n.txt b/n.txt\nnew file mode 100644\nindex 0000000..1234567\n\
             --- /dev/null\n+++ b/n.txt\n@@ -0,0 +1,3 @@\n+a\n+b\n+c\n",
        );

        assert_eq!(ra.hunks.len(), 1);
        let h = &ra.hunks[0];
        assert_eq!((h.old_start, h.old_count), (0, 0), "phía cũ rỗng");
        assert_eq!((h.new_start, h.new_count), (1, 3));
        assert!(
            h.lines.iter().all(|l| l.kind == LineKind::Added),
            "mọi dòng của tệp mới thêm phải là `added`"
        );
        assert!(
            h.lines.iter().all(|l| l.old_line.is_none()),
            "không dòng nào có số dòng cũ"
        );
        let so_moi: Vec<Option<u32>> = h.lines.iter().map(|l| l.new_line).collect();
        assert_eq!(so_moi, vec![Some(1), Some(2), Some(3)]);

        assert_eq!(
            ra.skipped_lines, 0,
            "`--- /dev/null` nằm trong header nên không được đếm là dòng bị bỏ"
        );
    }

    /// **Byte không phải UTF-8** trong nội dung dòng: giải mã lossy, dòng vẫn ra, và
    /// dòng **ngay sau** nó không bị mất (HIST-11 ở tầng diff).
    #[test]
    fn byte_khong_utf8_khong_lam_mat_dong_sau() {
        let mut buf = Vec::new();
        buf.extend_from_slice(HEADER.as_bytes());
        buf.extend_from_slice(b"@@ -1,3 +1,3 @@\n");
        buf.extend_from_slice(b"-t\xffep\n"); // byte 0xFF: không hợp lệ trong UTF-8
        buf.extend_from_slice(b"+dong hop le ngay sau\n");
        buf.extend_from_slice(b" ngu canh cuoi\n");

        let ra = parse_patch(&buf);
        let l = &ra.hunks[0].lines;

        assert_eq!(l.len(), 3, "cả ba dòng phải còn. Nhận được: {l:#?}");
        assert_eq!(l[0].kind, LineKind::Removed);
        assert!(
            l[0].content.contains('\u{fffd}'),
            "byte xấu phải thành ký tự thay thế, không panic. Nhận: {:?}",
            l[0].content
        );
        assert_eq!(
            l[1].content, "dong hop le ngay sau",
            "dòng NGAY SAU dòng xấu phải nguyên vẹn — đây là điều mà một bộ phân tích \
             gọi `String::from_utf8().unwrap()` làm mất cả bản vá"
        );
        assert_eq!(l[2].content, "ngu canh cuoi");
    }

    /// **CRLF**: `\r` không được lọt vào `DiffLine.content`.
    ///
    /// Buffer dựng ở đây dùng `\r\n` **thật** ở mọi dòng, kể cả dòng `@@` — đó là
    /// điều kiện để đột biến "bỏ bước cắt `\r`" làm test này đỏ.
    #[test]
    fn crlf_khong_lot_vao_noi_dung() {
        let buf = b"diff --git a/c.txt b/c.txt\r\nindex 1..2 100644\r\n--- a/c.txt\r\n+++ b/c.txt\r\n\
                    @@ -1,3 +1,3 @@\r\n a\r\n-b\r\n+B\r\n";

        let ra = parse_patch(buf);
        assert_eq!(ra.hunks.len(), 1, "đầu hunk kèm `\\r` vẫn phải phân tích được");
        assert_eq!(
            (ra.hunks[0].old_start, ra.hunks[0].old_count),
            (1, 3),
            "`\\r` treo ở cuối `@@ ... @@\\r` không được làm hỏng việc đọc số"
        );

        for l in &ra.hunks[0].lines {
            assert!(
                !l.content.ends_with('\r'),
                "nội dung dòng không được kết thúc bằng `\\r` — nó là ký tự vô hình \
                 làm lệch mọi phép so nội dung. Nhận: {:?}",
                l.content
            );
        }
        let noi_dung: Vec<&str> = ra.hunks[0]
            .lines
            .iter()
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(noi_dung, vec!["a", "b", "B"]);
    }

    /// Bản vá **rỗng** — git không in gì khi hai phía giống nhau. 0 hunk, không lỗi.
    #[test]
    fn ban_va_rong_cho_khong_hunk_khong_loi() {
        let ra = parse_patch(b"");
        assert_eq!(ra.hunks.len(), 0);
        assert!(!ra.truncated);
        assert_eq!(ra.skipped_lines, 0);

        // Chỉ header, không hunk nào (xảy ra với commit chỉ đổi mode tệp).
        let ra = p("diff --git a/f.txt b/f.txt\nold mode 100644\nnew mode 100755\n");
        assert_eq!(
            ra.hunks.len(),
            0,
            "bản vá chỉ đổi mode không có hunk, và đó không phải lỗi"
        );
        assert_eq!(ra.skipped_lines, 0);
    }

    /// Chặn trên [`MAX_HUNKS`]: vượt thì cắt và bật `truncated`.
    #[test]
    fn vuot_max_hunks_thi_cat_va_bat_co() {
        let mut s = String::from(HEADER);
        for i in 0..(MAX_HUNKS + 50) {
            s.push_str(&format!("@@ -{n},1 +{n},1 @@\n-a\n+b\n", n = i + 1));
        }

        let ra = p(&s);
        assert!(ra.truncated, "vượt chặn trên phải bật cờ `truncated`");
        assert_eq!(
            ra.hunks.len(),
            MAX_HUNKS,
            "phải cắt đúng ở chặn trên, không nạp hết"
        );
    }

    /// Chặn trên [`MAX_DIFF_LINES`]: một hunk khổng lồ cũng phải bị cắt.
    /// [`MAX_HUNKS`] một mình không chặn được ca này.
    #[test]
    fn vuot_max_diff_lines_thi_cat_va_bat_co() {
        let mut s = format!("{HEADER}@@ -1,{n} +1,{n} @@\n", n = MAX_DIFF_LINES + 100);
        for i in 0..(MAX_DIFF_LINES + 100) {
            s.push_str(&format!("+dong {i}\n"));
        }

        let ra = p(&s);
        assert!(ra.truncated, "một hunk dài quá chặn trên cũng phải bị cắt");
        assert_eq!(ra.hunks.len(), 1);
        assert_eq!(
            ra.hunks[0].lines.len(),
            MAX_DIFF_LINES,
            "phải cắt đúng ở MAX_DIFF_LINES"
        );
    }

    /// Dòng trong hunk có tiền tố lạ: bỏ nó, **đếm** nó, đọc tiếp.
    /// Đếm để `get_file_diff` ghi `tracing::warn!` được (T-03-16).
    #[test]
    fn dong_tien_to_la_bi_bo_va_duoc_dem() {
        let ra = p(&format!("{HEADER}@@ -1,2 +1,2 @@\n a\n?rac\n-b\n+B\n"));

        assert_eq!(
            ra.skipped_lines, 1,
            "dòng tiền tố lạ phải được ĐẾM, không im lặng bỏ qua"
        );
        let noi_dung: Vec<&str> = ra.hunks[0]
            .lines
            .iter()
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(
            noi_dung,
            vec!["a", "b", "B"],
            "các dòng còn lại phải nguyên vẹn — một dòng hỏng không làm hỏng cả bản vá"
        );
        assert_eq!(
            ra.hunks[0].lines[1].old_line,
            Some(2),
            "dòng rác không được làm tăng bộ đếm số dòng"
        );
    }

    /// Đầu hunk không có heading thì `heading` là chuỗi rỗng, không phải rác.
    #[test]
    fn dau_hunk_khong_heading_cho_chuoi_rong() {
        let ra = p(&format!("{HEADER}@@ -1,1 +1,1 @@\n-a\n+b\n"));
        assert_eq!(ra.hunks[0].heading, "");

        let ra = p(&format!("{HEADER}@@ -1,1 +1,1 @@   \n-a\n+b\n"));
        assert_eq!(ra.hunks[0].heading, "", "khoảng trắng thừa phải được cắt");
    }
}
