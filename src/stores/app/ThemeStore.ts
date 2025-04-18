import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'

export const useThemeStore = defineStore(
  'theme',
  () => {
    // 主题相关状态
    const osTheme = useOsTheme()
    const isDark = ref(osTheme.value === 'dark')
    const theme = computed(() => (isDark.value ? darkTheme : null))

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
  }
)
