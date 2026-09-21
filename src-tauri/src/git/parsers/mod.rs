//! Bộ phân tích đầu ra của git — mỗi định dạng một module.
//!
//! Quy tắc chung cho mọi bộ phân tích ở đây:
//!
//! * Nhận `&[u8]`, không nhận `&str`. Đầu ra của git là byte; tên tệp trên Linux
//!   và thông điệp commit mã hoá cũ không bảo đảm là UTF-8 hợp lệ.
//! * Giải mã lossy chỉ được phép ở trường dùng để **hiển thị**.
//! * Trên đường đi của bản vá thì tuyệt đối không giải mã (xem WORK-04).
//! * **Không bao giờ** `String::from_utf8(..).unwrap()` hay `.expect()`: cả hai panic
//!   trên repo thật. Dùng `String::from_utf8_lossy` và đánh dấu bản ghi đã lossy.
//! * Một bản ghi hỏng **không** được làm hỏng cả trang. Bỏ nó, đếm nó, trả phần còn
//!   lại — xem `LogParseResult::skipped_records`.

pub mod log;
