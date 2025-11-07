import { invokeWithAppContext } from './invoke-client'

export const subscriptionApi = {
  downloadSubscription(url: string, useOriginalConfig: boolean) {
    return invokeWithAppContext<void>(
      'download_subscription',
      { url, useOriginalConfig },
      { withProxyPort: true, withApiPort: true }
    )
  },

  updateSubscription(url: string) {
    return invokeWithAppContext<void>(
      'update_subscription',
      { url },
      { withProxyPort: true, withApiPort: true }
    )
  },

  addManualSubscription(content: string, useOriginalConfig: boolean) {
    return invokeWithAppContext<void>(
      'add_manual_subscription',
      { content, useOriginalConfig },
      { withProxyPort: true, withApiPort: true }
    )
  },

  getCurrentConfig() {
    return invokeWithAppContext<unknown>(
      'get_current_config',
      undefined,
      { withApiPort: true }
    )
  }
}
