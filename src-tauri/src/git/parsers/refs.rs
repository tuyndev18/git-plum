//! Phân tích đầu ra `git for-each-ref` theo byte — HIST-06, HIST-07, HIST-11.
//!
//! Một lệnh lấy hết: nhánh local, nhánh remote, tag, trạng thái thượng nguồn và cờ
//! HEAD. Cùng nguyên tắc byte với [`crate::git::parsers::log`] — tên nhánh do người
//! khác tạo ra không bảo đảm UTF-8, và một tên lạ không được làm mất cả danh sách.

use memchr::memchr_iter;

use crate::domain::{Ref, RefKind};

/// Chuỗi `--format=` của `git for-each-ref`. **Nguồn duy nhất** của định dạng này.
///
/// # Vì sao có `%(*objectname)` — trường thứ sáu mà tài liệu kế hoạch bỏ sót
///
/// Kế hoạch 02-04 đặc tả năm trường và dùng `%(objectname)` làm [`Ref::target`]. Đo
/// thật trên một repo có tag có chú thích cho thấy điều đó **sai**:
///
/// ```text
/// refs/tags/v1.0 | 5d051f68…(commit) | objecttype=commit | *objectname=
/// refs/tags/v2.0 | dbf19145…(tag)    | objecttype=tag    | *objectname=5d051f68…
/// ```
///
/// Với tag **có chú thích**, `%(objectname)` trả mã của *đối tượng tag*, không phải mã
/// commit. `git cat-file -t dbf19145…` trả `tag`. Mã đó không xuất hiện ở bất kỳ hàng
/// nào trong `git log`, nên nhãn tag sẽ **không neo được vào dòng nào** — nhãn biến
/// mất khỏi đồ thị trong im lặng, đúng thứ must-have của plan này cấm ("Ref trỏ đúng
/// mã commit để giao diện neo nhãn vào hàng commit").
///
/// `%(*objectname)` là mã đã giải tham chiếu, **rỗng** với ref không phải tag có chú
/// thích. Vì vậy [`parse_refs`] lấy `*objectname` khi nó khác rỗng, còn lại lấy
/// `objectname`. Xem test `tag_co_chu_thich_tra_ma_commit_khong_tra_ma_doi_tuong_tag`.
///
/// # 🔴 `%1f`, **không** phải `%x1f` — hai lệnh git dùng hai ngôn ngữ định dạng khác nhau
///
/// `for-each-ref` dùng ngôn ngữ `--format` của **ref**, không dùng ngôn ngữ
/// `pretty-format` của `git log`. Escape byte thô ở đây là `%<hai chữ số hex>`, tức
/// `%1f`. Chuỗi `%x1f` của `git log` **không** được diễn giải: git in ra đúng bốn ký
/// tự `%`, `x`, `1`, `f`.
///
/// Đo thật trên git 2.54.0:
///
/// ```text
/// $ git log -1 --format='%H%x1f%P' | od -c
///   0 f 9 6 f e … 3 e 037 8 6 5 1 …        ← 037 = 0x1f, ĐÚNG
///
/// $ git for-each-ref --format='%(refname)%x1f%(objectname)' | od -c
///   r e f s / h e a d s / m a i n % x 1 f 0 f 9 6 …  ← văn bản, SAI
///
/// $ git for-each-ref --format='%(refname)%1f%(objectname)' | od -c
///   r e f s / h e a d s / m a i n 037 0 f 9 6 …      ← 037, ĐÚNG
/// ```
///
/// `CONTEXT.md`, kế hoạch 02-04 và `docs/01-research-competitors.md` mục 5.2 đều ghi
/// `%x1f` cho lệnh này. Cả ba **sai**. Hỏng kiểu này rất khó thấy: lệnh vẫn thoát 0,
/// vẫn in ra dữ liệu trông hợp lý, chỉ là không có dấu phân tách nào — nên bộ phân
/// tích thấy mỗi dòng có đúng một trường, bỏ hết, và thanh bên **rỗng trong im lặng**.
/// Test đơn vị dựng buffer bằng tay không bao giờ bắt được: chúng tự chèn `0x1f`. Chỉ
/// test tích hợp chạy git thật mới thấy — xem `tests/refs_fixtures.rs`.
///
/// Thứ tự: `%(refname) %(objectname) %(upstream) %(upstream:track) %(HEAD) %(*objectname)`,
/// phân tách bằng `\x1f`, mỗi ref một dòng kết thúc bằng `\n`.
///
/// `%(*objectname)` đặt **cuối** có chủ ý: thêm trường vào cuối thì một đầu ra cũ
/// (năm trường) vẫn đọc được phần đầu, còn chèn giữa sẽ làm lệch mọi trường sau nó.
pub const REFS_FORMAT: &str = "--format=%(refname)%1f%(objectname)%1f%(upstream)%1f%(upstream:track)%1f%(HEAD)%1f%(*objectname)";

