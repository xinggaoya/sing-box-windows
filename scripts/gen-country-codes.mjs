#!/usr/bin/env node
/**
 * 从 `flag-icons` 包生成 `src/constants/countries.ts` 里的国旗国家码列表。
 *
 * ## 为什么需要生成而不是手写
 *
 * 原先 253 行国家码硬编码在 `ProxyView.vue` 里，且已与 `flag-icons` 实际提供的
 * 文件发生漂移：包里有 16 个码（如 ARAB / ASEAN / GB-ENG）视图里没有，
 * 视图里又有 3 个包已移除的码（AC / EA / TA）。这类不一致会让国旗图标静默不显示。
 *
 * 生成 + 测试守护后，升级 `flag-icons` 时跑一次本脚本即可保持同步。
 *
 * 用法：
 *   node scripts/gen-country-codes.mjs          # 校验并在不一致时报错
 *   node scripts/gen-country-codes.mjs --write  # 重新生成
 */
import { readdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const ROOT = join(__dirname, '..')
const FLAGS_DIR = join(ROOT, 'node_modules', 'flag-icons', 'flags', '4x3')
const TARGET = join(ROOT, 'src', 'constants', 'countries.ts')

const MARKER_START = 'const GENERATED_COUNTRY_CODES: readonly string[] = ['
const MARKER_END = ']'

function readPackageCodes() {
  return readdirSync(FLAGS_DIR)
    .filter((name) => name.endsWith('.svg'))
    .map((name) => name.replace('.svg', '').toUpperCase())
    .sort()
}

function render(codes) {
  const body = codes.map((code) => `  '${code}',`).join('\n')
  return `${MARKER_START}\n${body}\n${MARKER_END}`
}

/// 从源码里取出生成块中的国家码列表（只比内容，不比空白）。
function extractCodesFromSource(source) {
  const start = source.indexOf(MARKER_START)
  if (start === -1) return null
  // 从 MARKER_START 末尾之后开始找闭合的 `]`，否则会命中类型标注里的 `[]`
  const searchFrom = start + MARKER_START.length
  const end = source.indexOf(MARKER_END, searchFrom)
  if (end === -1) return null
  const body = source.slice(searchFrom, end)
  return body
    .split(String.fromCharCode(10))
    .map((line) => line.match(/'([A-Z0-9-]+)'/))
    .filter(Boolean)
    .map((match) => match[1])
}

const packageCodes = readPackageCodes()
const current = readFileSync(TARGET, 'utf8')
const currentCodes = extractCodesFromSource(current)

if (currentCodes === null) {
  console.error(`❌ 未在 ${TARGET} 中找到生成块标记：${MARKER_START}`)
  process.exit(1)
}

if (process.argv.includes('--write')) {
  const start = current.indexOf(MARKER_START)
  const searchFrom = start + MARKER_START.length
  const end = current.indexOf(MARKER_END, searchFrom)
  const next = current.slice(0, searchFrom) + render(packageCodes) + current.slice(end + 1)
  writeFileSync(TARGET, next, 'utf8')
  console.log(`✅ 已写入 ${packageCodes.length} 个国家码到 src/constants/countries.ts`)
  process.exit(0)
}

const onlyInPackage = packageCodes.filter((code) => !currentCodes.includes(code))
const onlyInSource = currentCodes.filter((code) => !packageCodes.includes(code))
if (onlyInPackage.length === 0 && onlyInSource.length === 0) {
  console.log(`✅ 国家码与 flag-icons 一致（${packageCodes.length} 个）`)
  process.exit(0)
}

console.error('❌ 国家码与 flag-icons 不一致：')
if (onlyInPackage.length) console.error(`   flag-icons 有但源码缺: ${onlyInPackage.join(', ')}`)
if (onlyInSource.length) console.error(`   源码有但 flag-icons 无: ${onlyInSource.join(', ')}`)

console.error('❌ 国家码与 flag-icons 不一致，请运行：node scripts/gen-country-codes.mjs --write')
process.exit(1)
