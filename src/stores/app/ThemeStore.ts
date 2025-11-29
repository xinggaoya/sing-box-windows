import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { darkTheme, type GlobalThemeOverrides, useOsTheme } from 'naive-ui'
import { DatabaseService } from '@/services/database-service'
import type { ThemeConfig } from '@/types/database'
import baseThemeOverrides from '@/assets/naive-ui-theme-overrides.json'

export type ThemeMode = 'light' | 'dark' | 'system'

const DEFAULT_ACCENT = '#6366f1'

const normalizeHexColor = (color?: string) => {
  if (!color) return DEFAULT_ACCENT
  const value = color.startsWith('#') ? color : `#${color}`
  return /^#([0-9a-fA-F]{6})$/.test(value) ? value.toLowerCase() : DEFAULT_ACCENT
}

const toRgb = (hex: string) => {
  const normalized = normalizeHexColor(hex).replace('#', '')
  const intValue = parseInt(normalized, 16)
  return {
    r: (intValue >> 16) & 255,
    g: (intValue >> 8) & 255,
    b: intValue & 255,
  }
}

const clampChannel = (value: number) => Math.min(255, Math.max(0, Math.round(value)))

const rgbToHex = (r: number, g: number, b: number) => {
  const value = (clampChannel(r) << 16) | (clampChannel(g) << 8) | clampChannel(b)
  return `#${value.toString(16).padStart(6, '0')}`
}

const adjustColor = (hex: string, delta: number) => {
  const { r, g, b } = toRgb(hex)
  const offset = (delta / 100) * 255
  return rgbToHex(r + offset, g + offset, b + offset)
}

const hexToRgba = (hex: string, alpha: number) => {
  const { r, g, b } = toRgb(hex)
  return `rgba(${r}, ${g}, ${b}, ${Math.min(1, Math.max(0, alpha))})`
}

const cloneOverrides = (): GlobalThemeOverrides =>
  JSON.parse(JSON.stringify(baseThemeOverrides)) as GlobalThemeOverrides

