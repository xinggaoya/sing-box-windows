# Sing-Box Windows

<div align="center">
    <img src="./src/assets/icon.png" alt="Sing-Box Windows Logo" width="128" height="128" />
    <h1>Sing-Box Windows</h1>
    <p>🚀 An elegant and modern Sing-Box Windows GUI client</p>
    <p>
        <img src="https://img.shields.io/github/license/xinggaoya/sing-box-windows" alt="license" />
        <img src="https://img.shields.io/github/stars/xinggaoya/sing-box-windows" alt="stars" />
        <img src="https://img.shields.io/github/downloads/xinggaoya/sing-box-windows/total" alt="downloads" />
        <img src="https://img.shields.io/github/v/release/xinggaoya/sing-box-windows" alt="release" />
        <img src="https://img.shields.io/github/last-commit/xinggaoya/sing-box-windows" alt="last commit" />
    </p>
    <p>
        <a href="README.zh.md">中文</a> | 
        <a href="README.md">English</a>
    </p>
</div>

## 🎯 Features

### 🖥️ **Modern User Interface**

- Built with [Tauri 2.0](https://tauri.app/) and [Vue 3](https://vuejs.org/) for optimal performance
- Responsive design with [Naive UI](https://www.naiveui.com/) components
- Support for light/dark themes with automatic system detection
- Intuitive and user-friendly interface design

### 🌐 **Comprehensive Proxy Support**

- **System Proxy Mode**: Automatic system-wide proxy configuration
- **TUN Mode**: Traffic routing at the network level (requires administrator privileges)
- **Manual Mode**: Custom proxy configuration for advanced users
- Smart proxy switching with one-click toggle

### 📊 **Advanced Statistics & Monitoring**

- Real-time traffic monitoring with beautiful charts
- Connection statistics and active connections viewer
- Memory usage monitoring
- Historical data tracking
- Bandwidth usage analysis

### 🔄 **Subscription Management**

- Multiple subscription formats support:
  - Direct subscription URLs
  - Sing-box JSON configuration
  - Automatic Base64 decoding
- Automatic subscription updates
- Subscription grouping and management
- Import/export configuration

### 🔍 **Complete Logging System**

- Real-time log viewing with syntax highlighting
- Multiple log levels (Info, Warning, Error)
- Log filtering and search capabilities
- Export logs for debugging
- Automatic log rotation

### ⚡ **Performance Optimization**

- Memory management and leak prevention
- Efficient WebSocket connections
- Lazy loading and virtual scrolling
- Background processing for heavy operations
- Minimal resource footprint

### 🔒 **Security Features**

- Secure configuration storage
- Automatic privilege escalation for TUN mode
- Safe subscription parsing
- Process isolation

### 🌍 **Internationalization**

- Multi-language support
- Currently available in:
  - English
  - 中文 (Chinese)
  - 日本語 (Japanese)
  - Русский (Russian)
- Easy to add new languages

## 📸 Screenshots

<div align="center">
    <img src="./docs/image.png" alt="Sing-Box Windows Main Interface" width="800">
    <p><em>Main interface showing proxy status, statistics, and controls</em></p>
</div>

## 🚀 Quick Start

### System Requirements

- **Operating System**: Windows 10 1809 or later
- **Architecture**: x64 (64-bit)
- **Memory**: 4GB RAM recommended
- **Disk Space**: 100MB available space
- **Network**: Internet connection for downloading core and subscriptions

### Download and Install

1. **Download**: Get the latest version from the [Releases](https://github.com/xinggaoya/sing-box-windows/releases) page
2. **Install**: Run the installer (`.msi` or `.exe` setup file)
   - The installer supports automatic updates
   - Choose installation directory if needed
3. **First Launch**: The application will automatically complete necessary configuration

### Basic Usage

#### 🔧 **Initial Setup**

1. **Download Sing-Box Core**:

   - Navigate to [Settings] → [Core Management]
   - Click "Download Latest Core"
   - Wait for automatic installation

2. **Configure Subscriptions**:
   - Go to [Subscriptions] page
   - Click "Add Subscription"
   - Enter your subscription URL or import JSON configuration
   - The app supports automatic Base64 decoding

#### 🌐 **Connecting to Proxy**

1. **Select Node**:

   - Go to [Home] page
   - Browse available nodes from your subscriptions
   - Test node latency using the "Test" button
   - Select your preferred node

2. **Choose Proxy Mode**:

   - **System Proxy**: Automatic system-wide configuration
   - **TUN Mode**: Network-level routing (requires admin rights)
   - **Manual**: Custom configuration

3. **Connect**:
   - Click the "Connect" button
   - Monitor connection status in real-time
   - View traffic statistics and active connections

> **💡 Tip**: For TUN mode, the application will request administrator privileges and automatically configure system settings

### Advanced Features

#### 📋 **Rule Management**

- Navigate to [Rules] page to view and manage routing rules
- Support for custom rule sets
- Automatic rule updates from subscriptions
- Rule priority management

#### 🛠️ **System Integration**

- **System Tray**: Quick access from Windows system tray
- **Auto-start**: Configure automatic startup on Windows boot
- **Auto-connect**: Automatically connect to last used configuration
- **Notifications**: Desktop notifications for connection status

#### 📊 **Advanced Monitoring**

- **Connections**: View active connections and their details
- **Traffic Analysis**: Detailed bandwidth usage statistics
- **Performance Metrics**: Memory usage and system performance
- **Logs**: Comprehensive logging with filtering options

## 🛠️ Development Guide

### Environment Requirements

- **Node.js**: 18.0 or higher
- **Rust**: Latest stable version (1.70+)
- **Visual Studio**: 2019 or higher with C++ development tools
- **Git**: Latest version
- **pnpm**: Package manager

### Local Development

```bash
# Clone the repository
git clone https://github.com/xinggaoya/sing-box-windows.git
cd sing-box-windows

# Install dependencies
pnpm install

# Start development server (with hot reload)
pnpm tauri dev

# Build production version
pnpm tauri build

# Run tests
pnpm test

# Type checking
pnpm type-check

# Lint code
pnpm lint
```

### Development Documentation

For comprehensive development documentation, please check:

- [Development Guide](./docs/development.md) - Detailed development instructions
- [Internationalization Guide](./docs/i18n.md) - Adding new languages

### Project Architecture

```
sing-box-windows/
├── src/                    # Frontend source code
│   ├── components/        # Reusable Vue components
│   │   ├── home/         # Home page components
│   │   ├── layout/       # Layout components
│   │   └── ...           # Other component categories
│   ├── stores/           # Pinia state management
│   │   ├── app/          # Application-level stores
│   │   ├── kernel/       # Kernel-related stores
│   │   ├── subscription/ # Subscription management
│   │   └── tray/         # System tray integration
│   ├── services/         # Business logic services
│   ├── utils/            # Utility functions
│   ├── locales/          # Internationalization files
│   └── views/            # Page components
├── src-tauri/            # Rust backend code
│   ├── src/              # Rust source code
│   │   ├── app/          # Application modules
│   │   │   ├── core/     # Core functionality
│   │   │   ├── network/  # Network operations
│   │   │   └── system/   # System integration
│   │   ├── utils/        # Utility functions
│   │   └── main.rs       # Application entry point
│   └── Cargo.toml        # Rust dependencies
├── docs/                 # Documentation
└── public/               # Static assets
```

## 📦 Technology Stack

### Frontend

- 🎯 **[Tauri 2.0](https://tauri.app/)** - Modern cross-platform application framework
- ⚡ **[Vue 3](https://vuejs.org/)** - Progressive JavaScript framework
- 🎨 **[Naive UI](https://www.naiveui.com/)** - High-quality Vue 3 component library
- 📊 **[Canvas API](https://developer.mozilla.org/en-US/docs/Web/API/Canvas_API)** - High-performance graphics rendering
- 🔧 **[TypeScript](https://www.typescriptlang.org/)** - Type-safe JavaScript
- 🗃️ **[Pinia](https://pinia.vuejs.org/)** - State management for Vue
- 🛣️ **[Vue Router](https://router.vuejs.org/)** - Official router for Vue.js

### Backend

- 🦀 **[Rust](https://www.rust-lang.org/)** - High-performance systems programming language
- ⚡ **[tokio](https://tokio.rs/)** - Asynchronous runtime for Rust
- 🔗 **[serde](https://serde.rs/)** - Serialization framework
- 🌐 **[reqwest](https://github.com/seanmonstar/reqwest)** - HTTP client library

### Tools & Infrastructure

- 📦 **[pnpm](https://pnpm.io/)** - Fast, disk space efficient package manager
- 🔍 **[ESLint](https://eslint.org/)** - JavaScript/TypeScript linting
- 🎨 **[Prettier](https://prettier.io/)** - Code formatting
- 🏗️ **[Vite](https://vitejs.dev/)** - Next generation frontend tooling

## 🤝 Contributing

We welcome all forms of contributions! Here's how you can help:

### Types of Contributions

- 🐛 **Bug Reports**: Report issues and bugs
- 💡 **Feature Requests**: Suggest new features
- 📝 **Documentation**: Improve documentation
- 🔧 **Code Contributions**: Fix bugs or implement features
- 🌍 **Translations**: Add support for new languages
- 🎨 **UI/UX Improvements**: Enhance user interface

### Contribution Process

1. **Fork** the repository
2. **Clone** your fork locally
3. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
4. **Make** your changes with clear commit messages
5. **Test** your changes thoroughly
6. **Push** to your branch (`git push origin feature/amazing-feature`)
7. **Submit** a Pull Request with detailed description

### Development Guidelines

- Follow existing code style and conventions
- Write meaningful commit messages
- Add tests for new features
- Update documentation when necessary
- Ensure all tests pass before submitting

### Translation Contributions

To add a new language:

1. Create a new file in `src/locales/` (e.g., `fr-FR.ts`)
2. Copy the structure from `en-US.ts`
3. Translate all text strings
4. Add the language to the language selector
5. Test the translations thoroughly

## 📄 License

This project is licensed under the [MIT License](LICENSE) - see the LICENSE file for details.

## 📮 Support & Community

- 🐛 **Bug Reports**: [GitHub Issues](https://github.com/xinggaoya/sing-box-windows/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/xinggaoya/sing-box-windows/discussions)
- 📧 **Email**: [Contact the maintainer](mailto:your-email@example.com)
- 📘 **Documentation**: [Development Docs](./docs/development.md)

## 🙏 Acknowledgements

Special thanks to these amazing projects and contributors:

### Core Technologies

- **[sing-box](https://github.com/SagerNet/sing-box)** - The powerful proxy platform
- **[Tauri](https://tauri.app/)** - Secure, fast, and lightweight application framework
- **[Vue.js](https://vuejs.org/)** - The progressive JavaScript framework
- **[Naive UI](https://www.naiveui.com/)** - Beautiful and powerful UI component library

### Community

- **[All Contributors](https://github.com/xinggaoya/sing-box-windows/graphs/contributors)** - Thank you for your valuable contributions
- **Users and Testers** - Your feedback helps improve the application
- **Translators** - Making the app accessible worldwide

## 📈 Roadmap

### Upcoming Features

- [ ] Plugin system for extensibility
- [ ] Custom themes and UI customization
- [ ] Advanced routing rules editor
- [ ] Cloud configuration sync
- [ ] Mobile companion app
- [ ] Network diagnostics tools

### Version History

- **v1.7.9** - Current stable release
- **v1.7.x** - Performance optimizations and bug fixes
- **v1.6.x** - Enhanced UI and internationalization
- **v1.5.x** - Initial public release

---

<div align="center">
    <p>If this project helps you, please consider giving it a ⭐️!</p>
    <p>Made with ❤️ by the Sing-Box Windows team</p>
</div>
