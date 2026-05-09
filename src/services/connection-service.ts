import { invokeWithAppContext } from './invoke-client'

export const connectionService = {
  closeAll(port?: number) {
    const args = typeof port === 'number' ? { port } : undefined
    return invokeWithAppContext<void>('close_all_connections', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port',
    })
  },

  closeOne(id: string, port?: number) {
    const args = typeof port === 'number' ? { id, port } : { id }
    return invokeWithAppContext<void>('close_connection', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port',
    })
  },
}
