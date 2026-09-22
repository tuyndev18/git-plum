//! Test tích hợp cho bước **word-level** của `get_file_diff` — plan 03-03, Task 2.
//!
//! # Điều được kiểm ở đây khác hẳn `word_diff_integration.rs`
//!
//! Tệp kia kiểm **bộ phân tích**: porcelain vào, `WordLine` ra, và hai bộ phân tích có
//! đồng ý với nhau không. Tệp này kiểm **quyết định của command**: lệnh git thứ hai
//! có chạy không, chạy khi nào, và khoảng có rơi vào đúng dòng không.
//!
//! Ba trong số đó **không quan sát được từ giá trị trả về**:
//!
//! * "tệp chỉ thêm thì KHÔNG chạy lệnh word-diff" — một cài đặt chạy rồi vứt kết quả
//!   trả về **đúng** cùng một `FileDiff`. Khác biệt duy nhất nằm trong `CommandLog`.
//! * "lần gọi thứ hai không sinh lệnh nào" — cùng lý do.
//! * "bước word-level nằm TRƯỚC `put_diff`" — đặt sau cũng trả đúng `spans` ở lần gọi
//!   **đầu**, và chỉ sai từ lần thứ hai trở đi.
//!
//! Nên test ở đây đọc `CommandLog`, không chỉ đọc kết quả. Cùng khuôn mà 03-02 phải
//! dùng cho cổng DIFF-06, và cùng lý do.

use std::sync::Arc;

use git_plum_lib::commands::diff::{lay_diff_tep, WORD_DIFF_MAX_CHANGED_LINES};
use git_plum_lib::domain::diff::{DiffKind, DiffLine, LineKind};
use git_plum_lib::state::AppState;
use git_plum_lib::testing::{require_diff_fixture, DiffFixture};

fn mo() -> Option<(AppState, Arc<git_plum_lib::state::RepoHandle>, DiffFixture)> {
    let fx = require_diff_fixture()?;
    let state = AppState::new();
    let repo = state.open_repo(&fx.repo);
    Some((state, repo, fx))
}

/// Mọi lệnh đã chạy, dạng chuỗi — để khẳng định *có* hoặc *không có* `word-diff`.
fn cac_lenh(state: &AppState) -> Vec<String> {
    state
        .command_log
        .entries()
        .into_iter()
        .map(|e| e.command)
        .collect()
}

/// Số lệnh mang cờ `--word-diff`.
fn so_lenh_word_diff(state: &AppState) -> usize {
    cac_lenh(state)
        .iter()
        .filter(|c| c.contains("word-diff"))
        .count()
}

/// Mọi `DiffLine` của một `FileDiff` dạng `text`.
fn dong_cua(fd: &git_plum_lib::domain::diff::FileDiff) -> Vec<DiffLine> {
    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("phải là kind `text`, nhận: {:?}", fd.kind);
    };
    hunks.iter().flat_map(|h| h.lines.clone()).collect()
}

// ---------------------------------------------------------------------------
// Ca chính: tệp vừa thêm vừa xoá → có spans, và spans HẸP HƠN cả dòng
// ---------------------------------------------------------------------------

