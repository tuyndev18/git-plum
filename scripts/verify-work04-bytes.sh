#!/usr/bin/env bash
# ---------------------------------------------------------------------------
# Cổng MÁY cho **tiêu chí thành công 2** của Phase 5 (WORK-04).
#
#   "Sau staging một phần: CRLF giữ nguyên, thiếu dòng cuối giữ nguyên, byte
#    không UTF-8 KHÔNG bị thay."
#
# `CONTEXT.md` mục 7 nói thẳng rằng tiêu chí này **máy kiểm được** bằng `od -c`.
# Đây là nó, thành script, nên nó chạy lại được thay vì là một lần nhìn bằng mắt
# rồi mất.
#
# ===========================================================================
# 🔴 VÌ SAO SCRIPT NÀY CHẠY HAI LƯỢT
# ===========================================================================
#
# `CONTEXT.md` mục 4.1 lỗi #4: *"fixture KHÔNG phân biệt được đột biến"*. Một cổng
# chưa bao giờ đỏ là một cổng chưa biết có chạy không.
#
# Phép kiểm `\r` ở lượt 1 đếm byte `\r` trước và sau khi stage rồi khẳng định
# chúng **bằng nhau**. Nhưng `0 == 0` cũng "bằng nhau" — nên một cài đặt đã xoá
# sạch mọi `\r` vẫn qua được phép kiểm đó. Chính xác lớp lỗi "hình dạng đúng, dữ
# liệu vô hại" của mục 4.2.
#
# Nên lượt 2 dựng **cùng một** kịch bản với tệp **LF thuần** và khẳng định số đo
# `\r` **khác** lượt 1. Nếu hai lượt cho cùng con số thì phép kiểm không đọc
# `\r` thật — nó đọc một hằng số. Script thoát 0 **chỉ khi** lượt 1 đạt **và**
# lượt 2 khác lượt 1.
#
# ===========================================================================
# 🔴 IN SỐ ĐO THÔ, BỌC `[...]`, RỒI MỚI QUYẾT ĐỊNH
# ===========================================================================
#
# `CONTEXT.md` mục 5: `git status --porcelain <paths> | wc -l` trả `0` trong khi
# cùng lệnh không qua ống trả ` M src/App.tsx`. Một con số đã nói dối **đúng lúc
# ra quyết định**. Với phép kiểm quyết định có chạy tiếp hay không, đếm là
# **không đủ** — nên mọi số đo ở đây được in dạng `[...]` để phân biệt "rỗng"
# với "khoảng trắng", và người đọc nhìn thấy số thật chứ không chỉ thấy ĐẠT.
#
# ===========================================================================
# 🔴 `core.autocrlf true` — TƯỜNG MINH, và đó là ca R2 chứ không phải cấu hình
# ===========================================================================
#
# `core.autocrlf=true` là **mặc định trên Windows** và là máy chủ dự án
# (`CONTEXT.md` R2). Đặt `false` ở đây sẽ né mất **chính ca** cần kiểm: với
# `autocrlf=true` git chuẩn hoá CRLF→LF lúc ghi vào index, nên một cài đặt
# staging theo khối cẩu thả sẽ ghi LF ngược lại ra thư mục làm việc.
#
# ===========================================================================
# 🔴 BYTE 0xE9 KHÔNG TẠO ĐƯỢC BẰNG SHELL
# ===========================================================================
#
# `printf 'caf\351'` bị chuyển sang UTF-8 (`caf\303\251`) **trước** khi tới hệ
# thống tệp — đã đo, ghi ở `scripts/fixtures/make-fixtures.sh` mục non-utf8 và
# gặp lại ở `make-hunk-fixtures.sh`. Dùng `node -e` với
# `Buffer.from(s, "binary")`, đúng khuôn đã chạy được ở hai script kia.
#
# ===========================================================================
# 🔴 REPO TẠM NẰM NGOÀI REPO DỰ ÁN
# ===========================================================================
#
# Trong thư mục tạm của hệ điều hành, **không bao giờ** trong repo dự án (một
# repo lồng repo làm `git status` của dự án mọc thêm mục) và **không bao giờ**
# trong `D:/MyCompanyProjects/quanly-truong-phong-so` — repo thật của chủ dự án,
# **chỉ-đọc**.
# ---------------------------------------------------------------------------
set -u

