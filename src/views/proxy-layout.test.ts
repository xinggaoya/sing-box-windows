import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

/**
 * 代理页布局守护
 *
 * ## 为什么需要这个测试
 *
 * 代理页主体是「定高 + 内部滚动」布局：`.page-shell` 是 flex 列，`.split-view`
 * （代理组侧栏 + 节点面板）靠 `flex: 1` 吃掉剩余高度，节点网格自己内部滚动。
 *
 * 2026-10-20 的提交把「出站总览」做成一张**有内容高度的卡片**直接塞进 `.page-shell`：
 * flex 分配高度时，`.split-view` 的 `min-height: 0` 允许它被压到 0，
 * 结果整页只剩一条缝 + 一张盖住主体的卡片 —— 代理组与节点列表完全不可见。
 *
 * 这类问题编译期、类型检查、单测都发现不了（纯 CSS 布局），只能靠"结构性约定 + 守护测试"：
 * 1. 代理页里**不得**出现任何出站总览的 UI（同层元素 = 抢高度）
 * 2. 出站总览必须有自己的独立页面 + 路由
 * 3. `.split-view` 必须保留保底高度，防止同类回归再次把代理区压扁
 */

const ROOT = resolve(process.cwd())
const PROXY_VIEW = resolve(ROOT, 'src/views/ProxyView.vue')
const PROXY_CSS = resolve(ROOT, 'src/assets/views/ProxyView.css')
const OUTBOUNDS_VIEW = resolve(ROOT, 'src/views/OutboundsView.vue')
const ROUTER = resolve(ROOT, 'src/router/index.ts')
const MAIN_LAYOUT = resolve(ROOT, 'src/components/layout/MainLayout.vue')

const readSrc = (file: string) => readFileSync(file, 'utf8')

describe('代理页布局守护', () => {
  it('代理页不得出现出站总览的 UI（同层元素会抢高度）', () => {
    const view = readSrc(PROXY_VIEW)

    // 只看“渲染了什么”，不看注释里提到什么
    expect(view).not.toMatch(/<Outbound\w+/)
    expect(view).not.toMatch(/useOutboundList/)
    expect(view).not.toContain('<n-drawer')
  })

  it('出站总览必须有独立页面与路由', () => {
    const view = readSrc(OUTBOUNDS_VIEW)
    const router = readSrc(ROUTER)
    const layout = readSrc(MAIN_LAYOUT)

    expect(view, '出站总览页面必须存在并渲染出站列表').toMatch(/outbound-row/)
    expect(router, '必须注册 /outbounds 路由').toContain("path: '/outbounds'")
    expect(layout, '侧边栏必须有「出站」入口').toContain("t('nav.outbounds')")
  })

  it('.split-view 必须有保底高度，避免被同层元素压成一条缝', () => {
    const css = readSrc(PROXY_CSS)
    const rule = css.match(/\.split-view\s*\{[^}]*\}/)?.[0] ?? ''

    expect(rule, '未找到 .split-view 规则').not.toBe('')
    expect(rule, '.split-view 必须声明 min-height 保底值').toMatch(/min-height:\s*\d+px/)
  })
})
