<template>
  <div class="page-container">
    <PageHeader :title="t('setting.title')" :subtitle="t('setting.subtitle')" />

    <!-- Settings Grid -->
    <div class="settings-grid">
      <!-- Kernel Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><SettingsOutline /></n-icon>
          <h3>{{ t('setting.kernel.title') }} · {{ t('setting.kernel.advancedTag') }}</h3>
        </div>
        <div class="section-card">
          <div class="alert-box info">
            <n-icon size="18"><InformationCircleOutline /></n-icon>
            <span>{{ t('setting.kernel.embeddedHint') }}</span>
          </div>

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
              type="default"
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
          <div v-if="usingOriginalConfig" class="alert-box info">
            <n-icon size="18"><InformationCircleOutline /></n-icon>
            <span>{{ t('setting.singboxProfile.originalConfigHint') }}</span>
          </div>
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
              <div class="setting-label">{{ t('setting.update.channel') }}</div>
            </div>
            <n-select
              v-model:value="updateStore.updateChannel"
              :options="updateChannelOptions"
              size="small"
              class="setting-input"
              @update:value="onUpdateChannelChange"
            />
          </div>
        </div>
      </div>

      <!-- Backup Settings -->
      <div class="settings-section">
        <div class="section-header">
          <n-icon size="20"><ArchiveOutline /></n-icon>
          <h3>{{ t('setting.backup.title') }}</h3>
        </div>
        <div class="section-card">
          <div class="setting-row align-start">
            <div class="setting-info">
              <div class="setting-label">{{ t('setting.backup.description') }}</div>
              <div class="setting-desc">{{ t('setting.backup.restoreHint') }}</div>
            </div>
            <div class="backup-actions">
              <n-button
                size="small"
                secondary
                :loading="backupExporting"
                :disabled="backupBusy"
                @click="handleExportBackup"
              >
                {{ t('setting.backup.exportAction') }}
              </n-button>
              <n-button
                size="small"
                secondary
                :loading="backupValidating"
                :disabled="backupBusy"
                @click="handleValidateBackup"
              >
                {{ t('setting.backup.validateAction') }}
              </n-button>
              <n-button
                size="small"
                type="warning"
                :loading="backupRestoring"
                :disabled="backupBusy"
                @click="handleRestoreBackup"
              >
                {{ t('setting.backup.restoreAction') }}
              </n-button>
            </div>
          </div>

          <div v-if="backupPreview" class="backup-preview">
            <div class="backup-preview-row">
              <span class="meta-label">{{ t('setting.backup.selectedFile') }}</span>
              <span class="backup-path">{{ backupPreview.file_path }}</span>
            </div>
            <div class="backup-preview-row">
              <span class="meta-label">{{ t('setting.backup.subscriptionCount') }}</span>
              <span class="meta-value">{{ backupPreview.subscriptions_count }}</span>
            </div>
            <div class="backup-preview-row" :class="{ warning: backupPreview.warnings.length > 0 }">
              <span class="meta-label">{{ t('setting.backup.warningCount') }}</span>
              <span class="meta-value">{{ backupPreview.warnings.length }}</span>
            </div>
            <div v-if="backupPreview.warnings.length > 0" class="backup-warning-list">
              <div v-for="(warning, idx) in backupPreview.warnings" :key="idx" class="backup-warning-item">
                {{ warning }}
              </div>
            </div>
          </div>
          <div v-else class="setting-desc">
            {{ t('setting.backup.noPreview') }}
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

    <!-- Manual Kernel Import Modal -->
    <n-modal
      v-model:show="showManualImportModal"
      preset="dialog"
      :title="t('setting.kernel.manualImportTitle')"
      class="modern-modal"
      :style="{ width: '520px' }"
    >
      <div class="manual-import-body">
        <div class="manual-import-desc">{{ t('setting.kernel.manualImportDesc') }}</div>
        <div class="manual-drop-zone" :class="{ active: manualDropActive }">
          <n-icon size="24"><DownloadOutline /></n-icon>
          <div>{{ t('setting.kernel.dropHint') }}</div>
          <div class="manual-drop-sub">{{ t('setting.kernel.dropSubHint') }}</div>
        </div>

        <div v-if="manualKernelPath" class="manual-selected">
          <div class="manual-selected-label">{{ t('setting.kernel.selectedFile') }}</div>
          <div class="manual-selected-path">{{ manualKernelPath }}</div>
        </div>
      </div>
      <template #action>
        <n-space justify="space-between" style="width: 100%">
          <n-button @click="pickManualKernelFile" :disabled="manualImporting">
            {{ t('setting.kernel.chooseFile') }}
          </n-button>
          <n-space>
            <n-button @click="showManualImportModal = false" :disabled="manualImporting">
              {{ t('common.cancel') }}
            </n-button>
            <n-button type="primary" @click="importManualKernel" :loading="manualImporting">
              {{ t('setting.kernel.importNow') }}
            </n-button>
          </n-space>
        </n-space>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, reactive } from 'vue'
