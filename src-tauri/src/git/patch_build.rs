//! Tách một bản vá nhiều hunk thành bản vá chứa đúng **một** hunk — WORK-03, WORK-04.
//!
//! # Toàn bộ module này chạy trên `&[u8]`, và đó là cả điểm của nó
//!
//! Bản vá mang **nội dung tệp của người dùng**. Một tệp Latin-1, một tệp có BOM, một
//! tệp nhị phân lọt vào đường text — tất cả đều là dữ liệu thật trên máy người dùng,
//! không phải ca giả định. Giải mã thành `String` ở **bất kỳ** điểm nào trên đường từ
//! `git diff` tới `git apply` sẽ thay mọi byte không hợp lệ UTF-8 bằng `U+FFFD`
//! (`EF BF BD`), và git sẽ ghi **ba byte đó** vào tệp của người dùng.
//!
//! Đó không phải một lỗi hiển thị. Đó là **làm hỏng dữ liệu**, im lặng, ở đúng phase
//! đầu tiên ghi vào nội dung tệp chứ không chỉ vào index.
//!
//! Phase 3 và Phase 4 đã trả giá hai lần cho việc giải mã sớm (`GitCommand::arg` gọi
//! `to_string_lossy`, nên thông điệp commit không UTF-8 không đi qua được nguyên vẹn).
//! Ở đó hậu quả là một thông điệp hiển thị sai; ở đây là một tệp hỏng.
//!
//! Vì vậy có một **cổng đọc mã nguồn** ở cuối tệp này khẳng định thân module không
//! chứa `from_utf8_lossy`, `to_string_lossy` hay `String::from_utf8`. Cổng đó bỏ chú
//! thích trước khi tìm — nếu không nó sẽ khớp chính đoạn văn xuôi bạn đang đọc.
//!
//! # Module này KHÔNG gọi git
//!
//! Nó nhận byte đã có và trả byte. Mọi lời gọi git đi qua `git/exec.rs` (bài học
//! `GIT_EXTERNAL_DIFF` của Phase 1: `env("GIT_EXTERNAL_DIFF", "")` làm git spawn một
//! chương trình tên `""` và **mọi** lệnh sinh bản vá thoát 128 với stdout rỗng, im
//! lặng). Việc nối vào IPC là của plan 05-02.

use crate::error::{GitError, Result};

/// Một bản vá đã tách thành phần đầu tệp và các hunk, **chưa giải mã**.
///
/// Không impl `Serialize`: đây là kiểu **nội bộ tầng Rust**. Đưa `Vec<u8>` qua IPC
/// dưới dạng JSON sẽ biến nó thành mảng số, và ai đó sẽ "tiện tay" đổi sang `String`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchTho {
    /// Mọi dòng **trước** `@@` đầu tiên.
    ///
    /// Gồm `diff --git`, `index`, `---`, `+++`, và cả `old mode`/`new mode`/
    /// `rename from`/`rename to`/`similarity index` khi có. Giữ **nguyên văn**: git
    /// đọc lại chính những dòng này để biết bản vá nói về tệp nào và ở chế độ gì.
    pub dau: Vec<u8>,

    /// Thân từng hunk, **kể cả** dòng `@@` của chính nó.
    ///
    /// Dòng `\ No newline at end of file` thuộc về hunk **chứa** nó, không tách ra.
    pub hunks: Vec<Vec<u8>>,
}

impl PatchTho {
    /// Số hunk đọc được.
    pub fn so_hunk(&self) -> usize {
        self.hunks.len()
    }
}

/// Dấu hiệu mở đầu một header hunk.
///
/// 🔴 Phải so khớp ở **đầu dòng**, không phải "có chứa". Một dòng **nội dung** hoàn
/// toàn hợp lệ có thể là ` @@ ...` (dấu cách ngữ cảnh + văn bản), `+@@ ...` hay
/// `-@@ ...` — ví dụ khi người dùng sửa chính một tệp `.patch`, hay một tệp Markdown
/// nói về diff. Dùng `contains` ở đây sẽ đọc dòng nội dung thành header, cắt hunk sai
/// chỗ, và sinh ra một bản vá git từ chối — hoặc tệ hơn, git **chấp nhận** và áp sai.
const DAU_HUNK: &[u8] = b"@@";

