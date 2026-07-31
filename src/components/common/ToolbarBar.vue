<template>
  <div class="toolbar-bar">
    <div class="toolbar-left">
      <div v-if="$slots.tabs" class="toolbar-tabs">
        <slot name="tabs"></slot>
      </div>
      <div v-if="$slots.filters" class="toolbar-filters">
        <slot name="filters"></slot>
      </div>
    </div>
    <div v-if="$slots.stats" class="toolbar-stats">
      <slot name="stats"></slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
/**
 * 工具栏容器：左侧 tabs + filters，右侧统计区。
 * 用于日志/连接/规则页统一顶部条结构。
 */
</script>

<style scoped>
.toolbar-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  flex-wrap: wrap;
  padding: var(--space-3) var(--space-4);
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--panel-shadow);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  flex-wrap: wrap;
  min-width: 0;
}

.toolbar-tabs {
  display: flex;
  align-items: center;
  /* tabs 与 filters 之间的视觉分隔 */
  padding-right: var(--space-4);
  border-right: 1px solid var(--border-color);
}

/* 当 tabs 为空（不传该插槽）时不显示多余分隔线 */
.toolbar-tabs:empty {
  display: none;
}

/* 压缩 naive-ui n-tabs(segment) 自带内边距，避免挤占 filters 空间 */
.toolbar-tabs :deep(.n-tabs) {
  --n-tab-padding: 5px 12px;
}

.toolbar-tabs :deep(.n-tabs-tab) {
  padding: 5px 12px !important;
}

.toolbar-tabs :deep(.n-tabs-nav--segment-type) {
  padding: 2px;
}

.toolbar-filters {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.toolbar-stats {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

@media (max-width: 768px) {
  .toolbar-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .toolbar-tabs {
    padding-right: 0;
    border-right: none;
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--border-color);
  }

  .toolbar-stats {
    justify-content: flex-start;
  }
}
</style>
