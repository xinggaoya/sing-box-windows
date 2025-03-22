# Sing-Box Windows 开发文档

## 目录

- [项目概述](#项目概述)
- [技术栈](#技术栈)
- [项目结构](#项目结构)
- [开发环境搭建](#开发环境搭建)
- [核心功能模块](#核心功能模块)
- [前端开发指南](#前端开发指南)
- [后端开发指南](#后端开发指南)
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

```
sing-box-windows/
├── src/                # 前端源代码
│   ├── assets/        # 静态资源
│   ├── components/    # 通用组件
│   │   └── layout/    # 布局组件
│   ├── router/        # 路由配置
│   ├── stores/        # 状态管理
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
│   │   │   ├── kernel_service.rs    # 内核服务
│   │   │   ├── proxy_service.rs     # 代理服务
│   │   │   ├── subscription_service.rs # 订阅服务
│   │   │   ├── system_service.rs    # 系统服务
│   │   │   └── update_service.rs    # 更新服务
│   │   ├── entity/    # 数据实体
│   │   ├── process/   # 进程管理
│   │   ├── utils/     # 工具函数
│   │   ├── config.rs  # 配置管理
│   │   ├── lib.rs     # 库入口
│   │   └── main.rs    # 程序入口
│   └── Cargo.toml     # Rust 依赖配置
└── package.json       # 项目配置
```

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

### 内核管理 (kernel_service.rs)

负责 Sing-Box 内核的下载、启动、停止和版本管理：

- `download_latest_kernel`：下载最新版内核
- `start_kernel`：启动内核服务
- `stop_kernel`：停止内核服务
- `restart_kernel`：重启内核服务
- `check_kernel_version`：检查内核版本

### 代理服务 (proxy_service.rs)

管理代理设置和节点选择：

- `set_system_proxy`：设置系统代理
- `set_tun_proxy`：设置TUN模式代理
- `toggle_proxy_mode`：切换代理模式
- `get_proxies`：获取代理节点列表
- `change_proxy`：切换使用的代理节点
- `test_node_delay`：测试节点延迟

### 订阅服务 (subscription_service.rs)

处理代理订阅的添加、更新和管理：

- `download_subscription`：下载订阅内容
- `add_manual_subscription`：手动添加订阅
- `get_current_config`：获取当前配置

### 系统服务 (system_service.rs)

处理与操作系统相关的功能：

- `check_admin`：检查管理员权限
- `restart_as_admin`：以管理员身份重启
- `get_traffic_data`：获取流量数据

### 更新服务 (update_service.rs)

处理应用程序的更新：

- `check_update`：检查更新
- `download_and_install_update`：下载并安装更新

## 前端开发指南

### 状态管理

项目使用 Pinia 进行状态管理，主要 Store 包括：

1. **appStore**：应用全局状态

   - 管理应用运行状态
   - 管理窗口状态
   - 处理应用配置

2. **infoStore**：内核信息状态

   - 管理内核版本信息
   - 处理内核状态和事件监听

3. **trayStore**：系统托盘状态
   - 管理系统托盘图标和菜单

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

## 后端开发指南

### 命令注册

所有前端可调用的后端命令在 `src-tauri/src/lib.rs` 的 `run()` 函数中通过 `invoke_handler` 注册：

```rust
.invoke_handler(tauri::generate_handler![
    start_kernel,
    download_latest_kernel,
    // 其他命令...
])
```

### 新增功能开发流程

1. 在相应的服务模块中定义函数
2. 在 `lib.rs` 中导入并注册该函数
3. 在前端通过 `invoke` 调用该函数

示例：

```rust
// 在 kernel_service.rs 中
#[tauri::command]
pub async fn my_new_function(param: String) -> Result<String, String> {
    // 实现逻辑
    Ok("成功".to_string())
}

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
