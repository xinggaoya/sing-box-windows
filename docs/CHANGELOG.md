# 更新日志

## [未发布]

### ✨ 新功能

- **新增「出站」页面（出站总览）**：展示内核里所有出站（含分组）的快照，按延迟排序，未测速显示 `—`，支持按名称/类型搜索。此前 `get_outbounds` 后端早已就绪但前端 0 引用，属于"能力已具备、界面没接线"。**独立成页面（侧边栏「出站」入口，路由 `/outbounds`），完全不占用代理页空间**
- **维护页新增「网络诊断」卡片**：5 项网络质量指标（`network_quality_test`）+ sing-box 1.14 服务状态列表（`get_services`）；并补齐后端 `probe_service_status` 命令用于逐个探测服务可用性
- **规则页新增「自定义规则」面板**：完整的增 / 删 / 改 / 查 / 启停，走「写配置文件 + 重启内核」路径（与运行时规则语义不同，界面已明确区分）
- **规则页运行时规则改为只读展示**：改用 sing-box 1.14 官方 gRPC `get_rules`，不再依赖已移除的 Clash API

### 🐛 问题修复

- **修复代理页被「出站总览」挤扁、主体几乎不可见**：代理页主体是「定高 + 内部滚动」布局（`.page-shell` 为 flex 列，`.split-view` 靠 `flex: 1` 吃掉剩余高度），而上一版把有内容高度的「出站总览」作为 `.page-shell` 的**同层子元素**放进了纵向流——flex 分配高度时 `.split-view`（`min-height: 0`）被压到近乎 0 高度，代理组侧栏与节点网格被整块挤出可视区。
  - 「出站总览」改为**独立页面**（`/outbounds`，侧边栏「出站」入口），代理页恢复到本次提交前的样子（零占用）
  - `.split-view` 加 320px 保底高度，防止同类回归再次压扁代理区
  - 新增 `src/views/proxy-layout.test.ts` 布局守护（代理页不得出现出站总览 UI / 出站总览必须有独立页面与路由 / `.split-view` 必须有 min-height 保底）与 `src/composables/useOutboundList.test.ts`（排序、过滤、统计）
- 修复后台运行一段时间后代理静默失效（需手动重启内核）：hysteria2 等 QUIC 隧道静默死亡（UDP NAT 超时/服务端断开）后，内核进程存活但所有代理流量黑洞。守护的连通性检测此前不走代理（直连探测永远成功），自愈永不触发；现改为通过内核 mixed 端口探测代理链路，并新增直连对照排除系统断网
- 修复自愈机制对"节点本身不可用"场景的处理：原实现一律重启内核，节点坏时会陷入无限重启循环。现改为分级自愈——先关闭全部连接（软恢复，清空黑洞隧道）→ 仍失败且手动选中的是具体节点时自动切换到"自动选择"（发通知告知）→ 最后才重启内核，且连续 2 次重启无效后熔断（暂停重启、提示用户切换节点/检查订阅，轻量自愈保留，链路恢复后自动解除）

### 🐛 稳定性与并发修复

- **修复 SQLite 并发写风险**：连接此前用 `SqlitePool::connect` 裸连（rollback journal + `busy_timeout=0`），而自动保存、内核守护回写自愈策略、订阅切换等多条路径会并发写，存在 `SQLITE_BUSY` 失败。现改为 WAL + `busy_timeout=5s` + 外键约束 + 连接池上限
- **修复数据库初始化失败后后台任务永久空转**：`background_tasks` 等待存储就绪时是无超时循环，DB 初始化失败后会每秒打一条 warn 且永不停止（日志磁盘无限增长）。现改为 30 秒上限并返回错误
- **修复「第二个数据库静默不建表」**：建表使用进程级 `OnceCell`，任何非首个数据库路径（测试库、未来多库）都不会建表。现改为按库幂等执行
- **修复 schema 迁移静默失败不可观测**：60 条 `ALTER TABLE` 的错误被 `.ok()` 直接丢弃，迁移失败在日志里完全无迹可循，线上仅表现为「某项设置不生效」且无法自查。现改为显式查列后补列、失败时携带完整 SQL 上抛，并新增 `PRAGMA user_version` 记录 schema 版本
- **修复 Tauri 事件监听被误杀**：`eventService.removeEventListener(name)`（不带 id）会清空该事件下**所有**监听器，而三个 kernel store 都是「先用自己的 unlisten 精确移除、再顺手调一次全局清理」，导致同事件的第二个订阅者被无提示误杀。现拆分为按 id 的精确退订与显式的 `removeAllListenersForEvent()`
- **修复流量监听器泄漏**：`TrafficStore` 丢弃了 `onTrafficData` 返回的 unlisten，只能靠「清空整事件」退订，现已持有引用
- **修复主进程错误处理器失效**：`main.ts` 的 `errorHandler` 写在 `mount()` 之后，挂载阶段抛出的错误根本捕获不到（白屏时无任何线索）
- 修复 `kernel-service` 的 `stopKernel({ force })` 为幽灵参数：在前端 3 层透传但被 service 丢弃，后端 `kernel_stop_enhanced` 也不接收，调用方会误以为传参生效
- 修复 `UpdateModal` 绕过 `eventService` 直接 `listen`，导致监听器无法被统一清理
- 修复 22 处日志中的 emoji 被替换成字面 `?`（编码损坏）
- 修复 `update-progress` 事件 `message` 字段缺失时前端会读到 `undefined`
- **统一内核存活判定**：此前「内核是否在跑」有三套彼此独立的实现且结论会互相矛盾 ——
  `is_kernel_running` 看进程句柄、`kernel_get_status_enhanced_v2` 看 TCP 端口、
  `KERNEL_STATE.readiness.process_alive` 谁写谁说了算。表现为「内核明明在跑、流量也正常，
  前端却显示已停止」。现收敛为 `kernel_service::health` 唯一入口（端口 > 托管句柄 > 平台扫描），
  端口探测抽象为 trait 后可在无真实内核的环境下完整覆盖优先级规则

