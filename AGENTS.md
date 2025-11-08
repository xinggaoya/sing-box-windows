# Repository Guidelines

## 项目结构与模块组织
前端 Vue 3 代码集中在 `src/`，其中 `views/` 存放页面，`components/` 承载可复用 UI，`stores/` 与 `services/` 管理状态与 API，静态资源在 `assets/`。`router/` 与 `composables/` 负责路由与组合式逻辑，公共类型聚合在 `types/` 便于复用。
国际化文案位于 `locales/`，公共工具和常量分别放在 `utils/` 与 `constants/`。`public/` 存放不会被打包处理的静态模板，`dist/` 是 Vite 构建输出，切勿手动修改。Tauri 原生壳层位于 `src-tauri/`，主要入口是 `src-tauri/src/main.rs`，平台特定配置集中在 `tauri.conf.json` 与 `capabilities/`，发行素材放在 `src-tauri/icons/`，缓存数据默认写入 `src-tauri/cache.db`。流程与设计记录放入 `docs/`，便于 Reviewer 查阅。

## 构建、测试与开发命令
- `pnpm dev`：运行 Vite + Tauri 调试服务器，默认热更新并挂载到 1420 端口。
- `pnpm build`：执行类型检查与 Vite 生产构建，产物输出到 `dist/`，供 Tauri 壳层引用。
- `pnpm preview`：本地预览构建后的前端，验证生产资源。
- `pnpm tauri dev` 或 `pnpm tauri`：启动桌面端调试窗口，并同时运行前端；可通过 `RUST_LOG=debug` 获取原生日志。
- `pnpm tauri build:windows` / `pnpm tauri build:linux:deb` / `pnpm tauri build:macos:dmg`：生成对应平台安装包，必要时使用 `pnpm tauri build:all` 进行全集成发布。
- `pnpm lint`、`pnpm lint:oxlint`、`pnpm lint:eslint`：分别触发 OXLint 与 ESLint 套件；`pnpm format` 用 Prettier 统一代码风格；`pnpm type-check` 独立运行 Vue TSC。

## 代码风格与命名约定
TypeScript 与 Vue 文件采用 2 空格缩进，优先使用 ES Modules 与 `script setup`，并遵守 `eslint.config.js` 与 `prettier` 规则。
组件命名使用 PascalCase（例如 `ServerStatusCard.vue`），Pinia store 以 `useXxxStore` 导出，路由文件保持 kebab-case。Rust 代码遵循 `rustfmt` 默认风格并保持模块私有性控制。
所有新建 API 或工具应配套类型定义放在 `types/`，并在运行 `pnpm type-check` 后提交自动生成的 `auto-imports.d.ts` 变动，以免 CI 失败。

## 测试与质量验证
当前仓库以类型检查与 lint 作为基础门槛，提交前至少运行 `pnpm type-check` 与 `pnpm lint`，确保无类型回退与静态检查问题。
如需添加单元测试，可在相邻目录创建 `*.spec.ts` 并使用 Vitest（与 Vite 生态兼容），同时在 PR 描述中说明覆盖范围与模拟数据。桌面功能请在 Tauri Dev 模式下自测主要平台差异，并在 `docs/` 内补充复现步骤与截图，帮助 QA 复核。

## Commit 与 Pull Request 规范
遵循 Conventional Commits（示例：`feat(app): 添加系统开机自启动功能`、`docs: 更新指南`），冒号前限定 scope，正文首句概述解决的问题或影响面。
PR 描述需包含：目的概述、关键变更点、验证步骤或截图，若解决 Issue 请使用 `Close #ID` 语法。对于跨平台功能，请附上 Windows 与 Linux 的最小验证说明，以及是否触及 `src-tauri/` 权限配置，确保 Reviewer 可复现。

## 安全与配置提示
敏感配置（如代理端口、订阅地址）不要写入仓库，可通过 `.env` 或 Tauri `environment` 注入，并在 `.gitignore` 中确认未被追踪。
确保在发布前检查 `src-tauri/tauri.conf.json` 中的权限声明，最小化 `allowlist`，并在 `capabilities/` 中同步更新；如需持久化用户数据，请优先存放于 Tauri `app_data_dir` 而非项目内的 `cache.db`，防止将调试数据提交到 Git。
