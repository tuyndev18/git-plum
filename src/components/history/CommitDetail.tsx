/**
 * Vùng chi tiết commit — HIST-08.
 *
 * Đọc `selectedCommitId` từ `selectionStore` (ARCHITECTURE.md Pattern 3: store
 * đó chỉ giữ id). Metadata (`Commit`) ưu tiên đọc từ `historyStore` — trang
 * lịch sử đã có sẵn, chọn commit là KHÔNG IPC cho phần metadata. Chỉ gọi
 * `ipc.getCommitDetail` cho danh sách TỆP, thứ mà trang lịch sử không có.
 *
 * Cache `CommitDetail` theo `commitId` trong một `Map` cấp module, chặn trên
 * số mục (`MAX_CACHE_ENTRIES`). Diff/danh sách tệp của một commit lịch sử là
 * BẤT BIẾN (không có thao tác nào ở Phase 2 sửa lịch sử), nên cache này không
 * bao giờ cần vô hiệu hoá — chỉ cần chặn trên để không phình vô hạn khi người
 * dùng lướt qua rất nhiều commit trong một phiên (ARCHITECTURE.md bảng cache).
 */

import { useEffect, useState } from 'react'

import { describeError, ipc, type Commit, type CommitDetail as CommitDetailPayload } from '@/lib/ipc'
import { useHistoryStore } from '@/stores/historyStore'
import { useDiffStore } from '@/stores/diffStore'
import { useSelectionStore } from '@/stores/selectionStore'
import { Avatar } from '@/components/Avatar'
import { FileList } from './FileList'

const MAX_CACHE_ENTRIES = 200

/** Cache cấp module — sống qua các lần mount/unmount của component. */
const detailCache = new Map<string, CommitDetailPayload>()

function cacheGet(commitId: string): CommitDetailPayload | undefined {
  return detailCache.get(commitId)
}

function cacheSet(commitId: string, detail: CommitDetailPayload): void {
  if (detailCache.size >= MAX_CACHE_ENTRIES && !detailCache.has(commitId)) {
    // Loại mục cũ nhất (thứ tự chèn của Map) — chặn trên đơn giản, không cần
    // LRU thật sự vì diff bất biến, chỉ cần không phình vô hạn.
    const oldest = detailCache.keys().next().value
    if (oldest !== undefined) detailCache.delete(oldest)
  }
  detailCache.set(commitId, detail)
}

/**
 * Xoá sạch cache — CHỈ dùng trong test. Cache cấp module sống qua các lần
 * mount/unmount là đúng ý định sản xuất (diff bất biến), nhưng nó làm rò rỉ
 * trạng thái giữa các test dùng chung `commitId` (ví dụ `'c1'`) nếu không xoá
 * ở `beforeEach`.
 */
export function __resetCommitDetailCacheForTest(): void {
  detailCache.clear()
}

function formatTime(unixSeconds: number): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(unixSeconds * 1000))
}

interface Props {
  repoId: string
}

