#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod glass;
#[cfg(windows)]
mod input;
mod picker_material;
mod preferences;
#[cfg(windows)]
mod render;

fn main() {
    #[cfg(windows)]
    if let Err(error) = app::run() {
        if std::env::args().any(|arg| arg == "--self-test") {
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
    #[cfg(not(windows))]
    eprintln!(
        "Accento richiede Windows 11. Il motore accento-core è indipendente dalla piattaforma."
    );
}
