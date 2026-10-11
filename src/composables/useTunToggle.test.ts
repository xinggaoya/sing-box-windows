import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { nextTick, ref } from 'vue'

/**
 * TUN 切换状态机的回归测试。
 *
 * 这段逻辑此前在 HomeView（3 处）与 TrayStore（3 处）各有变体，回滚顺序、
 * sudo 重试次数、状态归并各不相同。历史上该区域的 bug 几乎都是「某条路径忘了回滚」，
 * 而 TUN 回滚失败会让用户直接失去网络出口 —— 因此这里对回滚路径做穷举。
 *
 * 用依赖注入代替 mock 模块：用真实的 AppStore / KernelStore，只替换它们的方法。
 */

const state = vi.hoisted(() => ({
  tunEnabled: false,
  applyResult: true,
  restartResult: true,
  applyThrows: false,
  lastError: null as string | null,
  startupDiagnosisSummary: null as string | null,
  applyCalls: [] as Array<Record<string, unknown> | undefined>,
  restartCalls: 0,
  saveCalls: 0,
  stopCalls: 0,
  sudoCodes: [] as string[],
  sudoResult: true,
  privilegeCalls: 0,
}))

// 必须返回**稳定单例**：若每次调用新建对象，
// 测试里的 `spyOn(useKernelStore(), ...)` 就会作用在临时对象上而失效。
vi.mock('@/stores/app/AppStore', () => {
  const appStore = {
    get tunEnabled() {
      return state.tunEnabled
    },
    set tunEnabled(value: boolean) {
      state.tunEnabled = value
    },
    toggleTun: async (enabled: boolean) => {
      state.tunEnabled = enabled
    },
    saveToBackend: async () => {
      state.saveCalls += 1
    },
    isRunning: false,
  }
  return { useAppStore: () => appStore, __appStore: appStore }
})

vi.mock('@/stores/kernel/KernelStore', () => {
  const kernelStore = {
    get startupDiagnosisSummary() {
      return state.startupDiagnosisSummary
    },
    get lastError() {
      return state.lastError
    },
    applyProxySettings: async (overrides?: Record<string, unknown>) => {
      state.applyCalls.push(overrides)
      if (state.applyThrows) throw new Error('invoke 炸了')
      return state.applyResult
    },
    restartKernel: async () => {
      state.restartCalls += 1
      return state.restartResult
    },
    stopKernel: async () => {
      state.stopCalls += 1
    },
  }
  return { useKernelStore: () => kernelStore, __kernelStore: kernelStore }
})

function resetState() {
  state.tunEnabled = false
  state.applyResult = true
  state.restartResult = true
  state.applyThrows = false
  state.lastError = null
  state.startupDiagnosisSummary = null
  state.applyCalls = []
  state.restartCalls = 0
  state.saveCalls = 0
  state.stopCalls = 0
  state.sudoCodes = []
  state.sudoResult = true
  state.privilegeCalls = 0
}

const messages = { applyFailed: '应用代理配置失败', restartFailed: '内核重启失败' }

async function makeComposable(
  options: Partial<Parameters<typeof import('@/composables/useTunToggle').useTunToggle>[0]> = {},
) {
  const module = await import('@/composables/useTunToggle')
  return module.useTunToggle({
    messages,
    handleSudoFailure: async (code) => {
      state.sudoCodes.push(code)
      return state.sudoResult
    },
    ...options,
  })
}

beforeEach(() => {
  setActivePinia(createPinia())
  resetState()
  vi.clearAllMocks()
})

describe('useTunToggle - 成功路径', () => {
  it('开启 TUN：置位 -> 应用配置 -> 重启内核', async () => {
    const { applyTunSwitch } = await makeComposable()
    const outcome = await applyTunSwitch(true)

    expect(outcome).toEqual({ status: 'success' })
    expect(state.tunEnabled).toBe(true)
    expect(state.restartCalls).toBe(1)
  })

  it('applyOverrides 被透传给 applyProxySettings', async () => {
    const { applyTunSwitch } = await makeComposable({
      applyOverrides: (target) => ({ tun_enabled: target }),
    })
    await applyTunSwitch(true)

    expect(state.applyCalls).toEqual([{ tun_enabled: true }])
  })

  it('目标状态与当前一致时跳过且不产生副作用', async () => {
    state.tunEnabled = true
    const { applyTunSwitch } = await makeComposable()
    const outcome = await applyTunSwitch(true)

    expect(outcome).toEqual({ status: 'skipped' })
    expect(state.applyCalls).toHaveLength(0)
    expect(state.restartCalls).toBe(0)
  })
})

