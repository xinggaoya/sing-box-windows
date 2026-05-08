<template>
  <div class="settings-panel">
    <div class="settings-group">
      <div class="group-header">
        <div class="group-icon theme-icon">
          <n-icon :size="18"><ColorPaletteOutline /></n-icon>
        </div>
        <div>
          <div class="group-title">{{ props.t('setting.theme.title') }}</div>
          <div class="group-desc">{{ props.t('setting.theme.modeDesc') }}</div>
        </div>
      </div>

      <div class="group-body">
        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">{{ props.t('setting.theme.mode') }}</div>
          </div>
          <div class="theme-toggle-group">
            <button
              v-for="mode in themeModes"
              :key="mode.value"
              class="theme-toggle-btn"
              :class="{ active: props.themeStore.mode === mode.value }"
              @click="props.onThemeModeChange(mode.value as ThemeMode)"
            >
              <n-icon :size="16"><component :is="mode.icon" /></n-icon>
              {{ mode.label }}
            </button>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">{{ props.t('setting.theme.accent') }}</div>
          </div>
          <div class="accent-row">
            <n-color-picker
              :value="props.themeStore.accentColor"
              :modes="['hex']"
              size="small"
              :show-alpha="false"
              @update:value="props.onAccentChange"
            />
            <div class="accent-presets">
              <button
                v-for="color in props.accentPresets"
                :key="color"
                class="accent-dot"
                :class="{ active: color === props.themeStore.accentColor }"
                :style="{ background: color }"
                @click="props.selectAccentPreset(color)"
              ></button>
            </div>
          </div>
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">{{ props.t('setting.theme.compactMode') }}</div>
            <div class="setting-desc">{{ props.t('setting.theme.compactDesc') }}</div>
          </div>
          <n-switch
            :value="props.themeStore.compactMode"
            @update:value="props.onCompactModeChange"
          />
        </div>
      </div>
    </div>

    <div class="settings-group">
      <div class="group-header">
        <div class="group-icon lang-icon">
          <n-icon :size="18"><GlobeOutline /></n-icon>
        </div>
        <div>
          <div class="group-title">{{ props.t('setting.general.title') }}</div>
          <div class="group-desc">{{ props.t('setting.language.description') }}</div>
        </div>
      </div>

      <div class="group-body">
        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">{{ props.t('setting.language.title') }}</div>
          </div>
          <n-select
            :value="props.localeStore.locale"
            :options="props.languageOptions"
            size="small"
            style="width: 160px"
            @update:value="props.onChangeLanguage"
          />
        </div>
      </div>
    </div>

    <div class="settings-group">
      <div class="group-header">
        <div class="group-icon startup-icon">
          <n-icon :size="18"><PowerOutline /></n-icon>
        </div>
        <div>
          <div class="group-title">{{ props.t('setting.startup.title') }}</div>
          <div class="group-desc">{{ props.t('setting.autoStart.appDesc') }}</div>
        </div>
      </div>

      <div class="group-body">
        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">{{ props.t('setting.autoStart.app') }}</div>
          </div>
          <n-switch :value="props.autoStart" @update:value="props.onAutoStartChange" />
        </div>

        <div v-if="props.autoStart" class="setting-row sub-setting">
          <div class="setting-info">
            <div class="setting-label">
              {{ props.t('setting.startup.autoHideToTrayOnAutostart') }}
            </div>
            <div class="setting-desc">
              {{ props.t('setting.startup.autoHideToTrayOnAutostartDesc') }}
            </div>
          </div>
          <n-switch
            :value="props.autoHideToTrayOnAutostart"
            @update:value="props.onAutoHideToTrayOnAutostartChange"
          />
        </div>

        <div class="setting-row">
          <div class="setting-info">
            <div class="setting-label">{{ props.t('setting.startup.closeBehavior') }}</div>
            <div class="setting-desc">{{ props.t('setting.startup.closeBehaviorDesc') }}</div>
          </div>
          <n-select
            :value="props.trayCloseBehavior"
            :options="props.trayCloseBehaviorOptions"
            size="small"
            style="width: 160px"
            @update:value="props.onTrayCloseBehaviorChange"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  ColorPaletteOutline,
  GlobeOutline,
  PowerOutline,
  SunnyOutline,
  MoonOutline,
  DesktopOutline,
} from '@vicons/ionicons5'
import type { Locale } from '@/stores/app/LocaleStore'
import type { ThemeMode } from '@/stores/app/ThemeStore'
import type { TrayCloseBehavior } from '@/stores/app/AppStore'
import type { useLocaleStore, useThemeStore } from '@/stores'

