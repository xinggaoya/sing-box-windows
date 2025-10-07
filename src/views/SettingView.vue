<template>
  <div class="ultra-settings">
    <!-- 紧凑工具栏 -->
    <div class="settings-toolbar">
      <div class="toolbar-left">
        <div class="toolbar-icon">
          <n-icon size="16">
            <SettingsOutline />
          </n-icon>
        </div>
        <div class="toolbar-info">
          <span class="toolbar-title">{{ t('setting.title') }}</span>
          <span class="toolbar-stats">{{ t('setting.subtitle') }}</span>
        </div>
      </div>

      <div class="toolbar-right">
        <div class="version-info">
          <span class="version-label">{{ t('setting.appVersion') }}</span>
          <span class="version-value">{{ updateStore.appVersion }}</span>
        </div>
        <div class="version-divider"></div>
        <div class="version-info">
          <span class="version-label">{{ t('setting.kernelVersion') }}</span>
          <span class="version-value">{{
            formatVersion(kernelStore.version.version) || t('setting.notInstalled')
          }}</span>
        </div>
      </div>
    </div>

    <!-- 设置内容区 -->
    <div class="settings-content">
      <!-- 内核管理区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon kernel-icon">
            <n-icon size="18"><SettingsOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.kernel.title') }}</h2>
          </div>
          <div class="section-status">
            <n-tag
              v-if="kernelStore.version.version"
              type="success"
              size="small"
              round
            >
              {{ formatVersion(kernelStore.version.version) }}
            </n-tag>
            <n-tag v-else type="error" size="small" round>
              {{ t('setting.notInstalled') }}
            </n-tag>
            <n-tag v-if="hasNewVersion" type="warning" size="small" round>
              {{ t('setting.newVersionAvailable') }}
            </n-tag>
          </div>
        </div>

        <div class="section-content">
          <!-- 状态提醒 -->
          <div v-if="hasNewVersion || !kernelStore.version.version" class="status-alerts">
            <n-alert
              v-if="hasNewVersion"
              type="warning"
              :show-icon="false"
              size="small"
              class="compact-alert"
            >
              {{ t('setting.newVersionFound') }}
            </n-alert>

            <n-alert
              v-if="!kernelStore.version.version"
              type="error"
              :show-icon="false"
              size="small"
              class="compact-alert"
            >
              {{ t('setting.installPrompt') }}
            </n-alert>
          </div>

          <!-- 下载进度 -->
          <div v-if="downloading" class="download-area">
            <n-progress
              type="line"
              :percentage="downloadProgress"
              :processing="downloadProgress < 100"
              :indicator-placement="'inside'"
              size="small"
              class="download-progress"
            >
              {{ downloadMessage }}
            </n-progress>
          </div>

          <!-- 操作按钮区 -->
          <div class="action-buttons">
            <n-button
              type="primary"
              @click="downloadTheKernel"
              :loading="loading"
              :disabled="downloading"
              size="small"
              class="primary-action"
            >
              <template #icon>
                <n-icon size="14"><DownloadOutline /></n-icon>
              </template>
              {{
                hasNewVersion ? t('setting.update') : kernelStore.version.version ? t('setting.redownload') : t('setting.download')
              }}
            </n-button>

            <n-button
              @click="showManualDownloadModal"
              :disabled="downloading"
              size="small"
              class="secondary-action"
            >
              {{ t('setting.manualDownload') }}
            </n-button>

            <n-button
              @click="checkManualInstall"
              :disabled="downloading"
              size="small"
              class="secondary-action"
            >
              {{ t('setting.checkInstall') }}
            </n-button>
          </div>

          <!-- 错误提示 -->
          <n-alert v-if="downloadError" type="error" :show-icon="false" size="small" class="compact-alert">
            {{ downloadError }}
          </n-alert>
        </div>
      </section>

      <!-- 启动设置区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon startup-icon">
            <n-icon size="18"><PowerOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.startup.title') }}</h2>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-grid">
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.autoStart.app') }}</div>
              </div>
              <n-switch
                v-model:value="autoStart"
                @update-value="onAutoStartChange"
                size="small"
              />
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.autoStart.kernel') }}</div>
              </div>
              <n-switch
                v-model:value="appStore.autoStartKernel"
                size="small"
              />
            </div>
          </div>
        </div>
      </section>

      <!-- 常规设置区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon general-icon">
            <n-icon size="18"><GlobeOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.general.title') }}</h2>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-grid">
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ $t('setting.language.title') }}</div>
                <div class="setting-desc">{{ $t('setting.language.description') }}</div>
              </div>
              <n-select
                v-model:value="localeStore.locale"
                :options="languageOptions"
                size="small"
                @update:value="handleChangeLanguage"
                class="setting-select"
              />
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.network.ipv6') }}</div>
              </div>
              <n-switch
                v-model:value="appStore.preferIpv6"
                @update-value="onIpVersionChange"
                size="small"
              />
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.network.ports') }}</div>
                <div class="setting-desc">{{ t('setting.network.portsDesc') }}</div>
              </div>
              <n-button
                size="small"
                @click="showPortSettings"
                class="setting-button"
              >
                {{ t('setting.network.configure') }}
              </n-button>
            </div>
          </div>
        </div>
      </section>

      <!-- 更新设置区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon update-icon">
            <n-icon size="18"><RefreshOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.update.title') }}</h2>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-grid">
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.update.autoCheck') }}</div>
              </div>
              <n-switch
                v-model:value="updateStore.autoCheckUpdate"
                size="small"
              />
            </div>

            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.update.acceptPrerelease') }}</div>
              </div>
              <n-switch
                v-model:value="updateStore.acceptPrerelease"
                size="small"
                @update-value="onPrereleaseSettingChange"
              />
            </div>
          </div>

          <n-alert
            v-if="updateStore.acceptPrerelease"
            type="warning"
            :show-icon="false"
            size="small"
            class="compact-alert"
          >
            {{ t('setting.update.prereleaseWarningDesc') }}
          </n-alert>
        </div>
      </section>

      <!-- 开发者工具区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon developer-icon">
            <n-icon size="18"><CodeOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.developer.title') }}</h2>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-grid">
            <div class="setting-item">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.developer.openDevtools') }}</div>
                <div class="setting-desc">{{ t('setting.developer.description') }}</div>
              </div>
              <n-button
                size="small"
                @click="handleOpenDevtools"
                :loading="devtoolsLoading"
                class="setting-button"
              >
                {{ t('setting.developer.openDevtools') }}
              </n-button>
            </div>
          </div>

          <n-alert type="info" :show-icon="false" size="small" class="compact-alert">
            {{ t('setting.developer.warning') }}
          </n-alert>
        </div>
      </section>

      <!-- 关于信息区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon about-icon">
            <n-icon size="18"><InformationCircleOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.about.title') }}</h2>
          </div>
        </div>

        <div class="section-content">
          <div class="info-grid">
            <div class="info-item">
              <div class="info-label">{{ t('setting.appVersion') }}</div>
              <div class="info-value">{{ updateStore.appVersion }}</div>
            </div>

            <div class="info-item">
              <div class="info-label">{{ t('setting.kernelVersion') }}</div>
              <div class="info-value">
                {{ formatVersion(kernelStore.version.version) || t('setting.notInstalled') }}
              </div>
            </div>

            <div class="info-item">
              <div class="info-label">{{ t('setting.about.system') }}</div>
              <div class="info-value">Windows</div>
            </div>

            <div class="info-item">
              <div class="info-label">{{ t('setting.about.license') }}</div>
              <div class="info-value">MIT License</div>
            </div>
          </div>

          <div class="links-area">
            <n-button
              text
              tag="a"
              href="https://github.com/xinggaoya/sing-box-windows"
              target="_blank"
              size="small"
              class="link-button"
            >
              <template #icon>
                <n-icon size="14"><LogoGithub /></n-icon>
              </template>
              GitHub
            </n-button>
          </div>
        </div>
      </section>
    </div>
  </div>

  <!-- 端口设置对话框 -->
  <n-modal
    v-model:show="showPortModal"
    preset="dialog"
    :title="t('setting.network.portSettings')"
    class="modern-modal"
  >
    <div class="port-settings-form">
      <n-form :model="{ proxyPort: tempProxyPort, apiPort: tempApiPort }" size="large">
        <n-form-item :label="t('setting.network.proxyPort')">
          <n-input-number
            v-model:value="tempProxyPort"
            :min="1024"
            :max="65535"
            class="port-input"
          />
        </n-form-item>
        <n-form-item :label="t('setting.network.apiPort')">
          <n-input-number v-model:value="tempApiPort" :min="1024" :max="65535" class="port-input" />
        </n-form-item>
      </n-form>
    </div>

    <template #action>
      <n-space size="large">
        <n-button @click="showPortModal = false" size="large">
          {{ t('common.cancel') }}
        </n-button>
        <n-button
          type="primary"
          :loading="portSettingsLoading"
          @click="savePortSettings"
          size="large"
        >
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
  CodeOutline,
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
const autoStart = ref(false)
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

