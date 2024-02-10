use crate::{
    config::shortcuts::InputStr,
    error::Result,
    events::{channel::Tx, Action},
};
use crossterm::event::{Event, KeyCode};
use tui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use super::Modal;

/// Shows what keys the user is pressing
#[derive(Debug, Default)]
pub struct HotkeyModal {
    text: String,
}

impl Modal for HotkeyModal {
    fn handle_event(&mut self, tx: Tx, event: Event) -> Result<Option<Action>> {
        if let Event::Key(key) = event {
            if let KeyCode::Esc = key.code {
                return Ok(Some(Action::CloseModal));
            }
            self.text = InputStr::from(key).0;
        }
        Ok(None)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let mut chunk = super::get_modal_chunk(area);
        chunk.width = chunk.width.min(30);
        chunk.x = area.width.saturating_sub(chunk.width) / 2;

        let block = Block::default()
            .title(" Hotkey ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue));

        let paragraph = Paragraph::new(format!("\n{}", self.text))
            .block(block)
            .alignment(Alignment::Center);

        Clear.render(chunk, buf);
        paragraph.render(chunk, buf);
    }
}
