/**
 * Sổ đăng ký lệnh — PLAT-04.
 *
 * Mọi thao tác người dùng gọi được phải đăng ký ở đây, và giao diện gọi thao tác
 * đó **qua sổ đăng ký**, không gắn hàm xử lý thẳng vào `onClick`.
 *
 * Vì sao phải làm ngay từ đầu: bảng lệnh gõ nhanh và phím tắt tuỳ biến (v2) cần
 * một danh sách thao tác có tên ở thời điểm chạy. Nếu thao tác nằm rải rác trong
 * các hàm `onClick`, muốn thêm hai tính năng đó phải sửa lại mọi chỗ xử lý sự
 * kiện trong toàn ứng dụng. Làm đúng bây giờ gần như không tốn gì.
 *
 * Quy ước: `onClick={() => runCommand('repo.open')}`, không phải
 * `onClick={handleOpenRepo}`.
 */

/** Nhóm lệnh, dùng để gom nhóm trong bảng lệnh sau này. */
export type CommandCategory = 'repo' | 'view' | 'history' | 'branch' | 'help'

export interface Command {
  /** Mã định danh ổn định, dạng `nhóm.hành-động`. Không đổi sau khi đặt. */
  id: string
  /** Nhãn hiển thị cho người dùng. */
  title: string
  category: CommandCategory
  /**
   * Phím tắt mặc định, dạng `Ctrl+O`. Người dùng ghi đè được ở v2.
   * Chưa có bộ xử lý phím tắt ở Phase 1 — trường này chỉ để khai báo trước.
   */
  keybinding?: string
  /** Lệnh có gọi được vào lúc này không. Ví dụ: chưa mở repo thì không đóng được. */
  enabled?: () => boolean
  /** Việc thực sự phải làm. */
  run: () => void | Promise<void>
}

const registry = new Map<string, Command>()

/** Đăng ký một lệnh. Trùng mã định danh là lỗi lập trình, không im lặng ghi đè. */
export function registerCommand(command: Command): void {
  if (registry.has(command.id)) {
    throw new Error(`Lệnh đã được đăng ký: ${command.id}`)
  }
  registry.set(command.id, command)
}

export function registerCommands(commands: Command[]): void {
  for (const c of commands) registerCommand(c)
}

/** Gỡ đăng ký. Chủ yếu dùng để dọn dẹp giữa các lần kiểm thử. */
export function unregisterCommand(id: string): void {
  registry.delete(id)
}

export function clearCommands(): void {
  registry.clear()
}

export function getCommand(id: string): Command | undefined {
  return registry.get(id)
}

/** Toàn bộ lệnh đã đăng ký — nguồn dữ liệu cho bảng lệnh gõ nhanh ở v2. */
export function listCommands(): Command[] {
  return [...registry.values()]
}

/** Các lệnh đang gọi được vào lúc này. */
export function listEnabledCommands(): Command[] {
  return listCommands().filter((c) => c.enabled?.() ?? true)
}

/**
 * Chạy một lệnh theo mã định danh.
 *
 * Gọi lệnh chưa đăng ký là lỗi lập trình — ném ra ngay thay vì im lặng không làm
 * gì, để phát hiện lúc phát triển chứ không phải lúc người dùng bấm nút.
 */
export async function runCommand(id: string): Promise<void> {
  const command = registry.get(id)
  if (!command) {
    throw new Error(`Không có lệnh nào mang mã: ${id}`)
  }
  if (command.enabled && !command.enabled()) {
    return
  }
  await command.run()
}
