import { createI18n } from 'vue-i18n'
import zhCN from './zh-CN'
import enUS from './en-US'
import ruRU from './ru-RU'
import jaJP from './ja-JP'

export type LocaleCode = 'zh-CN' | 'en-US' | 'ru-RU' | 'ja-JP'

export interface SupportedLocale {
  code: LocaleCode
  name: string
}

export const DEFAULT_LOCALE: LocaleCode = 'zh-CN'

// 支持的语言列表
export const supportedLocales: SupportedLocale[] = [
  { code: 'zh-CN', name: '简体中文' },
  { code: 'en-US', name: 'English' },
  { code: 'ru-RU', name: 'Русский' },
  { code: 'ja-JP', name: '日本語' },
]

// 创建i18n实例
const i18n = createI18n({
  legacy: false, // 使用Vue 3 Composition API
  locale: DEFAULT_LOCALE, // 默认语言
  fallbackLocale: DEFAULT_LOCALE, // 备用语言
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
    'ru-RU': ruRU,
    'ja-JP': jaJP,
  },
  globalInjection: true, // 全局注入$t方法
})

export default i18n