export function CommitDetail({ repoId }: Props) {
  const selectedCommitId = useSelectionStore((s) => s.selectedByRepo[repoId] ?? null)
  const select = useSelectionStore((s) => s.select)
  const historyCommits = useHistoryStore((s) => s.byRepo[repoId]?.commits)

  const [detail, setDetail] = useState<CommitDetailPayload | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  // Đổi commit → bỏ chọn tệp của repo đó (plan 03-04). Tệp đang chọn có thể
  // KHÔNG tồn tại trong commit mới, và giữ nó nghĩa là `DiffViewer` gọi
  // `getFileDiff` với một path không có trong commit rồi hiện LỖI cho một thao
  // tác hoàn toàn bình thường. `commitChanged` chỉ chạm `selectedFileByRepo` —
  // `viewMode` và `showWhitespace` là lựa chọn người dùng và sống qua đây
  // (bài học HIST-09).
  useEffect(() => {
    useDiffStore.getState().commitChanged(repoId)
  }, [repoId, selectedCommitId])

  useEffect(() => {
    if (!selectedCommitId) {
      setDetail(null)
      setError(null)
      return
    }

    const cached = cacheGet(selectedCommitId)
    if (cached) {
      setDetail(cached)
      setError(null)
      return
    }

    let cancelled = false
    setIsLoading(true)
    setError(null)

    ipc
      .getCommitDetail(repoId, selectedCommitId)
      .then((result) => {
        if (cancelled) return
        cacheSet(selectedCommitId, result)
        setDetail(result)
      })
      .catch((e) => {
        if (cancelled) return
        setError(describeError(e))
      })
      .finally(() => {
        if (!cancelled) setIsLoading(false)
      })

    return () => {
      cancelled = true
    }
  }, [repoId, selectedCommitId])

  if (!selectedCommitId) {
    return (
      <div className="commit-detail commit-detail-empty">
        <p className="placeholder">Chọn một commit để xem chi tiết.</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="commit-detail commit-detail-error" role="alert">
        <pre>{error}</pre>
      </div>
    )
  }

  // Metadata: ưu tiên `historyStore` (đã nạp, không IPC). Nếu trang lịch sử
  // chưa có commit này (hiếm — chỉ khi chọn qua tìm kiếm tới hàng chưa nạp),
  // dùng `detail.commit` từ IPC làm nguồn phụ.
  const commitFromHistory = historyCommits?.find((c) => c?.id === selectedCommitId)
  const commit: Commit | undefined = commitFromHistory ?? detail?.commit

  if (!commit) {
    return (
      <div className="commit-detail commit-detail-loading">
        <p className="placeholder">{isLoading ? 'Đang tải…' : 'Không tìm thấy commit.'}</p>
      </div>
    )
  }

  return (
    <div className="commit-detail">
      <h3 className="commit-detail-subject">{commit.subject}</h3>
      {commit.body && <pre className="commit-detail-body">{commit.body}</pre>}

      <dl className="commit-detail-meta">
        <dt>Tác giả</dt>
        <dd className="commit-detail-author">
          {/*
            Avatar ở ĐÂY, không ở cột tác giả của danh sách commit.

            Đọc từ ảnh tham chiếu người dùng gửi: GitKraken để cột `AUTHOR`
            trong danh sách **chỉ có chữ** và dành avatar cho panel chi tiết bên
            phải, cạnh tên tác giả của commit đang chọn. Đưa avatar vào cột danh
            sách sẽ tốn một ô 16px trên **mỗi** hàng ở một cột vốn đã bị ép chỗ
            (xem nguyên nhân B của checkpoint Phase 2: badge ăn hết cột chữ),
            đổi lại thông tin mà một cái liếc vào panel đã cho.

            Mặc định là avatar tự sinh, không gọi mạng — xem `src/lib/avatar.ts`.
          */}
          <Avatar ten={commit.authorName} email={commit.authorEmail} co={20} />
          <span>
            {commit.authorName} <span className="commit-detail-email">{commit.authorEmail}</span>
          </span>
        </dd>

        <dt>Thời gian tác giả</dt>
        <dd>{formatTime(commit.authorTime)}</dd>

        {commit.committerName !== commit.authorName && (
          <>
            <dt>Người commit</dt>
            <dd>
              {commit.committerName}{' '}
              <span className="commit-detail-email">{commit.committerEmail}</span>
            </dd>
            <dt>Thời gian commit</dt>
            <dd>{formatTime(commit.committerTime)}</dd>
          </>
        )}

        <dt>Mã commit</dt>
        <dd className="commit-detail-sha">{commit.id}</dd>

        <dt>{commit.parents.length > 1 ? 'Mã cha' : 'Cha'}</dt>
        <dd>
          {commit.parents.length === 0 ? (
            <span className="commit-detail-root">commit gốc</span>
          ) : (
            <ul className="commit-detail-parents">
              {commit.parents.map((parentId) => (
                <li key={parentId}>
                  <button
                    className="commit-detail-parent-link"
                    onClick={() => select(repoId, parentId)}
                  >
                    {parentId}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </dd>
      </dl>

      <FileList files={detail?.files ?? []} truncated={detail?.truncated ?? false} repoId={repoId} />
    </div>
  )
}
