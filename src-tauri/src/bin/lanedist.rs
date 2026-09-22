//! Đo **phân bố lane** trên repo hiệu năng thật — một lần, không phải benchmark.
//!
//! Vì sao cần: `cargo bench` in `lane lớn nhất 20` trên repo 100k trong khi
//! `MAX_VISIBLE_LANES = 13`. Đó là hành vi đúng theo thiết kế (mọi commit phải có
//! lane, cap chỉ áp cho **cha**), nhưng `laneX` ở frontend clamp lane về cột 12, nên
//! mọi lane >= 13 vẽ **đè lên cùng một cột**. Chưa ai đo việc đó xảy ra bao nhiêu trên
//! lịch sử thật, và đó đúng là thứ người dùng sẽ thấy khi dogfood.
//!
//! Chạy: `cargo run --release --bin lanedist [đường-dẫn-repo]`
use git_plum_lib::git::parsers::log::{parse_log, LOG_FORMAT};
use git_plum_lib::graph::{assign, MAX_VISIBLE_LANES};
use std::process::Command;

fn main() {
    let repo = std::env::args().nth(1).unwrap_or_else(|| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../target/fixtures-perf/perf-100k").to_string()
    });
    let out = Command::new("git")
        .current_dir(&repo)
        .args(["log", "--all", "--topo-order"])
        .arg(LOG_FORMAT)
        .env("LC_ALL", "C")
        .output()
        .expect("git log");
    assert!(out.status.success(), "git log that bai: {:?}", out.status);

    let ket_qua = parse_log(&out.stdout);
    let commits = &ket_qua.commits;
    let rows = assign(commits);
    println!("repo: {repo}");
    println!("commit: {}, hang: {}", commits.len(), rows.len());

    let cap = MAX_VISIBLE_LANES;
    let max_lane = rows.iter().map(|r| r.lane).max().unwrap_or(0);
    let qua_cap = rows.iter().filter(|r| r.lane >= cap).count();
    let co_trunc = rows.iter().filter(|r| r.truncated_parents > 0).count();
    let tong_trunc: u64 = rows.iter().map(|r| r.truncated_parents as u64).sum();

    println!("MAX_VISIBLE_LANES = {cap}, lane lon nhat that = {max_lane}");
    println!(
        "hang co lane >= cap (bi clamp vao cot {}): {} ({:.4}%)",
        cap - 1,
        qua_cap,
        qua_cap as f64 * 100.0 / rows.len() as f64
    );
    println!(
        "hang co cha bi luoc: {} ({:.4}%), tong cha bi luoc: {}",
        co_trunc,
        co_trunc as f64 * 100.0 / rows.len() as f64,
        tong_trunc
    );

    let mut hist = vec![0usize; (max_lane as usize) + 1];
    for r in &rows {
        hist[r.lane as usize] += 1;
    }
    println!("--- phan bo lane ---");
    for (lane, n) in hist.iter().enumerate() {
        if *n > 0 {
            let flag = if lane as u16 >= cap { "  <-- CLAMP" } else { "" };
            println!("lane {lane:>3}: {n:>7}{flag}");
        }
    }
}
