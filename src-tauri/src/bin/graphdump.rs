//! In đồ thị dạng ASCII từ **đúng** `assign()` — để so với `git log --graph` hoặc
//! GitKraken khi đồ thị vẽ sai trên một repo thật.
//!
//! Chạy: `cargo run --bin graphdump -- <repo> [bắt-đầu-từ-sha] [số-hàng] [--topo-order|--date-order]`
//!
//! Mỗi hàng: `*` là nút commit, `|` là lane đi xuyên qua, `\`/`/` đánh dấu lane
//! có cạnh rẽ ra (cha thêm) hoặc hợp vào tại hàng đó.
use git_plum_lib::git::parsers::log::{parse_log, LOG_ARGS, LOG_FORMAT};
use git_plum_lib::graph::assign;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let repo = args.get(1).expect("thiếu đường dẫn repo");
    let start = args.get(2).cloned().unwrap_or_default();
    let count: usize = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(40);
    let order = args.get(4).cloned();

    let mut log_args: Vec<String> = LOG_ARGS.iter().map(|s| s.to_string()).collect();
    if let Some(o) = order {
        log_args.retain(|a| a != "--topo-order" && a != "--date-order");
        log_args.push(o);
    }
    let out = Command::new("git")
        .current_dir(repo)
        .args(&log_args)
        .arg(LOG_FORMAT)
        .env("LC_ALL", "C")
        .output()
        .expect("git log");
    assert!(out.status.success(), "git log thất bại");

    let commits = parse_log(&out.stdout).commits;
    let rows = assign(&commits);
    let max_lane = rows.iter().map(|r| r.lane).max().unwrap_or(0);
    println!(
        "args: {log_args:?}  commit: {}  lane lớn nhất: {max_lane}",
        rows.len()
    );

    let from = if start.is_empty() {
        0
    } else {
        commits
            .iter()
            .position(|c| c.id.starts_with(&start))
            .expect("không thấy sha")
    };
    for i in from..(from + count).min(rows.len()) {
        let r = &rows[i];
        let width = r
            .passthrough
            .iter()
            .chain(&r.out_edges)
            .flat_map(|e| [e.from_lane, e.to_lane])
            .chain([r.lane])
            .max()
            .unwrap_or(0) as usize
            + 1;
        let mut cells = vec![' '; width];
        for e in &r.passthrough {
            if e.from_lane == e.to_lane {
                cells[e.from_lane as usize] = '|';
            } else {
                cells[e.from_lane as usize] = '/';
            }
        }
        for e in &r.out_edges {
            if e.to_lane != r.lane {
                cells[e.to_lane as usize] = '\\';
            }
        }
        cells[r.lane as usize] = '*';
        let graph: String = cells.iter().flat_map(|c| [*c, ' ']).collect();
        let c = &commits[i];
        println!(
            "{graph:<24} {} {}",
            &c.id[..8],
            c.subject.chars().take(60).collect::<String>()
        );
    }
}
