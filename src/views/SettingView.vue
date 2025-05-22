<template>
  <div class="setting-container">
    <!-- 内容布局使用网格 -->
    <n-grid :cols="1" :x-gap="16" :y-gap="16">
      <!-- 内核管理卡片 -->
      <n-gi>
        <n-card class="setting-card setting-card-primary feature-card" :bordered="false">
          <template #header>
            <div class="card-header">
              <n-h3 class="card-title">
                <n-icon size="22" class="card-icon">
                  <settings-outline />
                </n-icon>
                {{ t('setting.kernel.title') }}
              </n-h3>
              <n-space align="center" :size="8">
                <n-tag
                  v-if="kernelStore.version.version"
                  :bordered="false"
                  type="default"
                  size="small"
                  class="version-tag"
                >
                  {{ formatVersion(kernelStore.version.version) }}
                </n-tag>
                <n-tag v-else :bordered="false" type="error" size="small" class="version-tag">
                  {{ t('setting.kernel.notInstalled') }}
                </n-tag>
                <n-tag
                  v-if="hasNewVersion"
                  :bordered="false"
                  type="warning"
                  size="small"
                  class="version-tag"
                >
                  {{ t('setting.kernel.newVersion') }}{{ formatVersion(kernelStore.newVersion) }}
                </n-tag>
              </n-space>
            </div>
          </template>

          <div class="card-content">
            <n-alert
              v-if="hasNewVersion"
              type="warning"
              :show-icon="true"
              :title="t('setting.kernel.newVersionFound')"
              class="version-alert compact-alert"
            >
              {{ t('setting.kernel.updateTip') }}
            </n-alert>

            <n-alert
              v-if="!kernelStore.version.version"
              type="error"
              :show-icon="true"
              :title="t('setting.kernel.notInstalled')"
              class="version-alert compact-alert"
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

            <div class="action-row">
              <n-button
                type="primary"
                @click="downloadTheKernel"
                :loading="loading"
                :disabled="downloading"
                size="small"
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

              <n-space :size="8">
                <n-button
                  text
                  size="small"
                  @click="showManualDownloadModal"
                  :disabled="downloading"
                  class="action-button"
                >
                  {{ t('setting.kernel.manualDownload') }}
                </n-button>
                <n-button text size="small" @click="checkManualInstall" :disabled="downloading">
                  {{ t('setting.kernel.checkInstall') }}
                </n-button>
              </n-space>
            </div>

            <n-alert v-if="downloadError" type="error" :show-icon="true" class="compact-alert">
              <template #header> {{ t('setting.kernel.downloadFailed') }} </template>
              <div style="white-space: pre-line">{{ downloadError }}</div>
            </n-alert>
          </div>
        </n-card>
      </n-gi>

      <!-- 启动设置卡片 -->
      <n-gi>
        <n-card class="setting-card" :bordered="false">
          <template #header>
            <div class="card-header">
              <n-h3 class="card-title">
                <n-icon size="18" class="card-icon">
                  <power-outline />
                </n-icon>
                {{ t('setting.startup.title') }}
              </n-h3>
            </div>
          </template>

          <div class="card-content">
            <div class="setting-grid">
              <div class="setting-row">
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
                <n-switch
                  v-model:value="appStore.autoStartApp"
                  @update-value="onAutoStartChange"
                  size="small"
                >
                  <template #checked>{{ t('common.on') }}</template>
                  <template #unchecked>{{ t('common.off') }}</template>
                </n-switch>
              </div>

              <div class="setting-row">
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
                <n-switch v-model:value="appStore.autoStartKernel" size="small">
                  <template #checked>{{ t('common.on') }}</template>
                  <template #unchecked>{{ t('common.off') }}</template>
                </n-switch>
              </div>
            </div>
          </div>
        </n-card>
      </n-gi>

      <!-- 常规设置卡片 -->
      <n-gi>
        <n-card class="setting-card" :bordered="false">
          <template #header>
            <div class="card-header">
              <n-h3 class="card-title">
                <n-icon size="18" class="card-icon">
                  <globe-outline />
                </n-icon>
                {{ t('setting.general.title') }}
              </n-h3>
            </div>
          </template>

          <div class="card-content">
            <div class="general-settings">
              <div class="setting-row">
                <div class="setting-title">{{ $t('setting.language.title') }}</div>
                <n-select
                  v-model:value="localeStore.locale"
                  :options="languageOptions"
                  size="small"
                  class="language-select"
                  @update:value="handleChangeLanguage"
                />
              </div>

              <div class="setting-row">
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
                <n-switch
                  v-model:value="appStore.preferIpv6"
                  @update-value="onIpVersionChange"
                  size="small"
                >
                  <template #checked>{{ t('common.on') }}</template>
                  <template #unchecked>{{ t('common.off') }}</template>
                </n-switch>
              </div>

              <div class="setting-row">
                <div class="setting-item">
                  <div class="setting-title">{{ t('setting.network.ports') }}</div>
                  <div class="setting-desc">
                    {{ t('setting.network.portsDesc') }}
                  </div>
                </div>
                <n-button size="small" @click="showPortSettings">
                  {{ t('setting.network.configure') }}
                </n-button>
              </div>
            </div>
          </div>
        </n-card>
      </n-gi>

      <!-- 关于信息卡片 -->
      <n-gi>
        <n-card class="setting-card about-card" :bordered="false">
          <template #header>
            <div class="card-header">
              <n-h3 class="card-title">
                <n-icon size="18" class="card-icon">
                  <information-circle-outline />
                </n-icon>
                {{ t('setting.about.title') }}
              </n-h3>
            </div>
          </template>

          <div class="card-content">
            <n-grid :cols="isMobile ? 1 : 4" :x-gap="16" :y-gap="16">
              <n-gi>
                <div class="about-item">
                  <div class="about-content">
                    <span class="about-label">{{ t('setting.about.appVersion') }}</span>
                    <span class="about-value">{{ updateStore.appVersion }}</span>
                  </div>
                  <n-button
                    text
                    size="tiny"
                    @click="handleCheckUpdate"
                    :loading="checkingUpdate"
                    class="check-button"
                  >
                    <template #icon>
                      <n-icon><refresh-outline /></n-icon>
                    </template>
                    {{ t('setting.update.check') }}
                  </n-button>
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
                  size="small"
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
                  size="small"
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
          </div>
        </n-card>
      </n-gi>
    </n-grid>

    <!-- 回到顶部 -->
    <n-back-top :right="16" :bottom="16" @click="scrollToTop">
      <div class="back-top-btn">
        <n-icon>
          <chevron-up-outline />
        </n-icon>
      </div>
    </n-back-top>
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

  <!-- 端口设置对话框 -->
  <n-modal v-model:show="showPortModal" preset="dialog" :title="t('setting.network.portSettings')">
    <div class="port-settings-form">
      <n-form :model="{ proxyPort: tempProxyPort, apiPort: tempApiPort }">
        <n-form-item :label="t('setting.network.proxyPort')">
          <n-input-number v-model:value="tempProxyPort" :min="1024" :max="65535" />
        </n-form-item>
        <n-form-item :label="t('setting.network.apiPort')">
          <n-input-number v-model:value="tempApiPort" :min="1024" :max="65535" />
        </n-form-item>
      </n-form>
    </div>

    <template #action>
      <n-space>
        <n-button @click="showPortModal = false">
          {{ t('common.cancel') }}
        </n-button>
        <n-button type="primary" :loading="portSettingsLoading" @click="savePortSettings">
          {{ t('common.save') }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useMessage, useDialog, useNotification } from 'naive-ui'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useAppStore } from '@/stores/app/AppStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useLocaleStore } from '@/stores/app/LocaleStore'
