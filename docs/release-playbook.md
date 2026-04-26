# Sing-Box Windows 发布部署手册

这份文档只覆盖“如何把当前仓库发布出去”这件事，目标是下次直接照单执行，不再重复排查版本文件、tag 触发条件和发布入口。

## 发布模型

当前仓库采用“两阶段发布”：

1. 推送 `vX.Y.Z` tag
2. GitHub Actions 自动触发 `.github/workflows/release.yml`
3. CI 构建 Windows / Linux / macOS 产物
4. 自动创建对应 tag 的 GitHub Pre-release
5. 验证通过后，手动触发 `.github/workflows/promote-release.yml`
6. 将同一 tag 转为正式版 Release

关键点：

- 触发条件是 `push.tags: v*`
- Release Notes 不是 GitHub 自动生成，而是从 `docs/CHANGELOG.md` 读取
- 预发布和正式发布共用同一个 tag

## 发布前提

至少满足以下条件再开始发布：

- 本机已具备 Git 推送凭据
- 能推送到 `origin`
- 依赖已安装完成：`pnpm install`
- 当前分支代码已经合并到准备发布的状态

推荐的 Git 凭据方案二选一：

- HTTPS + Git Credential Manager
- SSH key + 可访问 GitHub

如果执行 `git push` 时出现凭据错误，例如：

```text
schannel: AcquireCredentialsHandle failed: SEC_E_NO_CREDENTIALS
```

先修复本机 GitHub 登录，再继续发布，不要跳过推送步骤。

## 版本号来源

发布前必须同步以下文件中的应用版本：

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.lock`

其中 `src-tauri/Cargo.lock` 只需要同步根包 `sing-box-windows` 对应的版本。

## Changelog 规则

Release Notes 来自 `docs/CHANGELOG.md`，格式必须满足脚本要求：

```md
## [v2.2.7] - 2026-03-17
```

注意：

- tag 名必须和 changelog 标题版本一致
- 标题建议始终带 `v`
- 新版本发布前，把 `## [Unreleased]` 下准备发布的内容切成对应版本节

本地可先验证提取结果：

```bash
node .github/scripts/extract-release-notes.js v2.2.7
```

## 小版本发布步骤

以下示例以从 `2.2.6` 发布到 `2.2.7` 为例。

### 1. 确认工作区

```bash
git status --short --branch
git pull --ff-only
```

如果工作区有无关改动，先整理干净，再继续。

### 2. 修改版本号

同步更新：

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.lock`

### 3. 更新 changelog

把：

```md
## [Unreleased]
```

下的已完成内容切到：

```md
## [v2.2.7] - 2026-03-17
```

### 4. 执行发布前校验

完整校验：

```bash
pnpm lint
pnpm type-check
pnpm test:kernel-targets
cd src-tauri && cargo clippy && cargo test
```

最少要补一条 release notes 提取校验：

```bash
node .github/scripts/extract-release-notes.js v2.2.7
```

### 5. 提交发布 commit

```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock docs/CHANGELOG.md
git commit -m "chore: release v2.2.7"
```

### 6. 创建 tag

```bash
git tag v2.2.7
```

如需附注 tag，可改为：

```bash
git tag -a v2.2.7 -m "v2.2.7"
```

### 7. 推送 commit 和 tag

```bash
git push origin master v2.2.7
```

如果默认分支未来改名，不要硬编码 `master`，改为当前默认分支名。

### 8. 验证预发布

推送成功后检查：

- GitHub Actions 中 `Release (Pre-release)` workflow 已触发
- Windows / Linux / macOS 三个平台构建全部成功
- GitHub Releases 中出现 `[Pre-release] v2.2.7`
- Release Notes 内容与 `docs/CHANGELOG.md` 对应版本节一致
- 附件包含预期产物

当前 workflow 期望的主要产物包括：

- Windows: `.msi`、`nsis .exe`、portable `.zip`
- Linux: `.deb`、`.rpm`、`.AppImage`
- macOS: `.dmg`、`.app.tar.gz`

## 正式发布步骤

预发布验证通过后，手动触发 `.github/workflows/promote-release.yml`。

输入：

- `tag`: `v2.2.7`
- `make_latest`: `true`

触发后会把同一 tag 的预发布转成正式版。

## 回滚与修正

如果 tag 刚创建但还未推送，可以直接本地删除重做：

```bash
git tag -d v2.2.7
```

如果 tag 已推送但版本内容有误，处理要谨慎：

- 优先新发一个修正版本，例如 `v2.2.8`
- 不要默认删除并重写已公开的发布 tag

如果确认当前 tag 仍处于预发布验证阶段，且必须用同一个版本号重发（例如修复刚发布预览包中的阻断问题），按以下顺序撤回并重新触发：

```bash
# 1. 先确保修复 commit 已经推到默认分支
git push origin master

# 2. 删除 GitHub 预发布和远端 tag
gh release delete v2.2.7 --yes --cleanup-tag

# 3. 将本地 tag 移到新的修复 commit
git tag -f v2.2.7 HEAD

# 4. 重新推送 tag，触发 release.yml 再构建预发布
git push origin v2.2.7
```

注意：

- 只在预发布未转正式版、用户还未广泛下载时使用这条路径
- 已正式发布或已经公开分发的版本，优先递增补丁版本重新发布
- `gh release delete --cleanup-tag` 会删除远端 tag；执行前确认 tag 名无误

## 故障排查

### `git push` 凭据失败

表现：

```text
schannel: AcquireCredentialsHandle failed: SEC_E_NO_CREDENTIALS
```

处理：

- 检查 Git Credential Manager 是否可用
- 重新登录 GitHub
- 或改用已配置好的 SSH key

### `release.yml` 未触发

检查：

- 推送的是不是 `v*` tag
- tag 是否真的已推到远端
- `.github/workflows/release.yml` 是否仍然监听 `push.tags`

### Release Notes 为空或报错

检查：

- `docs/CHANGELOG.md` 是否存在该版本标题
- 标题格式是否为 `## [vX.Y.Z] - YYYY-MM-DD`
- tag 与 changelog 标题版本是否完全一致

### Linux 产物缺失

检查：

- workflow 是否仍安装 RPM / AppImage 所需系统依赖
- 是否仍通过 `scripts/tauri-wrapper.mjs` 进入构建
- `tauri.conf.json` 的 `bundle.targets` 是否包含 `deb`、`rpm`、`appimage`

## 快速命令模板

把版本变量改掉后可直接执行：

```bash
# 例：发布 2.2.7
git status --short --branch
node .github/scripts/extract-release-notes.js v2.2.7
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json src-tauri/Cargo.lock docs/CHANGELOG.md
git commit -m "chore: release v2.2.7"
git tag v2.2.7
git push origin master v2.2.7
```

## 本次状态记录

2026-03-17 本地已完成：

- 版本同步到 `2.2.7`
- changelog 切出 `v2.2.7`
- 创建 commit `6629e14`
- 创建 tag `v2.2.7`

当时未完成的唯一阻塞项是远端凭据不可用，导致 `git push origin master v2.2.7` 失败。