/// Tham số của lệnh liệt kê ref. Gom thành hằng cùng lý do như `LOG_ARGS`.
pub const REFS_ARGS: &[&str] = &["for-each-ref"];

/// Byte phân tách trường (Unit Separator, `%x1f`).
const UNIT_SEP: u8 = 0x1f;

/// Số trường [`REFS_FORMAT`] sinh ra. Đổi định dạng thì đổi cả số này.
const FIELD_COUNT: usize = 6;

/// Giải mã một trường hiển thị từ byte sang `String`.
///
/// Không đánh dấu cờ lossy như `parse_log`: [`Ref`] không có trường
/// `has_invalid_utf8`, và một tên nhánh hỏng mã hoá vẫn phải hiện ra để người dùng
/// thấy nó tồn tại. **Không bao giờ** `from_utf8().unwrap()` — panic trên repo thật.
fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Phân loại ref theo tiền tố và cắt tên rút gọn.
///
/// `refs/remotes/origin/main` cho `short_name == "origin/main"` chứ không phải `main`:
/// HIST-06 đòi phân biệt local với remote, và hai nhánh cùng tên ngắn là chuyện thường.
fn classify(full_name: &str) -> (RefKind, String) {
    for (prefix, kind) in [
        ("refs/heads/", RefKind::LocalBranch),
        ("refs/remotes/", RefKind::RemoteBranch),
        ("refs/tags/", RefKind::Tag),
    ] {
        if let Some(rest) = full_name.strip_prefix(prefix) {
            return (kind, rest.to_owned());
        }
    }
    // `refs/stash`, `refs/notes/*`, `refs/bisect/*`, và mọi thứ chưa nghĩ tới. Giữ
    // nguyên tên đầy đủ làm tên hiển thị — cắt bừa một tiền tố không biết trước sẽ ra
    // tên vô nghĩa, và một ref lạ hiện dưới nhóm "khác" tốt hơn là biến mất.
    (RefKind::Other, full_name.to_owned())
}

/// Đọc một số thập phân không dấu ngay sau `nhan` trong `track`.
///
/// Không dùng regex — dự án không có crate `regex` và việc này không cần tới nó.
///
/// `track` có dạng `[ahead 3, behind 1]`, `[ahead 2]`, `[behind 5]`, `[gone]`, hoặc
/// rỗng. Dấu ngoặc vuông là phần của định dạng git, nhưng hàm này **không** dựa vào
/// chúng: nó chỉ tìm nhãn rồi đọc chữ số. Nhờ vậy một phiên bản git đổi dấu ngoặc
/// vẫn đọc đúng số.
///
/// `LC_ALL=C` đã ghim ở `exec.rs`, nên nhãn luôn là tiếng Anh. Thiếu việc ghim đó thì
/// hàm này sai trên máy đặt ngôn ngữ khác — một lỗi chỉ lộ trên máy người dùng.
fn parse_track_count(track: &str, nhan: &str) -> u32 {
    let Some(rest) = track.find(nhan).map(|i| &track[i + nhan.len()..]) else {
        return 0;
    };
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    // `saturating` thay vì `unwrap_or(0)` trên chuỗi dài: một giá trị vài trăm chữ số
    // cho `u32::MAX` chứ không cho 0. Số đếm quá lớn vẫn là "rất nhiều", không phải
    // "không có".
    digits
        .parse::<u32>()
        .unwrap_or(if digits.is_empty() { 0 } else { u32::MAX })
}

