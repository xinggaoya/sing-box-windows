import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import { supportedLocales, DEFAULT_LOCALE, type LocaleCode } from '@/locales'
import { DatabaseService } from '@/services/database-service'
import type { LocaleConfig } from '@/types/database'

// 语言类型
export type Locale = LocaleCode | 'auto'

const isLocaleCode = (value: unknown): value is LocaleCode =>
  typeof value === 'string' && supportedLocales.some((loc) => loc.code === value)

const isLocaleValue = (value: unknown): value is Locale =>
  value === 'auto' || isLocaleCode(value)

export const useLocaleStore = defineStore(
  'locale',
  () => {
    // 添加语言设置
    const locale = ref<Locale>('auto')

    // 从数据库加载数据
    const loadFromBackend = async () => {
      try {
        console.log('🌐 从数据库加载语言配置...')
        const localeConfig = await DatabaseService.getLocaleConfig()
        const nextLocale = isLocaleValue(localeConfig.locale) ? localeConfig.locale : 'auto'

        // 更新响应式状态，自动回退到受支持值
        locale.value = nextLocale

        console.log('🌐 语言配置加载完成：', { locale: locale.value })
      } catch (error) {
        console.error('从数据库加载语言配置失败:', error)
        // 加载失败时使用默认值
        locale.value = 'auto'
      }
    }

    // 保存配置到数据库
    const saveToBackend = async () => {
      try {
        const config: LocaleConfig = { locale: locale.value }
        await DatabaseService.saveLocaleConfig(config)
        console.log('✅ 语言配置已保存到数据库')
      } catch (error) {
        console.error('保存语言配置到数据库失败:', error)
      }
    }

    // 计算实际使用的语言
    const currentLocale = computed<LocaleCode>(() => {
      if (locale.value === 'auto') {
        // 获取浏览器语言
        const browserLang = navigator.language
        return isLocaleCode(browserLang) ? browserLang : DEFAULT_LOCALE
      }
      return isLocaleCode(locale.value) ? locale.value : DEFAULT_LOCALE
    })

    // 语言切换
    const setLocale = async (newLocale: Locale) => {
      if (!isLocaleValue(newLocale)) {
        console.warn('试图设置不受支持的语言:', newLocale)
        return
      }
      locale.value = newLocale
      // 保存会在 watch 中自动处理
      console.log('语言已切换到:', newLocale)
    }

    // 获取当前语言名称
    const getCurrentLocaleName = computed(() => {
      const currentCode = currentLocale.value
      const localeEntry = supportedLocales.find((loc) => loc.code === currentCode)
      return localeEntry ? localeEntry.name : '简体中文'
    })

    // 标记是否正在初始化
    let isInitializing = false

    // 监听语言变化并自动保存到数据库
    watch(
      locale,
      async () => {
        // 初始化期间不保存
        if (isInitializing) return
        await saveToBackend()
      },
      { immediate: false }
    )

    // 初始化方法
    const initializeStore = async () => {
      isInitializing = true
      await loadFromBackend()
      // 等待一下确保数据加载完成
      await new Promise((resolve) => setTimeout(resolve, 100))
      isInitializing = false
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
