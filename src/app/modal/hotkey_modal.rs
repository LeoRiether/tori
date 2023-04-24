use crate::{
    app::component::{Mode, MyBackend},
    error::Result,
    events::Event, config::shortcuts::InputStr,
};
use tui::{layout::Alignment, widgets::{Block, Borders, BorderType, Paragraph, Clear}, style::{Color, Style}};
use crossterm::event::Event as CrosstermEvent;

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
    fn apply_style(&mut self, _style: Style) { }

    fn handle_event(&mut self, event: Event) -> Result<super::Message> {
        if let Event::Terminal(CrosstermEvent::Key(key)) = event {
            if let crossterm::event::KeyCode::Esc = key.code {
                return Ok(super::Message::Quit);
            }
            self.text = InputStr::from(key).0;
        }
        Ok(super::Message::Nothing)
    }

    fn render(&mut self, frame: &mut tui::Frame<'_, MyBackend>) {
        let chunk = super::get_modal_chunk(frame.size());

        let block = Block::default()
            .title(" Hotkey ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::LightBlue));

        let paragraph = Paragraph::new(format!("\n{}", self.text))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(Clear, chunk);
        frame.render_widget(paragraph, chunk);
    }

    fn mode(&self) -> Mode {
        Mode::Insert
    }
}
