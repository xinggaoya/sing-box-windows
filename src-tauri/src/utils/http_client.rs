use reqwest::Client;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// 统一 UA：此前散落着 "sing-box-windows/1.0 (...)" 与 "sing-box-windows/1.8.2"
/// 两个版本号（后者早已与项目版本脱节）
const USER_AGENT: &str = "sing-box-windows/1.0 (sing-box; compatible; Windows NT 10.0)";

/// 全局 HTTP 客户端管理器
/// 提供高效的连接池和重用机制
/// 全局 HTTP 客户端管理器
/// 提供高效的连接池和重用机制
pub struct HttpClientManager {
    client: Client,
    proxy_client: Client,
    /// 300s 超时：内核二进制下载
    long_download: Client,
    /// 20s 超时：GitHub Release 元数据
    github_api: Client,
    /// 60s 超时：rule-set 压缩包下载
    rule_set: Client,
    /// 6s 超时：连通性探测
    connectivity_probe: Client,
    /// 按代理 URL 缓存的客户端（见 [`HttpClientManager::client_for_proxy`]）
    proxy_clients: RwLock<HashMap<String, Arc<Client>>>,
}

/// 客户端档位：把「同一用途」的超时/连接池参数收敛到一个枚举，
/// 避免同用途散落出多个略有差异的客户端实例。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProfile {
    /// 默认：30s 超时，适用于常规 GET / JSON / 文本
    Default,
    /// 300s 超时，适用于内核二进制等大文件下载
    LongDownload,
    /// 20s 超时，适用于 GitHub Release API
    GitHubApi,
    /// 60s 超时，适用于 rule-set 压缩包
    RuleSet,
    /// 6s 超时，适用于连通性探测
    ConnectivityProbe,
}

impl Default for HttpClientManager {
    fn default() -> Self {
        Self::new_or_fallback()
    }
}

impl HttpClientManager {
    /// 创建新的 HTTP 客户端管理器。
    ///
    /// 返回 `Result` 而非内部 `expect`：原先的 `expect` 位于 `lazy_static` 初始化里，
    /// 一旦构建失败不仅当场 panic，还会**毒化静态量**（`lazy_static` 在初始化 panic 后
    /// 会持续 panic），把一次局部的客户端构建失败放大成“此后所有 HTTP 调用全挂”。
    pub fn new() -> Result<Self, reqwest::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(10))
            .no_proxy()
            .user_agent(USER_AGENT)
            .build()?;

        // 用于代理测试的客户端，超时时间更短
        let proxy_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(5)
            .pool_idle_timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .no_proxy()
            .user_agent(USER_AGENT)
            .build()?;

        let long_download = Self::build_user_agent(Duration::from_secs(300), 2)?;
        let github_api = Self::build_user_agent(Duration::from_secs(20), 4)?;
        let rule_set = Self::build_user_agent(Duration::from_secs(60), 4)?;
        let connectivity_probe = Self::build_user_agent(Duration::from_secs(6), 4)?;

        Ok(Self {
            client,
            proxy_client,
            long_download,
            github_api,
            rule_set,
            connectivity_probe,
            proxy_clients: RwLock::new(HashMap::new()),
        })
    }

    /// 构建带统一 UA 的无代理客户端。
    fn build_user_agent(timeout: Duration, idle_per_host: usize) -> Result<Client, reqwest::Error> {
        Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(idle_per_host)
            // 禁用系统代理：TUN 残留的 127.0.0.1 代理配置会让请求走错路
            .no_proxy()
            .user_agent(USER_AGENT)
            .build()
    }

    /// 带降级的构造：自定义参数构建失败时退回 reqwest 默认客户端。
    ///
    /// 我们的超时 / 连接池参数全是常量，实际不可构建；但保留降级路径后，
    /// 即便将来新增了非法配置，也只是退化为默认超时，而不会毒化全局静态量。
    /// `Client::new()` 使用 reqwest 默认 TLS 后端与参数，是官方提供的不可失败构造。
    fn new_or_fallback() -> Self {
        match Self::new() {
            Ok(manager) => manager,
            Err(error) => {
                tracing::error!(
                    "⚠️ 构建带自定义参数的 HTTP 客户端失败，已退回 reqwest 默认客户端（超时/连接池将不生效）: {}",
                    error
                );
                // 任一档位构建失败都退回 reqwest 默认客户端，保证调用方拿到的仍是可用实例
                let fallback = || Client::new();
                Self {
                    client: fallback(),
                    proxy_client: fallback(),
                    long_download: fallback(),
                    github_api: fallback(),
                    rule_set: fallback(),
                    connectivity_probe: fallback(),
                    proxy_clients: RwLock::new(HashMap::new()),
                }
            }
        }
    }

    /// 获取标准HTTP客户端
    pub fn get_client(&self) -> &Client {
        &self.client
    }

    /// 获取代理测试专用客户端
    pub fn get_proxy_client(&self) -> &Client {
        &self.proxy_client
    }

    /// 按用途档位获取（并缓存）客户端。
    ///
    /// 此前全仓有 11 处各自 `reqwest::Client::builder()`，其中
    /// `system_service::perform_proxy_probe` 被内核守护循环**每 8 秒**调用一次，
    /// 每次都重建一个带 `native-tls-vendored` 根证书的完整客户端 —— 既浪费又让
    /// 探测耗时的方差变大。现在按档位缓存，每个档位整个进程只构建一次。
    pub fn profile(&self, profile: ClientProfile) -> &Client {
        match profile {
            ClientProfile::Default => &self.client,
            ClientProfile::LongDownload => &self.long_download,
            ClientProfile::GitHubApi => &self.github_api,
            ClientProfile::RuleSet => &self.rule_set,
            ClientProfile::ConnectivityProbe => &self.connectivity_probe,
        }
    }

    /// 获取（并按代理 URL 缓存）走指定代理的客户端。
    ///
    /// 用于「经内核 mixed 入站端口探测外网连通性」这类场景：代理地址随
    /// `proxy_port` 配置而变，但配置极少变化，因此按 URL 缓存即可命中。
    /// 缓存数量以上限保护，避免恶意/异常输入把 map 撑爆。
    pub fn client_for_proxy(
        &self,
        proxy_url: &str,
        timeout: Duration,
    ) -> Result<Arc<Client>, reqwest::Error> {
        // 上限：每个代理端口一个实例，正常运行只会有 1~2 个；
        // 触顶时整体清空重建（等价于换个 key 空间，不会无界增长）
        const MAX_CACHED_PROXY_CLIENTS: usize = 8;

        if let Some(client) = self
            .proxy_clients
            .read()
            .ok()
            .and_then(|map| map.get(proxy_url).cloned())
        {
            return Ok(client);
        }

        let client = Arc::new(
            Client::builder()
                .timeout(timeout)
                .connect_timeout(timeout.min(Duration::from_secs(3)))
                .proxy(reqwest::Proxy::all(proxy_url)?)
                .user_agent("sing-box-windows/connectivity-check")
                .build()?,
        );

        if let Ok(mut map) = self.proxy_clients.write() {
            if map.len() >= MAX_CACHED_PROXY_CLIENTS {
                map.clear();
            }
            map.insert(proxy_url.to_owned(), Arc::clone(&client));
        }

        Ok(client)
    }

    /// 下载文件到指定路径
    pub async fn download_file(
        &self,
        url: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures::StreamExt;
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;

        let mut file = File::create(file_path).await?;

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
        }

        file.flush().await?;

        Ok(())
    }

    /// 获取JSON数据
    pub async fn get_json<T>(&self, url: &str) -> Result<T, reqwest::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        let response = self.client.get(url).send().await?;
        response.error_for_status()?.json::<T>().await
    }

    /// 获取文本数据
    pub async fn get_text(&self, url: &str) -> Result<String, reqwest::Error> {
        let response = self.client.get(url).send().await?;
        response.error_for_status()?.text().await
    }

    /// 测试URL连通性（用于代理测试）
    pub async fn test_connectivity(
        &self,
        url: &str,
        proxy_url: Option<&str>,
    ) -> Result<Duration, reqwest::Error> {
        let start = std::time::Instant::now();

        let client: Arc<Client> = match proxy_url {
            // 按代理 URL 缓存，避免每次测速都重建（见 client_for_proxy）
            Some(proxy) => self.client_for_proxy(proxy, Duration::from_secs(5))?,
            None => Arc::new(self.proxy_client.clone()),
        };

        let response = client.get(url).send().await?;
        response.error_for_status()?;

        Ok(start.elapsed())
    }
}

