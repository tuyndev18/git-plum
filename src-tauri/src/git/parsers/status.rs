//! Phân tích `git status --porcelain=v2 --branch -z --untracked-files=all` — WORK-01, WORK-11.
//!
//! # 🔴 Bản ghi dạng `2` chiếm HAI đoạn NUL, không phải một
//!
//! Đây là chỗ duy nhất trong tệp này mà một bộ phân tích "đúng theo trực giác" sẽ hỏng
//! **im lặng**. Với `-z`, git kết thúc mỗi bản ghi bằng NUL — **trừ** bản ghi dạng `2`
//! (đổi tên/sao chép), vốn mang **hai** đường dẫn ngăn nhau bằng NUL **bên trong** một
//! bản ghi logic. Đo thật trên git 2.54.0.windows.1 bằng `od -c` (2026-09-22), sau
//! `git mv a_old.txt a_new.txt` cộng hai tệp sửa đã stage:
//!
//! ```text
//! 0000260   c   1   9   c   5   2   f   3   4       R   1   0   0       a
//! 0000300   _   n   e   w   .   t   x   t  \0   a   _   o   l   d   .   t
//! 0000320   x   t  \0   1       M   .       N   .   .   .       1   0   0
//!                  ^^^^^^^^
//!            NUL rồi ĐƯỜNG DẪN CŨ rồi NUL — vẫn là MỘT bản ghi
//! ```
//!
//! Một bộ phân tích tách `\0` trơn rồi coi mỗi đoạn là một bản ghi đọc `a_old.txt`
//! thành một bản ghi **dạng `a`**, rồi gán sai **mọi** bản ghi sau nó — lệch nấc. Không
//! có lỗi, không có cảnh báo, chỉ có một danh sách tệp sai.
//!
//! Lớp lỗi này đã xảy ra **hai lần** trong dự án và cả hai lần đều im lặng:
//!
//! * bản ghi `R` của `--name-status` ở wave 5 Phase 3;
//! * `\0\n` thay vì `\0` trơn ở plan 02-04 (xem [`super::file_history`]).
//!
//! **Cách nó không xảy ra ở đây:** [`parse_status`] quét **tuần tự** và **dạng bản ghi
//! quyết định tiêu thụ bao nhiêu đoạn NUL**. Không có `split(0)` nào trong tệp này.
//! Chỉ dạng `2` đọc thêm một đoạn nữa. Test `hai_ban_ghi_sau_dang_2_khong_lech_nac`
//! ghim điều đó với **hai** bản ghi đứng sau — một bản ghi đứng sau chỉ chứng minh được
//! nửa vấn đề, vì một bộ phân tích lệch nấc vẫn cho đúng số lượng khi chỉ có một.
//!
//! # Trường ngăn nhau bằng KHOẢNG TRẮNG, không phải `\x1f`
//!
//! Khác `LOG_FORMAT` (dùng `%x1f`): ở đây các trường trong một bản ghi ngăn nhau bằng
//! khoảng trắng đơn. Đường dẫn là trường **cuối** nên nó **được phép chứa khoảng
//! trắng** — đã đo: `? z with space.txt`. Vì vậy phép tách trường phải có **giới hạn
//! số lần**; tách hết sẽ cắt tên tệp ở khoảng trắng đầu tiên.
//!
//! Đừng mang giả định `\x1f` từ bộ phân tích log sang đây.
//!
//! # Dòng `# branch.ab` VẮNG MẶT khi không có upstream
//!
//! git **không** in `+0 -0`; nó không in dòng đó. Đã đo: repo mới `git init` chỉ có
//! `branch.oid` và `branch.head`. Xem [`crate::domain::BranchInfo::ahead`] về lý do
//! WORK-09 cần phân biệt `None` với `Some(0)`.

use memchr::memchr;

use crate::domain::{BranchInfo, RepoStatus, StatusEntry, StatusGroup};

/// Byte kết thúc một đoạn của `-z`.
const NUL: u8 = 0;

/// Phân tích đầu ra của lệnh [`crate::domain::STATUS_ARGS`].
///
/// Hàm **thuần**: không async, không I/O, không sinh lệnh git. Khuôn của
/// [`super::log::parse_log`] và [`super::file_history::parse_file_history`].
///
/// # Một bản ghi méo không làm hỏng cả trang
///
/// Quy tắc chung của module `parsers` (HIST-11, ở đây áp cho trạng thái): bản ghi không
/// đọc được thì **bỏ** nó và trả phần còn lại. Cụ thể ở đây, đầu vào **cắt ngang** —
/// một bản ghi dạng `2` chỉ có đường dẫn mới, thiếu đường dẫn cũ — là ca thật, vì lệnh
/// có thể bị hạn giờ hoặc bị giết giữa đường. Nó phải cho một `RepoStatus` thiếu bản
/// ghi dở đó, **không** panic.
pub fn parse_status(stdout: &[u8]) -> RepoStatus {
    let mut ket_qua = RepoStatus::default();
    let mut con_lai = stdout;

    while !con_lai.is_empty() {
        let (doan, phan_du) = cat_doan(con_lai);
        con_lai = phan_du;

        if doan.is_empty() {
            continue;
        }

        // 🔴 Phân biệt dạng bản ghi bằng **byte đầu + một khoảng trắng theo sau**, không
        // phải chỉ byte đầu. Một đường dẫn bắt đầu bằng ký tự `2` là hợp lệ (`2.txt`),
        // và ở đầu vào cắt ngang nó có thể rơi đúng vào vị trí mà một bản ghi được phép
        // bắt đầu. Chỉ xem byte đầu thì ta sẽ đi đọc thêm một đoạn NUL cho một "đường
        // dẫn cũ" không tồn tại và ăn mất bản ghi kế tiếp.
        match doan.first().copied() {
            Some(b'#') if la_dang(doan, b'#') => doc_dong_branch(doan, &mut ket_qua.branch),

            Some(b'1') if la_dang(doan, b'1') => {
                if let Some(mut muc) = doc_ban_ghi_thuong(doan) {
                    them_theo_nhom(&mut ket_qua.entries, &mut muc);
                }
            }

            // Dạng `2` — và CHỈ dạng `2` — tiêu thụ một đoạn NUL thứ hai.
            Some(b'2') if la_dang(doan, b'2') => {
                let (duong_dan_cu, phan_du) = cat_doan(con_lai);

                // Đầu vào cắt ngang: đường dẫn cũ thiếu hẳn. Bỏ bản ghi dở, đừng đoán.
                if duong_dan_cu.is_empty() {
                    con_lai = phan_du;
                    continue;
                }
                con_lai = phan_du;

                if let Some(mut muc) = doc_ban_ghi_doi_ten(doan, duong_dan_cu) {
                    them_theo_nhom(&mut ket_qua.entries, &mut muc);
                }
            }

            Some(b'u') if la_dang(doan, b'u') => {
                ket_qua.has_conflicts = true;
                if let Some(muc) = doc_ban_ghi_xung_dot(doan) {
                    ket_qua.entries.push(muc);
                }
            }

            Some(b'?') if la_dang(doan, b'?') => {
                if let Some(muc) = doc_ban_ghi_chua_theo_doi(doan) {
                    ket_qua.entries.push(muc);
                }
            }

            // Dạng `!` (bị ignore) — **bỏ qua có chủ ý**, không vào `entries`.
            //
            // Ba nhóm của WORK-01 là đã stage / chưa stage / chưa theo dõi; tệp bị
            // `.gitignore` loại không thuộc nhóm nào và không stage được. Cho nó vào
            // `entries` làm `is_clean()` sai ở **mọi** repo có `.gitignore` — nghĩa là
            // gần như mọi repo — nên hàng WIP của WORK-11 sẽ không bao giờ biến mất.
            // git chỉ in dạng này khi có `--ignored`, mà STATUS_ARGS không có; xử lý ở
            // đây để ai thêm cờ đó sau này không vô tình đổi nghĩa `is_clean()`.
            Some(b'!') if la_dang(doan, b'!') => {}

            // Đoạn không nhận ra: bỏ, không panic. Gồm cả phần đuôi của một bản ghi
            // cắt ngang và đường dẫn cũ mồ côi của một dạng `2` bị cắt đầu.
            _ => {}
        }
    }

    ket_qua
}

