# CRUSH.md

This file provides guidance to AI agents (like Crush) working with code in this repository.

## Project Overview

sing-box-windows is a modern cross-platform proxy client built with Tauri 2.0 + Vue 3. It provides complete proxy management, routing rules, subscription management, and system service functionality for Windows and Linux.

### Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia + Naive UI
- **Backend**: Rust + Tauri 2.0 + tokio  
- **Architecture**: MVVM + modular design
- **Persistence**: Tauri Store + SQLite database (hybrid storage)
- **Build**: Vite (frontend) + cargo (Rust backend)
- **Package Manager**: pnpm

## Essential Commands

### Development
```bash
# Install dependencies
pnpm install
cd src-tauri && cargo fetch

# Start development server (with hot reload)
pnpm tauri dev
```

### Build and Packaging
```bash
# Build production version
pnpm tauri build

# Platform-specific builds
pnpm tauri build:windows          # Windows MSI
pnpm tauri build:linux:deb       # Linux DEB
pnpm tauri build:linux:appimage  # Linux AppImage

# Build all platforms
pnpm tauri build:all

# Skip target check (useful for CI)
pnpm tauri build -- --target x86_64-pc-windows-msi --no-target-check
```

### Code Quality
```bash
# Run all linting (ESLint + OXLint)
pnpm lint

# Individual linters
pnpm lint:eslint    # ESLint only
pnpm lint:oxlint    # OXLint only

# Code formatting
pnpm format

# TypeScript type checking
pnpm type-check

# Rust formatting (in src-tauri directory)
cd src-tauri && cargo fmt
```

### Testing
```bash
# Frontend unit tests
pnpm test

# Rust unit tests
cd src-tauri && cargo test

# Rust documentation tests
cd src-tauri && cargo test --doc
```

## Architecture Overview

### Frontend Structure
```
src/
├── stores/          # Pinia state management with custom StoreManager
│   ├── app/        # App-related stores (AppStore, ThemeStore, LocaleStore, etc.)
│   ├── kernel/     # Kernel-related stores (KernelStore, ProxyStore, TrafficStore, etc.)
│   ├── subscription/ # Subscription store
│   └── tray/       # System tray store
├── components/      # Vue components organized by function
│   ├── layout/     # Layout components (MainLayout, TrafficChart)
│   ├── home/       # Homepage components (ProxyModeCard, TrafficStatsCard, etc.)
│   └── utils/      # Utility components (LazyComponent, VirtualList, UpdateModal)
├── views/           # Page views (HomeView, ProxyView, SettingView, etc.)
├── services/        # API service layer (Tauri command wrappers)
├── utils/           # Utility functions (memory management, performance optimization)
├── locales/         # Internationalization files (zh-CN, en-US, ja-JP, ru-RU)
└── types/           # TypeScript type definitions
```

### Backend Structure
```
src-tauri/src/
├── app/             # Application service layer
│   ├── core/       # Core services (kernel_service, proxy_service)
│   ├── network/    # Network services (subscription_service)
│   ├── system/     # System services (system_service, update_service, config_service)
│   ├── storage/    # Storage services (enhanced_storage_service with SQLite)
│   └── constants/  # Constants definitions organized by module
├── entity/          # Data entity models
├── process/         # Process management
├── utils/           # Utility functions
├── main.rs          # Program entry point
└── lib.rs           # Library entry point and Tauri command registration
```

## Key Patterns and Conventions

### State Management
- **Custom StoreManager**: Implements lazy loading and memory optimization
- **Debounced Persistence**: Auto-debounce save state to prevent performance issues
- **Memory Leak Detection**: Built-in cleanup mechanisms for WebSocket connections and timers
- **Route-based Preloading**: Stores load based on route requirements

### Frontend-Backend Communication
- **Tauri Commands**: All frontend calls go through registered Tauri commands
- **Unified Error Handling**: Backend returns `Result<T, String>` format consistently
- **Type Safety**: Uses TypeScript bindings generated from Rust types
- **WebSocket Communication**: Real-time updates for kernel status and traffic data

### Component Patterns
- **Naming**: Page components use `XxxView.vue`, utility components use PascalCase
- **Auto-imports**: Components and Vue APIs auto-imported via unplugin
- **Lazy Loading**: Use `LazyComponent.vue` for heavy components
- **Virtual Scrolling**: Use `VirtualList.vue` for large datasets

