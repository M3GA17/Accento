#[derive(serde::Serialize)]
pub struct PlatformInfo {
    os: &'static str,
    executable_example: &'static str,
}
#[tauri::command]
pub fn get_platform() -> PlatformInfo {
    PlatformInfo {
        os: std::env::consts::OS,
        executable_example: if cfg!(windows) {
            "notepad.exe"
        } else if cfg!(target_os = "macos") {
            "Safari"
        } else {
            "firefox"
        },
    }
}
