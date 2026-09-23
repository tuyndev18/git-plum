//! Kiểu lỗi dùng chung cho mọi Tauri command.
//!
//! Tauri đòi kiểu lỗi phải impl `Serialize`. `thiserror` chỉ sinh `Display`,
//! nên phần `Serialize` viết tay ở cuối tệp. Đây là lý do `anyhow` không dùng
//! được ở ranh giới IPC.

use serde::{Serialize, Serializer};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GitError>;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("Không sinh được tiến trình git: {reason}")]
    SpawnFailed { args: Vec<String>, reason: String },

    #[error("Lệnh git quá hạn {seconds} giây")]
    Timeout { args: Vec<String>, seconds: u64 },

    #[error("Không ghi được dữ liệu vào stdin của git: {0}")]
    StdinWriteFailed(String),

    #[error("git thoát với mã {status}: {stderr}")]
    CommandFailed {
        args: Vec<String>,
        status: i32,
        stderr: String,
    },

    #[error("Không phải một repository git: {path}")]
    NotARepository { path: String },

    /// `.git/index.lock` đang bị một tiến trình git khác giữ — WORK-02, CONTEXT.md 2.3.
    ///
    /// # Vì sao ca này có mã lỗi RIÊNG, không phải `CommandFailed`
    ///
    /// Đây **không** phải một lỗi; nó là một trạng thái tạm thời có thể phục hồi, và
    /// người dùng cần một câu khác hẳn: *"đang chờ một tiến trình git khác"* chứ không
    /// phải *"git thoát với mã 128"*. Chủ dự án **dùng terminal song song** — đó là cả
    /// điểm của WORK-10 — nên ca này sẽ xảy ra thật, không phải phòng xa.
    ///
    /// Gộp nó vào [`GitError::CommandFailed`] là lặp lại đúng khuôn lỗi của
    /// `open_repository`, nơi "exit khác 0" bị coi là **một** nguyên nhân và người dùng
    /// nhận "Không phải một repository git" cho một repo hoàn toàn hợp lệ mà git từ
    /// chối vì `dubious ownership`.
    ///
    /// `stderr` giữ **nguyên văn** câu trả lời của git: nó là văn bản chẩn đoán mờ, để
    /// người đọc, không phải một hợp đồng phân tích được.
    #[error(
        "Không ghi được vào repository: một tiến trình git khác đang giữ \
         .git/index.lock. Bạn có thể đang chạy git ở nơi khác (terminal, IDE, \
         hay một lệnh rebase chưa xong). Thử lại sau khi lệnh đó kết thúc.\n\n{stderr}"
    )]
    IndexLocked { args: Vec<String>, stderr: String },

    /// Thông điệp commit rỗng hoặc chỉ khoảng trắng — WORK-08, rủi ro R5.
    ///
    /// # Vì sao ca này có mã riêng và bị chặn TRƯỚC khi chạy git
    ///
    /// git cũng từ chối thông điệp rỗng, nên variant này trông như thừa. Nó không thừa:
    /// trên một repo có hook, `commit-msg` chạy **trước** phép kiểm rỗng của git, nên
    /// người dùng nhận đầu ra của hook thay vì câu "thông điệp rỗng" và đi sửa nhầm
    /// chỗ. Cộng thêm: chạy `pre-commit` (thường là linter cả cây, hàng chục giây) cho
    /// một lệnh chắc chắn thất bại là lãng phí rất thật.
    ///
    /// 🔴 **Không tự sửa thông điệp** (R5). Ứng dụng không sinh một subject mặc định,
    /// không thêm nội dung. Nó nói cho người dùng biết và dừng lại.
    #[error("Thông điệp commit không được để trống.")]
    EmptyCommitMessage,

    /// Không có gì ở index để commit — WORK-08.
    ///
    /// Tách khỏi [`GitError::CommandFailed`] vì với người dùng đây **không phải một
    /// lỗi**: họ chỉ chưa chọn tệp nào. Thông báo phải nói việc cần làm ("chọn tệp để
    /// stage"), không phải "git thoát với mã 1".
    ///
    /// 🔴 Câu này git in ra **stdout**, không phải stderr — đã đo. Nên `output` mang
    /// **cả hai** luồng; đọc một luồng là mất ca này im lặng.
    #[error("Không có thay đổi nào đã stage để commit. Chọn tệp để stage trước.\n\n{output}")]
    NothingToCommit { output: String },

    /// `git commit` bị từ chối — hook, hoặc một nguyên nhân chưa phân loại được.
    ///
    /// # `output` mang NGUYÊN VĂN đầu ra, và đó là cả điểm
    ///
    /// Rủi ro R5 của `CONTEXT.md`: *"Không tự sửa thông điệp; hiện nguyên văn lỗi
    /// hook."* Đầu ra của một `pre-commit` hook là thứ **duy nhất** nói cho người dùng
    /// biết phải sửa gì — nó là tên tệp, số dòng, luật linter bị vi phạm. Thay nó bằng
    /// một câu chung ("commit thất bại") là vứt đi toàn bộ thông tin có ích, đúng lúc
    /// người ta cần đọc nó nhất. Đột biến M5 của plan ghim điều này.
    ///
    /// # Vì sao KHÔNG cố tách "hook" khỏi "chưa biết"
    ///
    /// Đầu ra hook là chuỗi **tuỳ ý do người dùng viết** — không có dấu hiệu ổn định
    /// nào để nhận ra nó, và mọi phép đoán sẽ sai ở một repo nào đó. Gộp hai ca là
    /// trung thực: *"git từ chối, đây là nguyên văn nó nói"*. Cố đoán rồi đoán sai là
    /// đúng khuôn lỗi KB-4b của `open_repository`.
    ///
    /// 🔴 `output` gộp **cả stdout lẫn stderr**: đã đo rằng git chuyển tiếp **cả hai**
    /// luồng của hook vào **stderr** của chính nó, trong khi các ca khác dùng stdout.
    #[error("git từ chối tạo commit (mã {status}):\n\n{output}")]
    HookRejected {
        args: Vec<String>,
        status: i32,
        output: String,
    },

    #[error("Chưa mở repository nào")]
    NoRepositoryOpen,

    #[error("Không tìm thấy repository với mã {0}")]
    UnknownRepository(String),

    #[error("Không đọc được đầu ra của git: {0}")]
    ParseFailed(String),

    #[error("Lỗi nhập xuất: {0}")]
    Io(String),

    /// Tệp đã đổi trên đĩa giữa lúc giao diện vẽ diff và lúc người dùng bấm stage —
    /// WORK-05, rủi ro R4.
    ///
    /// # Vì sao ca này KHÔNG được gộp vào `CommandFailed`
    ///
    /// Với người dùng đây **không phải một lỗi của ứng dụng**: tệp của họ đổi, và việc
    /// cần làm là **làm mới** rồi chọn lại khối. `git thoát với mã 1` không nói điều
    /// đó. Giao diện phải phân nhánh được để hiện nút "làm mới", nên nó cần một mã ổn
    /// định chứ không phải một phép so khớp chuỗi.
    ///
    /// Chủ dự án **chạy git ở terminal song song** — đó là cả điểm của WORK-10 — nên
    /// ca này là đường đi thường ngày, không phải phòng xa.
    ///
    /// # 🔴 Thông điệp là NGUYÊN VĂN ROADMAP
    ///
    /// *"tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới"*. Tiêu chí thành công 3
    /// của Phase 5 được kiểm bằng **chính chuỗi này**; đổi chữ ở đây làm tiêu chí đó
    /// không còn kiểm được.
    ///
    /// # 🔴 Sau lỗi này KHÔNG BAO GIỜ có một lần thử lại lỏng hơn
    ///
    /// ROADMAP xếp "không thương lượng": không `--whitespace=fix`, không `--3way`,
    /// không khớp mờ. Một bản vá không khớp được áp bằng khớp mờ là cách làm **hỏng
    /// tệp của người dùng**, và đây là phase đầu tiên ghi vào nội dung tệp. Cổng
    /// `duong_apply_khong_bao_gio_khop_mo` trong `commands/hunk.rs` ghim điều này.
    #[error("Tệp đã đổi từ lúc bạn xem khác biệt này, hãy làm mới")]
    FileChanged { path: String },
}

