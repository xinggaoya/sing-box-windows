<template>
  <span class="status-badge" :class="status">
    <span class="badge-dot"></span>
    <span class="badge-text"><slot>{{ label }}</slot></span>
  </span>
</template>

<script lang="ts" setup>
/**
 * 状态徽章：色点 + 文案，支持脉冲动画（运行中）。
 * status 决定配色：
 *   running / success → 绿
 *   starting / stopping / pending / disconnected → 黄
 *   stopped / failed / error → 红
 *   crashed → 橙
 *   info / default → 主色/灰
 */
withDefaults(
  defineProps<{
    status?:
      | 'running'
      | 'success'
      | 'starting'
      | 'stopping'
      | 'pending'
      | 'disconnected'
      | 'stopped'
      | 'failed'
      | 'error'
      | 'crashed'
      | 'info'
      | 'default'
    label?: string
    /** 运行中是否脉冲，默认 running 启用 */
    pulse?: boolean
  }>(),
  { status: 'default', pulse: undefined },
)
</script>

<style scoped>
.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 10px;
  border-radius: var(--radius-pill);
  font-size: var(--text-xs);
  font-weight: 600;
  background: var(--bg-surface-2);
  color: var(--text-secondary);
  line-height: 1.4;
}

.badge-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--text-tertiary);
  flex-shrink: 0;
}

/* 配色组 */
.status.running,
.status.success {
  background: var(--success-soft);
  color: var(--success-color);
}
.status.running .badge-dot,
.status.success .badge-dot {
  background: var(--success-color);
  box-shadow: 0 0 6px var(--success-color);
}

.status.starting,
.status.stopping,
.status.pending,
.status.disconnected {
  background: var(--warning-soft);
  color: var(--warning-color);
}
.status.starting .badge-dot,
.status.stopping .badge-dot,
.status.pending .badge-dot,
.status.disconnected .badge-dot {
  background: var(--warning-color);
  box-shadow: 0 0 6px var(--warning-color);
}

.status.stopped,
.status.failed,
.status.error {
  background: var(--error-soft);
  color: var(--error-color);
}
.status.stopped .badge-dot,
.status.failed .badge-dot,
.status.error .badge-dot {
  background: var(--error-color);
  box-shadow: 0 0 6px var(--error-color);
}

.status.crashed {
  background: var(--orange-500-soft);
  color: var(--orange-500);
}
.status.crashed .badge-dot {
  background: var(--orange-500);
  box-shadow: 0 0 6px var(--orange-500);
}

.status.info {
  background: var(--primary-soft);
  color: var(--primary-color);
}
.status.info .badge-dot {
  background: var(--primary-color);
  box-shadow: 0 0 6px var(--primary-color);
}

/* 脉冲动画：默认 running 启用 */
.status.running .badge-dot {
  animation: badge-pulse 2s ease-in-out infinite;
}

@keyframes badge-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
