# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 开发指导原则

- 请你全程使用中文进行交流
- 代码过程中在必要的地方要写一些注释
- 如果涉及到编写页面，请你以现代化、简约、美观的风格设计（当前为 Indigo 设计系统，tokens 见 `src/assets/tokens.css`）
- 你可以使用任何工具、以及MCP在开发时最好使用MCP熟悉最新文档
- **提交前文档门禁**：功能变更、行为调整、接口/命令增删、结构或流程变化时，必须先按需更新对应文档再提交（对照根 `AGENTS.md` 的 DOC SYNC GATE 表；用户可见变化补 `docs/CHANGELOG.md` 的 `[未发布]` 段）

## Project Overview

sing-box-windows is a modern cross-platform proxy client for Windows (x64/ARM64), Linux, and macOS built with Tauri 2.0 + Vue 3, providing complete proxy management, routing rules, subscription management, and system tray functionality. Kernel interaction uses the **sing-box 1.14+ official gRPC API** (`type: "api"`, gRPC-Web frames); the legacy Clash API (`experimental.clash_api`) has been fully removed.

### Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia + Naive UI + Vue I18n
- **Backend**: Rust + Tauri 2.0 + tokio + sqlx(SQLite) + reqwest
- **Architecture**: MVVM + modular design
- **Persistence**: SQLite (`EnhancedStorageService`) + 前端持久化 composable（`src/stores/app/composables/persistence.ts`）
- **Cross-language types**: ts-rs 由 Rust 结构体生成，落在 `src/types/generated/` 与 `src-tauri/src/types/generated/`，两侧都不手改
- **Build**: Vite (frontend) + `scripts/tauri-wrapper.mjs`（所有 tauri 命令的包装器）

## Common Commands

### Development
```bash
# Install dependencies
pnpm install
pnpm kernel:fetch          # 首次构建前必须；--all 拉全平台（stable/oldstable/beta/testing 轨道）

# Start development server
pnpm tauri dev
```

### Build and Packaging
```bash
# Build production version
pnpm tauri build

# Windows ARM64
pnpm tauri build:windows:arm64

# Linux RPM
pnpm tauri build:linux:rpm

# 所有 tauri 子命令均经 scripts/tauri-wrapper.mjs 包装
```

### Code Quality
```bash
# ESLint + OXLint
pnpm lint

# Code formatting
pnpm format

# TypeScript type checking
pnpm type-check

# Kernel target matrix tests (node --test)
pnpm test:kernel-targets

# Rust code formatting
cd src-tauri && cargo fmt

# Rust clippy linter
cd src-tauri && cargo clippy

# Rust unit tests
cd src-tauri && cargo test

# Run all quality gates（与 CI .github/workflows/ci.yml 一致）
pnpm lint && pnpm type-check && pnpm test:kernel-targets && cd src-tauri && cargo clippy && cargo test
```

## Core Architecture

### Architectural Patterns

1. **Event-Driven State Management**: 运行态数据（流量/日志/连接/代理组）由后端 gRPC server-streaming 订阅（`kernel_service/event.rs`，4 路：Status/Log/Connections/Groups，含指数退避重连）转成 **Tauri event** 推送；前端经 `event-service.ts` 消费，事件名常量集中在 `src/constants/events.ts`。前端组件通过 mitt 总线做局部队内通信。注意 `tauri-plugin-websocket` 仍注册但属遗留，运行态链路不经过它。

2. **Hybrid Storage Architecture**:
   - **SQLite**: 结构化数据经 `EnhancedStorageService`（OnceCell 单例，sqlx，自动迁移）
   - **前端持久化**: stores 经 `stores/app/composables/persistence.ts` 模式保存
   - 后端保存有 300ms 防抖，前端可用 `waitForSaveCompletion()` 等待落盘

3. **Service Layer Pattern**: 前端所有后端调用经 `services/` 层；页面/组件不直接 `invoke`。

### Frontend Architecture

