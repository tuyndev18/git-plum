//! Test tích hợp cho `parse_word_diff` — plan 03-03, diff mức **từ**.
//!
//! # Vì sao chạy git THẬT, không chỉ test đơn vị trên buffer tự dựng
//!
//! `git::parsers::word_diff` đã có 20 test đơn vị, và ở đó buffer tự dựng là **đúng**
//! — thứ được kiểm là một hàm thuần trên một định dạng. Ở đây thứ được kiểm là một
//! câu hỏi khác hẳn: **hai bộ phân tích có đồng ý với nhau không** khi cùng đọc đầu
//! ra của cùng một commit.
//!
//! Đó là bất biến quan trọng nhất của plan 03-03: `Span` là chỉ số byte vào
//! `DiffLine::content`, và `DiffLine` đến từ `parse_patch` (unified) trong khi `Span`
//! đến từ `parse_word_diff` (porcelain). Hai **định dạng khác nhau**, hai bộ phân tích
//! khác nhau, một phép cắt tiền tố mỗi bên. Lệch một byte ở bất kỳ đâu và mọi khoảng
//! tô sai chỗ — hoặc panic khi chỉ số vượt biên.
//!
//! Chỉ git thật trả lời được câu đó. Bài học `%x1f` của 02-04, chép nguyên: 19 test
//! đơn vị xanh trên buffer tự dựng trong khi lệnh git thật trả về một định dạng khác.
//!
//! # Fixture thiếu thì BỎ QUA ồn ào, không đỏ mù mờ
//!
//! `target/fixtures` đã từng bị dọn mất một lần (02-07-SUMMARY).

use std::process::Command;

use git_plum_lib::domain::diff::LineKind;
use git_plum_lib::git::parsers::patch::parse_patch;
use git_plum_lib::git::parsers::word_diff::{parse_word_diff, WORD_DIFF_REGEX};
use git_plum_lib::testing::{require_diff_fixture, DiffFixture};

/// Chạy `git diff` thật trên repo mẫu và trả **stdout thô**.
///
/// Gọi `git` trực tiếp qua `std::process::Command` chứ không qua `GitRunner`: ở đây ta
/// cần đúng byte mà git in ra, không cần nhật ký lệnh hay cache. `stderr` bị **vứt
/// bỏ** có chủ ý — git in cảnh báo CRLF ra đó, và trộn nó vào stdout sẽ được đọc
/// thành một đoạn từ giả (T-03-22).
fn git_diff(fx: &DiffFixture, them: &[&str], cha: &str, con: &str, path: &str) -> Vec<u8> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(&fx.repo)
        .args(["-c", "core.quotepath=false", "diff", "--unified=3"])
        .args(them)
        .args([cha, con, "--", path]);

    let ra = cmd.output().expect("phải chạy được `git diff`");
    assert!(
        ra.status.success(),
        "`git diff` thất bại trên {path}: {}",
        String::from_utf8_lossy(&ra.stderr)
    );
    ra.stdout
}

/// Hai cờ của lệnh word-diff, dựng từ **cùng** hằng mà bộ phân tích công bố.
///
/// Không gõ lại chuỗi regex ở đây: hai bản sao ở hai tệp là đúng lớp lỗi mà Phase 2
/// phải ghim bằng test đọc chéo. Nếu ai đó đổi [`WORD_DIFF_REGEX`], test này đi theo
/// và vẫn kiểm đúng thứ mã thật dùng.
fn co_word_diff() -> [String; 2] {
    [
        "--word-diff=porcelain".to_owned(),
        format!("--word-diff-regex={WORD_DIFF_REGEX}"),
    ]
}

/// SHA cha của một commit.
fn cha_cua(fx: &DiffFixture, sha: &str) -> String {
    let ra = Command::new("git")
        .arg("-C")
        .arg(&fx.repo)
        .args(["rev-parse", &format!("{sha}^")])
        .output()
        .expect("phải chạy được `git rev-parse`");
    assert!(ra.status.success(), "commit {sha} phải có cha");
    String::from_utf8_lossy(&ra.stdout).trim().to_owned()
}

