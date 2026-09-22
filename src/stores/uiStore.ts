/**
 * Trạng thái giao diện nhỏ, không gắn với một repo cụ thể — khác
 * `selectionStore`/`historyStore`/`refsStore` (khuôn `byRepo[repoId]`).
 *
 * `fileListView` là lựa chọn hiển thị của NGƯỜI DÙNG (phẳng hay cây), không
 * phải dữ liệu theo repo — giữ nó sống qua việc đổi commit hay đổi repo là
 * đúng ý định UX ("người dùng đã chọn cách xem, không phải chọn lại mỗi
 * lần" — `<behavior>` của Task 2, plan 02-06).
 */

import { create } from 'zustand'

export type FileListView = 'flat' | 'tree'

/** Khoá `localStorage` cho lựa chọn Gravatar. */
const KHOA_GRAVATAR = 'gitPlumGravatar'

/**
 * Người dùng đã bật lấy ảnh avatar từ Gravatar chưa.
 *
 * 🔴 **Mặc định `false`, và đó là một ràng buộc, không phải một mặc định tuỳ ý.**
 *
 * Ràng buộc Privacy của dự án (`CLAUDE.md`): "Không gửi gì ra ngoài khi chưa
 * được cho phép rõ ràng." Mỗi lần fetch Gravatar gửi hash email của **tác giả
 * commit** tới máy chủ bên thứ ba, và qua thời điểm cùng tần suất, nó tiết lộ
 * ai đang đọc repo nào lúc nào. Người dùng không mong đợi một trình xem lịch sử
 * git mở kết nối ra ngoài.
 *
 * # Vì sao `localStorage` chứ không phải `tauri-plugin-store`
 *
 * Dự án dùng cả hai, có phân công: `tauri-plugin-store` cho **dữ liệu** (repo
 * gần đây, bản nháp thông điệp commit), `localStorage` cho **lựa chọn hiển thị
 * trên máy này** (kích thước ba vùng qua `useDefaultLayout` của `AppLayout`, cờ
 * perf trong `lib/perf.ts`). Cờ này thuộc nhóm sau.
 *
 * Hai lý do kỹ thuật, và một lý do quan trọng hơn cả hai:
 *
 * 1. Đọc **đồng bộ** ngay lúc khởi tạo store. `tauri-plugin-store` là bất đồng
 *    bộ, nên sẽ có một khoảnh khắc cờ là `false` rồi nhảy sang `true` — và
 *    khoảnh khắc đó làm avatar nháy từ chữ cái sang ảnh.
 * 2. Không cần đồng bộ hay khôi phục cẩn thận như dữ liệu thật.
 * 3. 🔴 **Hỏng theo hướng an toàn.** Đây là một quyết định về *quyền riêng tư*,
 *    không phải một sở thích hiển thị bình thường. Mất nó — cài lại máy, xoá
 *    site data, đổi máy — đưa người dùng về **mặc định tắt**, tức không gửi gì
 *    ra ngoài cho tới khi họ chọn lại. Một kho lưu trữ bền hơn sẽ khiến một lần
 *    đồng ý cũ theo họ sang bối cảnh mới mà họ không nhớ đã đồng ý.
 *
 * `try/catch` vì `localStorage` ném trong cửa sổ riêng tư và khi site data bị
 * chặn. Ném ở đây sẽ làm cả store không khởi tạo được.
 */
function docCoGravatar(): boolean {
  try {
    return globalThis.localStorage?.getItem(KHOA_GRAVATAR) === '1'
  } catch {
    return false
  }
}

interface UiState {
  fileListView: FileListView
  toggleFileListView: () => void

  /** Đã bật lấy avatar từ Gravatar chưa. Mặc định **tắt**. */
  choPhepGravatar: boolean
  setChoPhepGravatar: (bat: boolean) => void
}

export const useUiStore = create<UiState>((set) => ({
  fileListView: 'flat',

  toggleFileListView: () =>
    set((s) => ({ fileListView: s.fileListView === 'flat' ? 'tree' : 'flat' })),

  choPhepGravatar: docCoGravatar(),

  setChoPhepGravatar: (bat) => {
    // Ghi đĩa trước, đặt state sau — nhưng **không** để lỗi ghi chặn state.
    // Người dùng vừa bấm một công tắc; nó phải phản hồi ngay cả khi
    // `localStorage` không dùng được. Hệ quả duy nhất là lựa chọn không sống
    // qua lần mở sau, và đó là suy giảm chấp nhận được.
    try {
      globalThis.localStorage?.setItem(KHOA_GRAVATAR, bat ? '1' : '0')
    } catch {
      // Cố ý nuốt — xem trên.
    }
    set({ choPhepGravatar: bat })
  },
}))
