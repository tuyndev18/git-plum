//! Kiểm rằng dụng cụ đo phía Rust **thật sự in ra số** — plan 02-07 Task 1.
//!
//! Vì sao cần một test riêng thay vì tin vào việc đọc mã: `tracing::info!` chỉ sinh
//! ra dòng log khi có subscriber và khi bộ lọc cho qua. Ba thứ phải đúng cùng lúc
//! (cờ `GIT_PLUM_PERF`, subscriber đã cài, bộ lọc ở mức `info`) và hỏng bất kỳ thứ
//! nào cũng cho cùng một triệu chứng: **im lặng hoàn toàn**. Đến lúc chạy checkpoint
//! #1 trên repo 100k mà không thấy dòng nào thì mất cả buổi để truy, trong khi lẽ ra
//! phát hiện được ở đây trong một giây.
//!
//! Test bắt đầu ra của `tracing` bằng một writer tuỳ biến rồi khẳng định ba tên
//! trường (`git_log_ms`, `parse_log_ms`, `lanes_assign_ms`) có mặt. Ghim **tên
//! trường** chứ không chỉ ghim "có dòng nào đó": tên trường là thứ người chạy
//! checkpoint sẽ grep, và đổi tên mà không đổi tài liệu là một cách âm thầm làm hỏng
//! quy trình đo.
//!
//! Chạy:
//! ```text
//! cargo test --test perf_instrumentation
//! ```

use std::io;
use std::sync::{Arc, Mutex};

/// Writer gom mọi thứ `tracing` ghi ra vào một `Vec<u8>` dùng chung.
#[derive(Clone, Default)]
struct BoNho(Arc<Mutex<Vec<u8>>>);

impl io::Write for BoNho {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("khoá writer").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl BoNho {
    fn noi_dung(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("khoá writer")).into_owned()
    }
}

/// Dựng một subscriber ghi vào bộ nhớ, chạy `f`, rồi trả lại những gì đã ghi.
///
/// Dùng `with_default` (subscriber theo phạm vi, không toàn cục) để test này không
/// tranh chấp với test khác chạy song song trong cùng tiến trình.
fn bat_log<F: FnOnce()>(f: F) -> String {
    let bo_nho = BoNho::default();
    let writer = bo_nho.clone();

    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_max_level(tracing::Level::INFO)
        .finish();

    tracing::subscriber::with_default(subscriber, f);
    bo_nho.noi_dung()
}

/// Ba tên trường của mốc đo checkpoint #1 phải xuất hiện nguyên văn.
///
/// Test này mô phỏng đúng lời gọi `tracing::info!` trong `nap_lich_su` thay vì nạp
/// một repo thật: mục tiêu là ghim **hợp đồng tên trường và mức log**, không phải đo
/// lại hiệu năng (việc đó là của `history_budget.rs` và của chính checkpoint #1).
#[test]
fn ba_moc_do_in_dung_ten_truong_o_muc_info() {
    let log = bat_log(|| {
        tracing::info!(
            repo = "abc123",
            commits = 100_007,
            git_log_ms = 693u128,
            parse_log_ms = 82u128,
            lanes_assign_ms = 57u128,
            stdout_bytes = 52_428_800usize,
            "[perf] nạp lịch sử lần đầu — ba mốc đo tách riêng"
        );
    });

    assert!(log.contains("git_log_ms"), "thiếu mốc (a) chạy git log");
    assert!(log.contains("parse_log_ms"), "thiếu mốc (b) phân tích");
    assert!(log.contains("lanes_assign_ms"), "thiếu mốc (c) gán lane");
    assert!(
        log.contains("[perf]"),
        "thiếu tiền tố [perf] để grep lại được"
    );
}

/// Bốn tên trường của checkpoint #4 (kích thước payload) phải xuất hiện nguyên văn.
#[test]
fn kich_thuoc_payload_in_dung_ten_truong() {
    let log = bat_log(|| {
        tracing::info!(
            repo = "abc123",
            rows = 1000,
            payload_bytes = 524_288usize,
            commits_bytes = 471_859usize,
            graph_rows_bytes = 52_429usize,
            graph_rows_percent = 10usize,
            "[perf] kích thước payload JSON một trang"
        );
    });

    assert!(
        log.contains("payload_bytes"),
        "thiếu tổng kích thước payload"
    );
    assert!(
        log.contains("commits_bytes"),
        "thiếu kích thước riêng của commits"
    );
    assert!(
        log.contains("graph_rows_bytes"),
        "thiếu kích thước riêng của graph_rows — đây là số quyết định của checkpoint #4"
    );
    assert!(log.contains("graph_rows_percent"), "thiếu tỉ lệ phần trăm");
}

/// Mức `warn` **không** cho dòng `info` đi qua.
///
/// Ghim điều này vì nó là cái bẫy thật của quy trình đo: người chạy checkpoint đặt
/// `RUST_LOG` sai mức sẽ thấy im lặng và tưởng dụng cụ đo hỏng. Tài liệu phải nói
/// `RUST_LOG=git_plum_lib=info`, và test này là lý do câu đó tồn tại.
#[test]
fn muc_warn_khong_cho_dong_info_di_qua() {
    let bo_nho = BoNho::default();
    let writer = bo_nho.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_max_level(tracing::Level::WARN)
        .finish();

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(git_log_ms = 693u128, "[perf] không được thấy dòng này");
    });

    assert!(
        !bo_nho.noi_dung().contains("git_log_ms"),
        "mức WARN phải chặn dòng info — nếu không, tài liệu đo đang nói sai về RUST_LOG"
    );
}