/// 🔴 **Bất biến quan trọng nhất của plan 03-03.**
///
/// `content` mà `parse_word_diff` dựng lại từ porcelain phải bằng **TỪNG BYTE** với
/// `content` mà `parse_patch` cho ra từ unified diff, trên cùng commit và cùng tệp.
///
/// Vì sao đây là bất biến chứ không phải một phép kiểm tuỳ chọn: `Span` là chỉ số
/// **vào `DiffLine::content`**. Nếu `parse_word_diff` dựng ra một chuỗi khác dù chỉ
/// một byte (thiếu một dấu cách thụt lề, còn một `\r`, mất một ký tự UTF-8), thì mọi
/// khoảng tính trên chuỗi của nó sẽ trỏ sai chỗ khi áp lên chuỗi của `parse_patch`.
/// Ở ca nhẹ nhất là tô lệch vài ký tự; ở ca nặng là `&content[start..end]` panic.
///
/// Chạy trên **nhiều hình dạng tệp**, không chỉ một: ca thụt lề, ca dòng rỗng, ca
/// CRLF, ca UTF-8, ca không newline cuối. Mỗi ca là một cách khác nhau để hai phép
/// cắt tiền tố lệch nhau.
#[test]
fn noi_dung_dung_lai_bang_tung_byte_voi_parse_patch() {
    let Some(fx) = require_diff_fixture() else {
        return;
    };

    // (nhãn commit, đường dẫn, vì sao ca này có mặt)
    let ca: &[(&str, &str, &str)] = &[
        ("text_simple", "text-simple.txt", "ca thường"),
        (
            "emptyline_mod",
            "empty-line.txt",
            "dòng RỖNG: porcelain in một đoạn `␠` nội dung rỗng",
        ),
        (
            "oneside_mod",
            "one-side.txt",
            "dòng chỉ có ở MỘT phía: một bên không có dòng ở vị trí đó",
        ),
        (
            "mixed_mod",
            "mixed.txt",
            "dòng rỗng CỘNG dòng một-phía trong cùng tệp",
        ),
        (
            "dup_mod",
            "dup-lines.txt",
            "hai dòng nội dung TRÙNG NHAU trong một hunk",
        ),
        (
            "crlf_mod",
            "crlf.txt",
            "CRLF: porcelain tách `\\r` thành một đoạn từ RIÊNG",
        ),
        (
            "noeol_mod",
            "no-eol.txt",
            "không newline cuối: porcelain in `~` nhưng KHÔNG in dòng báo hiệu",
        ),
    ];

    let mut da_kiem = 0usize;

    for (nhan, path, vi_sao) in ca {
        let con = fx.sha(nhan);
        let cha = cha_cua(&fx, con);

        let co = co_word_diff();
        let unified = git_diff(&fx, &[], &cha, con, path);
        let porcelain = git_diff(
            &fx,
            &[co[0].as_str(), co[1].as_str()],
            &cha,
            con,
            path,
        );

        let pp = parse_patch(&unified);
        let wd = parse_word_diff(&porcelain);

        assert_eq!(
            wd.skipped, 0,
            "[{nhan}] đầu ra porcelain thật phải phân tích được hết — `skipped = {}` \
             nghĩa là bộ phân tích gặp một khuôn nó không biết, và tầng trên sẽ bỏ \
             TOÀN BỘ spans của tệp này.\nCa: {vi_sao}",
            wd.skipped
        );

        // Ghép theo (phía, số dòng) — đúng cách tầng trên khớp, nên test này cũng
        // ghim luôn rằng phép khớp đó tìm thấy thứ nó cần tìm.
        for dong in pp.hunks.iter().flat_map(|h| &h.lines) {
            let (ds, so) = match dong.kind {
                LineKind::Removed => (&wd.old_lines, dong.old_line),
                LineKind::Added => (&wd.new_lines, dong.new_line),
                // Dòng ngữ cảnh không cần khoảng, nhưng vẫn kiểm nội dung: nó là
                // phép kiểm rẻ nhất cho việc dựng lại, và nếu dòng ngữ cảnh lệch thì
                // dòng đã sửa gần như chắc chắn cũng lệch.
                LineKind::Context => (&wd.new_lines, dong.new_line),
            };
            let Some(so) = so else { continue };
            let Some(w) = ds.iter().find(|w| w.line_no == so) else {
                panic!(
                    "[{nhan}] không tìm thấy WordLine cho dòng {so} phía {:?} — \
                     porcelain và unified bất đồng về TẬP dòng, không chỉ về nội \
                     dung. Nội dung mà parse_patch cho: {:?}\nCa: {vi_sao}",
                    dong.kind, dong.content
                );
            };

            assert_eq!(
                w.content, dong.content,
                "[{nhan}] dòng {so} phía {:?} LỆCH giữa hai bộ phân tích.\n  \
                 porcelain dựng lại : {:?}\n  \
                 unified cho        : {:?}\n\
                 `Span` là chỉ số byte vào chuỗi của unified, nên một byte lệch ở đây \
                 làm MỌI khoảng trên dòng này trỏ sai chỗ — hoặc panic nếu chỉ số \
                 vượt biên (T-03-17).\nCa: {vi_sao}",
                dong.kind, w.content, dong.content
            );

            // Bất biến biên ký tự, kiểm trên **chuỗi của unified** — đó là chuỗi mà
            // 03-04 thật sự cắt.
            for s in &w.spans {
                assert!(
                    s.end <= dong.content.len(),
                    "[{nhan}] khoảng [{}, {}) vượt biên chuỗi dài {} byte (T-03-17)",
                    s.start,
                    s.end,
                    dong.content.len()
                );
                assert!(
                    dong.content.is_char_boundary(s.start)
                        && dong.content.is_char_boundary(s.end),
                    "[{nhan}] khoảng [{}, {}) cắt giữa một ký tự của {:?} (T-03-18)",
                    s.start,
                    s.end,
                    dong.content
                );
            }
            da_kiem += 1;
        }
    }

    // Tiền đề: vòng lặp trên PHẢI đi qua dòng thật. Không có khẳng định này thì một
    // `parse_patch` trả 0 hunk làm cả test xanh mà không kiểm gì — cổng tự vô hiệu hoá.
    assert!(
        da_kiem >= 20,
        "tiền đề: phải kiểm được ít nhất 20 dòng trên bảy ca; chỉ kiểm {da_kiem} \
         nghĩa là fixture hoặc phép ghép đã hỏng và mọi khẳng định trên đều vô nghĩa"
    );
}

