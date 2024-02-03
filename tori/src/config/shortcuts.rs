use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

use crate::events;

/// Encapsulates a string representing some key event.
///
/// For example:
/// ```
/// use tori::config::shortcuts::InputStr;
/// use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
///
/// fn key_event(modifiers: KeyModifiers, code: KeyCode) -> KeyEvent {
///     KeyEvent {
///         code,
///         modifiers,
///         kind: KeyEventKind::Press,
///         state: KeyEventState::NONE,
///     }
/// }
///
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::NONE, KeyCode::Char('a'))),
///     InputStr("a".into())
/// );
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::CONTROL, KeyCode::Char('a'))),
///     InputStr("C-a".into())
/// );
/// assert_eq!(
///     InputStr::from(key_event(KeyModifiers::SHIFT, KeyCode::Char('B'))),
///     InputStr("B".into())
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
pub struct InputStr(pub String);

impl From<crossterm::event::KeyEvent> for InputStr {
    fn from(event: crossterm::event::KeyEvent) -> Self {
        let mut s = String::new();
        let is_char = |e: crossterm::event::KeyEvent| matches!(e.code, KeyCode::Char(_));

        // Modifiers
        if event.modifiers & KeyModifiers::CONTROL != KeyModifiers::NONE {
            s.push_str("C-");
        }
        if event.modifiers & KeyModifiers::SHIFT != KeyModifiers::NONE && !is_char(event) {
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
pub struct Shortcuts(pub HashMap<InputStr, events::Command>);

impl Shortcuts {
    pub fn new(map: HashMap<InputStr, events::Command>) -> Self {
        Self(map)
    }

    pub fn get_from_event(
        &self,
        event: crossterm::event::KeyEvent,
    ) -> Option<events::Command> {
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
            InputStr::from(key_event(KeyModifiers::SHIFT, KeyCode::Char('B'))),
            InputStr("B".into())
        );
        assert_eq!(
            // ctrl+shift+1, but shift+1 is !
            InputStr::from(key_event(
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                KeyCode::Char('!')
            )),
            InputStr("C-!".into())
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
