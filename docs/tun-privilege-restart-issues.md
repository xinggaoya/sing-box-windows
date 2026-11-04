# TUN 模式「需重启 + 需管理员」问题清单

> **性质**：代码调研产出 + 修复记录。
> **范围**：TUN 模式在 Windows 上的启用/关闭链路，聚焦「反复要求重启」「反复要求管理员权限」「TUN 开关自己弹回」三类用户可感知症状。
> **调研时间**：2026-11-04
> **修复状态**：12 条中 **11 条已修复**（TUN-01 ~ TUN-10、TUN-12），**TUN-11 按需求保持现状**（不引入计划任务等新的提权通道）。
> **基线**：`master` / `89db6af`

---

## 0. 一句话结论

用户感知到的「TUN 必须重启并以管理员身份运行」实际上由**三类不同性质的原因**叠加而成：

| 层次 | 原因 | 性质 | 可否消除 |
| --- | --- | --- | --- |
| ① 架构约束 | sing-box 无运行期增删 inbound 的 API，TUN inbound 必须在进程启动时存在 | 硬约束 | ❌ 不可，只能做到无感 |
| ② OS 约束 | Windows 上 Wintun 建适配器 + `auto_route` 改路由表必须 Administrator token | 硬约束 | ❌ 不可，但**只需内核提权，无需 GUI 提权** |
| ③ 实现缺陷 | 权限探测不可靠 + 读配置路径主动抹掉 TUN 开关 + 提权被绑定到整个 GUI | **可修** | ✅ 可彻底消除 |

其中 **③ 是本清单的主体**。第 ③ 类修完后，「反复重启 / TUN 开关自己弹回 / 已提权还弹 UAC」等实际故障即消失；剩余的只有「首次启用 TUN 需要一次授权」这一真实且必要的一次性动作。

---

## 1. 问题清单总表

| 编号 | 级别 | 标题 | 主位置 | 状态 |
| --- | --- | --- | --- | --- |
| TUN-01 | P0 | 管理员探测用 `net session`，存在假阴性 | `system_service.rs:321` | ✅ 已修 |
| TUN-02 | P0 | 读配置路径强制关闭 TUN **并写回数据库** | `enhanced_storage_service.rs:509` | ✅ 已修 |
| TUN-03 | P0 | 首页 / 托盘两条 TUN 路径行为不一致 | `HomeView.vue:274` / `TrayStore.ts:197` | ✅ 已修 |
| TUN-04 | P0 | `ensurePrivilege` 早于「状态未变」短路执行 | `useTunToggle.ts:131` | ✅ 已修 |
| TUN-05 | P1 | 权限类失败无专用诊断码（分类器已就绪但无发射点） | `process/mod.rs:15` / `utils.rs:262` | ✅ 已修 |
| TUN-06 | P1 | 内核启动重试不区分权限错误 | `kernel_service/runtime.rs` | ✅ 已修 |
| TUN-07 | P1 | Windows 缺少特权 kill 回退，残留高权限内核 | `process/manager.rs:266` | ✅ 已修（改为明确诊断） |
| TUN-08 | P1 | 前端关闭路径不做权限判定 | `HomeView.vue:415` / `TrayStore.ts:205` | ✅ 已修 |
| TUN-09 | P1 | 切换失败回滚只还原开关，不恢复运行态 | `useTunToggle.ts:145` | ✅ 已修 |
| TUN-10 | P1 | 管理员重启失败后网络出口丢失 | `useTunToggle.ts:171` | ✅ 已修 |
| TUN-11 | P2 | 提权被绑定到整个 GUI（需重启整个应用） | `system_service.rs` / `process/manager.rs:589` | ⏸️ **本次不做（保持现状）** |
| TUN-12 | P2 | TUN inbound 存在两份并行渲染实现，字段不一致 | `config_render.rs:46` / `tun_profile.rs:156` | ✅ 已修（配置内容有变） |

> **TUN-11 的处置**：按需求「**不再改变当前的方式（保持程序重启提权），仅作补充完善修复**」，
> 本次**不引入**计划任务 / Windows 服务等新的提权通道。
> 保留的已知限制：**Windows 上首次启用 TUN 仍需一次 UAC 并重启整个应用**（原因见第 5 节）。

---

## 2. P0 — 直接导致「反复重启 / 功能不可用」

### TUN-01 管理员探测用 `net session`，存在假阴性

