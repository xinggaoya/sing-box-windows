<template>
  <n-card class="status-card" :bordered="false">
    <div class="status-header">
      <div class="status-left">
        <div class="status-indicator">
          <div class="status-dot" :class="statusClass"></div>
          <span class="status-text">{{ statusText }}</span>
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
        <!-- 非管理员模式时显示重启按钮 -->
        <n-button
          v-if="!isAdmin"
          type="warning"
          size="small"
          @click="$emit('restart-as-admin')"
          :loading="isRestarting"
          class="admin-button"
        >
          <template #icon>
            <n-icon>
              <refresh-outline />
            </n-icon>
          </template>
          {{ t('notification.restartAsAdmin') }}
        </n-button>

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
import { PowerOutline, WifiOutline, CloseCircleOutline, ShieldCheckmarkOutline, ShieldOutline, RefreshOutline } from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'

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
  },
  isRestarting: {
    type: Boolean,
    default: false
  },
  isConnecting: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['start', 'stop', 'restart-as-admin'])
const { t } = useI18n()

const handleToggle = () => {
  if (props.isRunning) {
    emit('stop')
  } else {
    emit('start')
  }
}

const statusClass = computed(() => {
  if (props.isStarting || props.isStopping || props.isRestarting || props.isConnecting) {
    return 'status-pending'
  }
  return props.isRunning && props.wsConnected
    ? 'status-active'
    : props.isRunning && !props.wsConnected
    ? 'status-warning'
    : 'status-inactive'
})

const statusText = computed(() => {
  if (props.isStarting) {
    return t('status.starting')
  } else if (props.isStopping) {
    return t('status.stopping')
  } else if (props.isRestarting) {
    return t('status.restarting')
  } else if (props.isConnecting) {
    return t('status.connecting')
  } else if (props.isRunning && props.wsConnected) {
    return t('status.running')
  } else if (props.isRunning && !props.wsConnected) {
    return t('status.disconnected')
  } else {
    return t('status.stopped')
  }
})
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
  align-items: center;
  gap: 10px;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background-color: currentColor;
  display: inline-block;
  margin-right: 8px;
  position: relative;
}

.status-pending .status-dot::after {
  content: '';
  position: absolute;
  top: -4px;
  left: -4px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background-color: currentColor;
  opacity: 0.3;
  animation: pulse 1.5s infinite ease-in-out;
}

@keyframes pulse {
  0% {
    transform: scale(0.6);
    opacity: 0.5;
  }
  50% {
    transform: scale(1);
    opacity: 0.2;
  }
  100% {
    transform: scale(0.6);
    opacity: 0.5;
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

.admin-button {
  font-weight: 500;
  border-radius: 8px;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  height: 32px;
}

.admin-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(250, 173, 20, 0.25);
}

/* 添加状态颜色 */
.status-active {
  color: var(--success-color, #18a058);
}

.status-inactive {
  color: var(--error-color, #d03050);
}

.status-warning {
  color: var(--warning-color, #f0a020);
}

.status-pending {
  color: var(--info-color, #2080f0);
}

@media (max-width: 768px) {
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
  
  .status-right {
    flex-direction: row;
    justify-content: space-between;
  }
  
  .admin-button {
    flex: 1;
    margin-right: 8px;
  }
  
  .control-button {
    flex: 1;
  }
}
</style> 