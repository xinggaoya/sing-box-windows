# CrabNebula DevTools 接入 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use subagent-driven-development (recommended) or executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在开发态接入 Tauri 官方推荐的 CrabNebula DevTools，让日志、Tauri command/event 的 payload 与耗时可视化调试，同时保证 release 产物零影响、现有文件与终端日志不丢失。

**Architecture:** 用 cargo feature `devtools` 隔离插件（release 构建不编译插件、不开启 `tauri/tracing`）；开发态由 `tauri-plugin-devtools` 独占全局 tracing subscriber，项目的「文件 + 控制台」日志改经官方 `attach_logger` 桥接（自实现 `log::Log` 适配器）输出，因此 DevTools 面板与原有日志输出同时可用。日志基础设施拆成三块：`log_bridge`（过滤 + 格式化，纯逻辑）、`log_util`（目录 / 文件 writer / 直连 subscriber）、`devtools_util`（插件装配）。

**Tech Stack:** Rust 1.88+ / Tauri 2.11 / `tauri-plugin-devtools` 2.2.3 / `tauri-plugin-devtools-app` 2.1.4 / tracing + tracing-subscriber + tracing-appender / `log` 0.4 / Node 24 + `node:test`

**Spec:** `docs/design-crabnebula-devtools.md`（设计文档，与本计划一并阅读；事实编号 F1–F8 引用自该文档）

## Global Constraints

- `src-tauri/Cargo.toml` 的 `rust-version` 上调为 `1.88`（插件 2.2.3 / devtools-app 2.1.4 的 MSRV，F3）
- 依赖版本固定：`tauri-plugin-devtools = "2.2.3"`、`tauri-plugin-devtools-app = "2.1.4"`、新增直接依赖 `log = "0.4"`；`Cargo.lock` 由 cargo 生成，**不得手改**
- 插件只能经 cargo feature `devtools` 启用；`build*` 系列命令**不得**携带该 feature（release 产物零影响，F2）
- 日志默认指令串固定为 `debug,sing_box_windows=debug,tauri=info,sqlx=warn`（与现有 `log_util::build_env_filter` 保持一致）
- 不修改 `src-tauri/tauri.conf.json`（含 CSP，F6）；不修改任何前端代码
- Rust 约定：模块/函数 snake_case；单测放同名 `*.tests.rs`，用 `#[cfg(test)] #[path = "xxx.tests.rs"] mod tests;` 挂载；scripts 测试用 `node:test`，文件名 `scripts/*.test.mjs`
- 提交前按根 `AGENTS.md` 的 DOC SYNC GATE 同步文档：`docs/CHANGELOG.md` 的 `[未发布]`、根 `AGENTS.md` 的 COMMANDS、`src-tauri/AGENTS.md`
- 每个任务结束提交一次，提交信息用中文，格式 `feat: ...` / `refactor: ...` / `test: ...` / `docs: ...` / `ci: ...`

## Review Focus

以下输入/情形规格隐含、但没有哪条单测能覆盖，是评审时最该盯的地方：

1. **RUST_LOG 未设置 / 空白 / 含非法指令** → 过滤行为必须可预期（不能一条都不出、也不能全部放行），由 Task 1 单测锁定
2. **日志突发写入** → `tracing_appender` 默认是 lossy 模式，队列满会丢行（与现状一致，允许丢，但不得 panic / 不得阻塞调用方）
3. **3033 端口被占用** → 插件自动换端口，终端打印的链接必须与实际监听端口一致（Task 6 手工核对）
4. **release 构建误带 devtools feature** → 插件不得初始化、不得抢全局 subscriber（Task 3 用 `cargo tree` 断言）
5. **全局 subscriber 只能设置一次** → devtools 插件的 `init()` 遇已存在的 subscriber 会 **panic**（F1），初始化顺序必须是「先 devtools、后（仅非 devtools 模式）fmt 层」
6. **Embedded 形态需外网** + CrabNebula 桌面客户端无 Windows ARM64 版本 → 离线/ARM64 环境只能用 standalone 网页形态（Task 6 手工确认降级路径可用）

## File Structure

| 文件 | 动作 | 职责 |
| --- | --- | --- |
| `src-tauri/src/utils/log_bridge.rs` | 新建 | devtools 模式下的日志桥接：RUST_LOG 指令过滤（`LogDirectiveFilter`）、行格式化（`format_log_line` / `format_record_line`）、`log::Log` 适配器（`FileConsoleLogger`） |
| `src-tauri/src/utils/log_bridge.tests.rs` | 新建 | 过滤匹配 + 行格式化的单元测试 |
| `src-tauri/src/utils/log_util.rs` | 改造 | 抽出 `LogSink` / `create_log_sink` / `current_log_dir` / `install_direct_subscriber`；保留目录准备、轮转、清理任务 |
| `src-tauri/src/utils/log_util.tests.rs` | 新建 | sink 写入落盘测试 |
| `src-tauri/src/utils/devtools_util.rs` | 新建 | 装配 devtools 插件（`attach_logger` + `init`）与 devtools-app 插件，返回插件对 |
| `src-tauri/src/utils/mod.rs` | 修改 | 挂载新模块（`devtools_util` 受 `#[cfg(feature = "devtools")]` 门控） |
| `src-tauri/src/lib.rs` | 修改 | `run()` 内按模式接线：devtools 优先接管全局 subscriber，直连模式才装 fmt 层 |
| `src-tauri/Cargo.toml` | 修改 | 新增 `[features] devtools`、三个依赖、`rust-version = "1.88"` |
| `scripts/tauri-wrapper.mjs` | 修改 | `dev` 命令注入 `--features devtools` |
| `scripts/tauri-wrapper.test.mjs` | 修改 | 注入逻辑的 node:test 用例 |
| `.github/workflows/ci.yml` | 修改 | clippy / test 增加 `--features devtools`，保证 devtools 代码路径持续可编译 |
| `docs/CHANGELOG.md` / `AGENTS.md` / `src-tauri/AGENTS.md` | 修改 | 文档同步（DOC SYNC GATE） |

