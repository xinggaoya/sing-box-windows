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
├── scripts/             # 内核下载 fetch-kernel、tauri wrapper、目标矩阵、i18n 校验、国家码生成
├── src-tauri/           # Rust 后端 + Tauri 打包
│   └── src/
│       ├── app/             # 业务分层（core/network/singbox/singbox_api/storage/system/tray/constants）
│       ├── entity/          # 数据模型（config/github）
│       ├── platform/        # 平台特定实现（windows/linux/macos）
│       ├── process/         # 内核进程管理 ProcessManager
│       ├── types/generated/ # ts-rs 本地中间产物（gitignore），勿依赖勿修改
│       └── utils/           # 通用工具（http/file/log/process 等）
├── docs/                # 开发文档（development/i18n/release-playbook/devtools/CHANGELOG 等）
│   └── templates/       # 可复制到其它项目的接入模板（tauri-devtools 等）
│                       # + gap-analysis-vs-mature-projects（竞品对标与改进批次建议）
│                       # + implementation-plan-2026-10（存量接线 + 配置双模型收敛实施方案）
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
| 前端事件消费         | `src/services/event-service.ts` + `src/constants/events.ts` | 事件名常量约 27 个；统一精确退订 |
| 应用配置字段         | `src/stores/app/composables/appConfigSchema.ts`     | 52 个字段的单一事实来源；load/save/watch 全自动生成 |
| TUN 切换状态机       | `src/composables/useTunToggle.ts`                    | 置位→应用→重启→失败回滚；含 sudo 重试与提权 |
| 排序/分组列表        | `src/composables/useSortableGroupedList.ts`          | 日志页与连接页共用 |
| 内核错误归因         | `src/utils/kernel-errors.ts`                         | sudo 错误码识别 + 失败文案三级回退 |
| store 初始化守卫     | `src/utils/store-init.ts`                             | `flushWatchers()` 取代 setTimeout 赌调度 |
| 路由与托盘空白页     | `src/router/index.ts`, `src/views/BlankView.vue`  | `/blank` 是非标准但关键路径              |
| 语言切换生效链路     | `src/stores/app/LocaleStore.ts`                    | `currentLocale` 必须同步到 `i18n.global.locale`，否则 store/service 层的 `i18n.global.t` 停在初始语言 |
| gRPC API 客户端      | `src-tauri/src/app/singbox_api/`                  | 替代 experimental.clash_api              |
| 后端入口与命令注册   | `src-tauri/src/lib.rs`                            | 87 个命令；setup 同步+异步初始化链       |
| 内核生命周期         | `src-tauri/src/app/core/kernel_service/`          | orchestrator/runtime/guard/state/event/archive/health |
| 内核存活判定         | `src-tauri/src/app/core/kernel_service/health.rs` | 唯一判定入口：端口 > 托管句柄 > 平台扫描 |
| 内核版本管理         | `src-tauri/src/app/core/kernel_service/versioning.rs` | 默认最新版；指定版本下载前前端强制确认兼容风险 |
| 订阅解析与模式切换   | `src-tauri/src/app/network/subscription_service/` | parser/mode/helpers 分层明显             |
| sing-box 配置生成    | `src-tauri/src/app/singbox/config_generator.rs`   | 1.14 DNS（mDNS）与自定义规则注入 |
| 配置语义唯一来源    | `src-tauri/src/app/singbox/config_intent.rs`     | `AppConfig` → 配置语义的纯映射，生成/补丁共用 |
| 配置渲染共享层       | `src-tauri/src/app/singbox/config_render.rs`      | 入口/假 DNS 等两条路径共用的纯函数 |
| 配置补丁与归属       | `src-tauri/src/app/singbox/settings_patch.rs` + `ownership.rs` | 按归属打补丁；外来配置只改端口 |
| 自定义规则 CRUD      | `src-tauri/src/app/core/proxy_service.rs` + `app/storage/custom_rule.rs` | 5 个命令，存 `generic_config` 表 key=`custom_rules` |
| 自定义规则 UI        | `src/components/rules/CustomRulePanel.vue` + `src/stores/kernel/CustomRuleStore.ts` | 规则页 CRUD 面板（走文件+重启） |
| 出站总览页面        | `src/views/OutboundsView.vue` + `src/composables/useOutboundList.ts` | 侧边栏「出站」独立页（路由 `/outbounds`），代理页零占用 |
| 网络诊断 / 服务状态  | `src/views/setting/components/SettingsDiagnosticsCard.vue` | 维护页诊断指标 + `probe_service_status` |
| 内核压缩包解压       | `src-tauri/src/app/core/kernel_service/archive.rs`   | zip/tar.gz/tgz/tar；阻塞 IO 在 spawn_blocking |
| 国家 / 地区常量     | `src/constants/countries.ts`                       | 国旗码由 scripts/gen-country-codes.mjs 生成 + 测试守护 |
| 节点名 → 国旗解析   | `src/composables/useNodeFlag.ts`                   | 地名正则 → 本地化国名 → 裸 token 三层兜底；注意 flag-icons 文件名恒为小写 |
| 首页运行环境/节点模式| `src/composables/useHomeDashboard.ts`               | 平台+管理员探测、节点模式、一键测速 |
| 订阅表单（三来源）  | `src/composables/useSubscriptionForm.ts`            | URL / 手动 JSON / 手动 URI 的校验与提交 |
| 页面样式           | `src/assets/views/*.css`                            | 用 `@import` 而非 script import，保留 scoped 语义 |
| 应用配置读写         | `src-tauri/src/app/storage/database.rs` + `enhanced_storage_service.rs` | WAL + busy_timeout + user_version；AppConfig 内存快照 |
| 存储实现             | `src-tauri/src/app/storage/`                      | SQLite + OnceCell 单例初始化             |
| 应用内更新           | `src-tauri/src/app/system/update_service.rs`      | 自研 GitHub Release 下载，仅 Windows     |
| 托盘                 | `src-tauri/src/app/tray/`                         | 启动偏好、状态刷新、动作分发             |
| 发布流水线           | `.github/workflows/release.yml` + `promote-release.yml` | 两阶段发布，glibc 兼容检查         |
| 竞品对标与改进清单   | `docs/gap-analysis-vs-mature-projects.md`          | 对标 Clash Verge Rev / v2rayN / FlClash 的差距与 5 批次建议 |
| 实施方案（存量接线 + 双模型收敛） | `docs/implementation-plan-2026-10.md`      | 阶段 0 类型基线 / 阶段 1 前端接线 / 阶段 2 配置双模型收敛（7 步） |
| 调试工具指南（DevTools） | `docs/devtools.md`                     | 开发态面板用法 / 日志桥接原理 / 命令行验证 / 已知问题 |
| 调试工具接入模板（给后续项目） | `docs/templates/tauri-devtools/README.md` | 任意 Tauri 2 项目接入同一套调试工具的步骤与坑位清单 |

