//! `archive` 模块的单元测试。
//!
//! 这段代码位于「内核安装」的关键路径上（下载/导入内核都要过这里），
//! 历史上两个调用方各有一份实现且格式支持不一致（`.tgz` / `.tar` 只在一侧支持），
//! 因此对四种格式与关键错误分支做全覆盖。

use super::{extract, find_executable, ArchiveError, ArchiveFormat};
use std::io::Write;
use std::path::{Path, PathBuf};

/// 为每个测试建独立临时目录（按测试名隔离，避免并行测试互相干扰）
fn temp_dir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "singbox-archive-test-{}-{}",
        name,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    dir
}

/// 造一个 zip：内含 `bin/sing-box` 与 `README.txt`
fn make_zip(target: &Path) {
    let file = std::fs::File::create(target).expect("创建 zip");
    let mut zip = zip::ZipWriter::new(file);
    let options: zip::write::FileOptions<'_, ()> =
        zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip.start_file("bin/sing-box", options).expect("写入目录项");
    zip.write_all(b"fake-kernel-binary").expect("写入文件内容");
    zip.start_file("README.txt", options).expect("写入 README");
    zip.write_all(b"readme").expect("写入 README 内容");
    zip.finish().expect("收尾 ZIP");
}

/// 造一个 tar（可 gzip 压缩），内含 `sing-box`
fn make_tar(target: &Path, gzip: bool) {
    let file = std::fs::File::create(target).expect("创建 tar");
    let mut builder = tar::Builder::new(file);
    let content = b"fake-kernel-binary";
    let mut header = tar::Header::new_gnu();
    header.set_size(content.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    builder
        .append_data(&mut header, "sing-box", content.as_slice())
        .expect("写入 tar 条目");
    builder.finish().expect("收尾 tar");

    if gzip {
        let raw = std::fs::read(target).expect("读回 tar");
        let mut encoder = flate2::write::GzEncoder::new(
            std::fs::File::create(target).expect("重建文件"),
            flate2::Compression::fast(),
        );
        encoder.write_all(&raw).expect("写入 gzip");
        encoder.finish().expect("收尾 gzip");
    }
}

// ---------------- 格式判定 ----------------

#[test]
fn detects_all_supported_formats() {
    assert_eq!(
        ArchiveFormat::detect(Path::new("a.zip")),
        Some(ArchiveFormat::Zip)
    );
    assert_eq!(
        ArchiveFormat::detect(Path::new("sing-box-linux-amd64.tar.gz")),
        Some(ArchiveFormat::TarGz)
    );
    assert_eq!(
        ArchiveFormat::detect(Path::new("sing-box-linux-amd64.tgz")),
        Some(ArchiveFormat::TarGz)
    );
    assert_eq!(
        ArchiveFormat::detect(Path::new("sing-box-linux-amd64.tar")),
        Some(ArchiveFormat::Tar)
    );
}

#[test]
fn rejects_unsupported_extension() {
    assert_eq!(ArchiveFormat::detect(Path::new("a.7z")), None);
    assert_eq!(ArchiveFormat::detect(Path::new("no_extension")), None);
}

#[test]
fn detection_is_case_insensitive() {
    assert_eq!(
        ArchiveFormat::detect(Path::new("KERNEL.ZIP")),
        Some(ArchiveFormat::Zip)
    );
    assert_eq!(
        ArchiveFormat::detect(Path::new("KERNEL.TAR.GZ")),
        Some(ArchiveFormat::TarGz)
    );
}

// ---------------- 解压 ----------------

#[tokio::test]
async fn extracts_zip_with_nested_paths() {
    let root = temp_dir("zip");
    let archive = root.join("kernel.zip");
    let out = root.join("out");
    make_zip(&archive);

    extract(&archive, &out).await.expect("ZIP 解压应成功");

    let kernel = out.join("bin").join("sing-box");
    assert_eq!(
        std::fs::read_to_string(&kernel).unwrap(),
        "fake-kernel-binary"
    );
    assert_eq!(
        std::fs::read_to_string(out.join("README.txt")).unwrap(),
        "readme"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn extracts_tar_gz() {
    let root = temp_dir("targz");
    let archive = root.join("kernel.tar.gz");
    let out = root.join("out");
    make_tar(&archive, true);

    extract(&archive, &out).await.expect("tar.gz 解压应成功");
    assert_eq!(
        std::fs::read_to_string(out.join("sing-box")).unwrap(),
        "fake-kernel-binary"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn extracts_tgz_short_extension() {
    // 回归：`.tgz` 曾只在「导入内核」路径被支持，「内核下载」路径会报不支持格式
    let root = temp_dir("tgz");
    let archive = root.join("kernel.tgz");
    let out = root.join("out");
    make_tar(&archive, true);

    extract(&archive, &out).await.expect(".tgz 解压应成功");
    assert!(out.join("sing-box").is_file());

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn extracts_plain_tar() {
    // 回归：未压缩 `.tar` 曾只在「导入内核」路径被支持
    let root = temp_dir("tar");
    let archive = root.join("kernel.tar");
    let out = root.join("out");
    make_tar(&archive, false);

    extract(&archive, &out).await.expect(".tar 解压应成功");
    assert!(out.join("sing-box").is_file());

    let _ = std::fs::remove_dir_all(&root);
}

// ---------------- 错误分支 ----------------

#[tokio::test]
async fn errors_on_missing_archive() {
    let root = temp_dir("missing");
    let out = root.join("out");

    let error = extract(&root.join("nope.zip"), &out).await.unwrap_err();
    assert!(
        matches!(error, ArchiveError::NotFound(_)),
        "实际: {}",
        error
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn errors_on_empty_archive() {
    // 回归：空文件校验原先只在 download.rs 侧存在
    let root = temp_dir("empty");
    let archive = root.join("empty.zip");
    std::fs::write(&archive, b"").expect("写空文件");
    let out = root.join("out");

    let error = extract(&archive, &out).await.unwrap_err();
    assert!(matches!(error, ArchiveError::Empty(_)), "实际: {}", error);

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn errors_on_unsupported_format() {
    let root = temp_dir("unsupported");
    let archive = root.join("kernel.7z");
    std::fs::write(&archive, b"whatever").expect("写占位文件");
    let out = root.join("out");

    let error = extract(&archive, &out).await.unwrap_err();
    assert!(
        matches!(error, ArchiveError::UnsupportedFormat(_)),
        "实际: {}",
        error
    );

    let _ = std::fs::remove_dir_all(&root);
}

// ---------------- 可执行文件查找 ----------------

#[tokio::test]
async fn finds_executable_at_top_level_directly() {
    let root = temp_dir("exec-direct");
    std::fs::write(root.join("sing-box"), b"x").expect("写内核");

    let found = find_executable(&root, "sing-box").await.expect("应找到");
    assert_eq!(found, root.join("sing-box"));

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn finds_executable_recursively() {
    let root = temp_dir("exec-nested");
    let nested = root.join("sing-box-1.14.0-linux-amd64");
    std::fs::create_dir_all(&nested).expect("建目录");
    std::fs::write(nested.join("sing-box"), b"x").expect("写内核");

    let found = find_executable(&root, "sing-box")
        .await
        .expect("应递归找到");
    assert_eq!(found, nested.join("sing-box"));

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn errors_when_executable_absent() {
    let root = temp_dir("exec-missing");

    let error = find_executable(&root, "sing-box").await.unwrap_err();
    assert!(
        matches!(error, ArchiveError::Extract { .. }),
        "实际: {}",
        error
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn ignores_directory_named_like_executable() {
    // 目录与可执行文件同名时不能误判为命中
    let root = temp_dir("exec-dir-clash");
    std::fs::create_dir_all(root.join("sing-box")).expect("建同名目录");

    let error = find_executable(&root, "sing-box").await.unwrap_err();
    assert!(
        matches!(error, ArchiveError::Extract { .. }),
        "实际: {}",
        error
    );

    let _ = std::fs::remove_dir_all(&root);
}
