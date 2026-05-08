<template>
  <div class="settings-panel">
    <div class="settings-group">
      <div class="group-header">
        <div class="group-icon update-icon">
          <n-icon :size="18"><RefreshOutline /></n-icon>
        </div>
        <div class="group-title-area">
          <div class="group-title">{{ props.t('setting.update.title') }}</div>
        </div>
      </div>

      <div class="group-body">
        <div class="update-summary">
          <div class="version-chip" :class="props.updateStore.hasUpdate ? 'has-update' : 'latest'">
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

          <div v-if="!props.updateStore.supportsInAppUpdate" class="platform-hint">
            {{ props.t('setting.update.externalUpdateHint') }}
          </div>

          <div class="update-card-actions">
            <n-button
              type="primary"
              strong
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
      </div>
    </div>

    <div class="settings-group">
      <div class="group-header">
        <div class="group-icon backup-icon">
          <n-icon :size="18"><ArchiveOutline /></n-icon>
        </div>
        <div class="group-title-area">
          <div class="group-title">{{ props.t('setting.backup.title') }}</div>
        </div>
      </div>

      <div class="group-body">
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
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  ArchiveOutline,
  DownloadOutline,
  OpenOutline,
  RefreshOutline,
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
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-group {
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  overflow: hidden;
  background: var(--bg-secondary);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  border-bottom: 1px solid var(--panel-border);
  background: var(--bg-tertiary);
}

.group-icon {
  width: 34px;
  height: 34px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.group-icon.update-icon {
  background: rgba(14, 165, 233, 0.12);
  color: #0ea5e9;
}

.group-icon.backup-icon {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
}

.group-title-area {
  flex: 1;
}

.group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.group-body {
  padding: 6px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 18px;
  transition: background 0.15s ease;
}

.setting-row:hover {
  background: var(--bg-tertiary);
}

.setting-row.top-align {
  align-items: flex-start;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.update-summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 18px;
  border-bottom: 1px solid var(--panel-border);
}

.version-chip {
  display: flex;
  align-items: center;
  gap: 10px;
}

.version-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.version-number {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
}

.update-card {
  margin: 12px 18px;
  padding: 14px;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.08), rgba(6, 182, 212, 0.06));
  border: 1px solid rgba(16, 185, 129, 0.15);
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.update-card-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.update-card-item {
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
}

.update-card-label {
  font-size: 11px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.04em;
  margin-bottom: 4px;
}

.update-card-value {
  font-size: 16px;
  font-weight: 700;
  color: var(--text-primary);
}

.update-card-value.highlight {
  color: #10b981;
}

.release-notes {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.release-notes-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.release-notes-content {
  max-height: 120px;
  overflow: auto;
  padding: 10px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.platform-hint {
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(99, 102, 241, 0.06);
  border: 1px solid rgba(99, 102, 241, 0.12);
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.5;
}

.update-card-actions {
  display: flex;
  gap: 8px;
}

.progress-section {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text-secondary);
}

.progress-pct {
  font-weight: 600;
  color: var(--primary-color);
}

.error-text {
  font-size: 13px;
  color: #ef4444;
}

.backup-btns {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.backup-preview {
  margin: 8px 18px;
  padding: 12px;
  border: 1px solid var(--panel-border);
  border-radius: 10px;
  background: var(--bg-tertiary);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.preview-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.preview-label {
  font-size: 12px;
  color: var(--text-tertiary);
}

.preview-path {
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
  text-align: right;
}

.preview-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.preview-row.warning .preview-value {
  color: #f59e0b;
}

.warning-list {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.warning-item {
  font-size: 12px;
  color: #f59e0b;
  line-height: 1.5;
}

.empty-hint {
  padding: 4px 18px 8px;
  font-size: 12px;
  color: var(--text-tertiary);
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
