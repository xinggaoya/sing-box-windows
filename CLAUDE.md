# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

这是一个基于 Tauri 2.0 + Vue 3 的 Sing-Box Windows GUI 客户端，为 Windows 平台提供现代化的代理管理界面。

## 开发命令

### 开发环境
```bash
# 安装依赖
pnpm install

# 启动开发服务器（热重载）
pnpm tauri dev

# 构建生产版本
pnpm tauri build

# 类型检查
pnpm type-check

# 代码检查和修复
pnpm lint

# 代码格式化
pnpm format
```

### 测试
```bash
# 运行测试
pnpm test
```

## 项目架构

### 技术栈
- **前端**: Vue 3 + TypeScript + Naive UI + Pinia
- **后端**: Rust + Tauri 2.0
- **构建工具**: Vite + pnpm
- **状态管理**: Pinia (支持按需加载和内存优化)
- **通信**: WebSocket (实时数据) + Tauri IPC (命令调用)

### 目录结构
```
sing-box-windows/
├── src/                    # 前端源码
│   ├── components/         # 可复用组件
│   │   ├── home/          # 首页组件
│   │   ├── layout/        # 布局组件
│   │   └── ...
│   ├── stores/            # Pinia 状态管理
│   │   ├── app/           # 应用级状态
│   │   ├── kernel/        # 内核相关状态
│   │   ├── subscription/  # 订阅管理
│   │   └── tray/          # 系统托盘
│   ├── services/          # 业务逻辑服务
│   ├── utils/             # 工具函数
│   ├── locales/           # 国际化文件
│   └── views/             # 页面组件
├── src-tauri/             # Rust 后端
│   ├── src/
│   │   ├── app/           # 应用模块
│   │   │   ├── core/      # 核心功能
│   │   │   ├── network/   # 网络操作
│   │   │   └── system/    # 系统集成
│   │   └── main.rs        # 程序入口
│   └── Cargo.toml         # Rust 依赖
```

### 核心架构特性

#### Store 管理系统
- 使用 `StoreManager` 实现按需加载
- 支持内存优化和自动清理
- 核心Store在应用启动时预加载: `app`, `theme`, `locale`
- 路由级别的Store预加载机制

#### 实时数据通信
- WebSocket 服务用于实时数据传输（流量统计、连接状态、日志等）
- 自动重连机制和内存泄漏防护
- Tauri IPC 用于前端与后端的命令调用

#### 内存优化
- 组件懒加载
- Store按需加载和清理
- WebSocket连接管理和清理
- 临时Store监控机制

## 主要功能模块

### 内核管理 (src-tauri/src/app/core/)
- `kernel_service.rs`: 内核启动/停止/版本检查
- `proxy_service.rs`: 代理配置和模式切换
- `task_manager.rs`: 异步任务管理

### 网络功能 (src-tauri/src/app/network/)
- `subscription_service.rs`: 订阅下载和管理

### 系统集成 (src-tauri/src/app/system/)
- `system_service.rs`: 系统权限和开发工具
- `update_service.rs`: 应用更新检查
- `config_service.rs`: 配置管理

### 前端页面
- 首页 (`/`): 状态监控和快速操作
- 代理管理 (`/proxy`): 节点选择和代理配置
- 订阅管理 (`/sub`): 订阅添加和管理
- 连接监控 (`/connections`): 实时连接查看
- 日志查看 (`/log`): 系统日志监控
- 设置页面 (`/setting`): 应用配置
- 规则管理 (`/rules`): 路由规则查看

## 重要配置

### Rust 性能优化配置
- 开发模式: `opt-level = 1` (平衡编译速度和性能)
- 发布模式: `opt-level = 3` (最大性能优化)

### 前端开发配置
- Vite 开发服务器端口: 6221
- 自动导入 Naive UI 组件
- TypeScript 严格模式

## 国际化支持
支持多语言，语言文件位于 `src/locales/`:
- `en-US.ts`: 英文
- `zh-CN.ts`: 中文
- `ja-JP.ts`: 日文
- `ru-RU.ts`: 俄文

## 注意事项

### 开发相关
- 使用 pnpm 作为包管理器
- 前端热重载端口固定为 6221
- Rust 后端支持 Windows 特定的系统集成

### 内存管理
- StoreManager 负责状态的生命周期管理
- WebSocket 连接需要在应用关闭时正确清理
- 临时 Store 有全局内存监控机制

### Tauri 命令
所有前端到后端的通信都通过 Tauri 命令进行，命令定义在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中。