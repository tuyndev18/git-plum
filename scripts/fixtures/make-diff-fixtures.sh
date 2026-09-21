#!/usr/bin/env bash
#
# Sinh repo mẫu cho trình xem diff của Phase 3 — DIFF-01, DIFF-06.
#
# Khác make-fixtures.sh (Phase 2): ở đó mỗi repo mang một hình dạng ĐỒ THỊ bệnh lý.
# Ở đây một repo duy nhất mang nhiều hình dạng TỆP bệnh lý, mỗi hình một commit, vì
# thứ đang được kiểm là cách đọc bản vá của một tệp chứ không phải hình học lịch sử.
#
# TÍNH TẤT ĐỊNH: ghim cả sáu biến GIT_AUTHOR_*/GIT_COMMITTER_* và core.autocrlf=false,
# cùng lý do với make-fixtures.sh. Nhưng LƯU Ý: ở đây SHA vẫn không hoàn toàn cố định
# giữa các lần chạy vì repo `crlf` cố tình đổi autocrlf, nên script IN RA SHA của từng
# commit và test đọc SHA từ git chứ không dán cứng vào mã.
#
# Cách dùng:
#   bash scripts/fixtures/make-diff-fixtures.sh [thư-mục-đích]
# Mặc định: <gốc-dự-án>/target/fixtures/diff-cases

set -euo pipefail

# ---------------------------------------------------------------------------
# Đường dẫn và cổng an toàn — sao lại từ make-fixtures.sh, cùng lý do
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEST_ARG="${1:-$PROJECT_ROOT/target/fixtures/diff-cases}"

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
TICK=0

next_stamp() {
  TICK=$((TICK + 1))
  local ts=$((EPOCH_BASE + TICK * 60))
  export GIT_AUTHOR_DATE="$ts +0000"
  export GIT_COMMITTER_DATE="$ts +0000"
}

# Xoá và KIỂM đã xoá được thật. Trên Windows, OneDrive/trình diệt virus/một cửa sổ
# Explorer đang mở đều làm `rm -rf` thất bại MỘT PHẦN rồi trả mã 0. Chạy tiếp trên
# thư mục còn sót sinh repo hỏng với thông điệp vô nghĩa ở tận nơi khác.
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
git -C "$REPO" config user.name "$GIT_AUTHOR_NAME"
git -C "$REPO" config user.email "$GIT_AUTHOR_EMAIL"

# Bảng SHA in ra cuối script: `<nhãn> <sha> <ghi chú>`.
SHA_TABLE=""

# Commit mọi thứ đang chờ rồi ghi SHA vào bảng dưới một nhãn ổn định.
#
# Test tra SHA theo NHÃN, không theo thứ tự — chèn thêm một commit ở giữa về sau sẽ
# không làm mọi test sau nó trỏ nhầm commit.
ghi_commit() {
  local nhan="$1" msg="$2" ghi_chu="${3:-}"
  next_stamp
  git -C "$REPO" commit --quiet -m "$msg"
  local sha
  sha="$(git -C "$REPO" rev-parse HEAD)"
  SHA_TABLE="${SHA_TABLE}${nhan} ${sha} ${ghi_chu}"$'\n'
}

echo "Sinh repo mẫu diff vào: $REPO"
echo

# ---------------------------------------------------------------------------
# 1. Commit GỐC — ca EMPTY_TREE
#
# Commit gốc không có `<sha>^`, nên `git diff <sha>^ <sha>` thất bại với
# `unknown revision`. Cài đặt chỉ nối `^` sẽ vỡ ở commit đầu tiên của MỌI repo.
# ---------------------------------------------------------------------------
echo "[1] commit gốc (ca EMPTY_TREE)"
printf 'dong mot\ndong hai\ndong ba\n' >"$REPO/text-simple.txt"
printf 'se bi xoa\n' >"$REPO/deleted.txt"
git -C "$REPO" add -- text-simple.txt deleted.txt
ghi_commit root "commit goc them tep" "tệp mới thêm, mọi dòng là added"

# ---------------------------------------------------------------------------
# 2. text-simple.txt — ca thường
# ---------------------------------------------------------------------------
echo "[2] text-simple.txt: sửa thường"
printf 'dong mot\ndong hai da sua\ndong ba\ndong bon moi\n' >"$REPO/text-simple.txt"
git -C "$REPO" add -- text-simple.txt
ghi_commit text_simple "sua text-simple" "ca thường: một hunk, có added và removed"

