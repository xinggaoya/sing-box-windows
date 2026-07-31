<template>
  <div class="setting-section">
    <div class="kernel-status-row">
      <div class="kernel-version-badge" :class="versionBadgeClass">
        <n-icon :size="14">
          <CheckmarkCircleOutline v-if="props.kernelStore.hasVersionInfo()" />
          <AlertCircleOutline v-else />
        </n-icon>
        <span>{{ versionDisplay }}</span>
      </div>
      <div class="kernel-status-info">
        <div class="setting-label">
          {{
            props.kernelStore.hasVersionInfo()
              ? props.t('setting.kernel.title')
              : props.t('setting.kernel.installPrompt')
          }}
        </div>
        <div class="setting-desc">{{ props.t('setting.kernel.embeddedHint') }}</div>
      </div>
      <n-select
        :value="props.selectedKernelVersion"
        :options="props.kernelVersionOptions"
        :loading="props.kernelStore.isLoading"
        :disabled="props.downloading"
        size="small"
        style="width: 160px"
        placeholder="Latest"
        @update:value="props.onSelectedKernelVersionChange"
      />
    </div>

    <div
      v-if="props.hasNewVersion || !props.kernelStore.hasVersionInfo()"
      class="setting-alert"
      :class="props.hasNewVersion ? 'warning' : 'install'"
    >
      <n-icon :size="16">
        <WarningOutline v-if="props.hasNewVersion" />
        <DownloadOutline v-else />
      </n-icon>
      <span>
        {{
          props.hasNewVersion
            ? props.t('setting.update.newVersionFound', {
                version: props.kernelLatestVersion || props.t('setting.newVersionFound'),
              })
            : props.t('setting.kernel.installPrompt')
        }}
      </span>
    </div>

    <div v-if="props.downloading" class="download-progress-card">
      <div class="progress-header">
        <span class="progress-label">{{ props.t('setting.kernel.downloading') }}</span>
        <span class="progress-value">{{ props.downloadProgress.toFixed(0) }}%</span>
      </div>
      <n-progress
        type="line"
        :percentage="props.downloadProgress"
        :processing="props.downloadProgress < 100"
        indicator-placement="inside"
      />
      <div v-if="props.downloadMessage" class="progress-msg">{{ props.downloadMessage }}</div>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">
          {{
            props.kernelStore.hasVersionInfo()
              ? props.t('setting.kernel.redownload')
              : props.t('setting.kernel.download')
          }}
        </div>
      </div>
      <n-button
        :type="props.kernelStore.hasVersionInfo() ? 'default' : 'primary'"
        :loading="props.loading"
        :disabled="props.downloading"
        @click="props.downloadTheKernel"
      >
        <template #icon>
          <n-icon><DownloadOutline /></n-icon>
        </template>
        {{
          props.hasNewVersion
            ? props.t('setting.kernel.update')
            : props.kernelStore.hasVersionInfo()
              ? props.t('setting.kernel.redownload')
              : props.t('setting.kernel.download')
        }}
      </n-button>
    </div>

    <div class="kernel-sub-actions">
      <n-button text size="small" :disabled="props.downloading" @click="props.showManualDownloadModal">
        <template #icon><n-icon :size="14"><FolderOpenOutline /></n-icon></template>
        {{ props.t('setting.kernel.manualDownload') }}
      </n-button>
      <span class="sub-divider"></span>
      <n-button text size="small" :disabled="props.downloading" @click="props.checkManualInstall">
        <template #icon><n-icon :size="14"><RefreshOutline /></n-icon></template>
        {{ props.t('setting.kernel.checkInstall') }}
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  DownloadOutline,
  FolderOpenOutline,
  RefreshOutline,
  WarningOutline,
  CheckmarkCircleOutline,
  AlertCircleOutline,
} from '@vicons/ionicons5'
import type { useKernelStore } from '@/stores'

type KernelStoreLike = ReturnType<typeof useKernelStore>

interface Option {
  label: string
  value: string | undefined
}

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  kernelStore: KernelStoreLike
  selectedKernelVersion?: string
  kernelVersionOptions: Option[]
  hasNewVersion: boolean
  kernelLatestVersion: string
  downloading: boolean
  loading: boolean
  downloadProgress: number
  downloadMessage: string
  onSelectedKernelVersionChange: (value: string | undefined) => void
  downloadTheKernel: () => void | Promise<void>
  showManualDownloadModal: () => void
  checkManualInstall: () => void | Promise<void>
  formatVersion: (value: string) => string
}>()

const versionDisplay = computed(() =>
  props.kernelStore.hasVersionInfo()
    ? 'v' + props.formatVersion(props.kernelStore.getVersionString())
    : props.t('setting.notInstalled'),
)

const versionBadgeClass = computed(() =>
  props.kernelStore.hasVersionInfo() ? 'installed' : 'missing',
)
</script>

<style scoped>
.kernel-status-row {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-4) 0;
}

.kernel-version-badge {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: var(--radius-sm);
  font-size: var(--text-sm);
  font-weight: 700;
  flex-shrink: 0;
}

.kernel-version-badge.installed {
  background: var(--success-soft);
  color: var(--success-color);
}

.kernel-version-badge.missing {
  background: var(--error-soft);
  color: var(--error-color);
}

.kernel-status-info {
  flex: 1;
  min-width: 0;
}

.download-progress-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: var(--bg-surface-2);
  border: 1px solid var(--panel-border);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-label {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-secondary);
}

.progress-value {
  font-size: var(--text-sm);
  font-weight: 700;
  color: var(--primary-color);
}

.progress-msg {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-align: center;
}

.kernel-sub-actions {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-1) 0 var(--space-2);
}

.sub-divider {
  width: 3px;
  height: 3px;
  border-radius: var(--radius-pill);
  background: var(--text-tertiary);
}
</style>
