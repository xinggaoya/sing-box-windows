<template>
  <n-space vertical>
    <n-card>
      <n-space align="center" justify="space-between">
        <n-space>
          <n-h3 style="margin: 0">TUN 模式</n-h3>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-icon size="20">
                <HelpCircleOutline />
              </n-icon>
            </template>
            TUN模式需要管理员权限，可以实现全局透明代理
          </n-tooltip>
        </n-space>
        <n-switch v-model:value="isTunMode" @update:value="handleTunModeChange" />
      </n-space>
    </n-card>
  </n-space>
</template>

<script lang="ts" setup>
import { ref } from 'vue'
import { useMessage, useDialog } from 'naive-ui'
import { HelpCircleOutline } from '@vicons/ionicons5'
import { invoke } from '@tauri-apps/api/tauri'

const message = useMessage()
const dialog = useDialog()
const isTunMode = ref(false)

const handleTunModeChange = async (value: boolean) => {
  if (value) {
    try {
      await invoke('set_tun_proxy')
      message.success('已切换到TUN模式')
      isTunMode.value = true
    } catch (error: any) {
      if (error.includes('需要管理员权限')) {
        dialog.warning({
          title: '需要管理员权限',
          content: '切换到TUN模式需要管理员权限，是否以管理员身份重启应用？',
          positiveText: '重启',
          negativeText: '取消',
          onPositiveClick: async () => {
            try {
              await invoke('restart_as_admin')
            } catch (e) {
              message.error('重启失败')
              isTunMode.value = false
            }
          },
          onNegativeClick: () => {
            isTunMode.value = false
          }
        })
      } else {
        message.error('切换TUN模式失败：' + error)
        isTunMode.value = false
      }
    }
  } else {
    try {
      await invoke('set_system_proxy')
      message.success('已切换到系统代理模式')
      isTunMode.value = false
    } catch (error) {
      message.error('切换系统代理模式失败')
      isTunMode.value = true
    }
  }
}
</script> 