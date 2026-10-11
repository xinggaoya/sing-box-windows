/**
 * 全局类型出口
 *
 * 历史问题（已修复）：
 * 1. 此前这里是 `export * from './api' / './store' / './models'`，
 *    而 `./store` 与已被删除的 `./api`、`./models` 都指向不存在的文件 ——
 *    一旦有人真正 import 这个 barrel 就会直接构建失败。它之所以没炸，
 *    是因为唯一的引用者恰好是个 0 引用的死组件。
 * 2. `./api`、`./models`、`./controller` 三个文件里的类型（ProxyGroup / ProxyNode /
 *    Subscription / ConnectionMetadata / LogEntry / Rule 等）全部 0 引用，
 *    且与 `services/`、`stores/`、`types/events.ts` 中的同名类型重复且不兼容，
 *    已删除。有效定义只有：
 *    - `ProxyGroup`（proto 结构）→ `services/proxy-service.ts`
 *    - `ProxyGroup`（视图结构）→ `stores/kernel/ProxyStore.ts`
 *    - `Subscription`           → `types/generated/Subscription.ts`（ts-rs 生成）
 *    - `ConnectionMetadata`     → `types/events.ts`
 *    - `LogEntry`               → `stores/kernel/LogStore.ts`
 *    - `ProxyMode` / `TrayCloseBehavior` → `stores/app/composables/appConfigSchema.ts`
 *
 * 现在只保留真正被使用的类型；业务类型请就近定义在 service / store / types 域内，
 * 不要在此堆叠。
 */

// 托盘关闭行为（联合类型由 appConfigSchema 定义并在此转出）
export type { TrayCloseBehavior } from '@/stores/app/composables/appConfigSchema'

/** 代理模式 */
export type ProxyMode = 'system' | 'tun' | 'manual'

/** 状态卡片视觉变体 */
export type StatusCardType = 'default' | 'primary' | 'success' | 'warning' | 'error'
