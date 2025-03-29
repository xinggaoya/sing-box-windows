use std::io;
use winreg::enums::*;
use winreg::RegKey;
use crate::app::constants::registry;

pub fn disable_system_proxy() -> io::Result<()> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu.open_subkey_with_flags(registry::INTERNET_SETTINGS, KEY_WRITE)?;

    // 禁用代理
    settings.set_value(registry::PROXY_ENABLE, &0u32)?;

    // 清空代理服务器地址
    settings.set_value(registry::PROXY_SERVER, &"")?;

    // 通知系统代理设置已更改
    unsafe {
        winapi::um::wininet::InternetSetOptionW(
            std::ptr::null_mut(),
            winapi::um::wininet::INTERNET_OPTION_SETTINGS_CHANGED,
            std::ptr::null_mut(),
            0,
        );
        winapi::um::wininet::InternetSetOptionW(
            std::ptr::null_mut(),
            winapi::um::wininet::INTERNET_OPTION_REFRESH,
            std::ptr::null_mut(),
            0,
        );
    }

    Ok(())
}
