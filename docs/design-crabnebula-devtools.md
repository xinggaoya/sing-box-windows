# CrabNebula DevTools 调试能力接入设计

> 状态：待评审（2026-10-30）
> 目标版本：下一个次版本（开发期能力，不影响 release 产物）

## 1. 目标

为 `sing-box-windows` 接入 Tauri 2 官方推荐的 [CrabNebula DevTools](https://v2.tauri.app/zh-cn/develop/debug/crabnebula-devtools/)：

- 可视化查看**全部日志事件**（含依赖库），替代「终端刷屏 + 翻日志文件」
- 查看 **Tauri command 调用**的 payload、响应、耗时与 span 树
- 查看 **Tauri event** 的逐条 payload 与派发时序
- 对本项目价值最高的场景：定位「内核 gRPC 事件没到前端」「某 command 卡住」「订阅切换后状态不同步」这类跨模块链路问题

**约束**：release 构建产物**不含**任何 devtools 代码，且不因 feature 统一而打开 `tauri/tracing` 埋点。

## 2. 关键事实（已逐条核实）

| # | 事实 | 影响 |
|---|---|---|
| F1 | `tauri-plugin-devtools` 的 `Builder::try_init()` 内部执行 `tracing_subscriber::registry().with(...).try_init()`，抢占**全局** tracing subscriber；已被占用则返回 `Err`，而 `init()` 是 `unwrap()` → **panic** | 必须让 devtools 独占全局 subscriber，我们的日志输出改走官方 `attach_logger` 桥接 |
| F2 | Cargo **不支持**在 `[target.'cfg(debug_assertions)'.dependencies]` 用 `cfg(debug_assertions)`（Cargo 官方明确说明） | release 隔离只能靠 optional dependency + cargo feature |
| F3 | 最新版 `tauri-plugin-devtools = 2.2.3`（官方文档写的 `2.0.0` 已过时），要求 rustc ≥ **1.88**；`tauri-plugin-devtools-app = 2.1.4` 同 | 项目 `rust-version` 由 `1.77.2` 上调到 `1.88` |
| F4 | 插件默认监听 `127.0.0.1:3033`，被占用时自动改用 1025–65535 内可用端口（`strict_port=false`） | 与 sing-box `api_port`(28955) / `proxy_port` 不冲突；启动时终端打印 dash 链接 |
| F5 | 插件的 `BridgeLayer` 把 tracing 事件转成 `log::Record` 转发给 `attach_logger` 注入的 logger，**只带 message 文本**（结构化字段不透传） | 开发态文件日志不含结构化字段；项目 68 处 tracing 事件调用全部带 message 字段、无具名字段调用 → 实际无损失 |
| F6 | Tauri 的 CSP 只作用于本应用协议响应与 `data:` URL（`manager/webview.rs` 的注入点），**外部 URL 不受本应用 CSP 约束** | Embedded 模式（远程加载 devtools UI）**无需改** `tauri.conf.json` 的 CSP |
| F7 | `tauri-plugin-devtools-app` 要求 tauri 具备 `unstable` + `dynamic-acl`；本项目 tauri 2.11.5 默认特性已含 `dynamic-acl`，`unstable` 已显式开启 | 依赖约束天然满足 |
| F8 | `tauri dev --features <F>` / `tauri build --features <F>` 均受 CLI 支持 | 用 cargo feature 控制插件是否参与编译 |

## 3. 方案

### 3.1 依赖与编译隔离

```toml
# src-tauri/Cargo.toml
[features]
devtools = ["dep:tauri-plugin-devtools", "dep:tauri-plugin-devtools-app", "tauri/dynamic-acl"]

[dependencies]
tauri-plugin-devtools = { version = "2.2.3", optional = true }
tauri-plugin-devtools-app = { version = "2.1.4", optional = true }
```

- **开发态**：`scripts/tauri-wrapper.mjs` 在 `dev` 命令注入 `--features devtools`
- **发布态**：`build*` 命令不注入 → 插件不进依赖图，`tauri/tracing` 不开启，产物零影响
- **CI**：`cargo clippy --all-targets --features devtools` + `cargo test --features devtools`，保证 devtools 代码路径持续可编译（否则长期腐化）

### 3.2 日志桥接（保住现有文件日志 + 终端输出）

插件的全局 subscriber 只包含 `Layer`（喂给 DevTools）+ `BridgeLayer`（转发给 `log::Log`）。因此把「文件 + 控制台」实现成一个 `log::Log` 适配器挂上去，即可同时得到：DevTools 面板的日志流 **和** 原有文件/终端日志。

新增 `src-tauri/src/utils/log_bridge.rs`：

- `LogDirectiveFilter`：解析 RUST_LOG 风格指令，**语义对齐 `tracing_subscriber::EnvFilter`**（否则两种日志模式会“反着来”）：
  - 支持 `target=level`、全局 `level`、裸 `target`（等价 `target=trace`）
  - 任一指令非法 → **整串作废**并回退默认指令串（与 `EnvFilter::parse` + `build_env_filter` 的兜底一致）
  - `RUST_LOG` 未设置 → 默认指令串；`RUST_LOG` 为空 → 不放行任何日志（与 `EnvFilter::parse("")` 一致）
  - target 前缀最长匹配优先；同一 target 重复时后者覆盖前者
- `FileConsoleLogger`：实现 `log::Log`，输出格式对齐现有 fmt 层：`<UTC 时间戳> <LEVEL> <ThreadId> <file>:<line> <message>`（时间戳用 `Utc` + 微秒精度 + `Z`，与 fmt 层 `fmt::time::SystemTime` 一致）；写入 `tracing_appender` 非阻塞文件 writer + stdout；文件 sink 不可用时降级为**仅控制台**

`src-tauri/src/utils/log_util.rs` 拆分（保持单一职责）：

| 职责 | 归属 |
|---|---|
| 日志目录准备 / 文件 writer 创建 / 轮转 / 定时清理 | 保留在 `log_util.rs` |
| `create_log_sink() -> Option<(NonBlocking, PathBuf)>` | 抽出到 `log_util.rs`，两种模式共用 |
| 设置全局 fmt subscriber | 保留 `init_logging()`（**仅非 devtools 模式**调用） |
| `log::Log` 适配器 | 新增 `log_bridge.rs`（**仅 devtools 模式**使用） |

`WorkerGuard` 仍由 `log_util` 的 `OnceLock` 持有，两种模式都要保证进程退出前刷盘。

### 3.3 插件初始化与接线顺序（顺序是正确性关键）

新增 `src-tauri/src/utils/devtools_util.rs`：

```rust
pub fn init(logger: Box<dyn log::Log>) -> (TauriPlugin<Wry>, TauriPlugin<Wry>)
```

- 构造 `tauri_plugin_devtools::Builder::default().attach_logger(logger)` → `.init()`（此刻全局 subscriber 归 devtools）
- 构造 `tauri_plugin_devtools_app::init()`（Embedded 形态）

`src-tauri/src/lib.rs::run()` 顺序：

```
① log_util::create_log_sink()                    // 准备日志目录 + 文件 writer（两种模式共用）
② devtools 插件初始化（仅 debug_assertions + feature=devtools）
     └─ attach_logger(桥接 logger) → try_init()   // ← 全局 subscriber 在此被 devtools 接管
        （用 try_init 而非 init：init 内部 unwrap，抢占失败会 panic；
          失败时回退到直连日志模式，应用照常启动）
③ 仅非 devtools 模式：log_util::install_direct_subscriber()  // release / 未启用 feature 时才设置全局 fmt subscriber
④ builder.plugin(devtools).plugin(devtools_app)
⑤ 其余原有启动流程（托盘 / 存储 / 内核 / 后台任务）完全不动
```

### 3.4 两种 UI 形态

| 形态 | 入口 | 依赖 | 说明 |
|---|---|---|---|
| Standalone | 终端打印的 `https://devtools.crabnebula.dev/dash/{ip}/{port}`，浏览器打开；或 CrabNebula 桌面客户端 | 仅 `tauri-plugin-devtools` | 零侵入，不改配置 |
| Embedded | 应用内右键「Open DevTools」/ `Ctrl+Shift+M`（Win/Linux）、`Cmd+Shift+M`（macOS） | 追加 `tauri-plugin-devtools-app` | UI 由 `devtools.crabnebula.dev` 远程加载 → **需外网**；Windows ARM64 上桌面客户端仅有 x86_64，standalone 网页形态不受影响 |

### 3.5 各模式行为差异

| 项 | devtools 模式（debug + feature） | release / 默认 |
|---|---|---|
| 全局 tracing subscriber | devtools registry（`Layer` + `BridgeLayer`） | 我们的 fmt 层 |
| 终端 / 文件日志 | 经 `BridgeLayer` → `FileConsoleLogger`（含 message，不含结构化字段） | fmt 层（含结构化字段与 span 上下文） |
| 插件初始化失败 | 回退到直连日志模式（面板不可用，应用照常启动） | 不涉及 |
| `tauri/tracing` 内部埋点 | 开启 | 关闭 |
| DevTools 面板 | 可用 | 不可用（插件未编译） |

## 4. 风险与取舍

1. **开发态日志格式轻微退化**：结构化字段（`tracing::info!(port = 123, "…")`）不进文件日志。当前代码库无此类调用，风险可控；若后续大量使用，需升级为直连 `devtools-core` 自组层（方案 B）。
2. **依赖 `log` crate 桥接**：devtools 插件的 `BridgeLayer` 是官方兼容路径（`tauri-plugin-log` 兼容即由此实现），非私有 API。
3. **Embedded 模式需外网**：内网/离线开发环境只能用 standalone 网页形态。
4. **MSRV 上调**：1.77.2 → 1.88（本机 1.99、CI `dtolnay/rust-toolchain@stable` 均满足）。

## 5. 验收标准

1. `pnpm tauri dev` 启动后终端打印 devtools dash 链接
2. 浏览器打开该链接：可见日志流、Tauri command/event 的 payload 与耗时
3. 开发态终端与文件日志仍正常输出，格式与现状基本一致
4. `pnpm tauri build`（release）产物不含 devtools 代码；`cargo tree`（不带 feature）中无 `tauri-plugin-devtools`
5. `cargo test --features devtools` 全绿，含新增 `log_bridge` 单测（过滤匹配 + 行格式化）
6. `pnpm lint` / `pnpm type-check` / `pnpm format:check` / `cargo fmt` / `cargo clippy -D warnings` 全绿

## 6. 文档同步（DOC SYNC GATE）

- `docs/CHANGELOG.md` → `[未发布]` 新增「开发调试能力」条目
- 根 `AGENTS.md` → COMMANDS 段（`pnpm tauri dev` 自动启用 devtools）
- `src-tauri/AGENTS.md` → WHERE TO LOOK（新模块）+ NOTES（feature 隔离约定）
- `docs/development.md` → 通信链路未变，本次不改
