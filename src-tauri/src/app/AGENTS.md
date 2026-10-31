# BACKEND APP LAYER KNOWLEDGE BASE

## OVERVIEW

后端业务分层主域：`core/network/system/storage/singbox/constants`，是命令实际落地位置。

## STRUCTURE

```
src-tauri/src/app/
├── core/          # 内核启停、代理模式、gRPC 查询、事件中继
├── network/       # 订阅下载、解析、模式切换、自动更新
├── system/        # 更新（自研）、权限、系统状态、后台任务、启动恢复
├── storage/       # SQLite 服务与状态模型（含 custom_rule）
├── singbox/       # 配置语义与渲染（config_intent / config_render / config_generator / settings_patch / ownership）
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
| sing-box 配置生成 | `singbox/config_generator.rs`            | 规则/outbounds/DNS 组装，1.14 mDNS 注入 |
| 配置语义（唯一来源）| `singbox/config_intent.rs`             | `AppConfig` → 配置语义的纯映射；生成与补丁两条路径共用 |
| 配置渲染共享层     | `singbox/config_render.rs`              | 入口 / 假 DNS server / 规则等两条路径共用的纯函数 |
| 配置补丁（按归属） | `singbox/settings_patch.rs`             | 在已有配置上按 tag 打补丁；外来配置只改端口 |
| 配置归属判定       | `singbox/ownership.rs`                  | 记录本程序创建的 tag；落库 `managed_config_tags:<路径>` |
| gRPC API 交互     | `singbox_api/` + `core/proxy_service.rs` | 替代 experimental.clash_api |
| 自定义规则 CRUD   | `core/proxy_service.rs` + `storage/custom_rule.rs` | 5 个命令，存 `generic_config` 表 key=`custom_rules` |
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
- DNS 1.14：mDNS server 在 `singbox/config_render.rs` 渲染；假 DNS 规则采用官方标准写法（把 A/AAAA 查询**路由**到 `dns_fakeip` server），**不用** `action: evaluate` + `tag`——后者只打标记不做路由，没有配套 `match_response` 规则时不会生效。
- ⚠️ **`http_clients` 的直连客户端不能写 `detour`**：sing-box 1.14+ 的 `common/dialer/detour.go` 对「detour 指向空 direct 出站」直接判定为无意义并让内核**启动失败**（`detour to an empty direct outbound makes no sense`）。直连必须表达成**不写 `detour`**（`http_client` 不写 detour 时走系统 dialer，语义同样是直连）。统一由 `singbox/config_render.rs::build_http_clients` 渲染，生成路径与补丁路径共用；补丁路径还会纠正存量配置里写错的 `detour: "direct"`。改下载出站相关逻辑时务必守住这条。
- **配置双模型已收敛**：`AppConfig` → `ConfigIntent`（`config_intent.rs`）是配置语义的唯一映射点；生成路径（`config_generator::render_skeleton`）与补丁路径（`settings_patch::apply_intent_to_config`）共用 `config_render.rs` 的纯函数，**生成路径不再回调补丁路径**。改动配置语义时两处都要看，回归靠 `singbox/config_golden.tests.rs` 的黄金样本。
- **配置归属（Ownership）**：补丁前先判定"这份配置是不是本程序的"（`singbox/ownership.rs`）。判定依据是 `mixed-in` 入口 / 本程序自定义的 DNS server tag，**不靠通用分组名猜测**；判定为外来配置时只同步端口（`PortsOnly`），不重建入口、不改写路由终点——改坏用户导入的原始配置不会报错，只会表现为"能连但走错线路"。归属记录**只在认领到至少一个 tag 时才落库**（空壳记录会让后续补丁拿空清单过滤，导致 DNS 地址 / 测速 URL 静默不生效）。
- ⚠️ **已知缺口（D7，已部分修复）**：`singbox_dns_use_mdns` / `enable_tailscale_endpoint` 两项设置在**补丁路径上仍不落地**——生成路径会写 `dns_mdns` / `endpoints`，补丁路径对这两处零处理，所以改了设置已有配置不会变。原第三项 `singbox_download_detour` **已修复**：补丁路径新增 `sync_http_clients` + `sync_rule_set_http_client_refs`（见 `settings_patch.rs`），会按当前偏好重写 `http_clients` 并清理悬空的 `http_client` 引用。详见 `docs/gap-analysis-vs-mature-projects.md` 的 D7。
- 自定义规则模型（`CustomRule`/`CustomRuleAction`/`CustomRuleMatchType`）经 ts-rs 导出到仓库根 `src/types/generated/`（唯一事实来源，git 跟踪）；`src-tauri/src/types/generated/` 是 gitignore 的本地中间产物，勿依赖勿修改。改字段后需重新生成并同步前端类型。
- 新增/修改本层命令后，按根 AGENTS.md 的 DOC SYNC GATE 同步文档与 `docs/CHANGELOG.md`。
