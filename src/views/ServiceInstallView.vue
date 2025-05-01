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
            <n-space align="center" justify="space-between">
              <n-space align="center">
                <n-tag :type="serviceStore.isServiceInstalled ? 'success' : 'error'" size="medium">
                  {{ serviceStore.isServiceInstalled ? '已安装' : '未安装' }}
                </n-tag>
                <span>服务安装状态</span>
              </n-space>
              <n-button text size="small" @click="refreshServiceStatus" :loading="isRefreshing">
                <template #icon>
                  <n-icon><refresh-outline /></n-icon>
                </template>
                刷新状态
              </n-button>
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
          
          <!-- 服务信息展示 -->
          <n-collapse v-if="serviceStore.isServiceInstalled" class="service-details">
            <n-collapse-item title="服务详细信息" name="service-info">
              <n-descriptions bordered size="small">
                <n-descriptions-item label="服务名称">
                  SingBoxService
                </n-descriptions-item>
                <n-descriptions-item label="启动类型">
                  自动
                </n-descriptions-item>
                <n-descriptions-item label="服务路径">
                  {{ servicePath }}
                </n-descriptions-item>
                <n-descriptions-item label="服务说明">
                  提供Sing-Box代理内核的高权限运行环境，支持TUN模式和系统代理设置
                </n-descriptions-item>
              </n-descriptions>
            </n-collapse-item>
          </n-collapse>
        </div>
        
        <div class="action-area">
          <n-alert type="warning" v-if="!serviceStore.isServiceInstalled">
            您必须安装服务才能使用 Sing-Box 的全部功能，特别是TUN模式
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
import { NCard, NButton, NSpace, NTag, NAlert, NIcon, NCollapse, NCollapseItem, NDescriptions, NDescriptionsItem } from 'naive-ui'
import { SettingOutlined } from '@vicons/antd'
import { RefreshOutline } from '@vicons/ionicons5'
import { useServiceStore } from '@/stores/system/ServiceStore'
import { tauriApi } from '@/services/tauri-api'
import { appDataDir } from '@tauri-apps/api/path'

const router = useRouter()
const serviceStore = useServiceStore()

// 管理员权限状态
const isAdmin = ref(false)
const isRestarting = ref(false)
const isRefreshing = ref(false)

// 服务路径
const servicePath = ref('正在获取...')

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
    servicePath.value = '获取失败'
  }
}

// 刷新服务状态
async function refreshServiceStatus() {
  try {
    isRefreshing.value = true
    await serviceStore.checkServiceStatus()
    showOperationResult('success', '服务状态刷新成功')
  } catch (error) {
    console.error('刷新服务状态失败:', error)
    showOperationResult('error', `刷新服务状态失败: ${error}`)
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
    showOperationResult('error', `以管理员身份重启失败: ${error}`)
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
      showOperationResult('success', '服务安装成功！现在您可以使用TUN模式和更多高级功能')
      // 获取更新后的服务路径
      await getServicePath()
      // 延迟一点时间，让用户看到安装成功的消息
      setTimeout(() => {
        navigateToHome()
      }, 3000)
    }
  } catch (error) {
    showOperationResult('error', `服务安装失败: ${error}`)
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
      showOperationResult('success', '服务卸载成功')
    }
  } catch (error) {
    showOperationResult('error', `服务卸载失败: ${error}`)
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