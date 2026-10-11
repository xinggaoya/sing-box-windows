import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { ref } from 'vue'
import { useSubscriptionForm, type SubscriptionForm } from '@/composables/useSubscriptionForm'
import { useSubStore } from '@/stores'
import { DEFAULT_AUTO_UPDATE_MINUTES, type FrontendSubscription } from '@/stores/subscription/types'

/**
 * 订阅表单的三种来源（URL / 手动 JSON / 手动 URI）分支测试。
 *
 * 这段逻辑原先散在模板校验规则、`handleConfirm` 与 tab 切换 watch 三处，
 * 改一个来源要同步改三处。抽取后在这里把三条路径的行为钉死 ——
 * 尤其是「手动订阅要清空后端流量统计」这类容易漏掉的副作用。
 */

const svc = vi.hoisted(() => ({
  addManualSubscription: vi.fn(),
  downloadSubscription: vi.fn(),
  setActiveConfig: vi.fn(),
}))

vi.mock('@/services/subscription-service', () => ({
  subscriptionService: svc,
  // 类型导出在运行时不存在，测试只需要值
}))

vi.mock('vue-i18n', () => ({
  useI18n: () => ({
    t: (key: string, params?: Record<string, unknown>) =>
      params ? `${key}:${JSON.stringify(params)}` : key,
  }),
}))

vi.mock('naive-ui', () => ({
  useMessage: () => ({
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
  }),
}))

vi.mock('@/stores', () => {
  const list: FrontendSubscription[] = []
  return {
    useSubStore: () => ({
      list,
      activeIndex: null,
      saveToBackend: vi.fn(),
      setActiveIndex: vi.fn(),
    }),
    useAppStore: () => ({ setActiveConfigPath: vi.fn() }),
  }
})

/** 可控的 `n-form` 引用：记录 validate 是否被调用并可模拟校验失败 */
function makeFormRef(options: { hasErrors?: boolean } = {}) {
  return ref({
    validate: (callback: (errors: unknown[]) => void) => {
      callback(options.hasErrors ? [{ message: 'invalid' }] : [])
    },
  } as never)
}

function baseItem(overrides: Partial<FrontendSubscription> = {}): FrontendSubscription {
  return {
    name: 'sub',
    url: '',
    isLoading: false,
    isManual: false,
    ...overrides,
  } as FrontendSubscription
}

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  svc.addManualSubscription.mockResolvedValue({ configPath: '/tmp/a.json' })
  svc.downloadSubscription.mockResolvedValue({ configPath: '/tmp/b.json' })
  svc.setActiveConfig.mockResolvedValue(undefined)
  const store = useSubStore()
  store.list.length = 0
})

describe('表单初始状态', () => {
  it('初始为 URL 来源且字段为空', () => {
    const { activeTab, formValue, editIndex, useOriginalTouched } =
      useSubscriptionForm(makeFormRef())

    expect(activeTab.value).toBe('url')
    expect(editIndex.value).toBeNull()
    expect(useOriginalTouched.value).toBe(false)
    expect(formValue.value.name).toBe('')
    expect(formValue.value.autoUpdateIntervalMinutes).toBe(DEFAULT_AUTO_UPDATE_MINUTES)
  })

  it('只有 URL 来源可开启自动更新', () => {
    const { activeTab, autoUpdateDisabled } = useSubscriptionForm(makeFormRef())
    expect(autoUpdateDisabled.value).toBe(false)
    activeTab.value = 'manual'
    expect(autoUpdateDisabled.value).toBe(true)
  })
})

