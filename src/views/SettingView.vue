<template>
  <div class="page-container">
    <PageHeader :title="t('setting.title')" :subtitle="t('setting.subtitle')" />

    <div class="settings-shell">
      <n-tabs v-model:value="activeTab" type="segment" animated class="settings-tabs">
        <n-tab-pane name="basics" :tab="settingTabs.basics.label" display-directive="show:lazy">
          <div class="settings-tab-intro">
            <div class="tab-intro-icon">
              <n-icon size="20"><component :is="settingTabs.basics.icon" /></n-icon>
            </div>
            <div>
              <div class="tab-intro-title">{{ settingTabs.basics.label }}</div>
              <div class="tab-intro-desc">{{ settingTabs.basics.description }}</div>
            </div>
          </div>
          <SettingsBasicTab
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
        </n-tab-pane>

        <n-tab-pane name="kernel" :tab="settingTabs.kernel.label" display-directive="show:lazy">
          <div class="settings-tab-intro">
            <div class="tab-intro-icon">
              <n-icon size="20"><component :is="settingTabs.kernel.icon" /></n-icon>
            </div>
            <div>
              <div class="tab-intro-title">{{ settingTabs.kernel.label }}</div>
              <div class="tab-intro-desc">{{ settingTabs.kernel.description }}</div>
            </div>
          </div>
          <SettingsKernelTab
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
        </n-tab-pane>

        <n-tab-pane name="advanced" :tab="settingTabs.advanced.label" display-directive="show:lazy">
          <div class="settings-tab-intro">
            <div class="tab-intro-icon">
              <n-icon size="20"><component :is="settingTabs.advanced.icon" /></n-icon>
            </div>
            <div>
              <div class="tab-intro-title">{{ settingTabs.advanced.label }}</div>
              <div class="tab-intro-desc">{{ settingTabs.advanced.description }}</div>
            </div>
          </div>
          <SettingsAdvancedTab
            :t="t"
            :app-store="appStore"
            :tun-stack-options="tunStackOptions"
            :using-original-config="usingOriginalConfig"
            :on-ip-version-change="onIpVersionChange"
            :on-lan-access-change="onLanAccessChange"
            :show-port-settings="showPortSettings"
          />
        </n-tab-pane>

        <n-tab-pane name="maintenance" :tab="settingTabs.maintenance.label" display-directive="show:lazy">
          <div class="settings-tab-intro">
            <div class="tab-intro-icon">
              <n-icon size="20"><component :is="settingTabs.maintenance.icon" /></n-icon>
            </div>
            <div>
              <div class="tab-intro-title">{{ settingTabs.maintenance.label }}</div>
              <div class="tab-intro-desc">{{ settingTabs.maintenance.description }}</div>
            </div>
          </div>
          <SettingsMaintenanceTab
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
        </n-tab-pane>

        <n-tab-pane name="about" :tab="settingTabs.about.label" display-directive="show:lazy">
          <div class="settings-tab-intro">
            <div class="tab-intro-icon">
              <n-icon size="20"><component :is="settingTabs.about.icon" /></n-icon>
            </div>
            <div>
              <div class="tab-intro-title">{{ settingTabs.about.label }}</div>
              <div class="tab-intro-desc">{{ settingTabs.about.description }}</div>
            </div>
          </div>
          <SettingsAboutTab
            :t="t"
            :update-store="updateStore"
            :kernel-store="kernelStore"
            :platform-info="platformInfo"
            :format-version="formatVersion"
          />
        </n-tab-pane>
      </n-tabs>
    </div>

    <PortSettingsDialog v-model:show="showPortModal" />

    <!-- Manual Kernel Import Modal -->
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
  SettingsOutline,
  DownloadOutline,
  PowerOutline,
  OptionsOutline,
  RefreshOutline,
  InformationCircleOutline,
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
import PageHeader from '@/components/common/PageHeader.vue'
import PortSettingsDialog from '@/components/common/PortSettingsDialog.vue'
import { ACCENT_PRESETS, TUN_STACK_OPTIONS } from '@/views/setting/setting-options'
import { useKernelDownload } from '@/views/setting/useKernelDownload'
import { useUpdateProgressListener } from '@/views/setting/useUpdateProgressListener'
import SettingsBasicTab from '@/views/setting/components/SettingsBasicTab.vue'
import SettingsKernelTab from '@/views/setting/components/SettingsKernelTab.vue'
import SettingsAdvancedTab from '@/views/setting/components/SettingsAdvancedTab.vue'
import SettingsMaintenanceTab from '@/views/setting/components/SettingsMaintenanceTab.vue'
import SettingsAboutTab from '@/views/setting/components/SettingsAboutTab.vue'

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
interface SettingTabMeta {
  label: string
  description: string
  icon: Component
}

