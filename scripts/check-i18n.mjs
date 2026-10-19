#!/usr/bin/env node
/**
 * i18n key 同步校验
 *
 * 以 zh-CN 为基准，检查其余语言包是否：
 *   - 缺失 key（缺失会让用户直接看到裸 key，如 `home.memory`）
 *   - 多余 key（通常是旧路径改名后遗留的死翻译）
 *
 * 用法：
 *   node scripts/check-i18n.mjs           # 校验并以退出码表示结果
 *   node scripts/check-i18n.mjs --fix    # 打印建议补齐的 key 骨架
 *
 * 同时被 CI 与本地 `pnpm i18n:check` 使用。
 */
import { readdirSync, readFileSync } from 'node:fs'
import { join, dirname, basename } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const LOCALES_DIR = join(__dirname, '..', 'src', 'locales')

/** 作为基准的语言（必须最完整） */
const BASE_LOCALE = 'zh-CN'

/** 把 `{ a: { b: 1 } }` 拍平成 `['a.b']`；数组与原始值视为叶子节点 */
function flattenKeys(value, prefix = '') {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    return prefix ? [prefix] : []
  }
  return Object.entries(value).flatMap(([key, child]) =>
    flattenKeys(child, prefix ? `${prefix}.${key}` : key)
  )
}

function loadLocale(file) {
  const source = readFileSync(file, 'utf8')
  // locale 文件导出形如 `export default { ... }`，需要求值出对象。
  // 用 Function 构造器而不是 import，以便同时兼容 .ts 后缀的源码文件。
  const body = source.replace(/^\s*export\s+default\s+/m, 'return ')
   
  return new Function(body)()
}

function listLocaleFiles() {
  return readdirSync(LOCALES_DIR)
    .filter((name) => name.endsWith('.ts') && name !== 'index.ts')
    .map((name) => join(LOCALES_DIR, name))
}

const files = listLocaleFiles()
if (files.length === 0) {
  console.error('❌ 未找到任何 locale 文件')
  process.exit(1)
}

const baseFile = files.find((file) => basename(file, '.ts') === BASE_LOCALE)
if (!baseFile) {
  console.error(`❌ 未找到基准语言包 ${BASE_LOCALE}`)
  process.exit(1)
}

const baseKeys = new Set(flattenKeys(loadLocale(baseFile)))
const results = []
let hasFailure = false

for (const file of files) {
  const name = basename(file, '.ts')
  if (name === BASE_LOCALE) continue

  const keys = new Set(flattenKeys(loadLocale(file)))
  const missing = [...baseKeys].filter((key) => !keys.has(key)).sort()
  const extra = [...keys].filter((key) => !baseKeys.has(key)).sort()

  results.push({ name, missing, extra })
  if (missing.length > 0) hasFailure = true
}

console.log(`基准语言: ${BASE_LOCALE}（${baseKeys.size} 个叶子 key）\n`)

for (const { name, missing, extra } of results) {
  const status = missing.length === 0 ? '✅' : '❌'
  console.log(`${status} ${name}：缺失 ${missing.length} / 多余 ${extra.length}`)

  if (missing.length > 0) {
    console.log('   缺失 key（用户会看到裸 key）:')
    for (const key of missing) console.log(`     - ${key}`)
  }
  if (extra.length > 0) {
    console.log('   多余 key（疑似改名后的死翻译）:')
    for (const key of extra) console.log(`     + ${key}`)
  }
  console.log()
}

if (process.argv.includes('--fix')) {
  console.log('提示：在对应语言包中补齐上述缺失 key，并删除多余 key。')
}

/**
 * 校验「组件里实际用到的 key」是否存在于基准语言包。
 *
 * 为什么需要这一项：跨语言 key 对齐只能保证"4 个语言包彼此一致"，
 * 但如果所有语言包**同时漏掉**某个 key，它照样能通过对齐检查——
 * 而组件里 `t('rules.routeRules')` 就会直接把 key 原文渲染给用户。
 *
 * 历史教训：`rules.routeRules` / `rules.empty` / `rules.columnType` 等 7 个 key
 * 实际被放在语言包顶层（与 `rules` 平级），而组件用 `t('rules.xxx')` 引用，
 * 规则页对所有语言的用户都显示裸 key。本次一并修复并加入门禁。
 */
const SRC_DIR = join(__dirname, '..', 'src')

/** 递归收集 src/ 下的 .ts / .vue 文件（排除生成目录与测试文件） */
function listSourceFiles(dir) {
  const out = []
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    if (entry.name === 'node_modules' || entry.name === 'generated') continue
    const full = join(dir, entry.name)
    if (entry.isDirectory()) {
      out.push(...listSourceFiles(full))
    } else if (/\.(ts|vue)$/.test(entry.name) && !entry.name.endsWith('.test.ts')) {
      out.push(full)
    }
  }
  return out
}

/**
 * 抽取静态的 `t('...')` / `i18n.t('...')` 调用里的 key。
 *
 * 只处理字面量：动态 key（模板字符串、变量）无法静态判断，跳过。
 */
function extractUsedKeys(source) {
  const keys = new Set()
  const patterns = [/\bt\(\s*'([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)+)'/g, /\bt\(\s*"([a-zA-Z0-9_]+(?:\.[a-zA-Z0-9_]+)+)"/g]
  for (const pattern of patterns) {
    for (const match of source.matchAll(pattern)) {
      keys.add(match[1])
    }
  }
  return keys
}

const usedKeys = new Map()
for (const file of listSourceFiles(SRC_DIR)) {
  const source = readFileSync(file, 'utf8')
  for (const key of extractUsedKeys(source)) {
    if (!usedKeys.has(key)) usedKeys.set(key, [])
    usedKeys.get(key).push(file)
  }
}

const unknownKeys = [...usedKeys.entries()]
  .filter(([key]) => !baseKeys.has(key))
  .sort(([a], [b]) => a.localeCompare(b))

console.log(`组件实际引用: ${usedKeys.size} 个 key（基准语言包中不存在 ${unknownKeys.length} 个）\n`)

if (unknownKeys.length > 0) {
  hasFailure = true
  console.log('❌ 引用了基准语言包中不存在的 key（用户会看到裸 key）:')
  for (const [key, files] of unknownKeys) {
    console.log(`     - ${key}`)
    for (const file of files) console.log(`         ${file.replace(join(__dirname, '..'), '.')}`)
  }
  console.log()
}

process.exit(hasFailure ? 1 : 0)