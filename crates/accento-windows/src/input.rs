use crate::preferences::Preferences;
use accento_core::{Context, Effect, Engine, Key, KeyEvent};
use std::{
    cell::RefCell,
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Instant,
};
use windows::{
    Win32::{
        Foundation::*,
        System::{LibraryLoader::GetModuleHandleW, Threading::*},
        UI::{Accessibility::*, Input::KeyboardAndMouse::*, WindowsAndMessaging::*},
    },
    core::PWSTR,
};

pub const UI_EVENT: u32 = WM_APP + 1;
const COMMAND: u32 = WM_APP + 2;
const INSERT: u32 = WM_APP + 3;
const REFRESH: u32 = WM_APP + 4;
pub const OWN_TAG: usize = 0xACCE1701;

pub enum UiEvent {
    Effect(Effect),
    Error(String),
}
enum Command {
    Configure(Preferences),
    Pause(bool),
    Stop,
}

pub struct Worker {
    id: u32,
    commands: Sender<Command>,
    join: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(
        hwnd: HWND,
        preferences: Preferences,
        ui: Sender<UiEvent>,
    ) -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::channel();
        let window = hwnd.0 as usize;
        let join = thread::spawn(move || unsafe { run(window, preferences, rx, ui, ready_tx) });
        let id = ready_rx.recv().map_err(|e| e.to_string())??;
        Ok(Self {
            id,
            commands: tx,
            join: Some(join),
        })
    }
    fn command(&self, command: Command) {
        if self.commands.send(command).is_ok() {
            unsafe {
                let _ = PostThreadMessageW(self.id, COMMAND, WPARAM(0), LPARAM(0));
            }
        }
    }
    pub fn configure(&self, p: Preferences) {
        self.command(Command::Configure(p));
    }
    pub fn pause(&self, paused: bool) {
        self.command(Command::Pause(paused));
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.command(Command::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

struct Runtime {
    engine: Engine,
    prefs: Preferences,
    ui: Sender<UiEvent>,
    hwnd: usize,
    clock: Instant,
    down: [bool; 256],
    paused: bool,
    epoch: u64,
    pending: Option<(u64, Effect)>,
    timer: usize,
    process_window: usize,
    process_name: Option<String>,
    dead_key: bool,
    base_keys: [Option<char>; 256],
}

/// Windows primary language IDs for the national-language scope (script variants share IDs).
fn supported_layout(language_id: usize) -> bool {
    matches!(
        language_id & 0x3ff,
        0x02 | 0x03
            | 0x05
            | 0x06
            | 0x07
            | 0x08
            | 0x09
            | 0x0a
            | 0x0b
            | 0x0c
            | 0x0e
            | 0x0f
            | 0x10
            | 0x13
            | 0x14
            | 0x15
            | 0x16
            | 0x17
            | 0x18
            | 0x19
            | 0x1a
            | 0x1b
            | 0x1c
            | 0x1d
            | 0x1f
            | 0x22
            | 0x23
            | 0x24
            | 0x25
            | 0x26
            | 0x27
            | 0x2b
            | 0x2c
            | 0x2f
            | 0x37
            | 0x3a
            | 0x3c
            | 0x3f
            | 0x6e
            | 0x76
    )
}
fn translated_base(value: char) -> Option<char> {
    if value == 'İ' {
        return Some('i');
    }
    if !(value.is_alphabetic() || value == '\'') {
        return None;
    }
    let mut lower = value.to_lowercase();
    let first = lower.next()?;
    lower.next().is_none().then_some(first)
}

#[cfg(test)]
mod language_tests {
    use super::*;
    #[test]
    fn national_layouts_and_script_variants_are_recognized() {
        for layout in [
            0x0410, 0x041c, 0x0408, 0x0419, 0x0422, 0x0423, 0x042b, 0x0437, 0x043f, 0x042c, 0x046e,
            0x083c, 0x0814,
        ] {
            assert!(supported_layout(layout), "{layout:x}");
        }
        for layout in [0x0411, 0x0412, 0x0804] {
            assert!(!supported_layout(layout));
        }
    }
    #[test]
    fn composed_text_uses_one_replacement_and_preserves_every_utf16_unit() {
        for text in ["dh", "с́", "ÍJ́", "а́", "Ϊ́"] {
            let events = insertion_events(text, true);
            let units: Vec<_> = text.encode_utf16().collect();
            assert_eq!(events.len(), 2 + 2 * units.len());
            unsafe {
                assert_eq!(events[0].Anonymous.ki.wVk, VK_BACK);
                assert_eq!(events[1].Anonymous.ki.wVk, VK_BACK);
                for (pair, unit) in events[2..].as_chunks::<2>().0.iter().zip(units) {
                    assert_eq!(pair[0].Anonymous.ki.wScan, unit);
                    assert_eq!(pair[1].Anonymous.ki.wScan, unit);
                    assert_eq!(pair[0].Anonymous.ki.dwExtraInfo, OWN_TAG);
                    assert!(pair[1].Anonymous.ki.dwFlags.contains(KEYEVENTF_KEYUP));
                }
            }
        }
    }
    #[test]
    fn native_letter_identity_is_not_the_virtual_key_ascii_name() {
        for (letter, base) in [
            ('E', 'e'),
            ('Ε', 'ε'),
            ('Е', 'е'),
            ('Ա', 'ա'),
            ('Ა', 'ა'),
            ('İ', 'i'),
            ('Ç', 'ç'),
        ] {
            assert_eq!(translated_base(letter), Some(base));
        }
        assert_eq!(translated_base(' '), None);
        assert_eq!(translated_base('1'), None);
    }
}
thread_local! { static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) }; }

