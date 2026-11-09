<template>
  <div class="setting-section">
    <div class="kernel-status-row">
      <div class="kernel-version-badge" :class="versionBadgeClass">
        <n-icon :size="14">
          <CheckmarkCircleOutline v-if="isInstalled" />
          <AlertCircleOutline v-else />
        </n-icon>
        <span>{{ versionDisplay }}</span>
      </div>
      <div class="kernel-status-info">
        <div class="setting-label">
          {{
            isInstalled ? props.t('setting.kernel.title') : props.t('setting.kernel.installPrompt')
          }}
        </div>
        <div class="setting-desc">{{ installStateDesc }}</div>
      </div>
      <!-- 允许手动切换版本，但下载指定版本前由 SettingView 弹出兼容性强制确认 -->
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

    <!--
      安装态告警：把「未安装 / 文件被隔离 / 文件在但跑不起来」分开说，
      避免所有异常都退化成一句「内核未安装」（issue #70）。
    -->
    <n-alert
      v-if="installAlert"
      class="kernel-install-alert"
      :type="installAlert.tone"
      :title="installAlert.title"
    >
      <div class="install-alert-body">
        <div v-if="installAlert.hint" class="install-alert-hint">{{ installAlert.hint }}</div>
        <div v-if="installAlert.detail" class="install-alert-detail">{{ installAlert.detail }}</div>
      </div>
    </n-alert>

    <!-- 启动失败归因：配置非法 / 权限不足 / 端口占用 / 被安全软件拦截 -->
    <KernelDiagnosisCard class="setting-diagnosis" />

    <div v-if="props.hasNewVersion" class="setting-alert warning">
      <n-icon :size="16">
        <WarningOutline />
      </n-icon>
      <span>
        {{
          props.t('setting.update.newVersionFound', {
            version: props.kernelLatestVersion || props.t('setting.newVersionFound'),
          })
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
            isInstalled ? props.t('setting.kernel.redownload') : props.t('setting.kernel.download')
          }}
        </div>
      </div>
      <n-button
        :type="isInstalled ? 'default' : 'primary'"
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
            : isInstalled
              ? props.t('setting.kernel.redownload')
              : props.t('setting.kernel.download')
        }}
      </n-button>
    </div>

    <div class="kernel-sub-actions">
      <n-button
        text
        size="small"
        :disabled="props.downloading"
        @click="props.showManualDownloadModal"
      >
        <template #icon
          ><n-icon :size="14"><FolderOpenOutline /></n-icon
        ></template>
        {{ props.t('setting.kernel.manualDownload') }}
      </n-button>
      <span class="sub-divider"></span>
      <n-button text size="small" :disabled="props.downloading" @click="props.checkManualInstall">
        <template #icon
          ><n-icon :size="14"><RefreshOutline /></n-icon
        ></template>
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
import KernelDiagnosisCard from '@/components/kernel/KernelDiagnosisCard.vue'

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

/**
 * 「已安装」= 内核文件确实在磁盘上（文件层事实）。
 *
 * 此前这里用的是 `hasVersionInfo()`，而它读的是**运行中**内核的版本号——
 * 内核一停就变成空，界面于是永远显示「未安装」（issue #70）。
 */
const isInstalled = computed(() => props.kernelStore.isKernelInstalled)

const versionDisplay = computed(() =>
  isInstalled.value
    ? 'v' + props.formatVersion(props.kernelStore.getVersionString())
    : props.t('setting.notInstalled'),
)

const versionBadgeClass = computed(() => (isInstalled.value ? 'installed' : 'missing'))

/** 安装态说明行：告诉用户「装了但没跑」和「压根没装」的区别 */
const installStateDesc = computed(() => {
  if (!isInstalled.value) return props.t('setting.kernel.installState.missingHint')
  if (props.kernelStore.isRunning) return props.t('setting.kernel.installState.running')
  return props.t('setting.kernel.installState.installedNotRunning')
})

interface InstallAlert {
  tone: 'error' | 'warning' | 'info'
  title: string
  hint?: string
  detail?: string
}

/**
 * 安装态告警。
 *
 * - `file_lost`：文件曾经存在、现在没了 —— 典型是杀毒软件把它隔离了；
 * - `unreadable`：文件在，但执行不了 —— 同样常见于被安全软件拦截；
 * - `missing`：确实没装。
 */
const installAlert = computed<InstallAlert | null>(() => {
  const state = props.kernelStore.installState
  const detail = props.kernelStore.installation?.detail || undefined

  // 安装态未知时（还没探测出来）不报警，避免先闪一句「未检测到内核」
  if (!state) return null

  if (state === 'file_lost') {
    return {
      tone: 'error',
      title: props.t('setting.kernel.installState.fileLost'),
      hint: props.t('setting.kernel.installState.fileLostHint'),
    }
  }
  if (state === 'unreadable') {
    return {
      tone: 'error',
      title: props.t('setting.kernel.installState.unreadable'),
      hint: props.t('setting.kernel.installState.unreadableHint'),
      detail,
    }
  }
  if (state === 'missing') {
    return {
      tone: 'warning',
      title: props.t('setting.kernel.installState.missing'),
      hint: props.t('setting.kernel.installState.missingHint'),
    }
  }
  return null
})
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

.install-alert-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.kernel-install-alert {
  border-radius: var(--radius-sm);
  margin-bottom: var(--space-2);
}

.install-alert-hint {
  white-space: pre-wrap;
  word-break: break-word;
}

.install-alert-detail {
  white-space: pre-wrap;
  word-break: break-word;
  font-family: var(--font-mono, monospace);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.setting-diagnosis {
  margin-top: var(--space-3);
}
</style>
