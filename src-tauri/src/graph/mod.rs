//! Đồ thị nhánh: hình học từng dòng và thuật toán gán lane.
//!
//! Plan 02-02 khai báo các kiểu ở [`types`]. Thuật toán gán lane ở [`lanes`] (plan
//! 02-03) tiêu thụ `Vec<Commit>` theo thứ tự topo và sinh ra `Vec<GraphRow>` cùng
//! độ dài.

pub mod lanes;
pub mod types;

pub use lanes::assign;
pub use types::{Edge, GraphRow, LANE_COLORS, MAX_VISIBLE_LANES};