// 全局单例
lazy_static::lazy_static! {
    /// 全局HTTP客户端管理器实例
    pub static ref HTTP_CLIENT_MANAGER: HttpClientManager = HttpClientManager::new_or_fallback();
}

/// 获取全局HTTP客户端
pub fn get_client() -> &'static Client {
    HTTP_CLIENT_MANAGER.get_client()
}

/// 获取代理测试客户端
pub fn get_proxy_client() -> &'static Client {
    HTTP_CLIENT_MANAGER.get_proxy_client()
}

/// 按档位获取客户端（见 [`ClientProfile`]）
pub fn profile_client(profile: ClientProfile) -> &'static Client {
    HTTP_CLIENT_MANAGER.profile(profile)
}

/// 获取走指定代理的缓存客户端（见 [`HttpClientManager::client_for_proxy`]）
pub fn proxy_client_for(
    proxy_url: &str,
    timeout: Duration,
) -> Result<Arc<Client>, reqwest::Error> {
    HTTP_CLIENT_MANAGER.client_for_proxy(proxy_url, timeout)
}

/// 便捷方法：下载文件
pub async fn download_file(
    url: &str,
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    HTTP_CLIENT_MANAGER.download_file(url, file_path).await
}

/// 便捷方法：获取JSON
pub async fn get_json<T>(url: &str) -> Result<T, reqwest::Error>
where
    T: serde::de::DeserializeOwned,
{
    HTTP_CLIENT_MANAGER.get_json(url).await
}

/// 便捷方法：获取文本
pub async fn get_text(url: &str) -> Result<String, reqwest::Error> {
    HTTP_CLIENT_MANAGER.get_text(url).await
}

/// 便捷方法：测试连通性
pub async fn test_connectivity(
    url: &str,
    proxy_url: Option<&str>,
) -> Result<Duration, reqwest::Error> {
    HTTP_CLIENT_MANAGER.test_connectivity(url, proxy_url).await
}
