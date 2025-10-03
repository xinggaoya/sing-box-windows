# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

sing-box-windows 是一个基于 Tauri 2.0 + Vue 3 的现代化 Windows 代理客户端，提供完整的代理管理、路由规则、订阅管理和系统服务功能。

### 技术栈

- **前端**: Vue 3 + TypeScript + Vite + Pinia + Naive UI
- **后端**: Rust + Tauri 2.0 + tokio
- **架构**: MVVM + 模块化设计
- **持久化**: Tauri Store (替代 localStorage)
- **构建**: Vite (前端) + cargo-cp-artifact (Rust 后端)

## 常用命令

### 开发环境
```bash
# 安装依赖
pnpm install
cd src-tauri && cargo fetch

# 启动开发服务器
pnpm tauri dev
```

### 构建和打包
```bash
# 构建生产版本
pnpm tauri build

# 构建 MSI 安装包
pnpm tauri build -- --target x86_64-pc-windows-msi

# 构建时跳过目标检查
pnpm tauri build -- --target x86_64-pc-windows-msi --no-target-check
```

### 代码质量
```bash
# ESLint 检查
pnpm lint

# ESLint 自动修复
pnpm lint:fix

# TypeScript 类型检查
pnpm type-check

# 代码格式化 (需要手动安装 rustfmt)
cd src-tauri && cargo fmt
```

### 测试
```bash
# 前端单元测试
pnpm test

# Rust 单元测试
cd src-tauri && cargo test

# Rust 文档测试
cd src-tauri && cargo test --doc
```

## 核心架构

### 前端架构

```
src/
├── stores/          # Pinia 状态管理 (自定义 StoreManager)
│   ├── app/        # 应用相关 stores (AppStore, ThemeStore, LocaleStore等)
│   ├── kernel/     # 内核相关 stores (KernelStore, ProxyStore, TrafficStore等)
│   ├── subscription/ # 订阅相关 store
│   └── tray/       # 系统托盘 store
├── components/      # Vue 组件
│   ├── layout/     # 布局组件 (MainLayout, TrafficChart)
│   ├── home/       # 首页组件 (ProxyModeCard, TrafficStatsCard等)
│   └── utils/      # 工具组件 (LazyComponent, VirtualList, UpdateModal)
├── views/           # 页面视图
├── services/        # API 服务层 (websocket-service, tauri命令封装)
├── utils/           # 工具函数 (内存管理、性能优化)
├── locales/         # 国际化文件
└── types/           # TypeScript 类型定义
```

### 后端架构

```
src-tauri/src/
├── app/             # 应用服务层
│   ├── core/       # 核心服务 (kernel_service, proxy_service)
│   ├── network/    # 网络服务 (subscription_service)
│   ├── system/     # 系统服务 (system_service, update_service)
│   └── constants/  # 常量定义
├── entity/          # 数据实体模型
├── process/         # 进程管理
├── utils/           # 工具函数
├── main.rs          # 程序入口
└── lib.rs           # 库入口和命令注册
```

## 关键特性

### 1. Store 管理系统
- **自定义 StoreManager**: 支持按需加载和内存优化
- **防抖持久化**: 自动防抖保存状态到 Tauri Store
- **内存泄漏检测**: 内置内存泄漏检测和自动清理机制

### 2. 前后端通信
- **Tauri Commands**: 所有前端调用通过 Tauri 命令
- **统一错误处理**: 后端统一返回 Result<T, String> 格式
- **类型安全**: 使用 typescript-bindings 保证类型安全

### 3. 模块化设计
- **组件按功能分组**: 每个功能模块独立的组件目录
- **服务层抽象**: API 调用封装在 services/ 目录
- **类型定义集中**: 统一的 TypeScript 类型定义

### 4. 性能优化
- **虚拟滚动**: 使用自定义 VirtualList.vue 组件优化大列表
- **懒加载**: 使用 LazyComponent.vue 实现组件按需加载
- **内存管理**: 内置内存泄漏检测和 WebSocket 连接清理
- **自动导入**: 使用 unplugin-auto-import 和 unplugin-vue-components

## 开发规范

### 新增功能步骤
1. 在 `types/` 中定义 TypeScript 类型
2. 在 `stores/` 中创建状态管理 (如需要)
3. 在 `services/` 中创建 API 服务层
4. 在 `components/` 中创建 UI 组件
5. 在 `views/` 中创建页面视图
6. 在 `src-tauri/commands/` 中实现后端命令
7. 更新路由配置 (如需要)

### 组件命名规范
- **页面组件**: 使用 PascalCase，如 `ProxyPage.vue`
- **功能组件**: 使用 PascalCase，如 `ProxyConfig.vue`
- **工具组件**: 使用 PascalCase，如 `LoadingSpinner.vue`

### 状态管理规范
- **Store 命名**: 功能名 + Store，如 `proxyStore`
- **Action 命名**: 动词 + 名词，如 `updateConfig`
- **State 命名**: 使用 camelCase，避免缩写

## 调试指南

### 前端调试
- 开发环境下自动集成 Vue DevTools
- 使用 console.log 或 debugger 进行断点调试
- 网络请求通过浏览器开发者工具查看

### 后端调试
- 使用 `println!` 或 `log::info!` 输出调试信息
- 查看控制台输出获取 Rust 日志
- 复杂逻辑可以使用 VS Code 调试器 (需要配置 launch.json)

## 重要文件

### 配置文件
- `src-tauri/tauri.conf.json`: Tauri 应用配置
- `src-tauri/Cargo.toml`: Rust 依赖配置
- `package.json`: Node.js 依赖配置
- `vite.config.ts`: Vite 构建配置

### 核心文件
- `src/stores/index.ts`: Store 管理系统入口
- `src/stores/StoreManager.ts`: Store 生命周期管理器
- `src/services/websocket-service.ts`: WebSocket 通信服务
- `src/utils/memory-leak-fix.ts`: 内存管理工具
- `src-tauri/src/lib.rs`: Tauri 入口文件和命令注册
- `src-tauri/src/app/`: 后端服务层实现

## 注意事项

1. **内存管理**: 长时间运行的应用需要特别注意内存泄漏
2. **错误处理**: 所有异步操作都需要适当的错误处理
3. **类型安全**: 优先使用 TypeScript 类型，避免 any 类型
4. **性能优化**: 大数据量操作使用虚拟滚动或分页
5. **跨平台**: 虽然主要针对 Windows，但要保持代码的可移植性

## 常见问题

### 构建问题
- 如果遇到链接错误，检查 Visual Studio Build Tools 是否正确安装
- 如果遇到依赖问题，尝试删除 `node_modules` 和 `src-tauri/target` 后重新安装

### 开发问题
- 热重载不工作时，检查端口是否被占用
- Tauri 命令调用失败时，检查后端命令是否正确注册