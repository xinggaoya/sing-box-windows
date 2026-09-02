<template>
  <div class="page-shell settings-page">
    <div class="settings-tabs">
      <button
        v-for="item in navItems"
        :key="item.key"
        class="settings-tab"
        :class="{ active: activeTab === item.key }"
        @click="activeTab = item.key"
      >
        <n-icon :size="16"><component :is="item.icon" /></n-icon>
        <span class="settings-tab-label">{{ item.label }}</span>
        <span v-if="item.key === 'kernel' && hasNewVersion" class="tab-dot"></span>
        <span v-if="item.key === 'maintenance' && updateStore.hasUpdate" class="tab-dot"></span>
      </button>
    </div>

    <div class="settings-body">
      <transition name="section-fade" mode="out-in">
        <div :key="activeTab" class="settings-content">
          <SettingsBasicTab
            v-if="activeTab === 'basics'"
            :t="t"
            :locale-store="localeStore"
            :theme-store="themeStore"
            :auto-start="autoStart"
            :auto-hide-to-tray-on-autostart="autoHideToTrayOnAutostart"
            :tray-close-behavior="trayCloseBehavior"
            :language-options="languageOptions"
            :tray-close-behavior-options="trayCloseBehaviorOptions"
            :accent-presets="accentPresets"
            :on-auto-start-change="onAutoStartChange"
            :on-auto-hide-to-tray-on-autostart-change="onAutoHideToTrayOnAutostartChange"
            :on-tray-close-behavior-change="onTrayCloseBehaviorChange"
            :on-change-language="handleChangeLanguage"
            :on-theme-mode-change="onThemeModeChange"
            :on-accent-change="onAccentChange"
            :select-accent-preset="selectAccentPreset"
            :on-compact-mode-change="onCompactModeChange"
          />

          <SettingsKernelTab
            v-if="activeTab === 'kernel'"
            :t="t"
            :kernel-store="kernelStore"
            :selected-kernel-version="selectedKernelVersion"
            :kernel-version-options="kernelVersionOptions"
            :has-new-version="hasNewVersion"
            :kernel-latest-version="kernelLatestVersion"
            :downloading="downloading"
            :loading="loading"
            :download-progress="downloadProgress"
            :download-message="downloadMessage"
            :on-selected-kernel-version-change="onSelectedKernelVersionChange"
            :download-the-kernel="downloadTheKernel"
            :show-manual-download-modal="showManualDownloadModal"
            :check-manual-install="checkManualInstall"
            :format-version="formatVersion"
          />

          <SettingsAdvancedTab
            v-if="activeTab === 'advanced'"
            :t="t"
            :app-store="appStore"
            :tun-stack-options="tunStackOptions"
            :using-original-config="usingOriginalConfig"
            :on-ip-version-change="onIpVersionChange"
            :on-lan-access-change="onLanAccessChange"
            :show-port-settings="showPortSettings"
          />

          <SettingsMaintenanceTab
            v-if="activeTab === 'maintenance'"
            :t="t"
            :update-store="updateStore"
            :checking-update="checkingUpdate"
            :update-status="updateStatus"
            :update-progress="updateProgress"
            :update-message="updateMessage"
            :is-updating="isUpdating"
            :show-update-progress="showUpdateProgress"
            :update-channel-options="updateChannelOptions"
            :backup-exporting="backupExporting"
            :backup-validating="backupValidating"
            :backup-restoring="backupRestoring"
            :backup-busy="backupBusy"
            :backup-preview="backupPreview"
            :handle-update-now="handleUpdateNow"
            :handle-check-update="handleCheckUpdate"
            :on-auto-check-update-change="onAutoCheckUpdateChange"
            :on-update-channel-change="onUpdateChannelChange"
            :handle-export-backup="handleExportBackup"
            :handle-validate-backup="handleValidateBackup"
            :handle-restore-backup="handleRestoreBackup"
          />

          <SettingsAboutTab
            v-if="activeTab === 'about'"
            :t="t"
            :update-store="updateStore"
            :kernel-store="kernelStore"
            :platform-info="platformInfo"
            :format-version="formatVersion"
          />
        </div>
      </transition>
    </div>

    <PortSettingsDialog v-model:show="showPortModal" />

    <n-modal
      v-model:show="showManualImportModal"
      preset="dialog"
      :title="t('setting.kernel.manualImportTitle')"
      class="modern-modal"
      :style="{ width: '520px' }"
    >
      <div class="manual-import-body">
        <div class="manual-import-desc">{{ t('setting.kernel.manualImportDesc') }}</div>
        <div class="manual-drop-zone" :class="{ active: manualDropActive }">
          <n-icon size="24"><DownloadOutline /></n-icon>
          <div>{{ t('setting.kernel.dropHint') }}</div>
          <div class="manual-drop-sub">{{ t('setting.kernel.dropSubHint') }}</div>
        </div>

        <div v-if="manualKernelPath" class="manual-selected">
          <div class="manual-selected-label">{{ t('setting.kernel.selectedFile') }}</div>
          <div class="manual-selected-path">{{ manualKernelPath }}</div>
        </div>
      </div>
      <template #action>
        <n-space justify="space-between" style="width: 100%">
          <n-button @click="pickManualKernelFile" :disabled="manualImporting">
            {{ t('setting.kernel.chooseFile') }}
          </n-button>
          <n-space>
            <n-button @click="showManualImportModal = false" :disabled="manualImporting">
              {{ t('common.cancel') }}
            </n-button>
            <n-button type="primary" @click="importManualKernel" :loading="manualImporting">
              {{ t('setting.kernel.importNow') }}
            </n-button>
          </n-space>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, type Component } from 'vue'