---

## Task 1: 日志过滤与行格式化（纯逻辑 + 单测）

**Files:**
- Create: `src-tauri/src/utils/log_bridge.rs`
- Create: `src-tauri/src/utils/log_bridge.tests.rs`
- Modify: `src-tauri/src/utils/mod.rs`
- Modify: `src-tauri/Cargo.toml`（新增 `log = "0.4"`）

**Produces（后续任务依赖这些签名）:**
- `pub const DEFAULT_LOG_DIRECTIVES: &str = "debug,sing_box_windows=debug,tauri=info,sqlx=warn";`
- `pub struct LogDirectiveFilter`：`pub fn parse(spec: &str) -> Self`、`pub fn from_env() -> Self`、`pub fn enabled(&self, target: &str, level: log::Level) -> bool`
- `pub fn format_log_line(timestamp: &str, level: log::Level, thread: &str, file: Option<&str>, line: Option<u32>, message: &str) -> String`
- `pub fn format_record_line(record: &log::Record) -> String`

**Steps:**

- [ ] 1.1 在 `src-tauri/Cargo.toml` 的 `[dependencies]` 段新增一行（放在 `tracing-appender` 附近，保持字母序风格）：
  ```toml
  log = "0.4"
  ```
- [ ] 1.2 在 `src-tauri/src/utils/mod.rs` 新增 `pub mod log_bridge;`（按现有字母序插在 `log_util` 之前）
- [ ] 1.3 新建 `src-tauri/src/utils/log_bridge.tests.rs`，写入以下用例（先写测试）：
  ```rust
  use super::*;

  #[test]
  fn 未匹配 target 使用全局级别() {
      let filter = LogDirectiveFilter::parse(DEFAULT_LOG_DIRECTIVES);

      assert!(filter.enabled("other_crate::module", log::Level::Debug));
      assert!(!filter.enabled("other_crate::module", log::Level::Trace));
  }

  #[test]
  fn target 前缀指令覆盖全局级别() {
      let filter = LogDirectiveFilter::parse(DEFAULT_LOG_DIRECTIVES);

      assert!(filter.enabled("tauri::app::run", log::Level::Info));
      assert!(!filter.enabled("tauri::app::run", log::Level::Debug));
      assert!(filter.enabled("sqlx::query", log::Level::Warn));
      assert!(!filter.enabled("sqlx::query", log::Level::Info));
      assert!(filter.enabled("sing_box_windows::app::core", log::Level::Debug));
  }

  #[test]
  fn 更长前缀优先于更短前缀() {
      let filter = LogDirectiveFilter::parse("sing_box_windows=info,sing_box_windows::app=trace");

      assert!(!filter.enabled("sing_box_windows::app::core", log::Level::Debug));
      assert!(filter.enabled("sing_box_windows::app::core", log::Level::Trace));
      assert!(filter.enabled("sing_box_windows::other", log::Level::Info));
      assert!(!filter.enabled("sing_box_windows::other", log::Level::Debug));
  }

  #[test]
  fn 空指令串不输出任何日志() {
      let filter = LogDirectiveFilter::parse("");

      assert!(!filter.enabled("any", log::Level::Error));
  }

  #[test]
  fn 非法级别指令被忽略() {
      let filter = LogDirectiveFilter::parse("foo=bar,warn");

      assert!(filter.enabled("any", log::Level::Warn));
      assert!(!filter.enabled("any", log::Level::Info));
  }

  #[test]
  fn off 级别关闭全部输出() {
      let filter = LogDirectiveFilter::parse("off");

      assert!(!filter.enabled("any", log::Level::Error));
  }

  #[test]
  fn 行格式与 fmt 层风格一致() {
      let line = format_log_line(
          "2026-10-30T23:06:12.345",
          log::Level::Info,
          "ThreadId(1)",
          Some("core.rs"),
          Some(42),
          "内核已启动",
      );

      assert_eq!(
          line,
          "2026-10-30T23:06:12.345  INFO ThreadId(1) core.rs:42 内核已启动"
      );
  }

  #[test]
  fn 缺少文件行号时省略位置前缀() {
      let line = format_log_line(
          "2026-10-30T23:06:12.345",
          log::Level::Error,
          "ThreadId(2)",
          None,
          None,
          "失败",
      );

      assert_eq!(line, "2026-10-30T23:06:12.345 ERROR ThreadId(2) 失败");
  }

  #[test]
  fn 记录格式化会带上文件行号与级别() {
      let args = format_args!("内核已启动");
      let record = log::Record::builder()
          .args(args)
          .level(log::Level::Info)
          .target("sing_box_windows::app")
          .file(Some("core.rs"))
          .line(Some(42))
          .build();

      let line = format_record_line(&record);

      assert!(line.contains(" INFO "));
      assert!(line.contains("core.rs:42"));
      assert!(line.ends_with("内核已启动"));
  }
  ```
