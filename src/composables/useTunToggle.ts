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

/**
 * 文案既可直接给字面量，也可给 getter。
 *
 * 必填 getter：i18n 文案若在调用方 setup 时求值一次就存进普通对象，组件后续切换语言时
 * 不会被重新求值，错误提示会一直停留在切换前的语言（`useTunToggle` 内部是普通对象，
 * 不是 computed，追踪不到响应式依赖）。
 */
export type TunToggleMessageSource = {
  [K in keyof TunToggleMessages]: TunToggleMessages[K] | (() => TunToggleMessages[K])
}

export interface TunToggleOptions {
  messages: TunToggleMessageSource
  /** 提权前置；省略则直接执行核心流程 */
  ensurePrivilege?: (target: boolean) => Promise<PrivilegeOutcome>
  /** sudo 密码相关失败的处理；返回 true 表示原地重试成功 */
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

  /**
   * 读取文案。传入 getter 时**每次调用都重新求值**，
   * 因此调用方可以用 `() => t('...')` 避开 i18n 的时序问题。
   */
  const text = (key: keyof TunToggleMessages): string => {
    const value = options.messages[key]
    return typeof value === 'function' ? value() : value
  }

  /** 组装一次失败的归因信息（诊断摘要 > lastError > 注入的兜底文案） */
  const describe = (raw: unknown, fallback: string): KernelFailure =>
    describeKernelFailure(raw, {
      diagnosisSummary: kernelStore.startupDiagnosisSummary,
      lastError: kernelStore.lastError,
      fallback,
    })

  /**
   * 按目标状态重新应用配置并重启内核。
   *
   * 用于两处「需要把运行态拉回某个模式」的场景：
   * - 切换失败后的回滚（TUN-09）；
   * - 提权重启失败后恢复网络出口（TUN-10）。
   *
   * 尽力而为：失败时不抛错，由调用方决定如何汇报。
   */
  const restoreRuntime = async (target: boolean): Promise<void> => {
    try {
      const applied = await kernelStore.applyProxySettings(options.applyOverrides?.(target))
      if (applied) {
        await kernelStore.restartKernel()
      }
    } catch {
      // 恢复失败不能掩盖原始错误：调用方会把原始 failure 一并返回
    }
  }

  /**
   * 核心流程（不含回滚）。返回 `null` 表示成功，否则返回失败归因。
   *
   * sudo 重试最多一次（`allowSudoRetry` + 循环上限双重保证）：密码一直不对时
   * 无限弹框比直接失败更糟。
   */
  const runOnce = async (
    target: boolean,
    allowSudoRetry: boolean,
  ): Promise<KernelFailure | null> => {
    // 1) 置位开关（`toggleTun` 是幂等的，重试时重复调用安全）
    await appStore.toggleTun(target)

    // 2)~4) 应用配置 -> 重启内核 -> 必要时索要密码后重来一轮
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const mayRequestSudo = attempt === 0 && allowSudoRetry

      const applied = await kernelStore.applyProxySettings(options.applyOverrides?.(target))
      if (!applied) {
        return describe(null, text('applyFailed'))
      }

      const restarted = await kernelStore.restartKernel()
      if (restarted) return null

      const failure = describe(null, text('restartFailed'))
      if (!mayRequestSudo || !failure.needsSudoRetry || !options.handleSudoFailure) {
        return failure
      }

      // 用户重新提供了密码 —— 必须重新执行「应用配置 + 重启内核」，
      // 仅索要密码并不生效（密码只是写入 sudo 缓存，内核仍需重启才会用上）。
      const retried = await options.handleSudoFailure(failure.sudoCode!)
      if (!retried) return failure
    }