/// Chia byte thô của `git diff` thành phần đầu tệp và các hunk.
///
/// # Quy tắc tách
///
/// Quét từng dòng; một dòng bắt đầu bằng `@@` mở một hunk mới. Trước hunk đầu tiên mọi
/// thứ vào [`PatchTho::dau`]; sau đó mọi dòng vào hunk đang mở.
///
/// # 🔴 Byte kết dòng giữ NGUYÊN
///
/// Nếu dòng cuối của đầu vào **không** có `\n` thì đầu ra cũng **không** được thêm.
/// `git apply` nhạy với điều này ở ca `\ No newline at end of file`: thêm một `\n`
/// nghĩa là nói với git rằng tệp *có* dòng trống cuối, và git sẽ ghi thêm một byte vào
/// tệp người dùng.
///
/// Cũng **không** `trim` gì: một dòng nội dung kết thúc bằng `\r` (tệp CRLF chưa qua
/// clean filter) phải giữ `\r`. `trim_end()` ở đây là một cách làm hỏng dữ liệu đúng
/// nghĩa — xem test `crlf_khong_bi_trim`.
///
/// # Bản vá rỗng
///
/// Tệp không đổi cho bản vá rỗng. Trả [`PatchTho`] với `hunks` rỗng, **không** lỗi:
/// "không có gì để tách" là một câu trả lời hợp lệ, không phải một thất bại.
///
/// # Lỗi
///
/// Hiện không có đường nào trả `Err`; kiểu trả về giữ `Result` vì plan 05-02 sẽ thêm
/// phép từ chối bản vá nhị phân (R7) vào đúng chỗ này, và đổi chữ ký của một hàm đã có
/// người gọi thì đắt hơn là để sẵn.
pub fn tach_hunk_tho(patch: &[u8]) -> Result<PatchTho> {
    let mut dau: Vec<u8> = Vec::new();
    let mut hunks: Vec<Vec<u8>> = Vec::new();

    for dong in cac_dong(patch) {
        if dong.starts_with(DAU_HUNK) {
            // Mở hunk mới. Dòng `@@` thuộc về hunk của chính nó.
            hunks.push(dong.to_vec());
        } else if let Some(hien_tai) = hunks.last_mut() {
            hien_tai.extend_from_slice(dong);
        } else {
            dau.extend_from_slice(dong);
        }
    }

    Ok(PatchTho { dau, hunks })
}

/// Cắt `patch` thành các dòng, **giữ nguyên** byte kết dòng của mỗi dòng.
///
/// Khác `slice::split(|b| *b == b'\n')` ở hai điểm quan trọng:
///
/// 1. `split` **vứt bỏ** dấu phân tách, nên người gọi phải tự nối `\n` lại — và sẽ nối
///    thừa một cái vào dòng cuối khi đầu vào không kết thúc bằng `\n`. Đó đúng là ca
///    `\ No newline at end of file`.
/// 2. `split` sinh một lát **rỗng** ở cuối khi đầu vào kết thúc bằng `\n`, mà người
///    gọi phải nhớ bỏ qua.
///
/// Dùng [`memchr::memchr_iter`] để quét `\n`: `CLAUDE.md` chốt `memchr` cho đúng việc
/// này (quét SIMD trên vùng đệm lớn), và một bản vá của tệp vài chục nghìn dòng là
/// hàng megabyte.
fn cac_dong(patch: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut bat_dau = 0usize;
    let mut moc = memchr::memchr_iter(b'\n', patch);
    let mut xong = false;

    std::iter::from_fn(move || {
        if xong {
            return None;
        }
        match moc.next() {
            Some(i) => {
                // Lấy CẢ `\n`: `i + 1`, không phải `i`.
                let dong = &patch[bat_dau..=i];
                bat_dau = i + 1;
                Some(dong)
            }
            None => {
                xong = true;
                if bat_dau < patch.len() {
                    // Dòng cuối KHÔNG có `\n`. Trả nó nguyên vẹn, không thêm gì.
                    Some(&patch[bat_dau..])
                } else {
                    None
                }
            }
        }
    })
}

/// Dựng một bản vá chứa **đúng một** hunk: phần đầu tệp nối với hunk thứ `chi_so`.
///
/// # 🔴 `None` khi chỉ số vượt biên, KHÔNG panic — T-05-01
///
/// `chi_so` đến từ webview, tức từ JavaScript, tức là **số tuỳ ý**. Một chỉ số sai
/// không phải một giả định phòng xa: giao diện có thể gửi chỉ số của một bản vá đã cũ
/// sau khi tệp đổi trên đĩa (đúng ca R4/WORK-05). Index trực tiếp `hunks[chi_so]` sẽ
/// panic, và panic trong một Tauri command làm chết cả tiến trình backend.
///
/// # Vì sao phải nối `dau` chứ không chỉ trả hunk
///
/// Không có `diff --git` / `---` / `+++`, git không biết bản vá nói về tệp nào và từ
/// chối với *"error: no such file"*. Xem đột biến M3.
pub fn dung_ban_va_mot_hunk(p: &PatchTho, chi_so: usize) -> Option<Vec<u8>> {
    let hunk = p.hunks.get(chi_so)?;

    let mut ra = Vec::with_capacity(p.dau.len() + hunk.len());
    ra.extend_from_slice(&p.dau);
    ra.extend_from_slice(hunk);
    Some(ra)
}