# ---------------------------------------------------------------------------
# 3. binary.png — ca Binary
#
# Byte 0x00 thật trong nội dung. `git diff --numstat` in `-\t-` cho tệp nhị phân;
# đó là phán quyết của CHÍNH GIT, rẻ hơn và đúng hơn phép đoán "có byte 0 trong
# 8000 byte đầu" tự viết.
# ---------------------------------------------------------------------------
echo "[3] binary.png: tệp nhị phân"
printf '\211PNG\r\n\032\n\000\000\000\015IHDR\000\000\001\000' >"$REPO/binary.png"
printf '\000\001\002\003\004\005\006\007' >>"$REPO/binary.png"
git -C "$REPO" add -- binary.png
ghi_commit binary_add "them binary.png" "tệp nhị phân mới"

printf '\211PNG\r\n\032\n\000\000\000\015IHDR\000\000\002\000' >"$REPO/binary.png"
printf '\377\376\375\374\373\372\371\370\367\366' >>"$REPO/binary.png"
git -C "$REPO" add -- binary.png
ghi_commit binary_mod "sua binary.png" "ca Binary: hai phía đều khác 0 byte"

# ---------------------------------------------------------------------------
# 4. large.txt — ca TooLarge, > 5 MB
#
# 5 MB là MAX_DIFF_BLOB_BYTES. Sinh bằng vòng lặp `yes | head` chứ không commit một
# tệp 5 MB vào repo git-plum — nó nằm dưới target/, đã trong .gitignore.
# ---------------------------------------------------------------------------
echo "[4] large.txt: > 5 MB (ca TooLarge)"
# 100 byte mỗi dòng × 60000 dòng ≈ 6.0 MB, dư trên ngưỡng 5 MB (5242880 byte).
DONG_LON="$(printf 'x%.0s' $(seq 1 99))"
yes "$DONG_LON" 2>/dev/null | head -n 60000 >"$REPO/large.txt" || true
git -C "$REPO" add -- large.txt
ghi_commit large_add "them large.txt" "tệp > 5 MB, cổng kích thước phải chặn TRƯỚC git diff"

# Sửa đúng một dòng: nếu cổng kích thước chạy SAU `git diff` thì bản vá vẫn nhỏ và
# một cài đặt "lọc sau" vẫn trả kết quả trông đúng — đó là lý do phải kiểm bằng
# CommandLog chứ không bằng giá trị trả về.
printf 'DONG DAU DA SUA\n' >"$REPO/large.txt.tmp"
tail -n +2 "$REPO/large.txt" >>"$REPO/large.txt.tmp"
mv "$REPO/large.txt.tmp" "$REPO/large.txt"
git -C "$REPO" add -- large.txt
ghi_commit large_mod "sua mot dong trong large.txt" "bản vá NHỎ nhưng blob LỚN"

# ---------------------------------------------------------------------------
# 5. pointer.bin — con trỏ Git LFS
#
# KHÔNG cài git-lfs và KHÔNG tạo .gitattributes — CÓ CHỦ Ý.
#
# Đây chính là ca mà `git check-attr -z filter -- pointer.bin` trả `unspecified` cho
# một con trỏ LFS THẬT. `check-attr` trả lời "repo này có CẤU HÌNH lfs cho path này
# không", không trả lời "blob này CÓ PHẢI con trỏ lfs không" — hai câu khác nhau, và
# ta đang hỏi câu thứ hai. Vì vậy nhận biết bằng NỘI DUNG blob.
#
# `size 1048576` là kích thước tệp THẬT mà con trỏ đại diện (1 MB); bản thân tệp con
# trỏ chỉ ~130 byte. Lẫn hai số này là lỗi dễ mắc nhất ở cổng LFS.
# ---------------------------------------------------------------------------
echo "[5] pointer.bin: con trỏ LFS (không .gitattributes — có chủ ý)"
{
  printf 'version https://git-lfs.github.com/spec/v1\n'
  printf 'oid sha256:4d7a214614ab2935c943f9e0ff69d22eadbb8f32b1258daaa5e2ca24d17e2393\n'
  printf 'size 1048576\n'
} >"$REPO/pointer.bin"
git -C "$REPO" add -- pointer.bin
ghi_commit lfs "them con tro lfs" "oid và size đọc TỪ NỘI DUNG, không phải kích thước tệp con trỏ"