import { Window } from '@tauri-apps/api/window'
import { useDialog, useMessage } from 'naive-ui'
import {
  DownloadOutline,
  InformationCircleOutline,
  ColorPaletteOutline,
  HardwareChipOutline,
  GlobeOutline,
  CloudDownloadOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import {
  useAppStore,
  useKernelStore,
  useUpdateStore,
  useLocaleStore,
  useThemeStore,
} from '@/stores'
import { useSubStore } from '@/stores/subscription/SubStore'
import type { Locale } from '@/stores/app/LocaleStore'
import type { ThemeMode } from '@/stores/app/ThemeStore'
import type { TrayCloseBehavior } from '@/stores/app/AppStore'
import type { UpdateChannel } from '@/stores/app/UpdateStore'
import { systemService, type BackupImportResult } from '@/services/system-service'
import { supportedLocales } from '@/locales'
import PortSettingsDialog from '@/components/common/PortSettingsDialog.vue'
import { ACCENT_PRESETS, TUN_STACK_OPTIONS } from '@/views/setting/setting-options'
import { useKernelDownload } from '@/views/setting/useKernelDownload'
import { useUpdateProgressListener } from '@/views/setting/useUpdateProgressListener'
import SettingsBasicTab from '@/views/setting/components/SettingsBasicTab.vue'
import SettingsKernelTab from '@/views/setting/components/SettingsKernelTab.vue'
import SettingsAdvancedTab from '@/views/setting/components/SettingsAdvancedTab.vue'
import SettingsMaintenanceTab from '@/views/setting/components/SettingsMaintenanceTab.vue'
import SettingsAboutTab from '@/views/setting/components/SettingsAboutTab.vue'
import '@/views/setting/setting-shared.css'

const message = useMessage()
const dialog = useDialog()
const { t } = useI18n()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const themeStore = useThemeStore()
const subStore = useSubStore()

type SettingTabKey = 'basics' | 'kernel' | 'advanced' | 'maintenance' | 'about'

const activeTab = ref<SettingTabKey>('basics')
const selectedKernelVersion = ref<string | undefined>(undefined)
const platformInfo = ref<{ os: string; arch: string; display_name: string } | null>(null)

const autoStart = ref(false)
const autoHideToTrayOnAutostart = ref(true)
const trayCloseBehavior = ref<TrayCloseBehavior>('hide')
const checkingUpdate = ref(false)
const backupExporting = ref(false)
const backupValidating = ref(false)
const backupRestoring = ref(false)
const backupPreview = ref<BackupImportResult | null>(null)
const accentPresets = ACCENT_PRESETS

const showPortModal = ref(false)
const showManualImportModal = ref(false)
const manualImporting = ref(false)
const manualKernelPath = ref('')
const manualDropActive = ref(false)
let manualDropUnlisten: (() => void) | null = null

interface NavItem {
  key: SettingTabKey
  label: string
  icon: Component
}

const navItems = computed<NavItem[]>(() => [
  {
    key: 'basics',
    label: t('setting.navigation.basics'),
    icon: ColorPaletteOutline,
  },
  {
    key: 'kernel',
    label: t('setting.navigation.kernel'),
    icon: HardwareChipOutline,
  },
  {
    key: 'advanced',
    label: t('setting.navigation.advanced'),
    icon: GlobeOutline,
  },
  {
    key: 'maintenance',
    label: t('setting.navigation.maintenance'),
    icon: CloudDownloadOutline,
  },
  {
    key: 'about',
    label: t('setting.navigation.about'),
    icon: InformationCircleOutline,
  },
])

const languageOptions = computed<{ label: string; value: Locale }[]>(() => [
  { label: t('setting.language.auto'), value: 'auto' },
  ...supportedLocales.map((item) => ({
    label: item.name,
    value: item.code as Locale,
  })),
])
const updateChannelOptions = computed<{ label: string; value: UpdateChannel }[]>(() => [
  { label: t('setting.update.channelStable'), value: 'stable' },
  { label: t('setting.update.channelPrerelease'), value: 'prerelease' },
  { label: t('setting.update.channelAutobuild'), value: 'autobuild' },
])
const trayCloseBehaviorOptions = computed<{ label: string; value: TrayCloseBehavior }[]>(() => [
  { label: t('setting.startup.closeBehaviorHide'), value: 'hide' },
  { label: t('setting.startup.closeBehaviorLightweight'), value: 'lightweight' },
])

const tunStackOptions = TUN_STACK_OPTIONS

const kernelLatestVersion = computed(() => kernelStore.latestAvailableVersion || '')
const activeSubscription = computed(() => subStore.getActiveSubscription())
const usingOriginalConfig = computed(() => activeSubscription.value?.useOriginalConfig ?? false)
const hasNewVersion = computed(() => kernelStore.hasKernelUpdate)
const kernelVersionOptions = computed(() => {
  const versions = kernelStore.availableVersions || []
  return [
    { label: t('setting.kernel.latest'), value: undefined },
    ...versions.map((v) => ({ label: v, value: v })),
  ]
})
const updateStatus = computed(() => updateStore.updateState.status)
const updateProgress = computed(() => updateStore.updateState.progress || 0)
const updateMessage = computed(() => updateStore.updateState.message)
const isUpdating = computed(() => ['downloading', 'installing'].includes(updateStatus.value))
const showUpdateProgress = computed(
  () =>
    ['downloading', 'installing', 'completed'].includes(updateStatus.value) ||
    updateProgress.value > 0,
)
const backupBusy = computed(
  () => backupExporting.value || backupValidating.value || backupRestoring.value,
)

const {
  loading,
  downloading,
  downloadProgress,
  downloadMessage,
  downloadTheKernel,
  cleanupDownloadListener,
} = useKernelDownload({
  selectedVersion: selectedKernelVersion,
  message,
  t,
  checkKernelInstallation: () => kernelStore.checkKernelInstallation(),
})

const { setupUpdateProgressListener, cleanupUpdateProgressListener } = useUpdateProgressListener({
  message,
  updateStore,
  t,
})

const formatVersion = (v: string) => v.replace(/^v/, '')
const isSupportedLocale = (l: string) => languageOptions.value.some((opt) => opt.value === l)
const onSelectedKernelVersionChange = (value: string | undefined) => {
  selectedKernelVersion.value = value
}

watch(showManualImportModal, (visible) => {
  if (!visible) {
    manualDropActive.value = false
  }
})

const onAutoStartChange = async (value: boolean) => {
  try {
    await appStore.toggleAutoStart(value)
    message.success(t('common.saveSuccess'))
  } catch {
    message.error(t('common.saveFailed'))
    autoStart.value = !value
  }
}

const onTrayCloseBehaviorChange = async (value: TrayCloseBehavior) => {
  const previous = appStore.trayCloseBehavior
  try {
    await appStore.setTrayCloseBehavior(value)
    trayCloseBehavior.value = value
    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('保存关闭到托盘行为失败:', error)
    trayCloseBehavior.value = previous
    message.error(t('common.saveFailed'))
  }
}

const onAutoHideToTrayOnAutostartChange = async (value: boolean) => {
  const previous = appStore.autoHideToTrayOnAutostart
  try {
    await appStore.setAutoHideToTrayOnAutostart(value)
    autoHideToTrayOnAutostart.value = value
    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('保存开机后自动隐藏窗口到托盘设置失败:', error)
    autoHideToTrayOnAutostart.value = previous
    message.error(t('common.saveFailed'))
  }
}
const handleChangeLanguage = async (value: string) => {
  if (!isSupportedLocale(value)) {
    console.warn('选择了不受支持的语言:', value)
    return
  }

  const nextLocale = value === 'auto' ? 'auto' : value
  await localeStore.setLocale(nextLocale as Locale)
}

const onIpVersionChange = async (value: boolean) => {
  try {
    await appStore.setPreferIpv6(value)
    await appStore.saveToBackend()

    const toggled = await kernelStore.toggleIpVersion(value)
    if (!toggled) {
      throw new Error(kernelStore.lastError || t('notification.ipVersionChangeFailed'))
    }

    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('切换IPv6优先失败:', error)
    message.error(t('notification.proxyModeChangeFailed'))
  }
}

const onLanAccessChange = async (value: boolean) => {
  const previous = appStore.allowLanAccess
  appStore.allowLanAccess = value

  try {
    await appStore.saveToBackend({ applyRuntime: true })
    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('切换局域网访问失败:', error)
    appStore.allowLanAccess = previous
    message.error(t('common.saveFailed'))
  }
}

const onThemeModeChange = async (value: ThemeMode) => {
  await themeStore.setThemeMode(value)
}

const onAccentChange = async (value: string) => {
  await themeStore.setAccentColor(value)
}

const selectAccentPreset = async (color: string) => {
  await themeStore.setAccentColor(color)
}

const onCompactModeChange = async (value: boolean) => {
  await themeStore.setCompactMode(value)
}

const setupManualDropListener = async () => {
  if (manualDropUnlisten) return

  const appWindow = Window.getCurrent()
  manualDropUnlisten = await appWindow.onDragDropEvent((event) => {
    if (!showManualImportModal.value || manualImporting.value) return

    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      manualDropActive.value = true
      return
    }

    if (event.payload.type === 'leave') {
      manualDropActive.value = false
      return
    }

    if (event.payload.type === 'drop') {
      manualDropActive.value = false
      if (event.payload.paths.length > 0) {
        manualKernelPath.value = event.payload.paths[0]
      }
    }
  })
}

