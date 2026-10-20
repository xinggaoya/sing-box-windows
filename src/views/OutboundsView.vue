<template>
  <div class="page-shell outbounds-page">
    <PageHeader :title="t('outbounds.title')" :subtitle="t('outbounds.subtitle')">
      <template #icon>
        <n-icon :size="22"><LayersOutline /></n-icon>
      </template>
      <template #actions>
        <n-button secondary :loading="loading" :disabled="!kernelRunning" @click="refresh">
          <template #icon>
            <n-icon><RefreshOutline /></n-icon>
          </template>
          {{ t('outbounds.refresh') }}
        </n-button>
      </template>
    </PageHeader>

    <ToolbarBar>
      <template #filters>
        <n-input
          v-model:value="searchQuery"
          size="small"
          clearable
          :placeholder="t('outbounds.search')"
          class="outbound-search"
        >
          <template #prefix>
            <n-icon><SearchOutline /></n-icon>
          </template>
        </n-input>
      </template>
      <template #stats>
        <template v-if="kernelRunning">
          <span class="outbound-stat">{{ t('outbounds.count', { count: stats.total }) }}</span>
          <span class="outbound-stat">{{
            t('outbounds.measured', { count: stats.measured })
          }}</span>
          <span v-if="stats.fastest" class="outbound-stat" :class="delayClass(stats.fastest)">
            {{ t('outbounds.fastest', { delay: formatOutboundDelay(stats.fastest) }) }}
          </span>
          <span v-if="stats.slowest" class="outbound-stat">
            {{ t('outbounds.slowest', { delay: formatOutboundDelay(stats.slowest) }) }}
          </span>
        </template>
      </template>
    </ToolbarBar>

    <div class="outbounds-card">
      <div class="outbounds-head">
        <span class="col-name">{{ t('outbounds.columnName') }}</span>
        <span class="col-type">{{ t('outbounds.columnType') }}</span>
        <span class="col-delay">{{ t('outbounds.columnDelay') }}</span>
      </div>

      <div v-if="!kernelRunning" class="outbounds-hint">
        <n-icon :size="16"><InformationCircleOutline /></n-icon>
        <span>{{ t('outbounds.kernelStopped') }}</span>
      </div>

      <EmptyState
        v-else-if="!outbounds.length"
        :title="t('outbounds.empty')"
        :icon="LayersOutline"
      />

      <EmptyState
        v-else-if="!filtered.length"
        :title="t('outbounds.emptyFiltered')"
        :icon="SearchOutline"
      />

      <div v-else class="outbounds-rows">
        <div v-for="item in filtered" :key="item.tag" class="outbound-row">
          <span class="col-name" :title="item.tag">{{ item.tag }}</span>
          <span class="col-type">
            <n-tag size="tiny" round :bordered="false" type="info">{{ item.type }}</n-tag>
          </span>
          <span class="col-delay" :class="delayClass(item.urlTestDelay)">
            {{ formatOutboundDelay(item.urlTestDelay) }}
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue'
import {
  InformationCircleOutline,
  LayersOutline,
  RefreshOutline,
  SearchOutline,
} from '@vicons/ionicons5'
import { NIcon, NTag } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import PageHeader from '@/components/common/PageHeader.vue'
import ToolbarBar from '@/components/common/ToolbarBar.vue'
import EmptyState from '@/components/common/EmptyState.vue'
import { useOutboundList, formatOutboundDelay } from '@/composables/useOutboundList'
import { useKernelStore } from '@/stores/kernel/KernelStore'

defineOptions({ name: 'OutboundsView' })

const { t } = useI18n()
const kernelStore = useKernelStore()

const { outbounds, loading, searchQuery, filtered, stats, refresh } = useOutboundList()

const kernelRunning = computed(() => kernelStore.isRunning)

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
/* 页面样式抽到 `@/assets/views/OutboundsView.css`，与其它页面保持一致。 */
@import '@/assets/views/OutboundsView.css';
</style>
