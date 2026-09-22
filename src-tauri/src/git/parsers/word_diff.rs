//! Bộ phân tích `git diff --word-diff=porcelain` — diff mức **từ** (03-03).
//!
//! # Chính tả của cờ: `--word-diff=porcelain`, KHÔNG phải `--word-diff-porcelain`
//!
//! Dạng gạch ngang **không tồn tại** — git thoát 129 và in toàn bộ usage. Đo thật
//! trên git 2.54.0.windows.1:
//!
//! ```text
//! $ git diff --word-diff-porcelain <a> <b> -- <path>
//! usage: git diff [<options>] [<commit>] [--] [<path>...]      ← thoát 129
//!
//! $ git diff --word-diff=porcelain <a> <b> -- <path>
//! diff --git a/a.js b/a.js                                     ← chạy đúng
//! ```
//!
//! Ghi lại vì tài liệu kế hoạch của phase này viết sai tên ở ba chỗ. Cùng lớp lỗi với
//! `%x1f` của 02-04 nhưng **ồn hơn nhiều**: lỗi này thoát khác 0 và in usage, còn
//! `%x1f` thoát 0 với stdout trông hợp lý.
//!
//! # Định dạng: mỗi "từ" một dòng, `~` kết thúc một dòng NGUỒN
//!
//! Đây là một định dạng **khác hẳn** unified diff — không tái dùng
//! [`super::patch::parse_patch`] được. Đo thật trên đúng ca chủ dự án đưa
//! (`if (typeof cellData == "object")` → `===`):
//!
//! ```text
//! @@ -2 +2 @@ function f(cellData) {
//! ␠␠if (typeof cellData ␠      ← đoạn CHUNG, kết thúc bằng một dấu cách
//! -==                          ← chỉ phía CŨ
//! +===                         ← chỉ phía MỚI
//! ␠␠"object") {                ← đoạn CHUNG
//! ~                            ← HẾT một dòng nguồn
//! ```
//!
//! Tiền tố là đúng **một** ký tự: `␠` (chung), `-` (cũ), `+` (mới). **Cấu trúc dòng
//! nguồn bị tháo ra** — một dòng nguồn thành nhiều dòng đầu ra, nên muốn có khoảng
//! byte *trong* dòng thì phải **dựng lại** dòng nguồn bằng cách nối các đoạn. Đó là
//! phần việc chính của module này.
//!
//! # 🔴 Quy tắc đẩy ở `~`: cờ `touched`, và bộ đếm chỉ tăng cho phía được đẩy
//!
//! Hai quyết định này là **MỘT CẶP** — đúng một nửa vẫn sai. Bảng dưới đây là phép đo
//! thật (git 2.54.0.windows.1) trên ba fixture, chép vào đây để một mutation không đỏ
//! trong tương lai được chẩn đoán là **fixture sai** chứ không phải **mã đúng**:
//!
//! | quy tắc đẩy | bộ đếm | `empty-line` | `one-side` | `mixed` |
//! |---|---|---|---|---|
//! | `len() > 0` | chỉ tăng phía được đẩy | SAI nội dung | **ĐÚNG** | SAI nội dung |
//! | `len() > 0` | tăng ở mọi `~` | SAI nội dung | sai `line_no` | SAI nội dung |
//! | luôn đẩy cả hai | chỉ tăng phía được đẩy | ĐÚNG | SAI nội dung | SAI nội dung |
//! | luôn đẩy cả hai | tăng ở mọi `~` | ĐÚNG | SAI nội dung | SAI nội dung |
//! | **cờ `touched`** | **chỉ tăng phía được đẩy** | **ĐÚNG** | **ĐÚNG** | **ĐÚNG** |
//! | cờ `touched` | tăng ở mọi `~` | ĐÚNG | sai `line_no` | sai `line_no` |
//!
//! Ba điều đọc được, mỗi điều là một ràng buộc lên mã **và** lên bộ test:
//!
//! 1. **Chỉ một hàng đúng cả ba cột.** Hàng cuối cho thấy cờ `touched` với bộ đếm vô
//!    điều kiện vẫn sai `line_no` ở **hai** fixture.
//! 2. **`one-side` một mình vô dụng cho mutation `touched` → `len()`** — cột giữa
//!    hàng đầu là `ĐÚNG`. Mutation đó **phải** chạy trên `mixed`. Nếu nó cho 0 test
//!    đỏ thì kiểm test đang chạy trên tệp nào **trước khi** kết luận mã đúng.
//! 3. **`empty-line` một mình vô dụng cho mutation "luôn đẩy cả hai"** — cột đầu hàng
//!    ba là `ĐÚNG`. Mutation đó phải chạy trên `one-side` hoặc `mixed`.
//!
//! Câu hỏi đúng mà cờ trả lời là "**phía này có dòng ở vị trí này không**", không phải
//! "bộ đệm có byte không". Một dòng rỗng *có tồn tại* nhưng *không có byte*; một phía
//! bị xoá dòng *không tồn tại* ở vị trí đó. `len()` không phân biệt được hai thứ đó.
//!
//! # 🔴 Biên từ: [`WORD_DIFF_REGEX`], KHÔNG phải mặc định của git
//!
//! Plan 03-03 chỉ thị dùng biên từ **mặc định**. **Đã đo và điều đó SAI** — mặc định
//! làm việc dựng lại dòng nguồn trở nên **không thể**, và bất biến quan trọng nhất của
//! plan ("`content` dựng ra bằng từng byte với `parse_patch`") không đạt được.
//!
//! ## Lỗi: biên từ mặc định **mất** khoảng trắng ngăn cách
//!
//! Với biên mặc định, git coi một "từ" là một vệt ký tự không phải khoảng trắng, và
//! khoảng trắng ngăn cách **không thuộc đoạn nào** khi hai đoạn khác phía gặp nhau.
//! Hai phép đo trên git 2.54.0.windows.1, cả hai đều làm việc dựng lại sai:
//!
//! ```text
//! (1) `alpha beta` → `alpha`      — xoá từ CUỐI
//!      ␠alpha                        dựng lại phía CŨ: "alpha" + "beta"
//!      -beta                                          = "alphabeta"  ← MẤT dấu cách
//!      ~                             unified cho    : "alpha beta"
//!
//! (2) `dong hai` → `dong hai da sua` — thêm vào CUỐI
//!      ␠dong hai ␠                   dựng lại phía CŨ: "dong hai "
//!      +da sua                                        ← THỪA dấu cách ở cuối
//!      ~                             unified cho    : "dong hai"
//! ```
//!
//! Ca (2) là ca **thường gặp nhất có thể tưởng tượng** — thêm chữ vào cuối một dòng —
//! và nó nằm ngay trong fixture `text-simple.txt` của 03-02. Dấu cách ngăn cách bị
//! gắn vào đoạn chung khi phía mới còn chữ theo sau, và bị **bỏ hẳn** khi không phía
//! nào còn chữ. Không có quy tắc hậu xử lý nào lấy lại được: thông tin "dấu cách này
//! thuộc phía nào" **không có** trong đầu ra.
//!
//! ## Cách sửa: cho khoảng trắng thành một "từ" riêng
//!
//! [`WORD_DIFF_REGEX`] = `[^[:space:]]+|[[:space:]]+` — hoặc một vệt ký tự không
//! khoảng trắng, hoặc một vệt khoảng trắng. Mọi byte của dòng nguồn thuộc đúng một
//! "từ", nên nối các đoạn lại cho **đúng** dòng nguồn. Cùng hai ca trên:
//!
//! ```text
//! (1) ␠alpha / -␠beta / ~     → cũ = "alpha beta" ✅   mới = "alpha" ✅
//! (2) ␠dong hai / +␠da sua / ~ → cũ = "dong hai" ✅    mới = "dong hai da sua" ✅
//! ```
//!
//! ## Vì sao KHÔNG phải `--word-diff-regex=.`
//!
//! Plan cảnh báo đúng về `.`, và cảnh báo đó vẫn còn giá trị: `.` khớp theo **byte**
//! với đầu vào không phải ASCII, nên một ký tự tiếng Việt ba byte bị tách thành ba
//! "từ" và khoảng có thể cắt vào giữa ký tự. Dự án có repo mẫu `non-utf8` và HIST-11
//! là requirement **đã đạt**.
//!
//! [`WORD_DIFF_REGEX`] **không** có vấn đề đó: `[^[:space:]]+` khớp một **vệt** byte
//! không khoảng trắng, và mọi byte của một ký tự UTF-8 nhiều byte đều không phải
//! khoảng trắng, nên chúng luôn nằm trong cùng một vệt. Đo thật trên
//! `xéy dỏng test` → `xéy dỏng TEST` xác nhận `é` (2 byte) và `ỏ` (3 byte) đi qua
//! nguyên vẹn, và ca `==` → `===` của chủ dự án cho đầu ra **y hệt** biên mặc định.
//!
//! Test `khoang_khong_bao_gio_cat_giua_ky_tu_nhieu_byte` là cổng cho bất biến này, và
//! `noi_dung_dung_lai_bang_tung_byte_voi_parse_patch` (test tích hợp) là cổng cho phép
//! dựng lại. Cổng thứ hai chính là thứ đã bắt được lỗi biên mặc định.
//!
//! # `no_newline_at_eof` KHÔNG thuộc phạm vi module này
//!
//! Porcelain **không** cung cấp nó. Đo thật trên `keep\nlast == 1` (không newline
//! cuối) — porcelain in `~` bình thường và **không** in dòng báo hiệu, trong khi
//! `--unified=3` trên **cùng** tệp đó in `\ No newline at end of file` hai lần:
//!
//! ```text
//! === --word-diff=porcelain ===     === --unified=3 (CÙNG tệp) ===
//!  keep                              keep
//! ~                                 -last == 1
//!  last                             \ No newline at end of file
//! -==                               +last === 1
//! +===                              \ No newline at end of file
//!   1
//! ~            ← ~ CÓ MẶT, không dòng báo hiệu nào
//! ```
//!
//! Vì vậy [`WordLine`] **không có** trường đó, và người gọi **không được** ghi đè
//! `DiffLine::no_newline_at_eof` mà `parse_patch` đã đặt. Đừng đi tìm nó trong đầu ra
//! porcelain — nó không ở đó.

