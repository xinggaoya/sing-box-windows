<template>
  <div class="setting-section">
    <div class="about-hero">
      <div class="about-logo">
        <n-icon :size="28"><LogoGithub /></n-icon>
      </div>
      <div class="about-identity">
        <div class="about-name">{{ props.t('common.appName') }}</div>
        <div class="about-tagline">{{ props.t('setting.subtitle') }}</div>
      </div>
    </div>

    <div class="about-rows">
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.appVersion') }}</div>
        </div>
        <div class="setting-value">v{{ props.updateStore.appVersion }}</div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.kernel.version') }}</div>
        </div>
        <div class="setting-value">
          {{
            props.kernelStore.hasVersionInfo()
              ? 'v' + props.formatVersion(props.kernelStore.getVersionString())
              : props.t('setting.notInstalled')
          }}
        </div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.about.system') }}</div>
        </div>
        <div class="setting-value">{{ props.platformInfo?.display_name || props.t('common.loading') }}</div>
      </div>

      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.about.license') }}</div>
        </div>
        <div class="setting-value">MIT License</div>
      </div>
    </div>

    <div class="about-footer">
      <n-button
        text
        tag="a"
        href="https://github.com/xinggaoya/sing-box-windows"
        target="_blank"
      >
        <template #icon>
          <n-icon :size="16"><LogoGithub /></n-icon>
        </template>
        GitHub
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  LogoGithub,
} from '@vicons/ionicons5'
import type { useKernelStore, useUpdateStore } from '@/stores'

type KernelStoreLike = ReturnType<typeof useKernelStore>
type UpdateStoreLike = ReturnType<typeof useUpdateStore>

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  updateStore: UpdateStoreLike
  kernelStore: KernelStoreLike
  platformInfo: { os: string; arch: string; display_name: string } | null
  formatVersion: (value: string) => string
}>()
</script>

<style scoped>
.about-hero {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px 0;
  border-bottom: 1px solid var(--border-color);
}

.about-logo {
  width: 48px;
  height: 48px;
  border-radius: 14px;
  background: linear-gradient(135deg, var(--primary-color), var(--primary-hover));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.25);
}

.about-name {
  font-size: 18px;
  font-weight: 700;
  color: var(--text-primary);
}

.about-tagline {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.about-rows {
  display: flex;
  flex-direction: column;
}

.setting-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.about-footer {
  display: flex;
  justify-content: center;
  padding: 16px 0 8px;
}
</style>
