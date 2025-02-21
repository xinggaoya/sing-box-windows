<template>
  <div class="setting-container">
    <!-- 内核管理卡片 -->
    <n-card class="setting-card" :bordered="false">
      <template #header-extra>
        <n-space align="center">
          <n-tag :bordered="false" type="default" size="small">
            当前版本：{{ infoStore.version.version }}
          </n-tag>
          <n-tag v-if="hasNewVersion" :bordered="false" type="warning" size="small">
            新版本：{{ infoStore.newVersion }}
          </n-tag>
        </n-space>
      </template>
      <template #header>
        <div class="card-header">
          <n-h3 class="card-title">
            <n-icon size="20" class="card-icon">
              <settings-outline />
            </n-icon>
            内核管理
          </n-h3>
        </div>
      </template>

      <n-space vertical>
        <n-alert
          v-if="hasNewVersion"
          type="warning"
          :show-icon="true"
          title="发现新版本"
          style="margin-bottom: 16px"
        >
          有新版本的内核可供下载，建议更新以获得更好的体验。
        </n-alert>

        <n-progress
          v-if="downloading"
          type="line"
          :percentage="downloadProgress"
          :processing="downloadProgress < 100"
          :indicator-placement="'inside'"
          :rail-style="{ background: 'var(--n-color-disabled)' }"
        >
          {{ downloadMessage }}
        </n-progress>

        <n-space align="center" justify="space-between">
          <n-button
            type="primary"
            @click="downloadTheKernel"
            :loading="loading"
            :disabled="downloading"
            size="small"
          >
            <template #icon>
              <n-icon>
                <download-outline />
              </n-icon>
            </template>
            {{ hasNewVersion ? '下载新版本' : '重新下载当前版本' }}
          </n-button>
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
            启动设置
          </n-h3>
        </div>
      </template>

      <n-list>
        <n-list-item>
          <n-space justify="space-between" align="center" style="width: 100%">
            <div class="setting-item">
              <div class="setting-title">开机自启</div>
              <div class="setting-desc">
                {{ appStore.autoStart ? '应用将在系统启动时自动运行' : '应用需要手动启动' }}
              </div>
            </div>
            <n-switch v-model:value="appStore.autoStart" @update-value="onAutoStartChange">
              <template #checked>开启</template>
              <template #unchecked>关闭</template>
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
            <span class="about-value">1.0.0</span>
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
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useMessage } from 'naive-ui'
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
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri-api'

const message = useMessage()
const appStore = useAppStore()
const infoStore = useInfoStore()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')

// 监听下载进度事件
listen(
  'download-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    downloadProgress.value = progress
    downloadMessage.value = msg

    if (status === 'completed') {
      downloading.value = false
      message.success('内核下载完成！')
    }
  },
)

const hasNewVersion = computed(() => {
  if (!infoStore.newVersion || !infoStore.version.version) return false
  return infoStore.newVersion.includes(infoStore.version.version)
})

const downloadTheKernel = async () => {
  try {
    loading.value = true
    downloading.value = true
    downloadProgress.value = 0
    downloadMessage.value = '准备下载...'

    await tauriApi.subscription.downloadLatestKernel()
  } catch (error) {
    message.error(error as string)
    downloading.value = false
  } finally {
    loading.value = false
  }
}

const onAutoStartChange = async (value: boolean) => {
  try {
    if (!value) {
      await disable()
      message.success('已关闭开机自启')
    } else {
      await enable()
      message.success('已开启开机自启')
    }
  } catch (error) {
    message.error('设置失败')
    // 回滚状态
    appStore.autoStart = !value
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
</script>

<style scoped>
.setting-container {
  max-width: 800px;
  margin: 0 auto;
  padding: 16px;
}

.setting-card {
  margin-bottom: 16px;
  border-radius: 8px;
  transition: all 0.3s ease;
}

.setting-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 2px 12px 0 rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  align-items: center;
}

.card-title {
  display: flex;
  align-items: center;
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}

.card-icon {
  margin-right: 8px;
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
</style>
