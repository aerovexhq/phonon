import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { execSync } from 'child_process'

const commitHash = (() => {
  try {
    return execSync('git rev-parse --short HEAD').toString().trim()
  } catch {
    return Date.now().toString(36)
  }
})()

// https://vitejs.dev/config/
export default defineConfig({
  base: '/studio/',
  plugins: [react()],
  define: {
    __BUILD_HASH__: JSON.stringify(commitHash),
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  },
})

