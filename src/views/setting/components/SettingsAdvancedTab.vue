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
  </div>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import {
  GlobeOutline,
  OptionsOutline,
  SettingsOutline,
  InformationCircleOutline,
  ChevronDownOutline,
  LayersOutline,
} from '@vicons/ionicons5'
import { useMessage } from 'naive-ui'
import type { useAppStore } from '@/stores'
import { useAdvancedSettingsForm } from '@/views/setting/useAdvancedSettingsForm'

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

const expandedSections = reactive({
  network: true,
  proxy: false,
  profile: false,
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
