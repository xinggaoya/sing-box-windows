<template>
  <div class="status-card" :class="type">
    <div class="card-icon">
      <slot name="icon"></slot>
    </div>
    <div class="card-content">
      <div class="card-label">{{ label }}</div>
      <div class="card-value">{{ value }}</div>
      <div class="card-desc" v-if="description">{{ description }}</div>
    </div>
    <div class="card-action" v-if="$slots.action">
      <slot name="action"></slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
import type { StatusCardType } from '@/types'

interface StatusCardProps {
  label: string
  value: string | number
  description?: string
  type?: StatusCardType
}

withDefaults(defineProps<StatusCardProps>(), {
  type: 'default'
})
</script>

<style scoped>
.status-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  padding: var(--space-5);
  display: flex;
  align-items: center;
  gap: var(--space-4);
  transition:
    transform var(--transition-fast),
    box-shadow var(--transition-fast),
    border-color var(--transition-fast);
  position: relative;
  overflow: hidden;
}

.status-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--border-hover);
}

.card-icon {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-surface-2);
  color: var(--text-secondary);
  font-size: 22px;
  flex-shrink: 0;
}

/* 保证插槽中的 SVG 图标在不同平台有统一尺寸 */
.card-icon :deep(svg) {
  width: 1.2em;
  height: 1.2em;
  display: block;
}

.status-card.primary .card-icon {
  background: var(--primary-soft);
  color: var(--primary-color);
}

.status-card.success .card-icon {
  background: var(--success-soft);
  color: var(--success-color);
}

.status-card.warning .card-icon {
  background: var(--warning-soft);
  color: var(--warning-color);
}

.status-card.error .card-icon {
  background: var(--error-soft);
  color: var(--error-color);
}

.card-content {
  flex: 1;
  min-width: 0;
}

.card-label {
  font-size: var(--text-xs);
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
  margin-bottom: var(--space-1);
}

.card-value {
  font-size: var(--text-xl);
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.card-desc {
  font-size: var(--text-xs);
  color: var(--text-secondary);
  margin-top: 2px;
}

.card-action {
  margin-left: auto;
}
</style>
