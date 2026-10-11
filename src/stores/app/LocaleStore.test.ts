import { describe, it, expect, beforeEach, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { watch, nextTick } from 'vue'
import { createI18n } from 'vue-i18n'
import { useLocaleStore } from '@/stores/app/LocaleStore'

vi.mock('@/services/database-service', () => ({
  DatabaseService: {
    getLocaleConfig: vi.fn().mockResolvedValue({ locale: 'auto' }),
    saveLocaleConfig: vi.fn().mockResolvedValue(undefined),
  },
}))

const messages = {
  'zh-CN': { common: { refresh: '刷新' } },
  'en-US': { common: { refresh: 'Refresh' } },
  'ru-RU': { common: { refresh: 'Обновить' } },
  'ja-JP': { common: { refresh: '更新' } },
}

const setup = () => {
  const i18n = createI18n({
    legacy: false,
    locale: 'zh-CN',
    fallbackLocale: 'zh-CN',
    messages,
    globalInjection: true,
  })
  const store = useLocaleStore()

  // 这正是 useAppBootstrap 里缺失的那段同步
  const stop = watch(
    () => store.currentLocale,
    (l) => {
      i18n.global.locale.value = l
    },
    { immediate: true },
  )
  return { i18n, store, stop }
}

describe('LocaleStore → vue-i18n 实例同步', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('store.currentLocale 变化后 i18n.global.locale 跟随切换', async () => {
    const { i18n, store } = setup()

    await store.setLocale('en-US')
    await nextTick()

    expect(store.currentLocale).toBe('en-US')
    expect(i18n.global.locale.value).toBe('en-US')
    expect(i18n.global.t('common.refresh')).toBe('Refresh')
  })

  it('缺少同步时 global.t 停留在初始语言（复现用户报告的「改了语言没反应」）', async () => {
    // 不建立 watch，模拟当前 bootstrap 的缺失状态
    const store = useLocaleStore()
    const i18n = createI18n({
      legacy: false,
      locale: 'zh-CN',
      fallbackLocale: 'zh-CN',
      messages,
      globalInjection: true,
    })

    await store.setLocale('en-US')
    await nextTick()

    expect(store.currentLocale).toBe('en-US')
    // 实例仍停在 zh-CN —— store/service 里的 i18n.global.t() 会一直按初始语言翻译
    expect(i18n.global.locale.value).toBe('zh-CN')
    expect(i18n.global.t('common.refresh')).toBe('刷新')
  })

  it("'auto' 解析为受支持语言后再同步（不受支持的语言回退 zh-CN）", async () => {
    const { i18n, store } = setup()

    await store.setLocale('auto')
    await nextTick()

    // jsdom 的 navigator.language 是 'en-US'
    expect(store.currentLocale).toBe('en-US')
    expect(i18n.global.locale.value).toBe('en-US')
  })
})
