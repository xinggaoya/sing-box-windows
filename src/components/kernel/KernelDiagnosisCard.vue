<template>
  <n-alert v-if="view" :type="tone" class="kernel-diagnosis" :title="view.title">
    <div class="diagnosis-body">
      <div class="diagnosis-meta">
        <n-tag size="small" :type="tone">{{ diagnosis?.stage }}</n-tag>
        <n-tag size="small" :bordered="false">{{ diagnosis?.kind }}</n-tag>
      </div>

      <ul v-if="view.hints.length" class="diagnosis-actions">
        <li v-for="hint in view.hints" :key="hint">{{ hint }}</li>
      </ul>

      <details v-if="diagnosis?.detail" class="diagnosis-detail-toggle">
        <summary>{{ t('setting.kernel.diagnosis.technicalDetail') }}</summary>
        <div class="diagnosis-detail">{{ diagnosis.detail }}</div>
      </details>
    </div>
  </n-alert>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useKernelStore } from '@/stores/kernel/KernelStore'
import { useKernelDiagnosis } from '@/composables/useKernelDiagnosis'

defineOptions({ name: 'KernelDiagnosisCard' })

const { t } = useI18n()
const kernelStore = useKernelStore()
const { describe } = useKernelDiagnosis()

const diagnosis = computed(() => kernelStore.startupDiagnosis)
const view = computed(() => describe(diagnosis.value))

/** 可自愈的失败用 warning，其余用 error */
const tone = computed<'error' | 'warning'>(() =>
  diagnosis.value?.recoverable ? 'warning' : 'error',
)
</script>

<style scoped>
.kernel-diagnosis {
  border-radius: var(--radius-lg);
}

.diagnosis-body {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.diagnosis-meta {
  display: flex;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.diagnosis-actions {
  margin: 0;
  padding-left: 18px;
}

.diagnosis-detail-toggle {
  font-size: var(--text-sm);
}

.diagnosis-detail-toggle summary {
  cursor: pointer;
  color: var(--text-tertiary);
}

.diagnosis-detail {
  margin-top: var(--space-2);
  white-space: pre-wrap;
  word-break: break-word;
  font-family: var(--font-mono, monospace);
  font-size: var(--text-xs);
  color: var(--text-secondary);
}
</style>
