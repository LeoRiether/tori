use std::{collections::HashMap, error::Error};

use crossterm::event::{KeyCode, KeyModifiers};

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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

/// Stores a table of [Command](app/event_channel/enum.Command.html) shortcuts.
#[derive(Default)]
pub struct Shortcuts {
    pub normal: HashMap<InputStr, crate::command::Command>,
}

impl Shortcuts {
    /// Loads the shortcuts from the default path, which is
    /// [dirs::config_dir](https://docs.rs/dirs/latest/dirs/fn.config_dir.html)/tori.yaml
    pub fn from_default_location() -> Result<Self, Box<dyn Error>> {
        let path = dirs::config_dir().unwrap_or_default().join("tori.yaml");
        Self::from_path(path)
    }

    /// Loads the shortcuts from some path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let config: serde_yaml::Value = serde_yaml::from_reader(std::fs::File::open(path)?)?;
        let table = config.as_mapping().ok_or("Config yaml is not a mapping")?;

        let normal = {
            let map = table
                .get(serde_yaml::Value::String("normal".into()))
                .unwrap()
                .as_mapping()
                .unwrap();

            let res: Result<HashMap<_, _>, Box<dyn Error>> = map
                .iter()
                .map(|(k, v)| {
                    let k = k.as_str().unwrap().into();
                    let v = v.as_str().unwrap();
                    let v = v
                        .parse::<crate::command::Command>()
                        .map_err(|_| format!("Unrecognized command in config yaml: {}", v))?;
                    Ok((InputStr(k), v))
                })
                .collect();

            res?
        };

        Ok(Self { normal })
    }

    pub fn get_from_event(
        &self,
        event: crossterm::event::KeyEvent,
    ) -> Option<crate::command::Command> {
        self.normal.get(&event.into()).cloned()
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
