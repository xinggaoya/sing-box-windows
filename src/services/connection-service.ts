import { invokeWithAppContext } from './invoke-client'

/**
 * sing-box gRPC API 连接管理
 *
 * 端口和 secret 由后端从 AppConfig 自动注入，前端无需传 port 参数。
 */
export const connectionService = {
  /** 关闭所有连接（gRPC CloseAllConnections） */
  closeAll() {
    return invokeWithAppContext<void>('close_all_connections')
  },

  /** 关闭单个连接（gRPC CloseConnection） */
  closeOne(id: string) {
    return invokeWithAppContext<void>('close_connection', { id })
  },
}