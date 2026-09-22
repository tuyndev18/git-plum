#!/usr/bin/env bash
#
# Sinh repo mẫu `status-cases` cho bộ phân tích trạng thái của Phase 4 — WORK-01, WORK-11.
#
# Khác hai script fixture trước:
#   * make-fixtures.sh (Phase 2) — mỗi repo một hình dạng ĐỒ THỊ bệnh lý;
#   * make-diff-fixtures.sh (Phase 3) — một repo, nhiều hình dạng TỆP bệnh lý;
#   * đây (Phase 4) — một repo có THƯ MỤC LÀM VIỆC BẨN theo đúng hình dạng làm vỡ một
#     bộ phân tích `--porcelain=v2 -z` ngây thơ.
#
# Repo này KHÔNG commit thay đổi cuối: giá trị của nó nằm ở trạng thái bẩn còn lại trên
# đĩa. Chạy `git commit` hay `git reset` trong đó là phá fixture.
#
# ---------------------------------------------------------------------------
# 🔴 Ràng buộc bắt buộc: đích của `git mv` phải sort TRƯỚC ít nhất hai đường dẫn khác
# ---------------------------------------------------------------------------
#
# git sắp bản ghi theo đường dẫn. Bản ghi dạng `2` (đổi tên) chiếm HAI đoạn NUL trong
# MỘT bản ghi logic, nên một bộ phân tích tách `\0` trơn đọc đường dẫn cũ thành một bản
# ghi riêng rồi gán sai MỌI bản ghi sau nó — lệch nấc, hỏng IM LẶNG. Đó là R4 của
# CONTEXT.md và đúng lớp lỗi của bản ghi `R` ở wave 5 Phase 3.
#
# Test bắt lệch nấc chỉ phân biệt được khi có >= 2 bản ghi ĐỨNG SAU bản ghi dạng `2`.
# Đặt tên đích là `renamed.txt` thì nó sort CUỐI và còn 0 bản ghi sau nó — fixture khi
# đó không phân biệt được gì, đúng lỗi cổng #4 của CONTEXT.md mục 3.1. Đã đo cả hai
# cách. Tên `a_new.txt` sort ĐẦU và cho đúng 2 bản ghi theo sau.
#
# ĐỪNG ĐỔI TÊN CÁC TỆP DƯỚI ĐÂY mà không chạy lại phép đo đó.
#
# ---------------------------------------------------------------------------
# Tên tệp byte thô: phải qua `git fast-import`, không tạo được bằng shell
# ---------------------------------------------------------------------------
#
# Đã đo trên máy này (ghi trong make-fixtures.sh mục non-utf8): `printf 'caf\351.txt'`
# bị shell chuyển thành UTF-8 (`caf\303\251.txt`) TRƯỚC khi tới hệ thống tệp. Nên tên
# tệp byte Latin-1 phải đi vào repo qua `fast-import` rồi `reset --hard` để nó hiện ra
# trên đĩa. Ở đây ta cần nó ở dạng CHƯA THEO DÕI (bản ghi `?`), nên cách làm là: đưa nó
# vào một nhánh phụ bằng fast-import, checkout lấy tệp ra đĩa, rồi gỡ khỏi index.
#
# Cách dùng:
#   bash scripts/fixtures/make-status-fixtures.sh [thư-mục-đích]
# Mặc định: <gốc-dự-án>/target/fixtures/status-cases

set -euo pipefail

# ---------------------------------------------------------------------------
# Đường dẫn và cổng an toàn — sao lại từ make-diff-fixtures.sh, cùng lý do
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEST_ARG="${1:-$PROJECT_ROOT/target/fixtures/status-cases}"

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

# Xoá và KIỂM đã xoá được thật — xem ghi chú trong make-diff-fixtures.sh: trên Windows
# `rm -rf` thất bại MỘT PHẦN rồi trả mã 0 khi OneDrive/Explorer đang giữ tệp.
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
git -C "$REPO" config core.autocrlf false
git -C "$REPO" config commit.gpgsign false
git -C "$REPO" config core.quotepath true
git -C "$REPO" config user.name "$GIT_AUTHOR_NAME"
git -C "$REPO" config user.email "$GIT_AUTHOR_EMAIL"