## CODE MAP

| Symbol / Area                 | Type               | Location                                                | Role                                |
| ----------------------------- | ------------------ | ------------------------------------------------------- | ----------------------------------- |
| `useAppBootstrap`             | Frontend bootstrap | `src/boot/useAppBootstrap.ts`                           | 统一串联 store 初始化与后端事件桥接 |
| `run()`                       | Backend entry      | `src-tauri/src/lib.rs`                                  | 插件注册、异步初始化链、命令暴露    |
| `kernel_start_enhanced` 等    | Tauri commands     | `src-tauri/src/app/core/kernel_service/*`               | 内核启停与健康检查                  |
| `get_groups`/`get_rules` 等   | Tauri commands     | `src-tauri/src/app/core/proxy_service.rs`               | gRPC 代理查询/选择/测速/自定义规则  |
| `download_subscription` 等    | Tauri commands     | `src-tauri/src/app/network/subscription_service.rs`     | 订阅下载、切换、回滚                |
| `EnhancedStorageService`      | Storage service    | `src-tauri/src/app/storage/enhanced_storage_service.rs` | 应用配置与结构化数据持久化          |
| `config_generator`            | Config builder     | `src-tauri/src/app/singbox/config_generator.rs`         | 规则/outbounds/DNS 组装与 1.14 注入 |
| `ConfigIntent`                | Config semantics   | `src-tauri/src/app/singbox/config_intent.rs`            | `AppConfig` → 配置语义的唯一映射点    |
| `config_render`               | Config renderer    | `src-tauri/src/app/singbox/config_render.rs`             | 生成/补丁两条路径共用的纯函数         |
| `Ownership`                   | Config ownership   | `src-tauri/src/app/singbox/ownership.rs`                 | 本程序创建的 tag 清单，判定配置归属   |
| `event.rs` 中继               | Event relay        | `src-tauri/src/app/core/kernel_service/event.rs`        | 4 个 gRPC server-streaming → Tauri event |

## CONVENTIONS

- 命名：组件 PascalCase，页面 `*View.vue`，组合式函数 `useXxx.ts`。
- 路径别名：`@/* -> src/*`。
- TS 禁止：`as any`、`@ts-ignore`、`@ts-nocheck`。
- Rust 约定：命令返回 `Result<T, String>`，模块/函数 snake_case。
- 跨语言类型：Rust 结构体经 ts-rs 导出到 `src/types/generated/`（唯一事实来源，git 跟踪），不要手改生成文件；`src-tauri/src/types/generated/` 已 gitignore，是本地中间产物。
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
# 开发（dev 会自动加 --features devtools，启用 CrabNebula DevTools 调试面板）
pnpm install            # 首次
pnpm kernel:fetch       # 首次构建前必须；--all 为全平台（stable/oldstable/beta/testing 轨道）
pnpm tauri dev

