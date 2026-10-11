import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'

const vendorChunkGroups: Array<[string, string[]]> = [
  [
    'vendor-vue',
    [
      '/node_modules/vue/',
      '/node_modules/@vue/',
      '/node_modules/vue-router/',
      '/node_modules/vue-i18n/',
      '/node_modules/pinia/',
      '/node_modules/@vueuse/',
    ],
  ],
  [
    'vendor-naive-ui',
    [
      '/node_modules/naive-ui/',
      '/node_modules/vueuc/',
      '/node_modules/vooks/',
      '/node_modules/vdirs/',
      '/node_modules/css-render/',
      '/node_modules/@css-render/',
      '/node_modules/treemate/',
      '/node_modules/evtd/',
      '/node_modules/async-validator/',
      '/node_modules/date-fns/',
    ],
  ],
  ['vendor-icons', ['/node_modules/@vicons/', '/node_modules/ionicons/']],
  ['vendor-tauri', ['/node_modules/@tauri-apps/']],
]

const manualChunks = (id: string) => {
  const normalizedId = id.replace(/\\/g, '/')
  if (!normalizedId.includes('/node_modules/')) return undefined

  return vendorChunkGroups.find(([, packages]) =>
    packages.some((packagePath) => normalizedId.includes(packagePath)),
  )?.[0]
}

// https://vite.dev/config/
export default defineConfig(({ mode: _mode }) => ({
  plugins: [
    vue(),
    AutoImport({
      resolvers: [NaiveUiResolver()],
    }),
    Components({
      resolvers: [NaiveUiResolver()],
    }),
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
    watch: {
      // Rust 编译产物可能被 Windows 锁定，且不属于前端源码。
      ignored: ['**/src-tauri/target/**'],
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },
  // Vitest：与 vite 共用别名 / 插件，保证测试环境与运行时一致。
  // 环境选 jsdom —— 项目大量使用 `useLocalStorage`（@vueuse）与 DOM API。
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.ts'],
    globals: false,
    restoreMocks: true,
    coverage: {
      provider: 'v8',
      include: ['src/**/*.{ts,vue}'],
      exclude: ['src/**/*.test.ts', 'src/types/generated/**', 'src/main.ts', 'src/locales/**'],
      reporter: ['text-summary'],
    },
  },
}))
