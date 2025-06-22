# Sing-Box Windows 开发文档

## 目录

- [项目概述](#项目概述)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [开发环境搭建](#开发环境搭建)
- [核心功能模块](#核心功能模块)
- [前端开发指南](#前端开发指南)
- [后端开发指南](#后端开发指南)
- [多语言开发指南](#多语言开发指南)
- [构建与发布](#构建与发布)
- [常见问题](#常见问题)

## 项目概述

Sing-Box Windows 是一个基于 [Sing-Box](https://github.com/SagerNet/sing-box) 代理内核的 Windows GUI 客户端，使用 Tauri 2.0 框架开发，旨在提供优雅、高效、易用的代理管理体验。

项目主要特点：

- 现代化用户界面，支持亮暗主题
- 多种代理模式（系统代理/TUN模式）
- 多种订阅格式支持
- 实时流量监控与统计
- 丰富的日志系统
- 规则分流功能

## 技术栈

### 前端技术

- **Vue 3 (v3.5.13)**：核心前端框架，使用 Composition API
- **TypeScript (v5.6.3)**：类型安全的 JavaScript 超集
- **Naive UI (v2.41.0)**：高质量 Vue 3 组件库，支持亮暗主题
- **Pinia (v2.3.1)**：Vue 状态管理库，支持持久化存储
- **Vue Router (v4.5.1)**：Vue 路由管理，支持Hash路由
- **Vue I18n (v9.14.4)**：国际化支持，多语言切换
- **VueUse (v12.8.2)**：Vue 实用工具集合
- **Vite (v6.3.5)**：快速构建工具

### 后端技术

- **Rust (1.77.2+)**：高性能系统编程语言
- **Tauri 2.0 (v2.5.0)**：构建跨平台应用的框架
- **tokio**：异步运行时
- **serde & serde_json**：序列化和反序列化
- **reqwest**：HTTP 客户端，支持TLS
- **tracing & tracing-subscriber**：结构化日志记录
- **tauri-plugin-store**：持久化存储插件
- **tauri-plugin-websocket**：WebSocket 支持

### 开发工具

- **pnpm**：快速、节省磁盘空间的包管理器
- **ESLint + Prettier**：代码质量和格式化
- **oxlint**：高性能 JavaScript/TypeScript linter
- **unplugin-auto-import**：自动导入 API
- **unplugin-vue-components**：组件自动导入

## 项目结构

项目采用模块化的目录结构，按功能划分不同的模块：

```
sing-box-windows/
├── src/                # 前端源代码
│   ├── assets/        # 静态资源
│   │   ├── base.css           # 基础样式
│   │   ├── main.css           # 主样式文件
│   │   ├── icon.png           # 应用图标
│   │   ├── logo.svg           # Logo 图标
│   │   └── naive-ui-theme-overrides.json # UI主题覆盖配置
│   ├── components/    # 通用组件
│   │   ├── home/             # 主页组件
│   │   │   ├── ProxyModeCard.vue     # 代理模式卡片
│   │   │   ├── StatusCard.vue        # 状态卡片
│   │   │   └── TrafficStatsCard.vue  # 流量统计卡片
│   │   ├── layout/           # 布局组件
│   │   │   ├── MainLayout.vue        # 主布局
│   │   │   └── TrafficChart.vue      # 流量图表
│   │   ├── LazyComponent.vue  # 懒加载组件
│   │   ├── UpdateModal.vue    # 更新弹窗
│   │   ├── UpdateNotification.vue # 更新通知
│   │   └── VirtualList.vue    # 虚拟列表组件
│   ├── composables/   # 组合式API
│   │   └── useVirtualization.ts # 虚拟化工具
│   ├── constants/     # 前端常量
│   │   └── index.ts   # 常量定义
│   ├── locales/       # 国际化文件
│   │   ├── zh-CN.ts   # 中文翻译
│   │   ├── en-US.ts   # 英文翻译
│   │   ├── ja-JP.ts   # 日文翻译
│   │   ├── ru-RU.ts   # 俄文翻译
│   │   └── index.ts   # i18n配置
│   ├── router/        # 路由配置
│   │   └── index.ts   # 路由定义
│   ├── services/      # 服务层
│   │   ├── notification-service.ts # 通知服务
│   │   ├── proxy-service.ts        # 代理服务
│   │   ├── tauri-api.ts           # Tauri API 封装
│   │   └── websocket-service.ts   # WebSocket 服务
│   ├── stores/        # Pinia 状态管理
│   │   ├── index.ts   # Store 主入口和插件配置
│   │   ├── StoreManager.ts # Store 管理器
│   │   ├── app/       # 应用相关 store
│   │   │   ├── AppStore.ts     # 核心应用状态
│   │   │   ├── ThemeStore.ts   # 主题管理
│   │   │   ├── LocaleStore.ts  # 国际化状态
│   │   │   ├── WindowStore.ts  # 窗口管理
│   │   │   └── UpdateStore.ts  # 更新状态
│   │   ├── kernel/    # 内核相关 store
│   │   │   ├── KernelStore.ts         # 内核状态
│   │   │   ├── KernelRuntimeStore.ts  # 内核运行时状态
│   │   │   ├── ProxyStore.ts          # 代理状态
│   │   │   ├── ConnectionStore.ts     # 连接管理
│   │   │   ├── TrafficStore.ts        # 流量统计
│   │   │   └── LogStore.ts            # 日志管理
│   │   ├── subscription/ # 订阅相关 store
│   │   │   └── SubStore.ts     # 订阅管理
│   │   └── tray/      # 系统托盘 store
│   │       └── TrayStore.ts    # 托盘状态管理
│   ├── types/         # TypeScript 类型定义
│   │   ├── api.ts     # API 类型
│   │   ├── index.d.ts # 全局类型声明
│   │   ├── models.ts  # 数据模型类型
│   │   └── process.ts # 进程相关类型
│   ├── utils/         # 工具函数
│   │   ├── index.ts   # 通用工具函数
│   │   ├── memory-leak-fix.ts # 内存泄漏修复
│   │   ├── mitt.ts    # 事件总线
│   │   └── mitt.d.ts  # mitt 类型声明
│   ├── views/         # 页面组件
│   │   ├── BlankView.vue      # 空白页面(托盘模式)
│   │   ├── HomeView.vue       # 主页
│   │   ├── ProxyView.vue      # 代理管理
│   │   ├── SubView.vue        # 订阅管理
│   │   ├── LogView.vue        # 日志查看
│   │   ├── SettingView.vue    # 设置页面
│   │   ├── RulesView.vue      # 规则管理
│   │   └── ConnectionsView.vue # 连接管理
│   ├── App.vue        # 根组件
│   ├── main.ts        # 应用入口
│   └── env.d.ts       # 环境类型声明
├── src-tauri/         # Rust 后端代码
│   ├── src/           # 源代码
│   │   ├── app/       # 应用服务层
│   │   │   ├── constants/      # 常量定义模块
│   │   │   │   ├── mod.rs        # 常量模块入口
│   │   │   │   ├── core.rs       # 核心常量(进程、路径)
│   │   │   │   ├── network.rs    # 网络常量(API、配置)
│   │   │   │   ├── system.rs     # 系统常量(注册表、数据库)
│   │   │   │   └── common.rs     # 通用常量(消息、日志)
│   │   │   ├── core/           # 核心服务模块
│   │   │   │   ├── kernel_service.rs  # 内核服务(启动/停止/管理)
│   │   │   │   ├── proxy_service.rs   # 代理服务(系统代理/TUN模式)
│   │   │   │   ├── task_manager.rs    # 任务管理器
│   │   │   │   └── mod.rs             # 核心模块入口
│   │   │   ├── network/        # 网络服务模块
│   │   │   │   ├── subscription_service.rs # 订阅服务
│   │   │   │   └── mod.rs              # 网络模块入口
│   │   │   ├── system/         # 系统服务模块
│   │   │   │   ├── system_service.rs   # 系统功能(权限/服务)
│   │   │   │   ├── update_service.rs   # 更新服务
│   │   │   │   ├── config_service.rs   # 配置服务
│   │   │   │   └── mod.rs              # 系统模块入口
│   │   │   └── mod.rs          # 应用模块入口
│   │   ├── entity/    # 数据实体模型
│   │   │   ├── config_model.rs # 配置数据模型
│   │   │   ├── github_model.rs # GitHub API模型
│   │   │   └── mod.rs          # 实体模块入口
│   │   ├── process/   # 进程管理模块
│   │   │   ├── manager.rs      # 进程管理器
│   │   │   ├── error.rs        # 进程错误处理
│   │   │   └── mod.rs          # 进程模块入口
│   │   ├── utils/     # 工具函数模块
│   │   │   ├── app_util.rs     # 应用工具
│   │   │   ├── config_util.rs  # 配置工具
│   │   │   ├── file_util.rs    # 文件工具
│   │   │   ├── http_client.rs  # HTTP客户端
│   │   │   ├── proxy_util.rs   # 代理工具
│   │   │   └── mod.rs          # 工具模块入口
│   │   ├── error.rs   # 全局错误处理
│   │   ├── lib.rs     # 库入口和命令注册
│   │   └── main.rs    # 程序主入口
│   ├── config/        # 配置模板
│   │   └── template.json # Sing-box配置模板
│   ├── Cargo.toml     # Rust 依赖配置
│   ├── build.rs       # 构建脚本
│   └── tauri.conf.json # Tauri 应用配置
├── docs/              # 项目文档
│   ├── development.md # 开发文档
│   ├── i18n.md       # 国际化文档
│   └── image.png     # 文档图片
├── public/            # 静态资源
│   └── favicon.ico   # 网站图标
├── auto-imports.d.ts  # 自动导入类型声明
├── components.d.ts    # 组件类型声明
├── package.json       # 前端依赖配置
├── vite.config.ts     # Vite 构建配置
├── tsconfig.json      # TypeScript 配置
└── README.zh.md       # 中文说明文档
```

这种模块化的结构有以下优点：

1. **功能划分清晰**：按功能将代码划分为核心、网络、系统等模块
2. **易于维护**：相关功能集中在一起，便于定位和修改
3. **降低耦合度**：每个模块都有明确的职责和边界
4. **方便扩展**：添加新功能时可以在相应模块中扩展，不影响其他模块

## 开发环境搭建

### 系统要求

- **操作系统**：Windows 10 1809+ (Build 17763+) 或更高版本
- **架构**：x64 (64位)
- **Rust**：1.77.2 或更高版本 (最新稳定版)
- **Node.js**：18.0+ (推荐使用 LTS 版本)
- **包管理器**：pnpm (推荐) 或 npm/yarn
- **Visual Studio**：2019+ 或 Visual Studio Build Tools (含C++开发工具)
- **Git**：最新版本
- **内存**：至少 4GB RAM (开发环境推荐 8GB+)
- **磁盘空间**：至少 5GB 可用空间

### 环境安装

1. **安装 Rust 工具链**

   ```bash
   # 方法1：使用 rustup 安装 (推荐)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Windows 用户可直接访问 https://rustup.rs 下载安装器
   # 或使用 winget (Windows 10/11)
   winget install Rustlang.Rustup

   # 安装完成后重启终端并验证
   rustc --version
   cargo --version
   ```

2. **安装 Node.js 和包管理器**

   ```bash
   # 访问 https://nodejs.org/ 下载 LTS 版本
   # 或使用 winget 安装
   winget install OpenJS.NodeJS

   # 安装 pnpm (推荐，速度更快，占用空间更少)
   npm install -g pnpm

   # 验证安装
   node --version
   pnpm --version
   ```

3. **安装 Visual Studio 或 Build Tools**

   ```bash
   # 方法1：使用 winget 安装 Build Tools (仅C++开发工具)
   winget install Microsoft.VisualStudio.2022.BuildTools

   # 方法2：下载完整 Visual Studio Community (推荐新手)
   # 访问 https://visualstudio.microsoft.com/downloads/
   ```

   确保安装以下工作负载：

   - **C++ build tools** (C++构建工具)
   - **Windows 10/11 SDK** (最新版本)
   - **CMake tools** (可选，但推荐)

4. **配置 Tauri 环境**

   ```bash
   # 安装 Tauri CLI (项目依赖中已包含，也可全局安装)
   cargo install tauri-cli

   # 或使用项目本地版本 (推荐)
   # 项目中已配置，无需额外安装
   ```

5. **克隆并设置项目**

   ```bash
   # 克隆仓库
   git clone https://github.com/xinggaoya/sing-box-windows.git
   cd sing-box-windows

   # 检查环境要求
   pnpm tauri info
   ```

6. **安装项目依赖**

   ```bash
   # 安装前端依赖
   pnpm install

   # Rust 依赖会在首次构建时自动下载
   ```

7. **首次构建与启动**

   ```bash
   # 开发模式启动 (支持热重载)
   pnpm tauri dev

   # 如果遇到错误，可以尝试清理缓存后重试
   pnpm clean        # 清理前端构建缓存
   cargo clean       # 清理 Rust 构建缓存（在 src-tauri 目录下执行）
   ```

### 开发工具推荐

1. **代码编辑器**：

   - **Visual Studio Code** (推荐) + 以下扩展：
     - `rust-analyzer` - Rust 语言支持
     - `Tauri` - Tauri 项目支持
     - `Vue - Official` - Vue 3 支持
     - `TypeScript Vue Plugin (Volar)` - Vue TypeScript 支持
     - `ESLint` - 代码质量检查
     - `Prettier` - 代码格式化

2. **调试工具**：

   - **Rust**: 内置 `println!` 和 `tracing` 日志
   - **前端**: Vue DevTools 浏览器扩展
   - **Tauri**: 内置开发者工具 (开发模式自动启用)

3. **性能分析**：
   - **前端**: Chrome DevTools 性能面板
   - **Rust**: `cargo flamegraph` (需要额外安装)

### 验证安装

运行以下命令验证环境是否正确配置：

```bash
# 检查 Tauri 环境
pnpm tauri info

# 运行健康检查
pnpm run type-check  # TypeScript 类型检查
pnpm run lint        # 代码质量检查

# 尝试构建 (第一次可能较慢)
pnpm tauri dev
```

成功启动后，应该能看到应用程序窗口并且控制台没有致命错误。

## 核心功能模块

### 常量组织 (constants/)

应用常量按功能模块分类组织，便于维护和扩展：

- `constants/core.rs` - 核心相关常量

  - `process` - 进程管理相关常量
  - `paths` - 文件路径相关常量
  - `config` - 配置相关常量

- `constants/network.rs` - 网络相关常量

  - `network_config` - 网络配置常量
  - `api` - API相关常量
  - `server` - 服务器相关常量
  - `rate_limit` - 速率限制常量

- `constants/system.rs` - 系统相关常量

  - `registry` - 注册表相关常量
  - `database` - 数据库相关常量
  - `jwt` - JWT认证相关常量

- `constants/common.rs` - 通用常量
  - `messages` - 提示消息常量
  - `log` - 日志相关常量

### 内核管理 (core/kernel_service.rs)

负责 Sing-Box 内核的下载、启动、停止和版本管理：

- `download_latest_kernel`：下载最新版内核
- `start_kernel`：启动内核服务
- `stop_kernel`：停止内核服务
- `restart_kernel`：重启内核服务
- `check_kernel_version`：检查内核版本

### 代理服务 (core/proxy_service.rs)

管理代理设置和节点选择：

- `set_system_proxy`：设置系统代理
- `set_tun_proxy`：设置TUN模式代理
- `toggle_proxy_mode`：切换代理模式
- `get_proxies`：获取代理节点列表
- `change_proxy`：切换使用的代理节点
- `test_node_delay`：测试节点延迟

### 订阅服务 (network/subscription_service.rs)

处理代理订阅的添加、更新和管理：

- `download_subscription`：下载订阅内容
- `add_manual_subscription`：手动添加订阅
- `get_current_config`：获取当前配置

### 系统服务 (system/system_service.rs)

处理与操作系统相关的功能：

- `check_admin`：检查管理员权限
- `restart_as_admin`：以管理员身份重启
- `install_service`：安装系统服务
- `uninstall_service`：卸载系统服务
- `check_service_status`：检查服务状态

### 更新服务 (system/update_service.rs)

处理应用程序的更新：

- `check_update`：检查更新
- `download_and_install_update`：下载并安装更新

## 前端开发指南

### 状态管理

项目使用 **Pinia 2.3.1** 进行状态管理，配合自定义的持久化插件和内存管理机制，采用模块化的目录结构组织各个 Store：

```
src/stores/
├── index.ts                # Store 主入口和持久化插件配置
├── StoreManager.ts         # Store 生命周期管理器
├── app/                    # 应用相关 store
│   ├── AppStore.ts         # 核心应用状态 (运行状态、自启动、数据恢复)
│   ├── ThemeStore.ts       # 主题管理 (亮暗主题切换)
│   ├── LocaleStore.ts      # 国际化状态 (语言切换)
│   ├── WindowStore.ts      # 窗口管理 (窗口状态、操作)
│   └── UpdateStore.ts      # 应用更新 (版本检查、更新状态)
├── kernel/                 # 内核相关 store
│   ├── KernelStore.ts         # 内核状态和操作 (启动/停止/版本)
│   ├── KernelRuntimeStore.ts  # 内核运行时状态 (实时状态监控)
│   ├── ProxyStore.ts          # 代理设置 (模式切换、节点管理)
│   ├── ConnectionStore.ts     # 连接管理 (活动连接列表)
│   ├── TrafficStore.ts        # 流量监控 (实时流量、历史统计)
│   └── LogStore.ts            # 日志管理 (日志收集、过滤、导出)
├── subscription/           # 订阅相关 store
│   └── SubStore.ts         # 订阅管理 (订阅列表、更新、解析)
└── tray/                   # 系统托盘相关 store
    └── TrayStore.ts        # 系统托盘管理 (托盘菜单、状态)
```

#### 核心特性

1. **自定义持久化系统**：

   - 使用 Tauri Store 进行持久化存储，替代传统的 localStorage
   - 支持防抖保存，避免高频操作导致的性能问题
   - 可配置的持久化选项（包含/排除特定字段）

2. **内存管理机制**：

   - 内存泄漏检测和自动清理
   - WebSocket 连接池管理
   - 临时 Store 的生命周期管理

3. **数据恢复机制**：
   - 应用启动时自动从持久化存储恢复状态
   - 支持数据恢复完成回调
   - 错误处理和默认值回退

各 Store 的职责如下：

1. **应用相关 Store**

   - **AppStore**：管理核心应用状态，如运行状态、自动启动设置等
   - **ThemeStore**：管理应用主题（亮色/暗色）
   - **LocaleStore**：管理应用语言设置
   - **WindowStore**：管理窗口状态、操作和事件
   - **UpdateStore**：管理应用更新检查和安装

2. **内核相关 Store**

   - **KernelStore**：管理内核版本、启动/停止操作
   - **ProxyStore**：管理代理设置和节点
   - **ConnectionStore**：管理连接信息和统计
   - **TrafficStore**：管理流量监控和统计
   - **LogStore**：管理日志记录和显示

3. **订阅相关 Store**

   - **SubStore**：管理代理订阅

4. **系统托盘相关 Store**
   - **TrayStore**：管理系统托盘图标和菜单

### 使用 Store

在组件中使用 Store 的示例：

```typescript
// 导入需要的 store
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'

// 在组件中使用
const appStore = useAppStore()
const themeStore = useThemeStore()
const kernelStore = useKernelStore()

// 使用 store 中的状态
const isRunning = appStore.isRunning
const isDarkTheme = themeStore.isDark

// 调用 store 中的方法
themeStore.toggleTheme()
await kernelStore.startKernel()
```

### Store 之间的交互

各个 Store 可以通过以下方式进行交互：

1. **直接引用**：一个 Store 可以导入并使用另一个 Store

   ```typescript
   // 在 KernelStore 中使用 AppStore
   import { useAppStore } from '../app/AppStore'

   export const useKernelStore = defineStore('kernel', () => {
     const appStore = useAppStore()

     const startKernel = async () => {
       // ...
       appStore.setRunningState(true)
     }
   })
   ```

2. **事件总线**：使用 mitt 进行松耦合的通信

   ```typescript
   // 在一个 Store 中发送事件
   import mitt from '@/utils/mitt'

   mitt.emit('kernel-started')

   // 在另一个 Store 中监听事件
   mitt.on('kernel-started', () => {
     // 处理内核启动事件
   })
   ```

3. **监听状态变化**：使用 Vue 的 watch 函数监听其他 Store 的状态变化

   ```typescript
   import { watch } from 'vue'

   watch(
     () => appStore.isRunning,
     (newValue) => {
       // 处理运行状态变化
     },
   )
   ```

### 组件开发规范

1. **组件命名**：

   - 页面组件使用 `XxxView.vue` 格式
   - 通用组件使用 `XxxComponent.vue` 格式
   - 布局组件使用 `XxxLayout.vue` 格式

2. **样式规范**：

   - 使用 scoped CSS
   - 遵循 BEM 命名规范
   - 颜色和尺寸使用变量管理

3. **事件处理**：
   - 使用 mitt 事件总线处理跨组件通信
   - 组件内部事件使用 emits 选项声明

### 路由管理

所有路由在 `src/router/index.ts` 中定义，新增页面需要在此处添加路由配置。

## 多语言开发指南

### 国际化架构

项目使用 [vue-i18n v9.14.4](https://vue-i18n.intlify.dev/) 实现完整的多语言支持，支持 4 种语言，主要包含以下部分：

1. **支持的语言**：

   - **中文 (zh-CN)**：简体中文，默认语言
   - **英语 (en-US)**：英文
   - **日语 (ja-JP)**：日本語
   - **俄语 (ru-RU)**：Русский

2. **语言文件结构**：

   - 位于 `src/locales/` 目录
   - 每个语言文件约 500+ 行翻译条目
   - 按功能模块分类：`common`、`nav`、`home`、`proxy`、`settings`、`logs` 等
   - 使用嵌套对象结构组织翻译项，便于维护

3. **i18n 配置** (`src/locales/index.ts`)：

   - 使用 Vue 3 Composition API 模式 (`legacy: false`)
   - 支持全局注入 `$t` 方法
   - 自动回退到中文作为备用语言
   - 导出支持的语言列表供设置页面使用

4. **语言切换机制**：

   - 通过 `LocaleStore` 管理当前语言状态
   - 支持手动选择语言或跟随系统语言
   - 语言偏好自动持久化保存
   - 实时切换不需要重启应用

### 使用多语言

1. **在组件中引入**：

   ```typescript
   import { useI18n } from 'vue-i18n'

   // 在setup中
   const { t } = useI18n()
   ```

2. **翻译文本**：

   ```vue
   <!-- 在模板中 -->
   <div>{{ t('home.title') }}</div>

   <!-- 在JS中 -->
   const message = t('common.success')
   ```

3. **带参数的翻译**：
   ```vue
   <!-- 使用命名参数 -->
   {{ t('rules.fetchSuccess', { count: 10 }) }}
   ```

### 添加新语言项

1. **翻译键命名规则**：

   - 使用 `模块.功能.操作` 的层级结构
   - 例如：`proxy.settings.save`
   - 保持命名简洁但有描述性

2. **添加新翻译流程**：

   - 在相应模块下添加新的翻译键
   - 确保在所有语言文件中都添加对应的翻译
   - 在添加新功能时，同步更新翻译文件

3. **示例**：

   ```typescript
   // 在zh-CN.ts中
   export default {
     myModule: {
       newFeature: '新功能名称',
       description: '这是功能描述',
     },
   }

   // 在en-US.ts中
   export default {
     myModule: {
       newFeature: 'New Feature Name',
       description: 'This is feature description',
     },
   }
   ```

### 组件国际化最佳实践

1. **提取所有硬编码文本**：

   - 将所有用户可见的文本移至语言文件
   - 包括按钮文本、标题、提示、错误信息等

2. **使用命名空间隔离**：

   - 每个主要组件使用独立的命名空间
   - 例如：`settings.theme.title` 而非简单的 `title`

3. **处理动态内容**：

   - 对于需要拼接的文本，使用参数化翻译而非字符串拼接
   - 例如：`t('message.greeting', { name: userName })`

4. **处理复数和日期**：
   - 使用 vue-i18n 的 pluralization 功能处理复数形式
   - 使用本地化的日期格式化

### 语言切换

在设置页面中，用户可以选择界面语言：

- 系统会记住用户的语言选择
- 也提供了跟随系统语言的选项

### 扩展新语言支持

要添加新的语言支持：

1. 在 `src/locales/` 目录下创建新的语言文件，如 `ja-JP.ts`
2. 复制现有语言文件并翻译所有内容
3. 在 `src/plugins/i18n.ts` 中导入并注册新语言
4. 在语言选择器中添加新语言选项

## 后端开发指南

### 使用常量

项目中的常量已经按功能模块分类组织，使用时需要注意导入正确的模块：

```rust
// 导入常量模块
use crate::app::constants::{messages, network_config, paths, process};

// 使用常量
fn example() {
    // 使用进程相关常量
    let flags = process::CREATE_NO_WINDOW;

    // 使用路径相关常量
    let config_path = paths::get_config_path();

    // 使用网络相关常量
    let api_port = network_config::DEFAULT_CLASH_API_PORT;

    // 使用消息常量
    println!("{}", messages::INFO_PROCESS_STARTED);
}
```

添加新常量时，应该将其添加到相应的模块中，而不是直接在代码中使用硬编码值。

### 命令注册

所有前端可调用的后端命令在 `src-tauri/src/lib.rs` 的 `run()` 函数中通过 `invoke_handler` 注册。当前注册的命令按功能模块分类：

```rust
.invoke_handler(tauri::generate_handler![
    // Core - Kernel service commands (内核服务)
    crate::app::core::kernel_service::start_kernel,
    crate::app::core::kernel_service::stop_kernel,
    crate::app::core::kernel_service::restart_kernel,
    crate::app::core::kernel_service::download_latest_kernel,
    crate::app::core::kernel_service::check_kernel_version,
    crate::app::core::kernel_service::start_websocket_relay,
    crate::app::core::kernel_service::is_kernel_running,
    crate::app::core::kernel_service::check_kernel_status,

    // Core - Proxy service commands (代理服务)
    crate::app::core::proxy_service::set_system_proxy,
    crate::app::core::proxy_service::set_manual_proxy,
    crate::app::core::proxy_service::set_tun_proxy,
    crate::app::core::proxy_service::toggle_ip_version,
    crate::app::core::proxy_service::get_api_token,
    crate::app::core::proxy_service::get_proxies,
    crate::app::core::proxy_service::change_proxy,
    crate::app::core::proxy_service::test_node_delay,
    crate::app::core::proxy_service::test_group_delay,
    crate::app::core::proxy_service::get_version_info,
    crate::app::core::proxy_service::get_rules,

    // Network - Subscription service commands (订阅服务)
    crate::app::network::subscription_service::download_subscription,
    crate::app::network::subscription_service::add_manual_subscription,
    crate::app::network::subscription_service::get_current_config,
    crate::app::network::subscription_service::toggle_proxy_mode,
    crate::app::network::subscription_service::get_current_proxy_mode,

    // System - System service commands (系统服务)
    crate::app::system::system_service::check_admin,
    crate::app::system::system_service::restart_as_admin,
    crate::app::system::system_service::toggle_devtools,
    crate::app::system::system_service::open_devtools,
    crate::app::system::system_service::close_devtools,
    crate::app::system::system_service::is_devtools_open,

    // System - Update service commands (更新服务)
    crate::app::system::update_service::check_update,
    crate::app::system::update_service::download_and_install_update,

    // System - Config service commands (配置服务)
    crate::app::system::config_service::update_singbox_ports,
])
```

#### 命令分类说明

1. **内核服务命令** (Kernel Service)：

   - 内核的生命周期管理（启动、停止、重启）
   - 内核版本检查和下载
   - WebSocket 中继服务管理
   - 内核运行状态检查

2. **代理服务命令** (Proxy Service)：

   - 代理模式设置（系统代理、手动代理、TUN模式）
   - 代理节点管理和切换
   - 节点延迟测试
   - 代理规则获取

3. **订阅服务命令** (Subscription Service)：

   - 订阅下载和解析
   - 手动添加订阅
   - 配置管理
   - 代理模式切换

4. **系统服务命令** (System Service)：

   - 管理员权限检查和提升
   - 开发者工具控制
   - 系统级功能操作

5. **更新服务命令** (Update Service)：

   - 应用更新检查
   - 自动下载和安装更新

6. **配置服务命令** (Config Service)：
   - Sing-box 端口配置更新
   - 配置文件管理

### 模块组织

项目采用模块化的组织结构，每个模块都有自己的 `mod.rs` 文件作为入口点：

1. **模块入口文件**：

   - `app/mod.rs` - 应用模块入口，定义子模块并重导出常用组件
   - `app/core/mod.rs` - 核心服务模块入口
   - `app/network/mod.rs` - 网络服务模块入口
   - `app/system/mod.rs` - 系统服务模块入口
   - `app/constants/mod.rs` - 常量模块入口

2. **重导出机制**：

   - 每个模块的 `mod.rs` 文件会重新导出其子模块中的重要组件
   - 这样可以简化导入路径，例如使用 `crate::app::core::kernel_service` 而不是完整路径

3. **向后兼容性**：
   - `app/mod.rs` 中的重导出确保了代码重构不会破坏现有的导入路径
   - 例如：`pub use core::kernel_service;` 允许使用 `crate::app::kernel_service` 而不是新路径

### 新增功能开发流程

1. 在相应的服务模块中定义函数
2. 在模块的 `mod.rs` 中重导出该函数（如果需要在其他模块中使用）
3. 在 `lib.rs` 中导入并注册该函数
4. 在前端通过 `invoke` 调用该函数

示例：

```rust
// 在 app/core/kernel_service.rs 中
#[tauri::command]
pub async fn my_new_function(param: String) -> Result<String, String> {
    // 实现逻辑
    Ok("成功".to_string())
}

// 在 app/core/mod.rs 中确保导出该函数
pub use kernel_service::my_new_function;

// 在 lib.rs 中注册
.invoke_handler(tauri::generate_handler![
    // 其他命令...
    my_new_function,
])
```

```typescript
// 在前端调用
import { invoke } from '@tauri-apps/api/tauri'

async function callMyFunction() {
  try {
    const result = await invoke('my_new_function', { param: 'test' })
    console.log(result)
  } catch (error) {
    console.error(error)
  }
}
```

### 进程管理

内核进程管理位于 `process` 模块，在修改相关代码时需特别注意：

- 确保进程正确启动和终止
- 处理好权限问题
- 管理好进程的生命周期

## 构建与发布

### 开发环境

#### 开发服务器启动

```bash
# 启动开发服务器 (支持热重载)
pnpm tauri dev

# 或者使用 npm scripts
pnpm run tauri dev
```

#### 开发工具和调试

```bash
# 前端类型检查
pnpm run type-check

# 代码质量检查
pnpm run lint          # 运行所有 lint 检查
pnpm run lint:eslint   # 仅运行 ESLint
pnpm run lint:oxlint   # 仅运行 oxlint

# 代码格式化
pnpm run format        # 格式化 src/ 目录代码

# 清理缓存
pnpm clean             # 清理前端构建缓存
cd src-tauri && cargo clean  # 清理 Rust 构建缓存
```

### 生产环境构建

#### 完整构建流程

```bash
# 1. 安装依赖 (如果尚未安装)
pnpm install

# 2. 前端构建
pnpm run build

# 3. 完整应用构建 (包含前端和后端)
pnpm tauri build

# 构建产物位置：
# - Windows: src-tauri/target/release/bundle/msi/
# - 可执行文件: src-tauri/target/release/sing-box-windows.exe
```

#### 构建输出

生产构建会在以下位置生成文件：

- **安装包**: `src-tauri/target/release/bundle/msi/sing-box-windows_版本号_x64_zh-CN.msi`
- **可执行文件**: `src-tauri/target/release/sing-box-windows.exe`
- **前端构建产物**: `dist/` 目录

#### 性能优化

项目配置了优化的构建选项：

```toml
# Cargo.toml 中的发布配置
[profile.release]
opt-level = 3        # 最高优化级别
debug = false        # 禁用调试信息
lto = true          # 链接时优化
codegen-units = 1   # 单个代码生成单元
panic = "abort"     # panic 时直接终止
strip = true        # 移除符号信息
overflow-checks = false  # 禁用溢出检查
```

### 发布流程

#### 版本管理

项目使用统一的版本号管理，需要同时更新：

1. **package.json**: 前端版本号
2. **src-tauri/tauri.conf.json**: Tauri 应用版本
3. **src-tauri/Cargo.toml**: Rust 包版本

```bash
# 示例：更新到版本 1.8.0
# 更新 package.json
npm version 1.8.0 --no-git-tag-version

# 手动更新 tauri.conf.json 和 Cargo.toml 中的版本号
```

#### 完整发布流程

1. **版本准备**:

   ```bash
   # 确保所有更改已提交
   git status

   # 更新版本号
   npm version patch --no-git-tag-version  # 或 minor/major
   ```

2. **质量检查**:

   ```bash
   # 运行所有检查
   pnpm run type-check
   pnpm run lint

   # 测试构建
   pnpm tauri build
   ```

3. **测试验证**:

   - 测试生成的安装包在目标系统上的安装和运行
   - 验证核心功能正常工作
   - 检查内存使用和性能

4. **创建发布**:

   ```bash
   # 创建 git tag
   git add -A
   git commit -m "chore: bump version to v1.8.0"
   git tag -a v1.8.0 -m "Release v1.8.0"
   git push origin main --tags
   ```

5. **GitHub Release**:
   - 在 GitHub 上创建新的 Release
   - 上传构建产物 (.msi 安装包)
   - 编写 Release Notes，说明新功能和修复

#### 自动化发布 (CI/CD)

项目支持 GitHub Actions 自动化构建：

- 在推送 tag 时自动触发构建
- 自动运行测试和质量检查
- 自动构建多平台安装包
- 自动创建 GitHub Release

### 部署注意事项

1. **代码签名**: 生产环境建议对可执行文件进行代码签名
2. **安全扫描**: 上传前对构建产物进行安全扫描
3. **版本兼容性**: 确保新版本与现有配置文件兼容
4. **回滚计划**: 准备版本回滚方案

## 常见问题

### Q: 如何调试 Rust 后端代码？

A: 项目使用 `tracing` 进行结构化日志记录：

```rust
// 在代码中添加日志
tracing::info!("内核启动中...");
tracing::error!("启动失败: {}", error);

// 在开发模式下，日志会输出到控制台
// 可以通过环境变量控制日志级别
RUST_LOG=debug pnpm tauri dev
```

也可以使用 VS Code 的 Rust Analyzer 扩展和内置调试器，或启用 Tauri 开发者工具。

### Q: 前端和后端之间的通信方式？

A: 使用 Tauri 的 `invoke` API 进行通信：

```typescript
// 前端调用后端命令
import { invoke } from '@tauri-apps/api/tauri'

// 调用后端函数
const result = await invoke('start_kernel', {
  api_port: 9090,
  mixed_port: 7890,
})
```

```rust
// 后端命令定义
#[tauri::command]
pub async fn start_kernel(api_port: u16, mixed_port: u16) -> Result<String, String> {
    // 实现逻辑
    Ok("启动成功".to_string())
}
```

### Q: 如何处理系统权限问题？

A: 使用 `system_service.rs` 中的权限管理函数：

```rust
// 检查管理员权限
let is_admin = check_admin().await?;

// 如果需要权限且当前不是管理员，重启为管理员
if !is_admin && needs_admin_rights {
    restart_as_admin().await?;
}
```

TUN 模式需要管理员权限，应用会自动检测并请求权限提升。

### Q: 如何添加新的依赖？

A:

**前端依赖**:

```bash
# 添加运行时依赖
pnpm add vue-router

# 添加开发依赖
pnpm add -D @types/node

# 更新依赖
pnpm update
```

**后端依赖** (编辑 `src-tauri/Cargo.toml`):

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
```

### Q: 如何添加新的前端页面？

A:

1. 在 `src/views/` 创建新的 Vue 组件
2. 在 `src/router/index.ts` 添加路由配置
3. 在对应的导航组件中添加菜单项
4. 如果需要，创建对应的 Store 管理状态

### Q: 如何处理跨语言的数据类型？

A: 使用 `serde` 进行序列化，确保前后端数据类型一致：

```rust
// Rust 类型定义
#[derive(Serialize, Deserialize)]
pub struct ProxyNode {
    pub name: String,
    pub server: String,
    pub port: u16,
    pub delay: Option<u32>,
}
```

```typescript
// TypeScript 类型定义
interface ProxyNode {
  name: string
  server: string
  port: number
  delay?: number
}
```

### Q: 应用启动缓慢怎么办？

A: 检查以下优化点：

1. **前端优化**：

   - 使用懒加载组件 `LazyComponent.vue`
   - 启用虚拟滚动 `VirtualList.vue`
   - 检查 Store 数据恢复逻辑

2. **后端优化**：

   - 检查内核启动时间
   - 优化文件IO操作
   - 使用异步操作避免阻塞

3. **构建优化**：
   - 使用发布模式构建
   - 检查依赖大小

### Q: 内存使用过高怎么办？

A: 项目内置了内存管理机制：

```typescript
// 使用内存泄漏修复工具
import { webSocketCleaner, temporaryStoreManager } from '@/utils/memory-leak-fix'

// 清理WebSocket连接
webSocketCleaner.cleanupAll()

// 清理临时Store
temporaryStoreManager.cleanupAllStores()
```

定期检查：

- WebSocket 连接是否正确关闭
- 大量数据的Store是否正确清理
- 定时器是否正确销毁

### Q: 如何调试WebSocket连接问题？

A:

```typescript
// 检查WebSocket服务状态
import { webSocketService } from '@/services/websocket-service'

console.log('WebSocket 状态:', {
  isConnected: webSocketService.isWebSocketConnected(),
  connectionCount: webSocketService.getConnectionCount(),
})

// 启用WebSocket调试日志
localStorage.setItem('debug-websocket', 'true')
```

### Q: 如何贡献代码？

A:

1. **Fork 项目**并克隆到本地
2. **创建功能分支**: `git checkout -b feature/新功能名称`
3. **遵循代码规范**: 运行 `pnpm run lint` 检查代码质量
4. **编写测试**并确保现有测试通过
5. **提交更改**: 使用清晰的提交信息
6. **发起 Pull Request**并描述更改内容

代码提交前请确保：

- 代码通过所有lint检查
- TypeScript类型检查无错误
- 应用能正常构建和运行

## 性能优化与最佳实践

### 前端性能优化

#### 1. 组件层面优化

```vue
<!-- 使用懒加载组件 -->
<script setup>
import LazyComponent from '@/components/LazyComponent.vue'
</script>

<template>
  <LazyComponent>
    <HeavyComponent />
  </LazyComponent>
</template>
```

#### 2. 列表渲染优化

```vue
<!-- 大列表使用虚拟滚动 -->
<script setup>
import VirtualList from '@/components/VirtualList.vue'

const items = ref(largeDataArray) // 大量数据
</script>

<template>
  <VirtualList :items="items" :item-height="50" :visible-count="20">
    <template #item="{ item }">
      <div>{{ item.name }}</div>
    </template>
  </VirtualList>
</template>
```

#### 3. Store 优化

```typescript
// 使用防抖保存避免频繁IO
export const useTrafficStore = defineStore(
  'traffic',
  () => {
    // 配置防抖保存
  },
  {
    persist: {
      enabled: true,
      debounceDelay: 1000, // 1秒防抖
      excludeHighFrequencyKeys: ['currentSpeed', 'instantData'], // 排除高频数据
    },
  },
)
```

#### 4. 内存管理

```typescript
// 组件卸载时清理资源
import { onUnmounted } from 'vue'
import { webSocketCleaner } from '@/utils/memory-leak-fix'

onUnmounted(() => {
  // 清理定时器
  clearInterval(timer)

  // 清理事件监听
  mitt.off('some-event', handler)

  // 清理WebSocket连接
  webSocketCleaner.cleanup('connection-id')
})
```

### 后端性能优化

#### 1. 异步操作

```rust
// 使用异步避免阻塞
#[tauri::command]
pub async fn download_large_file(url: String) -> Result<String, String> {
    tokio::spawn(async move {
        // 长时间运行的任务
        download_file_async(url).await
    }).await.map_err(|e| e.to_string())?
}
```

#### 2. 资源管理

```rust
// 使用 Arc 和 Mutex 管理共享状态
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref KERNEL_STATE: Arc<Mutex<KernelState>> = Arc::new(Mutex::new(KernelState::new()));
}

// 正确释放资源
impl Drop for KernelManager {
    fn drop(&mut self) {
        self.stop_kernel();
        self.cleanup_resources();
    }
}
```

#### 3. 错误处理

```rust
// 使用结构化错误处理
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("配置错误: {message}")]
    Config { message: String },
}
```

### 构建优化

#### 1. Vite 配置优化

```typescript
// vite.config.ts
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['vue', 'vue-router', 'pinia'],
          ui: ['naive-ui'],
          utils: ['lodash', 'dayjs'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },
  // 生产环境移除console
  esbuild: {
    drop: process.env.NODE_ENV === 'production' ? ['console', 'debugger'] : [],
  },
})
```

#### 2. Cargo 优化配置

```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
strip = true

[profile.dev]
opt-level = 1  # 开发时适度优化
debug = true
```

### 安全最佳实践

#### 1. 前端安全

```typescript
// 输入验证
function validateUrl(url: string): boolean {
  try {
    const parsed = new URL(url)
    return ['http:', 'https:'].includes(parsed.protocol)
  } catch {
    return false
  }
}

// XSS 防护
function sanitizeInput(input: string): string {
  return input.replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
}
```

#### 2. 后端安全

```rust
// 路径验证
use std::path::{Path, PathBuf};

fn validate_path(path: &str) -> Result<PathBuf, String> {
    let path = Path::new(path);

    // 防止路径遍历攻击
    if path.components().any(|c| matches!(c, std::path::Component::ParentDir)) {
        return Err("不允许父目录访问".to_string());
    }

    Ok(path.to_path_buf())
}

// 敏感数据处理
use zeroize::Zeroize;

struct SensitiveData {
    secret: String,
}

impl Drop for SensitiveData {
    fn drop(&mut self) {
        self.secret.zeroize(); // 清零敏感数据
    }
}
```

### 测试最佳实践

#### 1. 前端测试

```typescript
// 组件测试示例
import { mount } from '@vue/test-utils'
import { describe, it, expect } from 'vitest'

describe('ProxyCard', () => {
  it('应该显示代理信息', () => {
    const wrapper = mount(ProxyCard, {
      props: {
        proxy: { name: 'test', server: '1.1.1.1', port: 8080 },
      },
    })

    expect(wrapper.text()).toContain('test')
  })
})
```

#### 2. 后端测试

```rust
// 单元测试
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_validation() {
        let result = validate_proxy_config(&valid_config()).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_parsing() {
        let url = "https://example.com/config";
        assert!(is_valid_subscription_url(url));
    }
}
```

### 监控与调试

#### 1. 性能监控

```typescript
// 前端性能监控
class PerformanceMonitor {
  static measureOperation<T>(name: string, operation: () => T): T {
    const start = performance.now()
    const result = operation()
    const end = performance.now()

    console.log(`${name} 耗时: ${end - start}ms`)
    return result
  }
}

// 使用示例
const data = PerformanceMonitor.measureOperation('数据处理', () => processLargeData(rawData))
```

#### 2. 内存监控

```typescript
// 内存使用监控
function checkMemoryUsage() {
  if ('memory' in performance) {
    const memory = (performance as any).memory
    console.log(`内存使用: ${Math.round(memory.usedJSHeapSize / 1024 / 1024)}MB`)
  }
}

setInterval(checkMemoryUsage, 30000) // 每30秒检查一次
```

---

## 贡献指南

我们欢迎各种形式的贡献，包括但不限于：

- 代码贡献
- 文档改进
- 问题报告
- 功能建议

请参考项目根目录的 README.md 中的贡献流程。
