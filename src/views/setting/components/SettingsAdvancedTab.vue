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
          <!--
            节点测速 URL：原 Clash API 时代用 proxyStore.latencyTestUrl 单独配置，
            官方 gRPC API 后由内核直接使用 singboxUrltestUrl 配置；该项已隐藏。
          -->
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
        <div class="setting-hint">{{ extraLabels.logRetentionHint }}</div>
      </div>
    </transition>

    <!-- sing-box 1.14 实验性 / 高级选项 -->
    <h3 class="setting-section-title">{{ extraLabels.experimentalTitle }}</h3>
    <div class="collapsible-header" @click="toggleSection('experimental')">
      <span class="collapsible-label">{{ extraLabels.experimentalLabel }}</span>
      <n-icon :size="16" class="collapse-arrow" :class="{ expanded: expandedSections.experimental }">
        <ChevronDownOutline />
      </n-icon>
    </div>
    <transition name="collapse">
      <div v-if="expandedSections.experimental" class="collapsible-body">
        <div class="setting-alert info">
          <n-icon :size="16"><InformationCircleOutline /></n-icon>
          <span>{{ extraLabels.experimentalHint }}</span>
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
            <n-input
              v-model:value="singboxExperimentalForm.dnsTimeout"
              :placeholder="'5s'"
            />
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

          <!-- TLS 抗指纹（仅 Windows x64/x86 + Admin） -->
          <div class="form-section-title">{{ extraLabels.tlsTitle }}</div>
          <div class="setting-toggles-grid">
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
import { computed, reactive } from 'vue'
import {
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
    logRetentionHint: zh
      ? '仅控制前端界面展示的日志条数；磁盘上的 sing-box.log 由内核启动时自动滚动（超过 10MB 保留最近 3 份）。'
      : 'Controls only the number of log rows shown in the UI. The on-disk sing-box.log is rotated automatically on kernel start (kept to last 3 files after 10MB).',
    // === sing-box 1.14 实验性 / 高级选项 ===
    experimentalTitle: zh ? '高级（1.14 实验性）' : 'Advanced (1.14 Experimental)',
    experimentalLabel: zh ? '1.14 新增能力' : '1.14 New Capabilities',
    experimentalHint: zh
      ? '以下选项对应 sing-box 1.14 内核能力。修改后需要重启内核生效。'
      : 'These options correspond to sing-box 1.14 capabilities. Kernel restart is required to take effect.',
    dns114Title: zh ? 'DNS 1.14 增强' : 'DNS 1.14 Enhancements',
    dnsOptimisticCache: zh ? '乐观 DNS 缓存（降低重复查询延迟）' : 'Optimistic DNS cache (reduces repeat query latency)',
    dnsUseMdns: zh ? '启用 mDNS（*.local / link-local）' : 'Enable mDNS (*.local / link-local)',
    dnsTimeout: zh ? 'DNS 超时（如 5s）' : 'DNS timeout (e.g. 5s)',
    hysteria2Title: zh ? 'Hysteria2 抗指纹' : 'Hysteria2 Fingerprint Resistance',
    hysteria2ObfsType: zh ? '混淆类型' : 'Obfuscation type',
    hysteria2DisableChromeParrot: zh ? '关闭 Chrome QUIC 指纹（Ed25519 证书时需开）' : 'Disable Chrome QUIC fingerprint (required for Ed25519 servers)',
    tlsTitle: zh ? 'TLS 抗指纹' : 'TLS Fingerprint Resistance',
    enableTlsSpoof: zh ? '启用 TLS spoof（SNI 诱骗，仅 Windows x64/x86 + Admin）' : 'Enable TLS spoof (SNI deception, Windows x64/x86 + Admin only)',
    clashModeTitle: zh ? 'Clash 模式' : 'Clash Mode',
    clashMode: zh ? '默认模式' : 'Default mode',
    enableWebDashboard: zh ? '启用 sing-box-dashboard Web 面板' : 'Enable sing-box-dashboard Web panel',
    tailscaleTitle: zh ? 'Tailscale 模式（实验性）' : 'Tailscale Mode (Experimental)',
    enableTailscaleEndpoint: zh ? '启用 Tailscale endpoint' : 'Enable Tailscale endpoint',
    tailscaleRunSshServer: zh ? '同时启用 Tailscale SSH server（tailnet:22）' : 'Also enable Tailscale SSH server (tailnet:22)',
    tailscaleTaildropDirectory: zh ? 'Taildrop 收件箱目录' : 'Taildrop inbox directory',
    save: zh ? '保存 1.14 实验性选项' : 'Save 1.14 Experimental Options',
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
