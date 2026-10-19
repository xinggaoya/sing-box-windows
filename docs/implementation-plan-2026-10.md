# 实施方案：① 存量能力接线 + ③ 配置双模型收敛

**编写日期**：2026-10-19
**对应文档**：`docs/gap-analysis-vs-mature-projects.md`（差距分析）
**范围**：仅覆盖差距分析中的 ①（存量能力接线）与 ③（配置生成双模型收敛），不含 WebDAV / 覆写 / 快捷键 / PAC 等竞品标配补齐项。

> **实施状态（2026-10-19）：阶段 0 / 阶段 1 / 阶段 2 已全部实施完毕并通过全部门禁。**
>
> | 阶段 | 状态 | 落地 |
> |---|---|---|
> | 阶段 0 类型基线 | ✅ | `src-tauri/src/app/singbox_api/types.rs` 全量 ts-rs 导出 → 仓库根 `src/types/generated/`；前端手写 interface 删除；`src/types/generated/grpc-types.test.ts` 守护 |
> | 阶段 1 存量接线 | ✅ | `OutboundOverviewCard.vue` / `SettingsDiagnosticsCard.vue` / `CustomRulePanel.vue` + `custom-rule-service.ts` + `CustomRuleStore.ts` + `probe_service_status` 命令 |
> | 阶段 2 双模型收敛 | ✅ | `config_intent.rs` / `config_render.rs` / `ownership.rs`；生成路径不再回调补丁路径；补丁路径改为按归属打补丁 |
>
> 验收：后端 232 项测试 + 黄金样本逐字段 diff 为空 + 「生成/补丁双模型收敛断言」通过；前端 165 项测试；
> `pnpm lint / type-check / format:check / test:i18n / test / test:kernel-targets / test:country-codes`、
> `cargo fmt --check`、`cargo clippy --all-targets -- -D warnings`、`cargo test` 全绿。
> 变更记录见 [`CHANGELOG.md`](./CHANGELOG.md) 的 `[未发布]` 段。

---

## 0. 前置订正

实施前对 gap-analysis 做了一轮代码级复核，**订正一处错误结论**：

| 原文结论 | 实际情况 | 处理 |
|---|---|---|
| C5「主页 Clash 三档快捷切换」缺失 | **已实现**：`HomeView.vue:131-147` 有 `mode-chips` 切换条；`useHomeDashboard.ts:96` → `kernel-service.ts:308` `switchNodeProxyMode` → 后端 `toggle_proxy_mode`（`mode.rs:28`，含持久化 + gRPC 立即生效 + 启动恢复） | 从待办中移除，gap-analysis 已同步订正 |

补充事实：后端 `PROXY_MODES` 只含 `global` / `rule` **两档**（`mode.rs:9`），不含 `direct`。`mode.rs:24-27` 明确禁止再写 `clash_mode: rule` 的 catch-all 规则。若未来要加"直连"档，需同时扩展 `PROXY_MODES` 并验证 `config_generator.rs:154-155` 的 `clash_mode: direct` 规则对。

---

## 1. 现状事实核查（已逐条核实）

### 1.1 存量能力矩阵

| 能力 | 后端实现 | Tauri 命令 | 返回类型 | ts-rs 导出 | 前端消费 | UI 落点 |
|---|---|---|---|---|---|---|
| 出站总览 | `client.rs:245` `get_outbounds` | ✅ `lib.rs:258` | `OutboundList{outbounds:Vec<GroupItem>}` | ❌ | ❌ 0 引用 | 代理页新增"出站总览" |
| 网络诊断 | `client.rs:265` `network_quality_test` | ✅ `lib.rs:260` | `NetworkQualityResult{5 字段}` | ❌ | ❌ 0 引用 | 设置页 → 维护 Tab |
| 服务列表 | `client.rs:235` `get_services` | ✅ `lib.rs:257` | `ServiceList{services:Vec<Service>}` | ❌ | ❌ 0 引用 | 主页/设置页"服务"卡片 |
| 服务状态探测 | `client.rs:449` `probe_service_status` | ❌ 未注册 | `ServiceStatusSnapshot` | ❌ | ❌ 0 引用 | 同上（需先注册命令） |
| 自定义规则 CRUD | `custom_rule_crud.rs:22/36/77/116/139` | ✅ `lib.rs:297-301` | `CustomRule` | ✅ | ❌ 仅类型定义 | 规则页新增"自定义规则"Tab |
| 运行时规则表 | `grpc_bridge.rs` `get_rules` | ✅ | `RuleList` | ❌ | ✅ `RulesView.vue` | 已完成（只读） |
| 弃用告警 | `client.rs:256` | ✅ `lib.rs:259` | `DeprecatedWarnings` | ❌ | ✅ `AppUpgradeNotices.vue:80` | 已完成 |
| 主页 Clash 切换 | `mode.rs:28` | ✅ `lib.rs:268` | `String` | — | ✅ `useHomeDashboard.ts:96` | 已完成 |