# Không `set -e`: script này **phải** chạy tới cuối để in cả hai lượt. Thoát sớm
# ở lượt 1 sẽ giấu mất chính phép đối chứng chứng minh cổng phân biệt được.

command -v node >/dev/null 2>&1 || {
  echo "🔴 không có \`node\` trên PATH — byte 0xE9 không tạo được bằng shell" >&2
  exit 2
}
command -v git >/dev/null 2>&1 || { echo "🔴 không có \`git\` trên PATH" >&2; exit 2; }
command -v od  >/dev/null 2>&1 || { echo "🔴 không có \`od\` trên PATH" >&2; exit 2; }

# ---------------------------------------------------------------------------
# 🔴 `GIT` — vì sao không gọi thẳng `git`
#
# Máy chủ dự án chạy một hook Claude Code viết lại `git …` thành `rtk git …`,
# và `rtk` **tóm tắt** đầu ra `git diff` thành một bản tóm lược cho người đọc:
#
#     m.txt | 6 +++---
#      1 file changed, 3 insertions(+), 3 deletions(-)
#     --- Changes ---
#     …
#
# Đã đo hôm nay: `grep -c '^@@'` trên đầu ra đó trả **1** trong khi bản vá thật
# có **3** khối. Một script dựng bản vá con từ đầu ra tóm tắt sẽ dựng ra rác, và
# nó làm thế **im lặng** — đúng lớp lỗi "một con số nói dối đúng lúc ra quyết
# định" của `CONTEXT.md` mục 5.
#
# `rtk proxy` chạy lệnh thô không qua bộ lọc. Không có `rtk` thì `git` trần là
# đúng (trên máy không có hook, không có gì viết lại).
# ---------------------------------------------------------------------------
if command -v rtk >/dev/null 2>&1; then
  GIT() { rtk proxy git "$@"; }
else
  GIT() { git "$@"; }
fi

# `node` là chương trình **Windows**: nó không hiểu đường dẫn POSIX của Git Bash.
# `node -e '…' /tmp/x` mở `C:\tmp\x` — đã đo hôm nay, và nó thất bại với `ENOENT`
# trỏ vào một đường dẫn không ai viết ra. Mọi đường dẫn đưa cho `node` đi qua đây.
if command -v cygpath >/dev/null 2>&1; then
  W() { cygpath -w "$1"; }
else
  W() { printf '%s' "$1"; }
fi

GOC_TAM="$(mktemp -d)"
trap 'rm -rf "$GOC_TAM"' EXIT

echo "git:  $(git --version)"
echo "node: $(node --version)"
echo "tạm:  $GOC_TAM"
echo

# ---------------------------------------------------------------------------
# `dem_cr <tệp>` — số byte `\r` trong tệp, đọc từ `od -c`.
#
# Qua `od -c` chứ không `grep -c $'\r'`: `grep` làm việc theo **dòng**, nên nó
# đếm số *dòng chứa* `\r`, không phải số *byte* `\r`. Trên một tệp mà mọi dòng
# đều CRLF hai con số trùng nhau — tức phép đếm sai vẫn cho kết quả đúng ở đúng
# ca ta đang kiểm, và chỉ sai ở ca hỗn hợp. Đúng lớp lỗi "dữ liệu vô hại".
#
# `od -c` in `\r` thành ký tự `\r` hai ký tự. Đếm số lần xuất hiện của nó.
# ---------------------------------------------------------------------------
dem_cr() {
  od -c "$1" | tr ' ' '\n' | grep -c '^\\r$' || true
}

