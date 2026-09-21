#!/usr/bin/env bash
#
# Sinh bộ repo mẫu cho thuật toán lane của Phase 2 — HIST-03, HIST-11.
#
# Mỗi repo mang đúng một hình dạng bệnh lý làm vỡ cài đặt lane ngây thơ. Bảng đầy đủ
# ở scripts/fixtures/README.md.
#
# TÍNH TẤT ĐỊNH LÀ YÊU CẦU CỨNG. Plan 02-03 chụp ảnh (snapshot) đầu ra gán lane bằng
# `insta`; nếu mã commit đổi mỗi lần chạy script thì snapshot đỏ vô cớ. Vì vậy:
#   * ghim cả sáu biến GIT_AUTHOR_* / GIT_COMMITTER_* (tên, email, ngày),
#   * dấu thời gian là một chuỗi tăng dần cố định bắt đầu 2020-01-01T00:00:00+00:00,
#   * `git init --initial-branch=main` để init.defaultBranch của máy không lọt vào,
#   * `core.autocrlf=false` mỗi repo — nếu không, máy Windows và máy Linux sinh ra
#     blob khác nhau cho cùng nội dung, và mã commit lệch theo.
#
# RÀNG BUỘC: mọi commit phải thay đổi nội dung tệp, kể cả commit merge. `git log` áp
# history simplification và LƯỢC BỎ commit có tree trùng một cha — merge không sửa tệp
# nào sẽ biến mất khỏi `git log` dù `git rev-list` vẫn thấy. Đã thử `--sparse`,
# `--full-history`, `--boundary`, `-m`: không cờ nào cứu được. Chi tiết trong README.
#
# Cách dùng:
#   bash scripts/fixtures/make-fixtures.sh [thư-mục-đích]
# Mặc định thư mục đích là <gốc-dự-án>/target/fixtures.

set -euo pipefail

# ---------------------------------------------------------------------------
# Đường dẫn và cổng an toàn (T-02-01)
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

DEST_ARG="${1:-$PROJECT_ROOT/target/fixtures}"

# Script này xoá thư mục để chạy lại được (idempotent). Xoá đệ quy dựa trên đối số
# người dùng là việc dễ gây tai hoạ, nên chặn trước khi làm gì cả:
#   * đối số rỗng hoặc "/" thì từ chối,
#   * đường dẫn phải nằm DƯỚI gốc dự án — không cho xoá ngoài cây dự án.
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
    echo "      Script chỉ được xoá và dựng lại thư mục bên trong cây dự án." >&2
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

# 2020-01-01T00:00:00+00:00 dưới dạng Unix epoch. Mỗi commit cộng 60 giây.
readonly EPOCH_BASE=1577836800
TICK=0

# Đặt dấu thời gian cho commit kế tiếp. Dùng định dạng "<epoch> +0000" vì đó là dạng
# git nhận trực tiếp, không phụ thuộc locale hay múi giờ của máy.
next_stamp() {
  TICK=$((TICK + 1))
  local ts=$((EPOCH_BASE + TICK * 60))
  export GIT_AUTHOR_DATE="$ts +0000"
  export GIT_COMMITTER_DATE="$ts +0000"
}

