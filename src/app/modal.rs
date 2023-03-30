use std::{error::Error, mem};

use crossterm::event::KeyCode;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::events::Event;

use super::{Mode, MyBackend};

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

/// A modal box that asks for user input
#[derive(Debug, Default)]
pub struct Modal {
    title: String,
    input: String,
    style: Style,
}

impl Modal {
    pub fn new(title: String) -> Self {
        Self {
            title,
            input: String::new(),
            style: Style::default().fg(Color::LightBlue),
        }
    }

    pub fn with_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn handle_event(&mut self, event: Event) -> Result<Message, Box<dyn Error>> {
        use Event::*;
        use KeyCode::*;
        if let Terminal(crossterm::event::Event::Key(event)) = event {
            match event.code {
                Char(c) => self.input.push(c),
                Backspace => {
                    self.input.pop();
                }
                Esc => {
                    self.input.clear();
                    return Ok(Message::Quit);
                }
                Enter => {
                    let input = mem::take(&mut self.input);
                    return Ok(Message::Commit(input));
                }
                _ => {}
            }
        }
        Ok(Message::Nothing)
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
            .title(self.title.as_str())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(self.style);

        let paragraph = Paragraph::new(format!("\n{}", self.input))
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(Clear, chunk);
        frame.render_widget(paragraph, chunk);
    }

    pub fn mode(&self) -> Mode {
        Mode::Insert
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::Event;
    use crossterm::event::{
        Event::Key,
        KeyCode::{Backspace, Char, Enter, Esc},
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
        let mut modal = Modal::new("commit lifecycle".into());
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
            modal
                .handle_event(Event::Terminal(key_event(Enter)))
                .ok(),
            Some(Message::Commit("hi".into()))
        );
    }

    #[test]
    fn test_modal_quit_lifecycle() {
        let mut modal = Modal::new("commit lifecycle".into());
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
                .handle_event(Event::Terminal(key_event(Esc)))
                .ok(),
            Some(Message::Quit)
        );
    }
}
