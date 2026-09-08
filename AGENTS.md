# PROJECT KNOWLEDGE BASE

**Updated:** 2026-09-07T00:00:00+08:00  
**Commit:** 89db6af  
**Branch:** master

## OVERVIEW

跨平台 Sing-Box 图形客户端（Windows x64/ARM64、Linux、macOS）。
前端为 Vue 3 + TypeScript，后端为 Tauri 2 + Rust；前后端通过 Tauri command + event 双通道通信。
内核交互使用 sing-box 1.14+ 官方 gRPC API（`type: "api"`，gRPC-Web 帧），Clash API（`experimental.clash_api`）兼容层已彻底移除。

## STRUCTURE

```
./
├── src/                 # 前端应用（路由/页面/store/service/composables）
├── src-tauri/           # Rust 后端 + Tauri 打包
│   └── src/
│       ├── app/             # 业务分层（core/network/singbox/singbox_api/storage/system/tray/constants）
│       ├── entity/          # 数据模型（config/github）
│       ├── platform/        # 平台特定实现（windows/linux/macos）
│       ├── process/         # 内核进程管理 ProcessManager
│       ├── types/generated/ # ts-rs 生成的 TS 类型镜像（如 CustomRule.ts）
│       └── utils/           # 通用工具（http/file/log/process 等）
├── scripts/             # 内核下载 fetch-kernel、tauri wrapper、目标矩阵（均带 .test.mjs）
├── docs/                # 开发文档（development/i18n/release-playbook/CHANGELOG 等）
└── .github/workflows/   # ci.yml（质量门禁）+ release.yml / promote-release.yml（两阶段发布）
```

## DOC SYNC GATE（提交前文档门禁）

> **规则：提交前若本次改动涉及功能变更、行为调整、接口/命令增删、结构或流程变化，必须先按需更新对应文档，再提交。**

提交前用 `git diff --stat` 扫一遍改动范围，按下表对照：

| 变更类型                        | 必须同步的文档                                                                             |
| ------------------------------- | ------------------------------------------------------------------------------------------ |
| 新增/修改 Tauri 命令、事件名、gRPC 接口 | 对应层级 AGENTS.md（根/src/src-tauri/src/app）；涉及通信架构时加 `docs/development.md`     |
| 内核版本 / sing-box API 协议变化 | `docs/sing-box-api-migration.md`、`src/AGENTS.md` 与 `src-tauri/AGENTS.md` 的 NOTES        |
| 用户可见功能变化（含 fix）       | `docs/CHANGELOG.md` 的 `[未发布]` 段                                                       |
| 构建脚本、依赖、npm scripts、CI  | 根 AGENTS.md 与 CLAUDE.md 的 COMMANDS 段；发布相关加 `docs/release-playbook.md`            |
| 目录结构、模块职责、启动链调整   | 对应层级 AGENTS.md 的 STRUCTURE / CODE MAP / WHERE TO LOOK                                 |
| i18n 新增语言或键结构调整        | `docs/i18n.md`                                                                             |

纯 fix 且不改变行为/接口时可不改结构文档，但仍建议补 CHANGELOG `[未发布]` 条目。

## WHERE TO LOOK

| Task                 | Location                                          | Notes                                    |
| -------------------- | ------------------------------------------------- | ---------------------------------------- |
| 前端启动链路         | `src/main.ts`, `src/boot/useAppBootstrap.ts`      | 初始化顺序与事件桥都在这里               |
| 前端调用后端         | `src/services/invoke-client.ts`                   | 统一入口（端口注入+数据恢复等待）        |
| 前端事件消费         | `src/services/event-service.ts` + `src/constants/events.ts` | 事件名常量约 27 个             |
| 路由与托盘空白页     | `src/router/index.ts`, `src/views/BlankView.vue`  | `/blank` 是非标准但关键路径              |
| gRPC API 客户端      | `src-tauri/src/app/singbox_api/`                  | 替代 experimental.clash_api              |
| 后端入口与命令注册   | `src-tauri/src/lib.rs`                            | 118 个命令；setup 同步+异步初始化链      |
| 模板市场与在线模板   | `src-tauri/src/app/template_marketplace/`         | 本地模板 KV 存储 + 市场客户端 + 17 命令  |
| 内核生命周期         | `src-tauri/src/app/core/kernel_service/`          | 高复杂度热点目录                         |
| 内核版本管理         | `src-tauri/src/app/core/kernel_service/versioning.rs` | 默认最新版；指定版本下载前前端强制确认兼容风险 |
| 订阅解析与模式切换   | `src-tauri/src/app/network/subscription_service/` | parser/mode/helpers 分层明显             |
| sing-box 配置生成    | `src-tauri/src/app/singbox/config_generator.rs`   | 1.14 DNS（mDNS/evaluate）与自定义规则注入 |
| 自定义规则 CRUD      | `src-tauri/src/app/core/proxy_service.rs` + `app/storage/custom_rule.rs` | 后端就绪，前端暂未接线 |
| 存储实现             | `src-tauri/src/app/storage/`                      | SQLite + OnceCell 单例初始化             |
| 应用内更新           | `src-tauri/src/app/system/update_service.rs`      | 自研 GitHub Release 下载，仅 Windows     |
| 托盘                 | `src-tauri/src/app/tray/`                         | 启动偏好、状态刷新、动作分发             |
| 发布流水线           | `.github/workflows/release.yml` + `promote-release.yml` | 两阶段发布，glibc 兼容检查         |

