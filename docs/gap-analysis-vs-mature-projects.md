# 竞品对标与改进建议（2026-10）

**结论先行**：本项目的**技术底座已达一线水平**（sing-box 1.14.4 原生 gRPC API、事件驱动运行态、schema 驱动配置、CI 全门禁），**不缺架构，缺产品面**。

对照 4 个成熟开源项目 + sing-box 官方桌面客户端后，差距集中在三类：

| 类别 | 数量 | 特征 | 投入产出比 |
|---|---|---|---|
| **A. 存量能力未接线** | 4 项 | 后端命令 + ts-rs 类型已就绪，前端零消费 | **最高**，改 UI 即可 |
| **B. 竞品标配能力缺失** | 6 项 | WebDAV / 覆写 / 规则编辑 / 快捷键 / PAC / 订阅转换 | 高，2–3 周可闭环 |
| **C. 工程重构与质量** | 5 项 | 双模型收敛、大文件拆分、代码生成、诊断模块 | 中，长期收益 |

> **执行状态（2026-10-19）**：本文档提出的 **阶段 0 / 阶段 1 / 阶段 2 已全部实施完毕**——
> A1–A5（存量接线 + 类型基线）、D1（配置双模型收敛）已闭环，
> 实施细节见 [`implementation-plan-2026-10.md`](./implementation-plan-2026-10.md)。
> B1–B6 / C1–C4 / D2–D6 仍为待办。
> 详见下文各条目的「状态」标记。
>
> ⚠️ 复核阶段 2 时新发现一条既有缺口 **D7**（mDNS / Tailscale endpoint / 下载出站三个设置
> 在补丁路径上不落地），详见该条目。

> 本文档只列**有证据的差距**。每条都标注了本仓库的具体文件位置与竞品出处，便于直接排期。

---

## 一、现状盘点：已经做得比竞品好的部分

先说结论里"不差"的部分，避免后续重复投入。

| 维度 | 本项目 | 对比 |
|---|---|---|
| 内核交互 | sing-box 1.14.4 官方 gRPC API（`type: "api"`），手写 protobuf 无 prost 依赖 | 官方桌面客户端同为 1.14 gRPC；CVR/Mihomo 仍走 Clash RESTful |
| 1.14 特性接入 | `optimistic` 缓存、mDNS、`http_clients`+`default_http_client` 替代 `download_detour`、`services:[api]`、Tailscale endpoint、SSH server、TLS spoof | 与官方客户端 1.14 能力面基本对齐 |
| deprecated 清理 | `experimental.clash_api` / `store_rdrc` / `independent_cache` / `download_detour` 全部迁移并有 `settings_patch` 兜底清残留 | 多数竞品未做 1.16 提前适配 |
| 配置一致性 | `appConfigSchema.ts` schema 驱动，加字段只改 1 处（原为 7 处） | 竞品普遍手写重复枚举 |
| 质量门禁 | lint / type-check / prettier / i18n key 对齐 / vitest 158 项 / cargo fmt / clippy / cargo test 194 项 / 国家码同步 / 内核目标矩阵，全部进 CI | 与 CVR 同级，CVR 另有 `crates/` 拆分 |
| 跨平台 | Win x64+ARM64 / Linux / macOS，ARM64 TLS spoof 按平台隐藏 | 竞品多数缺 ARM64 |

**结论：不要动的部分**——架构分层、事件链路、CI 门禁、schema 驱动配置，这四项已优于多数开源项目。

---

## 二、竞品基线（2026-10 实测）