import { useRouter } from 'vue-router'
import {
  DownloadOutline,
  SettingsOutline,
  PowerOutline,
  InformationCircleOutline,
  LogoGithub,
  GlobeOutline,
  RefreshOutline,
  ChevronUpOutline,
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri-api'
import { appDataDir } from '@tauri-apps/api/path'
import UpdateModal from '@/components/UpdateModal.vue'
import { supportedLocales } from '@/locales'
import { Locale } from '@/stores/app/LocaleStore'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import i18n from '@/locales'

const message = useMessage()
const dialog = useDialog()
const notification = useNotification()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const router = useRouter()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')
const { t } = useI18n()

// 判断是否为移动端视图
const isMobile = ref(window.innerWidth < 768)

// 监听窗口尺寸变化以更新移动端状态
const updateMobileStatus = () => {
  isMobile.value = window.innerWidth < 768
}

// 更新相关状态
const showUpdateModal = ref(false)
const latestVersion = ref('')
const downloadUrl = ref('')
const skipUpdateFlag = ref(false)

// 检查更新状态
const checkingUpdate = ref(false)

// 下载状态
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

// 滚动到顶部
const scrollToTop = () => {
  window.scrollTo({
    top: 0,
    behavior: 'smooth',
  })
}

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
    notification.info({
      title: t('setting.update.downloading'),
      content: t('setting.update.downloadingDescription'),
      duration: 3000,
    })
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
  const versionRegex = /\d+\.\d+\.\d+(?:-[\w.]+)?/
  const match = version.match(versionRegex)

  if (match) {
    return match[0]
  }

  // 如果没有匹配到版本号格式，则使用原始的处理方式
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

    notification.info({
      title: t('setting.kernel.downloading'),
      content: t('setting.kernel.downloadingDescription'),
      duration: 3000,
    })

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
    if (value) {
      await enable()
      notification.success({
        title: t('setting.startup.enabled'),
        content: t('setting.startup.enableSuccess'),
        duration: 3000,
      })
    } else {
      await disable()
      notification.info({
        title: t('setting.startup.disabled'),
        content: t('setting.startup.disableSuccess'),
        duration: 3000,
      })
    }
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
      notification.success({
        title: t('setting.network.ipVersionChanged'),
        content: value ? t('setting.network.ipv6Enabled') : t('setting.network.ipv4Only'),
        duration: 3000,
      })
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

  notification.success({
    title: t('setting.language.changed'),
    content: t('setting.language.changeSuccess'),
    duration: 3000,
  })
}

