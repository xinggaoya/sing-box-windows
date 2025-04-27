import WebSocket from '@tauri-apps/plugin-websocket';
import { useConnectionStore } from '@/stores/kernel/ConnectionStore';
import { useTrafficStore } from '@/stores/kernel/TrafficStore';
import { useLogStore } from '@/stores/kernel/LogStore';

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
   * 连接特定类型的WebSocket
   * @param type WebSocket类型: 'connections' | 'traffic' | 'logs' | 'memory'
   * @returns 是否连接成功
   */
  public async connect(type: string): Promise<boolean> {
    try {
      switch (type) {
        case 'connections':
          const connectionStore = useConnectionStore();
          return await this.setupConnectionsListener(connectionStore);
        case 'traffic':
          const trafficStore = useTrafficStore();
          return await this.setupTrafficListener(trafficStore);
        case 'logs':
          const logStore = useLogStore();
          return await this.setupLogListener(logStore);
        case 'memory':
          const memoryStore = useConnectionStore();
          return await this.setupMemoryListener(memoryStore);
        default:
          console.error(`未知的WebSocket类型: ${type}`);
          return false;
      }
    } catch (error) {
      console.error(`连接WebSocket失败 (${type}):`, error);
      return false;
    }
  }

  /**
   * 断开特定类型的WebSocket连接
   * @param type WebSocket类型: 'connections' | 'traffic' | 'logs' | 'memory'
   */
  public async disconnect(type: string): Promise<void> {
    try {
      switch (type) {
        case 'connections':
          if (this.connectionWs) {
            await this.connectionWs.disconnect();
            this.connectionWs = null;
          }
          break;
        case 'traffic':
          if (this.trafficWs) {
            await this.trafficWs.disconnect();
            this.trafficWs = null;
          }
          break;
        case 'logs':
          if (this.logWs) {
            await this.logWs.disconnect();
            this.logWs = null;
          }
          break;
        case 'memory':
          if (this.memoryWs) {
            await this.memoryWs.disconnect();
            this.memoryWs = null;
          }
          break;
        default:
          console.error(`未知的WebSocket类型: ${type}`);
      }
    } catch (error) {
      console.error(`断开WebSocket连接失败 (${type}):`, error);
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

      // 检查是否所有连接都成功
      return results.every(result => result.status === 'fulfilled');
    } catch (error) {
      console.error('WebSocket 连接检查失败:', error);
      return false;
    }
  }

  /**
   * 断开所有连接
   */
  public async disconnectAll(): Promise<void> {
    try {
      const connections = [
        this.connectionWs,
        this.trafficWs,
        this.logWs,
        this.memoryWs
      ];

      // 并行断开所有连接
      await Promise.all(
        connections
          .filter((conn): conn is WebSocket => conn !== null)
          .map(conn => conn.disconnect())
      );

      // 重置连接
      this.connectionWs = null;
      this.trafficWs = null;
      this.logWs = null;
      this.memoryWs = null;
    } catch (error) {
      console.error('断开 WebSocket 连接失败:', error);
    }
  }

  /**
   * 建立连接监听器
   */
  private async setupConnectionsListener(connectionStore: ReturnType<typeof useConnectionStore>): Promise<boolean> {
    try {
      // 断开旧连接
      if (this.connectionWs) {
        await this.connectionWs.disconnect();
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
      
      return true;
    } catch (error) {
      console.error('设置连接监听器失败:', error);
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
        await this.trafficWs.disconnect();
      }

      // 建立新连接
      this.trafficWs = await WebSocket.connect(`ws://127.0.0.1:12081/traffic?token=${this.token}`);
      
      // 添加消息监听器
      this.trafficWs.addListener(message => {
        try {
          if (message.data) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            trafficStore.updateTrafficStats(data);
          }
        } catch (error) {
          console.error('解析流量数据失败:', error);
        }
      });
      
      return true;
    } catch (error) {
      console.error('设置流量监听器失败:', error);
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
        await this.logWs.disconnect();
      }

      // 建立新连接
      this.logWs = await WebSocket.connect(`ws://127.0.0.1:12081/logs?token=${this.token}`);
      
      // 添加消息监听器
      this.logWs.addListener(message => {
        try {
          if (message.data) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            if (data && typeof data.type === 'string' && typeof data.payload === 'string') {
              logStore.addLog(data.type, data.payload);
            }
          }
        } catch (error) {
          console.error('解析日志数据失败:', error);
        }
      });
      
      return true;
    } catch (error) {
      console.error('设置日志监听器失败:', error);
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
        await this.memoryWs.disconnect();
      }

      // 建立新连接
      this.memoryWs = await WebSocket.connect(`ws://127.0.0.1:12081/memory?token=${this.token}`);
      
      // 添加消息监听器
      this.memoryWs.addListener(message => {
        try {
          if (message.data) {
            const data = JSON.parse(typeof message.data === 'string' ? message.data : JSON.stringify(message.data));
            connectionStore.updateMemory(data);
          }
        } catch (error) {
          console.error('解析内存数据失败:', error);
        }
      });
      
      return true;
    } catch (error) {
      console.error('设置内存监听器失败:', error);
      return false;
    }
  }
} 