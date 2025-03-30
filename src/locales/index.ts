import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import enUS from './en-US'
import ruRU from './ru-RU'
import jaJP from './ja-JP'

// 支持的语言列表
export const supportedLocales = [
  { code: 'zh-CN', name: '简体中文' },
  { code: 'en-US', name: 'English' },
  { code: 'ru-RU', name: 'Русский' },
  { code: 'ja-JP', name: '日本語' },
]

// 创建i18n实例
const i18n = createI18n({
  legacy: false, // 使用Vue 3 Composition API
  locale: 'zh-CN', // 默认语言
  fallbackLocale: 'zh-CN', // 备用语言
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
    'ru-RU': ruRU,
    'ja-JP': jaJP,
  },
  globalInjection: true, // 全局注入$t方法
})

export default i18n