// 初始化设置页面
const initializeSettings = async () => {
  // 并行执行多个初始化操作
  await Promise.all([
    updateStore.fetchAppVersion(),
    kernelStore.updateVersion(),
    appDataDir().then((path) => (appDataPath.value = path)),
  ])

  // 检查更新（非阻塞）
  checkUpdate()
}

// 端口设置对话框
const showPortModal = ref(false)
const tempProxyPort = ref(appStore.proxyPort)
const tempApiPort = ref(appStore.apiPort)
const portSettingsLoading = ref(false)

// 显示端口设置对话框
const showPortSettings = () => {
  tempProxyPort.value = appStore.proxyPort
  tempApiPort.value = appStore.apiPort
  showPortModal.value = true
}

// 保存端口设置
const savePortSettings = async () => {
  try {
    portSettingsLoading.value = true

    // 验证端口值
    if (
      tempProxyPort.value < 1024 ||
      tempProxyPort.value > 65535 ||
      tempApiPort.value < 1024 ||
      tempApiPort.value > 65535
    ) {
      message.error(t('setting.network.invalidPort'))
      return
    }

    // 检查端口是否相同
    if (tempProxyPort.value === tempApiPort.value) {
      message.error(t('setting.network.portConflict'))
      return
    }

    // 更新端口设置
    await tauriApi.config.updatePortConfig(tempProxyPort.value, tempApiPort.value)

    // 更新应用状态
    appStore.updatePorts(tempProxyPort.value, tempApiPort.value)

    // 是否需要重启
    if (appStore.isRunning) {
      // 显示需要重启的提示
      const shouldRestart = await dialog.warning({
        title: t('setting.network.restartRequired'),
        content: t('setting.network.restartDesc'),
        positiveText: t('common.restart'),
        negativeText: t('common.later'),
      })

      if (shouldRestart) {
        await tauriApi.kernel.restartKernel()
        notification.success({
          title: t('setting.network.portChanged'),
          content: t('setting.network.portChangeSuccess'),
          duration: 3000,
        })
      }
    } else {
      notification.success({
        title: t('setting.network.portChanged'),
        content: t('setting.network.portChangeSuccess'),
        duration: 3000,
      })
    }

    showPortModal.value = false
  } catch (error) {
    message.error(`${t('common.error')}: ${error}`)
  } finally {
    portSettingsLoading.value = false
  }
}

onMounted(() => {
  // 添加窗口大小改变监听器
  window.addEventListener('resize', updateMobileStatus)

  // 初始化数据（非阻塞）
  initializeSettings()
})
</script>

<style scoped>
.setting-container {
  max-width: 960px;
  margin: 0 auto;
  padding: 20px 16px;
  display: flex;
  flex-direction: column;
  gap: 20px;
  animation: fade-in 0.4s ease;
}

