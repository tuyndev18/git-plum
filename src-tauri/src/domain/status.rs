//! `RepoStatus` — hợp đồng dữ liệu trạng thái thư mục làm việc, nền của cả Phase 4.
//!
//! Kiểu này là đầu ra của [`crate::git::parsers::status::parse_status`] và là **nguồn
//! sự thật duy nhất** cho bốn requirement cùng lúc:
//!
//! * **WORK-01** — ba nhóm tệp (đã stage / chưa stage / chưa theo dõi) qua
//!   [`StatusGroup`];
//! * **WORK-02** — stage/unstage theo tệp, đọc lại một `RepoStatus` mới sau mỗi thao tác;
//! * **WORK-09** — cảnh báo amend cần ahead/behind, lấy từ [`BranchInfo`];
//! * **WORK-11** — số `✏3 +1` của hàng WIP, lấy từ [`RepoStatus::wip_counts`].
//!
//! # Vì sao `wip_counts` là hàm dẫn xuất chứ không phải một trường
//!
//! Ràng buộc ROADMAP nói nguyên văn: *"Số đếm lấy từ **cùng** lời gọi
//! `git status --porcelain=v2` của WORK-01 — **không** thêm lệnh git thứ hai chỉ để
//! đếm."* Một trường `wip_counts: WipCounts` có thể được ghi từ bất kỳ đâu, kể cả từ
//! một lời gọi git thứ hai, và không có gì trong *kiểu* ngăn điều đó. Một **hàm** trên
//! `RepoStatus` thì ngăn: không có đường nào lấy được số đếm mà không có sẵn một
//! `RepoStatus` trong tay. Ràng buộc được bảo đảm bằng kiểu, không bằng kỷ luật.
//!
//! Cùng lý do, [`STATUS_ARGS`] sống ở đây chứ không ở `commands/`: plan 04-02 dùng nó
//! và test của plan 04-01 khẳng định nó, nên nó phải nằm ở chỗ cả hai thấy được.

use serde::Serialize;
use std::collections::HashSet;

/// Đối số của lệnh trạng thái — **nguồn duy nhất** của định dạng này.
///
/// `["status", "--porcelain=v2", "--branch", "-z", "--untracked-files=all"]`
///
/// Vì sao từng cờ (CONTEXT.md mục 2.1, ROADMAP đã chốt, không thương lượng):
///
/// * **`--porcelain=v2`** chứ không `v1`: v2 mang XY, mode, SHA **và** thông tin đổi
///   tên trong cùng một bản ghi, nên không cần lệnh git thứ hai để biết tệp đổi tên.
/// * **`--branch`**: lấy nhánh hiện tại cùng ahead/behind trong **một** lời gọi —
///   WORK-09 cần biết commit sắp amend đã push hay chưa.
/// * **`-z`**: kết thúc bản ghi bằng NUL. Tên tệp có khoảng trắng, dấu ngoặc kép, hoặc
///   byte không phải UTF-8 là ca thật, và không có `-z` thì git **trích dẫn** đường dẫn
///   — một lớp mã hoá nữa phải gỡ, và là chỗ Phase 2 lẫn Phase 3 đã bị cắn.
/// * **`--untracked-files=all`**: liệt kê **từng** tệp chưa theo dõi thay vì gộp thành
///   một dòng thư mục. Gộp thì số đếm của WORK-11 sai, và người dùng không stage được
///   một tệp lẻ bên trong một thư mục mới.
///
/// Bỏ bất kỳ cờ nào trong bốn cờ này đều làm hỏng ít nhất một requirement — có test
/// khẳng định **từng cờ một theo tên**, không chỉ đếm số phần tử.
pub const STATUS_ARGS: &[&str] = &[
    "status",
    "--porcelain=v2",
    "--branch",
    "-z",
    "--untracked-files=all",
];

