//! Test tích hợp cho `get_file_diff` — plan 03-02, DIFF-01 và DIFF-06.
//!
//! # Vì sao chạy git THẬT trên repo mẫu, không dùng buffer tự dựng
//!
//! Bài học `%x1f` của 02-04, chép nguyên: **19 test đơn vị xanh trên buffer tự dựng**
//! trong khi lệnh git thật trả về một định dạng khác, và thanh bên rỗng hoàn toàn mà
//! không một thông báo lỗi nào. Một test dựng sẵn `stdout` rồi kiểm cách nó được đọc
//! sẽ kiểm đúng cái nó tự bịa ra.
//!
//! Bộ phân tích bản vá đã có test đơn vị riêng (`git::parsers::patch`) — ở đó buffer
//! tự dựng là đúng, vì thứ được kiểm là *hàm thuần*. Ở đây thứ được kiểm là **git có
//! trả về điều ta tưởng không**, và chỉ git thật trả lời được.
//!
//! # Fixture thiếu thì BỎ QUA ồn ào, không đỏ mù mờ
//!
//! `target/fixtures` **đã từng bị dọn mất một lần** (ghi trong 02-07-SUMMARY). Mỗi
//! test bắt đầu bằng `require_diff_fixture()`; thiếu thì in lời nhắc và `return`.

use std::sync::Arc;

use git_plum_lib::commands::diff::{lay_diff_tep, MAX_DIFF_BLOB_BYTES};
use git_plum_lib::domain::diff::{DiffKind, LineKind};
use git_plum_lib::state::AppState;
use git_plum_lib::testing::{require_diff_fixture, DiffFixture};

/// Mở repo mẫu trong một `AppState` sạch. Trả `(state, repo, fixture)`.
///
/// `AppState` mới cho mỗi test là bắt buộc, không phải cẩn thận thừa: cache diff và
/// `CommandLog` đều sống trong đó, và hai test dùng chung một state sẽ thấy nhau qua
/// cache — test đếm entry nhật ký thành ngẫu nhiên tuỳ thứ tự chạy.
fn mo() -> Option<(AppState, Arc<git_plum_lib::state::RepoHandle>, DiffFixture)> {
    let fx = require_diff_fixture()?;
    let state = AppState::new();
    let repo = state.open_repo(&fx.repo);
    Some((state, repo, fx))
}

/// Số entry trong nhật ký lệnh — mỗi entry là một tiến trình git **đã chạy**.
fn so_lenh(state: &AppState) -> usize {
    state.command_log.entries().len()
}

// ---------------------------------------------------------------------------
// Ca văn bản thường
// ---------------------------------------------------------------------------

#[tokio::test]
async fn text_simple_tra_hunk_da_phan_tich() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("text_simple"), "text-simple.txt")
        .await
        .expect("tệp văn bản bị sửa phải trả Ok");

    let DiffKind::Text {
        hunks,
        truncated,
        context_only,
    } = &fd.kind
    else {
        panic!("phải là kind `text`, nhận: {:?}", fd.kind);
    };
    assert!(!hunks.is_empty(), "phải có ít nhất một hunk");
    assert!(!truncated, "bản vá nhỏ không được bị cắt");
    // `text-simple.txt` là tệp mẫu vài dòng, tức dưới `MAX_BYTE_TOAN_TEP` (512 KB) rất
    // xa — nên nó phải đi đường **toàn tệp**. `context_only == true` ở đây nghĩa là
    // ngưỡng đang kích hoạt sai và mọi tệp bình thường bị rút gọn.
    assert!(
        !context_only,
        "tệp mẫu vài dòng phải hiện toàn tệp, không phải diff rút gọn"
    );
    assert_eq!(fd.path, "text-simple.txt");

    // Giao diện dựng hai cột từ số dòng — kiểm chúng có mặt, không chỉ kiểm có hunk.
    let co_added = hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| l.kind == LineKind::Added && l.new_line.is_some() && l.old_line.is_none());
    assert!(co_added, "phải có dòng `added` mang số dòng MỚI và không mang số dòng cũ");
}

// ---------------------------------------------------------------------------
// DIFF-06: tệp nhị phân
// ---------------------------------------------------------------------------

