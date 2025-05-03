<template>
  <n-card class="status-card" :bordered="false">
    <div class="status-header">
      <div class="status-left">
        <div class="status-indicator">
          <div class="status-dot" :class="{ active: isRunning }"></div>
          <span class="status-text">{{
            isRunning ? t('home.status.running') : t('home.status.stopped')
          }}</span>
        </div>
        <div class="status-tags">
          <n-tag
            :bordered="false"
            :type="wsConnected ? 'success' : 'error'"
            class="status-tag"
          >
            <template #icon>
              <n-icon size="16">
                <wifi-outline v-if="wsConnected" />
                <close-circle-outline v-else />
              </n-icon>
            </template>
            {{ wsConnected ? t('home.wsStatus.connected') : t('home.wsStatus.disconnected') }}
          </n-tag>
          <n-tag :bordered="false" :type="isAdmin ? 'success' : 'warning'" class="status-tag">
            <template #icon>
              <n-icon size="16">
                <shield-checkmark-outline v-if="isAdmin" />
                <shield-outline v-else />
              </n-icon>
            </template>
            {{ isAdmin ? t('home.adminStatus.admin') : t('home.adminStatus.normal') }}
          </n-tag>
        </div>
      </div>
      <div class="status-right">
        <!-- 启动/停止按钮 -->
        <n-button
          :type="isRunning ? 'error' : 'primary'"
          size="medium"
          :loading="isStarting || isStopping"
          @click="handleToggle"
          class="control-button"
        >
          <template #icon>
            <n-icon>
              <power-outline />
            </n-icon>
          </template>
          {{ isRunning ? t('home.stop') : t('home.start') }}
        </n-button>
      </div>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { PowerOutline, WifiOutline, CloseCircleOutline, ShieldCheckmarkOutline, ShieldOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'

defineOptions({
  name: 'StatusCard'
})

const props = defineProps({
  isRunning: {
    type: Boolean,
    default: false
  },
  wsConnected: {
    type: Boolean,
    default: false
  },
  isAdmin: {
    type: Boolean,
    default: false
  },
  isStarting: {
    type: Boolean,
    default: false
  },
  isStopping: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['start', 'stop'])
const { t } = useI18n()

const handleToggle = () => {
  if (props.isRunning) {
    emit('stop')
  } else {
    emit('start')
  }
}
</script>

<style scoped>
.status-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
  overflow: hidden;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(128, 128, 128, 0.1);
}

.status-card :deep(.n-card__content) {
  padding: 12px 16px;
}

.status-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.status-left {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-wrap: wrap;
}

.status-right {
  display: flex;
  gap: 12px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background-color: var(--n-text-color-disabled);
  transition: all 0.3s ease;
  position: relative;
}

.status-dot.active {
  background-color: var(--success-color);
  box-shadow: 0 0 10px var(--success-color);
}

.status-dot.active::after {
  content: '';
  position: absolute;
  top: -5px;
  left: -5px;
  right: -5px;
  bottom: -5px;
  border-radius: 50%;
  border: 1px solid var(--success-color);
  opacity: 0.4;
  animation: pulse 1.8s infinite;
}

@keyframes pulse {
  0% {
    transform: scale(0.95);
    opacity: 0.6;
  }
  70% {
    transform: scale(1.2);
    opacity: 0.3;
  }
  100% {
    transform: scale(0.95);
    opacity: 0.6;
  }
}

.status-text {
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.status-tags {
  display: flex;
  gap: 10px;
}

.status-tag {
  height: 28px;
  padding: 0 10px;
  border-radius: 8px;
  font-weight: 500;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.06);
}

.control-button {
  border-radius: 10px;
  font-weight: 600;
  padding: 0 18px;
  height: 38px;
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.1);
  transition: all 0.2s ease;
}

.control-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

@media (max-width: 768px) {
  .status-card :deep(.n-card__content) {
    padding: 12px 16px;
  }
  
  .status-left,
  .status-right {
    width: 100%;
  }

  .status-header {
    flex-direction: column;
    gap: 16px;
  }
  
  .status-left {
    justify-content: space-between;
  }
  
  .control-button {
    width: 100%;
  }
}
</style> 