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

const message = useMessage()
const dialog = useDialog()
const isTunMode = ref(false)

const handleTunModeChange = async (value: boolean) => {
}
</script> 