echo "Sinh repo mẫu trạng thái vào: $REPO"
echo

# ---------------------------------------------------------------------------
# 1. Commit nền — bốn tệp theo dõi
#
# `p_mm.txt` nằm ở commit nền, còn phần làm bẩn nó ở mục 6. Nó **phải** vào đây chứ
# không được có một `git commit` riêng ở mục 6: lúc đó bản ghi đổi tên của mục 3 đã
# nằm trong index, nên một commit thêm sẽ **commit luôn nó** và phá toàn bộ trạng thái
# bẩn — chính là thứ duy nhất fixture này có giá trị.
# ---------------------------------------------------------------------------
echo "[1] commit nền"
printf 'aaa\n' >"$REPO/a_old.txt"
printf 'z\n' >"$REPO/m_one.txt"
printf 'z\n' >"$REPO/n_two.txt"
printf 'z\n' >"$REPO/p_mm.txt"
git -C "$REPO" add -- a_old.txt m_one.txt n_two.txt p_mm.txt
git -C "$REPO" commit --quiet -m "commit nen"

# ---------------------------------------------------------------------------
# 2. Tên tệp byte Latin-1 (\351 = é trong Latin-1, KHÔNG hợp lệ UTF-8 đơn lẻ)
#
# Qua fast-import vì shell không tạo được tên tệp byte thô (xem đầu tệp). Đưa vào một
# nhánh phụ, lấy tệp ra đĩa bằng `checkout <nhánh> -- <đường dẫn>`, rồi `rm --cached`
# để nó thành CHƯA THEO DÕI — dạng `?`, đúng thứ ta cần kiểm.
# ---------------------------------------------------------------------------
echo "[2] tên tệp byte Latin-1 qua fast-import"
STREAM="$DEST/.status.fi"

# 🔴 TUYỆT ĐỐI KHÔNG viết số byte của khối `data <n>` bằng tay.
#
# make-fixtures.sh đã trả giá cho bài học này: đếm tay "nội dung a\n" ra 14 trong khi
# thật ra là 13 byte, fast-import lệch một byte rồi chết với một thông điệp chỉ vào
# dòng hoàn toàn khác ("unsupported command: ommit ..."). Tôi vừa mắc đúng lỗi đó ở
# lần chạy đầu của script này. Hàm dưới đo bằng `wc -c` trên đúng chuỗi byte sắp ghi.
emit_data() {
  local payload="$1"
  local tmp="$DEST/.emit.bin"
  printf '%s\n' "$payload" >"$tmp"
  printf 'data %s\n' "$(wc -c <"$tmp" | tr -d ' ')"
  cat "$tmp"
  rm -f "$tmp"
}

{
  printf 'blob\nmark :1\n'
  emit_data 'noi dung'
  printf 'commit refs/heads/rawname\nmark :2\n'
  printf 'author %s <%s> %d +0000\n' "$GIT_AUTHOR_NAME" "$GIT_AUTHOR_EMAIL" "$((EPOCH_BASE + 120))"
  printf 'committer %s <%s> %d +0000\n' "$GIT_COMMITTER_NAME" "$GIT_COMMITTER_EMAIL" "$((EPOCH_BASE + 120))"
  emit_data 'ten tep byte Latin-1'
  printf 'M 100644 :1 "b_caf\\351.txt"\n\n'
} >"$STREAM"

git -C "$REPO" fast-import --quiet <"$STREAM"
rm -f "$STREAM"

# Lấy tệp ra đĩa rồi gỡ khỏi index -> tệp chưa theo dõi mang byte thô trong tên.
#
# 🔴 KHÔNG truyền đường dẫn byte thô làm PATHSPEC qua shell.
#
# Đã đo ở lần chạy trước của chính script này: `git checkout rawname -- "b_caf\351.txt"`
# thất bại với `pathspec 'b_café.txt' did not match any file(s)` — shell đã chuyển
# \351 thành UTF-8 (\303\251) TRƯỚC khi git thấy nó, đúng cùng lớp lỗi mà mục non-utf8
# của make-fixtures.sh ghi lại. Blob nằm đúng trong tree (`git ls-tree` in
# `"b_caf\351.txt"`), chỉ có đường vào qua đối số dòng lệnh là hỏng.
#
# Cách đi vòng: `checkout-index -a` ghi MỌI tệp của index ra đĩa mà không cần ai gõ
# tên. Nạp index từ nhánh `rawname` bằng `read-tree`, ghi ra, rồi `read-tree` ngược về
# HEAD để index sạch lại — tệp byte thô ở lại đĩa ở dạng CHƯA THEO DÕI, đúng thứ cần.
RAW_OK=0
if git -C "$REPO" read-tree rawname 2>/dev/null \
  && git -C "$REPO" checkout-index -a 2>/dev/null; then
  RAW_OK=1