/// Ca của **chủ dự án** trên git THẬT, không phải trên buffer tự dựng.
///
/// `empty-line.txt` mang đúng hình dạng chủ dự án mô tả: `x == 1` → `x === 1`. Đây là
/// phép kiểm đầu-tới-cuối rằng chuỗi lệnh git → porcelain → khoảng byte cho đúng thứ
/// được yêu cầu, chứ không chỉ đúng trên một chuỗi ta tự gõ.
#[test]
fn ca_chu_du_an_tren_git_that_cho_khoang_hep_hon_ca_dong() {
    let Some(fx) = require_diff_fixture() else {
        return;
    };

    let con = fx.sha("emptyline_mod");
    let cha = cha_cua(&fx, con);
    let co = co_word_diff();
    let porcelain = git_diff(
        &fx,
        &[co[0].as_str(), co[1].as_str()],
        &cha,
        con,
        "empty-line.txt",
    );
    let wd = parse_word_diff(&porcelain);

    assert_eq!(wd.skipped, 0);

    let sua = wd
        .new_lines
        .iter()
        .find(|w| w.content.contains("==="))
        .unwrap_or_else(|| {
            panic!(
                "phải tìm được dòng đã sửa chứa `===`. Các dòng: {:?}",
                wd.new_lines
                    .iter()
                    .map(|w| (w.line_no, w.content.as_str()))
                    .collect::<Vec<_>>()
            )
        });

    assert_eq!(sua.spans.len(), 1, "đúng một khoảng, nhận: {:?}", sua.spans);
    let s = sua.spans[0];
    assert_eq!(
        &sua.content[s.start..s.end],
        "===",
        "khoảng phải trỏ đúng vào `===`, không vào cả dòng"
    );

    // 🔴 Khẳng định mà chủ dự án nêu tường minh: "không chỉ tô cả dòng".
    assert!(
        s.end - s.start < sua.content.len(),
        "khoảng phải HẸP HƠN cả dòng. Nhận [{}, {}) trên dòng {:?} dài {} byte — một \
         khoảng phủ toàn dòng qua được mọi phép kiểm \"có span\" nhưng làm sai đúng \
         điều chủ dự án yêu cầu",
        s.start,
        s.end,
        sua.content,
        sua.content.len()
    );
    assert!(
        s.start > 0,
        "khoảng phải bắt đầu SAU phần `  x ` không đổi; bắt đầu ở 0 nghĩa là đã tô \
         từ đầu dòng"
    );

    // Phía cũ trỏ vào `==`, tức hai phía nói về cùng một chỗ trong dòng.
    let cu = wd
        .old_lines
        .iter()
        .find(|w| w.content.contains("=="))
        .expect("phía cũ phải có dòng chứa `==`");
    assert_eq!(cu.spans.len(), 1);
    let sc = cu.spans[0];
    assert_eq!(&cu.content[sc.start..sc.end], "==");
    assert!(sc.end - sc.start < cu.content.len());
}