- [ ] 1.4 运行测试确认失败（模块尚不存在）：
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml log_bridge
  ```
  期望：编译错误 `could not find log_bridge in super`（或 `unresolved import`）
- [ ] 1.5 新建 `src-tauri/src/utils/log_bridge.rs`，实现：
  ```rust
  //! devtools 模式下的日志桥接。
  //!
  //! `tauri-plugin-devtools` 会独占全局 tracing subscriber（`Builder::try_init` 内部
  //! 调用 `tracing_subscriber::registry().with(...).try_init()`，失败即 panic）。
  //! 因此 devtools 模式下由插件持有全局 subscriber，本模块负责把事件转成
  //! 「文件 + 控制台」输出，通过官方 `Builder::attach_logger` 挂载回插件。

  use chrono::Local;

  /// 默认日志指令串，与 `log_util::build_env_filter` 的缺省值保持一致
  pub const DEFAULT_LOG_DIRECTIVES: &str = "debug,sing_box_windows=debug,tauri=info,sqlx=warn";

  /// 单条日志指令：`target=level` 或仅 `level`
  #[derive(Debug, Clone, PartialEq, Eq)]
  struct Directive {
      target_prefix: Option<String>,
      level: log::LevelFilter,
  }

  /// RUST_LOG 风格指令过滤器
  ///
  /// 匹配规则（简化自 `tracing_subscriber::EnvFilter`）：
  /// - 命中 target 前缀的指令中，**最长前缀**优先
  /// - 没有命中时回落到「无 target 的全局指令」
  /// - 没有任何可用指令时一律不放行
  #[derive(Debug, Clone, Default)]
  pub struct LogDirectiveFilter {
      directives: Vec<Directive>,
  }

  impl LogDirectiveFilter {
      /// 解析逗号分隔的指令串；非法级别指令被忽略
      pub fn parse(spec: &str) -> Self {
          let directives = spec
              .split(',')
              .filter_map(|item| {
                  let item = item.trim();
                  if item.is_empty() {
                      return None;
                  }
                  match item.split_once('=') {
                      Some((target, level)) => {
                          let level = parse_level_filter(level)?;
                          let target = target.trim();
                          if target.is_empty() {
                              None
                          } else {
                              Some(Directive {
                                  target_prefix: Some(target.to_string()),
                                  level,
                              })
                          }
                      }
                      None => {
                          let level = parse_level_filter(item)?;
                          Some(Directive {
                              target_prefix: None,
                              level,
                          })
                      }
                  }
              })
              .collect();

          Self { directives }
      }

      /// 读取 `RUST_LOG`；未设置或为空白时使用默认指令串
      pub fn from_env() -> Self {
          let spec = std::env::var("RUST_LOG")
              .ok()
              .filter(|value| !value.trim().is_empty())
              .unwrap_or_else(|| DEFAULT_LOG_DIRECTIVES.to_string());
          Self::parse(&spec)
      }

      /// 判断某条日志是否应当输出
      pub fn enabled(&self, target: &str, level: log::Level) -> bool {
          let allowed = self.level_for(target);
          match allowed {
              Some(filter) => filter >= level,
              None => false,
          }
      }

      /// 解析目标生效的级别：最长前缀优先，其次全局指令
      fn level_for(&self, target: &str) -> Option<log::LevelFilter> {
          let mut matched: Option<&Directive> = None;
          for directive in &self.directives {
              let Some(prefix) = directive.target_prefix.as_deref() else {
                  continue;
              };
              if !target.starts_with(prefix) {
                  continue;
              }
              let better = match matched {
                  None => true,
                  Some(current) => {
                      prefix.len()
                          > current.target_prefix.as_deref().map(str::len).unwrap_or(0)
                  }
              };
              if better {
                  matched = Some(directive);
              }
          }

          if let Some(directive) = matched {
              return Some(directive.level);
          }

          self.directives
              .iter()
              .find(|directive| directive.target_prefix.is_none())
              .map(|directive| directive.level)
      }
  }

  fn parse_level_filter(raw: &str) -> Option<log::LevelFilter> {
      match raw.trim().to_ascii_lowercase().as_str() {
          "off" => Some(log::LevelFilter::Off),
          "error" => Some(log::LevelFilter::Error),
          "warn" => Some(log::LevelFilter::Warn),
          "info" => Some(log::LevelFilter::Info),
          "debug" => Some(log::LevelFilter::Debug),
          "trace" => Some(log::LevelFilter::Trace),
          _ => None,
      }
  }

  /// 按 fmt 层风格拼装一行日志：`时间 级别 线程 文件:行号 消息`
  pub fn format_log_line(
      timestamp: &str,
      level: log::Level,
      thread: &str,
      file: Option<&str>,
      line: Option<u32>,
      message: &str,
  ) -> String {
      let location = match (file, line) {
          (Some(file), Some(line)) => format!(" {file}:{line}"),
          (Some(file), None) => format!(" {file}"),
          _ => String::new(),
      };

      format!("{timestamp} {level:>5} {thread}{location} {message}")
  }

  /// 把 `log::Record` 格式化成一行日志
  pub fn format_record_line(record: &log::Record) -> String {
      let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
      let thread = format!("{:?}", std::thread::current().id());

      format_log_line(
          &timestamp,
          record.level(),
          &thread,
          record.file(),
          record.line(),
          &record.args().to_string(),
      )
  }

  #[cfg(test)]
  #[path = "log_bridge.tests.rs"]
  mod tests;
  ```
  > 注：时间戳必须用 `chrono::Local`（项目已依赖 chrono 0.4，默认含 `clock` 特性）——`std::time::Local` 不支持 `%.3f` 这类 strftime 语法，无法产出毫秒精度时间戳。
- [ ] 1.6 运行测试确认通过：
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml log_bridge
  ```
  期望：9 个用例全部 PASS
- [ ] 1.7 格式与静态检查：
  ```bash
  cargo fmt --manifest-path src-tauri/Cargo.toml --all
  cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
  ```
- [ ] 1.8 提交：
  ```bash
  git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/utils/log_bridge.rs src-tauri/src/utils/log_bridge.tests.rs src-tauri/src/utils/mod.rs
  git commit -m "feat: 新增日志桥接模块（RUST_LOG 指令过滤 + 行格式化）"
  ```

