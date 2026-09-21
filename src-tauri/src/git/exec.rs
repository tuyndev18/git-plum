//! Lớp sinh tiến trình git — PLAT-02.
//!
//! Đây là **nơi duy nhất** trong toàn bộ ứng dụng được phép sinh tiến trình `git`.
//! Không có ngoại lệ. Mọi biến môi trường bên dưới đều tồn tại vì một lỗi cụ thể
//! đã từng xảy ra với các công cụ khác; xoá bất kỳ dòng nào sẽ làm lỗi đó quay lại.
//!
//! Xem `.planning/research/PITFALLS.md` và `docs/01-research-competitors.md` mục 5.0.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;

use crate::error::{GitError, Result};

/// Hạn giờ mặc định cho lệnh đọc dữ liệu cục bộ.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Hạn giờ cho lệnh có đi qua mạng (fetch, pull, push, clone).
/// Dài hơn, nhưng vẫn phải có — thiếu hạn giờ thì giao diện treo vô hạn.
pub const NETWORK_TIMEOUT: Duration = Duration::from_secs(120);

/// Kết quả thô của một lần chạy git. Giữ nguyên dạng byte.
///
/// Không giải mã sang `String` ở đây. Tên tệp và thông điệp commit không bảo đảm
/// là UTF-8 hợp lệ; việc giải mã là chuyện của từng bộ phân tích, và với đường đi
/// của bản vá thì tuyệt đối không được giải mã (xem WORK-04).
#[derive(Debug)]
pub struct GitOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: i32,
}

impl GitOutput {
    pub fn is_success(&self) -> bool {
        self.status == 0
    }

    /// stderr dạng chuỗi, chỉ dùng để hiển thị thông báo lỗi cho người dùng.
    pub fn stderr_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stderr).trim().to_string()
    }
}

/// Một lệnh git đã dựng xong, chưa chạy.
pub struct GitCommand {
    repo_path: PathBuf,
    args: Vec<String>,
    timeout: Duration,
    stdin_data: Option<Vec<u8>>,
}

impl GitCommand {
    /// Dựng một lệnh git chạy trong `repo_path`.
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
            args: Vec::new(),
            timeout: DEFAULT_TIMEOUT,
            stdin_data: None,
        }
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_string_lossy().into_owned());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        for a in args {
            self.args.push(a.as_ref().to_string_lossy().into_owned());
        }
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Đưa dữ liệu vào stdin của git. Dùng cho `git apply --cached -`.
    ///
    /// Nhận `Vec<u8>` chứ không phải `String`: bản vá phải đi qua dạng byte
    /// nguyên vẹn từ đầu tới cuối (WORK-04).
    pub fn stdin_bytes(mut self, data: Vec<u8>) -> Self {
        self.stdin_data = Some(data);
        self
    }

    /// Danh sách tham số, để hiển thị trong nhật ký lệnh (PLAT-08).
    pub fn display_args(&self) -> &[String] {
        &self.args
    }

    /// Chạy lệnh. Trả về `Err` khi không sinh được tiến trình hoặc quá hạn giờ;
    /// git thoát với mã khác 0 vẫn trả về `Ok` — người gọi tự quyết định.
    pub async fn run(self) -> Result<GitOutput> {
        let mut cmd = Command::new("git");

        cmd.current_dir(&self.repo_path);
        cmd.args(&self.args);

        apply_env_hardening(&mut cmd);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(if self.stdin_data.is_some() {
            Stdio::piped()
        } else {
            // Đóng stdin khi không dùng tới. Nếu để mở, git có thể chờ nhập
            // liệu vô hạn và treo cả giao diện.
            Stdio::null()
        });

        #[cfg(windows)]
        {
            // Không có cờ này thì mỗi lần gọi git sẽ nháy một cửa sổ console đen.
            // Chỉ thấy ở bản dựng release, KHÔNG thấy khi chạy `tauri dev`.
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = cmd.spawn().map_err(|e| GitError::SpawnFailed {
            args: self.args.clone(),
            reason: e.to_string(),
        })?;

        if let Some(data) = self.stdin_data {
            use tokio::io::AsyncWriteExt;
            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(&data)
                    .await
                    .map_err(|e| GitError::StdinWriteFailed(e.to_string()))?;
                stdin.shutdown().await.ok();
            }
        }

        let output = tokio::time::timeout(self.timeout, child.wait_with_output())
            .await
            .map_err(|_| GitError::Timeout {
                args: self.args.clone(),
                seconds: self.timeout.as_secs(),
            })?
            .map_err(|e| GitError::SpawnFailed {
                args: self.args.clone(),
                reason: e.to_string(),
            })?;

        Ok(GitOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status: output.status.code().unwrap_or(-1),
        })
    }
}

