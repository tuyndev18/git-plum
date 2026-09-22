#!/usr/bin/env bash
#
# Sinh repo mẫu `hunk-cases` cho staging theo khối của Phase 5 — WORK-03, WORK-04.
#
# Vị trí trong bộ bốn script fixture:
#   * make-fixtures.sh        (Phase 2) — mỗi repo một hình dạng ĐỒ THỊ bệnh lý;
#   * make-diff-fixtures.sh   (Phase 3) — một repo, nhiều hình dạng TỆP bệnh lý;
#   * make-status-fixtures.sh (Phase 4) — một repo có THƯ MỤC LÀM VIỆC BẨN theo hình
#                                          dạng làm vỡ bộ phân tích `--porcelain=v2 -z`;
#   * đây                     (Phase 5) — một repo có thư mục làm việc bẩn theo hình
#                                          dạng làm vỡ một bộ DỰNG BẢN VÁ CON.
#
# Như `status-cases`, repo này để BẨN CÓ CHỦ Ý: giá trị của nó nằm ở thư mục làm việc,
# không ở lịch sử. Chạy `git commit`, `git reset` hay `git clean` trong đó là phá fixture.
#
# ---------------------------------------------------------------------------
# 🔴 Ràng buộc bắt buộc: `ba-khoi.txt` phải cho ĐÚNG 3 hunk ở `-U3`
# ---------------------------------------------------------------------------
#
# 26 dòng (`a`..`z`), sửa dòng 2, 13, 25. Đã ĐO trên git 2.54.0.windows.1 — không suy ra.
#
# Ở `-U3` mỗi hunk lấy 3 dòng ngữ cảnh mỗi bên, nên hai chỗ sửa cách nhau <= 6 dòng sẽ
# GỘP thành một hunk. Khoảng cách ở đây là 11 dòng, dư an toàn. Đổi số dòng hay vị trí
# sửa mà không đo lại `grep -c '^@@'` là làm fixture mất khả năng phân biệt trong im
# lặng — đúng lỗi cổng #4 của CONTEXT.md mục 4.1.
#
# Script TỰ ĐO số hunk ở cuối và THOÁT 1 nếu sai. Một fixture sinh sai mà im lặng tệ
# hơn không có fixture.
#
# ---------------------------------------------------------------------------
# 🔴 `core.autocrlf` đặt TƯỜNG MINH `true`
# ---------------------------------------------------------------------------
#
# Mặc định trên Windows là `true` và đó là máy chủ dự án. Một fixture đặt `false` sẽ né
# mất chính ca R2 (CRLF bị chuẩn hoá mất) mà nó tồn tại để kiểm.
#
# ĐÃ ĐO, và đây là điều quan trọng nhất phải hiểu về fixture này:
#
#   với `autocrlf=true`, `git diff` in bản vá bằng LF THUẦN — clean filter đã bỏ `\r`
#   trước khi git so sánh. Bản vá KHÔNG chứa byte `\r` nào.
#
# Đó là ĐÚNG, không phải lỗi. `autocrlf=true` nghĩa là repo lưu LF còn đĩa giữ CRLF.
# Thứ WORK-04 bảo vệ là TỆP TRONG THƯ MỤC LÀM VIỆC, và nó không đổi.
#
# Hệ quả cho test: ca "CRLF không bị trim khi tách hunk" KHÔNG dựng được từ `git diff`
# của repo này. Nó phải là test BYTE-LITERAL trong `patch_build.rs`. Repo có
# `autocrlf=false` hoặc `core.eol=crlf` thì bản vá MỚI mang `\r`, nên ca đó có thật —
# chỉ là fixture này không phải đường vào của nó.
#
# ---------------------------------------------------------------------------
# 🔴 Byte 0xE9 KHÔNG tạo được bằng shell
# ---------------------------------------------------------------------------
#
# `printf 'caf\351'` bị chuyển sang UTF-8 (`caf\303\251`) TRƯỚC khi tới hệ thống tệp —
# đã đo và ghi trong `make-fixtures.sh` mục non-utf8, gặp lại ở `make-status-fixtures.sh`.
# Dùng `node -e` với `Buffer.from(s, "binary")`: không có `python` trên máy này.
#
# Khác `status-cases`: ở đó byte thô nằm trong TÊN TỆP và NTFS không đựng được nó. Ở
# đây nó nằm trong NỘI DUNG tệp, nơi mọi hệ thống tệp đều đựng được — nên ca này dựng
# được trên cả ba nền tảng, và script ĐÒI nó có mặt (thoát 1 nếu thiếu).
#
# Cách dùng:
#   bash scripts/fixtures/make-hunk-fixtures.sh [thư-mục-đích]
# Mặc định: <gốc-dự-án>/target/fixtures/hunk-cases

