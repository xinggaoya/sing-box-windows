<div align="center">
  <img src="src/assets/icon.png" alt="Sing-Box GUI 客户端" width="120" height="120">

  <h1>Sing-Box Windows</h1>

  <p>
    <strong>基于 Tauri 2.0 + Vue 3 构建的现代化 Sing-Box Windows GUI 客户端</strong>
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
    <a href="https://github.com/xinggaoya/sing-box-windows/releases">
      <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows?style=for-the-badge&logo=github" alt="GitHub release">
    </a>
    <img src="https://img.shields.io/badge/platform-Windows-blue?style=for-the-badge&logo=windows" alt="平台支持">
    <img src="https://img.shields.io/badge/version-1.8.2-informational?style=for-the-badge" alt="版本">
    <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="许可证">
  </p>
</div>

---

## 🌟 项目介绍

Sing-Box GUI 客户端是一款尖端的 Windows 应用程序，为管理 Sing-Box 代理配置提供了现代化、直观的界面。基于强大的 Tauri 2.0 和 Vue 3 技术栈构建，提供卓越的性能、安全性和用户体验。

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
- **内存优化**: 通过 StoreManager 实现高效的资源管理和清理

---

## 🚀 安装指南

### 📥 系统要求

- **操作系统**: Windows 10/11 (x64)
- **内存**: 最低 4GB RAM (推荐 8GB)
- **存储空间**: 100MB 可用磁盘空间
- **网络**: 订阅和更新需要网络连接

### 🎯 安装方式