| 项目 | 技术栈 | Star | 关键能力（README 摘要） |
|---|---|---|---|
| **Clash Verge Rev** | Rust + Tauri 2 + React | 152k | 配置文件 **Merge/Script 增强**、**可视化节点与规则编辑**、**WebDAV 备份同步**、主题色/托盘图标/**CSS Injection**、内核 Alpha 轨、8 语言 |
| **v2rayN** | C# / Avalonia | 118k | 多内核切换、**PAC 模式**、**真连接延迟(Real Ping)**、局域网共享、订阅**流量余量/到期时间**、快捷键切节点、二维码分享 |
| **FlClash** | Dart / Flutter | 56k | **内置配置编辑器 + 覆写脚本 + 自定义规则/代理/代理组**、实时**DNS 查询**视图、**WebDAV 备份**、桌面**全局快捷键**、F-Droid |
| **Mihomo Party** | TS + Tauri | 26k | **一键 Smart Core 规则覆写**、**Sub-Store 深度集成**、WebDAV 备份、**强大的覆写功能**、无服务模式 TUN |
| **GUI.for.SingBox** | Go/Wails + Vue3 | 8k | **插件中心**（sub-store / speedtest / 解除UWP回环 / AdGuardHome）、**向导模式**、逐字段配置 UI（inbounds/outbounds/route/dns） |
| **sing-box 官方桌面端** | Electron + React | 85 | `profiles.ts` **配置档管理**、`profileEditorWindows.ts` **独立编辑窗口**、`taildrop.ts` **Taildrop 收件**、`sharing.ts` **局域网共享**、`doctor.config.json` **诊断**、`repair.ts` **自修复**、`userDataSecurity.ts` **数据安全**、`x11Tray.ts` + `native/wayland-menu` **Linux 托盘** |

> 竞品共性：**备份同步、配置覆写、规则可视化、快捷键** 是四项高频出现的标配，本项目**全部缺失**。

---

## 三、P0-A｜存量能力未接线（成本最低、见效最快）

> 这类问题的本质是"后端 80% 完成度卡在最后 20% 的 UI"。

### A1. `get_outbounds` 后端就绪，前端零消费

> ✅ **已完成（2026-10-19）**：新增「出站总览」页（`src/views/OutboundsView.vue` + `src/composables/useOutboundList.ts`，按延迟排序），经 `src/services/proxy-service.ts` 的 `getOutbounds()` 消费。
> ⚠️ **2026-10-20 修正形态**：最初实现为代理页底部的卡片，与代理区争抢高度导致代理页主体被压扁；中间态改成抽屉仍占用代理页工具栏。现改为**独立页面**（侧边栏「出站」入口，路由 `/outbounds`），代理页零占用。

- 证据：`src-tauri/src/app/singbox_api/client.rs:245` 已实现；`src/` 下 **0 处**引用
- 价值：`GetOutbounds` 返回全部出站（含延迟），是"全节点按延迟排序"、"出站总览"、"服务列表"的数据源
- 建议：主页节点选择器支持"全部节点按延迟排序"；出站总览独立成页面（已实现，路由 `/outbounds`）

### A2. `network_quality_test` 后端就绪，前端零消费

> ✅ **已完成（2026-10-19）**：新增 `src/views/setting/components/SettingsDiagnosticsCard.vue`（维护页「网络诊断」，5 项质量指标），经 `networkQualityTest()` 消费。

- 证据：`client.rs:265` 已实现；`src/` 下 **0 处**引用
- 价值：sing-box 官方 1.14 的 `NetworkQualityTest` 提供丢包/抖动/带宽
- 建议：设置页 → 维护 Tab 增加"网络诊断"，输出可复制的诊断报告（见 C4）

### A3. `get_services` / `probe_service_status` 后端就绪，前端零消费

> ✅ **已完成（2026-10-19）**：诊断卡片内含服务状态列表；`probe_service_status` 已补为 Tauri 命令
> （`src-tauri/src/app/core/proxy_service/grpc_bridge.rs` + `lib.rs` 注册），前端经 `probeServiceStatus()` 调用。

- 证据：`client.rs:235` / `client.rs:449` 已实现；`src/` 下 **0 处**引用
- 价值：展示运行态服务（Tailscale / USB / IP endpoint），是 1.14 桌面的核心卖点之一
- 建议：新增"服务"卡片；Tailscale endpoint 开启后展示在线节点与 Taildrop 状态

### A4. `RulesView.vue` 是只读表，规则不可管理

> ✅ **已完成（2026-10-19）**：`src/views/RulesView.vue` 重写为「运行时规则（只读 `get_rules`）+ 自定义规则（CRUD）」双卡片；
> 自定义规则走 `src/services/custom-rule-service.ts` + `src/stores/kernel/CustomRuleStore.ts` + `src/components/rules/CustomRulePanel.vue`。

- 证据：`src/views/RulesView.vue:23` 仅 `n-data-table` 展示 `get_rules` 结果，无启停/编辑
- 现状：后端 **5 个自定义规则命令** + ts-rs 类型（`src/types/generated/CustomRule.ts`）**已完全就绪**，但 `src/` 下只有类型定义、**无任何调用**；`src/AGENTS.md:68` 自述"前端 UI 尚未实现"
- 建议：规则页拆为「运行时规则表（只读）+ 自定义规则 CRUD（可编辑）」双 Tab

> **预期收益**：A1–A4 合计约 2–3 天工作量，显著补齐"规则可视化"这一竞品标配。

### A5. gRPC 响应类型未走 ts-rs，前端手写 interface 存在静默漂移风险

> ✅ **已完成（2026-10-19，阶段 0）**：22 个 gRPC 类型已加 `TS` derive 并导出到仓库根 `src/types/generated/`（唯一事实来源，git 跟踪）；
> 前端手写的 `GroupItem`/`ProxyGroup`/`GroupsData`/`ClashModeStatus` 已删除；
> 新增 `src/types/generated/grpc-types.test.ts` 守护漂移。详见实施方案「阶段 0」。

- 现状：`src-tauri/src/app/singbox_api/types.rs` 的 `Groups` / `Group` / `GroupItem` / `OutboundList` / `Rule` / `RuleList` / `Service` / `ServiceList` / `NetworkQualityResult` / `DeprecatedWarnings` / `ClashModeStatus` 等 **全部未加 `TS` derive**；前端在 `src/services/proxy-service.ts:6-32` **手写**了 `GroupItem` / `ProxyGroup` / `GroupsData` / `ClashModeStatus`
- 风险：Rust 改字段名 → 前端**编译不报错**，运行时静默拿到 `undefined`。这与仓库自身约定"跨语言类型经 ts-rs 导出，不手写"相悖
- 附带发现：`src-tauri/src/types/generated/` 已被 `.gitignore:33` 忽略（`git ls-files` 返回 0 个跟踪文件），磁盘上残留的是 **ts-rs v7 时代**（`AlexanderD-r/ts-rs` 头注释）的本地遗留物；`CLAUDE.md:23` 仍写着"落在 `src/types/generated/` 与 `src-tauri/src/types/generated/`，两侧都不手改"——**该表述与实际不符**
- 建议：见实施方案「阶段 0」

---

## 四、P0-B｜竞品标配能力缺失

### B1. WebDAV 备份 / 同步（竞品 3/4 标配，本项目缺失）

- 现状：`src-tauri/src/app/system/backup_service.rs:610` 只有 `backup_export_snapshot` / `backup_import_snapshot` **本地快照**；全仓库 `webdav` **0 处**匹配
- 竞品：CVR「WebDav 配置备份和同步」、Mihomo Party「通过 WebDAV 一键备份和恢复配置」、FlClash「Backup and restore through WebDAV or a local file」
- 建议：`backup_service.rs` 增 `WebDavBackend` trait（`LocalFs` / `WebDav` 双实现），复用现有快照格式；WebDAV 需支持 Basic/Bearer 鉴权、目录级原子写入、失败重试与凭据本地加密存储
- 风险点：凭据落盘必须走 `userDataSecurity` 思路（官方客户端有此模块），不要明文写 `generic_config`

### B2. 配置覆写（Merge / Script）——竞品 3/4 都有

- 现状：无。`src-tauri/src/app/singbox/config_generator.rs` 直接生成、`settings_patch.rs` 再打补丁，两者是**硬编码双模型**，用户无法介入
- 竞品：CVR「配置文件管理和增强（Merge 和 Script）」、Mihomo Party「强大的覆写功能」、FlClash「override scripts」
- 建议：引入**覆写层**（`overrides`）概念，位于 `订阅配置 → 覆写 → 生成` 三段之间：
  1. Merge 覆写：按 JSON 路径深合并（`outbounds[].tag=proxy` 这类）
  2. Script 覆写：JS 片段对最终 config 求值（需明确沙箱边界）
- **注意**：这是本项目架构影响最大的一项，需要先做 C1 的双模型收敛，否则覆写层会再叠一层补丁

### B3. 全局快捷键（FlClash 明确具备）

- 现状：全仓库 `global_shortcut` / `hotkey` **0 处**匹配
- 竞品：FlClash「a tray menu and **global hotkeys** on the desktop」；v2rayN「自定义快捷键切换节点」
- 建议：Tauri `tauri-plugin-global-shortcut`，默认绑定 `Ctrl+Shift+T` 切代理开关、`Ctrl+Shift+N` 切下一个节点；设置页可配置

### B4. PAC 模式（v2rayN 具备）

- 现状：全仓库 `\bpac\b` / `autoConfigURL` **0 处**匹配（此前 grep 命中的 `package` 已排除）
- 竞品：v2rayN「支持 PAC 模式，能够根据远端配置文件动态更新分流列表」
- 建议：Windows 侧系统代理注册 `AutoConfigURL`；macOS/Linux 走 PAC 文件 + `network proxy` 约定
- 价值：补齐"不改全局代理、只让浏览器走 PAC"的合规/办公场景，是 Windows 用户高频需求

### B5. 订阅转换（Sub-Store / subconverter）

- 现状：全仓库 `Sub-Store` / `subconverter` **0 处**匹配（此前命中的 `SubStore` 是本项目自己的 Pinia store）
- 竞品：Mihomo Party「**深度集成 Sub-Store**，轻松管理订阅」、GUI.for.SingBox 插件中心内置 sub-store
- 建议：**不要**内置 Sub-Store 服务（运维成本高）。改为"订阅转换服务地址"配置项——允许用户填写自建 Sub-Store 实例地址，订阅更新时先过转换再入库；这正是 Mihomo Party 的做法

### B6. 规则可视化的可编辑化（与 A4 合并实现）

- 竞品：CVR「**可视化节点和规则编辑**」、GUI.for.SingBox 逐字段规则 UI
- 建议：自定义规则支持 `domain` / `domain_suffix` / `domain_keyword` / `ip_cidr` 四种匹配（后端 `CustomRuleMatchType` 已定义）+ `direct` / `proxy` / `block` 三种动作（`CustomRuleAction` 已定义），前端只需做表单

---

## 五、P1｜利用 sing-box 1.14 差异化的机会

### C1. 配置编辑器 + JSON Schema 补全（官方 1.14 提供 schema）

- 官方客户端有 `profileEditorWindows.ts`（独立编辑窗口）；本项目无任何配置编辑入口
- 1.14 内核提供 `sing-box schema` 输出 JSON Schema，可做补全 + 错误下划线
- 建议：设置页 → 高级 → "编辑当前配置"，用 CodeMirror/Monaco 加载 schema；保存前走现有 `config_validator.rs` 二次校验
- 价值：把"本项目只认自己生成的配置"这一限制打开，是与通用客户端的**核心差异点**

### C2. Taildrop 收件箱

- 官方客户端有 `src/main/taildrop.ts`；本项目 `AppConfig.tailscale_taildrop_directory` 字段已有（`appConfigSchema.ts:231`），但**只写入配置、不读目录**
- 建议：读该目录并在设置页/服务卡片展示文件列表与"打开所在目录"

### C3. DNS 实时查询视图

- FlClash「Live views of connections, **requests, DNS queries** and logs」
- 本项目有 connections / log 两条流，**无 DNS 查询流**
- 建议：接入 `SubscribeDNS`（若 1.14 gRPC 暴露）或从日志侧解析，作为连接页的第三个分组维度

### C4. 诊断 / 自修复模块（doctor）

- 官方客户端有 `doctor.config.json` + `reports.ts` + `repair.ts` + `appReports.ts`
- 本项目有 `kernel_service/guard.rs` 自愈，但**无用户可读的诊断报告**
- 建议：新增"诊断"页，采集内核版本、配置校验结果、端口占用、系统代理状态、权限状态、最近内核日志，导出可粘贴到 issue 的文本；一键"修复"执行内置修复动作（重建配置 / 重置系统代理 / 清理残留进程）

### ~~C5. 主页 Clash 三档快捷切换~~（**已于 2026-10-19 订正：该项已实现，无需再做**）

- **订正说明**：初稿误判"主页无 Clash 切换入口"。实际核查：`src/views/HomeView.vue:131-147` 已有 `mode-chips` 三段式切换，`src/composables/useHomeDashboard.ts:96` 的 `handleNodeProxyModeChange` 调 `kernel-service.ts:308` 的 `switchNodeProxyMode` → 后端 `toggle_proxy_mode`（`app/network/subscription_service/mode.rs:28`，含持久化 + gRPC 立即生效 + 启动恢复）。
- 补充：后端 `PROXY_MODES` 目前**只含 `global` / `rule` 两档**（`mode.rs:9`），不含 `direct`。`mode.rs:24-27` 的注释明确禁止再往 `route.rules` 写 `clash_mode: rule` 的 catch-all 规则（会导致全部流量命中单条规则且 outbound tag 无法对齐）。若后续要加"直连"档，需同时扩展 `PROXY_MODES` 并验证 `config_generator.rs:154-155` 的 `clash_mode: direct` 规则对。

---

## 六、P2｜工程重构与质量

### D1. `config_generator` + `settings_patch` 双模型收敛（**优先级最高的重构**）

> ✅ **已完成（2026-10-19，阶段 2）**：见实施方案「阶段 2」。
> 落地为三个新模块：`config_intent.rs`（`AppConfig` → 配置语义的唯一纯映射）、
> `config_render.rs`（生成/补丁共用的纯函数）、`ownership.rs`（配置归属判定 + 落库）。
> 生成路径**不再回调**补丁路径；补丁路径改为**按归属**打补丁，外来配置只同步端口。
> 行为等价由 `config_golden.tests.rs` 的 4 组黄金样本锁定（逐字段 diff 为空）。

- 现状：`config_generator.rs`（36KB）负责"从零生成"，`settings_patch.rs`（26KB）负责"在已有 config 上打补丁"，两者对同一批语义（DNS、rule-set、outbound、TUN）**各写一遍**
- 后果：新增一个 sing-box 字段要改 2 个地方；`settings_patch.rs` 的 `apply_profile_settings_if_present` 需要判断"是不是本程序生成的配置"，逻辑脆弱
- 建议：收敛为**单一渲染器**——以 `SingBoxConfig`（`config_schema.rs`）为唯一中间表示，`AppConfig` → `SingBoxConfig` → `Value`；用户导入的原始配置走"解析 → 合并 → 渲染"同一条路径（这同时是 B2 覆写层的前置条件）
- 收益：消除约 30% 的配置逻辑重复；B2/C1 的实现成本随之下降

### D2. 大文件拆分

| 文件 | 大小 | 建议 |
|---|---|---|
| `app/network/subscription_service/parser.rs` | 56KB | 按协议拆 `parser/vless.rs`、`vmess.rs`、`trojan.rs`、`hysteria2.rs`、`tuic.rs`、`anytls.rs` |
| `process/manager.rs` | 42KB | 拆 `manager/spawn.rs`、`manager/health.rs`、`manager/terminate.rs` |
| `app/storage/database.rs` | 38KB | 拆 `database/schema.rs`（迁移）、`database/app_config.rs`、`database/subscription.rs` |
| `app/core/kernel_service/runtime.rs` | 31KB | 拆 `runtime/lifecycle.rs`、`runtime/config_watch.rs` |
| `app/singbox/config_generator.rs` | 36KB | 随 D1 一并收敛 |

> 本项目已有 `kernel_service/` / `proxy_service/` / `tray/` 的**按职责拆子模块**先例（批次 3-D/3-E/3-K），本项是同一模式的延续，不是新引入风格。

### D3. 手写 protobuf 的长期成本

- 现状：`singbox_api/proto.rs`（22KB）手写 wire-format 解码，无 prost 依赖（这是刻意的设计决策，好处是零构建期依赖）
- 代价：新增 1.14 接口要手写字段；`proto.rs` 内有 10 处 `unwrap`（同类文件里最高的非测试值）
- 建议：**不改架构**。改为在 `scripts/` 加一个 `sing-box schema`/proto 拉取脚本，生成字段清单并**用测试断言手写解码器与 proto 定义一致**（类似现有 `gen-country-codes.mjs` + `countries.generated.test.ts` 的守护模式）
- 备注：官方 proto 用 `buf.gen.yaml` 管理，成熟项目普遍如此；本项目不必引入 buf，但要有"漂移检测"

### D4. i18n 文件拆分

- 现状：4 个语言包各是**单一大文件**（`src/locales/zh-CN.ts` 等）
- 竞品：CVR 有 140 个 locale 文件（按页拆分）
- 风险：单文件超过 ~1500 行后，**新增语言时的合并冲突成本线性上升**（本项目已有 4 语言，`zh-CN` 是基准）
- 建议：按页面/域拆分为 `locales/zh-CN/home.ts`、`proxy.ts`、`setting.ts` …，主文件聚合导出；`scripts/check-i18n.mjs` 改为递归扫描
- 收益：为将来加第 5、6 种语言（竞品普遍 6–8 种）做准备

### D5. 内核版本缓存的完整性校验

- 现状：`kernel_service/versioning.rs` 把版本号缓存进 SQLite
- 风险：用户手动替换 `resources/kernel/` 下的 exe 后，缓存不失效，UI 显示的版本与实际二进制不符
- 建议：缓存项增加 exe 文件 hash 或 mtime+size 校验

### D6. Linux 托盘兼容性

- 官方客户端专门实现了 `x11Tray.ts` + `native/wayland-menu`（Rust NAPI 原生菜单）+ `linux-after-install.sh`
- 建议：检查本项目在 Wayland 下的托盘菜单可用性；AppImage/deb/rpm 安装后是否需要 `ayatana-appindicator` 依赖提示（FlClash README 明确提示 `libayatana-appindicator3-1`）

### D7. 三个设置项在补丁路径上不落地（**既有问题，阶段 2 收敛后依然存在**）

- 现状：生成路径（`config_generator`）会写三处东西，而补丁路径（`settings_patch`）对它们**零处理**：

  | 设置项 | 生成路径写入 | 补丁路径 |
  |---|---|---|
  | `enable_tailscale_endpoint` | 顶层 `endpoints`（Tailscale endpoint） | 无处理 |
  | `singbox_dns_use_mdns` | `dns.servers` 的 `dns_mdns` + 对应 DNS 规则 | 无处理 |
  | `singbox_download_detour` | 顶层 `http_clients` + `route.rule_set[].http_client` | 只移除 deprecated 的 `download_detour` |

- 后果：用户在设置页改这三项后，**已有配置不会被改写**——改了等于没改，
  只有下次订阅刷新（重新生成配置）才生效。这是典型的"设置项看着改了、实际没生效"。
- 证据：`git show HEAD:src-tauri/src/app/singbox/settings_patch.rs` 中
  `endpoints` / `tailscale` / `mdns` 各 **0 处**匹配，`http_clients` 仅 **1 处注释**——
  **该缺口在双模型收敛之前就存在**，不是收敛引入的。
- 阶段 2 的收敛做到的是「**同一份 intent 的两条路径产物逐字段一致**」
  （由 `config_golden.tests.rs` 的往返断言锁死），**没有**补齐"设置变更 → 已有配置"这条方向的落地。
  这两件事不同：前者是"两条路径不打架"，后者是"设置真的能落到已有配置"。
- 建议：把三处的渲染逻辑同样提到 `config_render.rs` 由两条路径共用，补丁路径按归属改写。
  ⚠️ 风险点：Tailscale endpoint 写入后若缺 auth key 会**导致内核启动失败**，
  因此需要"用户显式开启且已配置 auth key 才写入"的额外判定，不能无条件跟随设置开关。

#### ✅ 已部分修复：`singbox_download_detour`（内核 1.14.4 适配时）

- 补丁路径新增 `sync_http_clients` + `sync_rule_set_http_client_refs`
  （`src-tauri/src/app/singbox/settings_patch.rs`），渲染逻辑提到
  `config_render::build_http_clients` 由两条路径共用：
  - 改下载偏好后，**已有配置**的 `http_clients` 会被重写
  - 偏好从「手动」切回「直连」时，指向已删除客户端的悬空 `route.rule_set[].http_client` 引用一并清掉
  - 外来配置（`Ownership.managed == false`）完全不动
- 顺带修掉一个会让**内核启动失败**的缺陷：直连客户端曾写成 `detour: "direct"`，
  指向空 direct 出站 → sing-box 1.14+ 直接 `FATAL`（详见 `singbox/config_render.rs::build_http_clients` 注释）
- ⚠️ 剩余缺口：`enable_tailscale_endpoint` / `singbox_dns_use_mdns` 两项**仍未落地**，待后续批次处理

---

## 七、建议执行顺序

| 批次 | 内容 | 工作量 | 收益 |
|---|---|---|---|
| **第 1 批** ✅ | A1–A4（存量接线）+ A5（类型基线对齐） | 2–3 天 | 立刻补齐"规则可视化 + 出站总览 + 诊断" |
| **第 3 批（前半）** ✅ | D1（双模型收敛） | 2–3 天 | 打开"用户自定义配置"这一核心差异点的前置条件 |
| **第 2 批** | B3（快捷键）+ B4（PAC）+ B1（WebDAV） | 1–2 周 | 补齐三项竞品标配 |
| **第 3 批（后半）** | B2（覆写层）→ C1（配置编辑器） | 1–2 周 | 打开"用户自定义配置"这一核心差异点 |
| **第 4 批** | A2/A3 + C2（Taildrop）+ C3（DNS 视图）+ C4（诊断模块） | 1–2 周 | 吃满 1.14 能力，差异化于 Clash 系 |
| **第 5 批** | D2（文件拆分）+ D3（proto 守护）+ D4（i18n 拆分）+ D5 + D6 | 持续 | 降低长期维护成本 |
| **随时** | B5（订阅转换服务地址配置项） | 0.5 天 | 低成本对齐 Mihomo Party |

> ✅ = 已于 2026-10-19 实施完毕。第 1 批与 D1 的实施细节见
> [`implementation-plan-2026-10.md`](./implementation-plan-2026-10.md)，
> 验收记录见 [`CHANGELOG.md`](./CHANGELOG.md) 的 `[未发布]` 段。

---

## 八、明确**不建议**做的事

避免对标过程中引入不必要的复杂度：

1. **不要做插件市场**。GUI.for.SingBox 的插件中心是 Wails 生态特性，维护成本极高（其自家文档也承认插件存在 bug），本项目走 sing-box 专用路线不需要。
2. **不要做多内核支持**。CVR 支持 mihomo、v2rayN 支持 Xray/v2fly/flcore 属于历史包袱；本项目专注 sing-box 1.14 是**优势**，切换内核会让 1.14 差异化能力（mDNS/Taildrop/TLS spoof）失效。
3. **不要内置 Sub-Store 服务**。只提供"自建转换服务地址"配置项（B5）。
4. **不要引入 prost/buf 改变 protobuf 方案**。零构建期依赖是当前设计的优点，用 D3 的"漂移检测测试"解决长期成本即可。

---

## 九、证据索引

| 结论 | 证据位置 |
|---|---|
| 内核 1.14.4 | `src-tauri/resources/kernel/windows/amd64/version.txt` |
| 1.14 特性已接入 | `src-tauri/src/app/singbox/config_generator.rs:217,241,285,289` |
| deprecated 清理 | `src-tauri/src/app/singbox/settings_patch.rs:39,62,75,291` |
| `get_outbounds` 未接线 | `src-tauri/src/app/singbox_api/client.rs:245`；`src/` 0 引用 |
| `network_quality_test` 未接线 | `src-tauri/src/app/singbox_api/client.rs:265`；`src/` 0 引用 |
| `get_services` 未接线 | `src-tauri/src/app/singbox_api/client.rs:235`；`src/` 0 引用 |
| 自定义规则前端未接线 | `src/types/generated/CustomRule.ts`（仅类型）；`src/AGENTS.md:68` |
| 规则表只读 | `src/views/RulesView.vue:23-31` |
| 无 WebDAV | 全仓库 `webdav` 0 匹配；`backup_service.rs:610,631` 仅本地快照 |
| 无快捷键 | 全仓库 `global_shortcut`/`hotkey` 0 匹配 |
| 无 PAC | 全仓库 `\bpac\b`/`autoConfigURL` 0 匹配 |
| 无 Sub-Store 集成 | 全仓库 `Sub-Store`/`subconverter` 0 匹配 |
| ~~Clash 模式仅在设置页~~（已订正为"主页已实现"） | `src/views/HomeView.vue:131-147`；`src/composables/useHomeDashboard.ts:96`；`src-tauri/src/app/network/subscription_service/mode.rs:9,28` |
| 双模型 | `config_generator.rs`（36KB）+ `settings_patch.rs`（26KB） |
| 大文件 | `parser.rs` 56KB / `manager.rs` 42KB / `database.rs` 38KB / `runtime.rs` 31KB |
| 测试规模 | 前端 158 项（`src/**/*.test.ts`）、Rust 194 项（`#[test]` / `#[tokio::test]`） |
| CI 门禁 | `.github/workflows/ci.yml:63-96` |
| 竞品基线 | Clash Verge Rev 152k★ / v2rayN 118k★ / FlClash 56k★ / Mihomo Party 26k★ / GUI.for.SingBox 8k★ / sing-box-for-desktop 85★（2026-10-19 通过 GitHub API 核实） |
| 官方客户端模块 | `SagerNet/sing-box-for-desktop`：`src/main/{profiles,profileEditorWindows,taildrop,sharing,doctor.config,repair,userDataSecurity,x11Tray}` |
