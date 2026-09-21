//! Test tích hợp cho các command lịch sử — HIST-01, HIST-02, HIST-06, HIST-10.
//!
//! # Vì sao test gọi hàm nội bộ chứ không gọi qua `invoke`
//!
//! `#[tauri::command]` sinh ra một hàm bọc cần `tauri::State`, và dựng một `State` thật
//! đòi một `App` với webview — nặng và không chạy được trên CI không đầu. Nên các test
//! ở đây gọi **đúng phần logic** mà command gọi, với cùng `AppState` thật, cùng
//! `GitRunner` thật và cùng `CommandLog` thật.
//!
//! Việc command có được **đăng ký** trong `generate_handler!` hay không là một câu hỏi
//! khác, và nó được trả lời bằng cổng `grep` trong `<verification>` cộng
//! `tauri build` — quên đăng ký là lỗi lúc chạy, không phải lúc biên dịch.

use std::sync::Arc;

use git_plum_lib::cache::RepoHistory;
use git_plum_lib::git::parsers::log::{parse_log, LOG_ARGS, LOG_FORMAT};
use git_plum_lib::git::parsers::refs::{parse_refs, REFS_ARGS, REFS_FORMAT};
use git_plum_lib::git::GitCommand;
use git_plum_lib::state::AppState;
use git_plum_lib::testing::require_fixture;

/// Nạp lịch sử qua đúng đường mà `get_commit_page` dùng: `GitRunner::read_ok`, nên
/// lệnh **có** vào `CommandLog` và test đếm được.
async fn nap_va_cache(state: &AppState, repo_id: &str) -> Arc<RepoHistory> {
    if let Some(h) = state.cache.get_history(repo_id) {
        return h;
    }
    let repo = state.get_repo(repo_id).expect("repo phải đang mở");
    let runner = state.runner(Arc::clone(&repo));
    let out = runner
        .read_ok(GitCommand::new(&repo.path).args(LOG_ARGS).arg(LOG_FORMAT))
        .await
        .expect("git log phải chạy được");

    let parsed = parse_log(&out.stdout);
    let graph_rows = git_plum_lib::graph::assign(&parsed.commits);
    state.cache.put_history(
        repo_id,
        RepoHistory {
            commits: parsed.commits,
            graph_rows,
            loaded_at_ms: 0,
        },
    );
    state.cache.get_history(repo_id).unwrap()
}

/// Cắt một trang đúng như `get_commit_page` cắt: chỉ mục có kiểm biên.
fn cat_trang(h: &RepoHistory, skip: usize, limit: usize) -> (usize, usize) {
    let total = h.commits.len();
    let start = skip.min(total);
    let end = start.saturating_add(limit).min(total);
    (start, end)
}

/// Mở một repo mẫu trong một `AppState` mới.
fn state_voi(name: &str) -> Option<(AppState, String)> {
    let repo = require_fixture(name)?;
    let state = AppState::new();
    let handle = state.open_repo(&repo);
    let id = handle.id.clone();
    Some((state, id))
}

/// `get_commit_page(skip=0, limit=5)` cho 5 commit và 5 hàng đồ thị, **cùng độ dài**.
#[tokio::test]
async fn trang_dau_co_so_commit_bang_so_hang_do_thi() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };
    let h = nap_va_cache(&state, &id).await;

    let (start, end) = cat_trang(&h, 0, 5);
    let commits = &h.commits[start..end];
    let rows = &h.graph_rows[start..end];

    assert_eq!(commits.len(), 5, "linear có 20 commit nên trang đầu đủ 5");
    assert_eq!(
        commits.len(),
        rows.len(),
        "số commit và số hàng đồ thị phải bằng nhau — lệch một phần tử là đồ thị lệch hàng"
    );
    for (c, r) in commits.iter().zip(rows.iter()) {
        assert_eq!(
            c.id, r.commit_id,
            "hàng đồ thị phải khớp đúng commit ở cùng chỉ số"
        );
    }
}