# `co_0xE9 <tệp>` → `CO` / `KHONG`. `351` là 0xE9 ở bát phân, dạng `od -c` in ra.
co_0xE9() {
  if od -c "$1" | tr ' ' '\n' | grep -q '^351$'; then echo "CO"; else echo "KHONG"; fi
}

# `byte_cuoi_la_newline <tệp>` → `CO` / `KHONG`.
#
# `tail -c 1` rồi `od -c`: nếu byte cuối là `\n` thì `od -c` in `\n`.
byte_cuoi_la_newline() {
  if [ "$(tail -c 1 "$1" | od -c | head -1 | tr -s ' ' | cut -d' ' -f2)" = '\n' ]; then
    echo "CO"
  else
    echo "KHONG"
  fi
}

# ---------------------------------------------------------------------------
# `chay_luot <tên> <kieu_dong: crlf|lf>`
#
# Dựng một repo tạm, một tệp 30 dòng với ba chỗ sửa cách xa nhau (→ ba khối),
# stage **khối giữa** bằng cùng tập cờ `CO_APPLY` của `commands/hunk.rs`, rồi ghi
# số đo trước/sau.
#
# Đặt kết quả vào bốn biến toàn cục (bash không trả mảng được):
#   R_CR_TRUOC R_CR_SAU R_CUOI_NL_SAU R_E9_SAU  R_SO_KHOI
# ---------------------------------------------------------------------------
chay_luot() {
  local ten="$1" kieu="$2"
  local repo="$GOC_TAM/$ten"
  local f="$repo/muctieu.txt"

  # Đặt sẵn giá trị "chưa đo được" cho MỌI biến ra: nếu một bước giữa chừng thất
  # bại và `return` sớm, phần in tổng kết vẫn đọc được thay vì nổ `unbound
  # variable` — và một lỗi nổ giữa chừng thì KHÔNG in được lý do thật.
  R_CR_TRUOC=-1; R_CR_SAU=-1; R_SO_KHOI=-1; R_APPLY_EXIT=-1; R_APPLY_LOG=/dev/null
  R_E9_TRUOC=KHONG; R_E9_SAU=KHONG; R_CACHED_GIUA=0; R_CACHED_NGOAI=99
  R_CUOI_NL_TRUOC=CO; R_CUOI_NL_SAU=CO

  mkdir -p "$repo"
  GIT -C "$repo" init --quiet
  GIT -C "$repo" config user.email "gate@git-plum.test"
  GIT -C "$repo" config user.name  "byte gate"

  # 🔴 TƯỜNG MINH, và `true` là mặc định Windows — xem doc comment đầu tệp.
  GIT -C "$repo" config core.autocrlf true

  # --- tệp nền: 30 dòng, byte 0xE9 ở dòng 6, KHÔNG dòng trống cuối ----------
  #
  # `Buffer.from(s, "binary")` là điều làm `\xe9` thành BYTE 0xE9. Ghi thẳng
  # chuỗi JS cho ra UTF-8 `\303\251` — đúng bẫy shell cũng mắc.
  #
  # Không có ký tự kết thúc dòng ở cuối: tệp phải **kết thúc giữa chừng một
  # dòng**, vì đó là nửa thứ hai của WORK-04.
  node -e '
    const fs = require("fs");
    // 🔴 EOL suy ra TỪ TÊN KIỂU, không nhận qua đối số.
    //
    // `node -e … "$(printf "\r\n")"` truyền chuỗi **RỖNG**: thay thế lệnh của
    // shell cắt mọi ký tự xuống dòng ở cuối, và `\r\n` toàn là ký tự xuống
    // dòng. Đã đo hôm nay — nó làm tệp "CRLF" ra đời với **0** byte `\r`, và
    // thứ bắt được là lượt ĐỐI CHỨNG (hai lượt cho cùng `\r=0` → script báo
    // "không phân biệt được" và thoát 1). Cổng đỏ đúng chỗ, trước khi ai kịp
    // tin một kết quả sai.
    const [p, kieu] = [process.argv[1], process.argv[2]];
    const eol = kieu === "crlf" ? "\r\n" : "\n";
    const L = [];
    for (let i = 1; i <= 30; i++) {
      if (i === 6) L.push("caf\xe9 dong 06");
      else L.push("dong " + String(i).padStart(2, "0"));
    }
    fs.writeFileSync(p, Buffer.from(L.join(eol), "binary"));
  ' "$(W "$f")" "$kieu"

  GIT -C "$repo" add -- muctieu.txt
  GIT -C "$repo" commit --quiet -m "nen"

  # --- làm bẩn ở BA chỗ cách xa nhau → ba khối ------------------------------
  #
  # Dòng 3, 16, 29. Cách nhau > 6 dòng nên `git diff --unified=3` KHÔNG gộp
  # chúng thành một khối — ba chỗ sát nhau sẽ cho MỘT khối, và một bản vá một
  # khối là fixture "đúng hình dạng, dữ liệu vô hại" (CONTEXT.md 4.2): nó không
  # phân biệt được "stage đúng khối giữa" với "stage cả tệp".
  #
  # Đọc lại bằng `"binary"` — đọc `"utf8"` thay 0xE9 bằng U+FFFD và phá chính ca
  # fixture này tồn tại vì nó.
  node -e '
    const fs = require("fs");
    const [p, kieu] = [process.argv[1], process.argv[2]];
    const eol = kieu === "crlf" ? "\r\n" : "\n";
    const L = fs.readFileSync(p).toString("binary").split(eol);
    L[2] = "MOD 03"; L[15] = "MOD 16"; L[28] = "MOD 29";
    fs.writeFileSync(p, Buffer.from(L.join(eol), "binary"));
  ' "$(W "$f")" "$kieu"

  # --- SỐ ĐO TRƯỚC ---------------------------------------------------------
  R_CR_TRUOC="$(dem_cr "$f")"
  local e9_truoc cuoi_truoc
  e9_truoc="$(co_0xE9 "$f")"
  cuoi_truoc="$(byte_cuoi_la_newline "$f")"

  # --- dựng bản vá chỉ chứa KHỐI GIỮA --------------------------------------
  #
  # `--no-ext-diff` + `GIT_EXTERNAL_DIFF` gỡ hẳn: `CONTEXT.md` 4.4 — một chương
  # trình external diff KHÔNG TỒN TẠI làm `git diff` thoát **0** với 0 hunk và
  # **không stderr**. Đúng lỗi Phase 1 tốn nhất, và nó im lặng.
  local patch_day="$GOC_TAM/$ten.day.patch" patch_giua="$GOC_TAM/$ten.giua.patch"
  ( unset GIT_EXTERNAL_DIFF; \
    GIT -C "$repo" --no-pager diff --no-ext-diff --no-color -- muctieu.txt ) \
    > "$patch_day" 2>/dev/null

  R_SO_KHOI="$(grep -c '^@@' "$patch_day" || true)"

  # Tách khối **thứ hai** ra: giữ nguyên phần đầu (`diff --git`… `+++`), rồi chỉ
  # chép thân của khối 2. Header `@@` chép **nguyên văn** — và đó là lý do
  # `--recount` bắt buộc, xem `CONTEXT.md` 2.2.
  node -e '
    const fs = require("fs");
    const [src, dst] = [process.argv[1], process.argv[2]];
    const raw = fs.readFileSync(src).toString("binary");
    const lines = raw.split("\n");
    const head = [], khoi = [];
    let cur = null;
    for (const l of lines) {
      if (l.startsWith("@@")) { cur = [l]; khoi.push(cur); }
      else if (cur === null) head.push(l);
      else cur.push(l);
    }
    if (khoi.length < 3) {
      console.error("🔴 tiền đề sai: cần >= 3 khối, thấy " + khoi.length);
      process.exit(3);
    }
    // 🔴 `+ "\n"` ở cuối, và nó KHÔNG phải làm đẹp.
    //
    // `git diff` kết thúc bản vá bằng một `\n`; `split("\n")` sinh một phần tử
    // rỗng cuối, và `join("\n")` trên các khối đã tách **mất** ký tự đó. Kết
    // quả: dòng cuối cùng của khối không có ký tự kết thúc dòng, và git trả
    //     error: corrupt patch at <tệp>:13
    // đúng ở dòng cuối. Đã đo hôm nay (exit 128).
    //
    // Đây là đúng lớp lỗi `CONTEXT.md` 4.3 ("phân tích byte — ba lần hỏng im
    // lặng"): một ký tự ở ranh giới, không lỗi biên dịch, và triệu chứng trỏ
    // vào một chỗ nghe như lỗi nội dung chứ không như lỗi thiếu newline.
    const out = head.concat(khoi[1]).join("\n") + "\n";
    fs.writeFileSync(dst, Buffer.from(out, "binary"));
  ' "$(W "$patch_day")" "$(W "$patch_giua")" || return 3

  # --- áp bản vá bằng CÙNG tập cờ `CO_APPLY` của `hunk.rs` ------------------
  #
  #   pub const CO_APPLY: &[&str] = &["apply", "--cached", "--recount"];
  #
  # `--cached` = chỉ index, KHÔNG chạm thư mục làm việc. Đây là điều WORK-04
  # dựa vào: nếu tệp thật đổi một byte thì mọi số đo dưới đây vô nghĩa.
  # `2>&1` vào một tệp thay vì `/dev/null`: khi exit != 0 ta **phải** in được
  # git nói gì. `CONTEXT.md` 4.3 — nuốt stderr là cách ba lỗi byte trước đây
  # hỏng im lặng.
  GIT -C "$repo" apply --cached --recount "$patch_giua" > "$GOC_TAM/$ten.apply.log" 2>&1
  R_APPLY_EXIT=$?
  R_APPLY_LOG="$GOC_TAM/$ten.apply.log"

  # --- Chỉ KHỐI GIỮA vào index, hai khối kia KHÔNG -------------------------
  #
  # 🔴 Không có phép kiểm này, một cài đặt stage **cả tệp** vẫn qua được cả ba
  # số đo byte: tệp trong thư mục làm việc không đổi (nhờ `--cached`), nên
  # `\r`, `0xE9` và dòng cuối đều "giữ nguyên" một cách hoàn hảo. Fixture đúng
  # hình dạng, dữ liệu vô hại — `CONTEXT.md` 4.2.
  #
  # `git diff --cached` = thứ đã vào vùng chờ. Phải có `MOD 16` và **không** có
  # `MOD 03` / `MOD 29`.
  local cached="$GOC_TAM/$ten.cached.patch"
  GIT -C "$repo" --no-pager diff --cached --no-ext-diff --no-color -- muctieu.txt \
    > "$cached" 2>/dev/null
  R_CACHED_GIUA="$(grep -c '^+MOD 16' "$cached" || true)"
  R_CACHED_NGOAI="$(( $(grep -c '^+MOD 03' "$cached" || true) + $(grep -c '^+MOD 29' "$cached" || true) ))"

  # --- SỐ ĐO SAU -----------------------------------------------------------
  R_CR_SAU="$(dem_cr "$f")"
  R_E9_SAU="$(co_0xE9 "$f")"
  R_CUOI_NL_SAU="$(byte_cuoi_la_newline "$f")"
  R_E9_TRUOC="$e9_truoc"
  R_CUOI_NL_TRUOC="$cuoi_truoc"
}

