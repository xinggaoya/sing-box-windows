<template>
  <div class="service-install-container">
    <n-card class="service-install-card">
      <template #header>
        <div class="service-install-header">
          <n-icon size="48" :depth="3">
            <setting-outlined />
          </n-icon>
          <h1>{{ $t('service.install.title') }}</h1>
        </div>
      </template>
      
      <div class="service-install-content">
        <div class="status-info">
          <p class="description">{{ $t('service.install.description') }}</p>
          
          <!-- 管理员权限状态显示 -->
          <n-alert type="warning" v-if="!isAdmin">
            <div class="admin-alert">
              <span>{{ $t('service.install.notAdmin') }}</span>
              <n-button type="primary" @click="restartAsAdmin" :loading="isRestarting">
                {{ $t('service.install.restartAsAdmin') }}
              </n-button>
            </div>
          </n-alert>
          
          <n-space vertical class="status-items">
            <n-space align="center" justify="space-between">
              <n-space align="center">
                <n-tag :type="serviceStore.isServiceInstalled ? 'success' : 'error'" size="medium">
                  {{ serviceStore.isServiceInstalled ? $t('service.install.installed') : $t('service.install.notInstalled') }}
                </n-tag>
                <span>{{ $t('service.install.serviceStatus') }}</span>
              </n-space>
              <n-button text size="small" @click="refreshServiceStatus" :loading="isRefreshing">
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                {{ $t('service.install.refresh') }}
              </n-button>
            </n-space>
            
            <n-space align="center" v-if="serviceStore.isServiceInstalled">
              <n-tag :type="serviceStore.isServiceRunning ? 'success' : 'warning'" size="medium">
                {{ serviceStore.isServiceRunning ? $t('service.install.running') : $t('service.install.notRunning') }}
              </n-tag>
              <span>{{ $t('service.install.runningStatus') }}</span>
            </n-space>
            
            <n-space align="center">
              <n-tag :type="isAdmin ? 'success' : 'error'" size="medium">
                {{ isAdmin ? $t('service.install.isAdmin') : $t('service.install.isNotAdmin') }}
              </n-tag>
              <span>{{ $t('service.install.adminStatus') }}</span>
            </n-space>
          </n-space>
          
          <!-- 服务信息展示 -->
          <n-collapse v-if="serviceStore.isServiceInstalled" class="service-details">
            <n-collapse-item :title="$t('service.install.serviceDetails')" name="service-info">
              <n-descriptions bordered size="small">
                <n-descriptions-item :label="$t('service.install.serviceName')">
                  SingBoxService
                </n-descriptions-item>
                <n-descriptions-item :label="$t('service.install.startupType')">
                  {{ $t('common.auto') }}
                </n-descriptions-item>
                <n-descriptions-item :label="$t('service.install.servicePath')">
                  {{ servicePath }}
                </n-descriptions-item>
                <n-descriptions-item :label="$t('common.description')">
                  {{ $t('service.install.serviceDescription') }}
                </n-descriptions-item>
              </n-descriptions>
            </n-collapse-item>
          </n-collapse>
        </div>
        
        <div class="action-area">
          <n-alert type="warning" v-if="!serviceStore.isServiceInstalled">
            {{ $t('service.install.requiredWarning') }}
          </n-alert>
          
          <n-alert type="error" v-if="serviceStore.installError">
            {{ serviceStore.installError }}
          </n-alert>
          
          <!-- 操作结果提示 -->
          <n-alert v-if="operationResult.show" :type="operationResult.type" class="operation-result">
            {{ operationResult.message }}
          </n-alert>
          
          <n-space justify="center" class="action-buttons">
            <n-button
              type="primary"
              size="large"
              @click="handleInstall"
              :loading="serviceStore.isInstalling"
              :disabled="serviceStore.isServiceInstalled || !isAdmin"
            >
              {{ serviceStore.isServiceInstalled ? $t('service.install.installed') : $t('service.install.installButton') }}
            </n-button>
            
            <n-button
              type="error"
              size="large"
              @click="handleUninstall"
              :loading="serviceStore.isUninstalling"
              :disabled="!serviceStore.isServiceInstalled || serviceStore.isInstalling || !isAdmin"
            >
              {{ $t('service.install.uninstallButton') }}
            </n-button>
          </n-space>
          
          <n-space justify="center" class="continue-button" v-if="serviceStore.isServiceInstalled">
            <n-button type="success" size="large" @click="navigateToHome">
              {{ $t('service.install.continueButton') }}
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
import { NCard, NButton, NSpace, NTag, NAlert, NIcon, NCollapse, NCollapseItem, NDescriptions, NDescriptionsItem } from 'naive-ui'
import { SettingOutlined } from '@vicons/antd'
import { RefreshOutline } from '@vicons/ionicons5'
import { useServiceStore } from '@/stores/system/ServiceStore'
import { tauriApi } from '@/services/tauri-api'
import { appDataDir } from '@tauri-apps/api/path'
import { useI18n } from 'vue-i18n'

