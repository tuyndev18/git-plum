//! Đo chi phí mà plan 02-04 **thêm** vào đường nóng — checkpoint #1.
//!
//! Wave 3 đo: `git log` 693ms + `parse_log` 82ms + `assign` 57ms = **832ms** trên
//! 100 007 commit, còn dư **~168ms** so với mốc một giây của Core Value. Plan này tiêu
//! phần dư đó bằng hai thứ: tra cache và tuần tự hoá JSON của một trang qua IPC.
//!
//! Test này **không** đặt ngưỡng thất bại tự động — cùng lý do plan 02-03 ghi trong
//! `ci.yml`: runner CI nhiễu hàng chục phần trăm và một ngưỡng cứng sẽ đỏ vì nhiễu chứ
//! không vì hồi quy. Nó in số ra để người đọc so với bảng ngân sách trong summary.
//!
//! # 🔴 Chạy ở profile `--release`, nếu không con số vô nghĩa
//!
//! Đo thật trên cùng một máy, cùng một repo 100 007 commit:
//!
//! | | debug | release |
//! |---|---|---|
//! | `parse_log` | 455 ms | **67 ms** |
//! | `assign` | 220 ms | **59 ms** |
//! | tổng đường nóng | 1366 ms | **794 ms** |
//!
//! Debug chậm gấp **gần 7 lần** ở `parse_log` và làm đường nóng *vượt* mốc một giây.
//! Bảng benchmark của wave 3 (82ms / 57ms) đo bằng `criterion`, vốn luôn dựng
//! release — nên chỉ số release so sánh được với nó. Đọc số debug rồi kết luận
//! "trượt Core Value" là sai; đọc số debug rồi kết luận "cache quá đắt" cũng sai.
//!
//! Chạy:
//! ```text
//! cargo test --release --test history_budget -- --nocapture --ignored
//! ```

use std::sync::Arc;
use std::time::Instant;

use git_plum_lib::cache::{RepoCache, RepoHistory};
use git_plum_lib::git::parsers::log::{parse_log, LOG_ARGS, LOG_FORMAT};
use git_plum_lib::git::GitCommand;

/// Thư mục repo hiệu năng 100k commit, `None` nếu chưa sinh.
fn repo_hieu_nang() -> Option<std::path::PathBuf> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("fixtures-perf")
        .join("perf-100k");
    if root.join(".git").exists() {
        Some(root)
    } else {
        eprintln!("BỎ QUA: chưa có repo hiệu năng ở {}", root.display());
        None
    }
}

/// `#[ignore]` vì nó chạy `git log --all` trên 100k commit — vài giây, không hợp cho
/// mỗi lần `cargo test`. Checkpoint #1 chạy nó tường minh.
#[test]
#[ignore = "chậm: nạp 100k commit; chạy tường minh cho checkpoint #1"]
fn chi_phi_cache_va_cat_trang_nam_trong_ngan_sach() {
    let Some(repo) = repo_hieu_nang() else {
        return;
    };

    let rt = tokio::runtime::Runtime::new().unwrap();

    let t0 = Instant::now();
    let out = rt
        .block_on(
            GitCommand::new(&repo)
                .args(LOG_ARGS)
                .arg(LOG_FORMAT)
                .timeout(std::time::Duration::from_secs(120))
                .run(),
        )
        .expect("git log phải chạy được");
    let t_git = t0.elapsed();

    let t0 = Instant::now();
    let parsed = parse_log(&out.stdout);
    let t_parse = t0.elapsed();

    let t0 = Instant::now();
    let rows = git_plum_lib::graph::assign(&parsed.commits);
    let t_assign = t0.elapsed();

    let so_commit = parsed.commits.len();
    assert_eq!(rows.len(), so_commit, "bất biến: mỗi commit một hàng");

    // --- Phần mà plan 02-04 thêm vào ---------------------------------------
    let cache = RepoCache::new();

    let t0 = Instant::now();
    cache.put_history(
        "perf",
        RepoHistory {
            commits: parsed.commits,
            graph_rows: rows,
            loaded_at_ms: 0,
        },
    );
    let t_put = t0.elapsed();

    let t0 = Instant::now();
    let h: Arc<RepoHistory> = cache.get_history("perf").expect("vừa put xong");
    let t_get = t0.elapsed();

    // Cắt một trang như `get_commit_page` cắt, ở giữa lịch sử (ca xấu nhất cho cache
    // CPU), rồi tuần tự hoá đúng payload mà IPC gửi đi.
    let skip = so_commit / 2;
    let limit = 100usize;
    let start = skip.min(so_commit);
    let end = start.saturating_add(limit).min(so_commit);

    let t0 = Instant::now();
    let commits = h.commits[start..end].to_vec();
    let graph_rows = h.graph_rows[start..end].to_vec();
    let t_slice = t0.elapsed();

    let t0 = Instant::now();
    let json = serde_json::to_string(&serde_json::json!({
        "commits": commits,
        "graphRows": graph_rows,
        "total": so_commit,
        "skippedRecords": 0,
    }))
    .expect("payload phải serialize được");
    let t_json = t0.elapsed();

    let them_vao = t_put + t_get + t_slice + t_json;

    eprintln!("\n=== Ngân sách đường nóng, {so_commit} commit ===");
    eprintln!(
        "  git log            {:>8.1} ms   (wave 2/3)",
        t_git.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  parse_log          {:>8.1} ms   (wave 2)",
        t_parse.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  assign             {:>8.1} ms   (wave 3)",
        t_assign.as_secs_f64() * 1000.0
    );
    eprintln!("  ---- plan 02-04 thêm vào ----");
    eprintln!(
        "  cache put          {:>8.3} ms",
        t_put.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  cache get          {:>8.3} ms",
        t_get.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  cắt trang {limit:>4}     {:>8.3} ms",
        t_slice.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  JSON {:>7} byte  {:>8.3} ms",
        json.len(),
        t_json.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  TỔNG THÊM VÀO      {:>8.3} ms   (ngân sách ~168 ms)",
        them_vao.as_secs_f64() * 1000.0
    );
    eprintln!(
        "  TỔNG ĐƯỜNG NÓNG    {:>8.1} ms   (mốc 1000 ms)\n",
        (t_git + t_parse + t_assign + them_vao).as_secs_f64() * 1000.0
    );

    // Lần gọi thứ hai: chỉ cache get + cắt + JSON, không git.
    let t0 = Instant::now();
    let h2 = cache.get_history("perf").unwrap();
    let c2 = h2.commits[start..end].to_vec();
    let g2 = h2.graph_rows[start..end].to_vec();
    let _ = serde_json::to_string(&serde_json::json!({
        "commits": c2, "graphRows": g2, "total": so_commit, "skippedRecords": 0,
    }))
    .unwrap();
    eprintln!(
        "  Trang thứ hai (cache hit, không git): {:.3} ms\n",
        t0.elapsed().as_secs_f64() * 1000.0
    );
}