```
src/
├── boot/            # useAppBootstrap：业务引导（store 初始化、事件桥）
├── stores/          # Pinia 状态（app/kernel/subscription/tray 四域）
├── components/
│   ├── common/      # EmptyState、PageHeader、StatCard、SectionCard 等通用件
│   ├── layout/      # MainLayout、AppHeader、AppSidebar、TrafficChart
│   └── system/      # SudoPasswordModal
├── views/           # 页面（Home/Proxy/Rules/Connections/Log/Sub/Setting/Blank）
│   └── setting/     # 设置 5 个 Tab + 组合式函数
├── composables/     # useCleanup、useKernelStatus、usePageTheme、useVirtualization
├── services/        # invoke + event 封装（12 个 service）
├── constants/       # events.ts（前后端共享事件名）
├── types/generated/ # ts-rs 生成类型
├── locales/         # zh-CN(默认)/en-US/ja-JP/ru-RU
└── assets/          # tokens.css（Indigo 设计系统三层 token）、base/main/page-layout.css
```

### Backend Architecture

```
src-tauri/src/
├── app/             # 业务分层主域
│   ├── core/        # kernel_service（启停/版本管理/事件中继）、proxy_service（gRPC 查询+自定义规则）
│   ├── network/     # subscription_service（下载/解析/切换/自动更新）
│   ├── singbox/     # config_generator（配置生成与 1.14 DNS/自定义规则注入）
│   ├── singbox_api/ # gRPC-Web 客户端（client/proto/types）
│   ├── storage/     # EnhancedStorageService + custom_rule 模型
│   ├── system/      # update_service（自研更新）、background_tasks、startup_* 服务
│   ├── tray/        # 托盘初始化、状态刷新、动作分发
│   └── constants/   # 领域常量
├── entity/          # config_model、github_model
├── platform/        # windows/linux/macos 平台实现
├── process/         # ProcessManager 内核进程管理
├── types/generated/ # ts-rs 生成 TS 类型
├── utils/           # http/file/log/process 等工具
├── error.rs         # 统一错误
├── main.rs          # 程序入口（薄）
└── lib.rs           # run()：插件注册、setup（同步+异步初始化链）、118 个命令注册
```

## Key Features

### 1. Store Management System
- **Standard Pinia**: 官方 Pinia；`app/`（App/Locale/Sudo/Theme/Update/Window）、`kernel/`（Kernel/KernelRuntime/Connection/Log/Proxy/Traffic）、`subscription/`、`tray/`
- **Component-based Lifecycle**: 生命周期事件由使用 store 的组件管理，并配合 `useCleanup` composable
- **Initialization Phases**: `startInitialization()` → load → `finishInitialization()` 模式
- **Data Restore Pattern**: `waitForDataRestore()` 防止启动竞态；`invoke-client` 调用前也会等待

### 2. Frontend-Backend Communication
- **Tauri Commands**: 前端调用统一走 `src/services/invoke-client.ts`（自动注入 apiPort/proxyPort、等待数据恢复）
- **Unified Error Handling**: 后端命令统一返回 `Result<T, String>`
- **Type Safety**: ts-rs 生成的 `types/generated/` 保证前后端类型一致
- **Event-Driven Updates**: 后端 gRPC 订阅 → `kernel_service/event.rs` → Tauri event（traffic-data/log-data/connections-data/groups-data/memory-data 等）

### 3. sing-box 1.14 gRPC API（重要）
- 客户端在 `src-tauri/src/app/singbox_api/`：`client.rs`（gRPC-Web over HTTP/1.1，reqwest 需 `.no_proxy()`）、`proto.rs`（手写极简 protobuf wire-format 解码，无 prost 依赖）、`types.rs`（手写 serde 结构）
- 前端对应命令：`get_groups`、`get_rules`、`get_services`、`select_outbound`、`url_test`、`network_quality_test`、`close_connection`、`get_clash_mode_status` 等
- 迁移记录见 `docs/sing-box-api-migration.md`；后续计划见 `docs/sing-box-1.14-roadmap.md`

### 4. Performance Optimization
- Virtual Scrolling（`VirtualList.vue` / `useVirtualization`）
- Lazy Loading（`LazyComponent.vue`）
- Auto Imports（unplugin-auto-import / unplugin-vue-components）
- Vite chunk 分割（Naive UI 单独合并 chunk，防启动白屏）

## Development Workflow

