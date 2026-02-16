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
        if s == "-" {
            return Some(Self {
                code: KeyCode::Char('-'),
                modifiers: KeyModifiers::empty(),
            });
        }

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
        let defaults = [
            ("j", MoveDown),
            ("k", MoveUp),
            ("G", MoveToBottom),
            ("ctrl-d", PageDown),
            ("ctrl-u", PageUp),
            ("i", EnterInsert),
            ("o", EnterInsertBelow),
            ("O", EnterInsertAbove),
            ("d", Delete),
            ("v", EnterVisual),
            ("r", EnterInsert),
            (":", EnterCommand),
            ("enter", CycleStatus),
            ("+", IncreasePriority),
            ("-", DecreasePriority),
            ("u", Undo),
            ("ctrl-r", Redo),
            ("p", Paste),
            ("y", Yank), // For single y if needed, but yy handled in tui
            ("q", Quit),
        ];

        for (key, action) in defaults {
            let combo = KeyCombination::from_str(key)
                .expect(&format!("Failed to parse default keybinding: {}", key));
            normal.insert(combo, action);
        }

        let visual = self.mappings.entry(Visual).or_default();
        let visual_defaults = [
            ("j", MoveDown),
            ("k", MoveUp),
            ("d", Delete),
            ("esc", Cancel),
        ];

        for (key, action) in visual_defaults {
            let combo = KeyCombination::from_str(key)
                .expect(&format!("Failed to parse visual keybinding: {}", key));
            visual.insert(combo, action);
        }

        let stats = self.mappings.entry(Stats).or_default();
        let stats_defaults = [
            ("q", Cancel),
            ("esc", Cancel),
        ];

        for (key, action) in stats_defaults {
            let combo = KeyCombination::from_str(key)
                .expect(&format!("Failed to parse stats keybinding: {}", key));
            stats.insert(combo, action);
        }
    }

    pub fn get_action(&self, mode: Mode, event: KeyEvent) -> Option<Action> {
        let combo = KeyCombination::from_event(event);
        self.mappings.get(&mode)?.get(&combo).copied()
    }
}
