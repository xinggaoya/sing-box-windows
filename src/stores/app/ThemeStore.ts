import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'

export const useThemeStore = defineStore(
  'theme',
  () => {
    // 主题相关状态
    const osTheme = useOsTheme()
    const isDark = ref(osTheme.value === 'dark')
    const theme = computed(() => (isDark.value ? darkTheme : null))

    // 应用暗黑类到DOM
    const applyThemeClass = (dark: boolean) => {
      if (typeof document !== 'undefined') {
        if (dark) {
          document.documentElement.classList.add('dark')
        } else {
          document.documentElement.classList.remove('dark')
        }
      }
    }

    // 监听主题变化并应用到DOM
    watch(
      isDark,
      (newValue) => {
        applyThemeClass(newValue)
      },
      { immediate: true },
    )

    // 主题切换
    const toggleTheme = () => {
      isDark.value = !isDark.value
    }

    // 设置深色模式
    const setDarkMode = (dark: boolean) => {
      isDark.value = dark
    }

    return {
      isDark,
      theme,
      toggleTheme,
      setDarkMode,
    }
  },
  {
    persist: true,
  },
)
