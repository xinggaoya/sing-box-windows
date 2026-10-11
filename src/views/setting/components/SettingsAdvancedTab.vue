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
        <template #icon
          ><n-icon :size="14"><SettingsOutline /></n-icon
        ></template>
        {{ props.t('setting.network.configure') }}
      </n-button>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.network.allowLanAccess') }}</div>
        <div class="setting-desc">{{ props.t('setting.network.allowLanAccessDesc') }}</div>
      </div>
      <n-switch :value="props.appStore.allowLanAccess" @update:value="props.onLanAccessChange" />
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
              <n-select
                v-model:value="proxyAdvancedForm.tunStack"
                :options="props.tunStackOptions"
              />
            </n-form-item>
          </div>

          <n-form-item :label="props.t('setting.proxyAdvanced.tunDnsMode')">
            <n-select v-model:value="proxyAdvancedForm.tunDnsMode" :options="tunDnsModeOptions" />
          </n-form-item>
          <div class="setting-hint">{{ props.t('setting.proxyAdvanced.tunDnsModeHint') }}</div>

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
              <span class="setting-toggle-label">{{
                props.t('setting.proxyAdvanced.enableIpv6')
              }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunEnableIpv6" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{
                props.t('setting.proxyAdvanced.autoRoute')
              }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunAutoRoute" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{
                props.t('setting.proxyAdvanced.strictRoute')
              }}</span>
              <n-switch v-model:value="proxyAdvancedForm.tunStrictRoute" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{
                props.t('setting.proxyAdvanced.tunSelfHeal')
              }}</span>
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
              <span class="setting-toggle-label">{{
                props.t('setting.singboxProfile.blockAds')
              }}</span>
              <n-switch v-model:value="singboxProfileForm.blockAds" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{
                props.t('setting.singboxProfile.dnsHijack')
              }}</span>
              <n-switch v-model:value="singboxProfileForm.dnsHijack" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{
                props.t('setting.singboxProfile.enableAppGroups')
              }}</span>
              <n-switch v-model:value="singboxProfileForm.enableAppGroups" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{
                props.t('setting.singboxProfile.fakeDnsEnabled')
              }}</span>
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
          <!--
            节点测速 URL：原 Clash API 时代用 proxyStore.latencyTestUrl 单独配置，
            官方 gRPC API 后由内核直接使用 singboxUrltestUrl 配置；该项已隐藏。
          -->
        </div>

        <div class="form-section-title">{{ extraLabels.logRetentionPrefs }}</div>
        <div class="setting-form-grid">
          <n-form-item :label="extraLabels.logMaxRows">
            <n-input-number v-model:value="logStore.maxLogs" :min="100" :max="5000" :step="100" />
          </n-form-item>
        </div>
        <div class="setting-hint">{{ extraLabels.logRetentionHint }}</div>
      </div>
    </transition>

    <!-- sing-box 1.14 实验性 / 高级选项 -->
    <h3 class="setting-section-title">{{ extraLabels.experimentalTitle }}</h3>
    <div class="collapsible-header" @click="toggleSection('experimental')">
      <span class="collapsible-label">{{ extraLabels.experimentalLabel }}</span>
      <n-icon
        :size="16"
        class="collapse-arrow"
        :class="{ expanded: expandedSections.experimental }"
      >
        <ChevronDownOutline />
      </n-icon>
    </div>
    <transition name="collapse">
      <div v-if="expandedSections.experimental" class="collapsible-body">
        <div class="setting-alert info">
          <n-icon :size="16"><InformationCircleOutline /></n-icon>
          <span>{{ extraLabels.experimentalHint }}</span>
        </div>
        <div class="setting-alert warning">
          <n-icon :size="16"><InformationCircleOutline /></n-icon>
          <span>{{ extraLabels.deprecatedNotice }}</span>
        </div>

        <n-form label-placement="top" class="advanced-form">
          <!-- DNS 1.14 增强 -->
          <div class="form-section-title">{{ extraLabels.dns114Title }}</div>
          <div class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ extraLabels.dnsOptimisticCache }}</span>
              <n-switch v-model:value="singboxExperimentalForm.dnsOptimisticCache" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ extraLabels.dnsUseMdns }}</span>
              <n-switch v-model:value="singboxExperimentalForm.dnsUseMdns" />
            </div>
          </div>
          <n-form-item :label="extraLabels.dnsTimeout">
            <n-input v-model:value="singboxExperimentalForm.dnsTimeout" :placeholder="'5s'" />
          </n-form-item>

          <!-- Hysteria2 抗指纹 -->
          <div class="form-section-title">{{ extraLabels.hysteria2Title }}</div>
          <div class="setting-form-grid">
            <n-form-item :label="extraLabels.hysteria2ObfsType">
              <n-select
                v-model:value="singboxExperimentalForm.hysteria2ObfsType"
                :options="hysteria2ObfsTypeOptions"
              />
            </n-form-item>
            <n-form-item :label="extraLabels.hysteria2DisableChromeParrot">
              <n-switch v-model:value="singboxExperimentalForm.hysteria2DisableChromeParrot" />
            </n-form-item>
          </div>

          <!-- TLS 抗指纹（Windows ARM64 内核不支持，自动隐藏入口） -->
          <div v-if="tlsSpoofAvailable" class="form-section-title">{{ extraLabels.tlsTitle }}</div>
          <div v-if="tlsSpoofAvailable" class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ extraLabels.enableTlsSpoof }}</span>
              <n-switch v-model:value="singboxExperimentalForm.enableTlsSpoof" />
            </div>
          </div>

          <!-- Clash Mode 持久化 -->
          <div class="form-section-title">{{ extraLabels.clashModeTitle }}</div>
          <n-form-item :label="extraLabels.clashMode">
            <n-select
              v-model:value="singboxExperimentalForm.clashMode"
              :options="clashModeOptions"
            />
          </n-form-item>

          <!-- Web Dashboard -->
          <div class="form-section-title">{{ extraLabels.dashboardTitle }}</div>
          <div class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ extraLabels.enableWebDashboard }}</span>
              <n-switch v-model:value="singboxExperimentalForm.enableWebDashboard" />
            </div>
          </div>

          <!-- Tailscale -->
          <div class="form-section-title">{{ extraLabels.tailscaleTitle }}</div>
          <div class="setting-toggles-grid">
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ extraLabels.enableTailscaleEndpoint }}</span>
              <n-switch v-model:value="singboxExperimentalForm.enableTailscaleEndpoint" />
            </div>
            <div class="setting-toggle-item">
              <span class="setting-toggle-label">{{ extraLabels.tailscaleRunSshServer }}</span>
              <n-switch v-model:value="singboxExperimentalForm.tailscaleRunSshServer" />
            </div>
          </div>
          <n-form-item :label="extraLabels.tailscaleTaildropDirectory">
            <n-input
              v-model:value="singboxExperimentalForm.tailscaleTaildropDirectory"
              :placeholder="'Taildrop'"
            />
          </n-form-item>

          <n-button
            type="primary"
            block
            :loading="savingSingboxExperimental"
            @click="saveSingboxExperimentalSettings"
          >
            {{ extraLabels.save }}
          </n-button>
        </n-form>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { SettingsOutline, InformationCircleOutline, ChevronDownOutline } from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import type { useAppStore } from '@/stores'