### Adding New Features
1. Define TypeScript types in `types/`（跨语言类型走 ts-rs，勿手改生成文件）
2. Create state management in `stores/` (if needed)
3. Create API service layer in `services/`
4. Create UI components in `components/`
5. Create page views in `views/`
6. Implement backend commands in `src-tauri/src/app/`
7. Update routing configuration (if needed)
8. Register new Tauri commands in `lib.rs`（当前 118 个命令）
9. **按 DOC SYNC GATE 更新对应层级文档与 CHANGELOG**

### Store Development Patterns
- **Initialization**: Always implement proper initialization with data restore waiting
- **Cleanup**: Implement `cleanupStore()` methods for resource cleanup
- **Event Handling**: Use event-driven updates; 事件名写入 `src/constants/events.ts`
- **Persistence**: 优先复用 `stores/app/composables/persistence.ts` 模式

### Backend Development Patterns
- **Service Organization**: Group services by domain (core/network/singbox/storage/system/tray)
- **Error Handling**: Return Result<T, String> for all commands；错误信息带上下文
- **Async Operations**: Use tokio；启动期任务走 `lib.rs` 的单任务顺序初始化链，勿乱序
- **Platform Logic**: 平台差异放 `platform/` 或 cfg 属性，不散落通用模块

## Key Files and Their Purposes

### Configuration Files
- `src-tauri/tauri.conf.json`: 窗口（1024x700、decorations:false）、bundle（msi/nsis/deb/rpm/appimage/dmg/app）、CSP 允许 127.0.0.1 本地内核
- `src-tauri/Cargo.toml`: 依赖、激进 release profile（LTO/strip/panic=abort）
- `package.json`: npm scripts（tauri 命令经 wrapper）
- `vite.config.ts`: auto-imports 与 chunk 优化

### Core Files
- `src/boot/useAppBootstrap.ts`: 业务引导（store 初始化 + 事件桥）
- `src/services/invoke-client.ts`: 统一 invoke（端口注入 + waitForDataRestore）
- `src/constants/events.ts`: 前后端共享事件名（约 27 个）
- `src-tauri/src/lib.rs`: 命令注册与启动初始化链
- `src-tauri/src/app/core/kernel_service/event.rs`: gRPC 订阅 → Tauri event 中继（4 路，指数退避）
- `src-tauri/src/app/singbox/config_generator.rs`: 配置生成（含 1.14 DNS mDNS/evaluate、自定义规则注入）
- `src-tauri/src/app/storage/enhanced_storage_service.rs`: SQLite 存储服务

## Storage System

