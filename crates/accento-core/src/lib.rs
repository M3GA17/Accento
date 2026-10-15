//! Deterministic accent selection. Time, keyboard state and target identity come from the host.

mod catalog;
pub use catalog::{Language, candidates};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Context {
    pub window: usize,
    pub focus: usize,
    pub thread: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Letter(char),
    Space,
    Left,
    Right,
    Escape,
    Shift,
    BlockingModifier,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub struct KeyEvent {
    pub key: Key,
    pub down: bool,
    pub repeat: bool,
    pub uppercase: bool,
    pub blocked: bool,
    pub context: Context,
    pub now_ms: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View {
    pub generation: u64,
    pub base: char,
    pub candidates: Vec<char>,
    pub selected: Option<usize>,
    pub context: Context,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    Show(View),
    Hide,
    Replace { context: Context, character: char },
    RestoreSpace { context: Context },
}

#[derive(Clone, Debug, Default)]
pub struct Outcome {
    pub consume: bool,
    pub effects: Vec<Effect>,
}

#[derive(Clone, Debug)]
struct Session {
    view: View,
    deadline: Option<u64>,
    open: bool,
    repeated: bool,
    trigger_released: bool,
}

pub struct Engine {
    languages: Vec<Language>,
    delay_ms: u64,
    generation: u64,
    session: Option<Session>,
    // Suppressed down events must have matching suppressed ups, even after cancellation.
    swallowed: [bool; 4],
    cancelled_owner: Option<char>,
}

impl Engine {
    pub fn new(languages: Vec<Language>, delay_ms: u64) -> Self {
        Self {
            languages,
            delay_ms: delay_ms.clamp(100, 1500),
            generation: 0,
            session: None,
            swallowed: [false; 4],
            cancelled_owner: None,
        }
    }

    pub fn configure(&mut self, languages: Vec<Language>, delay_ms: u64) -> Vec<Effect> {
        self.languages = languages;
        self.delay_ms = delay_ms.clamp(100, 1500);
        self.cancel()
    }

    pub fn cancel(&mut self) -> Vec<Effect> {
        self.generation += 1;
        if let Some(s) = self.session.take()
            && s.deadline.is_some()
        {
            self.cancelled_owner = Some(s.view.base);
            vec![Effect::Hide]
        } else {
            vec![]
        }
    }

    pub fn active(&self) -> bool {
        self.session.as_ref().is_some_and(|s| s.deadline.is_some())
    }

    /// Only a timer for the current generation may open the picker.
    pub fn tick(&mut self, generation: u64, now_ms: u64, context: Context) -> Vec<Effect> {
        let Some(s) = self.session.as_mut() else {
            return vec![];
        };
        if s.view.generation != generation {
            return vec![];
        }
        if s.view.context != context {
            return self.cancel();
        }
        if !s.open && s.deadline.is_some_and(|deadline| now_ms >= deadline) {
            s.open = true;
            return vec![Effect::Show(s.view.clone())];
        }
        vec![]
    }

    pub fn timer(&self) -> Option<(u64, u64)> {
        self.session
            .as_ref()
            .and_then(|s| s.deadline.map(|d| (s.view.generation, d)))
    }

    pub fn handle(&mut self, e: KeyEvent) -> Outcome {
        let mut out = Outcome::default();
        if let Some(owner) = self.cancelled_owner
            && e.key == Key::Letter(owner)
        {
            // Its original down reached the target, so its physical up must still pass through.
            if !e.down {
                self.cancelled_owner = None;
            } else {
                out.consume = true;
            }
            return out;
        }
        let slot = match e.key {
            Key::Space => Some(0),
            Key::Left => Some(1),
            Key::Right => Some(2),
            Key::Escape => Some(3),
            _ => None,
        };
        if e.down && e.repeat && slot.is_some_and(|slot| self.swallowed[slot]) {
            out.consume = true;
            return out;
        }
        if !e.down
            && let Some(slot) = slot
            && self.swallowed[slot]
        {
            self.swallowed[slot] = false;
            out.consume = true;
            if e.key == Key::Space
                && let Some(s) = self.session.as_mut()
            {
                s.trigger_released = true;
            }
            return out;
        }
        if self
            .session
            .as_ref()
            .is_some_and(|s| s.view.context != e.context)
        {
            out.effects.extend(self.cancel());
        }
        if e.blocked || e.key == Key::BlockingModifier {
            out.effects.extend(self.cancel());
            return out;
        }
        if let Some(s) = self.session.as_mut() {
            if e.key == Key::Letter(s.view.base) {
                if e.down {
                    if s.deadline.is_some() {
                        out.consume = true;
                    } else {
                        s.repeated = true;
                    }
                } else {
                    let s = self.session.take().unwrap();
                    if s.open {
                        if let Some(index) = s.view.selected {
                            out.effects.push(Effect::Replace {
                                context: s.view.context,
                                character: s.view.candidates[index],
                            });
                        }
                    } else if s.deadline.is_some() {
                        out.effects.push(Effect::RestoreSpace {
                            context: s.view.context,
                        });
                    }
                    if s.deadline.is_some() {
                        out.effects.push(Effect::Hide);
                    }
                }
                return out;
            }
            if e.down && e.key == Key::Escape && s.deadline.is_some() {
                self.swallowed[3] = true;
                out.consume = true;
                out.effects.extend(self.cancel());
                return out;
            }
            if e.down && e.key == Key::Space && s.deadline.is_none() && !s.repeated && !e.repeat {
                s.deadline = Some(e.now_ms + self.delay_ms);
                s.trigger_released = false;
                self.swallowed[0] = true;
                out.consume = true;
                return out;
            }
            if e.down
                && s.deadline.is_some()
                && matches!(e.key, Key::Space | Key::Left | Key::Right)
            {
                self.swallowed[slot.unwrap()] = true;
                out.consume = true;
                if s.open && s.trigger_released && !e.repeat {
                    let count = s.view.candidates.len();
                    s.view.selected = Some(match (s.view.selected, e.key) {
                        (None, Key::Left) => count - 1,
                        (None, _) => 0,
                        (Some(i), Key::Left) => (i + count - 1) % count,
                        (Some(i), _) => (i + 1) % count,
                    });
                    out.effects.push(Effect::Show(s.view.clone()));
                }
                return out;
            }
            if e.down && e.key != Key::Shift {
                out.effects.extend(self.cancel());
            }
        }
        if e.down
            && !e.repeat
            && let Key::Letter(base) = e.key
        {
            let list = candidates(base, e.uppercase, &self.languages);
            if !list.is_empty() && e.context.window != 0 {
                self.generation += 1;
                self.session = Some(Session {
                    view: View {
                        generation: self.generation,
                        base,
                        candidates: list,
                        selected: None,
                        context: e.context,
                    },
                    deadline: None,
                    open: false,
                    repeated: false,
                    trigger_released: false,
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const TARGET: Context = Context {
        window: 1,
        focus: 2,
        thread: 3,
    };
    fn key(key: Key, down: bool, now_ms: u64) -> KeyEvent {
        KeyEvent {
            key,
            down,
            repeat: false,
            uppercase: false,
            blocked: false,
            context: TARGET,
            now_ms,
        }
    }
    fn armed() -> Engine {
        let mut engine = Engine::new(vec![Language::Italian], 300);
        assert!(!engine.handle(key(Key::Letter('e'), true, 0)).consume);
        assert!(engine.handle(key(Key::Space, true, 10)).consume);
        engine
    }
    fn opened() -> Engine {
        let mut engine = armed();
        engine.handle(key(Key::Space, false, 50));
        let generation = engine.timer().unwrap().0;
        assert!(engine.tick(generation, 309, TARGET).is_empty());
        assert!(matches!(
            engine.tick(generation, 310, TARGET).as_slice(),
            [Effect::Show(View { selected: None, .. })]
        ));
        engine
    }
    #[test]
    fn accent_commits_on_base_release_only() {
        let mut engine = opened();
        engine.handle(key(Key::Right, true, 320));
        assert!(
            engine
                .handle(key(Key::Right, false, 330))
                .effects
                .is_empty()
        );
        assert!(matches!(
            engine
                .handle(key(Key::Letter('e'), false, 340))
                .effects
                .as_slice(),
            [
                Effect::Replace {
                    character: 'è', ..
                },
                Effect::Hide
            ]
        ));
    }
    #[test]
    fn quick_space_is_restored_once_and_key_up_is_balanced() {
        let mut engine = armed();
        assert!(matches!(
            engine
                .handle(key(Key::Letter('e'), false, 80))
                .effects
                .as_slice(),
            [Effect::RestoreSpace { .. }, Effect::Hide]
        ));
        assert!(engine.handle(key(Key::Space, false, 90)).consume);
        assert!(
            engine
                .handle(key(Key::Letter('e'), false, 100))
                .effects
                .is_empty()
        );
    }
    #[test]
    fn open_without_choice_keeps_the_base() {
        assert_eq!(
            opened().handle(key(Key::Letter('e'), false, 400)).effects,
            vec![Effect::Hide]
        );
    }
    #[test]
    fn escape_balances_key_up_and_invalidates_timer() {
        let mut engine = armed();
        let generation = engine.timer().unwrap().0;
        assert!(engine.handle(key(Key::Escape, true, 60)).consume);
        assert!(engine.handle(key(Key::Escape, false, 70)).consume);
        assert!(engine.tick(generation, 1000, TARGET).is_empty());
        assert!(engine.handle(key(Key::Space, false, 1100)).consume);
    }
    #[test]
    fn changed_target_never_receives_a_commit_or_replayed_space() {
        let mut engine = opened();
        engine.handle(key(Key::Right, true, 330));
        let mut event = key(Key::Letter('e'), false, 400);
        event.context.window = 9;
        assert_eq!(engine.handle(event).effects, vec![Effect::Hide]);
    }
    #[test]
    fn initial_left_and_wrap_are_deterministic() {
        let mut engine = opened();
        let out = engine.handle(key(Key::Left, true, 320));
        assert!(matches!(
            out.effects.as_slice(),
            [Effect::Show(View {
                selected: Some(1),
                ..
            })]
        ));
        engine.handle(key(Key::Left, false, 321));
        engine.handle(key(Key::Right, true, 330));
        assert!(matches!(
            engine
                .handle(key(Key::Letter('e'), false, 400))
                .effects
                .as_slice(),
            [
                Effect::Replace {
                    character: 'è', ..
                },
                Effect::Hide
            ]
        ));
    }
    #[test]
    fn auto_repeat_before_trigger_prevents_replacing_multiple_letters() {
        let mut engine = Engine::new(vec![Language::Italian], 300);
        engine.handle(key(Key::Letter('e'), true, 0));
        let mut repeat = key(Key::Letter('e'), true, 100);
        repeat.repeat = true;
        assert!(!engine.handle(repeat).consume);
        assert!(!engine.handle(key(Key::Space, true, 110)).consume);
        assert!(engine.timer().is_none());
    }
    #[test]
    fn shortcuts_cancel_without_consuming_them() {
        let mut engine = opened();
        let out = engine.handle(key(Key::BlockingModifier, true, 330));
        assert!(!out.consume);
        assert_eq!(out.effects, vec![Effect::Hide]);
    }
    #[test]
    fn cancellation_suppresses_owner_repeats_but_passes_its_release() {
        let mut engine = opened();
        engine.handle(key(Key::Escape, true, 320));
        let mut repeat = key(Key::Letter('e'), true, 330);
        repeat.repeat = true;
        assert!(engine.handle(repeat).consume);
        assert!(!engine.handle(key(Key::Letter('e'), false, 340)).consume);
        assert!(!engine.handle(key(Key::Letter('e'), true, 350)).consume);
    }
    #[test]
    fn cancelled_trigger_repeats_do_not_leak_into_the_document() {
        let mut engine = armed();
        engine.handle(key(Key::Escape, true, 20));
        let mut repeat = key(Key::Space, true, 30);
        repeat.repeat = true;
        assert!(engine.handle(repeat).consume);
        assert!(engine.handle(key(Key::Space, false, 40)).consume);
    }
    #[test]
    fn old_timer_does_not_open_a_new_session() {
        let mut engine = armed();
        let old = engine.timer().unwrap().0;
        engine.cancel();
        engine.handle(key(Key::Space, false, 20));
        engine.handle(key(Key::Letter('a'), true, 40));
        engine.handle(key(Key::Space, true, 50));
        assert!(engine.tick(old, 1000, TARGET).is_empty());
    }
    #[test]
    fn uppercase_is_captured_for_the_session() {
        let mut engine = Engine::new(vec![Language::Italian], 100);
        let mut e = key(Key::Letter('e'), true, 0);
        e.uppercase = true;
        engine.handle(e);
        engine.handle(key(Key::Space, true, 1));
        engine.handle(key(Key::Space, false, 2));
        engine.tick(engine.timer().unwrap().0, 101, TARGET);
        engine.handle(key(Key::Right, true, 102));
        assert!(matches!(
            engine
                .handle(key(Key::Letter('e'), false, 110))
                .effects
                .as_slice(),
            [
                Effect::Replace {
                    character: 'È', ..
                },
                Effect::Hide
            ]
        ));
    }
}
