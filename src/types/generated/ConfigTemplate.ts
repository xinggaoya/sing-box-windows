/**
 * 配置模板（本地 + 官方内置）
 *
 * 每条记录是一份 sing-box 配置骨架，通过 `{{NODES}}` 占位符或节点注入生成最终配置。
 * 官方内置模板 id 固定为 `official`（source = builtin），不落库、由后端实时生成。
 */
export interface ConfigTemplate {
  /** 本地唯一 ID；官方模板固定为 `official` */
  id: string
  name: string
  /** 内核类型，目前仅 `singbox` */
  kernel_type: string
  /** 模板内容（完整 sing-box JSON，可含 `{{NODES}}` 占位符） */
  content: string
  /** 来源：builtin / local / market */
  source: string
  description?: string | null
  author?: string | null
  /** 市场模板 ID（发布或从市场下载后有值） */
  remote_id?: string | null
  /** 我发布的市场模板的编辑令牌（仅保存在本地） */
  edit_token?: string | null
  /** 市场审核状态：pending / approved / rejected / deleted（本地标记：市场记录已不存在） */
  market_status?: string | null
  /** 审核拒绝理由 */
  market_reject_reason?: string | null
  /** 关联的内核 schema 版本 */
  schema_version?: string | null
  /** 编辑版本号（每次保存自增） */
  revision: number
  /** 创建时间（毫秒） */
  created_at: number
  /** 更新时间（毫秒） */
  updated_at: number
}
