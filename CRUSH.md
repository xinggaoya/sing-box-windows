# CRUSH.md

## Build & Development Commands

```bash
# Install dependencies
pnpm install && cd src-tauri && cargo fetch

# Development server
pnpm tauri dev

# Build production
pnpm tauri build

# MSI installer build
pnpm tauri build -- --target x86_64-pc-windows-msi

# Code quality
pnpm lint           # Run all linters (oxlint + eslint)
pnpm type-check     # TypeScript type checking
pnpm format         # Prettier formatting

# Testing
pnpm test           # Frontend tests
cd src-tauri && cargo test     # Rust tests
cd src-tauri && cargo test --doc  # Rust doc tests

# Single test (Rust)
cd src-tauri && cargo test test_name

# Performance testing (kernel management)
pnpm tauri dev -- --devtools && runKernelPerformanceTest()
```

## Code Style Guidelines

### TypeScript/Vue
- Use **PascalCase** for components: `ProxyPage.vue`, `TrafficChart.vue`
- Use **camelCase** for variables and functions: `updateConfig`, `proxyStore`
- **No semicolons**, single quotes, 100 char line width (Prettier config)
- Store naming: `FeatureNameStore` pattern
- Import order: Vue → Tauri → External → Internal
- Always use TypeScript types, avoid `any`

### Rust
- Follow `rustfmt` formatting
- Use `Result<T, String>` for Tauri command returns
- Module structure: `app/`, `entity/`, `utils/`, `process/`
- Async functions with proper error handling

### Architecture Rules
- All frontend-backend communication via Tauri commands
- State management in `stores/` with custom StoreManager
- API calls wrapped in `services/` layer
- Component grouping by feature in `components/feature/`
- Memory management: use LazyComponent, VirtualList for performance

### Error Handling
- Frontend: try/catch with proper error types
- Backend: `Result<T, String>` with descriptive messages
- Always handle async operations with proper error boundaries