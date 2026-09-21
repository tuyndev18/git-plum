#!/usr/bin/env bash
#
# Sinh repo đo hiệu năng ~100k commit CÓ HÌNH DẠNG NHÁNH THẬT — HIST-03.
#
# Chủ dự án đã chọn sinh repo thay vì tải Linux kernel (CONTEXT.md <preparation>).
#
# Vì sao không phải một đường thẳng: một đường thẳng 100k commit KHÔNG đo được gì về
# lane — mà lane chính là thứ cần đo. Repo này luôn có 8–20 nhánh sống đồng thời, mỗi
# nhánh 5–50 commit rồi merge về main bằng --no-ff, rải thêm merge octopus ba cha.
#
# ---------------------------------------------------------------------------
# RÀNG BUỘC BẮT BUỘC: MỌI COMMIT PHẢI THAY ĐỔI NỘI DUNG TỆP — TREE PHẢI KHÁC CHA.
# ---------------------------------------------------------------------------
# Kể cả commit merge. Đã kiểm chứng trên git 2.54.0.windows.1:
#
#   # Merge có tree TRÙNG một cha
#   git rev-list --all --count        → 3
#   git log --all --format=%H | wc -l → 2      ← commit merge BỊ LƯỢC
#   git log --all --format=%P | grep -c ' ' → 0  ← cổng đếm merge trả 0 dù CÓ merge
#
#   # Merge có tree KHÁC cả hai cha
#   git rev-list --all --count        → 4
#   git log --all --format=%H | wc -l → 4      ← khớp
#
# `git log` áp history simplification và lược commit có tree trùng một cha. `--sparse`,
# `--full-history`, `--boundary`, `-m` ĐỀU KHÔNG cứu được — đã thử cả bốn. Nếu script
# sinh ra commit không đổi nội dung (ví dụ merge chỉ để tạo hình dạng, không sửa tệp
# nào), repo vẫn hợp lệ nhưng cổng verify trả 0 và task thất bại theo cách không giải
# thích được. Vì vậy mọi commit ở dưới đều ghi `M 100644` cho ít nhất một tệp.
#
# Hệ quả cho việc đếm: dùng `git rev-list --all --format=%P` (lọc dòng `^commit `),
# KHÔNG dùng `git log`. `rev-list` không áp simplification.
#
# ---------------------------------------------------------------------------
# VÌ SAO DÙNG git fast-import
# ---------------------------------------------------------------------------
# `git commit` gọi 100k lần là 100k tiến trình — hàng giờ. `git fast-import` nhận một
# luồng trên stdin và làm việc đó trong vài phút. Luồng cũng cho phép khai báo nhiều
# dòng `merge` cho một commit, nên octopus làm được (đã xác minh: `from :3` cộng hai
# dòng `merge` cho commit ba cha).
#
# CẢNH BÁO `data <n>`: n đếm BYTE, không đếm ký tự. Lệch một byte thì fast-import chết
# theo cách vô dụng — "fatal: unsupported command: ommit refs/heads/main", không nói
# byte nào sai, không nói dòng nào. Vì vậy script này chỉ phát nội dung ASCII thuần và
# tính n trong hàm emit_data, hàm đó có cổng chặn ký tự ngoài ASCII. Không bao giờ viết
# n bằng tay.
#
# Script tự chạy thử một luồng 5 commit trước khi chạy luồng lớn. Thất bại ở 5 commit
# mất một giây; thất bại ở 100k commit mất nhiều phút và vẫn không biết sai đâu.
#
# Cách dùng:
#   bash scripts/fixtures/make-perf-repo.sh [thư-mục-đích] [số-commit]
# Mặc định: target/fixtures-perf/perf-100k, 100000 commit.
#
# ⚠️ Mất nhiều phút và chiếm dung lượng đĩa đáng kể. KHÔNG chạy trong CI mặc định
#    (threat T-02-03).

set -euo pipefail

# LC_ALL=C ghim hai thứ mà script này dựa vào: khoảng ký tự `[!\ -~]` của cổng ASCII
# trong emit_data, và việc ${#s} đếm BYTE chứ không đếm ký tự. Không ghim thì hành vi
# đổi theo locale của máy chạy.
export LC_ALL=C

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEST_ARG="${1:-$PROJECT_ROOT/target/fixtures-perf/perf-100k}"
TARGET_COMMITS="${2:-100000}"