// State
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

// Options
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

// Computed
const settingTabs = computed<Record<SettingTabKey, SettingTabMeta>>(() => ({
  basics: {
    label: t('setting.navigation.basics'),
    description: t('setting.navigation.basicsDesc'),
    icon: PowerOutline,
  },
  kernel: {
    label: t('setting.navigation.kernel'),
    description: t('setting.navigation.kernelDesc'),
    icon: SettingsOutline,
  },
  advanced: {
    label: t('setting.navigation.advanced'),
    description: t('setting.navigation.advancedDesc'),
    icon: OptionsOutline,
  },
  maintenance: {
    label: t('setting.navigation.maintenance'),
    description: t('setting.navigation.maintenanceDesc'),
    icon: RefreshOutline,
  },
  about: {
    label: t('setting.navigation.about'),
    description: t('setting.navigation.aboutDesc'),
    icon: InformationCircleOutline,
  },
}))
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

// Methods
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
  } catch (error) {
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

  // 选择 auto 时明确切回自动模式，避免保留旧值
  const nextLocale = value === 'auto' ? 'auto' : value
  await localeStore.setLocale(nextLocale as Locale)
}

const onIpVersionChange = async (value: boolean) => {
  try {
    // 先更新偏好并持久化，保证后端读取到最新配置
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

  // 获取平台信息
  try {
    platformInfo.value = await systemService.getDetailedPlatformInfo()
  } catch (error) {
    console.error('获取平台信息失败:', error)
  }

  await kernelStore.checkKernelInstallation()
  if (kernelStore.fetchLatestKernelVersion) {
    await kernelStore.fetchLatestKernelVersion()
  }
  if (
    kernelStore.fetchKernelReleases &&
    (!kernelStore.availableVersions || kernelStore.availableVersions.length === 0)
  ) {
    await kernelStore.fetchKernelReleases()
  }
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
.page-container {
  padding: var(--layout-page-padding-y, 24px) var(--layout-page-padding-x, 32px);
  max-width: var(--layout-page-max-width, 1200px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 24px);
}

.settings-shell {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--layout-card-radius, 16px);
  padding: var(--layout-card-padding, 20px);
  box-shadow: 0 18px 40px rgba(15, 23, 42, 0.04);
}

.settings-tabs :deep(.n-tabs-nav) {
  margin-bottom: 18px;
}

.settings-tabs :deep(.n-tabs-pane-wrapper) {
  overflow: visible;
}

.settings-tab-intro {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 16px;
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  background: var(--bg-tertiary);
  margin-bottom: 18px;
}

.tab-intro-icon {
  width: 38px;
  height: 38px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color);
  background: rgba(59, 130, 246, 0.12);
  flex-shrink: 0;
}

.tab-intro-title {
  font-size: 15px;
  font-weight: 700;
  color: var(--text-primary);
}

.tab-intro-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

:deep(.settings-tab-panel) {
  display: flex;
  flex-direction: column;
  gap: 18px;
}

:deep(.settings-grid) {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: var(--layout-grid-gap, 24px);
}

:deep(.settings-section.full-width) {
  grid-column: 1 / -1;
}

:deep(.settings-section) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

:deep(.section-header) {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
}

:deep(.section-header h3) {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

:deep(.section-card) {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--layout-card-radius, 16px);
  padding: var(--layout-card-padding, 20px);
  display: flex;
  flex-direction: column;
  gap: var(--layout-row-gap, 16px);
}

:deep(.setting-row) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--layout-row-gap, 16px);
}

:deep(.setting-info) {
  flex: 1;
}

:deep(.setting-label) {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 2px;
}

:deep(.setting-desc) {
  font-size: 12px;
  color: var(--text-tertiary);
}

:deep(.setting-input) {
  width: 140px;
}

:deep(.setting-input-wide) {
  width: 220px;
}

:deep(.setting-row.align-start) {
  align-items: flex-start;
}

:deep(.theme-card) {
  gap: 18px;
}

:deep(.theme-mode-selector) {
  display: flex;
  gap: 8px;
}

