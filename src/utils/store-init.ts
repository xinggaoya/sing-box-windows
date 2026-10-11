import { nextTick } from 'vue'

/**
 * store 初始化期的「抑制自动保存」守卫
 *
 * ## 为什么需要它
 *
 * 各 store 的模式都是：设置若干响应式 ref（配置项），并用 `watch` 自动回写后端。
 * 初始化时需要先把 DB 里的值读进这些 ref，但**不能让 watch 把刚读出来的值又写回去** ——
 * 那会造成启动瞬间的无意义写入，读写并发时还可能用半初始化的值覆盖数据库。
 *
 * 早期实现用 `await new Promise(r => setTimeout(r, 100))`「等数据落地」，属于用时间赌
 * Vue 调度器：机器慢时 100ms 不够（仍然会误保存），机器快时纯属浪费（白等 100ms ×5 个 store）。
 *
 * ## 正确做法
 *
 * Vue 的 `watch` 回调由调度器排队，在 `nextTick` 的微任务里 flush。因此
 * 「加载完成 → `await nextTick()` → 关闭抑制」的顺序能**确定性地**保证：
 * 由本次加载引发的所有 watch 回调都已执行过，它们的抑制守卫必然在 `isInitializing`
 * 仍为 `true` 时被求值，从而正确跳过保存。
 */

/**
 * 在抑制自动保存的上下文中执行初始化逻辑。
 *
 * 关闭抑制前会 `await nextTick()`，确保本次加载引发的 watch 回调已全部跑完
 * （见文件顶部说明）。
 *
 * @param markInitializing 由调用方提供的“进入抑制”回调（如 `() => { isInitializing = true }`）
 * @param markInitialized  由调用方提供的“退出抑制”回调
 * @param load             真正的加载逻辑
 */
export async function withSaveGuard<T>(
  markInitializing: () => void,
  markInitialized: () => void,
  load: () => Promise<T>
): Promise<T> {
  markInitializing()
  try {
    const result = await load()
    // 让本次加载引发的 watch 回调先跑完，再解除抑制
    await nextTick()
    return result
  } finally {
    markInitialized()
  }
}

/**
 * 等待 Vue 调度器把待处理的 watcher / 组件更新回调 flush 完。
 *
 * 与 `nextTick()` 等价，此处单独导出是为了在 store 里读起来意图更明确。
 */
export async function flushWatchers(): Promise<void> {
  await nextTick()
}
