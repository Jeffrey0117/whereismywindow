use windows::core::HSTRING;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegSetValueExW, HKEY, HKEY_CURRENT_USER,
    KEY_WRITE, REG_SZ,
};

const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "WhereIsMyWindow";

pub fn set_auto_start(enabled: bool) {
    let mut hkey = HKEY::default();
    let key_str = HSTRING::from(RUN_KEY);
    let result = unsafe {
        RegOpenKeyExW(HKEY_CURRENT_USER, &key_str, Some(0), KEY_WRITE, &mut hkey)
    };
    if result.is_err() {
        log::warn!("Failed to open registry Run key: {:?}", result);
        return;
    }

    let value_name = HSTRING::from(VALUE_NAME);

    if enabled {
        let exe_path = match std::env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                log::warn!("Failed to get exe path: {}", e);
                let _ = unsafe { RegCloseKey(hkey) };
                return;
            }
        };

        let path_str = exe_path.to_string_lossy().to_string();
        let wide: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
        let data = unsafe {
            std::slice::from_raw_parts(wide.as_ptr() as *const u8, wide.len() * 2)
        };

        let result = unsafe {
            RegSetValueExW(hkey, &value_name, Some(0), REG_SZ, Some(data))
        };
        if result.is_ok() {
            log::info!("Auto-start enabled: {}", path_str);
        } else {
            log::warn!("Failed to set auto-start registry value: {:?}", result);
        }
    } else {
        let result = unsafe { RegDeleteValueW(hkey, &value_name) };
        if result.is_ok() {
            log::info!("Auto-start disabled");
        } else {
            log::warn!("Failed to remove auto-start registry value: {:?}", result);
        }
    }

    let _ = unsafe { RegCloseKey(hkey) };
}

pub fn is_auto_start() -> bool {
    use windows::Win32::System::Registry::{RegQueryValueExW, KEY_READ};

    let mut hkey = HKEY::default();
    let key_str = HSTRING::from(RUN_KEY);
    let result = unsafe {
        RegOpenKeyExW(HKEY_CURRENT_USER, &key_str, Some(0), KEY_READ, &mut hkey)
    };
    if result.is_err() {
        return false;
    }

    let value_name = HSTRING::from(VALUE_NAME);
    let result = unsafe {
        RegQueryValueExW(hkey, &value_name, None, None, None, None)
    };
    let _ = unsafe { RegCloseKey(hkey) };
    result.is_ok()
}