# 质量门禁（CI 跑的就是这些）
pnpm lint
pnpm type-check
pnpm format:check
pnpm test:i18n
pnpm test
pnpm test:kernel-targets
cd src-tauri && cargo fmt --all -- --check
cd src-tauri && cargo clippy --all-targets --features devtools -- -D warnings
cd src-tauri && cargo test --features devtools

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

- 测试：后端 263 项（`cargo test`），前端 173 项（`vitest`，覆盖 TUN 切换状态机、排序分组、内核错误归因、事件退订、配置 schema、国旗码同步、订阅表单三来源、节点名→国旗解析、跨语言类型漂移守护等易回归逻辑）。CI（`.github/workflows/ci.yml`）在 PR/push 时跑 lint / type-check / prettier / i18n key 校验 / vitest / 内核目标矩阵 / 脚本单测 / cargo fmt / clippy / cargo test 全部门禁。
- **配置生成黄金样本**：`src-tauri/src/app/singbox/testdata/config_golden/*.json` 固化 4 组 `AppConfig` 的生成结果；改配置生成逻辑时必须让快照**逐字段不变**（`SINGBOX_BLESS_GOLDEN=1` 重新生成），否则说明配置语义漂移。
- **内嵌内核版本 1.14.4**（上游 2026-10-24 发布的最新稳定版；仓库内嵌资源在 `src-tauri/resources/kernel/<平台>/<架构>/`，gitignore，构建期由 `pnpm kernel:fetch` 拉取。**下载策略是「默认取最新稳定版」而非钉死版本**——钉版本需改 `scripts/fetch-kernel.mjs` 的默认解析或构建参数）。已逐文件 diff 上游 `v1.14.0...v1.14.4`（174 commits）确认：**gRPC API 协议层零变更**（`daemon/*.proto` 与 `service/api/web_bridge.go` 逐字节相同，`APIVersion` 仍为 4），手写 protobuf 与 gRPC-Web 客户端**不需要跟版**；配置 schema 变更均为内部 marshal 细节。**升级内核前先跑 `sing-box check -c <黄金配置>`**（用真实二进制）而不是只看 changelog。
- ⚠️ **`http_clients` 直连客户端不能写 `detour`**：sing-box 1.14+ 对「detour 指向空 direct 出站」直接 `FATAL`（`detour to an empty direct outbound makes no sense`），冷缓存时内核根本起不来。直连要表达成**不写 `detour`**（走系统 dialer）。统一由 `singbox/config_render.rs::build_http_clients` 渲染，生成与补丁两条路径共用。
- i18n：以 `zh-CN` 为基准，`pnpm test:i18n`（`scripts/check-i18n.mjs`）校验 4 个语言包 key 完全对齐；**禁止**在组件里写 `locale.startsWith('zh') ? '中文' : 'English'` 或 `t(key, '中文兜底')`，一律走 locale 语言包。
- 事件：统一走 `eventService`，并使用 `on*()` 返回的 unlisten 做**精确**退订；「清空某事件全部监听器」是显式的 `removeAllListenersForEvent()`。已移除 `mitt` 事件总线（事件链从 4 跳缩到 2 跳）。
- 发布为两阶段：tag 触发 `release.yml` 产 Pre-release（多平台矩阵，含 windows-arm64），再手动触发 `promote-release.yml` 转正式；Release Notes 取自 `docs/CHANGELOG.md`。
- 应用更新不走 tauri-plugin-updater，由 `system/update_service.rs` 自研（GitHub Releases API，应用内更新仅 Windows）。
- `tauri-plugin-websocket` 仍在 Rust 侧注册但属遗留；运行态数据链路为：后端 gRPC 订阅 → Tauri event → 前端 `event-service`。
- 分层 AGENTS 已在 `src/`、`src/stores/`、`src-tauri/`、`src-tauri/src/app/` 细化；改到哪层就核对哪层的文档。
- **开发调试面板**：`pnpm tauri dev` 会自动注入 `--features devtools`（`src-tauri/Cargo.toml` 的 `devtools` feature），启用 [CrabNebula DevTools](https://v2.tauri.app/zh-cn/develop/debug/crabnebula-devtools/)：日志流、Tauri command/event 的 payload 与耗时可视化。面板地址在启动时打印到终端（standalone 网页形态），应用内 `Ctrl+Shift+M` 可开 embedded 形态。release 构建（`pnpm tauri build`）不带该 feature，产物不含插件、也不打开 `tauri/tracing` 埋点。**用法见 `docs/devtools.md`**；**后续 Tauri 项目接入同一套工具直接抄 `docs/templates/tauri-devtools/`**（含坑位清单）。
