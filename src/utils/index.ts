export const createWebSocket = (url: string, onMessage: (data: any) => void) => {
  if (typeof (WebSocket) === 'undefined') {
    alert('您的浏览器不支持socket')
  } else {
    const ws = new WebSocket(url)
    //  //连接发生错误的回调方法
    ws.onerror = function() {
      console.log('ws连接发生错误')
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
    }
  }
}