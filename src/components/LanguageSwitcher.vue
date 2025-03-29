<template>
    <div class="language-switcher">
      <select v-model="currentLocale" @change="changeLocale">
        <option value="zh">中文</option>
        <option value="ru">Русский</option>
        <option value="en">English</option>
      </select>
      
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, watch } from 'vue'
  import { useI18n } from 'vue-i18n'
  import { GlobeOutline } from '@vicons/ionicons5'
  const { locale } = useI18n({ useScope: 'global' })
  const currentLocale = ref(locale.value)
  
  function changeLocale() {
    locale.value = currentLocale.value
  }
  
  // Сохраняем выбранный язык (необязательно)
  watch(locale, (newVal) => {
    localStorage.setItem('locale', newVal)
  })
  </script>
  
  <style scoped>
  .language-switcher select {
    padding: 4px 8px;
    font-size: 14px;
  }
  </style>
  