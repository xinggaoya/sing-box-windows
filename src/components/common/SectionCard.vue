<template>
  <section class="section-card" :class="{ flush: flush }">
    <header v-if="title || $slots.actions || $slots.title" class="section-header">
      <div class="section-heading">
        <div v-if="$slots.icon" class="section-icon">
          <slot name="icon"></slot>
        </div>
        <div class="section-titles">
          <h3 v-if="title" class="section-title">
            <slot name="title">{{ title }}</slot>
          </h3>
          <p v-if="subtitle" class="section-subtitle">{{ subtitle }}</p>
        </div>
      </div>
      <div v-if="$slots.actions" class="section-actions">
        <slot name="actions"></slot>
      </div>
    </header>
    <div class="section-body" :class="{ 'no-pad': flush }">
      <slot></slot>
    </div>
  </section>
</template>

<script lang="ts" setup>
/**
 * 通用卡片容器：统一标题/副标题/操作区/图标/内容区结构。
 * 替代散落的 .surface-card / .content-card 类。
 */
withDefaults(
  defineProps<{
    title?: string
    subtitle?: string
    /** 内容区不内边距（用于内部自带间距的表格/列表） */
    flush?: boolean
  }>(),
  { flush: false },
)
</script>

<style scoped>
.section-card {
  background: var(--panel-bg);
  border: 1px solid var(--panel-border);
  border-radius: var(--radius-xl);
  box-shadow: var(--panel-shadow);
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  padding: var(--space-5) var(--space-5) var(--space-3);
  flex-wrap: wrap;
}

.section-heading {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-width: 0;
}

.section-icon {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  background: var(--primary-soft);
  color: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.section-titles {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.section-title {
  margin: 0;
  font-size: var(--text-md);
  font-weight: 700;
  color: var(--text-primary);
  line-height: 1.3;
}

.section-subtitle {
  margin: 0;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.section-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-wrap: wrap;
}

.section-body {
  padding: var(--space-3) var(--space-5) var(--space-5);
}

.section-body.no-pad {
  padding: 0;
}
</style>
