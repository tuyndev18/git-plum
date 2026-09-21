//! Chạy lệnh git có ghi nhật ký và có xếp hàng ghi.
//!
//! Đây là lớp mà phần còn lại của ứng dụng gọi tới. `exec.rs` lo việc sinh tiến
//! trình cho đúng; lớp này lo việc ghi nhật ký (PLAT-08) và giữ khoá ghi (PLAT-03).

use std::sync::Arc;
use std::time::Instant;

use crate::error::{GitError, Result};
use crate::git::exec::{GitCommand, GitOutput};
use crate::state::{CommandLog, RepoHandle};

pub struct GitRunner {
    repo: Arc<RepoHandle>,
    log: Arc<CommandLog>,
}

impl GitRunner {
    pub fn new(repo: Arc<RepoHandle>, log: Arc<CommandLog>) -> Self {
        Self { repo, log }
    }

    /// Chạy một lệnh **chỉ đọc**. Không giữ khoá ghi, nên nhiều lệnh đọc chạy
    /// song song được.
    pub async fn read(&self, cmd: GitCommand) -> Result<GitOutput> {
        self.run_logged(cmd).await
    }

    /// Chạy một lệnh **làm thay đổi repository** — PLAT-03.
    ///
    /// Giữ khoá ghi của repository trong suốt thời gian chạy. Hai lệnh ghi không
    /// bao giờ chồng nhau, nên không đụng `index.lock`.
    pub async fn write(&self, cmd: GitCommand) -> Result<GitOutput> {
        let lock = self.repo.write_lock();
        let _guard = lock.lock().await;
        self.run_logged(cmd).await
    }

    /// Chạy lệnh đọc và đòi hỏi nó phải thành công.
    pub async fn read_ok(&self, cmd: GitCommand) -> Result<GitOutput> {
        let out = self.read(cmd).await?;
        ensure_success(out)
    }

    /// Chạy lệnh ghi và đòi hỏi nó phải thành công.
    pub async fn write_ok(&self, cmd: GitCommand) -> Result<GitOutput> {
        let out = self.write(cmd).await?;
        ensure_success(out)
    }

    async fn run_logged(&self, cmd: GitCommand) -> Result<GitOutput> {
        let args: Vec<String> = cmd.display_args().to_vec();
        let cwd = self.repo.path.to_string_lossy().into_owned();
        let started = Instant::now();

        let result = cmd.run().await;
        let elapsed = started.elapsed();

        match &result {
            Ok(out) => {
                let error = if out.is_success() {
                    None
                } else {
                    Some(out.stderr_lossy())
                };
                self.log
                    .record(&args, &cwd, Some(out.status), elapsed, error);
            }
            Err(e) => {
                // Lệnh không chạy nổi hoặc quá hạn giờ: vẫn phải xuất hiện trong
                // nhật ký, nếu không người dùng sẽ không hiểu vì sao không có gì
                // xảy ra.
                self.log
                    .record(&args, &cwd, None, elapsed, Some(e.to_string()));
            }
        }

        result
    }
}

/// Biến mã thoát khác 0 thành lỗi có kiểu, giữ lại stderr cho người dùng đọc.
fn ensure_success(out: GitOutput) -> Result<GitOutput> {
    if out.is_success() {
        return Ok(out);
    }
    Err(GitError::CommandFailed {
        args: Vec::new(),
        status: out.status,
        stderr: out.stderr_lossy(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    /// Trả về cả `AppState` để nó sống đủ lâu — nhật ký thuộc về state.
    fn runner_for(path: &std::path::Path) -> (AppState, GitRunner, Arc<CommandLog>) {
        let state = AppState::new();
        let repo = state.open_repo(path);
        let log = Arc::clone(&state.command_log);
        let runner = state.runner(repo);
        (state, runner, log)
    }

    #[tokio::test]
    async fn successful_command_lands_in_the_log() {
        let dir = tempfile::tempdir().unwrap();
        let (_state, runner, log) = runner_for(dir.path());

        let out = runner
            .read(GitCommand::new(dir.path()).arg("--version"))
            .await
            .unwrap();
        assert!(out.is_success());

        let entries = log.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].command, "git --version");
        assert_eq!(entries[0].exit_code, Some(0));
        assert!(entries[0].error.is_none());
    }

    #[tokio::test]
    async fn failing_command_records_its_error() {
        let dir = tempfile::tempdir().unwrap();
        let (_state, runner, log) = runner_for(dir.path());

        let out = runner
            .read(GitCommand::new(dir.path()).args(["rev-parse", "--git-dir"]))
            .await
            .unwrap();
        assert!(!out.is_success());

        let entries = log.entries();
        assert_eq!(entries[0].exit_code, Some(128));
        assert!(entries[0].error.is_some());
    }

    #[tokio::test]
    async fn read_ok_converts_failure_into_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let (_state, runner, _log) = runner_for(dir.path());

        let res = runner
            .read_ok(GitCommand::new(dir.path()).args(["rev-parse", "--git-dir"]))
            .await;
        assert!(matches!(res, Err(GitError::CommandFailed { .. })));
    }
}
