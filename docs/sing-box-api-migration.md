# Sing-Box 官方 gRPC API 替换 Clash API —— 迁移规划

**状态**：✅ **完成**（2026-09，迁移至 master，4 阶段全部勾选，四门禁全绿，11 个 gRPC 接口实机联调通过）

**目标**：将内核交互层从 `experimental.clash_api` (HTTP/JSON) 迁移到 sing-box 1.14.0+ 官方 `type: "api"` (gRPC-Web over **HTTP/1.1**，**不是 WebSocket**——sing-box 1.14.0 实际只支持 HTTP/1.1 传输)。

**触发原因**：

- 官方 API 是结构化的 protobuf 接口，类型安全；Clash API 是字面量 JSON，字段映射易错。
- 官方 API 提供 `URLTest`（整组测速，结果回写到 `urlTestDelay`）、`SetGroupExpand`（组展开状态内核侧持久化）、`GetStartedAt`、`GetDeprecatedWarnings` 等 Clash API 没有的能力。
- 4 个独立 WebSocket 中继（traffic/memory/logs/connections）合并为 3 个 gRPC server-streaming（Status/Log/Connections），连接更少、重连更原子化。
- 协议演进方向：sing-box 1.16+ 计划移除 `experimental.clash_api` 中"隐式 HTTP 客户端"等旧机制。

## 实机联调结果（sing-box 1.14.0 实测）

通过 `src-tauri/examples/grpc_smoke.rs` 验证所有 11 个接口：

```
✅ GetVersion: version=1.14.0, apiVersion=4
✅ GetGroups: 2 groups (proxy/auto selector+urltest)
✅ SelectOutbound: group=proxy → outbound=direct
✅ URLTest: triggered on group=proxy
✅ GetClashModeStatus: currentMode=, modeList=[]
✅ SetClashMode: → global
✅ SetGroupExpand: group=proxy → is_expand=true
✅ GetStartedAt: 1788171769950
✅ SubscribeLog: level=info "outbound/socks[jp-1]: outbound connection to cp.cloudflare.c..."
✅ SubscribeStatus / SubscribeConnections：server-streaming 正常
```

> sing-box 1.14.0 gRPC API 实际只支持 HTTP/1.1 gRPC-Web（POST + chunked），**不支持 WebSocket**。
> 关键修复：reqwest 客户端必须 `.no_proxy()` 禁用 Windows 系统代理（残留 127.0.0.1:12080）。

**本工程为一次性全面替换**：

- 关闭 `experimental.clash_api`，所有依赖该接口的 Rust 命令、Tauri command、Vue 组件同步移除。
- 移除 metacubexd 预下载与"打开内置 web dashboard"入口（metacubexd 是 Clash 客户端，依赖 Clash API）。
- 规则页与代理提供者页在官方 API 未暴露对应接口前隐藏（占位 UI），路由保留 meta 但 disabled。
- 自定义规则 CRUD（issue #62）走"写文件 + 重启内核"路径，不依赖 API，与现状一致。

## 接口映射（Clash API → gRPC API）

| 当前能力（Clash API） | 官方 API 方法 | 替换状态 |
|---|---|---|
| `GET /proxies` | `SubscribeGroups` (stream) | 替换 |
| `PUT /proxies/{group}` | `SelectOutbound(group, outbound)` | 替换 |
| `GET /proxies/{name}/delay` | `URLTest(outboundTag)` | 替换（整组） |
| `GET /rules` | ❌ 无 | **隐藏 UI** |
| `PATCH /rules/disable` | ❌ 无 | **隐藏 UI** |
| `GET /providers/proxies` | ❌ 无 | **隐藏 UI** |
| `PUT /providers/proxies/{n}` | ❌ 无 | **隐藏 UI** |
| `GET /providers/rules` | ❌ 无 | **隐藏 UI** |
| `PUT /providers/rules/{n}` | ❌ 无 | **隐藏 UI** |
| `WS /traffic` + `WS /memory` | `SubscribeStatus({interval})` (合并流) | 替换 |
| `WS /logs` | `SubscribeLog` | 替换 |
| `WS /connections` | `SubscribeConnections` | 替换 |
| `DELETE /connections/{id}` | `CloseConnection(id)` | 替换 |
| `DELETE /connections` | `CloseAllConnections` | 替换 |
| ❌（无） | `GetVersion` | **新增**（启动时版本协商） |
| ❌（无） | `GetClashModeStatus` + `SubscribeClashMode` + `SetClashMode` | **新增**（运行时切换 Rule/Direct/Global） |
| ❌（无） | `SetGroupExpand(group, isExpand)` | **新增**（组展开状态内核侧持久化） |
| ❌（无） | `GetStartedAt` | **新增**（精确显示运行时长） |
| ❌（无） | `GetDeprecatedWarnings` | **新增**（升级后主动提示） |
| ❌（无） | `StartNetworkQualityTest` / `StartSTUNTest` | **预留**（本次不实现 UI，预留后端命令） |

## 4 阶段执行

### 阶段 1：配置层切换 + UI 隐藏（不涉及新依赖）

- `src-tauri/src/app/singbox/config_generator.rs`：`experimental.clash_api` 块替换为顶层 `services: [{type:"api", listen, listen_port, dashboard}]`。
- `src-tauri/src/app/core/kernel_service/embedded.rs`：删除 `ensure_external_ui()`（metacubexd 预下载）。
- `src-tauri/src/app/system/config_service.rs`：删除 `update_singbox_config_ports` 中 `clash_api` 端口同步；只保留 inbound 端口同步。
- `src-tauri/src/app/constants/core/paths.rs`：删除 metacubexd 路径相关常量（如有）。
- `src-tauri/src/lib.rs`：`invoke_handler` 移除 Clash API 命令（保留 inbound 写入相关命令）。
- 前端路由：将 `ProxiesView` 中的"规则"tab、`ProxyProvidersView` 等路由 meta 标记 `hidden: true`。
- 前端托盘 / 设置：移除"打开内置 web"按钮（`tray-service.ts` / `SettingsView.vue`）。
- 验证：`cargo check` + `pnpm type-check`。

