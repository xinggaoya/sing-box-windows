<template>
  <div class="setting-container">
    <!-- 内核管理卡片 -->
    <n-card class="setting-card" :bordered="false">
      <template #header-extra>
        <n-space align="center" :size="12">
          <n-tag
            v-if="infoStore.version.version"
            :bordered="false"
            type="default"
            size="medium"
            class="version-tag"
          >
            {{ t('setting.kernel.currentVersion') }}{{ infoStore.version.version }}
          </n-tag>
          <n-tag v-else :bordered="false" type="error" size="medium" class="version-tag">
            {{ t('setting.kernel.notInstalled') }}
          </n-tag>
          <n-tag
            v-if="hasNewVersion"
            :bordered="false"
            type="warning"
            size="medium"
            class="version-tag"
          >
            {{ t('setting.kernel.newVersion') }}{{ infoStore.newVersion }}
          </n-tag>
        </n-space>
      </template>
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="24" class="card-icon">
              <settings-outline />
            </n-icon>
            {{ t('setting.kernel.title') }}
          </n-h3>
        </div>
      </template>

      <n-space vertical :size="20">
        <n-alert
          v-if="hasNewVersion"
          type="warning"
          :show-icon="true"
          :title="t('setting.kernel.newVersionFound')"
          class="version-alert"
        >
          {{ t('setting.kernel.updateTip') }}
        </n-alert>

        <n-alert
          v-if="!infoStore.version.version"
          type="error"
          :show-icon="true"
          :title="t('setting.kernel.notInstalled')"
          class="version-alert"
        >
          {{ t('setting.kernel.installPrompt') }}
        </n-alert>

        <n-progress
          v-if="downloading"
          type="line"
          :percentage="downloadProgress"
          :processing="downloadProgress < 100"
          :indicator-placement="'inside'"
          :rail-style="{ background: 'var(--n-color-disabled)' }"
          class="download-progress"
        >
          {{ downloadMessage }}
        </n-progress>

        <n-space align="center" justify="space-between">
          <n-button
            type="primary"
            @click="downloadTheKernel"
            :loading="loading"
            :disabled="downloading"
            size="medium"
            class="download-button"
          >
            <template #icon>
              <n-icon>
                <download-outline />
              </n-icon>
            </template>
            {{
              hasNewVersion
                ? t('setting.kernel.downloadNew')
                : infoStore.version.version
                  ? t('setting.kernel.redownload')
                  : t('setting.kernel.download')
            }}
          </n-button>

          <n-space :size="16">
            <n-button
              text
              size="medium"
              @click="showManualDownloadModal"
              :disabled="downloading"
              class="action-button"
            >
              {{ t('setting.kernel.manualDownload') }}
            </n-button>
            <n-button text size="medium" @click="checkManualInstall" :disabled="downloading">
              {{ t('setting.kernel.checkInstall') }}
            </n-button>
          </n-space>
        </n-space>

        <n-alert v-if="downloadError" type="error" :show-icon="true" style="margin-top: 16px">
          <template #header> {{ t('setting.kernel.downloadFailed') }} </template>
          <div style="white-space: pre-line">{{ downloadError }}</div>
        </n-alert>
      </n-space>
    </n-card>

    <!-- 启动设置卡片 -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <power-outline />
            </n-icon>
            {{ t('setting.startup.title') }}
          </n-h3>
        </div>
      </template>

      <n-list>
        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ t('setting.autoStart.app') }}</div>
              <div class="setting-desc">
                {{
                  appStore.autoStartApp
                    ? t('setting.startup.bootTip')
                    : t('setting.startup.manualTip')
                }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartApp" @update-value="onAutoStartChange">
              <template #checked>{{ t('common.on') }}</template>
              <template #unchecked>{{ t('common.off') }}</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ t('setting.autoStart.kernel') }}</div>
              <div class="setting-desc">
                {{
                  appStore.autoStartKernel
                    ? t('setting.startup.autoKernelTip')
                    : t('setting.startup.manualKernelTip')
                }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartKernel">
              <template #checked>{{ t('common.on') }}</template>
              <template #unchecked>{{ t('common.off') }}</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ $t('setting.language.title') }}</div>
              <div class="setting-desc">
                {{
                  appStore.locale === 'auto'
                    ? $t('setting.language.auto')
                    : supportedLocales.find((loc) => loc.code === appStore.locale)?.name
                }}
              </div>
            </div>
            <n-select
              v-model:value="appStore.locale"
              :options="languageOptions"
              size="small"
              style="min-width: 120px"
              @update:value="handleChangeLanguage"
            />
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ t('setting.network.ipv6') }}</div>
              <div class="setting-desc">
                {{
                  appStore.preferIpv6
                    ? t('setting.network.preferIpv6')
                    : t('setting.network.onlyIpv4')
                }}
              </div>
            </div>
            <n-switch v-model:value="appStore.preferIpv6" @update-value="onIpVersionChange">
              <template #checked>{{ t('common.on') }}</template>
              <template #unchecked>{{ t('common.off') }}</template>
            </n-switch>
          </n-space>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- 关于卡片 -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <information-circle-outline />
            </n-icon>
            {{ t('setting.about.title') }}
          </n-h3>
        </div>
      </template>

      <n-grid :cols="2" :x-gap="12" :y-gap="8">
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('setting.about.appVersion') }}</span>
            <n-space align="center">
              <span class="about-value">{{ appStore.appVersion }}</span>
              <n-button text size="tiny" @click="handleCheckUpdate" :loading="checkingUpdate">
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                {{ t('setting.update.check') }}
              </n-button>
            </n-space>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('setting.about.kernelVersion') }}</span>
            <span class="about-value">{{ infoStore.version.version }}</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('setting.about.system') }}</span>
            <span class="about-value">Windows</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('setting.about.license') }}</span>
            <span class="about-value">MIT License</span>
          </div>
        </n-gi>
      </n-grid>

      <div class="about-footer">
        <n-space justify="center" align="center">
          <n-button
            text
            tag="a"
            href="https://github.com/xinggaoya/sing-box-windows"
            target="_blank"
          >
            <template #icon>
              <n-icon><logo-github /></n-icon>
            </template>
            GitHub
          </n-button>
          <n-divider vertical />
          <n-button
            text
            tag="a"
            href="https://github.com/xinggaoya/sing-box-windows"
            target="_blank"
          >
            <template #icon>
              <n-icon><globe-outline /></n-icon>
            </template>
            {{ t('setting.about.website') }}
          </n-button>
        </n-space>
      </div>
    </n-card>
  </div>

  <!-- 应用更新对话框 -->
  <update-modal
    v-model:show="showUpdateModal"
    :latest-version="latestVersion"
    :current-version="appStore.appVersion"
    :download-url="downloadUrl"
    @update="handleUpdate"
    @cancel="skipUpdate"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import { useInfoStore } from '@/stores/infoStore'
