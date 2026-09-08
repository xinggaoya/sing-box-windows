import { invokeWithAppContext } from './invoke-client'
import type {
  ConfigTemplate,
  MarketHealth,
  MarketTemplate,
  MarketTemplateList,
  TemplateMarketSettings,
} from '@/types/generated'

/**
 * 在线模板市场服务封装。
 *
 * 后端命令返回/接收的模型字段为 snake_case（与 src/types/generated 中的镜像类型一致）；
 * Tauri 会自动把 camelCase 的调用参数名映射到 snake_case 命令参数。
 */

export interface MarketListOptions {
  search?: string
  sort?: 'newest' | 'downloads'
  page?: number
  pageSize?: number
}

export const templateMarketService = {
  // --- 本地模板 ---

  getConfigTemplates() {
    return invokeWithAppContext<ConfigTemplate[]>('get_config_templates', undefined, {
      skipDataRestore: true,
    })
  },

  saveConfigTemplate(template: ConfigTemplate) {
    return invokeWithAppContext<ConfigTemplate>('save_config_template', { template })
  },

  deleteConfigTemplate(templateId: string) {
    return invokeWithAppContext<void>('delete_config_template', { templateId })
  },

  exportOfficialTemplate() {
    return invokeWithAppContext<string>('export_official_template')
  },

  // --- 市场设置 ---

  getTemplateMarketSettings() {
    return invokeWithAppContext<TemplateMarketSettings>('get_template_market_settings')
  },

  setTemplateMarketSettings(settings: TemplateMarketSettings) {
    return invokeWithAppContext<TemplateMarketSettings>('set_template_market_settings', {
      settings,
    })
  },

  setActiveTemplate(templateId: string) {
    return invokeWithAppContext<TemplateMarketSettings>('set_active_template', { templateId })
  },

  // --- 模板市场（远程） ---

  marketCheckService(serviceUrl?: string) {
    return invokeWithAppContext<MarketHealth>('market_check_service', {
      serviceUrl: serviceUrl ?? null,
    })
  },

  marketListTemplates(options: MarketListOptions = {}) {
    return invokeWithAppContext<MarketTemplateList>('market_list_templates', {
      search: options.search ?? null,
      sort: options.sort ?? null,
      page: options.page ?? null,
      pageSize: options.pageSize ?? null,
    })
  },

  marketGetTemplateDetail(remoteId: string) {
    return invokeWithAppContext<MarketTemplate>('market_get_template_detail', { remoteId })
  },

  marketPublishTemplate(templateId: string, authorName?: string) {
    return invokeWithAppContext<ConfigTemplate>('market_publish_template', {
      templateId,
      authorName: authorName ?? null,
    })
  },

  marketUpdateTemplate(templateId: string) {
    return invokeWithAppContext<ConfigTemplate>('market_update_template', { templateId })
  },

  marketDeleteTemplate(templateId: string) {
    return invokeWithAppContext<ConfigTemplate>('market_delete_template', { templateId })
  },

  marketDownloadTemplate(remoteId: string) {
    return invokeWithAppContext<ConfigTemplate>('market_download_template', { remoteId })
  },

  marketRefreshMyTemplates() {
    return invokeWithAppContext<ConfigTemplate[]>('market_refresh_my_templates')
  },
}