### Storage System
- **Hybrid Approach**: Tauri Store for settings, SQLite for structured data
- **Locations**: 
  - Windows: `%APPDATA%\sing-box-windows\`
  - Linux: `~/.local/share/sing-box-windows/`

## Development Workflow

### Adding New Features
1. Define TypeScript types in `src/types/`
2. Create state management in `src/stores/` (if needed)
3. Create API service layer in `src/services/`
4. Create UI components in `src/components/`
5. Create page views in `src/views/`
6. Implement backend commands in `src-tauri/src/app/`
7. Update routing configuration (if needed)
8. Register new Tauri commands in `src-tauri/src/lib.rs`

### Adding Tauri Commands
1. Create function in appropriate service module with `#[tauri::command]` macro
2. Export function in module's `mod.rs`
3. Register command in `lib.rs` `invoke_handler`
4. Call from frontend using `invoke()` from `@tauri-apps/api/core`

### Internationalization
- **Supported Languages**: zh-CN (default), en-US, ja-JP, ru-RU
- **Files**: Located in `src/locales/`
- **Usage**: `import { useI18n } from 'vue-i18n'` and `t('module.key')`
- **Pattern**: Use `module.feature.action` naming structure

## Important Gotchas

### Memory Management
- Long-running application requires careful memory management
- Use StoreManager's cleanup methods for proper resource management
- Monitor WebSocket connections - ensure proper cleanup on component unmount
- Use `memory-leak-fix.ts` utilities for automatic cleanup

### Performance Considerations
- Large datasets: Use `VirtualList.vue` component
- Heavy components: Wrap in `LazyComponent.vue`
- High-frequency data: Exclude from persistence with `excludeHighFrequencyKeys`
- Store operations: Use debounced persistence to avoid IO thrashing

### Cross-Platform Compatibility
- Windows requires Visual Studio Build Tools for compilation
- Linux requires libwebkit2gtk-4.1-0, libssl3, libgtk-3-0 dependencies
- Test builds on both target platforms before releases
- Use platform-specific dependencies only when necessary

### Tauri-Specific Patterns
- All async operations require proper error handling
- Backend commands should return `Result<T, String>` for consistency
- Use `eventService` for real-time communication between backend and frontend
- System proxy integration differs between platforms (Windows API vs environment variables)

## Code Style and Standards

### Frontend
- **ESLint + OXLint**: Strict linting enforced
- **Prettier**: Semi-colons disabled, single quotes, 100 char line width
- **TypeScript**: Strict mode enabled
- **Vue 3**: Composition API preferred

### Backend
- **rustfmt**: Standard Rust formatting
- **tracing**: Structured logging with levels
- **tokio**: Async runtime for all operations
- **Result types**: Proper error handling throughout

## Configuration Files

### Key Files
- `package.json`: Frontend dependencies and scripts
- `src-tauri/Cargo.toml`: Rust dependencies with optimization profiles
- `src-tauri/tauri.conf.json`: Tauri application configuration
- `vite.config.ts`: Vite build with auto-imports and dev server on port 6221
- `tsconfig.json`: TypeScript strict mode configuration

### Build Configuration
- **Development**: opt-level = 1, debug enabled for faster compilation
- **Release**: opt-level = 3, LTO enabled, single codegen unit for max performance
- **Target Platforms**: Windows (MSI/NSIS), Linux (DEB/AppImage)

## Debugging and Troubleshooting

### Frontend Debugging
- Vue DevTools integrated in development mode
- Use `console.log` or `debugger` statements
- Network requests visible in browser dev tools
- Performance monitoring via `performance.memory` API

### Backend Debugging
- Use `tracing::info!`, `tracing::error!` for structured logging
- Log levels controlled via `RUST_LOG` environment variable
- VS Code with Rust Analyzer extension recommended
- Complex logic can use VS Code debugger with launch.json configuration

### Common Issues
- **Memory leaks**: Check WebSocket cleanup, timer disposal, Store subscriptions
- **Build failures**: Verify Rust toolchain, Visual Studio Build Tools (Windows), dependencies (Linux)
- **Permission issues**: TUN mode requires admin privileges on Windows
- **Connection issues**: Check WebSocket service status, kernel API readiness

## Testing and Quality Assurance

### Before Committing
1. Run `pnpm lint` to fix code style issues
2. Run `pnpm type-check` to verify TypeScript types  
3. Test functionality on target platforms
4. Verify memory usage for long-running operations
5. Check for WebSocket connection leaks

### CI/CD
- GitHub Actions handles automated builds for releases
- Multi-platform builds supported (Windows MSI, Linux DEB/AppImage)
- Tag-based releases trigger automated workflows
- Pre-release versions for testing before stable releases

## Platform-Specific Notes

### Windows
- Requires Visual Studio Build Tools 2019+ with C++ tools
- MSI and NSIS installer support
- System proxy integration via Windows Registry API
- Admin privileges required for TUN mode

### Linux  
- Requires libwebkit2gtk-4.1-0, libssl3, libgtk-3-0 dependencies
- DEB and AppImage package support
- System proxy integration via environment variables
- Package dependencies managed in `tauri.conf.json`

This project follows modern development practices with strong emphasis on performance, memory management, and cross-platform compatibility. The modular architecture makes it easy to extend and maintain.