**位置**：`src-tauri/src/app/system/system_service.rs:321-332`

```rust
fn check_admin_windows() -> bool {
    let result = std::process::Command::new("net")
        .arg("session")
        .creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW)
        .output();
    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
```

**根因**：把「当前进程 token 是否已提权」这个**进程内属性**，外包给一个**依赖 Server 服务（lanmanServer）可用性**的外部命令。两者并不等价：Server 服务被禁用 / 精简镜像 / 命令行为异常时，已提权的进程也会被判为非管理员。

**影响**：
- 直接触发 TUN-02 的自毁链（假阴性 → 判定非管理员 → 关闭并落库 TUN）。
- `restart_as_admin_windows` 开头的 `if check_admin() { return Ok(()); }` 同样误判 → **应用已经以管理员运行时，切换 TUN 仍会再去走一次 UAC 重启**，正是用户描述的「反复要求管理员运行」。

**修复建议**：
1. 改用 token elevation 检测：`OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY)` → `GetTokenInformation(TokenElevation)` → 读取 `TokenIsElevated`。
2. `windows` crate **0.61.3 已在 `Cargo.lock` 中**（Tauri 传递依赖），加为直接依赖成本极低。
3. 仅在 API 调用失败时回退 `net session`，不再把它作为主判据。

**验收**：
- 提权 / 非提权两种启动方式下 `check_admin` 返回值均正确。
- 在禁用 Server 服务的环境下重复上述测试，结果仍正确。
- 前端首页「运行环境」卡片的管理员状态显示与实际一致。

---

### TUN-02 读配置路径强制关闭 TUN 并写回数据库

**位置**：`src-tauri/src/app/storage/enhanced_storage_service.rs:509-533`

```rust
// Windows：非管理员启动时自动关闭 TUN，避免因缺少权限导致内核无法拉起
#[cfg(target_os = "windows")]
if config.tun_enabled && !crate::app::system::system_service::check_admin() {
    let previous_mode = config.proxy_mode.clone();
    config.tun_enabled = false;
    config.proxy_mode = if config.system_proxy_enabled { "system" } else { "manual" }.to_string();
    if let Err(err) = storage.save_app_config(&config).await {
        tracing::warn!("在非管理员模式下写入关闭 TUN 设置失败: {}", err);
    } else {
        tracing::info!("检测到当前未获得管理员权限，已自动关闭 TUN 模式（原模式: {}）", previous_mode);
    }
}
```

**根因**：一个**读**函数里包含了**写**副作用，且写的是用户显式设置的开关值。

**影响**（自毁链）：

```
用户开 TUN → 落库 tun_enabled=true
  → 紧接着 applyProxySettings 读配置 → 判定「非管理员」→ 改 false → 再次落库
  → resolve_proxy_runtime_state 拿到的 tun_enabled=false → 不写 TUN inbound
  → 重启内核 → TUN 未生效；数据库里也是 false
  → 前端开关弹回关闭 → 用户再点 → 再弹 UAC 重启 …… 循环
```

该读函数调用点极多，几乎覆盖全部配置读取路径：
- `core/kernel_service/runtime.rs:189`（`resolve_proxy_runtime_state`）
- `core/proxy_service/inbound_writer.rs:103`（`write_inbounds_to_config`）
- `core/proxy_service/inbound_writer.rs:80`（`load_allow_lan_access`）、`:207`（`update_dns_strategy`）
- `storage/enhanced_storage_service.rs:564`（`db_save_app_config` 且 `apply_runtime=true` 时）

**额外风险**：托盘路径通过 `applyOverrides: { tun_enabled: target }` 覆盖了内存态，但**数据库已被钳制为 false** → 数据库状态与运行态不一致，重启后 TUN 丢失、UI 开关与实际运行不符。

**修复建议**：
1. **删除落库副作用**：读配置不得修改持久化状态。
2. 「当前无提权」这一事实不应通过改写用户配置来表达，应改为**内核启动路径返回显式错误码** `KERNEL_PERMISSION_DENIED`（分类器已就绪，见 TUN-05）。
3. 若仍需保留「非提权时不生成 TUN inbound」的兜底，只在**内存态 / 本次生成的配置**上钳制，不触碰 DB。

**验收**：
- 非提权进程反复读配置，`tun_enabled` 不被改写；`db_get_app_config` 幂等。
- 开关打开 → 重启应用 → 状态仍为打开（仅当内核确实因权限起不来时才在启动阶段报错，而非静默改写配置）。