const cleanupManualDropListener = () => {
  if (manualDropUnlisten) {
    manualDropUnlisten()
    manualDropUnlisten = null
  }
}

const showManualDownloadModal = () => {
  manualKernelPath.value = ''
  showManualImportModal.value = true
}

const pickManualKernelFile = async () => {
  try {
    const selected = await systemService.pickKernelImportFile()
    if (selected) {
      manualKernelPath.value = selected
    }
  } catch (error) {
    console.error('选择内核文件失败:', error)
    message.error(t('setting.kernel.pickFailed'))
  }
}

const importManualKernel = async () => {
  if (!manualKernelPath.value) {
    message.warning(t('setting.kernel.noFileSelected'))
    return
  }

  manualImporting.value = true
  try {
    const result = await systemService.importKernelExecutable(manualKernelPath.value)
    message.success(result.message || t('common.saveSuccess'))
    showManualImportModal.value = false
    manualKernelPath.value = ''
    await kernelStore.checkKernelInstallation()
  } catch (error) {
    console.error('导入内核失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.kernel.importFailed')
    message.error(errMsg)
  } finally {
    manualImporting.value = false
  }
}

const checkManualInstall = async () => {
  await kernelStore.checkKernelInstallation()
}

const handleUpdateNow = async () => {
  if (!updateStore.hasUpdate) {
    message.info(t('setting.update.alreadyLatest'))
    return
  }

  try {
    if (!updateStore.supportsInAppUpdate) {
      await updateStore.openReleasePage()
      return
    }

    updateStore.updateProgress('downloading', 0, t('setting.update.preparingDownload'))
    await updateStore.downloadAndInstallUpdate()
  } catch (error) {
    console.error('执行更新操作失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.update.updateFailed')
    message.error(errMsg)
  }
}