onMounted(async () => {
  autoStart.value = await isEnabled()
})

// 检查更新状态
const checkingUpdate = ref(false)

// 下载状态
const downloadError = ref<string | null>(null)
const appDataPath = ref('')

// 语言选项
const languageOptions = [
  { label: t('setting.language.auto'), value: 'auto' as Locale },
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

    await tauriApi.system.downloadLatestKernel(window)

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
    await tauriApi.config.updateSingboxPorts(tempProxyPort.value, tempApiPort.value)

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

// 打开开发者工具
const devtoolsLoading = ref(false)

const handleOpenDevtools = async () => {
  try {
    devtoolsLoading.value = true
    await tauriApi.system.openDevtools()
    message.success(t('setting.developer.opened'))
  } catch (error) {
    message.error(`${t('common.error')}: ${error}`)
  } finally {
    devtoolsLoading.value = false
  }
}

// 预发布版本设置变更处理
const onPrereleaseSettingChange = (value: boolean) => {
  if (value) {
    dialog.warning({
      title: t('setting.update.prereleaseConfirm'),
      content: t('setting.update.prereleaseConfirmDesc'),
      positiveText: t('common.confirm'),
      negativeText: t('common.cancel'),
      onPositiveClick: () => {
        updateStore.acceptPrerelease = true
        notification.success({
          title: t('setting.update.prereleaseEnabled'),
          content: t('setting.update.prereleaseEnabledDesc'),
          duration: 3000,
        })
      },
      onNegativeClick: () => {
        updateStore.acceptPrerelease = false
      },
    })
  } else {
    notification.info({
      title: t('setting.update.prereleaseDisabled'),
      content: t('setting.update.prereleaseDisabledDesc'),
      duration: 3000,
    })
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
.ultra-settings {
  padding: 16px;
  background: var(--n-color-embedded);
  min-height: calc(100vh - 36px);
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: slideFadeIn 0.4s ease-out;
}

/* 紧凑工具栏 */
.settings-toolbar {
  background: var(--n-card-color);
  border-radius: 12px;
  padding: 12px 16px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: var(--n-box-shadow-1);
  border: 1px solid var(--n-border-color);
  backdrop-filter: blur(10px);
  -webkit-backdrop-filter: blur(10px);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.toolbar-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 12px rgba(139, 92, 237, 0.3);
}

.toolbar-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.toolbar-title {
  font-size: 1rem;
  font-weight: 600;
  color: var(--n-text-color-1);
  margin: 0;
}

.toolbar-stats {
  font-size: 0.75rem;
  color: var(--n-text-color-3);
  margin: 0;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.version-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.version-label {
  font-size: 0.7rem;
  color: var(--n-text-color-3);
  font-weight: 500;
}

.version-value {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--n-text-color-1);
}

.version-divider {
  width: 1px;
  height: 20px;
  background: var(--n-border-color);
  margin: 0 8px;
}

/* 设置内容区 */
.settings-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 设置区块 */
.settings-section {
  background: var(--n-card-color);
  border-radius: 12px;
  border: 1px solid var(--n-border-color);
  box-shadow: var(--n-box-shadow-1);
  transition: all 0.2s ease;
}

.settings-section:hover {
  box-shadow: var(--n-box-shadow-2);
  transform: translateY(-1px);
}

/* 区块头部 */
.section-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--n-border-color);
  background: rgba(0, 0, 0, 0.01);
}

.section-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.kernel-icon {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
}

.startup-icon {
  background: linear-gradient(135deg, #10b981, #059669);
}

.general-icon {
  background: linear-gradient(135deg, #8b5cf6, #7c3aed);
}

.update-icon {
  background: linear-gradient(135deg, #06b6d4, #0891b2);
}

.developer-icon {
  background: linear-gradient(135deg, #f59e0b, #d97706);
}

.about-icon {
  background: linear-gradient(135deg, #ef4444, #dc2626);
}

.section-info {
  flex: 1;
}

.section-title {
  font-size: 1rem;
  font-weight: 600;
  margin: 0;
  color: var(--n-text-color-1);
}

.section-status {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

/* 区块内容 */
.section-content {
  padding: 16px;
}

/* 状态提醒 */
.status-alerts {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
  flex-wrap: wrap;
}

.compact-alert {
  border-radius: 6px;
}

/* 下载区域 */
.download-area {
  margin-bottom: 16px;
}

.download-progress {
  height: 32px;
  border-radius: 8px;
}

/* 操作按钮区 */
.action-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.primary-action {
  font-weight: 500;
}

.secondary-action {
  font-weight: 400;
}

/* 设置网格 */
.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--n-color-embedded);
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.setting-item:hover {
  background: var(--n-color-embedded-modal);
  border-color: #8b5cf6;
  transform: translateY(-1px);
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-title {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--n-text-color-1);
}

.setting-desc {
  font-size: 0.75rem;
  color: var(--n-text-color-3);
  margin-top: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.setting-select {
  min-width: 120px;
  max-width: 150px;
}

.setting-button {
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.setting-button:hover {
  border-color: #8b5cf6;
  transform: translateY(-1px);
}

/* 信息网格 */
.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 12px;
  margin-bottom: 16px;
}

.info-item {
  padding: 16px;
  background: var(--n-color-embedded);
  border-radius: 8px;
  border: 1px solid var(--n-border-color);
  transition: all 0.2s ease;
}

.info-item:hover {
  background: var(--n-color-embedded-modal);
  border-color: #8b5cf6;
  transform: translateY(-1px);
}

.info-label {
  font-size: 0.7rem;
  color: var(--n-text-color-3);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.info-value {
  font-size: 0.875rem;
  font-weight: 600;
  color: var(--n-text-color-1);
}

/* 链接区域 */
.links-area {
  display: flex;
  align-items: center;
  justify-content: center;
  padding-top: 16px;
  border-top: 1px solid var(--n-border-color);
}

.link-button {
  font-weight: 500;
  transition: all 0.2s ease;
  color: #8b5cf6;
}

.link-button:hover {
  transform: translateY(-1px);
  color: #7c3aed;
}

/* 模态框样式 */
:deep(.n-modal) {
  border-radius: 12px;
}

.port-settings-form {
  padding: 16px 0;
}

/* 动画效果 */
@keyframes slideFadeIn {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .ultra-settings {
    padding: 12px;
    gap: 12px;
  }

  .settings-toolbar {
    padding: 10px 12px;
  }

  .toolbar-icon {
    width: 28px;
    height: 28px;
  }

  .toolbar-title {
    font-size: 0.875rem;
  }

  .toolbar-stats {
    font-size: 0.7rem;
  }

  .toolbar-right {
    gap: 8px;
  }

  .version-info {
    gap: 1px;
  }

  .version-label {
    font-size: 0.65rem;
  }

  .version-value {
    font-size: 0.75rem;
  }

  .version-divider {
    height: 16px;
    margin: 0 6px;
  }

  .section-header {
    padding: 12px;
  }

  .section-icon {
    width: 32px;
    height: 32px;
  }

  .section-title {
    font-size: 0.875rem;
  }

  .section-content {
    padding: 12px;
  }

  .settings-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .action-buttons {
    flex-direction: column;
    gap: 8px;
  }

  .primary-action,
  .secondary-action {
    width: 100%;
  }

  .info-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }
}

@media (max-width: 480px) {
  .ultra-settings {
    padding: 8px;
    gap: 8px;
  }

  .settings-toolbar {
    padding: 8px 10px;
  }

  .toolbar-left {
    gap: 8px;
  }

  .toolbar-icon {
    width: 24px;
    height: 24px;
  }

  .toolbar-title {
    font-size: 0.8rem;
  }

  .toolbar-right {
    gap: 6px;
  }

  .version-info {
    display: none;
  }

  .version-divider {
    display: none;
  }

  .section-header {
    padding: 10px;
    gap: 8px;
  }

  .section-icon {
    width: 28px;
    height: 28px;
  }

  .section-title {
    font-size: 0.8rem;
  }

  .section-content {
    padding: 10px;
  }

  .setting-item {
    padding: 10px;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
  }

  .setting-select {
    min-width: 100px;
    max-width: 100%;
  }

  .setting-info {
    text-align: center;
  }

  .setting-desc {
    white-space: normal;
    line-height: 1.3;
  }

  .info-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }
}

/* Naive UI 组件优化 */
:deep(.n-button__content) {
  font-size: 0.875rem !important;
}

:deep(.n-switch) {
  font-size: 0.875rem !important;
}

:deep(.n-select) {
  font-size: 0.875rem !important;
}

:deep(.n-alert) {
  font-size: 0.8rem !important;
}

:deep(.n-progress) {
  font-size: 0.8rem !important;
}

:deep(.n-tag) {
  font-size: 0.7rem !important;
  font-weight: 500 !important;
}
</style>
