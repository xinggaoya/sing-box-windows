# Sing-Box Windows

<div align="center">
    <img src="./src/assets/icon.png" alt="Sing-Box Windows Logo" width="128" height="128" />
    <h1>Sing-Box Windows</h1>
    <p>🚀 一个优雅的 Sing-Box Windows GUI 客户端</p>
    <p>
        <img src="https://img.shields.io/github/license/xinggaoya/sing-box-windows" alt="license" />
        <img src="https://img.shields.io/github/stars/xinggaoya/sing-box-windows" alt="stars" />
    </p>
</div>

## 🎯 特性

- 🖥️ 现代化的用户界面，基于 [Tauri 2.0](https://tauri.app/) 和 [Vue 3](https://vuejs.org/)
- 🌙 支持亮色/暗色主题
- 🔄 支持订阅链接导入和更新
- 🌐 系统代理和 TUN 模式支持
- 📊 流量统计和图表显示
- 🔍 实时日志查看
- ⚡ 低内存占用，高性能

## 📸 预览
<img src="./public/image.png" alt="sing-box-windows 预览" width="800">

## 🚀 快速开始

### 下载安装
1. 从 [Releases](https://github.com/xinggaoya/sing-box-windows/releases) 页面下载最新版本
2. 运行安装程序
3. 启动应用程序

### 基本使用
1. 首次使用请在【设置】中下载 Sing-Box 内核
2. 在【订阅】页面导入或更新您的订阅链接
3. 在【主页】中选择节点，开启系统代理即可使用

> 注意：TUN 模式需要以管理员身份运行程序

## 🛠️ 开发指南

### 环境要求
- [Node.js](https://nodejs.org/) 18.0 或更高版本
- [Rust](https://www.rust-lang.org/) 最新稳定版
- [Visual Studio](https://visualstudio.microsoft.com/) 2019 或更高版本（包含 C++ 开发工具）
- [Git](https://git-scm.com/) 用于版本控制

### 本地开发
```bash
# 克隆项目
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## 📦 技术栈

- 🎯 [Tauri 2.0](https://tauri.app/) - 构建跨平台应用
- ⚡ [Vue 3](https://vuejs.org/) - 渐进式 JavaScript 框架
- 🎨 [Naive UI](https://www.naiveui.com/) - Vue 3 组件库
- 📊 [ECharts](https://echarts.apache.org/) - 数据可视化图表库
- 🦀 [Rust](https://www.rust-lang.org/) - 系统编程语言
- 🔧 [TypeScript](https://www.typescriptlang.org/) - JavaScript 的超集

## 🤝 贡献指南

我们非常欢迎各种形式的贡献，包括但不限于：

- 🐛 提交问题和建议
- 📝 改进文档
- 🔧 提交代码修复
- ✨ 提供新功能

请确保在提交 Pull Request 之前：
1. Fork 本仓库
2. 创建新的功能分支
3. 提交代码并确保代码风格一致
4. 提交 Pull Request

## 📄 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 📮 联系方式

- 📧 邮箱：[xinggaoya@qq.com](mailto:xinggaoya@qq.com)
- 🐛 问题反馈：[GitHub Issues](https://github.com/xinggaoya/sing-box-windows/issues)

## ⭐ 鸣谢

- [sing-box](https://github.com/SagerNet/sing-box)
- [Tauri](https://tauri.app/)
- [Vue](https://vuejs.org/)

---

如果这个项目对你有帮助，欢迎给一个 Star ⭐️
