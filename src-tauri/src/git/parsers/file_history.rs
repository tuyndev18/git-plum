//! Phân tích `git log --follow --name-status -z` cho **một** tệp — DIFF-05.
//!
//! # 🔴 Dấu phân tách là `\0\n`, KHÔNG phải `\0` trơn
//!
//! Đây là chỗ duy nhất trong tệp này mà một bộ phân tích "đúng theo trực giác" sẽ
//! hỏng **im lặng**. Với `-z`, `git log` kết thúc phần `--format=` bằng `\0`, rồi in
//! một `\n` **trước** khối `--name-status`. Đo thật trên git 2.54.0.windows.1 bằng
//! `od -c` (2026-09-22, repo git-plum):
//!
//! ```text
//! 0000140     s   k   i   p   p   e   d   ,       p   a   t   h       A
//! 0000160     c   h   o   s   e   n       w   i   t   h   o   u   t
//! 0000200     e   v   i   d   e   n   c   e  \0  \n   M  \0   d   o   c   s
//!                                            ^^^^^^^^
//!                                            NUL rồi NEWLINE
//! ```
//!
//! Một bộ phân tích chỉ tách theo `\0` đọc trường trạng thái thành `"\nM"`, nên phép
//! so `status == "M"` **thất bại không một tiếng nào** và danh sách phiên bản về rỗng.
//! Không có lỗi, không có cảnh báo — đúng lớp lỗi `%x1f` của 02-04, nơi 19 test đơn vị
//! xanh trên buffer tự dựng trong khi git thật trả về thứ khác.
//!
//! Vì vậy [`cat_trang`] cắt `\n` và `\r` ở **hai đầu** mỗi trường, và có test dùng
//! buffer mang `\0\n` **thật** (không phải buffer tự dựng chỉ có `\0`).
//!
//! `\r` cũng bị cắt vì Windows: `git` có thể trả `\r\n` và một SHA mang `\r` ở cuối
//! sẽ trượt mọi phép so sánh (CLAUDE.md, mục "Windows runtime gotchas").
//!
//! # `%x1f` dùng được với `git log`
//!
//! Khác `for-each-ref` (vốn cần `%1f` — xem 02-04-SUMMARY), `git log` diễn giải
//! `%x1f` thành byte 0x1f thật. Đã thấy `037` trong `od -c` ở trên. Nên
//! [`FILE_HISTORY_FORMAT`] sao đúng khuôn [`super::log::LOG_FORMAT`].

use crate::domain::FileVersion;

/// Chuỗi `--format=` của lệnh lịch sử tệp. **Nguồn duy nhất** của định dạng này.
///
/// Thứ tự: `%H %an %at %s`, phân tách bằng `\x1f`. Đổi hằng này thì phải đổi
/// [`HEADER_FIELD_COUNT`] và vòng lặp của [`parse_file_history`] cùng lúc.
///
/// Không lấy `%P` (cha) như `LOG_FORMAT`: danh sách phiên bản của một tệp không vẽ
/// đồ thị, nên lane và cạnh không liên quan. Không lấy `%b` (thân): một hàng trong
/// danh sách 200 phiên bản chỉ hiện tiêu đề, và kéo toàn bộ thân commit qua IPC cho
/// 200 bản ghi là chi phí không ai dùng.
pub const FILE_HISTORY_FORMAT: &str = "--format=%H%x1f%an%x1f%at%x1f%s";

/// Byte phân tách trường (Unit Separator, `%x1f`).
const UNIT_SEP: u8 = 0x1f;

/// Số trường [`FILE_HISTORY_FORMAT`] sinh ra.
const HEADER_FIELD_COUNT: usize = 4;

