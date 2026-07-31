import pluginVue from 'eslint-plugin-vue'
import { withVueTs, vueTsConfigs } from '@vue/eslint-config-typescript'
import oxlint from 'eslint-plugin-oxlint'
import skipFormatting from '@vue/eslint-config-prettier/skip-formatting'

// withVueTs 组合 Vue + TypeScript 配置并返回一个 thenable（ESLint 10 会自动 await）。
// 所有自定义配置块作为参数传入，由 withVueTs 统一组装为 flat config 数组。
export default withVueTs(
  {
    name: 'app/files-to-lint',
    files: ['**/*.{ts,mts,tsx,vue}'],
  },

  {
    name: 'app/files-to-ignore',
    ignores: [
      '**/dist/**',
      '**/dist-ssr/**',
      '**/coverage/**',
      '**/src-tauri/target/**',
      '**/node_modules/**',
      '**/*.tgz',
      '**/*.tar.gz',
      '**/.nyc_output/**',
      '**/.vscode/**',
      '**/.idea/**',
      '**/.DS_Store',
      '**/Thumbs.db',
      '**/*.log',
      '**/npm-debug.log*',
      '**/yarn-debug.log*',
      '**/yarn-error.log*',
      '**/pnpm-debug.log*',
      '**/pids',
      '**/*.pid',
      '**/*.seed',
      '**/*.pid.lock',
      '**/tmp/**',
      '**/temp/**',
      '**/.env*',
    ],
  },

  pluginVue.configs['flat/essential'],
  vueTsConfigs.recommended,
  {
    name: 'app/no-unused-vars-policy',
    rules: {
      // 允许用 rest 解构排除 sibling 字段（如 const { uriContent, ...base } = form），
      // 并允许以 _ 前缀标记故意忽略的参数/变量。
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          ignoreRestSiblings: true,
        },
      ],
    },
  },
  oxlint.configs['flat/recommended'],
  skipFormatting,
)
