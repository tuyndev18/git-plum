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

    /// `.git/index.lock` đang bị một tiến trình git khác giữ — WORK-02, CONTEXT.md 2.3.
    ///
    /// # Vì sao ca này có mã lỗi RIÊNG, không phải `CommandFailed`
    ///
    /// Đây **không** phải một lỗi; nó là một trạng thái tạm thời có thể phục hồi, và
    /// người dùng cần một câu khác hẳn: *"đang chờ một tiến trình git khác"* chứ không
    /// phải *"git thoát với mã 128"*. Chủ dự án **dùng terminal song song** — đó là cả
    /// điểm của WORK-10 — nên ca này sẽ xảy ra thật, không phải phòng xa.
    ///
    /// Gộp nó vào [`GitError::CommandFailed`] là lặp lại đúng khuôn lỗi của
    /// `open_repository`, nơi "exit khác 0" bị coi là **một** nguyên nhân và người dùng
    /// nhận "Không phải một repository git" cho một repo hoàn toàn hợp lệ mà git từ
    /// chối vì `dubious ownership`.
    ///
    /// `stderr` giữ **nguyên văn** câu trả lời của git: nó là văn bản chẩn đoán mờ, để
    /// người đọc, không phải một hợp đồng phân tích được.
    #[error(
        "Không ghi được vào repository: một tiến trình git khác đang giữ \
         .git/index.lock. Bạn có thể đang chạy git ở nơi khác (terminal, IDE, \
         hay một lệnh rebase chưa xong). Thử lại sau khi lệnh đó kết thúc.\n\n{stderr}"
    )]
    IndexLocked { args: Vec<String>, stderr: String },

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
            Self::IndexLocked { .. } => "index_locked",
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
            | Self::CommandFailed { args, .. }
            | Self::IndexLocked { args, .. } => Some(args),
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

    /// Ca `index.lock` có mã **riêng** và nói rõ người dùng đang chờ cái gì.
    ///
    /// 🔴 Khẳng định mã **không** phải `command_failed`: gộp hai ca lại là lặp lại
    /// khuôn lỗi của `open_repository` (CONTEXT.md mục 0), nơi mọi exit khác 0 bị coi
    /// là một nguyên nhân duy nhất.
    #[test]
    fn index_locked_co_ma_rieng_va_thong_bao_noi_ro_dang_cho_gi() {
        let err = GitError::IndexLocked {
            args: vec!["add".into(), "--".into(), "a.txt".into()],
            stderr: "fatal: Unable to create '.../index.lock': File exists.".into(),
        };
        let json = serde_json::to_value(&err).unwrap();

        assert_eq!(json["code"], "index_locked");
        assert_ne!(
            json["code"], "command_failed",
            "ca lock KHÔNG được gộp vào command_failed — giao diện phải nói khác"
        );

        let msg = json["message"].as_str().unwrap();
        assert!(
            msg.contains("index.lock"),
            "thông báo phải nói tên tệp đang bị giữ: {msg:?}"
        );
        assert!(
            msg.contains("terminal") || msg.contains("nơi khác"),
            "thông báo phải nói người dùng có thể đang chạy git ở nơi khác: {msg:?}"
        );
        assert!(
            msg.contains("File exists"),
            "stderr NGUYÊN VĂN của git phải đi kèm — nó là văn bản chẩn đoán: {msg:?}"
        );
        assert_eq!(
            json["command"], "git add -- a.txt",
            "lệnh đã chạy phải hiện ra (PLAT-10)"
        );
    }

    /// Mọi mã lỗi phải **khác nhau đôi một**: hai variant cùng mã làm giao diện không
    /// phân nhánh được, và lỗi đó không gây lỗi biên dịch ở bên nào.
    #[test]
    fn moi_ma_loi_khac_nhau_doi_mot() {
        let mau = [
            GitError::SpawnFailed {
                args: vec![],
                reason: String::new(),
            },
            GitError::Timeout {
                args: vec![],
                seconds: 1,
            },
            GitError::StdinWriteFailed(String::new()),
            GitError::CommandFailed {
                args: vec![],
                status: 1,
                stderr: String::new(),
            },
            GitError::NotARepository {
                path: String::new(),
            },
            GitError::IndexLocked {
                args: vec![],
                stderr: String::new(),
            },
            GitError::NoRepositoryOpen,
            GitError::UnknownRepository(String::new()),
            GitError::ParseFailed(String::new()),
            GitError::Io(String::new()),
        ];

        let mut ma: Vec<&str> = mau.iter().map(|e| e.code()).collect();
        let so_luong = ma.len();
        ma.sort_unstable();
        ma.dedup();
        assert_eq!(
            ma.len(),
            so_luong,
            "có hai variant dùng cùng một mã lỗi: {ma:?}"
        );
        assert_eq!(so_luong, 10, "mười variant; thêm variant phải cập nhật test này");
    }

    #[test]
    fn errors_without_a_command_have_null_command() {
        let err = GitError::NoRepositoryOpen;
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "no_repository_open");
        assert!(json["command"].is_null());
    }
}
