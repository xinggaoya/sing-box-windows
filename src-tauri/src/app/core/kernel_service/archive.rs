//! 内核压缩包解压的统一实现
//!
//! ## 为什么要抽这个模块
//!
//! 此前「下载内核」与「导入内核」两条路径各自实现了一套几乎逐行相同的解压逻辑
//! （`download.rs` 与 `import.rs` 合计约 170 行重复），并且已经出现行为分叉：
//!
//! | 能力 | download.rs | import.rs |
//! |------|-------------|-----------|
//! | zip | ✅ | ✅ |
//! | tar.gz | ✅ | ✅ |
//! | tgz | ❌ 仅按 `.tar.gz` 判断 | ✅ |
//! | tar（未压缩） | ❌ | ✅ |
//! | 空文件校验 | ✅ | ❌ |
//! | 错误信息 | `Box<dyn Error>` | `String` |
//!
//! 也就是说：用户导入一个 `.tgz` 或 `.tar` 内核包能成功，但从「内核下载」走同格式会失败。
//! 现统一为一份实现，格式支持取两者的超集（zip / tar.gz / tgz / tar）。
//!
//! ## 为什么全部走 `spawn_blocking`
//!
//! 解压是纯 CPU + 文件 IO（`std::fs::File`、`std::io::copy`、`WalkDir`），
//! 之前的实现把它们声明成 `async fn` 却一行 `.await` 都没有，由 `#[tauri::command]`
//! 直接在 Tauri 的 tokio worker 上同步执行 —— 解压几十 MB 的内核包会**阻塞整个
//! 异步运行时**，期间其他 Tauri 命令（IPC、健康检查、事件转发）全部排队。
//! 现在统一把阻塞部分丢进 `spawn_blocking`。

use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("压缩文件不存在: {0:?}")]
    NotFound(PathBuf),

    #[error("压缩文件为空: {0:?}")]
    Empty(PathBuf),

    #[error("不支持的压缩格式（仅支持 zip / tar.gz / tgz / tar）: {0}")]
    UnsupportedFormat(String),

    #[error("解压失败 [{path}]: {message}")]
    Extract { path: String, message: String },

    #[error("解压任务异常终止: {0}")]
    Join(String),
}

type ArchiveResult<T> = Result<T, ArchiveError>;

/// 压缩包格式（按文件名后缀判定）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArchiveFormat {
    Zip,
    TarGz,
    Tar,
}

impl ArchiveFormat {
    /// 按文件名判定格式；无法识别返回 `None`。
    ///
    /// 用完整文件名后缀匹配（而非只取 `extension()`），因为 `.tar.gz` 的
    /// `extension()` 是 `gz`，单看扩展名无法与 `.tgz` 之外的 gzip 文件区分。
    fn detect(path: &Path) -> Option<Self> {
        let name = path.file_name()?.to_string_lossy().to_ascii_lowercase();
        if name.ends_with(".zip") {
            Some(Self::Zip)
        } else if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
            Some(Self::TarGz)
        } else if name.ends_with(".tar") {
            Some(Self::Tar)
        } else {
            None
        }
    }
}

/// 解压 `archive_path` 到 `extract_to`（目录不存在会自动创建）。
///
/// 阻塞 IO 部分在 `spawn_blocking` 中执行，不会阻塞异步运行时。
pub async fn extract(archive_path: &Path, extract_to: &Path) -> ArchiveResult<()> {
    let archive_path = archive_path.to_path_buf();
    let extract_to = extract_to.to_path_buf();

    tokio::task::spawn_blocking(move || extract_blocking(&archive_path, &extract_to))
        .await
        .map_err(|error| ArchiveError::Join(error.to_string()))?
}

/// 在指定目录树中查找名为 `executable_name` 的可执行文件。
///
/// 优先直接命中 `search_dir/executable_name`（内核发布包的常见布局），
/// 否则递归遍历；同名多个时取第一个（`WalkDir` 按字典序，结果稳定可预期）。
///
/// 阻塞 IO 部分在 `spawn_blocking` 中执行。
pub async fn find_executable(
    search_dir: &Path,
    executable_name: &str,
) -> ArchiveResult<PathBuf> {
    let search_dir = search_dir.to_path_buf();
    let executable_name = executable_name.to_owned();

    tokio::task::spawn_blocking(move || {
        let direct = search_dir.join(&executable_name);
        if direct.is_file() {
            info!("直接找到可执行文件: {:?}", direct);
            return Ok(direct);
        }

        for entry in walkdir::WalkDir::new(&search_dir)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name == executable_name)
            {
                debug!("递归找到可执行文件: {:?}", path);
                return Ok(path.to_path_buf());
            }
        }

        // 没找到时把目录内容打出来，便于用户/我们定位压缩包结构问题
        if let Ok(entries) = std::fs::read_dir(&search_dir) {
            warn!("未找到可执行文件，目录内容:");
            for entry in entries.flatten() {
                warn!("  - {:?}", entry.path());
            }
        }

        Err(ArchiveError::Extract {
            path: search_dir.to_string_lossy().to_string(),
            message: format!("未找到可执行文件: {}", executable_name),
        })
    })
    .await
    .map_err(|error| ArchiveError::Join(error.to_string()))?
}