describe('tab 切换的表单联动', () => {
  it('切到 URI 来源会关掉「沿用原始配置」（URI 不是 sing-box 配置）', async () => {
    const { activeTab, formValue } = useSubscriptionForm(makeFormRef())
    formValue.value.useOriginalConfig = true

    activeTab.value = 'uri'
    await new Promise((r) => setTimeout(r, 0))

    expect(formValue.value.useOriginalConfig).toBe(false)
  })

  it('切离 URL 来源会把自动更新间隔清零', async () => {
    const { activeTab, formValue } = useSubscriptionForm(makeFormRef())
    expect(formValue.value.autoUpdateIntervalMinutes).toBe(DEFAULT_AUTO_UPDATE_MINUTES)

    activeTab.value = 'manual'
    await new Promise((r) => setTimeout(r, 0))

    expect(formValue.value.autoUpdateIntervalMinutes).toBe(0)
  })

  it('切到手动来源时默认勾选「沿用原始配置」', async () => {
    const { activeTab, formValue } = useSubscriptionForm(makeFormRef())

    activeTab.value = 'manual'
    await new Promise((r) => setTimeout(r, 0))

    expect(formValue.value.useOriginalConfig).toBe(true)
  })

  it('用户手动改过之后不再随 tab 自动切换', async () => {
    const { activeTab, formValue, markUseOriginalTouched } = useSubscriptionForm(makeFormRef())

    activeTab.value = 'uri'
    await new Promise((r) => setTimeout(r, 0))

    formValue.value.useOriginalConfig = true
    markUseOriginalTouched()

    activeTab.value = 'url'
    await new Promise((r) => setTimeout(r, 0))

    expect(formValue.value.useOriginalConfig).toBe(true)
  })

  it('编辑已有条目时不覆盖用户的「沿用原始配置」选择', async () => {
    const form = makeFormRef()
    const { activeTab, formValue, handleEdit } = useSubscriptionForm(form)

    handleEdit(0, baseItem({ name: 'x', url: 'https://e.com/sub' }))
    formValue.value.useOriginalConfig = true

    activeTab.value = 'manual'
    await new Promise((r) => setTimeout(r, 0))

    expect(formValue.value.useOriginalConfig).toBe(true)
  })
})

describe('handleEdit 按内容分派来源', () => {
  it('JSON 手动内容归入 manual tab', () => {
    const { handleEdit, activeTab, formValue } = useSubscriptionForm(makeFormRef())

    handleEdit(0, baseItem({ isManual: true, manualContent: '{"outbounds":[]}' }))

    expect(activeTab.value).toBe('manual')
    expect(formValue.value.manualContent).toBe('{"outbounds":[]}')
    expect(formValue.value.uriContent).toBe('')
  })

  it('非 JSON 内容归入 uri tab', () => {
    const { handleEdit, activeTab, formValue } = useSubscriptionForm(makeFormRef())

    handleEdit(0, baseItem({ isManual: true, manualContent: 'vmess://abc' }))

    expect(activeTab.value).toBe('uri')
    expect(formValue.value.uriContent).toBe('vmess://abc')
  })

  it('URL 订阅归入 url tab 且不把内容塞进手动字段', () => {
    const { handleEdit, activeTab, formValue } = useSubscriptionForm(makeFormRef())

    handleEdit(0, baseItem({ url: 'https://e.com/sub' }))

    expect(activeTab.value).toBe('url')
    expect(formValue.value.manualContent).toBe('')
    expect(formValue.value.uriContent).toBe('')
  })

  it('进入编辑会打开弹窗并记录下标', () => {
    const { handleEdit, showAddModal, editIndex } = useSubscriptionForm(makeFormRef())

    handleEdit(3, baseItem({ url: 'https://e.com/sub' }))

    expect(showAddModal.value).toBe(true)
    expect(editIndex.value).toBe(3)
  })
})

