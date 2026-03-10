# Sing-Box Windows 开发文档

## 项目概览

Sing-Box Windows 是一个跨平台桌面客户端，前端使用 Vue 3 + TypeScript，后端使用 Rust + Tauri 2，围绕 sing-box 内核提供订阅管理、代理模式切换、运行状态查看、托盘控制、备份恢复与应用更新等能力。

这份文档聚焦“当前仓库真实实现”，优先说明：

- 启动链路在哪里
- 前后端如何通信
- 关键模块如何分层
- 开发和发布应该走哪些命令
- 哪些地方最容易改坏

## 快速索引

- 前端入口：`src/main.ts`
- 前端业务引导：`src/App.vue`、`src/boot/useAppBootstrap.ts`
- Tauri 命令调用统一入口：`src/services/invoke-client.ts`
- Tauri 事件监听入口：`src/services/event-service.ts`
- 路由与托盘空白页：`src/router/index.ts`、`src/views/BlankView.vue`
- 后端入口与命令注册：`src-tauri/src/lib.rs`
- 订阅下载/解析/自动更新：`src-tauri/src/app/network/subscription_service/`
- 内核生命周期：`src-tauri/src/app/core/kernel_service/`
- 本地存储：`src-tauri/src/app/storage/`
- 后端托盘能力：`src-tauri/src/app/tray/`
- 内核拉取与打包包装：`scripts/fetch-kernel.mjs`、`scripts/tauri-wrapper.mjs`
- 发布流程：`.github/workflows/release.yml`、`.github/workflows/promote-release.yml`

## 技术栈

### 前端

- Vue 3
- TypeScript
- Pinia
- Vue Router
- Vue I18n
- Naive UI
- Vite

### 后端

- Rust 1.77.2+
- Tauri 2
- tokio
- reqwest
- sqlx + SQLite
- tracing

## 架构总览

### 前端启动链

当前启动链是“两阶段”：

1. `src/main.ts`
   - 创建 Vue 应用
   - 注册 Pinia、Router、I18n
   - 调用 `initializationService.initializeApp()`
   - 初始化完成后再挂载应用
2. `src/App.vue`
   - 挂载 Naive UI 全局 Provider
   - 建立全局通知、消息、弹窗容器
   - 调用 `useAppBootstrap()` 进入业务初始化
3. `src/boot/useAppBootstrap.ts`
   - 初始化各类 Store
   - 同步语言、主题、窗口状态
   - 绑定 Tauri event 监听
   - 初始化托盘状态与消息总线
   - 在窗口隐藏时切到 `/blank`，降低托盘驻留开销

### 前后端通信

当前项目是典型的“双通道通信”：

- **命令通道**：前端通过 `invoke` 调后端命令
- **事件通道**：后端通过 Tauri event 向前端推送运行态数据

对应落点如下：

- `src/services/invoke-client.ts`
  - 统一封装 Tauri `invoke`
  - 注入前端上下文
  - 控制是否跳过数据恢复等选项
- `src/services/event-service.ts`
  - 统一注册/清理事件监听
  - 给流量、日志、连接、更新、托盘动作等事件提供稳定入口
- `src/constants/events.ts`
  - 维护前后端共享的事件名常量

### 后端模块分层

`src-tauri/src/app/` 是后端主业务目录，当前可以按职责理解为：

- `core/`
  - 内核启停
  - 代理模式切换
  - 运行健康检查
  - 流量、规则、连接等运行态能力
- `network/`
  - 订阅下载
  - 订阅解析
  - 自动更新
  - 配置切换与回滚
- `storage/`
  - SQLite 初始化与迁移
  - 应用配置、主题、语言、窗口、更新设置持久化
  - 与配置文件同步
- `system/`
  - 平台能力
  - 应用更新
  - sudo / 提权
  - 备份恢复
- `tray/`
  - 后端主导的托盘菜单与窗口控制

### 关键页面

当前主要页面和职责如下：

- `src/views/HomeView.vue`：内核状态、启动/停止、流量概览
- `src/views/SubView.vue`：订阅管理、导入、更新、回滚
- `src/views/ProxyView.vue`：节点与代理模式切换
- `src/views/ConnectionsView.vue`：活跃连接查看与筛选
- `src/views/RulesView.vue`：规则命中与路由规则查看
- `src/views/LogView.vue`：日志查看与过滤
- `src/views/SettingView.vue`：应用设置、更新、内核、备份恢复
- `src/views/BlankView.vue`：托盘驻留时的空白路由

## 项目结构

