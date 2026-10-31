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
| 路由 / DNS 规则语义 | `src/app/singbox/config_rules.rs` | 生成路径与补丁路径共用的规则定义与对齐（含 `action: resolve`） |
| 配置生成 / 补丁 | `src/app/singbox/config_generator.rs` / `settings_patch.rs` | 两条路径共享 `config_intent` 语义 |
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
- ⚠️ **路由规则里的 IP 匹配需要 `action: "resolve"`**：sing-box 1.11+ 起，`route.rules` 里的 `ip_cidr` / `geoip-*` 规则集**只在目标已经是 IP 时才命中**，目标仍是域名时**不会隐式解析**。因此 `geoip-cn`（按 IP 判定国内）必须排在一条 `{"action":"resolve"}` 规则之后，否则它对域名流量**完全不生效**——国内站点会一路落到 `final` 走代理。规则定义与顺序集中在 `src/app/singbox/config_rules.rs`（`build_route_rules` / `sync_route_rules`），**新增/调整规则只能改那里**，不要在两条路径里各写一份。
- ⚠️ **`dns.rules` 里不要写 `geoip-cn`**：IP 规则集在 DNS 阶段无法匹配"尚未解析的域名"（先有鸡还是先有蛋），且属于 sing-box 1.14 起废弃的 "Legacy Address Filter Fields"（1.16 移除，内核会打印弃用告警）。IP 判定一律交给 `route.rules`。
- ⚠️ **`dns.optimistic` 必须显式写出陈旧窗口**：裸 `optimistic: true` 隐含**3 天**的陈旧窗口，被污染的解析结果会被钉住 3 天，表现为"后台跑久了网络就坏，只能重启内核"（重启只是清空了内存缓存）。配置里始终写成 `{"enabled": true, "timeout": "<用户配置>"}`。
- ⚠️ **`urltest.interrupt_exist_connections` 保持 `false`**：urltest 每次测速都可能改选节点，开启后**所有**走该分组的长连接（视频 / 远程桌面 / 下载）会被成批掐断。sing-box 官方默认同样是 `false`。
