#!/usr/bin/env bash
#
# Sinh repo mẫu tái hiện lỗi **cạnh hàng WIP bị clamp** — WORK-11, plan 04-04.
#
# ===========================================================================
# Lỗi mà repo này tồn tại để kích hoạt
# ===========================================================================
#
# `git log --all --topo-order` xếp commit mới nhất theo topo của **MỌI** ref ở hàng 0.
# Nếu người dùng đang ở một nhánh **không** phải nhánh mới nhất, HEAD nằm **giữa** danh
# sách và lane của nó là bất kỳ.
#
#   🔴 **Hàng WIP nối xuống HEAD. Nó KHÔNG nối xuống hàng 0.** Hai thứ khác nhau.
#
# `laneX` ở frontend clamp **vô điều kiện** (`Math.min(lane, MAX_VISIBLE_LANES - 1)`),
# nên một HEAD ở lane >= cap làm cạnh WIP trỏ vào cột cuối — tức nối vào **một commit
# khác**. Đây không phải suy giảm nhìn-là-thấy; đây là một đồ thị **vẽ sai một cách tự
# tin**.
#
# ===========================================================================
# 🔴 Số nhánh SUY TỪ CAP, không viết cứng — và đây là chủ ý, không phải cầu kỳ
# ===========================================================================
#
# Bản đầu của recipe dùng 20 nhánh vì cap khi đó là **13**. Cap đổi 13 → 20 ngày
# 2026-09-22, và ở cap 20 thì 20 nhánh **không còn kích hoạt được lỗi** (lane cao nhất
# 19 < cap 20) — bộ test dựng trên nó sẽ xanh một cách **vô nghĩa**.
#
# Đây là lần thứ **hai** cap đổi trong hai ngày. Nên script đọc cap từ
# `src-tauri/src/graph/types.rs` và sinh `CAP + 2` nhánh: lần sau chủ dự án đổi cap,
# fixture vẫn đúng mà không ai phải nhớ sửa.
#
# ===========================================================================
# Chứng minh fixture PHÂN BIỆT ĐƯỢC, không chỉ "đúng hình dạng"
# ===========================================================================
#
# Lỗi cổng #4 và #7 của `CONTEXT.md` 3.1: một fixture đúng *hình dạng* vẫn có thể không
# phân biệt được nếu *dữ liệu* của nó vô hại. Nên script **tự kiểm** ở cuối bằng
# `git log --all --topo-order` + vị trí của HEAD, và **thất bại** nếu HEAD không rơi vào
# lane cao. Kiểm độc lập bằng:
#
#   cargo run --release --bin lanedist target/fixtures/lane-clamp
#
# Nó phải in `HEAD that su: hang N, lane L` với **L >= MAX_VISIBLE_LANES**. Nếu
# `L < CAP` thì fixture **sai** và mọi test dựa trên nó vô nghĩa.
#
# Repo mẫu `wide` đã có **KHÔNG** dùng được: 24 nhánh nhưng HEAD ở `main` và đã merge
# dần, nên `lanedist` in `HEAD that su: hang 0, lane 0`. Đã kiểm — đừng mượn nó.
#
# Cách dùng:
#   bash scripts/fixtures/make-lane-clamp-fixture.sh [thư-mục-đích]
# Mặc định: <gốc-dự-án>/target/fixtures/lane-clamp

set -euo pipefail

# ---------------------------------------------------------------------------
# Đường dẫn và cổng an toàn — sao lại từ make-diff-fixtures.sh, cùng lý do
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEST_ARG="${1:-$PROJECT_ROOT/target/fixtures/lane-clamp}"

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
# 🔴 Đọc cap từ nguồn Rust — KHÔNG viết cứng
# ---------------------------------------------------------------------------

TYPES_RS="$PROJECT_ROOT/src-tauri/src/graph/types.rs"

if [ ! -f "$TYPES_RS" ]; then
  echo "LỖI: không thấy $TYPES_RS — không đọc được MAX_VISIBLE_LANES." >&2
  exit 1
fi

CAP="$(sed -n 's/^pub const MAX_VISIBLE_LANES: u16 = \([0-9]\+\);.*/\1/p' "$TYPES_RS" | head -1)"

# Khẳng định **tiền đề**: không đọc được cap thì dừng, đừng lặng lẽ dùng một mặc định.
# Một mặc định ở đây là đúng lỗi #3 của CONTEXT.md 3.1 (không tìm thấy gì → vẫn chạy
# tiếp → fixture sai mà không ai biết).
if [ -z "$CAP" ] || [ "$CAP" -lt 1 ] 2>/dev/null; then
  echo "LỖI: không đọc được MAX_VISIBLE_LANES từ $TYPES_RS (được: '$CAP')." >&2
  echo "      Khai báo có thể đã đổi dạng. Sửa phép trích ở script này, ĐỪNG dùng số mặc định." >&2
  exit 1
fi

# CAP + 2 nhánh: cần **ít nhất** CAP + 1 lane sống song song để có lane >= CAP; thêm
# một cho biên an toàn khi thuật toán gán lane đổi cách cấp lane.
NHANH=$((CAP + 2))

echo "MAX_VISIBLE_LANES đọc được từ types.rs: $CAP"
echo "Sinh $NHANH nhánh song song (CAP + 2) vào: $DEST"

# ---------------------------------------------------------------------------
# Tính tất định
# ---------------------------------------------------------------------------

export GIT_AUTHOR_NAME="Plum Fixture"
export GIT_AUTHOR_EMAIL="fixture@git-plum.test"
export GIT_COMMITTER_NAME="Plum Fixture"
export GIT_COMMITTER_EMAIL="fixture@git-plum.test"