---

### TUN-03 首页 / 托盘两条 TUN 路径行为不一致

**位置**：
- `src/views/HomeView.vue:274-300`（`useTunToggle` 注入）—— **无** `applyOverrides`；Windows 管理员检查内联在 `toggleTunProxy`（`:412-425`）
- `src/stores/tray/TrayStore.ts:197-250`（`useTunToggle` 注入）—— **有** `applyOverrides: (target) => ({ tun_enabled: target })`（`:250`）；Windows 管理员检查在 `ensurePrivilege` 内（`:204-218`）

**根因**：`useTunToggle` 把「平台差异」抽象成回调，但两条调用方对回调的用法不同：
- 首页在**回调之外**自己做 Windows 提权判断，并返回 `'proceed'`；
- 托盘在**回调之内**做判断并返回 `'escalated'`。

**影响**：
- 运行时配置语义不同（托盘强制传 `tun_enabled` 覆盖，首页依赖 DB 值）。
- `ensurePrivilege` 的返回契约（`'proceed'` / `'escalated'`）在两条路径上语义不一致，后续维护极易再次分叉——而这正是 `useTunToggle` 文档注释里描述的历史 bug 类别（「某条路径忘了回滚」）。
- 首页的 `prepareTunModeWithAdminRestart`（`HomeView.vue:467`）与 `prepareTunAndRestartAsAdmin`（`useTunToggle.ts:171`）构成双层包装。

**修复建议**：
1. 平台差异**全部收敛到 `ensurePrivilege` / `handleSudoFailure` 回调内**，首页删除内联的 Windows 分支。
2. 统一 `applyOverrides` 语义（要么都传，要么都不传）——建议**都不传**，统一以 DB 持久化值为准，避免内存态与 DB 不一致。
3. `ensurePrivilege` 返回值语义写进 `useTunToggle` 文档注释并在两个调用方各加注释。

**验收**：首页与托盘触发同一目标状态时，后端收到的 `applyProxySettings` 参数完全一致（补一条对比测试）。

---

### TUN-04 `ensurePrivilege` 早于「状态未变」短路执行

**位置**：`src/composables/useTunToggle.ts:131-137`

```ts
const applyTunSwitch = async (target: boolean): Promise<TunSwitchOutcome> => {
  // 提权前置由调用方处理（可能直接转入「保存 + 管理员重启」，无需本地回滚）
  if (options.ensurePrivilege) {
    const outcome = await options.ensurePrivilege(target)
    if (outcome === 'escalated') return { status: 'escalated' }
  }

  const previous = appStore.tunEnabled
  if (previous === target) return { status: 'skipped' }
  ...
```

**根因**：短路判断在提权之后。

**影响**：
- 目标状态与当前一致时**仍会执行提权前置**。在托盘路径下 `ensurePrivilege` 会调用 `prepareTunAndRestartAsAdmin()`——该函数**无条件把 TUN 置为 true**（`useTunToggle.ts:177`），因此「关闭 TUN」在非提权状态下可能被反向置为开启并触发 UAC 重启。
- 当前托盘调用方 `applyTunToggleFromTray`（`TrayStore.ts:253-254`）有外部 `if (appStore.tunEnabled === enabled) return` 兜底，但**兜底在状态机之外**，属于隐性依赖；首页 `toggleTunProxy` 无此兜底。

**修复建议**：把 `previous === target` 判断提到 `ensurePrivilege` 之前。语义上「状态未变 ⇒ 无需任何特权」是成立的。

**验收**：新增用例——「状态未变 + 非提权」时，`ensurePrivilege` **不应**被调用（现有 `useTunToggle.test.ts` 需同步调整）。

---

## 3. P1 — 体验与可诊断性

### TUN-05 权限类失败无专用诊断码

**位置**：
- `src-tauri/src/process/mod.rs:15` —— `PermissionError(String)` **全仓库无任何构造点**（仅在 `:37` 有 Display 实现）。
- `src-tauri/src/app/core/kernel_service/utils.rs:262` —— 分类器**已能解析** `KERNEL_PERMISSION_DENIED` → `StartupDiagnosisKind::PermissionDenied`，但该错误码**全仓库无发射点**。
- `src-tauri/src/app/core/kernel_service/state.rs:50,70` —— `StartupDiagnosisKind::PermissionDenied` 已定义且优先级为 70。