# ===========================================================================
# LƯỢT 1 — CRLF THẬT
# ===========================================================================
echo "==========================================================="
echo " LƯỢT 1: CRLF thật  (core.autocrlf=true, 0xE9 dòng 6, thiếu dòng cuối)"
echo "==========================================================="
chay_luot luot1-crlf crlf
L1_CR_TRUOC="$R_CR_TRUOC"; L1_CR_SAU="$R_CR_SAU"
L1_E9_TRUOC="$R_E9_TRUOC"; L1_E9_SAU="$R_E9_SAU"
L1_NL_TRUOC="$R_CUOI_NL_TRUOC"; L1_NL_SAU="$R_CUOI_NL_SAU"
L1_KHOI="$R_SO_KHOI"; L1_APPLY="$R_APPLY_EXIT"; L1_LOG="$R_APPLY_LOG"
L1_CGIUA="$R_CACHED_GIUA"; L1_CNGOAI="$R_CACHED_NGOAI"

echo "  [số khối trong diff đầy đủ = $L1_KHOI]   (cần >= 3 — xem doc comment)"
echo "  [git apply --cached --recount exit = $L1_APPLY]"
echo "  [vùng chờ: '+MOD 16' = $L1_CGIUA (cần 1)   '+MOD 03'/'+MOD 29' = $L1_CNGOAI (cần 0)]"
echo "  [\\r  trước = $L1_CR_TRUOC   sau = $L1_CR_SAU]"
echo "  [0xE9 trước = $L1_E9_TRUOC   sau = $L1_E9_SAU]"
echo "  [byte cuối là \\n?  trước = $L1_NL_TRUOC   sau = $L1_NL_SAU]"
echo

