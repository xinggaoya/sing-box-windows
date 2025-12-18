use std::io;

#[cfg(target_os = "windows")]
use crate::app::constants::registry;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

/// 默认的系统代理绕过列表
pub const DEFAULT_BYPASS_LIST: &str =
    "localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;\
172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*";

fn parse_bypass_entries(raw: Option<&str>) -> Vec<String> {
    let source = raw
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(DEFAULT_BYPASS_LIST);

    source
        .split([';', ',', '\n'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// 禁用系统代理 (跨平台实现)
pub fn disable_system_proxy() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        disable_system_proxy_windows()
    }

    #[cfg(target_os = "linux")]
    {
        disable_system_proxy_linux()
    }

    #[cfg(target_os = "macos")]
    {
        disable_system_proxy_macos()
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(()) // 其他平台暂时不执行任何操作
    }
}

/// 启用系统代理 (跨平台实现)
pub fn enable_system_proxy(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        enable_system_proxy_windows(host, port, bypass)
    }

    #[cfg(target_os = "linux")]
    {
        enable_system_proxy_linux(host, port, bypass)
    }

    #[cfg(target_os = "macos")]
    {
        enable_system_proxy_macos(host, port, bypass)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Ok(()) // 其他平台暂时不执行任何操作
    }
}

#[cfg(target_os = "windows")]
fn disable_system_proxy_windows() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_WRITE)?;

    // 禁用代理
    settings.set_value(registry::PROXY_ENABLE, &0u32)?;

    // 清空代理服务器地址
    settings.set_value(registry::PROXY_SERVER, &"")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn disable_system_proxy_linux() -> io::Result<()> {
    // Linux下的系统代理设置通常通过环境变量
    // 这里可以尝试使用gsettings或者直接设置环境变量
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    std::env::remove_var("all_proxy");
    std::env::remove_var("ALL_PROXY");
    std::env::remove_var("no_proxy");
    std::env::remove_var("NO_PROXY");

    // 尝试使用gsettings重置代理设置 (GNOME/Unity/XFCE等)
    if std::process::Command::new("which")
        .arg("gsettings")
        .output()
        .is_ok()
    {
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.http", "host", "''"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.http", "port", "0"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.https", "host", "''"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.https", "port", "0"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", "'none'"])
            .output();
    }

    // 尝试使用kwriteconfig5/6重置代理设置 (KDE Plasma)
    for kwriteconfig in &["kwriteconfig6", "kwriteconfig5"] {
        if std::process::Command::new("which")
            .arg(kwriteconfig)
            .output()
            .is_ok()
        {
            // 设置代理模式为无代理 (0)
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "ProxyType",
                    "0",
                ])
                .output();

            // 通知KDE配置已更改
            let _ = std::process::Command::new("dbus-send")
                .args([
                    "--type=signal",
                    "/KIO/Scheduler",
                    "org.kde.KIO.Scheduler.reparseSlaveConfiguration",
                    "string:''",
                ])
                .output();
            break;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn disable_system_proxy_macos() -> io::Result<()> {
    // macOS使用networksetup命令来管理系统代理设置
    // 获取所有网络服务
    let output = std::process::Command::new("networksetup")
        .args(["-listallnetworkservices"])
        .output()?;

    if output.status.success() {
        let services = String::from_utf8_lossy(&output.stdout);

        // 跳过第一行（标题行），处理每个网络服务
        for line in services.lines().skip(1) {
            let service = line.trim();
            if !service.is_empty() && service != "*" {
                // 禁用HTTP代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setwebproxystate", service, "off"])
                    .output();

                // 禁用HTTPS代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setsecurewebproxystate", service, "off"])
                    .output();

                // 禁用SOCKS代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setsocksfirewallproxystate", service, "off"])
                    .output();
            }
        }
    }

    // 同时清除环境变量
    std::env::remove_var("http_proxy");
    std::env::remove_var("https_proxy");
    std::env::remove_var("HTTP_PROXY");
    std::env::remove_var("HTTPS_PROXY");
    std::env::remove_var("all_proxy");
    std::env::remove_var("ALL_PROXY");

    Ok(())
}

#[cfg(target_os = "windows")]
fn enable_system_proxy_windows(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_WRITE)?;

    // 设置代理服务器地址
    let proxy_server = format!("{}:{}", host, port);
    settings.set_value(registry::PROXY_SERVER, &proxy_server)?;

    // 启用代理
    settings.set_value(registry::PROXY_ENABLE, &1u32)?;

    // 设置绕过本地地址
    let entries = parse_bypass_entries(bypass);
    let override_value = if entries.is_empty() {
        DEFAULT_BYPASS_LIST.to_string()
    } else {
        entries.join(";")
    };
    settings.set_value(registry::PROXY_OVERRIDE, &override_value)?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn enable_system_proxy_linux(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    let proxy_url = format!("http://{}:{}", host, port);
    let proxy_url_secure = format!("https://{}:{}", host, port);
    let entries = parse_bypass_entries(bypass);
    let no_proxy = if entries.is_empty() {
        DEFAULT_BYPASS_LIST.replace(';', ",")
    } else {
        entries.join(",")
    };

    // 设置环境变量
    std::env::set_var("http_proxy", &proxy_url);
    std::env::set_var("https_proxy", &proxy_url_secure);
    std::env::set_var("HTTP_PROXY", &proxy_url);
    std::env::set_var("HTTPS_PROXY", &proxy_url_secure);
    std::env::set_var("all_proxy", &proxy_url);
    std::env::set_var("ALL_PROXY", &proxy_url);
    std::env::set_var("no_proxy", &no_proxy);
    std::env::set_var("NO_PROXY", &no_proxy);

    // 尝试使用gsettings设置代理 (GNOME/Unity/XFCE等)
    if std::process::Command::new("which")
        .arg("gsettings")
        .output()
        .is_ok()
    {
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.http", "host", host])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args([
                "set",
                "org.gnome.system.proxy.http",
                "port",
                &port.to_string(),
            ])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy.https", "host", host])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args([
                "set",
                "org.gnome.system.proxy.https",
                "port",
                &port.to_string(),
            ])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(["set", "org.gnome.system.proxy", "mode", "'manual'"])
            .output();
    }

    // 尝试使用kwriteconfig5/6设置代理 (KDE Plasma)
    for kwriteconfig in &["kwriteconfig6", "kwriteconfig5"] {
        if std::process::Command::new("which")
            .arg(kwriteconfig)
            .output()
            .is_ok()
        {
            let proxy_url = format!("http://{}:{}", host, port);

            // 设置HTTP代理
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "httpProxy",
                    &proxy_url,
                ])
                .output();

            // 设置HTTPS代理
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "httpsProxy",
                    &proxy_url,
                ])
                .output();

            // 设置代理模式为手动 (1)
            let _ = std::process::Command::new(kwriteconfig)
                .args([
                    "--file",
                    "kioslaverc",
                    "--group",
                    "Proxy Settings",
                    "--key",
                    "ProxyType",
                    "1",
                ])
                .output();

            // 通知KDE配置已更改
            let _ = std::process::Command::new("dbus-send")
                .args([
                    "--type=signal",
                    "/KIO/Scheduler",
                    "org.kde.KIO.Scheduler.reparseSlaveConfiguration",
                    "string:''",
                ])
                .output();
            break;
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn enable_system_proxy_macos(host: &str, port: u16, bypass: Option<&str>) -> io::Result<()> {
    // 获取所有网络服务
    let output = std::process::Command::new("networksetup")
        .args(["-listallnetworkservices"])
        .output()?;

    if output.status.success() {
        let services = String::from_utf8_lossy(&output.stdout);
        let entries = parse_bypass_entries(bypass);

        // 跳过第一行（标题行），处理每个网络服务
        for line in services.lines().skip(1) {
            let service = line.trim();
            if !service.is_empty() && service != "*" {
                // 设置HTTP代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setwebproxy", service, host, &port.to_string()])
                    .output();

                // 启用HTTP代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setwebproxystate", service, "on"])
                    .output();

                // 设置HTTPS代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setsecurewebproxy", service, host, &port.to_string()])
                    .output();

                // 启用HTTPS代理
                let _ = std::process::Command::new("networksetup")
                    .args(["-setsecurewebproxystate", service, "on"])
                    .output();

                // 设置代理绕过列表
                if !entries.is_empty() {
                    let mut cmd = std::process::Command::new("networksetup");
                    cmd.args(["-setproxybypassdomains", service]);
                    for entry in &entries {
                        cmd.arg(entry);
                    }
                    let _ = cmd.output();
                }
            }
        }
    }

    // 同时设置环境变量
    let proxy_url = format!("http://{}:{}", host, port);
    let proxy_url_secure = format!("https://{}:{}", host, port);

    std::env::set_var("http_proxy", &proxy_url);
    std::env::set_var("https_proxy", &proxy_url_secure);
    std::env::set_var("HTTP_PROXY", &proxy_url);
    std::env::set_var("HTTPS_PROXY", &proxy_url_secure);
    std::env::set_var("all_proxy", &proxy_url);
    std::env::set_var("ALL_PROXY", &proxy_url);

    Ok(())
}