```text
sing-box-windows/
├── src/
│   ├── boot/               # 前端业务引导
│   ├── components/         # 组件与布局
│   ├── constants/          # 常量与事件名
│   ├── locales/            # 多语言文案
│   ├── router/             # 路由定义
│   ├── services/           # 前端服务层
│   ├── stores/             # Pinia store
│   ├── types/              # 类型定义
│   ├── views/              # 页面
│   ├── App.vue             # 根组件
│   └── main.ts             # 前端入口
├── src-tauri/
│   ├── src/
│   │   ├── app/            # 后端业务主目录
│   │   ├── entity/         # 数据模型
│   │   ├── platform/       # 平台抽象
│   │   ├── process/        # 进程管理
│   │   ├── utils/          # 工具模块
│   │   └── lib.rs          # Tauri 注册入口
│   ├── tauri.conf.json     # Tauri 配置
│   └── Cargo.toml          # Rust 依赖与版本
├── scripts/                # 内核拉取与 tauri wrapper
├── docs/                   # 项目文档
└── .github/workflows/      # 发布工作流
```

## 开发环境与常用命令

### 推荐环境

- Node.js 20+
- pnpm 10+
- Rust 1.77.2+
- 能通过 Tauri 2 桌面环境依赖检查

### 首次安装

```bash
# 安装前端依赖
pnpm install

# 首次开发建议先拉取当前平台内核
pnpm kernel:fetch
```

### Linux 系统依赖

Ubuntu / Debian:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev \
  libappindicator3-dev librsvg2-dev
```

Fedora / RHEL 系:

```bash
sudo dnf install webkit2gtk4.1-devel openssl-devel curl wget file \
  libappindicator-gtk3-devel librsvg2-devel libxdo-devel
sudo dnf group install "c-development"
```

当前仓库的 Linux 打包已支持 `deb`、`rpm` 和 `AppImage`。在 Fedora 上本地构建 RPM 时，优先使用 wrapper 提供的目标别名，不要绕过 `scripts/tauri-wrapper.mjs`。

### 日常开发

```bash
# 启动桌面开发环境
pnpm tauri dev

# 前端类型检查
pnpm type-check

# 前端 lint
pnpm lint

# Rust 静态检查
cd src-tauri && cargo clippy

# Rust 测试
cd src-tauri && cargo test

# 脚本测试
pnpm test:kernel-targets
```

### 生产构建

```bash
# 构建桌面产物
pnpm tauri build

# 在 Linux x86_64 上仅构建 RPM
pnpm run tauri build:linux:rpm

# 等价写法：显式指定 Linux target + bundle 类型
pnpm run tauri build --target x86_64-unknown-linux-gnu --bundles rpm
```

当前项目统一要求通过 `scripts/tauri-wrapper.mjs` 触发 Tauri 构建，不要绕过 wrapper 直接修改资源打包逻辑。

## 内核资源拉取与打包链路

这是项目里最容易被文档写过时的一段，当前真实行为如下：

1. `pnpm tauri dev` / `pnpm tauri build`
2. 实际进入 `scripts/tauri-wrapper.mjs`
3. wrapper 会根据宿主平台或 `--target` 解析目标平台
4. 自动调用 `scripts/fetch-kernel.mjs`
5. 仅拉取目标平台对应的 sing-box 内核资源
6. 构建时生成 `src-tauri/.generated/tauri.kernel.<platform>.<arch>.conf.json`
7. 将目标平台资源注入 Tauri bundle，避免把多平台内核一起打进安装包

### 常用命令

```bash
# 拉取当前平台最新内核
pnpm kernel:fetch

# 拉取所有受支持平台的内核
pnpm kernel:fetch:all

# 强制重新拉取（通过环境变量让 wrapper 走强制模式）
# Windows PowerShell:
$env:SING_BOX_KERNEL_FETCH_MODE='force'; pnpm tauri build
```

### 相关环境变量

- `SING_BOX_KERNEL_FETCH_MODE=force`
  - 强制重新下载内核，而不是跳过已有资源
- `SING_BOX_GITHUB_TOKEN`
  - 给 GitHub API 请求加认证，降低 latest 拉取时被限流的概率

## 前端开发约定

### 命名和组织

- 组件使用 PascalCase
- 页面使用 `*View.vue`
- 组合式函数使用 `useXxx.ts`
- Store 文件使用 PascalCase，例如 `AppStore.ts`
- 不要使用 `as any`、`@ts-ignore`、`@ts-nocheck`

### 路由约定

- 路由定义在 `src/router/index.ts`
- `/blank` 不是普通页面，而是托盘驻留时的重要空白路由
- 改动托盘相关逻辑时，务必验证最小化、恢复、隐藏启动、退出流程

### 服务层约定

- 页面尽量通过 `stores/` 和 `services/` 与后端交互
- 不要在页面里分散写大量原始 `invoke`
- 新命令优先补到相应 service，再由页面调用

### 一个推荐写法

```ts
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { subscriptionService } from '@/services/subscription-service'