# Khởi tạo một repo mẫu sạch. Xoá trước để chạy lại cho kết quả y hệt.
init_repo() {
  local name="$1"
  local path="$DEST/$name"

  # Xoá và KIỂM đã xoá được thật. Trên Windows, OneDrive, trình diệt virus hoặc một
  # tiến trình còn mở tệp đều làm `rm -rf` thất bại một phần — nó in cảnh báo rồi trả
  # về mã 0. Không kiểm thì script chạy tiếp trên thư mục còn sót và vỡ ở tận nơi
  # khác với thông điệp vô nghĩa: `fatal: cannot lock ref 'HEAD'`.
  rm -rf "$path" 2>/dev/null || true
  if [ -e "$path" ]; then
    sleep 1
    rm -rf "$path" 2>/dev/null || true
  fi
  if [ -e "$path" ]; then
    echo "LỖI: không xoá được '$path' — có tiến trình đang giữ tệp trong đó." >&2
    echo "      Trên Windows thường là OneDrive đang đồng bộ, trình diệt virus, hoặc" >&2
    echo "      một cửa sổ Explorer/terminal đang mở thư mục đó." >&2
    echo "      Đóng chúng rồi chạy lại. Chạy tiếp trên thư mục còn sót sẽ sinh repo hỏng." >&2
    exit 1
  fi

  mkdir -p "$path"
  git init --initial-branch=main --quiet "$path"
  # core.autocrlf=false: giữ blob giống nhau giữa Windows và Linux.
  # user.* ghim thêm ở cấp repo để repo vẫn tất định khi ai đó chạy lệnh git thủ công.
  git -C "$path" config core.autocrlf false
  git -C "$path" config commit.gpgsign false
  git -C "$path" config user.name "$GIT_AUTHOR_NAME"
  git -C "$path" config user.email "$GIT_AUTHOR_EMAIL"
  echo "$path"
}

# Tạo một commit thay đổi nội dung tệp. Bắt buộc phải đổi nội dung — xem ghi chú
# history simplification ở đầu tệp.
commit_in() {
  local path="$1" file="$2" content="$3" msg="$4"
  printf '%s\n' "$content" >>"$path/$file"
  git -C "$path" add -- "$file"
  next_stamp
  git -C "$path" commit --quiet -m "$msg"
}

# Commit merge có sửa tệp. `git merge --no-ff` một mình có thể sinh merge commit mà
# tree trùng một cha (khi nhánh kia không đụng tệp nào mới); ta ghi thêm một dòng vào
# sổ merge rồi `--amend` để tree luôn khác cả hai cha, nhờ đó `git log` không lược.
merge_in() {
  local path="$1" branch="$2" msg="$3"
  next_stamp
  git -C "$path" merge --no-ff --no-edit --quiet -m "$msg" "$branch"
  printf 'merged %s\n' "$branch" >>"$path/merges.log"
  git -C "$path" add -- merges.log
  git -C "$path" commit --quiet --amend --no-edit
}

# In một dòng của bảng tổng kết.
FIXTURE_REPORT=""
report() {
  local name="$1" path="$2" note="${3:-}"
  local commits branches
  commits="$(git -C "$path" rev-list --all --count)"
  branches="$(git -C "$path" for-each-ref --format='%(refname)' refs/heads | wc -l | tr -d ' ')"
  FIXTURE_REPORT="${FIXTURE_REPORT}$(printf '%-12s | %8s | %8s | %s' \
    "$name" "$commits" "$branches" "$note")"$'\n'
}

echo "Sinh repo mẫu vào: $DEST"
echo

# ---------------------------------------------------------------------------
# linear — ca cơ sở, 20 commit một đường thẳng
# ---------------------------------------------------------------------------
echo "[linear] 20 commit tuần tự"
LINEAR="$(init_repo linear)"
for i in $(seq 1 20); do
  commit_in "$LINEAR" file.txt "dòng $i" "commit $i"
done
report linear "$LINEAR" "đường thẳng"

# ---------------------------------------------------------------------------
# octopus — một merge BỐN cha
# Cài đặt chỉ đọc parents[1] sẽ bỏ sót ba cha còn lại.
# ---------------------------------------------------------------------------
echo "[octopus] merge bốn cha"
OCTO="$(init_repo octopus)"
commit_in "$OCTO" base.txt "gốc" "commit gốc"
# Commit thứ hai trên main TRƯỚC khi rẽ nhánh là cần thiết. Nếu main chỉ có commit gốc
# thì main là tổ tiên thuần của cả bốn nhánh, chiến lược octopus tiến main tới f1 và
# merge commit chỉ còn BA cha. Có commit riêng, main không fast-forward được nữa và
# merge cho đúng bốn cha.
commit_in "$OCTO" base.txt "gốc 2" "commit gốc 2"
for i in 1 2 3 4; do
  git -C "$OCTO" checkout --quiet -b "f$i" main
  commit_in "$OCTO" "f$i.txt" "nhánh f$i" "f$i: một commit"
