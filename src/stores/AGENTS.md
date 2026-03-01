# STORES KNOWLEDGE BASE

## OVERVIEW

Pinia 状态中心，按领域拆分为 app/kernel/subscription/tray；通过组合式与服务层交互后端。

## WHERE TO LOOK

| Task                 | Location        | Notes                         |
| -------------------- | --------------- | ----------------------------- |
| Store 注入与导出入口 | `index.ts`      | 全局注册与聚合入口            |
| 应用级配置状态       | `app/`          | 主题、语言、窗口、持久化      |
| 内核运行与连接状态   | `kernel/`       | Kernel/Traffic/Connection/Log |
| 订阅状态             | `subscription/` | 活跃配置、节点/配置索引       |
| 托盘交互状态         | `tray/`         | 托盘联动与窗口行为            |

## CONVENTIONS

- Store 名称保持 `XxxStore.ts`。
- 对外暴露动作优先语义化方法，避免组件直接写底层细节。
- 涉及持久化时，优先走现有 persistence/composables 模式。
- 初始化顺序遵循 bootstrap 编排，不在单个 store 里擅自重排。

## ANTI-PATTERNS

- ❌ 在多个 store 间直接互相写状态（应通过动作或服务）。
- ❌ 将后端调用逻辑散落进组件（应留在 services + store action）。
- ❌ 在 store 内引入一次性调试副作用且不清理监听器。

## NOTES

- 内核相关 store 数量多且耦合高，修改时先核对 `boot/useAppBootstrap.ts` 初始化/清理顺序。
- `app/` 子域含 composables，已形成项目特有持久化模式。