/// Cắt một đoạn tới NUL, trả `(đoạn, phần còn lại sau NUL)`.
///
/// Không có NUL nào nữa thì cả phần còn lại là một đoạn (đầu vào cắt ngang) và phần dư
/// là rỗng — nên vòng lặp của [`parse_status`] luôn kết thúc.
fn cat_doan(bytes: &[u8]) -> (&[u8], &[u8]) {
    match memchr(NUL, bytes) {
        Some(i) => (&bytes[..i], &bytes[i + 1..]),
        None => (bytes, &[]),
    }
}

/// `true` khi đoạn mở đầu đúng bằng `<dang>` rồi **một khoảng trắng**.
///
/// Phép kiểm khoảng trắng là thứ ngăn một đường dẫn tên `2.txt` bị đọc thành một bản
/// ghi đổi tên. Xem ghi chú ở chỗ gọi trong [`parse_status`].
fn la_dang(doan: &[u8], dang: u8) -> bool {
    doan.len() >= 2 && doan[0] == dang && doan[1] == b' '
}

/// Giải mã lossy một đường dẫn, kèm cờ cho biết có byte không hợp lệ.
///
/// Trả `(chuỗi hiển thị, có_byte_khong_hop_le)`. **Không** `String::from_utf8().unwrap()`:
/// quy tắc chung của module `parsers`, và tên tệp không phải UTF-8 là ca thật trên
/// Linux/macOS (trên Windows/NTFS thì hệ thống tệp tự chặn — xem ghi chú trong
/// `make-status-fixtures.sh`).
fn giai_ma_duong_dan(bytes: &[u8]) -> (String, bool) {
    match std::str::from_utf8(bytes) {
        Ok(s) => (s.to_owned(), false),
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), true),
    }
}

/// Tách `doan` thành `so_truong` trường đầu ngăn bằng khoảng trắng, cộng **phần đuôi
/// nguyên vẹn**.
///
/// 🔴 Đây là phép tách **có giới hạn số lần** mà doc comment đầu module nói tới. Đường
/// dẫn là trường cuối và được phép chứa khoảng trắng (`z with space.txt` — đã đo), nên
/// tách hết rồi lấy phần tử cuối sẽ cắt tên tệp ở khoảng trắng đầu tiên. Đột biến M3
/// của plan ghim đúng chỗ này.
///
/// `None` khi đoạn không đủ `so_truong` trường — bản ghi méo, chỗ gọi bỏ nó.
fn tach_truong(doan: &[u8], so_truong: usize) -> Option<(Vec<&[u8]>, &[u8])> {
    let mut truong = Vec::with_capacity(so_truong);
    let mut con_lai = doan;

    for _ in 0..so_truong {
        let i = memchr(b' ', con_lai)?;
        truong.push(&con_lai[..i]);
        con_lai = &con_lai[i + 1..];
    }

    if con_lai.is_empty() {
        return None;
    }

    Some((truong, con_lai))
}

/// Đọc XY từ một trường hai ký tự, `None` nếu không đúng hai ký tự.
fn doc_xy(truong: &[u8]) -> Option<String> {
    if truong.len() != 2 {
        return None;
    }
    Some(String::from_utf8_lossy(truong).into_owned())
}

/// Bản ghi dạng `1`: `1 <XY> <sub> <mH> <mI> <mW> <hH> <hI> <path>` — 8 trường rồi path.
fn doc_ban_ghi_thuong(doan: &[u8]) -> Option<StatusEntry> {
    let (truong, duong_dan) = tach_truong(doan, 8)?;
    let xy = doc_xy(truong[1])?;
    let (path, hong) = giai_ma_duong_dan(duong_dan);

    Some(StatusEntry {
        path,
        old_path: None,
        xy,
        // Nhóm thật do `them_theo_nhom` quyết định từ XY; giá trị này là chỗ giữ tạm.
        group: StatusGroup::Staged,
        has_invalid_utf8: hong,
    })
}

