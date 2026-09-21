//! Test tích hợp: chạy `git for-each-ref` **thật** trên repo mẫu — HIST-06, HIST-07.
//!
//! Test đơn vị trong `parsers/refs.rs` dựng buffer bằng tay, nên chúng kiểm bộ phân
//! tích chứ **không** kiểm rằng định dạng `REFS_FORMAT` thật sự sinh ra thứ mà bộ phân
//! tích mong đợi. Chỉ có git thật trả lời được câu đó — và đó chính là chỗ kế hoạch
//! 02-04 sai về `%(objectname)` của tag có chú thích.

use git_plum_lib::domain::RefKind;
use git_plum_lib::git::parsers::refs::{parse_refs, REFS_ARGS, REFS_FORMAT};
use git_plum_lib::git::GitCommand;
use git_plum_lib::testing::require_fixture;

/// Chạy `for-each-ref` với `REFS_FORMAT` trong một repo và phân tích kết quả.
async fn refs_cua(repo: &std::path::Path) -> Vec<git_plum_lib::domain::Ref> {
    let out = GitCommand::new(repo)
        .args(REFS_ARGS)
        .arg(REFS_FORMAT)
        .run()
        .await
        .expect("for-each-ref phải chạy được");
    assert!(
        out.status == 0,
        "for-each-ref thất bại: {}",
        out.stderr_lossy()
    );
    parse_refs(&out.stdout)
}

/// `linear` có đúng một nhánh local đang checkout.
#[tokio::test]
async fn linear_co_nhanh_local() {
    let Some(repo) = require_fixture("linear") else {
        return;
    };
    let refs = refs_cua(&repo).await;

    assert!(!refs.is_empty(), "linear phải có ít nhất một ref");
    let local: Vec<_> = refs
        .iter()
        .filter(|r| r.kind == RefKind::LocalBranch)
        .collect();
    assert!(
        !local.is_empty(),
        "linear phải có ít nhất một LocalBranch, nhận được: {:?}",
        refs.iter().map(|r| &r.full_name).collect::<Vec<_>>()
    );

    // Mã commit phải dài đúng 40 ký tự hex — nếu bộ phân tích để lọt `\r` hay một dấu
    // phân tách thì con số này sai và mọi phép neo nhãn vào hàng commit thất bại.
    for r in &refs {
        assert_eq!(
            r.target.len(),
            40,
            "{} có target dài {} ký tự: {:?}",
            r.full_name,
            r.target.len(),
            r.target
        );
        assert!(
            r.target.chars().all(|c| c.is_ascii_hexdigit()),
            "{} có target không phải hex: {:?}",
            r.full_name,
            r.target
        );
    }
}

/// **Ca biên thứ năm của nghiên cứu mục 4:** HEAD tách rời không neo vào nhãn nhánh
/// nào, nên **không** ref nào có `is_head == true`. Một cài đặt so `%(HEAD) != ""` sẽ
/// đánh dấu mọi ref là HEAD và test này bắt được.
#[tokio::test]
async fn detached_khong_ref_nao_la_head() {
    let Some(repo) = require_fixture("detached") else {
        return;
    };
    let refs = refs_cua(&repo).await;

    assert!(
        !refs.is_empty(),
        "detached vẫn phải có ref (nhánh main và các nhánh remote)"
    );
    let head_refs: Vec<_> = refs.iter().filter(|r| r.is_head).collect();
    assert!(
        head_refs.is_empty(),
        "HEAD tách rời: không ref nào được có is_head = true, nhưng {:?} có",
        head_refs.iter().map(|r| &r.full_name).collect::<Vec<_>>()
    );
}

/// Repo clone phân biệt được nhánh local với nhánh remote. `detached` và `shallow` là
/// bản clone nên chúng có `refs/remotes/*`.
#[tokio::test]
async fn repo_clone_phan_biet_local_voi_remote() {
    let Some(repo) = require_fixture("detached") else {
        return;
    };
    let refs = refs_cua(&repo).await;

    let remote: Vec<_> = refs
        .iter()
        .filter(|r| r.kind == RefKind::RemoteBranch)
        .collect();
    assert!(
        !remote.is_empty(),
        "bản clone phải có nhánh remote, nhận được: {:?}",
        refs.iter()
            .map(|r| (&r.full_name, r.kind))
            .collect::<Vec<_>>()
    );
    for r in &remote {
        assert!(
            r.short_name.contains('/'),
            "tên ngắn của nhánh remote phải kèm tên remote, nhận được {:?}",
            r.short_name
        );
    }
}

/// **Chứng minh việc sửa `%(*objectname)` trên git thật.**
///
/// Kế hoạch đặc tả `%(objectname)` cho `Ref::target`. Với tag **có chú thích** đó là mã
/// của đối tượng tag, không phải mã commit — nên nhãn tag sẽ không neo được vào hàng
/// nào. Không repo mẫu nào có tag, nên test này tự dựng repo tạm.
///
/// Khẳng định cuối là điều quan trọng: `target` của tag có chú thích phải nằm trong
/// danh sách mã commit mà `git log` in ra.
#[tokio::test]
async fn tag_co_chu_thich_neo_vao_mot_hang_commit_that() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path();

    let git = |args: Vec<&str>| {
        let owned: Vec<String> = args.into_iter().map(String::from).collect();
        async move {
            let out = GitCommand::new(path)
                .args(&owned)
                .run()
                .await
                .unwrap_or_else(|e| panic!("git {owned:?} không chạy được: {e}"));
            assert!(
                out.status == 0,
                "git {:?} thất bại: {}",
                owned,
                out.stderr_lossy()
            );
            out
        }
    };

    git(vec!["init", "-q", "-b", "main", "."]).await;
    git(vec!["config", "user.email", "a@b.c"]).await;
    git(vec!["config", "user.name", "A"]).await;
    std::fs::write(path.join("f.txt"), b"x").unwrap();
    git(vec!["add", "."]).await;
    git(vec!["commit", "-q", "-m", "c1"]).await;
    // Tag nhẹ và tag có chú thích trỏ cùng một commit.
    git(vec!["tag", "nhe"]).await;
    git(vec!["tag", "-a", "cochuthich", "-m", "ghi chu"]).await;

    let refs = refs_cua(path).await;

    let head = git(vec!["rev-parse", "HEAD"]).await;
    let ma_commit = String::from_utf8_lossy(&head.stdout).trim().to_string();

    let tags: Vec<_> = refs.iter().filter(|r| r.kind == RefKind::Tag).collect();
    assert_eq!(tags.len(), 2, "phải thấy cả hai tag");

    for t in &tags {
        assert_eq!(
            t.target, ma_commit,
            "tag {} phải trỏ mã COMMIT {} để nhãn neo được vào hàng, nhưng trỏ {}",
            t.short_name, ma_commit, t.target
        );
    }

    // Và chứng minh cái sai mà việc này phòng: mã của **đối tượng tag** khác mã commit.
    let tag_obj = git(vec!["rev-parse", "cochuthich"]).await;
    let ma_doi_tuong_tag = String::from_utf8_lossy(&tag_obj.stdout).trim().to_string();
    assert_ne!(
        ma_doi_tuong_tag, ma_commit,
        "tiền đề của test: đối tượng tag có chú thích phải có mã riêng, khác mã commit"
    );
}
