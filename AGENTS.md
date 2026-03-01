# PROJECT KNOWLEDGE BASE

**Generated:** 2026-03-01T20:44:47+08:00  
**Commit:** 5a3d46c  
**Branch:** master

## OVERVIEW

跨平台 Sing-Box 图形客户端（Windows/Linux/macOS）。
前端为 Vue 3 + TypeScript，后端为 Tauri 2 + Rust；前后端通过 Tauri command + event 双通道通信。

## STRUCTURE

```
./
├── src/                 # 前端应用（路由/页面/store/service）
├── src-tauri/           # Rust 后端 + Tauri 打包
├── scripts/             # 内核下载与 tauri wrapper
├── docs/                # 开发文档（包含详细命令与约束）
└── .github/workflows/   # Release CI（多平台矩阵）
```

## WHERE TO LOOK

| Task               | Location                                          | Notes                           |
| ------------------ | ------------------------------------------------- | ------------------------------- |
| 前端启动链路       | `src/main.ts`, `src/boot/useAppBootstrap.ts`      | 初始化顺序与事件桥都在这里      |
| 前端调用后端       | `src/services/invoke-client.ts`                   | 命令调用统一入口（上下文注入）  |
| 路由与托盘空白页   | `src/router/index.ts`, `src/views/BlankView.vue`  | `/blank` 是非标准但关键路径     |
| 后端入口与命令注册 | `src-tauri/src/lib.rs`                            | `setup + invoke_handler` 为核心 |
| 内核生命周期       | `src-tauri/src/app/core/kernel_service/`          | 高复杂度热点目录                |
| 订阅解析与模式切换 | `src-tauri/src/app/network/subscription_service/` | parser/mode/helpers 分层明显    |
| 存储实现           | `src-tauri/src/app/storage/`                      | SQLite + OnceCell 单例初始化    |
| 发布流水线         | `.github/workflows/release.yml`                   | glibc 兼容检查与多平台产物      |

## CODE MAP

> 本地 LSP 不可用（缺少 `oxlint` 与 `rust-analyzer`），以下为静态扫描结果。

| Symbol / Area              | Type               | Location                                                | Role                                |
| -------------------------- | ------------------ | ------------------------------------------------------- | ----------------------------------- |
| `useAppBootstrap`          | Frontend bootstrap | `src/boot/useAppBootstrap.ts`                           | 统一串联 store 初始化与后端事件桥接 |
| `run()`                    | Backend entry      | `src-tauri/src/lib.rs`                                  | 插件注册、异步初始化、命令暴露      |
| `kernel_start_enhanced` 等 | Tauri commands     | `src-tauri/src/app/core/kernel_service/*`               | 内核启停与健康检查                  |
| `download_subscription` 等 | Tauri commands     | `src-tauri/src/app/network/subscription_service.rs`     | 订阅下载、切换、回滚                |
| `EnhancedStorageService`   | Storage service    | `src-tauri/src/app/storage/enhanced_storage_service.rs` | 应用配置与结构化数据持久化          |

## CONVENTIONS

- 命名：组件 PascalCase，页面 `*View.vue`，组合式函数 `useXxx.ts`。
- 路径别名：`@/* -> src/*`。
- TS 禁止：`as any`、`@ts-ignore`、`@ts-nocheck`。
- Rust 约定：命令返回 `Result<T, String>`，模块/函数 snake_case。
- 格式化：2 空格、`singleQuote: true`、`semi: false`、`printWidth: 100`。

## ANTI-PATTERNS (THIS PROJECT)

- ❌ 提交 secrets、日志、临时文件、本地覆盖文件。
- ❌ 为“过类型检查”使用类型压制。
- ❌ 跳过 `pnpm lint` / `pnpm type-check` / `cargo clippy`。
- ❌ 首次构建前不执行 `pnpm kernel:fetch`。
- ❌ 在 commit 中保留调试输出与一次性脚本改动。

## UNIQUE STYLES

- 前端与后端都使用“事件驱动”：前端 `mitt` + 后端 Tauri event。
- 启动链是双阶段：`main.ts` 初始化 + `useAppBootstrap` 实际业务引导。
- 后端 `lib.rs` 在启动后异步做：存储初始化、残留进程清理、自动任务启动。
- Linux CI 固定 Ubuntu 22.04，并强制 glibc <= 2.38 兼容性检查。

## COMMANDS

```bash
# 开发
pnpm kernel:fetch
pnpm tauri dev

# 质量门禁
pnpm lint
pnpm type-check
cd src-tauri && cargo clippy

# 测试（当前主要为 Rust 单测）
cd src-tauri && cargo test

# 构建
pnpm tauri build
```

## NOTES

- 当前前端测试文件极少，回归验证以类型检查 + Rust 测试 + 实机联调为主。
- `src-tauri/src/lib.rs` 存在重复 websocket plugin 注册，改动此文件前先确认行为影响。
- 分层 AGENTS 已在 `src/`、`src/stores/`、`src-tauri/`、`src-tauri/src/app/` 细化。