# ---------------------------------------------------------------------------
# 6. crlf.txt — nội dung CRLF
#
# git in nội dung nguyên vẹn, tức mỗi dòng diff kết thúc bằng \r\n. Không cắt \r thì
# MỌI DiffLine.content mang một ký tự vô hình ở cuối.
# ---------------------------------------------------------------------------
echo "[6] crlf.txt: nội dung CRLF"
printf 'alpha\r\nbeta\r\ngamma\r\n' >"$REPO/crlf.txt"
git -C "$REPO" add -- crlf.txt
ghi_commit crlf_add "them crlf.txt" "tệp CRLF mới"

printf 'alpha\r\nBETA DA SUA\r\ngamma\r\n' >"$REPO/crlf.txt"
git -C "$REPO" add -- crlf.txt
ghi_commit crlf_mod "sua crlf.txt" "ca CRLF: content KHÔNG được kết thúc bằng \\r"

# ---------------------------------------------------------------------------
# 7. no-eol.txt — không có dòng trống cuối
#
# Sinh dòng `\ No newline at end of file`. Dòng đó KHÔNG phải một DiffLine; nó đặt
# cờ lên dòng ngay trước nó. Đọc nó thành dòng nội dung thêm một dòng giả vào diff.
# ---------------------------------------------------------------------------
echo "[7] no-eol.txt: thiếu dòng trống cuối"
printf 'mot\nhai\nba' >"$REPO/no-eol.txt" # không có \n cuối
git -C "$REPO" add -- no-eol.txt
ghi_commit noeol_add "them no-eol.txt" "tệp không kết thúc bằng newline"

printf 'mot\nhai\nBA DA SUA' >"$REPO/no-eol.txt"
git -C "$REPO" add -- no-eol.txt
ghi_commit noeol_mod "sua dong cuoi no-eol.txt" "cả hai phía đều có \\ No newline"

# ---------------------------------------------------------------------------
# 8. renamed.txt -> renamed-new.txt, VỪA đổi tên VỪA sửa
#
# Một commit chỉ đổi tên cho R100; ta phải có R với điểm tương đồng < 100 để kiểm
# rằng status là chuỗi (`R87`) chứ không phải ký tự, và rằng vẫn có hunk.
# ---------------------------------------------------------------------------
echo "[8] renamed.txt -> renamed-new.txt: vừa đổi tên vừa sửa"
printf 'alpha\nbeta\ngamma\ndelta\nepsilon\nzeta\neta\ntheta\n' >"$REPO/renamed.txt"
git -C "$REPO" add -- renamed.txt
ghi_commit rename_add "them renamed.txt" "tệp trước khi đổi tên"

git -C "$REPO" mv renamed.txt renamed-new.txt
printf 'alpha\nBETA DA SUA\ngamma\ndelta\nepsilon\nzeta\neta\ntheta\n' >"$REPO/renamed-new.txt"
git -C "$REPO" add -- renamed-new.txt
ghi_commit rename_mod "doi ten va sua renamed.txt" "R với điểm tương đồng < 100"

# ---------------------------------------------------------------------------
# 9. Tên tệp không UTF-8 — cùng cách make-fixtures.sh dùng cho repo `non-utf8`
#
# Tên tệp trên Linux là byte tuỳ ý. Một bộ phân tích gọi String::from_utf8().unwrap()
# panic ở đây.
# ---------------------------------------------------------------------------
echo "[9] tên tệp không UTF-8"
TEN_XAU="$(printf 'ten-\xff-xau.txt')"
printf 'noi dung mot\nnoi dung hai\n' >"$REPO/$TEN_XAU"
git -C "$REPO" add -- "$TEN_XAU"
ghi_commit nonutf8_add "them tep ten khong utf8" "tên tệp chứa byte 0xFF"

printf 'noi dung mot\nnoi dung hai DA SUA\n' >"$REPO/$TEN_XAU"
git -C "$REPO" add -- "$TEN_XAU"
ghi_commit nonutf8_mod "sua tep ten khong utf8" "diff trên tệp tên byte lạ"

