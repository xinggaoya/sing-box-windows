# Gemini Code Assistant Context

This document provides context for the Gemini code assistant to understand the project structure, technologies, and conventions.

## Project Overview

This project is a graphical user interface (GUI) for the `sing-box` proxy utility, designed for Windows and Linux. It is built as a desktop application using the [Tauri](https://tauri.app/) framework.

### Architecture

The application follows a modern web-native architecture:

*   **Frontend:** The user interface is a single-page application (SPA) built with [Vue.js 3](https://vuejs.org/).
*   **Backend:** The core logic, system integration, and native functionalities are handled by a [Rust](https://www.rust-lang.org/) backend.
*   **Framework:** [Tauri](https://tauri.app/) is used to bundle the frontend and backend into a cohesive desktop application, using a native webview to render the UI.

### Key Technologies

*   **Frontend:**
    *   **Framework:** Vue.js 3
    *   **Build Tool:** Vite
    *   **Language:** TypeScript
    *   **UI Library:** Naive UI
    *   **State Management:** Pinia
    *   **Routing:** Vue Router
*   **Backend:**
    *   **Language:** Rust
    *   **Framework:** Tauri
*   **Package Manager:** pnpm

## Building and Running

The project uses `pnpm` as its package manager. The following scripts are defined in `package.json`:

*   **`pnpm tauri dev`**: Starts the application in development mode with hot-reloading for both the frontend and backend.
*   **`pnpm tauri build`**: Builds the application for production.
*   **`pnpm lint`**: Lints the codebase using ESLint and oxlint.
*   **`pnpm format`**: Formats the code using Prettier.
*   **`pnpm type-check`**: Performs TypeScript type checking.

## Development Conventions

### Code Style

*   **Formatting:** Code is formatted using [Prettier](https://prettier.io/). The configuration is defined in `.prettierrc.json`.
    *   Semicolons are not used (`"semi": false`).
    *   Single quotes are preferred (`"singleQuote": true`).
    *   The print width is set to 100 characters (`"printWidth": 100`).
*   **Linting:** [ESLint](https://eslint.org/) is used for linting the TypeScript and Vue.js code, along with [oxlint](https://oxc-project.github.io/docs/guide/linter/eslint-plugin.html). The configuration can be found in `eslint.config.js`.

### State Management

*   **Pinia:** The application uses [Pinia](https://pinia.vuejs.org/) for centralized state management. Store modules are organized by feature under the `src/stores` directory.

### Routing

*   **Vue Router:** Navigation is handled by [Vue Router](https://router.vuejs.org/). Route definitions are located in `src/router/index.ts`.

### Internationalization

*   The project supports multiple languages. Locale files are located in `src/locales`.

### Project Structure

```
sing-box-windows/
├── src/                    # Frontend (Vue 3)
│   ├── components/         # Reusable components
│   ├── stores/             # Pinia state management
│   ├── services/           # Business logic services
│   ├── utils/              # Utility functions
│   ├── locales/            # Internationalization
│   └── views/              # Page components
├── src-tauri/              # Backend (Rust)
│   ├── src/
│   │   ├── app/            # Application modules
│   │   └── main.rs         # Application entry point
│   └── Cargo.toml          # Rust dependencies
├── package.json            # Frontend dependencies and scripts
└── tauri.conf.json         # Tauri configuration
```
