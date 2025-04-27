import WebSocket from '@tauri-apps/plugin-websocket';
import { useConnectionStore } from '@/stores/kernel/ConnectionStore';
import { useTrafficStore } from '@/stores/kernel/TrafficStore';
import { useLogStore } from '@/stores/kernel/LogStore';
import mitt from '@/utils/mitt';

/**
 * WebSocket 连接状态接口
 */
export interface ConnectionState {
  connected: boolean;
  connecting: boolean;
  error: Error | null;
}

/**
 * WebSocket 服务类 - 单例模式
 * 管理所有 WebSocket 连接
 */
export class WebSocketService {
  private static instance: WebSocketService;
  private token: string = '';
  private connectionWs: WebSocket | null = null;
  private trafficWs: WebSocket | null = null;
  private logWs: WebSocket | null = null;
  private memoryWs: WebSocket | null = null;
  
  // 连接状态跟踪
  private hasActiveConnection: boolean = false;
  
  // 连接状态标志
  private connectionIsClosing: boolean = false;
  private trafficIsClosing: boolean = false;
  private logIsClosing: boolean = false;
  private memoryIsClosing: boolean = false;
  
  // 重连计时器
  private reconnectTimers: Record<string, number | null> = {
    connections: null,
    traffic: null,
    logs: null,
    memory: null
  };

  private constructor() {}

  /**
   * 获取 WebSocketService 实例
   */
  public static getInstance(): WebSocketService {
    if (!WebSocketService.instance) {
      WebSocketService.instance = new WebSocketService();
    }
    return WebSocketService.instance;
  }

  /**
   * 设置 API Token
   */
  public setToken(token: string) {
    this.token = token;
  }
  
  /**
   * 更新WebSocket连接状态并发出事件
   * @param isConnected 是否已连接
   */
  private updateConnectionStatus(isConnected: boolean) {
    // 如果状态发生变化，才发送事件
    if (this.hasActiveConnection !== isConnected) {
      this.hasActiveConnection = isConnected;
      
      if (isConnected) {
        console.log('WebSocket连接已建立，发送ws-connected事件');
        mitt.emit('ws-connected');
      } else {
        console.log('WebSocket连接已断开，发送ws-disconnected事件');
        mitt.emit('ws-disconnected');
      }
    }
    
    // 更新连接状态
    this.checkConnectionStatus();
  }
  
  /**
   * 检查连接状态
   * 如果任意一个WebSocket连接正常，则认为是连接状态
   */
  private checkConnectionStatus() {
    const isConnected = 
      this.connectionWs !== null || 
      this.trafficWs !== null || 
      this.logWs !== null || 
      this.memoryWs !== null;
    
    if (this.hasActiveConnection !== isConnected) {
      this.updateConnectionStatus(isConnected);
    }
  }
  
  /**
   * 清除特定类型的重连计时器
   */
  private clearReconnectTimer(type: string) {
    if (this.reconnectTimers[type]) {
      window.clearTimeout(this.reconnectTimers[type]!);
      this.reconnectTimers[type] = null;
    }
  }
  
  /**
   * 设置重连计时器
   */
  private scheduleReconnect(type: string, delay: number = 3000) {
    // 先清除可能存在的旧计时器
    this.clearReconnectTimer(type);
    
    // 设置新的重连计时器
    this.reconnectTimers[type] = window.setTimeout(() => {
      console.log(`尝试重新连接 ${type} WebSocket...`);
      this.connect(type).catch(err => {
        console.error(`重连 ${type} 失败:`, err);
        // 重连失败时，再次调度重连，延迟时间增加
        this.scheduleReconnect(type, Math.min(delay * 1.5, 30000));
      });
    }, delay);
  }

