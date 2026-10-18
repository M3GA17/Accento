#[derive(serde::Serialize)]
pub struct PlatformInfo {
    os: &'static str,
    architecture: &'static str,
    version: &'static str,
    executable_example: &'static str,
    #[cfg(target_os = "macos")]
    permissions: accento_platform::PermissionStatus,
}
#[tauri::command]
pub fn get_platform() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        version: env!("CARGO_PKG_VERSION"),
        #[cfg(target_os = "macos")]
        permissions: accento_platform::permission_status(),
        executable_example: if cfg!(windows) {
            "notepad.exe"
        } else if cfg!(target_os = "macos") {
            "Safari"
        } else {
            "firefox"
        },
    }
}
