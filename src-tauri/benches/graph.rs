//! Benchmark `criterion` cho hai nửa của đường nóng lịch sử: phân tích `git log` và
//! gán lane. Checkpoint #1 của phase đòi việc này **nằm trong CI ngay trong phase**.
//!
//! # Hai nhóm tách riêng, có chủ ý
//!
//! Checkpoint #1 phải trả lời được *thời gian nằm ở đâu*, không chỉ *tổng bao nhiêu*.
//! Nếu phân tích chiếm 90% thì tối ưu thuật toán lane là vô ích, và ngược lại. Wave 2
//! đã đo trên repo thật 100k commit:
//!
//! ```text
//!   git log    693 ms   (18,9 MB, mười trường)
//!   parse_log   65 ms
//!   ─────────────────
//!              758 ms   → còn khoảng 240 ms cho gán lane trước mốc một giây
//! ```
//!
//! # Số trên dữ liệu tổng hợp KHÔNG phải số báo cáo cho checkpoint #1
//!
//! CI runner không có repo 100k commit, nhưng benchmark **vẫn phải chạy được** để bắt
//! hồi quy — một benchmark bị bỏ qua khi thiếu dữ liệu là một benchmark không tồn tại.
//! Vì vậy khi không tìm thấy repo hiệu năng, tệp này **sinh dữ liệu tổng hợp tất định**
//! trong bộ nhớ.
//!
//! Dữ liệu tổng hợp so sánh được **qua thời gian** (cùng một máy, cùng một hình dạng,
//! nên hồi quy vẫn lộ ra) nhưng **không** là con số báo cáo cho checkpoint #1: hình
//! dạng nhánh của nó do một vòng lặp sinh ra, không phải hình dạng của lịch sử thật.
//! Con số cho checkpoint phải đo trên repo thật, tức là chạy benchmark này với
//! `GIT_PLUM_PERF_REPO` trỏ tới repo của `scripts/fixtures/make-perf-repo.sh`.
//!
//! # Chạy
//!
//! ```bash
//! cd src-tauri
//! # trên dữ liệu tổng hợp (CI)
//! cargo bench --bench graph
//! # trên repo thật (số cho checkpoint #1)
//! GIT_PLUM_PERF_REPO=../target/fixtures-perf cargo bench --bench graph
//! ```

use std::hint::black_box;
use std::path::PathBuf;
use std::process::Command;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use git_plum_lib::domain::Commit;
use git_plum_lib::git::parsers::log::{parse_log, LOG_ARGS, LOG_FORMAT};
use git_plum_lib::graph::assign;

/// Biến môi trường trỏ tới repo hiệu năng 100k commit.
const PERF_REPO_ENV: &str = "GIT_PLUM_PERF_REPO";

/// Ba mức đo. 100k là mức của tiêu chí thành công số 1.
const MUC_DO: [usize; 3] = [1_000, 10_000, 100_000];

/// Đường dẫn repo hiệu năng, `None` khi chưa sinh.
///
/// Đọc [`PERF_REPO_ENV`] trước; không đặt thì thử hai chỗ mặc định.
///
/// # Vì sao phải thử `perf-100k/` chứ không chỉ `fixtures-perf/`
///
/// `scripts/fixtures/make-perf-repo.sh` ghi ra **một thư mục con** trong
/// `target/fixtures-perf/` (`perf-100k`, cạnh `perf-smoke` và `tiny`), chứ không biến
/// `fixtures-perf` thành repo. Bản đầu của tệp này chỉ kiểm `fixtures-perf/.git`, nên
/// nó im lặng rơi về dữ liệu tổng hợp **dù repo thật đang có sẵn trên đĩa** — và số
/// báo cho checkpoint #1 sẽ là số tổng hợp mà không ai biết. Chỉ lộ ra vì benchmark in
/// nguồn dữ liệu vào tên từng ca đo.
fn perf_repo() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(PERF_REPO_ENV) {
        if !dir.is_empty() {
            let p = PathBuf::from(dir);
            if p.join(".git").exists() {
                return Some(p);
            }
        }
    }
    let goc = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("target")
        .join("fixtures-perf");
    [goc.join("perf-100k"), goc]
        .into_iter()
        .find(|p| p.join(".git").exists())
}

