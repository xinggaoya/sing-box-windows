<template>
  <div class="settings-panel">
    <div class="settings-group">
      <div class="group-header" @click="toggleSection('network')">
        <div class="group-icon net-icon">
          <n-icon :size="18"><GlobeOutline /></n-icon>
        </div>
        <div class="group-title-area">
          <div class="group-title">{{ props.t('setting.network.title') }}</div>
        </div>
        <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.network }">
          <ChevronDownOutline />
        </n-icon>
      </div>

      <transition name="collapse">
        <div v-if="expandedSections.network" class="group-body">
          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ props.t('setting.network.ipv6') }}</div>
              <div class="setting-desc">{{ props.t('setting.network.ipv6Desc') }}</div>
            </div>
            <n-switch :value="props.appStore.preferIpv6" @update:value="props.onIpVersionChange" />
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ props.t('setting.network.ports') }}</div>
              <div class="setting-desc">{{ props.t('setting.network.portsDesc') }}</div>
            </div>
            <n-button size="small" secondary @click="props.showPortSettings">
              <template #icon><n-icon :size="14"><SettingsOutline /></n-icon></template>
              {{ props.t('setting.network.configure') }}
            </n-button>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <div class="setting-label">{{ props.t('setting.network.allowLanAccess') }}</div>
              <div class="setting-desc">{{ props.t('setting.network.allowLanAccessDesc') }}</div>
            </div>
            <n-switch
              :value="props.appStore.allowLanAccess"
              @update:value="props.onLanAccessChange"
            />
          </div>
        </div>
      </transition>
    </div>

    <div class="settings-group">
      <div class="group-header" @click="toggleSection('proxy')">
        <div class="group-icon proxy-icon">
          <n-icon :size="18"><OptionsOutline /></n-icon>
        </div>
        <div class="group-title-area">
          <div class="group-title">{{ props.t('setting.proxyAdvanced.title') }}</div>
        </div>
        <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.proxy }">
          <ChevronDownOutline />
        </n-icon>
      </div>

      <transition name="collapse">
        <div v-if="expandedSections.proxy" class="group-body">
          <n-form label-placement="top" class="advanced-form">
            <div class="form-section">
              <n-form-item :label="props.t('setting.proxyAdvanced.systemBypass')">
                <n-input
                  v-model:value="proxyAdvancedForm.systemProxyBypass"
                  type="textarea"
                  :rows="3"
                  :placeholder="props.t('setting.proxyAdvanced.systemBypassPlaceholder')"
                />
              </n-form-item>
            </div>

            <div class="form-section-title">{{ props.t('setting.proxyAdvanced.tunTitle') }}</div>

            <div class="form-grid">
              <n-form-item :label="props.t('setting.proxyAdvanced.tunMtu')">
                <n-input-number v-model:value="proxyAdvancedForm.tunMtu" :min="576" :max="9000" />
              </n-form-item>
              <n-form-item :label="props.t('setting.proxyAdvanced.tunStack')">
                <n-select v-model:value="proxyAdvancedForm.tunStack" :options="props.tunStackOptions" />
              </n-form-item>
            </div>

            <n-form-item :label="props.t('setting.proxyAdvanced.tunRouteExcludeAddress')">
              <n-input
                v-model:value="proxyAdvancedForm.tunRouteExcludeAddressText"
                type="textarea"
                :rows="3"
                :placeholder="props.t('setting.proxyAdvanced.tunRouteExcludeAddressPlaceholder')"
              />
            </n-form-item>

            <div class="toggles-grid">
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.proxyAdvanced.enableIpv6') }}</span>
                <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.proxyAdvanced.autoRoute') }}</span>
                <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.proxyAdvanced.strictRoute') }}</span>
                <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.proxyAdvanced.tunSelfHeal') }}</span>
                <n-switch v-model:value="proxyAdvancedForm.tunSelfHealEnabled" />
              </div>
            </div>

            <n-form-item
              v-if="proxyAdvancedForm.tunSelfHealEnabled"
              :label="props.t('setting.proxyAdvanced.tunSelfHealCooldown')"
            >
              <n-input-number
                v-model:value="proxyAdvancedForm.tunSelfHealCooldownSecs"
                :min="15"
                :max="600"
              />
            </n-form-item>

            <n-button
              type="primary"
              block
              :loading="savingAdvanced"
              @click="saveProxyAdvancedSettings"
            >
              {{ props.t('setting.proxyAdvanced.save') }}
            </n-button>
          </n-form>
        </div>
      </transition>
    </div>

    <div class="settings-group">
      <div class="group-header" @click="toggleSection('profile')">
        <div class="group-icon profile-icon">
          <n-icon :size="18"><LayersOutline /></n-icon>
        </div>
        <div class="group-title-area">
          <div class="group-title">{{ props.t('setting.singboxProfile.title') }}</div>
        </div>
        <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.profile }">
          <ChevronDownOutline />
        </n-icon>
      </div>

      <transition name="collapse">
        <div v-if="expandedSections.profile" class="group-body">
          <div v-if="props.usingOriginalConfig" class="info-banner">
            <n-icon :size="16"><InformationCircleOutline /></n-icon>
            <span>{{ props.t('setting.singboxProfile.originalConfigHint') }}</span>
          </div>

          <n-form label-placement="top" class="advanced-form">
            <div class="form-section-title">{{ props.t('setting.singboxProfile.routingTitle') }}</div>

            <div class="form-grid">
              <n-form-item :label="props.t('setting.singboxProfile.defaultOutbound')">
                <n-select
                  v-model:value="singboxProfileForm.defaultProxyOutbound"
                  :options="defaultOutboundOptions"
                />
              </n-form-item>
              <n-form-item :label="props.t('setting.singboxProfile.downloadDetour')">
                <n-select
                  v-model:value="singboxProfileForm.downloadDetour"
                  :options="downloadDetourOptions"
                />
              </n-form-item>
            </div>

            <div class="toggles-grid">
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.singboxProfile.blockAds') }}</span>
                <n-switch v-model:value="singboxProfileForm.blockAds" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.singboxProfile.dnsHijack') }}</span>
                <n-switch v-model:value="singboxProfileForm.dnsHijack" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.singboxProfile.enableAppGroups') }}</span>
                <n-switch v-model:value="singboxProfileForm.enableAppGroups" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ props.t('setting.singboxProfile.fakeDnsEnabled') }}</span>
                <n-switch v-model:value="singboxProfileForm.fakeDnsEnabled" />
              </div>
            </div>

            <div class="form-section-title">{{ props.t('setting.singboxProfile.fakeDnsTitle') }}</div>

            <div class="form-grid">
              <n-form-item :label="props.t('setting.singboxProfile.fakeDnsFilterMode')">
                <n-select
                  v-model:value="singboxProfileForm.fakeDnsFilterMode"
                  :options="fakeDnsFilterOptions"
                  :disabled="!singboxProfileForm.fakeDnsEnabled"
                />
              </n-form-item>
              <n-form-item :label="props.t('setting.singboxProfile.fakeDnsIpv4Range')">
                <n-input
                  v-model:value="singboxProfileForm.fakeDnsIpv4Range"
                  placeholder="198.18.0.0/15"
                  :disabled="!singboxProfileForm.fakeDnsEnabled"
                />
              </n-form-item>
            </div>

            <div class="form-grid">
              <n-form-item :label="props.t('setting.singboxProfile.fakeDnsIpv6Range')">
                <n-input
                  v-model:value="singboxProfileForm.fakeDnsIpv6Range"
                  placeholder="fc00::/18"
                  :disabled="!singboxProfileForm.fakeDnsEnabled"
                />
              </n-form-item>
            </div>

            <div class="form-section-title">{{ props.t('setting.singboxProfile.dnsTitle') }}</div>

            <div class="form-grid">
              <n-form-item :label="props.t('setting.singboxProfile.dnsProxy')">
                <n-input
                  v-model:value="singboxProfileForm.dnsProxy"
                  placeholder="https://1.1.1.1/dns-query"
                />
              </n-form-item>
              <n-form-item :label="props.t('setting.singboxProfile.dnsCn')">
                <n-input
                  v-model:value="singboxProfileForm.dnsCn"
                  placeholder="h3://dns.alidns.com/dns-query"
                />
              </n-form-item>
            </div>

            <div class="form-grid">
              <n-form-item :label="props.t('setting.singboxProfile.dnsResolver')">
                <n-input
                  v-model:value="singboxProfileForm.dnsResolver"
                  placeholder="114.114.114.114"
                />
              </n-form-item>
              <n-form-item :label="props.t('setting.singboxProfile.urltestUrl')">
                <n-input
                  v-model:value="singboxProfileForm.urltestUrl"
                  placeholder="http://cp.cloudflare.com/generate_204"
                />
              </n-form-item>
            </div>

            <n-button
              type="primary"
              block
              :loading="savingSingboxProfile"
              @click="saveSingboxProfileSettings"
            >
              {{ props.t('setting.singboxProfile.save') }}
            </n-button>
          </n-form>
        </div>
      </transition>
    </div>

    <div class="settings-group">
      <div class="group-header" @click="toggleSection('dashboard')">
        <div class="group-icon dashboard-icon">
          <n-icon :size="18"><GridOutline /></n-icon>
        </div>
        <div class="group-title-area">
          <div class="group-title">{{ extraLabels.dashboardTitle }}</div>
        </div>
        <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.dashboard }">
          <ChevronDownOutline />
        </n-icon>
      </div>

      <transition name="collapse">
        <div v-if="expandedSections.dashboard" class="group-body">
          <div class="form-section-title">{{ extraLabels.proxyPrefs }}</div>
            <div class="form-grid">
              <n-form-item :label="extraLabels.proxyOrdering">
                <n-select v-model:value="proxyStore.ordering" :options="proxyOrderingOptions" />
              </n-form-item>
              <n-form-item :label="extraLabels.proxyDisplay">
                <n-select v-model:value="proxyStore.displayMode" :options="proxyDisplayOptions" />
              </n-form-item>
            </div>

            <div class="toggles-grid">
              <div class="toggle-item">
                <span class="toggle-label">{{ extraLabels.proxyHideUnavailable }}</span>
                <n-switch v-model:value="proxyStore.hideUnavailable" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ extraLabels.proxyAutoClose }}</span>
                <n-switch v-model:value="proxyStore.autoCloseConnections" />
              </div>
            </div>

            <div class="form-grid">
              <n-form-item :label="extraLabels.latencyTimeout">
                <n-input-number
                  v-model:value="proxyStore.latencyTimeoutMs"
                :min="1000"
                :max="20000"
                :step="500"
                />
              </n-form-item>
              <n-form-item :label="extraLabels.latencyUrl">
                <n-input v-model:value="proxyStore.latencyTestUrl" :placeholder="props.appStore.singboxUrltestUrl" />
              </n-form-item>
            </div>

            <div class="form-section-title">{{ extraLabels.connectionPrefs }}</div>
            <div class="form-grid">
              <n-form-item :label="extraLabels.connectionGrouping">
                <n-select v-model:value="connectionStore.groupingKey" :options="connectionGroupingOptions" clearable />
              </n-form-item>
              <n-form-item :label="extraLabels.connectionSort">
                <n-select v-model:value="connectionStore.sortKey" :options="connectionSortOptions" />
              </n-form-item>
            </div>

            <div class="toggles-grid">
              <div class="toggle-item">
                <span class="toggle-label">{{ extraLabels.connectionSortDesc }}</span>
                <n-switch v-model:value="connectionStore.sortDesc" />
              </div>
              <div class="toggle-item">
                <span class="toggle-label">{{ extraLabels.connectionQuickFilter }}</span>
                <n-switch v-model:value="connectionStore.quickFilterEnabled" />
              </div>
            </div>

            <div class="form-section-title">{{ extraLabels.logPrefs }}</div>
            <div class="form-grid">
              <n-form-item :label="extraLabels.logGrouping">
                <n-select v-model:value="logStore.groupingKey" :options="logGroupingOptions" clearable />
              </n-form-item>
              <n-form-item :label="extraLabels.logSort">
                <n-select v-model:value="logStore.sortKey" :options="logSortOptions" />
              </n-form-item>
            </div>

            <div class="form-grid">
              <n-form-item :label="extraLabels.logMaxRows">
                <n-input-number
                  v-model:value="logStore.maxLogs"
                :min="100"
                :max="5000"
                :step="100"
                />
              </n-form-item>
            </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue'