# ---------------------------------------------------------------------------
# Cổng an toàn khi xoá thư mục (T-02-01) — cùng lý lẽ như make-fixtures.sh
# ---------------------------------------------------------------------------
if [ -z "${DEST_ARG// /}" ] || [ "$DEST_ARG" = "/" ]; then
  echo "LỖI: thư mục đích rỗng hoặc là gốc hệ thống tệp — từ chối chạy." >&2
  exit 1
fi
case "$TARGET_COMMITS" in
  ''|*[!0-9]*)
    echo "LỖI: số commit '$TARGET_COMMITS' không phải số nguyên dương." >&2
    exit 1
    ;;
esac
if [ "$TARGET_COMMITS" -lt 10 ]; then
  echo "LỖI: số commit phải ≥10." >&2
  exit 1
fi

mkdir -p "$(dirname "$DEST_ARG")"
DEST_PARENT="$(cd "$(dirname "$DEST_ARG")" && pwd)"
DEST="$DEST_PARENT/$(basename "$DEST_ARG")"

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
AUTHOR_NAME="Plum Perf"
AUTHOR_EMAIL="perf@git-plum.test"
readonly EPOCH_BASE=1577836800

# Bộ sinh số giả ngẫu nhiên có HẠT GIỐNG CỐ ĐỊNH — lần chạy sau ra đúng repo cũ.
# Đây là một LCG (linear congruential generator) tối giản; tham số lấy từ Numerical
# Recipes. Dùng số học 32 bit để bash không tràn.
RAND_STATE=20200101
RAND_OUT=0
# Trả kết quả qua biến toàn cục RAND_OUT thay vì `echo`. Gọi qua $(rand n) sinh một
# subshell mỗi lần — với 100k commit đó là hàng trăm nghìn tiến trình.
rand() {
  RAND_STATE=$(( (RAND_STATE * 1103515245 + 12345) % 2147483648 ))
  RAND_OUT=$(( RAND_STATE % $1 ))
}

# ---------------------------------------------------------------------------
# Phát luồng fast-import
# ---------------------------------------------------------------------------

# Phát một khối `data <n>`.
#
# VỀ TỐC ĐỘ: bản đầu của hàm này ghi tệp tạm rồi gọi `wc -c` và `cat` cho MỖI khối
# data. Đã đo: 2000 commit mất 328 giây, tức 100k commit mất khoảng 4,5 GIỜ — mỗi
# commit sinh ba tiến trình con và đó là toàn bộ thời gian chạy. Không dùng được.
#
# Bản này tính số byte trong bash, không sinh tiến trình nào. An toàn vì:
#   * MỌI nội dung script này phát ra là ASCII thuần (xem emit_data_checked bên dưới —
#     có cổng kiểm, sai thì dừng chứ không sinh repo hỏng), và
#   * với ASCII thuần thì số ký tự = số byte, nên ${#s} đúng bằng `wc -c`.
#
# Cộng 1 cho newline mà `printf '%s\n'` thêm vào.
#
# KHÔNG mở rộng hàm này cho nội dung ngoài ASCII. Nội dung không-ASCII phải đi qua
# đường `wc -c` như make-fixtures.sh làm cho repo non-utf8 — ${#s} đếm ký tự trong
# một số locale và sẽ lệch, khiến fast-import chết với thông báo vô dụng
# "fatal: unsupported command: ommit refs/heads/main".
emit_data() {
  local s="$1"
  # Cổng ASCII: nếu lọt ký tự ngoài ASCII thì dừng ngay, đừng sinh repo hỏng rồi mới
  # phát hiện sau 100k commit.
  # `[!\ -~]` = mọi thứ ngoài khoảng in được của ASCII (0x20..0x7e). Phải viết đúng
  # dạng này: bản đầu dùng `*[!$' \t!-~']*` và nó khớp CẢ chuỗi ASCII thuần, làm cổng
  # báo sai ngay ở commit đầu — kiểu lỗi mà luồng thử 12 commit bắt được trong một giây.
  case $s in
    *[!\ -~]*)
      echo "LỖI nội bộ: emit_data nhận nội dung ngoài ASCII: $s" >&2
      echo "      Dùng đường wc -c cho nội dung này, xem ghi chú trong script." >&2
      exit 1
      ;;
  esac
  printf 'data %d\n%s\n' "$(( ${#s} + 1 ))" "$s"
}

