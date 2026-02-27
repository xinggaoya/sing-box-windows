# Project Knowledge Base

## Project Overview
Cross-platform Sing-Box proxy client (Windows/Linux/macOS) built with Tauri 2.0 + Vue 3 + TypeScript + Rust backend.

## Structure
```
. (root)
├── src/                    # Vue 3 + TypeScript frontend
│   ├── views/              # Route pages (*View.vue)
│   ├── components/         # Reusable Vue components
│   ├── stores/             # Pinia state management (app/, kernel/)
│   ├── services/           # API layer (Tauri commands, WebSocket)
│   ├── composables/        # Vue composables
│   ├── router/             # Vue Router config
│   ├── locales/            # i18n translations
│   ├── types/              # TypeScript definitions
│   └── utils/              # Utility functions
├── src-tauri/              # Rust + Tauri backend
│   ├── src/
│   │   ├── app/           # Core services (core/, network/, system/, storage/)
│   │   ├── entity/         # Data models
│   │   ├── process/        # Process management
│   │   ├── utils/          # Backend utilities
│   │   ├── main.rs         # Entry point
│   │   └── lib.rs          # Tauri command registration
│   └── Cargo.toml
├── scripts/                 # Build helpers (fetch-kernel.mjs, tauri-wrapper.mjs)
└── docs/                   # Development docs & changelog
```

## Entry Points
| Layer | File | Purpose |
|-------|------|---------|
| Frontend | `src/main.ts` | Vue app bootstrap, Pinia, router, i18n init |
| Frontend | `src/router/index.ts` | Hash routing, /blank for tray mode |
| Frontend | `src/services/initialization-service.ts` | Async pre-mount setup |
| Backend | `src-tauri/src/main.rs` | Platform entry → calls lib::run() |
| Backend | `src-tauri/src/lib.rs` | Tauri Builder, plugins, command registration |

## Key Services
- **InitializationService**: Async pre-mount setup in frontend
- **KernelService**: Sing-Box kernel lifecycle management
- **EnhancedStorageService**: SQLite-backed data persistence
- **SubscriptionService**: Proxy subscription parsing & updates
- **WebSocket**: Real-time traffic/status updates

## Commands
```bash
# Development
pnpm tauri dev

# Build
pnpm tauri build           # All platforms
pnpm tauri build:windows   # Windows only

# Quality
pnpm type-check
pnpm lint
pnpm format
cd src-tauri && cargo clippy
```

## Conventions
- EditorConfig: 2-space indent, UTF-8, trailing whitespace trimmed
- Prettier: `singleQuote: true`, `semi: false`, `printWidth: 100`
- Vue: PascalCase components, `*View.vue` for routes, `useXxx.ts` for composables
- Rust: `snake_case` modules, `Result<T, String>` for commands
- Path alias: `@/*` → `src/*`

## Anti-Patterns
- DO NOT commit secrets, logs, or local override files
- DO NOT use `as any` or `@ts-ignore` for type suppression
- DO NOT skip `pnpm type-check` and `pnpm lint` before PR
- DO NOT forget `pnpm kernel:fetch` before first build

## CI/CD
- `.github/workflows/release.yml`: Multi-platform matrix (win/linux/mac)
- Linux glibc check: max version 2.38
- Kernel auto-fetched during build via `scripts/fetch-kernel.mjs`
- Custom release notes extracted from `docs/CHANGELOG.md`

## Storage
- All data stored in SQLite database (`app_data.db`)
- Windows: `%APPDATA%\sing-box-windows\`
- Linux: `~/.local/share/sing-box-windows/`
- macOS: `~/Library/Application Support/sing-box-windows/`
- Tables: app_config, theme_config, locale_config, window_config, update_config, subscriptions

## Notes
- Single-instance app (tauri_plugin_single_instance)
- Frameless window with custom title bar controls
- WebSocket for real-time kernel status & traffic stats
- Auto-debounced persistence (300ms) with `waitForSaveCompletion()`