fi
# Index phải trở về đúng HEAD dù nhánh phụ có nạp được hay không: một index còn sót
# nội dung của `rawname` sẽ làm mọi bản ghi phía sau sai, và sai IM LẶNG.
git -C "$REPO" read-tree HEAD

# ---------------------------------------------------------------------------
# 3. Đổi tên: `a_old.txt` -> `a_new.txt`
#
# 🔴 `a_new.txt` sort ĐẦU. Xem khối ràng buộc ở đầu tệp trước khi đổi tên này.
# ---------------------------------------------------------------------------
echo "[3] đổi tên a_old.txt -> a_new.txt (sort ĐẦU, để lại >= 2 bản ghi phía sau)"
git -C "$REPO" mv a_old.txt a_new.txt

# ---------------------------------------------------------------------------
# 4. Hai tệp sửa đã stage — chính là hai bản ghi ĐỨNG SAU bản ghi dạng `2`
# ---------------------------------------------------------------------------
echo "[4] hai tệp sửa đã stage: m_one.txt, n_two.txt"
printf 'c\n' >>"$REPO/m_one.txt"
git -C "$REPO" add -- m_one.txt
printf 'c\n' >>"$REPO/n_two.txt"
git -C "$REPO" add -- n_two.txt

# ---------------------------------------------------------------------------
# 5. Tệp chưa theo dõi thường + tệp chưa theo dõi có KHOẢNG TRẮNG trong tên
#
# Khoảng trắng là một trong hai lý do `-z` tồn tại (lý do kia là byte không UTF-8 ở
# mục 2). Đường dẫn là trường CUỐI của bản ghi, nên phép tách trường theo khoảng trắng
# phải có GIỚI HẠN số lần, nếu không tên này bị cắt ở khoảng trắng đầu tiên.
# ---------------------------------------------------------------------------
echo "[5] tệp chưa theo dõi, một tệp có khoảng trắng trong tên"
printf 'u\n' >"$REPO/z_untracked.txt"
printf 'ws\n' >"$REPO/z with space.txt"

