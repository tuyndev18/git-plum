/**
 * Thanh công cụ trình xem diff, và **năm dạng thông báo DIFF-06**.
 *
 * # Bốn nút, tất cả qua sổ đăng ký lệnh (PLAT-04)
 *
 * `onClick={() => void runCommand('diff.toggleViewMode')}`, **không**
 * `onClick={handleToggle}`. Lý do là của PLAT-04, không của plan này: bảng lệnh
 * gõ nhanh và phím tắt tuỳ biến (v2) cần một danh sách thao tác **có tên** lúc
 * chạy, và một thao tác gắn thẳng vào `onClick` là vô hình với cả hai.
 *
 * Lệnh đăng ký trong `useEffect` và **gỡ khi unmount**: `registerCommand` ném
 * khi trùng id, nên thiếu bước gỡ làm component vỡ ở lần mount thứ hai (trong
 * test là từ test thứ hai trở đi). `FileList.tsx` đã có đúng khuôn này.
 *
 * # Nút là ICON, chữ nằm ở `title` + `aria-label`
 *
 * Năm nhãn tiếng Việt ("Hiện khoảng trắng", "Ẩn lịch sử tệp"…) ăn hết bề ngang
 * một thanh công cụ nằm trên panel mà `MIN_SPLIT_WIDTH` đã ghim sàn 720px —
 * chính panel đó còn phải chứa hai cột nội dung cộng hai cột số dòng.
 *
 * Hai thứ **phải** đi cùng mỗi icon, và thiếu một trong hai là lỗi im lặng:
 *
 * - `title` — người dùng chuột thấy chữ khi rê vào. Bỏ nó thì nút thành câu đố.
 * - `aria-label` — trình đọc màn hình đọc nó. `<svg>` trong `icons.tsx` có
 *   `aria-hidden` nên **không** có nguồn chữ nào khác; bỏ nó thì nút câm hoàn
 *   toàn với người dùng bàn phím.
 *
 * Nhãn đổi theo trạng thái (`viewMode`, `showWhitespace`, `historyOpen`) nên cả
 * hai thuộc tính phải đọc cùng biểu thức — lệch nhau thì tooltip nói một đằng,
 * trình đọc màn hình đọc một nẻo.
 *
 * Test tìm nút bằng `data-testid`, **không** bằng chữ, nên việc bỏ chữ không
 * đụng tới chúng. Giữ nguyên như vậy.
 *
 * # Năm dạng thông báo — năm câu KHÁC NHAU
 *
 * DIFF-06 không phải "báo là không xem được". Mỗi dạng nói một lý do khác và
 * kèm những con số khác nhau, vì thứ người dùng cần quyết định sau đó khác nhau:
 *
 * | dạng | câu hỏi người dùng đang có | con số phải nêu |
 * |---|---|---|
 * | `binary` | "tệp này to ra hay nhỏ đi?" | cả hai kích thước |
 * | `tooLarge` | "tôi cách ngưỡng bao xa?" | **cả** kích thước thật **và** ngưỡng |
 * | `lfsPointer` | "nội dung thật to bao nhiêu?" | `kind.size` (nội dung), **không** cỡ tệp con trỏ |
 * | `unchanged` | "sao không có gì?" | không số — nêu *lý do khả dĩ* (đổi quyền) |
 * | lỗi | "git nói gì?" | `describeError` (kèm lệnh đã chạy, PLAT-10) |
 *
 * Con số của `lfsPointer` là chỗ dễ sai nhất: tệp con trỏ chỉ ~130 byte, còn nội
 * dung thật có thể 45 MB. 03-02 đã tách hai số này (và có test Rust ghim đúng
 * điều đó — mutation #9 của plan đó); giao diện không được trộn lại.
 */

import { useEffect } from 'react'

import { registerCommands, runCommand, unregisterCommand } from '@/lib/commands'
import type { DiffKind } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'
import {
  IconChevronDown,
  IconChevronUp,
  IconHistory,
  IconViewSplit,
  IconViewUnified,
  IconWhitespace,
} from '@/components/icons'

/**
 * Kích thước byte thành chuỗi người đọc được, định dạng theo locale bằng
 * `Intl.NumberFormat`.
 *
 * Dùng `Intl` chứ không `toFixed` + dấu chấm cứng: người dùng Việt viết `1,2 MB`
 * còn người dùng Mỹ viết `1.2 MB`, và `Intl` biết điều đó. Cùng lý do dự án
 * truyền timestamp `i64` qua IPC rồi để `Intl.DateTimeFormat` định dạng
 * (CLAUDE.md: "What NOT to Use" → `chrono`).
 */
export function formatBytes(bytes: number): string {
  const nf = (value: number, digits: number) =>
    new Intl.NumberFormat(undefined, {
      minimumFractionDigits: digits,
      maximumFractionDigits: digits,
    }).format(value)

  if (bytes < 1024) return `${nf(bytes, 0)} B`
  if (bytes < 1024 * 1024) return `${nf(bytes / 1024, 1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${nf(bytes / (1024 * 1024), 1)} MB`
  return `${nf(bytes / (1024 * 1024 * 1024), 1)} GB`
}

