//! Bộ nhớ đệm trong RAM, khoá theo `RepoId` — HIST-01, HIST-05, PLAT-05.
//!
//! Không có bộ nhớ đệm trên đĩa và không có chỉ mục tăng dần: CONTEXT.md ghi rõ hai
//! thứ đó là Out of Scope. Ở đây chỉ có RAM, và vì là RAM nên **phải có chặn trên** —
//! xem [`repo_cache::MAX_CACHED_HISTORIES`].

pub mod repo_cache;

pub use repo_cache::{RepoCache, RepoHistory, MAX_CACHED_HISTORIES};
