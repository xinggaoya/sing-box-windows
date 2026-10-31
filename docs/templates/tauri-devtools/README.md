# 模板：把 CrabNebula DevTools 接入任意 Tauri 2 项目

本目录是 `sing-box-windows` 已验证可用接入方案的**可复制模板**。
目标：让**后续的 Tauri 项目**也能直接用同一套调试工具（面板看日志 / command payload / event 时序），
并且**不污染 release 产物**。

> 实测来源：本项目 `sing-box-windows`（Tauri 2.11 + Vue 3），详见
> [`docs/devtools.md`](../../devtools.md) 与
> [`docs/design-crabnebula-devtools.md`](../../design-crabnebula-devtools.md)。

## 0. 前置

- Rust **≥ 1.88**（`tauri-plugin-devtools` 2.2.x 的 MSRV；顺带建议把 crate 的 `rust-version` 一并上调）
- 已有 `tracing` + `tracing-subscriber`（或至少愿意引入）
- 网络可访问 `devtools.crabnebula.dev`（面板 UI 远程加载）

## 1. Cargo.toml

```toml
[features]
default = []
# 开发态调试面板；release 构建不启用，产物不含插件
devtools = ["dep:tauri-plugin-devtools", "dep:tauri-plugin-devtools-app", "tauri/dynamic-acl"]

[dependencies]
log = "0.4"                                        # 日志桥接用
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tracing-appender = "0.2"                           # 非阻塞文件 writer
chrono = { version = "0.4", features = ["serde"] }  # UTC 时间戳
tauri-plugin-devtools = { version = "2.2.3", optional = true }
tauri-plugin-devtools-app = { version = "2.1.4", optional = true }
```

## 2. 日志桥接模块

复制 [`log_bridge.rs`](./log_bridge.rs) 到 `<crate>/src/utils/log_bridge.rs`：

- `LogDirectiveFilter`：RUST_LOG 风格过滤，**语义对齐 `tracing_subscriber::EnvFilter`**
  （非法指令整串作废并回退默认、空串不放行、裸 token 视为 `target=trace`、最长前缀优先）
- `FileConsoleLogger`：`log::Log` 适配器，把事件写进文件 + 终端
  - 行内容与结尾换行**一次性写入**（`NonBlocking` 是 lossy 通道，拆开写会掉换行导致行粘连）
  - 时间戳用 UTC + 微秒 + `Z`，与 fmt 层 `fmt::time::SystemTime` 一致
  - 文件 sink 不可用时可降级为**仅控制台**

## 3. 插件装配模块

复制 [`devtools_util.rs`](./devtools_util.rs) 到 `<crate>/src/utils/devtools_util.rs`：

- 用 `Builder::default().attach_logger(logger)` + **`try_init()`**（不要用 `init()`，内部 unwrap 会 panic）
- `try_init()` 失败时回退：把日志 sink 交回调用方，改走直连 fmt subscriber，应用照常启动

## 4. 主入口接线（顺序是关键）

```rust
pub fn run() {
    // ① 准备日志 sink（日志目录 + 非阻塞文件 writer），两种模式共用
    let log_sink = log_util::create_log_sink();

    // ② devtools 模式：插件接管全局 tracing subscriber，日志经 log::Log 桥接输出
    #[cfg(all(feature = "devtools", debug_assertions))]
    let devtools_plugins = {
        match crate::utils::devtools_util::init(log_sink) {
            crate::utils::devtools_util::DevtoolsInit::Ready { probe, app } => Some((probe, app)),
            crate::utils::devtools_util::DevtoolsInit::Fallback { sink } => {
                log_util::install_direct_subscriber(sink.as_ref());   // 插件起不来也要有日志
                None
            }
        }
    };

    // ③ 直连模式：自行安装全局 fmt subscriber（release / 未启用 devtools feature）
    #[cfg(not(all(feature = "devtools", debug_assertions)))]
    log_util::install_direct_subscriber(log_sink.as_ref());

    let builder = tauri::Builder::default();
    // ④ devtools 模式：注册调试面板插件
    #[cfg(all(feature = "devtools", debug_assertions))]
    let builder = match devtools_plugins {
        Some((probe, app)) => builder.plugin(*probe).plugin(*app),
        None => builder,
    };
    builder
        // ...原有插件与 setup 逻辑不动
}
```

