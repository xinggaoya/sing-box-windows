<template>
  <div class="app-sidebar" :class="{ collapsed }">
    <div class="sider-inner">
      <!-- 导航分组 -->
      <nav class="sider-nav">
        <div v-for="group in groupedItems" :key="group.key" class="nav-group">
          <div v-if="!collapsed && group.label" class="nav-group-label">{{ group.label }}</div>
          <div
            v-for="item in group.items"
            :key="item.key"
            class="nav-item"
            :class="{ active: currentMenu === item.key, disabled: item.disabled }"
            @click="!item.disabled && emit('select', item.key)"
          >
            <n-tooltip v-if="collapsed" placement="right" :delay="200">
              <template #trigger>
                <span class="nav-item-inner">
                  <n-icon :size="20" class="nav-icon">
                    <component :is="item.icon" />
                  </n-icon>
                  <transition name="fade-slide">
                    <span v-if="!collapsed" class="nav-text">{{ item.label }}</span>
                  </transition>
                </span>
              </template>
              {{ item.label }}
            </n-tooltip>
            <span v-else class="nav-item-inner">
              <n-icon :size="20" class="nav-icon">
                <component :is="item.icon" />
              </n-icon>
              <transition name="fade-slide">
                <span v-if="!collapsed" class="nav-text">{{ item.label }}</span>
              </transition>
            </span>
            <span v-if="item.badge" class="nav-badge"></span>
            <span class="active-indicator" v-if="currentMenu === item.key"></span>
          </div>
        </div>
      </nav>

      <!-- 底部操作 -->
      <div class="sider-footer">
        <div class="footer-row">
          <button class="footer-btn" :title="themeLabel" @click="emit('toggle-theme')">
            <n-icon :size="18">
              <MoonOutline v-if="isDark" />
              <SunnyOutline v-else />
            </n-icon>
            <transition name="fade-slide">
              <span v-if="!collapsed" class="footer-text">{{ themeLabel }}</span>
            </transition>
          </button>
        </div>
        <div class="footer-row">
          <button class="footer-btn" :title="collapseLabel" @click="emit('toggle-collapse')">
            <n-icon :size="18">
              <ChevronForwardOutline v-if="collapsed" />
              <ChevronBackOutline v-else />
            </n-icon>
            <transition name="fade-slide">
              <span v-if="!collapsed" class="footer-text">{{ collapseLabel }}</span>
            </transition>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts" setup>
import { computed } from 'vue'
import type { Component } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  MoonOutline,
  SunnyOutline,
  ChevronBackOutline,
  ChevronForwardOutline,
} from '@vicons/ionicons5'

const { t, te } = useI18n()

const tt = (key: string, fallback: string) => (te(key) ? t(key) : fallback)

export interface NavItem {
  label: string
  key: string
  icon: Component
  disabled?: boolean
  badge?: boolean
}

export interface NavGroup {
  key: string
  label?: string
  items: NavItem[]
}

const props = defineProps<{
  collapsed: boolean
  currentMenu: string
  menuItems: NavItem[]
  isDark: boolean
  /** 按功能分组的配置；不传则全部归入默认组 */
  groups?: NavGroup[]
}>()

const emit = defineEmits<{
  (e: 'select', key: string): void
  (e: 'toggle-theme'): void
  (e: 'toggle-collapse'): void
}>()

// 若未提供分组配置，则将所有菜单项放入单一默认组
const groupedItems = computed<NavGroup[]>(() => {
  if (props.groups && props.groups.length > 0) return props.groups
  return [{ key: 'default', items: props.menuItems }]
})

const themeLabel = computed(() =>
  props.isDark ? tt('nav.switchToLight', '切换到亮色') : tt('nav.switchToDark', '切换到暗色'),
)
const collapseLabel = computed(() => (props.collapsed ? t('nav.expand') : t('nav.collapse')))
</script>

<style scoped>
.app-sidebar {
  width: var(--sider-width, 220px);
  height: 100%;
  background: var(--glass-bg);
  backdrop-filter: blur(var(--glass-blur, 18px));
  -webkit-backdrop-filter: blur(var(--glass-blur, 18px));
  border-right: 1px solid var(--border-color);
  transition: width var(--transition-base);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex-shrink: 0;
}

.app-sidebar.collapsed {
  width: var(--sider-collapsed-width, 64px);
}

.sider-inner {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: var(--space-4) var(--space-3);
  min-height: 0;
}

/* 导航 */
.sider-nav {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.nav-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.nav-group-label {
  font-size: var(--text-xs);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--text-tertiary);
  padding: 0 var(--space-3);
  margin-bottom: var(--space-1);
}

.nav-item {
  position: relative;
  height: 42px;
  border-radius: var(--radius-md);
  cursor: pointer;
  color: var(--text-secondary);
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
  display: flex;
  align-items: center;
}

.nav-item:hover:not(.disabled) {
  background: var(--bg-surface-2);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--primary-soft);
  color: var(--primary-color);
  font-weight: 600;
}

.nav-item.disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.nav-item-inner {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 0 var(--space-3);
  width: 100%;
  min-width: 0;
}

.collapsed .nav-item-inner {
  justify-content: center;
  padding: 0;
}

.nav-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.nav-text {
  font-size: var(--text-base);
  font-weight: inherit;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-badge {
  position: absolute;
  top: 8px;
  right: 10px;
  width: 6px;
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--error-color);
}

.active-indicator {
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 18px;
  background: var(--primary-color);
  border-radius: 0 var(--radius-pill) var(--radius-pill) 0;
}

/* 底部 */
.sider-footer {
  padding-top: var(--space-3);
  border-top: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.footer-row {
  display: flex;
}

.footer-btn {
  flex: 1;
  height: 38px;
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: 0 var(--space-3);
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition:
    background var(--transition-fast),
    color var(--transition-fast);
}

.collapsed .footer-btn {
  justify-content: center;
  padding: 0;
}

.footer-btn:hover {
  background: var(--bg-surface-2);
  color: var(--text-primary);
}

.footer-text {
  font-size: var(--text-base);
  white-space: nowrap;
}

/* 过渡 */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all var(--transition-fast);
}

.fade-slide-enter-from,
.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(-6px);
}
</style>