/// `kind: binary` với kích thước hai phía, và **payload JSON không chứa byte nội dung
/// nào** — nửa sau của DIFF-06: "không đổ nội dung thô ra màn hình".
///
/// Kiểm bằng độ dài JSON: một cài đặt lỡ nhúng nội dung sẽ vượt xa 1 KB. Đây là phép
/// kiểm ở tầng **quan sát được**, bổ sung cho `DiffKind::Binary` không có trường nội
/// dung ở tầng kiểu (T-03-13).
#[tokio::test]
async fn binary_tra_kich_thuoc_va_khong_co_noi_dung() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("binary_mod"), "binary.png")
        .await
        .expect("tệp nhị phân phải trả Ok, không phải lỗi");

    let DiffKind::Binary { old_size, new_size } = fd.kind else {
        panic!("phải là kind `binary`, nhận: {:?}", fd.kind);
    };
    assert!(old_size > 0, "phía cũ phải có kích thước thật");
    assert!(new_size > 0, "phía mới phải có kích thước thật");

    let json = serde_json::to_string(&*fd).expect("phải serialize được");
    assert!(
        json.len() < 1024,
        "payload của tệp nhị phân phải nhỏ — nó chỉ mang hai con số. Dài {} byte, \
         nghĩa là nội dung tệp đã lọt vào JSON và đi lên webview (T-03-13). \
         Payload: {json}",
        json.len()
    );
    assert!(
        !json.contains("PNG"),
        "chữ ký PNG không được xuất hiện trong payload"
    );
}

// ---------------------------------------------------------------------------
// DIFF-06: tệp vượt ngưỡng — cổng phải chạy TRƯỚC, không lọc SAU
// ---------------------------------------------------------------------------

/// **Mutation quan trọng nhất của plan này.**
///
/// Một cài đặt "lọc sau" (chạy `git diff` rồi mới xem kích thước) trả về **đúng**
/// `kind: tooLarge` với **đúng** `size` và `limit`. Mọi test kiểm *giá trị trả về* đều
/// xanh với nó. Khác biệt duy nhất quan sát được là: nó đã chạy `git diff` trên một
/// tệp 6 MB, tức đã đọc 6 MB và bắt người dùng chờ — đúng thứ DIFF-06 cấm.
///
/// Nên test đọc `CommandLog`: **không** entry nào được vừa chứa `diff` vừa chứa
/// `large.txt`.
#[tokio::test]
async fn too_large_khong_chay_git_diff_nao_tren_tep_do() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("large_mod"), "large.txt")
        .await
        .expect("tệp lớn phải trả Ok kèm thông báo, không phải lỗi");

    let DiffKind::TooLarge { size, limit } = fd.kind else {
        panic!("phải là kind `tooLarge`, nhận: {:?}", fd.kind);
    };
    assert_eq!(limit, MAX_DIFF_BLOB_BYTES, "limit là ngưỡng đang áp");
    assert_eq!(limit, 5 * 1024 * 1024);
    assert!(
        size > limit,
        "size phải là kích thước THẬT của tệp ({size}), lớn hơn ngưỡng ({limit})"
    );

    let pham_quy: Vec<String> = state
        .command_log
        .entries()
        .into_iter()
        .map(|e| e.command)
        .filter(|c| c.contains("diff") && c.contains("large.txt"))
        .collect();
    assert!(
        pham_quy.is_empty(),
        "KHÔNG lệnh `git diff` nào được chạy trên tệp vượt ngưỡng — cổng phải chặn \
         TRƯỚC, không lọc SAU. Lệnh đã chạy: {pham_quy:#?}"
    );

    // Tiền đề: nhật ký PHẢI có ghi lệnh, nếu không phép khẳng định trên luôn đúng
    // một cách vô nghĩa (cổng tự vô hiệu hoá).
    assert!(
        so_lenh(&state) > 0,
        "nhật ký phải ghi được lệnh (rev-parse, cat-file); nhật ký rỗng làm phép \
         khẳng định ở trên luôn xanh bất kể cài đặt"
    );
}

// ---------------------------------------------------------------------------
// DIFF-06: con trỏ Git LFS
// ---------------------------------------------------------------------------

