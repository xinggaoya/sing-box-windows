use futures_util::StreamExt;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::{error, info};
use zip::ZipArchive;
use crate::app::constants::{messages, network};

// 根据url下载文件到指定位置
pub async fn download_file<F>(url: String, path: &str, progress_callback: F) -> Result<(), String>
where
    F: Fn(u32) + Send + 'static,
{
    let file_path = Path::new(path);
    info!("{}: {} -> {}", messages::INFO_DOWNLOAD_STARTED, url, file_path.to_str().unwrap());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(network::HTTP_TIMEOUT_SECONDS))
        .build()
        .map_err(|e| format!("{}: {}", messages::ERR_HTTP_CLIENT_FAILED, e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_REQUEST_FAILED, e))?;

    if !response.status().is_success() {
        return Err(format!("{}: {}", messages::ERR_SERVER_ERROR, response.status()));
    }

    let total_size = response
        .content_length()
        .ok_or_else(|| messages::ERR_FILE_SIZE_UNKNOWN.to_string())?;

    // 创建目录
    if let Some(parent) = file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
            return Err(format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e));
        }
    }

    // 创建临时文件
    let temp_path = file_path.with_extension("tmp");
    let mut file = File::create(&temp_path).map_err(|e| format!("{}: {}", messages::ERR_CREATE_FILE_FAILED, e))?;

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let mut last_percent = 0u32;

    // 开始下载
    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("{}: {}", messages::ERR_REQUEST_FAILED, e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("{}: {}", messages::ERR_WRITE_FILE_FAILED, e))?;

        downloaded += chunk.len() as u64;
        let percent = ((downloaded as f64 / total_size as f64) * 100.0) as u32;

        // 只在进度变化时回调
        if percent != last_percent {
            last_percent = percent;
            progress_callback(percent);
        }
    }

    // 完成下载，重命名临时文件
    std::fs::rename(&temp_path, &file_path)
        .map_err(|e| format!("{}: {}", messages::ERR_WRITE_FILE_FAILED, e))?;

    Ok(())
}

pub async fn unzip_file(path: &str, to: &str) -> Result<(), String> {
    info!("{}: {} -> {}", messages::INFO_UNZIP_STARTED, path, to);

    // 打开ZIP文件
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("{}: {}", messages::ERR_OPEN_FILE_FAILED, e);
            return Err(format!("{}: {}", messages::ERR_OPEN_FILE_FAILED, e));
        }
    };

    // 创建ZipArchive对象
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            error!("{}: {}", messages::ERR_READ_ARCHIVE_FAILED, e);
            return Err(format!("{}: {}", messages::ERR_READ_ARCHIVE_FAILED, e));
        }
    };

    // 确保目标目录存在
    if let Err(e) = std::fs::create_dir_all(to) {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
        return Err(format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e));
    }

    // 遍历ZIP文件中的所有条目
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("{}: {}", messages::ERR_EXTRACT_FILE_FAILED, e))?;

        // 获取文件名并去除前导路径
        let file_name = match Path::new(file.name()).file_name() {
            Some(name) => name,
            None => {
                error!("{}: {}", messages::ERR_INVALID_FILENAME, file.name());
                continue;
            }
        };

        let outpath = Path::new(to).join(file_name);
        info!("{}: {}", messages::INFO_EXTRACTING_FILE, outpath.display());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e))?;
        } else {
            // 创建文件父目录
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent).map_err(|e| format!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e))?;
                }
            }
            
            // 创建文件并写入内容
            let mut outfile = File::create(&outpath).map_err(|e| format!("{}: {}", messages::ERR_CREATE_FILE_FAILED, e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("{}: {}", messages::ERR_WRITE_FILE_FAILED, e))?;
        }
    }

    Ok(())
}

// 从代理下载，失败后尝试直接下载
pub async fn download_with_fallback<F>(
    original_url: &str, 
    path: &str, 
    progress_callback: F
) -> Result<(), String>
where
    F: Fn(u32) + Send + Clone + 'static,
{
    // 首先尝试通过代理下载 https://gh-proxy.com/https://github.com/...
    let proxy_url = format!("https://gh-proxy.com/{}", original_url);
    info!("尝试通过代理下载: {}", proxy_url);
    
    match download_file(proxy_url, path, progress_callback.clone()).await {
        Ok(_) => {
            info!("通过代理下载成功");
            Ok(())
        },
        Err(e) => {
            info!("代理下载失败: {}，尝试直接下载", e);
            // 代理下载失败，尝试直接下载
            download_file(original_url.to_string(), path, progress_callback).await
        }
    }
}