describe('resetForm / handleCancel', () => {
  it('resetForm 清空全部状态', async () => {
    const form = makeFormRef()
    const { formValue, activeTab, editIndex, useOriginalTouched, handleEdit, resetForm } =
      useSubscriptionForm(form)

    handleEdit(2, baseItem({ url: 'https://e.com/sub' }))
    formValue.value.useOriginalConfig = true

    resetForm()

    expect(activeTab.value).toBe('url')
    expect(editIndex.value).toBeNull()
    expect(useOriginalTouched.value).toBe(false)
    expect(formValue.value.url).toBe('')
  })

  it('handleCancel 关闭弹窗并重置', () => {
    const form = makeFormRef()
    const { showAddModal, handleEdit, handleCancel } = useSubscriptionForm(form)

    handleEdit(0, baseItem({ url: 'https://e.com/sub' }))
    expect(showAddModal.value).toBe(true)

    handleCancel()
    expect(showAddModal.value).toBe(false)
  })
})

describe('校验规则按来源切换必填项', () => {
  it('只有当前来源对应的字段必填', () => {
    const { rules, activeTab } = useSubscriptionForm(makeFormRef())

    const pickValidator = (field: 'url' | 'manualContent') => {
      const entry = rules.value[field]
      const first = Array.isArray(entry) ? entry[0] : entry
      return first?.validator as (_rule: unknown, value: unknown) => boolean
    }
    const urlValidator = pickValidator('url')
    const manualValidator = pickValidator('manualContent')

    // URL 来源：URL 必填、手动内容不参与校验
    expect(urlValidator({}, 'https://e.com/sub')).toBe(true)
    expect(urlValidator({}, undefined)).toBe(false)
    expect(manualValidator({}, undefined)).toBe(true)

    // 切到手动来源：URL 不再必填，手动内容必填
    activeTab.value = 'manual'
    expect(urlValidator({}, undefined)).toBe(true)
    expect(manualValidator({}, undefined)).toBe(false)
    expect(manualValidator({}, '{"a":1}')).toBe(true)
  })

  it('校验不通过时不提交', () => {
    const store = useSubStore()
    const { submitForm } = useSubscriptionForm(makeFormRef({ hasErrors: true }))

    submitForm()
    expect(store.list).toHaveLength(0)
    expect(svc.downloadSubscription).not.toHaveBeenCalled()
  })
})

describe('applySubscriptionUserinfo', () => {
  it('写入后端返回的流量与到期时间', () => {
    const { applySubscriptionUserinfo } = useSubscriptionForm(makeFormRef())
    const target = baseItem({ url: 'https://e.com/sub' })

    applySubscriptionUserinfo(
      target,
      {
        subscriptionUpload: 1,
        subscriptionDownload: 2,
        subscriptionTotal: 3,
        subscriptionExpire: 4,
      } as never,
      false,
    )

    expect(target.subscriptionUpload).toBe(1)
    expect(target.subscriptionTotal).toBe(3)
    expect(target.subscriptionExpire).toBe(4)
  })

  it('手动订阅要**清空**而非保留旧统计（回归：切到手动后仍显示旧流量）', () => {
    const { applySubscriptionUserinfo } = useSubscriptionForm(makeFormRef())
    const target = baseItem({
      subscriptionUpload: 100,
      subscriptionDownload: 200,
      subscriptionTotal: 300,
      subscriptionExpire: 400,
    })

    applySubscriptionUserinfo(
      target,
      {
        subscriptionUpload: 1,
        subscriptionDownload: 2,
        subscriptionTotal: 3,
        subscriptionExpire: 4,
      } as never,
      true,
    )

    expect(target.subscriptionUpload).toBeUndefined()
    expect(target.subscriptionDownload).toBeUndefined()
    expect(target.subscriptionTotal).toBeUndefined()
    expect(target.subscriptionExpire).toBeUndefined()
  })

  it('无后端结果时也清空', () => {
    const { applySubscriptionUserinfo } = useSubscriptionForm(makeFormRef())
    const target = baseItem({ subscriptionUpload: 9 })

    applySubscriptionUserinfo(target, null, false)
    expect(target.subscriptionUpload).toBeUndefined()
  })
})

// 让 SubscriptionForm 类型在测试里被引用（避免未使用告警）
void ({} as SubscriptionForm)