/// `size` là số **trong con trỏ** (1048576), không phải kích thước của tệp con trỏ
/// (~130 byte). Hai số khác nhau, và lẫn chúng là lỗi dễ mắc nhất ở đây — nó im lặng
/// vì giao diện vẫn hiện một con số, chỉ là sai con số.
#[tokio::test]
async fn lfs_pointer_doc_oid_va_size_tu_noi_dung_con_tro() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("lfs"), "pointer.bin")
        .await
        .expect("con trỏ LFS phải trả Ok");

    let DiffKind::LfsPointer { oid, size } = &fd.kind else {
        panic!("phải là kind `lfsPointer`, nhận: {:?}", fd.kind);
    };

    assert_eq!(oid.len(), 64, "oid sha256 là 64 ký tự hex, nhận: {oid:?}");
    assert!(
        oid.chars().all(|c| c.is_ascii_hexdigit()),
        "oid phải toàn hex, nhận: {oid:?}"
    );
    assert_eq!(
        *size, 1_048_576,
        "size phải là số TRONG con trỏ (kích thước tệp thật), KHÔNG phải kích thước \
         của tệp con trỏ (~130 byte)"
    );
    assert!(
        *size > 1000,
        "một `size` cỡ trăm byte nghĩa là đã lấy nhầm kích thước của chính tệp con trỏ"
    );
}

// ---------------------------------------------------------------------------
// CRLF
// ---------------------------------------------------------------------------

#[tokio::test]
async fn crlf_khong_de_lot_ky_tu_cr_vao_noi_dung() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("crlf_mod"), "crlf.txt")
        .await
        .expect("tệp CRLF phải trả Ok");

    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("phải là kind `text`, nhận: {:?}", fd.kind);
    };

    let dong: Vec<&str> = hunks
        .iter()
        .flat_map(|h| &h.lines)
        .map(|l| l.content.as_str())
        .collect();
    assert!(!dong.is_empty(), "tiền đề: phải có dòng để kiểm");
    for d in &dong {
        assert!(
            !d.ends_with('\r'),
            "không `content` nào được kết thúc bằng `\\r` — ký tự vô hình làm lệch \
             mọi phép so nội dung. Nhận: {d:?}"
        );
    }
    assert!(
        dong.iter().any(|d| d.contains("BETA DA SUA")),
        "tiền đề: phải thấy dòng đã sửa; nhận {dong:?}"
    );
}

// ---------------------------------------------------------------------------
// `\ No newline at end of file`
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_eol_dat_co_tren_dung_dong() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("noeol_mod"), "no-eol.txt")
        .await
        .expect("tệp không có newline cuối phải trả Ok");

    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("phải là kind `text`, nhận: {:?}", fd.kind);
    };

    let co_co: Vec<&str> = hunks
        .iter()
        .flat_map(|h| &h.lines)
        .filter(|l| l.no_newline_at_eof)
        .map(|l| l.content.as_str())
        .collect();

    assert!(
        !co_co.is_empty(),
        "phải có ít nhất một dòng mang `noNewlineAtEof`"
    );

    // Dòng `\ No newline...` không được trở thành một dòng nội dung.
    let co_dong_gia = hunks
        .iter()
        .flat_map(|h| &h.lines)
        .any(|l| l.content.starts_with(" No newline") || l.content.contains("No newline at end"));
    assert!(
        !co_dong_gia,
        "`\\ No newline at end of file` KHÔNG được trở thành một DiffLine — nó là \
         siêu dữ liệu về dòng trước nó"
    );
}

// ---------------------------------------------------------------------------
// Đổi tên + sửa
// ---------------------------------------------------------------------------