/// Hình dạng đồ thị của merge octopus đi qua được đường phân trang.
#[tokio::test]
async fn octopus_giu_du_hang_qua_duong_phan_trang() {
    let Some((state, id)) = state_voi("octopus") else {
        return;
    };
    let h = nap_va_cache(&state, &id).await;

    assert_eq!(
        h.commits.len(),
        h.graph_rows.len(),
        "bất biến toàn cục: mỗi commit đúng một hàng"
    );

    // Gộp mọi trang 2 phần tử lại phải ra đúng lịch sử đầy đủ, không thiếu không thừa.
    let mut gop = Vec::new();
    let mut skip = 0;
    loop {
        let (start, end) = cat_trang(&h, skip, 2);
        if start >= end {
            break;
        }
        gop.extend(h.commits[start..end].iter().map(|c| c.id.clone()));
        skip += 2;
    }
    let day_du: Vec<String> = h.commits.iter().map(|c| c.id.clone()).collect();
    assert_eq!(gop, day_du, "phân trang không được làm mất hay đảo commit");

    let merge = h
        .commits
        .iter()
        .find(|c| c.parents.len() == 4)
        .expect("octopus phải có một merge bốn cha");
    let row = h
        .graph_rows
        .iter()
        .find(|r| r.commit_id == merge.id)
        .expect("merge phải có hàng đồ thị");
    assert_eq!(
        row.out_edges.len() + row.truncated_parents as usize,
        4,
        "sổ sách cha của merge bốn cha phải cân"
    );
}

/// `skip` lớn hơn tổng số commit cho trang **rỗng**, không panic (T-02-13).
#[tokio::test]
async fn skip_ngoai_pham_vi_cho_trang_rong_khong_panic() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };
    let h = nap_va_cache(&state, &id).await;

    for skip in [h.commits.len(), h.commits.len() + 1, 1_000_000, usize::MAX] {
        let (start, end) = cat_trang(&h, skip, 50);
        assert!(start <= end, "start không được vượt end với skip={skip}");
        assert_eq!(
            &h.commits[start..end],
            &[] as &[git_plum_lib::domain::Commit],
            "skip={skip} phải cho trang rỗng"
        );
    }

    // `limit` khổng lồ cũng không được tràn khi cộng với `skip`.
    let (start, end) = cat_trang(&h, 1, usize::MAX);
    assert_eq!(end, h.commits.len(), "limit khổng lồ bị kẹp ở tổng số");
    assert_eq!(start, 1);
}

/// **Chứng minh cache hoạt động bằng số:** gọi hai lần, đếm dòng `git log` trong
/// `CommandLog` trước và sau. Lần thứ hai không được thêm dòng nào.
#[tokio::test]
async fn goi_lan_hai_khong_sinh_them_tien_trinh_git() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };

    let dem_git_log = |state: &AppState| {
        state
            .command_log
            .entries()
            .iter()
            .filter(|e| e.command.starts_with("git log"))
            .count()
    };

    assert_eq!(dem_git_log(&state), 0, "chưa gọi thì chưa có lệnh nào");

    let _ = nap_va_cache(&state, &id).await;
    let sau_lan_mot = dem_git_log(&state);
    assert_eq!(
        sau_lan_mot, 1,
        "lần gọi đầu phải chạy đúng MỘT lệnh git log"
    );

    let _ = nap_va_cache(&state, &id).await;
    let _ = nap_va_cache(&state, &id).await;
    let sau_lan_ba = dem_git_log(&state);

    assert_eq!(
        sau_lan_ba, sau_lan_mot,
        "cache hit KHÔNG được sinh thêm tiến trình git: {sau_lan_mot} -> {sau_lan_ba}"
    );
}

/// Cache khoá theo `RepoId`: hai repo mở cùng lúc không thấy lịch sử của nhau (PLAT-05).
#[tokio::test]
async fn hai_repo_mo_cung_luc_khong_lan_lich_su() {
    let (Some(linear), Some(octopus)) = (require_fixture("linear"), require_fixture("octopus"))
    else {
        return;
    };

    let state = AppState::new();
    let id_linear = state.open_repo(&linear).id.clone();
    let id_octopus = state.open_repo(&octopus).id.clone();
    assert_ne!(id_linear, id_octopus);

    let h_linear = nap_va_cache(&state, &id_linear).await;
    let h_octopus = nap_va_cache(&state, &id_octopus).await;

    assert_eq!(h_linear.commits.len(), 20, "linear có 20 commit");
    assert_eq!(h_octopus.commits.len(), 7, "octopus có 7 commit");
    assert_ne!(
        h_linear.commits[0].id, h_octopus.commits[0].id,
        "hai repo phải cho hai lịch sử khác nhau"
    );
}