/// Đọc **một lần** toàn bộ `git log` của repo hiệu năng vào bộ nhớ.
///
/// Benchmark đo `parse_log` và `assign`, **không** đo `git log`. Nếu để lệnh git nằm
/// trong vòng lặp đo thì 693ms của git sẽ nhấn chìm 65ms của phân tích và con số trở
/// nên vô nghĩa. Vì vậy git chạy một lần ngoài vòng đo, đúng như wave 2 đã làm.
///
/// Dùng [`std::process::Command`] chứ không phải `GitCommand` của dự án: `GitCommand`
/// là `async` và lôi cả tokio runtime vào benchmark mà không đổi gì về thứ đang đo.
fn doc_log_repo_that(repo: &PathBuf) -> Option<Vec<u8>> {
    let out = Command::new("git")
        .current_dir(repo)
        .args(LOG_ARGS)
        .arg(LOG_FORMAT)
        .env("LC_ALL", "C")
        .output()
        .ok()?;
    if !out.status.success() || out.stdout.is_empty() {
        return None;
    }
    Some(out.stdout)
}

/// Sinh một buffer `git log` **tổng hợp tất định** có hình dạng nhánh thật.
///
/// # Vì sao không phải một đường thẳng `n` commit
///
/// Đường thẳng không đo được gì về gán lane: `lanes` luôn dài đúng 1, bước 2 và bước 4
/// không bao giờ chạy, và độ phức tạp `O(n × số_lane)` suy biến thành `O(n)`. Muốn con
/// số nói lên điều gì thì dữ liệu phải có nhánh sống song song và merge.
///
/// Hình dạng: `SO_NHANH` nhánh chạy song song, cứ `CHU_KY` commit lại có một merge hai
/// cha, và rải merge ba cha (octopus) — gần hình dạng của
/// `scripts/fixtures/make-perf-repo.sh`, vốn đo được 8–20 nhánh sống đồng thời, 3 182
/// merge trong đó 320 octopus, tối đa 21 lane đồng thời.
///
/// Tất định: không `rand`, nên hai lần chạy cho cùng một buffer và so sánh được qua
/// thời gian.
fn sinh_log_tong_hop(n: usize) -> Vec<u8> {
    /// Số nhánh chạy song song. Chọn 16 để nằm trong khoảng 8–20 của repo thật.
    const SO_NHANH: usize = 16;
    /// Cứ bấy nhiêu commit thì có một merge.
    const CHU_KY: usize = 12;

    // Mã commit giả nhưng đúng dạng: 40 ký tự hex, suy ra từ chỉ số nên tra được.
    fn ma(i: usize) -> String {
        format!("{i:040x}")
    }

    let mut buf: Vec<u8> = Vec::with_capacity(n * 190);

    for i in 0..n {
        // Cha đầu: commit trước trên cùng nhánh, cách SO_NHANH bước.
        let mut cha: Vec<String> = Vec::new();
        if i + SO_NHANH < n {
            cha.push(ma(i + SO_NHANH));
        }
        // Merge định kỳ: thêm một cha từ nhánh khác.
        if i % CHU_KY == 0 && i + SO_NHANH + 3 < n {
            cha.push(ma(i + SO_NHANH + 3));
            // Octopus rải rác: cứ 10 merge thì có một merge ba cha.
            if i % (CHU_KY * 10) == 0 && i + SO_NHANH + 5 < n {
                cha.push(ma(i + SO_NHANH + 5));
            }
        }

        // Mười trường theo đúng LOG_FORMAT, phân tách \x1f, kết thúc \x1e.
        buf.extend_from_slice(ma(i).as_bytes());
        buf.push(0x1f);
        buf.extend_from_slice(cha.join(" ").as_bytes());
        buf.push(0x1f);
        buf.extend_from_slice(b"Nguoi Dung");
        buf.push(0x1f);
        buf.extend_from_slice(b"nguoi@example.com");
        buf.push(0x1f);
        buf.extend_from_slice(format!("{}", 1_600_000_000i64 + i as i64).as_bytes());
        buf.push(0x1f);
        buf.extend_from_slice(b"Nguoi Commit");
        buf.push(0x1f);
        buf.extend_from_slice(b"commit@example.com");
        buf.push(0x1f);
        buf.extend_from_slice(format!("{}", 1_600_000_000i64 + i as i64).as_bytes());
        buf.push(0x1f);
        buf.extend_from_slice(format!("commit thu {i} tren nhanh {}", i % SO_NHANH).as_bytes());
        buf.push(0x1f);
        buf.extend_from_slice(b"than thong diep, hai dong\nde ban ghi khong qua ngan");
        buf.push(0x1e);
    }

    buf
}

