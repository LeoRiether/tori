use super::{get_modal_chunk, Modal};

use crossterm::event::{Event, KeyCode};
use tui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::{
    error::Result,
    events::{channel::Tx, Action},
};

/// A confirmation modal box that asks for user yes/no input
#[derive(Debug, Default)]
pub struct ConfirmationModal<C> {
    title: String,
    style: Style,
    on_yes: Option<C>,
}

impl<C> ConfirmationModal<C> {
    pub fn new(title: &str) -> Self {
        Self {
            title: format!("\n{} (y/n)", title),
            style: Style::default().fg(Color::LightBlue),
            on_yes: None,
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn on_yes(mut self, action: C) -> Self {
        self.on_yes = Some(action);
        self
    }
}

impl<C> Modal for ConfirmationModal<C>
where
    C: FnOnce() -> Action + Send + Sync,
{
    fn handle_event(&mut self, tx: Tx, event: Event) -> Result<Option<Action>> {
        use KeyCode::*;
        if let Event::Key(event) = event {
            return Ok(match event.code {
                Backspace | Esc | Char('q') | Char('n') | Char('N') => Some(Action::CloseModal),
                Enter | Char('y') | Char('Y') => {
                    tx.send(Action::CloseModal)?;
                    self.on_yes.take().map(|f| f())
                }
                _ => None,
            });
        }
        Ok(None)
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
}
