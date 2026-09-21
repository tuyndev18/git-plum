//! Lớp sinh tiến trình git — PLAT-02.
//!
//! Đây là **nơi duy nhất** trong toàn bộ ứng dụng được phép sinh tiến trình `git`.
//! Không có ngoại lệ. Mọi biến môi trường bên dưới đều tồn tại vì một lỗi cụ thể
//! đã từng xảy ra với các công cụ khác; xoá bất kỳ dòng nào sẽ làm lỗi đó quay lại.
//!
//! Xem `.planning/research/PITFALLS.md` và `docs/01-research-competitors.md` mục 5.0.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use crate::error::{GitError, Result};

/// Hạn giờ mặc định cho lệnh đọc dữ liệu cục bộ.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Hạn giờ cho lệnh có đi qua mạng (fetch, pull, push, clone).
/// Dài hơn, nhưng vẫn phải có — thiếu hạn giờ thì giao diện treo vô hạn.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(120);

/// Kết quả thô của một lần chạy git. Giữ nguyên dạng byte.
///
/// Không giải mã sang `String` ở đây. Tên tệp và thông điệp commit không bảo đảm
/// là UTF-8 hợp lệ; việc giải mã là chuyện của từng bộ phân tích, và với đường đi
/// của bản vá thì tuyệt đối không được giải mã (xem WORK-04).
#[derive(Debug)]
pub struct GitOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: i32,
}

impl GitOutput {
    pub fn is_success(&self) -> bool {
        self.status == 0
    }

    /// stderr dạng chuỗi, chỉ dùng để hiển thị thông báo lỗi cho người dùng.
    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).trim().to_string()
    }
}

/// Một lệnh git đã dựng xong, chưa chạy.
pub struct GitCommand {
    repo_path: PathBuf,
    args: Vec<String>,
    timeout: Duration,
    stdin_data: Option<Vec<u8>>,
}

impl GitCommand {
    /// Dựng một lệnh git chạy trong `repo_path`.
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
            args: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
            stdin_data: None,
        }
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_string_lossy().into_owned());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.args.push(a.as_ref().to_string_lossy().into_owned());
        }
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Đưa dữ liệu vào stdin của git. Dùng cho `git apply --cached -`.
    ///
    /// Nhận `Vec<u8>` chứ không phải `String`: bản vá phải đi qua dạng byte
    /// nguyên vẹn từ đầu tới cuối (WORK-04).
    pub fn stdin_bytes(mut self, data: Vec<u8>) -> Self {
        self.stdin_data = Some(data);
        self
    }

    /// Danh sách tham số, để hiển thị trong nhật ký lệnh (PLAT-08).
    pub fn display_args(&self) -> &[String] {
        &self.args
    }

    /// Chạy lệnh. Trả về `Err` khi không sinh được tiến trình hoặc quá hạn giờ;
    /// git thoát với mã khác 0 vẫn trả về `Ok` — người gọi tự quyết định.
    pub async fn run(self) -> Result<GitOutput> {
        let mut cmd = Command::new("git");

        cmd.current_dir(&self.repo_path);
        cmd.args(&self.args);

        apply_env_hardening(&mut cmd);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(if self.stdin_data.is_some() {
            Stdio::piped()
        } else {
            // Đóng stdin khi không dùng tới. Nếu để mở, git có thể chờ nhập
            // liệu vô hạn và treo cả giao diện.
            Stdio::null()
        });

        #[cfg(windows)]
        {
            // Không có cờ này thì mỗi lần gọi git sẽ nháy một cửa sổ console đen.
            // Chỉ thấy ở bản dựng release, KHÔNG thấy khi chạy `tauri dev`.
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd.spawn().map_err(|e| GitError::SpawnFailed {
            args: self.args.clone(),
            reason: e.to_string(),
        })?;

        if let Some(data) = self.stdin_data {
            use tokio::io::AsyncWriteExt;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(&data)
                    .await
                    .map_err(|e| GitError::StdinWriteFailed(e.to_string()))?;
                stdin.shutdown().await.ok();
            }
        }

        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| GitError::Timeout {
                args: self.args.clone(),
                seconds: self.timeout.as_secs(),
            })?
            .map_err(|e| GitError::SpawnFailed {
                args: self.args.clone(),
                reason: e.to_string(),
            })?;

        Ok(GitOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status: output.status.code().unwrap_or(-1),
        })
    }
}

