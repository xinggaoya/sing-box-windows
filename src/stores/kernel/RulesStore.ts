import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { ruleService } from '@/services/rule-service'
import type { RuleItem, RuleProvider } from '@/types/controller'

const normalizeRules = (input: RuleItem[] | Record<string, RuleItem>) => {
  if (Array.isArray(input)) {
    return input.map((rule, index) => ({
      ...rule,
      index: typeof rule.index === 'number' ? rule.index : index,
    }))
  }

  return Object.entries(input).map(([key, rule], index) => ({
    ...rule,
    index: typeof rule.index === 'number' ? rule.index : Number(key) || index,
  }))
}

export const useRulesStore = defineStore('rules', () => {
  const loading = ref(false)
  const rules = ref<RuleItem[]>([])
  const providers = ref<RuleProvider[]>([])
  const providerUpdatingMap = ref<Record<string, boolean>>({})
  const ruleUpdatingMap = ref<Record<number, boolean>>({})

  const fetchAll = async () => {
    loading.value = true
    try {
      const [rulesResponse, providersResponse] = await Promise.all([
        ruleService.getRules(),
        ruleService.getProviders(),
      ])

      rules.value = normalizeRules(rulesResponse.rules)
      providers.value = Object.values(providersResponse.providers || {})
    } finally {
      loading.value = false
    }
  }

  const updateProvider = async (providerName: string) => {
    providerUpdatingMap.value = {
      ...providerUpdatingMap.value,
      [providerName]: true,
    }

    try {
      await ruleService.updateProvider(providerName)
      await fetchAll()
    } finally {
      providerUpdatingMap.value = {
        ...providerUpdatingMap.value,
        [providerName]: false,
      }
    }
  }

  const updateAllProviders = async () => {
    await Promise.all(providers.value.map((provider) => updateProvider(provider.name)))
  }

  const toggleDisabled = async (rule: RuleItem) => {
    if (typeof rule.index !== 'number') return

    ruleUpdatingMap.value = {
      ...ruleUpdatingMap.value,
      [rule.index]: true,
    }

    try {
      await ruleService.toggleDisabled(rule.index, !rule.extra?.disabled)
      rules.value = rules.value.map((item) =>
        item.index === rule.index
          ? {
              ...item,
              extra: {
                ...item.extra,
                disabled: !item.extra?.disabled,
              },
            }
          : item,
      )
    } finally {
      ruleUpdatingMap.value = {
        ...ruleUpdatingMap.value,
        [rule.index]: false,
      }
    }
  }

  const ruleTypes = computed(() => Array.from(new Set(rules.value.map((rule) => rule.type))))

  return {
    loading,
    rules,
    providers,
    providerUpdatingMap,
    ruleUpdatingMap,
    ruleTypes,
    fetchAll,
    updateProvider,
    updateAllProviders,
    toggleDisabled,
  }
})