done
git -C "$OCTO" checkout --quiet main
next_stamp
# KHÔNG dùng `--no-ff` với merge octopus. Đã đo: `--no-ff` khiến git thêm chính main
# làm một cha nữa, ra NĂM cha chứ không phải bốn. Merge thường là đúng — dòng
# "Fast-forwarding to: f1" mà git in ra chỉ là bước nội bộ của chiến lược octopus,
# commit cuối vẫn có đủ bốn cha (đã xác minh bằng `git cat-file -p HEAD`).
git -C "$OCTO" merge --no-edit --quiet -m "merge octopus bốn cha" f1 f2 f3 f4
# Merge octopus giữ tree khác cha vì bốn nhánh mang bốn tệp mới, nhưng vẫn ghi thêm
# một tệp cho chắc và để nhất quán với ràng buộc "mọi commit đổi nội dung".
printf 'octopus f1 f2 f3 f4\n' >>"$OCTO/merges.log"
git -C "$OCTO" add -- merges.log
git -C "$OCTO" commit --quiet --amend --no-edit

OCTO_PARENTS="$(git -C "$OCTO" cat-file -p HEAD | /usr/bin/grep -c '^parent ' || true)"
if [ "$OCTO_PARENTS" != "4" ]; then
  echo "LỖI: octopus HEAD có $OCTO_PARENTS cha, cần đúng 4." >&2
  exit 1
fi
report octopus "$OCTO" "HEAD có $OCTO_PARENTS cha"

# ---------------------------------------------------------------------------
# unrelated — hai gốc rời, hợp bằng --allow-unrelated-histories
# Số lane tăng đột biến khi lane thứ hai xuất hiện từ hư không.
# ---------------------------------------------------------------------------
echo "[unrelated] hai gốc rời rồi hợp"
UNREL="$(init_repo unrelated)"
for i in 1 2 3; do
  commit_in "$UNREL" main.txt "main $i" "main $i"
done
git -C "$UNREL" checkout --quiet --orphan other
# Xoá sạch index và cây làm việc: nhánh mồ côi phải bắt đầu từ trắng, nếu còn tệp của
# main thì hai lịch sử chia sẻ blob và hình dạng không còn là "hai gốc rời" thật.
git -C "$UNREL" rm -rq --cached . 2>/dev/null || true
rm -f "$UNREL/main.txt"
for i in 1 2 3; do
  commit_in "$UNREL" other.txt "other $i" "other $i"
done
git -C "$UNREL" checkout --quiet main
next_stamp
git -C "$UNREL" merge other --allow-unrelated-histories --no-edit --quiet \
  -m "hợp hai lịch sử không liên quan"
printf 'merged other (unrelated)\n' >>"$UNREL/merges.log"
git -C "$UNREL" add -- merges.log
git -C "$UNREL" commit --quiet --amend --no-edit

UNREL_ROOTS="$(git -C "$UNREL" rev-list --all --max-parents=0 | wc -l | tr -d ' ')"
if [ "$UNREL_ROOTS" != "2" ]; then
  echo "LỖI: unrelated có $UNREL_ROOTS gốc, cần đúng 2." >&2
  exit 1
fi
report unrelated "$UNREL" "$UNREL_ROOTS gốc rời"

# ---------------------------------------------------------------------------
# orphan — một nhánh mồ côi, KHÔNG merge
# Không có cha chung; dễ bị coi là commit gốc nằm giữa lịch sử.
# ---------------------------------------------------------------------------
echo "[orphan] nhánh mồ côi không merge"
ORPHAN="$(init_repo orphan)"
for i in 1 2 3 4 5; do
  commit_in "$ORPHAN" app.txt "app $i" "app $i"
done
git -C "$ORPHAN" checkout --quiet --orphan docs
git -C "$ORPHAN" rm -rq --cached . 2>/dev/null || true
rm -f "$ORPHAN/app.txt"
for i in 1 2 3; do
  commit_in "$ORPHAN" "docs.txt" "tài liệu $i" "docs $i"