export const useThemeStore = defineStore(
  'theme',
  () => {
    let isInitializing = true
    const osTheme = useOsTheme()
    const mode = ref<ThemeMode>('system')
    const accentColor = ref<string>(DEFAULT_ACCENT)
    const compactMode = ref(false)

    const isDark = computed(() => {
      if (mode.value === 'dark') return true
      if (mode.value === 'light') return false
      return osTheme.value === 'dark'
    })

    const theme = computed(() => (isDark.value ? darkTheme : null))

    const buildOverrides = computed<GlobalThemeOverrides>(() => {
      const primary = normalizeHexColor(accentColor.value)
      const primaryHover = adjustColor(primary, 10)
      const primaryPressed = adjustColor(primary, -8)
      const overrides = cloneOverrides()

      overrides.common = {
        ...overrides.common,
        primaryColor: primary,
        primaryColorHover: primaryHover,
        primaryColorPressed: primaryPressed,
        primaryColorSuppl: primary,
      }

      overrides.Button = {
        ...overrides.Button,
        border: `1px solid ${hexToRgba(primary, 0.2)}`,
        borderHover: `1px solid ${hexToRgba(primary, 0.35)}`,
        borderFocus: `1px solid ${primary}`,
        borderPressed: `1px solid ${primary}`,
        rippleColor: hexToRgba(primary, 0.2),
      }

      overrides.Switch = {
        ...overrides.Switch,
        railColorActive: primary,
      }

      if (compactMode.value) {
        overrides.common = {
          ...overrides.common,
          heightSmall: '32px',
          heightMedium: '36px',
          heightLarge: '42px',
        }
        overrides.Input = {
          ...overrides.Input,
          heightSmall: '32px',
          heightMedium: '36px',
          paddingMedium: '0 12px',
        }
        overrides.Select = {
          ...overrides.Select,
          heightSmall: '32px',
          heightMedium: '36px',
          paddingMedium: '0 12px',
        }
        overrides.Button = {
          ...overrides.Button,
          paddingSmall: '0 12px',
          paddingMedium: '0 14px',
          paddingLarge: '0 18px',
        }
        overrides.Card = {
          ...overrides.Card,
          paddingMedium: '20px',
          paddingLarge: '24px',
        }
      }

      return overrides
    })

    const loadFromBackend = async () => {
      try {
        const themeConfig = await DatabaseService.getThemeConfig()
        const persistedMode = themeConfig.mode as ThemeMode
        if (persistedMode && ['light', 'dark', 'system'].includes(persistedMode)) {
          mode.value = persistedMode
        } else if (typeof themeConfig.is_dark === 'boolean') {
          mode.value = themeConfig.is_dark ? 'dark' : 'light'
        } else {
          mode.value = 'system'
        }
        accentColor.value = normalizeHexColor(themeConfig.accent_color)
        compactMode.value = themeConfig.compact_mode ?? false
      } catch (error) {
        console.error('从数据库加载主题配置失败:', error)
        mode.value = 'system'
        accentColor.value = DEFAULT_ACCENT
        compactMode.value = false
      }
    }

    const applyThemeClass = (dark: boolean) => {
      if (typeof document !== 'undefined') {
        document.documentElement.classList.toggle('dark', dark)
      }
    }

    const applyCompactClass = (compact: boolean) => {
      if (typeof document === 'undefined') return
      // 通过根节点类名切换全局紧凑布局样式
      document.documentElement.classList.toggle('compact-mode', compact)
    }

    const applyAccentVariables = () => {
      if (typeof document === 'undefined') return
      const primary = normalizeHexColor(accentColor.value)
      const hover = adjustColor(primary, 10)
      const active = adjustColor(primary, -8)

      document.documentElement.style.setProperty('--primary-color', primary)
      document.documentElement.style.setProperty('--primary-hover', hover)
      document.documentElement.style.setProperty('--primary-active', active)
      document.documentElement.style.setProperty('--chip-bg', hexToRgba(primary, 0.12))
      document.documentElement.style.setProperty('--chip-text', primary)
      document.documentElement.style.setProperty('--border-hover', hexToRgba(primary, 0.35))
    }

    const syncUiTheme = (dark: boolean, compact: boolean) => {
      applyThemeClass(dark)
      applyCompactClass(compact)
      applyAccentVariables()
    }

    let saveTimer: number | null = null
    const schedulePersist = () => {
      if (isInitializing) return
      if (saveTimer) {
        clearTimeout(saveTimer)
      }
      saveTimer = window.setTimeout(() => {
        saveTimer = null
        saveToBackend()
      }, 180)
    }

    const saveToBackend = async () => {
      try {
        const config: ThemeConfig = {
          is_dark: isDark.value,
          mode: mode.value,
          accent_color: normalizeHexColor(accentColor.value),
          compact_mode: compactMode.value,
        }
        await DatabaseService.saveThemeConfig(config)
      } catch (error) {
        console.error('保存主题配置到数据库失败:', error)
      }
    }

    watch(
      [isDark, mode, accentColor, compactMode],
      ([dark, , , compact]) => {
        syncUiTheme(dark, compact)
        schedulePersist()
      },
      { immediate: false },
    )

    const toggleTheme = async () => {
      mode.value = isDark.value ? 'light' : 'dark'
    }

    const setDarkMode = async (dark: boolean) => {
      mode.value = dark ? 'dark' : 'light'
    }

    const setThemeMode = async (value: ThemeMode) => {
      mode.value = value
    }

    const setAccentColor = async (color: string) => {
      accentColor.value = normalizeHexColor(color)
    }

    const setCompactMode = async (value: boolean) => {
      compactMode.value = value
    }

    const initializeStore = async () => {
      await loadFromBackend()
      syncUiTheme(isDark.value, compactMode.value)
      isInitializing = false
    }

    return {
      isDark,
      mode,
      accentColor,
      compactMode,
      theme,
      themeOverrides: buildOverrides,
      toggleTheme,
      setDarkMode,
      setThemeMode,
      setAccentColor,
      setCompactMode,
      naiveTheme: theme,
      initializeStore,
      loadFromBackend,
      saveToBackend,
    }
  },
)
