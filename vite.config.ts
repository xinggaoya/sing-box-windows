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
      '/node_modules/pinia-plugin-persistedstate/',
      '/node_modules/@vueuse/',
      '/node_modules/mitt/',
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
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks,
      },
    },
  },
}))
