# Sing-Box 1.14 升级与进一步增强路线图

**状态**：🟡 **规划中**（前置：gRPC API 迁移已完成 — 见 `docs/sing-box-api-migration.md`）

**编写日期**：2026-09-01

**适用版本**：本项目当前嵌入内核 1.13.15；目标对齐 sing-box 1.14.0（2026-08-31 发布）。

---

## 0. TL;DR

| 维度 | 当前 | 目标 |
|---|---|---|
| 嵌入内核 | 1.13.15 | 1.14.0（oldstable 轨保留 1.13.21） |
| gRPC 官方 API | ✅ 已完成（11 接口，4 流式） | 补全 `GetOutbounds` / `GetRules` / `GetServices` 等 |
| 1.14 关键特性使用率 | 0% | ≥ 60% |
| Deprecated 字段残留 | 4 处（`clash_api` / `store_rdrc` / `independent_cache` / `download_detour`） | 全部迁移 |
| Release Track | stable / prerelease / autobuild | 增加 `oldstable` / `beta`（内核下载） |
| 订阅协议覆盖 | 10 种 | 补 `snell` / `wireguard` / `tailscale` / `hysteria2` 新字段 |

总工期估算 **7 周**，分 5 阶段。

---

## 1. P0 — 必须先做（兼容性 / 升级 / 安全）

### 1.1 内核升级 1.13.15 → 1.14.0

**背景**：当前 `src-tauri/resources/kernel/windows/amd64/version.txt` 仍是 1.13.15，缺失 1.13.16-1.13.21 全部安全与稳定性修复；1.14 关键能力（Hysteria2 抗指纹、Snell、Tailscale SSH/Taildrop、TLS spoof、pre-match sniff、DNS 优化）全部吃不上。

**操作**：

```powershell
# 拉取所有平台/架构的 1.14.0 内核二进制
node scripts/fetch-kernel.mjs --all
```

**风险**：

- 1.14 默认开启 TUN `dns_mode=hijack`，会**修改系统 DNS**（Linux 改 `systemd-resolved`，Windows 改 per-interface DNS + WFP，macOS 改 `Network.framework`）。
- 老用户首次升级会看到 DNS 行为变化，需要在 `SettingsAdvancedTab.vue` 加"接管系统 DNS"开关，**默认开启但弹窗告知**。
- 配置 schema 收紧，部分 1.13 配置文件可能在 1.14 启动失败——`versioning.rs::check_config_validity` 已有，需保证启动前必跑。

**任务**：

- [ ] `scripts/fetch-kernel.mjs` 增加 `--track` 选项（默认 stable）
- [ ] `versioning.rs::get_kernel_releases` 透传 track 过滤
- [ ] `AppConfig` 增加 `installed_kernel_version_track: String` 字段
- [ ] `SettingsKernelTab.vue` 加"更新通道"下拉

### 1.2 配置生成清理 4 个 1.14-deprecated 字段

1.14 标记、1.16 计划删除：

| 字段 | 当前位置 | 1.16 后的替代 |
|---|---|---|
| `experimental.clash_api` | `src-tauri/src/app/singbox/config_generator.rs`、`settings_patch.rs:42-53` | 顶层 `services: [api]`（**已就绪**） |
| `experimental.cache_file.store_rdrc` | `config_generator.rs:240-243`、`settings_patch.rs:58-62` | `cache_file.store_dns` |
| `dns.independent_cache: true` | `config_generator.rs:248` | 删除（1.14 强制按 transport 缓存） |
| `rule_set[].download_detour` | `config_generator.rs:421`、`settings_patch.rs:268-276` | 顶层 `http_clients` + `route.default_http_client` |

**操作**：

- [ ] `config_generator.rs`：删 `experimental.clash_api` 整段（行 38-64），删除 `independent_cache` 字段
- [ ] `settings_patch.rs:42-63`：同上
- [ ] `config_generator.rs:421` 改写 `RemoteRuleSetConfig` —— 不写 `download_detour`，改写顶层 `http_clients`
- [ ] 1.14 启动日志加 `GetDeprecatedWarnings` 消费（gRPC 已就绪），启动后弹窗提示用户
- [ ] `SettingsAdvancedTab.vue` 加"我已了解 1.16 移除计划"提示卡

### 1.3 Release Track 切换（Linux/Docker 已 4 轨，桌面客户端应对齐）

1.14 之后 sing-box 在 Linux / Docker 已有 4 个发布轨：

