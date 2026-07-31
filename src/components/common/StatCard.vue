<template>
  <div class="stat-metric" :data-accent="accent">
    <div class="metric-icon">
      <slot name="icon"></slot>
    </div>
    <div class="metric-body">
      <div class="metric-value">{{ value }}</div>
      <div class="metric-label">{{ label }}</div>
    </div>
    <div v-if="$slots.extra" class="metric-extra">
      <slot name="extra"></slot>
    </div>
  </div>
</template>

<script lang="ts" setup>
/**
 * 紧凑型指标卡：图标 + 值 + 标签，支持 accent 色板。
 */
withDefaults(
  defineProps<{
    value: string | number
    label: string
    accent?: 'default' | 'pink' | 'blue' | 'amber' | 'purple' | 'green'
  }>(),
  { accent: 'default' },
)
</script>

<style scoped>
.stat-metric {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  box-shadow: var(--panel-shadow);
  transition:
    transform var(--transition-fast),
    box-shadow var(--transition-fast);
}

.stat-metric:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-md);
}

.metric-icon {
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background: var(--primary-soft);
  color: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-metric[data-accent='pink'] .metric-icon {
  background: rgba(236, 72, 153, 0.14);
  color: #ec4899;
}

.stat-metric[data-accent='blue'] .metric-icon {
  background: var(--info-soft);
  color: var(--info-color);
}

.stat-metric[data-accent='amber'] .metric-icon {
  background: var(--warning-soft);
  color: var(--warning-color);
}

.stat-metric[data-accent='purple'] .metric-icon {
  background: var(--primary-soft-strong);
  color: var(--primary-active);
}

.stat-metric[data-accent='green'] .metric-icon {
  background: var(--success-soft);
  color: var(--success-color);
}

.metric-body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.metric-value {
  font-size: var(--text-lg);
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.2;
  font-variant-numeric: tabular-nums;
}

.metric-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.metric-extra {
  margin-left: auto;
}
</style>
