/**
 * Lịch sử thay đổi của riêng **một** tệp — DIFF-05.
 *
 * # Danh sách phẳng, **không** ảo hoá — và đó là quyết định, không phải thiếu sót
 *
 * Backend chặn ở `MAX_FILE_HISTORY = 200`, nên DOM không bao giờ vượt 200 hàng
 * (T-03-34). `useVirtualizer` ở đây là phức tạp không mua được gì: nó thêm một lớp
 * đo chiều cao, một `ref` cuộn, và một lớp lỗi "hàng lệch so với vùng cuộn" mà
 * Phase 2 đã trả giá hai vòng checkpoint để sửa.
 *
 * `CommitList` **cần** ảo hoá vì nó dựng 100 nghìn hàng. Đây thì không. Ghi ra để
 * người sau không thêm nó theo quán tính "danh sách thì phải ảo hoá".
 *
 * # Bấm một phiên bản KHÔNG đổi commit đang chọn trên đồ thị
 *
 * `diffStore.historyCommitOverride` là một **ghi đè**, không phải một lời ghi vào
 * `selectionStore`. Lý do đầy đủ ở doc comment của trường đó; tóm lại: người dùng
 * đang đứng ở một chỗ trong lịch sử, và đọc lịch sử của một tệp không được làm họ
 * mất chỗ đó.
 *
 * # Chỗ đổi tên phải HIỆN RA
 *
 * `--follow` là thứ làm lịch sử không đứt ở chỗ đổi tên, và nó có giá (git tính điểm
 * tương đồng ở mỗi commit đụng path). Người dùng **phải thấy** mình nhận được gì:
 * một hàng nói "đổi tên từ `<tên cũ>`". Hưởng thụ im lặng nghĩa là đường dẫn ở hàng
 * dưới khác hàng trên mà không có gì giải thích.
 */

import { useEffect, useRef, useState } from 'react'

import { registerCommands, runCommand, unregisterCommand } from '@/lib/commands'
import { describeError, ipc, type FileHistory as FileHistoryData } from '@/lib/ipc'
import { useDiffStore } from '@/stores/diffStore'

/** Số phiên bản tối đa backend trả về. Khớp `MAX_FILE_HISTORY` phía Rust. */
export const MAX_FILE_HISTORY = 200

/** Lệnh của panel lịch sử tệp. Id ổn định — không đổi sau khi đặt (PLAT-04). */
const COMMAND_IDS = ['diff.toggleFileHistory'] as const

/**
 * Giây Unix → chuỗi ngày giờ theo locale người dùng.
 *
 * 🔴 `unixSeconds * 1000`: `Date` nhận **milli**. Thiếu phép nhân thì mọi hàng hiện
 * một ngày năm 1970, và ca đó đã nằm trong checklist checkpoint của 02-06 — tức là
 * một lỗi đã được lường trước, nên nó có test riêng.
 *
 * `Intl.DateTimeFormat` chứ không tự ghép chuỗi: nó biết locale và múi giờ của người
 * dùng, và đó đúng là lý do dự án truyền `i64` qua IPC thay vì dùng `chrono` phía
 * Rust (CLAUDE.md, "What NOT to Use").
 */
export function formatVersionTime(unixSeconds: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(unixSeconds * 1000))
}

interface Props {
  repoId: string
}

