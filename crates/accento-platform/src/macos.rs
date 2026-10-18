//! Quartz event tap and Unicode events. Accessibility authorization is required.
use super::{Command, Ui};
use crate::{
    preferences::Preferences,
    session::{Host, NativeKey, Session, Target},
};
use accento_core::{Context, Effect, Key, View};
use std::{
    ffi::{c_char, c_void},
    ptr,
    sync::mpsc,
    time::Instant,
};
type Ref = *const c_void;
type Event = *mut c_void;
const OWN_TAG: i64 = 0x414343454e544f;
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        mask: u64,
        callback: unsafe extern "C" fn(Ref, u32, Event, *mut c_void) -> Event,
        user: *mut c_void,
    ) -> Ref;
    fn CGEventTapEnable(tap: Ref, enabled: bool);
    fn CGEventGetIntegerValueField(event: Event, field: u32) -> i64;
    fn CGEventSetIntegerValueField(event: Event, field: u32, value: i64);
    fn CGEventGetFlags(event: Event) -> u64;
    fn CGEventSetFlags(event: Event, flags: u64);
    fn CGEventKeyboardGetUnicodeString(
        event: Event,
        max: usize,
        length: *mut usize,
        string: *mut u16,
    );
    fn CGEventKeyboardSetUnicodeString(event: Event, length: usize, string: *const u16);
    fn CGEventSourceCreate(state: i32) -> Ref;
    fn CGEventCreateKeyboardEvent(source: Ref, key: u16, down: bool) -> Event;
    fn CGEventPost(tap: u32, event: Event);
    fn CGPreflightListenEventAccess() -> bool;
    fn CGPreflightPostEventAccess() -> bool;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopCommonModes: Ref;
    fn CFRelease(value: Ref);
    fn CFHash(value: Ref) -> usize;
    fn CFStringCreateWithCString(allocator: Ref, text: *const c_char, encoding: u32) -> Ref;
    fn CFStringGetCString(value: Ref, buffer: *mut c_char, size: isize, encoding: u32) -> bool;
    fn CFMachPortCreateRunLoopSource(allocator: Ref, port: Ref, order: isize) -> Ref;
    fn CFMachPortInvalidate(port: Ref);
    fn CFRunLoopGetCurrent() -> Ref;
    fn CFRunLoopAddSource(loop_: Ref, source: Ref, mode: Ref);
    fn CFRunLoopRun();
    fn CFRunLoopStop(loop_: Ref);
    fn CFAbsoluteTimeGetCurrent() -> f64;
    fn CFRunLoopTimerCreate(
        allocator: Ref,
        fire: f64,
        interval: f64,
        flags: usize,
        order: isize,
        callback: unsafe extern "C" fn(Ref, *mut c_void),
        context: *mut TimerContext,
    ) -> Ref;
    fn CFRunLoopAddTimer(loop_: Ref, timer: Ref, mode: Ref);
    fn CFRunLoopTimerInvalidate(timer: Ref);
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> Ref;
    fn AXUIElementCopyAttributeValue(element: Ref, name: Ref, value: *mut Ref) -> i32;
    fn AXUIElementGetPid(element: Ref, pid: *mut i32) -> i32;
    fn AXUIElementSetMessagingTimeout(element: Ref, timeout: f32) -> i32;
}
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn IsSecureEventInputEnabled() -> bool;
}
#[link(name = "proc")]
unsafe extern "C" {
    fn proc_name(pid: i32, buffer: *mut c_void, size: u32) -> i32;
}
#[repr(C)]
struct TimerContext {
    version: isize,
    info: *mut c_void,
    retain: Ref,
    release: Ref,
    description: Ref,
}
#[derive(serde::Serialize)]
pub struct PermissionStatus {
    accessibility: bool,
    input_monitoring: bool,
    post_events: bool,
    secure_event_input: bool,
}
pub fn permission_status() -> PermissionStatus {
    unsafe {
        PermissionStatus {
            accessibility: AXIsProcessTrusted(),
            input_monitoring: CGPreflightListenEventAccess(),
            post_events: CGPreflightPostEventAccess(),
            secure_event_input: IsSecureEventInputEnabled(),
        }
    }
}
struct Owned(Ref);
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CFRelease(self.0);
            }
        }
    }
}
struct Native {
    system: Owned,
    source: Owned,
    ui: Ui,
}
impl Native {
    fn attribute(element: Ref, name: &std::ffi::CStr) -> Option<Owned> {
        unsafe {
            let key = Owned(CFStringCreateWithCString(
                ptr::null(),
                name.as_ptr(),
                0x08000100,
            ));
            let mut value = ptr::null();
            (AXUIElementCopyAttributeValue(element, key.0, &mut value) == 0 && !value.is_null())
                .then_some(Owned(value))
        }
    }
    fn string(value: Ref) -> String {
        unsafe {
            let mut bytes = [0i8; 256];
            if CFStringGetCString(value, bytes.as_mut_ptr(), bytes.len() as isize, 0x08000100) {
                std::ffi::CStr::from_ptr(bytes.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            }
        }
    }
    fn post(&self, key: u16, text: Option<&str>) -> Result<(), String> {
        let units = text.map(|text| text.encode_utf16().collect::<Vec<_>>());
        for down in [true, false] {
            unsafe {
                let event = CGEventCreateKeyboardEvent(self.source.0, key, down);
                if event.is_null() {
                    return Err("Quartz could not create a Unicode keyboard event.".into());
                }
                let _event = Owned(event);
                CGEventSetFlags(event, 0);
                CGEventSetIntegerValueField(event, 42, OWN_TAG);
                if let Some(units) = &units {
                    CGEventKeyboardSetUnicodeString(event, units.len(), units.as_ptr());
                }
                CGEventPost(0, event);
            }
        }
        Ok(())
    }
}
impl Host for Native {
    fn target(&mut self) -> Target {
        unsafe {
            if IsSecureEventInputEnabled() {
                return Target {
                    protected: true,
                    ..Default::default()
                };
            }
            let Some(focus) = Self::attribute(self.system.0, c"AXFocusedUIElement") else {
                return Target {
                    protected: true,
                    ..Default::default()
                };
            };
            let mut pid = 0;
            if AXUIElementGetPid(focus.0, &mut pid) != 0 || pid <= 0 {
                return Target::default();
            }
            let secure = [c"AXRole", c"AXSubrole"].iter().any(|name| {
                Self::attribute(focus.0, name)
                    .is_some_and(|value| Self::string(value.0) == "AXSecureTextField")
            });
            let mut name = [0i8; 1024];
            let executable = if proc_name(pid, name.as_mut_ptr().cast(), name.len() as u32) > 0 {
                std::ffi::CStr::from_ptr(name.as_ptr())
                    .to_string_lossy()
                    .into_owned()
            } else {
                String::new()
            };
            Target {
                context: Context {
                    window: pid as usize,
                    focus: CFHash(focus.0),
                    thread: pid as u32,
                },
                executable,
                protected: secure || pid as u32 == std::process::id(),
            }
        }
    }
    fn insert(&mut self, text: &str, replace: bool) -> Result<(), String> {
        if replace {
            self.post(51, None)?;
        }
        self.post(0, Some(text))
    }
    fn picker(&mut self, view: Option<View>) {
        self.ui.picker(view);
    }
}
struct Runtime {
    session: Session,
    native: Native,
    commands: mpsc::Receiver<Command>,
    clock: Instant,
    tap: Ref,
    pending: Vec<Effect>,
}
unsafe extern "C" fn callback(_: Ref, kind: u32, event: Event, user: *mut c_void) -> Event {
    unsafe {
        let state = &mut *user.cast::<Runtime>();
        if kind >= 0xfffffffe {
            state.session.reset(&mut state.native);
            state.pending.clear();
            CGEventTapEnable(state.tap, true);
            return event;
        }
        if event.is_null() || CGEventGetIntegerValueField(event, 42) == OWN_TAG {
            return event;
        }
        // Releases (including Shift after an uppercase letter) do not alter the
        // document. New presses, shortcuts and pointer actions invalidate a
        // deferred commit before they can move the caret or type another letter.
        if kind == 10
            || !matches!(kind, 10..=12)
            || (kind == 12 && CGEventGetFlags(event) & ((1 << 18) | (1 << 19) | (1 << 20)) != 0)
        {
            state.pending.clear();
        }
        if !matches!(kind, 10..=12) {
            state.session.cancel(&mut state.native);
            return event;
        }
        let code = CGEventGetIntegerValueField(event, 9) as u32;
        let flags = CGEventGetFlags(event);
        let down = if kind == 12 {
            match code {
                56 | 60 => flags & (1 << 17) != 0,
                59 | 62 => flags & (1 << 18) != 0,
                58 | 61 => flags & (1 << 19) != 0,
                55 | 54 => flags & (1 << 20) != 0,
                _ => false,
            }
        } else {
            kind == 10
        };
        let key = match code {
            49 => Key::Space,
            123 => Key::Left,
            124 => Key::Right,
            53 => Key::Escape,
            56 | 60 => Key::Shift,
            54 | 55 | 58 | 59 | 61 | 62 | 63 => Key::BlockingModifier,
            _ => {
                let mut length = 0;
                let mut text = [0u16; 8];
                CGEventKeyboardGetUnicodeString(event, text.len(), &mut length, text.as_mut_ptr());
                let decoded = String::from_utf16_lossy(&text[..length.min(text.len())]);
                let mut characters = decoded.chars();
                if let Some(letter) = characters
                    .next()
                    .filter(|c| c.is_alphabetic() || *c == '\'')
                {
                    if characters.next().is_none() {
                        Key::Letter(if letter == 'İ' {
                            'i'
                        } else {
                            letter.to_lowercase().next().unwrap_or(letter)
                        })
                    } else {
                        Key::Other
                    }
                } else {
                    Key::Other
                }
            }
        };
        let target = state.native.target();
        let outcome = state.session.event(
            NativeKey {
                physical: code,
                key,
                down,
                uppercase: (flags & (1 << 16) != 0) ^ (flags & (1 << 17) != 0),
                blocked: flags & ((1 << 18) | (1 << 19) | (1 << 20)) != 0,
                now_ms: state.clock.elapsed().as_millis() as u64,
            },
            target,
        );
        let mut immediate = Vec::new();
        for effect in outcome.effects {
            if matches!(effect, Effect::Replace { .. } | Effect::RestoreSpace { .. }) {
                state.pending.push(effect);
            } else {
                immediate.push(effect);
            }
        }
        if let Err(error) = state.session.apply(immediate, &mut state.native) {
            state.native.ui.failed(error);
        }
        if outcome.consume {
            ptr::null_mut()
        } else {
            event
        }
    }
}
unsafe extern "C" fn timer(_: Ref, user: *mut c_void) {
    unsafe {
        let state = &mut *user.cast::<Runtime>();
        for command in state.commands.try_iter() {
            match command {
                Command::Stop => {
                    state.session.pause(true, &mut state.native);
                    CFRunLoopStop(CFRunLoopGetCurrent());
                    return;
                }
                Command::Configure(preferences) => {
                    state.pending.clear();
                    state.session.configure(preferences, &mut state.native);
                }
                Command::Pause(paused) => {
                    state.pending.clear();
                    state.session.pause(paused, &mut state.native);
                }
            }
        }
        let pending = std::mem::take(&mut state.pending);
        let result = state
            .session
            .apply(pending, &mut state.native)
            .and_then(|()| {
                state
                    .session
                    .tick(state.clock.elapsed().as_millis() as u64, &mut state.native)
            });
        if let Err(error) = result {
            state.session.pause(true, &mut state.native);
            state.native.ui.failed(error);
        }
    }
}
pub fn run(
    preferences: Preferences,
    commands: mpsc::Receiver<Command>,
    ui: Ui,
    ready: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    unsafe {
        if !AXIsProcessTrusted() {
            return Err("Enable Accento in System Settings → Privacy & Security → Accessibility, then turn Accento on again. Input Monitoring may also be required by macOS. Settings remain available.".into());
        }
        let system = Owned(AXUIElementCreateSystemWide());
        AXUIElementSetMessagingTimeout(system.0, 0.03);
        let source = Owned(CGEventSourceCreate(-1));
        if source.0.is_null() {
            return Err("Quartz keyboard event source is unavailable.".into());
        }
        let mut runtime = Box::new(Runtime {
            session: Session::new(preferences),
            native: Native { system, source, ui },
            commands,
            clock: Instant::now(),
            tap: ptr::null(),
            pending: Vec::new(),
        });
        let user = (&mut *runtime as *mut Runtime).cast();
        let mask = [1, 3, 10, 11, 12, 22, 25]
            .iter()
            .fold(0u64, |mask, kind| mask | (1 << kind));
        let tap = Owned(CGEventTapCreate(1, 0, 0, mask, callback, user));
        if tap.0.is_null() {
            return Err("macOS denied the keyboard event tap. Grant Accessibility and Input Monitoring access to this Accento app, then turn it on again.".into());
        }
        runtime.tap = tap.0;
        let source = Owned(CFMachPortCreateRunLoopSource(ptr::null(), tap.0, 0));
        if source.0.is_null() {
            return Err("Cannot create the keyboard event run-loop source.".into());
        }
        let loop_ = CFRunLoopGetCurrent();
        CFRunLoopAddSource(loop_, source.0, kCFRunLoopCommonModes);
        let mut context = TimerContext {
            version: 0,
            info: user,
            retain: ptr::null(),
            release: ptr::null(),
            description: ptr::null(),
        };
        let timer_ = Owned(CFRunLoopTimerCreate(
            ptr::null(),
            CFAbsoluteTimeGetCurrent() + 0.008,
            0.008,
            0,
            0,
            timer,
            &mut context,
        ));
        if timer_.0.is_null() {
            return Err("Cannot create the keyboard scheduler.".into());
        }
        CFRunLoopAddTimer(loop_, timer_.0, kCFRunLoopCommonModes);
        CGEventTapEnable(tap.0, true);
        ready.send(Ok(())).map_err(|e| e.to_string())?;
        CFRunLoopRun();
        CGEventTapEnable(tap.0, false);
        CFRunLoopTimerInvalidate(timer_.0);
        CFMachPortInvalidate(tap.0);
        Ok(())
    }
}
