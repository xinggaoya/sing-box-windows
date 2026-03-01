# TAURI BACKEND KNOWLEDGE BASE

## OVERVIEW

Rust + Tauri 后端根目录，聚合命令注册、插件配置、平台打包配置与构建元信息。

## STRUCTURE

```
src-tauri/
├── src/            # 业务代码（app/entity/process/platform/utils）
├── Cargo.toml      # 依赖、profile、平台差异依赖
├── tauri.conf.json # 应用窗口/打包配置
└── icons/          # 多平台图标资源
```

## WHERE TO LOOK

| Task           | Location          | Notes                               |
| -------------- | ----------------- | ----------------------------------- |
| 命令注册总入口 | `src/lib.rs`      | `invoke_handler` 暴露边界           |
| 进程主入口     | `src/main.rs`     | 仅启动入口，逻辑薄                  |
| 后端分层实现   | `src/app/`        | core/network/system/storage/singbox |
| 平台差异依赖   | `Cargo.toml`      | cfg(windows/unix/macos)             |
| 打包行为       | `tauri.conf.json` | 窗口与 bundle 相关                  |

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

- 当前 `lib.rs` 存在 websocket plugin 重复注册，改动插件段时注意行为兼容。
- `Cargo.toml` release profile 偏激进（LTO/strip/panic=abort），排障时注意与 dev 行为差异。