done
git -C "$ORPHAN" checkout --quiet main
ORPHAN_ROOTS="$(git -C "$ORPHAN" rev-list --all --max-parents=0 | wc -l | tr -d ' ')"
if [ "$ORPHAN_ROOTS" != "2" ]; then
  echo "LỖI: orphan có $ORPHAN_ROOTS gốc, cần đúng 2." >&2
  exit 1
fi
report orphan "$ORPHAN" "$ORPHAN_ROOTS gốc, không merge"

# ---------------------------------------------------------------------------
# wide — 24 nhánh sống đồng thời rồi merge dần
# Mục tiêu: có thời điểm ≥20 lane sống, để kiểm giới hạn lane hiển thị và cách vẽ
# suy giảm. Merge dần từng nhánh nên số lane giảm từ 24 về 1.
# ---------------------------------------------------------------------------
echo "[wide] 24 nhánh đồng thời"
WIDE="$(init_repo wide)"
commit_in "$WIDE" base.txt "gốc" "commit gốc"
for i in $(seq -w 1 24); do
  git -C "$WIDE" checkout --quiet -b "w$i" main
  commit_in "$WIDE" "w$i.txt" "nhánh w$i commit 1" "w$i: commit 1"
  commit_in "$WIDE" "w$i.txt" "nhánh w$i commit 2" "w$i: commit 2"
done
git -C "$WIDE" checkout --quiet main
for i in $(seq -w 1 24); do
  merge_in "$WIDE" "w$i" "merge w$i"
done
WIDE_BRANCHES="$(git -C "$WIDE" for-each-ref --format='%(refname)' refs/heads | wc -l | tr -d ' ')"
report wide "$WIDE" "$WIDE_BRANCHES nhánh, 24 merge --no-ff"

# ---------------------------------------------------------------------------
# non-utf8 — HIST-11: tên tệp và thông điệp commit KHÔNG phải UTF-8
#
# Đây là repo phải dựng bằng `git fast-import`, không phải `git commit`. Đã đo trên
# máy này (git 2.54.0.windows.1):
#   * `printf 'caf\351.txt'` trong shell bị chuyển thành UTF-8 (caf\303\251.txt)
#     trước khi tới hệ thống tệp, nên không tạo được tên tệp byte thô bằng cách ghi
#     tệp rồi `git add`;
#   * `git commit -F <tệp chứa byte \377>` cũng chuyển \377 thành \303\277 và in
#     "warning: commit message did not conform to UTF-8";
#   * `git fast-import` GIỮ NGUYÊN byte thô ở cả đường dẫn (dạng "caf\351.txt" có
#     dấu ngoặc kép) và thông điệp (khối `data <n>` đếm byte). Đã xác nhận bằng
#     `git ls-tree -z | xxd` → 6361 66e9 ("caf\xe9") và `git cat-file commit` →
#     6261 6420 ff20 ("bad \xff ").
# Vì vậy fast-import là cách duy nhất tạo được dữ liệu đầu vào thật cho HIST-11.
# ---------------------------------------------------------------------------
echo "[non-utf8] byte thô trong tên tệp và thông điệp (qua fast-import)"
NONUTF8="$(init_repo non-utf8)"
STREAM="$DEST/.non-utf8.fi"
MSG_RAW="$DEST/.non-utf8-msg.bin"
MSG_EMOJI="$DEST/.non-utf8-emoji.bin"

# Thông điệp chứa byte \377 đơn lẻ — không hợp lệ UTF-8 ở bất kỳ vị trí nào.
printf 'thông điệp có byte thô \377 ở giữa' >"$MSG_RAW"
# Thông điệp emoji nhiều byte: 🍑 (U+1F351) = \360\237\215\221.
printf '\360\237\215\221 s\341\273\255a l\341\273\227i' >"$MSG_EMOJI"

