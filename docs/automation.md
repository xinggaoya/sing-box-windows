# 浏览器自动化测试

本项目的**端到端调试与回归测试**统一走浏览器：启动真实应用，用普通浏览器驱动前端，
并断言**真实后端数据**。

> 底座是外部插件 [tauri-plugin-web-dev-bridge](https://github.com/xinggaoya/tauri-plugin-web-dev-bridge)：
> 它在应用进程内架起本地 HTTP 桥，把浏览器的请求还原成**真正的** Tauri IPC 消息，
> 投递给应用**自身**的命令处理器。因此浏览器里看到的就是真实数据库、真实内核 gRPC 查询与真实事件流，
> **不是**任何形式的前端 mock。

---

## 1. 前提

| 条件                   | 说明                                                                                                            |
| ---------------------- | --------------------------------------------------------------------------------------------------------------- |
| 桌面应用已启动         | `pnpm tauri dev`（桥活在应用进程里）                                                                            |
| **桌面窗口处于可见状态** | 应用最小化到托盘时，前端会走托盘驻留路由 `/blank`，浏览器端也会跟着停在空白页（见第 6 节坑位）                   |
| vite dev server 已启动 | 默认 `http://localhost:6221`                                                                                    |
| 浏览器自动化工具       | 本仓库推荐 [agent-browser](https://github.com/xinggaoya/agent-browser)；也可直接用 Playwright / Puppeteer / CDP |

### 启动

```bash
pnpm tauri dev
```

等桌面窗口出现后，桥会就绪：

```bash
curl http://127.0.0.1:6222/health
# {"ok":true,"pid":12345,"webview":"main"}
```

> **必须先启动桌面应用**。桥不在时前端会给出明确提示（「未找到调试桥」），
> 而不是一堆 `undefined` 报错。

### 重启前先清残留

本项目有**单实例**约束，且 vite dev server 是独立进程。
上一次没退干净时，新实例会出现「桥能连上、但 `/invoke` 报 `webview main 不可用`」的怪状
（主窗口刚建好就被销毁），或直接报 `Port 6221 is already in use`。

重启前执行：

```bash
taskkill //F //IM sing-box-windows.exe   # 应用本体
netstat -ano | grep ":6221"               # vite 还在则按 PID kill
```

---

## 2. agent-browser 速查

```bash
# 打开页面（返回后即可断言；耗时通常 < 2s）
agent-browser open http://localhost:6221/#/

# 查看可交互元素（带 @eN 引用）
agent-browser snapshot -i

# 操作
agent-browser click @e3
agent-browser fill @e1 "关键词"
agent-browser press Enter

# 断言
agent-browser console                 # 控制台输出（查报错）
agent-browser console --clear         # 先清历史，再看新输出
agent-browser network requests       # 网络请求（查桥调用）
agent-browser eval "document.body.innerText.slice(0,200)"   # 页面文本
agent-browser screenshot out.png

# 关闭
agent-browser close
```

**页面文本是判断数据是否真实的最快手段**——例如代理页应能看到真实节点名与**实时测速延迟**：

```bash
agent-browser eval "document.body.innerText.replace(/\s+/g,' ').slice(0,300)"
# "... 推荐: 纽约 | 1 节点 ... 测试 自动选优 纽约 242ms ..."
```

---

## 3. 典型任务

### 3.1 验证某个页面拿到了真实数据

```bash
agent-browser open "http://localhost:6221/#/proxy"
sleep 5
agent-browser eval "document.body.innerText.includes('ms')"   # 延迟单位 ms
agent-browser console                                            # 应无 error
```

> 页面停在 `#/blank` 时先确认桌面窗口是不是被最小化到托盘了——
> 那种状态下前端会主动切到托盘驻留路由，浏览器端也会跟着停住。

### 3.2 验证一次真实的写操作

在浏览器里点「切换节点」，等价于桌面窗口里点击——内核会真的被切换：

```bash
agent-browser open "http://localhost:6221/#/proxy"
agent-browser snapshot -i
agent-browser click @e12          # 某个节点
sleep 3
agent-browser eval "document.body.innerText.slice(0,200)"
```

也可以直接问后端确认（见第 4 节）。

### 3.3 回归检查清单

对每个改动过的页面，跑一遍：

```bash
for p in "" sub proxy outbounds connections log setting rules; do
  agent-browser open "http://localhost:6221/#/$p" >/dev/null
  sleep 2
  echo "--- /$p ---"
  agent-browser console | grep -i error || echo "no error"
done
```

### 3.4 抓日志

应用日志在 `<work_dir>/logs/app.log`；浏览器侧只看控制台：

```bash
agent-browser console --clear     # 先清空
# ... 触发问题 ...
agent-browser console
```

---

## 4. 直接用 HTTP 桥做断言

桥的接口是一等公民，**不需要浏览器**就能验证后端行为——适合放进 CI 或脚本：

```bash
# 读配置
curl -X POST http://127.0.0.1:6222/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"db_get_app_config","args":{}}'

# 读主题
curl -X POST http://127.0.0.1:6222/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"db_get_theme_config","args":{}}'

# 读内核状态
curl -X POST http://127.0.0.1:6222/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"kernel_get_status_enhanced_v2","args":{}}'
```

响应信封：

```json
{ "ok": true, "data": { } }
{ "ok": false, "error": "命令错误信息" }
```

**这个通道走的是应用真实的命令处理器**（同一套 ACL 校验与错误转换），
因此在脚本里断言 `ok: true` 与在桌面窗口里点一下是等价的。

---

## 5. 事件流

事件按**标签页**隔离（每个浏览器上下文一组独立的 Rust 监听器），用短轮询获取：

```bash
# 订阅
curl -X POST http://127.0.0.1:6222/invoke \
  -H 'Content-Type: application/json' \
  -d '{"cmd":"plugin:event|listen","args":{"event":"traffic-data"},"clientId":"probe"}'

# 取（把 since 换成上次返回的 cursor）
curl "http://127.0.0.1:6222/events?client=probe&since=0"
# {"ok":true,"cursor":12,"events":[{"seq":12,"event":"traffic-data","payload":{...}}]}
```

命令行下也可用 SSE 长连接：

```bash
curl -N "http://127.0.0.1:6222/events/stream?client=probe"
```

---

## 6. 常见坑

| 现象                               | 原因                                                        | 处理                                                                                              |
| ---------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| 控制台报「未找到调试桥」           | 桌面应用没启动                                              | 先 `pnpm tauri dev`，再刷新页面                                                                   |
| 桥 `/health` 正常，但 `/invoke` 报「webview `main` 不可用」 | **有残留的旧实例**：旧实例的 `single_instance` 互斥与端口占用会让新实例的主窗口刚建好就被销毁 | 彻底清干净再启动：`taskkill //F //IM sing-box-windows.exe`，并确认 6221 / 6222 端口已释放（`netstat -ano \| grep 622`） |
| `pnpm tauri dev` 报 `Port 6221 is already in use` | 上一次的 vite dev server 还活着 | 杀掉占用进程：`netstat -ano \| grep ":6221"` 拿到 PID 后 `taskkill //F //PID <pid>` |
| `agent-browser open` 一直不返回    | 页面有**长连接**（SSE / WebSocket），_networkidle_ 永不达成 | 本项目的事件通道已改为**短轮询**（1s 一次），正常应在 1~2s 内返回；若仍挂起，检查是否有其它长连接 |
| 控制台里出现**早已消失**的错误    | `agent-browser console` 保留历史日志                       | 先 `agent-browser console --clear`，再做操作、再看日志                                            |
| 页面还是旧版本 / 报错             | hash 路由下 `open` 同一 URL **不会**真正刷新页面            | `agent-browser eval "location.reload()"`                                                          |
| 首屏报「未找到调试桥」但刷新后正常 | vite 正在预构建新依赖，此刻请求会失败                       | 等待预构建完成后刷新                                                                              |
| 控制台出现 CORS 报错               | 桥只监听 `127.0.0.1`；页面来源与之不同源                    | 同源访问 `localhost:6221`；或收紧 `Builder::allow_origin()`                                       |
| 页面数据是空的                     | 内核未运行 / 没有订阅                                       | 桌面窗口里确认内核状态；流量类数据需要内核在跑                                                    |
| 页面停在 `#/blank`（一片空白）     | 桌面应用被**最小化到托盘**了，前端走了托盘驻留路由           | 把桌面窗口恢复出来（托盘图标 → 显示主窗口），再 `location.hash = '#/proxy'`                       |
| `network requests` 里 OPTIONS 很多 | CORS 预检未被缓存                                           | 桥已带 `Access-Control-Max-Age: 86400`，预检只发生一次；持续出现说明跨域来源变了                  |

---

## 7. 与其它测试的分工

| 层次          | 工具                        | 覆盖                             |
| ------------- | --------------------------- | -------------------------------- |
| 单元测试      | `pnpm test`（vitest）       | store / 组合式函数 / 纯逻辑      |
| Rust 单元测试 | `cargo test`                | 后端业务函数、配置生成           |
| 端到端 / 回归 | **本文件：浏览器 + 调试桥** | 真实渲染、真实 IPC、真实内核交互 |

单元测试跑得快但看不到「界面 + 真实后端」这条链路；
浏览器自动化补上这一段，且**不需要打包桌面端**——改完前端热更新即可重测。