import { useAppStore } from '@/stores/AppStore'
import {
  DownloadOutline,
  SettingsOutline,
  PowerOutline,
  InformationCircleOutline,
  LogoGithub,
  GlobeOutline,
  RefreshOutline,
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri-api'
import { appDataDir } from '@tauri-apps/api/path'
import UpdateModal from '@/components/UpdateModal.vue'
import { supportedLocales } from '@/locales'
import { Locale } from '@/stores/AppStore'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import { getVersion } from '@tauri-apps/api/app'
import i18n from '@/locales'

const message = useMessage()
const dialog = useDialog()
const appStore = useAppStore()
const infoStore = useInfoStore()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')
const { t } = useI18n()

// 更新相关状态
const showUpdateModal = ref(false)
const latestVersion = ref('')
const downloadUrl = ref('')
const skipUpdateFlag = ref(false)

// 检查更新状态
const checkingUpdate = ref(false)

// 新增状态
const downloadError = ref<string | null>(null)
const appDataPath = ref('')

// 语言选项
const languageOptions = [
  { label: '自动', value: 'auto' as Locale },
  ...supportedLocales.map((locale) => ({
    label: locale.name,
    value: locale.code as Locale,
  })),
]

// 检查更新
const checkUpdate = async () => {
  try {
    if (skipUpdateFlag.value) return

    const result = await tauriApi.update.checkUpdate(appStore.appVersion)
    if (result.has_update) {
      showUpdateModal.value = true
      latestVersion.value = result.latest_version
      downloadUrl.value = result.download_url
    }
  } catch (error) {
    console.error(t('setting.update.checkError'), error)
  }
}

// 处理更新
const handleUpdate = async () => {
  try {
    await tauriApi.update.downloadAndInstallUpdate(downloadUrl.value)
  } catch (error) {
    message.error(t('setting.update.updateError') + error)
  }
}

// 跳过更新
const skipUpdate = () => {
  showUpdateModal.value = false
  skipUpdateFlag.value = true
}

// 手动检查更新
const handleCheckUpdate = async () => {
  try {
    checkingUpdate.value = true
    const result = await appStore.checkUpdate(false)
    if (result?.has_update) {
      showUpdateModal.value = true
      latestVersion.value = result.latest_version
      downloadUrl.value = result.download_url
      message.success(t('setting.update.newVersionFound', { version: result.latest_version }))
    } else {
      message.info(t('setting.update.alreadyLatest'))
    }
  } catch (error) {
    message.error(`${t('setting.update.checkError')}: ${error}`)
  } finally {
    checkingUpdate.value = false
  }
}

const hasNewVersion = computed(() => {
  if (!infoStore.newVersion || !infoStore.version.version) return false
  return infoStore.newVersion != infoStore.version.version
})

const downloadTheKernel = async () => {
  try {
    loading.value = true
    downloading.value = true
    downloadProgress.value = 0
    downloadMessage.value = t('setting.kernel.preparingDownload')
    downloadError.value = null

    await tauriApi.subscription.downloadLatestKernel()

    // 下载成功后更新版本信息
    await infoStore.updateVersion()
  } catch (error) {
    downloadError.value = error as string
    message.error(error as string)
  } finally {
    downloading.value = false
    loading.value = false
  }
}

// 开机自启动设置
const onAutoStartChange = async (value: boolean) => {
  try {
    // 检查管理员权限
    const isAdmin = await tauriApi.system.checkAdmin()
    if (!isAdmin) {
      // 如果没有管理员权限，请求以管理员权限重启
      await tauriApi.system.restartAsAdmin()
      return
    }

    // 移除旧版开机自启
    if (await isEnabled()) {
      await disable()
    }

    // 使用计划任务设置开机自启
    await tauriApi.system.setAutostart(value)
    message.success(
      value ? t('setting.startup.enableSuccess') : t('setting.startup.disableSuccess'),
    )
  } catch (error) {
    message.error(`${t('common.error')}: ${error}`)
    // 恢复原来的设置
    appStore.autoStartApp = !value
  }
}

const onIpVersionChange = async (value: boolean) => {
  try {
    await tauriApi.proxy.toggleIpVersion(value)
    // 切换后重启内核
    if (appStore.isRunning) {
      await tauriApi.kernel.restartKernel()
    }
  } catch (error: unknown) {
    message.error(`${t('common.error')}: ${error instanceof Error ? error.message : String(error)}`)
    // 回滚状态
    appStore.preferIpv6 = !value
  }
}

// 显示手动下载指引
const showManualDownloadModal = () => {
  dialog.info({
    title: t('setting.kernel.manualDownloadTitle'),
    content: t('setting.kernel.manualDownloadGuide'),
    positiveText: t('common.ok'),
  })
}

// 检查手动安装
const checkManualInstall = async () => {
  try {
    loading.value = true
    const success = await infoStore.checkKernelVersion()
    if (success) {
      message.success(t('setting.kernel.installSuccess'))
    } else {
      message.error(t('setting.kernel.installFailed'))
    }
  } catch (error) {
    message.error(`${t('common.error')}: ${error}`)
  } finally {
    loading.value = false
  }
}

// 获取应用数据目录
const getAppDataPath = async () => {
  try {
    appDataPath.value = await appDataDir()
  } catch (error) {
    console.error(t('setting.error.appDataDir'), error)
  }
}

// 监听下载进度事件
listen(
  'download-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    downloadProgress.value = progress
    downloadMessage.value = msg

    if (status === 'completed') {
      downloading.value = false
      downloadError.value = null
      message.success(t('setting.kernel.downloadComplete'))
      // 更新版本信息
      infoStore.updateVersion()
    }
  },
)

