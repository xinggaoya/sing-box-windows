use std::fs::File;
use std::path::Path;
use log::{error, info};
use zip::ZipArchive;
// 根据url下载文件到指定位置
pub async fn download_file(url: String, path: &str) -> Result<(), String> {
    let file_path = Path::new(path);
    info!("Downloading file from {} to {}", url, file_path.to_str().unwrap());

    let client = reqwest::Client::new();
    let response = client.get(&url).send().await;

    match response {
        Ok(response) => {
            let bytes = response.bytes().await;

            match bytes {
                Ok(bytes) => {
                    if let Err(e) = std::fs::create_dir_all(file_path.parent().unwrap()) {
                        error!("Error creating directory: {}", e);
                        return Err("Failed to create directory".to_string());
                    }

                    if let Err(e) = std::fs::write(&file_path, bytes) {
                        error!("Error writing file: {}", e);
                        return Err("Failed to write file".to_string());
                    }

                    info!("File downloaded successfully");
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to download file: {}", e);
                    Err("Failed to download file".to_string())
                }
            }
        }
        Err(e) => {
            error!("Failed to download file: {}", e);
            Err("Failed to download file".to_string())
        }
    }
}


pub async fn unzip_file(path: &str, to: &str) -> Result<(), String> {
    info!("从 {} 解压文件到 {}", path, to);

    // 打开ZIP文件
    let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;

    // 创建ZipArchive对象
    let mut archive = ZipArchive::new(file).map_err(|e| format!("读取归档失败: {}", e))?;

    // 遍历ZIP文件中的所有条目
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("读取文件失败: {}", e))?;

        // 获取文件名并去除前导路径
        let file_name = Path::new(file.name()).file_name().ok_or("获取文件名失败")?;
        let outpath = Path::new(to).join(file_name.to_str().unwrap());
        info!("正在解压文件: {}", outpath.display());

        // 如果是目录，则创建目录
        if file.is_dir() {
            std::fs::create_dir_all(&outpath).map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            // 创建之间的文件夹
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
            }
            let mut outfile = File::create(&outpath).map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| format!("复制文件失败: {}", e))?;
            info!("已解压文件: {}", outpath.display());
        }
    }

    // 删除zip
    std::fs::remove_file(path).map_err(|e| format!("删除文件失败: {}", e))?;
    info!("解压完成");
    Ok(())
}