### 1.2 配置生成调用图（实测）

```
【路径 A：生成】
  system/config_service.rs:108 ────────┐
  subscription_service.rs:622/680 ─────┴→ config_generator::generate_config_with_nodes
                                              └→ generate_base_config(app_config)      [config_generator.rs:28]
                                                    ├─ 构造 SingBoxConfig（typed）→ serde_json::to_value
                                                    ├─ apply_app_settings_to_config(&mut config, app_config)  ← ★语义重复
                                                    └─ 注入 services / endpoints（Tailscale）

【路径 B：改已有文件】
  enhanced_storage_service.rs:41 sync_settings_to_config_file
      ├─ Full      → apply_app_settings_to_config(config, app_config)     [settings_patch.rs:30]
      │                └─ apply_profile_settings_if_present(...)          [settings_patch.rs:151]  ← ★启发式归属
      └─ PortsOnly → apply_port_settings_only(config, app_config)        [settings_patch.rs:101]

【路径 C：其他注入】
  proxy_service/custom_rule_crud.rs:9    → config_generator::inject_custom_rules
  system/rule_set_cache_service.rs:16    → config_generator::remote_rule_set_sources
```

### 1.3 双模型的三处具体痛点

1. **同一语义两处实现**
   - 假 DNS：`config_generator.rs` 生成一份；`settings_patch.rs:453/460/493`（`sync_fake_dns_server` / `sync_fake_dns_rules` / `sync_fake_dns_route_rules`）再写一份
   - TUN 路由排除：`config_generator.rs` 一份；`settings_patch.rs:571/583` 一份
   - DNS server / rule-set / http_clients：同样双写
2. **归属靠启发式**：`apply_profile_settings_if_present`（`settings_patch.rs:151-420`，约 270 行）用"配置里有没有某个结构"判断"这是不是本程序生成的配置"。导入的第三方配置一旦长得像就会被误改。
3. **归属无元数据**：我们创建的 inbound tag / rule_set tag / dns server tag 落盘后没有任何标记，后续无法可靠识别"哪些是我们管的"。

---

## 2. 阶段 0：类型基线对齐（0.5 天，A1–A4 的前置）

### 2.1 目标

消除 gRPC 响应类型的前后端漂移风险，并清理一处失效的文档表述。

### 2.2 步骤

**Step 0.1 — Rust 侧加 ts-rs 导出**

文件：`src-tauri/src/app/singbox_api/types.rs`

对本期要用到的类型加 `TS` derive + `#[ts(export, export_to = "../src/types/generated/X.ts")]`：

| Rust 类型 | 导出文件名 | 注意事项 |
|---|---|---|
| `Groups` | `Groups.ts` | 字段 `group: Vec<Group>` |
| `Group` | `Group.ts` | ⚠️ `group_type` 带 `#[serde(rename = "type")]`，ts-rs **不联动** serde 规则，必须加 `#[ts(rename = "type")]`；`is_expand` 因 serde 是 camelCase，需 `#[ts(rename_all = "camelCase")]` |
| `GroupItem` | `GroupItem.ts` | ⚠️ 同上，`item_type` → `type`，`url_test_delay` → `urlTestDelay` |
| `OutboundList` | `OutboundList.ts` | |
| `Rule` / `RuleList` | `Rule.ts` / `RuleList.ts` | 字段已是 snake_case 且无 camelCase rename，ts-rs 默认同名即可 |
| `Service` / `ServiceList` | `Service.ts` / `ServiceList.ts` | `r#type` → ts-rs 导出为 `type` |
| `NetworkQualityResult` | `NetworkQualityResult.ts` | ⚠️ 该结构体**没有** `rename_all`，JSON 是 snake_case（`tcp_rtt_ms`），ts-rs 默认同名，保持一致 |
| `DeprecatedWarnings` / `DeprecatedWarning` | 同名 | `DeprecatedWarning` 有 camelCase，需 `#[ts(rename_all = "camelCase")]` |
| `ClashModeStatus` | `ClashModeStatus.ts` | |

