# TAURI BACKEND KNOWLEDGE BASE

## OVERVIEW

Rust + Tauri 后端根目录，聚合命令注册、插件配置、平台打包配置与构建元信息。

## STRUCTURE

```
src-tauri/
├── src/            # 业务代码（app/entity/platform/process/types/utils + lib.rs/main.rs/error.rs）
│   └── app/singbox_api/  # sing-box 1.14+ 官方 gRPC API 客户端（gRPC-Web over HTTP/1.1）
├── types/generated/ # ts-rs 本地中间产物（gitignore），勿依赖勿修改
├── Cargo.toml      # 依赖、激进 release profile、平台差异依赖
├── tauri.conf.json # 应用窗口/打包配置（decorations:false，bundle 全平台目标）
├── resources/kernel/ # pnpm kernel:fetch 下载的内核二进制（不入库）
└── icons/          # 多平台图标资源
```

## WHERE TO LOOK

| Task           | Location          | Notes                               |
| -------------- | ----------------- | ----------------------------------- |
| 命令注册总入口 | `src/lib.rs`      | `invoke_handler`（87 个命令）暴露边界 |
| 进程主入口     | `src/main.rs`     | 仅启动入口，逻辑薄                  |
| 后端分层实现   | `src/app/`        | core/network/singbox/singbox_api/storage/system/tray |
| gRPC API 客户端 | `src/app/singbox_api/` | 替代 experimental.clash_api    |
| 应用内更新     | `src/app/system/update_service.rs` | 自研 GitHub Release 更新，非 tauri-plugin-updater |
| 平台差异实现   | `src/platform/` + `Cargo.toml` | cfg(windows/unix/macos)        |
| 打包行为       | `tauri.conf.json` | 窗口与 bundle 相关                  |
| 日志桥接（devtools） | `src/utils/log_bridge.rs` | RUST_LOG 指令过滤 + 行格式化 + `log::Log` 适配器 |
| devtools 插件装配   | `src/utils/devtools_util.rs` | 仅 `devtools` feature 编译；必须早于全局 subscriber 安装 |
| 日志目录 / 写入器   | `src/utils/log_util.rs` | 日志目录、轮转、清理任务、直连 fmt subscriber |

## CONVENTIONS

- Tauri command 返回值统一 `Result<T, String>`。
- 领域服务优先放在 `src/app/*`，避免在 `lib.rs` 堆业务逻辑。
- 新命令必须在 `lib.rs` 注册，且与前端 service 名称对齐。
- 异步任务放启动阶段后台执行，避免阻塞应用启动。

## ANTI-PATTERNS

- ❌ 在 `lib.rs` 直接写大量业务逻辑而不下沉模块。
- ❌ 命令签名不统一，导致前后端错误处理不一致。
- ❌ 平台相关逻辑不放 `platform/` 而散落在通用模块。

## COMMANDS

```bash
cd src-tauri && cargo clippy
cd src-tauri && cargo test
```

## NOTES

- `tauri_plugin_websocket` 仅在 `lib.rs` 注册一次，无重复注册；但运行态数据已改走 gRPC 订阅 → Tauri event，该插件属遗留。
- `Cargo.toml` 配置了激进的 release profile（LTO/codegen-units=1/strip/panic=abort），排障时注意 release 与 dev 行为差异（panic 直接 abort，不可恢复）。
- **API 协议**：内核交互使用 sing-box 1.14+ 官方 `type: "api"`（gRPC-Web over HTTP/1.1），由 `src/app/singbox_api/` 客户端实现；metacubexd 兼容层（`experimental.clash_api`）已彻底移除。
- **内嵌内核 1.14.4**：`src-tauri/resources/kernel/<平台>/<架构>/`（gitignore，构建期 `pnpm kernel:fetch` 拉取，覆盖 windows/amd64+arm64、linux/amd64、macos/arm64+amd64 共 5 个目标）。升级内核时先 diff 上游 proto 与 `service/api/web_bridge.go`——1.14.0→1.14.4 这段**逐字节未变**，所以 `singbox_api/` 无需跟版。内核版本号的兜底值在 `core/kernel_service/download.rs`（现为 1.14.4）。
- **规则与代理提供者管理**未实现：官方 gRPC API 未暴露对应管理接口，规则页仅只读展示（`get_rules`），等待上游 API 扩展。
- **应用更新**不走 tauri-plugin-updater，由 `system/update_service.rs` 自研（GitHub Releases API，应用内更新仅 Windows）。
- 单实例由 `tauri_plugin_single_instance` 插件实现（tauri.conf.json 无配置段）；窗口 `decorations: false` 自绘。
- 新增/修改命令后必须同步：`lib.rs` 注册、前端 service、对应层级 AGENTS.md（见根 AGENTS.md 的 DOC SYNC GATE）。
- **全局 tracing subscriber 只能设置一次**：`tauri-plugin-devtools` 的 `Builder::try_init` 会抢占全局 subscriber（已被占用时 `init()` 直接 panic）。`lib.rs::run()` 中 devtools 插件先于 fmt 层初始化；`log_util::install_direct_subscriber` 仅在非 devtools 模式（release / 未启用 `devtools` feature）调用。devtools 模式下日志经 `log_bridge::FileConsoleLogger` 桥接输出。用法与排障见根目录 `docs/devtools.md`，接入其它项目见 `docs/templates/tauri-devtools/`。