/// Kết quả phân tích một lần chạy lệnh lịch sử tệp.
///
/// # Vì sao không phải `Result<Vec<FileVersion>, GitError>`
///
/// Cùng lập luận với [`super::log::LogParseResult`] và cùng requirement (HIST-11, ở
/// đây áp cho tầng lịch sử tệp): một bản ghi méo — do người khác tạo, nhiều năm
/// trước, trong một repo ta không kiểm soát — không được xoá trắng cả danh sách. Bỏ
/// nó, **đếm** nó, trả phần còn lại.
///
/// Đếm chứ không im lặng: im lặng thì lỗi được báo cáo dưới dạng "thiếu phiên bản"
/// nhiều tháng sau (T-03-38). `get_file_history` ghi `tracing::warn!` khi khác 0.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FileHistoryParse {
    /// Các phiên bản, **giữ nguyên thứ tự git in ra** (mới nhất trước).
    pub versions: Vec<FileVersion>,
    /// Số bản ghi bị bỏ vì méo (thiếu trường, SHA không phải 40 hex).
    pub skipped_records: usize,
    /// `true` khi đã chạm chặn trên `max` và còn bản ghi chưa đọc.
    pub truncated: bool,
}

/// Cắt `\n` và `\r` ở hai đầu một trường thô.
///
/// 🔴 Đây là bước mà cả bộ phân tích đứng hay đổ — xem doc comment đầu module. Trả
/// `&[u8]` chứ không `String` để chỗ gọi tự quyết định có giải mã hay không.
fn cat_trang(truong: &[u8]) -> &[u8] {
    let la_trang = |b: u8| b == b'\n' || b == b'\r';
    let mut dau = 0;
    let mut cuoi = truong.len();
    while dau < cuoi && la_trang(truong[dau]) {
        dau += 1;
    }
    while cuoi > dau && la_trang(truong[cuoi - 1]) {
        cuoi -= 1;
    }
    &truong[dau..cuoi]
}

/// Một chuỗi 40 ký tự hex — dấu hiệu bắt đầu một bản ghi header.
///
/// Dùng để phân biệt "trường này mở một commit mới" với "trường này là trạng thái
/// hoặc đường dẫn của commit đang đọc". Không có phép kiểm này thì một tệp tên đúng
/// 40 ký tự hex… vẫn được đọc đúng, vì ta chỉ hỏ�i ở vị trí mà một header được phép
/// bắt đầu. Nhưng nó cũng là cổng chặn bản ghi méo: SHA không 40 hex → bỏ, đếm.
fn la_sha(truong: &[u8]) -> bool {
    truong.len() == 40 && truong.iter().all(|b| b.is_ascii_hexdigit())
}