> **最容易踩的坑**：`serde` 的 `rename` / `rename_all` 与 `ts-rs` 的 `rename` / `rename_all` **不是两套独立规则**——ts-rs 11 默认开启 `serde-compat`，会继承 serde 的写法；但 `#[ts(...)]` 优先级更高，且继承行为依赖 feature 开关。所以**每个带 serde rename 的结构体都显式加 `#[ts(rename_all = "camelCase")]`**，让生成的字段名不依赖隐式行为，并在 Step 0.3 用断言锁死「两边写法一致」。

> **实施后订正（2026-10-19）**：本文档原先把 serde 与 ts-rs 的 rename 描述为"两套独立规则"，与 ts-rs 11.1 实际行为不符（`serde-compat` 默认开启，会继承 serde 的 rename）。结论不变——显式写 `#[ts(...)]` 仍是正确做法——但**原因**是"两边写法不一致时以 ts 为准 + 继承依赖 feature 开关"，而非"完全不传导"。`types.rs` 顶部注释、`docs/development.md` 与 `grpc-types.test.ts` 的提示语已同步更正。

**Step 0.2 — 生成并提交**

```bash
cd src-tauri && cargo test ts_rs  # 触发 ts-rs 导出
```

产出落在 `src/types/generated/`（ts-rs 的 `export_to` 相对**crate 根** `src-tauri/`，`../src/types/generated/` 即仓库根的 `src/types/generated/`）。需手动更新 `src/types/generated/index.ts` barrel 导出。

**Step 0.3 — 前端改用生成类型，删除手写 interface**

文件：`src/services/proxy-service.ts:6-32`

删除手写的 `GroupItem` / `ProxyGroup` / `GroupsData` / `ClashModeStatus`，改为：

```ts
import type { Groups } from '@/types/generated/Groups'
import type { OutboundList } from '@/types/generated/OutboundList'
import type { ClashModeStatus } from '@/types/generated/ClashModeStatus'
```

**注意**：`GroupsData.group` 改为 `Groups.group` 后，**所有消费方**（`ProxyStore.ts`、`ProxyView.vue` 等）都要同步改字段名。这是本次唯一的破坏性改动，需 `pnpm type-check` 全量兜底。

**Step 0.4 — 加漂移守护测试**

新增 `src/types/generated/grpc-types.test.ts`（或并入现有 vitest）：

- 断言 `proxy-service.ts` 不再出现手写 `interface GroupItem` / `interface ProxyGroup`
- 断言 `src/types/generated/index.ts` 导出了全部 9 个新类型
- 断言后端 `types.rs` 中**每个**带 `#[serde(rename_all = "camelCase")]` 的结构体**同时**带 `#[ts(rename_all = "camelCase")]`（正则扫描源文件）——把最容易复发的坑变成 CI 门禁

**Step 0.5 — 清理失效镜像与修正文档**

- 删除本地遗留目录 `src-tauri/src/types/generated/`（已被 `.gitignore:33` 忽略，`git ls-files` 返回 0 个跟踪文件；内容是 ts-rs v7 时代 `AlexanderD-r/ts-rs` 头注释的旧产物，会误导开发者改错文件）
- 修正 `CLAUDE.md:23`：把"落在 `src/types/generated/` 与 `src-tauri/src/types/generated/`，两侧都不手改"改为"落在 `src/types/generated/`（唯一事实来源，git 跟踪）；`src-tauri/src/types/generated/` 已被 gitignore，是 ts-rs 本地中间产物，勿依赖"

