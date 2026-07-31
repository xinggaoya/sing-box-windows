<template>
  <div class="empty-state-box">
    <div class="empty-icon-box">
      <slot name="icon">
        <n-icon :size="32" v-if="icon"><component :is="icon" /></n-icon>
      </slot>
    </div>
    <h3 class="empty-title-box">{{ title }}</h3>
    <p v-if="description" class="empty-desc-box">{{ description }}</p>
    <div v-if="$slots.action" class="empty-action">
      <slot name="action"></slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { Component } from 'vue'
/**
 * 统一空状态：图标 + 标题 + 描述 + 可选操作按钮。
 */
defineProps<{
  title: string
  description?: string
  icon?: Component
}>()
</script>

<style scoped>
.empty-state-box {
  padding: clamp(32px, 4vw, 56px) var(--space-6);
  text-align: center;
  border-radius: var(--radius-xl);
  background: var(--panel-bg);
  border: 1px dashed var(--border-color);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
}

.empty-icon-box {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-lg);
  background: var(--bg-surface-2);
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-title-box {
  margin: 0;
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

.empty-desc-box {
  margin: 0;
  font-size: var(--text-base);
  color: var(--text-secondary);
  max-width: 360px;
}

.empty-action {
  margin-top: var(--space-2);
}
</style>
