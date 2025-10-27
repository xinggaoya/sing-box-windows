# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

sing-box-windows is a modern cross-platform proxy client for Windows and Linux built with Tauri 2.0 + Vue 3, providing complete proxy management, routing rules, subscription management, and system service functionality.

### Tech Stack

- **Frontend**: Vue 3 + TypeScript + Vite + Pinia + Naive UI
- **Backend**: Rust + Tauri 2.0 + tokio
- **Architecture**: MVVM + modular design
- **Persistence**: Tauri Store (replaces localStorage) + SQLite database
- **Build**: Vite (frontend) + cargo-cp-artifact (Rust backend)

## Common Commands

### Development
```bash
# Install dependencies
pnpm install
cd src-tauri && cargo fetch

# Start development server
pnpm tauri dev
```

### Build and Packaging
```bash
# Build production version
pnpm tauri build

# Build MSI installer (Windows)
pnpm tauri build -- --target x86_64-pc-windows-msi

# Build DEB package (Linux)
pnpm tauri build -- --target x86_64-unknown-linux-gnu

# Build AppImage (Linux)
pnpm tauri build -- --target x86_64-unknown-linux-gnu

# Build with target check skip
pnpm tauri build -- --target x86_64-pc-windows-msi --no-target-check

# Build all targets
pnpm tauri build:all
```

