# Sing-Box Windows

<div align="center">
    <img src="./src/assets/icon.png" alt="Sing-Box Windows Logo" width="128" height="128" />
    <h1>Sing-Box Windows</h1>
    <p>🚀 一个优雅的 Sing-Box Windows GUI 客户端</p>
    <p>
        <img src="https://img.shields.io/github/license/xinggaoya/sing-box-windows" alt="license" />
        <img src="https://img.shields.io/github/stars/xinggaoya/sing-box-windows" alt="stars" />
        <img src="https://img.shields.io/github/downloads/xinggaoya/sing-box-windows/total" alt="downloads" />
        <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows" alt="release" />
    </p>
    <p>
        <a href="README.zh.md">中文</a> | 
        <a href="README.md">English</a>
    </p>
</div>

## 🎯 特性

- 🖥️ 现代化的用户界面，基于 [Tauri 2.0](https://tauri.app/) 和 [Vue 3](https://vuejs.org/)
- 🌙 支持亮色/暗色主题，自动跟随系统
- 🔄 支持多种订阅链接导入和自动更新
- 🌐 全面的代理模式支持
- 📊 丰富的统计功能
- 🔍 完整的日志系统
- ⚡ 优化的性能表现
- 🔒 安全特性
- 🌍 国际化支持

## 📸 预览

<img src="./public/image.png" alt="sing-box-windows 预览" width="800">

## 🚀 快速开始

### 下载安装

1. 从 [Releases](https://github.com/xinggaoya/sing-box-windows/releases) 页面下载最新版本
2. 运行安装程序（支持自动更新）
3. 首次启动会自动完成必要配置

### 基本使用

1. 首次使用请在【设置】中下载并安装 Sing-Box 内核
2. 在【订阅】页面添加或导入您的订阅链接
   - 支持直接输入链接
   - 支持sing-box json的配置，base64自动解码
3. 在【主页】中选择节点并连接
   - 支持快速切换节点
   - 支持节点延迟测试
   - 支持节点分组管理

> 提示：使用 TUN 模式时，程序会请求管理员权限并自动配置系统设置

### 进阶功能

- **规则设置**：支持自定义分流规则
- **快捷操作**：支持系统托盘快速操作
- **配置备份**：支持配置导出和恢复
- **自动化**：支持开机自启和自动连接

## 🛠️ 开发指南

### 环境要求

- [Node.js](https://nodejs.org/) 18.0 或更高版本
- [Rust](https://www.rust-lang.org/) 最新稳定版
- [Visual Studio](https://visualstudio.microsoft.com/) 2019 或更高版本（需包含 C++ 开发工具）
- [Git](https://git-scm.com/) 最新版本
- [pnpm](https://pnpm.io/) 包管理器

### 本地开发

```bash
# 克隆项目
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

### 更多开发文档

详细的开发文档请查看 [开发文档](./docs/development.md)，其中包含了项目结构、核心功能模块、开发规范等更多内容。

### 项目结构

```
sing-box-windows/
├── src/                # 前端源代码
│   ├── assets/        # 静态资源
│   ├── components/    # 通用组件
│   ├── router/        # 路由配置
│   ├── stores/        # 状态管理
│   ├── utils/         # 工具函数
│   └── views/         # 页面组件
├── src-tauri/         # Rust 后端代码
│   ├── src/           # 源代码
│   └── Cargo.toml     # Rust 依赖配置
└── package.json       # 项目配置
```

## 📦 技术栈

- 🎯 [Tauri 2.0](https://tauri.app/) - 现代化跨平台应用框架
- ⚡ [Vue 3](https://vuejs.org/) - 响应式前端框架
- 🎨 [Naive UI](https://www.naiveui.com/) - 高质量 Vue 3 组件库
- 📊 [Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API) - 高性能的图形渲染
- 🦀 [Rust](https://www.rust-lang.org/) - 高性能系统编程语言
- 🔧 [TypeScript](https://www.typescriptlang.org/) - 类型安全的 JavaScript

## 🤝 贡献指南

我们非常欢迎各种形式的贡献，包括但不限于：

- 🐛 问题报告和建议
- 📝 文档改进
- 🔧 代码修复
- ✨ 新功能开发
- 🌍 多语言支持

贡献流程：

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 提交 Pull Request

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 📮 联系方式

- 🐛 问题反馈：[GitHub Issues](https://github.com/xinggaoya/sing-box-windows/issues)
- 💬 讨论：[GitHub Discussions](https://github.com/xinggaoya/sing-box-windows/discussions)

## ⭐ 鸣谢

- [sing-box](https://github.com/SagerNet/sing-box) - 核心代理引擎
- [Tauri](https://tauri.app/) - 应用框架
- [Vue](https://vuejs.org/) - 前端框架
- [Naive UI](https://www.naiveui.com/) - UI 组件库
- [社区贡献者们](https://github.com/xinggaoya/sing-box-windows/graphs/contributors)

---

如果这个项目对你有帮助，欢迎给一个 Star ⭐️
