# BACKEND APP LAYER KNOWLEDGE BASE

## OVERVIEW

后端业务分层主域：`core/network/system/storage/singbox/constants`，是命令实际落地位置。

## STRUCTURE

```
src-tauri/src/app/
├── core/       # 内核启停、代理模式、事件中继
├── network/    # 订阅下载、解析、模式切换、自动更新
├── system/     # 更新、权限、系统状态、后台任务
├── storage/    # SQLite 服务与状态模型
├── singbox/    # 配置生成与注入
└── constants/  # 领域常量
```

## WHERE TO LOOK

| Task              | Location                                 | Notes                      |
| ----------------- | ---------------------------------------- | -------------------------- |
| 内核生命周期问题  | `core/kernel_service/`                   | 最复杂热点，启停/健康/版本 |
| 订阅解析异常      | `network/subscription_service/parser.rs` | 体量大、协议分支多         |
| 更新流程          | `system/update_service.rs`               | 平台产物与下载流程         |
| 存储读写          | `storage/enhanced_storage_service.rs`    | 与前端配置同步关键         |
| sing-box 配置生成 | `singbox/config_generator.rs`            | 规则与 outbounds 组装      |

## CONVENTIONS

- 模块与函数命名 snake_case，结构体/枚举 PascalCase。
- 复杂领域通过子模块拆分（如 kernel_service, subscription_service）。
- 错误信息优先携带上下文，便于前端展示与日志定位。
- 业务分层保持单向：command -> domain service -> utility。

## ANTI-PATTERNS

- ❌ 在 parser/config_generator 中混入无边界的跨域逻辑。
- ❌ command 函数中直接塞长流程，不提炼到子模块。
- ❌ 只改调用层不改常量/模型定义，造成前后语义漂移。

## TESTING

- 当前测试主要分布在 Rust 内联单测（`#[cfg(test)]`）中。
- 关键目录改动后至少执行：

```bash
cd src-tauri && cargo test
cd src-tauri && cargo clippy
```

## NOTES

- `network/subscription_service/` 与 `core/kernel_service/` 是高频联动区，改动其一需回归另一区域。
- `system/background_tasks` 在启动自动运行，新增任务需评估启动时序与资源占用。