**影响**：Windows 非提权下内核起不来时，诊断落到 `KERNEL_START_FAILED` / `KERNEL_PROCESS_EXITED_EARLY`，**用户看不到「需要管理员权限」的提示**，只会看到泛化的「内核启动失败」。

**修复建议**：
1. 在 `try_start_kernel_process` 的失败分类里识别权限类信号（stderr 中的 access denied / TUN 设备创建失败等），发射 `KERNEL_PERMISSION_DENIED`。
2. 真正构造并使用 `ProcessError::PermissionError`。
3. 前端 `src/utils/kernel-errors.ts` 补一条对应文案与「以管理员身份重启」动作按钮。

**验收**：非提权 + TUN 配置启动内核 → 前端明确提示需要管理员权限，而非泛化报错。

---

### TUN-06 内核启动重试不区分权限错误

**位置**：`src-tauri/src/app/core/kernel_service/runtime.rs:58`、`:80`、`:330` 附近的重试逻辑。

**影响**：权限类失败会白跑满 3 次重试；守护自愈重启（`guard.rs`）在非提权场景下会反复重试同一注定失败的启动，直到 `MAX_CONSECUTIVE_RESTARTS = 2` 用尽并进入 10 分钟熔断。

**修复建议**：权限类错误 **fail-fast**，不进入重试循环。

---

### TUN-07 Windows 缺少特权 kill 回退

**位置**：`src-tauri/src/process/manager.rs:268-311`（`try_kill_pid_with_optional_privilege`）

```rust
// manager.rs:283-284
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
let _ = app_handle;   // Windows 直接丢弃提权句柄
```

**影响**：内核若由管理员权限启动（当前必然如此，因为 GUI 整体被提权），普通权限的 GUI **杀不掉**它 → 残留高权限 sing-box 进程 + TUN 网卡 / 路由残留。

**对照**：Linux 已在 CHANGELOG 中修过同构问题（「TUN 提权启动后普通权限清理失败」「TUN 提权启动后会尽量记录真实 sing-box PID」），**Windows 至今没有对应实现**。

**修复建议**：Windows 增加等价的特权清理通道。可与 TUN-11 的提权通道复用同一个机制（计划任务 / 服务 / 提权 helper）。

**验收**：管理员启动内核 → 关闭 GUI（非提权）→ 高权限 sing-box 被正确结束，网卡与路由清理干净。

---

### TUN-08 前端关闭路径不做权限判定

**位置**：
- `src/views/HomeView.vue:415` —— `if (value && isWindowsPlatform.value) { ... }`
- `src/stores/tray/TrayStore.ts:205` —— `if (!target) return 'proceed'`

**影响**：只在**开启**方向判定权限，**关闭**方向一律用普通权限操作高权限内核（与 TUN-07 叠加，导致关闭失败或残留）。

**修复建议**：关闭方向同样做权限判定；不具备权限时走与开启方向对称的提权通道，或明确提示「请以管理员身份关闭 TUN」。

---

### TUN-09 切换失败回滚只还原开关，不恢复运行态

**位置**：`src/composables/useTunToggle.ts:145-149`（核心流程失败）与 `:183-193`（异常分支）

```ts
await appStore.toggleTun(previous)   // 只还原 DB 布尔值
return { status: 'failed', failure }
```

**影响**：切换失败后可能出现「开关显示已回滚，但内核仍是新模式（或已被停掉）」的不一致——内核配置、路由表、TUN 网卡与 UI 状态脱节。

**修复建议**：回滚时补一次「应用旧配置 + 重启内核」，使运行态与开关状态重新对齐。

**验收**：注入「应用配置失败」与「重启内核失败」两种故障，验证回滚后内核实际运行在旧模式。

---

### TUN-10 管理员重启失败后网络出口丢失

**位置**：`src/composables/useTunToggle.ts:171-193`（`prepareTunAndRestartAsAdmin`）

```ts
await appStore.toggleTun(true)
await appStore.saveToBackend()
if (appStore.isRunning) {
  await kernelStore.stopKernel()      // ← 内核已停
}
const { systemService } = await import('@/services/system-service')
await systemService.restartAsAdmin()  // ← 失败则抛错
return true
} catch {
  await appStore.toggleTun(previous)  // ← 只回滚开关，内核不重启
  return false
}
```

**影响**：内核已停 → `restartAsAdmin` 失败 → catch 只还原开关，**内核不重启** → 用户彻底失去代理出口（TUN 是全量接管，失去出口即断网）。