### ♻️ 重构

- **配置双模型收敛**（阶段 2）：此前 sing-box 配置有两条独立路径——`config_generator` 从零生成、`settings_patch` 在已有配置上打补丁——两者读同一批 `AppConfig` 字段、写同一批语义，**加一个设置项要改两处，漏改不报错**，只会让"生成的配置"与"补丁后的配置"悄悄分叉。
  - `ConfigIntent`（`app/singbox/config_intent.rs`）：`AppConfig` → 配置语义的**唯一**纯映射
  - `config_render`（`app/singbox/config_render.rs`）：入口构建、假 DNS server/规则等两条路径**共用**的纯函数
  - 生成路径**不再回调**补丁路径（`render_skeleton` 不再调用 `apply_app_settings_to_config`）
  - **配置归属（Ownership）**：补丁前先判定"这份配置是不是本程序的"（依据 `mixed-in` 入口 / 本程序自定义的 DNS server tag，**不靠通用分组名猜测**）。判定为外来配置时只同步端口，**不重建入口、不改写路由终点**——此前靠结构猜测，猜错就会把用户导入的机场原始配置改坏，且不报错、只表现为"能连但走错线路"
  - 归属随配置落库（`generic_config` 的 `managed_config_tags:<路径>`），老库/手改过的配置回退为按内容推断
  - 全程不改 Tauri 命令签名，前端零感知
- **跨语言类型统一到 ts-rs**（阶段 0）：`src/types/generated/` 成为**唯一事实来源**（git 跟踪），删除前端手写的后端结构体 interface（`GroupItem`/`ProxyGroup`/`GroupsData`/`ClashModeStatus` 等）；`src-tauri/src/types/generated/` 是 gitignore 的本地中间产物，勿依赖
- **TUN 切换状态机收敛**：「置位 → 应用代理配置 → 重启内核 → 失败回滚」此前在首页与托盘各有 3 处变体（回滚顺序、重试次数各不相同），现统一为 `useTunToggle`，平台提权与 sudo 密码交互通过回调注入
- **内核压缩包解压统一**：下载与导入两条路径各有约 170 行几乎相同的解压代码，且行为已分叉（`.tgz` / `.tar` 只在导入路径可用）。现合并为 `archive.rs`，格式支持取并集，阻塞 IO 全部走 `spawn_blocking`（此前解压大包会阻塞 Tauri 异步运行时）
- **应用配置改为 schema 驱动**：53 个配置字段此前在 7 处手工枚举（store 声明 / 持久化入参 / store return / 接口 / load / save / watch），加一个设置项要改 7 个地方且漏改无任何报错。现由 `appConfigSchema.ts` 单一声明，load/save/watch 自动生成
- **HTTP 客户端按档位复用**：11 处各自 `reqwest::Client::builder()` 收敛为 `ClientProfile`；其中守护循环每 8 秒重建一个含 TLS 根证书的客户端的问题一并修复
- **应用配置内存快照**：`db_get_app_config` 全仓 58 处调用点每次都是一次 SQLite 往返加全列映射（守护循环每 8 秒至少读 2 次），现改为写时更新快照、读时命中
- **移除 `mitt` 事件总线**：所有使用点都是把 Tauri 事件再转发一遍的冗余链路（4 跳缩到 2 跳），message 实例改为直连 store
- **国际化统一**：清除全部 `locale.startsWith('zh') ? '中文' : 'English'` 三元、`t(key, '中文兜底')` inline default 以及 service/store 层硬编码中文消息。语言包从 597 提升到 665 个 key，补齐日/俄语译文（日/俄用户此前在首页会看到裸 key）
- **日志页与连接页的排序/分组逻辑**合并为 `useSortableGroupedList`
- 删除后端死代码约 500 行（`src/error.rs`、`process/error.rs`、gRPC 迁移后遗留的 `event_relay.rs`、未注册的 `kernel_get_status_enhanced` / `kernel_get_snapshot`、未使用的 `KERNEL_READY_NOTIFY` / `ProcessStatus` / `ProcessConfig`）
- 删除前端死代码约 1100 行（5 个 0 引用组件、3 个 0 引用 composable、2 个 0 引用 service、0 引用的类型文件、失效的 `types/index.d.ts` barrel、`page-layout.css` 中 143 行从未生效的 hero/stat 规则）
- **超大视图拆分**：HomeView / SubView / ProxyView 合计 3059 → 1490 行。
  - 三个视图的 `<style scoped>`（935 行）迁到 `src/assets/views/*.css`，
    用 `<style scoped>@import ...</style>` 而非 script import ——
    这样外部文件里的规则仍经过 SFC 的 scoped 变换，隔离语义与内联完全一致（已验证构建产物）
  - HomeView：运行环境探测（平台/管理员权限）与节点模式（Clash mode）+ 一键测速
    → `useHomeDashboard`
  - SubView：三种来源（URL / 手动 JSON / 手动 URI）的表单状态、校验、提交
    → `useSubscriptionForm`
  - ProxyView：节点名 → 国家码 → 国旗的三层解析 → `useNodeFlag`
- 5 处用 `setTimeout(100ms)` 赌 Vue 调度时机的 store 初始化守卫，改为确定性的 `flushWatchers()`
- 统一三份重复的时间格式化实现，并修正其隐性问题：原先用 `toLocaleString()` 不传 locale，
  跟随的是浏览器环境语言而非应用内选定语言
- **`proxy_service` 按职责拆分**：原单文件 942 行同时承担 inbound 写入 / OS 代理 / gRPC 桥接 /
  状态查询 / 自定义规则 CRUD，现拆为 `inbound_writer` / `grpc_bridge` / `status` /
  `custom_rule_crud`，聚合文件保持既有调用路径
- **`tray/service` 按职责拆分**：原单文件 926 行承担文案 / 菜单 / 窗口 / 动作 / 运行态同步，
  现拆为 5 个子模块（最大 281 行）
