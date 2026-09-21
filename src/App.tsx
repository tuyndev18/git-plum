import { useEffect, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

import { describeError, ipc } from '@/lib/ipc'
import { clearCommands, registerCommands, runCommand } from '@/lib/commands'
import { forgetRepo } from '@/lib/recentRepos'
import { useActiveRepo, useRepoStore } from '@/stores/repoStore'
import { useHistoryStore } from '@/stores/historyStore'
import { AppLayout } from '@/components/AppLayout'
import { CommandLogPanel } from '@/components/CommandLogPanel'
import { RecentRepoList } from '@/components/RecentRepoList'
import { CommitList } from '@/components/history/CommitList'

export function App() {
  const activeRepo = useActiveRepo()
  const activeRepoId = useRepoStore((s) => s.activeRepoId)
  const historyError = useHistoryStore((s) => (activeRepoId ? s.byRepo[activeRepoId]?.error : null))
  const isOpening = useRepoStore((s) => s.isOpening)
  const openRepository = useRepoStore((s) => s.openRepository)
  const closeRepository = useRepoStore((s) => s.closeRepository)
  const recent = useRepoStore((s) => s.recent)

  const [gitVersion, setGitVersion] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [logVisible, setLogVisible] = useState(true)
  const [logRefreshKey, setLogRefreshKey] = useState(0)
  // Trạng thái chọn commit ở dạng useState cho plan này. Plan 02-06 cần vùng
  // chi tiết đọc cùng id, và ARCHITECTURE.md Pattern 3 khuyên một
  // selectionStore riêng ("giữ CHỈ id, không giữ dữ liệu commit") — ghi rõ ở
  // SUMMARY để plan 02-06 biết phải nâng cấp lên store nếu cần chia sẻ giữa
  // nhiều component không phải con của App.
  const [selectedCommitId, setSelectedCommitId] = useState<string | null>(null)

  // Mọi thao tác đi qua sổ đăng ký lệnh (PLAT-04), không gắn thẳng vào onClick.
  useEffect(() => {
    registerCommands([
      {
        id: 'repo.open',
        title: 'Mở repository…',
        category: 'repo',
        keybinding: 'Ctrl+O',
        run: async () => {
          setError(null)
          const selected = await open({ directory: true, multiple: false })
          if (typeof selected !== 'string') return
          try {
            await openRepository(selected)
          } catch (e) {
            setError(describeError(e))
          } finally {
            setLogRefreshKey((k) => k + 1)
          }
        },
      },
      {
        id: 'repo.close',
        title: 'Đóng repository',
        category: 'repo',
        enabled: () => useRepoStore.getState().activeRepoId !== null,
        run: async () => {
          const id = useRepoStore.getState().activeRepoId
          if (id) {
            await closeRepository(id)
            useHistoryStore.getState().reset(id)
          }
          setError(null)
          setSelectedCommitId(null)
        },
      },
      {
        id: 'view.toggleCommandLog',
        title: 'Bật tắt nhật ký lệnh',
        category: 'view',
        keybinding: 'Ctrl+`',
        run: () => setLogVisible((v) => !v),
      },
      {
        id: 'history.refresh',
        title: 'Nạp lại lịch sử',
        category: 'history',
        enabled: () => useRepoStore.getState().activeRepoId !== null,
        run: async () => {
          const id = useRepoStore.getState().activeRepoId
          if (id) await useHistoryStore.getState().loadFirstPage(id)
        },
      },
      {
        id: 'history.scrollToTop',
        title: 'Cuộn lên đỉnh lịch sử',
        category: 'history',
        enabled: () => useRepoStore.getState().activeRepoId !== null,
        run: () => {
          document.querySelector('[data-testid="commit-scroll"]')?.scrollTo({ top: 0 })
        },
      },
    ])
    return () => clearCommands()
  }, [openRepository, closeRepository])

  // Nạp trang đầu của lịch sử khi repository đang hoạt động đổi (mở repo mới,
  // hoặc chuyển sang repo khác nếu v2 hỗ trợ nhiều tab). Quy tắc MVP của phase
  // này: mở repository là thấy lịch sử thật ngay, không có bước "sẽ nối sau".
  useEffect(() => {
    if (activeRepoId) {
      void useHistoryStore.getState().loadFirstPage(activeRepoId)
    }
  }, [activeRepoId])

  useEffect(() => {
    ipc
      .gitVersion()
      .then(setGitVersion)
      .catch((e) => setError(describeError(e)))
  }, [])

  // Nạp danh sách gần đây một lần lúc gắn kết (PLAT-07).
  useEffect(() => {
    void useRepoStore.getState().loadRecent()
  }, [])

  /**
   * Mở một mục cụ thể trong danh sách gần đây.
   *
   * **Vì sao không đi qua sổ đăng ký lệnh (PLAT-04).** Quy ước của dự án là
   * giao diện gọi thao tác qua `runCommand(id)`, và ở đây tôi cố tình không
   * theo. Lý do: `Command.run` trong `src/lib/commands.ts` có chữ ký
   * `() => void | Promise<void>` — **không nhận tham số**. "Mở repository này"
   * là một thao tác *có tham số*: nó cần biết đường dẫn nào.
   *
   * Nhồi nó vào sổ đăng ký hiện tại chỉ có hai đường, cả hai đều tệ hơn: đăng
   * ký một lệnh cho mỗi mục trong danh sách (sổ đăng ký thành dữ liệu động, mã
   * định danh không còn ổn định — trái hẳn ý định của PLAT-04), hoặc nhét đường
   * dẫn vào một biến toàn cục rồi để lệnh đọc ra (một tham số trá hình, khó lần
   * ra hơn hẳn việc truyền thẳng).
   *
   * Mở rộng sổ đăng ký cho lệnh có tham số là việc của v2, lúc làm bảng lệnh gõ
   * nhanh — chỗ đó mới có đủ dữ kiện để chọn hình dạng đúng cho chữ ký. Đổi một
   * ràng buộc kiến trúc PLAT-04 vượt quá phạm vi một plan. Tới lúc đó, `repo.open`
   * và `repo.close` vẫn đi qua sổ đăng ký như cũ; chỉ hai thao tác có tham số
   * này nhận hàm xử lý qua prop.
   */
  const handleOpenRecent = (path: string) => {
    setError(null)
    // Repository có thể đã bị xoá, đổi tên, hoặc nằm trên ổ mạng tạm thời không
    // truy cập được. Báo lỗi đọc hiểu được đúng như nhánh `repo.open`, và
    // **không** tự xoá mục khỏi danh sách — người dùng tự quyết định, ổ mạng
    // hôm nay không vào được mai lại vào được.
    openRepository(path)
      .catch((e) => setError(describeError(e)))
      .finally(() => setLogRefreshKey((k) => k + 1))
  }

  const handleForgetRecent = (path: string) => {
    // `forgetRepo` nuốt lỗi và trả `[]` khi store hỏng, nên `[]` vừa có nghĩa
    // "đã xoá mục cuối cùng" vừa có nghĩa "ghi thất bại". Nạp lại từ đĩa để
    // phân biệt: xoá thật thì đọc lại vẫn rỗng, còn ghi hỏng thì danh sách cũ
    // hiện lại — thà hiện một mục người dùng vừa bấm xoá còn hơn làm biến mất
    // cả danh sách vì một lần ghi hỏng.
    forgetRepo(path).then((conLai) => {
      if (conLai.length > 0) useRepoStore.getState().setRecent(conLai)
      else void useRepoStore.getState().loadRecent()
    })
  }

  return (
    <div className="app">
      <header className="toolbar">
        <div className="toolbar-left">
          <span className="brand">git-plum</span>
          {activeRepo && (
            <>
              <span className="repo-name">{activeRepo.info.name}</span>
              {activeRepo.currentBranch && (
                <span className="branch">{activeRepo.currentBranch}</span>
              )}
            </>
          )}
        </div>
        <div className="toolbar-right">
          <button onClick={() => runCommand('repo.open')} disabled={isOpening}>
            {isOpening ? 'Đang mở…' : 'Mở repository'}
          </button>
          {activeRepo && <button onClick={() => runCommand('repo.close')}>Đóng</button>}
          <button onClick={() => runCommand('view.toggleCommandLog')}>Nhật ký lệnh</button>
        </div>
      </header>

      {(error ?? historyError) && (
        <div className="error-banner" role="alert">
          <pre>{error ?? historyError}</pre>
          <button
            onClick={() => {
              setError(null)
              // Lỗi lịch sử vẫn còn trong historyStore sau khi đóng banner —
              // đây là cùng cơ chế "nuốt lỗi, giữ trong slice" mà
              // `ensureRange` đã dùng, không thêm cơ chế lỗi mới. Nạp lại
              // (`history.refresh`) là cách xoá nó, không phải nút Đóng này.
            }}
          >
            Đóng
          </button>
        </div>
      )}

      <div className="body">
        <AppLayout
          sidebar={
            <aside className="pane sidebar">
              <h2>Nhánh</h2>
              <p className="placeholder">Plan 02-06 sẽ điền phần này.</p>
            </aside>
          }
          main={
            <main className={`pane main${activeRepo ? ' main-history' : ''}`}>
              {activeRepo ? (
                <CommitList
                  repoId={activeRepo.info.id}
                  selectedCommitId={selectedCommitId}
                  onSelect={setSelectedCommitId}
                />
              ) : (
                <div className="empty-state">
                  <h2>Chưa mở repository nào</h2>
                  <button onClick={() => runCommand('repo.open')}>Mở repository</button>
                  <RecentRepoList
                    items={recent}
                    onOpen={handleOpenRecent}
                    onForget={handleForgetRecent}
                  />
                </div>
              )}
            </main>
          }
          detail={
            <aside className="pane detail">
              <h2>Chi tiết</h2>
              <p className="placeholder">Plan 02-06 sẽ điền phần này.</p>
            </aside>
          }
          bottom={logVisible ? <CommandLogPanel refreshKey={logRefreshKey} /> : undefined}
        />
      </div>

      <footer className="statusbar">
        <span>{gitVersion ?? 'Đang đọc phiên bản git…'}</span>
        <span className="spacer" />
        <span>git-plum 0.1.0</span>
      </footer>
    </div>
  )
}
