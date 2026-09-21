//! Bộ phân tích đầu ra của git — mỗi định dạng một module.
//!
//! Quy tắc chung cho mọi bộ phân tích ở đây:
//!
//! * Nhận `&[u8]`, không nhận `&str`. Đầu ra của git là byte; tên tệp trên Linux
//!   và thông điệp commit mã hoá cũ không bảo đảm là UTF-8 hợp lệ.
//! * Giải mã lossy chỉ được phép ở trường dùng để **hiển thị**.
//! * Trên đường đi của bản vá thì tuyệt đối không giải mã (xem WORK-04).
//!
//! Phase 1 chưa có bộ phân tích nào — việc của phase này là chứng minh cầu IPC
//! và lớp sinh tiến trình chạy đúng trước khi đặt logic git lên trên.
