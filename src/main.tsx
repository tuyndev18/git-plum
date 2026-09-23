import React from 'react'
import ReactDOM from 'react-dom/client'

// Phông đóng gói cùng app thay vì trông vào máy người dùng: Inter và JetBrains
// Mono hiếm khi được cài sẵn trên Windows, và không tải từ mạng (app chạy
// offline, không gửi gì ra ngoài). Bản variable kèm subset tiếng Việt.
import '@fontsource-variable/inter'
import '@fontsource-variable/jetbrains-mono'

import { App } from '@/App'
import '@/styles/app.css'

const root = document.getElementById('root')
if (!root) throw new Error('Không tìm thấy phần tử #root')

ReactDOM.createRoot(root).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
