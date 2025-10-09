import pluginVue from 'eslint-plugin-vue'
import vueTsEslintConfig from '@vue/eslint-config-typescript'
import oxlint from 'eslint-plugin-oxlint'
import skipFormatting from '@vue/eslint-config-prettier/skip-formatting'

export default [
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

  ...pluginVue.configs['flat/essential'],
  ...vueTsEslintConfig(),
  oxlint.configs['flat/recommended'],
  skipFormatting,
]