/// **Ca của chủ dự án, đi qua đúng đường mà webview đi.**
///
/// `empty-line.txt` mang hình dạng `x == 1` → `x === 1`. Khẳng định then chốt là
/// khoảng **hẹp hơn cả dòng** — đó là điều chủ dự án yêu cầu tường minh ("không chỉ
/// tô cả dòng"), và một cài đặt trả về một khoảng phủ toàn dòng qua được mọi phép
/// kiểm "có span".
#[tokio::test]
async fn tep_vua_them_vua_xoa_co_spans_hep_hon_ca_dong() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("emptyline_mod"), "empty-line.txt")
        .await
        .expect("empty-line.txt phải trả Ok");

    let dong = dong_cua(&fd);

    let them = dong
        .iter()
        .find(|l| l.kind == LineKind::Added)
        .expect("phải có dòng thêm");
    assert_eq!(
        them.spans.len(),
        1,
        "dòng `added` của một ca sửa-tại-chỗ phải có ĐÚNG một khoảng. Nhận: {:?} trên \
         {:?}",
        them.spans,
        them.content
    );
    let s = them.spans[0];
    assert_eq!(
        &them.content[s.start..s.end],
        "===",
        "khoảng phải trỏ đúng vào `===` — đây là ca chủ dự án nêu tên"
    );
    assert!(
        s.end - s.start < them.content.len(),
        "khoảng phải HẸP HƠN cả dòng. Nhận [{}, {}) trên {:?} dài {} byte",
        s.start,
        s.end,
        them.content,
        them.content.len()
    );

    let xoa = dong
        .iter()
        .find(|l| l.kind == LineKind::Removed)
        .expect("phải có dòng xoá");
    assert_eq!(xoa.spans.len(), 1);
    let sc = xoa.spans[0];
    assert_eq!(&xoa.content[sc.start..sc.end], "==");
    assert!(sc.end - sc.start < xoa.content.len());

    // Dòng ngữ cảnh KHÔNG được mang khoảng — chúng không đổi.
    for l in dong.iter().filter(|l| l.kind == LineKind::Context) {
        assert!(
            l.spans.is_empty(),
            "dòng ngữ cảnh {:?} không được có khoảng thay đổi: {:?}",
            l.content,
            l.spans
        );
    }

    // Tiền đề: lệnh word-diff PHẢI đã chạy, nếu không mọi khẳng định trên vô nghĩa.
    assert_eq!(
        so_lenh_word_diff(&state),
        1,
        "đúng MỘT lệnh word-diff cho một lần mở diff. Lệnh đã chạy: {:#?}",
        cac_lenh(&state)
    );
}

/// **Khớp đúng dòng khi hai dòng có nội dung TRÙNG NHAU trong một hunk.**
///
/// `dup-lines.txt` có hai dòng `TRUNG` giống hệt nhau và chỉ bản **sau** bị sửa. Một
/// cài đặt khớp `WordLine` với `DiffLine` theo **nội dung** sẽ gán khoảng cho bản
/// trước — đúng dòng sai chỗ, và không phép kiểm "có span" nào bắt được.
#[tokio::test]
async fn khop_theo_so_dong_khong_theo_noi_dung_khi_co_dong_trung() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("dup_mod"), "dup-lines.txt")
        .await
        .expect("dup-lines.txt phải trả Ok");

    let dong = dong_cua(&fd);

    // Dòng `TRUNG` **nguyên vẹn** là dòng ngữ cảnh — nó KHÔNG đổi, nên không có khoảng.
    let trung_nguyen_ven: Vec<&DiffLine> = dong
        .iter()
        .filter(|l| l.content.trim() == "TRUNG" && l.kind == LineKind::Context)
        .collect();
    assert!(
        !trung_nguyen_ven.is_empty(),
        "tiền đề: phải còn ít nhất một dòng `TRUNG` nguyên vẹn làm ngữ cảnh. \
         Các dòng: {:?}",
        dong.iter()
            .map(|l| (l.kind, l.content.as_str()))
            .collect::<Vec<_>>()
    );
    for l in &trung_nguyen_ven {
        assert!(
            l.spans.is_empty(),
            "dòng `TRUNG` KHÔNG đổi (dòng {:?}) không được mang khoảng. Mang khoảng \
             nghĩa là phép khớp đã chọn theo NỘI DUNG và gán nhầm khoảng của dòng \
             `TRUNG DA SUA` cho nó — cùng nội dung, khác dòng. Nhận: {:?}",
            l.old_line.or(l.new_line),
            l.spans
        );
    }

    // Dòng thật sự bị sửa thì PHẢI có khoảng.
    let da_sua = dong
        .iter()
        .find(|l| l.kind == LineKind::Added && l.content.contains("DA SUA"))
        .expect("phải có dòng `TRUNG DA SUA`");
    assert!(
        !da_sua.spans.is_empty(),
        "dòng thật sự bị sửa phải có khoảng — không có nghĩa là phép khớp trượt hoàn \
         toàn, và test trên xanh một cách vô nghĩa"
    );
}

