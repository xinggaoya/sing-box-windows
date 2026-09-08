/** 发布结果：edit_token 明文仅此一次返回，客户端负责持久保存 */
export interface MarketPublishResult {
  id: string
  edit_token: string
  status: string
}