---

## Task 2: 拆分 log_util，抽出可复用的日志 sink

**Files:**
- Modify: `src-tauri/src/utils/log_util.rs`
- Create: `src-tauri/src/utils/log_util.tests.rs`

**Produces:**
- `pub struct LogSink`：`pub fn dir(&self) -> &Path`、`pub fn writer(&self) -> &NonBlocking`
- `pub fn create_log_sink_in(dir: PathBuf) -> Option<(LogSink, WorkerGuard)>`（测试用，guard 交调用方持有）
- `pub fn create_log_sink() -> Option<LogSink>`（默认目录；guard 存入模块级 `OnceLock`；目录记入模块级 `OnceLock`）
- `pub fn current_log_dir() -> Option<PathBuf>`
- `pub fn install_direct_subscriber(sink: Option<&LogSink>)`（直连模式安装全局 fmt subscriber）

**背景约束（务必遵守）:**
- `tracing_appender::non_blocking::NonBlocking` 的 `flush()` 是**空操作**，真正刷盘要靠 **drop `WorkerGuard`**（它会通知 worker 退出并 join）→ 测试必须显式 drop guard 后再读文件
- `NonBlocking` 默认是 **lossy** 模式，队列满会丢行（与现状一致，允许丢）
- `NonBlocking` 是 `Clone + Send + Sync`，`MakeWriter` 会返回克隆 → `log::Log::log(&self, …)` 里可克隆 writer 后写入

**Steps:**

- [ ] 2.1 新建 `src-tauri/src/utils/log_util.tests.rs`：
  ```rust
  use super::*;
  use std::io::Write;

  #[test]
  fn sink 写入的内容会落盘() {
      let dir = std::env::temp_dir().join(format!("sbw-log-sink-{}", std::process::id()));
      let _ = std::fs::remove_dir_all(&dir);

      let (sink, guard) = create_log_sink_in(dir.clone()).expect("应创建日志 sink");
      {
          let mut writer = sink.writer().clone();
          writer.write_all(b"hello bridge\n").expect("写入应成功");
      }
      // NonBlocking::flush 是空操作，必须 drop guard 让 worker 退出并刷盘
      drop(guard);

      let file = dir.join(format!("{}.log", crate::app::log::DEFAULT_FILE_PREFIX));
      let content = std::fs::read_to_string(&file).expect("日志文件应存在");

      assert!(content.contains("hello bridge"));
      let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn sink 会暴露日志目录() {
      let dir = std::env::temp_dir().join(format!("sbw-log-dir-{}", std::process::id()));
      let _ = std::fs::remove_dir_all(&dir);

      let (sink, guard) = create_log_sink_in(dir.clone()).expect("应创建日志 sink");

      assert_eq!(sink.dir(), dir.as_path());
      drop(guard);
      let _ = std::fs::remove_dir_all(&dir);
  }
  ```
- [ ] 2.2 运行测试确认失败：
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml log_util
  ```
  期望：编译错误（`create_log_sink_in` / `LogSink` 未定义）
- [ ] 2.3 改造 `src-tauri/src/utils/log_util.rs`：
  1. 新增模块级状态：
     ```rust
     static FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
     static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
     ```
  2. 新增类型与函数（保留原有 `prepare_log_dir` / `create_file_writer` / `spawn_log_cleanup_task` / `cleanup_once` / `perform_cleanup` / `build_env_filter` / `init_console_only` 不变）：
     ```rust
     /// 日志 sink：非阻塞文件 writer + 所属目录
     pub struct LogSink {
         writer: NonBlocking,
         dir: PathBuf,
     }

     impl LogSink {
         /// 日志目录
         pub fn dir(&self) -> &Path {
             &self.dir
         }

         /// 非阻塞文件 writer
         pub fn writer(&self) -> &NonBlocking {
             &self.writer
         }
     }

     /// 在指定目录创建日志 sink；guard 交调用方持有以保证退出前刷盘
     pub fn create_log_sink_in(dir: PathBuf) -> Option<(LogSink, WorkerGuard)> {
         let writer = create_file_writer(&dir).ok()?;
         Some((LogSink { writer: writer.0, dir }, writer.1))
     }

     /// 在默认工作目录创建日志 sink；guard 存入模块级 OnceLock 保证进程存活期写入
     pub fn create_log_sink() -> Option<LogSink> {
         let dir = prepare_log_dir().ok()?;
         let (sink, guard) = create_log_sink_in(dir.clone())?;
         let _ = FILE_GUARD.set(guard);
         let _ = LOG_DIR.set(dir);
         Some(sink)
     }

     /// 当前日志目录（供定时清理任务使用）
     pub fn current_log_dir() -> Option<PathBuf> {
         LOG_DIR.get().cloned()
     }

     /// 直连模式：安装全局 fmt subscriber（文件 + 控制台）
     ///
     /// devtools 模式下**不能**调用本函数——全局 subscriber 已被插件占用，
     /// 再次设置会失败（`tracing_subscriber` 的全局默认只能设置一次）。
     pub fn install_direct_subscriber(sink: Option<&LogSink>) {
         let env_filter = build_env_filter();

         let Some(sink) = sink else {
             init_console_only(env_filter);
             return;
         };

         let combined_writer = sink.writer().clone().and(io::stdout);
         if let Err(e) = fmt()
             .with_env_filter(env_filter)
             .with_target(false)
             .with_thread_ids(true)
             .with_file(true)
             .with_line_number(true)
             .with_writer(combined_writer)
             .try_init()
         {
             eprintln!("初始化日志订阅器失败，回退到控制台日志: {e}");
             init_console_only(build_env_filter());
         }
     }
     ```
  3. 把原 `init_logging()` 改名为 `install_direct_subscriber` 的调用方（或直接删除 `init_logging`，由 `lib.rs` 改为调用 `create_log_sink()` + `install_direct_subscriber()`）；**`lib.rs` 的接线改动放在 Task 3**，本任务只保证 `log_util` 自身可编译通过（临时把 `lib.rs` 里的 `log_util::init_logging()` 调用改成等价的两行即可，或保留 `init_logging` 作为薄封装直到 Task 3 收口）
  4. 文件末尾挂载测试：
     ```rust
     #[cfg(test)]
     #[path = "log_util.tests.rs"]
     mod tests;
     ```
- [ ] 2.4 运行测试确认通过：
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml log_util
  ```
  期望：2 个用例 PASS
