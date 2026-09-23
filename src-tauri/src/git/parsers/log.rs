//! Phân tích đầu ra `git log` theo byte — HIST-01, HIST-11.
//!
//! # Vì sao tệp này là chỗ HIST-11 đứng hay đổ
//!
//! Tiêu chí thành công số 6 của phase nói: mở repository có tên tệp và thông điệp
//! commit chứa ký tự không phải UTF-8 thì trang lịch sử vẫn hiện đầy đủ, **không dòng
//! nào bị mất**. Một `String::from_utf8(..).unwrap()` ở đây làm đổ cả trang; một
//! `?` ở giữa vòng lặp làm mất mọi commit sau commit xấu. Cả hai đều vi phạm tiêu chí.
//! Vì vậy [`parse_log`] **không** trả `Result` — xem ghi chú ở [`LogParseResult`].
//!
//! # Hiệu năng
//!
//! Đo trên repo 100k commit của `scripts/fixtures/make-perf-repo.sh`: riêng việc git
//! sinh và ống 8,4 MB dữ liệu đã tốn **847ms**, trong khi tiêu chí số 1 đòi toàn bộ
//! dưới một giây. Phần phân tích ở đây do đó phải rất mỏng: tách buffer bằng `memchr`
//! (SIMD) chứ không bằng `split`, và không sao chép byte nào ngoài những `String` thật
//! sự phải trả về.

use memchr::memchr_iter;

use crate::domain::Commit;

/// Chuỗi `--format=` của `git log`. **Nguồn duy nhất** của định dạng này.
///
/// # Đổi hằng này thì phải đổi [`parse_log`] cùng lúc
///
/// Hai thứ là một hợp đồng hai chiều: số trường, thứ tự trường và ký tự phân tách ở
/// đây quyết định vòng lặp bên dưới đọc gì. Thêm một `%x1f%<gì đó>` mà quên sửa
/// [`FIELD_COUNT`] sẽ khiến **mọi** bản ghi bị tính là thừa trường và phần thừa bị nối
/// vào `body` — bộ phân tích không báo lỗi, chỉ âm thầm trả dữ liệu sai chỗ.
///
/// Plan 02-04 dựng lệnh git phải dùng hằng này, không chép lại chuỗi.
///
/// Thứ tự: `%H %P %an %ae %at %cn %ce %ct %s %b`, phân tách bằng `\x1f`, kết thúc bản
/// ghi bằng `\x1e`. Hai ký tự điều khiển này gần như không bao giờ xuất hiện trong nội
/// dung commit — nhưng "gần như" không phải "không bao giờ", nên xem T-02-05 và test
/// `byte_1f_trong_body_khong_tach_thanh_hai_ban_ghi`.
pub const LOG_FORMAT: &str =
    "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%b%x1e";

/// Tham số bắt buộc của mọi lệnh `git log` đọc lịch sử trong ứng dụng này.
///
/// # `--date-order` không phải tuỳ chọn
///
/// Thuật toán gán lane (plan 02-03) giả định **cha luôn xuất hiện sau con**. Thứ tự
/// mặc định của `git log` là theo thời gian commit, và thời gian trong repo thật không
/// đáng tin: rebase, cherry-pick và đồng hồ máy lệch đều sinh ra cha *mới hơn* con.
/// Thiếu cờ này thì lane rò và đường nối trỏ ngược — một lỗi chỉ lộ ra trên repo thật,
/// không lộ trên lịch sử tuyến tính.
///
/// # Vì sao `--date-order` chứ không `--topo-order` (đổi 2026-09-23)
///
/// Cả hai đều bảo đảm cha sau con. Khác nhau ở chỗ `--topo-order` còn cố **không
/// xen kẽ** các dòng lịch sử: nó in hết chuỗi cha-thứ-nhất rồi mới in nhánh được
/// merge vào. Với mẫu "merge nhánh feature vào main nhiều lần liên tiếp" (repo thật
/// `quanly-truong-phong-so`), 20 merge in liền nhau, mỗi merge chờ một cha-thứ-hai
/// khác nhau → mở 20 lane bậc thang, commit nhánh dồn xuống cuối. `--date-order` xen
/// commit nhánh ngay dưới merge của nó theo thời gian commit — đúng cách GitKraken
/// vẽ, và lane được thu hồi ngay.
///
/// Gom thành hằng để không call site nào quên được. Dùng kèm [`LOG_FORMAT`]:
///
/// ```no_run
/// # use git_plum_lib::git::parsers::log::{LOG_ARGS, LOG_FORMAT};
/// # use git_plum_lib::git::GitCommand;
/// let cmd = GitCommand::new(".").args(LOG_ARGS).arg(LOG_FORMAT);
/// ```
pub const LOG_ARGS: &[&str] = &["log", "--all", "--date-order"];

