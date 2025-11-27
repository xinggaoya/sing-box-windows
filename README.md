<div align="center">
  <img src="src/assets/icon.png" alt="Sing-Box GUI Client" width="120" height="120">

  <h1>Sing-Box Windows</h1>

  <p>
    <strong>A modern Sing-Box GUI client for Windows, Linux, and macOS built with Tauri 2.0 + Vue 3</strong>
  </p>

  <p>
    <a href="#features">✨ Features</a> •
    <a href="#installation">🚀 Installation</a> •
    <a href="#quick-start">🎯 Quick Start</a> •
    <a href="#screenshots">📸 Screenshots</a> •
    <a href="#development">🛠️ Development</a> •
    <a href="#contributing">💡 Contributing</a>
  </p>

  <p>
    <a href="README.zh.md">🇨🇳 中文</a> •
    <a href="README.md">🇺🇸 English</a>
  </p>

  <p>
    <a href="https://github.com/xinggaoya/sing-box-windows/releases">
      <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows?style=for-the-badge&logo=github" alt="GitHub release">
    </a>
    <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=for-the-badge&logo=apple" alt="Platform">
    <img src="https://img.shields.io/badge/version-1.8.2-informational?style=for-the-badge" alt="Version">
    <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License">
  </p>
</div>

---

## 🌟 About

Sing-Box GUI Client is a cutting-edge cross-platform application for Windows, Linux, and macOS that provides a modern, intuitive interface for managing Sing-Box proxy configurations. Built with the powerful combination of Tauri 2.0 and Vue 3, it offers exceptional performance, security, and user experience.

### 🎯 Key Highlights

- **🚀 Blazing Fast**: Built with Rust backend and optimized Vue 3 frontend
- **💎 Modern UI**: Beautiful, responsive interface with Naive UI components
- **🔒 Secure**: Rust-based backend ensures memory safety and performance
- **🌐 Real-time**: WebSocket communication for live updates
- **🎨 Customizable**: Light/Dark themes with extensive personalization options

---

## ✨ Features

### 🎯 Core Functionality
- **Proxy Management**: Easy configuration and switching between different proxy modes
- **Subscription Management**: Automatic subscription updates and management
- **Connection Monitoring**: Real-time connection statistics and monitoring
- **Log Management**: Comprehensive logging with filtering and search
- **Rule Management**: Visual routing rule configuration and management
- **System Integration**: System tray support with quick access controls

### 🎨 User Experience
- **Modern Interface**: Clean, intuitive design based on Naive UI components
- **Multi-language Support**: English, Chinese, Japanese, and Russian
- **Theme System**: Light, Dark, and Auto themes with custom colors
- **Responsive Design**: Optimized for various screen sizes and DPI settings

### 🔧 Advanced Features
- **Real-time Statistics**: Live bandwidth and connection monitoring via WebSocket
- **Auto-start**: Automatic system startup with delayed launch option
- **Update Management**: Automatic update checking and installation
- **Memory Optimization**: Efficient resource management through standard Vue.js and Pinia practices.

---

## 🚀 Installation

Download the installation package for your platform from [GitHub Releases](https://github.com/xinggaoya/sing-box-windows/releases):

- **Windows**: `.exe` installer
- **Linux**: `.deb` package or `.AppImage` portable version
- **macOS**: `.dmg` disk image or `.app` archive

For detailed installation instructions, please refer to the project documentation.

---

## 🎯 Quick Start

1. **Install**: Download and install sing-box-windows
2. **Add Subscription**: Add your subscription link in the subscription page
3. **Configure Proxy**: Select a node and set proxy mode in the proxy page
4. **Start Using**: Enable proxy and enjoy network access

For detailed usage tutorials, please refer to the project documentation.

---

## 📸 Screenshots

<div align="center">
  <img src="docs/image.png" alt="Sing-Box GUI Client Interface" width="800">

  <p><em>Main application interface showing proxy management and system status</em></p>
</div>

---

## 🛠️ Development

```bash
# Clone repository
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# Install dependencies
pnpm install

# Start development mode
pnpm tauri dev

# Build release version
pnpm tauri build
```

For detailed development documentation, please refer to the project instructions.

---

## 🔧 Storage

The application uses **local storage** solutions, with all configuration and subscription data stored locally:

- **Windows**: `%APPDATA%\sing-box-windows\`
- **Linux**: `~/.local/share/sing-box-windows/`
- **macOS**: `~/Library/Application Support/sing-box-windows/`

Data includes application settings, subscription information, theme configurations, etc., ensuring user privacy and data security.

---

## 🤝 Contributing

Welcome to submit Issues and Pull Requests!

1. **Report Issues**: [Submit Issue](https://github.com/xinggaoya/sing-box-windows/issues)
2. **Feature Suggestions**: [Join Discussion](https://github.com/xinggaoya/sing-box-windows/discussions)
3. **Code Contributions**: Fork and submit PR

---

## 🙏 Acknowledgments

- [Sing-Box](https://github.com/SagerNet/sing-box) - The powerful proxy core
- [Tauri](https://tauri.app/) - Amazing Rust-based app framework
- [Vue.js](https://vuejs.org/) - Progressive JavaScript framework
- [Naive UI](https://www.naiveui.com/) - Beautiful Vue 3 component library
- [Vite](https://vitejs.dev/) - Fast build tool and dev server

---

---

<div align="center">
  <p>
    <strong>Built with ❤️ by XingGao</strong>
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
    <strong>📝 Disclaimer:</strong>This project is for learning and educational purposes only. All data is stored locally and will not be uploaded to the cloud.<br>
    Please comply with local laws and regulations and use this software responsibly.
  </p>
</div>