#[tokio::test]
async fn doi_ten_va_sua_tra_status_r_kem_old_path() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("rename_mod"), "renamed-new.txt")
        .await
        .expect("tệp vừa đổi tên vừa sửa phải trả Ok");

    assert!(
        fd.status.starts_with('R'),
        "status phải bắt đầu bằng `R`, nhận: {:?}",
        fd.status
    );
    assert_eq!(
        fd.old_path.as_deref(),
        Some("renamed.txt"),
        "oldPath là tên CŨ"
    );
    assert_eq!(fd.path, "renamed-new.txt", "path là tên MỚI");

    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("vừa đổi tên vừa sửa thì phải có hunk, nhận: {:?}", fd.kind);
    };
    assert!(
        !hunks.is_empty(),
        "đổi tên KÈM sửa nội dung phải có hunk — chỉ đổi tên mới cho 0 hunk"
    );

    // **Phần quan trọng hơn cả chữ trạng thái.** Nếu pathspec chỉ có tên mới thì git
    // không nhận ra đổi tên và in `new file mode` với TOÀN BỘ 8 dòng là dòng thêm.
    // Giao diện sẽ hiện cả tệp sáng xanh thay vì hiện một dòng đã sửa. Kiểm bằng
    // cách đếm: bản vá thật có đúng một dòng thêm và một dòng xoá.
    let dong: Vec<_> = hunks.iter().flat_map(|h| &h.lines).collect();
    let them = dong.iter().filter(|l| l.kind == LineKind::Added).count();
    let xoa = dong.iter().filter(|l| l.kind == LineKind::Removed).count();
    let ngu_canh = dong.iter().filter(|l| l.kind == LineKind::Context).count();

    assert_eq!(
        (them, xoa),
        (1, 1),
        "tệp đổi tên KÈM sửa một dòng phải cho đúng 1 dòng thêm và 1 dòng xoá. \
         Nhận ({them} thêm, {xoa} xoá) — con số kiểu (8, 0) nghĩa là git đã coi đây \
         là tệp mới hoàn toàn, tức pathspec đã lọc mất tên cũ trước khi phép phát \
         hiện đổi tên chạy"
    );
    assert!(
        ngu_canh > 0,
        "phải có dòng ngữ cảnh — bản vá của một tệp mới hoàn toàn không có dòng nào"
    );
}

// ---------------------------------------------------------------------------
// Commit gốc và tệp bị xoá
// ---------------------------------------------------------------------------

/// Commit **gốc** không có `<sha>^`. Cài đặt chỉ nối `^` sẽ báo lỗi ở đúng commit đầu
/// tiên của mọi repo.
#[tokio::test]
async fn commit_goc_tra_ok_voi_moi_dong_added() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("root"), "text-simple.txt")
        .await
        .expect("commit gốc phải trả Ok — so với cây rỗng, không phải Err");

    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("phải là kind `text`, nhận: {:?}", fd.kind);
    };
    let dong: Vec<_> = hunks.iter().flat_map(|h| &h.lines).collect();
    assert!(!dong.is_empty(), "commit gốc vẫn phải có hunk");
    assert!(
        dong.iter().all(|l| l.kind == LineKind::Added),
        "mọi dòng của tệp mới thêm phải là `added`"
    );
    assert!(
        dong.iter().all(|l| l.old_line.is_none()),
        "không dòng nào có số dòng cũ"
    );
}

#[tokio::test]
async fn tep_bi_xoa_tra_moi_dong_removed() {
    let Some((state, repo, fx)) = mo() else { return };

    let fd = lay_diff_tep(&state, repo, fx.sha("deleted"), "deleted.txt")
        .await
        .expect("tệp bị xoá phải trả Ok, không phải lỗi");

    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("phải là kind `text`, nhận: {:?}", fd.kind);
    };
    let dong: Vec<_> = hunks.iter().flat_map(|h| &h.lines).collect();
    assert!(!dong.is_empty(), "tệp bị xoá vẫn phải có hunk");
    assert!(
        dong.iter().all(|l| l.kind == LineKind::Removed),
        "mọi dòng của tệp bị xoá phải là `removed`"
    );
    assert!(
        dong.iter().all(|l| l.new_line.is_none()),
        "không dòng nào có số dòng mới"
    );
}

// ---------------------------------------------------------------------------
// Cache: tiêu chí thành công số 5 của phase
// ---------------------------------------------------------------------------