/// Ba nhóm tệp của WORK-01 tiêu chí 1.
///
/// # Vì sao phép phân nhóm ở domain chứ không ở TypeScript
///
/// Suy ra nhóm từ hai ký tự XY là **logic**, không phải trình bày: `.M` là chưa stage,
/// `M.` là đã stage, `MM` là **cả hai**. Để phép đó ở giao diện nghĩa là nó không có
/// một test Rust nào, và Phase 3 đã chứng minh test happy-dom không bắt được lớp lỗi
/// này (CONTEXT.md mục 3.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusGroup {
    /// Thay đổi đã nằm trong index — ký tự **đầu** của XY khác `.`.
    Staged,
    /// Thay đổi ở thư mục làm việc chưa vào index — ký tự **sau** của XY khác `.`.
    Unstaged,
    /// Tệp chưa theo dõi — bản ghi dạng `?`.
    Untracked,
}

/// Một tệp trong một nhóm trạng thái.
///
/// Một tệp XY = `MM` sinh **hai** `StatusEntry` — một `Staged`, một `Unstaged` — vì nó
/// thật sự xuất hiện ở cả hai nhóm trên giao diện (khuôn của GitHub Desktop). Đó là lý
/// do [`RepoStatus::wip_counts`] phải đếm theo **đường dẫn duy nhất**, không theo số
/// phần tử của [`RepoStatus::entries`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEntry {
    /// Đường dẫn hiện tại, tương đối gốc repo, dấu `/` như git in ra.
    ///
    /// Đã giải mã **lossy**: byte không hợp lệ thành U+FFFD và không khôi phục được.
    /// Xem [`StatusEntry::has_invalid_utf8`] trước khi dùng nó làm đầu vào lệnh git.
    pub path: String,

    /// Đường dẫn **cũ** của một tệp đổi tên hoặc sao chép (bản ghi dạng `2`).
    ///
    /// `None` cho mọi dạng bản ghi khác. Đường dẫn mới nằm ở [`StatusEntry::path`] —
    /// git in **mới trước, cũ sau**, đã đo bằng `od -c` chứ không đọc tài liệu.
    pub old_path: Option<String>,

    /// Hai ký tự trạng thái XY, ví dụ `"M."`, `".M"`, `"MM"`, `"R."`, `"??"`, `"UU"`.
    ///
    /// Là `String` chứ không phải `char`: cùng lý do `FileChange.status` của Phase 2 là
    /// `String`, và vì XY vốn là **hai** ký tự nên `char` không đựng nổi.
    pub xy: String,

    /// Nhóm mà phần tử này thuộc về.
    pub group: StatusGroup,

    /// `true` khi đường dẫn phải giải mã lossy (byte không phải UTF-8 hợp lệ).
    ///
    /// Khuôn của `Commit::has_invalid_utf8` từ Phase 2, cùng mục đích: đánh dấu bản ghi
    /// mà [`StatusEntry::path`] **không** dùng lại được làm đối số git.
    pub has_invalid_utf8: bool,
}

/// Thông tin nhánh, đọc từ các dòng `# branch.*` của **cùng** lời gọi status.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchInfo {
    /// Tên nhánh hiện tại, `None` khi HEAD tách rời.
    ///
    /// `# branch.head` in đúng chữ `(detached)` khi HEAD tách rời — đã đo. Ánh xạ nó
    /// thành `None` chứ không giữ nguyên chuỗi: phía giao diện cần một phép kiểm dứt
    /// khoát, không phải một phép so chuỗi rải khắp nơi.
    pub head: Option<String>,

    /// SHA của HEAD (`# branch.oid`). `(initial)` ở repo chưa có commit → `None`.
    pub oid: Option<String>,

    /// Nhánh upstream (`# branch.upstream`), `None` khi nhánh chưa có upstream.
    pub upstream: Option<String>,

    /// Số commit đi trước upstream (`# branch.ab +N -M`).
    ///
    /// # 🔴 `None` ≠ `Some(0)` — đây là một phân biệt có yêu cầu đứng sau
    ///
    /// Dòng `# branch.ab` **vắng mặt hoàn toàn** khi nhánh không có upstream; git
    /// **không** in `+0 -0`. Đã đo: repo mới `git init` chỉ có `branch.oid` và
    /// `branch.head`; repo git-plum có thêm `branch.upstream` và `branch.ab +27 -0`.
    ///
    /// WORK-09 phải phân biệt **"chưa từng push"** (`None` — amend vô hại) với
    /// **"đã push và đang đồng bộ"** (`Some(0)` — amend viết lại lịch sử người khác đã
    /// thấy, phải cảnh báo). Mặc định hoá `None` thành `Some(0)` làm cảnh báo đó nổ ở
    /// mọi repo cục bộ, và người dùng sẽ học cách bỏ qua nó.
    pub ahead: Option<u32>,

    /// Số commit đi sau upstream (`# branch.ab +N -M`). Xem ghi chú ở [`BranchInfo::ahead`].
    pub behind: Option<u32>,
}

