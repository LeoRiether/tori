pub mod confirmation_modal;
pub mod help_modal;
pub mod hotkey_modal;
pub mod input_modal;

pub use confirmation_modal::ConfirmationModal;
pub use help_modal::HelpModal;
pub use hotkey_modal::HotkeyModal;
pub use input_modal::InputModal;

use tui::{layout::Rect, style::Style, prelude::*};

use crate::{
    app::component::Mode,
    error::Result,
    events::Event,
};

///////////////////////////////////////////////////
//                    Message                    //
///////////////////////////////////////////////////
/// The return type for [Modal::handle_event].
#[derive(Debug, Default, PartialEq)]
pub enum Message {
    /// Nothing changed
    #[default]
    Nothing,

    /// User has quit the modal (by pressing Esc)
    Quit,

    /// User has written something (the String) in the modal and pressed Enter
    Commit(String),
}

/////////////////////////////////////////////////
//                    Modal                    //
/////////////////////////////////////////////////
pub trait Modal {
    fn apply_style(&mut self, style: Style);
    fn handle_event(&mut self, event: Event) -> Result<Message>;
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn mode(&self) -> Mode;
}

impl Default for Box<dyn Modal> {
    fn default() -> Self {
        Box::new(InputModal::new(String::default()))
    }
}

pub fn get_modal_chunk(frame: Rect) -> Rect {
    let width = (frame.width / 3).max(70).min(frame.width);
    let height = 5;

    Rect {
        x: frame.width.saturating_sub(width) / 2,
        width,
        y: frame.height.saturating_sub(height) / 2,
        height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Event;
    use crossterm::event::{
        Event::Key,
        KeyCode::{self, Backspace, Char, Enter, Esc},
        KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
    };

    fn key_event(code: KeyCode) -> crossterm::event::Event {
        Key(KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind: KeyEventKind::Press,
            state: KeyEventState::NONE,
        })
    }

    #[test]
    fn test_modal_commit_lifecycle() {
        let mut modal = InputModal::new("commit lifecycle");
        assert_eq!(
            modal
                .handle_event(Event::Terminal(key_event(Char('h'))))
                .ok(),
            Some(Message::Nothing)
        );
        assert_eq!(
            modal
                .handle_event(Event::Terminal(key_event(Char('i'))))
                .ok(),
            Some(Message::Nothing)
        );
        assert_eq!(
            modal
                .handle_event(Event::Terminal(key_event(Char('!'))))
                .ok(),
            Some(Message::Nothing)
        );
        assert_eq!(
            modal
                .handle_event(Event::Terminal(key_event(Backspace)))
                .ok(),
            Some(Message::Nothing)
        );
        assert_eq!(
            modal.handle_event(Event::Terminal(key_event(Enter))).ok(),
            Some(Message::Commit("hi".into()))
        );
    }

    #[test]
    fn test_modal_quit_lifecycle() {
        let mut modal = InputModal::new("commit lifecycle");
        assert_eq!(
            modal
                .handle_event(Event::Terminal(key_event(Char('h'))))
                .ok(),
            Some(Message::Nothing)
        );
        assert_eq!(
            modal
                .handle_event(Event::Terminal(key_event(Char('i'))))
                .ok(),
            Some(Message::Nothing)
        );
        assert_eq!(
            modal.handle_event(Event::Terminal(key_event(Esc))).ok(),
            Some(Message::Quit)
        );
    }
}
