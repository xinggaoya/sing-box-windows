import { MessageType } from '@/stores/kernel/LogStore'
import { useLogStore } from '@/stores/kernel/LogStore'

/**
 * 通知服务，提供统一的消息通知功能
 */
export class NotificationService {
  private static instance: NotificationService
  private messageHandler: ((type: MessageType, content: string) => void) | null = null
  private logStore = useLogStore()

  private constructor() {
    // 私有构造函数，防止外部直接创建实例
  }

  /**
   * 获取NotificationService单例
   */
  public static getInstance(): NotificationService {
    if (!NotificationService.instance) {
      NotificationService.instance = new NotificationService()
    }
    return NotificationService.instance
  }

  /**
   * 设置消息处理函数
   * @param handler 处理消息的回调函数
   */
  public setMessageHandler(handler: (type: MessageType, content: string) => void): void {
    this.messageHandler = handler
    this.logStore.setMessageCallback(handler)
  }

  /**
   * 显示成功消息
   * @param content 消息内容
   */
  public success(content: string): void {
    this.showMessage('success', content)
  }

  /**
   * 显示信息消息
   * @param content 消息内容
   */
  public info(content: string): void {
    this.showMessage('info', content)
  }

  /**
   * 显示警告消息
   * @param content 消息内容
   */
  public warning(content: string): void {
    this.showMessage('warning', content)
  }

  /**
   * 显示错误消息
   * @param content 消息内容
   */
  public error(content: string): void {
    this.showMessage('error', content)
  }

  /**
   * 显示消息的通用方法
   * @param type 消息类型
   * @param content 消息内容
   */
  private showMessage(type: MessageType, content: string): void {
    if (this.messageHandler) {
      this.messageHandler(type, content)
    } else {
      // 如果没有设置处理函数，使用LogStore提供的默认处理
      this.logStore.showMessage(type, content)
    }
  }
}