import {
  GlobeOutline,
  OptionsOutline,
  SettingsOutline,
  InformationCircleOutline,
  ChevronDownOutline,
  LayersOutline,
  GridOutline,
} from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import type { useAppStore } from '@/stores'
import { useAdvancedSettingsForm } from '@/views/setting/useAdvancedSettingsForm'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { useConnectionStore } from '@/stores/kernel/ConnectionStore'
import { useLogStore } from '@/stores/kernel/LogStore'
import { useI18n } from 'vue-i18n'

type LabeledOption = { label: string; value: string }
type AppStoreLike = ReturnType<typeof useAppStore>

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  appStore: AppStoreLike
  tunStackOptions: LabeledOption[]
  usingOriginalConfig: boolean
  onIpVersionChange: (value: boolean) => void | Promise<void>
  onLanAccessChange: (value: boolean) => void | Promise<void>
  showPortSettings: () => void
}>()

const message = useMessage()
const { locale } = useI18n()
const proxyStore = useProxyStore()
const connectionStore = useConnectionStore()
const logStore = useLogStore()

const expandedSections = reactive({
  network: true,
  proxy: false,
  profile: false,
  dashboard: false,
})

const toggleSection = (key: keyof typeof expandedSections) => {
  expandedSections[key] = !expandedSections[key]
}