### Code Quality
```bash
# ESLint check (includes OXLint)
pnpm lint

# Code formatting
pnpm format

# TypeScript type checking
pnpm type-check

# Rust code formatting (requires manual installation of rustfmt)
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

## Core Architecture

### Frontend Architecture

```
src/
├── stores/          # Pinia state management (custom StoreManager)
│   ├── app/        # App-related stores (AppStore, ThemeStore, LocaleStore, etc.)
│   ├── kernel/     # Kernel-related stores (KernelStore, ProxyStore, TrafficStore, etc.)
│   ├── subscription/ # Subscription store
│   └── tray/       # System tray store
├── components/      # Vue components
│   ├── layout/     # Layout components (MainLayout, TrafficChart)
│   ├── home/       # Homepage components (ProxyModeCard, TrafficStatsCard, etc.)
│   └── utils/      # Utility components (LazyComponent, VirtualList, UpdateModal)
├── views/           # Page views
├── services/        # API service layer (websocket-service, tauri command wrappers)
├── utils/           # Utility functions (memory management, performance optimization)
├── locales/         # Internationalization files
└── types/           # TypeScript type definitions
```

### Backend Architecture

```
src-tauri/src/
├── app/             # Application service layer
│   ├── core/       # Core services (kernel_service, proxy_service)
│   ├── network/    # Network services (subscription_service)
│   ├── system/     # System services (system_service, update_service, config_service)
│   ├── storage/    # Storage services (enhanced_storage_service with SQLite)
│   └── constants/  # Constants definitions
├── entity/          # Data entity models
├── process/         # Process management
├── utils/           # Utility functions
├── main.rs          # Program entry point
└── lib.rs           # Library entry point and command registration
```

## Key Features

### 1. Store Management System
- **Custom StoreManager**: Supports lazy loading and memory optimization
- **Debounced Persistence**: Auto-debounce save state to Tauri Store and SQLite
- **Memory Leak Detection**: Built-in memory leak detection and auto-cleanup mechanism
- **Route-based Preloading**: Stores are preloaded based on route requirements

### 2. Frontend-Backend Communication
- **Tauri Commands**: All frontend calls go through Tauri commands
- **Unified Error Handling**: Backend returns Result<T, String> format consistently
- **Type Safety**: Uses typescript-bindings to ensure type safety
- **WebSocket Communication**: Real-time updates via WebSocket for kernel status and traffic

### 3. Modular Design
- **Components grouped by functionality**: Each functional module has independent component directories
- **Service layer abstraction**: API calls wrapped in services/ directory
- **Centralized type definitions**: Unified TypeScript type definitions
- **Enhanced Storage**: Hybrid storage using Tauri Store for settings and SQLite for structured data

### 4. Performance Optimization
- **Virtual Scrolling**: Custom VirtualList.vue component optimizes large lists
- **Lazy Loading**: LazyComponent.vue implements on-demand component loading
- **Memory Management**: Built-in memory leak detection and WebSocket connection cleanup
- **Auto Imports**: Uses unplugin-auto-import and unplugin-vue-components

## Development Workflow

### Adding New Features
1. Define TypeScript types in `types/`
2. Create state management in `stores/` (if needed)
3. Create API service layer in `services/`
4. Create UI components in `components/`
5. Create page views in `views/`
6. Implement backend commands in `src-tauri/src/app/`
7. Update routing configuration (if needed)
8. Register new Tauri commands in `lib.rs`

### Component Naming Conventions
- **Page components**: Use PascalCase, e.g., `ProxyPage.vue`
- **Functional components**: Use PascalCase, e.g., `ProxyConfig.vue`
- **Utility components**: Use PascalCase, e.g., `LoadingSpinner.vue`

### State Management Conventions
- **Store naming**: Feature name + Store, e.g., `proxyStore`
- **Action naming**: Verb + noun, e.g., `updateConfig`
- **State naming**: Use camelCase, avoid abbreviations

## Key Files and Their Purposes

### Configuration Files
- `src-tauri/tauri.conf.json`: Tauri application configuration
- `src-tauri/Cargo.toml`: Rust dependency configuration
- `package.json`: Node.js dependency configuration
- `vite.config.ts`: Vite build configuration with auto-imports

### Core Files
- `src/stores/index.ts`: Store management system entry point
- `src/stores/StoreManager.ts`: Store lifecycle manager with lazy loading
- `src/services/websocket-service.ts`: WebSocket communication service
- `src/utils/memory-leak-fix.ts`: Memory management utilities
- `src-tauri/src/lib.rs`: Tauri entry file and command registration
- `src-tauri/src/app/`: Backend service layer implementation
- `src-tauri/src/app/storage/enhanced_storage_service.rs`: SQLite-based storage service

## Storage System

The application uses a hybrid storage approach:
- **Tauri Store**: For simple settings and preferences
- **SQLite Database**: For structured data (subscriptions, logs, etc.)
- **Storage Locations**:
  - Windows: `%APPDATA%\sing-box-windows\`
  - Linux: `~/.local/share/sing-box-windows/`

## Development Guidelines

### Memory Management
- Pay special attention to memory leaks for long-running applications
- Use the StoreManager's cleanup methods for proper resource management
- Monitor WebSocket connections and ensure proper cleanup

### Error Handling
- All async operations require proper error handling
- Backend commands should return Result<T, String> for consistent error handling
- Use TypeScript's strict type checking to avoid runtime errors

### Performance
- Use virtual scrolling or pagination for large data operations
- Implement lazy loading for non-critical components
- Leverage the StoreManager's route-based preloading

### Cross-Platform Compatibility
- Maintain compatibility with both Windows and Linux platforms
- Use platform-specific dependencies only when necessary
- Test builds on both target platforms before releases

## Debugging

### Frontend Debugging
- Vue DevTools automatically integrated in development environment
- Use console.log or debugger for breakpoint debugging
- Network requests viewed via browser developer tools

### Backend Debugging
- Use `println!` or `log::info!` for debug output
- Check console output for Rust logs
- Complex logic can use VS Code debugger (requires launch.json configuration)
- Log levels controlled via RUST_LOG environment variable

## Platform-Specific Notes

### Windows
- Requires Visual Studio Build Tools for compilation
- MSI and NSIS installers supported
- System proxy integration via Windows API

### Linux
- Requires libwebkit2gtk-4.1-0, libssl3, libgtk-3-0 dependencies
- DEB and AppImage packages supported
- System proxy integration via environment variables

## Testing and Quality Assurance

### Code Quality Tools
- **ESLint + OXLint**: JavaScript/TypeScript linting
- **Prettier**: Code formatting
- **rustfmt**: Rust code formatting
- **TypeScript**: Static type checking

### Before Committing
1. Run `pnpm lint` to fix code style issues
2. Run `pnpm type-check` to verify TypeScript types
3. Test functionality on target platforms
4. Verify memory usage for long-running operations