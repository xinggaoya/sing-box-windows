/** 市场模板（列表/详情共用；列表项不含 content） */
export interface MarketTemplate {
  id: string
  name: string
  description: string
  author_name: string
  kernel_type: string
  schema_version?: string | null
  /** 详情才有：pending / approved / rejected */
  status?: string
  reject_reason?: string
  downloads: number
  /** 列表项无 content，详情/下载接口才有 */
  content?: string
  created_at?: string | null
  updated_at?: string | null
}
