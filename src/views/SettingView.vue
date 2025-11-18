<template>
  <div class="page-shell settings-container" :style="pageThemeStyle">
    <section class="page-hero">
      <div class="hero-row">
        <div class="hero-left">
          <div class="hero-icon">
            <n-icon size="26">
              <SettingsOutline />
            </n-icon>
          </div>
          <div class="hero-meta">
            <p class="hero-subtitle">{{ t('setting.subtitle') }}</p>
            <h2 class="hero-title">{{ t('setting.title') }}</h2>
          </div>
        </div>
      </div>
      <div class="hero-stats">
        <div
          v-for="stat in settingStats"
          :key="stat.label"
          class="stat-card"
          :data-accent="stat.accent"
        >
          <div class="stat-icon">
            <n-icon :size="20">
              <component :is="stat.icon" />
            </n-icon>
          </div>
          <div class="stat-info">
            <div class="stat-value">{{ stat.value }}</div>
            <div class="stat-label">{{ stat.label }}</div>
          </div>
        </div>
      </div>
    </section>

    <!-- 设置内容区域 -->
    <section class="page-section settings-content">
      <n-grid
        :cols="24"
        :x-gap="12"
        :y-gap="12"
        responsive="screen"
      >
        <!-- 内核管理卡片 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card kernel-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon kernel-icon">
                <n-icon size="20">
                  <SettingsOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.kernel.title') }}</h3>
                <p class="card-description">{{ t('setting.kernel.description') }}</p>
              </div>
              <div class="card-status">
                <n-tag
                  v-if="kernelStore.hasVersionInfo()"
                  type="success"
                  size="small"
                  round
                >
                  {{ formatVersion(kernelStore.getVersionString()) }}
                </n-tag>
                <n-tag v-else type="error" size="small" round>
                  {{ t('setting.notInstalled') }}
                </n-tag>
              </div>
            </div>

            <div class="card-content">
              <!-- 状态提醒 -->
              <div v-if="hasNewVersion || !kernelStore.hasVersionInfo()" class="alert-section">
                <n-alert
                  v-if="hasNewVersion"
                  type="warning"
                  size="small"
                  :show-icon="true"
                  class="kernel-alert"
                >
                  {{ t('setting.newVersionFound') }}
                </n-alert>
                <n-alert
                  v-if="!kernelStore.hasVersionInfo()"
                  type="error"
                  size="small"
                  :show-icon="true"
                  class="kernel-alert"
                >
                  {{ t('setting.kernel.installPrompt') }}
                </n-alert>
              </div>

              <!-- 下载进度 -->
              <div v-if="downloading" class="download-section">
                <n-progress
                  type="line"
                  :percentage="downloadProgress"
                  :processing="downloadProgress < 100"
                  :indicator-placement="'inside'"
                  size="small"
                  class="download-progress"
                >
                  {{ downloadProgress }}%
                </n-progress>
                <div class="download-message">
                  {{ downloadMessage }}
                </div>
              </div>

              <!-- 操作按钮 -->
              <div class="action-section">
                <n-space vertical>
                  <n-button
                    type="primary"
                    @click="downloadTheKernel"
                    :loading="loading"
                    :disabled="downloading"
                    block
                    size="medium"
                  >
                    <template #icon>
                      <n-icon size="16">
                        <DownloadOutline />
                      </n-icon>
                    </template>
                    {{
                      hasNewVersion ? t('setting.kernel.update') : kernelStore.hasVersionInfo() ? t('setting.kernel.redownload') : t('setting.kernel.download')
                    }}
                  </n-button>

                  <n-space>
                    <n-button
                      @click="showManualDownloadModal"
                      :disabled="downloading"
                      size="small"
                      ghost
                    >
                      {{ t('setting.kernel.manualDownload') }}
                    </n-button>
                    <n-button
                      @click="checkManualInstall"
                      :disabled="downloading"
                      size="small"
                      ghost
                    >
                      {{ t('setting.kernel.checkInstall') }}
                    </n-button>
                  </n-space>
                </n-space>
              </div>

              <!-- 错误提示 -->
              <n-alert v-if="downloadError" type="error" size="small" class="error-alert">
                {{ downloadError }}
              </n-alert>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 启动设置卡片 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card startup-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon startup-icon">
                <n-icon size="20">
                  <PowerOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.startup.title') }}</h3>
                <p class="card-description">{{ t('setting.startup.description') }}</p>
              </div>
            </div>

            <div class="card-content">
              <div class="setting-item-list">
                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <LogInOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ t('setting.autoStart.app') }}</div>
                      <div class="setting-desc">{{ t('setting.autoStart.appDesc') }}</div>
                    </div>
                  </div>
                  <n-switch
                    v-model:value="autoStart"
                    @update:value="onAutoStartChange"
                    size="medium"
                  />
                </div>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 常规设置卡片 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card general-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon general-icon">
                <n-icon size="20">
                  <GlobeOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.general.title') }}</h3>
                <p class="card-description">{{ t('setting.general.description') }}</p>
              </div>
            </div>

            <div class="card-content">
              <div class="setting-item-list">
                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <LanguageOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ $t('setting.language.title') }}</div>
                      <div class="setting-desc">{{ $t('setting.language.description') }}</div>
                    </div>
                  </div>
                  <n-select
                    v-model:value="localeStore.locale"
                    :options="languageOptions"
                    size="small"
                    @update:value="handleChangeLanguage"
                    class="language-select"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <WifiOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ t('setting.network.ipv6') }}</div>
                      <div class="setting-desc">{{ t('setting.network.ipv6Desc') }}</div>
                    </div>
                  </div>
                  <n-switch
                    v-model:value="appStore.preferIpv6"
                    @update:value="onIpVersionChange"
                    size="medium"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <OptionsOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ t('setting.network.ports') }}</div>
                      <div class="setting-desc">{{ t('setting.network.portsDesc') }}</div>
                    </div>
                  </div>
                  <n-button
                    size="small"
                    @click="showPortSettings"
                    type="primary"
                    ghost
                  >
                    {{ t('setting.network.configure') }}
                  </n-button>
                </div>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 代理高级设置 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card proxy-advanced-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon advanced-icon">
                <n-icon size="20">
                  <OptionsOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.proxyAdvanced.title') }}</h3>
                <p class="card-description">{{ t('setting.proxyAdvanced.description') }}</p>
              </div>
            </div>

            <div class="card-content">
              <n-form label-placement="top">
                <n-form-item :label="t('setting.proxyAdvanced.systemBypass')">
                  <n-input
                    v-model:value="proxyAdvancedForm.systemProxyBypass"
                    type="textarea"
                    :rows="3"
                    :placeholder="t('setting.proxyAdvanced.systemBypassPlaceholder')"
                  />
                  <div class="helper-text">
                    {{ t('setting.proxyAdvanced.systemBypassDesc') }}
                  </div>
                </n-form-item>

                <div class="tun-section">
                  <div class="tun-section-title">{{ t('setting.proxyAdvanced.tunTitle') }}</div>
                  <p class="tun-section-desc">
                    {{ t('setting.proxyAdvanced.tunAddressInfo') }}
                  </p>
                  <n-grid :cols="24" :x-gap="12">
                    <n-grid-item :span="24" :s="24" :m="12">
                      <n-form-item :label="t('setting.proxyAdvanced.tunMtu')">
                        <n-input-number
                          v-model:value="proxyAdvancedForm.tunMtu"
                          :min="576"
                          :max="9000"
                          style="width: 100%"
                        />
                      </n-form-item>
                    </n-grid-item>
                    <n-grid-item :span="24" :s="24" :m="12">
                      <n-form-item :label="t('setting.proxyAdvanced.tunStack')">
                        <n-select
                          v-model:value="proxyAdvancedForm.tunStack"
                          :options="tunStackOptions"
                        />
                      </n-form-item>
                    </n-grid-item>
                    <n-grid-item :span="24" :s="24" :m="12">
                      <n-form-item :label="t('setting.proxyAdvanced.enableIpv6')">
                        <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
                      </n-form-item>
                      <div class="helper-text">
                        {{ t('setting.proxyAdvanced.enableIpv6Desc') }}
                      </div>
                    </n-grid-item>
                    <n-grid-item :span="24" :s="24" :m="12">
                      <div class="switch-group">
                        <div class="switch-item">
                          <div class="switch-label">{{ t('setting.proxyAdvanced.autoRoute') }}</div>
                          <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
                        </div>
                        <div class="switch-item">
                          <div class="switch-label">{{ t('setting.proxyAdvanced.strictRoute') }}</div>
                          <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
                        </div>
                      </div>
                    </n-grid-item>
                  </n-grid>
                </div>

                <n-button
                  type="primary"
                  block
                  :loading="savingAdvanced"
                  @click="saveProxyAdvancedSettings"
                >
                  {{ t('setting.proxyAdvanced.save') }}
                </n-button>
              </n-form>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 更新设置卡片 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card update-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon update-icon">
                <n-icon size="20">
                  <RefreshOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.update.title') }}</h3>
                <p class="card-description">{{ t('setting.update.description') }}</p>
              </div>
              <div class="card-status">
                <n-tag
                  v-if="updateStore.isChecking"
                  type="info"
                  size="small"
                  round
                >
                  {{ t('setting.update.checking') }}
                </n-tag>
                <n-tag
                  v-else-if="updateStore.hasUpdate"
                  type="warning"
                  size="small"
                  round
                >
                  {{ t('setting.update.hasUpdate') }}
                </n-tag>
                <n-tag
                  v-else
                  type="success"
                  size="small"
                  round
                >
                  {{ t('setting.update.latest') }}
                </n-tag>
              </div>
            </div>

            <div class="card-content">
              <!-- 检查更新按钮 -->
              <div class="check-update-section">
                <n-button
                  type="primary"
                  @click="handleCheckUpdate"
                  :loading="checkingUpdate"
                  :disabled="updateStore.isChecking"
                  block
                  size="medium"
                >
                  <template #icon>
                    <n-icon size="16">
                      <RefreshOutline />
                    </n-icon>
                  </template>
                  {{
                    checkingUpdate
                      ? t('setting.update.checking')
                      : updateStore.hasUpdate
                      ? t('setting.update.checkAgain')
                      : t('setting.update.checkNow')
                  }}
                </n-button>

                <!-- 更新信息 -->
                <div v-if="updateStore.hasUpdate" class="update-info">
                  <n-alert type="info" size="small" :show-icon="true" class="update-alert">
                    <div class="update-info-content">
                      <div class="update-version">
                        {{ t('setting.update.currentVersion') }}: {{ updateStore.appVersion }}
                      </div>
                      <div class="update-version">
                        {{ t('setting.update.latestVersion') }}: {{ updateStore.latestVersion }}
                      </div>
                    </div>
                  </n-alert>

                  <!-- 更新按钮 -->
                  <div class="update-actions">
                    <n-button
                      type="primary"
                      size="small"
                      :loading="updateStore.updateState.downloading"
                      :disabled="updateStore.updateState.downloading"
                      @click="handleDownloadUpdate"
                      class="update-btn"
                    >
                      <template #icon>
                        <n-icon size="14">
                          <DownloadOutline />
                        </n-icon>
                      </template>
                      {{
                        updateStore.updateState.downloading
                          ? t('setting.update.downloading')
                          : t('setting.update.updateNow')
                      }}
                    </n-button>

                    <n-button
                      size="small"
                      @click="handleSkipVersion"
                      class="update-btn secondary"
                    >
                      {{ t('setting.update.skipVersion') }}
                    </n-button>
                  </div>

                  <!-- 下载进度 -->
                  <div v-if="updateStore.updateState.downloading" class="download-section">
                    <n-progress
                      type="line"
                      :percentage="updateStore.updateState.progress"
                      :indicator-placement="'inside'"
                      processing
                      size="small"
                      class="download-progress"
                    >
                      {{ updateStore.updateState.progress }}%
                    </n-progress>
                    <div class="download-message">
                      {{ updateStore.updateState.message }}
                    </div>
                  </div>

                  <!-- 错误提示 -->
                  <n-alert v-if="updateStore.updateState.error" type="error" size="small" class="error-alert">
                    {{ updateStore.updateState.error }}
                  </n-alert>
                </div>
              </div>

              <div class="setting-item-list">
                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <CheckmarkCircleOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ t('setting.update.autoCheck') }}</div>
                      <div class="setting-desc">{{ t('setting.update.autoCheckDesc') }}</div>
                    </div>
                  </div>
                  <n-switch
                    v-model:value="updateStore.autoCheckUpdate"
                    size="medium"
                  />
                </div>

                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <FlaskOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ t('setting.update.acceptPrerelease') }}</div>
                      <div class="setting-desc">{{ t('setting.update.acceptPrereleaseDesc') }}</div>
                    </div>
                  </div>
                  <n-switch
                    v-model:value="updateStore.acceptPrerelease"
                    size="medium"
                    @update:value="onPrereleaseSettingChange"
                  />
                </div>
              </div>

              <div v-if="updateStore.acceptPrerelease" class="alert-section">
                <n-alert type="warning" size="small" :show-icon="true" class="prerelease-alert">
                  {{ t('setting.update.prereleaseWarningDesc') }}
                </n-alert>
              </div>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 开发者工具卡片 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card developer-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon developer-icon">
                <n-icon size="20">
                  <CodeOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.developer.title') }}</h3>
                <p class="card-description">{{ t('setting.developer.description') }}</p>
              </div>
            </div>

            <div class="card-content">
              <div class="setting-item-list">
                <div class="setting-item">
                  <div class="setting-main">
                    <div class="setting-icon">
                      <n-icon size="18">
                        <ConstructOutline />
                      </n-icon>
                    </div>
                    <div class="setting-info">
                      <div class="setting-name">{{ t('setting.developer.openDevtools') }}</div>
                      <div class="setting-desc">{{ t('setting.developer.openDevtoolsDesc') }}</div>
                    </div>
                  </div>
                  <n-button
                    size="small"
                    @click="handleOpenDevtools"
                    :loading="devtoolsLoading"
                    type="primary"
                    ghost
                  >
                    {{ t('setting.developer.open') }}
                  </n-button>
                </div>
              </div>

              <n-alert type="info" size="small" :show-icon="true" class="dev-alert">
                {{ t('setting.developer.warning') }}
              </n-alert>
            </div>
          </n-card>
        </n-grid-item>

        <!-- 关于信息卡片 -->
        <n-grid-item :span="24" :s="24" :m="24" :l="12" :xl="12" :xxl="12">
          <n-card class="settings-card about-card" :bordered="false">
            <div class="card-header">
              <div class="card-icon about-icon">
                <n-icon size="20">
                  <InformationCircleOutline />
                </n-icon>
              </div>
              <div class="card-info">
                <h3 class="card-title">{{ t('setting.about.title') }}</h3>
                <p class="card-description">{{ t('setting.about.description') }}</p>
              </div>
            </div>

            <div class="card-content">
              <div class="info-grid">
                <div class="info-item">
                  <div class="info-label">{{ t('setting.appVersion') }}</div>
                  <div class="info-value">{{ updateStore.appVersion }}</div>
                </div>

                <div class="info-item">
                  <div class="info-label">{{ t('setting.kernelVersion') }}</div>
                  <div class="info-value">
                    {{ kernelStore.hasVersionInfo() ? formatVersion(kernelStore.getVersionString()) : t('setting.notInstalled') }}
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

              <div class="links-section">
                <n-button
                  text
                  tag="a"
                  href="https://github.com/xinggaoya/sing-box-windows"
                  target="_blank"
                  size="medium"
                  class="github-link"
                >
                  <template #icon>
                    <n-icon size="18">
                      <LogoGithub />
                    </n-icon>
                  </template>
                  GitHub
                </n-button>
              </div>
            </div>
          </n-card>
        </n-grid-item>
      </n-grid>
    </section>

    <!-- 端口设置对话框 -->
    <n-modal
      v-model:show="showPortModal"
      preset="dialog"
      :title="t('setting.network.portSettings')"
      class="modern-modal"
      :style="{ width: '480px' }"
    >
      <div class="port-settings-form">
        <n-form :model="{ proxyPort: tempProxyPort, apiPort: tempApiPort }" size="large">
          <n-form-item :label="t('setting.network.proxyPort')" :show-feedback="false">
            <n-input-number
              v-model:value="tempProxyPort"
              :min="1024"
              :max="65535"
              placeholder="7890"
              class="port-input"
            />
          </n-form-item>
          <n-form-item :label="t('setting.network.apiPort')" :show-feedback="false">
            <n-input-number
              v-model:value="tempApiPort"
              :min="1024"
              :max="65535"
              placeholder="23333"
              class="port-input"
            />
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
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, reactive, watch } from 'vue'
import { useMessage, useDialog, useNotification } from 'naive-ui'
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useAppStore } from '@/stores/app/AppStore'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useLocaleStore } from '@/stores/app/LocaleStore'
import { useThemeStore } from '@/stores/app/ThemeStore'
import { usePageTheme } from '@/composables/usePageTheme'
import type { Locale } from '@/stores/app/LocaleStore'
import { useRouter } from 'vue-router'
import {
  DownloadOutline,
  SettingsOutline,
  PowerOutline,
  InformationCircleOutline,
  LogoGithub,
  GlobeOutline,
  RefreshOutline,
  CodeOutline,
  LogInOutline,
  LanguageOutline,
  WifiOutline,
  OptionsOutline,
  CheckmarkCircleOutline,
  FlaskOutline,
  ConstructOutline,
} from '@vicons/ionicons5'
import { listen } from '@tauri-apps/api/event'
import { tauriApi } from '@/services/tauri'
import { appDataDir } from '@tauri-apps/api/path'
import { supportedLocales } from '@/locales'
import { useI18n } from 'vue-i18n'
import mitt from '@/utils/mitt'
import i18n from '@/locales'

