//! git-plum — ứng dụng desktop quản lý repository Git.
//!
//! Kiến trúc: giao diện React gọi xuống qua Tauri IPC; lớp Rust sinh tiến trình
//! `git`, phân tích đầu ra dạng byte, và trả JSON lên. Không dùng libgit2.
//!
//! Xem `.planning/research/ARCHITECTURE.md` để biết ranh giới các module.

pub mod cache;
pub mod commands;
pub mod domain;
pub mod error;
pub mod git;
pub mod graph;
pub mod state;

/// Trợ giúp tìm repo mẫu cho test và benchmark. `pub` vì benchmark của criterion là
/// target riêng và không thấy mã dưới `#[cfg(test)]`. Mã ứng dụng không dùng.
pub mod testing;
pub mod watch;

pub use error::{GitError, Result};
pub use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "git_plum_lib=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::open_repository,
            commands::list_repositories,
            commands::close_repository,
            commands::command_log,
            commands::clear_command_log,
            commands::git_version,
            commands::current_branch,
            commands::get_commit_page,
            commands::list_refs,
            commands::get_commit_detail,
            commands::search_commits,
            commands::get_file_diff,
            commands::get_file_history,
            commands::get_worktree_diff,
            commands::get_status,
            commands::stage_files,
            commands::unstage_files,
            commands::create_commit,
            commands::amend_commit,
            commands::avatar_hash,
            commands::spike_blob_pair,
        ])
        .run(tauri::generate_context!())
        .expect("không khởi động được ứng dụng Tauri");
}