# ---------------------------------------------------------------------------
# 6. Nhóm CHƯA STAGE — thiếu ở bản đầu của fixture này (phát hiện ở plan 04-02)
#
# # 🔴 Vì sao mục này được thêm vào sau
#
# Bản đầu của fixture (plan 04-01) `git add` sau **mỗi** lần sửa, nên **mọi** bản ghi
# theo dõi đều là `R.` hoặc `M.` — tức nhóm `Unstaged` **hoàn toàn vắng mặt**. Test
# đơn vị của 04-01 không thấy điều đó vì chúng chạy trên byte tự dựng, nơi ca `.M` có
# test riêng; chỉ khi plan 04-02 chạy `get_status` **thật** trên fixture này và đòi đủ
# **ba** nhóm thì khoảng trống mới lộ ra.
#
# Đây đúng lớp "fixture có hình dạng đúng cho câu hỏi cũ nhưng không phủ câu hỏi mới"
# — họ hàng với lỗi #7 của CONTEXT.md mục 3.1.
#
# # Vì sao sửa bằng `p_mm.txt` chứ không bằng cách bỏ một `git add` ở mục 4
#
# Hai bản ghi của mục 4 (`m_one.txt`, `n_two.txt`) là **chính** hai bản ghi đứng sau
# bản ghi dạng `2`, và phép kiểm chống lệch nấc ở cuối script đếm chúng. Bỏ `git add`
# của một trong hai làm bản ghi đó thành `.M` — vẫn là một bản ghi, nên phép đếm vẫn
# qua — nhưng nó **đổi dữ liệu của ca kiểm chịu lực nhất** của plan 04-01, thứ vừa
# phải sửa một lần vì không phân biệt được đột biến M1. Thêm một tệp mới thì không
# đụng gì tới nó.
#
# `p_mm.txt` sắp xếp sau `n_two.txt` và trước `z*`, nên nó cũng nằm **sau** bản ghi
# dạng `2` và làm số bản ghi phía sau tăng chứ không giảm.
#
# Tệp này có XY = `MM`: sửa rồi stage, rồi sửa tiếp. Nó phủ **hai** thứ trong một bản
# ghi — nhóm `Unstaged` đang thiếu, và ca "một tệp sinh HAI phần tử" mà
# `RepoStatus::wip_counts` phải đếm là **một** (đột biến M8 của plan 04-01).
# ---------------------------------------------------------------------------
#
# 🔴 KHÔNG `git commit` ở đây. `p_mm.txt` đã nằm ở commit nền (mục 1) vì lúc này bản
# ghi đổi tên của mục 3 đã ở trong index, và một commit thêm sẽ commit luôn nó — phá
# toàn bộ trạng thái bẩn mà fixture này tồn tại để mang.
echo "[6] tệp MM: đã stage một lần rồi sửa tiếp — cung cấp nhóm CHƯA STAGE"
printf 'da stage\n' >>"$REPO/p_mm.txt"
git -C "$REPO" add -- p_mm.txt
printf 'chua stage\n' >>"$REPO/p_mm.txt"

# ---------------------------------------------------------------------------
# Kiểm chứng NGAY tại đây: fixture có thật sự phân biệt được lệch nấc không?
#
# Đây là cổng chống lỗi #4 của CONTEXT.md mục 3.1 ("fixture không phân biệt được đột
# biến"). Script tự khẳng định tiền đề của mình: nếu KHÔNG tìm thấy bản ghi dạng `2`,
# hoặc sau nó có ÍT HƠN 2 bản ghi, thì script THẤT BẠI — không phải in cảnh báo rồi
# thoát 0. Một fixture âm thầm mất tính phân biệt là fixture tệ hơn không có.
# ---------------------------------------------------------------------------
BANG_GHI="$DEST/records.txt"
git -C "$REPO" status --porcelain=v2 --branch -z --untracked-files=all \
  | tr '\0' '\n' >"$BANG_GHI"

# Chỉ số dòng của bản ghi dạng `2` trong đầu ra đã đổi NUL thành newline.
DONG_2="$(grep -n '^2 ' "$BANG_GHI" | head -1 | cut -d: -f1 || true)"

if [ -z "$DONG_2" ]; then
  echo "LỖI: không thấy bản ghi dạng '2' (đổi tên) trong đầu ra status." >&2
  echo "      Fixture này tồn tại ĐỂ có bản ghi đó; thiếu nó thì test lệch nấc vô dụng." >&2
  echo "      Có thể git không phát hiện đổi tên (kiểm cấu hình status.renames)." >&2
  exit 1
fi

# Đường dẫn cũ của bản ghi dạng `2` nằm ở dòng ngay sau nó (đoạn NUL thứ hai), nên
# các bản ghi THẬT đứng sau bắt đầu từ dòng DONG_2 + 2.
TONG_DONG="$(wc -l <"$BANG_GHI" | tr -d ' ')"
SAU="$((TONG_DONG - DONG_2 - 1))"

if [ "$SAU" -lt 2 ]; then
  echo "LỖI: chỉ có $SAU bản ghi sau bản ghi dạng '2' — cần >= 2." >&2
  echo "      Với < 2 bản ghi theo sau, một bộ phân tích tách NUL trơn (lệch nấc) vẫn" >&2
  echo "      cho kết quả giống bộ phân tích đúng, nên test KHÔNG phân biệt được gì." >&2
  echo "      Nguyên nhân thường gặp: đổi tên đích thành một tên sort muộn." >&2
  exit 1
fi

