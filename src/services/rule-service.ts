import type { RuleProvidersResponse, RulesResponse } from '@/types/controller'
import { invokeWithAppContext } from './invoke-client'

export const ruleService = {
  getRules(port?: number) {
    const args = typeof port === 'number' ? { port } : undefined
    return invokeWithAppContext<RulesResponse>('get_rules', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port',
    })
  },

  getProviders(port?: number) {
    const args = typeof port === 'number' ? { port } : undefined
    return invokeWithAppContext<RuleProvidersResponse>('get_rule_providers', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port',
    })
  },

  updateProvider(provider: string, port?: number) {
    const args = typeof port === 'number' ? { provider, port } : { provider }
    return invokeWithAppContext<void>('update_rule_provider', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port',
    })
  },

  toggleDisabled(index: number, disabled: boolean, port?: number) {
    const args = typeof port === 'number' ? { index, disabled, port } : { index, disabled }
    return invokeWithAppContext<void>('toggle_rule_disabled', args, {
      withApiPort: typeof port === 'number' ? undefined : 'port',
    })
  },
}
