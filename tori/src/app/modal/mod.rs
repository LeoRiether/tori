pub mod confirmation_modal;
pub mod help_modal;
pub mod hotkey_modal;
pub mod input_modal;

pub use confirmation_modal::ConfirmationModal;
pub use help_modal::HelpModal;
pub use hotkey_modal::HotkeyModal;
pub use input_modal::InputModal;

use crate::{
    error::Result,
    events::{channel::Tx, Action},
};
use crossterm::event::Event;
use tui::{layout::Rect, prelude::*};

pub trait Modal
where
    Self: Sync + Send,
{
    fn handle_event(&mut self, tx: Tx, event: Event) -> Result<Option<Action>>;
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn some_box(self) -> Option<Box<dyn Modal>>
    where
        Self: Sized + 'static,
    {
        Some(Box::new(self))
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