### 阶段 2：gRPC-Web over WebSocket 客户端 + 核心交互替换

- `Cargo.toml`：新增 `tonic` 拒绝；新增 `prost`、`prost-build`、`bytes`、`tokio-tungstenite`（已存在）。
- `src-tauri/src/app/api/`：新建模块。
  - `proto/started_service.proto`：从 sing-box 官方仓库复制（pin 到 1.14）。
  - `build.rs`：prost-build 编译 proto。
  - `mod.rs`：导出 `SingBoxApiClient`。
  - `grpc_client.rs`：手写 gRPC-Web over WebSocket 客户端（参照 dashboard `src/api/websocket.ts:111-140` 帧格式：`[flag:1][length:4 BE][body:N]`，flag `0x80` = metadata）。
  - `manager.rs`：连接生命周期、指数退避重连、单实例管理。
- `src-tauri/src/app/core/proxy_service.rs`：重写。
  - 删除所有 `http_client::get_json` / `client.put/patch/delete` 调用。
  - 新增 Tauri 命令：`kernel_get_groups`（SubscribeGroups 单快照）、`kernel_select_outbound(group, outbound)`、`kernel_url_test(outbound_tag)`、`kernel_get_clash_mode`、`kernel_set_clash_mode(mode)`、`kernel_set_group_expand(group, is_expand)`、`kernel_get_started_at`、`kernel_get_deprecated_warnings`、`kernel_close_connection_v2`、`kernel_close_all_connections_v2`。
  - 保留：`set_system_proxy` / `set_tun_proxy` / `set_manual_proxy`（写 inbound 配置）、`apply_os_proxy`。
- `src-tauri/src/app/core/event_relay.rs`：删除 4 个独立 WebSocket；替换为 3 个 gRPC server-streaming 中继（traffic 走 SubscribeStatus 的 Status、log 走 SubscribeLog 的 Log、connection 走 SubscribeConnections 的 ConnectionEvents）。
- 前端 `services/proxy-service.ts`：删除 Clash API 方法，改调新 Tauri 命令；类型替换为 protobuf 解码后的 Group/GroupItem/Status/Log/Connection。
- 前端 `services/connection-service.ts`：删除，改调 `kernel_close_connection_v2`。
- 前端 `services/rule-service.ts`：删除。
- 前端 `types/controller/`：删除 Clash 类型，新增 protobuf 解码类型（与后端 proto schema 同步）。
- 验证：手动启动内核（带 `type: api`） → 跑 `pnpm tauri dev` → 联调节点选择/测速/连接管理/流量/日志/Clash 模式/组展开。

### 阶段 3：清理旧代码

- 删除 `proxy_service.rs` 中所有 Clash API HTTP 调用代码（`build_controller_url`、`fetch_controller_json`、`put_controller`、`patch_controller_json`、`delete_controller`）。
- 删除 `event_relay.rs` 中的 `EventDirectRelay`（如果不再需要）。
- 删除 `src/services/connection-service.ts`、`src/services/rule-service.ts`。
- 删除 `src/types/controller/` 整个目录。
- `lib.rs` 命令注册同步：移除所有 Clash API 相关 Tauri 命令。
- 验证：`cargo check` + `cargo clippy` + `pnpm type-check` + `pnpm lint`。

### 阶段 4：文档 + 全门禁

- 更新 `AGENTS.md`（根 + `src-tauri/AGENTS.md` + `src-tauri/src/app/AGENTS.md` + `src/AGENTS.md`）：
  - "WHERE TO LOOK" 表更新：`core/proxy_service/` 改用 gRPC；`core/event_relay.rs` 改流式订阅；删除 `core/kernel_service/embedded.rs::ensure_external_ui` 描述；前端 `services/proxy-service.ts` 改 gRPC 调用。
  - "NOTES" 注明：协议从 Clash API 迁移到官方 gRPC API；metacubexd 不再使用；规则与代理提供者 UI 待官方 API 暴露后启用。
- 更新 `docs/CHANGELOG.md`：加 `Unreleased` 条目记录本迁移。
- 全门禁：`cargo clippy` + `cargo test` + `pnpm lint` + `pnpm type-check`。

## 风险与回退

- **协议细节**：gRPC-Web over WebSocket 帧格式在 dashboard 前端代码里有参考，但官方文档缺失。实施阶段如遇 frame flag / trailers / trailers-only 差异需对照 sing-box 1.14.0 源码确认。
- **API 版本协商**：`GetVersion` 返回 `apiVersion`，与客户端期待的 minimum 比较；1.14 之前内核没有 `type: api`，需在前端 store 提示"内核版本过低"。
- **回退策略**：本工程涉及 4 阶段，每个阶段独立 commit；任意阶段不能回滚的复杂度可控（最坏回到 Clash API：恢复 `experimental.clash_api`、恢复 metacubexd 预下载、关闭 `services[].type=api`）。

## 不在本次范围

- `experimental.cache_file` 等与 API 无关的 experimental 配置：保留。
- 自定义规则 CRUD（issue #62）：保持现有"写文件 + 重启内核"实现，不动。
- `experimental.clash_api` 内部的 `cache_file.store_fakeip` 等：保留。
- 内核下载、版本管理、托盘 UI 等：不动。