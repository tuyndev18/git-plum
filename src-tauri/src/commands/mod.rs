//! Các Tauri command — lớp vỏ mỏng, mỗi tệp một nhóm chức năng.
//!
//! Command chỉ làm ba việc: nhận tham số, gọi xuống lớp git, trả kết quả.
//! Không chứa logic nghiệp vụ — logic nằm ở `git` và `domain` để kiểm thử được
//! mà không cần chạy Tauri.

pub mod repo;

pub use repo::*;