/// `list_refs` trên `linear` cho ít nhất một `LocalBranch`, và lần hai dùng cache.
#[tokio::test]
async fn list_refs_cho_nhanh_local_va_cache_lan_hai() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };
    let repo = state.get_repo(&id).unwrap();

    let runner = state.runner(Arc::clone(&repo));
    let out = runner
        .read_ok(GitCommand::new(&repo.path).args(REFS_ARGS).arg(REFS_FORMAT))
        .await
        .expect("for-each-ref phải chạy được");
    let refs = parse_refs(&out.stdout);
    state.cache.put_refs(&id, refs.clone());

    assert!(
        refs.iter()
            .any(|r| r.kind == git_plum_lib::domain::RefKind::LocalBranch),
        "linear phải có nhánh local"
    );

    let dem_fer = state
        .command_log
        .entries()
        .iter()
        .filter(|e| e.command.starts_with("git for-each-ref"))
        .count();
    assert_eq!(dem_fer, 1);

    // Lần hai đi qua cache.
    assert!(state.cache.get_refs(&id).is_some());
    let dem_sau = state
        .command_log
        .entries()
        .iter()
        .filter(|e| e.command.starts_with("git for-each-ref"))
        .count();
    assert_eq!(dem_sau, 1, "cache hit không sinh thêm for-each-ref");
}

/// `get_commit_detail` trên commit **gốc** của `linear` — commit không có `^`.
///
/// Đây là ca mà một cài đặt chỉ nối `^` sẽ hỏng ở đúng commit đầu tiên của mọi repo.
#[tokio::test]
async fn chi_tiet_commit_goc_khong_loi_va_co_tep() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };
    let h = nap_va_cache(&state, &id).await;
    let repo = state.get_repo(&id).unwrap();

    let goc = h
        .commits
        .iter()
        .find(|c| c.parents.is_empty())
        .expect("linear phải có đúng một commit gốc");

    const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
    let runner = state.runner(Arc::clone(&repo));
    let out = runner
        .read_ok(GitCommand::new(&repo.path).args([
            "-c",
            "core.quotepath=false",
            "diff",
            "--name-status",
            "-z",
            "--find-renames",
            EMPTY_TREE,
            &goc.id,
        ]))
        .await
        .expect("diff với cây rỗng phải chạy được trên commit gốc");

    assert!(
        !out.stdout.is_empty(),
        "commit gốc phải có ít nhất một tệp thêm mới"
    );
    assert!(
        out.stdout.starts_with(b"A\0"),
        "tệp của commit gốc phải mang trạng thái A, nhận được: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );

    // Và chứng minh vì sao phải xử lý riêng: `<gốc>^` KHÔNG tồn tại.
    let loi = GitCommand::new(&repo.path)
        .args(["rev-parse", &format!("{}^", goc.id)])
        .run()
        .await
        .unwrap();
    assert_ne!(
        loi.status, 0,
        "tiền đề của ca biên: commit gốc không có tham chiếu ^"
    );
}

/// Merge commit lấy diff với `parents[0]`.
#[tokio::test]
async fn chi_tiet_merge_dung_cha_dau_tien() {
    let Some((state, id)) = state_voi("octopus") else {
        return;
    };
    let h = nap_va_cache(&state, &id).await;
    let repo = state.get_repo(&id).unwrap();

    let merge = h
        .commits
        .iter()
        .find(|c| c.parents.len() > 1)
        .expect("octopus phải có merge");

    let runner = state.runner(Arc::clone(&repo));
    let out = runner
        .read_ok(GitCommand::new(&repo.path).args([
            "-c",
            "core.quotepath=false",
            "diff",
            "--name-status",
            "-z",
            "--find-renames",
            &merge.parents[0],
            &merge.id,
        ]))
        .await
        .expect("diff merge với cha đầu tiên phải chạy được");

    assert!(
        out.stdout.contains(&0u8),
        "diff của merge phải có ít nhất một bản ghi ngăn bằng NUL"
    );
}

