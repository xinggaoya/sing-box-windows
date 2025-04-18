# Stores 目录结构说明

本项目使用 Pinia 进行状态管理，所有的 store 按照功能模块进行了组织。

## 目录结构

```
src/stores/
├── index.ts                # 主入口文件，导出所有 store
├── app/                    # 应用相关 store
│   ├── AppStore.ts         # 核心应用状态
│   ├── ThemeStore.ts       # 主题管理
│   ├── LocaleStore.ts      # 本地化/语言
│   ├── WindowStore.ts      # 窗口管理
│   └── UpdateStore.ts      # 应用更新
├── kernel/                 # 内核相关 store
│   ├── KernelStore.ts      # 内核状态和操作
│   ├── ProxyStore.ts       # 代理设置
│   ├── ConnectionStore.ts  # 连接管理
│   ├── TrafficStore.ts     # 流量监控
│   └── LogStore.ts         # 日志管理
├── subscription/           # 订阅相关 store
│   └── SubStore.ts         # 订阅管理
└── tray/                   # 系统托盘相关 store
    └── TrayStore.ts        # 系统托盘管理
```

## Store 职责说明

### 应用相关 Store

- **AppStore**: 管理核心应用状态，如运行状态、自动启动设置等
- **ThemeStore**: 管理应用主题（亮色/暗色）
- **LocaleStore**: 管理应用语言设置
- **WindowStore**: 管理窗口状态、操作和事件
- **UpdateStore**: 管理应用更新检查和安装

### 内核相关 Store

- **KernelStore**: 管理内核版本、启动/停止操作
- **ProxyStore**: 管理代理设置和节点
- **ConnectionStore**: 管理连接信息和统计
- **TrafficStore**: 管理流量监控和统计
- **LogStore**: 管理日志记录和显示

### 订阅相关 Store

- **SubStore**: 管理代理订阅

### 系统托盘相关 Store

- **TrayStore**: 管理系统托盘图标和菜单

## 使用示例

```typescript
// 导入需要的 store
import { useAppStore } from '@/stores/app/AppStore'
import { useThemeStore } from '@/stores/app/ThemeStore'

// 在组件中使用
const appStore = useAppStore()
const themeStore = useThemeStore()

// 使用 store 中的状态和方法
const isRunning = appStore.isRunning
themeStore.toggleTheme()
```
