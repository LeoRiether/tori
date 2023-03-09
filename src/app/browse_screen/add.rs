use std::{error::Error, mem};

use crossterm::event::{Event, KeyCode};
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{MyBackend, Mode};

#[derive(Debug, Default)]
pub enum AddState {
    #[default]
    Waiting,
    Selected(String),
    Canceled,
}

#[derive(Debug, Default)]
pub struct AddPane {
    pub path: String,
    pub state: AddState,
}

impl AddPane {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::single_match)]
    pub fn handle_event(&mut self, event: Event) -> Result<(), Box<dyn Error>> {
        use KeyCode::*;
        match event {
            Event::Key(event) => {
                match event.code {
                    Char(c) => self.path.push(c),
                    Backspace => {
                        self.path.pop();
                    }
                    Esc => {
                        self.path.clear();
                        self.state = AddState::Canceled;
                    }
                    Enter => {
                        self.state = AddState::Selected(mem::take(&mut self.path));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        let size = frame.size();
        let chunk = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(48),
                Constraint::Length(5),
                Constraint::Percentage(48),
            ])
            .split(size)[1];
        let chunk = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Ratio(1, 4),
                Constraint::Ratio(2, 4),
                Constraint::Ratio(1, 4),
            ])
            .split(chunk)[1];

        let block = Block::default()
            .title(" Add Song ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::LightBlue));

        let paragraph = Paragraph::new(format!("\n{}", self.path))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(Clear, chunk);
        frame.render_widget(paragraph, chunk);
    }

    pub fn mode(&self) -> Mode {
        Mode::Insert
    }
}
