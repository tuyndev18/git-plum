//! Kiểu dữ liệu miền — struct thuần, không biết gì về git hay về Tauri.
//!
//! Tách riêng để kiểm thử được mà không cần repository thật và không cần webview.
//!
//! Đây là **hợp đồng dữ liệu** của phase 2: bộ phân tích (`git::parsers`) sinh ra
//! chúng, thuật toán đồ thị (`graph`) đọc chúng, và Tauri command serialize chúng sang
//! webview. Mọi trường đều `#[serde(rename_all = "camelCase")]` vì phía TypeScript đọc
//! đúng những tên đó.

pub mod commit;
pub mod refs;

pub use commit::Commit;
pub use refs::{Ref, RefKind};