defineOptions({
  name: 'SettingsPage'
})

const message = useMessage()
const dialog = useDialog()
const notification = useNotification()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const themeStore = useThemeStore()
const pageThemeStyle = usePageTheme(themeStore)
const autoStart = ref(false)
const router = useRouter()
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')
const { t } = useI18n()
const savingAdvanced = ref(false)
const proxyAdvancedForm = reactive({
  systemProxyBypass: appStore.systemProxyBypass,
  tunMtu: appStore.tunMtu,
  tunAutoRoute: appStore.tunAutoRoute,
  tunStrictRoute: appStore.tunStrictRoute,
  tunStack: appStore.tunStack,
  tunEnableIpv6: appStore.tunEnableIpv6,
})

// 格式化版本号，只显示纯版本号部分
const formatVersion = (version: string) => {
  if (!version) return ''

  const versionRegex = /\d+\.\d+\.\d+(?:-[\w.]+)?/
  const match = version.match(versionRegex)

  if (match) {
    return match[0]
  }

  if (version.startsWith('sing-box version ')) {
    return version.split(' ')[2]
  }

  if (version.includes(' ')) {
    return version.split(' ')[0]
  }

  return version
}

const settingStats = computed(() => [
  {
    label: t('setting.appVersion'),
    value: updateStore.appVersion,
    icon: SettingsOutline,
    accent: 'purple',
  },
  {
    label: t('setting.kernelVersion'),
    value: kernelStore.hasVersionInfo()
      ? formatVersion(kernelStore.getVersionString())
      : t('setting.notInstalled'),
    icon: DownloadOutline,
    accent: 'blue',
  },
])