/// Số đếm hàng WIP của WORK-11 — chữ `✏3 +1` trong ảnh GitKraken chủ dự án đưa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WipCounts {
    /// Số **tệp** có nội dung khác HEAD (biểu tượng ✏).
    pub modified: u32,
    /// Số **tệp** mới: chưa theo dõi (`?`) **và** đã stage thêm mới (`A`) (biểu tượng +).
    pub added: u32,
}

/// Trạng thái thư mục làm việc tại một thời điểm.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoStatus {
    /// Nhánh hiện tại và quan hệ với upstream.
    pub branch: BranchInfo,

    /// Mọi phần tử, **giữ nguyên thứ tự git in ra** (git sắp theo đường dẫn).
    ///
    /// Một tệp có thể xuất hiện **hai lần** với hai nhóm khác nhau — xem [`StatusEntry`].
    pub entries: Vec<StatusEntry>,

    /// `true` khi có ít nhất một bản ghi dạng `u` (xung đột chưa giải quyết).
    ///
    /// Tách thành một cờ riêng vì nó chặn commit: vòng commit của WORK-08 phải biết
    /// điều này **trước** khi cho bấm nút, không phải bằng cách quét lại `entries`.
    pub has_conflicts: bool,
}

impl RepoStatus {
    /// Số tệp sửa / tệp mới cho hàng WIP của WORK-11.
    ///
    /// # Đếm theo đường dẫn duy nhất, KHÔNG theo số phần tử
    ///
    /// Một tệp XY = `MM` có **hai** phần tử trong [`RepoStatus::entries`] (một ở nhóm
    /// đã stage, một ở nhóm chưa stage) nhưng người dùng chỉ sửa **một** tệp, nên
    /// `✏` phải đếm 1. Đếm `entries.len()` cho 2 và con số hiện lên giao diện sai gấp
    /// đôi ở mọi repo có tệp sửa cả hai phía — ca rất thường khi người ta stage rồi sửa
    /// tiếp. Đột biến M8 của plan ghim đúng chỗ này.
    ///
    /// # Phân loại
    ///
    /// * `added` — tệp **mới** với mắt người dùng: dạng `?` (chưa theo dõi) **và** XY
    ///   có `A` (đã stage thêm mới). Một tệp đã `git add` xong vẫn là tệp mới.
    /// * `modified` — mọi tệp theo dõi còn lại có thay đổi (`M`, `R`, `C`, `D`, `T`,
    ///   `U`). Một tệp đã đếm là `added` **không** đếm lại là `modified`.
    ///
    /// Không sinh lệnh git nào — xem ghi chú đầu module về việc ràng buộc này được bảo
    /// đảm bằng *kiểu*.
    pub fn wip_counts(&self) -> WipCounts {
        let mut da_them: HashSet<&str> = HashSet::new();
        let mut da_sua: HashSet<&str> = HashSet::new();

        for e in &self.entries {
            if e.group == StatusGroup::Untracked || e.xy.contains('A') {
                da_them.insert(e.path.as_str());
            } else {
                da_sua.insert(e.path.as_str());
            }
        }

        // Một tệp vừa được stage thêm mới (`A.`) vừa sửa tiếp ở worktree (`.M`) sinh
        // hai phần tử: một rơi vào `da_them`, một rơi vào `da_sua`. Với mắt người dùng
        // đó là **một tệp mới**, nên `added` thắng và ta trừ nó khỏi `modified`.
        for p in &da_them {
            da_sua.remove(p);
        }

        WipCounts {
            modified: da_sua.len() as u32,
            added: da_them.len() as u32,
        }
    }