export function FileHistory({ repoId }: Props) {
  const selectedFile = useDiffStore((s) => s.selectedFileByRepo[repoId] ?? null)
  const historyOpen = useDiffStore((s) => s.historyOpen)
  const historyCommitOverride = useDiffStore((s) => s.historyCommitOverride)
  const selectHistoryVersion = useDiffStore((s) => s.selectHistoryVersion)

  const [data, setData] = useState<FileHistoryData | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  /*
   * 🔴 **Cổng chống đua (T-03-37).** `requestId` tăng mỗi lần effect chạy, và phản
   * hồi chỉ được nhận khi `requestId` của nó vẫn là lần mới nhất.
   *
   * Không có cổng này: người dùng mở lịch sử của `a.ts`, đổi sang `b.ts`, và phản hồi
   * của `a.ts` về **sau** — danh sách hiện các phiên bản của một tệp **khác** trong
   * khi tiêu đề nói `b.ts`, và **không lỗi nào** được báo. Cùng lớp lỗi đã chặn ở
   * 03-04 Task 2, và `cancelled = true` kiểu cũ không đủ: nó chỉ chặn effect **đã bị
   * dọn**, không chặn hai lời gọi song song về sai thứ tự.
   */
  const requestIdRef = useRef(0)

  useEffect(() => {
    if (!historyOpen || !selectedFile) {
      setData(null)
      setError(null)
      setIsLoading(false)
      return
    }

    const id = ++requestIdRef.current
    setIsLoading(true)
    setError(null)

    ipc
      .getFileHistory(repoId, selectedFile)
      .then((ra) => {
        if (id !== requestIdRef.current) return
        setData(ra)
      })
      .catch((e) => {
        if (id !== requestIdRef.current) return
        setError(describeError(e))
      })
      .finally(() => {
        if (id !== requestIdRef.current) return
        setIsLoading(false)
      })
  }, [repoId, selectedFile, historyOpen])

  /*
   * Lệnh đăng ký trong `useEffect` và **gỡ khi unmount**: `registerCommand` ném khi
   * trùng id, nên thiếu bước gỡ làm component vỡ ở lần mount thứ hai (trong test là
   * từ test thứ hai trở đi). `DiffToolbar.tsx` và `FileList.tsx` đã có khuôn này.
   */
  useEffect(() => {
    registerCommands([
      {
        id: 'diff.toggleFileHistory',
        title: 'Lịch sử của tệp đang chọn',
        category: 'history',
        // `enabled` của `Command` chứ không tự kiểm trong `onClick`: bảng lệnh gõ
        // nhanh (v2) đọc trường này để mờ đi lệnh không gọi được, và một phép kiểm
        // nằm trong `onClick` là vô hình với nó.
        enabled: () => {
          const s = useDiffStore.getState()
          return (s.selectedFileByRepo[repoId] ?? null) !== null
        },
        run: () => {
          const s = useDiffStore.getState()
          if (s.historyOpen) s.closeHistory()
          else s.openHistory()
        },
      },
    ])
    return () => {
      for (const id of COMMAND_IDS) unregisterCommand(id)
    }
  }, [repoId])

  if (!historyOpen || !selectedFile) return null

  return (
    <div className="file-history" data-testid="file-history">
      <div className="file-history-header">
        <span className="file-history-title">
          Lịch sử của <strong data-testid="file-history-path">{selectedFile}</strong>
        </span>
        <button
          data-testid="file-history-close"
          onClick={() => void runCommand('diff.toggleFileHistory')}
        >
          Đóng lịch sử
        </button>
      </div>

      {/*
        `truncated` hiện **trên giao diện**, không chỉ nằm trong payload (T-03-38).
        Một lịch sử bị cắt mà không ai nói ra là một lịch sử sai mà không ai biết.
      */}
      {data?.truncated && (
        <div className="diff-notice diff-notice-degraded" data-testid="file-history-truncated" role="status">
          Đang hiện <strong>{MAX_FILE_HISTORY} phiên bản gần nhất</strong>. Tệp này có
          nhiều commit hơn thế.
        </div>
      )}

      {error ? (
        <div className="diff-notice diff-notice-error" data-testid="file-history-error" role="alert">
          <pre>{error}</pre>
        </div>
      ) : isLoading ? (
        <p className="placeholder" data-testid="file-history-loading">
          Đang đọc lịch sử tệp…
        </p>
      ) : data && data.versions.length === 0 ? (
        /*
          Ba tình huống, ba câu: đang nạp, lỗi, và "không có trong lịch sử". Dùng một
          câu chung cho cả ba làm người dùng không phân biệt được "tôi gõ sai tên tệp"
          với "git vừa lỗi" — đúng khuôn năm thông báo của DIFF-06.
        */
        <p className="placeholder" data-testid="file-history-empty">
          Tệp này không có trong lịch sử.
        </p>
      ) : data ? (
        <ul className="file-version-list">
          {data.versions.map((v) => (
            <li key={v.commitId}>
              <button
                className={`file-version-row${
                  v.commitId === historyCommitOverride ? ' file-version-row-active' : ''
                }`}
                data-testid={`file-version-${v.commitId}`}
                aria-current={v.commitId === historyCommitOverride}
                /*
                  🔴 `selectHistoryVersion`, KHÔNG `selectionStore.select()`. Xem doc
                  comment đầu tệp và của `historyCommitOverride`.
                */
                onClick={() => selectHistoryVersion(v.commitId)}
              >
                <span className="file-version-time">{formatVersionTime(v.authorTime)}</span>
                <span className="file-version-author">{v.authorName}</span>
                <span className="file-version-subject">{v.subject}</span>
                <code className="file-version-sha">{v.commitId.slice(0, 8)}</code>
                {v.oldPath && (
                  <span className="file-version-rename" data-testid={`file-version-rename-${v.commitId}`}>
                    đổi tên từ <code>{v.oldPath}</code>
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      ) : null}
    </div>
  )
}
