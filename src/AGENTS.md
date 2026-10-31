# FRONTEND KNOWLEDGE BASE

## OVERVIEW

Vue 3 + TypeScript 前端主域，负责页面、状态、服务层与后端事件消费。

## STRUCTURE

```
src/
├── assets/       # tokens.css（Indigo 设计系统三层 token）与全局样式
├── boot/         # 启动编排（useAppBootstrap）
├── components/   # common/ layout/ system/ + 根级通用件（VirtualList、UpdateModal 等）
├── composables/  # useCleanup、useKernelStatus、usePageTheme、useVirtualization
├── constants/    # 仅 events.ts（前后端共享事件名，约 27 个）
├── locales/      # zh-CN(默认)/en-US/ja-JP/ru-RU
├── router/       # 路由与 /blank 托盘态
├── services/     # invoke + event 封装（13 个 service）
├── stores/       # Pinia 领域状态
├── views/        # 页面视图（Home/Proxy/Rules/Connections/Log/Sub/Setting/Blank）
└── types/        # TS 类型（含 ts-rs 生成的 generated/）
```

## WHERE TO LOOK

| Task                        | Location                                                                                                              | Notes                                                                        |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| 应用启动初始化顺序          | `boot/useAppBootstrap.ts`                                                                                             | store 初始化、事件桥、清理函数                                               |
| 后端命令调用封装            | `services/invoke-client.ts`                                                                                           | 自动上下文注入（端口等）                                                     |
| 后端事件监听                | `services/event-service.ts`                                                                                           | traffic/log/connection 入口                                                  |
| 内核业务前端编排            | `services/kernel-service.ts`                                                                                          | 命令调用最集中                                                               |
| 路由行为（含托盘）          | `router/index.ts`                                                                                                     | `/blank` 逻辑关键                                                            |
| 节点选择 / 测速             | `services/proxy-service.ts` + `stores/kernel/ProxyStore.ts`                                                           | gRPC API（sing-box 1.14+）                                                   |
| 规则页（只读 + 自定义规则） | `views/RulesView.vue`                                                                                                 | 运行时规则只读（gRPC `get_rules`）+ 自定义规则 CRUD 面板                     |
| 自定义规则前端接线          | `services/custom-rule-service.ts` + `stores/kernel/CustomRuleStore.ts`                                                | 走文件+重启路径，与运行时规则语义不同，UI 需区分                             |
| 出站总览 / 网络诊断         | `views/OutboundsView.vue` + `composables/useOutboundList.ts` + `views/setting/components/SettingsDiagnosticsCard.vue` | 「出站」独立页（路由 `/outbounds`）与维护页诊断视图；代理页不得挂出站总览 UI |
| 设计系统 / 主题             | `assets/tokens.css` + `stores/app/ThemeStore.ts`                                                                      | Indigo 三层 token，主题覆盖 `--primary-*`                                    |
| 资源清理                    | `composables/useCleanup.ts`                                                                                           | 组件卸载统一清理监听/定时器                                                  |

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
- **API 协议已切换**：内核交互从 Clash API（HTTP RESTful）迁移到 sing-box 1.14+ 官方 gRPC API（`type: "api"`）。前端 services 调用新 Tauri 命令（`get_groups`、`get_rules`、`get_services`、`select_outbound`、`url_test`、`network_quality_test`、`close_connection`、`get_clash_mode_status` 等）。
- **规则页已恢复（只读）+ 自定义规则 CRUD**：`RulesView.vue` 上半部调 gRPC `get_rules` 展示运行时规则（只读），下半部是 `components/rules/CustomRulePanel.vue` 的自定义规则 CRUD（走文件+重启，`services/custom-rule-service.ts` + `stores/kernel/CustomRuleStore.ts`）。代理提供者管理仍未实现（gRPC API 未暴露），详见 `docs/sing-box-api-migration.md`。
- **跨语言类型已统一**：`src/types/generated/` 是 ts-rs 生成的**唯一事实来源**（git 跟踪）；前端不得手写后端结构体对应的 interface，漂移由 `src/types/generated/grpc-types.test.ts` 守护。
- **运行态数据通道**：后端 gRPC 订阅 → Tauri event → `event-service.ts`；前端无独立 websocket-service，事件名必须用 `constants/events.ts` 常量。
- **代理页是「定高 + 内部滚动」布局**：`.page-shell`（flex 列）+ `.split-view`（`flex: 1`，节点网格自己内部滚动）构成。**同层不要再放有内容高度的卡片/面板**——flex 分配高度时会把 `.split-view` 压扁（`min-height: 0`），整页主体不可见（2026-10-20 的「出站总览」卡片就是这么把代理页挤废的）。辅助视图一律**独立成页面**（出站总览 → `/outbounds`），并由 `views/proxy-layout.test.ts` 守护这条约定。
- **UI 设计系统**：Indigo（`assets/tokens.css`），新增组件优先复用 `components/common/` 下的 SectionCard/StatCard/PageHeader 等。