- [ ] 2.5 全量后端测试与静态检查：
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml
  cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
  cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
  ```
- [ ] 2.6 提交：
  ```bash
  git add src-tauri/src/utils/log_util.rs src-tauri/src/utils/log_util.tests.rs src-tauri/src/lib.rs
  git commit -m "refactor: 拆分 log_util，抽出可复用的日志 sink 与直连 subscriber 安装入口"
  ```

---

## Task 3: 接入 devtools 插件（cargo feature + 插件装配 + lib.rs 接线）

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/utils/mod.rs`
- Modify: `src-tauri/src/utils/log_bridge.rs`（追加 `FileConsoleLogger`）
- Modify: `src-tauri/src/utils/log_bridge.tests.rs`（追加桥接落盘用例）
- Create: `src-tauri/src/utils/devtools_util.rs`
- Modify: `src-tauri/src/lib.rs`

**Produces:**
- `log_bridge::FileConsoleLogger`：`pub fn new(writer: NonBlocking, filter: LogDirectiveFilter) -> Self`，实现 `log::Log`
- `devtools_util::init(sink: Option<LogSink>) -> (tauri::plugin::TauriPlugin<tauri::Wry>, tauri::plugin::TauriPlugin<tauri::Wry>)`

**Steps:**

- [ ] 3.1 `src-tauri/Cargo.toml` 三处改动：
  1. `rust-version = "1.77.2"` → `rust-version = "1.88"`
  2. `[dependencies]` 新增：
     ```toml
     tauri-plugin-devtools = { version = "2.2.3", optional = true }
     tauri-plugin-devtools-app = { version = "2.1.4", optional = true }
     ```
  3. 新增 `[features]` 段（放在 `[dependencies]` 之前）：
     ```toml
     [features]
     default = []
     devtools = ["dep:tauri-plugin-devtools", "dep:tauri-plugin-devtools-app", "tauri/dynamic-acl"]
     ```
- [ ] 3.2 在 `src-tauri/src/utils/mod.rs` 新增（按现有字母序插在 `config_util` 之后）：
  ```rust
  #[cfg(feature = "devtools")]
  pub mod devtools_util;
  ```
- [ ] 3.3 在 `log_bridge.rs` 追加 `FileConsoleLogger`：
  ```rust
  use tracing_appender::non_blocking::NonBlocking;

  /// devtools 模式下的日志写入器
  ///
  /// 插件的 `BridgeLayer` 会把 tracing 事件转成 `log::Record` 转发到这里，
  /// 因此文件与控制台输出与 DevTools 面板的数据同源。
  pub struct FileConsoleLogger {
      writer: NonBlocking,
      filter: LogDirectiveFilter,
  }

  impl FileConsoleLogger {
      /// 用非阻塞文件 writer + 过滤器构造
      pub fn new(writer: NonBlocking, filter: LogDirectiveFilter) -> Self {
          Self { writer, filter }
      }
  }

  impl log::Log for FileConsoleLogger {
      fn enabled(&self, metadata: &log::Metadata) -> bool {
          self.filter.enabled(metadata.target(), metadata.level())
      }

      fn log(&self, record: &log::Record) {
          if !self.enabled(record.metadata()) {
              return;
          }

          let line = format_record_line(record);
          let mut writer = self.writer.clone();
          let _ = writer.write_all(line.as_bytes());
          let _ = writer.write_all(b"\n");
          println!("{line}");
      }

      fn flush(&self) {
          let mut writer = self.writer.clone();
          let _ = writer.flush();
      }
  }
  ```
  （`use std::io::Write;` 需要加到文件头部）
- [ ] 3.4 在 `log_bridge.tests.rs` 追加桥接落盘用例：
  ```rust
  #[test]
  fn 桥接写入器会把日志写到文件与标准输出() {
      let dir = std::env::temp_dir().join(format!("sbw-bridge-{}", std::process::id()));
      let _ = std::fs::remove_dir_all(&dir);

      let (sink, guard) = crate::utils::log_util::create_log_sink_in(dir.clone())
          .expect("应创建日志 sink");
      let logger = FileConsoleLogger::new(
          sink.writer().clone(),
          LogDirectiveFilter::parse("info,sing_box_windows=debug"),
      );

      let args = format_args!("桥接输出");
      let record = log::Record::builder()
          .args(args)
          .level(log::Level::Info)
          .target("sing_box_windows::app")
          .file(Some("core.rs"))
          .line(Some(7))
          .build();
      logger.log(&record);

      drop(guard);

      let file = dir.join(format!("{}.log", crate::app::log::DEFAULT_FILE_PREFIX));
      let content = std::fs::read_to_string(&file).expect("日志文件应存在");
      assert!(content.contains("桥接输出"));
      let _ = std::fs::remove_dir_all(&dir);
  }

  #[test]
  fn 桥接写入器遵守过滤指令() {
      let logger = FileConsoleLogger::new(
          crate::utils::log_util::create_log_sink_in(std::env::temp_dir()).unwrap().0.writer().clone(),
          LogDirectiveFilter::parse("warn"),
      );

      let args = format_args!("不该出现");
      let record = log::Record::builder()
          .args(args)
          .level(log::Level::Info)
          .target("any")
          .build();

      assert!(!logger.enabled(record.metadata()));
  }
  ```