# Sinh toàn bộ luồng fast-import ra stdout.
#
# Hình dạng: main tiến từng bước; định kỳ rẽ 8–20 nhánh tính năng, mỗi nhánh 5–50
# commit, rồi merge về main bằng một commit có hai cha (hoặc ba cha cho octopus).
# Mỗi commit ghi một tệp nên tree luôn khác cha — xem ràng buộc ở đầu tệp.
generate_stream() {
  local target="$1"
  local mark=0          # số hiệu mark kế tiếp
  local commits=0       # số commit đã phát
  local main_mark=0     # mark của đầu main
  local ts=$EPOCH_BASE
  local round=0

  # --- commit gốc trên main ---
  mark=$((mark + 1)); local blob=$mark
  printf 'blob\nmark :%d\n' "$blob"
  emit_data "khoi tao repo do hieu nang"
  mark=$((mark + 1)); main_mark=$mark
  ts=$((ts + 60))
  printf 'commit refs/heads/main\nmark :%d\n' "$main_mark"
  printf 'author %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
  printf 'committer %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
  emit_data "commit goc"
  printf 'M 100644 :%d main.txt\n\n' "$blob"
  commits=$((commits + 1))

  while [ "$commits" -lt "$target" ]; do
    round=$((round + 1))

    # Vài commit trên main giữa hai đợt rẽ nhánh, để main không chỉ gồm merge.
    rand 4; local straight=$(( RAND_OUT + 1 ))
    local k
    for (( k=1; k<=straight; k++ )); do
      [ "$commits" -ge "$target" ] && break
      mark=$((mark + 1)); blob=$mark
      printf 'blob\nmark :%d\n' "$blob"
      emit_data "main round $round buoc $k"
      mark=$((mark + 1)); local c=$mark
      ts=$((ts + 60))
      printf 'commit refs/heads/main\nmark :%d\n' "$c"
      printf 'author %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
      printf 'committer %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
      emit_data "main: round $round buoc $k"
      printf 'from :%d\n' "$main_mark"
      printf 'M 100644 :%d main.txt\n\n' "$blob"
      main_mark=$c
      commits=$((commits + 1))
    done
    [ "$commits" -ge "$target" ] && break

    # --- một đợt: 8–20 nhánh cùng rẽ từ đầu main hiện tại ---
    # Cùng rẽ từ một điểm nghĩa là chúng SỐNG ĐỒNG THỜI, đúng thứ cần đo về lane.
    rand 13; local fan=$(( RAND_OUT + 8 ))
    local branch_base=$main_mark
    local tips=""          # mark đầu của từng nhánh, để merge sau
    local b
    for (( b=1; b<=fan; b++ )); do
      [ "$commits" -ge "$target" ] && break
      rand 46; local depth=$(( RAND_OUT + 5 ))   # 5–50 commit mỗi nhánh
      local parent=$branch_base
      local d
      for (( d=1; d<=depth; d++ )); do
        [ "$commits" -ge "$target" ] && break
        mark=$((mark + 1)); blob=$mark
        printf 'blob\nmark :%d\n' "$blob"
        emit_data "feature r$round b$b commit $d"
        mark=$((mark + 1)); local fc=$mark
        ts=$((ts + 60))
        # Nhánh tính năng dùng ref tạm; sau khi merge thì xoá ref, nhưng commit vẫn
        # nằm trong lịch sử qua merge nên rev-list --all vẫn thấy.
        printf 'commit refs/heads/feat-r%d-b%d\nmark :%d\n' "$round" "$b" "$fc"
        printf 'author %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
        printf 'committer %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
        emit_data "feat r$round b$b: commit $d"
        printf 'from :%d\n' "$parent"
        printf 'M 100644 :%d feat-r%d-b%d.txt\n\n' "$blob" "$round" "$b"
        parent=$fc
        commits=$((commits + 1))
      done
      tips="$tips $parent"
    done

    # --- merge từng nhánh về main ---
    # Cứ năm đợt thì một đợt dùng octopus ba cha (hai dòng `merge`), để repo có cả
    # hình dạng nhiều hơn hai cha — thuật toán lane phải lặp qua mọi cha.
    local use_octopus=0
    [ $(( round % 5 )) -eq 0 ] && use_octopus=1

    local pending=""
    local pending_n=0
    local tip
    for tip in $tips; do
      if [ "$use_octopus" = "1" ]; then
        pending="$pending $tip"
        # Gom hai nhánh rồi phát một merge ba cha (main + 2 nhánh). Dùng biến đếm
        # riêng: `wc -w` là một tiến trình mỗi merge, và `set --` sẽ đè tham số hàm.
        pending_n=$(( pending_n + 1 ))
        if [ "$pending_n" -lt 2 ]; then
          continue
        fi
      fi

      mark=$((mark + 1)); blob=$mark
      printf 'blob\nmark :%d\n' "$blob"
      emit_data "merge log round $round"
      mark=$((mark + 1)); local mc=$mark
      ts=$((ts + 60))
      printf 'commit refs/heads/main\nmark :%d\n' "$mc"
      printf 'author %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
      printf 'committer %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
      if [ "$use_octopus" = "1" ]; then
        emit_data "merge octopus round $round"
      else
        emit_data "merge feat round $round"
      fi
      printf 'from :%d\n' "$main_mark"
      if [ "$use_octopus" = "1" ]; then
        local p
        for p in $pending; do
          printf 'merge :%d\n' "$p"
        done
        pending=""
        pending_n=0
      else
        printf 'merge :%d\n' "$tip"
      fi
      # Commit merge PHẢI sửa tệp, nếu không git log sẽ lược nó. Xem đầu tệp.
      printf 'M 100644 :%d merges.log\n\n' "$blob"
      main_mark=$mc
      commits=$((commits + 1))
    done

    # Nhánh nào còn sót (octopus gom lẻ) thì merge nốt bằng merge hai cha.
    for tip in $pending; do
      mark=$((mark + 1)); blob=$mark
      printf 'blob\nmark :%d\n' "$blob"
      emit_data "merge log tail round $round"
      mark=$((mark + 1)); local mc2=$mark
      ts=$((ts + 60))
      printf 'commit refs/heads/main\nmark :%d\n' "$mc2"
      printf 'author %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
      printf 'committer %s <%s> %d +0000\n' "$AUTHOR_NAME" "$AUTHOR_EMAIL" "$ts"
      emit_data "merge feat tail round $round"
      printf 'from :%d\n' "$main_mark"
      printf 'merge :%d\n' "$tip"
      printf 'M 100644 :%d merges.log\n\n' "$blob"
      main_mark=$mc2
      commits=$((commits + 1))
    done

    # Xoá ref nhánh tính năng: repo thật không giữ hàng nghìn nhánh đã merge, và
    # `for-each-ref` chậm đi vô ích nếu giữ. Commit vẫn còn qua merge.
    for (( b=1; b<=fan; b++ )); do
      printf 'reset refs/heads/feat-r%d-b%d\nfrom 0000000000000000000000000000000000000000\n\n' \
        "$round" "$b"
    done
  done
}