# ---------------------------------------------------------------------------
# 10. Xoá một tệp — ca mọi dòng là removed
# ---------------------------------------------------------------------------
echo "[10] xoá deleted.txt"
git -C "$REPO" rm --quiet -- deleted.txt
ghi_commit deleted "xoa deleted.txt" "mọi dòng phải là removed, KHÔNG phải lỗi"

# ---------------------------------------------------------------------------
# 11. empty-line.txt — DÒNG TRỐNG ngay trước dòng sửa
#
# Phục vụ 03-03 (diff mức từ). Sinh ở ĐÂY vì 03-03 không có make-diff-fixtures.sh
# trong files_modified của nó.
#
# Vì sao cần: quy tắc đẩy WordLine ở dấu `~` không được dùng `len() > 0` để quyết
# định có đẩy hay không. Một dòng TRỐNG có len() == 0 nhưng vẫn là một dòng thật.
# ---------------------------------------------------------------------------
echo "[11] empty-line.txt: dòng trống trước dòng sửa"
printf 'header\n\nfunc a() {\n  x == 1\n}\n' >"$REPO/empty-line.txt"
git -C "$REPO" add -- empty-line.txt
ghi_commit emptyline_add "them empty-line.txt" "có dòng trống ở dòng 2"

printf 'header\n\nfunc a() {\n  x === 1\n}\n' >"$REPO/empty-line.txt"
git -C "$REPO" add -- empty-line.txt
ghi_commit emptyline_mod "sua == thanh === trong empty-line.txt" "dòng trống ĐỨNG TRƯỚC dòng sửa"

# ---------------------------------------------------------------------------
# 12. dup-lines.txt — HAI DÒNG NỘI DUNG TRÙNG NHAU trong cùng một hunk
#
# Cần cho mutation #9 của 03-03 (khớp theo nội dung thay vì theo số dòng): không có
# hai dòng giống hệt nhau trong cùng hunk thì mutation đó cho 0 test đỏ, và cổng trở
# thành vô dụng mà không ai biết.
# ---------------------------------------------------------------------------
echo "[12] dup-lines.txt: hai dòng trùng nhau trong một hunk"
printf 'start\nTRUNG\nmid\nTRUNG\nend\n' >"$REPO/dup-lines.txt"
git -C "$REPO" add -- dup-lines.txt
ghi_commit dup_add "them dup-lines.txt" "hai dòng TRUNG giống hệt nhau"

printf 'start\nTRUNG\nmid\nTRUNG DA SUA\nend\n' >"$REPO/dup-lines.txt"
git -C "$REPO" add -- dup-lines.txt
ghi_commit dup_mod "sua dong TRUNG thu hai" "chỉ bản SAU bị sửa — khớp theo nội dung sẽ chọn nhầm bản trước"

# ---------------------------------------------------------------------------
# 13. one-side.txt — dòng chỉ có ở MỘT PHÍA
#
# Một commit XOÁ một dòng và THÊM một dòng khác ở chỗ khác — không phải sửa tại chỗ.
# Loại bỏ quy tắc "luôn đẩy cả hai phía" (03-03 mutation #5): quy tắc đó sinh
# WordLine RỖNG GIẢ ở phía không có dòng.
#
# ⚠️ ĐÃ ĐO: tệp này MỘT MÌNH KHÔNG phân biệt được quy tắc `len() > 0` với cờ
# `touched` — cả hai cho CÙNG nội dung VÀ cùng line_no. Nên nó KHÔNG đủ cho mutation
# #6 của 03-03; xem tệp 14 (mixed.txt).
# ---------------------------------------------------------------------------
echo "[13] one-side.txt: dòng chỉ có ở một phía"
printf 'DELETED_LINE\nkeep2\n' >"$REPO/one-side.txt"
git -C "$REPO" add -- one-side.txt
ghi_commit oneside_add "them one-side.txt" "trước: DELETED_LINE / keep2"

printf 'keep2\nADDED_LINE\n' >"$REPO/one-side.txt"
git -C "$REPO" add -- one-side.txt
ghi_commit oneside_mod "xoa mot dong va them mot dong khac" "KHÔNG đủ để phân biệt len()>0 với touched"

