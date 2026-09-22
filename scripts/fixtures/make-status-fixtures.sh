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
# 1. Commit nền — ba tệp theo dõi
# ---------------------------------------------------------------------------
echo "[1] commit nền"
printf 'aaa\n' >"$REPO/a_old.txt"
printf 'z\n' >"$REPO/m_one.txt"
printf 'z\n' >"$REPO/n_two.txt"
git -C "$REPO" add -- a_old.txt m_one.txt n_two.txt
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
