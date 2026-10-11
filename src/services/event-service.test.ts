import { beforeEach, describe, expect, it, vi } from 'vitest'

/**
 * 事件服务回归测试。
 *
 * 修复前的致命问题：`removeEventListener(name)`（不带 id）会清空该事件下
 * **所有**监听器。ConnectionStore / LogStore / TrafficStore 都是
 * 「先用自己的 unlisten() 精确移除，再顺手调一次全局清理」，
 * 于是任何第二个订阅者都会在别的 store 退订时被误杀，且没有任何报错。
 *
 * 这类问题靠读代码很难发现，只能用测试钉死。
 */

/** 每次 `listen()` 调用返回的底层 unlisten（记录注册次数） */
const registeredUnlisteners: Array<() => void> = []
/** 底层 unlisten 实际被调用的次数（记录解绑次数） */
let unlistenCalls = 0
let listenSpy: ReturnType<typeof vi.fn>

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

async function freshService() {
  const module = await import('@/services/event-service')
  module.EventService.destroyInstance()
  registeredUnlisteners.length = 0
  unlistenCalls = 0
  return module.EventService.getInstance()
}

beforeEach(async () => {
  const { listen } = await import('@tauri-apps/api/event')
  listenSpy = vi.mocked(listen)
  listenSpy.mockReset()
  listenSpy.mockImplementation(async () => {
    const fn = () => {
      unlistenCalls += 1
    }
    registeredUnlisteners.push(fn)
    return fn
  })
})

describe('EventService - 注册与精确退订', () => {
  it('on 返回的 unlisten 只移除自己那一个监听器', async () => {
    const service = await freshService()
    const calls: string[] = []

    const unlistenA = await service.on('connections-data', () => calls.push('a'))
    await service.on('connections-data', () => calls.push('b'))

    unlistenA()
    // 精确退订：只解绑 a 自己的那一个，b 不受影响
    expect(unlistenCalls).toBe(1)
  })

  it('多次订阅同一事件，各自独立退订', async () => {
    const service = await freshService()
    const un1 = await service.on('log-data', () => {})
    const un2 = await service.on('log-data', () => {})
    const un3 = await service.on('log-data', () => {})

    un2()
    un1()
    un3()
    expect(unlistenCalls).toBe(3)
  })

  it('退订可重复调用而不抛错', async () => {
    const service = await freshService()
    const unlisten = await service.on('traffic-data', () => {})

    unlisten()
    expect(() => unlisten()).not.toThrow()
  })

  it('unlisten 内部抛错时也会被清理并向上暴露不阻断', async () => {
    const service = await freshService()
    const { listen } = await import('@tauri-apps/api/event')
    vi.mocked(listen).mockResolvedValueOnce(() => {
      throw new Error('后端监听已销毁')
    })

    const unlisten = await service.on('memory-data', () => {})
    expect(() => unlisten()).not.toThrow()
  })
})

describe('EventService - 整事件清空是显式 API', () => {
  it('不再暴露会误杀同名订阅者的 removeEventListener', async () => {
    const service = await freshService()
    expect((service as unknown as Record<string, unknown>).removeEventListener).toBeUndefined()
  })

  it('removeAllListenersForEvent 才会清空整事件', async () => {
    const service = await freshService()
    await service.on('connections-data', () => {})
    await service.on('connections-data', () => {})
    await service.on('log-data', () => {})

    service.removeAllListenersForEvent('connections-data')
    expect(unlistenCalls).toBe(2)

    // 再次清空不应重复解绑（Map 已被删除）
    service.removeAllListenersForEvent('connections-data')
    expect(unlistenCalls).toBe(2)
  })

  it('清空后可以重新注册同一事件', async () => {
    const service = await freshService()
    await service.on('tray-runtime-state-updated', () => {})
    service.removeAllListenersForEvent('tray-runtime-state-updated')
    await service.on('tray-runtime-state-updated', () => {})
    expect(registeredUnlisteners).toHaveLength(2)
  })

  it('清空不存在的事件不抛错', async () => {
    const service = await freshService()
    expect(() => service.removeAllListenersForEvent('never-registered')).not.toThrow()
  })
})

describe('EventService - destroy 释放全部监听器', () => {
  it('destroy 会清掉所有已注册监听器', async () => {
    const service = await freshService()
    await service.on('connections-data', () => {})
    await service.on('log-data', () => {})
    await service.on('traffic-data', () => {})

    service.destroy()
    expect(unlistenCalls).toBe(3)
  })

  it('destroy 后重新 getInstance 得到干净实例', async () => {
    const module = await import('@/services/event-service')
    const service = await freshService()
    await service.on('log-data', () => {})

    // destroyInstance 内部会调用 destroy() 解绑
    module.EventService.destroyInstance()
    const afterDestroy = unlistenCalls
    expect(afterDestroy).toBe(1)

    const next = module.EventService.getInstance()
    expect(next).not.toBe(service)

    // 新实例不继承旧实例的监听器：清空不应产生任何额外解绑
    next.removeAllListenersForEvent('log-data')
    expect(unlistenCalls).toBe(afterDestroy)
  })
})

describe('EventService - 专用 on* 方法同样返回精确 unlisten', () => {
  it.each([
    'onTrafficData',
    'onMemoryData',
    'onLogData',
    'onConnectionsData',
    'onKernelReady',
  ] as const)('%s 可用且退订不误杀', async (method) => {
    const service = await freshService()
    const unlisten = await service[method](() => {})
    expect(typeof unlisten).toBe('function')
    expect(() => unlisten()).not.toThrow()
  })
})
