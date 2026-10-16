//! Native typing integrations and persistent preferences, independent of the desktop toolkit.
pub mod i18n;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
pub mod preferences;
pub mod session;
use accento_core::View;
use preferences::Preferences;
use std::sync::Arc;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::sync::mpsc;

pub trait DesktopUi: Send + Sync {
    fn picker(&self, view: Option<View>);
    fn failed(&self, message: String);
}
pub type Ui = Arc<dyn DesktopUi>;
pub enum Command {
    Configure(Preferences),
    Pause(bool),
    Stop,
}
/// Call before creating a GUI toolkit on X11.
pub fn initialize() {
    #[cfg(target_os = "linux")]
    unsafe {
        x11::xlib::XInitThreads();
    }
}
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn run(
    preferences: Preferences,
    commands: mpsc::Receiver<Command>,
    ui: Ui,
    ready: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    return linux::run(preferences, commands, ui, ready);
    #[cfg(target_os = "macos")]
    return macos::run(preferences, commands, ui, ready);
}
