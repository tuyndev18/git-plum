import { useEffect, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

import { describeError, ipc } from '@/lib/ipc'
import { clearCommands, registerCommands, runCommand } from '@/lib/commands'
import { useActiveRepo, useRepoStore } from '@/stores/repoStore'
import { AppLayout } from '@/components/AppLayout'
import { CommandLogPanel } from '@/components/CommandLogPanel'

export function App() {
  const activeRepo = useActiveRepo()
  const isOpening = useRepoStore((s) => s.isOpening)
  const openRepository = useRepoStore((s) => s.openRepository)
  const closeRepository = useRepoStore((s) => s.closeRepository)

  const [gitVersion, setGitVersion] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [logVisible, setLogVisible] = useState(true)
  const [logRefreshKey, setLogRefreshKey] = useState(0)

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
          if (id) await closeRepository(id)
          setError(null)
        },
      },
      {
        id: 'view.toggleCommandLog',
        title: 'Bật tắt nhật ký lệnh',
        category: 'view',
        keybinding: 'Ctrl+`',
        run: () => setLogVisible((v) => !v),
      },
    ])
    return () => clearCommands()
  }, [openRepository, closeRepository])

  useEffect(() => {
    ipc
      .gitVersion()
      .then(setGitVersion)
      .catch((e) => setError(describeError(e)))
  }, [])

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

      {error && (
        <div className="error-banner" role="alert">
          <pre>{error}</pre>
          <button onClick={() => setError(null)}>Đóng</button>
        </div>
      )}

      <div className="body">
        <AppLayout
          sidebar={
            <aside className="pane sidebar">
              <h2>Nhánh</h2>
              <p className="placeholder">Phase 2 sẽ điền phần này.</p>
            </aside>
          }
          main={
            <main className="pane main">
              {activeRepo ? (
                <div className="repo-summary">
                  <h2>{activeRepo.info.name}</h2>
                  <dl>
                    <dt>Đường dẫn</dt>
                    <dd>{activeRepo.info.path}</dd>
                    <dt>Nhánh hiện tại</dt>
                    <dd>{activeRepo.currentBranch ?? '(chưa có commit nào)'}</dd>
                  </dl>
                  <p className="placeholder">
                    Đồ thị commit và danh sách lịch sử thuộc Phase 2.
                  </p>
                </div>
              ) : (
                <div className="empty-state">
                  <h2>Chưa mở repository nào</h2>
                  <button onClick={() => runCommand('repo.open')}>Mở repository</button>
                </div>
              )}
            </main>
          }
          detail={
            <aside className="pane detail">
              <h2>Chi tiết</h2>
              <p className="placeholder">Phase 2 sẽ điền phần này.</p>
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