- [ ] 3.5 运行桥接测试：
  ```bash
  cargo test --manifest-path src-tauri/Cargo.toml log_bridge
  ```
  期望：11 个用例 PASS（`log_util::create_log_sink_in` 为 `pub`，需为 `pub` 可见性）
- [ ] 3.6 新建 `src-tauri/src/utils/devtools_util.rs`：
  ```rust
  //! CrabNebula DevTools 插件装配。
  //!
  //! 仅在 cargo feature `devtools` 开启时编译（见 `src-tauri/Cargo.toml`）。
  //! 插件必须在安装任何全局 tracing subscriber **之前**初始化：
  //! `tauri_plugin_devtools::Builder::try_init` 会调用
  //! `tracing_subscriber::registry().with(...).try_init()` 抢占全局 subscriber，
  //! 已被占用时 `init()` 会 panic。

  use crate::utils::log_bridge::{FileConsoleLogger, LogDirectiveFilter};
  use crate::utils::log_util::LogSink;
  use tauri::plugin::TauriPlugin;

  /// 初始化 devtools 插件与 devtools-app（Embedded 形态）插件
  ///
  /// 返回 `(probe 插件, app 插件)`，调用方负责 `.plugin(...)` 注册。
  pub fn init(sink: Option<LogSink>) -> (TauriPlugin<tauri::Wry>, TauriPlugin<tauri::Wry>) {
      let logger = sink.map(|sink| {
          let filter = LogDirectiveFilter::from_env();
          Box::new(FileConsoleLogger::new(sink.writer().clone(), filter)) as Box<dyn log::Log>
      });

      let mut builder = tauri_plugin_devtools::Builder::default();
      if let Some(logger) = logger {
          builder.attach_logger(logger);
      }

      let probe = builder.init();
      let app = tauri_plugin_devtools_app::init();

      (probe, app)
  }
  ```
- [ ] 3.7 改造 `src-tauri/src/lib.rs` 的 `run()`（**顺序是关键**）：

  把原来的
  ```rust
      // 初始化日志，支持文件输出 + 定期清理
      let log_dir = log_util::init_logging();

      tauri::Builder::default()
  ```
  替换为
  ```rust
      // 准备日志 sink（目录 + 非阻塞文件 writer），devtools 与直连模式共用
      let log_sink = log_util::create_log_sink();

      // devtools 模式：插件接管全局 tracing subscriber，日志经 log::Log 桥接输出
      #[cfg(all(feature = "devtools", debug_assertions))]
      let devtools_plugins = devtools_util::init(log_sink);

      // 直连模式：自行安装全局 fmt subscriber（release / 未启用 devtools feature）
      #[cfg(not(all(feature = "devtools", debug_assertions)))]
      log_util::install_direct_subscriber(log_sink.as_ref());

      // 日志目录（供定时清理任务使用）
      let log_dir = log_util::current_log_dir();

      let mut builder = tauri::Builder::default();
      #[cfg(all(feature = "devtools", debug_assertions))]
      {
          builder = builder
              .plugin(devtools_plugins.0)
              .plugin(devtools_plugins.1);
      }
      builder
  ```
  并把 `lib.rs` 顶部补上 `#[cfg(feature = "devtools")] use crate::utils::devtools_util;`（或直接用全路径 `crate::utils::devtools_util::init`）
- [ ] 3.8 编译验证（带 feature）：
  ```bash
  cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features devtools -- -D warnings
  cargo test --manifest-path src-tauri/Cargo.toml --features devtools
  ```
- [ ] 3.9 断言 release 依赖图不含插件（Review Focus #4）：
  ```bash
  cargo tree --manifest-path src-tauri/Cargo.toml -e normal | grep -c "tauri-plugin-devtools" || echo "OK: release 无插件"
  cargo tree --manifest-path src-tauri/Cargo.toml -e normal --features devtools | grep -c "tauri-plugin-devtools"
  ```
  期望：第一条输出 `OK: release 无插件`（或计数 0），第二条计数 ≥ 1
- [ ] 3.10 提交：
  ```bash
  git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/utils/mod.rs src-tauri/src/utils/log_bridge.rs src-tauri/src/utils/log_bridge.tests.rs src-tauri/src/utils/devtools_util.rs src-tauri/src/lib.rs
  git commit -m "feat: 接入 CrabNebula DevTools（cargo feature 隔离 + 日志桥接接线）"
  ```

---

## Task 4: dev 命令自动注入 devtools feature

**Files:**
- Modify: `scripts/tauri-wrapper.mjs`
- Modify: `scripts/tauri-wrapper.test.mjs`

**Produces:** `export function applyDevtoolsFeature(args)` —— 仅对 `dev` 命令、且用户未显式指定 `--features` 时注入 `--features devtools`

**Steps:**

- [ ] 4.1 在 `scripts/tauri-wrapper.mjs` 新增导出函数（放在 `applyAliasArgs` 之后）：
  ```js
  /**
   * dev 命令默认启用 devtools cargo feature（CrabNebula DevTools 调试插件）。
   * build 系列命令不注入，保证 release 产物不含插件、也不打开 tauri/tracing 埋点。
   */
  export function applyDevtoolsFeature(args) {
    const command = args[0] ?? ''
    if (command !== 'dev') {
      return args
    }
    if (hasOption(args, '--features', '-f')) {
      return args
    }
    return [args[0], '--features', 'devtools', ...args.slice(1)]
  }
  ```
  并在 `main()` 中接线（`let args = applyAliasArgs(rawArgs)` 之后立刻加一行）：
  ```js
  let args = applyDevtoolsFeature(applyAliasArgs(rawArgs))
  ```