const tunStackOptions = computed(() => [
  { label: t('setting.proxyAdvanced.stackOptions.system'), value: 'system' },
  { label: t('setting.proxyAdvanced.stackOptions.gvisor'), value: 'gvisor' },
  { label: t('setting.proxyAdvanced.stackOptions.mixed'), value: 'mixed' },
])

// 判断是否为移动端视图
const isMobile = ref(window.innerWidth < 768)

// 监听窗口尺寸变化以更新移动端状态
const updateMobileStatus = () => {
  isMobile.value = window.innerWidth < 768
}

watch(
  () => appStore.systemProxyBypass,
  value => {
    proxyAdvancedForm.systemProxyBypass = value
  }
)
watch(
  () => appStore.tunMtu,
  value => {
    proxyAdvancedForm.tunMtu = value
  }
)
watch(
  () => appStore.tunAutoRoute,
  value => {
    proxyAdvancedForm.tunAutoRoute = value
  }
)
watch(
  () => appStore.tunStrictRoute,
  value => {
    proxyAdvancedForm.tunStrictRoute = value
  }
)
watch(
  () => appStore.tunStack,
  value => {
    proxyAdvancedForm.tunStack = value
  }
)
watch(
  () => appStore.tunEnableIpv6,
  value => {
    proxyAdvancedForm.tunEnableIpv6 = value
  }
)