## CODE MAP

| Symbol / Area                 | Type               | Location                                                | Role                                |
| ----------------------------- | ------------------ | ------------------------------------------------------- | ----------------------------------- |
| `useAppBootstrap`             | Frontend bootstrap | `src/boot/useAppBootstrap.ts`                           | 统一串联 store 初始化与后端事件桥接 |
| `useTemplateStore`            | Template store     | `src/stores/template/TemplateStore.ts`                  | 本地模板/市场设置/激活模板状态      |
| `run()`                       | Backend entry      | `src-tauri/src/lib.rs`                                  | 插件注册、异步初始化链、命令暴露    |
| `kernel_start_enhanced` 等    | Tauri commands     | `src-tauri/src/app/core/kernel_service/*`               | 内核启停与健康检查                  |
| `get_groups`/`get_rules` 等   | Tauri commands     | `src-tauri/src/app/core/proxy_service.rs`               | gRPC 代理查询/选择/测速/自定义规则  |
| `download_subscription` 等    | Tauri commands     | `src-tauri/src/app/network/subscription_service.rs`     | 订阅下载、切换、回滚                |
| `EnhancedStorageService`      | Storage service    | `src-tauri/src/app/storage/enhanced_storage_service.rs` | 应用配置与结构化数据持久化          |
| `config_generator`            | Config builder     | `src-tauri/src/app/singbox/config_generator.rs`         | 规则/outbounds/DNS 组装与 1.14 注入 |
| `event.rs` 中继               | Event relay        | `src-tauri/src/app/core/kernel_service/event.rs`        | 4 个 gRPC server-streaming → Tauri event |

## CONVENTIONS

- 命名：组件 PascalCase，页面 `*View.vue`，组合式函数 `useXxx.ts`。
- 路径别名：`@/* -> src/*`。
- TS 禁止：`as any`、`@ts-ignore`、`@ts-nocheck`。
- Rust 约定：命令返回 `Result<T, String>`，模块/函数 snake_case。
- 跨语言类型：Rust 结构体经 ts-rs 导出到两侧 `types/generated/`，不要手改生成文件。
- 格式化：2 空格、`singleQuote: true`、`semi: false`、`printWidth: 100`。

## ANTI-PATTERNS (THIS PROJECT)

- ❌ 提交 secrets、日志、临时文件、本地覆盖文件。
- ❌ 为"过类型检查"使用类型压制。
- ❌ 跳过 `pnpm lint` / `pnpm type-check` / `cargo clippy`。
- ❌ 首次构建前不执行 `pnpm kernel:fetch`。
- ❌ 在 commit 中保留调试输出与一次性脚本改动。
- ❌ 功能变更不更新文档直接提交（见 DOC SYNC GATE）。

## COMMANDS

```bash
# 开发
pnpm install            # 首次
pnpm kernel:fetch       # 首次构建前必须；--all 为全平台（stable/oldstable/beta/testing 轨道）
pnpm tauri dev

# 质量门禁
pnpm lint
pnpm type-check
pnpm test:kernel-targets
cd src-tauri && cargo clippy
cd src-tauri && cargo test

# 构建（全部经 scripts/tauri-wrapper.mjs 包装）
pnpm tauri build
pnpm tauri build:windows:arm64
pnpm tauri build:linux:rpm
```

## UNIQUE STYLES

- 前端与后端都使用"事件驱动"：前端 `mitt` + 后端 Tauri event；运行态数据由后端 gRPC 订阅经 `event.rs` 转成 Tauri event 推送。
- 启动链是双阶段：`main.ts` 初始化 + `useAppBootstrap` 实际业务引导。
- 后端 `lib.rs` setup 后单任务顺序异步：存储初始化 → 启动恢复 → 残留进程清理 → 订阅升级刷新 → 内核自动管理 → 托盘状态刷新 → 后台任务 → 订阅自动更新。
- Linux CI 固定 Ubuntu 22.04，并强制 glibc <= 2.38 兼容性检查。

## NOTES

- 当前前端测试文件极少，回归验证以类型检查 + Rust 测试 + 实机联调为主；CI（`.github/workflows/ci.yml`）在 PR/push 时跑上述全部质量门禁。
- 发布为两阶段：tag 触发 `release.yml` 产 Pre-release（多平台矩阵，含 windows-arm64），再手动触发 `promote-release.yml` 转正式；Release Notes 取自 `docs/CHANGELOG.md`。
- 应用更新不走 tauri-plugin-updater，由 `system/update_service.rs` 自研（GitHub Releases API，应用内更新仅 Windows）。
- `tauri-plugin-websocket` 仍在 Rust 侧注册但属遗留；运行态数据链路为：后端 gRPC 订阅 → Tauri event → 前端 `event-service`。
- 分层 AGENTS 已在 `src/`、`src/stores/`、`src-tauri/`、`src-tauri/src/app/` 细化；改到哪层就核对哪层的文档。
