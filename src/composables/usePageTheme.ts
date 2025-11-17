import { computed } from 'vue'
import { useThemeStore } from '@/stores/app/ThemeStore'

export const usePageTheme = (store?: ReturnType<typeof useThemeStore>) => {
  const themeStore = store ?? useThemeStore()

  return computed(() => {
    const isDark = themeStore.isDark

    return {
      '--page-bg': isDark ? '#050816' : '#f6f7fb',
      '--hero-bg': isDark
        ? 'linear-gradient(135deg, rgba(24, 26, 56, 0.95), rgba(5, 6, 20, 0.95))'
        : 'linear-gradient(135deg, rgba(238, 242, 255, 0.96), rgba(250, 245, 255, 0.95))',
      '--hero-border': isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(15, 23, 42, 0.08)',
      '--hero-shadow': isDark ? '0 30px 70px rgba(2, 6, 23, 0.65)' : '0 30px 70px rgba(15, 23, 42, 0.12)',
      '--panel-bg': isDark ? 'rgba(14, 18, 33, 0.92)' : 'rgba(255, 255, 255, 0.95)',
      '--panel-border': isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(15, 23, 42, 0.08)',
      '--panel-shadow': isDark ? '0 25px 60px rgba(2, 6, 23, 0.55)' : '0 25px 60px rgba(15, 23, 42, 0.08)',
      '--text-primary': isDark ? '#f8fafc' : '#0f172a',
      '--text-muted': isDark ? '#a5b4fc' : '#475569',
      '--chip-bg': isDark ? 'rgba(148, 163, 184, 0.18)' : 'rgba(15, 23, 42, 0.06)',
      '--chip-text': isDark ? '#e0e7ff' : '#312e81',
      '--divider-color': isDark ? 'rgba(255, 255, 255, 0.08)' : 'rgba(15, 23, 42, 0.08)',
    }
  })
}
