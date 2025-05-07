import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../app/AppStore'
import { tauriApi } from '@/services/tauri-api'

export const useServiceStore = defineStore('service', () => {
  // 服务状态
  const isServiceInstalled = ref(false)
  const isServiceRunning = ref(false)
  const isInstalling = ref(false)
  const isUninstalling = ref(false)
  const isUpdating = ref(false)
  const installError = ref('')
  const needsUpdate = ref(false)
  
  // 检查服务是否已安装 - 通过服务状态检查命令
  async function checkServiceStatus() {
    try {
      const result = await invoke<{ installed: boolean; running: boolean }>('check_service_status')
      isServiceInstalled.value = result.installed
      isServiceRunning.value = result.running
      return { installed: result.installed, running: result.running }
    } catch (error) {
      console.error('检查服务状态失败:', error)
      isServiceInstalled.value = false
      isServiceRunning.value = false
      return { installed: false, running: false }
    }
  }
  
  // 检查服务是否需要更新
  async function checkServiceUpdateNeeded() {
    if (!isServiceInstalled.value) {
      needsUpdate.value = false
      return false
    }
    
    try {
      const result = await invoke<{ success: boolean; need_update: boolean; message: string }>('check_service_update_needed')
      
      if (result.success) {
        needsUpdate.value = result.need_update
        return result.need_update
      } else {
        needsUpdate.value = false
        return false
      }
    } catch (error) {
      console.error('检查服务更新状态失败:', error)
      needsUpdate.value = false
      return false
    }
  }
  
  // 安装服务
  async function installService() {
    const appStore = useAppStore()
    
    if (isInstalling.value) return
    
    isInstalling.value = true
    installError.value = ''
    
    try {
      await invoke('install_service')
      isServiceInstalled.value = true
      isServiceRunning.value = true
      appStore.showSuccessMessage('服务安装成功')
      return true
    } catch (error) {
      console.error('安装服务失败:', error)
      
      // 判断错误信息是否包含"服务已存在"
      const errorStr = String(error).toLowerCase()
      if (errorStr.includes('已安装') || 
          errorStr.includes('already installed') || 
          errorStr.includes('已存在') || 
          errorStr.includes('already exists')) {
        isServiceInstalled.value = true
        isServiceRunning.value = true
        appStore.showInfoMessage('服务已经安装')
        return true
      }
      
      installError.value = error as string
      appStore.showErrorMessage(`服务安装失败: ${error}`)
      return false
    } finally {
      isInstalling.value = false
    }
  }
  
  // 卸载服务
  async function uninstallService() {
    const appStore = useAppStore()
    
    if (isUninstalling.value) return
    
    isUninstalling.value = true
    
    try {
      await invoke('uninstall_service')
      isServiceInstalled.value = false
      isServiceRunning.value = false
      appStore.showSuccessMessage('服务卸载成功')
      return true
    } catch (error) {
      console.error('卸载服务失败:', error)
      appStore.showErrorMessage(`服务卸载失败: ${error}`)
      return false
    } finally {
      isUninstalling.value = false
    }
  }
  
  // 更新服务
  async function updateService() {
    const appStore = useAppStore()
    
    if (isUpdating.value) return false
    
    isUpdating.value = true
    
    try {
      const result = await invoke<{ success: boolean; updated: boolean; message: string }>('update_service')
      
      if (result.success) {
        if (result.updated) {
          appStore.showSuccessMessage('服务已更新')
        } else {
          appStore.showInfoMessage(result.message)
        }
        return result.updated
      } else {
        appStore.showErrorMessage(`服务更新失败: ${result.message}`)
        return false
      }
    } catch (error) {
      console.error('更新服务失败:', error)
      appStore.showErrorMessage(`服务更新失败: ${error}`)
      return false
    } finally {
      isUpdating.value = false
    }
  }
  
  return {
    isServiceInstalled,
    isServiceRunning,
    isInstalling,
    isUninstalling,
    isUpdating,
    installError,
    needsUpdate,
    checkServiceStatus,
    checkServiceUpdateNeeded,
    installService,
    uninstallService,
    updateService
  }
}, {
  persist: true, // 添加持久化存储
}) 