- 消除 `SettingsAdvancedTab` 的 i18n 双机制（同时 `useI18n()` 与 prop 传 `t`）
- **国家码改为生成 + 测试守护**：原先 258 行国家码硬编码在 `ProxyView.vue` 且已与 `flag-icons`
  漂移（包多出 ARAB / ASEAN / GB-ENG 等 16 个码，视图多出包已移除的 AC / EA / TA，导致国旗
  静默不显示）。现由 `scripts/gen-country-codes.mjs` 生成，`pnpm test:country-codes` 与
  15 项单元测试双重守护

### 🐛 修复

- **26 个 i18n key 层级错置，界面直接显示裸 key**（如代理页显示 `proxy.tabGroups`、
  `proxy.badgeFavorites`，通知项显示 `notification.ipv4Preferred`，设置页显示
  `setting.advanced.clashDirect`）。根因是这些 key 被放在语言包**根层**，而组件用
  `t('proxy.xxx')` 引用——4 个语言包结构一致，所以"跨语言对齐检查"查不出来。
  现已按实际引用层级归位（`proxy` / `notification` / `home` / `setting.advanced` /
  `common` / `rules`），并补齐 4 个语言包均缺失的 `common.minimizedToTray`
- **i18n 切换不生效、直接显示 key 原文**（用户报告：代理页显示 `proxy.badgeRecommended`、
  `PROXY.TABGROUPS` 而非文案）。根因有两处：
  1. **`LocaleStore` 从未把生效语言写入 vue-i18n 实例**。`createI18n` 建好后
     `locale` 停在初始的 `zh-CN`，而 store 与 service 层大量使用 `i18n.global.t(...)`
     （`kernel-service`、`KernelStore`、`TrayStore` 等），它们只读实例上的 locale。
     组件内 `useI18n()` 看起来正常，掩盖了实例层从未更新的事实。
     现由 `LocaleStore` 在 `currentLocale` 上 `watch` 并立即应用到实例，
     切换语言后 store/service 的提示同步生效，首帧也不会再缓存 key 原文。
  2. **`useTunToggle` 的 `messages` 固化了 `t()` 结果**。调用方在 setup/store 初始化时
     把 `t('...')` 求值一次存进普通对象，语言切换后不会被重新求值 —— TUN 切换失败的
     提示会停留在旧语言。改为支持 getter（字面量仍向后兼容），`HomeView` 与
     `TrayStore` 改用 `() => t(...)`。

### 🧪 测试与门禁

- **配置生成黄金样本**（阶段 2）：把 4 组代表性 `AppConfig`（默认 / TUN / FakeDNS / 局域网）的生成结果固化为 JSON 快照（`app/singbox/testdata/config_golden/`），改配置生成逻辑时必须**逐字段不变**，否则说明配置语义漂移；`SINGBOX_BLESS_GOLDEN=1` 可重新生成
- **双模型收敛断言**：新增「生成路径的产物再走一遍补丁路径必须逐字段不变」用例——收敛前补丁路径会重排 `dns.servers`、改写假 DNS 规则形态、覆盖 `route.final`，导致"订阅刚刷新的配置"与"改一次设置后的配置"不是一回事
- **i18n 引用 key 存在性门禁**：`scripts/check-i18n.mjs` 新增扫描 `src/**/*.{ts,vue}` 中静态 `t('a.b')` 调用并校验 key 在基准语言包中存在（当前 532 个引用 key / 0 缺失）——正是这道门禁能提前发现上面那 26 个裸 key
- **跨语言类型漂移守护**：新增 `src/types/generated/grpc-types.test.ts`，守护 ts-rs 导出路径、serde/ts rename 同步、生成类型不含 `bigint`、前端无手写 interface、barrel 导出齐全
- 前端测试基建（vitest + @vue/test-utils + jsdom）与 **165 项**测试，覆盖 TUN 切换回滚路径、排序分组、内核错误归因、事件精确退订、配置 schema 对称性、国旗码同步、订阅表单三来源、节点名→国旗解析、**locale 同步到 vue-i18n 实例**、**注入文案的语言切换**等此前零覆盖且易回归的逻辑
- 后端新增 **34 项**测试（198 → 232）：数据库全列往返与迁移幂等、多库各自建表、压缩包四种格式解压、配置内存快照一致性、内核存活判定优先级、**配置生成黄金样本**、**生成/补丁双模型收敛断言**、**配置归属判定**、**外来配置不被改写**
- CI 新增 4 道门禁：`cargo fmt --check`、`prettier --check`、前端测试、i18n key 对齐校验；并统一了全库格式（此前 97 个文件未按 prettier 格式化，格式化工具形同虚设）
- 测试过程中发现并修复 **3 个回归**：TUN 状态机重构时丢失的「索要 sudo 密码后重新执行应用与重启」、配置 schema 中空数组无法清空 `tun_include_macs`、**国旗 URL 大小写回归**（重构时误加 `.toUpperCase()`，而 `flag-icons` 文件名与 `import.meta.glob` 的 key 恒为小写 —— 症状是「节点都识别出来了但一个国旗都不显示」且无任何报错）

## [v2.3.2] - 2026-09-08

### 🐛 问题修复