# ---------------------------------------------------------------------------
# Kiểm chứng thứ hai: fixture có ĐỦ BA NHÓM không? (thêm ở plan 04-02)
#
# # 🔴 Vì sao phép kiểm này tồn tại
#
# Bản đầu của fixture có **hình dạng đúng** cho câu hỏi của plan 04-01 (một bản ghi
# dạng `2` đứng trước >= 2 bản ghi khác) nhưng **không có nhóm `Unstaged` nào** — nó
# `git add` sau mỗi lần sửa. Test đơn vị của 04-01 không thấy vì chúng chạy trên byte
# tự dựng; khoảng trống chỉ lộ ra khi plan 04-02 chạy `get_status` THẬT trên fixture
# và đòi cả ba nhóm.
#
# Nên phép kiểm này **khẳng định tiền đề của chính fixture**: một fixture mang tên
# "status-cases" mà thiếu một trong ba nhóm của WORK-01 là một fixture âm thầm không
# phủ được thứ nó hứa. Thất bại ở đây là ĐÚNG — script thoát 1, không in cảnh báo rồi
# thoát 0.
#
# Phân nhóm đọc thẳng từ XY của porcelain v2 (ký tự sau chữ `1`/`2` là XY):
#   - `?` ở đầu dòng           -> chưa theo dõi
#   - XY[0] khác `.`           -> đã stage
#   - XY[1] khác `.`           -> chưa stage
# ---------------------------------------------------------------------------
CO_STAGED="$(awk '/^[12] [^.]/ {print; exit}' "$BANG_GHI" || true)"
CO_UNSTAGED="$(awk '/^[12] .[^.]/ {print; exit}' "$BANG_GHI" || true)"
CO_UNTRACKED="$(grep -c '^? ' "$BANG_GHI" || true)"

THIEU=""
[ -z "$CO_STAGED" ] && THIEU="$THIEU staged"
[ -z "$CO_UNSTAGED" ] && THIEU="$THIEU unstaged"
[ "${CO_UNTRACKED:-0}" -eq 0 ] && THIEU="$THIEU untracked"

if [ -n "$THIEU" ]; then
  echo "LỖI: fixture thiếu nhóm:$THIEU" >&2
  echo "      WORK-01 có BA nhóm; một fixture tên 'status-cases' phải mang cả ba," >&2
  echo "      nếu không test 'get_status cho cả ba nhóm' không phủ được thứ nó hứa." >&2
  echo "      Bản ghi đọc được:" >&2
  sed 's/^/        /' "$BANG_GHI" >&2
  exit 1
fi
echo "    ba nhóm (đã stage / chưa stage / chưa theo dõi): ĐỦ"

# Byte thô trong tên tệp: ĐO, không giả định. Kết quả khác nhau theo hệ thống tệp.
#
# Đã đo trên máy này (Windows 11, NTFS, git 2.54.0.windows.1): blob vào tree ĐÚNG với
# byte thô — `git ls-tree` in `"b_caf\351.txt"` — nhưng khi ghi ra thư mục làm việc,
# NTFS lưu tên tệp dạng UTF-16 và không đựng được một byte \351 đơn lẻ, nên git đọc
# ngược lại thành UTF-8 `\303\251`. Xác nhận bằng `od -c` trên đầu ra status:
# `?   b   _   c   a   f 303 251   .   t   x   t  \0`.
#
# Nghĩa là: trên Windows, một tên tệp CHƯA THEO DÕI không phải UTF-8 là KHÔNG dựng
# được — giới hạn của hệ thống tệp, không phải lỗi script. Trên ext4/APFS thì dựng
# được và biến này sẽ thành "có".
#
# Vì vậy cổng non-UTF-8 THẬT SỰ của plan này là test BYTE-LITERAL trong
# parsers/status.rs, vốn chạy giống nhau trên mọi nền tảng. Fixture này chỉ xác nhận
# thêm khi nền tảng cho phép. Ghi rõ ra đây để không ai sau này đọc dòng "KHÔNG" rồi
# tưởng fixture hỏng.
if [ "$RAW_OK" -eq 1 ] && git -C "$REPO" status --porcelain=v2 -z --untracked-files=all \
    | xxd -p | tr -d '\n' | grep -q 'e9'; then
  RAW_NOTE="CÓ — byte \\xe9 thô nằm trong tên tệp chưa theo dõi"
