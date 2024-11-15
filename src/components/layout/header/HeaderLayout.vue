<script setup lang="ts">
import { BulbOutline, ChevronDown, CloseCircleOutline, Code } from '@vicons/ionicons5'
import logo from '@/assets/icon.png'
import { Window } from '@tauri-apps/api/window'

const appWindow = Window.getCurrent()

function handleDark() {}

async function handelCloseWindow() {
  await appWindow.hide()
}

async function handelHideWindow() {
  await appWindow.minimize()
}

async function handelFullScreen() {
  if (await appWindow.isFullscreen()) {
    await appWindow.setFullscreen(false)
  } else {
    await appWindow.setFullscreen(true)
  }
}
</script>

<template>
  <n-card class="layout-header" content-style="padding: 10px">
    <n-space justify="space-between" data-tauri-drag-region>
      <n-flex>
        <n-image :src="logo" width="38" height="38" />
        <n-text class="header-text" strong>Sing-Box-Windows</n-text>
      </n-flex>
      <n-flex>
        <!--        <n-button @click="handleDark" strong secondary circle type="default">-->
        <!--          <template #icon>-->
        <!--            <n-icon>-->
        <!--              <BulbOutline />-->
        <!--            </n-icon>-->
        <!--          </template>-->
        <!--        </n-button>-->
        <n-button @click="handelFullScreen" strong secondary circle type="default">
          <template #icon>
            <n-icon>
              <Code />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="handelHideWindow" strong secondary circle type="default">
          <template #icon>
            <n-icon>
              <ChevronDown />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="handelCloseWindow" strong secondary circle type="default">
          <template #icon>
            <n-icon>
              <CloseCircleOutline />
            </n-icon>
          </template>
        </n-button>
      </n-flex>
    </n-space>
  </n-card>
</template>

<style scoped>
.header-text {
  font-weight: bold;
  line-height: 32px;
  font-size: 20px;
}
</style>