L1_DAT=1

if [ "$L1_APPLY" -ne 0 ]; then
  echo "  🔴 KHÔNG ĐẠT — \`git apply --cached --recount\` thoát $L1_APPLY."
  echo "     Bản vá khối giữa không áp được; ba số đo dưới đây KHÔNG nói gì về"
  echo "     staging một phần, vì chưa có lần staging nào xảy ra."
  echo "     git nói: [$(cat "$L1_LOG" 2>/dev/null)]"
  L1_DAT=0
fi

if [ "$L1_KHOI" -lt 3 ]; then
  echo "  🔴 TIỀN ĐỀ SAI — chỉ $L1_KHOI khối, cần >= 3."
  echo "     Một bản vá ít hơn ba khối không phân biệt được \"stage đúng khối"
  echo "     giữa\" với \"stage cả tệp\" (CONTEXT.md 4.2). KHÔNG coi là đạt."
  L1_DAT=0
fi

# (0) ĐÚNG khối giữa vào vùng chờ — tiền đề của cả ba số đo byte
if [ "$L1_CGIUA" -eq 1 ] && [ "$L1_CNGOAI" -eq 0 ]; then
  echo "  ✅ (0) chỉ KHỐI GIỮA vào vùng chờ: +MOD 16 = $L1_CGIUA, hai khối kia = $L1_CNGOAI"