/// 🔴 **Khớp theo số dòng, trên ca mà khớp-theo-nội-dung THẬT SỰ cho kết quả khác.**
///
/// # Vì sao `dup-lines.txt` của 03-02 KHÔNG bắt được lỗi này — đã đo
///
/// Plan 03-03 chỉ định `dup-lines.txt` làm fixture cho mutation "khớp theo nội dung".
/// **Đã chạy mutation đó và nó XANH.** Lý do nằm ở hình dạng thật của fixture:
///
/// ```text
///  start / TRUNG / mid / TRUNG        →  start / TRUNG / mid / TRUNG DA SUA
///
/// WordLine phía cũ:  (2, "TRUNG", spans=[])   (4, "TRUNG", spans=[])
///                                      ↑ CẢ HAI khoảng RỖNG
/// ```
///
/// Git coi thay đổi đó là **thêm** ` DA SUA`, nên phía cũ không có khoảng nào. Khớp
/// theo nội dung chọn nhầm bản ghi dòng 2 — nhưng bản ghi đó cũng có `spans` rỗng,
/// nên kết quả **y hệt** và không test nào phân biệt được.
///
/// # Hình dạng THẬT SỰ phân biệt được: nội dung trùng, **khoảng khác nhau**
///
/// Cần hai dòng **cùng nội dung ở phía được khớp** mà **khoảng khác nhau**. Đạt được
/// bằng cách cho hai dòng khác nhau ở phía cũ **hội tụ** về cùng một nội dung ở phía
/// mới, đổi ở **hai vị trí khác nhau trong dòng**:
///
/// ```text
///  X b c / a b X        →     a b c / a b c
///
/// WordLine phía mới:  (2, "a b c", spans=[0,1))   (4, "a b c", spans=[4,5))
///                             ↑ cùng nội dung        ↑ KHÁC khoảng
/// ```
///
/// Khớp theo nội dung gán khoảng `[0,1)` cho **cả hai** dòng, nên dòng 4 tô chữ `a`
/// đầu dòng thay vì chữ `c` cuối dòng — sai chỗ một cách nhìn thấy được.
///
/// Fixture dựng **trong test** bằng `tempfile` chứ không thêm vào
/// `make-diff-fixtures.sh`: script đó không nằm trong `files_modified` của plan 03-03.
#[tokio::test]
async fn khop_theo_so_dong_khi_hai_dong_trung_noi_dung_nhung_khac_khoang() {
    let Some(_) = require_diff_fixture() else {
        return;
    };

    let tmp = tempfile::tempdir().expect("phải tạo được thư mục tạm");
    let repo_path = tmp.path().to_path_buf();

    let git = |args: &[&str]| {
        let ra = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo_path)
            .args(args)
            .output()
            .expect("phải chạy được git");
        assert!(
            ra.status.success(),
            "git {args:?} thất bại: {}",
            String::from_utf8_lossy(&ra.stderr)
        );
        String::from_utf8_lossy(&ra.stdout).trim().to_owned()
    };

    git(&["init", "-q", "."]);
    git(&["config", "user.email", "t@t"]);
    git(&["config", "user.name", "t"]);
    git(&["config", "core.autocrlf", "false"]);

    // Hai dòng KHÁC nhau ở phía cũ, đổi ở HAI vị trí khác nhau trong dòng...
    std::fs::write(repo_path.join("d.txt"), "start\nX b c\nmid\na b X\nend\n")
        .expect("phải ghi được tệp");
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "a"]);

    // ...hội tụ về CÙNG một nội dung ở phía mới.
    std::fs::write(repo_path.join("d.txt"), "start\na b c\nmid\na b c\nend\n")
        .expect("phải ghi được tệp");
    git(&["add", "-A"]);
    git(&["commit", "-q", "-m", "b"]);

    let sha = git(&["rev-parse", "HEAD"]);

    let state = AppState::new();
    let repo = state.open_repo(&repo_path);
    let fd = lay_diff_tep(&state, repo, &sha, "d.txt")
        .await
        .expect("phải trả Ok");

    let dong = dong_cua(&fd);
    let them: Vec<&DiffLine> = dong.iter().filter(|l| l.kind == LineKind::Added).collect();

    assert_eq!(
        them.len(),
        2,
        "tiền đề: phải có đúng hai dòng thêm. Các dòng: {:?}",
        dong.iter()
            .map(|l| (l.kind, l.content.as_str()))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        them[0].content, them[1].content,
        "tiền đề: hai dòng thêm phải TRÙNG nội dung, nếu không ca này không phân biệt \
         được khớp-theo-nội-dung với khớp-theo-số-dòng"
    );

    let s0 = them[0].spans.first().copied().unwrap_or_else(|| {
        panic!("dòng thêm thứ nhất phải có khoảng, nhận: {:?}", them[0])
    });
    let s1 = them[1].spans.first().copied().unwrap_or_else(|| {
        panic!("dòng thêm thứ hai phải có khoảng, nhận: {:?}", them[1])
    });

    assert_eq!(
        &them[0].content[s0.start..s0.end],
        "a",
        "dòng thêm THỨ NHẤT đổi ở ĐẦU dòng (`X b c` → `a b c`), nên khoảng phải trỏ \
         vào `a`. Nhận [{}, {}) trên {:?}",
        s0.start,
        s0.end,
        them[0].content
    );
    assert_eq!(
        &them[1].content[s1.start..s1.end],
        "c",
        "dòng thêm THỨ HAI đổi ở CUỐI dòng (`a b X` → `a b c`), nên khoảng phải trỏ \
         vào `c`.\n\
         Nhận `{}` — nếu nó là `a` thì phép khớp đã chọn theo NỘI DUNG và gán khoảng \
         của dòng thứ nhất cho dòng này: hai dòng cùng nội dung `a b c` nhưng KHÁC \
         khoảng, và chỉ khớp theo SỐ DÒNG phân biệt được chúng.",
        &them[1].content[s1.start..s1.end]
    );

    // Hai khoảng phải THẬT SỰ khác nhau, nếu không hai khẳng định trên vô nghĩa.
    assert_ne!(
        (s0.start, s0.end),
        (s1.start, s1.end),
        "tiền đề: hai khoảng phải khác nhau để ca này phân biệt được hai phép khớp"
    );
}

