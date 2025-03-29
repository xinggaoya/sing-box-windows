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
            当前版本：{{ infoStore.version.version }}
          </n-tag>
          <n-tag v-else :bordered="false" type="error" size="medium" class="version-tag">
            未安装内核
          </n-tag>
          <n-tag
            v-if="hasNewVersion"
            :bordered="false"
            type="warning"
            size="medium"
            class="version-tag"
          >
            新版本：{{ infoStore.newVersion }}
          </n-tag>
        </n-space>
      </template>
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="24" class="card-icon">
              <settings-outline />
            </n-icon>
            内核管理
          </n-h3>
        </div>
      </template>

      <n-space vertical :size="20">
        <n-alert
          v-if="hasNewVersion"
          type="warning"
          :show-icon="true"
          title="发现新版本"
          class="version-alert"
        >
          有新版本的内核可供下载，建议更新以获得更好的体验。
        </n-alert>

        <n-alert
          v-if="!infoStore.version.version"
          type="error"
          :show-icon="true"
          title="未安装内核"
          class="version-alert"
        >
          请下载并安装内核后使用。
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
                ? '下载新版本'
                : infoStore.version.version
                  ? '重新下载当前版本'
                  : '下载内核'
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
              手动下载
            </n-button>
            <n-button text size="medium" @click="checkManualInstall" :disabled="downloading">
              检查安装
            </n-button>
          </n-space>
        </n-space>

        <n-alert v-if="downloadError" type="error" :show-icon="true" style="margin-top: 16px">
          <template #header> 下载失败 </template>
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
            启动设置
          </n-h3>
        </div>
      </template>

      <n-list>
        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">开机自启动</div>
              <div class="setting-desc">
                {{ appStore.autoStartApp ? '应用将在系统启动时自动运行' : '应用需要手动启动' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartApp" @update-value="onAutoStartChange">
              <template #checked>开</template>
              <template #unchecked>关</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">自动启动内核</div>
              <div class="setting-desc">
                {{ appStore.autoStartKernel ? '应用启动时将自动启动内核' : '需要手动启动内核' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStartKernel">
              <template #checked>开启</template>
              <template #unchecked>关闭</template>
            </n-switch>
          </n-space>
        </n-list-item>

        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">IPv6优先</div>
              <div class="setting-desc">
                {{ appStore.preferIpv6 ? '优先使用IPv6连接' : '仅使用IPv4连接' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.preferIpv6" @update-value="onIpVersionChange">
              <template #checked>开启</template>
              <template #unchecked>关闭</template>
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
            关于
          </n-h3>
        </div>
      </template>

      <n-grid :cols="2" :x-gap="12" :y-gap="8">
        <n-gi>
          <div class="about-item">
            <span class="about-label">应用版本</span>
            <n-space align="center">
              <span class="about-value">{{ appStore.appVersion }}</span>
              <n-button text size="tiny" @click="handleCheckUpdate" :loading="checkingUpdate">
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                检查更新
              </n-button>
            </n-space>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">内核版本</span>
            <span class="about-value">{{ infoStore.version.version }}</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">系统</span>
            <span class="about-value">Windows</span>
          </div>
        </n-gi>
        <n-gi>
          <div class="about-item">
            <span class="about-label">开源协议</span>
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
            官网
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
    message.error('更新失败: ' + error)
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
      message.success(`发现新版本：${result.latest_version}`)
    } else {
      message.info('当前已是最新版本')
    }
  } catch (error) {
    message.error(`检查更新失败: ${error}`)
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
    downloadMessage.value = '准备下载...'
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
    if (value) {
      await enable()
      message.success('已设置开机启动')
    } else {
      await disable()
      message.success('已关闭开机启动')
    }
  } catch (error) {
    message.error(`设置失败: ${error}`)
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
    message.error(`设置失败: ${error instanceof Error ? error.message : String(error)}`)
    // 回滚状态
    appStore.preferIpv6 = !value
  }
}

// 显示手动下载指引
const showManualDownloadModal = () => {
  dialog.info({
    title: '手动下载说明',
    content: `请按照以下步骤操作：
1. 访问 https://github.com/SagerNet/sing-box/releases/latest
2. 下载对应系统版本的 sing-box
3. 将解压后的 sing-box.exe 放置在以下目录：
用户目录/AppData/Local/sing-box-windows/sing-box/

完成后点击"检查安装"按钮验证安装是否成功。`,
    positiveText: '我知道了',
  })
}

// 检查手动安装
const checkManualInstall = async () => {
  try {
    loading.value = true
    const success = await infoStore.checkKernelVersion()
    if (success) {
      message.success('内核安装验证成功！')
    } else {
      message.error('未检测到有效的内核文件')
    }
  } catch (error) {
    message.error(`检查失败: ${error}`)
  } finally {
    loading.value = false
  }
}

// 获取应用数据目录
const getAppDataPath = async () => {
  try {
    appDataPath.value = await appDataDir()
  } catch (error) {
    console.error('获取应用数据目录失败:', error)
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
      message.success('内核下载完成！')
      // 更新版本信息
      infoStore.updateVersion()
    }
  },
)

onMounted(async () => {
  // 获取当前版本号
  await appStore.fetchAppVersion()
  // 检查更新
  await checkUpdate()
  // 获取应用数据目录
  await getAppDataPath()
  // 获取内核版本信息
  await infoStore.updateVersion()
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