### 2.3 验收

- `pnpm type-check` / `pnpm lint` / `pnpm test` / `pnpm test:i18n` 全绿
- `cargo test` 通过且 `src/types/generated/` 新增 9 个文件
- `git ls-files src-tauri/src/types/generated` 仍为 0
- 上述漂移守护测试为红→绿

---

## 3. 阶段 1：存量能力接线（2–3 天）

> 全部为前端工作，后端零改动（仅 `probe_service_status` 需补注册命令）。

### 3.1 1-1 出站总览（`get_outbounds`）

| 项 | 内容 |
|---|---|
| 后端 | 无需改动（`lib.rs:258` 已注册） |
| Service | `src/services/proxy-service.ts` 新增 `getOutbounds(): Promise<OutboundList>` |
| Store | `src/stores/kernel/ProxyStore.ts` 新增 `outbounds` state + `refreshOutbounds()` |
| UI | `src/views/ProxyView.vue` 新增"出站总览"折叠面板：tag / type / delay（`urlTestDelay` 毫秒，0 显示"—"） |
| 价值 | "全节点按延迟排序"的数据源；用户可直观看到所有出站而不只是分组内节点 |
| 验收 | 面板在无节点/内核未运行时显示空态而非报错；刷新按钮复用现有 `useCleanup` 退订模式 |

### 3.2 1-2 网络诊断（`network_quality_test`）

| 项 | 内容 |
|---|---|
| 后端 | 无需改动（`lib.rs:260`） |
| Service | `proxy-service.ts` 新增 `networkQualityTest(): Promise<NetworkQualityResult>` |
| UI | `src/views/setting/components/SettingsMaintenanceTab.vue` 新增"网络诊断"卡片：TCP RTT / 下行速度 / 下行延迟 / 上行速度 / 上行延迟（5 字段对应 `NetworkQualityResult`） |
| 交互 | 内核未运行 → 按钮禁用并提示"需先启动内核"；测速中显示 loading；结果附"复制诊断结果"按钮（为后续 C4 诊断模块预留） |
| 验收 | 速度单位统一为 Mbps/Kbps 展示（`download_speed_bps` 为 bps） |

### 3.3 1-3 服务状态（`get_services` + `probe_service_status`）

| 项 | 内容 |
|---|---|
| 后端 | `probe_service_status` **未注册 Tauri 命令** → 需在 `src-tauri/src/lib.rs` 补注册（`grpc_bridge.rs` 中新增薄封装） |
| Service | `proxy-service.ts` 新增 `getServices(): Promise<ServiceList>`、`probeServiceStatus(): Promise<ServiceStatusSnapshot>` |
| UI | 主页或设置页新增"服务"卡片，列出 `services[].name` / `type` / `running` |
| 关联 | `AppConfig.enable_tailscale_endpoint` 开启时，服务卡片高亮 Tailscale 并显示 `tailscale_run_ssh_server` / `tailscale_taildrop_directory` 状态（为 C2 Taildrop 收件箱铺路） |
| 验收 | 内核未运行时显示空态；`running=false` 的服务用灰色 tag 区分 |

### 3.4 1-4 自定义规则 CRUD UI（工作量最大）

| 项 | 内容 |
|---|---|
| Service | 新建 `src/services/custom-rule-service.ts`：`listCustomRules` / `addCustomRule` / `updateCustomRule` / `deleteCustomRule` / `toggleCustomRule`，直接对应 `lib.rs:297-301` 五个命令 |
| Store | 新建 `src/stores/kernel/CustomRuleStore.ts`：rules 列表 + loading + 增删改查方法 |
| UI | `src/views/RulesView.vue` 从单页改为 **Naive UI `n-tabs` 双页签**：<br>① 「运行时规则」= 现有只读 `get_rules` 表格<br>② 「自定义规则」= CRUD 表格 + 新增/编辑抽屉 |
| 表单字段 | `match_type`（`domain` / `domain_suffix` / `domain_keyword` / `ip_cidr`）+ `payload`（多行文本，后端按换行或逗号拆分）+ `action`（`direct` / `proxy` / `block`）+ `note`（可选） |
| 类型 | 直接用 `src/types/generated/CustomRule.ts`（已由 ts-rs 导出，无需新增） |