/// Byte phân tách trường (Unit Separator, `%x1f`).
const UNIT_SEP: u8 = 0x1f;

/// Byte phân tách bản ghi (Record Separator, `%x1e`).
const RECORD_SEP: u8 = 0x1e;

/// Số trường [`LOG_FORMAT`] sinh ra. Đổi định dạng thì đổi cả số này.
const FIELD_COUNT: usize = 10;

/// Kết quả phân tích một trang `git log`.
///
/// # Vì sao không phải `Result<Vec<Commit>, GitError>`
///
/// HIST-11 đòi **không dòng nào bị mất**. Nếu một bản ghi méo làm cả hàm trả `Err`,
/// thì một commit hỏng duy nhất — do người khác tạo ra, từ nhiều năm trước, trong một
/// repo ta không kiểm soát — sẽ xoá trắng cả trang lịch sử. Đó là một dạng từ chối
/// dịch vụ (T-02-04 trong threat model).
///
/// Nhưng bản ghi méo cũng **không được biến mất trong im lặng**: im lặng thì không ai
/// biết bộ phân tích đang sai, và lỗi sẽ được báo cáo dưới dạng "thiếu commit" nhiều
/// tháng sau. Vì vậy số bản ghi bỏ qua đi kèm kết quả, và plan 02-04 ghi nó vào
/// `tracing::warn!` khi khác 0.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LogParseResult {
    /// Các commit đã phân tích, **giữ nguyên thứ tự git in ra**.
    ///
    /// Bộ phân tích không sắp xếp lại gì cả — thứ tự topo do `--topo-order` bảo đảm,
    /// và sắp xếp lại ở đây sẽ phá đúng cái bất biến mà thuật toán lane dựa vào.
    pub commits: Vec<Commit>,

    /// Số bản ghi bị bỏ vì không đủ [`FIELD_COUNT`] trường.
    ///
    /// Khác 0 gần như luôn nghĩa là [`LOG_FORMAT`] và [`parse_log`] đã lệch nhau, chứ
    /// không phải repo có vấn đề. Đáng ghi cảnh báo.
    pub skipped_records: usize,

    /// Số trường thời gian không phân tích được thành số (đã thay bằng `0`).
    ///
    /// Commit vẫn nằm trong `commits` — một timestamp lạ không phải lý do để giấu
    /// commit khỏi đồ thị.
    pub bad_timestamps: usize,
}

/// Giải mã một trường **hiển thị** từ byte sang `String`, đánh dấu nếu phải lossy.
///
/// Đặt `*dirty = true` khi byte không phải UTF-8 hợp lệ. Cờ đó chảy vào
/// [`Commit::has_invalid_utf8`] để test HIST-11 khẳng định được rằng dòng này *đã* đi
/// qua đường lossy mà vẫn còn trong danh sách.
///
/// **Tuyệt đối không** dùng `String::from_utf8(..).unwrap()` hay `.expect()` ở đây:
/// tên tệp trên Linux và thông điệp commit mã hoá cũ không bảo đảm UTF-8, và repo mẫu
/// `non-utf8` chứng minh cả bốn commit của nó sẽ làm `from_utf8` thất bại.
fn lossy(bytes: &[u8], dirty: &mut bool) -> String {
    match std::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => {
            *dirty = true;
            String::from_utf8_lossy(bytes).into_owned()
        }
    }
}

/// Đọc một số nguyên có dấu từ byte ASCII, `None` nếu không phải số.
///
/// Không đi qua `String`: đây là đường nóng chạy 200k lần trên repo 100k commit.
fn parse_i64(bytes: &[u8]) -> Option<i64> {
    let bytes = trim_ascii_whitespace(bytes);
    if bytes.is_empty() {
        return None;
    }

    let (negative, digits) = match bytes[0] {
        b'-' => (true, &bytes[1..]),
        b'+' => (false, &bytes[1..]),
        _ => (false, bytes),
    };
    if digits.is_empty() {
        return None;
    }

    let mut value: i64 = 0;
    for &b in digits {
        let d = (b as char).to_digit(10)?;
        // checked_* để timestamp rác dài vài trăm chữ số không tràn trong im lặng.
        value = value.checked_mul(10)?.checked_add(d as i64)?;
    }
    Some(if negative { -value } else { value })
}

