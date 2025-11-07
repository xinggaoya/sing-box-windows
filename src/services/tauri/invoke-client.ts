import { invoke } from '@tauri-apps/api/core'
import type { InvokeArgs } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app/AppStore'

export type PortInjectionOption = boolean | string

export interface InvokeContextOptions {
  /**
   * When true, injects the API port under the default "apiPort" key.
   * When a string is provided, it is used as the argument key.
   */
  withApiPort?: PortInjectionOption
  /**
   * When true, injects the proxy port under the default "proxyPort" key.
   * When a string is provided, it is used as the argument key.
   */
  withProxyPort?: PortInjectionOption
  /** Skip waiting for the persisted AppStore data to be restored. */
  skipDataRestore?: boolean
}

async function resolveAppStore(skipDataRestore?: boolean) {
  const store = useAppStore()
  if (!skipDataRestore) {
    await store.waitForDataRestore()
  }
  return store
}

function mergeArgs(base: InvokeArgs | undefined, extra: InvokeArgs): InvokeArgs {
  return base ? { ...base, ...extra } : extra
}

function resolveArgKey(option: PortInjectionOption | undefined, defaultKey: string) {
  if (typeof option === 'string') {
    return option
  }
  return option ? defaultKey : null
}

/**
 * Unified invoke helper that automatically enriches the arguments with the
 * application level configuration (ports, proxy mode, ...).
 */
export async function invokeWithAppContext<T>(
  command: string,
  args?: InvokeArgs,
  options: InvokeContextOptions = {}
): Promise<T> {
  const store = await resolveAppStore(options.skipDataRestore)

  let finalArgs = args ?? {}

  const apiPortKey = resolveArgKey(options.withApiPort, 'apiPort')
  if (apiPortKey && typeof (finalArgs as Record<string, unknown>)[apiPortKey] === 'undefined') {
    finalArgs = mergeArgs(finalArgs, { [apiPortKey]: store.apiPort })
  }

  const proxyPortKey = resolveArgKey(options.withProxyPort, 'proxyPort')
  if (proxyPortKey && typeof (finalArgs as Record<string, unknown>)[proxyPortKey] === 'undefined') {
    finalArgs = mergeArgs(finalArgs, { [proxyPortKey]: store.proxyPort })
  }

  return invoke<T>(command, finalArgs)
}

/**
 * Helper that gives direct access to the AppStore when the caller needs more
 * than the automatically injected properties.
 */
export async function withAppStore<T>(handler: (store: ReturnType<typeof useAppStore>) => Promise<T>): Promise<T> {
  const store = await resolveAppStore(false)
  return handler(store)
}
