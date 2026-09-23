/**
 * Thanh nút cho **một** khối thay đổi — WORK-03, WORK-05, WORK-06.
 *
 * Hai nút: đưa khối vào / lấy khối khỏi vùng chờ, và huỷ khối. Cộng một băng thông báo
 * WORK-05 khi tệp đã đổi từ lúc vẽ diff.
 *
 * # 🔴 Ràng buộc chi phối: component này KHÔNG phụ thuộc thành phần nào của Phase 4
 *
 * `CONTEXT.md` mục 0, hệ quả 2, nguyên văn:
 *
 * > *"Tầng giao diện của Phase 5 phải chịu được việc Phase 4 còn lỗi. Nếu `ChangeList`
 * > hoá ra hiển thị sai, staging theo khối vẫn phải dùng được từ một đường vào khác."*
 *
 * Và lý do nó **không** phải thận trọng thừa: **không requirement nào của Phase 4 có
 * bằng chứng từ mắt người** — hai cổng của nó còn đỏ, `docs/09-phase4-dogfood.md` chưa
 * tồn tại. Chủ dự án đã tìm **năm** lỗi hiển thị của Phase 3 chỉ bằng cách mở ứng dụng,
 * và cả năm đi qua hàng trăm test tự động. Wave 5 của Phase 4 còn tìm được một **vòng
 * khoá chết bốn điều kiện** mà không test cũ nào thấy: `ChangeList` là chỗ **duy nhất**
 * gọi `refresh`; nó chỉ mount khi vùng soạn commit đã mở; đường **duy nhất** mở vùng
 * soạn là bấm hàng WIP; hàng WIP chỉ hiện khi store đã có dữ liệu.
 *
 * Cách ràng buộc đó được **cài** ở đây, chứ không chỉ được hứa:
 *
 * * **Mọi** dữ liệu vào qua `props`. Không có `useStatusStore`, không có `useDiffStore`,
 *   không có provider nào — nên không có gì phải "đã nạp xong" trước khi nút hiện ra.
 * * Store duy nhất component này đọc là `useHunkStore`, thứ **Phase 5 tự tạo** và có
 *   giá trị đầu hợp lệ (`dangChon: null`, `canLamMoi: null`) ngay lúc module nạp. Không
 *   có trạng thái "chưa nạp" nào để kẹt vào.
 * * **Không** `disabled` nào phụ thuộc dữ liệu ngoài. Nút luôn bấm được.
 * * Component **không** tự gọi `getStatus` hay `getWorktreeDiff` — nạp dữ liệu là việc
 *   của chỗ khác, nên một lời gọi hỏng không nằm trong cùng cây con với nút.
 *
 * Có test ghim: `hunkbar_render_mot_minh_khong_co_phase4` (đột biến **M25**, đột biến
 * quan trọng nhất của plan này).
 *
 * # 🔴 Một câu hỏi plan này PHẢI trả lời thẳng: cái gì mount `HunkBar`?
 *
 * Trả lời trung thực: **ở commit này, không cái gì cả.** Plan 05-04 cố ý dừng trước
 * việc nối vào `App.tsx` — đó là 05-05, nơi có mắt người. Nên ràng buộc trên hiện là
 * *"component này **có thể** sống không cần Phase 4"*, đã ghim bằng test, **chưa** là
 * *"người dùng **có** một đường tới nó không đi qua Phase 4"*. Hai mệnh đề khác nhau,
 * và mệnh đề thứ hai là thứ vòng khoá chết của Phase 4 vi phạm. Xem 05-04-SUMMARY.
 *
 * # ⚠️ Bố cục CHƯA KIỂM CHỨNG
 *
 * happy-dom không tính CSS layout và không có cuộn thật (`CONTEXT.md` 4.5). Mọi khẳng
 * định về **bố cục** ở đây — hai nút không đè lên nhau, thanh không tràn khỏi khối,
 * băng "hãy làm mới" không che mất dòng diff bên dưới, thanh không nhảy chỗ khi băng
 * hiện ra — là **có mã, chưa kiểm** cho tới khi có người xem trên Chromium thật. Việc
 * kiểm mắt người nằm ở checkpoint của plan 05-05.
 *
 * Đây không phải một lời từ chối lấy lệ: lỗi `marginTop`→`paddingTop` của Phase 4 đi
 * qua **cả 666 test** vì không test nào hỏi về thuộc tính đó, và cả năm lỗi hiển thị
 * Phase 3 cũng vậy.
 */

import { nhanDongTu, type DongTu } from '@/lib/ipc'
import { useHunkStore } from '@/stores/hunkStore'

