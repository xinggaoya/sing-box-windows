import { useI18n } from 'vue-i18n'
import type { StartupDiagnosis, StartupDiagnosisKind } from '@/types/events'

/**
 * 启动失败归因的可展示视图
 *
 * 后端 `StartupDiagnosis.kind` 是机器可读的分类（config_invalid / binary_missing /
 * permission_denied / port_conflict ...），但直接甩给用户看等于没归因——
 * 用户需要知道「到底哪里坏了、现在该做什么」。
 */
export interface KernelDiagnosisView {
  /** 归因标题（本地化） */
  title: string
  /** 处理建议（本地化建议在前，后端原始建议在后） */
  hints: string[]
  /** 是否属于「内核文件层面」问题（文件缺失 / 被隔离） */
  fileIssue: boolean
  /** 是否属于「配置层面」问题（配置非法 / 配置缺失） */
  configIssue: boolean
  /** 是否需要提权才能启动 */
  needsElevation: boolean
}

/** 属于「内核文件层面」的归因：多半是杀毒软件隔离或文件被清理 */
const FILE_ISSUE_KINDS: StartupDiagnosisKind[] = ['binary_missing']

/** 属于「配置层面」的归因：与内核二进制无关，是配置本身不合法 */
const CONFIG_ISSUE_KINDS: StartupDiagnosisKind[] = ['config_invalid', 'config_missing']

/** 需要提权才能解决的归因 */
const ELEVATION_KINDS: StartupDiagnosisKind[] = [
  'permission_denied',
  'sudo_required',
  'sudo_invalid',
]

/**
 * 把后端的启动诊断翻译成「人能看懂的原因 + 该做什么」。
 *
 * 归因文案走 i18n（4 语言对齐），后端的 `detail` / `suggested_actions` 作为
 * 技术细节原样保留——它们包含 stderr 片段、状态码这类无法本地化的内容。
 */
export function useKernelDiagnosis() {
  const { t, te } = useI18n()

  const describe = (diagnosis?: StartupDiagnosis | null): KernelDiagnosisView | null => {
    if (!diagnosis) return null

    const kind = diagnosis.kind
    const titleKey = `setting.kernel.diagnosis.${kind}.title`
    const hintKey = `setting.kernel.diagnosis.${kind}.hint`

    const title = te(titleKey) ? t(titleKey) : diagnosis.message
    const hints: string[] = []
    if (te(hintKey)) {
      hints.push(t(hintKey))
    }
    for (const action of diagnosis.suggested_actions ?? []) {
      if (action && !hints.includes(action)) {
        hints.push(action)
      }
    }

    return {
      title,
      hints,
      fileIssue: FILE_ISSUE_KINDS.includes(kind),
      configIssue: CONFIG_ISSUE_KINDS.includes(kind),
      needsElevation: ELEVATION_KINDS.includes(kind),
    }
  }

  return { describe }
}
