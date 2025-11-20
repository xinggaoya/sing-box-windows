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
defineProps<{
  label: string
  value: string | number
  description?: string
  type?: 'default' | 'primary' | 'success' | 'warning' | 'error'
}>()
</script>

<style scoped>
.status-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.status-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--panel-shadow);
  border-color: var(--border-hover);
}

.card-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  font-size: 24px;
  flex-shrink: 0;
}

.status-card.primary .card-icon {
  background: rgba(99, 102, 241, 0.1);
  color: var(--primary-color);
}

.status-card.success .card-icon {
  background: rgba(16, 185, 129, 0.1);
  color: #10b981;
}

.status-card.warning .card-icon {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.status-card.error .card-icon {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.card-content {
  flex: 1;
  min-width: 0;
}

.card-label {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
  margin-bottom: 4px;
}

.card-value {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
}

.card-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-top: 2px;
}

.card-action {
  margin-left: auto;
}
</style>