/// Bản ghi dạng `2`: như dạng `1` cộng trường điểm số đổi tên (`R100`) — 9 trường rồi path.
///
/// `duong_dan_cu` là đoạn NUL **thứ hai** của cùng bản ghi. git in **mới trước, cũ
/// sau**; đảo lại là đột biến M2 và test `dang_2_duong_dan_moi_truoc_cu_sau` ghim nó.
///
/// Trường cuối trước đường dẫn mới là điểm số (`R100`/`C075`) — 9 trường chứ không 8,
/// nếu không điểm số bị nuốt vào `path`.
fn doc_ban_ghi_doi_ten(doan: &[u8], duong_dan_cu: &[u8]) -> Option<StatusEntry> {
    let (truong, duong_dan_moi) = tach_truong(doan, 9)?;
    let xy = doc_xy(truong[1])?;
    let (path, hong_moi) = giai_ma_duong_dan(duong_dan_moi);
    let (old, hong_cu) = giai_ma_duong_dan(duong_dan_cu);

    Some(StatusEntry {
        path,
        old_path: Some(old),
        xy,
        group: StatusGroup::Staged,
        has_invalid_utf8: hong_moi || hong_cu,
    })
}

/// Bản ghi dạng `u`: `u <XY> <sub> <m1> <m2> <m3> <mW> <h1> <h2> <h3> <path>` — 10 trường.
///
/// Xung đột không chia được thành "đã stage / chưa stage": tệp đang ở giữa một merge và
/// người dùng phải giải quyết nó. Xếp vào [`StatusGroup::Unstaged`] vì đó là nhóm mà
/// giao diện hiện việc **cần làm**, và [`RepoStatus::has_conflicts`] là cờ mà vòng
/// commit thật sự đọc.
fn doc_ban_ghi_xung_dot(doan: &[u8]) -> Option<StatusEntry> {
    let (truong, duong_dan) = tach_truong(doan, 10)?;
    let xy = doc_xy(truong[1])?;
    let (path, hong) = giai_ma_duong_dan(duong_dan);

    Some(StatusEntry {
        path,
        old_path: None,
        xy,
        group: StatusGroup::Unstaged,
        has_invalid_utf8: hong,
    })
}

/// Bản ghi dạng `?`: `? <path>` — một trường rồi path.
fn doc_ban_ghi_chua_theo_doi(doan: &[u8]) -> Option<StatusEntry> {
    let (_, duong_dan) = tach_truong(doan, 1)?;
    let (path, hong) = giai_ma_duong_dan(duong_dan);

    Some(StatusEntry {
        path,
        old_path: None,
        xy: "??".to_owned(),
        group: StatusGroup::Untracked,
        has_invalid_utf8: hong,
    })
}

/// Đẩy một phần tử vào một nhóm — hoặc **hai** nhóm khi XY nói cả hai phía đều đổi.
///
/// Ký tự **đầu** của XY là trạng thái index (→ [`StatusGroup::Staged`]), ký tự **sau**
/// là thư mục làm việc (→ [`StatusGroup::Unstaged`]), `.` là không đổi.
///
/// Một tệp XY = `MM` sinh **hai** phần tử vì nó thật sự xuất hiện ở cả hai nhóm trên
/// giao diện (khuôn GitHub Desktop) — đó là hành vi đúng cho WORK-01 và là lý do
/// [`RepoStatus::wip_counts`] phải đếm theo **đường dẫn duy nhất**.
fn them_theo_nhom(dich: &mut Vec<StatusEntry>, muc: &mut StatusEntry) {
    let mut byte = muc.xy.bytes();
    let x = byte.next().unwrap_or(b'.');
    let y = byte.next().unwrap_or(b'.');

    if x != b'.' {
        let mut m = muc.clone();
        m.group = StatusGroup::Staged;
        dich.push(m);
    }
    if y != b'.' {
        let mut m = muc.clone();
        m.group = StatusGroup::Unstaged;
        dich.push(m);
    }
}

/// Đọc một dòng `# branch.*` vào [`BranchInfo`].
///
/// Bốn dòng có thật (đã đo): `branch.oid`, `branch.head`, `branch.upstream`,
/// `branch.ab`. Dòng lạ thì bỏ qua — git có thể thêm dòng mới ở phiên bản sau và một
/// bộ phân tích chết vì gặp dòng nó chưa biết là một bộ phân tích tệ.
fn doc_dong_branch(doan: &[u8], ra: &mut BranchInfo) {
    let Some((_, con_lai)) = tach_truong(doan, 1) else {
        return;
    };
    let Some(i) = memchr(b' ', con_lai) else {
        return;
    };
    let khoa = &con_lai[..i];
    let gia_tri = &con_lai[i + 1..];

    match khoa {
        // `(initial)` ở repo chưa có commit nào — không phải một SHA, nên `None`.
        b"branch.oid" => {
            ra.oid = (gia_tri != b"(initial)")
                .then(|| String::from_utf8_lossy(gia_tri).into_owned());
        }

        // `(detached)` khi HEAD tách rời — ánh xạ thành `None` để giao diện có một
        // phép kiểm dứt khoát thay vì một phép so chuỗi rải khắp nơi.
        b"branch.head" => {
            ra.head = (gia_tri != b"(detached)")
                .then(|| String::from_utf8_lossy(gia_tri).into_owned());
        }

        b"branch.upstream" => {
            ra.upstream = Some(String::from_utf8_lossy(gia_tri).into_owned());
        }

        // `# branch.ab +N -M`. Dòng này VẮNG MẶT khi không có upstream, nên việc không
        // gán gì ở đây là cách `ahead`/`behind` giữ nguyên `None` — xem
        // `BranchInfo::ahead`. Đột biến M7 ghim điều đó.
        b"branch.ab" => {
            let mut phan = gia_tri.split(|b| *b == b' ');
            ra.ahead = phan.next().and_then(doc_so_co_dau);
            ra.behind = phan.next().and_then(doc_so_co_dau);
        }

        _ => {}
    }
}