// 切换语言
const handleChangeLanguage = async (value: string) => {
  appStore.setLocale(value as Locale)
  i18n.global.locale.value = value as 'zh-CN' | 'en-US' | 'ru-RU' | 'ja-JP'
  // 发送语言变更事件，通知托盘菜单刷新
  mitt.emit('language-changed')
}

onMounted(async () => {
  // 获取当前版本号
  await appStore.fetchAppVersion()
  // 检查更新
  await checkUpdate()
  // 获取应用数据目录
  await getAppDataPath()
  // 获取内核版本信息
  await infoStore.updateVersion()
  // 检查开机自启状态
  const isAutostartEnabled = await tauriApi.system.isAutostartEnabled()
  appStore.autoStartApp = isAutostartEnabled
})
</script>

<style scoped>
.setting-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 8px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  animation: slide-up 0.4s ease;
}

.setting-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
}

.setting-card:hover {
  transform: translateY(-2px);
  box-shadow: var(--shadow-medium);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
  color: var(--n-text-color);
}

.card-icon {
  color: var(--primary-color);
}

.version-tag {
  font-weight: 500;
  padding: 0 12px;
  height: 28px;
}

.version-alert {
  border-radius: 10px;
  font-size: 14px;
}

.download-progress {
  margin: 10px 0;
  height: 36px;
  font-weight: 500;
}

.download-button {
  font-weight: 500;
  min-width: 140px;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.download-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(64, 128, 255, 0.25);
}

.action-button {
  font-weight: 500;
  color: var(--n-text-color);
  transition: all 0.25s ease;
}

.action-button:hover:not(:disabled) {
  color: var(--primary-color);
  transform: translateY(-1px);
}

:deep(.n-switch) {
  --n-rail-color-active: var(--primary-color);
}

:deep(.n-radio-button) {
  border-radius: 8px;
}

:deep(.n-form-item-feedback) {
  font-size: 13px;
}

:deep(.n-tabs-nav) {
  background-color: transparent;
}

:deep(.n-tabs-tab) {
  font-weight: 500;
}

:deep(.n-tabs-tab.n-tabs-tab--active) {
  font-weight: 600;
  color: var(--primary-color);
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-title {
  font-size: 14px;
  font-weight: 500;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-color-3);
}

.about-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  background-color: var(--card-color);
  border-radius: 4px;
}

.about-label {
  color: var(--text-color-2);
  font-size: 13px;
}

.about-value {
  color: var(--text-color-1);
  font-size: 13px;
  font-weight: 500;
}

.about-footer {
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--divider-color);
}

.manual-path {
  font-family: monospace;
  background-color: var(--n-color-modal);
  padding: 8px;
  margin-top: 4px;
  border-radius: 4px;
  word-break: break-all;
}
</style>
