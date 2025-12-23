<div align="center">
  <img src="src/assets/icon.png" alt="Sing-Box GUI 客户端" width="120" height="120">

  <h1>Sing-Box Windows</h1>

  <p>
    <strong>基于 Tauri 2.0 + Vue 3 构建的现代化 Sing-Box 跨平台 GUI 客户端 (Windows + Linux + macOS)</strong>
  </p>

  <p>
    <a href="#功能特性">✨ 功能特性</a> •
    <a href="#安装指南">🚀 安装指南</a> •
    <a href="#快速开始">🎯 快速开始</a> •
    <a href="#界面截图">📸 界面截图</a> •
    <a href="#开发指南">🛠️ 开发指南</a> •
    <a href="#贡献指南">💡 贡献指南</a>
  </p>

  <p>
    <a href="README.zh.md">🇨🇳 中文</a> •
    <a href="README.md">🇺🇸 English</a>
  </p>

  <p>
    <a href="https://github.com/xinggaoya/sing-box-windows/releases">
      <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows?style=for-the-badge&logo=github" alt="GitHub release">
    </a>
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=for-the-badge&logo=apple" alt="平台支持">
    <img src="https://img.shields.io/badge/version-1.8.2-informational?style=for-the-badge" alt="版本">
    <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="许可证">
  </p>
</div>

---

## 🌟 项目介绍

Sing-Box GUI 客户端是一款尖端的跨平台应用程序（支持 Windows、Linux 和 macOS），为管理 Sing-Box 代理配置提供了现代化、直观的界面。基于强大的 Tauri 2.0 和 Vue 3 技术栈构建，提供卓越的性能、安全性和用户体验。

> **📱 Android 用户**：本项目也有 Android 版本，欢迎访问 [sing-box-windows-android](https://github.com/xinggaoya/sing-box-windows-android) 体验！

### 🎯 核心亮点

- **🚀 极速性能**: Rust 后端 + 优化的 Vue 3 前端
- **💎 现代界面**: 基于 Naive UI 的美观响应式设计
- **🔒 安全可靠**: Rust 后端确保内存安全和性能
- **🌐 实时通信**: WebSocket 实现实时数据更新
- **🎨 高度可定制**: 明暗主题切换，丰富的个性化选项

---

## ✨ 功能特性

### 🎯 核心功能
- **代理管理**: 轻松配置和切换不同代理模式
- **订阅管理**: 自动订阅更新和管理
- **连接监控**: 实时连接统计和监控
- **日志管理**: 完整的日志记录，支持过滤和搜索
- **规则管理**: 可视化路由规则配置和管理
- **系统集成**: 系统托盘支持，快速访问控制

### 🎨 用户体验
- **现代界面**: 基于 Naive UI 组件的简洁直观设计
- **多语言支持**: 简体中文、英文、日语、俄语
- **主题系统**: 明亮、暗黑、自动主题，支持自定义颜色
- **响应式设计**: 适配各种屏幕尺寸和 DPI 设置

### 🔧 高级特性
- **实时统计**: 通过 WebSocket 实现实时带宽和连接监控
- **自动启动**: 系统开机自动启动，支持延迟启动
- **更新管理**: 自动检查和安装更新
- **内存优化**: 遵循标准的 Vue.js 和 Pinia 实践，实现高效的资源管理。

---

## 🚀 安装

从 [GitHub Releases](https://github.com/xinggaoya/sing-box-windows/releases) 下载对应平台的安装包：

- **Windows**: `.exe` 安装程序
- **Linux**: `.deb` 包或 `.AppImage` 便携版
- **macOS**: `.dmg` 镜像文件或 `.app` 压缩包

详细安装说明请参考项目文档。

---

## 🎯 快速开始

1. **安装应用**：下载并安装 sing-box-windows
2. **安装内核**：首次启动时，进入设置页面下载并安装 Sing-Box 内核
3. **添加订阅**：在订阅页面添加您的订阅链接
4. **选择代理**：在代理页面选择节点并设置代理模式
5. **开始使用**：启用代理，享受网络访问

⚠️ **重要提示**：必须先安装内核才能添加和使用订阅，内核是运行代理服务的核心组件。

详细使用教程请参考项目文档。

---

## 📦 支持的订阅格式

- **sing-box JSON 配置**（顶层 `outbounds`）：`vless`、`vmess`、`trojan`、`shadowsocks`、`shadowsocksr`、`socks`、`http`、`hysteria2`
- **Clash/Mihomo YAML**（`proxies`）：`vmess`、`vless`、`trojan`、`ss`（shadowsocks）
- **URI 列表**（每行一个节点）：`vmess://`、`vless://`、`trojan://`、`ss://`、`hysteria2://`
- **说明**：YAML 的 `hysteria2` 暂未解析

---

## 📸 界面截图

<div align="center">
  <img src="docs/image.png" alt="Sing-Box GUI 客户端界面" width="800">

  <p><em>应用主界面，展示代理管理和系统状态</em></p>
</div>

---

## 🛠️ 开发

```bash
# 克隆仓库
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# 安装依赖
pnpm install

# 启动开发模式
pnpm tauri dev

# 构建发布版本
pnpm tauri build
```

详细开发文档请参考项目说明。

---

## 🔧 存储

应用采用 **本地存储** 方案，所有配置和订阅数据都保存在本地：

- **Windows**: `%APPDATA%\sing-box-windows\`
- **Linux**: `~/.local/share/sing-box-windows/`
- **macOS**: `~/Library/Application Support/sing-box-windows/`

数据包括应用设置、订阅信息、主题配置等，确保用户隐私和数据安全。

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. **报告问题**: [提交 Issue](https://github.com/xinggaoya/sing-box-windows/issues)
2. **功能建议**: [参与讨论](https://github.com/xinggaoya/sing-box-windows/discussions)
3. **代码贡献**: Fork 并提交 PR

---

## 🙏 致谢

- [Sing-Box](https://github.com/SagerNet/sing-box) - 强大的代理核心
- [Tauri](https://tauri.app/) - 优秀的 Rust 应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Naive UI](https://www.naiveui.com/) - 美观的 Vue 3 组件库
- [Vite](https://vitejs.dev/) - 快速的构建工具和开发服务器

---

---

<div align="center">
  <p>
    <strong>由 XingGao 用 ❤️ 构建</strong>
  </p>
  <p>
    <a href="https://github.com/xinggaoya/sing-box-windows">
      <img src="https://img.shields.io/github/stars/xinggaoya/sing-box-windows?style=social" alt="GitHub stars">
    </a>
    <a href="https://github.com/xinggaoya/sing-box-windows/fork">
      <img src="https://img.shields.io/github/forks/xinggaoya/sing-box-windows?style=social" alt="GitHub forks">
    </a>
  </p>
  <p style="font-size: 0.85em; color: #666;">
    <strong>📝 声明：</strong>本项目仅用于学习和交流目的，所有数据均采用本地存储，不会上传到云端。<br>
    请遵守当地法律法规，合理使用本软件。
  </p>
</div>