use crate::domain::diff::Span;

/// Biên từ truyền cho `git diff --word-diff-regex=<đây>`.
///
/// **Một vệt ký tự không khoảng trắng, HOẶC một vệt khoảng trắng.** Nghĩa là mọi byte
/// của dòng nguồn thuộc đúng một "từ" — điều kiện cần để nối các đoạn lại thành đúng
/// dòng nguồn.
///
/// Hằng số sống ở đây, cạnh bộ phân tích, chứ không ở `commands/diff.rs`: người gọi
/// lệnh git và người đọc đầu ra phải dùng **cùng một** biên từ, và hai chuỗi ở hai tệp
/// là đúng lớp lỗi mà Phase 2 phải ghim bằng test đọc chéo (`REF_COL_WIDTH`,
/// `MAX_VISIBLE_LANES`).
///
/// ⚠️ Đổi hằng này mà không đọc phần "Biên từ" ở tài liệu module là một bước lùi có
/// thật: biên mặc định của git **mất** khoảng trắng ngăn cách và làm việc dựng lại
/// dòng nguồn sai, còn `.` cắt giữa ký tự UTF-8 nhiều byte.
pub const WORD_DIFF_REGEX: &str = "[^[:space:]]+|[[:space:]]+";

/// Một dòng nguồn đã dựng lại từ porcelain, cộng các khoảng chữ thay đổi trong nó.
///
/// Cố ý **không có** `no_newline_at_eof`: porcelain không cung cấp trường đó (xem
/// tài liệu module). Thêm nó vào đây là mời người sau suy ra một giá trị sai từ một
/// nguồn không nói gì về nó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WordLine {
    /// Số dòng ở **phía của chính danh sách chứa nó**, đếm từ 1.
    ///
    /// `old_lines[i].line_no` là số dòng phía cũ; `new_lines[i].line_no` là số dòng
    /// phía mới. Người gọi khớp `WordLine` với `DiffLine` theo **đúng con số này**,
    /// không theo nội dung — hai dòng giống hệt nhau trong một hunk là chuyện thường.
    pub line_no: u32,
    /// Dòng nguồn đã dựng lại, **không** kèm tiền tố và **không** kèm `\r`.
    ///
    /// Bất biến quan trọng nhất của module: chuỗi này phải bằng **từng byte** với
    /// `DiffLine::content` mà `parse_patch` cho ra trên cùng commit/tệp/dòng. `spans`
    /// là chỉ số *vào chuỗi này*, nên lệch một byte làm mọi khoảng lệch theo.
    pub content: String,
    /// Khoảng byte của phần chữ **thay đổi** trong [`Self::content`]. Đã gộp những
    /// khoảng liền kề và sắp tăng dần.
    pub spans: Vec<Span>,
}

/// Kết quả phân tích một đầu ra `--word-diff=porcelain`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WordDiffParse {
    /// Các dòng nguồn phía **cũ**, theo thứ tự xuất hiện.
    pub old_lines: Vec<WordLine>,
    /// Các dòng nguồn phía **mới**, theo thứ tự xuất hiện.
    pub new_lines: Vec<WordLine>,
    /// Số dòng đầu vào không khớp khuôn nào.
    ///
    /// Bình thường là **0**. Khác 0 nghĩa là đầu vào không phải porcelain (ai đó
    /// truyền nhầm stdout của diff thường), và người gọi phải bỏ **toàn bộ** spans
    /// chứ không tô theo một phép đọc sai (T-03-23).
    ///
    /// Dòng header (`diff --git`, `index`, `---`, `+++`) và dòng
    /// `\ No newline at end of file` **không** tính vào đây — chúng vô hại, và đếm
    /// chúng sẽ tắt word-level cho mọi tệp.
    pub skipped: usize,
}

/// Trạng thái dựng lại **một** dòng nguồn, cho một phía.
///
/// Gộp ba thứ luôn phải được xử lý cùng nhau (bộ đệm, danh sách khoảng, cờ `touched`)
/// vào một struct: ba biến rời nhau là ba cơ hội quên đặt lại một cái ở `~`, và quên
/// đặt lại cờ là đúng lỗi làm mọi dòng sau sinh bản ghi giả.
#[derive(Default)]
struct BenDang {
    buf: String,
    spans: Vec<Span>,
    /// `true` khi phía này **có một dòng** ở vị trí đang dựng.
    ///
    /// 🔴 Đây là cờ, **không** phải `buf.len() > 0`. Một dòng nguồn rỗng cho đúng một
    /// đoạn `␠` với nội dung rỗng: `buf` rỗng nhưng dòng **tồn tại**. Xem bảng đo ở
    /// tài liệu module.
    touched: bool,
    /// Số dòng sẽ gán cho dòng đang dựng.
    line_no: u32,
}

impl BenDang {
    /// Nối một đoạn từ vào bộ đệm. `co_span` đánh dấu đoạn này là chữ **thay đổi**.
    fn them(&mut self, text: &str, co_span: bool) {
        let dau = self.buf.len();
        self.buf.push_str(text);
        self.touched = true;
        if co_span {
            self.gop_hoac_them(Span {
                start: dau,
                end: self.buf.len(),
            });
        }
    }