fn context() -> Context {
    unsafe {
        let window = GetForegroundWindow();
        if window.is_invalid() {
            return Context::default();
        }
        let thread = GetWindowThreadProcessId(window, None);
        let mut info = GUITHREADINFO {
            cbSize: size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(thread, &mut info).is_err() {
            return Context::default();
        }
        Context {
            window: window.0 as usize,
            focus: info.hwndFocus.0 as usize,
            thread,
        }
    }
}

impl Runtime {
    fn now(&self) -> u64 {
        self.clock.elapsed().as_millis() as u64
    }
    fn notify(&self, event: UiEvent) {
        if self.ui.send(event).is_ok() {
            unsafe {
                let _ = PostMessageW(Some(HWND(self.hwnd as _)), UI_EVENT, WPARAM(0), LPARAM(0));
            }
        }
    }
    fn effects(&mut self, effects: Vec<Effect>) {
        for effect in effects {
            match effect {
                Effect::Replace { .. } | Effect::RestoreSpace { .. } => {
                    self.pending = Some((self.epoch, effect));
                    unsafe {
                        let _ =
                            PostThreadMessageW(GetCurrentThreadId(), INSERT, WPARAM(0), LPARAM(0));
                    }
                }
                _ => self.notify(UiEvent::Effect(effect)),
            }
        }
        unsafe {
            if self.engine.active() && self.timer == 0 {
                self.timer = SetTimer(None, 0, 15, None);
            }
            if !self.engine.active() && self.timer != 0 {
                let _ = KillTimer(None, self.timer);
                self.timer = 0;
            }
        }
    }
    fn cancel(&mut self) {
        self.epoch += 1;
        self.pending = None;
        let effects = self.engine.cancel();
        self.effects(effects);
    }
    fn refresh_process(&mut self) {
        let c = context();
        self.process_window = c.window;
        self.process_name = None;
        unsafe {
            let mut pid = 0;
            GetWindowThreadProcessId(HWND(c.window as _), Some(&mut pid));
            if let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut buffer = [0u16; 32768];
                let mut length = buffer.len() as u32;
                if QueryFullProcessImageNameW(
                    process,
                    PROCESS_NAME_WIN32,
                    PWSTR(buffer.as_mut_ptr()),
                    &mut length,
                )
                .is_ok()
                {
                    self.process_name = String::from_utf16_lossy(&buffer[..length as usize])
                        .rsplit('\\')
                        .next()
                        .map(str::to_ascii_lowercase);
                }
                let _ = CloseHandle(process);
            }
        }
    }
    fn key(&mut self, data: &KBDLLHOOKSTRUCT, down: bool) -> bool {
        let vk = data.vkCode as usize;
        if vk >= 256 {
            return false;
        }
        if down {
            self.epoch += 1;
            self.pending = None;
        }
        let repeat = down && self.down[vk];
        self.down[vk] = down;
        if matches!(vk, 0xa0 | 0xa1) {
            self.down[VK_SHIFT.0 as usize] =
                self.down[VK_LSHIFT.0 as usize] || self.down[VK_RSHIFT.0 as usize];
        }
        if matches!(vk, 0xa2 | 0xa3) {
            self.down[VK_CONTROL.0 as usize] =
                self.down[VK_LCONTROL.0 as usize] || self.down[VK_RCONTROL.0 as usize];
        }
        if matches!(vk, 0xa4 | 0xa5) {
            self.down[VK_MENU.0 as usize] =
                self.down[VK_LMENU.0 as usize] || self.down[VK_RMENU.0 as usize];
        }
        let c = context();
        if self.process_window != c.window {
            // Some foreground transitions do not deliver a WinEvent (notably attached queues).
            // Resolve the executable after this callback, never by opening a process in the hook.
            unsafe {
                let _ = PostThreadMessageW(GetCurrentThreadId(), REFRESH, WPARAM(0), LPARAM(0));
            }
        }
        let layout = unsafe { GetKeyboardLayout(c.thread) }.0 as usize & 0xffff;
        let supported = supported_layout(layout);
        let modifier = [
            VK_CONTROL,
            VK_LCONTROL,
            VK_RCONTROL,
            VK_MENU,
            VK_LMENU,
            VK_RMENU,
            VK_LWIN,
            VK_RWIN,
        ]
        .iter()
        .any(|k| self.down[k.0 as usize]);
        let shift = self.down[VK_SHIFT.0 as usize]
            || self.down[VK_LSHIFT.0 as usize]
            || self.down[VK_RSHIFT.0 as usize];
        let excluded = !self.prefs.excluded_apps.is_empty()
            && (self.process_window != c.window
                || self.process_name.as_ref().is_none_or(|name| {
                    self.prefs
                        .excluded_apps
                        .iter()
                        .any(|s| s.eq_ignore_ascii_case(name))
                }));
        let password = c.focus != 0
            && unsafe { GetWindowLongW(HWND(c.focus as _), GWL_STYLE) } & 0x20 != 0
            && unsafe {
                let mut name = [0u16; 64];
                let len = GetClassNameW(HWND(c.focus as _), &mut name);
                String::from_utf16_lossy(&name[..len.max(0) as usize])
                    .to_ascii_lowercase()
                    .contains("edit")
            };
        let mut blocked =
            self.paused || modifier || !supported || excluded || password || self.dead_key;
        let mut uppercase = shift ^ (unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0);
        if down {
            // Flag 4 inspects translation without modifying the destination's dead-key state.
            let mut state = [0u8; 256];
            for (i, value) in state.iter_mut().enumerate() {
                if self.down[i] {
                    *value = 0x80;
                }
            }
            state[VK_CAPITAL.0 as usize] |= (unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1) as u8;
            let mut text = [0u16; 8];
            let count = unsafe {
                ToUnicodeEx(
                    data.vkCode,
                    data.scanCode,
                    &state,
                    &mut text,
                    4,
                    Some(GetKeyboardLayout(c.thread)),
                )
            };
            if !repeat {
                self.base_keys[vk] = None;
                if supported
                    && count == 1
                    && let Some(value) = char::from_u32(text[0] as u32)
                {
                    self.base_keys[vk] = translated_base(value);
                    uppercase = value.is_uppercase();
                }
            }
            if count < 0 {
                self.dead_key = true;
                blocked = true;
            } else if count > 0 && self.dead_key {
                self.dead_key = false;
                blocked = true;
            }
        }
        // Cache the translated base for the physical key until key-up, even if Shift or layout changes.
        let key = if let Some(base) = self.base_keys[vk] {
            Key::Letter(base)
        } else {
            match data.vkCode as u16 {
                0x20 => Key::Space,
                0x25 => Key::Left,
                0x27 => Key::Right,
                0x1b => Key::Escape,
                0x10 | 0xa0 | 0xa1 => Key::Shift,
                0x11 | 0x12 | 0x5b | 0x5c | 0xa2..=0xa5 => Key::BlockingModifier,
                _ => Key::Other,
            }
        };
        if !down {
            self.base_keys[vk] = None;
        }
        let now = self.now();
        if let Some((generation, _)) = self.engine.timer() {
            let effects = self.engine.tick(generation, now, c);
            self.effects(effects);
        }
        let out = self.engine.handle(KeyEvent {
            key,
            down,
            repeat,
            uppercase,
            blocked,
            context: c,
            now_ms: now,
        });
        self.effects(out.effects);
        out.consume
    }
}