/// Phân tích đầu ra `git log --follow <FILE_HISTORY_FORMAT> --name-status -z`.
///
/// `max` là chặn trên số phiên bản trả về; chạm chặn thì `truncated = true`.
///
/// # Lọc theo đường dẫn: KHÔNG làm ở đây, và đó là một phát hiện đo được
///
/// `<behavior>` của plan 03-05 nói *"`--follow` vẫn in mọi tệp của commit đó"* và đòi
/// một bước lọc theo path đang theo. **Đo thật thì điều đó sai**: `git log` với một
/// pathspec đã lọc `--name-status` xuống đúng path đó, kể cả khi commit sửa nhiều tệp.
/// Bằng chứng ở test `mot_commit_dung_nhieu_tep_git_da_loc_san` và ở
/// `03-05-SUMMARY.md`. Thêm một phép lọc ở đây sẽ là mã không bao giờ chạy — và tệ
/// hơn, nó cần biết "path đang theo" là gì, vốn **đổi** ở mỗi lần đổi tên, nên nó sẽ
/// lọc mất chính các phiên bản trước lần đổi tên mà `--follow` vừa mua được.
///
/// Điều thay thế nó: mỗi commit giữ **bản ghi tệp đầu tiên**. Commit không có bản ghi
/// tệp nào (merge — xem dưới) bị **bỏ** khỏi danh sách.
pub fn parse_file_history(stdout: &[u8], max: usize) -> FileHistoryParse {
    let mut ket_qua = FileHistoryParse::default();

    // Tách theo **cả hai** byte phân tách, và đó là điểm cốt yếu của hàm này:
    //
    // * `\x1f` (`UNIT_SEP`) ngăn các trường **trong** phần `--format=` — bốn trường
    //   của [`FILE_HISTORY_FORMAT`].
    // * `\0` ngăn các trường của khối `--name-status -z`, và cũng kết thúc phần
    //   `--format=`.
    //
    // Tách chỉ theo một trong hai là hỏng im lặng: chỉ `\0` thì cả header thành một
    // trường duy nhất (`sha\x1ftác giả\x1f…`) và không khớp `la_sha`, nên **mọi** bản
    // ghi bị tính là méo và danh sách về rỗng. Đo được: phép cắt sai kiểu này cho
    // `skipped_records = 9` trên một buffer ba commit hoàn toàn hợp lệ.
    //
    // Rồi [`cat_trang`] cắt `\n`/`\r` — đó là bước xử lý dấu phân tách `\0\n` thật.
    let truong: Vec<&[u8]> = stdout
        .split(|&b| b == 0 || b == UNIT_SEP)
        .map(cat_trang)
        .filter(|t| !t.is_empty())
        .collect();

    let mut i = 0;
    while i < truong.len() {
        if ket_qua.versions.len() >= max {
            ket_qua.truncated = true;
            break;
        }

        // Một header là `HEADER_FIELD_COUNT` trường liên tiếp mở đầu bằng 40 hex.
        if !la_sha(truong[i]) {
            // Không phải chỗ một header được phép bắt đầu: bản ghi méo. Bỏ **một**
            // trường rồi thử lại — bỏ cả cụm sẽ ăn mất bản ghi kế tiếp còn tốt.
            ket_qua.skipped_records += 1;
            i += 1;
            continue;
        }
        if i + HEADER_FIELD_COUNT > truong.len() {
            // Header cụt ở cuối buffer. Bỏ, đừng đoán.
            ket_qua.skipped_records += 1;
            break;
        }

        let commit_id = String::from_utf8_lossy(truong[i]).into_owned();
        let author_name = String::from_utf8_lossy(truong[i + 1]).into_owned();
        // Timestamp không đọc được → `0`, **không** làm rơi bản ghi. Khuôn
        // `bad_timestamps` của `parse_log`: một `%at` méo là lỗi hiển thị một hàng,
        // không phải lý do mất một phiên bản khỏi lịch sử.
        let author_time = String::from_utf8_lossy(truong[i + 2])
            .parse::<i64>()
            .unwrap_or(0);
        let subject = String::from_utf8_lossy(truong[i + 3]).into_owned();
        i += HEADER_FIELD_COUNT;

        // Khối `--name-status` của commit này: mọi trường tới header kế tiếp.
        let mut status: Option<String> = None;
        let mut path: Option<String> = None;
        let mut old_path: Option<String> = None;

        while i < truong.len() && !la_sha(truong[i]) {
            let st = String::from_utf8_lossy(truong[i]).into_owned();
            i += 1;

            // `R`/`C` mang điểm tương đồng và chiếm **hai** đường dẫn
            // (`R077\0cu.txt\0moi.txt\0`). Đọc chúng như bản ghi một đường dẫn làm
            // **mọi** bản ghi sau lệch một nấc: đường dẫn mới bị đọc thành trạng
            // thái. Một lỗi làm hỏng cả danh sách, không chỉ một hàng — đây là lý do
            // test có thêm hai bản ghi đứng **sau** bản ghi `R`.
            let doi_ten = st.starts_with('R') || st.starts_with('C');

            let Some(p1) = truong.get(i).map(|t| String::from_utf8_lossy(t).into_owned()) else {
                ket_qua.skipped_records += 1;
                break;
            };
            i += 1;

            let (p, op) = if doi_ten {
                let Some(p2) = truong.get(i).map(|t| String::from_utf8_lossy(t).into_owned())
                else {
                    ket_qua.skipped_records += 1;
                    break;
                };
                i += 1;
                (p2, Some(p1))
            } else {
                (p1, None)
            };

            // Giữ bản ghi **đầu tiên** của commit. Xem doc comment về việc không lọc
            // theo path: git đã lọc sẵn, nên bản ghi đầu tiên *là* tệp đang theo.
            if status.is_none() {
                status = Some(st);
                path = Some(p);
                old_path = op;
            }
        }

        // 🔴 Commit không có bản ghi tệp nào → **bỏ**, không thêm một hàng trống.
        //
        // Ca này là **merge commit**: `git log` mặc định không in `--name-status` cho
        // merge. Đo thật (xem `03-05-SUMMARY.md`): với `--follow`, merge commit không
        // chỉ thiếu khối tệp mà **không xuất hiện trong danh sách commit** ở tất cả —
        // nên nhánh này gần như không bao giờ chạy trên đầu ra git thật. Nó vẫn phải
        // ở đây: nó là cổng chặn một hàng trống lọt vào danh sách nếu một cờ khác
        // (`-m`, `--diff-merges`) được thêm về sau.
        match (status, path) {
            (Some(s), Some(p)) => ket_qua.versions.push(FileVersion {
                commit_id,
                author_name,
                author_time,
                subject,
                status: s,
                path: p,
                old_path,
            }),
            _ => continue,
        }
    }

    // Chạm `max` **đúng lúc** hết buffer không phải bị cắt. Kiểm lại ở đây thay vì
    // tin cờ đặt trong vòng lặp: `break` ở đầu vòng chạy cả khi `i == len`.
    if ket_qua.versions.len() >= max && i < truong.len() {
        ket_qua.truncated = true;
    }

    ket_qua
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Buffer mang dấu phân tách **thật** `\0\n`, đúng như `od -c` đo được.
    ///
    /// Hàm này là chỗ tập trung sự thật đó: một test tự dựng buffer chỉ có `\0` sẽ
    /// xanh trên mã sai, nên mọi test dưới đây đi qua đây.
    fn ban_ghi(sha: &str, tac_gia: &str, thoi_gian: &str, tieu_de: &str, tep: &[&str]) -> Vec<u8> {
        let mut ra = Vec::new();
        ra.extend_from_slice(sha.as_bytes());
        ra.push(UNIT_SEP);
        ra.extend_from_slice(tac_gia.as_bytes());
        ra.push(UNIT_SEP);
        ra.extend_from_slice(thoi_gian.as_bytes());
        ra.push(UNIT_SEP);
        ra.extend_from_slice(tieu_de.as_bytes());
        // 🔴 `\0` RỒI `\n` — đây là điều `od -c` đo được, không phải phỏng đoán.
        ra.push(0);
        ra.push(b'\n');
        for (idx, t) in tep.iter().enumerate() {
            ra.extend_from_slice(t.as_bytes());
            if idx + 1 < tep.len() {
                ra.push(0);
            }
        }
        ra.push(0);
        ra
    }

    fn sha(n: u8) -> String {
        std::iter::repeat_n(char::from(b'a' + n), 40).collect()
    }

    /// Ba commit sửa cùng một tệp → đúng ba phiên bản, thứ tự git giữ nguyên.
    #[test]
    fn ba_commit_cho_ba_phien_ban_giu_thu_tu() {
        let mut buf = ban_ghi(&sha(0), "Aa", "1790000003", "moi nhat", &["M", "a.txt"]);
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000002", "giua", &["M", "a.txt"]));
        buf.extend(ban_ghi(&sha(2), "Cc", "1790000001", "dau tien", &["A", "a.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.skipped_records, 0, "không bản ghi nào được coi là méo");
        assert_eq!(r.versions.len(), 3);
        assert_eq!(r.versions[0].subject, "moi nhat", "mới nhất phải đứng đầu");
        assert_eq!(r.versions[2].subject, "dau tien");
        assert_eq!(r.versions[0].author_name, "Aa");
        assert_eq!(r.versions[0].author_time, 1790000003);
    }

    /// 🔴 **Mutation then chốt.** `status` phải là `"M"`, KHÔNG `"\nM"`.
    ///
    /// Bỏ [`cat_trang`] khỏi đường phân tích làm test này đỏ, và nó là lỗi
    /// hỏng-im-lặng duy nhất trong tệp: `status == "M"` trượt, danh sách về rỗng, và
    /// không lỗi nào được báo.
    #[test]
    fn dau_phan_tach_nul_newline_khong_lot_vao_truong_status() {
        let buf = ban_ghi(&sha(0), "Aa", "1790000000", "co that", &["M", "a.txt"]);

        // Tiền đề: buffer thật sự mang `\0\n`. Không có khẳng định này thì test có
        // thể xanh vì `ban_ghi` lặng lẽ thôi sinh `\n`.
        assert!(
            buf.windows(2).any(|w| w == [0, b'\n']),
            "buffer phải mang dấu phân tách \\0\\n thật, nếu không test này vô nghĩa"
        );

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.versions.len(), 1, "một bản ghi phải đọc được");
        assert_eq!(
            r.versions[0].status, "M",
            "status phải là \"M\"; \"\\nM\" nghĩa là dấu phân tách \\0\\n chưa được cắt"
        );
        assert_eq!(r.versions[0].path, "a.txt");
    }

    /// Đường dẫn cũng không được mang `\n` hay `\r` — cùng lý do, cạnh khác.
    #[test]
    fn cat_trang_cung_ap_cho_duong_dan_va_sha() {
        let mut buf = ban_ghi(&sha(0), "Aa", "1790000000", "x", &["M", "a.txt"]);
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000000", "y", &["M", "b.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.versions.len(), 2);
        for v in &r.versions {
            assert!(
                !v.commit_id.contains('\n') && !v.commit_id.contains('\r'),
                "SHA mang ký tự xuống dòng sẽ trượt mọi phép so: {:?}",
                v.commit_id
            );
            assert!(!v.path.contains('\n'), "đường dẫn mang \\n: {:?}", v.path);
            assert!(!v.status.contains('\n'), "status mang \\n: {:?}", v.status);
        }
    }

    /// `R100` chiếm **hai** đường dẫn, và hai bản ghi **sau** nó không được lệch.
    ///
    /// Hai bản ghi sau là cả điểm của test: đọc `R` như một đường dẫn làm đường dẫn
    /// mới bị đọc thành trạng thái của commit kế tiếp, nên lỗi lộ ra ở bản ghi *sau*
    /// chứ không ở bản ghi `R`.
    #[test]
    fn ban_ghi_r_hai_duong_dan_khong_lam_lech_hai_ban_ghi_sau() {
        let mut buf = ban_ghi(
            &sha(0),
            "Aa",
            "1790000003",
            "doi ten",
            &["R100", "cu.txt", "moi.txt"],
        );
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000002", "sua", &["M", "cu.txt"]));
        buf.extend(ban_ghi(&sha(2), "Cc", "1790000001", "them", &["A", "cu.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.versions.len(), 3, "phải đọc đúng ba phiên bản");

        assert_eq!(r.versions[0].status, "R100");
        assert_eq!(r.versions[0].old_path.as_deref(), Some("cu.txt"));
        assert_eq!(r.versions[0].path, "moi.txt");

        // Hai bản ghi sau: chỗ lệch nấc lộ ra.
        assert_eq!(r.versions[1].status, "M", "bản ghi ngay sau R bị lệch nấc");
        assert_eq!(r.versions[1].path, "cu.txt");
        assert!(r.versions[1].old_path.is_none());
        assert_eq!(r.versions[2].status, "A", "bản ghi thứ hai sau R bị lệch nấc");
        assert_eq!(r.versions[2].path, "cu.txt");
        assert_eq!(r.skipped_records, 0);
    }

    /// `C` (sao chép) cũng chiếm hai đường dẫn, cùng đường mã với `R`.
    #[test]
    fn ban_ghi_c_cung_chiem_hai_duong_dan() {
        let buf = ban_ghi(
            &sha(0),
            "Aa",
            "1790000000",
            "sao chep",
            &["C075", "goc.txt", "ban.txt"],
        );
        let r = parse_file_history(&buf, 200);
        assert_eq!(r.versions.len(), 1);
        assert_eq!(r.versions[0].old_path.as_deref(), Some("goc.txt"));
        assert_eq!(r.versions[0].path, "ban.txt");
    }

    /// Commit đụng nhiều tệp: git đã lọc sẵn theo pathspec, nên chỉ có một bản ghi.
    ///
    /// Test này ghim **hình dạng đầu ra thật** (một bản ghi cho commit đa tệp) — xem
    /// test tích hợp cùng tên trong `tests/file_history_commands.rs`, nơi cùng điều
    /// đó được kiểm trên git thật. Ở đây kiểm rằng nếu git *có* in nhiều bản ghi thì
    /// ta giữ bản ghi đầu chứ không dựng hai phiên bản cho một commit.
    #[test]
    fn mot_commit_nhieu_ban_ghi_tep_chi_cho_mot_phien_ban() {
        let buf = ban_ghi(
            &sha(0),
            "Aa",
            "1790000000",
            "dung nhieu tep",
            &["M", "a.txt", "M", "khac.txt"],
        );

        let r = parse_file_history(&buf, 200);

        assert_eq!(
            r.versions.len(),
            1,
            "một commit phải cho đúng MỘT phiên bản, không phải một hàng cho mỗi tệp"
        );
        assert_eq!(r.versions[0].path, "a.txt");
    }

    /// Merge commit: commit không có bản ghi tệp nào → **bỏ**, không hàng trống.
    #[test]
    fn commit_khong_co_ban_ghi_tep_bi_bo_khong_thanh_hang_trong() {
        let mut buf = ban_ghi(&sha(0), "Aa", "1790000002", "merge", &[]);
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000001", "sua that", &["M", "a.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(
            r.versions.len(),
            1,
            "merge không sửa tệp nào phải bị bỏ, không thêm một hàng trống"
        );
        assert_eq!(r.versions[0].subject, "sua that");
        assert!(
            r.versions.iter().all(|v| !v.path.is_empty()),
            "không phiên bản nào được có đường dẫn rỗng"
        );
    }

    /// Tên tệp không UTF-8: giải mã lossy, và bản ghi **sau** không mất (HIST-11).
    #[test]
    fn ten_tep_khong_utf8_giai_ma_lossy_va_khong_lam_mat_ban_ghi_sau() {
        let mut buf = Vec::new();
        buf.extend_from_slice(sha(0).as_bytes());
        buf.push(UNIT_SEP);
        buf.extend_from_slice("Aa".as_bytes());
        buf.push(UNIT_SEP);
        buf.extend_from_slice(b"1790000002");
        buf.push(UNIT_SEP);
        buf.extend_from_slice(b"co ten xau");
        buf.push(0);
        buf.push(b'\n');
        buf.extend_from_slice(b"M");
        buf.push(0);
        // 0xFF không bao giờ hợp lệ trong UTF-8.
        buf.extend_from_slice(&[b'x', 0xFF, b'.', b't', b'x', b't']);
        buf.push(0);
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000001", "sau do", &["M", "a.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(
            r.versions.len(),
            2,
            "bản ghi sau một tên tệp không UTF-8 KHÔNG được mất (HIST-11)"
        );
        assert!(
            r.versions[0].path.contains('\u{FFFD}'),
            "tên tệp xấu phải được giải mã lossy, không panic: {:?}",
            r.versions[0].path
        );
        assert_eq!(r.versions[1].subject, "sau do");
    }

    /// Timestamp méo → `0`, bản ghi vẫn ra (khuôn `bad_timestamps` của `parse_log`).
    #[test]
    fn timestamp_khong_doc_duoc_thanh_0_khong_lam_roi_ban_ghi() {
        let mut buf = ban_ghi(&sha(0), "Aa", "khong-phai-so", "xau", &["M", "a.txt"]);
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000001", "tot", &["M", "a.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.versions.len(), 2, "timestamp méo không được làm rơi bản ghi");
        assert_eq!(r.versions[0].author_time, 0);
        assert_eq!(r.versions[1].author_time, 1790000001);
    }

    /// Vượt `max` → cắt, `truncated = true`.
    #[test]
    fn vuot_max_thi_cat_va_dat_co_truncated() {
        let mut buf = Vec::new();
        for n in 0..5u8 {
            buf.extend(ban_ghi(&sha(n), "Aa", "1790000000", "x", &["M", "a.txt"]));
        }

        let r = parse_file_history(&buf, 3);

        assert_eq!(r.versions.len(), 3);
        assert!(r.truncated, "chạm chặn trên phải đặt truncated");
    }

    /// Đúng `max` bản ghi và hết buffer → **không** truncated. Khác ca bị cắt.
    #[test]
    fn dung_max_ban_ghi_khong_phai_bi_cat() {
        let mut buf = Vec::new();
        for n in 0..3u8 {
            buf.extend(ban_ghi(&sha(n), "Aa", "1790000000", "x", &["M", "a.txt"]));
        }

        let r = parse_file_history(&buf, 3);

        assert_eq!(r.versions.len(), 3);
        assert!(
            !r.truncated,
            "đúng max bản ghi mà hết buffer thì không có gì bị cắt"
        );
    }

    /// Đầu ra rỗng (tệp không có trong lịch sử) → 0 phiên bản, KHÔNG lỗi.
    #[test]
    fn dau_ra_rong_cho_khong_phien_ban_va_khong_la_ca_loi() {
        let r = parse_file_history(b"", 200);
        assert_eq!(r.versions.len(), 0);
        assert_eq!(r.skipped_records, 0, "rỗng không phải méo");
        assert!(!r.truncated);
    }

    /// Bản ghi méo (SHA không 40 hex) bị đếm, và bản ghi tốt sau nó vẫn ra.
    #[test]
    fn sha_khong_40_hex_bi_dem_va_khong_lam_mat_ban_ghi_sau() {
        let mut buf = ban_ghi("khong-phai-sha", "Aa", "1790000002", "xau", &["M", "a.txt"]);
        buf.extend(ban_ghi(&sha(1), "Bb", "1790000001", "tot", &["M", "a.txt"]));

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.versions.len(), 1, "bản ghi tốt sau bản ghi méo phải ra");
        assert_eq!(r.versions[0].subject, "tot");
        assert!(
            r.skipped_records > 0,
            "bản ghi méo phải được ĐẾM, không biến mất im lặng (T-03-38)"
        );
    }

    /// Header cụt ở cuối buffer: bỏ, đếm, không đoán.
    #[test]
    fn header_cut_o_cuoi_buffer_bi_bo_va_dem() {
        let mut buf = ban_ghi(&sha(0), "Aa", "1790000002", "tot", &["M", "a.txt"]);
        // Header chỉ có SHA và tác giả, thiếu hai trường.
        buf.extend_from_slice(sha(1).as_bytes());
        buf.push(UNIT_SEP);
        buf.extend_from_slice(b"Bb");
        buf.push(0);

        let r = parse_file_history(&buf, 200);

        assert_eq!(r.versions.len(), 1);
        assert_eq!(r.skipped_records, 1);
    }

    /// `FILE_HISTORY_FORMAT` và [`HEADER_FIELD_COUNT`] là hợp đồng hai chiều.
    ///
    /// Thêm một `%x1f%<gì>` mà quên sửa con số làm mọi bản ghi lệch **im lặng** —
    /// đúng cái bẫy mà `LOG_FORMAT`/`FIELD_COUNT` của 02-02 ghi lại.
    #[test]
    fn so_truong_cua_format_khop_hang_so() {
        let so = FILE_HISTORY_FORMAT.matches("%x1f").count() + 1;
        assert_eq!(
            so, HEADER_FIELD_COUNT,
            "FILE_HISTORY_FORMAT sinh {so} trường nhưng HEADER_FIELD_COUNT = {HEADER_FIELD_COUNT}"
        );
    }
}
