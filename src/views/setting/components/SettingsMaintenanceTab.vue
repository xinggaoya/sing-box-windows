<template>
  <div class="setting-section">
    <h3 class="setting-section-title">{{ props.t('setting.update.title') }}</h3>

    <div class="setting-row">
      <div class="setting-info">
        <div class="version-inline">
          <span class="version-label">{{ props.t('setting.update.currentVersion') }}</span>
          <span class="version-number">v{{ props.updateStore.appVersion }}</span>
          <n-tag
            v-if="props.updateStore.hasUpdate"
            type="warning"
            size="small"
            round
            :bordered="false"
          >
            {{ props.t('setting.update.hasUpdate') }}
          </n-tag>
          <n-tag v-else type="success" size="small" round :bordered="false">
            {{ props.t('setting.update.latest') }}
          </n-tag>
        </div>
      </div>
      <n-button
        size="small"
        secondary
        :loading="props.checkingUpdate"
        :disabled="props.updateStore.isChecking"
        @click="props.handleCheckUpdate"
      >
        {{ props.checkingUpdate ? props.t('setting.update.checking') : props.t('setting.update.checkNow') }}
      </n-button>
    </div>

    <div v-if="props.updateStore.hasUpdate" class="update-card">
      <div class="update-card-row">
        <div class="update-card-item">
          <div class="update-card-label">{{ props.t('setting.update.newVersion') }}</div>
          <div class="update-card-value highlight">v{{ props.updateStore.latestVersion }}</div>
        </div>
        <div class="update-card-item">
          <div class="update-card-label">{{ props.t('setting.update.currentVersion') }}</div>
          <div class="update-card-value">v{{ props.updateStore.appVersion }}</div>
        </div>
      </div>

      <div v-if="props.updateStore.releaseNotes" class="release-notes">
        <div class="release-notes-label">{{ props.t('setting.update.releaseNotes') }}</div>
        <div class="release-notes-content">{{ props.updateStore.releaseNotes }}</div>
      </div>

      <div v-if="!props.updateStore.supportsInAppUpdate" class="setting-alert info">
        {{ props.t('setting.update.externalUpdateHint') }}
      </div>

      <div class="update-card-actions">
        <n-button
          type="primary"
          :loading="props.updateStore.supportsInAppUpdate && props.isUpdating"
          :disabled="
            props.updateStore.supportsInAppUpdate
              ? props.isUpdating
              : !props.updateStore.canOpenReleasePage
          "
          @click="props.handleUpdateNow"
        >
          <template #icon>
            <n-icon>
              <OpenOutline v-if="!props.updateStore.supportsInAppUpdate" />
              <DownloadOutline v-else />
            </n-icon>
          </template>
          {{
            !props.updateStore.supportsInAppUpdate
              ? props.t('setting.update.openReleasePage')
              : props.updateStatus === 'installing'
                ? props.t('setting.update.installing')
                : props.isUpdating
                  ? props.t('setting.update.downloading')
                  : props.t('setting.update.updateNow')
          }}
        </n-button>
      </div>

      <div
        v-if="props.updateStore.supportsInAppUpdate && props.showUpdateProgress"
        class="progress-section"
      >
        <div class="progress-header">
          <span>{{ props.updateMessage || props.t('setting.update.downloading') }}</span>
          <span class="progress-pct">{{ props.updateProgress.toFixed(0) }}%</span>
        </div>
        <n-progress
          type="line"
          :percentage="props.updateProgress"
          :processing="props.updateStatus === 'downloading'"
          :status="props.updateStatus === 'error' ? 'error' : 'default'"
          :show-indicator="false"
        />
      </div>

      <div
        v-else-if="props.updateStore.supportsInAppUpdate && props.updateStatus === 'error'"
        class="error-text"
      >
        {{ props.updateMessage || props.t('setting.update.updateFailed') }}
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.update.autoCheck') }}</div>
      </div>
      <n-switch
        :value="props.updateStore.autoCheckUpdate"
        @update:value="props.onAutoCheckUpdateChange"
      />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.update.channel') }}</div>
      </div>
      <n-select
        :value="props.updateStore.updateChannel"
        :options="props.updateChannelOptions"
        size="small"
        style="width: 160px"
        @update:value="props.onUpdateChannelChange"
      />
    </div>

    <h3 class="setting-section-title">{{ props.t('setting.backup.title') }}</h3>

    <div class="setting-row top-align">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.backup.description') }}</div>
        <div class="setting-desc">{{ props.t('setting.backup.restoreHint') }}</div>
      </div>
      <div class="backup-btns">
        <n-button
          size="small"
          secondary
          :loading="props.backupExporting"
          :disabled="props.backupBusy"
          @click="props.handleExportBackup"
        >
          {{ props.t('setting.backup.exportAction') }}
        </n-button>
        <n-button
          size="small"
          secondary
          :loading="props.backupValidating"
          :disabled="props.backupBusy"
          @click="props.handleValidateBackup"
        >
          {{ props.t('setting.backup.validateAction') }}
        </n-button>
        <n-button
          size="small"
          type="warning"
          :loading="props.backupRestoring"
          :disabled="props.backupBusy"
          @click="props.handleRestoreBackup"
        >
          {{ props.t('setting.backup.restoreAction') }}
        </n-button>
      </div>
    </div>

    <div v-if="props.backupPreview" class="backup-preview">
      <div class="preview-row">
        <span class="preview-label">{{ props.t('setting.backup.selectedFile') }}</span>
        <span class="preview-path">{{ props.backupPreview.file_path }}</span>
      </div>
      <div class="preview-row">
        <span class="preview-label">{{ props.t('setting.backup.subscriptionCount') }}</span>
        <span class="preview-value">{{ props.backupPreview.subscriptions_count }}</span>
      </div>
      <div
        class="preview-row"
        :class="{ warning: props.backupPreview.warnings.length > 0 }"
      >
        <span class="preview-label">{{ props.t('setting.backup.warningCount') }}</span>
        <span class="preview-value">{{ props.backupPreview.warnings.length }}</span>
      </div>
      <div v-if="props.backupPreview.warnings.length > 0" class="warning-list">
        <div v-for="(w, i) in props.backupPreview.warnings" :key="i" class="warning-item">
          {{ w }}
        </div>
      </div>
    </div>
    <div v-else class="empty-hint">
      {{ props.t('setting.backup.noPreview') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  DownloadOutline,
  OpenOutline,
} from '@vicons/ionicons5'
import type { UpdateChannel } from '@/stores/app/UpdateStore'
import type { useUpdateStore } from '@/stores'
import type { BackupImportResult } from '@/services/system-service'

type UpdateStoreLike = ReturnType<typeof useUpdateStore>

interface Option<T extends string = string> {
  label: string
  value: T
}

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  updateStore: UpdateStoreLike
  checkingUpdate: boolean
  updateStatus: string
  updateProgress: number
  updateMessage: string
  isUpdating: boolean
  showUpdateProgress: boolean
  updateChannelOptions: Option<UpdateChannel>[]
  backupExporting: boolean
  backupValidating: boolean
  backupRestoring: boolean
  backupBusy: boolean
  backupPreview: BackupImportResult | null
  handleUpdateNow: () => void | Promise<void>
  handleCheckUpdate: () => void | Promise<void>
  onAutoCheckUpdateChange: (value: boolean) => void
  onUpdateChannelChange: (value: UpdateChannel) => void | Promise<void>
  handleExportBackup: () => void | Promise<void>
  handleValidateBackup: () => void | Promise<void>
  handleRestoreBackup: () => void | Promise<void>
}>()
</script>

