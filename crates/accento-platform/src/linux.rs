//! XInput2 physical-device grabs, with XTEST forwarding through an ungrabbed virtual device.
//! No evdev access, uinput device, clipboard replacement or elevated process is required.
use super::{Command, Ui};
use crate::{
    preferences::Preferences,
    session::{Host, NativeKey, Session, Target},
};
use accento_core::{Context, Key, View};
use enigo::{Enigo, Keyboard, Settings};
use std::{
    collections::{HashMap, HashSet},
    ffi::CStr,
    ptr,
    sync::mpsc,
    time::Instant,
};
use x11::{xinput2 as xi, xlib as x, xtest};
use x11rb::{
    protocol::xproto::{AtomEnum, ConnectionExt},
    rust_connection::RustConnection,
};

struct Native {
    display: *mut x::Display,
    root: x::Window,
    extension: i32,
    devices: HashSet<i32>,
    held: HashSet<u32>,
    pressed: HashMap<u32, (u32, u64)>,
    context: RustConnection,
    pid_atom: u32,
    enigo: Enigo,
    ui: Ui,
}
impl Native {
    fn open(ui: Ui) -> Result<Self, String> {
        if std::env::var("XDG_SESSION_TYPE").is_ok_and(|s| s == "wayland")
            || std::env::var_os("WAYLAND_DISPLAY").is_some()
        {
            return Err("Global typing is currently available in an X11 session. Wayland requires a compositor input-method integration; XWayland alone cannot control native Wayland apps. Settings remain available.".into());
        }
        let (context, _) =
            x11rb::connect(None).map_err(|e| format!("Cannot connect to X11: {e}"))?;
        let pid_atom = context
            .intern_atom(false, b"_NET_WM_PID")
            .map_err(|e| e.to_string())?
            .reply()
            .map_err(|e| e.to_string())?
            .atom;
        let enigo = Enigo::new(&Settings::default())
            .map_err(|e| format!("Cannot initialize X11 text input: {e}"))?;
        unsafe {
            let display = x::XOpenDisplay(ptr::null());
            if display.is_null() {
                return Err(
                    "DISPLAY is unavailable. Start Accento inside an X11 desktop session.".into(),
                );
            }
            let mut native = Self {
                display,
                root: x::XDefaultRootWindow(display),
                extension: 0,
                devices: HashSet::new(),
                held: HashSet::new(),
                pressed: HashMap::new(),
                context,
                pid_atom,
                enigo,
                ui,
            };
            let (mut event, mut error) = (0, 0);
            if x::XQueryExtension(
                display,
                c"XInputExtension".as_ptr(),
                &mut native.extension,
                &mut event,
                &mut error,
            ) == 0
            {
                return Err("The X server does not provide XInput2.".into());
            }
            let (mut major, mut minor) = (2, 0);
            if xi::XIQueryVersion(display, &mut major, &mut minor) != x::Success as i32 {
                return Err("XInput2 2.0 or newer is required.".into());
            }
            let mut mask = [0u8; 4];
            xi::XISetMask(&mut mask, xi::XI_HierarchyChanged);
            xi::XISetMask(&mut mask, xi::XI_RawButtonPress);
            let mut events = xi::XIEventMask {
                deviceid: xi::XIAllDevices,
                mask_len: mask.len() as i32,
                mask: mask.as_mut_ptr(),
            };
            xi::XISelectEvents(display, native.root, &mut events, 1);
            native.grab_devices()?;
            x::XSync(display, x::False);
            Ok(native)
        }
    }
    unsafe fn grab_devices(&mut self) -> Result<(), String> {
        unsafe {
            let mut count = 0;
            let infos = xi::XIQueryDevice(self.display, xi::XIAllDevices, &mut count);
            if infos.is_null() {
                return Err("Cannot enumerate X11 keyboard devices.".into());
            }
            let candidates = std::slice::from_raw_parts(infos, count as usize)
                .iter()
                .filter(|info| info._use == xi::XISlaveKeyboard && info.enabled != 0)
                .filter(|info| {
                    !CStr::from_ptr(info.name)
                        .to_string_lossy()
                        .contains("XTEST")
                })
                .map(|info| info.deviceid)
                .collect::<Vec<_>>();
            xi::XIFreeDeviceInfo(infos);
            self.devices.retain(|id| candidates.contains(id));
            for id in candidates {
                if self.devices.contains(&id) {
                    continue;
                }
                let mut mask = [0u8; 4];
                xi::XISetMask(&mut mask, xi::XI_KeyPress);
                xi::XISetMask(&mut mask, xi::XI_KeyRelease);
                let mut events = xi::XIEventMask {
                    deviceid: id,
                    mask_len: mask.len() as i32,
                    mask: mask.as_mut_ptr(),
                };
                let status = xi::XIGrabDevice(
                    self.display,
                    id,
                    self.root,
                    x::CurrentTime,
                    0,
                    xi::XIGrabModeAsync,
                    xi::XIGrabModeAsync,
                    x::False,
                    &mut events,
                );
                if status != x::GrabSuccess {
                    return Err(format!(
                        "Cannot capture keyboard device {id} (XInput status {status}). Another input utility may own it."
                    ));
                }
                self.devices.insert(id);
            }
            if self.devices.is_empty() {
                return Err("No physical X11 keyboard is available for accent input.".into());
            }
            Ok(())
        }
    }
    fn focused(&self) -> Option<Target> {
        let focus = self.context.get_input_focus().ok()?.reply().ok()?.focus;
        if focus <= 1 {
            return None;
        }
        let mut window = focus;
        let mut pid = 0;
        for _ in 0..32 {
            if let Ok(cookie) =
                self.context
                    .get_property(false, window, self.pid_atom, AtomEnum::CARDINAL, 0, 1)
                && let Ok(reply) = cookie.reply()
                && let Some(value) = reply.value32().and_then(|mut values| values.next())
            {
                pid = value;
                break;
            }
            let tree = self.context.query_tree(window).ok()?.reply().ok()?;
            if tree.parent == tree.root || tree.parent == 0 {
                break;
            }
            window = tree.parent;
        }
        let executable = std::fs::read_link(format!("/proc/{pid}/exe"))
            .ok()
            .and_then(|p| p.file_name().map(|v| v.to_string_lossy().into_owned()))
            .unwrap_or_default();
        Some(Target {
            context: Context {
                window: window as usize,
                focus: focus as usize,
                thread: pid,
            },
            executable,
            protected: false,
        })
    }
    fn forward(&mut self, code: u32, down: bool) {
        unsafe {
            xtest::XTestFakeKeyEvent(self.display, code, down as i32, 0);
            x::XSync(self.display, x::False);
        }
        if down {
            self.held.insert(code);
        } else {
            self.held.remove(&code);
        }
    }
    fn repeat_delay(&self, code: u32) -> Option<u64> {
        unsafe {
            let mut state: x::XKeyboardState = std::mem::zeroed();
            x::XGetKeyboardControl(self.display, &mut state);
            if state.global_auto_repeat == 0
                || (state.auto_repeats[(code / 8) as usize] as u8) & (1 << (code % 8)) == 0
            {
                return None;
            }
            let (mut delay, mut interval) = (0, 0);
            (x::XkbGetAutoRepeatRate(self.display, 0x100, &mut delay, &mut interval) != 0)
                .then_some(delay as u64)
        }
    }
    fn key(&self, code: u32) -> (Key, bool, bool) {
        unsafe {
            let mut state: x::XkbStateRec = std::mem::zeroed();
            x::XkbGetState(self.display, 0x100, &mut state);
            let mut modifiers = 0;
            let mut symbol = 0;
            x::XkbLookupKeySym(
                self.display,
                code as u8,
                state.mods as u32 | ((state.group as u32) << 13),
                &mut modifiers,
                &mut symbol,
            );
            let blocked =
                state.mods as u32 & (x::ControlMask | x::Mod1Mask | x::Mod4Mask | x::Mod5Mask) != 0;
            let key = match symbol {
                0x20 => Key::Space,
                0xff51 => Key::Left,
                0xff53 => Key::Right,
                0xff1b => Key::Escape,
                0xffe1 | 0xffe2 => Key::Shift,
                0xffe3 | 0xffe4 | 0xffe7..=0xffee | 0xfe03 => Key::BlockingModifier,
                _ => {
                    if let Some(letter) = xkeysym::Keysym::new(symbol as u32).key_char() {
                        let lower = if letter == 'İ' {
                            'i'
                        } else {
                            letter.to_lowercase().next().unwrap_or(letter)
                        };
                        if letter.is_alphabetic() || letter == '\'' {
                            return (Key::Letter(lower), letter.is_uppercase(), blocked);
                        }
                    }
                    Key::Other
                }
            };
            (key, false, blocked)
        }
    }
}
impl Host for Native {
    fn target(&mut self) -> Target {
        self.focused().unwrap_or_default()
    }
    fn insert(&mut self, text: &str, replace: bool) -> Result<(), String> {
        if replace {
            self.enigo
                .key(enigo::Key::Backspace, enigo::Direction::Click)
                .map_err(|e| e.to_string())?;
        }
        self.enigo.text(text).map_err(|e| e.to_string())
    }
    fn picker(&mut self, view: Option<View>) {
        self.ui.picker(view);
    }
}
impl Drop for Native {
    fn drop(&mut self) {
        if self.display.is_null() {
            return;
        }
        unsafe {
            for code in &self.held {
                xtest::XTestFakeKeyEvent(self.display, *code, x::False, 0);
            }
            for device in &self.devices {
                xi::XIUngrabDevice(self.display, *device, x::CurrentTime);
            }
            x::XSync(self.display, x::False);
            x::XCloseDisplay(self.display);
        }
    }
}
pub fn run(
    preferences: Preferences,
    commands: mpsc::Receiver<Command>,
    ui: Ui,
    ready: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let mut native = Native::open(ui)?;
    let mut session = Session::new(preferences);
    ready.send(Ok(())).map_err(|e| e.to_string())?;
    let clock = Instant::now();
    loop {
        for command in commands.try_iter() {
            match command {
                Command::Stop => return Ok(()),
                Command::Configure(preferences) => session.configure(preferences, &mut native),
                Command::Pause(paused) => session.pause(paused, &mut native),
            }
        }
        unsafe {
            while x::XPending(native.display) > 0 {
                let mut event: x::XEvent = std::mem::zeroed();
                x::XNextEvent(native.display, &mut event);
                if event.get_type() != x::GenericEvent {
                    continue;
                }
                let mut cookie = event.generic_event_cookie;
                if cookie.extension != native.extension
                    || x::XGetEventData(native.display, &mut cookie) == 0
                {
                    continue;
                }
                match cookie.evtype {
                    xi::XI_KeyPress | xi::XI_KeyRelease => {
                        let event = &*(cookie.data as *const xi::XIDeviceEvent);
                        if native.devices.contains(&event.deviceid) {
                            let down = cookie.evtype == xi::XI_KeyPress;
                            let code = event.detail as u32;
                            let (key, uppercase, blocked) = native.key(code);
                            let target = native.target();
                            let physical = ((event.deviceid as u32) << 16) | code;
                            let now = clock.elapsed().as_millis() as u64;
                            if down
                                && key == Key::Space
                                && !session.active()
                                && let Some((&owner, &(owner_code, started))) = native
                                    .pressed
                                    .iter()
                                    .filter(|(_, (code, _))| {
                                        matches!(native.key(*code).0, Key::Letter(_))
                                    })
                                    .max_by_key(|(_, (_, started))| *started)
                                && native
                                    .repeat_delay(owner_code)
                                    .is_some_and(|delay| now.saturating_sub(started) >= delay)
                            {
                                let (key, uppercase, blocked) = native.key(owner_code);
                                let _ = session.event(
                                    NativeKey {
                                        physical: owner,
                                        key,
                                        down: true,
                                        uppercase,
                                        blocked,
                                        now_ms: now,
                                    },
                                    target.clone(),
                                );
                            }
                            if down {
                                native.pressed.entry(physical).or_insert((code, now));
                            } else {
                                native.pressed.remove(&physical);
                            }
                            let outcome = session.event(
                                NativeKey {
                                    physical,
                                    key,
                                    down,
                                    uppercase,
                                    blocked,
                                    now_ms: clock.elapsed().as_millis() as u64,
                                },
                                target,
                            );
                            if outcome.consume && down && key == Key::Space && session.active() {
                                // XTEST forwarding has its own server auto-repeat stream.
                                // Release forwarded character keys once a gesture owns them;
                                // physical releases are still tracked by Session.
                                let letters = native
                                    .held
                                    .iter()
                                    .copied()
                                    .filter(|code| matches!(native.key(*code).0, Key::Letter(_)))
                                    .collect::<Vec<_>>();
                                for code in letters {
                                    native.forward(code, false);
                                }
                            }
                            if !outcome.consume {
                                native.forward(code, down);
                            }
                            // Forward the owner's physical release before posting Unicode replacement.
                            if let Err(error) = session.apply(outcome.effects, &mut native) {
                                x::XFreeEventData(native.display, &mut cookie);
                                return Err(error);
                            }
                        }
                    }
                    xi::XI_RawButtonPress => session.cancel(&mut native),
                    xi::XI_HierarchyChanged => {
                        for code in native.held.iter().copied().collect::<Vec<_>>() {
                            native.forward(code, false);
                        }
                        native.pressed.clear();
                        session.reset(&mut native);
                        native.grab_devices()?;
                    }
                    _ => {}
                }
                x::XFreeEventData(native.display, &mut cookie);
            }
            session.tick(clock.elapsed().as_millis() as u64, &mut native)?;
            let mut descriptors = [libc::pollfd {
                fd: x::XConnectionNumber(native.display),
                events: libc::POLLIN,
                revents: 0,
            }];
            libc::poll(
                descriptors.as_mut_ptr(),
                1,
                if session.active() { 8 } else { 16 },
            );
        }
    }
}