const router = useRouter()
const serviceStore = useServiceStore()
const { t } = useI18n()

// 管理员权限状态
const isAdmin = ref(false)
const isRestarting = ref(false)
const isRefreshing = ref(false)

// 服务路径
const servicePath = ref(t('common.loading'))

// 操作结果提示
const operationResult = ref({
  show: false,
  type: 'info' as 'info' | 'success' | 'error' | 'warning',
  message: ''
})

// 显示操作结果提示
function showOperationResult(type: 'info' | 'success' | 'error' | 'warning', message: string, duration = 5000) {
  operationResult.value = {
    show: true,
    type,
    message
  }
  
  // 自动隐藏提示
  setTimeout(() => {
    operationResult.value.show = false
  }, duration)
}

// 检查管理员权限
async function checkAdminPermission() {
  try {
    isAdmin.value = await tauriApi.system.checkAdmin()
  } catch (error) {
    console.error('检查管理员权限失败:', error)
    isAdmin.value = false
  }
}

// 获取服务路径
async function getServicePath() {
  try {
    const dataDir = await appDataDir()
    servicePath.value = `${dataDir}sing-box-service.exe`
  } catch (error) {
    console.error('获取服务路径失败:', error)
    servicePath.value = t('common.unknown')
  }
}

// 刷新服务状态
async function refreshServiceStatus() {
  try {
    isRefreshing.value = true
    await serviceStore.checkServiceStatus()
    showOperationResult('success', t('service.install.refreshSuccess'))
  } catch (error) {
    console.error('刷新服务状态失败:', error)
    showOperationResult('error', t('service.install.refreshError', { error }))
  } finally {
    isRefreshing.value = false
  }
}

// 以管理员身份重启
async function restartAsAdmin() {
  try {
    isRestarting.value = true
    await tauriApi.system.restartAsAdmin()
  } catch (error) {
    console.error('以管理员身份重启失败:', error)
    showOperationResult('error', t('service.install.restartError', { error }))
  } finally {
    isRestarting.value = false
  }
}

// 安装服务
async function handleInstall() {
  if (!isAdmin.value) {
    return restartAsAdmin()
  }
  
  try {
    const success = await serviceStore.installService()
    if (success) {
      showOperationResult('success', t('service.install.installSuccess'))
      // 获取更新后的服务路径
      await getServicePath()
      // 延迟一点时间，让用户看到安装成功的消息
      setTimeout(() => {
        navigateToHome()
      }, 3000)
    }
  } catch (error) {
    showOperationResult('error', t('service.install.installError', { error }))
  }
}

// 卸载服务
async function handleUninstall() {
  if (!isAdmin.value) {
    return restartAsAdmin()
  }
  
  try {
    const success = await serviceStore.uninstallService()
    if (success) {
      showOperationResult('success', t('service.install.uninstallSuccess'))
    }
  } catch (error) {
    showOperationResult('error', t('service.install.uninstallError', { error }))
  }
}

// 继续使用
function navigateToHome() {
  router.push('/')
}

// 页面加载时检查服务状态和管理员权限
onMounted(async () => {
  await checkAdminPermission()
  await serviceStore.checkServiceStatus()
  
  if (serviceStore.isServiceInstalled) {
    await getServicePath()
  }
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

.service-details {
  margin-top: 12px;
}

.action-area {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.operation-result {
  margin-top: 8px;
}

.action-buttons {
  margin-top: 12px;
}

.continue-button {
  margin-top: 12px;
}

@media (max-width: 600px) {
  .service-install-container {
    padding: 12px;
    align-items: flex-start;
    overflow-y: auto;
  }
  
  .admin-alert {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .admin-alert button {
    width: 100%;
  }
  
  .action-buttons {
    flex-direction: column;
    width: 100%;
  }
  
  .action-buttons button {
    width: 100%;
  }
}
</style> 