- 修复 Windows on ARM 设备误下载 amd64 内核：架构检测改为读取宿主原生架构（`PROCESSOR_ARCHITEW6432` / `PROCESSOR_ARCHITECTURE`），x64 构建经模拟层运行时也能自动下载 arm64 原生内核（#68）
- 修复 Windows ARM64 上 TLS spoof 开关误显示：架构标识统一为 `arm64` 后前端门禁未同步，现同时兼容 `arm64` / `aarch64` 新旧标识
- 修复 Linux 32 位 ARM 设备"关于"页架构显示名回落为原始标识的问题（恢复显示 "Linux ARM"）
- 修复 `tun_dns_mode` 设置未注入内核 TUN 配置的问题：该字段此前仅持久化，配置生成与运行时切换两条路径现均写入 `tun.dns_mode`（1.14 起默认 hijack 接管系统 DNS，设置页可切换 native / disabled）
- 修复开启"IPv6 优先"后 TUN 模式 DNS 报错（`dial tcp6: cannot assign requested address`）：DNS 服务器自身域名的 bootstrap 解析不再跟随全局 IPv6 偏好，保持 IPv4 优先
- 修复关闭主窗口后托盘代理开关状态陈旧（#67）
- 修复订阅切换后代理节点列表不更新
- 修复 gRPC `URLTest` 测速完成后延迟不刷新
- 修复内核状态运行期不刷新
- 修复 Hysteria2 订阅节点字段丢失（#65）
- 修复 vless / vmess / trojan 订阅节点 TLS 字段透传

### ✨ 新功能

- 升级 sing-box 内核到 1.14.0
- 迁移到 sing-box 官方 gRPC API（`type: api`）
- 新增 gRPC 方法：`get_rules` / `get_services` / `get_outbounds` / `get_deprecated_warnings` / `network_quality_test`
- 设置页新增"TUN DNS 接管模式"选择（hijack / native / disabled），1.14 升级后可一键关闭系统 DNS 接管
- 首次升级弹出 TUN DNS 接管告知；内核启动后自动检测配置弃用字段并弹窗提示（1.16 移除计划预热）
- 设置页新增 1.16 弃用字段移除计划提示卡
- rule-set 本地缓存预热（1.14 `initial_path`）：后台下载 `<数据目录>/rule-sets/<tag>.srs` 副本，离线冷启动不再被首次下载阻塞
- 新增 Tailscale endpoint 实验性支持
- 新增 Web Dashboard 集成
- 恢复 RulesView 规则管理 UI
- `fetch-kernel.mjs` 内核下载支持 Release Track（stable / oldstable / beta / testing）
- 设置页内核版本下拉恢复可选：下载指定版本前强制弹窗确认（应用基于 1.14+ gRPC API，自行切换旧版内核可能出现兼容问题），确认后正常下载；默认仍为最新版本

### 🔧 优化改进

- 清理 4 个 deprecated 字段（`clash_api` / `store_rdrc` / `independent_cache` / `download_detour`）
- 远程 rule-set 下载完全迁移到 1.14 原生写法：顶层 `http_clients`（detour 跟随下载出站偏好）+ `route.default_http_client`，生成配置不再写 `download_detour`，不会再触发内核弃用告警
- `SubscribeConnections` 改为事件驱动（interval 0），仅在连接变化时推送，大幅减少空闲流量；连接页时长显示改用本地心跳
- Windows ARM64 平台自动隐藏 TLS spoof 开关（内核 WinDivert 驱动不支持）
- 远程 rule-set 清单收敛为单一来源（`remote_rule_set_sources`），配置生成与缓存预热共用
- 订阅解析器补 1.14 协议与字段（snell / wireguard / tailscale / hysteria2 新字段）
- gRPC-Web 流式解析优化
- 配置生成层注入 12 个 1.14 字段
- AppConfig 持久化 15 个 1.14 字段
- `proxy_service.rs` 大幅精简（1240 → 407 行）
- 事件中继 4 WebSocket → 4 server-streaming
- 代理模式改写 `route.rules` 最高优先级 `clash_mode` 规则
- 删除 metacubexd 兼容层 + 旧 `RulesStore` / `rule-service`
- 设置页新增"高级（1.14 实验性）"折叠组

## [v2.3.1] - 2026-06-17

### 🐛 问题修复

- **修复发布版本启动白屏** - 将 naive-ui 及其强依赖（vueuc、css-render、vooks、treemate 等）合并到同一打包 chunk，修复 Rollup 跨 chunk 循环引用导致的 `Cannot access 'ac' before initialization` 运行时错误

### ✨ 新功能

- **首页显示本地代理端口（#50）** - 首页新增本地代理端口卡片，直接展示浏览器插件常用的 HTTP / SOCKS5 地址；当前 sing-box 使用 mixed 入站，因此两种协议共用同一个代理端口，并可从首页快速打开端口设置
- **Release 增加 Intel macOS 产物（#51）** - 预发布流水线新增 `x86_64-apple-darwin` 构建，后续 Release 会同时上传 Apple Silicon 与 Intel Mac 安装包

### 🔧 优化改进

- **设置页侧边栏导航重构** - 将原有分段 Tab 导航替换为左侧图标导航栏 + 右侧内容面板的现代布局，通用/内核/网络/更新备份/关于 五大分区一目了然，切换时带平滑过渡动画；内核有新版本或应用有可用更新时，对应导航项会显示提示圆点；移动端自动切换为水平图标导航栏
- **通用设置常用项优先** - 主题模式、强调色、紧凑模式等高频设置移至页面顶部，采用自定义主题切换按钮组与色彩圆点选择器替代原有 Naive UI radio/picker；启动设置（开机自启等）移至底部，子选项按条件展开
- **内核管理卡片化** - 内核版本状态以醒目 badge 展示（绿色已安装/红色未安装），下载进度独立区域，子操作按钮更紧凑
- **高级设置可折叠分组** - 网络、代理高级、sing-box Profile 三个大组改为手风琴式可折叠布局，默认仅展开最常用的网络组，降低信息密度
- **更新与备份视觉统一** - 更新状态卡片采用渐变背景突出新版本信息；备份操作按钮水平排列更紧凑
- **关于页品牌化** - 顶部应用身份区域带渐变背景与应用图标，版本信息列表带颜色编码的分类图标
- **首页状态栏视觉增强** - 顶部状态改为独立 hero 卡片，带颜色编码指示灯与 glow 效果；各分区标题增加颜色编码小图标；节点模式选择使用 radio dot 替代旧指示器
- **端口设置弹窗复用** - 首页与设置页共用同一个端口设置弹窗，统一端口范围校验、端口冲突拦截与运行态应用逻辑，避免两处保存行为漂移
- **macOS 发布产物区分架构** - macOS `.dmg` 与 `.app.tar.gz` 上传前会追加架构后缀，避免 arm64 与 x86_64 产物在 Release 中同名覆盖

