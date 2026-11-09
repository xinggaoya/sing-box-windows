import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'

/**
 * 侧边栏入口守护
 *
 * ## 为什么需要这个测试
 *
 * 升级 sing-box 1.14 官方 gRPC API 时，规则页的侧边栏入口被注释掉了，理由是
 * 「官方 API 未暴露规则管理接口」——但**同一次迁移就已经接好了 `get_rules`**，
 * v2.4.0 又补上了规则页的「自定义规则」面板，功能、路由、页面全都在，
 * 唯独入口没跟着恢复。用户反馈「2.4.0 主界面看不到规则页」（issue #69）。
 *
 * 这类回归编译期、类型检查都发现不了（入口被注释掉仍然是合法 TS），
 * 只能靠「结构性约定 + 守护测试」把它钉死。
 */

const ROOT = resolve(process.cwd())
const MAIN_LAYOUT = resolve(ROOT, 'src/components/layout/MainLayout.vue')
const ROUTER = resolve(ROOT, 'src/router/index.ts')
const RULES_VIEW = resolve(ROOT, 'src/views/RulesView.vue')

const readSrc = (file: string) => readFileSync(file, 'utf8')

/**
 * 取出侧边栏 `menuItems` 里**真正生效**的菜单项。
 *
 * 不能直接 `toContain("t('nav.rules')")` —— 被注释掉的那一行同样包含这个字符串，
 * 守护测试就形同虚设（这正是当初入口被注释掉却没人发现的原因）。
 * 这里只认「去掉注释后以 `{ label:` 开头的行」。
 */
function activeMenuEntries(source: string): string[] {
  const block = source.match(/const menuItems = computed<NavItem\[\]>\(\(\) => \[([\s\S]*?)\n\]\)/)
  if (!block) return []
  return block[1]
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.startsWith('{ label:'))
}

describe('侧边栏入口守护', () => {
  it('规则页入口必须真实存在于侧边栏菜单项中（不得被注释掉）', () => {
    const layout = readSrc(MAIN_LAYOUT)
    const entries = activeMenuEntries(layout)

    expect(entries.length, '未能解析出侧边栏菜单项').toBeGreaterThan(0)
    expect(
      entries.some((entry) => entry.includes("key: 'rules'")),
      '侧边栏缺少「规则」入口（issue #69）',
    ).toBe(true)
    expect(
      entries.some((entry) => entry.includes("t('nav.rules')")),
      "规则入口必须使用本地化文案 t('nav.rules')",
    ).toBe(true)
  })

  it('规则页入口的图标必须已导入', () => {
    const layout = readSrc(MAIN_LAYOUT)
    const importBlock = layout.match(/import \{([\s\S]*?)\} from '@vicons\/ionicons5'/)

    expect(importBlock, '未找到 @vicons/ionicons5 的导入块').not.toBeNull()
    expect(importBlock?.[1]).toContain('AnalyticsOutline')
  })

  it('规则页路由、页面与跳转映射必须齐备', () => {
    const router = readSrc(ROUTER)
    const layout = readSrc(MAIN_LAYOUT)

    expect(existsSync(RULES_VIEW), '规则页视图必须存在').toBe(true)
    expect(router, '必须注册 /rules 路由').toContain("path: '/rules'")
    expect(layout, 'currentMenu 必须把 /rules 映射到 rules').toContain("'/rules': 'rules'")
    expect(layout, '点击菜单必须跳转到 /rules').toContain("rules: '/rules'")
  })
})