else
  echo "  🔴 (0) SAI khối vào vùng chờ: +MOD 16 = $L1_CGIUA (cần 1), hai khối kia = $L1_CNGOAI (cần 0)"
  echo "       Đây là \"staging MỘT PHẦN\" — stage cả tệp cũng cho ba số đo byte"
  echo "       hoàn hảo (nhờ --cached), nên thiếu phép kiểm này thì ba số đo kia"
  echo "       KHÔNG nói gì (CONTEXT.md 4.2)."
  L1_DAT=0
fi

# (1) số byte \r bằng nhau trước/sau
if [ "$L1_CR_TRUOC" = "$L1_CR_SAU" ] && [ "$L1_CR_TRUOC" -gt 0 ]; then
  echo "  ✅ (1) CRLF giữ nguyên:      \\r $L1_CR_TRUOC → $L1_CR_SAU"
else
  echo "  🔴 (1) CRLF KHÔNG giữ nguyên: \\r $L1_CR_TRUOC → $L1_CR_SAU"
  [ "$L1_CR_TRUOC" -eq 0 ] && echo "       (và \\r trước = 0 — fixture không mang CRLF, tiền đề sai)"
  L1_DAT=0
fi

# (2) byte cuối vẫn KHÔNG phải \n
if [ "$L1_NL_TRUOC" = "KHONG" ] && [ "$L1_NL_SAU" = "KHONG" ]; then
  echo "  ✅ (2) vẫn thiếu dòng cuối:  byte cuối là \\n? $L1_NL_TRUOC → $L1_NL_SAU"
