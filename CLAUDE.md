# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

sing-box-windows is a modern cross-platform proxy client for Windows, Linux, and macOS built with Tauri 2.0 + Vue 3, providing complete proxy management, routing rules, subscription management, and system service functionality.

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
pnpm tauri build:windows

# Build DEB package (Linux)
pnpm tauri build:linux:deb

# Build AppImage (Linux)
pnpm tauri build:linux:appimage

# Build DMG (macOS)
pnpm tauri build:macos:dmg

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

## Core Architecture

### Architectural Patterns

1. **Event-Driven State Management**: The application uses Tauri's event system for real-time updates between frontend and backend, with stores listening to events rather than polling.

2. **Hybrid Storage Architecture**:
   - **Tauri Store**: Simple settings and preferences
   - **SQLite Database**: Structured data via `EnhancedStorageService`
   - **Auto-debounced persistence**: 300ms debounce with `waitForSaveCompletion()`

3. **Service Layer Pattern**: All backend interactions go through a service layer with clear separation of concerns.

### Frontend Architecture

```
src/
├── stores/          # Pinia state management
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
- **Standard Pinia**: Uses the official Pinia library for state management.
- **Debounced Persistence**: Auto-debounce save state to Tauri Store and SQLite.
- **Component-based Lifecycle**: Lifecycle events (like starting/stopping listeners) are managed by the Vue components that use the stores, not by the stores themselves.
- **Initialization Phases**: Stores implement `startInitialization()` → load data → `finishInitialization()` patterns.
- **Data Restore Pattern**: Use `waitForDataRestore()` to prevent race conditions during startup.

### 2. Frontend-Backend Communication
- **Tauri Commands**: All frontend calls go through Tauri commands
- **Unified Error Handling**: Backend returns Result<T, String> format consistently
- **Type Safety**: Uses typescript-bindings to ensure type safety
- **Event-Driven Updates**: Real-time updates via Tauri events for kernel status and traffic
- **Invoke Client**: `invoke-client.ts` provides automatic port injection and context-aware argument merging

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

### Store Development Patterns
- **Initialization**: Always implement proper initialization with data restore waiting
- **Cleanup**: Implement `cleanupStore()` methods for resource cleanup
- **Event Handling**: Use event-driven updates with anti-stale mechanisms
- **Persistence**: Use the persistence composable for automatic save/load

### Backend Development Patterns
- **Service Organization**: Group services by domain (core, network, system, storage)
- **Error Handling**: Return Result<T, String> for all commands
- **State Management**: Use application-wide state for persistent data
- **Async Operations**: Use tokio for all async operations

## Key Files and Their Purposes

### Configuration Files
- `src-tauri/tauri.conf.json`: Tauri application configuration
- `src-tauri/Cargo.toml`: Rust dependency configuration
- `package.json`: Node.js dependency configuration
- `vite.config.ts`: Vite build configuration with auto-imports and chunk optimization

### Core Files
- `src/stores/index.ts`: Store management system entry point
- `src/services/invoke-client.ts`: Unified Tauri command invoker with port injection
- `src-tauri/src/lib.rs`: Tauri entry file and command registration
- `src-tauri/src/app/storage/enhanced_storage_service.rs`: SQLite-based storage service
- `src-tauri/src/app/core/kernel_service/mod.rs`: Core kernel management

## Storage System

The application uses a hybrid storage approach:
- **Tauri Store**: For simple settings and preferences
- **SQLite Database**: For structured data (subscriptions, logs, etc.)
- **Storage Locations**:
  - Windows: `%APPDATA%\sing-box-windows\`
  - Linux: `~/.local/share/sing-box-windows/`
  - macOS: `~/Library/Application Support/sing-box-windows/`

### Enhanced Storage Service
- Single initialization with `OnceCell` pattern
- Type-safe operations through SQLx
- Automatic schema migrations
- Backend-frontend synchronization

## Development Guidelines

### Memory Management
- Pay special attention to memory leaks for long-running applications.
- Ensure resources like event listeners and timers are properly cleaned up in Vue component `onUnmounted` hooks.
- Monitor WebSocket connections and ensure proper cleanup in the services that manage them.
- Implement `cleanupStore()` methods for all stores with resources.

### Error Handling
- All async operations require proper error handling
- Backend commands should return Result<T, String> for consistent error handling
- Use TypeScript's strict type checking to avoid runtime errors

### Performance
- Use virtual scrolling or pagination for large data operations
- Implement lazy loading for non-critical components
- Use the composable patterns for reusable logic

### Cross-Platform Compatibility
- Maintain compatibility with Windows, Linux, and macOS platforms
- Use platform-specific dependencies only when necessary
- Test builds on all target platforms before releases

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

### Logging Configuration
Set log level via environment variable:
```bash
RUST_LOG=debug pnpm tauri dev
```

## Platform-Specific Notes

### Windows
- Requires Visual Studio Build Tools for compilation
- MSI and NSIS installers supported
- System proxy integration via Windows API

### Linux
- Requires libwebkit2gtk-4.1-0, libssl3, libgtk-3-0 dependencies
- DEB and AppImage packages supported
- System proxy integration via environment variables

### macOS
- Requires Xcode Command Line Tools for compilation
- DMG and App bundles supported
- System proxy integration via network settings

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