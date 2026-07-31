<template>
  <div class="page-header">
    <div class="header-left">
      <div v-if="$slots.icon" class="header-icon">
        <slot name="icon"></slot>
      </div>
      <div class="header-main">
        <h2 class="header-title">{{ title }}</h2>
        <p v-if="subtitle" class="header-subtitle">{{ subtitle }}</p>
      </div>
    </div>
    <div class="header-right">
      <div v-if="$slots.extra" class="header-extra">
        <slot name="extra"></slot>
      </div>
      <div v-if="$slots.actions" class="header-actions">
        <slot name="actions"></slot>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
/**
 * 页面头部：图标 + 标题/副标题 + 额外信息/操作区。
 * 被订阅/日志/连接/规则等页面复用。
 */
defineProps<{
  title: string
  subtitle?: string
}>()
</script>

<style scoped>
.page-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--space-4);
  flex-wrap: wrap;
  margin-bottom: var(--space-5);
}

.header-left {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  min-width: 0;
}

.header-icon {
  width: 44px;
  height: 44px;
  border-radius: var(--radius-lg);
  background: linear-gradient(135deg, var(--primary-color), var(--primary-hover));
  color: var(--primary-contrast);
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 8px 20px var(--primary-soft-strong);
  flex-shrink: 0;
}

.header-main {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.header-title {
  font-size: var(--text-2xl);
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  line-height: 1.2;
  letter-spacing: -0.02em;
}

.header-subtitle {
  font-size: var(--text-base);
  color: var(--text-secondary);
  margin: 0;
}

.header-right {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.header-extra {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

@media (max-width: 600px) {
  .page-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .header-right {
    width: 100%;
  }
}
</style>
