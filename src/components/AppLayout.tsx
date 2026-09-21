/**
 * Bố cục ba vùng có thanh kéo — PLAT-09.
 *
 * `useDefaultLayout` ghi kích thước vào `localStorage`, nên tỉ lệ các vùng còn
 * nguyên ở lần mở ứng dụng sau.
 *
 * Lưu ý về phiên bản: react-resizable-panels v4 đổi tên so với v3 —
 * `Group` thay `PanelGroup`, `Separator` thay `PanelResizeHandle`, và
 * `orientation` thay `direction`. Tài liệu hay ví dụ viết cho v3 sẽ không chạy.
 */

import type { ReactNode } from 'react'
import { Group, Panel, Separator, useDefaultLayout } from 'react-resizable-panels'

interface Props {
  sidebar: ReactNode
  main: ReactNode
  detail: ReactNode
  /** Nhật ký lệnh. Không truyền thì vùng dưới không hiện. */
  bottom?: ReactNode
}

export function AppLayout({ sidebar, main, detail, bottom }: Props) {
  const outerLayout = useDefaultLayout({ groupId: 'git-plum-outer' })
  const innerLayout = useDefaultLayout({ groupId: 'git-plum-inner' })

  const panes = (
    <Group orientation="horizontal" className="pane-group" {...innerLayout}>
      <Panel id="sidebar" defaultSize="20%" minSize="12%" maxSize="40%">
        {sidebar}
      </Panel>

      <Separator className="resize-handle-v" />

      <Panel id="main" defaultSize="52%" minSize="25%">
        {main}
      </Panel>

      <Separator className="resize-handle-v" />

      <Panel id="detail" defaultSize="28%" minSize="15%">
        {detail}
      </Panel>
    </Group>
  )

  if (!bottom) {
    return <div className="layout">{panes}</div>
  }

  return (
    <Group orientation="vertical" className="layout" {...outerLayout}>
      <Panel id="top" defaultSize="70%" minSize="30%">
        {panes}
      </Panel>

      <Separator className="resize-handle-h" />

      <Panel id="bottom" defaultSize="30%" minSize="10%">
        {bottom}
      </Panel>
    </Group>
  )
}
