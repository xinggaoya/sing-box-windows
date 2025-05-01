<template>
  <div class="setting-container">
    <!-- 内核管理卡片 -->
    <n-card class="setting-card" :bordered="false">
      <template #header-extra>
        <n-space align="center" :size="12">
          <n-tag
            v-if="kernelStore.version.version"
            :bordered="false"
            type="default"
            size="medium"
            class="version-tag"
          >
            {{ t('setting.kernel.currentVersion') }}{{ formatVersion(kernelStore.version.version) }}
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
            {{ t('setting.kernel.newVersion') }}{{ formatVersion(kernelStore.newVersion) }}
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
          v-if="!kernelStore.version.version"
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
                : kernelStore.version.version
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

    <!-- 服务管理卡片 -->
    <n-card class="setting-card" :bordered="false">
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <server-outline />
            </n-icon>
            服务管理
          </n-h3>
        </div>
      </template>

      <n-space vertical :size="16">
        <n-alert
          v-if="!serviceStore.isServiceInstalled"
          type="warning"
          :show-icon="true"
          title="服务未安装"
          class="version-alert"
        >
          必须安装系统服务才能使用TUN模式和更多高级功能
        </n-alert>

        <n-space justify="space-between" align="center">
          <div>
            <n-space align="center" :size="12">
              <n-tag
                :type="serviceStore.isServiceInstalled ? 'success' : 'error'"
                :bordered="false"
                size="medium"
              >
                {{ serviceStore.isServiceInstalled ? '已安装' : '未安装' }}
              </n-tag>
              <span class="service-status">系统服务状态</span>
              <n-button
                text
                size="small"
                @click="refreshServiceStatus"
                :loading="checkingService"
              >
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                刷新
              </n-button>
            </n-space>

            <div class="service-desc" v-if="serviceStore.isServiceInstalled">
              <n-space align="center" :size="12">
                <n-tag
                  :type="serviceStore.isServiceRunning ? 'success' : 'warning'"
                  :bordered="false"
                  size="small"
                >
                  {{ serviceStore.isServiceRunning ? '运行中' : '已停止' }}
                </n-tag>
                <span>服务运行状态</span>
              </n-space>
            </div>
          </div>

          <n-space>
            <n-button
              type="error"
              @click="handleUninstallService"
              :loading="serviceStore.isUninstalling"
              :disabled="!serviceStore.isServiceInstalled || !isAdmin"
            >
              卸载服务
            </n-button>
            <n-button
              type="primary"
              @click="navigateToServiceInstall"
              :disabled="serviceStore.isServiceInstalled"
            >
              {{ serviceStore.isServiceInstalled ? '已安装' : '安装服务' }}
            </n-button>
          </n-space>
        </n-space>
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
                  localeStore.locale === 'auto'
                    ? $t('setting.language.auto')
                    : supportedLocales.find((loc) => loc.code === localeStore.locale)?.name
                }}
              </div>
            </div>
            <n-select
              v-model:value="localeStore.locale"
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
              <span class="about-value">{{ updateStore.appVersion }}</span>
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
            <span class="about-value">{{ formatVersion(kernelStore.version.version) }}</span>
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
    :current-version="updateStore.appVersion"
    :download-url="downloadUrl"
    @update="handleUpdate"
    @cancel="skipUpdate"
  />
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useAppStore } from '@/stores/app/AppStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useLocaleStore } from '@/stores/app/LocaleStore'
import { useServiceStore } from '@/stores/system/ServiceStore'
import { useRouter } from 'vue-router'
import {
  DownloadOutline,
  SettingsOutline,
  PowerOutline,
  InformationCircleOutline,
  LogoGithub,
  GlobeOutline,
  RefreshOutline,
  ServerOutline,
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri-api'
import { appDataDir } from '@tauri-apps/api/path'
import UpdateModal from '@/components/UpdateModal.vue'
import { supportedLocales } from '@/locales'
import { Locale } from '@/stores/app/LocaleStore'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import { getVersion } from '@tauri-apps/api/app'
import i18n from '@/locales'

const message = useMessage()
const dialog = useDialog()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const serviceStore = useServiceStore()
const router = useRouter()
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

    const result = await tauriApi.update.checkUpdate(updateStore.appVersion)
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
    const result = await updateStore.checkUpdate(false)
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

// 格式化版本号，只显示纯版本号部分
const formatVersion = (version: string) => {
  if (!version) return ''

  // 使用正则表达式提取版本号
  // 匹配常见的版本号格式，如 1.2.3，1.2.3-beta 等
  const versionRegex = /\d+\.\d+\.\d+(?:-[\w.]+)?/
  const match = version.match(versionRegex)

  if (match) {
    return match[0]
  }

  // 如果没有匹配到版本号格式，则使用原始的处理方式
  // 如果版本号以 'sing-box version ' 开头，只保留版本号部分
  if (version.startsWith('sing-box version ')) {
    return version.split(' ')[2]
  }

  // 如果包含空格，只取第一部分（通常是版本号）
  if (version.includes(' ')) {
    return version.split(' ')[0]
  }

  return version
}

const hasNewVersion = computed(() => {
  if (!kernelStore.newVersion || !kernelStore.version.version) return false
  return formatVersion(kernelStore.newVersion) != formatVersion(kernelStore.version.version)
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
    await kernelStore.updateVersion()
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

    if (value) {
      await enable()
    } else {
      await disable()
    }

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
    const success = await kernelStore.checkKernelVersion()
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
      kernelStore.updateVersion()
    }
  },
)

