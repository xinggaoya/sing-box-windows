# Repository Guidelines

## Project Structure & Module Organization
- `src/` contains the Vue 3 + TypeScript frontend.
- Key frontend folders: `views/` (route pages like `HomeView.vue`), `components/`, `stores/`, `services/`, `composables/`, `router/`, `locales/`, and `utils/`.
- `src-tauri/` contains the Rust backend and desktop packaging logic.
- Core backend modules live in `src-tauri/src/app/` (for example `core/`, `network/`, `system/`), with Tauri command registration in `src-tauri/src/lib.rs`.
- `scripts/` holds Node helpers (such as `fetch-kernel.mjs` and `tauri-wrapper.mjs`).
- `docs/` stores development docs and changelog; build artifacts in `dist/` and `src-tauri/target/` should not be committed.

## Build, Test, and Development Commands
- `pnpm install`: install frontend/tooling dependencies.
- `pnpm tauri dev`: run the desktop app in local development mode.
- `pnpm build`: run TypeScript checks and build frontend assets.
- `pnpm tauri build`: produce platform bundles/installers.
- `pnpm lint`: run ESLint + oxlint (with autofix enabled by scripts).
- `pnpm format`: format frontend code under `src/` with Prettier.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust unit tests.

## Coding Style & Naming Conventions
- Follow `.editorconfig`: UTF-8, 2-space indentation for JS/TS/Vue, final newline, trimmed trailing whitespace.
- Prettier rules: `singleQuote: true`, `semi: false`, `printWidth: 100`.
- Vue component files use PascalCase (for example `UpdateModal.vue`); routed views use `*View.vue`.
- Composables use `useXxx.ts`; Rust module/file names use `snake_case`.
- Keep comments concise and focused on non-obvious behavior.

## Testing Guidelines
- Prefer Rust unit tests near implementation (`#[cfg(test)] mod tests`).
- Existing backend tests cover parser/state/process utilities; extend these modules when adding logic.
- Frontend test automation is not part of the current CI workflow, so PRs must at least pass `pnpm type-check`, `pnpm lint`, and manual smoke checks via `pnpm tauri dev`.

## Commit & Pull Request Guidelines
- Use Conventional Commits seen in history: `feat(scope): ...`, `fix(scope): ...`, `refactor(scope): ...`, `docs: ...`, `chore: ...`.
- Keep each commit focused and explain platform-specific impact when relevant.
- PRs should include: change summary, linked issue (`Closes #123`), verification commands run, and screenshots/recordings for UI changes.

## Security & Configuration Tips
- Do not commit secrets, logs, or local override files (see `.gitignore`).
- `src-tauri/resources/` and `src-tauri/.generated/` are generated/ignored; use `pnpm kernel:fetch` to prepare local kernel resources when needed.