**必须处理的两个行为差异**（`custom_rule_crud.rs:3-6` 的注释已写明，UI 要显性表达）：

1. 自定义规则走**「写配置文件 + 重启内核」**路径，**不走 gRPC**；与运行时规则（gRPC 实时、不持久化）语义完全不同 → UI 上必须用不同区块/不同图标区分，保存后提示"配置已更新，内核将重启"
2. `inject_custom_rules` **只对本程序生成的配置生效**；对"使用原始配置"的订阅无效 → 自定义规则 Tab 需检测当前订阅是否为"原始配置"模式，非原始配置时给出明确提示

**副作用注意**：`add_custom_rule` 内部会调 `inject_into_active_config` → 触发配置重写。需确认这条路径是否会导致内核重启（`apply_runtime_config_update` 的 `force_restart` 实测为 `false`，见 `enhanced_storage_service.rs:507`），若不重启则需在 UI 上手动提供"重启内核生效"按钮。

### 3.5 阶段 1 验收

- 4 个新 UI 全部走后端命令，**前端零手写响应类型**
- `pnpm lint` / `pnpm type-check` / `pnpm test` / `pnpm test:i18n` / `cargo test` / `cargo clippy` 全绿
- i18n 4 语言 key 对齐（新增文案必须 4 语言同步，`pnpm test:i18n` 把关）
- 新增 vitest：`CustomRuleStore` 的增删改查状态转换 + `proxy-service` 命令参数拼装
- 文档同步（见 §5）

---

## 4. 阶段 2：配置双模型收敛（3–4 天）

> 目标：把"生成"与"补丁"两条路径统一到**同一份配置语义**上，消除双写与启发式归属，为后续「配置覆写层」（差距分析 B2）与「JSON Schema 配置编辑器」（C1）铺路。

### 4.1 目标架构

```
                        AppConfig
                            │
              ConfigIntent::from_app_config()   ← ★唯一把 AppConfig 翻译成配置语义的地方
                            │
                        ConfigIntent（纯值对象，不可变）
                       ┌────┴────┐
                       ↓         ↓
        render_skeleton(intent)   apply_intent_to_config(config, intent, ownership)
              【生成路径】              【补丁路径】
                       │         │
                       └────┬────┘
                            ↓
                          Value
                            │
                    (可选) 用户覆写层 ← 阶段 3 的 B2/C1 挂在这里
```

`Ownership` 记录"这个配置文件里哪些 tag 是本程序创建的"，**随配置一起落库**，替换掉 `apply_profile_settings_if_present` 的结构猜测。

### 4.2 分步实施（7 步，每步一个独立 commit）

#### Step 2.0 黄金样本测试（**必须最先做**）

文件：`src-tauri/src/app/singbox/config_generator.tests.rs`

把当前 `generate_base_config()` 在 4 组 `AppConfig` 下的输出固化为 JSON 快照：

- `AppConfig::default()`
- `tun_enabled = true` + 自定义 `tun_stack` / `tun_mtu`
- `singbox_fake_dns_enabled = true` + `singbox_dns_optimistic_cache = true`
- `allow_lan_access = true` + 非默认端口

快照存 `src-tauri/src/app/singbox/testdata/config_golden/*.json`，测试做**逐字段 diff**（不是整串相等，容忍 key 顺序差异）。

**为什么必须先做**：Step 2.3 要删掉生成路径里的 `apply_app_settings_to_config` 调用。没有黄金样本就无法证明"生成 + 补丁"与"纯生成"产出的配置等价。

#### Step 2.1 新增 `ConfigIntent`

新文件：`src-tauri/src/app/singbox/config_intent.rs`