| 轨 | 含义 | 适用 |
|---|---|---|
| `stable` / `latest` | 当前稳定版 | 99% 用户 |
| `beta` / `latest-beta` | 稳定预发版 | 早期测试 |
| `testing` | testing 分支 | 开发者 |
| `oldstable` / `latest-oldstable` | 上一稳定版 | 兼容性回退 |

**操作**：

- [ ] `fetch-kernel.mjs`：加 `--track stable|beta|testing|oldstable` 参数
- [ ] `versioning.rs::get_latest_kernel_version` 透传 track
- [ ] `kernel-targets.mjs`：URL 模板支持 track（oldstable 走 `releases?per_page=200` 找 1.13.latest）
- [ ] `AppConfig` 新增 `kernel_update_track` 字段
- [ ] `SettingsKernelTab.vue`：UI 暴露 + 默认值

---

## 2. P1 — 显著提升体验（用上 1.14 重要新能力）

### 2.1 业务分流组配置重构：rule-set `tag` 数组 + `{tag}` placeholder

**当前**（`config_generator.rs:57-89`、`:194-202`）：

- 5 个 selector（`TAG_TELEGRAM` 等）写死
- 5 个 rule_set 各自独立
- 5 条 route 规则各自独立

**1.14 改进**：用 `rule-set.tag: [list]` + `{tag}` placeholder 一次声明。

```jsonc
// 改造前（5 个独立 rule_set）
{ "tag": "geosite-telegram", "type": "remote", "url": "...telegram.srs", "download_detour": "direct" },
{ "tag": "geosite-youtube", "type": "remote", "url": "...youtube.srs",  "download_detour": "direct" },
// ... 3 more

// 改造后（1 个 rule-set，多 tag）
{
  "type": "remote",
  "tag": ["geosite-telegram", "geosite-youtube", "geosite-netflix", "geosite-openai", "geosite-google"],
  "url": "https://.../geosite-{tag}.srs",
  "update_interval": "7d"
}
```

**收益**：配置体量 ↓ ~40%，rule-set 解析错误定位更简单，前端业务分流组"开/关"改 1 个开关。

**任务**：

- [ ] `config_schema.rs::RemoteRuleSetConfig` 改 `tag: Vec<String>` 或保持 `String` + 文档化"逗号分隔"
- [ ] `config_generator.rs:114-161` 5 条 rule_set 合并
- [ ] `settings_patch.rs:268-296` 同步逻辑改数组
- [ ] 回归：5 个 selector 仍能分别测速/切换

### 2.2 DNS 升级 4 件套

**a. Optimistic DNS Cache**（`config_generator.rs:245-251`）

```jsonc
"dns": {
  "servers": [...],
  "optimistic": true,    // 1.14 新增
  "final": "proxy-dns"
}
```

效果：重复查询命中过期缓存立即返回 + 后台刷新，尾延迟显著降低。

**b. mDNS Server + `preferred_by`**

```jsonc
"servers": [
  { "tag": "mdns-local", "type": "mdns" },              // 1.14 新增
  { "tag": "local", "type": "local" }                  // 已有
],
"rules": [
  { "rule_set": "geosite-local", "server": "mdns-local" },     // *.local 走 mDNS
  { "preferred_by": ["local", "dhcp"], "action": "reject" }   // 1.14 新增
]
```

**c. DNS `evaluate` + `match_response`** —— 替代 deprecated `ip_cidr` / `ip_is_private` 旧 DNS 字段：

```jsonc
"rules": [
  { "action": "evaluate", "server": "fakeip", "tag": "fakeip-result" },
  { "match_response": "fakeip-result", "ip_cidr": "198.18.0.0/15", "action": "reject" }
]
```

**d. `dns.timeout`** —— per-query 超时（1.14 新增）

**任务**：

- [ ] `config_generator.rs` 注入 `optimistic: true`
- [ ] `AppConfig` 加 `dns_use_optimistic_cache: bool`（默认 true）
- [ ] `AppConfig` 加 `dns_use_mdns: bool`（默认 true）
- [ ] `AppConfig` 加 `dns_timeout: String`（默认 "5s"）
- [ ] `fake_dns_filter_mode` 逻辑迁到 `evaluate` + `match_response`
- [ ] `SettingsAdvancedTab.vue` 暴露以上开关

### 2.3 TUN 模式新选项（v1.14 默认接管平台 DNS）

| 字段 | 1.14 默认 | 影响 | 任务 |
|---|---|---|---|
| `tun.dns_mode` | `"hijack"` | 改系统 DNS | AppConfig 加开关，默认弹窗 |
| `tun.dns_address` | 平台原生 | 系统级 DNS 注入 | 暴露给设置页 |
| `tun.include_mac_address` | `[]` | 仅代理指定网卡 | UI 暴露 |
| `tun.exclude_mac_address` | `[]` | 排除指定网卡 | UI 暴露 |
| `tun.exclude_mptcp` | `true` | 拒绝 MPTCP | 已是 1.13 默认，确认 |
| `tun.stack` | `"mixed"` | gvisor/system/mixed | UI 已有（`proxy-service.ts:50` 类型） |

