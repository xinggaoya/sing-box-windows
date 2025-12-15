# Sing-Box Windows 架构设计文档

## 目录

- [架构概览](#架构概览)
- [前端架构](#前端架构)
- [后端架构](#后端架构)
- [数据流架构](#数据流架构)
- [模块间通信](#模块间通信)
- [存储架构](#存储架构)
- [安全架构](#安全架构)
- [性能优化](#性能优化)
- [扩展性设计](#扩展性设计)

## 架构概览

Sing-Box Windows 采用现代化的分层架构设计，基于 Tauri 2.0 框架，实现了前后端分离、模块化组织、事件驱动的设计理念。

### 整体架构图

```
┌─────────────────────────────────────────────────────────────┐
│                      前端层 (Frontend)                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   Vue 3 UI      │  │  状态管理 Pinia │  │ 路由 Vue R   │ │
│  │   Components    │  │     Stores      │  │    outer     │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   IPC 通信层      │
                    │  (Tauri Events)   │
                    └─────────┬─────────┘
                              │
┌─────────────────────────────────────────────────────────────┐
│                      后端层 (Backend)                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   核心服务      │  │   网络服务      │  │  系统服务    │ │
│  │  Core Service   │  │ Network Service │  │System Service│ │
│  │                 │  │                 │  │              │ │
│  │ • 内核管理      │  │ • 订阅管理      │  │ • 权限管理   │ │
│  │ • 代理服务      │  │ • 配置解析      │  │ • 系统集成   │ │
│  │ • 进程管理      │  │ • 自动更新      │  │ • sudo 支持  │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
│                                                             │
│  ┌─────────────────┐  ┌─────────────────┐  ┌──────────────┐ │
│  │   存储服务      │  │   Sing-Box      │  │  WebSocket   │ │
│  │ Storage Service │  │   集成层        │  │    服务      │ │
│  │                 │  │                 │  │              │ │
│  │ • SQLite 数据库 │  │ • 配置生成      │  │ • 实时通信   │ │
│  │ • 配置同步      │  │ • 节点管理      │  │ • 事件推送   │ │
│  │ • 持久化存储    │  │ • 规则管理      │  │ • 状态同步   │ │
│  └─────────────────┘  └─────────────────┘  └──────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                    ┌─────────▼─────────┐
                    │   Sing-Box 核心   │
                    │   (外部进程)      │
                    └───────────────────┘
```

### 设计原则

1. **前后端分离**：通过 Tauri IPC 机制通信，保持清晰的边界
2. **模块化设计**：功能模块独立，低耦合高内聚
3. **事件驱动**：使用事件系统实现松耦合的模块通信
4. **数据一致性**：通过统一的数据模型和状态管理保证一致性
5. **可扩展性**：预留扩展接口，支持插件化架构

## 前端架构

### 技术栈

- **Vue 3**：响应式框架，使用 Composition API
- **TypeScript**：类型安全，提高代码质量
- **Pinia**：状态管理，支持持久化
- **Naive UI**：组件库，提供一致的 UI 体验
- **Vue Router**：路由管理
- **Vue I18n**：国际化支持

### 组件架构

```
App.vue (根组件)
├── MainLayout.vue (主布局)
│   ├── Sidebar.vue (侧边栏)
│   ├── Header.vue (顶部栏)
│   └── Content.vue (内容区)
│       ├── HomeView.vue (首页)
│       ├── ProxyView.vue (代理管理)
│       ├── SubView.vue (订阅管理)
│       ├── RulesView.vue (规则管理)
│       ├── ConnectionsView.vue (连接管理)
│       ├── LogView.vue (日志查看)
│       └── SettingView.vue (设置页面)
├── SystemTray.vue (系统托盘)
└── UpdateModal.vue (更新弹窗)
```

### 状态管理架构

Pinia Store 采用领域驱动设计，按功能模块划分：

```
stores/
├── app/ (应用级状态)
│   ├── AppStore.ts (应用核心状态)
│   ├── ThemeStore.ts (主题管理)
│   ├── LocaleStore.ts (语言设置)
│   ├── WindowStore.ts (窗口管理)
│   └── UpdateStore.ts (更新管理)
├── kernel/ (内核相关状态)
│   ├── KernelStore.ts (内核运行状态)
│   ├── ProxyStore.ts (代理设置)
│   ├── TrafficStore.ts (流量统计)
│   ├── ConnectionStore.ts (连接管理)
│   └── LogStore.ts (日志管理)
├── subscription/ (订阅管理)
│   └── SubStore.ts
└── tray/ (系统托盘)
    └── TrayStore.ts
```

### 服务层架构

前端服务层负责封装与后端的通信：

```
services/
├── invoke-client.ts (统一的 Tauri 命令调用封装)
├── websocket-service.ts (WebSocket 实时通信)
└── notification-service.ts (系统通知)
```

## 后端架构

### 模块组织

后端采用分层架构，从内到外分为：

1. **领域层（Domain）**：核心业务逻辑
2. **服务层（Service）**：业务服务实现
3. **应用层（Application）**：外部接口定义
4. **基础设施层（Infrastructure）**：技术实现

### 服务模块详解

#### 1. 核心服务（Core Services）

```rust
app/core/
├── kernel_service.rs (内核管理服务)
│   ├── 下载和安装内核
│   ├── 启动/停止内核
│   ├── 健康检查
│   └── 状态监控
├── proxy_service.rs (代理服务)
│   ├── 系统代理设置
│   ├── TUN 模式管理
│   ├── 节点切换
│   └── 延迟测试
└── task_manager.rs (任务管理器)
    ├── 异步任务调度
    └── 后台任务管理
```

#### 2. 网络服务（Network Services）

```rust
app/network/
├── subscription_service/ (订阅服务)
│   ├── parser.rs (订阅解析器)
│   │   ├── 多格式支持（JSON/YAML/Base64）
│   │   ├── 协议识别（vmess/vless/trojan）
│   │   └── 节点提取和标准化
│   ├── auto_update.rs (自动更新)
│   │   ├── 定时任务
│   │   ├── 增量更新
│   │   └── 错误重试
│   └── config_generator.rs (配置生成)
│       ├── Sing-Box 配置生成
│       ├── 路由规则处理
│       └── DNS 配置
└── mod.rs
```

#### 3. 系统服务（System Services）

```rust
app/system/
├── system_service.rs (系统集成)
│   ├── 权限管理
│   ├── 系统代理集成
│   ├── 网络状态检测
│   └── 开发者工具
├── sudo_service.rs (sudo 管理)
│   ├── 密码存储（keyring）
│   ├── 密码验证
│   └── 跨平台支持
├── background_tasks.rs (后台任务)
│   ├── 更新检查
│   ├── 健康监控
│   └── 定时任务调度
├── update_service.rs (更新服务)
│   ├── 版本检查
│   ├── 下载管理
│   └── 安装流程
└── config_service.rs (配置服务)
    ├── 端口管理
    ├── 配置同步
    └── 设置迁移
```

#### 4. 存储服务（Storage Services）

```rust
app/storage/
├── enhanced_storage_service.rs (增强存储服务)
│   ├── SQLite 数据库操作
│   ├── 配置持久化
│   ├── 数据同步
│   └── 事务管理
├── database.rs (数据库服务)
│   ├── 连接池管理
│   ├── 迁移处理
│   └── 查询优化
└── state_model.rs (数据模型)
    ├── 实体定义
    ├── 序列化支持
    └── 验证逻辑
```

#### 5. Sing-Box 集成（Sing-Box Integration）

```rust
app/singbox/
├── config_generator.rs (配置生成器)
│   ├── 高级配置支持
│   ├── 模板系统
│   ├── 动态生成
│   └── 验证机制
└── mod.rs
```

## 数据流架构

### 前后端数据流

```
用户操作 → Vue组件 → Pinia Store → Tauri IPC → Rust Service → Sing-Box
   ↑                                                             ↓
   └─────── WebSocket事件 ←───── Event Emitter ←────── 内核状态 ──┘
```

### 事件驱动架构

1. **用户事件**：UI 交互触发
2. **应用事件**：Store 状态变更
3. **系统事件**：内核状态、网络变化
4. **定时事件**：自动更新、健康检查

### 数据同步机制

1. **配置同步**：应用配置 → SQLite → Sing-Box 配置文件
2. **状态同步**：内核状态 → WebSocket → 前端 Store
3. **持久化同步**：Store 状态 → Tauri Store → 磁盘

## 模块间通信

### IPC 通信

使用 Tauri 的 `invoke` API 进行同步调用：

```typescript
// 前端调用
const result = await invoke('kernel_start_enhanced', {
  api_port: 9090,
  proxy_port: 7890
})
```

```rust
// 后端定义
#[tauri::command]
pub async fn kernel_start_enhanced(
  api_port: Option<u16>,
  proxy_port: Option<u16>
) -> Result<Value, String>
```

### 事件通信

使用 Tauri Events 进行异步通信：

```rust
// 后端发送事件
app_handle.emit("kernel-status-changed", status)?;

// 前端监听
listen("kernel-status-changed", (event) => {
  // 处理状态变更
})
```

### WebSocket 中继

为前端提供实时数据访问：

```rust
// WebSocket 中继服务
// 将 Sing-Box API 通过 WebSocket 转发到前端
```

## 存储架构

### 存储层次

1. **内存层**：运行时状态、缓存
2. **数据库层**：SQLite 存储持久化数据
3. **文件系统层**：Sing-Box 配置文件、日志
4. **系统存储层**：密钥环、注册表

### 数据库设计

```sql
-- 应用配置表
CREATE TABLE app_config (
  id INTEGER PRIMARY KEY,
  key TEXT UNIQUE NOT NULL,
  value TEXT NOT NULL,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 订阅数据表
CREATE TABLE subscriptions (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  url TEXT NOT NULL,
  node_count INTEGER DEFAULT 0,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 主题配置表
CREATE TABLE theme_config (
  id INTEGER PRIMARY KEY,
  theme TEXT NOT NULL,
  primary_color TEXT,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 配置管理

1. **应用配置**：存储在 SQLite，通过 UI 管理
2. **运行配置**：动态生成的 Sing-Box JSON 配置
3. **用户设置**：主题、语言、窗口状态等

## 安全架构

### 数据安全

1. **密码存储**：使用系统密钥环（keyring）
2. **敏感数据**：内存加密，及时清零
3. **配置保护**：限制文件权限

### 网络安全

1. **HTTPS 验证**：证书验证，中间人攻击防护
2. **订阅安全**：URL 验证，防止恶意链接
3. **代理安全**：规则隔离，防止泄露

### 代码安全

1. **输入验证**：所有用户输入严格验证
2. **路径安全**：防止路径遍历攻击
3. **权限最小化**：仅请求必要权限

## 性能优化

### 前端优化

1. **组件懒加载**：`LazyComponent.vue`
2. **虚拟滚动**：`VirtualList.vue`
3. **防抖节流**：Store 更新防抖
4. **内存管理**：自动清理，防止泄漏

### 后端优化

1. **异步操作**：使用 tokio 异步运行时
2. **连接池**：数据库连接复用
3. **缓存策略**：配置缓存，减少 IO
4. **资源管理**：RAII 模式，自动释放

### 网络优化

1. **并发控制**：限制并发请求
2. **重试机制**：指数退避重试
3. **压缩传输**：支持 gzip 压缩
4. **CDN 加速**：资源使用 CDN

## 扩展性设计

### 插件架构

预留插件接口，支持：

1. **自定义解析器**：支持新的订阅格式
2. **自定义规则**：添加新的路由规则
3. **主题扩展**：支持自定义主题
4. **功能插件**：扩展新功能

### API 设计

1. **RESTful API**：统一的接口规范
2. **版本兼容**：向后兼容的 API 版本
3. **文档完善**：自动生成 API 文档
4. **测试覆盖**：完整的 API 测试

### 配置扩展

1. **动态配置**：运行时修改配置
2. **配置模板**：预定义配置模板
3. **导入导出**：配置的备份和迁移
4. **配置验证**：确保配置正确性

## 总结

Sing-Box Windows 的架构设计遵循了现代软件工程的最佳实践：

1. **清晰的分层**：前后端分离，职责明确
2. **模块化设计**：功能独立，易于维护
3. **事件驱动**：松耦合，响应式
4. **数据一致性**：统一的数据管理
5. **安全可靠**：多重安全防护
6. **高性能**：异步优化，资源高效
7. **可扩展**：预留扩展点，支持插件

这种架构设计确保了应用的可维护性、可扩展性和高性能，为后续的功能开发和优化奠定了坚实的基础。