```rust
/// 配置意图：AppConfig → 配置语义 的唯一映射产物。
/// 纯值对象，不含 IO，不含 serde_json::Value。
pub struct ConfigIntent {
    pub mixed_inbound: InboundIntent,   // tag / listen / port
    pub tun_inbound: Option<TunIntent>,  // enabled / ipv4 / ipv6 / mtu / stack / route_exclude
    pub api_service: ApiServiceIntent,   // listen / port / dashboard
    pub dns: DnsIntent,                  // final / optimistic / timeout / servers / fakeip
    pub route: RouteIntent,              // final / auto_detect / default_http_client
    pub http_clients: Vec<HttpClientIntent>,
    pub rule_sets: Vec<RuleSetIntent>,
    pub groups: Vec<GroupIntent>,
    pub clash_mode_rules: bool,
}

impl ConfigIntent {
    pub fn from_app_config(cfg: &AppConfig) -> Self { /* 纯函数 */ }
}
```

**本步是纯搬迁**：把 `config_generator.rs` + `settings_patch.rs` 中读取 `AppConfig` 的部分抽到这里，**不改任何行为**。验收：`cargo test` 全绿 + 黄金样本不变。

#### Step 2.2 抽出 `render_skeleton`

新目录：`src-tauri/src/app/singbox/config_render/`

- `mod.rs`：`render_skeleton(intent: &ConfigIntent) -> Value`
- `dns.rs`：DNS server / rules / fakeip 渲染
- `tun.rs`：TUN inbound 渲染
- `outbounds.rs`：urltest / selector / 分组渲染
- `route.rs`：rule_set / rules / http_clients 渲染
- `services.rs`：api service / tailscale endpoint 渲染

把 `generate_base_config` 的函数体整体搬进 `render_skeleton`，**逻辑一行不改**，只把 `app_config.xxx` 换成 `intent.xxx`。

#### Step 2.3 生成路径切换（**消除双写的关键一步**）

```rust
// config_generator.rs
pub fn generate_base_config(app_config: &AppConfig) -> Value {
    let intent = ConfigIntent::from_app_config(app_config);
    render_skeleton(&intent)
}
```

**删除** `render_skeleton` 内部对 `apply_app_settings_to_config` 的调用——`ConfigIntent` 已经是完整的配置语义，补丁路径要做的事生成路径本来就不该再做。

**验收**：黄金样本 diff **必须为空**。若不为空，说明 `apply_app_settings_to_config` 里做了生成路径之外的事（例如 `ensure_api_service`、`ensure_sniff_route_rule`），需把这些逻辑并入 `render_skeleton` 后再重试。

#### Step 2.4 补丁路径改为按归属打补丁

文件：`src-tauri/src/app/singbox/settings_patch.rs`（建议重命名为 `config_patch.rs`）

```rust
pub fn apply_intent_to_config(config: &mut Value, intent: &ConfigIntent, ownership: &Ownership) {
    // 只改 ownership 声明属于我们的 tag
}
```

- `apply_app_settings_to_config(config, app_config)` 保留为 **deprecated 薄封装**，内部转 `apply_intent_to_config(config, &ConfigIntent::from_app_config(app_config), &Ownership::inferred(config))`
- `apply_port_settings_only` 逻辑不变（最小侵入路径，天然不需要 ownership）

#### Step 2.5 引入 `Ownership` 显式归属

- 新增 `src-tauri/src/app/singbox/ownership.rs`：

```rust
pub struct Ownership {
    pub inbounds: Vec<String>,   // 我们创建的 inbound tag
    pub rule_sets: Vec<String>,
    pub dns_servers: Vec<String>,
    pub groups: Vec<String>,
}
```

- 生成时（`render_skeleton` / `generate_config_with_nodes`）把本次创建的 tag 集合**写进 `generic_config`**（key 建议 `managed_config_tags`，与 `custom_rule.rs` 的 `STORAGE_KEY` 同机制）
- 补丁时（`apply_runtime_config_update` → `sync_settings_to_config_file`）先读 ownership，再决定改哪些字段

#### Step 2.6 清理启发式归属

删除 `apply_profile_settings_if_present`（`settings_patch.rs:151-420`，约 270 行）中基于结构猜测的分支，改为查 `Ownership`。

**保留 fallback**：读不到 ownership（老库 / 用户手改过的配置）时，**退化为 `PortsOnly`**（只改端口与 listen 地址），而不是继续猜测。

