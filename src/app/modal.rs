use std::{borrow::Cow, error::Error, mem};

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

// TODO: maybe I should have a Component trait? With handle_event<T>, render and mode.
// Then modals are simply Component<Message>
pub trait Modal {
    fn apply_style(&mut self, style: Style);
    fn handle_event(&mut self, event: Event) -> Result<Message, Box<dyn Error>>;
    fn render(&mut self, frame: &mut Frame<'_, MyBackend>);
    fn mode(&self) -> Mode;
}

impl Default for Box<dyn Modal> {
    fn default() -> Self {
        Box::new(InputModal::new(String::default()))
    }
}

/// A modal box that asks for user input
#[derive(Debug, Default)]
pub struct InputModal<'t> {
    title: Cow<'t, str>,
    input: String,
    style: Style,
}

impl<'t> InputModal<'t> {
    pub fn new(title: impl Into<Cow<'t, str>>) -> Self {
        Self {
            title: title.into(),
            input: String::new(),
            style: Style::default().fg(Color::LightBlue),
        }
    }
}

impl<'t> Modal for InputModal<'t> {
    fn apply_style(&mut self, style: Style) {
        self.style = style;
    }

    fn handle_event(&mut self, event: Event) -> Result<Message, Box<dyn Error>> {
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

    fn render(&mut self, frame: &mut Frame<'_, MyBackend>) {
        let size = frame.size();
        let chunk = get_modal_chunk(size);

        let block = Block::default()
            .title(self.title.as_ref())
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

    fn mode(&self) -> Mode {
        Mode::Insert
    }
}

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

    fn handle_event(&mut self, event: Event) -> Result<Message, Box<dyn Error>> {
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

fn get_modal_chunk(size: tui::layout::Rect) -> tui::layout::Rect {
    let chunk = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(48),
            Constraint::Length(5),
            Constraint::Percentage(48),
        ])
        .split(size)[1];
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(2, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(chunk)[1]
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
