# Sing-Box Windows 开发文档

## 目录

- [项目概述](#项目概述)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [开发环境搭建](#开发环境搭建)
- [核心功能模块](#核心功能模块)
- [前端开发指南](#前端开发指南)
- [后端开发指南](#后端开发指南)
- [系统服务管理](#系统服务管理)
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

- **Vue 3**：核心前端框架
- **TypeScript**：类型安全的 JavaScript 超集
- **Naive UI**：高质量 Vue 3 组件库
- **Pinia**：Vue 状态管理库
- **Vue Router**：Vue 路由管理
- **VueUse**：Vue 实用工具集合

### 后端技术

- **Rust**：高性能系统编程语言
- **Tauri 2.0**：构建跨平台应用的框架
- **tokio**：异步运行时
- **serde**：序列化和反序列化
- **reqwest**：HTTP 客户端

## 项目结构

项目采用模块化的目录结构，按功能划分不同的模块：

```
sing-box-windows/
├── src/                # 前端源代码
│   ├── assets/        # 静态资源
│   ├── components/    # 通用组件
│   │   └── layout/    # 布局组件
│   ├── router/        # 路由配置
│   ├── stores/        # 状态管理
│   │   ├── index.ts   # 主入口文件
│   │   ├── app/       # 应用相关 store
│   │   ├── kernel/    # 内核相关 store
│   │   ├── subscription/ # 订阅相关 store
│   │   └── tray/      # 托盘相关 store
│   ├── utils/         # 工具函数
│   │   ├── format.ts  # 格式化工具
│   │   └── mitt.ts    # 事件总线
│   ├── views/         # 页面组件
│   │   ├── HomeView.vue      # 主页
│   │   ├── ProxyView.vue     # 代理管理
│   │   ├── SubView.vue       # 订阅管理
│   │   ├── LogView.vue       # 日志查看
│   │   ├── SettingView.vue   # 设置页面
│   │   ├── RulesView.vue     # 规则管理
│   │   └── ConnectionsView.vue # 连接管理
│   └── App.vue         # 根组件
├── src-tauri/         # Rust 后端代码
│   ├── src/           # 源代码
│   │   ├── app/       # 应用服务
│   │   │   ├── constants/          # 常量定义
│   │   │   │   ├── mod.rs            # 常量模块入口
│   │   │   │   ├── core.rs           # 核心相关常量(进程、路径、配置)
│   │   │   │   ├── network.rs        # 网络相关常量(API、网络配置)
│   │   │   │   ├── system.rs         # 系统相关常量(注册表、数据库、JWT)
│   │   │   │   └── common.rs         # 通用常量(消息、日志)
│   │   │   ├── core/               # 核心服务
│   │   │   │   ├── kernel_service.rs  # 内核服务
│   │   │   │   └── proxy_service.rs   # 代理服务
│   │   │   ├── network/            # 网络服务
│   │   │   │   └── subscription_service.rs # 订阅服务
│   │   │   └── system/             # 系统服务
│   │   │       ├── system_service.rs  # 系统功能
│   │   │       └── update_service.rs  # 更新功能
│   │   ├── entity/    # 数据实体
│   │   ├── process/   # 进程管理
│   │   ├── utils/     # 工具函数
│   │   ├── config.rs  # 配置管理
│   │   ├── lib.rs     # 库入口
│   │   └── main.rs    # 程序入口
│   └── Cargo.toml     # Rust 依赖配置
└── package.json       # 项目配置
```

这种模块化的结构有以下优点：

1. **功能划分清晰**：按功能将代码划分为核心、网络、系统等模块
2. **易于维护**：相关功能集中在一起，便于定位和修改
3. **降低耦合度**：每个模块都有明确的职责和边界
4. **方便扩展**：添加新功能时可以在相应模块中扩展，不影响其他模块

## 开发环境搭建

### 系统要求

- Windows 10 1809 或更高版本
- 最新版 Rust 工具链
- Node.js 18.0+
- pnpm 包管理器
- Visual Studio 2019+ (含C++开发工具)
- Git

### 环境安装

1. **安装 Rust**

   ```bash
   # 使用 rustup 安装
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   # Windows 可访问 https://rustup.rs 下载安装器
   ```

2. **安装 Node.js 和 pnpm**

   ```bash
   # 安装 Node.js：https://nodejs.org/
   # 安装 pnpm
   npm install -g pnpm
   ```

3. **安装 Visual Studio**
   确保安装了"使用C++的桌面开发"工作负载。

4. **克隆项目**

   ```bash
   git clone https://github.com/xinggaoya/sing-box-windows.git
   cd sing-box-windows
   ```

5. **安装依赖**

   ```bash
   pnpm install
   ```

6. **启动开发服务器**
   ```bash
   pnpm tauri dev
   ```

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

项目使用 Pinia 进行状态管理，采用模块化的目录结构组织各个 Store：

```
src/stores/
├── index.ts                # 主入口文件，导出所有 store
├── app/                    # 应用相关 store
│   ├── AppStore.ts         # 核心应用状态
│   ├── ThemeStore.ts       # 主题管理
│   ├── LocaleStore.ts      # 本地化/语言
│   ├── WindowStore.ts      # 窗口管理
│   └── UpdateStore.ts      # 应用更新
├── kernel/                 # 内核相关 store
│   ├── KernelStore.ts      # 内核状态和操作
│   ├── ProxyStore.ts       # 代理设置
│   ├── ConnectionStore.ts  # 连接管理
│   ├── TrafficStore.ts     # 流量监控
│   └── LogStore.ts         # 日志管理
├── subscription/           # 订阅相关 store
│   └── SubStore.ts         # 订阅管理
└── tray/                   # 系统托盘相关 store
    └── TrayStore.ts        # 系统托盘管理
```

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
     }
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

项目使用 [vue-i18n](https://vue-i18n.intlify.dev/) 实现多语言支持，主要包含以下部分：

1. **语言文件**：

   - 位于 `src/locales/` 目录
   - `zh-CN.ts`：中文翻译
   - `en-US.ts`：英文翻译
   - 每个文件导出一个包含翻译键值对的对象

2. **翻译结构**：

   - 翻译按功能模块分类（common, nav, home, proxy等）
   - 使用嵌套对象结构组织翻译项

3. **i18n配置**：
   - 位于 `src/plugins/i18n.ts`
   - 负责加载所有语言文件并创建i18n实例

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

所有前端可调用的后端命令在 `src-tauri/src/lib.rs` 的 `run()` 函数中通过 `invoke_handler` 注册：

```rust
.invoke_handler(tauri::generate_handler![
    // Core - Kernel service commands
    start_kernel,
    stop_kernel,
    restart_kernel,
    download_latest_kernel,

    // Network - Subscription service commands
    download_subscription,
    add_manual_subscription,

    // 其他命令...
])
```

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

## 系统服务管理

Sing-Box Windows 使用 Windows 服务机制来提供更高权限的功能，特别是 TUN 模式。服务管理是应用程序的重要组成部分。

### 服务架构

1. **服务组件**：
   - `sing-box-service.exe`：核心服务可执行文件
   - 安装在系统服务中，名称为 "SingBoxService"
   - 以 SYSTEM 权限运行，提供 TUN 模式所需的网络接口管理能力

2. **服务流程**：
   - 服务首次安装需要管理员权限
   - 安装时将服务可执行文件复制到缓存目录，确保即使原始文件被锁定也能安装
   - 服务安装后会自动启动，并可以跟随系统启动

3. **权限管理**：
   - 服务安装/卸载需要管理员权限
   - 应用程序自动检测管理员状态，并提供相应的提示和操作

### 服务管理页面

`ServiceInstallView.vue` 是一个专门用于服务安装和管理的页面，提供以下功能：

1. **状态显示**：
   - 显示服务安装状态（已安装/未安装）
   - 显示服务运行状态（运行中/已停止）
   - 显示当前应用程序的管理员权限状态

2. **操作功能**：
   - 安装服务：以管理员权限将服务安装到系统
   - 卸载服务：移除已安装的系统服务
   - 管理员重启：当缺少管理员权限时，提供以管理员身份重启的选项

3. **导航控制**：
   - 服务安装完成后提供"继续使用"按钮返回主页
   - 确保用户能够清楚了解服务的状态和必要性

### 后端服务管理接口

系统服务管理的后端实现在 `system_service.rs` 中：

```rust
// 检查服务状态
#[tauri::command]
pub async fn check_service_status() -> Result<serde_json::Value, String> {
    // 使用 Windows SC 命令检查服务状态
    // 返回服务的安装和运行状态
}

// 安装服务
#[tauri::command]
pub async fn install_service() -> Result<serde_json::Value, String> {
    // 复制服务文件到缓存目录
    // 执行服务安装命令
}

// 卸载服务
#[tauri::command]
pub async fn uninstall_service() -> Result<serde_json::Value, String> {
    // 执行服务卸载命令
}
```

### 前端服务管理 Store

`ServiceStore.ts` 提供了与后端通信和管理服务状态的功能：

```typescript
// 检查服务状态
async function checkServiceStatus() {
  try {
    const result = await tauriApi.system.checkServiceStatus()
    isServiceInstalled.value = result.installed
    isServiceRunning.value = result.running
    return { installed: result.installed, running: result.running }
  } catch (error) {
    // 错误处理...
  }
}

// 安装服务
async function installService() {
  // 调用后端安装服务并处理结果...
}

// 卸载服务
async function uninstallService() {
  // 调用后端卸载服务并处理结果...
}
```

### 设置页面的服务管理

`SettingView.vue` 集成了服务管理功能：

1. **状态展示**：
   - 显示服务的安装和运行状态
   - 提供服务状态刷新功能

2. **操作功能**：
   - 卸载服务按钮：允许用户直接从设置页面卸载服务
   - 安装服务跳转：提供跳转到专门的服务安装页面

3. **权限控制**：
   - 根据管理员权限状态动态控制按钮的可用性
   - 提供适当的提示和引导

### 服务管理开发最佳实践

1. **错误处理**：
   - 全面捕获和处理服务操作中可能出现的异常
   - 为用户提供清晰的错误提示和解决建议

2. **权限检查**：
   - 在执行需要管理员权限的操作前，始终检查权限状态
   - 提供明确的管理员权限获取路径

3. **服务文件管理**：
   - 使用缓存目录存放服务文件副本，避免文件锁定问题
   - 确保服务文件路径解析正确，特别是在不同的安装环境中

4. **用户体验**：
   - 提供明确的服务状态反馈和操作结果通知
   - 设计清晰的服务管理流程和界面导航

5. **扩展服务功能**：
   - 添加新的服务功能时，需要同时更新服务可执行文件和管理接口
   - 确保新功能在不同权限环境下的兼容性

## 构建与发布

### 开发环境构建

```bash
pnpm tauri dev
```

### 生产环境构建

```bash
pnpm tauri build
```

此命令会在 `src-tauri/target/release/bundle` 目录下生成安装包。

### 发布流程

1. 更新版本号（`package.json` 和 `src-tauri/tauri.conf.json`）
2. 执行构建
3. 测试安装包
4. 创建 GitHub Release
5. 上传安装包

## 常见问题

### Q: 如何调试 Rust 后端代码？

A: 可以使用 Visual Studio Code 的 Rust Analyzer 扩展，配合 `console.log` 和 Tauri 的日志功能。
在 `Cargo.toml` 的 `[features]` 部分添加 `devtools` 特性也可启用开发者工具。

### Q: 前端和后端之间的通信方式？

A: 使用 Tauri 的 `invoke` API 进行通信，后端通过 `#[tauri::command]` 标记的函数接收请求。

### Q: 如何处理系统权限问题？

A: 使用 `system_service.rs` 中的 `check_admin` 和 `restart_as_admin` 函数检查和请求管理员权限。
TUN 模式必须以管理员权限运行。

### Q: 如何添加新的依赖？

A: 前端依赖通过 `pnpm add` 添加，后端依赖在 `Cargo.toml` 中添加。

---

## 贡献指南

我们欢迎各种形式的贡献，包括但不限于：

- 代码贡献
- 文档改进
- 问题报告
- 功能建议

请参考项目根目录的 README.md 中的贡献流程。