else
  echo "  🔴 (2) dòng cuối bị THÊM:    byte cuối là \\n? $L1_NL_TRUOC → $L1_NL_SAU"
  L1_DAT=0
fi

# (3) byte 0xE9 vẫn còn
if [ "$L1_E9_TRUOC" = "CO" ] && [ "$L1_E9_SAU" = "CO" ]; then
  echo "  ✅ (3) byte 0xE9 giữ nguyên: $L1_E9_TRUOC → $L1_E9_SAU"
else
  echo "  🔴 (3) byte 0xE9 MẤT/đổi:    $L1_E9_TRUOC → $L1_E9_SAU"
  L1_DAT=0
fi
echo

# ===========================================================================
# LƯỢT 2 — ĐỐI CHỨNG LF THUẦN
#
# 🔴 Lượt này KHÔNG phải một ca thứ hai cần đạt. Nó tồn tại để chứng minh phép
# kiểm `\r` của lượt 1 **đọc byte thật**, chứ không trả một hằng số. Nếu hai lượt
# cho CÙNG số `\r` thì lượt 1 không đo gì cả, và "ĐẠT" của nó vô giá trị —
# CONTEXT.md 4.1 lỗi #4.
# ===========================================================================
echo "==========================================================="
echo " LƯỢT 2: đối chứng LF thuần  (cùng kịch bản, chỉ đổi ký tự kết thúc dòng)"
echo "==========================================================="
chay_luot luot2-lf lf
L2_CR_TRUOC="$R_CR_TRUOC"; L2_CR_SAU="$R_CR_SAU"
L2_KHOI="$R_SO_KHOI"; L2_APPLY="$R_APPLY_EXIT"

echo "  [số khối trong diff đầy đủ = $L2_KHOI]"
echo "  [git apply --cached --recount exit = $L2_APPLY]"
echo "  [\\r  trước = $L2_CR_TRUOC   sau = $L2_CR_SAU]"
echo

PHAN_BIET=1
if [ "$L1_CR_TRUOC" = "$L2_CR_TRUOC" ]; then
  echo "  🔴 KHÔNG PHÂN BIỆT ĐƯỢC — hai lượt cho CÙNG số \\r ($L1_CR_TRUOC)."
  echo "     Phép kiểm \\r không đọc byte thật; nó trả một hằng số. \"ĐẠT\" của"
  echo "     lượt 1 KHÔNG nói gì (CONTEXT.md 4.1 lỗi #4)."
  PHAN_BIET=0
else
  echo "  ✅ PHÂN BIỆT ĐƯỢC — lượt 1 \\r=$L1_CR_TRUOC, lượt 2 \\r=$L2_CR_TRUOC (khác nhau)."
  echo "     Phép kiểm \\r của lượt 1 đọc byte thật của tệp."
fi
echo

# ===========================================================================
# TỔNG KẾT
# ===========================================================================
echo "==========================================================="
printf '    [lượt 1: CRLF thật    ] \\r trước=%-4s sau=%-4s → %s\n' \
  "$L1_CR_TRUOC" "$L1_CR_SAU" "$([ "$L1_DAT" -eq 1 ] && echo ĐẠT || echo 'KHÔNG ĐẠT')"
printf '    [lượt 2: đối chứng LF ] \\r trước=%-4s sau=%-4s → %s\n' \
  "$L2_CR_TRUOC" "$L2_CR_SAU" "$([ "$PHAN_BIET" -eq 1 ] && echo 'KHÁC lượt 1 (đúng)' || echo 'GIỐNG lượt 1 (SAI)')"
echo "==========================================================="

if [ "$L1_DAT" -eq 1 ] && [ "$PHAN_BIET" -eq 1 ]; then
  echo "✅ WORK-04 / tiêu chí thành công 2 — ĐẠT, do MÁY kiểm."
  exit 0
fi

echo "🔴 WORK-04 / tiêu chí thành công 2 — KHÔNG ĐẠT."
exit 1