import { useAdvancedSettingsForm } from '@/views/setting/useAdvancedSettingsForm'
import { useProxyStore } from '@/stores/kernel/ProxyStore'
import { useLogStore } from '@/stores/kernel/LogStore'
import { useI18n } from 'vue-i18n'
import { systemService } from '@/services/system-service'

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
const { t } = useI18n()
const proxyStore = useProxyStore()
const logStore = useLogStore()

const expandedSections = reactive({
  proxy: false,
  profile: false,
  dashboard: false,
  experimental: false,
})

const toggleSection = (key: keyof typeof expandedSections) => {
  expandedSections[key] = !expandedSections[key]
}

const {
  savingAdvanced,
  proxyAdvancedForm,
  savingSingboxProfile,
  singboxProfileForm,
  savingSingboxExperimental,
  singboxExperimentalForm,
  defaultOutboundOptions,
  downloadDetourOptions,
  fakeDnsFilterOptions,
  tunDnsModeOptions,
  hysteria2ObfsTypeOptions,
  clashModeOptions,
  saveProxyAdvancedSettings,
  saveSingboxProfileSettings,
  saveSingboxExperimentalSettings,
} = useAdvancedSettingsForm({
  appStore: props.appStore,
  message,
  t: props.t,
})

// tls.spoof 依赖 WinDivert 内核驱动，Windows ARM64 内核不支持；
// 平台检测不可用（如旧版后端）时保守显示开关
const systemArch = ref('')
onMounted(async () => {
  try {
    const info = await systemService.getDetailedPlatformInfo()
    systemArch.value = info.arch
  } catch {
    systemArch.value = ''
  }
})
const tlsSpoofAvailable = computed(() => {
  if (!systemArch.value) return true
  // 后端架构标识：新后端返回 arm64，旧后端返回 aarch64
  const isArm64 = ['arm64', 'aarch64'].includes(systemArch.value)
  return !(isArm64 && navigator.userAgent.includes('Windows'))
})

