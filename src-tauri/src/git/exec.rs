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

        let args = them_no_ext_diff(self.args);

        cmd.current_dir(&self.repo_path);
        cmd.args(&args);

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
            args: args.clone(),
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
                args: args.clone(),
                seconds: self.timeout.as_secs(),
            })?
            .map_err(|e| GitError::SpawnFailed {
                args: args.clone(),
                reason: e.to_string(),
            })?;

        Ok(GitOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            status: output.status.code().unwrap_or(-1),
        })
    }
}

/// Các lệnh con của git có gọi tới trình khác biệt, tức cần `--no-ext-diff`.
///
/// `log` và `show` nằm trong danh sách vì cả hai **in được bản vá** (`git log -p`,
/// `git show` mặc định in diff của commit). Thiếu chúng thì lỗi quay lại y nguyên ở
/// một đường khác.
const LENH_CO_DIFF: [&str; 4] = ["diff", "show", "log", "diff-tree"];

/// Chèn `--no-ext-diff` vào lệnh git khi lệnh đó có thể gọi trình khác biệt ngoài.
///
/// # Vì sao chèn ở tầng này, không để người gọi tự thêm
///
/// Người gọi **sẽ quên**. Đây đúng là lỗi đã xảy ra: `GIT_EXTERNAL_DIFF=""` được đặt
/// với ý vô hiệu hoá trình diff ngoài, nhưng git lại đem chuỗi rỗng đi spawn và mọi
/// lệnh diff sinh bản vá thoát 128 với stdout **rỗng** — im lặng, không lỗi biên dịch,
/// và Phase 2 không phát hiện vì nó chỉ dùng `--name-status` (không gọi trình diff).
/// Đặt ở đây thì không lệnh nào lọt.
///
/// Cờ phải đứng **sau** lệnh con: `git -c a=b diff --no-ext-diff` đúng, còn
/// `git --no-ext-diff -c a=b diff` thì git không nhận. Nên hàm này bỏ qua các cặp
/// `-c key=value` dẫn đầu để tìm đúng vị trí lệnh con.
///
/// Không chèn hai lần nếu người gọi đã tự thêm.
fn them_no_ext_diff(args: Vec<String>) -> Vec<String> {
    if args.iter().any(|a| a == "--no-ext-diff") {
        return args;
    }

    // Bỏ qua các tuỳ chọn toàn cục dẫn đầu để tìm lệnh con. `-c` mang giá trị ở
    // tham số kế tiếp nên phải nhảy hai bước.
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => i += 2,
            a if a.starts_with('-') => i += 1,
            _ => break,
        }
    }

    let Some(lenh) = args.get(i) else {
        return args;
    };
    if !LENH_CO_DIFF.contains(&lenh.as_str()) {
        return args;
    }

    let mut ra = args;
    ra.insert(i + 1, "--no-ext-diff".to_owned());
    ra
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
    // Khoá thứ tư mà ROADMAP đòi — `diff.external` — KHÔNG nằm trong chuỗi này, và
    // cũng KHÔNG ghim được bằng `diff.external=` rỗng: git sẽ đi spawn chương trình
    // tên rỗng rồi chết (đã đo, xem `them_no_ext_diff`). Nó được ghim bằng cờ
    // `--no-ext-diff`, thứ thắng cả cấu hình lẫn biến môi trường. Không phải bỏ sót.
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
    //
    // # Vì sao `env_remove` chứ KHÔNG `env("GIT_EXTERNAL_DIFF", "")`
    //
    // Bản đầu đặt biến này thành chuỗi **rỗng**, tưởng rằng rỗng nghĩa là "không có
    // trình nào". Git không hiểu thế: nó nhận biến đã-được-đặt rồi đi **spawn đúng
    // chương trình tên rỗng**, và thất bại:
    //
    // ```text
    // error: cannot spawn : No such file or directory
    // fatal: external diff died, stopping at file.txt
    // ```
    //
    // Hậu quả: **mọi** lệnh diff có sinh nội dung bản vá thoát 128 với stdout rỗng.
    // Đã đo trên git 2.54.0.windows.1 với `--unified=3` và `--word-diff=porcelain`.
    // Phase 2 không gặp vì nó chỉ dùng `--name-status`, vốn không gọi tới trình diff
    // nên không spawn gì — lỗi ngủ yên tới lúc Phase 3 cần bản vá thật.
    //
    // Đặt `diff.external=` rỗng trong `PINNED_GIT_CONFIG` có **cùng lỗi** (đã đo).
    // Cách đúng là `--no-ext-diff`, cờ của chính git cho việc này: nó thắng **cả**
    // `diff.external` trong cấu hình **và** `GIT_EXTERNAL_DIFF` trong môi trường
    // (đã đo cả ba tổ hợp). Cờ đó được thêm ở `run()` cho mọi lệnh `diff`.
    //
    // Ở đây chỉ cần **xoá** biến của người dùng khỏi môi trường tiến trình con, để
    // một `GIT_EXTERNAL_DIFF` có sẵn trong shell không rò vào những lệnh không mang
    // cờ trên.
    cmd.env_remove("GIT_EXTERNAL_DIFF");

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
    // --- Trình khác biệt ngoài: hồi quy của một lỗi im lặng -----------------

    /// Dựng một repo có hai commit trên một tệp, qua đúng lớp `GitCommand` thật.
    ///
    /// Trả `(TempDir, đường dẫn)`. Giữ `TempDir` sống ở người gọi.
    async fn repo_co_mot_thay_doi(ten: &str, cu: &str, moi: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();

        for args in [
            vec!["init", "--initial-branch=main"],
            vec!["config", "user.email", "t@e.com"],
            vec!["config", "user.name", "T"],
        ] {
            GitCommand::new(p).args(args).run().await.unwrap();
        }

        std::fs::write(p.join(ten), cu).unwrap();
        GitCommand::new(p).args(["add", ten]).run().await.unwrap();
        GitCommand::new(p)
            .args(["commit", "-m", "one"])
            .run()
            .await
            .unwrap();

        std::fs::write(p.join(ten), moi).unwrap();
        GitCommand::new(p).args(["add", ten]).run().await.unwrap();
        GitCommand::new(p)
            .args(["commit", "-m", "two"])
            .run()
            .await
            .unwrap();

        dir
    }

    /// **Lỗi thật, tìm thấy ở plan 03-01.** `GIT_EXTERNAL_DIFF=""` làm git đi spawn
    /// chương trình tên rỗng, nên **mọi** lệnh diff sinh bản vá thoát 128 với stdout
    /// rỗng:
    ///
    /// ```text
    /// error: cannot spawn : No such file or directory
    /// fatal: external diff died, stopping at file.txt
    /// ```
    ///
    /// Đây là test lẽ ra phải tồn tại từ Phase 1. Phase 2 không gặp lỗi vì nó chỉ
    /// dùng `--name-status`, vốn **không** gọi tới trình diff — nên một test chỉ
    /// kiểm `--name-status` vẫn xanh trong khi cả đường bản vá đã vỡ.
    #[tokio::test]
    async fn diff_sinh_ban_va_thuc_su_co_noi_dung() {
        let dir = repo_co_mot_thay_doi("a.txt", "mot\nhai\n", "mot\nhai da sua\n").await;

        let out = GitCommand::new(dir.path())
            .args(["diff", "--unified=3", "HEAD~1", "HEAD", "--", "a.txt"])
            .run()
            .await
            .unwrap();

        assert!(
            out.is_success(),
            "git diff phải thành công, nhận exit {} stderr {:?}",
            out.status,
            out.stderr_lossy()
        );
        let patch = String::from_utf8_lossy(&out.stdout);
        assert!(
            patch.contains("@@"),
            "bản vá phải có đầu hunk, nhận được: {patch:?}"
        );
        assert!(patch.contains("+hai da sua"), "bản vá phải có dòng thêm");
    }

    /// `--word-diff=porcelain` là đường mà plan 03-03 dựa vào cho diff mức từ. Nó
    /// cũng gọi trình diff nên vỡ theo cùng lỗi trên; ghim riêng vì 03-03 sẽ hỏng vì
    /// một lý do rất khó truy nếu cờ này thoát 128 với stdout rỗng.
    ///
    /// Ghim thêm **chính tả**: `--word-diff-porcelain` (gạch ngang, không dấu bằng)
    /// **không tồn tại** — nó thoát khác 0 và in usage. Ba chỗ trong CONTEXT.md từng
    /// viết sai tên này.
    ///
    /// Ca đo là đúng ca chủ dự án đưa: `==` → `===`.
    #[tokio::test]
    async fn word_diff_porcelain_chay_duoc_va_dung_chinh_ta() {
        let dir = repo_co_mot_thay_doi(
            "b.js",
            "if (typeof cellData == \"object\") {\n",
            "if (typeof cellData === \"object\") {\n",
        )
        .await;

        let dung = GitCommand::new(dir.path())
            .args([
                "diff",
                "--word-diff=porcelain",
                "HEAD~1",
                "HEAD",
                "--",
                "b.js",
            ])
            .run()
            .await
            .unwrap();
        assert!(
            dung.is_success(),
            "--word-diff=porcelain phải chạy được, nhận exit {} stderr {:?}",
            dung.status,
            dung.stderr_lossy()
        );

        let ra = String::from_utf8_lossy(&dung.stdout);
        assert!(ra.contains("@@"), "phải có đầu hunk, nhận: {ra:?}");
        assert!(
            ra.lines().any(|l| l == "-=="),
            "mỗi từ chiếm một dòng riêng kèm tiền tố; phải có dòng `-==`. Nhận: {ra:?}"
        );
        assert!(
            ra.lines().any(|l| l == "+==="),
            "phải có dòng `+===`. Nhận: {ra:?}"
        );

        let sai = GitCommand::new(dir.path())
            .args(["diff", "--word-diff-porcelain", "HEAD~1", "HEAD"])
            .run()
            .await
            .unwrap();
        assert!(
            !sai.is_success(),
            "`--word-diff-porcelain` KHÔNG tồn tại. Nếu lệnh này thành công thì git đã \
             đổi hành vi và plan 03-03 phải đọc lại giả định của nó"
        );
    }

    /// Cờ phải đứng **sau** lệnh con, và không được chèn vào lệnh không liên quan.
    #[test]
    fn chen_no_ext_diff_dung_vi_tri() {
        assert_eq!(
            them_no_ext_diff(vec!["diff".into(), "--unified=3".into()]),
            vec!["diff", "--no-ext-diff", "--unified=3"]
        );

        // Sau cặp `-c key=value` dẫn đầu, không phải trước: git không nhận
        // `--no-ext-diff` đứng trước lệnh con.
        assert_eq!(
            them_no_ext_diff(vec![
                "-c".into(),
                "core.quotepath=false".into(),
                "diff".into(),
                "--name-status".into(),
            ]),
            vec![
                "-c",
                "core.quotepath=false",
                "diff",
                "--no-ext-diff",
                "--name-status"
            ]
        );

        // `show` và `log` cũng in được bản vá.
        assert!(them_no_ext_diff(vec!["show".into(), "HEAD".into()])
            .contains(&"--no-ext-diff".to_owned()));
        assert!(
            them_no_ext_diff(vec!["log".into(), "-p".into()]).contains(&"--no-ext-diff".to_owned())
        );

        // Lệnh không liên quan để nguyên — thêm cờ lạ vào `rev-parse` làm nó thất bại.
        assert_eq!(
            them_no_ext_diff(vec!["rev-parse".into(), "HEAD".into()]),
            vec!["rev-parse", "HEAD"]
        );
        assert_eq!(
            them_no_ext_diff(vec!["--version".into()]),
            vec!["--version"]
        );

        // Không chèn hai lần.
        assert_eq!(
            them_no_ext_diff(vec!["diff".into(), "--no-ext-diff".into()]),
            vec!["diff", "--no-ext-diff"]
        );
    }
}