// ---------------------------------------------------------------------------
// Ba cổng bỏ qua — quan sát bằng CommandLog, không bằng giá trị trả về
// ---------------------------------------------------------------------------

/// **Tệp chỉ THÊM (tệp mới) → KHÔNG lệnh `--word-diff` nào.**
///
/// Word-level trên một tệp mới vô nghĩa: mọi dòng đều là dòng thêm, nên "phần chữ
/// thay đổi" là cả dòng. Một lệnh git thừa cho mỗi lần mở một tệp mới là chi phí thật
/// và nó không mua được gì (T-03-19).
///
/// Quan sát bằng `CommandLog`: một cài đặt chạy lệnh rồi vứt kết quả trả về **đúng**
/// cùng một `FileDiff`.
#[tokio::test]
async fn tep_chi_them_khong_chay_lenh_word_diff_nao() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("root"), "text-simple.txt")
        .await
        .expect("tệp mới thêm phải trả Ok");

    let dong = dong_cua(&fd);
    assert!(
        dong.iter().all(|l| l.kind == LineKind::Added),
        "tiền đề: mọi dòng của tệp mới phải là `added`, nếu không đây không phải ca \
         chỉ-thêm và cổng không kiểm đúng thứ nó tưởng"
    );

    assert_eq!(
        so_lenh_word_diff(&state),
        0,
        "KHÔNG lệnh `--word-diff` nào được chạy trên tệp chỉ THÊM — mọi dòng đều mới \
         nên word-level không nói thêm gì, và lệnh git thứ hai là chi phí thật \
         (T-03-19). Lệnh đã chạy: {:#?}",
        cac_lenh(&state)
    );
    // Tiền đề: nhật ký PHẢI ghi được lệnh, nếu không khẳng định trên luôn đúng vô nghĩa.
    assert!(
        !cac_lenh(&state).is_empty(),
        "nhật ký phải ghi được lệnh; nhật ký rỗng làm khẳng định trên xanh bất kể cài đặt"
    );
}