/// Dựng một bản vá chứa **một tập con** các hunk, theo thứ tự tăng dần của chỉ số.
///
/// Trả `Err` khi **bất kỳ** chỉ số nào vượt biên: khác [`dung_ban_va_mot_hunk`] ở chỗ
/// bỏ qua im lặng một chỉ số sai nghĩa là người dùng chọn bốn khối mà chỉ ba khối được
/// đưa vào vùng chờ — và không có gì nói cho họ biết.
///
/// # 🔴 Bản vá này CẦN `--recount`
///
/// Số đếm trong header `@@` là **của riêng từng hunk**, không tích luỹ, nên chép
/// nguyên văn header của một hunk vào bản vá con thì nó **vẫn đúng** — đã đo, và đó là
/// lý do một fixture "chọn hunk thứ hai" **không** phân biệt được việc bỏ `--recount`.
///
/// Thứ làm header sai là **cắt bớt dòng thân** trong khi giữ số đếm cũ. Hàm này
/// **không** cắt thân, nên về lý thuyết bản vá của nó vẫn đúng số đếm. Nhưng
/// `--recount` vẫn là bắt buộc ở chỗ gọi `git apply`, vì nó là thứ duy nhất chịu được
/// mọi biến thể tương lai của việc dựng tập con — và cái giá của nó là 0.
pub fn dung_ban_va_nhieu_hunk(p: &PatchTho, chi_so: &[usize]) -> Result<Vec<u8>> {
    let mut sap_xep: Vec<usize> = chi_so.to_vec();
    sap_xep.sort_unstable();
    sap_xep.dedup();

    if let Some(qua) = sap_xep.iter().find(|i| **i >= p.hunks.len()) {
        return Err(GitError::ParseFailed(format!(
            "chỉ số hunk {qua} vượt biên: bản vá chỉ có {} hunk",
            p.hunks.len()
        )));
    }

    let mut ra = Vec::new();
    ra.extend_from_slice(&p.dau);
    for i in sap_xep {
        ra.extend_from_slice(&p.hunks[i]);
    }
    Ok(ra)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Byte `0xE9` — `é` trong Latin-1, **không** hợp lệ UTF-8 khi đứng một mình.
    ///
    /// Đi qua `String::from_utf8_lossy` nó thành `EF BF BD` (U+FFFD), dài **ba** byte
    /// thay vì một. Đó là dấu vết quan sát được mà mọi test dưới đây săn.
    const E9: u8 = 0xE9;

    /// Bản vá ba hunk tự dựng bằng byte literal.
    ///
    /// 🔴 Dựng bằng `b"..."` + `extend_from_slice`, **không** bằng `format!`:
    /// `format!` nhận `&str` và `&str` **không biểu diễn được** byte `0xE9` đơn lẻ.
    /// Một fixture dựng bằng `format!` sẽ trông đúng hình dạng và kiểm được đúng 0
    /// điều về việc giữ byte thô — đúng lớp "hình dạng đúng, dữ liệu vô hại".
    fn ba_hunk() -> Vec<u8> {
        let mut p = Vec::new();
        p.extend_from_slice(b"diff --git a/f.txt b/f.txt\n");
        p.extend_from_slice(b"index 0edb856..e69f8bd 100644\n");
        p.extend_from_slice(b"--- a/f.txt\n");
        p.extend_from_slice(b"+++ b/f.txt\n");
        p.extend_from_slice(b"@@ -1,5 +1,5 @@\n");
        p.extend_from_slice(b" a\n-b\n+M1\n c\n d\n e\n");
        p.extend_from_slice(b"@@ -10,7 +10,7 @@ i\n");
        p.extend_from_slice(b" j\n k\n l\n-m\n+M2\n n\n o\n p\n");
        p.extend_from_slice(b"@@ -22,5 +22,5 @@ u\n");
        p.extend_from_slice(b" v\n w\n x\n-y\n+M3\n z\n");
        p
    }

    // -- Test 1 ---------------------------------------------------------------

    /// Bản vá ba hunk cho đúng **ba** hunk, và phần đầu chứa `diff --git` nhưng
    /// **không** chứa `@@`.
    ///
    /// # Dòng nội dung chứa `@@` phải KHÔNG bị đọc thành header — đột biến M2
    ///
    /// Bản vá dưới đây có một dòng **ngữ cảnh** ` @@ khong phai header` và một dòng
    /// **thêm** `+@@ cung khong phai`. Cả hai là nội dung tệp hợp lệ (người dùng đang
    /// sửa một tệp nói về diff). Một cài đặt nhận hunk bằng `contains(b"@@")` thay vì
    /// `starts_with` sẽ đọc chúng thành header và cho **bốn** hunk thay vì hai.
    ///
    /// 🔴 Đây là dữ liệu **có dấu vết quan sát được**, không phải hình dạng suông: nếu
    /// bản vá chỉ chứa các dòng nội dung vô hại thì `contains` và `starts_with` cho
    /// kết quả **giống hệt nhau** và test không nói gì về đột biến M2.
    #[test]
    fn tach_dung_so_hunk_va_khong_nham_dong_noi_dung() {
        let p = tach_hunk_tho(&ba_hunk()).expect("bản vá hợp lệ");

        assert_eq!(p.so_hunk(), 3, "bản vá ba hunk phải cho đúng 3 hunk");
        assert!(
            p.dau.starts_with(b"diff --git"),
            "phần đầu phải giữ dòng `diff --git`; không có nó git không biết bản vá \
             nói về tệp nào"
        );
        assert!(
            !p.dau.windows(2).any(|w| w == b"@@"),
            "phần đầu KHÔNG được chứa `@@`: nó dừng ngay trước header hunk đầu tiên"
        );

        // Ca chịu lực: dòng nội dung bắt đầu bằng `@@` sau một dấu cách hoặc dấu `+`.
        let mut co_bay = Vec::new();
        co_bay.extend_from_slice(b"diff --git a/p.md b/p.md\n");
        co_bay.extend_from_slice(b"--- a/p.md\n+++ b/p.md\n");
        co_bay.extend_from_slice(b"@@ -1,3 +1,4 @@\n");
        co_bay.extend_from_slice(b" @@ khong phai header\n");
        co_bay.extend_from_slice(b"+@@ cung khong phai\n");
        co_bay.extend_from_slice(b" con lai\n");
        co_bay.extend_from_slice(b"@@ -20,2 +21,2 @@\n");
        co_bay.extend_from_slice(b"-cu\n+moi\n");

        let q = tach_hunk_tho(&co_bay).expect("bản vá hợp lệ");
        assert_eq!(
            q.so_hunk(),
            2,
            "chỉ hai dòng bắt đầu bằng `@@` Ở ĐẦU DÒNG là header. Đọc được {} hunk \
             nghĩa là một dòng NỘI DUNG chứa `@@` đã bị nhận nhầm — bản vá sinh ra sẽ \
             cắt sai chỗ và git áp sai vị trí (đột biến M2)",
            q.so_hunk()
        );
        assert!(
            q.hunks[0].windows(21).any(|w| w == b" @@ khong phai header"),
            "dòng nội dung ` @@ ...` phải NẰM TRONG hunk 1, không thành hunk riêng"
        );
    }

    // -- Test 2 ---------------------------------------------------------------

    /// `dung_ban_va_mot_hunk(p, 1)` cho bản vá chỉ chứa hunk **thứ hai**, và bản vá đó
    /// giữ nguyên phần đầu tệp.
    ///
    /// # Phần đầu là bắt buộc — đột biến M3
    ///
    /// Bỏ `dau` cho một bản vá không có `diff --git`/`---`/`+++`, và git từ chối với
    /// *"error: no such file"*. Test đòi cả ba mốc, không chỉ đòi "có một hunk".
    #[test]
    fn dung_ban_va_mot_hunk_lay_dung_hunk_va_giu_phan_dau() {
        let p = tach_hunk_tho(&ba_hunk()).expect("bản vá hợp lệ");
        let ra = dung_ban_va_mot_hunk(&p, 1).expect("chỉ số 1 nằm trong biên");

        let so_header = ra
            .windows(3)
            .filter(|w| *w == b"@@ ")
            .count();
        assert_eq!(
            so_header, 2,
            "bản vá một hunk có ĐÚNG hai mốc `@@ `: mở và đóng header của cùng một dòng"
        );

        assert!(
            ra.starts_with(b"diff --git"),
            "thiếu `diff --git` thì git không biết bản vá nói về tệp nào (đột biến M3)"
        );
        assert!(
            ra.windows(11).any(|w| w == b"--- a/f.txt"),
            "thiếu dòng `---` thì git từ chối bản vá (đột biến M3)"
        );
        assert!(
            ra.windows(11).any(|w| w == b"+++ b/f.txt"),
            "thiếu dòng `+++` thì git từ chối bản vá (đột biến M3)"
        );

        // Lấy ĐÚNG hunk giữa, không phải hunk 1 hay hunk 3.
        assert!(
            ra.windows(3).any(|w| w == b"+M2"),
            "phải chứa nội dung của hunk THỨ HAI (`+M2`)"
        );
        assert!(
            !ra.windows(3).any(|w| w == b"+M1"),
            "KHÔNG được chứa nội dung hunk 1 (`+M1`)"
        );
        assert!(
            !ra.windows(3).any(|w| w == b"+M3"),
            "KHÔNG được chứa nội dung hunk 3 (`+M3`)"
        );
    }

    // -- Test 3 — TEST QUAN TRỌNG NHẤT CỦA PLAN -------------------------------

    /// 🔴 **Byte không UTF-8 đi qua đường dựng bản vá NGUYÊN VẸN.**
    ///
    /// Đây là test chịu lực của cả WORK-04. Một cài đặt lỡ đi qua
    /// `String::from_utf8_lossy` ở **bất kỳ** đâu trên đường sẽ đổi byte `0xE9` thành
    /// `EF BF BD` (U+FFFD), và git sẽ ghi **ba byte đó** vào tệp của người dùng thay
    /// cho một byte gốc. Tệp hỏng, im lặng.
    ///
    /// Test khẳng định **cả hai chiều**, và chiều thứ hai mới là chiều bắt được lỗi:
    ///
    /// 1. byte `0xE9` **còn** trong đầu ra — bắt ca "bị thay/bị bỏ";
    /// 2. chuỗi `EF BF BD` **không** có trong đầu ra — bắt ca "bị thay bằng U+FFFD".
    ///
    /// Chỉ khẳng định (1) là chưa đủ nếu một ngày nào đó có cả byte gốc lẫn bản thay
    /// thế; chỉ khẳng định (2) là chưa đủ vì xoá sạch byte cũng cho 0 khớp U+FFFD.
    #[test]
    fn byte_khong_utf8_di_qua_nguyen_ven() {
        let mut patch = Vec::new();
        patch.extend_from_slice(b"diff --git a/caf.txt b/caf.txt\n");
        patch.extend_from_slice(b"--- a/caf.txt\n+++ b/caf.txt\n");
        patch.extend_from_slice(b"@@ -1,3 +1,3 @@\n");
        patch.extend_from_slice(b" dong mot\n");
        patch.extend_from_slice(b"-caf");
        patch.push(E9); // byte thô, GIỮA một dòng nội dung
        patch.extend_from_slice(b" cu\n");
        patch.extend_from_slice(b"+caf");
        patch.push(E9);
        patch.extend_from_slice(b" moi\n");
        patch.extend_from_slice(b" dong ba\n");

        let so_e9_vao = patch.iter().filter(|b| **b == E9).count();
        assert_eq!(so_e9_vao, 2, "tiền đề: đầu vào phải có đúng hai byte 0xE9");

        let p = tach_hunk_tho(&patch).expect("bản vá hợp lệ");
        let ra = dung_ban_va_mot_hunk(&p, 0).expect("chỉ số 0 nằm trong biên");

        let so_e9_ra = ra.iter().filter(|b| **b == E9).count();
        assert_eq!(
            so_e9_ra, 2,
            "byte 0xE9 phải đi qua NGUYÊN VẸN: vào 2, ra {so_e9_ra}. Mất nó nghĩa là \
             đường bản vá đã giải mã qua String ở đâu đó — WORK-04 đổ ở đây, và hậu \
             quả là tệp người dùng bị ghi sai byte"
        );

        assert!(
            !ra.windows(3).any(|w| w == [0xEF, 0xBF, 0xBD]),
            "thấy U+FFFD (EF BF BD) trong đầu ra: một `from_utf8_lossy` đã thay byte \
             0xE9 bằng ký tự thay thế. Đây đúng đột biến M1"
        );
    }

    // -- Test 4 ---------------------------------------------------------------

    /// Dòng `\ No newline at end of file` thuộc về hunk **chứa** nó.
    ///
    /// Không rơi vào `dau` (nó đứng sau một header `@@`) và không bị bỏ. Bỏ nó nghĩa
    /// là nói với git rằng tệp *có* dòng trống cuối, và git sẽ **thêm một byte** vào
    /// tệp người dùng — đúng ca WORK-04 "thiếu dòng trống cuối giữ nguyên".
    #[test]
    fn dau_khong_co_dong_cuoi_nam_trong_hunk_chua_no() {
        let mut patch = Vec::new();
        patch.extend_from_slice(b"diff --git a/f.txt b/f.txt\n");
        patch.extend_from_slice(b"--- a/f.txt\n+++ b/f.txt\n");
        patch.extend_from_slice(b"@@ -1,3 +1,3 @@\n");
        patch.extend_from_slice(b" a\n-b\n+B\n c\n");
        patch.extend_from_slice(b"@@ -26,5 +26,5 @@\n");
        patch.extend_from_slice(b" x\n-y\n+Y\n z\n");
        patch.extend_from_slice(b"\\ No newline at end of file\n");

        let p = tach_hunk_tho(&patch).expect("bản vá hợp lệ");
        assert_eq!(p.so_hunk(), 2, "tiền đề: bản vá này có hai hunk");

        let dau_hieu = b"\\ No newline at end of file";

        assert!(
            !p.dau.windows(dau_hieu.len()).any(|w| w == dau_hieu),
            "dấu `\\ No newline` KHÔNG được rơi vào phần đầu tệp"
        );

        let so_trong_hunk: usize = p
            .hunks
            .iter()
            .map(|h| h.windows(dau_hieu.len()).filter(|w| *w == dau_hieu).count())
            .sum();
        assert_eq!(
            so_trong_hunk, 1,
            "dấu `\\ No newline` phải có ĐÚNG MỘT lần trong các hunk. Đếm được \
             {so_trong_hunk}: 0 nghĩa là đã bị BỎ (đột biến M4) và git sẽ thêm một \
             byte `\\n` vào tệp người dùng"
        );

        assert!(
            p.hunks[1].windows(dau_hieu.len()).any(|w| w == dau_hieu),
            "dấu phải nằm trong hunk CUỐI (hunk chứa nó), không phải hunk đầu"
        );

        // Và nó phải đi theo hunk đó khi dựng bản vá con.
        let ra = dung_ban_va_mot_hunk(&p, 1).expect("chỉ số 1 nằm trong biên");
        assert!(
            ra.windows(dau_hieu.len()).any(|w| w == dau_hieu),
            "bản vá của hunk cuối phải MANG THEO dấu `\\ No newline`"
        );
    }

    // -- Test 5 ---------------------------------------------------------------

    /// Bản vá **rỗng** (tệp không đổi) cho 0 hunk, không panic.
    ///
    /// Đường này có thật: `git diff` của một tệp không đổi trả stdout rỗng, và giao
    /// diện vẫn có thể hỏi nó (người dùng bấm vào một tệp vừa được người khác stage).
    #[test]
    fn ban_va_rong_cho_khong_hunk_va_khong_panic() {
        let p = tach_hunk_tho(b"").expect("bản vá rỗng là hợp lệ, không phải lỗi");
        assert_eq!(p.so_hunk(), 0, "bản vá rỗng có 0 hunk");
        assert!(p.dau.is_empty(), "bản vá rỗng có phần đầu rỗng");

        // Và mọi chỉ số đều ngoài biên -> None, KHÔNG panic (T-05-01).
        assert!(
            dung_ban_va_mot_hunk(&p, 0).is_none(),
            "chỉ số 0 trên bản vá 0 hunk phải cho None"
        );
    }

    // -- Test 6 ---------------------------------------------------------------

    /// 🔴 Dòng nội dung kết thúc bằng `\r` **không** bị trim — đột biến M5.
    ///
    /// # Ca này KHÔNG dựng được từ repo mẫu `hunk-cases`, và đó là lý do nó là test
    /// byte-literal
    ///
    /// Đã đo: với `core.autocrlf=true` (mặc định Windows, và cấu hình của `hunk-cases`),
    /// `git diff` in bản vá bằng **LF thuần** — clean filter bỏ `\r` trước khi git so
    /// sánh. Bản vá từ repo đó có **0** byte `\r`, nên nó không phân biệt được gì về
    /// đột biến M5.
    ///
    /// Bản vá **có** `\r` là có thật: repo đặt `core.autocrlf=false` hoặc
    /// `core.eol=crlf`, hoặc `.gitattributes` gắn `-text` cho tệp. Trong những repo đó
    /// một `trim_end()` sẽ ăn mất `\r` của **mọi** dòng và git ghi lại tệp bằng LF —
    /// tức chuẩn hoá xuống dòng mà người dùng không hề yêu cầu. Đó chính là R2.
    #[test]
    fn crlf_khong_bi_trim() {
        let mut patch = Vec::new();
        patch.extend_from_slice(b"diff --git a/w.txt b/w.txt\n");
        patch.extend_from_slice(b"--- a/w.txt\n+++ b/w.txt\n");
        patch.extend_from_slice(b"@@ -1,3 +1,3 @@\r\n");
        patch.extend_from_slice(b" dong mot\r\n");
        patch.extend_from_slice(b"-dong hai cu\r\n");
        patch.extend_from_slice(b"+dong hai moi\r\n");
        patch.extend_from_slice(b" dong ba\r\n");

        let so_cr_vao = patch.iter().filter(|b| **b == b'\r').count();
        assert_eq!(
            so_cr_vao, 5,
            "tiền đề: đầu vào phải có đúng 5 byte \\r (header + 4 dòng thân)"
        );

        let p = tach_hunk_tho(&patch).expect("bản vá hợp lệ");
        let ra = dung_ban_va_mot_hunk(&p, 0).expect("chỉ số 0 nằm trong biên");

        let so_cr_ra = ra.iter().filter(|b| **b == b'\r').count();
        assert_eq!(
            so_cr_ra, so_cr_vao,
            "số byte \\r phải KHÔNG ĐỔI: vào {so_cr_vao}, ra {so_cr_ra}. Mất chúng \
             nghĩa là một `trim_end()` đã chuẩn hoá xuống dòng của người dùng — R2, \
             và git sẽ ghi lại tệp bằng LF (đột biến M5)"
        );
    }

    // -- Ca biên ---------------------------------------------------------------

    /// Chỉ số vượt biên cho `None`, **không** panic — T-05-01.
    ///
    /// `chi_so` đến từ webview. Một panic trong Tauri command làm chết tiến trình
    /// backend, nên đây là ca bảo mật, không phải chuyện lịch sự.
    #[test]
    fn chi_so_vuot_bien_cho_none_khong_panic() {
        let p = tach_hunk_tho(&ba_hunk()).expect("bản vá hợp lệ");
        assert_eq!(p.so_hunk(), 3, "tiền đề: bản vá có 3 hunk");

        assert!(dung_ban_va_mot_hunk(&p, 2).is_some(), "chỉ số 2 hợp lệ");
        assert!(
            dung_ban_va_mot_hunk(&p, 3).is_none(),
            "chỉ số 3 vượt biên trên bản vá 3 hunk phải cho None"
        );
        assert!(
            dung_ban_va_mot_hunk(&p, usize::MAX).is_none(),
            "chỉ số khổng lồ phải cho None, không panic"
        );
    }

    /// Dòng cuối **không** có `\n` thì đầu ra cũng không được thêm.
    ///
    /// Tách rồi nối lại toàn bộ phải cho **đúng** byte đầu vào. Đây là phép kiểm mạnh
    /// nhất về việc không mất/không thêm byte: nó không phụ thuộc vào việc đoán xem
    /// chỗ nào có thể sai.
    #[test]
    fn tach_roi_noi_lai_cho_dung_byte_dau_vao() {
        for (ten, goc) in [
            ("ba hunk, kết thúc bằng \\n", ba_hunk()),
            ("không kết thúc bằng \\n", {
                let mut v = ba_hunk();
                assert_eq!(v.pop(), Some(b'\n'), "tiền đề: bản gốc kết thúc bằng \\n");
                v
            }),
        ] {
            let p = tach_hunk_tho(&goc).expect("bản vá hợp lệ");

            let mut noi_lai = p.dau.clone();
            for h in &p.hunks {
                noi_lai.extend_from_slice(h);
            }

            assert_eq!(
                noi_lai.len(),
                goc.len(),
                "[{ten}] số byte phải không đổi: vào {}, ra {}",
                goc.len(),
                noi_lai.len()
            );
            assert!(
                noi_lai == goc,
                "[{ten}] tách rồi nối lại phải cho ĐÚNG byte đầu vào"
            );
        }
    }

    /// `dung_ban_va_nhieu_hunk` lấy đúng tập con, sắp xếp, và **báo lỗi** khi chỉ số sai.
    #[test]
    fn dung_ban_va_nhieu_hunk_lay_dung_tap_con() {
        let p = tach_hunk_tho(&ba_hunk()).expect("bản vá hợp lệ");

        // Cố tình đưa chỉ số KHÔNG theo thứ tự và có TRÙNG.
        let ra = dung_ban_va_nhieu_hunk(&p, &[2, 0, 0]).expect("chỉ số hợp lệ");

        assert!(ra.windows(3).any(|w| w == b"+M1"), "phải có hunk 0");
        assert!(ra.windows(3).any(|w| w == b"+M3"), "phải có hunk 2");
        assert!(
            !ra.windows(3).any(|w| w == b"+M2"),
            "KHÔNG được có hunk 1 — nó không được chọn"
        );

        let so_m1 = ra.windows(3).filter(|w| *w == b"+M1").count();
        assert_eq!(so_m1, 1, "chỉ số trùng phải bị gộp, hunk 0 chỉ xuất hiện một lần");

        // Thứ tự phải tăng dần bất kể thứ tự đầu vào: vị trí `+M1` trước `+M3`.
        let vt_m1 = ra.windows(3).position(|w| w == b"+M1").expect("có +M1");
        let vt_m3 = ra.windows(3).position(|w| w == b"+M3").expect("có +M3");
        assert!(
            vt_m1 < vt_m3,
            "hunk phải nằm theo thứ tự tăng dần của chỉ số; git apply đọc bản vá tuần tự"
        );

        assert!(
            dung_ban_va_nhieu_hunk(&p, &[0, 9]).is_err(),
            "chỉ số vượt biên phải báo LỖI, không bỏ qua im lặng: bỏ qua nghĩa là \
             người dùng chọn hai khối mà chỉ một khối được stage, không ai báo"
        );
    }

    // -- Cổng đọc mã nguồn -----------------------------------------------------

    /// Thân module này, đã **bỏ dòng chú thích** và đã **cắt `mod tests`**.
    ///
    /// # Vì sao cả hai phép cắt đều bắt buộc
    ///
    /// **Bỏ chú thích:** cổng thứ **năm** trong chín cổng không-thể-fail của dự án
    /// (CONTEXT.md 4.1) chết đúng vì điều này — nó grep trên nguồn **thô** và khớp một
    /// chú thích **do chính mutation sinh ra**. Doc comment ở đầu tệp này nói *về*
    /// `from_utf8_lossy` để giải thích vì sao **không** được dùng nó, nên một cổng đọc
    /// nguồn thô sẽ khớp chính đoạn văn xuôi đó và ĐỎ vĩnh viễn — hoặc tệ hơn, ai đó sẽ
    /// xoá đoạn văn xuôi để làm nó xanh.
    ///
    /// **Cắt `mod tests`:** test có quyền nhắc tên hàm bị cấm trong chuỗi khẳng định
    /// của chính nó. Không cắt thì cổng ĐỎ vì đọc chính mình.
    fn than_khong_chu_thich() -> String {
        let src = include_str!("patch_build.rs");
        let sach: String = src
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n");

        sach.split("mod tests")
            .next()
            .expect("split luôn cho ít nhất một phần")
            .to_owned()
    }

    /// 🔴 **Đường bản vá không giải mã ở BẤT KỲ đâu.**
    ///
    /// Cổng này tồn tại vì test byte `0xE9` một mình là chưa đủ: nó chỉ phủ đường đi
    /// **hiện tại**. Một hàm mới thêm vào module này ngày mai có thể giải mã mà không
    /// test nào hỏi tới, và đó đúng là **lỗi #9** của CONTEXT.md 4.1 — thứ sống sót
    /// qua cả 666 test vì không ai từng viết test hỏi về nó.
    #[test]
    fn duong_ban_va_khong_bao_gio_giai_ma() {
        let than = than_khong_chu_thich();

        // 🔴 Khẳng định TIỀN ĐỀ trước. Cổng thứ ba của CONTEXT.md 4.1 chết vì đường
        // dẫn tệp sai -> grep không thấy gì -> XANH với mọi mã.
        assert!(
            than.contains("pub fn tach_hunk_tho"),
            "🔴 tiền đề sai: không thấy `pub fn tach_hunk_tho` trong thân đã lọc. \
             Cổng này đang tìm trong một chuỗi rỗng hoặc cắt sai chỗ, và sẽ XANH với \
             MỌI mã. Sửa phép lọc trước khi tin kết quả bên dưới"
        );
        assert!(
            than.contains("pub fn dung_ban_va_mot_hunk"),
            "🔴 tiền đề sai: không thấy `dung_ban_va_mot_hunk` trong thân đã lọc"
        );

        for cam in ["from_utf8_lossy", "to_string_lossy", "String::from_utf8"] {
            assert!(
                !than.contains(cam),
                "🔴 `{cam}` trên đường bản vá — WORK-04 đổ ở đây. Bản vá mang nội dung \
                 tệp của người dùng; giải mã nó thay mọi byte không hợp lệ UTF-8 bằng \
                 U+FFFD và git ghi BA byte đó vào tệp thật. Xem CONTEXT.md 2.1"
            );
        }
    }

    /// Phép lọc chú thích phải thật sự **cắt** được `mod tests`.
    ///
    /// Không có test này thì `than_khong_chu_thich` có thể âm thầm trả cả tệp (ví dụ
    /// khi ai đó đổi tên `mod tests`), và cổng trên sẽ đỏ vì một lý do sai — hoặc, nếu
    /// phép cắt trả chuỗi rỗng, xanh vì một lý do sai. Cả hai đều là cổng hỏng.
    #[test]
    fn phep_loc_nguon_cat_dung_cho() {
        let than = than_khong_chu_thich();
        let toan_bo = include_str!("patch_build.rs");

        assert!(
            than.len() < toan_bo.len(),
            "thân đã lọc phải NGẮN HƠN toàn tệp: lọc không cắt được gì nghĩa là cổng \
             đang đọc cả chú thích lẫn test"
        );
        assert!(
            !than.is_empty(),
            "thân đã lọc KHÔNG được rỗng: một chuỗi rỗng làm cổng XANH với mọi mã"
        );
        assert!(
            !than.contains("fn byte_khong_utf8_di_qua_nguyen_ven"),
            "thân đã lọc không được chứa mã test — phép cắt `mod tests` hỏng"
        );
        assert!(
            toan_bo.contains("from_utf8_lossy"),
            "tiền đề của chính phép lọc: tệp này CÓ nhắc `from_utf8_lossy` trong chú \
             thích và trong test. Không còn thì cổng `duong_ban_va_khong_bao_gio_giai_ma` \
             đang kiểm một thứ không tồn tại và không chứng minh được nó biết ĐỎ"
        );
    }
}