/** Bốn lệnh của trình xem diff. Id ổn định — không đổi sau khi đặt (PLAT-04). */
const COMMAND_IDS = [
  'diff.toggleViewMode',
  'diff.toggleWhitespace',
  'diff.nextHunk',
  'diff.prevHunk',
] as const

interface ToolbarProps {
  /**
   * Nhảy tới khối kế tiếp / trước đó. `DiffViewer` truyền vào vì chỉ nó giữ
   * `EditorView` và `lineMeta`.
   *
   * Không truyền (ví dụ khi chưa mở tệp nào) → lệnh vẫn đăng ký nhưng không làm
   * gì. Đăng ký có điều kiện sẽ làm `runCommand` **ném** khi người dùng gõ phím
   * tắt lúc chưa mở tệp, và một ngoại lệ cho một thao tác vô hại là tệ hơn.
   */
  onNextHunk?: () => void
  onPrevHunk?: () => void
  /**
   * Repo đang mở, để nút "Lịch sử tệp" biết có tệp nào đang chọn không (DIFF-05).
   *
   * **Optional** theo đúng tiền lệ `repoId` của `FileList` (03-04 deviation #4): các
   * test hiện có dựng `DiffToolbar` không có prop này, và một prop bắt buộc sẽ phá
   * chúng vì một lý do không liên quan tới hành vi đang kiểm. Không truyền → nút
   * hiện mờ, và `runCommand` vẫn không làm gì nhờ `enabled` của lệnh.
   */
  repoId?: string
}

export function DiffToolbar({ onNextHunk, onPrevHunk, repoId }: ToolbarProps = {}) {
  const viewMode = useDiffStore((s) => s.viewMode)
  const showWhitespace = useDiffStore((s) => s.showWhitespace)
  const historyOpen = useDiffStore((s) => s.historyOpen)
  const coTepDangChon = useDiffStore((s) =>
    repoId ? (s.selectedFileByRepo[repoId] ?? null) !== null : false,
  )

  useEffect(() => {
    registerCommands([
      {
        id: 'diff.toggleViewMode',
        title: 'Đổi chế độ xem diff (hợp nhất / hai cột)',
        category: 'view',
        run: () => useDiffStore.getState().toggleViewMode(),
      },
      {
        id: 'diff.toggleWhitespace',
        title: 'Bật tắt hiển thị ký tự khoảng trắng',
        category: 'view',
        run: () => useDiffStore.getState().toggleWhitespace(),
      },
      {
        id: 'diff.nextHunk',
        title: 'Khối thay đổi kế tiếp',
        category: 'view',
        run: () => onNextHunk?.(),
      },
      {
        id: 'diff.prevHunk',
        title: 'Khối thay đổi trước đó',
        category: 'view',
        run: () => onPrevHunk?.(),
      },
    ])
    return () => {
      for (const id of COMMAND_IDS) unregisterCommand(id)
    }
  }, [onNextHunk, onPrevHunk])

  return (
    <div className="diff-toolbar">
      <button
        className="icon-button"
        data-testid="diff-toggle-view-mode"
        title={viewMode === 'unified' ? 'Xem hai cột' : 'Xem hợp nhất'}
        aria-label={viewMode === 'unified' ? 'Xem hai cột' : 'Xem hợp nhất'}
        onClick={() => void runCommand('diff.toggleViewMode')}
      >
        {viewMode === 'unified' ? <IconViewSplit /> : <IconViewUnified />}
      </button>

      <button
        className="icon-button"
        data-testid="diff-toggle-whitespace"
        aria-pressed={showWhitespace}
        title={showWhitespace ? 'Ẩn khoảng trắng' : 'Hiện khoảng trắng'}
        aria-label={showWhitespace ? 'Ẩn khoảng trắng' : 'Hiện khoảng trắng'}
        onClick={() => void runCommand('diff.toggleWhitespace')}
      >
        <IconWhitespace />
      </button>

      {/*
        Lịch sử tệp (DIFF-05). Lệnh `diff.toggleFileHistory` được **`FileHistory`**
        đăng ký, không phải component này: nó là chủ của `historyOpen`, và đăng ký ở
        hai nơi làm `registerCommand` ném khi trùng id.

        Nút này không tự kiểm "có tệp đang chọn không" — `runCommand` đọc `enabled`
        của lệnh và không làm gì khi lệnh không gọi được (PLAT-04). `disabled` dưới
        đây chỉ để người dùng **thấy** điều đó, không phải để thực thi nó.
      */}
      <button
        className="icon-button"
        data-testid="diff-toggle-file-history"
        aria-pressed={historyOpen}
        disabled={!coTepDangChon}
        title={historyOpen ? 'Ẩn lịch sử tệp' : 'Lịch sử tệp'}
        aria-label={historyOpen ? 'Ẩn lịch sử tệp' : 'Lịch sử tệp'}
        onClick={() => void runCommand('diff.toggleFileHistory')}
      >
        <IconHistory />
      </button>

      <span className="diff-toolbar-spacer" />

      <button
        className="icon-button"
        data-testid="diff-prev-hunk"
        title="Khối thay đổi trước đó"
        aria-label="Khối thay đổi trước đó"
        onClick={() => void runCommand('diff.prevHunk')}
      >
        <IconChevronUp />
      </button>
      <button
        className="icon-button"
        data-testid="diff-next-hunk"
        title="Khối thay đổi kế tiếp"
        aria-label="Khối thay đổi kế tiếp"
        onClick={() => void runCommand('diff.nextHunk')}
      >
        <IconChevronDown />
      </button>
    </div>
  )
}

