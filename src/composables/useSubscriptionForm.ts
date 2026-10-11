import { computed, ref, watch, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import type { FormInst, FormRules } from 'naive-ui'
import { useAppStore, useSubStore } from '@/stores'
import {
  subscriptionService,
  type SubscriptionPersistResult,
} from '@/services/subscription-service'
import { generateConfigFileName, isJsonContent } from '@/views/sub/subscription-utils'
import { DEFAULT_AUTO_UPDATE_MINUTES, type FrontendSubscription } from '@/stores/subscription/types'

export type Subscription = FrontendSubscription

/** 表单额外持有的字段：`uriContent` 用于区分 JSON 手动配置与节点 URI 列表 */
export interface SubscriptionForm extends Subscription {
  uriContent?: string
}

/** 订阅编辑的三种来源，各自对应不同的必填项与保存路径 */
export type SubscriptionTab = 'url' | 'manual' | 'uri'

const emptyForm = (): SubscriptionForm => ({
  name: '',
  url: '',
  isLoading: false,
  isManual: false,
  manualContent: '',
  uriContent: '',
  useOriginalConfig: false,
  autoUpdateIntervalMinutes: DEFAULT_AUTO_UPDATE_MINUTES,
})

/**
 * 订阅的新增 / 编辑表单
 *
 * 这里把「三种来源（URL 下载 / 手动 JSON / 手动 URI 列表）」的分支集中处理：
 * 它们共享同一份表单与同一套校验，差别只在必填项、保存方式与
 * `useOriginalConfig` 的可用性。以前这些分支散在模板校验规则、提交逻辑、
 * tab 切换 watch 里各写一遍，改一个来源要同步改三处。
 *
 * @param formRef 模板里的 `n-form` 引用，用于触发校验
 */
export function useSubscriptionForm(formRef: Ref<FormInst | null>) {
  const { t } = useI18n()
  const message = useMessage()
  const subStore = useSubStore()
  const appStore = useAppStore()

  const showAddModal = ref(false)
  /** 正在编辑的条目下标；`null` 表示新增 */
  const editIndex = ref<number | null>(null)
  const isLoading = ref(false)
  const activeTab = ref<SubscriptionTab>('url')
  /** 用户是否手动改过 `useOriginalConfig`：改过之后就不再随 tab 自动切换 */
  const useOriginalTouched = ref(false)
  const formValue = ref<SubscriptionForm>(emptyForm())

  const autoUpdateDisabled = computed(() => activeTab.value !== 'url')

  const markUseOriginalTouched = () => {
    useOriginalTouched.value = true
  }

  /** 校验规则：按当前 tab 决定哪一项必填 */
  const rules = computed<FormRules>(() => ({
    name: [{ required: true, message: t('sub.nameRequired'), trigger: 'blur' }],
    url: [
      {
        required: true,
        message: t('sub.urlRequired'),
        trigger: 'blur',
        validator: (_rule, value) => (activeTab.value === 'url' ? !!value : true),
      },
    ],
    manualContent: [
      {
        required: true,
        message: t('sub.contentRequired'),
        trigger: 'blur',
        validator: (_rule, value) => (activeTab.value === 'manual' ? !!value : true),
      },
    ],
    uriContent: [
      {
        required: true,
        message: t('sub.uriContentRequired'),
        trigger: 'blur',
        validator: (_rule, value) => (activeTab.value === 'uri' ? !!value : true),
      },
    ],
  }))

  const resetForm = () => {
    formValue.value = emptyForm()
    useOriginalTouched.value = false
    editIndex.value = null
    activeTab.value = 'url'
  }

  /** 切换来源时的表单联动（不覆盖用户已手动改过的选项） */
  watch(activeTab, (tab) => {
    if (tab === 'uri') {
      formValue.value.useOriginalConfig = false
    }
    if (tab !== 'url') {
      formValue.value.autoUpdateIntervalMinutes = 0
    }
    if (editIndex.value !== null || useOriginalTouched.value) return
    formValue.value.useOriginalConfig = tab === 'manual'
  })

  /** 进入编辑：把 `manualContent` 按 JSON / URI 列表分派到对应字段与 tab */
  const handleEdit = (index: number, item: Subscription) => {
    const manualContent = item.manualContent ?? ''
    const isJson = isJsonContent(manualContent)
    const isNodeList = !isJson && manualContent.trim().length > 0

    editIndex.value = index
    formValue.value = {
      ...item,
      manualContent: item.isManual && isJson ? manualContent : '',
      uriContent: item.isManual && !isJson && isNodeList ? manualContent : '',
    }
    activeTab.value = item.isManual ? (isNodeList ? 'uri' : 'manual') : 'url'
    showAddModal.value = true
  }

  /**
   * 把后端返回的 userinfo（流量 / 到期时间）写入本地条目。
   *
   * 注意：手动订阅没有后端统计，因此是**清空**而不是「合并」——
   * 若在此处直接 return，用户从 URL 订阅切到手动订阅后会继续看到旧的流量数字。
   */
  const applySubscriptionUserinfo = (
    target: Subscription,
    result: SubscriptionPersistResult | null,
    isManual: boolean,
  ) => {
    if (isManual || !result) {
      target.subscriptionUpload = undefined
      target.subscriptionDownload = undefined
      target.subscriptionTotal = undefined
      target.subscriptionExpire = undefined
      return
    }

    target.subscriptionUpload = result.subscriptionUpload
    target.subscriptionDownload = result.subscriptionDownload
    target.subscriptionTotal = result.subscriptionTotal
    target.subscriptionExpire = result.subscriptionExpire
  }

  /** 校验通过后提交新增或编辑 */
  const submitForm = () => {
    formRef.value?.validate(async (errors) => {
      if (errors) return
      try {
        isLoading.value = true
        const tab = activeTab.value
        const isManual = tab !== 'url'
        const urlInput = formValue.value.url?.trim() ?? ''
        const manualInput = formValue.value.manualContent?.trim() ?? ''
        const uriInput = formValue.value.uriContent?.trim() ?? ''
        const resolvedManualContent = tab === 'manual' ? manualInput : tab === 'uri' ? uriInput : ''
        // URI 列表本身是协议行，不是一份 sing-box 配置，无法「沿用原始配置」
        const useOriginalConfig = tab === 'uri' ? false : formValue.value.useOriginalConfig
        const persistOptions = {
          fileName: generateConfigFileName(formValue.value.name || 'sub'),
        }
        let savedResult: SubscriptionPersistResult | null = null

        if (editIndex.value === null) {
          if (isManual && resolvedManualContent) {
            if (useOriginalConfig && !isJsonContent(resolvedManualContent)) {
              message.warning(t('sub.originalConfigJsonOnly'))
              return
            }
            savedResult = await subscriptionService.addManualSubscription(
              resolvedManualContent,
              useOriginalConfig,
              { ...persistOptions, applyRuntime: false },
            )
          } else if (!isManual) {
            savedResult = await subscriptionService.downloadSubscription(
              urlInput,
              useOriginalConfig,
              { ...persistOptions, applyRuntime: false },
            )
          }
          const savedPath = savedResult?.configPath ?? null

          const { uriContent, ...base } = formValue.value
          void uriContent
          const newItem: Subscription = {
            ...base,
            url: isManual ? '' : urlInput,
            lastUpdate: Date.now(),
            isManual,
            manualContent: isManual ? resolvedManualContent : undefined,
            useOriginalConfig,
            autoUpdateIntervalMinutes: isManual ? 0 : base.autoUpdateIntervalMinutes,
            configPath: savedPath || undefined,
            backupPath: savedPath ? `${savedPath}.bak` : undefined,
          }
          applySubscriptionUserinfo(newItem, savedResult, isManual)

          subStore.list.push(newItem)
          await subStore.saveToBackend()
          await subStore.setActiveIndex(subStore.list.length - 1)

          if (savedPath) {
            await subscriptionService.setActiveConfig(savedPath, { useOriginalConfig })
            await appStore.setActiveConfigPath(savedPath)
          }

          message.success(t('sub.addAndUseSuccess'))
        } else {
          if (isManual && resolvedManualContent && useOriginalConfig) {
            if (!isJsonContent(resolvedManualContent)) {
              message.warning(t('sub.originalConfigJsonOnly'))
              return
            }
          }
          const { uriContent, ...base } = formValue.value
          void uriContent
          subStore.list[editIndex.value] = {
            ...subStore.list[editIndex.value],
            ...base,
            url: isManual ? '' : urlInput,
            isManual,
            manualContent: isManual ? resolvedManualContent : undefined,
            useOriginalConfig,
            autoUpdateIntervalMinutes: isManual ? 0 : base.autoUpdateIntervalMinutes,
          }
          if (isManual) {
            applySubscriptionUserinfo(subStore.list[editIndex.value], null, true)
          }
          await subStore.saveToBackend()
          message.success(t('sub.updateSuccess'))
        }

        showAddModal.value = false
        resetForm()
      } catch (error) {
        message.error(t('sub.operationFailed') + error)
      } finally {
        isLoading.value = false
      }
    })
  }

  const handleCancel = () => {
    showAddModal.value = false
    resetForm()
  }

  return {
    showAddModal,
    editIndex,
    isLoading,
    activeTab,
    useOriginalTouched,
    formValue,
    autoUpdateDisabled,
    rules,
    markUseOriginalTouched,
    resetForm,
    handleEdit,
    handleCancel,
    submitForm,
    applySubscriptionUserinfo,
  }
}
