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

  // 记录监听器移除函数
  private removeListenerFuncs: Record<string, (() => void) | null> = {
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
   * 清除WebSocket的监听器
   * @param type WebSocket类型
   */
  private cleanupListener(type: string) {
    if (this.removeListenerFuncs[type]) {
      console.log(`清除 ${type} WebSocket 监听器`);
      this.removeListenerFuncs[type]!();
      this.removeListenerFuncs[type] = null;
    }
  }

  /**
   * 断开特定类型的WebSocket连接
   * @param type WebSocket类型: 'connections' | 'traffic' | 'logs' | 'memory'
   */
  public async disconnect(type: string): Promise<void> {
    // 清除重连计时器
    this.clearReconnectTimer(type);
    
    // 获取相应的WebSocket引用
    let ws: WebSocket | null = null;
    let isClosing = false;
    
    switch (type) {
      case 'connections':
        ws = this.connectionWs;
        isClosing = this.connectionIsClosing;
        break;
      case 'traffic':
        ws = this.trafficWs;
        isClosing = this.trafficIsClosing;
        break;
      case 'logs':
        ws = this.logWs;
        isClosing = this.logIsClosing;
        break;
      case 'memory':
        ws = this.memoryWs;
        isClosing = this.memoryIsClosing;
        break;
      default:
        console.error(`未知的WebSocket类型: ${type}`);
        return;
    }
    
    // 如果连接不存在或已经在关闭中，直接返回
    if (!ws || isClosing) {
      console.log(`${type} WebSocket不存在或已经在关闭中，跳过断开操作`);
      return;
    }
    
    try {
      // 先清除监听器，避免disconnection期间仍接收消息
      this.cleanupListener(type);
      
      // 设置关闭状态标志
      this.setClosingState(type, true);
      
      try {
        // 尝试断开连接
        console.log(`正在断开 ${type} WebSocket连接...`);
        await Promise.race([
          ws.disconnect(),
          // 添加超时，避免永久等待
          new Promise((_, reject) => 
            setTimeout(() => reject(new Error(`断开${type}连接超时`)), 3000)
          )
        ]);
        console.log(`${type} WebSocket连接已成功断开`);
      } catch (disconnectError) {
        // 断开连接出错，但继续执行清理
        console.warn(`断开${type} WebSocket连接出错:`, disconnectError);
      }
      
      // 无论断开是否成功，都清理引用
      switch (type) {
        case 'connections':
          this.connectionWs = null;
          break;
        case 'traffic':
          this.trafficWs = null;
          break;
        case 'logs':
          this.logWs = null;
          break;
        case 'memory':
          this.memoryWs = null;
          break;
      }
      
      // 重置关闭状态
      this.setClosingState(type, false);
      
      // 断开后检查并更新连接状态
      this.checkConnectionStatus();
    } catch (error) {
      console.error(`处理WebSocket断开连接失败 (${type}):`, error);
      
      // 即使发生错误，也重置状态和引用
      this.cleanupListener(type);
      
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
    
    // 获取所有活跃连接的引用
    const activeConnections = [];
    
    // 收集当前活跃的WebSocket连接
    if (this.connectionWs && !this.connectionIsClosing) {
      activeConnections.push({type: 'connections', ws: this.connectionWs});
      this.connectionIsClosing = true;
    }
    
    if (this.trafficWs && !this.trafficIsClosing) {
      activeConnections.push({type: 'traffic', ws: this.trafficWs});
      this.trafficIsClosing = true;
    }
    
    if (this.logWs && !this.logIsClosing) {
      activeConnections.push({type: 'logs', ws: this.logWs});
      this.logIsClosing = true;
    }
    
    if (this.memoryWs && !this.memoryIsClosing) {
      activeConnections.push({type: 'memory', ws: this.memoryWs});
      this.memoryIsClosing = true;
    }
    
    try {
      // 清除所有监听器
      Object.keys(this.removeListenerFuncs).forEach(key => {
        this.cleanupListener(key);
      });
      
      // 并行断开所有连接
      if (activeConnections.length > 0) {
        console.log(`正在断开 ${activeConnections.length} 个WebSocket连接...`);
        
        await Promise.allSettled(
          activeConnections.map(({type, ws}) => 
            Promise.race([
              ws.disconnect().catch(e => console.warn(`断开${type} WebSocket失败:`, e)),
              // 添加超时
              new Promise((resolve) => setTimeout(resolve, 3000))
            ])
          )
        );
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
      // 清除可能存在的旧监听器
      this.cleanupListener('connections');
      
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
      console.log(`尝试连接连接管理WebSocket: ws://127.0.0.1:12081/connections?token=${this.token.substring(0, 5)}...`);
      this.connectionWs = await WebSocket.connect(`ws://127.0.0.1:12081/connections?token=${this.token}`);
      console.log('连接管理WebSocket连接成功');
      
      // 添加消息监听器
      const removeListener = this.connectionWs.addListener(message => {
        try {
          // 在处理消息前检查WebSocket状态
          if (!this.connectionWs || this.connectionIsClosing) {
            console.log('连接WebSocket已关闭或正在关闭，忽略新收到的消息');
            return;
          }
          
          if (!message.data) {
            console.warn('收到空的连接消息');
            return;
          }
          
          let data;
          if (typeof message.data === 'string') {
            data = JSON.parse(message.data);
          } else {
            data = JSON.parse(JSON.stringify(message.data));
          }
          
          // 更新连接数据
          connectionStore.updateConnections(data);
        } catch (error) {
          console.error('解析连接数据失败:', error);
        }
      });
      
      // 保存移除监听器的函数
      this.removeListenerFuncs['connections'] = removeListener;
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置连接监听器失败:', error);
      this.connectionWs = null;
      this.removeListenerFuncs['connections'] = null;
      return false;
    }
  }

  /**
   * 建立流量监听器
   */
  private async setupTrafficListener(trafficStore: ReturnType<typeof useTrafficStore>): Promise<boolean> {
    try {
      // 清除可能存在的旧监听器
      this.cleanupListener('traffic');
      
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
      console.log(`尝试连接流量WebSocket: ws://127.0.0.1:12081/traffic?token=${this.token.substring(0, 5)}...`);
      this.trafficWs = await WebSocket.connect(`ws://127.0.0.1:12081/traffic?token=${this.token}`);
      console.log('流量WebSocket连接成功');
      
      // 添加消息监听器
      const removeListener = this.trafficWs.addListener(message => {
        try {
          // 在处理消息前检查WebSocket状态
          if (!this.trafficWs || this.trafficIsClosing) {
            console.log('流量WebSocket已关闭或正在关闭，忽略新收到的消息');
            return;
          }
          
          if (!message.data) {
            console.warn('收到空的流量消息');
            return;
          }
          
          let data;
          if (typeof message.data === 'string') {
            data = JSON.parse(message.data);
          } else {
            data = JSON.parse(JSON.stringify(message.data));
          }
          
          // 更新流量数据
          trafficStore.updateTrafficStats(data);
        } catch (error) {
          console.error('解析流量数据失败:', error);
        }
      });
      
      // 保存移除监听器的函数
      this.removeListenerFuncs['traffic'] = removeListener;
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置流量监听器失败:', error);
      this.trafficWs = null;
      this.removeListenerFuncs['traffic'] = null;
      return false;
    }
  }

  /**
   * 建立日志监听器
   */
  private async setupLogListener(logStore: ReturnType<typeof useLogStore>): Promise<boolean> {
    try {
      // 清除可能存在的旧监听器
      this.cleanupListener('logs');
      
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
      console.log(`尝试连接日志WebSocket: ws://127.0.0.1:12081/logs?token=${this.token.substring(0, 5)}...`);
      this.logWs = await WebSocket.connect(`ws://127.0.0.1:12081/logs?token=${this.token}`);
      console.log('日志WebSocket连接成功');
      
      // 添加消息监听器
      const removeListener = this.logWs.addListener(message => {
        try {
          console.log('收到日志消息:', message);
          if (!message.data) {
            console.warn('收到空的日志消息');
            return;
          }
          
          if (!this.logWs || this.logIsClosing) {
            console.warn('日志WebSocket已关闭或正在关闭，丢弃消息');
            return;
          }
          
          let data;
          if (typeof message.data === 'string') {
            data = JSON.parse(message.data);
          } else {
            data = JSON.parse(JSON.stringify(message.data));
          }
          
          console.log('收到日志数据:', data);
          
          if (!data) {
            console.warn('解析后的日志数据为空');
            return;
          }
          
          if (typeof data.type !== 'string') {
            console.warn(`无效的日志类型: ${data.type}`);
            return;
          }
          
          if (typeof data.payload !== 'string') {
            console.warn(`无效的日志内容: ${data.payload}`);
            return;
          }
          
          // 发送到日志存储
          console.log(`添加日志: [${data.type}] ${data.payload}`);
          logStore.addLog(data.type, data.payload);
          
          // 同时触发Tauri事件，确保LogStore中的监听器也能收到数据
          const eventPayload = {
            type: data.type,
            payload: data.payload
          };
          
          // 使用mitt发出日志事件，确保事件总线也能收到
          mitt.emit('log-data', eventPayload);
          
        } catch (error) {
          console.error('处理日志数据失败:', error, '原始消息:', message.data);
        }
      });
      
      // 保存移除监听器的函数
      this.removeListenerFuncs['logs'] = removeListener;
      
      // 测试发送一条日志，确认监听器正常工作
      setTimeout(() => {
        logStore.addLog('info', '日志WebSocket连接已建立，等待内核数据...');
      }, 500);
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置日志监听器失败:', error);
      this.logWs = null;
      this.removeListenerFuncs['logs'] = null;
      
      // 即使WebSocket连接失败，也添加一条本地日志
      setTimeout(() => {
        logStore.addLog('error', `日志连接失败: ${error}`);
      }, 500);
      
      return false;
    }
  }

  /**
   * 建立内存监听器
   */
  private async setupMemoryListener(connectionStore: ReturnType<typeof useConnectionStore>): Promise<boolean> {
    try {
      // 清除可能存在的旧监听器
      this.cleanupListener('memory');
      
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
      console.log(`尝试连接内存WebSocket: ws://127.0.0.1:12081/memory?token=${this.token.substring(0, 5)}...`);
      this.memoryWs = await WebSocket.connect(`ws://127.0.0.1:12081/memory?token=${this.token}`);
      console.log('内存WebSocket连接成功');
      
      // 添加消息监听器
      const removeListener = this.memoryWs.addListener(message => {
        try {
          // 在处理消息前检查WebSocket状态
          if (!this.memoryWs || this.memoryIsClosing) {
            console.log('内存WebSocket已关闭或正在关闭，忽略新收到的消息');
            return;
          }
          
          if (!message.data) {
            console.warn('收到空的内存消息');
            return;
          }
          
          let data;
          if (typeof message.data === 'string') {
            data = JSON.parse(message.data);
          } else {
            data = JSON.parse(JSON.stringify(message.data));
          }
          
          // 更新内存数据
          connectionStore.updateMemory(data);
        } catch (error) {
          console.error('解析内存数据失败:', error);
        }
      });
      
      // 保存移除监听器的函数
      this.removeListenerFuncs['memory'] = removeListener;
      
      // 连接成功，更新状态
      this.updateConnectionStatus(true);
      return true;
    } catch (error) {
      console.error('设置内存监听器失败:', error);
      this.memoryWs = null;
      this.removeListenerFuncs['memory'] = null;
      return false;
    }
  }
} 