unsafe extern "system" fn keyboard(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        let data = unsafe { &*(lp.0 as *const KBDLLHOOKSTRUCT) };
        if data.dwExtraInfo != OWN_TAG {
            let message = wp.0 as u32;
            if matches!(message, WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP) {
                let consume = RUNTIME.with(|r| {
                    r.borrow_mut()
                        .as_mut()
                        .is_some_and(|r| r.key(data, matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN)))
                });
                if consume {
                    return LRESULT(1);
                }
            }
        }
    }
    unsafe { CallNextHookEx(None, code, wp, lp) }
}
unsafe extern "system" fn mouse(code: i32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32
        && matches!(
            wp.0 as u32,
            WM_LBUTTONDOWN | WM_RBUTTONDOWN | WM_MBUTTONDOWN | WM_MOUSEWHEEL | WM_XBUTTONDOWN
        )
    {
        RUNTIME.with(|r| {
            if let Some(r) = r.borrow_mut().as_mut() {
                r.cancel();
            }
        });
    }
    unsafe { CallNextHookEx(None, code, wp, lp) }
}
unsafe extern "system" fn foreground(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    RUNTIME.with(|r| {
        if let Some(r) = r.borrow_mut().as_mut() {
            r.cancel();
        }
    });
    unsafe {
        let _ = PostThreadMessageW(GetCurrentThreadId(), REFRESH, WPARAM(0), LPARAM(0));
    }
}

