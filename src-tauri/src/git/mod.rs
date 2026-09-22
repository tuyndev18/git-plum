//! Lớp git: sinh tiến trình, chạy có ghi nhật ký, và các bộ phân tích đầu ra.

pub mod exec;
pub mod parsers;
pub mod patch_build;
pub mod runner;

pub use exec::{GitCommand, GitOutput, DEFAULT_TIMEOUT, NETWORK_TIMEOUT};
pub use runner::GitRunner;
