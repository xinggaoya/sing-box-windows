import { describe, expect, it } from 'vitest'
import { readFileSync, readdirSync } from 'node:fs'
import { resolve } from 'node:path'

/**
 * 跨语言类型（ts-rs）漂移守护
 *
 * ## 为什么需要这个测试
 *
 * Rust 侧的 gRPC 数据类型经 ts-rs 导出到 `src/types/generated/`，是前后端的**唯一事实来源**。
 * 在此之前，前端在 `services/proxy-service.ts` 手写了一份同名结构（`GroupItem` / `ProxyGroup` /
 * `GroupsData` / `ClashModeStatus`），后端改字段名时**前端编译不报错**，只在运行时静默拿到
 * `undefined`——这类问题在 UI 上表现为"某个延迟一直是 0"、"模式名不显示"，极难定位。
 *
 * 本测试把最容易复发的三类漂移变成 CI 门禁：
 * 1. 前端又手写了生成类型
 * 2. Rust 加了 `#[serde(rename_all)]` 却忘了同步 `#[ts(rename_all)]`
 * 3. 生成类型里出现 `bigint`（i64/u64 默认映射，与 JSON 实际载荷不符）
 *
 * 关于第 2 条：ts-rs 11 默认开启 `serde-compat`，**会继承** serde 的 rename，
 * 所以"忘了写 `#[ts(rename_all)]`"本身不会立刻产生错误字段名。但 `#[ts(...)]`
 * 优先级更高、且继承行为依赖 feature 开关——两边写法不一致时最终以 `#[ts]` 为准。
 * 这里守的是"**只改一边**"这个真正会出事的情况，因此要求显式声明。
 *
 * @see docs/implementation-plan-2026-10.md 阶段 0
 */

const ROOT = resolve(process.cwd())
const GENERATED_DIR = resolve(ROOT, 'src/types/generated')
const RUST_TYPES = resolve(ROOT, 'src-tauri/src/app/singbox_api/types.rs')
const RUST_STORAGE_DIR = resolve(ROOT, 'src-tauri/src/app/storage')

const readRust = (file: string) => readFileSync(file, 'utf8')

describe('ts-rs 生成类型', () => {
  it('Rust 侧导出路径统一指向 git 跟踪目录 src/types/generated/', () => {
    const sources = [RUST_TYPES, ...readdirSync(RUST_STORAGE_DIR).map((f) => resolve(RUST_STORAGE_DIR, f))]
    const exportPaths: string[] = []

    for (const file of sources) {
      if (!file.endsWith('.rs')) continue
      const content = readRust(file)
      for (const match of content.matchAll(/export_to\s*=\s*"([^"]+)"/g)) {
        exportPaths.push(match[1])
      }
    }

    expect(exportPaths.length).toBeGreaterThan(0)
    for (const p of exportPaths) {
      // ts-rs 的相对路径基于 crate 源码根（src-tauri/src），故 "../../src/types/generated/"
      // 才是仓库根的 src/types/generated/。写成 "../src/..." 会落到被 gitignore 的
      // src-tauri/src/types/generated/（ts-rs v7 时代遗留），导致前后端类型不同步。
      expect(p, `导出路径必须落在 src/types/generated/，实际为 ${p}`).toMatch(
        /^\.\.\/\.\.\/src\/types\/generated\/[A-Za-z0-9_]+\.ts$/,
      )
    }
  })

  it('带 serde rename_all 的结构体必须同步声明 ts-rs rename_all', () => {
    const content = readRust(RUST_TYPES)

    // 逐个结构体块检查：块内出现 serde(rename_all) 就必须出现 ts(rename_all)
    const blocks = content.split(/\n(?=#\[derive)/).filter((b) => b.includes('pub struct') || b.includes('pub enum'))

    expect(blocks.length).toBeGreaterThan(0)

    for (const block of blocks) {
      const hasSerdeRenameAll = /#\[serde\(rename_all\s*=/.test(block)
      if (!hasSerdeRenameAll) continue

      const nameMatch = block.match(/pub (?:struct|enum)\s+(\w+)/)
      expect(nameMatch, '无法解析结构体名').not.toBeNull()
      const name = nameMatch![1]

      // ts-rs 11 会继承 serde 的 rename_all，但 #[ts(...)] 优先级更高、且继承依赖 feature 开关。
      // 显式声明两边一致，生成的 TS 字段名才不会随依赖或写法变化而漂移。
      expect(
        /#\[ts\(rename_all\s*=/.test(block),
        `${name} 有 #[serde(rename_all)] 但缺少 #[ts(rename_all)]，两边写法不一致时最终以 ts 为准，容易与预期字段名不符`,
      ).toBe(true)
    }
  })

  it('带 serde(rename = "...") 的字段必须同步声明 ts(rename = "...")', () => {
    const content = readRust(RUST_TYPES)
    const blocks = content.split(/\n(?=#\[derive)/).filter((b) => b.includes('pub struct'))

    for (const block of blocks) {
      const serdeRenames = [...block.matchAll(/#\[serde\(rename\s*=\s*"([^"]+)"\)\]/g)].map((m) => m[1])
      if (serdeRenames.length === 0) continue

      const nameMatch = block.match(/pub struct\s+(\w+)/)
      const name = nameMatch![1]

      for (const renamed of serdeRenames) {
        expect(
          new RegExp(`#\\[ts\\(rename\\s*=\\s*"${renamed}"\\)\\]`).test(block),
          `${name} 的字段 "${renamed}" 缺少对应的 #[ts(rename = "${renamed}")]`,
        ).toBe(true)
      }
    }
  })

  it('生成类型中不出现 bigint（i64/u64 必须覆写为 number）', () => {
    const files = readdirSync(GENERATED_DIR).filter(
      (f) => f.endsWith('.ts') && f !== 'index.ts' && !f.endsWith('.test.ts'),
    )
    expect(files.length).toBeGreaterThan(0)

    for (const file of files) {
      const content = readFileSync(resolve(GENERATED_DIR, file), 'utf8')
      // JSON 载荷里 i64/u64 就是普通数字，ts-rs 默认映射成 bigint 会让前端类型与实际不符。
      // 只匹配类型位置（`: bigint`），避免误伤文档注释里出现的 "bigint" 字样。
      expect(
        /:\s*bigint\b/.test(content),
        `${file} 出现 bigint，请在 Rust 侧加 #[ts(type = "number")]`,
      ).toBe(false)
    }
  })

  it('前端不再手写 gRPC 响应类型', () => {
    const service = readFileSync(resolve(ROOT, 'src/services/proxy-service.ts'), 'utf8')
    for (const name of ['GroupItem', 'ProxyGroup', 'GroupsData', 'ClashModeStatus']) {
      expect(
        new RegExp(`export interface ${name}\\b`).test(service),
        `proxy-service.ts 又手写了 ${name}，请改用 @/types/generated 下的生成类型`,
      ).toBe(false)
    }
  })

  it('生成类型的 barrel 导出齐全', () => {
    const index = readFileSync(resolve(GENERATED_DIR, 'index.ts'), 'utf8')
    for (const name of [
      'Groups',
      'Group',
      'GroupItem',
      'OutboundList',
      'Rule',
      'RuleList',
      'Service',
      'ServiceList',
      'ServiceStatus',
      'ServiceStatusSnapshot',
      'NetworkQualityResult',
      'DeprecatedWarning',
      'DeprecatedWarnings',
      'ClashModeStatus',
    ]) {
      expect(index.includes(`export type { ${name} }`), `index.ts 缺少 ${name} 导出`).toBe(true)
    }
  })
})