# ---------------------------------------------------------------------------
# 14. 🔴 mixed.txt — DÒNG RỖNG **CỘNG** DÒNG MỘT-PHÍA trong cùng một tệp
#
# Đây là fixture DUY NHẤT phân biệt được cả ba quy tắc đẩy cùng lúc, và là fixture
# mà mutation #6 của 03-03 PHẢI chạy trên. Bảng đo đầy đủ nằm trong 03-02-PLAN.md.
#
# Hình dạng:
#   a / <dòng trống> / DELETED / keep    →    a / <dòng trống> / keep / ADDED
#
# Đầu ra `git diff --word-diff=porcelain` THẬT của nó (đã đo, git 2.54 — dán vào đây
# để người sau không phải đo lại):
#
#   @@ -1,4 +1,4 @@
#    a
#   ~
#    ␠          ← dòng rỗng (một dấu cách, rồi hết dòng)
#   ~
#   -DELETED
#   ~
#    keep
#   ~
#   +ADDED
#   ~
#
# Đọc bảng: dấu `~` đánh dấu kết thúc một dòng nguồn. Quy tắc `len() > 0` bỏ qua
# dòng rỗng (dòng 2) nên nó KHÔNG đẩy gì ở đó; cờ `touched` thì có. Và chỉ khi có
# CẢ dòng một-phía (DELETED/ADDED) thì việc "tăng bộ đếm cho phía nào" mới quan sát
# được. Thiếu một trong hai yếu tố, hai quy tắc sai vẫn cho kết quả đúng.
# ---------------------------------------------------------------------------
echo "[14] mixed.txt: dòng rỗng CỘNG dòng một-phía (fixture duy nhất phân biệt cả ba quy tắc)"
printf 'a\n\nDELETED\nkeep\n' >"$REPO/mixed.txt"
git -C "$REPO" add -- mixed.txt
ghi_commit mixed_add "them mixed.txt" "trước: a / <trống> / DELETED / keep"

printf 'a\n\nkeep\nADDED\n' >"$REPO/mixed.txt"
git -C "$REPO" add -- mixed.txt
ghi_commit mixed_mod "doi DELETED thanh ADDED trong mixed.txt" "🔴 fixture của mutation #6 (03-03)"

# ---------------------------------------------------------------------------
# 15. mode-only.txt — chỉ đổi mode tệp, ca Unchanged
#
# Trên Windows core.filemode thường là false nên commit này CÓ THỂ không đổi gì và
# git từ chối commit rỗng. Dùng `--allow-empty` để script không vỡ; test phải chịu
# được cả hai kết quả và không dán cứng giả định về nó.
# ---------------------------------------------------------------------------
echo "[15] mode-only: chỉ đổi mode (ca Unchanged, phụ thuộc nền tảng)"
printf 'khong doi noi dung\n' >"$REPO/mode-only.txt"
git -C "$REPO" add -- mode-only.txt
ghi_commit modeonly_add "them mode-only.txt" "nội dung cơ sở"

git -C "$REPO" update-index --chmod=+x -- mode-only.txt 2>/dev/null || true
next_stamp
git -C "$REPO" commit --quiet --allow-empty -m "doi mode mode-only.txt"
SHA_TABLE="${SHA_TABLE}modeonly_chmod $(git -C "$REPO" rev-parse HEAD) chỉ đổi mode: git không in hunk nào"$'\n'

# ---------------------------------------------------------------------------
# Bảng SHA — test tra theo NHÃN, không đoán
# ---------------------------------------------------------------------------

BANG="$DEST/shas.txt"
printf '%s' "$SHA_TABLE" >"$BANG"

echo
echo "=== SHA của từng commit (cũng ghi vào $BANG) ==="
printf '%s' "$SHA_TABLE"
echo
echo "Repo: $REPO"
echo "Tổng số commit: $(git -C "$REPO" rev-list --all --count)"
echo
echo "Test Rust đọc bảng này qua git_plum_lib::testing::diff_fixture_sha(\"<nhãn>\")."
echo "Fixture ĐÃ TỪNG BỊ DỌN MẤT một lần (xem 02-07-SUMMARY) — test phải kiểm tồn tại"
echo "và bỏ qua kèm thông báo rõ nếu thiếu, không đỏ mù mờ."