**操作**：

- [ ] `AppConfig` 加 `tun_dns_mode: String`、`tun_dns_address: String`、`tun_include_mac: Vec<String>`、`tun_exclude_mac: Vec<String>`
- [ ] `settings_patch.rs::apply_inbounds_settings` 注入以上
- [ ] `SettingsAdvancedTab.vue` TUN 折叠组加表单
- [ ] 升级弹窗文案："1.14 默认接管系统 DNS，可在高级设置中关闭"

### 2.4 Hysteria2 抗指纹（v1.14 默认开启 Chrome QUIC Parrot）

v1.14 Hysteria2 客户端默认伪装成 Chrome 的 QUIC 握手，但：

- 服务器使用 Ed25519 证书时会失败（Chrome 不声明 Ed25519）
- 用户需要 `disable_chrome_parrot: true` 才能连上

**1.14 Hysteria2 新增字段**：

- `bbr_profile`（BBR 拥塞控制）
- `hop_interval_max`（跳跃间隔随机化）
- `obfs.type: "gecko"`（新一代混淆，替代 salamander）
- `obfs.min_packet_size` / `obfs.max_packet_size`

**任务**：

- [ ] 订阅解析器 `parser.rs::is_supported_outbound_type` 已在；补 hysteria2 字段映射
- [ ] `AppConfig` 加 `hysteria2_disable_chrome_parrot: bool`、`hysteria2_obfs_type: String`（"salamander" | "gecko"）、`hysteria2_bbr_profile: String`
- [ ] UI 在 Hysteria2 节点详情页暴露以上

### 2.5 订阅解析器（`parser.rs`）补 1.14 协议

当前支持：`vless / vmess / trojan / shadowsocks / shadowsocksr / socks / http / hysteria2 / tuic / anytls`

需要补：

| 协议 | 类型 | 任务 |
|---|---|---|
| `snell` | outbound | 1.14 新增，机场增量协议 |
| `wireguard` | endpoint | 1.13 已有，parser 漏识别 |
| `tailscale` | endpoint | 1.14 加 `ssh_server`、`taildrop_directory`、`listen_port` |
| `hysteria2` 增强 | outbound | `bbr_profile`、`hop_interval_max`、`realm`、`disable_chrome_parrot` |
| `vless` 增强 | outbound | `flow`、`packet_encoding`、`reality` 精细映射 |
| `tuic` 增强 | outbound | 1.14 统一 HTTP/2+QUIC 参数 |

**任务**：

- [ ] `is_supported_outbound_type` 加 `"snell"`, `"wireguard"`, `"tailscale"`
- [ ] `parser.rs` 写 `parse_snell_uri` / `parse_tailscale_endpoint` / hysteria2 字段映射
- [ ] 补充单元测试（项目已有 `*.tests.rs` 模式）

### 2.6 规则 / 协议增强（次优先级）

- **HTTP/2 + QUIC 参数统一**：1.14 顶层 `http_clients` 共享
- **certificate provider 体系**：`acme` / `cloudflare-origin-ca` / `tailscale` provider
- **`cloudflared` inbound**：无需 Cloudflare 账号的快速隧道
- **`tls.spoof` / `tls.spoof_method`**：SNI 诱骗，**仅 Windows x64/x86 + Admin**（ARM64 不支持）
- **bridge outbound + L3 转发**：1.14 新能力，对 TUN 用户很关键

---

## 3. P2 — 体验增强（gRPC API 深度集成）

### 3.1 gRPC 客户端补全未实现方法

`src-tauri/src/app/singbox_api/client.rs:138-292` 当前只暴露 8 个方法。1.14 daemon 还支持：

| 方法 | 用途 | 项目价值 |
|---|---|---|
| `GetOutbounds` | 列出所有 outbounds（含 delay 排序） | 主页"按延迟排序所有节点" |
| `GetGroup` | 按 tag 取单组 | 单组刷新无需整组 |
| `GetRules` | 路由规则表 | 规则管理 UI（恢复 `RulesView.vue`） |
| `GetServices` | 当前运行的服务列表 | 状态页展示 Tailscale/USB/IP |
| `NetworkQualityTest` | 网络质量 | 设置页加"网络诊断" |
| `STUN` | 测 NAT | Tailscale / 端口可达性 |
| `Tailscale*` 系列 | 状态 / 节点 / 操作 | Tailscale 模式 UI（见 3.3） |

