# sing-box-windows Linux 适配总结

## 概述

本项目已成功完成从 Windows 到 Linux 的跨平台适配，将 sing-box-windows 应用转换为支持 Linux 的通用代理客户端。

## 主要变更

### 1. 项目名称调整
- 保持原有 `sing-box-windows` 名称，但实际功能已支持跨平台
- 内部包名更新为 `sing-box-universal` 以体现跨平台特性

### 2. Rust 后端适配

#### 2.1 依赖管理
```toml
# 新增 Linux 平台依赖
[target.'cfg(unix)'.dependencies]
nix = { version = "0.29", features = ["user"] }
which = "7.0"

# Windows 特定依赖条件化
[target.'cfg(windows)'.dependencies]
winreg = "0.52.0"
winapi = { version = "0.3", features = ["wininet"] }
```

#### 2.2 跨平台实现

**系统服务适配** (`src/app/system/system_service.rs`)
- Windows: 使用 VBS 脚本实现 UAC 权限提升
- Linux: 使用 `pkexec`、`gksu`、`kdesudo` 等工具实现权限提升
- 权限检查: Windows 通过 `net session`，Linux 通过 `nix::unistd::getuid()`

**进程管理适配** (`src/process/manager.rs`)
- Windows: 使用 `tasklist`/`taskkill` 管理进程
- Linux: 使用 `pgrep`/`pkill` 管理进程
- 跨平台进程创建配置，Windows 支持隐藏控制台窗口

**系统代理适配** (`src/utils/proxy_util.rs`)
- Windows: 通过注册表设置系统代理
- Linux: 通过 `gsettings` 和环境变量设置代理
- 支持 GNOME 桌面环境的代理配置

### 3. 前端适配

#### 3.1 国际化更新
- 更新中文和英文的语言包，添加 Linux 相关的安装说明
- 修正 sing-box 可执行文件路径说明

#### 3.2 路径管理
- 应用工具类 (`src/utils/app_util.rs`) 支持跨平台路径
- Windows: `%LOCALAPPDATA%\sing-box-windows`
- Linux: `~/.local/share/sing-box-universal`

### 4. 构建配置

#### 4.1 Tauri 配置 (`tauri.conf.json`)
```json
{
  "bundle": {
    "targets": ["msi", "nsis", "deb", "appimage"],
    "linux": {
      "deb": {
        "depends": ["libwebkit2gtk-4.1-0", "libssl3", "libgtk-3-0"]
      }
    }
  }
}
```

#### 4.2 构建脚本
```json
{
  "scripts": {
    "tauri build:linux": "tauri build --target x86_64-unknown-linux-gnu",
    "tauri build:linux:deb": "tauri build --target x86_64-unknown-linux-gnu -- --bundle deb",
    "tauri build:linux:appimage": "tauri build --target x86_64-unknown-linux-gnu -- --bundle appimage"
  }
}
```

### 5. CI/CD 更新

#### 5.1 GitHub Actions
- 新增 Linux 构建任务 (`release-linux`)
- 支持生成 DEB 包和 AppImage 格式
- 自动化发布到 GitHub Releases

### 6. 文档完善

#### 6.1 新增文档
- `docs/linux-development.md` - Linux 开发指南
- `docs/linux-adaptation-summary.md` - 适配总结
- `src-tauri/app.desktop` - Linux 桌面文件

## 技术特性

### 跨平台兼容性
- ✅ Windows (x64)
- ✅ Linux (x64)
- 🔄 macOS (未来可扩展)

### 功能完整性
- ✅ 内核进程管理 (启动/停止/重启)
- ✅ 系统代理设置
- ✅ 权限管理
- ✅ 订阅管理
- ✅ 流量监控
- ✅ 连接状态显示

### Linux 特有特性
- 🎯 支持现代 Linux 桌面环境 (GNOME, KDE, XFCE)
- 🎯 集成 gsettings 进行系统代理设置
- 🎯 支持多种权限提升工具
- 🎯 环境变量代理配置
- 🎯 DEB 包和 AppImage 分发格式

## 系统要求

### 开发环境
- Linux (x64) with GTK 3
- Node.js 18+
- Rust 1.77.2+
- pnpm

### 运行环境
- libwebkit2gtk-4.1-0
- libssl3
- libgtk-3-0

## 安装方式

### 1. AppImage (推荐)
```bash
chmod +x sing-box-windows_*.AppImage
./sing-box-windows_*.AppImage
```

### 2. DEB 包
```bash
sudo dpkg -i sing-box-windows_*.deb
sudo apt-get install -f  # 如有依赖问题
```

### 3. 从源码构建
```bash
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows
pnpm install
pnpm tauri build:linux
```

## 测试验证

### 编译测试
```bash
✅ cargo check  # Rust 代码编译检查通过
✅ 前端类型检查通过
✅ 所有依赖正确解析
```

### 功能测试
- ✅ 应用启动正常
- ✅ 内核进程管理功能正常
- ✅ 系统代理设置功能正常
- ✅ 权限提升功能正常
- ✅ UI 界面适配正常

## 未来改进

### 短期目标
1. 🔄 添加 macOS 支持
2. 🔄 完善 Linux 发行版兼容性测试
3. 🔄 优化安装包体积

### 长期目标
1. 🔄 支持 ARM 架构 (arm64, aarch64)
2. 🔄 集成 Flatpak 分发
3. 🔄 添加系统集成服务

## 贡献指南

欢迎社区贡献代码和反馈问题！

1. Fork 项目仓库
2. 创建特性分支
3. 提交 Pull Request
4. 参与 Issues 讨论

## 总结

本次 Linux 适配成功地将原本仅支持 Windows 的 sing-box-windows 应用转换为跨平台代理客户端，保持了原有功能完整性的同时，针对 Linux 平台进行了深度优化和适配。项目现在具备了在主流操作系统上运行的能力，为更广泛的用户群体提供服务。

---

**适配完成时间**: 2025年10月21日
**适配工程师**: Claude AI
**项目版本**: v1.8.2