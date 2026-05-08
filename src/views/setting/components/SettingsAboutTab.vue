<template>
  <div class="settings-panel">
    <div class="about-hero">
      <div class="about-logo-wrapper">
        <div class="about-logo">
          <n-icon :size="32"><LogoGithub /></n-icon>
        </div>
      </div>
      <div class="about-identity">
        <div class="about-name">{{ props.t('common.appName') }}</div>
        <div class="about-tagline">
          {{ props.t('setting.subtitle') }}
        </div>
      </div>
    </div>

    <div class="about-details">
      <div class="detail-row">
        <div class="detail-icon">
          <n-icon :size="16"><InformationCircleOutline /></n-icon>
        </div>
        <div class="detail-info">
          <div class="detail-label">{{ props.t('setting.appVersion') }}</div>
        </div>
        <div class="detail-value">v{{ props.updateStore.appVersion }}</div>
      </div>

      <div class="detail-row">
        <div class="detail-icon kernel">
          <n-icon :size="16"><HardwareChipOutline /></n-icon>
        </div>
        <div class="detail-info">
          <div class="detail-label">{{ props.t('setting.kernel.version') }}</div>
        </div>
        <div class="detail-value">
          {{
            props.kernelStore.hasVersionInfo()
              ? 'v' + props.formatVersion(props.kernelStore.getVersionString())
              : props.t('setting.notInstalled')
          }}
        </div>
      </div>

      <div class="detail-row">
        <div class="detail-icon platform">
          <n-icon :size="16"><DesktopOutline /></n-icon>
        </div>
        <div class="detail-info">
          <div class="detail-label">{{ props.t('setting.about.system') }}</div>
        </div>
        <div class="detail-value">{{ props.platformInfo?.display_name || props.t('common.loading') }}</div>
      </div>

      <div class="detail-row">
        <div class="detail-icon license">
          <n-icon :size="16"><ShieldCheckmarkOutline /></n-icon>
        </div>
        <div class="detail-info">
          <div class="detail-label">{{ props.t('setting.about.license') }}</div>
        </div>
        <div class="detail-value">MIT License</div>
      </div>
    </div>

    <div class="about-footer">
      <n-button
        text
        tag="a"
        href="https://github.com/xinggaoya/sing-box-windows"
        target="_blank"
        class="github-link"
      >
        <template #icon>
          <n-icon :size="18"><LogoGithub /></n-icon>
        </template>
        GitHub
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  InformationCircleOutline,
  LogoGithub,
  DesktopOutline,
  ShieldCheckmarkOutline,
  HardwareChipOutline,
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
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.about-hero {
  display: flex;
  align-items: center;
  gap: 18px;
  padding: 24px;
  border-radius: 14px;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.06), rgba(168, 85, 247, 0.04));
  border: 1px solid rgba(99, 102, 241, 0.1);
}

.about-logo-wrapper {
  flex-shrink: 0;
}

.about-logo {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  background: linear-gradient(135deg, var(--primary-color), var(--primary-hover));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.25);
}

.about-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
}

.about-tagline {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.about-details {
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  overflow: hidden;
  background: var(--bg-secondary);
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  transition: background 0.15s ease;
}

.detail-row:hover {
  background: var(--bg-tertiary);
}

.detail-row + .detail-row {
  border-top: 1px solid var(--panel-border);
}

.detail-icon {
  width: 30px;
  height: 30px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-tertiary);
  color: var(--text-secondary);
  flex-shrink: 0;
}

.detail-icon.kernel {
  color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.detail-icon.platform {
  color: #0ea5e9;
  background: rgba(14, 165, 233, 0.1);
}

.detail-icon.license {
  color: #f59e0b;
  background: rgba(245, 158, 11, 0.1);
}

.detail-info {
  flex: 1;
}

.detail-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.detail-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.about-footer {
  display: flex;
  justify-content: center;
  padding: 8px 0;
}

.github-link {
  font-size: 14px;
}
</style>
