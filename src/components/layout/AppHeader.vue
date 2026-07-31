<template>
  <header class="app-header" data-tauri-drag-region>
    <!-- 品牌 + 运行状态 -->
    <div class="header-brand">
      <div class="brand-logo-wrapper" @click="emit('home')">
        <img
          :src="logo"
          alt="Logo"
          class="brand-logo"
          :class="{ 'is-running': kernelStatusClass === 'running' }"
        />
      </div>
      <div class="brand-text">
        <h1 class="app-name">{{ appName }}</h1>
        <div class="app-status" :class="kernelStatusClass">
          <span class="status-dot"></span>
          <span class="status-label">{{ appStatusLabel }}</span>
        </div>
      </div>
    </div>

    <!-- 中部速览（运行中才显示流量与连接数） -->
    <div class="header-glance" v-if="kernelStatusClass === 'running'">
      <div class="glance-item up">
        <n-icon :size="14"><ArrowUpOutline /></n-icon>
        <span class="glance-value">{{ formatSpeed(uploadSpeed) }}</span>
      </div>
      <div class="glance-item down">
        <n-icon :size="14"><ArrowDownOutline /></n-icon>
        <span class="glance-value">{{ formatSpeed(downloadSpeed) }}</span>
      </div>
      <div class="glance-divider"></div>
      <div class="glance-item">
        <n-icon :size="14"><LinkOutline /></n-icon>
        <span class="glance-value">{{ connectionCount }}</span>
      </div>
    </div>

    <!-- 窗口控制 -->
    <div class="window-controls">
      <button class="control-btn minimize" @click="emit('minimize')">
        <n-icon size="16"><RemoveOutline /></n-icon>
      </button>
      <button class="control-btn maximize" @click="emit('toggle-maximize')">
        <n-icon size="13"><SquareOutline /></n-icon>
      </button>
      <button class="control-btn close" @click="emit('close')">
        <n-icon size="16"><CloseOutline /></n-icon>
      </button>
    </div>
  </header>
</template>

<script lang="ts" setup>
import {
  ArrowUpOutline,
  ArrowDownOutline,
  LinkOutline,
  RemoveOutline,
  SquareOutline,
  CloseOutline,
} from '@vicons/ionicons5'
import logo from '@/assets/icon.png'
import { formatSpeed } from '@/utils'

defineProps<{
  appName: string
  kernelStatusClass: string
  appStatusLabel: string
  uploadSpeed: number
  downloadSpeed: number
  connectionCount: number
}>()

const emit = defineEmits<{
  (e: 'home'): void
  (e: 'minimize'): void
  (e: 'toggle-maximize'): void
  (e: 'close'): void
}>()
</script>

<style scoped>
.app-header {
  height: var(--header-height, 48px);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 var(--space-4);
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur, 18px));
  -webkit-backdrop-filter: blur(var(--glass-blur, 18px));
  border-bottom: 1px solid var(--border-color);
  z-index: var(--z-header, 200);
  gap: var(--space-4);
}

/* 品牌 */
.header-brand {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-width: 0;
}

.brand-logo-wrapper {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: transform var(--transition-fast);
}

.brand-logo-wrapper:hover {
  transform: scale(1.06);
}

.brand-logo {
  width: 26px;
  height: 26px;
  object-fit: contain;
  filter: drop-shadow(0 2px 6px rgba(0, 0, 0, 0.12));
  transition: filter var(--transition-base);
}

.brand-logo.is-running {
  filter: drop-shadow(0 0 10px var(--primary-soft-strong));
}

.brand-text {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
}

.app-name {
  font-size: var(--text-md, 15px);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  line-height: 1.1;
  white-space: nowrap;
}

.app-status {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: 1;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--text-tertiary);
  transition: background var(--transition-base);
}

.app-status.running .status-dot {
  background: var(--success-color);
  box-shadow: 0 0 8px var(--success-color);
  animation: pulse 2s ease-in-out infinite;
}

.app-status.running {
  color: var(--success-color);
}

.app-status.starting,
.app-status.stopping,
.app-status.disconnected {
  color: var(--warning-color);
}

.app-status.starting .status-dot,
.app-status.stopping .status-dot,
.app-status.disconnected .status-dot {
  background: var(--warning-color);
  box-shadow: 0 0 6px var(--warning-color);
}

.app-status.stopped,
.app-status.failed {
  color: var(--error-color);
}

.app-status.stopped .status-dot,
.app-status.failed .status-dot {
  background: var(--error-color);
  box-shadow: 0 0 6px var(--error-color);
}

.app-status.crashed {
  color: var(--orange-500);
}

.app-status.crashed .status-dot {
  background: var(--orange-500);
  box-shadow: 0 0 6px var(--orange-500);
}

@keyframes pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.55;
  }
}

/* 中部速览 */
.header-glance {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: 0 var(--space-3);
  -webkit-app-region: no-drag;
}

.glance-item {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: var(--text-sm);
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.glance-item.up {
  color: var(--success-color);
}

.glance-item.down {
  color: var(--info-color);
}

.glance-value {
  font-weight: 600;
}

.glance-divider {
  width: 1px;
  height: 14px;
  background: var(--border-color);
}

/* 窗口控制 */
.window-controls {
  display: flex;
  gap: var(--space-1);
  -webkit-app-region: no-drag;
}

.control-btn {
  width: 30px;
  height: 30px;
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.control-btn:hover {
  background: var(--bg-surface-2);
  color: var(--text-primary);
}

.control-btn.close:hover {
  background: var(--red-500-soft);
  color: var(--error-color);
}

@media (max-width: 720px) {
  .header-glance {
    display: none;
  }
}
</style>