## [v2.3.0] - 2026-04-22

### ✨ 新功能

- **订阅 URI 新增 TUIC / AnyTLS 支持（#47）** - 订阅编辑页的“节点链接 / YAML”模式新增 `tuic://` 与 `anytls://` 解析能力；手动粘贴 URI 列表和从订阅正文提取节点时，都会按 sing-box 官方 outbound 结构生成对应节点

### 🔧 优化改进

- **设置页分组重构** - 设置页改为分段 Tab 导航，将基础偏好、内核管理、网络与高级配置、更新备份、关于信息按使用场景归类；同时拆分原有大页面为多个设置组件，降低单页信息密度并保留原有弹窗与运行态保存行为
- **URI 解析字段适配官方文档** - `tuic` 节点补齐 `congestion_control`、`udp_relay_mode`、`udp_over_stream`、`zero_rtt_handshake`、`heartbeat`、`network` 等常用参数映射；`anytls` 节点补齐 `idle_session_check_interval`、`idle_session_timeout`、`min_idle_session`，并统一写入标准 `tls.server_name` / `tls.insecure` / `tls.alpn`
- **订阅入口提示同步更新** - 多语言文案中的 URI 支持说明补充 `tuic://`、`anytls://`，避免界面仍显示旧的协议支持范围
- **TUN 排除路由地址前端配置（#49）** - 设置页新增 `route_exclude_address` 多行 CIDR 输入，保存前会先拦截明显无效的条目；留空时不写入该字段，继续使用后端/内核默认行为
- **启动诊断链路结构化** - 内核状态快照新增 `startup_diagnosis` 与 `readiness`，保留原有兼容字段的同时，把启动前校验、启动后探针、自动管理与守护失败统一到同一套诊断结构里，避免继续只看到笼统的“启动失败”
- **首页/日志页失败排查入口补齐** - 首页会直接展示最近一次启动诊断摘要与建议动作；日志页也会补充独立的启动诊断区块，即使 relay 日志尚未建立，也能查看最近一次失败详情
- **启动状态同步口径统一** - `kernel-error` 与 `kernel-status-changed` 现在分别承载结构化失败诊断与 readiness 信息，前端改由 `KernelStore` 统一接管这两类状态，减少首页、托盘和日志页之间的提示漂移

### 🐛 问题修复

- **GUI 拉起内核长时间运行断网修复** - 修复应用启动 sing-box 时 `stderr` 被管道化但未持续读取，日志输出累积后可能阻塞内核进程的问题；同时将内核日志固定写入工作目录 `sing-box.log`、修复事件中继提前结束，避免手动运行稳定但 GUI 拉起一段时间后连接失效
- **自启动托盘图标状态回填修复** - 开机自启并隐藏到托盘时，后端会在启动恢复与内核自动管理后主动刷新托盘运行态；系统代理/TUN 已开启时无需先打开主界面，托盘图标和菜单状态即可立即正确着色与回显
- **内核启动失败提示可读性修复（#42）** - `kernel-error` 事件现在会附带结构化启动诊断，`KernelStore` 统一接管失败事实；托盘与首页不再只能依赖 `lastError` 字符串，`/version` 400、配置缺失/无效、权限问题等常见失败路径会保留更明确的分类和上下文
- **启动重试残留旧错误修复（#42）** - 新一次启动会重置上一轮失败诊断，成功拉起后也会及时清空旧错误；像 relay 启动失败、前台自动管理重试、sudo/权限类启动失败这类边缘路径，现在也会尽量同步到统一诊断状态，避免页面继续挂着过期失败信息

## [v2.2.9] - 2026-04-04

### ✨ 新功能

- **开机自启动托盘隐藏开关** - 设置页启动设置新增“开机后自动隐藏到托盘”选项，可独立控制系统自启动时是否直接进入托盘

### 🔧 优化改进

- **自启动启动参数链路重构** - 自启动参数从固定 `--hide` 改为统一 `--autostart`，并在应用启动早期根据本地启动偏好决定是否隐藏窗口，避免后端把托盘行为写死
- **启动前轻量配置落盘** - 新增独立的启动偏好文件，持久化开机自启、开机隐藏到托盘和关闭窗口行为，保证系统登录拉起时可以在数据库初始化前完成窗口模式决策
- **自启动托盘体验无闪屏** - 当启用“开机后自动隐藏到托盘”时，主窗口会在最早安全时机直接隐藏，不再依赖前端初始化后再切换，减少启动闪现

### 🐛 问题修复

- **sing-box 1.13 入站字段迁移修复** - 清理配置生成与运行时 patch 链路里残留的 legacy inbound 字段，不再输出 `inbounds[].sniff` / `sniff_override_destination`；同时统一保证 `route.rules` 中存在 `action: "sniff"`，修复升级到 sing-box 1.13+ 后因 legacy inbound fields 被移除而导致的启动失败
- **原始订阅配置覆盖修复（#44）** - 新增、切换、刷新、回滚和启动恢复订阅时，会先同步最新订阅元数据，再按当前订阅真实模式决定配置 patch 策略；勾选“使用原始配置”后将稳定保持“仅替换端口”，不再被高级设置误判后写回 `sniff`、DNS 等字段而导致自定义 sing-box 配置被覆盖
- **节点代理模式状态回显修复（替代 #45）** - 首页启动后会主动从当前生效配置与运行中的 Clash API 同步节点代理模式，不再固定显示 `rule`；切换 `global/rule` 时会优先即时同步运行态，若内核暂未接入则明确提示“已保存、重启后生效”，避免 UI、`active_config_path` 与实际运行模式长期漂移
- **轻量模式与开机自启联动修复** - 系统自启动且自动隐藏到托盘时，现会正确继承“轻量模式（销毁窗口）”策略，不再始终退化为简单隐藏窗口
- **自启动窗口显示控制修复** - 关闭“开机后自动隐藏到托盘”后，即使开启系统自启动，应用也会在登录后正常显示主窗口，不再被固定参数强制隐藏
- **轻量模式托盘控制失效修复** - 托盘中的“系统代理 / TUN / 重启内核”等控制型动作改为后端直接执行，主窗口销毁后不再因前端监听器缺失而失效
- **Windows 托盘开启 TUN 回退修复** - 轻量模式下从托盘开启 TUN 且当前不是管理员时，会先恢复主窗口并回退到原有“弹确认后以管理员重启”的前端流程，避免无界面提权链路失效导致无法真正进入 TUN