import { Window } from '@tauri-apps/api/window'
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
  ArchiveOutline,
} from '@vicons/ionicons5'
import { useI18n } from 'vue-i18n'
import { useAppStore, useKernelStore, useUpdateStore, useLocaleStore, useThemeStore } from '@/stores'
import { useSubStore } from '@/stores/subscription/SubStore'
import type { Locale } from '@/stores/app/LocaleStore'
import type { ThemeMode } from '@/stores/app/ThemeStore'
import type { UpdateChannel } from '@/stores/app/UpdateStore'
import { systemService, type BackupImportResult } from '@/services/system-service'
import { supportedLocales } from '@/locales'
import PageHeader from '@/components/common/PageHeader.vue'
import { ACCENT_PRESETS, TUN_STACK_OPTIONS } from '@/views/setting/setting-options'
import { useKernelDownload } from '@/views/setting/useKernelDownload'
import { useUpdateProgressListener } from '@/views/setting/useUpdateProgressListener'
import { useAdvancedSettingsForm } from '@/views/setting/useAdvancedSettingsForm'

const message = useMessage()
const { t } = useI18n()
const appStore = useAppStore()
const kernelStore = useKernelStore()
const updateStore = useUpdateStore()
const localeStore = useLocaleStore()
const themeStore = useThemeStore()
const subStore = useSubStore()

// State
const selectedKernelVersion = ref<string | undefined>(undefined)
const platformInfo = ref<{ os: string; arch: string; display_name: string } | null>(null)

const autoStart = ref(false)
const checkingUpdate = ref(false)
const backupExporting = ref(false)
const backupValidating = ref(false)
const backupRestoring = ref(false)
const backupPreview = ref<BackupImportResult | null>(null)
const themeForm = reactive({
  mode: 'system' as ThemeMode,
  accentColor: '#6366f1',
  compactMode: false,
})
const accentPresets = ACCENT_PRESETS

const showPortModal = ref(false)
const tempProxyPort = ref(12080)
const tempApiPort = ref(12081)
const portSettingsLoading = ref(false)
const showManualImportModal = ref(false)
const manualImporting = ref(false)
const manualKernelPath = ref('')
const manualDropActive = ref(false)
let manualDropUnlisten: (() => void) | null = null

// Options
const languageOptions = computed<{ label: string; value: Locale }[]>(() => [
  { label: t('setting.language.auto'), value: 'auto' },
  ...supportedLocales.map((item) => ({
    label: item.name,
    value: item.code as Locale,
  })),
])
const updateChannelOptions = computed<{ label: string; value: UpdateChannel }[]>(() => [
  { label: t('setting.update.channelStable'), value: 'stable' },
  { label: t('setting.update.channelPrerelease'), value: 'prerelease' },
  { label: t('setting.update.channelAutobuild'), value: 'autobuild' },
])

const tunStackOptions = TUN_STACK_OPTIONS