/// Ghim môi trường cho tiến trình git.
///
/// Mỗi dòng dưới đây chống lại một lỗi cụ thể. Đọc kỹ trước khi sửa.
fn apply_env_hardening(cmd: &mut Command) {
    // --- Ngôn ngữ ---------------------------------------------------------
    // Không ghim thì trên máy đặt ngôn ngữ khác tiếng Anh, định dạng ngày và
    // thông báo lỗi đổi kiểu, bộ phân tích vỡ trên máy người dùng nhưng chạy
    // tốt trên máy lập trình viên.
    cmd.env("LC_ALL", "C");
    cmd.env("LANG", "C");
    cmd.env("LC_MESSAGES", "C");

    // --- Chặn mọi đường hỏi thông tin đăng nhập ---------------------------
    // Đây là cái bẫy nghiêm trọng nhất khi bọc git bằng giao diện đồ hoạ:
    // git chờ nhập mật khẩu trên stdin, ứng dụng treo vĩnh viễn, không báo gì.
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_ASKPASS", "");
    cmd.env("SSH_ASKPASS", "");
    cmd.env("GCM_INTERACTIVE", "never");

    // --- Loại bỏ ảnh hưởng từ cấu hình bên ngoài --------------------------
    // Chữ ký GPG in thêm dòng vào đầu ra của `git log`, làm lệch bộ phân tích.
    cmd.env("GIT_CONFIG_PARAMETERS", "'log.showSignature=false'");

    // Người dùng có thể đã đặt sẵn các biến này trong shell; thừa hưởng chúng
    // sẽ khiến commit do ứng dụng tạo ra mang danh tính sai.
    cmd.env_remove("GIT_AUTHOR_NAME");
    cmd.env_remove("GIT_AUTHOR_EMAIL");
    cmd.env_remove("GIT_AUTHOR_DATE");
    cmd.env_remove("GIT_COMMITTER_NAME");
    cmd.env_remove("GIT_COMMITTER_EMAIL");
    cmd.env_remove("GIT_COMMITTER_DATE");

    // Trình khác biệt bên ngoài sẽ thay thế đầu ra chuẩn của `git diff`.
    cmd.env("GIT_EXTERNAL_DIFF", "");

    // Trình phân trang chặn tiến trình chờ người dùng bấm phím.
    cmd.env("GIT_PAGER", "cat");
    cmd.env("PAGER", "cat");

    // Không để git đọc cấu hình toàn hệ thống của máy build trong lúc chạy test.
    // (Chỉ bật khi chạy kiểm thử; bản chạy thật cần cấu hình người dùng.)
    #[cfg(test)]
    {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1");
        cmd.env("HOME", "/nonexistent-git-plum-test");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lớp bọc phải chạy được lệnh cơ bản nhất.
    #[tokio::test]
    async fn runs_git_version() {
        let out = GitCommand::new(".").arg("--version").run().await.unwrap();
        assert!(out.is_success());
        assert!(out.stdout.starts_with(b"git version"));
    }

    /// Thư mục không phải repository phải trả mã khác 0 kèm stderr đọc được,
    /// không được treo và không được panic (PLAT-10).
    #[tokio::test]
    async fn non_repo_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let out = GitCommand::new(dir.path())
            .args(["rev-parse", "--git-dir"])
            .run()
            .await
            .unwrap();
        assert!(!out.is_success());
        assert!(!out.stderr_lossy().is_empty());
    }

    /// Hạn giờ phải bắn ra lỗi Timeout chứ không treo mãi.
    #[tokio::test]
    async fn timeout_fires() {
        // `git help --all` không treo, nên dùng hạn giờ cực ngắn để ép quá hạn.
        let res = GitCommand::new(".")
            .args(["help", "--all"])
            .timeout(Duration::from_nanos(1))
            .run()
            .await;
        assert!(matches!(res, Err(GitError::Timeout { .. })));
    }
}