## [v2.2.8] - 2026-03-22

### ✨ 新功能

- **局域网访问开关** - 设置页新增“允许局域网访问”选项，开启后代理入站会监听 `0.0.0.0`，局域网内设备可通过当前机器 IP 使用代理
- **托盘轻量模式** - 新增“关闭窗口行为”设置，可选择继续隐藏到托盘，或在关闭时销毁主窗口，仅保留托盘与后台任务，以显著降低后台 `WebView` 内存占用

### 🔧 优化改进

- **控制面维持本机访问** - 暴露局域网代理时，Clash API 控制端口仍保持绑定 `127.0.0.1`，避免将管理接口一并暴露到局域网
- **运行态监听地址链路补齐** - 持久化、配置 patch、内核启动运行态与订阅辅助链路统一读取 `allow_lan_access`，避免启动后监听地址被重置
- **托盘模式识别增强** - 托盘图标改为复用原始应用图标并按代理模式动态着色：系统代理显示红色、TUN 显示紫色、手动模式保持默认图标，切换模式时会即时刷新，便于快速识别当前状态
- **托盘代理开关改为双独立控制** - 托盘菜单改为分别控制“系统代理”和“TUN”，移除单独的手动模式入口，并在提示文案中展示组合状态
- **轻量模式恢复链路完善** - 托盘恢复主窗口时，后端会按 `main` 窗口配置重建 `WebviewWindow` 并补发待恢复路由，避免重建后的首屏状态丢失
- **轻量托盘入口语义优化** - 进入托盘轻量空白页后，托盘菜单首项会从“显示主界面”切换为“重建窗口”，更贴合当前窗口被隐藏并等待恢复的状态

### 🐛 问题修复

- **托盘切换残留运行态修复** - 从托盘关闭 TUN 时会补齐运行态应用与内核重启链路，避免切换后旧的 TUN 仍继续生效
- **VLESS Reality 链接解析补齐** - 订阅/URI 导入 `security=reality` 的 VLESS 节点时，现会正确解析 `pbk`、`sid` 与 `fp`，生成 `tls.reality` 配置并保留指定的 uTLS 指纹，避免因缺失 Reality 握手参数导致连接失败
- **订阅流量信息兼容增强** - 订阅下载首次请求缺少 `subscription-userinfo` 响应头时，会自动使用兼容客户端 `User-Agent` 重试补取流量和到期时间；保留首次请求拿到的订阅正文，避免因机场按 UA 分流导致节点内容被意外替换
- **原生关闭事件统一托盘策略** - `Alt+F4`、系统关闭按钮与自定义标题栏关闭按钮统一走托盘策略，避免轻量模式只对自定义关闭按钮生效
- **轻量模式退出回归修复** - 销毁主窗口后会显式阻止“最后一个窗口关闭即退出应用”的默认行为，确保托盘常驻；托盘“退出”仍能正常停止内核并彻底退出

## [v2.2.7] - 2026-03-17

### ✨ 主要更新

- **Fedora / RPM 支持补齐** - Linux 打包新增 `rpm` 产物，本地与 Release CI 都可直接生成 `x86_64` RPM 安装包
- **Linux 安装包选择更准确** - 更新检查会根据发行版偏好选择 `rpm` / `deb` / `AppImage`，Fedora / RHEL 系优先返回 RPM
- **Linux TUN 关闭链路修复** - TUN 模式下若内核以 `sudo/root` 身份启动，停止/快速重启会在普通权限清理失败后自动回退到 sudo 清理，避免出现“能启动但无法关闭”的问题

### 🔧 优化改进

- **Tauri 打包别名修正** - `scripts/tauri-wrapper.mjs` 的 Linux / macOS 预设别名改为使用 Tauri 2 正确的 `--bundles` 参数，避免 bundle 参数误透传给 cargo
- **发布流水线补齐 RPM** - Release 工作流增加 Linux RPM 依赖安装与产物上传，预发布页面会一并附带 `.rpm`
- **本机构建入口新增 RPM** - 新增 `pnpm run tauri build:linux:rpm`，便于 Fedora 等 RPM 系发行版本地构建验证
- **Linux 进程探测更稳健** - `sing-box` 进程检测新增活跃 PID 过滤，忽略 zombie 进程并输出更明确的复核日志
- **Linux 生命周期管理增强** - TUN 提权启动后会尽量记录真实 `sing-box` PID，停止/替换/更新内核时优先复用真实托管 PID 进行清理
- **清理失败诊断增强** - `kill/pkill` 与 sudo 清理链路补充错误透传和回退日志，便于定位权限不足、残留 root 进程等问题

### 📚 文档更新

- 重写 `README.md`、`README.zh.md`、`docs/development.md`、`docs/i18n.md`，统一当前跨平台架构、内核拉取流程、本地数据目录与发布说明
- **Linux 构建文档补充** - 开发文档新增 Fedora / RHEL 依赖安装说明，以及 Linux RPM 构建命令示例
- **更新日志补充 Linux TUN 修复说明** - 记录 Linux 下 TUN root 进程关闭回退、活跃 PID 复核与残留进程清理策略调整

## [v2.2.6] - 2026-03-05

### ✨ 主要更新

