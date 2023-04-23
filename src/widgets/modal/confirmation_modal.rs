use super::{get_modal_chunk, Message, Modal};

use crossterm::event::KeyCode;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::{
    error::Result,
    app::component::{Mode, MyBackend},
    events::Event,
};

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
        use Event::*;
        use KeyCode::*;
        if let Terminal(crossterm::event::Event::Key(event)) = event {
            return match event.code {
                Backspace | Esc | Char('q') | Char('n') | Char('N') => Ok(Message::Quit),
                Enter | Char('y') | Char('Y') => Ok(Message::Commit("y".into())),
                _ => Ok(Message::Nothing),
            };
        }
        Ok(Message::Nothing)
    }

    fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        let size = frame.size();
        let chunk = get_modal_chunk(size);

        let block = Block::default()
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(self.style);

        let paragraph = Paragraph::new(self.title.as_str())
            .block(block)
            .style(self.style)
            .alignment(Alignment::Center);

        frame.render_widget(Clear, chunk);
        frame.render_widget(paragraph, chunk);
    }

    fn mode(&self) -> Mode {
        Mode::Insert
    }
}
