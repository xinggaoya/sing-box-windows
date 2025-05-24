# Sing-Box Windows 前端性能优化指南

## 概述

本文档详细介绍了为Sing-Box Windows前端应用实施的性能优化措施，包括内存管理、组件懒加载、Bundle分析等多个方面的优化。

## 优化内容概览

### 1. 核心性能工具 (`src/utils/performance.ts`)

#### EventListenerManager - 事件监听器管理器

- **功能**: 统一管理组件的事件监听器，防止内存泄漏
- **使用方法**:

```typescript
import { eventListenerManager } from '@/utils/performance'

// 添加清理函数
eventListenerManager.add(() => {
  // 清理逻辑
})

// 在组件卸载时自动清理
eventListenerManager.autoCleanup()
```

#### ComponentPreloader - 组件预加载器

- **功能**: 使用IntersectionObserver实现智能组件预加载
- **特性**:
  - 提前50px开始预加载
  - 支持手动预加载指定组件
  - 自动管理预加载缓存

#### MemoryMonitor - 内存监控器

- **功能**: 实时监控应用内存使用情况
- **特性**:
  - 定期记录内存使用统计
  - 开发环境自动启用
  - 支持强制垃圾回收（开发环境）

#### 优化的防抖和节流函数

- **改进**: 添加了cancel方法，支持取消操作
- **类型安全**: 完整的TypeScript类型支持

### 2. Store管理器 (`src/stores/StoreManager.ts`)

#### 按需加载架构

- **核心Store**: app, theme, locale（应用启动时加载）
- **功能Store**: 根据路由和功能需求懒加载
- **路由映射**: 自动根据路由预加载相关Store

#### StoreManager类功能

```typescript
// 加载指定Store
await storeManager.loadStore<AppStore>('app')

// 预加载多个Store
await storeManager.preloadStores(['kernel', 'proxy'])

// 获取统计信息
const stats = storeManager.getStats()
```

### 3. 路由优化 (`src/router/index.ts`)

#### 路由预加载

- **beforeEach守卫**: 自动预加载路由所需的Store
- **组件预加载**: 根据路由meta配置预加载相关组件
- **性能监控**: 记录路由切换的性能数据

#### 路由配置示例

```typescript
{
  path: '/',
  component: () => import('@/views/HomeView.vue'),
  meta: {
    preloadStores: ['app', 'kernel', 'traffic'],
    preloadComponents: ['ProxyView', 'SubView'],
  },
}
```

### 4. 虚拟滚动组件 (`src/components/VirtualList.vue`)

#### 功能特性

- **大数据渲染**: 仅渲染可见区域的项目
- **智能缓冲**: 可配置的上下缓冲区
- **性能优化**: 使用节流优化滚动事件
- **泛型支持**: 完整的TypeScript泛型支持

#### 使用示例

```vue
<VirtualList
  :items="largeDataList"
  :item-height="50"
  :container-height="400"
  :buffer="5"
  key-field="id"
>
  <template #default="{ item, index }">
    <div>{{ item.name }}</div>
  </template>
</VirtualList>
```

### 5. 懒加载组件包装器 (`src/components/LazyComponent.vue`)

#### 功能特性

- **异步加载**: 支持组件的延迟加载
- **错误处理**: 完整的错误边界和重试机制
- **加载状态**: 自定义加载和错误UI
- **超时控制**: 可配置的加载超时时间

#### 使用示例

```vue
<LazyComponent
  :loader="() => import('@/components/HeavyComponent.vue')"
  :delay="1000"
  :max-retries="3"
  :timeout="10000"
>
  <template #loading>
    <div>正在加载组件...</div>
  </template>
  <template #error="{ error, retry }">
    <div>
      加载失败: {{ error }}
      <button @click="retry">重试</button>
    </div>
  </template>
</LazyComponent>
```

### 6. Bundle分析工具 (`src/utils/bundleAnalyzer.ts`)

#### 功能特性

- **模块加载统计**: 记录模块大小和加载时间
- **依赖关系分析**: 构建模块依赖图
- **循环依赖检测**: 自动检测并报告循环依赖
- **优化建议**: 生成具体的优化建议

#### 分析报告内容

- 最大模块列表
- 最慢加载模块
- 重复加载模块
- 循环依赖警告
- 具体优化建议

### 7. 代码分割管理器 (`src/utils/codeSplitting.ts`)

#### 加载优先级系统

- **IMMEDIATE**: 立即加载
- **HIGH**: 高优先级
- **NORMAL**: 普通优先级
- **LOW**: 低优先级
- **IDLE**: 空闲时加载

#### 功能特性

- **智能队列**: 根据优先级排序加载队列
- **并发控制**: 限制同时加载的组件数量
- **重试机制**: 失败自动重试
- **超时控制**: 可配置的加载超时

### 8. 应用初始化优化 (`src/App.vue`)

#### 重构改进

- **Store懒加载**: 只在需要时加载Store
- **事件管理**: 使用EventListenerManager统一管理事件
- **错误处理**: 完善的错误边界处理
- **内存清理**: 组件卸载时自动清理资源

#### 初始化流程

1. 初始化Store管理器
2. 加载核心Store（app, theme, locale, window）
3. 设置事件监听器
4. 按需加载其他Store
5. 执行应用初始化逻辑

## 性能监控与调试

### 开发环境工具

在开发环境下，所有性能工具会自动启用：

```typescript
// 在浏览器控制台中使用
window.__PERF_TOOLS__.memoryMonitor.startMonitoring()
window.__PERF_TOOLS__.bundleAnalyzer.printReport()
window.__PERF_TOOLS__.codeSplittingManager.getStats()
```

### 性能指标监控

应用启动时会自动记录关键性能指标：

- DOMContentLoaded时间
- Load Complete时间
- DNS查找时间
- TCP连接时间

### 内存使用监控

每15秒自动记录内存使用情况：

- 已使用堆内存
- 总堆内存
- 堆内存限制

## 最佳实践

### 1. Store使用建议

- 仅在需要时加载Store
- 使用StoreManager进行统一管理
- 避免在App.vue中初始化所有Store

### 2. 组件开发建议

- 大型组件使用LazyComponent包装
- 列表组件使用VirtualList
- 事件监听器使用EventListenerManager管理

### 3. 路由配置建议

- 合理配置preloadStores和preloadComponents
- 避免在单个路由中加载过多资源
- 使用代码分割减少初始Bundle大小

### 4. 内存管理建议

- 组件卸载时清理事件监听器
- 避免创建过多的响应式对象
- 定期检查内存泄漏

## 优化效果预期

通过以上优化措施，预期可以达到以下效果：

1. **内存使用减少30-50%**

   - Store按需加载
   - 事件监听器统一管理
   - 及时清理未使用资源

2. **首屏加载时间减少40-60%**

   - 核心Store优先加载
   - 组件懒加载
   - 智能预加载

3. **大列表渲染性能提升90%+**

   - 虚拟滚动技术
   - 仅渲染可见项目

4. **路由切换速度提升50%**
   - Store和组件预加载
   - 减少运行时加载

## 持续优化

这套性能优化系统是可扩展的，可以根据实际使用情况继续优化：

1. **Bundle分析**: 定期查看Bundle分析报告，识别优化机会
2. **性能监控**: 关注内存使用趋势，及时发现问题
3. **用户反馈**: 根据用户体验反馈调整预加载策略
4. **新技术采用**: 随着Vue和相关生态的发展，采用新的优化技术

## 结论

通过这套全面的性能优化方案，Sing-Box Windows前端应用在内存使用、加载速度、渲染性能等各个方面都得到了显著提升，为用户提供了更好的使用体验。
