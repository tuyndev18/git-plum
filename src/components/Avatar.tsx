/**
 * Avatar tác giả — vòng tròn chữ cái đầu, tuỳ chọn phủ ảnh Gravatar lên.
 *
 * Mọi quyết định về *nguồn ảnh* và *quyền riêng tư* nằm ở `src/lib/avatar.ts`;
 * tệp này chỉ dựng DOM. Đọc doc comment ở đó trước khi đổi hành vi mạng.
 */

import { useEffect, useState } from 'react'

import { avatarTuSinh, urlGravatar } from '@/lib/avatar'
import { ipc } from '@/lib/ipc'
import { useUiStore } from '@/stores/uiStore'

interface Props {
  ten: string
  email: string
  /** Cạnh ô vuông, px. */
  co?: number
}

export function Avatar({ ten, email, co = 16 }: Props) {
  const { chu, mauNen } = avatarTuSinh(ten, email)
  const choPhepGravatar = useUiStore((s) => s.choPhepGravatar)

  /*
   * Hash lấy qua IPC, KHÔNG phải một trường trên `Commit`.
   *
   * Xem `src-tauri/src/commands/avatar.rs`: gắn hash vào mỗi bản ghi commit
   * thêm ~3,2MB JSON trên repo 100k, cho một tính năng mặc định tắt.
   *
   * 🔴 Effect này **không chạy** khi `choPhepGravatar` là `false` — cờ nằm
   * trong điều kiện thoát sớm chứ không chỉ trong nhánh render. Đặt nó ở chỗ
   * khác nghĩa là ứng dụng vẫn gọi IPC (và Rust vẫn băm email) cho một tính
   * năng người dùng chưa bật. Không rò ra mạng, nhưng vẫn là làm việc mà người
   * dùng không yêu cầu, và nó che mất ý định khi ai đó đọc lại mã.
   */
  const [hash, setHash] = useState<string | null>(null)

  useEffect(() => {
    if (!choPhepGravatar) {
      setHash(null)
      return
    }

    let huy = false
    ipc
      .avatarHash(email)
      .then((h) => {
        if (!huy) setHash(h)
      })
      .catch(() => {
        // Không có hash -> không có ảnh -> avatar tự sinh. Đây không phải lỗi
        // đáng báo cho người dùng: họ mất một ảnh trang trí, không mất dữ liệu.
        if (!huy) setHash(null)
      })

    return () => {
      huy = true
    }
  }, [email, choPhepGravatar])

  /*
   * `anhHong` chứ không phải `anhXong`: trạng thái mặc định là "chưa biết", và
   * ta chỉ cần biết **một** điều — ảnh có hỏng không.
   *
   * Vì sao cần state thay vì để `<img>` tự xử: `onError` của `<img>` không xoá
   * chính nó khỏi DOM, nó để lại một ô vỡ. Muốn lui về avatar tự sinh thì phải
   * gỡ hẳn thẻ `img`, mà việc đó cần một lần render nữa.
   */
  const [anhHong, setAnhHong] = useState(false)

  /*
   * Đổi tác giả -> quên kết quả cũ.
   *
   * Thiếu effect này thì một lần 404 sẽ **dính vĩnh viễn** vào vị trí đó trong
   * cây React: React tái sử dụng component cho hàng khác khi cuộn, nên tác giả
   * thứ hai kế thừa `anhHong = true` của tác giả thứ nhất và ảnh của họ không
   * bao giờ được thử. Lỗi này chỉ lộ ra khi cuộn, đúng lớp lỗi mà `maxLane`
   * vừa mắc.
   */
  useEffect(() => {
    setAnhHong(false)
  }, [hash])

  const hienAnh = choPhepGravatar && !!hash && !anhHong

  return (
    <span
      className="avatar"
      style={{ width: co, height: co, background: mauNen, fontSize: Math.round(co * 0.44) }}
      // Tên đầy đủ trong tooltip: chữ cái đầu không đủ để biết ai, và cột tác
      // giả có thể đã cắt tên bằng ellipsis.
      title={`${ten} <${email}>`}
    >
      {hienAnh ? (
        <img
          className="avatar-img"
          src={urlGravatar(hash, co)}
          width={co}
          height={co}
          // Ảnh là trang trí: `title` của thẻ bọc ngoài đã mang tên đầy đủ, nên
          // một `alt` có tên nữa làm trình đọc màn hình đọc hai lần.
          alt=""
          // 404 (`d=404` trong URL) -> lui về chữ cái đầu bên dưới.
          onError={() => setAnhHong(true)}
          // Không gửi trang đang xem sang gravatar.com. Người dùng đã đồng ý
          // cho tra ảnh theo hash email, không đồng ý cho biết gì thêm.
          referrerPolicy="no-referrer"
          loading="lazy"
        />
      ) : (
        chu
      )}
    </span>
  )
}