要点：

- **插件必须早于任何全局 subscriber 安装**，否则 `try_init()` 直接失败（全局 subscriber 只能设一次）
- `debug_assertions` 二次门控：即使有人手动 `cargo build --release --features devtools`，插件也不会启用

## 5. 让 dev 命令自动带上 feature

任选其一：

**A. Cargo feature + npm script**（最简单）

```json
{
  "scripts": {
    "tauri:dev": "tauri dev --features devtools"
  }
}
```

**B. 构建包装脚本注入**（本项目采用，适合有统一 wrapper 的项目）

```js
// scripts/tauri-wrapper.mjs
export function applyDevtoolsFeature(args) {
  const command = args[0] ?? ''
  if (command !== 'dev') return args            // build 系列不注入，release 零影响
  if (hasOption(args, '--features', '-f')) return args  // 用户已显式指定时不覆盖
  return [args[0], '--features', 'devtools', ...args.slice(1)]
}
```

并配一条单测锁住"release 不带 feature"这条护栏：

```js
test('applyDevtoolsFeature 只对 dev 命令注入 devtools feature', () => {
  assert.deepEqual(applyDevtoolsFeature(['dev']), ['dev', '--features', 'devtools'])
  assert.deepEqual(applyDevtoolsFeature(['build']), ['build'])
})
```

## 6. CI 门禁（两条编译路径都要过）

`lib.rs` 里 devtools 分支与直连分支是 **互补的 `cfg`**，只跑一种配置会让另一条路径失去编译期护栏：

```yaml
- name: Rust clippy (devtools feature)
  run: cargo clippy --all-targets --features devtools -- -D warnings
- name: Rust clippy (release path)
  run: cargo clippy --all-targets -- -D warnings
- name: Rust test (devtools feature)
  run: cargo test --features devtools
- name: Rust test (release path)
  run: cargo test
```

## 7. 验收清单

| # | 检查 | 期望 |
|---|---|---|
| 1 | `pnpm tauri dev` | 终端打印 `Tauri Devtools v2.x` + `https://devtools.crabnebula.dev/dash/127.0.0.1/<port>` |
| 2 | 浏览器打开该地址 | 面板出现，日志流实时滚动 |
| 3 | 点一次会打后端的按钮 | 面板里能看到对应 command 的 payload 与耗时 |
| 4 | 看终端与日志文件 | 开发态日志**没有中断**（桥接生效），格式与原 fmt 层一致 |
| 5 | `cargo tree -e normal`（不带 feature） | 搜不到 `tauri-plugin-devtools` |
| 6 | `cargo build --release` | 编译通过，产物不含插件 |
| 7 | `curl` 健康检查 | `grpc.health.v1.Health/Check` 返回 200 |

## 8. 坑位清单（接入时务必看）

1. **全局 subscriber 只能设一次**：devtools 插件会抢占，先装插件再装 fmt 层 → 顺序反了必 panic
2. **`init()` 会 panic**：用 `try_init()` 并准备回退路径
3. **BridgeLayer 只转发带 `message` 的事件**：没有 message 的事件不会进文件/终端日志
4. **lossy 通道会掉消息**：行内容与换行必须一次写入
5. **release 别带 feature**：Cargo **不支持**在 `[target.'cfg(debug_assertions)'.dependencies]` 用
   `cfg(debug_assertions)`，只能靠 optional dependency + feature 隔离
6. **MSRV 连带影响**：`rust-version` 上调到 1.88 后，clippy 会开始报 `chunks_exact_to_as_chunks` 之类
   之前被 MSRV 抑制的 lint，需要一并修