/**
 * Thông báo cho bốn dạng không phải `text`, cộng băng cảnh báo `truncated`.
 *
 * Trả `null` cho `text` **không** bị cắt: nội dung là việc của `DiffViewer`, và
 * một thông báo ở đây sẽ chồng lên nó.
 *
 * Dạng `text` **bị cắt** là ca đặc biệt: người dùng đang xem *một phần* và phải
 * biết điều đó, nên băng hiện **cùng với** nội dung chứ không thay nó.
 */
export function DiffNotice({ kind }: { kind: DiffKind }) {
  if (kind.kind === 'text') {
    /*
     * Hai cờ **độc lập**, và cùng `true` thì hiện **cả hai** băng.
     *
     * Chúng nói hai điều khác nhau: `truncated` = mất nội dung; `contextOnly` =
     * thấy đủ thay đổi, thiếu ngữ cảnh giữa. Một tệp vừa lớn vừa có quá nhiều
     * khối đổi là ca thật, và cho cờ này đè cờ kia sẽ giấu mất một nửa sự thật.
     */
    if (!kind.truncated && !kind.contextOnly) return null
    return (
      <>
        {kind.truncated && (
          <div
            className="diff-notice diff-notice-truncated"
            data-testid="diff-truncated"
            role="status"
          >
            ⚠️ Diff đã bị cắt vì tệp này có quá nhiều khối thay đổi — bạn đang xem{' '}
            <strong>một phần</strong> nội dung.
          </div>
        )}
        {kind.contextOnly && (
          /*
           * 🔴 KHÔNG dùng `⚠️` và KHÔNG dùng chữ "bị cắt".
           *
           * Người dùng **không mất thay đổi nào** ở đây — mọi dòng thêm/xoá đều
           * có mặt, chỉ các đoạn không đổi ở giữa là bị lược. Một cảnh báo kiểu
           * "diff bị cắt" cho một tệp 600 KB hoàn toàn bình thường làm người
           * dùng mất tin vào trình xem, và có test ghim điều đó.
           *
           * Nhưng nó **phải** hiện ra: im lặng lùi về rút gọn đúng là việc làm
           * người dùng tưởng số dòng nhảy là lỗi và báo hai lần (2026-09-22).
           */
          <div
            className="diff-notice diff-notice-degraded"
            data-testid="diff-context-only"
            role="status"
          >
            Tệp lớn nên đang hiện <strong>các khối thay đổi</strong> kèm 3 dòng ngữ cảnh,
            không phải toàn tệp. Mọi thay đổi đều có mặt — số dòng nhảy là vì các đoạn
            không đổi được lược bớt.
          </div>
        )}
      </>
    )
  }

  if (kind.kind === 'binary') {
    return (
      <div className="diff-notice" data-testid="diff-notice" role="status">
        <strong>Tệp nhị phân</strong> — {formatBytes(kind.oldSize)} → {formatBytes(kind.newSize)}
        <p className="diff-notice-detail">
          Không có cách hiển thị khác biệt theo dòng cho nội dung nhị phân.
        </p>
      </div>
    )
  }

  if (kind.kind === 'tooLarge') {
    return (
      <div className="diff-notice" data-testid="diff-notice" role="status">
        Tệp <strong>{formatBytes(kind.size)}</strong> vượt ngưỡng{' '}
        <strong>{formatBytes(kind.limit)}</strong>
        <p className="diff-notice-detail">
          Ngưỡng là mức mà trình xem còn dùng được, không phải mức máy còn chịu nổi.
        </p>
      </div>
    )
  }

  if (kind.kind === 'lfsPointer') {
    return (
      <div className="diff-notice" data-testid="diff-notice" role="status">
        <strong>Con trỏ Git LFS</strong> — nội dung thật {formatBytes(kind.size)}
        <p className="diff-notice-detail">
          Nội dung thật nằm ngoài repo. Mã đối tượng: <code>{kind.oid.slice(0, 8)}</code>
        </p>
      </div>
    )
  }

  // `unchanged` — khác hẳn ca `text` với 0 hunk (tệp có đổi nhưng diff rỗng sau
  // khi lọc) và khác hẳn ca lỗi. Ba tình huống, ba câu.
  return (
    <div className="diff-notice" data-testid="diff-notice" role="status">
      Tệp <strong>không thay đổi nội dung</strong> (có thể chỉ đổi quyền truy cập tệp).
    </div>
  )
}
