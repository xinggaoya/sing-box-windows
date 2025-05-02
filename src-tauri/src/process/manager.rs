use super::{ProcessError, ProcessInfo, ProcessStatus, Result};
use crate::app::constants::{messages, network_config, paths};
use crate::utils::proxy_util::disable_system_proxy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

// 定义API响应的数据结构
#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

// 定义状态响应的数据结构
#[derive(Debug, Serialize, Deserialize)]
struct StatusResponse {
    running: bool,
    pid: Option<u32>,
    // 可以根据实际返回添加更多字段
}

pub struct ProcessManager {
    client: Client,
    api_base_url: String,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_base_url: format!("http://127.0.0.1:{}/api", network_config::DEFAULT_API_PORT),
        }
    }

    // 获取进程状态
    pub async fn get_status(&self) -> ProcessInfo {
        match self.query_status().await {
            Ok((running, pid)) => ProcessInfo {
                pid,
                status: if running {
                    ProcessStatus::Running
                } else {
                    ProcessStatus::Stopped
                },
                last_error: None,
            },
            Err(e) => ProcessInfo {
                pid: None,
                status: ProcessStatus::Failed(e.to_string()),
                last_error: Some(e.to_string()),
            },
        }
    }

    // 查询API获取状态
    async fn query_status(&self) -> Result<(bool, Option<u32>)> {
        let url = format!("{}/status", self.api_base_url);

        match self.client.get(&url).send().await {
            Ok(response) => {
                // 先保存状态码
                let status_code = response.status();
                let is_success = status_code.is_success();

                if is_success {
                    match response.json::<ApiResponse>().await {
                        Ok(api_resp) => {
                            if api_resp.success && api_resp.data.is_some() {
                                if let Ok(status_data) =
                                    serde_json::from_value::<StatusResponse>(api_resp.data.unwrap())
                                {
                                    return Ok((status_data.running, status_data.pid));
                                }
                            }
                            // API返回成功但无数据，可能是服务未运行
                            warn!("获取状态信息: {}", api_resp.message);
                            return Ok((false, None));
                        }
                        Err(e) => {
                            error!("解析状态响应失败: {}", e);
                            return Err(ProcessError::Other(format!("解析状态响应失败: {}", e)));
                        }
                    }
                } else {
                    let err_msg = format!("获取状态响应错误: HTTP {}", status_code);
                    error!("{}", err_msg);
                    return Err(ProcessError::Other(err_msg));
                }
            }
            Err(e) => {
                // 服务可能未启动
                return Err(ProcessError::Other(format!("获取状态请求失败: {}", e)));
            }
        }
    }

    // 检查进程是否在运行
    pub async fn is_running(&self) -> bool {
        match self.query_status().await {
            Ok((running, _)) => running,
            Err(_) => false,
        }
    }

    // 启动进程
    pub async fn start(&self) -> Result<()> {
        // 检查配置文件
        self.check_config().await?;

        // 调用API启动服务
        let url = format!("{}/start", self.api_base_url);

        match self.client.post(&url).send().await {
            Ok(response) => {
                // 先保存状态码
                let status_code = response.status();
                let is_success = status_code.is_success();

                if is_success {
                    match response.json::<ApiResponse>().await {
                        Ok(api_resp) => {
                            if api_resp.success {
                                // 启动成功，等待一段时间确保服务已启动
                                sleep(Duration::from_secs(1)).await;
                                info!("{}", messages::INFO_PROCESS_STARTED);
                                return Ok(());
                            } else {
                                return Err(ProcessError::StartFailed(format!(
                                    "启动失败: {}",
                                    api_resp.message
                                )));
                            }
                        }
                        Err(e) => {
                            // 尝试处理非标准JSON响应
                            if is_success {
                                // 虽然解析失败，但HTTP状态是成功的，可能API返回非标准格式
                                // 假设操作成功
                                sleep(Duration::from_secs(1)).await;
                                info!("内核启动成功（响应格式不标准但状态码正常）");
                                return Ok(());
                            }

                            return Err(ProcessError::StartFailed(format!(
                                "解析启动响应失败: {}",
                                e
                            )));
                        }
                    }
                } else {
                    return Err(ProcessError::StartFailed(format!(
                        "启动响应错误: HTTP {}",
                        status_code
                    )));
                }
            }
            Err(e) => {
                return Err(ProcessError::StartFailed(format!("启动请求失败: {}", e)));
            }
        }
    }

    // 停止进程
    pub async fn stop(&self) -> Result<()> {
        // 调用API停止服务
        let url = format!("{}/stop", self.api_base_url);

        match self.client.post(&url).send().await {
            Ok(response) => {
                // 先保存状态码
                let status_code = response.status();
                let is_success = status_code.is_success();

                if is_success {
                    match response.json::<ApiResponse>().await {
                        Ok(api_resp) => {
                            if api_resp.success {
                                // 停止成功，等待一段时间确保服务已停止
                                sleep(Duration::from_secs(1)).await;

                                // 关闭系统代理
                                if let Err(e) = disable_system_proxy() {
                                    warn!("关闭系统代理失败: {}", e);
                                } else {
                                    info!("{}", messages::INFO_SYSTEM_PROXY_DISABLED);
                                }

                                info!("{}", messages::INFO_PROCESS_STOPPED);
                                return Ok(());
                            } else {
                                return Err(ProcessError::StopFailed(format!(
                                    "停止失败: {}",
                                    api_resp.message
                                )));
                            }
                        }
                        Err(e) => {
                            // 尝试处理非标准JSON响应
                            if is_success {
                                // 虽然解析失败，但HTTP状态是成功的，可能API返回非标准格式
                                // 假设操作成功
                                sleep(Duration::from_secs(1)).await;

                                // 关闭系统代理
                                if let Err(e) = disable_system_proxy() {
                                    warn!("关闭系统代理失败: {}", e);
                                } else {
                                    info!("{}", messages::INFO_SYSTEM_PROXY_DISABLED);
                                }

                                info!("内核停止成功（响应格式不标准但状态码正常）");
                                return Ok(());
                            }

                            return Err(ProcessError::StopFailed(format!(
                                "解析停止响应失败: {}",
                                e
                            )));
                        }
                    }
                } else {
                    return Err(ProcessError::StopFailed(format!(
                        "停止响应错误: HTTP {}",
                        status_code
                    )));
                }
            }
            Err(e) => {
                return Err(ProcessError::StopFailed(format!("停止请求失败: {}", e)));
            }
        }
    }

    // 重启进程
    pub async fn restart(&self) -> Result<()> {
        // 调用API重启服务
        let url = format!("{}/restart", self.api_base_url);

        match self.client.post(&url).send().await {
            Ok(response) => {
                // 先保存状态码
                let status_code = response.status();
                let is_success = status_code.is_success();

                if is_success {
                    match response.json::<ApiResponse>().await {
                        Ok(api_resp) => {
                            if api_resp.success {
                                // 重启成功，等待一段时间确保服务已重启
                                sleep(Duration::from_secs(2)).await;
                                info!("内核已重启");
                                return Ok(());
                            } else {
                                return Err(ProcessError::Other(format!(
                                    "重启失败: {}",
                                    api_resp.message
                                )));
                            }
                        }
                        Err(e) => {
                            // 尝试处理非标准JSON响应
                            if is_success {
                                // 虽然解析失败，但HTTP状态是成功的，可能API返回非标准格式
                                // 假设操作成功
                                sleep(Duration::from_secs(2)).await;
                                info!("内核重启成功（响应格式不标准但状态码正常）");
                                return Ok(());
                            }

                            return Err(ProcessError::Other(format!("解析重启响应失败: {}", e)));
                        }
                    }
                } else {
                    return Err(ProcessError::Other(format!(
                        "重启响应错误: HTTP {}",
                        status_code
                    )));
                }
            }
            Err(e) => {
                // 尝试先停止再启动，以防API不支持直接重启
                warn!("重启请求失败: {}，尝试停止后再启动", e);
                self.stop().await?;
                // 休眠1s
                sleep(Duration::from_secs(1)).await;
                self.start().await?;
                return Ok(());
            }
        }
    }

    // 检查配置文件
    async fn check_config(&self) -> Result<()> {
        // 检查配置文件是否存在
        let config_path = paths::get_config_path();
        if !config_path.exists() {
            return Err(ProcessError::ConfigError(
                messages::ERR_CONFIG_READ_FAILED.to_string(),
            ));
        }

        // 验证配置文件
        let config_str = std::fs::read_to_string(&config_path).map_err(|e| {
            ProcessError::ConfigError(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))
        })?;

        // 解析JSON
        let json_result: serde_json::Result<serde_json::Value> = serde_json::from_str(&config_str);
        if let Err(e) = json_result {
            return Err(ProcessError::ConfigError(format!(
                "配置文件JSON格式错误: {}",
                e
            )));
        }

        info!("配置文件检查通过");
        Ok(())
    }
}
