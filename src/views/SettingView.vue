<template>
  <div class="page-container">
    <PageHeader :title="t('setting.title')" :subtitle="t('setting.subtitle')" />

    <!-- Settings Grid -->
    <div class="settings-grid">
      <!-- Kernel Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><SettingsOutline /></n-icon>
          <h3>{{ t('setting.kernel.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.kernel.version') }}</div>
              <div class="setting-desc">{{ t('setting.kernel.description') }}</div>
            </div>
            <div class="setting-action">
              <n-tag v-if="kernelStore.hasVersionInfo()" type="success" round :bordered="false">
                {{ formatVersion(kernelStore.getVersionString()) }}
              </n-tag>
              <n-tag v-else type="error" round :bordered="false">
                {{ t('setting.notInstalled') }}
              </n-tag>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.kernel.selectVersion') }}</div>
              <div class="setting-desc">{{ t('setting.kernel.selectVersionDesc') }}</div>
            </div>
            <n-select
              v-model:value="selectedKernelVersion"
              :options="kernelVersionOptions"
              :loading="kernelStore.isLoading"
              :disabled="downloading"
              size="small"
              class="setting-input"
              placeholder="Latest"
            />
          </div>

          <div v-if="hasNewVersion || !kernelStore.hasVersionInfo()" class="alert-box warning">
            <n-icon size="18"><WarningOutline /></n-icon>
            <span>
              {{
                hasNewVersion
                  ? t('setting.update.newVersionFound', { version: kernelLatestVersion || t('setting.newVersionFound') })
                  : t('setting.kernel.installPrompt')
              }}
            </span>
          </div>

          <div v-if="downloading" class="download-box">
            <n-progress
              type="line"
              :percentage="downloadProgress"
              :processing="downloadProgress < 100"
              indicator-placement="inside"
            />
            <div class="download-text">{{ downloadMessage }}</div>
          </div>

          <div class="actions-row">
            <n-button
              type="primary"
              @click="downloadTheKernel"
              :loading="loading"
              :disabled="downloading"
              block
              secondary
            >
              <template #icon><n-icon><DownloadOutline /></n-icon></template>
              {{ hasNewVersion ? t('setting.kernel.update') : kernelStore.hasVersionInfo() ? t('setting.kernel.redownload') : t('setting.kernel.download') }}
            </n-button>
            <div class="sub-actions">
              <n-button size="small" ghost @click="showManualDownloadModal" :disabled="downloading">
                {{ t('setting.kernel.manualDownload') }}
              </n-button>
              <n-button size="small" ghost @click="checkManualInstall" :disabled="downloading">
                {{ t('setting.kernel.checkInstall') }}
              </n-button>
            </div>
          </div>
        </div>
      </div>

      <!-- Startup Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><PowerOutline /></n-icon>
          <h3>{{ t('setting.startup.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.autoStart.app') }}</div>
              <div class="setting-desc">{{ t('setting.autoStart.appDesc') }}</div>
            </div>
            <n-switch v-model:value="autoStart" @update:value="onAutoStartChange" />
          </div>
        </div>
      </div>

      <!-- General Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><GlobeOutline /></n-icon>
          <h3>{{ t('setting.general.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.language.title') }}</div>
              <div class="setting-desc">{{ t('setting.language.description') }}</div>
            </div>
            <n-select
              v-model:value="localeStore.locale"
              :options="languageOptions"
              size="small"
              @update:value="handleChangeLanguage"
              class="setting-input"
            />
          </div>
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.network.ipv6') }}</div>
              <div class="setting-desc">{{ t('setting.network.ipv6Desc') }}</div>
            </div>
            <n-switch v-model:value="appStore.preferIpv6" @update:value="onIpVersionChange" />
          </div>
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.network.ports') }}</div>
              <div class="setting-desc">{{ t('setting.network.portsDesc') }}</div>
            </div>
            <n-button size="small" secondary @click="showPortSettings">
              {{ t('setting.network.configure') }}
            </n-button>
          </div>
        </div>
      </div>

      <!-- Theme Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><ColorPaletteOutline /></n-icon>
          <h3>{{ t('setting.theme.title') }}</h3>
        </div>
        <div class="section-card theme-card">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.theme.mode') }}</div>
              <div class="setting-desc">{{ t('setting.theme.modeDesc') }}</div>
            </div>
            <n-radio-group
              v-model:value="themeForm.mode"
              size="small"
              class="theme-mode-selector"
              @update:value="onThemeModeChange"
            >
              <n-radio-button value="system">{{ t('setting.theme.system') }}</n-radio-button>
              <n-radio-button value="light">{{ t('setting.theme.light') }}</n-radio-button>
              <n-radio-button value="dark">{{ t('setting.theme.dark') }}</n-radio-button>
            </n-radio-group>
          </div>

          <div class="setting-row align-start">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.theme.accent') }}</div>
              <div class="setting-desc">{{ t('setting.theme.accentDesc') }}</div>
            </div>
            <div class="theme-accent">
              <n-color-picker
                v-model:value="themeForm.accentColor"
                :modes="['hex']"
                size="small"
                :show-alpha="false"
                @update:value="onAccentChange"
              />
              <div class="preset-swatches">
                <button
                  v-for="color in accentPresets"
                  :key="color"
                  class="preset-swatch"
                  :style="{ background: color }"
                  @click="selectAccentPreset(color)"
                >
                  <span v-if="color === themeForm.accentColor" class="swatch-active"></span>
                </button>
              </div>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.theme.compactMode') }}</div>
              <div class="setting-desc">{{ t('setting.theme.compactDesc') }}</div>
            </div>
            <n-switch v-model:value="themeForm.compactMode" @update:value="onCompactModeChange" />
          </div>
        </div>
      </div>

      <!-- Advanced Proxy Settings -->
      <div class="settings-section full-width">
        <div class="section-header">
          <n-icon size="20"><OptionsOutline /></n-icon>
          <h3>{{ t('setting.proxyAdvanced.title') }}</h3>
        </div>
        <div class="section-card">
          <n-form label-placement="top" class="advanced-form">
            <n-grid :cols="24" :x-gap="24" :y-gap="16">
              <n-grid-item :span="24">
                <n-form-item :label="t('setting.proxyAdvanced.systemBypass')">
                  <n-input
                    v-model:value="proxyAdvancedForm.systemProxyBypass"
                    type="textarea"
                    :rows="3"
                    :placeholder="t('setting.proxyAdvanced.systemBypassPlaceholder')"
                  />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="24">
                <div class="subsection-title">{{ t('setting.proxyAdvanced.tunTitle') }}</div>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.proxyAdvanced.tunMtu')">
                  <n-input-number v-model:value="proxyAdvancedForm.tunMtu" :min="576" :max="9000" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.proxyAdvanced.tunStack')">
                  <n-select v-model:value="proxyAdvancedForm.tunStack" :options="tunStackOptions" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="24">
                <div class="toggles-row">
                  <div class="toggle-item">
                    <span>{{ t('setting.proxyAdvanced.enableIpv6') }}</span>
                    <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
                  </div>
                  <div class="toggle-item">
                    <span>{{ t('setting.proxyAdvanced.autoRoute') }}</span>
                    <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
                  </div>
                  <div class="toggle-item">
                    <span>{{ t('setting.proxyAdvanced.strictRoute') }}</span>
                    <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
                  </div>
                </div>
              </n-grid-item>

              <n-grid-item :span="24">
                <n-button
                  type="primary"
                  block
                  :loading="savingAdvanced"
                  @click="saveProxyAdvancedSettings"
                >
                  {{ t('setting.proxyAdvanced.save') }}
                </n-button>
              </n-grid-item>
            </n-grid>
          </n-form>
        </div>
      </div>

      <!-- sing-box 订阅配置生成（高级） -->
      <div class="settings-section full-width">
        <div class="section-header">
          <n-icon size="20"><OptionsOutline /></n-icon>
          <h3>{{ t('setting.singboxProfile.title') }}</h3>
        </div>
        <div class="section-card">
          <n-form label-placement="top" class="advanced-form">
            <n-grid :cols="24" :x-gap="24" :y-gap="16">
              <n-grid-item :span="24">
                <div class="subsection-title">{{ t('setting.singboxProfile.routingTitle') }}</div>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.singboxProfile.defaultOutbound')">
                  <n-select v-model:value="singboxProfileForm.defaultProxyOutbound" :options="defaultOutboundOptions" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.singboxProfile.downloadDetour')">
                  <n-select v-model:value="singboxProfileForm.downloadDetour" :options="downloadDetourOptions" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="24">
                <div class="toggles-row">
                  <div class="toggle-item">
                    <span>{{ t('setting.singboxProfile.blockAds') }}</span>
                    <n-switch v-model:value="singboxProfileForm.blockAds" />
                  </div>
                  <div class="toggle-item">
                    <span>{{ t('setting.singboxProfile.dnsHijack') }}</span>
                    <n-switch v-model:value="singboxProfileForm.dnsHijack" />
                  </div>
                  <div class="toggle-item">
                    <span>{{ t('setting.singboxProfile.enableAppGroups') }}</span>
                    <n-switch v-model:value="singboxProfileForm.enableAppGroups" />
                  </div>
                </div>
              </n-grid-item>

              <n-grid-item :span="24">
                <div class="subsection-title">{{ t('setting.singboxProfile.dnsTitle') }}</div>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.singboxProfile.dnsProxy')">
                  <n-input v-model:value="singboxProfileForm.dnsProxy" placeholder="https://1.1.1.1/dns-query" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.singboxProfile.dnsCn')">
                  <n-input v-model:value="singboxProfileForm.dnsCn" placeholder="h3://dns.alidns.com/dns-query" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.singboxProfile.dnsResolver')">
                  <n-input v-model:value="singboxProfileForm.dnsResolver" placeholder="114.114.114.114" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="12" :s="24" :m="12">
                <n-form-item :label="t('setting.singboxProfile.urltestUrl')">
                  <n-input v-model:value="singboxProfileForm.urltestUrl" placeholder="http://cp.cloudflare.com/generate_204" />
                </n-form-item>
              </n-grid-item>

              <n-grid-item :span="24">
                <n-button
                  type="primary"
                  block
                  :loading="savingSingboxProfile"
                  @click="saveSingboxProfileSettings"
                >
                  {{ t('setting.singboxProfile.save') }}
                </n-button>
              </n-grid-item>
            </n-grid>
          </n-form>
        </div>
      </div>

      <!-- Update Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><RefreshOutline /></n-icon>
          <h3>{{ t('setting.update.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="update-status">
            <div class="version-info">
              <span>{{ t('setting.update.currentVersion') }}: {{ updateStore.appVersion }}</span>
              <n-tag v-if="updateStore.hasUpdate" type="warning" size="small" round :bordered="false">
                {{ t('setting.update.hasUpdate') }}
              </n-tag>
              <n-tag v-else type="success" size="small" round :bordered="false">
                {{ t('setting.update.latest') }}
              </n-tag>
            </div>
            <n-button
              size="small"
              secondary
              @click="handleCheckUpdate"
              :loading="checkingUpdate"
              :disabled="updateStore.isChecking"
            >
              {{ checkingUpdate ? t('setting.update.checking') : t('setting.update.checkNow') }}
            </n-button>
          </div>

          <div v-if="updateStore.hasUpdate" class="update-alert-card">
            <div class="update-meta">
              <div class="meta-box">
                <div class="meta-label">{{ t('setting.update.newVersion') }}</div>
                <div class="meta-value">v{{ updateStore.latestVersion }}</div>
              </div>
              <div class="meta-box">
                <div class="meta-label">{{ t('setting.update.currentVersion') }}</div>
                <div class="meta-value">v{{ updateStore.appVersion }}</div>
              </div>
            </div>

            <div v-if="updateStore.releaseNotes" class="update-notes-preview">
              <span class="meta-label">{{ t('setting.update.releaseNotes') }}</span>
              <div class="notes custom-scrollbar">
                {{ updateStore.releaseNotes }}
              </div>
            </div>

            <div class="update-actions">
              <n-button
                type="primary"
                strong
                :loading="isUpdating"
                :disabled="isUpdating"
                @click="handleUpdateNow"
              >
                <template #icon><n-icon><DownloadOutline /></n-icon></template>
                {{
                  updateStatus === 'installing'
                    ? t('setting.update.installing')
                    : isUpdating
                      ? t('setting.update.downloading')
                      : t('setting.update.updateNow')
                }}
              </n-button>
              <n-button
                size="small"
                text
                @click="handleCheckUpdate"
                :disabled="checkingUpdate || isUpdating"
              >
                {{ t('setting.update.checkAgain') }}
              </n-button>
            </div>

            <div v-if="showUpdateProgress" class="update-progress">
              <div class="progress-header">
                <span class="progress-text">{{ updateMessage || t('setting.update.downloading') }}</span>
                <span class="progress-value">{{ updateProgress.toFixed(0) }}%</span>
              </div>
              <n-progress
                type="line"
                :percentage="updateProgress"
                :processing="updateStatus === 'downloading'"
                :status="updateStatus === 'error' ? 'error' : 'default'"
                :show-indicator="false"
              />
            </div>

            <div v-else-if="updateStatus === 'error'" class="update-error">
              {{ updateMessage || t('setting.update.updateFailed') }}
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.update.autoCheck') }}</div>
            </div>
            <n-switch v-model:value="updateStore.autoCheckUpdate" />
          </div>
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.update.acceptPrerelease') }}</div>
            </div>
            <n-switch v-model:value="updateStore.acceptPrerelease" @update:value="onPrereleaseSettingChange" />
          </div>
        </div>
      </div>

      <!-- About -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><InformationCircleOutline /></n-icon>
          <h3>{{ t('setting.about.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="about-list">
            <div class="about-item">
              <span class="label">{{ t('setting.appVersion') }}</span>
              <span class="value">{{ updateStore.appVersion }}</span>
            </div>
            <div class="about-item">
              <span class="label">{{ t('setting.kernel.version') }}</span>
              <span class="value">{{ kernelStore.hasVersionInfo() ? formatVersion(kernelStore.getVersionString()) : t('setting.notInstalled') }}</span>
            </div>
            <div class="about-item">
              <span class="label">平台</span>
              <span class="value">{{ platformInfo?.display_name || '加载中...' }}</span>
            </div>
            <div class="about-item">
              <span class="label">{{ t('setting.about.license') }}</span>
              <span class="value">MIT License</span>
            </div>
            <div class="about-actions">
              <n-button text tag="a" href="https://github.com/xinggaoya/sing-box-windows" target="_blank">
                <template #icon><n-icon><LogoGithub /></n-icon></template>
                GitHub
              </n-button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Port Modal -->
    <n-modal
      v-model:show="showPortModal"
      preset="dialog"
      :title="t('setting.network.portSettings')"
      class="modern-modal"
      :style="{ width: '400px' }"
    >
      <n-form label-placement="top">
        <n-form-item :label="t('setting.network.proxyPort')">
          <n-input-number v-model:value="tempProxyPort" :min="1024" :max="65535" />
        </n-form-item>
        <n-form-item :label="t('setting.network.apiPort')">
          <n-input-number v-model:value="tempApiPort" :min="1024" :max="65535" />
        </n-form-item>
      </n-form>
      <template #action>
        <n-space justify="end">
          <n-button @click="showPortModal = false">{{ t('common.cancel') }}</n-button>
          <n-button type="primary" @click="savePortSettings" :loading="portSettingsLoading">
            {{ t('common.save') }}
          </n-button>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, reactive } from 'vue'
import { useMessage } from 'naive-ui'
import {
  SettingsOutline,
  WarningOutline,
  DownloadOutline,
  PowerOutline,
  GlobeOutline,
  OptionsOutline,
  RefreshOutline,
  InformationCircleOutline,
  LogoGithub,
  ColorPaletteOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useAppStore, useKernelStore, useUpdateStore, useLocaleStore, useThemeStore } from '@/stores'
import type { Locale } from '@/stores/app/LocaleStore'
import type { ThemeMode } from '@/stores/app/ThemeStore'
import { APP_EVENTS } from '@/constants/events'
import { systemService } from '@/services/system-service'
import { eventService } from '@/services/event-service'
import { supportedLocales } from '@/locales'
import type { KernelDownloadPayload } from '@/services/kernel-service'
import PageHeader from '@/components/common/PageHeader.vue'

const message = useMessage()
const { t } = useI18n()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const themeStore = useThemeStore()

// State
const loading = ref(false)
const downloading = ref(false)
const downloadProgress = ref(0)
const downloadMessage = ref('')
const downloadError = ref('')
let downloadListener: (() => void) | null = null
let updateProgressListener: (() => void) | null = null
const selectedKernelVersion = ref<string | undefined>(undefined)
const platformInfo = ref<{ os: string; arch: string; display_name: string } | null>(null)

const autoStart = ref(false)
const locale = ref(localeStore.locale)
const checkingUpdate = ref(false)
const themeForm = reactive({
  mode: 'system' as ThemeMode,
  accentColor: '#6366f1',
  compactMode: false,
})
const accentPresets = ['#6366f1', '#0ea5e9', '#22c55e', '#f59e0b', '#e11d48', '#8b5cf6']

const showPortModal = ref(false)
const tempProxyPort = ref(12080)
const tempApiPort = ref(12081)
const portSettingsLoading = ref(false)

const savingAdvanced = ref(false)
const proxyAdvancedForm = reactive({
  systemProxyBypass: '',
  tunMtu: 9000,
  tunStack: 'mixed' as 'system' | 'gvisor' | 'mixed',
  tunEnableIpv6: false,
  tunAutoRoute: true,
  tunStrictRoute: true
})

const savingSingboxProfile = ref(false)
const singboxProfileForm = reactive({
  defaultProxyOutbound: 'manual' as 'manual' | 'auto',
  downloadDetour: 'manual' as 'manual' | 'direct',
  blockAds: true,
  dnsHijack: true,
  enableAppGroups: true,
  dnsProxy: '',
  dnsCn: '',
  dnsResolver: '',
  urltestUrl: '',
})

// Options
const languageOptions: { label: string; value: Locale }[] = [
  { label: t('setting.language.auto'), value: 'auto' },
  ...supportedLocales.map((item) => ({
    label: item.name,
    value: item.code as Locale,
  })),
]

const tunStackOptions = [
  { label: 'System', value: 'system' },
  { label: 'gVisor', value: 'gvisor' },
  { label: 'Mixed', value: 'mixed' }
]

const defaultOutboundOptions = [
  { label: t('setting.singboxProfile.outboundManual'), value: 'manual' },
  { label: t('setting.singboxProfile.outboundAuto'), value: 'auto' },
]

const downloadDetourOptions = [
  { label: t('setting.singboxProfile.detourManual'), value: 'manual' },
  { label: t('setting.singboxProfile.detourDirect'), value: 'direct' },
]

// Computed
const kernelLatestVersion = computed(() => kernelStore.latestAvailableVersion || '')
const hasNewVersion = computed(() => kernelStore.hasKernelUpdate)
const kernelVersionOptions = computed(() => {
  const versions = kernelStore.availableVersions || []
  return [
    { label: t('setting.kernel.latest'), value: undefined },
    ...versions.map(v => ({ label: v, value: v }))
  ]
})
const updateStatus = computed(() => updateStore.updateState.status)
const updateProgress = computed(() => updateStore.updateState.progress || 0)
const updateMessage = computed(() => updateStore.updateState.message)
const isUpdating = computed(() => ['downloading', 'installing'].includes(updateStatus.value))
const showUpdateProgress = computed(() =>
  ['downloading', 'installing', 'completed'].includes(updateStatus.value) || updateProgress.value > 0
)

// Methods
const formatVersion = (v: string) => v.replace(/^v/, '')
const isSupportedLocale = (l: string) => languageOptions.some(opt => opt.value === l)

// Initialize form data
watch(() => appStore.isDataRestored, (restored) => {
  if (restored) {
    proxyAdvancedForm.systemProxyBypass = appStore.systemProxyBypass
    proxyAdvancedForm.tunMtu = appStore.tunMtu
    proxyAdvancedForm.tunStack = appStore.tunStack as 'system' | 'gvisor' | 'mixed'
    proxyAdvancedForm.tunEnableIpv6 = appStore.tunEnableIpv6
    proxyAdvancedForm.tunAutoRoute = appStore.tunAutoRoute
    proxyAdvancedForm.tunStrictRoute = appStore.tunStrictRoute

    singboxProfileForm.defaultProxyOutbound = appStore.singboxDefaultProxyOutbound as 'manual' | 'auto'
    singboxProfileForm.downloadDetour = appStore.singboxDownloadDetour as 'manual' | 'direct'
    singboxProfileForm.blockAds = appStore.singboxBlockAds
    singboxProfileForm.dnsHijack = appStore.singboxDnsHijack
    singboxProfileForm.enableAppGroups = appStore.singboxEnableAppGroups
    singboxProfileForm.dnsProxy = appStore.singboxDnsProxy
    singboxProfileForm.dnsCn = appStore.singboxDnsCn
    singboxProfileForm.dnsResolver = appStore.singboxDnsResolver
    singboxProfileForm.urltestUrl = appStore.singboxUrltestUrl
  }
}, { immediate: true })

const saveSingboxProfileSettings = async () => {
  try {
    savingSingboxProfile.value = true

    appStore.singboxDefaultProxyOutbound = singboxProfileForm.defaultProxyOutbound
    appStore.singboxDownloadDetour = singboxProfileForm.downloadDetour
    appStore.singboxBlockAds = singboxProfileForm.blockAds
    appStore.singboxDnsHijack = singboxProfileForm.dnsHijack
    appStore.singboxEnableAppGroups = singboxProfileForm.enableAppGroups
    appStore.singboxDnsProxy = singboxProfileForm.dnsProxy.trim() || appStore.singboxDnsProxy
    appStore.singboxDnsCn = singboxProfileForm.dnsCn.trim() || appStore.singboxDnsCn
    appStore.singboxDnsResolver = singboxProfileForm.dnsResolver.trim() || appStore.singboxDnsResolver
    appStore.singboxUrltestUrl = singboxProfileForm.urltestUrl.trim() || appStore.singboxUrltestUrl

    await appStore.saveToBackend()
    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('保存 sing-box 配置生成高级选项失败:', error)
    message.error(t('common.saveFailed'))
  } finally {
    savingSingboxProfile.value = false
  }
}

const syncThemeForm = () => {
  themeForm.mode = themeStore.mode as ThemeMode
  themeForm.accentColor = themeStore.accentColor
  themeForm.compactMode = themeStore.compactMode
}

watch(
  () => [themeStore.mode, themeStore.accentColor, themeStore.compactMode],
  () => syncThemeForm(),
  { immediate: true },
)

const onAutoStartChange = async (value: boolean) => {
  try {
    await appStore.toggleAutoStart(value)
    message.success(t('common.saveSuccess'))
  } catch (error) {
    message.error(t('common.saveFailed'))
    autoStart.value = !value
  }
}
const handleChangeLanguage = async (value: string) => {
  if (!isSupportedLocale(value)) {
    console.warn('选择了不受支持的语言:', value)
    return
  }

  // 选择 auto 时明确切回自动模式，避免保留旧值
  const nextLocale = value === 'auto' ? 'auto' : value
  await localeStore.setLocale(nextLocale as any)
  locale.value = localeStore.currentLocale
}

const onIpVersionChange = async (value: boolean) => {
  try {
    // 先更新偏好并持久化，保证后端读取到最新配置
    await appStore.setPreferIpv6(value)
    await appStore.saveToBackend()

    const toggled = await kernelStore.toggleIpVersion(value)
    if (!toggled) {
      throw new Error(kernelStore.lastError || '切换IP版本失败')
    }

    message.success(t('common.saveSuccess'))
  } catch (error) {
    console.error('切换IPv6优先失败:', error)
    message.error(t('notification.proxyModeChangeFailed'))
  }
}

const onThemeModeChange = async (value: ThemeMode) => {
  await themeStore.setThemeMode(value)
}

const onAccentChange = async (value: string) => {
  await themeStore.setAccentColor(value)
}

const selectAccentPreset = async (color: string) => {
  themeForm.accentColor = color
  await themeStore.setAccentColor(color)
}

const onCompactModeChange = async (value: boolean) => {
  await themeStore.setCompactMode(value)
}

const cleanupDownloadListener = () => {
  if (downloadListener) {
    downloadListener()
    downloadListener = null
  }
}

const downloadTheKernel = async () => {
  if (downloading.value) return
  let downloadCompleted = false
  loading.value = true
  downloading.value = true
  downloadProgress.value = 0
  downloadMessage.value = t('setting.kernel.preparingDownload')
  downloadError.value = ''

  // 监听后端推送的下载进度，实时刷新 UI
  cleanupDownloadListener()
  downloadListener = await eventService.on(APP_EVENTS.kernelDownloadProgress, (payload) => {
    const data = payload as KernelDownloadPayload
    if (typeof data.progress === 'number') {
      downloadProgress.value = Math.min(100, Math.max(0, data.progress))
    }
    if (data.message) {
      downloadMessage.value = data.message
    }

    if (data.status === 'completed') {
      downloadCompleted = true
      downloading.value = false
      loading.value = false
      message.success(t('setting.kernel.downloadSuccess'))
      kernelStore.checkKernelInstallation()
      cleanupDownloadListener()
    } else if (data.status === 'error') {
      downloading.value = false
      loading.value = false
      downloadError.value = data.message || t('setting.kernel.downloadFailed')
      message.error(downloadError.value)
      cleanupDownloadListener()
    } else {
      downloadMessage.value ||= t('setting.kernel.downloadingDescription')
    }
  })

  try {
    await systemService.downloadKernel(selectedKernelVersion.value)
    // 如果后端未推送完成事件，也主动校验一次安装结果
    if (!downloadCompleted) {
      await kernelStore.checkKernelInstallation()
    }
  } catch (e) {
    console.error('下载内核失败:', e)
    downloadError.value = e instanceof Error ? e.message : t('setting.kernel.downloadFailed')
    message.error(downloadError.value)
    downloading.value = false
    loading.value = false
    cleanupDownloadListener()
  } finally {
    // 最终状态由事件驱动更新；若未收到事件则确保按钮可再次点击
    if (downloading.value) {
      loading.value = false
      downloading.value = false
    }
  }
}

const showManualDownloadModal = () => {
  message.info('Manual download not implemented yet')
}

const checkManualInstall = async () => {
  await kernelStore.checkKernelInstallation()
}

const handleUpdateNow = async () => {
  if (!updateStore.hasUpdate) {
    message.info(t('setting.update.alreadyLatest'))
    return
  }

  try {
    updateStore.updateProgress('downloading', 0, t('setting.update.preparingDownload'))
    await updateStore.downloadAndInstallUpdate()
  } catch (error) {
    console.error('启动更新失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.update.updateFailed')
    message.error(`${t('setting.update.updateFailed')}: ${errMsg}`)
  }
}

const saveProxyAdvancedSettings = async () => {
  savingAdvanced.value = true
  try {
    appStore.systemProxyBypass = proxyAdvancedForm.systemProxyBypass
    appStore.tunMtu = proxyAdvancedForm.tunMtu
    appStore.tunAutoRoute = proxyAdvancedForm.tunAutoRoute
    appStore.tunStrictRoute = proxyAdvancedForm.tunStrictRoute
    appStore.tunStack = proxyAdvancedForm.tunStack
    appStore.tunEnableIpv6 = proxyAdvancedForm.tunEnableIpv6
    await appStore.saveToBackend()
    message.success(t('common.saveSuccess'))
  } catch (e) {
    message.error(t('common.saveFailed'))
  } finally {
    savingAdvanced.value = false
  }
}

const handleCheckUpdate = async () => {
  checkingUpdate.value = true
  try {
    await updateStore.checkUpdate()
  } finally {
    checkingUpdate.value = false
  }
}

const onPrereleaseSettingChange = async (value: boolean) => {
  try {
    await updateStore.setAcceptPrerelease(value)
  } catch (error) {
    console.error('保存接收测试版本设置失败:', error)
    message.error(t('common.saveFailed'))
    // 回滚切换状态，避免 UI 与实际状态不一致
    updateStore.acceptPrerelease = !value
  }
}

const showPortSettings = () => {
  tempProxyPort.value = appStore.proxyPort
  tempApiPort.value = appStore.apiPort
  showPortModal.value = true
}

const savePortSettings = async () => {
  portSettingsLoading.value = true
  try {
    appStore.proxyPort = tempProxyPort.value
    appStore.apiPort = tempApiPort.value
    await appStore.saveToBackend()
    message.success(t('common.saveSuccess'))
    showPortModal.value = false
  } catch (e) {
    message.error(t('common.saveFailed'))
  } finally {
    portSettingsLoading.value = false
  }
}

// 监听更新下载进度，保持设置页状态与后端事件同步
const setupUpdateProgressListener = async () => {
  try {
    updateProgressListener = await eventService.on(APP_EVENTS.updateProgress, (payload) => {
      const data = payload as { status?: string; progress?: number; message?: string }
      const progress = typeof data.progress === 'number' ? data.progress : updateProgress.value
      const status = data.status || updateStatus.value
      const rawMessage = data.message || ''
      const localizedMessage = status === 'installing' ? t('setting.update.installStarted') : rawMessage

      updateStore.updateProgress(status, progress, localizedMessage)

      if (status === 'completed') {
        message.success(t('notification.updateDownloaded'))
      } else if (status === 'error') {
        message.error(localizedMessage || t('setting.update.updateFailed'))
      }
    })
  } catch (error) {
    console.error('监听更新进度失败:', error)
  }
}

const cleanupUpdateProgressListener = () => {
  if (updateProgressListener) {
    updateProgressListener()
    updateProgressListener = null
  }
}

onMounted(async () => {
  await appStore.waitForDataRestore()
  await appStore.syncAutoStartWithSystem()
  autoStart.value = appStore.autoStartApp
  watch(
    () => appStore.autoStartApp,
    (enabled) => {
      autoStart.value = enabled
    },
    { immediate: false },
  )

  // 获取平台信息
  try {
    platformInfo.value = await systemService.getDetailedPlatformInfo()
  } catch (error) {
    console.error('获取平台信息失败:', error)
  }

  await kernelStore.checkKernelInstallation()
  if (kernelStore.fetchLatestKernelVersion) {
    await kernelStore.fetchLatestKernelVersion()
  }
  if (kernelStore.fetchKernelReleases && (!kernelStore.availableVersions || kernelStore.availableVersions.length === 0)) {
    await kernelStore.fetchKernelReleases()
  }
  await updateStore.initializeStore?.()
  await setupUpdateProgressListener()
})

onUnmounted(() => {
  cleanupDownloadListener()
  cleanupUpdateProgressListener()
})
</script>

<style scoped>
.page-container {
  padding: var(--layout-page-padding-y, 24px) var(--layout-page-padding-x, 32px);
  max-width: var(--layout-page-max-width, 1200px);
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--layout-page-gap, 24px);
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: var(--layout-grid-gap, 24px);
}

.settings-section.full-width {
  grid-column: 1 / -1;
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text-secondary);
}

.section-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.section-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--layout-card-radius, 16px);
  padding: var(--layout-card-padding, 20px);
  display: flex;
  flex-direction: column;
  gap: var(--layout-row-gap, 16px);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--layout-row-gap, 16px);
}

