import { ref } from 'vue'
import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { describeKernelFailure, type KernelFailure } from '@/utils/kernel-errors'

/**
 * TUN 模式切换的统一状态机
 *
 * 背景：「置位 TUN 开关 → 应用代理配置 → 重启内核 → 失败则回滚开关」这段逻辑此前在
 * `HomeView.vue`（3 处）与 `TrayStore.ts`（3 处）各写了一遍，共 6 份变体。它们的差异只在
 * 两点：**平台提权处理**与**sudo 密码重试**，而回滚顺序、重试次数、状态归并这些真正容易
 * 出错的细节却各不相同 —— 历史上出现过的 bug 类别基本都是「某条路径忘了回滚」。
 *
 * 这里把平台无关的核心收敛为唯一实现，平台差异通过回调注入：
 * - `ensurePrivilege`：提权前置（Windows 管理员 / Unix sudo），返回 `'escalated'` 表示
 *   已转入「保存配置并以管理员身份重启」流程，调用方应结束本次切换；
 * - `handleSudoFailure`：sudo 密码缺失/失效时的处理，返回 `true` 表示原地重试成功。
 *
 * 消息文案由调用方注入，保证 i18n 上下文（`useI18n()` vs `i18n.global.t`）正确。
 */

export type PrivilegeOutcome = 'proceed' | 'escalated'

export interface TunToggleMessages {
  /** `applyProxySettings` 失败时的兜底文案 */
  applyFailed: string
  /** `restartKernel` 失败时的兜底文案 */
  restartFailed: string
}

export interface TunToggleOptions {
  messages: TunToggleMessages
  /** 提权前置；省略则直接执行核心流程 */
  ensurePrivilege?: (target: boolean) => Promise<PrivilegeOutcome>
  /** sudo 密码相关失败的处理；返回 true 表示重试成功 */
  handleSudoFailure?: (code: 'required' | 'invalid') => Promise<boolean>
  /** 传给 `applyProxySettings` 的额外覆盖（如 `{ tun_enabled: true }`） */
  applyOverrides?: (target: boolean) => Record<string, unknown> | undefined
}

export type TunSwitchOutcome =
  | { status: 'success' }
  /** 目标状态与当前一致，未做改动 */
  | { status: 'skipped' }
  /** 已转入提权重启流程，本次切换由重启接手 */
  | { status: 'escalated' }
  | { status: 'failed'; failure: KernelFailure }

export function useTunToggle(options: TunToggleOptions) {
  const appStore = useAppStore()
  const kernelStore = useKernelStore()

  /** 切换进行中；用于禁用 UI 开关，防止并发触发两条状态机 */
  const pending = ref(false)

  /** 组装一次失败的归因信息（诊断摘要 > lastError > 注入的兜底文案） */
  const describe = (raw: unknown, fallback: string): KernelFailure =>
    describeKernelFailure(raw, {
      diagnosisSummary: kernelStore.startupDiagnosisSummary,
      lastError: kernelStore.lastError,
      fallback,
    })

  /**
   * 核心流程（不含回滚）。返回 `null` 表示成功，否则返回失败归因。
   *
   * `allowSudoRetry` 只在第一次调用时为 true：密码错误最多索要一次，
   * 避免「密码一直不对 → 无限弹框」的循环。
   */
  const runOnce = async (
    target: boolean,
    allowSudoRetry: boolean
  ): Promise<KernelFailure | null> => {
    // 1) 置位开关（`toggleTun` 是幂等的，重试时重复调用安全）
    await appStore.toggleTun(target)

    // 2) 应用代理配置
    const overrides = options.applyOverrides?.(target)
    const applied = await kernelStore.applyProxySettings(overrides)
    if (!applied) {
      return describe(null, options.messages.applyFailed)
    }

    // 3) 重启内核
    const restarted = await kernelStore.restartKernel()
    if (restarted) return null

    const failure = describe(null, options.messages.restartFailed)
    if (failure.needsSudoRetry && allowSudoRetry && options.handleSudoFailure) {
      const retried = await options.handleSudoFailure(failure.sudoCode!)
      if (retried) return null
    }
    return failure
  }

  /**
   * 把 TUN 开关切到 `target`，失败时回滚到原状态。
   *
   * @param target 目标开关状态
   */
  const applyTunSwitch = async (target: boolean): Promise<TunSwitchOutcome> => {
    // 提权前置由调用方处理（可能直接转入「保存 + 管理员重启」，无需本地回滚）
    if (options.ensurePrivilege) {
      const outcome = await options.ensurePrivilege(target)
      if (outcome === 'escalated') return { status: 'escalated' }
    }

    const previous = appStore.tunEnabled
    if (previous === target) return { status: 'skipped' }

    pending.value = true
    try {
      const failure = await runOnce(target, true)
      if (!failure) return { status: 'success' }

      // 回滚开关：TUN 是全量接管，回滚失败会让用户失去网络出口，
      // 因此这里的错误不能吞掉。
      await appStore.toggleTun(previous)
      return { status: 'failed', failure }
    } catch (error) {
      await appStore.toggleTun(previous)
      return { status: 'failed', failure: describe(error, options.messages.restartFailed) }
    } finally {
      pending.value = false
    }
  }

  return { pending, applyTunSwitch }
}
/**
 * 「保存 TUN 配置 → 以管理员身份重启」流程。
 *
 * 与 `useTunToggle` 的区别：这里不是就地重启内核，而是让整个应用以管理员权限重启，
 * 用于 Windows 上「当前进程没有提权、但把配置存下来后重启就能拿到 TUN 权限」的场景。
 *
 * 此前 HomeView（`prepareTunModeWithAdminRestart`）与 TrayStore（`ensurePrivilege` 分支）
 * 各写一份，且行为不一致：HomeView 多调了一次 `applyProxySettings`（对一个马上就要被
 * 杀掉的进程做运行时配置应用，既无必要又会在失败时提前中断流程），TrayStore 则没有。
 * 这里统一为「置位 → 落库 → 停内核 → 请求重启」，任何一步失败都回滚开关。
 *
 * @returns 是否成功走到「请求重启」这一步
 */
export async function prepareTunAndRestartAsAdmin(): Promise<boolean> {
  const appStore = useAppStore()
  const kernelStore = useKernelStore()

  const previous = appStore.tunEnabled
  try {
    await appStore.toggleTun(true)
    // 必须落库：新进程启动时从 DB 读取配置，不落库则重启后 TUN 不会生效
    await appStore.saveToBackend()

    if (appStore.isRunning) {
      await kernelStore.stopKernel()
    }

    const { systemService } = await import('@/services/system-service')
    await systemService.restartAsAdmin()
    return true
  } catch {
    // 回滚开关，避免用户下次启动直接进入未验证的 TUN 状态
    await appStore.toggleTun(previous)
    return false
  }
}
