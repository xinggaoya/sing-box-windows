<template>
  <n-card class="mode-card" :bordered="false">
    <div class="mode-card-header">
      <n-icon size="20" class="mode-card-icon">
        <component :is="icon" />
      </n-icon>
      <h3 class="mode-card-title">{{ title }}</h3>
    </div>
    <div class="mode-card-content">
      <div class="mode-buttons">
        <n-button-group size="medium">
          <n-button 
            v-for="mode in modes"
            :key="mode.value"
            :type="currentMode === mode.value ? 'primary' : 'default'"
            :disabled="disabled"
            @click="$emit('mode-change', mode.value)"
            class="mode-button"
          >
            <template #icon>
              <n-icon>
                <component :is="mode.icon" />
              </n-icon>
            </template>
            {{ mode.label || (mode.nameKey ? t(mode.nameKey) : '') }}
          </n-button>
        </n-button-group>
      </div>
      <div class="mode-description">
        {{ currentMode ? t(`${descriptionPrefix}${currentMode}Description`) : '' }}
      </div>
    </div>
  </n-card>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { computed } from 'vue'
import type { Component } from 'vue'

defineOptions({
  name: 'ProxyModeCard'
})

interface ModeItem {
  value: string
  nameKey?: string
  label?: string
  icon: Component
}

const props = defineProps({
  title: {
    type: String,
    required: true
  },
  icon: {
    type: Object as () => Component,
    required: true
  },
  currentMode: {
    type: String,
    default: ''
  },
  modes: {
    type: Array as () => ModeItem[],
    required: true
  },
  disabled: {
    type: Boolean,
    default: false
  },
  descriptionPrefix: {
    type: String,
    default: 'home.proxyMode.'
  }
})

defineEmits(['mode-change'])

const { t } = useI18n()
</script>

<style scoped>
.mode-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
  height: 100%;
  overflow: hidden;
  backdrop-filter: blur(8px);
  border: 1px solid rgba(128, 128, 128, 0.1);
}

.mode-card :deep(.n-card__content) {
  padding: 12px 16px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.mode-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid rgba(128, 128, 128, 0.1);
}

.mode-card-icon {
  color: var(--primary-color);
  flex-shrink: 0;
}

.mode-card-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.mode-card-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
  flex-grow: 1;
}

.mode-buttons {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.mode-buttons :deep(.n-button-group) {
  width: 100%;
  display: flex;
}

.mode-buttons :deep(.n-button) {
  flex: 1;
  border-radius: 8px;
  font-weight: 500;
  height: 36px;
}

/* 使用更现代的阴影和交互效果 */
.mode-buttons :deep(.n-button:not(:disabled)) {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.2s ease;
}

.mode-buttons :deep(.n-button:not(:disabled):hover) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.mode-buttons :deep(.n-button-group .n-button:not(:first-child)) {
  margin-left: 8px;
}

.mode-description {
  font-size: 14px;
  color: var(--n-text-color-3);
  line-height: 1.5;
  padding: 4px 2px;
  flex-grow: 1;
  display: flex;
  align-items: flex-start;
}

@media (max-width: 768px) {
  .mode-card :deep(.n-card__content) {
    padding: 12px 16px;
  }
  
  .mode-card-header {
    margin-bottom: 12px;
  }
  
  .mode-buttons :deep(.n-button) {
    height: 36px;
  }
}
</style> 