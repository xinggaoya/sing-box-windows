export const createWebSocket = (url: string, onMessage: (data: any) => void, onClose?: () => void) => {
  if (typeof (WebSocket) === 'undefined') {
    alert('您的浏览器不支持socket')
  } else {
    const ws = new WebSocket(url)
    //  //连接发生错误的回调方法
    ws.onerror = function() {
      console.log('ws连接发生错误')
      onClose?.()
      // 一秒后重新连接
      setTimeout(() => {
        createWebSocket(url, onMessage)
      }, 1000)
    }
    //连接成功建立的回调方法
    ws.onopen = function() {
      console.log('ws连接成功')
    }
    //接收到消息的回调方法
    ws.onmessage = function(event) {
      const data = JSON.parse(event.data)
      onMessage(data)
    }
    //连接关闭的回调方法
    ws.onclose = function() {
      console.log('ws连接关闭')
      onClose?.()
      // 一秒后重新连接
      setTimeout(() => {
        createWebSocket(url, onMessage)
      }, 1000)
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
