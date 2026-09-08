import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import type { ConfigTemplate, TemplateMarketSettings } from '@/types/generated'
import { templateMarketService } from '@/services/template-market-service'

/** 官方内置模板的固定 ID（与后端 OFFICIAL_TEMPLATE_ID 对齐） */
export const OFFICIAL_TEMPLATE_ID = 'official'

/**
 * 模板市场 Store：本地模板列表 + 市场设置（服务地址/当前激活模板）。
 * 与订阅 Store 不同，这里不做深度 watch 自动持久化，
 * 一律通过显式 action 调用后端命令后回读，保证与后端校验结果一致。
 */
export const useTemplateStore = defineStore('template', () => {
  const templates = ref<ConfigTemplate[]>([])
  const settings = ref<TemplateMarketSettings>({
    service_url: '',
    active_template_id: OFFICIAL_TEMPLATE_ID,
  })
  const isInitializing = ref(false)
  let initializePromise: Promise<void> | null = null

  const officialTemplate = computed(() =>
    templates.value.find((t) => t.id === OFFICIAL_TEMPLATE_ID),
  )
  const localTemplates = computed(() =>
    templates.value.filter((t) => t.id !== OFFICIAL_TEMPLATE_ID),
  )
  const activeTemplate = computed(
    () => templates.value.find((t) => t.id === settings.value.active_template_id) ?? null,
  )
  const isActiveOfficial = computed(
    () => settings.value.active_template_id === OFFICIAL_TEMPLATE_ID,
  )

  /** 我发布到市场的模板（含本地保留的市场关联信息） */
  const myPublishedTemplates = computed(() => localTemplates.value.filter((t) => t.remote_id))

  async function loadFromBackend() {
    const [list, marketSettings] = await Promise.all([
      templateMarketService.getConfigTemplates(),
      templateMarketService.getTemplateMarketSettings(),
    ])
    templates.value = list
    settings.value = marketSettings
  }

  /** 单例初始化守卫（同 SubStore 模式） */
  async function initializeStore() {
    if (initializePromise) return initializePromise
    isInitializing.value = true
    initializePromise = (async () => {
      try {
        await loadFromBackend()
      } finally {
        isInitializing.value = false
      }
    })()
    return initializePromise
  }

  async function saveTemplate(template: ConfigTemplate) {
    const saved = await templateMarketService.saveConfigTemplate(template)
    await loadFromBackend()
    return saved
  }

  async function deleteTemplate(templateId: string) {
    await templateMarketService.deleteConfigTemplate(templateId)
    await loadFromBackend()
  }

  async function setActiveTemplate(templateId: string) {
    settings.value = await templateMarketService.setActiveTemplate(templateId)
  }

  async function saveSettings(next: TemplateMarketSettings) {
    settings.value = await templateMarketService.setTemplateMarketSettings(next)
  }

  async function publishTemplate(templateId: string, authorName?: string) {
    await templateMarketService.marketPublishTemplate(templateId, authorName)
    await loadFromBackend()
  }

  async function updateMarketTemplate(templateId: string) {
    await templateMarketService.marketUpdateTemplate(templateId)
    await loadFromBackend()
  }

  async function deleteMarketTemplate(templateId: string) {
    await templateMarketService.marketDeleteTemplate(templateId)
    await loadFromBackend()
  }

  async function downloadMarketTemplate(remoteId: string) {
    const saved = await templateMarketService.marketDownloadTemplate(remoteId)
    await loadFromBackend()
    return saved
  }

  async function refreshMyTemplates() {
    const list = await templateMarketService.marketRefreshMyTemplates()
    await loadFromBackend()
    return list
  }

  return {
    templates,
    settings,
    isInitializing,
    officialTemplate,
    localTemplates,
    activeTemplate,
    isActiveOfficial,
    myPublishedTemplates,
    initializeStore,
    loadFromBackend,
    saveTemplate,
    deleteTemplate,
    setActiveTemplate,
    saveSettings,
    publishTemplate,
    updateMarketTemplate,
    deleteMarketTemplate,
    downloadMarketTemplate,
    refreshMyTemplates,
  }
})
