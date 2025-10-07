import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { darkTheme } from 'naive-ui'
import { useOsTheme } from 'naive-ui'
import { storageService } from '@/services/backend-storage-service'

export const useThemeStore = defineStore(
  'theme',
  () => {
    // 主题相关状态
    const osTheme = useOsTheme()
    const isDark = ref(osTheme.value === 'dark')
    const theme = computed(() => (isDark.value ? darkTheme : null))

    // 从后端加载数据
    const loadFromBackend = async () => {
      try {
        console.log('🎨 从后端加载主题配置...')
        const themeConfig = await storageService.getThemeConfig()
        
        // 如果后端有保存的主题设置，使用后端的设置
        // 否则使用系统主题
        isDark.value = themeConfig.is_dark
        
        console.log('🎨 主题配置加载完成：', { isDark: isDark.value })
      } catch (error) {
        console.error('从后端加载主题配置失败:', error)
        // 加载失败时使用系统主题
        isDark.value = osTheme.value === 'dark'
      }
    }

    // 保存配置到后端
    const saveToBackend = async () => {
      try {
        await storageService.updateThemeConfig(isDark.value)
        console.log('✅ 主题配置已保存到后端')
      } catch (error) {
        console.error('保存主题配置到后端失败:', error)
      }
    }

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
      async (newValue) => {
        applyThemeClass(newValue)
        // 自动保存到后端
        await saveToBackend()
      },
      { immediate: true },
    )

    // 主题切换
    const toggleTheme = async () => {
      isDark.value = !isDark.value
      // 保存已在 watch 中处理
    }

    // 设置深色模式
    const setDarkMode = async (dark: boolean) => {
      isDark.value = dark
      // 保存已在 watch 中处理
    }

    // 初始化方法
    const initializeStore = async () => {
      await loadFromBackend()
    }

    return {
      isDark,
      theme,
      toggleTheme,
      setDarkMode,
      initializeStore,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)