/// Gọi hai lần cùng `(sha, path)` → lần hai thêm **0** entry vào `CommandLog`.
///
/// Đây là cách duy nhất chứng minh tiêu chí thành công số 5 ("chọn đi chọn lại: hiện
/// ra tức thì, không tính lại"). Đo thời gian thì nhiễu — một lần chạy nhanh có thể
/// chỉ là bộ nhớ đệm hệ điều hành đã ấm.
#[tokio::test]
async fn goi_lan_hai_khong_sinh_tien_trinh_git_nao() {
    let Some((state, repo, fx)) = mo() else { return };
    let sha = fx.sha("text_simple").to_owned();

    let mot = lay_diff_tep(&state, Arc::clone(&repo), &sha, "text-simple.txt")
        .await
        .expect("lần một phải chạy được");
    let sau_lan_mot = so_lenh(&state);
    assert!(
        sau_lan_mot > 0,
        "tiền đề: lần một PHẢI sinh lệnh git, nếu không phép so bên dưới vô nghĩa"
    );

    let hai = lay_diff_tep(&state, repo, &sha, "text-simple.txt")
        .await
        .expect("lần hai phải chạy được");
    let sau_lan_hai = so_lenh(&state);

    assert_eq!(
        sau_lan_hai, sau_lan_mot,
        "lần gọi thứ hai cho cùng (sha, path) phải thêm ĐÚNG 0 entry vào nhật ký. \
         Thêm {} entry nghĩa là cache không được tra, và tiêu chí thành công số 5 \
         của phase trượt.",
        sau_lan_hai - sau_lan_mot
    );
    assert!(
        Arc::ptr_eq(&mot, &hai),
        "lần hai phải trả về CÙNG một Arc — clone dữ liệu ở đây biến cache \
         'tránh tính lại' thành 'tránh gọi git nhưng vẫn sao chép hàng MB'"
    );
}

/// Cache phân biệt hai tệp khác nhau trong **cùng một** commit.
#[tokio::test]
async fn cache_khong_tra_nham_giua_hai_tep_cung_commit() {
    let Some((state, repo, fx)) = mo() else { return };
    let sha = fx.sha("mixed_mod").to_owned();

    let a = lay_diff_tep(&state, Arc::clone(&repo), &sha, "mixed.txt")
        .await
        .expect("mixed.txt phải đọc được");
    let b = lay_diff_tep(&state, repo, &sha, "text-simple.txt")
        .await
        .expect("text-simple.txt phải đọc được");

    assert_eq!(a.path, "mixed.txt");
    assert_eq!(
        b.path, "text-simple.txt",
        "cùng sha khác path phải là hai mục cache riêng"
    );
}

// ---------------------------------------------------------------------------
// Ca lỗi: path không tồn tại
// ---------------------------------------------------------------------------

/// "Tệp không tồn tại" **khác** "tệp không đổi", và giao diện phải nói khác nhau.
/// Trả `Text` với 0 hunk cho ca đầu làm người dùng tưởng tệp không thay đổi.
#[tokio::test]
async fn path_khong_ton_tai_tra_loi_doc_hieu_duoc() {
    let Some((state, repo, fx)) = mo() else { return };

    let ra = lay_diff_tep(
        &state,
        repo,
        fx.sha("text_simple"),
        "khong-he-co-tep-nay.txt",
    )
    .await;

    let err = ra.expect_err("path không tồn tại phải trả Err, không phải Text với 0 hunk");
    let msg = err.to_string();
    assert!(
        msg.contains("khong-he-co-tep-nay.txt"),
        "thông báo lỗi phải nêu tên tệp để người dùng hiểu chuyện gì xảy ra, nhận: {msg:?}"
    );
}

// ---------------------------------------------------------------------------
// Tên tệp không UTF-8 — HIST-11 ở tầng diff
// ---------------------------------------------------------------------------

/// Tên tệp chứa byte 0xFF không được làm panic. Repo thật trên Linux có những tên này.
#[tokio::test]
async fn ten_tep_khong_utf8_khong_panic() {
    let Some((state, repo, fx)) = mo() else { return };

    // Tên đi qua ranh giới IPC nên nó là `String` — byte 0xFF đã thành U+FFFD. Dùng
    // đúng dạng đó: nó là dạng mà giao diện thật sự gửi xuống, và ca đáng kiểm là
    // "không panic", không phải "tìm được tệp".
    let ten = String::from_utf8_lossy(b"ten-\xff-xau.txt").into_owned();
    let ra = lay_diff_tep(&state, repo, fx.sha("nonutf8_mod"), &ten).await;

    // Kết quả nào cũng chấp nhận được (Ok nếu git khớp được pathspec, Err nếu không).
    // Điều KHÔNG chấp nhận được là panic — và nếu có, test này đã không chạy tới đây.
    match ra {
        Ok(fd) => assert_eq!(fd.path, ten),
        Err(e) => assert!(
            !e.to_string().is_empty(),
            "lỗi phải có thông điệp đọc được, không phải chuỗi rỗng"
        ),
    }
}