**修复建议**：catch 分支补上「按旧模式重启内核」，恢复用户原有出口后再提示失败。

---

## 4. P2 — 架构性（改造量大，收益也大）

### TUN-11 提权被绑定到整个 GUI

**位置**：
- `src-tauri/src/process/manager.rs:589-604` —— sing-box 以**普通子进程**启动，继承 GUI 的 token：

  ```rust
  // Windows: 直接启动（假设应用已以管理员权限运行）
  #[cfg(target_os = "windows")]
  {
      let _ = (tun_enabled, kernel_str, app_handle);
      let mut cmd = Command::new(kernel_path);
      cmd.args(["run", "-D", work_dir_str, "-c", config_str]);
      ...
  }
  ```
- `src-tauri/src/app/system/system_service.rs` —— `restart_as_admin_windows`（PowerShell `Start-Process -Verb RunAs`）是当前唯一的提权手段，它提权的是**整个应用**。

**根因**：因为内核需要 Administrator token，就把 GUI 也一起提权。**这是实现耦合，不是 OS 要求。**

**影响**：每次启用 TUN 都要「关掉整个 App → UAC 弹窗 → 整个 App 以管理员重启」，用户体感就是「必须重启应用」。

**修复方向**（三选一，工作量递减 / 稳定性递减）：

| 方案 | 机制 | UAC 次数 | 工作量 | 备注 |
| --- | --- | --- | --- | --- |
| **B1 计划任务（推荐）** | `schtasks /Create /RL HIGHEST` 注册高权限任务，之后 `schtasks /Run` 无弹窗启动高权限 sing-box，`/End` 停止 | 1 次（安装时） | 中 | GUI → 内核已是 gRPC over TCP，**通信层零改动** |
| B2 Windows 服务 | 独立服务进程托管 sing-box（Clash Verge Rev 路线） | 1 次（安装时） | 大 | 最稳，GUI 重启不断网；需第二个二进制 + 安装/卸载 + 体积增大 |
| B3 `runas` 直接提权内核 | `ShellExecuteEx` 提权 sing-box 子进程 | 每次 | 小 | 只省掉「重启 GUI」，UAC 仍每次弹；stderr 管道无法跨提权边界继承；停止时又需提权 kill |

**建议**：先做 TUN-01 ~ TUN-10（不改架构、零功能风险），再评估 B1。

---

### TUN-12 TUN inbound 存在两份并行渲染实现，字段不一致

**位置**：
- `src-tauri/src/app/singbox/config_render.rs:46-66`（`build_inbounds`）—— 被 `config_generator.rs:237`（生成路径）与 `settings_patch.rs:471`（补丁路径）使用；产出 **不含 `interface_name`** 的 TUN inbound。
- `src-tauri/src/app/core/tun_profile.rs:156-190`（`to_inbounds`）—— 被 `core/proxy_service/inbound_writer.rs:53`（运行时切换路径）使用；产出 **含 `interface_name`**，Windows 下默认为 `"singbox_tun"`（`tun_profile.rs:235`）。

**影响**：同一个 `AppConfig`，**生成路径**与**运行时切换路径**写出的 TUN inbound 不一致。Windows 上 Wintun 适配器名称会因此不同，可能表现为「切换 TUN 后网卡名变了 / 旧网卡残留」。

**修复建议**：按根 AGENTS.md「配置语义唯一来源」的原则，把 `config_render::build_inbounds` 收敛到 `ConfigIntent` 单一映射点，或让 `TunProfile` 成为唯一渲染实现、`config_render` 复用它。改动需同步校验 `src-tauri/src/app/singbox/testdata/config_golden/*.json` 黄金样本（用 `SINGBOX_BLESS_GOLDEN=1` 重新生成，并确认逐字段差异符合预期）。

**注意**：此项会改变生成的 sing-box 配置内容，**不是纯 bug 修复**，需单独评估并补 CHANGELOG。

---

## 5. 明确无法消除的限制

