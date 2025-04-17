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
