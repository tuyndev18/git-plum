//! Các Tauri command — lớp vỏ mỏng, mỗi tệp một nhóm chức năng.
//!
//! Command chỉ làm ba việc: nhận tham số, gọi xuống lớp git, trả kết quả.
//! Không chứa logic nghiệp vụ — logic nằm ở `git` và `domain` để kiểm thử được
//! mà không cần chạy Tauri.

pub mod diff;
pub mod diff_spike;
pub mod history;
pub mod repo;
pub mod worktree;

pub use diff::*;
pub use diff_spike::*;
pub use history::*;
pub use repo::*;
pub use worktree::*;
