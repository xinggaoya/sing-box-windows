<template>
  <div class="setting-container">
    <!-- 内核管理卡片 (Kernel Management Card) -->
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
            {{ t('settings.current_version', { version: infoStore.version.version }) }}
          </n-tag>
          <n-tag
            v-else
            :bordered="false"
            type="error"
            size="medium"
            class="version-tag"
          >
            {{ t('settings.kernel_not_installed') }}
          </n-tag>
          <n-tag
            v-if="hasNewVersion"
            :bordered="false"
            type="warning"
            size="medium"
            class="version-tag"
          >
            {{ t('settings.new_version', { version: infoStore.newVersion }) }}
          </n-tag>
        </n-space>
      </template>
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="24" class="card-icon">
              <settings-outline />
            </n-icon>
            {{ t('settings.kernel_management') }}
          </n-h3>
        </div>
      </template>

      <n-space vertical :size="20">
        <n-alert
          v-if="hasNewVersion"
          type="warning"
          :show-icon="true"
          :title="t('settings.new_version_alert_title')"
          class="version-alert"
        >
          {{ t('settings.new_version_alert_content') }}
        </n-alert>

        <n-alert
          v-if="!infoStore.version.version"
          type="error"
          :show-icon="true"
          :title="t('settings.kernel_not_installed_alert_title')"
          class="version-alert"
        >
          {{ t('settings.kernel_not_installed_alert_content') }}
        </n-alert>

        <n-progress
          v-if="downloading"
          type="line"
          :percentage="downloadProgress"
          :processing="downloadProgress < 100"
          indicator-placement="inside"
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
            <!-- Выбираем текст кнопки по условию -->
            <template v-if="hasNewVersion">
              {{ t('settings.download_new_version') }}
            </template>
            <template v-else-if="infoStore.version.version">
              {{ t('settings.redownload_current') }}
            </template>
            <template v-else>
              {{ t('settings.download_kernel') }}
            </template>
          </n-button>

          <n-space :size="16">
            <n-button
              text
              size="medium"
              @click="showManualDownloadModal"
              :disabled="downloading"
              class="action-button"
            >
              {{ t('settings.manual_download') }}
            </n-button>
            <n-button text size="medium" @click="checkManualInstall" :disabled="downloading">
              {{ t('settings.check_installation') }}
            </n-button>
          </n-space>
        </n-space>

        <n-alert v-if="downloadError" type="error" :show-icon="true" style="margin-top: 16px">
          <template #header>
            {{ t('settings.download_failed') }}
          </template>
          <div style="white-space: pre-line">{{ downloadError }}</div>
        </n-alert>
      </n-space>
    </n-card>

    <!-- 启动设置卡片 (Startup Settings Card) -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <power-outline />
            </n-icon>
            {{ t('settings.startup_settings') }}
          </n-h3>
        </div>
      </template>

      <n-list>
        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ t('settings.auto_start_app') }}</div>
              <div class="setting-desc">
                {{ appStore.autoStartApp ? t('settings.auto_start_app_desc_true') : t('settings.auto_start_app_desc_false') }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartApp" @update-value="onAutoStartChange">
              <template #checked>{{ t('settings.switch_on') }}</template>
              <template #unchecked>{{ t('settings.switch_off') }}</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ t('settings.auto_start_kernel') }}</div>
              <div class="setting-desc">
                {{ appStore.autoStartKernel ? t('settings.auto_start_kernel_desc_true') : t('settings.auto_start_kernel_desc_false') }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartKernel">
              <template #checked>{{ t('settings.switch_on') }}</template>
              <template #unchecked>{{ t('settings.switch_off') }}</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">{{ t('settings.ipv6_preferred') }}</div>
              <div class="setting-desc">
                {{ appStore.preferIpv6 ? t('settings.ipv6_preferred_desc_true') : t('settings.ipv6_preferred_desc_false') }}
              </div>
            </div>
            <n-switch v-model:value="appStore.preferIpv6" @update-value="onIpVersionChange">
              <template #checked>{{ t('settings.switch_on') }}</template>
              <template #unchecked>{{ t('settings.switch_off') }}</template>
            </n-switch>
          </n-space>
        </n-list-item>
      </n-list>
    </n-card>

    <!-- 关于卡片 (About Card) -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <information-circle-outline />
            </n-icon>
            {{ t('settings.about') }}
          </n-h3>
        </div>
      </template>

      <n-grid :cols="2" :x-gap="12" :y-gap="8">
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('settings.app_version') }}</span>
            <n-space align="center">
              <span class="about-value">{{ appStore.appVersion }}</span>
              <n-button text size="tiny" @click="handleCheckUpdate" :loading="checkingUpdate">
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                {{ t('settings.check_update') }}
              </n-button>
            </n-space>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('settings.kernel_version') }}</span>
            <span class="about-value">{{ infoStore.version.version }}</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('settings.system') }}</span>
            <span class="about-value">Windows</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">{{ t('settings.license') }}</span>
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
            {{ t('settings.github') }}
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
            {{ t('settings.official_website') }}
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
import { enable, disable } from '@tauri-apps/plugin-autostart'
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
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
const message = useMessage()
const dialog = useDialog()
const appStore = useAppStore()
const infoStore = useInfoStore()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')

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
    console.error('检查更新失败:', error)
  }
}

