//! Shared gesture integration for the Unix hosts. Native events stay outside the engine.
use crate::preferences::Preferences;
use accento_core::{Context, Effect, Engine, Key, KeyEvent, Outcome, View};
use std::collections::HashMap;

#[derive(Clone, Default)]
pub struct Target {
    pub context: Context,
    pub executable: String,
    pub protected: bool,
}

pub trait Host {
    fn target(&mut self) -> Target;
    fn insert(&mut self, text: &str, replace: bool) -> Result<(), String>;
    fn picker(&mut self, view: Option<View>);
}

pub struct NativeKey {
    pub physical: u32,
    pub key: Key,
    pub down: bool,
    pub uppercase: bool,
    pub blocked: bool,
    pub now_ms: u64,
}

pub struct Session {
    engine: Engine,
    preferences: Preferences,
    paused: bool,
    down: HashMap<u32, Key>,
}
impl Session {
    pub fn new(preferences: Preferences) -> Self {
        Self {
            engine: Engine::new(
                preferences.languages(),
                preferences.delay_ms,
                preferences.currency_symbols,
            ),
            preferences,
            paused: false,
            down: HashMap::new(),
        }
    }
    pub fn configure(&mut self, preferences: Preferences, host: &mut impl Host) {
        let effects = self.engine.configure(
            preferences.languages(),
            preferences.delay_ms,
            preferences.currency_symbols,
        );
        self.preferences = preferences;
        let _ = self.apply(effects, host);
    }
    pub fn pause(&mut self, paused: bool, host: &mut impl Host) {
        self.paused = paused;
        self.cancel(host);
    }
    pub fn cancel(&mut self, host: &mut impl Host) {
        let effects = self.engine.cancel();
        let _ = self.apply(effects, host);
    }
    /// Recover after a device is disconnected or the OS reports that hook events were lost.
    pub fn reset(&mut self, host: &mut impl Host) {
        self.engine = Engine::new(
            self.preferences.languages(),
            self.preferences.delay_ms,
            self.preferences.currency_symbols,
        );
        self.down.clear();
        host.picker(None);
    }
    pub fn active(&self) -> bool {
        self.engine.active()
    }
    pub fn event(&mut self, input: NativeKey, target: Target) -> Outcome {
        let NativeKey {
            physical,
            key,
            down,
            uppercase,
            blocked,
            now_ms,
        } = input;
        let repeat = down && self.down.contains_key(&physical);
        let key = if down {
            *self.down.entry(physical).or_insert(key)
        } else {
            self.down.remove(&physical).unwrap_or(key)
        };
        let excluded = self
            .preferences
            .excluded_apps
            .iter()
            .any(|name| name.eq_ignore_ascii_case(&target.executable));
        self.engine.handle(KeyEvent {
            key,
            down,
            repeat,
            uppercase,
            blocked: blocked || self.paused || excluded || target.protected,
            context: target.context,
            now_ms,
        })
    }
    pub fn tick(&mut self, now_ms: u64, host: &mut impl Host) -> Result<(), String> {
        if let Some((generation, _)) = self.engine.timer() {
            let target = host.target();
            let effects = if target.protected {
                self.engine.cancel()
            } else {
                self.engine.tick(generation, now_ms, target.context)
            };
            self.apply(effects, host)?;
        }
        Ok(())
    }
    pub fn apply(&mut self, effects: Vec<Effect>, host: &mut impl Host) -> Result<(), String> {
        for effect in effects {
            match effect {
                Effect::Show(view) => host.picker(Some(view)),
                Effect::Hide => host.picker(None),
                Effect::Replace { context, text } => {
                    let target = host.target();
                    if !self.paused && !target.protected && target.context == context {
                        host.insert(&text, true)?;
                    }
                }
                Effect::RestoreSpace { context } => {
                    let target = host.target();
                    if !self.paused && !target.protected && target.context == context {
                        host.insert(" ", false)?;
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct TestHost {
        target: Target,
        inserted: Vec<(String, bool)>,
        shown: Option<View>,
    }
    impl Host for TestHost {
        fn target(&mut self) -> Target {
            self.target.clone()
        }
        fn insert(&mut self, text: &str, replace: bool) -> Result<(), String> {
            self.inserted.push((text.into(), replace));
            Ok(())
        }
        fn picker(&mut self, view: Option<View>) {
            self.shown = view;
        }
    }
    fn fixture() -> (Session, TestHost) {
        (
            Session::new(Preferences::default()),
            TestHost {
                target: Target {
                    context: Context {
                        window: 1,
                        focus: 2,
                        thread: 3,
                    },
                    executable: "editor".into(),
                    protected: false,
                },
                ..Default::default()
            },
        )
    }
    fn event(
        session: &mut Session,
        host: &mut TestHost,
        code: u32,
        key: Key,
        down: bool,
        time: u64,
    ) -> bool {
        let outcome = session.event(
            NativeKey {
                physical: code,
                key,
                down,
                uppercase: false,
                blocked: false,
                now_ms: time,
            },
            host.target(),
        );
        session.apply(outcome.effects, host).unwrap();
        outcome.consume
    }
    #[test]
    fn native_key_identity_survives_layout_changes_and_release_commits() {
        let (mut session, mut host) = fixture();
        assert!(!event(
            &mut session,
            &mut host,
            24,
            Key::Letter('e'),
            true,
            0
        ));
        assert!(event(&mut session, &mut host, 65, Key::Space, true, 1));
        assert!(event(&mut session, &mut host, 65, Key::Space, false, 2));
        session.tick(400, &mut host).unwrap();
        assert!(host.shown.is_some());
        event(&mut session, &mut host, 24, Key::Letter('x'), false, 401);
        assert_eq!(host.inserted, [("è".into(), true)]);
        assert!(host.shown.is_none());
    }
    #[test]
    fn currencies_reach_the_host_and_opt_out_survives_device_reset() {
        let (mut session, mut host) = fixture();
        event(&mut session, &mut host, 24, Key::Letter('e'), true, 0);
        event(&mut session, &mut host, 65, Key::Space, true, 1);
        event(&mut session, &mut host, 65, Key::Space, false, 2);
        session.tick(400, &mut host).unwrap();
        event(&mut session, &mut host, 113, Key::Left, true, 401);
        event(&mut session, &mut host, 113, Key::Left, false, 402);
        event(&mut session, &mut host, 24, Key::Letter('e'), false, 403);
        assert_eq!(host.inserted, [("€".into(), true)]);
        session.configure(
            Preferences {
                currency_symbols: false,
                ..Default::default()
            },
            &mut host,
        );
        session.reset(&mut host);
        event(&mut session, &mut host, 46, Key::Letter('l'), true, 500);
        assert!(!event(&mut session, &mut host, 65, Key::Space, true, 501));
        session.configure(Preferences::default(), &mut host);
        session.reset(&mut host);
        event(&mut session, &mut host, 46, Key::Letter('l'), true, 600);
        assert!(event(&mut session, &mut host, 65, Key::Space, true, 601));
        session.tick(1000, &mut host).unwrap();
        event(&mut session, &mut host, 46, Key::Letter('l'), false, 1001);
        assert_eq!(host.inserted.last(), Some(&("£".into(), true)));
    }
    #[test]
    fn focus_change_and_protected_fields_never_receive_replacement() {
        let (mut session, mut host) = fixture();
        event(&mut session, &mut host, 24, Key::Letter('e'), true, 0);
        event(&mut session, &mut host, 65, Key::Space, true, 1);
        event(&mut session, &mut host, 65, Key::Space, false, 2);
        host.target.context.focus = 10;
        session.tick(400, &mut host).unwrap();
        event(&mut session, &mut host, 24, Key::Letter('e'), false, 401);
        assert!(host.inserted.is_empty());
        host.target.protected = true;
        event(&mut session, &mut host, 24, Key::Letter('e'), true, 500);
        assert!(!event(&mut session, &mut host, 65, Key::Space, true, 501));
    }
    #[test]
    fn pause_and_exclusion_preserve_normal_input() {
        let (mut session, mut host) = fixture();
        let preferences = Preferences {
            excluded_apps: vec!["editor".into()],
            ..Default::default()
        };
        session.configure(preferences, &mut host);
        event(&mut session, &mut host, 24, Key::Letter('e'), true, 0);
        assert!(!event(&mut session, &mut host, 65, Key::Space, true, 1));
        session.pause(true, &mut host);
        assert!(host.inserted.is_empty());
    }
    #[test]
    fn lost_device_events_do_not_leave_keys_or_suppression_stuck() {
        let (mut session, mut host) = fixture();
        event(&mut session, &mut host, 24, Key::Letter('e'), true, 0);
        event(&mut session, &mut host, 65, Key::Space, true, 1);
        session.reset(&mut host);
        assert!(!event(&mut session, &mut host, 65, Key::Space, true, 10));
        event(&mut session, &mut host, 65, Key::Space, false, 11);
        event(&mut session, &mut host, 24, Key::Letter('e'), true, 20);
        assert!(event(&mut session, &mut host, 65, Key::Space, true, 21));
    }
}
