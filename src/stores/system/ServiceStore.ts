import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '../app/AppStore'

export const useServiceStore = defineStore('service', () => {
  // 服务状态
  const isServiceInstalled = ref(false)
  const isServiceRunning = ref(false)
  const isInstalling = ref(false)
  const isUninstalling = ref(false)
  const installError = ref('')
  
  // 检查服务是否已安装 - 通过服务安装命令的返回结果判断
  async function checkServiceStatus() {
    try {
      // 直接通过安装服务来判断状态
      // 如果安装服务成功，说明服务不存在
      // 如果返回"服务已存在"的错误，说明服务已安装
      await invoke('install_service')
      // 如果执行到这里，说明安装成功了，但实际上是首次安装
      isServiceInstalled.value = true
      isServiceRunning.value = true
      return { installed: true, running: true }
    } catch (error) {
      console.error('检查服务状态失败:', error)
      
      // 判断错误信息是否包含"服务已存在"
      const errorStr = String(error).toLowerCase()
      if (errorStr.includes('已安装') || 
          errorStr.includes('already installed') || 
          errorStr.includes('已存在') || 
          errorStr.includes('already exists')) {
        isServiceInstalled.value = true
        isServiceRunning.value = true
        return { installed: true, running: true }
      }
      
      isServiceInstalled.value = false
      isServiceRunning.value = false
      return { installed: false, running: false }
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
  
  return {
    isServiceInstalled,
    isServiceRunning,
    isInstalling,
    isUninstalling,
    installError,
    checkServiceStatus,
    installService,
    uninstallService
  }
}, {
  persist: true, // 添加持久化存储
}) 