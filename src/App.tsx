import { lazy, Suspense, useEffect, useRef, useState } from 'react'
import { open } from '@tauri-apps/plugin-dialog'

import { describeError, ipc } from '@/lib/ipc'
import { isPerfEnabled } from '@/lib/perf'
import { clearCommands, registerCommands, runCommand } from '@/lib/commands'
import { forgetRepo } from '@/lib/recentRepos'
import { useActiveRepo, useRepoStore } from '@/stores/repoStore'
import { useHistoryStore } from '@/stores/historyStore'
import { useSelectionStore } from '@/stores/selectionStore'
import { useUiStore } from '@/stores/uiStore'
import { AppLayout } from '@/components/AppLayout'
import {
  IconClose,
  IconFolderOpen,
  IconSpinner,
  IconAvatar,
  IconTerminal,
} from '@/components/icons'
import { CommandLogPanel } from '@/components/CommandLogPanel'
import { Logo } from '@/components/Logo'
import { RecentRepoList } from '@/components/RecentRepoList'
import { RefSidebar } from '@/components/RefSidebar'
import { CommitList, type CommitListHandle } from '@/components/history/CommitList'
import { CommitDetail } from '@/components/history/CommitDetail'
import { CommitSearch } from '@/components/history/CommitSearch'
import { DiffViewer } from '@/components/diff/DiffViewer'
import { ChangeList } from '@/components/worktree/ChangeList'
import { CommitBox } from '@/components/worktree/CommitBox'
import { WorktreePane } from '@/components/worktree/WorktreePane'
import { useDiffStore } from '@/stores/diffStore'
import { useCommitStore } from '@/stores/commitStore'
import { noiWatcherVaoStore, useStatusStore } from '@/stores/statusStore'

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

  const choPhepGravatar = useUiStore((s) => s.choPhepGravatar)
  const setChoPhepGravatar = useUiStore((s) => s.setChoPhepGravatar)

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

  /*
   * Vùng soạn commit đang mở hay không — WORK-11 bước 8 của checkpoint.
   *
   * `useState` cấp `App` chứ không phải một store: đây là trạng thái **của một khung
   * nhìn**, không khoá theo repo và không ai ngoài cây này đọc. `selectionStore` tồn
   * tại vì `RefSidebar` và `CommitDetail` — hai nhánh anh em — cùng cần một id; ở đây
   * chỉ có `CommitList` (ghi) và vùng `detail` (đọc), và cả hai là con trực tiếp của
   * `App`. Thêm một store cho nó là thêm một nguồn sự thật không ai cần.
   *
   * Mở lại về `false` khi đổi repo: vùng soạn của repo A không được nằm mở sẵn khi
   * người dùng vừa chuyển sang repo B. Nháp thì `commitStore.switchRepo` lo giữ.
   */
  const [commitBoxOpen, setCommitBoxOpen] = useState(false)

  /**
   * Vùng `main` đang hiện **đồ thị commit** hay **thay đổi chưa commit** (Phase 5).
   *
   * # 🔴 Vì sao đây là một công tắc ở thanh công cụ, KHÔNG phải một nút trong `DiffToolbar`
   *
   * Plan 05-05 viết *"bật/tắt bằng một nút trong `DiffToolbar` (\"Danh sách khối\")"*.
   * Làm đúng chữ đó sẽ dựng lại **chính** vòng khoá chết mà cả plan tồn tại để tránh:
   *
   * `DiffToolbar` render bên trong `DiffViewer`; `DiffViewer` chỉ render khi
   * `selectedFile` **và** `selectedCommitId` đều có (`App.tsx` dòng "activeRepo &&
   * selectedFile", và `DiffViewer` tự `return` sớm khi thiếu `selectedCommitId`); đường
   * duy nhất đặt `selectedFile` cho thư mục làm việc là bấm một hàng của `ChangeList`;
   * `ChangeList` chỉ mount khi vùng soạn commit đã mở; vùng soạn chỉ mở khi bấm hàng
   * WIP. **Năm** điều kiện — nhiều hơn cái vòng bốn điều kiện của Phase 4 một bậc, và
   * mắt xích đầu tiên (`DiffViewer` cần một commit đang chọn) là thứ làm nó **không
   * bao giờ** mở được cho diff thư mục làm việc.
   *
   * Nút ở thanh công cụ có **một** điều kiện: có repo đang mở. Xem `WorktreePane.tsx`,
   * mục "ba câu hỏi vòng khoá chết", và `05-05-SUMMARY.md`.
   *
   * # Vì sao `main` chứ không thêm panel thứ tư
   *
   * Cùng lập luận và cùng tiền lệ với `DiffViewer` (03-04) và vùng soạn commit
   * (04-05): đổi thứ render **bên trong** một vùng, **không** đổi một dòng nào trong
   * `AppLayout.tsx`. Vùng `main` là vùng rộng nhất, và bảng khối cần bề ngang — nó
   * hiện nội dung từng khối cạnh một thanh nút.
   *
   * 🔴 Đây là **thay đổi bố cục**, và cả Phase 2 lẫn Phase 3 cho thấy bố cục là chỗ
   * hay sai nhất. Nó là bước của checkpoint Task 3 để chủ dự án xác nhận.
   */
  const [xemThayDoi, setXemThayDoi] = useState(false)

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

  /**
   * Bấm một nhánh/tag ở thanh bên: chọn commit **và cuộn đồ thị tới nó**.
   *
   * Vì sao không gộp vào `setSelectedCommitId`: `CommitList` cũng gọi hàm đó khi
   * người dùng bấm thẳng một hàng. Cuộn ở đó sẽ kéo hàng vừa bấm — vốn đang nằm
   * ngay dưới con trỏ — về giữa khung, tức giao diện tự nhảy dưới tay người
   * dùng. Chỉ đường vào từ thanh bên mới cần cuộn, nên nó có handler riêng.
   *
   * Trả về `false` khi commit đích nằm ngoài phần lịch sử đã nạp (nhánh cũ hơn
   * 1000 commit đầu). Khi đó commit **vẫn được chọn** — `CommitDetail` hiện
   * được nhờ `ipc.getCommitDetail`, không cần hàng nào trong danh sách — và
   * `RefSidebar` nói ra rằng nó không cuộn được, thay vì im lặng không làm gì.
   */
  const handleSelectRef = (commitId: string): boolean => {
    setSelectedCommitId(commitId)
    // Đang xem diff thì vùng `main` không render `CommitList` (xem ghi chú bố
    // cục ở trên), nên `commitListRef.current` là `null`. Đó không phải lỗi:
    // báo "không cuộn được" là đúng trạng thái người dùng đang thấy.
    return commitListRef.current?.scrollToCommit(commitId) ?? false
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

  /**
   * Nối watcher `.git` của Rust vào `statusStore` — WORK-10, tiêu chí 5.
   *
   * # Vì sao **một lần lúc gắn kết**, không theo `activeRepoId`
   *
   * `noiWatcherVaoStore` đăng ký **một** người nghe cho **mọi** repo: sự kiện mang
   * `repoId` của chính nó (xem doc comment của hàm). Đặt `activeRepoId` vào mảng phụ
   * thuộc sẽ huỷ và đăng ký lại mỗi lần đổi repo — n lần ghi store cho một sự kiện,
   * đúng thứ hàm đó được viết ra để tránh.
   *
   * # 🔴 Huỷ đăng ký phải chịu được việc unmount xảy ra TRƯỚC khi promise xong
   *
   * `noiWatcherVaoStore` trả `Promise<() => void>`. Ở chế độ Strict của React 19,
   * effect chạy → dọn → chạy lại **đồng bộ**, nên lần dọn đầu tiên xảy ra khi promise
   * còn đang bay và chưa có hàm huỷ nào để gọi. Cờ `daHuy` ghi lại ý định đó, và
   * `.then` tự huỷ ngay khi hàm huỷ về tay — nếu không, người nghe của lần gắn kết
   * thứ nhất sống sót vĩnh viễn và mỗi sự kiện ghi store hai lần.
   *
   * Lỗi đăng ký watcher **không** dựng banner lỗi: watcher là tiện ích tự làm mới, và
   * mọi đường ghi (`stage`/`unstage`/`commit`) vẫn tự cập nhật store từ giá trị trả
   * về. Mất watcher nghĩa là phải bấm làm mới, không phải ứng dụng hỏng.
   */
  useEffect(() => {
    let daHuy = false
    let huy: (() => void) | null = null

    void noiWatcherVaoStore()
      .then((fn) => {
        if (daHuy) fn()
        else huy = fn
      })
      .catch(() => {
        // Nuốt có chủ ý — xem doc comment trên.
      })

    return () => {
      daHuy = true
      huy?.()
    }
  }, [])

  /**
   * Mở repo → nạp `RepoStatus` lần đầu, và xả/nạp nháp commit.
   *
   * # 🔴 Vì sao `App` phải tự gọi `refresh`, dù `ChangeList` cũng gọi
   *
   * Bản nối dây đầu tiên để phép gọi này cho `ChangeList` lo, với lý lẽ "gọi thêm ở
   * `App` là một tiến trình `git status` thừa". Lý lẽ đó **sai**, và ba test đỏ chỉ
   * đúng chỗ: `ChangeList` chỉ được gắn kết khi vùng soạn **đang mở**, mà đường duy
   * nhất mở vùng soạn là **bấm hàng WIP**, mà hàng WIP chỉ hiện khi `statusStore` đã
   * có `RepoStatus`. Ba điều kiện đó khoá vòng vào nhau: mở một repo có thay đổi
   * chưa commit thì hàng WIP **không bao giờ** xuất hiện, nên không bấm được, nên
   * `ChangeList` không bao giờ mount, nên không ai gọi `refresh`.
   *
   * Hàng WIP là thứ WORK-11 đòi người dùng thấy **ngay khi mở repo** — nó không phải
   * phần thưởng cho việc đã mở vùng soạn. Nên chủ sở hữu phép nạp đầu tiên là vòng
   * đời **repo** (`App`), không phải vòng đời của một component có thể chưa tồn tại.
   *
   * Phép gọi thứ hai của `ChangeList` lúc nó mount **không** sinh tiến trình thừa:
   * `statusStore` gộp các `refresh` đang bay theo `repoId` (`dangBay`), và nếu lời
   * gọi trước đã xong thì đọc lại trạng thái lúc mở một bảng tệp là đúng đắn — người
   * dùng có thể đã sửa tệp ở terminal trong lúc đó.
   *
   * 🔴 `switchRepo`, **không** `hydrate` thẳng: `commitStore.ts` nói rõ mọi đường đổi
   * repo phải đi qua nó, vì nó `flushDraft()` **trước**. Gọi `hydrate` thẳng làm những
   * ký tự gõ ngay trước lúc chuyển repo mất cùng phép ghi trì hoãn bị huỷ.
   */
  useEffect(() => {
    if (!activeRepoId) return
    setCommitBoxOpen(false)
    // Cùng lý do với `commitBoxOpen`: khung nhìn của repo A không nằm mở sẵn cho
    // repo B. `WorktreePane` cũng tự bỏ chọn tệp khi `repoId` đổi (xem tệp đó).
    setXemThayDoi(false)
    void useStatusStore.getState().refresh(activeRepoId)
    void useCommitStore.getState().switchRepo(activeRepoId)
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
        {/*
          Nút là ICON, chữ nằm ở `title` + `aria-label` — xem doc comment của
          `DiffToolbar.tsx` cho lý do đầy đủ. Ở đây chữ ngắn hơn nên sức ép bề
          ngang nhẹ hơn, nhưng hai thanh công cụ dùng hai kiểu nút khác nhau thì
          giao diện trông như hai ứng dụng ghép lại.
        */}
        <div className="toolbar-right">
          <button
            className="icon-button"
            onClick={() => runCommand('repo.open')}
            disabled={isOpening}
            title={isOpening ? 'Đang mở…' : 'Mở repository'}
            aria-label={isOpening ? 'Đang mở…' : 'Mở repository'}
          >
            {isOpening ? <IconSpinner /> : <IconFolderOpen />}
          </button>
          {activeRepo && (
            <button
              className="icon-button"
              onClick={() => runCommand('repo.close')}
              title="Đóng repository"
              aria-label="Đóng repository"
            >
              <IconClose />
            </button>
          )}
          {activeRepo && (
            /*
             * 🔴 Công tắc "Thay đổi chưa commit" — ĐƯỜNG VÀO của staging theo khối.
             *
             * Nút có CHỮ chứ không icon, cố ý và ngược với quy ước của thanh này.
             *
             * Tiêu chí thành công 4 hỏi *"bạn **tìm thấy nó** mà không phải hỏi nó ở
             * đâu?"*, và bước 8 của checkpoint hỏi đúng câu đó cho danh sách "Vừa huỷ
             * gần đây" — thứ chỉ tới được **qua** nút này. Một icon không nhãn cho
             * đường vào **duy nhất** tới cả staging theo khối lẫn danh sách vừa huỷ là
             * đặt cược tiêu chí đó vào việc người dùng rê chuột đúng chỗ.
             *
             * `aria-pressed` vì đây là công tắc hai trạng thái, không phải một lệnh.
             */
            <button
              className="toolbar-toggle"
              data-testid="toolbar-xem-thay-doi"
              aria-pressed={xemThayDoi}
              onClick={() => setXemThayDoi((v) => !v)}
              title={xemThayDoi ? 'Về đồ thị commit' : 'Xem thay đổi chưa commit theo từng khối'}
            >
              {xemThayDoi ? 'Đồ thị' : 'Thay đổi'}
            </button>
          )}
          <button
            className="icon-button"
            onClick={() => runCommand('view.toggleCommandLog')}
            aria-pressed={logVisible}
            title="Nhật ký lệnh"
            aria-label="Nhật ký lệnh"
          >
            <IconTerminal />
          </button>
          {/*
            Công tắc Gravatar — **mặc định tắt**, và tooltip phải nói ra điều
            đang thực sự xảy ra.

            Ràng buộc Privacy của dự án đòi người dùng cho phép **rõ ràng**
            trước khi có gì gửi ra ngoài. Một công tắc ghi "Avatar" thoả mãn
            chữ nhưng không thoả mãn tinh thần: người dùng bật nó mà không biết
            mình vừa cho phép gửi hash email của tác giả tới một máy chủ bên
            thứ ba. Nên nhãn nói thẳng gravatar.com.
          */}
          <button
            className="icon-button"
            onClick={() => setChoPhepGravatar(!choPhepGravatar)}
            aria-pressed={choPhepGravatar}
            title={
              choPhepGravatar
                ? 'Tắt ảnh đại diện từ gravatar.com'
                : 'Bật ảnh đại diện từ gravatar.com (gửi hash email tác giả ra ngoài)'
            }
            aria-label={
              choPhepGravatar
                ? 'Tắt ảnh đại diện từ gravatar.com'
                : 'Bật ảnh đại diện từ gravatar.com'
            }
          >
            <IconAvatar />
          </button>
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
                <RefSidebar repoId={activeRepo.info.id} onSelectCommit={handleSelectRef} />
              ) : (
                <p className="placeholder">Mở một repository để xem nhánh và tag.</p>
              )}
            </aside>
          }
          main={
            <main className={`pane main${activeRepo ? ' main-history' : ''}`}>
              {/*
                🔴 `WorktreePane` đứng TRƯỚC `DiffViewer` trong chuỗi điều kiện, và
                thứ tự đó là một ràng buộc, không phải một sở thích.

                Đặt nó sau nghĩa là `selectedFile` (state của Phase 3) quyết định
                người dùng có thấy được vùng staging theo khối hay không — tức một
                lựa chọn tệp còn sót lại từ trình xem diff **che mất** đường vào
                thứ hai. `CONTEXT.md` mục 0 hệ quả 2 nói thẳng rằng đường này phải
                dùng được **kể cả khi** tầng Phase 3/4 hỏng, nên nó không được nằm
                sau một điều kiện do tầng đó đặt ra.

                Hệ quả ngược lại — bật "Thay đổi" thì trình xem diff bị che — là
                đánh đổi ĐÚNG chiều: nút "Đồ thị" đưa nó về ngay, và đó là cùng
                đánh đổi mà 03-04 và 04-05 đã chọn cho vùng này.
              */}
              {activeRepo && xemThayDoi ? (
                <WorktreePane repoId={activeRepo.info.id} />
              ) : activeRepo && selectedFile ? (
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
                    onOpenCommitBox={() => setCommitBoxOpen(true)}
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
            /*
             * **Quyết định bố cục của Task 2, và vì sao.**
             *
             * Vùng soạn commit + ba nhóm tệp chiếm vùng `detail`, **thay** chi tiết
             * commit, và — như tiền lệ 03-04 đã ghi ngay trên — điều đó đổi thứ render
             * *bên trong* một vùng chứ **không đổi một dòng nào** trong `AppLayout.tsx`.
             *
             * Vì sao `detail` chứ không phải `main`: vùng `main` đang là đồ thị, và
             * hàng WIP — chỗ vào của cả vòng commit — **nằm trong** đồ thị đó. Đặt vùng
             * soạn vào `main` sẽ che mất chính hàng vừa bấm để mở nó, và bước 5 của
             * checkpoint (hàng WIP thẳng cột, cuộn lên xuống) sẽ không quan sát được
             * cùng lúc với vùng soạn. `detail` giữ đồ thị nguyên vẹn bên trái trong khi
             * người dùng stage và gõ thông điệp bên phải — hai thứ họ nhìn qua lại
             * liên tục trong một vòng commit.
             *
             * Đổi lại: chi tiết commit bị che khi vùng soạn mở. Nút "Đóng" đưa nó về,
             * và đó là **cùng** đánh đổi mà diff viewer của 03-04 đã chọn cho `main` —
             * nên giao diện không mọc thêm một kiểu điều hướng thứ hai.
             *
             * 🔴 Đây là **thay đổi bố cục**, và Phase 3 có 5 lỗi hiển thị lọt qua 435
             * test vì happy-dom không tính CSS layout. Nên đây là **bước 1** của
             * checkpoint Task 3 để chủ dự án xác nhận hoặc yêu cầu đổi sang `main`.
             */
            <aside className="pane detail">
              {activeRepo && commitBoxOpen ? (
                <>
                  <div className="detail-header">
                    <h2>Thay đổi chưa commit</h2>
                    <button
                      className="icon-button"
                      onClick={() => setCommitBoxOpen(false)}
                      title="Đóng vùng soạn commit"
                      aria-label="Đóng vùng soạn commit"
                    >
                      <IconClose />
                    </button>
                  </div>
                  {/*
                    `ChangeList` tự gọi `statusStore.refresh(repoId)` lúc gắn kết và
                    khi `repoId` đổi, nên `App` **không** gọi thêm một lần nữa: đó sẽ
                    là một tiến trình `git status` thứ hai cho cùng một trạng thái.
                    `dangBay` trong store gộp hai lời gọi trùng, nhưng dựa vào nó để
                    che một lời gọi thừa là dựa vào một chi tiết cài đặt.
                  */}
                  <ChangeList repoId={activeRepo.info.id} />
                  <CommitBox repoId={activeRepo.info.id} />
                </>
              ) : activeRepo ? (
                <>
                  <h2>Chi tiết</h2>
                  <CommitDetail repoId={activeRepo.info.id} />
                </>
              ) : (
                <>
                  <h2>Chi tiết</h2>
                  <p className="placeholder">Chọn một repository để xem chi tiết commit.</p>
                </>
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