- [ ] 4.2 在 `scripts/tauri-wrapper.test.mjs` 追加用例（并在文件头的 import 中加入 `applyDevtoolsFeature`）：
  ```js
  test('applyDevtoolsFeature 只对 dev 命令注入 devtools feature', () => {
    assert.deepEqual(applyDevtoolsFeature(['dev']), ['dev', '--features', 'devtools'])
    assert.deepEqual(applyDevtoolsFeature(['build']), ['build'])
    assert.deepEqual(applyDevtoolsFeature(['build:windows:arm64']), ['build:windows:arm64'])
  })

  test('applyDevtoolsFeature 不覆盖用户显式指定的 features', () => {
    const args = applyDevtoolsFeature(['dev', '--features', 'custom'])

    assert.deepEqual(args, ['dev', '--features', 'custom'])
  })

  test('applyDevtoolsFeature 保留 dev 命令的其他参数', () => {
    const args = applyDevtoolsFeature(['dev', '--no-watch'])

    assert.deepEqual(args, ['dev', '--features', 'devtools', '--no-watch'])
  })
  ```
- [ ] 4.3 运行测试：
  ```bash
  node --test scripts/tauri-wrapper.test.mjs
  ```
  期望：全部 PASS
- [ ] 4.4 提交：
  ```bash
  git add scripts/tauri-wrapper.mjs scripts/tauri-wrapper.test.mjs
  git commit -m "feat: dev 命令自动启用 devtools feature"
  ```

---

## Task 5: CI 门禁与文档同步

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/CHANGELOG.md`
- Modify: `AGENTS.md`
- Modify: `src-tauri/AGENTS.md`

**Steps:**

- [ ] 5.1 `.github/workflows/ci.yml`：给 Rust 门禁加上 `--features devtools`，保证 devtools 代码路径持续可编译（否则长期腐化）
  ```yaml
      - name: Rust clippy
        run: cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features devtools -- -D warnings

      - name: Rust test
        run: cargo test --manifest-path src-tauri/Cargo.toml --features devtools
  ```
- [ ] 5.2 `docs/CHANGELOG.md`：在 `[未发布]` 段新增小节（放在 `### ✨ 新功能` 之后）
  ```markdown
  ### 🛠️ 开发体验

  - **开发态接入 CrabNebula DevTools**：通过 cargo feature `devtools` 隔离（release 产物零影响），开发态可用可视化面板查看日志流、Tauri command/event 的 payload 与耗时。日志输出经官方 `attach_logger` 桥接，终端与文件日志保持不变。`pnpm tauri dev` 自动启用；面板地址在启动日志中打印（standalone 网页形态），应用内 `Ctrl+Shift+M` 可打开 Embedded 形态
  ```
- [ ] 5.3 根 `AGENTS.md`：在 COMMANDS 段补充说明
  ```markdown
  # 开发（dev 自动启用 devtools feature：CrabNebula DevTools 调试面板）
  pnpm install            # 首次
  pnpm kernel:fetch       # 首次构建前必须；--all 为全平台（stable/oldstable/beta/testing 轨道）
  pnpm tauri dev
  ```
  并在 NOTES 段补一条：
  ```markdown
  - **调试面板**：`pnpm tauri dev` 会自动加 `--features devtools`（`src-tauri/Cargo.toml` 的 `devtools` feature），启用 CrabNebula DevTools；release 构建不带该 feature，产物不含插件。
  ```
- [ ] 5.4 `src-tauri/AGENTS.md`：WHERE TO LOOK 表格新增两行，NOTES 段新增一条
  ```markdown
  | 日志桥接（devtools） | `src/utils/log_bridge.rs` | RUST_LOG 过滤 + 行格式化 + `log::Log` 适配器 |
  | devtools 插件装配   | `src/utils/devtools_util.rs` | 仅 `devtools` feature 编译；必须早于全局 subscriber 安装 |
  ```
  ```markdown
  - **全局 tracing subscriber 只能设置一次**：`tauri-plugin-devtools` 的 `Builder::try_init` 会抢占全局 subscriber（已占用则 `init()` panic）。`lib.rs::run()` 中 devtools 插件先于 fmt 层初始化；`log_util::install_direct_subscriber` 仅在非 devtools 模式调用。
  ```
- [ ] 5.5 全量门禁自检：
  ```bash
  cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
  cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features devtools -- -D warnings
  cargo test --manifest-path src-tauri/Cargo.toml --features devtools
  node --test scripts/tauri-wrapper.test.mjs
  node --test scripts/kernel-targets.test.mjs
  pnpm lint
  pnpm type-check
  pnpm format:check
  ```
- [ ] 5.6 提交：
  ```bash
  git add .github/workflows/ci.yml docs/CHANGELOG.md AGENTS.md src-tauri/AGENTS.md
  git commit -m "ci: devtools 门禁与文档同步"
  ```

---

## Task 6: 端到端手工验证

**前置:** 已执行 `pnpm kernel:fetch`（首次构建前必须）

**Steps:**

- [ ] 6.1 启动开发态：
  ```bash
  pnpm tauri dev
  ```
  期望：
  - 终端出现 devtools 的 dash 链接，形如 `https://devtools.crabnebula.dev/dash/127.0.0.1/<port>`
  - 应用正常启动，无 panic
  - 终端仍能看到 `tracing` 日志（INFO 级别行），说明桥接生效
- [ ] 6.2 浏览器打开终端打印的链接，确认：
  - 日志流实时出现（能看到本项目 `sing_box_windows::...` 的日志）
  - Tauri command 的 payload / 响应 / 耗时可见
  - Tauri event 的 payload 可见