**任务**：

- [ ] 用 `prost-build` 重新生成 proto（当前 `proto.rs` 是手写）
- [ ] `client.rs` 补 7 个方法
- [ ] `lib.rs` 注册对应 Tauri command
- [ ] 前端 `proxy-service.ts` 补方法

### 3.2 JSON Schema 集成（1.14 文档第 29 条）

v1.14 内核发布 `sing-box schema` 命令输出 JSON Schema。

**操作**：

- [ ] 启动时调 `sing-box schema > resources/schema.json`（首次）
- [ ] `SettingsAdvancedTab.vue` 加"在编辑器中编辑高级配置"入口
- [ ] 用 Monaco Editor / CodeMirror 加载 schema，**编辑器自带补全 + 错误下划线**
- [ ] 保存后调 `versioning.rs::check_config_validity`（已有）二次校验

### 3.3 Tailscale 模式 UI（1.14 desktop 客户端核心卖点）

v1.14 desktop 客户端核心新增 Tailscale 模式。本项目作为 Tauri 客户端应对齐：

- [ ] `AppConfig` 加 `enable_tailscale: bool`
- [ ] 开启后 `config_generator` 注入 `endpoints: [{ type: "tailscale", ... }]`
- [ ] UI 展示 Tailscale 状态（在线节点 / SSH 入口 / **Taildrop 收件箱**）
- [ ] 收件箱：读 `taildrop_directory` 目录文件列表，前端展示
- [ ] `kernelService` 增加 `getServices` / `tailscale operations` 透传到 gRPC

### 3.4 连接页按"规则 / 出站"分组（已有数据未消费）

`ConnectionStore.ts` 用了 `connectionsData` payload，但 UI 只按 IP 排序。1.14 Connection 已经带 `rule`、`outbound`、`outbound_type` 字段（见 `singbox_api/types.rs:111-138`）。

**任务**：

- [ ] `ConnectionsView.vue` 加"按规则分组" / "按出站分组"折叠面板
- [ ] `TrafficStore` 按"规则"切片可视化

### 3.5 主页 Clash Mode 快捷切换

- 已有 gRPC `getClashModeStatus` / `setClashMode`（`lib.rs:242-243`）
- 主页加 3 选 1 切换条：**Rule（默认）/ Global / Direct**
- 启动时持久化到 `AppConfig.clash_mode`，注入 `route.rules` 的 `clash_mode: global/direct`（`config_generator.rs:185-188` 已有基础结构）

### 3.6 Web Dashboard 集成（1.14 官方）

1.14 官方提供 `sing-box-dashboard` Web UI。`api` service 配 `dashboard: { enabled: true }` 直接挂在 `listen_port` 上（`config_generator.rs:283` 当前显式关闭）。

**操作**：

- [ ] `AppConfig` 加 `enable_web_dashboard: bool`（默认 false）
- [ ] 开启后内置 webview 标签页指向 `http://127.0.0.1:<api_port>/`
- [ ] 安全：必须保持 `listen: 127.0.0.1`（已 OK）

### 3.7 隐藏 UI 恢复（`RulesView.vue` / `ProxyProvidersView`）

`docs/sing-box-api-migration.md` 记录规则/代理提供者 UI 在迁移阶段被隐藏。1.14 gRPC 支持 `GetRules` / `GetOutbounds` 后可逐步恢复：

- [ ] `RulesView.vue` 恢复：调 `GetRules` 展示路由规则表，支持禁用/启用
- [ ] `ProxyProvidersView` 恢复：调 `GetOutbounds` 展示所有 outbounds

---

## 4. P3 — 健壮性 / 跨平台

### 4.1 gRPC-Web 实现细节加固

`client.rs` 几处可改进：

- **`parse_response_headers`**（`client.rs:326-339`）：用 `body_stream.next()` 一次性把整个 body 读进 buffer 才解析，**对 server-streaming 大流量（SubscribeLog）会 OOM**。建议：边读边 parse（同 `HttpStream::new` 的方式）。
- **`SubscribeConnections` 间隔**：当前固定传 1s。1.14 `interval_nanos: 0` 表示"事件驱动"（仅变化时推送），能省 ~90% gRPC 流量。
- **`get_groups_snapshot`**（`client.rs:130-137`）：有 `unary("GetVersion", &[])` 占位 + 注释，**功能上等于先调 GetVersion 拿空响应再忽略**——遗留，删掉。

### 4.2 启动恢复 / Service 化