# ---------------------------------------------------------------------------
# Chạy thử luồng 5 commit TRƯỚC khi chạy luồng lớn
# ---------------------------------------------------------------------------
# Thất bại ở 5 commit mất một giây. Thất bại ở 100k commit mất nhiều phút và thông báo
# của fast-import vẫn không nói byte nào sai.
echo "Chạy thử luồng nhỏ để kiểm cú pháp fast-import..."
SMOKE_DIR="$(mktemp -d)"
git init --initial-branch=main --quiet "$SMOKE_DIR/repo"
if ! generate_stream 12 | git -C "$SMOKE_DIR/repo" fast-import --quiet 2>"$SMOKE_DIR/err.txt"; then
  echo "LỖI: luồng thử thất bại — cú pháp fast-import sai, KHÔNG chạy luồng lớn." >&2
  cat "$SMOKE_DIR/err.txt" >&2
  rm -rf "$SMOKE_DIR"
  exit 1
fi
SMOKE_COUNT="$(git -C "$SMOKE_DIR/repo" rev-list --all --count)"
rm -rf "$SMOKE_DIR"
echo "  luồng thử OK — $SMOKE_COUNT commit."
echo

# ---------------------------------------------------------------------------
# Luồng thật
# ---------------------------------------------------------------------------
echo "Sinh repo $TARGET_COMMITS commit vào: $DEST"
rm -rf "$DEST"
mkdir -p "$DEST"
git init --initial-branch=main --quiet "$DEST"
git -C "$DEST" config core.autocrlf false
git -C "$DEST" config gc.auto 0

