//! Thùng rác — kiểu miền cho WORK-06 và WORK-07.
//!
//! Struct thuần, không biết gì về git hay Tauri. Lệnh git nằm ở
//! [`crate::commands::trash`].
//!
//! # Vì sao module này tồn tại tách khỏi lệnh git
//!
//! Hai thứ ở đây — [`BienNhan`] và [`dong_tu_cho`] — là **ràng buộc**, không phải dữ
//! liệu. `BienNhan` là một bằng chứng kiểu-hoá rằng bước lưu đã chạy; `dong_tu_cho` là
//! một nguồn sự thật duy nhất cho câu hỏi "động từ hiện cho người dùng là gì". Cả hai
//! kiểm được bằng test đơn vị thuần, không cần repository thật.

use serde::Serialize;

use super::status::StatusGroup;

/// Một mục trong danh sách "Vừa huỷ gần đây".
///
/// # Vì sao `object_id` giữ được **hai** loại object
///
/// Đo được trên git 2.54.0.windows.1: `git stash create` trả về **chuỗi rỗng** khi chỉ
/// có tệp chưa theo dõi thay đổi, và tệp chưa theo dõi **không bao giờ** nằm trong cây
/// của stash commit kể cả khi lời gọi trả về sha (vì một tệp đã theo dõi khác đang bẩn).
/// Nên ca chưa-theo-dõi phải đi đường riêng: `git hash-object -w` cho ra một **blob**.
///
/// Cũng đo được: `git update-ref` chấp nhận trỏ thẳng vào một blob, không chỉ commit —
/// nên cả hai loại neo được dưới **cùng một** không gian ref, không cần dựng commit giả.
/// [`la_blob`](MucThungRac::la_blob) là thứ phân nhánh cách khôi phục.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MucThungRac {
    /// Tên ref đầy đủ, luôn có tiền tố [`TIEN_TO_REF`].
    pub ref_name: String,

    /// SHA của object được giữ: commit (stash) **hoặc** blob (tệp chưa theo dõi).
    pub object_id: String,

    /// `true` khi [`object_id`](MucThungRac::object_id) là một **blob**, tức nội dung
    /// thô của **một** tệp chưa theo dõi. Khôi phục một blob là ghi byte thẳng ra tệp;
    /// khôi phục một commit là `git checkout <ref> -- <paths>`.
    pub la_blob: bool,

    /// Các đường dẫn mà mục này giữ nội dung cho.
    pub paths: Vec<String>,

    /// Giây Unix. Số nguyên chứ không phải chuỗi đã định dạng: `Intl.DateTimeFormat` ở
    /// phía TypeScript lo múi giờ và ngôn ngữ của người dùng đúng hơn bất cứ thứ gì ta
    /// tự ghép ở Rust (quyết định từ CLAUDE.md — không dùng `chrono`).
    pub luc: i64,

    /// Nhãn hiện cho người dùng, ví dụ `"Huỷ bỏ 2 tệp"` hay `"Xoá bar.txt"`.
    pub nhan: String,
}

/// Không gian ref riêng cho thùng rác.
///
/// # 🔴 Vì sao KHÔNG phải `refs/stash` và KHÔNG phải `refs/heads`
///
/// `refs/stash` là stash **thật của người dùng**; ghi đè nó nghĩa là một tính năng
/// chống mất dữ liệu đi phá dữ liệu của người dùng ở chỗ khác. `refs/heads/*` hiện
/// thành **nhánh** trong thanh bên, nên mỗi lần huỷ sẽ đẻ ra một nhánh rác.
///
/// Có cổng ghim: `ref_nam_duoi_dung_khong_gian_rieng`.
pub const TIEN_TO_REF: &str = "refs/git-plum-trash/";

/// Bằng chứng rằng nội dung đã được lưu **xong** và ref đã đọc lại được.
///
/// # 🔴 Đây là một ràng buộc kiểu, không phải một struct tiện lợi
///
/// Trường là **private** và module này **không** cấp hàm dựng công khai. Chỉ
/// [`crate::commands::trash::luu_truoc_khi_huy`] tạo được — nó nằm cùng crate và dùng
/// [`BienNhan::moi`], thứ `pub(crate)`.
///
/// Hệ quả là thứ ta muốn: mọi hàm huỷ **đòi** một `BienNhan` làm tham số, nên một đường
/// huỷ mới viết trong tương lai mà quên bước lưu sẽ **không biên dịch được**. So với
/// việc dựa vào kỷ luật ("nhớ gọi `luu_truoc_khi_huy` trước nhé"), cái này không quên
/// được — và WORK-07 tồn tại đúng vì một lần quên là mất dữ liệu thật của người dùng
/// (rủi ro R3).
///
/// Mối đe doạ T-05-09 được đóng bằng cách này.
#[derive(Debug, Clone)]
pub struct BienNhan {
    muc: MucThungRac,
}