:deep(.theme-accent) {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

:deep(.preset-swatches) {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

:deep(.preset-swatch) {
  width: 34px;
  height: 24px;
  border-radius: 10px;
  border: 2px solid transparent;
  cursor: pointer;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.08);
  transition:
    transform 0.15s ease,
    box-shadow 0.15s ease,
    border-color 0.2s ease;
  position: relative;
}

:deep(.preset-swatch:hover) {
  transform: translateY(-1px);
  box-shadow: 0 6px 14px rgba(0, 0, 0, 0.12);
}

:deep(.swatch-active) {
  position: absolute;
  inset: 4px;
  border-radius: 8px;
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.2);
}

:deep(.alert-box) {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
}

:deep(.alert-box.warning) {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

:deep(.alert-box.info) {
  background: rgba(14, 165, 233, 0.12);
  color: #0ea5e9;
  margin-bottom: 12px;
}

:deep(.download-box) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

:deep(.download-text) {
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}

:deep(.actions-row) {
  display: flex;
  flex-direction: column;
  gap: var(--layout-inline-gap, 12px);
}

:deep(.sub-actions) {
  display: flex;
  gap: var(--layout-inline-gap-tight, 8px);
  justify-content: center;
}

:deep(.advanced-form) {
  width: 100%;
}

:deep(.subsection-title) {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

:deep(.toggles-row) {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

:deep(.toggle-item) {
  display: flex;
  align-items: center;
  gap: calc(var(--layout-inline-gap, 12px) - 2px);
  font-size: 13px;
  color: var(--text-secondary);
}

:deep(.update-status) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 8px;
}

:deep(.version-info) {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--text-secondary);
}

:deep(.about-list) {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

:deep(.about-item) {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

:deep(.about-item .label) {
  color: var(--text-tertiary);
}

:deep(.about-item .value) {
  color: var(--text-primary);
  font-weight: 500;
}

:deep(.about-actions) {
  margin-top: 8px;
  display: flex;
  justify-content: center;
}

:deep(.update-alert-card) {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.12), rgba(6, 182, 212, 0.1));
  border: 1px solid rgba(16, 185, 129, 0.2);
}

:deep(.update-meta) {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
}

:deep(.meta-box) {
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
}

:deep(.meta-label) {
  display: block;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 6px;
}

:deep(.meta-value) {
  font-weight: 700;
  color: var(--text-primary);
  font-size: 16px;
}

:deep(.update-actions) {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

:deep(.update-notes-preview) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

:deep(.update-notes-preview .notes) {
  max-height: 120px;
  overflow: auto;
  padding: 10px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

:deep(.update-platform-hint) {
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(99, 102, 241, 0.08);
  border: 1px solid rgba(99, 102, 241, 0.16);
  color: var(--text-secondary);
  font-size: 12px;
  line-height: 1.6;
}

:deep(.update-progress) {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

:deep(.progress-header) {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: var(--text-secondary);
}

:deep(.progress-value) {
  font-weight: 600;
  color: var(--text-primary);
}

:deep(.update-error) {
  font-size: 13px;
  color: #ef4444;
}

:deep(.backup-actions) {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 160px;
}

:deep(.backup-preview) {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--panel-border);
  border-radius: 10px;
  background: var(--bg-tertiary);
}

:deep(.backup-preview-row) {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

:deep(.backup-preview-row.warning .meta-value) {
  color: #f59e0b;
}

:deep(.backup-path) {
  flex: 1;
  text-align: right;
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
}

:deep(.backup-warning-list) {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

:deep(.backup-warning-item) {
  font-size: 12px;
  color: #f59e0b;
  line-height: 1.5;
}

.manual-import-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.manual-import-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.manual-drop-zone {
  border: 1px dashed var(--panel-border);
  border-radius: 10px;
  background: var(--bg-tertiary);
  padding: 16px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  transition:
    border-color 0.2s ease,
    background 0.2s ease;
}

.manual-drop-zone.active {
  border-color: var(--primary-color);
  background: rgba(59, 130, 246, 0.12);
}

.manual-drop-sub {
  font-size: 12px;
  color: var(--text-tertiary);
}

.manual-selected {
  border: 1px solid var(--panel-border);
  border-radius: 8px;
  padding: 10px 12px;
  background: var(--bg-tertiary);
}

.manual-selected-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 4px;
}

.manual-selected-path {
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
  line-height: 1.45;
}

@media (max-width: 768px) {
  .settings-shell {
    padding: 14px;
  }

  :deep(.settings-grid) {
    grid-template-columns: 1fr;
  }

  :deep(.setting-row) {
    align-items: flex-start;
    flex-direction: column;
  }

  :deep(.setting-input),
  :deep(.setting-input-wide) {
    width: 100%;
  }
}
</style>
