import { invokeWithAppContext } from './invoke-client'
import type { CustomRule } from '@/types/generated/CustomRule'
import type { CustomRuleAction } from '@/types/generated/CustomRuleAction'
import type { CustomRuleMatchType } from '@/types/generated/CustomRuleMatchType'

export type { CustomRule, CustomRuleAction, CustomRuleMatchType }

/**
 * 自定义规则 CRUD（后端 `proxy_service::custom_rule_crud`）
 *
 * ⚠️ 与运行时规则的关键差异（后端 `custom_rule_crud.rs` 顶部注释已写明）：
 *
 * 1. **走「写配置文件 + 重启内核」路径，不走 gRPC**。
 *    运行时规则的启停是 gRPC 实时操作、不持久化；自定义规则则持久化到本地存储
 *    并写入本程序生成的配置，**保存后需要重启内核才生效**。
 * 2. **只对本程序生成的配置生效**。
 *    对「使用订阅原始配置」的配置，自定义规则不会被注入。
 *
 * UI 上必须把这两点显性表达出来，否则用户会以为保存即生效。
 */

/** 新增 / 更新规则的表单载荷 */
export interface CustomRuleInput {
  match_type: CustomRuleMatchType
  payload: string
  action: CustomRuleAction
  note?: string | null
}

export const customRuleService = {
  /** 列出全部自定义规则（按创建时间升序） */
  list(): Promise<CustomRule[]> {
    return invokeWithAppContext<CustomRule[]>('list_custom_rules')
  },

  /** 新增一条规则，返回落库后的完整规则（含生成的 id） */
  add(input: CustomRuleInput): Promise<CustomRule> {
    return invokeWithAppContext<CustomRule>('add_custom_rule', {
      match_type: input.match_type,
      matchType: input.match_type,
      payload: input.payload,
      action: input.action,
      note: input.note ?? null,
    })
  },

  /** 更新已有规则（按 id 定位） */
  update(id: string, input: CustomRuleInput): Promise<void> {
    return invokeWithAppContext<void>('update_custom_rule', {
      id,
      match_type: input.match_type,
      matchType: input.match_type,
      payload: input.payload,
      action: input.action,
      note: input.note ?? null,
    })
  },

  /** 删除规则（按 id 定位） */
  remove(id: string): Promise<void> {
    return invokeWithAppContext<void>('delete_custom_rule', { id })
  },

  /** 启用 / 禁用规则（按 id 定位） */
  toggle(id: string): Promise<void> {
    return invokeWithAppContext<void>('toggle_custom_rule', { id })
  },
}
