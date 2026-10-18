#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod desktop;
#[cfg(windows)]
mod glass;
use accento_platform::{i18n, preferences};
#[cfg(windows)]
mod input;
#[cfg(windows)]
mod picker_material;
mod platform_info;
#[cfg(any(target_os = "linux", target_os = "macos"))]
mod portable;
#[cfg(windows)]
mod render;

fn main() {
    if std::env::args().any(|arg| arg == "--diagnostics") {
        println!(
            "{}",
            serde_json::to_string_pretty(&platform_info::get_platform())
                .expect("Cannot serialize platform diagnostics")
        );
        return;
    }
    #[cfg(windows)]
    if let Err(error) =
        if std::env::args().any(|arg| matches!(arg.as_str(), "--self-test" | "--ui-test")) {
            app::run()
        } else {
            desktop::run()
        }
    {
        if std::env::args().any(|arg| matches!(arg.as_str(), "--self-test" | "--ui-test")) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        use windows::{Win32::UI::WindowsAndMessaging::*, core::PCWSTR};
        let message: Vec<u16> = format!("Accento non può avviarsi.\n\n{error}")
            .encode_utf16()
            .chain(Some(0))
            .collect();
        unsafe {
            MessageBoxW(
                None,
                PCWSTR(message.as_ptr()),
                windows::core::w!("Accento"),
                MB_OK | MB_ICONERROR,
            );
        }
    }
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    if let Err(error) = portable::run() {
        eprintln!("Accento: {error}");
        std::process::exit(1);
    }
}