    /// Thêm một khoảng, **gộp** nếu nó chạm khoảng liền trước.
    ///
    /// Git tách được một vùng thay đổi thành nhiều đoạn liên tiếp. Không gộp thì
    /// 03-04 vẽ hai decoration chạm nhau và một đường viền hiện ra giữa vùng tô.
    fn gop_hoac_them(&mut self, moi: Span) {
        match self.spans.last_mut() {
            Some(cuoi) if cuoi.end == moi.start => cuoi.end = moi.end,
            _ => self.spans.push(moi),
        }
    }

    /// Kết thúc dòng đang dựng. Trả `Some` **chỉ khi** phía này có dòng ở đây, và khi
    /// đó **tăng** bộ đếm của riêng phía này.
    ///
    /// 🔴 Bộ đếm tăng **trong** nhánh `touched`, không ở ngoài. Phía không `touched`
    /// không có dòng ở vị trí này nên số dòng của nó **không tiến**. Với
    /// `-DELETED_LINE`: `old.line_no` tăng, `new.line_no` **giữ nguyên**.
    fn ket_thuc_dong(&mut self) -> Option<WordLine> {
        let ra = if self.touched {
            let w = WordLine {
                line_no: self.line_no,
                content: std::mem::take(&mut self.buf),
                spans: std::mem::take(&mut self.spans),
            };
            self.line_no += 1;
            Some(w)
        } else {
            None
        };
        // Đặt lại cờ. Bộ đệm và danh sách khoảng đã được `mem::take` ở nhánh trên
        // **lấy rỗng**, nên không cần `clear()` thêm ở đây.
        //
        // # Vì sao KHÔNG có `self.buf.clear()` phòng xa ở đây
        //
        // Bản đầu có hai lời gọi `clear()` sau khối `if`, đặt vào với lý do "đặt lại
        // cả ba bất kể có đẩy hay không". Chạy mutation xoá chúng cho **0 test đỏ**,
        // và phân tích cho thấy vì sao: [`BenDang::them`] là **nơi duy nhất** ghi vào
        // `buf`, và nó **luôn** đặt `touched = true`. Nên "bộ đệm không rỗng" kéo theo
        // "`touched` đúng", tức nhánh `else` ở trên luôn có bộ đệm đã rỗng sẵn. Hai
        // lời gọi đó là mã **không bao giờ chạy**.
        //
        // Giữ lại thì có hại chứ không vô hại: chúng làm người đọc tin rằng có một
        // đường đi cần được dọn, và làm một mutation tương lai xoá chúng trông như
        // "cổng hỏng" trong khi thật ra là "mã chết". Bất biến thật — dòng nguồn
        // không dính vào nhau — do `mem::take` giữ, và mutation đổi `mem::take` thành
        // `clone` cho 13 test đỏ.
        self.touched = false;
        ra
    }
}

/// Phân tích đầu ra `git diff --word-diff=porcelain` của **một** tệp.
///
/// Một lượt qua đầu vào, tích luỹ **hai** bộ đệm song song. Porcelain mô tả cả hai
/// phía cùng lúc (`␠` chung, `-` cũ, `+` mới) trong khi `DiffLine` là **một** phía,
/// nên hai bộ đệm là cách duy nhất tách chúng ra mà chỉ đọc đầu vào một lần.
pub fn parse_word_diff(stdout: &[u8]) -> WordDiffParse {
    let mut ra = WordDiffParse::default();
    let mut cu = BenDang::default();
    let mut moi = BenDang::default();
    let mut trong_hunk = false;

    for dong in tach_dong(stdout) {
        // --- Đầu hunk: khởi tạo hai bộ đếm ---------------------------------
        //
        // Số dòng là cách DUY NHẤT khớp `WordLine` với `DiffLine` ở tầng trên —
        // không khớp bằng nội dung, vì hai dòng giống nhau trong một hunk là chuyện
        // thường (fixture `dup-lines.txt` tồn tại để ghim đúng điều đó).
        if dong.starts_with(b"@@") {
            match doc_dau_hunk(dong) {
                Some((bat_dau_cu, bat_dau_moi)) => {
                    cu.line_no = bat_dau_cu;
                    moi.line_no = bat_dau_moi;
                    trong_hunk = true;
                }
                None => ra.skipped += 1,
            }
            continue;
        }

        // Mọi thứ trước `@@` đầu tiên là header của tệp.
        //
        // ⚠️ Phải bỏ qua **trước** phép đọc tiền tố: `---` và `+++` bắt đầu bằng `-`
        // và `+`, đúng hai tiền tố của đoạn từ. Đọc chúng thành đoạn từ sẽ nhét
        // `-- a/f.txt` vào bộ đệm phía cũ.
        if !trong_hunk {
            continue;
        }

        // --- `\ No newline at end of file` ---------------------------------
        //
        // Porcelain **không** in dòng này (đã đo — xem tài liệu module). Nhưng nếu
        // một phiên bản git nào đó in nó thì bỏ qua như header, **không** tính vào
        // `skipped`: người gọi bỏ TOÀN BỘ spans khi `skipped > 0`, nên đếm một dòng
        // vô hại vào đó sẽ tắt word-level cho cả tệp.
        if dong.starts_with(b"\\") {
            continue;
        }

        // --- `~`: kết thúc một dòng nguồn -----------------------------------
        if dong == b"~" {
            if let Some(w) = cu.ket_thuc_dong() {
                ra.old_lines.push(w);
            }
            if let Some(w) = moi.ket_thuc_dong() {
                ra.new_lines.push(w);
            }
            continue;
        }

        // --- Đoạn từ --------------------------------------------------------
        //
        // Tiền tố là đúng MỘT ký tự, và ký tự đó có thể là dấu cách. Cắt đúng một
        // byte — **không** `trim_start()`, thứ sẽ ăn luôn khoảng trắng thụt lề của
        // nội dung thật và làm lệch mọi khoảng sau đó trên dòng.
        let Some((&tien_to, text)) = dong.split_first() else {
            // Dòng hoàn toàn rỗng trong một hunk porcelain. Không khớp khuôn nào,
            // nhưng cũng không mang thông tin — bỏ qua, không đếm.
            continue;
        };
        let text = String::from_utf8_lossy(text);

        match tien_to {
            // Đoạn CHUNG: nối vào **cả hai** bộ đệm, không sinh khoảng.
            //
            // Nối kể cả khi `text` rỗng — đó là cách duy nhất biết "cả hai phía đều
            // CÓ một dòng ở đây". Dòng nguồn rỗng cho đúng một đoạn `␠` rỗng.
            b' ' => {
                cu.them(&text, false);
                moi.them(&text, false);
            }
            b'-' => cu.them(&text, true),
            b'+' => moi.them(&text, true),
            // Không khớp khuôn nào: đếm, **không đoán**. Hỏng to thì phải thấy được.
            _ => ra.skipped += 1,
        }
    }

    // Đầu ra porcelain hợp lệ luôn kết thúc bằng `~`, nên tới đây hai bộ đệm phải
    // rỗng. Nếu không (đầu ra bị cắt giữa chừng), đẩy nốt phần dở — mất một dòng còn
    // tệ hơn một dòng chưa đủ, vì mất dòng làm lệch phép khớp theo số dòng.
    if let Some(w) = cu.ket_thuc_dong() {
        ra.old_lines.push(w);
    }
    if let Some(w) = moi.ket_thuc_dong() {
        ra.new_lines.push(w);
    }

    ra
}