- [ ] 6.3 端口核对（Review Focus #3）：把链接里的端口与 `netstat -ano | findstr :<port>` 的监听状态对照，确认链接端口 = 实际监听端口
- [ ] 6.4 Embedded 形态：在应用窗口右键 → 「Open DevTools」，或按 `Ctrl+Shift+M`，确认抽屉面板可打开（需外网加载 UI；离线时此步失败属预期，standalone 形态不受影响）
- [ ] 6.5 文件日志核对：确认工作目录 `logs/app.log` 仍有内容且包含 devtools 模式下产生的日志
- [ ] 6.6 关闭应用，确认无残留进程、无 panic 栈
- [ ] 6.7 release 隔离核对：
  ```bash
  cargo build --release --manifest-path src-tauri/Cargo.toml
  ```
  期望：编译通过且日志走直连 fmt 层（终端输出格式为 `时间 级别 线程 文件:行号 消息`，与桥接模式一致）
- [ ] 6.8 如有偏差，回到对应任务修正后重跑；全部通过后提交验证记录（可选写入 `docs/CHANGELOG.md` 同一条目）

---

## 附：任务依赖关系

```
Task 1 (log_bridge 纯逻辑)
   └─▶ Task 3 (插件接入) ──┐
Task 2 (log_util sink) ───┘   └─▶ Task 5 (CI + 文档)
Task 4 (wrapper 注入) ─────────────▶ Task 5
                                     └─▶ Task 6 (端到端)
```

Task 1 与 Task 2 可并行；Task 3 依赖两者；Task 4 与 Task 3 无耦合，可并行。

---

## 执行记录（2026-10-30）

### 提交

| commit | 内容 |
| --- | --- |
| `dcaded4` | Task 1：日志桥接模块（RUST_LOG 指令过滤 + 行格式化） |
| `e0e055c` | Task 2：拆分 log_util，抽出可复用的日志 sink 与直连 subscriber 安装入口 |
| `7b257e0` | Task 3：接入 devtools 插件（cargo feature 隔离 + 插件装配 + lib.rs 接线） |
| `ace37ba` | Task 4：dev 命令自动启用 devtools feature |
| `413e822` | Task 5：CI 门禁与文档同步 |
| `e2f0503` | e2e 修复：桥接日志行粘连（行内容与换行一次性写入） |
| `9aaaf33` | 评审修复批：过滤语义对齐 EnvFilter、UTC 时间戳、初始化失败可降级、CI 双路径校验 |

### Rulings（执行中的判断，均已记入提交信息或本表）

- 测试函数命名用**英文 snake_case**（仓库既有约定，如 `parse_uri_list_vless_trojan`），计划草稿里的中文用例名作废
- 计划里 `longest_prefix_wins` 的期望自相矛盾（spec `sing_box_windows::app=trace` 已允许 Debug），按「最长前缀优先」修正
- `create_log_sink_in` 返回 `io::Result` 而非 `Option`：保留原「文件日志初始化失败，使用控制台日志: {e}」的错误信息
- `lib.rs` 用 `let builder = builder.plugin(..)` 遮蔽而非 `let mut builder`：否则非 devtools 构建触发 `unused_mut`（`-D warnings` 直接失败）
- MSRV 1.77.2 → 1.88 后 clippy 的 `chunks_exact_to_as_chunks` 开始生效 → 连带修 `src/app/tray/icon.rs`（语义等价，模块自带单测覆盖）
- 滚动日志文件名带日期后缀（`app.log.YYYY-MM-DD`），测试按目录扫描前缀而非写死文件名

### 端到端验证证据

- wrapper 注入生效：`cargo run --no-default-features --features devtools`
- 插件初始化成功并打印面板地址：`Tauri Devtools v2.2.3` / `Tauri Devtools App v2.1.4` → `https://devtools.crabnebula.dev/dash/127.0.0.1/3033`
- 桥接生效：终端与文件日志均为桥接格式（UTC 微秒时间戳 + `Z`、级别右对齐 5、`file:line` 齐全）
- 面板数据通路：`rs.devtools.instrument.Instrument/WatchUpdates` 流 10 秒回传数百 KB，含本项目日志帧与本项目模块 span（`app_lib::app::core::kernel_service::event` / `guard` / `proxy_service::status` / `singbox_api::client`）及 Tauri 内部 span（`app::plugin::register` / `plugin::hooks::initialize`）
- release 隔离：`cargo tree -e normal` 不带 feature → `tauri-plugin-devtools` 计数 0；带 feature → 3

### 评审结论与处置

fresh reviewer 给出 0 Critical、5 Important、10 Minor。已修：过滤语义与 `EnvFilter` 不一致（含非法/空 `RUST_LOG` 下两种模式相反）、CI 只编译 devtools 路径（release 分支失去编译期护栏）、wrapper 脚本测试从未进 CI、插件 init 失败即 panic 且 sink 失败时零日志、时间戳与直连模式不一致。Minor 中同步处理：终端写失败不再 panic、release+feature 组合给出提示、日志目录提示补齐、guard 重复初始化告警、文档与实现对齐。未处理并记档：`NonBlocking` 每次日志克隆 writer 的热路径开销（M-2）、`log_bridge` 未随 feature 门控以保单测可跑（M-10）。

### 既有失败（非本次引入，未修）

- `pnpm format:check` 报 14 个 `src/` 文件格式漂移（本次未触碰 `src/`）
- `scripts/fetch-kernel.test.mjs` 期望 `--all` 4 个目标、实际 5 个（目标矩阵已扩到 5 平台）；该文件不在任何 npm script / CI 门禁内
