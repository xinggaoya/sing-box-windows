# Linux 开发指南

## 系统要求

### 开发环境
- Linux (x64) with GTK 3
- Node.js 18+
- Rust 1.77.2+
- pnpm

### 系统依赖
```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libappindicator3-dev \
    librsvg2-dev \
    patchelf \
    libssl3 \
    pkg-config

# Fedora/RHEL/CentOS
sudo dnf install -y \
    gtk3-devel \
    webkit2gtk4.1-devel \
    libappindicator-gtk3 \
    librsvg2-devel \
    patchelf \
    openssl-devel \
    pkgconfig

# Arch Linux
sudo pacman -S --needed \
    gtk3 \
    webkit2gtk-4.1 \
    libappindicator-gtk3 \
    librsvg \
    patchelf \
    openssl \
    pkgconf
```

## 安装步骤

### 1. 克隆项目
```bash
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows
```

### 2. 安装依赖
```bash
pnpm install
cd src-tauri && cargo fetch
```

### 3. 开发模式运行
```bash
pnpm tauri dev
```

## 构建应用

### 构建所有平台
```bash
pnpm tauri build:all
```

### 仅构建 Linux
```bash
# 构建 DEB 包
pnpm tauri build:linux:deb

# 构建 AppImage
pnpm tauri build:linux:appimage

# 构建所有 Linux 格式
pnpm tauri build:linux
```

## 安装 sing-box 内核

### 1. 下载 sing-box
```bash
# 获取最新版本
wget https://github.com/SagerNet/sing-box/releases/latest/download/sing-box-linux-amd64.tar.gz

# 解压
tar -xzf sing-box-linux-amd64.tar.gz

# 安装到应用目录
mkdir -p ~/.local/share/sing-box-windows/sing-box
cp sing-box-linux-amd64/sing-box ~/.local/share/sing-box-windows/sing-box/
chmod +x ~/.local/share/sing-box-windows/sing-box/sing-box
```

### 2. 验证安装
```bash
~/.local/share/sing-box-windows/sing-box/sing-box version
```

## 系统代理设置

### GNOME (gsettings)
应用会自动尝试通过 gsettings 设置系统代理。如果需要手动设置：

```bash
# 启用代理
gsettings set org.gnome.system.proxy mode 'manual'
gsettings set org.gnome.system.proxy.http host '127.0.0.1'
gsettings set org.gnome.system.proxy.http port 2080
gsettings set org.gnome.system.proxy.https host '127.0.0.1'
gsettings set org.gnome.system.proxy.https port 2080

# 禁用代理
gsettings set org.gnome.system.proxy mode 'none'
```

### 环境变量
应用也会设置相应的环境变量，可以在终端中验证：

```bash
echo $http_proxy
echo $https_proxy
echo $all_proxy
```

## 故障排除

### 常见问题

1. **应用无法启动**
   ```bash
   # 检查依赖是否正确安装
   ldd target/release/sing-box-windows
   ```

2. **权限问题**
   ```bash
   # 确保可执行文件有正确权限
   chmod +x ~/.local/share/sing-box-windows/sing-box/sing-box
   ```

3. **系统代理不生效**
   ```bash
   # 检查 gsettings 是否可用
   which gsettings

   # 手动设置环境变量
   export http_proxy=http://127.0.0.1:2080
   export https_proxy=http://127.0.0.1:2080
   ```

4. **内核进程无法启动**
   ```bash
   # 检查 sing-box 是否存在
   ls -la ~/.local/share/sing-box-windows/sing-box/sing-box

   # 检查配置文件
   ls -la ~/.local/share/sing-box-windows/sing-box/config.json
   ```

### 调试模式
以调试模式启动应用获取详细日志：

```bash
RUST_LOG=debug pnpm tauri dev
```

### 日志位置
- 应用日志：`~/.local/share/sing-box-windows/logs/`
- 配置文件：`~/.local/share/sing-box-windows/sing-box/`

## 打包和分发

### AppImage (推荐)
```bash
# 构建后生成的文件位置
ls -la src-tauri/target/release/bundle/appimage/
```

### DEB 包
```bash
# 构建后生成的文件位置
ls -la src-tauri/target/release/bundle/deb/
```

### 安装 DEB 包
```bash
sudo dpkg -i sing-box-windows_*.deb
# 如果有依赖问题
sudo apt-get install -f
```

## 开发注意事项

1. **路径处理**：Linux 使用 `/` 作为路径分隔符，注意使用 `PathBuf` 处理跨平台路径
2. **权限管理**：Linux 使用不同的权限管理机制，应用会在需要时提示使用 `pkexec`
3. **系统代理**：通过 gsettings 和环境变量设置，与 Windows 注册表不同
4. **进程管理**：使用 `pgrep`/`pkill` 而不是 Windows 的 `tasklist`/`taskkill`

## 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 项目
2. 创建特性分支：`git checkout -b feature/amazing-feature`
3. 提交更改：`git commit -m 'Add some amazing feature'`
4. 推送分支：`git push origin feature/amazing-feature`
5. 提交 Pull Request

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件