1.14 启动要做的事变多：连接 Tailscale endpoint、下载 rule-set、ACME 注册、DERP 初始化。`startup_restore_service` 当前只是"找到 active config 路径就完事"。

- [ ] 引入 `rule_set.initial_path`（1.14 新增）：rule-set 本地兜底，避免冷启动卡 30s
- [ ] 升级后 TUN/Service 启动加超时与重试
- [ ] `kernel_auto_manage::kill_existing_processes` 路径在 1.14 也要保留（避免孤儿进程），但要注意 Tailscale endpoint 启动后是 `sing-box.exe` 的子进程，**不能 kill parent**

### 4.3 跨平台差异

- **Windows ARM64**：不支持 `tls.spoof`（1.14 文档说明），UI 隐藏
- **macOS SFM (App Store)**：1.14 迁移到新开发者账号 + `sing-box MT`，本项目独立分发（`update_service.rs:11` 是 `xinggaoya/sing-box-windows`），不受影响
- **Linux AppImage / DEB / RPM**：1.14 后有 4 轨，建议 `UpdateChannel` 加 `Oldstable` / `Testing`

### 4.4 性能 / 资源

- `ConnectionStore.ts:65-66` 的 `connectionsUnlisten` / `memoryUnlisten` 在 `cleanupEventListeners` 调了但没 await `Promise.resolve`，若 store 频繁创建可能泄漏
- `KernelRuntimeStore` 复用了同一组事件，**两个 store 同时订阅同一事件源**——需要做共享
- `versioning.rs::check_kernel_version` 缓存了版本号到 SQLite，但如果用户手动换了 `kernel/` 目录里的 exe，缓存不会失效。建议增加 exe 文件 hash 校验

---

## 5. 实施路线

| 阶段 | 周期 | 内容 | 风险 |
|---|---|---|---|
| **A. 兼容性收尾** | 1 周 | 内核升 1.14.0 + 4 个 deprecated 配置迁移 + ARM64 检测 | 中（要回滚兼容方案） |
| **B. 1.14 体验增强** | 2 周 | Release Track + DNS 4 件套 + 订阅协议补全 + Hysteria2 抗指纹 | 低（纯加法） |
| **C. gRPC 客户端补全** | 1 周 | 用 protoc 重新生成 proto + 补 GetOutbounds/GetRules/GetServices/NetworkQuality | 低 |
| **D. Tailscale / Dashboard / 高级 UI** | 2 周 | Tailscale 模式 + Web Dashboard 入口 + JSON Schema 编辑器 | 中（涉及 Tailscale auth 流程） |
| **E. 健壮性** | 1 周 | gRPC 流式解析优化 + 启动恢复改造 + 跨平台差异处理 | 低 |

**总计约 7 周可以吃满 1.14 全部能力，并提前适配 1.16 的 breaking change。**

---

## 6. 立即可做的"小赢"（半天内）

1. 跑 `node scripts/fetch-kernel.mjs --all` 升内核到 1.14.0
2. `singbox_api/client.rs:130-137` 删掉 `get_groups_snapshot` 里那段 `unary("GetVersion", &[])` 占位代码
3. `SettingsAdvancedTab.vue` 加"TUN 接管系统 DNS"开关（`dns_mode`）
4. `docs/CHANGELOG.md` 加 `Unreleased` 条目记录本路线图启动

---

## 7. 关联文档

- `docs/sing-box-api-migration.md` — 已完成的 gRPC API 迁移（前置）
- `docs/CHANGELOG.md` — 变更日志
- `docs/development.md` — 开发者文档
- `docs/release-playbook.md` — 发布流程
- `src-tauri/examples/grpc_smoke.rs` — gRPC 实机联调示例
- `src-tauri/src/app/singbox_api/` — gRPC 客户端模块

---

## 8. 风险与回退

| 风险 | 缓解 |
|---|---|
| 1.14 改平台 DNS 导致用户网络异常 | 升级弹窗 + 设置页"接管系统 DNS"开关，默认开 |
| 1.16 强制移除 deprecated 字段 | 在 1.14 阶段就完成迁移，留 1+ minor 缓冲 |
| Tailscale endpoint 启动失败影响主流程 | Tailscale 模式独立开关，失败回退到普通 mode |
| gRPC 客户端 OOM（SubscribeLog） | 边读边 parse + max buffer 限制 |
| Windows ARM64 TLS spoof 不可用 | 平台检测，UI 自动隐藏 |
| macOS 独立版用户数据迁移 | README 加提示，文档化迁移步骤 |
| 跨平台 4 轨 release 复杂度 | `oldstable` 默认隐藏，`beta/testing` 默认只在高级设置可见 |