/// Các khoá cấu hình git bị ghim cho mọi tiến trình con, dạng của
/// `GIT_CONFIG_PARAMETERS` (git ≥ 2.31): mỗi cặp `key=value` bọc trong một dấu nháy
/// đơn, các cặp ngăn nhau bằng một dấu cách. **Sai định dạng thì git bỏ qua toàn bộ
/// chuỗi trong im lặng** — đó là lý do phải có test đọc ngược từng giá trị từ tiến
/// trình con thay vì soát chuỗi này bằng mắt.
///
/// Từng khoá chống một lỗi cụ thể:
///
/// - `log.showSignature=false` — chữ ký GPG in thêm dòng vào đầu ra `git log`,
///   làm lệch bộ phân tích.
/// - `diff.noprefix=false` — người dùng đặt `diff.noprefix=true` sẽ làm mọi đầu ra
///   diff mất tiền tố `a/` `b/`, khiến bộ phân tích tên tệp ở Phase 3 và bộ áp bản vá
///   ở Phase 5 đọc sai đường dẫn.
/// - `format.coverLetter=false` — chặn `format-patch` sinh thêm tệp thư giới thiệu
///   ngoài dự kiến.
const PINNED_GIT_CONFIG: &str =
    "'log.showSignature=false' 'diff.noprefix=false' 'format.coverLetter=false'";

