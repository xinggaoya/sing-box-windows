import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
// 注释掉 vueDevTools 导入以避免生产构建时的 localStorage 错误
// import vueDevTools from 'vite-plugin-vue-devtools'

// https://vite.dev/config/
export default defineConfig(({ mode }) => ({
  plugins: [
    vue(),
    AutoImport({
      resolvers: [NaiveUiResolver()],
    }),
    Components({
      resolvers: [NaiveUiResolver()],
    }),
    // Vue DevTools 已被注释以避免生产构建时的 localStorage 错误
    // 如需在开发时使用，可以取消注释导入并添加条件判断
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    host: '0.0.0.0',
    port: 6221,
    strictPort: true,
  },
  build: {
    chunkSizeWarningLimit: 1000, // 增加警告阈值到 1MB
    rollupOptions: {
      output: {
        manualChunks(id) {
          // 将大型依赖分离到单独的 chunk
          if (id.includes('node_modules')) {
            if (id.includes('vue') || id.includes('pinia')) {
              return 'vendor'
            }
            if (id.includes('naive-ui')) {
              return 'naive-ui'
            }
            if (id.includes('@tauri-apps')) {
              return 'tauri'
            }
          }
        }
      }
    }
  }
}))
