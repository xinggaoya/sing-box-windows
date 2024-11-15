<script setup lang="ts">
import {RouterView} from 'vue-router'
import {dateZhCN, zhCN} from "naive-ui";
import themeOverrides from '@/assets/naive-ui-theme-overrides.json'

</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides" :locale="zhCN" :date-locale="dateZhCN" class="container">
    <n-message-provider>
      <n-dialog-provider>
        <RouterView/>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style scoped>
</style>
