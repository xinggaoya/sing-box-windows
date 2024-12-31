use log::{error, info};
use std::fs::File;
use std::path::Path;
use zip::ZipArchive;
use futures_util::StreamExt;
use std::io::Write;

// 根据url下载文件到指定位置
pub async fn download_file<F>(url: String, path: &str, progress_callback: F) -> Result<(), String>
where
    F: Fn(u32) + Send + 'static,
{
    let file_path = Path::new(path);
    info!(
        "开始下载文件: {} -> {}",
        url,
        file_path.to_str().unwrap()
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("服务器返回错误状态码: {}", response.status()));
    }

    let total_size = response
        .content_length()
        .ok_or_else(|| "无法获取文件大小".to_string())?;

    // 创建目录
    if let Some(parent) = file_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            error!("创建目录失败: {}", e);
            return Err(format!("创建目录失败: {}", e));
        }
    }

    // 创建临时文件
    let temp_path = file_path.with_extension("tmp");
    let mut file = File::create(&temp_path)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;

    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    let mut last_percent = 0u32;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载过程中出错: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        
        downloaded += chunk.len() as u64;
        let percent = (downloaded as f64 / total_size as f64 * 100.0) as u32;
        
        // 每当进度变化时通知回调
        if percent != last_percent {
            progress_callback(percent);
            last_percent = percent;
        }
    }

    // 下载完成后，将临时文件重命名为目标文件
    if let Err(e) = std::fs::rename(&temp_path, file_path) {
        error!("重命名文件失败: {}", e);
        // 如果重命名失败，尝试删除临时文件
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!("重命名文件失败: {}", e));
    }

    info!("文件下载完成");
    Ok(())
}

pub async fn unzip_file(path: &str, to: &str) -> Result<(), String> {
    info!("开始解压文件: {} -> {}", path, to);

    // 打开ZIP文件
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            error!("打开文件失败: {}", e);
            return Err(format!("打开文件失败: {}", e));
        }
    };

    // 创建ZipArchive对象
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            error!("读取归档失败: {}", e);
            return Err(format!("读取归档失败: {}", e));
        }
    };

    // 确保目标目录存在
    if let Err(e) = std::fs::create_dir_all(to) {
        error!("创建目标目录失败: {}", e);
        return Err(format!("创建目标目录失败: {}", e));
    }

    // 遍历ZIP文件中的所有条目
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取文件失败: {}", e))?;

        // 获取文件名并去除前导路径
        let file_name = match Path::new(file.name()).file_name() {
            Some(name) => name,
            None => {
                error!("无效的文件名: {}", file.name());
                continue;
            }
        };

        let outpath = Path::new(to).join(file_name);
        info!("正在解压: {}", outpath.display());

        if file.is_dir() {
            std::fs::create_dir_all(&outpath)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("创建父目录失败: {}", e))?;
                }
            }

            let mut outfile = File::create(&outpath)
                .map_err(|e| format!("创建文件失败: {}", e))?;

            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("复制文件失败: {}", e))?;

            // 设置可执行权限（仅在类Unix系统上）
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if outpath.extension().map_or(true, |ext| ext == "") {
                    if let Ok(metadata) = outfile.metadata() {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        if let Err(e) = std::fs::set_permissions(&outpath, perms) {
                            error!("设置执行权限失败: {}", e);
                        }
                    }
                }
            }
        }
    }

    // 删除zip文件
    if let Err(e) = std::fs::remove_file(path) {
        error!("删除zip文件失败: {}", e);
        return Err(format!("删除zip文件失败: {}", e));
    }

    info!("解压完成");
    Ok(())
}
