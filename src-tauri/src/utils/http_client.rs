use reqwest::Client;
use std::time::Duration;

/// 全局 HTTP 客户端管理器
/// 提供高效的连接池和重用机制
pub struct HttpClientManager {
    client: Client,
    proxy_client: Client,
}

impl Default for HttpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClientManager {
    /// 创建新的 HTTP 客户端管理器
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .connect_timeout(Duration::from_secs(10))
            .no_proxy()
            .user_agent("sing-box-windows/1.0 (sing-box; compatible; Windows NT 10.0)")
            .build()
            .expect("创建HTTP客户端失败");

        // 用于代理测试的客户端，超时时间更短
        let proxy_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(5)
            .pool_idle_timeout(Duration::from_secs(60))
            .connect_timeout(Duration::from_secs(5))
            .no_proxy()
            .user_agent("sing-box-windows/1.0 (sing-box; compatible; Windows NT 10.0)")
            .build()
            .expect("创建代理测试HTTP客户端失败");

        Self {
            client,
            proxy_client,
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

    /// 下载文件到指定路径
    pub async fn download_file(
        &self,
        url: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use futures_util::StreamExt;
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

        let client = if let Some(proxy) = proxy_url {
            // 创建带代理的临时客户端
            Client::builder()
                .timeout(Duration::from_secs(5))
                .connect_timeout(Duration::from_secs(3))
                .proxy(reqwest::Proxy::all(proxy)?)
                .build()?
        } else {
            self.proxy_client.clone()
        };

        let response = client.get(url).send().await?;
        response.error_for_status()?;

        Ok(start.elapsed())
    }
}

// 全局单例
lazy_static::lazy_static! {
    /// 全局HTTP客户端管理器实例
    pub static ref HTTP_CLIENT_MANAGER: HttpClientManager = HttpClientManager::new();
}

/// 获取全局HTTP客户端
pub fn get_client() -> &'static Client {
    HTTP_CLIENT_MANAGER.get_client()
}

/// 获取代理测试客户端
pub fn get_proxy_client() -> &'static Client {
    HTTP_CLIENT_MANAGER.get_proxy_client()
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