readonly EPOCH_BASE=1577836800

# ---------------------------------------------------------------------------
# Xoá và KIỂM đã xoá được thật
#
# Trên Windows, OneDrive / trình diệt virus / một cửa sổ Explorer đang mở đều làm
# `rm -rf` thất bại MỘT PHẦN rồi trả mã 0. Chạy tiếp trên một thư mục còn sót là sinh
# một repo lai giữa hai lần chạy — và lỗi đó biểu hiện thành số lane sai.
# ---------------------------------------------------------------------------

rm -rf "$DEST"
if [ -e "$DEST" ]; then
  echo "LỖI: không xoá được '$DEST' (OneDrive/Explorer/AV đang giữ tệp?)." >&2
  exit 1
fi
mkdir -p "$DEST"

cd "$DEST"

git init -q -b main .
git config user.email "$GIT_AUTHOR_EMAIL"
git config user.name "$GIT_AUTHOR_NAME"
git config core.autocrlf false

printf 'base\n' > f.txt
git add -A
GIT_AUTHOR_DATE="$EPOCH_BASE +0000" GIT_COMMITTER_DATE="$EPOCH_BASE +0000" \
  git commit -qm "base"

# ---------------------------------------------------------------------------
# $NHANH nhánh song song, MỖI nhánh một commit, tất cả rẽ từ `main`
#
# 🔴 Ngày committer **tăng dần theo số nhánh**: `--topo-order` dùng ngày để xếp giữa
# các nhánh không có quan hệ tổ tiên. Nhờ vậy nhánh **cuối** là nhánh mới nhất theo
# topo và rơi vào hàng 0, còn nhánh **đầu** (br01) rơi xuống cuối danh sách — đó chính
# là hình dạng "HEAD không phải nhánh mới nhất" mà lỗi cần.
# ---------------------------------------------------------------------------

for i in $(seq -w 1 "$NHANH"); do
  git checkout -q -b "br$i" main
  printf 'b%s\n' "$i" > "b$i.txt"
  git add -A
  ts=$((EPOCH_BASE + 10#$i * 86400))
  GIT_AUTHOR_DATE="$ts +0000" GIT_COMMITTER_DATE="$ts +0000" \
    git commit -qm "br$i work"
done

# 🔴 `br01` — nhánh **ĐẦU**, không phải nhánh cuối. Checkout nhánh cuối thì HEAD trùng
# hàng 0 (lane 0) và fixture KHÔNG kích hoạt được lỗi — đúng lý do repo perf 100k không
# phát hiện được vấn đề này.
git checkout -q br01

# ---------------------------------------------------------------------------
# 🔴 TỰ KIỂM: fixture có THẬT SỰ kích hoạt được lỗi không
#
# Không dùng `lanedist` ở đây (nó cần cargo, và cargo có thể đang bị khoá bởi ứng dụng
# của chủ dự án). Dùng một phép xấp xỉ **bảo thủ**: đếm số nhánh sống song song ở hàng
# của HEAD. Với hình dạng "N nhánh cùng rẽ từ một base", commit của nhánh thứ k theo
# thứ tự topo nhận lane k, nên HEAD (br01, nhánh **cuối** theo topo) nhận lane
# ~ NHANH - 1.
#
# Phép kiểm tất định thật sự vẫn là `lanedist` — xem đầu tệp.
# ---------------------------------------------------------------------------

SO_COMMIT="$(git rev-list --all --count)"
# `git log` bị cắt còn 50 dòng trong môi trường này (CONTEXT.md 3.6) — dùng rev-list.
VI_TRI_HEAD="$(git rev-list --all --topo-order | grep -n "^$(git rev-parse HEAD)$" | cut -d: -f1)"
HANG_HEAD=$((VI_TRI_HEAD - 1))

echo ""
echo "--- tự kiểm ---"
echo "commit: $SO_COMMIT (mong đợi $((NHANH + 1)))"
echo "HEAD: $(git rev-parse --abbrev-ref HEAD) @ $(git rev-parse --short HEAD)"
echo "hàng của HEAD theo topo: $HANG_HEAD (0 = hàng đầu)"

if [ "$SO_COMMIT" -ne $((NHANH + 1)) ]; then
  echo "LỖI: số commit $SO_COMMIT != $((NHANH + 1)) — repo sinh sai." >&2
  exit 1
fi

# HEAD ở hàng 0 nghĩa là fixture **KHÔNG** kích hoạt được lỗi: hàng 0 và HEAD trùng
# nhau, đúng ca mà repo perf 100k rơi vào và vì thế bỏ sót vấn đề.
if [ "$HANG_HEAD" -lt "$CAP" ]; then
  echo "" >&2
  echo "🔴 LỖI: HEAD ở hàng $HANG_HEAD < cap $CAP." >&2
  echo "   Fixture KHÔNG kích hoạt được lỗi clamp, nên mọi test dựa trên nó vô nghĩa." >&2
  echo "   Kiểm lại thứ tự ngày committer và nhánh được checkout (phải là br01)." >&2
  exit 1
fi

echo "✓ HEAD ở hàng $HANG_HEAD >= cap $CAP — fixture kích hoạt được ca clamp."
echo ""
echo "Kiểm độc lập (BẮT BUỘC dán vào SUMMARY):"
echo "  cargo run --release --bin lanedist \"$DEST\""
echo "  → phải in: HEAD that su: hang N, lane L  với L >= $CAP"