const handleCheckUpdate = async () => {
  checkingUpdate.value = true
  try {
    await updateStore.checkUpdate()
  } finally {
    checkingUpdate.value = false
  }
}

const onAutoCheckUpdateChange = (value: boolean) => {
  updateStore.autoCheckUpdate = value
}

const onUpdateChannelChange = async (value: UpdateChannel) => {
  const previous = updateStore.updateChannel
  try {
    await updateStore.setUpdateChannel(value)
  } catch (error) {
    console.error('保存更新通道设置失败:', error)
    message.error(t('common.saveFailed'))
    updateStore.updateChannel = previous
  }
}

const reloadStoresAfterBackupRestore = async () => {
  await Promise.all([
    appStore.loadFromBackend(),
    themeStore.loadFromBackend(),
    localeStore.loadFromBackend(),
    updateStore.loadFromBackend(),
    subStore.loadFromBackend(),
  ])
  autoStart.value = appStore.autoStartApp
}

const handleExportBackup = async () => {
  backupExporting.value = true
  try {
    const result = await systemService.backupExportSnapshot()
    message.success(t('setting.backup.exportSuccess', { path: result.file_path }))
  } catch (error) {
    console.error('导出备份失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.backup.operationFailed')
    message.error(errMsg)
  } finally {
    backupExporting.value = false
  }
}

