use std::io;

#[cfg(target_os = "windows")]
use crate::app::constants::registry;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

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

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Ok(()) // 其他平台暂时不执行任何操作
    }
}

/// 启用系统代理 (跨平台实现)
pub fn enable_system_proxy(host: &str, port: u16) -> io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        enable_system_proxy_windows(host, port)
    }

    #[cfg(target_os = "linux")]
    {
        enable_system_proxy_linux(host, port)
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
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

    // 尝试使用gsettings重置代理设置 (如果可用)
    if let Ok(_) = std::process::Command::new("which").arg("gsettings").output() {
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.http", "host", "''"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.http", "port", "0"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.https", "host", "''"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.https", "port", "0"])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "'none'"])
            .output();
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn enable_system_proxy_windows(host: &str, port: u16) -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_WRITE)?;

    // 设置代理服务器地址
    let proxy_server = format!("{}:{}", host, port);
    settings.set_value(registry::PROXY_SERVER, &proxy_server)?;

    // 启用代理
    settings.set_value(registry::PROXY_ENABLE, &1u32)?;

    // 设置绕过本地地址
    settings.set_value(registry::PROXY_OVERRIDE, &"localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*")?;

    Ok(())
}

#[cfg(target_os = "linux")]
fn enable_system_proxy_linux(host: &str, port: u16) -> io::Result<()> {
    let proxy_url = format!("http://{}:{}", host, port);
    let proxy_url_secure = format!("https://{}:{}", host, port);
    let no_proxy = "localhost,127.0.0.1,::1,10.*,172.16.*,172.17.*,172.18.*,172.19.*,172.20.*,172.21.*,172.22.*,172.23.*,172.24.*,172.25.*,172.26.*,172.27.*,172.28.*,172.29.*,172.30.*,172.31.*,192.168.*";

    // 设置环境变量
    std::env::set_var("http_proxy", &proxy_url);
    std::env::set_var("https_proxy", &proxy_url_secure);
    std::env::set_var("HTTP_PROXY", &proxy_url);
    std::env::set_var("HTTPS_PROXY", &proxy_url_secure);
    std::env::set_var("all_proxy", &proxy_url);
    std::env::set_var("ALL_PROXY", &proxy_url);
    std::env::set_var("no_proxy", no_proxy);
    std::env::set_var("NO_PROXY", no_proxy);

    // 尝试使用gsettings设置代理 (如果可用)
    if let Ok(_) = std::process::Command::new("which").arg("gsettings").output() {
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.http", "host", host])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.http", "port", &port.to_string()])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.https", "host", host])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.https", "port", &port.to_string()])
            .output();
        let _ = std::process::Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "'manual'"])
            .output();
    }

    Ok(())
}
