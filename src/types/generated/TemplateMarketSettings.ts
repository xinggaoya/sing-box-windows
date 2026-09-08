/** 模板市场设置（服务地址 + 当前激活模板） */
export interface TemplateMarketSettings {
  /** 市场服务地址（如 https://templates.example.com），空表示未配置 */
  service_url: string
  /** 当前生效模板：`official` 或本地模板 ID；订阅刷新/下载时统一使用 */
  active_template_id: string
}