- **更新设置更灵活** - 新增 `stable / prerelease / autobuild` 三种更新通道，可按偏好选择更新节奏
- **备份与恢复上线** - 可以导出当前应用数据，也可先预检再恢复，迁移或重装更省心
- **订阅状态更清晰** - 自动更新会显示失败次数、原因和重试时间，方便判断是否需要手动处理
- **多语言体验提升** - 俄语/日语翻译补齐，通知与错误文案更统一，减少中英混杂
- **设置页文案本地化** - “系统平台”“加载中”等文案支持随语言切换

### 🌐 网络与代理（#38）

- **新增 Fake DNS（默认关闭）** - 支持 `fakeip` DNS server，默认不启用，避免影响既有配置行为
- **Fake DNS 生效范围可选** - 支持 `proxy_only`（仅代理流量）与 `global_non_cn`（全局非 CN）两种策略
- **Fake DNS 地址池可配置** - 支持自定义 IPv4/IPv6 CIDR（默认 `198.18.0.0/15`、`fc00::/18`），并在前端增加格式校验
- **反向映射缓存联动** - 启用 Fake DNS 时自动开启 `dns.reverse_mapping` 与 `experimental.cache_file.store_rdrc`
- **TUN 网络自愈能力** - TUN 连通性连续失败后自动触发内核自愈重启，缓解网络切换/插网线后的断网问题
- **自愈策略可配置** - 新增自愈开关与冷却时间（15-600 秒，默认 90 秒），并持久化到应用配置

### 🐛 稳定性与安全

- **修复重复注册问题** - 避免部分场景下的异常行为
- **修复隐藏启动与自动更新边界问题** - 启动更稳，订阅自动更新更可靠
- **增强旧备份兼容性** - 老版本备份恢复时会自动补默认更新通道（`stable`）
- **安全加固** - 收紧默认权限并完善 CSP 配置，降低潜在风险
- **修复 TUN 插网线后无网络连接** - 守护进程在 TUN 模式下支持自动自愈恢复，降低网络拓扑变化导致的全局断连概率
- **启动冲突自动清理（#42）** - 启动前若检测到旧内核进程残留，会先按进程名强制清理，再继续内核启动流程
- **强制清理失败提示增强（#42）** - 若旧内核进程无法结束，将阻断本次启动并明确提示手动结束进程/以管理员权限重试
- **快速重启兜底增强** - 停止阶段失败或超时时，升级为按进程名强制清理，减少重启被残留进程卡住的问题

## [v2.2.5] - 2026-03-02

### ✨ 新功能

- **内核操作编排器** - 新增生命周期编排队列，统一串行处理 `start/stop/restart/auto_manage`，减少并发竞态
- **状态快照接口** - 新增 `kernel_get_snapshot` 命令，前端初始化可先拉取快照再订阅增量事件
- **操作事件通道** - 新增 `kernel-operation-started` / `kernel-operation-finished` / `kernel-operation-failed` 事件，便于追踪内核操作链路
- **升级后订阅自动刷新** - 应用版本升级时自动尝试刷新当前激活订阅，失败后后台重试并通知前端手动处理

### 🔧 优化改进

- **生命周期一致性** - 自动托管逻辑改为复用统一编排路径，不再绕过生命周期入口直接启停
- **状态语义增强** - 内核状态事件与状态查询返回补充 `kernel_state`、`state_version`（以及操作返回中的 `op_id` 元信息）
- **启动稳定性校验** - 内核启动后增加短窗口稳定性检查，减少“返回启动成功但立即退出”的假成功体验
- **前端状态同步** - `KernelStore` 初始化改为“先快照后订阅”，并基于 `state_version` 忽略过期状态覆盖
- **设置保存策略调整** - 前端自动保存配置默认仅持久化到数据库，不再自动触发运行态重配，避免编辑设置时频繁触发内核重启/配置覆盖
- **订阅切换即时生效** - 切换活动配置路径时显式触发运行态应用，确保 `active_config_path` 与内核实际使用配置保持一致
- **下载出站默认值统一** - `singbox_download_detour` 默认值统一为 `direct`，与当前下载链路策略保持一致
- **内核更新入口降级** - 设置页保留手动内核更新能力，但标记为“高级维护”并弱化主操作入口，默认引导使用内嵌推荐内核
- **错误事件结构化** - `kernel-error` 事件补充 `code/message/details/source/recoverable/timestamp` 字段，并保留兼容 `error` 字段
- **全局错误提示与去重** - 前端新增内核失败全局提示中心，统一消费 `kernel-error` 与 `kernel-operation-failed`，同文案 10 秒内去重

### 🐛 问题修复

- **事件中继清理** - 中继任务清理从仅 `abort` 调整为 `abort + await`，降低重启窗口残留任务风险
- **启停并发冲突** - 修复手动启停、快速重启、自动托管可能并发触发导致状态漂移的问题
- **更新安装包匹配修复** - 修复安装包优先级提前退出条件，避免部分平台/架构场景下选择到错误安装包
- **sing-box 1.13 DNS 配置迁移** - 配置生成全面切换至新 DNS server 格式，移除 legacy `address/address_resolver` 与 `dns.servers[].strategy`
- **Domain Strategy 迁移修复** - 移除 `dns.strategy`/`default_domain_strategy` 等 legacy 选项，统一迁移到 `domain_resolver/default_domain_resolver` 对象格式
- **DNS detour 兼容修复** - 避免输出 `dns.servers[].detour=direct`，修复新内核报错 `detour to an empty direct outbound makes no sense`
- **启动前配置预检增强** - 启动前执行 `sing-box check --config`，对常见迁移错误输出可操作中文提示（legacy DNS / legacy domain strategy）
- **失败状态可见性修复** - 前端状态展示新增 `failed/crashed`，不再将内核失败统一折叠为 `stopped`
- **守护失败漏报修复** - 修复守护重启失败（非 sudo 场景）仅日志不通知前端的问题
- **自动拉起失败提示修复** - 应用启动自动管理内核失败时，前端可立即收到错误提示

