<template>
  <div class="setting-section">
    <h3 class="setting-section-title">{{ props.t('setting.theme.title') }}</h3>

    <!-- 主题分组卡片：让设置项与背景产生层次感 -->
    <div class="setting-group-card">
      <div class="setting-row">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.theme.mode') }}</div>
          <div class="setting-desc">{{ props.t('setting.theme.modeDesc') }}</div>
        </div>
        <div class="theme-toggle-group">
          <button
            v-for="mode in themeModes"
            :key="mode.value"
            class="theme-toggle-btn"
            :class="{ active: props.themeStore.mode === mode.value }"
            @click="props.onThemeModeChange(mode.value as ThemeMode)"
          >
            <n-icon :size="14"><component :is="mode.icon" /></n-icon>
            {{ mode.label }}
          </button>
        </div>
      </div>

      <!-- 强调色：独占整行，颜色选择器与预设色块上下分排，避免拥挤 -->
      <div class="setting-row accent-block">
        <div class="setting-info">
          <div class="setting-label">{{ props.t('setting.theme.accent') }}</div>
          <div class="setting-desc">{{ props.t('setting.theme.accentDesc') }}</div>
        </div>
        <div class="accent-controls">
          <div class="accent-presets">
            <button
              v-for="color in props.accentPresets"
              :key="color"
              class="accent-dot"
              :class="{ active: color === props.themeStore.accentColor }"
              :style="{ background: color }"
              :title="color"
              @click="props.selectAccentPreset(color)"
            ></button>
          </div>
          <div class="accent-custom">
            <n-color-picker
              :value="props.themeStore.accentColor"
              :modes="['hex']"
              size="small"
              :show-alpha="false"
              @update:value="props.onAccentChange"
            />
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

    <h3 class="setting-section-title">{{ props.t('setting.general.title') }}</h3>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.language.title') }}</div>
        <div class="setting-desc">{{ props.t('setting.language.description') }}</div>
      </div>
      <n-select
        :value="props.localeStore.locale"
        :options="props.languageOptions"
        size="small"
        style="width: 160px"
        @update:value="props.onChangeLanguage"
      />
    </div>

    <h3 class="setting-section-title">{{ props.t('setting.startup.title') }}</h3>

    <div class="setting-row">
      <div class="setting-info">
        <div class="setting-label">{{ props.t('setting.autoStart.app') }}</div>
        <div class="setting-desc">{{ props.t('setting.autoStart.appDesc') }}</div>
      </div>
      <n-switch :value="props.autoStart" @update:value="props.onAutoStartChange" />
    </div>

    <div v-if="props.autoStart" class="setting-row" style="padding-left: 24px;">
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
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
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
.theme-toggle-group {
  display: flex;
  gap: 3px;
  background: var(--bg-surface-2);
  border-radius: var(--radius-sm);
  padding: 3px;
}

.theme-toggle-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 5px 10px;
  border-radius: var(--radius-xs);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: var(--text-xs);
  font-weight: 500;
  cursor: pointer;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
  white-space: nowrap;
}

.theme-toggle-btn:hover {
  color: var(--text-primary);
}

.theme-toggle-btn.active {
  background: var(--bg-elevated);
  color: var(--primary-color);
  box-shadow: var(--shadow-xs);
}

/* 强调色区块：让色块与自定义选择器有充足空间 */
.accent-block {
  flex-direction: column;
  align-items: stretch !important;
  gap: var(--space-3) !important;
}

.accent-controls {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.accent-presets {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.accent-dot {
  width: 26px;
  height: 26px;
  border-radius: var(--radius-pill);
  border: 2px solid transparent;
  cursor: pointer;
  padding: 0;
  transition: transform var(--transition-fast);
}

.accent-dot:hover {
  transform: scale(1.15);
}

.accent-dot.active {
  border-color: var(--text-primary);
  box-shadow: 0 0 0 2px var(--bg-elevated), var(--shadow-sm);
}

/* 自定义颜色选择器固定宽度，避免撑满整行 */
.accent-custom {
  width: 150px;
  margin-left: auto;
}

@media (max-width: 600px) {
  .accent-custom {
    width: 100%;
    margin-left: 0;
  }
}
</style>