type LocaleStoreLike = ReturnType<typeof useLocaleStore>
type ThemeStoreLike = ReturnType<typeof useThemeStore>

interface Option<T extends string = string> {
  label: string
  value: T
}

const props = defineProps<{
  t: (key: string, params?: Record<string, string | number>) => string
  localeStore: LocaleStoreLike
  themeStore: ThemeStoreLike
  autoStart: boolean
  autoHideToTrayOnAutostart: boolean
  trayCloseBehavior: TrayCloseBehavior
  languageOptions: Option<Locale>[]
  trayCloseBehaviorOptions: Option<TrayCloseBehavior>[]
  accentPresets: string[]
  onAutoStartChange: (value: boolean) => void | Promise<void>
  onAutoHideToTrayOnAutostartChange: (value: boolean) => void | Promise<void>
  onTrayCloseBehaviorChange: (value: TrayCloseBehavior) => void | Promise<void>
  onChangeLanguage: (value: string) => void | Promise<void>
  onThemeModeChange: (value: ThemeMode) => void | Promise<void>
  onAccentChange: (value: string) => void | Promise<void>
  selectAccentPreset: (value: string) => void | Promise<void>
  onCompactModeChange: (value: boolean) => void | Promise<void>
}>()

const themeModes = computed(() => [
  {
    value: 'system',
    label: props.t('setting.theme.system'),
    icon: DesktopOutline,
  },
  {
    value: 'light',
    label: props.t('setting.theme.light'),
    icon: SunnyOutline,
  },
  {
    value: 'dark',
    label: props.t('setting.theme.dark'),
    icon: MoonOutline,
  },
])
</script>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
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
  padding: 14px 18px;
  border-bottom: 1px solid var(--panel-border);
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

.group-icon.theme-icon {
  background: linear-gradient(135deg, rgba(168, 85, 247, 0.15), rgba(236, 72, 153, 0.15));
  color: #a855f7;
}

.group-icon.lang-icon {
  background: rgba(14, 165, 233, 0.12);
  color: #0ea5e9;
}

.group-icon.startup-icon {
  background: rgba(16, 185, 129, 0.12);
  color: #10b981;
}

.group-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.group-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 1px;
}

.group-body {
  padding: 6px 0;
}

.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 10px 18px;
  transition: background 0.15s ease;
}

.setting-row:hover {
  background: var(--bg-tertiary);
}

.setting-row.sub-setting {
  padding-left: 36px;
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

.theme-toggle-group {
  display: flex;
  gap: 4px;
  background: var(--bg-tertiary);
  border-radius: 8px;
  padding: 3px;
}

.theme-toggle-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border-radius: 6px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.theme-toggle-btn:hover {
  color: var(--text-primary);
}

.theme-toggle-btn.active {
  background: var(--bg-secondary);
  color: var(--primary-color);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
}

.accent-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.accent-presets {
  display: flex;
  gap: 6px;
}

.accent-dot {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  position: relative;
}

.accent-dot:hover {
  transform: scale(1.15);
}

.accent-dot.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px var(--bg-secondary), 0 2px 8px rgba(0, 0, 0, 0.2);
}
</style>