impl BienNhan {
    /// Dựng biên nhận. `pub(crate)` **có chủ ý**: xem ghi chú của [`BienNhan`].
    ///
    /// 🔴 Đừng nới thành `pub`. Nới nghĩa là bất kỳ đâu cũng dựng được một biên nhận
    /// rỗng nghĩa, và ràng buộc kiểu mất **toàn bộ** giá trị — hàm huỷ vẫn biên dịch,
    /// vẫn chạy, và không còn gì bảo đảm nội dung đã được lưu.
    pub(crate) fn moi(muc: MucThungRac) -> Self {
        Self { muc }
    }

    /// Mục thùng rác mà biên nhận này chứng nhận.
    pub fn muc(&self) -> &MucThungRac {
        &self.muc
    }

    /// Lấy lại mục, tiêu thụ biên nhận.
    pub fn vao_muc(self) -> MucThungRac {
        self.muc
    }
}

/// Động từ hiện cho người dùng khi huỷ một tệp thuộc nhóm `group`.
///
/// # 🔴 Vì sao phép chọn này ở Rust chứ không ở giao diện
///
/// ROADMAP: *"Với tệp chưa theo dõi, động từ phải là **Xoá**, không phải Huỷ bỏ."* Để
/// giao diện tự đoán nghĩa là **hai nguồn sự thật** cho cùng một câu hỏi, và nhánh đoán
/// đó không có một test Rust nào — đúng lớp lỗi #9 của `CONTEXT.md` §4.1, nơi một
/// thuộc tính không có test nào hỏi về nó sống sót qua cả suite.
///
/// # Vì sao khác biệt này quan trọng với người dùng
///
/// "Huỷ bỏ" hàm ý *quay lại phiên bản đã lưu* — có một bản gốc ở đâu đó. Với tệp
/// **chưa theo dõi** thì **không có** bản gốc nào: nội dung đó chưa từng vào git, nên
/// thao tác này là **xoá**. Dùng sai động từ là nói sai mức độ nghiêm trọng đúng lúc
/// người dùng đang quyết định có bấm hay không.
pub fn dong_tu_cho(group: StatusGroup) -> DongTu {
    match group {
        StatusGroup::Untracked => DongTu::Xoa,
        StatusGroup::Staged | StatusGroup::Unstaged => DongTu::HuyBo,
    }
}

/// Động từ hiện cho người dùng. Xem [`dong_tu_cho`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DongTu {
    /// Tệp đã theo dõi: có bản gốc trong git để quay về.
    HuyBo,
    /// Tệp chưa theo dõi: nội dung **không có ở đâu khác**.
    Xoa,
}

