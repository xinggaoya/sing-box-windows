<template>
  <SectionCard :title="t('proxy.outboundOverview')" :subtitle="t('proxy.outboundOverviewDesc')">
    <template #icon>
      <n-icon :size="18"><LayersOutline /></n-icon>
    </template>

    <template #actions>
      <n-button
        size="small"
        secondary
        :loading="loading"
        :disabled="!kernelRunning"
        @click="refresh"
      >
        <template #icon>
          <n-icon :size="14"><RefreshOutline /></n-icon>
        </template>
        {{ t('proxy.outboundRefresh') }}
      </n-button>
    </template>

    <div v-if="!kernelRunning" class="hint">
      {{ t('proxy.outboundKernelStopped') }}
    </div>

    <template v-else-if="outbounds.length === 0">
      <EmptyState :title="t('proxy.outboundEmpty')" :icon="LayersOutline" />
    </template>

    <template v-else>
      <div class="summary">{{ t('proxy.outboundCount', { count: outbounds.length }) }}</div>
      <div class="outbound-list">
        <div v-for="item in outbounds" :key="item.tag" class="outbound-row">
          <span class="outbound-name" :title="item.tag">{{ item.tag }}</span>
          <n-tag size="tiny" round :bordered="false" type="info">{{ item.type }}</n-tag>
          <span class="outbound-delay" :class="delayClass(item.urlTestDelay)">
            {{ formatDelay(item.urlTestDelay) }}
          </span>
        </div>
      </div>
    </template>
  </SectionCard>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { LayersOutline, RefreshOutline } from '@vicons/ionicons5'
import { NIcon, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import SectionCard from '@/components/common/SectionCard.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import { proxyService } from '@/services/proxy-service'
import type { GroupItem } from '@/types/generated/GroupItem'
import { useKernelStore } from '@/stores/kernel/KernelStore'

defineOptions({ name: 'OutboundOverviewCard' })

const { t } = useI18n()
const kernelStore = useKernelStore()

const outbounds = ref<GroupItem[]>([])
const loading = ref(false)

const kernelRunning = computed(() => kernelStore.isRunning)

/**
 * 出站总览：内核里所有出站的平铺快照（GetOutbounds）。
 *
 * 与代理页的「分组视图」不同——这里既包含节点，也包含 urltest/selector 等分组本身，
 * 用于一眼看清"内核当前到底有哪些出站"。
 */
const refresh = async () => {
  if (loading.value) return
  loading.value = true
  try {
    const result = await proxyService.getOutbounds()
    const list = result?.outbounds ?? []
    // 按延迟升序；未测速（0）的排到最后
    outbounds.value = [...list].sort((a, b) => {
      const av = a.urlTestDelay > 0 ? a.urlTestDelay : Number.MAX_SAFE_INTEGER
      const bv = b.urlTestDelay > 0 ? b.urlTestDelay : Number.MAX_SAFE_INTEGER
      return av - bv
    })
  } catch (error) {
    console.warn('[OutboundOverviewCard] get_outbounds failed:', error)
    outbounds.value = []
  } finally {
    loading.value = false
  }
}

const formatDelay = (delay: number) => {
  if (!delay || delay <= 0) return '—'
  return `${delay}ms`
}

const delayClass = (delay: number) => {
  if (!delay || delay <= 0) return 'delay-unknown'
  if (delay < 150) return 'delay-good'
  if (delay < 500) return 'delay-mid'
  return 'delay-slow'
}

onMounted(() => {
  if (kernelRunning.value) void refresh()
})
</script>

<style scoped>
.summary {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  margin-bottom: var(--space-2);
}

.outbound-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  max-height: 320px;
  overflow-y: auto;
}

.outbound-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  border-radius: var(--radius-sm);
  transition: background var(--transition-fast, 0.15s ease);
}

.outbound-row:hover {
  background: var(--bg-surface-2);
}

.outbound-name {
  flex: 1;
  min-width: 0;
  font-size: var(--text-sm);
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.outbound-delay {
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  min-width: 52px;
  text-align: right;
}

.delay-good {
  color: var(--success-color);
}

.delay-mid {
  color: var(--warning-color);
}

.delay-slow {
  color: var(--error-color);
}

.delay-unknown {
  color: var(--text-tertiary);
}

.hint {
  padding: var(--space-2) 0;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
</style>
