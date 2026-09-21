//! Kiểu lỗi dùng chung cho mọi Tauri command.
//!
//! Tauri đòi kiểu lỗi phải impl `Serialize`. `thiserror` chỉ sinh `Display`,
//! nên phần `Serialize` viết tay ở cuối tệp. Đây là lý do `anyhow` không dùng
//! được ở ranh giới IPC.

use serde::{Serialize, Serializer};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Không sinh được tiến trình git: {reason}")]
    SpawnFailed { args: Vec<String>, reason: String },

    #[error("Lệnh git quá hạn {seconds} giây")]
    Timeout { args: Vec<String>, seconds: u64 },

    #[error("Không ghi được dữ liệu vào stdin của git: {0}")]
    StdinWriteFailed(String),

    #[error("git thoát với mã {status}: {stderr}")]
    CommandFailed {
        args: Vec<String>,
        status: i32,
        stderr: String,
    },

    #[error("Không phải một repository git: {path}")]
    NotARepository { path: String },

    #[error("Chưa mở repository nào")]
    NoRepositoryOpen,

    #[error("Không tìm thấy repository với mã {0}")]
    UnknownRepository(String),

    #[error("Không đọc được đầu ra của git: {0}")]
    ParseFailed(String),

    #[error("Lỗi nhập xuất: {0}")]
    Io(String),
}

impl GitError {
    /// Mã lỗi ổn định để phía giao diện phân nhánh xử lý, thay vì so khớp chuỗi.
    pub fn code(&self) -> &'static str {
        match self {
            Self::SpawnFailed { .. } => "spawn_failed",
            Self::Timeout { .. } => "timeout",
            Self::StdinWriteFailed(_) => "stdin_write_failed",
            Self::CommandFailed { .. } => "command_failed",
            Self::NotARepository { .. } => "not_a_repository",
            Self::NoRepositoryOpen => "no_repository_open",
            Self::UnknownRepository(_) => "unknown_repository",
            Self::ParseFailed(_) => "parse_failed",
            Self::Io(_) => "io",
        }
    }

    /// Tham số của lệnh git đã chạy, nếu lỗi này sinh ra từ một lệnh cụ thể.
    /// Giao diện hiển thị kèm thông báo lỗi để người dùng biết chuyện gì xảy ra
    /// (PLAT-10).
    pub fn command_args(&self) -> Option<&[String]> {
        match self {
            Self::SpawnFailed { args, .. }
            | Self::Timeout { args, .. }
            | Self::CommandFailed { args, .. } => Some(args),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// Dạng lỗi gửi sang phía giao diện.
#[derive(Serialize)]
struct SerializedError<'a> {
    code: &'a str,
    message: String,
    /// Lệnh git đã chạy, dạng `git log --format=... --all`, để hiện trong
    /// thông báo lỗi và trong nhật ký lệnh.
    command: Option<String>,
}

impl Serialize for GitError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let command = self
            .command_args()
            .map(|args| format!("git {}", args.join(" ")));

        SerializedError {
            code: self.code(),
            message: self.to_string(),
            command,
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_code_and_command() {
        let err = GitError::CommandFailed {
            args: vec!["status".into(), "--porcelain=v2".into()],
            status: 128,
            stderr: "fatal: not a git repository".into(),
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "command_failed");
        assert_eq!(json["command"], "git status --porcelain=v2");
        assert!(json["message"].as_str().unwrap().contains("128"));
    }

    #[test]
    fn errors_without_a_command_have_null_command() {
        let err = GitError::NoRepositoryOpen;
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "no_repository_open");
        assert!(json["command"].is_null());
    }
}