<style scoped>
.version-inline {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.version-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.version-number {
  font-size: var(--text-base);
  font-weight: 700;
  color: var(--text-primary);
}

.update-card {
  margin: var(--space-1) 0;
  padding: var(--space-4);
  border-radius: var(--radius-md);
  background: linear-gradient(135deg, var(--success-soft), var(--info-soft));
  border: 1px solid rgba(16, 185, 129, 0.18);
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.update-card-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
}

.update-card-item {
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-overlay);
}

.update-card-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: var(--space-1);
}

.update-card-value {
  font-size: var(--text-md);
  font-weight: 700;
  color: var(--text-primary);
}

.update-card-value.highlight {
  color: var(--success-color);
}

.release-notes {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.release-notes-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.release-notes-content {
  max-height: 120px;
  overflow: auto;
  padding: var(--space-3);
  border-radius: var(--radius-sm);
  background: var(--bg-overlay);
  font-size: var(--text-xs);
  color: var(--text-secondary);
  line-height: 1.5;
}

.update-card-actions {
  display: flex;
  gap: var(--space-2);
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.progress-pct {
  font-weight: 600;
  color: var(--primary-color);
}

.error-text {
  font-size: var(--text-sm);
  color: var(--error-color);
}

.backup-btns {
  display: flex;
  gap: var(--space-2);
  flex-shrink: 0;
}

.backup-preview {
  padding: var(--space-3);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-md);
  background: var(--bg-surface-2);
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-bottom: var(--space-2);
}

.preview-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.preview-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.preview-path {
  font-size: var(--text-xs);
  color: var(--text-primary);
  word-break: break-all;
  text-align: right;
  font-family: var(--font-mono);
}

.preview-value {
  font-size: var(--text-base);
  font-weight: 600;
  color: var(--text-primary);
}

.preview-row.warning .preview-value {
  color: var(--warning-color);
}

.warning-list {
  margin-top: var(--space-1);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.warning-item {
  font-size: var(--text-xs);
  color: var(--warning-color);
  line-height: 1.5;
}

.empty-hint {
  padding: var(--space-1) 0 var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.setting-row.top-align {
  align-items: flex-start;
}

@media (max-width: 768px) {
  .backup-btns {
    flex-direction: column;
  }

  .update-card-row {
    grid-template-columns: 1fr;
  }
}
</style>