# `data <n>`: n đếm BYTE, không đếm ký tự.
#
# TUYỆT ĐỐI KHÔNG viết số n bằng tay. Đã trả giá: đếm tay "nội dung a\n" ra 14 trong
# khi thật ra là 13 byte (tiếng Việt có ký tự đa byte), fast-import lệch một byte rồi
# chết với "fatal: unsupported command: ommit refs/heads/main" — chữ 'c' của 'commit'
# bị khối data trước ăn mất. Thông báo không nói byte nào sai, không nói dòng nào.
#
# Hai hàm dưới đây loại bỏ hẳn khả năng đếm sai: mọi khối `data` đều đi qua chúng và
# n luôn do `wc -c` đo trên đúng chuỗi byte sắp ghi ra.

# Phát một khối `data <n>` với nội dung lấy từ đối số (thêm newline cuối).
emit_data() {
  local payload="$1"
  local tmp="$DEST/.emit.bin"
  printf '%s\n' "$payload" >"$tmp"
  printf 'data %s\n' "$(wc -c <"$tmp" | tr -d ' ')"
  cat "$tmp"
  rm -f "$tmp"
}

# Phát một khối `data <n>` với nội dung lấy nguyên byte từ một tệp (không thêm gì).
emit_data_file() {
  local file="$1"
  printf 'data %s\n' "$(wc -c <"$file" | tr -d ' ')"
  cat "$file"
}

# Phát hai dòng author/committer đã ghim, dấu thời gian tăng dần cố định.
emit_ident() {
  local ts="$1"
  printf 'author %s <%s> %d +0000\n' "$GIT_AUTHOR_NAME" "$GIT_AUTHOR_EMAIL" "$ts"
  printf 'committer %s <%s> %d +0000\n' "$GIT_COMMITTER_NAME" "$GIT_COMMITTER_EMAIL" "$ts"
}

{
  # commit 1: tên tệp byte Latin-1 thô (caf\351.txt), thông điệp ASCII.
  printf 'blob\nmark :1\n'
  emit_data 'nội dung a'
  printf 'commit refs/heads/main\nmark :2\n'
  emit_ident $((EPOCH_BASE + 60))
  emit_data 'them tep co ten byte Latin-1'
  printf 'M 100644 :1 "caf\\351.txt"\n\n'

  # commit 2: thông điệp chứa byte \377 đơn lẻ.
  printf 'blob\nmark :3\n'
  emit_data 'nội dung b'
  printf 'commit refs/heads/main\nmark :4\n'
  emit_ident $((EPOCH_BASE + 120))
  emit_data_file "$MSG_RAW"
  printf '\nfrom :2\nM 100644 :3 b.txt\n\n'

  # commit 3: thông điệp emoji nhiều byte (UTF-8 hợp lệ nhưng ngoài BMP).
  printf 'blob\nmark :5\n'
  emit_data 'nội dung c'
  printf 'commit refs/heads/main\nmark :6\n'
  emit_ident $((EPOCH_BASE + 180))
  emit_data_file "$MSG_EMOJI"
  printf '\nfrom :4\nM 100644 :5 c.txt\n\n'

  # commit 4: tên thư mục chứa byte thô, để kiểm phân tích đường dẫn nhiều cấp.
  printf 'blob\nmark :7\n'
  emit_data 'nội dung d'
  printf 'commit refs/heads/main\nmark :8\n'
  emit_ident $((EPOCH_BASE + 240))
  emit_data 'them duong dan nhieu cap'
  printf 'M 100644 :7 "d\\356r/fi\\374le.txt"\n\n'
} >"$STREAM"

git -C "$NONUTF8" fast-import --quiet <"$STREAM"
rm -f "$STREAM" "$MSG_RAW" "$MSG_EMOJI"
git -C "$NONUTF8" reset --hard --quiet main 2>/dev/null || true

# Xác minh byte thô thật sự nằm trong tree, không phải đã bị chuyển thành UTF-8.
if git -C "$NONUTF8" ls-tree -r -z --name-only HEAD | xxd -p | tr -d '\n' \
    | /usr/bin/grep -q 'e9'; then
  NONUTF8_NOTE="tên tệp có byte thô \\xe9"
else
  NONUTF8_NOTE="CẢNH BÁO: không thấy byte thô trong tên tệp"
  echo "CẢNH BÁO: non-utf8 không giữ được byte thô trong tên tệp trên máy này." >&2
