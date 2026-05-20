<template>
  <div class="setting-section">
    <h3 class="setting-section-title">{{ props.t('setting.network.title') }}</h3>

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

    <h3 class="setting-section-title">{{ props.t('setting.proxyAdvanced.title') }}</h3>

    <div class="collapsible-header" @click="toggleSection('proxy')">
      <span class="collapsible-label">{{ props.t('setting.proxyAdvanced.systemBypass') }}</span>
      <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.proxy }">
        <ChevronDownOutline />
      </n-icon>
    </div>
    <transition name="collapse">
      <div v-if="expandedSections.proxy" class="collapsible-body">
        <n-form label-placement="top" class="advanced-form">
          <n-form-item :label="props.t('setting.proxyAdvanced.systemBypass')">
            <n-input
              v-model:value="proxyAdvancedForm.systemProxyBypass"
              type="textarea"
              :rows="3"
              :placeholder="props.t('setting.proxyAdvanced.systemBypassPlaceholder')"
            />
          </n-form-item>

          <div class="form-section-title">{{ props.t('setting.proxyAdvanced.tunTitle') }}</div>

          <div class="setting-form-grid">
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

          <div class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.enableIpv6') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.autoRoute') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.strictRoute') }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.proxyAdvanced.tunSelfHeal') }}</span>
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

    <h3 class="setting-section-title">{{ props.t('setting.singboxProfile.title') }}</h3>

    <div class="collapsible-header" @click="toggleSection('profile')">
      <span class="collapsible-label">{{ props.t('setting.singboxProfile.routingTitle') }}</span>
      <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.profile }">
        <ChevronDownOutline />
      </n-icon>
    </div>
    <transition name="collapse">
      <div v-if="expandedSections.profile" class="collapsible-body">
        <div v-if="props.usingOriginalConfig" class="setting-alert info">
          <n-icon :size="16"><InformationCircleOutline /></n-icon>
          <span>{{ props.t('setting.singboxProfile.originalConfigHint') }}</span>
        </div>

        <n-form label-placement="top" class="advanced-form">
          <div class="form-section-title">{{ props.t('setting.singboxProfile.routingTitle') }}</div>

          <div class="setting-form-grid">
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

          <div class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.singboxProfile.blockAds') }}</span>
              <n-switch v-model:value="singboxProfileForm.blockAds" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.singboxProfile.dnsHijack') }}</span>
              <n-switch v-model:value="singboxProfileForm.dnsHijack" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.singboxProfile.enableAppGroups') }}</span>
              <n-switch v-model:value="singboxProfileForm.enableAppGroups" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ props.t('setting.singboxProfile.fakeDnsEnabled') }}</span>
              <n-switch v-model:value="singboxProfileForm.fakeDnsEnabled" />
            </div>
          </div>

          <div class="form-section-title">{{ props.t('setting.singboxProfile.fakeDnsTitle') }}</div>

          <div class="setting-form-grid">
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

          <div class="setting-form-grid">
            <n-form-item :label="props.t('setting.singboxProfile.fakeDnsIpv6Range')">
              <n-input
                v-model:value="singboxProfileForm.fakeDnsIpv6Range"
                placeholder="fc00::/18"
                :disabled="!singboxProfileForm.fakeDnsEnabled"
              />
            </n-form-item>
          </div>

          <div class="form-section-title">{{ props.t('setting.singboxProfile.dnsTitle') }}</div>

          <div class="setting-form-grid">
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

          <div class="setting-form-grid">
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

    <h3 class="setting-section-title">{{ extraLabels.dashboardTitle }}</h3>

    <div class="collapsible-header" @click="toggleSection('dashboard')">
      <span class="collapsible-label">{{ extraLabels.proxyPrefs }}</span>
      <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.dashboard }">
        <ChevronDownOutline />
      </n-icon>
    </div>
    <transition name="collapse">
      <div v-if="expandedSections.dashboard" class="collapsible-body">
        <div class="form-section-title">{{ extraLabels.proxyPrefs }}</div>
        <div class="setting-form-grid">
          <n-form-item :label="extraLabels.proxyOrdering">
            <n-select v-model:value="proxyStore.ordering" :options="proxyOrderingOptions" />
          </n-form-item>
          <n-form-item :label="extraLabels.proxyDisplay">
            <n-select v-model:value="proxyStore.displayMode" :options="proxyDisplayOptions" />
          </n-form-item>
        </div>

        <div class="setting-toggles-grid">
          <div class="setting-toggle-item">
            <span class="setting-toggle-label">{{ extraLabels.proxyHideUnavailable }}</span>
            <n-switch v-model:value="proxyStore.hideUnavailable" />
          </div>
          <div class="setting-toggle-item">
            <span class="setting-toggle-label">{{ extraLabels.proxyAutoClose }}</span>
            <n-switch v-model:value="proxyStore.autoCloseConnections" />
          </div>
        </div>

        <div class="setting-form-grid">
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

        <div class="form-section-title">{{ extraLabels.logRetentionPrefs }}</div>
        <div class="setting-form-grid">
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
</template>

<script setup lang="ts">
import { computed, reactive } from 'vue'
import {
  GlobeOutline,
  OptionsOutline,
  SettingsOutline,
  InformationCircleOutline,
  ChevronDownOutline,
} from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import type { useAppStore } from '@/stores'
import { useAdvancedSettingsForm } from '@/views/setting/useAdvancedSettingsForm'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
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
const logStore = useLogStore()

const expandedSections = reactive({
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
    logRetentionPrefs: zh ? '日志保留' : 'Log Retention',
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
</script>

<style scoped>
.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
  cursor: pointer;
  user-select: none;
  border-top: 1px solid var(--border-color);
}

.collapsible-header:hover {
  color: var(--primary-color);
}

.collapsible-label {
  font-size: 13px;
  font-weight: 500;
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

.collapsible-body {
  padding: 0 0 16px;
}

.form-section-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin: 16px 0 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color);
}

.form-section-title:first-child {
  margin-top: 8px;
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
}
</style>