const handleValidateBackup = async () => {
  backupValidating.value = true
  try {
    const result = await systemService.backupImportSnapshot({ dryRun: true })
    backupPreview.value = result
    if (result.warnings.length > 0) {
      message.warning(t('setting.backup.validateWithWarnings', { count: result.warnings.length }))
    } else {
      message.success(t('setting.backup.validateSuccess', { count: result.subscriptions_count }))
    }
  } catch (error) {
    console.error('预检备份失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.backup.operationFailed')
    message.error(errMsg)
  } finally {
    backupValidating.value = false
  }
}

const confirmRestoreBackup = () => {
  let resolved = false

  return new Promise<boolean>((resolve) => {
    const finish = (result: boolean) => {
      if (resolved) return
      resolved = true
      resolve(result)
    }

    dialog.warning({
      title: t('setting.backup.restoreAction'),
      content: t('setting.backup.restoreConfirm'),
      positiveText: t('common.confirm'),
      negativeText: t('common.cancel'),
      maskClosable: false,
      onPositiveClick: () => finish(true),
      onNegativeClick: () => finish(false),
      onClose: () => finish(false),
    })
  })
}

const handleRestoreBackup = async () => {
  const confirmed = await confirmRestoreBackup()
  if (!confirmed) {
    return
  }

  backupRestoring.value = true
  try {
    const result = await systemService.backupImportSnapshot({
      filePath: backupPreview.value?.file_path,
      dryRun: false,
    })
    backupPreview.value = result
    await reloadStoresAfterBackupRestore()
    if (result.warnings.length > 0) {
      message.warning(t('setting.backup.restoreWithWarnings', { count: result.warnings.length }))
    } else {
      message.success(t('setting.backup.restoreSuccess', { count: result.subscriptions_count }))
    }
  } catch (error) {
    console.error('恢复备份失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.backup.operationFailed')
    message.error(errMsg)
  } finally {
    backupRestoring.value = false
  }
}

const showPortSettings = () => {
  showPortModal.value = true
}