RAND_STATE=20200101   # đặt lại hạt giống: luồng thật phải độc lập với luồng thử

START_EPOCH="$(date +%s)"
generate_stream "$TARGET_COMMITS" | git -C "$DEST" fast-import --quiet
IMPORT_EPOCH="$(date +%s)"

echo "  import xong sau $((IMPORT_EPOCH - START_EPOCH))s. Đang đóng gói lại..."
git -C "$DEST" repack -ad --quiet
git -C "$DEST" reset --hard --quiet main
PACK_EPOCH="$(date +%s)"

# ---------------------------------------------------------------------------
# Số thật — người chạy chép lại vào README
# ---------------------------------------------------------------------------
COUNT="$(git -C "$DEST" rev-list --all --count)"
LOG_COUNT="$(git -C "$DEST" log --all --topo-order --format=%H | wc -l | tr -d ' ')"
MERGES="$(git -C "$DEST" rev-list --all --format=%P \
  | /usr/bin/grep -v '^commit ' | /usr/bin/grep -c ' ' || true)"
OCTOPUS="$(git -C "$DEST" rev-list --all --format=%P \
  | /usr/bin/grep -v '^commit ' | /usr/bin/grep -cE ' .* ' || true)"
GIT_SIZE="$(du -sh "$DEST/.git" 2>/dev/null | cut -f1)"

# Lệnh mà plan 02-02 sẽ phân tích — con số này là mốc so sánh.
LOG_START="$(date +%s%N)"
LOG_BYTES="$(git -C "$DEST" log --all --topo-order --format=%H%x1f%P%x1e | wc -c | tr -d ' ')"
LOG_END="$(date +%s%N)"
LOG_MS=$(( (LOG_END - LOG_START) / 1000000 ))

echo
echo "===================== SỐ THẬT ====================="
printf 'git rev-list --all --count        : %s\n' "$COUNT"
printf 'git log --all --topo-order (dòng) : %s\n' "$LOG_COUNT"
printf 'commit merge (nhiều cha)          : %s\n' "$MERGES"
printf '  trong đó octopus (≥3 cha)       : %s\n' "$OCTOPUS"
printf 'kích thước .git trên đĩa          : %s\n' "$GIT_SIZE"
printf 'thời gian sinh (import)           : %ss\n' "$((IMPORT_EPOCH - START_EPOCH))"
printf 'thời gian đóng gói (repack)       : %ss\n' "$((PACK_EPOCH - IMPORT_EPOCH))"
printf 'git log --all --topo-order ... |wc: %sms (%s byte)\n' "$LOG_MS" "$LOG_BYTES"
echo "==================================================="

# Cổng: rev-list PHẢI bằng số dòng git log. Lệch nghĩa là script sinh ra commit có tree
# trùng cha và bị history simplification lược — phải sửa SCRIPT, không sửa cổng.
if [ "$COUNT" != "$LOG_COUNT" ]; then
  echo >&2
  echo "LỖI: rev-list ($COUNT) khác git log ($LOG_COUNT)." >&2
  echo "      Script đã sinh ra commit có tree trùng cha và bị git log lược bỏ." >&2
  echo "      Sửa script để mọi commit đều ghi tệp — KHÔNG nới cổng này." >&2
  exit 1
fi

if [ "$MERGES" -lt 1 ]; then
  echo "LỖI: không có commit merge nào — repo là đường thẳng, không đo được lane." >&2
  exit 1
fi

echo
echo "Xong. Repo: $DEST"
echo "⚠️  Không chạy script này trong CI mặc định (T-02-03)."
