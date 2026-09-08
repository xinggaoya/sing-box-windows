import type { MarketTemplate } from './MarketTemplate'

/** 市场模板列表（分页） */
export interface MarketTemplateList {
  items: MarketTemplate[]
  total: number
  page: number
  page_size: number
}
