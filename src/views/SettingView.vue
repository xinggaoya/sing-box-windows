<template>
  <div class="setting-view">
    <!-- 主要内容区 -->
    <div class="page-container">
      <!-- 页面标题栏 -->
      <div class="page-header">
        <div class="header-content">
          <div class="title-section">
            <h1 class="page-title">设置</h1>
            <div class="title-divider"></div>
          </div>
        </div>
      </div>

      <!-- 设置内容区 -->
      <div class="settings-grid">
        <!-- 内核管理卡片 -->
        <div class="setting-card kernel-card">
          <div class="card-header">
            <div class="header-left">
              <div class="card-icon kernel-icon">
                <n-icon size="24"><settings-outline /></n-icon>
              </div>
              <div class="header-info">
                <h3 class="card-title">{{ t('setting.kernel.title') }}</h3>
                <div class="version-tags">
                  <n-tag
                    v-if="kernelStore.version.version"
                    type="success"
                    size="small"
                    round
                    class="version-tag"
                  >
                    {{ formatVersion(kernelStore.version.version) }}
                  </n-tag>
                  <n-tag v-else type="error" size="small" round class="version-tag">
                    {{ t('setting.kernel.notInstalled') }}
                  </n-tag>
                  <n-tag v-if="hasNewVersion" type="warning" size="small" round class="version-tag">
                    {{ t('setting.kernel.newVersion') }}{{ formatVersion(kernelStore.newVersion) }}
                  </n-tag>
                </div>
              </div>
            </div>
          </div>

          <div class="card-content">
            <n-alert
              v-if="hasNewVersion"
              type="warning"
              :show-icon="true"
              :title="t('setting.kernel.newVersionFound')"
              class="status-alert"
            >
              {{ t('setting.kernel.updateTip') }}
            </n-alert>

            <n-alert
              v-if="!kernelStore.version.version"
              type="error"
              :show-icon="true"
              :title="t('setting.kernel.notInstalled')"
              class="status-alert"
            >
              {{ t('setting.kernel.installPrompt') }}
            </n-alert>

            <n-progress
              v-if="downloading"
              type="line"
              :percentage="downloadProgress"
              :processing="downloadProgress < 100"
              :indicator-placement="'inside'"
              class="download-progress"
            >
              {{ downloadMessage }}
            </n-progress>

            <div class="action-section">
              <n-button
                type="primary"
                @click="downloadTheKernel"
                :loading="loading"
                :disabled="downloading"
                size="large"
                class="primary-action-btn"
                round
              >
                <template #icon>
                  <n-icon><download-outline /></n-icon>
                </template>
                {{
                  hasNewVersion
                    ? t('setting.kernel.downloadNew')
                    : kernelStore.version.version
                      ? t('setting.kernel.redownload')
                      : t('setting.kernel.download')
                }}
              </n-button>

              <div class="secondary-actions">
                <n-button
                  quaternary
                  @click="showManualDownloadModal"
                  :disabled="downloading"
                  class="secondary-action-btn"
                >
                  {{ t('setting.kernel.manualDownload') }}
                </n-button>
                <n-button
                  quaternary
                  @click="checkManualInstall"
                  :disabled="downloading"
                  class="secondary-action-btn"
                >
                  {{ t('setting.kernel.checkInstall') }}
                </n-button>
              </div>
            </div>

            <n-alert v-if="downloadError" type="error" :show-icon="true" class="status-alert">
              <template #header>{{ t('setting.kernel.downloadFailed') }}</template>
              <div style="white-space: pre-line">{{ downloadError }}</div>
            </n-alert>
          </div>
        </div>

        <!-- 启动设置卡片 -->
        <div class="setting-card">
          <div class="card-header">
            <div class="header-left">
              <div class="card-icon startup-icon">
                <n-icon size="24"><power-outline /></n-icon>
              </div>
              <div class="header-info">
                <h3 class="card-title">{{ t('setting.startup.title') }}</h3>
              </div>
            </div>
          </div>

          <div class="card-content">
            <div class="setting-list">
              <div class="setting-item">
                <div class="setting-info">
                  <div class="setting-name">{{ t('setting.autoStart.app') }}</div>
                  <div class="setting-description">
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
                  size="large"
                  class="setting-switch"
                >
                  <template #checked>{{ t('common.on') }}</template>
                  <template #unchecked>{{ t('common.off') }}</template>
                </n-switch>
              </div>

              <div class="setting-item">
                <div class="setting-info">
                  <div class="setting-name">{{ t('setting.autoStart.kernel') }}</div>
                  <div class="setting-description">
                    {{
                      appStore.autoStartKernel
                        ? t('setting.startup.autoKernelTip')
                        : t('setting.startup.manualKernelTip')
                    }}
                  </div>
                </div>
                <n-switch
                  v-model:value="appStore.autoStartKernel"
                  size="large"
                  class="setting-switch"
                >
                  <template #checked>{{ t('common.on') }}</template>
                  <template #unchecked>{{ t('common.off') }}</template>
                </n-switch>
              </div>
            </div>
          </div>
        </div>

        <!-- 常规设置卡片 -->
        <div class="setting-card">
          <div class="card-header">
            <div class="header-left">
              <div class="card-icon general-icon">
                <n-icon size="24"><globe-outline /></n-icon>
              </div>
              <div class="header-info">
                <h3 class="card-title">{{ t('setting.general.title') }}</h3>
              </div>
            </div>
          </div>

          <div class="card-content">
            <div class="setting-list">
              <div class="setting-item">
                <div class="setting-info">
                  <div class="setting-name">{{ $t('setting.language.title') }}</div>
                </div>
                <n-select
                  v-model:value="localeStore.locale"
                  :options="languageOptions"
                  size="large"
                  class="setting-select"
                  @update:value="handleChangeLanguage"
                />
              </div>

              <div class="setting-item">
                <div class="setting-info">
                  <div class="setting-name">{{ t('setting.network.ipv6') }}</div>
                  <div class="setting-description">
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
                  size="large"
                  class="setting-switch"
                >
                  <template #checked>{{ t('common.on') }}</template>
                  <template #unchecked>{{ t('common.off') }}</template>
                </n-switch>
              </div>

              <div class="setting-item">
                <div class="setting-info">
                  <div class="setting-name">{{ t('setting.network.ports') }}</div>
                  <div class="setting-description">
                    {{ t('setting.network.portsDesc') }}
                  </div>
                </div>
                <n-button size="large" @click="showPortSettings" class="config-btn">
                  {{ t('setting.network.configure') }}
                </n-button>
              </div>
            </div>
          </div>
        </div>

        <!-- 关于信息卡片 -->
        <div class="setting-card about-card">
          <div class="card-header">
            <div class="header-left">
              <div class="card-icon about-icon">
                <n-icon size="24"><information-circle-outline /></n-icon>
              </div>
              <div class="header-info">
                <h3 class="card-title">{{ t('setting.about.title') }}</h3>
              </div>
            </div>
          </div>

          <div class="card-content">
            <div class="about-grid">
              <div class="about-item">
                <div class="about-info">
                  <div class="about-label">{{ t('setting.about.appVersion') }}</div>
                  <div class="about-value">{{ updateStore.appVersion }}</div>
                </div>
                <n-button
                  ghost
                  @click="handleCheckUpdate"
                  :loading="checkingUpdate"
                  class="update-btn"
                >
                  <template #icon>
                    <n-icon><refresh-outline /></n-icon>
                  </template>
                  {{ t('setting.update.check') }}
                </n-button>
              </div>

              <div class="about-item">
                <div class="about-info">
                  <div class="about-label">{{ t('setting.about.kernelVersion') }}</div>
                  <div class="about-value">{{ formatVersion(kernelStore.version.version) }}</div>
                </div>
              </div>

              <div class="about-item">
                <div class="about-info">
                  <div class="about-label">{{ t('setting.about.system') }}</div>
                  <div class="about-value">Windows</div>
                </div>
              </div>

              <div class="about-item">
                <div class="about-info">
                  <div class="about-label">{{ t('setting.about.license') }}</div>
                  <div class="about-value">MIT License</div>
                </div>
              </div>
            </div>

            <div class="about-footer">
              <div class="footer-links">
                <n-button
                  text
                  tag="a"
                  href="https://github.com/xinggaoya/sing-box-windows"
                  target="_blank"
                  class="footer-link"
                >
                  <template #icon>
                    <n-icon><logo-github /></n-icon>
                  </template>
                  GitHub
                </n-button>
                <div class="divider"></div>
                <n-button
                  text
                  tag="a"
                  href="https://github.com/xinggaoya/sing-box-windows"
                  target="_blank"
                  class="footer-link"
                >
                  <template #icon>
                    <n-icon><globe-outline /></n-icon>
                  </template>
                  {{ t('setting.about.website') }}
                </n-button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 回到顶部 -->
    <n-back-top :right="24" :bottom="24" @click="scrollToTop">
      <div class="back-top-button">
        <n-icon size="20"><chevron-up-outline /></n-icon>
      </div>
    </n-back-top>
  </div>

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
import { ref, computed, onMounted, onUnmounted } from 'vue'
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