// 切换语言
const handleChangeLanguage = async (value: string) => {
  localeStore.setLocale(value as Locale)
  i18n.global.locale.value = value as 'zh-CN' | 'en-US' | 'ru-RU' | 'ja-JP'
  // 发送语言变更事件，通知托盘菜单刷新
  mitt.emit('language-changed')
}

const isAdmin = ref(false)
const checkingService = ref(false)

// 检查管理员权限
async function checkAdminPermission() {
  try {
    isAdmin.value = await tauriApi.system.checkAdmin()
  } catch (error) {
    console.error('检查管理员权限失败:', error)
    isAdmin.value = false
  }
}

// 刷新服务状态
async function refreshServiceStatus() {
  try {
    checkingService.value = true
    await serviceStore.checkServiceStatus()
    message.success('服务状态刷新成功')
  } catch (error) {
    message.error(`服务状态刷新失败: ${error}`)
  } finally {
    checkingService.value = false
  }
}

// 卸载服务
async function handleUninstallService() {
  if (!isAdmin.value) {
    try {
      await tauriApi.system.restartAsAdmin()
      return
    } catch (error) {
      message.error(`以管理员身份重启失败: ${error}`)
      return
    }
  }

  try {
    dialog.warning({
      title: '卸载服务',
      content: '确定要卸载系统服务吗？卸载后将无法使用TUN模式等高级功能。',
      positiveText: '确定卸载',
      negativeText: '取消',
      onPositiveClick: async () => {
        const success = await serviceStore.uninstallService()
        if (success) {
          message.success('服务卸载成功')
        }
      }
    })
  } catch (error) {
    message.error(`服务卸载失败: ${error}`)
  }
}

// 跳转到服务安装页面
function navigateToServiceInstall() {
  router.push('/service-install')
}

onMounted(async () => {
  // 获取当前版本号
  await updateStore.fetchAppVersion()
  // 检查更新
  await checkUpdate()
  // 获取应用数据目录
  await getAppDataPath()
  // 获取内核版本信息
  await kernelStore.updateVersion()
  // 检查管理员权限
  await checkAdminPermission()
  // 检查服务状态
  await serviceStore.checkServiceStatus()
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

.service-status {
  font-size: 14px;
  font-weight: 500;
}

.service-desc {
  margin-top: 8px;
  font-size: 12px;
  color: var(--text-color-3);
}
</style>