/// **Tệp bị XOÁ (chỉ xoá) → KHÔNG lệnh `--word-diff` nào.** Cùng lý do.
#[tokio::test]
async fn tep_chi_xoa_khong_chay_lenh_word_diff_nao() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("deleted"), "deleted.txt")
        .await
        .expect("tệp bị xoá phải trả Ok");

    let dong = dong_cua(&fd);
    assert!(
        dong.iter().all(|l| l.kind == LineKind::Removed),
        "tiền đề: mọi dòng của tệp bị xoá phải là `removed`"
    );

    assert_eq!(
        so_lenh_word_diff(&state),
        0,
        "KHÔNG lệnh `--word-diff` nào trên tệp chỉ XOÁ. Lệnh đã chạy: {:#?}",
        cac_lenh(&state)
    );
    assert!(!cac_lenh(&state).is_empty(), "tiền đề: nhật ký phải ghi được lệnh");
}

/// **Cache dùng chung mục: gọi lần hai thêm ĐÚNG 0 entry**, kể cả lệnh word-diff.
///
/// Đây cũng là cổng cho thứ tự: bước word-level phải nằm **trước** `put_diff`. Đặt
/// sau thì lần gọi đầu vẫn đúng và chỉ lần thứ hai mất `spans` — một lỗi chỉ hiện ra
/// khi người dùng bấm lại vào cùng một tệp, tức đúng thao tác thường gặp nhất.
#[tokio::test]
async fn goi_lan_hai_dung_cache_va_giu_nguyen_spans() {
    let Some((state, repo, fx)) = mo() else { return };

    let lan_dau = lay_diff_tep(
        &state,
        Arc::clone(&repo),
        fx.sha("emptyline_mod"),
        "empty-line.txt",
    )
    .await
    .expect("lần đầu phải trả Ok");
    let so_sau_lan_dau = cac_lenh(&state).len();
    assert!(so_sau_lan_dau > 0, "tiền đề: lần đầu phải sinh lệnh git");

    let lan_hai = lay_diff_tep(&state, repo, fx.sha("emptyline_mod"), "empty-line.txt")
        .await
        .expect("lần hai phải trả Ok");

    assert_eq!(
        cac_lenh(&state).len(),
        so_sau_lan_dau,
        "lần gọi thứ hai cho cùng (sha, path) phải thêm ĐÚNG 0 entry vào nhật ký — \
         kể cả lệnh `--word-diff`. Lệnh đã chạy: {:#?}",
        cac_lenh(&state)
    );

    let d1 = dong_cua(&lan_dau);
    let d2 = dong_cua(&lan_hai);
    assert_eq!(
        d1.len(),
        d2.len(),
        "hai lần gọi phải cho cùng số dòng"
    );

    let spans1: Vec<_> = d1.iter().map(|l| l.spans.clone()).collect();
    let spans2: Vec<_> = d2.iter().map(|l| l.spans.clone()).collect();
    assert_eq!(
        spans1, spans2,
        "`spans` của lần hai phải BẰNG lần đầu. Lệch nghĩa là bước word-level nằm SAU \
         `put_diff`, nên mục cache được ghi TRƯỚC khi khoảng được gắn vào — lần đầu \
         vẫn đúng, mọi lần sau mất word-level"
    );
    assert!(
        spans1.iter().any(|s| !s.is_empty()),
        "tiền đề: phải có ít nhất một dòng mang khoảng, nếu không phép so trên là so \
         hai danh sách rỗng và luôn xanh"
    );
}

/// Ngưỡng [`WORD_DIFF_MAX_CHANGED_LINES`] là một con số cụ thể mà giao diện và người
/// bảo trì đều phải biết. Đọc **giá trị**, không `grep -c`.
#[test]
fn nguong_so_dong_sua_dung_hai_nghin() {
    assert_eq!(
        WORD_DIFF_MAX_CHANGED_LINES, 2_000,
        "vượt ngưỡng thì bỏ hẳn lệnh word-diff và trả `spans` rỗng — suy giảm có chủ \
         ý, cùng khuôn `MAX_VISIBLE_LANES` của Phase 2"
    );
}

// ---------------------------------------------------------------------------
// Suy giảm: mất word-level KHÔNG được làm mất cả diff
// ---------------------------------------------------------------------------