// 手动检查更新
const handleCheckUpdate = async () => {
  try {
    checkingUpdate.value = true
    const result = await updateStore.checkUpdate(false)
    if (result?.has_update) {
      // 发送全局更新弹窗事件
      mitt.emit('show-update-modal', {
        show: true,
        latestVersion: result.latest_version,
        currentVersion: updateStore.appVersion,
        downloadUrl: result.download_url,
        releaseNotes: result.release_notes || '',
        releaseDate: result.release_date || '',
        fileSize: result.file_size || 0,
      })
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

// 监听应用更新进度事件
listen(
  'update-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    updateStore.updateProgress(status, progress, msg)
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
  handleCheckUpdate()
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

// 清理事件监听器
onUnmounted(() => {
  window.removeEventListener('resize', updateMobileStatus)
})
</script>

<style scoped>
.setting-view {
  min-height: 100vh;
  background: var(--n-color-embedded);
  padding: 0;
}

.page-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 页面标题栏 */
.page-header {
  background: var(--n-card-color);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 16px 24px;
  box-shadow: var(--n-box-shadow-2);
  border: 1px solid var(--n-border-color);
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.page-title {
  font-size: 1.5rem;
  font-weight: 700;
  margin: 0;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
  background-clip: text;
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.title-divider {
  width: 3px;
  height: 24px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
  border-radius: 2px;
}

/* 设置网格 */
.settings-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 16px;
}

/* 设置卡片 */
.setting-card {
  background: var(--n-card-color);
  backdrop-filter: blur(20px);
  border-radius: 16px;
  padding: 0;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  overflow: hidden;
}

.setting-card:hover {
  transform: translateY(-4px);
  box-shadow:
    0 16px 48px rgba(0, 0, 0, 0.12),
    0 4px 8px rgba(0, 0, 0, 0.08);
}

.kernel-card {
  position: relative;
  overflow: hidden;
}

.kernel-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 4px;
  background: linear-gradient(90deg, #4080ff 0%, #2266dd 50%, #009a1a 100%);
}

/* 卡片头部 */
.card-header {
  padding: 16px 24px 12px;
  border-bottom: 1px solid var(--n-border-color);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-icon {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 600;
}

.kernel-icon {
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  box-shadow: 0 8px 24px rgba(64, 128, 255, 0.3);
}

.startup-icon {
  background: linear-gradient(135deg, #00b42a 0%, #009a1a 100%);
  box-shadow: 0 8px 24px rgba(0, 180, 42, 0.3);
}

.general-icon {
  background: linear-gradient(135deg, #909399 0%, #7b7e83 100%);
  box-shadow: 0 8px 24px rgba(144, 147, 153, 0.3);
}

.about-icon {
  background: linear-gradient(135deg, #ff7d00 0%, #d66600 100%);
  box-shadow: 0 8px 24px rgba(255, 125, 0, 0.3);
}

.header-info {
  flex: 1;
}

.card-title {
  font-size: 1.125rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0 0 6px 0;
}

.version-tags {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.version-tag {
  font-weight: 600;
  font-size: 0.75rem;
  padding: 4px 10px;
}

/* 卡片内容 */
.card-content {
  padding: 16px 24px 20px;
}

/* 状态提醒 */
.status-alert {
  margin-bottom: 12px;
  border-radius: 10px;
  border: 1px solid var(--n-border-color);
}

/* 下载进度 */
.download-progress {
  margin-bottom: 16px;
  height: 36px;
  border-radius: 18px;
  overflow: hidden;
  background: var(--n-color-embedded);
}

/* 操作区域 */
.action-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.primary-action-btn {
  font-weight: 600;
  height: 40px;
  padding: 0 20px;
  box-shadow:
    0 6px 24px rgba(64, 128, 255, 0.25),
    0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.primary-action-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow:
    0 12px 40px rgba(64, 128, 255, 0.4),
    0 4px 8px rgba(0, 0, 0, 0.15);
}

.secondary-actions {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.secondary-action-btn {
  font-weight: 500;
  transition: all 0.3s ease;
}

.secondary-action-btn:hover:not(:disabled) {
  color: var(--n-color-primary);
  transform: translateY(-1px);
}

/* 设置列表 */
.setting-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 0;
  border-bottom: 1px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.setting-item:last-child {
  border-bottom: none;
}

.setting-item:hover {
  background: var(--n-color-embedded-popover);
  padding-left: 12px;
  padding-right: 12px;
  margin-left: -12px;
  margin-right: -12px;
  border-radius: 10px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin-bottom: 4px;
}

.setting-description {
  font-size: 0.875rem;
  color: var(--n-text-color-3);
  line-height: 1.4;
}

.setting-switch {
  flex-shrink: 0;
  margin-left: 16px;
}

.setting-select {
  flex-shrink: 0;
  min-width: 120px;
  max-width: 130px;
  margin-left: 16px;
}

.setting-select :deep(.n-base-selection) {
  border-radius: 12px;
  border: 2px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.setting-select :deep(.n-base-selection:hover) {
  border-color: var(--n-color-primary);
}

.setting-select :deep(.n-base-selection.n-base-selection--focus) {
  border-color: var(--n-color-primary);
  box-shadow: 0 0 0 3px var(--n-color-primary-hover);
}

.config-btn {
  flex-shrink: 0;
  margin-left: 16px;
  border: 2px solid var(--n-border-color);
  border-radius: 10px;
  transition: all 0.3s ease;
}

.config-btn:hover {
  border-color: var(--n-color-primary);
  transform: translateY(-1px);
}

/* 关于信息 */
.about-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.about-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px;
  background: var(--n-color-embedded-popover);
  border-radius: 12px;
  border: 1px solid var(--n-border-color);
  transition: all 0.3s ease;
}

.about-item:hover {
  transform: translateY(-2px);
  background: var(--n-card-color);
  box-shadow: var(--n-box-shadow-2);
}

.about-info {
  flex: 1;
  min-width: 0;
}

.about-label {
  font-size: 0.75rem;
  font-weight: 500;
  color: var(--n-text-color-3);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.about-value {
  font-size: 1rem;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.update-btn {
  margin-left: 10px;
  border: 2px solid var(--n-border-color);
  border-radius: 10px;
  transition: all 0.3s ease;
}

.update-btn:hover {
  border-color: var(--n-color-primary);
  transform: translateY(-1px);
}

/* 页脚 */
.about-footer {
  padding-top: 16px;
  border-top: 1px solid var(--n-border-color);
}

.footer-links {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.footer-link {
  font-weight: 500;
  transition: all 0.3s ease;
}

.footer-link:hover {
  color: var(--n-color-primary);
  transform: translateY(-1px);
}

.divider {
  width: 1px;
  height: 16px;
  background: var(--n-border-color);
}

/* 回到顶部 */
.back-top-button {
  width: 44px;
  height: 44px;
  border-radius: 22px;
  background: linear-gradient(135deg, #4080ff 0%, #2266dd 100%);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow:
    0 6px 24px rgba(64, 128, 255, 0.25),
    0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.back-top-button:hover {
  transform: translateY(-3px);
  box-shadow:
    0 12px 40px rgba(64, 128, 255, 0.4),
    0 4px 8px rgba(0, 0, 0, 0.15);
}

/* 端口设置表单 */
.port-settings-form {
  padding: 12px 0;
}

/* 深色模式样式会通过CSS变量自动应用 */

/* 文本颜色和边框会通过CSS变量自动适配暗色模式 */

/* 响应式设计 */
@media (max-width: 768px) {
  .page-container {
    padding: 12px 8px;
    gap: 12px;
  }

  .page-header {
    padding: 12px 16px;
    border-radius: 12px;
  }

  .page-title {
    font-size: 1.25rem;
  }

  .card-header {
    padding: 12px 16px 8px;
  }

  .header-left {
    gap: 8px;
  }

  .card-icon {
    width: 36px;
    height: 36px;
  }

  .card-title {
    font-size: 1rem;
  }

  .card-content {
    padding: 12px 16px 16px;
  }

  .setting-item {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
    padding: 12px 0;
  }

  .setting-switch,
  .setting-select,
  .config-btn {
    margin-left: 0;
    align-self: flex-end;
  }

  .setting-select {
    min-width: 140px;
  }

  .about-grid {
    grid-template-columns: 1fr;
    gap: 10px;
  }

  .about-item {
    padding: 12px;
    border-radius: 10px;
  }

  .footer-links {
    gap: 8px;
  }

  .action-section {
    gap: 8px;
  }

  .secondary-actions {
    flex-direction: column;
    gap: 6px;
  }
}

@media (max-width: 480px) {
  .page-container {
    padding: 12px 8px;
  }

  .card-header {
    padding: 16px 20px 8px;
  }

  .card-content {
    padding: 16px 20px 20px;
  }

  .setting-card {
    border-radius: 16px;
  }

  .primary-action-btn {
    height: 44px;
  }
}

/* 动画效果 */
@keyframes slide-up {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.setting-view {
  animation: slide-up 0.4s ease;
}

/* 开关和按钮的增强样式 */
:deep(.n-switch) {
  transition: all 0.3s ease;
}

:deep(.n-switch:hover) {
  transform: scale(1.05);
}

:deep(.n-button) {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

:deep(.n-button:not(.n-button--disabled):hover) {
  transform: translateY(-1px);
}
</style>