// 处理更新
const handleUpdate = async () => {
  try {
    await tauriApi.update.downloadAndInstallUpdate(downloadUrl.value)
  } catch (error) {
    message.error(t('settings.update_fail', { error: error }))
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
      message.success(t('settings.new_version_found', { version: result.latest_version }))
    } else {
      message.info(t('settings.up_to_date'))
    }
  } catch (error) {
    message.error(t('settings.update_fail', { error: error }))
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
    downloadMessage.value = t('settings.download_prepare')
    downloadError.value = null

    await tauriApi.subscription.downloadLatestKernel()

    // 下载 успешно – обновляем версию
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
    if (value) {
      await enable()
      message.success(t('settings.auto_start_enabled'))
    } else {
      await disable()
      message.success(t('settings.auto_start_disabled'))
    }
  } catch (error) {
    message.error(t('settings.auto_start_fail', { error: error }))
    appStore.autoStartApp = !value
  }
}

const onIpVersionChange = async (value: boolean) => {
  try {
    await tauriApi.proxy.toggleIpVersion(value)
    if (appStore.isRunning) {
      await tauriApi.kernel.restartKernel()
    }
  } catch (error: unknown) {
    message.error(t('settings.ip_version_fail', { error: error instanceof Error ? error.message : String(error) }))
    appStore.preferIpv6 = !value
  }
}

const showManualDownloadModal = () => {
  dialog.info({
    title: t('settings.manual_download_title'),
    content: t('settings.manual_download_content'),
    positiveText: t('settings.positive_text')
  })
}

const checkManualInstall = async () => {
  try {
    loading.value = true
    const success = await infoStore.checkKernelVersion()
    if (success) {
      message.success(t('settings.manual_install_success'))
    } else {
      message.error(t('settings.manual_install_fail'))
    }
  } catch (error) {
    message.error(t('settings.manual_install_check_fail', { error: error }))
  } finally {
    loading.value = false
  }
}

const getAppDataPath = async () => {
  try {
    appDataPath.value = await appDataDir()
  } catch (error) {
    console.error('获取应用数据目录失败:', error)
  }
}

listen(
  'download-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    downloadProgress.value = progress
    downloadMessage.value = msg

    if (status === 'completed') {
      downloading.value = false
      downloadError.value = null
      message.success(t('settings.download_complete'))
      infoStore.updateVersion()
    }
  },
)

onMounted(async () => {
  await appStore.fetchAppVersion()
  await checkUpdate()
  await getAppDataPath()
  await infoStore.updateVersion()
})
</script>

<style scoped>
/* Стили остаются без изменений */
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
