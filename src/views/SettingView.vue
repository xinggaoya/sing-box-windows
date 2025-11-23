<template>
  <div class="page-container">
    <PageHeader :title="t('setting.title')" :subtitle="t('setting.subtitle')" />

    <!-- Settings Grid -->
    <div class="settings-grid">
      <!-- Kernel Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><SettingsOutline /></n-icon>
          <h3>{{ t('setting.kernel.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.kernel.version') }}</div>
              <div class="setting-desc">{{ t('setting.kernel.description') }}</div>
            </div>
            <div class="setting-action">
              <n-tag v-if="kernelStore.hasVersionInfo()" type="success" round :bordered="false">
                {{ formatVersion(kernelStore.getVersionString()) }}
              </n-tag>
              <n-tag v-else type="error" round :bordered="false">
                {{ t('setting.notInstalled') }}
              </n-tag>
            </div>
          </div>

          <div v-if="hasNewVersion || !kernelStore.hasVersionInfo()" class="alert-box warning">
            <n-icon size="18"><WarningOutline /></n-icon>
            <span>{{ hasNewVersion ? t('setting.newVersionFound') : t('setting.kernel.installPrompt') }}</span>
          </div>

          <div v-if="downloading" class="download-box">
            <n-progress
              type="line"
              :percentage="downloadProgress"
              :processing="downloadProgress < 100"
              indicator-placement="inside"
            />
            <div class="download-text">{{ downloadMessage }}</div>
          </div>

          <div class="actions-row">
            <n-button
              type="primary"
              @click="downloadTheKernel"
              :loading="loading"
              :disabled="downloading"
              block
              secondary
            >
              <template #icon><n-icon><DownloadOutline /></n-icon></template>
              {{ hasNewVersion ? t('setting.kernel.update') : kernelStore.hasVersionInfo() ? t('setting.kernel.redownload') : t('setting.kernel.download') }}
            </n-button>
            <div class="sub-actions">
              <n-button size="small" ghost @click="showManualDownloadModal" :disabled="downloading">
                {{ t('setting.kernel.manualDownload') }}
              </n-button>
              <n-button size="small" ghost @click="checkManualInstall" :disabled="downloading">
                {{ t('setting.kernel.checkInstall') }}
              </n-button>
            </div>
          </div>
        </div>
      </div>

      <!-- Startup Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><PowerOutline /></n-icon>
          <h3>{{ t('setting.startup.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.autoStart.app') }}</div>
              <div class="setting-desc">{{ t('setting.autoStart.appDesc') }}</div>
            </div>
            <n-switch v-model:value="autoStart" @update:value="onAutoStartChange" />
          </div>
        </div>
      </div>

      <!-- General Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><GlobeOutline /></n-icon>
          <h3>{{ t('setting.general.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.language.title') }}</div>
              <div class="setting-desc">{{ t('setting.language.description') }}</div>
            </div>
            <n-select
              v-model:value="localeStore.locale"
              :options="languageOptions"
              size="small"
              @update:value="handleChangeLanguage"
              class="setting-input"
            />
          </div>
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.network.ipv6') }}</div>
              <div class="setting-desc">{{ t('setting.network.ipv6Desc') }}</div>
            </div>
            <n-switch v-model:value="appStore.preferIpv6" @update:value="onIpVersionChange" />
          </div>
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.network.ports') }}</div>
              <div class="setting-desc">{{ t('setting.network.portsDesc') }}</div>
            </div>
            <n-button size="small" secondary @click="showPortSettings">
              {{ t('setting.network.configure') }}
            </n-button>
          </div>
        </div>
      </div>

      <!-- Advanced Proxy Settings -->
      <div class="settings-section full-width">
        <div class="section-header">
          <n-icon size="20"><OptionsOutline /></n-icon>
          <h3>{{ t('setting.proxyAdvanced.title') }}</h3>
        </div>
        <div class="section-card">
          <n-form label-placement="top" class="advanced-form">
            <n-grid :cols="24" :x-gap="24" :y-gap="16">
              <n-grid-item :span="24">
                <n-form-item :label="t('setting.proxyAdvanced.systemBypass')">
                  <n-input
                    v-model:value="proxyAdvancedForm.systemProxyBypass"
                    type="textarea"
                    :rows="3"
                    :placeholder="t('setting.proxyAdvanced.systemBypassPlaceholder')"
                  />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="24">
                <div class="subsection-title">{{ t('setting.proxyAdvanced.tunTitle') }}</div>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.proxyAdvanced.tunMtu')">
                  <n-input-number v-model:value="proxyAdvancedForm.tunMtu" :min="576" :max="9000" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.proxyAdvanced.tunStack')">
                  <n-select v-model:value="proxyAdvancedForm.tunStack" :options="tunStackOptions" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="24">
                <div class="toggles-row">
                  <div class="toggle-item">
                    <span>{{ t('setting.proxyAdvanced.enableIpv6') }}</span>
                    <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
                  </div>
                  <div class="toggle-item">
                    <span>{{ t('setting.proxyAdvanced.autoRoute') }}</span>
                    <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
                  </div>
                  <div class="toggle-item">
                    <span>{{ t('setting.proxyAdvanced.strictRoute') }}</span>
                    <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
                  </div>
                </div>
              </n-grid-item>

              <n-grid-item :span="24">
                <n-button
                  type="primary"
                  block
                  :loading="savingAdvanced"
                  @click="saveProxyAdvancedSettings"
                >
                  {{ t('setting.proxyAdvanced.save') }}
                </n-button>
              </n-grid-item>
            </n-grid>
          </n-form>
        </div>
      </div>

      <!-- Update Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><RefreshOutline /></n-icon>
          <h3>{{ t('setting.update.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="update-status">
            <div class="version-info">
              <span>{{ t('setting.update.currentVersion') }}: {{ updateStore.appVersion }}</span>
              <n-tag v-if="updateStore.hasUpdate" type="warning" size="small" round :bordered="false">
                {{ t('setting.update.hasUpdate') }}
              </n-tag>
              <n-tag v-else type="success" size="small" round :bordered="false">
                {{ t('setting.update.latest') }}
              </n-tag>
            </div>
            <n-button
              size="small"
              secondary
              @click="handleCheckUpdate"
              :loading="checkingUpdate"
              :disabled="updateStore.isChecking"
            >
              {{ checkingUpdate ? t('setting.update.checking') : t('setting.update.checkNow') }}
            </n-button>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.update.autoCheck') }}</div>
            </div>
            <n-switch v-model:value="updateStore.autoCheckUpdate" />
          </div>
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.update.acceptPrerelease') }}</div>
            </div>
            <n-switch v-model:value="updateStore.acceptPrerelease" @update:value="onPrereleaseSettingChange" />
          </div>
        </div>
      </div>

      <!-- About -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><InformationCircleOutline /></n-icon>
          <h3>{{ t('setting.about.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="about-list">
            <div class="about-item">
              <span class="label">{{ t('setting.appVersion') }}</span>
              <span class="value">{{ updateStore.appVersion }}</span>
            </div>
            <div class="about-item">
              <span class="label">{{ t('setting.kernel.version') }}</span>
              <span class="value">{{ kernelStore.hasVersionInfo() ? formatVersion(kernelStore.getVersionString()) : t('setting.notInstalled') }}</span>
            </div>
            <div class="about-item">
              <span class="label">{{ t('setting.about.license') }}</span>
              <span class="value">MIT License</span>
            </div>
            <div class="about-actions">
              <n-button text tag="a" href="https://github.com/xinggaoya/sing-box-windows" target="_blank">
                <template #icon><n-icon><LogoGithub /></n-icon></template>
                GitHub
              </n-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Port Modal -->
    <n-modal
      v-model:show="showPortModal"
      preset="dialog"
      :title="t('setting.network.portSettings')"
      class="modern-modal"
      :style="{ width: '400px' }"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('setting.network.proxyPort')">
          <n-input-number v-model:value="tempProxyPort" :min="1024" :max="65535" />
        </n-form-item>
        <n-form-item :label="t('setting.network.apiPort')">
          <n-input-number v-model:value="tempApiPort" :min="1024" :max="65535" />
        </n-form-item>
      </n-form>
      <template #action>
        <n-space justify="end">
          <n-button @click="showPortModal = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" @click="savePortSettings" :loading="portSettingsLoading">
            {{ t('common.save') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, reactive, watch } from 'vue'
import { useMessage } from 'naive-ui'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useAppStore } from '@/stores/app/AppStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useLocaleStore } from '@/stores/app/LocaleStore'
import type { Locale } from '@/stores/app/LocaleStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import {
  DownloadOutline,
  SettingsOutline,
  PowerOutline,
  InformationCircleOutline,
  LogoGithub,
  GlobeOutline,
  RefreshOutline,
  WarningOutline,
  OptionsOutline,
} from '@vicons/ionicons5'
import { tauriApi } from '@/services/tauri'
import { eventService } from '@/services/event-service'
import { supportedLocales } from '@/locales'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'

defineOptions({
  name: 'SettingView'
})

const message = useMessage()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const { t, locale } = useI18n()

const autoStart = ref(false)
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')
const savingAdvanced = ref(false)
const checkingUpdate = ref(false)
const showPortModal = ref(false)
const tempProxyPort = ref(7890)
const tempApiPort = ref(9090)
const portSettingsLoading = ref(false)

const proxyAdvancedForm = reactive({
  systemProxyBypass: appStore.systemProxyBypass,
  tunMtu: appStore.tunMtu,
  tunAutoRoute: appStore.tunAutoRoute,
  tunStrictRoute: appStore.tunStrictRoute,
  tunStack: appStore.tunStack,
  tunEnableIpv6: appStore.tunEnableIpv6,
})

const hasNewVersion = computed(() => false) // Placeholder logic
const downloadError = ref('')
type KernelDownloadPayload = {
  status?: 'downloading' | 'extracting' | 'completed' | 'error'
  progress?: number
  message?: string
}

const languageOptions = computed(() => [
  { label: t('setting.language.auto'), value: 'auto' },
  ...supportedLocales.map((l) => ({ label: l.name, value: l.code })),
])

const tunStackOptions = [
  { label: 'gVisor', value: 'gvisor' },
  { label: 'System', value: 'system' },
  { label: 'Mixed', value: 'mixed' },
]

let downloadListener: (() => void) | null = null

// Methods
const formatVersion = (version: string) => {
  if (!version) return ''
  const match = version.match(/\d+\.\d+\.\d+(?:-[\w.]+)?/)
  return match ? match[0] : version
}

const onAutoStartChange = async (value: boolean) => {
  try {
    await appStore.toggleAutoStart(value)
    autoStart.value = value
  } catch (e) {
    message.error(t('setting.autoStart.error'))
    autoStart.value = !value
  }
}

const isSupportedLocale = (value: string): value is Locale => {
  if (value === 'auto') return true
  return supportedLocales.some((loc) => loc.code === value)
}

const handleChangeLanguage = async (value: string) => {
  if (!isSupportedLocale(value)) {
    console.warn('选择了不受支持的语言:', value)
    return
  }

  await localeStore.setLocale(value)
  locale.value = localeStore.currentLocale
}

const onIpVersionChange = async (value: boolean) => {
  try {
    // 先更新偏好并持久化，保证后端读取到最新配置
    await appStore.setPreferIpv6(value)
    await appStore.saveToBackend()

    const toggled = await kernelStore.toggleIpVersion(value)
    if (!toggled) {
      throw new Error(kernelStore.lastError || '切换IP版本失败')
    }

    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('切换IPv6优先失败:', error)
    message.error(t('notification.proxyModeChangeFailed'))
  }
}

const cleanupDownloadListener = () => {
  if (downloadListener) {
    downloadListener()
    downloadListener = null
  }
}

const downloadTheKernel = async () => {
  if (downloading.value) return
  let downloadCompleted = false
  loading.value = true
  downloading.value = true
  downloadProgress.value = 0
  downloadMessage.value = t('setting.kernel.preparingDownload')
  downloadError.value = ''

  // 监听后端推送的下载进度，实时刷新 UI
  cleanupDownloadListener()
  downloadListener = await eventService.on('kernel-download-progress', (payload) => {
    const data = payload as KernelDownloadPayload
    if (typeof data.progress === 'number') {
      downloadProgress.value = Math.min(100, Math.max(0, data.progress))
    }
    if (data.message) {
      downloadMessage.value = data.message
    }

    if (data.status === 'completed') {
      downloadCompleted = true
      downloading.value = false
      loading.value = false
      message.success(t('setting.kernel.downloadSuccess'))
      kernelStore.checkKernelInstallation()
      cleanupDownloadListener()
    } else if (data.status === 'error') {
      downloading.value = false
      loading.value = false
      downloadError.value = data.message || t('setting.kernel.downloadFailed')
      message.error(downloadError.value)
      cleanupDownloadListener()
    } else {
      downloadMessage.value ||= t('setting.kernel.downloadingDescription')
    }
  })

  try {
    await tauriApi.downloadLatestKernel()
    // 如果后端未推送完成事件，也主动校验一次安装结果
    if (!downloadCompleted) {
      await kernelStore.checkKernelInstallation()
    }
  } catch (e) {
    console.error('下载内核失败:', e)
    downloadError.value = e instanceof Error ? e.message : t('setting.kernel.downloadFailed')
    message.error(downloadError.value)
    downloading.value = false
    loading.value = false
    cleanupDownloadListener()
  } finally {
    // 最终状态由事件驱动更新；若未收到事件则确保按钮可再次点击
    if (downloading.value) {
      loading.value = false
      downloading.value = false
    }
  }
}

const showManualDownloadModal = () => {
  message.info('Manual download not implemented yet')
}

const checkManualInstall = async () => {
  await kernelStore.checkKernelInstallation()
}

const saveProxyAdvancedSettings = async () => {
  savingAdvanced.value = true
  try {
    appStore.systemProxyBypass = proxyAdvancedForm.systemProxyBypass
    appStore.tunMtu = proxyAdvancedForm.tunMtu
    appStore.tunAutoRoute = proxyAdvancedForm.tunAutoRoute
    appStore.tunStrictRoute = proxyAdvancedForm.tunStrictRoute
    appStore.tunStack = proxyAdvancedForm.tunStack
    appStore.tunEnableIpv6 = proxyAdvancedForm.tunEnableIpv6
    await appStore.saveToBackend()
    message.success(t('common.saveSuccess'))
  } catch (e) {
    message.error(t('common.saveFailed'))
  } finally {
    savingAdvanced.value = false
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

const onPrereleaseSettingChange = async () => {
  await updateStore.saveToBackend()
}

const showPortSettings = () => {
  tempProxyPort.value = appStore.proxyPort
  tempApiPort.value = appStore.apiPort
  showPortModal.value = true
}

const savePortSettings = async () => {
  portSettingsLoading.value = true
  try {
    appStore.proxyPort = tempProxyPort.value
    appStore.apiPort = tempApiPort.value
    await appStore.saveToBackend()
    message.success(t('common.saveSuccess'))
    showPortModal.value = false
  } catch (e) {
    message.error(t('common.saveFailed'))
  } finally {
    portSettingsLoading.value = false
  }
}

onMounted(async () => {
  await appStore.waitForDataRestore()
  await appStore.syncAutoStartWithSystem()
  autoStart.value = appStore.autoStartApp
  watch(
    () => appStore.autoStartApp,
    (enabled) => {
      autoStart.value = enabled
    },
    { immediate: false },
  )
  await kernelStore.checkKernelInstallation()
})

onUnmounted(() => {
  cleanupDownloadListener()
})
</script>

<style scoped>
.page-container {
  padding: 24px 32px;
  max-width: 1200px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 24px;
}

.settings-section.full-width {
  grid-column: 1 / -1;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.section-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: 16px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.setting-info {
  flex: 1;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
}

.setting-input {
  width: 140px;
}

.alert-box {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
}

.alert-box.warning {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.download-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.download-text {
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}

.actions-row {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.sub-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.advanced-form {
  width: 100%;
}

.subsection-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.toggles-row {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--text-secondary);
}

.update-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 8px;
}

.version-info {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--text-secondary);
}

.about-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.about-item {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.about-item .label {
  color: var(--text-tertiary);
}

.about-item .value {
  color: var(--text-primary);
  font-weight: 500;
}

.about-actions {
  margin-top: 8px;
  display: flex;
  justify-content: center;
}

@media (max-width: 768px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }
}
</style>