impl GitError {
    /// Mã lỗi ổn định để phía giao diện phân nhánh xử lý, thay vì so khớp chuỗi.
    pub fn code(&self) -> &'static str {
        match self {
            Self::SpawnFailed { .. } => "spawn_failed",
            Self::Timeout { .. } => "timeout",
            Self::StdinWriteFailed(_) => "stdin_write_failed",
            Self::CommandFailed { .. } => "command_failed",
            Self::NotARepository { .. } => "not_a_repository",
            Self::IndexLocked { .. } => "index_locked",
            Self::EmptyCommitMessage => "empty_commit_message",
            Self::NothingToCommit { .. } => "nothing_to_commit",
            Self::HookRejected { .. } => "hook_rejected",
            Self::NoRepositoryOpen => "no_repository_open",
            Self::UnknownRepository(_) => "unknown_repository",
            Self::ParseFailed(_) => "parse_failed",
            Self::Io(_) => "io",
            Self::FileChanged { .. } => "file_changed",
        }
    }

    /// Tham số của lệnh git đã chạy, nếu lỗi này sinh ra từ một lệnh cụ thể.
    /// Giao diện hiển thị kèm thông báo lỗi để người dùng biết chuyện gì xảy ra
    /// (PLAT-10).
    pub fn command_args(&self) -> Option<&[String]> {
        match self {
            Self::SpawnFailed { args, .. }
            | Self::Timeout { args, .. }
            | Self::CommandFailed { args, .. }
            | Self::IndexLocked { args, .. }
            | Self::HookRejected { args, .. } => Some(args),
            _ => None,
        }
    }
}