const {
  savingAdvanced,
  proxyAdvancedForm,
  savingSingboxProfile,
  singboxProfileForm,
  defaultOutboundOptions,
  downloadDetourOptions,
  fakeDnsFilterOptions,
  saveProxyAdvancedSettings,
  saveSingboxProfileSettings,
} = useAdvancedSettingsForm({
  appStore: props.appStore,
  message,
  t: props.t,
})

const extraLabels = computed(() => {
  const zh = locale.value.startsWith('zh')
  return {
    dashboardTitle: zh ? '看板与列表偏好' : 'Dashboard & List Preferences',
    proxyPrefs: zh ? '代理页偏好' : 'Proxy Preferences',
    proxyOrdering: zh ? '节点排序' : 'Node Ordering',
    proxyDisplay: zh ? '节点展示模式' : 'Node Display Mode',
    proxyHideUnavailable: zh ? '隐藏不可用节点' : 'Hide unavailable nodes',
    proxyAutoClose: zh ? '切换节点后关闭现有连接' : 'Close existing connections after switch',
    latencyTimeout: zh ? '测速超时(ms)' : 'Latency timeout (ms)',
    latencyUrl: zh ? '测速 URL' : 'Latency URL',
    connectionPrefs: zh ? '连接页偏好' : 'Connections Preferences',
    connectionGrouping: zh ? '默认分组' : 'Default Grouping',
    connectionSort: zh ? '默认排序' : 'Default Sort',
    connectionSortDesc: zh ? '降序排序' : 'Sort descending',
    connectionQuickFilter: zh ? '启用快速筛选' : 'Enable quick filter',
    logPrefs: zh ? '日志页偏好' : 'Logs Preferences',
    logGrouping: zh ? '日志分组' : 'Log Grouping',
    logSort: zh ? '日志排序' : 'Log Sort',
    logMaxRows: zh ? '最大日志条数' : 'Maximum log rows',
  }
})

const proxyOrderingOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '原始顺序' : 'Natural', value: 'natural' },
    { label: zh ? '按延迟' : 'Latency', value: 'latency' },
    { label: zh ? '按名称' : 'Name', value: 'name' },
  ]
})

const proxyDisplayOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '卡片' : 'Card', value: 'card' },
    { label: zh ? '紧凑列表' : 'Compact List', value: 'list' },
  ]
})

const connectionGroupingOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '按规则' : 'Rule', value: 'rule' },
    { label: zh ? '按进程' : 'Process', value: 'process' },
    { label: zh ? '按目标' : 'Destination', value: 'host' },
    { label: zh ? '按来源 IP' : 'Source IP', value: 'sourceIP' },
  ]
})

const connectionSortOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '连接时间' : 'Start time', value: 'start' },
    { label: zh ? '下载速度' : 'Download speed', value: 'downloadSpeed' },
    { label: zh ? '上传速度' : 'Upload speed', value: 'uploadSpeed' },
    { label: zh ? '目标地址' : 'Destination', value: 'host' },
    { label: zh ? '规则' : 'Rule', value: 'rule' },
    { label: zh ? '进程' : 'Process', value: 'process' },
  ]
})

const logGroupingOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '按级别' : 'Level', value: 'type' },
    { label: zh ? '按日期' : 'Date', value: 'date' },
  ]
})

