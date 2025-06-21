<template>
  <div class="modern-settings">
    <!-- 英雄式头部 -->
    <div class="settings-hero">
      <div class="hero-content">
        <div class="hero-info">
          <div class="hero-icon">
            <n-icon size="32">
              <SettingsOutline />
            </n-icon>
          </div>
          <div class="hero-text">
            <h1 class="hero-title">{{ t('setting.title') }}</h1>
            <p class="hero-subtitle">{{ t('setting.subtitle') }}</p>
          </div>
        </div>

        <div class="hero-status">
          <div class="status-card">
            <div class="status-item">
              <span class="status-label">{{ t('setting.appVersion') }}</span>
              <span class="status-value">{{ updateStore.appVersion }}</span>
            </div>
            <div class="status-item">
              <span class="status-label">{{ t('setting.kernelVersion') }}</span>
              <span class="status-value">{{
                formatVersion(kernelStore.version.version) || t('setting.notInstalled')
              }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 设置内容区 -->
    <div class="settings-content">
      <!-- 内核管理区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon kernel-icon">
            <n-icon size="24"><SettingsOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.kernel.title') }}</h2>
            <p class="section-description">{{ t('setting.kernel.subtitle') }}</p>
          </div>
          <div class="section-status">
            <n-tag
              v-if="kernelStore.version.version"
              type="success"
              size="medium"
              round
              class="status-tag"
            >
              {{ formatVersion(kernelStore.version.version) }}
            </n-tag>
            <n-tag v-else type="error" size="medium" round class="status-tag">
              {{ t('setting.kernel.notInstalled') }}
            </n-tag>
            <n-tag v-if="hasNewVersion" type="warning" size="medium" round class="status-tag">
              {{
                t('setting.newVersionAvailable', { version: formatVersion(kernelStore.newVersion) })
              }}
            </n-tag>
          </div>
        </div>

        <div class="section-content">
          <!-- 状态提醒 -->
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

          <!-- 下载进度 -->
          <div v-if="downloading" class="download-area">
            <n-progress
              type="line"
              :percentage="downloadProgress"
              :processing="downloadProgress < 100"
              :indicator-placement="'inside'"
              class="download-progress"
            >
              {{ downloadMessage }}
            </n-progress>
          </div>

          <!-- 操作按钮区 -->
          <div class="actions-grid">
            <n-button
              type="primary"
              @click="downloadTheKernel"
              :loading="loading"
              :disabled="downloading"
              size="large"
              class="primary-action"
              round
            >
              <template #icon>
                <n-icon><DownloadOutline /></n-icon>
              </template>
              {{
                hasNewVersion
                  ? t('setting.kernel.downloadNew')
                  : kernelStore.version.version
                    ? t('setting.kernel.redownload')
                    : t('setting.kernel.download')
              }}
            </n-button>

            <n-button
              @click="showManualDownloadModal"
              :disabled="downloading"
              size="large"
              class="secondary-action"
              round
            >
              {{ t('setting.manualDownload') }}
            </n-button>

            <n-button
              @click="checkManualInstall"
              :disabled="downloading"
              size="large"
              class="secondary-action"
              round
            >
              {{ t('setting.checkInstallation') }}
            </n-button>
          </div>

          <!-- 错误提示 -->
          <n-alert v-if="downloadError" type="error" :show-icon="true" class="status-alert">
            <template #header>{{ t('setting.kernel.downloadFailed') }}</template>
            <div style="white-space: pre-line">{{ downloadError }}</div>
          </n-alert>
        </div>
      </section>

      <!-- 启动设置区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon startup-icon">
            <n-icon size="24"><PowerOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.startup.title') }}</h2>
            <p class="section-description">{{ t('setting.startup.subtitle') }}</p>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-list">
            <div class="setting-row">
              <div class="setting-info">
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
                size="large"
                class="setting-control"
              >
                <template #checked>{{ t('common.on') }}</template>
                <template #unchecked>{{ t('common.off') }}</template>
              </n-switch>
            </div>

            <div class="setting-row">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.autoStart.kernel') }}</div>
                <div class="setting-desc">
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
                class="setting-control"
              >
                <template #checked>{{ t('common.on') }}</template>
                <template #unchecked>{{ t('common.off') }}</template>
              </n-switch>
            </div>
          </div>
        </div>
      </section>

      <!-- 常规设置区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon general-icon">
            <n-icon size="24"><GlobeOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.general.title') }}</h2>
            <p class="section-description">{{ t('setting.general.subtitle') }}</p>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-list">
            <div class="setting-row">
              <div class="setting-info">
                <div class="setting-title">{{ $t('setting.language.title') }}</div>
              </div>
              <n-select
                v-model:value="localeStore.locale"
                :options="languageOptions"
                size="large"
                class="setting-control"
                @update:value="handleChangeLanguage"
                style="width: 200px"
              />
            </div>

            <div class="setting-row">
              <div class="setting-info">
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
                size="large"
                class="setting-control"
              >
                <template #checked>{{ t('common.on') }}</template>
                <template #unchecked>{{ t('common.off') }}</template>
              </n-switch>
            </div>

            <div class="setting-row">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.network.ports') }}</div>
                <div class="setting-desc">{{ t('setting.network.portsDesc') }}</div>
              </div>
              <n-button
                size="large"
                @click="showPortSettings"
                class="setting-control config-button"
              >
                {{ t('setting.network.configure') }}
              </n-button>
            </div>
          </div>
        </div>
      </section>

      <!-- 开发者工具区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon developer-icon">
            <n-icon size="24"><CodeOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.developer.title') }}</h2>
            <p class="section-description">{{ t('setting.developer.subtitle') }}</p>
          </div>
        </div>

        <div class="section-content">
          <div class="settings-list">
            <div class="setting-row">
              <div class="setting-info">
                <div class="setting-title">{{ t('setting.developer.openDevtools') }}</div>
                <div class="setting-desc">{{ t('setting.developer.description') }}</div>
              </div>
              <n-button
                type="primary"
                size="large"
                @click="handleOpenDevtools"
                :loading="devtoolsLoading"
                class="setting-control"
                round
              >
                {{ t('setting.developer.openDevtools') }}
              </n-button>
            </div>
          </div>

          <n-alert type="warning" :show-icon="true" class="status-alert" style="margin-top: 16px">
            {{ t('setting.developer.warning') }}
          </n-alert>
        </div>
      </section>

      <!-- 关于信息区域 -->
      <section class="settings-section">
        <div class="section-header">
          <div class="section-icon about-icon">
            <n-icon size="24"><InformationCircleOutline /></n-icon>
          </div>
          <div class="section-info">
            <h2 class="section-title">{{ t('setting.about.title') }}</h2>
            <p class="section-description">{{ t('setting.about.subtitle') }}</p>
          </div>
        </div>

        <div class="section-content">
          <!-- 信息网格 -->
          <div class="info-grid">
            <div class="info-card">
              <div class="info-header">
                <span class="info-label">{{ t('setting.about.appVersion') }}</span>
                <n-button
                  ghost
                  @click="handleCheckUpdate"
                  :loading="checkingUpdate"
                  size="small"
                  class="check-update-btn"
                  round
                >
                  <template #icon>
                    <n-icon><RefreshOutline /></n-icon>
                  </template>
                  {{ t('setting.update.check') }}
                </n-button>
              </div>
              <div class="info-value">{{ updateStore.appVersion }}</div>
            </div>

            <div class="info-card">
              <div class="info-header">
                <span class="info-label">{{ t('setting.about.kernelVersion') }}</span>
              </div>
              <div class="info-value">
                {{ formatVersion(kernelStore.version.version) || t('setting.notInstalled') }}
              </div>
            </div>

            <div class="info-card">
              <div class="info-header">
                <span class="info-label">{{ t('setting.about.system') }}</span>
              </div>
              <div class="info-value">Windows</div>
            </div>

            <div class="info-card">
              <div class="info-header">
                <span class="info-label">{{ t('setting.about.license') }}</span>
              </div>
              <div class="info-value">MIT License</div>
            </div>
          </div>

          <!-- 链接区域 -->
          <div class="links-area">
            <n-button
              text
              tag="a"
              href="https://github.com/xinggaoya/sing-box-windows"
              target="_blank"
              class="link-button"
              size="large"
            >
              <template #icon>
                <n-icon><LogoGithub /></n-icon>
              </template>
              {{ t('setting.about.githubRepo') }}
            </n-button>
            <div class="divider-line"></div>
            <n-button
              text
              tag="a"
              href="https://github.com/xinggaoya/sing-box-windows"
              target="_blank"
              class="link-button"
              size="large"
            >
              <template #icon>
                <n-icon><GlobeOutline /></n-icon>
              </template>
              {{ t('setting.about.website') }}
            </n-button>
          </div>
        </div>
      </section>
    </div>

    <!-- 回到顶部 -->
    <n-back-top :right="32" :bottom="32" @click="scrollToTop">
      <div class="back-top-button">
        <n-icon size="20"><ChevronUpOutline /></n-icon>
      </div>
    </n-back-top>
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
.modern-settings {
  display: flex;
  flex-direction: column;
  gap: 32px;
  min-height: 100%;
  padding: 0;
}

/* 英雄式头部 */
.settings-hero {
  background: var(--n-card-color);
  border-radius: 24px;
  border: 1px solid var(--n-border-color);
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  overflow: hidden;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-hero:hover {
  box-shadow: 0 8px 40px rgba(0, 0, 0, 0.12);
  transform: translateY(-2px);
}

.hero-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 32px 40px;
}

.hero-info {
  display: flex;
  align-items: center;
  gap: 20px;
}

.hero-icon {
  width: 64px;
  height: 64px;
  border-radius: 20px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 8px 24px rgba(99, 102, 241, 0.3);
}

.hero-text {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.hero-title {
  font-size: 32px;
  font-weight: 800;
  margin: 0;
  color: var(--n-text-color);
  line-height: 1.2;
}

.hero-subtitle {
  font-size: 16px;
  color: var(--n-text-color-2);
  margin: 0;
}

.hero-status {
  flex-shrink: 0;
}

.status-card {
  display: flex;
  gap: 24px;
  padding: 16px 24px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 16px;
  border: 1px solid var(--n-border-color);
}

.status-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  text-align: center;
}

.status-label {
  font-size: 12px;
  color: var(--n-text-color-3);
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.status-value {
  font-size: 14px;
  font-weight: 700;
  color: var(--n-text-color);
}

/* 设置内容区 */
.settings-content {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

/* 设置区块 */
.settings-section {
  background: var(--n-card-color);
  border-radius: 20px;
  border: 1px solid var(--n-border-color);
  overflow: hidden;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.settings-section:hover {
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
}

/* 区块头部 */
.section-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 24px 32px;
  border-bottom: 1px solid var(--n-border-color);
  background: rgba(0, 0, 0, 0.01);
}

.section-icon {
  width: 48px;
  height: 48px;
  border-radius: 16px;
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
  font-size: 20px;
  font-weight: 700;
  margin: 0 0 4px 0;
  color: var(--n-text-color);
}

.section-description {
  font-size: 14px;
  color: var(--n-text-color-2);
  margin: 0;
}

.section-status {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.status-tag {
  font-weight: 600;
}

/* 区块内容 */
.section-content {
  padding: 24px 32px 32px;
}

/* 状态提醒 */
.status-alert {
  margin-bottom: 20px;
  border-radius: 12px;
}

/* 下载区域 */
.download-area {
  margin-bottom: 20px;
}

.download-progress {
  height: 40px;
  border-radius: 20px;
  overflow: hidden;
}

/* 操作按钮网格 */
.actions-grid {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: 16px;
}

.primary-action {
  height: 48px;
  font-weight: 600;
  grid-column: span 1;
}

.secondary-action {
  height: 48px;
  font-weight: 500;
}

/* 设置列表 */
.settings-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 0;
  border-bottom: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.setting-row:last-child {
  border-bottom: none;
}

.setting-row:hover {
  background: rgba(0, 0, 0, 0.02);
  margin: 0 -16px;
  padding: 20px 16px;
  border-radius: 12px;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--n-text-color);
  margin-bottom: 4px;
}

.setting-desc {
  font-size: 14px;
  color: var(--n-text-color-2);
  line-height: 1.4;
}

.setting-control {
  flex-shrink: 0;
  margin-left: 20px;
}

.config-button {
  border: 2px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.config-button:hover {
  border-color: var(--n-primary-color);
  transform: translateY(-2px);
}

/* 信息网格 */
.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.info-card {
  padding: 20px;
  background: rgba(0, 0, 0, 0.02);
  border-radius: 16px;
  border: 1px solid var(--n-border-color);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.info-card:hover {
  background: rgba(0, 0, 0, 0.04);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.08);
}

.info-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.info-label {
  font-size: 12px;
  color: var(--n-text-color-3);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.info-value {
  font-size: 16px;
  font-weight: 700;
  color: var(--n-text-color);
}

.check-update-btn {
  height: 28px;
  font-size: 12px;
}

/* 链接区域 */
.links-area {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  padding-top: 20px;
  border-top: 1px solid var(--n-border-color);
}

.link-button {
  font-weight: 600;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.link-button:hover {
  transform: translateY(-2px);
}

.divider-line {
  width: 1px;
  height: 20px;
  background: var(--n-border-color);
}

/* 回到顶部 */
.back-top-button {
  width: 48px;
  height: 48px;
  border-radius: 24px;
  background: linear-gradient(135deg, #6366f1, #8b5cf6);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 6px 20px rgba(99, 102, 241, 0.3);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.back-top-button:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(99, 102, 241, 0.4);
}

/* 模态框样式 */
.modern-modal {
  border-radius: 20px;
}

.port-settings-form {
  padding: 20px 0;
}

.port-input {
  border-radius: 12px;
}

/* 深色模式适配 */
[data-theme='dark'] .status-card {
  background: rgba(255, 255, 255, 0.04);
}

[data-theme='dark'] .setting-row:hover {
  background: rgba(255, 255, 255, 0.04);
}

[data-theme='dark'] .info-card {
  background: rgba(255, 255, 255, 0.04);
}

[data-theme='dark'] .info-card:hover {
  background: rgba(255, 255, 255, 0.08);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .hero-content {
    flex-direction: column;
    gap: 24px;
    padding: 24px;
    text-align: center;
  }

  .status-card {
    flex-direction: column;
    gap: 16px;
  }

  .section-header {
    flex-direction: column;
    gap: 12px;
    padding: 20px;
    text-align: center;
  }

  .section-content {
    padding: 20px;
  }

  .setting-row {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .setting-control {
    margin-left: 0;
    align-self: flex-end;
  }

  .actions-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .info-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .links-area {
    flex-direction: column;
    gap: 12px;
  }

  .divider-line {
    width: 40px;
    height: 1px;
  }
}

@media (max-width: 480px) {
  .modern-settings {
    gap: 20px;
  }

  .hero-icon {
    width: 56px;
    height: 56px;
  }

  .hero-title {
    font-size: 24px;
  }

  .section-icon {
    width: 40px;
    height: 40px;
  }

  .section-title {
    font-size: 18px;
  }
}
</style>