#### 方式一：下载安装包（推荐）
1. 访问 [发布页面](https://github.com/xinggaoya/sing-box-windows/releases)
2. 下载最新的 `sing-box-windows-x.x.x-setup.exe`
3. 运行安装程序并按照向导操作
4. 从开始菜单或桌面快捷方式启动应用

#### 方式二：便携版
1. 下载最新的 `sing-box-windows-x.x.x-portable.zip`
2. 解压到您想要的文件夹
3. 运行 `sing-box-windows.exe`

#### 方式三：包管理器（即将支持）
```powershell
# 使用 winget（计划中）
winget install sing-box-windows

# 使用 Chocolatey（计划中）
choco install sing-box-windows
```

---

## 🎯 快速开始

### 1. 初始设置
安装完成后，应用程序将引导您完成初始设置：

1. **语言选择**: 选择您偏好的语言
2. **主题配置**: 选择明亮/暗黑/自动主题
3. **网络设置**: 配置基本网络设置

### 2. 添加订阅
1. 导航到 **订阅** 选项卡
2. 点击 **添加订阅**
3. 输入您的订阅 URL 和名称
4. 配置更新设置
5. 点击 **保存并更新**

### 3. 配置代理
1. 转到 **代理** 选项卡
2. 从列表中选择您偏好的服务器
3. 选择代理模式（全局/规则/直连）
4. 切换 **连接** 按钮以激活

### 4. 监控连接
1. 访问 **连接** 选项卡
2. 查看实时连接统计
3. 监控带宽使用情况
4. 过滤和搜索连接

---

## 📸 界面截图

<div align="center">
  <img src="docs/image.png" alt="Sing-Box GUI 客户端界面" width="800">

  <p><em>应用主界面，展示代理管理和系统状态</em></p>
</div>

---

## 🛠️ 开发指南

### 📋 开发环境要求

- **Node.js**: 18+ (推荐使用 [pnpm](https://pnpm.io/))
- **Rust**: 1.70+ 带有 nightly 工具链
- **Tauri CLI**: 最新版本

### 🚀 搭建开发环境

1. **克隆仓库**
   ```bash
   git clone https://github.com/xinggaoya/sing-box-windows.git
   cd sing-box-windows
   ```

2. **安装依赖**
   ```bash
   # 安装前端依赖
   pnpm install

   # 安装 Rust 依赖（自动）
   ```

3. **启动开发服务器**
   ```bash
   # 启动开发模式（热重载）
   pnpm tauri dev
   ```

4. **构建生产版本**
   ```bash
   # 构建发布版本
   pnpm tauri build
   ```

### 🧪 开发命令

```bash
# 开发
pnpm tauri dev          # 启动开发服务器（热重载）

# 构建
pnpm tauri build        # 构建发布版本

# 代码质量
pnpm lint               # 运行 ESLint 和 OXLint 自动修复
pnpm format             # 使用 Prettier 格式化代码
pnpm type-check         # TypeScript 类型检查

# 仅前端
pnpm dev                # 启动 Vite 开发服务器
pnpm build              # 仅构建前端
```

### 🏗️ 项目结构

```
sing-box-windows/
├── 📁 src/                    # 前端 (Vue 3)
│   ├── 📁 components/         # 可复用组件
│   │   ├── 📁 home/          # 仪表板组件
│   │   ├── 📁 layout/        # 布局组件
│   │   └── 📁 common/        # 通用 UI 组件
│   ├── 📁 stores/            # Pinia 状态管理
│   ├── 📁 services/          # 业务逻辑服务
│   ├── 📁 utils/             # 工具函数
│   ├── 📁 locales/           # 国际化
│   └── 📁 views/             # 页面组件
├── 📁 src-tauri/             # 后端 (Rust)
│   ├── 📁 src/
│   │   ├── 📁 app/           # 应用模块
│   │   │   ├── 📁 core/      # 核心功能
│   │   │   ├── 📁 network/   # 网络操作
│   │   │   └── 📁 system/    # 系统集成
│   │   └── 📄 main.rs        # 应用入口点
│   └── 📄 Cargo.toml         # Rust 依赖
└── 📁 docs/                  # 文档
```

---

## 🔧 配置说明

### 📄 存储系统

本应用使用 **Tauri Store 插件** 进行后端数据持久化，提供安全高效的二进制数据库文件存储。

### 🗄️ 存储位置

- **Windows**: `%APPDATA%\sing-box-windows\*.bin` (二进制数据库文件)
- **便携版**: `<应用目录>\*.bin` (二进制数据库文件)

### ⚙️ 配置结构

应用使用 Tauri Store 插件将配置数据存储在结构化的二进制文件中。主要配置区域包括：

#### 应用设置 (`app.bin`)
```typescript
{
  language: "en-US" | "zh-CN" | "ja-JP" | "ru-RU",
  theme: "light" | "dark" | "auto",
  proxyMode: "system" | "tun" | "manual",
  autoStartKernel: boolean,
  preferIpv6: boolean,
  proxyPort: number,      // 默认: 12080
  apiPort: number         // 默认: 12081
}
```

#### 主题设置 (`theme.bin`)
```typescript
{
  primaryColor: string,
  isDark: boolean,
  followSystem: boolean
}
```

#### 订阅数据 (`subscription.bin`)
```typescript
{
  subscriptions: Array<{
    id: string,
    name: string,
    url: string,
    autoUpdate: boolean,
    lastUpdate: string
  }>
}
```

### 🔧 数据持久化特性

- **二进制存储**: 使用高效的二进制格式进行快速读写操作
- **自动同步**: 前端和后端之间的实时数据同步
- **内存优化**: 针对大数据集的智能缓存和懒加载
- **数据完整性**: 内置验证和错误恢复机制
- **跨会话持久化**: 所有设置和数据在应用重启后保持不变

---

## 🤝 贡献指南

我们欢迎社区贡献！以下是如何参与的方式：

### 🎯 贡献方式

1. **🐛 报告问题**: 发现了问题？[提交 issue](https://github.com/xinggaoya/sing-box-windows/issues)
2. **💡 功能建议**: 有好的想法？[在讨论区提出](https://github.com/xinggaoya/sing-box-windows/discussions)
3. **🔧 代码贡献**: Fork 并提交 Pull Request
4. **📖 文档改进**: 帮助完善文档
5. **🌐 本地化**: 参与 [翻译工作](src/locales/)

### 🚀 开发流程

1. 从 [GitHub](https://github.com/xinggaoya/sing-box-windows) Fork 仓库
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 进行修改并充分测试
4. 提交更改: `git commit -m 'Add amazing feature'`
5. 推送到分支: `git push origin feature/amazing-feature`
6. 创建 Pull Request

### 📋 代码规范

- **前端**: 遵循 [Vue 3 风格指南](https://vuejs.org/style-guide/)
- **后端**: 遵循 [Rust API 指南](https://rust-lang.github.io/api-guidelines/)
- **提交**: 使用 [约定式提交](https://www.conventionalcommits.org/zh-hans/)
- **代码检查**: 代码需通过 ESLint 和 OXLint 检查

---

## 🙏 致谢

- [Sing-Box](https://github.com/SagerNet/sing-box) - 强大的代理核心
- [Tauri](https://tauri.app/) - 优秀的 Rust 应用框架
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Naive UI](https://www.naiveui.com/) - 美观的 Vue 3 组件库
- [Vite](https://vitejs.dev/) - 快速的构建工具和开发服务器

---

## 📄 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

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
</div>