/// Ghim môi trường cho tiến trình git.
///
/// Mỗi dòng dưới đây chống lại một lỗi cụ thể. Đọc kỹ trước khi sửa.
fn apply_env_hardening(cmd: &mut Command) {
    // --- Ngôn ngữ ---------------------------------------------------------
    // Không ghim thì trên máy đặt ngôn ngữ khác tiếng Anh, định dạng ngày và
    // thông báo lỗi đổi kiểu, bộ phân tích vỡ trên máy người dùng nhưng chạy
    // tốt trên máy lập trình viên.
    cmd.env("LC_ALL", "C");
    cmd.env("LANG", "C");
    cmd.env("LC_MESSAGES", "C");

    // --- Chặn mọi đường hỏi thông tin đăng nhập ---------------------------
    // Đây là cái bẫy nghiêm trọng nhất khi bọc git bằng giao diện đồ hoạ:
    // git chờ nhập mật khẩu trên stdin, ứng dụng treo vĩnh viễn, không báo gì.
    cmd.env("GIT_TERMINAL_PROMPT", "0");
    cmd.env("GIT_ASKPASS", "");
    cmd.env("SSH_ASKPASS", "");
    cmd.env("GCM_INTERACTIVE", "never");

    // --- Loại bỏ ảnh hưởng từ cấu hình bên ngoài --------------------------
    // Xem tài liệu của `PINNED_GIT_CONFIG` ở trên để biết từng khoá chống lỗi gì.
    // Khoá thứ tư mà ROADMAP đòi — `diff.external` — KHÔNG nằm trong chuỗi này:
    // nó được ghim mạnh hơn bằng biến `GIT_EXTERNAL_DIFF=""` bên dưới, vì biến môi
    // trường thắng cấu hình. Không phải bỏ sót.
    cmd.env("GIT_CONFIG_PARAMETERS", PINNED_GIT_CONFIG);

    // TODO(Phase 4): ràng buộc `--cleanup=whitespace` của PLAT-02 không đặt được ở
    // đây — nó là tham số dòng lệnh của `git commit`, không phải biến môi trường.
    // Phase 1 chưa có lệnh commit nào. Chi tiết và cách kiểm chứng:
    // `docs/02-phase4-commit-notes.md`.

    // Người dùng có thể đã đặt sẵn các biến này trong shell; thừa hưởng chúng
    // sẽ khiến commit do ứng dụng tạo ra mang danh tính sai.
    cmd.env_remove("GIT_AUTHOR_NAME");
    cmd.env_remove("GIT_AUTHOR_EMAIL");
    cmd.env_remove("GIT_AUTHOR_DATE");
    cmd.env_remove("GIT_COMMITTER_NAME");
    cmd.env_remove("GIT_COMMITTER_EMAIL");
    cmd.env_remove("GIT_COMMITTER_DATE");

    // Trình khác biệt bên ngoài sẽ thay thế đầu ra chuẩn của `git diff`.
    // Đây chính là cách ghim khoá `diff.external` mà ROADMAP đòi: đặt biến này rỗng
    // vô hiệu hoá `diff.external` của người dùng, và biến môi trường thắng cấu hình
    // nên cách này mạnh hơn việc thêm một cặp vào `PINNED_GIT_CONFIG`.
    cmd.env("GIT_EXTERNAL_DIFF", "");

    // Trình phân trang chặn tiến trình chờ người dùng bấm phím.
    cmd.env("GIT_PAGER", "cat");
    cmd.env("PAGER", "cat");

    // Không để git đọc cấu hình toàn hệ thống của máy build trong lúc chạy test.
    // (Chỉ bật khi chạy kiểm thử; bản chạy thật cần cấu hình người dùng.)
    #[cfg(test)]
    {
        cmd.env("GIT_CONFIG_NOSYSTEM", "1");
        cmd.env("HOME", "/nonexistent-git-plum-test");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Lớp bọc phải chạy được lệnh cơ bản nhất.
    #[tokio::test]
    async fn runs_git_version() {
        let out = GitCommand::new(".").arg("--version").run().await.unwrap();
        assert!(out.is_success());
        assert!(out.stdout.starts_with(b"git version"));
    }

    /// Thư mục không phải repository phải trả mã khác 0 kèm stderr đọc được,
    /// không được treo và không được panic (PLAT-10).
    #[tokio::test]
    async fn non_repo_fails_cleanly() {
        let dir = tempfile::tempdir().unwrap();
        let out = GitCommand::new(dir.path())
            .args(["rev-parse", "--git-dir"])
            .run()
            .await
            .unwrap();
        assert!(!out.is_success());
        assert!(!out.stderr_lossy().is_empty());
    }

    /// Hạn giờ phải bắn ra lỗi Timeout chứ không treo mãi.
    #[tokio::test]
    async fn timeout_fires() {
        // `git help --all` không treo, nên dùng hạn giờ cực ngắn để ép quá hạn.
        let res = GitCommand::new(".")
            .args(["help", "--all"])
            .timeout(Duration::from_nanos(1))
            .run()
            .await;
        assert!(matches!(res, Err(GitError::Timeout { .. })));
    }

    /// Đọc ngược một khoá cấu hình từ chính tiến trình git con.
    ///
    /// Đây là điểm mấu chốt của nhóm test bên dưới: sai cú pháp nháy đơn thì git bỏ
    /// qua toàn bộ `GIT_CONFIG_PARAMETERS` trong im lặng, và một test chỉ đọc mã bằng
    /// mắt sẽ không thấy gì. `git config --get` chạy được cả ngoài repository khi giá
    /// trị đến từ `GIT_CONFIG_PARAMETERS`, nên không cần `git init`.
    async fn read_back(key: &str) -> (String, i32) {
        let dir = tempfile::tempdir().unwrap();
        let out = GitCommand::new(dir.path())
            .args(["config", "--get", key])
            .run()
            .await
            .unwrap();
        // So sánh sau khi `trim`: git kết thúc dòng bằng `\n`, trên Windows có thể `\r\n`.
        (
            String::from_utf8_lossy(&out.stdout).trim().to_string(),
            out.status,
        )
    }

    /// `diff.noprefix=true` của người dùng làm mọi đầu ra diff mất tiền tố `a/` `b/`.
    #[tokio::test]
    async fn pins_diff_noprefix() {
        let (value, status) = read_back("diff.noprefix").await;
        assert_eq!(status, 0, "git không thấy khoá diff.noprefix");
        assert_eq!(value, "false");
    }

    /// `format.coverLetter` bật sẽ khiến `format-patch` sinh thêm tệp ngoài dự kiến.
    #[tokio::test]
    async fn pins_format_cover_letter() {
        let (value, status) = read_back("format.coverLetter").await;
        assert_eq!(status, 0, "git không thấy khoá format.coverLetter");
        assert_eq!(value, "false");
    }

    /// Mục đã ghim từ trước — test chống hồi quy.
    #[tokio::test]
    async fn pins_log_show_signature() {
        let (value, status) = read_back("log.showSignature").await;
        assert_eq!(status, 0, "git không thấy khoá log.showSignature");
        assert_eq!(value, "false");
    }

    /// Chứng minh git phân tích được toàn bộ chuỗi `GIT_CONFIG_PARAMETERS` chứ không
    /// bỏ qua vì sai cú pháp: cả ba khoá phải cùng xuất hiện trong một lần liệt kê.
    ///
    /// Lưu ý: `--get-regexp` in tên khoá đã hạ chữ thường (`log.showsignature`,
    /// `format.coverletter`) vì git chuẩn hoá phần tên khoá; so khớp phải theo dạng đó.
    #[tokio::test]
    async fn all_pinned_keys_reach_the_child_process() {
        let dir = tempfile::tempdir().unwrap();
        let out = GitCommand::new(dir.path())
            .args(["config", "--get-regexp", r"^(diff|format|log)\."])
            .run()
            .await
            .unwrap();
        assert!(out.is_success(), "git config --get-regexp thất bại");
        let listing = String::from_utf8_lossy(&out.stdout).to_lowercase();
        for key in ["log.showsignature", "diff.noprefix", "format.coverletter"] {
            assert!(
                listing.contains(key),
                "thiếu {key} trong đầu ra:\n{listing}"
            );
        }
    }
}
