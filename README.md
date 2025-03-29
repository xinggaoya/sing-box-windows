# Sing-Box Windows

<div align="center">
    <img src="./src/assets/icon.png" alt="Sing-Box Windows Logo" width="128" height="128" />
    <h1>Sing-Box Windows</h1>
    <p>🚀 A beautiful Sing-Box Windows GUI client</p>
    <p>
        <img src="https://img.shields.io/github/license/xinggaoya/sing-box-windows" alt="license" />
        <img src="https://img.shields.io/github/stars/xinggaoya/sing-box-windows" alt="stars" />
        <img src="https://img.shields.io/github/downloads/xinggaoya/sing-box-windows/total" alt="downloads" />
        <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows" alt="release" />
    </p>
    <p>
        <a href="./README.md">English</a> | 
        <a href="./README_zh-CN.md">简体中文</a>
    </p>
</div>

## 🎯 Features

- 🖥️ Modern user interface, based on [Tauri 2.0](https://tauri.app/) and [Vue 3](https://vuejs.org/)
- 🌙 Support for light/dark theme, automatically follows the system
- 🔄 Support for multiple subscription link imports and automatic updates
- 🌐 Comprehensive proxy mode support
- 📊 Rich statistical functions
- 🔍 Complete logging system
- ⚡ Optimized performance
- 🔒 Security features

## 📸 Preview

<img src="./public/image.png" alt="sing-box-windows preview" width="800">

## 🚀 Quick Start

### Download and Install

1. Download the latest version from the [Releases](https://github.com/xinggaoya/sing-box-windows/releases) page
2. Run the installer (supports automatic updates)
3. The necessary configuration will be completed automatically on first launch

### Basic Usage

1. On first use, please download and install the Sing-Box kernel in the [Settings] section
2. Add or import your subscription link in the [Subscription] section
   - Supports direct link input
   - Supports sing-box json configuration, base64 automatic decoding
3. Select a node and connect in the [Home] section
   - Supports quick node switching
   - Supports node latency testing
   - Supports node group management

> Tip: When using TUN mode, the program will request administrator privileges and automatically configure system settings

### Advanced Features

- **Rule Settings**: Supports custom split rules
- **Quick Actions**: Supports system tray quick operations
- **Configuration Backup**: Supports configuration export and recovery
- **Automation**: Supports startup and automatic connection

## 🛠️ Development Guide

### Environment Requirements

- [Node.js](https://nodejs.org/) 18.0 or higher
- [Rust](https://www.rust-lang.org/) latest stable version
- [Visual Studio](https://visualstudio.microsoft.com/) 2019 or higher (includes C++ development tools)
- [Git](https://git-scm.com/) latest version
- [pnpm](https://pnpm.io/) package manager

### Local Development

```bash
# Clone the project
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# Install dependencies
pnpm install

# Start the development server
pnpm tauri dev

# Build the production version
pnpm tauri build
```

### More Development Documents

For detailed development documentation, please refer to [Development Documentation](./docs/development.md), which includes project structure, core functional modules, development specifications, and more.

### Project Structure

```
sing-box-windows/
├── src/                # Front-end source code
│   ├── assets/        # Static resources
│   ├── components/    # Common components
│   ├── router/        # Routing configuration
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
- ⚡ [Vue 3](https://vuejs.org/) - Responsive front-end framework
- 🎨 [Naive UI](https://www.naiveui.com/) - High-quality Vue 3 component library
- 📊 [Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API) - High-performance graphics rendering
- 🦀 [Rust](https://www.rust-lang.org/) - High-performance system programming language
- 🔧 [TypeScript](https://www.typescriptlang.org/) - Type-safe JavaScript

## 🤝 Contribution Guide

We welcome all forms of contributions, including but not limited to:

- 🐛 Issue reporting and suggestions
- 📝 Documentation improvement
- 🔧 Code fixes
- ✨ New feature development
- 🌍 Multilingual support

Contribution process:

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Submit a Pull Request

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 📮 Contact

- 📣 Issue Feedback: [GitHub Issues](https://github.com/xinggaoya/sing-box-windows/issues)
- 💬 Discussion: [GitHub Discussions](https://github.com/xinggaoya/sing-box-windows/discussions)

## ⭐ Acknowledgments

- [sing-box](https://github.com/SagerNet/sing-box) - Core proxy engine
- [Tauri](https://tauri.app/) - Application framework
- [Vue](https://vuejs.org/) - Front-end framework
- [Naive UI](https://www.naiveui.com/) - UI component library
- [Community contributors](https://github.com/xinggaoya/sing-box-windows/graphs/contributors)

---

If this project is helpful to you, please give a Star ⭐️