- **SQLite**（`EnhancedStorageService`）: 结构化数据（订阅、通用配置如 custom_rules 存 `generic_config` 表、内核版本缓存等）
- **前端持久化**: stores persistence composable
- **Storage Locations**:
  - Windows: `%APPDATA%\sing-box-windows\`
  - Linux: `~/.local/share/sing-box-windows/`
  - macOS: `~/Library/Application Support/sing-box-windows/`

### Enhanced Storage Service
- Single initialization with `OnceCell` pattern
- Type-safe operations through SQLx
- Automatic schema migrations
- Backend-frontend synchronization

## Development Guidelines

### Memory Management
- 长驻应用注意内存泄漏：事件监听/定时器在 `onUnmounted` 或 `useCleanup` 中清理
- 后端事件中继有清理纪元（epoch）机制防订阅泄漏，改动 `event.rs` 时保持该模式
- 监控 WebSocket/gRPC 订阅连接的正确关闭

### Error Handling
- All async operations require proper error handling
- Backend commands should return Result<T, String> for consistent error handling
- Use TypeScript's strict type checking to avoid runtime errors（禁止 as any / @ts-ignore）

### Performance
- Use virtual scrolling or pagination for large data operations
- Implement lazy loading for non-critical components
- Use the composable patterns for reusable logic

### Cross-Platform Compatibility
- Maintain compatibility with Windows (x64/ARM64), Linux, and macOS
- Use platform-specific dependencies only when necessary
- Linux 发布目标 glibc <= 2.38（CI 固定 ubuntu-22.04 构建并检查）

## Debugging

### Frontend Debugging
- Vue DevTools automatically integrated in development environment
- Network/事件流可用浏览器开发者工具与 Vue DevTools 观察

### Backend Debugging
- Use `tracing`/`log` 输出；内核日志见 sing-box.log（按大小轮转）
- Log levels controlled via RUST_LOG environment variable:
```bash
RUST_LOG=debug pnpm tauri dev
RUST_LOG=tauri=info,sing_box_windows=debug pnpm tauri dev
```

## 常见问题解决

### 开发环境问题
1. **Windows 编译错误**: 确保安装了 Visual Studio Build Tools 或 Visual Studio Community with C++ support
2. **Linux 依赖问题**: 安装必要的系统依赖 `sudo apt-get install libwebkit2gtk-4.1-0 libssl3 libgtk-3-0`
3. **macOS 编译**: 需要安装 Xcode Command Line Tools `xcode-select --install`
4. **首次构建失败**: 未执行 `pnpm kernel:fetch`，`src-tauri/resources/kernel/` 下无内核二进制

### 构建优化
- 生产构建会自动排除 Vue DevTools
- Naive UI 依赖必须保持单一 chunk（拆开曾导致启动白屏）
- release profile 极激进（panic=abort），排障时注意 release 与 dev 行为差异

## Platform-Specific Notes

### Windows
- Requires Visual Studio Build Tools for compilation
- MSI and NSIS installers supported；另有 **ARM64** 构建（`pnpm tauri build:windows:arm64`）
- 系统代理经 Windows API；应用内更新仅 Windows 支持

### Linux
- Requires libwebkit2gtk-4.1-0, libssl3, libgtk-3-0 dependencies
- DEB/RPM/AppImage packages supported
- System proxy integration via environment variables

### macOS
- Requires Xcode Command Line Tools for compilation
- DMG and App bundles supported
- 支持 `--hide` 延迟启动（后台启动不显示窗口）

## Testing and Quality Assurance

### Code Quality Tools
- **ESLint + OXLint**: JavaScript/TypeScript linting
- **Prettier**: Code formatting（2 空格、singleQuote、semi: false、printWidth 100）
- **rustfmt + clippy**: Rust formatting and linting
- **vue-tsc**: TypeScript static type checking
- **node --test**: scripts 目标矩阵测试（`pnpm test:kernel-targets`）
- **cargo test**: Rust 内联单测

### Before Committing
1. Run `pnpm lint` to fix code style issues
2. Run `pnpm type-check` to verify TypeScript types
3. Run `cd src-tauri && cargo clippy` for Rust linting（CI 以 `-D warnings` 门禁）
4. Test functionality on target platforms
5. **文档门禁**：`git diff --stat` 对照根 AGENTS.md 的 DOC SYNC GATE 表，功能/接口/结构/流程变化必须先更新对应文档与 `docs/CHANGELOG.md`，再提交

## 特别说明

### 单实例应用
单实例由 Rust 插件 `tauri_plugin_single_instance` 实现（tauri.conf.json 无相关配置段），重复启动时唤起已有窗口。

### 无边框窗口
`decorations: false` 自绘窗口控制；窗口隐藏时前端切到 `/blank` 降低托盘驻留开销。

### 运行态数据通道
后端 gRPC server-streaming（Status/Log/Connections/Groups）→ `kernel_service/event.rs` 转 Tauri event → 前端 `event-service.ts`。前端无独立 websocket-service；`tauri-plugin-websocket` 仅遗留注册。

### 应用更新
不走 tauri-plugin-updater；`system/update_service.rs` 自研（GitHub Releases API 查询 + 下载安装），应用内更新仅 Windows。

### 内核版本管理
版本默认跟随最新 stable（`kernel_service/versioning.rs`，GitHub API + 3 镜像 + DB 缓存）；设置页版本下拉可手动指定版本，下载指定版本前强制弹窗确认兼容风险（应用基于 1.14+ gRPC API，旧内核不兼容），确认后正常下载；下载轨道支持 stable/oldstable/beta/testing（`scripts/fetch-kernel.mjs`）。

### 自定义规则 CRUD
后端就绪（`proxy_service.rs` 5 个命令 + `storage/custom_rule.rs` 模型 + `config_generator.rs` 注入），前端 UI 暂未接线；类型经 ts-rs 导出到两侧 `types/generated/CustomRule*`。

### 国际化支持
- 简体中文 (zh-CN, 默认)
- English (en-US)
- 日本語 (ja-JP)
- Русский (ru-RU)
- 详见 `docs/i18n.md`
