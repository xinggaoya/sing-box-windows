import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { tauriApi } from './tauri'
import { NotificationService } from './notification-service'
import i18n from '@/locales'

export class ProxyService {
  private static instance: ProxyService
  private appStore = useAppStore()
  private kernelStore = useKernelStore()
  private proxyStore = useProxyStore()
  private notificationService = NotificationService.getInstance()
  private t = i18n.global.t

  private constructor() {}

  public static getInstance(): ProxyService {
    if (!ProxyService.instance) {
      ProxyService.instance = new ProxyService()
    }
    return ProxyService.instance
  }

  /**
   * 切换代理模式
   * @param mode 代理模式
   * @param messageCallback 消息回调
   * @returns 是否需要关闭应用（重启管理员）
   */
  public async switchMode(
    mode: 'system' | 'tun' | 'manual',
    messageCallback?: (type: 'success' | 'info' | 'error', content: string) => void,
  ): Promise<boolean> {
    try {
      // 根据模式设置代理
      if (mode === 'system') {
        await tauriApi.proxy.setSystemProxy()
        this.appStore.proxyMode = 'system'
        if (messageCallback) {
          messageCallback('success', '系统代理模式已启用')
        }
      } else if (mode === 'manual') {
        await tauriApi.proxy.setManualProxy()
        this.appStore.proxyMode = 'manual'
        if (messageCallback) {
          messageCallback('info', '手动代理模式已启用，请手动设置系统代理')
        }
      } else {
        // TUN模式
        await tauriApi.proxy.setTunProxy()
        this.appStore.proxyMode = 'tun'
        if (messageCallback) {
          messageCallback('success', 'TUN模式已启用')
        }
      }

      // 如果内核正在运行，需要重启
      if (this.appStore.isRunning) {
        try {
          if (messageCallback) messageCallback('info', this.t('home.status.restarting'))
          else this.notificationService.info(this.t('home.status.restarting'))

          await this.kernelStore.restartKernel()

          if (messageCallback) messageCallback('success', this.t('notification.kernelRestarted'))
          else this.notificationService.success(this.t('notification.kernelRestarted'))
        } catch (error) {
          const errorMsg = `${this.t('proxy.modeChangeFailed')}: ${error}`
          if (messageCallback) messageCallback('error', errorMsg)
          else this.notificationService.error(errorMsg)
        }
      }

      return false // 不需要关闭应用
    } catch (error) {
      if (messageCallback) {
        messageCallback('error', `切换代理模式失败: ${error}`)
      }
      console.error('切换代理模式失败:', error)
      return false
    }
  }
}