  /**
   * 连接特定类型的WebSocket
   * @param type WebSocket类型: 'connections' | 'traffic' | 'logs' | 'memory'
   * @returns 是否连接成功
   */
  public async connect(type: string): Promise<boolean> {
    try {
      // 如果正在关闭连接，等待一下
      if (this.isClosing(type)) {
        console.log(`${type} WebSocket正在关闭中，延迟重连...`);
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
      
      // 清除该类型的重连计时器
      this.clearReconnectTimer(type);
      
      let result = false;
      
      switch (type) {
        case 'connections':
          const connectionStore = useConnectionStore();
          result = await this.setupConnectionsListener(connectionStore);
          break;
        case 'traffic':
          const trafficStore = useTrafficStore();
          result = await this.setupTrafficListener(trafficStore);
          break;
        case 'logs':
          const logStore = useLogStore();
          result = await this.setupLogListener(logStore);
          break;
        case 'memory':
          const memoryStore = useConnectionStore();
          result = await this.setupMemoryListener(memoryStore);
          break;
        default:
          console.error(`未知的WebSocket类型: ${type}`);
          return false;
      }
      
      if (!result) {
        // 连接失败时设置重连
        this.scheduleReconnect(type);
      }
      
      // 连接后检查并更新连接状态
      this.checkConnectionStatus();
      return result;
    } catch (error) {
      console.error(`连接WebSocket失败 (${type}):`, error);
      // 连接异常时也设置重连
      this.scheduleReconnect(type);
      this.checkConnectionStatus();
      return false;
    }
  }
  
  /**
   * 判断特定WebSocket连接是否正在关闭
   */
  private isClosing(type: string): boolean {
    switch (type) {
      case 'connections':
        return this.connectionIsClosing;
      case 'traffic':
        return this.trafficIsClosing;
      case 'logs':
        return this.logIsClosing;
      case 'memory':
        return this.memoryIsClosing;
      default:
        return false;
    }
  }
  
  /**
   * 设置WebSocket连接的关闭状态
   */
  private setClosingState(type: string, isClosing: boolean) {
    switch (type) {
      case 'connections':
        this.connectionIsClosing = isClosing;
        break;
      case 'traffic':
        this.trafficIsClosing = isClosing;
        break;
      case 'logs':
        this.logIsClosing = isClosing;
        break;
      case 'memory':
        this.memoryIsClosing = isClosing;
        break;
    }
  }

  /**
   * 断开特定类型的WebSocket连接
   * @param type WebSocket类型: 'connections' | 'traffic' | 'logs' | 'memory'
   */
  public async disconnect(type: string): Promise<void> {
    // 清除重连计时器
    this.clearReconnectTimer(type);
    
    try {
      switch (type) {
        case 'connections':
          if (this.connectionWs) {
            this.setClosingState('connections', true);
            await this.connectionWs.disconnect();
            this.connectionWs = null;
            this.setClosingState('connections', false);
          }
          break;
        case 'traffic':
          if (this.trafficWs) {
            this.setClosingState('traffic', true);
            await this.trafficWs.disconnect();
            this.trafficWs = null;
            this.setClosingState('traffic', false);
          }
          break;
        case 'logs':
          if (this.logWs) {
            this.setClosingState('logs', true);
            await this.logWs.disconnect();
            this.logWs = null;
            this.setClosingState('logs', false);
          }
          break;
        case 'memory':
          if (this.memoryWs) {
            this.setClosingState('memory', true);
            await this.memoryWs.disconnect();
            this.memoryWs = null;
            this.setClosingState('memory', false);
          }
          break;
        default:
          console.error(`未知的WebSocket类型: ${type}`);
      }
      
      // 断开后检查并更新连接状态
      this.checkConnectionStatus();
    } catch (error) {
      console.error(`断开WebSocket连接失败 (${type}):`, error);
      // 即使发生错误，也重置状态和引用
      switch (type) {
        case 'connections':
          this.connectionWs = null;
          this.setClosingState('connections', false);
          break;
        case 'traffic':
          this.trafficWs = null;
          this.setClosingState('traffic', false);
          break;
        case 'logs':
          this.logWs = null;
          this.setClosingState('logs', false);
          break;
        case 'memory':
          this.memoryWs = null;
          this.setClosingState('memory', false);
          break;
      }
      this.checkConnectionStatus();
    }
  }

  /**
   * 检查所有连接
   * @returns 是否所有连接都成功建立
   */
  public async checkAllConnections(): Promise<boolean> {
    try {
      // 初始化所有连接
      const connectionStore = useConnectionStore();
      const trafficStore = useTrafficStore();
      const logStore = useLogStore();

      // 并行建立所有连接
      const results = await Promise.allSettled([
        this.setupConnectionsListener(connectionStore),
        this.setupTrafficListener(trafficStore),
        this.setupLogListener(logStore),
        this.setupMemoryListener(connectionStore)
      ]);
      
      // 建立连接后检查并更新连接状态
      this.checkConnectionStatus();

      // 检查是否所有连接都成功
      const allSuccess = results.every(result => result.status === 'fulfilled');
      
      // 如果不是所有连接都成功，尝试重连失败的连接
      if (!allSuccess) {
        const types = ['connections', 'traffic', 'logs', 'memory'];
        results.forEach((result, index) => {
          if (result.status === 'rejected') {
            this.scheduleReconnect(types[index]);
          }
        });
      }
      
      return allSuccess;
    } catch (error) {
      console.error('WebSocket 连接检查失败:', error);
      // 全部尝试重连
      ['connections', 'traffic', 'logs', 'memory'].forEach(type => {
        this.scheduleReconnect(type);
      });
      this.checkConnectionStatus();
      return false;
    }
  }

  /**
   * 断开所有连接
   */
  public async disconnectAll(): Promise<void> {
    // 清除所有重连计时器
    Object.keys(this.reconnectTimers).forEach(key => {
      this.clearReconnectTimer(key);
    });
    
    // 标记所有连接为正在关闭
    this.connectionIsClosing = true;
    this.trafficIsClosing = true;
    this.logIsClosing = true;
    this.memoryIsClosing = true;
    
    try {
      const disconnectPromises = [];
      
      // 收集所有需要断开的连接
      if (this.connectionWs) disconnectPromises.push(this.connectionWs.disconnect().catch(e => console.error('断开连接WebSocket失败:', e)));
      if (this.trafficWs) disconnectPromises.push(this.trafficWs.disconnect().catch(e => console.error('断开流量WebSocket失败:', e)));
      if (this.logWs) disconnectPromises.push(this.logWs.disconnect().catch(e => console.error('断开日志WebSocket失败:', e)));
      if (this.memoryWs) disconnectPromises.push(this.memoryWs.disconnect().catch(e => console.error('断开内存WebSocket失败:', e)));

      // 并行断开所有连接
      if (disconnectPromises.length > 0) {
        await Promise.allSettled(disconnectPromises);
      }

      // 重置连接
      this.connectionWs = null;
      this.trafficWs = null;
      this.logWs = null;
      this.memoryWs = null;
      
      // 断开后更新连接状态
      this.updateConnectionStatus(false);
    } catch (error) {
      console.error('断开所有 WebSocket 连接失败:', error);
    } finally {
      // 无论成功失败，都重置状态标志
      this.connectionIsClosing = false;
      this.trafficIsClosing = false;
      this.logIsClosing = false;
      this.memoryIsClosing = false;
      this.checkConnectionStatus();
    }
  }

  /**
   * 建立连接监听器
   */
  private async setupConnectionsListener(connectionStore: ReturnType<typeof useConnectionStore>): Promise<boolean> {
    try {
      // 断开旧连接
      if (this.connectionWs) {
        this.connectionIsClosing = true;
        try {
          await this.connectionWs.disconnect();
        } catch (e) {
          console.error('断开旧连接 WebSocket 失败:', e);
        } finally {
          this.connectionWs = null;
          this.connectionIsClosing = false;
        }
      }

      // 建立新连接
      this.connectionWs = await WebSocket.connect(`ws://127.0.0.1:12081/connections?token=${this.token}`);
      
      // 添加消息监听器
      this.connectionWs.addListener(message => {
        try {
          if (message.data) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            connectionStore.updateConnections(data);
          }
        } catch (error) {
          console.error('解析连接数据失败:', error);
        }
      });
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置连接监听器失败:', error);
      this.connectionWs = null;
      return false;
    }
  }

