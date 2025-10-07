import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { supportedLocales } from '@/locales'
import { storageService } from '@/services/backend-storage-service'

// 语言类型
export type Locale = 'zh-CN' | 'en-US' | 'ru-RU' | 'ja-JP' | 'auto'

export const useLocaleStore = defineStore(
  'locale',
  () => {
    // 添加语言设置
    const locale = ref<Locale>('auto')

    // 从后端加载数据
    const loadFromBackend = async () => {
      try {
        console.log('🌐 从后端加载语言配置...')
        const localeConfig = await storageService.getLocaleConfig()
        
        // 更新响应式状态
        locale.value = localeConfig.locale as Locale
        
        console.log('🌐 语言配置加载完成：', { locale: locale.value })
      } catch (error) {
        console.error('从后端加载语言配置失败:', error)
        // 加载失败时使用默认值
        locale.value = 'auto'
      }
    }

    // 保存配置到后端
    const saveToBackend = async () => {
      try {
        await storageService.updateLocaleConfig(locale.value)
        console.log('✅ 语言配置已保存到后端')
      } catch (error) {
        console.error('保存语言配置到后端失败:', error)
      }
    }

    // 计算实际使用的语言
    const currentLocale = computed(() => {
      if (locale.value === 'auto') {
        // 获取浏览器语言
        const browserLang = navigator.language
        // 检查是否支持这个语言
        const isSupported = supportedLocales.some((loc) => loc.code === browserLang)
        return isSupported ? browserLang : 'zh-CN'
      }
      return locale.value
    })

    // 语言切换
    const setLocale = async (newLocale: Locale) => {
      locale.value = newLocale
      
      // 保存到后端
      await saveToBackend()
      
      // 语言变更事件现在通过Pinia响应式系统处理
      console.log('语言已切换到:', newLocale)
    }

    // 获取当前语言名称
    const getCurrentLocaleName = computed(() => {
      const currentCode = currentLocale.value
      const locale = supportedLocales.find((loc) => loc.code === currentCode)
      return locale ? locale.name : '简体中文'
    })

    // 初始化方法
    const initializeStore = async () => {
      await loadFromBackend()
    }

    return {
      locale,
      currentLocale,
      setLocale,
      getCurrentLocaleName,
      initializeStore,
      loadFromBackend,
      saveToBackend,
    }
  },
  // 移除 persist 配置，现在使用后端存储
)