1. **Windows 全量接管必须 Administrator。** Wintun 适配器创建、`auto_route` 路由表写入均属内核态操作，没有 user-mode 替代方案（WinDivert / WFP 同样需要安装驱动）。**本次保持现状：提权仍绑定到整个 App**（首次启用 TUN 需一次 UAC + 重启应用），仅修掉了误判与重复弹窗。
2. **切换 TUN 必须重启 sing-box。** sing-box 没有配置热重载 API，运行中的进程不会重读 `config.json`。可优化到「秒级重启、GUI 不断、路由自动恢复」，但无法做到零中断。
3. **Linux / macOS 的 sudo 密码方案无法免掉。** `app/system/sudo_service.rs` 的密码存储 + 提权启动是既定设计，优化空间只在提权失败时的诊断与重试（对应 TUN-05 / TUN-06）。
4. **UAC 首次授权无法绕过。** 除非引入计划任务 / 服务把提权固化下来（TUN-11，本次不做），否则首次启用 TUN 必然弹窗。**但修复后它只弹一次**（此前因误判会反复弹）。

---

## 6. 修复批次与结果

| 批次 | 内容 | 编号 | 状态 |
| --- | --- | --- | --- |
| **第 1 批** | 修掉「反复重启 / 开关自己弹回 / 已提权还弹 UAC」 | TUN-01 ~ TUN-04 | ✅ 已完成 |
| **第 2 批** | 补齐权限诊断、失败回滚、关闭链路对称性 | TUN-05 ~ TUN-10 | ✅ 已完成 |
| **第 3 批** | 提权与 GUI 解耦（消灭「重启整个应用」） | TUN-11 | ⏸️ 本次不做（保持现状） |
| **第 4 批** | TUN inbound 渲染收敛到单一事实来源 | TUN-12 | ✅ 已完成（黄金样本已同步） |

**修复后 Windows 上的实际表现**：
- 已提权进程：切换 TUN 不再弹 UAC、不再重启 GUI，只重启内核（秒级）。
- 未提权进程：切换 TUN 仍需一次 UAC + 重启应用（**OS 约束，见第 5 节**），但**只弹一次**，
  且失败时给出明确诊断（`KERNEL_PERMISSION_DENIED` → 「需要管理员权限」+ 一键重启入口），
  而不是静默改写用户设置。

### 6.1 关键改动落点

| 编号 | 主要改动位置 |
| --- | --- |
| TUN-01 | 新增 `app/system/elevation/`（`state.rs` 合并规则 + `windows.rs` token 检测）；`system_service::check_admin` 改为委托 |
| TUN-02 | 删除 `enhanced_storage_service::db_get_app_config_internal` 的落库副作用；新增 `core/proxy_service/platform_policy.rs` |
| TUN-03 | 首页与托盘共用 `useTunToggle` 的 `ensurePrivilege` 与 `ensureWindowsTunPrivilege` |
| TUN-04 | `useTunToggle::applyTunSwitch` 的短路判断提前 |
| TUN-05 | `kernel_service/utils::is_permission_failure_detail` + `KERNEL_PERMISSION_DENIED` 发射点；前端 `kernel-errors.ts` 新增 `requiresElevation` |
| TUN-06 | `ProcessManager::start` 重试循环遇权限错误直接返回 |
| TUN-07 | `ProcessManager::try_kill_pid_with_optional_privilege` 返回类型化错误并上报 |
| TUN-08 | 首页/托盘的 `ensurePrivilege` 对**开启与关闭**对称；新增 i18n `home.tunConfirm.disableDescription` |
| TUN-09 | `useTunToggle` 新增 `restoreRuntime`，回滚时按旧模式重新应用配置并重启内核 |
| TUN-10 | `prepareTunAndRestartAsAdmin` 失败时按旧模式恢复内核 |
| TUN-12 | `TunProfile::to_tun_inbound` 成为唯一实现；`TunIntent::to_proxy_options` 统一「意图 → 运行时选项」映射 |

---

## 7. 关联文档

- `docs/development.md` —— 前后端通信与提权链路
- `docs/gap-analysis-vs-mature-projects.md` —— 与 Clash Verge Rev / v2rayN / FlClash 的 TUN 提权方案对比
- `docs/implementation-plan-2026-10.md` —— 配置双模型收敛（与 TUN-12 直接相关）
- `docs/CHANGELOG.md` —— 实施各批次后需补 `[未发布]` 条目

> **实施提醒**（根 AGENTS.md「DOC SYNC GATE」）：本清单涉及行为修复落地后，需同步更新
> `docs/CHANGELOG.md` 的 `[未发布]` 段；若新增 Tauri 命令 / 事件 / 诊断码，还需同步
> `src/AGENTS.md` 与 `src-tauri/src/app/AGENTS.md`。
