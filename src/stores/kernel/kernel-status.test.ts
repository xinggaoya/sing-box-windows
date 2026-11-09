import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useKernelStore } from '@/stores/kernel/KernelStore'

/**
 * 内核「已安装 / 运行中」语义的回归测试（issue #70）。
 *
 * ## 为什么需要这个测试
 *
 * 此前设置页用**运行中**内核的版本号（gRPC GetVersion）判断「内核是否已安装」。
 * 内核一停，这个值就是 null，界面随之从「已安装 v1.14.4」跳回「未安装」——
 * 用户反馈「更新内核成功了也一直提示未安装」的直接成因。
 *
 * 同样的坑还有第二个：状态整体展开合并（`{...status, ...next}`）时，
 * 后端回传的 `version: null` 会把**已经知道的**版本号一起抹掉，
 * 于是「刚下载完内核」的好消息在 3 秒轮询里被无声吃掉。
 *
 * 这里钉死三条约定：
 * 1. 「已安装」看文件层（`installed` / `install_state`），不看运行态；
 * 2. `version: null` 不得覆盖已知的版本号；
 * 3. 文件丢失（疑似被杀毒软件隔离）不得被当成「已安装」。
 */

const state = vi.hoisted(() => ({
  snapshot: null as Record<string, unknown> | null,
  status: null as Record<string, unknown> | null,
  version: '',
  installation: null as Record<string, unknown> | null,
}))

vi.mock('@/services/kernel-service', () => ({
  kernelService: {
    onKernelStatusChange: vi.fn(async () => () => {}),
    onKernelError: vi.fn(async () => () => {}),
    getKernelSnapshot: vi.fn(async () => state.snapshot),
    getKernelStatus: vi.fn(async () => state.status),
    getKernelVersion: vi.fn(async () => state.version),
    getKernelInstallation: vi.fn(async () => state.installation),
    getKernelReleases: vi.fn(async () => []),
    getLatestKernelVersion: vi.fn(async () => ''),
    restartKernel: vi.fn(),
    stopKernel: vi.fn(),
  },
}))

vi.mock('@/services/event-service', () => ({
  eventService: {
    on: vi.fn(async () => () => {}),
  },
}))

vi.mock('@/stores/app/AppStore', () => ({
  useAppStore: () => ({
    setRunningState: vi.fn(),
    isRunning: false,
  }),
}))

const BASE_STATUS = {
  process_running: false,
  api_ready: false,
  websocket_ready: false,
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.useFakeTimers()
  state.snapshot = { ...BASE_STATUS }
  state.status = { ...BASE_STATUS }
  state.version = ''
  state.installation = null
})

afterEach(() => {
  vi.useRealTimers()
})

async function makeStore() {
  const store = useKernelStore()
  await store.initializeStore()
  return store
}

// 整个 store 依赖图（含 4 份语言包）首次加载在冷启动时可能超过默认 5s 超时，
// 这里放宽整组用例的预算，避免并行跑全量套件时偶发红灯。
describe('内核安装态语义', () => {
  it('内核未运行时不得显示为「未安装」——已安装看文件层，不看运行态', async () => {
    // 内核没在跑：运行态 version 为 null，但文件在、版本已知
    state.snapshot = {
      ...BASE_STATUS,
      version: null,
      installed: true,
      installed_version: '1.14.4',
      install_state: 'installed',
    }

    const store = await makeStore()

    expect(store.hasVersionInfo(), '内核已安装就必须显示为已安装').toBe(true)
    expect(store.getVersionString()).toBe('1.14.4')
    expect(store.isKernelInstalled).toBe(true)
  })

  it('状态轮询回传的 version: null 不得抹掉已知的版本号', async () => {
    state.version = '1.14.4'
    state.snapshot = { ...BASE_STATUS, installed: true, installed_version: '1.14.4' }
    state.status = { ...BASE_STATUS, version: null, installed: true, installed_version: '1.14.4' }

    const store = await makeStore()
    await store.checkKernelInstallation()
    expect(store.getVersionString()).toBe('1.14.4')

    // 模拟 3 秒轮询：后端回传 version: null
    await store.refreshStatus()

    expect(store.getVersionString(), 'null 不得覆盖已知版本号').toBe('1.14.4')
    expect(store.hasVersionInfo()).toBe(true)
  })

  it('内核文件缺失（疑似被杀毒软件隔离）不得被当成「已安装」', async () => {
    state.snapshot = {
      ...BASE_STATUS,
      version: null,
      installed: false,
      installed_version: null,
      install_state: 'file_lost',
    }
    state.installation = {
      state: 'file_lost',
      version: null,
      detail: '内核文件缺失（可能被杀毒软件隔离或被清理）',
      kernel_path: 'C:/Users/x/AppData/Local/sing-box-windows/sing-box/sing-box.exe',
    }

    const store = await makeStore()

    expect(store.installState).toBe('file_lost')
    expect(store.isKernelInstalled).toBe(false)
    expect(store.hasVersionInfo()).toBe(false)
  })

  it('文件存在但读不出版本时仍算「已安装」，并保留技术细节供排障', async () => {
    state.snapshot = {
      ...BASE_STATUS,
      installed: true,
      installed_version: null,
      install_state: 'unreadable',
    }
    state.installation = {
      state: 'unreadable',
      version: null,
      detail: '获取版本信息失败: 被安全软件拦截',
      kernel_path: 'C:/Users/x/AppData/Local/sing-box-windows/sing-box/sing-box.exe',
    }

    const store = await makeStore()

    expect(store.installState).toBe('unreadable')
    expect(store.isKernelInstalled, '文件在磁盘上就是已安装').toBe(true)
    expect(store.installation?.detail).toContain('安全软件')
  })

  it('确实从未安装时才算「未安装」', async () => {
    state.snapshot = {
      ...BASE_STATUS,
      version: null,
      installed: false,
      installed_version: null,
      install_state: 'missing',
    }

    const store = await makeStore()

    expect(store.installState).toBe('missing')
    expect(store.isKernelInstalled).toBe(false)
    expect(store.hasVersionInfo()).toBe(false)
  })

  it('内核文件被删除后不得继续拿旧版本号显示「已安装」', async () => {
    // 先是装好的
    state.snapshot = {
      ...BASE_STATUS,
      installed: true,
      installed_version: '1.14.4',
      install_state: 'installed',
    }
    const store = await makeStore()
    expect(store.getVersionString()).toBe('1.14.4')

    // 随后文件没了（杀软隔离 / 被清理）
    state.status = {
      ...BASE_STATUS,
      version: null,
      installed: false,
      installed_version: null,
      install_state: 'file_lost',
    }
    await store.refreshStatus()

    expect(store.isKernelInstalled).toBe(false)
    expect(store.hasVersionInfo(), '文件没了就不能还说「已安装」').toBe(false)
  })
}, 30_000)
