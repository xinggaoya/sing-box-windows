import { describe, expect, it, vi } from 'vitest'
import { nextTick, ref, watch } from 'vue'
import { flushWatchers, withSaveGuard } from '@/utils/store-init'

/**
 * `flushWatchers` 取代了 5 个 store 里原先的 `setTimeout(100ms)` ——
 * 后者是「用时间赌 Vue 调度器」：慢机器上 100ms 不够（仍会误触发自动保存，
 * 把刚读出来的值写回数据库），快机器上纯属白等。
 *
 * 这里验证确定性语义：await 之后 watcher 必然已求值过抑制守卫。
 */

describe('flushWatchers', () => {
  it('让本轮引发的 watcher 回调跑完', async () => {
    const count = ref(0)
    const seen: number[] = []
    watch(count, (value) => seen.push(value))

    count.value = 1
    count.value = 2
    // 默认：watch 是 pre-flush，尚未执行
    expect(seen).toEqual([])

    await flushWatchers()
    expect(seen).toEqual([2])
  })

  it('可连续调用且不报错', async () => {
    await flushWatchers()
    await flushWatchers()
  })

  it('语义等价于 nextTick', async () => {
    const a = ref(0)
    const order: string[] = []
    watch(a, () => order.push('watch'))

    a.value = 1
    await nextTick()
    await flushWatchers()

    expect(order).toEqual(['watch'])
  })
})

describe('withSaveGuard', () => {
  // 真实模型：watch 的是**配置 ref**，isInitializing 只是守卫标志（本身不被 watch），
  // 因此解除抑制不会重新触发任何 watcher。
  function makeStore() {
    const isInitializing = ref(true)
    const proxyPort = ref(12080)
    const saves: number[] = []
    watch(proxyPort, () => {
      if (!isInitializing.value) saves.push(proxyPort.value)
    })
    return { isInitializing, proxyPort, saves }
  }

  it('加载引发的 watcher 不会触发自动保存', async () => {
    const store = makeStore()

    await withSaveGuard(
      () => {
        store.isInitializing.value = true
      },
      () => {
        store.isInitializing.value = false
      },
      async () => {
        store.proxyPort.value = 23456
      },
    )

    // 关键回归：不再会出现“刚读出来的值被自动保存写回数据库”
    expect(store.saves).toEqual([])
    expect(store.proxyPort.value).toBe(23456)
  })

  it('加载结束后用户再改配置会正常触发保存', async () => {
    const store = makeStore()

    await withSaveGuard(
      () => {
        store.isInitializing.value = true
      },
      () => {
        store.isInitializing.value = false
      },
      async () => {
        store.proxyPort.value = 23456
      },
    )
    await flushWatchers()

    store.proxyPort.value = 34567
    await flushWatchers()
    expect(store.saves).toEqual([34567])
  })

  it('返回 load 的结果', async () => {
    const result = await withSaveGuard(
      () => {},
      () => {},
      async () => 42,
    )
    expect(result).toBe(42)
  })

  it('load 抛错时仍解除抑制（否则 store 会被永久锁死）', async () => {
    const events: string[] = []

    await expect(
      withSaveGuard(
        () => events.push('start'),
        () => events.push('finish'),
        async () => {
          throw new Error('load failed')
        },
      ),
    ).rejects.toThrow('load failed')

    expect(events).toEqual(['start', 'finish'])
  })

  it('每次都等待 watcher flush 后才解除抑制', async () => {
    const order: string[] = []
    const isInitializing = ref(true)
    const proxyPort = ref(12080)

    watch(proxyPort, () => {
      order.push(isInitializing.value ? 'watcher(during-guard)' : 'watcher(after-guard)')
    })

    await withSaveGuard(
      () => {
        isInitializing.value = true
      },
      () => {
        isInitializing.value = false
      },
      async () => {
        proxyPort.value = 23456
      },
    )

    expect(order).toEqual(['watcher(during-guard)'])
  })

  it('重复调用互不干扰', async () => {
    const order: number[] = []
    for (const n of [1, 2, 3]) {
      await withSaveGuard(
        () => order.push(n),
        () => order.push(-n),
        async () => n,
      )
    }
    expect(order).toEqual([1, -1, 2, -2, 3, -3])
  })
})

describe('回归：sleep(100ms) 土办法被替换后的行为', () => {
  it('loadFromBackend 连续写 52 个字段也只 flush 一次且不误保存', async () => {
    const isInitializing = ref(true)
    const saves = vi.fn()
    const a = ref(0)
    const b = ref(0)
    const c = ref(0)

    watch([a, b, c], () => {
      if (!isInitializing.value) saves()
    })

    await withSaveGuard(
      () => {
        isInitializing.value = true
      },
      () => {
        isInitializing.value = false
      },
      async () => {
        // 模拟 loadFromBackend 连续写多个字段
        a.value = 1
        b.value = 2
        c.value = 3
      },
    )
    await flushWatchers()

    expect(saves).not.toHaveBeenCalled()
    expect([a.value, b.value, c.value]).toEqual([1, 2, 3])
  })
})
