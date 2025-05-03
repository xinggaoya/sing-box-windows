<template>
  <div class="setting-container">
    <n-spin :show="pageLoading" description="加载中...">
      <!-- 内容布局使用网格 -->
      <n-grid :cols="1" :x-gap="16" :y-gap="16">
        <!-- 上部分核心设置 -->
        <n-gi>
          <n-grid :cols="24" :x-gap="16" :y-gap="16">
            <!-- 内核管理 -->
            <n-gi :span="isMobile ? 24 : 14">
              <n-card class="setting-card setting-card-primary" :bordered="false">
                <template #header>
                  <div class="card-header">
                    <n-h3 class="card-title">
                      <n-icon size="22" class="card-icon">
                        <settings-outline />
                      </n-icon>
                      {{ t('setting.kernel.title') }}
                    </n-h3>
                    <n-space align="center" :size="8">
                      <n-tag v-if="kernelStore.version.version" :bordered="false" type="default" size="small" class="version-tag">
                        {{ formatVersion(kernelStore.version.version) }}
                      </n-tag>
                      <n-tag v-else :bordered="false" type="error" size="small" class="version-tag">
                        {{ t('setting.kernel.notInstalled') }}
                      </n-tag>
                      <n-tag v-if="hasNewVersion" :bordered="false" type="warning" size="small" class="version-tag">
                        {{ t('setting.kernel.newVersion') }}{{ formatVersion(kernelStore.newVersion) }}
                      </n-tag>
                    </n-space>
                  </div>
                </template>

                <div class="card-content">
                  <n-alert v-if="hasNewVersion" type="warning" :show-icon="true" :title="t('setting.kernel.newVersionFound')"
                    class="version-alert compact-alert">
                    {{ t('setting.kernel.updateTip') }}
                  </n-alert>

                  <n-alert v-if="!kernelStore.version.version" type="error" :show-icon="true"
                    :title="t('setting.kernel.notInstalled')" class="version-alert compact-alert">
                    {{ t('setting.kernel.installPrompt') }}
                  </n-alert>

                  <n-progress v-if="downloading" type="line" :percentage="downloadProgress" :processing="downloadProgress < 100"
                    :indicator-placement="'inside'" :rail-style="{ background: 'var(--n-color-disabled)' }"
                    class="download-progress">
                    {{ downloadMessage }}
                  </n-progress>

                  <div class="action-row">
                    <n-button type="primary" @click="downloadTheKernel" :loading="loading" :disabled="downloading" size="small"
                      class="download-button">
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
                      <n-button text size="small" @click="showManualDownloadModal" :disabled="downloading" class="action-button">
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

            <!-- 服务管理 -->
            <n-gi :span="isMobile ? 24 : 10">
              <n-card class="setting-card" :bordered="false">
                <template #header>
                  <div class="card-header">
                    <n-h3 class="card-title">
                      <n-icon size="20" class="card-icon">
                        <server-outline />
                      </n-icon>
                      {{ $t('service.install.title') }}
                    </n-h3>
                  </div>
                </template>

                <div class="card-content compact-card">
                  <div class="status-row">
                    <n-space align="center" :size="8" wrap-item="false">
                      <n-tag :type="serviceStore.isServiceInstalled ? 'success' : 'error'" :bordered="false" size="small">
                        {{ serviceStore.isServiceInstalled ? $t('service.install.installed') :
                          $t('service.install.notInstalled') }}
                      </n-tag>
                      <span class="service-status">{{ $t('service.install.serviceStatus') }}</span>
                      <n-button text size="small" @click="refreshServiceStatus" :loading="checkingService" class="mini-button">
                        <template #icon>
                          <n-icon><refresh-outline /></n-icon>
                        </template>
                      </n-button>
                    </n-space>

                    <div class="service-desc" v-if="serviceStore.isServiceInstalled">
                      <n-tag :type="serviceStore.isServiceRunning ? 'success' : 'warning'" :bordered="false" size="tiny">
                        {{ serviceStore.isServiceRunning ? $t('service.install.running') : $t('service.install.notRunning') }}
                      </n-tag>
                    </div>
                  </div>

                  <div class="admin-bar" v-if="!isAdmin">
                    <n-button size="small" type="primary" @click="restartAsAdmin" :loading="isRestarting">
                      {{ $t('service.install.restartAsAdmin') }}
                    </n-button>
                  </div>

                  <div class="button-row">
                    <n-space :size="8">
                      <n-button size="small" type="warning" @click="handleUpdateService" :loading="serviceStore.isUpdating"
                        :disabled="!serviceStore.isServiceInstalled || !isAdmin">
                        <div class="button-content">
                          {{ $t('service.update.buttonText') }}
                          <n-tag v-if="serviceStore.needsUpdate" size="tiny" round type="warning" class="badge">
                            {{ $t('service.update.available') }}
                          </n-tag>
                        </div>
                      </n-button>
                      <n-button size="small" type="error" @click="handleUninstallService" :loading="serviceStore.isUninstalling"
                        :disabled="!serviceStore.isServiceInstalled || !isAdmin">
                        {{ $t('service.install.uninstallButton') }}
                      </n-button>
                      <n-button size="small" type="primary" @click="navigateToServiceInstall" :disabled="serviceStore.isServiceInstalled">
                        {{ $t('service.install.installButton') }}
                      </n-button>
                    </n-space>
                  </div>
                </div>
              </n-card>
            </n-gi>
            
            <!-- 启动设置 -->
            <n-gi :span="isMobile ? 24 : 12">
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
                          {{ appStore.autoStartApp ? t('setting.startup.bootTip') : t('setting.startup.manualTip') }}
                        </div>
                      </div>
                      <n-switch v-model:value="appStore.autoStartApp" @update-value="onAutoStartChange" size="small">
                        <template #checked>{{ t('common.on') }}</template>
                        <template #unchecked>{{ t('common.off') }}</template>
                      </n-switch>
                    </div>
                    
                    <div class="setting-row">
                      <div class="setting-item">
                        <div class="setting-title">{{ t('setting.autoStart.kernel') }}</div>
                        <div class="setting-desc">
                          {{ appStore.autoStartKernel ? t('setting.startup.autoKernelTip') : t('setting.startup.manualKernelTip') }}
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
            
            <!-- 语言和网络设置 -->
            <n-gi :span="isMobile ? 24 : 12">
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
                      <n-select v-model:value="localeStore.locale" :options="languageOptions" size="small"
                        class="language-select" @update:value="handleChangeLanguage" />
                    </div>
                    
                    <div class="setting-row">
                      <div class="setting-item">
                        <div class="setting-title">{{ t('setting.network.ipv6') }}</div>
                        <div class="setting-desc">
                          {{ appStore.preferIpv6 ? t('setting.network.preferIpv6') : t('setting.network.onlyIpv4') }}
                        </div>
                      </div>
                      <n-switch v-model:value="appStore.preferIpv6" @update-value="onIpVersionChange" size="small">
                        <template #checked>{{ t('common.on') }}</template>
                        <template #unchecked>{{ t('common.off') }}</template>
                      </n-switch>
                    </div>
                  </div>
                </div>
              </n-card>
            </n-gi>
          </n-grid>
        </n-gi>
        
        <!-- 关于信息 -->
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
                    <n-button text size="tiny" @click="handleCheckUpdate" :loading="checkingUpdate" class="check-button">
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
                  <n-button text tag="a" size="small" href="https://github.com/xinggaoya/sing-box-windows" target="_blank">
                    <template #icon>
                      <n-icon><logo-github /></n-icon>
                    </template>
                    GitHub
                  </n-button>
                  <n-divider vertical />
                  <n-button text tag="a" size="small" href="https://github.com/xinggaoya/sing-box-windows" target="_blank">
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
    </n-spin>
    
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
  <update-modal v-model:show="showUpdateModal" :latest-version="latestVersion" :current-version="updateStore.appVersion"
    :download-url="downloadUrl" @update="handleUpdate" @cancel="skipUpdate" />