/// Đọc `old_start` và `new_start` từ `@@ -<os>[,<oc>] +<ns>[,<nc>] @@[ <heading>]`.
///
/// Chỉ cần hai số **bắt đầu**: số đếm không dùng ở đây, vì module này đếm dòng bằng
/// cách đi qua từng `~` chứ không tin vào số đếm trong header.
fn doc_dau_hunk(dong: &[u8]) -> Option<(u32, u32)> {
    let sau_at = dong.strip_prefix(b"@@")?;
    let dong_cuoi = memchr::memmem::find(sau_at, b"@@")?;
    let giua = &sau_at[..dong_cuoi];

    let mut phan = giua.split(|b| *b == b' ').filter(|p| !p.is_empty());
    let cu = phan.next()?.strip_prefix(b"-")?;
    let moi = phan.next()?.strip_prefix(b"+")?;

    Some((doc_bat_dau(cu)?, doc_bat_dau(moi)?))
}

/// Đọc phần `<start>` của `<start>[,<count>]`.
fn doc_bat_dau(s: &[u8]) -> Option<u32> {
    let s = match memchr::memchr(b',', s) {
        Some(i) => &s[..i],
        None => s,
    };
    if s.is_empty() {
        return None;
    }
    let mut n: u32 = 0;
    for b in s {
        if !b.is_ascii_digit() {
            return None;
        }
        n = n.checked_mul(10)?.checked_add((b - b'0') as u32)?;
    }
    Some(n)
}

