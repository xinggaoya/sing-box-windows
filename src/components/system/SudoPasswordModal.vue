<template>
  <n-modal v-model:show="sudoStore.visible" :mask-closable="false">
    <n-card class="sudo-card" :title="t('home.sudoPassword.title')" :bordered="false" size="large">
      <div class="sudo-desc">{{ t('home.sudoPassword.description') }}</div>

      <n-input
        v-model:value="sudoStore.password"
        type="password"
        show-password-on="click"
        :placeholder="t('home.sudoPassword.placeholder')"
        @keyup.enter="handleSubmit"
      />

      <n-alert v-if="errorText" class="sudo-alert" type="error" :show-icon="false">
        {{ errorText }}
      </n-alert>

      <div class="sudo-actions">
        <n-button @click="sudoStore.close(false)">{{ t('common.cancel') }}</n-button>
        <n-button type="primary" :loading="sudoStore.saving" @click="handleSubmit">
          {{ t('home.sudoPassword.save') }}
        </n-button>
      </div>
    </n-card>
  </n-modal>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useMessage } from 'naive-ui'
import { useSudoStore } from '@/stores'

defineOptions({ name: 'SudoPasswordModal' })

const { t } = useI18n()
const message = useMessage()
const sudoStore = useSudoStore()

const errorText = computed(() => {
  switch (sudoStore.errorCode) {
    case 'empty':
      return t('home.sudoPassword.empty')
    case 'invalid':
      return t('home.sudoPassword.invalid')
    case 'failed':
      return sudoStore.errorDetail
        ? `${t('home.sudoPassword.failed')}：${sudoStore.errorDetail}`
        : t('home.sudoPassword.failed')
    default:
      return ''
  }
})

const handleSubmit = async () => {
  const ok = await sudoStore.submit()
  if (ok) {
    message.success(t('home.sudoPassword.saved'))
  }
}
</script>

<style scoped>
.sudo-card {
  width: 420px;
  max-width: calc(100vw - 32px);
}

.sudo-desc {
  margin-bottom: 12px;
  color: var(--text-secondary);
  line-height: 1.5;
  font-size: 13px;
}

.sudo-alert {
  margin-top: 12px;
}

.sudo-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 16px;
}
</style>

