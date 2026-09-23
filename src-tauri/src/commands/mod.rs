//! Các Tauri command — lớp vỏ mỏng, mỗi tệp một nhóm chức năng.
//!
//! Command chỉ làm ba việc: nhận tham số, gọi xuống lớp git, trả kết quả.
//! Không chứa logic nghiệp vụ — logic nằm ở `git` và `domain` để kiểm thử được
//! mà không cần chạy Tauri.

pub mod avatar;
pub mod diff;
pub mod diff_spike;
pub mod history;
pub mod hunk;
pub mod commit;
pub mod repo;
pub mod trash;
pub mod worktree;

pub use avatar::*;
pub use diff::*;
pub use diff_spike::*;
pub use history::*;
pub use hunk::*;
pub use commit::*;
pub use repo::*;
pub use trash::*;
pub use worktree::*;