onMounted(async () => {
  try {
    await setupManualDropListener()
  } catch (error) {
    console.error('初始化手动导入拖放监听失败:', error)
  }

  await appStore.waitForDataRestore()
  await appStore.syncAutoStartWithSystem()
  autoStart.value = appStore.autoStartApp
  autoHideToTrayOnAutostart.value = appStore.autoHideToTrayOnAutostart
  trayCloseBehavior.value = appStore.trayCloseBehavior
  watch(
    () => appStore.autoStartApp,
    (enabled) => {
      autoStart.value = enabled
    },
    { immediate: false },
  )
  watch(
    () => appStore.autoHideToTrayOnAutostart,
    (enabled) => {
      autoHideToTrayOnAutostart.value = enabled
    },
    { immediate: false },
  )
  watch(
    () => appStore.trayCloseBehavior,
    (behavior) => {
      trayCloseBehavior.value = behavior
    },
    { immediate: false },
  )

  try {
    platformInfo.value = await systemService.getDetailedPlatformInfo()
  } catch (error) {
    console.error('获取平台信息失败:', error)
  }

  await kernelStore.checkKernelInstallation()
  if (kernelStore.fetchLatestKernelVersion) {
    await kernelStore.fetchLatestKernelVersion()
  }
  // 版本选择已锁定为最新版（旧内核与 1.14+ gRPC API 不兼容），不再拉取 release 列表填充下拉框
  await updateStore.initializeStore?.()
  await setupUpdateProgressListener()
})

onUnmounted(() => {
  cleanupManualDropListener()
  cleanupDownloadListener()
  cleanupUpdateProgressListener()
})
</script>

<style scoped>
.settings-page {
  max-width: var(--content-max-width, 1440px);
  margin: 0 auto;
}

.settings-tabs {
  display: flex;
  gap: var(--space-1);
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  box-shadow: var(--panel-shadow);
  border-radius: var(--radius-lg);
  padding: var(--space-1);
  flex-wrap: wrap;
}

.settings-tab {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: var(--space-2) var(--space-4);
  border-radius: var(--radius-sm);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: 500;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
  position: relative;
  white-space: nowrap;
}

.settings-tab:hover {
  color: var(--text-primary);
  background: var(--bg-surface-2);
}

.settings-tab.active {
  background: var(--primary-color);
  color: var(--primary-contrast);
  box-shadow: 0 2px 8px var(--primary-soft-strong);
}

.tab-dot {
  width: 7px;
  height: 7px;
  border-radius: var(--radius-pill);
  background: var(--warning-color);
  box-shadow: 0 0 6px var(--warning-color);
  flex-shrink: 0;
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.settings-content {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--panel-shadow);
  /* 提供统一的水平内边距，所有子组件内容据此对齐 */
  padding: var(--space-2) var(--space-5) var(--space-4);
}

.section-fade-enter-active,
.section-fade-leave-active {
  transition:
    opacity var(--transition-fast),
    transform var(--transition-fast);
}

.section-fade-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.section-fade-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}

.manual-import-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.manual-import-desc {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}

.manual-drop-zone {
  border: 1px dashed var(--border-strong);
  border-radius: var(--radius-md);
  background: var(--bg-surface-2);
  padding: var(--space-4);
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  transition:
    border-color var(--transition-fast),
    background var(--transition-fast);
}

.manual-drop-zone.active {
  border-color: var(--primary-color);
  background: var(--primary-soft);
}

.manual-drop-sub {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.manual-selected {
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-sm);
  padding: var(--space-3);
  background: var(--bg-surface-2);
}

.manual-selected-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-bottom: var(--space-1);
}

.manual-selected-path {
  font-size: var(--text-xs);
  color: var(--text-primary);
  word-break: break-all;
  line-height: 1.45;
  font-family: var(--font-mono);
}

@media (max-width: 768px) {
  .settings-tabs {
    overflow-x: auto;
    flex-wrap: nowrap;
  }

  .settings-tab-label {
    display: none;
  }

  .settings-tab {
    padding: var(--space-2) var(--space-3);
  }
}
</style>
