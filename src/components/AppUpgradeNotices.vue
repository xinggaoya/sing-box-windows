<template>
  <!-- 这个组件只承载弹窗/通知逻辑，不需要渲染内容 -->
  <div style="display: none"></div>
</template>

<script setup lang="ts">
import { computed, h, onBeforeUnmount, onMounted, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useDialog } from 'naive-ui'
import type { DialogReactive } from 'naive-ui'
import { useAppStore } from '@/stores/app/AppStore'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { invokeWithAppContext } from '@/services/invoke-client'

interface DeprecatedWarningItem {
  message?: string
  description?: string
  deprecatedVersion?: string
  scheduledVersion?: string
}

interface DeprecatedWarningsResult {
  warnings: DeprecatedWarningItem[]
}

const DNS_HIJACK_NOTICE_KEY = 'singbox.dnsHijackNoticeAcknowledged.v1'
const DEPRECATED_NOTICE_KEY = 'singbox.deprecatedNoticeAcknowledged.v1'

const { t, locale } = useI18n()
const dialog = useDialog()
const appStore = useAppStore()
const kernelStore = useKernelStore()

const zh = computed(() => locale.value.startsWith('zh'))
let dnsHijackDialog: DialogReactive | null = null
let deprecatedDialog: DialogReactive | null = null
// pending = 未检查；done = 本次会话已检查；failed = 上次失败（内核未就绪等，等待重试）
let deprecatedCheckState: 'pending' | 'done' | 'failed' = 'pending'
let stopStatusWatch: (() => void) | null = null

// 1.14 起 TUN 默认 hijack 接管系统 DNS；老用户首次升级弹窗告知一次
const maybeShowDnsHijackNotice = () => {
  if (localStorage.getItem(DNS_HIJACK_NOTICE_KEY)) return
  if ((appStore.tunDnsMode || 'hijack') !== 'hijack') return
  if (dnsHijackDialog) return

  dnsHijackDialog = dialog.create({
    title: t('upgradeNotices.dnsHijackTitle'),
    content: t('upgradeNotices.dnsHijackBody'),
    positiveText: zh.value ? '知道了' : 'Got it',
    onPositiveClick: () => {
      try {
        localStorage.setItem(DNS_HIJACK_NOTICE_KEY, '1')
      } catch {
        // localStorage 不可用时每次启动都会提示，可接受
      }
      dnsHijackDialog?.destroy()
      dnsHijackDialog = null
    },
    onClose: () => {
      dnsHijackDialog = null
    },
  })
}

// 内核 gRPC 就绪后拉取弃用告警（仅覆盖自定义配置模板；内置生成配置已迁移）
const checkDeprecatedWarnings = async () => {
  if (deprecatedCheckState === 'done' || deprecatedDialog) return
  try {
    let acknowledged = false
    try {
      acknowledged = Boolean(localStorage.getItem(DEPRECATED_NOTICE_KEY))
    } catch {
      acknowledged = false
    }
    if (acknowledged) {
      deprecatedCheckState = 'done'
      return
    }
    const result = await invokeWithAppContext<DeprecatedWarningsResult>(
      'get_deprecated_warnings',
      undefined,
      { skipDataRestore: true },
    )
    deprecatedCheckState = 'done'
    const lines = (result?.warnings ?? [])
      .map((w) => w.message || w.description || '')
      .filter((line) => line.trim().length > 0)
    if (lines.length === 0) return

    deprecatedDialog = dialog.create({
      title: t('upgradeNotices.deprecatedTitle'),
      // 白名单字段清单逐行展示；naive-ui 字符串 content 不保留换行，用渲染函数输出
      content: () =>
        h(
          'div',
          { style: 'white-space: pre-line; line-height: 1.6' },
          `${t('upgradeNotices.deprecatedBody')}\n${lines.join('\n')}`,
        ),
      positiveText: zh.value ? '知道了' : 'Got it',
      onPositiveClick: () => {
        try {
          localStorage.setItem(DEPRECATED_NOTICE_KEY, '1')
        } catch {
          // localStorage 不可用时下次启动会再次提示
        }
        deprecatedDialog?.destroy()
        deprecatedDialog = null
      },
      onClose: () => {
        deprecatedDialog = null
      },
    })
  } catch {
    // 内核未运行或为旧版本内核：标记失败，下次 api_ready 时重试
    deprecatedCheckState = 'failed'
  }
}

const handleApiReady = (ready: boolean) => {
  if (ready) void checkDeprecatedWarnings()
}

onMounted(async () => {
  try {
    await appStore.waitForDataRestore()
  } catch {
    // 数据恢复失败也不阻塞提示流程
  }
  maybeShowDnsHijackNotice()
  stopStatusWatch = watch(
    () => kernelStore.status.api_ready,
    handleApiReady,
    { immediate: true },
  )
})

onBeforeUnmount(() => {
  stopStatusWatch?.()
  dnsHijackDialog?.destroy()
  deprecatedDialog?.destroy()
})
</script>
