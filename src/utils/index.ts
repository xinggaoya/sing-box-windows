// 定义 WebSocket 消息类型
interface WSTrafficData {
  up: number
  down: number
}

interface WSMemoryData {
  inuse: number
  oslimit: number
}

interface WSLogData {
  time: string
  level: string
  message: string
}

type WSData = WSTrafficData | WSMemoryData | WSLogData

export const createWebSocket = (
  url: string,
  onMessage: (data: WSData) => void,
  onClose?: () => void,
) => {
  if (typeof WebSocket === 'undefined') {
    alert('您的浏览器不支持socket')
    return
  }

  let ws: WebSocket | null = null
  let reconnectTimer: number | null = null

  const connect = () => {
    ws = new WebSocket(url)

    ws.onerror = () => {
      console.log('ws连接发生错误')
      onClose?.()
      scheduleReconnect()
    }

    ws.onopen = () => {
      console.log('ws连接成功')
      if (reconnectTimer) {
        window.clearTimeout(reconnectTimer)
        reconnectTimer = null
      }
    }

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as WSData
        onMessage(data)
      } catch (error) {
        console.error('解析消息失败:', error)
      }
    }

    ws.onclose = () => {
      console.log('ws连接关闭')
      onClose?.()
      scheduleReconnect()
    }
  }

  const scheduleReconnect = () => {
    if (!reconnectTimer) {
      reconnectTimer = window.setTimeout(() => {
        console.log('尝试重新连接...')
        connect()
      }, 3000) // 3秒后重连
    }
  }

  connect()

  // 返回清理函数
  return () => {
    if (ws) {
      ws.close()
    }
    if (reconnectTimer) {
      window.clearTimeout(reconnectTimer)
    }
  }
}

export function formatBandwidth(kbps: number) {
  kbps = kbps / 1024
  // 计算 MB/s 和 GB/s
  const mbps = kbps / 1024 // 将 KB/s 转为 MB/s
  const gbps = mbps / 1024 // 将 MB/s 转为 GB/s

  // 选择最佳单位
  let formattedBandwidth
  if (gbps >= 1) {
    formattedBandwidth = `${gbps.toFixed(2)} GB`
  } else if (mbps >= 1) {
    formattedBandwidth = `${mbps.toFixed(2)} MB`
  } else {
    formattedBandwidth = `${kbps.toFixed(2)} KB`
  }

  // 格式化输出，保持小数点后两位
  return formattedBandwidth
}