const { t } = useI18n()
const message = useMessage()

const refreshCurrentSubscription = async () => {
  try {
    // 把后端调用收敛到 service，页面只负责交互反馈。
    await subscriptionService.updateCurrentSubscription()
    message.success(t('sub.updateSuccess'))
  } catch (error) {
    // 错误提示统一走 UI 文案，避免页面散落硬编码字符串。
    message.error(String(error ?? t('common.operationFailed')))
  }
}
```

## 后端开发约定

### 命令约定

- Tauri 命令尽量返回 `Result<T, String>`
- 新命令加完后要在 `src-tauri/src/lib.rs` 注册
- 如果命令会影响运行时状态，考虑是否需要同步发 event 给前端

### 模块边界

- 内核启停与健康相关逻辑放 `core/`
- 订阅下载与解析放 `network/`
- 配置持久化放 `storage/`
- 平台、更新、备份、提权放 `system/`
- 托盘行为放 `tray/`

### 日志建议

- 使用 `tracing` 记录后端关键行为
- 错误信息尽量保留上下文，方便前端直接展示或定位
- 不要保留一次性的调试输出

### 启动阶段注意点

`src-tauri/src/lib.rs` 的 `setup` 阶段除了注册命令外，还会做异步初始化工作，例如：

- 存储初始化
- 残留进程清理
- 自动任务或运行态初始化
- 托盘与窗口协同

这里是高风险修改点，改动前建议先理解初始化顺序。

## 多语言开发

- 语言文件位于 `src/locales/`
- 当前内置：`zh-CN`、`en-US`、`ja-JP`、`ru-RU`
- `src/stores/app/LocaleStore.ts` 负责读取和保存语言偏好
- `src/locales/index.ts` 维护 `DEFAULT_LOCALE`、`LocaleCode` 和 `supportedLocales`

更详细的国际化说明请直接查看 `docs/i18n.md`。

## 数据存储

当前项目不是简单依赖浏览器 `localStorage`，核心配置由后端 SQLite 持久化。

默认工作目录：

- Windows：`%LOCALAPPDATA%\sing-box-windows\`
- Linux：`~/.local/share/sing-box-windows/`
- macOS：`~/Library/Application Support/sing-box-windows/`

这里通常会存放：

- SQLite 数据库
- 生成后的配置文件
- 日志文件
- 备份文件
- 已下载的内核与版本信息

## 发布流程

### 版本同步

发布前至少同步以下文件：

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `docs/CHANGELOG.md`

### 质量门禁

```bash
pnpm lint
pnpm type-check
cd src-tauri && cargo clippy && cargo test
pnpm test:kernel-targets
```

### 预发布

1. 提交版本变更
2. 创建并推送 `vX.Y.Z` tag
3. GitHub Actions 自动触发 `.github/workflows/release.yml`
4. CI 构建 Windows / Linux / macOS 产物
5. Release Notes 从 `docs/CHANGELOG.md` 对应版本节自动提取

### 正式发布

预发布验证通过后，手动触发 `.github/workflows/promote-release.yml`，把同一 tag 转为正式版。

## 常见改动点检查清单

### 改了前端页面

- 是否仍通过 service/store 访问后端
- 是否补齐多语言文案
- 是否验证亮暗主题
- 是否验证托盘隐藏/恢复后的行为

### 改了订阅解析

- 是否补充或更新 Rust 单测
- 是否验证 sing-box JSON / Clash YAML / URI 列表三类输入
- 是否确认手动配置与自动更新的边界行为

### 改了内核/代理逻辑

- 是否验证 system proxy / TUN / manual 三种模式
- 是否检查异常停止后的提示与恢复
- 是否确认连接、规则、日志页面还能正常刷新

### 改了发布与打包逻辑

- 是否仍通过 wrapper 注入目标平台内核
- 是否验证 `--target` 下资源映射正确
- 是否确认 Linux glibc 兼容性约束未被破坏

## 容易踩坑的点

- 不要把 `/blank` 当成普通页面删掉或弱化
- 不要绕过 `scripts/tauri-wrapper.mjs` 直接改资源打包
- 不要为了过类型检查使用类型压制
- 不要把多平台内核一起塞进单平台安装包
- `src-tauri/src/lib.rs` 当前存在重复 websocket plugin 注册风险，修改时先确认影响

## 参考文档

- `README.md`
- `README.zh.md`
- `docs/i18n.md`
- `docs/CHANGELOG.md`