set -euo pipefail

# ---------------------------------------------------------------------------
# Đường dẫn và cổng an toàn — sao lại từ make-status-fixtures.sh, cùng lý do
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEST_ARG="${1:-$PROJECT_ROOT/target/fixtures/hunk-cases}"

if [ -z "${DEST_ARG// /}" ] || [ "$DEST_ARG" = "/" ]; then
  echo "LỖI: thư mục đích rỗng hoặc là gốc hệ thống tệp — từ chối chạy." >&2
  exit 1
fi

mkdir -p "$DEST_ARG"
DEST="$(cd "$DEST_ARG" && pwd)"

case "$DEST" in
  "$PROJECT_ROOT"/*) ;;
  *)
    echo "LỖI: thư mục đích '$DEST' không nằm dưới gốc dự án '$PROJECT_ROOT'." >&2
    exit 1
    ;;
esac

if [ "$DEST" = "$PROJECT_ROOT" ]; then
  echo "LỖI: thư mục đích trùng gốc dự án — từ chối chạy." >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Tính tất định
# ---------------------------------------------------------------------------

export GIT_AUTHOR_NAME="Plum Fixture"
export GIT_AUTHOR_EMAIL="fixture@git-plum.test"
export GIT_COMMITTER_NAME="Plum Fixture"
export GIT_COMMITTER_EMAIL="fixture@git-plum.test"

readonly EPOCH_BASE=1577836800
export GIT_AUTHOR_DATE="$((EPOCH_BASE + 60)) +0000"
export GIT_COMMITTER_DATE="$((EPOCH_BASE + 60)) +0000"

# Xoá và KIỂM đã xoá được thật — trên Windows `rm -rf` thất bại MỘT PHẦN rồi trả mã 0
# khi OneDrive/Explorer đang giữ tệp. Xem ghi chú trong make-diff-fixtures.sh.
xoa_that() {
  local path="$1"
  rm -rf "$path" 2>/dev/null || true
  if [ -e "$path" ]; then
    sleep 1
    rm -rf "$path" 2>/dev/null || true
  fi
  if [ -e "$path" ]; then
    echo "LỖI: không xoá được '$path' — có tiến trình đang giữ tệp trong đó." >&2
    echo "      Trên Windows thường là OneDrive, trình diệt virus, hoặc một cửa sổ" >&2
    echo "      Explorer/terminal đang mở thư mục đó." >&2
    exit 1
  fi
}

REPO="$DEST/repo"
xoa_that "$REPO"
mkdir -p "$REPO"

git init --initial-branch=main --quiet "$REPO"

# 🔴 TƯỜNG MINH `true` — xem khối ràng buộc ở đầu tệp. Đây là điểm khác biệt DUY NHẤT
# quan trọng so với ba script fixture trước, vốn đều đặt `false` để đầu ra tất định.
git -C "$REPO" config core.autocrlf true
git -C "$REPO" config commit.gpgsign false
git -C "$REPO" config core.quotepath true
git -C "$REPO" config user.name "$GIT_AUTHOR_NAME"
git -C "$REPO" config user.email "$GIT_AUTHOR_EMAIL"

echo "Sinh repo mẫu hunk vào: $REPO"
echo "  core.autocrlf = $(git -C "$REPO" config core.autocrlf)   (TƯỜNG MINH, xem đầu script)"
echo

# ---------------------------------------------------------------------------
# 1. `ba-khoi.txt` — 26 dòng LF, ca `--recount` và ca tách hunk cơ bản
# ---------------------------------------------------------------------------
echo "[1] ba-khoi.txt: 26 dòng LF, sẽ sửa dòng 2/13/25 -> đúng 3 hunk ở -U3"

# Ghi bằng node chứ không bằng shell: `join("\n") + "\n"` cho đúng 26 dòng và ĐÚNG MỘT
# newline cuối. Một vòng `split`/`join` qua chuỗi để lại phần tử rỗng cuối và sinh
# newline THỪA — đã mắc lỗi đó lúc dò, và nó làm `git diff` mọc thêm một dòng `+` ma.
node -e '
const fs = require("fs");
const L = [];
for (let i = 0; i < 26; i++) L.push(String.fromCharCode(97 + i));
fs.writeFileSync(process.argv[1], L.join("\n") + "\n");
' "$REPO/ba-khoi.txt"

# ---------------------------------------------------------------------------
# 2. `crlf-khong-dong-cuoi.txt` — BA lớp bệnh lý trong MỘT tệp
#
# 30 dòng CRLF · không `\n` cuối · byte `0xE9` ở dòng 6.
#
# Ba lớp cùng một tệp là CÓ CHỦ Ý: CONTEXT.md 4.2 nói một tệp chỉ dùng LF hoặc toàn
# ASCII là fixture "đúng hình dạng, dữ liệu vô hại". Gộp cả ba bắt một cài đặt phải
# giữ nguyên CẢ BA cùng lúc, không được đúng từng cái một rồi hỏng khi giao nhau.
# ---------------------------------------------------------------------------
echo "[2] crlf-khong-dong-cuoi.txt: 30 dòng CRLF + byte 0xE9 dòng 6 + KHÔNG newline cuối"

node -e '
const fs = require("fs");
const L = [];
for (let i = 1; i <= 30; i++) {
  // \xe9 chỉ thành BYTE 0xE9 khi đi qua Buffer.from(..., "binary").
  // Ghi thẳng chuỗi JS sẽ ra UTF-8 \303\251 — đúng cái bẫy shell cũng mắc.
  if (i === 6) L.push("caf\xe9 dong 06");
  else L.push("dong " + String(i).padStart(2, "0"));
}
// KHÔNG có "+ \r\n" ở cuối: tệp phải KẾT THÚC giữa chừng một dòng.
fs.writeFileSync(process.argv[1], Buffer.from(L.join("\r\n"), "binary"));
' "$REPO/crlf-khong-dong-cuoi.txt"

# ---------------------------------------------------------------------------
# 3. `nhi-phan.bin` — có byte NUL, R7 ("tệp nhị phân lọt vào đường text")
# ---------------------------------------------------------------------------
echo "[3] nhi-phan.bin: có byte NUL, git phải coi là nhị phân"

node -e '
const fs = require("fs");
const b = Buffer.alloc(64);
for (let i = 0; i < 64; i++) b[i] = i * 3 % 256;
b[10] = 0x00; b[11] = 0x00; b[30] = 0x00;   // NUL: dấu hiệu git dùng để nhận nhị phân
fs.writeFileSync(process.argv[1], b);
' "$REPO/nhi-phan.bin"

# ---------------------------------------------------------------------------
# 4. `doi-ten-va-sua.txt` — ca ROADMAP "vừa đổi tên vừa sửa"
# ---------------------------------------------------------------------------
echo "[4] doi-ten-cu.txt: sẽ được git mv rồi sửa nội dung"

node -e '
const fs = require("fs");
const L = [];
for (let i = 1; i <= 20; i++) L.push("noi dung goc dong " + String(i).padStart(2, "0"));
fs.writeFileSync(process.argv[1], L.join("\n") + "\n");
' "$REPO/doi-ten-cu.txt"

# --- commit nền: mọi tệp trên vào lịch sử ở trạng thái SẠCH -----------------
git -C "$REPO" add -- ba-khoi.txt crlf-khong-dong-cuoi.txt nhi-phan.bin doi-ten-cu.txt
git -C "$REPO" commit --quiet -m "commit nen"

# ---------------------------------------------------------------------------
# 5. Làm bẩn thư mục làm việc — KHÔNG commit sau điểm này
# ---------------------------------------------------------------------------
echo
echo "[5] làm bẩn thư mục làm việc (KHÔNG commit sau đây)"

# 5a. ba-khoi.txt: sửa dòng 2, 13, 25 (chỉ số mảng 1, 12, 24)
node -e '
const fs = require("fs");
const L = [];
for (let i = 0; i < 26; i++) L.push(String.fromCharCode(97 + i));
L[1] = "M1"; L[12] = "M2"; L[24] = "M3";
fs.writeFileSync(process.argv[1], L.join("\n") + "\n");
' "$REPO/ba-khoi.txt"
echo "    ba-khoi.txt: sửa dòng 2, 13, 25"

# 5b. crlf-khong-dong-cuoi.txt: sửa dòng 3, 16, 29 — giữ NGUYÊN CRLF và vẫn KHÔNG
#     newline cuối. Đọc lại bằng "binary" để byte 0xE9 của dòng 6 đi qua nguyên vẹn;
#     đọc bằng "utf8" sẽ thay nó bằng U+FFFD và phá chính ca fixture này tồn tại vì nó.
node -e '
const fs = require("fs");
const p = process.argv[1];
const L = fs.readFileSync(p).toString("binary").split("\r\n");
L[2] = "MOD 03"; L[15] = "MOD 16"; L[28] = "MOD 29";
fs.writeFileSync(p, Buffer.from(L.join("\r\n"), "binary"));
' "$REPO/crlf-khong-dong-cuoi.txt"
echo "    crlf-khong-dong-cuoi.txt: sửa dòng 3, 16, 29 (CRLF và thiếu newline cuối giữ nguyên)"

# 5c. nhi-phan.bin: sửa vài byte
node -e '
const fs = require("fs");
const p = process.argv[1];
const b = fs.readFileSync(p);
b[5] = 0xff; b[40] = 0x01; b[41] = 0x00;
fs.writeFileSync(p, b);
' "$REPO/nhi-phan.bin"
echo "    nhi-phan.bin: sửa byte 5, 40, 41"

# 5d. doi-ten-va-sua.txt: git mv rồi sửa nội dung
git -C "$REPO" mv doi-ten-cu.txt doi-ten-va-sua.txt
node -e '
const fs = require("fs");
const p = process.argv[1];
const L = fs.readFileSync(p, "utf8").split("\n");
L[3] = "DA SUA sau khi doi ten";
fs.writeFileSync(p, L.join("\n"));
' "$REPO/doi-ten-va-sua.txt"
echo "    doi-ten-cu.txt -> doi-ten-va-sua.txt, rồi sửa dòng 4"

# ===========================================================================
# KIỂM CHỨNG — script tự khẳng định tiền đề của chính nó
#
# Mọi phép kiểm dưới đây THOÁT 1 khi sai. Không có nhánh nào in cảnh báo rồi thoát 0:
# một fixture âm thầm mất tính phân biệt tệ hơn một fixture không tồn tại (lỗi cổng #4,
# CONTEXT.md 4.1).
#
# 🔴 stdout của `git diff` đi vào TỆP, stderr đi chỗ khác. Với `autocrlf=true`, git in
# `warning: ... LF will be replaced by CRLF` ra stderr, và gộp hai luồng sẽ nhét dòng
# cảnh báo đó vào GIỮA NỘI DUNG BẢN VÁ. Đây đúng lớp lỗi T-03-15 mà `diff.rs` đã ghi,
# và ở Phase 5 hậu quả nặng hơn: nó thành nội dung tệp người dùng.
# ===========================================================================
echo
echo "=== Kiểm chứng ==="

BANG="$DEST/hunks.txt"
: >"$BANG"
LOI=0

# --- Đo số hunk của từng tệp ------------------------------------------------
dem_hunk() {
  local tep="$1"
  local ra="$DEST/.d.patch"
  git -C "$REPO" --no-pager diff --no-ext-diff -U3 -- "$tep" >"$ra" 2>"$DEST/.d.err"
  grep -c '^@@' "$ra" || true
}

H_BA="$(dem_hunk ba-khoi.txt)"
H_CRLF="$(dem_hunk crlf-khong-dong-cuoi.txt)"

printf 'ba-khoi.txt %s\n' "$H_BA" >>"$BANG"
printf 'crlf-khong-dong-cuoi.txt %s\n' "$H_CRLF" >>"$BANG"

echo "  ba-khoi.txt                hunks = $H_BA   (cần ĐÚNG 3)"
if [ "$H_BA" -ne 3 ]; then
  echo "LỖI: ba-khoi.txt cho $H_BA hunk, cần ĐÚNG 3." >&2
  echo "      Với != 3 hunk, mọi test 'chọn hunk giữa' mất ý nghĩa: hoặc không có hunk" >&2
  echo "      giữa, hoặc hai chỗ sửa đã GỘP vì cách nhau <= 6 dòng ở -U3." >&2
  LOI=1
fi

echo "  crlf-khong-dong-cuoi.txt   hunks = $H_CRLF   (cần ĐÚNG 3)"
if [ "$H_CRLF" -ne 3 ]; then
  echo "LỖI: crlf-khong-dong-cuoi.txt cho $H_CRLF hunk, cần ĐÚNG 3." >&2
  LOI=1
fi

# --- Byte 0xE9 phải CÓ trong thư mục làm việc -------------------------------
#
# Đo bằng `od -c`: `351` là 0xE9 ở dạng bát phân. Đây là phép kiểm mà CONTEXT.md 4.3
# đòi ("mọi chỗ đọc/ghi phải `od -c` một lần trên git thật trước khi tin").
if od -c "$REPO/crlf-khong-dong-cuoi.txt" | grep -q '351'; then
  echo "  byte 0xE9 trong thư mục làm việc: CÓ"
else
  echo "LỖI: không thấy byte 0xE9 (\\351) trong crlf-khong-dong-cuoi.txt." >&2
  echo "      node -e đã ghi chuỗi ra UTF-8 thay vì byte thô — kiểm Buffer.from(.., \"binary\")." >&2
  LOI=1
fi

# --- CRLF phải CÒN trong thư mục làm việc -----------------------------------
if od -c "$REPO/crlf-khong-dong-cuoi.txt" | grep -q '\\r'; then
  echo "  CRLF trong thư mục làm việc: CÓ"
else
  echo "LỖI: không thấy byte \\r trong crlf-khong-dong-cuoi.txt." >&2
  LOI=1
fi

# --- Tệp KHÔNG được kết thúc bằng newline -----------------------------------
#
# Đọc byte CUỐI CÙNG. `tail -c 1 | od` là cách duy nhất chắc chắn: nhìn `od -c` bằng
# mắt thì một `\n` ở áp chót trông giống hệt một `\n` ở chót.
BYTE_CUOI="$(tail -c 1 "$REPO/crlf-khong-dong-cuoi.txt" | od -An -c | tr -d ' \n')"
echo "  byte cuối của tệp: [$BYTE_CUOI]   (KHÔNG được là \\n)"
if [ "$BYTE_CUOI" = '\n' ]; then
  echo "LỖI: crlf-khong-dong-cuoi.txt kết thúc bằng newline — ca 'thiếu dòng cuối' mất." >&2
  LOI=1
fi

# --- Bản vá phải mang dấu `\ No newline at end of file` ---------------------
#
# Đây là thứ `tach_hunk_tho` phải giữ đúng chỗ (Test 4 của plan 05-01): dòng đó thuộc
# về hunk CHỨA nó, không rơi vào phần đầu và không bị bỏ.
git -C "$REPO" --no-pager diff --no-ext-diff -U3 -- crlf-khong-dong-cuoi.txt \
  >"$DEST/.crlf.patch" 2>/dev/null
SO_NONEWLINE="$(grep -c 'No newline at end of file' "$DEST/.crlf.patch" || true)"
echo "  dấu '\\ No newline at end of file' trong bản vá: $SO_NONEWLINE   (cần >= 1)"
if [ "${SO_NONEWLINE:-0}" -lt 1 ]; then
  echo "LỖI: bản vá không mang dấu '\\ No newline at end of file'." >&2
  LOI=1
fi

# --- Byte 0xE9 phải đi qua được VÀO bản vá ----------------------------------
#
# Khác phép kiểm trên: ở đó nó nằm trên ĐĨA, ở đây nó phải sống sót qua `git diff`.
# Nếu git coi tệp là nhị phân vì byte đó thì bản vá sẽ là "Binary files differ" và mọi
# test staging theo hunk của WORK-04 mất đường vào — im lặng.
if od -c "$DEST/.crlf.patch" | grep -q '351'; then
  echo "  byte 0xE9 trong BẢN VÁ: CÓ (git coi tệp là text, đúng như cần)"
else
  echo "LỖI: byte 0xE9 không có trong bản vá — git có thể đã coi tệp là nhị phân." >&2
  echo "      Kiểm: git diff có in 'Binary files ... differ' không?" >&2
  LOI=1
fi

# --- ĐO (không đòi) số byte \r trong bản vá ---------------------------------
#
# 🔴 Đây là phép ĐO, có chủ ý KHÔNG phải phép đòi. Với `autocrlf=true` con số này là 0
# và đó là ĐÚNG: clean filter bỏ `\r` trước khi git so sánh, nên bản vá là LF thuần dù
# tệp trên đĩa là CRLF. Xem khối giải thích ở đầu script.
#
# In ra để người chạy THẤY, vì nó quyết định một điều trong test: ca "CRLF không bị
# trim khi tách hunk" phải là test BYTE-LITERAL, không dựng được từ repo này.
# 🔴 `|| true` là BẮT BUỘC, không phải cẩn thận thừa: `grep -o` thoát 1 khi KHÔNG khớp,
# và số khớp đúng ở đây thường là 0 (autocrlf=true). Dưới `set -e` một lệnh thay thế
# thoát khác 0 giết cả script — đã xảy ra, và nó dừng ngay TRƯỚC phép kiểm nhị phân nên
# trông hệt như phép kiểm nhị phân hỏng. Một phép ĐO không bao giờ được làm chết script.
SO_CR="$( { od -c "$DEST/.crlf.patch" | grep -o '\\r' || true; } | wc -l | tr -d ' ')"
echo "  số byte \\r trong bản vá: $SO_CR   (ĐO, không đòi — với autocrlf=true thì 0 là ĐÚNG)"
printf 'ban-va-so-cr %s\n' "$SO_CR" >>"$BANG"

# --- Tệp nhị phân phải được git NHẬN là nhị phân ----------------------------
#
# 🔴 Đo bằng `--numstat`, KHÔNG bằng cách tìm chuỗi "Binary files" trong `git diff`.
#
# Lý do là một bẫy công cụ đã tốn một lần chạy của chính script này: trên máy này `git`
# đi qua một proxy (`rtk`) LỌC đầu ra `git diff` — với tệp nhị phân nó thay bản vá thật
# bằng một dòng tóm tắt `nhi-phan.bin | Bin 64 -> 64 bytes`. Chuỗi "Binary files" biến
# mất, nên phép kiểm ĐỎ trong khi git hoàn toàn đúng. Chuyển hướng `> tệp` KHÔNG thoát
# được bộ lọc: nó lọc trước khi chuyển hướng.
#
# `--numstat` in `-<TAB>-<TAB><đường dẫn>` cho tệp nhị phân và đi qua NGUYÊN VẸN cả hai
# đường (đã đo: `git ... --numstat` và `rtk proxy git ... --numstat` cho đầu ra giống
# hệt nhau, `-^I-^Inhi-phan.bin$`). Nó cũng là hợp đồng ỔN ĐỊNH của git, không phải văn
# xuôi người đọc — một phép kiểm dựa vào câu tiếng Anh của git là phép kiểm mong manh
# kể cả khi không có proxy nào.
git -C "$REPO" --no-pager diff --no-ext-diff --numstat -- nhi-phan.bin \
  >"$DEST/.bin.numstat" 2>/dev/null
if grep -q '^-	-	' "$DEST/.bin.numstat"; then
  echo "  nhi-phan.bin: git nhận là NHỊ PHÂN (đúng — R7 phải từ chối rõ ràng)"
else
  echo "LỖI: git KHÔNG coi nhi-phan.bin là nhị phân; ca R7 mất đường vào." >&2
  echo "      Đầu ra --numstat đọc được:" >&2
  sed 's/^/        /' "$DEST/.bin.numstat" >&2
  LOI=1
fi

# --- Đổi tên phải được git PHÁT HIỆN là đổi tên -----------------------------
if git -C "$REPO" status --porcelain=v2 -z | tr '\0' '\n' | grep -q '^2 '; then
  echo "  doi-ten-va-sua.txt: git phát hiện ĐỔI TÊN (bản ghi dạng '2')"
else
  echo "LỖI: git không phát hiện đổi tên; ca 'vừa đổi tên vừa sửa' mất." >&2
  echo "      Kiểm cấu hình status.renames." >&2
  LOI=1
fi

rm -f "$DEST/.d.patch" "$DEST/.d.err" "$DEST/.bin.numstat"

if [ "$LOI" -ne 0 ]; then
  echo >&2
  echo "LỖI: fixture KHÔNG đạt — xem các dòng LỖI ở trên. Không dùng repo này." >&2
  exit 1
fi

# ===========================================================================
# Bảng tổng kết
# ===========================================================================
echo
echo "=== Tổng kết hunk-cases ==="
printf '%-28s %-8s %s\n' "TỆP" "HUNK" "VÌ SAO CÓ MẶT"
printf '%-28s %-8s %s\n' "----" "----" "-------------"
printf '%-28s %-8s %s\n' "ba-khoi.txt" "$H_BA" "ca tách hunk cơ bản, ca --recount"
printf '%-28s %-8s %s\n' "crlf-khong-dong-cuoi.txt" "$H_CRLF" "WORK-04: CRLF + 0xE9 + thiếu dòng cuối"
printf '%-28s %-8s %s\n' "nhi-phan.bin" "nhị phân" "R7: phải từ chối rõ ràng"
printf '%-28s %-8s %s\n' "doi-ten-va-sua.txt" "đổi tên" "ca ROADMAP 'vừa đổi tên vừa sửa'"
echo
echo "Repo: $REPO"
echo "Số hunk đo được ghi vào: $BANG"
echo
echo "Test Rust tìm repo này qua git_plum_lib::testing::require_hunk_fixture()."
echo "KHÔNG chạy git commit/reset/clean trong repo đó: giá trị của nó là trạng thái BẨN."