@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.setting-card {
  border-radius: 16px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light, 0 2px 12px rgba(0, 0, 0, 0.08));
  overflow: hidden;
  height: 100%;
  backdrop-filter: blur(10px);
  border: 1px solid var(--border-color, rgba(239, 239, 245, 0.6));
}

.setting-card-primary {
  background: linear-gradient(135deg, var(--primary-color-fade-1, #f0f7ff), transparent);
}

.feature-card {
  min-height: 180px;
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  margin-bottom: 4px;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0;
  font-weight: 600;
  font-size: 16px;
  color: var(--n-text-color);
}

.card-icon {
  color: var(--primary-color);
}

.card-content {
  padding: 0 16px 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.compact-card {
  gap: 10px;
}

.compact-alert {
  padding: 10px;
  margin: 0;
  font-size: 13px;
  border-radius: 8px;
}

.version-tag {
  font-weight: 500;
  padding: 0 10px;
  height: 22px;
  border-radius: 11px;
}

.action-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 4px;
}

.download-progress {
  margin: 10px 0;
  height: 32px;
  font-weight: 500;
  border-radius: 16px;
  overflow: hidden;
}

.download-button {
  font-weight: 500;
  border-radius: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.download-button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px var(--primary-color-fade-3, rgba(64, 158, 255, 0.25));
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

.setting-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px;
  border-radius: 10px;
  background-color: var(--card-color, rgba(0, 0, 0, 0.01));
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.setting-row .setting-item {
  flex: 1;
  padding-right: 16px;
}

.setting-row:hover {
  background-color: var(--hover-color, rgba(0, 0, 0, 0.03));
  border-color: var(--border-color, rgba(239, 239, 245, 0.6));
  transform: translateY(-1px);
}

.setting-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.setting-title {
  font-size: 14px;
  font-weight: 500;
  width: 180px;
  min-width: max-content;
  color: var(--text-color-1);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-color-3);
}

/* 语言和网络设置 */
.general-settings {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.language-select {
  min-width: 120px;
}

.about-card {
  margin-bottom: 16px;
  background: linear-gradient(135deg, var(--card-color, rgba(255, 255, 255, 0.8)), transparent);
}

.about-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 14px;
  border-radius: 10px;
  background-color: var(--card-color, rgba(0, 0, 0, 0.01));
  transition: all 0.2s ease;
  border: 1px solid transparent;
}

.about-item:hover {
  background-color: var(--hover-color, rgba(0, 0, 0, 0.03));
  border-color: var(--border-color, rgba(239, 239, 245, 0.6));
  transform: translateY(-1px);
}

.about-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.about-label {
  color: var(--text-color-2);
  font-size: 12px;
}

.about-value {
  color: var(--text-color-1);
  font-size: 14px;
  font-weight: 500;
}

.about-footer {
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--divider-color);
  text-align: center;
}

.check-button {
  font-size: 12px;
  transition: all 0.3s ease;
}

.check-button:hover {
  transform: translateY(-1px);
  color: var(--primary-color);
}

.back-top-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 40px;
  width: 40px;
  border-radius: 20px;
  background-color: var(--primary-color);
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.back-top-btn:hover {
  transform: translateY(-3px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

/* 按钮效果 */
:deep(.n-button) {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

:deep(.n-button:not(.n-button--disabled)):hover {
  transform: translateY(-2px);
}

:deep(.n-button--primary-type:not(.n-button--disabled)):hover {
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.3);
}

:deep(.n-button--error-type:not(.n-button--disabled)):hover {
  box-shadow: 0 4px 12px rgba(255, 77, 79, 0.3);
}

:deep(.n-button--warning-type:not(.n-button--disabled)):hover {
  box-shadow: 0 4px 12px rgba(250, 173, 20, 0.3);
}

/* 移动端适配 */
@media (max-width: 768px) {
  .setting-container {
    padding: 12px 8px;
    gap: 16px;
  }

  .card-content {
    padding: 0 12px 12px;
    gap: 12px;
  }

  .card-header {
    padding: 10px 12px;
  }

  .action-row {
    flex-direction: column;
    align-items: stretch;
  }

  .about-item {
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
  }

  .check-button {
    align-self: flex-end;
  }
}
</style>