onMounted(async () => {
  // 从 AppStore 获取系统开机自启动状态
  autoStart.value = appStore.autoStartApp
  // 同时检查系统实际状态，确保一致性
  try {
    const systemAutoStart = await isEnabled()
    if (autoStart.value !== systemAutoStart) {
      console.warn('系统自启动状态不一致，更新AppStore状态', {
        appStore: autoStart.value,
        system: systemAutoStart
      })
      appStore.autoStartApp = systemAutoStart
      autoStart.value = systemAutoStart
    }
  } catch (error) {
    console.warn('检查系统自启动状态失败:', error)
  }
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

// 下载并安装更新
const handleDownloadUpdate = async () => {
  try {
    await updateStore.downloadAndInstallUpdate()
    message.success(t('setting.update.downloadStarted'))
  } catch (error) {
    message.error(t('setting.update.downloadError', { error: String(error) }))
  }
}

// 跳过当前版本
const handleSkipVersion = async () => {
  try {
    await updateStore.skipCurrentVersion()
    message.success(t('setting.update.skipSuccess'))
  } catch (error) {
    message.error(t('setting.update.skipError', { error: String(error) }))
  }
}

const hasNewVersion = computed(() => {
  if (!kernelStore.newVersion || !kernelStore.hasVersionInfo()) return false
  return formatVersion(kernelStore.newVersion) != formatVersion(kernelStore.getVersionString())
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

    await tauriApi.system.downloadLatestKernel()
    // 注意：版本信息更新现在由事件监听器处理，以避免重复操作
  } catch (error) {
    downloadError.value = error as string
    message.error(t('setting.kernel.downloadFailedMessage', { error: String(error) }))
    downloading.value = false
    loading.value = false
  }
}

const normalizeBypassList = (value: string) => {
  return value
    .split(/[\n,;]+/)
    .map(item => item.trim())
    .filter(Boolean)
    .join(';')
}

const saveProxyAdvancedSettings = async () => {
  try {
    savingAdvanced.value = true
    const bypassInput = proxyAdvancedForm.systemProxyBypass.trim()
    if (!bypassInput) {
      message.error(t('setting.proxyAdvanced.errors.bypassRequired'))
      return
    }
    if (proxyAdvancedForm.tunMtu < 576 || proxyAdvancedForm.tunMtu > 9000) {
      message.error(t('setting.proxyAdvanced.errors.invalidMtu'))
      return
    }
    const stackValid = ['system', 'gvisor', 'mixed'].includes(proxyAdvancedForm.tunStack)
    if (!stackValid) {
      message.error(t('setting.proxyAdvanced.errors.invalidStack'))
      return
    }

    await appStore.updateProxyAdvancedSettings({
      systemProxyBypass: normalizeBypassList(bypassInput),
      tunMtu: proxyAdvancedForm.tunMtu,
      tunAutoRoute: proxyAdvancedForm.tunAutoRoute,
      tunStrictRoute: proxyAdvancedForm.tunStrictRoute,
      tunStack: proxyAdvancedForm.tunStack as 'system' | 'gvisor' | 'mixed',
      tunEnableIpv6: proxyAdvancedForm.tunEnableIpv6,
    })

    notification.success({
      title: t('setting.proxyAdvanced.savedTitle'),
      content: t('setting.proxyAdvanced.savedDesc'),
      duration: 3000,
    })
  } catch (error) {
    message.error(`${t('common.error')}: ${error}`)
  } finally {
    savingAdvanced.value = false
  }
}

// 开机自启动设置
const onAutoStartChange = async (value: boolean) => {
  try {
    // 直接使用 AppStore 的方法，它会处理系统自启动和状态保存
    await appStore.toggleAutoStart(value)

    // 显示成功消息
    if (value) {
      notification.success({
        title: t('setting.startup.enabled'),
        content: t('setting.startup.enableSuccess'),
        duration: 3000,
      })
    } else {
      notification.info({
        title: t('setting.startup.disabled'),
        content: t('setting.startup.disableSuccess'),
        duration: 3000,
      })
    }
  } catch (error) {
    // 检测 autostart 插件的已知无害错误
    const errorMessage = String(error)
    const isHarmlessError = errorMessage.includes('os error 2') ||
                           errorMessage.includes('system') ||
                           errorMessage.includes('No such file or directory')

    if (isHarmlessError) {
      // 这是 autostart 插件的已知问题，功能实际生效了，不显示错误给用户
      console.log('Autostart 插件已知的无害错误，功能已生效:', error)
      // 但仍然显示成功消息，因为功能确实生效了
      if (value) {
        notification.success({
          title: t('setting.startup.enabled'),
          content: t('setting.startup.enableSuccess'),
          duration: 3000,
        })
      } else {
        notification.info({
          title: t('setting.startup.disabled'),
          content: t('setting.startup.disableSuccess'),
          duration: 3000,
        })
      }
      return
    }

    // 对于其他真正的错误，显示给用户
    message.error(`${t('common.error')}: ${error}`)
  }
}

// 自动启动内核设置
const onIpVersionChange = async (value: boolean) => {
  try {
    // 先更新 AppStore 状态并保存到数据库
    await appStore.setPreferIpv6(value)

    // 使用 KernelStore 来处理IP版本切换
    const { useKernelStore } = await import('@/stores/kernel/KernelStore')
    const kernelStore = useKernelStore()

    const result = await kernelStore.toggleIpVersion(value)

    if (result) {
      notification.success({
        title: t('setting.network.ipVersionChanged'),
        content: value ? t('setting.network.ipv6Enabled') : t('setting.network.ipv4Only'),
        duration: 3000,
      })
    } else {
      throw new Error(kernelStore.lastError || 'IP版本切换失败')
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

// 存储事件监听器的清理函数
let kernelDownloadUnlisten: (() => void) | null = null
let updateProgressUnlisten: (() => void) | null = null

// 监听内核下载进度事件
listen(
  'kernel-download-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    downloadProgress.value = progress
    downloadMessage.value = msg

    if (status === 'completed') {
      downloading.value = false
      loading.value = false
      downloadError.value = null
      message.success(t('setting.kernel.downloadComplete'))
      // 更新版本信息
      kernelStore.updateVersion()
    } else if (status === 'error') {
      downloading.value = false
      loading.value = false
      downloadError.value = msg
      message.error(t('setting.kernel.downloadFailedMessage', { error: msg }))
    }
  },
).then((unlisten) => {
  kernelDownloadUnlisten = unlisten
})

// 监听应用更新进度事件
listen(
  'update-progress',
  (event: { payload: { status: string; progress: number; message: string } }) => {
    const { status, progress, message: msg } = event.payload
    updateStore.updateProgress(status, progress, msg)
  },
).then((unlisten) => {
  updateProgressUnlisten = unlisten
})

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

  // 初始化数据（非阻塞）
  // handleCheckUpdate() // 自动检查更新逻辑已移至 App.vue
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

    // 更新端口设置到配置文件
    await tauriApi.config.updateSingboxPorts(tempProxyPort.value, tempApiPort.value)

    // 直接更新 AppStore 状态并保存到数据库
    appStore.proxyPort = tempProxyPort.value
    appStore.apiPort = tempApiPort.value
    await appStore.saveToBackend()

    if (appStore.isRunning) {
      notification.info({
        title: t('setting.network.restartRequired'),
        content: t('setting.network.restartDesc'),
        duration: 2500,
      })

      const restartResult = await kernelStore.restartKernel({
        keepAlive: appStore.autoStartKernel,
      })

      if (restartResult) {
        notification.success({
          title: t('setting.network.portChanged'),
          content: t('setting.network.portChangeSuccess'),
          duration: 3000,
        })
      } else {
        message.error(kernelStore.lastError || t('notification.restartFailed'))
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

  // 清理事件监听器
  if (kernelDownloadUnlisten) {
    kernelDownloadUnlisten()
    kernelDownloadUnlisten = null
  }
  if (updateProgressUnlisten) {
    updateProgressUnlisten()
    updateProgressUnlisten = null
  }
})
</script>

<style scoped>
.settings-container {
  padding: 24px;
  background: transparent;
  animation: fadeInUp 0.4s ease-out;
}

/* 设置内容区域 */
.settings-content {
  min-height: calc(100vh - 200px);
}

/* 设置卡片 */
.settings-card {
  height: 100%;
  transition: all 0.3s ease;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  position: relative;
  overflow: hidden;
}

.settings-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 40px v-bind('themeStore.isDark ? "rgba(0, 0, 0, 0.15)" : "rgba(0, 0, 0, 0.1)"');
}

.settings-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3px;
  background: linear-gradient(90deg, var(--card-color-start), var(--card-color-end));
  opacity: 0;
  transition: opacity 0.3s ease;
}

.settings-card:hover::before {
  opacity: 1;
}

/* 卡片头部 */
.card-header {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 12px;
  padding-bottom: 16px;
  border-bottom: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.card-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
  transition: all 0.3s ease;
}

.card-icon:hover {
  transform: scale(1.05);
}

.kernel-icon {
  background: linear-gradient(135deg, #3b82f6, #2563eb);
  --card-color-start: #3b82f6;
  --card-color-end: #2563eb;
}

.startup-icon {
  background: linear-gradient(135deg, #10b981, #059669);
  --card-color-start: #10b981;
  --card-color-end: #059669;
}

.general-icon {
  background: linear-gradient(135deg, #8b5cf6, #7c3aed);
  --card-color-start: #8b5cf6;
  --card-color-end: #7c3aed;
}

.advanced-icon {
  background: linear-gradient(135deg, #14b8a6, #0d9488);
  --card-color-start: #14b8a6;
  --card-color-end: #0d9488;
}

.update-icon {
  background: linear-gradient(135deg, #06b6d4, #0891b2);
  --card-color-start: #06b6d4;
  --card-color-end: #0891b2;
}

.developer-icon {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  --card-color-start: #f59e0b;
  --card-color-end: #d97706;
}

.about-icon {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  --card-color-start: #ef4444;
  --card-color-end: #dc2626;
}

.card-info {
  flex: 1;
  min-width: 0;
}

.proxy-advanced-card .helper-text {
  margin-top: 4px;
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#9ca3af" : "#6b7280"');
}

.tun-section {
  margin-top: 8px;
  padding-top: 16px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255,255,255,0.06)" : "rgba(15,23,42,0.08)"');
}

.tun-section-title {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 12px;
  color: v-bind('themeStore.isDark ? "#f1f5f9" : "#0f172a"');
}

.tun-section-desc {
  font-size: 12px;
  color: v-bind('themeStore.isDark ? "#9ca3af" : "#475569"');
  margin: 0 0 16px;
  line-height: 1.5;
}

.switch-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-top: 8px;
}

@media (min-width: 768px) {
  .switch-group {
    flex-direction: row;
  }
}

.switch-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-radius: 12px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255,255,255,0.06)" : "rgba(15,23,42,0.06)"');
  background: v-bind('themeStore.isDark ? "rgba(39,39,42,0.5)" : "#f8fafc"');
}

.switch-label {
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#e4e4e7" : "#0f172a"');
}

.card-title {
  font-size: 1.1rem;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin: 0 0 4px 0;
  line-height: 1.3;
}

.card-description {
  font-size: 0.85rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  margin: 0;
  line-height: 1.4;
}

.card-status {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

/* 卡片内容 */
.card-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 设置项列表 */
.setting-item-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.03)" : "rgba(0, 0, 0, 0.02)"');
  border-radius: 10px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  transition: all 0.2s ease;
}

.setting-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  transform: translateX(2px);
}

.setting-main {
  display: flex;
  align-items: center;
  gap: 12px;
  flex: 1;
  min-width: 0;
}

.setting-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
  display: flex;
  align-items: center;
  justify-content: center;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  flex-shrink: 0;
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-name {
  font-size: 0.9rem;
  font-weight: 500;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
  margin-bottom: 2px;
}

.setting-desc {
  font-size: 0.75rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  line-height: 1.3;
}

.language-select {
  min-width: 120px;
  max-width: 200px;
}

/* 警报区域 */
.alert-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.kernel-alert,
.prerelease-alert,
.dev-alert,
.error-alert {
  border-radius: 8px;
  font-size: 0.8rem;
}

/* 下载区域 */
.download-section {
  margin-bottom: 16px;
}

.download-progress {
  height: 32px;
  border-radius: 8px;
}

.download-message {
  margin-top: 8px;
  font-size: 0.8rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  line-height: 1.4;
  text-align: center;
}

/* 操作区域 */
.action-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 信息网格 */
.info-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  margin-bottom: 16px;
}