    /// `true` khi không có thay đổi nào — hàng WIP của WORK-11 phải **biến mất**.
    ///
    /// Đây là cờ mà tiêu chí thành công số 7 của ROADMAP đo ("hàng WIP xuất hiện và
    /// biến mất đúng lúc"). Dựa trên `entries` chứ không trên `wip_counts`: một bản ghi
    /// dạng `u` (xung đột) không đếm vào `added` lẫn `modified` theo cách hiển thị,
    /// nhưng repo có xung đột thì **không** sạch.
    pub fn is_clean(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dựng nhanh một phần tử cho test.
    fn muc(path: &str, xy: &str, group: StatusGroup) -> StatusEntry {
        StatusEntry {
            path: path.to_owned(),
            old_path: None,
            xy: xy.to_owned(),
            group,
            has_invalid_utf8: false,
        }
    }

    /// `✏3 +1` — đúng con số trong ảnh GitKraken mà WORK-11 phải hiện.
    ///
    /// Ba tệp sửa (một đã stage, một chưa, một cả hai) cộng một tệp chưa theo dõi.
    #[test]
    fn wip_counts_ba_tep_sua_mot_tep_moi() {
        let st = RepoStatus {
            entries: vec![
                muc("da_stage.txt", "M.", StatusGroup::Staged),
                muc("chua_stage.txt", ".M", StatusGroup::Unstaged),
                muc("ca_hai.txt", "MM", StatusGroup::Staged),
                muc("ca_hai.txt", "MM", StatusGroup::Unstaged),
                muc("moi.txt", "??", StatusGroup::Untracked),
            ],
            ..Default::default()
        };

        assert_eq!(
            st.wip_counts(),
            WipCounts {
                modified: 3,
                added: 1
            },
            "hàng WIP phải hiện ✏3 +1 — ba ĐƯỜNG DẪN sửa, một tệp mới"
        );
    }

    /// 🔴 Đột biến M8: một tệp `MM` sinh HAI phần tử nhưng là MỘT tệp sửa.
    ///
    /// Tách riêng khỏi test trên để khi nó đỏ thì thông điệp chỉ thẳng vào nguyên nhân
    /// ("đếm entry thay vì đường dẫn") chứ không lẫn với bốn tệp khác.
    #[test]
    fn wip_counts_tep_mm_dem_la_mot_khong_phai_hai() {
        let st = RepoStatus {
            entries: vec![
                muc("ca_hai.txt", "MM", StatusGroup::Staged),
                muc("ca_hai.txt", "MM", StatusGroup::Unstaged),
            ],
            ..Default::default()
        };

        assert_eq!(
            st.wip_counts().modified,
            1,
            "một tệp XY=MM có hai phần tử nhưng người dùng chỉ sửa MỘT tệp; \
             đếm entries.len() cho 2 và số trên giao diện sai gấp đôi"
        );
    }

    /// Status sạch → không số đếm nào, và `is_clean()` để hàng WIP biến mất (tiêu chí 7).
    #[test]
    fn wip_counts_status_sach_la_khong_va_is_clean() {
        let st = RepoStatus::default();

        assert_eq!(st.wip_counts(), WipCounts::default());
        assert!(
            st.is_clean(),
            "repo sạch phải cho is_clean() = true để hàng WIP biến mất"
        );
    }

    /// Có bất kỳ phần tử nào thì `is_clean()` phải là `false`.
    #[test]
    fn is_clean_false_khi_co_phan_tu() {
        let st = RepoStatus {
            entries: vec![muc("a.txt", ".M", StatusGroup::Unstaged)],
            ..Default::default()
        };

        assert!(!st.is_clean());
    }

    /// Tệp `A` đã stage vẫn là "tệp mới" với mắt người dùng, không phải tệp sửa.
    #[test]
    fn wip_counts_tep_a_da_stage_dem_la_tep_moi() {
        let st = RepoStatus {
            entries: vec![
                muc("them_moi.txt", "A.", StatusGroup::Staged),
                muc("chua_theo_doi.txt", "??", StatusGroup::Untracked),
            ],
            ..Default::default()
        };

        assert_eq!(
            st.wip_counts(),
            WipCounts {
                modified: 0,
                added: 2
            },
            "một tệp đã `git add` vẫn là tệp MỚI, không phải tệp sửa"
        );
    }

    /// Tệp vừa `A` (index) vừa `M` (worktree) đếm **một lần**, và đếm là tệp mới.
    #[test]
    fn wip_counts_tep_am_dem_mot_lan_va_la_tep_moi() {
        let st = RepoStatus {
            entries: vec![
                muc("moi_roi_sua.txt", "AM", StatusGroup::Staged),
                muc("moi_roi_sua.txt", "AM", StatusGroup::Unstaged),
            ],
            ..Default::default()
        };

        assert_eq!(
            st.wip_counts(),
            WipCounts {
                modified: 0,
                added: 1
            },
            "tệp mới rồi sửa tiếp vẫn là MỘT tệp mới, không phải một mới + một sửa"
        );
    }

    /// `BranchInfo` có upstream: `ahead`/`behind` là `Some`.
    #[test]
    fn branch_info_co_upstream_thi_ahead_behind_la_some() {
        let b = BranchInfo {
            head: Some("main".into()),
            oid: Some("abc".into()),
            upstream: Some("origin/main".into()),
            ahead: Some(3),
            behind: Some(2),
        };

        assert_eq!(b.ahead, Some(3));
        assert_eq!(b.behind, Some(2));
    }

    /// 🔴 Không upstream → `None`, **không** phải `Some(0)` — đột biến M7.
    ///
    /// WORK-09 dùng đúng phân biệt này để không cảnh báo amend ở repo chưa từng push.
    #[test]
    fn branch_info_khong_upstream_thi_none_khong_phai_some_0() {
        let b = BranchInfo {
            head: Some("main".into()),
            oid: Some("abc".into()),
            ..Default::default()
        };

        assert_eq!(
            b.ahead, None,
            "chưa có upstream phải là None; Some(0) nghĩa là ĐÃ push và đang đồng bộ"
        );
        assert_eq!(b.behind, None);
        assert_ne!(b.ahead, Some(0), "None và Some(0) là hai trạng thái KHÁC nhau");
    }

    /// 🔴 Đột biến M5/M6: khẳng định **từng cờ một theo tên**, không chỉ đếm phần tử.
    ///
    /// Đếm số phần tử thôi thì đổi `--untracked-files=all` thành `--untracked-files=no`
    /// vẫn cho 5 và cổng xanh — đúng lớp "cổng không thể fail" của CONTEXT.md 3.1.
    #[test]
    fn status_args_co_du_bon_co_theo_ten() {
        assert_eq!(
            STATUS_ARGS[0], "status",
            "phần tử đầu phải là subcommand `status`"
        );
        assert!(
            STATUS_ARGS.contains(&"--porcelain=v2"),
            "thiếu --porcelain=v2: v1 không mang thông tin đổi tên trong một bản ghi"
        );
        assert!(
            STATUS_ARGS.contains(&"--branch"),
            "thiếu --branch: WORK-09 không lấy được ahead/behind mà không thêm lệnh git thứ hai"
        );
        assert!(
            STATUS_ARGS.contains(&"-z"),
            "thiếu -z: đường dẫn sẽ bị git trích dẫn, thêm một lớp mã hoá phải gỡ"
        );
        assert!(
            STATUS_ARGS.contains(&"--untracked-files=all"),
            "thiếu --untracked-files=all: git gộp thư mục mới thành một dòng, \
             số đếm WORK-11 sai và không stage được tệp lẻ trong đó"
        );
        assert_eq!(
            STATUS_ARGS.len(),
            5,
            "đúng subcommand + bốn cờ; thêm cờ lạ phải làm test này đỏ để ai đó đọc lại lý do"
        );
    }

    /// `--porcelain=v2` chứ **không** `v1` hay `--porcelain` trơn.
    ///
    /// Tách riêng vì `contains(&"--porcelain=v2")` ở trên vẫn xanh nếu ai đó **thêm**
    /// `--porcelain` trơn vào; ở đây khẳng định không có biến thể nào khác lọt vào.
    #[test]
    fn status_args_khong_co_bien_the_porcelain_khac() {
        let khac: Vec<&&str> = STATUS_ARGS
            .iter()
            .filter(|a| a.starts_with("--porcelain") && **a != "--porcelain=v2")
            .collect();

        assert!(
            khac.is_empty(),
            "chỉ được có --porcelain=v2, thấy thêm: {khac:?}"
        );
    }
}
