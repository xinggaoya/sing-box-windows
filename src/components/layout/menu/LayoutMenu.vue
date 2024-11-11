<template>
  <div style="height: 100%">
    <n-card content-style="padding: 0;">
      <n-menu
          :options="menuOptions"
          v-model:value="currentMenu"
          :on-update:value="onSelect"
      >
      </n-menu>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import {h, ref} from 'vue'
import {useRouter} from 'vue-router'
import {NIcon} from 'naive-ui'
import {
  AtCircleOutline,
  BalloonOutline,
  BarChartOutline,
  InformationCircleOutline,
  SettingsOutline
} from '@vicons/ionicons5'


const router = useRouter()
const currentMenu = ref(0)

const menuOptions = [
  {
    label: '概括',
    key: 0,
    icon: () => {
      return h(NIcon, null, {default: () => h(BarChartOutline)})
    },
  },
  {
    label: '代理',
    key: 1,
    icon: () => {
      return h(NIcon, null, {default: () => h(BalloonOutline)})
    }
  },
  {
    label: '订阅',
    key: 2,
    icon: () => {
      return h(NIcon, null, {default: () => h(AtCircleOutline)})
    }
  },
  {
    label: '日志',
    key: 3,
    icon: () => {
      return h(NIcon, null, {default: () => h(InformationCircleOutline)})
    }
  },
  {
    label: '设置',
    key: 4,
    icon: () => {
      return h(NIcon, null, {
        default: () => h(SettingsOutline)
      })
    }
  },
]


function onSelect(key: number) {
  switch (key) {
    case 0:
      router.push('/')
      break
    case 1:
      router.push('/proxy')
      break
    case 2:
      router.push('/sub')
      break
    case 3:
      router.push('/log')
      break
    case 4:
      router.push('/setting')
      break
    default:
      break
  }
  currentMenu.value = key
}

</script>

<style lang="scss" scoped>

</style>