/// **Lệnh word-diff thất bại → `spans` rỗng và `DiffKind::Text` vẫn trả về.**
///
/// Word-level là phần **trang trí**; mất nó không được làm mất cả diff. Mô phỏng ở
/// tầng đơn vị bằng cách đưa stdout **rỗng** vào bộ phân tích — đúng thứ mà một lệnh
/// thất bại để lại.
#[test]
fn stdout_rong_cho_khong_khoang_va_khong_panic() {
    use git_plum_lib::git::parsers::word_diff::parse_word_diff;

    let ra = parse_word_diff(b"");
    assert_eq!(ra.old_lines.len(), 0);
    assert_eq!(ra.new_lines.len(), 0);
    assert_eq!(
        ra.skipped, 0,
        "stdout rỗng của một lệnh thất bại KHÔNG phải dữ liệu hỏng — nó chỉ là không \
         có dữ liệu. Đếm nó vào `skipped` làm log đầy cảnh báo vô nghĩa"
    );
}

/// Tệp **nhị phân** không bao giờ tới bước word-level: cổng nhị phân trả về trước đó.
/// Một lệnh `--word-diff` trên tệp nhị phân vừa vô nghĩa vừa tốn.
#[tokio::test]
async fn tep_nhi_phan_khong_chay_lenh_word_diff_nao() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("binary_mod"), "binary.png")
        .await
        .expect("tệp nhị phân phải trả Ok");
    assert!(
        matches!(fd.kind, DiffKind::Binary { .. }),
        "tiền đề: phải là kind `binary`"
    );

    assert_eq!(
        so_lenh_word_diff(&state),
        0,
        "tệp nhị phân phải trả về TRƯỚC bước word-level. Lệnh đã chạy: {:#?}",
        cac_lenh(&state)
    );
}

/// Tệp **vượt ngưỡng kích thước** cũng không tới bước word-level — cổng DIFF-06 của
/// 03-02 trả về trước, và thêm một lệnh git trên tệp 6 MB là đúng thứ DIFF-06 cấm.
#[tokio::test]
async fn tep_vuot_nguong_kich_thuoc_khong_chay_lenh_word_diff_nao() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("large_mod"), "large.txt")
        .await
        .expect("tệp lớn phải trả Ok");
    assert!(
        matches!(fd.kind, DiffKind::TooLarge { .. }),
        "tiền đề: phải là kind `tooLarge`"
    );

    assert_eq!(
        so_lenh_word_diff(&state),
        0,
        "tệp vượt ngưỡng phải trả về TRƯỚC mọi lệnh diff, kể cả word-diff. \
         Lệnh đã chạy: {:#?}",
        cac_lenh(&state)
    );
}

// ---------------------------------------------------------------------------
// `no_newline_at_eof` KHÔNG được suy từ porcelain
// ---------------------------------------------------------------------------

/// 🔴 **Bước word-level KHÔNG được ghi đè `no_newline_at_eof`.**
///
/// Porcelain **không** cung cấp trường đó — nó in `~` bình thường cho dòng cuối không
/// có newline và **không** in `\ No newline at end of file` (đã đo; `--unified` trên
/// cùng tệp thì có in). Một cài đặt suy trường này từ porcelain sẽ luôn cho `false`,
/// xoá mất giá trị đúng mà `parse_patch` đã đặt.
///
/// Hệ quả nhìn thấy được: trình xem mất chỉ báo "tệp không kết thúc bằng dòng mới",
/// và Phase 5 (staging theo khối) dựng lại bản vá thiếu dòng `\ No newline...` — một
/// bản vá như vậy **git apply từ chối**.
#[tokio::test]
async fn buoc_word_level_khong_ghi_de_no_newline_at_eof() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("noeol_mod"), "no-eol.txt")
        .await
        .expect("tệp không có newline cuối phải trả Ok");

    let dong = dong_cua(&fd);
    let co_co: Vec<&DiffLine> = dong.iter().filter(|l| l.no_newline_at_eof).collect();

    assert!(
        !co_co.is_empty(),
        "phải còn ít nhất một dòng mang `no_newline_at_eof` SAU khi bước word-level \
         chạy. Rỗng nghĩa là bước đó đã ghi đè trường bằng một giá trị suy từ \
         porcelain — và porcelain KHÔNG cung cấp trường này (đã đo: nó in `~` bình \
         thường và không in dòng `\\ No newline at end of file`). \
         Các dòng: {:?}",
        dong.iter()
            .map(|l| (l.kind, l.content.as_str(), l.no_newline_at_eof))
            .collect::<Vec<_>>()
    );

    // Và bước word-level vẫn phải làm việc của nó trên cùng tệp đó.
    assert_eq!(
        so_lenh_word_diff(&state),
        1,
        "tệp này vừa thêm vừa xoá nên lệnh word-diff PHẢI chạy — nếu không, khẳng \
         định trên xanh một cách vô nghĩa vì bước word-level chưa từng chạm vào dữ liệu"
    );
}

