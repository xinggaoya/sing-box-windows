# 调试工具指南（CrabNebula DevTools）

本文说明本项目开发态的调试工具链：**CrabNebula DevTools**（Tauri 2 官方推荐的调试方案），
以及如何用它在开发期定位「内核事件没到前端」「某命令卡住」「跨模块状态不同步」这类问题。

> 接入设计见 [`docs/design-crabnebula-devtools.md`](./design-crabnebula-devtools.md)；
> 给**其他 Tauri 项目**复用时，直接抄 [`docs/templates/tauri-devtools/`](./templates/tauri-devtools/)。

## 1. 快速开始

```bash
pnpm kernel:fetch      # 首次构建前必须
pnpm tauri dev         # 自动带 --features devtools
```

启动后终端会打印：

```
   Tauri Devtools v2.2.3
   →   Local:   https://devtools.crabnebula.dev/dash/127.0.0.1/3033
   Tauri Devtools App v2.1.4
```

**浏览器打开上面那行地址**即可使用面板（standalone 形态，零安装）；
也可装 [CrabNebula 桌面客户端](https://crabnebula.dev/devtools/) 获得更完整体验。

应用内 `Ctrl+Shift+M`（macOS 为 `Cmd+Shift+M`）或窗口右键「Open DevTools」
可打开 **embedded 形态**抽屉面板。

> embedded 形态的 UI 从 `devtools.crabnebula.dev` 远程加载，**需要外网**；
> 离线环境请用 standalone 网页形态。

## 2. 面板能看什么

| 能力 | 说明 |
|---|---|
| 日志流 | 全量 `tracing` 事件（含依赖库），可按级别/target/文本过滤 |
| Tauri command | 每次命令调用的 **payload、响应、耗时**与 span 树 |
| Tauri event | 每条事件的 payload 与派发时序 |
| 应用配置 | `tauri.conf.json` 解析结果 |
| 指标 | `GetMetrics`（当前内核返回空） |

典型用法：面板里按 `command` 维度排序，能直接看出**哪个 command 最慢**；
按 event 维度看，能确认**内核事件是否真的派发到了前端**。

## 3. 日志为什么没有"丢"

`tauri-plugin-devtools` 的 `Builder::try_init()` 会**抢占全局 tracing subscriber**（全局只能设置一次，
被占用时 `init()` 直接 panic）。因此开发态由插件持有全局 subscriber，
本进程的文件与终端输出改经官方 `Builder::attach_logger` **桥接**回插件：

```
tracing 事件 ─┬─→ devtools Layer    ─→ 面板（gRPC-Web → 127.0.0.1:3033）
              └─→ BridgeLayer ─┐
                               └─→ FileConsoleLogger（log::Log）─→ 文件 + 终端
```

实现见 `src-tauri/src/utils/log_bridge.rs`（过滤 + 行格式化）与
`src-tauri/src/utils/devtools_util.rs`（插件装配）。

需要注意的两点：

- BridgeLayer **只转发带 `message` 字段的事件**，结构化字段（`tracing::info!(port = 1, "…")` 的 `port`）
  不会落盘；本项目 68 处 `tracing` 事件调用全部带 `message`，实际无损失
- 过滤规则与 `EnvFilter` 对齐：非法 `RUST_LOG` 整串作废并回退默认指令，不会出现"静默失声"

## 4. 命令行 / 脚本化验证（无需打开浏览器）

面板消费的 gRPC-Web 接口可以直接 curl，用于自动化回归：

```bash
# 健康检查（插件服务是否活着）
curl -s -o /dev/null -w "%{http_code}\n" -X POST \
  "http://127.0.0.1:3033/grpc.health.v1.Health/Check" \
  -H "Content-Type: application/grpc-web+proto" -H "X-Grpc-Web: 1" \
  --data-binary $'\x00\x00\x00\x00\x02\x0a\x00'

# 订阅实时事件流（日志 / span / command payload）
curl -s -N -X POST \
  "http://127.0.0.1:3033/rs.devtools.instrument.Instrument/WatchUpdates" \
  -H "Content-Type: application/grpc-web+proto" -H "X-Grpc-Web: 1" \
  --data-binary $'\x00\x00\x00\x00\x00' | strings | grep -a url_test
```

> ⚠️ 二进制帧里含 NUL 字节，**不能**直接写在命令行参数里（argv 以 NUL 结尾会被截断），
> 必须先写文件再用 `--data-binary @frame.bin`。

## 5. 自动化操作真实窗口（UI 回归）

Tauri 在 Windows 上用 WebView2，可通过环境变量开 CDP 端口，让外部工具直接驱动真实窗口：

```bash
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9222" pnpm tauri dev
# 之后可用任何 CDP 客户端连接 http://127.0.0.1:9222
```

这样既能点真实 UI（含完整 Tauri IPC），又能用第 4 节的 gRPC 流核对后端调用。

## 6. 已知问题

- **偶发窗口"无响应"**：开发态下曾观察到窗口偶发无响应（宿主主线程停止泵消息，
  此时 IPC 不再派发，但 Rust 侧后台任务与插件 gRPC 服务仍存活）。
  目前**未定位到根因**，与插件/拖拽的关联性也未确认（带插件与不带插件的构建都出现过与未出现过）。
  复现时请记录：构建（带/不带 `devtools` feature）、触发动作、当时停留的页面。
- **出站总览页偶发一直 loading**：`get_outbounds` 走 sing-box 的 **server-streaming** 接口
  （`SubscribeOutbounds` 首帧），而 streaming 请求**没有超时**；内核未就绪时该请求会长时间挂起，
  页面停在 loading 且不会自动重试（需手动点「刷新」）。
- embedded 形态依赖外网；Windows ARM64 没有 CrabNebula 桌面客户端（网页形态不受影响）。

## 7. 常见问题

| 现象 | 原因与处理 |
|---|---|
| 终端没有面板地址 | 没带 `devtools` feature：`pnpm tauri dev` 会自动注入；手动 `cargo run` 时需 `--features devtools` |
| 面板打不开 | 先 `curl http://127.0.0.1:3033/...Health/Check` 确认插件服务活着；端口被占用时插件会自动换端口，以终端打印的地址为准 |
| 面板里没有本项目日志 | 检查 `RUST_LOG` 是否设成了空串或非法值（非法会回退默认，不会全静默） |
| release 构建里没有面板 | 预期行为：`pnpm tauri build` 不带该 feature，产物不含插件 |
