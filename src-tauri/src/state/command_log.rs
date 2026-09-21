//! Nhật ký lệnh git — PLAT-08.
//!
//! Hiển thị đúng những lệnh mà ứng dụng đã chạy, kèm mã thoát và thời gian chạy.
//!
//! Đây là câu trả lời trực tiếp cho chỉ trích phổ biến nhất nhắm vào các công cụ
//! Git đồ hoạ: che giấu Git khiến người dùng không học được gì và không tự cứu
//! được khi có sự cố. Chi phí gần như bằng không vì lớp bọc đã dựng sẵn tham số.

use std::collections::VecDeque;
use std::time::Duration;

use parking_lot::Mutex;
use serde::Serialize;

/// Số bản ghi giữ lại. Đủ để soi lại một phiên làm việc, không đủ để phình bộ nhớ.
const MAX_ENTRIES: usize = 500;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandLogEntry {
    /// Số thứ tự tăng dần, dùng làm khoá khi vẽ danh sách.
    pub seq: u64,
    /// Lệnh đầy đủ, ví dụ `git log --topo-order --max-count=100`.
    pub command: String,
    /// Thư mục chạy lệnh.
    pub cwd: String,
    /// Mã thoát. `None` khi lệnh không chạy nổi hoặc quá hạn giờ.
    pub exit_code: Option<i32>,
    /// Thời gian chạy, tính bằng mili giây.
    pub duration_ms: u64,
    /// Dòng đầu của stderr khi lệnh thất bại. Giữ ngắn để danh sách dễ đọc.
    pub error: Option<String>,
    /// Thời điểm chạy, tính bằng mili giây từ mốc Unix.
    pub started_at_ms: u64,
}

pub struct CommandLog {
    entries: Mutex<VecDeque<CommandLogEntry>>,
    next_seq: Mutex<u64>,
}

impl CommandLog {
    pub fn new() -> Self {
        Self {
            entries: Mutex::new(VecDeque::with_capacity(MAX_ENTRIES)),
            next_seq: Mutex::new(1),
        }
    }

    /// Ghi lại một lệnh đã chạy xong.
    pub fn record(
        &self,
        args: &[String],
        cwd: &str,
        exit_code: Option<i32>,
        duration: Duration,
        error: Option<String>,
    ) -> CommandLogEntry {
        let seq = {
            let mut n = self.next_seq.lock();
            let current = *n;
            *n += 1;
            current
        };

        let entry = CommandLogEntry {
            seq,
            command: format!("git {}", args.join(" ")),
            cwd: cwd.to_string(),
            exit_code,
            duration_ms: duration.as_millis() as u64,
            error: error.map(|e| first_line(&e)),
            started_at_ms: now_ms(),
        };

        let mut entries = self.entries.lock();
        if entries.len() >= MAX_ENTRIES {
            entries.pop_front();
        }
        entries.push_back(entry.clone());

        entry
    }

    /// Toàn bộ bản ghi, mới nhất trước.
    pub fn entries(&self) -> Vec<CommandLogEntry> {
        self.entries.lock().iter().rev().cloned().collect()
    }

    pub fn clear(&self) {
        self.entries.lock().clear();
    }
}

impl Default for CommandLog {
    fn default() -> Self {
        Self::new()
    }
}

fn first_line(s: &str) -> String {
    s.lines().next().unwrap_or("").trim().to_string()
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_a_successful_command() {
        let log = CommandLog::new();
        let entry = log.record(
            &["status".into(), "--porcelain=v2".into()],
            "C:/work/repo",
            Some(0),
            Duration::from_millis(42),
            None,
        );

        assert_eq!(entry.command, "git status --porcelain=v2");
        assert_eq!(entry.exit_code, Some(0));
        assert_eq!(entry.duration_ms, 42);
        assert!(entry.error.is_none());
    }

    #[test]
    fn keeps_only_the_first_line_of_an_error() {
        let log = CommandLog::new();
        let entry = log.record(
            &["push".into()],
            "C:/work/repo",
            Some(1),
            Duration::from_millis(5),
            Some("fatal: không có quyền\ndòng thứ hai bị bỏ".into()),
        );
        assert_eq!(entry.error.as_deref(), Some("fatal: không có quyền"));
    }

    #[test]
    fn returns_newest_entries_first() {
        let log = CommandLog::new();
        log.record(&["one".into()], ".", Some(0), Duration::ZERO, None);
        log.record(&["two".into()], ".", Some(0), Duration::ZERO, None);

        let entries = log.entries();
        assert_eq!(entries[0].command, "git two");
        assert_eq!(entries[1].command, "git one");
    }

    #[test]
    fn evicts_oldest_beyond_capacity() {
        let log = CommandLog::new();
        for i in 0..(MAX_ENTRIES + 10) {
            log.record(&[format!("cmd{i}")], ".", Some(0), Duration::ZERO, None);
        }
        let entries = log.entries();
        assert_eq!(entries.len(), MAX_ENTRIES);
        // Bản ghi mới nhất vẫn còn, bản ghi đầu tiên đã bị đẩy ra.
        assert_eq!(entries[0].command, format!("git cmd{}", MAX_ENTRIES + 9));
    }

    #[test]
    fn sequence_numbers_increase() {
        let log = CommandLog::new();
        let a = log.record(&["a".into()], ".", Some(0), Duration::ZERO, None);
        let b = log.record(&["b".into()], ".", Some(0), Duration::ZERO, None);
        assert!(b.seq > a.seq);
    }
}
