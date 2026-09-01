# FRONTEND KNOWLEDGE BASE

## OVERVIEW

Vue 3 + TypeScript 前端主域，负责页面、状态、服务层与后端事件消费。

## STRUCTURE

```
src/
├── boot/         # 启动编排（useAppBootstrap）
├── router/       # 路由与 /blank 托盘态
├── services/     # invoke + event 封装
├── stores/       # Pinia 领域状态
├── views/        # 页面视图
└── types/        # TS 类型（含 generated）
```

## WHERE TO LOOK

| Task               | Location                     | Notes                          |
| ------------------ | ---------------------------- | ------------------------------ |
| 应用启动初始化顺序 | `boot/useAppBootstrap.ts`    | store 初始化、事件桥、清理函数 |
| 后端命令调用封装   | `services/invoke-client.ts`  | 自动上下文注入（端口等）       |
| 后端事件监听       | `services/event-service.ts`  | traffic/log/connection 入口    |
| 内核业务前端编排   | `services/kernel-service.ts` | 命令调用最集中                 |
| 路由行为（含托盘） | `router/index.ts`            | `/blank` 逻辑关键              |
| 节点选择 / 测速   | `services/proxy-service.ts` + `stores/kernel/ProxyStore.ts` | gRPC API（sing-box 1.14+） |

## CONVENTIONS (FRONTEND ONLY)

- 页面文件统一 `*View.vue`。
- 组合式函数统一 `useXxx.ts`。
- 通过 `@/*` 别名引入，避免深层相对路径。
- 事件名优先使用 `src/constants/events.ts` 常量，不写裸字符串。
- 服务层负责调用后端；页面/组件尽量不直接 `invoke`。

## ANTI-PATTERNS

- ❌ 在页面组件里直接拼命令名并调用 `invoke`。
- ❌ 绕过 `invoke-client` 造成上下文参数不一致。
- ❌ 新增事件不写到 `constants/events.ts`。
- ❌ 用 `as any` 临时压类型通过检查。

## TESTING / VERIFY

- 当前前端自动化测试很少；改动后至少执行：

```bash
pnpm lint
pnpm type-check
```

## NOTES

- `main.ts` 仅做壳层初始化；真实业务启动在 `boot/useAppBootstrap.ts`。
- 大文件热点集中在 `views/` 与 `services/kernel-service.ts`，改动前先查现有模式。
- **API 协议已切换**：内核交互从 Clash API（HTTP RESTful）迁移到 sing-box 1.14+ 官方 gRPC API（`type: "api"`）。前端 services 调用新 Tauri 命令（`get_groups`、`select_outbound`、`url_test`、`close_connection`、`get_clash_mode_status` 等）。
- **规则页与代理提供者页**已隐藏：官方 gRPC API 未暴露规则管理接口，对应 UI 暂时显示占位说明。详见 `docs/sing-box-api-migration.md`。