fi
report non-utf8 "$NONUTF8" "$NONUTF8_NOTE"

# ---------------------------------------------------------------------------
# shallow — bản sao nông của linear
# Cha của commit cũ nhất trỏ tới thứ KHÔNG có trong tập dữ liệu. Lane phải kết thúc
# gọn, không được sập.
#
# `--no-local` là bắt buộc: clone theo đường dẫn cục bộ mặc định dùng hardlink và
# BỎ QUA --depth trong im lặng, cho ra repo đầy đủ chứ không nông.
# ---------------------------------------------------------------------------
echo "[shallow] bản sao nông --depth=3"
rm -rf "$DEST/shallow"
git -c core.autocrlf=false clone --depth=3 --no-local --quiet "$LINEAR" "$DEST/shallow"
SHALLOW="$DEST/shallow"
git -C "$SHALLOW" config core.autocrlf false
if [ ! -f "$SHALLOW/.git/shallow" ]; then
  echo "LỖI: $SHALLOW/.git/shallow không tồn tại — clone không nông." >&2
  exit 1
fi
SHALLOW_COUNT="$(git -C "$SHALLOW" rev-list --all --count)"
report shallow "$SHALLOW" "nông, $SHALLOW_COUNT/20 commit"

# ---------------------------------------------------------------------------
# detached — HEAD tách rời tại commit giữa
# Không nhãn nhánh nào neo vào hàng đó.
# ---------------------------------------------------------------------------
echo "[detached] HEAD tách rời"
rm -rf "$DEST/detached"
git -c core.autocrlf=false clone --quiet "$LINEAR" "$DEST/detached"
DETACHED="$DEST/detached"
git -C "$DETACHED" config core.autocrlf false
git -C "$DETACHED" checkout --quiet --detach HEAD~5
if git -C "$DETACHED" symbolic-ref --quiet HEAD >/dev/null 2>&1; then
  echo "LỖI: detached HEAD vẫn trỏ tới một nhánh." >&2
  exit 1
fi
report detached "$DETACHED" "HEAD tách rời tại HEAD~5"

# ---------------------------------------------------------------------------
# submodule — một commit có submodule
# Nghiên cứu ghi "chưa ai kiểm submodule có làm lệch gán lane không", nên phải có
# repo để kiểm. Thất bại ở repo này KHÔNG làm cả script thất bại.
#
# `protocol.file.allow=always` là bắt buộc: git ≥2.38 chặn submodule qua giao thức
# file theo mặc định (CVE-2022-39253). Cờ này chỉ đặt cho MỘT lệnh ở đây, tuyệt đối
# không đặt trong exec.rs của ứng dụng (T-02-02).
# ---------------------------------------------------------------------------
echo "[submodule] một commit có submodule"
SUB="$(init_repo submodule)"
SUB_OK=0
commit_in "$SUB" readme.txt "repo cha" "commit gốc"
if git -C "$SUB" -c protocol.file.allow=always submodule add --quiet "$LINEAR" sub 2>/dev/null; then
  git -C "$SUB" add -A
  next_stamp
  if git -C "$SUB" commit --quiet -m "thêm submodule sub"; then
    SUB_OK=1
  fi
fi
if [ "$SUB_OK" = "1" ]; then
  report submodule "$SUB" "có submodule 'sub'"
else
  echo "CẢNH BÁO: không dựng được repo 'submodule' trên máy này — bỏ qua." >&2
  report submodule "$SUB" "CẢNH BÁO: submodule add thất bại"
fi

# ---------------------------------------------------------------------------
# Bảng tổng kết
# ---------------------------------------------------------------------------
echo
printf '%-12s | %8s | %8s | %s\n' "repo" "commit" "nhánh" "ghi chú"
printf '%-12s-+-%8s-+-%8s-+-%s\n' "------------" "--------" "--------" "-------"
printf '%s' "$FIXTURE_REPORT"
echo
echo "Xong. Thư mục fixture: $DEST"
echo "Chỉ cho test Rust thấy thư mục khác bằng biến môi trường GIT_PLUM_FIXTURES."