/// Buffer `git log` cho một mức đo, cùng nhãn nói rõ nguồn dữ liệu.
///
/// Ưu tiên repo thật; thiếu thì sinh tổng hợp. Nhãn đi vào tên benchmark để không ai
/// đọc lẫn số tổng hợp thành số của checkpoint #1.
fn buffer_cho_muc(n: usize, log_that: Option<&Vec<u8>>) -> (Vec<u8>, &'static str) {
    if let Some(that) = log_that {
        // Cắt đúng `n` bản ghi đầu từ buffer thật, giữ nguyên byte kết thúc \x1e.
        let mut het = 0usize;
        let mut dem = 0usize;
        for (i, b) in that.iter().enumerate() {
            if *b == 0x1e {
                dem += 1;
                if dem == n {
                    het = i + 1;
                    break;
                }
            }
        }
        if het > 0 {
            return (that[..het].to_vec(), "repo-that");
        }
    }
    (sinh_log_tong_hop(n), "tong-hop")
}

/// Đo `parse_log` trên buffer đã nằm sẵn trong bộ nhớ.
fn bench_parse_log(c: &mut Criterion) {
    let log_that = perf_repo().and_then(|r| doc_log_repo_that(&r));
    if log_that.is_none() {
        eprintln!(
            "LƯU Ý: không tìm thấy repo hiệu năng (đặt {PERF_REPO_ENV} hoặc chạy \
             `bash scripts/fixtures/make-perf-repo.sh`). Benchmark chạy trên dữ liệu \
             TỔNG HỢP — so sánh được qua thời gian, nhưng KHÔNG phải số báo cáo cho \
             checkpoint #1."
        );
    }

    let mut g = c.benchmark_group("parse_log");
    for n in MUC_DO {
        let (buf, nguon) = buffer_cho_muc(n, log_that.as_ref());
        g.throughput(Throughput::Bytes(buf.len() as u64));
        g.bench_with_input(BenchmarkId::new(nguon, n), &buf, |b, buf| {
            b.iter(|| {
                let r = parse_log(black_box(buf));
                black_box(r.commits.len())
            });
        });
    }
    g.finish();
}

/// Đo `assign` trên `Vec<Commit>` đã phân tích sẵn.
///
/// Phân tích nằm **ngoài** vòng đo: thứ đang đo là thuật toán lane, và 65ms của
/// `parse_log` sẽ làm mờ con số nếu để lẫn vào.
fn bench_assign_lanes(c: &mut Criterion) {
    let log_that = perf_repo().and_then(|r| doc_log_repo_that(&r));

    let mut g = c.benchmark_group("assign_lanes");
    for n in MUC_DO {
        let (buf, nguon) = buffer_cho_muc(n, log_that.as_ref());
        let commits: Vec<Commit> = parse_log(&buf).commits;

        // In hình dạng thật của dữ liệu đang đo: một benchmark trên dữ liệu suy biến
        // (một lane duy nhất) cho số đẹp mà vô nghĩa, và chỉ có cách này mới thấy.
        let rows = assign(&commits);
        let max_lane = rows.iter().map(|r| r.lane).max().unwrap_or(0);
        let so_merge = commits.iter().filter(|c| c.parents.len() > 1).count();
        eprintln!(
            "assign_lanes/{nguon}/{n}: {} commit, {} hàng, lane lớn nhất {}, {} merge",
            commits.len(),
            rows.len(),
            max_lane,
            so_merge
        );

        g.throughput(Throughput::Elements(commits.len() as u64));
        g.bench_with_input(BenchmarkId::new(nguon, n), &commits, |b, commits| {
            b.iter(|| {
                let rows = assign(black_box(commits));
                black_box(rows.len())
            });
        });
    }
    g.finish();
}

criterion_group!(benches, bench_parse_log, bench_assign_lanes);
criterion_main!(benches);