</template>

<script setup lang="ts">
import { ref, computed, onMounted, nextTick } from 'vue'
import { useMessage, useDialog, useNotification } from 'naive-ui'
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
import { getVersion } from '@tauri-apps/api/app'
import i18n from '@/locales'

const message = useMessage()
const dialog = useDialog()
const notification = useNotification()
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

// 页面加载状态
const pageLoading = ref(true)

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

// 滚动到顶部
const scrollToTop = () => {
  window.scrollTo({
    top: 0,
    behavior: 'smooth'
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
      duration: 3000
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

    notification.info({
      title: t('setting.kernel.downloading'),
      content: t('setting.kernel.downloadingDescription'),
      duration: 3000
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
    // 检查管理员权限
    const isAdmin = await tauriApi.system.checkAdmin()
    if (!isAdmin) {
      // 如果没有管理员权限，请求以管理员权限重启
      await tauriApi.system.restartAsAdmin()
      return
    }

    if (value) {
      await enable()
      notification.success({
        title: t('setting.startup.enabled'),
        content: t('setting.startup.enableSuccess'),
        duration: 3000
      })
    } else {
      await disable()
      notification.info({
        title: t('setting.startup.disabled'),
        content: t('setting.startup.disableSuccess'),
        duration: 3000
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
        duration: 3000
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
  
  notification.success({
    title: t('setting.language.changed'),
    content: t('setting.language.changeSuccess'),
    duration: 3000
  })
}

const isAdmin = ref(false)
const checkingService = ref(false)
const isRestarting = ref(false)

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
    message.success(t('service.install.refreshSuccess'))
  } catch (error) {
    message.error(t('service.install.refreshError', { error }))
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
      message.error(t('service.install.restartError', { error }))
      return
    }
  }

  try {
    dialog.warning({
      title: t('service.install.uninstallConfirmTitle'),
      content: t('service.install.uninstallConfirmContent'),
      positiveText: t('service.install.uninstallConfirmButton'),
      negativeText: t('service.install.cancelButton'),
      onPositiveClick: async () => {
        const success = await serviceStore.uninstallService()
        if (success) {
          message.success(t('service.install.uninstallSuccess'))
        }
      }
    })
  } catch (error) {
    message.error(t('service.install.uninstallError', { error }))
  }
}

// 更新服务
async function handleUpdateService() {
  if (!isAdmin.value) {
    try {
      await tauriApi.system.restartAsAdmin()
      return
    } catch (error) {
      message.error(t('service.install.restartError', { error }))
      return
    }
  }

  try {
    const updated = await serviceStore.updateService()
    if (updated) {
      message.success(t('service.update.successMessage'))
    }
  } catch (error) {
    message.error(t('service.update.errorMessage', { error }))
  }
}

// 跳转到服务安装页面
function navigateToServiceInstall() {
  router.push('/service-install')
}

// 重启为管理员
async function restartAsAdmin() {
  try {
    isRestarting.value = true
    await tauriApi.system.restartAsAdmin()
  } catch (error) {
    message.error(t('service.install.restartError', { error }))
  } finally {
    isRestarting.value = false
  }
}

// 初始化加载
const initializeData = async () => {
  try {
    pageLoading.value = true
    
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
    // 检查服务是否需要更新
    if (serviceStore.isServiceInstalled) {
      await serviceStore.checkServiceUpdateNeeded()
    }
    
  } catch (error) {
    console.error('初始化数据失败:', error)
  } finally {
    // 使用nextTick确保UI更新
    await nextTick()
    pageLoading.value = false
  }
}

onMounted(async () => {
  // 添加窗口大小改变监听器
  window.addEventListener('resize', updateMobileStatus)
  
  await initializeData()
})
</script>

<style scoped>
.setting-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px 12px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  animation: fade-in 0.3s ease;
}

@keyframes fade-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.setting-card {
  border-radius: 12px;
  transition: all 0.3s ease;
  box-shadow: var(--shadow-light);
  overflow: hidden;
  height: 100%;
}

.setting-card-primary {
  background: linear-gradient(to right bottom, var(--primary-color-fade-1, #f0f7ff), transparent);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 4px 0;
}

.card-title {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0;
  font-weight: 600;
  font-size: 16px;
  color: var(--n-text-color);
}

.card-icon {
  color: var(--primary-color);
}

.card-content {
  padding: 4px 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.compact-card {
  gap: 8px;
}

.compact-alert {
  padding: 8px;
  margin: 0;
  font-size: 13px;
}

.version-tag {
  font-weight: 500;
  padding: 0 8px;
  height: 22px;
  border-radius: 11px;
}

.action-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.download-progress {
  margin: 8px 0;
  height: 30px;
  font-weight: 500;
  border-radius: 15px;
  overflow: hidden;
}

.download-button {
  font-weight: 500;
  border-radius: 8px;
}

.action-button {
  font-weight: 500;
  color: var(--n-text-color);
  transition: all 0.25s ease;
}

.action-button:hover:not(:disabled) {
  color: var(--primary-color);
}

.setting-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.setting-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-radius: 8px;
  background-color: var(--card-color, rgba(0,0,0,0.01));
  transition: background-color 0.2s ease;
}

.setting-row .setting-item {
  flex: 1;
  padding-right: 16px;
}

.setting-row:hover {
  background-color: var(--hover-color, rgba(0,0,0,0.03));
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
}

.setting-desc {
  font-size: 12px;
  color: var(--text-color-3);
}

/* 分隔线 */
.setting-divider {
  height: 1px;
  background-color: var(--divider-color, rgba(0,0,0,0.06));
  margin: 8px 0;
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
}

.about-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-radius: 8px;
  background-color: var(--card-color, rgba(0,0,0,0.01));
  transition: background-color 0.2s ease;
}

.about-item:hover {
  background-color: var(--hover-color, rgba(0,0,0,0.03));
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
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--divider-color);
  text-align: center;
}

.status-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.admin-bar {
  display: flex;
  justify-content: flex-end;
  margin: 8px 0;
}

.button-row {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}

.service-status {
  font-size: 13px;
  font-weight: 500;
}

.service-desc {
  margin-top: 4px;
  display: flex;
  align-items: center;
}

.mini-button {
  padding: 2px;
  margin: 0;
}

.badge {
  margin-left: 4px;
  border-radius: 10px;
  font-size: 10px;
  padding: 0 6px;
  height: 16px;
  line-height: 16px;
}

.check-button {
  font-size: 12px;
}

.back-top-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 36px;
  width: 36px;
  border-radius: 18px;
  background-color: var(--primary-color);
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  cursor: pointer;
  transition: all 0.3s ease;
}

.back-top-btn:hover {
  transform: translateY(-3px);
  box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
}

/* 按钮效果 */
:deep(.n-button) {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
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
    padding: 8px;
    gap: 12px;
  }
  
  .card-content {
    padding: 4px;
    gap: 8px;
  }
  
  .action-row {
    flex-direction: column;
    align-items: stretch;
  }
  
  .button-row {
    justify-content: center;
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