**配套新增回归测试**：
- 对一份"用户导入的原始配置"（不含本程序特征）执行 Full patch 后，`inbounds` / `dns` / `route.rules` **必须逐字段不变**
- 对一份本程序生成的配置执行 Full patch，行为与重构前一致

#### Step 2.7 迁移残留函数

以下从 `settings_patch.rs` 迁到 `config_render/` 下对应模块，生成/补丁**共用同一实现**：

| 函数 | 当前位置 | 目标 |
|---|---|---|
| `sync_fake_dns_server` | `settings_patch.rs:453` | `config_render/dns.rs` |
| `sync_fake_dns_rules` | `settings_patch.rs:460` | `config_render/dns.rs` |
| `sync_fake_dns_route_rules` | `settings_patch.rs:493` | `config_render/dns.rs` |
| `is_fake_dns_route_rule` | `settings_patch.rs:513` | `config_render/dns.rs` |
| `resolve_tun_route_exclude_address_for_patch` | `settings_patch.rs:571` | `config_render/tun.rs` |
| `extract_existing_tun_route_exclude_address` | `settings_patch.rs:583` | `config_render/tun.rs` |

同时把 `settings_patch.rs` 中 `apply_profile_settings_if_present` 里的 DNS / rule-set / http_clients 分支统一改为调用 `config_render` 的共享函数。

> **实施后订正（2026-10-19）**：阶段 2 收敛做到的是「**同一份 intent 的两条路径产物逐字段一致**」
> （新增往返断言锁死），但**没有**补齐"设置变更 → 已有配置"这条方向。
> 复核发现 mDNS / Tailscale endpoint / 下载出站三个设置在补丁路径上不落地——
> 该缺口在 HEAD 就存在（`settings_patch.rs` 对 `endpoints`/`tailscale`/`mdns` 零匹配），
> 不是收敛引入的，但同属"设置项在两条路径上不一致"这一类。
> 已登记为 [`gap-analysis-vs-mature-projects.md` 的 D7](./gap-analysis-vs-mature-projects.md)，待后续单独处理。

### 4.3 阶段 2 验收

| 项 | 标准 |
|---|---|
| 单元测试 | `cargo test` 全绿；黄金样本 diff **为空** |
| 行为等价 | `settings_patch.tests.rs` 现有 8 个用例**不修改断言**且全部通过 |
| 原始配置保护 | 新增用例：用户原始配置经 Full patch 后 `inbounds`/`dns`/`route.rules` 逐字段不变 |
| 类型安全 | `cargo clippy --all-targets -- -D warnings` 通过；`cargo fmt --check` 通过 |
| 接口稳定 | **87 个 Tauri 命令签名零变更** → 前端无需同步改动，回退不产生跨端不一致 |
| 实机验证 | Windows x64 实机：启停内核成功、`sing-box check` 通过、订阅切换后配置正确、TUN 启停正常、端口改动生效 |
| 文档 | 根 `AGENTS.md` / `src-tauri/AGENTS.md` / `src-tauri/src/app/AGENTS.md` 的 STRUCTURE 与 CODE MAP 同步 |

### 4.4 风险矩阵

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 生成配置字段漂移导致内核启动失败 | 中 | **高** | Step 2.0 黄金样本 diff + 实机 `sing-box check` + 每步独立 commit 可单独 revert |
| 误改用户导入的原始配置 | 中 | **高** | Step 2.6 ownership 查表 + fallback 退化为 PortsOnly + 新增"原始配置不被改写"回归测试 |
| 订阅用户配置"变形"（多字段/顺序变化） | 中 | 中 | 黄金样本逐字段对比；序列化保持 `to_string_pretty` |
| ownership 落库与配置实际内容不同步（用户手改配置） | 中 | 中 | 补丁前校验 ownership 声明的 tag 仍存在，不存在则按"外来修改"处理并跳过该字段 |
| 阶段 1 与阶段 2 改动冲突 | 低 | 低 | 阶段 1 只动前端 + `types.rs`；阶段 2 只动 `app/singbox/`；两者文件集不重叠 |
| 过度抽象导致可读性下降 | 中 | 中 | `ConfigIntent` 保持**纯数据**（无行为），行为全部留在 `config_render/` 的纯函数里 |