/// 为解压出来的文件补上可执行权限（Windows 无意义，直接返回）。
pub fn set_executable_permission(file_path: &Path) -> ArchiveResult<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms =
            std::fs::metadata(file_path).map_err(|e| ArchiveError::Extract {
                path: file_path.to_string_lossy().to_string(),
                message: format!("读取文件权限失败: {}", e),
            })?
            .permissions();
        // 0o755：保留原有读权限，补齐执行位，避免把用户自建的更严格权限改宽
        perms.set_mode(perms.mode() | 0o755);
        std::fs::set_permissions(file_path, perms).map_err(|e| ArchiveError::Extract {
            path: file_path.to_string_lossy().to_string(),
            message: format!("设置执行权限失败: {}", e),
        })?;
        debug!("已设置执行权限: {:?}", file_path);
    }

    #[cfg(not(unix))]
    {
        let _ = file_path;
    }

    Ok(())
}

/// 阻塞实现：必须在 `spawn_blocking` 内调用。
fn extract_blocking(archive_path: &Path, extract_to: &Path) -> ArchiveResult<()> {
    if !archive_path.exists() {
        return Err(ArchiveError::NotFound(archive_path.to_path_buf()));
    }

    let metadata = std::fs::metadata(archive_path).map_err(|e| ArchiveError::Extract {
        path: archive_path.to_string_lossy().to_string(),
        message: format!("读取压缩文件属性失败: {}", e),
    })?;
    if metadata.len() == 0 {
        return Err(ArchiveError::Empty(archive_path.to_path_buf()));
    }

    let format = ArchiveFormat::detect(archive_path).ok_or_else(|| {
        ArchiveError::UnsupportedFormat(
            archive_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("<无扩展名>")
                .to_string(),
        )
    })?;

    std::fs::create_dir_all(extract_to).map_err(|e| ArchiveError::Extract {
        path: extract_to.to_string_lossy().to_string(),
        message: format!("创建解压目录失败: {}", e),
    })?;

    info!(
        "开始解压 {:?} ({:?}) 到 {:?}",
        archive_path, format, extract_to
    );

    match format {
        ArchiveFormat::Zip => extract_zip_blocking(archive_path, extract_to),
        ArchiveFormat::TarGz | ArchiveFormat::Tar => {
            extract_tar_blocking(archive_path, extract_to, format)
        }
    }
    .inspect(|_| {
        if let Ok(entries) = std::fs::read_dir(extract_to) {
            info!("解压后的文件:");
            for entry in entries.flatten() {
                info!("  - {:?}", entry.path());
            }
        }
    })
}

fn extract_zip_blocking(archive_path: &Path, extract_to: &Path) -> ArchiveResult<()> {
    use zip::ZipArchive;

    let file = std::fs::File::open(archive_path).map_err(|e| ArchiveError::Extract {
        path: archive_path.to_string_lossy().to_string(),
        message: format!("打开 ZIP 失败: {}", e),
    })?;
    let mut zip = ZipArchive::new(file).map_err(|e| ArchiveError::Extract {
        path: archive_path.to_string_lossy().to_string(),
        message: format!("解析 ZIP 失败: {}", e),
    })?;

    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|e| ArchiveError::Extract {
            path: archive_path.to_string_lossy().to_string(),
            message: format!("读取 ZIP 条目失败: {}", e),
        })?;
        let target = extract_to.join(entry.name());

        // ZIP 的目录项以 '/' 结尾；只建目录，不落文件
        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|e| ArchiveError::Extract {
                path: target.to_string_lossy().to_string(),
                message: format!("创建目录失败: {}", e),
            })?;
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ArchiveError::Extract {
                path: parent.to_string_lossy().to_string(),
                message: format!("创建父目录失败: {}", e),
            })?;
        }

        let mut out = std::fs::File::create(&target).map_err(|e| ArchiveError::Extract {
            path: target.to_string_lossy().to_string(),
            message: format!("写入文件失败: {}", e),
        })?;
        std::io::copy(&mut entry, &mut out).map_err(|e| ArchiveError::Extract {
            path: target.to_string_lossy().to_string(),
            message: format!("解压 ZIP 失败: {}", e),
        })?;
    }

    info!("✅ ZIP 解压完成");
    Ok(())
}

fn extract_tar_blocking(
    archive_path: &Path,
    extract_to: &Path,
    format: ArchiveFormat,
) -> ArchiveResult<()> {
    use tar::Archive;

    let open = |message: &str| -> ArchiveResult<std::fs::File> {
        std::fs::File::open(archive_path).map_err(|e| ArchiveError::Extract {
            path: archive_path.to_string_lossy().to_string(),
            message: format!("{}: {}", message, e),
        })
    };

    // `Archive` 的泛型随 reader 变化，因此按格式分两个分支，
    // 而不是在同一个变量上重新赋值（类型不一致）。
    match format {
        ArchiveFormat::TarGz => Archive::new(flate2::read::GzDecoder::new(open("打开 tar.gz 失败")?))
            .unpack(extract_to)
            .map_err(|e| ArchiveError::Extract {
                path: archive_path.to_string_lossy().to_string(),
                message: format!("解压 tar.gz 失败: {}", e),
            })?,
        _ => Archive::new(open("打开 tar 失败")?)
            .unpack(extract_to)
            .map_err(|e| ArchiveError::Extract {
                path: archive_path.to_string_lossy().to_string(),
                message: format!("解压 tar 失败: {}", e),
            })?,
    }

    info!("✅ tar 解压完成");
    Ok(())
}

#[cfg(test)]
#[path = "archive.tests.rs"]
mod tests;