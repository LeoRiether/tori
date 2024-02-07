use super::{get_modal_chunk, Message, Modal};

use std::{borrow::Cow, mem};

use crossterm::event::{KeyCode, Event};
use std::sync::Mutex;
use tui::{
    layout::Alignment,
    prelude::*,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget},
};

use crate::error::Result;

/// A modal box that asks for user input
#[derive(Debug, Default)]
pub struct InputModal<'t> {
    title: Cow<'t, str>,
    cursor: usize,
    scroll: Mutex<u16>,
    input: String,
    style: Style,
}

impl<'t> InputModal<'t> {
    pub fn new(title: impl Into<Cow<'t, str>>) -> Self {
        Self {
            title: title.into(),
            cursor: 0,
            scroll: Mutex::new(0),
            input: String::default(),
            style: Style::default().fg(Color::LightBlue),
        }
    }

    pub fn set_input(mut self, input: String) -> Self {
        self.input = input;
        self.cursor = self.input.len();
        self
    }

    fn move_cursor(&mut self, x: isize) {
        let inc = |y: usize| (y as isize + x).min(self.input.len() as isize).max(0) as usize;
        self.cursor = inc(self.cursor);

        while !self.input.is_char_boundary(self.cursor) {
            self.cursor = inc(self.cursor);
        }
    }
}

impl<'t> Modal for InputModal<'t> {
    fn apply_style(&mut self, style: Style) {
        self.style = style;
    }

    fn handle_event(&mut self, event: Event) -> Result<Message> {
        use KeyCode::*;
        if let Event::Key(event) = event {
            match event.code {
                Char(c) => {
                    self.input.insert(self.cursor, c);
                    self.move_cursor(1);
                }
                Backspace => {
                    if self.cursor > 0 {
                        self.move_cursor(-1);
                        self.input.remove(self.cursor);
                    }
                }
                Delete => {
                    if self.cursor < self.input.len() {
                        self.input.remove(self.cursor);
                    }
                }
                Left => {
                    self.move_cursor(-1);
                }
                Right => {
                    self.move_cursor(1);
                }
                Home => {
                    self.cursor = 0;
                }
                End => {
                    self.cursor = self.input.len();
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

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let chunk = get_modal_chunk(area);
        let prefix = " ❯ ";
        let scroll = self.calculate_scroll(chunk.width - prefix.len() as u16);

        let block = Block::default()
            .title(self.title.as_ref())
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(self.style);

        // split input as [left, cursor, right]
        let (left, right) = self.input.split_at(self.cursor);
        let mut indices = right.char_indices();
        let (in_cursor, right) = indices
            .next()
            .map(|_| right.split_at(indices.next().map(|(w, _)| w).unwrap_or(right.len())))
            .unwrap_or((" ", ""));

        let paragraph = Paragraph::new(vec![
            Line::from(vec![]), // empty first line
            Line::from(vec![
                Span::styled(prefix, self.style),
                Span::raw(left),
                Span::styled(in_cursor, Style::default().add_modifier(Modifier::REVERSED)),
                Span::raw(right),
            ]),
        ])
        .block(block)
        .scroll((0, scroll))
        .alignment(Alignment::Left);

        Clear.render(chunk, buf);
        paragraph.render(chunk, buf);
    }

    fn mode(&self) -> ! {
        todo!()
    }
}

impl<'t> InputModal<'t> {
    /// Updates and calculates the Paragraph's scroll based on the current cursor and input
    fn calculate_scroll(&self, chunk_width: u16) -> u16 {
        let mut scroll = self.scroll.lock().unwrap();
        if self.cursor as u16 > *scroll + chunk_width - 1 {
            *scroll = self.cursor as u16 + 1 - chunk_width;
        }

        if (self.cursor as u16) <= *scroll {
            *scroll = (self.cursor as u16).saturating_sub(1);
        }

        *scroll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modal_cursor_ascii() {
        let mut modal = InputModal::new("modal cursor");
        modal.input = "Hello World!".into();
        assert_eq!(modal.cursor, 0);

        modal.move_cursor(1);
        assert_eq!(modal.cursor, 1);

        modal.move_cursor(1);
        assert_eq!(modal.cursor, 2);

        modal.move_cursor(-1);
        assert_eq!(modal.cursor, 1);

        modal.move_cursor(-1);
        assert_eq!(modal.cursor, 0);

        modal.move_cursor(-1);
        assert_eq!(modal.cursor, 0);

        modal.move_cursor(1000);
        assert_eq!(modal.cursor, modal.input.len());
    }

    #[test]
    fn test_modal_cursor_unicode() {
        let mut modal = InputModal::new("modal cursor");
        modal.input = "おはよう".into();
        assert_eq!(modal.cursor, 0);

        modal.move_cursor(1);
        assert_eq!(modal.cursor, 3);

        modal.move_cursor(1);
        assert_eq!(modal.cursor, 6);

        modal.move_cursor(-1);
        assert_eq!(modal.cursor, 3);

        modal.move_cursor(-1);
        assert_eq!(modal.cursor, 0);
    }
}