fn input(key: u16, unicode: bool, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: if unicode {
                    VIRTUAL_KEY(0)
                } else {
                    VIRTUAL_KEY(key)
                },
                wScan: if unicode { key } else { 0 },
                dwFlags: (if unicode {
                    KEYEVENTF_UNICODE
                } else {
                    KEYBD_EVENT_FLAGS(0)
                }) | (if up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYBD_EVENT_FLAGS(0)
                }),
                dwExtraInfo: OWN_TAG,
                ..Default::default()
            },
        },
    }
}
fn insert(effect: Effect) -> Result<(), String> {
    let (expected, text, replace) = match effect {
        Effect::Replace { context, text } => (context, text, true),
        Effect::RestoreSpace { context } => (context, " ".into(), false),
        _ => return Ok(()),
    };
    if context() != expected {
        return Ok(());
    }
    let events = insertion_events(&text, replace);
    let sent = unsafe { SendInput(&events, size_of::<INPUT>() as i32) } as usize;
    if sent != events.len() {
        return Err(format!(
            "Inserimento interrotto ({sent}/{} eventi). Il testo potrebbe essere cambiato. Nessun tentativo automatico. Controlla anche se il programma destinatario è elevato.",
            events.len()
        ));
    }
    Ok(())
}
fn insertion_events(text: &str, replace: bool) -> Vec<INPUT> {
    let mut events = Vec::with_capacity(2 + text.encode_utf16().count() * 2);
    if replace {
        events.push(input(VK_BACK.0, false, false));
        events.push(input(VK_BACK.0, false, true));
    }
    for unit in text.encode_utf16() {
        events.push(input(unit, true, false));
        events.push(input(unit, true, true));
    }
    events
}