/// Bỏ byte khoảng trắng ASCII ở hai đầu.
///
/// `[u8]::trim_ascii` ổn định từ Rust 1.80; dự án ghim `rust-version = 1.82` nên dùng
/// được, nhưng viết tay ở đây để rõ rằng chỉ **khoảng trắng ASCII** bị bỏ — byte cao
/// (≥ 0x80) của tên tác giả mã hoá Latin-1 phải được giữ nguyên để đường lossy xử lý.
fn trim_ascii_whitespace(bytes: &[u8]) -> &[u8] {
    let mut start = 0;
    let mut end = bytes.len();
    while start < end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &bytes[start..end]
}

/// Phân tích một trang đầu ra `git log` đã định dạng bằng [`LOG_FORMAT`].
///
/// Không bao giờ panic và không bao giờ trả lỗi — xem [`LogParseResult`]. Buffer rỗng
/// cho kết quả rỗng; bản ghi cuối bị cắt giữa chừng (đọc theo luồng, trang chưa hết)
/// bị bỏ qua chứ không làm hỏng các bản ghi trước.
pub fn parse_log(stdout: &[u8]) -> LogParseResult {
    let mut result = LogParseResult::default();

    let mut start = 0usize;
    for sep in memchr_iter(RECORD_SEP, stdout) {
        let record = &stdout[start..sep];
        start = sep + 1;
        parse_record(record, &mut result);
    }

    // Phần sau dấu `\x1e` cuối cùng là một bản ghi **chưa kết thúc**. git luôn đóng bản
    // ghi bằng `\x1e`, nên phần dư hợp lệ duy nhất là khoảng trắng (git chèn `\n` sau
    // mỗi bản ghi). Bất kỳ thứ gì khác là dữ liệu bị cắt giữa chừng: bỏ, đừng đoán.
    // Không đếm vào `skipped_records` — đây là chuyện bình thường khi đọc theo luồng,
    // không phải dấu hiệu định dạng sai.

    result
}

