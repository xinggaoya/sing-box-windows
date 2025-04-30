<template>
  <div class="service-install-container">
    <n-card class="service-install-card">
      <template #header>
        <div class="service-install-header">
          <n-icon size="48" :depth="3">
            <setting-outlined />
          </n-icon>
          <h1>服务安装</h1>
        </div>
      </template>
      
      <div class="service-install-content">
        <div class="status-info">
          <p class="description">Sing-Box 需要安装必要的系统服务才能继续使用。此操作需要管理员权限。</p>
          
          <!-- 管理员权限状态显示 -->
          <n-alert type="warning" v-if="!isAdmin">
            <div class="admin-alert">
              <span>当前应用未以管理员身份运行，无法安装服务</span>
              <n-button type="primary" @click="restartAsAdmin" :loading="isRestarting">
                以管理员身份重启
              </n-button>
            </div>
          </n-alert>
          
          <n-space vertical class="status-items">
            <n-space align="center">
              <n-tag :type="serviceStore.isServiceInstalled ? 'success' : 'error'" size="medium">
                {{ serviceStore.isServiceInstalled ? '已安装' : '未安装' }}
              </n-tag>
              <span>服务安装状态</span>
            </n-space>
            
            <n-space align="center" v-if="serviceStore.isServiceInstalled">
              <n-tag :type="serviceStore.isServiceRunning ? 'success' : 'warning'" size="medium">
                {{ serviceStore.isServiceRunning ? '运行中' : '未运行' }}
              </n-tag>
              <span>服务运行状态</span>
            </n-space>
            
            <n-space align="center">
              <n-tag :type="isAdmin ? 'success' : 'error'" size="medium">
                {{ isAdmin ? '是' : '否' }}
              </n-tag>
              <span>管理员权限</span>
            </n-space>
          </n-space>
        </div>
        
        <div class="action-area">
          <n-alert type="warning" v-if="!serviceStore.isServiceInstalled">
            您必须安装服务才能使用 Sing-Box
          </n-alert>
          
          <n-alert type="error" v-if="serviceStore.installError">
            {{ serviceStore.installError }}
          </n-alert>
          
          <n-space justify="center" class="action-buttons">
            <n-button
              type="primary"
              size="large"
              @click="handleInstall"
              :loading="serviceStore.isInstalling"
              :disabled="serviceStore.isServiceInstalled || !isAdmin"
            >
              {{ serviceStore.isServiceInstalled ? '已安装' : '安装服务' }}
            </n-button>
            
            <n-button
              type="error"
              size="large"
              @click="handleUninstall"
              :loading="serviceStore.isUninstalling"
              :disabled="!serviceStore.isServiceInstalled || serviceStore.isInstalling || !isAdmin"
            >
              卸载服务
            </n-button>
          </n-space>
          
          <n-space justify="center" class="continue-button" v-if="serviceStore.isServiceInstalled">
            <n-button type="success" size="large" @click="navigateToHome">
              继续使用
            </n-button>
          </n-space>
        </div>
      </div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { NCard, NButton, NSpace, NTag, NAlert, NIcon } from 'naive-ui'
import { SettingOutlined } from '@vicons/antd'
import { useServiceStore } from '@/stores/system/ServiceStore'
import { tauriApi } from '@/services/tauri-api'

const router = useRouter()
const serviceStore = useServiceStore()

// 管理员权限状态
const isAdmin = ref(false)
const isRestarting = ref(false)

// 检查管理员权限
async function checkAdminPermission() {
  try {
    isAdmin.value = await tauriApi.system.checkAdmin()
  } catch (error) {
    console.error('检查管理员权限失败:', error)
    isAdmin.value = false
  }
}

// 以管理员身份重启
async function restartAsAdmin() {
  try {
    isRestarting.value = true
    await tauriApi.system.restartAsAdmin()
  } catch (error) {
    console.error('以管理员身份重启失败:', error)
  } finally {
    isRestarting.value = false
  }
}

// 安装服务
async function handleInstall() {
  if (!isAdmin.value) {
    return restartAsAdmin()
  }
  const success = await serviceStore.installService()
  if (success) {
    // 延迟一点时间，让用户看到安装成功的消息
    setTimeout(() => {
      navigateToHome()
    }, 1500)
  }
}

// 卸载服务
async function handleUninstall() {
  if (!isAdmin.value) {
    return restartAsAdmin()
  }
  await serviceStore.uninstallService()
}

// 继续使用
function navigateToHome() {
  router.push('/')
}

// 页面加载时检查服务状态和管理员权限
onMounted(async () => {
  await checkAdminPermission()
  await serviceStore.checkServiceStatus()
})
</script>

<style scoped>
.service-install-container {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100vh;
  padding: 20px;
  background-color: var(--n-color-modal-backdrop);
}

.service-install-card {
  width: 100%;
  max-width: 600px;
  border-radius: 12px;
}

.service-install-header {
  display: flex;
  align-items: center;
  gap: 16px;
}

.service-install-header h1 {
  margin: 0;
  font-size: 24px;
}

.service-install-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.status-info {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.description {
  font-size: 16px;
  line-height: 1.6;
  margin: 0;
}

.admin-alert {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 12px;
}

.status-items {
  margin-top: 12px;
}

.action-area {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.action-buttons {
  margin-top: 12px;
}

.continue-button {
  margin-top: 12px;
}
</style> 