.info-item {
  padding: 12px;
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.03)" : "rgba(0, 0, 0, 0.02)"');
  border-radius: 8px;
  border: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  transition: all 0.2s ease;
}

.info-item:hover {
  background: v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.06)" : "rgba(0, 0, 0, 0.04)"');
  transform: translateY(-1px);
}

.info-label {
  font-size: 0.7rem;
  color: v-bind('themeStore.isDark ? "#94a3b8" : "#64748b"');
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  margin-bottom: 4px;
}

.info-value {
  font-size: 0.85rem;
  font-weight: 600;
  color: v-bind('themeStore.isDark ? "#f8fafc" : "#1e293b"');
}

/* 检查更新区域 */
.check-update-section {
  margin-bottom: 16px;
}

.update-info {
  margin-top: 12px;
}

.update-info-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.update-version {
  font-size: 0.8rem;
  line-height: 1.4;
}

.update-alert {
  border-radius: 8px;
  font-size: 0.8rem;
}

.update-actions {
  margin-top: 12px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.update-btn {
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.25s ease;
}

.update-btn.primary {
  box-shadow: 0 2px 8px rgba(24, 160, 88, 0.2);
}

.update-btn.primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(24, 160, 88, 0.3);
}

.update-btn.secondary {
  opacity: 0.7;
}