// Computed
const kernelLatestVersion = computed(() => kernelStore.latestAvailableVersion || '')
const activeSubscription = computed(() => subStore.getActiveSubscription())
const usingOriginalConfig = computed(() => activeSubscription.value?.useOriginalConfig ?? false)
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
const backupBusy = computed(
  () => backupExporting.value || backupValidating.value || backupRestoring.value,
)

const {
  loading,
  downloading,
  downloadProgress,
  downloadMessage,
  downloadTheKernel,
  cleanupDownloadListener,
} = useKernelDownload({
  selectedVersion: selectedKernelVersion,
  message,
  t,
  checkKernelInstallation: () => kernelStore.checkKernelInstallation(),
})

const { setupUpdateProgressListener, cleanupUpdateProgressListener } = useUpdateProgressListener({
  message,
  updateStore,
  t,
})

const {
  savingAdvanced,
  proxyAdvancedForm,
  savingSingboxProfile,
  singboxProfileForm,
  defaultOutboundOptions,
  downloadDetourOptions,
  saveProxyAdvancedSettings,
  saveSingboxProfileSettings,
} = useAdvancedSettingsForm({
  appStore,
  message,
  t,
})

// Methods
const formatVersion = (v: string) => v.replace(/^v/, '')
const isSupportedLocale = (l: string) => languageOptions.value.some(opt => opt.value === l)

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

watch(showManualImportModal, (visible) => {
  if (!visible) {
    manualDropActive.value = false
  }
})

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
  await localeStore.setLocale(nextLocale as Locale)
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

const setupManualDropListener = async () => {
  if (manualDropUnlisten) return

  const appWindow = Window.getCurrent()
  manualDropUnlisten = await appWindow.onDragDropEvent((event) => {
    if (!showManualImportModal.value || manualImporting.value) return

    if (event.payload.type === 'enter' || event.payload.type === 'over') {
      manualDropActive.value = true
      return
    }

    if (event.payload.type === 'leave') {
      manualDropActive.value = false
      return
    }

    if (event.payload.type === 'drop') {
      manualDropActive.value = false
      if (event.payload.paths.length > 0) {
        manualKernelPath.value = event.payload.paths[0]
      }
    }
  })
}

const cleanupManualDropListener = () => {
  if (manualDropUnlisten) {
    manualDropUnlisten()
    manualDropUnlisten = null
  }
}

const showManualDownloadModal = () => {
  manualKernelPath.value = ''
  showManualImportModal.value = true
}

const pickManualKernelFile = async () => {
  try {
    const selected = await systemService.pickKernelImportFile()
    if (selected) {
      manualKernelPath.value = selected
    }
  } catch (error) {
    console.error('选择内核文件失败:', error)
    message.error(t('setting.kernel.pickFailed'))
  }
}

