<template>
  <!-- 这个组件通过编程方式创建通知，不需要渲染内容 -->
  <div style="display: none"></div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { useNotification, NText, NIcon, NSpace, NButton } from 'naive-ui'
import type { NotificationReactive } from 'naive-ui'
import { CalendarOutline, DownloadOutline } from '@vicons/ionicons5'
import mitt from '@/utils/mitt'
import { useUpdateStore } from '@/stores/app/UpdateStore'

const { t } = useI18n()
const notification = useNotification()
const updateStore = useUpdateStore()

// 当前活跃的通知实例
let activeNotification: NotificationReactive | null = null

// 格式化文件大小
const formatFileSize = (size: number): string => {
  if (!size) return ''
  const units = ['B', 'KB', 'MB', 'GB']
  let fileSize = size
  let unitIndex = 0

  while (fileSize >= 1024 && unitIndex < units.length - 1) {
    fileSize /= 1024
    unitIndex++
  }

  return `${fileSize.toFixed(1)} ${units[unitIndex]}`
}

// 格式化发布日期
const formatDate = (dateStr: string): string => {
  if (!dateStr) return ''
  try {
    const date = new Date(dateStr)
    return date.toLocaleDateString('zh-CN', {
      month: 'short',
      day: 'numeric',
    })
  } catch {
    return dateStr
  }
}

// 创建通知内容
const createNotificationContent = (updateInfo: {
  latest_version: string
  release_date?: string
  file_size?: number
  is_prerelease?: boolean
}) => {
  const { latest_version, release_date, file_size, is_prerelease } = updateInfo

  return h(
    'div',
    {
      style: {
        display: 'flex',
        flexDirection: 'column',
        gap: '8px',
      },
    },
    [
      // 版本信息
      h(
        'div',
        {
          style: {
            display: 'flex',
            flexDirection: 'column',
            gap: '2px',
          },
        },
        [
          h(
            'div',
            { style: { display: 'flex', alignItems: 'center', gap: '8px' } },
            [
              h(
                NText,
                { strong: true },
                () => `${t('setting.update.newVersion')}: ${latest_version}`,
              ),
              is_prerelease
                ? h(
                    'span',
                    {
                      style: {
                        padding: '2px 6px',
                        backgroundColor: '#faad14',
                        color: 'white',
                        borderRadius: '4px',
                        fontSize: '10px',
                        fontWeight: 'bold',
                      },
                    },
                    t('setting.update.beta'),
                  )
                : null,
            ].filter(Boolean),
          ),
          h(
            NText,
            { depth: 3, style: { fontSize: '12px' } },
            () => `${t('setting.update.current')}: ${updateStore.appVersion}`,
          ),
        ],
      ),

      // 发布日期（如果有）
      release_date
        ? h(
            'div',
            {
              style: {
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
              },
            },
            [
              h(NIcon, { size: 14 }, () => h(CalendarOutline)),
              h(NText, { depth: 2, style: { fontSize: '12px' } }, () => formatDate(release_date)),
            ],
          )
        : null,

      // 文件大小（如果有）
      file_size
        ? h(
            'div',
            {
              style: {
                display: 'flex',
                alignItems: 'center',
                gap: '4px',
              },
            },
            [
              h(NIcon, { size: 14 }, () => h(DownloadOutline)),
              h(NText, { depth: 2, style: { fontSize: '12px' } }, () => formatFileSize(file_size)),
            ],
          )
        : null,
    ].filter(Boolean),
  )
}

// 处理更新
const handleUpdate = () => {
  if (activeNotification) {
    activeNotification.destroy()
    activeNotification = null
  }

  // 使用完整的更新信息，从 updateStore 中获取
  mitt.emit('show-update-modal', {
    show: true,
    latestVersion: updateStore.latestVersion,
    currentVersion: updateStore.appVersion,
    downloadUrl: updateStore.downloadUrl,
    releaseNotes: updateStore.releaseNotes,
    releaseDate: updateStore.releaseDate,
    fileSize: updateStore.fileSize,
  })
}

// 处理稍后提醒
const handleLater = () => {
  if (activeNotification) {
    activeNotification.destroy()
    activeNotification = null
  }
}

// 处理跳过版本
const handleSkip = () => {
  if (activeNotification) {
    activeNotification.destroy()
    activeNotification = null
  }
  updateStore.skipCurrentVersion()
}

// 监听更新可用事件
const handleUpdateAvailable = (updateInfo: {
  latest_version: string
  release_date?: string
  file_size?: number
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
    content: () => createNotificationContent(updateInfo),
    action: () =>
      h(NSpace, { size: 'small' }, () => [
        h(
          NButton,
          {
            size: 'small',
            quaternary: true,
            onClick: handleSkip,
          },
          () => t('setting.update.skipVersion'),
        ),
        h(
          NButton,
          {
            size: 'small',
            onClick: handleLater,
          },
          () => t('setting.update.later'),
        ),
        h(
          NButton,
          {
            size: 'small',
            type: 'primary',
            onClick: handleUpdate,
          },
          () => t('setting.update.updateNow'),
        ),
      ]),
    duration: 0, // 不自动关闭
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
