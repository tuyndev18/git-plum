/**
 * Danh sách repository gần đây ở màn hình trống — PLAT-07.
 *
 * Đây là nửa sau của tiêu chí thành công số 1 của Phase 1: người dùng mở một
 * repository, đóng ứng dụng, mở lại — repository đó có mặt ở đây và mở ra bằng
 * đúng một lần bấm, không phải đi lại qua hộp thoại chọn thư mục.
 */

import type { RecentRepo } from '@/lib/recentRepos'

interface Props {
  items: RecentRepo[]
  /** Mở một repository theo đường dẫn. Nhận đường dẫn thô đã lưu trong store. */
  onOpen: (path: string) => void
  /** Xoá một mục khỏi danh sách, không đụng tới repository trên đĩa. */
  onForget: (path: string) => void
}

export function RecentRepoList({ items, onOpen, onForget }: Props) {
  // Danh sách rỗng thì không render gì. Màn hình trống đã có nút "Mở
  // repository" rồi; thêm một tiêu đề trên khoảng trắng chỉ là nhiễu.
  if (items.length === 0) return null

  return (
    <section className="recent-repos">
      <h3>Mở gần đây</h3>
      <ul>
        {/* `path` là khoá duy nhất theo thiết kế khử trùng lặp của `mergeRecent`. */}
        {items.map((item) => (
          <li key={item.path}>
            {/*
              Là `<button>` chứ không phải `<div onClick>`: bàn phím tab tới
              được, Enter và Space kích hoạt được, trình đọc màn hình đọc đúng
              là một thao tác gọi được.
            */}
            <button
              className="recent-repo-item"
              onClick={() => onOpen(item.path)}
              title={item.path}
            >
              <span className="recent-repo-name">{item.name}</span>
              <span className="recent-repo-path">{item.path}</span>
            </button>
            <button
              className="recent-repo-forget"
              // Nút nằm cạnh nút mở, không lồng trong nó (HTML không cho lồng
              // button). `stopPropagation` vẫn giữ để nếu sau này bọc cả dòng
              // vào một vùng bấm được thì việc xoá không kéo theo việc mở.
              onClick={(e) => {
                e.stopPropagation()
                onForget(item.path)
              }}
              title={`Xoá ${item.name} khỏi danh sách gần đây`}
              aria-label={`Xoá ${item.name} khỏi danh sách gần đây`}
            >
              ✕
            </button>
          </li>
        ))}
      </ul>
    </section>
  )
}