impl DongTu {
    /// Chuỗi hiển thị, **có dấu**.
    ///
    /// 🔴 `"Xoá"` có dấu sắc trên `o`, không phải `"Xoa"`. Tên biến thể Rust không mang
    /// dấu được (định danh ASCII), nên chuỗi cho người dùng phải là một bước dịch
    /// **tường minh** — nếu không, ai đó sẽ hiện thẳng tên biến thể ra giao diện.
    pub fn nhan(self) -> &'static str {
        match self {
            DongTu::HuyBo => "Huỷ bỏ",
            DongTu::Xoa => "Xoá",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Thân **không-test** của tệp này, đã bỏ dòng chú thích.
    ///
    /// Cùng khuôn với `than_khong_chu_thich` của `commands/hunk.rs`, và cùng lý do:
    /// lỗi #1 và #5 của `CONTEXT.md` §4.1 là cổng grep khớp **chú thích** thay vì mã.
    /// Tệp này nguy hiểm đúng kiểu đó — doc comment của nó chứa nguyên văn cụm
    /// `pub fn new` trong câu giải thích vì sao **không** có `pub fn new`.
    fn than_khong_chu_thich() -> String {
        let src = include_str!("trash.rs");
        let loc: String = src
            .lines()
            .filter(|l| {
                let t = l.trim_start();
                !t.starts_with("//") && !t.starts_with("///") && !t.starts_with("//!")
            })
            .collect::<Vec<_>>()
            .join("\n");

        loc.split("mod tests")
            .next()
            .expect("split luôn cho ít nhất một phần tử")
            .to_owned()
    }

    /// Phép lọc **tự nó** hoạt động — không có test này thì cả hai hướng hỏng đều tạo
    /// ra cổng hỏng, và không hướng nào gây lỗi biên dịch.
    #[test]
    fn phep_loc_chu_thich_hoat_dong() {
        let than = than_khong_chu_thich();

        assert!(
            !than.contains("mod tests"),
            "🔴 phép cắt `mod tests` hỏng — cổng đọc cả mã test, tức khớp chính chuỗi \
             khẳng định của chính nó"
        );
        assert!(
            !than.contains("Đừng nới thành"),
            "🔴 một dòng `///` sống sót qua phép lọc — cổng đang đo văn xuôi, không đo mã"
        );
        assert!(
            than.contains("pub struct BienNhan"),
            "🔴 phép lọc quá tay: thân mất cả khai báo `BienNhan`. Thân rỗng làm MỌI \
             cổng dưới đây XANH với mọi mã"
        );
    }

    /// **`BienNhan` không dựng được từ ngoài crate** — Task 1 Test 3.
    ///
    /// Đây là cổng giữ cho ràng buộc T-05-09 có thật. Nếu ai đó thêm một
    /// `pub fn new`/`pub fn moi` thì mọi đường huỷ lại dựng được biên nhận rỗng nghĩa
    /// và "không biên dịch được nếu quên lưu" thành một lời hứa suông.
    #[test]
    fn bien_nhan_khong_co_ham_dung_cong_khai() {
        let than = than_khong_chu_thich();

        // Tiền đề trước (lỗi #3): không thấy thứ cần kiểm ⇒ ĐỎ, không phải xanh.
        assert!(
            than.contains("struct BienNhan"),
            "🔴 tiền đề sai: không thấy `struct BienNhan` trong thân đã lọc — cổng \
             đang tìm trong chuỗi rỗng và sẽ XANH với MỌI mã"
        );

        for cam in ["pub fn moi", "pub fn new", "pub fn tao"] {
            assert!(
                !than.contains(cam),
                "🔴 `{cam}` làm `BienNhan` dựng được từ ngoài module. Ràng buộc \
                 'đường huỷ quên lưu thì không biên dịch được' mất hết giá trị: một \
                 đường huỷ mới chỉ cần tự dựng biên nhận là bỏ qua được bước lưu"
            );
        }

        assert!(
            than.contains("pub(crate) fn moi"),
            "🔴 tiền đề sai: không thấy `pub(crate) fn moi`. Hoặc hàm dựng đã đổi tên \
             (cập nhật cổng), hoặc nó đã thành `pub` (phá ràng buộc)"
        );
    }

    /// Ref nằm dưới **đúng** không gian riêng — Task 1 Test 4.
    #[test]
    fn ref_nam_duoi_dung_khong_gian_rieng() {
        assert_eq!(
            TIEN_TO_REF, "refs/git-plum-trash/",
            "🔴 không gian ref đổi — cập nhật cả `restore_trash`, thứ lọc theo tiền tố này"
        );
        assert!(
            !TIEN_TO_REF.starts_with("refs/stash"),
            "🔴 `refs/stash` là stash THẬT của người dùng. Ghi đè nó nghĩa là một tính \
             năng chống mất dữ liệu đi phá dữ liệu của người dùng ở chỗ khác"
        );
        assert!(
            !TIEN_TO_REF.starts_with("refs/heads"),
            "🔴 `refs/heads/*` hiện thành NHÁNH trong thanh bên — mỗi lần huỷ đẻ một \
             nhánh rác"
        );
        assert!(
            TIEN_TO_REF.ends_with('/'),
            "🔴 thiếu `/` cuối: phép kiểm tiền tố của `restore_trash` sẽ chấp nhận \
             `refs/git-plum-trash-evil/...`, tức một ref NGOÀI không gian riêng"
        );
    }

    /// Cả **ba** nhánh `StatusGroup` — Task 3 Test 1.
    ///
    /// 🔴 Khẳng định cả ba, không chỉ nhánh `Untracked`. Chỉ khẳng định `Untracked` thì
    /// một cài đặt trả `Xoa` cho **mọi** nhóm vẫn xanh — đúng lớp lỗi #4 "fixture không
    /// phân biệt được". Đột biến M19 (luôn `HuyBo`) và M20 (luôn `Xoa`) phải **đều** đỏ
    /// ở test này; nếu chỉ một trong hai đỏ thì test chỉ kiểm một chiều.
    #[test]
    fn dong_tu_cho_phan_biet_ca_ba_nhom() {
        assert_eq!(
            dong_tu_cho(StatusGroup::Untracked),
            DongTu::Xoa,
            "🔴 tệp CHƯA THEO DÕI phải là `Xoá` — ROADMAP, không thương lượng. Nội \
             dung đó không có ở đâu khác, nên 'Huỷ bỏ' nói SAI mức độ nghiêm trọng"
        );
        assert_eq!(
            dong_tu_cho(StatusGroup::Unstaged),
            DongTu::HuyBo,
            "🔴 tệp đã theo dõi chưa stage phải là `Huỷ bỏ` — có bản gốc trong git để \
             quay về. Nếu nhánh này cũng trả `Xoa` thì hàm không phân biệt gì"
        );
        assert_eq!(
            dong_tu_cho(StatusGroup::Staged),
            DongTu::HuyBo,
            "🔴 tệp đã stage phải là `Huỷ bỏ`. Nếu nhánh này cũng trả `Xoa` thì hàm \
             không phân biệt gì"
        );
    }

    /// Chuỗi hiển thị đúng **dấu** — Task 3 Test 2.
    #[test]
    fn nhan_hien_thi_co_dau_tieng_viet() {
        assert_eq!(
            DongTu::Xoa.nhan(),
            "Xoá",
            "🔴 phải là `Xoá` có dấu sắc, không phải `Xoa` — tên biến thể Rust không \
             mang dấu được, nên thiếu bước dịch này nghĩa là tên biến thể ASCII rò ra \
             thẳng giao diện"
        );
        assert_eq!(DongTu::HuyBo.nhan(), "Huỷ bỏ");
        assert_ne!(
            DongTu::Xoa.nhan(),
            DongTu::HuyBo.nhan(),
            "🔴 hai động từ phải khác nhau, nếu không thì phân nhánh vô nghĩa"
        );
    }

    /// Serialize — Task 3 Test 3. Liệt kê **toàn bộ** giá trị có thể, không kiểm một giá trị.
    #[test]
    fn dong_tu_serialize_camel_case_day_du() {
        let cap: Vec<(DongTu, &str)> = vec![(DongTu::HuyBo, "\"huyBo\""), (DongTu::Xoa, "\"xoa\"")];

        assert_eq!(
            cap.len(),
            2,
            "🔴 `DongTu` có thêm biến thể mà bảng này chưa liệt kê — thêm nó vào đây, \
             đừng chỉ sửa số"
        );

        for (v, mong) in cap {
            assert_eq!(
                serde_json::to_string(&v).expect("serialize được"),
                mong,
                "🔴 JSON của {v:?} sai. Lớp lỗi `rename_all` đã cắn ở Phase 3 và nó \
                 KHÔNG gây lỗi biên dịch ở bên nào"
            );
        }
    }

    /// **Toàn bộ** khoá JSON của `MucThungRac`, so **bằng**.
    ///
    /// 🔴 So bằng chứ không `contains_key`: một khoá **thừa** hay một khoá **thiếu** đều
    /// là lỗi, và `contains_key` chỉ bắt được khoá thiếu. Đây đúng là cách lỗi
    /// `rename_all` của Phase 3 bị bắt (`domain/diff.rs` ghi lại).
    #[test]
    fn muc_thung_rac_khoa_json_day_du() {
        let m = MucThungRac {
            ref_name: "refs/git-plum-trash/1-0".to_owned(),
            object_id: "deadbeef".to_owned(),
            la_blob: false,
            paths: vec!["a.txt".to_owned()],
            luc: 42,
            nhan: "Huỷ bỏ 1 tệp".to_owned(),
        };

        let v: serde_json::Value = serde_json::to_value(&m).expect("serialize được");
        let obj = v.as_object().expect("là object JSON");

        let mut khoa: Vec<&str> = obj.keys().map(String::as_str).collect();
        khoa.sort_unstable();

        assert_eq!(
            khoa,
            vec!["laBlob", "luc", "nhan", "objectId", "paths", "refName"],
            "🔴 tập khoá JSON đổi. Phía TypeScript đọc ĐÚNG những tên này, và lệch tên \
             không gây lỗi biên dịch ở bên nào — nó chỉ làm trường thành `undefined` \
             lúc chạy"
        );
    }
}