/// Đọc `+3` hoặc `-2` thành `3`/`2`. `None` nếu không đúng dạng.
fn doc_so_co_dau(truong: &[u8]) -> Option<u32> {
    let (dau, so) = truong.split_first()?;
    if *dau != b'+' && *dau != b'-' {
        return None;
    }
    std::str::from_utf8(so).ok()?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Byte thật của một bản ghi dạng `2` rồi **hai** bản ghi dạng `1`.
    ///
    /// Sao đúng từ `od -c` trên repo mẫu `status-cases` (2026-09-22), không phải tự
    /// dựng theo trí nhớ: chính khoảng cách giữa "byte ta tưởng git in" và "byte git
    /// thật sự in" đã làm hỏng hai bộ phân tích trước (xem doc comment đầu module).
    const DANG_2_ROI_HAI_DANG_1: &[u8] = b"# branch.oid 31b31da1e0a130b4f360ad453d9d70f440e9abfd\0\
        # branch.head main\0\
        2 R. N... 100644 100644 100644 72943a16fb2c8f38f9dde202b7a70ccc19c52f34 72943a16fb2c8f38f9dde202b7a70ccc19c52f34 R100 a_new.txt\0a_old.txt\0\
        1 M. N... 100644 100644 100644 b68025345d5301abad4d9ec9166f455243a0d746 8781b9bf316ddad22ae05c901b7990ab33f286e6 m_one.txt\0\
        1 M. N... 100644 100644 100644 b68025345d5301abad4d9ec9166f455243a0d746 8781b9bf316ddad22ae05c901b7990ab33f286e6 n_two.txt\0";

    /// 🔴 **Ca chịu lực của cả plan.** Đột biến M1 phải làm test này đỏ.
    ///
    /// # Vì sao test này KHÔNG dùng `DANG_2_ROI_HAI_DANG_1`, và đó là một phát hiện
    ///
    /// Bản đầu của test này chạy trên `DANG_2_ROI_HAI_DANG_1` (đường dẫn cũ là
    /// `a_old.txt`) và **sống sót qua đột biến M1** — 0 test đỏ. Nguyên nhân đo được:
    /// khi bản ghi dạng `2` không tiêu thụ đoạn NUL thứ hai, đoạn `a_old.txt` rò ra ở
    /// vòng lặp sau; nhưng `a_old.txt` **không chứa khoảng trắng**, nên `la_dang` trượt
    /// mọi dạng, nó rơi vào nhánh `_ => {}` và **bị bỏ trong im lặng**. Kết quả vẫn là
    /// đúng 3 phần tử với đúng đường dẫn — fixture không phân biệt được gì, đúng lỗi
    /// cổng #4 của CONTEXT.md mục 3.1.
    ///
    /// Điều đó cũng nói một điều về *mã*: phép lệch nấc chỉ **im lặng** khi đường dẫn cũ
    /// tình cờ vô hại. Đường dẫn cũ **có khoảng trắng** thì đoạn rò ra thành một bản ghi
    /// thật sự sai, và đó là ca phải ghim. Tên tệp có khoảng trắng là ca thường, không
    /// dị biệt — cả `make-status-fixtures.sh` cũng dựng một tệp như vậy.
    ///
    /// Thử `old name.txt` cũng **không** đủ: nó có khoảng trắng nhưng token đầu (`old`)
    /// không phải ký tự dạng, nên `la_dang` vẫn trượt và đoạn rò ra vẫn bị bỏ im lặng.
    /// Phép khẳng định `old_path` cũng không bắt được M1, vì M1 chỉ bỏ bước **tiêu thụ**
    /// — `duong_dan_cu` vẫn được đọc đúng, nên `old_path` vẫn đúng.
    ///
    /// Thứ **thật sự** phân biệt M1 là đoạn rò ra phải **trông giống một bản ghi**, để
    /// nó biến thành một phần tử rác đếm được. Tên tệp `? cu.txt` làm được điều đó: git
    /// cho phép mọi byte trừ NUL và `/` trong tên tệp, nên đây là dữ liệu hợp lệ. Đã
    /// kiểm: với đầu vào này M1 cho **3** phần tử thay vì 2, và test đỏ.
    #[test]
    fn hai_ban_ghi_sau_dang_2_khong_lech_nac() {
        // Đường dẫn cũ TRÔNG GIỐNG một bản ghi dạng `?`, và SAU bản ghi dạng `2` có HAI
        // bản ghi nữa. Hai là số tối thiểu: với một bản ghi, bộ phân tích lệch nấc vẫn
        // cho đúng số lượng và test không phân biệt được gì.
        let vao: &[u8] = b"2 R. N... 100644 100644 100644 aaa bbb R100 a_new.txt\0? cu.txt\0\
            1 M. N... 100644 100644 100644 ccc ddd m_one.txt\0\
            1 M. N... 100644 100644 100644 eee fff n_two.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            3,
            "phải đúng 3 phần tử (1 đổi tên + 2 sửa). Được {} — \
             bộ phân tích đang coi đường dẫn cũ của bản ghi dạng 2 là một bản ghi riêng: {:?}",
            st.entries.len(),
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );

        // 🔴 Phép phân biệt thật của M1: đường dẫn cũ phải được GHÉP vào bản ghi đổi
        // tên. Không tiêu thụ đoạn thứ hai thì `old_path` là None (hoặc sai), bất kể
        // đoạn rò ra sau đó có bị bỏ im lặng hay không.
        assert_eq!(
            st.entries[0].path, "a_new.txt",
            "được: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
        assert_eq!(
            st.entries[0].old_path,
            Some("? cu.txt".to_owned()),
            "đường dẫn cũ phải được ghép vào CHÍNH bản ghi đổi tên"
        );

        // 🔴 Phép phân biệt M1: không có phần tử nào thuộc nhóm Untracked. Đầu vào không
        // có bản ghi dạng `?` nào; một phần tử Untracked nghĩa là đoạn `? cu.txt` đã
        // KHÔNG được tiêu thụ và bị đọc lại thành một bản ghi riêng.
        assert!(
            !st.entries
                .iter()
                .any(|e| e.group == StatusGroup::Untracked),
            "đoạn đường dẫn cũ `? cu.txt` bị đọc lại thành bản ghi dạng `?` — \
             bản ghi dạng 2 không tiêu thụ đoạn NUL thứ hai: {:?}",
            st.entries
                .iter()
                .map(|e| (&e.path, e.group))
                .collect::<Vec<_>>()
        );

        assert_eq!(st.entries[1].path, "m_one.txt");
        assert_eq!(
            st.entries[2].path, "n_two.txt",
            "phần tử CUỐI phải là n_two.txt; sai ở đây nghĩa là mọi bản ghi sau bản ghi \
             dạng 2 đã bị lệch một nấc"
        );
        // Hai bản ghi dạng `1` KHÔNG được mang old_path — nếu chúng có, nghĩa là bộ
        // phân tích đang ghép sai đoạn cho sai bản ghi.
        assert_eq!(st.entries[1].old_path, None);
        assert_eq!(st.entries[2].old_path, None);
    }

    /// 🔴 M1 phần hai: đường dẫn cũ **tự nó trông giống một bản ghi** thì lệch nấc
    /// không còn im lặng — nó sinh thêm một phần tử rác.
    ///
    /// Ca này bổ sung cho test trên: ở đó phép phân biệt là `old_path` bị mất; ở đây là
    /// `entries.len()` tăng. Một đột biến phải vượt **cả hai** mới lọt, và không có đột
    /// biến nào làm được điều đó.
    ///
    /// Đường dẫn `? cu.txt` là tên tệp hợp lệ (git cho phép mọi byte trừ NUL và `/`),
    /// và nó **có** khoảng trắng sau một ký tự dạng — nên khi rò ra nó bị đọc thành một
    /// bản ghi dạng `?` thật.
    #[test]
    fn duong_dan_cu_trong_giong_ban_ghi_thi_lech_nac_sinh_phan_tu_rac() {
        let vao: &[u8] = b"2 R. N... 100644 100644 100644 aaa bbb R100 moi.txt\0? cu.txt\0\
            1 M. N... 100644 100644 100644 ccc ddd sau.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            2,
            "đường dẫn cũ `? cu.txt` phải được TIÊU THỤ như một phần của bản ghi đổi \
             tên, không được đọc lại thành một bản ghi dạng `?`. Được: {:?}",
            st.entries
                .iter()
                .map(|e| (&e.path, e.group))
                .collect::<Vec<_>>()
        );
        assert_eq!(st.entries[0].path, "moi.txt");
        assert_eq!(st.entries[0].old_path, Some("? cu.txt".to_owned()));
        assert_eq!(st.entries[1].path, "sau.txt");
        assert!(
            !st.entries
                .iter()
                .any(|e| e.group == StatusGroup::Untracked),
            "không có tệp chưa theo dõi nào trong đầu vào; một phần tử Untracked nghĩa là \
             đường dẫn cũ đã bị đọc lại thành bản ghi dạng `?` — lệch nấc: {:?}",
            st.entries
                .iter()
                .map(|e| (&e.path, e.group))
                .collect::<Vec<_>>()
        );
    }

    /// 🔴 Đột biến M2: git in đường dẫn **mới trước, cũ sau** — đã đo bằng `od -c`.
    #[test]
    fn dang_2_duong_dan_moi_truoc_cu_sau() {
        let st = parse_status(DANG_2_ROI_HAI_DANG_1);

        assert_eq!(
            st.entries[0].path, "a_new.txt",
            "đường dẫn MỚI nằm ở đoạn NUL thứ nhất"
        );
        assert_eq!(
            st.entries[0].old_path,
            Some("a_old.txt".to_owned()),
            "đường dẫn CŨ nằm ở đoạn NUL thứ hai"
        );
        assert_eq!(
            st.entries[0].xy, "R.",
            "XY của một đổi tên đã stage là `R.`; điểm số R100 KHÔNG được lọt vào XY"
        );
    }

    /// **Hai** bản ghi dạng `2` liên tiếp rồi một dạng `1`.
    ///
    /// Một bản ghi dạng `2` chỉ chứng minh được nửa vấn đề: một bộ phân tích tiêu thụ
    /// **sai số lượng** đoạn (ví dụ luôn đọc hai đoạn cho mọi dạng) vẫn có thể lọt qua
    /// ca một bản ghi.
    #[test]
    fn hai_ban_ghi_dang_2_lien_tiep_roi_mot_dang_1() {
        let vao: &[u8] = b"2 R. N... 100644 100644 100644 aaa bbb R100 a_new.txt\0a_old.txt\0\
            2 R. N... 100644 100644 100644 ccc ddd R090 b_new.txt\0b_old.txt\0\
            1 M. N... 100644 100644 100644 eee fff z_last.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            3,
            "được {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
        assert_eq!(st.entries[0].old_path, Some("a_old.txt".to_owned()));
        assert_eq!(st.entries[1].old_path, Some("b_old.txt".to_owned()));
        assert_eq!(
            st.entries[2].path, "z_last.txt",
            "bản ghi sau HAI bản ghi dạng 2 vẫn phải đúng"
        );
        assert_eq!(st.entries[2].old_path, None);
    }

    /// 🔴 Đột biến M3: đường dẫn chứa khoảng trắng phải nguyên vẹn.
    ///
    /// Đường dẫn là trường **cuối** nên phép tách trường phải có giới hạn số lần. Tách
    /// hết rồi lấy phần tử cuối cho `"space.txt"` chứ không phải `"z with space.txt"`.
    #[test]
    fn duong_dan_co_khoang_trang_khong_bi_cat() {
        let vao: &[u8] = b"? z with space.txt\0\
            1 M. N... 100644 100644 100644 aaa bbb dir name/file name.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 2);
        assert_eq!(
            st.entries[0].path, "z with space.txt",
            "tên tệp chưa theo dõi bị cắt ở khoảng trắng — phép tách trường không có giới hạn"
        );
        assert_eq!(
            st.entries[1].path, "dir name/file name.txt",
            "đường dẫn dạng `1` có HAI khoảng trắng cũng phải nguyên vẹn"
        );
    }

    /// Đường dẫn đổi tên chứa khoảng trắng ở **cả hai** đoạn NUL.
    #[test]
    fn duong_dan_doi_ten_co_khoang_trang_ca_hai_dau() {
        let vao: &[u8] =
            b"2 R. N... 100644 100644 100644 aaa bbb R100 a new name.txt\0old name here.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 1);
        assert_eq!(st.entries[0].path, "a new name.txt");
        assert_eq!(st.entries[0].old_path, Some("old name here.txt".to_owned()));
    }

    /// 🔴 Đột biến M10: đường dẫn byte không phải UTF-8 vẫn có mặt, kèm cờ.
    ///
    /// `\xe9` là `é` trong Latin-1 và **không** hợp lệ UTF-8 đơn lẻ. Đây là cổng
    /// non-UTF-8 thật sự của plan: nó chạy giống nhau trên mọi nền tảng, khác với
    /// fixture repo thật (Windows/NTFS không đựng được byte thô trong tên tệp — xem
    /// ghi chú trong `make-status-fixtures.sh`).
    #[test]
    fn duong_dan_byte_latin1_khong_mat_ban_ghi() {
        let vao: &[u8] = b"? caf\xe9.txt\0? sau_do.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            2,
            "byte không hợp lệ KHÔNG được làm mất bản ghi nào, kể cả bản ghi sau nó"
        );
        assert!(
            st.entries[0].has_invalid_utf8,
            "đường dẫn có byte \\xe9 phải được đánh dấu has_invalid_utf8"
        );
        assert!(
            st.entries[0].path.contains('\u{fffd}'),
            "giải mã lossy phải cho U+FFFD, được: {:?}",
            st.entries[0].path
        );
        assert!(
            !st.entries[1].has_invalid_utf8,
            "bản ghi UTF-8 hợp lệ KHÔNG được bị đánh dấu chỉ vì đứng sau một bản ghi hỏng"
        );
    }

    /// Byte không hợp lệ ở đường dẫn **cũ** của một đổi tên cũng phải đặt cờ.
    #[test]
    fn duong_dan_cu_byte_latin1_cung_dat_co() {
        let vao: &[u8] =
            b"2 R. N... 100644 100644 100644 aaa bbb R100 moi.txt\0c\xe9u.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 1);
        assert!(
            st.entries[0].has_invalid_utf8,
            "byte hỏng ở ĐƯỜNG DẪN CŨ cũng làm bản ghi không dùng lại được làm đối số git"
        );
    }

    /// `# branch.ab +3 -2` → `Some(3)` / `Some(2)`.
    #[test]
    fn branch_ab_doc_duoc_ahead_va_behind() {
        let vao: &[u8] = b"# branch.oid abc123\0# branch.head main\0\
            # branch.upstream origin/main\0# branch.ab +3 -2\0";

        let st = parse_status(vao);

        assert_eq!(st.branch.ahead, Some(3));
        assert_eq!(st.branch.behind, Some(2));
        assert_eq!(st.branch.upstream, Some("origin/main".to_owned()));
        assert_eq!(st.branch.head, Some("main".to_owned()));
        assert_eq!(st.branch.oid, Some("abc123".to_owned()));
    }

    /// 🔴 Đột biến M7: thiếu dòng `# branch.ab` → `None`, **không** `Some(0)`.
    ///
    /// Đã đo: git **không** in `+0 -0` khi nhánh không có upstream; nó không in dòng đó.
    /// WORK-09 dùng phân biệt này để không cảnh báo amend ở repo chưa từng push.
    #[test]
    fn khong_co_branch_ab_thi_none_khong_phai_some_0() {
        let vao: &[u8] = b"# branch.oid abc123\0# branch.head main\0";

        let st = parse_status(vao);

        assert_eq!(
            st.branch.ahead, None,
            "thiếu dòng branch.ab phải cho None; Some(0) nghĩa là ĐÃ push và đồng bộ"
        );
        assert_eq!(st.branch.behind, None);
        assert_eq!(
            st.branch.upstream, None,
            "không có dòng branch.upstream thì upstream cũng phải None"
        );
    }

    /// `# branch.head (detached)` → `head == None`.
    #[test]
    fn branch_head_detached_thi_none() {
        let vao: &[u8] = b"# branch.oid abc123\0# branch.head (detached)\0";

        let st = parse_status(vao);

        assert_eq!(
            st.branch.head, None,
            "HEAD tách rời phải cho None, không phải chuỗi \"(detached)\""
        );
        assert_eq!(
            st.branch.oid,
            Some("abc123".to_owned()),
            "HEAD tách rời vẫn có OID"
        );
    }

    /// Repo chưa có commit: `# branch.oid (initial)` → `oid == None`.
    #[test]
    fn branch_oid_initial_thi_none() {
        let vao: &[u8] = b"# branch.oid (initial)\0# branch.head main\0";

        let st = parse_status(vao);

        assert_eq!(st.branch.oid, None);
        assert_eq!(st.branch.head, Some("main".to_owned()));
    }

    /// Bản ghi dạng `u` (xung đột) → `has_conflicts == true`.
    ///
    /// Byte thật, đo trên một merge xung đột (2026-09-22): dạng `u` có **mười** trường
    /// trước đường dẫn (ba mode stage + mode worktree + ba hash), không tám như dạng `1`.
    #[test]
    fn ban_ghi_dang_u_dat_co_xung_dot() {
        let vao: &[u8] = b"u UU N... 100644 100644 100644 100644 df967b96 ba2906d0 e45c9c26 c.txt\0";

        let st = parse_status(vao);

        assert!(
            st.has_conflicts,
            "bản ghi dạng `u` phải đặt has_conflicts — vòng commit của WORK-08 đọc cờ này"
        );
        assert_eq!(st.entries.len(), 1);
        assert_eq!(
            st.entries[0].path, "c.txt",
            "dạng `u` có 10 trường trước path, không 8 — đếm sai thì path bị cắt"
        );
        assert_eq!(st.entries[0].xy, "UU");
        assert!(!st.is_clean(), "repo có xung đột KHÔNG sạch");
    }

    /// Không có bản ghi `u` nào → `has_conflicts == false`.
    #[test]
    fn khong_co_dang_u_thi_khong_co_xung_dot() {
        let st = parse_status(DANG_2_ROI_HAI_DANG_1);
        assert!(!st.has_conflicts);
    }

    /// 🔴 Đột biến M9: dạng `!` (bị ignore) **bỏ qua**, không vào `entries`.
    ///
    /// Cho nó vào `entries` làm `is_clean()` sai ở mọi repo có `.gitignore`, nên hàng
    /// WIP của WORK-11 không bao giờ biến mất (tiêu chí 7).
    #[test]
    fn ban_ghi_dang_ignored_bi_bo_qua() {
        let vao: &[u8] = b"! target/debug/build\0! node_modules/\0";

        let st = parse_status(vao);

        assert!(
            st.entries.is_empty(),
            "tệp bị ignore không thuộc nhóm nào của WORK-01, được: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
        assert!(
            st.is_clean(),
            "repo chỉ có tệp bị ignore phải là SẠCH — nếu không hàng WIP không bao giờ biến mất"
        );
    }

    /// Dạng `!` xen giữa hai bản ghi thật: bỏ nó mà không lệch nấc.
    #[test]
    fn dang_ignored_xen_giua_khong_lam_lech_nac() {
        let vao: &[u8] = b"? truoc.txt\0! bi_ignore.txt\0? sau.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 2);
        assert_eq!(st.entries[0].path, "truoc.txt");
        assert_eq!(
            st.entries[1].path, "sau.txt",
            "bản ghi sau một dạng `!` bị bỏ vẫn phải đúng"
        );
    }

    /// Đầu vào **rỗng** (repo sạch) → `RepoStatus` rỗng, `is_clean()`, không panic.
    #[test]
    fn dau_vao_rong_khong_panic() {
        let st = parse_status(b"");

        assert!(st.is_clean());
        assert_eq!(st.entries.len(), 0);
        assert_eq!(st.branch, BranchInfo::default());
        assert!(!st.has_conflicts);
    }

    /// 🔴 Đột biến M4: đầu vào **cắt ngang** giữa một bản ghi dạng `2`.
    ///
    /// Bản ghi dạng `2` có đường dẫn mới nhưng **thiếu** đường dẫn cũ (lệnh bị hạn giờ
    /// hoặc bị giết giữa đường — ca thật). Phải bỏ bản ghi dở, không panic, và không
    /// đi ăn mất bản ghi khác.
    #[test]
    fn dau_vao_cat_ngang_giua_dang_2_khong_panic() {
        let vao: &[u8] =
            b"1 M. N... 100644 100644 100644 aaa bbb dau.txt\0\
            2 R. N... 100644 100644 100644 ccc ddd R100 moi.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            1,
            "bản ghi dạng 2 thiếu đường dẫn cũ phải bị BỎ, được: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
        assert_eq!(st.entries[0].path, "dau.txt");
    }

    /// Đầu vào cắt ngang giữa một bản ghi dạng `1` (thiếu cả NUL cuối).
    #[test]
    fn dau_vao_cat_ngang_giua_dang_1_khong_panic() {
        let vao: &[u8] = b"? xong.txt\0 1 M. N... 100644 100";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 1);
        assert_eq!(st.entries[0].path, "xong.txt");
    }

    /// 🔴 Đột biến M4 (phần hai): một đường dẫn **tên `2 ...`** không được đọc thành
    /// bản ghi đổi tên.
    ///
    /// Phép nhận dạng là "byte đầu **cộng một khoảng trắng**". Chỉ xem byte đầu thì
    /// đoạn mồ côi này bị coi là một bản ghi dạng `2`, bộ phân tích đi đọc thêm một đoạn
    /// NUL cho một "đường dẫn cũ" không tồn tại, và **ăn mất** bản ghi kế tiếp.
    #[test]
    fn doan_bat_dau_bang_2_nhung_khong_phai_dang_2_khong_an_ban_ghi_sau() {
        // Một đoạn mồ côi `2 ` giả (ví dụ phần đuôi của một bản ghi bị cắt đầu) rồi hai
        // bản ghi thật. Nếu phép nhận dạng đúng thì cả hai bản ghi sau vẫn còn.
        let vao: &[u8] = b"2abc.txt\0? mot.txt\0? hai.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            2,
            "đoạn `2abc.txt` không có khoảng trắng sau `2` nên KHÔNG phải bản ghi dạng 2; \
             coi nó là dạng 2 sẽ ăn mất `? mot.txt`. Được: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
        assert_eq!(st.entries[0].path, "mot.txt");
        assert_eq!(st.entries[1].path, "hai.txt");
    }

    /// Tệp XY = `MM` sinh **hai** phần tử (một mỗi nhóm) nhưng `wip_counts` đếm **một**.
    ///
    /// Nối Task 1 với Task 2: phép phân nhóm ở đây là thứ sinh ra hai phần tử, và
    /// `wip_counts` của domain là thứ phải chịu được điều đó.
    #[test]
    fn xy_mm_sinh_hai_phan_tu_nhung_wip_counts_dem_mot() {
        let vao: &[u8] = b"1 MM N... 100644 100644 100644 aaa bbb ca_hai.txt\0";

        let st = parse_status(vao);

        assert_eq!(
            st.entries.len(),
            2,
            "XY=MM xuất hiện ở CẢ HAI nhóm trên giao diện (khuôn GitHub Desktop)"
        );
        assert_eq!(st.entries[0].group, StatusGroup::Staged);
        assert_eq!(st.entries[1].group, StatusGroup::Unstaged);
        assert_eq!(st.entries[0].path, st.entries[1].path);

        assert_eq!(
            st.wip_counts().modified,
            1,
            "hai phần tử nhưng MỘT tệp sửa — wip_counts đếm đường dẫn duy nhất"
        );
    }

    /// Phân nhóm từ XY: `M.` chỉ đã stage, `.M` chỉ chưa stage, `?` là chưa theo dõi.
    #[test]
    fn phan_nhom_tu_xy_dung_cho_ba_nhom() {
        let vao: &[u8] = b"1 M. N... 100644 100644 100644 aaa bbb chi_stage.txt\0\
            1 .M N... 100644 100644 100644 aaa bbb chi_worktree.txt\0\
            ? chua_theo_doi.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 3);
        assert_eq!(st.entries[0].group, StatusGroup::Staged);
        assert_eq!(st.entries[1].group, StatusGroup::Unstaged);
        assert_eq!(st.entries[2].group, StatusGroup::Untracked);
    }

    /// Tệp xoá ở worktree (`.D`) và tệp xoá đã stage (`D.`) vào đúng nhóm.
    #[test]
    fn tep_xoa_vao_dung_nhom() {
        let vao: &[u8] = b"1 D. N... 100644 000000 000000 aaa 0000 xoa_da_stage.txt\0\
            1 .D N... 100644 100644 000000 aaa bbb xoa_o_worktree.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.entries.len(), 2);
        assert_eq!(st.entries[0].group, StatusGroup::Staged);
        assert_eq!(st.entries[0].xy, "D.");
        assert_eq!(st.entries[1].group, StatusGroup::Unstaged);
    }

    /// Bản ghi dạng `#` lạ (git phiên bản sau thêm dòng mới) không làm hỏng gì.
    #[test]
    fn dong_branch_la_bi_bo_qua_khong_hong() {
        let vao: &[u8] = b"# branch.head main\0# branch.tuong_lai gi_do\0? a.txt\0";

        let st = parse_status(vao);

        assert_eq!(st.branch.head, Some("main".to_owned()));
        assert_eq!(st.entries.len(), 1, "dòng `#` lạ không được ăn bản ghi sau nó");
    }

    /// Đầu ra thật của lệnh, chạy trên repo mẫu `status-cases`.
    ///
    /// # Vì sao cần test này BÊN CẠNH các test byte-literal
    ///
    /// Test byte-literal chứng minh bộ phân tích đúng với byte ta **tưởng** git in. Test
    /// này chứng minh git **thật sự** in đúng byte đó. Phase 3 mất thời gian đúng ở
    /// khoảng cách giữa hai điều đó: `--word-diff-porcelain` hoá ra không tồn tại, và
    /// `--follow` hoá ra đã tự lọc theo path.
    ///
    /// Gọi git trực tiếp qua `std::process::Command` chứ không qua `git/exec.rs`: đây là
    /// test của một hàm **thuần**, và đi qua lớp exec sẽ kéo cả `GitCommand` cùng
    /// `tauri::State` vào một test đơn vị đồng bộ. Mã ứng dụng thật (plan 04-02) thì
    /// **phải** đi qua `git/exec.rs` — xem CONTEXT.md mục 3.3.
    #[test]
    fn doc_duoc_dau_ra_that_tu_repo_mau() {
        let Some(repo) = crate::testing::require_status_fixture() else {
            return; // đã in lời nhắc, không phải lỗi
        };

        let ra = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(crate::domain::STATUS_ARGS)
            .output()
            .expect("chạy được git status");

        assert!(
            ra.status.success(),
            "git status thất bại: {}",
            String::from_utf8_lossy(&ra.stderr)
        );

        let st = parse_status(&ra.stdout);

        // Tiền đề của test: fixture PHẢI có một bản ghi đổi tên. Không có nó thì test
        // này không kiểm được gì và phải ĐỎ, không phải xanh — quy tắc chống cổng #4
        // của CONTEXT.md mục 3.1.
        let doi_ten: Vec<&StatusEntry> =
            st.entries.iter().filter(|e| e.old_path.is_some()).collect();

        assert_eq!(
            doi_ten.len(),
            1,
            "repo mẫu phải có ĐÚNG một bản ghi đổi tên; được {}. \
             Sinh lại bằng: {}",
            doi_ten.len(),
            crate::testing::STATUS_FIXTURES_COMMAND
        );
        assert_eq!(doi_ten[0].path, "a_new.txt");
        assert_eq!(doi_ten[0].old_path, Some("a_old.txt".to_owned()));

        // Tiền đề thứ hai: phải có >= 2 bản ghi ĐỨNG SAU bản ghi đổi tên, nếu không
        // phép kiểm lệch nấc dưới đây không phân biệt được gì.
        let vi_tri = st
            .entries
            .iter()
            .position(|e| e.old_path.is_some())
            .expect("vừa khẳng định có bản ghi đổi tên");
        let so_sau = st.entries.len() - vi_tri - 1;
        assert!(
            so_sau >= 2,
            "cần >= 2 bản ghi SAU bản ghi đổi tên để phân biệt lệch nấc, có {so_sau}. \
             Sinh lại fixture bằng: {}",
            crate::testing::STATUS_FIXTURES_COMMAND
        );

        // Hai tệp sửa đã stage phải đọc ra đúng, và KHÔNG có bản ghi nào mang đường dẫn
        // `a_old.txt` như một phần tử riêng — đó là dấu hiệu lệch nấc.
        assert!(
            st.entries.iter().any(|e| e.path == "m_one.txt"),
            "thiếu m_one.txt: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );
        assert!(
            st.entries.iter().any(|e| e.path == "n_two.txt"),
            "thiếu n_two.txt"
        );
        assert!(
            !st.entries.iter().any(|e| e.path == "a_old.txt"),
            "`a_old.txt` xuất hiện như một PHẦN TỬ RIÊNG — đó đúng là lệch nấc: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );

        // Tên tệp có khoảng trắng phải nguyên vẹn trên đầu ra THẬT, không chỉ trên
        // byte tự dựng.
        assert!(
            st.entries.iter().any(|e| e.path == "z with space.txt"),
            "tên tệp có khoảng trắng bị cắt trên đầu ra thật: {:?}",
            st.entries.iter().map(|e| &e.path).collect::<Vec<_>>()
        );

        // Repo mẫu không có upstream, nên ahead/behind phải là None — xác nhận trên
        // đầu ra thật rằng git đúng là KHÔNG in `# branch.ab +0 -0`.
        assert_eq!(
            st.branch.ahead, None,
            "repo mẫu không có upstream: git phải KHÔNG in dòng branch.ab"
        );
        assert_eq!(st.branch.behind, None);
        assert_eq!(st.branch.head, Some("main".to_owned()));
        assert!(st.branch.oid.is_some(), "repo mẫu có commit nên phải có OID");

        assert!(!st.is_clean(), "repo mẫu cố tình BẨN");
        assert!(!st.has_conflicts, "repo mẫu không có xung đột");
    }

    /// Số đếm WIP trên repo mẫu **thật**, từ **cùng** một `RepoStatus` — WORK-11.
    ///
    /// Không có lệnh git thứ hai nào ở đây; đó là cả điểm của ràng buộc.
    #[test]
    fn wip_counts_tren_repo_mau_that() {
        let Some(repo) = crate::testing::require_status_fixture() else {
            return;
        };

        let ra = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(crate::domain::STATUS_ARGS)
            .output()
            .expect("chạy được git status");

        let st = parse_status(&ra.stdout);
        let dem = st.wip_counts();

        // Fixture: 1 đổi tên + 2 tệp sửa = 3 tệp sửa; 3 tệp chưa theo dõi
        // (z_untracked.txt, `z with space.txt`, và tệp tên Latin-1 mà trên
        // Windows/NTFS thành `b_café.txt`). Khẳng định theo cách đếm được từ chính
        // `entries` để test không vỡ khi script thêm một tệp.
        let so_chua_theo_doi = st
            .entries
            .iter()
            .filter(|e| e.group == StatusGroup::Untracked)
            .count() as u32;

        assert_eq!(
            dem.added, so_chua_theo_doi,
            "repo mẫu không có tệp `A` đã stage, nên `added` phải bằng số tệp chưa theo dõi"
        );
        assert_eq!(
            dem.modified, 3,
            "1 đổi tên + 2 tệp sửa = 3 tệp sửa. Được {}: {:?}",
            dem.modified,
            st.entries.iter().map(|e| (&e.path, &e.xy)).collect::<Vec<_>>()
        );
    }
}