// ---------------------------------------------------------------------------
// Fixture của wave 3: kiểm CHÚNG TỒN TẠI và đọc được qua đường này
// ---------------------------------------------------------------------------

/// Ba fixture mà 03-03 (diff mức từ) phụ thuộc phải đọc được qua `get_file_diff`.
///
/// Sinh ở plan này vì 03-03 **không** có `make-diff-fixtures.sh` trong `files_modified`
/// của nó. Kiểm ở đây để wave 3 không phát hiện fixture thiếu hay sai hình dạng sau
/// khi đã viết nửa plan.
#[tokio::test]
async fn fixture_cho_wave_3_doc_duoc_va_dung_hinh_dang() {
    let Some((state, repo, fx)) = mo() else { return };

    // `empty-line.txt`: phải có một dòng ngữ cảnh RỖNG.
    let fd = lay_diff_tep(
        &state,
        Arc::clone(&repo),
        fx.sha("emptyline_mod"),
        "empty-line.txt",
    )
    .await
    .expect("empty-line.txt phải đọc được");
    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("empty-line.txt phải là text");
    };
    assert!(
        hunks
            .iter()
            .flat_map(|h| &h.lines)
            .any(|l| l.kind == LineKind::Context && l.content.is_empty()),
        "empty-line.txt phải cho một dòng ngữ cảnh RỖNG — đó là lý do nó tồn tại \
         (quy tắc đẩy ở 03-03 không được dùng `len() > 0`)"
    );

    // `one-side.txt`: có cả dòng chỉ-thêm và dòng chỉ-xoá.
    let fd = lay_diff_tep(
        &state,
        Arc::clone(&repo),
        fx.sha("oneside_mod"),
        "one-side.txt",
    )
    .await
    .expect("one-side.txt phải đọc được");
    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("one-side.txt phải là text");
    };
    let loai: Vec<LineKind> = hunks.iter().flat_map(|h| &h.lines).map(|l| l.kind).collect();
    assert!(
        loai.contains(&LineKind::Added) && loai.contains(&LineKind::Removed),
        "one-side.txt phải có CẢ dòng thêm lẫn dòng xoá, nhận: {loai:?}"
    );

    // `mixed.txt`: dòng rỗng CỘNG dòng một-phía — fixture DUY NHẤT phân biệt được cả
    // ba quy tắc đẩy của 03-03.
    let fd = lay_diff_tep(&state, Arc::clone(&repo), fx.sha("mixed_mod"), "mixed.txt")
        .await
        .expect("mixed.txt phải đọc được");
    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("mixed.txt phải là text");
    };
    let dong: Vec<_> = hunks.iter().flat_map(|h| &h.lines).collect();
    assert!(
        dong.iter()
            .any(|l| l.kind == LineKind::Context && l.content.is_empty()),
        "mixed.txt phải có dòng ngữ cảnh rỗng"
    );
    assert!(
        dong.iter().any(|l| l.kind == LineKind::Removed),
        "mixed.txt phải có dòng chỉ ở phía cũ (DELETED)"
    );
    assert!(
        dong.iter().any(|l| l.kind == LineKind::Added),
        "mixed.txt phải có dòng chỉ ở phía mới (ADDED)"
    );

    // `dup-lines.txt`: hai dòng nội dung TRÙNG NHAU, cần cho mutation #9 của 03-03.
    let fd = lay_diff_tep(&state, repo, fx.sha("dup_mod"), "dup-lines.txt")
        .await
        .expect("dup-lines.txt phải đọc được");
    let DiffKind::Text { hunks, .. } = &fd.kind else {
        panic!("dup-lines.txt phải là text");
    };
    let noi_dung: Vec<&str> = hunks
        .iter()
        .flat_map(|h| &h.lines)
        .map(|l| l.content.as_str())
        .collect();
    let so_trung = noi_dung.iter().filter(|c| c.trim() == "TRUNG").count();
    assert!(
        so_trung >= 1,
        "dup-lines.txt phải còn ít nhất một dòng `TRUNG` nguyên vẹn trong hunk — \
         mutation #9 của 03-03 cần hai dòng giống nhau để phân biệt khớp-theo-nội-dung \
         với khớp-theo-số-dòng. Nhận: {noi_dung:?}"
    );
}
