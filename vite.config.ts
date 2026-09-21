import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// Cổng 1420 khớp với devUrl trong tauri.conf.json.
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // src-tauri do cargo theo dõi, Vite không cần dòm vào.
      ignored: ['**/src-tauri/**'],
    },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'chrome110',
    sourcemap: false,
  },
})