const logSortOptions = computed(() => {
  const zh = locale.value.startsWith('zh')
  return [
    { label: zh ? '序号' : 'Sequence', value: 'seq' },
    { label: zh ? '级别' : 'Level', value: 'type' },
    { label: zh ? '时间' : 'Time', value: 'timestamp' },
  ]
})
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-group {
  border: 1px solid var(--panel-border);
  border-radius: 14px;
  overflow: hidden;
  background: var(--bg-secondary);
}

.group-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  cursor: pointer;
  transition: background 0.15s ease;
  user-select: none;
}

.group-header:hover {
  background: var(--bg-tertiary);
}

.group-icon {
  width: 34px;
  height: 34px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.group-icon.net-icon {
  background: rgba(14, 165, 233, 0.12);
  color: #0ea5e9;
}

.group-icon.proxy-icon {
  background: rgba(168, 85, 247, 0.12);
  color: #a855f7;
}

.group-icon.profile-icon {
  background: rgba(16, 185, 129, 0.12);
  color: #10b981;
}

.group-icon.dashboard-icon {
  background: rgba(245, 158, 11, 0.12);
  color: #f59e0b;
}

.group-title-area {
  flex: 1;
  min-width: 0;
}

.group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.collapse-arrow {
  color: var(--text-tertiary);
  transition: transform 0.2s ease;
  flex-shrink: 0;
}

.collapse-arrow.expanded {
  transform: rotate(180deg);
}

.group-body {
  padding: 4px 16px 16px;
  border-top: 1px solid var(--panel-border);
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 0;
}

.setting-row + .setting-row {
  border-top: 1px solid var(--panel-border);
}

.setting-info {
  flex: 1;
  min-width: 0;
}

.setting-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.info-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 14px;
  border-radius: 8px;
  background: rgba(14, 165, 233, 0.08);
  border: 1px solid rgba(14, 165, 233, 0.15);
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 12px;
}

.form-section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin: 16px 0 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--panel-border);
}

.form-section-title:first-child {
  margin-top: 8px;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
}

.toggles-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
  padding: 12px;
  background: var(--bg-tertiary);
  border-radius: 10px;
  margin: 4px 0;
}

.toggle-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.toggle-label {
  font-size: 13px;
  color: var(--text-secondary);
}

.collapse-enter-active,
.collapse-leave-active {
  transition: all 0.25s ease;
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
  padding-top: 0;
  padding-bottom: 0;
}

@media (max-width: 768px) {
  .form-grid {
    grid-template-columns: 1fr;
  }

  .toggles-grid {
    grid-template-columns: 1fr;
  }
}
</style>