    // 不可达：循环最多两轮，第二轮必定在上面的分支里返回
    return describe(null, text('restartFailed'))
  }

  /**
   * 把 TUN 开关切到 `target`，失败时回滚到原状态。
   *
   * @param target 目标开关状态
   */
  const applyTunSwitch = async (target: boolean): Promise<TunSwitchOutcome> => {
    const previous = appStore.tunEnabled

    // TUN-04：状态未变时**先短路**，再考虑提权。
    // 提权前置在 Windows 上会弹 UAC 并以管理员身份重启整个应用 ——
    // 什么都没改的切换不该触发它，否则用户会看到「没动却弹 UAC 重启」。
    if (previous === target) return { status: 'skipped' }

    // 提权前置由调用方处理（可能直接转入「保存 + 管理员重启」，无需本地回滚）
    if (options.ensurePrivilege) {
      const outcome = await options.ensurePrivilege(target)
      if (outcome === 'escalated') return { status: 'escalated' }
    }

    pending.value = true
    try {
      const failure = await runOnce(target, true)
      if (!failure) return { status: 'success' }

      // 回滚开关：TUN 是全量接管，回滚失败会让用户失去网络出口，
      // 因此这里的错误不能吞掉。
      await appStore.toggleTun(previous)
      // TUN-09：只还原 DB 布尔值会让「界面状态」与「内核实际模式」脱节 ——
      // 内核可能已被停掉、或仍跑着新模式（残留 TUN 网卡与路由）。
      await restoreRuntime(previous)
      return { status: 'failed', failure }
    } catch (error) {
      await appStore.toggleTun(previous)
      await restoreRuntime(previous)
      return { status: 'failed', failure: describe(error, text('restartFailed')) }
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
 * @param target 目标开关状态。开启传 `true`，关闭传 `false`（关闭同样需要结束
 *               一个高权限内核进程，普通权限杀不掉 —— TUN-08）。
 * @returns 是否成功走到「请求重启」这一步
 */
export async function prepareTunAndRestartAsAdmin(target = true): Promise<boolean> {
  const appStore = useAppStore()
  const kernelStore = useKernelStore()

  const previous = appStore.tunEnabled
  // 目标与当前一致时无需变更，更不该触发「停内核 + 重启应用」
  if (previous === target) return true

  try {
    await appStore.toggleTun(target)
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
    // TUN-10：内核已经被停掉，而 TUN 是全量接管 —— 若不把内核拉回来，
    // 用户会直接失去网络出口。这里按旧模式恢复运行态。
    try {
      const applied = await kernelStore.applyProxySettings({ tun_enabled: previous })
      if (applied) {
        await kernelStore.restartKernel()
      }
    } catch {
      // 尽力而为：恢复失败也不能掩盖原始错误
    }
    return false
  }
}

/**
 * Windows 提权前置：首页与托盘共用。
 *
 * 当前进程没有管理员权限时，转入「落库 + 以管理员身份重启应用」。
 *
 * 开启与关闭**对称**（TUN-08）：关闭 TUN 同样要结束一个高权限内核进程，
 * 普通权限杀不掉，因此也必须走管理员重启。
 *
 * 返回值语义：
 * - `'proceed'`：**已经具备管理员权限**（或本次根本不需要提权），
 *   调用方应继续走本地状态机；
 * - `'escalated'`：本次切换已由提权流程接手（成功重启 / 用户取消 / 提权失败）。
 *   提权失败时**不会**退回本地状态机 —— 否则会出现「二次报错 + 二次弹窗」。
 */
export async function ensureWindowsTunPrivilege(
  target: boolean,
  options: {
    /** 确认回调（首页有确认弹窗，托盘没有）。返回 `false` 表示用户取消。 */
    confirm?: (target: boolean) => Promise<boolean>
    /** 提权失败时的回调（用于提示用户） */
    onFailure?: () => void
  } = {},
): Promise<PrivilegeOutcome> {
  const { systemService } = await import('@/services/system-service')
  const isAdmin = await systemService.checkAdmin()
  if (isAdmin) return 'proceed'

  if (options.confirm) {
    const confirmed = await options.confirm(target)
    if (!confirmed) return 'escalated'
  }

  const escalated = await prepareTunAndRestartAsAdmin(target)
  if (!escalated) {
    options.onFailure?.()
  }
  return 'escalated'
}
