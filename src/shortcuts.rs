use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

/// Encapsulates a string representing some key event.
///
/// For example:
/// ```
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::NONE, KeyCode::Char('a'))),
///     InputStr("a".into())
/// );
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::CONTROL, KeyCode::Char('a'))),
///     InputStr("C-a".into())
/// );
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::SHIFT, KeyCode::Char('b'))),
///     InputStr("S-b".into())
/// );
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::ALT, KeyCode::Enter)),
///     InputStr("A-enter".into())
/// );
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::CONTROL | KeyModifiers::SHIFT, KeyCode::Tab)),
///     InputStr("C-S-tab".into())
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct InputStr(String);

impl From<crossterm::event::KeyEvent> for InputStr {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        let mut s = String::new();

        // Modifiers
        if event.modifiers & KeyModifiers::CONTROL != KeyModifiers::NONE {
            s.push_str("C-");
        }
        if event.modifiers & KeyModifiers::SHIFT != KeyModifiers::NONE {
            s.push_str("S-");
        }
        if event.modifiers & KeyModifiers::ALT != KeyModifiers::NONE {
            s.push_str("A-");
        }

        // Actual key
        use KeyCode::*;
        match event.code {
            Char(c) => {
                s.push(c);
            }
            other => s.push_str(&format!("{:?}", other).to_lowercase()),
        }

        InputStr(s)
    }
}

/// Stores a table of [Command](crate::command::Command) shortcuts.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Shortcuts(HashMap<InputStr, crate::command::Command>);

impl Shortcuts {
    pub fn get_from_event(
        &self,
        event: crossterm::event::KeyEvent,
    ) -> Option<crate::command::Command> {
        self.0.get(&event.into()).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key_event(modifiers: KeyModifiers, code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn test_input_str() {
        assert_eq!(
            InputStr::from(key_event(KeyModifiers::NONE, KeyCode::Char('a'))),
            InputStr("a".into())
        );
        assert_eq!(
            InputStr::from(key_event(KeyModifiers::CONTROL, KeyCode::Char('a'))),
            InputStr("C-a".into())
        );
        assert_eq!(
            InputStr::from(key_event(KeyModifiers::SHIFT, KeyCode::Char('b'))),
            InputStr("S-b".into())
        );
        assert_eq!(
            InputStr::from(key_event(KeyModifiers::ALT, KeyCode::Enter)),
            InputStr("A-enter".into())
        );
        assert_eq!(
            InputStr::from(key_event(
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                KeyCode::Tab
            )),
            InputStr("C-S-tab".into())
        );
    }
}