// 文案统一取自 locale 语言包（`setting.advanced.*`）。
// 原实现用 `locale.startsWith('zh') ? '中文' : 'English'` 内联了约 40 条，
// 导致 ja-JP / ru-RU 用户直接看到英文，且翻译无法被 i18n 校验脚本覆盖。
const extraLabels = computed(() => ({
  dashboardTitle: t('setting.advanced.dashboardTitle'),
  proxyPrefs: t('setting.advanced.proxyPrefs'),
  proxyOrdering: t('setting.advanced.proxyOrdering'),
  proxyDisplay: t('setting.advanced.proxyDisplay'),
  proxyHideUnavailable: t('setting.advanced.proxyHideUnavailable'),
  proxyAutoClose: t('setting.advanced.proxyAutoClose'),
  latencyTimeout: t('setting.advanced.latencyTimeout'),
  latencyUrl: t('setting.advanced.latencyUrl'),
  logRetentionPrefs: t('setting.advanced.logRetentionPrefs'),
  logMaxRows: t('setting.advanced.logMaxRows'),
  logRetentionHint: t('setting.advanced.logRetentionHint'),
  // === sing-box 1.14 实验性 / 高级选项 ===
  experimentalTitle: t('setting.advanced.experimentalTitle'),
  experimentalLabel: t('setting.advanced.experimentalLabel'),
  experimentalHint: t('setting.advanced.experimentalHint'),
  deprecatedNotice: t('setting.advanced.deprecatedNotice'),
  dns114Title: t('setting.advanced.dns114Title'),
  dnsOptimisticCache: t('setting.advanced.dnsOptimisticCache'),
  dnsUseMdns: t('setting.advanced.dnsUseMdns'),
  dnsTimeout: t('setting.advanced.dnsTimeout'),
  hysteria2Title: t('setting.advanced.hysteria2Title'),
  hysteria2ObfsType: t('setting.advanced.hysteria2ObfsType'),
  hysteria2DisableChromeParrot: t('setting.advanced.hysteria2DisableChromeParrot'),
  tlsTitle: t('setting.advanced.tlsTitle'),
  enableTlsSpoof: t('setting.advanced.enableTlsSpoof'),
  clashModeTitle: t('setting.advanced.clashModeTitle'),
  clashMode: t('setting.advanced.clashMode'),
  enableWebDashboard: t('setting.advanced.enableWebDashboard'),
  tailscaleTitle: t('setting.advanced.tailscaleTitle'),
  enableTailscaleEndpoint: t('setting.advanced.enableTailscaleEndpoint'),
  tailscaleRunSshServer: t('setting.advanced.tailscaleRunSshServer'),
  tailscaleTaildropDirectory: t('setting.advanced.tailscaleTaildropDirectory'),
  save: t('setting.advanced.save'),
}))

const proxyOrderingOptions = computed(() => [
  { label: t('setting.advanced.orderNatural'), value: 'natural' },
  { label: t('setting.advanced.orderLatency'), value: 'latency' },
  { label: t('setting.advanced.orderName'), value: 'name' },
])

const proxyDisplayOptions = computed(() => [
  { label: t('setting.advanced.displayCard'), value: 'card' },
  { label: t('setting.advanced.displayList'), value: 'list' },
])
</script>

<style scoped>
.collapsible-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-3) 0;
  cursor: pointer;
  user-select: none;
  border-top: 1px solid var(--border-color);
}

.collapsible-header:hover .collapsible-label {
  color: var(--primary-color);
}

.collapsible-label {
  font-size: var(--text-xs);
  font-weight: 700;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  transition: color var(--transition-fast);
}

.collapse-arrow {
  color: var(--text-tertiary);
  transition: transform var(--transition-fast);
  flex-shrink: 0;
}

.collapse-arrow.expanded {
  transform: rotate(180deg);
}

.collapsible-body {
  padding: 0 0 var(--space-4);
}

.form-section-title {
  font-size: var(--text-xs);
  font-weight: 600;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: var(--space-4) 0 var(--space-2);
  padding-bottom: var(--space-2);
  border-bottom: 1px solid var(--border-color);
}

.form-section-title:first-child {
  margin-top: var(--space-2);
}

.setting-hint {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: 1.5;
  margin: var(--space-1) 0 0;
}

.collapse-enter-active,
.collapse-leave-active {
  transition: all var(--transition-base);
  overflow: hidden;
}

.collapse-enter-from,
.collapse-leave-to {
  opacity: 0;
  max-height: 0;
}
</style>