.update-btn.secondary:hover:not(:disabled) {
  opacity: 1;
}

/* 链接区域 */
.links-section {
  display: flex;
  align-items: center;
  justify-content: center;
  padding-top: 16px;
  border-top: 1px solid v-bind('themeStore.isDark ? "rgba(255, 255, 255, 0.08)" : "rgba(0, 0, 0, 0.06)"');
}

.github-link {
  font-weight: 500;
  transition: all 0.2s ease;
  color: #5b4cfd;
}

.github-link:hover {
  transform: translateY(-1px);
  color: #7c3aed;
}

/* 移除 Naive UI 模态框样式覆盖 */

.port-settings-form {
  padding: 16px 0;
}

.port-input {
  width: 100%;
}

/* 动画效果 */
@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* 响应式设计 */
@media (max-width: 768px) {
  .settings-container {
    padding: 16px;
  }

  .page-header {
    padding: 16px 20px;
    margin-bottom: 12px;
  }

  .header-content {
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;
  }

  .header-left {
    gap: 12px;
  }

  .header-icon {
    width: 40px;
    height: 40px;
  }

  .page-title {
    font-size: 1.25rem;
  }

  .page-subtitle {
    font-size: 0.85rem;
  }

  .version-badge {
    padding: 6px 12px;
  }

  .version-label {
    font-size: 0.75rem;
  }

  .card-header {
    gap: 12px;
    margin-bottom: 16px;
    padding-bottom: 12px;
  }

  .card-icon {
    width: 40px;
    height: 40px;
  }

  .card-title {
    font-size: 1rem;
  }

  .card-description {
    font-size: 0.8rem;
  }

  
  .setting-main {
    gap: 12px;
  }

  .setting-icon {
    width: 36px;
    height: 36px;
  }

  .setting-name {
    font-size: 0.85rem;
  }

  .setting-desc {
    font-size: 0.7rem;
  }

  .language-select {
    min-width: 100px;
    max-width: 100%;
  }

  .language-select {
    max-width: 180px;
  }

  .info-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }

  .info-item {
    padding: 10px;
  }

  .info-label {
    font-size: 0.65rem;
  }

  .info-value {
    font-size: 0.8rem;
  }

  .update-info-content {
    gap: 6px;
  }

  .update-version {
    font-size: 0.75rem;
  }
}

@media (max-width: 480px) {
  .settings-container {
    padding: 12px;
  }

  .page-header {
    padding: 12px 16px;
    margin-bottom: 16px;
  }

  .header-icon {
    width: 36px;
    height: 36px;
  }

  .page-title {
    font-size: 1.1rem;
  }

  .page-subtitle {
    font-size: 0.8rem;
  }

  .version-badge {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    padding: 8px 12px;
  }

  .version-separator {
    display: none;
  }

  .card-header {
    gap: 10px;
    padding-bottom: 10px;
  }

  .card-icon {
    width: 36px;
    height: 36px;
  }

  .card-title {
    font-size: 0.95rem;
  }

  .card-description {
    font-size: 0.75rem;
  }

  .setting-item {
    padding: 12px;
  }

  .language-select {
    max-width: 160px;
  }

  .setting-icon {
    width: 32px;
    height: 32px;
  }

  .setting-name {
    font-size: 0.8rem;
  }

  .setting-desc {
    font-size: 0.65rem;
  }
}

/* 移除 Naive UI 组件内部样式覆盖，使用官方主题系统 */
</style>