const importManualKernel = async () => {
  if (!manualKernelPath.value) {
    message.warning(t('setting.kernel.noFileSelected'))
    return
  }

  manualImporting.value = true
  try {
    const result = await systemService.importKernelExecutable(manualKernelPath.value)
    message.success(result.message || t('common.saveSuccess'))
    showManualImportModal.value = false
    manualKernelPath.value = ''
    await kernelStore.checkKernelInstallation()
  } catch (error) {
    console.error('导入内核失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.kernel.importFailed')
    message.error(errMsg)
  } finally {
    manualImporting.value = false
  }
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

const handleCheckUpdate = async () => {
  checkingUpdate.value = true
  try {
    await updateStore.checkUpdate()
  } finally {
    checkingUpdate.value = false
  }
}

const onUpdateChannelChange = async (value: UpdateChannel) => {
  const previous = updateStore.updateChannel
  try {
    await updateStore.setUpdateChannel(value)
  } catch (error) {
    console.error('保存更新通道设置失败:', error)
    message.error(t('common.saveFailed'))
    updateStore.updateChannel = previous
  }
}

const reloadStoresAfterBackupRestore = async () => {
  await Promise.all([
    appStore.loadFromBackend(),
    themeStore.loadFromBackend(),
    localeStore.loadFromBackend(),
    updateStore.loadFromBackend(),
    subStore.loadFromBackend(),
  ])
  autoStart.value = appStore.autoStartApp
}

const handleExportBackup = async () => {
  backupExporting.value = true
  try {
    const result = await systemService.backupExportSnapshot()
    message.success(t('setting.backup.exportSuccess', { path: result.file_path }))
  } catch (error) {
    console.error('导出备份失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.backup.operationFailed')
    message.error(errMsg)
  } finally {
    backupExporting.value = false
  }
}

const handleValidateBackup = async () => {
  backupValidating.value = true
  try {
    const result = await systemService.backupImportSnapshot({ dryRun: true })
    backupPreview.value = result
    if (result.warnings.length > 0) {
      message.warning(t('setting.backup.validateWithWarnings', { count: result.warnings.length }))
    } else {
      message.success(t('setting.backup.validateSuccess', { count: result.subscriptions_count }))
    }
  } catch (error) {
    console.error('预检备份失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.backup.operationFailed')
    message.error(errMsg)
  } finally {
    backupValidating.value = false
  }
}

const handleRestoreBackup = async () => {
  const confirmed = window.confirm(t('setting.backup.restoreConfirm'))
  if (!confirmed) {
    return
  }

  backupRestoring.value = true
  try {
    const result = await systemService.backupImportSnapshot({
      filePath: backupPreview.value?.file_path,
      dryRun: false,
    })
    backupPreview.value = result
    await reloadStoresAfterBackupRestore()
    if (result.warnings.length > 0) {
      message.warning(t('setting.backup.restoreWithWarnings', { count: result.warnings.length }))
    } else {
      message.success(t('setting.backup.restoreSuccess', { count: result.subscriptions_count }))
    }
  } catch (error) {
    console.error('恢复备份失败:', error)
    const errMsg = error instanceof Error ? error.message : t('setting.backup.operationFailed')
    message.error(errMsg)
  } finally {
    backupRestoring.value = false
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
    await appStore.saveToBackend({ applyRuntime: true })
    message.success(t('common.saveSuccess'))
    showPortModal.value = false
  } catch (e) {
    message.error(t('common.saveFailed'))
  } finally {
    portSettingsLoading.value = false
  }
}

onMounted(async () => {
  try {
    await setupManualDropListener()
  } catch (error) {
    console.error('初始化手动导入拖放监听失败:', error)
  }

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
  cleanupManualDropListener()
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

.alert-box.info {
  background: rgba(14, 165, 233, 0.12);
  color: #0ea5e9;
  margin-bottom: 12px;
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

.backup-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 160px;
}

.backup-preview {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border: 1px solid var(--panel-border);
  border-radius: 10px;
  background: var(--bg-tertiary);
}

.backup-preview-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.backup-preview-row.warning .meta-value {
  color: #f59e0b;
}

.backup-path {
  flex: 1;
  text-align: right;
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
}

.backup-warning-list {
  margin-top: 4px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.backup-warning-item {
  font-size: 12px;
  color: #f59e0b;
  line-height: 1.5;
}

.manual-import-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.manual-import-desc {
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.manual-drop-zone {
  border: 1px dashed var(--panel-border);
  border-radius: 10px;
  background: var(--bg-tertiary);
  padding: 16px;
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  transition: border-color 0.2s ease, background 0.2s ease;
}

.manual-drop-zone.active {
  border-color: var(--primary-color);
  background: rgba(59, 130, 246, 0.12);
}

.manual-drop-sub {
  font-size: 12px;
  color: var(--text-tertiary);
}

.manual-selected {
  border: 1px solid var(--panel-border);
  border-radius: 8px;
  padding: 10px 12px;
  background: var(--bg-tertiary);
}

.manual-selected-label {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-bottom: 4px;
}

.manual-selected-path {
  font-size: 12px;
  color: var(--text-primary);
  word-break: break-all;
  line-height: 1.45;
}

@media (max-width: 768px) {
  .settings-grid {
    grid-template-columns: 1fr;
  }
}
</style>
