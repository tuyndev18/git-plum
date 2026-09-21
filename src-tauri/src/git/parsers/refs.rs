//! Phân tích đầu ra `git for-each-ref` theo byte — HIST-06, HIST-07, HIST-11.

use crate::domain::Ref;

/// Chuỗi `--format=` của `git for-each-ref`. **Nguồn duy nhất** của định dạng này.
pub const REFS_FORMAT: &str = "--format=%(refname)%x1f%(objectname)%x1f%(upstream)%x1f%(upstream:track)%x1f%(HEAD)%x1f%(*objectname)";

/// Số trường REFS_FORMAT sinh ra.
const FIELD_COUNT: usize = 6;

/// Tham số của lệnh liệt kê ref.
pub const REFS_ARGS: &[&str] = &["for-each-ref"];

/// Phân tích đầu ra `for-each-ref` đã định dạng bằng [`REFS_FORMAT`].
pub fn parse_refs(_stdout: &[u8]) -> Vec<Ref> {
    // RED: chưa cài. Các test dưới đây phải đỏ trước khi có cài đặt.
    Vec::new()
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
        buf.extend(dong([b"refs/heads/other", &[b'b'; 40], b"", b"", b" ", b""]));
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
        let so_dau_phan_tach = REFS_FORMAT.matches("%x1f").count();
        assert_eq!(
            so_dau_phan_tach,
            FIELD_COUNT - 1,
            "REFS_FORMAT có {} dấu %x1f nhưng FIELD_COUNT = {}",
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
