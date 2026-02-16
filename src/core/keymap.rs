use crossterm::event::{KeyCode, KeyModifiers, KeyEvent};
use std::collections::HashMap;
use crate::core::actions::Action;
use crate::core::state::Mode;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombination {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombination {
    pub fn from_event(event: KeyEvent) -> Self {
        Self {
            code: event.code,
            modifiers: event.modifiers,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        let mut modifiers = KeyModifiers::empty();
        let code_str;

        if parts.len() > 1 {
            for i in 0..parts.len() - 1 {
                match parts[i].to_lowercase().as_str() {
                    "ctrl" | "c" => modifiers.insert(KeyModifiers::CONTROL),
                    "alt" | "a" => modifiers.insert(KeyModifiers::ALT),
                    "shift" | "s" => modifiers.insert(KeyModifiers::SHIFT),
                    _ => {}
                }
            }
            code_str = parts[parts.len() - 1];
        } else {
            code_str = parts[0];
        }

        let code = match code_str.to_lowercase().as_str() {
            "enter" => KeyCode::Enter,
            "esc" => KeyCode::Esc,
            "backspace" => KeyCode::Backspace,
            "tab" => KeyCode::Tab,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "space" => KeyCode::Char(' '),
            s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
            _ => return None,
        };

        Some(Self { code, modifiers })
    }
}

#[derive(Debug, Clone, Default)]
pub struct Keymap {
    pub mappings: HashMap<Mode, HashMap<KeyCombination, Action>>,
}

impl Keymap {
    pub fn new() -> Self {
        let mut keymap = Self::default();
        keymap.add_defaults();
        keymap
    }

    fn add_defaults(&mut self) {
        use Mode::*;
        use Action::*;

        let normal = self.mappings.entry(Normal).or_default();
        normal.insert(KeyCombination::from_str("j").unwrap(), MoveDown);
        normal.insert(KeyCombination::from_str("k").unwrap(), MoveUp);
        normal.insert(KeyCombination::from_str("G").unwrap(), MoveToBottom);
        normal.insert(KeyCombination::from_str("ctrl-d").unwrap(), PageDown);
        normal.insert(KeyCombination::from_str("ctrl-u").unwrap(), PageUp);
        normal.insert(KeyCombination::from_str("i").unwrap(), EnterInsert);
        normal.insert(KeyCombination::from_str("o").unwrap(), EnterInsertBelow);
        normal.insert(KeyCombination::from_str("O").unwrap(), EnterInsertAbove);
        normal.insert(KeyCombination::from_str("d").unwrap(), Delete);
        normal.insert(KeyCombination::from_str("v").unwrap(), EnterVisual);
        normal.insert(KeyCombination::from_str(":").unwrap(), EnterCommand);
        normal.insert(KeyCombination::from_str("enter").unwrap(), CycleStatus);
        normal.insert(KeyCombination::from_str("+").unwrap(), IncreasePriority);
        normal.insert(KeyCombination::from_str("-").unwrap(), DecreasePriority);
        normal.insert(KeyCombination::from_str("q").unwrap(), Quit);

        let visual = self.mappings.entry(Visual).or_default();
        visual.insert(KeyCombination::from_str("j").unwrap(), MoveDown);
        visual.insert(KeyCombination::from_str("k").unwrap(), MoveUp);
        visual.insert(KeyCombination::from_str("d").unwrap(), Delete);
        visual.insert(KeyCombination::from_str("esc").unwrap(), Cancel);

        let stats = self.mappings.entry(Stats).or_default();
        stats.insert(KeyCombination::from_str("q").unwrap(), Cancel);
        stats.insert(KeyCombination::from_str("esc").unwrap(), Cancel);
    }

    pub fn get_action(&self, mode: Mode, event: KeyEvent) -> Option<Action> {
        let combo = KeyCombination::from_event(event);
        self.mappings.get(&mode)?.get(&combo).copied()
    }
}