/// Tách theo `\n` rồi **cắt `\r` cuối mỗi dòng** — cùng khuôn với `parse_patch`.
///
/// # `\r` là một đoạn từ RIÊNG trong porcelain, và đó là điều phải đo mới biết
///
/// Trên tệp CRLF, git **không** dính `\r` vào từ cuối. Đo thật bằng `od -c` trên
/// `crlf.txt` (`beta` → `BETA DA SUA`):
///
/// ```text
///  a l p h a \r \n ~ \n - b e t a \n + B E T A   D A   S U A \n   \r \n ~ \n
///                                                              ^^^^^^
///                                            đoạn CHUNG chứa đúng một byte \r
/// ```
///
/// Cắt `\r` ở đây xử lý **cả hai** dạng: ` alpha\r` thành `alpha`, và đoạn ` \r`
/// thành một đoạn chung **rỗng** — nối vào cả hai bộ đệm mà không thêm byte nào, đúng
/// điều ta muốn. Không cắt thì `\r` lọt vào `content`, phép so
/// `word_line.content == diff_line.content` ở tầng trên trượt trên **mọi** dòng của
/// một repo CRLF, và word-level tắt im lặng cho cả tệp.
fn tach_dong(buf: &[u8]) -> impl Iterator<Item = &[u8]> {
    let mut vi_tri = 0usize;
    std::iter::from_fn(move || {
        if vi_tri >= buf.len() {
            return None;
        }
        let con_lai = &buf[vi_tri..];
        let (dong, buoc) = match memchr::memchr(b'\n', con_lai) {
            Some(i) => (&con_lai[..i], i + 1),
            None => (con_lai, con_lai.len()),
        };
        vi_tri += buoc;
        Some(match dong.last() {
            Some(b'\r') => &dong[..dong.len() - 1],
            _ => dong,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Header đầy đủ, để test không chạy trên một đầu vào đã bị đơn giản hoá tới mức
    /// không còn giống đầu ra git.
    const HEADER: &str =
        "diff --git a/f.txt b/f.txt\nindex 1234567..89abcde 100644\n--- a/f.txt\n+++ b/f.txt\n";

    fn p(s: &str) -> WordDiffParse {
        parse_word_diff(s.as_bytes())
    }

    /// `(line_no, content)` của một phía — dạng gọn để ghim trong assertion.
    fn cap(v: &[WordLine]) -> Vec<(u32, &str)> {
        v.iter().map(|w| (w.line_no, w.content.as_str())).collect()
    }

    /// Chuỗi con mà một span trỏ tới. Panic khi span vượt biên hoặc cắt giữa ký tự —
    /// đó là điều ta muốn thấy đỏ.
    fn chu(w: &WordLine, i: usize) -> &str {
        let s = w.spans[i];
        &w.content[s.start..s.end]
    }

    // -----------------------------------------------------------------------
    // Ca của chủ dự án — nguyên văn
    // -----------------------------------------------------------------------

    /// **Ca mà chủ dự án nêu tên tường minh.** Đầu vào là đầu ra porcelain THẬT
    /// (đo trên git 2.54.0.windows.1).
    ///
    /// Yêu cầu của chủ dự án: `if (typeof cellData == "object")` → `===`, và **chỉ**
    /// `==`/`===` được tô, phần còn lại của dòng để nguyên.
    ///
    /// Khẳng định then chốt là **span HẸP HƠN cả dòng**. Một cài đặt trả về một span
    /// phủ toàn dòng qua được mọi phép kiểm "có span" — và nó đúng là thứ chủ dự án
    /// nói là KHÔNG muốn.
    #[test]
    fn ca_chu_du_an_chi_to_phan_dau_bang_khong_to_ca_dong() {
        // Đầu vào chép **nguyên văn** từ `git diff --word-diff=porcelain --unified=0`
        // trên đúng ca chủ dự án đưa. Ba đoạn từ, và chú ý số dấu cách:
        //   `␠␠␠if (typeof cellData ␠`  = tiền tố + hai dấu thụt lề + ... + một dấu
        //                                 cách theo sau từ `cellData`
        //   `-==` / `+===`              = phần thay đổi
        //   `␠␠"object") {`             = tiền tố + dấu cách nối + phần còn lại
        //
        // Bản đầu của test này tự gõ lại đầu vào và **thiếu một dấu cách** ở đoạn
        // đầu — test đỏ trong khi mã đúng. Chép từ đầu ra đo được, không gõ lại.
        let ra = p(&format!(
            "{HEADER}@@ -2 +2 @@ function f(cellData) {{\n   if (typeof cellData \n-==\n+===\n  \"object\") {{\n~\n"
        ));

        assert_eq!(ra.skipped, 0, "đầu ra porcelain hợp lệ không được bỏ dòng nào");
        assert_eq!(ra.new_lines.len(), 1, "một dòng nguồn → một WordLine phía mới");
        assert_eq!(ra.old_lines.len(), 1, "và một WordLine phía cũ");

        let moi = &ra.new_lines[0];
        assert_eq!(
            moi.content, "  if (typeof cellData === \"object\") {",
            "dòng nguồn phía MỚI phải được dựng lại nguyên vẹn từ các đoạn từ"
        );
        assert_eq!(moi.spans.len(), 1, "đúng MỘT khoảng thay đổi, nhận: {:?}", moi.spans);
        assert_eq!(chu(moi, 0), "===", "khoảng phải trỏ đúng vào `===`");

        let cu = &ra.old_lines[0];
        assert_eq!(cu.content, "  if (typeof cellData == \"object\") {");
        assert_eq!(cu.spans.len(), 1);
        assert_eq!(chu(cu, 0), "==", "phía cũ trỏ vào `==`");

        // 🔴 Khẳng định then chốt của cả plan.
        for (ten, w) in [("mới", moi), ("cũ", cu)] {
            let s = w.spans[0];
            assert!(
                s.end - s.start < w.content.len(),
                "phía {ten}: khoảng phải HẸP HƠN cả dòng. Chủ dự án nêu rõ \"không chỉ \
                 tô cả dòng\" — một cài đặt trả [0, len) qua được mọi phép kiểm \"có \
                 span\" nhưng làm sai đúng điều được yêu cầu. \
                 Khoảng [{}, {}) trên dòng dài {} byte",
                s.start,
                s.end,
                w.content.len()
            );
        }
    }

    // -----------------------------------------------------------------------
    // Quy tắc đẩy ở `~` — bảng đo ba-quy-tắc × ba-fixture
    // -----------------------------------------------------------------------

    /// 🔴 **Ca `mixed` — fixture DUY NHẤT phân biệt được cả ba quy tắc đẩy.**
    ///
    /// Đầu vào là đầu ra porcelain THẬT của `mixed.txt` (đã đo lại bằng `cat -A`,
    /// khớp từng byte với điều 03-02 dán vào script fixture).
    ///
    /// **Phải ghim `line_no`, không chỉ nội dung.** Hai cấu hình sai chỉ lệch ở
    /// `line_no` chứ không lệch nội dung (xem bảng ở tài liệu module), nên một test
    /// chỉ so `content` sẽ xanh trên cả hai. Và Task 2 khớp `WordLine` với `DiffLine`
    /// **theo số dòng**, nên `line_no` sai nghĩa là span rơi vào dòng khác.
    #[test]
    fn mixed_ghim_ca_line_no_lan_noi_dung_o_ca_hai_phia() {
        let ra = p(&format!(
            "{HEADER}@@ -1,4 +1,4 @@\n a\n~\n \n~\n-DELETED\n~\n keep\n~\n+ADDED\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "a"), (2, ""), (3, "keep"), (4, "ADDED")],
            "phía MỚI: bốn dòng, dòng 2 rỗng, và `ADDED` ở dòng 4.\n\
             Sai `line_no` ở đây nghĩa là bộ đếm đã tăng cho một phía KHÔNG có dòng — \
             xem bảng ba-quy-tắc trong tài liệu module"
        );
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "a"), (2, ""), (3, "DELETED"), (4, "keep")],
            "phía CŨ: `DELETED` ở dòng 3, `keep` ở dòng 4"
        );

        assert!(
            ra.new_lines.iter().all(|w| w.spans.is_empty())
                || ra.new_lines[3].spans.len() == 1,
            "dòng chỉ-thêm được phép có span phủ cả dòng; dòng ngữ cảnh thì không"
        );
        for w in ra.new_lines.iter().take(3) {
            assert!(
                w.spans.is_empty(),
                "dòng ngữ cảnh (chung cả hai phía) KHÔNG có khoảng thay đổi. \
                 Dòng {}: {:?}",
                w.line_no,
                w.spans
            );
        }
    }

    /// 🔴 **Dòng chỉ có ở MỘT phía** — loại bỏ quy tắc "luôn đẩy cả hai phía".
    ///
    /// Đầu ra porcelain thật của `one-side.txt`. Tại `~` đầu, phía mới **không có
    /// dòng nào** ở vị trí đó.
    ///
    /// Khẳng định **SỐ LƯỢNG** `WordLine` của từng phía, không chỉ nội dung: một cài
    /// đặt "luôn đẩy cả hai" cho đúng nội dung ở mọi dòng khác và chỉ sai ở số lượng
    /// của phía trống (nó sinh một bản ghi rỗng GIẢ).
    ///
    /// ⚠️ Ca này **không** phân biệt được `len() > 0` với cờ `touched` — đã đo, hai
    /// quy tắc cho y hệt nhau ở đây. Mutation đó phải chạy trên `mixed`.
    #[test]
    fn one_side_khong_sinh_ban_ghi_rong_gia_o_phia_khong_co_dong() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n-DELETED_LINE\n~\n keep2\n~\n+ADDED_LINE\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            ra.new_lines.len(),
            2,
            "phía MỚI có ĐÚNG hai dòng. Ba nghĩa là một bản ghi rỗng giả đã được đẩy \
             ở chỗ phía mới không có dòng nào — đó là quy tắc \"luôn đẩy cả hai phía\" \
             và nó sai. Nhận: {:?}",
            cap(&ra.new_lines)
        );
        assert_eq!(
            ra.old_lines.len(),
            2,
            "phía CŨ cũng đúng hai dòng. Nhận: {:?}",
            cap(&ra.old_lines)
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "keep2"), (2, "ADDED_LINE")]);
        assert_eq!(cap(&ra.old_lines), vec![(1, "DELETED_LINE"), (2, "keep2")]);
    }

    /// Dòng rỗng trong nguồn: porcelain in một đoạn `␠` với nội dung rỗng.
    ///
    /// Đầu ra thật của `empty-line.txt`. Quy tắc `len() > 0` bỏ hẳn dòng 2 ở đây —
    /// và mất một dòng làm **mọi** `line_no` sau nó lệch một nấc.
    #[test]
    fn dong_rong_van_la_mot_dong_va_khong_lam_lech_dong_sau() {
        let ra = p(&format!(
            "{HEADER}@@ -1,5 +1,5 @@\n header\n~\n \n~\n func a() {{\n~\n   x \n-==\n+===\n  1\n~\n }}\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![
                (1, "header"),
                (2, ""),
                (3, "func a() {"),
                (4, "  x === 1"),
                (5, "}")
            ],
            "dòng rỗng ở vị trí 2 phải CÓ MẶT với nội dung rỗng. Bỏ nó đi (quy tắc \
             `len() > 0`) làm mọi dòng sau lệch một nấc — và Task 2 khớp span theo \
             số dòng"
        );
        assert_eq!(
            cap(&ra.old_lines),
            vec![
                (1, "header"),
                (2, ""),
                (3, "func a() {"),
                (4, "  x == 1"),
                (5, "}")
            ]
        );

        // Span nằm đúng dòng 4, và hẹp hơn cả dòng.
        let d4 = &ra.new_lines[3];
        assert_eq!(d4.spans.len(), 1, "dòng 4 có đúng một khoảng");
        assert_eq!(chu(d4, 0), "===");
        assert!(d4.spans[0].end - d4.spans[0].start < d4.content.len());
    }

    /// **Bộ đếm đi qua NHIỀU dòng rỗng liên tiếp.**
    ///
    /// Ca này và ca `mixed` ghim hai thứ khác nhau và không thay được nhau: ở đây bộ
    /// đếm đi qua nhiều dòng rỗng liền nhau, còn ở `mixed` bộ đếm gặp tình huống
    /// **một phía không có dòng**.
    #[test]
    fn hai_dong_rong_lien_tiep_ghim_line_no() {
        let ra = p(&format!(
            "{HEADER}@@ -1,7 +1,7 @@\n header\n~\n \n~\n mid\n~\n \n~\n func a() {{\n~\n   x \n-==\n+===\n  1\n~\n }}\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![
                (1, "header"),
                (2, ""),
                (3, "mid"),
                (4, ""),
                (5, "func a() {"),
                (6, "  x === 1"),
                (7, "}")
            ],
            "bộ đếm phải tiến qua TỪNG dòng rỗng. Mỗi dòng rỗng bị bỏ làm mọi dòng \
             sau lệch thêm một nấc, nên ca hai dòng rỗng lệch HAI nấc"
        );
    }

    // -----------------------------------------------------------------------
    // Hình dạng span
    // -----------------------------------------------------------------------

    /// **Nhiều khoảng trong một dòng**, hai chỗ cách xa nhau → đúng **hai** span,
    /// không gộp thành một span phủ từ chỗ đầu tới chỗ cuối.
    #[test]
    fn hai_cho_sua_cach_xa_nhau_cho_hai_span_rieng() {
        let ra = p(&format!(
            "{HEADER}@@ -1 +1 @@\n let \n-a\n+A\n  = b + \n-c\n+C\n  + d\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[0];
        assert_eq!(moi.content, "let A = b + C + d");
        assert_eq!(
            moi.spans.len(),
            2,
            "hai chỗ sửa CÁCH XA nhau phải cho HAI khoảng riêng. Một khoảng nghĩa là \
             cài đặt đã gộp từ chỗ đầu tới chỗ cuối và tô luôn `= b +` ở giữa. \
             Nhận: {:?} trên {:?}",
            moi.spans,
            moi.content
        );
        assert_eq!(chu(moi, 0), "A");
        assert_eq!(chu(moi, 1), "C");
        assert!(
            moi.spans[0].end < moi.spans[1].start,
            "hai khoảng phải RỜI nhau, không chạm nhau"
        );

        let cu = &ra.old_lines[0];
        assert_eq!(cu.content, "let a = b + c + d");
        assert_eq!(cu.spans.len(), 2);
        assert_eq!(chu(cu, 0), "a");
        assert_eq!(chu(cu, 1), "c");
    }

    /// **Hai đoạn cùng phía LIỀN KỀ phải gộp thành một khoảng.**
    ///
    /// ⚠️ **Đầu vào của test này là TỔNG HỢP, không phải đầu ra git thật** — và đó là
    /// một sự thật phải ghi rõ chứ không giấu đi.
    ///
    /// Plan 03-03 khẳng định "git có thể tách một vùng thay đổi thành hai đoạn `+`
    /// liên tiếp". **Đã đo và điều đó SAI**: git gộp sẵn mọi vệt từ cùng phía thành
    /// một đoạn, ở cả biên từ mặc định lẫn `--word-diff-regex`. Ba phép đo trên
    /// git 2.54.0.windows.1:
    ///
    /// ```text
    /// `giu cu giu2` → `giu mot hai giu2`   (một từ thành HAI từ)
    ///   -cu / +mot hai            ← MỘT đoạn `+`, không phải hai
    ///
    /// `aaa bbb ccc` → `AAA BBB ccc`        (hai từ cạnh nhau cùng đổi)
    ///   -aaa bbb / +AAA BBB       ← MỘT đoạn mỗi phía
    ///
    /// cùng ca, --word-diff-regex=.         (mức byte, tách mạnh nhất)
    ///   -aaa / +AAA / ␠␠ / -bbb / +BBB     ← vẫn xen đoạn chung ở giữa
    /// ```
    ///
    /// Hệ quả: mutation "bỏ bước gộp khoảng liền kề" **không kiểm được bằng dữ liệu
    /// git thật** — không đầu vào thật nào làm nó đỏ. Bước gộp vẫn được giữ, và test
    /// này vẫn tồn tại, vì hai lý do:
    ///
    /// 1. Nó ghim hành vi cho một phiên bản git tương lai **có** tách đoạn. Định dạng
    ///    porcelain không hứa hẹn gì về việc gộp.
    /// 2. Nếu ai đó thêm một bước tiền xử lý tách đoạn (ví dụ để hỗ trợ
    ///    `--word-diff-regex` ở tầng của ta), bất biến "khoảng liền kề thì gộp" phải
    ///    còn đúng.
    ///
    /// Test chạy trên đầu vào tổng hợp là **đúng chỗ** ở đây vì thứ được kiểm là một
    /// hàm thuần trên một định dạng, không phải "git có trả về điều ta tưởng không".
    #[test]
    fn hai_doan_cung_phia_lien_ke_gop_thanh_mot_khoang() {
        // TỔNG HỢP: git thật in `+mot hai` thành một đoạn. Xem tài liệu của test.
        let ra = p(&format!(
            "{HEADER}@@ -1 +1 @@\n giu \n-cu\n+mot\n+ hai\n  giu2\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[0];
        assert_eq!(moi.content, "giu mot hai giu2");
        assert_eq!(
            moi.spans.len(),
            1,
            "hai đoạn `+` LIỀN KỀ phải gộp thành MỘT khoảng. Hai khoảng chạm nhau làm \
             03-04 vẽ hai decoration sát nhau và viền hiện giữa vùng tô. \
             Nhận: {:?} trên {:?}",
            moi.spans,
            moi.content
        );
        assert_eq!(
            chu(moi, 0),
            "mot hai",
            "khoảng gộp phải phủ cả hai đoạn liền kề"
        );
    }

    /// **Dòng chỉ THÊM** (không có đối ứng bên cũ): span phủ toàn dòng là **đúng** ở
    /// đây — ca này khác hẳn ca "dòng sửa", và trộn hai ca là cách dễ nhất để có một
    /// cài đặt sai mà test vẫn xanh.
    #[test]
    fn dong_chi_them_duoc_phep_co_span_phu_ca_dong() {
        let ra = p(&format!(
            "{HEADER}@@ -1,1 +1,2 @@\n giu\n~\n+dong moi hoan toan\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(cap(&ra.new_lines), vec![(1, "giu"), (2, "dong moi hoan toan")]);
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "giu")],
            "phía cũ KHÔNG có dòng thứ hai"
        );

        let them = &ra.new_lines[1];
        assert_eq!(them.spans.len(), 1);
        let s = them.spans[0];
        assert_eq!(
            (s.start, s.end),
            (0, them.content.len()),
            "với dòng chỉ-thêm, span phủ TOÀN dòng là đúng: cả dòng đều là chữ mới"
        );
    }

    /// **Dòng không đổi** giữa hai dòng đã sửa: `spans` rỗng, nhưng dòng **vẫn ra**.
    #[test]
    fn dong_khong_doi_co_spans_rong_nhung_van_ra() {
        let ra = p(&format!(
            "{HEADER}@@ -1,3 +1,3 @@\n a \n-x\n+X\n\n~\n khong doi\n~\n b \n-y\n+Y\n\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        assert_eq!(ra.new_lines.len(), 3, "cả ba dòng phải ra");
        assert_eq!(ra.new_lines[1].content, "khong doi");
        assert!(
            ra.new_lines[1].spans.is_empty(),
            "dòng không đổi KHÔNG có khoảng thay đổi. Nhận: {:?}",
            ra.new_lines[1].spans
        );
        assert!(!ra.new_lines[0].spans.is_empty(), "tiền đề: dòng 1 có sửa");
        assert!(!ra.new_lines[2].spans.is_empty(), "tiền đề: dòng 3 có sửa");
    }

    /// 🔴 **Khoảng trắng ngăn cách phải đi qua nguyên vẹn — hồi quy của lỗi đã đo.**
    ///
    /// Đây là ca đã buộc plan 03-03 đổi biên từ. Với biên **mặc định** của git, hai
    /// hình dạng dưới đây làm việc dựng lại dòng nguồn **sai**, và bất biến "bằng
    /// từng byte với `parse_patch`" không đạt được:
    ///
    /// ```text
    /// (1) `alpha beta` → `alpha`         (xoá từ CUỐI)
    ///     mặc định : ␠alpha / -beta / ~     → cũ = "alphabeta"   ← MẤT dấu cách
    ///     regex    : ␠alpha / -␠beta / ~    → cũ = "alpha beta"  ✅
    ///
    /// (2) `dong hai` → `dong hai da sua`  (thêm vào CUỐI)
    ///     mặc định : ␠dong hai ␠ / +da sua / ~ → cũ = "dong hai " ← THỪA dấu cách
    ///     regex    : ␠dong hai / +␠da sua / ~  → cũ = "dong hai"  ✅
    /// ```
    ///
    /// Ca (2) nằm ngay trong fixture `text-simple.txt` của 03-02 — tức là hình dạng
    /// thường gặp nhất có thể tưởng tượng, không phải ca biên hiếm.
    ///
    /// Đầu vào dưới đây là đầu ra THẬT dưới [`WORD_DIFF_REGEX`].
    #[test]
    fn khoang_trang_ngan_cach_khong_bi_mat_hay_thua() {
        // Ca (1): xoá từ cuối. Dấu cách thuộc phía CŨ.
        let ra = p(&format!("{HEADER}@@ -1 +1 @@\n alpha\n- beta\n~\n"));
        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "alpha beta")],
            "dấu cách ngăn cách phải CÒN ở phía cũ. `alphabeta` nghĩa là biên từ đang \
             là mặc định của git, và mọi phép so với `parse_patch` sẽ trượt"
        );
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "alpha")],
            "phía mới KHÔNG được mang dấu cách thừa"
        );
        assert_eq!(
            &ra.old_lines[0].content[ra.old_lines[0].spans[0].start
                ..ra.old_lines[0].spans[0].end],
            " beta",
            "khoảng phủ cả dấu cách bị xoá — đó đúng là phần biến mất khỏi dòng"
        );

        // Ca (2): thêm vào cuối. Dấu cách thuộc phía MỚI.
        let ra = p(&format!("{HEADER}@@ -1 +1 @@\n dong hai\n+ da sua\n~\n"));
        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "dong hai")],
            "phía cũ KHÔNG được có dấu cách treo ở cuối — `\"dong hai \"` là đúng lỗi \
             mà biên từ mặc định gây ra trên fixture `text-simple.txt`"
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "dong hai da sua")]);
        let m = &ra.new_lines[0];
        assert_eq!(
            &m.content[m.spans[0].start..m.spans[0].end],
            " da sua",
            "khoảng phủ cả dấu cách được thêm"
        );
    }

    // -----------------------------------------------------------------------
    // UTF-8 nhiều byte — T-03-18
    // -----------------------------------------------------------------------

    /// **Mọi khoảng phải nằm trên biên ký tự ở CẢ HAI đầu.**
    ///
    /// Đầu vào là đầu ra porcelain THẬT trên `xéy dỏng test` → `xéy dỏng TEST`
    /// (đo trên git 2.54; `é` là 2 byte, `ỏ` là 3 byte).
    ///
    /// Cổng này tồn tại để người sau đổi sang `--word-diff-regex=.` sẽ thấy nó ĐỎ,
    /// chứ không phát hiện qua báo lỗi người dùng: `.` trong regex của git khớp theo
    /// **byte**, nên một ký tự tiếng Việt ba byte bị tách thành ba "từ" và khoảng có
    /// thể cắt vào giữa ký tự.
    #[test]
    fn khoang_khong_bao_gio_cat_giua_ky_tu_nhieu_byte() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n xey dong test\n~\n xéy dỏng \n-test\n+TEST\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[1];
        assert_eq!(
            moi.content, "xéy dỏng TEST",
            "byte UTF-8 phải đi qua nguyên vẹn khi dựng lại dòng"
        );

        // Tiền đề: dòng PHẢI có ký tự nhiều byte, nếu không cổng này luôn xanh một
        // cách vô nghĩa (cổng tự vô hiệu hoá).
        assert!(
            moi.content.chars().count() < moi.content.len(),
            "tiền đề: dòng phải chứa ký tự nhiều byte, nếu không cổng biên ký tự \
             không kiểm được gì. {} ký tự / {} byte",
            moi.content.chars().count(),
            moi.content.len()
        );

        for w in ra.new_lines.iter().chain(ra.old_lines.iter()) {
            for s in &w.spans {
                assert!(
                    w.content.is_char_boundary(s.start),
                    "đầu khoảng {} cắt giữa một ký tự của {:?} — cắt chuỗi ở đó làm \
                     `&content[..]` panic ở Rust và hỏng UTF-8 ở JS (T-03-18)",
                    s.start,
                    w.content
                );
                assert!(
                    w.content.is_char_boundary(s.end),
                    "cuối khoảng {} cắt giữa một ký tự của {:?} (T-03-18)",
                    s.end,
                    w.content
                );
                assert!(
                    s.end <= w.content.len(),
                    "khoảng [{}, {}) vượt biên chuỗi dài {} byte (T-03-17)",
                    s.start,
                    s.end,
                    w.content.len()
                );
            }
        }

        assert_eq!(chu(moi, 0), "TEST", "khoảng phải trỏ đúng vào từ đã đổi");
        assert!(
            moi.spans[0].start > 0,
            "khoảng phải bắt đầu SAU phần `xéy dỏng ` không đổi — bắt đầu ở 0 nghĩa \
             là cài đặt đã tô cả dòng"
        );
    }

    // -----------------------------------------------------------------------
    // Ca biên của định dạng
    // -----------------------------------------------------------------------

    /// **Tiền tố là đúng MỘT ký tự, và ký tự đó có thể là dấu cách.**
    ///
    /// Cắt bằng `trim_start()` thay vì bỏ đúng một byte sẽ ăn mất khoảng trắng THỤT
    /// LỀ của nội dung thật, làm lệch mọi khoảng sau đó trên dòng.
    #[test]
    fn thut_le_dau_dong_khong_bi_an_mat() {
        let ra = p(&format!(
            "{HEADER}@@ -1 +1 @@\n        sau tam dau cach \n-cu\n+moi\n~\n"
        ));

        assert_eq!(ra.skipped, 0);
        let moi = &ra.new_lines[0];
        assert_eq!(
            moi.content, "       sau tam dau cach moi",
            "một byte tiền tố bị bỏ, BẢY dấu cách thụt lề còn lại phải nguyên vẹn. \
             `trim_start()` ăn hết cả tám và làm mọi khoảng sau đó lệch bảy byte"
        );
        assert!(
            moi.content.starts_with("       s"),
            "thụt lề phải còn. Nhận: {:?}",
            moi.content
        );
        assert_eq!(chu(moi, 0), "moi", "khoảng vẫn trỏ đúng sau khi giữ thụt lề");
    }

    /// **Bộ đệm phải được XOÁ ở mỗi `~`.** Không xoá thì mọi dòng nối lại thành một.
    #[test]
    fn bo_dem_duoc_xoa_o_moi_dau_ngã() {
        let ra = p(&format!("{HEADER}@@ -1,3 +1,3 @@\n mot\n~\n hai\n~\n ba\n~\n"));

        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "mot"), (2, "hai"), (3, "ba")],
            "ba dòng RIÊNG. Nội dung kiểu `mothai` hay `mothaiba` nghĩa là bộ đệm \
             không được xoá ở `~`"
        );
        for w in &ra.new_lines {
            assert!(
                !w.content.contains("mothai"),
                "dòng {:?} chứa hai dòng nguồn dính vào nhau",
                w.content
            );
        }
    }

    /// **CRLF**: `\r` không được lọt vào `content` — cùng bất biến với `parse_patch`.
    ///
    /// Đầu vào là đầu ra porcelain THẬT của `crlf.txt` (đo bằng `od -c`). Lưu ý hình
    /// dạng đặc biệt: git tách `\r` thành một đoạn từ **riêng** (` \r`) nằm SAU đoạn
    /// `+BETA DA SUA`, chứ không dính vào đoạn đó.
    #[test]
    fn crlf_khong_lot_vao_noi_dung() {
        let buf = b"diff --git a/c.txt b/c.txt\nindex b4ec4d1..bf24d7b 100644\n\
                    --- a/c.txt\n+++ b/c.txt\n@@ -1,3 +1,3 @@\n alpha\r\n~\n\
                    -beta\n+BETA DA SUA\n \r\n~\n gamma\r\n~\n";
        let ra = parse_word_diff(buf);

        assert_eq!(ra.skipped, 0);
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "alpha"), (2, "BETA DA SUA"), (3, "gamma")],
            "`\\r` là ký tự VÔ HÌNH: lọt vào `content` thì phép so `word_line.content \
             == diff_line.content` ở Task 2 trượt trên MỌI dòng của repo CRLF, và \
             word-level tắt im lặng cho cả tệp"
        );
        for w in ra.new_lines.iter().chain(ra.old_lines.iter()) {
            assert!(
                !w.content.contains('\r'),
                "không `content` nào được chứa `\\r`. Nhận: {:?}",
                w.content
            );
        }
    }

    /// 🔴 **Tệp không có dòng trống cuối: `~` VẪN ĐƯỢC IN, và porcelain KHÔNG in
    /// `\ No newline at end of file`.**
    ///
    /// Đầu vào là đầu ra porcelain THẬT của `no-eol.txt` (đo trên git 2.54). Đây là
    /// điểm mà porcelain khác `--unified`: cùng tệp đó, `--unified=3` in
    /// `\ No newline at end of file` **hai lần**, còn porcelain không in dòng nào.
    ///
    /// Hệ quả bắt buộc: `no_newline_at_eof` **không lấy được** từ porcelain, nên nó
    /// không thuộc phạm vi bộ phân tích này — trường đó do `parse_patch` cung cấp.
    #[test]
    fn khong_newline_cuoi_van_in_dau_nga_va_khong_co_dong_bao_hieu() {
        let ra = p(&format!(
            "{HEADER}@@ -1,3 +1,3 @@\n mot\n~\n hai\n~\n-ba\n+BA DA SUA\n~\n"
        ));

        assert_eq!(
            ra.skipped, 0,
            "porcelain của tệp không-newline-cuối là đầu vào HỢP LỆ; `skipped > 0` ở \
             đây làm Task 2 bỏ TOÀN BỘ spans của tệp"
        );
        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "mot"), (2, "hai"), (3, "BA DA SUA")],
            "dòng cuối vẫn phải ra đầy đủ dù tệp không kết thúc bằng newline"
        );
        assert_eq!(cap(&ra.old_lines), vec![(1, "mot"), (2, "hai"), (3, "ba")]);
    }

    /// Nếu một phiên bản git nào đó **có** in `\ No newline at end of file`, nó
    /// **không được** tính vào `skipped`.
    ///
    /// Lý do: bước 6 của Task 2 bỏ **toàn bộ** spans khi `skipped > 0`, nên đếm một
    /// dòng vô hại vào đó sẽ tắt word-level cho cả tệp. Bỏ qua nó như dòng header.
    #[test]
    fn dong_bao_khong_newline_khong_duoc_tinh_vao_skipped() {
        let ra = p(&format!(
            "{HEADER}@@ -1,2 +1,2 @@\n giu\n~\n-cu\n\\ No newline at end of file\n+moi\n\\ No newline at end of file\n~\n"
        ));

        assert_eq!(
            ra.skipped, 0,
            "`\\ No newline at end of file` là dòng VÔ HẠI — đếm nó vào `skipped` làm \
             Task 2 tắt word-level cho cả tệp vì một dòng không mang thông tin nào"
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "giu"), (2, "moi")]);
        assert_eq!(cap(&ra.old_lines), vec![(1, "giu"), (2, "cu")]);
    }

    /// Đầu ra **rỗng**: 0 dòng, không lỗi, không panic.
    #[test]
    fn dau_ra_rong_cho_khong_dong_khong_loi() {
        let ra = parse_word_diff(b"");
        assert_eq!(ra.old_lines.len(), 0);
        assert_eq!(ra.new_lines.len(), 0);
        assert_eq!(ra.skipped, 0);

        // Chỉ header, không hunk nào.
        let ra = p(HEADER);
        assert_eq!(ra.new_lines.len(), 0);
        assert_eq!(
            ra.skipped, 0,
            "dòng header không phải dòng bị bỏ — đếm chúng làm Task 2 tắt word-level \
             cho MỌI tệp"
        );
    }

    /// **Đầu vào không phải porcelain** (ai đó truyền stdout của diff thường vào):
    /// `skipped > 0` và không panic. Hỏng to thì phải thấy được.
    #[test]
    fn dau_vao_khong_phai_porcelain_thi_skipped_khac_khong() {
        // Diff thường: dòng `-cu` và `+moi` KHÔNG kèm `~` nào, và có dòng `?` lạ.
        let ra = p(&format!("{HEADER}@@ -1,2 +1,2 @@\n giu\n?rac khong hieu\n-cu\n+moi\n"));
        assert!(
            ra.skipped > 0,
            "một dòng không khớp khuôn nào phải được ĐẾM, không im lặng bỏ qua — \
             đó là cách duy nhất Task 2 biết mà tắt word-level thay vì tô sai chỗ \
             (T-03-23)"
        );
    }

    /// `@@` khởi tạo hai bộ đếm theo `old_start`/`new_start` của **chính hunk đó**.
    /// Số dòng là cách DUY NHẤT khớp `WordLine` với `DiffLine` ở Task 2.
    #[test]
    fn dau_hunk_khoi_tao_bo_dem_theo_start_cua_chinh_no() {
        let ra = p(&format!(
            "{HEADER}@@ -10,2 +20,2 @@ fn x()\n giu\n~\n-cu\n+moi\n~\n"
        ));

        assert_eq!(
            cap(&ra.old_lines),
            vec![(10, "giu"), (11, "cu")],
            "phía cũ bắt đầu từ `old_start` = 10"
        );
        assert_eq!(
            cap(&ra.new_lines),
            vec![(20, "giu"), (21, "moi")],
            "phía mới bắt đầu từ `new_start` = 20 — HAI bộ đếm độc lập, không phải một"
        );
    }

    /// Hai hunk trong một tệp: bộ đếm phải **đặt lại** theo `@@` của hunk thứ hai.
    #[test]
    fn hunk_thu_hai_dat_lai_bo_dem() {
        let ra = p(&format!(
            "{HEADER}@@ -1,1 +1,1 @@\n-a\n+A\n~\n@@ -50,1 +50,1 @@\n-b\n+B\n~\n"
        ));

        assert_eq!(
            cap(&ra.new_lines),
            vec![(1, "A"), (50, "B")],
            "hunk thứ hai KHÔNG được tiếp tục bộ đếm của hunk đầu — nó đặt lại theo \
             `@@` của chính nó"
        );
    }

    /// Dòng `diff --git` / `index` / `---` / `+++` bị bỏ qua và **không** tính vào
    /// `skipped`.
    ///
    /// ⚠️ `---` và `+++` là cái bẫy thật: chúng bắt đầu bằng `-` và `+`, đúng hai
    /// tiền tố của đoạn từ. Đọc chúng thành đoạn từ sẽ nhét `-- a/f.txt` vào bộ đệm
    /// phía cũ.
    #[test]
    fn header_khong_bi_doc_thanh_doan_tu() {
        let ra = p(&format!("{HEADER}@@ -1 +1 @@\n-cu\n+moi\n~\n"));

        assert_eq!(ra.skipped, 0, "header không phải dòng bị bỏ");
        assert_eq!(
            cap(&ra.old_lines),
            vec![(1, "cu")],
            "`--- a/f.txt` bắt đầu bằng `-` nhưng KHÔNG phải đoạn từ. Nội dung kiểu \
             `-- a/f.txtcu` nghĩa là header đã bị đọc thành đoạn từ"
        );
        assert_eq!(cap(&ra.new_lines), vec![(1, "moi")]);
        for w in ra.old_lines.iter().chain(ra.new_lines.iter()) {
            assert!(
                !w.content.contains("f.txt"),
                "đường dẫn từ header lọt vào nội dung dòng: {:?}",
                w.content
            );
        }
    }
}
