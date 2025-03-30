import { Window } from '@tauri-apps/api/window'
import { useAppStore } from '@/stores/AppStore'
import { useInfoStore } from '@/stores/infoStore'
import { tauriApi } from './tauri-api'
import { NotificationService } from './notification-service'
import i18n from '@/locales'

export class ProxyService {
  private static instance: ProxyService
  private appStore = useAppStore()
  private infoStore = useInfoStore()
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
   * @param showMessage 消息提示函数（可选）
   * @returns 是否需要关闭窗口
   */
  public async switchMode(
    mode: 'system' | 'tun',
    showMessage?: (type: 'success' | 'info' | 'error', content: string) => void,
  ): Promise<boolean> {
    try {
      if (mode === 'system') {
        await tauriApi.proxy.setSystemProxy()
        this.appStore.proxyMode = 'system'
        if (showMessage)
          showMessage(
            'success',
            this.t('proxy.modeChangeSuccess', { mode: this.t('proxy.mode.global') }),
          )
        else
          this.notificationService.success(
            this.t('proxy.modeChangeSuccess', { mode: this.t('proxy.mode.global') }),
          )
      } else {
        // TUN模式需要管理员权限
        const isAdmin = await tauriApi.proxy.checkAdmin()
        if (!isAdmin) {
          try {
            await tauriApi.proxy.restartAsAdmin()
            return true // 需要关闭窗口
          } catch (error) {
            if (showMessage) showMessage('error', this.t('proxy.modeChangeError'))
            else this.notificationService.error(this.t('proxy.modeChangeError'))
            return false
          }
        }
        await tauriApi.proxy.setTunProxy()
        this.appStore.proxyMode = 'tun'
        if (showMessage)
          showMessage(
            'success',
            this.t('proxy.modeChangeSuccess', { mode: this.t('proxy.mode.tun') }),
          )
        else
          this.notificationService.success(
            this.t('proxy.modeChangeSuccess', { mode: this.t('proxy.mode.tun') }),
          )
      }

      // 如果内核正在运行，需要重启
      if (this.appStore.isRunning) {
        try {
          if (showMessage) showMessage('info', this.t('home.status.restarting'))
          else this.notificationService.info(this.t('home.status.restarting'))

          await this.infoStore.restartKernel()

          if (showMessage) showMessage('success', this.t('notification.kernelRestarted'))
          else this.notificationService.success(this.t('notification.kernelRestarted'))
        } catch (error) {
          const errorMsg = `${this.t('proxy.modeChangeFailed')}: ${error}`
          if (showMessage) showMessage('error', errorMsg)
          else this.notificationService.error(errorMsg)
        }
      }

      return false // 不需要关闭窗口
    } catch (error) {
      const errorMsg = `${this.t('proxy.modeChangeFailed')}: ${error}`
      if (showMessage) showMessage('error', errorMsg)
      else this.notificationService.error(errorMsg)
      return false
    }
  }
}