unsafe fn run(
    hwnd: usize,
    prefs: Preferences,
    commands: Receiver<Command>,
    ui: Sender<UiEvent>,
    ready: Sender<Result<u32, String>>,
) {
    unsafe {
        let mut message = MSG::default();
        let _ = PeekMessageW(&mut message, None, 0, 0, PM_NOREMOVE);
        let mut runtime = Runtime {
            engine: Engine::new(prefs.languages(), prefs.delay_ms, prefs.currency_symbols),
            prefs,
            ui,
            hwnd,
            clock: Instant::now(),
            down: [false; 256],
            paused: false,
            epoch: 0,
            pending: None,
            timer: 0,
            process_window: 0,
            process_name: None,
            dead_key: false,
            base_keys: [None; 256],
        };
        for vk in 0..256 {
            runtime.down[vk] = GetAsyncKeyState(vk as i32) < 0;
        }
        runtime.refresh_process();
        RUNTIME.with(|r| *r.borrow_mut() = Some(runtime));
        let module = GetModuleHandleW(None).ok().map(|m| HINSTANCE(m.0));
        let keyboard_hook = match SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), module, 0) {
            Ok(h) => h,
            Err(e) => {
                let _ = ready.send(Err(format!(
                    "Impossibile installare il listener tastiera: {e}"
                )));
                return;
            }
        };
        let mouse_hook = match SetWindowsHookExW(WH_MOUSE_LL, Some(mouse), module, 0) {
            Ok(h) => h,
            Err(e) => {
                let _ = UnhookWindowsHookEx(keyboard_hook);
                let _ = ready.send(Err(e.to_string()));
                return;
            }
        };
        let foreground_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(foreground),
            0,
            0,
            WINEVENT_OUTOFCONTEXT,
        );
        if foreground_hook.is_invalid() {
            let _ = UnhookWindowsHookEx(keyboard_hook);
            let _ = UnhookWindowsHookEx(mouse_hook);
            let _ = ready.send(Err("Impossibile osservare i cambi di finestra.".into()));
            return;
        }
        let _ = ready.send(Ok(GetCurrentThreadId()));
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            if message.message == INSERT {
                // Release the runtime borrow before SendInput can synchronously re-enter a hook.
                let pending = RUNTIME.with(|r| {
                    r.borrow_mut()
                        .as_mut()
                        .and_then(|r| r.pending.take().filter(|(epoch, _)| *epoch == r.epoch))
                });
                if let Some((_, effect)) = pending
                    && let Err(error) = insert(effect)
                {
                    RUNTIME.with(|r| {
                        if let Some(r) = r.borrow().as_ref() {
                            r.notify(UiEvent::Error(error));
                        }
                    });
                }
            } else if message.message == COMMAND {
                let mut stop = false;
                RUNTIME.with(|r| {
                    let mut r = r.borrow_mut();
                    let r = r.as_mut().unwrap();
                    while let Ok(command) = commands.try_recv() {
                        match command {
                            Command::Configure(p) => {
                                r.cancel();
                                let effects = r.engine.configure(
                                    p.languages(),
                                    p.delay_ms,
                                    p.currency_symbols,
                                );
                                r.prefs = p;
                                r.effects(effects);
                            }
                            Command::Pause(paused) => {
                                r.cancel();
                                r.paused = paused;
                            }
                            Command::Stop => {
                                r.cancel();
                                stop = true;
                            }
                        }
                    }
                });
                if stop {
                    break;
                }
            } else if message.message == REFRESH {
                RUNTIME.with(|r| {
                    if let Some(r) = r.borrow_mut().as_mut() {
                        r.refresh_process();
                    }
                });
            } else if message.message == WM_TIMER {
                RUNTIME.with(|r| {
                    if let Some(r) = r.borrow_mut().as_mut()
                        && let Some((generation, _)) = r.engine.timer()
                    {
                        let effects = r.engine.tick(generation, r.now(), context());
                        r.effects(effects);
                    }
                });
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
        let _ = UnhookWindowsHookEx(keyboard_hook);
        let _ = UnhookWindowsHookEx(mouse_hook);
        let _ = UnhookWinEvent(foreground_hook);
        RUNTIME.with(|r| {
            if let Some(mut r) = r.borrow_mut().take() {
                r.cancel();
            }
        });
    }
}
