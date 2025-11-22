<template>
  <div class="blank-view">
    <div class="tray-indicator" v-show="showIndicator">
      <div class="tray-icon"></div>
      <span class="tray-text">{{ t('common.minimizedToTray') }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'

const { t } = useI18n()
const showIndicator = ref(false)

let indicatorTimer: number | null = null

onMounted(() => {
  // 进入空白页时清理消息，避免提示在托盘模式下悬挂
  mitt.emit('clear-ui-messages')

  indicatorTimer = window.setTimeout(() => {
    showIndicator.value = true
  }, 300)
})

onBeforeUnmount(() => {
  if (indicatorTimer) {
    clearTimeout(indicatorTimer)
    indicatorTimer = null
  }
})
</script>

<style scoped>
.blank-view {
  width: 100%;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--n-card-color);
  color: var(--n-text-color-disabled);
}

.tray-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  opacity: 0.55;
}

.tray-icon {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--n-text-color-disabled);
  position: relative;
}

.tray-icon::after {
  content: '';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--n-success-color);
}

.tray-text {
  font-size: 12px;
  color: var(--n-text-color-disabled);
  font-weight: 500;
  text-rendering: optimizeSpeed;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>