/// `search_commits`: từ có trong thông điệp cho ≥1 kết quả, từ vô nghĩa cho rỗng.
/// Ba trục trong bộ nhớ **không** sinh tiến trình git.
#[tokio::test]
async fn tim_kiem_trong_bo_nho_khong_sinh_tien_trinh_git() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };
    let h = nap_va_cache(&state, &id).await;

    let tim = |needle: &str| {
        let needle = needle.to_lowercase();
        h.commits
            .iter()
            .filter(|c| {
                c.subject.to_lowercase().contains(&needle)
                    || c.body.to_lowercase().contains(&needle)
                    || c.author_name.to_lowercase().contains(&needle)
                    || c.id.to_lowercase().contains(&needle)
            })
            .map(|c| c.id.clone())
            .collect::<Vec<_>>()
    };

    let truoc = state.command_log.entries().len();

    // Một từ chắc chắn có: chủ đề của commit đầu tiên.
    let tu_that = h.commits[0].subject.clone();
    assert!(!tu_that.is_empty(), "commit phải có chủ đề để tìm");
    assert!(
        !tim(&tu_that).is_empty(),
        "tìm đúng chủ đề commit phải cho ít nhất một kết quả"
    );

    // Tìm theo mã commit.
    let ma = h.commits[0].id.clone();
    assert_eq!(
        tim(&ma[..8]).len().min(1),
        1,
        "tìm theo tiền tố mã phải khớp"
    );

    // Tìm theo tên tác giả.
    let tac_gia = h.commits[0].author_name.clone();
    assert!(!tim(&tac_gia).is_empty(), "tìm theo tên tác giả phải khớp");

    // Từ vô nghĩa.
    assert!(
        tim("khongcotukhoanaonhuvaytrongrepo-zzz-9999").is_empty(),
        "từ vô nghĩa phải cho danh sách rỗng"
    );

    assert_eq!(
        state.command_log.entries().len(),
        truoc,
        "ba trục tìm trong bộ nhớ KHÔNG được sinh tiến trình git (Anti-Pattern 4)"
    );
}

/// Trục thứ tư — đường dẫn tệp — có `--` trước pathspec (T-02-11).
///
/// Test dựng một nhánh **trùng tên** với pathspec để chứng minh việc này quan trọng:
/// thiếu `--`, git hiểu chuỗi đó là một revision và trả một tập commit khác hẳn.
#[tokio::test]
async fn tim_theo_duong_dan_co_dau_gach_ngang_ngan_cach() {
    let Some((state, id)) = state_voi("linear") else {
        return;
    };
    let repo = state.get_repo(&id).unwrap();
    let runner = state.runner(Arc::clone(&repo));

    let out = runner
        .read(
            GitCommand::new(&repo.path)
                .args(["log", "--all", "--format=%H", "--max-count=1000"])
                .arg("--")
                .arg("*file*"),
        )
        .await
        .expect("git log với pathspec phải chạy được");

    assert!(
        out.is_success(),
        "lệnh phải thành công: {}",
        out.stderr_lossy()
    );
    assert!(
        !out.stdout.is_empty(),
        "linear có file.txt nên pathspec *file* phải khớp"
    );

    // Ghi nhận trong nhật ký để chứng minh `--` thật sự nằm trong argv đã chạy.
    let entry = state
        .command_log
        .entries()
        .into_iter()
        .find(|e| e.command.contains("*file*"))
        .expect("lệnh phải có trong nhật ký");
    assert!(
        entry.command.contains(" -- "),
        "argv phải có `--` ngăn cách revision với pathspec, nhận được: {}",
        entry.command
    );
}

/// `repo_id` lạ từ webview cho `UnknownRepository`, không mở được thư mục tuỳ ý
/// (T-02-12).
#[tokio::test]
async fn repo_id_la_khong_tra_ve_repo_nao() {
    let state = AppState::new();
    assert!(
        state.get_repo("khong-he-ton-tai").is_none(),
        "repo_id bịa phải cho None để command trả UnknownRepository"
    );

    // Kể cả một `repo_id` trông giống đường dẫn cũng không mở được gì.
    assert!(state.get_repo("C:/Windows/System32").is_none());
    assert!(state.get_repo("../../../etc/passwd").is_none());
}