impl From<std::io::Error> for GitError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

/// Dạng lỗi gửi sang phía giao diện.
#[derive(Serialize)]
struct SerializedError<'a> {
    code: &'a str,
    message: String,
    /// Lệnh git đã chạy, dạng `git log --format=... --all`, để hiện trong
    /// thông báo lỗi và trong nhật ký lệnh.
    command: Option<String>,
}

impl Serialize for GitError {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let command = self
            .command_args()
            .map(|args| format!("git {}", args.join(" ")));

        SerializedError {
            code: self.code(),
            message: self.to_string(),
            command,
        }
        .serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_with_code_and_command() {
        let err = GitError::CommandFailed {
            args: vec!["status".into(), "--porcelain=v2".into()],
            status: 128,
            stderr: "fatal: not a git repository".into(),
        };
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "command_failed");
        assert_eq!(json["command"], "git status --porcelain=v2");
        assert!(json["message"].as_str().unwrap().contains("128"));
    }

    /// Ca `index.lock` có mã **riêng** và nói rõ người dùng đang chờ cái gì.
    ///
    /// 🔴 Khẳng định mã **không** phải `command_failed`: gộp hai ca lại là lặp lại
    /// khuôn lỗi của `open_repository` (CONTEXT.md mục 0), nơi mọi exit khác 0 bị coi
    /// là một nguyên nhân duy nhất.
    #[test]
    fn index_locked_co_ma_rieng_va_thong_bao_noi_ro_dang_cho_gi() {
        let err = GitError::IndexLocked {
            args: vec!["add".into(), "--".into(), "a.txt".into()],
            stderr: "fatal: Unable to create '.../index.lock': File exists.".into(),
        };
        let json = serde_json::to_value(&err).unwrap();

        assert_eq!(json["code"], "index_locked");
        assert_ne!(
            json["code"], "command_failed",
            "ca lock KHÔNG được gộp vào command_failed — giao diện phải nói khác"
        );

        let msg = json["message"].as_str().unwrap();
        assert!(
            msg.contains("index.lock"),
            "thông báo phải nói tên tệp đang bị giữ: {msg:?}"
        );
        assert!(
            msg.contains("terminal") || msg.contains("nơi khác"),
            "thông báo phải nói người dùng có thể đang chạy git ở nơi khác: {msg:?}"
        );
        assert!(
            msg.contains("File exists"),
            "stderr NGUYÊN VĂN của git phải đi kèm — nó là văn bản chẩn đoán: {msg:?}"
        );
        assert_eq!(
            json["command"], "git add -- a.txt",
            "lệnh đã chạy phải hiện ra (PLAT-10)"
        );
    }

    /// Mọi mã lỗi phải **khác nhau đôi một**: hai variant cùng mã làm giao diện không
    /// phân nhánh được, và lỗi đó không gây lỗi biên dịch ở bên nào.
    #[test]
    fn moi_ma_loi_khac_nhau_doi_mot() {
        let mau = [
            GitError::SpawnFailed {
                args: vec![],
                reason: String::new(),
            },
            GitError::Timeout {
                args: vec![],
                seconds: 1,
            },
            GitError::StdinWriteFailed(String::new()),
            GitError::CommandFailed {
                args: vec![],
                status: 1,
                stderr: String::new(),
            },
            GitError::NotARepository {
                path: String::new(),
            },
            GitError::IndexLocked {
                args: vec![],
                stderr: String::new(),
            },
            GitError::EmptyCommitMessage,
            GitError::NothingToCommit {
                output: String::new(),
            },
            GitError::HookRejected {
                args: vec![],
                status: 1,
                output: String::new(),
            },
            GitError::NoRepositoryOpen,
            GitError::UnknownRepository(String::new()),
            GitError::ParseFailed(String::new()),
            GitError::Io(String::new()),
            GitError::FileChanged {
                path: String::new(),
            },
        ];

        let mut ma: Vec<&str> = mau.iter().map(|e| e.code()).collect();
        let so_luong = ma.len();
        ma.sort_unstable();
        ma.dedup();
        assert_eq!(
            ma.len(),
            so_luong,
            "có hai variant dùng cùng một mã lỗi: {ma:?}"
        );
        assert_eq!(
            so_luong, 14,
            "mười bốn variant; thêm variant phải cập nhật test này. Lịch sử con số: \
             10 (hết 04-02) → 13 (04-03 thêm EmptyCommitMessage, NothingToCommit, \
             HookRejected cho vòng commit của WORK-08) → 14 (05-02 thêm FileChanged \
             cho WORK-05). Cập nhật con số là ĐÚNG khi \
             thêm variant thật; nới nó thành `>=` thì KHÔNG — phép so bằng tuyệt đối \
             là thứ bắt được một variant thêm vào mà quên khai mã lỗi"
        );
    }

    /// Ba variant của vòng commit giữ **nguyên văn** đầu ra git — đột biến M5.
    ///
    /// 🔴 Đây là cổng bảo vệ công sức người dùng. Đầu ra của một `pre-commit` hook là
    /// thứ **duy nhất** nói cho họ biết phải sửa gì (tên tệp, số dòng, luật bị vi
    /// phạm); thay nó bằng "commit thất bại" là vứt đi toàn bộ thông tin có ích đúng
    /// lúc họ cần nhất.
    #[test]
    fn loi_vong_commit_giu_nguyen_van_dau_ra_hook() {
        let err = GitError::HookRejected {
            args: vec!["commit".into(), "--cleanup=whitespace".into()],
            status: 1,
            output: "HOOK-PRE-COMMIT-REJECTED (stdout)\nHOOK-PRE-COMMIT-REJECTED (stderr)".into(),
        };
        let json = serde_json::to_value(&err).unwrap();

        assert_eq!(json["code"], "hook_rejected");
        assert_ne!(
            json["code"], "command_failed",
            "hook từ chối KHÔNG được gộp vào command_failed: với người dùng đó là một \
             thông báo cần ĐỌC, không phải một lỗi của ứng dụng"
        );

        let msg = json["message"].as_str().unwrap();
        assert!(
            msg.contains("HOOK-PRE-COMMIT-REJECTED"),
            "🔴 NGUYÊN VĂN đầu ra hook phải đi tới người dùng (R5, đột biến M5): {msg:?}"
        );
        assert!(
            msg.contains("(stdout)") && msg.contains("(stderr)"),
            "🔴 CẢ HAI luồng của hook phải tới nơi. git gộp stdout lẫn stderr của hook \
             vào stderr của chính nó — đã đo — nên mất một dòng nghĩa là phép gộp luồng \
             đang bỏ sót: {msg:?}"
        );

        // Ca "không có gì để commit" nói VIỆC CẦN LÀM, không nói mã thoát.
        let khong_co_gi = GitError::NothingToCommit {
            output: "nothing to commit, working tree clean".into(),
        };
        let msg2 = khong_co_gi.to_string();
        assert!(
            msg2.contains("stage"),
            "thông báo phải nói việc cần làm (chọn tệp để stage), không phải 'git \
             thoát với mã 1' — với người dùng đây không phải một lỗi: {msg2:?}"
        );
        assert_eq!(khong_co_gi.code(), "nothing_to_commit");

        // Thông điệp rỗng KHÔNG mang lệnh git: nó bị chặn TRƯỚC khi chạy git.
        let rong = GitError::EmptyCommitMessage;
        assert_eq!(rong.code(), "empty_commit_message");
        assert!(
            rong.command_args().is_none(),
            "🔴 ca thông điệp rỗng phải bị chặn TRƯỚC khi sinh lệnh git, nên nó không \
             có lệnh để hiện. Có lệnh ở đây nghĩa là git đã chạy — tức hook pre-commit \
             (thường là linter cả cây) vừa chạy cho một lệnh chắc chắn thất bại"
        );
    }

    /// `FileChanged` có mã **riêng** `file_changed` — WORK-05, tiêu chí thành công 3.
    ///
    /// 🔴 Khẳng định mã **không** phải `command_failed`: gộp hai ca lại thì giao diện
    /// không phân nhánh được để hiện nút "làm mới", và người dùng nhận "git thoát với
    /// mã 1" cho một tình huống hoàn toàn bình thường (họ vừa sửa tệp ở terminal).
    #[test]
    fn file_changed_co_ma_rieng() {
        let err = GitError::FileChanged {
            path: "src/App.tsx".into(),
        };
        let json = serde_json::to_value(&err).unwrap();

        assert_eq!(json["code"], "file_changed");
        assert_ne!(
            json["code"], "command_failed",
            "ca tệp-đã-đổi KHÔNG được gộp vào command_failed — giao diện phải phân \
             nhánh được để hiện đường làm mới"
        );
        assert_eq!(err.code(), "file_changed");
    }

    /// 🔴 Liệt kê **TOÀN BỘ** khoá JSON rồi so **bằng**, không chỉ kiểm khoá mình mong
    /// có mặt.
    ///
    /// # Vì sao phép so bằng, không phải `contains_key`
    ///
    /// Đây là cách `binary_khong_mang_byte_noi_dung` bắt được lỗi `rename_all` ở
    /// Phase 3 (ghi trong `domain/diff.rs`): một phép kiểm "có khoá `code`" vẫn **xanh**
    /// khi serde thêm một khoá thứ tư, hay khi ai đó đổi `#[serde(rename_all)]` và làm
    /// `command` thành `commandArgs` **cộng thêm** một khoá cũ. Phía TypeScript đọc
    /// đúng ba khoá này; một khoá thừa là một hợp đồng đã đổi mà không ai thấy.
    ///
    /// Và `command` phải là `null` — `FileChanged` **không** sinh từ một lệnh git cụ
    /// thể (phép kiểm blob hash chạy trước, và bản vá chưa bao giờ được áp), nên
    /// `command_args()` để nó rơi vào nhánh `_ => None`.
    #[test]
    fn file_changed_serialize_dung_ba_khoa_va_khong_thua() {
        let err = GitError::FileChanged {
            path: "a.txt".into(),
        };
        let json = serde_json::to_value(&err).unwrap();

        let mut khoa: Vec<&str> = json
            .as_object()
            .expect("lỗi phải serialize thành một object JSON")
            .keys()
            .map(String::as_str)
            .collect();
        khoa.sort_unstable();

        assert_eq!(
            khoa,
            vec!["code", "command", "message"],
            "🔴 TOÀN BỘ khoá phải đúng ba cái này. Một khoá THỪA nghĩa là hợp đồng với \
             TypeScript đã đổi mà không ai thấy; một khoá THIẾU nghĩa là giao diện đọc \
             `undefined`. Phép kiểm 'có khoá X' xanh ở cả hai ca — đó là lý do test \
             này so BẰNG. Đọc được {khoa:?}"
        );

        assert!(
            json["command"].is_null(),
            "🔴 `FileChanged` bị chặn TRƯỚC khi áp bản vá, nên không có lệnh git nào \
             để hiện. Có lệnh ở đây nghĩa là một `git apply` đã chạy rồi — tức phép \
             kiểm blob hash không còn đứng trước nó"
        );
    }

    /// Thông điệp chứa **nguyên văn** câu ROADMAP đòi — tiêu chí thành công 3.
    ///
    /// 🔴 Tiêu chí *"tệp đổi ở nơi khác → báo và đòi làm mới"* được kiểm bằng **chính
    /// chuỗi này**. Đổi chữ ở đây (dù chỉ thành "hãy tải lại") làm tiêu chí đó không
    /// còn kiểm được bằng máy, và không ai sẽ nhận ra cho tới lúc có người đọc lại
    /// ROADMAP.
    #[test]
    fn file_changed_noi_nguyen_van_hay_lam_moi() {
        let msg = GitError::FileChanged {
            path: "a.txt".into(),
        }
        .to_string();

        assert!(
            msg.contains("hãy làm mới"),
            "🔴 thông điệp phải chứa NGUYÊN VĂN `hãy làm mới` — đó là việc người dùng \
             cần làm, và là chuỗi mà tiêu chí thành công 3 kiểm: {msg:?}"
        );
        assert!(
            msg.contains("đã đổi"),
            "thông điệp phải nói tệp đã đổi, không chỉ nói phải làm mới: {msg:?}"
        );
    }

    #[test]
    fn errors_without_a_command_have_null_command() {
        let err = GitError::NoRepositoryOpen;
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "no_repository_open");
        assert!(json["command"].is_null());
    }
}
