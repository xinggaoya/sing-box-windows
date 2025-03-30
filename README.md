# Sing-Box Windows

<div align="center">
    <img src="./src/assets/icon.png" alt="Sing-Box Windows Logo" width="128" height="128" />
    <h1>Sing-Box Windows</h1>
    <p>🚀 An elegant Sing-Box Windows GUI client</p>
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

## 🎯 Features

- 🖥️ Modern user interface based on [Tauri 2.0](https://tauri.app/) and [Vue 3](https://vuejs.org/)
- 🌙 Support for light/dark themes, automatically following system settings
- 🔄 Support for multiple subscription links import and automatic updates
- 🌐 Comprehensive proxy mode support
- 📊 Rich statistical features
- 🔍 Complete logging system
- ⚡ Optimized performance
- 🔒 Security features
- 🌍 Internationalization support

## 📸 Preview

<img src="./public/image.png" alt="sing-box-windows preview" width="800">

## 🚀 Quick Start

### Download and Install

1. Download the latest version from the [Releases](https://github.com/xinggaoya/sing-box-windows/releases) page
2. Run the installer (supports automatic updates)
3. The necessary configuration will be automatically completed on first launch

### Basic Usage

1. For first-time use, please download and install the Sing-Box core in the [Settings] page
2. Add or import your subscription links on the [Subscriptions] page
   - Support direct link input
   - Support sing-box json configuration with automatic base64 decoding
3. Select and connect to nodes on the [Home] page
   - Support quick node switching
   - Support node latency testing
   - Support node group management

> Tip: When using TUN mode, the program will request administrator privileges and automatically configure system settings

### Advanced Features

- **Rule Settings**: Support for custom routing rules
- **Quick Operations**: Support for system tray quick actions
- **Configuration Backup**: Support for configuration export and restore
- **Automation**: Support for auto-start on boot and auto-connect

## 🛠️ Development Guide

### Environment Requirements

- [Node.js](https://nodejs.org/) 18.0 or higher
- [Rust](https://www.rust-lang.org/) latest stable version
- [Visual Studio](https://visualstudio.microsoft.com/) 2019 or higher (with C++ development tools)
- [Git](https://git-scm.com/) latest version
- [pnpm](https://pnpm.io/) package manager

### Local Development

```bash
# Clone the project
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# Install dependencies
pnpm install

# Start development server
pnpm tauri dev

# Build production version
pnpm tauri build
```

### More Development Documentation

For detailed development documentation, please check the [Development Documentation](./docs/development.md), which includes project structure, core functional modules, development specifications, and more.

### Project Structure

```
sing-box-windows/
├── src/                # Frontend source code
│   ├── assets/        # Static resources
│   ├── components/    # Common components
│   ├── router/        # Route configuration
│   ├── stores/        # State management
│   ├── utils/         # Utility functions
│   └── views/         # Page components
├── src-tauri/         # Rust backend code
│   ├── src/           # Source code
│   └── Cargo.toml     # Rust dependency configuration
└── package.json       # Project configuration
```

## 📦 Technology Stack

- 🎯 [Tauri 2.0](https://tauri.app/) - Modern cross-platform application framework
- ⚡ [Vue 3](https://vuejs.org/) - Reactive frontend framework
- 🎨 [Naive UI](https://www.naiveui.com/) - High-quality Vue 3 component library
- 📊 [Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API) - High-performance graphics rendering
- 🦀 [Rust](https://www.rust-lang.org/) - High-performance systems programming language
- 🔧 [TypeScript](https://www.typescriptlang.org/) - Type-safe JavaScript

## 🤝 Contribution Guidelines

We welcome all forms of contributions, including but not limited to:

- 🐛 Issue reports and suggestions
- 📝 Documentation improvements
- 🔧 Code fixes
- ✨ New feature development
- 🌍 Multi-language support

Contribution process:

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Submit a Pull Request

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 📮 Contact

- 🐛 Issue Feedback: [GitHub Issues](https://github.com/xinggaoya/sing-box-windows/issues)
- 💬 Discussion: [GitHub Discussions](https://github.com/xinggaoya/sing-box-windows/discussions)

## ⭐ Acknowledgements

- [sing-box](https://github.com/SagerNet/sing-box) - Core proxy engine
- [Tauri](https://tauri.app/) - Application framework
- [Vue](https://vuejs.org/) - Frontend framework
- [Naive UI](https://www.naiveui.com/) - UI component library
- [Community contributors](https://github.com/xinggaoya/sing-box-windows/graphs/contributors)

---

If this project is helpful to you, please give it a Star ⭐️
