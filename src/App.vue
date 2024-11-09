<script setup lang="ts">
import {RouterView} from 'vue-router'
import {dateZhCN, zhCN} from "naive-ui";

const themeOverrides = {
  "common": {
    "borderRadius": "5px",
    "primaryColor": "#409EFF",
    "primaryColorHover": "#66b1ff",
    "primaryColorPressed": "#3a8ee6",
    "primaryColorSuppl": "#409eff",
    "infoColor": "#909399",
    "infoColorHover": "#a6a9ad",
    "infoColorPressed": "#82848a",
    "infoColorSuppl": "#909399",
    "successColor": "#67c23a",
    "successColorHover": "#85ce61",
    "successColorPressed": "#5daf34",
    "successColorSuppl": "#67c23a",
    "warningColor": "#e6a23c",
    "warningColorHover": "#ebb563",
    "warningColorPressed": "#cf9236",
    "warningColorSuppl": "#e6a23c",
    "errorColor": "#f56c6c",
    "errorColorHover": "#f78989",
    "errorColorPressed": "#dd6161",
    "errorColorSuppl": "#f56c6c"
  }
}

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
