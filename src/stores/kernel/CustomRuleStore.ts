import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  customRuleService,
  type CustomRule,
  type CustomRuleInput,
} from '@/services/custom-rule-service'

/**
 * 自定义规则 store
 *
 * 数据源：后端 `list_custom_rules`（持久化在 SQLite 的 generic_config 表）。
 * 变更走「写配置文件 + 重启内核」路径，见 `services/custom-rule-service.ts` 的说明。
 */
export const useCustomRuleStore = defineStore('custom-rule', () => {
  const rules = ref<CustomRule[]>([])
  const loading = ref(false)
  /** 正在执行写操作的规则 id（用于行内按钮 loading 态） */
  const pendingId = ref<string | null>(null)

  /** 已启用的规则 */
  const enabledRules = computed(() => rules.value.filter((r) => r.enabled))

  /** 已禁用的规则 */
  const disabledRules = computed(() => rules.value.filter((r) => !r.enabled))

  const fetchRules = async () => {
    if (loading.value) return
    loading.value = true
    try {
      const list = await customRuleService.list()
      rules.value = Array.isArray(list) ? list : []
    } finally {
      loading.value = false
    }
  }

  const addRule = async (input: CustomRuleInput) => {
    const created = await customRuleService.add(input)
    await fetchRules()
    return created
  }

  const updateRule = async (id: string, input: CustomRuleInput) => {
    pendingId.value = id
    try {
      await customRuleService.update(id, input)
      await fetchRules()
    } finally {
      pendingId.value = null
    }
  }

  const removeRule = async (id: string) => {
    pendingId.value = id
    try {
      await customRuleService.remove(id)
      await fetchRules()
    } finally {
      pendingId.value = null
    }
  }

  const toggleRule = async (id: string) => {
    pendingId.value = id
    try {
      await customRuleService.toggle(id)
      await fetchRules()
    } finally {
      pendingId.value = null
    }
  }

  return {
    rules,
    loading,
    pendingId,
    enabledRules,
    disabledRules,
    fetchRules,
    addRule,
    updateRule,
    removeRule,
    toggleRule,
  }
})