/// Phân tích **một** bản ghi (phần giữa hai dấu `\x1e`) và nối vào `result`.
fn parse_record(record: &[u8], result: &mut LogParseResult) {
    // git chèn `\n` **sau** mỗi `\x1e`, nên mọi bản ghi từ bản thứ hai trở đi bắt đầu
    // bằng một byte xuống dòng. Bỏ bước trim này thì trường `%H` mang `\n` ở đầu và
    // **mọi** mã commit dài 41 ký tự — một lỗi làm mọi phép so mã commit về sau thất
    // bại trong im lặng. Xem test `newline_dau_ban_ghi_bi_cat_bo`.
    let record = trim_ascii_whitespace(record);
    if record.is_empty() {
        // Phần dư rỗng ở cuối buffer, hoặc hai `\x1e` liền nhau. Không phải bản ghi méo.
        return;
    }

    // Thu các vị trí `\x1f`. Bản ghi hợp lệ có đúng FIELD_COUNT - 1 dấu.
    let mut field_start = 0usize;
    let mut fields: Vec<&[u8]> = Vec::with_capacity(FIELD_COUNT);
    for sep in memchr_iter(UNIT_SEP, record) {
        if fields.len() == FIELD_COUNT - 1 {
            // Đã đủ chín dấu phân tách: mọi `\x1f` còn lại **thuộc về** trường cuối
            // (`%b`). Dừng tách ở đây thay vì tiếp tục, để phần thừa nằm nguyên trong
            // body thay vì phải nối lại. Xem T-02-05: người tạo commit có thể cố ý
            // chèn `\x1f` vào thông điệp, và điều đó không được sinh ra bản ghi giả.
            break;
        }
        fields.push(&record[field_start..sep]);
        field_start = sep + 1;
    }
    fields.push(&record[field_start..]);

    if fields.len() < FIELD_COUNT {
        // Thiếu trường: định dạng và bộ phân tích đã lệch nhau, hoặc dữ liệu hỏng.
        // Bỏ bản ghi này, giữ mọi bản ghi khác, và đếm để có người biết.
        result.skipped_records += 1;
        return;
    }

    let mut dirty = false;

    // `%P` là danh sách mã hex cách nhau bởi dấu cách; rỗng nghĩa là commit gốc.
    // `split` trên dấu cách rồi bỏ phần tử rỗng, nên `""` cho `vec![]` chứ không phải
    // `vec![""]` — một `parents` chứa chuỗi rỗng sẽ thành một cạnh trỏ tới hư không.
    let parents: Vec<String> = fields[1]
        .split(|&b| b == b' ')
        .filter(|p| !p.is_empty())
        .map(|p| lossy(p, &mut dirty))
        .collect();

    let author_time = match parse_i64(fields[4]) {
        Some(t) => t,
        None => {
            // Timestamp lạ **không** phải lý do giấu commit khỏi đồ thị: nó vẫn có mã,
            // vẫn có cha, vẫn phải chiếm một dòng và một lane.
            result.bad_timestamps += 1;
            0
        }
    };
    let committer_time = match parse_i64(fields[7]) {
        Some(t) => t,
        None => {
            result.bad_timestamps += 1;
            0
        }
    };

    result.commits.push(Commit {
        id: lossy(fields[0], &mut dirty),
        parents,
        author_name: lossy(fields[2], &mut dirty),
        author_email: lossy(fields[3], &mut dirty),
        author_time,
        committer_name: lossy(fields[5], &mut dirty),
        committer_email: lossy(fields[6], &mut dirty),
        committer_time,
        subject: lossy(fields[8], &mut dirty),
        // `%b` giữ nguyên xuống dòng bên trong; chỉ cắt khoảng trắng ở hai đầu, vì git
        // luôn thêm `\n` sau body.
        body: lossy(trim_ascii_whitespace(fields[9]), &mut dirty),
        has_invalid_utf8: dirty,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dựng một bản ghi từ mười trường, đúng cách git in ra:
    /// các trường nối bằng `\x1f`, kết thúc bằng `\x1e\n`.
    fn ban_ghi(fields: [&[u8]; FIELD_COUNT]) -> Vec<u8> {
        let mut out = Vec::new();
        for (i, f) in fields.iter().enumerate() {
            if i > 0 {
                out.push(UNIT_SEP);
            }
            out.extend_from_slice(f);
        }
        out.push(RECORD_SEP);
        out.push(b'\n');
        out
    }

    fn ban_ghi_mau() -> Vec<u8> {
        ban_ghi([
            b"0f96fe6d9aca619b71d986419ad2201b0943183e",
            b"86514e0401ac9e1dfefc87b7ffb7ad12a952f00b",
            b"Plum Fixture",
            b"fixture@git-plum.test",
            b"1577838000",
            b"Nguoi Commit",
            b"committer@git-plum.test",
            b"1577838060",
            b"commit 20",
            b"",
        ])
    }

    /// Ca cơ sở: mọi trường phải về đúng chỗ của nó. Test này bắt lỗi lệch chỉ số —
    /// đổi `fields[2]` với `fields[3]` vẫn cho mười trường và vẫn "đỗ" nếu chỉ đếm.
    #[test]
    fn mot_ban_ghi_cho_mot_commit_moi_truong_dung_cho() {
        let r = parse_log(&ban_ghi_mau());

        assert_eq!(r.commits.len(), 1);
        assert_eq!(r.skipped_records, 0);

        let c = &r.commits[0];
        assert_eq!(c.id, "0f96fe6d9aca619b71d986419ad2201b0943183e");
        assert_eq!(c.parents, vec!["86514e0401ac9e1dfefc87b7ffb7ad12a952f00b"]);
        assert_eq!(c.author_name, "Plum Fixture");
        assert_eq!(c.author_email, "fixture@git-plum.test");
        assert_eq!(c.author_time, 1_577_838_000);
        assert_eq!(c.committer_name, "Nguoi Commit");
        assert_eq!(c.committer_email, "committer@git-plum.test");
        assert_eq!(c.committer_time, 1_577_838_060);
        assert_eq!(c.subject, "commit 20");
        assert_eq!(c.body, "");
        assert!(
            !c.has_invalid_utf8,
            "dữ liệu ASCII sạch không được đánh dấu"
        );
    }

    /// Thứ tự phải giữ **đúng như git in ra**: `--topo-order` bảo đảm cha đứng sau con,
    /// và bộ phân tích sắp xếp lại sẽ phá bất biến mà thuật toán lane dựa vào.
    #[test]
    fn ba_ban_ghi_giu_dung_thu_tu_xuat_hien() {
        let mut buf = Vec::new();
        for i in 0..3u8 {
            let id = vec![b'0' + i; 40];
            buf.extend_from_slice(&ban_ghi([
                &id, b"", b"A", b"a@x", b"100", b"C", b"c@x", b"200", b"chu de", b"",
            ]));
        }

        let r = parse_log(&buf);
        assert_eq!(r.commits.len(), 3, "không được mất bản ghi nào");
        assert_eq!(r.commits[0].id, "0".repeat(40));
        assert_eq!(r.commits[1].id, "1".repeat(40));
        assert_eq!(r.commits[2].id, "2".repeat(40));
    }

    /// Commit gốc: `%P` rỗng phải cho `Vec` **rỗng**, không phải `vec![""]`.
    /// Một chuỗi rỗng trong `parents` sẽ thành cạnh trỏ tới một commit không tồn tại.
    #[test]
    fn commit_goc_cho_parents_rong_chu_khong_phai_chuoi_rong() {
        let buf = ban_ghi([
            b"abc0000000000000000000000000000000000000",
            b"", // không cha
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"commit dau tien",
            b"",
        ]);

        let r = parse_log(&buf);
        assert_eq!(r.commits.len(), 1);
        assert!(
            r.commits[0].parents.is_empty(),
            "commit gốc phải có parents rỗng, nhận được {:?}",
            r.commits[0].parents
        );
    }

    /// Merge octopus bốn cha — ràng buộc số 3 của ROADMAP. Repo mẫu `octopus` có đúng
    /// hình dạng này ở HEAD.
    #[test]
    fn merge_octopus_bon_cha_cho_du_bon() {
        let parents = [
            "1".repeat(40),
            "2".repeat(40),
            "3".repeat(40),
            "4".repeat(40),
        ]
        .join(" ");

        let buf = ban_ghi([
            b"abc0000000000000000000000000000000000000",
            parents.as_bytes(),
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"merge bon nhanh",
            b"",
        ]);

        let r = parse_log(&buf);
        assert_eq!(r.commits[0].parents.len(), 4, "phải giữ đủ bốn cha");
        assert_eq!(r.commits[0].parents[3], "4".repeat(40));
    }

    /// **Đây là HIST-11.** Một byte xấu trong `%s` không được làm mất cả trang: bản ghi
    /// trước và sau nó phải còn nguyên vẹn, và bản ghi xấu phải có mặt kèm cờ lossy.
    #[test]
    fn byte_khong_utf8_khong_lam_mat_ban_ghi_lan_can() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&ban_ghi([
            &[b'a'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"truoc",
            b"",
        ]));
        buf.extend_from_slice(&ban_ghi([
            &[b'b'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"byte xau \xff o day",
            b"",
        ]));
        buf.extend_from_slice(&ban_ghi([
            &[b'c'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"sau",
            b"",
        ]));

        let r = parse_log(&buf);

        assert_eq!(
            r.commits.len(),
            3,
            "một byte xấu không được làm mất dòng nào"
        );
        assert_eq!(r.commits[0].subject, "truoc");
        assert_eq!(r.commits[2].subject, "sau");

        let xau = &r.commits[1];
        assert!(xau.has_invalid_utf8, "phải đánh dấu đã đi qua đường lossy");
        assert!(
            xau.subject.contains('\u{FFFD}'),
            "byte xấu phải thành U+FFFD, nhận được {:?}",
            xau.subject
        );
        assert!(
            !r.commits[0].has_invalid_utf8,
            "bản ghi sạch không bị lây cờ"
        );
        assert!(
            !r.commits[2].has_invalid_utf8,
            "bản ghi sạch không bị lây cờ"
        );
    }

    /// `%b` nhiều dòng phải giữ đủ các dòng — chi tiết commit (HIST-04) hiện cả thân.
    #[test]
    fn body_nhieu_dong_giu_du_cac_dong() {
        let buf = ban_ghi([
            &[b'a'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"chu de",
            b"dong mot\ndong hai\n\ndong bon",
        ]);

        let r = parse_log(&buf);
        let body = &r.commits[0].body;
        assert_eq!(body, "dong mot\ndong hai\n\ndong bon");
        assert_eq!(body.lines().count(), 4);
    }

    /// T-02-05: người tạo commit cố ý chèn `\x1f` vào thông điệp không được tách bản
    /// ghi thành hai. Hành vi đã chọn: phần thừa **nằm lại trong `body`**, và test này
    /// ghi rõ điều đó để nó là quyết định chứ không phải ngẫu nhiên.
    #[test]
    fn byte_1f_trong_body_khong_tach_thanh_hai_ban_ghi() {
        let buf = ban_ghi([
            &[b'a'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"chu de",
            b"than truoc\x1fthan sau",
        ]);

        let r = parse_log(&buf);
        assert_eq!(r.commits.len(), 1, "một bản ghi, không phải hai");
        assert_eq!(r.skipped_records, 0);
        assert_eq!(
            r.commits[0].body, "than truoc\u{1f}than sau",
            "phần sau \\x1f phải nằm lại trong body"
        );
        // Và các trường trước đó vẫn đúng — không bị lệch một nấc.
        assert_eq!(r.commits[0].subject, "chu de");
    }

    /// Buffer rỗng là ca thường (repo mới `git init` chưa có commit nào), không phải lỗi.
    #[test]
    fn buffer_rong_cho_ket_qua_rong_khong_loi() {
        let r = parse_log(b"");
        assert!(r.commits.is_empty());
        assert_eq!(r.skipped_records, 0);
    }

    /// T-02-04: bản ghi cuối bị cắt (đọc theo luồng, trang chưa về hết) phải bị bỏ mà
    /// **không** panic và **không** làm mất các bản ghi hoàn chỉnh trước nó.
    #[test]
    fn ban_ghi_cuoi_bi_cat_bi_bo_cac_ban_ghi_truoc_van_con() {
        let mut buf = ban_ghi([
            &[b'a'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"nguyen ven",
            b"",
        ]);
        // Bản ghi thứ hai dừng giữa chừng: không có `\x1e` kết thúc.
        buf.extend_from_slice(b"bbbbbbbb\x1fcccc\x1fTen Tac Gi");

        let r = parse_log(&buf);
        assert_eq!(r.commits.len(), 1, "bản ghi hoàn chỉnh phải còn");
        assert_eq!(r.commits[0].subject, "nguyen ven");
    }

    /// Bản ghi thiếu trường bị bỏ và **được đếm** — không panic, không làm hỏng bản ghi
    /// khác. Đếm để lệch định dạng không trôi qua trong im lặng.
    #[test]
    fn ban_ghi_thieu_truong_bi_bo_va_duoc_dem() {
        let mut buf = Vec::new();
        // Bản ghi méo: chỉ ba trường.
        buf.extend_from_slice(b"aaaa\x1fbbbb\x1fcccc\x1e\n");
        buf.extend_from_slice(&ban_ghi([
            &[b'd'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"con nguyen",
            b"",
        ]));

        let r = parse_log(&buf);
        assert_eq!(r.skipped_records, 1, "bản ghi méo phải được đếm");
        assert_eq!(r.commits.len(), 1, "bản ghi hợp lệ vẫn phải trả về");
        assert_eq!(r.commits[0].subject, "con nguyen");
    }

    /// git chèn `\n` **sau** mỗi `\x1e` (đã kiểm bằng `xxd` trên fixture `linear`:
    /// buffer chứa `1e 0a`). Không cắt byte đó thì `%H` dài 41 ký tự và mọi phép so mã
    /// commit về sau thất bại trong im lặng — một lỗi rất khó truy.
    #[test]
    fn newline_dau_ban_ghi_bi_cat_bo() {
        let mut buf = ban_ghi([
            &[b'a'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"dau tien",
            b"",
        ]);
        buf.extend_from_slice(&ban_ghi([
            &[b'b'; 40],
            b"",
            b"A",
            b"a@x",
            b"100",
            b"C",
            b"c@x",
            b"200",
            b"thu hai",
            b"",
        ]));

        let r = parse_log(&buf);
        assert_eq!(r.commits.len(), 2);
        for c in &r.commits {
            assert_eq!(
                c.id.len(),
                40,
                "mã commit phải đúng 40 ký tự, nhận được {:?}",
                c.id
            );
            assert!(
                !c.id.starts_with('\n'),
                "newline đầu bản ghi chưa bị cắt: {:?}",
                c.id
            );
        }
        assert_eq!(r.commits[1].id, "b".repeat(40));
    }

    /// Timestamp không đọc được → `0` và **giữ** commit. Commit có đồng hồ hỏng vẫn
    /// phải chiếm một dòng và một lane trên đồ thị.
    #[test]
    fn timestamp_hong_cho_khong_va_van_giu_commit() {
        let buf = ban_ghi([
            &[b'a'; 40],
            b"",
            b"A",
            b"a@x",
            b"khong-phai-so",
            b"C",
            b"c@x",
            b"200",
            b"dong ho hong",
            b"",
        ]);

        let r = parse_log(&buf);
        assert_eq!(
            r.commits.len(),
            1,
            "timestamp hỏng không được làm mất commit"
        );
        assert_eq!(r.commits[0].author_time, 0);
        assert_eq!(r.commits[0].committer_time, 200);
        assert_eq!(r.bad_timestamps, 1);
    }

    /// Timestamp âm (commit ghi ngày trước 1970) là hợp lệ và có thật trong repo cũ.
    /// Đây là lý do trường là `i64` chứ không phải `u64`.
    #[test]
    fn timestamp_am_phan_tich_duoc() {
        assert_eq!(parse_i64(b"-86400"), Some(-86_400));
        assert_eq!(parse_i64(b"1577838000"), Some(1_577_838_000));
        assert_eq!(parse_i64(b""), None);
        assert_eq!(parse_i64(b"12x4"), None);
        // Số dài hơn i64 phải cho None chứ không tràn âm thầm.
        assert_eq!(parse_i64(b"99999999999999999999999999"), None);
    }

    /// [`LOG_FORMAT`] và [`FIELD_COUNT`] là một hợp đồng hai chiều. Test này bắt lỗi
    /// sửa một bên mà quên bên kia — kiểu lỗi làm mọi bản ghi lệch một nấc trong im lặng.
    #[test]
    fn log_format_co_dung_so_truong() {
        let so_dau_phan_tach = LOG_FORMAT.matches("%x1f").count();
        assert_eq!(
            so_dau_phan_tach,
            FIELD_COUNT - 1,
            "LOG_FORMAT có {} dấu %x1f nhưng FIELD_COUNT là {}",
            so_dau_phan_tach,
            FIELD_COUNT
        );
        assert!(
            LOG_FORMAT.ends_with("%x1e"),
            "bản ghi phải kết thúc bằng %x1e"
        );
        assert!(
            LOG_FORMAT.starts_with("--format=%H"),
            "trường đầu phải là %H"
        );
    }

    /// `--date-order` là bắt buộc — thiếu nó thì cha có thể đứng trước con và thuật
    /// toán lane rò. Không được là `--topo-order`: nó dồn nhánh xuống cuối và vẽ lane
    /// bậc thang (xem doc comment của [`LOG_ARGS`]).
    #[test]
    fn log_args_luon_co_date_order() {
        assert!(
            LOG_ARGS.contains(&"--date-order"),
            "--date-order là bắt buộc cho thuật toán lane"
        );
        assert!(!LOG_ARGS.contains(&"--topo-order"));
        assert_eq!(LOG_ARGS[0], "log");
    }

    /// Đếm dòng đầu ra `git log --all --topo-order --format=%H` của một repo mẫu.
    ///
    /// Đây là **mẫu số đúng** để so với số commit phân tích được, và việc chọn nó thay
    /// vì `git rev-list --all --count` là có chủ ý — xem ghi chú ở
    /// [`tests::non_utf8_khong_mat_dong_nao`].
    async fn dem_dong_log(repo: &std::path::Path) -> usize {
        let out = crate::git::GitCommand::new(repo)
            .args(LOG_ARGS)
            .arg("--format=%H")
            .run()
            .await
            .expect("không chạy được git log trên repo mẫu");
        assert!(out.is_success(), "git log thất bại: {}", out.stderr_lossy());

        out.stdout
            .split(|&b| b == b'\n')
            .filter(|l| !trim_ascii_whitespace(l).is_empty())
            .count()
    }

    /// **Bài kiểm chứng thật của HIST-11.** Chạy git thật trên repo mẫu `non-utf8` —
    /// repo có byte thô không hợp lệ trong **cả** đường dẫn lẫn thông điệp commit — rồi
    /// khẳng định số commit phân tích được bằng đúng số dòng git in ra.
    ///
    /// # Vì sao so với `git log` chứ không phải `git rev-list --all --count`
    ///
    /// Bất biến đang kiểm là *"bộ phân tích có bỏ sót bản ghi nào trong đầu ra **nó
    /// nhận được** hay không"*. Mẫu số đúng vì vậy là đầu ra của chính lệnh
    /// `parse_log` đọc.
    ///
    /// Hai lệnh **không** tương đương trong trường hợp tổng quát: `git log` áp
    /// *history simplification* và có thể lược commit có tree trùng với một cha, còn
    /// `rev-list` thì không. Trên mọi repo mẫu hiện tại hai số khớp nhau (plan 02-01
    /// ràng buộc mọi commit phải đổi nội dung tệp), nên viết theo `rev-list` vẫn sẽ đỗ
    /// — nhưng nó phát biểu một chân lý phổ quát vốn không phổ quát, và sẽ vỡ ở repo
    /// khác với thông báo lỗi trỏ sai chỗ.
    #[tokio::test]
    async fn non_utf8_khong_mat_dong_nao() {
        let Some(repo) = crate::testing::require_fixture("non-utf8") else {
            return; // đã in lời nhắc, không phải lỗi
        };

        let mong_doi = dem_dong_log(&repo).await;
        assert!(mong_doi > 0, "repo mẫu non-utf8 phải có commit");

        let out = crate::git::GitCommand::new(&repo)
            .args(LOG_ARGS)
            .arg(LOG_FORMAT)
            .run()
            .await
            .expect("không chạy được git log");
        assert!(out.is_success(), "git log thất bại: {}", out.stderr_lossy());

        let r = parse_log(&out.stdout);

        assert_eq!(
            r.commits.len(),
            mong_doi,
            "HIST-11: mất dòng. git in {} bản ghi, phân tích được {} (bỏ {})",
            mong_doi,
            r.commits.len(),
            r.skipped_records
        );
        assert_eq!(r.skipped_records, 0, "không bản ghi nào được phép méo");

        // Mã commit phải dùng được ngay làm đầu vào cho lệnh git khác: đủ 40 ký tự hex.
        for c in &r.commits {
            assert_eq!(c.id.len(), 40, "mã commit sai độ dài: {:?}", c.id);
            assert!(
                c.id.chars().all(|ch| ch.is_ascii_hexdigit()),
                "mã commit không phải hex: {:?}",
                c.id
            );
        }

        // Và điều làm repo này đáng giá: ít nhất một commit **thật sự** đi qua đường
        // lossy. Không có khẳng định này thì test vẫn đỗ trên một repo ASCII thuần và
        // HIST-11 coi như không được kiểm.
        let so_lossy = r.commits.iter().filter(|c| c.has_invalid_utf8).count();
        assert!(
            so_lossy > 0,
            "repo mẫu non-utf8 phải có ít nhất một commit đi qua đường lossy; \
             nếu số này bằng 0 thì fixture đã hỏng, không phải bộ phân tích đã đúng"
        );
    }

    /// Merge octopus **bốn cha** đọc từ repo thật, không phải từ buffer dựng tay.
    /// Ràng buộc số 3 của ROADMAP đo trên dữ liệu git thật sinh ra.
    #[tokio::test]
    async fn octopus_doc_du_bon_cha_tu_repo_that() {
        let Some(repo) = crate::testing::require_fixture("octopus") else {
            return;
        };

        let out = crate::git::GitCommand::new(&repo)
            .args(LOG_ARGS)
            .arg(LOG_FORMAT)
            .run()
            .await
            .expect("không chạy được git log");

        let r = parse_log(&out.stdout);
        assert_eq!(r.commits.len(), dem_dong_log(&repo).await);

        let so_cha_lon_nhat = r.commits.iter().map(|c| c.parents.len()).max().unwrap_or(0);
        assert_eq!(
            so_cha_lon_nhat, 4,
            "repo mẫu octopus phải có một merge bốn cha"
        );
    }

    /// Mọi repo mẫu đều phải phân tích trọn vẹn — chín hình dạng, không hình dạng nào
    /// làm mất bản ghi. Ca `shallow` (cha trỏ ra ngoài tập dữ liệu) và `detached`
    /// (không nhãn nhánh nào) là hai ca dễ làm bộ phân tích ngây thơ vấp nhất.
    #[tokio::test]
    async fn moi_repo_mau_phan_tich_tron_ven() {
        for name in crate::testing::FIXTURE_NAMES {
            let Some(repo) = crate::testing::fixture_path(name) else {
                continue; // fixture chưa sinh; test khác đã in lời nhắc
            };

            let out = crate::git::GitCommand::new(&repo)
                .args(LOG_ARGS)
                .arg(LOG_FORMAT)
                .run()
                .await
                .expect("không chạy được git log");
            assert!(out.is_success(), "git log thất bại trên {name}");

            let mong_doi = dem_dong_log(&repo).await;
            let r = parse_log(&out.stdout);

            assert_eq!(
                r.commits.len(),
                mong_doi,
                "repo mẫu {name}: git in {} bản ghi, phân tích được {}",
                mong_doi,
                r.commits.len()
            );
            assert_eq!(r.skipped_records, 0, "repo mẫu {name} có bản ghi méo");
        }
    }
}
