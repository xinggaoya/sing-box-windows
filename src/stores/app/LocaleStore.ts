import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { supportedLocales } from '@/locales'

// 语言类型
export type Locale = 'zh-CN' | 'en-US' | 'ru-RU' | 'ja-JP' | 'auto'

export const useLocaleStore = defineStore(
  'locale',
  () => {
    // 添加语言设置
    const locale = ref<Locale>('auto')

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
    const setLocale = (newLocale: Locale) => {
      locale.value = newLocale
      // 语言变更事件现在通过Pinia响应式系统处理
      console.log('语言已切换到:', newLocale)
    }

    // 获取当前语言名称
    const getCurrentLocaleName = computed(() => {
      const currentCode = currentLocale.value
      const locale = supportedLocales.find((loc) => loc.code === currentCode)
      return locale ? locale.name : '简体中文'
    })

    return {
      locale,
      currentLocale,
      setLocale,
      getCurrentLocaleName
    }
  },
  {
    persist: true,
  }
)
