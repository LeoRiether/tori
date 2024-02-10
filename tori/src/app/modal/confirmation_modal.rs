use super::{get_modal_chunk, Message, Modal};

use crossterm::event::{Event, KeyCode};
use tui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::error::Result;

/// A confirmation modal box that asks for user yes/no input
#[derive(Debug, Default)]
pub struct ConfirmationModal {
    title: String,
    style: Style,
}

impl ConfirmationModal {
    pub fn new(title: &str) -> Self {
        Self {
            title: format!("\n{} (y/n)", title),
            style: Style::default().fg(Color::LightBlue),
        }
    }
}

impl Modal for ConfirmationModal {
    fn apply_style(&mut self, style: Style) {
        self.style = style;
    }

    fn handle_event(&mut self, event: Event) -> Result<Message> {
        use KeyCode::*;
        if let Event::Key(event) = event {
            return match event.code {
                Backspace | Esc | Char('q') | Char('n') | Char('N') => Ok(Message::Quit),
                Enter | Char('y') | Char('Y') => Ok(Message::Commit("y".into())),
                _ => Ok(Message::Nothing),
            };
        }
        Ok(Message::Nothing)
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let chunk = get_modal_chunk(area);

        let block = Block::default()
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(self.style);

        let paragraph = Paragraph::new(self.title.as_str())
            .block(block)
            .style(self.style)
            .alignment(Alignment::Center);

        Clear.render(chunk, buf);
        paragraph.render(chunk, buf);
    }

    fn mode(&self) -> ! {
        todo!()
    }
}
