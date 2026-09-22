import { lazy, Suspense, useEffect, useRef, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

import { describeError, ipc } from '@/lib/ipc'
import { isPerfEnabled } from '@/lib/perf'
import { clearCommands, registerCommands, runCommand } from '@/lib/commands'
import { forgetRepo } from '@/lib/recentRepos'
import { useActiveRepo, useRepoStore } from '@/stores/repoStore'
import { useHistoryStore } from '@/stores/historyStore'
import { useSelectionStore } from '@/stores/selectionStore'
import { AppLayout } from '@/components/AppLayout'
import { CommandLogPanel } from '@/components/CommandLogPanel'
import { Logo } from '@/components/Logo'
import { RecentRepoList } from '@/components/RecentRepoList'
import { RefSidebar } from '@/components/RefSidebar'
import { CommitList, type CommitListHandle } from '@/components/history/CommitList'
import { CommitDetail } from '@/components/history/CommitDetail'
import { CommitSearch } from '@/components/history/CommitSearch'
import { DiffViewer } from '@/components/diff/DiffViewer'
import { useDiffStore } from '@/stores/diffStore'

/**
 * Khung đo của checkpoint #3 (plan 03-01) — nạp lười, **spike tạm thời**.
 *
 * `lazy()` chứ không nhập tĩnh: `SpikeHarness` kéo theo `diffSpike.ts`, vốn kéo
 * `@codemirror/*`. Nhập tĩnh sẽ đưa CodeMirror vào bundle đường chính kể cả khi cờ
 * perf tắt — tức spike làm chậm khởi động, đúng thứ nó được viết ra để bảo vệ.
 *
 * Xoá cùng lúc với `diffSpike.ts` và `SpikeHarness.tsx` khi Task 3 chốt xong A/B.
 */
const SpikeHarness = lazy(() =>
  import('@/components/diff/SpikeHarness').then((m) => ({ default: m.SpikeHarness })),
)

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
  /*
   * Nhật ký lệnh mặc định **ẩn** — lỗi 2 của checkpoint vòng 2 plan 03-04.
   *
   * Người dùng báo trình xem diff "chưa full height". Đo bằng Chromium thật cho
   * thấy chuỗi CSS không đứt ở đâu cả: `.app` 900 → `.body` 831.5 → `.layout`
   * 831.5 → `[data-panel=top]` **581.3**. Chỗ tụt 250px là `Panel id="bottom"`,
   * tức panel này, chiếm 30% chiều cao dọc (đo được 249.2px) ngay từ lần mở đầu
   * — trong khi nội dung nó hiện chỉ là "Chưa có lệnh nào được chạy."
   *
   * PLAT-08 đòi người dùng **nhìn thấy được** lệnh git đã chạy; nó không đòi
   * panel mở sẵn. Nút thanh công cụ và `Ctrl+\`` vẫn mở được nên PLAT-08 trọn,
   * còn Core Value ("đọc lịch sử phải tức thì") lấy lại đủ 830px.
   */
  const [logVisible, setLogVisible] = useState(false)
  const [logRefreshKey, setLogRefreshKey] = useState(0)
  const commitListRef = useRef<CommitListHandle>(null)

  // Trạng thái chọn commit nâng cấp lên `selectionStore` (từ `useState` của
  // plan 02-05) — ARCHITECTURE.md Pattern 3: vùng chi tiết (`CommitDetail`)
  // và thanh bên (`RefSidebar`, bấm nhánh để điều hướng) đều cần đọc/ghi
  // cùng id, và cả hai không phải con trực tiếp của nhau trong cây component
  // (đều là con của `App` qua `AppLayout`). Store chỉ giữ id — không dữ liệu
  // commit — nên chọn commit khác không làm `CommitList` (đọc `historyStore`
  // riêng) render lại.
  const selectedCommitId = useSelectionStore((s) =>
    activeRepoId ? (s.selectedByRepo[activeRepoId] ?? null) : null,
  )
  const selectCommit = useSelectionStore((s) => s.select)

  /*
   * Tệp đang chọn quyết định vùng `main` hiện đồ thị hay hiện diff (plan 03-04).
   *
   * **Quyết định bố cục, và vì sao.** `<action>` Task 2 cho hai lựa chọn:
   * (a) diff chiếm vùng `main`, hay (b) thêm một `Panel` thứ tư. Chọn **(a)**.
   *
   * Cách (b) an toàn hơn về hồi quy — không đụng `AppLayout.tsx` vốn vừa qua hai
   * vòng checkpoint — nhưng nó chia bề rộng cửa sổ thành **bốn** phần, và chế độ
   * hai cột cần hai cột nội dung cộng hai cột số dòng trong **một** panel.
   * `<layout_constraints>` mục 4 gọi hẹp là "ca hẹp tệ nhất". Cách (a) cho diff
   * vùng rộng nhất (`main`, `defaultSize="52%"`) mà cũng **không đổi một dòng
   * nào** trong `AppLayout.tsx` — nó chỉ đổi thứ render *bên trong* vùng `main`.
   *
   * Đổi lại: đồ thị bị che khi đang xem diff. Nút "Đóng diff" đưa nó về, và
   * `FileList` ở vùng `detail` vẫn hiện nên người dùng không mất ngữ cảnh commit.
   *
   * 🔴 Đây là **thay đổi bố cục** và Phase 2 cho thấy bố cục là chỗ hay sai nhất
   * — nên nó là **bước 1** của checkpoint để chủ dự án xác nhận hoặc chọn (b).
   */
  const selectedFile = useDiffStore((s) =>
    activeRepoId ? (s.selectedFileByRepo[activeRepoId] ?? null) : null,
  )
  const setSelectedCommitId = (commitId: string) => {
    if (activeRepoId) selectCommit(activeRepoId, commitId)
  }

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
            useSelectionStore.getState().clear(id)
          }
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
          <Logo size={20} />
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
              {activeRepo ? (
                <RefSidebar repoId={activeRepo.info.id} onSelectCommit={setSelectedCommitId} />
              ) : (
                <p className="placeholder">Mở một repository để xem nhánh và tag.</p>
              )}
            </aside>
          }
          main={
            <main className={`pane main${activeRepo ? ' main-history' : ''}`}>
              {activeRepo && selectedFile ? (
                <DiffViewer repoId={activeRepo.info.id} />
              ) : activeRepo ? (
                <div className="main-history-body">
                  <CommitSearch
                    repoId={activeRepo.info.id}
                    scrollToIndex={(index) => commitListRef.current?.scrollToIndex(index)}
                  />
                  <CommitList
                    ref={commitListRef}
                    repoId={activeRepo.info.id}
                    selectedCommitId={selectedCommitId}
                    onSelect={setSelectedCommitId}
                  />
                  {/* Cờ perf tắt → không render và KHÔNG nạp module CodeMirror.
                      `isPerfEnabled()` đọc `localStorage` lúc chạy, nên bật được
                      trên bản release đã dựng mà không phải dựng lại. */}
                  {isPerfEnabled() && (
                    <Suspense fallback={null}>
                      <SpikeHarness repoId={activeRepo.info.id} />
                    </Suspense>
                  )}
                </div>
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
              {activeRepo ? (
                <CommitDetail repoId={activeRepo.info.id} />
              ) : (
                <p className="placeholder">Chọn một repository để xem chi tiết commit.</p>
              )}
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
