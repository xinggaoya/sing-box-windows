# BACKEND APP LAYER KNOWLEDGE BASE

## OVERVIEW

后端业务分层主域：`core/network/system/storage/singbox/constants`，是命令实际落地位置。

## STRUCTURE

```
src-tauri/src/app/
├── core/          # 内核启停、代理模式、gRPC 查询、事件中继
├── network/       # 订阅下载、解析、模式切换、自动更新
├── template_marketplace/  # 在线模板：本地 KV 存储、市场客户端（独立私有服务端）、15 个命令
├── system/        # 更新（自研）、权限、系统状态、后台任务、启动恢复
├── storage/       # SQLite 服务与状态模型（含 custom_rule）
├── singbox/       # 配置生成与注入（含 1.14 DNS、自定义规则）
├── singbox_api/   # sing-box 1.14+ 官方 gRPC API 客户端
├── tray/          # 托盘初始化、状态刷新、动作分发
└── constants/     # 领域常量
```

## WHERE TO LOOK

| Task              | Location                                 | Notes                      |
| ----------------- | ---------------------------------------- | -------------------------- |
| 内核生命周期问题  | `core/kernel_service/`                   | 最复杂热点，启停/健康/版本锁定 |
| 订阅解析异常      | `network/subscription_service/parser.rs` | 体量大、协议分支多         |
| 更新流程          | `system/update_service.rs`               | 自研 GitHub Release 下载，仅 Windows 应用内更新 |
| 存储读写          | `storage/enhanced_storage_service.rs`    | 与前端配置同步关键         |
| sing-box 配置生成 | `singbox/config_generator.rs`            | 规则/outbounds/DNS 组装，1.14 mDNS/evaluate 注入 |
| gRPC API 交互     | `singbox_api/` + `core/proxy_service.rs` | 替代 experimental.clash_api |
| 自定义规则 CRUD   | `core/proxy_service.rs` + `storage/custom_rule.rs` | 5 个命令，存 `generic_config` 表 key=`custom_rules` |
| 模板/市场         | `template_marketplace/commands.rs` + `local_store.rs` | 15 个命令，存 `generic_config` 表 key=`config_templates`/`template_market_settings` |
| 事件中继          | `core/kernel_service/event.rs`           | 4 个 gRPC server-streaming（Status/Log/Connections/Groups），指数退避+清理纪元 |
| 托盘              | `tray/`                                  | 启动偏好、关窗后状态刷新、动作分发 |

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
- `system/background_tasks` 在启动自动运行，新增任务需评估启动时序与资源占用；`lib.rs` setup 后是单任务顺序初始化链（存储 → 启动恢复 → 残留进程清理 → 订阅升级刷新 → 内核自动管理 → 托盘刷新 → 后台任务 → 订阅自动更新），勿乱序。
- `singbox_api/` 客户端实现 gRPC-Web over HTTP/1.1 协议（参照 sing-box-dashboard `src/api/websocket.ts` 的帧格式）；无 prost 依赖，所有 protobuf 解码手写在 `singbox_api/proto.rs`，reqwest client 需 `.no_proxy()`。
- metacubexd 预下载（`core/kernel_service/embedded.rs::ensure_external_ui`）已删除；启动时不再下载 metacubexd UI。
- DNS 1.14：mDNS server 与 fakeip `evaluate` action 在 `singbox/config_generator.rs` 生成；`match_response` 目前仅存在于注释中，属未来扩展。
- 自定义规则模型（`CustomRule`/`CustomRuleAction`/`CustomRuleMatchType`）经 ts-rs 导出到两侧 `types/generated/`，改字段后需重新生成并同步前端类型。
- 新增/修改本层命令后，按根 AGENTS.md 的 DOC SYNC GATE 同步文档与 `docs/CHANGELOG.md`。
