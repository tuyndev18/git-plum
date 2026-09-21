//! Command mở và đóng repository, và đọc nhật ký lệnh.
//!
//! PLAT-06, PLAT-07, PLAT-08, PLAT-10.

use std::sync::Arc;

use tauri::State;

use crate::error::{GitError, Result};
use crate::git::GitCommand;
use crate::state::{AppState, CommandLogEntry, RepoInfo};

/// Mở một repository theo đường dẫn — PLAT-06.
///
/// Kiểm tra đường dẫn thật sự là một repository trước khi ghi vào trạng thái.
/// Thư mục không hợp lệ phải trả lỗi đọc hiểu được, không được treo (PLAT-10).
#[tauri::command]
pub async fn open_repository(path: String, state: State<'_, AppState>) -> Result<RepoInfo> {
    // `rev-parse --show-toplevel` vừa xác nhận đây là repository, vừa trả về
    // thư mục gốc — mở một thư mục con thì vẫn ra đúng repository cha.
    let out = GitCommand::new(&path)
        .args(["rev-parse", "--show-toplevel"])
        .run()
        .await?;

    // Ghi vào nhật ký kể cả khi thất bại, để người dùng thấy ứng dụng đã thử gì.
    state.command_log.record(
        &["rev-parse".into(), "--show-toplevel".into()],
        &path,
        Some(out.status),
        std::time::Duration::ZERO,
        if out.is_success() {
            None
        } else {
            Some(out.stderr_lossy())
        },
    );

    if !out.is_success() {
        return Err(GitError::NotARepository { path });
    }

    let toplevel = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if toplevel.is_empty() {
        return Err(GitError::NotARepository { path });
    }

    let handle = state.open_repo(&toplevel);
    Ok(RepoInfo::from(handle.as_ref()))
}

/// Danh sách repository đang mở. v1 chỉ có tối đa một, nhưng API trả về danh
/// sách ngay từ đầu (PLAT-05).
#[tauri::command]
pub fn list_repositories(state: State<'_, AppState>) -> Vec<RepoInfo> {
    state.list_repos()
}

#[tauri::command]
pub fn close_repository(id: String, state: State<'_, AppState>) {
    state.close_repo(&id);
}

/// Nhật ký lệnh git, mới nhất trước — PLAT-08.
#[tauri::command]
pub fn command_log(state: State<'_, AppState>) -> Vec<CommandLogEntry> {
    state.command_log.entries()
}

#[tauri::command]
pub fn clear_command_log(state: State<'_, AppState>) {
    state.command_log.clear();
}

/// Phiên bản git trên máy. Dùng để hiện trong phần Giới thiệu, và để kiểm tra
/// sớm rằng lớp bọc tiến trình chạy được.
#[tauri::command]
pub async fn git_version(state: State<'_, AppState>) -> Result<String> {
    let repo = state.active_repo();
    let cwd = repo
        .as_ref()
        .map(|r| r.path.clone())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let out = GitCommand::new(&cwd).arg("--version").run().await?;

    if !out.is_success() {
        return Err(GitError::CommandFailed {
            args: vec!["--version".into()],
            status: out.status,
            stderr: out.stderr_lossy(),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Tên nhánh đang checkout của repository đang hoạt động.
///
/// Đây là lệnh đầu tiên đọc dữ liệu thật từ repository, dùng để chứng minh
/// toàn bộ chuỗi giao diện → IPC → lớp bọc → git chạy thông (tiêu chí thành
/// công của Phase 1).
#[tauri::command]
pub async fn current_branch(state: State<'_, AppState>) -> Result<String> {
    let repo = state.active_repo().ok_or(GitError::NoRepositoryOpen)?;
    let runner = state.runner(Arc::clone(&repo));

    let out = runner
        .read(GitCommand::new(&repo.path).args(["rev-parse", "--abbrev-ref", "HEAD"]))
        .await?;

    if !out.is_success() {
        return Err(GitError::CommandFailed {
            args: vec!["rev-parse".into(), "--abbrev-ref".into(), "HEAD".into()],
            status: out.status,
            stderr: out.stderr_lossy(),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