else
  RAW_NOTE="KHÔNG — hệ thống tệp của máy này không đựng được byte thô trong tên tệp (ca thường trên Windows/NTFS)"
fi

echo
echo "=== Đầu ra status (NUL đã đổi thành newline) ==="
grep -n . "$BANG_GHI"
echo
echo "Bản ghi dạng '2' ở dòng $DONG_2; sau nó còn $SAU bản ghi (cần >= 2). ĐẠT."
echo "Tên tệp byte thô: $RAW_NOTE"
echo
echo "Repo: $REPO"
echo "Đầu ra đã ghi vào: $BANG_GHI"
echo
echo "Test Rust tìm repo này qua git_plum_lib::testing::require_status_fixture()."
echo "KHÔNG chạy git commit/reset/clean trong repo đó: giá trị của nó là trạng thái BẨN."

# ===========================================================================
# Repo mẫu có HOOK TỪ CHỐI — plan 04-03, WORK-08
# ===========================================================================
#
# # Vì sao hai repo riêng chứ không thêm hook vào `repo`
#
# `repo` ở trên là fixture CHỈ ĐỌC: giá trị của nó nằm ở thư mục làm việc bẩn, và
# `require_status_fixture` ghi rõ rằng chạy `git commit` trong đó là PHÁ fixture. Test
# hook PHẢI chạy `git commit` thật — đó là cả điểm. Nên chúng cần repo riêng.
#
# # 🔴 Hook phải CHẠY ĐƯỢC, và script tự CHỨNG MINH điều đó
#
# Một hook không chạy được cho test XANH mà chẳng kiểm gì — đúng khuôn cổng #4 của
# CONTEXT.md mục 3.1 (fixture không phân biệt được đột biến). Trên Windows đặc biệt dễ
# mắc: không có bit executable của POSIX, git chạy hook qua shell đi kèm (`sh.exe` của
# Git for Windows), và một hook thiếu `#!/bin/sh` hoặc có CRLF sẽ im lặng không chạy.
#
# Vì vậy mỗi repo hook dưới đây kết thúc bằng một phép ĐO: chạy `git commit` thật rồi
# khẳng định (a) nó THẤT BẠI và (b) chuỗi nhận diện của hook CÓ trong đầu ra. Thiếu
# một trong hai thì script THOÁT 1 — không in cảnh báo rồi thoát 0.
#
# Chuỗi nhận diện là duy nhất và dễ grep, để test Rust khẳng định NGUYÊN VĂN đầu ra
# hook đi tới người dùng, chứ không chỉ khẳng định "có lỗi" (đột biến M5).

readonly DAU_HIEU_PRE_COMMIT="HOOK-PRE-COMMIT-REJECTED"
readonly DAU_HIEU_COMMIT_MSG="HOOK-COMMIT-MSG-REJECTED"