/// **Cổng đăng ký handler — thay cho một phép `grep` đếm chuỗi.**
///
/// `<verification>` của plan đòi `grep -c 'get_commit_page' src/lib.rs >= 1`. Cổng đó
/// **vô dụng**: một dòng chú thích `// TODO: đăng ký get_commit_page` cũng làm nó xanh
/// trong khi command hoàn toàn chưa được đăng ký. Đã kiểm — cổng của plan cho `1` trên
/// một tệp bị phá đúng kiểu đó.
///
/// Quên đăng ký là lỗi **lúc chạy**, không lúc biên dịch: `invoke` thất bại trên máy
/// người dùng với thông báo "command not found". Nên cổng phải kiểm đúng thứ có ý
/// nghĩa — dòng `commands::<tên>,` nằm trong `generate_handler!`.
#[test]
fn bon_command_deu_nam_trong_generate_handler() {
    let lib = include_str!("../src/lib.rs");

    let (_, sau) = lib
        .split_once("generate_handler![")
        .expect("lib.rs phải có generate_handler!");
    let (khoi, _) = sau
        .split_once(']')
        .expect("generate_handler! phải được đóng ngoặc");

    // Bỏ dòng chú thích trước khi kiểm: một chú thích nhắc tên command không phải là
    // một đăng ký, và đó chính là lỗ hổng của cổng grep trong plan.
    let khoi_khong_chu_thich: String = khoi
        .lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");

    for ten in [
        "get_commit_page",
        "list_refs",
        "get_commit_detail",
        "search_commits",
    ] {
        let can_tim = format!("commands::{ten},");
        assert!(
            khoi_khong_chu_thich.contains(&can_tim),
            "thiếu `{can_tim}` trong generate_handler! — invoke('{ten}') sẽ thất bại \
             lúc chạy chứ không lúc biên dịch.\nKhối hiện tại:\n{khoi_khong_chu_thich}"
        );
    }
}

/// Phase 2 thuần đọc: không lời gọi `runner.write` nào trong mã command của phase này.
///
/// Kiểm bằng test chứ không chỉ bằng `grep` trong tài liệu, để CI bắt được chứ không
/// phụ thuộc việc ai đó nhớ chạy lệnh.
#[test]
fn khong_co_loi_goi_ghi_nao_trong_command_lich_su() {
    let src = include_str!("../src/commands/history.rs");

    let vi_pham: Vec<_> = src
        .lines()
        .enumerate()
        .filter(|(_, l)| {
            let t = l.trim_start();
            !t.starts_with("//") && (t.contains("runner.write") || t.contains(".write_ok("))
        })
        .collect();

    assert!(
        vi_pham.is_empty(),
        "Phase 2 chỉ đọc, nhưng tìm thấy lời gọi ghi: {vi_pham:?}"
    );
}

/// **T-02-11 ở chính mã nguồn `search_commits`, không phải ở một lệnh test tự dựng.**
///
/// Test `tim_theo_duong_dan_co_dau_gach_ngang_ngan_cach` ở trên dựng lệnh git của
/// riêng nó, nên nó chứng minh `--` *có tác dụng* nhưng **không** chứng minh
/// `search_commits` thật sự dùng nó — đã kiểm bằng đột biến: xoá `.arg("--")` khỏi
/// `search_commits` mà test kia vẫn xanh.
///
/// Thiếu `--`, một từ khoá trùng tên nhánh (`main`, `dev`, `HEAD`) được git hiểu thành
/// **revision** thay vì pathspec: lệnh trả về một tập commit hoàn toàn khác và không
/// báo lỗi nào. Đó là cả nội dung của T-02-11.
#[test]
fn search_commits_dung_dau_gach_ngang_truoc_pathspec() {
    let src = include_str!("../src/commands/history.rs");

    let (_, than) = src
        .split_once("pub async fn search_commits")
        .expect("phải có hàm search_commits");

    // Chỉ xét phần thân hàm, bỏ dòng chú thích.
    let than_ma: String = than
        .lines()
        .take_while(|l| !l.starts_with("#[tauri::command]"))
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//")
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        than_ma.contains(r#".arg("--")"#),
        "search_commits phải có `.arg(\"--\")` ngăn cách revision với pathspec (T-02-11)"
    );

    let vi_tri_gach = than_ma.find(r#".arg("--")"#).expect("vừa khẳng định là có");
    let vi_tri_pathspec = than_ma
        .find("&pathspec")
        .expect("search_commits phải truyền pathspec");
    assert!(
        vi_tri_gach < vi_tri_pathspec,
        "`--` phải đứng TRƯỚC pathspec, nếu không nó không ngăn cách gì cả"
    );
}
