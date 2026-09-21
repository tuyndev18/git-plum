/**
 * Nhật ký lệnh git — PLAT-08.
 *
 * Hiện đúng những lệnh ứng dụng đã chạy. Đây là thứ phân biệt git-plum với các
 * công cụ đồ hoạ che giấu Git: người dùng học được git từ chính việc dùng công
 * cụ, và khi có sự cố thì biết chính xác chuyện gì đã xảy ra.
 */

import { useCallback, useEffect, useState } from 'react'
import clsx from 'clsx'

import { ipc, type CommandLogEntry } from '@/lib/ipc'

interface Props {
  /** Tăng giá trị này để buộc nạp lại nhật ký sau một thao tác. */
  refreshKey: number
}

export function CommandLogPanel({ refreshKey }: Props) {
  const [entries, setEntries] = useState<CommandLogEntry[]>([])

  const refresh = useCallback(() => {
    ipc.commandLog().then(setEntries).catch(() => setEntries([]))
  }, [])

  useEffect(refresh, [refresh, refreshKey])

  return (
    <section className="pane command-log">
      <header className="command-log-header">
        <h2>Nhật ký lệnh</h2>
        <div>
          <button onClick={refresh}>Làm mới</button>
          <button
            onClick={() => {
              ipc.clearCommandLog().then(refresh)
            }}
          >
            Xoá
          </button>
        </div>
      </header>

      {entries.length === 0 ? (
        <p className="placeholder">Chưa có lệnh nào được chạy.</p>
      ) : (
        <table className="command-log-table">
          <thead>
            <tr>
              <th>Lệnh</th>
              <th>Mã thoát</th>
              <th>Thời gian</th>
            </tr>
          </thead>
          <tbody>
            {entries.map((e) => (
              <tr
                key={e.seq}
                className={clsx({ failed: e.exitCode !== 0 })}
                title={e.error ?? undefined}
              >
                <td className="cmd">
                  <code>{e.command}</code>
                  {e.error && <div className="cmd-error">{e.error}</div>}
                </td>
                <td className="exit">{e.exitCode ?? '—'}</td>
                <td className="duration">{e.durationMs} ms</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  )
}
