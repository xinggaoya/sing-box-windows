<template>
  <!-- 这个组件通过编程方式创建通知，不需要渲染内容 -->
  <div style="display: none"></div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { useNotification, NButton } from 'naive-ui'
import type { NotificationReactive } from 'naive-ui'
import mitt from '@/utils/mitt'
import { useUpdateStore } from '@/stores/app/UpdateStore'
import { useRouter } from 'vue-router'

const { t } = useI18n()
const notification = useNotification()
const updateStore = useUpdateStore()
const router = useRouter()

// 当前活跃的通知实例
let activeNotification: NotificationReactive | null = null

// 跳转到设置页面
// 跳转到设置页面（实际路径为 /setting）
const handleGoToSettings = () => {
  router.push('/setting')
  if (activeNotification) {
    activeNotification.destroy()
    activeNotification = null
  }
}

// 监听更新可用事件
const handleUpdateAvailable = (updateInfo: {
  is_prerelease?: boolean
}) => {
  // 如果已经有通知显示，先销毁它
  if (activeNotification) {
    activeNotification.destroy()
  }

  // 创建新的通知
  const notificationTitle = updateInfo.is_prerelease
    ? t('notification.prereleaseAvailable')
    : t('notification.updateAvailable')

  activeNotification = notification.create({
    title: notificationTitle,
    content: () => t('notification.updatePrompt'),
    action: () =>
      h(
        NButton,
        {
          size: 'small',
          type: 'primary',
          onClick: handleGoToSettings,
        },
        () => t('notification.goToSettings'),
      ),
    duration: 15000, // 15秒后自动关闭
    closable: true,
    onClose: () => {
      activeNotification = null
    },
  })
}

onMounted(() => {
  mitt.on('update-available', handleUpdateAvailable)
})

onBeforeUnmount(() => {
  mitt.off('update-available', handleUpdateAvailable)

  // 清理活跃的通知
  if (activeNotification) {
    activeNotification.destroy()
    activeNotification = null
  }
})
</script>

<style scoped>
.notification-content {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.version-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.release-date,
.file-size {
  display: flex;
  align-items: center;
  gap: 4px;
}

.update-notification {
  max-width: 400px;
}
</style>