// ---------------------------------------------------------------------------
// Chính tả cờ, đọc từ chuỗi tham số THẬT mà hàm dựng ra
// ---------------------------------------------------------------------------

/// **Chuỗi tham số thật phải chứa `--word-diff=porcelain` và KHÔNG chứa
/// `--word-diff-porcelain`.**
///
/// Đọc từ `CommandLog` — tức từ lệnh **đã chạy thật**, không từ mã nguồn. Plan nêu rõ
/// vì sao cổng `grep` trên mã nguồn vô dụng ở đây: chuỗi `--word-diff=porcelain` xuất
/// hiện trong **doc comment** giải thích chính tả đúng, nên `grep -c >= 1` xanh cả khi
/// mã thật dùng sai chính tả.
///
/// Hai khẳng định, không một: khẳng định thứ hai nói cho người sau biết *vì sao* nó
/// đỏ — đúng khuôn mà 02-04 dùng cho `%x1f`.
#[tokio::test]
async fn chuoi_tham_so_that_dung_chinh_ta_dau_bang() {
    let Some((state, repo, fx)) = mo() else { return };

    lay_diff_tep(&state, repo, fx.sha("emptyline_mod"), "empty-line.txt")
        .await
        .expect("phải trả Ok");

    let lenh = cac_lenh(&state);
    let wd: Vec<&String> = lenh.iter().filter(|c| c.contains("word-diff")).collect();
    assert_eq!(
        wd.len(),
        1,
        "đúng một lệnh word-diff. Lệnh đã chạy: {lenh:#?}"
    );

    assert!(
        wd[0].contains("--word-diff=porcelain"),
        "chính tả đúng là `--word-diff=porcelain` với dấu BẰNG. Lệnh thật: {}",
        wd[0]
    );
    assert!(
        !wd[0].contains("--word-diff-porcelain"),
        "`--word-diff-porcelain` (dấu GẠCH NGANG) KHÔNG TỒN TẠI — git thoát 129 và in \
         usage. Tài liệu kế hoạch của phase này viết dạng đó ở ba chỗ; khẳng định này \
         là thứ sẽ đỏ nếu ai đó \"sửa lại\" theo tài liệu. Lệnh thật: {}",
        wd[0]
    );
}

/// **`--unified` của lệnh word-diff phải KHỚP lệnh diff chính.**
///
/// Hai giá trị khác nhau cho hai tập dòng khác nhau ở biên hunk, và phép khớp theo số
/// dòng sẽ trượt đúng ở đó — những dòng ngữ cảnh mà một lệnh thấy còn lệnh kia không.
#[tokio::test]
async fn hai_lenh_diff_dung_cung_muc_unified() {
    let Some((state, repo, fx)) = mo() else { return };

    lay_diff_tep(&state, repo, fx.sha("emptyline_mod"), "empty-line.txt")
        .await
        .expect("phải trả Ok");

    let lenh = cac_lenh(&state);
    let muc = |c: &str| -> Option<String> {
        c.split_whitespace()
            .find(|t| t.starts_with("--unified="))
            .map(|t| t.to_owned())
    };

    let chinh = lenh
        .iter()
        .find(|c| c.contains("--unified=") && !c.contains("word-diff"))
        .and_then(|c| muc(c))
        .expect("phải có lệnh diff chính mang --unified");
    let wd = lenh
        .iter()
        .find(|c| c.contains("word-diff"))
        .and_then(|c| muc(c))
        .expect("lệnh word-diff phải mang --unified");

    assert_eq!(
        chinh, wd,
        "hai lệnh phải dùng CÙNG mức `--unified`. Lệch thì hai đầu ra có tập dòng khác \
         nhau và phép khớp theo số dòng trượt ở biên hunk. Lệnh: {lenh:#?}"
    );
}