.setting-info {
  flex: 1;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
}

.setting-input {
  width: 140px;
}

.setting-row.align-start {
  align-items: flex-start;
}

.theme-card {
  gap: 18px;
}

.theme-mode-selector {
  display: flex;
  gap: 8px;
}

.theme-accent {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.preset-swatches {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.preset-swatch {
  width: 34px;
  height: 24px;
  border-radius: 10px;
  border: 2px solid transparent;
  cursor: pointer;
  box-shadow: 0 4px 10px rgba(0, 0, 0, 0.08);
  transition: transform 0.15s ease, box-shadow 0.15s ease, border-color 0.2s ease;
  position: relative;
}

.preset-swatch:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 14px rgba(0, 0, 0, 0.12);
}

.swatch-active {
  position: absolute;
  inset: 4px;
  border-radius: 8px;
  border: 2px solid #fff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.2);
}

.alert-box {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px;
  border-radius: 8px;
  font-size: 13px;
}

.alert-box.warning {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
}

.download-box {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.download-text {
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}

.actions-row {
  display: flex;
  flex-direction: column;
  gap: var(--layout-inline-gap, 12px);
}

.sub-actions {
  display: flex;
  gap: var(--layout-inline-gap-tight, 8px);
  justify-content: center;
}

.advanced-form {
  width: 100%;
}

.subsection-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.toggles-row {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 8px;
}

.toggle-item {
  display: flex;
  align-items: center;
  gap: calc(var(--layout-inline-gap, 12px) - 2px);
  font-size: 13px;
  color: var(--text-secondary);
}

.update-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 8px;
}

.version-info {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
  color: var(--text-secondary);
}

.about-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.about-item {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}

.about-item .label {
  color: var(--text-tertiary);
}

.about-item .value {
  color: var(--text-primary);
  font-weight: 500;
}

.about-actions {
  margin-top: 8px;
  display: flex;
  justify-content: center;
}

.update-alert-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(16, 185, 129, 0.12), rgba(6, 182, 212, 0.1));
  border: 1px solid rgba(16, 185, 129, 0.2);
}

.update-meta {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: 12px;
}

.meta-box {
  padding: 10px 12px;
  border-radius: 10px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.meta-label {
  display: block;
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 6px;
}

.meta-value {
  font-weight: 700;
  color: var(--text-primary);
  font-size: 16px;
}

.update-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.update-notes-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.update-notes-preview .notes {
  max-height: 120px;
  overflow: auto;
  padding: 10px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.04);
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.update-progress {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 13px;
  color: var(--text-secondary);
}

.progress-value {
  font-weight: 600;
  color: var(--text-primary);
}

.update-error {
  font-size: 13px;
  color: #ef4444;
}

@media (max-width: 768px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }
}
</style>