# Dựng một repo có đúng một hook từ chối.
#   $1 = tên thư mục repo dưới $DEST
#   $2 = tên hook (`pre-commit` hoặc `commit-msg`)
#   $3 = chuỗi nhận diện hook in ra
dung_repo_hook() {
  local ten="$1" ten_hook="$2" dau_hieu="$3"
  local r="$DEST/$ten"

  xoa_that "$r"
  mkdir -p "$r"

  git init --initial-branch=main --quiet "$r"
  git -C "$r" config core.autocrlf false
  git -C "$r" config commit.gpgsign false
  git -C "$r" config user.name "$GIT_AUTHOR_NAME"
  git -C "$r" config user.email "$GIT_AUTHOR_EMAIL"

  # Commit nền: `--amend` cần một commit để sửa, và ca "không có gì để commit" cần
  # một HEAD phân giải được. Repo chưa có commit nào là một ca khác hẳn.
  printf 'nen\n' >"$r/nen.txt"
  git -C "$r" add -- nen.txt
  git -C "$r" commit --quiet -m "commit nen"

  # 🔴 Hook ghi ra CẢ stdout LẪN stderr.
  #
  # git chuyển tiếp cả hai luồng của hook, nhưng chúng đi tới hai chỗ khác nhau trong
  # `GitOutput`. Một hook chỉ ghi stderr cho một test chỉ đọc stderr XANH mà không
  # chứng minh gì về đường stdout, và ngược lại. Cho hook ghi cả hai thì test khẳng
  # định được rằng phép phân loại lỗi đọc CẢ HAI — thiếu một luồng là một cách rất
  # thật để nuốt mất thông báo của hook.
  local hook="$r/.git/hooks/$ten_hook"
  printf '#!/bin/sh\necho "%s (stdout)"\necho "%s (stderr)" >&2\nexit 1\n' \
    "$dau_hieu" "$dau_hieu" >"$hook"
  chmod +x "$hook"

  # Tệp đã stage để `git commit` có thứ để commit — nếu không, lệnh thất bại vì
  # "nothing to commit" TRƯỚC khi hook chạy, và phép đo dưới đây sẽ đo nhầm thứ.
  printf 'de commit\n' >"$r/them.txt"
  git -C "$r" add -- them.txt

  # -----------------------------------------------------------------------
  # PHÉP ĐO: hook có THẬT SỰ chạy không?
  # -----------------------------------------------------------------------
  local truoc sau ra ma
  truoc="$(git -C "$r" rev-parse HEAD)"

  # `|| true` vì `set -e` sẽ giết script ở một lệnh thất bại có chủ ý.
  ra="$(git -C "$r" commit -m "phep do hook" 2>&1 || true)"
  ma="$(git -C "$r" rev-parse HEAD)"

  if ! printf '%s' "$ra" | grep -q "$dau_hieu"; then
    echo "LỖI: hook '$ten_hook' của repo '$ten' KHÔNG CHẠY." >&2
    echo "      Không thấy chuỗi nhận diện '$dau_hieu' trong đầu ra của git commit." >&2
    echo "      Một hook không chạy được làm MỌI test hook xanh mà chẳng kiểm gì —" >&2
    echo "      đúng khuôn cổng #4 của CONTEXT.md 3.1." >&2
    echo "      Nguyên nhân thường gặp trên Windows: thiếu '#!/bin/sh', tệp có CRLF," >&2
    echo "      hoặc core.hooksPath trỏ đi chỗ khác." >&2
    echo "      Đầu ra đọc được:" >&2
    printf '%s\n' "$ra" | sed 's/^/        /' >&2
    exit 1
  fi

  if [ "$ma" != "$truoc" ]; then
    echo "LỖI: hook '$ten_hook' in ra chuỗi nhận diện NHƯNG commit vẫn được tạo." >&2
    echo "      HEAD đổi $truoc -> $ma. Hook phải THOÁT KHÁC 0 để chặn commit;" >&2
    echo "      một hook chỉ in chữ rồi exit 0 không chặn gì cả." >&2
    exit 1
  fi

  # Tệp phải VẪN CÒN stage sau khi hook từ chối — `<hook_behavior>` của plan 04-03
  # khẳng định điều này, và test Rust dựa vào nó để kiểm "noVerify=true thì thành công"
  # trên CÙNG repo ngay sau đó.
  if ! git -C "$r" diff --cached --quiet -- them.txt; then
    : # có thay đổi đã stage — đúng như mong đợi
  else
    echo "LỖI: sau khi hook từ chối, 'them.txt' không còn ở index." >&2
    echo "      Test 'cùng repo: noVerify=true thì commit thành công' sẽ không có gì" >&2
    echo "      để commit, và nó sẽ xanh/đỏ vì một lý do khác hẳn." >&2
    exit 1
  fi

  echo "    $ten: hook '$ten_hook' CHẠY, chặn commit, index giữ nguyên. ĐẠT."
  echo "      đầu ra hook: $(printf '%s' "$ra" | grep "$dau_hieu" | head -2 | tr '\n' ' ')"
}

echo
echo "[7] repo mẫu có hook từ chối"
dung_repo_hook "hook-reject" "pre-commit" "$DAU_HIEU_PRE_COMMIT"
dung_repo_hook "hook-msg-reject" "commit-msg" "$DAU_HIEU_COMMIT_MSG"

echo
echo "Repo hook: $DEST/hook-reject (pre-commit), $DEST/hook-msg-reject (commit-msg)"
echo "Test Rust tìm chúng qua git_plum_lib::testing::require_hook_fixture(<tên>)."
