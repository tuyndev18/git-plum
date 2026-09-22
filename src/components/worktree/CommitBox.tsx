/**
 * Vùng soạn commit — WORK-08, WORK-09.
 *
 * Textarea + công tắc `no-verify` + chế độ amend + nút. Thao tác đi qua sổ lệnh
 * (PLAT-04) theo khuôn `ChangeList.tsx` / `FileList.tsx`.
 *
 * # 🔴 Ba ràng buộc dễ bị "sửa" ngược, và cả ba có đột biến ghim
 *
 * 1. **`no-verify` mặc định TẮT.** Hook `pre-commit` và `commit-msg` chạy **mặc
 *    định**; bỏ qua chúng là một hành động người dùng phải **tự bật**. Nhãn ô tick nói
 *    rõ nó **bỏ qua hook** — không phải một chữ viết tắt mà chỉ người viết mã hiểu.
 * 2. **Amend trên commit đã push: cảnh báo, KHÔNG chặn.** Cảnh báo là một **dòng chữ
 *    cạnh nút**, không phải hộp thoại. Một hộp thoại "bạn có chắc?" là bước đầu của
 *    việc chặn, và ROADMAP nói không chặn. Nút **không bao giờ** `disabled` vì
 *    `wasPushed` (đột biến M9).
 * 3. **Lỗi hook hiện trong `<pre>`.** Đầu ra hook có xuống dòng và thụt lề **mang
 *    nghĩa** (`src/a.ts:12  thiếu dấu chấm phẩy`). Nhồi vào một dòng `<p>` làm nó khó
 *    đọc đúng lúc người dùng cần đọc nhất, và `white-space: pre-wrap` là thứ giữ nó.
 *
 * # Nút vô hiệu phải NÓI LÝ DO
 *
 * Một nút xám không rõ vì sao là một người dùng bị kẹt — họ không biết phải làm gì để
 * nó sáng lại. Nên mỗi lý do vô hiệu đi kèm một câu hiện trên màn hình, và câu đó nói
 * **hành động tiếp theo** ("hãy stage ít nhất một tệp"), không chỉ nêu tình trạng.
 *
 * # ⚠️ Bố cục CHƯA KIỂM CHỨNG
 *
 * happy-dom không tính CSS layout (`CONTEXT.md` 3.4). Chiều cao textarea, vùng soạn
 * không tràn, nút không bị cắt, khối `<pre>` lỗi hook không đẩy nút ra khỏi khung —
 * tất cả là **có mã, chưa kiểm** cho tới checkpoint 04-05 trên Chromium thật.
 */

import { useEffect } from 'react'

import { registerCommands, runCommand, unregisterCommand } from '@/lib/commands'
import { useCommitStore } from '@/stores/commitStore'
import { useHistoryStore } from '@/stores/historyStore'
import { useStatusStore } from '@/stores/statusStore'

interface Props {
  /** Repo đang hiển thị. Không truyền → không render gì, khuôn `ChangeList`. */
  repoId?: string
}

/**
 * Repo đang chờ lệnh commit/amend đọc.
 *
 * Cấp module, không trong store — nó là **đối số của một lời gọi** sống vài micro giây
 * giữa `onClick` và `run`, không phải trạng thái ứng dụng. Cùng khuôn và cùng lý do
 * với `pendingSelection` của `ChangeList.tsx`.
 */
let pendingCommit: { repoId: string } | null = null

/**
 * Làm mới đồ thị sau khi commit thành công — tiêu chí 3.
 *
 * `historyStore` **không** có action `invalidate`, và plan này cố ý **không thêm một
 * cái**: `App.tsx` và `CommitList.tsx` tiêu thụ store đó và đang mang thay đổi chưa
 * commit của một session khác. `reset` + `loadFirstPage` là đường **đã có**.
 *
 * ⚠️ Nếu cặp này gây nháy màn hình trên Chromium thật, đó là việc của checkpoint
 * 04-05 xác nhận — không phải chỗ để tự tối ưu bằng một action mới ở wave này.
 */
async function lamMoiDoThi(repoId: string): Promise<void> {
  useHistoryStore.getState().reset(repoId)
  await useHistoryStore.getState().loadFirstPage(repoId)
}