describe('useTunToggle - 回滚路径（关键）', () => {
  it('applyProxySettings 失败时回滚开关', async () => {
    state.tunEnabled = false
    state.applyResult = false
    const { applyTunSwitch } = await makeComposable()

    const outcome = await applyTunSwitch(true)

    expect(outcome.status).toBe('failed')
    expect(state.tunEnabled).toBe(false)
    expect(state.restartCalls).toBe(0)
  })

  it('restartKernel 失败时回滚开关', async () => {
    state.tunEnabled = false
    state.restartResult = false
    const { applyTunSwitch } = await makeComposable()

    const outcome = await applyTunSwitch(true)

    expect(outcome.status).toBe('failed')
    expect(state.tunEnabled).toBe(false)
  })

  it('关闭 TUN 失败时回滚为「开启」', async () => {
    state.tunEnabled = true
    state.restartResult = false
    const { applyTunSwitch } = await makeComposable()

    await applyTunSwitch(false)
    expect(state.tunEnabled).toBe(true)
  })

  it('applyProxySettings 抛异常时同样回滚', async () => {
    state.tunEnabled = false
    state.applyThrows = true
    const { applyTunSwitch } = await makeComposable()

    const outcome = await applyTunSwitch(true)
    expect(outcome.status).toBe('failed')
    expect(state.tunEnabled).toBe(false)
  })

  it('失败时给出非空文案', async () => {
    state.restartResult = false
    const { applyTunSwitch } = await makeComposable()

    const outcome = await applyTunSwitch(true)
    expect(outcome.status).toBe('failed')
    if (outcome.status === 'failed') {
      expect(outcome.failure.text.length).toBeGreaterThan(0)
    }
  })

  it('pending 在结束后复位（不会把 UI 永久锁死）', async () => {
    state.restartResult = false
    const { applyTunSwitch, pending } = await makeComposable()

    expect(pending.value).toBe(false)
    await applyTunSwitch(true)
    expect(pending.value).toBe(false)
  })
})

describe('useTunToggle - sudo 密码重试', () => {
  it('密码错误会触发一次重试；重试成功则不回滚', async () => {
    // 第一次重启失败（带 sudo 错误码），重试时成功
    state.lastError = 'SUDO_PASSWORD_REQUIRED'
    state.restartResult = false
    const { __kernelStore } = (await import('@/stores/kernel/KernelStore')) as unknown as {
      __kernelStore: { restartKernel: () => Promise<boolean> }
    }
    const restartSpy = vi.spyOn(__kernelStore, 'restartKernel').mockImplementation(async () => {
      state.restartCalls += 1
      return state.restartCalls > 1
    })

    state.tunEnabled = false
    const { applyTunSwitch } = await makeComposable()

    const outcome = await applyTunSwitch(true)

    expect(outcome).toEqual({ status: 'success' })
    expect(state.sudoCodes).toEqual(['required'])
    expect(state.tunEnabled).toBe(true)
    expect(restartSpy).toHaveBeenCalledTimes(2)
  })

  it('密码错误且用户放弃时回滚', async () => {
    state.lastError = 'SUDO_PASSWORD_INVALID'
    state.restartResult = false
    state.sudoResult = false

    state.tunEnabled = false
    const { applyTunSwitch } = await makeComposable()

    const outcome = await applyTunSwitch(true)

    expect(outcome.status).toBe('failed')
    expect(state.tunEnabled).toBe(false)
    expect(state.sudoCodes).toEqual(['invalid'])
  })

  it('不提供 handleSudoFailure 时不重试，直接回滚', async () => {
    state.lastError = 'SUDO_PASSWORD_REQUIRED'
    state.restartResult = false

    state.tunEnabled = false
    const { applyTunSwitch } = await makeComposable({ handleSudoFailure: undefined })

    const outcome = await applyTunSwitch(true)
    expect(outcome.status).toBe('failed')
    expect(state.tunEnabled).toBe(false)
    expect(state.sudoCodes).toEqual([])
  })
})

describe('useTunToggle - 提权前置', () => {
  it('escalated 时不执行本地状态机', async () => {
    const { applyTunSwitch } = await makeComposable({
      ensurePrivilege: async () => {
        state.privilegeCalls += 1
        return 'escalated'
      },
    })

    const outcome = await applyTunSwitch(true)

    expect(outcome).toEqual({ status: 'escalated' })
    expect(state.privilegeCalls).toBe(1)
    expect(state.applyCalls).toHaveLength(0)
    expect(state.restartCalls).toBe(0)
  })

  it('proceed 时继续执行状态机', async () => {
    const { applyTunSwitch } = await makeComposable({
      ensurePrivilege: async () => {
        state.privilegeCalls += 1
        return 'proceed'
      },
    })

    const outcome = await applyTunSwitch(true)
    expect(outcome).toEqual({ status: 'success' })
    expect(state.privilegeCalls).toBe(1)
  })

  it('提权检查在「状态未变」的短路之前执行（提权本身可能有副作用）', async () => {
    state.tunEnabled = true
    const { applyTunSwitch } = await makeComposable({
      ensurePrivilege: async () => {
        state.privilegeCalls += 1
        return 'proceed'
      },
    })

    await applyTunSwitch(true)
    expect(state.privilegeCalls).toBe(1)
  })
})

describe('prepareTunAndRestartAsAdmin', () => {
  it('成功：落库 -> 停内核 -> 请求重启', async () => {
    const module = await import('@/composables/useTunToggle')
    const systemService = await import('@/services/system-service')
    const restartAsAdmin = vi
      .spyOn(systemService.systemService, 'restartAsAdmin')
      .mockResolvedValue()

    state.tunEnabled = false
    const ok = await module.prepareTunAndRestartAsAdmin()

    expect(ok).toBe(true)
    expect(state.tunEnabled).toBe(true)
    expect(state.saveCalls).toBe(1)
    expect(restartAsAdmin).toHaveBeenCalled()
  })

  it('失败时回滚开关', async () => {
    const module = await import('@/composables/useTunToggle')
    const systemService = await import('@/services/system-service')
    vi.spyOn(systemService.systemService, 'restartAsAdmin').mockRejectedValue(new Error('denied'))

    state.tunEnabled = false
    const ok = await module.prepareTunAndRestartAsAdmin()

    expect(ok).toBe(false)
    expect(state.tunEnabled).toBe(false)
  })
})

// 避免 unused 导入告警
void ref
void nextTick