### 🔧 优化改进

- **目标内核资源映射** - 构建时改为 `bundle.resources` 对象映射，只注入当前 `--target` 对应的 `sing-box(.exe)` 与 `version.txt`
- **CI 构建入口统一** - Release 工作流显式使用 `pnpm run tauri build`（wrapper 脚本），确保多平台构建遵循同一内核资源注入策略
- **Latest 拉取稳健性增强** - `fetch-kernel` 新增 release 页面兜底解析；当 GitHub API 波动时仍可自动解析最新版本
- **CI 鉴权拉取最新内核** - Release 工作流透传 `SING_BOX_GITHUB_TOKEN` 给内核下载脚本，降低匿名请求触发限流的概率
- **脚本回归测试补齐** - 新增 `scripts/kernel-targets.test.mjs`，覆盖目标资源映射与不支持目标兜底行为

### 📚 文档更新

- **开发文档补充** - 明确“仅当前目标平台内核随包分发”的资源注入机制，以及 CI 自动拉取 latest 的执行链路

### 🐛 问题修复

- **Release CI 编译修复** - 修复 `sudo_service` 在 Linux/macOS 目标构建时缺少 `tauri::Manager` trait 导入导致的编译失败

## [v2.2.4] - 2026-02-12

### 🔧 优化改进

- **Google 分流** - 新增 Google 分流组，使用 `geosite-google` 规则集覆盖所有 Google 服务（搜索、Gmail、YouTube、Gemini、Cloud 等）
- **错误提示优化** - 管理员重启失败时，首页弹窗会展示后端返回的具体错误信息，便于定位问题

### 🐛 问题修复

- **Windows 提权重启修复** - 移除易受路径编码影响的 VBS 提权流程，改用 PowerShell `Start-Process -Verb RunAs`，修复中文用户名路径下“找不到可执行文件”问题

## [v2.2.3] - 2025-12-22

### 🐛 问题修复

- 修复 Cargo.lock 依赖版本被错误改写导致的 Linux 构建失败

## [v2.2.2] - 2025-12-22

### 🔧 优化改进

- **原始订阅处理** - 使用原始配置时仅替换端口，不再改写 DNS/路由等高级字段
- **订阅入口提示** - 明确三种添加方式与支持类型，原始配置模式提示高级选项不生效
- **设置页提示** - 当前订阅为原始配置时，高级订阅模板区域展示提示说明
- **内核内嵌与自动拉取** - dev/build 自动拉取当前平台内核，缺失才下载
- **构建流程统一** - 本地与 CI 统一使用 `pnpm tauri dev/build`，自动注入平台配置

### 🐛 问题修复

- 修复订阅配置路径重复导致切换无效的问题（切换时自动重新生成配置）
- 启动时清理残留内核进程，避免复用非本程序启动的内核实例

---

## [v2.2.1] - 2025-12-20

### ✨ 新功能

- **Hysteria2 协议支持** - 新增对 hysteria2:// 协议的完整解析支持
- **URI 节点列表导入** - 支持直接粘贴 vmess://、vless://、trojan://、ss://、hysteria2:// 节点链接
- **业务分流组增强** - 完善 Telegram、YouTube、Netflix、OpenAI 等服务的节点候选列表
- **订阅管理优化** - 新增 URI 列表标签页，简化节点导入流程

### 🔧 优化改进

- **自动更新逻辑** - 修复多订阅环境下自动更新时的配置覆盖问题
- **手动配置限制** - 禁用手动配置的自动更新选项，避免潜在的配置冲突
- **配置导入兼容性** - 增强多层解码策略，提高对各种订阅格式的兼容性
- **表单验证增强** - 智能切换配置选项，提升用户输入体验

### 🐛 问题修复

- 修复多订阅自动更新时配置被覆盖的问题
- 解决 Linux 和 macOS 平台进程检测误杀自身应用的问题
- 修复订阅内容解析的兼容性问题

### 🌐 国际化

- 新增 URI 列表相关的多语言支持
- 更新手动配置提示信息的翻译
- 优化用户引导文本的表述

### 💡 使用提示

- URI 列表支持一行粘贴多个节点链接
- 手动配置（JSON/URI）不支持自动更新，请手动刷新
- Hysteria2 协议节点支持完整的参数配置（sni、alpn、insecure 等）

---

## [v2.2.0] - 2025-12-18

### ✨ 新功能

- **内核版本选择** - 支持选择并下载指定版本的内核
- **内核快速重启** - 实现更快速的内核重启功能，优化停止逻辑
- **日志轮转** - 引入文件日志轮转功能，防止日志文件过大
- **Clash/Mihomo订阅** - 新增对Clash/Mihomo订阅格式的完整支持
- **主题设置** - 支持深色/浅色主题切换，自定义主题色
- **私网IP规则** - 内置私网IP规则集，优化路由配置

### 🔧 优化改进

- **sudo提权** - 重构sudo密码存储与提权逻辑，提升安全性
- **进程检测** - 优化Linux和macOS平台的进程检测，避免误杀自身应用
- **延迟测试** - 重构延迟测试功能，优化代理组配置
- **配置生成** - 重构配置生成器与设置补丁模块，提升性能

### 🛠️ 技术更新

- **依赖更新** - 更新项目依赖包版本
- **架构检查** - 添加平台信息检测与架构兼容性检查
- **命令优化** - 移除冗余的内核安装和更新命令
- **IPv6 TUN** - 默认禁用IPv6 TUN功能，提升稳定性

### 🐛 问题修复

- 修复多订阅配置路径同步问题
- 解决平台相关的进程管理问题

---

## v2.1.0 - 之前版本

### 主要功能

- 跨平台代理客户端（Windows、Linux、macOS）
- 完整的代理管理、路由规则配置
- 订阅管理和自动更新
- 系统服务集成
- 流量统计和监控

---

> 💡 **提示**: 建议定期更新以获得最新功能和安全修复。