### 4.5 回退

- 阶段 2 拆成 7 个独立 commit，**任意一步可单独 `git revert`**
- 全程**不改 Tauri 命令签名**，前端零感知 → 回退不需要前后端同步发版
- Step 2.1 / 2.2 / 2.3 是纯搬迁 + 一次调用点替换，行为由黄金样本锁死

---

## 5. 文档同步清单（对照根 `AGENTS.md` 的 DOC SYNC GATE）

| 阶段 | 必须同步的文档 |
|---|---|
| 阶段 0 | `CLAUDE.md`（修正"两侧 types/generated"表述）；根 `AGENTS.md` NOTES（类型导出口径） |
| 阶段 1 | 根 `AGENTS.md` WHERE TO LOOK（新增 UI 落点）；`src/AGENTS.md`（新 service/store）；`docs/CHANGELOG.md` `[未发布]` 段（用户可见功能） |
| 阶段 2 | 根 `AGENTS.md` CODE MAP + WHERE TO LOOK；`src-tauri/AGENTS.md`；`src-tauri/src/app/AGENTS.md`（新模块 `config_intent` / `config_render` / `ownership`）；`docs/development.md`（若启动链或模块职责变化） |
| 收尾 | `docs/gap-analysis-vs-mature-projects.md` 标记 A1–A5 / D1 已完成 |

---

## 6. 排期

| 阶段 | 内容 | 工作量 | 依赖 | 可独立交付 |
|---|---|---|---|---|
| 0 | 类型基线（ts-rs 导出 + 清理镜像 + 守护测试） | 0.5 天 | — | ✅ |
| 1-1 | `get_outbounds` → 出站总览 | 0.5 天 | 0 | ✅ |
| 1-2 | `network_quality_test` → 网络诊断 | 0.5 天 | 0 | ✅ |
| 1-3 | `get_services` → 服务卡片（+ 注册 `probe_service_status`） | 0.5–1 天 | 0 | ✅ |
| 1-4 | 自定义规则 CRUD UI | 1–1.5 天 | 0 | ✅ |
| 2-0 | 黄金样本测试 | 0.5 天 | — | ✅ |
| 2-1～2-7 | 双模型收敛 | 3–4 天 | 2-0 | 分 7 步 |
| — | 合计 | **约 7–8 人日** | | |

建议节奏：**阶段 0 + 阶段 1 作为一个 PR**（用户可感知的功能补齐，低风险）；**阶段 2 单独一个 PR**（纯后端重构，需实机验证）；阶段 3（覆写层 / Schema 编辑器）在阶段 2 之后另起。

---

## 7. 实施时容易踩的坑（备忘）

1. **serde 的 rename 会被 ts-rs 继承，但 `#[ts(...)]` 优先级更高**——ts-rs 11 默认开启 `serde-compat`；两边写法不一致时最终以 `#[ts]` 为准，所以显式写 `#[ts(rename_all = ...)]`（不依赖隐式继承）。
2. **ts-rs 的 `export_to` 相对 crate 根**（`src-tauri/`），`../src/types/generated/` 指的是**仓库根**的 `src/types/generated/`，不是 `src-tauri/src/types/generated/`。后者是 gitignore 的本地遗留物。
3. **字段名从 `GroupsData.group` 变成 `Groups.group`** 是阶段 0 唯一的破坏性改动，需 `pnpm type-check` 全量兜底，别漏改模板里的深层访问。
4. **自定义规则走文件+重启，不走 gRPC**，与运行时规则语义不同，UI 必须区分（`custom_rule_crud.rs:3-6` 已写明）。
5. **不要往 `route.rules` 写 `clash_mode: rule` 的 catch-all 规则**——历史实现硬编码 `"proxy"` 导致内核校验失败无法启动（`mode.rs:24-27`）。
6. **Step 2.3 删除 `apply_app_settings_to_config` 调用后，黄金样本必须仍为空**；不为空说明生成路径原本依赖补丁路径的副作用，需先把副作用并入 `render_skeleton`。
7. **ownership 缺失时退化为 PortsOnly**，而不是继续猜结构——猜错会改坏用户导入的原始配置。
