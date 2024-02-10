use crate::{
    app::component::Mode,
    config::shortcuts::InputStr,
    error::Result,
    events::Event,
};
use crossterm::event::Event as CrosstermEvent;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
    prelude::*,
};

use super::Modal;

///////////////////////////////
//        HotkeyModal        //
///////////////////////////////
/// Shows what keys the user is pressing
#[derive(Debug, Default)]
pub struct HotkeyModal {
    text: String,
}

impl Modal for HotkeyModal {
    fn apply_style(&mut self, _style: Style) {}

    fn handle_event(&mut self, event: Event) -> Result<super::Message> {
        if let Event::Terminal(CrosstermEvent::Key(key)) = event {
            if let crossterm::event::KeyCode::Esc = key.code {
                return Ok(super::Message::Quit);
            }
            self.text = InputStr::from(key).0;
        }
        Ok(super::Message::Nothing)
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

    fn mode(&self) -> Mode {
        Mode::Insert
    }
}