export function CommitBox({ repoId }: Props) {
  const draft = useCommitStore((s) => (repoId ? (s.draftByRepo[repoId] ?? '') : ''))
  const isCommitting = useCommitStore((s) => s.isCommitting)
  const error = useCommitStore((s) => s.error)
  const errorIsHookOutput = useCommitStore((s) => s.errorIsHookOutput)
  const noVerify = useCommitStore((s) => s.noVerify)
  const amendMode = useCommitStore((s) => s.amendMode)
  const wasPushed = useCommitStore((s) => s.wasPushed)
  const setDraft = useCommitStore((s) => s.setDraft)
  const setNoVerify = useCommitStore((s) => s.setNoVerify)
  const setAmendMode = useCommitStore((s) => s.setAmendMode)
  const hydrate = useCommitStore((s) => s.hydrate)

  const status = useStatusStore((s) => (repoId ? s.byRepo[repoId]?.status : undefined))

  useEffect(() => {
    registerCommands([
      {
        id: 'commit.create',
        title: 'Tạo commit từ các tệp đã stage',
        category: 'repo',
        run: async () => {
          const cho = pendingCommit
          if (!cho) return
          pendingCommit = null
          const ok = await useCommitStore.getState().commit(cho.repoId)
          // Đồ thị chỉ được làm mới khi commit **thật sự** được tạo. Làm mới sau một
          // hook từ chối là một lệnh git thừa cho một lịch sử không đổi.
          if (ok) await lamMoiDoThi(cho.repoId)
        },
      },
      {
        id: 'commit.amend',
        title: 'Sửa commit gần nhất',
        category: 'repo',
        run: async () => {
          const cho = pendingCommit
          if (!cho) return
          pendingCommit = null
          const ok = await useCommitStore.getState().amend(cho.repoId)
          if (ok) await lamMoiDoThi(cho.repoId)
        },
      },
    ])
    return () => {
      unregisterCommand('commit.create')
      unregisterCommand('commit.amend')
    }
  }, [])

  // Nạp nháp bền vững khi mở repo — nửa "đóng mở lại ứng dụng" của tiêu chí 6.
  useEffect(() => {
    if (repoId) void hydrate(repoId)
  }, [repoId, hydrate])

  if (!repoId) return null

  const soTepDaStage = status?.entries.filter((e) => e.group === 'staged').length ?? 0
  const thongDiepRong = draft.trim() === ''

  /*
   * 🔴 Danh sách lý do vô hiệu — và `wasPushed` KHÔNG có mặt ở đây, có chủ ý.
   *
   * Đột biến M9 thêm nó vào. "Cảnh báo, không chặn" nghĩa là cảnh báo sống ở phần
   * hiển thị, không ở phép tính `disabled`.
   *
   * `amend` **không** đòi có tệp đã stage: sửa riêng thông điệp của commit gần nhất
   * là ca dùng thường nhất của amend, và nó không cần tệp nào mới.
   */
  const lyDoVoHieu: string | null = thongDiepRong
    ? 'Hãy viết thông điệp commit trước.'
    : !amendMode && soTepDaStage === 0
      ? 'Hãy stage ít nhất một tệp trước khi commit.'
      : null

  const voHieu = lyDoVoHieu !== null || isCommitting

  function bamNut(): void {
    if (voHieu) return
    pendingCommit = { repoId: repoId! }
    void runCommand(amendMode ? 'commit.amend' : 'commit.create')
  }

  const nhanNut = amendMode ? 'Sửa commit gần nhất' : 'Commit'

  return (
    <div className="commit-box" data-testid="commit-box">
      <textarea
        className="commit-message"
        data-testid="commit-message"
        aria-label="Thông điệp commit"
        placeholder="Thông điệp commit"
        value={draft}
        onChange={(e) => setDraft(repoId, e.target.value)}
      />

      <div className="commit-options">
        <label className="commit-option">
          <input
            type="checkbox"
            data-testid="amend-toggle"
            checked={amendMode}
            onChange={(e) => setAmendMode(e.target.checked)}
          />
          Sửa commit gần nhất (amend)
        </label>

        {/*
         * Nhãn nói rõ nó **bỏ qua hook**, không phải chữ viết tắt `no-verify` mà chỉ
         * người viết mã hiểu. Người dùng phải đọc được hậu quả trước khi bật.
         */}
        <label className="commit-option">
          <input
            type="checkbox"
            data-testid="no-verify-toggle"
            checked={noVerify}
            onChange={(e) => setNoVerify(e.target.checked)}
          />
          Bỏ qua hook pre-commit và commit-msg (no-verify)
        </label>
      </div>

      {/*
       * 🔴 Cảnh báo amend — một dòng chữ cạnh nút, KHÔNG phải hộp thoại chặn đường.
       * Nút bên dưới không đọc `wasPushed`. Đột biến M9.
       */}
      {amendMode && wasPushed && (
        <p className="commit-warning" role="status" data-testid="amend-warning">
          Commit vừa sửa đã được đẩy lên nhánh thượng nguồn. Lịch sử người khác đã thấy
          vừa bị viết lại — họ sẽ cần `git pull --rebase`.
        </p>
      )}

      {error !== null &&
        (errorIsHookOutput ? (
          // Đầu ra hook có xuống dòng và thụt lề mang nghĩa — `<pre>` giữ chúng.
          <pre className="commit-hook-error" role="alert" data-testid="commit-hook-error">
            {error}
          </pre>
        ) : (
          <p className="commit-error" role="alert" data-testid="commit-error">
            {error}
          </p>
        ))}

      <div className="commit-actions">
        {lyDoVoHieu !== null && (
          // Nút xám không rõ vì sao là một người dùng bị kẹt.
          <span className="commit-disabled-reason" data-testid="commit-disabled-reason">
            {lyDoVoHieu}
          </span>
        )}
        <button
          type="button"
          className="commit-button"
          data-testid="commit-button"
          disabled={voHieu}
          onClick={bamNut}
        >
          {isCommitting ? 'Đang commit…' : nhanNut}
        </button>
      </div>
    </div>
  )
}