  /**
   * 建立流量监听器
   */
  private async setupTrafficListener(trafficStore: ReturnType<typeof useTrafficStore>): Promise<boolean> {
    try {
      // 断开旧连接
      if (this.trafficWs) {
        this.trafficIsClosing = true;
        try {
          await this.trafficWs.disconnect();
        } catch (e) {
          console.error('断开旧流量 WebSocket 失败:', e);
        } finally {
          this.trafficWs = null;
          this.trafficIsClosing = false;
        }
      }

      // 建立新连接
      this.trafficWs = await WebSocket.connect(`ws://127.0.0.1:12081/traffic?token=${this.token}`);
      
      // 添加消息监听器
      this.trafficWs.addListener(message => {
        try {
          if (message.data && this.trafficWs && !this.trafficIsClosing) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            trafficStore.updateTrafficStats(data);
          }
        } catch (error) {
          console.error('解析流量数据失败:', error);
        }
      });
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置流量监听器失败:', error);
      this.trafficWs = null;
      return false;
    }
  }

  /**
   * 建立日志监听器
   */
  private async setupLogListener(logStore: ReturnType<typeof useLogStore>): Promise<boolean> {
    try {
      // 断开旧连接
      if (this.logWs) {
        this.logIsClosing = true;
        try {
          await this.logWs.disconnect();
        } catch (e) {
          console.error('断开旧日志 WebSocket 失败:', e);
        } finally {
          this.logWs = null;
          this.logIsClosing = false;
        }
      }

      // 建立新连接
      this.logWs = await WebSocket.connect(`ws://127.0.0.1:12081/logs?token=${this.token}`);
      
      // 添加消息监听器
      this.logWs.addListener(message => {
        try {
          if (message.data && this.logWs && !this.logIsClosing) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            if (data && typeof data.type === 'string' && typeof data.payload === 'string') {
              logStore.addLog(data.type, data.payload);
            }
          }
        } catch (error) {
          console.error('解析日志数据失败:', error);
        }
      });
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置日志监听器失败:', error);
      this.logWs = null;
      return false;
    }
  }

  /**
   * 建立内存监听器
   */
  private async setupMemoryListener(connectionStore: ReturnType<typeof useConnectionStore>): Promise<boolean> {
    try {
      // 断开旧连接
      if (this.memoryWs) {
        this.memoryIsClosing = true;
        try {
          await this.memoryWs.disconnect();
        } catch (e) {
          console.error('断开旧内存 WebSocket 失败:', e);
        } finally {
          this.memoryWs = null;
          this.memoryIsClosing = false;
        }
      }

      // 建立新连接
      this.memoryWs = await WebSocket.connect(`ws://127.0.0.1:12081/memory?token=${this.token}`);
      
      // 添加消息监听器
      this.memoryWs.addListener(message => {
        try {
          if (message.data && this.memoryWs && !this.memoryIsClosing) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            connectionStore.updateMemory(data);
          }
        } catch (error) {
          console.error('解析内存数据失败:', error);
        }
      });
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置内存监听器失败:', error);
      this.memoryWs = null;
      return false;
    }
  }
} 