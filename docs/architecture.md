# 架构与职责说明

## 目标
- 前端专注展示与轻量交互，业务校验和状态真相尽量下沉到后端。
- 通信与事件流有明确契约，减少魔法字符串与隐式依赖。
- 可维护性：模块边界清晰，初始化与资源回收有统一入口。

## 总览
- **技术栈**：Tauri 2 + Rust 后端 / Vue 3 + Pinia + Naive UI 前端。
- **运行流程**：前端启动时加载持久化配置 → 通过 `invokeWithAppContext` 调用后端命令 → 后端拉起 sing-box、订阅 WebSocket → 经 Tauri 事件推送流量/日志/状态到前端。
- **持久化**：后端 `EnhancedStorageService` 负责存储 App/Theme/Locale/Window/Update/Subscription 等配置；前端通过各 Pinia Store 的 `initializeStore` + `saveToBackend` 读写。
  - 后台任务：Tauri 侧负责自动更新检查、内核健康巡检、订阅自动刷新（按订阅配置的 `auto_update_interval_minutes`），结果通过事件推送。

## 前端分层
- **入口**：`src/main.ts` 负责挂载、全局错误处理、beforeunload 清理。
- **UI Shell**：`src/App.vue` 承载全局 Provider、主路由视图、更新通知。后续应最小化业务逻辑，仅委托给 boot/服务层。
- **路由/布局**：`src/router` + `components/layout/MainLayout.vue`。
- **状态层**：`src/stores/*`（App/Kernel/Traffic/Connection/Update/Tray/Sub 等），内含持久化逻辑与事件监听。
- **服务层**：`src/services/*` 对应后端命令和事件封装（kernel/proxy/subscription/system 等），`invoke-client` 自动注入端口等上下文。
- **事件/消息**：`event-service` 封装 Tauri 事件监听，`mitt` 用于前端内部消息。

## 后端分层（Tauri）
- **命令注册**：`src-tauri/src/lib.rs`；启动时初始化日志、单实例、窗口状态、增强存储，并触发内核自动管理。
- **核心模块**：`app/core`（kernel_service、kernel_auto_manage、proxy_service、event_relay）、`app/network/subscription_service`、`app/system`（system/update/config）、`app/storage`（database/enhanced_storage_service）。
- **事件推送**：`event_relay` 直连 sing-box API 的 WS，将流量/内存/日志/连接等数据通过 Tauri 事件抛给前端。
- **持久化**：`EnhancedStorageService` 统一读写配置，供命令与前端持久化使用。

## 数据与通信流
- **命令调用（主要）**
  - 内核：`kernel_start_enhanced` / `kernel_stop_enhanced` / `kernel_auto_manage` / `kernel_check_health` / `check_kernel_version` / `get_latest_kernel_version_cmd` / `get_kernel_releases_cmd`。
  - 代理配置：`apply_proxy_settings` / `toggle_ip_version` / `update_singbox_ports`。
  - 订阅：`download_subscription` / `add_manual_subscription` / `set_active_config_path` / `delete_subscription_config` / `rollback_subscription_config` / `toggle_proxy_mode` / `get_current_config`。
  - 系统/更新：`check_update` / `download_and_install_update` / `install_update` / `restart_as_admin` 等。
- **事件推送（主要）**
  - 数据流：`traffic-data`、`memory-data`、`log-data`、`connections-data`。
  - 内核：`kernel-status-changed`、`kernel-ready`、`kernel-error`、`kernel-starting`、`kernel-started`、`kernel-stopped`、`kernel-download-progress`、`kernel-health`（后台健康巡检）。
  - 更新：`update-progress`、`update-available`（后台定时检查）。
  - 订阅：`subscription-updated`（后台自动刷新完毕后推送）。
- **存储契约**
  - 配置项（端口、代理模式、TUN 参数、自动启动、主题、语言、窗口、订阅等）由后端存储提供，前端 Store 初始化时读取，更新后通过命令保存。
  - 默认值建议收敛到后端常量，前端只展示/编辑。

## 角色与边界建议
- 前端
  - 负责视图渲染、轻量输入校验、状态展示。
  - 初始化/定时任务等调度逻辑收口到独立 boot/service 模块，方便替换为后端调度。
  - 事件监听/清理集中管理，避免分散在各视图。
- 后端
  - 负责业务校验与降级策略（端口可用性、代理模式合法性、订阅内容校验、内核健康重试/超时）。
  - 负责定时/后台任务（自动更新检查、内核健康巡检、订阅定期刷新），通过事件推送结果。
  - 输出明确的命令/事件契约，避免隐式字段。

## 近期重构路线（建议）
1. **契约收敛**：前端集中定义事件名与负载类型，服务层统一封装命令参数/返回值，减少魔法字符串。
2. **启动/调度抽离**：将 `App.vue` 中初始化、定时任务拆到 `boot/` 或服务模块，UI 仅挂载与提供容器。
3. **后端迁移逻辑**：把自动更新轮询、内核健康检查/重启、订阅校验/应用搬到 Tauri，前端改为监听事件和展示状态。
4. **默认配置下沉**：端口、TUN、绕过列表等默认值收口到后端常量并通过命令读写，前端仅透传。
5. **测试与校验**：对关键命令/事件契约补充类型与最小化单测（服务层），确保 `pnpm type-check`/`pnpm lint` 通过。