/// Phân tích đầu ra `for-each-ref` đã định dạng bằng [`REFS_FORMAT`].
///
/// Không bao giờ panic và không trả `Result`, cùng lý do như
/// [`crate::git::parsers::log::parse_log`]: một ref méo không được xoá trắng thanh bên.
/// Dòng thiếu trường bị bỏ qua.
pub fn parse_refs(stdout: &[u8]) -> Vec<Ref> {
    let mut out = Vec::new();

    for line in stdout.split(|&b| b == b'\n') {
        // Bỏ `\r` cuối dòng: trên Windows git có thể kết thúc dòng bằng `\r\n`, và một
        // `\r` sót lại trong trường cuối làm mọi phép so mã commit thất bại trong im
        // lặng (cùng hạng lỗi với `\n` đầu bản ghi ở parse_log).
        let line = match line.split_last() {
            Some((b'\r', head)) => head,
            _ => line,
        };
        if line.is_empty() {
            continue;
        }
        if let Some(r) = parse_line(line) {
            out.push(r);
        }
    }

    out
}

/// Phân tích **một** dòng. `None` khi dòng không đủ trường.
fn parse_line(line: &[u8]) -> Option<Ref> {
    let mut fields: Vec<&[u8]> = Vec::with_capacity(FIELD_COUNT);
    let mut field_start = 0usize;
    for sep in memchr_iter(UNIT_SEP, line) {
        if fields.len() == FIELD_COUNT - 1 {
            break;
        }
        fields.push(&line[field_start..sep]);
        field_start = sep + 1;
    }
    fields.push(&line[field_start..]);

    if fields.len() < FIELD_COUNT {
        return None;
    }

    let full_name = lossy(fields[0]);
    if full_name.is_empty() {
        return None;
    }
    let (kind, short_name) = classify(&full_name);

    let upstream = lossy(fields[2]);
    let track = lossy(fields[3]);

    // `%(HEAD)` trả `*` cho ref đang checkout và **một dấu cách** cho mọi ref khác —
    // không phải chuỗi rỗng. Đo thật trên repo mẫu `wide`. So `== "*"` sau khi trim là
    // đúng cho cả hai; so `!= ""` thì **mọi** ref đều thành HEAD.
    let is_head = fields[4].contains(&b'*');

    // Tag có chú thích: `%(objectname)` là mã đối tượng tag, `%(*objectname)` là mã
    // commit. Xem doc của `REFS_FORMAT`.
    let deref = lossy(fields[5]);
    let target = if deref.is_empty() {
        lossy(fields[1])
    } else {
        deref
    };

    Some(Ref {
        full_name,
        short_name,
        kind,
        target,
        // `[gone]` giữ **nguyên tên** thượng nguồn: giao diện cần nói "origin/main đã
        // mất" chứ không phải "không có thượng nguồn". Hai trạng thái đó khác nhau.
        upstream: if upstream.is_empty() {
            None
        } else {
            Some(upstream)
        },
        ahead: parse_track_count(&track, "ahead "),
        behind: parse_track_count(&track, "behind "),
        is_head,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RefKind;

    /// Dựng một dòng đầu ra `for-each-ref` từ sáu trường, dạng byte.
    ///
    /// Nhận `&[u8]` cho từng trường chứ không `&str`: test tên nhánh không UTF-8 cần
    /// chèn byte thô, và một hàm trợ giúp chỉ nhận `&str` sẽ khiến ca đó không viết được.
    fn dong(fields: [&[u8]; 6]) -> Vec<u8> {
        let mut out = Vec::new();
        for (i, f) in fields.iter().enumerate() {
            if i > 0 {
                out.push(0x1f);
            }
            out.extend_from_slice(f);
        }
        out.push(b'\n');
        out
    }

    /// Dòng nhánh local đơn giản: chỉ đặt tên và mã, phần còn lại rỗng.
    fn dong_don_gian(refname: &str, objectname: &str) -> Vec<u8> {
        dong([
            refname.as_bytes(),
            objectname.as_bytes(),
            b"",
            b"",
            b" ",
            b"",
        ])
    }

    #[test]
    fn refs_heads_cho_nhanh_local() {
        let out = parse_refs(&dong_don_gian("refs/heads/main", &"a".repeat(40)));
        assert_eq!(out.len(), 1, "phải phân tích được đúng một ref");
        assert_eq!(out[0].kind, RefKind::LocalBranch);
        assert_eq!(out[0].short_name, "main");
        assert_eq!(out[0].full_name, "refs/heads/main");
    }

    /// `short_name` của nhánh remote phải giữ tên remote: HIST-06 đòi phân biệt local
    /// với remote, và `origin/main` với `main` là hai nhánh khác nhau cùng tên ngắn.
    #[test]
    fn refs_remotes_cho_nhanh_remote_kem_ten_remote() {
        let out = parse_refs(&dong_don_gian("refs/remotes/origin/main", &"b".repeat(40)));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RefKind::RemoteBranch);
        assert_eq!(
            out[0].short_name, "origin/main",
            "tên ngắn của nhánh remote phải là origin/main, không phải main"
        );
    }

    #[test]
    fn refs_tags_cho_tag() {
        let out = parse_refs(&dong_don_gian("refs/tags/v1.0", &"c".repeat(40)));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RefKind::Tag);
        assert_eq!(out[0].short_name, "v1.0");
    }

    /// `refs/stash` và các tên lạ phải ra `Other` và **không panic**. Giữ nguyên tên
    /// đầy đủ làm tên hiển thị.
    #[test]
    fn ten_la_cho_other_va_khong_panic() {
        let mut buf = dong_don_gian("refs/stash", &"d".repeat(40));
        buf.extend(dong_don_gian("refs/notes/commits", &"e".repeat(40)));
        buf.extend(dong_don_gian("HEAD", &"f".repeat(40)));

        let out = parse_refs(&buf);
        assert_eq!(out.len(), 3, "cả ba ref lạ phải còn trong danh sách");
        for r in &out {
            assert_eq!(r.kind, RefKind::Other, "{} phải là Other", r.full_name);
            assert_eq!(
                r.short_name, r.full_name,
                "ref lạ giữ nguyên tên đầy đủ làm tên hiển thị"
            );
        }
    }

    #[test]
    fn track_ahead_va_behind_cung_luc() {
        let out = parse_refs(&dong([
            b"refs/heads/main",
            &[b'a'; 40],
            b"refs/remotes/origin/main",
            b"[ahead 3, behind 1]",
            b"*",
            b"",
        ]));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].ahead, 3, "[ahead 3, behind 1] phải cho ahead = 3");
        assert_eq!(out[0].behind, 1, "[ahead 3, behind 1] phải cho behind = 1");
    }

    #[test]
    fn track_chi_co_ahead() {
        let out = parse_refs(&dong([
            b"refs/heads/main",
            &[b'a'; 40],
            b"refs/remotes/origin/main",
            b"[ahead 2]",
            b" ",
            b"",
        ]));
        assert_eq!(out[0].ahead, 2);
        assert_eq!(out[0].behind, 0, "không có nhãn behind thì behind = 0");
    }

    #[test]
    fn track_chi_co_behind() {
        let out = parse_refs(&dong([
            b"refs/heads/main",
            &[b'a'; 40],
            b"refs/remotes/origin/main",
            b"[behind 5]",
            b" ",
            b"",
        ]));
        assert_eq!(out[0].ahead, 0, "không có nhãn ahead thì ahead = 0");
        assert_eq!(out[0].behind, 5);
    }

    /// `[gone]` — thượng nguồn đã bị xoá. `ahead`/`behind` là 0, nhưng **tên thượng
    /// nguồn vẫn giữ**: giao diện cần nói "origin/main đã mất", và điều đó khác hẳn
    /// với "nhánh này chưa có thượng nguồn".
    #[test]
    fn track_gone_giu_ten_thuong_nguon() {
        let out = parse_refs(&dong([
            b"refs/heads/main",
            &[b'a'; 40],
            b"refs/remotes/origin/main",
            b"[gone]",
            b"*",
            b"",
        ]));
        assert_eq!(out[0].ahead, 0);
        assert_eq!(out[0].behind, 0);
        assert_eq!(
            out[0].upstream.as_deref(),
            Some("refs/remotes/origin/main"),
            "[gone] phải giữ tên thượng nguồn, không xoá thành None"
        );
    }

    #[test]
    fn track_rong_cho_khong_ahead_khong_behind() {
        let out = parse_refs(&dong([
            b"refs/heads/main",
            &[b'a'; 40],
            b"refs/remotes/origin/main",
            b"",
            b" ",
            b"",
        ]));
        assert_eq!(out[0].ahead, 0);
        assert_eq!(out[0].behind, 0);
    }

    /// Không có thượng nguồn thì `upstream` là `None`, để frontend phân biệt được với
    /// "có thượng nguồn và đang bằng nhau".
    #[test]
    fn khong_co_thuong_nguon_cho_none() {
        let out = parse_refs(&dong_don_gian("refs/heads/standalone", &"a".repeat(40)));
        assert!(out[0].upstream.is_none());
    }

    /// `%(HEAD)` trả `*` cho ref đang checkout và **một dấu cách** cho các ref khác —
    /// không phải chuỗi rỗng (đo thật trên repo mẫu). Một cài đặt so `!= ""` sẽ đánh
    /// dấu **mọi** ref là HEAD.
    #[test]
    fn head_dau_sao_bat_co_dau_cach_thi_khong() {
        let mut buf = dong([b"refs/heads/main", &[b'a'; 40], b"", b"", b"*", b""]);
        buf.extend(dong([
            b"refs/heads/other",
            &[b'b'; 40],
            b"",
            b"",
            b" ",
            b"",
        ]));
        buf.extend(dong([b"refs/heads/third", &[b'c'; 40], b"", b"", b"", b""]));

        let out = parse_refs(&buf);
        assert_eq!(out.len(), 3);
        assert!(out[0].is_head, "dấu * phải cho is_head = true");
        assert!(!out[1].is_head, "một dấu cách phải cho is_head = false");
        assert!(!out[2].is_head, "trường rỗng phải cho is_head = false");
    }

    /// HIST-11 ở tầng ref: tên nhánh chứa byte không UTF-8 vẫn ra một `Ref`, không
    /// mất dòng và không panic.
    #[test]
    fn ten_nhanh_khong_utf8_van_ra_mot_ref() {
        let refname = b"refs/heads/nh\xe1nh-l\xffa".to_vec();
        let mut buf = dong([&refname, &[b'a'; 40], b"", b"", b" ", b""]);
        // Thêm một ref hợp lệ SAU ref xấu: nếu cài đặt dừng ở dòng xấu thì ref này mất.
        buf.extend(dong_don_gian("refs/heads/main", &"b".repeat(40)));

        let out = parse_refs(&buf);
        assert_eq!(
            out.len(),
            2,
            "tên không UTF-8 không được làm mất dòng nào (HIST-11)"
        );
        assert_eq!(out[0].kind, RefKind::LocalBranch);
        assert!(
            out[0].short_name.contains('\u{fffd}'),
            "byte xấu phải đi qua đường lossy, nhận được: {:?}",
            out[0].short_name
        );
        assert_eq!(out[1].short_name, "main", "ref sau ref xấu phải còn nguyên");
    }

    /// Dòng thiếu trường bị bỏ qua, các dòng khác giữ nguyên — cùng nguyên tắc
    /// `skipped_records` của `parse_log`.
    #[test]
    fn dong_thieu_truong_bi_bo_qua_khong_panic() {
        let mut buf = b"refs/heads/hong\x1fabc\n".to_vec();
        buf.extend(dong_don_gian("refs/heads/main", &"a".repeat(40)));

        let out = parse_refs(&buf);
        assert_eq!(out.len(), 1, "chỉ dòng đủ trường được giữ");
        assert_eq!(out[0].short_name, "main");
    }

    #[test]
    fn buffer_rong_cho_danh_sach_rong() {
        assert!(parse_refs(b"").is_empty());
        assert!(parse_refs(b"\n\n\n").is_empty());
    }

    /// **Ca mà kế hoạch bỏ sót.** Với tag có chú thích, `%(objectname)` là mã của đối
    /// tượng tag, không phải mã commit. Nhãn tag trỏ vào mã đó sẽ không neo được vào
    /// hàng nào trong `git log` và biến mất khỏi đồ thị trong im lặng.
    #[test]
    fn tag_co_chu_thich_tra_ma_commit_khong_tra_ma_doi_tuong_tag() {
        let ma_doi_tuong_tag = "d".repeat(40);
        let ma_commit = "5".repeat(40);

        let out = parse_refs(&dong([
            b"refs/tags/v2.0",
            ma_doi_tuong_tag.as_bytes(),
            b"",
            b"",
            b" ",
            ma_commit.as_bytes(),
        ]));

        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].target, ma_commit,
            "tag có chú thích phải trả mã COMMIT đã giải tham chiếu, không trả mã đối tượng tag"
        );
    }

    /// Tag nhẹ và nhánh không có `*objectname` (trường rỗng) thì dùng `objectname`.
    #[test]
    fn tag_nhe_va_nhanh_dung_objectname() {
        let ma = "7".repeat(40);
        let out = parse_refs(&dong([
            b"refs/tags/v1.0",
            ma.as_bytes(),
            b"",
            b"",
            b" ",
            b"",
        ]));
        assert_eq!(out[0].target, ma);
    }

    /// `REFS_FORMAT` và `FIELD_COUNT` là hợp đồng hai chiều: sửa một bên quên bên kia
    /// làm **mọi** dòng lệch một nấc mà không lỗi nào được báo. Cùng lý do như test
    /// `log_format_co_dung_so_truong` của `parse_log`.
    #[test]
    fn refs_format_co_dung_so_truong() {
        let so_dau_phan_tach = REFS_FORMAT.matches("%1f").count();
        assert_eq!(
            so_dau_phan_tach,
            FIELD_COUNT - 1,
            "REFS_FORMAT có {} dấu %1f nhưng FIELD_COUNT = {}",
            so_dau_phan_tach,
            FIELD_COUNT
        );
    }

    /// `%(*objectname)` phải có trong định dạng, nếu không tag có chú thích trỏ sai mã.
    #[test]
    fn refs_format_co_objectname_da_giai_tham_chieu() {
        assert!(
            REFS_FORMAT.contains("%(*objectname)"),
            "thiếu %(*objectname) thì nhãn tag có chú thích không neo được vào hàng commit"
        );
    }

    /// Dòng kết thúc bằng `\r\n` (Windows) không để `\r` lọt vào trường cuối. Trường
    /// cuối là `*objectname`, và một `\r` ở đó làm mã commit dài 41 ký tự.
    #[test]
    fn ket_thuc_dong_crlf_khong_de_lot_ky_tu_cr() {
        let ma = "8".repeat(40);
        let mut buf = Vec::new();
        buf.extend_from_slice(b"refs/tags/v2.0\x1f");
        buf.extend_from_slice(&[b'd'; 40]);
        buf.extend_from_slice(b"\x1f\x1f\x1f \x1f");
        buf.extend_from_slice(ma.as_bytes());
        buf.extend_from_slice(b"\r\n");

        let out = parse_refs(&buf);
        assert_eq!(out.len(), 1);
        assert_eq!(
            out[0].target.len(),
            40,
            "mã commit phải đúng 40 ký tự, nhận được {:?}",
            out[0].target
        );
        assert_eq!(out[0].target, ma);
    }
}