/// **Chính tả cờ**: `--word-diff=porcelain` chạy, `--word-diff-porcelain` **thoát
/// khác 0 và in usage**.
///
/// Ghim bằng cách chạy git thật cả hai dạng. Tài liệu kế hoạch của phase này viết dạng
/// gạch ngang ở ba chỗ; test này là thứ sẽ đỏ nếu ai đó "sửa lại" theo tài liệu.
///
/// Hai khẳng định, không một: khẳng định thứ hai nói cho người sau biết *vì sao* —
/// đúng khuôn mà 02-04 dùng cho `%x1f`.
#[test]
fn chinh_ta_co_word_diff_dang_bang_chu_khong_phai_gach_ngang() {
    let Some(fx) = require_diff_fixture() else {
        return;
    };

    let con = fx.sha("text_simple").to_owned();
    let cha = cha_cua(&fx, &con);

    let chay = |co: &str| {
        Command::new("git")
            .arg("-C")
            .arg(&fx.repo)
            .args(["diff", co, &cha, &con, "--", "text-simple.txt"])
            .output()
            .expect("phải chạy được git")
    };

    let dung = chay("--word-diff=porcelain");
    assert!(
        dung.status.success(),
        "`--word-diff=porcelain` (dấu BẰNG) phải chạy được. stderr: {}",
        String::from_utf8_lossy(&dung.stderr)
    );
    assert!(
        dung.stdout.windows(1).any(|b| b == b"~"),
        "đầu ra porcelain phải chứa ký tự `~` đánh dấu hết dòng nguồn"
    );

    let sai = chay("--word-diff-porcelain");
    assert!(
        !sai.status.success(),
        "`--word-diff-porcelain` (dấu GẠCH NGANG) KHÔNG tồn tại — git phải thoát khác \
         0 và in usage. Nó chạy được nghĩa là phiên bản git đã đổi hành vi và tài liệu \
         của `git::parsers::word_diff` cần đo lại"
    );
    assert!(
        String::from_utf8_lossy(&sai.stderr).contains("usage: git diff"),
        "git phải in usage khi gặp cờ không tồn tại — đây là lý do lỗi này ỒN (dễ \
         thấy) chứ không im lặng như `%x1f` của 02-04. stderr: {}",
        String::from_utf8_lossy(&sai.stderr)
    );
}