export interface HunkBarProps {
  repoId: string
  path: string
  /** Chỉ số khối trong tệp, **0-based** — cùng đánh số với `hunk_index` phía Rust. */
  index: number
  /**
   * Mã băm blob của tệp **lúc giao diện vẽ diff**.
   *
   * Phía Rust so lại nó ngay trước khi áp và ném `file_changed` khi lệch (WORK-05).
   * Giao diện **không** phải chỗ tin cậy cho phép kiểm này (T-05-15) — nó chỉ chuyển
   * tiếp thứ nó đã thấy, và phép kiểm thật nằm ở nơi bản vá được áp.
   */
  blobHash: string
  /** Quyết định nút là "Đưa khối vào vùng chờ" hay "Lấy khối khỏi vùng chờ". */
  staged: boolean
  /**
   * 🔴 Quyết định **động từ huỷ**: `true` → "Xoá", `false` → "Huỷ bỏ".
   *
   * ROADMAP, không thương lượng. Xem `nhanDongTu` ở `lib/ipc.ts`.
   */
  untracked: boolean
  /**
   * Nạp lại diff của tệp này — chỗ gọi cung cấp.
   *
   * # 🔴 Vì sao là một prop chứ không phải `useDiffStore()` gọi thẳng ở đây
   *
   * Nạp lại diff là việc của **trình xem diff**, và trình xem diff là Phase 4 — thứ
   * chưa ai nhìn. Đọc `diffStore` ở đây sẽ dựng đúng phụ thuộc mà đột biến **M25**
   * tồn tại để cấm: một `HunkBar` không render được vì một store Phase 4 chưa nạp.
   * Nhận qua prop giữ nguyên hướng phụ thuộc — chỗ gọi biết cách nạp lại diff của
   * mình, `HunkBar` chỉ nói *khi nào* cần.
   *
   * # 🔴 `undefined` là ca THẬT, không phải ca lười
   *
   * Không truyền thì nút vẫn hiện và vẫn **gỡ băng đi**, chỉ không nạp lại được gì.
   * Đó là suy giảm đúng: người dùng vẫn thoát được khỏi băng và vẫn bấm lại được
   * (lần bấm sau sẽ lại thất bại nếu họ chưa làm mới thật, và họ sẽ thấy lại băng —
   * không có gì bị mất, không có gì bị áp sai).
   *
   * 🔴 Và nó **chưa được truyền ở đâu cả** ở commit này: 05-04 không nối vào `App.tsx`.
   * Xem 05-04-SUMMARY, mục "cái gì mount HunkBar".
   */
  onLamMoi?: () => void
}

export function HunkBar({
  repoId,
  path,
  index,
  blobHash,
  staged,
  untracked,
  onLamMoi,
}: HunkBarProps) {
  const stageHunk = useHunkStore((s) => s.stageHunk)
  const unstageHunk = useHunkStore((s) => s.unstageHunk)
  const discardHunk = useHunkStore((s) => s.discardHunk)
  const xoaCanLamMoi = useHunkStore((s) => s.xoaCanLamMoi)

  /**
   * Băng chỉ hiện trên khối của **đúng tệp** đang gặp lỗi.
   *
   * `canLamMoi` sống ở cấp store, nên so `path` là bắt buộc: không có nó, mọi `HunkBar`
   * đang mount sẽ hiện "tệp đã đổi" cùng lúc, kể cả những khối của tệp không đổi gì.
   */
  const canLamMoi = useHunkStore((s) =>
    s.canLamMoi?.path === path ? s.canLamMoi : null,
  )

  const dongTu: DongTu = untracked ? 'xoa' : 'huyBo'
  const nhanHuy = nhanDongTu(dongTu)
  const nhanVungCho = staged ? 'Lấy khối khỏi vùng chờ' : 'Đưa khối vào vùng chờ'

  return (
    <div className="hunk-bar" data-testid="hunk-bar">
      <button
        type="button"
        className="hunk-bar-button"
        data-testid="hunk-nut-vung-cho"
        onClick={() => {
          void (staged ? unstageHunk : stageHunk)(repoId, path, index, blobHash)
        }}
      >
        {nhanVungCho}
      </button>

      <button
        type="button"
        className="hunk-bar-button hunk-bar-button-danger"
        data-testid="hunk-nut-huy"
        onClick={() => {
          void discardHunk(repoId, path, index, blobHash)
        }}
      >
        {nhanHuy} khối
      </button>

      {canLamMoi && (
        /*
         * 🔴 `role="alert"` và `data-testid` riêng: phần tử này **chỉ** tồn tại ở
         * trạng thái `file_changed`. Neo test vào vỏ `hunk-bar` — có mặt ở **mọi**
         * trạng thái — là đúng lỗi #6 của `CONTEXT.md` 4.1, thứ đã sống sót một wave
         * với 1 lần đỏ trên 5 lần chạy.
         *
         * `message` là **nguyên văn** từ Rust, nơi câu *"hãy làm mới"* được ghim bằng
         * test (`file_changed_noi_nguyen_van_hay_lam_moi` ở `error.rs`). Ghép câu ở
         * đây sẽ là **hai nguồn sự thật** cho một câu, và chúng lệch nhau được.
         */
        <div className="hunk-bar-can-lam-moi" data-testid="hunk-can-lam-moi" role="alert">
          <span className="hunk-bar-can-lam-moi-text">{canLamMoi.message}</span>
          <button
            type="button"
            className="hunk-bar-button"
            data-testid="hunk-nut-lam-moi"
            onClick={() => {
              // Nạp lại TRƯỚC, gỡ băng SAU. Thứ tự này quan trọng: gỡ băng trước rồi
              // mới nạp lại sẽ để người dùng nhìn một khoảnh khắc không có băng lẫn
              // không có diff mới — trông như cú bấm không làm gì.
              onLamMoi?.()
              xoaCanLamMoi()
            }}
          >
            Làm mới
          </button>
        </div>
      )}
    </div>
  )
}
