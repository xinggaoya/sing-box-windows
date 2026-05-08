<template>
  <n-modal
    v-model:show="show"
    preset="dialog"
    :title="t('setting.network.portSettings')"
    class="modern-modal"
    :style="{ width: '400px' }"
  >
    <n-form label-placement="top">
      <n-form-item :label="t('setting.network.proxyPort')">
        <n-input-number v-model:value="tempProxyPort" :min="1024" :max="65535" />
      </n-form-item>
      <n-form-item :label="t('setting.network.apiPort')">
        <n-input-number v-model:value="tempApiPort" :min="1024" :max="65535" />
      </n-form-item>
    </n-form>
    <template #action>
      <n-space justify="end">
        <n-button @click="show = false">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" @click="savePortSettings" :loading="portSettingsLoading">
          {{ t('common.save') }}
        </n-button>
      </n-space>
    </template>
  </n-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useAppStore } from '@/stores'

const show = defineModel<boolean>('show', { required: true })
const emit = defineEmits<{
  saved: []
}>()

const { t } = useI18n()
const message = useMessage()
const appStore = useAppStore()

const tempProxyPort = ref<number | null>(12080)
const tempApiPort = ref<number | null>(12081)
const portSettingsLoading = ref(false)

const resetPortSettings = () => {
  tempProxyPort.value = appStore.proxyPort
  tempApiPort.value = appStore.apiPort
}

const isValidPort = (value: number | null): value is number =>
  value !== null && Number.isInteger(value) && value >= 1024 && value <= 65535

const savePortSettings = async () => {
  if (!isValidPort(tempProxyPort.value) || !isValidPort(tempApiPort.value)) {
    message.error(t('setting.network.invalidPort'))
    return
  }

  if (tempProxyPort.value === tempApiPort.value) {
    message.error(t('setting.network.portConflict'))
    return
  }

  portSettingsLoading.value = true
  try {
    appStore.proxyPort = tempProxyPort.value
    appStore.apiPort = tempApiPort.value
    await appStore.saveToBackend({ applyRuntime: true })
    message.success(t('common.saveSuccess'))
    show.value = false
    emit('saved')
  } catch (error) {
    message.error(t('common.saveFailed'))
  } finally {
    portSettingsLoading.value = false
  }
}

watch(
  show,
  (visible) => {
    if (visible) {
      resetPortSettings()
    }
  },
  { immediate: true },
)
</script>
