/**
 * 内核错误处理工具
 *
 * 背景：后端 `Result<T, String>` 的错误一路经 `invoke` 抛回前端，形态不统一
 * （`Error` / 裸字符串 / `{ message }` / `{ error }` 对象都有），而“是否需要 sudo 密码重试”
 * 这类判断此前在 App.vue、HomeView.vue、TrayStore.ts 里各写了一份**逐字相同**的实现，
 * 只要后端改了错误码前缀，三处就会静默不一致（表现为“某些入口不弹密码框”）。
 *
 * 这里收敛为唯一来源，并顺带消除另外两处重复：
 * - `parseSudoCode`：3 份 -> 1份
 * - `getKernelFailureText`（诊断摘要 > lastError > fallback）：2 份 -> 1 份
 */

/** 后端 sudo 错误码前缀（与 `app/system/sudo_service.rs` 保持一致） */
const SUDO_PASSWORD_REQUIRED = 'SUDO_PASSWORD_REQUIRED'
const SUDO_PASSWORD_INVALID = 'SUDO_PASSWORD_INVALID'

/**
 * sudo 密码错误类型。
 * - `required`：需要用户输入密码
 * - `invalid`：已输入但校验不通过
 */
export type SudoFailureCode = 'required' | 'invalid'

/**
 * 从任意形态的错误中提取可读消息。
 *
 * 兼容：`Error`、裸字符串、`{ message }`、`{ error }`、以及 Tauri `invoke` 拒绝时
 * 抛出的对象包装。
 */
export function extractKernelErrorMessage(raw: unknown): string {
  if (typeof raw === 'string') return raw
  // `Error` 也走这里：它的 `message` 是字符串字段
  if (raw && typeof raw === 'object' && 'message' in raw) {
    const message = (raw as { message?: unknown }).message
    if (typeof message === 'string') return message
  }
  if (raw && typeof raw === 'object' && 'error' in raw) {
    const error = (raw as { error?: unknown }).error
    return typeof error === 'string' ? error : String(error ?? '')
  }
  return String(raw ?? '')
}

/**
 * 判断错误是否来自 sudo 密码缺失/校验失败。
 *
 * 用于决定“是否弹出密码输入框并重试该操作”。
 */
export function parseSudoCode(raw: unknown): SudoFailureCode | null {
  const message = extractKernelErrorMessage(raw)
  if (message.includes(SUDO_PASSWORD_REQUIRED)) return 'required'
  if (message.includes(SUDO_PASSWORD_INVALID)) return 'invalid'
  return null
}

/**
 * 内核失败文案的三级回退：启动诊断摘要 > 最后一次错误 > 调用方给的兜底文案。
 *
 * 诊断摘要优先是因为它比 `lastError` 更具体（含阶段、端口、stderr 片段）。
 */
export function resolveKernelFailureText(
  diagnosisSummary: string | null | undefined,
  lastError: string | null | undefined,
  fallback: string
): string {
  return diagnosisSummary || lastError || fallback
}

/**
 * 一次内核操作失败时的统一归因：
 * 既给出可展示文案，也标出是否需要走 sudo 密码重试流程。
 *
 * 调用方典型用法：
 * ```ts
 * const failure = describeKernelFailure(error, {
 *   diagnosisSummary: kernelStore.startupDiagnosisSummary,
 *   lastError: kernelStore.lastError,
 *   fallback: t('notification.kernelRestartFailed'),
 * })
 * if (failure.sudoCode === 'required') { /* 弹密码框 *\/ }
 * message.error(failure.text)
 * ```
 */
export interface KernelFailure {
  /** 可直接展示给用户的文案（已非空） */
  text: string
  /** sudo 密码错误类型；`null` 表示与 sudo 无关 */
  sudoCode: SudoFailureCode | null
  /** 是否应当引导用户输入 sudo 密码并重试 */
  needsSudoRetry: boolean
}

export function describeKernelFailure(
  raw: unknown,
  options: {
    diagnosisSummary?: string | null
    lastError?: string | null
    fallback: string
  }
): KernelFailure {
  const rawText = extractKernelErrorMessage(raw)
  // 优先用“本次操作真实抛出的消息”——它比历史 lastError 更贴近当前失败原因；
  // 只有当本次没有可读消息时才回退到诊断摘要 / lastError。
  const text =
    rawText || resolveKernelFailureText(options.diagnosisSummary, options.lastError, options.fallback)
  const sudoCode = parseSudoCode(text)

  return {
    text,
    sudoCode,
    needsSudoRetry: sudoCode === 'required' || sudoCode === 'invalid',
  }
}