import { computed, ref } from 'vue'
import { GlobeOutline, RadioOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useAppStore } from '@/stores'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { proxyService } from '@/services/proxy-service'
import { kernelService } from '@/services/kernel-service'
import { systemService } from '@/services/system-service'

/**
 * 首页的「运行环境」探测
 *
 * TUN 是否可用取决于两件事：运行在哪个平台、是否已拿到管理员权限。
 * 这两项都来自后端探测且在页面生命周期内只变一次，抽出后 HomeView 的
 * script 不必同时关心「怎么探测」和「探测结果怎么用」。
 */

export type Platform = 'windows' | 'linux' | 'macos' | 'unknown'

/** 受支持的平台；`unknown` 表示探测失败 */
const KNOWN_PLATFORMS: readonly Platform[] = ['windows', 'linux', 'macos']

export function useHomeEnvironment() {
  const platform = ref<Platform>('unknown')
  const isAdmin = ref(false)

  const isWindowsPlatform = computed(() => platform.value === 'windows')
  const isUnixPlatform = computed(() => platform.value === 'linux' || platform.value === 'macos')

  /** 探测操作系统；后端返回非预期值时归为 `unknown`（而不是误判成某个平台） */
  const detectPlatform = async () => {
    try {
      const raw = await systemService.getPlatformInfo()
      platform.value = KNOWN_PLATFORMS.includes(raw as Platform) ? (raw as Platform) : 'unknown'
    } catch {
      platform.value = 'unknown'
    }
  }

  const checkAdmin = async () => {
    try {
      isAdmin.value = await systemService.checkAdmin()
    } catch {
      // 探测失败按「无权限」处理：宁可多走一次提权流程，也不要让用户卡在无权限状态
      isAdmin.value = false
    }
  }

  return { platform, isAdmin, isWindowsPlatform, isUnixPlatform, detectPlatform, checkAdmin }
}

/**
 * 首页的「节点模式」（Clash mode）切换与一键测速
 *
 * 与代理开关（TUN / 系统代理）是两件事：前者决定 sing-box 内部的路由策略，
 * 后者决定流量怎么进入 sing-box。两者只是都出现在首页，因此一度写在了一起。
 */
export function useNodeProxyMode() {
  const { t } = useI18n()
  const message = useMessage()
  const appStore = useAppStore()
  const proxyStore = useProxyStore()

  /** 当前节点模式（global / rule / direct） */
  const currentNodeProxyMode = ref('rule')
  /** 一键测速进行中 */
  const quickTesting = ref(false)

  /** 模式选项；`nameKey` 走 locale 语言包而非硬编码 */
  const nodeProxyModes = [
    { value: 'global', nameKey: 'home.nodeMode.global', icon: GlobeOutline },
    { value: 'rule', nameKey: 'home.nodeMode.rule', icon: RadioOutline },
  ] as const

  /** 从内核拉取当前模式；只接受内核实际会返回的值 */
  const syncCurrentNodeProxyMode = async () => {
    try {
      const status = await proxyService.getClashModeStatus()
      const mode = status.currentMode
      if (mode === 'global' || mode === 'rule' || mode === 'direct') {
        currentNodeProxyMode.value = mode
      }
    } catch {
      // 内核未启动 / gRPC 不可用：保留上次的显示值，不闪回默认值
    }
  }

  /**
   * 切换节点模式（Clash mode）。
   *
   * 注意：切换期间不禁用 UI —— 原实现也没有，节点模式只影响内核内部路由策略，
   * 不涉及入站变更，没有并发冲突风险。系统代理 / TUN 开关的互斥由 HomeView 的
   * `proxyModePending` 负责。
   */
  const handleNodeProxyModeChange = async (mode: string) => {
    if (currentNodeProxyMode.value === mode) return

    try {
      const result = await kernelService.switchNodeProxyMode(mode as 'global' | 'rule')
      await syncCurrentNodeProxyMode()

      if (result.includes('重启后生效')) {
        message.warning(result)
        return
      }
      message.success(t('home.nodeModeChangeSuccess'))
    } catch {
      message.error(t('home.nodeModeChangeFailed'))
    }
  }

  /** 一键测速：先刷新节点列表，再对所有分组测速 */
  const quickTestAll = async () => {
    if (quickTesting.value) return
    try {
      quickTesting.value = true
      await proxyStore.fetchProxies()
      await proxyStore.testAllGroups()
      message.success(t('proxy.batchTestComplete'))
    } catch {
      message.error(t('proxy.testErrorMessage'))
    } finally {
      quickTesting.value = false
    }
  }

  /** 本地代理地址（展示用） */
  const proxyAddress = computed(() => `127.0.0.1:${appStore.proxyPort}`)

  return {
    currentNodeProxyMode,
    quickTesting,
    nodeProxyModes,
    proxyAddress,
    syncCurrentNodeProxyMode,
    handleNodeProxyModeChange,
    quickTestAll,
  }
}
