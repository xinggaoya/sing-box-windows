<div align="center">
  <img src="src/assets/icon.png" alt="Sing-Box GUI Client" width="120" height="120">

  <h1>Sing-Box Windows</h1>

  <p>
    <strong>基于 Tauri 2 + Vue 3 构建的 sing-box 跨平台图形客户端，支持 Windows、Linux 和 macOS</strong>
  </p>

  <p>
    <a href="#功能特性">✨ 功能特性</a> •
    <a href="#架构概览">🏗️ 架构概览</a> •
    <a href="#快速开始">🚀 快速开始</a> •
    <a href="#开发">🛠️ 开发</a> •
    <a href="#文档">📚 文档</a>
  </p>

  <p>
    <a href="README.zh.md">🇨🇳 中文</a> •
    <a href="README.md">🇺🇸 English</a>
  </p>

  <p>
    <a href="https://github.com/xinggaoya/sing-box-windows/releases">
      <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows?style=for-the-badge&logo=github" alt="GitHub release">
    </a>
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=for-the-badge&logo=tauri" alt="Platform">
    <img src="https://img.shields.io/badge/stack-Tauri%202%20%2B%20Vue%203-purple?style=for-the-badge" alt="Stack">
    <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License">
  </p>
</div>

---

## 项目介绍

Sing-Box Windows 是一个围绕 sing-box 内核构建的现代桌面客户端，目标是把订阅导入、代理模式切换、运行状态查看和桌面集成整合成一套更顺手的跨平台体验。

> Android 用户也可以查看 [sing-box-windows-android](https://github.com/xinggaoya/sing-box-windows-android)。

## 功能特性

- **订阅导入**：支持 sing-box JSON、Clash/Mihomo YAML、URI 列表三类输入。
- **代理模式**：支持系统代理、TUN 模式、手动模式切换。
- **运行态可视化**：实时查看流量、活跃连接、规则和日志。
- **内核管理**：支持内嵌内核资源、版本下载以及手动导入内核。
- **桌面集成**：支持托盘、开机启动、更新检查、窗口状态记忆。
- **可靠性能力**：提供备份恢复、订阅回滚、更新通道与内核自愈重启能力。
- **多语言**：内置 `zh-CN`、`en-US`、`ja-JP`、`ru-RU`。

## 架构概览

- **前端**：Vue 3 + TypeScript + Pinia + Vue Router + Vue I18n + Naive UI。
- **桥接层**：通过 Tauri `invoke` 执行命令，通过 Tauri event 推送运行时事件。
- **后端**：Rust + Tauri 2，按 `core`、`network`、`storage`、`system`、`tray` 分层。
- **存储**：基于 SQLite 的本地持久化，保存应用设置、订阅、语言、主题、更新偏好等数据。
- **构建链路**：`scripts/tauri-wrapper.mjs` 会在 `tauri dev/build` 前注入目标平台对应的内核资源。

## 支持的订阅格式

| 格式 | 输入方式 | 说明 |
| --- | --- | --- |
| sing-box JSON | 原始 JSON 配置 | 解析顶层 `outbounds` |
| Clash / Mihomo YAML | YAML 订阅 | 解析 `proxies` 并转换支持的节点 |
| URI 列表 | `vmess://`、`vless://`、`trojan://`、`ss://`、`hysteria2://` | 支持单条或多条节点 |

当前支持的节点类型包括 `vless`、`vmess`、`trojan`、`shadowsocks`、`shadowsocksr`、`socks`、`http`、`hysteria2`。

## 快速开始

1. 从 [GitHub Releases](https://github.com/xinggaoya/sing-box-windows/releases) 下载最新安装包。
2. 在 **订阅** 页面导入订阅内容。
3. 在 **代理** 页面选择节点并设置代理模式。
4. 回到 **首页** 启动内核，并在相关页面检查流量或连接状态。

## 界面截图

<div align="center">
  <img src="docs/image.png" alt="Sing-Box GUI Client Interface" width="880">
  <p><em>主界面包含代理控制、运行状态与多页面导航</em></p>
</div>

## 开发

```bash
# 安装依赖
pnpm install

# 首次环境建议执行；dev/build 在缺失时也会自动拉取
pnpm kernel:fetch

# 启动桌面开发环境
pnpm tauri dev

# 前端检查
pnpm type-check
pnpm lint

# 后端检查
cd src-tauri && cargo clippy && cargo test

# 构建发布产物
pnpm tauri build
```

## 本地数据目录

所有数据默认都保存在本机，不上传云端。

- **Windows**：`%LOCALAPPDATA%\sing-box-windows\`
- **Linux**：`~/.local/share/sing-box-windows/`
- **macOS**：`~/Library/Application Support/sing-box-windows/`

目录中通常包含 SQLite 数据库、生成后的配置文件、日志、备份文件以及已下载的内核元数据。

## 文档

- [English README](README.md)
- [开发文档](docs/development.md)
- [多语言文档](docs/i18n.md)
- [更新日志](docs/CHANGELOG.md)

## 贡献

欢迎提交 Issue 和 Pull Request。

1. 在 [Issues](https://github.com/xinggaoya/sing-box-windows/issues) 报告问题
2. 在 [Discussions](https://github.com/xinggaoya/sing-box-windows/discussions) 讨论需求
3. 提交改动时尽量保持与现有架构和质量门禁一致

## 致谢

- [Sing-Box](https://github.com/SagerNet/sing-box)
- [Tauri](https://tauri.app/)
- [Vue.js](https://vuejs.org/)
- [Naive UI](https://www.naiveui.com/)
- [Vite](https://vitejs.dev/)

---

<div align="center">
  <p><strong>由 XingGao 用 ❤️ 构建</strong></p>
  <p>
    <strong>声明：</strong>本项目仅用于学习与交流，请遵守当地法律法规并合理使用。
  </p>
</div>
