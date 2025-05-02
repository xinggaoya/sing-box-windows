use crate::app::constants::registry;
use std::io;
use winreg::enums::*;
use winreg::RegKey;

pub fn disable_system_proxy() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_WRITE)?;

    // 禁用代理
    settings.set_value(registry::PROXY_ENABLE, &0u32)?;

    // 清空代理服务器地址
    settings.set_value(registry::PROXY_SERVER, &"")?;

    Ok(())
}

pub fn enable_system_proxy(host: &str, port: u16) -> io::Result<()> {
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
